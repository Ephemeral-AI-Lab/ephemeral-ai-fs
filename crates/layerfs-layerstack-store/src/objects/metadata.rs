//! Append-only physical inode-value groups. Ordinals locate values; the complete
//! group digest and reconstructed canonical inode leaf authenticate them.
use super::{pack, spill};
use crate::{schema::StoreDb, Result, StoreError};
use layerfs_content::{tree::compact, ObjectId};
use rusqlite::{Connection, OptionalExtension};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const VALUES_PER_GROUP: usize = 165;
const VALUE_MAGIC: &[u8; 8] = b"LFSIVL1\0";

#[derive(Clone, Copy)]
pub(super) struct Group {
    pub first: u64,
    pub count: usize,
    pub pack: i64,
    pub number: usize,
    pub digest: ObjectId,
}

impl StoreDb {
    pub(super) fn metadata_group(&self, ordinal: u64) -> Result<Group> {
        if !(1..=u32::MAX as u64).contains(&ordinal) {
            return Err(StoreError::Integrity("metadata ordinal"));
        }
        let row = self
            .reader()?
            .prepare_cached(
                "SELECT first_ordinal,count,pack_id,group_number,digest FROM metadata_value_groups WHERE first_ordinal <= ?1 ORDER BY first_ordinal DESC LIMIT 1",
            )?
            .query_row([ordinal as i64], |row| Ok((row.get::<_, u32>(0)? as u64, row.get::<_, u32>(1)? as usize, row.get::<_, i64>(2)?, row.get::<_, u32>(3)? as usize, row.get::<_, Vec<u8>>(4)?)),
            )
            .optional()?
            .ok_or(StoreError::Integrity("metadata value missing"))?;
        if ordinal == 0 || row.0 + row.1 as u64 <= ordinal || row.1 > VALUES_PER_GROUP {
            return Err(StoreError::Integrity("metadata ordinal range"));
        }
        Ok(Group {
            first: row.0,
            count: row.1,
            pack: row.2,
            number: row.3,
            digest: ObjectId::from_bytes(&row.4)?,
        })
    }

    pub(crate) fn validate_metadata_groups(&self) -> Result<()> {
        if !self.compact_namespace() {
            return Ok(());
        }
        if self.reader()?.query_row("SELECT EXISTS(SELECT 1 FROM objects JOIN metadata_value_groups USING (pack_id,group_number))", [], |row| row.get::<_, bool>(0))? {
            return Err(StoreError::Integrity("metadata pool aliases canonical group"));
        }
        let end = self.next_metadata_ordinal()?;
        let mut next = 1;
        while next < end {
            let group = self.metadata_group(next)?;
            if group.first != next {
                return Err(StoreError::Integrity("metadata catalogue gap"));
            }
            self.read_metadata_values(group)?;
            next += group.count as u64;
        }
        Ok(())
    }

    pub(super) fn next_metadata_ordinal(&self) -> Result<u64> {
        next_ordinal(&*self.reader()?)
    }
}

// If M is the largest start, any start <= M-165 ends <= M, while the last
// group ends > M. Schema bounds therefore limit the exact MAX to <=165 rows,
// even with gaps/overlaps; no full scan or unverified last-row assumption.
const NEXT_ORDINAL_SQL: &str = "SELECT COALESCE(MAX(first_ordinal+count),1)
    FROM metadata_value_groups
    WHERE first_ordinal > (SELECT MAX(first_ordinal) FROM metadata_value_groups)-?1";

pub(super) fn next_ordinal(connection: &Connection) -> Result<u64> {
    let next: i64 = connection
        .prepare_cached(NEXT_ORDINAL_SQL)?
        .query_row([VALUES_PER_GROUP as i64], |row| row.get(0))?;
    if !(1..=1 + i64::from(u32::MAX)).contains(&next) {
        return Err(StoreError::Integrity("metadata ordinal maximum"));
    }
    Ok(next as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cached(first: u64, count: usize) -> Vec<[u8; 73]> {
        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            let mut value = [0u8; 73];
            value[0] = first as u8;
            value[1] = index as u8;
            values.push(value);
        }
        values
    }

    /// The pooled value cache is deliberately shared by every target of one
    /// record-group wave, but the physical work allowance belongs to one
    /// `metadata_chain`. Starting another chain must reset the allowance without
    /// discarding values a sibling target already decoded, or the shared cache
    /// would stop paying for itself and the wave would re-read every group.
    #[test]
    fn pool_work_allowance_is_per_chain_not_per_wave() {
        let mut pool = PoolRead::default();
        pool.groups.insert(1, cached(1, VALUES_PER_GROUP));
        pool.groups
            .insert(1 + VALUES_PER_GROUP as u64, cached(2, 3));
        let retained = pool.retained;
        pool.retained += 73 * 8 + 256;
        pool.decoded_work = 31 * 1024 * 1024;
        assert_eq!(pool.groups.len(), 2);

        pool.begin_chain();
        // The allowance restarts for the new chain...
        assert_eq!(pool.decoded_work, 0);
        // ...and the decoded value cache survives, so the sibling target still
        // finds its groups cached instead of re-reading them.
        assert_eq!(pool.groups.len(), 2);
        assert_eq!(pool.groups[&1].len(), VALUES_PER_GROUP);
        assert_eq!(pool.groups[&(1 + VALUES_PER_GROUP as u64)].len(), 3);
        assert!(pool.retained >= retained);
        // The retention bound is untouched: starting a chain never widens it.
        assert!(pool.retained < 512 * 1024);

        // Re-starting is idempotent and never resurrects a spent allowance.
        pool.decoded_work = 32 * 1024 * 1024;
        pool.begin_chain();
        assert_eq!(pool.decoded_work, 0);
        assert_eq!(pool.groups.len(), 2);
    }

    /// One `metadata_chain` can name at most `METADATA_EDGES + 1` pooled nodes,
    /// each at most 100 rows (`physical_length`), so the per-chain lookup ceiling
    /// is exactly the 1 700 the decoded-work comment claims -- 348 lookups below
    /// the 2 048-unit ceiling a single chain may spend. That is
    /// why the allowance can be owned per chain without weakening any bound.
    #[test]
    fn one_chain_cannot_exceed_the_decoded_work_ceiling() {
        let rows_per_node = (8192 - 44) / 81;
        assert_eq!(rows_per_node, 100);
        assert!(physical_length(44 + rows_per_node * 81).is_ok());
        assert!(physical_length(44 + (rows_per_node + 1) * 81).is_err());
        let maximum_lookups = (super::super::read::METADATA_EDGES + 1) * rows_per_node;
        assert_eq!(maximum_lookups, 1_700);
        assert!(maximum_lookups * 16 * 1024 < 32 * 1024 * 1024);
        // The logical-work guard in `expand` is charged per call, so a single
        // call can never exceed 192 KiB / 94 B either.
        assert!(rows_per_node * 94 < 192 * 1024);
        let mut leaf = [0u8; 44];
        leaf[13..21].copy_from_slice(b"LFS6INT\0");
        leaf[23] = 7;
        assert!(super::super::read::metadata_leaf(&leaf));
    }

    /// A pooled leaf row addresses a value group, and a miss costs one 16-KiB
    /// unit however many rows share that group. Rows spaced one full group apart
    /// therefore cost one unit each -- the recipe a scattered-ordinal corpus
    /// needs, and the reason such a corpus needs `VALUES_PER_GROUP` distinct
    /// admitted values per charged unit.
    #[test]
    fn group_lookup_spacing_controls_the_miss_count() {
        assert_eq!(VALUES_PER_GROUP, 165);
        let same = [1u64, 2];
        let spaced = [1u64, 1 + VALUES_PER_GROUP as u64];
        assert_eq!(
            same.iter()
                .map(|ordinal| ordinal / VALUES_PER_GROUP as u64)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            1
        );
        assert_eq!(
            spaced
                .iter()
                .map(|ordinal| ordinal / VALUES_PER_GROUP as u64)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            2
        );
    }
}

#[cfg(test)]
#[path = "metadata_endpoint_tests.rs"]
mod endpoint_tests;

const INDEX_VALUES: usize = 131_072;

// Only a candidate filter. Full authenticated bytes decide equality below.
fn fingerprint(value: &[u8; 73]) -> i64 {
    i64::from_le_bytes(
        ObjectId::for_bytes(value).as_bytes()[..8]
            .try_into()
            .unwrap(),
    )
}

#[cfg(test)]
#[path = "metadata_fingerprint_tests.rs"]
mod fingerprint_tests;

pub(crate) struct ValueIndex {
    // Reuse the existing bounded macOS scratch database and cleanup owner. This
    // is disposable derivation, never a required Store index or recovery source.
    connection: Connection,
    _path: spill::TempPath,
    next: u64,
    entries: usize,
}

impl ValueIndex {
    pub(super) fn new() -> Result<Self> {
        let (connection, path) = spill::scratch_index("metadata-value-index",
            "PRAGMA page_size=4096; PRAGMA max_page_count=8192; CREATE TABLE values_by_fingerprint(fingerprint INTEGER NOT NULL, ordinal INTEGER NOT NULL, PRIMARY KEY(fingerprint,ordinal)) WITHOUT ROWID;")?;
        Ok(Self {
            connection,
            _path: path,
            next: 1,
            entries: 0,
        })
    }

    pub(super) fn sync(&mut self, db: &StoreDb) -> Result<()> {
        let started = std::time::Instant::now();
        let end = db.next_metadata_ordinal()?;
        if end < self.next {
            return Err(StoreError::Integrity("metadata index chronology"));
        }
        if self.next == 1 && end > 1 + INDEX_VALUES as u64 {
            self.next = Self::retained_start(db, end)?;
        }
        self.sync_to(db, end)?;
        db.note_physical(crate::PhysicalStorageReceipt {
            metadata_index_sync_ns: super::elapsed_ns(started),
            ..Default::default()
        });
        Ok(())
    }

    // Reproduce the exact whole-group eviction recurrence before reading payloads
    // that would immediately be discarded. ponytail: the header audit is O(groups);
    // payload authentication/insertion after reopen is bounded to INDEX_VALUES.
    // Complete catalogue authentication remains in validate_metadata_groups, and
    // every value actually reused still passes find_batch's authenticated check.
    fn retained_start(db: &StoreDb, end: u64) -> Result<u64> {
        let connection = db.reader()?;
        let mut statement = connection.prepare_cached(
            "SELECT first_ordinal,count FROM metadata_value_groups ORDER BY first_ordinal",
        )?;
        let mut rows = statement.query([])?;
        let mut next = 1u64;
        let mut first = 1u64;
        let mut entries = 0usize;
        while let Some(row) = rows.next()? {
            let ordinal = u64::from(row.get::<_, u32>(0)?);
            let count = row.get::<_, u32>(1)? as usize;
            if ordinal != next || !(1..=VALUES_PER_GROUP).contains(&count) {
                return Err(StoreError::Integrity("metadata catalogue gap/range"));
            }
            next = next
                .checked_add(count as u64)
                .filter(|next| *next <= 1 + u64::from(u32::MAX))
                .ok_or(StoreError::Integrity("metadata ordinal maximum"))?;
            if entries + count > INDEX_VALUES {
                first = ordinal;
                entries = 0;
            }
            entries += count;
        }
        if next != end {
            return Err(StoreError::Integrity("metadata catalogue endpoint"));
        }
        Ok(first)
    }

    fn sync_to(&mut self, db: &StoreDb, end: u64) -> Result<()> {
        if self.next < end {
            // One scratch transaction per synchronization: every group in this
            // call commits together or nothing does. Cursor and entry counts
            // advance inside the transaction; a failed commit leaves the whole
            // index to the existing publication-rollback invalidation path.
            let transaction = self.connection.transaction()?;
            {
                // One compiled INSERT per existing transaction; dropped before commit.
                let mut insert = transaction.prepare_cached(
                    "INSERT OR IGNORE INTO values_by_fingerprint(fingerprint,ordinal) VALUES (?1,?2)",
                )?;
                while self.next < end {
                    let group = db.metadata_group(self.next)?;
                    if group.first != self.next {
                        return Err(StoreError::Integrity("metadata catalogue gap"));
                    }
                    let values = db.read_metadata_values(group)?;
                    // ponytail: retain at most 131072 indexed values (32-MiB scratch
                    // file, 4-MiB SQLite cache); use partitioned lookup only if longer
                    // histories justify its cost. Eviction causes duplicate physical
                    // values, not loss.
                    if self.entries + values.len() > INDEX_VALUES {
                        transaction.execute("DELETE FROM values_by_fingerprint", [])?;
                        self.entries = 0;
                    }
                    let fingerprints = values.iter().map(fingerprint).collect::<Vec<_>>();
                    for (index, value) in fingerprints.iter().enumerate() {
                        insert.execute(rusqlite::params![
                            *value,
                            (group.first + index as u64) as i64
                        ])?;
                    }
                    self.entries += values.len();
                    self.next += values.len() as u64;
                }
            }
            transaction.commit()?;
        }
        Ok(())
    }

    /// Fingerprints filter candidates; authenticated complete values choose the
    /// same minimum ordinal as the old exact-key index, including collisions.
    pub(super) fn find_batch(
        &self,
        db: &StoreDb,
        values: &[[u8; 73]],
    ) -> Result<BTreeMap<[u8; 73], u32>> {
        let mut found = BTreeMap::new();
        if values.is_empty() {
            return Ok(found);
        }
        // Charge both exact sets/maps, fingerprint vector, the maximum candidate
        // vector and one decoded group to the existing bounded index allowance.
        if values
            .len()
            .saturating_mul(520)
            .saturating_add(INDEX_VALUES * 4 + 16 * 1024 + super::read::VALIDATION_RESERVE)
            > super::CANDIDATE_INDEX_BYTES
        {
            return Err(StoreError::Integrity("metadata fingerprint query bound"));
        }
        let wanted = values.iter().copied().collect::<BTreeSet<_>>();
        // Global deduplication prevents hash collisions crossing SQL pages from
        // multiplying candidate visits by the number of requested values.
        let hashes = wanted
            .iter()
            .map(fingerprint)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let mut candidates = Vec::<u32>::new();
        let parameters = usize::try_from(
            self.connection
                .limit(rusqlite::limits::Limit::SQLITE_LIMIT_VARIABLE_NUMBER)?,
        )
        .unwrap_or(0);
        let page = hashes.len().min(512).min(parameters.max(1));
        for chunk in hashes.chunks(page) {
            let mut sql =
                String::from("SELECT ordinal FROM values_by_fingerprint WHERE fingerprint IN (?1");
            for parameter in 2..=page {
                sql.push(',');
                sql.push('?');
                sql.push_str(&parameter.to_string());
            }
            sql.push(')');
            let mut statement = self.connection.prepare_cached(&sql)?;
            for (index, value) in chunk
                .iter()
                .chain(std::iter::repeat(&chunk[chunk.len() - 1]))
                .take(page)
                .enumerate()
            {
                statement.raw_bind_parameter(index + 1, *value)?;
            }
            let mut rows = statement.raw_query();
            loop {
                let next = rows.next()?;
                let Some(row) = next else {
                    break;
                };
                let ordinal: u32 = row.get(0)?;
                if candidates.len() == INDEX_VALUES
                    || u64::from(ordinal) >= self.next
                    || u64::from(ordinal) < self.next - self.entries as u64
                {
                    return Err(StoreError::Integrity("metadata fingerprint ordinal bound"));
                }
                candidates.push(ordinal);
            }
            drop(rows);
        }
        candidates.sort_unstable();
        candidates.dedup();
        let mut group_values: Option<(u64, Vec<[u8; 73]>)> = None;
        for ordinal in candidates {
            let ordinal64 = u64::from(ordinal);
            if group_values
                .as_ref()
                .is_none_or(|(first, values)| ordinal64 >= first + values.len() as u64)
            {
                let group = db.metadata_group(ordinal64)?;
                group_values = Some((group.first, db.read_metadata_values(group)?));
            }
            let (first, values) = group_values.as_ref().unwrap();
            let value = values[(ordinal64 - first) as usize];
            if wanted.contains(&value) {
                found.entry(value).or_insert(ordinal);
            }
            if found.len() == wanted.len() {
                break;
            }
        }
        Ok(found)
    }
}

pub(super) fn physical_length(canonical_length: usize) -> Result<usize> {
    let payload = canonical_length
        .checked_sub(44)
        .ok_or(StoreError::Integrity("pooled leaf length"))?;
    if payload == 0 || payload % 81 != 0 || payload / 81 > 100 {
        return Err(StoreError::Integrity("pooled leaf count"));
    }
    Ok(44 + payload / 81 * 12)
}

pub(super) fn value_canonical(value: &[u8; 73]) -> Result<Vec<u8>> {
    compact::decode_inode_value(value)?;
    let mut bytes = Vec::with_capacity(81);
    bytes.extend_from_slice(VALUE_MAGIC);
    bytes.extend_from_slice(value);
    Ok(layerfs_content::encode_bytes_object(&bytes)?)
}

pub(super) fn decode_values(body: &[u8], group: Group) -> Result<Vec<[u8; 73]>> {
    if body.len() > 16 * 1024 || ObjectId::for_bytes(body) != group.digest {
        return Err(StoreError::Integrity("metadata value group identity"));
    }
    // The digest covers count, offsets and every canonical value before parsing.
    let mut values = Vec::with_capacity(group.count);
    pack::visit_records(body, false, |_, record| {
        let pack::Record::Full(canonical) = record else {
            return Err(StoreError::Integrity("metadata value group FULL role"));
        };
        let value = layerfs_content::decode_bytes_object(canonical)?;
        if value.len() != 81 || &value[..8] != VALUE_MAGIC || values.len() == group.count {
            return Err(StoreError::Integrity("metadata value group record"));
        }
        let record: [u8; 73] = value[8..].try_into().unwrap();
        compact::decode_inode_value(&record)?;
        values.push(record);
        Ok(())
    })?;
    if values.len() != group.count {
        return Err(StoreError::Integrity("metadata value group count"));
    }
    Ok(values)
}

#[derive(Default)]
pub(super) struct PoolRead {
    groups: BTreeMap<u64, Vec<[u8; 73]>>,
    retained: usize,
    /// Physical value lookups already charged to the single metadata chain that
    /// is currently being served. The decoded value cache above is bounded and
    /// reused across every target of one record-group wave, but this work
    /// allowance belongs to one chain: unrelated targets that merely share a
    /// wave must not spend each other's allowance. `begin_chain` resets it, so
    /// the existing ceiling still bounds exactly one chain's lookups.
    decoded_work: usize,
}

impl PoolRead {
    /// Start the work allowance for one metadata chain. Cache contents and the
    /// retention bound are untouched, so a sibling target still reuses values an
    /// earlier target in the same wave already decoded.
    pub(super) fn begin_chain(&mut self) {
        self.decoded_work = 0;
    }

    pub(super) fn expand(
        &mut self,
        db: &StoreDb,
        physical: &[u8],
        canonical_length: usize,
        mut budget: Option<&mut super::read::HintReadBudget>,
    ) -> Result<Option<Vec<u8>>> {
        if physical.len() != physical_length(canonical_length)?
            || !super::read::metadata_leaf(physical)
        {
            return Err(StoreError::Integrity("pooled leaf physical length/role"));
        }
        let mut canonical = Vec::with_capacity(canonical_length);
        canonical.extend_from_slice(&physical[..44]);
        // The logical-work guard stays per call: sharing the decoded value cache
        // across a bounded demand wave must not relax the per-object ceiling.
        let mut logical_work = 0usize;
        for row in physical[44..].chunks_exact(12) {
            logical_work += 94;
            if logical_work > 192 * 1024 {
                return Err(StoreError::Integrity("metadata pool logical work"));
            }
            let ordinal = u32::from_be_bytes(row[8..12].try_into().unwrap()) as u64;
            let cached = self
                .groups
                .range(..=ordinal)
                .next_back()
                .and_then(|(first, values)| values.get((ordinal - first) as usize).copied());
            let value = if let Some(value) = cached {
                value
            } else {
                let group = db.metadata_group(ordinal)?;
                // At most 1700 value lookups per bounded metadata chain. This
                // separate physical-work ceiling includes cache misses and
                // reloads, and is charged to the chain whose `begin_chain` ran
                // most recently -- never to a sibling target of the same wave.
                self.decoded_work += 16 * 1024;
                if self.decoded_work > 32 * 1024 * 1024 {
                    return Err(StoreError::Integrity("metadata pool decoded work"));
                }
                if budget
                    .as_deref_mut()
                    .is_some_and(|budget| !budget.charge_metadata_pool(16 * 1024))
                {
                    return Ok(None);
                }
                let values = db.read_metadata_values(group)?;
                let value = values[(ordinal - group.first) as usize];
                let retained = values.capacity() * 73 + 256;
                if self.retained + retained > 512 * 1024 || self.groups.len() == 128 {
                    self.groups.clear();
                    self.retained = 0;
                }
                self.retained += retained;
                self.groups.insert(group.first, values);
                value
            };
            canonical.extend_from_slice(&row[..8]);
            canonical.extend_from_slice(&value);
        }
        Ok(Some(canonical))
    }
}

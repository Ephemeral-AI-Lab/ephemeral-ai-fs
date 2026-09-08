//! Prepared whole-pack admission, with one probe and one final recheck.
use super::{
    pack, read, AuthenticatedCanonicalObject, ObjectInsertMetrics, ADMISSION_BATCH_BYTES,
    ADMISSION_BATCH_COUNT, OBJECT_PAGE_COUNT,
};
use crate::schema::StoreDb;
use crate::{Result, StoreError};
use layerfs_content::ObjectId;
use rusqlite::{limits::Limit, params_from_iter, types::Value, Transaction, TransactionBehavior};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::time::Instant;

struct PreparedObject {
    id: ObjectId,
    length: usize,
    pack: usize,
    group: usize,
    record: usize,
    canonical: Range<usize>,
    // Compressed records retain the original authenticated comparison operand.
    retained: Option<Vec<u8>>,
}

pub(crate) struct PreparedAdmission {
    packs: Vec<Vec<u8>>,
    objects: Vec<PreparedObject>,
    metrics: ObjectInsertMetrics,
}

impl PreparedAdmission {
    pub(crate) fn prepare_missing(missing: super::MissingBatch) -> Result<Self> {
        let objects = missing.0;
        let length = objects
            .iter()
            .try_fold(0usize, |sum, object| sum.checked_add(object.bytes.len()))
            .ok_or(StoreError::Integrity("admission length overflow"))?;
        let reserve = objects
            .iter()
            .map(|object| read::validation_reserve(object.bytes.len()))
            .sum::<usize>();
        if objects.len() > ADMISSION_BATCH_COUNT
            || length > ADMISSION_BATCH_BYTES
            || reserve > read::VALIDATION_RESERVE
        {
            return Err(StoreError::Integrity("prepared admission bound"));
        }
        let ids = objects.iter().map(|object| object.id).collect::<Vec<_>>();
        let candidates = ids.iter().copied().collect::<BTreeSet<_>>();
        if candidates.len() != ids.len() {
            return Err(StoreError::Integrity("admission duplicate ownership"));
        }
        let metrics = ObjectInsertMetrics {
            submitted_rows: ids.len() as u64,
            ..Default::default()
        };

        let mut prepared = Self {
            packs: Vec::new(),
            objects: Vec::new(),
            metrics,
        };
        prepared.prepare_full(objects)?;
        Ok(prepared)
    }

    fn prepare_full(&mut self, objects: Vec<AuthenticatedCanonicalObject>) -> Result<()> {
        let mut ordinary = Vec::new();
        let mut bytes = 0usize;
        let mut count = 0usize;
        for object in objects {
            if object.bytes.len() + 9 > pack::GROUP_LIMIT {
                self.prepare_ordinary(std::mem::take(&mut ordinary))?;
                bytes = 0;
                count = 0;
                self.prepare_singleton(object)?;
                continue;
            }
            // Charge one directory entry per possible group. This conservative
            // incremental bound avoids growing-prefix group recounts.
            let next = object.bytes.len() + 5 + 20;
            if bytes + next + 16 > pack::PACK_LIMIT || count == pack::RECORD_COUNT_LIMIT {
                self.prepare_ordinary(std::mem::take(&mut ordinary))?;
                bytes = 0;
                count = 0;
            }
            bytes += next;
            count += 1;
            ordinary.push(object);
        }
        self.prepare_ordinary(ordinary)
    }

    fn prepare_ordinary(&mut self, mut objects: Vec<AuthenticatedCanonicalObject>) -> Result<()> {
        if objects.is_empty() {
            return Ok(());
        }
        let mut groups = Vec::<Vec<usize>>::new();
        let mut pending = [Vec::new(), Vec::new()];
        let mut sizes = [4usize, 4usize];
        for (index, object) in objects.iter().enumerate() {
            let content = is_content(&object.bytes)?;
            let role = usize::from(content);
            let target = if content { 32 * 1024 } else { 16 * 1024 };
            let next = 5 + object.bytes.len();
            if !pending[role].is_empty() && sizes[role] + next > target {
                groups.push(std::mem::take(&mut pending[role]));
                sizes[role] = 4;
            }
            pending[role].push(index);
            sizes[role] += next;
        }
        for group in pending {
            if !group.is_empty() {
                groups.push(group);
            }
        }
        let mut encoded = Vec::with_capacity(groups.len());
        for group in &groups {
            let canonical = group
                .iter()
                .map(|index| objects[*index].bytes.as_slice())
                .collect::<Vec<_>>();
            encoded.push(pack::full_group(&canonical)?);
        }
        let pack_index = self.packs.len();
        let mut offset = 16 + 16 * groups.len();
        for (group_number, group) in groups.iter().enumerate() {
            let mut cursor = offset + 4 + 4 * group.len();
            for (record_number, index) in group.iter().enumerate() {
                let object = &mut objects[*index];
                let end = cursor + 1 + object.bytes.len();
                self.objects.push(PreparedObject {
                    id: object.id,
                    length: object.bytes.len(),
                    pack: pack_index,
                    group: group_number,
                    record: record_number,
                    canonical: cursor + 1..end,
                    retained: (encoded[group_number].codec == pack::Codec::Zstandard)
                        .then(|| std::mem::take(&mut object.0.bytes)),
                });
                cursor = end;
            }
            offset += encoded[group_number].bytes.len();
        }
        self.packs.push(pack::assemble(&encoded)?);
        Ok(())
    }

    fn prepare_singleton(&mut self, object: AuthenticatedCanonicalObject) -> Result<()> {
        let id = object.id;
        let length = object.bytes.len();
        let total = length + 41;
        let mut prefix = Vec::with_capacity(41);
        prefix.extend_from_slice(b"LFPACK\0\0\x01\x00\x00\x00\x01\x00\x00\x00");
        for value in [32usize, length + 9, length + 9] {
            prefix.extend_from_slice(&(value as u32).to_le_bytes());
        }
        prefix.extend_from_slice(&[0; 4]);
        prefix.extend_from_slice(&1u32.to_le_bytes());
        prefix.extend_from_slice(&((length + 1) as u32).to_le_bytes());
        prefix.push(0);
        // The existing private temporary-file owner avoids a second resident
        // multi-MiB canonical copy. Its exact-byte digest is held only here.
        let (mut file, path) = super::spill::temporary_file("prepared-pack")?;
        let _path = super::spill::TempPath(path);
        let mut digest = blake3::Hasher::new();
        digest.update(&prefix);
        digest.update(&object.bytes);
        file.write_all(&prefix)?;
        file.write_all(&object.bytes)?;
        drop(object);
        file.seek(SeekFrom::Start(0))?;
        let mut bytes = vec![0; total];
        file.read_exact(&mut bytes)?;
        let mut trailing = [0];
        if file.read(&mut trailing)? != 0 || blake3::hash(&bytes) != digest.finalize() {
            return Err(StoreError::Integrity("prepared pack spool identity"));
        }
        self.objects.push(PreparedObject {
            id,
            length,
            pack: self.packs.len(),
            group: 0,
            record: 0,
            canonical: 41..total,
            retained: None,
        });
        self.packs.push(bytes);
        Ok(())
    }

    pub(crate) fn publish<T>(
        mut self,
        db: &StoreDb,
        statement_number: &mut u64,
        publish: impl FnOnce(&Transaction<'_>, &ObjectInsertMetrics, &mut u64) -> Result<T>,
    ) -> Result<(T, super::AdmissionBatchMetrics)> {
        let ids = self
            .objects
            .iter()
            .map(|object| object.id)
            .collect::<Vec<_>>();
        let supplied = self
            .objects
            .iter()
            .map(|object| {
                (
                    object.id,
                    object
                        .retained
                        .as_deref()
                        .unwrap_or_else(|| &self.packs[object.pack][object.canonical.clone()]),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let _permit = db.enter_operation()?;
        let late = db.object_locations(&ids)?;
        compare(db, &late, &supplied, &mut self.metrics)?;
        let mut winners = vec![Vec::new(); self.packs.len()];
        for object in &self.objects {
            if !late.contains_key(&object.id) {
                winners[object.pack].push(object);
            }
        }
        let started = Instant::now();
        let mut connection = db.writer()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let begin_ns = super::elapsed_ns(started);
        let started = Instant::now();
        self.insert(&transaction, &winners, statement_number)?;
        self.metrics.insert_ns += super::elapsed_ns(started);
        self.metrics.objects = winners.iter().map(|objects| objects.len() as u64).sum();
        self.metrics.bytes = winners
            .iter()
            .flatten()
            .map(|object| object.length as u64)
            .sum();
        self.metrics.returned_ids = self.metrics.objects;
        let result = publish(&transaction, &self.metrics, statement_number)?;
        let started = Instant::now();
        transaction.commit()?;
        Ok((
            result,
            super::AdmissionBatchMetrics {
                insert: self.metrics,
                begin_ns,
                commit_ns: super::elapsed_ns(started),
            },
        ))
    }

    fn insert(
        &self,
        transaction: &Transaction<'_>,
        winners: &[Vec<&PreparedObject>],
        statement_number: &mut u64,
    ) -> Result<()> {
        let mut next: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(pack_id),0) FROM object_packs",
            [],
            |row| row.get(0),
        )?;
        let mut packs = Vec::new();
        let mut locators = Vec::new();
        for (index, objects) in winners.iter().enumerate() {
            if objects.is_empty() {
                continue;
            }
            next = next
                .checked_add(1)
                .filter(|id| *id > 0)
                .ok_or(StoreError::Integrity("pack identity exhausted"))?;
            packs.push((next, self.packs[index].as_slice()));
            for object in objects {
                locators.push((next, *object));
            }
        }
        let pack_rows = sql_rows(transaction, 2, 6)?;
        let blob_limit = usize::try_from(transaction.limit(Limit::SQLITE_LIMIT_LENGTH)?)
            .map_err(|_| StoreError::Integrity("SQLite BLOB limit"))?;
        let mut start = 0;
        while start < packs.len() {
            let mut end = start;
            let mut bytes = 0usize;
            while end < packs.len() && end - start < pack_rows {
                let length = packs[end].1.len();
                if length > blob_limit || length > ADMISSION_BATCH_BYTES + 41 {
                    return Err(StoreError::Integrity("pack INSERT byte limit"));
                }
                // Ordinary statements bind at most 1 MiB of BLOBs. The format's
                // oversized RAW singleton is one separately bounded parameter.
                if end > start && bytes + length > 1024 * 1024 {
                    break;
                }
                bytes += length;
                end += 1;
            }
            let page = &packs[start..end];
            let sql = format!(
                "INSERT INTO object_packs(pack_id,data) VALUES {}",
                vec!["(?,?)"; page.len()].join(",")
            );
            let values = page.iter().flat_map(|(id, bytes)| {
                [id as &dyn rusqlite::ToSql, bytes as &dyn rusqlite::ToSql]
            });
            *statement_number += 1;
            crate::schema::fail_transaction_statement(*statement_number)?;
            if transaction.execute(&sql, params_from_iter(values))? != page.len() {
                return Err(StoreError::Integrity("pack insertion cardinality"));
            }
            start = end;
        }
        let locator_rows = sql_rows(transaction, 5, 12)?;
        for page in locators.chunks(locator_rows) {
            let sql = format!("INSERT INTO objects(object_id,canonical_length,pack_id,group_number,record_number) VALUES {}", vec!["(?,?,?,?,?)"; page.len()].join(","));
            let values = page.iter().flat_map(|(pack, object)| {
                [
                    Value::Blob(object.id.as_bytes().to_vec()),
                    Value::Integer(object.length as i64),
                    Value::Integer(*pack),
                    Value::Integer(object.group as i64),
                    Value::Integer(object.record as i64),
                ]
            });
            *statement_number += 1;
            crate::schema::fail_transaction_statement(*statement_number)?;
            if transaction.execute(&sql, params_from_iter(values))? != page.len() {
                return Err(StoreError::Integrity("locator insertion cardinality"));
            }
        }
        Ok(())
    }
}

fn sql_rows(
    connection: &rusqlite::Connection,
    parameters: usize,
    row_bytes: usize,
) -> Result<usize> {
    let parameters_limit =
        usize::try_from(connection.limit(Limit::SQLITE_LIMIT_VARIABLE_NUMBER)?).unwrap_or(0);
    let sql_limit = usize::try_from(connection.limit(Limit::SQLITE_LIMIT_SQL_LENGTH)?).unwrap_or(0);
    let count = OBJECT_PAGE_COUNT
        .min(parameters_limit / parameters)
        .min(sql_limit.saturating_sub(256) / row_bytes);
    if count == 0 {
        return Err(StoreError::Integrity("SQLite bulk statement limit"));
    }
    Ok(count)
}

fn is_content(canonical: &[u8]) -> Result<bool> {
    if canonical.get(4) == Some(&(layerfs_content::ObjectKind::Directory as u8)) {
        return Ok(false);
    }
    let value = layerfs_content::decode_bytes_object(canonical)?;
    Ok(!matches!(
        value.get(..8),
        Some(
            b"LFS4FSR\0"
                | b"LFS4INT\0"
                | b"LFS4INO\0"
                | b"LFS4DIR\0"
                | b"LFS4NSP\0"
                | b"LFS4MET\0"
                | b"LFS4MAP\0"
                | b"LFS4LNK\0"
        )
    ))
}

pub(super) fn compare(
    db: &StoreDb,
    known: &BTreeMap<ObjectId, read::Location>,
    supplied: &BTreeMap<ObjectId, &[u8]>,
    metrics: &mut ObjectInsertMetrics,
) -> Result<()> {
    let started = Instant::now();
    let mut ordinary = Vec::new();
    for (id, location) in known {
        let canonical = supplied
            .get(id)
            .ok_or(StoreError::Integrity("unexpected membership result"))?;
        if canonical.len() != location.canonical_length {
            return Err(StoreError::Integrity("object length collision"));
        }
        if location.canonical_length > pack::GROUP_LIMIT {
            db.compare_singleton(*id, *location, canonical)?;
        } else {
            ordinary.push((*id, *location));
        }
        metrics.skipped_ids += 1;
        metrics.skipped_bytes += canonical.len() as u64;
    }
    db.visit_locations(&ordinary, |object| {
        if supplied.get(&object.id).copied() != Some(object.bytes.as_slice()) {
            return Err(StoreError::Integrity("object collision"));
        }
        Ok(())
    })?;
    metrics.collision_checks += known.len() as u64;
    metrics.conflict_read_rows += known.len() as u64;
    metrics.conflict_read_bytes += known
        .values()
        .map(|location| location.canonical_length as u64)
        .sum::<u64>();
    metrics.conflict_read_ns += super::elapsed_ns(started);
    Ok(())
}

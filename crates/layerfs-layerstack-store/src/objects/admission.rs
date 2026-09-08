//! Prepared whole-pack admission, with one probe and one final recheck.
use super::diagnostic;
use super::{
    ADMISSION_BATCH_BYTES, ADMISSION_BATCH_COUNT, AuthenticatedCanonicalObject, OBJECT_PAGE_COUNT,
    ObjectInsertMetrics, pack, read,
};
use crate::schema::StoreDb;
use crate::{Result, StoreError};
use layerfs_content::ObjectId;
use rusqlite::{Transaction, TransactionBehavior, limits::Limit, params_from_iter, types::Value};
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
    delta: bool,
    diagnostic_terminal: u8,
}

#[allow(dead_code)]
struct UndiagnosedPreparedObject {
    id: ObjectId,
    length: usize,
    pack: usize,
    group: usize,
    record: usize,
    canonical: Range<usize>,
    retained: Option<Vec<u8>>,
    delta: bool,
}
const _: () = {
    assert!(
        std::mem::size_of::<PreparedObject>() == std::mem::size_of::<UndiagnosedPreparedObject>()
    );
    assert!(
        std::mem::align_of::<PreparedObject>() == std::mem::align_of::<UndiagnosedPreparedObject>()
    );
};

pub(crate) struct PreparedAdmission {
    packs: Vec<Vec<u8>>,
    objects: Vec<PreparedObject>,
    metrics: ObjectInsertMetrics,
}

impl PreparedAdmission {
    pub(crate) fn prepare_missing(db: &StoreDb, missing: super::MissingBatch) -> Result<Self> {
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
        let count = ids.len();
        drop(ids);
        drop(candidates);
        let metrics = ObjectInsertMetrics {
            submitted_rows: count as u64,
            ..Default::default()
        };

        let mut prepared = Self {
            packs: Vec::new(),
            objects: Vec::with_capacity(count),
            metrics,
        };
        let mut stats = crate::PhysicalStorageReceipt::default();
        let result = prepared.prepare_full(db, objects, &mut stats);
        db.note_physical(stats);
        result?;
        Ok(prepared)
    }

    fn prepare_full(
        &mut self,
        db: &StoreDb,
        objects: Vec<AuthenticatedCanonicalObject>,
        stats: &mut crate::PhysicalStorageReceipt,
    ) -> Result<()> {
        let input_associations =
            objects.capacity() * std::mem::size_of::<AuthenticatedCanonicalObject>();
        let mut search = DeltaSearch {
            input_associations,
            ..Default::default()
        };
        let mut ordinary = Vec::new();
        let mut bytes = 0usize;
        let mut count = 0usize;
        for object in objects {
            if object.bytes.len() + 9 > pack::GROUP_LIMIT {
                self.prepare_ordinary(db, std::mem::take(&mut ordinary), &mut search, stats)?;
                bytes = 0;
                count = 0;
                stats.full_alternative_bytes += (object.bytes.len() + 9) as u64;
                stats.selected_encoded_bytes += (object.bytes.len() + 9) as u64;
                self.prepare_singleton(object)?;
                continue;
            }
            // Charge one directory entry per possible group. This conservative
            // incremental bound avoids growing-prefix group recounts.
            let next = object.bytes.len() + 5 + 20;
            if bytes + next + 16 > pack::PACK_LIMIT || count == pack::RECORD_COUNT_LIMIT {
                self.prepare_ordinary(db, std::mem::take(&mut ordinary), &mut search, stats)?;
                bytes = 0;
                count = 0;
            }
            bytes += next;
            count += 1;
            ordinary.push(object);
        }
        self.prepare_ordinary(db, ordinary, &mut search, stats)
    }

    fn prepare_ordinary(
        &mut self,
        db: &StoreDb,
        mut objects: Vec<AuthenticatedCanonicalObject>,
        search: &mut DeltaSearch,
        stats: &mut crate::PhysicalStorageReceipt,
    ) -> Result<()> {
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
        let pack_index = self.packs.len();
        let mut offset = 16 + 16 * groups.len();
        let fixed_associations = search.input_associations
            + self.packs.capacity() * std::mem::size_of::<Vec<u8>>()
            + self.objects.capacity() * std::mem::size_of::<PreparedObject>()
            + objects.capacity() * std::mem::size_of::<AuthenticatedCanonicalObject>()
            + groups.capacity() * std::mem::size_of::<Vec<usize>>()
            + groups
                .iter()
                .map(|group| group.capacity() * std::mem::size_of::<usize>())
                .sum::<usize>()
            + encoded.capacity() * std::mem::size_of::<pack::EncodedGroup>();
        let mut backing = 0usize;
        for (group_number, group) in groups.iter().enumerate() {
            let mut deltas = Vec::with_capacity(group.len());
            // All live delta capacities sum to at most the group's FULL decoded
            // size. Matching scratch is dropped before either codec invocation.
            let associations = fixed_associations
                + group.len()
                    * (std::mem::size_of::<Option<Vec<u8>>>() + std::mem::size_of::<&[u8]>());
            // Worst codec phase: static 1-MiB context, RAW group and complete
            // compressBound output. Optional B additionally keeps A and programs.
            // Canonical comparison operands belong to the other <=6 MiB; count
            // their vector associations here as a conservative duplicate charge.
            let required = backing + associations + 1024 * 1024 + 2 * (pack::GROUP_LIMIT + 1024);
            if required > 2 * 1024 * 1024 {
                return Err(StoreError::Io(std::io::Error::other(
                    "physical encoding reservation",
                )));
            }
            let optional = required + 2 * (pack::GROUP_LIMIT + 1024) <= 2 * 1024 * 1024;
            for index in group {
                let object = &objects[*index];
                let mut terminal = diagnostic::state(object);
                let before_bases = stats.usable_bases;
                let before_fetch = stats.fetch_budget_skips;
                let before_match = stats.match_budget_skips;
                let before_instruction = stats.instruction_budget_skips;
                let before_memory = stats.memory_budget_skips;
                if optional {
                    deltas.push(search.candidate(db, object, stats)?);
                } else {
                    stats.memory_budget_skips += 1;
                    stats.budget_skips += 1;
                    deltas.push(None);
                }
                if terminal != 0 {
                    let has_candidate = deltas.last().unwrap().is_some();
                    let budget = stats.fetch_budget_skips != before_fetch
                        || stats.match_budget_skips != before_match
                        || stats.instruction_budget_skips != before_instruction
                        || stats.memory_budget_skips != before_memory;
                    let base = stats.usable_bases != before_bases;
                    stats.diag_event_base += u64::from(base);
                    stats.diag_event_base_bytes += u64::from(base) * object.bytes.len() as u64;
                    stats.diag_event_budget += u64::from(budget);
                    stats.diag_event_budget_bytes += u64::from(budget) * object.bytes.len() as u64;
                    stats.diag_event_candidate += u64::from(has_candidate);
                    stats.diag_event_candidate_bytes +=
                        u64::from(has_candidate) * object.bytes.len() as u64;
                    if stats.fetch_budget_skips != before_fetch {
                        stats.diag_event_fetch_budget_count += 1;
                        stats.diag_event_fetch_budget_bytes += object.bytes.len() as u64;
                    }
                    if stats.match_budget_skips != before_match {
                        stats.diag_event_match_budget_count += 1;
                        stats.diag_event_match_budget_bytes += object.bytes.len() as u64;
                    }
                    if stats.instruction_budget_skips != before_instruction {
                        stats.diag_event_instruction_budget_count += 1;
                        stats.diag_event_instruction_budget_bytes += object.bytes.len() as u64;
                    }
                    if stats.memory_budget_skips != before_memory {
                        stats.diag_event_memory_budget_count += 1;
                        stats.diag_event_memory_budget_bytes += object.bytes.len() as u64;
                    }
                    if has_candidate {
                        terminal = diagnostic::DELTA;
                    } else if terminal == diagnostic::BASE {
                        terminal = if budget {
                            diagnostic::BUDGET
                        } else if base {
                            diagnostic::NO_DELTA
                        } else {
                            diagnostic::BASE
                        };
                    }
                }
                // Grant credit is consumed by initial CAS before MissingBatch.
                // From here no object can be spilled/rebuffered: reuse this byte
                // for terminal state until the PreparedObject takes ownership.
                stats.diag_invalid += u64::from(objects[*index].1.diagnostic_grants != 0);
                objects[*index].1.diagnostic_grants = terminal;
            }
            let canonical = group
                .iter()
                .map(|index| objects[*index].bytes.as_slice())
                .collect::<Vec<_>>();
            let (selected, mixed) = pack::encode_group(&canonical, &deltas, stats)?;
            let mut cursor = offset + 4 + 4 * group.len();
            for (record_number, index) in group.iter().enumerate() {
                let object = &mut objects[*index];
                let delta = mixed && deltas[record_number].is_some();
                let record_length = if delta {
                    deltas[record_number].as_ref().unwrap().len()
                } else {
                    1 + object.bytes.len()
                };
                let end = cursor + record_length;
                let mut diagnostic_terminal = object.1.diagnostic_grants;
                if diagnostic_terminal == diagnostic::DELTA && !mixed {
                    diagnostic_terminal = diagnostic::MIXED_REJECTION;
                    stats.diag_event_mixed_rejection += 1;
                    stats.diag_event_mixed_rejection_bytes += object.bytes.len() as u64;
                }
                self.objects.push(PreparedObject {
                    id: object.id,
                    length: object.bytes.len(),
                    pack: pack_index,
                    group: group_number,
                    record: record_number,
                    canonical: cursor + 1..end,
                    retained: (delta || selected.codec == pack::Codec::Zstandard)
                        .then(|| std::mem::take(&mut object.0.bytes)),
                    delta,
                    diagnostic_terminal,
                });
                cursor = end;
            }
            offset += selected.bytes.len();
            backing += selected.bytes.capacity();
            encoded.push(selected);
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
            delta: false,
            diagnostic_terminal: 0,
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
        let mut diagnostic_stats = self.insert(&transaction, &winners, statement_number)?;
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
        for object in &self.objects {
            if object.diagnostic_terminal == 0 {
                continue;
            }
            if late.contains_key(&object.id) {
                diagnostic_stats.diag_race_count += 1;
                diagnostic_stats.diag_race_bytes += object.length as u64;
            } else {
                if object.delta {
                    diagnostic_stats.diag_new_delta_count += 1;
                    diagnostic_stats.diag_new_delta_bytes += object.length as u64;
                } else {
                    diagnostic_stats.diag_new_full_count += 1;
                    diagnostic_stats.diag_new_full_bytes += object.length as u64;
                }
                diagnostic::terminal(
                    object.diagnostic_terminal,
                    object.length,
                    &mut diagnostic_stats,
                );
            }
        }
        db.note_physical(diagnostic_stats);
        db.note_physical(crate::PhysicalStorageReceipt {
            full_selected: winners
                .iter()
                .flatten()
                .filter(|object| !object.delta)
                .count() as u64,
            delta_selected: winners
                .iter()
                .flatten()
                .filter(|object| object.delta)
                .count() as u64,
            ..Default::default()
        });
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
    ) -> Result<crate::PhysicalStorageReceipt> {
        let mut diagnostic_stats = crate::PhysicalStorageReceipt::default();
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
            diagnostic_stats.diag_selected_pack_count += 1;
            diagnostic_stats.diag_selected_pack_last_id = next as u64;
            diagnostic_stats.diag_selected_pack_bytes += self.packs[index].len() as u64;
            diagnostic_stats.diag_selected_pack_groups +=
                u32::from_le_bytes(self.packs[index][12..16].try_into().unwrap()) as u64;
            let first = self.objects.partition_point(|object| object.pack < index);
            let last = self.objects.partition_point(|object| object.pack <= index);
            diagnostic_stats.diag_selected_pack_records += (last - first) as u64;
            diagnostic_stats.diag_selected_unlocated_records +=
                (last - first - objects.len()) as u64;
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
            let sql = format!(
                "INSERT INTO objects(object_id,canonical_length,pack_id,group_number,record_number) VALUES {}",
                vec!["(?,?,?,?,?)"; page.len()].join(",")
            );
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
        Ok(diagnostic_stats)
    }
}

/// Optional search is batch-local; admitted immutable locations are the only
/// source of bases. No record in this prepared batch can become an anchor.
struct DeltaSearch {
    reads: read::HintReadBudget,
    trials: usize,
    remaining: usize,
    input_associations: usize,
}

impl Default for DeltaSearch {
    fn default() -> Self {
        Self {
            reads: read::HintReadBudget::default(),
            trials: 0,
            remaining: 16 * 1024 * 1024,
            input_associations: 0,
        }
    }
}

impl DeltaSearch {
    fn candidate(
        &mut self,
        db: &StoreDb,
        object: &AuthenticatedCanonicalObject,
        stats: &mut crate::PhysicalStorageReceipt,
    ) -> Result<Option<Vec<u8>>> {
        // Payload span hints retain their policy. S1 additionally accepts only
        // exact inode leaves carrying the tree editor's immutable origin.
        let value = layerfs_content::decode_bytes_object(&object.bytes);
        let Ok(value) = value else {
            return Ok(None);
        };
        let inode_leaf = super::is_inode_table_leaf(&object.bytes)?;
        if !inode_leaf {
            if !value.starts_with(layerfs_content::file::extent_codec::CHUNK_MAGIC) {
                return Ok(None);
            }
            layerfs_content::file::extent_codec::decode_chunk_payload(value)?;
        }
        if object.bytes.len() + 9 > pack::GROUP_LIMIT {
            return Ok(None);
        }
        stats.eligible_targets += 1;
        let hints = object.prior_ids();
        if !object.1.has_predecessor {
            stats.absent_predecessors += 1;
        }
        if hints.iter().all(Option::is_none) {
            stats.targets_without_hints += 1;
            return Ok(None);
        }
        let mut seen_hints = BTreeSet::new();
        let mut anchors = BTreeSet::new();
        let mut best: Option<(ObjectId, Vec<u8>)> = None;
        self.reads.begin_target();
        for id in hints.iter().flatten().copied() {
            if !seen_hints.insert(id) || anchors.contains(&id) {
                continue;
            }
            if self.trials == 512 || self.remaining == 0 {
                stats.budget_skips += 1;
                stats.match_budget_skips += 1;
                break;
            }
            stats.predecessor_hints += 1;
            let prior = db.read_hint(id, false, &mut self.reads)?;
            let base = match prior {
                Some(read::HintRecord::Full(base)) => base,
                Some(read::HintRecord::Anchor(_)) if inode_leaf => continue,
                Some(read::HintRecord::Anchor(id)) => {
                    if anchors.contains(&id) {
                        continue;
                    }
                    match db.read_hint(id, true, &mut self.reads)? {
                        Some(read::HintRecord::Full(base)) => base,
                        Some(read::HintRecord::Anchor(_)) => {
                            return Err(StoreError::Integrity("delta anchor is not FULL"));
                        }
                        None if self.reads.exhausted => {
                            stats.budget_skips += 1;
                            stats.fetch_budget_skips += 1;
                            break;
                        }
                        None => continue,
                    }
                }
                None if self.reads.exhausted => {
                    stats.budget_skips += 1;
                    stats.fetch_budget_skips += 1;
                    break;
                }
                None => continue,
            };
            if !anchors.insert(base.id) {
                continue;
            }
            if inode_leaf {
                if !super::is_inode_table_leaf(&base.bytes)? {
                    continue;
                }
            } else {
                let base_value = layerfs_content::decode_bytes_object(&base.bytes)?;
                if !base_value.starts_with(layerfs_content::file::extent_codec::CHUNK_MAGIC) {
                    continue;
                }
                layerfs_content::file::extent_codec::decode_chunk_payload(base_value)?;
            }
            stats.usable_bases += 1;
            stats.candidate_trials += 1;
            self.trials += 1;
            let started = Instant::now();
            let result = pack::delta_record(
                base.id,
                &base.bytes,
                &object.bytes,
                &mut self.remaining,
                stats,
            );
            stats.matching_ns += super::elapsed_ns(started);
            let candidate = result?;
            if let Some(candidate) = candidate {
                if best
                    .as_ref()
                    .is_none_or(|(id, bytes)| (candidate.len(), base.id) < (bytes.len(), *id))
                {
                    best = Some((base.id, candidate));
                }
            }
            if self.remaining == 0 {
                stats.budget_skips += 1;
                break;
            }
        }
        Ok(best.map(|(_, bytes)| bytes))
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
    db.visit_locations(&mut ordinary, |object| {
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

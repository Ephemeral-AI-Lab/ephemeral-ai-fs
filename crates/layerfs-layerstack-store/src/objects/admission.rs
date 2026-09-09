//! Prepared whole-pack admission with epoch-validated final collision checks.
use super::diagnostic;
use super::{
    pack, read, AuthenticatedCanonicalObject, ObjectInsertMetrics, ADMISSION_BATCH_BYTES,
    OBJECT_PAGE_COUNT,
};
use crate::schema::StoreDb;
use crate::{Result, StoreError};
use layerfs_content::ObjectId;
use rusqlite::{limits::Limit, params_from_iter, types::Value, Connection};
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
    session: std::sync::Arc<super::AdmissionSession>,
    final_batch: bool,
    absence_epoch: Option<u64>,
    packs: Vec<Vec<u8>>,
    objects: Vec<PreparedObject>,
    metrics: ObjectInsertMetrics,
    native_base_max_pack: i64,
    canonical_live_capacity: usize,
    oversized_backing: usize,
}

impl PreparedAdmission {
    pub(crate) fn prepare_missing(db: &StoreDb, missing: super::MissingBatch) -> Result<Self> {
        let session = missing.1.clone();
        session.resolve(Self::prepare_missing_inner(db, missing))
    }

    fn prepare_missing_inner(db: &StoreDb, missing: super::MissingBatch) -> Result<Self> {
        if !db.same_instance(&missing.1.db) {
            return Err(StoreError::Integrity("admission Store ownership"));
        }
        missing.1.ensure_active()?;
        let objects = missing.0;
        let length = objects
            .iter()
            .try_fold(0usize, |sum, object| sum.checked_add(object.bytes.len()))
            .ok_or(StoreError::Integrity("admission length overflow"))?;
        // Output ownership is bounded independently from collision-read waves.
        // Maximal canonical objects retain their existing isolated treatment.
        if objects.len() > super::PHYSICAL_ADMISSION_BATCH_COUNT
            || length > ADMISSION_BATCH_BYTES
            || (objects.len() > 1 && length > 2 * super::INITIALIZATION_SLAB_BYTES)
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

        let canonical_live_capacity = objects.iter().map(|o| o.bytes.capacity()).sum::<usize>();
        if canonical_live_capacity > 6 * 1024 * 1024 {
            return Err(StoreError::Io(std::io::Error::other(
                "canonical data reservation",
            )));
        }
        let mut prepared = Self {
            session: missing.1,
            final_batch: missing.2,
            absence_epoch: missing.3,
            // No pack-pointer growth during either lane: at most one pack/object.
            packs: Vec::with_capacity(count),
            objects: Vec::with_capacity(count),
            metrics,
            native_base_max_pack: 0,
            canonical_live_capacity,
            oversized_backing: 0,
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
        let (small, objects): (Vec<_>, Vec<_>) = objects.into_iter().partition(|object| object.is_small_content());
        self.prepare_small(db, small, stats)?;
        // Native and legacy lanes preserve canonical input order independently.
        // Publish the native lane first; no prepared record can become a base.
        let (native, objects): (Vec<_>, Vec<_>) = objects
            .into_iter()
            .partition(|object| db.native_format() && object.is_file_payload());
        // The original input Vec has dropped; both actual lane allocations live.
        search.input_associations = (native.capacity() + objects.capacity())
            * std::mem::size_of::<AuthenticatedCanonicalObject>();
        self.prepare_native(db, native, &mut search, stats)?;
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

    fn physical_backing(&self) -> usize {
        self.packs.iter().map(Vec::capacity).sum::<usize>() - self.oversized_backing
    }

    fn data_reserve(&self, extra: usize) -> Result<()> {
        let owned =
            self.canonical_live_capacity + self.packs.iter().map(Vec::capacity).sum::<usize>();
        if owned + extra > 6 * 1024 * 1024 {
            return Err(StoreError::Io(std::io::Error::other(
                "prepared data reservation",
            )));
        }
        Ok(())
    }

    fn native_scratch(
        &self,
        pending: &Vec<NativePrepared>,
        groups: &Vec<pack::EncodedGroup>,
        input_associations: usize,
        extra: usize,
    ) -> Result<()> {
        let owned = self.physical_backing()
            + pending.iter().map(|p| p.record.capacity()).sum::<usize>()
            + groups.iter().map(|g| g.bytes.capacity()).sum::<usize>();
        let associations = input_associations
            + self.objects.capacity() * std::mem::size_of::<PreparedObject>()
            + self.packs.capacity() * std::mem::size_of::<Vec<u8>>()
            + pending.capacity() * std::mem::size_of::<NativePrepared>()
            + groups.capacity() * std::mem::size_of::<pack::EncodedGroup>();
        if owned + associations + extra > 2 * 1024 * 1024 {
            return Err(StoreError::Io(std::io::Error::other(
                "native scratch reservation",
            )));
        }
        Ok(())
    }

    fn prepare_small(&mut self, db: &StoreDb, objects: Vec<AuthenticatedCanonicalObject>, stats: &mut crate::PhysicalStorageReceipt) -> Result<()> {
        if objects.is_empty() { return Ok(()); }
        if !db.small_content_format() { return Err(StoreError::Integrity("SmallContent write requires schema 8")); }
        let predecessors = objects.iter().filter_map(|o| o.1.prior_ids[0]).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        let locations = db.object_locations(&predecessors)?;
        drop(predecessors);
        let mut groups = Vec::new();
        let mut group_bytes = 0;
        let mut encoder = None;
        for object in objects {
            // Static codec workspace is charged to data; operands and handoff to physical output.
            self.data_reserve(3 * 1024 * 1024)?;
            if self.physical_backing() + group_bytes * 2 + 1024 * 1024 > 2 * 1024 * 1024 {
                return Err(StoreError::Integrity("SmallContent physical output budget"));
            }
            let raw = layerfs_content::file::content::small_bytes(&object.bytes)?.ok_or(StoreError::Integrity("SmallContent role"))?;
            let anchor = if let Some(prior) = object.1.prior_ids[0] {
                // Decoder and encoder never overlap. No predecessor decode to find a base ID.
                drop(encoder.take());
                db.small_anchor(prior, locations.get(&prior).copied())?
            } else { None };
            if encoder.is_none() { encoder = Some(pack::NativeEncoder::new_small()?); }
            let started = Instant::now();
            let full = encoder.as_mut().unwrap().compress(raw, None)?;
            stats.encoding_calls += 1;
            stats.full_alternative_bytes += (full.len() + 9 + 16) as u64;
            let mut base = None;
            let mut frame = full;
            if let Some((anchor, location)) = anchor {
                let prefix = layerfs_content::file::content::small_bytes(&anchor.bytes)?.ok_or(StoreError::Integrity("SmallContent anchor role"))?;
                stats.usable_bases += 1;
                stats.candidate_trials += 1;
                let delta = encoder.as_mut().unwrap().compress(raw, Some(prefix))?;
                stats.encoding_calls += 1;
                if delta.len() + 32 < frame.len() {
                    base = Some(anchor.id);
                    frame = delta;
                    self.native_base_max_pack = self.native_base_max_pack.max(location.pack);
                }
            }
            stats.encoding_ns += started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
            let delta = base.is_some();
            let group = super::delta::encode(raw.len(), base, frame)?;
            if 16 + 16 * (groups.len() + 1) + group_bytes + group.bytes.len() > pack::PACK_LIMIT || groups.len() == pack::GROUP_COUNT_LIMIT {
                self.packs.push(pack::assemble_small(&groups)?);
                groups.clear();
                group_bytes = 0;
            }
            stats.eligible_targets += 1;
            stats.full_selected += u64::from(!delta);
            stats.delta_selected += u64::from(delta);
            stats.selected_encoded_bytes += group.bytes.len() as u64;
            self.objects.push(PreparedObject {
                id: object.id, length: object.bytes.len(), pack: self.packs.len(), group: groups.len(), record: 0,
                canonical: 0..0, retained: Some(object.0.bytes), delta,
                diagnostic_terminal: if delta { diagnostic::DELTA } else { diagnostic::NO_DELTA },
            });
            group_bytes += group.bytes.len();
            groups.push(group);
        }
        drop(encoder);
        if !groups.is_empty() { self.packs.push(pack::assemble_small(&groups)?); }
        Ok(())
    }

    fn prepare_native(
        &mut self,
        db: &StoreDb,
        objects: Vec<AuthenticatedCanonicalObject>,
        search: &mut DeltaSearch,
        stats: &mut crate::PhysicalStorageReceipt,
    ) -> Result<()> {
        if objects.is_empty() {
            return Ok(());
        }
        let input_associations = search.input_associations;
        let planned_associations = input_associations
            + objects.len() * std::mem::size_of::<NativePrepared>()
            + objects.len().min(pack::GROUP_COUNT_LIMIT)
                * std::mem::size_of::<pack::EncodedGroup>()
            + self.objects.capacity() * std::mem::size_of::<PreparedObject>()
            + self.packs.capacity() * std::mem::size_of::<Vec<u8>>();
        if planned_associations > 2 * 1024 * 1024 {
            return Err(StoreError::Io(std::io::Error::other(
                "native association reservation",
            )));
        }
        let mut pending = Vec::<NativePrepared>::with_capacity(objects.len());
        let mut groups =
            Vec::<pack::EncodedGroup>::with_capacity(objects.len().min(pack::GROUP_COUNT_LIMIT));
        let mut full_group_length = 4usize;
        let mut encoder = None;
        for object in objects {
            self.native_scratch(
                &pending,
                &groups,
                input_associations,
                pack::NATIVE_ENCODE_WORKSPACE
                    + 3 * (pack::NATIVE_FRAME_LIMIT + 37)
                    + pack::NATIVE_RAW_LIMIT
                    + 21,
            )?;
            let raw = layerfs_content::file::extent_codec::decode_chunk_payload(
                layerfs_content::decode_bytes_object(&object.bytes)?,
            )?;
            let started = Instant::now();
            let full = (|| {
                if encoder.is_none() {
                    encoder = Some(pack::NativeEncoder::new()?);
                }
                encoder.as_mut().unwrap().compress(raw, None)
            })();
            let elapsed = super::elapsed_ns(started);
            stats.native_full_encode_calls += 1;
            stats.native_full_encode_ns += elapsed;
            stats.encoding_calls += 1;
            stats.encoding_ns += elapsed;
            let full = full?;
            stats.native_full_frame_count += 1;
            stats.native_full_frame_bytes += full.len() as u64;
            // Grouping is frozen by the complete FULL alternative, not the
            // eventual PREFIX size. Pack assembly still uses actual group bytes.
            let next = 4 + 5 + full.len();
            if full_group_length + next > pack::GROUP_LIMIT {
                self.flush_native_group(
                    &mut pending,
                    &mut groups,
                    input_associations,
                    full.capacity(),
                    &mut encoder,
                    stats,
                )?;
                full_group_length = 4;
            }
            full_group_length += next;
            let mut terminal = diagnostic::state(&object);
            let mut chosen = None;
            let mut fallback = NativeFallback::NoHint;
            stats.eligible_targets += 1;
            stats.absent_predecessors += u64::from(!object.1.has_predecessor);
            if let Some(id) = object.prior_ids().iter().flatten().next().copied() {
                // Reader and encoder each own bounded scratch; never overlap them.
                drop(encoder.take());
                search.reads.begin_target();
                if search.trials == 512
                    || self
                        .native_scratch(
                            &pending,
                            &groups,
                            input_associations,
                            full.capacity() + 1024 * 1024,
                        )
                        .is_err()
                {
                    fallback = NativeFallback::Budget;
                    stats.budget_skips += 1;
                    if search.trials == 512 {
                        stats.match_budget_skips += 1;
                        stats.diag_event_match_budget_count += 1;
                        stats.diag_event_match_budget_bytes += object.bytes.len() as u64;
                    } else {
                        stats.memory_budget_skips += 1;
                        stats.diag_event_memory_budget_count += 1;
                        stats.diag_event_memory_budget_bytes += object.bytes.len() as u64;
                    }
                } else {
                    stats.predecessor_hints += 1;
                    match db.read_native_prior(id, &mut search.reads)? {
                        read::NativePriorOutcome::Unavailable => {
                            fallback = NativeFallback::Unavailable
                        }
                        read::NativePriorOutcome::UnsupportedLegacyDelta => {
                            fallback = NativeFallback::LegacyDelta
                        }
                        read::NativePriorOutcome::UnsupportedRole => {
                            fallback = NativeFallback::Role
                        }
                        read::NativePriorOutcome::Budget => {
                            fallback = NativeFallback::Budget;
                            stats.budget_skips += 1;
                            stats.fetch_budget_skips += 1;
                            stats.diag_event_fetch_budget_count += 1;
                            stats.diag_event_fetch_budget_bytes += object.bytes.len() as u64;
                        }
                        read::NativePriorOutcome::Available {
                            canonical,
                            location,
                            depth,
                            raw_closure,
                        } => {
                            if depth >= 4
                                || raw_closure
                                    .checked_add(raw.len())
                                    .is_none_or(|n| n > 1024 * 1024)
                            {
                                fallback = NativeFallback::Depth;
                            } else if self
                                .native_scratch(
                                    &pending,
                                    &groups,
                                    input_associations,
                                    pack::NATIVE_ENCODE_WORKSPACE
                                        + full.capacity()
                                        + 2 * (pack::NATIVE_FRAME_LIMIT + 37)
                                        + canonical.bytes.capacity(),
                                )
                                .is_err()
                            {
                                fallback = NativeFallback::Budget;
                                stats.budget_skips += 1;
                                stats.memory_budget_skips += 1;
                                stats.diag_event_memory_budget_count += 1;
                                stats.diag_event_memory_budget_bytes += object.bytes.len() as u64;
                            } else {
                                stats.usable_bases += 1;
                                stats.candidate_trials += 1;
                                stats.diag_event_base += 1;
                                stats.diag_event_base_bytes += object.bytes.len() as u64;
                                search.trials += 1;
                                let prefix =
                                    layerfs_content::file::extent_codec::decode_chunk_payload(
                                        layerfs_content::decode_bytes_object(&canonical.bytes)?,
                                    )?;
                                // The reader owns a separate canonical allocation;
                                // target and prefix cannot overlap as codec operands.
                                let started = Instant::now();
                                let result = (|| {
                                    encoder = Some(pack::NativeEncoder::new()?);
                                    encoder.as_mut().unwrap().compress(raw, Some(prefix))
                                })();
                                let elapsed = super::elapsed_ns(started);
                                stats.native_prefix_encode_calls += 1;
                                stats.native_prefix_encode_ns += elapsed;
                                stats.encoding_calls += 1;
                                stats.encoding_ns += elapsed;
                                match result {
                                    Ok(frame) => {
                                        stats.native_prefix_frame_count += 1;
                                        stats.native_prefix_frame_bytes += frame.len() as u64;
                                        stats.diag_event_candidate += 1;
                                        stats.diag_event_candidate_bytes +=
                                            object.bytes.len() as u64;
                                        if 37 + frame.len() < 5 + full.len() {
                                            self.native_base_max_pack =
                                                self.native_base_max_pack.max(location.pack);
                                            chosen = Some(pack::native_encode_record(
                                                raw.len(),
                                                Some(id),
                                                &frame,
                                            )?);
                                            terminal = diagnostic::DELTA;
                                        } else {
                                            fallback = NativeFallback::FullWins;
                                        }
                                    }
                                    Err(StoreError::Io(_)) => {
                                        // Codec helper has no I/O: this variant denotes
                                        // its bounded workspace/output resource failure.
                                        fallback = NativeFallback::Budget;
                                        stats.budget_skips += 1;
                                        stats.memory_budget_skips += 1;
                                        stats.diag_event_memory_budget_count += 1;
                                        stats.diag_event_memory_budget_bytes +=
                                            object.bytes.len() as u64;
                                    }
                                    Err(error) => return Err(error),
                                }
                            }
                        }
                    }
                }
            } else {
                stats.targets_without_hints += 1;
            }
            let delta = chosen.is_some();
            if !delta {
                fallback.note(object.bytes.len(), stats);
                if terminal == diagnostic::BASE {
                    terminal = match fallback {
                        NativeFallback::Budget => diagnostic::BUDGET,
                        NativeFallback::FullWins => diagnostic::NO_DELTA,
                        _ => diagnostic::BASE,
                    };
                }
                if matches!(fallback, NativeFallback::Budget) {
                    stats.diag_event_budget += 1;
                    stats.diag_event_budget_bytes += object.bytes.len() as u64;
                }
            }
            let record = match chosen {
                Some(record) => record,
                None => pack::native_encode_record(raw.len(), None, &full)?,
            };
            stats.diag_invalid += u64::from(object.1.diagnostic_grants != 0);
            pending.push(NativePrepared {
                canonical: object.0,
                record,
                delta,
                terminal,
            });
        }
        drop(encoder.take());
        self.flush_native_group(
            &mut pending,
            &mut groups,
            input_associations,
            0,
            &mut encoder,
            stats,
        )?;
        if !groups.is_empty() {
            let length =
                16 + 16 * groups.len() + groups.iter().map(|g| g.bytes.len()).sum::<usize>();
            self.native_scratch(&pending, &groups, input_associations, length)?;
            self.data_reserve(length)?;
            self.packs.push(pack::assemble_native(&groups)?);
        }
        Ok(())
    }

    fn flush_native_group(
        &mut self,
        pending: &mut Vec<NativePrepared>,
        groups: &mut Vec<pack::EncodedGroup>,
        input_associations: usize,
        live_full_capacity: usize,
        encoder: &mut Option<pack::NativeEncoder>,
        stats: &mut crate::PhysicalStorageReceipt,
    ) -> Result<()> {
        if pending.is_empty() {
            return Ok(());
        }
        let group_length =
            4 + 4 * pending.len() + pending.iter().map(|p| p.record.len()).sum::<usize>();
        let assembled_length =
            16 + 16 * groups.len() + groups.iter().map(|g| g.bytes.len()).sum::<usize>();
        // Keep scratch reuse only when the unchanged physical ceiling also fits
        // old records, copied group/pack and references. Releasing the encoder
        // restores the previous assembly ownership without changing pack layout.
        let extra = live_full_capacity
            + group_length
            + assembled_length
            + pending.len() * std::mem::size_of::<&[u8]>();
        if encoder.is_some()
            && self
                .native_scratch(
                    pending,
                    groups,
                    input_associations,
                    extra + pack::NATIVE_ENCODE_WORKSPACE,
                )
                .is_err()
        {
            drop(encoder.take());
        }
        self.native_scratch(
            pending,
            groups,
            input_associations,
            extra
                + if encoder.is_some() {
                    pack::NATIVE_ENCODE_WORKSPACE
                } else {
                    0
                },
        )?;
        let refs = pending
            .iter()
            .map(|p| p.record.as_slice())
            .collect::<Vec<_>>();
        let group = pack::native_group(&refs)?;
        drop(refs);
        let next_bytes = 16
            + 16 * (groups.len() + 1)
            + group.bytes.len()
            + groups.iter().map(|g| g.bytes.len()).sum::<usize>();
        let next_records = group.records + groups.iter().map(|g| g.records).sum::<usize>();
        if next_bytes > pack::PACK_LIMIT
            || groups.len() == pack::GROUP_COUNT_LIMIT
            || next_records > pack::RECORD_COUNT_LIMIT
        {
            self.data_reserve(assembled_length)?;
            self.packs.push(pack::assemble_native(groups)?);
            groups.clear();
        }
        let pack = self.packs.len();
        let group_number = groups.len();
        for (record_number, entry) in pending.drain(..).enumerate() {
            let length = entry.canonical.bytes.len();
            self.objects.push(PreparedObject {
                id: entry.canonical.id,
                length,
                pack,
                group: group_number,
                record: record_number,
                canonical: 0..0,
                // Native FULL is internally compressed too: final CAS must retain
                // the authentic canonical operand, never compare frame bytes.
                retained: Some(entry.canonical.bytes),
                delta: entry.delta,
                diagnostic_terminal: entry.terminal,
            });
        }
        stats.selected_encoded_bytes += group.bytes.len() as u64;
        groups.push(group);
        Ok(())
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
        let mut backing = self.physical_backing();
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
                let canonical_length = object.bytes.len();
                let retained = if delta || selected.codec == pack::Codec::Zstandard {
                    Some(std::mem::take(&mut object.0.bytes))
                } else {
                    self.canonical_live_capacity -= object.0.bytes.capacity();
                    object.0.bytes = Vec::new();
                    None
                };
                self.objects.push(PreparedObject {
                    id: object.id,
                    length: canonical_length,
                    pack: pack_index,
                    group: group_number,
                    record: record_number,
                    canonical: cursor + 1..end,
                    retained,
                    delta,
                    diagnostic_terminal,
                });
                cursor = end;
            }
            offset += selected.bytes.len();
            backing += selected.bytes.capacity();
            encoded.push(selected);
        }
        let length = 16 + 16 * encoded.len() + encoded.iter().map(|g| g.bytes.len()).sum::<usize>();
        self.data_reserve(length)?;
        if backing + fixed_associations + length > 2 * 1024 * 1024 {
            return Err(StoreError::Io(std::io::Error::other(
                "legacy assembly reservation",
            )));
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
        self.canonical_live_capacity -= object.bytes.capacity();
        drop(object);
        self.data_reserve(total)?;
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
        // This exception belongs only to the existing constructed version-1
        // RAW singleton; ordinary/native packs never enter this data-only lane.
        self.oversized_backing += bytes.capacity();
        self.packs.push(bytes);
        Ok(())
    }

    pub(crate) fn publish<T>(
        self,
        db: &StoreDb,
        statement_number: &mut u64,
        publish: impl FnOnce(&Connection, &ObjectInsertMetrics, &mut u64) -> Result<T>,
    ) -> Result<(T, super::AdmissionBatchMetrics)> {
        let session = self.session.clone();
        session.resolve(self.publish_inner(db, statement_number, publish))
    }

    fn publish_inner<T>(
        mut self,
        db: &StoreDb,
        statement_number: &mut u64,
        publish: impl FnOnce(&Connection, &ObjectInsertMetrics, &mut u64) -> Result<T>,
    ) -> Result<(T, super::AdmissionBatchMetrics)> {
        if !db.same_instance(&self.session.db) {
            return Err(StoreError::Integrity("admission Store ownership"));
        }
        self.session.ensure_active()?;
        let ids = self
            .objects
            .iter()
            .map(|object| object.id)
            .collect::<Vec<_>>();
        let mut supplied = self
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
            .collect::<Vec<_>>();

        // Absence was authenticated by the exact earlier lookup under this
        // exclusive owner. An intervening publication requires the normal CAS
        // recheck; every positive collision still compares canonical bytes.
        let late = if self.absence_epoch
            == Some(
                self.session
                    .publication_epoch
                    .load(std::sync::atomic::Ordering::Acquire),
            ) {
            BTreeMap::new()
        } else {
            db.object_locations(&ids)?
        };
        drop(ids);
        let retained = self.physical_backing()
            + self.objects.capacity() * std::mem::size_of::<PreparedObject>()
            + self.packs.capacity() * std::mem::size_of::<Vec<u8>>();
        compare(db, &late, &mut supplied, &mut self.metrics, retained)?;
        let mut winners = vec![Vec::new(); self.packs.len()];
        for object in &self.objects {
            if !late.contains_key(&object.id) {
                winners[object.pack].push(object);
            }
        }
        let connection = db.writer()?;
        let canonical_bytes = self.objects.iter().map(|object| object.length as u64).sum();
        self.session.begin_batch(
            &connection,
            self.metrics.submitted_rows,
            canonical_bytes,
            &mut self.metrics.sql,
        )?;
        let started = Instant::now();
        let mut diagnostic_stats = self.insert(&connection, &winners, statement_number)?;
        self.metrics.insert_ns += super::elapsed_ns(started);
        self.metrics.objects = winners.iter().map(|objects| objects.len() as u64).sum();
        self.metrics.bytes = winners
            .iter()
            .flatten()
            .map(|object| object.length as u64)
            .sum();
        self.metrics.returned_ids = self.metrics.objects;
        let result = publish(&connection, &self.metrics, statement_number)?;
        self.session
            .note_published_ids(winners.iter().flatten().map(|object| &object.id))?;
        if self.final_batch || !self.session.coalesce {
            self.session
                .commit_pending(&connection, &mut self.metrics.sql, self.final_batch)?;
        }
        // Epoch tracks connection-visible object publications, not disk commits.
        self.session
            .publication_epoch
            .fetch_update(
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
                |epoch| epoch.checked_add(1),
            )
            .map_err(|_| StoreError::Integrity("admission publication epoch overflow"))?;
        if self.final_batch {
            self.session.retain();
        }
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
                if self.packs[object.pack][8..12] == [2, 0, 0, 0] {
                    if object.delta {
                        diagnostic_stats.native_admitted_prefix_count += 1;
                        diagnostic_stats.native_admitted_prefix_bytes += object.length as u64;
                    } else {
                        diagnostic_stats.native_admitted_full_count += 1;
                        diagnostic_stats.native_admitted_full_bytes += object.length as u64;
                    }
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
                begin_ns: self.metrics.sql.begin_ns,
                commit_ns: self.metrics.sql.commit_ns,
            },
        ))
    }

    fn insert(
        &self,
        transaction: &Connection,
        winners: &[Vec<&PreparedObject>],
        statement_number: &mut u64,
    ) -> Result<crate::PhysicalStorageReceipt> {
        let mut diagnostic_stats = crate::PhysicalStorageReceipt::default();
        let mut next: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(pack_id),0) FROM object_packs",
            [],
            |row| row.get(0),
        )?;
        // Bases came from selected immutable locations before preparation.
        // All newly assigned IDs exceed this transaction's existing maximum.
        if self.native_base_max_pack > next {
            return Err(StoreError::Integrity("native base publication chronology"));
        }
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
            if transaction
                .prepare_cached(&sql)?
                .execute(params_from_iter(values))?
                != page.len()
            {
                return Err(StoreError::Integrity("pack insertion cardinality"));
            }
            start = end;
        }
        // Preserve pack bytes/order; only the SQL primary-key insertion order changes.
        let sort_started = Instant::now();
        locators.sort_unstable_by_key(|(_, object)| object.id);
        crate::telemetry::note_workspace_admission_sort(super::elapsed_ns(sort_started));
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
            if transaction
                .prepare_cached(&sql)?
                .execute(params_from_iter(values))?
                != page.len()
            {
                return Err(StoreError::Integrity("locator insertion cardinality"));
            }
        }
        Ok(diagnostic_stats)
    }
}

struct NativePrepared {
    canonical: super::CanonicalObject,
    record: Vec<u8>,
    delta: bool,
    terminal: u8,
}

#[derive(Clone, Copy)]
enum NativeFallback {
    NoHint,
    Unavailable,
    LegacyDelta,
    Role,
    Depth,
    Budget,
    FullWins,
}
impl NativeFallback {
    fn note(self, bytes: usize, stats: &mut crate::PhysicalStorageReceipt) {
        macro_rules! count {
            ($n:ident,$b:ident) => {{
                stats.$n += 1;
                stats.$b += bytes as u64;
            }};
        }
        match self {
            Self::NoHint => count!(native_fallback_no_hint_count, native_fallback_no_hint_bytes),
            Self::Unavailable => count!(
                native_fallback_unavailable_count,
                native_fallback_unavailable_bytes
            ),
            Self::LegacyDelta => count!(
                native_fallback_legacy_delta_count,
                native_fallback_legacy_delta_bytes
            ),
            Self::Role => count!(native_fallback_role_count, native_fallback_role_bytes),
            Self::Depth => count!(native_fallback_depth_count, native_fallback_depth_bytes),
            Self::Budget => count!(native_fallback_budget_count, native_fallback_budget_bytes),
            Self::FullWins => count!(
                native_fallback_full_wins_count,
                native_fallback_full_wins_bytes
            ),
        }
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
    supplied: &mut Vec<(ObjectId, &[u8])>,
    metrics: &mut ObjectInsertMetrics,
    retained_physical: usize,
) -> Result<()> {
    let started = Instant::now();
    if supplied.len() > super::PHYSICAL_ADMISSION_BATCH_COUNT || known.len() > supplied.len() {
        return Err(StoreError::Integrity("comparison ownership count"));
    }
    supplied.sort_unstable_by_key(|(id, _)| *id);
    if supplied.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(StoreError::Integrity("duplicate comparison operand"));
    }
    let canonical_for = |id: ObjectId| {
        supplied
            .binary_search_by_key(&id, |(id, _)| *id)
            .ok()
            .map(|index| supplied[index].1)
    };
    let mut ordinary = Vec::with_capacity(known.len());
    for (id, location) in known {
        let canonical =
            canonical_for(*id).ok_or(StoreError::Integrity("unexpected membership result"))?;
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
    if !ordinary.is_empty() {
        // Retained locator/map/slot ownership is charged for the whole lookup;
        // each active wave retains its own conservative association charge too.
        // The other 1 MiB remains reserved for active decoding/reconstruction.
        let retained = retained_physical
            .checked_add(supplied.capacity() * std::mem::size_of::<(ObjectId, &[u8])>())
            .and_then(|n| n.checked_add(known.len() * 512))
            .ok_or(StoreError::Integrity("comparison ownership overflow"))?;
        let reserve = read::VALIDATION_RESERVE
            .checked_sub(retained)
            .ok_or(StoreError::Integrity("comparison physical reservation"))?;
        db.visit_locations_with_reserve(&mut ordinary, reserve, |object| {
            if canonical_for(object.id) != Some(object.bytes.as_slice()) {
                return Err(StoreError::Integrity("object collision"));
            }
            Ok(())
        })?;
    }
    metrics.collision_checks += known.len() as u64;
    metrics.conflict_read_rows += known.len() as u64;
    metrics.conflict_read_bytes += known
        .values()
        .map(|location| location.canonical_length as u64)
        .sum::<u64>();
    metrics.conflict_read_ns += super::elapsed_ns(started);
    Ok(())
}

#[cfg(test)]
mod native_tests;

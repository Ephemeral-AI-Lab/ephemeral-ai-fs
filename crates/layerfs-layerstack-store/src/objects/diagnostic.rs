//! Fixed observations. Outcome counters never decide product admission.
//! The actual file-owner provenance bit is also used by the native payload lane.
use super::AuthenticatedCanonicalObject;
use crate::PhysicalStorageReceipt as Receipt;

pub(super) const FILE: u8 = 16;
pub(super) const INHERITED: u8 = 8;
pub(super) const COMPLETE: u8 = 1;
pub(super) const MEMORY: u8 = 2;
pub(super) const FILE_LIMIT: u8 = 3;
pub(super) const OPERATION: u8 = 4;
pub(super) const DESCRIPTOR: u8 = 5;
pub(super) const NO_PREDECESSOR: u8 = 1;
pub(super) const MISSING_SPAN: u8 = 2;
pub(super) const NO_OVERLAP: u8 = 3;
pub(super) const CORRESPONDENCE_LIMIT: u8 = 4;
pub(super) const BASE: u8 = 5;
pub(super) const BUDGET: u8 = 6;
pub(super) const NO_DELTA: u8 = 7;
pub(super) const MIXED_REJECTION: u8 = 8;
pub(super) const DELTA: u8 = 9;
pub(super) const UNKNOWN: u8 = 10;

pub(super) fn valid(tag: u8, grants: u8) -> bool {
    tag & !31 == 0
        && tag & 7 <= DESCRIPTOR
        && grants <= 7
        && (tag & INHERITED == 0 || ((2..=5).contains(&(tag & 7)) && grants == 0))
        && (tag & 15 == 0 || tag & FILE != 0)
}

pub(super) fn chunk(bytes: &[u8]) -> bool {
    layerfs_content::decode_bytes_object(bytes)
        .is_ok_and(|value| layerfs_content::file::extent_codec::decode_chunk_payload(value).is_ok())
}

pub(super) fn file(object: &AuthenticatedCanonicalObject) -> bool {
    object.is_file_payload()
}

pub(super) fn state(object: &AuthenticatedCanonicalObject) -> u8 {
    if !file(object) {
        return 0;
    }
    if !object.1.has_predecessor {
        return NO_PREDECESSOR;
    }
    if object.1.first_span.is_none() {
        return MISSING_SPAN;
    }
    if object.1.prior_ids.iter().any(Option::is_some) {
        return BASE;
    }
    match object.1.diagnostic & 7 {
        COMPLETE => NO_OVERLAP,
        MEMORY..=DESCRIPTOR => CORRESPONDENCE_LIMIT,
        _ => UNKNOWN,
    }
}

// route0=initial preexisting,1=initial missing,2=duplicate occurrence.
pub(super) fn occurrence(
    object: &mut AuthenticatedCanonicalObject,
    route: u8,
    stats: &mut Receipt,
) {
    let grants = u64::from(std::mem::take(&mut object.1.diagnostic_grants));
    if !file(object) {
        stats.diag_invalid += u64::from(grants != 0);
        return;
    }
    let bytes = object.bytes.len() as u64;
    match route {
        0 => {
            stats.diag_occurrence_preexisting_count += 1;
            stats.diag_occurrence_preexisting_bytes += bytes;
            stats.diag_occurrence_preexisting_grants += grants;
        }
        1 => {
            stats.diag_occurrence_missing_count += 1;
            stats.diag_occurrence_missing_bytes += bytes;
            stats.diag_occurrence_missing_grants += grants;
        }
        _ => {
            stats.diag_occurrence_duplicate_count += 1;
            stats.diag_occurrence_duplicate_bytes += bytes;
            stats.diag_occurrence_duplicate_grants += grants;
        }
    }
}

pub(super) fn eligible(object: &AuthenticatedCanonicalObject, stats: &mut Receipt) {
    if !file(object) {
        if chunk(&object.bytes) {
            stats.diag_nonfile_chunk_count += 1;
            stats.diag_nonfile_chunk_bytes += object.bytes.len() as u64;
        } else if object.1.diagnostic & FILE != 0 {
            stats.diag_file_source_without_chunk += 1;
            stats.diag_invalid += 1;
        }
        return;
    }
    let bytes = object.bytes.len() as u64;
    stats.diag_eligible_count += 1;
    stats.diag_eligible_bytes += bytes;
    match bytes {
        0..64 => {
            stats.diag_size_lt64_count += 1;
            stats.diag_size_lt64_bytes += bytes;
        }
        64..256 => {
            stats.diag_size_lt256_count += 1;
            stats.diag_size_lt256_bytes += bytes;
        }
        256..1024 => {
            stats.diag_size_lt1024_count += 1;
            stats.diag_size_lt1024_bytes += bytes;
        }
        1024..4096 => {
            stats.diag_size_lt4096_count += 1;
            stats.diag_size_lt4096_bytes += bytes;
        }
        4096..16384 => {
            stats.diag_size_lt16384_count += 1;
            stats.diag_size_lt16384_bytes += bytes;
        }
        16384..65536 => {
            stats.diag_size_lt65536_count += 1;
            stats.diag_size_lt65536_bytes += bytes;
        }
        _ => {
            stats.diag_size_ge65536_count += 1;
            stats.diag_size_ge65536_bytes += bytes;
        }
    }
    match object.1.prior_ids.iter().flatten().count() {
        0 => {
            stats.diag_hints_0_count += 1;
            stats.diag_hints_0_bytes += bytes;
        }
        1 => {
            stats.diag_hints_1_count += 1;
            stats.diag_hints_1_bytes += bytes;
        }
        2 => {
            stats.diag_hints_2_count += 1;
            stats.diag_hints_2_bytes += bytes;
        }
        3 => {
            stats.diag_hints_3_count += 1;
            stats.diag_hints_3_bytes += bytes;
        }
        4 => {
            stats.diag_hints_4_count += 1;
            stats.diag_hints_4_bytes += bytes;
        }
        _ => unreachable!(),
    }
    match (
        object.1.diagnostic & 7,
        object.1.diagnostic & INHERITED != 0,
    ) {
        (2, false) => {
            stats.diag_limit_memory_first_count += 1;
            stats.diag_limit_memory_first_bytes += bytes;
        }
        (2, true) => {
            stats.diag_limit_memory_inherited_count += 1;
            stats.diag_limit_memory_inherited_bytes += bytes;
        }
        (3, false) => {
            stats.diag_limit_file_first_count += 1;
            stats.diag_limit_file_first_bytes += bytes;
        }
        (3, true) => {
            stats.diag_limit_file_inherited_count += 1;
            stats.diag_limit_file_inherited_bytes += bytes;
        }
        (4, false) => {
            stats.diag_limit_operation_first_count += 1;
            stats.diag_limit_operation_first_bytes += bytes;
        }
        (4, true) => {
            stats.diag_limit_operation_inherited_count += 1;
            stats.diag_limit_operation_inherited_bytes += bytes;
        }
        (5, false) => {
            stats.diag_limit_descriptor_first_count += 1;
            stats.diag_limit_descriptor_first_bytes += bytes;
        }
        (5, true) => {
            stats.diag_limit_descriptor_inherited_count += 1;
            stats.diag_limit_descriptor_inherited_bytes += bytes;
        }
        _ => {}
    }
    if object.1.prior_ids.iter().all(Option::is_none) {
        match (
            object.1.diagnostic & 7,
            object.1.diagnostic & INHERITED != 0,
        ) {
            (2, false) => {
                stats.diag_limit_memory_first_empty_count += 1;
                stats.diag_limit_memory_first_empty_bytes += bytes;
            }
            (2, true) => {
                stats.diag_limit_memory_inherited_empty_count += 1;
                stats.diag_limit_memory_inherited_empty_bytes += bytes;
            }
            (3, false) => {
                stats.diag_limit_file_first_empty_count += 1;
                stats.diag_limit_file_first_empty_bytes += bytes;
            }
            (3, true) => {
                stats.diag_limit_file_inherited_empty_count += 1;
                stats.diag_limit_file_inherited_empty_bytes += bytes;
            }
            (4, false) => {
                stats.diag_limit_operation_first_empty_count += 1;
                stats.diag_limit_operation_first_empty_bytes += bytes;
            }
            (4, true) => {
                stats.diag_limit_operation_inherited_empty_count += 1;
                stats.diag_limit_operation_inherited_empty_bytes += bytes;
            }
            (5, false) => {
                stats.diag_limit_descriptor_first_empty_count += 1;
                stats.diag_limit_descriptor_first_empty_bytes += bytes;
            }
            (5, true) => {
                stats.diag_limit_descriptor_inherited_empty_count += 1;
                stats.diag_limit_descriptor_inherited_empty_bytes += bytes;
            }
            _ => {}
        }
    }
    match state(object) {
        NO_PREDECESSOR => {
            stats.diag_no_predecessor_count += 1;
            stats.diag_no_predecessor_bytes += bytes;
        }
        MISSING_SPAN => {
            stats.diag_missing_span_count += 1;
            stats.diag_missing_span_bytes += bytes;
            stats.diag_invalid += 1;
        }
        NO_OVERLAP => {
            stats.diag_complete_empty_count += 1;
            stats.diag_complete_empty_bytes += bytes;
        }
        CORRESPONDENCE_LIMIT => {
            stats.diag_limited_empty_count += 1;
            stats.diag_limited_empty_bytes += bytes;
        }
        BASE if object.1.diagnostic & 7 == COMPLETE => {
            stats.diag_complete_hints_count += 1;
            stats.diag_complete_hints_bytes += bytes;
        }
        BASE if (2..=5).contains(&(object.1.diagnostic & 7)) => {
            stats.diag_limited_hints_count += 1;
            stats.diag_limited_hints_bytes += bytes;
        }
        _ => {
            stats.diag_invalid += 1;
        }
    }
}

pub(super) fn terminal(code: u8, bytes: usize, stats: &mut Receipt) {
    let bytes = bytes as u64;
    match code {
        0 => {}
        NO_PREDECESSOR => {
            stats.diag_terminal_no_predecessor_count += 1;
            stats.diag_terminal_no_predecessor_bytes += bytes;
        }
        MISSING_SPAN => {
            stats.diag_terminal_missing_span_count += 1;
            stats.diag_terminal_missing_span_bytes += bytes;
        }
        NO_OVERLAP => {
            stats.diag_terminal_no_overlap_count += 1;
            stats.diag_terminal_no_overlap_bytes += bytes;
        }
        CORRESPONDENCE_LIMIT => {
            stats.diag_terminal_correspondence_limit_count += 1;
            stats.diag_terminal_correspondence_limit_bytes += bytes;
        }
        BASE => {
            stats.diag_terminal_base_count += 1;
            stats.diag_terminal_base_bytes += bytes;
        }
        BUDGET => {
            stats.diag_terminal_budget_count += 1;
            stats.diag_terminal_budget_bytes += bytes;
        }
        NO_DELTA => {
            stats.diag_terminal_no_delta_count += 1;
            stats.diag_terminal_no_delta_bytes += bytes;
        }
        MIXED_REJECTION => {
            stats.diag_terminal_mixed_rejection_count += 1;
            stats.diag_terminal_mixed_rejection_bytes += bytes;
        }
        DELTA => {
            stats.diag_terminal_delta_count += 1;
            stats.diag_terminal_delta_bytes += bytes;
        }
        _ => {
            stats.diag_terminal_unknown_count += 1;
            stats.diag_terminal_unknown_bytes += bytes;
            stats.diag_invalid += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        CheckedOutputAdmission, DeferredObjectStore, ObjectBuffer, PreparedAdmission,
    };
    use super::*;

    fn admit(db: &crate::schema::StoreDb, objects: DeferredObjectStore) {
        let mut admission = CheckedOutputAdmission::new(db).unwrap();
        admission.admit(objects).unwrap();
        let finished = admission.finish().unwrap();
        let mut statement = finished.statement_number;
        PreparedAdmission::prepare_missing(db, finished.final_batch)
            .unwrap()
            .publish(db, &mut statement, |_, _, _| Ok(()))
            .unwrap();
    }

    #[test]
    fn actual_file_source_and_metadata_rope_have_distinct_diagnostic_provenance() {
        let mut generic = ObjectBuffer::empty().unwrap();
        // This exact generic rope path is also used by filesystem mode/mtime.
        let (root, _) = layerfs_content::file::rope::build_bytes(&mut generic, b"mode").unwrap();
        let built = generic.finish(root.0, 0).unwrap();
        built
            .objects
            .consume_prevalidated_pages(|page| {
                for object in page {
                    assert!(!file(&object));
                }
                Ok(())
            })
            .unwrap();
        let built = ObjectBuffer::build_complete_file(b"mode".as_slice(), 4).unwrap();
        let mut payloads = 0;
        built
            .objects
            .consume_prevalidated_pages(|page| {
                payloads += page.iter().filter(|object| file(object)).count();
                Ok(())
            })
            .unwrap();
        assert_eq!(payloads, 1);
    }

    #[test]
    fn spill_handoff_admission_and_pack_provenance_conserve_the_file_cohort() {
        let folder = std::env::temp_dir().join(format!(
            "layerfs-diagnostic-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let store = crate::LayerStackStore::create(folder.join("store.sqlite")).unwrap();
        let base = ObjectBuffer::build_complete_file(b"abcdefgh".as_slice(), 8).unwrap();
        let prior_root = base.root_id;
        admit(&store.db, base.objects);
        let before = store.db.physical_storage_receipt();
        let mut target = ObjectBuffer::new(&store).unwrap();
        target
            .set_physical_predecessor(
                store.snapshot_reader(prior_root),
                layerfs_content::file::rope::FileStateRoot(prior_root),
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            )
            .unwrap();
        let mut built = target
            .build_complete_with_predecessor(b"abcDefgh".as_slice(), 8)
            .unwrap();
        built.objects.spill().unwrap();
        admit(&store.db, built.objects.all_reachable().unwrap());
        let stats = store.db.physical_storage_receipt().since(before);
        assert_eq!(
            (stats.diag_eligible_count, stats.diag_eligible_bytes),
            (1, 29)
        );
        assert_eq!(stats.diag_complete_hints_count, 1);
        assert_eq!(stats.diag_cursor_grants, 2);
        assert_eq!(stats.diag_occurrence_missing_grants, 2);
        assert_eq!(stats.diag_new_full_count + stats.diag_new_delta_count, 1);
        assert_eq!((stats.diag_race_count, stats.diag_invalid), (0, 0));
        assert!(stats.diag_selected_pack_count > 0);
        let last: i64 = store
            .db
            .reader()
            .unwrap()
            .query_row("SELECT max(pack_id) FROM object_packs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stats.diag_selected_pack_last_id, last as u64);
        assert_eq!(stats.diag_selected_unlocated_records, 0);

        let before = store.db.physical_storage_receipt();
        let mut limited = ObjectBuffer::new(&store).unwrap();
        limited
            .set_physical_predecessor(
                store.snapshot_reader(prior_root),
                layerfs_content::file::rope::FileStateRoot(prior_root),
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
                    super::super::CORRESPONDENCE_OPERATION_RESERVATION_BYTES / 131136 * 131136,
                )),
            )
            .unwrap();
        let bytes = vec![b'z'; 40000];
        let built = limited
            .build_complete_with_predecessor(bytes.as_slice(), bytes.len() as u64)
            .unwrap();
        admit(&store.db, built.objects);
        let limited = store.db.physical_storage_receipt().since(before);
        assert_eq!(limited.diag_cursor_operation_limit, 1);
        assert_eq!(limited.diag_cursor_grants, 0);
        assert_eq!(limited.diag_limit_operation_first_empty_count, 1);
        assert!(limited.diag_limit_operation_inherited_empty_count > 0);
        assert_eq!(
            limited.diag_limited_empty_count,
            limited.diag_eligible_count
        );

        let before = store.db.physical_storage_receipt();
        let missing = layerfs_content::ObjectId::for_bytes(b"not an admitted predecessor");
        let mut broken = ObjectBuffer::new(&store).unwrap();
        broken
            .set_physical_predecessor(
                store.snapshot_reader(missing),
                layerfs_content::file::rope::FileStateRoot(missing),
                std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            )
            .unwrap();
        let broken = broken
            .build_complete_with_predecessor(b"failure".as_slice(), 7)
            .unwrap();
        assert!(broken
            .objects
            .consume_prevalidated_pages(|_| Ok(()))
            .is_err());
        let failure = store.db.physical_storage_receipt().since(before);
        assert_ne!(failure.diag_invalid, 0);
        assert_eq!(failure.diag_cursor_grants, 1);
        drop(store);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn duplicate_credit_and_sticky_gauges_are_not_additive_targets() {
        let bytes = layerfs_content::file::extent_codec::encode_chunk_object(b"duplicate").unwrap();
        let mut object = AuthenticatedCanonicalObject::new(bytes, None).unwrap();
        object.1.diagnostic = FILE | COMPLETE;
        object.1.diagnostic_grants = 2;
        object.1.first_span = Some((0, 9));
        let mut stats = Receipt::default();
        occurrence(&mut object, 2, &mut stats);
        assert_eq!(
            (
                stats.diag_occurrence_duplicate_grants,
                object.1.diagnostic_grants
            ),
            (2, 0)
        );
        assert_eq!(stats.diag_eligible_count, 0);
        let counters = crate::telemetry::PhysicalStorageCounters::default();
        counters.note(Receipt {
            diag_selected_pack_last_id: 10,
            diag_selected_pack_count: 2,
            diag_invalid: 1,
            ..Default::default()
        });
        let before = counters.snapshot();
        counters.note(Receipt {
            diag_selected_pack_last_id: 12,
            diag_selected_pack_count: 2,
            ..Default::default()
        });
        let after = counters.snapshot().since(before);
        assert_eq!(
            (
                after.diag_selected_pack_count,
                after.diag_selected_pack_last_id,
                after.diag_invalid
            ),
            (2, 12, 1)
        );
    }

    #[test]
    fn late_race_retains_whole_partial_pack_and_invalidates_unique_cohort() {
        let folder = std::env::temp_dir().join(format!(
            "layerfs-diagnostic-race-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let db = crate::schema::StoreDb::create(folder.join("store.sqlite")).unwrap();
        let make = |value: &[u8]| {
            let mut object = AuthenticatedCanonicalObject::new(
                layerfs_content::file::extent_codec::encode_chunk_object(value).unwrap(),
                None,
            )
            .unwrap();
            object.1.diagnostic = FILE;
            object.1.first_span = Some((0, value.len() as u32));
            object
        };
        let a = make(b"first");
        let b = make(b"second");
        let mut owner = CheckedOutputAdmission::new(&db).unwrap();
        owner.admit_page(vec![a.clone()]).unwrap();
        let mut first_batch = owner.finish().unwrap().final_batch;
        let session = first_batch.1.clone();
        first_batch.2 = false;
        let first = PreparedAdmission::prepare_missing(&db, first_batch).unwrap();
        // Two prepared cohorts deliberately share one writer owner. This preserves
        // the late CAS race without trying to acquire a second independent permit.
        let mut owner = CheckedOutputAdmission::with_session(&db, session.clone(), 0).unwrap();
        owner.admit_page(vec![a, b.clone()]).unwrap();
        let second =
            PreparedAdmission::prepare_missing(&db, owner.finish().unwrap().final_batch).unwrap();
        first.publish(&db, &mut 0, |_, _, _| Ok(())).unwrap();
        let before = db.physical_storage_receipt();
        second.publish(&db, &mut 0, |_, _, _| Ok(())).unwrap();
        let interval = db.physical_storage_receipt().since(before);
        assert_eq!(
            (interval.diag_race_count, interval.diag_new_full_count),
            (1, 1)
        );
        assert_eq!(
            (
                interval.diag_selected_pack_count,
                interval.diag_selected_pack_records,
                interval.diag_selected_unlocated_records
            ),
            (1, 2, 1)
        );
        let all = db.physical_storage_receipt();
        assert_eq!(all.diag_eligible_count, 3);
        assert_eq!(all.diag_new_full_count + all.diag_new_delta_count, 2);
        assert_ne!(
            all.diag_eligible_count,
            all.diag_new_full_count + all.diag_new_delta_count
        );
        assert_eq!(db.read_object_row(b.id).unwrap(), b.bytes);
        drop(session);
        drop(db);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn resumed_file_context_and_first_owner_survive_spill_without_metadata_promotion() {
        use super::super::{ObjectSource, ObjectStore};
        struct Empty;
        impl ObjectSource for Empty {
            fn read_object(&self, id: layerfs_content::ObjectId) -> crate::Result<Vec<u8>> {
                Err(crate::StoreError::MissingObject(id))
            }
        }
        let mut buffer = ObjectBuffer::empty().unwrap();
        let (metadata, _) = layerfs_content::file::rope::build_bytes(&mut buffer, b"same").unwrap();
        buffer.diagnostic_file_payloads();
        let canonical = layerfs_content::file::extent_codec::encode_chunk_object(b"same").unwrap();
        ObjectStore::put_file_payload(&mut buffer, canonical.clone(), 0, 4).unwrap();
        let mut deferred = buffer.into_resumable().unwrap();
        deferred.spill().unwrap();
        let mut resumed = ObjectBuffer::resume_prevalidated(&Empty, deferred);
        assert!(resumed.objects.diagnostic_file_context);
        let new = layerfs_content::file::extent_codec::encode_chunk_object(b"new").unwrap();
        let new_id = ObjectStore::put_file_payload(&mut resumed, new, 4, 3).unwrap();
        let mut deferred = resumed.into_resumable().unwrap();
        // The original metadata occurrence remains the first owner; adding a
        // later file occurrence must not silently rewrite its source label.
        deferred.reachable = super::super::IdOrder::Memory(vec![
            layerfs_content::ObjectId::for_bytes(&canonical),
            new_id,
        ]);
        let deferred = deferred.all_reachable().unwrap();
        let mut observed = Vec::new();
        deferred
            .consume_prevalidated_pages(|page| {
                observed.extend(page.iter().map(|object| (object.id, file(object))));
                Ok(())
            })
            .unwrap();
        assert!(observed.contains(&(layerfs_content::ObjectId::for_bytes(&canonical), false)));
        assert!(observed.contains(&(new_id, true)));
        let _ = metadata;
        eprintln!(
            "D owner sizes: DeferredObjectStore={}, ObjectBuffer={}, AuthenticatedCanonicalObject={}, PhysicalHints={}",
            std::mem::size_of::<DeferredObjectStore>(),
            std::mem::size_of::<ObjectBuffer<'_>>(),
            std::mem::size_of::<AuthenticatedCanonicalObject>(),
            std::mem::size_of::<super::super::PhysicalHints>()
        );
    }
    #[test]
    fn tags_and_role_partition_do_not_treat_metadata_as_file_payload() {
        assert!(valid(FILE | COMPLETE, 2));
        assert!(valid(FILE | INHERITED | OPERATION, 0));
        assert!(!valid(FILE | INHERITED | COMPLETE, 0));
        assert!(!valid(FILE | INHERITED | OPERATION, 1));
        assert!(!valid(FILE | 7, 0));
        let bytes = layerfs_content::file::extent_codec::encode_chunk_object(b"abc").unwrap();
        let mut object = AuthenticatedCanonicalObject::new(bytes, None).unwrap();
        assert!(!file(&object));
        object.1.diagnostic = FILE;
        object.1.has_predecessor = true;
        let mut stats = Receipt::default();
        eligible(&object, &mut stats);
        assert_eq!((stats.diag_missing_span_count, stats.diag_invalid), (1, 1));
        object.1.first_span = Some((0, 3));
        object.1.diagnostic = FILE | INHERITED | OPERATION;
        assert_eq!(state(&object), CORRESPONDENCE_LIMIT);
        object.1.prior_ids[0] = Some(object.id);
        assert_eq!(state(&object), BASE); // partial hints continue, not a forced limit terminal
    }
}

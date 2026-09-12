use super::super::{CheckedOutputAdmission, PhysicalHints};
use super::*;

struct Fixture {
    db: StoreDb,
    folder: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let folder = loop {
            let folder = std::env::temp_dir().join(format!(
                "native-admission-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            match std::fs::create_dir(&folder) {
                Ok(()) => break folder,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("{e}"),
            }
        };
        Self {
            db: StoreDb::create(folder.join("store.sqlite")).unwrap(),
            folder,
        }
    }
    fn prepare(&self, objects: Vec<AuthenticatedCanonicalObject>) -> PreparedAdmission {
        let mut owner = CheckedOutputAdmission::new(&self.db).unwrap();
        owner.admit_page(objects).unwrap();
        PreparedAdmission::prepare_missing(&self.db, owner.finish().unwrap().final_batch).unwrap()
    }
    fn publish(&self, prepared: PreparedAdmission) {
        prepared
            .publish(&self.db, &mut 0, |_, _, _| Ok(()))
            .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.folder);
    }
}
fn object(raw: &[u8], prior: Option<ObjectId>) -> AuthenticatedCanonicalObject {
    let mut object = AuthenticatedCanonicalObject::new(
        layerfs_content::file::extent_codec::encode_chunk_object(raw).unwrap(),
        None,
    )
    .unwrap();
    object.1 = PhysicalHints {
        prior_ids: [prior, None, None, None],
        first_span: Some((0, raw.len() as u32)),
        has_predecessor: prior.is_some(),
        diagnostic: diagnostic::FILE | diagnostic::COMPLETE,
        ..Default::default()
    };
    object
}
fn random() -> Vec<u8> {
    let mut x = 0x174ab28du32;
    (0..32768)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x as u8
        })
        .collect()
}

#[test]
fn compact_namespace_admission_and_reads_require_the_new_schema() {
    let mut f = Fixture::new();
    let path = f.folder.join("legacy9.sqlite");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch(crate::statements::schema::V9).unwrap();
    drop(db);
    f.db = StoreDb::connect(&path).unwrap();
    let canonical = layerfs_content::tree::compact::encode_directory(
        &layerfs_content::tree::compact::DirectoryNode::Leaf(Vec::new()),
    )
    .unwrap();
    let object = AuthenticatedCanonicalObject::new(canonical.clone(), None).unwrap();
    let result = (|| -> Result<()> {
        let mut owner = CheckedOutputAdmission::new(&f.db)?;
        owner.admit_page(vec![object])?;
        PreparedAdmission::prepare_missing(&f.db, owner.finish()?.final_batch)?;
        Ok(())
    })();
    assert!(matches!(
        result,
        Err(StoreError::Integrity(
            "compact namespace requires schema 10"
        ))
    ));
    assert_eq!(
        f.db.reader()
            .unwrap()
            .query_row("SELECT count(*) FROM objects", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );

    let mut native = Fixture::new();
    let object = AuthenticatedCanonicalObject::new(canonical, None).unwrap();
    let id = object.id;
    native.publish(native.prepare(vec![object]));
    let malformed = native.db.path().to_owned();
    let replacement = StoreDb::create(native.folder.join("replacement.sqlite")).unwrap();
    drop(std::mem::replace(&mut native.db, replacement));
    // Disposable deliberately mislabeled Store: no supported downgrade is implied.
    let db = rusqlite::Connection::open(&malformed).unwrap();
    db.execute_batch(
        "DROP TABLE scope_allocator; DROP TABLE metadata_value_groups; PRAGMA user_version=9;",
    )
    .unwrap();
    drop(db);
    let db = StoreDb::connect(&malformed).unwrap();
    assert!(matches!(
        db.read_object_row(id),
        Err(StoreError::Integrity(
            "compact namespace requires schema 10"
        ))
    ));
}

#[test]
fn native_admission_actual_prior_depth_first_hint_and_readback() {
    let f = Fixture::new();
    let mut raw = random();
    let first = object(&raw, None);
    let mut id = first.id;
    f.publish(f.prepare(vec![first]));
    for depth in 1..=5 {
        raw[depth] ^= depth as u8;
        let target = object(&raw, Some(id));
        id = target.id;
        let prepared = f.prepare(vec![target.clone()]);
        assert_eq!(prepared.objects[0].delta, depth <= 4);
        assert!(prepared.objects[0].retained.is_some());
        assert_eq!(
            pack::versioned_header(
                prepared.packs[0][..16].try_into().unwrap(),
                prepared.packs[0].len()
            )
            .unwrap()
            .version,
            pack::Version::Native
        );
        f.publish(prepared);
        assert_eq!(f.db.read_object_row(id).unwrap(), target.bytes);
    }
    assert_eq!(
        f.db.physical_storage_receipt().native_fallback_depth_count,
        1
    );
    let mut target = object(
        b"only first delivered hint",
        Some(ObjectId::for_bytes(b"unavailable")),
    );
    target.1.prior_ids[1] = Some(id);
    let before = f.db.physical_storage_receipt();
    let prepared = f.prepare(vec![target.clone()]);
    assert!(!prepared.objects[0].delta);
    f.publish(prepared);
    let interval = f.db.physical_storage_receipt().since(before);
    assert_eq!(interval.predecessor_hints, 1);
    assert_eq!(interval.native_fallback_unavailable_count, 1);
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
}

#[test]
fn native_admission_late_races_compare_canonical_full_and_prefix() {
    for prefix_first in [false, true] {
        let f = Fixture::new();
        let raw = random();
        let base = object(&raw, None);
        let base_id = base.id;
        f.publish(f.prepare(vec![base]));
        let mut changed = raw.clone();
        changed[100] ^= 1;
        let target = object(&changed, Some(base_id));
        let mut prefix = f.prepare(vec![target.clone()]);
        assert!(prefix.objects[0].delta);
        // Exercise the final canonical CAS recheck with two prepared forms under
        // one owner; independent public admissions now serialize before prepare.
        let session = prefix.session.clone();
        prefix.final_batch = false;
        let full = PreparedAdmission::prepare_missing(
            &f.db,
            super::super::MissingBatch(vec![object(&changed, None)], session.clone(), false, None),
        )
        .unwrap();
        assert!(!full.objects[0].delta);
        let before = f.db.physical_storage_receipt();
        if prefix_first {
            f.publish(prefix);
            f.publish(full);
        } else {
            f.publish(full);
            f.publish(prefix);
        }
        session.retain();
        drop(session);
        let after = f.db.physical_storage_receipt().since(before);
        assert_eq!(after.diag_race_count, 1);
        assert_eq!(after.native_admitted_prefix_count, u64::from(prefix_first));
        assert_eq!(after.native_admitted_full_count, u64::from(!prefix_first));
        assert_eq!(after.diag_selected_pack_count, 1);
        assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
        // Recurrence is filtered by the initial CAS owner and creates no pack.
        let before = f.db.physical_storage_receipt();
        f.publish(f.prepare(vec![target]));
        assert_eq!(
            f.db.physical_storage_receipt()
                .since(before)
                .diag_selected_pack_count,
            0
        );
    }
}

#[test]
fn native_admission_peak_reservations_reject_unowned_buffers() {
    let f = Fixture::new();
    let first = object(b"bounded", None);
    let mut prepared = f.prepare(vec![first]);
    let mut pending = Vec::<NativePrepared>::new();
    let groups = Vec::<pack::EncodedGroup>::new();
    pending.reserve_exact(30000);
    assert!(prepared.native_scratch(&pending, &groups, 0, 0).is_err());
    assert!(prepared.data_reserve(6 * 1024 * 1024).is_err());
    // An already assembled ordinary pack remains charged in the next lane.
    prepared.packs.push(vec![0; 2 * 1024 * 1024]);
    assert!(prepared.native_scratch(&Vec::new(), &groups, 0, 1).is_err());
    let signature_capacity = prepared.small_signatures.capacity();
    let small = AuthenticatedCanonicalObject::new(
        layerfs_content::file::content::encode_small(b"bounded").unwrap(),
        None,
    )
    .unwrap();
    assert!(prepared
        .prepare_small(&f.db, vec![small], &mut Default::default())
        .is_err());
    assert_eq!(
        prepared.small_signatures.capacity(),
        signature_capacity,
        "reject before allocating signature storage"
    );
}

#[test]
fn ordinary_full_batch_preserves_physical_budget_without_small_signature_slots() {
    let f = Fixture::new();
    let mut state = 0x174ab28du32;
    let objects = (0..512)
        .map(|_| {
            let raw = (0..992)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    state as u8
                })
                .collect::<Vec<_>>();
            AuthenticatedCanonicalObject::new(
                layerfs_content::encode_bytes_object(&raw).unwrap(),
                None,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let bytes = objects
        .iter()
        .map(|object| object.bytes.len())
        .sum::<usize>();
    println!(
        "ordinary batch count={} capacity={} bytes={bytes} prepared_slot={} canonical_slot={}",
        objects.len(),
        objects.capacity(),
        std::mem::size_of::<PreparedObject>(),
        std::mem::size_of::<AuthenticatedCanonicalObject>()
    );
    assert!(bytes <= 2 * super::super::INITIALIZATION_SLAB_BYTES);
    let expected = objects
        .iter()
        .map(|object| (object.id, object.bytes.clone()))
        .collect::<Vec<_>>();
    let prepared = f.prepare(objects);
    assert_eq!(prepared.objects.len(), expected.len());
    assert_eq!(prepared.small_signatures.capacity(), 0);
    f.publish(prepared);
    for (id, bytes) in expected {
        assert_eq!(f.db.read_object_row(id).unwrap(), bytes);
    }
}

#[test]
#[ignore = "explicit prepared metadata-cardinality input diagnostic"]
fn metadata_cardinality_prepared_input_reservation_diagnostic() {
    let f = Fixture::new();
    let input = std::env::var_os("LAYERFS_CARDINALITY_DIAGNOSTIC_INPUT")
        .expect("explicit immutable metadata-cardinality fixture path");
    let store = crate::LayerStackStore { db: f.db.clone() };
    store
        .initialize_layerstack(
            crate::EntityName::new("cardinality").unwrap(),
            crate::LayerStackInitialization::Directory(input.into()),
        )
        .unwrap();
}

#[test]
fn native_admission_batches_bound_output_and_stream_late_collision_waves() {
    for corrupt in [false, true] {
        let f = Fixture::new();
        let base = object(b"preexisting retained witness", None);
        f.publish(f.prepare(vec![base.clone()]));
        let raw = random();
        let objects = (0_u64..120)
            .map(|index| {
                let mut bytes = raw[..4096].to_vec();
                bytes[..8].copy_from_slice(&index.to_le_bytes());
                object(&bytes, None)
            })
            .collect::<Vec<_>>();
        assert!(objects.iter().map(|o| o.bytes.len()).sum::<usize>() < 512 * 1024);
        assert!(
            objects
                .iter()
                .map(|o| read::validation_reserve(o.bytes.len()))
                .sum::<usize>()
                > read::VALIDATION_RESERVE
        );
        let prepared = f.prepare(objects.clone());
        assert_eq!(
            prepared.objects.len(),
            objects.len(),
            "a bounded output batch is not a single collision-read wave"
        );
        let session = prepared.session.clone();
        let other = PreparedAdmission::prepare_missing(
            &f.db,
            super::super::MissingBatch(objects.clone(), session.clone(), false, None),
        )
        .unwrap();
        f.publish(other);
        if corrupt {
            let id = objects.last().unwrap().id;
            let location = f.db.object_locations(&[id]).unwrap()[&id];
            let db = f.db.writer().unwrap();
            let mut packed: Vec<u8> = db
                .query_row(
                    "SELECT data FROM object_packs WHERE pack_id=?1",
                    [location.pack],
                    |row| row.get(0),
                )
                .unwrap();
            let header =
                pack::versioned_header(packed[..16].try_into().unwrap(), packed.len()).unwrap();
            let start = 16 + 16 * location.group;
            let entry = pack::versioned_entry(
                packed[start..start + 16].try_into().unwrap(),
                header,
                packed.len(),
            )
            .unwrap();
            let count = u32::from_le_bytes(
                packed[entry.range.start..entry.range.start + 4]
                    .try_into()
                    .unwrap(),
            ) as usize;
            let ends = &packed[entry.range.start + 4..entry.range.start + 4 + 4 * count];
            let range =
                pack::native_record_range(count, ends, entry.range.len(), location.record).unwrap();
            packed[entry.range.start + range.end - 1] ^= 1;
            assert_eq!(
                db.execute(
                    "UPDATE object_packs SET data=?1 WHERE pack_id=?2",
                    rusqlite::params![packed, location.pack]
                )
                .unwrap(),
                1
            );
        }
        let result = prepared.publish(&f.db, &mut 0, |_, _, _| Ok(()));
        if corrupt {
            assert!(
                result.is_err(),
                "later-wave corruption must fail authentication"
            );
            assert_eq!(
                f.db.reader()
                    .unwrap()
                    .query_row("SELECT count(*) FROM objects", [], |row| row
                        .get::<_, i64>(0))
                    .unwrap(),
                1
            );
        } else {
            let (_, metrics) = result.unwrap();
            assert_eq!(metrics.insert.skipped_ids, objects.len() as u64);
            for object in objects {
                assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
            }
        }
        assert_eq!(f.db.read_object_row(base.id).unwrap(), base.bytes);
        drop(session);
    }
}

#[test]
fn collision_comparison_preserves_the_physical_reserve_and_unique_operands() {
    let f = Fixture::new();
    let object = object(b"authenticated comparison operand", None);
    f.publish(f.prepare(vec![object.clone()]));
    let known = f.db.object_locations(&[object.id]).unwrap();
    let mut supplied = vec![(object.id, object.bytes.as_slice())];
    assert!(matches!(
        compare(
            &f.db,
            &known,
            &mut supplied,
            &mut ObjectInsertMetrics::default(),
            read::VALIDATION_RESERVE
        ),
        Err(StoreError::Integrity("comparison physical reservation"))
    ));
    let mut locations = known
        .iter()
        .map(|(&id, &location)| (id, location))
        .collect::<Vec<_>>();
    assert!(f
        .db
        .visit_locations_with_reserve(&mut locations, read::VALIDATION_RESERVE + 1, |_| Ok(()))
        .is_err());
    assert!(f
        .db
        .visit_locations_with_reserve(&mut locations, 0, |_| Ok(()))
        .is_err());
    supplied.push(supplied[0]);
    assert!(matches!(
        compare(
            &f.db,
            &known,
            &mut supplied,
            &mut ObjectInsertMetrics::default(),
            0
        ),
        Err(StoreError::Integrity("duplicate comparison operand"))
    ));
    assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
}

#[test]
fn failed_admission_reclaims_private_packs_and_preserves_waiting_owners_and_bases() {
    let f = Fixture::new();
    let raw = random();
    let base = object(&raw, None);
    let base_id = base.id;
    f.publish(f.prepare(vec![base.clone()]));
    let mut retained_bytes = raw.clone();
    retained_bytes[100] ^= 1;
    let retained = object(&retained_bytes, Some(base_id));
    f.publish(f.prepare(vec![retained.clone()]));
    let baseline_packs: i64 =
        f.db.reader()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM object_packs", [], |row| row.get(0))
            .unwrap();
    let mut private_bytes = retained_bytes;
    private_bytes[200] ^= 1;
    let private = object(&private_bytes, Some(retained.id));
    let mut owner = CheckedOutputAdmission::new(&f.db).unwrap();
    owner
        .admit_page(vec![base.clone(), private.clone()])
        .unwrap();
    owner.flush().unwrap();
    assert_eq!(f.db.read_object_row(private.id).unwrap(), private.bytes);

    std::thread::scope(|scope| {
        let (ready, started) = std::sync::mpsc::sync_channel(1);
        let db = f.db.clone();
        let target = private.clone();
        let waiting = scope.spawn(move || {
            ready.send(()).unwrap();
            let mut owner = CheckedOutputAdmission::new(&db).unwrap();
            owner.admit_page(vec![target]).unwrap();
            let finished = owner.finish().unwrap();
            PreparedAdmission::prepare_missing(&db, finished.final_batch)
                .unwrap()
                .publish(&db, &mut 0, |_, _, _| Ok(()))
                .unwrap();
        });
        started.recv().unwrap();
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        while f.db.operation_waiters() == 0 {
            assert!(
                Instant::now() < deadline,
                "second owner must wait at the publication gate"
            );
            std::thread::yield_now();
        }
        let finished = owner.finish().unwrap();
        let failed = PreparedAdmission::prepare_missing(&f.db, finished.final_batch)
            .unwrap()
            .publish(&f.db, &mut 0, |_, _, _| {
                Err::<(), _>(StoreError::Integrity("publication proof"))
            });
        assert!(matches!(
            failed,
            Err(StoreError::Integrity("publication proof"))
        ));
        waiting.join().unwrap();
    });
    assert_eq!(f.db.read_object_row(base.id).unwrap(), base.bytes);
    assert_eq!(f.db.read_object_row(retained.id).unwrap(), retained.bytes);
    assert_eq!(f.db.read_object_row(private.id).unwrap(), private.bytes);
    let packs: i64 =
        f.db.reader()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM object_packs", [], |row| row.get(0))
            .unwrap();
    assert_eq!(packs, baseline_packs + 1, "failed private pack was reclaimed before the waiting owner re-admitted its canonical bytes");
}

#[test]
fn failed_cleanup_quarantines_writes_and_keeps_preexisting_reads() {
    let f = Fixture::new();
    let base = object(&random(), None);
    f.publish(f.prepare(vec![base.clone()]));
    let mut changed = random();
    changed[100] ^= 1;
    let private = object(&changed, Some(base.id));
    let mut owner = CheckedOutputAdmission::new(&f.db).unwrap();
    owner.admit_page(vec![private]).unwrap();
    owner.flush().unwrap();
    f.db.writer().unwrap().execute_batch(
        "CREATE TEMP TRIGGER fail_cleanup BEFORE DELETE ON objects BEGIN SELECT RAISE(ABORT, 'cleanup proof'); END;",
    ).unwrap();
    let finished = owner.finish().unwrap();
    let failed = PreparedAdmission::prepare_missing(&f.db, finished.final_batch)
        .unwrap()
        .publish(&f.db, &mut 0, |_, _, _| {
            Err::<(), _>(StoreError::Integrity("publication proof"))
        });
    assert!(failed
        .err()
        .unwrap()
        .to_string()
        .contains("admission cleanup failed"));
    assert!(matches!(
        f.db.writer(),
        Err(StoreError::Integrity(
            "Store writes quarantined after failed admission cleanup"
        ))
    ));
    assert!(matches!(
        f.db.enter_operation(),
        Err(StoreError::Integrity(
            "Store writes quarantined after failed admission cleanup"
        ))
    ));
    assert_eq!(f.db.read_object_row(base.id).unwrap(), base.bytes);
}

#[test]
fn admission_watermark_preserves_preexisting_counts_and_dependency_authentication() {
    for missing_dependency in [false, true] {
        let f = Fixture::new();
        let base = object(b"preexisting witness", None);
        f.publish(f.prepare(vec![base.clone()]));
        let fresh = object(b"new owned payload", None);
        let mut owner = CheckedOutputAdmission::new(&f.db).unwrap();
        for _ in 0..3 {
            owner.admit_page(vec![base.clone(), fresh.clone()]).unwrap();
            owner.flush().unwrap();
        }
        assert_eq!(owner.seen.count, 1);
        assert_eq!(
            (
                owner.checked.candidate_objects,
                owner.checked.inserted_objects,
                owner.checked.reused_objects
            ),
            (2, 1, 1)
        );
        assert_eq!(owner.receipt.preexisting_reused_objects, 1);
        let invalid = if missing_dependency {
            use layerfs_content::file::{extent::FileStateV3, extent_codec};
            AuthenticatedCanonicalObject::new(
                extent_codec::encode_file_state(FileStateV3 {
                    logical_len: 1,
                    extent_count: 1,
                    tree_level: 0,
                    profile_id: extent_codec::profile_id(),
                    mapping_root: ObjectId::for_bytes(b"missing dependency"),
                })
                .unwrap(),
                None,
            )
            .unwrap()
        } else {
            let mut corrupt = fresh.clone();
            *corrupt.0.bytes.last_mut().unwrap() ^= 1;
            corrupt
        };
        let result = owner
            .admit_page(vec![invalid])
            .and_then(|_| owner.finish().map(|_| ()));
        assert!(
            matches!(result, Err(StoreError::Integrity(message)) if message == if missing_dependency { "new object dependency missing" } else { "object collision" })
        );
        assert_eq!(f.db.read_object_row(base.id).unwrap(), base.bytes);
        assert!(f.db.read_object_row(fresh.id).is_err());
    }
}

#[test]
fn locator_publication_is_sorted_without_changing_native_pack_bytes() {
    let f = Fixture::new();
    let random = random();
    let objects = (0_u64..200)
        .map(|i| {
            let mut raw = random[..1024].to_vec();
            raw[..8].copy_from_slice(&i.to_be_bytes());
            object(&raw, None)
        })
        .collect::<Vec<_>>();
    let prepared = f.prepare(objects.clone());
    let packs = prepared.packs.clone();
    f.db.writer().unwrap().execute_batch("CREATE TEMP TABLE insertion_order(id BLOB); CREATE TEMP TRIGGER capture_order AFTER INSERT ON main.objects BEGIN INSERT INTO insertion_order VALUES (new.object_id); END;").unwrap();
    f.publish(prepared);
    let inserted: Vec<Vec<u8>> =
        f.db.reader()
            .unwrap()
            .prepare("SELECT id FROM insertion_order ORDER BY rowid")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
    let mut expected = objects
        .iter()
        .map(|o| o.id.as_bytes().to_vec())
        .collect::<Vec<_>>();
    expected.sort_unstable();
    assert_eq!(
        inserted, expected,
        "locator insertion must retain canonical-key locality"
    );
    let stored: Vec<Vec<u8>> =
        f.db.reader()
            .unwrap()
            .prepare("SELECT data FROM object_packs ORDER BY pack_id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
    assert_eq!(
        stored, packs,
        "sorting SQL locators must not change physical encoding"
    );
    for object in objects {
        assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
    }
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn unchanged_absence_proof_avoids_reprobe_but_intervening_publication_rechecks() {
    for stage in 0..3 {
        let f = Fixture::new();
        f.publish(f.prepare(vec![object(b"preexisting witness", None)]));
        let objects = (0_u64..200)
            .map(|i| object(&i.to_le_bytes(), None))
            .collect::<Vec<_>>();
        let mut owner = CheckedOutputAdmission::new(&f.db).unwrap();
        owner.admit_page(objects.clone()).unwrap();
        owner.probe_incoming().unwrap();
        let publish_other = |session| {
            let other = PreparedAdmission::prepare_missing(
                &f.db,
                super::super::MissingBatch(objects.clone(), session, false, None),
            )
            .unwrap();
            f.publish(other);
        };
        if stage == 1 {
            publish_other(owner.session.clone());
        }
        let prepared =
            PreparedAdmission::prepare_missing(&f.db, owner.finish().unwrap().final_batch).unwrap();
        if stage == 2 {
            publish_other(prepared.session.clone());
        }
        crate::schema::reset_sql_trace();
        let (_, metrics) = prepared.publish(&f.db, &mut 0, |_, _, _| Ok(())).unwrap();
        let queries = crate::schema::sql_trace()
            .iter()
            .filter(|s| s.contains("FROM objects WHERE object_id IN ("))
            .count();
        assert_eq!(
            queries > 0,
            stage != 0,
            "only an unchanged publication epoch proves absence"
        );
        assert_eq!(metrics.insert.skipped_ids, if stage != 0 { 200 } else { 0 });
        for object in objects {
            assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
        }
    }
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn streaming_absence_proofs_advance_only_over_disjoint_owned_batches() {
    let f = Fixture::new();
    f.publish(f.prepare(vec![object(b"preexisting witness", None)]));
    let objects = (0_u64..1200)
        .map(|i| object(&i.to_le_bytes(), None))
        .collect::<Vec<_>>();
    crate::schema::reset_sql_trace();
    f.publish(f.prepare(objects.clone()));
    let queries = crate::schema::sql_trace()
        .iter()
        .filter(|s| s.contains("FROM objects WHERE object_id IN ("))
        .count();
    assert_eq!(
        queries,
        1200_usize.div_ceil(OBJECT_PAGE_COUNT),
        "one initial lookup per bounded page; no repeated negative lookup"
    );
    for object in objects {
        assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
    }
}

#[test]
fn small_content_upper_range_exact_cas_reuse() {
    let f = Fixture::new();
    let mut raw = random().repeat(3);
    let small = |raw: &[u8], prior| {
        let mut object = AuthenticatedCanonicalObject::new(
            layerfs_content::file::content::encode_small(raw).unwrap(),
            None,
        )
        .unwrap();
        object.1.prior_ids[0] = prior;
        object
    };
    let full = small(&raw, None);
    f.publish(f.prepare(vec![full.clone()]));
    raw[70000] ^= 1;
    let delta = small(&raw, Some(full.id));
    let prepared = f.prepare(vec![delta.clone()]);
    assert!(prepared.objects[0].delta);
    f.publish(prepared);
    let legacy = AuthenticatedCanonicalObject::new(
        layerfs_content::encode_bytes_object(&raw).unwrap(),
        None,
    )
    .unwrap();
    f.publish(f.prepare(vec![legacy.clone()]));
    for object in [full, delta, legacy] {
        let before = f.db.physical_storage_receipt();
        f.publish(f.prepare(vec![object.clone()]));
        assert_eq!(
            f.db.physical_storage_receipt()
                .since(before)
                .diag_selected_pack_count,
            0
        );
        let known = f.db.object_locations(&[object.id]).unwrap();
        let mut changed = object.bytes.clone();
        *changed.last_mut().unwrap() ^= 1;
        assert!(compare(
            &f.db,
            &known,
            &mut vec![(object.id, &changed)],
            &mut ObjectInsertMetrics::default(),
            0
        )
        .is_err());
        assert_eq!(f.db.read_object_row(object.id).unwrap(), object.bytes);
    }
}

#[path = "issue100_diagnostic.rs"]
mod issue100_diagnostic;

#[path = "small_chain_tests.rs"]
mod small_chain_tests;

#[test]
#[ignore = "fixed external Git2.47.1 programs, identical original fixture pairs"]
fn issue100_identical_base_git_matcher() {
    let root = std::path::Path::new(
        "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/git-matcher-study",
    );
    for name in [
        "translation-growth",
        "catalog-growth",
        "icons-growth",
        "rename-ledger",
        "tool-schemas",
    ] {
        let base = std::fs::read(root.join(format!("{name}.base"))).unwrap();
        let target = std::fs::read(root.join(format!("{name}.target"))).unwrap();
        let program = std::fs::read(root.join(format!("{name}.gitdelta"))).unwrap();
        assert!(base.len() < 131072 && target.len() < 131072 && program.len() < 131072);
        let mut encoder = pack::NativeEncoder::new_small().unwrap();
        let base_full = encoder.compress(&base, None).unwrap();
        let started = Instant::now();
        let full = encoder.compress(&target, None).unwrap();
        let full_ns = started.elapsed().as_nanos();
        let started = Instant::now();
        let prefix = encoder.compress(&target, Some(&base)).unwrap();
        let prefix_ns = started.elapsed().as_nanos();
        let started = Instant::now();
        let git = encoder.compress(&program, None).unwrap();
        let git_compress_ns = started.elapsed().as_nanos();
        drop(encoder);
        assert_eq!(
            pack::small_decompress(&prefix, target.len(), Some(&base)).unwrap(),
            target
        );
        assert_eq!(
            pack::small_decompress(&full, target.len(), None).unwrap(),
            target
        );
        // Upstream patch_delta already replayed this exact program to target.
        // Equality after decoding preserves that proof without another parser.
        assert_eq!(
            pack::small_decompress(&git, program.len(), None).unwrap(),
            program
        );
        println!("pair\t{name}\tbase_raw={}\ttarget_raw={}\tbase_FULL_cost={}\tFULL_target_cost={}\tprefix_DELTA_cost={}\tgit_program_raw={}\tgit_program_DELTA_cost={}\tFULL_encode_ns={full_ns}\tprefix_encode_ns={prefix_ns}\tgit_program_compress_ns={git_compress_ns}\tverified=true", base.len(),target.len(),base_full.len()+25,full.len()+25,prefix.len()+57,program.len(),git.len()+61);
    }
}

#[path = "small_candidate_tests.rs"]
mod small_candidate_tests;

#[path = "metadata_tests.rs"]
mod metadata_tests;

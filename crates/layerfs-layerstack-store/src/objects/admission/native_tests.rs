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
            super::super::MissingBatch(vec![object(&changed, None)], session.clone(), false),
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

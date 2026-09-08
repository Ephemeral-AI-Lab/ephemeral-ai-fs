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
        let prefix = f.prepare(vec![target.clone()]);
        assert!(prefix.objects[0].delta);
        let full = f.prepare(vec![object(&changed, None)]);
        assert!(!full.objects[0].delta);
        let before = f.db.physical_storage_receipt();
        if prefix_first {
            f.publish(prefix);
            f.publish(full);
        } else {
            f.publish(full);
            f.publish(prefix);
        }
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

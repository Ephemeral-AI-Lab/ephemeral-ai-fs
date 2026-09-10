use super::*;

fn small(raw: &[u8]) -> AuthenticatedCanonicalObject {
    AuthenticatedCanonicalObject::new(
        layerfs_content::file::content::encode_small(raw).unwrap(), None,
    ).unwrap()
}

#[test]
fn selected_small_candidate_reuse_late_cas_and_rollback() {
    let f = Fixture::new();
    let old = small(b"preexisting unrelated immutable content remains after rollback");
    f.publish(f.prepare(vec![old.clone()]));
    let raw = random().repeat(3);
    let base = small(&raw);
    let mut prepared = f.prepare(vec![base.clone()]);
    let session = prepared.session.clone();
    prepared.final_batch = false;
    f.publish(prepared);
    let prepare = |object: AuthenticatedCanonicalObject| {
        PreparedAdmission::prepare_missing(&f.db,
            crate::objects::MissingBatch(vec![object], session.clone(), false, None)).unwrap()
    };
    let mut changed = b"shifted new path\n".to_vec();
    changed.extend_from_slice(&raw);
    changed[70000] ^= 1;
    let target = small(&changed);
    let prepared = prepare(target.clone());
    assert!(prepared.objects[0].delta);
    // Cache contains only a selected FULL, so this new-path target uses kind 1.
    assert_eq!(&prepared.packs[0][8..12], &4u32.to_le_bytes());
    assert_eq!(prepared.packs[0][20], 1);
    f.publish(prepare(target.clone()));
    let before = f.db.physical_storage_receipt();
    f.publish(prepared);
    assert_eq!(f.db.physical_storage_receipt().since(before).diag_selected_pack_count, 0);
    assert_eq!(f.db.small_physical_base(target.id).unwrap(), Some(base.id));
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);

    // Prepared-only FULLs cannot become cache bases before selection.
    let unrelated = small(&vec![b'x'; 9000]);
    let pending = prepare(unrelated.clone());
    assert!(!pending.objects[0].delta);
    let mut other = vec![b'x'; 9000];
    other[100] = b'y';
    assert!(!prepare(small(&other)).objects[0].delta);
    drop(pending);
    session.rollback().unwrap();
    assert!(session.ensure_active().is_err());
    assert!(f.db.object_locations(&[base.id, target.id]).unwrap().is_empty());
    assert_eq!(f.db.read_object_row(old.id).unwrap(), old.bytes);
}

#[test]
#[ignore = "uses the sealed original Git matcher fixture"]
fn selected_small_candidate_original_tool_schema_pair() {
    let root = std::path::Path::new("/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/git-matcher-study");
    let f = Fixture::new();
    let base = small(&std::fs::read(root.join("tool-schemas.base")).unwrap());
    let target = small(&std::fs::read(root.join("tool-schemas.target")).unwrap());
    let mut prepared = f.prepare(vec![base.clone()]);
    let session = prepared.session.clone();
    prepared.final_batch = false;
    f.publish(prepared);
    let prepared = PreparedAdmission::prepare_missing(&f.db,
        crate::objects::MissingBatch(vec![target.clone()], session.clone(), false, None)).unwrap();
    assert!(prepared.objects[0].delta);
    f.publish(prepared);
    assert_eq!(f.db.small_physical_base(target.id).unwrap(), Some(base.id));
    assert_eq!(f.db.read_object_row(target.id).unwrap(), target.bytes);
    session.retain();
}

#[test]
fn selected_small_candidate_fingerprint_bounds() {
    use super::super::super::small_candidates::{signature, Candidates};
    let raw = random();
    let id = ObjectId::for_bytes(&raw);
    let target = ObjectId::for_bytes(b"target");
    let mut candidates = Candidates::new();
    candidates.insert(id, signature(&raw));
    let mut shifted = b"three-byte-shift".to_vec();
    shifted.extend_from_slice(&raw);
    assert_eq!(candidates.find(target, &signature(&shifted)), Some(id));
    assert_eq!(candidates.find(id, &signature(&raw)), None);
    assert_eq!(candidates.find(target, &signature(b"unrelated short literal")), None);
    assert_eq!(candidates.find(target, &signature(b"")), None);
}

#[test]
fn selected_small_candidate_compact_references_and_eviction() {
    use super::super::super::small_candidates::Candidates;
    let pair = |a| [a, a + 1, u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX];
    let base = ObjectId::for_bytes(b"base");
    let target = ObjectId::for_bytes(b"target");
    let mut candidates = Candidates::new();
    candidates.insert(base, pair(2));
    // These keys collided in the old duplicated 1024-slot representation.
    candidates.insert(ObjectId::for_bytes(b"other"), pair(1026));
    assert_eq!(candidates.find(target, &pair(2)), Some(base));
    for i in 0..1022_u64 {
        candidates.insert(ObjectId::for_bytes(&i.to_le_bytes()), pair((i + 10) * 8));
    }
    assert_eq!(candidates.find(target, &pair(2)), Some(base));
    candidates.insert(ObjectId::for_bytes(b"replacement"), pair(6000));
    assert_eq!(candidates.find(target, &pair(2)), None);
    assert_eq!(candidates.find(target, &pair(1026)), Some(ObjectId::for_bytes(b"other")));
    candidates.insert(ObjectId::for_bytes(b"collision"), pair(1026 + 8192));
    assert_eq!(candidates.find(target, &pair(1026)), None);
}

#[test]
fn selected_small_candidate_retained_handoff_rollback_and_cold_reopen() {
    let folder = Fixture::new();
    let path = folder.folder.join("retained.sqlite");
    let db = StoreDb::create(&path).unwrap();
    let raw = random().repeat(3);
    let base = small(&raw);
    let mut changed = raw.clone();
    changed[70000] ^= 1;
    let target = small(&changed);
    let private = small(&raw.iter().map(|byte| !byte).collect::<Vec<_>>());
    {
        let prepare = |object| {
            let mut owner = CheckedOutputAdmission::new(&db).unwrap();
            owner.admit_page(vec![object]).unwrap();
            PreparedAdmission::prepare_missing(&db, owner.finish().unwrap().final_batch).unwrap()
        };
        let publish = |prepared: PreparedAdmission| {
            prepared.publish(&db, &mut 0, |_, _, _| Ok(())).unwrap();
        };
        let mut initial = prepare(base.clone());
        let session = initial.session.clone();
        initial.final_batch = false;
        publish(initial);
        session.retain();
        drop(session);
        let next = prepare(target.clone());
        assert!(next.objects[0].delta);
        assert_eq!(&next.packs[0][8..12], &4u32.to_le_bytes());
        assert_eq!(next.packs[0][20], 1);
        publish(next);
        assert_eq!(db.small_physical_base(target.id).unwrap(), Some(base.id));
        assert_eq!(db.read_object_row(target.id).unwrap(), target.bytes);

        let mut pending = prepare(private.clone());
        assert!(!pending.objects[0].delta);
        let session = pending.session.clone();
        pending.final_batch = false;
        publish(pending);
        session.rollback().unwrap();
        drop(session);
        assert!(db.take_small_candidates().is_none());
        assert!(db.object_locations(&[private.id]).unwrap().is_empty());
        assert_eq!(db.read_object_row(base.id).unwrap(), base.bytes);
        assert_eq!(db.read_object_row(target.id).unwrap(), target.bytes);

        let mut retained = prepare(private.clone());
        let session = retained.session.clone();
        retained.final_batch = false;
        publish(retained);
        session.retain();
        drop(session);
    }
    let cache = db.take_small_candidates().expect("retained cache is warm");
    db.return_small_candidates(cache);
    drop(db);
    let reopened = StoreDb::connect(&path).unwrap();
    assert!(reopened.take_small_candidates().is_none());
    assert_eq!(reopened.read_object_row(target.id).unwrap(), target.bytes);
    assert_eq!(reopened.read_object_row(private.id).unwrap(), private.bytes);
    drop(reopened);
}

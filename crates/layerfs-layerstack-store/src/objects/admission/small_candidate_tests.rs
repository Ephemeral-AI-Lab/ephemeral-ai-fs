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
    assert_eq!(prepared.packs[0][32], 1);
    f.publish(prepare(target.clone()));
    let mut next_raw = changed.clone();
    next_raw[80000] ^= 1;
    let next = small(&next_raw);
    let chained = prepare(next.clone());
    assert!(chained.objects[0].delta);
    assert_eq!(chained.packs[0][32], 2);
    f.publish(chained);
    assert_eq!(f.db.small_physical_base(next.id).unwrap(), Some(target.id));
    assert_eq!(f.db.read_object_row(next.id).unwrap(), next.bytes);
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

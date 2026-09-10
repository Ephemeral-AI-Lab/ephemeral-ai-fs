use layerfs_content::filesystem::{self, ContentChange, LogicalCounters};
use layerfs_content::object::access::ObjectStore;
use layerfs_content::tree::compact::InodeSerial;
use layerfs_content::{CanonicalPath, CoreError, CoreResult, ObjectId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Store {
    objects: BTreeMap<ObjectId, Vec<u8>>,
    highwater: u64,
}
impl ObjectStore for Store {
    fn compact_namespace(&self) -> bool { true }
    fn small_content_format(&self) -> bool { true }
    fn allocate_inode_serial(&mut self, _: ObjectId) -> CoreResult<InodeSerial> {
        self.highwater += 1;
        InodeSerial::new(self.highwater)
    }
    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> { self.objects.get(&id).cloned().ok_or(CoreError::MissingObject) }
    fn put(&mut self, bytes: &[u8]) -> CoreResult<ObjectId> {
        let id = ObjectId::for_bytes(bytes);
        self.objects.insert(id, bytes.to_vec());
        Ok(id)
    }
}

fn path(name: &str) -> CanonicalPath { CanonicalPath::new(name).unwrap() }
fn content(store: &Store, root: ObjectId, name: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    filesystem::stream(store, root, &path(name), &mut bytes).unwrap(); bytes
}

#[test]
fn compact_logical_lifecycle_preserves_scopes_hardlinks_and_history() {
    let mut store = Store::default();
    let genesis = filesystem::empty_root(&mut store, [31; 32]).unwrap();
    let scope = filesystem::namespace(&store, genesis).unwrap().scope.unwrap();
    let original: Vec<_> = (0..131071).map(|n| (n % 251) as u8).collect();
    let first = filesystem::apply_changes(&mut store, genesis, &[
        ContentChange::Write { path: "a".into(), bytes: original.clone(), mode: 0o640 },
        ContentChange::Mkdir { path: "sub".into(), mode: 0o700 },
        ContentChange::Symlink { path: "link".into(), target: b"a".to_vec() },
        ContentChange::HardLink { source: "a".into(), target: "alias".into() },
    ], [32; 32]).unwrap().root_id;
    let original_inode = filesystem::resolve(&store, first, &path("a"), &mut LogicalCounters::default()).unwrap().inode;
    assert_eq!(filesystem::resolve(&store, first, &path("alias"), &mut LogicalCounters::default()).unwrap().inode, original_inode);
    assert_eq!(filesystem::stat(&store, first, &path("alias")).unwrap().0.namespace_ref_count, 2);
    let second = filesystem::apply_changes(&mut store, first, &[
        ContentChange::Write { path: "a".into(), bytes: b"changed".to_vec(), mode: 0o600 },
        ContentChange::Rename { source: "a".into(), target: "sub/b".into() },
        ContentChange::SetMtime { path: "alias".into(), seconds: 1234, nanoseconds: 5678 },
    ], [33; 32]).unwrap().root_id;
    assert_eq!(content(&store, first, "a"), original);
    assert_eq!(content(&store, second, "alias"), b"changed");
    assert_eq!(content(&store, second, "sub/b"), b"changed");
    assert_eq!(filesystem::resolve(&store, second, &path("sub/b"), &mut LogicalCounters::default()).unwrap().inode, original_inode);
    let third = filesystem::apply_changes(&mut store, second, &[
        ContentChange::Remove { path: "alias".into() },
        ContentChange::Write { path: "a".into(), bytes: b"recreated".to_vec(), mode: 0o644 },
    ], [34; 32]).unwrap().root_id;
    assert_ne!(filesystem::resolve(&store, third, &path("a"), &mut LogicalCounters::default()).unwrap().inode, original_inode);
    assert_eq!(filesystem::stat(&store, third, &path("sub/b")).unwrap().0.namespace_ref_count, 1);
    assert_eq!(filesystem::namespace(&store, third).unwrap().scope, Some(scope));
    let mut changes = Vec::new();
    filesystem::diff_roots(&store, first, third, |change| { changes.push(change); Ok(()) }).unwrap();
    assert!(!changes.is_empty());

    // Full closure is checked against real canonical objects. No separate inode
    // records or directory-state wrappers are needed by a compact history.
    let mut seen = BTreeSet::new();
    let mut pending = vec![genesis, first, second, third];
    while let Some(id) = pending.pop() {
        if !seen.insert(id) { continue; }
        let bytes = store.get(id).unwrap();
        layerfs_content::authenticate_identity(&bytes, id).unwrap();
        let value = layerfs_content::decode_bytes_object(&bytes).unwrap();
        assert!(!value.starts_with(b"LFS4INO\0") && !value.starts_with(b"LFS4DIR\0"));
        pending.extend(layerfs_content::object::references::referenced_objects(&bytes).unwrap());
    }
    let mut foreign = filesystem::namespace(&store, third).unwrap();
    foreign.scope = Some(ObjectId::for_bytes(b"another allocation domain"));
    let foreign = store.put(&layerfs_content::tree::directory::codec::encode_namespace_root(foreign).unwrap()).unwrap();
    assert!(filesystem::diff_roots(&store, third, foreign, |_| Ok(())).is_err());
    assert!(filesystem::reconcile_roots(&mut store, first, third, foreign).is_err());
    assert!(filesystem::replace_paths_from_snapshot(&mut store, third, foreign, &[path("a")]).is_err());
    let inode_table = filesystem::namespace(&store, third).unwrap().inode_table_root;
    let bytes = store.objects.get_mut(&inode_table).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    assert!(filesystem::stat(&store, third, &path("a")).is_err());
}

#[test]
fn compact_reconciliation_merges_independent_content_and_metadata_edits() {
    let mut store = Store::default();
    let empty = filesystem::empty_root(&mut store, [61; 32]).unwrap();
    let base = filesystem::apply_changes(&mut store, empty, &[ContentChange::Write { path: "file".into(), bytes: b"base".to_vec(), mode: 0o644 }], [62; 32]).unwrap().root_id;
    let source = filesystem::set_mode(&mut store, base, &path("file"), 0o600).unwrap().root();
    let destination = filesystem::replace_range(&mut store, base, &path("file"), 0, 4, b"new bytes".as_slice()).unwrap().root();
    let merged = filesystem::reconcile_roots(&mut store, base, source, destination).unwrap().unwrap().root();
    assert_eq!(content(&store, merged, "file"), b"new bytes");
    assert_eq!(filesystem::stat(&store, merged, &path("file")).unwrap().0.metadata_root,
        filesystem::stat(&store, source, &path("file")).unwrap().0.metadata_root);
    assert_eq!(content(&store, base, "file"), b"base");
    let restored = filesystem::replace_paths_from_snapshot(&mut store, merged, base, &[path("file")]).unwrap();
    assert_eq!(content(&store, restored, "file"), b"base");
}

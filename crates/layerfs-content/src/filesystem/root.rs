use super::change::metadata;
use crate::object::access::ObjectStore;
use crate::tree::directory::codec::{encode_namespace_root, profile_id};
use crate::tree::directory::empty_directory;
use crate::tree::inode::codec::encode_inode_record;
use crate::tree::inode::{inode_table_from_root, InodeId, InodeKind, InodeRecordV1};
use crate::tree::NamespaceRootV1;
use crate::{CoreResult, ObjectId};

pub fn empty_root<S: ObjectStore>(store: &mut S, seed: [u8; 32]) -> CoreResult<ObjectId> {
    let scope = store.compact_namespace().then(|| crate::tree::compact::scope_for_seed(seed));
    let root_inode = match scope {
        Some(scope) => store.allocate_inode_serial(scope)?.inode_key(),
        None => InodeId::allocate(seed, 0),
    };
    let directory = empty_directory(store)?;
    let metadata = metadata(store, InodeKind::Directory, 0o755)?;
    let record = InodeRecordV1 {
        kind: InodeKind::Directory,
        namespace_ref_count: 0,
        content_root: directory.0,
        metadata_root: metadata,
    };
    let table = if scope.is_some() {
        crate::tree::inode::InodeTableRoot(store.put_owned(crate::tree::compact::encode_inode(
            &crate::tree::compact::InodeNode::Leaf(vec![(crate::tree::compact::InodeSerial::from_inode_key(root_inode)?, record)]),
        )?)?)
    } else {
        let record = store.put(&encode_inode_record(record)?)?;
        inode_table_from_root(store, root_inode, record)?
    };
    store.put(&encode_namespace_root(NamespaceRootV1 {
        scope,
        profile_id: if scope.is_some() { crate::tree::compact::profile_id() } else { profile_id() },
        root_directory_inode: root_inode,
        inode_table_root: table.0,
    })?)
}

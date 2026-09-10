use crate::tree::inode::InodeId;
use crate::ObjectId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamespaceRootV1 {
    /// Full allocation domain for serial keys; None denotes the legacy hash-key format.
    pub scope: Option<ObjectId>,
    pub profile_id: ObjectId,
    pub root_directory_inode: InodeId,
    pub inode_table_root: ObjectId,
}

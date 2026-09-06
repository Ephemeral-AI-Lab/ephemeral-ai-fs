#![forbid(unsafe_code)]

pub mod backing;
pub mod file_edit;
mod limits;
pub use limits::ResourcePolicy;

use layerfs_content::file::rope::FileStateRoot;
use layerfs_content::tree::directory::DirectoryStateRoot;
use layerfs_content::tree::inode::InodeId;
use std::collections::{BTreeMap, BTreeSet};

/// Portable failures retain the native Workspace's error classes and messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidInput(&'static str),
    Integrity(&'static str),
    NotFound(&'static str),
}
pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(pub u64);
pub const ROOT: NodeId = NodeId(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    File,
    Directory,
    Symlink,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Attr {
    pub node: NodeId,
    pub size: u64,
    pub kind: Kind,
    pub mode: u32,
    pub links: u32,
    pub mtime_seconds: i64,
    pub mtime_nanoseconds: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Data {
    File(FileData),
    Directory(DirectoryData),
    Symlink(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileData {
    Base {
        root: FileStateRoot,
        len: u64,
    },
    Edited {
        base: Option<(FileStateRoot, u64)>,
        spool_high_water: u64,
        pieces: crate::file_edit::PieceTree,
        edits: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectoryData {
    pub base: Option<DirectoryStateRoot>,
    pub changes: BTreeMap<Vec<u8>, Option<NodeId>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub canonical: Option<InodeId>,
    pub paths: BTreeSet<String>,
    pub mode: u32,
    pub links: u32,
    pub pins: u32,
    pub mtime_seconds: i64,
    pub mtime_nanoseconds: u32,
    pub data: Data,
}

impl Node {
    pub fn attr(&self, node: NodeId) -> Attr {
        let (kind, size) = match &self.data {
            Data::File(FileData::Base { len, .. }) => (Kind::File, *len),
            Data::File(FileData::Edited { pieces, .. }) => (Kind::File, pieces.len()),
            Data::Directory(_) => (Kind::Directory, 0),
            Data::Symlink(target) => (Kind::Symlink, target.len() as u64),
        };
        Attr {
            node,
            size,
            kind,
            mode: self.mode,
            links: self.links,
            mtime_seconds: self.mtime_seconds,
            mtime_nanoseconds: self.mtime_nanoseconds,
        }
    }
}

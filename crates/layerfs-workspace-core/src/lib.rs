#![forbid(unsafe_code)]

pub mod backing;
pub mod file_edit;
mod limits;
pub use limits::ResourcePolicy;

use layerfs_content::file::rope::FileStateRoot;
use layerfs_content::tree::directory::DirectoryStateRoot;
use layerfs_content::tree::inode::InodeId;
use std::collections::{BTreeMap, BTreeSet, HashMap};

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

/// An immutable owned description. The adapter performs acquisition and physical
/// reads after releasing live-state locks; the retained pieces keep old bytes alive.
pub struct ReadPlan {
    pub requested: u64,
    pub source: ReadSource,
    pub tree_visits: usize,
}

pub enum ReadSource {
    Base(FileStateRoot, u64, u64),
    Edited(Vec<file_edit::Piece>),
}

impl ReadPlan {
    pub fn for_file(data: &FileData, offset: u64, size: usize) -> Result<Self> {
        let len = match data {
            FileData::Base { len, .. } => *len,
            FileData::Edited { pieces, .. } => pieces.len(),
        };
        let offset = offset.min(len);
        let end = len.min(offset.saturating_add(size as u64));
        let (source, tree_visits) = match data {
            FileData::Base { root, .. } => (ReadSource::Base(*root, offset, end), 0),
            FileData::Edited { pieces, .. } => {
                let (ranges, visited) = pieces.range_with_visits(offset, end)?;
                (ReadSource::Edited(ranges), visited)
            }
        };
        Ok(Self {
            requested: end.saturating_sub(offset),
            source,
            tree_visits,
        })
    }
}

#[cfg(test)]
mod read_tests {
    use super::*;
    use crate::file_edit::{Piece, PieceTree};

    #[test]
    fn metadata_changes_share_aliases_and_reject_before_generation_overflow() {
        let mut live = LiveWorkspace::new(Node {
            canonical: None,
            paths: BTreeSet::from([String::new()]),
            mode: 0o755,
            links: 2,
            pins: 0,
            mtime_seconds: 0,
            mtime_nanoseconds: 0,
            data: Data::Directory(DirectoryData {
                base: None,
                changes: BTreeMap::new(),
            }),
        });
        let file = NodeId(2);
        let mut node = live.nodes[&ROOT].clone();
        node.data = Data::File(FileData::Edited {
            base: None,
            spool_high_water: 0,
            pieces: PieceTree::empty(),
            edits: 0,
        });
        node.paths = BTreeSet::from(["a".to_owned(), "b".to_owned()]);
        live.nodes.insert(file, node);
        live.chmod(file, 0o6750).unwrap();
        live.set_mtime(file, 123, 456).unwrap();
        assert_eq!(live.attr(file).unwrap().mode, 0o750);
        assert_eq!(live.mutation_generation, 2);
        assert_eq!(live.mutation_paths["a"], live.mutation_paths["b"]);
        let before = live.nodes[&file].clone();
        assert!(live.set_mtime(file, 999, 1_000_000_000).is_err());
        assert_eq!(live.nodes[&file], before);
        live.mutation_generation = u64::MAX;
        assert!(live.chmod(file, 0).is_err());
        assert!(live.set_mtime(file, 999, 0).is_err());
        assert_eq!(live.nodes[&file], before);
        assert_eq!(live.mutation_paths["a"], 2);
    }

    #[test]
    fn reads_at_and_beyond_eof_are_empty_for_base_and_edited_files() {
        let root = FileStateRoot(layerfs_content::ObjectId::for_bytes(b"base"));
        let edited = FileData::Edited {
            base: None,
            spool_high_water: 0,
            pieces: PieceTree::empty()
                .replace(0, 0, [Piece::Zero { len: 8 }])
                .unwrap(),
            edits: 1,
        };
        for file in [FileData::Base { root, len: 8 }, edited] {
            for offset in [8, 9, u64::MAX] {
                let plan = ReadPlan::for_file(&file, offset, 16).unwrap();
                assert_eq!(plan.requested, 0);
                if let ReadSource::Edited(pieces) = plan.source {
                    assert!(pieces.is_empty());
                }
            }
        }
    }
}

/// The live inode table and its coherent change generation. Native and daemon
/// adapters own one instance per writable Workspace lifetime.
pub struct LiveWorkspace {
    pub nodes: HashMap<NodeId, Node>,
    pub dirty: BTreeSet<NodeId>,
    pub mutation_generation: u64,
    pub mutation_paths: BTreeMap<String, u64>,
}

impl LiveWorkspace {
    pub fn new(root: Node) -> Self {
        Self {
            nodes: HashMap::from([(ROOT, root)]),
            dirty: BTreeSet::new(),
            mutation_generation: 0,
            mutation_paths: BTreeMap::new(),
        }
    }

    pub fn attr(&self, node: NodeId) -> Result<Attr> {
        Ok(self
            .nodes
            .get(&node)
            .ok_or(Error::NotFound("node"))?
            .attr(node))
    }

    pub fn next_generation(&self) -> Result<u64> {
        self.mutation_generation
            .checked_add(1)
            .ok_or(Error::Integrity("Workspace mutation generation"))
    }

    pub fn note_mutation(&mut self, paths: impl IntoIterator<Item = String>) -> Result<()> {
        self.mutation_generation = self.next_generation()?;
        for path in paths {
            self.mutation_paths.insert(path, self.mutation_generation);
        }
        Ok(())
    }

    pub fn chmod(&mut self, node: NodeId, mode: u32) -> Result<()> {
        let generation = self.next_generation()?;
        let value = self.nodes.get_mut(&node).ok_or(Error::NotFound("node"))?;
        value.mode = mode & 0o1777;
        self.dirty.insert(node);
        self.mutation_generation = generation;
        for path in &value.paths {
            self.mutation_paths.insert(path.clone(), generation);
        }
        Ok(())
    }

    pub fn set_mtime(&mut self, node: NodeId, seconds: i64, nanos: u32) -> Result<()> {
        if nanos > 999_999_999 {
            return Err(Error::InvalidInput("mtime"));
        }
        let generation = self.next_generation()?;
        let value = self.nodes.get_mut(&node).ok_or(Error::NotFound("node"))?;
        value.mtime_seconds = seconds;
        value.mtime_nanoseconds = nanos;
        self.dirty.insert(node);
        self.mutation_generation = generation;
        for path in &value.paths {
            self.mutation_paths.insert(path.clone(), generation);
        }
        Ok(())
    }
}

use crate::file_edit::PieceTree;
use crate::{Attr, Data, DirectoryData, Error, FileData, LiveWorkspace, Node, NodeId, Result};
use layerfs_content::tree::directory::DirectoryStateRoot;
use layerfs_content::CanonicalName;

// ponytail: one namespace writer per Workspace; split directory locks only if
// measured contention warrants it. Acquisition runs outside that state access.

/// A name whose binding was resolved at an exact parent revision.
pub struct ResolvedName {
    parent: NodeId,
    revision: u64,
    name: CanonicalName,
    existing: Option<NodeId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResourcePolicy, ROOT};
    use std::collections::BTreeMap;

    fn fixture() -> LiveWorkspace {
        let live = LiveWorkspace::new(
            Node {
                revision: 0,
                canonical: None,
                paths: [String::new()].into(),
                mode: 0o755,
                links: 2,
                pins: 0,
                mtime_seconds: 0,
                mtime_nanoseconds: 0,
                data: Data::Directory(DirectoryData {
                    base: None,
                    changes: BTreeMap::new(),
                }),
            },
            ResourcePolicy::default(),
        );
        live
    }

    #[test]
    fn directories_symlinks_and_acquired_aliases_use_the_same_live_nodes() {
        use layerfs_content::file::rope::FileStateRoot;
        use layerfs_content::tree::inode::InodeId;
        let mut live = fixture();
        let NameLookup::Ready(name) = live.prepare_name(ROOT, b"dir").unwrap() else {
            panic!("owned root")
        };
        let directory = live.mkdir(name, 0o1750, None).unwrap();
        assert_eq!(directory.mode, 0o1750);
        assert_eq!(live.parent_of(directory.node).unwrap(), ROOT);
        let NameLookup::Ready(name) = live.prepare_name(directory.node, b"link").unwrap() else {
            panic!("owned directory")
        };
        let link = live.symlink(name, b"target".to_vec()).unwrap();
        assert_eq!(link.kind, crate::Kind::Symlink);
        let mut acquired = Node {
            revision: 0,
            canonical: Some(InodeId([9; 32])),
            paths: ["cold".to_owned()].into(),
            mode: 0o600,
            links: 2,
            pins: 0,
            mtime_seconds: 0,
            mtime_nanoseconds: 0,
            data: Data::File(FileData::Base {
                root: FileStateRoot(layerfs_content::ObjectId::for_bytes(b"base")),
                len: 8,
            }),
        };
        let node = live.install_immutable_node(acquired.clone()).unwrap();
        let write = live.prepare_write(node, 0, 1, None).unwrap();
        live.apply_write(write).unwrap();
        let changed = live.nodes[&node].data.clone();
        acquired.paths = ["alias".to_owned()].into();
        assert_eq!(live.install_immutable_node(acquired.clone()).unwrap(), node);
        assert_eq!(live.nodes[&node].data, changed);
        assert_eq!(live.nodes[&node].paths.len(), 2);
        acquired.mtime_nanoseconds = 1_000_000_000;
        assert!(live.install_immutable_node(acquired).is_err());
        assert_eq!(live.nodes[&node].data, changed);
    }

    #[test]
    fn resolved_create_revalidates_and_rejects_before_consuming_identity() {
        let mut live = fixture();
        let NameLookup::Ready(first) = live.prepare_name(ROOT, b"first").unwrap() else {
            panic!("owned directory")
        };
        let NameLookup::Ready(stale) = live.prepare_name(ROOT, b"second").unwrap() else {
            panic!("owned directory")
        };
        let first = live.create_file(first, 0o600, None).unwrap();
        let before = (live.nodes.len(), live.next_node, live.mutation_generation);
        assert!(live.create_file(stale, 0o600, None).is_err());
        assert_eq!(
            (live.nodes.len(), live.next_node, live.mutation_generation),
            before
        );
        let NameLookup::Ready(existing) = live.prepare_name(ROOT, b"first").unwrap() else {
            panic!("owned binding")
        };
        assert_eq!(existing.existing(), Some(first.node));
        live.reserved.insert(NodeId(100));
        assert!(live
            .create_file(existing, 0o600, Some(NodeId(100)))
            .is_err());
        assert!(live.reserved.contains(&NodeId(100)));
        assert_eq!(
            (live.nodes.len(), live.next_node, live.mutation_generation),
            before
        );
        let NameLookup::Ready(second) = live.prepare_name(ROOT, b"second").unwrap() else {
            panic!("owned directory")
        };
        assert_eq!(
            live.create_file(second, 0o640, Some(NodeId(100)))
                .unwrap()
                .node,
            NodeId(100)
        );
        assert!(!live.reserved.contains(&NodeId(100)));
        let base = DirectoryStateRoot(layerfs_content::ObjectId::for_bytes(b"immutable-directory"));
        live.directory_mut(ROOT).unwrap().base = Some(base);
        let NameLookup::Acquire(input) = live.prepare_name(ROOT, b"cold").unwrap() else {
            panic!("immutable acquisition")
        };
        assert_eq!(input.directory, base);
        live.chmod(ROOT, 0o700).unwrap();
        assert!(live.resolve_name(input, None).is_err());
        let NameLookup::Acquire(input) = live.prepare_name(ROOT, b"cold").unwrap() else {
            panic!("immutable acquisition")
        };
        let cold = live.resolve_name(input, None).unwrap();
        live.next_node = u64::MAX;
        let before = (live.nodes.clone(), live.mutation_generation);
        assert!(live.create_file(cold, 0o600, None).is_err());
        assert_eq!(live.nodes, before.0);
        assert_eq!(live.mutation_generation, before.1);
    }
}

pub struct NameInput {
    parent: NodeId,
    revision: u64,
    pub directory: DirectoryStateRoot,
    pub name: CanonicalName,
}

pub enum NameLookup {
    Ready(ResolvedName),
    Acquire(NameInput),
}

impl ResolvedName {
    pub fn existing(&self) -> Option<NodeId> {
        self.existing
    }
}

impl LiveWorkspace {
    pub fn directory(&self, node: NodeId) -> Result<&DirectoryData> {
        match &self.nodes.get(&node).ok_or(Error::NotFound("node"))?.data {
            Data::Directory(directory) => Ok(directory),
            _ => Err(Error::InvalidInput("directory")),
        }
    }

    pub fn directory_mut(&mut self, node: NodeId) -> Result<&mut DirectoryData> {
        let node_value = self.nodes.get_mut(&node).ok_or(Error::NotFound("node"))?;
        let Data::Directory(directory) = &mut node_value.data else {
            return Err(Error::InvalidInput("directory"));
        };
        node_value.revision = node_value
            .revision
            .checked_add(1)
            .ok_or(Error::Integrity("inode revision"))?;
        self.dirty.insert(node);
        Ok(directory)
    }

    pub fn path_of(&self, node: NodeId) -> Result<String> {
        self.nodes
            .get(&node)
            .and_then(|node| node.paths.first())
            .cloned()
            .ok_or(Error::NotFound("node path"))
    }

    pub fn child_path(&self, parent: NodeId, name: &[u8]) -> Result<String> {
        CanonicalName::from_bytes(name)?;
        let prefix = self.path_of(parent)?;
        let name = std::str::from_utf8(name).map_err(|_| Error::Integrity("name"))?;
        Ok(if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        })
    }

    pub fn prepare_name(&self, parent: NodeId, name: &[u8]) -> Result<NameLookup> {
        let name = CanonicalName::from_bytes(name)?;
        let directory = self.directory(parent)?;
        let revision = self.nodes[&parent].revision;
        let existing = match directory.changes.get(name.as_bytes()) {
            Some(existing) => *existing,
            None => match directory.base {
                Some(directory) => {
                    return Ok(NameLookup::Acquire(NameInput {
                        parent,
                        revision,
                        directory,
                        name,
                    }))
                }
                None => None,
            },
        };
        Ok(NameLookup::Ready(ResolvedName {
            parent,
            revision,
            name,
            existing,
        }))
    }

    /// Called after immutable acquisition, never while an adapter waits on I/O.
    pub fn resolve_name(&self, input: NameInput, existing: Option<NodeId>) -> Result<ResolvedName> {
        if self.nodes.get(&input.parent).map(|node| node.revision) != Some(input.revision)
            || self.directory(input.parent)?.base != Some(input.directory)
        {
            return Err(Error::Integrity("stale name acquisition"));
        }
        if existing.is_some_and(|node| !self.nodes.contains_key(&node)) {
            return Err(Error::Integrity("acquired name inode"));
        }
        Ok(ResolvedName {
            parent: input.parent,
            revision: input.revision,
            name: input.name,
            existing,
        })
    }

    pub fn allocate_node(&mut self, node: Node) -> Result<NodeId> {
        let id = NodeId(self.next_node);
        let next = self
            .next_node
            .checked_add(1)
            .ok_or(Error::Integrity("node identity"))?;
        if self.nodes.contains_key(&id) || self.reserved.contains(&id) {
            return Err(Error::Integrity("node identity"));
        }
        self.nodes
            .try_reserve(1)
            .map_err(|_| Error::InvalidInput("workspace live allocation"))?;
        self.nodes.insert(id, node);
        self.next_node = next;
        Ok(id)
    }

    pub fn create_file(
        &mut self,
        name: ResolvedName,
        mode: u32,
        reserved: Option<NodeId>,
    ) -> Result<Attr> {
        self.create_node(
            name,
            mode & 0o777,
            Data::File(FileData::Edited {
                base: None,
                spool_high_water: 0,
                pieces: PieceTree::empty(),
                edits: 0,
            }),
            reserved,
        )
    }

    pub fn mkdir(
        &mut self,
        name: ResolvedName,
        mode: u32,
        reserved: Option<NodeId>,
    ) -> Result<Attr> {
        self.create_node(
            name,
            mode & 0o1777,
            Data::Directory(DirectoryData {
                base: None,
                changes: Default::default(),
            }),
            reserved,
        )
    }

    pub fn symlink(&mut self, name: ResolvedName, target: Vec<u8>) -> Result<Attr> {
        if target.len() > 4096 || target.contains(&0) {
            return Err(Error::InvalidInput("symlink"));
        }
        self.create_node(name, 0o777, Data::Symlink(target), None)
    }

    pub fn parent_of(&self, node: NodeId) -> Result<NodeId> {
        self.directory_parents
            .get(&node)
            .copied()
            .ok_or(Error::Integrity("Workspace parent"))
    }

    pub fn remember_directory_parent(&mut self, node: NodeId, parent: NodeId) -> Result<()> {
        if !matches!(
            self.nodes.get(&node).ok_or(Error::NotFound("node"))?.data,
            Data::Directory(_)
        ) {
            return Ok(());
        }
        match self.directory_parents.get(&node).copied() {
            Some(known) if known != parent => Err(Error::Integrity("Workspace parent")),
            Some(_) => Ok(()),
            None => {
                self.directory_parents
                    .try_reserve(1)
                    .map_err(|_| Error::InvalidInput("workspace live allocation"))?;
                self.directory_parents.insert(node, parent);
                Ok(())
            }
        }
    }

    pub fn install_immutable_node(&mut self, node: Node) -> Result<NodeId> {
        use layerfs_content::tree::inode::InodeKind;
        use layerfs_content::tree::metadata::PortableMetadataV1;
        let inode = node
            .canonical
            .ok_or(Error::Integrity("acquired inode identity"))?;
        if node.revision != 0
            || node.pins != 0
            || matches!(node.data, Data::File(FileData::Edited { .. }))
        {
            return Err(Error::Integrity("acquired mutable inode"));
        }
        let kind = match &node.data {
            Data::File(FileData::Base { .. }) => InodeKind::RegularFile,
            Data::Directory(directory)
                if directory.base.is_some() && directory.changes.is_empty() =>
            {
                InodeKind::Directory
            }
            Data::Symlink(target) if target.len() <= 4096 && !target.contains(&0) => {
                InodeKind::Symlink
            }
            _ => return Err(Error::Integrity("acquired inode data")),
        };
        PortableMetadataV1 {
            permission_mode: node.mode,
            mtime_seconds: node.mtime_seconds,
            mtime_nanoseconds: node.mtime_nanoseconds,
        }
        .validate(kind)?;
        if let Some(id) = self.canonical_nodes.get(&inode).copied() {
            let live = self
                .nodes
                .get_mut(&id)
                .ok_or(Error::Integrity("Workspace inode"))?;
            if live.attr(id).kind != node.attr(id).kind {
                return Err(Error::Integrity("acquired inode kind"));
            }
            live.paths.extend(node.paths);
            return Ok(id);
        }
        self.canonical_nodes
            .try_reserve(1)
            .map_err(|_| Error::InvalidInput("workspace live allocation"))?;
        let id = self.allocate_node(node)?;
        self.canonical_nodes.insert(inode, id);
        Ok(id)
    }

    fn create_node(
        &mut self,
        name: ResolvedName,
        mode: u32,
        data: Data,
        reserved: Option<NodeId>,
    ) -> Result<Attr> {
        if self.nodes.get(&name.parent).map(|node| node.revision) != Some(name.revision) {
            return Err(Error::Integrity("stale name acquisition"));
        }
        self.directory(name.parent)?;
        if name.existing.is_some() {
            return Err(Error::InvalidInput("name exists"));
        }
        let generation = self.next_generation()?;
        name.revision
            .checked_add(1)
            .ok_or(Error::Integrity("inode revision"))?;
        let path = self.child_path(name.parent, name.name.as_bytes())?;
        if let Some(node) = reserved {
            if !self.reserved.contains(&node) || self.nodes.contains_key(&node) {
                return Err(Error::Integrity("reserved node"));
            }
        }
        self.nodes
            .try_reserve(1)
            .map_err(|_| Error::InvalidInput("workspace live allocation"))?;
        let directory = matches!(data, Data::Directory(_));
        let file = matches!(data, Data::File(_));
        if directory {
            self.directory_parents
                .try_reserve(1)
                .map_err(|_| Error::InvalidInput("workspace live allocation"))?;
        }
        let value = Node {
            revision: 0,
            canonical: None,
            paths: [path.clone()].into(),
            mode,
            links: if directory { 2 } else { 1 },
            pins: 0,
            mtime_seconds: 0,
            mtime_nanoseconds: 0,
            data,
        };
        let node = match reserved {
            Some(node) => {
                self.reserved.remove(&node);
                self.nodes.insert(node, value);
                node
            }
            None => self.allocate_node(value)?,
        };
        self.directory_mut(name.parent)?
            .changes
            .insert(name.name.as_bytes().to_vec(), Some(node));
        if file {
            self.edited_nodes.insert(node);
        }
        if directory {
            self.directory_parents.insert(node, name.parent);
        }
        self.dirty.insert(node);
        self.mutation_generation = generation;
        self.mutation_paths.insert(path, generation);
        self.attr(node)
    }
}

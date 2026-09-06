use crate::{ResourcePolicy, WorkspaceState};
use layerfs_content::file::extent_codec::decode_file_state;
use layerfs_content::file::rope::{read_all_bounded, state, FileStateRoot, RopeCounters};
use layerfs_content::filesystem::{self as logical, LogicalCounters};
use layerfs_content::object::access::ObjectRead;
use layerfs_content::tree::directory::codec::decode_symlink;
use layerfs_content::tree::directory::{
    directory_page_after, DirectoryStateRoot, NamespaceCounters,
};
use layerfs_content::tree::inode::codec::decode_inode_record;
use layerfs_content::tree::inode::{
    inode_table_lookup, inode_table_lookup_many, InodeId, InodeKind, InodeTableCounters,
    InodeTableRoot,
};
use layerfs_content::tree::metadata::{metadata_lookup, MetadataKey, PortableMetadataV1};
use layerfs_content::CanonicalPath;
use layerfs_layerstack_store::{
    BranchId, CommitId, CoreReader, LayerId, LayerStackStore, Result, SnapshotReader,
    StoreError as StorageError,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

pub use layerfs_workspace_core::{Attr, Kind, NodeId, ROOT};
pub(crate) use layerfs_workspace_core::{Data, DirectoryData, FileData, Node};

pub(crate) struct WorkspaceSnapshot {
    pub(crate) store: LayerStackStore,
    pub(crate) workspace_id: [u8; 16],
    pub(crate) branch_id: BranchId,
    pub(crate) expected_head: Option<CommitId>,
    pub(crate) expected_base: LayerId,
    pub(crate) root: layerfs_content::ObjectId,
    pub(crate) reader: SnapshotReader,
}

pub struct Workspace {
    pub(crate) live: layerfs_workspace_core::LiveWorkspace,
    pub(crate) store: LayerStackStore,
    pub(crate) workspace_id: [u8; 16],
    pub(crate) reader: SnapshotReader,
    pub(crate) branch_id: BranchId,
    pub(crate) expected_head: Option<CommitId>,
    pub(crate) expected_base: LayerId,
    pub(crate) base_root: layerfs_content::ObjectId,
    pub(crate) base_inodes: InodeTableRoot,
    pub(crate) directory_lookup_cache: layerfs_content::tree::directory::DirectoryLookupCache,
    pub(crate) spool: PathBuf,
    pub(crate) backing: crate::file_io::HostSpool,
    pub(crate) capture: crate::capture::CaptureState,
    pub(crate) state: WorkspaceState,
    pub(crate) presentation_failed: bool,
    pub(crate) resolution: Option<crate::reconcile::ResolutionState>,
    pub(crate) pending_checkpoint: Option<crate::changes::Checkpoint>,
    pub(crate) pending_stage: Option<layerfs_content::ObjectId>,
    pub(crate) pending_publication:
        Option<(layerfs_layerstack_store::CommitOutcome, LayerId, bool)>,
}

impl Workspace {
    #[cfg(test)]
    pub fn open(
        store: LayerStackStore,
        branch_id: BranchId,
        spool: impl AsRef<Path>,
    ) -> Result<Self> {
        Self::open_with_policy(store, branch_id, spool, ResourcePolicy::default())
    }

    #[cfg(test)]
    pub fn open_with_policy(
        store: LayerStackStore,
        branch_id: BranchId,
        spool: impl AsRef<Path>,
        policy: ResourcePolicy,
    ) -> Result<Self> {
        let pinned = store.pin_branch(branch_id)?;
        Self::from_snapshot(
            WorkspaceSnapshot {
                store,
                workspace_id: crate::WorkspaceId::new().bytes(),
                branch_id,
                expected_head: pinned.branch.head_commit_id,
                expected_base: pinned.branch.base_layer_id,
                root: pinned.root,
                reader: pinned.reader,
            },
            spool.as_ref(),
            policy,
        )
    }

    pub(crate) fn clean_copy(&self, spool: impl AsRef<Path>) -> Result<Self> {
        if self.pending_stage.is_some() || self.pending_publication.is_some() {
            return Err(StorageError::InvalidInput("workspace completion pending"));
        }
        Self::from_snapshot(
            WorkspaceSnapshot {
                store: self.store.clone(),
                workspace_id: self.workspace_id,
                branch_id: self.branch_id,
                expected_head: self.expected_head,
                expected_base: self.expected_base,
                root: self.base_root,
                reader: self.reader.clone(),
            },
            spool.as_ref(),
            self.live.policy,
        )
    }

    pub(crate) fn from_snapshot(
        snapshot: WorkspaceSnapshot,
        spool: &Path,
        policy: ResourcePolicy,
    ) -> Result<Self> {
        let WorkspaceSnapshot {
            store,
            workspace_id,
            branch_id,
            expected_head,
            expected_base,
            root: base_root,
            reader,
        } = snapshot;
        let core = CoreReader(&reader);
        let namespace = logical::namespace(&core, base_root)?;
        let resolved = logical::resolve(
            &core,
            base_root,
            &CanonicalPath::root(),
            &mut LogicalCounters::default(),
        )?;
        let portable =
            portable_metadata(&core, resolved.record.metadata_root, resolved.record.kind)?;
        let spool = spool.to_owned();
        std::fs::create_dir_all(&spool)?;
        let root = Node {
            revision: 0,
            canonical: Some(resolved.inode),
            paths: BTreeSet::from([String::new()]),
            mode: portable.permission_mode,
            links: 2,
            pins: 0,
            mtime_seconds: portable.mtime_seconds,
            mtime_nanoseconds: portable.mtime_nanoseconds,
            data: Data::Directory(DirectoryData {
                base: Some(DirectoryStateRoot(resolved.record.content_root)),
                changes: BTreeMap::new(),
            }),
        };
        Ok(Self {
            live: layerfs_workspace_core::LiveWorkspace::new(root, policy, base_root),
            store,
            workspace_id,
            reader,
            branch_id,
            expected_head,
            expected_base,
            base_root,
            base_inodes: InodeTableRoot(namespace.inode_table_root),
            directory_lookup_cache: Default::default(),
            spool,
            backing: Default::default(),
            capture: crate::capture::CaptureState::default(),
            state: WorkspaceState::Active,
            presentation_failed: false,
            resolution: None,
            pending_checkpoint: None,
            pending_stage: None,
            pending_publication: None,
        })
    }

    pub fn attr(&self, node: NodeId) -> Result<Attr> {
        self.live.attr(node).map_err(crate::live_error)
    }

    pub fn lookup(&mut self, parent: NodeId, name: &[u8]) -> Result<Attr> {
        let node = self.lookup_node(parent, name)?;
        self.attr(node)
    }

    pub fn readlink(&self, node: NodeId) -> Result<Vec<u8>> {
        match &self
            .live
            .nodes
            .get(&node)
            .ok_or(StorageError::NotFound("node"))?
            .data
        {
            Data::Symlink(target) => Ok(target.clone()),
            _ => Err(StorageError::InvalidInput("readlink")),
        }
    }

    pub fn readdir(&mut self, node: NodeId) -> Result<Vec<(NodeId, Kind, Vec<u8>)>> {
        Ok(self
            .readdirplus(node)?
            .into_iter()
            .map(|(attr, name)| (attr.node, attr.kind, name))
            .collect())
    }

    pub fn readdirplus(&mut self, node: NodeId) -> Result<Vec<(Attr, Vec<u8>)>> {
        let parent = self.parent_of(node)?;
        let mut output = VecDeque::from([
            (self.attr(node)?, b".".to_vec()),
            (self.attr(parent)?, b"..".to_vec()),
        ]);
        for (name, child) in self.directory_entries(node)? {
            output.push_back((self.attr(child)?, name));
        }
        Ok(output.into())
    }

    pub(crate) fn reserve_nodes(&mut self, count: u32) -> Result<NodeId> {
        self.ensure_active()?;
        if count == 0 || count > 65_536 {
            return Err(StorageError::InvalidInput("node reservation"));
        }
        let start = self.live.next_node;
        self.live.next_node = self
            .live
            .next_node
            .checked_add(u64::from(count))
            .ok_or(StorageError::Integrity("node reservation"))?;
        self.live
            .reserved
            .extend((start..self.live.next_node).map(NodeId));
        Ok(NodeId(start))
    }

    pub(crate) fn create_file_reserved(
        &mut self,
        parent: NodeId,
        name: &[u8],
        mode: u32,
        node: NodeId,
    ) -> Result<Attr> {
        self.ensure_active()?;
        let name = self.acquire_name(parent, name)?;
        self.live
            .create_file(name, mode, Some(node))
            .map_err(crate::live_error)
    }

    fn acquire_name(
        &mut self,
        parent: NodeId,
        name: &[u8],
    ) -> Result<layerfs_workspace_core::namespace::ResolvedName> {
        use layerfs_workspace_core::namespace::NameLookup;
        let input = match self
            .live
            .prepare_name(parent, name)
            .map_err(crate::live_error)?
        {
            NameLookup::Ready(name) => return Ok(name),
            NameLookup::Acquire(input) => input,
        };
        let inode = self.directory_lookup_cache.lookup(
            &CoreReader(&self.reader),
            input.directory,
            &input.name,
            &mut NamespaceCounters::default(),
        )?;
        if let Some(node) = inode.and_then(|inode| self.live.canonical_nodes.get(&inode).copied()) {
            let path = self.child_path(parent, name)?;
            let resolved = self
                .live
                .resolve_name(input, Some(node))
                .map_err(crate::live_error)?;
            self.remember_directory_parent(node, parent)?;
            self.live.nodes.get_mut(&node).unwrap().paths.insert(path);
            return Ok(resolved);
        }
        let acquired = inode
            .map(|inode| acquire_inode(&self.reader, self.base_inodes, inode))
            .transpose()?;
        self.live
            .complete_name(input, acquired)
            .map_err(crate::live_error)
    }

    pub(crate) fn note_mutation(&mut self, paths: impl IntoIterator<Item = String>) -> Result<()> {
        if self.pending_stage.is_some() {
            return Err(StorageError::InvalidInput("workspace stage pending"));
        }
        self.live.note_mutation(paths).map_err(crate::live_error)
    }

    pub(crate) fn lookup_node(&mut self, parent: NodeId, name: &[u8]) -> Result<NodeId> {
        self.acquire_name(parent, name)?
            .existing()
            .ok_or(StorageError::NotFound("name"))
    }

    fn materialize_record(
        &mut self,
        inode: InodeId,
        path: String,
        record: layerfs_content::tree::inode::InodeRecordV1,
        file_len: Option<u64>,
    ) -> Result<NodeId> {
        if let Some(node) = self.live.canonical_nodes.get(&inode).copied() {
            self.live.nodes.get_mut(&node).unwrap().paths.insert(path);
            return Ok(node);
        }
        let acquired = acquire_inode_record(&self.reader, inode, record, file_len)?;
        self.live
            .install_immutable_node(acquired, path)
            .map_err(crate::live_error)
    }

    pub(crate) fn directory_entries(&mut self, node: NodeId) -> Result<BTreeMap<Vec<u8>, NodeId>> {
        let parent = node;
        let (base, changes) = {
            let directory = self.directory(node)?;
            (directory.base, directory.changes.clone())
        };
        let prefix = self.path_of(node)?;
        let mut base_entries = Vec::new();
        if let Some(base) = base {
            let mut after = None;
            loop {
                let page = directory_page_after(
                    &CoreReader(&self.reader),
                    base,
                    after.as_ref(),
                    128,
                    256 * 1024,
                    &mut NamespaceCounters::default(),
                )?;
                base_entries.extend(page.entries);
                let Some(next) = page.continuation else { break };
                after = Some(next);
            }
        }
        let mut entries = BTreeMap::new();
        let mut pending = Vec::new();
        for (name, inode) in base_entries {
            if changes.contains_key(name.as_bytes()) {
                continue;
            }
            let path = join(&prefix, name.as_bytes())?;
            if let Some(child) = self.live.canonical_nodes.get(&inode).copied() {
                self.live.nodes.get_mut(&child).unwrap().paths.insert(path);
                self.remember_directory_parent(child, parent)?;
                entries.insert(name.as_bytes().to_vec(), child);
            } else {
                pending.push((name, inode, path));
            }
        }
        for pending in pending.chunks(128) {
            let reader = self.reader.clone();
            let core = CoreReader(&reader);
            let inodes = pending
                .iter()
                .map(|(_, inode, _)| *inode)
                .collect::<Vec<_>>();
            let record_ids = inode_table_lookup_many(
                &core,
                self.base_inodes,
                &inodes,
                &mut InodeTableCounters::default(),
            )?
            .into_iter()
            .map(|record| record.ok_or(StorageError::Integrity("Workspace inode")))
            .collect::<Result<Vec<_>>>()?;
            let mut records = BTreeMap::new();
            core.get_authenticated_batch(&record_ids, |id, payload| {
                records.insert(
                    id,
                    decode_inode_record(&layerfs_content::encode_bytes_object(payload)?)?,
                );
                Ok(())
            })?;
            let file_states = records
                .values()
                .filter(|record| record.kind == InodeKind::RegularFile)
                .map(|record| record.content_root)
                .collect::<Vec<_>>();
            let mut file_lengths = BTreeMap::new();
            core.get_authenticated_batch(&file_states, |id, payload| {
                file_lengths.insert(
                    id,
                    decode_file_state(&layerfs_content::encode_bytes_object(payload)?)?.logical_len,
                );
                Ok(())
            })?;
            for ((name, inode, path), record_id) in pending.iter().zip(record_ids) {
                let record = *records
                    .get(&record_id)
                    .ok_or(StorageError::Integrity("Workspace inode record"))?;
                let child = self.materialize_record(
                    *inode,
                    path.clone(),
                    record,
                    file_lengths.get(&record.content_root).copied(),
                )?;
                self.remember_directory_parent(child, parent)?;
                entries.insert(name.as_bytes().to_vec(), child);
            }
        }
        for (name, desired) in changes {
            if let Some(child) = desired {
                entries.insert(name, child);
            }
        }
        Ok(entries)
    }

    pub(crate) fn directory_is_empty(&self, node: NodeId) -> Result<bool> {
        let directory = self.directory(node)?;
        if directory.changes.values().any(Option::is_some) {
            return Ok(false);
        }
        let Some(base) = directory.base else {
            return Ok(true);
        };
        let mut after = None;
        loop {
            let page = directory_page_after(
                &CoreReader(&self.reader),
                base,
                after.as_ref(),
                128,
                256 * 1024,
                &mut NamespaceCounters::default(),
            )?;
            if page
                .entries
                .iter()
                .any(|(name, _)| !directory.changes.contains_key(name.as_bytes()))
            {
                return Ok(false);
            }
            let Some(next) = page.continuation else {
                return Ok(true);
            };
            after = Some(next);
        }
    }

    fn directory(&self, node: NodeId) -> Result<&DirectoryData> {
        self.live.directory(node).map_err(crate::live_error)
    }

    pub(crate) fn directory_mut(&mut self, node: NodeId) -> Result<&mut DirectoryData> {
        self.live.directory_mut(node).map_err(crate::live_error)
    }

    pub(crate) fn path_of(&self, node: NodeId) -> Result<String> {
        self.live.path_of(node).map_err(crate::live_error)
    }

    pub(crate) fn child_path(&self, parent: NodeId, name: &[u8]) -> Result<String> {
        self.live
            .child_path(parent, name)
            .map_err(crate::live_error)
    }

    fn parent_of(&self, node: NodeId) -> Result<NodeId> {
        self.live.parent_of(node).map_err(crate::live_error)
    }

    fn remember_directory_parent(&mut self, node: NodeId, parent: NodeId) -> Result<()> {
        self.live
            .remember_directory_parent(node, parent)
            .map_err(crate::live_error)
    }
}

pub(crate) fn acquire_inode(
    reader: &SnapshotReader,
    inodes: InodeTableRoot,
    inode: InodeId,
) -> Result<layerfs_workspace_core::namespace::AcquiredInode> {
    let core = CoreReader(reader);
    let record_id = inode_table_lookup(&core, inodes, inode, &mut InodeTableCounters::default())?
        .ok_or(StorageError::Integrity("Workspace inode"))?;
    let record = core.with_authenticated_canonical(record_id, decode_inode_record)?;
    acquire_inode_record(reader, inode, record, None)
}

fn acquire_inode_record(
    reader: &SnapshotReader,
    inode: InodeId,
    record: layerfs_content::tree::inode::InodeRecordV1,
    file_len: Option<u64>,
) -> Result<layerfs_workspace_core::namespace::AcquiredInode> {
    record.validate(false)?;
    let reader = CoreReader(reader);
    let portable = portable_metadata(&reader, record.metadata_root, record.kind)?;
    let data = match record.kind {
        InodeKind::RegularFile => {
            let len = match file_len {
                Some(len) => len,
                None => {
                    state(
                        &reader,
                        FileStateRoot(record.content_root),
                        &mut RopeCounters::default(),
                    )?
                    .logical_len
                }
            };
            Data::File(FileData::Base {
                root: FileStateRoot(record.content_root),
                len,
            })
        }
        InodeKind::Directory => Data::Directory(DirectoryData {
            base: Some(DirectoryStateRoot(record.content_root)),
            changes: BTreeMap::new(),
        }),
        InodeKind::Symlink => Data::Symlink(
            reader
                .with_authenticated_canonical(record.content_root, decode_symlink)?
                .target,
        ),
    };
    Ok(layerfs_workspace_core::namespace::AcquiredInode {
        inode,
        mode: portable.permission_mode,
        links: if record.kind == InodeKind::Directory {
            2
        } else {
            u32::try_from(record.namespace_ref_count)
                .map_err(|_| StorageError::Integrity("inode link count"))?
        },
        mtime_seconds: portable.mtime_seconds,
        mtime_nanoseconds: portable.mtime_nanoseconds,
        data,
    })
}

pub(crate) fn portable_metadata<S: ObjectRead>(
    store: &S,
    root: layerfs_content::ObjectId,
    kind: InodeKind,
) -> Result<PortableMetadataV1> {
    let value = |name: &[u8], maximum| -> Result<Vec<u8>> {
        let entry = metadata_lookup(
            store,
            root,
            &MetadataKey::new("portable".to_owned(), name.to_vec())?,
        )?
        .ok_or(StorageError::Integrity("portable metadata"))?;
        let mut bytes = Vec::new();
        read_all_bounded(
            store,
            FileStateRoot(entry.value_file_root),
            maximum,
            &mut bytes,
        )?;
        Ok(bytes)
    };
    let mode = value(b"mode", 4)?;
    let mtime = value(b"mtime", 12)?;
    let metadata = PortableMetadataV1 {
        permission_mode: u32::from_be_bytes(
            mode.try_into()
                .map_err(|_| StorageError::Integrity("mode"))?,
        ),
        mtime_seconds: i64::from_be_bytes(
            mtime[..8]
                .try_into()
                .map_err(|_| StorageError::Integrity("mtime"))?,
        ),
        mtime_nanoseconds: u32::from_be_bytes(
            mtime[8..]
                .try_into()
                .map_err(|_| StorageError::Integrity("mtime"))?,
        ),
    };
    metadata.validate(kind)?;
    Ok(metadata)
}

fn join(parent: &str, name: &[u8]) -> Result<String> {
    let name = std::str::from_utf8(name).map_err(|_| StorageError::Integrity("name"))?;
    Ok(if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}/{name}")
    })
}

impl Workspace {
    pub fn create_file(&mut self, parent: NodeId, name: &[u8], mode: u32) -> Result<Attr> {
        self.ensure_active()?;
        let name = self.acquire_name(parent, name)?;
        self.live
            .create_file(name, mode, None)
            .map_err(crate::live_error)
    }

    pub fn mkdir(&mut self, parent: NodeId, name: &[u8], mode: u32) -> Result<Attr> {
        self.ensure_active()?;
        let name = self.acquire_name(parent, name)?;
        self.live.mkdir(name, mode, None).map_err(crate::live_error)
    }

    pub(crate) fn mkdir_reserved(
        &mut self,
        parent: NodeId,
        name: &[u8],
        mode: u32,
        node: NodeId,
    ) -> Result<Attr> {
        self.ensure_active()?;
        let name = self.acquire_name(parent, name)?;
        self.live
            .mkdir(name, mode, Some(node))
            .map_err(crate::live_error)
    }

    pub fn symlink(&mut self, parent: NodeId, name: &[u8], target: Vec<u8>) -> Result<Attr> {
        self.ensure_active()?;
        let name = self.acquire_name(parent, name)?;
        self.live.symlink(name, target).map_err(crate::live_error)
    }

    pub fn link(&mut self, node: NodeId, parent: NodeId, name: &[u8]) -> Result<Attr> {
        self.ensure_active()?;
        if matches!(
            self.live
                .nodes
                .get(&node)
                .ok_or(StorageError::NotFound("node"))?
                .data,
            Data::Directory(_)
        ) {
            return Err(StorageError::InvalidInput("directory link"));
        }
        let target = self.child_path(parent, name)?;
        self.insert_name(parent, name, node)?;
        let value = self.live.nodes.get_mut(&node).unwrap();
        value.links += 1;
        value.paths.insert(target.clone());
        let paths = value.paths.iter().cloned().collect::<Vec<_>>();
        self.note_mutation(paths)?;
        self.attr(node)
    }

    pub fn unlink(&mut self, parent: NodeId, name: &[u8], directory: bool) -> Result<()> {
        self.ensure_active()?;
        let node = self.lookup_node(parent, name)?;
        let (is_directory, mut paths) = {
            let value = self.live.nodes.get(&node).unwrap();
            (
                matches!(value.data, Data::Directory(_)),
                value.paths.iter().cloned().collect::<Vec<_>>(),
            )
        };
        if directory != is_directory {
            return Err(StorageError::InvalidInput("unlink kind"));
        }
        if directory && !self.directory_entries(node)?.is_empty() {
            return Err(StorageError::InvalidInput("directory not empty"));
        }
        let path = self.child_path(parent, name)?;
        paths.push(path.clone());
        self.directory_mut(parent)?
            .changes
            .insert(name.to_vec(), None);
        let value = self.live.nodes.get_mut(&node).unwrap();
        value.links = value.links.saturating_sub(1);
        value.paths.remove(&path);
        self.live.dirty.insert(node);
        self.reclaim(node);
        self.note_mutation(paths)?;
        Ok(())
    }

    pub fn rename(
        &mut self,
        parent: NodeId,
        name: &[u8],
        target_parent: NodeId,
        target: &[u8],
        no_replace: bool,
    ) -> Result<()> {
        self.ensure_active()?;
        let node = self.lookup_node(parent, name)?;
        let source = self.child_path(parent, name)?;
        let destination = self.child_path(target_parent, target)?;
        if source == destination {
            return Ok(());
        }
        let source_directory = matches!(self.live.nodes[&node].data, Data::Directory(_));
        let existing = match self.lookup_node(target_parent, target) {
            Ok(existing) if existing == node => return Ok(()),
            Ok(existing) => Some(existing),
            Err(StorageError::NotFound(_)) => None,
            Err(error) => return Err(error),
        };
        if no_replace && existing.is_some() {
            return Err(StorageError::InvalidInput("rename target"));
        }
        if let Some(existing) = existing {
            let target_directory = matches!(self.live.nodes[&existing].data, Data::Directory(_));
            if source_directory != target_directory {
                return Err(StorageError::InvalidInput("rename type"));
            }
        }
        if source_directory {
            let target_parent_path = self.path_of(target_parent)?;
            if target_parent_path == source
                || (target_parent_path.starts_with(&source)
                    && target_parent_path.as_bytes().get(source.len()) == Some(&b'/'))
            {
                return Err(StorageError::InvalidInput("rename descendant"));
            }
        }
        if let Some(existing) = existing {
            if source_directory && !self.directory_is_empty(existing)? {
                return Err(StorageError::InvalidInput("directory not empty"));
            }
        }
        if existing.is_some() {
            self.unlink(target_parent, target, source_directory)?;
        }
        self.directory_mut(parent)?
            .changes
            .insert(name.to_vec(), None);
        self.directory_mut(target_parent)?
            .changes
            .insert(target.to_vec(), Some(node));
        self.replace_path_prefix(&source, &destination);
        if source_directory {
            self.live.directory_parents.insert(node, target_parent);
        }
        self.note_mutation([source, destination])?;
        Ok(())
    }

    pub fn pin(&mut self, node: NodeId, truncate: bool) -> Result<()> {
        self.live.check_pin(node).map_err(crate::live_error)?;
        if truncate {
            self.truncate(node, 0)?;
        }
        self.live.pin(node).map_err(crate::live_error)
    }

    pub fn unpin(&mut self, node: NodeId) -> Result<()> {
        self.finish_capture(Some(node));
        if self.live.unpin(node).map_err(crate::live_error)? {
            self.retire_spool_segments();
        }
        Ok(())
    }

    pub fn chmod(&mut self, node: NodeId, mode: u32) -> Result<()> {
        self.ensure_active()?;
        self.live.chmod(node, mode).map_err(crate::live_error)
    }

    pub fn set_mtime(&mut self, node: NodeId, seconds: i64, nanos: u32) -> Result<()> {
        self.ensure_active()?;
        self.live
            .set_mtime(node, seconds, nanos)
            .map_err(crate::live_error)
    }

    fn insert_name(&mut self, parent: NodeId, name: &[u8], node: NodeId) -> Result<()> {
        match self.lookup_node(parent, name) {
            Ok(_) => return Err(StorageError::InvalidInput("name exists")),
            Err(StorageError::NotFound(_)) => {}
            Err(error) => return Err(error),
        }
        self.directory_mut(parent)?
            .changes
            .insert(name.to_vec(), Some(node));
        self.live.dirty.insert(node);
        self.remember_directory_parent(node, parent)
    }

    fn replace_path_prefix(&mut self, source: &str, target: &str) {
        for node in self.live.nodes.values_mut() {
            node.paths = node
                .paths
                .iter()
                .map(|path| {
                    if path == source {
                        target.to_owned()
                    } else if path.starts_with(source)
                        && path.as_bytes().get(source.len()) == Some(&b'/')
                    {
                        format!("{target}{}", &path[source.len()..])
                    } else {
                        path.clone()
                    }
                })
                .collect();
        }
    }

    fn reclaim(&mut self, node: NodeId) {
        if self.live.reclaim(node) {
            self.retire_spool_segments();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ROOT;
    use layerfs_layerstack_store::{
        EntityName, LayerStackInitialization, LayerStackStore, LocalForkSource,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct Snapshot {
        nodes: std::collections::HashMap<NodeId, Node>,
        canonical_nodes: std::collections::HashMap<layerfs_content::tree::inode::InodeId, NodeId>,
        directory_parents: std::collections::HashMap<NodeId, NodeId>,
        dirty: BTreeSet<NodeId>,
        next_node: u64,
        spool_bytes: u64,
        inline_bytes: u64,
        piece_allocation_bytes: u64,
    }

    fn snapshot(workspace: &Workspace) -> Snapshot {
        Snapshot {
            nodes: workspace.live.nodes.clone(),
            canonical_nodes: workspace.live.canonical_nodes.clone(),
            directory_parents: workspace.live.directory_parents.clone(),
            dirty: workspace.live.dirty.clone(),
            next_node: workspace.live.next_node,
            spool_bytes: workspace.live.spool_bytes,
            inline_bytes: workspace.live.inline_bytes,
            piece_allocation_bytes: workspace.live.piece_allocation_bytes,
        }
    }

    fn fixture(label: &str) -> (std::path::PathBuf, Workspace) {
        let root = std::env::temp_dir().join(format!(
            "layerfs-rename-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let store = LayerStackStore::create(root.join("store.sqlite")).unwrap();
        let genesis = store
            .initialize_layerstack(
                EntityName::new("project").unwrap(),
                LayerStackInitialization::Empty,
            )
            .unwrap()
            .genesis_layer_id;
        let id = store
            .fork_branch(
                EntityName::new("main").unwrap(),
                LocalForkSource::Layer { layer_id: genesis },
            )
            .unwrap();
        let workspace = Workspace::open(store, id, root.join("spool")).unwrap();
        (root, workspace)
    }

    #[test]
    fn reserved_directory_consumes_the_exact_node_once() {
        let (root, mut workspace) = fixture("reserved-directory");
        let node = workspace.reserve_nodes(1).unwrap();
        let directory = workspace
            .mkdir_reserved(ROOT, b"directory", 0o700, node)
            .unwrap();
        assert_eq!(directory.node, node);
        assert_eq!(workspace.lookup(ROOT, b"directory").unwrap().node, node);
        assert_eq!(workspace.parent_of(node).unwrap(), ROOT);
        let child = workspace.mkdir(ROOT, b"child", 0o700).unwrap().node;
        workspace
            .rename(ROOT, b"child", node, b"moved", true)
            .unwrap();
        assert_eq!(workspace.parent_of(child).unwrap(), node);
        assert_eq!(
            workspace
                .readdirplus(child)
                .unwrap()
                .into_iter()
                .find(|(_, name)| name == b"..")
                .unwrap()
                .0
                .node,
            node,
        );
        assert!(workspace
            .mkdir_reserved(ROOT, b"duplicate", 0o700, node)
            .is_err());
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rename_validates_every_noop_and_rejection_before_mutation() {
        let (root, mut workspace) = fixture("same-path");
        workspace.create_file(ROOT, b"a", 0o600).unwrap();
        let before = snapshot(&workspace);
        workspace.rename(ROOT, b"a", ROOT, b"a", false).unwrap();
        assert_eq!(snapshot(&workspace), before);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();

        let (root, mut workspace) = fixture("same-inode");
        let file = workspace.create_file(ROOT, b"a", 0o600).unwrap();
        workspace.link(file.node, ROOT, b"b").unwrap();
        let before = snapshot(&workspace);
        workspace.rename(ROOT, b"a", ROOT, b"b", false).unwrap();
        assert_eq!(snapshot(&workspace), before);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();

        let (root, mut workspace) = fixture("file-over-directory");
        workspace.create_file(ROOT, b"file", 0o600).unwrap();
        workspace.mkdir(ROOT, b"directory", 0o700).unwrap();
        assert_rejected(&mut workspace, |workspace| {
            workspace.rename(ROOT, b"file", ROOT, b"directory", false)
        });
        assert_rejected(&mut workspace, |workspace| {
            workspace.rename(ROOT, b"directory", ROOT, b"file", false)
        });
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();

        let (root, mut workspace) = fixture("nonempty-directory");
        workspace.mkdir(ROOT, b"source", 0o700).unwrap();
        let target = workspace.mkdir(ROOT, b"target", 0o700).unwrap();
        workspace.create_file(target.node, b"child", 0o600).unwrap();
        assert_rejected(&mut workspace, |workspace| {
            workspace.rename(ROOT, b"source", ROOT, b"target", false)
        });
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();

        let (root, mut workspace) = fixture("descendant");
        let source = workspace.mkdir(ROOT, b"source", 0o700).unwrap();
        let child = workspace.mkdir(source.node, b"child", 0o700).unwrap();
        workspace.create_file(child.node, b"target", 0o600).unwrap();
        assert_rejected(&mut workspace, |workspace| {
            workspace.rename(ROOT, b"source", child.node, b"target", false)
        });
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spool_write_metrics_are_aggregate_and_reset() {
        let (root, mut workspace) = fixture("spool-write-metrics");
        let file = workspace.create_file(ROOT, b"file", 0o600).unwrap();
        workspace.write(file.node, 0, b"data").unwrap();
        workspace.fsync(Some(file.node)).unwrap();
        assert_eq!(workspace.backing.segments.len(), 1);
        let metrics = workspace.take_spool_write_metrics();
        assert_eq!(metrics.write_bytes, 4);
        assert_eq!(metrics.write_open_count, 1);
        assert_eq!(metrics.fence_count, 1);
        assert_eq!(
            workspace.take_spool_write_metrics(),
            crate::file_io::SpoolWriteMetrics::default()
        );
        workspace.unlink(ROOT, b"file", false).unwrap();
        assert!(workspace.backing.segments.is_empty());
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_spool_io_does_not_advance_the_overlay() {
        let (root, mut workspace) = fixture("failed-write");
        let file = workspace.create_file(ROOT, b"file", 0o600).unwrap();
        workspace.write(file.node, 0, b"base").unwrap();
        crate::file_io::spool_segment(workspace.backing.segments.values().next().unwrap())
            .unwrap()
            .file
            .set_len(3)
            .unwrap();
        let before = snapshot(&workspace);
        assert!(workspace.write(file.node, 4, b"lost").is_err());
        assert_eq!(snapshot(&workspace), before);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();

        let (root, mut workspace) = fixture("failed-truncate");
        let file = workspace.create_file(ROOT, b"file", 0o600).unwrap();
        workspace.write(file.node, 0, b"base").unwrap();
        crate::file_io::spool_segment(workspace.backing.segments.values().next().unwrap())
            .unwrap()
            .file
            .set_len(3)
            .unwrap();
        let before = snapshot(&workspace);
        assert!(workspace.truncate(file.node, 2).is_err());
        assert_eq!(snapshot(&workspace), before);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolution_fingerprint_ignores_unrelated_paths_and_tracks_affected_state() {
        let (root, mut workspace) = fixture("resolution-fingerprint");
        let affected = workspace.create_file(ROOT, b"affected", 0o600).unwrap();
        workspace.write(affected.node, 0, b"before").unwrap();
        let path = layerfs_content::CanonicalPath::new("affected").unwrap();
        let before = workspace
            .resolution_fingerprint(std::slice::from_ref(&path))
            .unwrap();

        let unrelated = workspace.create_file(ROOT, b"unrelated", 0o600).unwrap();
        workspace.write(unrelated.node, 0, b"change").unwrap();
        assert_eq!(
            workspace
                .resolution_fingerprint(std::slice::from_ref(&path))
                .unwrap(),
            before
        );

        workspace.write(affected.node, 0, b"after!").unwrap();
        assert_ne!(workspace.resolution_fingerprint(&[path]).unwrap(), before);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn assert_rejected(
        workspace: &mut Workspace,
        operation: impl FnOnce(&mut Workspace) -> Result<()>,
    ) {
        let before = snapshot(workspace);
        assert!(operation(workspace).is_err());
        assert_eq!(snapshot(workspace), before);
    }
}

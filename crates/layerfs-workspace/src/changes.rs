use crate::cow_tree::{portable_metadata, Attr, Data, FileData, Kind, NodeId, Workspace, ROOT};
use layerfs_content::file::rope::{
    self, FileMutationBatch, FileStateRoot, ObjectStore, RopeCounters,
};
use layerfs_content::filesystem::{self, InodeMutation, LogicalCounters, PortableMetadataCache};
use layerfs_content::object::access::ObjectRead;
use layerfs_content::object::{ContentDigestWriter, ObjectId};
use layerfs_content::tree::batch::{
    directory_apply_sorted_with_budget, inode_table_apply_sorted_with_budget,
    SORTED_TREE_UPDATE_SCRATCH_BYTES,
};
use layerfs_content::tree::directory::codec::encode_namespace_root;
use layerfs_content::tree::directory::{
    directory_lookup, directory_page_after, empty_directory, DirectoryStateRoot, NamespaceCounters,
};
use layerfs_content::tree::inode::codec::{decode_inode_record, encode_inode_record};
use layerfs_content::tree::inode::{
    inode_table_lookup, inode_table_lookup_many, InodeTableCounters,
};
use layerfs_content::tree::inode::{InodeId, InodeKind, InodeRecordV1, InodeTableRoot};
use layerfs_content::tree::NamespaceRootV1;
use layerfs_content::{CanonicalName, CanonicalPath};
use layerfs_layerstack_store::{
    BuiltRoot, CoreReader, ObjectBuffer, Result, StoreError as StorageError, WorkspaceCommitPhase,
};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::FileExt;
use std::time::Instant;

#[derive(Clone, Copy)]
struct BaseEntry {
    record: InodeRecordV1,
}

#[derive(Clone, Copy)]
struct FinalEntry {
    node: NodeId,
    attr: Attr,
}

fn anonymous_journal(directory: &std::path::Path) -> Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    let path = directory.join(format!("candidate-{}", crate::WorkspaceId::new()));
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    std::fs::remove_file(path)?;
    Ok(file)
}

// Mutable identities stay with Workspace; Store receives only the canonical candidate.
pub(crate) struct PreparedCommit {
    pub(crate) built: BuiltRoot,
    pub(crate) checkpoint: Checkpoint,
}

pub(crate) struct Checkpoint {
    pub(crate) root: ObjectId,
    pub(crate) generation: u64,
    file: File,
    count: u64,
}

struct CheckpointJournal {
    writer: std::io::BufWriter<File>,
    count: u64,
    byte_limit: u64,
}

impl CheckpointJournal {
    fn new(workspace: &Workspace) -> Result<Self> {
        let file = anonymous_journal(&workspace.spool)?;
        Ok(Self {
            writer: std::io::BufWriter::with_capacity(256, file),
            count: 0,
            // Metadata facts are candidate scratch, not payload spool bytes.
            byte_limit: (workspace.nodes.len() as u64).saturating_mul(104),
        })
    }

    fn push(&mut self, node: NodeId, inode: InodeId, content: ObjectId, attr: Attr) -> Result<()> {
        if self.count.saturating_add(1).saturating_mul(104) > self.byte_limit {
            return Err(StorageError::InvalidInput(
                "workspace checkpoint journal limit",
            ));
        }
        let mut bytes = [0; 104];
        bytes[..8].copy_from_slice(&node.0.to_le_bytes());
        bytes[8..40].copy_from_slice(inode.as_bytes());
        bytes[40..72].copy_from_slice(content.as_bytes());
        bytes[72..80].copy_from_slice(&attr.size.to_le_bytes());
        bytes[80..84].copy_from_slice(&attr.mode.to_le_bytes());
        bytes[84..88].copy_from_slice(&attr.links.to_le_bytes());
        bytes[88..96].copy_from_slice(&attr.mtime_seconds.to_le_bytes());
        bytes[96..100].copy_from_slice(&attr.mtime_nanoseconds.to_le_bytes());
        bytes[100] = match attr.kind {
            Kind::File => 1,
            Kind::Directory => 2,
            Kind::Symlink => 3,
        };
        self.writer.write_all(&bytes)?;
        self.count += 1;
        Ok(())
    }

    fn validate(
        &mut self,
        objects: &ObjectBuffer<'_>,
        metadata_cache: &PortableMetadataCache,
        mut record: impl FnMut(InodeId) -> Result<InodeRecordV1>,
    ) -> Result<()> {
        self.writer.flush()?;
        Checkpoint::visit_file(
            self.writer.get_ref(),
            self.count,
            |_, inode, content, attr| {
                let final_record = record(inode)?;
                let metadata = match metadata_cache
                    .get_by_root(final_record.kind, final_record.metadata_root)
                {
                    Some(metadata) => metadata,
                    None => {
                        portable_metadata(objects, final_record.metadata_root, final_record.kind)?
                    }
                };
                let size = match final_record.kind {
                    InodeKind::RegularFile => {
                        rope::state(
                            objects,
                            FileStateRoot(content),
                            &mut RopeCounters::default(),
                        )?
                        .logical_len
                    }
                    InodeKind::Directory => 0,
                    InodeKind::Symlink => ObjectRead::with_authenticated_canonical(
                        objects,
                        content,
                        layerfs_content::tree::directory::codec::decode_symlink,
                    )?
                    .target
                    .len() as u64,
                };
                if final_record.content_root != content
                    || kind(final_record.kind) != attr.kind
                    || size != attr.size
                    || metadata.permission_mode != attr.mode
                    || metadata.mtime_seconds != attr.mtime_seconds
                    || metadata.mtime_nanoseconds != attr.mtime_nanoseconds
                    || (attr.kind != Kind::Directory
                        && final_record.namespace_ref_count != u64::from(attr.links))
                {
                    return Err(StorageError::Integrity("candidate checkpoint record"));
                }
                Ok(())
            },
        )
    }

    fn finish(mut self, built: BuiltRoot, generation: u64) -> Result<PreparedCommit> {
        self.writer.flush()?;
        let file = self
            .writer
            .into_inner()
            .map_err(|error| error.into_error())?;
        Ok(PreparedCommit {
            checkpoint: Checkpoint {
                root: built.root_id,
                generation,
                file,
                count: self.count,
            },
            built,
        })
    }
}

impl Checkpoint {
    pub(crate) fn visit(
        &self,
        mut visitor: impl FnMut(NodeId, InodeId, ObjectId, Attr) -> Result<()>,
    ) -> Result<()> {
        Self::visit_file(&self.file, self.count, &mut visitor)
    }

    fn visit_file(
        file: &File,
        count: u64,
        mut visitor: impl FnMut(NodeId, InodeId, ObjectId, Attr) -> Result<()>,
    ) -> Result<()> {
        use std::io::{Seek, SeekFrom};
        let mut reader = std::io::BufReader::with_capacity(256, file);
        reader.seek(SeekFrom::Start(0))?;
        for _ in 0..count {
            let mut bytes = [0; 104];
            reader.read_exact(&mut bytes)?;
            let node = NodeId(u64::from_le_bytes(bytes[..8].try_into().unwrap()));
            let attr = Attr {
                node,
                size: u64::from_le_bytes(bytes[72..80].try_into().unwrap()),
                mode: u32::from_le_bytes(bytes[80..84].try_into().unwrap()),
                links: u32::from_le_bytes(bytes[84..88].try_into().unwrap()),
                mtime_seconds: i64::from_le_bytes(bytes[88..96].try_into().unwrap()),
                mtime_nanoseconds: u32::from_le_bytes(bytes[96..100].try_into().unwrap()),
                kind: match bytes[100] {
                    1 => Kind::File,
                    2 => Kind::Directory,
                    3 => Kind::Symlink,
                    _ => return Err(StorageError::Integrity("checkpoint kind")),
                },
            };
            visitor(
                node,
                InodeId(bytes[8..40].try_into().unwrap()),
                ObjectId::from_bytes(&bytes[40..72])?,
                attr,
            )?;
        }
        if reader.read(&mut [0; 1])? != 0 {
            return Err(StorageError::Integrity("checkpoint journal length"));
        }
        Ok(())
    }
}

impl Workspace {
    pub(crate) fn build_candidate(&mut self) -> Result<PreparedCommit> {
        #[cfg(any(debug_assertions, feature = "test-instrumentation"))]
        if INJECT_CANDIDATE_FAILURE.with(|inject| inject.replace(false)) {
            return Err(StorageError::Integrity(
                "injected Workspace candidate failure",
            ));
        }
        self.build_frontier_candidate()
    }

    // Directory overlays already are the final binding delta. Applying their inode
    // edges directly preserves untouched subtrees, including a renamed directory,
    // without building either complete namespace manifest.
    fn build_frontier_candidate(&mut self) -> Result<PreparedCommit> {
        let started = Instant::now();
        self.policy.check_final_delta(1024)?;
        let batch_size = (self.policy.max_final_delta_memory_bytes / 4096).clamp(1, 128) as usize;
        let tree_scratch = usize::try_from(
            self.policy
                .max_final_delta_memory_bytes
                .saturating_sub(1024)
                / 2,
        )
        .unwrap_or(usize::MAX)
        .min(SORTED_TREE_UPDATE_SCRATCH_BYTES);
        let captured = self.take_capture();
        if let Some(captured) = &captured {
            layerfs_layerstack_store::note_workspace_capture(1, captured.len);
        }
        let (mut objects, captured) = match captured {
            Some(crate::capture::CapturedFile {
                node,
                len,
                root,
                counters,
                objects,
            }) => (
                ObjectBuffer::resume_prevalidated(&self.reader, objects),
                Some((node, len, root, counters)),
            ),
            None => (ObjectBuffer::new(&self.reader)?, None),
        };
        let mut inodes = FrontierInodes::new(
            self.base_root,
            // The fixed 1 KiB allowance includes the first 256-byte map entry.
            1 + (self
                .policy
                .max_final_delta_memory_bytes
                .saturating_sub(1024)
                / 512) as usize,
            tree_scratch,
            &self.spool,
        );
        let mut metadata_cache = PortableMetadataCache::default();
        let mut cdc_bytes_scanned = 0_u64;
        let mut checkpoint = CheckpointJournal::new(self)?;
        note_commit_phase(WorkspaceCommitPhase::CandidatePlan, started);
        let started = Instant::now();
        for &node in &self.dirty {
            let value = self
                .nodes
                .get(&node)
                .ok_or(StorageError::Integrity("dirty node"))?;
            layerfs_layerstack_store::note_workspace_namespace_visits(0, 0, 0, 0, 1);
            if value.paths.is_empty() && value.links == 0 {
                continue;
            }
            let inode = self.frontier_inode(node)?;
            let before = match value.canonical {
                Some(_) => Some(inodes.record(&objects, inode)?),
                None => None,
            };
            let attr = self.attr(node)?;
            let inode_kind = match attr.kind {
                Kind::File => InodeKind::RegularFile,
                Kind::Directory => InodeKind::Directory,
                Kind::Symlink => InodeKind::Symlink,
            };
            let content_root = match &value.data {
                Data::Directory(directory) => {
                    let mut content = match directory.base {
                        Some(base) => base,
                        None => empty_directory(&mut objects)?,
                    };
                    for desired in directory.changes.values() {
                        layerfs_layerstack_store::note_workspace_namespace_visits(
                            0,
                            u64::from(desired.is_some()),
                            0,
                            0,
                            0,
                        );
                    }
                    content = self.apply_frontier_directory(
                        &mut objects,
                        content,
                        &directory.changes,
                        batch_size,
                        tree_scratch,
                    )?;
                    content.0
                }
                Data::Symlink(target) => match before {
                    Some(record) => record.content_root,
                    None => filesystem::symlink_content(&mut objects, target.clone())?,
                },
                Data::File(_) => {
                    if let Some((_, _, root, counters)) =
                        captured.filter(|(id, _, _, _)| *id == node)
                    {
                        cdc_bytes_scanned = cdc_bytes_scanned
                            .checked_add(counters.cdc_bytes_scanned)
                            .ok_or(StorageError::Integrity("CDC counter"))?;
                        root.0
                    } else if let Some(record) = before {
                        if !self.file_may_differ(node, record.content_root)? {
                            record.content_root
                        } else {
                            let base = BaseEntry { record };
                            let changed = self.mutate_existing_file(&mut objects, node, base)?;
                            let changed = match changed {
                                Some(changed) => Some(changed),
                                None if self
                                    .incremental_file_supported(node, record.content_root) =>
                                {
                                    None
                                }
                                None => Some(rope::build(
                                    &mut objects,
                                    WorkspaceFileReader::new(self, node)?,
                                )?),
                            };
                            if let Some((root, counters)) = changed {
                                cdc_bytes_scanned = cdc_bytes_scanned
                                    .checked_add(counters.cdc_bytes_scanned)
                                    .ok_or(StorageError::Integrity("CDC counter"))?;
                                root.0
                            } else {
                                record.content_root
                            }
                        }
                    } else {
                        let (root, counters) =
                            rope::build(&mut objects, WorkspaceFileReader::new(self, node)?)?;
                        cdc_bytes_scanned = cdc_bytes_scanned
                            .checked_add(counters.cdc_bytes_scanned)
                            .ok_or(StorageError::Integrity("CDC counter"))?;
                        root.0
                    }
                }
            };
            let old_metadata = before
                .map(|record| {
                    portable_metadata(&CoreReader(&self.reader), record.metadata_root, record.kind)
                })
                .transpose()?;
            let metadata_root = if old_metadata.is_some_and(|metadata| {
                metadata.permission_mode == attr.mode
                    && metadata.mtime_seconds == attr.mtime_seconds
                    && metadata.mtime_nanoseconds == attr.mtime_nanoseconds
            }) {
                before.unwrap().metadata_root
            } else {
                metadata_cache
                    .get_or_build(
                        &mut objects,
                        inode_kind,
                        attr.mode,
                        attr.mtime_seconds,
                        attr.mtime_nanoseconds,
                    )?
                    .0
            };
            let record = InodeRecordV1 {
                kind: inode_kind,
                content_root,
                metadata_root,
                // New inodes have every final binding materialized. Existing
                // inodes can have unseen aliases and retain their stored count.
                namespace_ref_count: before.map_or(value.paths.len() as u64, |record| {
                    record.namespace_ref_count
                }),
            };
            layerfs_layerstack_store::note_workspace_namespace_visits(
                0,
                0,
                u64::from(before != Some(record)),
                u64::from(before == Some(record)),
                0,
            );
            checkpoint.push(node, inode, content_root, attr)?;
            if before != Some(record) {
                inodes.set(inode, Some(record))?;
            }
        }
        note_commit_phase(WorkspaceCommitPhase::Content, started);
        let started = Instant::now();
        // Add all final edges before releasing old ones. A move therefore never
        // destroys the moved inode, and aliases outside the overlay retain their
        // original references even when they were never materialized in Workspace.
        for additions in [true, false] {
            for value in self
                .dirty
                .iter()
                .filter_map(|node| self.nodes.get(node))
                .filter(|value| !value.paths.is_empty())
            {
                let Data::Directory(directory) = &value.data else {
                    continue;
                };
                for (name, desired) in &directory.changes {
                    if additions
                        && desired.is_some_and(|node| {
                            self.nodes
                                .get(&node)
                                .is_some_and(|value| value.canonical.is_none())
                        })
                    {
                        // Its final references were emitted with the new record.
                        // The removal pass still releases replaced old bindings.
                        continue;
                    }
                    let name = CanonicalName::from_bytes(name)?;
                    let before = match directory.base {
                        Some(base) => directory_lookup(
                            &CoreReader(&self.reader),
                            base,
                            &name,
                            &mut NamespaceCounters::default(),
                        )?,
                        None => None,
                    };
                    let after = desired.map(|node| self.frontier_inode(node)).transpose()?;
                    layerfs_layerstack_store::note_workspace_namespace_visits(
                        u64::from(before.is_some()),
                        u64::from(after.is_some()),
                        0,
                        0,
                        0,
                    );
                    if before == after {
                        continue;
                    }
                    if additions {
                        if let Some(inode) = after {
                            let mut record = inodes.record(&objects, inode)?;
                            record.namespace_ref_count = record
                                .namespace_ref_count
                                .checked_add(1)
                                .ok_or(StorageError::Integrity("namespace reference overflow"))?;
                            inodes.set(inode, Some(record))?;
                        }
                    } else if let Some(inode) = before {
                        inodes.release(
                            &objects,
                            &CoreReader(&self.reader),
                            inode,
                            self.policy.max_final_delta_memory_bytes,
                        )?;
                    }
                }
            }
        }
        checkpoint.validate(&objects, &metadata_cache, |inode| {
            inodes.record(&objects, inode)
        })?;
        inodes.finish(&mut objects)?;
        note_commit_phase(WorkspaceCommitPhase::Namespace, started);
        let started = Instant::now();
        let built = objects.finish(inodes.root, cdc_bytes_scanned)?;
        let built = checkpoint.finish(built, self.mutation_generation);
        note_commit_phase(WorkspaceCommitPhase::CandidateFinish, started);
        built
    }

    fn apply_frontier_directory(
        &self,
        objects: &mut ObjectBuffer<'_>,
        root: DirectoryStateRoot,
        changes: &BTreeMap<Vec<u8>, Option<NodeId>>,
        batch_size: usize,
        scratch_limit: usize,
    ) -> Result<DirectoryStateRoot> {
        let mut source_error = None;
        let deltas = changes.iter().map(|(name, desired)| {
            let result: Result<_> = (|| {
                Ok((
                    CanonicalName::from_bytes(name)?,
                    desired
                        .map(|child| self.frontier_inode(child))
                        .transpose()?,
                ))
            })();
            match result {
                Ok(delta) => Ok(delta),
                Err(error) => {
                    source_error = Some(error);
                    Err(layerfs_content::CoreError::InvalidRecord(
                        "Workspace directory delta",
                    ))
                }
            }
        });
        let sorted = directory_apply_sorted_with_budget(objects, root, deltas, scratch_limit);
        if let Some(error) = source_error {
            return Err(error);
        }
        match sorted {
            Ok((root, _)) => Ok(root),
            Err(
                layerfs_content::CoreError::ObjectLimitExceeded
                | layerfs_content::CoreError::Unsupported,
            ) => {
                let mut root = root;
                let mut batch = Vec::with_capacity(batch_size);
                for (name, desired) in changes {
                    batch.push((
                        CanonicalName::from_bytes(name)?,
                        desired
                            .map(|child| self.frontier_inode(child))
                            .transpose()?,
                    ));
                    if batch.len() == batch_size {
                        root =
                            filesystem::apply_directory_changes(objects, root, batch.drain(..))?.0;
                    }
                }
                if !batch.is_empty() {
                    root = filesystem::apply_directory_changes(objects, root, batch)?.0;
                }
                Ok(root)
            }
            Err(error) => Err(error.into()),
        }
    }

    fn frontier_inode(&self, node: NodeId) -> Result<InodeId> {
        let value = self
            .nodes
            .get(&node)
            .ok_or(StorageError::Integrity("frontier node"))?;
        if let Some(inode) = value.canonical {
            return Ok(inode);
        }
        let path = value
            .paths
            .first()
            .ok_or(StorageError::Integrity("frontier path"))?;
        let batch_allowance =
            (self.policy.max_final_delta_memory_bytes / 4096).clamp(1, 128) * 1024;
        self.policy
            .check_final_delta(batch_allowance.saturating_add(path_charge(path)))?;
        layerfs_layerstack_store::note_workspace_namespace_visits(0, 1, 0, 0, 0);
        // Bind new identity to this base snapshot: replacing one alias must not
        // accidentally reuse the still-live inode originally allocated at its path.
        Ok(filesystem::allocated_inode(
            self.base_root.to_bytes(),
            &CanonicalPath::new(path)?,
        ))
    }

    pub(crate) fn resolution_fingerprint(
        &mut self,
        affected_paths: &[CanonicalPath],
    ) -> Result<[u8; 32]> {
        let base = self.base_manifest()?;
        let final_view = self.final_manifest(manifest_charge(&base))?;
        let mut affected = affected_paths.iter().collect::<Vec<_>>();
        affected.sort();
        let mut digest = ContentDigestWriter::new();
        digest.write_all(b"layerfs/workspace-resolution/v2\0")?;
        for affected_path in affected {
            digest.write_all(b"A")?;
            frame(&mut digest, affected_path.as_bytes())?;
            for (path, entry) in final_view
                .iter()
                .filter(|(path, _)| path_intersects(path, affected_path.as_str()))
            {
                digest.write_all(b"E")?;
                frame(&mut digest, path.as_bytes())?;
                digest.write_all(&[match entry.attr.kind {
                    Kind::File => 1,
                    Kind::Directory => 2,
                    Kind::Symlink => 3,
                }])?;
                digest.write_all(&entry.attr.size.to_be_bytes())?;
                digest.write_all(&entry.attr.mode.to_be_bytes())?;
                digest.write_all(&entry.attr.links.to_be_bytes())?;
                digest.write_all(&entry.attr.mtime_seconds.to_be_bytes())?;
                digest.write_all(&entry.attr.mtime_nanoseconds.to_be_bytes())?;
                let node = self
                    .nodes
                    .get(&entry.node)
                    .ok_or(StorageError::Integrity("resolution node"))?;
                digest.write_all(&(node.paths.len() as u64).to_be_bytes())?;
                for alias in &node.paths {
                    frame(&mut digest, alias.as_bytes())?;
                }
                match entry.attr.kind {
                    Kind::File => {
                        let mut reader = WorkspaceFileReader::new(self, entry.node)?;
                        std::io::copy(&mut reader, &mut digest)?;
                    }
                    Kind::Symlink => frame(&mut digest, &self.readlink(entry.node)?)?,
                    Kind::Directory => {}
                }
            }
            digest.write_all(b"Z")?;
        }
        Ok(digest.finish())
    }

    fn base_manifest(&self) -> Result<BTreeMap<String, BaseEntry>> {
        let reader = CoreReader(&self.reader);
        let mut output = BTreeMap::new();
        let mut charge = 0_u64;
        let mut pending = vec![CanonicalPath::root()];
        while let Some(directory) = pending.pop() {
            let mut after = None;
            loop {
                let (page, _) = filesystem::list(
                    &reader,
                    self.base_root,
                    &directory,
                    after.as_ref(),
                    128,
                    256 * 1024,
                )?;
                layerfs_layerstack_store::note_workspace_namespace_visits(
                    page.entries.len() as u64,
                    0,
                    0,
                    0,
                    0,
                );
                for (name, _) in page.entries {
                    let path = join(&directory, name.as_str())?;
                    let resolved = filesystem::resolve(
                        &reader,
                        self.base_root,
                        &path,
                        &mut LogicalCounters::default(),
                    )?;
                    if resolved.record.kind == InodeKind::Directory {
                        pending.push(path.clone());
                    }
                    let path = path.as_str().to_owned();
                    charge = charge.saturating_add(path_charge(&path));
                    output.insert(
                        path,
                        BaseEntry {
                            record: resolved.record,
                        },
                    );
                    self.policy.check_final_delta(charge)?;
                }
                let Some(next) = page.continuation else { break };
                after = Some(next);
            }
        }
        Ok(output)
    }

    fn final_manifest(&mut self, base_charge: u64) -> Result<BTreeMap<String, FinalEntry>> {
        let mut output = BTreeMap::new();
        let mut charge = base_charge;
        let mut pending = vec![(ROOT, String::new())];
        while let Some((directory, prefix)) = pending.pop() {
            for (name, node) in self.directory_entries(directory)? {
                let dirty = self.dirty.contains(&node);
                layerfs_layerstack_store::note_workspace_namespace_visits(
                    0,
                    1,
                    u64::from(dirty),
                    u64::from(!dirty),
                    0,
                );
                let name = std::str::from_utf8(&name)
                    .map_err(|_| StorageError::Integrity("Workspace path"))?;
                let path = if prefix.is_empty() {
                    name.to_owned()
                } else {
                    format!("{prefix}/{name}")
                };
                let attr = self.attr(node)?;
                charge = charge.saturating_add(path_charge(&path));
                output.insert(path.clone(), FinalEntry { node, attr });
                self.policy.check_final_delta(charge)?;
                if attr.kind == Kind::Directory {
                    pending.push((node, path));
                }
            }
        }
        Ok(output)
    }

    fn file_may_differ(&self, node: NodeId, base: ObjectId) -> Result<bool> {
        match &self
            .nodes
            .get(&node)
            .ok_or(StorageError::NotFound("node"))?
            .data
        {
            Data::File(FileData::Base { root, .. }) if root.0 == base => Ok(false),
            Data::File(FileData::Edited {
                base: Some((root, base_len)),
                pieces,
                ..
            }) if root.0 == base => Ok(pieces.len() != *base_len
                || !matches!(pieces.pieces().as_slice(), [crate::file_edit::Piece::Base { root: piece_root, offset: 0, len }] if *piece_root == *root && *len == *base_len)),
            Data::File(_) => Ok(!self.file_matches(node, base)?),
            _ => Err(StorageError::InvalidInput("file")),
        }
    }

    fn incremental_file_supported(&self, node: NodeId, base: ObjectId) -> bool {
        matches!(
            self.nodes.get(&node).map(|node| &node.data),
            Some(Data::File(FileData::Edited {
                base: Some((root, _)),
                ..
            })) if root.0 == base
        )
    }

    fn mutate_existing_file(
        &self,
        objects: &mut ObjectBuffer<'_>,
        node: NodeId,
        base: BaseEntry,
    ) -> Result<Option<(FileStateRoot, RopeCounters)>> {
        let Data::File(FileData::Edited {
            base: Some((file_root, _)),
            pieces,
            ..
        }) = &self
            .nodes
            .get(&node)
            .ok_or(StorageError::NotFound("node"))?
            .data
        else {
            return Ok(None);
        };
        if file_root.0 != base.record.content_root {
            return Ok(None);
        }
        let (file_root, pieces) = (*file_root, pieces.pieces());
        let mut batch = FileMutationBatch::new(objects, Some(file_root))?;
        let mut changed = false;
        let original_len = layerfs_content::file::rope::state(
            &CoreReader(&self.reader),
            file_root,
            &mut RopeCounters::default(),
        )?
        .logical_len;
        let mut base_cursor = 0_u64;
        let mut final_cursor = 0_u64;
        let mut replacement_len = 0_u64;
        for piece in pieces {
            match piece {
                crate::file_edit::Piece::Base { root, offset, len } => {
                    if root != file_root || offset < base_cursor {
                        return Err(StorageError::Integrity("Workspace base piece order"));
                    }
                    let delete_len = offset - base_cursor;
                    if (delete_len != 0 || replacement_len != 0)
                        && (delete_len != replacement_len
                            || final_cursor != base_cursor
                            || !self.workspace_range_matches_base(
                                node,
                                file_root,
                                final_cursor,
                                final_cursor + replacement_len,
                            )?)
                    {
                        batch.replace(
                            final_cursor,
                            delete_len,
                            WorkspaceRangeReader::new(self, node, final_cursor, replacement_len)?,
                        )?;
                        changed = true;
                    }
                    final_cursor += replacement_len + len;
                    replacement_len = 0;
                    base_cursor = offset + len;
                }
                piece => {
                    replacement_len = replacement_len
                        .checked_add(piece.len())
                        .ok_or(StorageError::InvalidInput("file length"))?
                }
            }
        }
        let delete_len = original_len
            .checked_sub(base_cursor)
            .ok_or(StorageError::Integrity("Workspace base piece order"))?;
        if (delete_len != 0 || replacement_len != 0)
            && (delete_len != replacement_len
                || final_cursor != base_cursor
                || !self.workspace_range_matches_base(
                    node,
                    file_root,
                    final_cursor,
                    final_cursor + replacement_len,
                )?)
        {
            batch.replace(
                final_cursor,
                delete_len,
                WorkspaceRangeReader::new(self, node, final_cursor, replacement_len)?,
            )?;
            changed = true;
        }
        if batch.logical_len()? != self.attr(node)?.size {
            return Err(StorageError::Integrity("Workspace file mutation length"));
        }
        if !changed {
            return Ok(None);
        }
        Ok(Some(batch.finish()?))
    }

    fn workspace_range_matches_base(
        &self,
        node: NodeId,
        base: FileStateRoot,
        start: u64,
        end: u64,
    ) -> Result<bool> {
        let mut offset = start;
        while offset < end {
            let count = (end - offset).min(64 * 1024) as usize;
            let final_bytes = self.read(node, offset, count)?;
            let mut base_bytes = Vec::with_capacity(count);
            rope::read_range(
                &CoreReader(&self.reader),
                base,
                offset..offset + count as u64,
                &mut base_bytes,
            )?;
            if final_bytes != base_bytes {
                return Ok(false);
            }
            offset += count as u64;
        }
        Ok(true)
    }

    fn file_matches(&self, node: NodeId, base: ObjectId) -> Result<bool> {
        match &self
            .nodes
            .get(&node)
            .ok_or(StorageError::NotFound("node"))?
            .data
        {
            Data::File(FileData::Base { root, .. }) if root.0 == base => return Ok(true),
            Data::File(FileData::Edited {
                base: Some((root, base_len)),
                pieces,
                ..
            }) if root.0 == base
                && pieces.len() == *base_len
                && matches!(pieces.pieces().as_slice(), [crate::file_edit::Piece::Base { root: piece_root, offset: 0, len }] if *piece_root == *root && *len == *base_len) =>
            {
                return Ok(true)
            }
            Data::File(_) => {}
            _ => return Err(StorageError::InvalidInput("file")),
        }
        let mut final_digest = ContentDigestWriter::new();
        let mut input = WorkspaceFileReader::new(self, node)?;
        std::io::copy(&mut input, &mut final_digest)?;
        let mut base_digest = ContentDigestWriter::new();
        rope::read_all(
            &CoreReader(&self.reader),
            FileStateRoot(base),
            &mut base_digest,
        )?;
        Ok(final_digest.finish() == base_digest.finish())
    }
}

#[cfg(any(debug_assertions, feature = "test-instrumentation"))]
thread_local! {
    static INJECT_CANDIDATE_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(any(debug_assertions, feature = "test-instrumentation"))]
pub(crate) fn inject_candidate_failure_once() {
    INJECT_CANDIDATE_FAILURE.with(|inject| inject.set(true));
}

#[cfg(test)]
thread_local! {
    static INJECT_INODE_MERGE_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// Final inode changes are coalesced before touching the immutable base table.
// The bounded map spills ordered fixed records; tombstones never fall through to base.
struct FrontierInodes {
    root: ObjectId,
    pending: BTreeMap<InodeId, Option<InodeRecordV1>>,
    batch_size: usize,
    scratch_limit: usize,
    directory: std::path::PathBuf,
    spill: Option<File>,
    count: u64,
}

impl FrontierInodes {
    fn new(
        root: ObjectId,
        batch_size: usize,
        scratch_limit: usize,
        directory: &std::path::Path,
    ) -> Self {
        Self {
            root,
            pending: BTreeMap::new(),
            batch_size,
            scratch_limit,
            directory: directory.to_owned(),
            spill: None,
            count: 0,
        }
    }

    fn read(file: &File, index: u64) -> Result<(InodeId, Option<InodeRecordV1>)> {
        let mut bytes = [0; 144];
        file.read_exact_at(&mut bytes, index * 144)?;
        Ok((
            InodeId(bytes[..32].try_into().unwrap()),
            if bytes[32] == 0 {
                None
            } else {
                Some(InodeRecordV1 {
                    kind: InodeKind::try_from(bytes[32])?,
                    namespace_ref_count: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
                    content_root: ObjectId::from_bytes(&bytes[48..80])?,
                    metadata_root: ObjectId::from_bytes(&bytes[80..112])?,
                })
            },
        ))
    }

    fn write(file: &File, index: u64, inode: InodeId, record: Option<InodeRecordV1>) -> Result<()> {
        let mut bytes = [0; 144];
        bytes[..32].copy_from_slice(inode.as_bytes());
        if let Some(record) = record {
            bytes[32] = record.kind as u8;
            bytes[40..48].copy_from_slice(&record.namespace_ref_count.to_le_bytes());
            bytes[48..80].copy_from_slice(record.content_root.as_bytes());
            bytes[80..112].copy_from_slice(record.metadata_root.as_bytes());
        }
        file.write_all_at(&bytes, index * 144)?;
        Ok(())
    }

    fn spilled(&self, inode: InodeId) -> Result<Option<(u64, Option<InodeRecordV1>)>> {
        let Some(file) = &self.spill else {
            return Ok(None);
        };
        let (mut start, mut end) = (0, self.count);
        while start < end {
            let middle = start + (end - start) / 2;
            let (key, record) = Self::read(file, middle)?;
            match key.cmp(&inode) {
                std::cmp::Ordering::Equal => return Ok(Some((middle, record))),
                std::cmp::Ordering::Less => start = middle + 1,
                std::cmp::Ordering::Greater => end = middle,
            }
        }
        Ok(None)
    }

    fn record(&self, objects: &ObjectBuffer<'_>, inode: InodeId) -> Result<InodeRecordV1> {
        self.record_with_base(objects, inode, None)
    }

    fn record_with_base(
        &self,
        objects: &ObjectBuffer<'_>,
        inode: InodeId,
        base: Option<InodeRecordV1>,
    ) -> Result<InodeRecordV1> {
        if let Some(record) = self.pending.get(&inode) {
            return record.ok_or(StorageError::Integrity("released frontier inode"));
        }
        if let Some((_, record)) = self.spilled(inode)? {
            return record.ok_or(StorageError::Integrity("released frontier inode"));
        }
        if let Some(record) = base {
            return Ok(record);
        }
        let namespace = filesystem::namespace(objects, self.root)?;
        let id = inode_table_lookup(
            objects,
            InodeTableRoot(namespace.inode_table_root),
            inode,
            &mut InodeTableCounters::default(),
        )?
        .ok_or(StorageError::Integrity("frontier inode record"))?;
        Ok(ObjectStore::with_authenticated_canonical(
            objects,
            id,
            decode_inode_record,
        )?)
    }

    fn set(&mut self, inode: InodeId, record: Option<InodeRecordV1>) -> Result<()> {
        if let Some(pending) = self.pending.get_mut(&inode) {
            *pending = record;
        } else if let Some((index, _)) = self.spilled(inode)? {
            Self::write(self.spill.as_ref().unwrap(), index, inode, record)?;
        } else {
            if self.pending.len() == self.batch_size {
                self.merge_pending()?;
            }
            self.pending.insert(inode, record);
        }
        Ok(())
    }

    fn merge_pending(&mut self) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let file = anonymous_journal(&self.directory)?;
        let mut pending = self.pending.iter().peekable();
        let (mut cursor, mut count) = (0, 0);
        let mut old = if cursor < self.count {
            Some(Self::read(self.spill.as_ref().unwrap(), cursor)?)
        } else {
            None
        };
        // ponytail: new-key merges cost O(N² / buffer capacity) beyond the budget;
        // use tiered runs if substantially larger changes make spill IO dominant.
        while old.is_some() || pending.peek().is_some() {
            let delta = match (old, pending.peek()) {
                (Some(previous), Some((&key, _))) if previous.0 < key => {
                    cursor += 1;
                    old = if cursor < self.count {
                        Some(Self::read(self.spill.as_ref().unwrap(), cursor)?)
                    } else {
                        None
                    };
                    previous
                }
                (Some(previous), None) => {
                    cursor += 1;
                    old = if cursor < self.count {
                        Some(Self::read(self.spill.as_ref().unwrap(), cursor)?)
                    } else {
                        None
                    };
                    previous
                }
                _ => {
                    let (&key, &record) = pending.next().unwrap();
                    if old.is_some_and(|entry| entry.0 == key) {
                        cursor += 1;
                        old = if cursor < self.count {
                            Some(Self::read(self.spill.as_ref().unwrap(), cursor)?)
                        } else {
                            None
                        };
                    }
                    (key, record)
                }
            };
            Self::write(&file, count, delta.0, delta.1)?;
            count += 1;
        }
        #[cfg(test)]
        if INJECT_INODE_MERGE_FAILURE.with(|inject| inject.replace(false)) {
            return Err(StorageError::Integrity("injected inode merge failure"));
        }
        // Keep previous spill and pending entries intact until all writes succeed.
        self.spill = Some(file);
        self.count = count;
        self.pending.clear();
        Ok(())
    }

    fn finish(&mut self, objects: &mut ObjectBuffer<'_>) -> Result<()> {
        self.merge_pending()?;
        let Some(file) = &self.spill else {
            return Ok(());
        };
        // Encode each final record once, before lending the object writer to the
        // sorted builder. The same journal retains raw records for bounded fallback.
        for index in 0..self.count {
            if let (_, Some(record)) = Self::read(file, index)? {
                let id = objects.put_owned(encode_inode_record(record)?)?;
                file.write_all_at(id.as_bytes(), index * 144 + 112)?;
            }
        }
        let namespace = filesystem::namespace(objects, self.root)?;
        let mut source_error = None;
        let deltas = (0..self.count).map(|index| {
            let result: Result<_> = (|| {
                let (inode, record) = Self::read(file, index)?;
                let id = if record.is_some() {
                    let mut id = [0; 32];
                    file.read_exact_at(&mut id, index * 144 + 112)?;
                    Some(ObjectId::from_bytes(&id)?)
                } else {
                    None
                };
                Ok((inode, id))
            })();
            result.map_err(|error| {
                source_error = Some(error);
                layerfs_content::CoreError::InvalidRecord("Workspace inode delta")
            })
        });
        let sorted = inode_table_apply_sorted_with_budget(
            objects,
            InodeTableRoot(namespace.inode_table_root),
            deltas,
            self.scratch_limit,
        );
        if let Some(error) = source_error {
            return Err(error);
        }
        self.root = match sorted {
            Ok((table, _)) if table.0 == namespace.inode_table_root => self.root,
            Ok((table, _)) => objects.put_owned(encode_namespace_root(NamespaceRootV1 {
                inode_table_root: table.0,
                ..namespace
            })?)?,
            Err(
                layerfs_content::CoreError::ObjectLimitExceeded
                | layerfs_content::CoreError::Unsupported,
            ) => {
                let mut root = self.root;
                for index in 0..self.count {
                    let (inode, record) = Self::read(file, index)?;
                    let mutation = match record {
                        Some(record) => InodeMutation::Upsert { inode, record },
                        None => InodeMutation::Remove { inode },
                    };
                    root = filesystem::apply_inode_mutations(objects, root, [mutation])?.root();
                }
                root
            }
            Err(error) => return Err(error.into()),
        };
        Ok(())
    }

    fn release(
        &mut self,
        objects: &ObjectBuffer<'_>,
        base: &CoreReader<'_>,
        inode: InodeId,
        budget: u64,
    ) -> Result<()> {
        struct Cursor {
            root: DirectoryStateRoot,
            after: Option<CanonicalName>,
            children: std::collections::VecDeque<(InodeId, InodeRecordV1)>,
            finished: bool,
        }
        let namespace = filesystem::namespace(objects, self.root)?;
        let mut directories: Vec<Cursor> = Vec::new();
        let mut next = Some((inode, None));
        loop {
            if let Some((inode, prefetched)) = next.take() {
                let started = Instant::now();
                // Earlier additions/releases override authenticated page prefetch.
                let mut record = self.record_with_base(objects, inode, prefetched)?;
                record.namespace_ref_count = record
                    .namespace_ref_count
                    .checked_sub(1)
                    .ok_or(StorageError::Integrity("namespace reference underflow"))?;
                self.set(inode, (record.namespace_ref_count != 0).then_some(record))?;
                note_commit_phase(WorkspaceCommitPhase::DeletionRecords, started);
                if record.namespace_ref_count == 0 && record.kind == InodeKind::Directory {
                    directories.push(Cursor {
                        root: DirectoryStateRoot(record.content_root),
                        after: None,
                        children: Default::default(),
                        finished: false,
                    });
                }
            }
            let held = self.pending.len() as u64 * 256
                + directories
                    .iter()
                    .map(|cursor| 512 + cursor.children.capacity() as u64 * 144)
                    .sum::<u64>();
            let Some(cursor) = directories.last_mut() else {
                break;
            };
            if let Some((inode, record)) = cursor.children.pop_front() {
                next = Some((inode, Some(record)));
                continue;
            }
            if cursor.finished {
                directories.pop();
                continue;
            }
            let limit = budget.saturating_sub(held.saturating_add(1024)) / 1024;
            if limit == 0 {
                return Err(StorageError::InvalidInput("workspace final-delta limit"));
            }
            let limit = limit.min(128) as usize;
            let started = Instant::now();
            let page = directory_page_after(
                objects,
                cursor.root,
                cursor.after.as_ref(),
                limit,
                limit * 512,
                &mut NamespaceCounters::default(),
            )?;
            note_commit_phase(WorkspaceCommitPhase::DeletionCursor, started);
            layerfs_layerstack_store::note_workspace_namespace_visits(
                page.entries.len() as u64,
                0,
                0,
                0,
                0,
            );
            cursor.finished = page.continuation.is_none();
            cursor.after = page.continuation;
            let started = Instant::now();
            let inodes = page
                .entries
                .iter()
                .map(|(_, inode)| *inode)
                .collect::<Vec<_>>();
            // Removed directories originate in the immutable base. Batch their
            // records there, then preserve sequential changes at consumption above.
            // A lookup batch also holds decoded and raw inode-table pages.
            // Small policies keep the existing single-key canonical read width.
            let lookup_limit = (budget.saturating_sub(held + limit as u64 * 1024) / (32 * 1024))
                .clamp(1, 128) as usize;
            let mut ids = Vec::with_capacity(inodes.len());
            for keys in inodes.chunks(lookup_limit) {
                for id in inode_table_lookup_many(
                    base,
                    InodeTableRoot(namespace.inode_table_root),
                    keys,
                    &mut InodeTableCounters::default(),
                )? {
                    ids.push(id.ok_or(StorageError::Integrity("deleted inode record"))?);
                }
            }
            let mut records = BTreeMap::new();
            base.get_authenticated_batch(&ids, |id, payload| {
                records.insert(
                    id,
                    decode_inode_record(&layerfs_content::encode_bytes_object(payload)?)?,
                );
                Ok(())
            })?;
            cursor.children = inodes
                .into_iter()
                .zip(ids)
                .map(|(inode, id)| (inode, records[&id]))
                .collect();
            note_commit_phase(WorkspaceCommitPhase::DeletionRecords, started);
        }
        Ok(())
    }
}

fn note_commit_phase(phase: WorkspaceCommitPhase, started: Instant) {
    layerfs_layerstack_store::note_workspace_commit_phase(
        phase,
        started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64,
    );
}

fn frame(output: &mut impl Write, value: &[u8]) -> Result<()> {
    output.write_all(&(value.len() as u64).to_be_bytes())?;
    output.write_all(value)?;
    Ok(())
}

fn path_intersects(left: &str, right: &str) -> bool {
    left == right || path_is_ancestor(left, right) || path_is_ancestor(right, left)
}

fn path_is_ancestor(parent: &str, child: &str) -> bool {
    parent.is_empty()
        || (child.starts_with(parent) && child.as_bytes().get(parent.len()) == Some(&b'/'))
}

fn manifest_charge<T>(manifest: &BTreeMap<String, T>) -> u64 {
    manifest.keys().map(|path| path_charge(path)).sum()
}

fn path_charge(path: &str) -> u64 {
    (path.len() as u64).saturating_mul(4).saturating_add(512)
}

struct WorkspaceFileReader<'a> {
    source: WorkspaceFileSource<'a>,
    offset: u64,
    len: u64,
}

#[derive(Clone, Copy)]
enum WorkspaceFileSource<'a> {
    Direct(&'a File),
    Mixed {
        workspace: &'a Workspace,
        node: NodeId,
    },
}

struct WorkspaceRangeReader<'a> {
    workspace: &'a Workspace,
    node: NodeId,
    offset: u64,
    end: u64,
}

impl<'a> WorkspaceRangeReader<'a> {
    fn new(workspace: &'a Workspace, node: NodeId, offset: u64, len: u64) -> Result<Self> {
        let end = offset
            .checked_add(len)
            .ok_or(StorageError::InvalidInput("file range"))?;
        if end > workspace.attr(node)?.size {
            return Err(StorageError::InvalidInput("file range"));
        }
        Ok(Self {
            workspace,
            node,
            offset,
            end,
        })
    }
}

impl Read for WorkspaceRangeReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        if self.offset == self.end || output.is_empty() {
            return Ok(0);
        }
        let bytes = self
            .workspace
            .read(
                self.node,
                self.offset,
                output.len().min((self.end - self.offset) as usize),
            )
            .map_err(std::io::Error::other)?;
        output[..bytes.len()].copy_from_slice(&bytes);
        self.offset += bytes.len() as u64;
        Ok(bytes.len())
    }
}

impl<'a> WorkspaceFileReader<'a> {
    fn new(workspace: &'a Workspace, node: NodeId) -> Result<Self> {
        let len = workspace.attr(node)?.size;
        let source = match &workspace
            .nodes
            .get(&node)
            .ok_or(StorageError::NotFound("node"))?
            .data
        {
            Data::File(FileData::Edited {
                base: None,
                spool,
                spool_high_water,
                pieces,
                ..
            }) if *spool_high_water == len
                && pieces
                    .pieces()
                    .iter()
                    .try_fold(0_u64, |offset, piece| match piece {
                        crate::file_edit::Piece::Spool {
                            offset: source,
                            len,
                        } if *source == offset => offset.checked_add(*len),
                        _ => None,
                    })
                    == Some(len) =>
            {
                WorkspaceFileSource::Direct(workspace.spool_file(node, spool)?)
            }
            Data::File(_) => WorkspaceFileSource::Mixed { workspace, node },
            _ => return Err(StorageError::InvalidInput("file")),
        };
        Ok(Self {
            source,
            offset: 0,
            len,
        })
    }
}

impl Read for WorkspaceFileReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        if self.offset == self.len || output.is_empty() {
            return Ok(0);
        }
        let count = output.len().min((self.len - self.offset) as usize);
        match self.source {
            WorkspaceFileSource::Direct(file) => {
                let mut read = 0;
                while read < count {
                    let next = file.read_at(&mut output[read..count], self.offset + read as u64)?;
                    if next == 0 {
                        return Err(std::io::ErrorKind::UnexpectedEof.into());
                    }
                    read += next;
                }
                self.offset += read as u64;
                Ok(read)
            }
            WorkspaceFileSource::Mixed { workspace, node } => {
                let bytes = workspace
                    .read(node, self.offset, count)
                    .map_err(std::io::Error::other)?;
                output[..bytes.len()].copy_from_slice(&bytes);
                self.offset += bytes.len() as u64;
                Ok(bytes.len())
            }
        }
    }
}

fn kind(kind: InodeKind) -> Kind {
    match kind {
        InodeKind::RegularFile => Kind::File,
        InodeKind::Directory => Kind::Directory,
        InodeKind::Symlink => Kind::Symlink,
    }
}

fn join(parent: &CanonicalPath, name: &str) -> Result<CanonicalPath> {
    let value = if parent.is_root() {
        name.to_owned()
    } else {
        format!("{}/{name}", parent.as_str())
    };
    CanonicalPath::new(&value).map_err(Into::into)
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = self.clear_spool();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layerfs_layerstack_store::{
        CommitOutcome, EntityName, LayerStackInitialization, LayerStackStore, LocalForkSource,
    };
    use std::collections::BTreeSet;

    fn empty_workspace(label: &str) -> (std::path::PathBuf, Workspace) {
        let root = std::env::temp_dir().join(format!(
            "layerfs-direct-spool-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let store = LayerStackStore::create(root.join("store.sqlite")).unwrap();
        let layer = store
            .initialize_layerstack(
                EntityName::new("project").unwrap(),
                LayerStackInitialization::Empty,
            )
            .unwrap()
            .genesis_layer_id;
        let branch = store
            .fork_branch(
                EntityName::new(label).unwrap(),
                LocalForkSource::Layer { layer_id: layer },
            )
            .unwrap();
        let workspace = Workspace::open(store, branch, root.join("spool")).unwrap();
        (root, workspace)
    }

    #[test]
    fn paged_release_keeps_prior_moves_and_repeated_unseen_aliases() {
        let (root, mut workspace) = empty_workspace("paged-release");
        let directory = workspace.mkdir(ROOT, b"remove", 0o700).unwrap().node;
        for index in 0..80 {
            let name = format!("f{index:03}");
            let node = workspace
                .create_file(directory, name.as_bytes(), 0o600)
                .unwrap()
                .node;
            workspace.write(node, 0, name.as_bytes()).unwrap();
            if index == 0 {
                workspace.link(node, ROOT, b"outside").unwrap();
                workspace.link(node, ROOT, b"hidden").unwrap();
                workspace.link(node, directory, b"another").unwrap();
            }
        }
        workspace.commit().unwrap();
        let branch = workspace.branch_id;
        let store = workspace.store.clone();
        drop(workspace);
        let mut workspace = Workspace::open_with_policy(
            store,
            branch,
            root.join("again"),
            crate::ResourcePolicy {
                max_final_delta_memory_bytes: 16 * 1024,
                ..crate::ResourcePolicy::default()
            },
        )
        .unwrap();
        let directory = workspace.lookup(ROOT, b"remove").unwrap().node;
        workspace
            .rename(directory, b"f001", ROOT, b"moved", false)
            .unwrap();
        for index in 0..80 {
            if index != 1 {
                workspace
                    .unlink(directory, format!("f{index:03}").as_bytes(), false)
                    .unwrap();
            }
        }
        workspace.unlink(directory, b"another", false).unwrap();
        workspace.unlink(ROOT, b"remove", true).unwrap();
        workspace.commit().unwrap();
        assert!(workspace.lookup(ROOT, b"remove").is_err());
        let outside = workspace.lookup(ROOT, b"outside").unwrap();
        assert_eq!(outside.links, 2);
        assert_eq!(
            workspace.lookup(ROOT, b"hidden").unwrap().node,
            outside.node
        );
        let moved = workspace.lookup(ROOT, b"moved").unwrap();
        assert_eq!(moved.links, 1);
        assert_eq!(workspace.read(moved.node, 0, 4).unwrap(), b"f001");
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn coalesced_spill_keeps_tombstones_and_reference_updates_across_merge_failure() {
        let (root, mut workspace) = empty_workspace("inode-merge");
        let file = workspace.create_file(ROOT, b"file", 0o600).unwrap().node;
        workspace.write(file, 0, b"data").unwrap();
        workspace.commit().unwrap();
        let objects = ObjectBuffer::new(&workspace.reader).unwrap();
        let inode = workspace.nodes[&file].canonical.unwrap();
        let mut inodes = FrontierInodes::new(workspace.base_root, 1, 0, &workspace.spool);
        let mut record = inodes.record(&objects, inode).unwrap();
        record.namespace_ref_count = 3;
        inodes.set(inode, Some(record)).unwrap();
        let extra = InodeId::allocate([9; 32], 1);
        inodes.set(extra, Some(record)).unwrap(); // spills the first key
        record.namespace_ref_count = 2;
        inodes.set(inode, Some(record)).unwrap(); // updates an existing spilled key
        inodes.set(extra, None).unwrap();
        INJECT_INODE_MERGE_FAILURE.with(|inject| inject.set(true));
        assert!(inodes.merge_pending().is_err());
        assert_eq!(
            inodes.record(&objects, inode).unwrap().namespace_ref_count,
            2
        );
        assert!(inodes.record(&objects, extra).is_err());
        inodes.merge_pending().unwrap();
        assert_eq!(inodes.count, 2);
        assert_eq!(
            inodes.record(&objects, inode).unwrap().namespace_ref_count,
            2
        );
        assert!(inodes.record(&objects, extra).is_err());
        let first = FrontierInodes::read(inodes.spill.as_ref().unwrap(), 0).unwrap();
        let second = FrontierInodes::read(inodes.spill.as_ref().unwrap(), 1).unwrap();
        assert!(first.0 < second.0);
        drop(objects);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dirty_checkpoint_keeps_rejected_edit_accounting_and_unseen_pinned_alias() {
        let (root, mut workspace) = empty_workspace("dirty-checkpoint");
        let a = workspace.create_file(ROOT, b"a", 0o600).unwrap().node;
        workspace.create_file(ROOT, b"b", 0o600).unwrap();
        workspace.write(a, 0, b"abc").unwrap();
        workspace.link(a, ROOT, b"hidden").unwrap();
        workspace.commit().unwrap();
        let branch = workspace.branch_id;
        let store = workspace.store.clone();
        drop(workspace);
        let mut workspace = Workspace::open(store, branch, root.join("again")).unwrap();
        let a = workspace.lookup(ROOT, b"a").unwrap().node;
        let b = workspace.lookup(ROOT, b"b").unwrap().node;
        workspace.policy.max_spool_bytes = 0;
        assert!(workspace.write(a, 0, b"x").is_err());
        workspace.chmod(b, 0o640).unwrap();
        workspace.commit().unwrap();
        workspace.policy.max_spool_bytes = 1024;
        workspace.write(a, 0, b"x").unwrap();
        workspace.pin(a, false).unwrap();
        workspace.unlink(ROOT, b"a", false).unwrap();
        workspace.commit().unwrap();
        let mut published = Vec::new();
        filesystem::stream(
            &CoreReader(&workspace.reader),
            workspace.base_root,
            &CanonicalPath::new("hidden").unwrap(),
            &mut published,
        )
        .unwrap();
        assert_eq!(published, b"xbc");
        assert_eq!(workspace.lookup(ROOT, b"hidden").unwrap().node, a);
        assert_eq!(workspace.read(a, 0, 3).unwrap(), b"xbc");
        workspace.unpin(a).unwrap();
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn checkpoint_checks_final_references_and_preserves_payload_limit() {
        let (root, mut workspace) = empty_workspace("checkpoint-checks");
        workspace.policy.max_spool_bytes = 3;
        let file = workspace.create_file(ROOT, b"file", 0o640).unwrap().node;
        workspace.write(file, 0, b"abc").unwrap();
        workspace.commit().unwrap();
        workspace.policy.max_spool_bytes = 0;
        workspace.set_mtime(file, 17, 3).unwrap();
        workspace.commit().unwrap();
        let mut journal = CheckpointJournal::new(&workspace).unwrap();
        let attr = workspace.attr(file).unwrap();
        let inode = workspace.nodes[&file].canonical.unwrap();
        let objects = ObjectBuffer::new(&workspace.reader).unwrap();
        let mut record = FrontierInodes::new(workspace.base_root, 1, 0, &workspace.spool)
            .record(&objects, inode)
            .unwrap();
        journal
            .push(file, inode, record.content_root, attr)
            .unwrap();
        journal
            .validate(&objects, &PortableMetadataCache::default(), |_| Ok(record))
            .unwrap();
        record.namespace_ref_count += 1;
        assert!(matches!(
            journal.validate(&objects, &PortableMetadataCache::default(), |_| Ok(record)),
            Err(StorageError::Integrity("candidate checkpoint record"))
        ));
        drop(objects);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn structural_frontier_keeps_unrelated_namespace_out_of_budget() {
        let (root, empty) = empty_workspace("frontier");
        let store = empty.store.clone();
        drop(empty);
        let source = root.join("source");
        std::fs::create_dir_all(source.join("background")).unwrap();
        for index in 0..200 {
            std::fs::write(
                source.join(format!("background/file-{index:03}")),
                b"unchanged",
            )
            .unwrap();
        }
        for name in ["tree", "keep"] {
            std::fs::create_dir(source.join(name)).unwrap();
            std::fs::write(source.join(name).join("child"), name.as_bytes()).unwrap();
        }
        std::fs::hard_link(source.join("tree/child"), source.join("outside")).unwrap();
        std::fs::write(source.join("shared"), b"original alias").unwrap();
        std::fs::hard_link(source.join("shared"), source.join("hidden")).unwrap();
        let layer = store
            .initialize_layerstack(
                EntityName::new("fixture").unwrap(),
                LayerStackInitialization::Directory(source),
            )
            .unwrap()
            .genesis_layer_id;
        let branch = store
            .fork_branch(
                EntityName::new("changed").unwrap(),
                LocalForkSource::Layer { layer_id: layer },
            )
            .unwrap();
        let mut workspace = Workspace::open_with_policy(
            store.clone(),
            branch,
            root.join("changed-spool"),
            crate::ResourcePolicy {
                max_final_delta_memory_bytes: 16 * 1024,
                ..crate::ResourcePolicy::default()
            },
        )
        .unwrap();
        // The former structural fallback cannot even hold this small fixture.
        assert!(matches!(
            workspace.base_manifest(),
            Err(StorageError::InvalidInput("workspace final-delta limit"))
        ));
        let before_root = workspace.base_root;
        let reader = store.snapshot_reader(before_root);
        let before = |path: &str| {
            filesystem::resolve(
                &CoreReader(&reader),
                before_root,
                &CanonicalPath::new(path).unwrap(),
                &mut LogicalCounters::default(),
            )
            .unwrap()
        };
        let background = before("background");
        let keep = before("keep");
        let old_shared = before("shared");
        let old_tree = before("tree");
        let old_child = before("tree/child");

        // Neither hidden alias is looked up: releasing only the known binding
        // must still retain its inode and content under the unseen name.
        workspace.unlink(ROOT, b"shared", false).unwrap();
        let replacement = workspace.create_file(ROOT, b"shared", 0o600).unwrap();
        workspace
            .write(replacement.node, 0, b"replacement")
            .unwrap();
        workspace
            .rename(ROOT, b"tree", ROOT, b"moved", false)
            .unwrap();
        let moved = workspace.lookup(ROOT, b"moved").unwrap();
        workspace.unlink(moved.node, b"child", false).unwrap();
        workspace.unlink(ROOT, b"moved", true).unwrap();
        workspace
            .rename(ROOT, b"keep", ROOT, b"kept", false)
            .unwrap();
        let added = workspace.create_file(ROOT, b"added", 0o640).unwrap();
        workspace.write(added.node, 0, b"linked payload").unwrap();
        workspace.link(added.node, ROOT, b"added-alias").unwrap();
        workspace.link(added.node, ROOT, b"second-alias").unwrap();
        workspace.unlink(ROOT, b"added", false).unwrap();
        workspace.mkdir(ROOT, b"new-directory", 0o750).unwrap();
        for index in 0..120 {
            workspace
                .create_file(ROOT, format!("new-{index:03}").as_bytes(), 0o600)
                .unwrap();
        }
        workspace.commit().unwrap();
        let final_root = workspace.base_root;
        let reader = store.snapshot_reader(final_root);
        let core = CoreReader(&reader);
        let resolve = |path: &str| {
            filesystem::resolve(
                &core,
                final_root,
                &CanonicalPath::new(path).unwrap(),
                &mut LogicalCounters::default(),
            )
            .unwrap()
        };
        assert_eq!(resolve("background").record, background.record);
        assert_eq!(resolve("kept").inode, keep.inode);
        assert_eq!(resolve("kept").record, keep.record);
        assert_eq!(resolve("hidden").inode, old_shared.inode);
        assert_eq!(resolve("hidden").record.namespace_ref_count, 1);
        assert_ne!(resolve("shared").inode, old_shared.inode);
        assert_eq!(resolve("outside").inode, old_child.inode);
        assert_eq!(resolve("outside").record.namespace_ref_count, 1);
        assert_eq!(resolve("added-alias").record.namespace_ref_count, 2);
        assert_eq!(resolve("added-alias").inode, resolve("second-alias").inode);
        assert_eq!(resolve("new-directory").record.namespace_ref_count, 1);
        for (path, expected) in [
            ("hidden", b"original alias".as_slice()),
            ("shared", b"replacement"),
            ("outside", b"tree"),
            ("kept/child", b"keep"),
            ("added-alias", b"linked payload"),
        ] {
            let mut bytes = Vec::new();
            filesystem::stream(
                &core,
                final_root,
                &CanonicalPath::new(path).unwrap(),
                &mut bytes,
            )
            .unwrap();
            assert_eq!(bytes, expected, "{path}");
        }
        for index in 0..120 {
            assert_eq!(
                resolve(&format!("new-{index:03}"))
                    .record
                    .namespace_ref_count,
                1
            );
        }
        let namespace = filesystem::namespace(&core, final_root).unwrap();
        let table = layerfs_content::tree::inode::InodeTableRoot(namespace.inode_table_root);
        assert!(inode_table_lookup(
            &core,
            table,
            old_tree.inode,
            &mut InodeTableCounters::default()
        )
        .unwrap()
        .is_none());
        let entries = layerfs_content::tree::inode::inode_table_entries(
            &core,
            table,
            &mut InodeTableCounters::default(),
        )
        .unwrap();
        // root + background directory/files + kept directory/file + outside +
        // hidden + replacement + aliased new file + new directory + 120 files.
        assert_eq!(entries.len(), 329);
        // A valid long path still must fit the transient planner allocation,
        // together with its pending inode batch, under a custom small policy.
        workspace.policy.max_final_delta_memory_bytes = 4096;
        let name = "x".repeat(250);
        let mut parent = ROOT;
        for _ in 0..4 {
            parent = workspace
                .mkdir(parent, name.as_bytes(), 0o700)
                .unwrap()
                .node;
        }
        let path = workspace.nodes[&parent].paths.first().unwrap();
        assert!(CanonicalPath::new(path).is_ok());
        assert!(matches!(
            workspace.frontier_inode(parent),
            Err(StorageError::InvalidInput("workspace final-delta limit"))
        ));
        drop(workspace);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dense_existing_file_delta_uses_bounded_frontier_and_preserves_aliases() {
        let (root, mut workspace) = empty_workspace("dense-frontier");
        let mut files = Vec::new();
        for index in 0..16 {
            let name = format!("file-{index:02}");
            let node = workspace
                .create_file(ROOT, name.as_bytes(), 0o640)
                .unwrap()
                .node;
            workspace.write(node, 0, b"initial").unwrap();
            files.push((name, node));
        }
        workspace.link(files[0].1, ROOT, b"alias").unwrap();
        let sentinel = workspace
            .create_file(ROOT, b"sentinel", 0o600)
            .unwrap()
            .node;
        workspace.write(sentinel, 0, b"unchanged").unwrap();
        workspace.commit().unwrap();
        let original_inodes = files
            .iter()
            .map(|(_, node)| workspace.nodes[node].canonical.unwrap())
            .collect::<Vec<_>>();
        workspace.policy.max_final_delta_memory_bytes = 4096;
        for (index, (_, node)) in files.iter().enumerate() {
            workspace
                .write(*node, 0, format!("changed-{index:02}").as_bytes())
                .unwrap();
            workspace.set_mtime(*node, 1700000001, 123).unwrap();
        }
        assert!(workspace.nodes.values().all(|node| !matches!(&node.data,
            Data::Directory(directory) if !directory.changes.is_empty())));
        assert!(
            workspace
                .mutation_paths
                .keys()
                .map(|path| path_charge(path))
                .sum::<u64>()
                > workspace.policy.max_final_delta_memory_bytes
        );
        workspace.commit().unwrap();
        let reader = workspace.store.snapshot_reader(workspace.base_root);
        let core = CoreReader(&reader);
        for (index, (name, _)) in files.iter().enumerate() {
            let path = CanonicalPath::new(name).unwrap();
            let resolved = filesystem::resolve(
                &core,
                workspace.base_root,
                &path,
                &mut LogicalCounters::default(),
            )
            .unwrap();
            assert_eq!(resolved.inode, original_inodes[index]);
            let metadata =
                portable_metadata(&core, resolved.record.metadata_root, resolved.record.kind)
                    .unwrap();
            assert_eq!(
                (
                    metadata.permission_mode,
                    metadata.mtime_seconds,
                    metadata.mtime_nanoseconds
                ),
                (0o640, 1700000001, 123)
            );
            let mut bytes = Vec::new();
            filesystem::stream(&core, workspace.base_root, &path, &mut bytes).unwrap();
            assert_eq!(bytes, format!("changed-{index:02}").as_bytes());
        }
        for (path, expected) in [
            ("alias", b"changed-00".as_slice()),
            ("sentinel", b"unchanged"),
        ] {
            let mut bytes = Vec::new();
            filesystem::stream(
                &core,
                workspace.base_root,
                &CanonicalPath::new(path).unwrap(),
                &mut bytes,
            )
            .unwrap();
            assert_eq!(bytes, expected);
        }
        let alias = filesystem::resolve(
            &core,
            workspace.base_root,
            &CanonicalPath::new("alias").unwrap(),
            &mut LogicalCounters::default(),
        )
        .unwrap();
        assert_eq!(alias.inode, original_inodes[0]);
        assert_eq!(alias.record.namespace_ref_count, 2);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn clean_workspace_commit_is_immediately_up_to_date() {
        let (root, mut workspace) = empty_workspace("clean-commit");
        let base_root = workspace.base_root;
        let (outcome, transition) = workspace.commit().unwrap();
        assert!(matches!(
            outcome,
            CommitOutcome::UpToDate { root_id } if root_id == base_root
        ));
        assert_eq!(transition, crate::lifecycle::CommitTransition::Checkpointed);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fully_charged_new_file_reads_directly_from_retained_spool() {
        let (root, mut workspace) = empty_workspace("direct-reader");
        let data = (0..128 * 1024)
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();
        let file = workspace.create_file(ROOT, b"full", 0o600).unwrap();
        workspace.write(file.node, 0, &data).unwrap();
        let mut reader = WorkspaceFileReader::new(&workspace, file.node).unwrap();
        assert!(matches!(reader.source, WorkspaceFileSource::Direct(_)));
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, data);

        let sparse = workspace.create_file(ROOT, b"sparse", 0o600).unwrap();
        workspace.write(sparse.node, 4096, b"x").unwrap();
        assert!(matches!(
            WorkspaceFileReader::new(&workspace, sparse.node)
                .unwrap()
                .source,
            WorkspaceFileSource::Mixed { .. }
        ));
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sequential_new_file_capture_is_canonical_and_commit_ready() {
        let (root, mut workspace) = empty_workspace("streaming-capture");
        let data = (0..2 * 1024 * 1024)
            .map(|index| ((index * 29 + index / 11) % 251) as u8)
            .collect::<Vec<_>>();
        let file = workspace.create_file(ROOT, b"payload", 0o600).unwrap();
        for (index, chunk) in data.chunks(64 * 1024).enumerate() {
            workspace
                .write(file.node, (index * 64 * 1024) as u64, chunk)
                .unwrap();
        }
        workspace.fsync(Some(file.node)).unwrap();
        if let Data::File(FileData::Edited { pieces, .. }) =
            &mut workspace.nodes.get_mut(&file.node).unwrap().data
        {
            *pieces = crate::file_edit::PieceTree::empty()
                .replace(
                    0,
                    0,
                    [
                        crate::file_edit::Piece::Spool { offset: 0, len: 1 },
                        crate::file_edit::Piece::Spool {
                            offset: 1,
                            len: data.len() as u64 - 1,
                        },
                    ],
                )
                .unwrap();
            assert_eq!(pieces.count(), 2);
        }
        let captured = workspace
            .take_capture()
            .expect("sequential capture is reusable");
        let captured_root = captured.root;
        workspace.capture = crate::capture::CaptureState::Ready(Box::new(captured));

        let store = workspace.store.clone();
        let branch = store.branch(workspace.branch_id).unwrap().unwrap();
        let built = workspace.build_candidate().unwrap();
        let outcome = store
            .commit_candidate(
                &branch,
                workspace.base_root,
                workspace.expected_base,
                built.built,
            )
            .unwrap();
        let CommitOutcome::Committed { root_id, .. } = outcome else {
            panic!("capture Commit was not created")
        };
        let reader = store.snapshot_reader(root_id);
        let resolved = filesystem::resolve(
            &CoreReader(&reader),
            root_id,
            &CanonicalPath::new("payload").unwrap(),
            &mut LogicalCounters::default(),
        )
        .unwrap();
        assert_eq!(resolved.record.content_root, captured_root.0);
        let mut actual = Vec::new();
        filesystem::stream(
            &CoreReader(&reader),
            root_id,
            &CanonicalPath::new("payload").unwrap(),
            &mut actual,
        )
        .unwrap();
        assert_eq!(actual, data);

        drop(workspace);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn backward_write_invalidates_capture_and_uses_exact_fallback() {
        let (root, mut workspace) = empty_workspace("capture-fallback");
        let mut expected = vec![17; 128 * 1024];
        let file = workspace.create_file(ROOT, b"payload", 0o600).unwrap();
        workspace.write(file.node, 0, &expected).unwrap();
        workspace.write(file.node, 4096, b"backward10").unwrap();
        expected[4096..4106].copy_from_slice(b"backward10");
        assert!(matches!(
            workspace.capture,
            crate::capture::CaptureState::Invalid
        ));

        let store = workspace.store.clone();
        let branch = store.branch(workspace.branch_id).unwrap().unwrap();
        let built = workspace.build_candidate().unwrap();
        let outcome = store
            .commit_candidate(
                &branch,
                workspace.base_root,
                workspace.expected_base,
                built.built,
            )
            .unwrap();
        let CommitOutcome::Committed { root_id, .. } = outcome else {
            panic!("fallback Commit was not created")
        };
        let reader = store.snapshot_reader(root_id);
        let mut actual = Vec::new();
        filesystem::stream(
            &CoreReader(&reader),
            root_id,
            &CanonicalPath::new("payload").unwrap(),
            &mut actual,
        )
        .unwrap();
        assert_eq!(actual, expected);

        drop(workspace);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn existing_file_mutations_are_exact_and_bounded() {
        let base = (0..1024 * 1024)
            .map(|index| ((index * 31 + index / 7) % 251) as u8)
            .collect::<Vec<_>>();
        for case in ["overwrite", "noop", "append", "shrink", "grow", "rename"] {
            let root = std::env::temp_dir().join(format!(
                "layerfs-incremental-{case}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let source = root.join("source");
            std::fs::create_dir_all(&source).unwrap();
            std::fs::write(source.join("payload"), &base).unwrap();
            let store = LayerStackStore::create(root.join("store.sqlite")).unwrap();
            let layer = store
                .initialize_layerstack(
                    EntityName::new("project").unwrap(),
                    LayerStackInitialization::Directory(source),
                )
                .unwrap()
                .genesis_layer_id;
            let branch = store
                .fork_branch(
                    EntityName::new(case).unwrap(),
                    LocalForkSource::Layer { layer_id: layer },
                )
                .unwrap();
            let policy = if case == "overwrite" {
                crate::ResourcePolicy {
                    max_final_delta_memory_bytes: 1024,
                    ..crate::ResourcePolicy::default()
                }
            } else {
                crate::ResourcePolicy::default()
            };
            let mut workspace =
                Workspace::open_with_policy(store.clone(), branch, root.join("spool"), policy)
                    .unwrap();
            let file = workspace.lookup(ROOT, b"payload").unwrap().node;
            let original_inode = workspace.nodes[&file].canonical.unwrap();
            let mut expected = base.clone();
            let (expected_cdc, expected_path) = match case {
                "overwrite" => {
                    let offset = 543_219;
                    workspace.write(file, offset, b"changed-10").unwrap();
                    expected[offset as usize..offset as usize + 10].copy_from_slice(b"changed-10");
                    (10, "payload")
                }
                "noop" => {
                    let offset = 123_457;
                    workspace
                        .write(file, offset, &base[offset as usize..offset as usize + 10])
                        .unwrap();
                    (0, "payload")
                }
                "append" => {
                    workspace
                        .write(file, base.len() as u64, b"append-010")
                        .unwrap();
                    expected.extend_from_slice(b"append-010");
                    (10, "payload")
                }
                "shrink" => {
                    workspace.truncate(file, base.len() as u64 - 4096).unwrap();
                    expected.truncate(base.len() - 4096);
                    (0, "payload")
                }
                "grow" => {
                    workspace.truncate(file, base.len() as u64 + 4096).unwrap();
                    expected.resize(base.len() + 4096, 0);
                    (4096, "payload")
                }
                "rename" => {
                    workspace
                        .rename(ROOT, b"payload", ROOT, b"renamed", false)
                        .unwrap();
                    (0, "renamed")
                }
                _ => unreachable!(),
            };
            let built = workspace.build_candidate().unwrap();
            assert_eq!(
                built.built.counters.cdc_bytes_scanned, expected_cdc,
                "{case}"
            );
            assert!(built.built.objects.encoded_bytes() < 256 * 1024, "{case}");
            if case == "noop" {
                assert_eq!(built.built.root_id, workspace.base_root);
                assert!(built.built.objects.is_empty());
            }
            let candidate_root = built.built.root_id;
            let record = store.branch(branch).unwrap().unwrap();
            let outcome = store
                .commit_candidate(
                    &record,
                    workspace.base_root,
                    workspace.expected_base,
                    built.built,
                )
                .unwrap();
            assert_eq!(
                matches!(outcome, CommitOutcome::UpToDate { .. }),
                case == "noop"
            );
            let reader = store.snapshot_reader(candidate_root);
            let mut actual = Vec::new();
            let resolved = filesystem::resolve(
                &CoreReader(&reader),
                candidate_root,
                &CanonicalPath::new(expected_path).unwrap(),
                &mut LogicalCounters::default(),
            )
            .unwrap();
            if case == "rename" {
                assert_eq!(resolved.inode, original_inode);
                assert!(filesystem::resolve(
                    &CoreReader(&reader),
                    candidate_root,
                    &CanonicalPath::new("payload").unwrap(),
                    &mut LogicalCounters::default(),
                )
                .is_err());
            }
            filesystem::stream(
                &CoreReader(&reader),
                candidate_root,
                &CanonicalPath::new(expected_path).unwrap(),
                &mut actual,
            )
            .unwrap();
            assert_eq!(actual, expected, "{case}");

            drop(workspace);
            drop(store);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn owner_prepend_scans_only_replacement_and_retains_every_base_payload() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-owner-prepend-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = root.join("source");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("payload"), vec![0x5a; 1024 * 1024]).unwrap();
        let store = LayerStackStore::create(root.join("store.sqlite")).unwrap();
        let layer = store
            .initialize_layerstack(
                EntityName::new("project").unwrap(),
                LayerStackInitialization::Directory(source),
            )
            .unwrap()
            .genesis_layer_id;
        let branch = store
            .fork_branch(
                EntityName::new("main").unwrap(),
                LocalForkSource::Layer { layer_id: layer },
            )
            .unwrap();
        let mut workspace = Workspace::open(store.clone(), branch, root.join("spool")).unwrap();
        let file = workspace.lookup(ROOT, b"payload").unwrap().node;
        let base_root = match workspace.nodes[&file].data {
            Data::File(FileData::Base { root, .. }) => root,
            _ => panic!("base file"),
        };
        let mut base_payloads = BTreeSet::new();
        rope::visit_extents(&CoreReader(&workspace.reader), base_root, |extents| {
            base_payloads.extend(extents.iter().map(|extent| extent.payload_object_id));
            Ok(())
        })
        .unwrap();
        workspace
            .edit_many(
                file,
                vec![(
                    0,
                    0,
                    crate::WorkspaceFileReplacement::Inline(b"PREPEND010".to_vec()),
                )],
            )
            .unwrap();
        let reads_before = workspace.reader.read_metrics_snapshot().unwrap();
        let built = workspace.build_candidate().unwrap();
        let reads_after = workspace.reader.read_metrics_snapshot().unwrap();
        assert_eq!(built.built.counters.cdc_bytes_scanned, 10);
        assert_eq!(
            reads_after.payload_bytes_read - reads_before.payload_bytes_read,
            0
        );
        let record = store.branch(branch).unwrap().unwrap();
        let outcome = store
            .commit_candidate(
                &record,
                workspace.base_root,
                workspace.expected_base,
                built.built,
            )
            .unwrap();
        let CommitOutcome::Committed { root_id, .. } = outcome else {
            panic!("prepend commit")
        };
        let reader = store.snapshot_reader(root_id);
        let resolved = filesystem::resolve(
            &CoreReader(&reader),
            root_id,
            &CanonicalPath::new("payload").unwrap(),
            &mut LogicalCounters::default(),
        )
        .unwrap();
        let mut final_payloads = BTreeSet::new();
        rope::visit_extents(
            &CoreReader(&reader),
            FileStateRoot(resolved.record.content_root),
            |extents| {
                final_payloads.extend(extents.iter().map(|extent| extent.payload_object_id));
                Ok(())
            },
        )
        .unwrap();
        assert!(base_payloads.is_subset(&final_payloads));
        drop(workspace);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }
}

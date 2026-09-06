use crate::cow_tree::{portable_metadata, Attr, Data, FileData, Kind, NodeId, Workspace, ROOT};
use layerfs_content::file::rope::{
    self, FileMutationBatch, FileStateRoot, ObjectStore, RopeCounters,
};
use layerfs_content::filesystem::{self, InodeMutation, LogicalCounters, PortableMetadataCache};
use layerfs_content::object::access::ObjectRead;
use layerfs_content::object::{ContentDigestWriter, ObjectId};
use layerfs_content::tree::batch::{
    directory_apply_sorted_observed, inode_table_apply_sorted_with_budget,
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
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
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

fn journal_io_bytes(budget: u64) -> usize {
    (budget.saturating_sub(1024) / 64).clamp(256, 64 * 1024) as usize
}

// Original/final edge facts from the existing sorted directory leaf merge.
// New inode reference counts are already initialized from their final bindings.
struct ReferenceJournal<'a> {
    directory: &'a std::path::Path,
    io_bytes: usize,
    writer: Option<BufWriter<File>>,
    count: u64,
}

impl<'a> ReferenceJournal<'a> {
    fn new(directory: &'a std::path::Path, io_bytes: usize) -> Self {
        Self {
            directory,
            io_bytes,
            writer: None,
            count: 0,
        }
    }

    fn push(&mut self, before: Option<InodeId>, after: Option<InodeId>) -> Result<()> {
        if before == after {
            return Ok(());
        }
        if self.writer.is_none() {
            self.writer = Some(BufWriter::with_capacity(
                self.io_bytes,
                anonymous_journal(self.directory)?,
            ));
        }
        let mut row = [0; 65];
        if let Some(inode) = before {
            row[0] |= 1;
            row[1..33].copy_from_slice(inode.as_bytes());
        }
        if let Some(inode) = after {
            row[0] |= 2;
            row[33..65].copy_from_slice(inode.as_bytes());
        }
        self.writer.as_mut().unwrap().write_all(&row)?;
        self.count = self
            .count
            .checked_add(1)
            .ok_or(StorageError::Integrity("reference journal count"))?;
        Ok(())
    }

    fn rewind(&mut self, count: u64) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            writer.flush()?;
            let end = count
                .checked_mul(65)
                .ok_or(StorageError::Integrity("reference journal length"))?;
            writer.get_ref().set_len(end)?;
            writer.seek(SeekFrom::Start(end))?;
        }
        self.count = count;
        Ok(())
    }

    fn finish(self) -> Result<Option<(File, u64)>> {
        self.writer
            .map(|mut writer| {
                writer.flush()?;
                Ok((
                    writer.into_inner().map_err(|error| error.into_error())?,
                    self.count,
                ))
            })
            .transpose()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CandidatePurpose {
    Commit,
    Preview,
}

// Mutable identities stay with Workspace; Store receives only the canonical candidate.
pub(crate) struct PreparedCommit {
    pub(crate) built: BuiltRoot,
    pub(crate) checkpoint: Checkpoint,
    pub(crate) admission: Option<layerfs_layerstack_store::WorkspaceAdmission>,
}

pub(crate) struct Checkpoint {
    pub(crate) root: ObjectId,
    pub(crate) generation: u64,
    file: File,
    count: u64,
    io_bytes: usize,
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
            writer: BufWriter::with_capacity(
                journal_io_bytes(workspace.policy.max_final_delta_memory_bytes),
                file,
            ),
            count: 0,
            // Metadata facts are candidate scratch, not payload spool bytes.
            byte_limit: (workspace.live.nodes.len() as u64).saturating_mul(104),
        })
    }

    fn push(
        &mut self,
        node: NodeId,
        inode: InodeId,
        content: ObjectId,
        attr: Attr,
        checked_file_len: Option<u64>,
    ) -> Result<()> {
        if attr.kind == Kind::File && checked_file_len != Some(attr.size) {
            return Err(StorageError::Integrity("checkpoint file length"));
        }
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

    #[cfg(test)]
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
            self.writer.capacity(),
            |_, inode, content, attr| {
                let final_record = record(inode)?;
                Self::validate_record(objects, metadata_cache, final_record, content, attr)
            },
        )
    }

    fn validate_record(
        objects: &ObjectBuffer<'_>,
        metadata_cache: &PortableMetadataCache,
        final_record: InodeRecordV1,
        content: ObjectId,
        attr: Attr,
    ) -> Result<()> {
        let metadata =
            match metadata_cache.get_by_root(final_record.kind, final_record.metadata_root) {
                Some(metadata) => metadata,
                None => portable_metadata(objects, final_record.metadata_root, final_record.kind)?,
            };
        let size = match final_record.kind {
            InodeKind::RegularFile => attr.size, // checked at the completed-file handoff
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
    }

    fn finish(mut self, built: BuiltRoot, generation: u64) -> Result<PreparedCommit> {
        self.writer.flush()?;
        let io_bytes = self.writer.capacity();
        let file = self
            .writer
            .into_inner()
            .map_err(|error| error.into_error())?;
        Ok(PreparedCommit {
            admission: None,
            checkpoint: Checkpoint {
                root: built.root_id,
                generation,
                file,
                count: self.count,
                io_bytes,
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
        Self::visit_file(&self.file, self.count, self.io_bytes, &mut visitor)
    }

    fn visit_file(
        file: &File,
        count: u64,
        io_bytes: usize,
        mut visitor: impl FnMut(NodeId, InodeId, ObjectId, Attr) -> Result<()>,
    ) -> Result<()> {
        let mut reader = BufReader::with_capacity(io_bytes, file);
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
    pub(crate) fn build_candidate(&mut self, purpose: CandidatePurpose) -> Result<PreparedCommit> {
        #[cfg(any(debug_assertions, feature = "test-instrumentation"))]
        if INJECT_CANDIDATE_FAILURE.with(|inject| inject.replace(false)) {
            return Err(StorageError::Integrity(
                "injected Workspace candidate failure",
            ));
        }
        self.build_frontier_candidate(purpose)
    }

    // Directory overlays already are the final binding delta. Applying their inode
    // edges directly preserves untouched subtrees, including a renamed directory,
    // without building either complete namespace manifest.
    fn build_frontier_candidate(&mut self, purpose: CandidatePurpose) -> Result<PreparedCommit> {
        self.build_frontier_candidate_with_workers(purpose, 1)
    }

    fn build_frontier_candidate_with_workers(
        &mut self,
        purpose: CandidatePurpose,
        worker_limit: usize,
    ) -> Result<PreparedCommit> {
        let started = Instant::now();
        self.policy
            .check_final_delta(1024)
            .map_err(crate::live_error)?;
        let batch_size = (self.policy.max_final_delta_memory_bytes / 4096).clamp(1, 128) as usize;
        let io_bytes = journal_io_bytes(self.policy.max_final_delta_memory_bytes);
        // Task-index and all worker-result buffers split one existing journal allowance.
        let frontier_budget = self
            .policy
            .max_final_delta_memory_bytes
            .saturating_sub(4 * io_bytes.saturating_sub(256) as u64);
        let tree_scratch = usize::try_from(frontier_budget.saturating_sub(1024) / 2)
            .unwrap_or(usize::MAX)
            .min(SORTED_TREE_UPDATE_SCRATCH_BYTES);
        let captured = self.take_capture();
        if let Some(captured) = &captured {
            layerfs_layerstack_store::note_workspace_capture(1, captured.len);
        }
        let inputs = StableFileInputs {
            nodes: &self.live.nodes,
            dirty: &self.live.dirty,
            reader: self.reader.clone(),
            base_inodes: self.base_inodes,
            generation: self.live.mutation_generation,
            spool: &self.spool,
            io_bytes: io_bytes / 2,
            captured: std::sync::Mutex::new(captured),
        };
        note_commit_phase(WorkspaceCommitPhase::CandidatePlan, started);
        let content_started = Instant::now();
        let mut objects = ObjectBuffer::bounded_output(Some(&self.reader))?;
        let plan = inputs.prepare()?;
        let workers = worker_limit.min(plan.count).min(io_bytes / 256).min(8);
        if workers == 0 && plan.count != 0 {
            return Err(StorageError::InvalidInput("file producer budget"));
        }
        let tasks = plan.tasks(io_bytes / 2)?;
        let initialize = |worker| inputs.worker(worker, workers.max(1));
        let step =
            |worker: &mut FileResultWriter,
             ordinal,
             task: Result<NodeId>,
             writer: &mut layerfs_layerstack_store::FinalizedOutputWriter| {
                inputs.produce_file(worker, &plan.file, ordinal, task?, &mut |selected| {
                    writer.send_selected(selected)
                })
            };
        let finish = |worker: FileResultWriter| worker.finish();
        let (workers, admission) = match purpose {
            CandidatePurpose::Commit => {
                let (workers, admission) = self.store.construct_workspace_files(
                    self.workspace_id,
                    workers,
                    plan.count,
                    tasks,
                    initialize,
                    step,
                    finish,
                )?;
                (workers, Some(admission))
            }
            CandidatePurpose::Preview => (
                objects.construct_files(workers, plan.count, tasks, initialize, step, finish)?,
                None,
            ),
        };
        let mut files = FileResults::new(plan, workers, io_bytes / 2)?;
        drop(inputs);
        note_commit_phase(WorkspaceCommitPhase::Content, content_started);
        let started = Instant::now();
        let mut inodes = FrontierInodes::new(
            self.base_root,
            // The fixed 1 KiB allowance includes the first 256-byte map entry.
            1 + (frontier_budget.saturating_sub(1024) / 512) as usize,
            tree_scratch,
            &self.spool,
        );
        let mut metadata_cache = PortableMetadataCache::default();
        let mut references = ReferenceJournal::new(&self.spool, io_bytes / 2);
        note_commit_phase(WorkspaceCommitPhase::CandidatePlan, started);
        let started = Instant::now();
        for &node in &self.live.dirty {
            let value = self
                .live
                .nodes
                .get(&node)
                .ok_or(StorageError::Integrity("dirty node"))?;
            layerfs_layerstack_store::note_workspace_namespace_visits(0, 0, 0, 0, 1);
            if value.paths.is_empty() && value.links == 0 {
                continue;
            }
            let inode = self.frontier_inode(node)?;
            let file_result = if matches!(value.data, Data::File(_)) {
                Some(files.next(node, self.live.mutation_generation, self.attr(node)?.size)?)
            } else {
                None
            };
            let before = if let Some((_, before)) = file_result {
                before
            } else {
                match value.canonical {
                    Some(_) => Some(inodes.record(&objects, inode)?),
                    None => None,
                }
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
                        &mut references,
                        !value.paths.is_empty(),
                    )?;
                    content.0
                }
                Data::Symlink(target) => match before {
                    Some(record) => record.content_root,
                    None => filesystem::symlink_content(&mut objects, target.clone())?,
                },
                Data::File(_) => file_result.expect("prepared file result").0,
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
            inodes.set_checkpoint(inode, record, node, content_root)?;
        }
        files.finish_read()?;
        note_commit_phase(WorkspaceCommitPhase::Content, started);
        let started = Instant::now();
        let file_counters = files.counters;
        drop(files);
        inodes.apply_references(
            &objects,
            &CoreReader(&self.reader),
            references.finish()?,
            frontier_budget,
            io_bytes / 2,
        )?;
        let mut checkpoint = CheckpointJournal::new(self)?;
        inodes.finish(&mut objects, |objects, inode, node, content, record| {
            let attr = self.attr(node)?;
            CheckpointJournal::validate_record(objects, &metadata_cache, record, content, attr)?;
            checkpoint.push(
                node,
                inode,
                content,
                attr,
                (attr.kind == Kind::File).then_some(attr.size),
            )
        })?;
        note_commit_phase(WorkspaceCommitPhase::Namespace, started);
        let started = Instant::now();
        let mut built = objects.finish(inodes.root, 0)?;
        add_build_counters(&mut built.counters, file_counters);
        let built = checkpoint
            .finish(built, self.live.mutation_generation)
            .map(|mut prepared| {
                prepared.admission = admission;
                prepared
            });
        note_commit_phase(WorkspaceCommitPhase::CandidateFinish, started);
        built
    }

    // Keep the existing frontier inputs explicit without adding a wrapper type.
    #[allow(clippy::too_many_arguments)]
    fn apply_frontier_directory(
        &self,
        objects: &mut ObjectBuffer<'_>,
        root: DirectoryStateRoot,
        changes: &BTreeMap<Vec<u8>, Option<NodeId>>,
        batch_size: usize,
        scratch_limit: usize,
        references: &mut ReferenceJournal<'_>,
        record_edges: bool,
    ) -> Result<DirectoryStateRoot> {
        let checkpoint = references.count;
        let mut edge_error = None;
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
        let sorted = directory_apply_sorted_observed(
            objects,
            root,
            deltas,
            scratch_limit,
            |before, after| {
                if !record_edges {
                    return Ok(());
                }
                layerfs_layerstack_store::note_workspace_namespace_visits(
                    u64::from(before.is_some()),
                    u64::from(after.is_some()),
                    0,
                    0,
                    0,
                );
                // Only existing inodes need an added reference; new records already
                // carry every final alias. Original bindings always belong to base.
                let after = after.filter(|inode| self.canonical_nodes.contains_key(inode));
                references.push(before, after).map_err(|error| {
                    edge_error = Some(error);
                    layerfs_content::CoreError::Io
                })
            },
        );
        if let Some(error) = edge_error {
            return Err(error);
        }
        if let Some(error) = source_error {
            return Err(error);
        }
        match sorted {
            Ok((root, _)) => Ok(root),
            Err(
                layerfs_content::CoreError::ObjectLimitExceeded
                | layerfs_content::CoreError::Unsupported,
            ) => {
                references.rewind(checkpoint)?;
                let original = root;
                let mut root = root;
                let mut batch = Vec::with_capacity(batch_size);
                for (name, desired) in changes {
                    let name = CanonicalName::from_bytes(name)?;
                    let after = desired
                        .map(|child| self.frontier_inode(child))
                        .transpose()?;
                    if record_edges {
                        let before = directory_lookup(
                            objects,
                            original,
                            &name,
                            &mut NamespaceCounters::default(),
                        )?;
                        references.push(
                            before,
                            after.filter(|inode| self.canonical_nodes.contains_key(inode)),
                        )?;
                    }
                    batch.push((name, after));
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
            .live
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
            .check_final_delta(batch_allowance.saturating_add(path_charge(path)))
            .map_err(crate::live_error)?;
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
                    .live
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
                    self.policy
                        .check_final_delta(charge)
                        .map_err(crate::live_error)?;
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
                let dirty = self.live.dirty.contains(&node);
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
                self.policy
                    .check_final_delta(charge)
                    .map_err(crate::live_error)?;
                if attr.kind == Kind::Directory {
                    pending.push((node, path));
                }
            }
        }
        Ok(output)
    }
}

struct StableFileInputs<'a> {
    nodes: &'a std::collections::HashMap<NodeId, crate::cow_tree::Node>,
    dirty: &'a BTreeSet<NodeId>,
    reader: layerfs_layerstack_store::SnapshotReader,
    base_inodes: InodeTableRoot,
    generation: u64,
    spool: &'a std::path::Path,
    io_bytes: usize,
    captured: std::sync::Mutex<Option<crate::capture::CapturedFile>>,
}

// Fixed task slots restore ordinal order without an in-memory completion map.
// Each slot holds NodeId, worker+1, journal offset and encoded result length.
const FILE_TASK_BYTES: u64 = 32;
struct FileTaskPlan {
    file: File,
    count: usize,
    generation: u64,
}
struct FileTasks {
    reader: BufReader<File>,
    remaining: usize,
}
impl Iterator for FileTasks {
    type Item = Result<NodeId>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let mut slot = [0; FILE_TASK_BYTES as usize];
        Some(
            self.reader
                .read_exact(&mut slot)
                .map(|()| NodeId(u64::from_le_bytes(slot[..8].try_into().unwrap())))
                .map_err(Into::into),
        )
    }
}
impl FileTaskPlan {
    fn tasks(&self, io_bytes: usize) -> Result<FileTasks> {
        let mut file = self.file.try_clone()?;
        file.seek(SeekFrom::Start(0))?;
        Ok(FileTasks {
            reader: BufReader::with_capacity(io_bytes, file),
            remaining: self.count,
        })
    }
}
struct FileResultWriter {
    worker: usize,
    partitions: usize,
    journal: BufWriter<File>,
    offset: u64,
    count: u64,
    counters: layerfs_layerstack_store::BuildCounters,
}
struct WorkerFileResults {
    worker: usize,
    file: File,
    io_bytes: usize,
    count: u64,
    counters: layerfs_layerstack_store::BuildCounters,
}
impl FileResultWriter {
    fn finish(mut self) -> Result<WorkerFileResults> {
        self.journal.flush()?;
        let io_bytes = self.journal.capacity();
        let mut file = self
            .journal
            .into_inner()
            .map_err(|error| error.into_error())?;
        file.seek(SeekFrom::Start(0))?;
        Ok(WorkerFileResults {
            worker: self.worker,
            file,
            io_bytes,
            count: self.count,
            counters: self.counters,
        })
    }
}
struct FileResults {
    index: BufReader<File>,
    files: Vec<BufReader<File>>,
    offsets: Vec<u64>,
    remaining: u64,
    generation: u64,
    counters: layerfs_layerstack_store::BuildCounters,
}

fn add_build_counters(
    total: &mut layerfs_layerstack_store::BuildCounters,
    next: layerfs_layerstack_store::BuildCounters,
) {
    total.cdc_bytes_scanned = total
        .cdc_bytes_scanned
        .saturating_add(next.cdc_bytes_scanned);
    total.encode_hash_invocations = total
        .encode_hash_invocations
        .saturating_add(next.encode_hash_invocations);
    total.first_store_write_bytes = total
        .first_store_write_bytes
        .saturating_add(next.first_store_write_bytes);
    total.reachable_copy_write_bytes = total
        .reachable_copy_write_bytes
        .saturating_add(next.reachable_copy_write_bytes);
    total.spill_peak_bytes = total.spill_peak_bytes.max(next.spill_peak_bytes);
    total.spill_count = total.spill_count.saturating_add(next.spill_count);
}

impl StableFileInputs<'_> {
    fn prepare(&self) -> Result<FileTaskPlan> {
        let mut writer = BufWriter::with_capacity(self.io_bytes, anonymous_journal(self.spool)?);
        let mut count = 0_usize;
        for &id in self.dirty {
            let node = self
                .nodes
                .get(&id)
                .ok_or(StorageError::Integrity("frozen file node"))?;
            if !matches!(node.data, Data::File(_)) || (node.paths.is_empty() && node.links == 0) {
                continue;
            }
            let mut slot = [0; FILE_TASK_BYTES as usize];
            slot[..8].copy_from_slice(&id.0.to_le_bytes());
            writer.write_all(&slot)?;
            count = count
                .checked_add(1)
                .ok_or(StorageError::Integrity("file task count"))?;
        }
        writer.flush()?;
        Ok(FileTaskPlan {
            file: writer.into_inner().map_err(|error| error.into_error())?,
            count,
            generation: self.generation,
        })
    }

    fn worker(&self, worker: usize, workers: usize) -> Result<FileResultWriter> {
        Ok(FileResultWriter {
            worker,
            partitions: workers,
            journal: BufWriter::with_capacity(
                self.io_bytes / workers,
                anonymous_journal(self.spool)?,
            ),
            offset: 0,
            count: 0,
            counters: Default::default(),
        })
    }

    fn produce_file(
        &self,
        worker: &mut FileResultWriter,
        index: &File,
        ordinal: usize,
        id: NodeId,
        emit: &mut dyn FnMut(layerfs_layerstack_store::DeferredObjectStore) -> Result<()>,
    ) -> Result<()> {
        let node = self
            .nodes
            .get(&id)
            .ok_or(StorageError::Integrity("frozen file node"))?;
        let input = FrozenFile::from_node(&self.reader, node)?;
        let before = node
            .canonical
            .map(|inode| -> Result<_> {
                let core = CoreReader(&self.reader);
                let record = inode_table_lookup(
                    &core,
                    self.base_inodes,
                    inode,
                    &mut InodeTableCounters::default(),
                )?
                .ok_or(StorageError::Integrity("frozen file inode"))?;
                Ok(core.with_authenticated_canonical(record, decode_inode_record)?)
            })
            .transpose()?;
        let captured = {
            let mut captured = self
                .captured
                .lock()
                .map_err(|_| StorageError::Integrity("captured file input"))?;
            if captured
                .as_ref()
                .is_some_and(|captured| captured.node == id)
            {
                captured.take()
            } else {
                None
            }
        };
        let built = input.build(before, captured, worker.partitions)?;
        let root = built.root_id;
        add_build_counters(&mut worker.counters, built.counters);
        emit(built.objects)?;
        let before = before.map(encode_inode_record).transpose()?;
        let mut record = Vec::with_capacity(308);
        record.extend_from_slice(&id.0.to_le_bytes());
        record.extend_from_slice(&input.len.to_le_bytes());
        record.extend_from_slice(root.as_bytes());
        record.extend_from_slice(&(before.as_ref().map_or(0, Vec::len) as u32).to_le_bytes());
        if let Some(before) = before {
            record.extend_from_slice(&before);
        }
        worker.journal.write_all(&record)?;
        let mut location = [0; 24];
        location[..8].copy_from_slice(&(worker.worker as u64 + 1).to_le_bytes());
        location[8..16].copy_from_slice(&worker.offset.to_le_bytes());
        location[16..].copy_from_slice(&(record.len() as u64).to_le_bytes());
        let offset = (ordinal as u64)
            .checked_mul(FILE_TASK_BYTES)
            .and_then(|v| v.checked_add(8))
            .ok_or(StorageError::Integrity("file task offset"))?;
        index.write_all_at(&location, offset)?;
        worker.offset = worker
            .offset
            .checked_add(record.len() as u64)
            .ok_or(StorageError::Integrity("file result offset"))?;
        worker.count += 1;
        Ok(())
    }
}

impl FileResults {
    fn new(
        mut plan: FileTaskPlan,
        workers: Vec<WorkerFileResults>,
        io_bytes: usize,
    ) -> Result<Self> {
        let mut counters = layerfs_layerstack_store::BuildCounters::default();
        let mut count = 0_u64;
        let mut files = Vec::with_capacity(workers.len());
        for worker in workers {
            if worker.worker != files.len() {
                return Err(StorageError::Integrity("file result worker order"));
            }
            count += worker.count;
            // Sum worker-local spill peaks as an upper bound on concurrent spill.
            let peak = counters
                .spill_peak_bytes
                .saturating_add(worker.counters.spill_peak_bytes);
            add_build_counters(&mut counters, worker.counters);
            counters.spill_peak_bytes = peak;
            files.push(BufReader::with_capacity(worker.io_bytes, worker.file));
        }
        if count != plan.count as u64 {
            return Err(StorageError::Integrity("file result task coverage"));
        }
        plan.file.seek(SeekFrom::Start(0))?;
        Ok(Self {
            index: BufReader::with_capacity(io_bytes, plan.file),
            offsets: vec![0; files.len()],
            files,
            remaining: count,
            generation: plan.generation,
            counters,
        })
    }

    fn next(
        &mut self,
        node: NodeId,
        generation: u64,
        len: u64,
    ) -> Result<(ObjectId, Option<InodeRecordV1>)> {
        if generation != self.generation || self.remaining == 0 {
            return Err(StorageError::Integrity("file result generation"));
        }
        let mut slot = [0; FILE_TASK_BYTES as usize];
        self.index.read_exact(&mut slot)?;
        let worker = u64::from_le_bytes(slot[8..16].try_into().unwrap())
            .checked_sub(1)
            .and_then(|worker| usize::try_from(worker).ok())
            .ok_or(StorageError::Integrity("missing file result"))?;
        let offset = u64::from_le_bytes(slot[16..24].try_into().unwrap());
        let encoded_len = u64::from_le_bytes(slot[24..].try_into().unwrap());
        if u64::from_le_bytes(slot[..8].try_into().unwrap()) != node.0
            || self.offsets.get(worker) != Some(&offset)
            || !(52..=308).contains(&encoded_len)
        {
            return Err(StorageError::Integrity("file result task identity"));
        }
        let file = &mut self.files[worker];
        let mut header = [0; 52];
        file.read_exact(&mut header)?;
        if u64::from_le_bytes(header[..8].try_into().unwrap()) != node.0
            || u64::from_le_bytes(header[8..16].try_into().unwrap()) != len
        {
            return Err(StorageError::Integrity("file result identity"));
        }
        let root = ObjectId::from_bytes(&header[16..48])?;
        let size = u32::from_le_bytes(header[48..52].try_into().unwrap()) as usize;
        let mut record = [0; 256];
        if size > record.len() || encoded_len != 52 + size as u64 {
            return Err(StorageError::Integrity("file result record"));
        }
        file.read_exact(&mut record[..size])?;
        self.offsets[worker] += encoded_len;
        self.remaining -= 1;
        Ok((
            root,
            if size == 0 {
                None
            } else {
                Some(decode_inode_record(&record[..size])?)
            },
        ))
    }
    fn finish_read(&mut self) -> Result<()> {
        if self.remaining != 0 || self.index.read(&mut [0; 1])? != 0 {
            return Err(StorageError::Integrity("file result coverage"));
        }
        for file in &mut self.files {
            if file.read(&mut [0; 1])? != 0 {
                return Err(StorageError::Integrity("file result journal coverage"));
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
struct FrozenFile {
    reader: layerfs_layerstack_store::SnapshotReader,
    data: FileData,
    len: u64,
}

impl FrozenFile {
    fn from_node(
        reader: &layerfs_layerstack_store::SnapshotReader,
        node: &crate::cow_tree::Node,
    ) -> Result<Self> {
        let Data::File(data) = &node.data else {
            return Err(StorageError::InvalidInput("file input"));
        };
        let len = match data {
            FileData::Base { len, .. } => *len,
            FileData::Edited { pieces, .. } => pieces.len(),
        };
        Ok(Self {
            reader: reader.clone(),
            data: data.clone(),
            len,
        })
    }

    fn read(&self, offset: u64, size: usize) -> Result<Vec<u8>> {
        crate::file_io::ReadPlan::for_file(self.reader.clone(), &self.data, offset, size)?.read()
    }

    fn reader(&self) -> WorkspaceFileReader {
        let source = match &self.data {
            FileData::Edited {
                base: None, pieces, ..
            } if pieces.compact_spool().is_some() => {
                let slice = pieces.compact_spool().unwrap();
                WorkspaceFileSource::Direct(slice.segment.clone(), slice.offset)
            }
            _ => WorkspaceFileSource::Mixed(self.clone()),
        };
        WorkspaceFileReader {
            source,
            offset: 0,
            len: self.len,
        }
    }
    fn build(
        &self,
        before: Option<InodeRecordV1>,
        captured: Option<crate::capture::CapturedFile>,
        partitions: usize,
    ) -> Result<BuiltRoot> {
        if before.is_none() && captured.is_none() {
            return ObjectBuffer::build_complete_file_partition(
                self.reader(),
                self.len,
                partitions,
            );
        }
        let (mut objects, captured_root) = match captured {
            Some(captured) => {
                if captured.len != self.len {
                    return Err(StorageError::Integrity("captured file length"));
                }
                (
                    ObjectBuffer::resume_prevalidated(&self.reader, captured.objects),
                    Some((captured.root, captured.counters)),
                )
            }
            None => (ObjectBuffer::bounded_output(Some(&self.reader))?, None),
        };
        objects.partition_output(partitions)?;
        let (root, counters) = if let Some(captured) = captured_root {
            captured
        } else if let Some(record) = before {
            if !self.file_may_differ(record.content_root)? {
                (FileStateRoot(record.content_root), RopeCounters::default())
            } else {
                match self.mutate_existing_file(&mut objects, BaseEntry { record })? {
                    Some(changed) => changed,
                    None if self.incremental_file_supported(record.content_root) => {
                        (FileStateRoot(record.content_root), RopeCounters::default())
                    }
                    None => {
                        return ObjectBuffer::build_complete_file_partition(
                            self.reader(),
                            self.len,
                            partitions,
                        )
                    }
                }
            }
        } else {
            return ObjectBuffer::build_complete_file_partition(
                self.reader(),
                self.len,
                partitions,
            );
        };
        if rope::state(&objects, root, &mut RopeCounters::default())?.logical_len != self.len {
            return Err(StorageError::Integrity("completed file length"));
        }
        objects.finish(root.0, counters.cdc_bytes_scanned)
    }

    fn file_may_differ(&self, base: ObjectId) -> Result<bool> {
        match &self.data {
            FileData::Base { root, .. } if root.0 == base => Ok(false),
            FileData::Edited {
                base: Some((root, base_len)),
                pieces,
                ..
            } if root.0 == base => Ok(pieces.len() != *base_len
                || !matches!(pieces.pieces().as_slice(), [crate::file_edit::Piece::Base { root: piece_root, offset: 0, len }] if *piece_root == *root && *len == *base_len)),
            _ => Ok(!self.file_matches(base)?),
        }
    }

    fn incremental_file_supported(&self, base: ObjectId) -> bool {
        matches!(
            &self.data,
            FileData::Edited {
                base: Some((root, _)),
                ..
            } if root.0 == base
        )
    }

    fn mutate_existing_file(
        &self,
        objects: &mut ObjectBuffer<'_>,
        base: BaseEntry,
    ) -> Result<Option<(FileStateRoot, RopeCounters)>> {
        let FileData::Edited {
            base: Some((file_root, _)),
            pieces,
            ..
        } = &self.data
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
                                file_root,
                                final_cursor,
                                final_cursor + replacement_len,
                            )?)
                    {
                        batch.replace(
                            final_cursor,
                            delete_len,
                            WorkspaceRangeReader::new(self, final_cursor, replacement_len)?,
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
                    file_root,
                    final_cursor,
                    final_cursor + replacement_len,
                )?)
        {
            batch.replace(
                final_cursor,
                delete_len,
                WorkspaceRangeReader::new(self, final_cursor, replacement_len)?,
            )?;
            changed = true;
        }
        if batch.logical_len()? != self.len {
            return Err(StorageError::Integrity("Workspace file mutation length"));
        }
        if !changed {
            return Ok(None);
        }
        Ok(Some(batch.finish()?))
    }

    fn workspace_range_matches_base(
        &self,
        base: FileStateRoot,
        start: u64,
        end: u64,
    ) -> Result<bool> {
        let mut offset = start;
        while offset < end {
            let count = (end - offset).min(64 * 1024) as usize;
            let final_bytes = self.read(offset, count)?;
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

    fn file_matches(&self, base: ObjectId) -> Result<bool> {
        match &self.data {
            FileData::Base { root, .. } if root.0 == base => return Ok(true),
            FileData::Edited {
                base: Some((root, base_len)),
                pieces,
                ..
            } if root.0 == base
                && pieces.len() == *base_len
                && matches!(pieces.pieces().as_slice(), [crate::file_edit::Piece::Base { root: piece_root, offset: 0, len }] if *piece_root == *root && *len == *base_len) =>
            {
                return Ok(true)
            }
            _ => {}
        }
        let mut final_digest = ContentDigestWriter::new();
        let mut input = self.reader();
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

#[derive(Clone, Copy)]
struct FrontierValue {
    record: Option<InodeRecordV1>,
    checkpoint: Option<(NodeId, ObjectId)>,
}

// Final inode changes are coalesced before touching the immutable base table.
// The bounded map spills ordered fixed records; tombstones never fall through to base.
struct FrontierInodes {
    root: ObjectId,
    pending: BTreeMap<InodeId, FrontierValue>,
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

    fn read(file: &File, index: u64) -> Result<(InodeId, FrontierValue)> {
        let mut bytes = [0; 192];
        file.read_exact_at(&mut bytes, index * 192)?;
        Self::decode(&bytes)
    }

    fn decode(bytes: &[u8; 192]) -> Result<(InodeId, FrontierValue)> {
        Ok((
            InodeId(bytes[..32].try_into().unwrap()),
            FrontierValue {
                record: if bytes[32] == 0 {
                    None
                } else {
                    Some(InodeRecordV1 {
                        kind: InodeKind::try_from(bytes[32])?,
                        namespace_ref_count: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
                        content_root: ObjectId::from_bytes(&bytes[48..80])?,
                        metadata_root: ObjectId::from_bytes(&bytes[80..112])?,
                    })
                },
                checkpoint: if bytes[112] == 0 {
                    None
                } else {
                    Some((
                        NodeId(u64::from_le_bytes(bytes[120..128].try_into().unwrap())),
                        ObjectId::from_bytes(&bytes[128..160])?,
                    ))
                },
            },
        ))
    }

    fn write(file: &File, index: u64, inode: InodeId, value: FrontierValue) -> Result<()> {
        file.write_all_at(&Self::encode(inode, value), index * 192)?;
        Ok(())
    }

    fn encode(inode: InodeId, value: FrontierValue) -> [u8; 192] {
        let mut bytes = [0; 192];
        bytes[..32].copy_from_slice(inode.as_bytes());
        if let Some(record) = value.record {
            bytes[32] = record.kind as u8;
            bytes[40..48].copy_from_slice(&record.namespace_ref_count.to_le_bytes());
            bytes[48..80].copy_from_slice(record.content_root.as_bytes());
            bytes[80..112].copy_from_slice(record.metadata_root.as_bytes());
        }
        if let Some((node, content)) = value.checkpoint {
            bytes[112] = 1;
            bytes[120..128].copy_from_slice(&node.0.to_le_bytes());
            bytes[128..160].copy_from_slice(content.as_bytes());
        }
        bytes
    }

    fn io_bytes(&self) -> usize {
        // The unused half of each pending-entry reservation funds run I/O.
        self.batch_size
            .saturating_sub(1)
            .saturating_mul(128)
            .clamp(128, 64 * 1024)
    }

    fn spilled(&self, inode: InodeId) -> Result<Option<(u64, FrontierValue)>> {
        let Some(file) = &self.spill else {
            return Ok(None);
        };
        let (mut start, mut end) = (0, self.count);
        while start < end {
            let middle = start + (end - start) / 2;
            let mut key = [0; 32];
            file.read_exact_at(&mut key, middle * 192)?;
            match InodeId(key).cmp(&inode) {
                std::cmp::Ordering::Equal => {
                    return Ok(Some((middle, Self::read(file, middle)?.1)))
                }
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
            return record
                .record
                .ok_or(StorageError::Integrity("released frontier inode"));
        }
        if let Some((_, record)) = self.spilled(inode)? {
            return record
                .record
                .ok_or(StorageError::Integrity("released frontier inode"));
        }
        self.base_record(objects, inode, base)
    }

    fn base_record(
        &self,
        objects: &ObjectBuffer<'_>,
        inode: InodeId,
        base: Option<InodeRecordV1>,
    ) -> Result<InodeRecordV1> {
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

    #[cfg(test)]
    fn set(&mut self, inode: InodeId, record: Option<InodeRecordV1>) -> Result<()> {
        self.set_value(inode, record, None)
    }

    fn set_checkpoint(
        &mut self,
        inode: InodeId,
        record: InodeRecordV1,
        node: NodeId,
        content: ObjectId,
    ) -> Result<()> {
        self.set_value(inode, Some(record), Some((node, content)))
    }

    fn set_value(
        &mut self,
        inode: InodeId,
        record: Option<InodeRecordV1>,
        checkpoint: Option<(NodeId, ObjectId)>,
    ) -> Result<()> {
        if let Some(pending) = self.pending.get_mut(&inode) {
            pending.record = record;
            pending.checkpoint = checkpoint.or(pending.checkpoint);
        } else if let Some((index, mut value)) = self.spilled(inode)? {
            value.record = record;
            value.checkpoint = checkpoint.or(value.checkpoint);
            Self::write(self.spill.as_ref().unwrap(), index, inode, value)?;
        } else {
            self.insert_new(inode, FrontierValue { record, checkpoint })?;
        }
        Ok(())
    }

    fn insert_new(&mut self, inode: InodeId, value: FrontierValue) -> Result<()> {
        if self.pending.len() == self.batch_size {
            self.merge_pending()?;
        }
        self.pending.insert(inode, value);
        Ok(())
    }

    fn change_references(
        &mut self,
        objects: &ObjectBuffer<'_>,
        inode: InodeId,
        base: Option<InodeRecordV1>,
        amount: u64,
        additions: bool,
    ) -> Result<InodeRecordV1> {
        let adjust = |value: &mut FrontierValue| -> Result<InodeRecordV1> {
            let mut record = value
                .record
                .ok_or(StorageError::Integrity("released frontier inode"))?;
            record.namespace_ref_count = if additions {
                record
                    .namespace_ref_count
                    .checked_add(amount)
                    .ok_or(StorageError::Integrity("namespace reference overflow"))?
            } else {
                record
                    .namespace_ref_count
                    .checked_sub(amount)
                    .ok_or(StorageError::Integrity("namespace reference underflow"))?
            };
            value.record = (record.namespace_ref_count != 0).then_some(record);
            Ok(record)
        };
        if let Some(value) = self.pending.get_mut(&inode) {
            return adjust(value);
        }
        if let Some((index, mut value)) = self.spilled(inode)? {
            let record = adjust(&mut value)?;
            Self::write(self.spill.as_ref().unwrap(), index, inode, value)?;
            return Ok(record);
        }
        let mut value = FrontierValue {
            record: Some(self.base_record(objects, inode, base)?),
            checkpoint: None,
        };
        let record = adjust(&mut value)?;
        self.insert_new(inode, value)?;
        Ok(record)
    }

    fn merge_pending(&mut self) -> Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let file = anonymous_journal(&self.directory)?;
        let mut writer = BufWriter::with_capacity(self.io_bytes(), file);
        let mut reader = self
            .spill
            .as_ref()
            .map(|file| BufReader::with_capacity(self.io_bytes(), file));
        if let Some(reader) = &mut reader {
            reader.seek(SeekFrom::Start(0))?;
        }
        let mut remaining = self.count;
        let mut next_old = || -> Result<Option<(InodeId, FrontierValue)>> {
            if remaining == 0 {
                return Ok(None);
            }
            let mut bytes = [0; 192];
            reader
                .as_mut()
                .ok_or(StorageError::Integrity("frontier spill"))?
                .read_exact(&mut bytes)?;
            remaining -= 1;
            Self::decode(&bytes).map(Some)
        };
        let mut pending = self.pending.iter().peekable();
        let mut count = 0;
        let mut old = next_old()?;
        // ponytail: bounded sorted runs still merge O(N² / buffer capacity) new keys;
        // use tiered runs only if substantially larger deltas make this dominant.
        while old.is_some() || pending.peek().is_some() {
            let delta = match (old, pending.peek()) {
                (Some(previous), Some((&key, _))) if previous.0 < key => {
                    old = next_old()?;
                    previous
                }
                (Some(previous), None) => {
                    old = next_old()?;
                    previous
                }
                _ => {
                    let (&key, &record) = pending.next().unwrap();
                    if old.is_some_and(|entry| entry.0 == key) {
                        old = next_old()?;
                    }
                    (key, record)
                }
            };
            writer.write_all(&Self::encode(delta.0, delta.1))?;
            count += 1;
        }
        writer.flush()?;
        let file = writer.into_inner().map_err(|error| error.into_error())?;
        drop(reader);
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

    fn finish(
        &mut self,
        objects: &mut ObjectBuffer<'_>,
        mut checkpoint: impl FnMut(
            &ObjectBuffer<'_>,
            InodeId,
            NodeId,
            ObjectId,
            InodeRecordV1,
        ) -> Result<()>,
    ) -> Result<()> {
        let mut encode = |objects: &mut ObjectBuffer<'_>,
                          inode: InodeId,
                          value: FrontierValue|
         -> Result<Option<ObjectId>> {
            if let Some((node, content)) = value.checkpoint {
                checkpoint(
                    objects,
                    inode,
                    node,
                    content,
                    value
                        .record
                        .ok_or(StorageError::Integrity("released checkpoint inode"))?,
                )?;
            }
            value
                .record
                .map(|record| {
                    objects
                        .put_owned(encode_inode_record(record)?)
                        .map_err(Into::into)
                })
                .transpose()
        };
        let mut memory = Vec::new();
        if self.spill.is_none() {
            // Consume the bounded map: memory-resident final changes need no journal.
            memory.reserve_exact(self.pending.len());
            while let Some((inode, record)) = self.pending.pop_first() {
                let id = encode(objects, inode, record)?;
                memory.push((inode, id));
            }
        } else {
            self.merge_pending()?;
            let file = self.spill.as_ref().unwrap();
            // Encode once and attach IDs in sequential blocks, preserving raw
            // records for diagnostics and the same sorted-run representation.
            let rows_per_block = (self.io_bytes() / 192).max(1);
            let mut bytes = vec![0; rows_per_block * 192];
            let mut first = 0;
            while first < self.count {
                let rows = (self.count - first).min(rows_per_block as u64) as usize;
                let block = &mut bytes[..rows * 192];
                file.read_exact_at(block, first * 192)?;
                for row in block.chunks_exact_mut(192) {
                    let (inode, value) = Self::decode((&*row).try_into().unwrap())?;
                    if let Some(id) = encode(objects, inode, value)? {
                        row[160..192].copy_from_slice(id.as_bytes());
                    }
                }
                file.write_all_at(block, first * 192)?;
                first += rows as u64;
            }
        }
        let count = if self.spill.is_some() {
            self.count
        } else {
            memory.len() as u64
        };
        if count == 0 {
            return Ok(());
        }
        let namespace = filesystem::namespace(objects, self.root)?;
        let mut reader = self
            .spill
            .as_ref()
            .map(|file| BufReader::with_capacity(self.io_bytes(), file));
        if let Some(reader) = &mut reader {
            reader.seek(SeekFrom::Start(0))?;
        }
        let mut source_error = None;
        let deltas = (0..count).map(|index| {
            let result = Self::final_delta(&mut reader, &memory, index);
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
                if let Some(reader) = &mut reader {
                    reader.seek(SeekFrom::Start(0))?;
                }
                let mut root = self.root;
                for index in 0..count {
                    let (inode, id) = Self::final_delta(&mut reader, &memory, index)?;
                    let mutation = match id {
                        Some(id) => InodeMutation::Upsert {
                            inode,
                            record: ObjectStore::with_authenticated_canonical(
                                objects,
                                id,
                                decode_inode_record,
                            )?,
                        },
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

    fn final_delta(
        reader: &mut Option<BufReader<&File>>,
        memory: &[(InodeId, Option<ObjectId>)],
        index: u64,
    ) -> Result<(InodeId, Option<ObjectId>)> {
        if let Some(reader) = reader {
            let mut bytes = [0; 192];
            reader.read_exact(&mut bytes)?;
            Ok((
                InodeId(bytes[..32].try_into().unwrap()),
                if bytes[32] == 0 {
                    None
                } else {
                    Some(ObjectId::from_bytes(&bytes[160..192])?)
                },
            ))
        } else {
            Ok(memory[index as usize])
        }
    }

    fn base_records(
        base: &CoreReader<'_>,
        table: InodeTableRoot,
        keys: &[InodeId],
        lookup_limit: usize,
    ) -> Result<Vec<InodeRecordV1>> {
        let mut ids = Vec::with_capacity(keys.len());
        for keys in keys.chunks(lookup_limit) {
            for id in
                inode_table_lookup_many(base, table, keys, &mut InodeTableCounters::default())?
            {
                ids.push(id.ok_or(StorageError::Integrity("referenced inode record"))?);
            }
        }
        let objects = base.0.read_authenticated_objects(&ids)?;
        if objects.len() != ids.len() {
            return Err(StorageError::Integrity("reference record cardinality"));
        }
        objects
            .into_iter()
            .zip(ids)
            .map(|(object, id)| {
                if object.id != id {
                    return Err(StorageError::Integrity("reference record identity"));
                }
                decode_inode_record(&object.bytes).map_err(Into::into)
            })
            .collect()
    }

    fn apply_references(
        &mut self,
        objects: &ObjectBuffer<'_>,
        base: &CoreReader<'_>,
        journal: Option<(File, u64)>,
        budget: u64,
        io_bytes: usize,
    ) -> Result<()> {
        let Some((file, count)) = journal else {
            return Ok(());
        };
        let namespace = filesystem::namespace(objects, self.root)?;
        // Complete additions before any zero-reference traversal. Within each
        // bounded page aliases coalesce, while current changes override prefetch.
        for additions in [true, false] {
            let mut reader = BufReader::with_capacity(io_bytes, &file);
            reader.seek(SeekFrom::Start(0))?;
            let mut remaining = count;
            while remaining != 0 {
                let held = self.pending.len() as u64 * 256;
                let limit =
                    (budget.saturating_sub(held + 1024) / (32 * 1024 + 512)).clamp(1, 128) as usize;
                let mut changes = BTreeMap::<InodeId, u64>::new();
                for _ in 0..remaining.min(limit as u64) {
                    let mut row = [0; 65];
                    reader.read_exact(&mut row)?;
                    if row[0] == 0 || row[0] > 3 {
                        return Err(StorageError::Integrity("reference journal flags"));
                    }
                    let (flag, offset) = if additions { (2, 33) } else { (1, 1) };
                    if row[0] & flag != 0 {
                        *changes
                            .entry(InodeId(row[offset..offset + 32].try_into().unwrap()))
                            .or_default() += 1;
                    }
                    remaining -= 1;
                }
                if changes.is_empty() {
                    continue;
                }
                let started = Instant::now();
                let keys = changes.keys().copied().collect::<Vec<_>>();
                let records = Self::base_records(
                    base,
                    InodeTableRoot(namespace.inode_table_root),
                    &keys,
                    limit,
                )?;
                note_commit_phase(WorkspaceCommitPhase::DeletionRecords, started);
                let retained = keys.len() as u64 * 512;
                drop(keys);
                for ((inode, amount), record) in changes.into_iter().zip(records) {
                    if additions {
                        self.change_references(objects, inode, Some(record), amount, true)?;
                    } else {
                        self.release(
                            objects,
                            base,
                            inode,
                            Some(record),
                            amount,
                            budget.saturating_sub(retained),
                        )?;
                    }
                }
            }
            if reader.read(&mut [0; 1])? != 0 {
                return Err(StorageError::Integrity("reference journal coverage"));
            }
        }
        Ok(())
    }

    fn release(
        &mut self,
        objects: &ObjectBuffer<'_>,
        base: &CoreReader<'_>,
        inode: InodeId,
        prefetched: Option<InodeRecordV1>,
        amount: u64,
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
        let mut next = Some((inode, prefetched, amount));
        loop {
            if let Some((inode, prefetched, amount)) = next.take() {
                let started = Instant::now();
                // Earlier additions/releases override authenticated page prefetch.
                let record = self.change_references(objects, inode, prefetched, amount, false)?;
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
                next = Some((inode, Some(record), 1));
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
            let records = Self::base_records(
                base,
                InodeTableRoot(namespace.inode_table_root),
                &inodes,
                lookup_limit,
            )?;
            cursor.children = inodes.into_iter().zip(records).collect();
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

struct WorkspaceFileReader {
    source: WorkspaceFileSource,
    offset: u64,
    len: u64,
}

enum WorkspaceFileSource {
    Direct(layerfs_workspace_core::backing::BackingRef, u64),
    Mixed(FrozenFile),
}

struct WorkspaceRangeReader<'a> {
    input: &'a FrozenFile,
    offset: u64,
    end: u64,
}

impl<'a> WorkspaceRangeReader<'a> {
    fn new(input: &'a FrozenFile, offset: u64, len: u64) -> Result<Self> {
        let end = offset
            .checked_add(len)
            .ok_or(StorageError::InvalidInput("file range"))?;
        if end > input.len {
            return Err(StorageError::InvalidInput("file range"));
        }
        Ok(Self { input, offset, end })
    }
}

impl Read for WorkspaceRangeReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        if self.offset == self.end || output.is_empty() {
            return Ok(0);
        }
        let bytes = self
            .input
            .read(
                self.offset,
                output.len().min((self.end - self.offset) as usize),
            )
            .map_err(std::io::Error::other)?;
        output[..bytes.len()].copy_from_slice(&bytes);
        self.offset += bytes.len() as u64;
        Ok(bytes.len())
    }
}

impl WorkspaceFileReader {
    fn new(workspace: &Workspace, node: NodeId) -> Result<Self> {
        Ok(FrozenFile::from_node(
            &workspace.reader,
            workspace
                .live
                .nodes
                .get(&node)
                .ok_or(StorageError::NotFound("node"))?,
        )?
        .reader())
    }
}

impl Read for WorkspaceFileReader {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        if self.offset == self.len || output.is_empty() {
            return Ok(0);
        }
        let count = output.len().min((self.len - self.offset) as usize);
        match &self.source {
            WorkspaceFileSource::Direct(file, start) => {
                let mut read = 0;
                while read < count {
                    let next = crate::file_io::spool_segment(file)
                        .map_err(std::io::Error::other)?
                        .file
                        .read_at(&mut output[read..count], start + self.offset + read as u64)?;
                    if next == 0 {
                        return Err(std::io::ErrorKind::UnexpectedEof.into());
                    }
                    read += next;
                }
                self.offset += read as u64;
                Ok(read)
            }
            WorkspaceFileSource::Mixed(input) => {
                let bytes = input
                    .read(self.offset, count)
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
    fn producer_workers_preserve_roots_alias_capture_and_empty_inputs() {
        let (root, mut workspace) = empty_workspace("producer-workers");
        workspace.mkdir(ROOT, b"directory", 0o700).unwrap();
        let empty = workspace
            .build_frontier_candidate_with_workers(CandidatePurpose::Preview, 4)
            .unwrap();
        assert_eq!(empty.built.counters.cdc_bytes_scanned, 0);
        drop(empty);
        let mut files = Vec::new();
        for index in 0..4 {
            let data = vec![index as u8 + 1; 100_000 + index * 123];
            let node = workspace
                .create_file(ROOT, format!("file-{index}").as_bytes(), 0o640)
                .unwrap()
                .node;
            workspace.write(node, 0, &data).unwrap();
            files.push((node, data));
        }
        workspace.link(files[0].0, ROOT, b"alias").unwrap();
        let arm_capture = |workspace: &mut Workspace| {
            let mut objects = ObjectBuffer::empty().unwrap();
            let (root, counters) = rope::build(&mut objects, files[0].1.as_slice()).unwrap();
            workspace.capture =
                crate::capture::CaptureState::Ready(Box::new(crate::capture::CapturedFile {
                    node: files[0].0,
                    len: files[0].1.len() as u64,
                    root,
                    counters,
                    objects: objects.into_resumable(),
                }));
        };
        let before = workspace.store.store_counts().unwrap();
        arm_capture(&mut workspace);
        let serial = workspace
            .build_frontier_candidate_with_workers(CandidatePurpose::Preview, 1)
            .unwrap();
        let expected = serial.built.root_id;
        let expected_ids = serial
            .built
            .objects
            .ids_in_order(usize::MAX)
            .unwrap()
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let scanned = files
            .iter()
            .map(|(_, bytes)| bytes.len() as u64)
            .sum::<u64>();
        assert_eq!(serial.built.counters.cdc_bytes_scanned, scanned);
        drop(serial);
        // Corrupt only the captured inode's backing to prove that its completed
        // output is reused, rather than silently rebuilding it for either alias.
        let Data::File(FileData::Edited { pieces, .. }) = &workspace.live.nodes[&files[0].0].data
        else {
            unreachable!()
        };
        let captured_slice = pieces.compact_spool().unwrap();
        let captured_segment = captured_slice.segment.clone();
        let captured_offset = captured_slice.offset;
        crate::file_io::spool_segment(&captured_segment)
            .unwrap()
            .file
            .write_all_at(&vec![0; files[0].1.len()], captured_offset)
            .unwrap();
        arm_capture(&mut workspace);
        let parallel = workspace
            .build_frontier_candidate_with_workers(CandidatePurpose::Preview, 4)
            .unwrap();
        assert_eq!(parallel.built.root_id, expected);
        assert_eq!(parallel.built.counters.cdc_bytes_scanned, scanned);
        assert_eq!(
            parallel
                .built
                .objects
                .ids_in_order(usize::MAX)
                .unwrap()
                .unwrap()
                .into_iter()
                .collect::<BTreeSet<_>>(),
            expected_ids
        );
        assert_eq!(workspace.store.store_counts().unwrap(), before);
        assert!(workspace.take_capture().is_none());
        drop(parallel);
        crate::file_io::spool_segment(&captured_segment)
            .unwrap()
            .file
            .write_all_at(&files[0].1, captured_offset)
            .unwrap();
        drop(captured_segment);
        arm_capture(&mut workspace);
        let admitted = workspace
            .build_frontier_candidate_with_workers(CandidatePurpose::Commit, 4)
            .unwrap();
        assert_eq!(admitted.built.root_id, expected);
        assert_eq!(admitted.built.counters.cdc_bytes_scanned, scanned);
        assert!(admitted.admission.is_some());
        drop(admitted);
        workspace.commit().unwrap();
        assert_eq!(workspace.base_root, expected);
        let old = workspace.reader.clone();
        workspace.write(files[1].0, 7, b"edit").unwrap();
        workspace.truncate(files[2].0, 150_000).unwrap();
        let serial = workspace
            .build_frontier_candidate_with_workers(CandidatePurpose::Preview, 1)
            .unwrap();
        let parallel = workspace
            .build_frontier_candidate_with_workers(CandidatePurpose::Preview, 4)
            .unwrap();
        assert_eq!(serial.built.root_id, parallel.built.root_id);
        assert_eq!(
            serial.built.counters.cdc_bytes_scanned,
            parallel.built.counters.cdc_bytes_scanned
        );
        assert!(parallel.built.counters.cdc_bytes_scanned < scanned);
        assert!(layerfs_layerstack_store::ObjectSource::read_object(&old, expected).is_ok());
        drop(serial);
        drop(parallel);
        drop(old);
        workspace.policy.max_final_delta_memory_bytes = 1024;
        let result = workspace.build_frontier_candidate_with_workers(CandidatePurpose::Preview, 4);
        // Existing tiny-budget fallback is allowed to reject, never to exceed its policy.
        if let Err(error) = result {
            assert!(error.to_string().contains("limit"));
        }
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn file_result_journals_restore_reverse_completion_and_reject_missing_slots() {
        let (root, mut workspace) = empty_workspace("reverse-results");
        for index in 0..4 {
            let node = workspace
                .create_file(ROOT, format!("file-{index}").as_bytes(), 0o600)
                .unwrap()
                .node;
            workspace.write(node, 0, &[index as u8; 17]).unwrap();
        }
        let inputs = StableFileInputs {
            nodes: &workspace.live.nodes,
            dirty: &workspace.live.dirty,
            reader: workspace.reader.clone(),
            base_inodes: workspace.base_inodes,
            generation: workspace.live.mutation_generation,
            spool: &workspace.spool,
            io_bytes: 1024,
            captured: std::sync::Mutex::new(None),
        };
        let plan = inputs.prepare().unwrap();
        assert_eq!(plan.count, 4);
        let tasks = plan
            .tasks(256)
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        let mut workers = (0..4)
            .map(|index| inputs.worker(index, 4).unwrap())
            .collect::<Vec<_>>();
        for ordinal in (0..4).rev() {
            inputs
                .produce_file(
                    &mut workers[ordinal],
                    &plan.file,
                    ordinal,
                    tasks[ordinal],
                    &mut |_| Ok(()),
                )
                .unwrap();
        }
        let mut output = FileResults::new(
            plan,
            workers
                .into_iter()
                .map(|worker| worker.finish().unwrap())
                .collect(),
            256,
        )
        .unwrap();
        for node in &tasks {
            output.next(*node, inputs.generation, 17).unwrap();
        }
        output.finish_read().unwrap();
        let missing = inputs.prepare().unwrap();
        assert!(FileResults::new(missing, Vec::new(), 256).is_err());
        drop(output);
        drop(inputs);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preview_stays_private_and_streamed_commit_has_the_same_root() {
        let (root, mut workspace) = empty_workspace("selected-output");
        for name in [b"first".as_slice(), b"second"] {
            let node = workspace.create_file(ROOT, name, 0o640).unwrap().node;
            workspace.write(node, 0, b"same selected payload").unwrap();
        }
        let counts = workspace.store.store_counts().unwrap();
        let preview = workspace
            .build_candidate(CandidatePurpose::Preview)
            .unwrap();
        assert_eq!(workspace.store.store_counts().unwrap(), counts);
        assert!(preview.admission.is_none());
        let expected = preview.built.root_id;
        drop(preview);
        workspace.commit().unwrap();
        assert_eq!(workspace.base_root, expected);
        assert_eq!(
            workspace.store.store_counts().unwrap().commits,
            counts.commits + 1
        );
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
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
    fn reference_journal_rewind_keeps_prior_edges_and_latest_alias_counts() {
        let (root, mut workspace) = empty_workspace("reference-facts");
        let mut nodes = Vec::new();
        for name in [b"first", b"other", b"alias"] {
            let node = workspace.create_file(ROOT, name, 0o600).unwrap().node;
            workspace.write(node, 0, name).unwrap();
            nodes.push(node);
        }
        workspace.commit().unwrap();
        let ids = nodes
            .iter()
            .map(|node| workspace.live.nodes[node].canonical.unwrap())
            .collect::<Vec<_>>();
        let mut journal = ReferenceJournal::new(&workspace.spool, 128);
        journal.push(Some(ids[0]), None).unwrap();
        let before_trial = journal.count;
        journal.push(Some(ids[1]), None).unwrap();
        journal.rewind(before_trial).unwrap();
        journal.push(None, Some(ids[2])).unwrap();
        journal.push(None, Some(ids[2])).unwrap();
        journal.push(Some(ids[2]), None).unwrap();
        let objects = ObjectBuffer::new(&workspace.reader).unwrap();
        let mut inodes = FrontierInodes::new(workspace.base_root, 1, 0, &workspace.spool);
        inodes
            .apply_references(
                &objects,
                &CoreReader(&workspace.reader),
                journal.finish().unwrap(),
                8 * 1024 * 1024,
                128,
            )
            .unwrap();
        assert!(inodes.record(&objects, ids[0]).is_err());
        assert_eq!(
            inodes.record(&objects, ids[1]).unwrap().namespace_ref_count,
            1
        );
        assert_eq!(
            inodes.record(&objects, ids[2]).unwrap().namespace_ref_count,
            2
        );
        drop(objects);
        // A single-file deletion remains supported at the existing minimum budget.
        workspace.policy.max_final_delta_memory_bytes = 1024;
        workspace.unlink(ROOT, b"other", false).unwrap();
        workspace.commit().unwrap();
        assert!(workspace.lookup(ROOT, b"other").is_err());
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn final_records_match_in_memory_spill_and_small_budget_builders() {
        assert!(std::mem::size_of::<(InodeId, FrontierValue)>() + 64 <= 256);
        let (root, mut workspace) = empty_workspace("final-records");
        let mut nodes = Vec::new();
        for index in 0..12 {
            let node = workspace
                .create_file(ROOT, format!("f{index}").as_bytes(), 0o600)
                .unwrap()
                .node;
            workspace.write(node, 0, &[index]).unwrap();
            nodes.push(node);
        }
        workspace.commit().unwrap();
        let mut roots = Vec::new();
        for (capacity, scratch) in [(32, 16384), (2, 16384), (2, 0)] {
            let mut objects = ObjectBuffer::new(&workspace.reader).unwrap();
            let mut inodes =
                FrontierInodes::new(workspace.base_root, capacity, scratch, &workspace.spool);
            let first = inodes
                .record(&objects, workspace.live.nodes[&nodes[0]].canonical.unwrap())
                .unwrap();
            let first_inode = workspace.live.nodes[&nodes[0]].canonical.unwrap();
            inodes
                .set_checkpoint(first_inode, first, nodes[0], first.content_root)
                .unwrap();
            let discarded = workspace.live.nodes[nodes.last().unwrap()]
                .canonical
                .unwrap();
            inodes.set(discarded, Some(first)).unwrap();
            for node in &nodes {
                let inode = workspace.live.nodes[node].canonical.unwrap();
                let mut record = inodes.record(&objects, inode).unwrap();
                record.content_root = first.content_root;
                inodes.set(inode, Some(record)).unwrap();
            }
            inodes.set(discarded, None).unwrap();
            let mut checked = 0;
            inodes
                .finish(&mut objects, |_, inode, node, content, record| {
                    assert_eq!(
                        (inode, node, content),
                        (first_inode, nodes[0], first.content_root)
                    );
                    assert_eq!(record.content_root, content);
                    checked += 1;
                    Ok(())
                })
                .unwrap();
            assert_eq!(checked, 1);
            assert_eq!(inodes.spill.is_none(), capacity == 32);
            roots.push(inodes.root);
        }
        assert!(roots.windows(2).all(|pair| pair[0] == pair[1]));
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
        let inode = workspace.live.nodes[&file].canonical.unwrap();
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
        for pinned in [false, true] {
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
            if pinned {
                workspace.pin(a, false).unwrap();
            }
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
            if pinned {
                workspace.unpin(a).unwrap();
            }
            drop(workspace);
            std::fs::remove_dir_all(root).unwrap();
        }
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
        let inode = workspace.live.nodes[&file].canonical.unwrap();
        let objects = ObjectBuffer::new(&workspace.reader).unwrap();
        let mut record = FrontierInodes::new(workspace.base_root, 1, 0, &workspace.spool)
            .record(&objects, inode)
            .unwrap();
        journal
            .push(file, inode, record.content_root, attr, Some(attr.size))
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
        let path = workspace.live.nodes[&parent].paths.first().unwrap();
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
            .map(|(_, node)| workspace.live.nodes[node].canonical.unwrap())
            .collect::<Vec<_>>();
        workspace.policy.max_final_delta_memory_bytes = 4096;
        for (index, (_, node)) in files.iter().enumerate() {
            workspace
                .write(*node, 0, format!("changed-{index:02}").as_bytes())
                .unwrap();
            workspace.set_mtime(*node, 1700000001, 123).unwrap();
        }
        assert!(workspace
            .live
            .nodes
            .values()
            .all(|node| !matches!(&node.data,
            Data::Directory(directory) if !directory.changes.is_empty())));
        assert!(
            workspace
                .live
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
        assert!(matches!(reader.source, WorkspaceFileSource::Direct(..)));
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, data);

        let sparse = workspace.create_file(ROOT, b"sparse", 0o600).unwrap();
        workspace.write(sparse.node, 4096, b"x").unwrap();
        assert!(matches!(
            WorkspaceFileReader::new(&workspace, sparse.node)
                .unwrap()
                .source,
            WorkspaceFileSource::Mixed(..)
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
            &mut workspace.live.nodes.get_mut(&file.node).unwrap().data
        {
            let mut original = pieces.pieces().into_iter();
            let crate::file_edit::Piece::Spool {
                segment,
                offset,
                len,
            } = original.next().unwrap()
            else {
                panic!("spool piece")
            };
            let count = pieces.count();
            *pieces = crate::file_edit::PieceTree::empty()
                .replace(
                    0,
                    0,
                    [
                        crate::file_edit::Piece::Spool {
                            segment: segment.clone(),
                            offset,
                            len: 1,
                        },
                        crate::file_edit::Piece::Spool {
                            segment,
                            offset: offset + 1,
                            len: len - 1,
                        },
                    ]
                    .into_iter()
                    .chain(original),
                )
                .unwrap();
            assert_eq!(pieces.count(), count + 1);
        }
        let captured = workspace
            .take_capture()
            .expect("sequential capture is reusable");
        let captured_root = captured.root;
        workspace.capture = crate::capture::CaptureState::Ready(Box::new(captured));

        let store = workspace.store.clone();
        let branch = store.branch(workspace.branch_id).unwrap().unwrap();
        let built = workspace
            .build_candidate(CandidatePurpose::Preview)
            .unwrap();
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
        let built = workspace
            .build_candidate(CandidatePurpose::Preview)
            .unwrap();
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
            let original_inode = workspace.live.nodes[&file].canonical.unwrap();
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
            let built = workspace
                .build_candidate(CandidatePurpose::Preview)
                .unwrap();
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
        let base_root = match workspace.live.nodes[&file].data {
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
        let built = workspace
            .build_candidate(CandidatePurpose::Preview)
            .unwrap();
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

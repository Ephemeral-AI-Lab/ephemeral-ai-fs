use crate::cow_tree::{Data, FileData, Node, NodeId, Workspace};
use crate::file_edit::{
    Piece, PieceTree, SpoolSlice, MAX_EDITS_PER_FILE, MAX_INLINE_PER_EDIT,
    MAX_INLINE_PER_WORKSPACE, MAX_PIECE_ALLOCATION,
};
use layerfs_content::file::rope::read_range;
use layerfs_layerstack_store::{CoreReader, Result, SnapshotReader, StoreError};
use layerfs_workspace_core::backing::{BackingId, BackingRef};
use layerfs_workspace_core::ReadSource;
use std::fs::{File, OpenOptions};
#[cfg(test)]
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

// Physical segments are shared by immutable piece/read-plan references. Only the
// existing exclusive Workspace writer appends or rolls back an unpublished tail.
const SPOOL_SEGMENT_BYTES: u64 = 1024 * 1024;

#[derive(Debug)]
pub(crate) struct SpoolSegment {
    pub(crate) file: File,
    id: u64,
    len: AtomicU64,
    capacity: u64,
    physical: Arc<Mutex<crate::cow_tree::PhysicalSpoolMetrics>>,
}

impl SpoolSegment {
    fn new(
        directory: &Path,
        id: u64,
        capacity: u64,
        physical: Arc<Mutex<crate::cow_tree::PhysicalSpoolMetrics>>,
    ) -> Result<Self> {
        let path = directory.join(format!("segment-{}", crate::WorkspaceId::new()));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        // Access uses the exclusively owned descriptor; no mutable path can
        // replace this backing, and no per-logical-file unlink remains at Commit.
        std::fs::remove_file(path)?;
        let segment = Self {
            file,
            id,
            len: AtomicU64::new(0),
            capacity,
            physical,
        };
        segment.observe();
        Ok(segment)
    }

    fn observe(&self) {
        let metadata = self.file.metadata();
        if let Ok(mut physical) = self.physical.lock() {
            match metadata {
                Ok(metadata) => physical.observe(self.id, &metadata),
                Err(_) => physical.error(),
            }
        }
    }

    fn check(&self) -> Result<()> {
        let metadata = self.file.metadata().inspect_err(|_| {
            if let Ok(mut physical) = self.physical.lock() {
                physical.error();
            }
        })?;
        if metadata.len() != self.len.load(Ordering::Relaxed) {
            if let Ok(mut physical) = self.physical.lock() {
                physical.error();
            }
            return Err(StoreError::Integrity("spool segment high-water"));
        }
        Ok(())
    }
}

impl Drop for SpoolSegment {
    fn drop(&mut self) {
        if let Ok(mut physical) = self.physical.lock() {
            physical.removed(self.id);
        }
    }
}

pub(crate) fn spool_segment(backing: &BackingRef) -> Result<&SpoolSegment> {
    backing
        .resource()
        .ok_or(StoreError::Integrity("host spool backing"))
}

pub(crate) struct EditCheckpoint {
    node: NodeId,
    value: crate::cow_tree::Node,
    dirty: bool,
    spool_bytes: u64,
    spool_bytes_peak: u64,
    inline_bytes: u64,
    piece_allocation_bytes: u64,
    spool_write_metrics: crate::cow_tree::SpoolWriteMetrics,
    mutation_generation: u64,
    mutation_paths: std::collections::BTreeMap<String, u64>,
}

pub struct ReadPlan {
    reader: SnapshotReader,
    plan: layerfs_workspace_core::ReadPlan,
}

impl ReadPlan {
    pub(crate) fn for_file(
        reader: SnapshotReader,
        data: &FileData,
        offset: u64,
        size: usize,
    ) -> Result<Self> {
        let plan = layerfs_workspace_core::ReadPlan::for_file(data, offset, size)
            .map_err(crate::live_error)?;
        layerfs_layerstack_store::note_workspace_commit_tree_visits(plan.tree_visits as u64);
        Ok(Self { reader, plan })
    }

    pub fn read(self) -> Result<Vec<u8>> {
        let started = std::time::Instant::now();
        let reader = self.reader.clone();
        let output = match self.plan.source {
            ReadSource::Base(root, start, end) => read_base(&self.reader, root, start, end),
            ReadSource::Edited(pieces) => {
                let mut output = Vec::with_capacity(as_usize(self.plan.requested)?);
                for piece in pieces {
                    match piece {
                        Piece::Base { root, offset, len } => {
                            output.extend(read_base(&self.reader, root, offset, offset + len)?)
                        }
                        Piece::Inline { bytes, offset, len } => output
                            .extend_from_slice(&bytes[as_usize(offset)?..as_usize(offset + len)?]),
                        Piece::Zero { len } => output.resize(output.len() + as_usize(len)?, 0),
                        Piece::Spool {
                            segment,
                            offset,
                            len,
                        } => {
                            let start = output.len();
                            output.resize(start + as_usize(len)?, 0);
                            read_exact_at(
                                &spool_segment(&segment)?.file,
                                &mut output[start..],
                                offset,
                            )?;
                        }
                    }
                }
                Ok(output)
            }
        }?;
        reader.note_workspace_read(
            self.plan.requested,
            output.len() as u64,
            elapsed_ns(started),
        )?;
        Ok(output)
    }
}

impl Workspace {
    pub(crate) fn note_commit_edit_state(&self) -> Result<()> {
        let mut edits = 0_u64;
        let mut pieces = 0_u64;
        let mut height = 0_u64;
        let mut charge = 0_u64;
        let mut spool_live = 0_u64;
        let mut metric_nodes_scanned = 0_u64;
        for node in self
            .live
            .dirty
            .iter()
            .filter_map(|node| self.live.nodes.get(node))
        {
            metric_nodes_scanned = metric_nodes_scanned.saturating_add(1);
            if let Data::File(FileData::Edited {
                pieces: tree,
                edits: file_edits,
                ..
            }) = &node.data
            {
                edits = edits.saturating_add(u64::from(*file_edits));
                pieces = pieces.saturating_add(tree.count() as u64);
                height = height.max(tree.height() as u64);
                charge = charge.saturating_add(
                    tree.logical_allocation_charge()
                        .map_err(crate::live_error)?,
                );
                spool_live = spool_live.saturating_add(tree.spool_len());
            }
        }
        layerfs_layerstack_store::note_workspace_commit_edit_state(
            edits,
            pieces,
            height,
            charge,
            self.live.spool_bytes,
            self.live.spool_bytes_peak,
            spool_live,
            self.live.spool_bytes.saturating_sub(spool_live),
            metric_nodes_scanned,
        );
        let (current, peak, errors, observations) = self.physical_spool_snapshot();
        layerfs_layerstack_store::note_workspace_physical_spool(
            current,
            peak,
            errors,
            observations,
        );
        Ok(())
    }

    pub(crate) fn edit_checkpoint(&self, node: NodeId) -> Result<EditCheckpoint> {
        Ok(EditCheckpoint {
            node,
            value: self
                .live
                .nodes
                .get(&node)
                .ok_or(StoreError::NotFound("node"))?
                .clone(),
            dirty: self.live.dirty.contains(&node),
            spool_bytes: self.live.spool_bytes,
            spool_bytes_peak: self.live.spool_bytes_peak,
            inline_bytes: self.live.inline_bytes,
            piece_allocation_bytes: self.live.piece_allocation_bytes,
            spool_write_metrics: self.spool_write_metrics,
            mutation_generation: self.live.mutation_generation,
            mutation_paths: self.live.mutation_paths.clone(),
        })
    }

    pub(crate) fn restore_edit(&mut self, checkpoint: EditCheckpoint) -> Result<()> {
        if matches!(checkpoint.value.data, Data::File(FileData::Base { .. })) {
            self.live.edited_nodes.remove(&checkpoint.node);
        } else {
            self.live.edited_nodes.insert(checkpoint.node);
        }
        self.live.nodes.insert(checkpoint.node, checkpoint.value);
        if checkpoint.dirty {
            self.live.dirty.insert(checkpoint.node);
        } else {
            self.live.dirty.remove(&checkpoint.node);
        }
        self.live.spool_bytes = checkpoint.spool_bytes;
        self.live.spool_bytes_peak = checkpoint.spool_bytes_peak;
        self.live.inline_bytes = checkpoint.inline_bytes;
        self.live.piece_allocation_bytes = checkpoint.piece_allocation_bytes;
        self.spool_write_metrics = checkpoint.spool_write_metrics;
        self.live.mutation_generation = checkpoint.mutation_generation;
        self.live.mutation_paths = checkpoint.mutation_paths;
        self.retire_spool_segments();
        Ok(())
    }

    pub fn read(&self, node: NodeId, offset: u64, size: usize) -> Result<Vec<u8>> {
        self.read_plan(node, offset, size)?.read()
    }
    pub fn read_plan(&self, node: NodeId, offset: u64, size: usize) -> Result<ReadPlan> {
        match &self
            .live
            .nodes
            .get(&node)
            .ok_or(StoreError::NotFound("node"))?
            .data
        {
            Data::File(data) => ReadPlan::for_file(self.reader.clone(), data, offset, size),
            _ => Err(StoreError::InvalidInput("read")),
        }
    }

    pub fn write(&mut self, node: NodeId, offset: u64, bytes: &[u8]) -> Result<usize> {
        self.write_inner(node, offset, bytes.len(), Some(bytes))
    }
    pub(crate) fn write_zero(&mut self, node: NodeId, offset: u64, len: usize) -> Result<usize> {
        self.invalidate_capture();
        self.write_inner(node, offset, len, None)
    }
    fn write_inner(
        &mut self,
        node: NodeId,
        offset: u64,
        byte_len: usize,
        bytes: Option<&[u8]>,
    ) -> Result<usize> {
        self.ensure_active()?;
        if byte_len == 0 {
            return Ok(0);
        }
        let old_len = self.attr(node)?.size;
        let end = offset
            .checked_add(byte_len as u64)
            .ok_or(StoreError::InvalidInput("write length"))?;
        let physical = if bytes.is_some() {
            Some(self.append_segment(byte_len as u64)?)
        } else {
            None
        };
        let prepared = self
            .live
            .prepare_write(
                node,
                offset,
                byte_len,
                physical.as_ref().map(|(segment, offset)| SpoolSlice {
                    segment: segment.clone(),
                    offset: *offset,
                    len: byte_len as u64,
                }),
            )
            .map_err(crate::live_error)?;
        let appended = bytes.map_or(0, |bytes| bytes.len() as u64);
        #[cfg(feature = "test-instrumentation")]
        if appended > 0
            && crate::lifecycle::consume_verification_fault(
                self.branch_id,
                crate::lifecycle::VerificationFault::NoSpace,
                self.live.spool_bytes,
            )
        {
            let mut lowered = self.live.policy;
            lowered.max_spool_bytes = self.live.spool_bytes;
            lowered
                .check(
                    self.live
                        .spool_bytes
                        .checked_add(appended)
                        .ok_or(StoreError::InvalidInput("workspace spool limit"))?,
                )
                .map_err(crate::live_error)?;
        }
        if let Some(bytes) = bytes {
            let started = std::time::Instant::now();
            #[cfg(feature = "test-instrumentation")]
            let inject_short = crate::lifecycle::consume_verification_fault(
                self.branch_id,
                crate::lifecycle::VerificationFault::ShortAppend,
                self.live.spool_bytes,
            );
            let (backing, physical_start) = physical.as_ref().unwrap();
            let segment = spool_segment(backing)?;
            segment.check()?;
            let file = &segment.file;
            #[cfg(feature = "test-instrumentation")]
            let append = if inject_short {
                file.write_all_at(&bytes[..bytes.len() / 2], *physical_start)
                    .and_then(|_| Err(std::io::Error::other("injected short spool append")))
            } else {
                append_spool(file, bytes, *physical_start)
            };
            #[cfg(not(feature = "test-instrumentation"))]
            let append = append_spool(file, bytes, *physical_start);
            segment.observe();
            if let Err(error) = append {
                // No visible range references this tail; other files' earlier
                // bytes in the shared segment must never be truncated.
                #[cfg(test)]
                let cleanup = if INJECT_APPEND_CLEANUP_FAILURE.with(|inject| inject.replace(false))
                {
                    Err(std::io::Error::other("injected spool rollback failure"))
                } else {
                    file.set_len(*physical_start)
                };
                #[cfg(not(test))]
                let cleanup = file.set_len(*physical_start);
                segment.observe();
                if cleanup.is_err() {
                    let retained = file.metadata()?.len();
                    let extra = retained
                        .checked_sub(*physical_start)
                        .ok_or(StoreError::Integrity("spool append cleanup length"))?;
                    segment.len.store(retained, Ordering::Relaxed);
                    self.segment_bytes = self
                        .segment_bytes
                        .checked_add(extra)
                        .ok_or(StoreError::Integrity("spool segment charge"))?;
                    return Err(StoreError::Integrity("spool append cleanup failure"));
                }
                return Err(error.into());
            }
            segment
                .len
                .store(*physical_start + appended, Ordering::Relaxed);
            self.segment_bytes += appended;
            self.spool_write_metrics.write_bytes = self
                .spool_write_metrics
                .write_bytes
                .saturating_add(appended);
            self.spool_write_metrics.write_ns = self
                .spool_write_metrics
                .write_ns
                .saturating_add(elapsed_ns(started));
        }
        #[cfg(test)]
        if INJECT_STALE_WRITE.with(|inject| inject.replace(false)) {
            self.live
                .set_mtime(node, 123, 0)
                .map_err(crate::live_error)?;
        }
        self.live.apply_write(prepared).map_err(crate::live_error)?;
        if let Some(bytes) = bytes {
            self.capture_write(node, offset, old_len, bytes);
        }
        debug_assert_eq!(self.attr(node)?.size, old_len.max(end));
        Ok(byte_len)
    }

    pub(crate) fn edit_many(
        &mut self,
        node: NodeId,
        edits: Vec<(u64, u64, crate::WorkspaceFileReplacement)>,
    ) -> Result<()> {
        if edits.is_empty() {
            return Err(StoreError::InvalidInput("workspace edit batch"));
        }
        self.ensure_active()?;
        let (old, prior_edits, was_base) = match &self.live.nodes[&node].data {
            Data::File(FileData::Base { root, len }) => (
                PieceTree::base(*root, *len).map_err(crate::live_error)?,
                0,
                true,
            ),
            Data::File(FileData::Edited { pieces, edits, .. }) => (pieces.clone(), *edits, false),
            _ => return Err(StoreError::InvalidInput("file")),
        };
        let total_edits = prior_edits
            .checked_add(
                u32::try_from(edits.len())
                    .map_err(|_| StoreError::InvalidInput("workspace edit limit"))?,
            )
            .filter(|value| *value <= MAX_EDITS_PER_FILE)
            .ok_or(StoreError::InvalidInput("workspace edit limit"))?;
        let generation = self
            .live
            .mutation_generation
            .checked_add(edits.len() as u64)
            .ok_or(StoreError::Integrity("Workspace mutation generation"))?;
        let mut next = old.clone();
        for (start, delete_len, replacement) in edits {
            let piece = match replacement {
                crate::WorkspaceFileReplacement::Inline(bytes) => {
                    if bytes.len() > MAX_INLINE_PER_EDIT {
                        return Err(StoreError::InvalidInput("workspace inline edit limit"));
                    }
                    (!bytes.is_empty()).then(|| Piece::Inline {
                        len: bytes.len() as u64,
                        bytes: Arc::from(bytes),
                        offset: 0,
                    })
                }
                crate::WorkspaceFileReplacement::Zero(len) => {
                    (len != 0).then_some(Piece::Zero { len })
                }
            };
            next = next
                .replace(start, delete_len, piece)
                .map_err(crate::live_error)?;
            if was_base {
                self.live
                    .inline_bytes
                    .checked_add(next.inline_len())
                    .filter(|value| *value <= MAX_INLINE_PER_WORKSPACE)
                    .ok_or(StoreError::InvalidInput("workspace inline limit"))?;
                self.live
                    .piece_allocation_bytes
                    .checked_add(
                        next.logical_allocation_charge()
                            .map_err(crate::live_error)?,
                    )
                    .filter(|value| *value <= MAX_PIECE_ALLOCATION)
                    .ok_or(StoreError::InvalidInput("workspace piece allocation limit"))?;
            } else {
                self.check_piece_resources(&old, &next)?;
            }
        }
        let paths = self.live.nodes[&node].paths.iter().cloned().collect();
        self.invalidate_capture();
        self.ensure_edited(node)?;
        let (high_water, installed, _) = self.edited_state(node)?;
        self.install_edit(
            node,
            installed,
            next,
            total_edits,
            high_water,
            0,
            generation,
            paths,
        )
    }

    pub fn truncate(&mut self, node: NodeId, size: u64) -> Result<()> {
        self.invalidate_capture();
        self.ensure_active()?;
        let old_len = self.attr(node)?.size;
        if size == old_len {
            return Ok(());
        }
        self.ensure_edited(node)?;
        let (high_water, old, edits) = self.edited_state(node)?;
        for piece in old.pieces() {
            if let Piece::Spool { segment, .. } = piece {
                spool_segment(&segment)?.check()?;
            }
        }
        let (start, delete_len, replacement) = if size < old_len {
            (size, old_len - size, None)
        } else {
            (
                old_len,
                0,
                Some(Piece::Zero {
                    len: size - old_len,
                }),
            )
        };
        let next = old
            .replace(start, delete_len, replacement)
            .map_err(crate::live_error)?;
        let generation = self.next_generation()?;
        let paths = self.live.nodes[&node].paths.iter().cloned().collect();
        self.check_piece_resources(&old, &next)?;
        self.install_edit(
            node,
            old,
            next,
            next_edit(edits)?,
            high_water,
            0,
            generation,
            paths,
        )
    }

    fn next_generation(&self) -> Result<u64> {
        self.live.next_generation().map_err(crate::live_error)
    }
    fn edited_state(&self, node: NodeId) -> Result<(u64, PieceTree, u32)> {
        match &self.live.nodes[&node].data {
            Data::File(FileData::Edited {
                spool_high_water,
                pieces,
                edits,
                ..
            }) => Ok((*spool_high_water, pieces.clone(), *edits)),
            _ => Err(StoreError::InvalidInput("file")),
        }
    }
    fn check_piece_resources(&self, old: &PieceTree, next: &PieceTree) -> Result<()> {
        self.live
            .check_piece_resources(Some(old), next)
            .map(|_| ())
            .map_err(crate::live_error)
    }
    #[allow(clippy::too_many_arguments)]
    fn install_edit(
        &mut self,
        node: NodeId,
        old: PieceTree,
        next: PieceTree,
        edits: u32,
        high_water: u64,
        appended: u64,
        generation: u64,
        paths: Vec<String>,
    ) -> Result<()> {
        let revision = self.live.nodes[&node]
            .revision
            .checked_add(1)
            .ok_or(StoreError::Integrity("inode revision"))?;
        // A successful explicit empty state retires the prior logical edit
        // generation. Keep spool history for in-flight reads; only subsequent
        // mutations receive a fresh edit budget, after existing admission checks.
        let emptied = !old.is_empty() && next.is_empty();
        self.live.inline_bytes = self.live.inline_bytes - old.inline_len() + next.inline_len();
        self.live.piece_allocation_bytes = self.live.piece_allocation_bytes
            - old.logical_allocation_charge().map_err(crate::live_error)?
            + next
                .logical_allocation_charge()
                .map_err(crate::live_error)?;
        self.live.spool_bytes += appended;
        self.live.spool_bytes_peak = self.live.spool_bytes_peak.max(self.live.spool_bytes);
        let Data::File(FileData::Edited {
            pieces,
            edits: current_edits,
            spool_high_water,
            ..
        }) = &mut self.live.nodes.get_mut(&node).unwrap().data
        else {
            return Err(StoreError::Integrity("edited file"));
        };
        *pieces = next;
        *current_edits = if emptied { 0 } else { edits };
        *spool_high_water = high_water;
        self.live.nodes.get_mut(&node).unwrap().revision = revision;
        self.live.dirty.insert(node);
        self.live.mutation_generation = generation;
        for path in paths {
            self.live.mutation_paths.insert(path, generation);
        }
        Ok(())
    }

    pub fn fsync(&mut self, node: Option<NodeId>) -> Result<()> {
        let started = std::time::Instant::now();
        let mut segments = std::collections::BTreeMap::new();
        if let Some(node) = node {
            if let Data::File(FileData::Edited { pieces, .. }) = &self
                .live
                .nodes
                .get(&node)
                .ok_or(StoreError::NotFound("node"))?
                .data
            {
                for piece in pieces.pieces() {
                    if let Piece::Spool { segment, .. } = piece {
                        segments.insert(segment.id(), segment);
                    }
                }
            }
            for segment in segments.values() {
                spool_segment(segment)?.check()?;
                spool_segment(segment)?.observe();
            }
        } else {
            for segment in self.spool_segments.values() {
                spool_segment(segment)?.check()?;
                spool_segment(segment)?.observe();
            }
        }
        self.finish_capture(node);
        self.spool_write_metrics.fence_count =
            self.spool_write_metrics.fence_count.saturating_add(1);
        self.spool_write_metrics.fence_ns = self
            .spool_write_metrics
            .fence_ns
            .saturating_add(elapsed_ns(started));
        Ok(())
    }
    pub(crate) fn take_spool_write_metrics(&mut self) -> crate::cow_tree::SpoolWriteMetrics {
        std::mem::take(&mut self.spool_write_metrics)
    }
    pub(crate) fn clear_spool(&mut self) -> Result<()> {
        self.invalidate_capture();
        for id in &self.live.edited_nodes {
            if let Some(Node {
                data: Data::File(FileData::Edited { pieces, .. }),
                ..
            }) = self.live.nodes.get_mut(id)
            {
                *pieces = PieceTree::empty();
            }
        }
        self.live.edited_nodes.clear();
        self.current_spool = None;
        self.spool_segments.clear();
        self.segment_bytes = 0;
        self.live.spool_bytes = 0;
        self.live.spool_bytes_peak = 0;
        self.live.inline_bytes = 0;
        self.live.piece_allocation_bytes = 0;
        Ok(())
    }
    pub(crate) fn new_spool_node(&mut self, mode: u32, path: String) -> Result<NodeId> {
        let node = NodeId(self.next_node);
        self.new_spool_node_reserved_inner(node, mode, path, false)?;
        Ok(node)
    }
    pub(crate) fn new_spool_node_reserved(
        &mut self,
        node: NodeId,
        mode: u32,
        path: String,
    ) -> Result<()> {
        self.new_spool_node_reserved_inner(node, mode, path, true)
    }
    fn new_spool_node_reserved_inner(
        &mut self,
        node: NodeId,
        mode: u32,
        path: String,
        reserved: bool,
    ) -> Result<()> {
        let data = Data::File(FileData::Edited {
            base: None,
            spool_high_water: 0,
            pieces: PieceTree::empty(),
            edits: 0,
        });
        let value = Node {
            revision: 0,
            canonical: None,
            paths: [path].into(),
            mode,
            links: 1,
            pins: 0,
            mtime_seconds: 0,
            mtime_nanoseconds: 0,
            data,
        };
        if reserved {
            if self.live.nodes.insert(node, value).is_some() {
                return Err(StoreError::Integrity("reserved node"));
            }
        } else {
            let allocated = self.allocate(value);
            debug_assert_eq!(allocated, node);
        }
        self.live.edited_nodes.insert(node);
        Ok(())
    }
    fn ensure_edited(&mut self, node: NodeId) -> Result<()> {
        if let Data::File(FileData::Base { root, len }) = self.live.nodes[&node].data {
            let pieces = PieceTree::base(root, len).map_err(crate::live_error)?;
            let next_allocation = self
                .live
                .piece_allocation_bytes
                .checked_add(
                    pieces
                        .logical_allocation_charge()
                        .map_err(crate::live_error)?,
                )
                .filter(|v| *v <= MAX_PIECE_ALLOCATION)
                .ok_or(StoreError::InvalidInput("workspace piece allocation limit"))?;
            self.live.nodes.get_mut(&node).unwrap().data = Data::File(FileData::Edited {
                base: Some((root, len)),
                spool_high_water: 0,
                pieces,
                edits: 0,
            });
            self.live.piece_allocation_bytes = next_allocation;
            self.live.edited_nodes.insert(node);
        }
        matches!(
            self.live.nodes[&node].data,
            Data::File(FileData::Edited { .. })
        )
        .then_some(())
        .ok_or(StoreError::InvalidInput("file"))
    }
    pub(crate) fn physical_spool_snapshot(&self) -> (Option<u64>, Option<u64>, u64, u64) {
        self.physical_spool
            .lock()
            .map_or((None, None, 1, 0), |metrics| metrics.snapshot())
    }

    fn append_segment(&mut self, bytes: u64) -> Result<(BackingRef, u64)> {
        if self.segment_bytes.saturating_add(bytes) > self.live.policy.max_spool_bytes {
            self.retire_spool_segments();
        }
        self.live
            .policy
            .check(
                self.live
                    .spool_bytes
                    .max(self.segment_bytes)
                    .checked_add(bytes)
                    .ok_or(StoreError::InvalidInput("workspace spool limit"))?,
            )
            .map_err(crate::live_error)?;
        if let Some(segment) = self
            .current_spool
            .and_then(|id| self.spool_segments.get(&id))
        {
            let physical = spool_segment(segment)?;
            let offset = physical.len.load(Ordering::Relaxed);
            if offset.saturating_add(bytes) <= physical.capacity {
                return Ok((segment.clone(), offset));
            }
        }
        self.retire_spool_segments();
        let started = std::time::Instant::now();
        let id = self.next_spool;
        self.next_spool = id
            .checked_add(1)
            .ok_or(StoreError::Integrity("spool segment identity"))?;
        let segment = BackingRef::new(
            BackingId(id),
            SpoolSegment::new(
                &self.spool,
                id,
                SPOOL_SEGMENT_BYTES
                    .max(bytes)
                    .min(self.live.policy.max_spool_bytes),
                self.physical_spool.clone(),
            )?,
        );
        self.spool_segments.insert(id, segment.clone());
        self.current_spool = Some(id);
        self.note_spool_open(elapsed_ns(started));
        Ok((segment, 0))
    }

    pub(crate) fn retire_spool_segments(&mut self) {
        let started = std::time::Instant::now();
        let mut scan_ns = 0_u64;
        let mut retired = 0_u64;
        self.spool_segments.retain(|id, segment| {
            let scan_started = std::time::Instant::now();
            let keep = !segment.is_unique();
            if !keep {
                self.segment_bytes = self.segment_bytes.saturating_sub(
                    spool_segment(segment)
                        .expect("host segment registry")
                        .len
                        .load(Ordering::Relaxed),
                );
                if self.current_spool == Some(*id) {
                    self.current_spool = None;
                }
                retired += 1;
            }
            scan_ns = scan_ns.saturating_add(elapsed_ns(scan_started));
            keep
        });
        layerfs_layerstack_store::note_workspace_spool_retirement(
            elapsed_ns(started),
            scan_ns,
            retired,
        );
    }
    fn note_spool_open(&mut self, ns: u64) {
        self.spool_write_metrics.write_open_count =
            self.spool_write_metrics.write_open_count.saturating_add(1);
        self.spool_write_metrics.write_ns = self.spool_write_metrics.write_ns.saturating_add(ns);
    }
}

fn next_edit(edits: u32) -> Result<u32> {
    crate::file_edit::next_edit(edits).map_err(crate::live_error)
}
fn elapsed_ns(started: std::time::Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}
fn append_spool(file: &File, bytes: &[u8], offset: u64) -> std::io::Result<()> {
    #[cfg(test)]
    if INJECT_SHORT_APPEND.with(|inject| inject.replace(false)) {
        file.write_all_at(&bytes[..bytes.len() / 2], offset)?;
        return Err(std::io::Error::other("injected short spool append"));
    }
    file.write_all_at(bytes, offset)
}

#[cfg(test)]
thread_local! {
    static INJECT_SHORT_APPEND: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static INJECT_APPEND_CLEANUP_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static INJECT_STALE_WRITE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
fn read_base(
    reader: &SnapshotReader,
    root: layerfs_content::file::rope::FileStateRoot,
    start: u64,
    end: u64,
) -> Result<Vec<u8>> {
    if start >= end {
        return Ok(Vec::new());
    }
    let mut bytes = Vec::with_capacity(as_usize(end - start)?);
    let counters = read_range(&CoreReader(reader), root, start..end, &mut bytes)?;
    reader.note_rope_read(counters)?;
    Ok(bytes)
}
fn read_exact_at(file: &File, mut output: &mut [u8], mut offset: u64) -> Result<()> {
    while !output.is_empty() {
        let read = file.read_at(output, offset)?;
        if read == 0 {
            return Err(StoreError::Integrity("spool eof"));
        }
        offset += read as u64;
        output = &mut output[read..];
    }
    Ok(())
}

fn as_usize(value: u64) -> Result<usize> {
    usize::try_from(value).map_err(|_| StoreError::InvalidInput("file range"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ROOT;
    use layerfs_layerstack_store::{
        EntityName, LayerStackInitialization, LayerStackStore, LocalForkSource,
    };

    #[test]
    fn stale_prepared_append_is_not_repeated_and_keeps_its_physical_charge() {
        let (root, mut workspace) = workspace("stale-prepared-append");
        let file = workspace.create_file(ROOT, b"file", 0o600).unwrap().node;
        workspace.live.policy.max_spool_bytes = 9;
        workspace.write(file, 0, b"hello").unwrap();
        let held = workspace.read_plan(file, 0, 5).unwrap();
        INJECT_STALE_WRITE.with(|inject| inject.set(true));
        assert!(matches!(
            workspace.write(file, 0, b"bad"),
            Err(StoreError::Integrity("stale prepared write"))
        ));
        assert_eq!(workspace.read(file, 0, 5).unwrap(), b"hello");
        assert_eq!(workspace.live.spool_bytes, 5);
        assert_eq!(workspace.segment_bytes, 8);
        assert_eq!(workspace.spool_write_metrics.write_bytes, 8);
        assert!(workspace.write(file, 0, b"no").is_err());
        workspace.commit().unwrap();
        assert_eq!(
            workspace.segment_bytes, 8,
            "old reader retains the whole segment"
        );
        assert_eq!(held.read().unwrap(), b"hello");
        workspace.commit().unwrap();
        assert_eq!(workspace.segment_bytes, 0);
        workspace.write(file, 0, b"ok").unwrap();
        assert_eq!(workspace.read(file, 0, 5).unwrap(), b"okllo");
        workspace.discard().unwrap();
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn workspace(label: &str) -> (std::path::PathBuf, Workspace) {
        let root = std::env::temp_dir().join(format!(
            "layerfs-file-edit-{label}-{}-{}",
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
                EntityName::new("main").unwrap(),
                LocalForkSource::Layer { layer_id: layer },
            )
            .unwrap();
        let workspace = Workspace::open(store, branch, root.join("spool")).unwrap();
        (root, workspace)
    }

    fn workspace_with_file(
        label: &str,
    ) -> (
        std::path::PathBuf,
        Workspace,
        layerfs_layerstack_store::BranchId,
    ) {
        let root = std::env::temp_dir().join(format!(
            "layerfs-file-edit-source-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = root.join("source");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("file"), b"abcdefghij").unwrap();
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
        let workspace = Workspace::open(store, branch, root.join("spool")).unwrap();
        (root, workspace, branch)
    }

    #[test]
    fn failed_tail_cleanup_keeps_discarded_physical_bytes_charged() {
        let (root, mut workspace) = workspace("failed-tail-cleanup");
        workspace.live.policy.max_spool_bytes = 6;
        let file = workspace.create_file(ROOT, b"file", 0o600).unwrap().node;
        workspace.write(file, 0, b"ok").unwrap();
        INJECT_SHORT_APPEND.with(|inject| inject.set(true));
        INJECT_APPEND_CLEANUP_FAILURE.with(|inject| inject.set(true));
        assert!(matches!(
            workspace.write(file, 2, b"bad!"),
            Err(StoreError::Integrity("spool append cleanup failure"))
        ));
        assert_eq!(workspace.read(file, 0, 2).unwrap(), b"ok");
        assert_eq!(workspace.live.spool_bytes, 2);
        assert_eq!(workspace.segment_bytes, 4);
        workspace.live.policy.max_spool_bytes = 4;
        assert!(workspace.write(file, 2, b"x").is_err());
        workspace.commit().unwrap();
        assert_eq!(workspace.segment_bytes, 0);
        workspace.write(file, 2, b"x").unwrap();
        assert_eq!(workspace.read(file, 0, 3).unwrap(), b"okx");
        workspace.discard().unwrap();
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shared_segments_keep_interleaved_rollback_reads_and_open_unlinked_lifetime() {
        let (root, mut workspace) = workspace("shared-segment-lifetime");
        let a = workspace.create_file(ROOT, b"a", 0o600).unwrap().node;
        let b = workspace.create_file(ROOT, b"b", 0o600).unwrap().node;
        workspace.write(a, 0, b"abc").unwrap();
        workspace.write(b, 0, b"XYZ").unwrap();
        workspace.write(a, 3, b"def").unwrap();
        assert_eq!(workspace.spool_segments.len(), 1);
        let held = workspace.read_plan(a, 0, 6).unwrap();
        workspace.write(a, 0, b"Q").unwrap();
        workspace.truncate(b, 2).unwrap();
        workspace.write(b, 5, b"R").unwrap();
        let end = workspace.segment_bytes;
        INJECT_SHORT_APPEND.with(|inject| inject.set(true));
        assert!(workspace.write(a, 6, b"failed tail").is_err());
        assert_eq!(workspace.segment_bytes, end);
        assert_eq!(workspace.read(a, 0, 6).unwrap(), b"Qbcdef");
        assert_eq!(workspace.read(b, 0, 6).unwrap(), b"XY\0\0\0R");
        workspace.pin(a, false).unwrap();
        workspace.unlink(ROOT, b"a", false).unwrap();
        let c = workspace.create_file(ROOT, b"c", 0o600).unwrap().node;
        workspace
            .write(c, 0, &vec![7; SPOOL_SEGMENT_BYTES as usize])
            .unwrap();
        assert_eq!(workspace.spool_segments.len(), 2);
        workspace.fsync(None).unwrap();
        workspace.commit().unwrap();
        assert_eq!(workspace.spool_segments.len(), 1);
        assert_eq!(
            workspace.segment_bytes, end,
            "retained segment dead bytes remain charged"
        );
        assert_eq!(workspace.read(a, 0, 6).unwrap(), b"Qbcdef");
        workspace.chmod(c, 0o640).unwrap();
        workspace.commit().unwrap();
        workspace.unpin(a).unwrap();
        assert_eq!(
            workspace.spool_segments.len(),
            1,
            "prepared read owns old extents"
        );
        assert_eq!(held.read().unwrap(), b"abcdef");
        workspace.commit().unwrap();
        assert!(workspace.spool_segments.is_empty());
        assert_eq!(workspace.segment_bytes, 0);
        assert_eq!(workspace.physical_spool_snapshot().0, Some(0));
        workspace.end_clean().unwrap();
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn edit_limit_and_sparse_physical_charge_are_exact() {
        let (root, mut workspace) = workspace("limits");
        let sparse = workspace.create_file(ROOT, b"sparse", 0o600).unwrap().node;
        workspace.write(sparse, 60 * 1024, b"x").unwrap();
        assert_eq!(workspace.attr(sparse).unwrap().size, 60 * 1024 + 1);
        assert_eq!(workspace.live.spool_bytes, 1);
        assert_eq!(
            workspace
                .spool_segments
                .values()
                .map(|segment| spool_segment(segment)
                    .unwrap()
                    .file
                    .metadata()
                    .unwrap()
                    .len())
                .sum::<u64>(),
            1
        );

        let file = workspace.create_file(ROOT, b"limit", 0o600).unwrap().node;
        for value in 0..MAX_EDITS_PER_FILE {
            workspace.write(file, 0, &[(value & 0xff) as u8]).unwrap();
        }
        let before = workspace.read(file, 0, 1).unwrap();
        assert!(workspace.write(file, 0, b"x").is_err());
        assert_eq!(workspace.read(file, 0, 1).unwrap(), before);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn short_spool_append_restores_high_water_and_piece_root() {
        let (root, mut workspace) = workspace("short-append");
        let file = workspace.create_file(ROOT, b"file", 0o600).unwrap().node;
        workspace.write(file, 0, b"base").unwrap();
        let before = workspace.live.nodes[&file].clone();
        let before_charge = workspace.live.spool_bytes;
        let before_peak = workspace.live.spool_bytes_peak;
        INJECT_SHORT_APPEND.with(|inject| inject.set(true));
        assert!(workspace.write(file, 4, b"failure").is_err());
        assert_eq!(workspace.live.nodes[&file], before);
        assert_eq!(workspace.live.spool_bytes, before_charge);
        assert_eq!(workspace.live.spool_bytes_peak, before_peak);
        assert_eq!(
            workspace
                .spool_segments
                .values()
                .map(|segment| spool_segment(segment)
                    .unwrap()
                    .file
                    .metadata()
                    .unwrap()
                    .len())
                .sum::<u64>(),
            before_charge
        );
        assert_eq!(workspace.read(file, 0, 16).unwrap(), b"base");
        workspace.live.policy.max_spool_bytes = before_charge;
        assert!(workspace.write(file, 4, b"x").is_err());
        assert_eq!(workspace.live.spool_bytes, before_charge);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn physical_spool_peak_survives_short_append_reclaim_and_discard() {
        let (root, mut workspace) = workspace("physical-spool-accounting");
        let first = workspace.create_file(ROOT, b"first", 0o600).unwrap().node;
        let second = workspace.create_file(ROOT, b"second", 0o600).unwrap().node;
        workspace.write(first, 0, b"a").unwrap();
        workspace.write(second, 0, &[0x53; 8192]).unwrap();
        let actual = || {
            workspace
                .spool_segments
                .values()
                .map(|f| spool_segment(f).unwrap().file.metadata().unwrap().blocks() * 512)
                .sum::<u64>()
        };
        assert_eq!(workspace.physical_spool_snapshot().0, Some(actual()));
        assert_eq!(
            workspace.spool_segments.len(),
            1,
            "two files share one physical segment"
        );
        let initial_peak = workspace.physical_spool_snapshot().1.unwrap();
        let failed = workspace.create_file(ROOT, b"failed", 0o600).unwrap().node;
        let logical_before = (workspace.live.spool_bytes, workspace.live.spool_bytes_peak);
        INJECT_SHORT_APPEND.with(|inject| inject.set(true));
        assert!(workspace.write(failed, 0, &vec![0xa5; 256 * 1024]).is_err());
        assert_eq!(
            (workspace.live.spool_bytes, workspace.live.spool_bytes_peak),
            logical_before
        );
        assert_eq!(workspace.attr(failed).unwrap().size, 0);
        assert_eq!(
            workspace
                .spool_segments
                .values()
                .map(|segment| spool_segment(segment)
                    .unwrap()
                    .file
                    .metadata()
                    .unwrap()
                    .len())
                .sum::<u64>(),
            logical_before.0
        );
        let (current, peak, errors, count) = workspace.physical_spool_snapshot();
        let actual = workspace
            .spool_segments
            .values()
            .map(|f| spool_segment(f).unwrap().file.metadata().unwrap().blocks() * 512)
            .sum::<u64>();
        assert_eq!(current, Some(actual));
        assert!(
            peak.unwrap() > initial_peak,
            "partial append allocation must be observed before truncation"
        );
        assert_eq!(errors, 0);
        assert!(count > 0);
        let capture = layerfs_layerstack_store::capture_workspace_commit_diagnostics().unwrap();
        {
            let _timer = layerfs_layerstack_store::begin_workspace_commit(
                layerfs_layerstack_store::CaptureMode::Materialized,
            )
            .unwrap();
            workspace.note_commit_edit_state().unwrap();
        }
        let diagnostic = layerfs_layerstack_store::take_workspace_commit_diagnostics()
            .pop()
            .unwrap();
        assert_eq!(diagnostic.physical_spool_allocated_bytes, current);
        assert_eq!(diagnostic.physical_spool_peak_bytes, peak);
        assert_eq!(diagnostic.physical_spool_observation_errors, 0);
        drop(capture);

        workspace.pin(first, false).unwrap();
        workspace.unlink(ROOT, b"first", false).unwrap();
        assert_eq!(
            workspace.physical_spool_snapshot().0,
            current,
            "open unlinked spool stays charged"
        );
        workspace.unpin(first).unwrap();
        assert_eq!(workspace.physical_spool_snapshot().0, Some(actual));
        assert_eq!(workspace.physical_spool_snapshot().1, peak);

        workspace.commit().unwrap();
        assert_eq!(workspace.physical_spool_snapshot().0, Some(0));
        assert_eq!(
            workspace.physical_spool_snapshot().1,
            peak,
            "Commit checkpoint keeps lifetime allocation evidence"
        );
        let last = workspace.create_file(ROOT, b"last", 0o600).unwrap().node;
        workspace.write(last, 0, b"later").unwrap();
        workspace.discard().unwrap();
        assert_eq!(workspace.physical_spool_snapshot().0, Some(0));
        assert_eq!(workspace.physical_spool_snapshot().1, peak);
        workspace.physical_spool.lock().unwrap().error();
        assert_eq!(workspace.physical_spool_snapshot().0, None);
        assert_eq!(workspace.physical_spool_snapshot().1, None);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workspace_inline_limit_accepts_eight_mib_and_rejects_the_next_byte() {
        let (root, mut workspace) = workspace("inline-limit");
        for index in 0..8 {
            let name = format!("file-{index}");
            let file = workspace
                .create_file(ROOT, name.as_bytes(), 0o600)
                .unwrap()
                .node;
            workspace
                .edit_many(
                    file,
                    vec![(
                        0,
                        0,
                        crate::WorkspaceFileReplacement::Inline(vec![index; 1024 * 1024]),
                    )],
                )
                .unwrap();
        }
        assert_eq!(workspace.live.inline_bytes, MAX_INLINE_PER_WORKSPACE);
        let extra = workspace.create_file(ROOT, b"extra", 0o600).unwrap().node;
        assert!(workspace
            .edit_many(
                extra,
                vec![(0, 0, crate::WorkspaceFileReplacement::Inline(vec![0]),)],
            )
            .is_err());
        assert_eq!(workspace.live.inline_bytes, MAX_INLINE_PER_WORKSPACE);
        assert_eq!(workspace.attr(extra).unwrap().size, 0);
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generation_overflow_rejects_without_logical_or_physical_change() {
        let (root, mut workspace, _) = workspace_with_file("generation-overflow");
        let file = workspace.lookup(ROOT, b"file").unwrap().node;
        workspace.live.mutation_generation = u64::MAX;
        let before = workspace.live.nodes[&file].clone();
        let charges = (
            workspace.live.spool_bytes,
            workspace.live.inline_bytes,
            workspace.live.piece_allocation_bytes,
        );
        assert!(workspace
            .edit_many(
                file,
                vec![(0, 0, crate::WorkspaceFileReplacement::Inline(b"P".to_vec()),)],
            )
            .is_err());
        assert_eq!(workspace.live.nodes[&file], before);
        assert_eq!(
            (
                workspace.live.spool_bytes,
                workspace.live.inline_bytes,
                workspace.live.piece_allocation_bytes,
            ),
            charges
        );
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discard_reclaims_one_live_base_inline_zero_spool_composition() {
        let (root, mut workspace, branch) = workspace_with_file("mixed-discard");
        let branch_root = workspace.store.pin_branch(branch).unwrap().root;
        let file = workspace.lookup(ROOT, b"file").unwrap().node;
        workspace
            .edit_many(
                file,
                vec![
                    (1, 2, crate::WorkspaceFileReplacement::Inline(b"X".to_vec())),
                    (2, 0, crate::WorkspaceFileReplacement::Zero(2)),
                ],
            )
            .unwrap();
        let end = workspace.attr(file).unwrap().size;
        workspace.write(file, end, b"S").unwrap();
        let Data::File(FileData::Edited { pieces, .. }) = &workspace.live.nodes[&file].data else {
            panic!("edited file")
        };
        let variants = pieces.pieces();
        assert!(variants
            .iter()
            .any(|piece| matches!(piece, Piece::Base { .. })));
        assert!(variants
            .iter()
            .any(|piece| matches!(piece, Piece::Inline { .. })));
        assert!(variants
            .iter()
            .any(|piece| matches!(piece, Piece::Zero { .. })));
        assert!(variants
            .iter()
            .any(|piece| matches!(piece, Piece::Spool { .. })));
        drop(variants);
        assert_eq!(workspace.spool_segments.len(), 1);
        workspace.discard().unwrap();
        assert_eq!(
            workspace.store.pin_branch(branch).unwrap().root,
            branch_root
        );
        assert_eq!(workspace.live.spool_bytes, 0);
        assert_eq!(workspace.live.inline_bytes, 0);
        assert_eq!(workspace.live.piece_allocation_bytes, 0);
        assert!(workspace.spool_segments.is_empty());
        assert_eq!(workspace.physical_spool_snapshot().0, Some(0));
        drop(workspace);
        std::fs::remove_dir_all(root).unwrap();
    }
}

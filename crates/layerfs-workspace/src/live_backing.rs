//! Host-only physical backing and immutable input for a remote live owner.
use crate::cow_tree::{acquire_inode, WorkspaceSnapshot};
use crate::file_io::{spool_segment, HostSpool};
use crate::ResourcePolicy;
use layerfs_content::file::rope::{read_range, FileStateRoot};
use layerfs_content::tree::directory::{directory_lookup, DirectoryStateRoot, NamespaceCounters};
use layerfs_content::tree::inode::InodeTableRoot;
use layerfs_content::CanonicalName;
use layerfs_fuse::live_wire::{self as wire, Input};
use layerfs_layerstack_store::{CoreReader, Result, StoreError};
use layerfs_workspace_core::backing::{BackingId, BackingRef};
use layerfs_workspace_core::{Node, NodeId};
use std::collections::{BTreeSet, HashMap};
use std::os::unix::fs::FileExt;
use std::path::PathBuf;

pub(crate) struct BackingOwner {
    pub(crate) snapshot: WorkspaceSnapshot,
    pub(crate) root: Node,
    pub(crate) spool: HostSpool,
    pub(crate) directory: PathBuf,
    pub(crate) policy: ResourcePolicy,
    // Remote references and acknowledged facts keep physical ranges owned.
    retained: HashMap<BackingId, BackingRef>,
    pub(crate) facts: HashMap<NodeId, Node>,
    pub(crate) dirty: BTreeSet<NodeId>,
    pub(crate) generation: u64,
    incoming: Option<(u64, HashMap<NodeId, Node>, BTreeSet<NodeId>)>,
}

impl BackingOwner {
    pub(crate) fn new(
        snapshot: WorkspaceSnapshot,
        root: Node,
        directory: PathBuf,
        policy: ResourcePolicy,
    ) -> Self {
        Self {
            snapshot,
            root,
            spool: HostSpool::default(),
            directory,
            policy,
            retained: HashMap::new(),
            facts: HashMap::new(),
            dirty: BTreeSet::new(),
            generation: 0,
            incoming: None,
        }
    }

    pub(crate) fn request(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
        let mut input = Input(bytes);
        let mut out = Vec::new();
        match input.byte()? {
            wire::SEED => {
                input.done()?;
                out.extend_from_slice(self.snapshot.root.as_bytes());
                wire::u64_out(&mut out, self.policy.max_spool_bytes);
                wire::u64_out(&mut out, self.policy.max_final_delta_memory_bytes);
                out.extend(wire::node_out(layerfs_workspace_core::ROOT, &self.root)?);
            }
            wire::LOOKUP => {
                let namespace = input.object()?;
                let directory = DirectoryStateRoot(input.object()?);
                let name = CanonicalName::from_bytes(input.bytes()?)?;
                input.done()?;
                if namespace != self.snapshot.root {
                    return Err(StoreError::Integrity("backing namespace"));
                }
                let core = CoreReader(&self.snapshot.reader);
                let namespace = layerfs_content::filesystem::namespace(&core, namespace)?;
                let inode =
                    directory_lookup(&core, directory, &name, &mut NamespaceCounters::default())?;
                out.push(u8::from(inode.is_some()));
                if let Some(inode) = inode {
                    let acquired = acquire_inode(
                        &self.snapshot.reader,
                        InodeTableRoot(namespace.inode_table_root),
                        inode,
                    )?;
                    let node = Node {
                        revision: 0,
                        canonical: Some(inode),
                        paths: BTreeSet::new(),
                        mode: acquired.mode,
                        links: acquired.links,
                        pins: 0,
                        mtime_seconds: acquired.mtime_seconds,
                        mtime_nanoseconds: acquired.mtime_nanoseconds,
                        data: acquired.data,
                    };
                    out.extend(wire::node_out(NodeId(1), &node)?);
                }
            }
            wire::RESERVE => {
                let len = input.u64()?;
                input.done()?;
                if len == 0 || len > 1024 * 1024 {
                    return Err(StoreError::InvalidInput("backing reservation"));
                }
                let (segment, offset) =
                    self.spool
                        .reserve_append(&self.directory, len, 0, self.policy)?;
                let capacity = spool_segment(&segment)?.capacity;
                wire::u64_out(&mut out, segment.id().0);
                wire::u64_out(&mut out, offset);
                wire::u64_out(&mut out, capacity);
                self.retained.insert(segment.id(), segment);
            }
            wire::APPEND => {
                let id = BackingId(input.u64()?);
                let offset = input.u64()?;
                let data = input.bytes()?;
                input.done()?;
                if data.len() > 1024 * 1024 {
                    return Err(StoreError::InvalidInput("backing append"));
                }
                let segment = self
                    .retained
                    .get(&id)
                    .ok_or(StoreError::NotFound("backing"))?
                    .clone();
                self.spool.append(
                    &segment,
                    offset,
                    data,
                    #[cfg(feature = "test-instrumentation")]
                    false,
                )?;
            }
            wire::READ_BACKING => {
                let id = BackingId(input.u64()?);
                let offset = input.u64()?;
                let len = input.u32()? as usize;
                input.done()?;
                if len > 1024 * 1024 {
                    return Err(StoreError::InvalidInput("backing read"));
                }
                let segment = self
                    .retained
                    .get(&id)
                    .or_else(|| self.spool.segments.get(&id.0))
                    .ok_or(StoreError::NotFound("backing"))?;
                let physical = spool_segment(segment)?;
                if offset
                    .checked_add(len as u64)
                    .is_none_or(|end| end > physical.len.load(std::sync::atomic::Ordering::Relaxed))
                {
                    return Err(StoreError::Integrity("backing read range"));
                }
                out.resize(len, 0);
                physical.file.read_exact_at(&mut out, offset)?;
            }
            wire::READ_BASE => {
                let root = FileStateRoot(input.object()?);
                let offset = input.u64()?;
                let len = input.u32()? as usize;
                input.done()?;
                if len > 1024 * 1024 {
                    return Err(StoreError::InvalidInput("immutable read"));
                }
                read_range(
                    &CoreReader(&self.snapshot.reader),
                    root,
                    offset
                        ..offset
                            .checked_add(len as u64)
                            .ok_or(StoreError::InvalidInput("immutable range"))?,
                    &mut out,
                )?;
            }
            wire::CHECK => {
                while !input.0.is_empty() {
                    let id = input.u64()?;
                    spool_segment(
                        self.spool
                            .segments
                            .get(&id)
                            .ok_or(StoreError::NotFound("backing"))?,
                    )?
                    .check()?;
                }
            }
            wire::RELEASE => {
                while !input.0.is_empty() {
                    self.retained.remove(&BackingId(input.u64()?));
                }
                self.spool.retire();
            }
            wire::FACTS_BEGIN => {
                let generation = input.u64()?;
                input.done()?;
                if self.incoming.is_some() {
                    return Err(StoreError::Integrity("unfinished backing group"));
                }
                self.incoming = Some((generation, HashMap::new(), BTreeSet::new()));
            }
            wire::FACTS_NODE => {
                while !input.0.is_empty() {
                    let dirty = match input.byte()? {
                        0 => false,
                        1 => true,
                        _ => return Err(StoreError::Integrity("backing dirty flag")),
                    };
                    let (id, node) = wire::node_in(input.bytes()?, |id, offset, len| {
                        let reference = self
                            .retained
                            .get(&id)
                            .or_else(|| self.spool.segments.get(&id.0))
                            .ok_or_else(wire::invalid)?;
                        let segment = spool_segment(reference).map_err(|_| wire::invalid())?;
                        if offset.checked_add(len).is_none_or(|end| {
                            end > segment.len.load(std::sync::atomic::Ordering::Relaxed)
                        }) {
                            return Err(wire::invalid());
                        }
                        Ok(reference.clone())
                    })?;
                    let (_, nodes, changed) = self
                        .incoming
                        .as_mut()
                        .ok_or(StoreError::Integrity("missing backing group"))?;
                    nodes.insert(id, node);
                    if dirty {
                        changed.insert(id);
                    }
                }
            }
            wire::FACTS_END => {
                let generation = input.u64()?;
                let count = input.u64()?;
                input.done()?;
                let (prepared, nodes, dirty) = self
                    .incoming
                    .take()
                    .ok_or(StoreError::Integrity("missing backing group"))?;
                if prepared != generation || nodes.len() as u64 != count {
                    return Err(StoreError::Integrity("incomplete backing group"));
                }
                for id in &dirty {
                    if let layerfs_workspace_core::Data::Directory(directory) = &nodes[id].data {
                        if directory
                            .changes
                            .values()
                            .flatten()
                            .any(|id| !nodes.contains_key(id))
                        {
                            return Err(StoreError::Integrity("backing group child"));
                        }
                    }
                }
                self.facts = nodes;
                self.dirty = dirty;
                self.generation = generation;
            }
            _ => return Err(StoreError::InvalidInput("backing request")),
        }
        Ok(out)
    }
}

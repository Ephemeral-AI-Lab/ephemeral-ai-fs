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
    append_reservation: Option<(BackingId, u64, u64)>,
    pub(crate) facts: HashMap<NodeId, Node>,
    pub(crate) dirty: BTreeSet<NodeId>,
    pub(crate) generation: u64,
    fact_reservations: Vec<layerfs_fuse::live_runtime::LiveReservation>,
    incoming_reservations: Vec<layerfs_fuse::live_runtime::LiveReservation>,
    incoming_charge: usize,
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
            append_reservation: None,
            facts: HashMap::new(),
            dirty: BTreeSet::new(),
            generation: 0,
            incoming: None,
            fact_reservations: Vec::new(),
            incoming_reservations: Vec::new(),
            incoming_charge: 0,
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
            wire::DIRECTORY_PAGE => {
                let namespace = input.object()?;
                let directory = DirectoryStateRoot(input.object()?);
                let after = input.bytes()?;
                let after = if after.is_empty() {
                    None
                } else {
                    Some(CanonicalName::from_bytes(after)?)
                };
                input.done()?;
                if namespace != self.snapshot.root {
                    return Err(StoreError::Integrity("backing namespace"));
                }
                let core = CoreReader(&self.snapshot.reader);
                let namespace = layerfs_content::filesystem::namespace(&core, namespace)?;
                let page = layerfs_content::tree::directory::directory_page_after(
                    &core,
                    directory,
                    after.as_ref(),
                    128,
                    256 * 1024,
                    &mut NamespaceCounters::default(),
                )?;
                out.push(u8::from(page.continuation.is_some()));
                out.extend_from_slice(&(page.entries.len() as u32).to_be_bytes());
                let ids = page
                    .entries
                    .iter()
                    .map(|(_, inode)| *inode)
                    .collect::<Vec<_>>();
                let acquired = crate::cow_tree::acquire_inodes(
                    &self.snapshot.reader,
                    InodeTableRoot(namespace.inode_table_root),
                    &ids,
                )?;
                for ((name, _), acquired) in page.entries.into_iter().zip(acquired) {
                    wire::bytes_out(&mut out, name.as_bytes())?;
                    wire::bytes_out(
                        &mut out,
                        &wire::node_out(
                            NodeId(1),
                            &Node {
                                revision: 0,
                                canonical: Some(acquired.inode),
                                paths: BTreeSet::new(),
                                mode: acquired.mode,
                                links: acquired.links,
                                pins: 0,
                                mtime_seconds: acquired.mtime_seconds,
                                mtime_nanoseconds: acquired.mtime_nanoseconds,
                                data: acquired.data,
                            },
                        )?,
                    )?;
                }
            }
            wire::RESERVE => {
                let len = input.u64()?;
                input.done()?;
                if len == 0 || len > 1024 * 1024 {
                    return Err(StoreError::InvalidInput("backing reservation"));
                }
                if self.append_reservation.is_some() {
                    return Err(StoreError::Integrity("unfinished backing reservation"));
                }
                let (segment, offset) =
                    self.spool
                        .reserve_append(&self.directory, len, 0, self.policy)?;
                let capacity = spool_segment(&segment)?.capacity;
                wire::u64_out(&mut out, segment.id().0);
                wire::u64_out(&mut out, offset);
                wire::u64_out(&mut out, capacity);
                self.append_reservation = Some((segment.id(), offset, len));
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
                let remaining = match self.append_reservation {
                    Some((held, start, remaining))
                        if held == id
                            && start == offset
                            && !data.is_empty()
                            && data.len() as u64 <= remaining =>
                    {
                        remaining
                    }
                    _ => return Err(StoreError::Integrity("backing append reservation")),
                };
                let segment = self
                    .retained
                    .get(&id)
                    .ok_or(StoreError::NotFound("backing"))?
                    .clone();
                // Consume before physical I/O: an uncertain/failed append cannot
                // be replayed. Earlier acknowledged ranges stay retained.
                self.append_reservation = None;
                self.spool.append(
                    &segment,
                    offset,
                    data,
                    #[cfg(feature = "test-instrumentation")]
                    false,
                )?;
                let remaining = remaining - data.len() as u64;
                self.append_reservation =
                    (remaining != 0).then_some((id, offset + data.len() as u64, remaining));
            }
            wire::CANCEL_RESERVATION => {
                let id = BackingId(input.u64()?);
                let offset = input.u64()?;
                input.done()?;
                if !self
                    .append_reservation
                    .is_some_and(|held| held.0 == id && held.1 == offset)
                {
                    return Err(StoreError::Integrity("backing cancellation reservation"));
                }
                self.append_reservation = None;
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
                let started = std::time::Instant::now();
                let synchronize = match input.byte()? {
                    0 => false,
                    1 => true,
                    _ => return Err(StoreError::InvalidInput("backing check")),
                };
                while !input.0.is_empty() {
                    let id = input.u64()?;
                    let segment = spool_segment(
                        self.spool
                            .segments
                            .get(&id)
                            .ok_or(StoreError::NotFound("backing"))?,
                    )?;
                    segment.check()?;
                    if synchronize {
                        segment.observe();
                    }
                }
                if synchronize {
                    self.spool.metrics.fence_count += 1;
                    self.spool.metrics.fence_ns =
                        self.spool.metrics.fence_ns.saturating_add(
                            started.elapsed().as_nanos().min(u64::MAX as u128) as u64,
                        );
                }
            }
            wire::RELEASE => {
                while !input.0.is_empty() {
                    let id = BackingId(input.u64()?);
                    if self.append_reservation.is_some_and(|held| held.0 == id) {
                        self.append_reservation = None;
                    }
                    self.retained.remove(&id);
                }
                self.spool.retire();
            }
            wire::FACTS_BEGIN => {
                let generation = input.u64()?;
                input.done()?;
                if self.incoming.is_some() {
                    return Err(StoreError::Integrity("unfinished backing group"));
                }
                self.incoming_charge = 0;
                self.incoming_reservations.clear();
                self.incoming = Some((generation, HashMap::new(), BTreeSet::new()));
            }
            wire::FACTS_NODE => {
                let mut count = 0;
                while !input.0.is_empty() {
                    if count == wire::FACT_PAGE_NODES {
                        return Err(StoreError::InvalidInput("backing fact page"));
                    }
                    count += 1;
                    let dirty = match input.byte()? {
                        0 => false,
                        1 => true,
                        _ => return Err(StoreError::Integrity("backing dirty flag")),
                    };
                    let encoded = input.bytes()?;
                    let charge = encoded
                        .len()
                        .checked_mul(8)
                        .and_then(|n| n.checked_add(1024))
                        .ok_or(StoreError::InvalidInput("backing fact limit"))?;
                    let total = self
                        .incoming_charge
                        .checked_add(charge)
                        .filter(|n| *n <= 16 * 1024 * 1024)
                        .ok_or(StoreError::InvalidInput("backing fact limit"))?;
                    let reservation = layerfs_fuse::live_runtime::LiveRuntime::shared()?
                        .scheduler()
                        .reserve_live(charge)?;
                    let (id, node) = wire::node_in(encoded, |id, offset, len| {
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
                    if nodes.contains_key(&id) {
                        return Err(StoreError::Integrity("duplicate backing fact"));
                    }
                    nodes.insert(id, node);
                    self.incoming_charge = total;
                    self.incoming_reservations.push(reservation);
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
                self.fact_reservations = std::mem::take(&mut self.incoming_reservations);
                self.dirty = dirty;
                self.generation = generation;
            }
            _ => return Err(StoreError::InvalidInput("backing request")),
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layerfs_layerstack_store::{
        EntityName, LayerStackInitialization, LayerStackStore, LocalForkSource,
    };

    #[test]
    fn live_owner_builds_and_checkpoints_through_real_host_backing() {
        use layerfs_fuse::{live_owner::LiveOwner, live_runtime::LiveRuntime, FilesystemPort};
        use std::sync::Mutex;
        let directory = std::env::temp_dir().join(format!(
            "layerfs-live-integrated-{}",
            crate::WorkspaceId::new()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let store = LayerStackStore::create(directory.join("store.sqlite")).unwrap();
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
        let mut workspace = crate::Workspace::open(store, branch, directory.join("spool")).unwrap();
        let remote = RemoteWorkspace::start(&workspace).unwrap();
        let runtime = LiveRuntime::shared().unwrap();
        let endpoint = format!("127.0.0.1:{}", remote.server.port());
        let owner = runtime
            .block_on(LiveOwner::connect(
                endpoint.clone(),
                remote.server.capability(),
                runtime.scheduler(),
            ))
            .unwrap();
        let control = owner
            .serve_control(endpoint, remote.server.capability())
            .unwrap();
        workspace.remote = Some(remote.clone());
        let workspace = Mutex::new(workspace);
        let directory_id = owner.mkdir(crate::ROOT, b"bulk", 0o755).unwrap().node;
        let file = owner
            .create_file_open(directory_id, b"file", 0o600)
            .unwrap()
            .node;
        owner.write(file, 0, b"first").unwrap();
        assert_eq!(owner.read(file, 0, 100).unwrap(), b"first");
        owner.fsync(None).unwrap();
        assert_eq!(remote.backing.lock().unwrap().generation, 3);
        for index in 0..130 {
            let id = owner
                .create_file(directory_id, format!("sibling-{index}").as_bytes(), 0o600)
                .unwrap()
                .node;
            owner
                .write(id, 0, b"retained across a fact page boundary")
                .unwrap();
            assert_eq!(
                owner.read(id, 0, 100).unwrap(),
                b"retained across a fact page boundary"
            );
        }
        assert_eq!(remote.observe().unwrap().0, 263);
        let metrics = remote.server.take_write_metrics().unwrap();
        assert!(metrics.live_backing_calls > 0 && metrics.live_backing_calls < 130);
        assert!(metrics.live_backing_wait_ns > 0);
        assert!(metrics.live_backing_queue_ns > 0);
        assert!(metrics.host_dispatch_ns > 0);
        assert_eq!(metrics.client_frame_bytes, metrics.host_frame_bytes);
        remote.server.control("pause").unwrap();
        let (first, _) = workspace.lock().unwrap().commit().unwrap();
        install_checkpoint(&workspace).unwrap();
        remote.server.control("resume").unwrap();
        assert_eq!(owner.lookup(directory_id, b"file").unwrap().node, file);
        assert_eq!(owner.read(file, 0, 20).unwrap(), b"first");
        assert_eq!(remote.observe().unwrap().0, 0);
        owner.link(file, directory_id, b"alias").unwrap();
        owner
            .rename(directory_id, b"alias", crate::ROOT, b"moved", false)
            .unwrap();
        assert_eq!(owner.lookup(crate::ROOT, b"moved").unwrap().node, file);
        owner.unlink(crate::ROOT, b"moved", false).unwrap();
        let page = runtime
            .block_on(owner.directory_page_async(directory_id, 0))
            .unwrap();
        assert_eq!(page.len(), 128);
        let cookie = page.last().unwrap().0;
        let removed = page
            .iter()
            .find(|(_, _, name)| name.starts_with(b"sibling-"))
            .unwrap()
            .2
            .clone();
        owner.unlink(directory_id, &removed, false).unwrap();
        owner
            .create_file(directory_id, b"new-child", 0o600)
            .unwrap();
        let rest = runtime
            .block_on(owner.directory_page_async(directory_id, cookie))
            .unwrap();
        assert!(rest.iter().any(|(_, _, name)| name == b"new-child"));
        assert!(rest
            .iter()
            .all(|(_, _, name)| !page.iter().any(|(_, _, seen)| seen == name)));
        let empty = owner
            .mkdir(crate::ROOT, b"open-directory", 0o700)
            .unwrap()
            .node;
        owner.pin_directory(empty).unwrap();
        owner.unlink(crate::ROOT, b"open-directory", true).unwrap();
        assert_eq!(
            owner.attr(empty).unwrap().kind,
            layerfs_fuse::Kind::Directory
        );
        owner.unpin_directory(empty).unwrap();
        assert!(owner.attr(empty).is_err());
        owner.write(file, 0, b"later").unwrap();
        remote.server.control("pause").unwrap();
        let (second, _) = workspace.lock().unwrap().commit().unwrap();
        install_checkpoint(&workspace).unwrap();
        remote.server.control("resume").unwrap();
        assert_ne!(first, second);
        assert_eq!(owner.read(file, 0, 20).unwrap(), b"later");
        owner
            .write(file, 0, b"retained after host failure")
            .unwrap();
        remote.backing.lock().unwrap().append_reservation = None;
        assert!(owner.fsync(None).is_err());
        assert_eq!(
            owner.read(file, 0, 100).unwrap(),
            b"retained after host failure"
        );
        assert!(owner.write(file, 0, b"must not replay").is_err());
        owner.unpin(file, true).unwrap();
        let server = remote.server.clone();
        let ending = std::thread::spawn(move || server.control("shutdown"));
        control.wait_for_shutdown().unwrap();
        control.finish_shutdown(true).unwrap();
        ending.join().unwrap().unwrap();
        drop(owner);
        drop(workspace);
        drop(remote);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn append_consumes_only_its_exact_reservation() {
        let directory = std::env::temp_dir().join(format!(
            "layerfs-live-backing-{}",
            crate::WorkspaceId::new()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let store = LayerStackStore::create(directory.join("store.sqlite")).unwrap();
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
        let workspace = crate::Workspace::open(store, branch, directory.join("spool")).unwrap();
        let mut owner = BackingOwner::new(
            WorkspaceSnapshot {
                store: workspace.store.clone(),
                workspace_id: workspace.workspace_id,
                branch_id: workspace.branch_id,
                expected_head: workspace.expected_head,
                expected_base: workspace.expected_base,
                root: workspace.base_root,
                reader: workspace.reader.clone(),
            },
            workspace.live.nodes[&crate::ROOT].clone(),
            workspace.spool.clone(),
            workspace.live.policy,
        );
        let mut reserve = vec![wire::RESERVE];
        wire::u64_out(&mut reserve, 3);
        let response = owner.request(&reserve).unwrap();
        let mut input = Input(&response);
        let id = input.u64().unwrap();
        let offset = input.u64().unwrap();
        assert!(
            owner.request(&reserve).is_err(),
            "second reservation aliases an outstanding tail"
        );
        let append = |bytes: &[u8]| {
            let mut out = vec![wire::APPEND];
            wire::u64_out(&mut out, id);
            wire::u64_out(&mut out, offset);
            wire::bytes_out(&mut out, bytes).unwrap();
            out
        };
        assert!(owner.request(&append(b"oversized")).is_err());
        owner.request(&append(b"abc")).unwrap();
        assert!(
            owner.request(&append(b"abc")).is_err(),
            "completed append replayed"
        );
        let mut read = vec![wire::READ_BACKING];
        wire::u64_out(&mut read, id);
        wire::u64_out(&mut read, offset);
        read.extend_from_slice(&3u32.to_be_bytes());
        assert_eq!(owner.request(&read).unwrap(), b"abc");
        let response = owner.request(&reserve).unwrap();
        let mut input = Input(&response);
        let next_id = input.u64().unwrap();
        let next_offset = input.u64().unwrap();
        let mut part = vec![wire::APPEND];
        wire::u64_out(&mut part, next_id);
        wire::u64_out(&mut part, next_offset);
        wire::bytes_out(&mut part, b"x").unwrap();
        owner.request(&part).unwrap();
        assert!(owner.request(&part).is_err(), "partial prefix replayed");
        let mut cancel = vec![wire::CANCEL_RESERVATION];
        wire::u64_out(&mut cancel, next_id);
        wire::u64_out(&mut cancel, next_offset + 1);
        owner.request(&cancel).unwrap();
        assert!(owner.request(&reserve).is_ok());
        assert_eq!(owner.request(&read).unwrap(), b"abc");
        drop(owner);
        drop(workspace);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[derive(Clone)]
pub(crate) struct RemoteWorkspace {
    pub(crate) backing: std::sync::Arc<std::sync::Mutex<BackingOwner>>,
    pub(crate) server: std::sync::Arc<layerfs_fuse::live_transport::BackingServer>,
}

impl RemoteWorkspace {
    pub(crate) fn start(workspace: &crate::Workspace) -> Result<Self> {
        use std::sync::{Arc, Mutex};
        let backing = Arc::new(Mutex::new(BackingOwner::new(
            WorkspaceSnapshot {
                store: workspace.store.clone(),
                workspace_id: workspace.workspace_id,
                branch_id: workspace.branch_id,
                expected_head: workspace.expected_head,
                expected_base: workspace.expected_base,
                root: workspace.base_root,
                reader: workspace.reader.clone(),
            },
            workspace.live.nodes[&crate::ROOT].clone(),
            workspace.spool.clone(),
            workspace.live.policy,
        )));
        let handler = backing.clone();
        let server = layerfs_fuse::live_transport::BackingServer::start(move |bytes| {
            handler
                .lock()
                .map_err(|_| layerfs_fuse::PortError::Io)?
                .request(bytes)
                .map_err(crate::projection::storage_port_error)
        })?;
        Ok(Self {
            backing,
            server: Arc::new(server),
        })
    }

    pub(crate) fn observe(&self) -> crate::WorkspaceResult<(u64, u64, u64)> {
        let response = self
            .server
            .request(&[wire::OBSERVE])
            .map_err(|_| crate::WorkspaceError::InvalidExecution)?;
        let mut input = Input(&response);
        let result = (input.u64()?, input.u64()?, input.u64()?);
        input.done()?;
        Ok(result)
    }
}

pub(crate) fn install_checkpoint(
    workspace: &std::sync::Mutex<crate::Workspace>,
) -> crate::WorkspaceResult<()> {
    use crate::WorkspaceError;
    use layerfs_layerstack_store::CommitOutcome;
    let pending = {
        let mut workspace = workspace
            .lock()
            .map_err(|_| WorkspaceError::WorkspaceBusy)?;
        let Some(remote) = workspace.remote.clone() else {
            return Ok(());
        };
        let Some((outcome, base, _)) = workspace.pending_publication else {
            return Ok(());
        };
        (remote, outcome, base, workspace.pending_checkpoint.take())
    };
    let (remote, outcome, base, checkpoint) = pending;
    let root = match outcome {
        CommitOutcome::Committed { root_id, .. } | CommitOutcome::UpToDate { root_id } => root_id,
    };
    let installed = (|| -> crate::WorkspaceResult<()> {
        if let Some(checkpoint) = &checkpoint {
            let mut begin = vec![wire::INSTALL_BEGIN];
            begin.extend_from_slice(root.as_bytes());
            wire::u64_out(&mut begin, checkpoint.generation);
            remote
                .server
                .request(&begin)
                .map_err(|_| WorkspaceError::InvalidExecution)?;
            let mut count = 0;
            let mut page = vec![wire::INSTALL_NODE];
            let mut page_count = 0;
            checkpoint.visit(|id, inode, content, attr| {
                let mut node = remote
                    .backing
                    .lock()
                    .map_err(|_| StoreError::Integrity("live backing lock"))?
                    .facts
                    .get(&id)
                    .ok_or(StoreError::Integrity("checkpoint fact"))?
                    .clone();
                node.canonical = Some(inode);
                if node.attr(id) != attr {
                    return Err(StoreError::Integrity("checkpoint fact attr"));
                }
                let mut record = content.as_bytes().to_vec();
                // No mutable ranges are needed to validate/install a canonical checkpoint.
                match &mut node.data {
                    layerfs_workspace_core::Data::File(data) => {
                        *data = layerfs_workspace_core::FileData::Base {
                            root: FileStateRoot(content),
                            len: attr.size,
                        }
                    }
                    layerfs_workspace_core::Data::Directory(directory) => {
                        directory.base = Some(DirectoryStateRoot(content));
                        directory.changes.clear();
                    }
                    _ => {}
                }
                wire::bytes_out(&mut record, &wire::node_out(id, &node)?)?;
                if page_count != 0
                    && (page_count == wire::FACT_PAGE_NODES
                        || page.len() + record.len() > wire::FACT_PAGE_BYTES)
                {
                    remote
                        .server
                        .request(&page)
                        .map_err(|_| StoreError::Integrity("remote checkpoint page"))?;
                    page.truncate(1);
                    page_count = 0;
                }
                if record.len() + 1 > wire::MAX_FRAME {
                    return Err(StoreError::InvalidInput("checkpoint page"));
                }
                page.extend(record);
                page_count += 1;
                count += 1;
                Ok(())
            })?;
            if page_count != 0 {
                remote
                    .server
                    .request(&page)
                    .map_err(|_| WorkspaceError::InvalidExecution)?;
            }
            let mut end = vec![wire::INSTALL_END];
            end.extend_from_slice(root.as_bytes());
            wire::u64_out(&mut end, count);
            remote
                .server
                .request(&end)
                .map_err(|_| WorkspaceError::InvalidExecution)?;
        }
        let mut workspace = workspace
            .lock()
            .map_err(|_| WorkspaceError::WorkspaceBusy)?;
        let reader = workspace
            .store
            .snapshot_reader(root)
            .with_read_metrics_from(&workspace.reader);
        let namespace = layerfs_content::filesystem::namespace(&CoreReader(&reader), root)
            .map_err(StoreError::from)?;
        workspace.reader = reader.clone();
        workspace.base_root = root;
        workspace.base_inodes = InodeTableRoot(namespace.inode_table_root);
        workspace.expected_base = base;
        if let CommitOutcome::Committed { commit_id, .. } = outcome {
            workspace.expected_head = Some(commit_id);
        }
        let mut backing = remote
            .backing
            .lock()
            .map_err(|_| WorkspaceError::WorkspaceBusy)?;
        backing.snapshot.reader = reader;
        backing.snapshot.root = root;
        backing.facts.clear();
        backing.fact_reservations.clear();
        backing.dirty.clear();
        backing.generation = 0;
        backing.spool.retire();
        workspace.pending_publication = None;
        Ok(())
    })();
    if installed.is_err() {
        workspace
            .lock()
            .map_err(|_| WorkspaceError::WorkspaceBusy)?
            .pending_checkpoint = checkpoint;
    }
    installed
}

pub(crate) fn generation(worker: &crate::worker::WorkspaceWorker) -> crate::WorkspaceResult<u64> {
    let (remote, generation) = {
        let workspace = worker
            .workspace
            .lock()
            .map_err(|_| crate::WorkspaceError::WorkspaceBusy)?;
        (workspace.remote.clone(), workspace.live.mutation_generation)
    };
    match remote {
        Some(remote) => remote.observe().map(|values| values.0),
        None => Ok(generation),
    }
}

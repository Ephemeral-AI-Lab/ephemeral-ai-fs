//! Execution-side live state. Physical storage and canonical construction stay on the host.
use crate::live_runtime::{LiveRuntime, OperationGate, Scheduler};
use crate::live_transport::BackingConnection;
use crate::live_wire::{self as wire, Input};
use crate::{Attr, FilesystemPort, Kind, NodeId, PortError, PortResult, ROOT};
use layerfs_workspace_core::backing::{BackingId, BackingRef};
use layerfs_workspace_core::file_edit::{Piece, SpoolSlice};
use layerfs_workspace_core::namespace::{AcquiredInode, NameLookup, ResolvedName};
use layerfs_workspace_core::{Data, LiveWorkspace, ReadPlan, ReadSource, ResourcePolicy};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LiveOwner(Arc<Owner>);

struct Owner {
    scheduler: Scheduler,
    state: Mutex<LiveWorkspace>,
    backing: BackingConnection,
    // Only physical append allocation is ordered across files. No state lock
    // is retained while a physical reservation or append waits.
    append: tokio::sync::Mutex<()>,
    ordering: Mutex<HashMap<NodeId, Arc<tokio::sync::Mutex<()>>>>,
    ranges: Mutex<HashMap<BackingId, BackingRef>>,
    gate: OperationGate,
    writes: crate::write_metrics::AtomicFuseWriteMetrics,
    reads: crate::write_metrics::AtomicFuseReadMetrics,
    #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
    notifier: std::sync::OnceLock<fuser::Notifier>,
    cut: Mutex<Option<crate::live_runtime::OperationCut>>,
    install: Mutex<
        Option<(
            layerfs_content::ObjectId,
            u64,
            std::collections::BTreeMap<
                NodeId,
                (
                    layerfs_content::tree::inode::InodeId,
                    layerfs_content::ObjectId,
                    Attr,
                ),
            >,
        )>,
    >,
}

fn core(error: layerfs_workspace_core::Error) -> PortError {
    use layerfs_workspace_core::Error;
    match error {
        Error::NotFound(_) => PortError::NotFound,
        Error::InvalidInput("name exists") => PortError::Exists,
        Error::InvalidInput("directory not empty") => PortError::NotEmpty,
        Error::InvalidInput("workspace spool limit" | "workspace live allocation") => {
            PortError::NoSpace
        }
        Error::InvalidInput(_) | Error::Core(_) => PortError::Invalid,
        Error::Integrity(_) => PortError::Io,
    }
}
fn io(_: std::io::Error) -> PortError {
    PortError::Io
}

impl LiveOwner {
    pub async fn connect(
        endpoint: String,
        capability: [u8; 32],
        scheduler: Scheduler,
    ) -> PortResult<Self> {
        let backing = BackingConnection::connect(endpoint, capability, &scheduler)
            .await
            .map_err(io)?;
        let seed = backing.call(&[wire::SEED]).await?;
        let mut input = Input(&seed);
        let root = input.object().map_err(io)?;
        let policy = ResourcePolicy {
            max_spool_bytes: input.u64().map_err(io)?,
            max_final_delta_memory_bytes: input.u64().map_err(io)?,
        };
        let (id, node) = wire::node_in(input.0, |_, _, _| Err(wire::invalid())).map_err(io)?;
        if id != ROOT {
            return Err(PortError::Io);
        }
        Ok(Self(Arc::new(Owner {
            scheduler,
            writes: Default::default(),
            reads: Default::default(),
            #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
            notifier: Default::default(),
            state: Mutex::new(LiveWorkspace::new(node, policy, root)),
            backing,
            append: Default::default(),
            ordering: Default::default(),
            ranges: Default::default(),
            gate: Default::default(),
            cut: Default::default(),
            install: Default::default(),
        })))
    }

    fn state(&self) -> PortResult<std::sync::MutexGuard<'_, LiveWorkspace>> {
        self.0.state.lock().map_err(|_| PortError::Io)
    }

    async fn ordered(&self, node: NodeId) -> PortResult<tokio::sync::OwnedMutexGuard<()>> {
        let order = {
            let mut ordering = self.0.ordering.lock().map_err(|_| PortError::Io)?;
            ordering.entry(node).or_default().clone()
        };
        Ok(order.lock_owned().await)
    }

    async fn name(&self, parent: NodeId, name: &[u8]) -> PortResult<ResolvedName> {
        let (lookup, root) = {
            let state = self.state()?;
            (
                state.prepare_name(parent, name).map_err(core)?,
                state.base_root,
            )
        };
        match lookup {
            NameLookup::Ready(name) => Ok(name),
            NameLookup::Acquire(input) => {
                let mut request = vec![wire::LOOKUP];
                request.extend_from_slice(root.as_bytes());
                request.extend_from_slice(input.directory.0.as_bytes());
                wire::bytes_out(&mut request, input.name.as_bytes()).map_err(io)?;
                let response = self.0.backing.call(&request).await?;
                let mut received = Input(&response);
                let acquired = match received.byte().map_err(io)? {
                    0 => {
                        received.done().map_err(io)?;
                        None
                    }
                    1 => {
                        let (_, node) = wire::node_in(received.0, |_, _, _| Err(wire::invalid()))
                            .map_err(io)?;
                        Some(AcquiredInode {
                            inode: node.canonical.ok_or(PortError::Io)?,
                            mode: node.mode,
                            links: node.links,
                            mtime_seconds: node.mtime_seconds,
                            mtime_nanoseconds: node.mtime_nanoseconds,
                            data: node.data,
                        })
                    }
                    _ => return Err(PortError::Io),
                };
                self.state()?.complete_name(input, acquired).map_err(core)
            }
        }
    }

    pub async fn write_owned(&self, node: NodeId, offset: u64, bytes: &[u8]) -> PortResult<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if bytes.len() > 1024 * 1024 {
            return Err(PortError::Invalid);
        }
        let _order = self.ordered(node).await?;
        let _append = self.0.append.lock().await;
        let mut reserve = vec![wire::RESERVE];
        wire::u64_out(&mut reserve, bytes.len() as u64);
        let response = self.0.backing.call(&reserve).await?;
        let mut input = Input(&response);
        let id = BackingId(input.u64().map_err(io)?);
        let start = input.u64().map_err(io)?;
        let capacity = input.u64().map_err(io)?;
        input.done().map_err(io)?;
        if start
            .checked_add(bytes.len() as u64)
            .is_none_or(|end| end > capacity)
        {
            return Err(PortError::Io);
        }
        let reference = self
            .0
            .ranges
            .lock()
            .map_err(|_| PortError::Io)?
            .entry(id)
            .or_insert_with(|| BackingRef::new(id, capacity))
            .clone();
        let prepared = self
            .state()?
            .prepare_write(
                node,
                offset,
                bytes.len(),
                Some(SpoolSlice {
                    segment: reference,
                    offset: start,
                    len: bytes.len() as u64,
                }),
            )
            .map_err(core);
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                let mut release = vec![wire::CANCEL_RESERVATION];
                wire::u64_out(&mut release, id.0);
                wire::u64_out(&mut release, start);
                self.0.backing.call(&release).await?;
                return Err(error);
            }
        };
        let mut append = vec![wire::APPEND];
        wire::u64_out(&mut append, id.0);
        wire::u64_out(&mut append, start);
        wire::bytes_out(&mut append, bytes).map_err(io)?;
        self.0.backing.call(&append).await?;
        self.state()?.apply_edit(prepared).map_err(core)
    }

    pub async fn read_owned(&self, node: NodeId, offset: u64, size: usize) -> PortResult<Vec<u8>> {
        if size > 1024 * 1024 {
            return Err(PortError::Invalid);
        }
        let plan = {
            let state = self.state()?;
            let Data::File(file) = &state.nodes.get(&node).ok_or(PortError::NotFound)?.data else {
                return Err(PortError::Invalid);
            };
            ReadPlan::for_file(file, offset, size).map_err(core)?
        };
        let pieces = match plan.source {
            ReadSource::Base(root, start, end) => vec![Piece::Base {
                root,
                offset: start,
                len: end - start,
            }],
            ReadSource::Edited(pieces) => pieces,
        };
        let mut out = Vec::with_capacity(plan.requested as usize);
        for piece in pieces {
            let mut request;
            let length;
            match piece {
                Piece::Zero { len } => {
                    out.resize(out.len() + len as usize, 0);
                    continue;
                }
                Piece::Inline { bytes, offset, len } => {
                    out.extend_from_slice(&bytes[offset as usize..(offset + len) as usize]);
                    continue;
                }
                Piece::Base { root, offset, len } => {
                    request = vec![wire::READ_BASE];
                    request.extend_from_slice(root.0.as_bytes());
                    wire::u64_out(&mut request, offset);
                    length = len;
                }
                Piece::Spool {
                    ref segment,
                    offset,
                    len,
                } => {
                    request = vec![wire::READ_BACKING];
                    wire::u64_out(&mut request, segment.id().0);
                    wire::u64_out(&mut request, offset);
                    length = len;
                }
            }
            if length == 0 {
                continue;
            }
            request.extend_from_slice(&(length as u32).to_be_bytes());
            let bytes = self.0.backing.call(&request).await?;
            if bytes.len() as u64 != length {
                return Err(PortError::Io);
            }
            out.extend_from_slice(&bytes);
        }
        Ok(out)
    }

    fn run<T>(&self, future: impl std::future::Future<Output = PortResult<T>>) -> PortResult<T> {
        LiveRuntime::shared().map_err(io)?.block_on(future)
    }
}

impl FilesystemPort for LiveOwner {
    fn note_kernel_operation(&self, operation: crate::KernelOperation) {
        self.0.reads.note_kernel_operation(operation);
    }
    fn note_fuse_max_write(&self, bytes: u32) {
        self.0.writes.note_max_write(bytes as u64);
    }
    fn note_fuse_read_config(&self, bytes: u32, flags: u64) {
        self.0.reads.note_config(bytes as u64, flags);
    }
    fn note_readdir_page(&self, offset: u64, entries: u64) {
        self.0.reads.note_readdir_page(offset, entries);
    }

    fn admit_callback(
        &self,
        _operation: crate::KernelOperation,
        bytes: usize,
        writeback: bool,
    ) -> PortResult<crate::CallbackGuard> {
        match _operation {
            crate::KernelOperation::Write => self.0.writes.note_kernel_write(bytes as u64),
            crate::KernelOperation::Read => self.0.reads.note_kernel_read(bytes as u64),
            _ => {}
        }
        let admission = self
            .0
            .scheduler
            .admit_from_receiver(
                bytes
                    .checked_mul(3)
                    .and_then(|n| n.checked_add(65536))
                    .ok_or(PortError::NoSpace)?,
            )
            .map_err(io)?;
        let gate = if matches!(
            _operation,
            crate::KernelOperation::Release | crate::KernelOperation::Releasedir
        ) {
            None
        } else {
            Some(
                self.0
                    .scheduler
                    .handle
                    .block_on(self.0.gate.enter(writeback)),
            )
        };
        Ok(crate::CallbackGuard {
            _gate: gate,
            _admission: Some(admission),
        })
    }
    #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
    fn submit_write(
        &self,
        node: NodeId,
        offset: u64,
        bytes: &[u8],
        _: bool,
        reply: crate::WriteReply,
    ) {
        let owner = self.clone();
        let bytes = bytes.to_vec();
        self.0.scheduler.handle.spawn(async move {
            reply.complete(owner.write_owned(node, offset, &bytes).await);
        });
    }
    #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
    fn submit_read(&self, node: NodeId, offset: u64, size: usize, reply: crate::ReadReply) {
        let owner = self.clone();
        self.0.scheduler.handle.spawn(async move {
            reply.complete(owner.read_owned(node, offset, size).await);
        });
    }
    fn lookup(&self, parent: NodeId, name: &[u8]) -> PortResult<Attr> {
        self.run(async {
            let _order = self.ordered(parent).await?;
            let name = self.name(parent, name).await?;
            self.state()?
                .attr(name.existing().ok_or(PortError::NotFound)?)
                .map_err(core)
        })
    }
    fn attr(&self, node: NodeId) -> PortResult<Attr> {
        self.state()?.attr(node).map_err(core)
    }
    fn readlink(&self, node: NodeId) -> PortResult<Vec<u8>> {
        match &self
            .state()?
            .nodes
            .get(&node)
            .ok_or(PortError::NotFound)?
            .data
        {
            Data::Symlink(target) => Ok(target.clone()),
            _ => Err(PortError::Invalid),
        }
    }
    fn create_file(&self, parent: NodeId, name: &[u8], mode: u32) -> PortResult<Attr> {
        self.run(async {
            let _order = self.ordered(parent).await?;
            let name = self.name(parent, name).await?;
            self.state()?.create_file(name, mode, None).map_err(core)
        })
    }
    fn mkdir(&self, parent: NodeId, name: &[u8], mode: u32) -> PortResult<Attr> {
        self.run(async {
            let _order = self.ordered(parent).await?;
            let name = self.name(parent, name).await?;
            self.state()?.mkdir(name, mode, None).map_err(core)
        })
    }
    fn symlink(&self, parent: NodeId, name: &[u8], target: Vec<u8>) -> PortResult<Attr> {
        self.run(async {
            let _order = self.ordered(parent).await?;
            let name = self.name(parent, name).await?;
            self.state()?.symlink(name, target).map_err(core)
        })
    }
    fn read(&self, node: NodeId, offset: u64, size: usize) -> PortResult<Vec<u8>> {
        self.run(self.read_owned(node, offset, size))
    }
    fn write(&self, node: NodeId, offset: u64, bytes: &[u8]) -> PortResult<usize> {
        self.run(self.write_owned(node, offset, bytes))
    }
    fn pin(&self, node: NodeId, truncate: bool, _: bool) -> PortResult<()> {
        self.state()?.check_pin(node).map_err(core)?;
        if truncate {
            self.truncate(node, 0)?;
        }
        self.state()?.pin(node).map_err(core)
    }
    fn unpin(&self, node: NodeId, _: bool) -> PortResult<()> {
        self.state()?.unpin(node).map(drop).map_err(core)
    }
    fn truncate(&self, node: NodeId, size: u64) -> PortResult<()> {
        self.run(async {
            let _order = self.ordered(node).await?;
            let prepared = self.state()?.prepare_truncate(node, size).map_err(core)?;
            if let Some(prepared) = prepared {
                let mut check = vec![wire::CHECK];
                for range in prepared.backing_ranges() {
                    wire::u64_out(&mut check, range.segment.id().0);
                }
                self.0.backing.call(&check).await?;
                self.state()?.apply_edit(prepared).map_err(core)?;
            }
            Ok(())
        })
    }
    fn chmod(&self, node: NodeId, mode: u32) -> PortResult<()> {
        self.run(async {
            let _order = self.ordered(node).await?;
            self.state()?.chmod(node, mode).map_err(core)
        })
    }
    fn set_mtime(&self, node: NodeId, seconds: i64, nanos: u32) -> PortResult<()> {
        self.run(async {
            let _order = self.ordered(node).await?;
            self.state()?.set_mtime(node, seconds, nanos).map_err(core)
        })
    }
    fn fsync(&self, _: Option<NodeId>) -> PortResult<()> {
        self.run(async {
            let _append = self.0.append.lock().await;
            let mut check = vec![wire::CHECK];
            for id in self.0.ranges.lock().map_err(|_| PortError::Io)?.keys() {
                wire::u64_out(&mut check, id.0);
            }
            self.0.backing.call(&check).await.map(drop)
        })
    }
    fn readdir(&self, _: NodeId) -> PortResult<Vec<(NodeId, Kind, Vec<u8>)>> {
        Err(PortError::Invalid)
    }
    fn link(&self, _: NodeId, _: NodeId, _: &[u8]) -> PortResult<Attr> {
        Err(PortError::Invalid)
    }
    fn unlink(&self, _: NodeId, _: &[u8], _: bool) -> PortResult<()> {
        Err(PortError::Invalid)
    }
    fn rename(&self, _: NodeId, _: &[u8], _: NodeId, _: &[u8], _: bool) -> PortResult<()> {
        Err(PortError::Invalid)
    }
}

impl LiveOwner {
    #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
    pub fn set_notifier(&self, notifier: fuser::Notifier) -> std::io::Result<()> {
        self.0.notifier.set(notifier).map_err(|_| wire::invalid())
    }

    fn invalidate(&self, node: NodeId) -> PortResult<()> {
        #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
        {
            self.0
                .notifier
                .get()
                .ok_or(PortError::Io)?
                .inval_inode(fuser::INodeNo(node.0), 0, 0)
                .map_err(io)
        }
        #[cfg(not(all(target_os = "linux", any(feature = "host", feature = "proxy"))))]
        {
            let _ = node;
            Err(PortError::Invalid)
        }
    }

    pub async fn freeze(&self) -> PortResult<()> {
        if self.0.cut.lock().map_err(|_| PortError::Io)?.is_some() {
            return Ok(());
        }
        let cut = self.0.gate.cache_flush().await.finish().await;
        *self.0.cut.lock().map_err(|_| PortError::Io)? = Some(cut);
        let (generation, ids) = {
            let state = self.state()?;
            let mut ids = state.dirty.clone();
            for id in &state.dirty {
                if let Some(layerfs_workspace_core::Node {
                    data: Data::Directory(directory),
                    ..
                }) = state.nodes.get(id)
                {
                    ids.extend(directory.changes.values().flatten());
                }
            }
            (state.mutation_generation, ids)
        };
        let mut begin = vec![wire::FACTS_BEGIN];
        wire::u64_out(&mut begin, generation);
        self.0.backing.call(&begin).await?;
        for id in &ids {
            let frame = {
                let state = self.state()?;
                let node = state.nodes.get(id).ok_or(PortError::Io)?;
                let mut frame = vec![wire::FACTS_NODE, u8::from(state.dirty.contains(id))];
                wire::bytes_out(&mut frame, &wire::node_out(*id, node).map_err(io)?).map_err(io)?;
                frame
            };
            self.0.backing.call(&frame).await?;
        }
        let mut end = vec![wire::FACTS_END];
        wire::u64_out(&mut end, generation);
        wire::u64_out(&mut end, ids.len() as u64);
        self.0.backing.call(&end).await.map(drop)
    }

    async fn control_request(&self, bytes: &[u8]) -> PortResult<Vec<u8>> {
        let mut input = Input(bytes);
        let mut out = Vec::new();
        let opcode = input.byte().map_err(io)?;
        match opcode {
            wire::WRITE_METRICS => {
                input.done().map_err(io)?;
                self.0.writes.take().write_to(&mut out).map_err(io)?;
            }
            wire::READ_METRICS => {
                input.done().map_err(io)?;
                self.0.reads.take().write_to(&mut out).map_err(io)?;
            }
            wire::INVALIDATE => {
                let node = NodeId(input.u64().map_err(io)?);
                input.done().map_err(io)?;
                self.invalidate(node)?;
            }
            wire::FREEZE => {
                input.done().map_err(io)?;
                self.freeze().await?;
            }
            wire::RESUME => {
                input.done().map_err(io)?;
                self.0.cut.lock().map_err(|_| PortError::Io)?.take();
            }
            wire::OBSERVE => {
                input.done().map_err(io)?;
                let state = self.state()?;
                wire::u64_out(&mut out, state.mutation_generation);
                wire::u64_out(&mut out, state.dirty.len() as u64);
                wire::u64_out(&mut out, state.spool_bytes);
            }
            wire::INSTALL_BEGIN => {
                let root = input.object().map_err(io)?;
                let generation = input.u64().map_err(io)?;
                input.done().map_err(io)?;
                if self.0.cut.lock().map_err(|_| PortError::Io)?.is_none()
                    || self.state()?.mutation_generation != generation
                {
                    return Err(PortError::Invalid);
                }
                let mut install = self.0.install.lock().map_err(|_| PortError::Io)?;
                if install.is_some() {
                    return Err(PortError::Busy);
                }
                *install = Some((root, generation, Default::default()));
            }
            wire::INSTALL_NODE => {
                let content = input.object().map_err(io)?;
                let (id, node) =
                    wire::node_in(input.0, |_, _, _| Err(wire::invalid())).map_err(io)?;
                let inode = node.canonical.ok_or(PortError::Invalid)?;
                self.state()?
                    .validate_checkpoint_record(id, inode, node.attr(id))
                    .map_err(core)?;
                let mut install = self.0.install.lock().map_err(|_| PortError::Io)?;
                let (_, _, records) = install.as_mut().ok_or(PortError::Invalid)?;
                if records.len() >= 16384 {
                    return Err(PortError::NoSpace);
                }
                if records.contains_key(&id) {
                    return Err(PortError::Invalid);
                }
                records.insert(id, (inode, content, node.attr(id)));
            }
            wire::INSTALL_END => {
                let root = input.object().map_err(io)?;
                let count = input.u64().map_err(io)?;
                input.done().map_err(io)?;
                let mut install = self.0.install.lock().map_err(|_| PortError::Io)?;
                let (expected, generation, records) = install.as_ref().ok_or(PortError::Invalid)?;
                if *expected != root || records.len() as u64 != count {
                    return Err(PortError::Invalid);
                }
                let mut state = self.state()?;
                if state.mutation_generation != *generation {
                    return Err(PortError::Invalid);
                }
                for (&id, &(inode, _, attr)) in records {
                    state
                        .validate_checkpoint_record(id, inode, attr)
                        .map_err(core)?;
                }
                for (&id, &(inode, content, attr)) in records {
                    state
                        .install_checkpoint_record(id, inode, content, attr)
                        .map_err(core)?;
                }
                state.finish_checkpoint(root).map_err(core)?;
                *install = None;
            }
            _ => return Err(PortError::Invalid),
        }
        if opcode == wire::INSTALL_END {
            self.retire_ranges().await?;
        }
        Ok(out)
    }

    async fn retire_ranges(&self) -> PortResult<()> {
        let ids: Vec<_> = self
            .0
            .ranges
            .lock()
            .map_err(|_| PortError::Io)?
            .iter()
            .filter_map(|(id, reference)| reference.is_unique().then_some(*id))
            .collect();
        for ids in ids.chunks(128) {
            let mut release = vec![wire::RELEASE];
            for id in ids {
                wire::u64_out(&mut release, id.0);
            }
            self.0.backing.call(&release).await?;
            let mut ranges = self.0.ranges.lock().map_err(|_| PortError::Io)?;
            for id in ids {
                ranges.remove(id);
            }
        }
        Ok(())
    }

    pub fn serve_control(
        &self,
        endpoint: String,
        capability: [u8; 32],
    ) -> std::io::Result<LiveControl> {
        use std::net::ToSocketAddrs;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let address = endpoint
            .to_socket_addrs()?
            .next()
            .ok_or_else(wire::invalid)?;
        let (shutdown_send, shutdown) = std::sync::mpsc::sync_channel(1);
        let (finished, mut finish) = tokio::sync::oneshot::channel();
        let owner = self.clone();
        let thread = self.0.scheduler.handle.spawn(async move {
            let mut stream = tokio::net::TcpStream::connect(address).await?;
            stream.set_nodelay(true)?;
            stream.write_all(&capability).await?;
            stream.write_u8(b'c').await?;
            if stream.read_u8().await? != 1 {
                return Err(wire::invalid());
            }
            loop {
                let len = stream.read_u32().await? as usize;
                if len == 0 || len > wire::MAX_FRAME {
                    return Err(wire::invalid());
                }
                let _admitted = owner.0.scheduler.admit(len + wire::MAX_FRAME).await?;
                let mut bytes = vec![0; len];
                stream.read_exact(&mut bytes).await?;
                let shutdown_requested = bytes == [wire::SHUTDOWN];
                let result = if shutdown_requested {
                    shutdown_send.send(()).map_err(|_| wire::invalid())?;
                    if (&mut finish).await.map_err(|_| wire::invalid())? {
                        Ok(Vec::new())
                    } else {
                        Err(PortError::Io)
                    }
                } else {
                    owner.control_request(&bytes).await
                };
                match result {
                    Ok(bytes) => {
                        stream.write_u32((bytes.len() + 1) as u32).await?;
                        stream.write_u8(0).await?;
                        stream.write_all(&bytes).await?;
                    }
                    Err(error) => {
                        stream.write_u32(2).await?;
                        stream.write_u8(1).await?;
                        stream.write_u8(crate::protocol::error_code(error)).await?;
                    }
                }
                if shutdown_requested {
                    return Ok(());
                }
            }
        });
        Ok(LiveControl {
            shutdown,
            finished: Some(finished),
            thread,
        })
    }
}

pub struct LiveControl {
    shutdown: std::sync::mpsc::Receiver<()>,
    finished: Option<tokio::sync::oneshot::Sender<bool>>,
    thread: tokio::task::JoinHandle<std::io::Result<()>>,
}
impl LiveControl {
    pub fn wait_for_shutdown(&self) -> std::io::Result<()> {
        self.shutdown.recv().map_err(|_| wire::invalid())
    }
    pub fn finish_shutdown(mut self, success: bool) -> std::io::Result<()> {
        self.finished
            .take()
            .ok_or_else(wire::invalid)?
            .send(success)
            .map_err(|_| wire::invalid())?;
        LiveRuntime::shared()?
            .block_on(self.thread)
            .map_err(|_| wire::invalid())?
    }
}

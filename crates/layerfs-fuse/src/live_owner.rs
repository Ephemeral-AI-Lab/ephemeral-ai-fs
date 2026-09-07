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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct LiveOwner(Arc<Owner>);

pub struct LocalFacts {
    pub root: layerfs_content::ObjectId,
    pub generation: u64,
    pub nodes: HashMap<NodeId, layerfs_workspace_core::Node>,
    pub dirty: std::collections::BTreeSet<NodeId>,
    pub charge: crate::live_runtime::LiveReservation,
}

struct Owner {
    scheduler: Scheduler,
    read_scope: u64,
    state: Mutex<LiveWorkspace>,
    head: Mutex<Option<[u8; 33]>>,
    backing: BackingConnection,
    local_facts: Option<Arc<dyn Fn(LocalFacts) -> PortResult<()> + Send + Sync>>,
    // Only physical append allocation is ordered across files. No state lock
    // is retained while a physical reservation or append waits.
    append: tokio::sync::Mutex<Option<AppendWindow>>,
    failed: AtomicBool,
    closing: AtomicBool,
    cached: Mutex<std::collections::BTreeSet<NodeId>>,
    facts_sync: tokio::sync::Mutex<Option<(layerfs_content::ObjectId, u64)>>,
    ordering: Mutex<HashMap<NodeId, Arc<tokio::sync::Mutex<()>>>>,
    namespace: tokio::sync::Mutex<()>,
    directories: Mutex<HashMap<NodeId, DirectoryCookies>>,
    ranges: Mutex<HashMap<BackingId, BackingRef>>,
    edit: Mutex<Option<PendingSplices>>,
    gate: OperationGate,
    writes: crate::write_metrics::AtomicFuseWriteMetrics,
    reads: crate::write_metrics::AtomicFuseReadMetrics,
    #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
    notifier: std::sync::OnceLock<fuser::Notifier>,
    #[cfg(target_os = "linux")]
    kernel_root: Mutex<Option<Arc<std::fs::File>>>,
    cut: Mutex<Option<crate::live_runtime::OperationCut>>,
    install: Mutex<Option<PendingCheckpoint>>,
}

impl Drop for Owner {
    fn drop(&mut self) {
        self.scheduler
            .immutable_reads()
            .remove_scope(self.read_scope);
    }
}

type PendingCheckpoint = (
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
);

struct PendingSplices {
    node: NodeId,
    count: usize,
    received: usize,
    prepared: Option<layerfs_workspace_core::file_edit::PreparedFileEdit>,
    cache_ranges: Vec<std::ops::Range<u64>>,
    cut: crate::live_runtime::OperationCut,
    _charge: crate::live_runtime::LiveReservation,
}

struct DirectoryCookies {
    generation: (layerfs_content::ObjectId, u64),
    next: u64,
    names: std::collections::BTreeMap<Vec<u8>, (u64, NodeId)>,
    ordered: std::collections::BTreeMap<u64, (NodeId, Vec<u8>)>,
}

struct AppendWindow {
    backing: BackingRef,
    start: u64,
    reserved: u64,
    filled: u64,
}
struct BufferedFrame {
    bytes: Vec<u8>,
    start: u64,
    _charge: crate::live_runtime::LiveReservation,
}
enum PendingBytes {
    Filling(BufferedFrame),
    Sending(Arc<BufferedFrame>),
    Backed,
}
struct PendingBacking(Mutex<PendingBytes>);

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
        Self::from_backing(backing, scheduler, None).await
    }

    pub async fn local(
        handler: Arc<crate::live_transport::BackingHandler>,
        facts: Arc<dyn Fn(LocalFacts) -> PortResult<()> + Send + Sync>,
        scheduler: Scheduler,
    ) -> PortResult<Self> {
        Self::from_backing(
            BackingConnection::local(handler, scheduler.clone()),
            scheduler,
            Some(facts),
        )
        .await
    }

    async fn from_backing(
        backing: BackingConnection,
        scheduler: Scheduler,
        local_facts: Option<Arc<dyn Fn(LocalFacts) -> PortResult<()> + Send + Sync>>,
    ) -> PortResult<Self> {
        let seed = backing.call(&[wire::SEED]).await?;
        let mut input = Input(&seed);
        let root = input.object().map_err(io)?;
        let head = input.head().map_err(io)?;
        let policy = ResourcePolicy {
            max_spool_bytes: input.u64().map_err(io)?,
            max_final_delta_memory_bytes: input.u64().map_err(io)?,
        };
        let (id, node) = wire::node_in(input.0, |_, _, _| Err(wire::invalid())).map_err(io)?;
        if id != ROOT {
            return Err(PortError::Io);
        }
        Ok(Self(Arc::new(Owner {
            read_scope: scheduler.immutable_reads().new_scope(),
            scheduler,
            writes: Default::default(),
            reads: Default::default(),
            #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
            notifier: Default::default(),
            #[cfg(target_os = "linux")]
            kernel_root: Default::default(),
            state: Mutex::new(LiveWorkspace::new(node, policy, root)),
            head: Mutex::new(head),
            backing,
            local_facts,
            append: Default::default(),
            failed: AtomicBool::new(false),
            closing: AtomicBool::new(false),
            cached: Default::default(),
            facts_sync: Default::default(),
            ordering: Default::default(),
            namespace: Default::default(),
            directories: Default::default(),
            ranges: Default::default(),
            edit: Default::default(),
            gate: Default::default(),
            cut: Default::default(),
            install: Default::default(),
        })))
    }

    fn state(&self) -> PortResult<std::sync::MutexGuard<'_, LiveWorkspace>> {
        self.0.state.lock().map_err(|_| PortError::Io)
    }

    async fn ordered(&self, node: NodeId) -> PortResult<tokio::sync::OwnedMutexGuard<()>> {
        self.state()?.attr(node).map_err(core)?;
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
                let cache = self.0.scheduler.immutable_reads();
                if let Some(encoded) = cache.get_name(
                    self.0.read_scope,
                    root,
                    input.directory.0,
                    input.name.as_bytes(),
                ) {
                    let acquired = Self::immutable_node(&encoded)?;
                    return self
                        .state()?
                        .complete_name(input, Some(acquired))
                        .map_err(core);
                }
                if let Some(page) = cache.get_page(self.0.read_scope, root, input.directory.0, &[])
                {
                    let mut page = Input(&page);
                    let more = page.byte().map_err(io)? != 0;
                    let count = page.u32().map_err(io)?;
                    let mut last = Vec::new();
                    let mut found = None;
                    for _ in 0..count {
                        let name = page.bytes().map_err(io)?;
                        let encoded = page.bytes().map_err(io)?;
                        page.bytes().map_err(io)?; // cached pages contain no payloads
                        if name == input.name.as_bytes() {
                            found = Some(Self::immutable_node(encoded)?);
                        }
                        last = name.to_vec();
                    }
                    page.done().map_err(io)?;
                    if found.is_some() || !more || input.name.as_bytes() <= last.as_slice() {
                        return self.state()?.complete_name(input, found).map_err(core);
                    }
                }
                let directory = input.directory;
                let mut request = vec![wire::LOOKUP];
                request.extend_from_slice(root.as_bytes());
                request.extend_from_slice(directory.0.as_bytes());
                wire::bytes_out(&mut request, input.name.as_bytes()).map_err(io)?;
                let response = self.0.backing.call(&request).await?;
                let mut received = Input(&response);
                let mut prefetched = Vec::new();
                let mut complete_entries = std::collections::BTreeMap::new();
                let acquired = match received.byte().map_err(io)? {
                    0 => None,
                    1 => {
                        let (encoded, acquired) =
                            self.immutable_reply(&mut received, &mut prefetched)?;
                        complete_entries.insert(input.name.as_bytes().to_vec(), encoded);
                        Some(acquired)
                    }
                    _ => return Err(PortError::Io),
                };
                let count = received.u32().map_err(io)?;
                if count > 127 {
                    return Err(PortError::Io);
                }
                let mut previous = Vec::new();
                let mut siblings = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    let name = received.bytes().map_err(io)?.to_vec();
                    layerfs_content::CanonicalName::from_bytes(&name).map_err(|_| PortError::Io)?;
                    if name <= previous || name == input.name.as_bytes() {
                        return Err(PortError::Io);
                    }
                    previous = name.clone();
                    let (encoded, _) = self.immutable_reply(&mut received, &mut prefetched)?;
                    complete_entries.insert(name.clone(), encoded.clone());
                    siblings.push((name, encoded));
                }
                let complete = match received.byte().map_err(io)? {
                    0 => false,
                    1 => true,
                    _ => return Err(PortError::Io),
                };
                received.done().map_err(io)?;
                // complete_name revalidates root and parent revision before changing live state.
                let resolved = self.state()?.complete_name(input, acquired).map_err(core)?;
                self.cache_prefetched(prefetched);
                if complete {
                    let mut page = vec![0];
                    page.extend_from_slice(&(complete_entries.len() as u32).to_be_bytes());
                    for (name, encoded) in complete_entries {
                        wire::bytes_out(&mut page, &name).map_err(io)?;
                        wire::bytes_out(&mut page, &encoded).map_err(io)?;
                        wire::bytes_out(&mut page, &[]).map_err(io)?;
                    }
                    cache.insert_page(self.0.read_scope, root, directory.0, &[], page);
                }
                for (name, encoded) in siblings {
                    cache.insert_name(self.0.read_scope, root, directory.0, &name, encoded);
                }
                Ok(resolved)
            }
        }
    }

    fn immutable_node(encoded: &[u8]) -> PortResult<AcquiredInode> {
        let (_, node) = wire::node_in(encoded, |_, _, _| Err(wire::invalid())).map_err(io)?;
        if !node.paths.is_empty() || node.pins != 0 || node.revision != 0 {
            return Err(PortError::Io);
        }
        match &node.data {
            Data::File(layerfs_workspace_core::FileData::Edited { .. }) => {
                return Err(PortError::Io)
            }
            Data::Directory(directory)
                if directory.base.is_none() || !directory.changes.is_empty() =>
            {
                return Err(PortError::Io)
            }
            _ => {}
        }
        Ok(AcquiredInode {
            inode: node.canonical.ok_or(PortError::Io)?,
            mode: node.mode,
            links: node.links,
            mtime_seconds: node.mtime_seconds,
            mtime_nanoseconds: node.mtime_nanoseconds,
            data: node.data,
        })
    }

    fn immutable_reply(
        &self,
        input: &mut Input<'_>,
        prefetched: &mut Vec<(layerfs_content::ObjectId, Vec<u8>)>,
    ) -> PortResult<(Vec<u8>, AcquiredInode)> {
        let encoded = input.bytes().map_err(io)?.to_vec();
        let acquired = Self::immutable_node(&encoded)?;
        let bytes = input.bytes().map_err(io)?;
        if !bytes.is_empty() {
            let Data::File(layerfs_workspace_core::FileData::Base { root, len }) = &acquired.data
            else {
                return Err(PortError::Io);
            };
            if bytes.len() > wire::IMMUTABLE_PREFETCH_FILE_BYTES || bytes.len() as u64 != *len {
                return Err(PortError::Io);
            }
            prefetched.push((root.0, bytes.to_vec()));
        }
        Ok((encoded, acquired))
    }

    fn cache_prefetched(&self, prefetched: Vec<(layerfs_content::ObjectId, Vec<u8>)>) {
        for (root, bytes) in prefetched {
            self.0.reads.note_read_ahead_miss(0, bytes.len() as u64, 0);
            self.0
                .scheduler
                .immutable_reads()
                .insert(self.0.read_scope, root, 0, bytes);
        }
    }

    pub async fn write_owned(&self, node: NodeId, offset: u64, bytes: &[u8]) -> PortResult<usize> {
        if self.0.failed.load(Ordering::Acquire) {
            return Err(PortError::Io);
        }
        if bytes.is_empty() {
            return Ok(0);
        }
        if bytes.len() > 1024 * 1024 {
            return Err(PortError::Invalid);
        }
        let _order = self.ordered(node).await?;
        let mut window = self.0.append.lock().await;
        if window
            .as_ref()
            .is_some_and(|window| window.reserved - window.filled < bytes.len() as u64)
        {
            self.flush_append(&mut window).await?;
        }
        if window.is_none() {
            let mut wanted = {
                let state = self.state()?;
                (1024 * 1024)
                    .min(
                        state
                            .policy
                            .max_spool_bytes
                            .saturating_sub(state.spool_bytes),
                    )
                    .max(bytes.len() as u64)
            };
            // Own pending capacity before requesting physical space or copying payload.
            let charge = self
                .0
                .scheduler
                .reserve_transfer(wanted as usize + 21)
                .map_err(|_| PortError::NoSpace)?;
            let mut reserve = vec![wire::RESERVE];
            wire::u64_out(&mut reserve, wanted);
            let response = match self.0.backing.call(&reserve).await {
                Err(PortError::NoSpace) if wanted > bytes.len() as u64 => {
                    wanted = bytes.len() as u64;
                    reserve.truncate(1);
                    wire::u64_out(&mut reserve, wanted);
                    self.0.backing.call(&reserve).await?
                }
                result => result?,
            };
            let mut input = Input(&response);
            let id = BackingId(input.u64().map_err(io)?);
            let start = input.u64().map_err(io)?;
            let capacity = input.u64().map_err(io)?;
            input.done().map_err(io)?;
            if start.checked_add(wanted).is_none_or(|end| end > capacity) {
                return Err(PortError::Io);
            }
            let backing = self
                .0
                .ranges
                .lock()
                .map_err(|_| PortError::Io)?
                .entry(id)
                .or_insert_with(|| {
                    BackingRef::new(id, PendingBacking(Mutex::new(PendingBytes::Backed)))
                })
                .clone();
            let mut frame = Vec::with_capacity(wanted as usize + 21);
            frame.push(wire::APPEND);
            wire::u64_out(&mut frame, id.0);
            wire::u64_out(&mut frame, start);
            frame.extend_from_slice(&0u32.to_be_bytes());
            *backing
                .resource::<PendingBacking>()
                .ok_or(PortError::Io)?
                .0
                .lock()
                .map_err(|_| PortError::Io)? = PendingBytes::Filling(BufferedFrame {
                bytes: frame,
                start,
                _charge: charge,
            });
            *window = Some(AppendWindow {
                backing,
                start,
                reserved: wanted,
                filled: 0,
            });
        }
        let window = window.as_mut().ok_or(PortError::Io)?;
        let preparing = Instant::now();
        let prepared = self
            .state()?
            .prepare_write(
                node,
                offset,
                bytes.len(),
                Some(SpoolSlice {
                    segment: window.backing.clone(),
                    offset: window.start + window.filled,
                    len: bytes.len() as u64,
                }),
            )
            .map_err(core);
        self.0
            .writes
            .live_edit_ns
            .fetch_add(ns(preparing), Ordering::Relaxed);
        let prepared = prepared?;
        let encoding = Instant::now();
        {
            let mut pending = window
                .backing
                .resource::<PendingBacking>()
                .ok_or(PortError::Io)?
                .0
                .lock()
                .map_err(|_| PortError::Io)?;
            let PendingBytes::Filling(frame) = &mut *pending else {
                return Err(PortError::Io);
            };
            frame.bytes.extend_from_slice(bytes);
        }
        window.filled += bytes.len() as u64;
        self.0
            .writes
            .note_client_frame(0, bytes.len() as u64, ns(encoding), 0);
        let applying = Instant::now();
        let result = self.state()?.apply_edit(prepared).map_err(core);
        self.0
            .writes
            .live_edit_ns
            .fetch_add(ns(applying), Ordering::Relaxed);
        result
    }

    async fn flush_append(&self, window: &mut Option<AppendWindow>) -> PortResult<()> {
        if self.0.failed.load(Ordering::Acquire) {
            return Err(PortError::Io);
        }
        let Some(window) = window.take() else {
            return Ok(());
        };
        let resource = window
            .backing
            .resource::<PendingBacking>()
            .ok_or(PortError::Io)?;
        let frame = {
            let mut pending = resource.0.lock().map_err(|_| PortError::Io)?;
            let PendingBytes::Filling(mut frame) =
                std::mem::replace(&mut *pending, PendingBytes::Backed)
            else {
                return Err(PortError::Io);
            };
            frame.bytes[17..21].copy_from_slice(&(window.filled as u32).to_be_bytes());
            let frame = Arc::new(frame);
            *pending = PendingBytes::Sending(frame.clone());
            frame
        };
        let result = async {
            if window.filled != 0 {
                self.0.backing.call(&frame.bytes).await?;
            }
            if window.filled < window.reserved {
                let mut cancel = vec![wire::CANCEL_RESERVATION];
                wire::u64_out(&mut cancel, window.backing.id().0);
                wire::u64_out(&mut cancel, window.start + window.filled);
                self.0.backing.call(&cancel).await?;
            }
            Ok(())
        }
        .await;
        if result.is_ok() {
            *resource.0.lock().map_err(|_| PortError::Io)? = PendingBytes::Backed;
        } else {
            // Preserve acknowledged local bytes and their charge after ambiguous
            // backing failure. Never replay the append or discard its prefix.
            self.0.failed.store(true, Ordering::Release);
        }
        result
    }

    pub async fn read_owned(&self, node: NodeId, offset: u64, size: usize) -> PortResult<Vec<u8>> {
        if size > 1024 * 1024 {
            return Err(PortError::Invalid);
        }
        let file = {
            let state = self.state()?;
            let Data::File(file) = &state.nodes.get(&node).ok_or(PortError::NotFound)?.data else {
                return Err(PortError::Invalid);
            };
            file.clone()
        };
        self.read_file(&file, offset, size).await
    }

    async fn read_file(
        &self,
        file: &layerfs_workspace_core::FileData,
        offset: u64,
        size: usize,
    ) -> PortResult<Vec<u8>> {
        let plan = ReadPlan::for_file(file, offset, size).map_err(core)?;
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
                    if let Some(bytes) = self.0.scheduler.immutable_reads().get(
                        self.0.read_scope,
                        root.0,
                        offset,
                        len as usize,
                    ) {
                        self.0.reads.note_read_ahead_hit(bytes.len() as u64);
                        out.extend(bytes);
                        continue;
                    }
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
                    let local = {
                        let pending = segment
                            .resource::<PendingBacking>()
                            .ok_or(PortError::Io)?
                            .0
                            .lock()
                            .map_err(|_| PortError::Io)?;
                        let frame = match &*pending {
                            PendingBytes::Filling(frame) => Some(frame),
                            PendingBytes::Sending(frame) => Some(frame.as_ref()),
                            PendingBytes::Backed => None,
                        };
                        frame
                            .filter(|frame| offset + len > frame.start)
                            .map(|frame| {
                                let prefix = frame.start.saturating_sub(offset).min(len);
                                let start = 21 + (offset + prefix - frame.start) as usize;
                                let end = start + (len - prefix) as usize;
                                frame
                                    .bytes
                                    .get(start..end)
                                    .map(|bytes| (prefix, bytes.to_vec()))
                                    .ok_or(PortError::Io)
                            })
                            .transpose()?
                    };
                    if let Some((prefix, local)) = local {
                        if prefix != 0 {
                            let mut request = vec![wire::READ_BACKING];
                            wire::u64_out(&mut request, segment.id().0);
                            wire::u64_out(&mut request, offset);
                            request.extend_from_slice(&(prefix as u32).to_be_bytes());
                            let prefix_bytes = self.0.backing.call(&request).await?;
                            if prefix_bytes.len() as u64 != prefix {
                                return Err(PortError::Io);
                            }
                            out.extend(prefix_bytes);
                        }
                        out.extend(local);
                        continue;
                    }
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
            if let Piece::Base { root, offset, .. } = piece {
                self.0
                    .reads
                    .note_read_ahead_miss(length, bytes.len() as u64, length);
                self.0.scheduler.immutable_reads().insert(
                    self.0.read_scope,
                    root.0,
                    offset,
                    bytes.clone(),
                );
            }
            out.extend_from_slice(&bytes);
        }
        Ok(out)
    }

    async fn entries(
        &self,
        node: NodeId,
    ) -> PortResult<std::collections::BTreeMap<Vec<u8>, NodeId>> {
        let (root, base, revision, changes) = {
            let state = self.state()?;
            let directory = state.directory(node).map_err(core)?;
            (
                state.base_root,
                directory.base,
                state.nodes[&node].revision,
                directory.changes.clone(),
            )
        };
        let mut entries = std::collections::BTreeMap::new();
        if let Some(base) = base {
            let mut after = Vec::new();
            loop {
                let cursor = after.clone();
                let cache = self.0.scheduler.immutable_reads();
                let response =
                    if let Some(page) = cache.get_page(self.0.read_scope, root, base.0, &cursor) {
                        page
                    } else {
                        let mut request = vec![wire::DIRECTORY_PAGE];
                        request.extend_from_slice(root.as_bytes());
                        request.extend_from_slice(base.0.as_bytes());
                        wire::bytes_out(&mut request, &cursor).map_err(io)?;
                        self.0.backing.call(&request).await?
                    };
                let mut input = Input(&response);
                let more = match input.byte().map_err(io)? {
                    0 => false,
                    1 => true,
                    _ => return Err(PortError::Io),
                };
                let count = input.u32().map_err(io)? as usize;
                if count > 128 || (more && count == 0) {
                    return Err(PortError::Io);
                }
                // Cache immutable metadata/completeness only; payloads have their
                // own entries in the same budget, avoiding duplicate content charge.
                let mut encoded_page = vec![u8::from(more)];
                encoded_page.extend_from_slice(&(count as u32).to_be_bytes());
                let mut prefetched = Vec::new();
                let mut page = Vec::with_capacity(count);
                for _ in 0..count {
                    let name = input.bytes().map_err(io)?.to_vec();
                    if name <= after {
                        return Err(PortError::Io);
                    }
                    after = name.clone();
                    let (encoded, acquired) = self.immutable_reply(&mut input, &mut prefetched)?;
                    wire::bytes_out(&mut encoded_page, &name).map_err(io)?;
                    wire::bytes_out(&mut encoded_page, &encoded).map_err(io)?;
                    wire::bytes_out(&mut encoded_page, &[]).map_err(io)?;
                    page.push((name, acquired));
                }
                input.done().map_err(io)?;
                let mut state = self.state()?;
                if state.base_root != root
                    || state.nodes.get(&node).map(|node| node.revision) != Some(revision)
                {
                    return Err(PortError::Io);
                }
                for (name, acquired) in page {
                    if changes.contains_key(&name) {
                        continue;
                    }
                    let path = state.child_path(node, &name).map_err(core)?;
                    let child = state.install_immutable_node(acquired, path).map_err(core)?;
                    state.remember_directory_parent(child, node).map_err(core)?;
                    state
                        .remember_name(node, &name, Some(child))
                        .map_err(core)?;
                    entries.insert(name, child);
                }
                drop(state);
                self.cache_prefetched(prefetched);
                cache.insert_page(self.0.read_scope, root, base.0, &cursor, encoded_page);
                if entries.len() > 16384 {
                    return Err(PortError::NoSpace);
                }
                if !more {
                    break;
                }
            }
        }
        for (name, child) in changes {
            if let Some(child) = child {
                entries.insert(name, child);
            }
        }
        if entries.len() > 16384 {
            return Err(PortError::NoSpace);
        }
        Ok(entries)
    }

    async fn empty_directory(&self, node: NodeId) -> PortResult<bool> {
        {
            let state = self.state()?;
            let directory = state.directory(node).map_err(core)?;
            if directory.changes.values().any(Option::is_some) {
                return Ok(false);
            }
            if directory.base.is_none() {
                return Ok(true);
            }
        }
        Ok(self.entries(node).await?.is_empty())
    }

    fn run<T>(&self, future: impl std::future::Future<Output = PortResult<T>>) -> PortResult<T> {
        LiveRuntime::shared().map_err(io)?.block_on(future)
    }
}

impl FilesystemPort for LiveOwner {
    fn note_cached_open(&self, node: NodeId) {
        if let Ok(mut cached) = self.0.cached.lock() {
            cached.insert(node);
        }
    }

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
        if matches!(
            _operation,
            crate::KernelOperation::Release | crate::KernelOperation::Releasedir
        ) {
            return Ok(crate::CallbackGuard::default());
        }
        if self.0.closing.load(Ordering::Acquire) {
            return Err(PortError::Io);
        }
        match _operation {
            crate::KernelOperation::Write => self.0.writes.note_kernel_write(bytes as u64),
            crate::KernelOperation::Read => self.0.reads.note_kernel_read(bytes as u64),
            _ => {}
        }
        let control = writeback
            || matches!(
                _operation,
                crate::KernelOperation::Read
                    | crate::KernelOperation::Fsync
                    | crate::KernelOperation::Fsyncdir
                    | crate::KernelOperation::Flush
            );
        let admission = self
            .0
            .scheduler
            .try_admit_from_receiver(
                bytes
                    .checked_mul(3)
                    .and_then(|n| n.checked_add(65536))
                    .ok_or(PortError::NoSpace)?,
                control,
            )
            .map_err(|_| PortError::NoSpace)?;
        Ok(crate::CallbackGuard {
            _gate: None,
            _admission: Some(admission),
        })
    }
    fn callback_gate(
        &self,
        operation: crate::KernelOperation,
        writeback: bool,
    ) -> crate::PortFuture<'_, Option<tokio::sync::OwnedRwLockReadGuard<()>>> {
        Box::pin(async move {
            if matches!(
                operation,
                crate::KernelOperation::Release | crate::KernelOperation::Releasedir
            ) {
                return Ok(None);
            }
            if self.0.failed.load(Ordering::Acquire)
                && !matches!(
                    operation,
                    crate::KernelOperation::Read
                        | crate::KernelOperation::Getattr
                        | crate::KernelOperation::Lookup
                        | crate::KernelOperation::Readlink
                        | crate::KernelOperation::Readdir
                        | crate::KernelOperation::Readdirplus
                        | crate::KernelOperation::Access
                        | crate::KernelOperation::Statfs
                )
            {
                return Err(PortError::Io);
            }
            // A page-fault READ can own a folio required by invalidation.
            // Drain it with laundering callbacks, not behind the ordinary cut.
            let writeback = writeback
                || matches!(
                    operation,
                    crate::KernelOperation::Read
                        | crate::KernelOperation::Fsync
                        | crate::KernelOperation::Fsyncdir
                        | crate::KernelOperation::Flush
                );
            let gate = self.0.gate.enter(writeback).await;
            if self.0.closing.load(Ordering::Acquire) {
                return Err(PortError::Io);
            }
            Ok(Some(gate))
        })
    }

    #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
    fn submit_write(
        &self,
        node: NodeId,
        offset: u64,
        bytes: &[u8],
        writeback: bool,
        mut reply: crate::WriteReply,
    ) {
        let owner = self.clone();
        // Cached write-through callbacks can hold a folio needed by invalidation.
        let kernel_cache = writeback
            || self
                .0
                .cached
                .lock()
                .is_ok_and(|cached| cached.contains(&node));
        let bytes = bytes.to_vec();
        self.0.writes.note_client_copy(bytes.len() as u64);
        let queued = Instant::now();
        self.0.scheduler.submit(async move {
            owner
                .0
                .writes
                .live_write_dispatch_ns
                .fetch_add(ns(queued), Ordering::Relaxed);
            reply._guard._gate = match owner
                .callback_gate(crate::KernelOperation::Write, kernel_cache)
                .await
            {
                Ok(gate) => gate,
                Err(error) => {
                    reply.complete(Err(error));
                    return;
                }
            };
            let result = owner.write_owned(node, offset, &bytes).await;
            if writeback && result.is_err() {
                owner.0.failed.store(true, Ordering::Release);
            }
            reply.complete(result);
        });
    }
    #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
    fn submit_read(&self, node: NodeId, offset: u64, size: usize, mut reply: crate::ReadReply) {
        let owner = self.clone();
        self.0.scheduler.submit(async move {
            reply._guard._gate = match owner
                .callback_gate(crate::KernelOperation::Read, false)
                .await
            {
                Ok(gate) => gate,
                Err(error) => {
                    reply.complete(Err(error));
                    return;
                }
            };
            reply.complete(owner.read_owned(node, offset, size).await);
        });
    }
    fn lookup(&self, parent: NodeId, name: &[u8]) -> PortResult<Attr> {
        self.run(self.lookup_async(parent, name))
    }
    fn lookup_async<'a>(&'a self, parent: NodeId, name: &'a [u8]) -> crate::PortFuture<'a, Attr> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
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
        self.run(self.create_file_async(parent, name, mode))
    }
    fn create_file_async<'a>(
        &'a self,
        parent: NodeId,
        name: &'a [u8],
        mode: u32,
    ) -> crate::PortFuture<'a, Attr> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
            let name = self.name(parent, name).await?;
            self.state()?.create_file(name, mode, None).map_err(core)
        })
    }
    fn mkdir(&self, parent: NodeId, name: &[u8], mode: u32) -> PortResult<Attr> {
        self.run(self.mkdir_async(parent, name, mode))
    }
    fn mkdir_async<'a>(
        &'a self,
        parent: NodeId,
        name: &'a [u8],
        mode: u32,
    ) -> crate::PortFuture<'a, Attr> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
            let name = self.name(parent, name).await?;
            self.state()?.mkdir(name, mode, None).map_err(core)
        })
    }
    fn symlink(&self, parent: NodeId, name: &[u8], target: Vec<u8>) -> PortResult<Attr> {
        self.run(self.symlink_async(parent, name, target))
    }
    fn symlink_async<'a>(
        &'a self,
        parent: NodeId,
        name: &'a [u8],
        target: Vec<u8>,
    ) -> crate::PortFuture<'a, Attr> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
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
    fn pin(&self, node: NodeId, truncate: bool, writable: bool) -> PortResult<()> {
        self.run(self.pin_async(node, truncate, writable))
    }
    fn pin_async<'a>(&'a self, node: NodeId, truncate: bool, _: bool) -> crate::PortFuture<'a, ()> {
        Box::pin(async move {
            self.state()?.pin(node).map_err(core)?;
            if truncate {
                if let Err(error) = self.truncate_async(node, 0).await {
                    self.state()?.unpin(node).map_err(core)?;
                    return Err(error);
                }
            }
            Ok(())
        })
    }
    fn create_file_open(&self, parent: NodeId, name: &[u8], mode: u32) -> PortResult<Attr> {
        self.run(self.create_file_open_async(parent, name, mode))
    }
    fn create_file_open_async<'a>(
        &'a self,
        parent: NodeId,
        name: &'a [u8],
        mode: u32,
    ) -> crate::PortFuture<'a, Attr> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
            let name = self.name(parent, name).await?;
            let mut state = self.state()?;
            let attr = state.create_file(name, mode, None).map_err(core)?;
            state.pin(attr.node).map_err(core)?;
            Ok(attr)
        })
    }
    fn pin_directory(&self, node: NodeId) -> PortResult<()> {
        self.state()?.pin_directory(node).map_err(core)
    }
    fn unpin_directory(&self, node: NodeId) -> PortResult<()> {
        self.unpin(node, false)
    }
    fn unpin(&self, node: NodeId, _: bool) -> PortResult<()> {
        let reclaimed = {
            let mut state = self.state()?;
            state.unpin(node).map_err(core)?;
            !state.nodes.contains_key(&node)
        };
        if reclaimed {
            self.0
                .ordering
                .lock()
                .map_err(|_| PortError::Io)?
                .remove(&node);
            self.0
                .directories
                .lock()
                .map_err(|_| PortError::Io)?
                .remove(&node);
            self.0
                .cached
                .lock()
                .map_err(|_| PortError::Io)?
                .remove(&node);
        }
        Ok(())
    }
    fn truncate(&self, node: NodeId, size: u64) -> PortResult<()> {
        self.run(self.truncate_async(node, size))
    }
    fn truncate_async<'a>(&'a self, node: NodeId, size: u64) -> crate::PortFuture<'a, ()> {
        Box::pin(async move {
            let _order = self.ordered(node).await?;
            let prepared = self.state()?.prepare_truncate(node, size).map_err(core)?;
            if let Some(prepared) = prepared {
                let mut check = vec![wire::CHECK, 0];
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
        self.run(self.chmod_async(node, mode))
    }
    fn chmod_async<'a>(&'a self, node: NodeId, mode: u32) -> crate::PortFuture<'a, ()> {
        Box::pin(async move {
            let directory = self.state()?.attr(node).map_err(core)?.kind == Kind::Directory;
            let _namespace = if directory {
                Some(self.0.namespace.lock().await)
            } else {
                None
            };
            let _order = self.ordered(node).await?;
            self.state()?.chmod(node, mode).map_err(core)
        })
    }
    fn set_mtime(&self, node: NodeId, seconds: i64, nanos: u32) -> PortResult<()> {
        self.run(self.set_mtime_async(node, seconds, nanos))
    }
    fn set_mtime_async<'a>(
        &'a self,
        node: NodeId,
        seconds: i64,
        nanos: u32,
    ) -> crate::PortFuture<'a, ()> {
        Box::pin(async move {
            let directory = self.state()?.attr(node).map_err(core)?.kind == Kind::Directory;
            let _namespace = if directory {
                Some(self.0.namespace.lock().await)
            } else {
                None
            };
            let _order = self.ordered(node).await?;
            self.state()?.set_mtime(node, seconds, nanos).map_err(core)
        })
    }
    fn fsync(&self, node: Option<NodeId>) -> PortResult<()> {
        self.run(self.fsync_async(node))
    }
    fn fsync_async<'a>(&'a self, _node: Option<NodeId>) -> crate::PortFuture<'a, ()> {
        Box::pin(async move {
            let mut window = self.0.append.lock().await;
            self.flush_append(&mut window).await?;
            let mut check = vec![wire::CHECK, 1];
            for id in self.0.ranges.lock().map_err(|_| PortError::Io)?.keys() {
                wire::u64_out(&mut check, id.0);
            }
            self.0.backing.call(&check).await?;
            self.publish_facts().await?;
            self.retire_ranges().await
        })
    }
    fn readdir(&self, node: NodeId) -> PortResult<Vec<(NodeId, Kind, Vec<u8>)>> {
        self.run(async {
            let _namespace = self.0.namespace.lock().await;
            let entries = self.entries(node).await?;
            let state = self.state()?;
            let mut result = vec![
                (node, Kind::Directory, b".".to_vec()),
                (
                    state.parent_of(node).map_err(core)?,
                    Kind::Directory,
                    b"..".to_vec(),
                ),
            ];
            for (name, id) in entries {
                result.push((id, state.attr(id).map_err(core)?.kind, name));
            }
            Ok(result)
        })
    }
    fn link(&self, node: NodeId, parent: NodeId, name: &[u8]) -> PortResult<Attr> {
        self.run(self.link_async(node, parent, name))
    }
    fn link_async<'a>(
        &'a self,
        node: NodeId,
        parent: NodeId,
        name: &'a [u8],
    ) -> crate::PortFuture<'a, Attr> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
            let name = self.name(parent, name).await?;
            self.state()?.link(node, name).map_err(core)
        })
    }
    fn unlink(&self, parent: NodeId, name: &[u8], directory: bool) -> PortResult<()> {
        self.run(self.unlink_async(parent, name, directory))
    }
    fn unlink_async<'a>(
        &'a self,
        parent: NodeId,
        name: &'a [u8],
        directory: bool,
    ) -> crate::PortFuture<'a, ()> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
            let name = self.name(parent, name).await?;
            let node = name.existing().ok_or(PortError::NotFound)?;
            let empty = if directory {
                self.empty_directory(node).await?
            } else {
                true
            };
            self.state()?.unlink(name, directory, empty).map_err(core)?;
            if !self.state()?.nodes.contains_key(&node) {
                self.0
                    .ordering
                    .lock()
                    .map_err(|_| PortError::Io)?
                    .remove(&node);
                self.0
                    .directories
                    .lock()
                    .map_err(|_| PortError::Io)?
                    .remove(&node);
            }
            Ok(())
        })
    }
    fn rename(
        &self,
        parent: NodeId,
        name: &[u8],
        target_parent: NodeId,
        target: &[u8],
        no_replace: bool,
    ) -> PortResult<()> {
        self.run(self.rename_async(parent, name, target_parent, target, no_replace))
    }
    fn rename_async<'a>(
        &'a self,
        parent: NodeId,
        name: &'a [u8],
        target_parent: NodeId,
        target: &'a [u8],
        no_replace: bool,
    ) -> crate::PortFuture<'a, ()> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
            let source = self.name(parent, name).await?;
            let target = self.name(target_parent, target).await?;
            let empty = if let Some(node) = target.existing() {
                if self.state()?.attr(node).map_err(core)?.kind == Kind::Directory {
                    self.empty_directory(node).await?
                } else {
                    true
                }
            } else {
                true
            };
            self.state()?
                .rename(source, target, no_replace, empty)
                .map_err(core)
        })
    }
    fn readdir_page_async<'a>(
        &'a self,
        node: NodeId,
        after: usize,
    ) -> crate::PortFuture<'a, Vec<(NodeId, Kind, Vec<u8>)>> {
        Box::pin(async move {
            self.directory_page_async(node, after as u64)
                .await
                .map(|entries| {
                    entries
                        .into_iter()
                        .map(|(_, attr, name)| (attr.node, attr.kind, name))
                        .collect()
                })
        })
    }
    fn readdirplus_page_async<'a>(
        &'a self,
        node: NodeId,
        after: usize,
    ) -> crate::PortFuture<'a, Vec<(Attr, Vec<u8>)>> {
        Box::pin(async move {
            self.directory_page_async(node, after as u64)
                .await
                .map(|entries| {
                    entries
                        .into_iter()
                        .map(|(_, attr, name)| (attr, name))
                        .collect()
                })
        })
    }
    fn directory_page_async<'a>(
        &'a self,
        node: NodeId,
        after: u64,
    ) -> crate::PortFuture<'a, Vec<(u64, Attr, Vec<u8>)>> {
        Box::pin(async move {
            let _namespace = self.0.namespace.lock().await;
            let generation = {
                let state = self.state()?;
                (
                    state.base_root,
                    state.nodes.get(&node).ok_or(PortError::NotFound)?.revision,
                )
            };
            let refresh = self
                .0
                .directories
                .lock()
                .map_err(|_| PortError::Io)?
                .get(&node)
                .is_none_or(|cursor| cursor.generation != generation);
            if refresh {
                let entries = self.entries(node).await?;
                let mut directories = self.0.directories.lock().map_err(|_| PortError::Io)?;
                let cursor = directories.entry(node).or_insert_with(|| DirectoryCookies {
                    generation,
                    next: 3,
                    names: Default::default(),
                    ordered: Default::default(),
                });
                cursor.names.retain(|name, (cookie, id)| {
                    let keep = entries.get(name) == Some(id);
                    if !keep {
                        cursor.ordered.remove(cookie);
                    }
                    keep
                });
                for (name, id) in entries {
                    if !cursor.names.contains_key(&name) {
                        let cookie = cursor.next;
                        cursor.next = cookie.checked_add(1).ok_or(PortError::NoSpace)?;
                        cursor.names.insert(name.clone(), (cookie, id));
                        cursor.ordered.insert(cookie, (id, name));
                    }
                }
                cursor.generation = generation;
            }
            let state = self.state()?;
            let directories = self.0.directories.lock().map_err(|_| PortError::Io)?;
            let cursor = directories.get(&node).ok_or(PortError::Io)?;
            let mut page = Vec::new();
            if after == 0 {
                page.push((1, state.attr(node).map_err(core)?, b".".to_vec()));
            }
            if after < 2 {
                page.push((
                    2,
                    state
                        .attr(state.parent_of(node).map_err(core)?)
                        .map_err(core)?,
                    b"..".to_vec(),
                ));
            }
            for (cookie, (id, name)) in cursor
                .ordered
                .range((std::ops::Bound::Excluded(after), std::ops::Bound::Unbounded))
                .take(128 - page.len())
            {
                page.push((*cookie, state.attr(*id).map_err(core)?, name.clone()));
            }
            Ok(page)
        })
    }
}

impl LiveOwner {
    #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
    pub fn set_notifier(&self, notifier: fuser::Notifier) -> std::io::Result<()> {
        self.0.notifier.set(notifier).map_err(|_| wire::invalid())
    }

    #[cfg(target_os = "linux")]
    pub fn set_kernel_root(&self, root: std::fs::File) -> std::io::Result<()> {
        let mut slot = self.0.kernel_root.lock().map_err(|_| wire::invalid())?;
        if slot.is_some() {
            return Err(wire::invalid());
        }
        *slot = Some(Arc::new(root));
        Ok(())
    }

    pub fn prepare_shutdown(&self) -> std::io::Result<()> {
        self.0.closing.store(true, Ordering::Release);
        self.0.cut.lock().map_err(|_| wire::invalid())?.take();
        self.0.edit.lock().map_err(|_| wire::invalid())?.take();
        #[cfg(target_os = "linux")]
        self.0
            .kernel_root
            .lock()
            .map_err(|_| wire::invalid())?
            .take();
        Ok(())
    }

    async fn flush_kernel_cache(&self) -> PortResult<()> {
        #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
        {
            use std::os::fd::AsRawFd;
            let cached = self.0.cached.lock().map_err(|_| PortError::Io)?.clone();
            let nodes: Vec<_> = {
                let state = self.state()?;
                cached
                    .into_iter()
                    .filter(|id| state.nodes.contains_key(id))
                    .collect()
            };
            if nodes.is_empty() {
                return Ok(());
            }
            let notifier = self.0.notifier.get().ok_or(PortError::Io)?.clone();
            let root = self
                .0
                .kernel_root
                .lock()
                .map_err(|_| PortError::Io)?
                .clone()
                .ok_or(PortError::Io)?;
            self.0
                .scheduler
                .kernel(move || {
                    for node in nodes {
                        notifier.inval_inode(fuser::INodeNo(node.0), 0, 0)?;
                    }
                    // Linux invalidation does not propagate every laundering error.
                    // This descriptor predates writes, so syncfs observes the mount's
                    // writeback error sequence, including kernel-side allocation errors.
                    nix::unistd::syncfs(root.as_raw_fd()).map_err(std::io::Error::from)
                })
                .await
                .map_err(io)?;
        }
        if self.0.failed.load(Ordering::Acquire) {
            return Err(PortError::Io);
        }
        Ok(())
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
        if self.0.failed.load(Ordering::Acquire) {
            return Err(PortError::Io);
        }
        if self.0.cut.lock().map_err(|_| PortError::Io)?.is_none() {
            let flush = self.0.gate.cache_flush().await;
            self.flush_kernel_cache().await?;
            let cut = flush.finish().await;
            *self.0.cut.lock().map_err(|_| PortError::Io)? = Some(cut);
        }
        self.flush_append(&mut *self.0.append.lock().await).await?;
        self.publish_facts().await?;
        self.retire_ranges().await
    }

    async fn publish_facts(&self) -> PortResult<()> {
        let mut acknowledged = self.0.facts_sync.lock().await;
        if let Some(sink) = &self.0.local_facts {
            let facts = {
                let state = self.state()?;
                if *acknowledged == Some((state.base_root, state.mutation_generation)) {
                    return Ok(());
                }
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
                let mut charge = 0usize;
                for id in &ids {
                    let node = state.nodes.get(id).ok_or(PortError::Io)?;
                    let inline = match &node.data {
                        Data::File(layerfs_workspace_core::FileData::Edited { pieces, .. }) => {
                            pieces.inline_len() as usize
                        }
                        _ => 0,
                    };
                    charge = charge
                        .checked_add(
                            wire::node_encoded_bound(node)
                                .map_err(io)?
                                .saturating_sub(inline)
                                .checked_mul(8)
                                .and_then(|n| n.checked_add(1024))
                                .ok_or(PortError::NoSpace)?,
                        )
                        .filter(|n| *n <= wire::MAX_FACT_MEMORY)
                        .ok_or(PortError::NoSpace)?;
                }
                let charge = self
                    .0
                    .scheduler
                    .reserve_live(charge)
                    .map_err(|_| PortError::NoSpace)?;
                LocalFacts {
                    root: state.base_root,
                    generation: state.mutation_generation,
                    nodes: ids
                        .into_iter()
                        .map(|id| (id, state.nodes[&id].clone()))
                        .collect(),
                    dirty: state.dirty.clone(),
                    charge,
                }
            };
            let identity = (facts.root, facts.generation);
            let sink = sink.clone();
            self.0
                .scheduler
                .physical(move || Ok(sink(facts)))
                .await
                .map_err(io)??;
            *acknowledged = Some(identity);
            return Ok(());
        }
        let (root, generation, count, pages, _charge) = {
            let state = self.state()?;
            if *acknowledged == Some((state.base_root, state.mutation_generation)) {
                return Ok(());
            }
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
            let mut charge = 0usize;
            for id in &ids {
                let node = state.nodes.get(id).ok_or(PortError::Io)?;
                charge = charge
                    .checked_add(
                        wire::node_encoded_bound(node)
                            .map_err(io)?
                            .checked_mul(8)
                            .and_then(|n| n.checked_add(1024))
                            .ok_or(PortError::NoSpace)?,
                    )
                    .filter(|n| *n <= wire::MAX_FACT_MEMORY)
                    .ok_or(PortError::NoSpace)?;
            }
            let reservation = self
                .0
                .scheduler
                .reserve_live(charge)
                .map_err(|_| PortError::NoSpace)?;
            // Capture one coherent changed-record generation. Encoding holds the
            // short state lock, but no network/physical wait occurs under it.
            let mut pages = Vec::new();
            let mut page = vec![wire::FACTS_NODE];
            let mut count = 0;
            for id in &ids {
                let node = state.nodes.get(id).ok_or(PortError::Io)?;
                let dirty = u8::from(state.dirty.contains(id));
                let encoded = wire::node_out(*id, node).map_err(io)?;
                let mut record = vec![dirty];
                wire::bytes_out(&mut record, &encoded).map_err(io)?;
                if count != 0
                    && (count == wire::FACT_PAGE_NODES
                        || page.len() + record.len() > wire::FACT_PAGE_BYTES)
                {
                    pages.push(std::mem::replace(&mut page, vec![wire::FACTS_NODE]));
                    count = 0;
                }
                if record.len() + 1 > wire::MAX_FRAME {
                    let mut begin = vec![wire::FACTS_NODE_BEGIN, dirty];
                    wire::u64_out(&mut begin, encoded.len() as u64);
                    pages.push(begin);
                    for chunk in encoded.chunks(wire::MAX_FRAME - 1) {
                        let mut frame = vec![wire::FACTS_NODE_CHUNK];
                        frame.extend_from_slice(chunk);
                        pages.push(frame);
                    }
                    continue;
                }
                page.extend(record);
                count += 1;
            }
            if count != 0 {
                pages.push(page);
            }
            (
                state.base_root,
                state.mutation_generation,
                ids.len(),
                pages,
                reservation,
            )
        };
        let mut begin = vec![wire::FACTS_BEGIN];
        wire::u64_out(&mut begin, generation);
        self.0.backing.call(&begin).await?;
        for page in pages {
            self.0.backing.call(&page).await?;
        }
        let mut end = vec![wire::FACTS_END];
        wire::u64_out(&mut end, generation);
        wire::u64_out(&mut end, count as u64);
        self.0.backing.call(&end).await?;
        *acknowledged = Some((root, generation));
        Ok(())
    }

    pub async fn local_control(&self, bytes: &[u8]) -> PortResult<Vec<u8>> {
        if bytes == [wire::SHUTDOWN] {
            self.prepare_shutdown().map_err(io)?;
            return Ok(Vec::new());
        }
        let result = self.control_request(bytes).await;
        if result.is_err()
            && matches!(
                bytes.first(),
                Some(&wire::EDIT_BEGIN) | Some(&wire::EDIT_PART) | Some(&wire::EDIT_END)
            )
        {
            self.0.edit.lock().map_err(|_| PortError::Io)?.take();
        }
        result
    }

    async fn control_request(&self, bytes: &[u8]) -> PortResult<Vec<u8>> {
        let mut input = Input(bytes);
        let mut out = Vec::new();
        let opcode = input.byte().map_err(io)?;
        match opcode {
            wire::EDIT_BEGIN => {
                let path = std::str::from_utf8(input.bytes().map_err(io)?)
                    .map_err(|_| PortError::Invalid)?
                    .to_owned();
                layerfs_content::CanonicalPath::new(&path).map_err(|_| PortError::Invalid)?;
                let count = input.u64().map_err(io)? as usize;
                input.done().map_err(io)?;
                if count == 0
                    || count > layerfs_workspace_core::file_edit::MAX_EDITS_PER_FILE as usize
                {
                    return Err(PortError::Invalid);
                }
                if self.0.edit.lock().map_err(|_| PortError::Io)?.is_some()
                    || self.0.cut.lock().map_err(|_| PortError::Io)?.is_some()
                {
                    return Err(PortError::Busy);
                }
                let charge = self
                    .0
                    .scheduler
                    .reserve_live(9 * 1024 * 1024)
                    .map_err(|_| PortError::NoSpace)?;
                self.freeze().await?;
                let cut = self
                    .0
                    .cut
                    .lock()
                    .map_err(|_| PortError::Io)?
                    .take()
                    .ok_or(PortError::Io)?;
                let mut node = ROOT;
                for name in path.split('/').filter(|name| !name.is_empty()) {
                    node = self.lookup_async(node, name.as_bytes()).await?.node;
                }
                *self.0.edit.lock().map_err(|_| PortError::Io)? = Some(PendingSplices {
                    node,
                    count,
                    received: 0,
                    prepared: None,
                    cache_ranges: Vec::new(),
                    cut,
                    _charge: charge,
                });
            }
            wire::EDIT_PART => {
                let start = input.u64().map_err(io)?;
                let delete = input.u64().map_err(io)?;
                let mut held = self.0.edit.lock().map_err(|_| PortError::Io)?;
                let pending = held.as_mut().ok_or(PortError::Invalid)?;
                if pending.received == pending.count {
                    return Err(PortError::Invalid);
                }
                let piece = match input.byte().map_err(io)? {
                    0 => {
                        let bytes = input.bytes().map_err(io)?;
                        if bytes.len() > layerfs_workspace_core::file_edit::MAX_INLINE_PER_EDIT {
                            return Err(PortError::Invalid);
                        }
                        (!bytes.is_empty()).then(|| Piece::Inline {
                            bytes: Arc::from(bytes),
                            offset: 0,
                            len: bytes.len() as u64,
                        })
                    }
                    1 => {
                        let len = input.u64().map_err(io)?;
                        (len != 0).then_some(Piece::Zero { len })
                    }
                    _ => return Err(PortError::Invalid),
                };
                input.done().map_err(io)?;
                let replacement_len = piece.as_ref().map_or(0, Piece::len);
                let cache_end = if replacement_len == delete {
                    start
                        .checked_add(replacement_len)
                        .ok_or(PortError::Invalid)?
                } else {
                    // An insertion/deletion changes every following byte's position.
                    u64::MAX
                };
                pending.cache_ranges.push(start..cache_end);
                let state = self.state()?;
                if let Some(prepared) = pending.prepared.as_mut() {
                    state
                        .extend_splices(prepared, start, delete, piece)
                        .map_err(core)?;
                } else {
                    pending.prepared = Some(
                        state
                            .prepare_splices(pending.node, vec![(start, delete, piece)])
                            .map_err(core)?,
                    );
                }
                pending.received += 1;
            }
            wire::EDIT_END => {
                input.done().map_err(io)?;
                let pending = self
                    .0
                    .edit
                    .lock()
                    .map_err(|_| PortError::Io)?
                    .take()
                    .ok_or(PortError::Invalid)?;
                if pending.received != pending.count {
                    return Err(PortError::Invalid);
                }
                let node = pending.node;
                #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
                let cache_data = self
                    .0
                    .cached
                    .lock()
                    .map_err(|_| PortError::Io)?
                    .contains(&node);
                #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
                let _cache_charge = self
                    .0
                    .notifier
                    .get()
                    .filter(|_| cache_data)
                    .map(|_| {
                        self.0
                            .scheduler
                            .reserve_live(2 * 1024 * 1024)
                            .map_err(|_| PortError::NoSpace)
                    })
                    .transpose()?;
                self.state()?
                    .apply_edit(pending.prepared.ok_or(PortError::Invalid)?)
                    .map_err(core)?;
                let file = match &self
                    .state()?
                    .nodes
                    .get(&node)
                    .ok_or(PortError::NotFound)?
                    .data
                {
                    Data::File(file) => file.clone(),
                    _ => return Err(PortError::Invalid),
                };
                // Queued post-flush faults must read the installed view before
                // notification can wait for their locked kernel folios.
                drop(pending.cut);
                #[cfg(all(target_os = "linux", any(feature = "host", feature = "proxy")))]
                if let Some(notifier) = self.0.notifier.get().cloned() {
                    let updated = async {
                        let mut ranges = if cache_data {
                            pending.cache_ranges
                        } else {
                            Vec::new()
                        };
                        ranges.sort_unstable_by_key(|range| range.start);
                        let mut end = 0;
                        for range in ranges {
                            let mut offset = range.start.max(end);
                            while offset < range.end {
                                let size = (range.end - offset).min(1024 * 1024) as usize;
                                let bytes = self.read_file(&file, offset, size).await?;
                                if bytes.is_empty() {
                                    break;
                                }
                                let length = bytes.len() as u64;
                                let notifier = notifier.clone();
                                // Invalidation can launder a refaulted dirty page. Update
                                // the edited bytes first so that writeback preserves them.
                                self.0
                                    .scheduler
                                    .kernel(move || {
                                        notifier.store(fuser::INodeNo(node.0), offset, &bytes)
                                    })
                                    .await
                                    .map_err(io)?;
                                offset += length;
                            }
                            end = end.max(offset);
                        }
                        let owner = self.clone();
                        self.0
                            .scheduler
                            .kernel(move || owner.invalidate(node).map_err(|_| wire::invalid()))
                            .await
                            .map_err(io)
                    }
                    .await;
                    if let Err(error) = updated {
                        self.0.failed.store(true, Ordering::Release);
                        return Err(error);
                    }
                }
                #[cfg(not(all(target_os = "linux", any(feature = "host", feature = "proxy"))))]
                let _ = (node, file);
            }
            wire::WRITE_METRICS => {
                input.done().map_err(io)?;
                let mut metrics = self.0.writes.take();
                metrics.merge(self.0.backing.metrics.take());
                metrics.write_to(&mut out).map_err(io)?;
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
                let head = self.0.head.lock().map_err(|_| PortError::Io)?;
                wire::bytes_out(&mut out, head.as_ref().map_or(&[][..], |bytes| &bytes[..]))
                    .map_err(io)?;
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
                let mut count = 0;
                while !input.0.is_empty() {
                    if count == wire::FACT_PAGE_NODES {
                        return Err(PortError::Invalid);
                    }
                    let content = input.object().map_err(io)?;
                    let (id, node) =
                        wire::node_in(input.bytes().map_err(io)?, |_, _, _| Err(wire::invalid()))
                            .map_err(io)?;
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
                    count += 1;
                }
            }
            wire::INSTALL_END => {
                let root = input.object().map_err(io)?;
                let count = input.u64().map_err(io)?;
                let head = input.head().map_err(io)?;
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
                *self.0.head.lock().map_err(|_| PortError::Io)? = head;
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
        let observer_owner = self.clone();
        let observer: tokio::task::JoinHandle<std::io::Result<()>> =
            self.0.scheduler.handle.spawn(async move {
                let mut stream = tokio::net::TcpStream::connect(address).await?;
                stream.set_nodelay(true)?;
                stream.write_all(&capability).await?;
                stream.write_u8(b'o').await?;
                if stream.read_u8().await? != 1 {
                    return Err(wire::invalid());
                }
                loop {
                    if stream.read_u32().await? != 1 || stream.read_u8().await? != wire::OBSERVE {
                        return Err(wire::invalid());
                    }
                    let result = observer_owner
                        .control_request(&[wire::OBSERVE])
                        .await
                        .map_err(|_| wire::invalid())?;
                    crate::live_transport::write_frame(&mut stream, Some(0), &result).await?;
                }
            });
        let (shutdown_send, shutdown) = std::sync::mpsc::sync_channel(1);
        let wake = shutdown_send.clone();
        let requested = Arc::new(AtomicBool::new(false));
        let requested_control = requested.clone();
        let stopped = Arc::new(AtomicBool::new(false));
        let stopped_control = stopped.clone();
        let stopped_wake = wake.clone();
        let (finished, mut finish) = tokio::sync::oneshot::channel();
        let owner = self.clone();
        let thread = self.0.scheduler.handle.spawn(async move {
            let result = async {
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
                    let _admitted = owner
                        .0
                        .scheduler
                        .admit_lifecycle(len + wire::MAX_FRAME)
                        .await?;
                    let mut bytes = vec![0; len];
                    stream.read_exact(&mut bytes).await?;
                    let shutdown_requested = bytes == [wire::SHUTDOWN];
                    let result = if shutdown_requested {
                        requested_control.store(true, Ordering::Release);
                        match shutdown_send.try_send(()) {
                            Ok(()) | Err(std::sync::mpsc::TrySendError::Full(())) => {}
                            Err(_) => return Err(wire::invalid()),
                        }
                        if (&mut finish).await.map_err(|_| wire::invalid())? {
                            Ok(Vec::new())
                        } else {
                            Err(PortError::Io)
                        }
                    } else {
                        owner.control_request(&bytes).await
                    };
                    if result.is_err()
                        && matches!(
                            bytes.first(),
                            Some(&wire::EDIT_BEGIN)
                                | Some(&wire::EDIT_PART)
                                | Some(&wire::EDIT_END)
                        )
                    {
                        owner.0.edit.lock().map_err(|_| wire::invalid())?.take();
                    }
                    match result {
                        Ok(bytes) => {
                            crate::live_transport::write_frame(&mut stream, Some(0), &bytes)
                                .await?;
                        }
                        Err(error) => {
                            crate::live_transport::write_frame(
                                &mut stream,
                                Some(1),
                                &[crate::protocol::error_code(error)],
                            )
                            .await?;
                        }
                    }
                    if shutdown_requested {
                        return Ok(());
                    }
                }
            }
            .await;
            stopped_control.store(true, Ordering::Release);
            let _ = stopped_wake.try_send(());
            result
        });
        Ok(LiveControl {
            shutdown,
            wake,
            requested,
            stopped,
            finished: Some(finished),
            thread: Some(thread),
            observer: Some(observer),
        })
    }
}

pub struct LiveControl {
    shutdown: std::sync::mpsc::Receiver<()>,
    wake: std::sync::mpsc::SyncSender<()>,
    requested: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    finished: Option<tokio::sync::oneshot::Sender<bool>>,
    thread: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
    observer: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
}
impl LiveControl {
    pub fn wait_for_shutdown(&self) -> std::io::Result<()> {
        self.shutdown.recv().map_err(|_| wire::invalid())?;
        if self.shutdown_requested() {
            Ok(())
        } else {
            Err(wire::invalid())
        }
    }
    pub fn shutdown_waker(&self) -> std::sync::mpsc::SyncSender<()> {
        self.wake.clone()
    }
    pub fn shutdown_requested(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }
    pub fn poll_shutdown(&self, timeout: std::time::Duration) -> std::io::Result<bool> {
        match self.shutdown.recv_timeout(timeout) {
            Ok(()) if self.stopped.load(Ordering::Acquire) && !self.shutdown_requested() => {
                Err(wire::invalid())
            }
            Ok(()) => Ok(true),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(false),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(wire::invalid()),
        }
    }
    pub fn cancel(mut self) -> std::io::Result<()> {
        let runtime = LiveRuntime::shared()?;
        for task in [self.thread.take(), self.observer.take()]
            .into_iter()
            .flatten()
        {
            task.abort();
            let _ = runtime.block_on(task);
        }
        Ok(())
    }

    pub fn finish_shutdown(mut self, success: bool) -> std::io::Result<()> {
        self.finished
            .take()
            .ok_or_else(wire::invalid)?
            .send(success)
            .map_err(|_| wire::invalid())?;
        let result = LiveRuntime::shared()?
            .block_on(self.thread.take().ok_or_else(wire::invalid)?)
            .map_err(|_| wire::invalid())?;
        if let Some(observer) = self.observer.take() {
            observer.abort();
            let _ = LiveRuntime::shared()?.block_on(observer);
        }
        result
    }
}

impl Drop for LiveControl {
    fn drop(&mut self) {
        if let Some(thread) = &self.thread {
            thread.abort();
        }
        if let Some(observer) = &self.observer {
            observer.abort();
        }
    }
}

fn ns(started: Instant) -> u64 {
    started.elapsed().as_nanos().min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod immutable_acquisition_tests {
    use super::*;
    use layerfs_content::{
        file::rope::FileStateRoot, tree::directory::DirectoryStateRoot, tree::inode::InodeId,
        ObjectId,
    };
    use layerfs_workspace_core::{DirectoryData, FileData, Node};
    use std::time::Duration;

    fn identity(label: &[u8]) -> ObjectId {
        ObjectId::for_bytes(label)
    }

    fn node(data: Data, directory: bool) -> Node {
        Node {
            revision: 0,
            canonical: Some(InodeId([if directory { 1 } else { 2 }; 32])),
            paths: Default::default(),
            mode: if directory { 0o755 } else { 0o644 },
            links: if directory { 2 } else { 1 },
            pins: 0,
            mtime_seconds: 0,
            mtime_nanoseconds: 0,
            data,
        }
    }

    fn seed() -> Vec<u8> {
        let mut root = node(
            Data::Directory(DirectoryData {
                base: Some(DirectoryStateRoot(identity(b"directory"))),
                changes: Default::default(),
            }),
            true,
        );
        root.paths.insert(String::new());
        let policy = ResourcePolicy::default();
        let mut out = identity(b"old namespace").as_bytes().to_vec();
        wire::bytes_out(&mut out, &[]).unwrap();
        wire::u64_out(&mut out, policy.max_spool_bytes);
        wire::u64_out(&mut out, policy.max_final_delta_memory_bytes);
        out.extend(wire::node_out(ROOT, &root).unwrap());
        out
    }

    fn fact(content: &[u8], declared_len: u64) -> Vec<u8> {
        let file = node(
            Data::File(FileData::Base {
                root: FileStateRoot(identity(b"file root")),
                len: declared_len,
            }),
            false,
        );
        let mut out = Vec::new();
        wire::bytes_out(&mut out, &wire::node_out(NodeId(2), &file).unwrap()).unwrap();
        wire::bytes_out(&mut out, content).unwrap();
        out
    }

    fn lookup_reply(content: &[u8], declared_len: u64) -> Vec<u8> {
        let mut out = vec![1];
        out.extend(fact(content, declared_len));
        out.extend_from_slice(&1u32.to_be_bytes());
        wire::bytes_out(&mut out, b"sibling").unwrap();
        out.extend(fact(b"old", 3));
        out.push(0);
        out
    }

    #[test]
    fn delayed_lookup_cannot_install_after_root_or_parent_revision_changes() {
        let runtime = LiveRuntime::new().unwrap();
        for change_root in [true, false] {
            let (entered, pending) = std::sync::mpsc::channel();
            let (release, released) = std::sync::mpsc::channel();
            let released = Mutex::new(released);
            let handler = Arc::new(move |request: &[u8]| match request.first() {
                Some(&wire::SEED) => Ok(seed()),
                Some(&wire::LOOKUP) => {
                    entered.send(()).unwrap();
                    released
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap();
                    Ok(lookup_reply(b"old", 3))
                }
                _ => Err(PortError::Io),
            });
            let owner = runtime
                .block_on(LiveOwner::local(
                    handler,
                    Arc::new(|_| Ok(())),
                    runtime.scheduler(),
                ))
                .unwrap();
            let acquiring = owner.clone();
            let task = runtime
                .scheduler()
                .handle
                .spawn(async move { acquiring.lookup_async(ROOT, b"file").await });
            pending.recv_timeout(Duration::from_secs(5)).unwrap();
            // Exercise the install race directly: a completed checkpoint changes
            // base_root; an SDK namespace/metadata edit changes the parent revision.
            let nodes_after_change = {
                let mut state = owner.state().unwrap();
                if change_root {
                    state.base_root = identity(b"new namespace");
                } else {
                    state.chmod(ROOT, 0o700).unwrap();
                }
                state.nodes.clone()
            };
            release.send(()).unwrap();
            assert_eq!(runtime.block_on(task).unwrap(), Err(PortError::Io));
            assert_eq!(owner.state().unwrap().nodes, nodes_after_change);
            assert!(
                owner
                    .0
                    .scheduler
                    .immutable_reads()
                    .get_name(
                        owner.0.read_scope,
                        identity(b"old namespace"),
                        identity(b"directory"),
                        b"sibling",
                    )
                    .is_none(),
                "stale acquisition must not install sibling metadata"
            );
        }
    }

    #[test]
    fn malformed_grouped_replies_do_not_install_live_nodes() {
        let runtime = LiveRuntime::new().unwrap();
        let mut truncated = lookup_reply(b"old", 3);
        truncated.pop();
        let oversized = vec![b'x'; wire::IMMUTABLE_PREFETCH_FILE_BYTES + 1];
        let mut invalid_count = vec![0];
        invalid_count.extend_from_slice(&128u32.to_be_bytes());
        for reply in [
            lookup_reply(b"no", 3),
            lookup_reply(&oversized, oversized.len() as u64),
            truncated,
            invalid_count,
        ] {
            let handler = Arc::new(move |request: &[u8]| match request.first() {
                Some(&wire::SEED) => Ok(seed()),
                Some(&wire::LOOKUP) => Ok(reply.clone()),
                _ => Err(PortError::Io),
            });
            let owner = runtime
                .block_on(LiveOwner::local(
                    handler,
                    Arc::new(|_| Ok(())),
                    runtime.scheduler(),
                ))
                .unwrap();
            let before = owner.state().unwrap().nodes.clone();
            assert_eq!(
                runtime.block_on(owner.lookup_async(ROOT, b"file")),
                Err(PortError::Io)
            );
            assert_eq!(owner.state().unwrap().nodes, before);
            assert!(owner
                .0
                .scheduler
                .immutable_reads()
                .get_name(
                    owner.0.read_scope,
                    identity(b"old namespace"),
                    identity(b"directory"),
                    b"sibling",
                )
                .is_none());
        }
    }
}

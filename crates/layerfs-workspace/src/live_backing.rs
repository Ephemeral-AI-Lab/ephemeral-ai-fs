//! Host-only physical backing and immutable input for a remote live owner.
use crate::cow_tree::{acquire_inode, acquire_inodes, WorkspaceSnapshot};
use crate::file_io::{spool_segment, HostSpool};
use crate::ResourcePolicy;
use layerfs_content::file::content::{read_range, FileContentRoot};
use layerfs_content::tree::directory::{
    DirectoryLookupCache, DirectoryStateRoot, NamespaceCounters,
};
use layerfs_content::tree::inode::InodeTableRoot;
use layerfs_content::{CanonicalName, ObjectId};
use layerfs_fuse::live_wire::{self as wire, Input};
use layerfs_layerstack_store::{CoreReader, Result, StoreError};
use layerfs_workspace_core::backing::{BackingId, BackingRef};
use layerfs_workspace_core::{Node, NodeId};
use std::collections::{BTreeSet, HashMap, HashSet};
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
    directory_lookup: DirectoryLookupCache,
    request_profile: [u64; 16],
    lookup_profile: [u64; 2],
    exported_contents: HashSet<ObjectId>,
    facts_complete: bool,
    fact_reservations: Vec<layerfs_fuse::live_runtime::LiveReservation>,
    incoming_reservations: Vec<layerfs_fuse::live_runtime::LiveReservation>,
    incoming_charge: usize,
    partial_fact: Option<PartialFact>,
    incoming: Option<(u64, HashMap<NodeId, Node>, BTreeSet<NodeId>)>,
}

impl Drop for BackingOwner {
    fn drop(&mut self) {
        if std::env::var_os("LAYERFS_BACKING_PROFILE").is_some() {
            println!(
                "{{\"kind\":\"backing-profile\",\"counts_by_opcode\":{:?},\"lookup_negative\":{},\"lookup_positive\":{}}}",
                self.request_profile, self.lookup_profile[0], self.lookup_profile[1]
            );
        }
    }
}

struct PartialFact {
    dirty: bool,
    total: usize,
    encoded: Vec<u8>,
    _charge: layerfs_fuse::live_runtime::LiveReservation,
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
            directory_lookup: DirectoryLookupCache::default(),
            request_profile: [0; 16],
            lookup_profile: [0; 2],
            exported_contents: HashSet::new(),
            facts_complete: true,
            incoming: None,
            fact_reservations: Vec::new(),
            incoming_reservations: Vec::new(),
            incoming_charge: 0,
            partial_fact: None,
        }
    }

    pub(crate) fn frozen_generation(&self) -> Result<u64> {
        if !self.facts_complete {
            return Err(StoreError::Integrity("incomplete backing group"));
        }
        Ok(self.generation)
    }

    fn install_fact(&mut self, dirty: bool, encoded: &[u8]) -> Result<()> {
        let charge = encoded
            .len()
            .checked_mul(8)
            .and_then(|n| n.checked_add(1024))
            .ok_or(StoreError::InvalidInput("backing fact limit"))?;
        let total = self
            .incoming_charge
            .checked_add(charge)
            .filter(|n| *n <= wire::MAX_FACT_MEMORY)
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
            if offset
                .checked_add(len)
                .is_none_or(|end| end > segment.len.load(std::sync::atomic::Ordering::Relaxed))
            {
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
        Ok(())
    }

    fn acquired_out(
        &mut self,
        acquired: layerfs_workspace_core::namespace::AcquiredInode,
        content_budget: &mut usize,
    ) -> Result<Vec<u8>> {
        let node = Node {
            revision: 0,
            canonical: Some(acquired.inode),
            paths: BTreeSet::new(),
            mode: acquired.mode,
            links: acquired.links,
            pins: 0,
            mtime_seconds: acquired.mtime_seconds,
            mtime_nanoseconds: acquired.mtime_nanoseconds,
            data: acquired.data,
        };
        let mut out = Vec::new();
        wire::bytes_out(&mut out, &wire::node_out(NodeId(1), &node)?)?;
        let mut content = Vec::new();
        if let layerfs_workspace_core::Data::File(layerfs_workspace_core::FileData::Base {
            root,
            len,
        }) = &node.data
        {
            if *len <= wire::IMMUTABLE_PREFETCH_FILE_BYTES as u64
                && *len <= *content_budget as u64
                && !self.exported_contents.contains(&root.0)
            {
                // Optional acquisition never substitutes for a demanded read error.
                match read_range(
                    &CoreReader(&self.snapshot.reader),
                    *root,
                    0..*len,
                    &mut content,
                ) {
                    Ok(counters) => {
                        self.snapshot.reader.note_rope_read(counters)?;
                        if content.len() as u64 == *len {
                            *content_budget -= content.len();
                            // Hints only suppress optional exports. Eviction or a lost
                            // response always falls back to a normal demanded READ_BASE.
                            if self.exported_contents.len() == 8192 {
                                self.exported_contents.clear();
                            }
                            self.exported_contents.insert(root.0);
                        } else {
                            content.clear();
                        }
                    }
                    Err(_) => content.clear(),
                }
            }
        }
        wire::bytes_out(&mut out, &content)?;
        Ok(out)
    }

    pub(crate) fn request(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
        if let Some(count) = bytes
            .first()
            .and_then(|opcode| self.request_profile.get_mut(*opcode as usize))
        {
            *count += 1;
        }
        let mut input = Input(bytes);
        let mut out = Vec::new();
        match input.byte()? {
            wire::SEED => {
                input.done()?;
                out.extend_from_slice(self.snapshot.root.as_bytes());
                wire::bytes_out(
                    &mut out,
                    self.snapshot
                        .expected_head
                        .as_ref()
                        .map_or(Vec::new(), |head| head.to_bytes().to_vec())
                        .as_slice(),
                )?;
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
                let inode = self.directory_lookup.lookup(
                    &core,
                    directory,
                    &name,
                    &mut NamespaceCounters::default(),
                )?;
                let mut content_budget = wire::IMMUTABLE_PREFETCH_PAGE_BYTES;
                self.lookup_profile[usize::from(inode.is_some())] += 1;
                out.push(u8::from(inode.is_some()));
                if let Some(inode) = inode {
                    let acquired = acquire_inode(
                        &self.snapshot.reader,
                        InodeTableRoot(namespace.inode_table_root),
                        inode,
                    )?;
                    out.extend(self.acquired_out(acquired, &mut content_budget)?);
                }
                let mut siblings = Vec::new();
                // Reuse the entire already validated leaf, including earlier names.
                // Only a fully exported root leaf establishes directory completeness.
                let entries = self
                    .directory_lookup
                    .leaf_entries(directory)
                    .iter()
                    .filter(|(candidate, _)| candidate != &name)
                    .take(127)
                    .cloned()
                    .collect::<Vec<_>>();
                let ids = entries.iter().map(|(_, inode)| *inode).collect::<Vec<_>>();
                if let Ok(acquired) = acquire_inodes(
                    &self.snapshot.reader,
                    InodeTableRoot(namespace.inode_table_root),
                    &ids,
                ) {
                    for ((name, _), acquired) in entries.into_iter().zip(acquired) {
                        if let Ok(node) = self.acquired_out(acquired, &mut content_budget) {
                            siblings.push((name, node));
                        }
                    }
                }
                let complete = self.directory_lookup.leaf_complete(directory)
                    && siblings.len() + usize::from(inode.is_some())
                        == self.directory_lookup.leaf_entries(directory).len();
                out.extend_from_slice(&(siblings.len() as u32).to_be_bytes());
                for (name, node) in siblings {
                    wire::bytes_out(&mut out, name.as_bytes())?;
                    out.extend(node);
                }
                out.push(u8::from(complete));
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
                let acquired = acquire_inodes(
                    &self.snapshot.reader,
                    InodeTableRoot(namespace.inode_table_root),
                    &ids,
                )?;
                let mut content_budget = wire::IMMUTABLE_PREFETCH_PAGE_BYTES;
                for ((name, _), acquired) in page.entries.into_iter().zip(acquired) {
                    wire::bytes_out(&mut out, name.as_bytes())?;
                    out.extend(self.acquired_out(acquired, &mut content_budget)?);
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
                #[cfg(feature = "test-instrumentation")]
                if crate::lifecycle::consume_verification_fault(
                    self.snapshot.branch_id,
                    crate::lifecycle::VerificationFault::NoSpace,
                    self.spool.bytes,
                ) {
                    return Err(StoreError::InvalidInput("workspace spool limit"));
                }
                #[cfg(feature = "test-instrumentation")]
                let inject_short = crate::lifecycle::consume_verification_fault(
                    self.snapshot.branch_id,
                    crate::lifecycle::VerificationFault::ShortAppend,
                    self.spool.bytes,
                );
                self.spool.append(
                    &segment,
                    offset,
                    data,
                    #[cfg(feature = "test-instrumentation")]
                    inject_short,
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
                let root = FileContentRoot(input.object()?);
                let offset = input.u64()?;
                let len = input.u32()? as usize;
                input.done()?;
                if len > 1024 * 1024 {
                    return Err(StoreError::InvalidInput("immutable read"));
                }
                let counters = read_range(
                    &CoreReader(&self.snapshot.reader),
                    root,
                    offset
                        ..offset
                            .checked_add(len as u64)
                            .ok_or(StoreError::InvalidInput("immutable range"))?,
                    &mut out,
                )?;
                self.snapshot.reader.note_rope_read(counters)?;
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
                // Construction holds this same backing lock through consumption.
                // No producer can retain these facts after this lock is acquired.
                self.facts.clear();
                self.dirty.clear();
                self.fact_reservations.clear();
                self.facts_complete = false;
                self.incoming_charge = 0;
                self.incoming_reservations.clear();
                self.incoming = Some((generation, HashMap::new(), BTreeSet::new()));
            }
            wire::FACTS_NODE => {
                if self.partial_fact.is_some() {
                    return Err(StoreError::Integrity("unfinished backing node"));
                }
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
                    self.install_fact(dirty, encoded)?;
                }
            }
            wire::FACTS_NODE_BEGIN => {
                let dirty = match input.byte()? {
                    0 => false,
                    1 => true,
                    _ => return Err(StoreError::Integrity("backing dirty flag")),
                };
                let total = usize::try_from(input.u64()?)
                    .map_err(|_| StoreError::InvalidInput("backing node length"))?;
                input.done()?;
                if total == 0
                    || total > wire::MAX_NODE_BYTES
                    || self.partial_fact.is_some()
                    || self.incoming.is_none()
                {
                    return Err(StoreError::InvalidInput("backing node length"));
                }
                let charge = layerfs_fuse::live_runtime::LiveRuntime::shared()?
                    .scheduler()
                    .reserve_live(total)?;
                let mut encoded = Vec::new();
                encoded
                    .try_reserve_exact(total)
                    .map_err(|_| StoreError::InvalidInput("backing node allocation"))?;
                self.partial_fact = Some(PartialFact {
                    dirty,
                    total,
                    encoded,
                    _charge: charge,
                });
            }
            wire::FACTS_NODE_CHUNK => {
                let partial = self
                    .partial_fact
                    .as_mut()
                    .ok_or(StoreError::Integrity("missing backing node"))?;
                if input.0.is_empty() || input.0.len() > partial.total - partial.encoded.len() {
                    return Err(StoreError::InvalidInput("backing node chunk"));
                }
                partial.encoded.extend_from_slice(input.0);
                if partial.encoded.len() == partial.total {
                    let partial = self.partial_fact.take().unwrap();
                    self.install_fact(partial.dirty, &partial.encoded)?;
                }
            }
            wire::FACTS_END => {
                if self.partial_fact.is_some() {
                    return Err(StoreError::Integrity("unfinished backing node"));
                }
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
                self.facts_complete = true;
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
    fn cold_grouped_reads_preserve_alias_mutations_and_peer_isolation() {
        use layerfs_fuse::FilesystemPort;
        let directory = std::env::temp_dir().join(format!(
            "layerfs-live-cold-cache-{}",
            crate::WorkspaceId::new()
        ));
        let fixture = directory.join("fixture");
        std::fs::create_dir_all(&fixture).unwrap();
        std::fs::write(fixture.join("a"), b"original").unwrap();
        std::fs::hard_link(fixture.join("a"), fixture.join("b")).unwrap();
        std::fs::write(fixture.join("c"), b"untouched").unwrap();
        std::fs::write(fixture.join("d"), b"replace-me").unwrap();
        let store = LayerStackStore::create(directory.join("store.sqlite")).unwrap();
        let layer = store
            .initialize_layerstack(
                EntityName::new("project").unwrap(),
                LayerStackInitialization::Directory(fixture),
            )
            .unwrap()
            .genesis_layer_id;
        let first_branch = store
            .fork_branch(
                EntityName::new("first").unwrap(),
                LocalForkSource::Layer { layer_id: layer },
            )
            .unwrap();
        let peer_branch = store
            .fork_branch(
                EntityName::new("peer").unwrap(),
                LocalForkSource::Layer { layer_id: layer },
            )
            .unwrap();
        let first_workspace =
            crate::Workspace::open(store.clone(), first_branch, directory.join("first-spool"))
                .unwrap();
        let peer_workspace =
            crate::Workspace::open(store.clone(), peer_branch, directory.join("peer-spool"))
                .unwrap();
        let original_root = store.pin_branch(first_branch).unwrap().root;
        // Both start_local calls use LiveRuntime::shared(): one cache/scheduler,
        // distinct owners, capabilities, mutable state and spool backing.
        let first_remote = RemoteWorkspace::start_local(&first_workspace).unwrap();
        let peer_remote = RemoteWorkspace::start_local(&peer_workspace).unwrap();
        let first = first_remote.server.local_owner().unwrap();
        let peer = peer_remote.server.local_owner().unwrap();
        first_remote.server.take_write_metrics().unwrap();
        let a = first.lookup(crate::ROOT, b"a").unwrap().node;
        assert_eq!(first.read(a, 0, 100).unwrap(), b"original");
        assert!(
            first_remote
                .server
                .take_write_metrics()
                .unwrap()
                .live_backing_calls
                > 0
        );
        let c = first.lookup(crate::ROOT, b"c").unwrap().node;
        assert_eq!(first.read(c, 0, 100).unwrap(), b"untouched");
        assert_eq!(
            first.lookup(crate::ROOT, b"absent"),
            Err(layerfs_fuse::PortError::NotFound)
        );
        assert_eq!(
            first_remote
                .server
                .take_write_metrics()
                .unwrap()
                .live_backing_calls,
            0
        );

        first.write(a, 0, b"modified").unwrap();
        first_remote.server.take_write_metrics().unwrap();
        let b = first.lookup(crate::ROOT, b"b").unwrap().node;
        assert_eq!(
            a, b,
            "prefetched alias must retain the already edited live inode"
        );
        assert_eq!(
            first_remote
                .server
                .take_write_metrics()
                .unwrap()
                .live_backing_calls,
            0
        );
        assert_eq!(first.read(b, 0, 100).unwrap(), b"modified");
        // d was prefetched but not installed: remove/recreate must beat cached facts.
        first.unlink(crate::ROOT, b"d", false).unwrap();
        assert!(first.lookup(crate::ROOT, b"d").is_err());
        let replacement = first.create_file(crate::ROOT, b"d", 0o600).unwrap().node;
        first.write(replacement, 0, b"new").unwrap();
        assert_eq!(first.lookup(crate::ROOT, b"d").unwrap().node, replacement);
        assert_eq!(first.read(replacement, 0, 100).unwrap(), b"new");

        peer_remote.server.take_write_metrics().unwrap();
        let peer_a = peer.lookup(crate::ROOT, b"a").unwrap().node;
        assert_eq!(peer.read(peer_a, 0, 100).unwrap(), b"original");
        assert!(
            peer_remote
                .server
                .take_write_metrics()
                .unwrap()
                .live_backing_calls
                > 0,
            "a distinct capability must authenticate its own cold acquisition"
        );
        assert_eq!(peer.lookup(crate::ROOT, b"b").unwrap().node, peer_a);
        let peer_d = peer.lookup(crate::ROOT, b"d").unwrap().node;
        assert_eq!(peer.read(peer_d, 0, 100).unwrap(), b"replace-me");
        assert_eq!(store.pin_branch(first_branch).unwrap().root, original_root);
        assert_eq!(store.pin_branch(peer_branch).unwrap().root, original_root);
        first_remote.server.control("shutdown").unwrap();
        peer_remote.server.control("shutdown").unwrap();
        drop((first, peer, first_remote, peer_remote));
        drop((first_workspace, peer_workspace, store));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn live_owner_builds_and_checkpoints_through_real_host_backing() {
        live_owner_check(false);
    }

    #[test]
    fn local_owner_builds_and_checkpoints_through_direct_host_backing() {
        live_owner_check(true);
    }

    fn live_owner_check(local: bool) {
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
        let remote = if local {
            RemoteWorkspace::start_local(&workspace)
        } else {
            RemoteWorkspace::start(&workspace)
        }
        .unwrap();
        let runtime = LiveRuntime::new().unwrap();
        let (owner, control) = if local {
            assert_eq!(
                remote.server.port(),
                0,
                "native backing does not listen on a socket"
            );
            (remote.server.local_owner().unwrap(), None)
        } else {
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
            (owner, Some(control))
        };
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
        let callback = runtime
            .block_on(owner.callback_gate(layerfs_fuse::KernelOperation::Write, false))
            .unwrap();
        let server = remote.server.clone();
        let (sent, received) = std::sync::mpsc::channel();
        let freezing = std::thread::spawn(move || {
            sent.send(server.control("pause")).unwrap();
        });
        assert!(matches!(
            received.recv_timeout(std::time::Duration::from_millis(20)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        let observing = remote.clone();
        let (sent, observation) = std::sync::mpsc::channel();
        let observer = std::thread::spawn(move || {
            sent.send(observing.observe()).unwrap();
        });
        assert_eq!(
            observation
                .recv_timeout(std::time::Duration::from_secs(1))
                .unwrap()
                .unwrap()
                .0,
            263
        );
        observer.join().unwrap();
        drop(callback);
        received
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap()
            .unwrap();
        freezing.join().unwrap();
        remote.server.control("resume").unwrap();
        let metrics = remote.server.take_write_metrics().unwrap();
        assert!(metrics.live_backing_calls > 0 && metrics.live_backing_calls < 130);
        assert!(metrics.live_backing_request_bytes > 0);
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
        remote.server.take_write_metrics().unwrap();
        remote
            .edit(
                "bulk/file",
                vec![crate::WorkspaceFileRangeEdit {
                    workspace_id: crate::WorkspaceId::new(),
                    path: "bulk/file".into(),
                    start: 1,
                    delete_len: 2,
                    replacement: crate::WorkspaceFileReplacement::Inline(b"XY".to_vec()),
                }],
            )
            .unwrap();
        assert_eq!(owner.read(file, 0, 20).unwrap(), b"fXYst");
        let sdk_metrics = remote.server.take_write_metrics().unwrap();
        if !local {
            assert!(sdk_metrics.live_backing_request_bytes > 0);
        }
        assert_eq!(sdk_metrics.client_frame_bytes, 0);
        assert_eq!(sdk_metrics.host_frame_bytes, 0);
        assert_eq!(sdk_metrics.frame_payload_copy_bytes, 0);
        let streamed = owner
            .create_file_open(crate::ROOT, b"streamed", 0o600)
            .unwrap()
            .node;
        let splice = |start, delete_len, bytes| crate::WorkspaceFileRangeEdit {
            workspace_id: crate::WorkspaceId::new(),
            path: "streamed".into(),
            start,
            delete_len,
            replacement: crate::WorkspaceFileReplacement::Inline(bytes),
        };
        remote
            .edit(
                "streamed",
                (0..9)
                    .map(|index| {
                        splice(
                            0,
                            if index == 0 { 0 } else { 1024 * 1024 },
                            vec![index; 1024 * 1024],
                        )
                    })
                    .collect(),
            )
            .unwrap();
        assert_eq!(owner.attr(streamed).unwrap().size, 1024 * 1024);
        assert_eq!(owner.read(streamed, 0, 10).unwrap(), vec![8; 10]);
        assert!(remote
            .edit(
                "streamed",
                vec![splice(0, 1, vec![99]), splice(2 * 1024 * 1024, 0, vec![1])]
            )
            .is_err());
        assert_eq!(owner.read(streamed, 0, 10).unwrap(), vec![8; 10]);
        owner.write(file, 0, b"later").unwrap();
        remote
            .edit(
                "streamed",
                (0..8)
                    .map(|index| {
                        splice(
                            index * 1024 * 1024,
                            if index == 0 { 1024 * 1024 } else { 0 },
                            vec![index as u8; 1024 * 1024],
                        )
                    })
                    .collect(),
            )
            .unwrap();
        assert_eq!(owner.attr(streamed).unwrap().size, 8 * 1024 * 1024);
        owner
            .fsync(None)
            .expect("accepted eight-MiB inline file can transfer frozen facts");
        assert!(remote.backing.lock().unwrap().facts.contains_key(&streamed));
        remote
            .edit("streamed", vec![splice(0, 1, vec![42])])
            .unwrap();
        owner
            .fsync(None)
            .expect("replacement facts reuse the released prior group budget");
        assert_eq!(owner.read(streamed, 0, 1).unwrap(), vec![42]);
        owner.unlink(crate::ROOT, b"streamed", false).unwrap();
        owner.unpin(streamed, true).unwrap();
        remote.server.control("pause").unwrap();
        let (second, _) = workspace.lock().unwrap().commit().unwrap();
        install_checkpoint(&workspace).unwrap();
        remote.server.control("resume").unwrap();
        assert_ne!(first, second);
        assert_eq!(owner.read(file, 0, 20).unwrap(), b"later");
        let orphan = owner
            .create_file_open(crate::ROOT, b"orphan", 0o600)
            .unwrap()
            .node;
        owner.write(orphan, 0, b"open after unlink").unwrap();
        owner.unlink(crate::ROOT, b"orphan", false).unwrap();
        remote.server.control("pause").unwrap();
        workspace.lock().unwrap().commit().unwrap();
        install_checkpoint(&workspace).unwrap();
        remote.server.control("resume").unwrap();
        assert_eq!(owner.read(orphan, 0, 100).unwrap(), b"open after unlink");
        assert!(!remote.backing.lock().unwrap().spool.segments.is_empty());
        owner.unpin(orphan, true).unwrap();
        owner.fsync(None).unwrap();
        assert!(
            remote.backing.lock().unwrap().spool.segments.is_empty(),
            "last-release backing retires at a no-op fence"
        );
        owner
            .write(file, 0, b"retained after host failure")
            .unwrap();
        remote.backing.lock().unwrap().append_reservation = None;
        assert!(remote.server.control("pause").is_err());
        assert!(
            remote.server.control("pause").is_err(),
            "a retained cut cannot acknowledge failed backing on retry"
        );
        assert_eq!(
            owner.read(file, 0, 100).unwrap(),
            b"retained after host failure"
        );
        assert!(owner.write(file, 0, b"must not replay").is_err());
        owner.unpin(file, true).unwrap();
        if let Some(control) = control {
            let server = remote.server.clone();
            let ending = std::thread::spawn(move || server.control("shutdown"));
            control.wait_for_shutdown().unwrap();
            control.finish_shutdown(true).unwrap();
            ending.join().unwrap().unwrap();
        } else {
            remote.server.control("shutdown").unwrap();
        }
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
        #[cfg(feature = "test-instrumentation")]
        for (fault, expected) in [
            (
                crate::VerificationFault::ShortAppend,
                layerfs_fuse::PortError::Io,
            ),
            (
                crate::VerificationFault::NoSpace,
                layerfs_fuse::PortError::NoSpace,
            ),
        ] {
            let response = owner.request(&reserve).unwrap();
            let mut input = Input(&response);
            let mut append = vec![wire::APPEND];
            wire::u64_out(&mut append, input.u64().unwrap());
            wire::u64_out(&mut append, input.u64().unwrap());
            wire::bytes_out(&mut append, b"bad").unwrap();
            crate::arm_verification_fault(branch, fault).unwrap();
            let error = owner.request(&append).unwrap_err();
            assert_eq!(crate::projection::storage_port_error(error), expected);
            let receipt = crate::take_verification_fault_receipt().unwrap().unwrap();
            assert_eq!(receipt.hit_count, 1);
            assert!(owner.append_reservation.is_none());
            assert_eq!(owner.request(&read).unwrap(), b"abc");
        }
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
        Self::start_with_placement(workspace, false)
    }
    #[cfg(any(test, all(target_os = "linux", feature = "host-fuse")))]
    pub(crate) fn start_local(workspace: &crate::Workspace) -> Result<Self> {
        Self::start_with_placement(workspace, true)
    }

    fn start_with_placement(workspace: &crate::Workspace, local: bool) -> Result<Self> {
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
        let handler: Arc<layerfs_fuse::live_transport::BackingHandler> = Arc::new(move |bytes| {
            handler
                .lock()
                .map_err(|_| layerfs_fuse::PortError::Io)?
                .request(bytes)
                .map_err(crate::projection::storage_port_error)
        });
        let server = if local {
            let target = backing.clone();
            let sink = Arc::new(move |mut facts: layerfs_fuse::live_owner::LocalFacts| {
                use layerfs_workspace_core::file_edit::{Piece, PieceTree};
                let mut backing = target.lock().map_err(|_| layerfs_fuse::PortError::Io)?;
                if facts.root != backing.snapshot.root {
                    return Err(layerfs_fuse::PortError::Io);
                }
                for node in facts.nodes.values_mut() {
                    if let layerfs_workspace_core::Data::File(
                        layerfs_workspace_core::FileData::Edited { pieces, .. },
                    ) = &mut node.data
                    {
                        let values = pieces
                            .pieces()
                            .into_iter()
                            .map(|piece| match piece {
                                Piece::Spool {
                                    segment,
                                    offset,
                                    len,
                                } => {
                                    let physical = backing
                                        .spool
                                        .segments
                                        .get(&segment.id().0)
                                        .ok_or(layerfs_fuse::PortError::Io)?;
                                    if offset.checked_add(len).is_none_or(|end| {
                                        spool_segment(physical).map_or(true, |segment| {
                                            end > segment
                                                .len
                                                .load(std::sync::atomic::Ordering::Relaxed)
                                        })
                                    }) {
                                        return Err(layerfs_fuse::PortError::Io);
                                    }
                                    Ok(Piece::Spool {
                                        segment: physical.clone(),
                                        offset,
                                        len,
                                    })
                                }
                                piece => Ok(piece),
                            })
                            .collect::<std::result::Result<Vec<_>, _>>()?;
                        *pieces = PieceTree::empty()
                            .replace(0, 0, values)
                            .map_err(|_| layerfs_fuse::PortError::Io)?;
                    }
                }
                for id in &facts.dirty {
                    let node = facts.nodes.get(id).ok_or(layerfs_fuse::PortError::Io)?;
                    if let layerfs_workspace_core::Data::Directory(directory) = &node.data {
                        if directory
                            .changes
                            .values()
                            .flatten()
                            .any(|id| !facts.nodes.contains_key(id))
                        {
                            return Err(layerfs_fuse::PortError::Io);
                        }
                    }
                }
                backing.facts = facts.nodes;
                backing.dirty = facts.dirty;
                backing.generation = facts.generation;
                backing.fact_reservations = vec![facts.charge];
                backing.facts_complete = true;
                Ok(())
            });
            let runtime = layerfs_fuse::live_runtime::LiveRuntime::shared()?;
            let owner = runtime
                .block_on(layerfs_fuse::live_owner::LiveOwner::local(
                    handler,
                    sink,
                    runtime.scheduler(),
                ))
                .map_err(|_| StoreError::Integrity("local live owner"))?;
            layerfs_fuse::live_transport::BackingServer::local(owner)
        } else {
            layerfs_fuse::live_transport::BackingServer::start(move |bytes| handler(bytes))?
        };
        Ok(Self {
            backing,
            server: Arc::new(server),
        })
    }

    pub(crate) fn edit(
        &self,
        path: &str,
        edits: Vec<crate::WorkspaceFileRangeEdit>,
    ) -> crate::WorkspaceResult<()> {
        if edits.is_empty()
            || edits.len() > layerfs_workspace_core::file_edit::MAX_EDITS_PER_FILE as usize
        {
            return Err(crate::WorkspaceError::InvalidExecution);
        }
        if edits.iter().any(|edit| matches!(&edit.replacement, crate::WorkspaceFileReplacement::Inline(bytes) if bytes.len() > 1024 * 1024)) {
            return Err(crate::WorkspaceError::InvalidExecution);
        }
        let _charge = layerfs_fuse::live_runtime::LiveRuntime::shared()?
            .scheduler()
            .reserve_live(wire::MAX_FRAME)?;
        let mut begin = vec![wire::EDIT_BEGIN];
        wire::bytes_out(&mut begin, path.as_bytes())?;
        wire::u64_out(&mut begin, edits.len() as u64);
        let parts = edits.into_iter().map(|edit| {
            let mut frame = vec![wire::EDIT_PART];
            wire::u64_out(&mut frame, edit.start);
            wire::u64_out(&mut frame, edit.delete_len);
            match edit.replacement {
                crate::WorkspaceFileReplacement::Inline(bytes) => {
                    frame.push(0);
                    wire::bytes_out(&mut frame, &bytes).expect("validated inline length");
                }
                crate::WorkspaceFileReplacement::Zero(len) => {
                    frame.push(1);
                    wire::u64_out(&mut frame, len);
                }
            }
            frame
        });
        self.server
            .request_group(
                std::iter::once(begin)
                    .chain(parts)
                    .chain(std::iter::once(vec![wire::EDIT_END])),
            )
            .map(drop)
            .map_err(|_| crate::WorkspaceError::InvalidExecution)
    }

    pub(crate) fn observe(
        &self,
    ) -> crate::WorkspaceResult<(u64, u64, u64, Option<layerfs_layerstack_store::CommitId>)> {
        let response = self
            .server
            .observe()
            .map_err(|_| crate::WorkspaceError::InvalidExecution)?;
        let mut input = Input(&response);
        let result = (
            input.u64()?,
            input.u64()?,
            input.u64()?,
            input
                .head()?
                .map(layerfs_layerstack_store::CommitId::from_bytes)
                .transpose()?,
        );
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
        let head = match outcome {
            CommitOutcome::Committed { commit_id, .. } => Some(commit_id),
            _ => workspace.expected_head,
        };
        (
            remote,
            outcome,
            base,
            head,
            workspace.pending_checkpoint.take(),
        )
    };
    let (remote, outcome, base, head, checkpoint) = pending;
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
                            root: FileContentRoot(content),
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
            wire::bytes_out(
                &mut end,
                head.as_ref()
                    .map_or(Vec::new(), |head| head.to_bytes().to_vec())
                    .as_slice(),
            )?;
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
    let remote = worker
        .remote
        .lock()
        .map_err(|_| crate::WorkspaceError::WorkspaceBusy)?
        .clone();
    if let Some(remote) = remote {
        return remote.observe().map(|values| values.0);
    }
    Ok(worker
        .workspace
        .lock()
        .map_err(|_| crate::WorkspaceError::WorkspaceBusy)?
        .live
        .mutation_generation)
}

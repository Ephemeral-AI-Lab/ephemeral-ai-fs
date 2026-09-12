//! Sorted final-state updates. Only the last sibling is kept private until its
//! neighbour establishes the final partition; untouched subtrees remain IDs.
use super::directory::codec::{decode_directory_node, encode_directory_node, DirectoryNodeV1};
use super::inode::codec::{decode_inode_table_node, encode_inode_table_node, InodeTableNodeV1};
use super::inode::InodeId;
use crate::file::rope::ObjectStore;
use crate::{CanonicalName, CoreError, CoreResult, ObjectId};
use std::{cell::Cell, marker::PhantomData, rc::Rc};

/// The caller reserves this together with its delta stream and other live state.
/// Every internal page/decode allocation is charged before allocation.
pub const SORTED_TREE_UPDATE_SCRATCH_BYTES: usize = 4 * 1024 * 1024;
const PAGE_ITEMS: usize = 234; // 8-KiB directory page, minimum 35-byte entry, plus overflow.
/// Every tree page the engine reads is bounded by the same 8-KiB ceiling the
/// point route enforces, so a bounded batch can be charged before it allocates.
const MAX_TREE_PAGE_BYTES: usize = 8192;
/// Sibling children read in one bounded authenticated batch. Small enough that
/// a whole chunk's worst-case retained bytes stay far inside the existing 4-MiB
/// tree ledger, large enough that consecutive siblings share their physical
/// groups and their pooled metadata values.
const TREE_BATCH_CHILDREN: usize = 32;
/// One bounded batch of children: how many of the requested entries were read,
/// their canonical pages, and the allocation lease retained while the caller
/// descends into them. No lease asks the caller for the ordinary point route.
type BatchChildren = (usize, Vec<(ObjectId, Vec<u8>)>, Option<Lease>);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeBatchCounters {
    pub nodes_read: u64,
    pub nodes_created: u64,
    pub nodes_reused: u64,
    pub delta_keys: u64,
    pub peak_scratch_bytes: usize,
}

struct Budget {
    used: Cell<usize>,
    peak: Cell<usize>,
    limit: usize,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            used: Cell::new(0),
            peak: Cell::new(0),
            limit: SORTED_TREE_UPDATE_SCRATCH_BYTES,
        }
    }
}
struct Lease {
    budget: Rc<Budget>,
    bytes: usize,
}
impl Budget {
    fn reserve(self: &Rc<Self>, bytes: usize) -> CoreResult<Lease> {
        let next = self
            .used
            .get()
            .checked_add(bytes)
            .ok_or(CoreError::LengthOverflow)?;
        if next > self.limit {
            return Err(CoreError::ObjectLimitExceeded);
        }
        self.used.set(next);
        self.peak.set(self.peak.get().max(next));
        Ok(Lease {
            budget: self.clone(),
            bytes,
        })
    }
}
impl Lease {
    fn grow(&mut self, bytes: usize) -> CoreResult<()> {
        let next = self
            .budget
            .used
            .get()
            .checked_add(bytes)
            .ok_or(CoreError::LengthOverflow)?;
        if next > self.budget.limit {
            return Err(CoreError::ObjectLimitExceeded);
        }
        self.budget.used.set(next);
        self.budget.peak.set(self.budget.peak.get().max(next));
        self.bytes += bytes;
        Ok(())
    }
    fn shrink(&mut self, bytes: usize) {
        self.bytes -= bytes;
        self.budget.used.set(self.budget.used.get() - bytes);
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.budget.used.set(self.budget.used.get() - self.bytes);
    }
}

struct Wire<K, V> {
    level: u8,
    count: u64,
    bytes: u64,
    entries: Vec<(K, ObjectId, V)>,
    size: usize,
}
struct ReadPage<K, V> {
    wire: Wire<K, V>,
    _lease: Lease,
}
trait Format {
    type Key: Ord + Clone;
    type Value: Default + Clone;
    fn leaf_value<S: crate::object::access::ObjectRead>(
        _store: &S,
        _id: ObjectId,
        value: Self::Value,
    ) -> CoreResult<Self::Value> {
        Ok(value)
    }
    fn decode(bytes: &[u8]) -> CoreResult<Wire<Self::Key, Self::Value>>;
    fn encode(page: &Page<Self::Key, Self::Value>) -> CoreResult<Vec<u8>>;
    fn width(key: &Self::Key, level: u8) -> usize;
    fn page_items(_level: u8) -> usize {
        PAGE_ITEMS
    }
    fn heap_bytes(key: &Self::Key) -> usize;
    fn decode_scratch(bytes: usize) -> usize;
    fn filled(size: usize, count: usize, level: u8) -> bool;
    fn fits(size: usize, count: usize, level: u8) -> bool;
    fn empty_allowed() -> bool;
}

struct Entry<K, V> {
    key: K,
    id: ObjectId,
    value: V,
    count: u64,
    bytes: u64,
    size: usize,
    items: usize,
    pending: Option<Box<Page<K, V>>>,
}
type PageSlot<K, V> = Option<Box<Page<K, V>>>;

struct Page<K, V> {
    level: u8,
    entries: Vec<Entry<K, V>>,
    origin: Option<ObjectId>,
    _lease: Lease,
}
struct Node<K, V> {
    max: Option<K>,
    id: Option<ObjectId>,
    level: u8,
    count: u64,
    bytes: u64,
    size: usize,
    items: usize,
    pending: Option<Box<Page<K, V>>>,
}
impl<K: Clone, V> Node<K, V> {
    fn existing(id: ObjectId, wire: &Wire<K, V>) -> Self {
        Self {
            max: wire.entries.last().map(|v| v.0.clone()),
            id: Some(id),
            level: wire.level,
            count: wire.count,
            bytes: wire.bytes,
            size: wire.size,
            items: wire.entries.len(),
            pending: None,
        }
    }
}
struct Engine<'a, S, F> {
    store: &'a mut S,
    changes: &'a mut dyn FnMut(Option<ObjectId>, Option<ObjectId>) -> CoreResult<()>,
    budget: Rc<Budget>,
    counters: TreeBatchCounters,
    format: PhantomData<F>,
}
impl<S: ObjectStore, F: Format> Engine<'_, S, F> {
    fn page(&self, level: u8) -> CoreResult<Page<F::Key, F::Value>> {
        if level > 31 {
            return Err(CoreError::MappingDepthExceeded);
        }
        let lease = self
            .budget
            .reserve(std::mem::size_of::<Page<F::Key, F::Value>>())?;
        Ok(Page {
            level,
            entries: Vec::new(),
            origin: None,
            _lease: lease,
        })
    }
    fn read(&mut self, id: ObjectId, root: bool) -> CoreResult<ReadPage<F::Key, F::Value>> {
        self.counters.nodes_read += 1;
        let budget = self.budget.clone();
        let (wire, lease) = self.store.with_authenticated_canonical(id, |bytes| {
            if bytes.len() > MAX_TREE_PAGE_BYTES {
                return Err(CoreError::ObjectLimitExceeded);
            }
            let lease = budget.reserve(
                F::decode_scratch(bytes.len()) + std::mem::size_of::<Wire<F::Key, F::Value>>(),
            )?;
            Ok((F::decode(bytes)?, lease))
        })?;
        Self::check_page(root, &wire)?;
        Ok(ReadPage {
            wire,
            _lease: lease,
        })
    }
    /// Decode one already authenticated canonical page of a bounded batch. The
    /// caller has already paid for and holds the batch's retained bytes, so this
    /// charges exactly the same decode lease the point route charges.
    fn decode_batched(
        &mut self,
        root: bool,
        canonical: &[u8],
    ) -> CoreResult<ReadPage<F::Key, F::Value>> {
        self.counters.nodes_read += 1;
        if canonical.len() > MAX_TREE_PAGE_BYTES {
            return Err(CoreError::ObjectLimitExceeded);
        }
        let lease = self.budget.reserve(
            F::decode_scratch(canonical.len()) + std::mem::size_of::<Wire<F::Key, F::Value>>(),
        )?;
        let wire = F::decode(canonical)?;
        Self::check_page(root, &wire)?;
        Ok(ReadPage {
            wire,
            _lease: lease,
        })
    }
    /// Retain the allocation lease with the fetched children through recursive
    /// edits. Narrow before reading when canonical buffers plus one decode do
    /// not fit; None asks the caller to use the ordinary point decoder directly.
    fn batch_children(
        &mut self,
        entries: &[(F::Key, ObjectId, F::Value)],
        width: usize,
    ) -> CoreResult<BatchChildren> {
        debug_assert!(width > 0 && width <= entries.len());
        let associations =
            std::mem::size_of::<ObjectId>() + std::mem::size_of::<(ObjectId, Vec<u8>)>();
        let decode =
            F::decode_scratch(MAX_TREE_PAGE_BYTES) + std::mem::size_of::<Wire<F::Key, F::Value>>();
        for chunk in (1..=width).rev() {
            let reserved = chunk
                .checked_mul(MAX_TREE_PAGE_BYTES + associations)
                .and_then(|bytes| bytes.checked_add(decode))
                .ok_or(CoreError::LengthOverflow)?;
            let Ok(mut lease) = self.budget.reserve(reserved) else {
                continue;
            };
            let ids = entries[..chunk]
                .iter()
                .map(|(_, id, _)| *id)
                .collect::<Vec<_>>();
            let mut fetched = Vec::with_capacity(chunk);
            self.store
                .get_authenticated_canonical_batch(&ids, |id, canonical| {
                    if canonical.len() > MAX_TREE_PAGE_BYTES || fetched.len() == chunk {
                        return Err(CoreError::ObjectLimitExceeded);
                    }
                    fetched.push((id, canonical.to_vec()));
                    Ok(())
                })?;
            drop(ids);
            let actual = fetched.capacity() * std::mem::size_of::<(ObjectId, Vec<u8>)>()
                + fetched
                    .iter()
                    .map(|(_, bytes)| bytes.capacity())
                    .sum::<usize>();
            if actual > reserved {
                lease.grow(actual - reserved)?;
            } else {
                lease.shrink(reserved - actual);
            }
            return Ok((chunk, fetched, Some(lease)));
        }
        // Do not clone a point read into an uncharged canonical buffer. The
        // caller invokes read(), whose decode lease already covers that path.
        Ok((1, Vec::new(), None))
    }
    /// The canonical-form checks every read page must pass, on both routes.
    fn check_page(root: bool, wire: &Wire<F::Key, F::Value>) -> CoreResult<()> {
        if (!root && !F::filled(wire.size, wire.entries.len(), wire.level))
            || (wire.level > 0 && wire.entries.len() < 2)
            || (!F::empty_allowed() && wire.entries.is_empty())
        {
            return Err(CoreError::NonCanonicalPagePartition);
        }
        Ok(())
    }
    fn node(&self, mut page: Page<F::Key, F::Value>) -> CoreResult<Node<F::Key, F::Value>> {
        page._lease
            .grow(page.entries.last().map_or(0, |e| F::heap_bytes(&e.key)))?;
        let count = page.entries.iter().try_fold(0u64, |n, e| {
            n.checked_add(e.count).ok_or(CoreError::LengthOverflow)
        })?;
        let bytes = page.entries.iter().try_fold(0u64, |n, e| {
            n.checked_add(e.bytes).ok_or(CoreError::LengthOverflow)
        })?;
        let size = 44
            + page
                .entries
                .iter()
                .map(|e| F::width(&e.key, page.level))
                .sum::<usize>();
        Ok(Node {
            max: page.entries.last().map(|e| e.key.clone()),
            id: page.origin,
            level: page.level,
            count,
            bytes,
            size,
            items: page.entries.len(),
            pending: Some(Box::new(page)),
        })
    }
    fn filled(node: &Node<F::Key, F::Value>) -> bool {
        F::filled(node.size, node.items, node.level)
    }
    fn persist(&mut self, mut node: Node<F::Key, F::Value>) -> CoreResult<Node<F::Key, F::Value>> {
        if let Some(mut page) = node.pending.take() {
            for entry in &mut page.entries {
                self.persist_entry(entry, page.level)?;
            }
            let _codec = self.budget.reserve(
                node.size * 2
                    + page.entries.len() * std::mem::size_of::<(F::Key, ObjectId, F::Value)>()
                    + page
                        .entries
                        .iter()
                        .map(|e| F::heap_bytes(&e.key))
                        .sum::<usize>(),
            )?;
            let canonical = F::encode(&page)?;
            if node
                .id
                .is_some_and(|id| ObjectId::for_bytes(&canonical) == id)
            {
                self.counters.nodes_reused += 1;
            } else {
                node.id = Some(self.store.put_tree_origin(canonical, page.origin)?);
                self.counters.nodes_created += 1;
            }
        } else {
            self.counters.nodes_reused += 1;
        }
        Ok(node)
    }
    fn entry(&mut self, node: Node<F::Key, F::Value>) -> CoreResult<Entry<F::Key, F::Value>> {
        Ok(Entry {
            key: node.max.ok_or(CoreError::NonCanonicalPagePartition)?,
            id: node.id.unwrap_or_else(|| ObjectId::from_digest([0; 32])),
            value: F::Value::default(),
            count: node.count,
            bytes: node.bytes,
            size: node.size,
            items: node.items,
            pending: node.pending,
        })
    }
    fn persist_entry(&mut self, entry: &mut Entry<F::Key, F::Value>, level: u8) -> CoreResult<()> {
        if let Some(page) = entry.pending.take() {
            if level == 0 {
                return Err(CoreError::WrongLogicalRole);
            }
            let node = self.node(*page)?;
            if !Self::filled(&node) {
                return Err(CoreError::NonCanonicalPagePartition);
            }
            entry.id = self.persist(node)?.id.ok_or(CoreError::IdentityMismatch)?;
        }
        Ok(())
    }
    fn child(entry: Entry<F::Key, F::Value>, level: u8) -> Node<F::Key, F::Value> {
        let items = entry
            .pending
            .as_ref()
            .map_or_else(|| entry.items, |p| p.entries.len());
        Node {
            max: Some(entry.key),
            id: Some(entry.id),
            level,
            count: entry.count,
            bytes: entry.bytes,
            size: entry.size,
            items,
            pending: entry.pending,
        }
    }
    fn materialize(&mut self, node: Node<F::Key, F::Value>) -> CoreResult<Page<F::Key, F::Value>> {
        if let Some(page) = node.pending {
            return Ok(*page);
        }
        let id = node.id.ok_or(CoreError::IdentityMismatch)?;
        let read = self.read(id, true)?;
        self.page_from_wire(id, read)
    }
    fn page_from_wire(
        &mut self,
        id: ObjectId,
        read: ReadPage<F::Key, F::Value>,
    ) -> CoreResult<Page<F::Key, F::Value>> {
        let mut page = self.page(read.wire.level)?;
        page.origin = Some(id);
        for (key, id, value) in read.wire.entries {
            if page.level == 0 {
                self.append_entry(
                    &mut page,
                    Entry {
                        bytes: F::width(&key, 0) as u64,
                        key,
                        id,
                        value,
                        count: 1,
                        size: 0,
                        items: 0,
                        pending: None,
                    },
                )?;
            } else {
                let child = self.read(id, false)?;
                self.check_child(page.level, &key, &child.wire)?;
                self.append_entry(
                    &mut page,
                    Entry {
                        key,
                        id,
                        value,
                        count: child.wire.count,
                        bytes: child.wire.bytes,
                        size: child.wire.size,
                        items: child.wire.entries.len(),
                        pending: None,
                    },
                )?;
            }
        }
        let count = page.entries.iter().try_fold(0u64, |n, e| {
            n.checked_add(e.count).ok_or(CoreError::LengthOverflow)
        })?;
        let bytes = page.entries.iter().try_fold(0u64, |n, e| {
            n.checked_add(e.bytes).ok_or(CoreError::LengthOverflow)
        })?;
        if count != read.wire.count || bytes != read.wire.bytes {
            return Err(CoreError::InvalidRecord("batched tree subtree summary"));
        }
        Ok(page)
    }
    fn check_child(
        &self,
        level: u8,
        max: &F::Key,
        child: &Wire<F::Key, F::Value>,
    ) -> CoreResult<()> {
        if child.level.checked_add(1) != Some(level)
            || child.entries.last().map(|e| &e.0) != Some(max)
        {
            return Err(CoreError::InvalidRecord("batched tree child summary"));
        }
        Ok(())
    }
    fn append_entry(
        &self,
        page: &mut Page<F::Key, F::Value>,
        entry: Entry<F::Key, F::Value>,
    ) -> CoreResult<()> {
        if page.entries.len() == page.entries.capacity() {
            let before = page.entries.capacity();
            let capacity = (before.max(1) * 2).min(F::page_items(page.level));
            let size = std::mem::size_of::<Entry<F::Key, F::Value>>();
            page._lease.grow(capacity * size)?;
            if page.entries.try_reserve_exact(capacity - before).is_err() {
                page._lease.shrink(capacity * size);
                return Err(CoreError::ObjectLimitExceeded);
            }
            page._lease.shrink(before * size);
        }
        page._lease.grow(F::heap_bytes(&entry.key))?;
        page.entries.push(entry);
        Ok(())
    }
    fn push(
        &mut self,
        page: &mut Page<F::Key, F::Value>,
        entry: Entry<F::Key, F::Value>,
    ) -> CoreResult<Option<Node<F::Key, F::Value>>> {
        if page.entries.len() == F::page_items(page.level) {
            return Err(CoreError::ObjectLimitExceeded);
        }
        // Keep the outer boundary children private. A neighbouring parent's
        // singleton underfull chain can still redistribute their entries.
        // Interior children cannot meet that chain after this final-state merge.
        if page.level > 0 && page.entries.len() > 1 {
            self.persist_entry(page.entries.last_mut().unwrap(), page.level)?;
        }
        self.append_entry(page, entry)?;
        let size = 44
            + page
                .entries
                .iter()
                .map(|e| F::width(&e.key, page.level))
                .sum::<usize>();
        if F::fits(size, page.entries.len(), page.level) {
            return Ok(None);
        }
        let mut right = self.page(page.level)?;
        let split = super::directory::nearest_half(
            page.entries
                .iter()
                .map(|e| F::width(&e.key, page.level))
                .collect(),
        );
        for entry in page.entries.drain(split..) {
            self.append_entry(&mut right, entry)?;
        }
        page.origin = None;
        let left = std::mem::replace(page, right);
        self.node(left).map(Some)
    }
    #[allow(clippy::type_complexity)]
    fn merge(
        &mut self,
        left: Node<F::Key, F::Value>,
        right: Node<F::Key, F::Value>,
        output: &mut dyn FnMut(&mut Self, Node<F::Key, F::Value>) -> CoreResult<()>,
    ) -> CoreResult<()> {
        let mut left = self.materialize(left)?;
        let right = self.materialize(right)?;
        if left.level != right.level {
            return Err(CoreError::WrongLogicalRole);
        }
        left.origin = None;
        if left.level == 0 {
            for entry in right.entries {
                if let Some(node) = self.push(&mut left, entry)? {
                    output(self, node)?;
                }
            }
        } else {
            let mut page = self.page(left.level)?;
            let mut pending = None;
            let level = left.level - 1;
            for entry in left.entries.into_iter().chain(right.entries) {
                self.sibling(
                    &mut pending,
                    Self::child(entry, level),
                    &mut |engine, child| {
                        let entry = engine.entry(child)?;
                        if let Some(node) = engine.push(&mut page, entry)? {
                            output(engine, node)?;
                        }
                        Ok(())
                    },
                )?;
            }
            if let Some(child) = pending {
                let entry = self.entry(child)?;
                if let Some(node) = self.push(&mut page, entry)? {
                    output(self, node)?;
                }
            }
            left = page;
        }
        {
            let node = self.node(left)?;
            output(self, node)
        }
    }
    #[allow(clippy::type_complexity)]
    fn sibling(
        &mut self,
        pending: &mut Option<Node<F::Key, F::Value>>,
        next: Node<F::Key, F::Value>,
        output: &mut dyn FnMut(&mut Self, Node<F::Key, F::Value>) -> CoreResult<()>,
    ) -> CoreResult<()> {
        let Some(previous) = pending.take() else {
            *pending = Some(next);
            return Ok(());
        };
        if Self::filled(&previous) && Self::filled(&next) {
            output(self, previous)?;
            *pending = Some(next);
        } else {
            self.merge(previous, next, &mut |engine, node| {
                if let Some(previous) = pending.replace(node) {
                    output(engine, previous)?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }
    #[allow(clippy::type_complexity)]
    fn edit<I: Iterator<Item = CoreResult<(F::Key, Option<(ObjectId, F::Value)>)>>>(
        &mut self,
        id: Option<ObjectId>,
        read: ReadPage<F::Key, F::Value>,
        bound: Option<&F::Key>,
        deltas: &mut Deltas<I, F::Key, F::Value>,
        output: &mut dyn FnMut(&mut Self, Node<F::Key, F::Value>) -> CoreResult<()>,
    ) -> CoreResult<()> {
        if !deltas.in_range(bound) {
            return output(
                self,
                Node::existing(id.ok_or(CoreError::MissingObject)?, &read.wire),
            );
        }
        let mut page = self.page(read.wire.level)?;
        page.origin = id;
        if page.level == 0 {
            let mut old = read
                .wire
                .entries
                .into_iter()
                .map(|(key, id, value)| (key, (id, value)))
                .peekable();
            while old.peek().is_some() || deltas.in_range(bound) {
                let delta_first = deltas.in_range(bound)
                    && old
                        .peek()
                        .is_none_or(|old| old.0 >= deltas.next.as_ref().unwrap().0);
                let (key, value) = if delta_first {
                    let (key, value) = deltas.take()?;
                    self.counters.delta_keys += 1;
                    let previous = if old.peek().is_some_and(|old| old.0 == key) {
                        old.next().map(|(_, value)| value)
                    } else {
                        None
                    };
                    (self.changes)(
                        previous.map(|value| value.0),
                        value.as_ref().map(|value| value.0),
                    )?;
                    (key, value)
                } else {
                    let (key, value) = old.next().unwrap();
                    (key, Some(value))
                };
                if let Some((id, value)) = value {
                    let value = F::leaf_value(self.store, id, value)?;
                    let entry = Entry {
                        bytes: F::width(&key, 0) as u64,
                        key,
                        id,
                        value,
                        count: 1,
                        size: 0,
                        items: 0,
                        pending: None,
                    };
                    if let Some(node) = self.push(&mut page, entry)? {
                        output(self, node)?;
                    }
                }
            }
        } else {
            let level = page.level;
            let entries = read.wire.entries;
            let count = entries.len();
            let mut old_count = 0u64;
            let mut old_bytes = 0u64;
            let mut pending = None;
            let mut start = 0;
            while start < count {
                // One bounded authenticated batch serves this chunk: the store
                // resolves every location in one demand wave and selects and
                // decompresses each physical group once for all of its demands.
                // The chunk is narrowed to the remaining ledger when the full
                // chunk does not fit, so a reduced budget degrades the batch
                // instead of failing where a point read would have succeeded.
                // Every page is still read, authenticated and decoded, and every
                // check below still runs per child in ascending key order.
                let width = TREE_BATCH_CHILDREN.min(count - start);
                let (chunk, mut fetched, mut retained) =
                    self.batch_children(&entries[start..], width)?;
                for (index, (key, child_id, _)) in entries[start..start + chunk].iter().enumerate()
                {
                    let child = if let Some(retained) = &mut retained {
                        let position = fetched
                            .iter()
                            .position(|(id, _)| id == child_id)
                            .ok_or(CoreError::MissingObject)?;
                        let (_, canonical) = fetched.remove(position);
                        let child = self.decode_batched(false, &canonical)?;
                        retained.shrink(canonical.capacity());
                        drop(canonical);
                        child
                    } else {
                        self.read(*child_id, false)?
                    };
                    self.check_child(level, key, &child.wire)?;
                    old_count = old_count
                        .checked_add(child.wire.count)
                        .ok_or(CoreError::LengthOverflow)?;
                    old_bytes = old_bytes
                        .checked_add(child.wire.bytes)
                        .ok_or(CoreError::LengthOverflow)?;
                    let child_bound = if start + index + 1 == count {
                        bound
                    } else {
                        Some(key)
                    };
                    self.edit(
                        Some(*child_id),
                        child,
                        child_bound,
                        deltas,
                        &mut |engine, node| {
                            engine.sibling(&mut pending, node, &mut |engine, node| {
                                let entry = engine.entry(node)?;
                                if let Some(node) = engine.push(&mut page, entry)? {
                                    output(engine, node)?;
                                }
                                Ok(())
                            })
                        },
                    )?;
                }
                start += chunk;
            }
            if old_count != read.wire.count || old_bytes != read.wire.bytes {
                return Err(CoreError::InvalidRecord("batched tree subtree summary"));
            }
            if let Some(node) = pending {
                let entry = self.entry(node)?;
                if let Some(node) = self.push(&mut page, entry)? {
                    output(self, node)?;
                }
            }
        }
        if !page.entries.is_empty() {
            let node = self.node(page)?;
            output(self, node)?;
        }
        Ok(())
    }
}

struct Deltas<I, K, V> {
    source: I,
    next: Option<(K, Option<(ObjectId, V)>)>,
}
impl<I: Iterator<Item = CoreResult<(K, Option<(ObjectId, V)>)>>, K: Ord, V> Deltas<I, K, V> {
    fn new(mut source: I) -> CoreResult<Self> {
        let next = source.next().transpose()?;
        Ok(Self { source, next })
    }
    fn in_range(&self, bound: Option<&K>) -> bool {
        self.next
            .as_ref()
            .is_some_and(|v| bound.is_none_or(|bound| &v.0 <= bound))
    }
    fn take(&mut self) -> CoreResult<(K, Option<(ObjectId, V)>)> {
        let current = self.next.take().ok_or(CoreError::UnexpectedEof)?;
        self.next = self.source.next().transpose()?;
        if self.next.as_ref().is_some_and(|next| next.0 <= current.0) {
            return Err(CoreError::NonCanonicalOrdering);
        }
        Ok(current)
    }
}

#[cfg(test)]
fn apply<S: ObjectStore, F: Format>(
    store: &mut S,
    root: ObjectId,
    source: impl Iterator<Item = CoreResult<(F::Key, Option<ObjectId>)>>,
) -> CoreResult<(ObjectId, TreeBatchCounters)> {
    apply_budgeted::<S, F>(
        store,
        root,
        source.map(|row| row.map(|(key, id)| (key, id.map(|id| (id, F::Value::default()))))),
        SORTED_TREE_UPDATE_SCRATCH_BYTES,
        None,
        &mut |_, _| Ok(()),
    )
}
fn apply_budgeted<S: ObjectStore, F: Format>(
    store: &mut S,
    root: ObjectId,
    source: impl Iterator<Item = CoreResult<(F::Key, Option<(ObjectId, F::Value)>)>>,
    scratch_limit: usize,
    expected: Option<(u8, u64)>,
    changes: &mut dyn FnMut(Option<ObjectId>, Option<ObjectId>) -> CoreResult<()>,
) -> CoreResult<(ObjectId, TreeBatchCounters)> {
    apply_budgeted_root::<S, F>(store, Some(root), source, scratch_limit, expected, changes)
}
fn apply_budgeted_root<S: ObjectStore, F: Format>(
    store: &mut S,
    root: Option<ObjectId>,
    source: impl Iterator<Item = CoreResult<(F::Key, Option<(ObjectId, F::Value)>)>>,
    scratch_limit: usize,
    expected: Option<(u8, u64)>,
    changes: &mut dyn FnMut(Option<ObjectId>, Option<ObjectId>) -> CoreResult<()>,
) -> CoreResult<(ObjectId, TreeBatchCounters)> {
    let mut deltas = Deltas::new(source)?;
    if deltas.next.is_none() {
        return Ok((
            root.ok_or(CoreError::InvalidRecord("empty initial tree"))?,
            TreeBatchCounters::default(),
        ));
    }
    let mut engine = Engine::<S, F> {
        store,
        changes,
        budget: Rc::new(Budget {
            limit: scratch_limit,
            ..Budget::default()
        }),
        counters: TreeBatchCounters::default(),
        format: PhantomData,
    };
    let read = match root {
        Some(root) => engine.read(root, true)?,
        None => ReadPage {
            wire: Wire {
                level: 0,
                count: 0,
                bytes: 0,
                entries: Vec::new(),
                size: 44,
            },
            _lease: engine
                .budget
                .reserve(std::mem::size_of::<Wire<F::Key, F::Value>>())?,
        },
    };
    if expected.is_some_and(|value| value != (read.wire.level, read.wire.count)) {
        return Err(CoreError::InvalidRecord("batched tree root summary"));
    }
    let level = read.wire.level;
    let mut first = None;
    let _frontier = engine
        .budget
        .reserve(32 * std::mem::size_of::<PageSlot<F::Key, F::Value>>())?;
    let mut levels: Vec<PageSlot<F::Key, F::Value>> = (0..32).map(|_| None).collect();
    engine.edit(root, read, None, &mut deltas, &mut |engine, node| {
        if first.is_none() && levels.iter().all(Option::is_none) {
            first = Some(node);
            return Ok(());
        }
        if let Some(prior) = first.take() {
            append_root(engine, &mut levels, prior)?;
        }
        append_root(engine, &mut levels, node)
    })?;
    let mut root_node = if let Some(first) = first {
        first
    } else {
        let mut last = None;
        for index in usize::from(level) + 1..32 {
            if let Some(page) = levels[index].take() {
                let node = engine.node(*page)?;
                if levels[index + 1..].iter().all(Option::is_none) {
                    last = Some(node);
                    break;
                }
                append_root(&mut engine, &mut levels, node)?;
            }
        }
        match last {
            Some(node) => node,
            None if F::empty_allowed() => {
                let mut page = engine.page(0)?;
                page.origin = root;
                engine.node(page)?
            }
            None => return Err(CoreError::InvalidRecord("empty inode table")),
        }
    };
    while root_node.level > 0 && root_node.items == 1 {
        let page = engine.materialize(root_node)?;
        root_node = Engine::<S, F>::child(page.entries.into_iter().next().unwrap(), page.level - 1);
    }
    let root = engine
        .persist(root_node)?
        .id
        .ok_or(CoreError::IdentityMismatch)?;
    engine.counters.peak_scratch_bytes = engine.budget.peak.get();
    Ok((root, engine.counters))
}
fn append_root<S: ObjectStore, F: Format>(
    engine: &mut Engine<'_, S, F>,
    levels: &mut [PageSlot<F::Key, F::Value>],
    mut node: Node<F::Key, F::Value>,
) -> CoreResult<()> {
    loop {
        let level = node
            .level
            .checked_add(1)
            .filter(|level| *level <= 31)
            .ok_or(CoreError::MappingDepthExceeded)?;
        let entry = engine.entry(node)?;
        let slot = &mut levels[usize::from(level)];
        if slot.is_none() {
            *slot = Some(Box::new(engine.page(level)?));
        }
        match engine.push(slot.as_mut().unwrap(), entry)? {
            Some(next) => node = next,
            None => return Ok(()),
        }
    }
}

struct Directory;
impl Format for Directory {
    type Key = CanonicalName;
    type Value = ();
    fn decode(bytes: &[u8]) -> CoreResult<Wire<Self::Key, Self::Value>> {
        let (level, count, logical, entries) = match decode_directory_node(bytes)? {
            DirectoryNodeV1::Leaf {
                compact: _,
                subtree_encoded_bytes,
                entries,
            } => (
                0,
                entries.len() as u64,
                subtree_encoded_bytes,
                entries
                    .into_iter()
                    .map(|(k, v)| (k, ObjectId::from_digest(v.0)))
                    .collect(),
            ),
            DirectoryNodeV1::Branch {
                compact: _,
                level,
                subtree_entry_count,
                subtree_encoded_bytes,
                children,
            } => (level, subtree_entry_count, subtree_encoded_bytes, children),
        };
        Ok(Wire {
            level,
            count,
            bytes: logical,
            entries: entries.into_iter().map(|(key, id)| (key, id, ())).collect(),
            size: bytes.len(),
        })
    }
    fn encode(page: &Page<Self::Key, Self::Value>) -> CoreResult<Vec<u8>> {
        let bytes = page.entries.iter().try_fold(0u64, |n, e| {
            n.checked_add(e.bytes).ok_or(CoreError::LengthOverflow)
        })?;
        let node = if page.level == 0 {
            DirectoryNodeV1::Leaf {
                compact: false,
                subtree_encoded_bytes: bytes,
                entries: page
                    .entries
                    .iter()
                    .map(|e| (e.key.clone(), InodeId(e.id.to_bytes())))
                    .collect(),
            }
        } else {
            DirectoryNodeV1::Branch {
                compact: false,
                level: page.level,
                subtree_entry_count: page.entries.iter().try_fold(0u64, |n, e| {
                    n.checked_add(e.count).ok_or(CoreError::LengthOverflow)
                })?,
                subtree_encoded_bytes: bytes,
                children: page.entries.iter().map(|e| (e.key.clone(), e.id)).collect(),
            }
        };
        encode_directory_node(&node)
    }
    fn width(key: &Self::Key, _: u8) -> usize {
        34 + key.as_bytes().len()
    }
    fn heap_bytes(key: &Self::Key) -> usize {
        key.owned_capacity_bytes()
    }
    fn decode_scratch(bytes: usize) -> usize {
        bytes * 3 + bytes.saturating_sub(44) / 35 * std::mem::size_of::<(CanonicalName, ObjectId)>()
    }
    fn filled(size: usize, _: usize, _: u8) -> bool {
        size * 5 >= 8192 * 2
    }
    fn fits(size: usize, _: usize, _: u8) -> bool {
        size <= 8192
    }
    fn empty_allowed() -> bool {
        true
    }
}
struct Inodes;
impl Format for Inodes {
    type Key = InodeId;
    type Value = ();
    fn decode(bytes: &[u8]) -> CoreResult<Wire<Self::Key, Self::Value>> {
        let (level, count, entries) = match decode_inode_table_node(bytes)? {
            InodeTableNodeV1::Leaf(entries) => (0, entries.len() as u64, entries),
            InodeTableNodeV1::Branch {
                level,
                subtree_entry_count,
                children,
            } => (level, subtree_entry_count, children),
        };
        Ok(Wire {
            level,
            count,
            bytes: count.checked_mul(64).ok_or(CoreError::LengthOverflow)?,
            entries: entries.into_iter().map(|(key, id)| (key, id, ())).collect(),
            size: bytes.len(),
        })
    }
    fn encode(page: &Page<Self::Key, Self::Value>) -> CoreResult<Vec<u8>> {
        let entries = page.entries.iter().map(|e| (e.key, e.id)).collect();
        let node = if page.level == 0 {
            InodeTableNodeV1::Leaf(entries)
        } else {
            InodeTableNodeV1::Branch {
                level: page.level,
                subtree_entry_count: page.entries.iter().try_fold(0u64, |n, e| {
                    n.checked_add(e.count).ok_or(CoreError::LengthOverflow)
                })?,
                children: entries,
            }
        };
        encode_inode_table_node(&node)
    }
    fn width(_: &Self::Key, _: u8) -> usize {
        64
    }
    fn heap_bytes(_: &Self::Key) -> usize {
        0
    }
    fn decode_scratch(bytes: usize) -> usize {
        bytes.saturating_sub(44)
    }
    fn filled(_: usize, count: usize, _: u8) -> bool {
        count >= 64
    }
    fn fits(_: usize, count: usize, _: u8) -> bool {
        count <= 127
    }
    fn empty_allowed() -> bool {
        false
    }
}

struct CompactInodes;
impl Format for CompactInodes {
    type Key = super::compact::InodeSerial;
    type Value = Option<super::inode::InodeRecordV1>;
    fn leaf_value<S: crate::object::access::ObjectRead>(
        store: &S,
        id: ObjectId,
        value: Self::Value,
    ) -> CoreResult<Self::Value> {
        match value {
            Some(value) => Ok(Some(value)),
            None => store
                .with_authenticated_canonical(id, super::inode::codec::decode_inode_record)
                .map(Some),
        }
    }
    fn decode(bytes: &[u8]) -> CoreResult<Wire<Self::Key, Self::Value>> {
        use super::compact::InodeNode;
        let (level, count, entries) = match super::compact::decode_inode(bytes)? {
            InodeNode::Leaf(rows) => (
                0,
                rows.len() as u64,
                rows.into_iter()
                    .map(|(key, record)| Ok((key, inode_value_id(record)?, Some(record))))
                    .collect::<CoreResult<Vec<_>>>()?,
            ),
            InodeNode::Branch {
                level,
                subtree_count,
                children,
            } => (
                level,
                subtree_count,
                children
                    .into_iter()
                    .map(|(key, id)| (key, id, None))
                    .collect(),
            ),
        };
        Ok(Wire {
            level,
            count,
            bytes: count.checked_mul(81).ok_or(CoreError::LengthOverflow)?,
            entries,
            size: bytes.len(),
        })
    }
    fn encode(page: &Page<Self::Key, Self::Value>) -> CoreResult<Vec<u8>> {
        use super::compact::InodeNode;
        let node = if page.level == 0 {
            InodeNode::Leaf(
                page.entries
                    .iter()
                    .map(|entry| Ok((entry.key, entry.value.ok_or(CoreError::WrongLogicalRole)?)))
                    .collect::<CoreResult<_>>()?,
            )
        } else {
            InodeNode::Branch {
                level: page.level,
                subtree_count: page.entries.iter().try_fold(0u64, |sum, entry| {
                    sum.checked_add(entry.count)
                        .ok_or(CoreError::LengthOverflow)
                })?,
                children: page
                    .entries
                    .iter()
                    .map(|entry| (entry.key, entry.id))
                    .collect(),
            }
        };
        super::compact::encode_inode(&node)
    }
    fn width(_: &Self::Key, level: u8) -> usize {
        if level == 0 {
            81
        } else {
            40
        }
    }
    fn heap_bytes(_: &Self::Key) -> usize {
        0
    }
    fn decode_scratch(bytes: usize) -> usize {
        bytes * 4 + 8192
    }
    fn filled(_: usize, count: usize, level: u8) -> bool {
        count >= if level == 0 { 50 } else { 64 }
    }
    fn fits(_: usize, count: usize, level: u8) -> bool {
        count <= if level == 0 { 100 } else { 127 }
    }
    fn empty_allowed() -> bool {
        false
    }
}

// This comparison identity is not a separately admitted inode object. The
// canonical leaf authenticates the actual inline value and its references.
fn inode_value_id(record: super::inode::InodeRecordV1) -> CoreResult<ObjectId> {
    Ok(ObjectId::for_bytes(
        &super::inode::codec::encode_inode_record(record)?,
    ))
}

pub fn compact_inode_table_apply_sorted<S: ObjectStore>(
    store: &mut S,
    root: super::inode::InodeTableRoot,
    deltas: impl Iterator<
        Item = CoreResult<(
            super::compact::InodeSerial,
            Option<super::inode::InodeRecordV1>,
        )>,
    >,
    scratch_limit: usize,
) -> CoreResult<(super::inode::InodeTableRoot, TreeBatchCounters)> {
    let deltas = deltas.map(|row| {
        let (key, record) = row?;
        Ok((
            key,
            record
                .map(|record| Ok((inode_value_id(record)?, Some(record))))
                .transpose()?,
        ))
    });
    let (root, counters) = apply_budgeted::<S, CompactInodes>(
        store,
        root.0,
        deltas,
        scratch_limit,
        None,
        &mut |_, _| Ok(()),
    )?;
    Ok((super::inode::InodeTableRoot(root), counters))
}

/// Construct a compact table from strictly ordered values without an admitted
/// seed leaf or intermediate record objects. The same bounded engine seals pages.
pub fn compact_inode_table_from_sorted<S: ObjectStore>(
    store: &mut S,
    rows: impl Iterator<Item = CoreResult<(super::compact::InodeSerial, super::inode::InodeRecordV1)>>,
    scratch_limit: usize,
) -> CoreResult<(super::inode::InodeTableRoot, TreeBatchCounters)> {
    let rows = rows.map(|row| {
        let (key, record) = row?;
        Ok((key, Some((inode_value_id(record)?, Some(record)))))
    });
    let (root, counters) = apply_budgeted_root::<S, CompactInodes>(
        store,
        None,
        rows,
        scratch_limit,
        None,
        &mut |_, _| Ok(()),
    )?;
    Ok((super::inode::InodeTableRoot(root), counters))
}

struct CompactDirectory;
impl Format for CompactDirectory {
    type Key = CanonicalName;
    type Value = ();
    fn decode(bytes: &[u8]) -> CoreResult<Wire<Self::Key, Self::Value>> {
        use super::compact::DirectoryNode;
        let (level, count, logical, entries) = match super::compact::decode_directory(bytes)? {
            DirectoryNode::Leaf(rows) => {
                let logical = rows.iter().try_fold(0u64, |sum, row| {
                    sum.checked_add(10 + row.0.as_bytes().len() as u64)
                        .ok_or(CoreError::LengthOverflow)
                })?;
                (
                    0,
                    rows.len() as u64,
                    logical,
                    rows.into_iter()
                        .map(|(name, serial)| {
                            (name, ObjectId::from_digest(serial.inode_key().0), ())
                        })
                        .collect(),
                )
            }
            DirectoryNode::Branch {
                level,
                subtree_count,
                subtree_bytes,
                children,
            } => (
                level,
                subtree_count,
                subtree_bytes,
                children
                    .into_iter()
                    .map(|(name, id)| (name, id, ()))
                    .collect(),
            ),
        };
        Ok(Wire {
            level,
            count,
            bytes: logical,
            entries,
            size: bytes.len(),
        })
    }
    fn encode(page: &Page<Self::Key, Self::Value>) -> CoreResult<Vec<u8>> {
        use super::compact::{DirectoryNode, InodeSerial};
        let node = if page.level == 0 {
            DirectoryNode::Leaf(
                page.entries
                    .iter()
                    .map(|entry| {
                        Ok((
                            entry.key.clone(),
                            InodeSerial::from_inode_key(InodeId(entry.id.to_bytes()))?,
                        ))
                    })
                    .collect::<CoreResult<_>>()?,
            )
        } else {
            DirectoryNode::Branch {
                level: page.level,
                subtree_count: page.entries.iter().try_fold(0u64, |sum, entry| {
                    sum.checked_add(entry.count)
                        .ok_or(CoreError::LengthOverflow)
                })?,
                subtree_bytes: page.entries.iter().try_fold(0u64, |sum, entry| {
                    sum.checked_add(entry.bytes)
                        .ok_or(CoreError::LengthOverflow)
                })?,
                children: page
                    .entries
                    .iter()
                    .map(|entry| (entry.key.clone(), entry.id))
                    .collect(),
            }
        };
        super::compact::encode_directory(&node)
    }
    fn width(key: &Self::Key, level: u8) -> usize {
        (if level == 0 { 10 } else { 34 }) + key.as_bytes().len()
    }
    fn heap_bytes(key: &Self::Key) -> usize {
        key.owned_capacity_bytes()
    }
    fn page_items(level: u8) -> usize {
        if level == 0 {
            (8192 - 44) / 11 + 1
        } else {
            PAGE_ITEMS
        }
    }
    fn decode_scratch(bytes: usize) -> usize {
        bytes * 5 + bytes.saturating_sub(44) / 11 * std::mem::size_of::<(CanonicalName, ObjectId)>()
    }
    fn filled(size: usize, _: usize, _: u8) -> bool {
        size * 5 >= 8192 * 2
    }
    fn fits(size: usize, _: usize, _: u8) -> bool {
        size <= 8192
    }
    fn empty_allowed() -> bool {
        true
    }
}

pub fn compact_directory_apply_sorted<S: ObjectStore>(
    store: &mut S,
    root: super::directory::DirectoryStateRoot,
    deltas: impl Iterator<Item = CoreResult<(CanonicalName, Option<super::compact::InodeSerial>)>>,
    scratch_limit: usize,
    mut changes: impl FnMut(Option<InodeId>, Option<InodeId>) -> CoreResult<()>,
) -> CoreResult<(super::directory::DirectoryStateRoot, TreeBatchCounters)> {
    let deltas = deltas.map(|row| {
        row.map(|(key, serial)| {
            (
                key,
                serial.map(|serial| (ObjectId::from_digest(serial.inode_key().0), ())),
            )
        })
    });
    let (root, counters) = apply_budgeted::<S, CompactDirectory>(
        store,
        root.0,
        deltas,
        scratch_limit,
        None,
        &mut |before, after| {
            changes(
                before.map(|id| InodeId(id.to_bytes())),
                after.map(|id| InodeId(id.to_bytes())),
            )
        },
    )?;
    Ok((super::directory::DirectoryStateRoot(root), counters))
}

/// Sorted unique final bindings; None ensures absence, Some upserts.
pub fn directory_apply_sorted<S: ObjectStore>(
    store: &mut S,
    root: super::directory::DirectoryStateRoot,
    deltas: impl Iterator<Item = CoreResult<(CanonicalName, Option<InodeId>)>>,
) -> CoreResult<(super::directory::DirectoryStateRoot, TreeBatchCounters)> {
    directory_apply_sorted_with_budget(store, root, deltas, SORTED_TREE_UPDATE_SCRATCH_BYTES)
}
/// Same engine with a caller-reserved simultaneous scratch ceiling.
pub fn directory_apply_sorted_with_budget<S: ObjectStore>(
    store: &mut S,
    root: super::directory::DirectoryStateRoot,
    deltas: impl Iterator<Item = CoreResult<(CanonicalName, Option<InodeId>)>>,
    scratch_limit: usize,
) -> CoreResult<(super::directory::DirectoryStateRoot, TreeBatchCounters)> {
    directory_apply_sorted_observed(store, root, deltas, scratch_limit, |_, _| Ok(()))
}

/// Reports each original/final binding from the same leaf merge. Observations
/// are provisional until success; callers must discard them if construction fails.
pub fn directory_apply_sorted_observed<S: ObjectStore>(
    store: &mut S,
    root: super::directory::DirectoryStateRoot,
    deltas: impl Iterator<Item = CoreResult<(CanonicalName, Option<InodeId>)>>,
    scratch_limit: usize,
    mut changes: impl FnMut(Option<InodeId>, Option<InodeId>) -> CoreResult<()>,
) -> CoreResult<(super::directory::DirectoryStateRoot, TreeBatchCounters)> {
    use super::directory::codec::{decode_directory_state, encode_directory_state};
    let mut state = store.with_authenticated_canonical(root.0, decode_directory_state)?;
    if state.profile_id == super::compact::profile_id() {
        let deltas = deltas.map(|row| {
            let (name, key) = row?;
            Ok((
                name,
                key.map(super::compact::InodeSerial::from_inode_key)
                    .transpose()?,
            ))
        });
        let (root, mut counters) =
            compact_directory_apply_sorted(store, root, deltas, scratch_limit, changes)?;
        counters.nodes_read += 1;
        return Ok((root, counters));
    }
    let (mapping, mut counters) = apply_budgeted::<S, Directory>(
        store,
        state.mapping_root,
        deltas.map(|v| v.map(|(k, v)| (k, v.map(|v| (ObjectId::from_digest(v.0), ()))))),
        scratch_limit,
        Some((state.tree_level, state.entry_count)),
        &mut |before, after| {
            changes(
                before.map(|id| InodeId(id.to_bytes())),
                after.map(|id| InodeId(id.to_bytes())),
            )
        },
    )?;
    counters.nodes_read += 1;
    if mapping == state.mapping_root {
        return Ok((root, counters));
    }
    let wire = store.with_authenticated_canonical(mapping, Directory::decode)?;
    counters.nodes_read += 1;
    state.mapping_root = mapping;
    state.entry_count = wire.count;
    state.tree_level = wire.level;
    Ok((
        super::directory::DirectoryStateRoot(store.put_owned(encode_directory_state(state)?)?),
        counters,
    ))
}
/// Sorted unique final record IDs, sharing exactly the directory mutation engine.
pub fn inode_table_apply_sorted<S: ObjectStore>(
    store: &mut S,
    root: super::inode::InodeTableRoot,
    deltas: impl Iterator<Item = CoreResult<(InodeId, Option<ObjectId>)>>,
) -> CoreResult<(super::inode::InodeTableRoot, TreeBatchCounters)> {
    inode_table_apply_sorted_with_budget(store, root, deltas, SORTED_TREE_UPDATE_SCRATCH_BYTES)
}
/// Same engine with a caller-reserved simultaneous scratch ceiling.
pub fn inode_table_apply_sorted_with_budget<S: ObjectStore>(
    store: &mut S,
    root: super::inode::InodeTableRoot,
    deltas: impl Iterator<Item = CoreResult<(InodeId, Option<ObjectId>)>>,
    scratch_limit: usize,
) -> CoreResult<(super::inode::InodeTableRoot, TreeBatchCounters)> {
    let mut deltas = deltas.peekable();
    if deltas.peek().is_none() {
        return Ok((root, TreeBatchCounters::default()));
    }
    if super::compact::is_inode_table(store, root.0)? {
        let deltas = deltas.map(|row| {
            let (key, id) = row?;
            Ok((
                super::compact::InodeSerial::from_inode_key(key)?,
                id.map(|id| (id, None)),
            ))
        });
        let (root, mut counters) = apply_budgeted::<S, CompactInodes>(
            store,
            root.0,
            deltas,
            scratch_limit,
            None,
            &mut |_, _| Ok(()),
        )?;
        counters.nodes_read += 1;
        return Ok((super::inode::InodeTableRoot(root), counters));
    }
    let (root, mut counters) = apply_budgeted::<S, Inodes>(
        store,
        root.0,
        deltas.map(|row| row.map(|(key, id)| (key, id.map(|id| (id, ()))))),
        scratch_limit,
        None,
        &mut |_, _| Ok(()),
    )?;
    counters.nodes_read += 1;
    Ok((super::inode::InodeTableRoot(root), counters))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::build_initial_directory;
    use crate::tree::directory::{
        directory_entries, directory_lookup, empty_directory, DirectoryStateRoot, NamespaceCounters,
    };
    use crate::tree::inode::{inode_table_entries, inode_table_upsert, InodeTableRoot};
    use std::collections::BTreeMap;

    #[test]
    fn initial_compact_table_streams_only_final_reachable_nodes() {
        use crate::tree::{
            compact::InodeSerial,
            inode::{InodeKind, InodeRecordV1},
        };
        let mut store = MemoryStore::default();
        let record = InodeRecordV1 {
            kind: InodeKind::RegularFile,
            namespace_ref_count: 1,
            content_root: value(1),
            metadata_root: value(2),
        };
        let (root, counters) = compact_inode_table_from_sorted(
            &mut store,
            (1..=13000).map(|n| Ok((InodeSerial::new(n).unwrap(), record))),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
        )
        .unwrap();
        assert_eq!(counters.nodes_read, 0);
        assert!(counters.peak_scratch_bytes <= SORTED_TREE_UPDATE_SCRATCH_BYTES);
        let mut reached = std::collections::BTreeSet::new();
        reachable::<CompactInodes>(&store, root.0, &mut reached);
        assert_eq!(reached, store.objects.keys().copied().collect());
    }

    #[test]
    fn compact_inline_inode_updates_reuse_the_bounded_tree_engine() {
        use crate::tree::compact::{self, InodeNode, InodeSerial};
        use crate::tree::inode::{
            inode_record_lookup_many, InodeKind, InodeRecordV1, InodeTableCounters,
        };
        let mut store = MemoryStore::default();
        let record = InodeRecordV1 {
            kind: InodeKind::RegularFile,
            namespace_ref_count: 1,
            content_root: ObjectId::for_bytes(b"content"),
            metadata_root: ObjectId::for_bytes(b"metadata"),
        };
        let serial = |n| InodeSerial::new(n).unwrap();
        let first = store
            .put(&compact::encode_inode(&InodeNode::Leaf(vec![(serial(1), record)])).unwrap())
            .unwrap();
        let (initial, counters) = compact_inode_table_apply_sorted(
            &mut store,
            InodeTableRoot(first),
            (2..=13000).map(|n| Ok((serial(n), Some(record)))),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
        )
        .unwrap();
        assert!(counters.peak_scratch_bytes <= SORTED_TREE_UPDATE_SCRATCH_BYTES);
        assert!(matches!(
            compact::decode_inode(&store.get(initial.0).unwrap()).unwrap(),
            InodeNode::Branch { level: 2, .. }
        ));
        let changed = InodeRecordV1 {
            namespace_ref_count: 2,
            ..record
        };
        let (updated, _) = compact_inode_table_apply_sorted(
            &mut store,
            initial,
            (1..=13000)
                .step_by(3)
                .map(|n| Ok((serial(n), if n % 2 == 0 { None } else { Some(changed) }))),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
        )
        .unwrap();
        assert_ne!(initial, updated);
        for (root, edited) in [(initial, false), (updated, true)] {
            for keys in (1..=13000)
                .map(|n| serial(n).inode_key())
                .collect::<Vec<_>>()
                .chunks(128)
            {
                let values = inode_record_lookup_many(
                    &store,
                    root,
                    keys,
                    &mut InodeTableCounters::default(),
                )
                .unwrap();
                for (key, actual) in keys.iter().zip(values) {
                    let n = InodeSerial::from_inode_key(*key).unwrap().get();
                    let expected = if edited && (n - 1) % 3 == 0 {
                        if n % 2 == 0 {
                            None
                        } else {
                            Some(changed)
                        }
                    } else {
                        Some(record)
                    };
                    assert_eq!(actual, expected);
                }
            }
        }
        assert!(store
            .objects
            .values()
            .all(|bytes| !crate::decode_bytes_object(bytes)
                .unwrap()
                .starts_with(b"LFS4INO\0")));
        assert!(compact_inode_table_apply_sorted(
            &mut store,
            updated,
            std::iter::once(Ok((serial(2), Some(changed)))),
            128
        )
        .is_err());
        assert_eq!(
            compact::inode_lookup(
                &store,
                updated.0,
                serial(2),
                &mut InodeTableCounters::default()
            )
            .unwrap(),
            Some(record)
        );
    }

    #[test]
    fn compact_directory_updates_preserve_names_serials_and_snapshots() {
        use crate::tree::compact::{self, DirectoryNode, InodeSerial};
        fn rows(
            store: &MemoryStore,
            root: ObjectId,
            output: &mut Vec<(CanonicalName, InodeSerial)>,
        ) -> (u64, u64) {
            match compact::decode_directory(&store.get(root).unwrap()).unwrap() {
                DirectoryNode::Leaf(entries) => {
                    let count = entries.len() as u64;
                    let bytes = entries
                        .iter()
                        .map(|row| 10 + row.0.as_bytes().len() as u64)
                        .sum();
                    output.extend(entries);
                    (count, bytes)
                }
                DirectoryNode::Branch {
                    subtree_count,
                    subtree_bytes,
                    children,
                    ..
                } => {
                    let mut total = (0, 0);
                    for (maximum, child) in children {
                        let (count, bytes) = rows(store, child, output);
                        assert_eq!(output.last().unwrap().0, maximum);
                        total.0 += count;
                        total.1 += bytes;
                    }
                    assert_eq!(total, (subtree_count, subtree_bytes));
                    total
                }
            }
        }
        let mut store = MemoryStore::default();
        let empty = store
            .put(&compact::encode_directory(&DirectoryNode::Leaf(Vec::new())).unwrap())
            .unwrap();
        let expected: Vec<_> = (1..=4000)
            .map(|n| {
                (
                    CanonicalName::from_bytes(format!("name-{n:05}").as_bytes()).unwrap(),
                    InodeSerial::new(n).unwrap(),
                )
            })
            .collect();
        let (initial, counters) = compact_directory_apply_sorted(
            &mut store,
            DirectoryStateRoot(empty),
            expected
                .iter()
                .cloned()
                .map(|(key, id)| Ok((key, Some(id)))),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
            |_, _| Ok(()),
        )
        .unwrap();
        assert!(counters.peak_scratch_bytes <= SORTED_TREE_UPDATE_SCRATCH_BYTES);
        let mut observed = 0;
        let (updated, _) = compact_directory_apply_sorted(
            &mut store,
            initial,
            expected
                .iter()
                .step_by(2)
                .map(|(key, _)| Ok((key.clone(), None))),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
            |before, after| {
                assert!(before.is_some());
                assert!(after.is_none());
                observed += 1;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(observed, 2000);
        for (root, expected) in [
            (initial, expected.clone()),
            (updated, expected.into_iter().skip(1).step_by(2).collect()),
        ] {
            let mut actual = Vec::new();
            rows(&store, root.0, &mut actual);
            assert_eq!(actual, expected);
        }
    }

    #[derive(Default)]
    struct MemoryStore {
        objects: BTreeMap<ObjectId, Vec<u8>>,
        puts: usize,
    }
    impl ObjectStore for MemoryStore {
        fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
            self.objects
                .get(&id)
                .cloned()
                .ok_or(CoreError::PathNotFound)
        }
        fn put(&mut self, bytes: &[u8]) -> CoreResult<ObjectId> {
            let id = ObjectId::for_bytes(bytes);
            self.puts += 1;
            self.objects.insert(id, bytes.to_vec());
            Ok(id)
        }
    }
    fn name(index: usize) -> CanonicalName {
        CanonicalName::new(&format!("entry-{index:08}")).unwrap()
    }
    fn inode(index: usize) -> InodeId {
        let mut bytes = [0; 32];
        bytes[24..].copy_from_slice(&(index as u64).to_be_bytes());
        InodeId(bytes)
    }
    fn value(index: usize) -> ObjectId {
        ObjectId::for_bytes(&(index as u64).to_le_bytes())
    }

    #[test]
    fn sorted_directory_reports_original_and_final_bindings_from_the_merge() {
        let mut store = MemoryStore::default();
        let root =
            build_initial_directory(&mut store, (0..12).map(|i| (name(i), inode(i)))).unwrap();
        let deltas = [
            (name(1), Some(inode(99))),
            (name(3), None),
            (name(5), Some(inode(5))),
            (name(13), Some(inode(13))),
            (name(14), None),
        ];
        let mut observed = Vec::new();
        let (actual, _) = directory_apply_sorted_observed(
            &mut store,
            root,
            deltas.clone().into_iter().map(Ok),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
            |before, after| {
                observed.push((before, after));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(
            observed,
            [
                (Some(inode(1)), Some(inode(99))),
                (Some(inode(3)), None),
                (Some(inode(5)), Some(inode(5))),
                (None, Some(inode(13))),
                (None, None)
            ]
        );
        let (expected, _) =
            directory_apply_sorted(&mut store, root, deltas.clone().into_iter().map(Ok)).unwrap();
        assert_eq!(actual, expected);
        let mut prefix = 0;
        let failed = directory_apply_sorted_observed(
            &mut store,
            root,
            deltas.into_iter().map(Ok),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
            |_, _| {
                prefix += 1;
                if prefix == 2 {
                    Err(CoreError::Io)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(failed, Err(CoreError::Io)));
        assert_eq!(prefix, 2);
        assert_eq!(
            directory_lookup(&store, root, &name(3), &mut NamespaceCounters::default()).unwrap(),
            Some(inode(3))
        );
    }

    #[test]
    fn sorted_builds_match_existing_canonical_split_boundaries() {
        for count in [1, 127, 128, 169, 170, 171, 254, 255] {
            let entries = (0..count)
                .map(|index| (name(index), inode(index)))
                .collect::<Vec<_>>();
            let mut existing = MemoryStore::default();
            let expected = build_initial_directory(&mut existing, entries.iter().cloned()).unwrap();
            let mut sorted = MemoryStore::default();
            let empty = empty_directory(&mut sorted).unwrap();
            let actual = directory_apply_sorted(
                &mut sorted,
                empty,
                entries
                    .into_iter()
                    .map(|entry| Ok((entry.0, Some(entry.1)))),
            )
            .unwrap()
            .0;
            assert_eq!(actual, expected, "directory count={count}");
            assert_eq!(
                sorted.objects.get(&actual.0),
                existing.objects.get(&expected.0),
                "directory canonical bytes count={count}"
            );
        }

        for count in [63, 64, 65, 126, 127, 128, 129, 254, 255] {
            let leaf = encode_inode_table_node(&InodeTableNodeV1::Leaf(vec![(inode(0), value(0))]))
                .unwrap();
            let mut existing = MemoryStore::default();
            let mut expected = InodeTableRoot(existing.put(&leaf).unwrap());
            for index in 1..count {
                expected = inode_table_upsert(&mut existing, expected, inode(index), value(index))
                    .unwrap()
                    .0;
            }
            let mut sorted = MemoryStore::default();
            let initial = InodeTableRoot(sorted.put(&leaf).unwrap());
            let actual = inode_table_apply_sorted(
                &mut sorted,
                initial,
                (1..count).map(|index| Ok((inode(index), Some(value(index))))),
            )
            .unwrap()
            .0;
            assert_eq!(actual, expected, "inode count={count}");
            assert_eq!(
                sorted.objects.get(&actual.0),
                existing.objects.get(&expected.0),
                "inode canonical bytes count={count}"
            );
        }
    }

    #[test]
    fn sorted_directory_dense_mixed_sparse_and_empty() {
        let mut store = MemoryStore::default();
        let empty = empty_directory(&mut store).unwrap();
        let (root, create) = directory_apply_sorted(
            &mut store,
            empty,
            (0..20000).map(|i| Ok((name(i), Some(inode(i))))),
        )
        .unwrap();
        assert!(create.nodes_created < 500, "{create:?}");
        assert!(create.peak_scratch_bytes < SORTED_TREE_UPDATE_SCRATCH_BYTES);
        let mut expected: BTreeMap<_, _> = (0..20000).map(|i| (name(i), inode(i))).collect();
        let changes: BTreeMap<_, _> = (0..20000)
            .filter(|i| i % 3 != 0)
            .map(|i| {
                (
                    name(i),
                    if i % 3 == 1 {
                        None
                    } else {
                        Some(inode(i + 30000))
                    },
                )
            })
            .chain((20000..24000).map(|i| (name(i), Some(inode(i)))))
            .collect();
        let (mixed, stats) = directory_apply_sorted(
            &mut store,
            root,
            changes.iter().map(|(k, v)| Ok((k.clone(), *v))),
        )
        .unwrap();
        for (key, value) in changes {
            match value {
                Some(value) => {
                    expected.insert(key, value);
                }
                None => {
                    expected.remove(&key);
                }
            }
        }
        assert_eq!(
            directory_entries(&store, mixed, &mut NamespaceCounters::default()).unwrap(),
            expected.clone().into_iter().collect::<Vec<_>>()
        );
        assert!(stats.nodes_created < 700, "{stats:?}");
        assert_eq!(
            directory_lookup(&store, root, &name(1), &mut NamespaceCounters::default()).unwrap(),
            Some(inode(1))
        );
        let (sparse, stats) = directory_apply_sorted(
            &mut store,
            mixed,
            std::iter::once(Ok((name(12000), Some(inode(90000))))),
        )
        .unwrap();
        assert!(stats.nodes_read < 1000, "{stats:?}");
        assert!(stats.nodes_created < 12, "{stats:?}");
        expected.insert(name(12000), inode(90000));
        assert_eq!(
            directory_entries(&store, sparse, &mut NamespaceCounters::default()).unwrap(),
            expected.clone().into_iter().collect::<Vec<_>>()
        );
        let before = store.puts;
        let (same, stats) = directory_apply_sorted(
            &mut store,
            sparse,
            std::iter::once(Ok((name(12000), Some(inode(90000))))),
        )
        .unwrap();
        assert_eq!(same, sparse);
        assert_eq!(stats.nodes_created, 0);
        assert_eq!(store.puts, before);
        let (cleared, _) = directory_apply_sorted(
            &mut store,
            sparse,
            expected.into_keys().map(|k| Ok((k, None))),
        )
        .unwrap();
        assert_eq!(cleared, empty);
    }

    #[test]
    fn sorted_inode_dense_delete_preserves_sparse_historical_records() {
        let mut store = MemoryStore::default();
        let root = InodeTableRoot(
            store
                .put(
                    &encode_inode_table_node(&InodeTableNodeV1::Leaf(vec![(inode(0), value(0))]))
                        .unwrap(),
                )
                .unwrap(),
        );
        let (root, stats) = inode_table_apply_sorted(
            &mut store,
            root,
            (1..20000).map(|i| Ok((inode(i), Some(value(i))))),
        )
        .unwrap();
        assert!(stats.nodes_created < 400, "{stats:?}");
        let (sparse, stats) = inode_table_apply_sorted(
            &mut store,
            root,
            std::iter::once(Ok((inode(9000), Some(value(90000))))),
        )
        .unwrap();
        assert!(stats.nodes_read < 500, "{stats:?}");
        assert!(stats.nodes_created < 10, "{stats:?}");
        let (small, stats) = inode_table_apply_sorted(
            &mut store,
            sparse,
            (1..20000)
                .filter(|i| i % 777 != 0)
                .map(|i| Ok((inode(i), None))),
        )
        .unwrap();
        let expected: Vec<_> = std::iter::once((inode(0), value(0)))
            .chain(
                (1..20000)
                    .filter(|i| i % 777 == 0)
                    .map(|i| (inode(i), value(i))),
            )
            .collect();
        assert_eq!(
            inode_table_entries(
                &store,
                small,
                &mut super::super::inode::InodeTableCounters::default()
            )
            .unwrap(),
            expected
        );
        assert!(stats.nodes_created < 20, "{stats:?}");
        assert_eq!(
            inode_table_entries(
                &store,
                root,
                &mut super::super::inode::InodeTableCounters::default()
            )
            .unwrap()
            .len(),
            20000
        );
        let before = store.puts;
        let (same, stats) = inode_table_apply_sorted(
            &mut store,
            small,
            std::iter::once(Ok((inode(0), Some(value(0))))),
        )
        .unwrap();
        assert_eq!(same, small);
        assert_eq!(stats.nodes_created, 0);
        assert_eq!(before, store.puts);
    }

    #[test]
    fn sorted_updates_validate_input_and_reserve_before_allocation() {
        let budget = Rc::<Budget>::default();
        let held = budget.reserve(SORTED_TREE_UPDATE_SCRATCH_BYTES).unwrap();
        assert!(budget.reserve(1).is_err());
        assert_eq!(budget.used.get(), SORTED_TREE_UPDATE_SCRATCH_BYTES);
        drop(held);
        assert_eq!(budget.used.get(), 0);
        let mut store = MemoryStore::default();
        let root = empty_directory(&mut store).unwrap();
        assert!(matches!(
            directory_apply_sorted(
                &mut store,
                root,
                vec![Ok((name(2), Some(inode(2)))), Ok((name(1), Some(inode(1))))].into_iter()
            ),
            Err(CoreError::NonCanonicalOrdering)
        ));
        assert!(matches!(
            directory_apply_sorted(
                &mut store,
                root,
                vec![Ok((name(1), Some(inode(1)))), Ok((name(1), Some(inode(2))))].into_iter()
            ),
            Err(CoreError::NonCanonicalOrdering)
        ));
        assert_eq!(
            directory_apply_sorted(&mut store, root, std::iter::once(Ok((name(1), None))))
                .unwrap()
                .0,
            root
        );
    }
    fn reachable<F: Format>(
        store: &MemoryStore,
        id: ObjectId,
        output: &mut std::collections::BTreeSet<ObjectId>,
    ) {
        if !output.insert(id) {
            return;
        }
        let wire = F::decode(store.objects.get(&id).unwrap()).unwrap();
        if wire.level > 0 {
            for (_, child, _) in wire.entries {
                reachable::<F>(store, child, output);
            }
        }
    }

    #[test]
    fn sorted_mixed_boundaries_emit_only_final_reachable_pages() {
        // Two-level branches plus deletes leaving singleton underfull chains
        // force redistribution across old parent boundaries, not just leaf pairs.
        let mut store = MemoryStore::default();
        let root = store
            .put(
                &encode_inode_table_node(&InodeTableNodeV1::Leaf(vec![(inode(0), value(0))]))
                    .unwrap(),
            )
            .unwrap();
        let (root, _) = apply::<_, Inodes>(
            &mut store,
            root,
            (1..20000).map(|i| Ok((inode(i), Some(value(i))))),
        )
        .unwrap();
        for seed in 1..=3 {
            let before: std::collections::BTreeSet<_> = store.objects.keys().copied().collect();
            let mut expected: BTreeMap<_, _> = (0..20000).map(|i| (inode(i), value(i))).collect();
            let mut random = seed as u64;
            let deltas: Vec<_> = (1..22000)
                .filter_map(|i| {
                    random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let final_value = if i > 20000 {
                        Some(value(i))
                    } else if i < 8000 || random % 9 < 7 {
                        None
                    } else if random % 9 == 7 {
                        Some(value(i + 30000))
                    } else {
                        return None;
                    };
                    match final_value {
                        Some(v) => {
                            expected.insert(inode(i), v);
                        }
                        None => {
                            expected.remove(&inode(i));
                        }
                    }
                    Some(Ok((inode(i), final_value)))
                })
                .collect();
            let (next, counters) =
                apply::<_, Inodes>(&mut store, root, deltas.into_iter()).unwrap();
            assert_eq!(
                inode_table_entries(
                    &store,
                    InodeTableRoot(next),
                    &mut super::super::inode::InodeTableCounters::default()
                )
                .unwrap(),
                expected.into_iter().collect::<Vec<_>>()
            );
            let mut final_pages = std::collections::BTreeSet::new();
            reachable::<Inodes>(&store, next, &mut final_pages);
            let unreachable: Vec<_> = store
                .objects
                .keys()
                .filter(|id| !before.contains(id) && !final_pages.contains(id))
                .collect();
            assert!(
                unreachable.is_empty(),
                "seed={seed}, unreachable={}, counters={counters:?}",
                unreachable.len()
            );
        }
    }

    #[test]
    fn sorted_tiny_inode_update_uses_actual_capacity() {
        let mut store = MemoryStore::default();
        let bytes = encode_inode_table_node(&InodeTableNodeV1::Leaf(vec![
            (inode(0), value(0)),
            (inode(1), value(1)),
        ]))
        .unwrap();
        let root = InodeTableRoot(store.put(&bytes).unwrap());
        let (next, counters) = inode_table_apply_sorted_with_budget(
            &mut store,
            root,
            std::iter::once(Ok((inode(1), Some(value(2))))),
            1024,
        )
        .unwrap();
        assert_ne!(next, root);
        assert!(counters.peak_scratch_bytes <= 1024, "{counters:?}");
        assert_eq!(
            inode_table_entries(
                &store,
                next,
                &mut super::super::inode::InodeTableCounters::default()
            )
            .unwrap(),
            vec![(inode(0), value(0)), (inode(1), value(2))]
        );
        assert!(matches!(
            inode_table_apply_sorted_with_budget(
                &mut store,
                root,
                std::iter::once(Ok((inode(1), Some(value(2))))),
                128
            ),
            Err(CoreError::ObjectLimitExceeded)
        ));
    }

    #[test]
    fn sorted_variable_names_balance_across_parent_boundaries() {
        let mut store = MemoryStore::default();
        let empty = store
            .put(
                &encode_directory_node(&DirectoryNodeV1::Leaf {
                    compact: false,
                    subtree_encoded_bytes: 0,
                    entries: Vec::new(),
                })
                .unwrap(),
            )
            .unwrap();
        let key =
            |i: usize| CanonicalName::new(&format!("{i:08}-{}", "x".repeat((i % 5) * 50))).unwrap();
        let (root, _) = apply::<_, Directory>(
            &mut store,
            empty,
            (0..6000).map(|i| Ok((key(i), Some(ObjectId::from_digest(inode(i).0))))),
        )
        .unwrap();
        let before: std::collections::BTreeSet<_> = store.objects.keys().copied().collect();
        let mut expected: BTreeMap<_, _> = (0..6000).map(|i| (key(i), inode(i))).collect();
        let deltas = (0..6500)
            .filter_map(|i| {
                let final_value = if i >= 6000 {
                    Some(inode(i))
                } else if i < 3000 || i % 11 < 9 {
                    None
                } else {
                    return None;
                };
                match final_value {
                    Some(value) => {
                        expected.insert(key(i), value);
                    }
                    None => {
                        expected.remove(&key(i));
                    }
                }
                Some(Ok((
                    key(i),
                    final_value.map(|v| ObjectId::from_digest(v.0)),
                )))
            })
            .collect::<Vec<_>>();
        let (next, stats) = apply::<_, Directory>(&mut store, root, deltas.into_iter()).unwrap();
        let mut final_pages = std::collections::BTreeSet::new();
        reachable::<Directory>(&store, next, &mut final_pages);
        let unreachable = store
            .objects
            .keys()
            .filter(|id| !before.contains(id) && !final_pages.contains(id))
            .count();
        assert_eq!(unreachable, 0, "{stats:?}");
        let mut actual = BTreeMap::new();
        for page in final_pages {
            if let DirectoryNodeV1::Leaf { entries, .. } =
                decode_directory_node(&store.objects[&page]).unwrap()
            {
                actual.extend(entries);
            }
        }
        assert_eq!(actual, expected);
        let (cleared, _) = apply::<_, Directory>(
            &mut store,
            next,
            expected.into_keys().map(|key| Ok((key, None))),
        )
        .unwrap();
        assert_eq!(cleared, empty);
    }

    /// Stage 2 (#111): a counting store that keeps the point route's semantics
    /// but records every bounded batch the engine asks for.
    #[derive(Default)]
    struct BatchProbe {
        batches: std::cell::RefCell<Vec<Vec<ObjectId>>>,
    }
    impl BatchProbe {
        fn calls(&self) -> usize {
            self.batches.borrow().len()
        }
        fn requested(&self) -> usize {
            self.batches.borrow().iter().map(Vec::len).sum()
        }
        fn widest(&self) -> usize {
            self.batches
                .borrow()
                .iter()
                .map(Vec::len)
                .max()
                .unwrap_or(0)
        }
        fn distinct(&self) -> std::collections::BTreeSet<ObjectId> {
            self.batches.borrow().iter().flatten().copied().collect()
        }
    }
    struct CountingStore {
        inner: MemoryStore,
        probe: std::rc::Rc<BatchProbe>,
    }
    impl ObjectStore for CountingStore {
        fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
            self.inner.get(id)
        }
        fn put(&mut self, bytes: &[u8]) -> CoreResult<ObjectId> {
            self.inner.put(bytes)
        }
        fn get_authenticated_canonical_batch<F>(
            &self,
            ids: &[ObjectId],
            mut callback: F,
        ) -> CoreResult<()>
        where
            F: FnMut(ObjectId, &[u8]) -> CoreResult<()>,
        {
            assert!(
                ids.len() <= TREE_BATCH_CHILDREN,
                "batch wider than the declared chunk"
            );
            self.probe.batches.borrow_mut().push(ids.to_vec());
            for id in ids {
                self.inner
                    .with_authenticated_canonical(*id, |canonical| callback(*id, canonical))?;
            }
            Ok(())
        }
    }

    #[test]
    fn stage2_fetched_children_keep_their_allocation_lease_until_drop() {
        let mut store = MemoryStore::default();
        let mut entries = Vec::new();
        for block in 0..2 {
            let rows = (block * 64..(block + 1) * 64)
                .map(|index| (inode(index), value(index)))
                .collect();
            let canonical = encode_inode_table_node(&InodeTableNodeV1::Leaf(rows)).unwrap();
            let id = store.put(&canonical).unwrap();
            entries.push((inode((block + 1) * 64 - 1), id, ()));
        }
        let budget = Rc::new(Budget {
            limit: 4 * MAX_TREE_PAGE_BYTES,
            ..Budget::default()
        });
        let mut changes = |_, _| Ok(());
        let mut engine = Engine::<_, Inodes> {
            store: &mut store,
            changes: &mut changes,
            budget: budget.clone(),
            counters: TreeBatchCounters::default(),
            format: PhantomData,
        };
        let fetched = engine.batch_children(&entries, entries.len()).unwrap();
        let actual = fetched.1.capacity() * std::mem::size_of::<(ObjectId, Vec<u8>)>()
            + fetched
                .1
                .iter()
                .map(|(_, bytes)| bytes.capacity())
                .sum::<usize>();
        println!(
            "retained children={} actual={actual} charged={} id_size={} tuple_size={}",
            fetched.1.len(),
            budget.used.get(),
            std::mem::size_of::<ObjectId>(),
            std::mem::size_of::<(ObjectId, Vec<u8>)>()
        );
        assert_eq!(
            budget.used.get(),
            actual,
            "live fetched children lost their allocation lease"
        );
        assert!(budget.reserve(budget.limit - actual + 1).is_err());
        drop(fetched);
        assert_eq!(budget.used.get(), 0);
    }

    fn stage2_compact_table(
        store: &mut MemoryStore,
        count: u64,
    ) -> (
        crate::tree::inode::InodeTableRoot,
        crate::tree::inode::InodeRecordV1,
    ) {
        use crate::tree::compact::{self, InodeNode, InodeSerial};
        use crate::tree::inode::{InodeKind, InodeRecordV1, InodeTableRoot};
        let record = InodeRecordV1 {
            kind: InodeKind::RegularFile,
            namespace_ref_count: 1,
            content_root: value(1),
            metadata_root: value(2),
        };
        let serial = |n| InodeSerial::new(n).unwrap();
        let first = store
            .put(&compact::encode_inode(&InodeNode::Leaf(vec![(serial(1), record)])).unwrap())
            .unwrap();
        let (root, _) = compact_inode_table_apply_sorted(
            store,
            InodeTableRoot(first),
            (2..=count).map(|n| Ok((serial(n), Some(record)))),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
        )
        .unwrap();
        (root, record)
    }

    fn stage2_mutate<S: ObjectStore>(
        store: &mut S,
        root: crate::tree::inode::InodeTableRoot,
        record: crate::tree::inode::InodeRecordV1,
        keys: impl Iterator<Item = u64>,
        scratch: usize,
    ) -> CoreResult<(crate::tree::inode::InodeTableRoot, TreeBatchCounters)> {
        use crate::tree::compact::InodeSerial;
        compact_inode_table_apply_sorted(
            store,
            root,
            keys.map(|n| Ok((InodeSerial::new(n).unwrap(), Some(record)))),
            scratch,
        )
    }

    #[test]
    fn stage2_batched_children_read_every_child_and_match_the_point_route() {
        use crate::tree::compact::InodeSerial;
        use crate::tree::inode::InodeRecordV1;
        let mut plain = MemoryStore::default();
        let (root, record) = stage2_compact_table(&mut plain, 13_000);
        let changed = InodeRecordV1 {
            namespace_ref_count: 9,
            ..record
        };
        let keys = [1_u64, 4_321, 12_999];
        let point = stage2_mutate(
            &mut plain,
            root,
            changed,
            keys.into_iter(),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
        )
        .unwrap();

        let probe = std::rc::Rc::new(BatchProbe::default());
        let mut counted = CountingStore {
            inner: MemoryStore::default(),
            probe: probe.clone(),
        };
        let (counted_root, _) = stage2_compact_table(&mut counted.inner, 13_000);
        assert_eq!(counted_root, root);
        let probe = std::rc::Rc::new(BatchProbe::default());
        counted.probe = probe.clone();
        let batched = stage2_mutate(
            &mut counted,
            root,
            changed,
            keys.into_iter(),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
        )
        .unwrap();
        // Identical output and identical page-read count on both routes.
        assert_eq!(batched.0, point.0);
        assert_eq!(batched.1.nodes_read, point.1.nodes_read);
        assert_eq!(batched.1.nodes_created, point.1.nodes_created);
        assert_eq!(batched.1.delta_keys, point.1.delta_keys);
        assert_eq!(batched.1.peak_scratch_bytes, point.1.peak_scratch_bytes);
        assert!(probe.calls() > 0, "no bounded batch was issued");
        assert!(probe.widest() <= TREE_BATCH_CHILDREN);
        // Every branch page that the point route opened is demanded through a
        // batch, and the demanded pages are distinct per node: the batch route
        // cannot skip a child or read one twice inside a node.
        assert!(probe.requested() as u64 <= batched.1.nodes_read);
        assert!(probe.distinct().len() <= probe.requested());
        for (key, expected) in keys
            .iter()
            .map(|n| (InodeSerial::new(*n).unwrap(), changed))
        {
            assert_eq!(
                crate::tree::compact::inode_lookup(
                    &counted.inner,
                    batched.0 .0,
                    key,
                    &mut crate::tree::inode::InodeTableCounters::default()
                )
                .unwrap(),
                Some(expected)
            );
        }
    }

    /// A reduced tree ledger must never make the bounded batch route fail where
    /// the point route survives: the chunk is narrowed to the ledger, and where
    /// even one child's worst-case charge does not fit, that child is read as a
    /// point read. This is the invariant the workspace's per-object fallback
    /// depends on, checked across a wide sweep of budgets and table sizes rather
    /// than at one synthetic value.
    ///
    /// Measurement note: this differential holds on the unhardened route too for
    /// every table shape measured here, because the batch route's per-chunk
    /// reservation is released to the bytes actually retained and both routes
    /// peak at the same byte for a single-key update. The hardening removes the
    /// structural failure mode rather than a demonstrated one; the control route
    /// fails only in the same cases this route does.
    #[test]
    fn stage2_reduced_scratch_never_fails_a_batch_that_a_point_read_survives() {
        use crate::tree::inode::InodeRecordV1;
        for size in [1_000_u64, 2_600, 13_000] {
            let mut plain = MemoryStore::default();
            let (root, record) = stage2_compact_table(&mut plain, size);
            let changed = InodeRecordV1 {
                namespace_ref_count: 6,
                ..record
            };
            let keys = [1_u64, 2, size / 2, size];
            let mut survivor = 0;
            for scratch in [
                1_usize,
                4096,
                8192,
                16 * 1024,
                40 * 1024,
                64 * 1024,
                120 * 1024,
                200 * 1024,
                262_144,
                300 * 1024,
                316_264,
                400 * 1024,
                SORTED_TREE_UPDATE_SCRATCH_BYTES,
            ] {
                let point = stage2_mutate(&mut plain, root, changed, keys.iter().copied(), scratch);
                let probe = std::rc::Rc::new(BatchProbe::default());
                let mut counted = CountingStore {
                    inner: MemoryStore::default(),
                    probe: probe.clone(),
                };
                let (counted_root, _) = stage2_compact_table(&mut counted.inner, size);
                assert_eq!(counted_root, root);
                let batched =
                    stage2_mutate(&mut counted, root, changed, keys.iter().copied(), scratch);
                match (&point, &batched) {
                    (Ok((point_root, _)), Ok((batch_root, counters))) => {
                        assert_eq!(point_root, batch_root, "size={size} scratch={scratch}");
                        assert!(
                            counters.peak_scratch_bytes <= scratch,
                            "size={size} scratch={scratch} peak={}",
                            counters.peak_scratch_bytes
                        );
                        survivor += 1;
                    }
                    (Ok(_), Err(error)) => {
                        panic!("size={size} scratch={scratch}: batch failed with {error:?}")
                    }
                    // Both routes reject a ledger that cannot hold one page; the
                    // workspace's fallback still sees the same error class.
                    (Err(point_error), Err(batch_error)) => {
                        assert_eq!(point_error, batch_error, "size={size} scratch={scratch}");
                    }
                    (Err(error), Ok(_)) => {
                        panic!("size={size} scratch={scratch}: point route failed with {error:?}")
                    }
                }
                // A narrower chunk still never demands a child twice inside the
                // node, and never exceeds the declared ceiling.
                assert!(probe.widest() <= TREE_BATCH_CHILDREN);
            }
            assert!(
                survivor > 0,
                "size={size}: no scratch survived the batch route"
            );
        }
    }

    #[test]
    fn stage2_batch_work_counts_follow_changed_keys_not_chunk_size() {
        let mut store = MemoryStore::default();
        let (root, record) = stage2_compact_table(&mut store, 13_000);
        let changed = crate::tree::inode::InodeRecordV1 {
            namespace_ref_count: 5,
            ..record
        };
        let mut previous = 0;
        for count in [1_u64, 10, 100] {
            let probe = std::rc::Rc::new(BatchProbe::default());
            let mut counted = CountingStore {
                inner: MemoryStore::default(),
                probe: probe.clone(),
            };
            let (counted_root, _) = stage2_compact_table(&mut counted.inner, 13_000);
            assert_eq!(counted_root, root);
            let keys = (0..count)
                .map(|index| 1 + index * (13_000 / count).max(1))
                .collect::<Vec<_>>();
            let (_, counters) = stage2_mutate(
                &mut counted,
                root,
                changed,
                keys.iter().copied(),
                SORTED_TREE_UPDATE_SCRATCH_BYTES,
            )
            .unwrap();
            // The batch route reads exactly the pages the point route reads; it
            // never re-reads a child to fill a chunk.
            assert_eq!(
                probe.requested() as u64,
                counters.nodes_read.saturating_sub(1)
            );
            assert!(counters.nodes_read >= previous);
            previous = counters.nodes_read;
        }
    }

    #[test]
    fn stage2_batch_work_counts_follow_namespace_size_at_fixed_k() {
        let mut store = MemoryStore::default();
        let record = crate::tree::inode::InodeRecordV1 {
            kind: crate::tree::inode::InodeKind::RegularFile,
            namespace_ref_count: 1,
            content_root: value(1),
            metadata_root: value(2),
        };
        let changed = crate::tree::inode::InodeRecordV1 {
            namespace_ref_count: 2,
            ..record
        };
        let mut previous = 0;
        for size in [1_000_u64, 13_000] {
            let probe = std::rc::Rc::new(BatchProbe::default());
            let mut counted = CountingStore {
                inner: MemoryStore::default(),
                probe: probe.clone(),
            };
            let (root, _) = stage2_compact_table(&mut counted.inner, size);
            let (_, counters) = stage2_mutate(
                &mut counted,
                root,
                changed,
                std::iter::once(1_u64),
                SORTED_TREE_UPDATE_SCRATCH_BYTES,
            )
            .unwrap();
            assert_eq!(
                probe.requested() as u64,
                counters.nodes_read.saturating_sub(1)
            );
            assert!(counters.nodes_read > previous, "size {size}");
            previous = counters.nodes_read;
            let _ = &mut store;
        }
    }

    #[test]
    fn stage2_insufficient_batch_scratch_is_the_declared_fallback_error() {
        let mut store = MemoryStore::default();
        let (root, record) = stage2_compact_table(&mut store, 13_000);
        let changed = crate::tree::inode::InodeRecordV1 {
            namespace_ref_count: 3,
            ..record
        };
        // One byte cannot cover even one chunk's worst-case retained bytes, so
        // the bounded batch charge fails with exactly the error the workspace
        // fallback matches.
        let forced = stage2_mutate(&mut store, root, changed, std::iter::once(1_u64), 1);
        assert_eq!(forced.unwrap_err(), CoreError::ObjectLimitExceeded);
    }

    /// The entered spine for key 1 in a 13 000-inode compact table: the root's
    /// first level-1 branch and that branch's first leaf.
    fn stage2_spine(
        store: &MemoryStore,
        root: crate::tree::inode::InodeTableRoot,
    ) -> (ObjectId, ObjectId, u8) {
        use crate::tree::compact::{self, InodeNode};
        let InodeNode::Branch { children, .. } =
            compact::decode_inode(&store.get(root.0).unwrap()).unwrap()
        else {
            panic!("expected a compact inode branch root");
        };
        let branch_id = children[0].1;
        let InodeNode::Branch {
            level,
            children: grandchildren,
            ..
        } = compact::decode_inode(&store.get(branch_id).unwrap()).unwrap()
        else {
            panic!("expected a compact inode branch at level 1");
        };
        (branch_id, grandchildren[0].1, level)
    }

    /// Runs the same delta on the point route and on the bounded batch route
    /// after the same tampering, and returns both outcomes. The two stores are
    /// built from the same seed, so equal outcomes mean equivalent behaviour.
    fn stage2_case(
        tamper: &dyn Fn(
            &mut MemoryStore,
            crate::tree::inode::InodeTableRoot,
        ) -> Option<crate::tree::inode::InodeTableRoot>,
        keys: &[u64],
    ) -> (Result<ObjectId, CoreError>, Result<ObjectId, CoreError>) {
        use crate::tree::inode::InodeRecordV1;
        let mut plain = MemoryStore::default();
        let (root, record) = stage2_compact_table(&mut plain, 13_000);
        let base_root = root;
        let root = tamper(&mut plain, root).unwrap_or(root);
        let changed = InodeRecordV1 {
            namespace_ref_count: 7,
            ..record
        };
        let point = stage2_mutate(
            &mut plain,
            root,
            changed,
            keys.iter().copied(),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
        )
        .map(|(root, _)| root.0);

        let mut counted = CountingStore {
            inner: MemoryStore::default(),
            probe: std::rc::Rc::new(BatchProbe::default()),
        };
        let (counted_root, counted_record) = stage2_compact_table(&mut counted.inner, 13_000);
        assert_eq!(counted_root, base_root);
        let counted_root = tamper(&mut counted.inner, counted_root).unwrap_or(counted_root);
        let counted_changed = InodeRecordV1 {
            namespace_ref_count: 7,
            ..counted_record
        };
        let batched = stage2_mutate(
            &mut counted,
            counted_root,
            counted_changed,
            keys.iter().copied(),
            SORTED_TREE_UPDATE_SCRATCH_BYTES,
        )
        .map(|(root, _)| root.0);
        (point, batched)
    }

    #[test]
    fn stage2_malformed_siblings_are_rejected_identically_on_both_routes() {
        use crate::tree::compact::{self, InodeNode, InodeSerial};

        let healthy = stage2_case(&|_, _| None, &[1]);
        assert!(healthy.0.is_ok() && healthy.1.is_ok(), "{healthy:?}");

        let mut probe = MemoryStore::default();
        let (root, _) = stage2_compact_table(&mut probe, 13_000);
        let (branch_id, leaf_id, level) = stage2_spine(&probe, root);
        let leaf = probe.objects[&leaf_id].clone();
        let branch = probe.objects[&branch_id].clone();
        let ids = (branch_id, leaf_id, level);

        // 1. Wrong child level.
        let case = stage2_case(
            &|store, _| {
                let mut tampered = leaf.clone();
                tampered[11] = ids.2;
                store.objects.insert(ids.1, tampered);
                None
            },
            &[1],
        );
        assert_eq!(case.0, case.1, "wrong child level differs");
        assert!(case.0.is_err(), "wrong child level accepted: {case:?}");

        // 2. Wrong maximum key for the leaf's binding in its parent.
        let case = stage2_case(
            &|store, _| {
                let InodeNode::Branch {
                    level, children, ..
                } = compact::decode_inode(&branch).unwrap()
                else {
                    panic!("expected a branch");
                };
                let mut rows = children.clone();
                let index = rows.iter().position(|(_, id)| *id == ids.1).unwrap();
                let wrong = InodeSerial::new(rows[index].0.get() + 1).unwrap();
                rows[index] = (wrong, ids.1);
                let total = rows.len() as u64;
                let node = InodeNode::Branch {
                    level,
                    subtree_count: total,
                    children: rows,
                };
                store
                    .objects
                    .insert(ids.0, compact::encode_inode(&node).unwrap());
                None
            },
            &[1],
        );
        assert_eq!(case.0, case.1, "wrong child maximum differs");
        assert!(case.0.is_err(), "wrong child maximum accepted: {case:?}");

        // 3. Inflated parent subtree count.
        let case = stage2_case(
            &|store, _| {
                let InodeNode::Branch {
                    level, children, ..
                } = compact::decode_inode(&branch).unwrap()
                else {
                    panic!("expected a branch");
                };
                let node = InodeNode::Branch {
                    level,
                    subtree_count: 13_000,
                    children,
                };
                store
                    .objects
                    .insert(ids.0, compact::encode_inode(&node).unwrap());
                None
            },
            &[1],
        );
        assert_eq!(case.0, case.1, "parent count differs");
        assert!(case.0.is_err(), "parent count accepted: {case:?}");

        // 4. Underfilled child.
        let case = stage2_case(
            &|store, _| {
                let mut short = leaf.clone();
                short.truncate(31 + 49 * 81);
                store.objects.insert(ids.1, short);
                None
            },
            &[1],
        );
        assert_eq!(case.0, case.1, "underfilled child differs");
        assert!(case.0.is_err(), "underfilled child accepted: {case:?}");

        // 5. Corrupted demanded child.
        let case = stage2_case(
            &|store, _| {
                let mut corrupted = leaf.clone();
                corrupted[31] ^= 0xff;
                store.objects.insert(ids.1, corrupted);
                None
            },
            &[1],
        );
        assert_eq!(case.0, case.1, "corrupted child differs");
        assert_eq!(case.0, Err(CoreError::IdentityMismatch));

        // 6. Missing dependency.
        let case = stage2_case(
            &|store, _| {
                store.objects.remove(&ids.1);
                None
            },
            &[1],
        );
        assert_eq!(case.0, case.1, "missing dependency differs");
        assert!(case.0.is_err(), "missing dependency accepted: {case:?}");

        // 7. Wrong child level through a correctly addressed page: the entered
        //    leaf slot is rebound to a copy of its own parent branch, whose
        //    bytes and identity are consistent, and the spine is re-rooted.
        let case = stage2_case(
            &|store, root| {
                let InodeNode::Branch {
                    level, children, ..
                } = compact::decode_inode(&branch).unwrap()
                else {
                    panic!("expected a branch");
                };
                let replacement = ObjectId::for_bytes(&branch);
                store.objects.insert(replacement, branch.clone());
                let mut rows = children.clone();
                let index = rows.iter().position(|(_, id)| *id == ids.1).unwrap();
                rows[index] = (rows[index].0, replacement);
                let total = rows.len() as u64;
                let new_branch = compact::encode_inode(&InodeNode::Branch {
                    level,
                    subtree_count: total,
                    children: rows,
                })
                .unwrap();
                let new_branch_id = ObjectId::for_bytes(&new_branch);
                store.objects.insert(new_branch_id, new_branch);
                let InodeNode::Branch {
                    level: root_level,
                    subtree_count,
                    children: root_children,
                } = compact::decode_inode(&store.get(root.0).unwrap()).unwrap()
                else {
                    panic!("expected a branch root");
                };
                let mut root_rows = root_children.clone();
                let position = root_rows.iter().position(|(_, id)| *id == ids.0).unwrap();
                root_rows[position] = (root_rows[position].0, new_branch_id);
                let new_root = compact::encode_inode(&InodeNode::Branch {
                    level: root_level,
                    subtree_count,
                    children: root_rows,
                })
                .unwrap();
                let new_root_id = ObjectId::for_bytes(&new_root);
                store.objects.insert(new_root_id, new_root);
                Some(crate::tree::inode::InodeTableRoot(new_root_id))
            },
            &[1],
        );
        assert_eq!(case.0, case.1, "wrong child level differs");
        assert!(case.0.is_err(), "wrong child level accepted: {case:?}");

        // Clustered and spread changed keys exercise the same bounded chunks.
        for keys in [
            vec![1_u64, 2, 3, 4, 5],
            vec![1, 4_000, 8_000, 12_999],
            (1..=13_000).step_by(97).collect::<Vec<u64>>(),
        ] {
            let case = stage2_case(&|_, _| None, &keys);
            assert!(case.0.is_ok() && case.1.is_ok(), "keys {keys:?}: {case:?}");
            assert_eq!(case.0, case.1, "keys {keys:?}");
        }
    }
}

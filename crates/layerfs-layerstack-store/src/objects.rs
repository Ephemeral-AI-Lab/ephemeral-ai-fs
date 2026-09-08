mod admission;
mod diagnostic;
pub(crate) use admission::PreparedAdmission;
mod pack;
mod read;
mod spill;
#[cfg(test)]
use spill::SeenStorage;
pub use spill::SpillableObjectSet;
use spill::{IdOrder, SpillObjects, TempPath, temporary_file};

use crate::{Result, StoreError};
use layerfs_content::filesystem::{self, ContentChange, ReconcileConflict};
use layerfs_content::object::access::{ObjectRead, ObjectStore};
use layerfs_content::object::references::referenced_objects;
use layerfs_content::{CoreError, CoreResult, ObjectId};
use rusqlite::{Connection, params_from_iter, types::Value};
#[cfg(test)]
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub const OBJECT_PAGE_COUNT: usize = 128;
pub const OBJECT_PAGE_BYTES: usize = 4 * 1024 * 1024;
pub const ADMISSION_BATCH_COUNT: usize = 8191;
pub const ADMISSION_BATCH_BYTES: usize = OBJECT_PAGE_BYTES - 1;
pub(crate) const INITIALIZATION_ADMISSION_BATCH_COUNT: usize = ADMISSION_BATCH_COUNT;
pub(crate) const INITIALIZATION_SLAB_BYTES: usize = 256 * 1024;
pub(crate) const INITIALIZATION_SLAB_OBJECTS: usize = 512;
pub(crate) const INITIALIZATION_SLAB_QUEUE_SLOTS: usize = 4;
pub(crate) const INITIALIZATION_TASK_STRUCTURAL_BYTES: usize = 256 * 1024;
const CANDIDATE_MEMORY_BYTES: usize = 8 * 1024 * 1024;
const CANDIDATE_INDEX_BYTES: usize = 64 * 1024 * 1024;
// C: cumulative metadata-lookup allowance, not resident memory. The frozen
// full157 bound is 3763 attached flat-root cursors * 2 grants * 131136 bytes.
const CORRESPONDENCE_OPERATION_RESERVATION_BYTES: u64 = 1024 * 1024 * 1024;
const CANDIDATE_SPILL_BUFFER_BYTES: usize = 1024 * 1024;

#[cfg(feature = "test-instrumentation")]
thread_local! {
    static READ_BATCH_COUNTERS: std::cell::RefCell<ReadBatchCounters> = const {
        std::cell::RefCell::new(ReadBatchCounters {
            unique_hashes: 0,
            cloned_bytes: 0,
        })
    };
}

#[cfg(feature = "test-instrumentation")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReadBatchCounters {
    pub unique_hashes: u64,
    pub cloned_bytes: u64,
}

#[cfg(feature = "test-instrumentation")]
pub fn reset_read_batch_counters() {
    READ_BATCH_COUNTERS.with(|counters| *counters.borrow_mut() = ReadBatchCounters::default());
}

#[cfg(feature = "test-instrumentation")]
pub fn read_batch_counters() -> ReadBatchCounters {
    READ_BATCH_COUNTERS.with(|counters| *counters.borrow())
}

#[cfg(feature = "test-instrumentation")]
fn note_read_batch_hash() {
    READ_BATCH_COUNTERS.with(|counters| {
        counters.borrow_mut().unique_hashes += 1;
    });
}

#[cfg(not(feature = "test-instrumentation"))]
fn note_read_batch_hash() {}

#[cfg(feature = "test-instrumentation")]
fn note_read_batch_clone(bytes: usize) {
    READ_BATCH_COUNTERS.with(|counters| {
        let mut counters = counters.borrow_mut();
        counters.cloned_bytes = counters.cloned_bytes.saturating_add(bytes as u64);
    });
}

#[cfg(not(feature = "test-instrumentation"))]
fn note_read_batch_clone(_bytes: usize) {}

thread_local! {
    static PARENT_PAYLOAD_COPY_BYTES: std::cell::Cell<Option<u64>> = const {
        std::cell::Cell::new(None)
    };
}

pub(crate) struct ParentPayloadCopyCounter(Option<u64>);

impl ParentPayloadCopyCounter {
    pub(crate) fn start() -> Self {
        Self(PARENT_PAYLOAD_COPY_BYTES.with(|bytes| bytes.replace(Some(0))))
    }

    pub(crate) fn bytes(&self) -> u64 {
        PARENT_PAYLOAD_COPY_BYTES.with(|bytes| bytes.get().unwrap_or(0))
    }
}

impl Drop for ParentPayloadCopyCounter {
    fn drop(&mut self) {
        PARENT_PAYLOAD_COPY_BYTES.with(|bytes| bytes.set(self.0));
    }
}

fn note_parent_payload_copy(bytes: usize) {
    PARENT_PAYLOAD_COPY_BYTES.with(|total| {
        if let Some(current) = total.get() {
            total.set(Some(current.saturating_add(bytes as u64)));
        }
    });
}

#[derive(Debug, Eq, PartialEq)]
pub struct CanonicalObject {
    pub id: ObjectId,
    pub bytes: Vec<u8>,
}

impl Clone for CanonicalObject {
    fn clone(&self) -> Self {
        note_parent_payload_copy(self.bytes.len());
        Self {
            id: self.id,
            bytes: self.bytes.clone(),
        }
    }
}

pub(crate) struct FinalizedObjectSlab {
    pub objects: Vec<AuthenticatedCanonicalObject>,
    pub payload_bytes: usize,
}

/// Immutable ownership of bytes whose identity and complete outer framing were checked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthenticatedCanonicalObject(CanonicalObject, PhysicalHints);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PhysicalHints {
    prior_ids: [Option<ObjectId>; 4],
    first_span: Option<(u64, u32)>,
    has_predecessor: bool,
    diagnostic: u8,
    diagnostic_grants: u8,
}

// Diagnostic bytes must not alter canonical batching or association reservations.
#[allow(dead_code)]
struct UndiagnosedHints {
    prior_ids: [Option<ObjectId>; 4],
    first_span: Option<(u64, u32)>,
    has_predecessor: bool,
}
const _: () = {
    assert!(std::mem::size_of::<PhysicalHints>() == std::mem::size_of::<UndiagnosedHints>());
    assert!(std::mem::align_of::<PhysicalHints>() == std::mem::align_of::<UndiagnosedHints>());
    assert!(
        std::mem::size_of::<AuthenticatedCanonicalObject>()
            == std::mem::size_of::<(CanonicalObject, UndiagnosedHints)>()
    );
};

impl std::ops::Deref for AuthenticatedCanonicalObject {
    type Target = CanonicalObject;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<CanonicalObject> for AuthenticatedCanonicalObject {
    fn as_ref(&self) -> &CanonicalObject {
        &self.0
    }
}
impl AsRef<CanonicalObject> for CanonicalObject {
    fn as_ref(&self) -> &CanonicalObject {
        self
    }
}
impl AuthenticatedCanonicalObject {
    pub(crate) fn prior_ids(&self) -> &[Option<ObjectId>; 4] {
        &self.1.prior_ids
    }
    fn new(bytes: Vec<u8>, expected: Option<ObjectId>) -> CoreResult<Self> {
        let id = match expected {
            Some(id) => {
                layerfs_content::authenticate_identity(&bytes, id)?;
                id
            }
            None => layerfs_content::identify_canonical(&bytes)?.0,
        };
        Ok(Self(
            CanonicalObject { id, bytes },
            PhysicalHints::default(),
        ))
    }
}

pub(crate) struct InitializationTaskObjectBuffer {
    objects: Vec<AuthenticatedCanonicalObject>,
    payload_bytes: usize,
}

impl InitializationTaskObjectBuffer {
    pub(crate) fn new() -> Self {
        Self {
            objects: Vec::with_capacity(128),
            payload_bytes: 0,
        }
    }

    pub(crate) fn explicit_owned_bytes(&self) -> u64 {
        self.payload_bytes as u64
            + (self.objects.capacity() * std::mem::size_of::<AuthenticatedCanonicalObject>()) as u64
    }

    pub(crate) fn hash_invocations(&self) -> u64 {
        self.objects.len() as u64
    }

    pub(crate) fn move_into(self, store: &mut FinalizedOutputWriter) -> CoreResult<()> {
        for object in self.objects {
            store.push_authenticated(object)?;
        }
        Ok(())
    }

    fn push_owned(&mut self, canonical: Vec<u8>) -> CoreResult<ObjectId> {
        let owned = self
            .payload_bytes
            .checked_add(canonical.len())
            .and_then(|payload| {
                payload.checked_add(
                    self.objects
                        .len()
                        .checked_add(1)?
                        .checked_mul(std::mem::size_of::<AuthenticatedCanonicalObject>())?,
                )
            })
            .ok_or(CoreError::LengthOverflow)?;
        if owned > INITIALIZATION_TASK_STRUCTURAL_BYTES {
            return Err(CoreError::ObjectLimitExceeded);
        }
        let canonical_len = canonical.len();
        let object = AuthenticatedCanonicalObject::new(canonical, None)?;
        let id = object.id;
        self.payload_bytes = self
            .payload_bytes
            .checked_add(canonical_len)
            .ok_or(CoreError::LengthOverflow)?;
        self.objects.push(object);
        Ok(id)
    }
}

impl ObjectStore for InitializationTaskObjectBuffer {
    fn get(&self, _id: ObjectId) -> CoreResult<Vec<u8>> {
        Err(CoreError::InvalidRecord("direct structural get"))
    }

    fn put(&mut self, canonical: &[u8]) -> CoreResult<ObjectId> {
        self.push_owned(canonical.to_vec())
    }

    fn put_owned(&mut self, canonical: Vec<u8>) -> CoreResult<ObjectId> {
        self.push_owned(canonical)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct OutputWriterMetrics {
    pub selected_memory_bytes: u64,
    pub selected_spill_bytes: u64,
    pub selected_storage_authentication_ns: u64,
    pub handoffs: u64,
    pub objects: u64,
    pub payload_bytes: u64,
    pub payload_capacity_bytes: u64,
    pub canonical_hash_calls: u64,
    pub blocked_ns: u64,
    pub partial_peak_objects: u64,
    pub partial_peak_payload_bytes: u64,
    pub candidate_copy_bytes: u64,
    pub parent_payload_copy_bytes: u64,
    pub structural_peak_bytes: u64,
    pub producer_wall_ns: u64,
    pub producer_completion_offset_ns: u64,
    pub producer_tasks: u64,
    pub producer_files: u64,
    pub producer_bytes: u64,
}

impl OutputWriterMetrics {
    // Work counts/time totals sum. Per-worker slab peaks remain maxima; the sum
    // of structural peaks is a conservative simultaneous-memory bound.
    pub(crate) fn merge(&mut self, other: Self) {
        macro_rules! sum { ($($field:ident),* $(,)?) => { $(self.$field = self.$field.saturating_add(other.$field);)* }; }
        macro_rules! peak { ($($field:ident),* $(,)?) => { $(self.$field = self.$field.max(other.$field);)* }; }
        sum!(
            selected_memory_bytes,
            selected_spill_bytes,
            selected_storage_authentication_ns,
            handoffs,
            objects,
            payload_bytes,
            payload_capacity_bytes,
            canonical_hash_calls,
            blocked_ns,
            candidate_copy_bytes,
            parent_payload_copy_bytes,
            structural_peak_bytes,
            producer_tasks,
            producer_files,
            producer_bytes
        );
        peak!(
            partial_peak_objects,
            partial_peak_payload_bytes,
            producer_wall_ns,
            producer_completion_offset_ns
        );
    }
}

#[derive(Default)]
pub(crate) struct OutputQueueMetrics {
    queued: AtomicU64,
    queued_bytes: AtomicU64,
    peak: AtomicU64,
    peak_bytes: AtomicU64,
}

impl OutputQueueMetrics {
    fn before_send(&self, bytes: usize) {
        let queued = self.queued.fetch_add(1, Ordering::AcqRel) + 1;
        let queued_bytes =
            self.queued_bytes.fetch_add(bytes as u64, Ordering::AcqRel) + bytes as u64;
        self.peak.fetch_max(
            queued.min(INITIALIZATION_SLAB_QUEUE_SLOTS as u64),
            Ordering::Relaxed,
        );
        self.peak_bytes.fetch_max(
            queued_bytes.min((INITIALIZATION_SLAB_QUEUE_SLOTS * INITIALIZATION_SLAB_BYTES) as u64),
            Ordering::Relaxed,
        );
    }

    fn send_failed(&self, bytes: usize) {
        self.received(bytes);
    }

    pub(crate) fn received(&self, bytes: usize) {
        self.queued.fetch_sub(1, Ordering::AcqRel);
        self.queued_bytes.fetch_sub(bytes as u64, Ordering::AcqRel);
    }

    pub(crate) fn peak(&self) -> u64 {
        self.peak.load(Ordering::Relaxed)
    }

    pub(crate) fn peak_bytes(&self) -> u64 {
        self.peak_bytes.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
pub(crate) struct OutputPipelineMetrics {
    pub wall_ns: u64,
    pub consumer_idle_ns: u64,
    pub last_receive_ns: u64,
    pub queue_peak: u64,
    pub queue_peak_bytes: u64,
    pub producer_peak: u64,
    pub producers_after: u64,
}

// One bounded ownership/drain/join implementation for native and Workspace inputs.
// Keep lifecycle callbacks explicit rather than introducing a configuration wrapper.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_finalized_output<I, S: Send, T: Send>(
    worker_limit: usize,
    task_count: usize,
    tasks: I,
    cancelled: &std::sync::atomic::AtomicBool,
    initialize: impl Fn(usize) -> Result<S> + Sync,
    step: impl Fn(&mut S, usize, I::Item, &mut FinalizedOutputWriter) -> Result<()> + Sync,
    finish: impl Fn(S) -> Result<T> + Sync,
    mut consume: impl FnMut(Vec<AuthenticatedCanonicalObject>) -> Result<()>,
) -> Result<(Vec<(T, OutputWriterMetrics)>, OutputPipelineMetrics)>
where
    I: Iterator + Send,
    I::Item: Send,
{
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::atomic::Ordering;
    if worker_limit == 0 && task_count != 0 {
        return Err(StoreError::InvalidInput("canonical output worker limit"));
    }
    let workers = worker_limit.min(task_count);
    let started = Instant::now();
    let tasks = Mutex::new(tasks.enumerate());
    let claimed = AtomicU64::new(0);
    let completed = AtomicU64::new(0);
    let queue = std::sync::Arc::new(OutputQueueMetrics::default());
    let (sender, receiver) = std::sync::mpsc::sync_channel(INITIALIZATION_SLAB_QUEUE_SLOTS);
    let active = AtomicU64::new(0);
    let peak = AtomicU64::new(0);
    let mut metrics = OutputPipelineMetrics::default();
    let output = std::thread::scope(|scope| {
        let handles = (0..workers)
            .map(|index| {
                let mut writer = FinalizedOutputWriter::new(sender.clone(), queue.clone());
                let (initialize, step, finish) = (&initialize, &step, &finish);
                let (tasks, claimed, completed, active, peak) =
                    (&tasks, &claimed, &completed, &active, &peak);
                scope.spawn(move || {
                    struct Active<'a>(&'a AtomicU64);
                    impl Drop for Active<'_> {
                        fn drop(&mut self) {
                            self.0.fetch_sub(1, Ordering::AcqRel);
                        }
                    }
                    let count = active.fetch_add(1, Ordering::AcqRel) + 1;
                    peak.fetch_max(count, Ordering::Relaxed);
                    let _active = Active(active);
                    let producer_started = Instant::now();
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        let mut state = initialize(index)?;
                        let mut task_total = 0_u64;
                        loop {
                            if cancelled.load(Ordering::Acquire) {
                                break;
                            }
                            let next = {
                                let mut tasks = tasks
                                    .lock()
                                    .map_err(|_| StoreError::Integrity("canonical task source"))?;
                                if cancelled.load(Ordering::Acquire) {
                                    None
                                } else {
                                    tasks.next()
                                }
                            };
                            let Some((ordinal, task)) = next else { break };
                            if ordinal >= task_count {
                                return Err(StoreError::Integrity("canonical task count"));
                            }
                            claimed.fetch_add(1, Ordering::Relaxed);
                            step(&mut state, ordinal, task, &mut writer)?;
                            task_total += 1;
                            completed.fetch_add(1, Ordering::Relaxed);
                        }
                        // Result journals seal before the final output flush. Neither a
                        // finish error nor a writer error may publish successful coverage.
                        let output = finish(state)?;
                        let mut writer = writer.finish()?;
                        writer.producer_tasks = task_total;
                        writer.producer_wall_ns = elapsed_ns(producer_started);
                        writer.producer_completion_offset_ns = elapsed_ns(started);
                        Ok((output, writer))
                    }))
                    .unwrap_or_else(|_| {
                        Err(StoreError::Integrity("canonical output producer panic"))
                    });
                    if result.is_err() {
                        cancelled.store(true, Ordering::Release);
                    }
                    result
                })
            })
            .collect::<Vec<_>>();
        drop(sender);
        let mut failure = None;
        loop {
            let waiting = Instant::now();
            let received = receiver.recv();
            metrics.consumer_idle_ns = metrics.consumer_idle_ns.saturating_add(elapsed_ns(waiting));
            let Ok(slab) = received else { break };
            metrics.last_receive_ns = elapsed_ns(started);
            queue.received(slab.payload_bytes);
            if failure.is_none() {
                let result = catch_unwind(AssertUnwindSafe(|| consume(slab.objects))).unwrap_or(
                    Err(StoreError::Integrity("canonical output consumer panic")),
                );
                if let Err(error) = result {
                    failure = Some(error);
                    cancelled.store(true, Ordering::Release);
                }
            }
            // Drain even after failure: every sender and retained source owner joins.
        }
        let mut output = Vec::with_capacity(workers);
        for handle in handles {
            match handle.join() {
                Ok(Ok(result)) => output.push(result),
                Ok(Err(error)) => {
                    failure.get_or_insert(error);
                }
                Err(_) => {
                    failure.get_or_insert(StoreError::Integrity("canonical output producer"));
                }
            }
        }
        if failure.is_none()
            && !cancelled.load(Ordering::Acquire)
            && (claimed.load(Ordering::Relaxed) != task_count as u64
                || completed.load(Ordering::Relaxed) != task_count as u64
                || tasks
                    .lock()
                    .map_err(|_| StoreError::Integrity("canonical task source"))?
                    .next()
                    .is_some())
        {
            failure = Some(StoreError::Integrity("canonical task coverage"));
            cancelled.store(true, Ordering::Release);
        }
        failure.map_or(Ok(output), Err)
    })?;
    metrics.wall_ns = elapsed_ns(started);
    metrics.queue_peak = queue.peak();
    metrics.queue_peak_bytes = queue.peak_bytes();
    metrics.producer_peak = peak.load(Ordering::Relaxed);
    metrics.producers_after = active.load(Ordering::Acquire);
    Ok((output, metrics))
}

pub struct FinalizedOutputWriter {
    sender: std::sync::mpsc::SyncSender<FinalizedObjectSlab>,
    queue: std::sync::Arc<OutputQueueMetrics>,
    objects: Vec<AuthenticatedCanonicalObject>,
    payload_bytes: usize,
    metrics: OutputWriterMetrics,
}

pub(crate) struct InitializationDirectAdmissionWriter<'admission> {
    admission: &'admission mut CheckedOutputAdmission,
    error: Option<StoreError>,
    transient_owned_bytes: u64,
    pub metrics: OutputWriterMetrics,
}

impl<'admission> InitializationDirectAdmissionWriter<'admission> {
    pub(crate) fn new(admission: &'admission mut CheckedOutputAdmission) -> Self {
        Self {
            admission,
            error: None,
            transient_owned_bytes: 0,
            metrics: OutputWriterMetrics::default(),
        }
    }

    pub(crate) fn error(&mut self, fallback: CoreError) -> StoreError {
        self.error.take().unwrap_or_else(|| fallback.into())
    }

    fn push_owned(&mut self, canonical: Vec<u8>, copied: bool) -> CoreResult<ObjectId> {
        let object = AuthenticatedCanonicalObject::new(canonical, None)?;
        let id = object.id;
        self.metrics.canonical_hash_calls += 1;
        let bytes = object.bytes.len() as u64;
        self.metrics.payload_capacity_bytes = self
            .metrics
            .payload_capacity_bytes
            .saturating_add(object.bytes.capacity() as u64);
        self.admission
            .observe_final_owned_bytes(self.transient_owned_bytes, object.bytes.capacity() as u64);
        if let Err(error) = self.admission.admit_object(object) {
            self.error = Some(error);
            return Err(CoreError::Io);
        }
        self.admission
            .observe_final_owned_bytes(self.transient_owned_bytes, 0);
        self.metrics.objects += 1;
        self.metrics.payload_bytes = self.metrics.payload_bytes.saturating_add(bytes);
        if copied {
            self.metrics.candidate_copy_bytes =
                self.metrics.candidate_copy_bytes.saturating_add(bytes);
        }
        Ok(id)
    }
}

impl ObjectStore for InitializationDirectAdmissionWriter<'_> {
    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
        if let Some(index) = self.admission.incoming_index.get(&id) {
            return Ok(self.admission.incoming[*index].bytes.clone());
        }
        if let Some(index) = self.admission.pending.get(&id) {
            return Ok(self.admission.batch[*index].bytes.clone());
        }
        self.admission
            .db
            .read_object_row(id)
            .map_err(core_read_error)
    }

    fn put(&mut self, canonical: &[u8]) -> CoreResult<ObjectId> {
        self.push_owned(canonical.to_vec(), true)
    }

    fn put_owned(&mut self, canonical: Vec<u8>) -> CoreResult<ObjectId> {
        self.push_owned(canonical, false)
    }

    fn note_transient_owned_bytes(&mut self, bytes: u64) -> CoreResult<()> {
        self.transient_owned_bytes = bytes;
        self.admission.observe_final_owned_bytes(bytes, 0);
        Ok(())
    }
}

impl FinalizedOutputWriter {
    pub(crate) fn new(
        sender: std::sync::mpsc::SyncSender<FinalizedObjectSlab>,
        queue: std::sync::Arc<OutputQueueMetrics>,
    ) -> Self {
        Self {
            sender,
            queue,
            objects: Vec::with_capacity(INITIALIZATION_SLAB_OBJECTS),
            payload_bytes: 0,
            metrics: OutputWriterMetrics::default(),
        }
    }

    /// Accept only completed selected output from ObjectBuffer::finish or the
    /// complete-file builder's established finality contract.
    pub fn send_selected(&mut self, objects: DeferredObjectStore) -> Result<()> {
        let bytes = objects.encoded_bytes();
        if matches!(objects.storage, DeferredObjects::Spill(_)) {
            self.metrics.selected_spill_bytes =
                self.metrics.selected_spill_bytes.saturating_add(bytes);
        } else {
            self.metrics.selected_memory_bytes =
                self.metrics.selected_memory_bytes.saturating_add(bytes);
        }
        let storage_authentication_ns = objects.consume_prevalidated_pages(|page| {
            for object in page {
                self.push_object(object, false)?;
            }
            Ok(())
        })?;
        self.metrics.selected_storage_authentication_ns = self
            .metrics
            .selected_storage_authentication_ns
            .saturating_add(storage_authentication_ns);
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<OutputWriterMetrics> {
        self.flush()?;
        Ok(self.metrics)
    }

    pub(crate) fn note_hash_invocations(&mut self, calls: u64) {
        self.metrics.canonical_hash_calls = self.metrics.canonical_hash_calls.saturating_add(calls);
    }

    fn flush(&mut self) -> Result<()> {
        if self.objects.is_empty() {
            return Ok(());
        }
        let slab = FinalizedObjectSlab {
            objects: std::mem::take(&mut self.objects),
            payload_bytes: std::mem::take(&mut self.payload_bytes),
        };
        self.queue.before_send(slab.payload_bytes);
        match self.sender.try_send(slab) {
            Ok(()) => {}
            Err(std::sync::mpsc::TrySendError::Full(slab)) => {
                let started = Instant::now();
                if let Err(error) = self.sender.send(slab) {
                    self.queue.send_failed(error.0.payload_bytes);
                    return Err(StoreError::Integrity("initialization slab receiver"));
                }
                self.metrics.blocked_ns =
                    self.metrics.blocked_ns.saturating_add(elapsed_ns(started));
            }
            Err(std::sync::mpsc::TrySendError::Disconnected(slab)) => {
                self.queue.send_failed(slab.payload_bytes);
                return Err(StoreError::Integrity("initialization slab receiver"));
            }
        }
        self.metrics.handoffs += 1;
        self.objects = Vec::with_capacity(INITIALIZATION_SLAB_OBJECTS);
        Ok(())
    }

    fn push_owned(&mut self, canonical: Vec<u8>, copied: bool) -> CoreResult<ObjectId> {
        let object = AuthenticatedCanonicalObject::new(canonical, None)?;
        let id = object.id;
        self.metrics.canonical_hash_calls += 1;
        self.push_object(object, copied)?;
        Ok(id)
    }

    fn push_authenticated(&mut self, object: AuthenticatedCanonicalObject) -> CoreResult<()> {
        self.push_object(object, false)
    }

    fn push_object(
        &mut self,
        object: AuthenticatedCanonicalObject,
        copied: bool,
    ) -> CoreResult<()> {
        let canonical = &object.bytes;
        if canonical.len() > INITIALIZATION_SLAB_BYTES {
            return Err(CoreError::ObjectLimitExceeded);
        }
        if !self.objects.is_empty()
            && (self.objects.len() == INITIALIZATION_SLAB_OBJECTS
                || self.payload_bytes.saturating_add(canonical.len()) > INITIALIZATION_SLAB_BYTES)
        {
            self.flush().map_err(|_| CoreError::Io)?;
        }
        self.payload_bytes += canonical.len();
        self.metrics.objects += 1;
        self.metrics.payload_bytes = self
            .metrics
            .payload_bytes
            .saturating_add(canonical.len() as u64);
        self.metrics.payload_capacity_bytes = self
            .metrics
            .payload_capacity_bytes
            .saturating_add(object.bytes.capacity() as u64);
        if copied {
            self.metrics.candidate_copy_bytes = self
                .metrics
                .candidate_copy_bytes
                .saturating_add(canonical.len() as u64);
        }
        self.objects.push(object);
        self.metrics.partial_peak_objects = self
            .metrics
            .partial_peak_objects
            .max(self.objects.len() as u64);
        self.metrics.partial_peak_payload_bytes = self
            .metrics
            .partial_peak_payload_bytes
            .max(self.payload_bytes as u64);
        Ok(())
    }
}

impl ObjectStore for FinalizedOutputWriter {
    fn get(&self, _id: ObjectId) -> CoreResult<Vec<u8>> {
        Err(CoreError::InvalidRecord("direct initialization get"))
    }

    fn put(&mut self, canonical: &[u8]) -> CoreResult<ObjectId> {
        self.push_owned(canonical.to_vec(), true)
    }

    fn put_owned(&mut self, canonical: Vec<u8>) -> CoreResult<ObjectId> {
        self.push_owned(canonical, false)
    }
}

pub trait ObjectSource: Send + Sync {
    fn read_object(&self, id: ObjectId) -> Result<Vec<u8>>;

    fn read_authenticated_objects(&self, ids: &[ObjectId]) -> Result<Vec<CanonicalObject>> {
        ids.iter()
            .map(|id| {
                let bytes = self.read_object(*id)?;
                layerfs_content::authenticate_identity(&bytes, *id)?;
                Ok(CanonicalObject { id: *id, bytes })
            })
            .collect()
    }
}

pub struct CoreReader<'a>(pub &'a dyn ObjectSource);

impl ObjectRead for CoreReader<'_> {
    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
        self.0.read_object(id).map_err(core_read_error)
    }

    fn get_authenticated_batch<F>(&self, ids: &[ObjectId], mut callback: F) -> CoreResult<()>
    where
        F: FnMut(ObjectId, &[u8]) -> CoreResult<()>,
    {
        if ids.len() > OBJECT_PAGE_COUNT {
            return Err(CoreError::InvalidRecord("object read page"));
        }
        let objects = self
            .0
            .read_authenticated_objects(ids)
            .map_err(core_read_error)?;
        if objects.len() != ids.len() {
            return Err(CoreError::MissingObject);
        }
        for (expected, object) in ids.iter().zip(objects) {
            if object.id != *expected {
                return Err(CoreError::IdentityMismatch);
            }
            callback(
                object.id,
                layerfs_content::decode_bytes_object(&object.bytes)?,
            )?;
        }
        Ok(())
    }

    fn with_authenticated_canonical<T, F>(&self, id: ObjectId, callback: F) -> CoreResult<T>
    where
        F: FnOnce(&[u8]) -> CoreResult<T>,
    {
        let mut objects = self
            .0
            .read_authenticated_objects(&[id])
            .map_err(core_read_error)?;
        if objects.len() != 1 {
            return Err(CoreError::MissingObject);
        }
        let object = objects.pop().expect("authenticated object");
        if object.id != id {
            return Err(CoreError::IdentityMismatch);
        }
        callback(&object.bytes)
    }
}

fn core_read_error(error: StoreError) -> CoreError {
    match error {
        StoreError::MissingObject(_) => CoreError::MissingObject,
        StoreError::Integrity(message) | StoreError::InvalidInput(message) => {
            CoreError::InvalidRecord(message)
        }
        StoreError::StoreMissing => CoreError::ValidationAuthorityUnavailable,
        StoreError::WrongStoreSchema => CoreError::SchemaMismatch,
        StoreError::CommitHeadMoved { .. }
        | StoreError::LayerHeadMoved { .. }
        | StoreError::LayerStackNameConflict { .. }
        | StoreError::BranchNameConflict { .. } => CoreError::PublicationConflict,
        StoreError::Core(error) => error,
        StoreError::StoreBusy
        | StoreError::StoreAlreadyExists
        | StoreError::NotFound(_)
        | StoreError::Database(_)
        | StoreError::Io(_) => CoreError::Io,
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BuildCounters {
    pub cdc_bytes_scanned: u64,
    pub encode_hash_invocations: u64,
    pub first_store_write_bytes: u64,
    pub reachable_copy_write_bytes: u64,
    pub spill_peak_bytes: u64,
    pub spill_count: u64,
}

pub struct BuiltRoot {
    pub root_id: ObjectId,
    pub objects: DeferredObjectStore,
    pub counters: BuildCounters,
}

pub struct CandidateReconciliation {
    pub root_id: ObjectId,
    pub objects: DeferredObjectStore,
    pub conflicts: Vec<ReconcileConflict>,
}

pub fn reconcile_candidate(
    source: &dyn ObjectSource,
    base_root: ObjectId,
    current_root: ObjectId,
    candidate_root: ObjectId,
) -> Result<CandidateReconciliation> {
    reconcile_candidate_with(source, base_root, current_root, candidate_root, |_| None)
}

pub fn reconcile_candidate_with(
    source: &dyn ObjectSource,
    base_root: ObjectId,
    current_root: ObjectId,
    candidate_root: ObjectId,
    choice: impl FnMut(
        &layerfs_content::filesystem::ReconcileConflict,
    ) -> Option<layerfs_content::filesystem::ReconcileChoice>,
) -> Result<CandidateReconciliation> {
    let mut objects = ObjectBuffer::new(source)?;
    let reconciled = filesystem::reconcile_with(
        &mut objects,
        base_root,
        current_root,
        candidate_root,
        choice,
    )?;
    let built = objects.finish(reconciled.root_id, 0)?;
    Ok(CandidateReconciliation {
        root_id: reconciled.root_id,
        objects: built.objects,
        conflicts: reconciled.conflicts,
    })
}

pub fn apply_reconcile_choices(
    source: &dyn ObjectSource,
    working_root: ObjectId,
    branch_root: ObjectId,
    layer_root: ObjectId,
    conflicts: &[ReconcileConflict],
    choices: &[filesystem::ReconcileChoice],
) -> Result<BuiltRoot> {
    if conflicts.len() != choices.len() {
        return Err(StoreError::InvalidInput("reconciliation choice count"));
    }
    let mut objects = ObjectBuffer::new(source)?;
    let mut root = working_root;
    for (conflict, choice) in conflicts.iter().zip(choices) {
        let selected_root = match choice {
            filesystem::ReconcileChoice::Branch => branch_root,
            filesystem::ReconcileChoice::Layer => layer_root,
            filesystem::ReconcileChoice::WorkingTree => continue,
        };
        root = filesystem::replace_conflict_from_snapshot(
            &mut objects,
            root,
            selected_root,
            conflict,
        )?;
    }
    objects.finish(root, 0)
}

pub struct DeferredObjectStore {
    storage: DeferredObjects,
    reachable: IdOrder,
    references: Option<BTreeMap<ObjectId, Vec<ObjectId>>>,
    reference_bytes: usize,
    count: u64,
    encoded_bytes: u64,
    first_store_write_bytes: u64,
    spill_peak_bytes: u64,
    spill_count: u64,
    memory_limit: usize,
    index_limit: usize,
    spill_buffer_bytes: usize,
    order_memory_bytes: usize,
    diagnostic_file_context: bool,
    predecessor: Option<(
        crate::SnapshotReader,
        layerfs_content::file::rope::FileStateRoot,
        std::sync::Arc<std::sync::atomic::AtomicU64>,
        bool,
    )>,
}

#[cfg(test)]
pub(crate) struct AppendOnlyInitializationWriter {
    writer: std::fs::File,
    reader: std::fs::File,
    path: TempPath,
    pending: Vec<u8>,
    pending_limit: usize,
    end: u64,
    objects: u64,
    bytes: u64,
    write_calls: u64,
    write_bytes: u64,
    get_calls: Cell<u64>,
}

#[cfg(test)]
pub(crate) struct AppendOnlyInitializationSegment {
    reader: Option<BufReader<CountedFile>>,
    unbuffered_reader: Option<CountedFile>,
    reader_capacity: usize,
    _path: TempPath,
    cursor: u64,
    end: u64,
    objects: u64,
    bytes: u64,
    read_objects: u64,
    read_bytes: u64,
    write_calls: u64,
    write_bytes: u64,
    get_calls: u64,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InitializationTaskBlock {
    pub task_ordinal: usize,
    pub worker_index: usize,
    pub start: u64,
    pub end: u64,
    pub object_count: u64,
    pub byte_count: u64,
}

pub(crate) struct CompactInodePairWriter {
    writer: std::fs::File,
    reader: std::fs::File,
    path: TempPath,
    pending: Vec<u8>,
    pending_limit: usize,
    end: u64,
    pairs: u64,
    write_calls: u64,
    write_bytes: u64,
}

pub(crate) struct CompactInodePairSegment {
    reader: BufReader<CountedFile>,
    _path: TempPath,
    cursor: u64,
    end: u64,
    pairs: u64,
    read_pairs: u64,
    write_calls: u64,
    write_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CompactInodePairBlock {
    pub task_ordinal: usize,
    pub worker_index: usize,
    pub start: u64,
    pub end: u64,
    pub pair_count: u64,
}

pub(crate) struct CompactInodePairStream {
    segments: Vec<CompactInodePairSegment>,
    blocks: std::vec::IntoIter<CompactInodePairBlock>,
    current: Option<(CompactInodePairBlock, u64)>,
    last_task: Option<usize>,
    done: bool,
}

struct CountedFile {
    file: std::fs::File,
    reads: u64,
    bytes: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct InitializationSegmentIoMetrics {
    pub frames: u64,
    pub payload_bytes: u64,
    pub framing_bytes: u64,
    pub write_calls: u64,
    pub write_bytes: u64,
    pub raw_read_calls: u64,
    pub raw_read_bytes: u64,
    pub passes: u64,
}

impl Read for CountedFile {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.reads = self.reads.saturating_add(1);
        let bytes = self.file.read(buffer)?;
        self.bytes = self.bytes.saturating_add(bytes as u64);
        Ok(bytes)
    }
}

enum DeferredObjects {
    Memory {
        order: Vec<ObjectId>,
        rows: BTreeMap<ObjectId, AuthenticatedCanonicalObject>,
        bytes: usize,
    },
    Spill(SpillObjects),
}

#[cfg(test)]
impl AppendOnlyInitializationWriter {
    pub(crate) fn new(pending_limit: usize) -> Result<Self> {
        if pending_limit == 0 {
            return Err(StoreError::InvalidInput("initialization segment buffer"));
        }
        let (writer, path) = temporary_file("initialization-segment")?;
        let reader = std::fs::File::open(&path)?;
        Ok(Self {
            writer,
            reader,
            path: TempPath(path),
            pending: Vec::with_capacity(pending_limit),
            pending_limit,
            end: 0,
            objects: 0,
            bytes: 0,
            write_calls: 0,
            write_bytes: 0,
            get_calls: Cell::new(0),
        })
    }

    pub(crate) fn checkpoint(&self) -> (u64, u64, u64) {
        (self.end, self.objects, self.bytes)
    }

    pub(crate) fn block_since(
        &self,
        task_ordinal: usize,
        worker_index: usize,
        checkpoint: (u64, u64, u64),
    ) -> Result<InitializationTaskBlock> {
        let (start, objects, bytes) = checkpoint;
        Ok(InitializationTaskBlock {
            task_ordinal,
            worker_index,
            start,
            end: self.end,
            object_count: self
                .objects
                .checked_sub(objects)
                .ok_or(StoreError::Integrity("initialization segment objects"))?,
            byte_count: self
                .bytes
                .checked_sub(bytes)
                .ok_or(StoreError::Integrity("initialization segment bytes"))?,
        })
    }

    pub(crate) fn get_calls(&self) -> u64 {
        self.get_calls.get()
    }

    pub(crate) fn seal(mut self) -> Result<AppendOnlyInitializationSegment> {
        self.flush()?;
        if self.reader.metadata()?.len() != self.end {
            return Err(StoreError::Integrity("initialization segment length"));
        }
        let Self {
            writer,
            mut reader,
            path,
            end,
            objects,
            bytes,
            write_calls,
            write_bytes,
            get_calls,
            pending_limit,
            ..
        } = self;
        drop(writer);
        reader.seek(SeekFrom::Start(0))?;
        #[cfg(unix)]
        std::fs::remove_file(&path.0)?;
        Ok(AppendOnlyInitializationSegment {
            reader: None,
            unbuffered_reader: Some(CountedFile {
                file: reader,
                reads: 0,
                bytes: 0,
            }),
            reader_capacity: pending_limit,
            _path: path,
            cursor: 0,
            end,
            objects,
            bytes,
            read_objects: 0,
            read_bytes: 0,
            write_calls,
            write_bytes,
            get_calls: get_calls.get(),
        })
    }

    fn append(&mut self, id: ObjectId, canonical: &[u8]) -> Result<()> {
        let row_len = canonical
            .len()
            .checked_add(40)
            .ok_or(StoreError::Integrity("initialization segment length"))?;
        if !self.pending.is_empty()
            && self.pending.len().saturating_add(row_len) > self.pending_limit
        {
            self.flush()?;
        }
        self.pending.extend_from_slice(id.as_bytes());
        self.pending
            .extend_from_slice(&(canonical.len() as u64).to_le_bytes());
        self.pending.extend_from_slice(canonical);
        self.end = self
            .end
            .checked_add(row_len as u64)
            .ok_or(StoreError::Integrity("initialization segment length"))?;
        self.objects = self
            .objects
            .checked_add(1)
            .ok_or(StoreError::Integrity("initialization segment objects"))?;
        self.bytes = self
            .bytes
            .checked_add(canonical.len() as u64)
            .ok_or(StoreError::Integrity("initialization segment bytes"))?;
        if self.pending.len() >= self.pending_limit {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if !self.pending.is_empty() {
            self.writer.write_all(&self.pending)?;
            self.write_calls = self.write_calls.saturating_add(1);
            self.write_bytes = self.write_bytes.saturating_add(self.pending.len() as u64);
            self.pending.clear();
        }
        Ok(())
    }
}

#[cfg(test)]
impl ObjectStore for AppendOnlyInitializationWriter {
    fn get(&self, _id: ObjectId) -> CoreResult<Vec<u8>> {
        self.get_calls.set(self.get_calls.get().saturating_add(1));
        Err(CoreError::InvalidRecord("append-only initialization get"))
    }

    fn put(&mut self, canonical: &[u8]) -> CoreResult<ObjectId> {
        let id = ObjectId::for_bytes(canonical);
        self.append(id, canonical).map_err(|_| CoreError::Io)?;
        Ok(id)
    }
}

#[cfg(test)]
impl AppendOnlyInitializationSegment {
    fn reader(&mut self) -> Result<&mut BufReader<CountedFile>> {
        if self.reader.is_none() {
            let reader = self
                .unbuffered_reader
                .take()
                .ok_or(StoreError::Integrity("initialization segment reader"))?;
            self.reader = Some(BufReader::with_capacity(self.reader_capacity, reader));
        }
        self.reader
            .as_mut()
            .ok_or(StoreError::Integrity("initialization segment reader"))
    }

    pub(crate) fn consume_block(
        &mut self,
        block: InitializationTaskBlock,
        mut visitor: impl FnMut(CanonicalObject) -> Result<()>,
    ) -> Result<()> {
        if block.start != self.cursor || block.end > self.end || block.start > block.end {
            return Err(StoreError::Integrity("initialization segment block order"));
        }
        let before_objects = self.read_objects;
        let before_bytes = self.read_bytes;
        while self.cursor < block.end {
            let mut id = [0; 32];
            self.reader()?.read_exact(&mut id)?;
            let id = ObjectId::from_bytes(&id)?;
            let mut length = [0; 8];
            self.reader()?.read_exact(&mut length)?;
            let length = usize::try_from(u64::from_le_bytes(length))
                .map_err(|_| StoreError::Integrity("initialization segment object length"))?;
            let next = self
                .cursor
                .checked_add(40)
                .and_then(|cursor| cursor.checked_add(length as u64))
                .ok_or(StoreError::Integrity("initialization segment length"))?;
            if next > block.end {
                return Err(StoreError::Integrity("initialization segment block length"));
            }
            let mut bytes = vec![0; length];
            self.reader()?.read_exact(&mut bytes)?;
            self.cursor = next;
            self.read_objects += 1;
            self.read_bytes = self.read_bytes.saturating_add(length as u64);
            visitor(CanonicalObject { id, bytes })?;
        }
        if self.cursor != block.end
            || self.read_objects - before_objects != block.object_count
            || self.read_bytes - before_bytes != block.byte_count
        {
            return Err(StoreError::Integrity("initialization segment block"));
        }
        Ok(())
    }

    pub(crate) fn finish_consumption(self) -> Result<InitializationSegmentIoMetrics> {
        if self.end == 0 {
            let (raw_read_calls, raw_read_bytes) = if let Some(reader) = &self.reader {
                (reader.get_ref().reads, reader.get_ref().bytes)
            } else if let Some(reader) = &self.unbuffered_reader {
                (reader.reads, reader.bytes)
            } else {
                return Err(StoreError::Integrity("initialization segment reader"));
            };
            if self.cursor != 0
                || self.objects != 0
                || self.bytes != 0
                || self.read_objects != 0
                || self.read_bytes != 0
                || self.write_calls != 0
                || self.write_bytes != 0
                || raw_read_calls != 0
                || raw_read_bytes != 0
                || self.get_calls != 0
            {
                return Err(StoreError::Integrity("initialization segment consumption"));
            }
            return Ok(InitializationSegmentIoMetrics::default());
        }
        let reader = self
            .reader
            .ok_or(StoreError::Integrity("initialization segment reader"))?;
        if self.cursor != self.end
            || self.read_objects != self.objects
            || self.read_bytes != self.bytes
            || self.end != self.bytes.saturating_add(self.objects.saturating_mul(40))
            || self.write_bytes != self.end
            || reader.get_ref().bytes != self.end
            || self.get_calls != 0
        {
            return Err(StoreError::Integrity("initialization segment consumption"));
        }
        Ok(InitializationSegmentIoMetrics {
            frames: self.objects,
            payload_bytes: self.bytes,
            framing_bytes: self.objects.saturating_mul(40),
            write_calls: self.write_calls,
            write_bytes: self.write_bytes,
            raw_read_calls: reader.get_ref().reads,
            raw_read_bytes: reader.get_ref().bytes,
            passes: 1,
        })
    }

    #[cfg(test)]
    fn path(&self) -> &std::path::Path {
        &self._path.0
    }

    #[cfg(test)]
    pub(crate) fn reader_capacity(&self) -> usize {
        self.reader
            .as_ref()
            .map_or(self.reader_capacity, BufReader::capacity)
    }

    #[cfg(test)]
    fn raw_reads(&self) -> u64 {
        self.reader
            .as_ref()
            .map_or(0, |reader| reader.get_ref().reads)
    }

    #[cfg(test)]
    fn raw_read_bytes(&self) -> u64 {
        self.reader
            .as_ref()
            .map_or(0, |reader| reader.get_ref().bytes)
    }
}

impl CompactInodePairWriter {
    pub(crate) fn new(pending_limit: usize) -> Result<Self> {
        if pending_limit < 64 {
            return Err(StoreError::InvalidInput("inode pair segment buffer"));
        }
        let (writer, path) = temporary_file("initialization-inode-pairs")?;
        let reader = std::fs::File::open(&path)?;
        Ok(Self {
            writer,
            reader,
            path: TempPath(path),
            pending: Vec::with_capacity(pending_limit),
            pending_limit,
            end: 0,
            pairs: 0,
            write_calls: 0,
            write_bytes: 0,
        })
    }

    pub(crate) fn checkpoint(&self) -> (u64, u64) {
        (self.end, self.pairs)
    }

    pub(crate) fn push(
        &mut self,
        inode: layerfs_content::tree::inode::InodeId,
        record: ObjectId,
    ) -> Result<()> {
        if !self.pending.is_empty() && self.pending.len() + 64 > self.pending_limit {
            self.flush()?;
        }
        self.pending.extend_from_slice(inode.as_bytes());
        self.pending.extend_from_slice(record.as_bytes());
        self.end = self
            .end
            .checked_add(64)
            .ok_or(StoreError::Integrity("inode pair segment length"))?;
        self.pairs = self
            .pairs
            .checked_add(1)
            .ok_or(StoreError::Integrity("inode pair segment count"))?;
        if self.pending.len() >= self.pending_limit {
            self.flush()?;
        }
        Ok(())
    }

    pub(crate) fn block_since(
        &self,
        task_ordinal: usize,
        worker_index: usize,
        checkpoint: (u64, u64),
    ) -> Result<CompactInodePairBlock> {
        let (start, pairs) = checkpoint;
        Ok(CompactInodePairBlock {
            task_ordinal,
            worker_index,
            start,
            end: self.end,
            pair_count: self
                .pairs
                .checked_sub(pairs)
                .ok_or(StoreError::Integrity("inode pair segment count"))?,
        })
    }

    pub(crate) fn seal(mut self) -> Result<CompactInodePairSegment> {
        self.flush()?;
        if self.reader.metadata()?.len() != self.end {
            return Err(StoreError::Integrity("inode pair segment length"));
        }
        let Self {
            writer,
            mut reader,
            path,
            end,
            pairs,
            write_calls,
            write_bytes,
            pending_limit,
            ..
        } = self;
        drop(writer);
        reader.seek(SeekFrom::Start(0))?;
        #[cfg(unix)]
        std::fs::remove_file(&path.0)?;
        Ok(CompactInodePairSegment {
            reader: BufReader::with_capacity(
                pending_limit,
                CountedFile {
                    file: reader,
                    reads: 0,
                    bytes: 0,
                },
            ),
            _path: path,
            cursor: 0,
            end,
            pairs,
            read_pairs: 0,
            write_calls,
            write_bytes,
        })
    }

    #[cfg(test)]
    pub(crate) fn pending_capacity(&self) -> usize {
        self.pending.capacity()
    }

    fn flush(&mut self) -> Result<()> {
        if !self.pending.is_empty() {
            self.writer.write_all(&self.pending)?;
            self.write_calls = self.write_calls.saturating_add(1);
            self.write_bytes = self.write_bytes.saturating_add(self.pending.len() as u64);
            self.pending.clear();
        }
        Ok(())
    }
}

impl CompactInodePairSegment {
    fn read_pair(
        &mut self,
        block_end: u64,
    ) -> Result<(layerfs_content::tree::inode::InodeId, ObjectId)> {
        let next = self
            .cursor
            .checked_add(64)
            .ok_or(StoreError::Integrity("inode pair segment length"))?;
        if next > block_end {
            return Err(StoreError::Integrity("inode pair block length"));
        }
        let mut pair = [0; 64];
        self.reader.read_exact(&mut pair)?;
        self.cursor = next;
        self.read_pairs += 1;
        Ok((
            layerfs_content::tree::inode::InodeId::from_slice(&pair[..32])?,
            ObjectId::from_bytes(&pair[32..])?,
        ))
    }

    fn consumed(&self) -> bool {
        self.cursor == self.end && self.read_pairs == self.pairs
    }

    #[cfg(test)]
    fn path(&self) -> &std::path::Path {
        &self._path.0
    }

    #[cfg(test)]
    pub(crate) fn reader_capacity(&self) -> usize {
        self.reader.capacity()
    }

    #[cfg(test)]
    fn raw_reads(&self) -> u64 {
        self.reader.get_ref().reads
    }

    #[cfg(test)]
    fn raw_read_bytes(&self) -> u64 {
        self.reader.get_ref().bytes
    }
}

impl CompactInodePairStream {
    pub(crate) fn new(
        segments: Vec<CompactInodePairSegment>,
        blocks: Vec<CompactInodePairBlock>,
    ) -> Result<Self> {
        if blocks.len() > 1_000
            || blocks.iter().enumerate().any(|(task, block)| {
                block.task_ordinal != task
                    || block.worker_index >= segments.len()
                    || block.start > block.end
                    || block.pair_count.checked_mul(64) != Some(block.end - block.start)
            })
        {
            return Err(StoreError::Integrity("inode pair block order"));
        }
        Ok(Self {
            segments,
            blocks: blocks.into_iter(),
            current: None,
            last_task: None,
            done: false,
        })
    }

    fn fail(
        &mut self,
        error: StoreError,
    ) -> Option<CoreResult<(layerfs_content::tree::inode::InodeId, ObjectId)>> {
        self.done = true;
        Some(Err(core_read_error(error)))
    }

    pub(crate) fn finish(self) -> Result<InitializationSegmentIoMetrics> {
        if !self.done
            || self.current.is_some()
            || self.blocks.len() != 0
            || !self.segments.iter().all(CompactInodePairSegment::consumed)
        {
            return Err(StoreError::Integrity("inode pair segment consumption"));
        }
        let mut metrics = InitializationSegmentIoMetrics::default();
        for segment in self.segments {
            if segment.end != segment.pairs.saturating_mul(64)
                || segment.write_bytes != segment.end
                || segment.reader.get_ref().bytes != segment.end
            {
                return Err(StoreError::Integrity("inode pair segment consumption"));
            }
            metrics.frames = metrics.frames.saturating_add(segment.pairs);
            metrics.payload_bytes = metrics.payload_bytes.saturating_add(segment.end);
            metrics.write_calls = metrics.write_calls.saturating_add(segment.write_calls);
            metrics.write_bytes = metrics.write_bytes.saturating_add(segment.write_bytes);
            metrics.raw_read_calls = metrics
                .raw_read_calls
                .saturating_add(segment.reader.get_ref().reads);
            metrics.raw_read_bytes = metrics
                .raw_read_bytes
                .saturating_add(segment.reader.get_ref().bytes);
            metrics.passes = metrics.passes.saturating_add(1);
        }
        Ok(metrics)
    }
}

impl Iterator for CompactInodePairStream {
    type Item = CoreResult<(layerfs_content::tree::inode::InodeId, ObjectId)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        loop {
            if let Some((block, read)) = self.current {
                let segment = match self.segments.get_mut(block.worker_index) {
                    Some(segment) => segment,
                    None => {
                        return self.fail(StoreError::Integrity("inode pair segment worker"));
                    }
                };
                if segment.cursor == block.end {
                    if read != block.pair_count {
                        return self.fail(StoreError::Integrity("inode pair block count"));
                    }
                    self.last_task = Some(block.task_ordinal);
                    self.current = None;
                    continue;
                }
                match segment.read_pair(block.end) {
                    Ok(pair) => {
                        self.current = Some((block, read + 1));
                        return Some(Ok(pair));
                    }
                    Err(error) => return self.fail(error),
                }
            }

            let Some(block) = self.blocks.next() else {
                self.done = true;
                if self.segments.iter().all(CompactInodePairSegment::consumed) {
                    return None;
                }
                return Some(Err(CoreError::InvalidRecord(
                    "inode pair segment consumption",
                )));
            };
            if self
                .last_task
                .is_some_and(|task| block.task_ordinal != task + 1)
                || self
                    .segments
                    .get(block.worker_index)
                    .is_none_or(|segment| segment.cursor != block.start)
            {
                return self.fail(StoreError::Integrity("inode pair block order"));
            }
            self.current = Some((block, 0));
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ObjectInsertMetrics {
    pub payload_ns: u64,
    pub insert_ns: u64,
    pub objects: u64,
    pub bytes: u64,
    pub submitted_rows: u64,
    pub returned_ids: u64,
    pub skipped_ids: u64,
    pub skipped_bytes: u64,
    pub collision_checks: u64,
    pub sql_string_build_ns: u64,
    pub sql_prepare_ns: u64,
    pub sql_bind_step_returning_ns: u64,
    pub conflict_read_calls: u64,
    pub conflict_read_rows: u64,
    pub conflict_read_bytes: u64,
    pub conflict_read_ns: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct InitializationAdmissionDiagnostics {
    pub pending_duplicate_objects: u64,
    pub pending_duplicate_bytes: u64,
    pub cross_batch_skipped_objects: u64,
    pub cross_batch_skipped_bytes: u64,
    pub collision_checks: u64,
    pub batch_peak_objects: u64,
    pub batch_peak_payload_bytes: u64,
    pub batch_peak_vec_capacity: u64,
    pub pending_index_peak_entries: u64,
    pub pending_index_peak_bytes: u64,
    pub final_batch_peak_payload_bytes: u64,
    pub final_batch_peak_vec_capacity: u64,
    pub final_pending_index_peak_bytes: u64,
    pub final_simultaneous_owned_peak_bytes: u64,
    pub sql_batch_count: u64,
    pub sql_row_shapes: BTreeSet<u64>,
    pub sql_submitted_rows: u64,
    pub sql_returned_ids: u64,
    pub sql_skipped_ids: u64,
    pub sql_string_build_ns: u64,
    pub sql_prepare_ns: u64,
    pub sql_bind_step_returning_ns: u64,
    pub conflict_read_calls: u64,
    pub conflict_read_rows: u64,
    pub conflict_read_bytes: u64,
    pub conflict_read_ns: u64,
    pub sql_begin_ns: u64,
    pub sql_commit_ns: u64,
    pub pipeline_commit_count: u64,
    pub pipeline_commit_ns: u64,
    pub pipeline_commit_max_ns: u64,
    pub pipeline_commit_max_ordinal: u64,
    pub final_build_commit_count: u64,
    pub final_build_commit_ns: u64,
    pub final_build_commit_max_ns: u64,
    pub final_build_commit_max_ordinal: u64,
    pub publication_commit_ns: u64,
}

#[derive(Clone, Copy)]
pub(crate) enum InitializationSqlPhase {
    Pipeline,
    FinalBuild,
    Publication,
}

impl InitializationAdmissionDiagnostics {
    pub(crate) fn record_sql_batch(
        &mut self,
        metrics: ObjectInsertMetrics,
        begin_ns: u64,
        commit_ns: u64,
        phase: InitializationSqlPhase,
    ) {
        if metrics.submitted_rows != 0 {
            self.sql_batch_count += 1;
            self.sql_row_shapes.insert(metrics.submitted_rows);
        }
        self.cross_batch_skipped_objects = self
            .cross_batch_skipped_objects
            .saturating_add(metrics.skipped_ids);
        self.cross_batch_skipped_bytes = self
            .cross_batch_skipped_bytes
            .saturating_add(metrics.skipped_bytes);
        self.collision_checks = self
            .collision_checks
            .saturating_add(metrics.collision_checks);
        self.sql_submitted_rows = self
            .sql_submitted_rows
            .saturating_add(metrics.submitted_rows);
        self.sql_returned_ids = self.sql_returned_ids.saturating_add(metrics.returned_ids);
        self.sql_skipped_ids = self.sql_skipped_ids.saturating_add(metrics.skipped_ids);
        self.sql_string_build_ns = self
            .sql_string_build_ns
            .saturating_add(metrics.sql_string_build_ns);
        self.sql_prepare_ns = self.sql_prepare_ns.saturating_add(metrics.sql_prepare_ns);
        self.sql_bind_step_returning_ns = self
            .sql_bind_step_returning_ns
            .saturating_add(metrics.sql_bind_step_returning_ns);
        self.conflict_read_calls = self
            .conflict_read_calls
            .saturating_add(metrics.conflict_read_calls);
        self.conflict_read_rows = self
            .conflict_read_rows
            .saturating_add(metrics.conflict_read_rows);
        self.conflict_read_bytes = self
            .conflict_read_bytes
            .saturating_add(metrics.conflict_read_bytes);
        self.conflict_read_ns = self
            .conflict_read_ns
            .saturating_add(metrics.conflict_read_ns);
        self.sql_begin_ns = self.sql_begin_ns.saturating_add(begin_ns);
        self.sql_commit_ns = self.sql_commit_ns.saturating_add(commit_ns);
        let ordinal = self.sql_batch_count;
        match phase {
            InitializationSqlPhase::Pipeline => {
                self.pipeline_commit_count += 1;
                self.pipeline_commit_ns = self.pipeline_commit_ns.saturating_add(commit_ns);
                if commit_ns > self.pipeline_commit_max_ns {
                    self.pipeline_commit_max_ns = commit_ns;
                    self.pipeline_commit_max_ordinal = ordinal;
                }
            }
            InitializationSqlPhase::FinalBuild => {
                self.final_build_commit_count += 1;
                self.final_build_commit_ns = self.final_build_commit_ns.saturating_add(commit_ns);
                if commit_ns > self.final_build_commit_max_ns {
                    self.final_build_commit_max_ns = commit_ns;
                    self.final_build_commit_max_ordinal = ordinal;
                }
            }
            InitializationSqlPhase::Publication => {
                self.publication_commit_ns = commit_ns;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CheckedAdmission {
    pub candidate_objects: u64,
    pub candidate_bytes: u64,
    pub inserted_objects: u64,
    pub inserted_bytes: u64,
    pub reused_objects: u64,
    pub reused_bytes: u64,
    pub transactions: u64,
    pub max_transaction_objects: u64,
    pub max_transaction_bytes: u64,
    pub begin_ns: u64,
    pub insert_ns: u64,
    pub commit_ns: u64,
}

impl CheckedAdmission {
    fn record(&mut self, metrics: &AdmissionBatchMetrics) {
        let bytes = metrics
            .insert
            .bytes
            .saturating_add(metrics.insert.skipped_bytes);
        self.transactions += 1;
        self.max_transaction_objects = self
            .max_transaction_objects
            .max(metrics.insert.submitted_rows);
        self.max_transaction_bytes = self.max_transaction_bytes.max(bytes);
        self.begin_ns = self.begin_ns.saturating_add(metrics.begin_ns);
        self.insert_ns = self.insert_ns.saturating_add(metrics.insert.insert_ns);
        self.commit_ns = self.commit_ns.saturating_add(metrics.commit_ns);
        self.candidate_objects += metrics.insert.submitted_rows;
        self.candidate_bytes = self.candidate_bytes.saturating_add(bytes);
        self.inserted_objects += metrics.insert.objects;
        self.inserted_bytes = self.inserted_bytes.saturating_add(metrics.insert.bytes);
        self.reused_objects += metrics.insert.skipped_ids;
        self.reused_bytes = self
            .reused_bytes
            .saturating_add(metrics.insert.skipped_bytes);
    }
}

pub(crate) struct AdmissionBatchMetrics {
    pub(crate) insert: ObjectInsertMetrics,
    pub(crate) begin_ns: u64,
    pub(crate) commit_ns: u64,
}

pub(crate) struct CheckedOutputAdmission {
    db: crate::schema::StoreDb,
    incoming: Vec<AuthenticatedCanonicalObject>,
    incoming_index: HashMap<ObjectId, usize>,
    incoming_bytes: usize,
    batch: Vec<AuthenticatedCanonicalObject>,
    available: SpillableObjectSet,
    seen: SpillableObjectSet,
    pending: HashMap<ObjectId, usize>,
    batch_bytes: usize,
    validation_reserve: usize,
    statement_number: u64,
    receipt: crate::CandidateReceipt,
    checked: CheckedAdmission,
    diagnostics: InitializationAdmissionDiagnostics,
    final_phase: bool,
}

/// Owned initially missing canonical output with closed external dependencies.
/// Only the accumulator creates this handoff; final publishers cannot reprobe it.
pub(crate) struct MissingBatch(Vec<AuthenticatedCanonicalObject>);

impl std::ops::Deref for MissingBatch {
    type Target = [AuthenticatedCanonicalObject];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct FinishedOutputAdmission {
    pub final_batch: MissingBatch,
    pub statement_number: u64,
    pub receipt: crate::CandidateReceipt,
    pub checked: CheckedAdmission,
    pub diagnostics: InitializationAdmissionDiagnostics,
}

impl DeferredObjectStore {
    pub fn new() -> Result<Self> {
        Self::with_reference_index(true)
    }

    pub(crate) fn new_all_reachable() -> Result<Self> {
        Self::with_reference_index(false)
    }

    fn with_reference_index(reference_index: bool) -> Result<Self> {
        Ok(Self {
            storage: DeferredObjects::Memory {
                order: Vec::new(),
                rows: BTreeMap::new(),
                bytes: 0,
            },
            reachable: IdOrder::empty(),
            references: reference_index.then(BTreeMap::new),
            reference_bytes: 0,
            count: 0,
            encoded_bytes: 0,
            first_store_write_bytes: 0,
            spill_peak_bytes: 0,
            spill_count: 0,
            memory_limit: CANDIDATE_MEMORY_BYTES - 2 * 1024 * 1024,
            index_limit: CANDIDATE_INDEX_BYTES,
            spill_buffer_bytes: CANDIDATE_SPILL_BUFFER_BYTES - 2 * spill::ID_BUFFER_BYTES,
            order_memory_bytes: CANDIDATE_MEMORY_BYTES,
            diagnostic_file_context: false,
            predecessor: None,
        })
    }

    pub fn len(&self) -> u64 {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    pub fn has_reference_index(&self) -> bool {
        self.references.is_some()
    }

    pub fn cached_references(&self, id: ObjectId) -> Option<Vec<ObjectId>> {
        self.references
            .as_ref()
            .and_then(|references| references.get(&id).cloned())
    }

    pub fn ids_in_order(&self, limit: usize) -> Result<Option<Vec<ObjectId>>> {
        if self.count > limit as u64 {
            return Ok(None);
        }
        let mut ids = Vec::with_capacity(self.count as usize);
        self.reachable.visit(|id| {
            ids.push(id);
            Ok(())
        })?;
        Ok(Some(ids))
    }

    fn order_missing(&self, missing: &SpillableObjectSet, expected: usize) -> Result<IdOrder> {
        let mut output = IdOrder::empty();
        let mut count = 0_usize;
        let mut page = Vec::with_capacity(OBJECT_PAGE_COUNT);
        let mut consume = |page: &[ObjectId]| -> Result<()> {
            let known = missing.membership(page)?;
            for &id in page {
                if known.contains(&id) {
                    output.push_bounded(id, self.index_limit)?;
                    count += 1;
                }
            }
            Ok(())
        };
        let mut push = |id| -> Result<()> {
            page.push(id);
            if page.len() == OBJECT_PAGE_COUNT {
                consume(&page)?;
                page.clear();
            }
            Ok(())
        };
        match &self.storage {
            DeferredObjects::Memory { order, .. } => {
                for &id in order {
                    push(id)?;
                }
            }
            DeferredObjects::Spill(spill) => spill.visit_ids(&mut push)?,
        }
        if !page.is_empty() {
            consume(&page)?;
        }
        if count != expected {
            return Err(StoreError::Integrity("candidate publication order"));
        }
        output.seal()?;
        Ok(output)
    }

    fn visit_prevalidated_order(
        &self,
        order: &IdOrder,
        visitor: &mut dyn FnMut(ObjectId, &[u8]) -> Result<()>,
    ) -> Result<()> {
        match &self.storage {
            DeferredObjects::Memory { rows, .. } => order.visit(|id| {
                visitor(
                    id,
                    &rows.get(&id).ok_or(StoreError::MissingObject(id))?.bytes,
                )
            }),
            DeferredObjects::Spill(spill) => {
                spill.visit_ordered(order, &mut |id, bytes, _| visitor(id, bytes))
            }
        }
    }

    fn visit_authenticated_order(
        &self,
        order: &IdOrder,
        visitor: &mut dyn FnMut(&AuthenticatedCanonicalObject) -> Result<()>,
    ) -> Result<()> {
        match &self.storage {
            DeferredObjects::Memory { rows, .. } => {
                order.visit(|id| visitor(rows.get(&id).ok_or(StoreError::MissingObject(id))?))
            }
            DeferredObjects::Spill(spill) => spill.visit_ordered(order, &mut |id, bytes, hints| {
                let mut checked =
                    AuthenticatedCanonicalObject::new(std::mem::take(bytes), Some(id))?;
                checked.1 = hints;
                let result = visitor(&checked);
                *bytes = checked.0.bytes;
                result
            }),
        }
    }

    // Returns required selected-spill authentication time; memory owners retain proof.
    fn consume_prevalidated_pages(
        mut self,
        mut visitor: impl FnMut(Vec<AuthenticatedCanonicalObject>) -> Result<()>,
    ) -> Result<u64> {
        self.reachable.seal()?;
        // Preserve the checked graph's child-first order through spill delivery;
        // each bounded admission can carry closed dependency facts forward.
        let mut memory_owned_bytes = 0_u64;
        let mut spill_readback_bytes = 0_u64;
        let mut storage_authentication_ns = 0_u64;
        let capacity = usize::try_from(self.count)
            .unwrap_or(INITIALIZATION_ADMISSION_BATCH_COUNT)
            .min(INITIALIZATION_SLAB_OBJECTS)
            .min((self.memory_limit / std::mem::size_of::<AuthenticatedCanonicalObject>()).max(1));
        let page_limit = self.memory_limit.min(INITIALIZATION_SLAB_BYTES);
        let mut page = Vec::with_capacity(capacity);
        let mut page_bytes = 0_usize;
        let mut predecessor = self
            .predecessor
            .take()
            .map(|(reader, root, budget, available)| {
                (
                    reader,
                    layerfs_content::file::rope::PredecessorCursor::new(root),
                    budget,
                    available,
                )
            });
        let mut file_reserved = 0u64;
        let mut diagnostic_stop = 0u8;
        let mut diagnostic_stats = crate::PhysicalStorageReceipt::default();
        diagnostic_stats.diag_cursor_attached = u64::from(predecessor.is_some());
        let mut push = |mut object: AuthenticatedCanonicalObject| {
            object.1.has_predecessor |= predecessor.is_some();
            if let (Some((reader, cursor, operation_reserved, available)), Some((start, len))) =
                (&mut predecessor, object.1.first_span)
            {
                // Each optional metadata object can require a target group and a FULL
                // anchor group. Reserve the larger encoded bound for both counters.
                const FETCH_RESERVATION: u64 = 131136;
                diagnostic_stats.diag_invalid +=
                    u64::from(object.1.diagnostic & 15 != 0 || object.1.diagnostic_grants != 0);
                let inherited = cursor.counters().1 != 0;
                let reserved_before = file_reserved;
                let mut denied = 0;
                diagnostic_stats.diag_cursor_queries += 1;
                diagnostic_stats.diag_cursor_query_bytes += object.bytes.len() as u64;
                diagnostic_stats.diag_cursor_inherited += u64::from(inherited);
                object.1.prior_ids = cursor.hints(&CoreReader(reader), start, len, || {
                    if !*available {
                        denied = diagnostic::MEMORY;
                        return false;
                    }
                    if file_reserved + FETCH_RESERVATION > 1024 * 1024 {
                        denied = diagnostic::FILE_LIMIT;
                        return false;
                    }
                    if operation_reserved
                        .fetch_update(
                            std::sync::atomic::Ordering::Relaxed,
                            std::sync::atomic::Ordering::Relaxed,
                            |used| {
                                used.checked_add(FETCH_RESERVATION)
                                    .filter(|sum| *sum <= CORRESPONDENCE_OPERATION_RESERVATION_BYTES)
                            },
                        )
                        .is_err()
                    {
                        denied = diagnostic::OPERATION;
                        return false;
                    }
                    file_reserved += FETCH_RESERVATION;
                    true
                })?;
                let grants = (file_reserved - reserved_before) / FETCH_RESERVATION;
                diagnostic_stats.diag_cursor_grants += grants;
                object.1.diagnostic_grants = grants as u8;
                if !inherited && cursor.counters().1 != 0 {
                    diagnostic_stop = if denied != 0 {
                        denied
                    } else {
                        diagnostic::DESCRIPTOR
                    };
                    match diagnostic_stop {
                        diagnostic::MEMORY => diagnostic_stats.diag_cursor_memory_limit += 1,
                        diagnostic::FILE_LIMIT => diagnostic_stats.diag_cursor_file_limit += 1,
                        diagnostic::OPERATION => diagnostic_stats.diag_cursor_operation_limit += 1,
                        _ => diagnostic_stats.diag_cursor_descriptor_limit += 1,
                    }
                }
                object.1.diagnostic = (object.1.diagnostic & diagnostic::FILE)
                    | if cursor.counters().1 != 0 {
                        diagnostic_stop
                    } else {
                        diagnostic::COMPLETE
                    }
                    | if inherited { diagnostic::INHERITED } else { 0 };
                diagnostic_stats.diag_invalid += u64::from(!diagnostic::valid(
                    object.1.diagnostic,
                    object.1.diagnostic_grants,
                ));
            }

            if object.bytes.len() > ADMISSION_BATCH_BYTES {
                return Err(StoreError::Integrity("canonical object admission size"));
            }
            if !page.is_empty()
                && (page.len() == INITIALIZATION_ADMISSION_BATCH_COUNT
                    || page_bytes.saturating_add(object.bytes.len()) > page_limit)
            {
                visitor(std::mem::replace(&mut page, Vec::with_capacity(capacity)))?;
                page_bytes = 0;
            }
            page_bytes = page_bytes.saturating_add(object.bytes.len());
            page.push(object);
            Ok(())
        };
        let delivery_result: Result<()> = match &mut self.storage {
            DeferredObjects::Memory { rows, .. } => self.reachable.visit(|id| {
                let object = rows.remove(&id).ok_or(StoreError::MissingObject(id))?;
                memory_owned_bytes = memory_owned_bytes.saturating_add(object.bytes.len() as u64);
                push(object)
            }),
            DeferredObjects::Spill(spill) => {
                spill.visit_ordered(&self.reachable, &mut |id, bytes, hints| {
                    spill_readback_bytes = spill_readback_bytes.saturating_add(bytes.len() as u64);
                    let started = Instant::now();
                    let mut object =
                        AuthenticatedCanonicalObject::new(std::mem::take(bytes), Some(id))?;
                    object.1 = hints;
                    storage_authentication_ns =
                        storage_authentication_ns.saturating_add(elapsed_ns(started));
                    push(object)
                })
            }
        };
        if let Err(error) = delivery_result {
            if let Some((reader, _, _, _)) = &predecessor {
                diagnostic_stats.diag_invalid += 1;
                // Include a successful reservation whose metadata read failed
                // before its occurrence could reach admission.
                diagnostic_stats.diag_cursor_grants = file_reserved / 131136;
                reader.note_delivery_diagnostic(diagnostic_stats);
            }
            return Err(error);
        }
        if let Some((reader, cursor, _, available)) = predecessor {
            let (descriptors, skips) = cursor.counters();
            reader.note_predecessor_correspondence(file_reserved, descriptors, skips, !available);
            reader.note_delivery_diagnostic(diagnostic_stats);
        }
        if !page.is_empty() {
            visitor(page)?;
        }
        crate::telemetry::note_workspace_candidate_delivery(
            memory_owned_bytes,
            spill_readback_bytes,
            storage_authentication_ns,
        );
        Ok(storage_authentication_ns)
    }

    pub fn visit_batches(
        &self,
        visitor: &mut dyn FnMut(&[CanonicalObject], bool) -> Result<()>,
    ) -> Result<()> {
        let mut batch = Vec::with_capacity(OBJECT_PAGE_COUNT);
        let mut bytes = 0_usize;
        let mut push = |object: CanonicalObject| -> Result<()> {
            if !batch.is_empty()
                && (batch.len() == OBJECT_PAGE_COUNT
                    || bytes + object.bytes.len() > OBJECT_PAGE_BYTES)
            {
                visitor(&batch, false)?;
                batch.clear();
                bytes = 0;
            }
            bytes += object.bytes.len();
            batch.push(object);
            Ok(())
        };
        match &self.storage {
            DeferredObjects::Memory { rows, .. } => self.reachable.visit(|id| {
                push(
                    rows.get(&id)
                        .ok_or(StoreError::MissingObject(id))?
                        .as_ref()
                        .clone(),
                )
            })?,
            DeferredObjects::Spill(spill) => {
                spill.visit_ordered(&self.reachable, &mut |id, bytes, _| {
                    push(CanonicalObject {
                        id,
                        bytes: std::mem::take(bytes),
                    })
                })?
            }
        }
        if !batch.is_empty() {
            visitor(&batch, true)?;
        }
        Ok(())
    }

    fn visit_membership_batches(
        &self,
        mut visitor: impl FnMut(&[(ObjectId, u64)]) -> Result<()>,
    ) -> Result<()> {
        self.reachable.visit_pages(|ids| {
            let lengths = match &self.storage {
                DeferredObjects::Memory { rows, .. } => ids
                    .iter()
                    .map(|id| rows.get(id).map(|object| object.bytes.len() as u64))
                    .collect::<Vec<_>>(),
                DeferredObjects::Spill(spill) => spill
                    .locations(ids)?
                    .into_iter()
                    .map(|location| location.map(|(_, length)| length))
                    .collect(),
            };
            let page = ids
                .iter()
                .copied()
                .zip(lengths)
                .map(|(id, length)| Ok((id, length.ok_or(StoreError::MissingObject(id))?)))
                .collect::<Result<Vec<_>>>()?;
            visitor(&page)
        })
    }

    fn reachable_from(mut self, root: ObjectId) -> Result<Self> {
        if let DeferredObjects::Spill(spill) = &mut self.storage {
            spill.flush()?;
        }
        let mut seen = SpillableObjectSet::bounded(self.index_limit)?;
        seen.insert_page(&[root])?;
        let mut active = BTreeSet::new();
        let mut stack = vec![(root, false)];
        let mut order = self.predecessor.is_none().then(IdOrder::empty);
        let mut count = 0_u64;
        let mut encoded_bytes = 0_u64;
        while let Some((id, expanded)) = stack.pop() {
            let length = match self.encoded_length(id) {
                Ok(length) => length,
                Err(StoreError::MissingObject(_)) => continue,
                Err(error) => return Err(error),
            };
            if expanded {
                active.remove(&id);
                if let Some(order) = &mut order {
                    order.push_bounded(id, self.index_limit)?;
                }
                count += 1;
                encoded_bytes = encoded_bytes
                    .checked_add(length)
                    .ok_or(StoreError::Integrity("candidate bytes"))?;
                continue;
            }
            if !active.insert(id) {
                return Err(StoreError::Integrity("object cycle"));
            }
            let children = match self.cached_references(id) {
                Some(children) => children,
                None => {
                    let canonical = self.get(id)?.ok_or(StoreError::MissingObject(id))?;
                    layerfs_content::authenticate_identity(&canonical, id)?;
                    let mut children = referenced_objects(&canonical)?;
                    children.sort();
                    children.dedup();
                    children
                }
            };
            if children.iter().any(|child| active.contains(child)) {
                return Err(StoreError::Integrity("object cycle"));
            }
            let mut inserted = Vec::new();
            for page in children.chunks(OBJECT_PAGE_COUNT) {
                inserted.extend(seen.insert_page(page)?);
            }
            stack.push((id, true));
            stack.extend(inserted.into_iter().rev().map(|child| (child, false)));
        }
        // File constructors emit children before parents. Preserve first payload
        // spans through their existing selection; generic callers retain DFS order.
        self.reachable = if let Some(mut order) = order {
            order.seal()?;
            order
        } else {
            self.order_missing(&seen, count as usize)?
        };
        self.count = count;
        self.encoded_bytes = encoded_bytes;
        if let DeferredObjects::Spill(spill) = &mut self.storage {
            spill.seal()?;
        }
        Ok(self)
    }

    fn retain_references(&mut self, id: ObjectId, children: &[ObjectId]) {
        let Some(references) = self.references.as_mut() else {
            return;
        };
        let charge = 64_usize.saturating_add(children.len().saturating_mul(32));
        if self.reference_bytes.saturating_add(charge) > self.index_limit {
            self.references = None;
            self.reference_bytes = 0;
            return;
        }
        references.insert(id, children.to_vec());
        self.reference_bytes += charge;
    }

    fn get(&self, id: ObjectId) -> Result<Option<Vec<u8>>> {
        match &self.storage {
            DeferredObjects::Memory { rows, .. } => {
                Ok(rows.get(&id).map(|object| object.bytes.clone()))
            }
            DeferredObjects::Spill(spill) => spill.get(id),
        }
    }

    fn encoded_length(&self, id: ObjectId) -> Result<u64> {
        match &self.storage {
            DeferredObjects::Memory { rows, .. } => rows
                .get(&id)
                .map(|object| object.bytes.len() as u64)
                .ok_or(StoreError::MissingObject(id)),
            DeferredObjects::Spill(spill) => spill.encoded_length(id),
        }
    }

    #[cfg(test)]
    fn put(&mut self, id: ObjectId, canonical: &[u8]) -> Result<()> {
        self.put_authenticated(AuthenticatedCanonicalObject::new(
            canonical.to_vec(),
            Some(id),
        )?)
    }

    fn put_authenticated(&mut self, object: AuthenticatedCanonicalObject) -> Result<()> {
        let id = object.id;
        if let Some(known) = self.get(id)? {
            return if known == object.bytes {
                Ok(())
            } else {
                Err(StoreError::Integrity("candidate object collision"))
            };
        }
        let length = object.bytes.len();
        let children = if self.references.is_some() {
            let mut children = referenced_objects(&object.bytes)?;
            children.sort();
            children.dedup();
            Some(children)
        } else {
            None
        };
        // The checked owner retains its identity alongside the existing index key.
        let charge = object
            .bytes
            .capacity()
            .saturating_add(64 + std::mem::size_of::<AuthenticatedCanonicalObject>());
        if matches!(&self.storage, DeferredObjects::Memory { bytes, .. } if bytes.saturating_add(charge) > self.memory_limit)
        {
            self.spill()?;
        }
        match &mut self.storage {
            DeferredObjects::Memory { order, rows, bytes } => {
                order.push(id);
                rows.insert(id, object);
                *bytes += charge;
            }
            DeferredObjects::Spill(spill) => spill.put(&object)?,
        }
        self.reachable.push_bounded(id, self.index_limit)?;
        self.count += 1;
        self.encoded_bytes = self
            .encoded_bytes
            .checked_add(length as u64)
            .ok_or(StoreError::Integrity("candidate bytes"))?;
        self.first_store_write_bytes = self
            .first_store_write_bytes
            .checked_add(length as u64)
            .ok_or(StoreError::Integrity("candidate first-store bytes"))?;
        if matches!(self.storage, DeferredObjects::Spill(_)) {
            self.spill_peak_bytes = self.spill_peak_bytes.max(self.encoded_bytes);
        }
        if let Some(children) = children {
            self.retain_references(id, &children);
        }
        Ok(())
    }

    fn spill(&mut self) -> Result<()> {
        let DeferredObjects::Memory { order, rows, .. } = std::mem::replace(
            &mut self.storage,
            DeferredObjects::Memory {
                order: Vec::new(),
                rows: BTreeMap::new(),
                bytes: 0,
            },
        ) else {
            return Ok(());
        };
        let (file, path) = temporary_file("candidate-objects")?;
        let reader = std::fs::File::open(&path)?;
        let mut spill = SpillObjects {
            writer: Some(file),
            reader: Mutex::new(reader),
            path,
            pending: Vec::with_capacity(self.spill_buffer_bytes),
            pending_index: BTreeMap::new(),
            end: 0,
            index: Some(BTreeMap::new()),
            index_bytes: 0,
            disk_index: None,
            index_limit: self.index_limit,
            buffer_bytes: self.spill_buffer_bytes,
            order_memory_bytes: self.order_memory_bytes,
            failed: false,
        };
        for id in order {
            spill.put(
                rows.get(&id)
                    .ok_or(StoreError::Integrity("candidate object"))?,
            )?;
        }
        self.storage = DeferredObjects::Spill(spill);
        self.spill_count += 1;
        self.spill_peak_bytes = self.spill_peak_bytes.max(self.encoded_bytes);
        Ok(())
    }

    fn all_reachable(mut self) -> Result<Self> {
        self.reachable.seal()?;
        if let DeferredObjects::Spill(spill) = &mut self.storage {
            spill.seal()?;
        }
        Ok(self)
    }
}

/// The same full-file completion check serves direct native output and private
/// Workspace candidates; persistence/finality policy remains with the owner.
pub(crate) fn build_checked_file(
    objects: &mut impl ObjectStore,
    source: impl Read,
    expected_len: u64,
) -> Result<layerfs_content::file::rope::CompletedFile> {
    let completed = layerfs_content::file::rope::build_complete(objects, source)?;
    if completed.logical_len != expected_len {
        return Err(StoreError::Integrity("completed file length"));
    }
    Ok(completed)
}

pub struct ObjectBuffer<'a> {
    source: Option<&'a dyn ObjectSource>,
    objects: DeferredObjectStore,
}

impl<'a> ObjectBuffer<'a> {
    /// Diagnostic source marker: actual regular-file owners only. Generic rope
    /// construction also serves mode/mtime metadata and must leave this unset.
    #[doc(hidden)]
    pub fn diagnostic_file_payloads(&mut self) {
        self.objects.diagnostic_file_context = true;
    }

    #[doc(hidden)]
    pub fn set_physical_predecessor(
        &mut self,
        reader: crate::SnapshotReader,
        root: layerfs_content::file::rope::FileStateRoot,
        operation_reserved: std::sync::Arc<std::sync::atomic::AtomicU64>,
    ) -> Result<()> {
        // Producer correspondence may overlap admission and other producers.
        // Reserve its cursor (64 KiB) plus one bounded physical metadata read
        // (512 KiB) from this producer's existing partition, never the encoder's
        // 2-MiB scratch. Small partitions preserve context but skip optional work.
        const CORRESPONDENCE_MEMORY: usize = 576 * 1024;
        let available = self.objects.memory_limit >= CORRESPONDENCE_MEMORY + 32 * 1024;
        if available {
            self.objects.memory_limit -= CORRESPONDENCE_MEMORY;
            // Spilled output retains its pending Vec capacity while ordered
            // delivery allocates read-ahead. Shrink both owners prospectively;
            // changing only the canonical spill threshold does not release them.
            self.objects.spill_buffer_bytes = self
                .objects
                .spill_buffer_bytes
                .saturating_sub(CORRESPONDENCE_MEMORY)
                .max(spill::ID_BUFFER_BYTES);
            if let DeferredObjects::Spill(spill) = &mut self.objects.storage {
                spill.flush()?;
                spill.pending = Vec::with_capacity(self.objects.spill_buffer_bytes);
                spill.buffer_bytes = self.objects.spill_buffer_bytes;
            }
            if matches!(&self.objects.storage, DeferredObjects::Memory { bytes, .. } if *bytes > self.objects.memory_limit)
            {
                self.objects.spill()?;
            }
        }
        self.objects.predecessor = Some((reader, root, operation_reserved, available));
        Ok(())
    }

    #[doc(hidden)]
    pub fn build_complete_with_predecessor(
        mut self,
        source: impl Read,
        expected_len: u64,
    ) -> Result<BuiltRoot> {
        self.diagnostic_file_payloads();
        self.objects.references = None;
        let completed = build_checked_file(&mut self, source, expected_len)?;
        self.finish_all_reachable(completed.root.0, completed.counters.cdc_bytes_scanned)
    }

    pub fn new(source: &'a dyn ObjectSource) -> Result<Self> {
        Ok(Self {
            source: Some(source),
            objects: DeferredObjectStore::new()?,
        })
    }

    pub fn empty() -> Result<Self> {
        Ok(Self {
            source: None,
            objects: DeferredObjectStore::new()?,
        })
    }

    pub(crate) fn empty_all_reachable() -> Result<Self> {
        Ok(Self {
            source: None,
            objects: DeferredObjectStore::new_all_reachable()?,
        })
    }

    #[doc(hidden)]
    pub fn resume_prevalidated(source: &'a dyn ObjectSource, objects: DeferredObjectStore) -> Self {
        Self {
            source: Some(source),
            objects,
        }
    }

    #[doc(hidden)]
    pub fn into_resumable(mut self) -> Result<DeferredObjectStore> {
        // The canonical spool remains resumable; the ID reader sees only a sealed order.
        self.objects.reachable.seal()?;
        Ok(self.objects)
    }

    #[doc(hidden)]
    pub fn into_prevalidated(self) -> Result<DeferredObjectStore> {
        self.objects.all_reachable()
    }

    /// Scratch for a single completed-file producer; queue and admission own the
    /// other bounded portions of the existing canonical-output allowance.
    pub fn bounded_output(source: Option<&'a dyn ObjectSource>) -> Result<Self> {
        let mut output = match source {
            Some(source) => Self::new(source)?,
            None => Self::empty()?,
        };
        output.objects.memory_limit = CANDIDATE_SPILL_BUFFER_BYTES;
        Ok(output)
    }

    /// Partition the existing aggregate candidate allowances, including spill
    /// buffers and indexes. The SQLite fallback's fixed cache must still fit.
    pub fn partition_output(&mut self, partitions: usize) -> Result<()> {
        if partitions == 0 || partitions > 8 {
            return Err(StoreError::InvalidInput("candidate partitions"));
        }
        if partitions == 1 {
            return Ok(());
        }
        self.objects.memory_limit = CANDIDATE_SPILL_BUFFER_BYTES / partitions;
        self.objects.index_limit = CANDIDATE_INDEX_BYTES / partitions;
        self.objects.spill_buffer_bytes = (CANDIDATE_SPILL_BUFFER_BYTES - spill::ID_BUFFER_BYTES)
            / partitions
            - spill::ID_BUFFER_BYTES;
        self.objects.order_memory_bytes = CANDIDATE_MEMORY_BYTES / partitions;
        if matches!(&self.objects.storage, DeferredObjects::Memory { bytes, .. } if *bytes > self.objects.memory_limit)
        {
            self.objects.spill()?;
        }
        if let DeferredObjects::Spill(spill) = &mut self.objects.storage {
            spill.flush()?;
            spill.pending = Vec::with_capacity(self.objects.spill_buffer_bytes);
            spill.buffer_bytes = self.objects.spill_buffer_bytes;
            spill.index_limit = self.objects.index_limit;
            spill.order_memory_bytes = self.objects.order_memory_bytes;
            if spill.index_bytes > spill.index_limit {
                spill.spill_index()?;
            }
        }
        if matches!(&self.objects.reachable, IdOrder::Memory(ids) if ids.len().saturating_mul(32) > self.objects.index_limit)
        {
            let mut order = IdOrder::empty();
            self.objects.reachable.seal()?;
            self.objects
                .reachable
                .visit(|id| order.push_bounded(id, self.objects.index_limit))?;
            order.seal()?;
            self.objects.reachable = order;
        }
        if self.objects.reference_bytes > self.objects.index_limit {
            self.objects.references = None;
            self.objects.reference_bytes = 0;
        }
        Ok(())
    }

    /// Full-file rope construction emits sealed prefixes, attaches every prefix
    /// to its final mapping root, and then emits FileState. This is the same
    /// append-only finality used by native import; incremental edits do not use it.
    /// Keep the entire file private until successful construction and length check.
    pub fn build_complete_file(source: impl Read, expected_len: u64) -> Result<BuiltRoot> {
        Self::build_complete_file_partition(source, expected_len, 1)
    }

    pub fn build_complete_file_partition(
        source: impl Read,
        expected_len: u64,
        partitions: usize,
    ) -> Result<BuiltRoot> {
        let mut objects = Self::bounded_output(None)?;
        objects.diagnostic_file_payloads();
        objects.partition_output(partitions)?;
        objects.objects.references = None;
        let completed = build_checked_file(&mut objects, source, expected_len)?;
        objects.finish_all_reachable(completed.root.0, completed.counters.cdc_bytes_scanned)
    }

    pub fn finish(self, root_id: ObjectId, cdc_bytes_scanned: u64) -> Result<BuiltRoot> {
        let encode_hash_invocations = self.objects.len();
        let objects = self.objects.reachable_from(root_id)?;
        Ok(BuiltRoot {
            root_id,
            counters: BuildCounters {
                cdc_bytes_scanned,
                encode_hash_invocations,
                first_store_write_bytes: objects.first_store_write_bytes,
                reachable_copy_write_bytes: 0,
                spill_peak_bytes: objects.spill_peak_bytes,
                spill_count: objects.spill_count,
            },
            objects,
        })
    }

    pub(crate) fn finish_all_reachable(
        self,
        root_id: ObjectId,
        cdc_bytes_scanned: u64,
    ) -> Result<BuiltRoot> {
        let encode_hash_invocations = self.objects.len();
        let objects = self.objects.all_reachable()?;
        Ok(BuiltRoot {
            root_id,
            counters: BuildCounters {
                cdc_bytes_scanned,
                encode_hash_invocations,
                first_store_write_bytes: objects.first_store_write_bytes,
                reachable_copy_write_bytes: 0,
                spill_peak_bytes: objects.spill_peak_bytes,
                spill_count: objects.spill_count,
            },
            objects,
        })
    }

    /// Preview/reconciliation uses the same task driver, with a private sink.
    pub fn construct_files<I, S: Send, T: Send>(
        &mut self,
        worker_limit: usize,
        task_count: usize,
        tasks: I,
        initialize: impl Fn(usize) -> Result<S> + Sync,
        step: impl Fn(&mut S, usize, I::Item, &mut FinalizedOutputWriter) -> Result<()> + Sync,
        finish: impl Fn(S) -> Result<T> + Sync,
    ) -> Result<Vec<T>>
    where
        I: Iterator + Send,
        I::Item: Send,
    {
        let cancelled = std::sync::atomic::AtomicBool::new(false);
        let (output, _) = run_finalized_output(
            worker_limit,
            task_count,
            tasks,
            &cancelled,
            initialize,
            step,
            finish,
            |page| {
                for object in page {
                    self.objects.put_authenticated(object)?;
                }
                Ok(())
            },
        )?;
        Ok(output.into_iter().map(|(result, _)| result).collect())
    }

    pub fn merge_prevalidated(&mut self, objects: DeferredObjectStore) -> Result<()> {
        objects
            .consume_prevalidated_pages(|page| {
                for object in page {
                    self.objects.put_authenticated(object)?;
                }
                Ok(())
            })
            .map(|_| ())
    }
}

impl ObjectStore for ObjectBuffer<'_> {
    fn get(&self, id: ObjectId) -> CoreResult<Vec<u8>> {
        if let Some(bytes) = self.objects.get(id).map_err(|_| CoreError::Io)? {
            return Ok(bytes);
        }
        self.source
            .ok_or(CoreError::MissingObject)?
            .read_object(id)
            .map_err(core_read_error)
    }

    fn with_authenticated_canonical<T, F>(&self, id: ObjectId, callback: F) -> CoreResult<T>
    where
        F: FnOnce(&[u8]) -> CoreResult<T>,
    {
        match &self.objects.storage {
            DeferredObjects::Memory { rows, .. } => {
                if let Some(object) = rows.get(&id) {
                    return callback(&object.bytes);
                }
            }
            DeferredObjects::Spill(_) => {
                if let Some(bytes) = self.objects.get(id).map_err(core_read_error)? {
                    layerfs_content::authenticate_identity(&bytes, id)?;
                    return callback(&bytes);
                }
            }
        }
        CoreReader(self.source.ok_or(CoreError::MissingObject)?)
            .with_authenticated_canonical(id, callback)
    }

    fn put_file_payload(
        &mut self,
        canonical: Vec<u8>,
        start: u64,
        len: u32,
    ) -> CoreResult<ObjectId> {
        let mut object = AuthenticatedCanonicalObject::new(canonical, None)?;
        object.1.first_span = Some((start, len));
        if self.objects.diagnostic_file_context {
            object.1.diagnostic = diagnostic::FILE;
        }
        let id = object.id;
        self.objects
            .put_authenticated(object)
            .map_err(|_| CoreError::Io)?;
        Ok(id)
    }

    fn put(&mut self, canonical: &[u8]) -> CoreResult<ObjectId> {
        self.put_owned(canonical.to_vec())
    }

    fn put_owned(&mut self, canonical: Vec<u8>) -> CoreResult<ObjectId> {
        let object = AuthenticatedCanonicalObject::new(canonical, None)?;
        let id = object.id;
        self.objects
            .put_authenticated(object)
            .map_err(|_| CoreError::Io)?;
        Ok(id)
    }
}

impl ObjectSource for ObjectBuffer<'_> {
    fn read_object(&self, id: ObjectId) -> Result<Vec<u8>> {
        ObjectStore::get(self, id).map_err(StoreError::from)
    }
}

impl ObjectSource for DeferredObjectStore {
    fn read_object(&self, id: ObjectId) -> Result<Vec<u8>> {
        self.get(id)?.ok_or(StoreError::MissingObject(id))
    }
}

pub fn empty_root(seed: [u8; 32]) -> Result<BuiltRoot> {
    let mut store = ObjectBuffer::empty()?;
    let root_id = filesystem::empty_root(&mut store, seed)?;
    store.finish(root_id, 0)
}

pub fn apply_changes(
    source: &dyn ObjectSource,
    base_root: ObjectId,
    changes: &[ContentChange],
    seed: [u8; 32],
) -> Result<BuiltRoot> {
    let mut store = ObjectBuffer::new(source)?;
    let applied = filesystem::apply_changes(&mut store, base_root, changes, seed)?;
    store.finish(applied.root_id, applied.counters.cdc_bytes_scanned)
}

pub(crate) fn combine_candidates(
    root_id: ObjectId,
    candidates: &[&DeferredObjectStore],
) -> Result<DeferredObjectStore> {
    let mut combined = DeferredObjectStore::new()?;
    for candidate in candidates {
        candidate.visit_authenticated_order(&candidate.reachable, &mut |object| {
            combined.put_authenticated(object.clone())
        })?;
    }
    combined.reachable_from(root_id)
}

fn elapsed_ns(started: Instant) -> u64 {
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

impl crate::schema::StoreDb {
    pub fn read_object_row(&self, id: ObjectId) -> Result<Vec<u8>> {
        let location = self
            .object_locations(&[id])?
            .remove(&id)
            .ok_or(StoreError::Integrity("visible object missing"))?;
        let mut bytes = None;
        self.visit_locations(&mut [(id, location)], |object| {
            bytes = Some(object.bytes);
            Ok(())
        })?;
        bytes.ok_or(StoreError::Integrity("visible object missing"))
    }

    pub fn read_object_rows(&self, ids: &[ObjectId]) -> Result<Vec<CanonicalObject>> {
        if let [id] = ids {
            return Ok(vec![CanonicalObject {
                id: *id,
                bytes: self.read_object_row(*id)?,
            }]);
        }
        if ids.len() > OBJECT_PAGE_COUNT {
            return Err(StoreError::InvalidInput("object read page"));
        }
        let mut remaining = BTreeMap::<ObjectId, usize>::new();
        for id in ids {
            *remaining.entry(*id).or_default() += 1;
        }
        let distinct = remaining.keys().copied().collect::<Vec<_>>();
        let locations = self.object_locations(&distinct)?;
        if locations.len() != distinct.len() {
            return Err(StoreError::Integrity("visible object cardinality"));
        }
        let mut locations = locations.into_iter().collect::<Vec<_>>();
        let mut rows = BTreeMap::new();
        self.visit_locations(&mut locations, |object| {
            rows.insert(object.id, object.bytes);
            Ok(())
        })?;
        let mut output = Vec::with_capacity(ids.len());
        for id in ids {
            let count = remaining
                .get_mut(id)
                .ok_or(StoreError::Integrity("visible object order"))?;
            *count -= 1;
            let bytes = if *count == 0 {
                rows.remove(id)
                    .ok_or(StoreError::Integrity("visible object missing"))?
            } else {
                let bytes = rows
                    .get(id)
                    .ok_or(StoreError::Integrity("visible object missing"))?
                    .clone();
                note_read_batch_clone(bytes.len());
                bytes
            };
            output.push(CanonicalObject { id: *id, bytes });
        }
        Ok(output)
    }

    pub fn object_membership(&self, ids: &[ObjectId]) -> Result<BTreeMap<ObjectId, u64>> {
        if ids.len() > OBJECT_PAGE_COUNT {
            return Err(StoreError::InvalidInput("object membership page"));
        }
        Ok(self
            .object_locations(ids)?
            .into_iter()
            .map(|(id, location)| (id, location.canonical_length as u64))
            .collect())
    }
}

impl CheckedOutputAdmission {
    pub(crate) fn new(db: &crate::schema::StoreDb) -> Result<Self> {
        Ok(Self {
            db: db.clone(),
            incoming: Vec::with_capacity(INITIALIZATION_SLAB_OBJECTS),
            incoming_index: HashMap::new(),
            incoming_bytes: 0,
            batch: Vec::with_capacity(INITIALIZATION_SLAB_OBJECTS),
            // Two independent facts share the existing index allowance: seen
            // input IDs and closed, available dependencies. Leave half for
            // bounded logical-reference decoding and temporary lookup pages.
            seen: SpillableObjectSet::bounded(CANDIDATE_INDEX_BYTES / 4)?,
            available: SpillableObjectSet::bounded(CANDIDATE_INDEX_BYTES / 4)?,
            pending: HashMap::new(),
            batch_bytes: 0,
            validation_reserve: 0,
            statement_number: 0,
            receipt: crate::CandidateReceipt::default(),
            checked: CheckedAdmission::default(),
            diagnostics: InitializationAdmissionDiagnostics::default(),
            final_phase: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn admit_worker_segment(&mut self, objects: DeferredObjectStore) -> Result<()> {
        self.admit(objects)
    }

    pub(crate) fn finish(mut self) -> Result<FinishedOutputAdmission> {
        self.probe_incoming()?;
        let final_batch = self.take_batch()?;
        Ok(FinishedOutputAdmission {
            final_batch,
            statement_number: self.statement_number,
            receipt: self.receipt,
            checked: self.checked,
            diagnostics: self.diagnostics,
        })
    }

    pub(crate) fn prepare_final_phase(&mut self) -> Result<()> {
        self.final_phase = true;
        Ok(())
    }

    fn observe_final_owned_bytes(&mut self, transient: u64, incoming: u64) {
        if !self.final_phase {
            return;
        }
        let owned = transient
            .saturating_add(incoming)
            .saturating_add(self.incoming_bytes as u64)
            .saturating_add(
                (self.incoming.capacity() * std::mem::size_of::<CanonicalObject>()) as u64,
            )
            .saturating_add((self.incoming_index.capacity() * 64) as u64)
            .saturating_add(self.batch_bytes as u64)
            .saturating_add((self.batch.capacity() * std::mem::size_of::<CanonicalObject>()) as u64)
            .saturating_add((self.pending.capacity() * 64) as u64);
        self.diagnostics.final_simultaneous_owned_peak_bytes = self
            .diagnostics
            .final_simultaneous_owned_peak_bytes
            .max(owned);
    }

    pub(crate) fn admit(&mut self, objects: DeferredObjectStore) -> Result<()> {
        objects
            .consume_prevalidated_pages(|page| self.admit_page(page))
            .map(|_| ())
    }

    pub(crate) fn admit_page(&mut self, page: Vec<AuthenticatedCanonicalObject>) -> Result<()> {
        for object in page {
            self.admit_object(object)?;
        }
        Ok(())
    }

    fn take_batch(&mut self) -> Result<MissingBatch> {
        self.close_dependencies()?;
        let batch = std::mem::take(&mut self.batch);
        self.pending.clear();
        self.batch_bytes = 0;
        self.validation_reserve = 0;
        Ok(MissingBatch(batch))
    }

    fn close_dependencies(&mut self) -> Result<()> {
        let mut dependencies = BTreeSet::new();
        for object in &self.batch {
            for id in referenced_objects(&object.bytes)? {
                if !self.pending.contains_key(&id) {
                    dependencies.insert(id);
                }
            }
        }
        let dependencies = dependencies.into_iter().collect::<Vec<_>>();
        for page in dependencies.chunks(OBJECT_PAGE_COUNT) {
            let known = self.available.membership(page)?;
            let missing = page
                .iter()
                .copied()
                .filter(|id| !known.contains(id))
                .collect::<Vec<_>>();
            if !self.db.objects_exist(&missing)? {
                return Err(StoreError::Integrity("new object dependency missing"));
            }
            self.available.insert_page(&missing)?;
        }
        Ok(())
    }

    fn probe_incoming(&mut self) -> Result<()> {
        if self.incoming.is_empty() {
            return Ok(());
        }
        let page = std::mem::take(&mut self.incoming);
        self.incoming_index.clear();
        self.incoming_bytes = 0;
        let ids = page.iter().map(|object| object.id).collect::<Vec<_>>();
        let first = self
            .seen
            .insert_page(&ids)?
            .into_iter()
            .collect::<BTreeSet<_>>();
        let (fresh, repeated): (Vec<_>, Vec<_>) = page
            .into_iter()
            .partition(|object| first.contains(&object.id));
        if !repeated.is_empty() {
            // Supplied duplicate occurrences still compare their actual bytes;
            // they are not new admission candidates or new initial probes.
            let ids = repeated.iter().map(|object| object.id).collect::<Vec<_>>();
            let known = self.db.object_locations(&ids)?;
            if known.len() != ids.len() {
                return Err(StoreError::Integrity("flushed duplicate missing"));
            }
            let supplied = repeated
                .iter()
                .map(|object| (object.id, object.bytes.as_slice()))
                .collect();
            let mut metrics = ObjectInsertMetrics::default();
            admission::compare(&self.db, &known, &supplied, &mut metrics)?;
            self.note_collision_reads(metrics);
            self.diagnostics.cross_batch_skipped_objects += repeated.len() as u64;
            self.diagnostics.cross_batch_skipped_bytes += metrics.skipped_bytes;
        }
        let mut diagnostic_stats = crate::PhysicalStorageReceipt::default();
        for mut object in repeated {
            diagnostic::occurrence(&mut object, 2, &mut diagnostic_stats);
        }
        self.db.note_physical(diagnostic_stats);
        let ids = fresh.iter().map(|object| object.id).collect::<Vec<_>>();
        let known = self.db.object_locations(&ids)?;
        let supplied = fresh
            .iter()
            .map(|object| (object.id, object.bytes.as_slice()))
            .collect();
        let mut reused = ObjectInsertMetrics::default();
        admission::compare(&self.db, &known, &supplied, &mut reused)?;
        self.note_collision_reads(reused);
        self.available
            .insert_page(&known.keys().copied().collect::<Vec<_>>())?;
        self.checked.candidate_objects += reused.skipped_ids;
        self.checked.candidate_bytes += reused.skipped_bytes;
        self.checked.reused_objects += reused.skipped_ids;
        self.checked.reused_bytes += reused.skipped_bytes;
        self.receipt.candidate_objects += reused.skipped_ids;
        self.receipt.candidate_bytes += reused.skipped_bytes;
        self.receipt.reused_objects += reused.skipped_ids;
        self.receipt.reused_bytes += reused.skipped_bytes;
        self.receipt.preexisting_reused_objects += reused.skipped_ids;
        self.receipt.preexisting_reused_bytes += reused.skipped_bytes;
        drop(supplied);
        let mut diagnostic_stats = crate::PhysicalStorageReceipt::default();
        for mut object in fresh {
            let missing = !known.contains_key(&object.id);
            diagnostic::occurrence(&mut object, u8::from(missing), &mut diagnostic_stats);
            if missing {
                diagnostic::eligible(&object, &mut diagnostic_stats);
            }
            if missing {
                self.push_pending(object)?;
            }
        }
        self.db.note_physical(diagnostic_stats);
        self.incoming = Vec::with_capacity(INITIALIZATION_SLAB_OBJECTS);
        Ok(())
    }

    fn note_collision_reads(&mut self, metrics: ObjectInsertMetrics) {
        self.diagnostics.collision_checks += metrics.collision_checks;
        self.diagnostics.conflict_read_calls += metrics.conflict_read_calls;
        self.diagnostics.conflict_read_rows += metrics.conflict_read_rows;
        self.diagnostics.conflict_read_bytes += metrics.conflict_read_bytes;
        self.diagnostics.conflict_read_ns += metrics.conflict_read_ns;
    }

    pub(crate) fn admit_object(&mut self, mut object: AuthenticatedCanonicalObject) -> Result<()> {
        if object.bytes.len() > ADMISSION_BATCH_BYTES {
            return Err(StoreError::Integrity("canonical object admission size"));
        }
        if let Some(&index) = self.pending.get(&object.id) {
            let mut stats = crate::PhysicalStorageReceipt::default();
            diagnostic::occurrence(&mut object, 2, &mut stats);
            self.db.note_physical(stats);
            return self.admit_duplicate(index, &object.bytes);
        }
        if let Some(&index) = self.incoming_index.get(&object.id) {
            let mut stats = crate::PhysicalStorageReceipt::default();
            diagnostic::occurrence(&mut object, 2, &mut stats);
            self.db.note_physical(stats);
            self.diagnostics.collision_checks += 1;
            if self.incoming[index].bytes != object.bytes {
                return Err(StoreError::Integrity("object collision"));
            }
            self.diagnostics.pending_duplicate_objects += 1;
            self.diagnostics.pending_duplicate_bytes += object.bytes.len() as u64;
            return Ok(());
        }
        let large = object.bytes.len() > INITIALIZATION_SLAB_BYTES;
        if !self.incoming.is_empty()
            && (self.incoming.len() == INITIALIZATION_SLAB_OBJECTS
                || self.incoming_bytes + object.bytes.len() > INITIALIZATION_SLAB_BYTES)
        {
            self.probe_incoming()?;
        }
        if large {
            self.flush_batch()?;
        }
        self.incoming_bytes += object.bytes.len();
        self.incoming_index.insert(object.id, self.incoming.len());
        self.incoming.push(object);
        if large {
            // Retire a maximal source object before requesting another source
            // page; it must not coexist with a second maximal canonical read.
            self.probe_incoming()?;
            self.flush_batch()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.probe_incoming()?;
        self.flush_batch()
    }

    fn admit_duplicate(&mut self, index: usize, bytes: &[u8]) -> Result<()> {
        self.diagnostics.collision_checks += 1;
        if self.batch[index].bytes != bytes {
            return Err(StoreError::Integrity("object collision"));
        }
        self.diagnostics.pending_duplicate_objects += 1;
        self.diagnostics.pending_duplicate_bytes = self
            .diagnostics
            .pending_duplicate_bytes
            .saturating_add(bytes.len() as u64);
        Ok(())
    }

    fn push_pending(&mut self, object: AuthenticatedCanonicalObject) -> Result<()> {
        if object.bytes.len() > ADMISSION_BATCH_BYTES {
            return Err(StoreError::Integrity("canonical object admission size"));
        }
        let reserve = read::validation_reserve(object.bytes.len());
        if !self.batch.is_empty()
            && (self.batch.len() == INITIALIZATION_ADMISSION_BATCH_COUNT
                || self.batch_bytes.saturating_add(object.bytes.len())
                    > 2 * INITIALIZATION_SLAB_BYTES
                || self.validation_reserve + reserve > read::VALIDATION_RESERVE)
        {
            self.flush_batch()?;
        }
        self.validation_reserve += reserve;
        self.batch_bytes = self.batch_bytes.saturating_add(object.bytes.len());
        self.pending.insert(object.id, self.batch.len());
        self.batch.push(object);
        self.diagnostics.batch_peak_objects = self
            .diagnostics
            .batch_peak_objects
            .max(self.batch.len() as u64);
        self.diagnostics.batch_peak_payload_bytes = self
            .diagnostics
            .batch_peak_payload_bytes
            .max(self.batch_bytes as u64);
        self.diagnostics.batch_peak_vec_capacity = self
            .diagnostics
            .batch_peak_vec_capacity
            .max(self.batch.capacity() as u64);
        self.diagnostics.pending_index_peak_entries = self
            .diagnostics
            .pending_index_peak_entries
            .max(self.pending.len() as u64);
        self.diagnostics.pending_index_peak_bytes = self
            .diagnostics
            .pending_index_peak_bytes
            .max((self.pending.capacity() * 64) as u64);
        if self.final_phase {
            self.diagnostics.final_batch_peak_payload_bytes = self
                .diagnostics
                .final_batch_peak_payload_bytes
                .max(self.batch_bytes as u64);
            self.diagnostics.final_batch_peak_vec_capacity = self
                .diagnostics
                .final_batch_peak_vec_capacity
                .max(self.batch.capacity() as u64);
            self.diagnostics.final_pending_index_peak_bytes = self
                .diagnostics
                .final_pending_index_peak_bytes
                .max((self.pending.capacity() * 64) as u64);
        }
        Ok(())
    }

    fn flush_batch(&mut self) -> Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }
        let capacity = self.batch.capacity();
        let batch = self.take_batch()?;
        if batch.is_empty() {
            return Ok(());
        }
        let ids = batch.iter().map(|object| object.id).collect::<Vec<_>>();
        let metrics = consume_checked_owned_page(&self.db, batch, &mut self.statement_number)?;
        self.available.insert_page(&ids)?;
        self.checked.record(&metrics);
        self.batch = Vec::with_capacity(capacity);
        self.diagnostics.record_sql_batch(
            metrics.insert,
            metrics.begin_ns,
            metrics.commit_ns,
            if self.final_phase {
                InitializationSqlPhase::FinalBuild
            } else {
                InitializationSqlPhase::Pipeline
            },
        );
        self.batch_bytes = 0;
        self.validation_reserve = 0;
        self.pending.clear();
        record_admission_receipt(&mut self.receipt, metrics.insert, false);
        Ok(())
    }
}

pub(crate) fn record_admission_receipt(
    receipt: &mut crate::CandidateReceipt,
    metrics: ObjectInsertMetrics,
    final_batch: bool,
) {
    let bytes = metrics.bytes + metrics.skipped_bytes;
    receipt.candidate_objects += metrics.submitted_rows;
    receipt.candidate_bytes += bytes;
    receipt.inserted_objects += metrics.objects;
    receipt.inserted_bytes += metrics.bytes;
    receipt.reused_objects += metrics.skipped_ids;
    receipt.reused_bytes += metrics.skipped_bytes;
    receipt.preexisting_reused_objects += metrics.skipped_ids;
    receipt.preexisting_reused_bytes += metrics.skipped_bytes;
    receipt.max_transaction_objects = receipt.max_transaction_objects.max(metrics.submitted_rows);
    receipt.max_transaction_bytes = receipt.max_transaction_bytes.max(bytes);
    if final_batch {
        receipt.final_inserted_objects += metrics.objects;
        receipt.final_inserted_bytes += metrics.bytes;
    } else {
        receipt.batch_inserted_objects += metrics.objects;
        receipt.batch_inserted_bytes += metrics.bytes;
        receipt.admission_transactions += 1;
    }
}

pub struct WorkspaceAdmission {
    pub(crate) db: crate::schema::StoreDb,
    pub(crate) workspace_id: [u8; 16],
    admission: CheckedOutputAdmission,
}

impl crate::LayerStackStore {
    pub fn workspace_admission(&self, workspace_id: [u8; 16]) -> Result<WorkspaceAdmission> {
        Ok(WorkspaceAdmission {
            db: self.db.clone(),
            workspace_id,
            admission: CheckedOutputAdmission::new(&self.db)?,
        })
    }

    // Match the shared driver's explicit task inputs and lifecycle callbacks.
    #[allow(clippy::too_many_arguments)]
    pub fn construct_workspace_files<I, S: Send, T: Send>(
        &self,
        workspace_id: [u8; 16],
        worker_limit: usize,
        task_count: usize,
        tasks: I,
        initialize: impl Fn(usize) -> Result<S> + Sync,
        step: impl Fn(&mut S, usize, I::Item, &mut FinalizedOutputWriter) -> Result<()> + Sync,
        finish: impl Fn(S) -> Result<T> + Sync,
    ) -> Result<(Vec<T>, WorkspaceAdmission)>
    where
        I: Iterator + Send,
        I::Item: Send,
    {
        let mut token = self.workspace_admission(workspace_id)?;
        let cancelled = std::sync::atomic::AtomicBool::new(false);
        let mut admission_ns = 0_u64;
        let (output, pipeline) = run_finalized_output(
            worker_limit,
            task_count,
            tasks,
            &cancelled,
            initialize,
            step,
            finish,
            |page| {
                let started = Instant::now();
                let result = token.admission.admit_page(page);
                admission_ns = admission_ns.saturating_add(elapsed_ns(started));
                result
            },
        )?;
        // Keep the final file batch owned by the Workspace token. Namespace
        // construction may add to it before staging closes the operation.
        let mut writer = OutputWriterMetrics::default();
        let output = output
            .into_iter()
            .map(|(result, metrics)| {
                writer.merge(metrics);
                result
            })
            .collect();
        if writer.producer_tasks != task_count as u64 {
            return Err(StoreError::Integrity("Workspace file task coverage"));
        }
        crate::telemetry::note_workspace_candidate_delivery(
            writer.selected_memory_bytes,
            writer.selected_spill_bytes,
            writer.selected_storage_authentication_ns,
        );
        crate::telemetry::note_workspace_output_pipeline(
            pipeline.wall_ns,
            admission_ns,
            writer.blocked_ns,
            pipeline.consumer_idle_ns,
            pipeline.queue_peak_bytes,
        );
        Ok((output, token))
    }
}

impl WorkspaceAdmission {
    pub(crate) fn admit_remaining(
        mut self,
        objects: DeferredObjectStore,
    ) -> Result<(CheckedAdmission, u64)> {
        objects.consume_prevalidated_pages(|page| self.admission.admit_page(page))?;
        self.admission.flush()?;
        let finished = self.admission.finish()?;
        let admission = finished.checked;
        if admission.candidate_objects != admission.inserted_objects + admission.reused_objects
            || admission.candidate_bytes != admission.inserted_bytes + admission.reused_bytes
            || admission.max_transaction_objects > ADMISSION_BATCH_COUNT as u64
            || admission.max_transaction_bytes > ADMISSION_BATCH_BYTES as u64
        {
            return Err(StoreError::Integrity("checked admission equation"));
        }
        Ok((admission, finished.statement_number))
    }
}

fn consume_checked_owned_page(
    db: &crate::schema::StoreDb,
    batch: MissingBatch,
    statement_number: &mut u64,
) -> Result<AdmissionBatchMetrics> {
    let prepared = admission::PreparedAdmission::prepare_missing(db, batch)?;
    let (_, metrics) = prepared.publish(db, statement_number, |_, _, _| {
        #[cfg(feature = "test-instrumentation")]
        crate::schema::verification_store_checkpoint(
            crate::schema::VerificationStoreFault::LaterAdmissionBatch,
        )?;
        Ok(())
    })?;
    #[cfg(feature = "test-instrumentation")]
    crate::schema::verification_early_committed();
    Ok(metrics)
}

impl ObjectSource for crate::schema::StoreDb {
    fn read_object(&self, id: ObjectId) -> Result<Vec<u8>> {
        self.read_object_row(id)
    }

    fn read_authenticated_objects(&self, ids: &[ObjectId]) -> Result<Vec<CanonicalObject>> {
        self.read_object_rows(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_file_output_matches_reachability_selection_across_spill_and_tree_boundaries() {
        use layerfs_content::file::{cdc::MAXIMUM_CHUNK_BYTES, extent::MAX_ENTRIES, rope};
        for size in [
            0,
            1,
            MAXIMUM_CHUNK_BYTES + 1,
            MAXIMUM_CHUNK_BYTES * (MAX_ENTRIES + 1),
        ] {
            for repetitive in [false, true] {
                let mut random = 7_u64;
                let bytes = (0..size)
                    .map(|_| {
                        random ^= random << 13;
                        random ^= random >> 7;
                        random ^= random << 17;
                        if repetitive { 0 } else { random as u8 }
                    })
                    .collect::<Vec<_>>();
                let mut selected = ObjectBuffer::bounded_output(None).unwrap();
                let (root, counters) = rope::build(&mut selected, bytes.as_slice()).unwrap();
                let selected = selected.finish(root.0, counters.cdc_bytes_scanned).unwrap();
                let direct =
                    ObjectBuffer::build_complete_file(bytes.as_slice(), size as u64).unwrap();
                assert_eq!(direct.root_id, selected.root_id);
                assert_eq!(direct.objects.len(), selected.objects.len());
                assert_eq!(
                    direct.objects.encoded_bytes(),
                    selected.objects.encoded_bytes()
                );
                assert_eq!(
                    direct
                        .objects
                        .ids_in_order(usize::MAX)
                        .unwrap()
                        .unwrap()
                        .into_iter()
                        .collect::<BTreeSet<_>>(),
                    selected
                        .objects
                        .ids_in_order(usize::MAX)
                        .unwrap()
                        .unwrap()
                        .into_iter()
                        .collect::<BTreeSet<_>>()
                );
                assert!(!direct.objects.has_reference_index());
                if !repetitive && size > CANDIDATE_SPILL_BUFFER_BYTES {
                    assert!(direct.counters.spill_count > 0);
                }
            }
        }
        assert!(matches!(
            ObjectBuffer::build_complete_file(b"short".as_slice(), 6),
            Err(StoreError::Integrity("completed file length"))
        ));
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::Other.into())
            }
        }
        assert!(ObjectBuffer::build_complete_file(Broken, 1).is_err());
    }

    #[test]
    fn partitioned_completed_files_share_direct_facts_and_keep_failures_private() {
        let mut random = 23_u64;
        let data = (0..2 * 1024 * 1024 + 17)
            .map(|_| {
                random ^= random << 13;
                random ^= random >> 7;
                random ^= random << 17;
                random as u8
            })
            .collect::<Vec<_>>();
        let private =
            ObjectBuffer::build_complete_file_partition(data.as_slice(), data.len() as u64, 4)
                .unwrap();
        assert!(private.counters.spill_count > 0);
        let expected_ids = private
            .objects
            .ids_in_order(usize::MAX)
            .unwrap()
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut direct_ids = BTreeSet::new();
        let cancelled = std::sync::atomic::AtomicBool::new(false);
        let (direct, _) = run_finalized_output(
            1,
            1,
            std::iter::once(()),
            &cancelled,
            |_| Ok(None),
            |result, _, _, writer| {
                *result = Some(build_checked_file(
                    writer,
                    data.as_slice(),
                    data.len() as u64,
                )?);
                Ok(())
            },
            |result| result.ok_or(StoreError::Integrity("missing completion")),
            |page| {
                direct_ids.extend(page.into_iter().map(|object| object.id));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(direct[0].0.root.0, private.root_id);
        assert_eq!(direct[0].0.logical_len, data.len() as u64);
        assert_eq!(
            direct[0].0.counters.cdc_bytes_scanned,
            private.counters.cdc_bytes_scanned
        );
        assert_eq!(direct_ids, expected_ids);
        let mut buffer = ObjectBuffer::bounded_output(None).unwrap();
        buffer.partition_output(4).unwrap();
        assert_eq!(
            buffer.objects.memory_limit * 4,
            CANDIDATE_SPILL_BUFFER_BYTES
        );
        assert_eq!(buffer.objects.index_limit * 4, CANDIDATE_INDEX_BYTES);
        assert_eq!(
            buffer.objects.spill_buffer_bytes * 4,
            CANDIDATE_SPILL_BUFFER_BYTES
        );
        assert!(buffer.partition_output(0).is_err());

        struct Broken {
            remaining: usize,
        }
        impl Read for Broken {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                if self.remaining == 0 {
                    return Err(std::io::ErrorKind::Other.into());
                }
                let size = bytes.len().min(self.remaining);
                bytes[..size].fill(7);
                self.remaining -= size;
                Ok(size)
            }
        }
        let path = std::env::temp_dir().join(format!(
            "layerfs-private-failure-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = crate::LayerStackStore::create(&path).unwrap();
        let before = store.store_counts().unwrap();
        let failed = store.construct_workspace_files(
            [5; 16],
            4,
            1,
            std::iter::once(()),
            |_| Ok(()),
            |_, _, _, writer| {
                let built = ObjectBuffer::build_complete_file_partition(
                    Broken {
                        remaining: 2 * 1024 * 1024,
                    },
                    2 * 1024 * 1024 + 1,
                    4,
                )?;
                writer.send_selected(built.objects)
            },
            |_| Ok(()),
        );
        assert!(failed.is_err());
        assert_eq!(store.store_counts().unwrap(), before);
        assert!(
            ObjectBuffer::build_complete_file_partition(data.as_slice(), data.len() as u64 + 1, 4)
                .is_err()
        );
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn seen_index_spill_preserves_exact_membership_and_private_cleanup() {
        let ids = [b"first".as_slice(), b"second", b"third"].map(ObjectId::for_bytes);
        let mut seen = SpillableObjectSet::empty().unwrap();
        assert_eq!(seen.insert_page(&ids[..2]).unwrap(), ids[..2]);
        seen.spill().unwrap();
        let path = match &seen.storage {
            SeenStorage::Spill { connection, _path } => {
                let connection = connection.lock().unwrap();
                let plan: String = connection
                    .query_row(
                        "EXPLAIN QUERY PLAN SELECT 1 FROM seen WHERE id=?1",
                        [ids[0].as_bytes().as_slice()],
                        |row| row.get(3),
                    )
                    .unwrap();
                assert!(plan.contains("SEARCH") && plan.contains("PRIMARY KEY"));
                assert_eq!(
                    connection
                        .pragma_query_value::<i64, _>(None, "cache_size", |row| row.get(0))
                        .unwrap(),
                    -4096
                );
                _path.0.clone()
            }
            _ => panic!("forced seen spill"),
        };
        assert!(seen.contains(ids[0]).unwrap());
        assert!(!seen.contains(ids[2]).unwrap());
        assert_eq!(
            seen.insert_page(&[ids[1], ids[2], ids[2]]).unwrap(),
            [ids[2]]
        );
        assert_eq!(seen.count, 3);
        drop(seen);
        assert!(!path.exists());
    }

    #[test]
    fn consumer_batches_flushed_duplicate_checks_without_recounting() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-duplicate-pages-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let db = crate::schema::StoreDb::create(root.join("store.sqlite")).unwrap();
        let mut admission = CheckedOutputAdmission::new(&db).unwrap();
        let objects = (0..300_u64)
            .map(|index| {
                let bytes = layerfs_content::encode_bytes_object(&index.to_le_bytes()).unwrap();
                AuthenticatedCanonicalObject::new(bytes, None).unwrap()
            })
            .collect::<Vec<_>>();
        admission.admit_page(objects.clone()).unwrap();
        admission.flush().unwrap();
        #[cfg(feature = "test-instrumentation")]
        crate::schema::reset_sql_trace();
        admission
            .admit_page(objects.iter().rev().cloned().collect())
            .unwrap();
        admission.flush().unwrap();
        #[cfg(feature = "test-instrumentation")]
        assert_eq!(
            crate::schema::sql_trace()
                .iter()
                .filter(|sql| sql.contains("WHERE object_id IN ("))
                .count(),
            3
        );
        assert_eq!(
            (
                admission.checked.candidate_objects,
                admission.checked.inserted_objects,
                admission.checked.reused_objects
            ),
            (300, 300, 0)
        );
        let mut corrupt = objects[0].clone();
        // Deliberately violate the private invariant to exercise exact conflict comparison.
        // Production owners expose no mutable bytes.
        corrupt.0.bytes = objects[1].bytes.clone();
        assert!(matches!(
            admission
                .admit_page(vec![corrupt])
                .and_then(|_| admission.flush()),
            Err(StoreError::Integrity("object collision"))
        ));
        drop(admission);
        drop(db);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn finalized_output_tasks_restore_coverage_and_join_all_failures() {
        use std::sync::atomic::AtomicBool;
        use std::sync::{Condvar, Mutex};
        for workers in [1, 4] {
            let cancelled = AtomicBool::new(false);
            let order = (Mutex::new((3_usize, Vec::new())), Condvar::new());
            let (output, metrics) = run_finalized_output(
                workers,
                4,
                0..4,
                &cancelled,
                |_| Ok(Vec::new()),
                |local, ordinal, task, _| {
                    assert_eq!(ordinal, task);
                    if workers > 1 {
                        let mut state = order.0.lock().unwrap();
                        while state.0 != ordinal {
                            state = order.1.wait(state).unwrap();
                        }
                        state.1.push(ordinal);
                        state.0 = state.0.saturating_sub(1);
                        order.1.notify_all();
                    }
                    local.push(ordinal);
                    Ok(())
                },
                Ok,
                |_| Ok(()),
            )
            .unwrap();
            assert_eq!(metrics.producers_after, 0);
            assert_eq!(
                output
                    .iter()
                    .map(|(_, metrics)| metrics.producer_tasks)
                    .sum::<u64>(),
                4
            );
            let mut ordinals = output
                .into_iter()
                .flat_map(|(rows, _)| rows)
                .collect::<Vec<_>>();
            ordinals.sort();
            assert_eq!(ordinals, vec![0, 1, 2, 3]);
            if workers > 1 {
                assert_eq!(order.0.lock().unwrap().1, vec![3, 2, 1, 0]);
            }
        }
        let cancelled = AtomicBool::new(false);
        let (empty, metrics) = run_finalized_output(
            4,
            0,
            std::iter::empty::<()>(),
            &cancelled,
            |_| -> Result<()> { panic!("empty worker") },
            |_, _, _, _| Ok(()),
            Ok,
            |_| Ok(()),
        )
        .unwrap();
        assert!(empty.is_empty());
        assert_eq!(metrics.producer_peak, 0);
        assert!(
            run_finalized_output(
                1,
                2,
                0..1,
                &cancelled,
                |_| Ok(()),
                |_, _, _, _| Ok(()),
                Ok,
                |_| Ok(())
            )
            .is_err()
        );

        for failure in [
            "producer",
            "producer-panic",
            "consumer",
            "consumer-panic",
            "finish",
            "finish-panic",
            "writer-finish",
        ] {
            let cancelled = AtomicBool::new(false);
            let joined = AtomicU64::new(0);
            struct Finished<'a>(&'a AtomicU64);
            impl Drop for Finished<'_> {
                fn drop(&mut self) {
                    self.0.fetch_add(1, Ordering::Release);
                }
            }
            let result = run_finalized_output(
                2,
                64,
                0..64,
                &cancelled,
                |_| Ok(Finished(&joined)),
                |_, index, _, writer| {
                    if index == 0 && failure == "producer" {
                        return Err(StoreError::Integrity("test producer failure"));
                    }
                    if index == 0 && failure == "producer-panic" {
                        panic!("test producer panic");
                    }
                    writer.push_object(
                        AuthenticatedCanonicalObject::new(
                            layerfs_content::encode_bytes_object(b"selected")?,
                            None,
                        )?,
                        false,
                    )?;
                    if failure == "writer-finish" {
                        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
                        drop(receiver);
                        writer.sender = sender;
                    } else {
                        writer.flush()?;
                    }
                    Ok(())
                },
                |owner| {
                    if failure == "finish" {
                        return Err(StoreError::Integrity("test finish failure"));
                    }
                    if failure == "finish-panic" {
                        panic!("test finish panic");
                    }
                    drop(owner);
                    Ok(())
                },
                |_| {
                    if failure == "consumer-panic" {
                        panic!("test consumer panic");
                    }
                    if failure == "consumer" {
                        Err(StoreError::Integrity("test consumer failure"))
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(result.is_err(), "{failure}");
            assert_eq!(joined.load(Ordering::Acquire), 2, "{failure}");
            assert!(cancelled.load(Ordering::Acquire), "{failure}");
        }
    }

    #[test]
    fn workspace_delivery_selects_before_admission_and_deduplicates_across_phases() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-output-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let store = crate::LayerStackStore::create(root.join("store.sqlite")).unwrap();
        let canonical = [b"existing".as_slice(), b"selected", b"provisional"]
            .map(|bytes| layerfs_content::encode_bytes_object(bytes).unwrap());
        let ids = canonical.each_ref().map(|bytes| ObjectId::for_bytes(bytes));
        let selected = |index: usize| {
            let mut buffer = ObjectBuffer::empty().unwrap();
            buffer.put(&canonical[2]).unwrap();
            let id = buffer.put(&canonical[index]).unwrap();
            buffer.finish(id, 0).unwrap().objects
        };
        store
            .workspace_admission([0; 16])
            .unwrap()
            .admit_remaining(selected(0))
            .unwrap();
        let (_, token) = store
            .construct_workspace_files(
                [1; 16],
                2,
                3,
                [0, 1, 1].into_iter(),
                |_| Ok(()),
                |_, _, index, writer| writer.send_selected(selected(index)),
                |_| Ok(()),
            )
            .unwrap();
        let (receipt, _) = token.admit_remaining(selected(1)).unwrap();
        assert_eq!(
            (
                receipt.candidate_objects,
                receipt.inserted_objects,
                receipt.reused_objects
            ),
            (2, 1, 1)
        );
        assert_eq!(
            receipt.candidate_bytes,
            (canonical[0].len() + canonical[1].len()) as u64
        );
        assert_eq!(store.db.read_object_row(ids[0]).unwrap(), canonical[0]);
        assert_eq!(store.db.read_object_row(ids[1]).unwrap(), canonical[1]);
        assert!(store.db.object_membership(&[ids[2]]).unwrap().is_empty());
        assert_eq!(store.store_counts().unwrap().objects, 2);
        assert_eq!(store.store_counts().unwrap().commits, 0);
        assert!(store.workspace_stage([1; 16]).unwrap().is_none());
        crate::schema::set_transaction_failure_at(Some(1));
        let failed = store.construct_workspace_files(
            [2; 16],
            1,
            1,
            std::iter::once(2),
            |_| Ok(()),
            |_, _, index, writer| writer.send_selected(selected(index)),
            |_| Ok(()),
        );
        crate::schema::set_transaction_failure_at(None);
        assert!(matches!(
            failed,
            Err(StoreError::Integrity("injected transaction failure"))
        ));
        assert_eq!(store.store_counts().unwrap().objects, 2);
        assert!(store.workspace_stage([2; 16]).unwrap().is_none());
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn owned_delivery_authenticates_fresh_spill_before_handoff() {
        use std::os::unix::fs::FileExt;
        let mut buffer = ObjectBuffer::empty().unwrap();
        let root = buffer
            .put_owned(layerfs_content::encode_bytes_object(b"spill witness").unwrap())
            .unwrap();
        buffer.objects.spill().unwrap();
        let (writer, offset) = match &buffer.objects.storage {
            DeferredObjects::Spill(spill) => (
                spill.writer.as_ref().unwrap().try_clone().unwrap(),
                spill.location(root).unwrap().unwrap().0,
            ),
            _ => panic!("forced payload spill"),
        };
        let built = buffer.finish(root, 0).unwrap();
        // Simulate corruption through a test-only descriptor retained before seal.
        writer.write_all_at(&[0xff], offset + 9).unwrap();
        let mut handed_off = 0;
        assert!(
            built
                .objects
                .consume_prevalidated_pages(|page| {
                    handed_off += page.len();
                    Ok(())
                })
                .is_err()
        );
        assert_eq!(handed_off, 0);
    }

    #[test]
    fn checked_owned_output_requires_complete_framing_and_identity() {
        let canonical = layerfs_content::encode_bytes_object(b"checked owner").unwrap();
        let owner = AuthenticatedCanonicalObject::new(canonical.clone(), None).unwrap();
        assert_eq!(owner.id, ObjectId::for_bytes(&canonical));
        assert_eq!(
            std::mem::size_of::<AuthenticatedCanonicalObject>(),
            std::mem::size_of::<CanonicalObject>() + std::mem::size_of::<PhysicalHints>()
        );
        assert!(matches!(
            AuthenticatedCanonicalObject::new(
                canonical.clone(),
                Some(ObjectId::for_bytes(b"wrong"))
            ),
            Err(CoreError::IdentityMismatch)
        ));
        for invalid in [
            canonical[..8].to_vec(),
            [canonical.as_slice(), &[0]].concat(),
            b"unframed".to_vec(),
        ] {
            let expected = ObjectId::for_bytes(&invalid);
            assert!(AuthenticatedCanonicalObject::new(invalid.clone(), None).is_err());
            assert!(AuthenticatedCanonicalObject::new(invalid, Some(expected)).is_err());
        }
    }

    #[test]
    fn structural_handoff_identity_is_fixed_at_buffer_insertion() {
        let bytes =
            layerfs_content::encode_bytes_object(b"authenticated structural object").unwrap();
        let expected = ObjectId::for_bytes(&bytes);
        let wrong = ObjectId::for_bytes(b"different structural object");
        assert!(matches!(
            AuthenticatedCanonicalObject::new(bytes.clone(), Some(wrong)),
            Err(CoreError::IdentityMismatch)
        ));

        let mut buffer = InitializationTaskObjectBuffer::new();
        assert_eq!(buffer.put_owned(bytes).unwrap(), expected);
        assert_eq!(buffer.objects[0].id, expected);
    }

    #[test]
    fn parent_payload_copy_counter_has_a_positive_control() {
        let counter = ParentPayloadCopyCounter::start();
        let object = CanonicalObject {
            id: ObjectId::for_bytes(b"parent-copy-control"),
            bytes: b"parent-copy-control".to_vec(),
        };
        let copy = object.clone();
        assert_eq!(counter.bytes(), copy.bytes.len() as u64);
    }

    fn sealed_segment(objects: Vec<CanonicalObject>) -> DeferredObjectStore {
        let mut segment = DeferredObjectStore::new_all_reachable().unwrap();
        for object in objects {
            segment.put(object.id, &object.bytes).unwrap();
        }
        segment.all_reachable().unwrap()
    }

    fn finish_segment_admission(
        db: &crate::schema::StoreDb,
        admission: CheckedOutputAdmission,
    ) -> (crate::CandidateReceipt, Vec<ObjectId>) {
        let finished = admission.finish().unwrap();
        let ids = finished
            .final_batch
            .iter()
            .map(|object| object.id)
            .collect::<Vec<_>>();
        let mut statement_number = finished.statement_number;
        let (_, metrics) = PreparedAdmission::prepare_missing(&db, finished.final_batch)
            .unwrap()
            .publish(db, &mut statement_number, |_, _, _| Ok(()))
            .unwrap();
        let mut receipt = finished.receipt;
        record_admission_receipt(&mut receipt, metrics.insert, true);
        assert_eq!(receipt.candidate_objects, receipt.inserted_objects);
        assert_eq!(receipt.candidate_bytes, receipt.inserted_bytes);
        assert_eq!(
            receipt.inserted_objects,
            receipt.batch_inserted_objects + receipt.final_inserted_objects
        );
        assert_eq!(
            receipt.inserted_bytes,
            receipt.batch_inserted_bytes + receipt.final_inserted_bytes
        );
        (receipt, ids)
    }

    #[test]
    fn memory_segment_moves_owned_canonical_bytes() {
        let bytes = layerfs_content::encode_bytes_object(b"owned").unwrap();
        let id = ObjectId::for_bytes(&bytes);
        let mut segment = DeferredObjectStore::new_all_reachable().unwrap();
        segment.put(id, &bytes).unwrap();
        let original = match &segment.storage {
            DeferredObjects::Memory { rows, .. } => rows.get(&id).unwrap().bytes.as_ptr(),
            DeferredObjects::Spill(_) => panic!("small segment spilled"),
        };
        let buffer = ObjectBuffer {
            source: None,
            objects: segment,
        };
        ObjectStore::with_authenticated_canonical(&buffer, id, |bytes| {
            assert_eq!(bytes.as_ptr(), original);
            Ok(())
        })
        .unwrap();
        buffer
            .into_resumable()
            .unwrap()
            .all_reachable()
            .unwrap()
            .consume_prevalidated_pages(|page| {
                assert_eq!(page.len(), 1);
                assert_eq!(page[0].id, id);
                assert_eq!(page[0].bytes.as_ptr(), original);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn append_only_writer_rejects_and_counts_outer_gets() {
        let writer = AppendOnlyInitializationWriter::new(64).unwrap();
        let path = writer.path.0.clone();
        assert!(ObjectStore::get(&writer, ObjectId::for_bytes(b"missing")).is_err());
        assert_eq!(writer.get_calls(), 1);
        drop(writer);
        assert!(!path.exists());
    }

    #[test]
    fn empty_append_only_segment_finishes_without_a_read_pass() {
        let writer = AppendOnlyInitializationWriter::new(64).unwrap();
        let path = writer.path.0.clone();
        let segment = writer.seal().unwrap();
        assert!(!path.exists());
        assert_eq!(
            segment.finish_consumption().unwrap(),
            InitializationSegmentIoMetrics::default()
        );
    }

    #[cfg(unix)]
    #[test]
    fn append_only_blocks_are_written_once_read_forward_and_unlinked() {
        let first = layerfs_content::encode_bytes_object(b"first").unwrap();
        let second = layerfs_content::encode_bytes_object(b"second").unwrap();
        let mut writer = AppendOnlyInitializationWriter::new(64).unwrap();
        let first_checkpoint = writer.checkpoint();
        let first_id = ObjectStore::put(&mut writer, &first).unwrap();
        let first_block = writer.block_since(0, 0, first_checkpoint).unwrap();
        let second_checkpoint = writer.checkpoint();
        let second_id = ObjectStore::put(&mut writer, &second).unwrap();
        let second_block = writer.block_since(1, 0, second_checkpoint).unwrap();
        assert_eq!(first_block.object_count, 1);
        assert_eq!(first_block.byte_count, first.len() as u64);
        assert_eq!(second_block.object_count, 1);
        assert_eq!(second_block.byte_count, second.len() as u64);
        assert_eq!(second_block.end, (first.len() + second.len() + 80) as u64);
        assert_eq!(writer.get_calls(), 0);
        let path = writer.path.0.clone();
        let mut segment = writer.seal().unwrap();
        assert!(segment.reader_capacity() <= 64);
        assert_eq!(segment.path(), path);
        assert!(!path.exists());
        assert!(segment.consume_block(second_block, |_| Ok(())).is_err());
        let mut ids = Vec::new();
        segment
            .consume_block(first_block, |object| {
                ids.push(object.id);
                Ok(())
            })
            .unwrap();
        segment
            .consume_block(second_block, |object| {
                ids.push(object.id);
                Ok(())
            })
            .unwrap();
        assert_eq!(ids, vec![first_id, second_id]);
        assert_eq!(segment.raw_read_bytes(), second_block.end);
        assert!(
            segment.raw_reads()
                < first_block
                    .object_count
                    .saturating_add(second_block.object_count)
                    * 3
        );
        segment.finish_consumption().unwrap();

        let mut failed = AppendOnlyInitializationWriter::new(64).unwrap();
        ObjectStore::put(&mut failed, &first).unwrap();
        failed.flush().unwrap();
        failed.writer.set_len(0).unwrap();
        let failed_path = failed.path.0.clone();
        assert!(failed.seal().is_err());
        assert!(!failed_path.exists());
    }

    #[test]
    fn buffered_append_reader_scales_raw_reads_with_bytes_not_tiny_frames() {
        let capacity = 4096;
        let mut writer = AppendOnlyInitializationWriter::new(capacity).unwrap();
        let checkpoint = writer.checkpoint();
        for index in 0_u64..5_000 {
            let canonical = layerfs_content::encode_bytes_object(&index.to_be_bytes()).unwrap();
            ObjectStore::put(&mut writer, &canonical).unwrap();
        }
        let block = writer.block_since(0, 0, checkpoint).unwrap();
        let mut segment = writer.seal().unwrap();
        let mut frames = 0_u64;
        segment
            .consume_block(block, |_| {
                frames += 1;
                Ok(())
            })
            .unwrap();
        let expected_max_reads = block.end.div_ceil(capacity as u64).saturating_add(1);
        assert_eq!(frames, 5_000);
        assert_eq!(segment.raw_read_bytes(), block.end);
        assert!(segment.raw_reads() <= expected_max_reads);
        assert!(segment.raw_reads() * 10 < frames);
        let metrics = segment.finish_consumption().unwrap();
        assert_eq!(metrics.frames, frames);
        assert_eq!(metrics.write_bytes, block.end);
        assert_eq!(metrics.raw_read_bytes, block.end);
    }

    #[test]
    fn append_only_reversed_worker_completion_sorts_to_task_order() {
        let first = layerfs_content::encode_bytes_object(b"first-task").unwrap();
        let second = layerfs_content::encode_bytes_object(b"second-task").unwrap();
        let mut worker_zero = AppendOnlyInitializationWriter::new(64).unwrap();
        let zero_checkpoint = worker_zero.checkpoint();
        let second_id = ObjectStore::put(&mut worker_zero, &second).unwrap();
        let second_block = worker_zero.block_since(1, 0, zero_checkpoint).unwrap();
        let mut worker_one = AppendOnlyInitializationWriter::new(64).unwrap();
        let one_checkpoint = worker_one.checkpoint();
        let first_id = ObjectStore::put(&mut worker_one, &first).unwrap();
        let first_block = worker_one.block_since(0, 1, one_checkpoint).unwrap();
        let mut segments = vec![worker_zero.seal().unwrap(), worker_one.seal().unwrap()];
        let mut completed = vec![second_block, first_block];
        completed.sort_by_key(|block| block.task_ordinal);
        let mut ids = Vec::new();
        for block in completed {
            segments[block.worker_index]
                .consume_block(block, |object| {
                    ids.push(object.id);
                    Ok(())
                })
                .unwrap();
        }
        assert_eq!(ids, vec![first_id, second_id]);
        for segment in segments {
            segment.finish_consumption().unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn compact_inode_pair_sidecar_is_bounded_forward_only_and_unlinked() {
        let mut writer = CompactInodePairWriter::new(128).unwrap();
        let pair_buffer_capacity = writer.pending_capacity();
        assert!(pair_buffer_capacity >= 128);
        let first_checkpoint = writer.checkpoint();
        let first = (
            layerfs_content::tree::inode::InodeId::allocate([1; 32], 1),
            ObjectId::for_bytes(b"first-record"),
        );
        writer.push(first.0, first.1).unwrap();
        let first_block = writer.block_since(0, 0, first_checkpoint).unwrap();
        let second_checkpoint = writer.checkpoint();
        let second = (
            layerfs_content::tree::inode::InodeId::allocate([1; 32], 2),
            ObjectId::for_bytes(b"second-record"),
        );
        writer.push(second.0, second.1).unwrap();
        let second_block = writer.block_since(1, 0, second_checkpoint).unwrap();
        assert_eq!((first_block.start, first_block.end), (0, 64));
        assert_eq!((second_block.start, second_block.end), (64, 128));
        let path = writer.path.0.clone();
        let segment = writer.seal().unwrap();
        assert_eq!(segment.path(), path);
        assert!(!path.exists());
        assert!(
            CompactInodePairStream::new(vec![segment], vec![second_block, first_block]).is_err()
        );

        let mut writer = CompactInodePairWriter::new(1024).unwrap();
        let checkpoint = writer.checkpoint();
        writer.push(first.0, first.1).unwrap();
        let first_block = writer.block_since(0, 0, checkpoint).unwrap();
        let checkpoint = writer.checkpoint();
        writer.push(second.0, second.1).unwrap();
        let second_block = writer.block_since(1, 0, checkpoint).unwrap();
        let mut segment = writer.seal().unwrap();
        assert!(segment.reader_capacity() <= 1024);
        let pairs = vec![
            segment.read_pair(first_block.end).unwrap(),
            segment.read_pair(second_block.end).unwrap(),
        ];
        assert_eq!(pairs, vec![first, second]);
        assert_eq!(segment.raw_read_bytes(), second_block.end);
        assert!(segment.raw_reads() < 2);
        assert!(segment.consumed());
    }

    #[cfg(unix)]
    #[test]
    fn spill_index_overflow_preserves_lookup_dedup_and_cleanup_without_scans() {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(CANDIDATE_INDEX_BYTES, 64 * 1024 * 1024);
        let canonical = [b"first".as_slice(), b"second", b"third", b"fourth"]
            .map(|bytes| layerfs_content::encode_bytes_object(bytes).unwrap());
        let ids = canonical.each_ref().map(|bytes| ObjectId::for_bytes(bytes));
        let mut objects = DeferredObjectStore::new_all_reachable().unwrap();
        for index in 0..2 {
            objects.put(ids[index], &canonical[index]).unwrap();
        }
        objects.spill().unwrap();
        let DeferredObjects::Spill(spill) = &mut objects.storage else {
            panic!("forced spill");
        };
        assert_eq!(spill.index.as_ref().unwrap().len(), 2);
        assert!(spill.disk_index.is_none());
        spill.index_limit = 2 * 64; // Test-only transition; production remains64MiB.
        objects.put(ids[2], &canonical[2]).unwrap();
        let DeferredObjects::Spill(spill) = &objects.storage else {
            panic!("forced spill");
        };
        assert!(spill.index.is_none());
        assert_eq!(spill.index_bytes, 0);
        assert!(spill.pending.is_empty());
        let disk = spill.disk_index.as_ref().unwrap();
        let index_path = disk.test_path().to_path_buf();
        let payload_path = spill.path.clone();
        assert_eq!(
            std::fs::metadata(&index_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        {
            let connection = disk.test_connection();
            let rows: i64 = connection
                .query_row("SELECT count(*) FROM offsets", [], |row| row.get(0))
                .unwrap();
            assert_eq!(rows, 3);
            let cache: i64 = connection
                .pragma_query_value(None, "cache_size", |row| row.get(0))
                .unwrap();
            let mmap: i64 = connection
                .pragma_query_value(None, "mmap_size", |row| row.get(0))
                .unwrap();
            let spill: i64 = connection
                .pragma_query_value(None, "cache_spill", |row| row.get(0))
                .unwrap();
            let temp: i64 = connection
                .pragma_query_value(None, "temp_store", |row| row.get(0))
                .unwrap();
            assert_eq!((cache, mmap, temp), (-4096, 0, 1));
            assert!(spill > 0);
        }
        spill
            .reader
            .lock()
            .unwrap()
            .seek(SeekFrom::Start(0))
            .unwrap();
        let missing = ObjectId::for_bytes(b"not-present");
        assert_eq!(objects.get(missing).unwrap(), None);
        assert_eq!(
            objects.encoded_length(ids[1]).unwrap(),
            canonical[1].len() as u64
        );
        assert!(
            matches!(objects.encoded_length(missing), Err(StoreError::MissingObject(id)) if id == missing)
        );
        // Missing-ID and length queries must not touch/scan the payload spool.
        assert_eq!(spill.reader.lock().unwrap().stream_position().unwrap(), 0);
        for index in 0..3 {
            assert_eq!(
                objects.get(ids[index]).unwrap(),
                Some(canonical[index].clone())
            );
        }
        objects.put(ids[3], &canonical[3]).unwrap();
        assert_eq!(objects.get(ids[3]).unwrap(), Some(canonical[3].clone()));
        assert_eq!(
            objects.encoded_length(ids[3]).unwrap(),
            canonical[3].len() as u64
        );
        let count = objects.len();
        let written = objects.first_store_write_bytes;
        objects.put(ids[0], &canonical[0]).unwrap();
        assert_eq!(
            (objects.len(), objects.first_store_write_bytes),
            (count, written)
        );
        assert!(matches!(
            objects.put_authenticated(AuthenticatedCanonicalObject(
                CanonicalObject {
                    id: ids[0],
                    bytes: canonical[1].clone()
                },
                PhysicalHints::default()
            )),
            Err(StoreError::Integrity("candidate object collision"))
        ));
        let objects = objects.all_reachable().unwrap();
        assert!(!payload_path.exists());
        assert_eq!(objects.get(ids[3]).unwrap(), Some(canonical[3].clone()));
        drop(objects);
        assert!(!index_path.exists());
        for suffix in ["-journal", "-wal", "-shm"] {
            assert!(!PathBuf::from(format!("{}{suffix}", index_path.display())).exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn spilled_candidate_visits_selected_objects_in_graph_order() {
        let mut segment = DeferredObjectStore::new_all_reachable().unwrap();
        let mut ids = Vec::new();
        for payload in [b"first".as_slice(), b"discarded", b"last"] {
            let bytes = layerfs_content::encode_bytes_object(payload).unwrap();
            let id = ObjectId::for_bytes(&bytes);
            segment.put(id, &bytes).unwrap();
            ids.push(id);
        }
        segment.spill().unwrap();
        let mut segment = segment.all_reachable().unwrap();
        if let DeferredObjects::Spill(spill) = &segment.storage {
            spill
                .reader
                .lock()
                .unwrap()
                .seek(SeekFrom::Start(7))
                .unwrap();
            let mut physical_ids = Vec::new();
            spill
                .visit_ids(&mut |id| {
                    physical_ids.push(id);
                    Ok(())
                })
                .unwrap();
            assert_eq!(physical_ids, ids);
            assert_eq!(
                spill.reader.lock().unwrap().stream_position().unwrap(),
                7,
                "bounded resident offset index needs no payload framing scan"
            );
        }
        let order = IdOrder::Memory(vec![ids[2], ids[0]]);
        let mut visited = Vec::new();
        segment
            .visit_prevalidated_order(&order, &mut |id, bytes| {
                layerfs_content::authenticate_identity(bytes, id)?;
                visited.push(id);
                Ok(())
            })
            .unwrap();
        assert_eq!(visited, vec![ids[2], ids[0]]);
        segment.reachable = order;
        let mut owned = Vec::new();
        segment
            .consume_prevalidated_pages(|page| {
                for object in &page {
                    layerfs_content::authenticate_identity(&object.bytes, object.id)?;
                }
                owned.extend(page.into_iter().map(|object| object.id));
                Ok(())
            })
            .unwrap();
        // The same exact subset is delivered physically, excluding the middle row.
        assert_eq!(owned, vec![ids[0], ids[2]]);
    }

    #[cfg(unix)]
    #[test]
    fn spilled_segment_streams_bounded_pages_once_and_unlinks() {
        let first = layerfs_content::encode_bytes_object(b"first").unwrap();
        let first_id = ObjectId::for_bytes(&first);
        let tail = layerfs_content::encode_bytes_object(b"pending tail").unwrap();
        let tail_id = ObjectId::for_bytes(&tail);
        let mut segment = DeferredObjectStore::new_all_reachable().unwrap();
        segment.put(first_id, &first).unwrap();
        segment.spill().unwrap();
        segment.put(tail_id, &tail).unwrap();
        let path = match &segment.storage {
            DeferredObjects::Spill(spill) => {
                assert!(!spill.pending.is_empty());
                spill.path.clone()
            }
            DeferredObjects::Memory { .. } => panic!("forced segment did not spill"),
        };
        let mut ids = Vec::new();
        segment
            .all_reachable()
            .unwrap()
            .consume_prevalidated_pages(|page| {
                assert!(page.len() <= INITIALIZATION_ADMISSION_BATCH_COUNT);
                assert!(
                    page.iter().map(|object| object.bytes.len()).sum::<usize>()
                        <= ADMISSION_BATCH_BYTES
                );
                ids.extend(page.into_iter().map(|object| object.id));
                Ok(())
            })
            .unwrap();
        assert_eq!(ids, vec![first_id, tail_id]);
        assert!(!path.exists());
    }

    #[test]
    fn segment_admission_deduplicates_across_pending_segments() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-segment-admission-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let db = crate::schema::StoreDb::create(root.join("store.sqlite")).unwrap();

        let shared = layerfs_content::encode_bytes_object(b"shared").unwrap();
        let shared_id = ObjectId::for_bytes(&shared);
        let mut admission = CheckedOutputAdmission::new(&db).unwrap();
        admission
            .admit_worker_segment(sealed_segment(vec![CanonicalObject {
                id: shared_id,
                bytes: shared.clone(),
            }]))
            .unwrap();
        admission
            .admit_worker_segment(sealed_segment(vec![CanonicalObject {
                id: shared_id,
                bytes: shared,
            }]))
            .unwrap();
        let finished = admission.finish().unwrap();
        assert_eq!(finished.receipt, crate::CandidateReceipt::default());
        assert_eq!(finished.final_batch.len(), 1);
        assert_eq!(finished.diagnostics.pending_duplicate_objects, 1);
        assert_eq!(
            finished.diagnostics.pending_duplicate_bytes,
            finished.final_batch[0].bytes.len() as u64
        );
        assert_eq!(finished.diagnostics.collision_checks, 1);

        drop(db);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn direct_admission_honors_every_frozen_count_boundary() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-segment-boundaries-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        for count in [0_usize, 1, 127, 128, 8190, 8191] {
            let db = crate::schema::StoreDb::create(root.join(format!("{count}.sqlite"))).unwrap();
            let objects = (0..count)
                .map(|index| {
                    let bytes = layerfs_content::encode_bytes_object(&(index as u64).to_be_bytes())
                        .unwrap();
                    CanonicalObject {
                        id: ObjectId::for_bytes(&bytes),
                        bytes,
                    }
                })
                .collect::<Vec<_>>();
            let expected_bytes = objects
                .iter()
                .map(|object| object.bytes.len() as u64)
                .sum::<u64>();
            assert!(expected_bytes < ADMISSION_BATCH_BYTES as u64);
            let mut admission = CheckedOutputAdmission::new(&db).unwrap();
            admission
                .admit_worker_segment(sealed_segment(objects))
                .unwrap();
            let (receipt, _) = finish_segment_admission(&db, admission);
            assert_eq!(receipt.candidate_objects, count as u64);
            assert_eq!(receipt.candidate_bytes, expected_bytes);
            assert_eq!(receipt.max_transaction_objects, count as u64);
            assert_eq!(receipt.max_transaction_bytes, expected_bytes);
            drop(db);
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn direct_admission_is_independent_of_segment_order() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-segment-order-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let canonical = [
            b"shared".as_slice(),
            b"left".as_slice(),
            b"right".as_slice(),
        ]
        .map(|payload| layerfs_content::encode_bytes_object(payload).unwrap());
        let ids = canonical
            .iter()
            .map(|bytes| ObjectId::for_bytes(bytes))
            .collect::<Vec<_>>();
        let mut results = Vec::new();
        for reverse in [false, true] {
            let db =
                crate::schema::StoreDb::create(root.join(format!("{reverse}.sqlite"))).unwrap();
            let left = sealed_segment(vec![
                CanonicalObject {
                    id: ids[0],
                    bytes: canonical[0].clone(),
                },
                CanonicalObject {
                    id: ids[1],
                    bytes: canonical[1].clone(),
                },
            ]);
            let right = sealed_segment(vec![
                CanonicalObject {
                    id: ids[0],
                    bytes: canonical[0].clone(),
                },
                CanonicalObject {
                    id: ids[2],
                    bytes: canonical[2].clone(),
                },
            ]);
            let mut admission = CheckedOutputAdmission::new(&db).unwrap();
            let mut segments = if reverse {
                vec![right, left]
            } else {
                vec![left, right]
            };
            for segment in segments.drain(..) {
                admission.admit_worker_segment(segment).unwrap();
            }
            let (receipt, _) = finish_segment_admission(&db, admission);
            let mut sorted_ids = ids.to_vec();
            sorted_ids.sort_unstable();
            let rows = db.read_object_rows(&sorted_ids).unwrap();
            results.push((receipt, rows));
            drop(db);
        }
        assert_eq!(results[0], results[1]);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn segment_admission_deduplicates_after_a_batch_boundary() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-segment-boundary-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let db = crate::schema::StoreDb::create(root.join("store.sqlite")).unwrap();
        let mut objects = Vec::new();
        for index in 0_u64..130 {
            let mut payload = vec![index as u8; layerfs_content::file::cdc::MAXIMUM_CHUNK_BYTES];
            payload[..8].copy_from_slice(&index.to_le_bytes());
            let bytes = layerfs_content::file::extent_codec::encode_chunk_object(&payload).unwrap();
            objects.push(CanonicalObject {
                id: ObjectId::for_bytes(&bytes),
                bytes,
            });
        }
        let duplicate = objects[0].clone();
        let duplicate_id = duplicate.id;
        let expected_objects = objects.len() as u64;
        let mut admission = CheckedOutputAdmission::new(&db).unwrap();
        admission
            .admit_worker_segment(sealed_segment(objects))
            .unwrap();
        assert!(admission.receipt.admission_transactions > 0);
        admission
            .admit_worker_segment(sealed_segment(vec![duplicate]))
            .unwrap();
        let finished = admission.finish().unwrap();
        let mut statement_number = finished.statement_number;
        let (_, metrics) = PreparedAdmission::prepare_missing(&db, finished.final_batch)
            .unwrap()
            .publish(&db, &mut statement_number, |_, _, _| Ok(()))
            .unwrap();
        assert_eq!(finished.diagnostics.cross_batch_skipped_objects, 1);
        assert_eq!(
            finished.receipt.batch_inserted_objects + metrics.insert.objects,
            expected_objects
        );
        let forged = layerfs_content::file::extent_codec::encode_chunk_object(
            &vec![255; layerfs_content::file::cdc::MAXIMUM_CHUNK_BYTES],
        )
        .unwrap();
        assert!(AuthenticatedCanonicalObject::new(forged, Some(duplicate_id)).is_err());
        drop(db);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shared_admission_keeps_every_object_transaction_below_the_frozen_bounds() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-bounded-admission-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let db = crate::schema::StoreDb::create(root.join("store.sqlite")).unwrap();
        let count = 2 * ADMISSION_BATCH_COUNT as u64 + 46;
        let mut admission = CheckedOutputAdmission::new(&db).unwrap();
        for index in 0..count {
            let bytes = layerfs_content::encode_bytes_object(&index.to_le_bytes()).unwrap();
            admission
                .admit_object(AuthenticatedCanonicalObject::new(bytes, None).unwrap())
                .unwrap();
        }
        let (receipt, _) = finish_segment_admission(&db, admission);
        assert_eq!(receipt.candidate_objects, count);
        assert_eq!(receipt.inserted_objects, count);
        assert!(receipt.max_transaction_objects <= ADMISSION_BATCH_COUNT as u64);
        assert!(receipt.max_transaction_bytes < OBJECT_PAGE_BYTES as u64);
        drop(db);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn finished_candidate_payload_spill_is_private_read_only_and_authenticated() {
        use std::os::unix::fs::PermissionsExt;

        let mut objects = DeferredObjectStore::new().unwrap();
        let mut root = None;
        let chunk_bytes = layerfs_content::file::cdc::MAXIMUM_CHUNK_BYTES;
        for index in 0..=CANDIDATE_MEMORY_BYTES / chunk_bytes + 1 {
            let mut payload = vec![index as u8; chunk_bytes];
            payload[..8].copy_from_slice(&(index as u64).to_be_bytes());
            let canonical =
                layerfs_content::file::extent_codec::encode_chunk_object(&payload).unwrap();
            let id = ObjectId::for_bytes(&canonical);
            objects.put(id, &canonical).unwrap();
            root = Some(id);
        }
        let path = match &objects.storage {
            DeferredObjects::Spill(spill) => spill.path.clone(),
            DeferredObjects::Memory { .. } => panic!("candidate did not spill"),
        };
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let root = root.unwrap();
        let objects = objects.reachable_from(root).unwrap();
        assert!(!path.exists());
        let spill = match &objects.storage {
            DeferredObjects::Spill(spill) => spill,
            DeferredObjects::Memory { .. } => panic!("candidate did not spill"),
        };
        assert!(spill.writer.is_none());
        assert!(spill.reader.lock().unwrap().write_all(&[0]).is_err());
        let mut missing = SpillableObjectSet::empty().unwrap();
        missing.insert_page(&[root]).unwrap();
        let order = objects.order_missing(&missing, missing.count).unwrap();
        let mut visited = Vec::new();
        objects
            .visit_prevalidated_order(&order, &mut |id, bytes| {
                layerfs_content::authenticate_identity(bytes, id)?;
                visited.push(id);
                Ok(())
            })
            .unwrap();
        assert_eq!(visited, vec![root]);
        let canonical = objects.read_object(root).unwrap();
        layerfs_content::authenticate_identity(&canonical, root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn prevalidated_transfer_seals_a_nonempty_spill_tail() {
        let mut objects = DeferredObjectStore::new_all_reachable().unwrap();
        let chunk_bytes = layerfs_content::file::cdc::MAXIMUM_CHUNK_BYTES;
        for index in 0..=CANDIDATE_MEMORY_BYTES / chunk_bytes + 1 {
            let mut payload = vec![index as u8; chunk_bytes];
            payload[..8].copy_from_slice(&(index as u64).to_be_bytes());
            let canonical =
                layerfs_content::file::extent_codec::encode_chunk_object(&payload).unwrap();
            objects
                .put(ObjectId::for_bytes(&canonical), &canonical)
                .unwrap();
        }
        let tail = layerfs_content::encode_bytes_object(b"pending tail").unwrap();
        let tail_id = ObjectId::for_bytes(&tail);
        objects.put(tail_id, &tail).unwrap();
        let expected_count = objects.len();

        let (path, pending_bytes) = match &objects.storage {
            DeferredObjects::Spill(spill) => (spill.path.clone(), spill.pending.len()),
            DeferredObjects::Memory { .. } => panic!("candidate did not spill"),
        };
        assert!(pending_bytes > 0);

        let objects = ObjectBuffer {
            source: None,
            objects,
        }
        .into_prevalidated()
        .unwrap();
        let spill = match &objects.storage {
            DeferredObjects::Spill(spill) => spill,
            DeferredObjects::Memory { .. } => panic!("candidate did not spill"),
        };
        assert!(spill.writer.is_none());
        assert!(spill.pending.is_empty());
        assert!(!path.exists());

        let mut receiver = ObjectBuffer::empty_all_reachable().unwrap();
        receiver.merge_prevalidated(objects).unwrap();
        assert_eq!(receiver.objects.len(), expected_count);
        let transferred_tail = receiver.objects.read_object(tail_id).unwrap();
        assert_eq!(transferred_tail, tail);
        layerfs_content::authenticate_identity(&transferred_tail, tail_id).unwrap();
    }

    #[test]
    fn resumable_transfer_keeps_a_spill_writable() {
        let mut objects = DeferredObjectStore::new_all_reachable().unwrap();
        let chunk_bytes = layerfs_content::file::cdc::MAXIMUM_CHUNK_BYTES;
        for index in 0..=CANDIDATE_MEMORY_BYTES / chunk_bytes + 1 {
            let mut payload = vec![index as u8; chunk_bytes];
            payload[..8].copy_from_slice(&(index as u64).to_be_bytes());
            let canonical =
                layerfs_content::file::extent_codec::encode_chunk_object(&payload).unwrap();
            objects
                .put(ObjectId::for_bytes(&canonical), &canonical)
                .unwrap();
        }
        assert!(matches!(objects.storage, DeferredObjects::Spill(_)));

        let objects = ObjectBuffer {
            source: None,
            objects,
        }
        .into_resumable()
        .unwrap();
        let mut resumed = ObjectBuffer {
            source: None,
            objects,
        };
        let tail = layerfs_content::encode_bytes_object(b"resumed tail").unwrap();
        let tail_id = ObjectId::for_bytes(&tail);
        resumed.objects.put(tail_id, &tail).unwrap();
        assert_eq!(resumed.objects.read_object(tail_id).unwrap(), tail);
    }

    #[cfg(feature = "test-instrumentation")]
    #[test]
    fn durable_batches_hash_unique_rows_once_and_move_on_last_use() {
        let root = std::env::temp_dir().join(format!(
            "layerfs-read-batch-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db = crate::schema::StoreDb::create(root.join("store.sqlite")).unwrap();
        let first = layerfs_content::encode_bytes_object(b"first").unwrap();
        let second = layerfs_content::encode_bytes_object(b"second").unwrap();
        let corrupt = layerfs_content::encode_bytes_object(b"corrupt-id").unwrap();
        let first_id = ObjectId::for_bytes(&first);
        let second_id = ObjectId::for_bytes(&second);
        let corrupt_id = ObjectId::for_bytes(&corrupt);
        {
            let connection = db.writer().unwrap();
            for (id, bytes) in [
                (first_id, first.as_slice()),
                (second_id, second.as_slice()),
                (corrupt_id, second.as_slice()),
            ] {
                connection
                    .execute(
                        crate::statements::objects::INSERT,
                        rusqlite::params![id.as_bytes().as_slice(), bytes],
                    )
                    .unwrap();
            }
        }

        reset_read_batch_counters();
        let rows = db
            .read_object_rows(&[first_id, second_id, first_id])
            .unwrap();
        assert_eq!(
            rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![first_id, second_id, first_id]
        );
        assert_eq!(
            read_batch_counters(),
            ReadBatchCounters {
                unique_hashes: 2,
                cloned_bytes: first.len() as u64,
            }
        );
        reset_read_batch_counters();
        crate::schema::reset_sql_trace();
        assert!(db.read_object_rows(&[]).unwrap().is_empty());
        assert!(crate::schema::sql_trace().is_empty());
        let singleton = db.read_object_rows(&[first_id]).unwrap();
        assert_eq!(
            singleton,
            vec![CanonicalObject {
                id: first_id,
                bytes: first.clone()
            }]
        );
        assert_eq!(
            read_batch_counters(),
            ReadBatchCounters {
                unique_hashes: 1,
                cloned_bytes: 0
            }
        );
        let trace = crate::schema::sql_trace();
        assert_eq!(trace.len(), 1);
        assert!(trace[0].contains("WHERE object_id ="));
        assert!(!trace[0].contains(" IN "));
        assert!(matches!(
            db.read_object_rows(&[ObjectId::for_bytes(b"missing")]),
            Err(StoreError::Integrity("visible object cardinality"))
        ));
        assert!(db.read_object_rows(&[corrupt_id]).is_err());

        struct Claimed(Vec<CanonicalObject>);
        impl ObjectSource for Claimed {
            fn read_object(&self, _: ObjectId) -> Result<Vec<u8>> {
                Err(StoreError::Integrity("unexpected single read"))
            }

            fn read_authenticated_objects(&self, _: &[ObjectId]) -> Result<Vec<CanonicalObject>> {
                Ok(self.0.clone())
            }
        }
        let reversed = Claimed(vec![
            CanonicalObject {
                id: second_id,
                bytes: second.clone(),
            },
            CanonicalObject {
                id: first_id,
                bytes: first.clone(),
            },
        ]);
        assert!(
            CoreReader(&reversed)
                .get_authenticated_batch(&[first_id, second_id], |_, _| Ok(()))
                .is_err()
        );
        let short = Claimed(vec![CanonicalObject {
            id: first_id,
            bytes: first,
        }]);
        assert!(
            CoreReader(&short)
                .get_authenticated_batch(&[first_id, second_id], |_, _| Ok(()))
                .is_err()
        );
        struct Untrusted(Vec<u8>);
        impl ObjectSource for Untrusted {
            fn read_object(&self, _: ObjectId) -> Result<Vec<u8>> {
                Ok(self.0.clone())
            }
        }
        assert!(
            CoreReader(&Untrusted(second))
                .get_authenticated_batch(&[corrupt_id], |_, _| Ok(()))
                .is_err()
        );

        drop(db);
        std::fs::remove_dir_all(root).unwrap();
    }
}

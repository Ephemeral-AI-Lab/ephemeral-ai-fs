# FUSE redesign requirements for accelerating LayerFS Exec

Date: 2026-09-06. Status: design acceptance checklist from the user discussion; not an implementation report, released behavior, or performance proof. Check an item only when the delivered implementation and appropriate code/test/measurement references support it. Existing proposal documents are design inputs, not evidence that requirements are satisfied.

Tracking context: [#48 Exec/FUSE](https://github.com/Ephemeral-AI-Lab/layerfs/issues/48), [#47 integration](https://github.com/Ephemeral-AI-Lab/layerfs/issues/47), [#49 shared producers](https://github.com/Ephemeral-AI-Lab/layerfs/issues/49).

## Objective

Make ordinary filesystem operations cheaper while preserving one coherent live Workspace. Reduced Exec time must come from less work or fewer blocking dependencies, not weakened semantics, unbounded buffering, or shifting the same cost into Commit/End.

Lock/load amendment, 2026-09-06: evaluate the replacement against the retained optimized #48 implementation for contention, progress and overload behavior. Local execution must replace implicit request/reply throttling with explicit bounded admission and backpressure. CPU concurrency/work and memory ownership must have finite configured bounds; neither a Session abstraction nor fewer RPCs establishes safer load handling.

```text
Linux FUSE / macFUSE / Windows adapter
                    |
                    v
          Shared live Workspace operation core
          identities, bindings, handles, ordering,
          mutable ranges and bounded ownership
                    |
                    | explicit synchronization / Commit boundary
                    v
          Existing shared canonical construction
          and checked persistence pipeline
                    |
                    v
                Host Store
```

The portable principle is to execute ordinary operations beside the application's filesystem driver when valid authority and state are available. It is not a requirement that every deployment execute inside Linux. Driver-specific behavior remains in adapters; future-platform support is not claimed merely because the core is shared.

## 1. Shared architecture and code reuse

- [ ] Platform adapters call one shared live-Workspace operation core; they do not implement separate mutation engines.
- [ ] Reuse existing PieceTree, COW, rope/extent, CDC, CAS, deduplication and checked persistence mechanisms.
- [ ] Preserve #49's applicable shared producer execution and completed-file handling where construction is required; verify actual adopted source rather than assuming issue completion proves integration.
- [ ] Learn from applicable #48 mechanisms and safety checks, distinguishing retained from rejected experiments; implement on current main without requiring migration of the research Exec patch, and preserve separately owned work.
- [ ] Keep mounting, request/reply translation, permission mapping, cache notifications and platform capabilities in the adapters.
- [ ] No optimization engine is selected by benchmark/application name, file-size bucket or deletion density. Differences arising from real operation semantics remain explicit.
- [ ] Remove superseded implementations only after transferring necessary responsibilities and all callers; no permanent old/new engine facade.
- [ ] Implement for the actual Linux deployment first while keeping future adapter boundaries clear; do not scaffold unused platform backends or a generic framework without real callers.

## 2. Communication and the application critical path

- [ ] Execute an operation locally when the execution-side owner has sufficient authoritative state.
- [ ] Avoid a host round trip merely to update metadata or a binding already owned locally.
- [ ] Acquire missing directory, inode and content information lazily in bounded batches; mounting must not require materializing the entire workspace.
- [ ] Reuse immutable canonical content and avoid retransmitting unchanged bytes.
- [ ] Transfer new bytes and changes through bounded, ordered delivery, with explicit acknowledgment and retained ownership.
- [ ] Synchronize at real dependency, observation, ownership-transfer or Commit boundaries rather than automatically after every operation.
- [ ] Local deployments may use direct calls; do not force loopback RPC solely for uniform implementation appearance.
- [ ] Demonstrate lower application waiting and complete elapsed time; reduced RPC counts alone are not a performance result.

## 3. Authority, acknowledgment and visibility

- [ ] Every mutable inode, binding and handle has a clear authoritative owner.
- [ ] Successful local acknowledgment means the owner has validated and applied the operation, not merely queued it for a remote decision that can later reject it.
- [ ] Check available resources, permissions, argument validity and inode/handle lifetime before acknowledgment.
- [ ] Cached attributes alone never grant mutation authority.
- [ ] Subsequent filesystem operations observe acknowledged changes correctly.
- [ ] Host SDK reads/edits, `session()`, `diff()`, synchronization and Commit observe the same state through a defined synchronization contract.
- [ ] Ownership transfer prevents simultaneous conflicting authorities and makes stale grants/state unusable.
- [ ] Lost replies and retries cannot duplicate mutations or silently discard acknowledged changes.
- [ ] Authentication and workspace/mount authorization apply to acquisition, mutation, content access and synchronization requests; validate lengths and allocation bounds at trust boundaries.

## 4. Concurrent access and multiple mounted workspaces

- [ ] Multiple processes can use one workspace concurrently and remain alive across a successful Commit; command activity and ordinary open file handles must not gate Commit success.
- [ ] Independent operations can progress concurrently; conflicting operations have defined ordering and atomicity.
- [ ] Design for at least 100 independent live agent workspaces with isolated state/handles/errors/accounting and shared bounded execution resources; assess this scale using source accounting and existing lower-volume evidence for now, with physical 100-workspace qualification deferred and not a current completion gate.
- [ ] Mutable cache and handle keys distinguish workspace and mount lifetime; NodeId alone must not accidentally alias state across workspaces.
- [ ] All hardlink aliases of an inode share content and metadata state.
- [ ] Cross-directory rename and other multi-object operations use a deliberate atomicity/locking protocol.
- [ ] A slow, full or disconnected workspace cannot indefinitely block unrelated workspaces.
- [ ] Multiple views/mounts of the same workspace have an explicit support contract; cross-machine writable coherence is not implicitly claimed.
- [ ] One authoritative owner does not imply a global worker/lock serializing all mounted workspaces.

## 5. Filesystem behavior and kernel coherence

- [ ] Preserve supported create/exclusive-create, read/write, truncate, append, rename, link, unlink and directory operations.
- [ ] Preserve required open-handle behavior across rename/unlink; retain backing while readers, mappings or operations require it.
- [ ] Preserve correct partial writes, quota errors, permission errors, missing-file errors and unsupported-operation results.
- [ ] Explicitly define and test locking, memory mapping, executable permissions, symlinks and metadata required by supported tools; do not claim full application compatibility from create/delete benchmarks alone.
- [ ] Directory enumeration and cookies remain correct under the supported concurrent-mutation model.
- [ ] Kernel name, attribute and page caches remain coherent after filesystem mutations, SDK edits, ownership transfer and Commit.
- [ ] Callback suppression, TTL changes and writeback require a valid coherence/error-delivery argument; no permission bypass or blind cache extension.
- [ ] Ordinary shell/editor/installer behavior works through any permitted launcher accessing the mount. External processes must still be accounted for by the chosen synchronization contract; managed-Exec tracking alone does not prove no writer exists.
- [ ] Platform-specific filename, deletion, security and notification semantics are mapped explicitly; native Windows behavior is not assumed identical to POSIX.

## 6. File representation, Commit and continuation

- [ ] Writes never modify canonical objects reachable from an existing snapshot.
- [ ] Record live edits with the existing mutable range representation; retain unchanged extents and sparse/zero semantics.
- [ ] Do not force full CDC/CAS construction synchronously onto every small write when recording a mutable change suffices.
- [ ] Temporary and overwritten intermediate states need not become published objects.
- [ ] Only valid final objects enter the published snapshot; incremental provisional state retains required readable storage and finality checks.
- [ ] Commit consumes known changed state without rediscovering the entire workspace.
- [ ] Commit establishes a consistent filesystem-operation boundary independently of shell/Exec tracking. Order or pause conflicting mutations, drain admitted work, preserve process/mount/handle continuity and later dirty writes; never require command exit or publish torn state.
- [ ] Clearing published changes cannot clear post-boundary changes.
- [ ] Successful Commit advances existing live nodes/handles instead of reconstructing the Workspace.
- [ ] Expected-head/base checks prevent silent overwrites by competing workspaces. Final references preserve unseen aliases and add/remove ordering.
- [ ] Publication/install retry completes the exact retained result without creating another Commit.
- [ ] Old snapshots remain readable after new edits, commits, deletion and temporary-storage reclamation.

## 7. Failure, durability and cleanup

- [ ] Document the guarantees of successful write, close, file/directory synchronization and Commit separately.
- [ ] Distinguish application, daemon, transport, host and power-loss failure. Preserve the actual product contract without silently weakening or overstating durability.
- [ ] Interrupted transfer/construction never appears as a successfully published snapshot.
- [ ] Producer/consumer failure, cancellation and finishing errors drain or disconnect safely and join all workers.
- [ ] Unmount and shutdown have bounded, defined behavior with active requests, open handles and retained mappings.
- [ ] Failed/disconnected workspaces retain or release acknowledged dirty data under an explicit recovery contract.
- [ ] Repeated commits and failures do not accumulate unbounded segments, descriptors, queued operations or cleanup debt.
- [ ] Physical retirement stays accounted for within the declared lifecycle; do not move cleanup after timing or uncharge still-live storage.

## 8. Resource limits, synchronization and privileges

- [ ] Account live state, caches, dirty ranges, partial batches, queues, producer scratch, pending results and retained sources simultaneously.
- [ ] Account physical backing, open descriptors and retained readers for their actual lifetimes, including anonymous files.
- [ ] Enforce per-workspace and aggregate limits; adding workers or mounts does not silently multiply a previous allowance.
- [ ] Bound backpressure and ensure it remains safe during cancellation/disconnect/shutdown.
- [ ] Reserve the required bounded pending-operation/byte capacity before accepting a mutation that depends on it; state must not be changed successfully and then become impossible to record because its queue is full. Release or transfer reservations on every outcome.
- [ ] Do not hold state locks while waiting for remote communication, queue capacity or worker completion. Document distinct ordering/admission mechanisms and prove their progress behavior.
- [ ] Multi-object locking follows a consistent acquisition order; avoid lock inversion across callbacks, transport, observers and shutdown.
- [ ] Test races, deadlock scenarios, starvation and error paths. Actors, queues or lock-free code are not assumed risk-free.
- [ ] The core redesign requires no additional privileges beyond the existing mount/runtime setup.
- [ ] Optional platform accelerations do not become mandatory dependencies of the portable core; platform-specific privilege requirements remain explicit.
- [ ] SQLite stays outside Docker and no host-data mount is introduced. Any proposed execution-side temporary backing has explicit ownership, resource accounting and compatibility with the chosen topology.

### 8.1 Bounded CPU and work admission

- [ ] Declare finite limits for execution workers, outstanding requests, in-flight I/O and concurrently active mounts/operations, both per workspace and across the daemon. Include helper/background/construction work; no thread or unbounded task creation per filesystem call.
- [ ] Divide the configured CPU concurrency allowance among active workspaces and worker stages instead of granting every mount the whole allowance. Independent progress must not require oversubscribing without bound.
- [ ] Bound work admitted per batch/request by bytes, entries and applicable traversal limits. Avoid repeated full-prefix scans, unbounded retry loops and unbounded recursive work; large supported operations must make bounded, cancellation-aware progress.
- [ ] Block or await capacity efficiently under pressure; no busy polling, spin-until-ready loops or retry storms when the host/consumer is slow or disconnected.
- [ ] Use bounded retry counts/work budgets and the existing failure contract. Cancellation and shutdown must stop admission of new work and leave already-owned work recoverable or releasable.
- [ ] Record CPU time per completed workload, worker utilization, queue waits and any OS quota throttling separately for host and container. A faster latency obtained only by an undisclosed CPU increase is not a demonstrated efficiency gain.
- [ ] Distinguish concurrency/work bounds from a hard CPU-time quota. A fixed worker count does not itself enforce a percentage/core-time cap. Where a hard quota is required, name and verify the deployment's actual enforcement mechanism without expanding core privileges; otherwise report only the bounds actually enforced.
- [ ] Preserve the current Linux container CPU/PID limits. The macOS host is outside that cgroup; define its own worker/admission budget and do not describe host CPU as container-capped.

### 8.2 Bounded memory and retained backing

- [ ] Specify numeric byte/count limits and checked accounting before implementation for caches, dirty metadata/ranges, input/output queues, partial frames, worker scratch, result/reorder storage, active handles and mount records. Reuse existing limits where applicable; no implicit unlimited defaults.
- [ ] Enforce the aggregate allocation/reservation bound before allocation, decoding or acknowledgment. Include container and host ownership during transfer, when both copies can coexist; do not assume ownership has moved before it actually has.
- [ ] Bound out-of-order completions and blocked requests as well as actively processed data. A slow first result or host consumer must not cause an unlimited waiting-results map.
- [ ] Evict only clean reacquirable cache entries. Dirty state, acknowledged pending writes and open-reader backing remain charged until safely transferred or released; reaching the limit must block admission or return the defined error without data loss.
- [ ] Charge retained physical segment capacity and required descriptors, including partly obsolete segments pinned by a small live range. Repeated write/Commit/disconnect cycles must not accumulate uncharged storage or memory.
- [ ] Make simultaneous ownership explicit: live state + caches + all worker scratch + partial/queued batches + results + retained sources + control reserve must fit the declared application-managed memory budget.
- [ ] Separately disclose allocator overhead, SQLite/cache allocations, thread stacks, kernel page cache, socket buffers and VM/system memory. An application buffer counter is not a whole-process/system hard memory bound; verify an actual deployment-enforced envelope when that claim is required.
- [ ] Preserve existing container memory/no-swap limits and report host RSS/footprint separately. Admission must react to its defined limits before relying on OOM termination as flow control.

### 8.3 Lock contention, overload and fairness acceptance

- [ ] Avoid a global state mutex or ordered queue that lets unrelated workspaces inherit one slow workspace's wait. Keep required per-inode/directory/session ordering explicit without serializing all mounts.
- [ ] Document the lock/wait dependency graph, including FUSE callbacks, host observers, data transfer, Commit and teardown. Specifically prevent a host Workspace lock from being held while waiting for a Session drain whose application needs that same lock.
- [ ] Ensure cancellation, drain and teardown can progress when ordinary queues are full. Reserve the necessary bounded control capacity or otherwise prove progress; control operations must not overtake required predecessor mutations incorrectly.
- [ ] Apply bounded scheduling/fairness so a hot workspace cannot indefinitely monopolize workers or buffers. A cap on total threads alone is not a fairness guarantee.
- [ ] Compare the retained #48 baseline and replacement under the same workload, concurrency and resource settings: normal load, saturated load, slow/stalled host, queue exhaustion and transport/consumer failure. Record throughput, latency distribution where applicable, lock/queue waits, CPU, peak memory, retained storage and progress of an unrelated workspace.
- [ ] Exercise opposing cross-directory operations, host observations during drain, cancellation/unmount with active requests, and repeated recovery cycles. Demonstrate no deadlock, leaked ownership or starvation in these scenarios; do not claim universal risk-free concurrency from finite tests.
- [ ] An increased tail latency, contention level or resource footprint must be explained and evaluated against the declared limits and intended throughput benefit. Do not accept a rewrite solely because single-thread average latency improved while overload behavior worsened.

## 9. Evidence, rollout and fast iteration

- [ ] Exercise real filesystem operations through the supported mount and public operation surface.
- [ ] Where attribution is missing, distinguish kernel/daemon dispatch, queue wait, local processing, host work, reply and application resumption. Unattributed time is not automatically removable overhead.
- [ ] Measure single-process and concurrent access, including several mounted workspaces.
- [ ] Check representative installation/editor behavior as well as synthetic file operations.
- [ ] Freeze workload identity, counts, bytes, metadata and synchronization obligations within a comparison; version changed recipes and preserve historical evidence.
- [ ] Measure complete Exec/Commit/End, so buffering does not merely transfer work between phases.
- [ ] Report CPU, memory, physical storage, latency and cleanup, with host/container scopes separated and nested timers identified.
- [ ] Use focused semantic checks and one selected serial performance sample per iteration. Independent proofs remain at the agreed final stage, with the existing 45-second work / 59-second hard end-to-end limit; no exhaustive per-iteration proof campaign.
- [ ] Use the shared measurement lock and preserve other tasks' source, containers, caches and artifacts.
- [ ] Validate the actually integrated #48/#49/redesign product before claiming combined performance; do not add favorable phase numbers from separate branches.
- [ ] Record actual shared callers, removed dependencies/methods, rejected experiments, exact source identities and remaining limitations.
- [ ] Set performance acceptance from the selected workload contract; this checklist does not invent a new universal latency target or release gate.

## Decisions required before implementation

| Decision | Required written answer |
|---|---|
| Live authority | What state is owned at the execution side, what remains Store-side, and how ownership changes |
| Operation ordering | Which operations can run concurrently and where conflicting operations become ordered |
| Cache coherence | Which kernel/daemon caches exist and what invalidates each after every supported mutation/observer path |
| Commit boundary | How stable state is captured, what active writers observe, and how post-boundary changes survive |
| Acknowledgment/recovery | What success means at each boundary and what happens after transport/daemon/host failure |
| Resource ownership | Where bytes, handles and queued work are charged, including across mounts and failures |
| CPU/work envelope | Numeric worker/request/in-flight/mount limits, aggregate sharing, per-operation work limits and any actual OS-enforced quota scope |
| Memory envelope | Numeric component and aggregate budgets, reservation/eviction/backpressure behavior, physical retention and memory outside application counters |
| Lock/load acceptance | Lock/wait graph, fairness/progress under full queues and stalled hosts, and matched baseline/replacement overload checks |

Resolve these before tuning worker counts, batch sizes or kernel-specific acceleration. Proposed architecture documents can suggest answers; only implemented and checked behavior satisfies the checklist.

## References

- [Shared producer implementation plan](issue49-shared-producers-plan.md): reuse and task-lifetime design input; not proof of FUSE semantics.
- [Exec root-cause research](issue47-exec-root-cause-research.md): historical measured-source findings, with their stated limits.
- [Linux FUSE overview](https://www.kernel.org/doc/html/latest/filesystems/fuse/fuse.html): kernel/userspace request and mount model.
- [Linux FUSE I/O modes](https://www.kernel.org/doc/html/latest/filesystems/fuse/fuse-io.html): direct/cached I/O and writeback coherence assumptions.
- [WinFsp native API versus FUSE](https://github.com/winfsp/winfsp/wiki/Native-API-vs-FUSE): platform-semantic differences to consider for a future Windows adapter.

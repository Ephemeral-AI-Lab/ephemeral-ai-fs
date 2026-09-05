# FUSE rewrite checklist coverage map

Status: proposed design traceability; **all 109 requirements remain unchecked**. This map is not an implementation or test PASS. Read the [specification](fuse-exec-rewrite-spec.md) alongside the [normative checklist](fuse-exec-redesign-requirements.md).

The requirement text is reproduced exactly below, with Markdown table escaping. IDs identify each checkbox in checklist order. S references identify specification sections; T references identify the focused acceptance groups in S11. The implementation owner must record source-bound checks and limitations before changing any checklist status. A mapped obligation still fails acceptance if its implementation, compatibility inventory, or required evidence is missing.

Checklist SHA-256 at generation: `777545abe39f40fec47eefa73600ffe51f7be656a23140de4218b56c7d7cea37`. Regenerate/check this map when requirements change; do not let a stale map imply coverage.

| Requirement | Exact checklist obligation | Spec sections | Required checks |
|---|---|---|---|
| R1.01 | Platform adapters call one shared live-Workspace operation core; they do not implement separate mutation engines. | S1, S3, S10 | T2, T7 |
| R1.02 | Reuse existing PieceTree, COW, rope/extent, CDC, CAS, deduplication and checked persistence mechanisms. | S1, S3, S10 | T2, T7 |
| R1.03 | Preserve #49's applicable shared producer execution and completed-file handling where construction is required; verify actual adopted source rather than assuming issue completion proves integration. | S1, S3, S10 | T2, T7 |
| R1.04 | Incorporate applicable #48 improvements before replacing equivalent behavior; preserve source/patch ownership and distinguish retained from rejected experiments. | S1, S3, S10 | T2, T7 |
| R1.05 | Keep mounting, request/reply translation, permission mapping, cache notifications and platform capabilities in the adapters. | S1, S3, S10 | T2, T7 |
| R1.06 | No optimization engine is selected by benchmark/application name, file-size bucket or deletion density. Differences arising from real operation semantics remain explicit. | S1, S3, S10 | T2, T7 |
| R1.07 | Remove superseded implementations only after transferring necessary responsibilities and all callers; no permanent old/new engine facade. | S1, S3, S10 | T2, T7 |
| R1.08 | Implement for the actual Linux deployment first while keeping future adapter boundaries clear; do not scaffold unused platform backends or a generic framework without real callers. | S1, S3, S10 | T2, T7 |
| R2.01 | Execute an operation locally when the execution-side owner has sufficient authoritative state. | S3, S4, S5, S11 | T3, T6 |
| R2.02 | Avoid a host round trip merely to update metadata or a binding already owned locally. | S3, S4, S5, S11 | T3, T6 |
| R2.03 | Acquire missing directory, inode and content information lazily in bounded batches; mounting must not require materializing the entire workspace. | S3, S4, S5, S11 | T3, T6 |
| R2.04 | Reuse immutable canonical content and avoid retransmitting unchanged bytes. | S3, S4, S5, S11 | T3, T6 |
| R2.05 | Transfer new bytes and changes through bounded, ordered delivery, with explicit acknowledgment and retained ownership. | S3, S4, S5, S11 | T3, T6 |
| R2.06 | Synchronize at real dependency, observation, ownership-transfer or Commit boundaries rather than automatically after every operation. | S3, S4, S5, S11 | T3, T6 |
| R2.07 | Local deployments may use direct calls; do not force loopback RPC solely for uniform implementation appearance. | S3, S4, S5, S11 | T3, T6 |
| R2.08 | Demonstrate lower application waiting and complete elapsed time; reduced RPC counts alone are not a performance result. | S3, S4, S5, S11 | T3, T6 |
| R3.01 | Every mutable inode, binding and handle has a clear authoritative owner. | S4, S5, S9 | T1, T3, T4 |
| R3.02 | Successful local acknowledgment means the owner has validated and applied the operation, not merely queued it for a remote decision that can later reject it. | S4, S5, S9 | T1, T3, T4 |
| R3.03 | Check available resources, permissions, argument validity and inode/handle lifetime before acknowledgment. | S4, S5, S9 | T1, T3, T4 |
| R3.04 | Cached attributes alone never grant mutation authority. | S4, S5, S9 | T1, T3, T4 |
| R3.05 | Subsequent filesystem operations observe acknowledged changes correctly. | S4, S5, S9 | T1, T3, T4 |
| R3.06 | Host SDK reads/edits, `session()`, `diff()`, synchronization and Commit observe the same state through a defined synchronization contract. | S4, S5, S9 | T1, T3, T4 |
| R3.07 | Ownership transfer prevents simultaneous conflicting authorities and makes stale grants/state unusable. | S4, S5, S9 | T1, T3, T4 |
| R3.08 | Lost replies and retries cannot duplicate mutations or silently discard acknowledged changes. | S4, S5, S9 | T1, T3, T4 |
| R3.09 | Authentication and workspace/mount authorization apply to acquisition, mutation, content access and synchronization requests; validate lengths and allocation bounds at trust boundaries. | S4, S5, S9 | T1, T3, T4 |
| R4.01 | Multiple processes can use one workspace concurrently. | S1, S4, S6, S9 | T1, T3, T5 |
| R4.02 | Independent operations can progress concurrently; conflicting operations have defined ordering and atomicity. | S1, S4, S6, S9 | T1, T3, T5 |
| R4.03 | Support at least 100 simultaneously mounted independent live workspaces per daemon, with concurrent clients, isolated mutable state/handles/errors/accounting, and shared bounded execution resources. | S1, S4, S6, S9 | T1, T3, T5 |
| R4.04 | Mutable cache and handle keys distinguish workspace and mount lifetime; NodeId alone must not accidentally alias state across workspaces. | S1, S4, S6, S9 | T1, T3, T5 |
| R4.05 | All hardlink aliases of an inode share content and metadata state. | S1, S4, S6, S9 | T1, T3, T5 |
| R4.06 | Cross-directory rename and other multi-object operations use a deliberate atomicity/locking protocol. | S1, S4, S6, S9 | T1, T3, T5 |
| R4.07 | A slow, full or disconnected workspace cannot indefinitely block unrelated workspaces. | S1, S4, S6, S9 | T1, T3, T5 |
| R4.08 | Multiple views/mounts of the same workspace have an explicit support contract; cross-machine writable coherence is not implicitly claimed. | S1, S4, S6, S9 | T1, T3, T5 |
| R4.09 | One authoritative owner does not imply a global worker/lock serializing all mounted workspaces. | S1, S4, S6, S9 | T1, T3, T5 |
| R5.01 | Preserve supported create/exclusive-create, read/write, truncate, append, rename, link, unlink and directory operations. | S4, S6, S8, S9 | T1, T3, T4 |
| R5.02 | Preserve required open-handle behavior across rename/unlink; retain backing while readers, mappings or operations require it. | S4, S6, S8, S9 | T1, T3, T4 |
| R5.03 | Preserve correct partial writes, quota errors, permission errors, missing-file errors and unsupported-operation results. | S4, S6, S8, S9 | T1, T3, T4 |
| R5.04 | Explicitly define and test locking, memory mapping, executable permissions, symlinks and metadata required by supported tools; do not claim full application compatibility from create/delete benchmarks alone. | S4, S6, S8, S9 | T1, T3, T4 |
| R5.05 | Directory enumeration and cookies remain correct under the supported concurrent-mutation model. | S4, S6, S8, S9 | T1, T3, T4 |
| R5.06 | Kernel name, attribute and page caches remain coherent after filesystem mutations, SDK edits, ownership transfer and Commit. | S4, S6, S8, S9 | T1, T3, T4 |
| R5.07 | Callback suppression, TTL changes and writeback require a valid coherence/error-delivery argument; no permission bypass or blind cache extension. | S4, S6, S8, S9 | T1, T3, T4 |
| R5.08 | Ordinary shell/editor/installer behavior works through any permitted launcher accessing the mount. External processes must still be accounted for by the chosen synchronization contract; managed-Exec tracking alone does not prove no writer exists. | S4, S6, S8, S9 | T1, T3, T4 |
| R5.09 | Platform-specific filename, deletion, security and notification semantics are mapped explicitly; native Windows behavior is not assumed identical to POSIX. | S4, S6, S8, S9 | T1, T3, T4 |
| R6.01 | Writes never modify canonical objects reachable from an existing snapshot. | S3, S9 | T2, T4 |
| R6.02 | Record live edits with the existing mutable range representation; retain unchanged extents and sparse/zero semantics. | S3, S9 | T2, T4 |
| R6.03 | Do not force full CDC/CAS construction synchronously onto every small write when recording a mutable change suffices. | S3, S9 | T2, T4 |
| R6.04 | Temporary and overwritten intermediate states need not become published objects. | S3, S9 | T2, T4 |
| R6.05 | Only valid final objects enter the published snapshot; incremental provisional state retains required readable storage and finality checks. | S3, S9 | T2, T4 |
| R6.06 | Commit consumes known changed state without rediscovering the entire workspace. | S3, S9 | T2, T4 |
| R6.07 | Commit establishes a consistent boundary. Active writes are either correctly separated into generations or explicitly excluded; never publish a torn state. | S3, S9 | T2, T4 |
| R6.08 | Clearing published changes cannot clear post-boundary changes. | S3, S9 | T2, T4 |
| R6.09 | Successful Commit advances existing live nodes/handles instead of reconstructing the Workspace. | S3, S9 | T2, T4 |
| R6.10 | Expected-head/base checks prevent silent overwrites by competing workspaces. Final references preserve unseen aliases and add/remove ordering. | S3, S9 | T2, T4 |
| R6.11 | Publication/install retry completes the exact retained result without creating another Commit. | S3, S9 | T2, T4 |
| R6.12 | Old snapshots remain readable after new edits, commits, deletion and temporary-storage reclamation. | S3, S9 | T2, T4 |
| R7.01 | Document the guarantees of successful write, close, file/directory synchronization and Commit separately. | S5, S7, S9 | T4, T5 |
| R7.02 | Distinguish application, daemon, transport, host and power-loss failure. Preserve the actual product contract without silently weakening or overstating durability. | S5, S7, S9 | T4, T5 |
| R7.03 | Interrupted transfer/construction never appears as a successfully published snapshot. | S5, S7, S9 | T4, T5 |
| R7.04 | Producer/consumer failure, cancellation and finishing errors drain or disconnect safely and join all workers. | S5, S7, S9 | T4, T5 |
| R7.05 | Unmount and shutdown have bounded, defined behavior with active requests, open handles and retained mappings. | S5, S7, S9 | T4, T5 |
| R7.06 | Failed/disconnected workspaces retain or release acknowledged dirty data under an explicit recovery contract. | S5, S7, S9 | T4, T5 |
| R7.07 | Repeated commits and failures do not accumulate unbounded segments, descriptors, queued operations or cleanup debt. | S5, S7, S9, S11 | T4, T5 |
| R7.08 | Physical retirement stays accounted for within the declared lifecycle; do not move cleanup after timing or uncharge still-live storage. | S5, S7, S9, S11 | T4, T5 |
| R8.01 | Account live state, caches, dirty ranges, partial batches, queues, producer scratch, pending results and retained sources simultaneously. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.02 | Account physical backing, open descriptors and retained readers for their actual lifetimes, including anonymous files. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.03 | Enforce per-workspace and aggregate limits; adding workers or mounts does not silently multiply a previous allowance. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.04 | Bound backpressure and ensure it remains safe during cancellation/disconnect/shutdown. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.05 | Reserve the required bounded pending-operation/byte capacity before accepting a mutation that depends on it; state must not be changed successfully and then become impossible to record because its queue is full. Release or transfer reservations on every outcome. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.06 | Do not hold state locks while waiting for remote communication, queue capacity or worker completion. Document distinct ordering/admission mechanisms and prove their progress behavior. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.07 | Multi-object locking follows a consistent acquisition order; avoid lock inversion across callbacks, transport, observers and shutdown. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.08 | Test races, deadlock scenarios, starvation and error paths. Actors, queues or lock-free code are not assumed risk-free. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.09 | The core redesign requires no additional privileges beyond the existing mount/runtime setup. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.10 | Optional platform accelerations do not become mandatory dependencies of the portable core; platform-specific privilege requirements remain explicit. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.11 | SQLite stays outside Docker and no host-data mount is introduced. Any proposed execution-side temporary backing has explicit ownership, resource accounting and compatibility with the chosen topology. | S5, S6, S7, S8, S9 | T3, T4, T5 |
| R8.1.01 | Declare finite limits for execution workers, outstanding requests, in-flight I/O and concurrently active mounts/operations, both per workspace and across the daemon. Include helper/background/construction work; no thread or unbounded task creation per filesystem call. | S4, S6, S7, S11 | T5, T6 |
| R8.1.02 | Divide the configured CPU concurrency allowance among active workspaces and worker stages instead of granting every mount the whole allowance. Independent progress must not require oversubscribing without bound. | S4, S6, S7, S11 | T5, T6 |
| R8.1.03 | Bound work admitted per batch/request by bytes, entries and applicable traversal limits. Avoid repeated full-prefix scans, unbounded retry loops and unbounded recursive work; large supported operations must make bounded, cancellation-aware progress. | S4, S6, S7, S11 | T5, T6 |
| R8.1.04 | Block or await capacity efficiently under pressure; no busy polling, spin-until-ready loops or retry storms when the host/consumer is slow or disconnected. | S4, S6, S7, S11 | T5, T6 |
| R8.1.05 | Use bounded retry counts/work budgets and the existing failure contract. Cancellation and shutdown must stop admission of new work and leave already-owned work recoverable or releasable. | S4, S6, S7, S11 | T5, T6 |
| R8.1.06 | Record CPU time per completed workload, worker utilization, queue waits and any OS quota throttling separately for host and container. A faster latency obtained only by an undisclosed CPU increase is not a demonstrated efficiency gain. | S4, S6, S7, S11, S11 | T5, T6 |
| R8.1.07 | Distinguish concurrency/work bounds from a hard CPU-time quota. A fixed worker count does not itself enforce a percentage/core-time cap. Where a hard quota is required, name and verify the deployment's actual enforcement mechanism without expanding core privileges; otherwise report only the bounds actually enforced. | S4, S6, S7, S11 | T5, T6 |
| R8.1.08 | Preserve the current Linux container CPU/PID limits. The macOS host is outside that cgroup; define its own worker/admission budget and do not describe host CPU as container-capped. | S4, S6, S7, S11 | T5, T6 |
| R8.2.01 | Specify numeric byte/count limits and checked accounting before implementation for caches, dirty metadata/ranges, input/output queues, partial frames, worker scratch, result/reorder storage, active handles and mount records. Reuse existing limits where applicable; no implicit unlimited defaults. | S5, S6, S7 | T4, T5, T6 |
| R8.2.02 | Enforce the aggregate allocation/reservation bound before allocation, decoding or acknowledgment. Include container and host ownership during transfer, when both copies can coexist; do not assume ownership has moved before it actually has. | S5, S6, S7 | T4, T5, T6 |
| R8.2.03 | Bound out-of-order completions and blocked requests as well as actively processed data. A slow first result or host consumer must not cause an unlimited waiting-results map. | S5, S6, S7 | T4, T5, T6 |
| R8.2.04 | Evict only clean reacquirable cache entries. Dirty state, acknowledged pending writes and open-reader backing remain charged until safely transferred or released; reaching the limit must block admission or return the defined error without data loss. | S5, S6, S7 | T4, T5, T6 |
| R8.2.05 | Charge retained physical segment capacity and required descriptors, including partly obsolete segments pinned by a small live range. Repeated write/Commit/disconnect cycles must not accumulate uncharged storage or memory. | S5, S6, S7 | T4, T5, T6 |
| R8.2.06 | Make simultaneous ownership explicit: live state + caches + all worker scratch + partial/queued batches + results + retained sources + control reserve must fit the declared application-managed memory budget. | S5, S6, S7 | T4, T5, T6 |
| R8.2.07 | Separately disclose allocator overhead, SQLite/cache allocations, thread stacks, kernel page cache, socket buffers and VM/system memory. An application buffer counter is not a whole-process/system hard memory bound; verify an actual deployment-enforced envelope when that claim is required. | S5, S6, S7 | T4, T5, T6 |
| R8.2.08 | Preserve existing container memory/no-swap limits and report host RSS/footprint separately. Admission must react to its defined limits before relying on OOM termination as flow control. | S5, S6, S7 | T4, T5, T6 |
| R8.3.01 | Avoid a global state mutex or ordered queue that lets unrelated workspaces inherit one slow workspace's wait. Keep required per-inode/directory/session ordering explicit without serializing all mounts. | S6, S7, S9, S11 | T3, T4, T5, T6 |
| R8.3.02 | Document the lock/wait dependency graph, including FUSE callbacks, host observers, data transfer, Commit and teardown. Specifically prevent a host Workspace lock from being held while waiting for a Session drain whose application needs that same lock. | S6, S7, S9, S11 | T3, T4, T5, T6 |
| R8.3.03 | Ensure cancellation, drain and teardown can progress when ordinary queues are full. Reserve the necessary bounded control capacity or otherwise prove progress; control operations must not overtake required predecessor mutations incorrectly. | S6, S7, S9, S11 | T3, T4, T5, T6 |
| R8.3.04 | Apply bounded scheduling/fairness so a hot workspace cannot indefinitely monopolize workers or buffers. A cap on total threads alone is not a fairness guarantee. | S6, S7, S9, S11 | T3, T4, T5, T6 |
| R8.3.05 | Compare the retained #48 baseline and replacement under the same workload, concurrency and resource settings: normal load, saturated load, slow/stalled host, queue exhaustion and transport/consumer failure. Record throughput, latency distribution where applicable, lock/queue waits, CPU, peak memory, retained storage and progress of an unrelated workspace. | S6, S7, S9, S11 | T3, T4, T5, T6 |
| R8.3.06 | Exercise opposing cross-directory operations, host observations during drain, cancellation/unmount with active requests, and repeated recovery cycles. Demonstrate no deadlock, leaked ownership or starvation in these scenarios; do not claim universal risk-free concurrency from finite tests. | S6, S7, S9, S11 | T3, T4, T5, T6 |
| R8.3.07 | An increased tail latency, contention level or resource footprint must be explained and evaluated against the declared limits and intended throughput benefit. Do not accept a rewrite solely because single-thread average latency improved while overload behavior worsened. | S6, S7, S9, S11 | T3, T4, T5, T6 |
| R9.01 | Exercise real filesystem operations through the supported mount and public operation surface. | S2, S10, S11 | T6, T7 |
| R9.02 | Where attribution is missing, distinguish kernel/daemon dispatch, queue wait, local processing, host work, reply and application resumption. Unattributed time is not automatically removable overhead. | S2, S10, S11 | T6, T7 |
| R9.03 | Measure single-process and concurrent access, including several mounted workspaces. | S2, S10, S11 | T6, T7 |
| R9.04 | Check representative installation/editor behavior as well as synthetic file operations. | S2, S10, S11 | T6, T7 |
| R9.05 | Freeze workload identity, counts, bytes, metadata and synchronization obligations within a comparison; version changed recipes and preserve historical evidence. | S2, S10, S11 | T6, T7 |
| R9.06 | Measure complete Exec/Commit/End, so buffering does not merely transfer work between phases. | S2, S10, S11 | T6, T7 |
| R9.07 | Report CPU, memory, physical storage, latency and cleanup, with host/container scopes separated and nested timers identified. | S2, S10, S11, S11, S11 | T6, T7 |
| R9.08 | Use focused semantic checks and one selected serial performance sample per iteration. Independent proofs remain at the agreed final stage, with the existing 45-second work / 59-second hard end-to-end limit; no exhaustive per-iteration proof campaign. | S2, S10, S11 | T6, T7 |
| R9.09 | Use the shared measurement lock and preserve other tasks' source, containers, caches and artifacts. | S2, S10, S11 | T6, T7 |
| R9.10 | Validate the actually integrated #48/#49/redesign product before claiming combined performance; do not add favorable phase numbers from separate branches. | S2, S10, S11 | T6, T7 |
| R9.11 | Record actual shared callers, removed dependencies/methods, rejected experiments, exact source identities and remaining limitations. | S2, S10, S11 | T6, T7 |
| R9.12 | Set performance acceptance from the selected workload contract; this checklist does not invent a new universal latency target or release gate. | S2, S10, S11 | T6, T7 |

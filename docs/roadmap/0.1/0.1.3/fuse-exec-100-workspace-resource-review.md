# 100-workspace resource guideline: static review and lower-volume inference

Date: 2026-09-06. Reviewed source: `ba57b4917aa0c160d7db65a37ccc7102a3d77fe5` and installed fuser 0.18.0. Three read-only reviewers covered mount/process lifecycle, retained memory/backing, and CPU/admission/fairness. No builds, tests, benchmarks, mounts or experiments were run. The user has deferred experiments for the current and near-term work.

## Decision

**The target remains 100 simultaneously mounted independent agent workspaces. The previous numerical resource table is not yet statically consistent, and must not be used as proof of that capacity.** Shared live authority is a plausible direction, but the current launcher/ingress and several hidden resource owners must change.

This is a design reference, not a requirement to physically exercise 100 workspaces now. The user accepts inference from existing lower-volume benchmark evidence plus source-based resource accounting. Lack of a 100-workspace run must not block current optimization or completion of its agreed lower-volume scope. Source-confirmed over-allocation and unsafe ownership remain engineering findings to address. No inference becomes a measured 100-workspace PASS.

## Source-backed blockers

| Area | Existing source | Static consequence |
|---|---|---|
| Mount lifetime | `crates/layerfs-daemon/src/main.rs`: accepted handler at 334; mount helper at 1311–1324; output/wait/lifecycle at 1359–1392, 1547, 1691. Helper control thread in `proxy_client.rs`; fuser `Session::spawn` and `run` each add a thread | At least 9 Linux tasks per mount along the normal current path. 100 mounts imply at least 900 tasks before shared/application tasks; incompatible with the matched 256-PID limit |
| FUSE receive buffers | fuser 0.18.0 `src/read_buf.rs::FuseReadBuf::new`, `src/session.rs::MAX_WRITE_SIZE` | One `16 MiB + 4096` owned receive allocation per loop. 100 loops = 1,600.390625 MiB of logical buffer capacity. Negotiating 1 MiB writes does not shrink this allocation. This is userspace managed memory, not kernel overhead; logical capacity is not measured RSS |
| Admission | daemon `admission_limit` at 1902 and checks in `handle_exec`/`handle_mount` | One `(RLIMIT_NOFILE - 32) / 8`, clamped-to-256 limit counts mounts plus managed executions. It does not implement separate mount, CPU, connection or PID budgets |
| Managed commands | daemon `handle_exec`: spawn under global state mutex; per-call control/output/watch threads | At least 5 Linux tasks per active managed Exec before descendants. Two filesystem workers do not cap Bash/Node/compiler CPU or child threads |
| Optional capture | `crates/layerfs-workspace/src/capture.rs::start_capture`, `capture_write`, `build_capture`; Store `ObjectBuffer::bounded_output` | Per-capture thread, copies before channel send, retained Running/Ready output and default index/order buffers bypass the proposed aggregate accounting unless explicitly admitted |
| Physical backing | `crates/layerfs-workspace/src/file_io.rs`, 1 MiB nominal open anonymous segments | Filling 100 GiB with 1 MiB segments would retain approximately 102,400 backing descriptors. Smaller partially pinned segments can worsen the descriptor/byte ratio. A byte cap alone is insufficient |
| Idle state | `cow_tree.rs` node, path, canonical and parent maps; `reclaim` | A Workspace that becomes idle after a scan may retain materialized state. Existing reclamation is not general eviction of every clean reachable inode |
| Host producer budget | `objects.rs::run_finalized_output` receives a per-invocation worker limit | A global eight-worker promise needs actual shared admission across construction/capture jobs; SQLite serialization does not supply it |

Installed dependency evidence is under `.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fuser-0.18.0/src/{session,read_buf}.rs`. Repository source locations above identify the reviewed revision; line numbers may change later.

## Required simplification

```text
100 mount identities
        |
shared lifecycle/authentication/control supervision
        |
one bounded ingress per mount OR explicitly bounded shared ingress
        |
shared filesystem operation workers
        |
shared nonblocking transport

Managed commands: separate admission and output supervision
Host construction/capture: separate global permits and memory ownership
```

Remove the per-mount helper/supervision stack from the chosen consolidated design. Preserve readiness, credential, teardown and mount-lifetime contracts. Removing process isolation changes the process-crash containment boundary; document that explicitly, while preserving logical error/state isolation. Do not claim unchanged crash isolation merely because Workspace IDs remain distinct.

For fuser, `n_threads=1` is insufficient: its current background coordinator and receive worker are separate threads. A small maintained adapter/dependency change may run the event loop without the extra joining thread. Derive receive capacity from the actual supported/negotiated maximum request plus protocol/alignment overhead; audit INIT and non-write requests as well as writes. Do not blindly change a constant and assume all supported requests fit.

## Memory decision to carry into detailed design

The earlier table totals 208 MiB before missing ingress allocations:

```text
128 MiB live state
 32 MiB clean cache
 32 MiB transfer / pending buffers
 16 MiB scratch / results / control
-------
208 MiB
```

The 100 progress reservations already use 25 MiB of the transfer budget and 6.25 MiB of scratch; these are subsets, not additional free capacity.

**Prefer a simple, explicitly larger managed-memory budget over inventing a complex shared FUSE multiplexer merely to preserve the arbitrary 256 MiB number.** A candidate per-mount receiver bounded near 1 MiB would require approximately 100.4 MiB for 100 mounts. Reserving 104 MiB for ingress yields 312 MiB with the existing components; a proposed **384 MiB execution managed-memory ceiling** leaves 72 MiB for inventoried ownership not already charged. This is a design alternative with arithmetic, not an implemented limit or RSS prediction. The exact request-size audit must precede adoption. A 256 MiB alternative must genuinely share ingress buffers or reduce other allocations; it cannot omit receive buffers from accounting.

Stacks, allocator overhead, kernel caches, sockets and agent application processes remain separately accounted within the actual deployment envelope. The 2 GiB container cap is unchanged by this proposal. Host transport/recovery at 256 MiB is not host total: shared construction/capture, SQLite/native caches and other host owners need their own explicit aggregate equation.

Reserve a minimum live-state allocation for each admitted Workspace inside the global live-state allowance; request/transfer reserves alone do not prevent heavy mounts from exhausting all metadata capacity. Select the numeric floor from the actual root/handle/binding representation inventory, not an assumed per-agent footprint. Limit heavy borrowers to the shared remainder.

Shared output slabs are active service capacity. A stalled mount must retain partial output in its own charged window and release shared service capacity; four stalled mounts must not permanently occupy all four global slabs. Preserve complete-operation acknowledgment and old-read ownership.

Capture must acquire shared producer and retained-output reservations before copying/spawning. Charge producer, queue, sender copy and Ready output simultaneously. Reuse existing optional-capture fallback when admission is unavailable. Do not add another capture implementation.

Add per-workspace and aggregate segment/descriptor admission alongside byte limits, including sockets, SQLite handles and pinned obsolete ranges. The 100 GiB ceiling is not a promise that 100 mounts have physically reserved 1 GiB each. Preserve anonymous-file lifetime; do not reopen or prematurely reclaim pinned segments to evade descriptor accounting.

## CPU, command and progress gate

Rename the two-worker budget to **filesystem operation workers**. Mounted-workspace slots, ordinary filesystem request slots, control slots, managed-command slots and host construction slots have different lifetimes.

Queued/running commands must not retain filesystem worker permits or ordinary FUSE request slots for their process lifetime: they need those resources to perform their own I/O. Command admission requires a finite count and byte-bounded queue, per-workspace fairness and timeout/cancellation. Choose active command count from the supported process-tree envelope; do not equate it to mount count or infer it solely from two CPU cores. Idle mounts consume no command slot.

Process groups allow termination; they do not enforce descendant CPU/PID/memory limits. Arbitrary external launchers bypass managed Exec admission. Without an existing delegated enforcement mechanism, only shared container limits and an explicit supported-workload condition may be claimed; do not add privilege requirements silently.

Reserve starting-command ownership under a short lock, release the lock before spawning, then install or undo the reservation. Shutdown must observe starting reservations. Consolidate output and stop supervision so slow output consumers do not create unbounded buffers or thread sets.

Host construction/capture obtains global job, producer, scratch and retained-result admission before freezing another Workspace. Waiting jobs hold no Workspace/Store mutex and create no per-job worker pool. Finalized-output reuse alone does not implement these global limits.

Required static equations:

```text
Linux tasks = shared runtime + all ingress/coordinators/supervision
            + active command leaders/descendants + teardown headroom
            <= deployment PID limit

Managed bytes = all live capacities + clean cache + ingress buffers
              + simultaneous sender/receiver/read ownership
              + partial/queued/results + capture Running/Ready + reserves
              <= each explicitly scoped managed-memory limit

Descriptors = mounts + sockets + segments + retained readers
            + SQLite/temp indexes + command pipes + control headroom
            <= per-process and aggregate supported limits
```

Unbounded owners, hidden per-mount multipliers and circular waits remain source-backed defects or design gaps. Record their scope and required correction without introducing a physical 100-workspace acceptance gate. Proposed limits need an enforcement point and release path; a quota table alone does not establish enforcement.

## Design review and evidence status

| Review area | Evidence to retain |
|---|---|
| Ownership inventory | Every process/thread, buffer, retained map, capture output and descriptor has a named owner and scope |
| Arithmetic | 100 mounts and declared active-work allowance fit simultaneous bounds; no double-spending of progress reserves |
| Admission | Capacity reserved before dependent allocation/mutation/spawn; mount/command/FUSE/control/construction admission are distinct |
| Progress | Static wait graph has no dependency cycle; shared service buffers and workers cannot be permanently captured by stalled mounts |
| Failures | Acknowledgments, partial results, cancellation and teardown retain/release exact ownership without mutation replay |
| API compatibility | Request-size/INIT/frame audit and handle/mapping/process support limits are explicit |

Current result: **shared-core direction remains suitable; the previous complete resource envelope cannot support a verified-capacity claim as written.** Keep the concrete corrections as design guidance. Existing lower-volume observations can support labeled CPU/memory estimates where their measurement scope permits, while contention, tails and whole-system capacity remain unmeasured.

Deferred runtime scenarios remain useful future qualification: 100 idle mounts, staggered agent calls, bursty activity, scans followed by idle, stalled hosts and repeated cleanup. Do not launch or schedule these now. Their absence does not block current work or its agreed lower-volume completion. Record 100-workspace capacity as inferred/unverified; never label it physically tested or silently make it a new terminal gate.

## How to infer from existing lower-volume evidence

Use retained receipts and reviewed code only for the current assessment. Record the producing source/product identity, topology, workload recipe, timer/resource scope and any omitted host or application costs. Old measurements do not become measurements of the rewrite.

- Separate mounted count, active agent fraction, call frequency, operation mix and burst concurrency. A few expensive searches/installs can dominate many small edits.
- CPU estimate: `calls/second × observed CPU-seconds/comparable call`, calculated separately for host and container. Account for tool/application CPU, Commit frequency and construction/capture. A bulk benchmark is a scenario reference, not a measured ordinary agent-call cost.
- Memory estimate: `shared baseline + mounted state + active buffers + retained dirty/captured/read state + kernel/application overhead`. Shared budgets are counted once; truly per-mount allocations are counted per mount. Do not multiply whole-process peak RSS by 100 and call it incremental mount cost.
- Linear scaling is appropriate for code-proven fixed costs, such as a retained buffer or thread per mount, within that exact implementation. It does not establish linear throughput, resident memory, tail latency or lock contention. Namespace cardinality, cold acquisition and file-count/byte-throughput changes can produce nonlinear behavior.
- Use ranges or explicit unresolved terms where existing receipts do not isolate a cost. Distinguish measured observations, source-derived bounds, assumed workload parameters and projected totals.
- Existing lower-volume correctness/performance evidence remains sufficient for its declared scope. Preserve future 100-workspace scenarios as optional deferred qualification; do not launch or schedule them now or hold current work for their availability.

The design target remains 100 independent agent workspaces. The evidence label is **resource guideline supported by lower-volume inference; physical 100-workspace capacity unverified**.

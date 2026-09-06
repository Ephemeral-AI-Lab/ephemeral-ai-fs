# Issue 49: shared live Workspace implementation plan

Status: reviewed implementation plan, not implemented behavior or a performance PASS. Date: 2026-09-06. Three read-only audits covered FUSE/runtime, portable state/backing/construction, and lifecycle/SDK/acceptance. Source inspected: saved checkout `ba57b4917aa0c160d7db65a37ccc7102a3d77fe5`, its current documentation amendments, finalized Commit source and retained #48 evidence. No builds, tests, benchmarks or mounts were run for this plan.

Issue: [#49](https://github.com/Ephemeral-AI-Lab/layerfs/issues/49), under #47/#46/#39. The former #49 producer refactor is complete and must be reused. This plan specializes the [rewrite spec](fuse-exec-rewrite-spec.md), [109 requirements](fuse-exec-redesign-requirements.md), [coverage map](fuse-exec-rewrite-checklist-map.md), and [100-workspace resource guideline](fuse-exec-100-workspace-resource-review.md). This plan's precise observation rules and execution order supersede broader earlier proposal wording; correctness/resource obligations remain.

## 1. Final mental model and scope

**One mutable live owner per mounted Workspace. Reuse the existing file algorithms. Keep host physical backing and canonical publication. Acquire missing immutable state outside state locks; resume the same bounded operation.**

Start from current `main` and its finalized Commit code. **Do not make migration, cherry-picking or integration of the research worktree's Exec patch a prerequisite or a separate handoff task.** Learn from its successful mechanisms, safety checks and failed hypotheses, then implement the shared architecture directly on current code.

The goal is a generic execution core used by ordinary FUSE and native-host callers. The primary optimization/iteration case is `tiny-bulk-create-100-mixed-v3`. It creates 1,000 affected files totaling 100 MiB, including one 50 MiB file, with the existing separate 200-file/1 MiB witness. It is not a 100-file workload. Do not change its bytes, names, metadata normalization, sync obligations or generator to manufacture a gain.

No kernel patch, native-directory mirror, Docker SQLite, host-data mount, application/benchmark/size-selected engine, lease service or automatic mutation replay is introduced. The architecture preserves CAS, CDC, COW, PieceTree, rope/extent construction, aliases, stable handles, finality and exact publication/checkpoint recovery.

### Before: current main, with some existing buffered/local operations

```text
Docker/Linux                                      host
application
    |
Linux FUSE
    |
helper process + per-mount supervision
    |
LayerFs / synchronous FilesystemPort / ProxyClient
    |                   |
selected pending/cache  host-decided operation ------------>
operations locally                              FuseView / WorkspaceWorker
                                                host live Workspace
                                                names / attrs / PieceTree
    <------------------------------------------------------ reply

writes -------------------------------> existing shared host segments
Commit --------------------------------> existing shared builders / SQLite
                                          checkpoint host live nodes
```

Caption: not every current operation is a roundtrip; pending creates, reads and deletes already have optimizations. The separate retained #48 research adds further metadata authority/batching. The rewrite must improve on retained responsibilities, not compare only with the original slow setter implementation.

### After: same semantics, local decisions and one construction pipeline

```text
application / ordinary launcher                 host SDK live operation
          |                                               |
Linux FUSE                                                |
          |                                               |
thin LayerFs adapter -- owned operation + reply            |
          |                                               |
          +----------> one LiveWorkspace owner <-----------+
                            |
                +-----------+------------------+
                |                              |
       state/input already owned       missing immutable input
       validate / apply / reply        park bounded operation
                |                              |
                |                         acquire / revalidate
                |                              |
                +-----------+------------------+
                            |
              bounded new bytes + resolved facts
                            |
                    HOST backing service
             existing SpoolSegment / physical ownership
                            |
                    complete-prefix ACK
                            |
            explicit Commit cut -> frozen changed inputs
                            |
             EXISTING producer / rope / extent / CAS pipeline
                            |
                    expected-head publication
                            |
                 exact CheckpointRecord pages
                            |
                  same live nodes continue
```

Caption: local success requires applied state and owned resources. Cold acquisition, actual storage errors and required sync boundaries still cost time. The host installs resolved facts without re-deciding mutable names. This is not optimistic success followed by a second host mutation engine.

### What waiting is removed versus retained

```text
Known-file chmod / mtime / binding change:
BEFORE   callback -> possible drain -> host mutation -> response -> reply
AFTER    callback -> shared local validation + mutation -> reply
                    bounded facts retained/transferred under their contract

Cold read:
BOTH     need actual immutable bytes
AFTER    acquire outside state lock and without occupying a filesystem worker

Commit:
BOTH     require stable input + valid final objects + conditional publication
AFTER    one owner cut, existing builders, exact installation; no Workspace rebuild
```

Fewer requests alone are not a performance result. The retained #48 grant/replay reduction already failed to improve elapsed time. Create-100 results must account for full Exec/Commit/End and resource/cleanup ownership.

## 2. Fixed boundaries and deliberate simplifications

| ID | Boundary | Final decision / prohibited shortcut |
|---|---|---|
| B1 | Live authority | One `LiveWorkspace` per writable mount lifetime; host facade routes live calls to it. No per-method split between host and daemon authorities |
| B2 | Canonical vs mutable data | Existing PieceTree/COW records edits and unchanged roots; host reuses existing canonical builders. No CDC/CAS publication forced into each FUSE write |
| B3 | Acquisition | Inspect/prepare, release state locks, acquire bounded immutable input, reacquire/revalidate, apply once. A blocked acquisition relinquishes filesystem-worker capacity |
| B4 | Success and backing | Reserve before dependent mutation. ACK only complete resolved-fact groups whose referenced bytes are retained. Preserve range ownership across reads, partial writes, failure and checkpoint |
| B5 | Logical observation | Current `session()`/`diff()` fields need a coherent owner metadata/generation read, not payload transfer or a global backing fence. `sessions()` releases registry lock before contacting owners |
| B6 | Actual synchronization | File reads order relevant inode/alias changes. Required fsync/Exec completion/Commit backing fences retain the actual existing error/durability contract. Do not turn every observer into a global drain |
| B7 | Commit | Commands may remain running while Commit succeeds. One owner orders filesystem mutations at a consistent cut; no has_executions/command-exit or ordinary-open-fd gate. Host constructs from frozen changes and installs into the same nodes; later mutations continue |
| B8 | Handles / SDK edits | Running processes and ordinary open read/write handles remain valid across Commit and SDK operations. Use filesystem operation ordering and required kernel coherence, not command lifetime or waiting for close. Preserve open-unlinked reads; actual invalidation failures retain exact result and failed presentation |
| B9 | Concurrency | Short per-Workspace namespace writer plus sorted affected-inode ordering initially; independent data/other Workspace operations can progress. No state lock across I/O, capacity waits, joins or publication |
| B10 | Runtime | One small fuser receive loop per mount; shared lifecycle/auth/control/transport. Reuse decoder/replies. No custom FUSE multiplexer solely to defend the earlier 256 MiB estimate |
| B11 | Portable boundary | Core contains canonical byte identities, names, validation, ranges and state only. Linux mount/credentials/readiness, Unix File, SQLite and Docker stay outside it |
| B12 | Resource evidence | 100 workspaces is an inference-based guideline, not a current physical test gate. Component limits are explicit and counted together; measurements retain source/topology/scope |

Known ceiling: one namespace writer serializes namespace mutations within a Workspace. Mark it in the implementation with a `ponytail:` comment naming the ceiling; change to finer directory locks only if actual contention justifies it. Do not introduce global all-mount ordering.

Consolidating helper processes changes process-crash containment: one daemon crash can affect its mounts. Preserve logical state/error isolation and documented recovery; do not claim identical process isolation. No universal lock-free/risk-free or complete future-platform compatibility claim.

### Commands are not the Commit boundary

The user's filesystem contract supersedes the earlier instruction to preserve managed-Exec Busy. Shell/Exec is a launcher and benchmark surface; Commit must work regardless of whether a process was launched through that API or externally. Multiple commands can stay alive and continue using the same cwd, mount and handles across a successful Commit.

The minimum implementation pauses conflicting filesystem mutations, drains admitted operations without state locks, captures known state and its required backing, publishes/checkpoints, and resumes queued mutations. Holding that mutation pause through publication/install is acceptable initially; a more complex concurrent-generation Commit is not required. Processes are not killed, suspended as process groups, or awaited to exit. Supported mutation calls wait at the boundary rather than receiving Busy solely because Commit is active. Post-cut mutations execute afterward and remain dirty for the next Commit.

Remove command-count coupling in `commit_workspace_session_with_status`, `WorkspaceWorker::quiesce`/writer draining, ordinary SDK edits and admission/start paths. `note_execution` may remain for resource/output/teardown accounting, not to define snapshot consistency. Do not require all writable fds to close. Supported kernel dirty pages/mappings must participate in the real synchronization contract; do not blindly remove checks or silently drop supported behavior. Legitimate head conflicts, failures, cancellation/deadline and resource errors remain. End/discard/unmount/owner-loss cleanup is separate.

A Commit captures filesystem state at an operation cut, not an entire shell command's transaction. A long command can have some completed writes in this snapshot and later writes in the next. Add focused checks with two live commands/handles, writes before and after the cut, direct external access, and unchanged cwd/mount/handle identity. These small correctness checks do not introduce a 100-workspace experiment.

## 3. Explicit folder, type and method plan

Paths are repository-relative. Existing names were verified in the inspected source. Names prefixed **proposed** describe intended ownership, not an already implemented API. Rust structs/enums are the class-level boundaries here.

Confidence captions: **H** = existing responsibility/algorithm and callers support this change; **M** = direction established but exact interface/lifetime split needs bounded exploration; **E** = a specific design decision must be resolved at the indicated slice before wider edits. These are qualitative engineering assessments, not measured performance confidence.

### C1 — portable live core and dependency direction — H direction / M signatures

Proposed folder `crates/layerfs-workspace-core/`, introduced only with its real native-host and daemon consumers. Start with `Cargo.toml`, `src/lib.rs`, and moved portions of `cow_tree.rs`, `file_edit.rs`, `limits.rs`; add `backing.rs` only for the concrete shared range/read-plan boundary. Do not scaffold future platform folders or a general VFS framework.

```text
Dependency arrows mean "depends on":
layerfs-workspace-core -> layerfs-content
layerfs-fuse           -> layerfs-workspace-core (+ optional fuser adapter)
layerfs-daemon runtime -> layerfs-workspace-core + layerfs-fuse
layerfs-workspace      -> core + Store + existing daemon client + projection

core -X-> Store / SnapshotReader / SQLite / FUSE / Docker / Unix File
```

Keep daemon client-library builds separate from Linux mount-runtime feature dependencies where needed; do not make importing client protocol force a kernel backend on other platforms. Existing public Workspace constructors/facades remain compatible where required.

| Existing files/items | Concrete change / deletion | Confidence and exploration exit |
|---|---|---|
| `workspace/src/cow_tree.rs`: `NodeId`, `Kind`, `Attr`, `Data`, `FileData`, `DirectoryData`, `Node` | Move actual shared values/state to core; proposed `LiveWorkspace` owns live tables. Existing facade re-exports compatible public values | H. Verify all conversion/call sites; move definitions, do not duplicate them |
| Same: `attr`, `create_file`, `mkdir`, `symlink`, `link`, `unlink`, `rename`, `pin`, `unpin`, `chmod`, `set_mtime` | Transfer existing algorithms in coherent slices; operate on already acquired/revalidated state | H/M. Exit: host caller uses the moved body before daemon activation |
| Same: `lookup_node`, `materialize`, `materialize_record`, `directory_entries`, `directory_is_empty` | Separate immutable page/record acquisition from state mutation; reuse merge/validation algorithms | M/E. Exit: cold acquire cannot perform socket I/O under live-state locks |
| Same: `Workspace`, `WorkspaceSnapshot`, Store/reader/spool/publication fields | Host facade retains Store/session/publication ownership; core receives explicit immutable facts and range ownership | H/M. Exit: core Cargo dependency graph has no host Store or FUSE cycle |
| `fuse/src/port.rs`: duplicate `NodeId`, `Kind`, `Attr` | Re-export compatible shared values when actual callers transfer | H. Preserve public conversion behavior; remove duplicate identity logic |

Reuse `layerfs_content::ObjectRead`, canonical types and portable errors where they fit. Remote object acquisition must not hide blocking I/O behind a synchronous `ObjectRead` call under a core mutex. Move the minimum existing filesystem error vocabulary if needed; do not invent an error registry.

### C2 — portable range ownership and read/write split — H algorithms / M representation

| Existing files/items | Concrete change / deletion | Confidence and exploration exit |
|---|---|---|
| `workspace/src/file_edit.rs`: `Piece`, `PieceTree`, `SpoolSlice`, `replace`, `range`, compact sequential representation | Move existing tree logic. Replace physical `Arc<SpoolSegment>` dependency with **proposed** `BackingId`/`BackingRange` owned token; retain compact representation and existing counters | H/M. Exit: split/merge/clone/last-reader lifetime remains exact without per-inode lease machinery |
| `workspace/src/file_io.rs`: `ReadPlan`, `ReadSource`, `ReadPlan::for_file` | Core returns bounded owned range plan; host/direct or remote adapter performs canonical/segment reads outside core locks | H/M. Exit: held old read survives later write/unlink/checkpoint |
| Same: `write_inner`, `edited_state`, `check_piece_resources`, `install_edit`, `truncate`, `edit_many`, `EditCheckpoint` | Extract existing prepare/reserve/apply phases; **proposed** `prepare_write`/`apply_write` names only if they clarify real phases | M/E. Exit: exact partial-write result and rollback when backing append fails; no successful unrecordable mutation |
| Same: `SpoolSegment`, `append_segment`, `append_spool`, `read_exact_at`, `retire_spool_segments`, `PhysicalSpoolMetrics` | Remain host physical implementation; resolve portable ranges through one direct/remote backing boundary | H. Preserve anonymous FD retention, physical charges and release batching |

Preserve the affected inode's operation ordering and prepared version across out-of-lock backing work. Revalidate that exact state before installation; if stale, safely retain or reclaim the unused appended range while continuing to charge its ownership. Never repeat an append or install the prepared edit against a different live state. This requires no new replay framework.

A local pending buffer and an acknowledged host segment are backing states of the same range ownership contract, not separate file engines. A read shares the owner's retained range reference; do not introduce pin/unpin RPC per read.

### C3 — FUSE submission and resumable acquisition — M / critical E

| Existing files/items | Concrete change / deletion | Confidence and exploration exit |
|---|---|---|
| `fuse/src/adapter.rs::LayerFs`, `node`, `attr`, `open_handle`, `handle`, `errno` | Retain credentials, stable kernel/handle mapping, FileAttr and reply translation; submit an owned admitted operation | H/M. Charge copied borrowed callback arguments and reply ownership before retention |
| `fuse/src/filesystem.rs` callback methods | Retain fuser API/capabilities/counters; return from ingestion after handing off owned reply/arguments; complete reply exactly once | M/E. Cancellation/teardown must own unfinished replies explicitly |
| `fuse/src/port.rs::FilesystemPort` synchronous `PortResult<T>` methods | Adapt to one minimal ready/pending submission seam; preserve native-host facade through same core. **Proposed** `OwnedOperation` and `OperationStep::Ready/NeedBacking` are concrete candidates, not a generic interpreter | M/E. Exit before shared pool: two cold acquisitions cannot occupy all filesystem workers; resume revalidates and never repeats an applied mutation |
| `fuse/src/proxy_client.rs::ProxyClient` | Transfer retained codec/metrics and backing access; delete competing pending-create/live-metadata decisions, reserved-node grants and ad hoc category drains after actual callers migrate | H direction/M delivery. No partial runtime routing where create has one authority and rename has another |
| `fuse/src/protocol.rs`, `proxy_host.rs::ProxyHost::start`/`serve` | Reuse authentication/framing; represent bounded acquire, backing/fact groups, ACK, fence and ownership release. Host installs resolved facts, not POSIX operation replay | M/E. Exit: partial group cannot ACK; lost reply cannot duplicate mutation; declared frame cap respected before allocation |

Do not simply place the current synchronous `FilesystemPort` behind two threads. That permits two remote waits to block the entire shared executor. Resolve the smallest explicit suspension state before building scheduling around it. Do not add generic Actor/Effect/Transaction/Route frameworks.

### C4 — fuser ingress and daemon lifecycle — H need / M API patch

| Existing files/items | Concrete change / deletion | Confidence and exploration exit |
|---|---|---|
| `fuse/src/host_mount.rs::HostMount`, `mount_host`, `notifier`, `unmount`, `join` | Reuse for daemon in-process and native-host mounting; expose readiness/completion directly instead of helper stdout | H/M. Preserve mount flags, exact readiness, destroy/unmount/join ordering |
| fuser 0.18 `Session::spawn`, `Session::run` | Narrow maintained patch: single configured receive loop runs without a second joining coordinator thread | H need/M patch. Do not redesign unrelated multithread behavior |
| fuser `FuseReadBuf::new`, `Session::handshake`, retained negotiated configuration | Use bounded INIT input and receive capacity derived from every supported request maximum plus headers/alignment; retain necessary negotiated limit | H need/E exact bound. Negotiated 1 MiB writes alone do not change existing 16 MiB allocation; audit INIT and non-write requests before choosing capacity |
| `daemon/src/main.rs::ActiveMount`, `handle_mount` | Own an in-process mount handle; reserve under registry lock, mount/INIT outside it, install ready state or undo admission | H/M. Preserve authenticated lifetime, teardown and cancellation of affected managed commands |
| Same: `watch_mount`, `drain_mount_output`, helper spawn/stdout/waiter/lifecycle branches | Remove after shared supervision owns their exact responsibilities | H. No orphan mount, lost failure reason or early close ACK |
| **Proposed** `daemon/src/runtime.rs` | One justified module for bounded ready-workspace dispatch, existing connection state, authentication, timers and lifecycle/output readiness | M. Enable only needed existing `nix` readiness features; no runtime framework added by default |
| `daemon/Cargo.toml`, `fuse/Cargo.toml`, root workspace manifest/lock | Add core/runtime dependencies and pin narrow fuser patch consistently | H/M. Upstreamable patch preferred; exact vendoring/package form is a bounded implementation choice, not a copied FUSE decoder |

The current runtime has at least nine Linux tasks per mount and fuser reserves 16 MiB + 4 KiB per receive loop. These are source-backed reasons to correct the design, not a reason to require a 100-mount run. Prefer one small receive loop/mount and an honest revised memory budget over a speculative FUSE multiplexer.

### C5 — host backing, capture and construction admission — H reuse / M global owner

| Existing files/items | Concrete change / deletion | Confidence and exploration exit |
|---|---|---|
| `workspace/src/file_io.rs` segment allocation/retirement | Add exact shared segment/descriptor reservations alongside byte ownership, while retaining existing storage algorithms | H/M. A 100 GiB byte ceiling must not silently allow 102,400 open one-MiB segments |
| `workspace/src/capture.rs::CaptureState`, `capture_write`, `finish_capture`, `invalidate_capture`, `take_capture`, `build_capture` | Keep host optional canonical capture; feed admitted backed data, account Running and Ready outputs, reuse fallback when shared admission is unavailable | M/E. Exit: no extra mandatory capture wait on each FUSE write; no uncharged pre-send copy or per-workspace full index allowance |
| Store `objects.rs::run_finalized_output`, `construct_workspace_files` | Retain delivered task/producer/consumer algorithms. Supply actual host-global permits around jobs/capture without changing semantic input classes | H/M. Per-invocation worker limit is not global admission; waiting jobs hold no live/Store locks |
| Existing owned-slab queue and backing connection state | Shared service slabs are reusable service capacity; park stalled output in its charged mount window | H/M. Four stalled peers cannot own all four shared slots indefinitely |

Do not drop capture or canonical deduplication to make the extraction easier. Resource-based admission/fallback uses existing behavior, not a benchmark-selected engine. Host transport/recovery budget is not total host memory.

### C6 — frozen Commit inputs and exact checkpoint — H reuse / M adapter

| Existing files/items | Concrete change / deletion | Confidence and exploration exit |
|---|---|---|
| `workspace/src/changes.rs::StableFileInputs`, `FrozenFile`, `WorkspaceFileReader` | Consume **proposed** `FrozenWorkspaceChanges`: known changed records/ranges at one generation plus unchanged roots | H/M. No entire Workspace clone, full namespace manifest or repeated path discovery |
| Same: `FileTaskPlan`, `FileTasks`, `FileResultWriter`, `FileResults` | Reuse current task slots, ordered result journals and bounded buffers | H. Do not redo completed #49 producer sharing or change default workers merely to fit extraction |
| Same: `build_candidate`, `build_frontier_candidate_with_workers`, `apply_frontier_directory`, `FrontierInodes`, `ReferenceJournal` | Remain host construction; accept actual frozen context rather than mutable-host Workspace dependency | H/M. Preserve latest alias/reference precedence and fresh/incremental/private finality |
| Same: `CheckpointJournal`, `Checkpoint::visit`, `PreparedCommit` | Keep host bounded installation facts; **proposed** portable `CheckpointRecord` represents existing NodeId/root/attribute records | H/M. Stream bounded pages; retain candidate/outcome through exact installation |
| `lifecycle.rs::Workspace::commit`, `transition_committed`, `install_checkpoint`, pending publication | Keep expected-head/stage/publication authority on host; move actual node install to same live owner | H/M. Lost installation reply finishes retained result, not another Commit |
| `reconcile.rs`, `changes.rs::resolution_fingerprint`, `base_manifest`, `final_manifest` | Trace real reconciliation/private-preview callers; adapt frozen views while retaining their semantics | H retention/M adaptation. These methods are not proven dead merely because ordinary Commit uses the frontier |

### C7 — SDK observation, edits and one quiescence boundary — H semantics / M transport

| Existing files/items | Concrete change / deletion | Confidence and exploration exit |
|---|---|---|
| SDK `client.rs::commit_workspace_session_with_status` | Preserve public result/status contract | H. No user-facing API rewrite needed solely for placement |
| `workspace/src/worker.rs::WorkspaceWorker`, `enter_callback`, `note_writer`, `note_execution`, `quiesce`, `wait_for_writers` | Actual live owner tracks callback/writer cut; host routes control and retains only actual host responsibilities | H/M. Avoid host + daemon independently counting/draining the same work |
| `lifecycle.rs::Workspaces::commit_workspace_session_with_status` | Request one filesystem-operation cut/frozen input; remove managed-command Busy gating, preserve actual failure/resume and publication recovery | H/M. No live state/Store/registry lock across owner request or backing drain |
| `edit_workspace_file_ranges` | Call existing shared edit validation/operation at owner; preserve exact rollback/exclusion and kernel invalidation | H/M. No blanket command/open-fd rejection; supported mapping/dirty-page coherence requires a real capture boundary |
| `session`, `diff`, `sessions` | Owner metadata/generation reads; copy registry references before contacting owners | H. No payload flush/global freeze just to report dirty/generation/execution summary |
| `projection.rs::pause`, `resume`, `refresh_file`; `docker.rs::DockerProjection::{pause,resume,invalidate_file}` | Thin control/notification adapters; remove duplicate host live mutation decisions | H/M. Failed invalidation/presentation cannot silently resume stale cached access |
| `recover_workspace_presentation`, `end_workspace_session` | Reuse exact recovery, retained-summary/discard and bounded retirement semantics | H/M. No blind reconnect/replay; release/forget/cancellation remain admissible while draining |

### C8 — managed commands and launcher cleanup — H need / M scheduling

`daemon/src/main.rs::Shared.limit`, `admission_limit`, `handle_exec`, accept loop, `watch_exec`, `pump` must separate mount, connection, command and filesystem admission. Reserve a tracked starting command under a short lock, release before `Command::spawn`, then install/rollback so shutdown cannot lose it. Output/stop supervision uses bounded shared readiness. Command slots do not occupy filesystem slots for their process lifetime.

`workspace/src/execution.rs::{exec,shell,spawn,stop,output}` and `daemon.rs::{DaemonOwner::mount,DaemonOwner::start,DaemonMount}` retain existing public launcher/output contracts and exact mount identity. External shell access through the mount remains supported; arbitrary external process trees are not magically bounded by managed Exec admission.

`fuse/src/bin/layerfs-fuse.rs`, `workspace/src/docker.rs::{DockerProjection::attach,attach_daemon,ProjectionLauncher}`, and `benchmark/fs-bench-pro/Dockerfile.layerfs`: migrate real helper/bootstrap/image callers before deleting the binary or packaging. Keep `mount_host` because native host projection is a real caller. No new standalone helper engine or permanent old/new switch.

Confidence H on distinct admission lifetimes and obsolete helper responsibilities; M on bounded connection/output state implementation. Exploration exit: all shared and per-command task/FD owners enumerated, no thread-per-connection admission escape, no global lock across spawn/wait/output.

## 4. Execution order and reviewable deliverables

One implementation owner controls shared core/Workspace/Store seams. Independently owned runtime or adapter work may proceed only after agreeing actual interfaces; read-only reviewers are safe in parallel. No overlapping edits to an active #48 research owner without custody transfer.

| Step | Work and dependency | Exit artifact / create-100 relevance |
|---|---|---|
| P0 — freeze | Fetch actual current main; bring latest local plan/guideline into branch; inspect #48 mechanisms, retained evidence and rejected hypotheses as references; freeze the exact mixed-v3 receipt/recipe | Source/caller/deletion ledger. No new benchmark baseline campaign; existing numbers remain producing-source evidence |
| P1 — minimum portable seam | C1/C2 identities, PieceTree ownership and read/write preparation; native-host facade uses moved bodies first | Concrete before/after signatures, core dependency check, inherited focused range/lifetime checks. Do not extract every Workspace subsystem upfront |
| P2 — minimal runtime entry | Narrow fuser single-loop/buffer patch, direct owner mount plumbing, capacity ownership needed by the first operation slice | Reused decoder and safe request size/lifetime; exact source-based buffer/thread arithmetic. P2 can be prepared alongside P1 with fixed interfaces |
| P3 — complete create path | C3 local owner handles cold parent acquisition, mkdir/create/write/read/metadata/release/sync needed by ordinary create; C5 backing + C6 frozen Commit bridge + C7 current SDK observers included | End-to-end create/read/Commit/continue on one owner. This is the first meaningful performance candidate; no per-benchmark dispatch or alternate host authority for unsupported mutations |
| P4 — create-100 iteration | Focused changed-seam checks, one serial source-bound create-100 performance sample when execution work is resumed | Compare complete lifecycle and phase/resource counters; keep/revise one substantive change. If no gain despite fewer RPCs, investigate attribution before expanding scheduling/protocol work |
| P5 — remaining supported surface + consolidation | Transfer rename/link/delete/cursors and remaining SDK/preview/reconciliation callers; complete shared supervision/command/capture admission; remove migrated proxy/helper code as callers disappear | Product default changes only after required supported callers are complete. No permanent mixed-authority path. Physical 100-workspace testing is not an exit condition |
| P6 — stable candidate | One same-source delete-100 control, applicable existing low-tier checks, final selected bounded proof pair after both parent performance conditions pass | Exact results, omissions, source/product identities, remaining limits and final deletion ledger; no automatic parent closure |

P3 is a branch-local coherent vertical slice, not permission to ship a mount whose remaining operations silently route to a second owner. Do not spend weeks building all resource/general-runtime machinery before this first useful slice. Conversely a fast create result does not excuse missing supported aliases, recovery or observer semantics at completion.

## 5. Create-100 fast iteration and acceptance

This document specifies the implementation workflow; it launches no experiments now. The physical 100-workspace scenarios stay deferred. During the resumed implementation use the ordinary agreed lower-volume workflow, not a new scale campaign.

Current adopted product reference, seconds to four decimals: create Exec **0.9172**, Commit **0.3587**, complete **1.2914**; delete Exec **0.2684**, Commit **0.0148**, complete **0.2980**. Their source/product IDs are in the spec and [campaign ledger](issue49-ten-family-refresh.md). Retained research create Exec **0.4792** / complete **0.9680** is a different product. Never add its Exec to the adopted Commit number or call mixed-v3 count reduction a code speedup.

### Fixed development/test topology and reset policy

Use the actual macOS-host + Docker Linux + real FUSE path for performance and end-to-end behavior. Host owns SQLite, SDK/benchmark coordination, physical spool backing and canonical construction/publication. Linux owns workloads, FUSE and the new live operation core. Moving that core is the intended rewrite; it does not move SQLite or the benchmark coordinator into Docker. Native focused tests are supplemental and cannot substitute for the product topology. No data mounts, container SQLite or native-directory benchmark shortcut.

Normal reset is **`--setup clone`**, through existing family scripts. `setup.sh` delegates to the shared runner's `--prepare-only`, creates or validates/reuses the protected host master, and does no performance run. `perf.sh` acquires compatible preparation itself, makes an independent disposable sample Store, and owns fresh container/FUSE lifetime plus cleanup. Do not call setup before every iteration unnecessarily, reset Docker, prune build caches, discard the preparation cache or replay namespace initialization routinely.

The copy implementation is `runtime.closed_store_copy` (`closed-quiescent-byte-copy`), not APFS clone/reflink. Retain validation, SQLite quiescence and unchanged-master checks. A new source version does not alone invalidate the reusable fixture; the runner's schema/fixture/seed/content compatibility remains authoritative. Workload/schema changes must acquire the correct new master instead of forcing reuse. The CLI alternative is `fresh`, not `refresh`; it is not the normal iteration reset.

After product-source edits, run existing cached host/image build commands to obtain matching seals; do not hand-edit identity sidecars or use a stale image. Reuse sealed artifacts when still matching. Read cache-hit/clone-method/master-unchanged/cleanup evidence in each sample. Preserve 2CPU/2GiB/no-swap/256PID container limits and host resource reporting.

Cycle:

1. Read the last retained attempt and choose one hypothesis affecting create-100: local metadata/binding decisions, repeated acquisition/normalization work, dispatch/reply waiting, backing ownership/copies, or required fences. Keep unrelated mechanisms fixed.
2. Make one reviewable change and run the smallest existing focused semantic checks that exercise its actual changed boundary. Reuse passing checks unless code/new evidence invalidates them. No full benchmark/proof suite after every edit.
3. Build matching host binary and Linux daemon/workload image through the existing runner and shared measurement lock. Builds/preparation are separate from proof budgets. Host SQLite, no data mounts, container 2 CPUs/2 GiB/no swap/256 PIDs remain; host resources are separate.
4. Run **one** selected serial performance sample, seed 1, unchanged mixed-v3 recipe. Fresh output directory; reuse compatible protected preparation. No favorable-repeat search.
5. Inspect full lifecycle, Exec, Commit, End/cleanup, normalization and actual callback/queue/acquisition/backing/CPU/memory counters. Distinguish nested spans from disjoint elapsed time; unattributed time is not automatically removable.
6. Retain/revise based on evidence. Replan genuine no-go results; do not add routes/frameworks or tighten small phase targets merely to keep tuning. Remove dead replaced methods as the corresponding slice completes.

Performance command using the current entrypoint (for implementation execution, not run by this review):

```bash
# Build/refresh sealed artifacts only when relevant source changes.
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
export LAYERFS_BENCH_IMAGE="$(python3 benchmark/fs-bench-pro/shared/runner.py --build-image)"

# Prepare once; repeat only to ensure new/changed compatible input when needed.
bash benchmark/fs-bench-pro/families/tiny_file_churn/setup.sh \
  --topology host-store --case tiny-bulk-create-100-mixed-v3 \
  --seed 1 --setup clone --image "$LAYERFS_BENCH_IMAGE"

# Normal reset and one full sample.
bash benchmark/fs-bench-pro/families/tiny_file_churn/perf.sh \
  --topology host-store --case tiny-bulk-create-100-mixed-v3 \
  --seed 1 --setup clone --perf-fast --image "$LAYERFS_BENCH_IMAGE"
```

Build commands remain `runner.py --build-host` and `runner.py --build-image`; bind the resulting image/source as documented in QUICKSTART. Default performance watchdog 120 seconds product / 130 seconds outer is a diagnostic allowance, not the pass target. Do not replace create-100 with a tiny smoke case as performance proof, and do not run tier500 or all families routinely.

**Working ambition is now create-100 complete lifecycle around 0.7000–0.8000 s**, with an indicative Exec budget around 0.3000–0.4000 s while preserving current Commit efficiency. These are planning objectives, not measured predictions or separate hard phase gates. The earlier approximately 0.5000 s Exec expectation is a reference checkpoint, not the rewrite's ambition. The research run used an older Commit implementation (0.4718 s versus the finalized reference 0.3587 s), so its 0.9680 s total is not the desired endpoint. Do not combine measurements from different products into a claimed achieved lifecycle. A roughly 0.0500 s difference from the working band should not trigger prolonged minor tuning when the structural result is sound.

**Parent target remains both tier100 create/delete complete lifecycles strictly below 1.0000 s using unrounded measurements.** No new isolated Exec/Commit millisecond gate. A roughly 0.0500 s phase difference is not a reason for prolonged minor tuning, but a strict total MISS remains a MISS. Preserve Commit's existing gain; a faster Exec offset by Commit/End is insufficient.

Independent proofs stay at the final stable-candidate stage after performance. Use actual `benchmark/fs-bench-pro/verify-selected.py` with exact family/case/source/input/image/seed from the performance receipt; 45-second work / 59-second hard end-to-end per proof, separately recorded preparation, explicit sampled coverage/omissions. Do not invent IDs in example commands or bypass the source-bound wrapper. Focused correctness checks and independent benchmark proofs are distinct. No benchmark run occurs as part of this planning turn.

Stop conditions are separate: P3 demonstrates an integrated slice; P4 demonstrates an optimization outcome; P5/P6 complete the supported rewrite and declared evidence. None alone is the other. 100-workspace feasibility remains inferred/unverified and does not block the agreed lower-volume completion.

## 6. Checklist traceability and future portability

The companion coverage map associates each of the 109 requirements with spec sections, focused test groups, implementation components C1–C8 and steps P0–P6. This is coverage of obligations, not 109 per-iteration test gates and not implemented PASS. Distinguish existing behavior preserved, changed seam requiring a focused check, and deferred capability/scale evidence in the implementation ledger.

Linux is the actual adapter target. macFUSE/Windows can reuse the core state/range/admission/construction contracts later; names, permissions, deletion, mapping, invalidation, mounting and readiness remain platform adaptations. Do not put Unix File or fuser reply types into the portable core, scaffold unused backends, or claim identical POSIX/Windows behavior. Existing host materialization and private-preview/reconciliation callers must not be deleted merely because FUSE is the optimization focus.

Architectural confidence: high for reuse, fixed authority, cheap owner observations and exact checkpoint responsibilities; medium for range-token, acquisition-suspension, frozen-input and capture feed interfaces. Performance magnitude is unproven. Exploration is bounded to these concrete seams at P1–P3; update exact signatures/caller ledger when resolved instead of silently expanding the architecture.

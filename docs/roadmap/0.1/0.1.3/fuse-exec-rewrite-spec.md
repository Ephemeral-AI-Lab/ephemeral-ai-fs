# FUSE Exec rewrite: one live owner with bounded host backing

Date: 2026-09-06. Status: proposed implementation specification, reviewed by three read-only specialists covering Linux FUSE/kernel behavior, filesystem ownership/concurrency, and transport/resources. This document specifies a design; it does not establish implementation, correctness or performance PASS.

Normative requirements: [redesign checklist](fuse-exec-redesign-requirements.md). Per-item design/test traceability: [coverage map](fuse-exec-rewrite-checklist-map.md). Tracking context: #48 Exec, #47 integration, #49 completed producer refactor. No implementation, runtime change, benchmark or issue closure is performed by writing this specification.

## Active issue and adopted starting code

Issue [#49](https://github.com/Ephemeral-AI-Lab/layerfs/issues/49) is repurposed, at the user's request, for this FUSE Exec redesign. Its former producer-sharing scope is complete; retain the [completed producer plan and adoption ledger](issue49-shared-producers-plan.md) as historical implementation evidence. Do not repeat that refactor or treat old issue comments as the current work order. Parent #47, under #46/#39, remains open.

The starting code is finalized Commit branch `codex/issue47-commit-first` at `d6cc03f500a9aaf3f629fd5a76e9fc26e95ae00f`, integrated with this specification for publication to `main`. New implementation work must fetch remote `main` and record its actual starting SHA. Do not start from the older saved-project checkout or replace current Workspace/Store files with stale #48 research copies.

The completed [ten-family campaign](issue49-ten-family-refresh.md) records 138/138 performance cases and 34/34 selected proofs passing under its fast scope. Its tested product is `3fb2f18f1c636b8c3aa8b2a901bb3ca2a56cb075e674b6f520e8c685ca2b1053`, source seal `4ce40e2c3c3573902e3820d8e16c5f1be793f42dbeb14e4bb72ed4274557fc1f`; product build commit `6de381837d1c22e5eb21dfaf446f85c9c30ea6ec`. Subsequent finalized branch changes are documentation only. Current mixed-v3 measurements are:

| Tier / operation | Exec (s) | Commit (s) | Complete lifecycle (s) |
|---|---:|---:|---:|
| 100 create | 0.9172 | 0.3587 | 1.2914 |
| 100 delete | 0.2684 | 0.0148 | 0.2980 |
| 500 create | 4.1284 | 1.7790 | 5.9276 |
| 500 delete | 0.9103 | 0.0541 | 0.9771 |

Integration CI requires Rust 1.96.0 formatting. The follow-up formats two benchmark source files without changing their operations, workload recipe or timer semantics. CI also exposed existing Clippy warnings in the finalized producer/frontier/fsync code. Integration follow-up retains those interfaces and backing layouts, uses scoped lint annotations, and simplifies equivalent expressions without changing ordering, failure or workload semantics. Historical source/product seals remain attached to their original measurements; these integration corrections are not a new measured product qualification.

Timings are seconds rounded to four decimals; decisions use unrounded receipts. Full per-case results and provenance are preserved in [#46 results](https://github.com/Ephemeral-AI-Lab/layerfs/issues/46#issuecomment-5555290362) and [evidence](https://github.com/Ephemeral-AI-Lab/layerfs/issues/46#issuecomment-5555291106). The strict tier100 create lifecycle target still misses; its strict final proof pair remains deferred. The campaign does not include the separate retained #48 research implementation. S2's research timings are a second-source comparator, never additive phases of this adopted product.

Detailed execution order, exact file/type/method transfers, confidence/exploration captions and create-100 iteration instructions are in the [reviewed implementation plan](issue49-fuse-exec-implementation-plan.md). Its narrower observer semantics avoid unnecessary payload fences.

## S1. Decision and scope

Use **one execution-side authoritative live Workspace core per mounted workspace**, thin platform adapters, existing host-owned shared spool backing, and the existing host construction/SQLite pipeline. Host SDK operations on live state invoke that same owner. The host may retain acknowledged recovery facts, but it must not independently decide live namespace mutations.

The first version retains host spool placement. It does not add container-side SQLite, a native-directory mirror, a second disk spool engine, kernel patches, FUSE passthrough, automatic reconnect/replay, per-inode leases, or an application-specific fast route. Locality concerns live decisions and already-owned data; cold immutable data and backing I/O still cross the existing topology.

```text
 Linux applications / concurrent shells
                  |
          Linux FUSE + fuser
                  |
       thin callback/reply adapter
                  |
       bounded ready-workspace dispatch
                  |
        authoritative Workspace core
        bindings / attrs / handles / pieces
          |                         |
      local result            owned pending bytes/facts
          |                         |
       FUSE reply         bounded ordered backing channel
                                    |
                         host shared segments + retained
                         acknowledged recovery facts
                                    |
                   frozen delta at explicit Commit cut
                                    |
                   existing #49 builders / checked Store
                                    |
                        exact publication + checkpoint
```

This is one logical writable state owner. Host recovery facts are a synchronized prefix for recovery/frozen construction, not a second live mutation API. Do not replay and re-decide every POSIX operation in a shadow filesystem: transfer resolved existing record/range facts and reuse existing checked installation primitives where available. If that transfer cannot be implemented without a broad new effect framework, stop and simplify the seam before coding more layers.

Baseline scope: concurrent processes within one writable mount; at least 100 simultaneously mounted independent live workspaces per daemon; one writable mount per Workspace lifetime. The earlier 8-mount proposal is superseded by the user's 100-workspace requirement. Reject a second writable mount of the same Workspace explicitly. Additional immutable snapshot mounts are independent read-only views. Cross-machine multi-writer coherence and transparent mount migration are not claimed.

## S2. Lessons and comparator, not architecture mandates

The retained Exec research task is `01a072c8-2e80-7982-966b-df26da9fdeb5`, worktree `/Users/yifanxu/.codex/worktrees/401f/layerfs`. Its retained mixed-v3 source seal is `c09f058ee659e72428c6e9e07a67e20850db165d70dbc1c0115e9c7aa0d94d0b`, product seal `68cf387dba5749287c0e9c5f950cbe60128084ae153455fa5618edbf97a9859c`.

| Retained observation | Design implication |
|---|---|
| Mixed-v3 create Exec 479.150250 ms, Commit 471.756458 ms, complete 968.009875 ms | Compare against this optimized source/recipe, not only the original slow implementation |
| Delete Exec 239.114375 ms, complete 258.102210 ms | Preserve efficient delete behavior; no payload-production benefit assumed |
| Both bounded sampled proofs passed; one performance sample each | Correctness coverage is sampled and create margin is narrow; not reliability or integrated-product qualification |
| Metadata authority/replay, DIRECT_IO-handle NOFLUSH and bounded allocation gave useful earlier improvements | Transfer their validation, visibility, error and lifetime responsibilities |
| Grants/replay pages 157 each to 20 each did not improve elapsed time | Fewer frames alone do not justify the rewrite |
| No positive LOOKUP entry had an expired advertised TTL in the v3 diagnostic | Do not begin with a TTL increase; zero expiry does not identify every kernel lookup cause |
| Completion drains below 1 ms in retained v3 samples | No evidence of a large completion-drain backlog |

The research implementation remains a dirty source/evidence checkpoint based on `8cbbb425b`; do not merge its entire worktree over the integration branch. Use its retained mechanisms and rejected experiments as evidence. The user does not request migration/cherry-picking of the research Exec patch; do not create that as a prerequisite. Implement the shared rewrite on current main while preserving equivalent correctness/lifetime responsibilities. Read its `research/issue48-exec/real-fuse/{mixed-v3-baseline,mixed-v3-next-investigation,residual-findings-20260906}.md` with their stated evidence limits. Earlier proposal documents are alternatives, not measurements.

## S3. Components and reuse boundary

1. **Platform adapter:** retains fuser/kernel mount, credentials, handles-to-core mapping, replies, negotiated capabilities and cache notifications. It owns no duplicate file-mutation semantics.
2. **Live core:** extract existing Workspace inode/binding/PieceTree/validation algorithms into a shared runtime-independent library used by the native-host facade and daemon owner. A small `layerfs-workspace-core` extraction is justified only by those two real users; move existing code rather than creating a second implementation. Core depends on canonical content types/builders, not FUSE, Docker, SQLite or host paths. Existing public facades re-export compatible types where possible.
3. **Backing boundary:** supplies immutable objects and owned written ranges, using existing host segments and bounded transport. Local-host placement invokes the same boundary directly; remote placement frames requests. I/O executes outside live-state locks.
4. **Host construction adapter:** consumes frozen changed records/ranges through existing #49 input/producer/finalizer code. Unchanged roots remain reused. It does not reconstruct a live Workspace or walk the complete namespace.

Reuse stable NodeId semantics, portable metadata validation, existing piece split/replace, sparse/zero handling, range equality and canonical algorithms. An owned written range may be backed by a charged pending buffer or acknowledged host segment ID+offset+length; extending the range backing reference must preserve compact sequential representation and current lifetime/accounting tests. This is location adaptation, not a second file-tree algorithm.

No FUSE mutation triggers full CDC/CAS publication. No full native-directory population at mount. No size/density/benchmark policy chooses an engine. Fresh versus incremental canonical output retains its actual construction-finality distinction.

## S4. State, authority and operation flow

Each mount owns Workspace ID, an unforgeable mount-lifetime identity, base snapshot identity, generation, shared live nodes/bindings, handles, charged clean cache, dirty pieces/facts, reservations, transfer status and lifecycle state. Mount identity scopes all handles, cache entries and backing requests. A reconnect is not a continuation of an old mount.

Lifecycle: `Active -> Quiescing -> Frozen -> PublishedPendingInstall -> Active`; errors with uncertain scope enter `Failed`; teardown enters `Closing`. These are one owner's states, not independent distributed state machines.

```text
authenticate/decode bounded request
  -> reserve request/result/dirty/byte capacity
  -> acquire missing immutable facts without state locks
  -> obtain affected-operation ordering permit
  -> short locks: revalidate identity/permission/generation, apply
  -> retain authoritative result/backing, release state locks
  -> send reply; schedule backing transfer as needed
```

A cached Attr is insufficient authority. Exclusive create checks the current parent binding under namespace coordination. Append chooses its offset and installs its range under that inode's ordering. A failed allocation or invalid argument cannot follow a success reply. Oversized supported user operations are processed in bounded chunks with their existing partial-result semantics.

If a lookup becomes stale while fetching, revalidate before mutation. Bound speculative retries; after repeated interference, park behind the affected object's ordering reservation rather than spin or restart an entire namespace scan. A reservation may exclude conflicting operations across I/O, but is not a state mutex and must not exclude unrelated workspaces or independent data operations.

## S5. Backing transport and acknowledgment

Keep one bounded ordered data channel per remote mounted Workspace and the existing independent lifecycle-control path. The data channel serves immutable acquisition, backing append/retention, acknowledged resolved facts, and fences. It is not another persistent Store.

Use the smallest mount-scoped monotonic batch identifier needed to bind replies and backing ownership. No persistent opcode log, automatic resend, lease service or generalized progress/reconnect framework. Batches may contain many operations; there is no mandatory grant/reply per metadata setter.

A backing acknowledgment identifies the last completely installed operation group in the resolved-fact order. All byte ranges referenced by that prefix must already be retained. Multi-object operations such as rename/replace install atomically in the recovery view; partial frames or partial operation groups never advance its acknowledged prefix. Coalescing cannot cross an acknowledged or observation cut.

| Boundary | Meaning |
|---|---|
| Local mutation success | Valid owner state and required local byte/fact ownership exist; all synchronous failure conditions within its authority were checked |
| Backing acknowledgment | Host accepted the identified range/fact prefix into its existing backing/recovery ownership; only then may transfer ownership be released |
| Logical observation | Coherent owner metadata/generation read; current session/diff fields do not require payload transfer |
| Backing fence | Required prior backing/facts for the declared synchronization cut are acknowledged and available |
| Commit | Existing expected-head publication and exact live checkpoint contract, with its actual configured durability |

Reserve transfer capacity before applying a mutation that needs it. Unacknowledged byte buffers and facts remain charged and available for local reads or explicit failure reporting. After acknowledgment, logical pieces retain host-range ownership; memory is released only when no active read still holds it. Host quota acceptance must be established before operations whose synchronous semantics depend on it: allocate each mount its declared bounded quota at admission and retain ordinary filesystem I/O errors at their established write/sync boundary. Socket capacity is not disk capacity.

Host range retention ends only after recovery/frozen-view references and execution-owner/read-plan references are released. A fact update or checkpoint cannot reclaim backing held by an older read. Reuse existing segment ownership with mount-scoped, ordered release batches; retain the charge until acknowledgment or explicit teardown resolves ownership. Local reads share the owner's retained range reference, so this does not require a host pin/unpin roundtrip for each read.

Namespace/metadata success cannot be followed by a routine host semantic rejection: the local owner has sole authority, shared validation rules, and reserved host fact-storage capacity. The host validates identity/framing/ownership and installs the resolved facts, rather than re-resolving mutable names. A disagreement is an integrity failure; mark the affected view unavailable and retain recovery ownership instead of silently reverting acknowledged state.

Backing I/O can fail after a buffered write, as allowed by the explicit baseline error contract. Retain failed pending data within its budget; stop further affected admission, report the error at the required boundary, and preserve successful prefixes. A recoverable ENOSPC is not automatically permanent failure of every mounted workspace. No blind mutation replay after an uncertain connection loss.

A read orders only relevant inode/alias predecessors; it does not globally flush unrelated writes. Current `session()`/`diff()` fields use a short coherent owner metadata/generation read without payload transfer or a global backing fence. Exec completion, explicit backing synchronization and freeze cover their actual required cuts. Metadata may coalesce between unobserved dependency boundaries only if local operation counts/generation/error semantics remain exact. Do not keep an independent opcode journal plus a competing final-state cache.

Before rollout, compare daemon-loss/host-loss/write/close/fsync behavior with the actual retained implementation and fill T4's failure matrix. Existing flags and historical receipts do not establish power-loss durability. In particular, retain host-acknowledged facts at successful synchronization/Exec completion so a daemon failure does not silently reduce the previously supported recovery boundary. Recovery views never masquerade as complete live state after an uncertain suffix.

## S6. Concurrency, locks and fairness

Use one short lifecycle/admission lock per Workspace, namespace topology coordination per Workspace, and per-inode ordering/state locks. Initially serialize conflicting namespace mutations within a Workspace using one namespace writer; allow independent file data operations and other workspaces to proceed. Cross-object acquisition order is namespace coordination followed by stable sorted inode identities.

```text
capacity wait / immutable fetch     NO state lock, NO execution-worker occupancy
ordering -> short state mutation   release state locks before I/O
backing/query wait                 parked bounded continuation
host observer/Commit drain         NO host Workspace or live-state lock
install final result               short exact-generation mutation
```

Never create `host Workspace lock -> Session drain -> host apply needs Workspace lock`. Host SDK operations release Store/manager locks before invoking the live owner. A host live read/edit uses that owner; immutable snapshot reads remain Store-local. SDK-originated edits coordinate kernel entry/attribute/data invalidation before conflicting cached observations resume.

Host-to-owner SDK requests use the control connection independently of the owner's backing/acquisition connection. A waiting host SDK handler holds neither the data pump nor a Store/Workspace lock needed by that acquisition. Ordinary SDK work consumes ordinary admission; it cannot consume the reserved cancellation/drain slots. This prevents an owner callback from needing a host service lane occupied by its own caller.

Use a small bounded ready-workspace queue and a fixed execution pool. Dispatch one bounded operation/continuation quantum per ready Workspace, then rotate. Park I/O/capacity waits instead of occupying every execution worker. Reuse existing threading/queue primitives, not a general actor framework. Each mount has bounded connection state served by shared nonblocking transport workers, so a stalled stream cannot occupy the complete execution or transport pool. No dedicated blocking transport/control thread per mount is added. Cancellation/control has reserved admission but cannot skip required mutation predecessors.

FUSE callbacks must copy borrowed payloads or retain reply/request objects only after charging their owned capacity. Kernel pending requests are distinct from admitted userspace work and must remain visible in overload reporting.

## S7. Numeric initial envelope

**Static review amendment:** the [100-workspace resource review](fuse-exec-100-workspace-resource-review.md) found hidden fuser receive/coordinator costs, lifecycle task multiplication, and missing capture/descriptor/global-admission ownership. The table below is provisional and is not a statically accepted complete envelope. That review governs the corrections, including an explicitly scoped memory-budget alternative; do not implement the old numbers by hiding omitted allocations. The user treats 100 workspaces as a design guideline supported by existing lower-volume evidence and source-based accounting. Physical 100-workspace exercises are deferred and are not a current implementation, optimization or completion gate. Preserve known resource findings and label capacity projections as inferred/unverified.


These are proposed finite design defaults, not measured/enforced properties of current code. Validate actual allocation charges and current supported workloads before adopting them. Resource configuration may vary by deployment; it cannot select different algorithms. Do not silently reduce an existing API's supported single-operation size; stream it within bounds.

| Resource | Initial envelope / enforcement |
|---|---|
| Mounted live workspaces | Design for at least 100 independent agent workspaces; 100 is the reference scale, not a mandatory physical test or an artificial cap. Actual admission follows explicit deployment resource limits |
| FUSE ingress | Initially at most 1 receive thread per mount, 100 aggregate; bounded ingestion only, not 100 execution workers; count actual fuser helper threads before adoption |
| Filesystem operation workers | 2 across the Linux daemon, shared fairly; at most 2 state operations per Workspace |
| Ordinary admitted requests | 128 aggregate, at most 8 per Workspace; includes one reserved ordinary slot for each of 100 admitted mounts; request bytes have their separate stricter budget |
| Per-mount progress reservation | 1 ordinary request, 256 KiB transfer space and 64 KiB operation/result space per mount, inside aggregate budgets; 25 MiB transfer and 6.25 MiB operation reserves at 100 mounts; unavailable to other mounts |
| Control admission | 2 reserved entries per mount; included in managed-memory envelope |
| Transport workers | 2 shared nonblocking I/O workers per side for all mounted data/control connections; bounded per-connection state; no per-mount transport thread and no execution-worker permit held by I/O waits |
| Live nodes/bindings/dirty ranges/handles/cursors | 32 MiB per Workspace, 128 MiB aggregate, charged by actual owned capacities |
| Clean immutable cache | 8 MiB per Workspace, 32 MiB aggregate; only clean entries evict |
| All transport/request/reply/pending-byte buffers | 8 MiB per Workspace, 32 MiB aggregate, including simultaneous sender/receiver/local-read ownership |
| Output slabs | 256 KiB each, at most 4 queued globally inside the transport budget; larger user requests split through bounded frames |
| Ordinary write chunk | Retain supported negotiation up to 1 MiB; no payload allocation before reservation |
| Execution scratch/results/order/control data | 16 MiB aggregate; control capacity explicitly retained under saturation |
| Application-managed allocation envelope | 256 MiB execution side; same explicit 256 MiB host transport/recovery envelope, separately enforced and reported |
| Handles | 8,192 per mount / 65,536 aggregate, also charged to live-state bytes; negotiated deployment limits must be compatibility-checked |
| Directory acquisition | At most 128 entries and 256 KiB per page; stable bounded cursor, never whole-directory reply required |
| Unlink/change batches | At most 512 entries and wire-byte bound; preserve per-operation ordering/errors |
| Backing retained bytes | Existing 1 GiB per Workspace maximum; explicit 100 GiB host aggregate ceiling for 100 quotas, including partial/dead-but-pinned ranges; allocate backing on demand, not 100 GiB at mount; actual host disk capacity and errors remain explicit |
| Host construction CPU | One shared configured producer allowance, initially at most 8 active workers across construction jobs; do not multiply by mounts |
| Host service CPU | At most 2 active processing workers plus 2 shared nonblocking transport workers; construction uses its separately declared allowance |

**100 mounted workspaces does not allocate 100 full per-workspace memory budgets or 100 sets of workers.** The shared execution pool stays at 2 workers, and execution managed memory stays at 256 MiB. Per-workspace maxima are ceilings; aggregate limits and reservations are enforced simultaneously. Idle mounts retain bounded identity/root/handle/connection state and acquire namespace/content lazily. All 100 may have concurrent clients; requests exceeding active capacity wait fairly or receive their defined resource error, rather than denying mounts merely because only two workers execute at once.

The initial 100 FUSE receive threads plus 2 execution and 2 transport workers require an explicit thread/stack/PID census, including lifecycle helpers and child applications. Preserve the existing 256-PID container limit for the matched benchmark deployment. Mount count does not promise capacity for 100 arbitrary process trees under that limit. No hidden per-mount worker pool is allowed. Use the existing transport runtime's readiness facilities where available; readiness waits must service control and data without a socket-blocked thread per connection. This is a required adjustment from the earlier 8-mount proposal, not an optional optimization.

The 256 KiB per-mount transfer reservation supports bounded progress frames; negotiated writes up to 1 MiB retain their API support. A larger callback must reserve additional shared capacity before copying its borrowed payload, or wait at ingress without a state lock. Do not pre-copy 100 one-MiB requests into an uncharged queue. The aggregate transfer budget remains 32 MiB; the initial progress reservations consume 25 MiB of it. Actual framing/read-plan/ack ownership must fit the remaining capacity or safely retain/wait within the mount's own reservation.

The per-mount progress reserve prevents another mount's retained dirty buffers from consuming every ordinary-operation slot/byte. Shared surplus is allocated fairly. A mount that exhausts its own retained-state quota must return the declared resource error or wait on its own releasable work; it cannot wait indefinitely on another mount's quota. Control reserve is separate. The guarantee is progress of admitted supported work or a bounded explicit error, not unlimited successful writes into full storage.

Existing protocol fields may support larger frames than one write chunk. Parse incrementally within reserved byte budgets, validate lengths before allocation and preserve supported operations through chunking; do not allocate the legacy maximum frame for each pending request. Fix all transport header/request counts before enabling new wire version.

The managed-memory subtotal above leaves reserve under 256 MiB; the implementation must produce an actual simultaneous-ownership equation covering every allocation site, not just this table. Worker stacks, allocator overhead, SQLite/native caches, kernel page cache, socket buffers and VM/system memory are separately reported. A managed counter is not a hard RSS guarantee. Preserve Docker 2 CPUs / 2 GiB / no swap / 256 PIDs; its cgroup does not cap the macOS host. Finite host workers/admission bound application concurrency/work, not a hard host CPU percentage. Do not claim unsupported process/system quotas or require new scheduler privileges.

All reservations use checked arithmetic and are acquired before dependent mutation. When budgets fill, wait without locks or return the established resource error; never evict acknowledged dirty state or rely on OOM as normal flow control. Retry loops are bounded, no busy polling, and long supported traversals yield cancellation-aware bounded work units. Admission of another mount includes its baseline state, thread stacks and quota commitments.

## S8. Kernel behavior, ordinary tools and portability

For the first Linux comparison retain the tested Linux 6.12.76/fuser 0.18 environment, `DefaultPermissions`, NoDev/NoSuid/NoAtime and existing one-second name/attribute TTL. Preserve the capability intersection. Keep writeback cache disabled; retain `DIRECT_IO|NOFLUSH` only for the already-tested newly created handle class and ordinary-open KEEP_CACHE behavior. RELEASE/lifetime and explicit fsync/error delivery remain independent obligations.

Do not equate max_background/congestion settings with all foreground request or worker limits. Explicit kernel invalidation and forced stat can cause requests even before TTL expiry. No TTL extension, permission bypass, kernel upgrade, DAX, passthrough or io_uring dependency is part of this core design. Any later capability experiment has its own platform support, privilege, coherence and resource checks.

Preserve supported atomic create/exclusive-create, append offset selection, truncate, rename/replace, hardlinks, symlinks, open-unlinked handles and exact partial writes. Use an authoritative directory cursor with stable cookies and bounded retained state; never use an index into a changing vector. Preserve existing error mappings; explicitly specify unsupported operations.

Define same-mount advisory locking and mapping behavior for supported tools before claiming installer/editor compatibility. Retain current normal-open mapping support and document created DIRECT_IO-handle limitations. Supported writable mappings/dirty-page paths require a consistent kernel-to-owner operation cut. An ordinary open writable handle or running process is not a reason to reject Commit or wait for close. Do not enable unsupported writeback behavior, silently downgrade supported mappings, or remove guards without replacing their consistency responsibility. Avoid claiming full POSIX/Windows semantics from a tiny-file benchmark.

SDK mutations use per-operation ordering and required kernel coherence; they must not reject solely because commands run or ordinary fds remain open. Userspace ordering alone cannot stop a cache hit in the kernel. Replace blanket process/writer-count exclusions while preserving their actual data-consistency and notification responsibilities. Required entry/attribute/page invalidation completes before SDK success and resumption of conflicting access. If invalidation fails after mutation, retain the exact result and fail the affected presentation; do not reopen with stale cache or blindly replay the edit.

Platform adapters map credentials, names, cache notifications, handles and security semantics. Native Windows deletion/ACL/name behavior is explicitly different; a future WinFsp adapter needs its own compatibility tests. macOS/Windows hosting a Linux workload still uses the Linux adapter. Local host placement uses direct core/backing calls, not mandatory loopback RPC. Initial portability means architectural separation and preserved semantics, not shipped untested adapters.

## S9. Commit, observers, handles and teardown

Commit must succeed while multiple commands/processes remain running against the Workspace. Shell/Exec is a launcher surface, not the snapshot boundary. Remove managed-command Busy gating and do not require ordinary writable handles to close. Define the boundary at actual filesystem operations, including external callers and supported kernel dirty-page/mapping paths. Processes, cwd, mount and handle identities continue; zero-blocking mutation progress during publication is not required initially.

Commit closes mutation admission, drains admitted operations without state locks, completes required backing/recovery acknowledgments and freezes the known dirty delta. Waiting external mutations remain bounded at ingress; immutable reads may use retained stable plans where safe. Host #49 construction reads frozen metadata/ranges and unchanged canonical roots; no complete namespace scan or reconstructed live Workspace.

Release/forget/cancellation and required drain completion remain admissible through reserved control capacity. Queue conflicting mutations at the cut instead of rejecting them merely because Commit is active. Do not wait for command exit or an ordinary writable fd to close. A bounded initial mutation pause may cover construction/publication/install; queued post-cut writes then resume as subsequent dirty state. Actual deadline/cancellation/failure may abort the cut under the existing error contract after resolving ownership. Publication/install uncertainty follows the retained-outcome rule below.

After conditional expected-head/base publication, retain its exact outcome and candidate installation facts until the same owner checkpoints existing nodes/handles and clears only that frozen change set. A lost installation reply cannot trigger another Commit. Failure before publication preserves the old visible head and retry/discard state. Failure after publication remains `PublishedPendingInstall`; reopen live mutation only after exact installation/recovery succeeds.

`session()`/`diff()` observe the owner's current metadata/generation without freezing the Workspace or waiting for payload transfer; `sessions()` copies registry references before invoking owners. Per-file SDK reads/edits use only relevant ordering. Cache invalidation covers changed parent/name entries as well as inode attrs/data. Unlink removes a name, not a live handle's storage. Segment/root references outlive reads and aliases as required; namespace deletion never reclaims immutable history.

Normal unmount closes admission and applies the declared drain/Busy timeout policy. Forced teardown returns explicit errors to remaining requests and records recovery status; it never presents the last committed snapshot as though unsynchronized writes had succeeded there. Release ranges/handles/reservations once. No automatic reconnection of ambiguous mutating calls; bounded recovery/discard is explicit.

## S10. Implementation boundaries and deletion ledger

The [implementation plan](issue49-fuse-exec-implementation-plan.md) supplies concrete C1–C8 component transfers and P0–P6 execution order. Start with the minimum portable ownership/acquisition seam needed by the create-100 vertical slice, rather than extracting every subsystem before demonstrating useful behavior.

Start from current main with the finalized Commit implementation. Research #48 source is a reference for mechanisms/evidence, not a patch-migration prerequisite. Preserve source identities and implement the new shared path directly; do not overwrite current files from the research worktree. A single owner controls core/Workspace/Store seams and preserves separately owned work.

| Area | Work |
|---|---|
| Existing Workspace live files (`cow_tree`, `file_edit`, live portions of `file_io`, limits) | Extract/reuse the shared live core, operation preflight/installation, read plans and range ownership; do not duplicate algorithms |
| `layerfs-fuse/{filesystem,adapter,handles,host_mount,port}` | Thin adapter over owner operations, real handle/cursor identity, coherent replies/notifications and bounded admission |
| `layerfs-daemon` | Own bounded mount registry, executor scheduling and existing lifecycle integration |
| `protocol` and host service | Bounded acquisition/backing/fact transfer with mount-scoped ownership and acknowledged prefix; no generic distributed framework |
| Workspace runtime/projection/SDK routing | Invoke one owner for live state; freeze/observer/unmount coordination; prevent host-lock/drain cycles |
| Store/host `changes` construction adapter | Consume frozen changed inputs through existing producers/builders, stage/publication and exact checkpoint facts |

Retire `proxy_client`'s competing pending-create/metadata/live-binding decision paths, per-page authority negotiation, ad hoc category-transition drains, duplicate host live mutation entrypoints and unused identity wrappers only after their real semantics transfer. A rollback uses the retained known-good build, not a permanent runtime old/new policy engine. Keep wire codec, kernel bridge, current validators, canonical code and backing mechanics where reusable. Do not create empty abstractions or multiple live/transport/effect frameworks.

## S11. Verification and falsifiable rollout

Test identifiers below are referenced by the checklist map. Each is a focused group, not a demand to rerun all tests on every edit.

| ID | Required checks |
|---|---|
| T1 Adapter/ordinary semantics | create/exclusive-create/write/truncate/append/rename/link/unlink; directory cookies; permissions/errors; supported locks/mapping; Linux notifications and external-launcher access |
| T2 Canonical/continuation | fresh/incremental/zero/captured state, unchanged extents and old roots, exact reference counts, NodeId/handle continuity, private preview, published-install retry |
| T3 Concurrency/coherence | simultaneous same/different inode operations, opposing renames, two commands/open handles alive across successful Commit, writes before/after cut, external access, same cwd/mount/handle continuation, owner SDK edit/read, observer during drain, stale acquisition/replaced mount, namespace-only/no-op Commit |
| T4 Failure/durability | partial append and cleanup failure, resource exhaustion before mutation, host I/O failure, lost reply/no replay, mid-transfer disconnect, daemon/host loss at every acknowledgment cut, cancellation/unmount with open readers/mappings |
| T5 Resource/load | Current review: existing lower-volume receipts plus source-based thread/buffer/descriptor/admission accounting and clearly labeled inference to 100 agent workspaces. Physical 100-mount idle/concurrent/saturated/stalled/teardown scenarios are deferred reference qualification, not a current completion gate. Preserve applicable lower-volume semantic/resource checks within their agreed scope |
| T6 Performance/custody | exact optimized baseline vs replacement source/recipe, diagnostics off, complete Exec/Commit/End, CPU/RSS/backing and queue/lock waits, explicit actual work/copies/acquisitions/acks, unchanged raw history |
| T7 Integration/adoption | actual #48/#49 source adoption, native direct-host parity, private SDK/reconciliation behavior, deleted caller ledger, no unsupported platform or combined-phase claims |

Deferred runtime sequence (do not launch or schedule for this review; physical 100-workspace exercise is not a prerequisite for current lower-volume work):

1. Start from current main and reuse its source-bound baseline where applicable, using unchanged mixed-v3 tier100 definitions. Learn from retained research mechanisms without requiring Exec patch migration. Existing research timings are separate-source context; do not add favorable phases from different products.
2. Implement a vertical slice of known-state metadata/binding operations in the shared owner, with bounded backing/fact acknowledgment and SDK observation. Keep kernel flags/TTL unchanged. Run its semantic tests, then one selected real-FUSE sample. Hypothesis: callback decision/host dependency time decreases without moving equal work to Commit/End or worsening load behavior.
3. If roundtrips drop but elapsed does not, inspect request receipt -> core -> reply -> application resumption. Do not expand transport machinery to explain unattributed time. Replan the limiting layer rather than promise a rewrite gain.
4. Transfer the remaining supported operations through the same core; include one two-workspace stall test and bounded saturation test early. Add broader concurrency/platform checks only when the implemented surface requires them.
5. Retire old execution paths after integrated semantic coverage; collect final selected evidence/proofs under the existing workflow. Stop unnecessary tuning once the applicable accepted outcome is satisfied; #47 complete-lifecycle obligations are separate from an Exec-phase milestone.

The preferred create-100 working objective is approximately 0.7000–0.8000 s complete lifecycle, with indicative Exec around 0.3000–0.4000 s and preserved Commit efficiency. This is higher ambition than reproducing the research result, not a measured forecast or a new hard phase threshold. The formal parent lifecycle requirement remains below 1.0000 s per tier100 create/delete case; avoid prolonged tuning for minor differences from the preferred band. No speedup magnitude is promised. Acceptance requires a measured improvement in the chosen complete workload or a specifically agreed simplification outcome, with no hidden resource/overload tradeoff. Fewer messages, higher CPU consumption or a faster isolated phase are insufficient. Freeze numerical performance targets in the applicable issue before qualification rather than invent them here.

All builds/tests/samples use the shared host measurement lock; preserve other task artifacts. Iterate one substantive change and one selected performance sample. Independent sampled proofs remain final-stage only, 45 seconds work / 59 seconds hard end-to-end. Performance diagnostic allowances retain their separately approved values; a longer watchdog is not a looser pass threshold. Each focused overload/failure check is deadline-bounded and reaps its own workers. Preserve rejected hypotheses and do not search for favorable repeats.

## S12. Design acceptance and remaining evidence obligations

The 100-workspace resource target is a design reference, evaluated now through existing lower-volume measurements and source-based accounting. No physical 100-workspace run is required for the current work. Report measured lower-volume outcomes and inferred larger-scale feasibility separately; retain unknown contention, tails and process/RSS costs without converting them into a new blocking verification campaign.

The accompanying map covers every unchecked requirement as a design/test obligation. Nothing is marked implemented by this spec. Before implementation commits, finalize the core extraction/caller map, exact wire bounds/version, allocation charge equation, existing acknowledgment/durability matrix and platform support table. These are concrete interface inventories and compatibility checks, not permission to leave behavior unspecified.

If fixed local authority plus host backing cannot preserve synchronous errors and acknowledged recovery using the bounded window, do not silently acknowledge remote work. Keep the necessary synchronous boundary for that semantic operation while revising the same core/transport contract, or reject the design slice. Do not introduce benchmark-specific exception engines. If the shared live-core extraction is larger than justified, reduce the migrated surface with a vertical implementation sequence rather than ship duplicate authorities.

## Sources

- [Redesign requirements](fuse-exec-redesign-requirements.md), [#49 producer plan](issue49-shared-producers-plan.md), and source-bound retained research listed in S2. Proposal texts are not performance evidence.
- [Linux 6.12.76 FUSE file operations](https://raw.githubusercontent.com/gregkh/linux/v6.12.76/fs/fuse/file.c): NOFLUSH, RELEASE and synchronization behavior.
- [Linux 6.12.76 FUSE directory operations](https://raw.githubusercontent.com/gregkh/linux/v6.12.76/fs/fuse/dir.c): explicit invalidation, forced attribute reads and permission paths.
- [Linux FUSE I/O modes](https://www.kernel.org/doc/html/latest/filesystems/fuse/fuse-io.html): direct/cached/writeback behavior and coherence assumptions.
- [Linux 6.12.76 passthrough](https://raw.githubusercontent.com/gregkh/linux/v6.12.76/fs/fuse/passthrough.c): backing/privilege/lifetime requirements; excluded from first implementation.
- [WinFsp native API versus FUSE](https://github.com/winfsp/winfsp/wiki/Native-API-vs-FUSE): future Windows adapter differences.

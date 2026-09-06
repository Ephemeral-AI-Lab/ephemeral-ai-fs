# Issue 49 implementation ledger

Status: P0 source custody complete; portable core, frozen construction and owned read/write reply seams implemented through P3h. Execution-owner activation, remaining supported transfers and P4-P6 are incomplete. No rewritten-product performance claim.

## Source custody and frozen workload

- Fetched `origin/main` on 2026-09-06: `ccffed469a96f2877cfa87eb4138d944dc14430b` (actual starting SHA).
- Implementation branch: `codex/issue49-live-workspace`; isolated worktree `/Users/yifanxu/.codex/worktrees/issue49-live/layerfs`.
- Original checkout, peer worktrees, research source, preparation masters and benchmark artifacts remain independently owned and untouched.
- Read the C1–C8/P0–P6 implementation plan, spec, requirements, 109-item map, resource review, completed producer ledger, benchmark AGENTS/QUICKSTART/general rules before implementation.
- Read retained research mixed-v3 baseline, next investigation and residual findings. No research patch migration. Grant/page reductions and acknowledging-setter experiment have no demonstrated elapsed improvement; do not repeat them.
- Primary case: `tiny-bulk-create-100-mixed-v3`, seed 1; 1,000 files / 104,857,600 bytes including 50 MiB file; separate 200-file / 1 MiB witness. Preserve normalization/synchronization/generator.
- Recipe document SHA-256: `c85fdcdbabb568a395c837463c0a2701db86827bd0ad575a0e59ac03ee43ccef`.
- Host owns SDK/coordinator, SQLite, physical spool, construction and publication. Docker owns Linux daemon/FUSE/workload/live core. Container remains 2 CPUs / 2 GiB / no swap / 256 PIDs; host resources separate.
- Existing finalized reference create Exec 0.9172 s / Commit 0.3587 s / lifecycle 1.2914 s; research 0.4792 s / 0.4718 s / 0.9680 s. Different products, never additive phases.
- Builds/checks/resource work use existing `$TMPDIR/layerfs-infra-measurement.lock`; sample reset is runner-owned `--setup clone`. Final sampled proofs only after stable create/delete strict <1.0000 s.

## Caller / dependency / deletion ledger

| Component | Current callers and retained responsibility | Transfer / deletion status |
|---|---|---|
| C1 identity/live state | Workspace `cow_tree`, host `FuseView`, FUSE `port` and adapter | Shared identity/state, metadata, create/mkdir/symlink, acquisition installation, pin/reclaim transferred; link/unlink/rename and lifecycle transfer pending |
| C2 ranges | `file_io` live reads/writes/truncate/SDK edits; `changes::FrozenFile`, direct construction reader; checkpoint/reclaim | PieceTree and owned BackingRef shared; write/truncate use PreparedFileEdit; host HostSpool isolated; SDK multi-edit transfer pending |
| C3 adapter/proxy | `LayerFs` callbacks → synchronous `FilesystemPort` → `ProxyClient`/host `FuseView` | Read/write own their replies at submission; native/proxy synchronous defaults remain until runtime owner switch. Other submissions and proxy retirement pending |
| C4 runtime/fuser | `ActiveMount`, `handle_mount`, helper binary, `HostMount` | Maintained request-size/one-loop patch tested natively and Linux; mount ownership transfer pending |
| C5 backing/capture | `SpoolSegment`, append/rollback/retirement, Running/Ready capture | Physical resources remain host-side; shared admission pending |
| C6 construction/checkpoint | `StableFileInputs`, `FrozenFile`, journals, frontier/references, `PreparedCommit` | Existing producers now consume FrozenWorkspaceChanges through host CandidateInputs; detached changed-input test passes. Exact installation transport pending |
| C7 SDK | `WorkspaceWorker`, lifecycle observer/edit/Commit, projections, preview/reconcile | Current semantics retained; one-owner routing/cut pending |
| C8 commands/packaging | daemon exec/start/watch/pump; Docker launcher/helper image | Tracked command start reservation implemented and Linux-tested; consolidated supervision pending; helper remains a real caller |

## Confidence / bounded exploration

| Seam | Decision / evidence | Remaining exit |
|---|---|---|
| Backing representation | Retain one-word cloneable reference in each Piece/compact slice. Adapter resource ownership is retained through the last reference; no Unix File type in core. | Check compact layout, interleaved rollback, read after unlink/checkpoint and owner-drop lifetime |
| Error vocabulary | Preserve existing InvalidInput/Integrity/NotFound classes and messages at native boundary; no Store dependency in core. | Compile/check actual adapters |
| Range instrumentation | Return tree visit count from core acquisition; host records existing Commit diagnostic. | Preserve counter call scope |
| Prepare/acquire/apply | Must bind exact inode/version and appended range, reserve first, reject stale install without replay. | P1/P3 implementation; no synchronous ObjectRead under core locks |
| Callback suspension | Must own charged callback/reply and park I/O outside filesystem workers. | P2/P3 concrete interface before executor activation |
| Frozen inputs/capture | Retain known dirty frontier, exact checkpoint journal, optional host capture fallback. | P3 concrete input and admission adapters |
| fuser allocation/lifecycle | Review every supported request maximum and INIT; retain decoder/replies. | P2 narrow patch and source accounting |

## Evidence

No new build, test, performance sample or proof at P0. Resource scale 100 remains inferred/unverified; no physical campaign scheduled. Requirements are not marked implemented from this ledger.

## P1a — portable identity and retained range checkpoint

Native Workspace and the actual FUSE `port` now depend on `layerfs-workspace-core`. `NodeId`/`ROOT`, `Kind`, `Attr`, `Node`/`Data`/`FileData`/`DirectoryData`, `Node::attr`, ResourcePolicy and the existing PieceTree algorithms have one definition. Removed the old Workspace `file_edit.rs`/`limits.rs` implementation files and FUSE identity/attribute conversion bodies. This is shared data/range code, not yet daemon live-operation ownership.

`BackingRef(Arc<Backing>)` replaces `Arc<SpoolSegment>` inside Piece/SpoolSlice and the native registry/construction reader. It remains one machine word per reference. Each segment adds an Arc header, BackingId and a boxed adapter resource; this fixed per-segment cost is distinct from the inherited 8-byte **logical** compact-piece charge (which is not actual heap accounting). Equality uses owner-reference identity, so two mounts using the same numeric ID cannot coalesce. Host `spool_segment(&BackingRef) -> StoreResult<&SpoolSegment>` checks placement. The portable core contains no Store/SnapshotReader/SQLite/Unix File/fuser dependency or physical I/O. Host retirement requires a unique backing reference; held plans/frozen inputs keep the physical resource and its physical accounting alive through the last release.

`PieceTree::range_with_visits(start, end) -> Result<(Vec<Piece>, usize)>` returns the existing visit count; native ReadPlan records it at the same call boundary. Core InvalidInput/Integrity/NotFound errors map to the same Store classes/messages in the native adapter.

Checks under the shared measurement lock, Rust 1.85.1, two build jobs:
- `cargo check -p layerfs-workspace --no-default-features -j2`: PASS, 9.13 s first build.
- `cargo test -p layerfs-workspace-core -p layerfs-workspace --no-default-features --lib -j2`: PASS, 50 Workspace tests (1.09 s) and 9 core tests (0.05 s), 4.98 s final build.
- New ownership check retains a sliced old range after the live tree and registry reference drop, releases the resource exactly once, checks one-word reference size and separate-owner identity.
- Inherited compact/fragmentation/sparse/limit tests moved into core. Native tests retain actual host-file rollback/physical high-water, open-unlinked/old read, captured output, old snapshots and exact checkpoint-install recovery coverage.
- Two intermediate test compilations caught incomplete caller migration (obsolete converters/test file access, then borrowed/materialization NodeId conversions). Corrected the adapter scope rather than changing semantics. No runtime test failed.
- Rust 1.96.0 formatting applied only to the affected packages.

No performance build/sample/proof: this extraction has not yet changed live placement and is not P3's coherent create candidate. Next: portable read plans and prepared write/version ownership, then minimal runtime entry. All C3–C8 supported-operation transfers remain pending.

## P1b — live metadata and portable read preparation

`Workspace.live: LiveWorkspace` now holds the actual native inode table, dirty set, mutation generation and mutation-path map. Native `attr`, `chmod`, `set_mtime` and mutation recording invoke the moved shared bodies. Construction, capture, SDK observation, reconciliation and checkpoint installation address those same nodes through the field; no second inode table was introduced. Namespace acquisition/mutation and write preparation are still being transferred; daemon ownership has not switched.

Core `ReadPlan::for_file(&FileData, offset, size)` owns only canonical roots/pieces/requested length/tree visits. Native `ReadPlan` retains SnapshotReader and executes physical reads separately. Inspection found inherited edited-file reads beyond EOF called `PieceTree::range` with start > end. The new base/edited EOF regression failed with InvalidInput("file range") before the shared planner clamped start to EOF, then passed for EOF, EOF+1 and u64::MAX. Eight affected native file-I/O checks passed after this change.

Moved metadata bodies check generation capacity before changing attributes and update every known alias path in the same generation. New core regression covers alias metadata/generation, invalid nanoseconds and u64::MAX rejection without attribute changes.

Verification after inode-table transfer: 50 native Workspace tests and 11 portable-core tests PASS (Rust 1.85.1, build 6.62 s; test execution 1.09 s / 0.05 s). The field migration initially exposed two retained capture accesses and a chained projection observer access; all now use the same live table. Rust 1.96.0 affected-package Clippy with warnings denied PASS. An earlier Clippy invocation used the wrong 1.85.1 toolchain and reported two unchanged Store findings; CI specifies 1.96.0, and no Store algorithm was changed for those findings. Added conventional `is_empty` methods for the newly public piece types and removed two redundant native borrows.

Remaining P1 exit: exact prepared inode/version, range and resource ownership across backing work. P2–P6 remain open. No rewritten-product performance result or terminal PASS.

## P1c — exact prepared writes through the native caller

`LiveWorkspace` now also owns the existing logical spool/peak, inline/piece charges, edited-inode set and ResourcePolicy; physical segment bytes, descriptors, files and capture remain host-side. `Node.revision` advances on shared metadata and file edits and is retained by exact rollback. Native write preparation no longer converts Base to Edited before validation/backing success.

Concrete boundary:

```rust
LiveWorkspace::prepare_write(&self, node: NodeId, offset: u64,
    bytes: usize, backing: Option<SpoolSlice>) -> Result<PreparedWrite>
LiveWorkspace::apply_write(&mut self, prepared: PreparedWrite) -> Result<usize>
```

PreparedWrite owns the exact inode/revision/value and chosen range, is not Clone, and is consumed on installation. The existing PieceTree splice, zero gap, edit/inline/allocation/length/spool checks are shared. Apply checks exact prepared state before installation; unrelated inode metadata can progress without invalidating the prepared file. Native write keeps physical append/check/rollback and capture feed, then calls shared apply once. Failed/stale installation never repeats the append. Full old node comparison additionally covers current native alias/pin changes while remaining namespace algorithms transfer; native namespace revision stamping and daemon ordering/admission still belong to P3/P5.

The adapter must retain affected-inode ordering and resource admission across prepare/acquire/apply. The native caller currently has exclusive Workspace access. This checkpoint does **not** claim concurrent daemon reservations or worker suspension is implemented; those must exist before activating the remote caller. Resource checks at apply preserve safe rejection if limits/state changed. Existing host segment_bytes counts an appended unused tail until retirement, independently of logical spool bytes.

Verification under the shared lock: 50 native Workspace tests and 12 core tests PASS (Rust 1.85.1; build 6.48 s; tests 1.10 s / 0.05 s). New core test checks no preparation mutation, unrelated-inode progress, revision rejection including a same-value metadata mutation, exact byte/range/overflow/quota checks and retained backing. Additional native stale-append injection PASS (0.02 s): physical append occurs once, old content survives, all 8 physical bytes remain charged versus 5 logical bytes, further over-quota write is rejected, an old read pins the complete segment across Commit, and retirement occurs after its final release. Affected-package Rust 1.96.0 Clippy with warnings denied PASS (2.17 s). Two constructor call sites needed the moved policy parameter; no test semantics were weakened.

No performance candidate yet. Next is P2's minimal safe ingress/runtime entry, then P3 cold parent acquisition/create/metadata/backing/frozen Commit/SDK/continue on the same owner. Truncate and multi-edit still use the retained native body and will transfer with their actual exclusion/rollback callers before enabling the supported rewrite.

## Contract correction and P2 ingress checkpoint

Merged `origin/main` through `4c98995e4` normally at `56abd490969477810cc1e34ccc89477846947ea5`, preserving implementation and incorporating the user-authorized command-independent Commit correction. The original starting SHA remains ccffed469. Commands and ordinary writable fds must span Commit/SDK operations; command counts are launcher/resource/teardown facts only. P3/P5 must replace Busy/close gating with a bounded filesystem-operation cut, handle supported dirty-page/mapping coherence, queue post-cut mutations and checkpoint the same nodes. Merely deleting the old guards is not an implementation. Add two-command/handle, external-access and before/after-cut focused checks. Earlier ledger references to preserved native tests describe historical checks, not the final corrected acceptance contract.

P2's narrow maintained fuser patch is under `crates/vendor/fuser`, selected through the root manifest and lock. Its [provenance, request-size audit, resource arithmetic and focused check commands](../../../../crates/vendor/fuser/LAYERFS-PATCH.md) identify the upstream release and the two changed source files. It reuses the decoder and reply machinery. Single-loop run eliminates one joining/receive-thread multiplier; helpers and other runtime task owners still remain until caller transfer. Linux INIT/negotiation determines actual receive allocation with room for ioctl/xattr/retrieve and fixed/name request classes. No kernel flag/TTL/writeback/permission change.

Shared-lock focused results: socket-backed INIT/decode/reply/direct-run/exactly-once-destroy PASS; aligned buffer and negotiated/max/default-page/overflow checks 2 PASS. Initial dependency compilation took 29.96 s; revised INIT test build 4.08 s; buffer test launch 0.06 s, test bodies <0.01 s. These are native component checks with macos-no-mount, not real-FUSE or future-platform support evidence. Root native Workspace check after patch resolution PASS (1.22 s); root Cargo.lock selects the vendored package. An initial root `-p fuser --features` test invocation was rejected because it is deliberately excluded from workspace membership; the standalone package command fixed the invocation without changing membership or product behavior.

P2 runtime admission/mount integration and P3 ownership transfer remain pending. No new performance sample or proof, and no 100-workspace run.

### P2 build custody

Matching sealed host and Linux image build PASS at product-source commit `09d9325b6d5a39a2f3567978a61c1f4c497a7f18`:
- Source seal `5cf1d381534e03e6d0c0bf9f1ecf69285cd6557534faf753d86b8465e9ee3e85`.
- Product seal `b537c90f3d16e117d58f3b5e246825f9b7bad835024aaa7d6df9e6b4952908b5`.
- Host binary SHA-256 `201c1c68c15ad6546efa404ee5acc5d97e75aa2d51e898fe945ab3bfb9887b5a` and identity at `target/release/fs-benchmark-pro.identity.json`.
- Image `layerfs-bench-infra:5cf1d381534e03e6`; inspected labels match both host seals. The existing runner records source-dirty=true even for a clean checkout; preserve that producer field.

The first image-build invocation failed before building because the configured desktop-linux Docker socket was absent. Docker Desktop was stopped (only vmnetd remained); started the existing application with `docker desktop start`, without restart/reset/prune or context changes. Retained runtime reports Docker 29.5.2 / Linux 6.12.76-linuxkit, VM 8 CPUs / 4,108,828,672 bytes; these VM settings are not the sample container's required 2-CPU/2-GiB/no-swap/256-PID limits. The subsequent image build passed through the existing measurement-lock runner. No performance/fixture/proof was run.

## P2/C8 — tracked command starts outside the registry lock

Daemon Active records now distinguish an admitted start (`pgid=None`) from a started process. `reserve_execution` validates owner/mount/capacity/identity and inserts the starting record under a short lock; the handler then spawns without that lock; `install_execution` publishes its PID only while ownership/mount readiness and cancellation still agree. ActiveGuard keeps the starting record visible until success/failure is resolved. Error socket replies are outside these locked helpers. Owner loss and mount teardown mark cancellation under the registry lock before collecting already-started PIDs for signaling outside it, closing the otherwise possible None-to-PID race.

The original combined admission limit and per-command output/watch supervision remain pending replacement by C4/C8's distinct admission/shared supervision. This checkpoint does not claim those lifetimes or PID multipliers are consolidated. The mount helper remains a real caller.

Added an explicit `runtime-check` Docker build stage to the existing Dockerfile; ordinary final image builds do not depend on or run it. It exercises only daemon/fuser components, no Store/SQLite/benchmark coordinator. Linux checks PASS: all 5 daemon binary tests, including starting reservation/cancellation/capacity/normal process status and existing cleanup/close/resource tests; fuser INIT/single-thread dispatch/destroy test; 2 receive-buffer checks. The actual Linux session test additionally confirms its negotiated receive capacity. Existing caches are reused; first unit-test dependency builds account for most of this stage's 56.1922 s external build duration. These are component checks, not independent benchmark proofs or real mounted workload timings.

Receipt directory: `benchmark-results/issue49/runtime-check-1788657756537002000`; raw stdout/stderr, exact command/identity/returncode/wall/timeout receipt and SHA-256 manifest retained. Source seal `b9fda2dedbc4bb5a51a493be01dfe6c08592489f8e0c1132ea329eadeb83d494`; product seal `fa709788dd70dee842140a519338299e29db20d05668c8399f723ac3d7031f79`. Exit 0, no timeout or output truncation. No create/delete performance sample yet.

### Prepared-write handle independence correction

The expanded prepared-write regression first failed because full-Node equality rejected a pin-only change. PreparedWrite now retains only the exact inode revision and prior FileData, plus its proposed file/range state. Pin accounting is not a content revision; release/pin changes can proceed without aborting a valid prepared write. Actual content/metadata revision changes still reject it. Installation records mutation paths from the current same inode, preserving alias routing without copying every path during preparation. This replaces the earlier P1c full-node capture; pending operation/handle lifetime and namespace ordering remain explicit runtime obligations.

The focused test now passes with a pin increment between prepare/apply and still rejects content changes and same-value metadata operations that advanced revision. All 9 affected native file-I/O checks PASS, including stale physical append and old-reader retention. No Linux/runtime suite rerun: this change does not alter their tested seam. No performance claim.

## P3a — resolved-name/create mutation seam

Core `namespace.rs` now owns existing directory/path access, checked NodeId allocation and file creation. `prepare_name(parent, name)` returns `NameLookup::Ready(ResolvedName)` for owned binding facts or `Acquire(NameInput)` for missing immutable directory input. `resolve_name` checks the exact parent revision/base after acquisition. `create_file` consumes the resolved binding, rejects existing/stale/invalid reserved identity and generation exhaustion before semantic mutation, then updates the same live node/directory/change tables. Directory mutation advances its revision. The namespace writer remains per Workspace, not global.

Native create and reserved-create call this shared body; old `new_spool_node`/reserved construction bodies are removed after caller transfer. Native other inode creation/materialization uses the checked shared allocator. Native namespace queries still use the retained synchronous host acquisition adapter; full out-of-lock runtime acquisition and daemon activation are pending. Cold directory input here is explicit, not a blocking ObjectRead hidden in core.

Checks: all 51 native Workspace and 12 preexisting core tests PASS (build 7.20 s; execution 1.10 s / 0.06 s). Additional namespace regression PASS checks stale prepared-name rejection without allocating a new identity, duplicate-create/reserved-ID preservation, exact reserved creation, stale cold acquisition, and NodeId exhaustion without live mutation. Affected-package Rust 1.96.0 Clippy with warnings denied PASS (2.64 s). The crate still has only layerfs-content as a dependency. New helper types are specific name-acquisition facts, not a generic operation/effect framework.

Next slice: shared mkdir and immutable inode installation, then actual execution-owner submission/backing/frozen construction bridge. The supported rewrite and performance acceptance remain incomplete; no measured create-100 candidate exists yet.

## P3b — shared directory creation and immutable inode installation

Native mkdir/reserved-mkdir and symlink now use the same resolved-name/new-node installation body as shared file creation. Core owns canonical-inode identity mapping and directory-parent mapping; native checkpoint/reclaim/construction accesses those same maps. Removed the native new-directory constructor and duplicate mkdir/symlink insertion bodies. Immutable inode installation validates canonical identity, immutable data class and portable metadata, and adds an acquired alias to an existing live inode without replacing its edited content. Physical/canonical acquisition remains host-side.

Checks: 51 native Workspace tests and 13 existing core tests PASS (build 6.12 s; execution 1.10 s / 0.06 s). Added core namespace coverage for directory parent identity/sticky mode, symlink kind, acquired aliases sharing the edited live inode and malformed acquired metadata rejection; both namespace tests PASS. All 12 native file-edit integration tests PASS (7.56 s), including alias/rename/unlink/reclamation and exact publication/retry; both reconciliation integration tests PASS (0.03 s). Affected-package Clippy PASS. Native allocation failure maps to the existing NoSpace adapter class rather than Invalid.

This remains foundation for the first execution-owner slice, not a switched runtime or final corrected Commit contract. Full out-of-lock acquisition, bounded submission/backing, frozen construction and command-independent operation cut are pending.

## P3c — host physical backing without a live namespace

`HostSpool` now owns the existing segment registry/current identity/high-water, physical observations and write metrics separately from LiveWorkspace. Native write calls `reserve_append(directory, bytes, logical_bytes, policy)`, shared core preparation, `HostSpool::append(exact reference, offset, bytes)`, then shared core apply. Physical append/check/rollback and retirement bodies moved, without CDC/CAS or namespace operations in this host boundary. Foreign range references, stale offsets and capacity overflow are rejected before physical write. Existing short-append/cleanup-failure accounting remains; a backing wait can now be placed outside live-state access by the incoming runtime adapter.

The legacy native refresh moves the retained HostSpool as one value, retaining next identity and physical charges while preserving its old current-segment/metric reset behavior. This does not qualify reconstructed refresh as the final command-independent Commit implementation; that legacy control path still needs replacement.

Checks under the shared lock: 52 native Workspace tests PASS (4.82 s build, 1.10 s test bodies), including rollback, stale append, physical peaks, old readers, refresh and exact install retry. New HostSpool test runs with no Store or live namespace, checks foreign/stale append rejection, unchanged earlier bytes and last-reader retirement. Test-instrumentation feature check PASS (1.84 s); Clippy with Rust 1.96.0 and warnings denied PASS. The initial extraction compilation found four mechanical receiver field accesses and a now feature-only local variable; corrected before the passing checks.

Network backing service/submission/frozen construction and the new operation-level Commit cut remain pending. No performance candidate or terminal PASS.

## P3d — immutable acquisition facts independent of live installation

Host acquisition now returns `AcquiredInode` containing immutable identity/data/metadata without live paths, pins or revision. Shared `complete_name` validates the exact directory revision, directory root and base namespace identity before installing a NodeId/path. Native lookup/create reuse this path; removed the old duplicate materialize path. The base namespace check rejects an old inode-table reply even when a checkpoint retains the same directory root. Existing acquired aliases retain edited inode state.

Checks: full 52 Workspace and 14 core unit checks passed after extraction; subsequent wrapper consolidation and base identity guard passed the 5 native namespace checks, both portable namespace checks, and affected-package Clippy with warnings denied. `git diff --check` clean. Native acquisition is still synchronous in its adapter; this establishes the input/installation seam, not runtime suspension. No performance claim.

## P3e — canonical construction consumes frozen changes

`FrozenWorkspaceChanges` is a portable read-only view of node records, dirty identities, canonical identity index, base identity, generation and policy. `CandidateInputs` combines this with existing host reader/Store/spool context. StableFileInputs, FrozenFile, producer plans/results, frontier/reference logic and CheckpointJournal now execute through CandidateInputs without receiving a live Workspace. Native candidate construction borrows the same records without cloning the namespace or payload. Preview/reconciliation methods remain on their actual native caller.

The detached-input regression copies only dirty records and their directly referenced children for the test, omits an unchanged cached file, then writes/unlinks newer live state. Existing builders produce the exact earlier root and checkpoint generation from the detached records while the live file retains newer bytes. This establishes construction independence and range retention, not the daemon transfer protocol or runtime Commit admission.

All 18 construction tests PASS (0.39 s test bodies). An attempted checkpoint-journal limit change from input nodes to dirty count broke the existing clean-record validation check; reverted that unnecessary change, retaining the existing input-node bound. One test constructor and six redundant borrows were corrected during extraction. Affected-package Clippy with warnings denied PASS. No performance sample or final rewrite claim.

## P3f — owned FUSE read/write completions

LayerFs read/write now transfer one-shot ReadReply/WriteReply through FilesystemPort submission methods. Replies contain no borrowed request data or LayerFs references; a retained reply can complete after the decoder returns, validates response length, and uses fuser's existing cancellation EIO on drop. Incoming write bytes stay borrowed at submission so the execution owner can reserve before copying. Existing host/proxy callers use the synchronous default pending their transfer; no worker-pool or backing-suspension performance claim is made by these defaults.

Linux socket-backed real fuser decoder check PASS: a parked write allows a later unrelated getattr to reply first; explicit ENOSPC, oversized-result rejection and dropped completion each produce exactly one correctly identified reply. This is a decoder/ownership component check, not a mounted workload or worker-admission proof. Five daemon and three maintained-fuser component checks also pass in the existing optional runtime-check stage. First attempt failed to compile five explicit test Fixture initializers missing the new test-only field; all corrected without changing prior test behavior.

Retained failing receipt: `benchmark-results/issue49/reply-check-1788664357883623000`. Passing receipt: `benchmark-results/issue49/reply-check-1788664416224326000`, source seal `c9f08489e3ce2d4c36962f59a558d9910f43abe549453dac9431177a3bb96d35`, product seal `722ae02745cce91308ebf5d870afa856af6176a744a8ae32f112fe00f9f87407`, image `layerfs-runtime-check:c9f08489e3ce2d4c`. Build wall 22.2369 s, exit 0, no timeout/truncation. Raw command, output and hashes retained.

## P3g — exact prepared truncate and range checks

Write and truncate share `PreparedFileEdit` / `apply_edit`; `prepare_truncate` retains the exact before inode revision/data and ranges without converting the live base inode to edited state. Native truncate checks the prepared ranges through host physical backing, then applies once. No-op truncate avoids building a PieceTree. Preserved edit-budget checks before resetting an explicitly emptied generation, sparse zero growth and the existing rule that truncate retains spool history even after lowering the payload limit. Removed native truncate installation body and its unused generation/edit wrappers; SDK multi-edit installation remains pending transfer.

Checks: all 53 native Workspace and 14 existing core unit checks PASS (5.42 s build, 1.10 s / 0.05 s test bodies). Added targeted prepared-truncate check PASS for stale metadata rejection, pin-only continuation, old-read retention, exact history charge, no-op and edit-budget rejection without mutation. Affected-package Clippy PASS. These historical lifecycle tests still include old Busy assertions; they do not satisfy the corrected command-independent Commit acceptance. No performance claim.

## P3h — shared handle pins and live reclamation

Native pin/unpin and unlink reclamation now use the shared live core's pin counts and existing canonical/path/dirty/edited cleanup algorithm. Host capture completion and physical retirement remain in native adapters; shared unpin reports when released edited ranges permit retirement. Pin overflow is checked before a dependent native truncate. Pin-only changes do not advance content revision.

Focused native interleaved-write/rollback/open-unlinked lifetime check PASS (0.07 s); all 16 core checks PASS, including pin overflow/underflow, two-handle last-release reclamation and prepared write/truncate across real shared pin calls. Clippy PASS. No runtime ownership transfer or performance claim at this checkpoint.

### Next runtime slice (in progress, not acceptance)

Using Tokio 1.48.0 with only rt-multi-thread/sync/net/io-util/time, keeping it out of the portable core. Its existing readiness, executor and owned semaphore primitives cover the required suspension seam; no custom actor/effect framework or FUSE multiplexer. Source: https://docs.rs/tokio/1.48.0/tokio/runtime/struct.Builder.html . `max_blocking_threads` alone leaves an unbounded queue, so physical work separately acquires a permit before spawn_blocking.

Initial LiveRuntime component uses two execution workers and at most two admitted physical workers. Scheduler handles do not own Runtime, avoiding last-runtime destruction on its own worker. Request capacity (256) and transfer bytes (32 MiB) are distinct owned permits; receive-loop capacity waits precede borrowed-argument copying and occupy no filesystem worker. These are runtime component bounds, not whole-process budgets, mount fairness, or RSS proof. Runtime is not yet connected to real daemon mounts.

Native 1.85.1 dependency check PASS (10.81 s); lock adds exactly Tokio, bytes, mio, pin-project-lite, socket2 and wasi. Native real-TCP component check PASS (5.32 s build, 0.02 s test bodies): two parked socket operations leave execution capacity for another task, reply wakeups complete, all count/byte reservations return, oversized requests reject and a cancelled capacity wait leaks no request slot. Daemon integration, per-Workspace/inode ordering, source-sealed Linux check and backing protocol are next; no performance candidate yet.

Runtime implementation hypothesis for the corrected mapping contract: first queue conflicting ordinary callbacks and drain already-admitted reads/writes; perform kernel inode/page invalidation while allowing kernel FUSE_WRITE_CACHE writeback callbacks to finish on the same owner; only then close writeback admission and establish the cut. Publish/install under the bounded mutation pause and resume the same handles. SDK edits use the affected-inode version of that sequence with the required post-edit invalidation/error retention. A blanket write pause before invalidation would deadlock laundering. This is an unimplemented hypothesis pending Linux mmap/dirty-page checks and kernel-return-value audit, not accepted mapping correctness. Ordinary cache hits and in-flight read replies must be accounted for before declaring the cache-flush boundary complete.

First backing candidate should reuse host segment allocation and append checking, with one exact prepared write held across an async physical acknowledgment, before adding payload batching. Missing immutable input goes through a separate adapter acquisition step and shared complete_name revalidation. Measure the coherent create candidate before deciding whether write-ack latency warrants another transfer mechanism. Preserve compact references by sharing the returned physical segment identity across its ranges. Host frozen records are immutable construction/recovery facts, never a POSIX replay target or another live Workspace.

## P3i — async execution and operation-cut components

LiveRuntime/Scheduler implement distinct request-count/transfer-byte admission and separately admitted physical jobs with Tokio. OperationGate/CacheFlush/OperationCut encode the two-phase ordinary-callback drain then writeback drain; neither commands, persistent handles nor owner metadata observations take those guards. LayerFs passes the actual FUSE_WRITE_CACHE bit through owned write submission. These components are not yet activated on daemon mounts; the kernel cache-flush mechanism and full owner routing are still pending. The gate test proves its ordering/resumption, not kernel mmap correctness.

Native tests and warnings-denied Clippy PASS. Sealed Linux runtime-check PASS: both new runtime tests, owned fuser reply/writeback-class check, 5 daemon tests and 3 maintained-fuser checks. Receipt `benchmark-results/issue49/async-check-1788667367281489000`, source seal `16fb6f1054fb0f7b75656b25896e19eb8075f145b41b2017f049412cf7082502`, product seal `4b7afac2588b0cb1ec56fcdd51ce0ea8d9b4f06d4614ab022ef72298dad75683`, image `layerfs-runtime-check:16fb6f1054fb0f7b`. External build 39.4790 s including new dependency compilation; exit 0, no timeout/truncation. No create/delete sample yet.

## Restart integration development slice (2026-09-06)

Checkpoint `35f4a68fcc887f88ef81b773adfa3a8ccd1c9baf` verified in the isolated restart checkout history. Predecessor checkout and evidence left untouched. Host WIP dependency check passed; its unused backing/guard warnings confirmed missing callers, not a diagnosed hang.

Connected the existing mount helper startup to one `LiveOwner`, host immutable/HostSpool backing, frozen `CandidateInputs`, and streamed checkpoint installation using shared native/core installation methods. Owner observation bypasses payload freeze. Read/write callbacks retain admitted replies; writes resume their exact prepared edit after physical acknowledgment. Physical reservation now has one exact outstanding length/offset and an explicit cancellation; the failing reservation-alias regression passes. Frozen host facts retain explicit shared byte reservations until replacement/installation. Physical spool metrics now read the actual backing owner.

Focused checks: three existing checkpoint/reference/partial-install regressions pass after installation extraction. New real-TCP create/write/freeze/existing-builder/install/continue test passes across two Commits with one open handle and stable NodeId. First Linux image build compiled successfully; subsequent cleanup/accounting changes require matching rebuild before sampling.

This is a development slice, not terminal acceptance: daemon still launches the existing helper; namespace enumeration/link/unlink/rename and remote SDK edit/reconciliation remain incomplete. Kernel mapping/cache-flush boundary, general callback suspension/control progress, full aggregate state/descriptor accounting, exact ambiguous-install recovery and obsolete-path retirement remain required. Running-command rejection is removed only from the connected remote Commit path; native semantics remain pending correction. No create/delete performance or final independent proof is claimed yet.

First connected real-FUSE sample: `benchmark-results/host-store/results/run-b96b419bf224/perf.jsonl`, source `818d350ef7b803e8504189e82191ea40c05b43dcf2b69c3e5e1b79d02fcf4ab6`, product `6bdc4351987fc6b54466e7e42db132066b915553c281159fb64ce87b32c9f3b0`, image `sha256:efe93fa0de004ca698e722e1d8f20e73380b87a26035bb383ebee96377e3c11d`. Create 0.0106 s, Exec 1.2028 s, Commit 0.8335 s, visibility 0.0001 s, End 0.0050 s; exact lifecycle 2,052,001,292 ns: strict #47 TARGET_MISS. The generic 15-second family PASS is not acceptance. Exactly 1,000 file writes /104,857,600 bytes; normalization and root sync unchanged. Protected preparation acquired normally in this new checkout, cloned by closed-quiescent byte copy, master unchanged; cleanup PASS. Container 2 CPUs/2 GiB/no swap/256 PIDs, no mounts; observed peak 14,712,832 bytes and command-window CPU505,468,000ns. Host peak RSS78,954,496bytes, uncapped host CPU separately recorded. Physical spool105,017,344bytes before Commit and zero afterward. First receipt's open-spool/retained-segment fields incorrectly read dormant native fields; the actual allocation fields were routed correctly. New caller correction reads remote HostSpool for those descriptor/retention observations. Transport copy/timing fields remain uninstrumented and cannot support a zero-work claim.

Next hypothesis: individual frozen/checkpoint node round trips caused substantial added Commit cost (pause234,145,250ns; combined installation/resume319,704,292ns, including nested retirement). Frame the same complete groups in pages bounded by128nodes and64KiB target, retaining the existing hard frame limit and final complete-group acknowledgment. New real-TCP continuation check crosses the128-node page boundary and passes both Commits. Also classify actual remote installation under Checkpoint rather than Resume; this corrects nested attribution without changing the lifecycle timer. No sample repeat before matching builds; supported-surface and terminal obligations remain open.

Paged-facts sample: `benchmark-results/host-store/results/run-7b6f73d79764/perf.jsonl`, source `1c67db448b4a851bfa020f9cc9833d9afe899366ec96f2d23dd154f685c82a45`, product `0e37c73a216eb0b3f0df7ca82fb5f79edb902083c8b6829cc6b57e25478298c2`, image `sha256:fda1075691b5c400ae1b54040ced6d2df0aecd3d57769b8de8787de02146a7c9`. Exact lifecycle1,208,739,668ns (1.2087s), Exec0.8117s, Commit0.3799s, End0.0077s: strict TARGET_MISS. Cut5,182,042ns; checkpoint99,252,625ns includes nested87,559,416ns spool retirement; resume714,750ns. Commit batching is supported by those changed-boundary timers. The unchanged Exec path also varied materially; do not attribute its reduction to this change or combine it with another run. Preparation cache hit, master unchanged, cleanup PASS;104 physical segments/104,857,600 retained bytes before Commit, zero after Commit, allocation105,017,344bytes, zero observation errors. No final proof or delete sample yet.

Next focused change reuses a bounded≤1MiB physical reservation interval for consecutive appends, preserving host acknowledgment before each write reply and exact per-inode prepared installation. Each append must start at the reservation's current offset and fit its remaining bytes; success advances it once, failure consumes it, cancellation abandons only the unused suffix. Quota pressure may fall back once to the exact request size after a rejected allocation, without replaying a mutation. Root sync/freeze cancel the unused suffix. Focused physical-reservation regression now checks partial-prefix replay rejection/cancellation/earlier-byte retention; it and the two-Commit real-TCP page-boundary continuation check PASS.

Reservation-window sample: `benchmark-results/host-store/results/run-a2e1a17230aa/perf.jsonl`, source `efcc2fb86601678b6161724a40e3494d43184983b6dd3930503afd35e17bf781`, image `sha256:378721cb901e023bdf32952f0399a85478089919da8043f1daf1bfcbf091c2d9`. Lifecycle1,333,440,960ns (1.3334s), Exec0.9150s, Commit0.3994s, End0.0085s: TARGET_MISS and no demonstrated elapsed gain from reservation reuse. Preparation cache hit and cleanup PASS. Do not select the previous favorable Exec timing or claim code-derived message reduction proves performance.

Next diagnostic change populates actual existing frame/copy/physical-dispatch fields and adds five bounded atomic receipt counters: live backing call count, client exchange wait (including connection ordering), physical admission/dispatch queue wait, owned-write enqueue-to-first-poll delay, and prepare/apply work. Client exchange time contains host queue/work and must not be added to them. These counters do not alter the workload/synchronization recipe; their purpose is to locate dispatch/wait/work cost before further transport changes. Real-TCP regression passes serialized metric roundtrip, matching sender/receiver frame-byte totals, nonzero queue/work observations, and checkpoint continuation.

Attributed sample: `benchmark-results/host-store/results/run-d1759f498608/perf.jsonl`, source `bbe0fb1fec04b642116d3cfe54a0f62c3e37c20797218fbe12756d1efab79dc2`, image `sha256:eda99fed94de49ef78d26564e6e68abd984dfef66e68be62ba6686118b638b1b`. Lifecycle1,247,506,625ns (1.2475s), Exec0.8857s, Commit0.3288s, End0.0247s: TARGET_MISS; preparation cache hit and cleanup PASS. Actual1269 backing calls, cumulative exchange447,018,380ns, physical queue7,303,431ns, host dispatch62,296,759ns, write dispatch30,144,984ns, prepare/apply2,665,429ns. Client socket-write42,795,905ns and host body-read71,417,323ns are nested stages, not independent lifecycle additions. Actual request and encoding copies each104,857,600bytes; sender/receiver frame-byte totals match105,091,133bytes.

The next concrete transport correction uses vectored writes for frame prefix/status plus the existing payload. Previous framing used separate TCP_NODELAY writes for these fields, adding packet/readiness boundaries. No payload copy, buffering acknowledgment, kernel flags or recipe change is introduced. Short-write regression with a7-byte duplex channel passes request, empty success and error framing; the real-TCP checkpoint/continuation/receipt check also passes. Performance still requires the matching build and single next sample.

Vectored-frame sample: `benchmark-results/host-store/results/run-344d3040ad1d/perf.jsonl`, source `1d1a86b37921b2e0707c3af75a7288291d8e609cb0a1bcc19fb267e135c75667`, image `sha256:a89fcd8d45eb9a8855ee9f1037b9efe487a82bdbb7b51c32df52a603f569429f`. Lifecycle1,262,983,208ns (1.2630s), Exec0.8561s, Commit0.3901s, End0.0064s: TARGET_MISS.1269 backing calls still consume426,829,876ns; physical queue6,971,416ns, host dispatch49,138,083ns, write dispatch28,532,408ns, prepare/apply2,282,796ns. No useful lifecycle gain established by framing changes. Ordinary buffered-backpressure semantics, rather than more framing machinery, are the next measured dependency.

Buffered backing change: reserve shared transfer capacity before payload allocation, reuse the already accepted≤1MiB physical interval, and retain exact applied bytes with the shared BackingRef. New writes use the same PreparedFileEdit/PieceTree validation and installation; pending reads read those owned bytes, including the boundary between an acknowledged prefix and pending tail. At capacity or sync, send the retained frame without another payload copy. A failed/ambiguous append retains its bytes/charge, rejects subsequent writes and is never replayed. Existing prior proxy behavior already retained bounded closed-create/write buffers before dependency fences; this changes the connected slice's per-write physical-ACK policy to validated/applied/retained local acknowledgment and does not claim write/close power-loss durability.

Fsync now drains pending bytes and acknowledges one coherent changed-record generation at the host. Commit reuses that acknowledged generation after closing operation admission, otherwise transfers the new exact cut. Snapshot serialization has bounded pre-reserved memory and performs no network wait under live-state locks. Host synchronization uses existing segment check/observation and records its fence counters. Tests pass for local reads before transfer, host fact generation after sync, page-boundary frozen construction, two Commit continuations with a live handle, retention/readability after injected host append failure, and rejection of later writes without replay. One visibility adapter correction was required to call the existing host segment observation method. No new performance result yet; all final supported-surface/resource/coherence obligations remain open.

Buffered sample: `benchmark-results/host-store/results/run-992d08bcfffe/perf.jsonl`, source `30bd1a43e31554e2508c18635559a7bfacd534c4c93482d93c24f84f5a1224dd`, image `sha256:dc76e257064062ddc71f953eec5e97d8295844f997e7445d4eb1a297aeb46740`. Exact lifecycle991,228,085ns (0.9912s): strict create100 timing PASS for this development source, with narrow margin. Exec0.6105s, Commit0.3674s, End0.0043s;274 backing calls,186,325,926ns exchange wait,1,958,537ns physical queue,39,749,504ns host dispatch,28,266,209ns write dispatch,1,797,305ns prepare/apply. Exactly104,857,600 bytes written, one recorded workspace fence; clone preparation cache hit, master unchanged and cleanup PASS. This is neither terminal PASS nor stable final qualification. No favorable repeat, delete control or independent proof is run at this partial-surface source. Next required work is real callback queuing/mapping coherence and supported caller completion; subsequent substantive source changes require their own matched evidence.

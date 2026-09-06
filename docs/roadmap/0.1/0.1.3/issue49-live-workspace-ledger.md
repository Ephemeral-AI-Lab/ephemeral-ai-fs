# Issue 49 implementation ledger

Status: P0 complete; P1 in progress. No rewritten-product performance claim.

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
| C1 identity/live state | Workspace `cow_tree`, host `FuseView`, FUSE `port` and adapter | Extract shared identity, inode values and attribute body first; namespace/lifecycle transfer pending |
| C2 ranges | `file_io` live reads/writes/truncate/SDK edits; `changes::FrozenFile`, direct construction reader; checkpoint/reclaim | Move existing PieceTree unchanged in algorithm; replace physical segment coupling with owned portable reference; host retains physical adapter |
| C3 adapter/proxy | `LayerFs` callbacks → synchronous `FilesystemPort` → `ProxyClient`/host `FuseView` | Pending. No runtime owner switch until coherent supported slice; proxy paths are not yet dead |
| C4 runtime/fuser | `ActiveMount`, `handle_mount`, helper binary, `HostMount` | Pending request-max audit, one-loop patch and mount ownership transfer |
| C5 backing/capture | `SpoolSegment`, append/rollback/retirement, Running/Ready capture | Physical resources remain host-side; shared admission pending |
| C6 construction/checkpoint | `StableFileInputs`, `FrozenFile`, journals, frontier/references, `PreparedCommit` | Delivered producers retained; frozen-owner boundary and exact installation transport pending |
| C7 SDK | `WorkspaceWorker`, lifecycle observer/edit/Commit, projections, preview/reconcile | Current semantics retained; one-owner routing/cut pending |
| C8 commands/packaging | daemon exec/start/watch/pump; Docker launcher/helper image | Pending tracked start reservation and consolidated supervision; helper remains a real caller |

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

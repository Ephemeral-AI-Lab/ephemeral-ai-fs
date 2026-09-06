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

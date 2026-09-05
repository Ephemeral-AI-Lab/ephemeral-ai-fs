# Issue 49: shared producer execution with different inputs and configuration

Implementation handoff, 2026-09-06. Parent [#47](https://github.com/Ephemeral-AI-Lab/layerfs/issues/47); task [#49](https://github.com/Ephemeral-AI-Lab/layerfs/issues/49). Consolidates three read-only subagent reviews of integration checkpoint `9600e7f90` in `/Users/yifanxu/.codex/worktrees/7311/layerfs`.

## Outcome and scope

One implementation owns task claiming, cancellation, completed-file production/output, ordered results and worker completion. Native initialization and ordinary Commit provide their own prepared inputs and resource arguments. Simplify actual duplicated code, not just wrap two unchanged whole-worker functions.

Preserve initial production settings: native initialization uses its existing available-CPU/task-limited ceiling of eight producers; Commit uses one. Exercise multiple Commit workers in focused correctness tests, not a performance-default change. No new millisecond target, all-tier campaign or broad producer framework. Benchmark migration and all four-tier absolute observations are already recorded; do not repeat them merely because planning starts.

Existing product behavior remains: canonical format, incremental extent reuse, validated owned output, collision checks, bounded memory/storage, publication recovery and live Workspace continuation. #48 owns Exec/FUSE; coordinate shared files with its owner. #49 completion does not close #47 or claim integration of unpublished Exec work.

## Dependency and architecture boundaries

```text
Native discovery / acquired metadata     Stable Workspace inodes / pieces
Native worker allowance                 Commit worker allowance
                 \                       /
                  v                     v
                  Shared producer execution
                  claim -> cancel check -> step
                           -> final output
                           -> completion facts
                              |
                              v
                  Existing bounded output queue
                  Existing checked consumer
                              |
                              v
                  Ordered result reduction
                              |
                 Caller-owned final namespace
                 and publication/checkpoint
```

Keep dependency direction: `layerfs-content` owns canonical builders; `layerfs-layerstack-store` depends on it and owns output/admission; `layerfs-workspace` depends on both and owns Workspace input interpretation. Do not import `Workspace`, `FrozenFile`, `NodeId` or `CapturedFile` into Store/content.

## A. Extend the existing driver, not a second scheduler

Primary file: `crates/layerfs-layerstack-store/src/objects.rs`, `run_finalized_output`.

Today the driver owns threads/channel/drain/join but receives an entire worker body. Move the common claim/cancel/execute/writer-finish sequence inside it. A concrete implementation outline is:

```text
run_finalized_output(
    worker_limit, bounded prepared task source, cancellation,
    initialize_worker,
    execute_task(worker_state, task_ordinal, writer),
    finish_worker,
    consume_owned_batch
)
    -> bounded worker results + writer metrics + pipeline metrics
```

Use existing closure/type patterns. This is a contract, not a required new trait hierarchy or public configuration API. Resource configuration initially needs only the worker limit alongside existing aggregate allowances. Inputs express content semantics; do not add mode=init/commit, fast-path, size/density or benchmark switches.

The driver owns task acquisition, cancellation checks, worker-local initialization, execution, result finishing, output-writer finishing, error propagation and joining. Define finishing order explicitly; if result finalization or writer flush fails, every owner still drains/joins and no successful publication is reported. Preserve native intentional fallback cancellation without treating an incomplete task set as successful coverage.

Actual workers are limited by available tasks and the declared memory budget. Handle zero eligible file tasks explicitly: return valid empty ordered results and zero counters; do not divide by zero or fail because a result vector has no last element. Namespace-only/no-op Commit still uses its normal finalizer.

## B. Once-only input partitioning and deterministic output

Primary file: `crates/layerfs-workspace/src/changes.rs`, `StableFileInputs`, `FileResults`.

Enumerate eligible file inodes once in the finalizer's expected order. Assign stable task ordinals, one per inode rather than per alias. Group descriptors into bounded contiguous blocks. Reuse native task-ordinal/journal-offset patterns; native task units can remain directories or file ranges where its discovery semantics require that granularity.

Choose an existing bounded indexed representation or a shared advancing block iterator. Never use `dirty.iter().nth/skip` from the start for each claim. Never invoke the current whole-dirty-set `produce` loop independently from each worker. Do not retain all frozen file contents or completed objects in a giant vector/map.

Workers release the short task-acquisition lock before content construction or blocking output. Each task retains the source description, exact length/generation and segment/base ownership it needs. Preserve single captured-result consumption.

Use worker-local result journals with bounded block descriptors: task ordinal, journal location and record count. Restore ordinal order after joining before `FileResults::next` validates NodeId/generation/length and feeds the existing finalizer. Do not let completion order change the namespace. Bound result descriptors too; use existing spill mechanisms when needed. Preserve reader error propagation and reject missing, duplicate or unexpected results.

In Store `construct_workspace_files`, replace the hardcoded one-worker invocation and `output.pop()` assumption with a caller-supplied resource limit and complete bounded result/counter reduction. Keep Workspace default one at its caller, not a mode rule inside the common driver. Preserve `WorkspaceAdmission`, exact seen/inserted/reused accounting, database operation scope and final admission flush.

## C. Share completed-file construction, retain source semantics

Files: Store `layerstack.rs` (`NativeImport::regular_file`, regular-file branch of `NativeImport::directory`); Store `objects.rs` (`ObjectBuffer::build_complete_file`); Workspace `changes.rs` (`FrozenFile::build`). A small supporting extraction in content `file/rope/build.rs` and its existing exports is permitted if needed to expose final root/length/counters once.

The full-file rope builder already knows logical length and final root at FileState construction. Expose the necessary completed-root facts from that point rather than rereading the root just to establish the length. Preserve the existing `rope::build` result wrapper for other callers; do not spread an unnecessary signature migration across the repository.

Route native full files and Workspace fresh/full rebuilds through the same checked completed-file operation. Native retains direct finalized output. Workspace retains its private bounded output until file read/construction and length validation succeed. Do not force native initialization through the private Workspace buffering wrapper.

Consolidate the duplicated native regular-file body into one helper accepting metadata already acquired by traversal and the counted reader. Avoid adding a second stat/open by naively delegating through the old public-shaped wrapper. Keep native hardlink discovery, identity/record-slot allocation, metadata cache and counters with native preparation.

Workspace keeps these semantic inputs: unchanged root reuse; PieceTree-to-FileMutationBatch plan/equality checks for edits; full source for new/full rebuild; reusable captured output. Share construction/completion/output behavior where it is actually common. Do not rebuild unchanged data, discard capture, or move Workspace-specific mutation interpretation into Store/content merely to remove a method name.

Full-file finality remains the builder's established contract. Incremental output can contain superseded objects and must retain readable provisional state and selection. Validated immutable objects retain identity/framing proof; storage reads still authenticate. Preview/reconciliation construction remains private where currently required and cannot accidentally start early durable admission.

## D. Native caller migration and code retirement

`layerstack.rs` prepares existing native tasks and worker-local metadata/pair state, then delegates execution to the shared driver. Keep empty-Store guards, discovery/identity semantics, hardlink fallback, initial publication and failure cleanup outside the common producer. Workspace's publication/staging/checkpoint owner stays unchanged.

Delete only after transfer and caller search:

- Native local task cursor/fetch/cancellation loop and caller-owned writer finish.
- Workspace whole-dirty-set execution/cancellation loop.
- Single-worker-only result extraction and incomplete counter aggregation.
- Duplicate native regular-file construction body.
- Redundant full-file completion readback where builder facts replace it.
- Superseded whole-worker callback wrappers after both actual callers migrate.

Keep `run_finalized_output`, the existing finalized writer/checked consumer, source readers, canonical builders and necessary semantic adapters. No legacy alternative producer engine or compatibility path should remain merely for later agents to reuse.

## Resource and failure invariants

Divide aggregate journal/scratch budgets across active workers instead of multiplying the previous allowance. Account worker partial slabs, content scratch, queue, consumer batch, descriptors, result storage and retained sources simultaneously. Fixed output queue/slab bounds remain unchanged. Use checked arithmetic and retain supported low-budget behavior.

Producer failure, consumer failure, panic and result/writer finishing failure must cancel safely, drain/disconnect without deadlock and join every worker before source ownership is released. Preserve initialization's existing failure cleanup and Workspace's existing retained selected-object/candidate behavior; never copy whole-Store deletion into Commit. Publication requires successful completed task coverage and final structure.

## Implementation sequence

1. Inspect current source/callers and record the ownership/deletion ledger. Consolidate the native regular-file duplication and expose completed-root facts only as needed.
2. Extend the existing shared driver and add a small deterministic task/failure test. Migrate native caller while preserving its default worker policy and semantic fallback.
3. Migrate Workspace task preparation, per-task construction and bounded ordered results. Preserve one production worker. Remove the old single-result adapter after all callers transfer.
4. Run focused one/multiple-worker regressions and existing affected suites. Remove dead wrappers only after caller verification.
5. Record clean local implementation commits, exact test scope and adoption/deletion ledger in #49/results. Do not claim a throughput gain from code reuse alone. Replan concrete implementation obstacles without expanding into a framework or restarting benchmark optimization.

The order may be adjusted to keep intermediate code compiling, but both callers must use real shared execution and completed-file construction by completion.

## Focused verification and stopping rule

Reuse existing tests; add only missing seams:

| Check | Required result |
|---|---|
| One vs several workers; reversed completion | Identical roots/object and result counts, exact ordinals, no missing/duplicate inode builds |
| Aliases and captured content | One build per inode, correct surviving references, captured output consumed exactly once |
| Empty/namespace-only input | Valid empty results, zero payload counters, correct final Commit behavior |
| Full/direct vs private construction | Equal roots and selected IDs/bytes for empty, repetitive, tree/spill-boundary data |
| Mid-read failure after at least one chunk; length mismatch | No successful completed-file result; Workspace does not admit incomplete-file output; native retains its own cleanup contract |
| Incremental/no-op/sparse/captured/preview | Existing root reuse, private preview, finality and generation checks preserved |
| Producer/consumer/finish failure | All workers join, blocked senders release, source owners/accounting remain correct |
| Tight budgets | Bounded scratch/journals/descriptors, established resource errors without leakage |
| Counter reduction | All worker counts accumulated; peak metrics use their actual defined semantics, not blind sums |

Run appropriate native initialization, Workspace, file-edit/reconciliation and staging/recovery checks under the shared host lock, once per relevant change. Do not rerun passing suites without a changed concern. No independent proof after each refactor slice; existing final-stage bounded proof policy remains. No worker sweep or repeated tier1/10/100/500 campaign. If behavior/performance concerns require a sample, select the smallest relevant source-bound case and disclose its scope.

Stop when actual duplicate producer/construction control flow is removed, both callers are adopted, resource and semantic checks pass, and the ledger is complete. Further concurrency tuning requires a separate measured decision; #47 full-lifecycle/Exec integration obligations remain open.

## Implementation and adoption ledger

Implemented from clean coverage checkpoint `9600e7f90` in the integration worktree. Plan adoption: `cc6d3b0ef`; checked completed-file facts/native regular-file consolidation: `2570c0d81`; shared task execution and ordered Workspace results: `da40c431c`. No unpublished #48 changes, benchmark definitions or worker-performance defaults were adopted or changed.

| Responsibility | Actual adoption | Retired code |
|---|---|---|
| Task claiming, cancellation, step execution, result finish, writer finish, drain/join | `objects.rs::run_finalized_output`; native direct, buffered fallback and serial-root fallback; Workspace Commit and private Preview | Native local atomic claim loops (including the old test comparator), whole-worker cancellation/finish wrappers; Workspace whole-dirty-set execution loop |
| Once-only Workspace inputs | `StableFileInputs::prepare` emits one generation-bound task slot per eligible NodeId, in dirty-inode order; `FileTasks` advances a buffered journal under the driver's short claim lock | Repeated worker traversal; alias-per-path work; single-worker-only execution assumption |
| Ordered completed results | Each worker seals its own bounded journal. A32-byte fixed ordinal slot holds NodeId/worker+1/journal offset/encoded length. Finalizer reads slots in order and validates worker coverage, NodeId, generation, length, per-worker cursor and final EOF | Single `FileResults` writer/reader assumption; Store `output.pop()` extraction; no in-memory all-inode completion map |
| Completed full-file facts | Content `rope::build_complete` emits root/logical length/counters; existing `rope::build` wrapper remains. Store `build_checked_file` serves native direct output and Workspace private full-file construction | Duplicate native regular-file body; completed-root reread solely for length validation |
| Metadata and source interpretation | Native `regular_file_with_metadata` uses metadata already acquired by traversal; Workspace retains PieceTree/incremental/captured/no-op interpretation | Extra native body duplication, without adding stat/open calls or moving NodeId/FrozenFile into Store/content |
| Output and counters | Existing authenticated owned writer and checked consumer remain; Store reduces every worker result. Work counters sum, per-worker slab peaks use max, structural peak sums retain their conservative-bound meaning | Per-caller writer completion; incomplete single-result/counter reduction |
| Lifecycle | Native discovery, hardlink fallback and empty-Store cleanup remain native; Workspace namespace/reference finalizer, admission token, stage/exact-head/publication/install recovery remain Workspace-owned | No native whole-Store cleanup imported into Commit; no provisional-object admission shortcut |

The fixed on-disk ordinal index is the bounded Workspace adaptation of native task-ordinal/worker/journal-offset blocks. Native task units have a bounded descriptor vector; Workspace inode counts can be larger, so its descriptors are stored on disk rather than copied into an unbounded vector. Each file adds one32-byte metadata slot and one positional slot update. Result data remains in worker-local buffered journals and restores forward order within each worker. This is a metadata-ordering tradeoff, not a measured performance improvement or a new file-tree algorithm.

### Resource and default behavior

Native production keeps available-CPU/task-limited workers capped at8. Workspace's actual production caller still passes1; multiple-worker entry is private and exercised by tests. Zero file tasks create no workers and return valid empty results. Buffered native fallback descriptor count is capped by the existing task-block allowance; larger unsupported partitioning retains its serial-root semantic fallback, now executed through the same driver.

The task-index buffer and all worker-result buffers split one existing journal allowance, including the1024-byte low-policy path; aggregate scratch reservation is not multiplied by worker count. Candidate memory, spill buffers, reference/ordering/index allowances divide across workers, with the existing fixed SQLite spill-index cache included in the per-worker bound. Source/captured owners stay retained through join. Queue/slab limits are unchanged; partial slabs and irreducible per-worker CDC scratch remain bounded by the active worker cap. Workspace multiworker `spill_peak_bytes` is explicitly a conservative sum of worker-local spill peaks; default-one semantics are unchanged. This is not labeled an exact concurrent physical peak.

Result journals seal before final writer flush. Step, consumer, result-finish, writer-finish and panic failures cancel, drain and join all workers. Native intentional fallback cancellation is checked by the native owner before result publication. The Workspace adapter requires full task coverage. Private Preview uses the same task driver with a private ObjectBuffer sink; it cannot invoke early durable admission.

### Verification ledger

All builds/checks/tests used the shared host lock, serially. No performance sample, worker sweep, all-tier rerun or independent proof campaign was added.

- Store library suite:51passed,1existing large-spill test ignored. Covers native canonical equivalence, direct/buffered paths, hardlink fallback, bounded task/record state, checked ownership/dedup/collision handling, spill/index behavior and native admission/publication failure cleanup.
- New driver test passes serial/four-worker exact task coverage, deliberately reversed completion, zero tasks and wrong declared coverage; producer/consumer/result-finish failures and panics plus writer-finish failure all return after every worker owner is dropped.
- New partitioned construction test passes actual spilled private output versus direct writer root/object-ID/counter equality, divided allowances, wrong length and read failure after2MiB. The failed private build leaves Workspace Store counts unchanged.
- Final Workspace library run:58passed. Added serial/four-worker root/object coverage, alias-once/counter checks, captured reuse (with temporarily corrupted backing to prove it is not silently rebuilt), empty/namespace-only tasks, incremental/sparse reuse, tight-budget behavior, reverse journal completion and missing-result rejection. Existing stage/exact-head/checkpoint-install recovery and open-unlinked/rollback tests pass.
- File-edit integration12passed and reconciliation integration2passed, covering publication retries, aliases, sparse/overlap/zero operations, discarded candidates, old-root continuation and projection recovery.
- Content streaming-wrapper equivalence check passed. After moving the final serial native fallback through the shared driver, both native directory-import canonical/finality checks passed. Final `cargo check -p layerfs-sdk -j2` passes with no warnings.

One intermediate Store type-check failed while its adapter still had the old whole-worker signature (E0277/E0061). Migrating that actual caller to the task interface resolved it; subsequent checks passed. Injected panic messages in the failure test are expected and caught, not unhandled test failures. Passing checks were repeated only after corresponding journal-budget, capture-proof or fallback changes.

The #49 implementation scope is complete in local commits. Existing benchmark observations remain attached to their pre-refactor source/product identities; this refactor does not establish a throughput gain, current-source performance qualification or parent #47/#46/#39 completion. #48 integration and final performance/proof obligations remain separate. No issue is closed by this ledger.

### Subsequent user-requested measurements

The user's follow-up asking for current numbers authorized one revised tier100 create/delete pair after implementation. Source `6de381837` records create Commit358.675583ms/full1,291.390876ms and delete Commit14.803083ms/full298.008125ms. Both observed Commits are below400ms; strict #47 create lifecycle remains a miss. Full custody and comparison limits are recorded in results attempts21–22. This does not alter the completed refactor's worker defaults or qualify a speedup; no independent proofs or other-tier reruns were added.

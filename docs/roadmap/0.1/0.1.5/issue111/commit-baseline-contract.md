# Commit baseline and phase attribution: current main — measurement contract

Freeze before instrumentation and collection. Repository
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs`, issue #111 (Commit investigation;
the original cold-Init <=2.7 s objective stays open), context #115/#108.
Storage treatment is unchanged **promoted-uncompacted** (current main plus the
preserved uncommitted compaction-removal work, byte-identical).

This is measurement and root-cause attribution only. No optimization, storage
policy change, guarded-predecessor promotion, or Init campaign restart. Stop
after the plain baseline, the phase breakdown, and one evidence-led next-step
recommendation.

## 1. Question

With current main (after 441be212e compact fingerprint index), measure and
attribute the public Commit cost of `Client::commit_workspace_session`:

1. no changes (expected `UpToDate`), one small change, repeated
   change/reversion, and larger changed sets (K=10, K=100 files);
2. retained Store versus Store dropped/reconnected before workspace creation;
3. phase partition (queueing/fence, discovery/reconciliation, candidate/tree
   construction, admission/encoding incl. metadata pool, publication), with
   nested details and residual;
4. which work scales with total namespace/history versus the changed portion;
5. SDK-prepared edits versus an ordinary POSIX/FUSE-write route followed by the
   same public Commit;
6. the single measured bottleneck, if any, that warrants the next experiment.

## 2. Traced public operation (fixed)

`Client::commit_workspace_session(workspace_id)` → SDK `observe()`
(receipts/monitor, not in the timed wall) →
`Workspaces::commit_workspace_session_with_status`:

- worker lookup, lifecycle lock, `begin_workspace_commit(CaptureMode::Live)`
  (thread-local `WorkspaceCommitReceipt` timer);
- `projection::pause` → **pause_fence_ns**; `wait_for_writers`/`quiesce` →
  **quiesce_ns**; `projection::capture` → **capture_ns** (no-op for the Docker
  FUSE projection: both SDK and FUSE mutations are live-backed, `capture(0,0)`);
- `workspace.commit()`:
  - pending publication retry path; conflict-resolution path
    (`commit_workspace_reconciliation`);
  - generation 0 (no mutation): empty candidate → `UpToDate`;
  - else `build_candidate`: **candidate_plan_ns**, **content_ns** (streaming
    file admission runs inside content), **namespace_ns** (nested
    deletion_cursor_ns/deletion_records_ns), **candidate_finish_ns**;
- `commit_workspace_candidate`: `admit_remaining` → **object_admission_ns**
  (nested begin/insert/commit/authentication/storage_authentication/sort; the
  metadata pool `prepare_values` incl. `ValueIndex::sync`/`find_batch` runs
  here), then `publish` (INSERT_COMMIT, ADVANCE_BRANCH) → **publication_ns**
  (nested begin/payload/insert/metadata/commit);
- presentation: `install_checkpoint` → **checkpoint_ns** (nested
  spool_retirement_ns/scan) and `resume`/`refresh` → **resume_ns**; timer Drop
  emits the receipt with `unattributed_ns = total − attributed`.

`dirty_compare_ns` and `local_admission_ns` exist in the receipt schema but no
current-main code path notes them; they are reported as zero-with-status
(schema present, no clock in main) and are covered by the residual.

Receipts are reused, not duplicated: per Commit the harness records the public
wall, the `WorkspaceCommitReceipt` (monitor snapshot), the paired
`WorkspaceCommitDiagnostics`, and the per-operation
`PhysicalStorageReceipt` delta (incl. `metadata_index_sync_ns`,
`metadata_pool_*`, candidate/encoding counters). Stage-boundary
`store.physical_storage_receipt()` snapshots are taken before workspace
creation, after creation, after each edit, and after each Commit.

## 3. Fixtures and validation

The four registered namespace fixtures (registry-confirmed; there is no
`namespace-10000-compact-v3`), reused immutable prepared inputs under
`benchmark-results/host-store/fixtures/`:

| Case | Files | Logical bytes | Fixture digest |
|---|---:|---:|---|
| namespace-100-compact-v3 | 100 | 5000000 | c3ff9877…2986ec1 |
| namespace-1000-compact-v3 | 1000 | 20000000 | c88afb4a…c9e87a1 |
| namespace-10000 | 10000 | 300000000 | 5a464369…905a9dc8 |
| namespace-100000 | 100000 | 500000000 | 6fc793a9…1ac80a7e |

Once per campaign and per tier, before any cell: full file/directory inventory
against the prepared manifest, file mode 0640 / directory mode 0750 / mtime
1700000000000000000 ns for every entry including the payload root, and a
complete content SHA-256 pass (no OS-cache eviction; this study's cache state
is uncontrolled, so this validation is identity-only, never a coldness claim).
A wrong-fixture cell is rejected; inputs are never repaired after measurement.

Bootstrap uses full public `Client::initialize_layerstack` with
`LayerStackInitialization::Directory(<fixture payload>)` plus
`fork_branch` from the genesis layer, outside all Commit timers, with the scan
receipt checked against the registered file/byte counts. Fresh SQLite Store
per repetition; no prebuilt output Store, no cloned Store substitute.

## 4. Cell matrix, repetitions, and order

Plain cohort (product byte-identical to current main+preserved dirty work;
only the benchmark harness differs) — n=3 independent fresh Stores per cell:

| Cell (per tier, all four tiers) | Sequence |
|---|---|
| `nochange` | bootstrap → create workspace → Commit#1 (expect `UpToDate`, unchanged head) → end |
| `retained` | bootstrap (owners retained) → create → edit#1 → Commit#1 → edit#2 → Commit#2 → edit#3 (revert) → Commit#3 → Commit#4 (no edit, `UpToDate` check) → end |
| `reopened` | bootstrap → drop every Client/Store owner → reconnect same SQLite Store → create → edit#1 → Commit#1 → edit#2 → Commit#2 → edit#3 (revert) → Commit#3 → Commit#4 (no edit) → end |

At namespace-100000 only (plain, n=3, retained lifetime, first-change shape):

| Cell | Changed set |
|---|---|
| `k10` | K=10 files, edit#1 then Commit#1 |
| `k100` | K=100 files, edit#1 then Commit#1 |
| `fuse-posix` | one file edited by ordinary POSIX write through the FUSE mount (workload `namespace-edit` = registered init_namespace mutation, marker `E000000001`), Commit#1; then workload `edit PATH 1 SIZE` (marker `E000000002`, different offset, no mtime normalization), Commit#2 |

Diagnostic cohort (separate worktree/seal; product = plain product plus
metadata-index diagnostic counters only; harness identical) — n=2 per cell:
`retained` and `reopened` for all four tiers, plus `nochange`, `k10`, `k100`,
`fuse-posix` at namespace-100000.

Traversal order is fixed: tiers ascending (100, 1000, 10000, 100000); within a
tier: nochange 1..3, retained 1..3, reopened 1..3, then k10 1..3, k100 1..3,
fuse-posix 1..3. Diagnostic cohort runs after the plain cohort in the same
order. Several commits on one Store are stages of one repetition, never
several repetitions. Edit routes are labeled separately (`sdk-range-edit`,
`posix-fuse-write`); SDK timings are never presented as covering the FUSE route
or vice versa.

### Edits (frozen)

- K=1 chains edit `d0000/f000000` at `namespace_edit_offset(size)` with
  `delete_len=10`: #1 marker `I000000001`, #2 marker `I000000002`, #3 reverts
  to the original fixture bytes at the same offset. All through public
  `Client::edit_workspace_file_range`.
- K cells select global file ordinal `t_j = floor(j·regular_files/K)`, j=0..K−1
  (path `d{t/100}/f{t}`), advancing t by one while the file is empty (<11
  bytes; recorded picks: at K=100, t=8000→8001 and t=94000→94001). Marker per
  file j: `C{K:03}{j:06}` (exactly 10 bytes), same offset rule per file size.
  The batch API is same-file only, so K files use K single public edit calls;
  the whole edit stage (and each call) is reported separately from Commit.
- `fuse-posix` uses the registered workload helper through
  `Client::exec_workspace_session` (`namespace-edit`, then `edit … 1 …`);
  the exec interval is reported as the mutation interval, never as SDK time.

## 5. Timers, cache, and lifetime labels

- `commit_api_ns`: monotonic Instant immediately before
  `Client::commit_workspace_session` → immediately after its return. Only this
  is the Commit metric. All other intervals (bootstrap-init, bootstrap-fork,
  store-reconnect, workspace-create, each edit call, exec-write, post-commit
  checks, workspace-end, reopen verification) are recorded separately;
  edit+Commit totals are secondary context.
- OS page cache: **uncontrolled** for every cell. No cold claim, no warm-up
  Commits, no residency inference from read volume. Each sample reports
  lifetime (retained/reopened), sequence position, cache profile label
  `commit-study-os-uncontrolled`, process disk read/write bytes per stage, and
  the fixture-validation receipt. No Init latency target is invented or reused.
- Reopened means Store/index lifetime only (fresh derived ValueIndex on first
  metadata preparation; the harness does not otherwise touch the index).

## 6. Correctness, resources, cleanup (gates)

Outside timers, per cell: expected Commit results (`Created` with new head /
`UpToDate` with unchanged head) and public-call counts; visible head via SDK
query after every Commit; complete bytes of every changed file (all K ≤ 100);
10 deterministic unchanged sampled files after the final Commit; final root and
edited-file content re-verified after dropping owners and reconnecting the
Store; `end_workspace_session(Clean)`; zero active workspaces/executions;
container removed; spool/runtime dirs and Store apparent/allocated bytes
recorded. Canonical object/byte counts and physical-receipt counters
(candidate/inserted/reused objects and bytes, pooled admitted values,
`metadata_index_sync_ns`, encoding counters) are captured per stage so missing
persistence or encoding cannot masquerade as a faster Commit.

Resources per stage: user/system CPU, disk reads/writes, RSS snapshot
(post-call, labeled as snapshot, never phase peak), process lifetime peak RSS,
threads, swaps; container cgroup memory/swap/OOM deltas; resource limits
(2 CPUs / 2 GiB / 256 PIDs container, host uncapped). Any swap/OOM/abnormal
exit is a hard failure. Verification is separate from timing; coverage
(changed files exhaustive ≤ K, unchanged files sampled) is stated per row.

Missing required evidence fails the row (fail-closed). Invalid attempts are
retained in full; at most one bounded replacement of an entire affected
cell is allowed only for demonstrated infrastructure invalidity (never for
timing); no outlier removal, no slower/faster-arm retry, no scope growth after
results are seen.

## 7. Diagnostics and reconciliation

Plain and diagnostic cohorts have separate worktrees, seals, binaries, images,
folders, and collector runs; rows are never pooled. Instrumentation overhead is
never subtracted.

Top-level wall partition per Commit (disjoint, sums with residual to the
public wall): pause_fence, quiesce, capture, candidate_plan, content,
namespace, candidate_finish, object_admission, publication, checkpoint,
resume, unattributed. Nested (never added into parents): deletion_cursor/
deletion_records; object_admission begin/insert/commit/authentication/
storage_authentication/sort and memory-owned/spill-readback bytes;
publication begin/payload/insert/metadata/commit; spool_retirement(scan).
Diagnostic-arm-only counters: ValueIndex creations, replayed (re-inserted)
values, evictions, find_batch inputs, fingerprint candidate ordinals, and
authenticated candidate groups. Counter deltas before/after workspace
creation, edit, and Commit establish whether work moved into setup/edit.

## 8. Custody

SQLite, SDK/coordinator, admission/publication and spool on the macOS host;
Docker only daemon/FUSE/workload (2 CPUs/2 GiB). Shared
`runner.py --build-host`/`--build-image` under the runner-owned measurement
lock, serialized builds/tests/collection, no child double-acquisition. No edits
under crates/, tools/ or benchmark/ between a build and its measurements
without resealing. 4 KiB pages, cache settings, worker counts, schema10, exact
CAS, pooled metadata, pack/zstd/FULL/DELTA/CDC and authentication unchanged.
Focused existing store/workspace suites run for the instrumented paths.
Evidence root (outside the repository), recorded in
[commit-baseline-evidence-pointer.md](commit-baseline-evidence-pointer.md),
retains every attempt, command, log, exit, patch, binary, image identity,
fixture validation, raw receipts, and analyzers. Original #104/#109/#110
evidence and prior issue111 evidence roots remain untouched.

Deliverables: `commit-baseline-results.md` (plain tables with n/median/range,
setup/edit context, lifetime/cache labels, resources, physical receipts,
correctness, custody), a measured disjoint phase table with nested details and
residual, namespace-size and changed-set scaling tables, the six owner
answers, and exactly one next-experiment recommendation (or an explicit
no-optimization recommendation). Issue #111 updated with #115/#108 links; no
release/tag/deployment.

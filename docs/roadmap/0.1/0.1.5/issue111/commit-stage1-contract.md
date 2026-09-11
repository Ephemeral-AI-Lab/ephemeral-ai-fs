# Commit Stage 1: inner namespace and edit-stage attribution — measurement contract

Freeze before instrumentation, before any Stage 1 build, and before any Stage 1
cell. Repository `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`, issue #111 (Commit
investigation; the original cold-Init <=2.7 s objective stays **open**), context
#115 (shared admission/publication) and #108 (branch history). Storage treatment
is unchanged **promoted-uncompacted** (current main plus the preserved
uncommitted compaction-removal work, byte-identical).

This stage is **measurement and attribution only**. No optimization, no storage
policy change, no guarded-predecessor promotion, no Init campaign restart. It
stops after measured attribution plus exactly one concrete Stage 2 experiment
recommendation.

## 1. Question

The completed current-main baseline recorded a 241.33 ms `namespace_ns` inside a
313.29 ms K=100 public Commit and a 659.08 ms SDK edit stage preceding it at
N=100000 files, without identifying what consumes either. Stage 1 answers:

1. Which inner operation consumes `namespace_ns`, measured with disjoint inner
   clocks and work counters, at K=1/10/100?
2. Which inner operation consumes the SDK edit stage, measured with per-edit
   counters on the daemon-side live owner and the host-side fact consumer?
3. What is the removable-cost ceiling implied by those measurements, if any?
4. Is one Stage 2 namespace experiment justified, and on which exact code
   boundary, with which falsification conditions?

Non-goals: implementing or promoting any optimization; moving work between
timers; reopened/smaller-tier re-measurement; the full future scaling campaign.

## 2. Hypotheses under test (falsifiable, not assumed)

- **H1 (old, corrected).** `crates/layerfs-workspace/src/changes.rs:805` ends
  content; namespace starts at 806 and covers `apply_references` and
  `inodes.finish`. The preceding dirty-directory loop is charged to content.
  Measured content-only edits introduce no directory-binding mutations. H1's
  original "per-changed-directory tree reconstruction" prediction is therefore
  unsupported and is **rejected**; it is re-tested here only as a counter check
  (`nodes_read`/`nodes_created` versus K).
- **H2.** At `changes.rs:2414` all inode deltas already enter one sorted batch
  (`inode_table_apply_sorted_with_budget`) — re-verified by counters.
- **H3 (measured here).** `crates/layerfs-content/src/tree/batch.rs` reads every
  sibling child before the unchanged-range early return (`:579` before `:519`),
  and compact-leaf decode derives a canonical inode-value hash per decoded
  record (`:946-955`). Untouched sibling records can therefore pay
  authentication, decode and hash-derivation cost. Stage 1 measures the size of
  that cost; it does not assume it is the dominant term.
- **H4 (measured here).** Every SDK edit barrier (`live_owner.rs:2619`
  `EDIT_BEGIN` → `:2404` `freeze` → `:2416` `retire_ranges` → `:2419`
  `publish_facts`) re-serializes the complete accumulated dirty set. For K
  successive edits the cumulative row count is triangular.
- **H5 (measured here).** `live_owner.rs:2344` cached-inode flush and `:2949`
  `retire_ranges` scans grow with the retained sets at each barrier.
- **H6.** `changes.rs:2430-2450` sorted-batch fallback is exercised by a forced
  test rather than labeled quadratic by inspection.

## 3. Frozen cell matrix (bounded)

Namespace fixture: the original pseudorandom **`namespace-100000`** only —
100000 files, 1000 data directories, 500000000 logical bytes, fixture digest
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`. The newer
structured-text variant is not selected and no file class or size changes.

| # | Cell | Sequence (unchanged from the baseline harness) |
|---:|---|---|
| 1 | `nochange` | bootstrap → create workspace → Commit#1 (`UpToDate`) → end |
| 2 | `retained` | bootstrap retained → create → edit#1 → Commit#1 → edit#2 → Commit#2 → edit#3 (revert) → Commit#3 → Commit#4 (no edit, `UpToDate`) |
| 3 | `k10` | 10 SDK range edits (spread ordinals) → Commit#1 |
| 4 | `k100` | 100 SDK range edits (spread ordinals) → Commit#1 |
| 5 | `fuse-posix` | workload `namespace-edit` (marker `E000000001`) → Commit#1 → workload `edit` (marker `E000000002`) → Commit#2 |

Edits are exactly the frozen baseline selections: K=1 chains edit
`d0000/f000000` at `namespace_edit_offset(size)`, `delete_len=10`, markers
`I000000001`/`I000000002`/original-bytes revert; K cells select ordinal
`t_j = floor(j·N/K)` with empty-file advance and markers `C{K:03}{j:06}`; the
FUSE route is labeled separately and never pooled with SDK rows.

Cohorts and repetitions:

| Cohort | Product | Repetitions | Cells |
|---|---|---:|---|
| plain | current main product, byte-identical; only the campaign harness differs | n=3 independent fresh Stores | 1–5 |
| diagnostic | plain product **plus Stage 1 diagnostic clocks/counters only** | n=2 independent fresh Stores | 1–5 |

Frozen traversal order: all plain cells first — `nochange` 1..3, `retained`
1..3, `k10` 1..3, `k100` 1..3, `fuse-posix` 1..3 — then all diagnostic cells in
the same order with repetition 1..2. Several Commits on one Store are stages of
one repetition, never several repetitions. Plain and diagnostic rows are never
pooled; instrumentation overhead is reported, never subtracted.

The reopened lifetime and the 100/1000/10000 tiers are **not** re-measured: no
stated Stage 1 attribution question needs new samples there. Their completed
baseline rows remain the reference and are cited, not re-derived.

Each cell uses one fresh SQLite Store, one fresh container
(2 CPUs / 2 GiB / no swap / 256 PIDs), full public
`Client::initialize_layerstack` + `fork_branch` outside every timer, and the
gated bootstrap identity (112451 canonical objects, 513026835 canonical bytes,
100002 pooled metadata values).

## 4. Timers, cache, and lifetime labels

- `commit_api_ns` remains the only Commit metric: monotonic `Instant`
  immediately before `Client::commit_workspace_session` → immediately after its
  return. Stage 1 does not add any new timing boundary at the public level and
  does not move work between timers.
- The secondary metric is the per-sample sum of all edit preparation plus that
  Commit, computed per sample before aggregation (never a sum of medians).
- Inner namespace clocks are nested, disjoint sub-intervals of the existing
  `namespace_ns`; they never replace or extend it. Inner clocks must not
  double-count nested callbacks and must not be added into a wall-time partition
  together with their parents.
- Daemon-side (Linux container) and host-side (macOS) clocks are reported
  separately. No timestamp from one process is subtracted from a timestamp in
  another; correlation is by call/row counts and by each side's own durations.
- OS page cache stays **uncontrolled** for every row: cache profile label
  `commit-study-os-uncontrolled`. Store reopen means owners dropped and
  reconnected; it never means OS-cold. No untimed warm-up Commit.

## 5. Required diagnostics (minimum)

Product-side, host, inside `namespace_ns` — reuse `TreeBatchCounters` before
adding anything new; the `FrontierInodes::finish` caller currently discards
them:

1. Disjoint inner clocks: reference handling; checkpoint validation and record
   encoding; frontier record encoding; spill merge; batched tree application;
   fallback; final namespace-root encoding; residual. Each interval is entered
   once per Commit and excludes nested children.
2. Counters: batch calls, nodes read / created / reused, changed keys, scratch
   peak; compact records decoded; synthetic inode-value IDs derived;
   authenticated bytes read; checkpoint records validated; encoded record bytes;
   fallback reason and mutations applied by the fallback path.

Edit side, daemon (`live_owner.rs`) — one aggregate line per barrier, no
per-record logging:

3. Snapshot/publication calls, collected dirty ids, serialized fact rows and
   bytes, fact pages, clean/encode/replace/backing-call wall, cached-inode
   flushes and invalidations, scanned versus released backing references.

Edit side, host consumer:

4. Fact frames, rows and bytes received; dirty-set size at each publication;
   host-side consume wall.

Counting unique object IDs for uniqueness is not required; if used, state is
bounded and the bound is reported. Diagnostic scratch must be explicitly
bounded: no unbounded history-sized counter maps.

## 6. Counter-only adversarial tests (no timing claims)

Deterministic, bounded, counter-only unit tests in the existing suites:

- forced sorted-batch fallback (`ObjectLimitExceeded`/`Unsupported`) with the
  mutation count and resulting root checked;
- small-policy spill threshold crossed to observe run-merge record traffic
  growth;
- growing dirty/cache/backing sets across successive barriers;
- overlapping and adjacent edit ranges, and range-versus-folio test counts.

These are reported as work counts, never as N=100000 timings. Any required
extension is frozen here before its timings are inspected.

## 7. Build, custody, and identities

- Two isolated git worktrees at Stage 1 HEAD, each with an exact copy of the
  declared dirty treatment applied as the recorded binary patch, plus the
  byte-identical campaign harness. Source/product/workload seals are computed
  before and after collection and must be unchanged.
- Builds via the shared runner from each isolated root:
  `python3 benchmark/fs-bench-pro/shared/runner.py --build-host` and
  `--build-image`, stdout/stderr, tag, immutable image ID and exit codes kept in
  the evidence root. The runner owns the build lock; no second lock is taken.
  Host jobs at most eight; no profile or job-count change between matched arms.
- Collection runs under the runner-compatible
  `layerfs-infra-measurement.lock` in the parent-process `TMPDIR` (or `/tmp`),
  acquired in the collector's parent process only. No nested acquisition, no
  overlapping builds/tests/proofs/measurements, no interruption of another
  owner's run.
- No `crates/`, `tools/` or `benchmark/` edit between a build and its completed
  collection without resealing and rebuilding.

## 8. Invalidity, replacement, and retention

A row is invalid (fail-closed) if any required evidence is missing, a
correctness/resource gate fails, the container has swap or OOM, the sealed
binary/image identity differs, or an exit code is nonzero. Every attempt is
retained. At most **one** replacement of a whole affected cell (all its
repetitions in both cohorts if the defect is shared) is allowed, and only for
demonstrated infrastructure invalidity. Valid outliers are never removed and a
slower or faster observation alone is never retried. The collector never
executes archived scripts in place; the archived evidence roots stay read-only.

Cell timeout 300 s per harness process, container deadline 120 s, cleanup
deadline 30 s. Any timeout is an infrastructure-invalid attempt.

## 9. Correctness and resource gates (unchanged from the baseline)

Per cell: expected `Created`/`UpToDate` outcomes and head movement; visible head
after every Commit; complete bytes of every changed file; 10 deterministic
unchanged sampled files after the final Commit; final root and edited-content
verification after dropping owners and reconnecting the Store;
`end_workspace_session(Clean)`; zero active workspaces/executions; container
removed; store and runtime byte accounting. Schema 10, 4 KiB pages,
authentication, exact CAS, zstd/pack/FULL/DELTA/CDC, metadata/admission/cache
limits and worker counts unchanged. No compaction/VACUUM/repack/GC.

Resources: user/system CPU, threads, disk/spool growth, swaps/OOM, host RSS and
container cgroup memory. Post-call RSS is a snapshot and is never reported as an
operation peak. Diagnostic scratch is bounded and its limit is declared.

## 10. Deliverables

- `commit-stage1-results.md` under this directory: identities and drift versus
  the recorded baseline; all raw rows, n, median/range, cache/lifetime/route,
  diagnostic overhead, invalid attempts; reconciled inner namespace and
  edit-stage tables; operation counts, resources, correctness and a simple
  actual-path diagram; measured-versus-unmeasured portions; the ranked measured
  removable costs; exactly one Stage 2 experiment recommendation (or an explicit
  no-experiment recommendation); a scaling ledger separating confirmed
  quadratic mechanisms, potential risks, bounded ordinary work and untested
  fallbacks, each with the next proof required.
- Evidence pointer, custody manifest, and an issue #111 update linking #115 and
  #108. The cold-Init objective and #111's open status remain untouched. No
  release, tag, deployment, or optimization promotion.

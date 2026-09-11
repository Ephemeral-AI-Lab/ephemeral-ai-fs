# Handoff: Stage 2 first step — bounded authenticated-read optimization

Execute end to end: inspect, freeze, establish physical-group reuse feasibility,
implement ONE supported candidate, build, measure paired arms, verify and report.
This prompt authorizes this bounded work. Stage 1 is already complete; do not
repeat its full attribution campaign. The owner's latest reference to “stage
one” is interpreted as the first step of the next stage, described here.

If feasibility does not support the proposed optimization, finish with the
measured blocker and a concrete alternative; do not invent a cache, weaken
validation, or stack another optimization to meet a target. No release/tag/deploy.

## Objective and performance targets

Repository: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`.
Issue: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111.
N=100000 total files; K=1/10/100 changed files before one public
`Client::commit_workspace_session`. This is Commit optimization inside the
100,000-file namespace, not Init, not one million files, and not K=100000.

Primary direction: reduce repeated physical-group retrieval/decompression using
existing bounded authenticated batch-reading machinery, while preserving ALL
current canonical/page/parent-child/partition validation. No speculative skipping
of unchanged children. Establish that sufficient repeated work exists first.

| Metric | Stage 1 plain reference | Preferred target |
|---|---:|---:|
| K100 namespace | 254.09 ms | <=120 ms |
| K100 public Commit | 329.76 ms | <=200 ms |
| K10 namespace | 59.87 ms | <=31 ms |
| K10 public Commit | 82.33 ms | <=50 ms |
| K1 public Commit | 20.47 ms | No material regression |
| nochange public Commit | 2.26 ms | No material regression |

Freeze two distinct outcomes before candidate timing: a worthwhile screen
(>=25% paired K100 namespace reduction, clear public-wall improvement, reduced
CPU, all correctness/resource gates) and preferred absolute targets above.
Report a valid partial improvement honestly if it misses 200ms; never relabel
it as the preferred target passing. These are proposed experimental targets,
not forecasts. Use fresh paired controls, not historical timings for deltas.

Planning only:329.76−254.09=75.67ms outside namespace;120ms namespace would imply
~195.67ms total with little margin. Actual gates use per-sample timings, not
sums/subtractions of independently aggregated medians. Do not transfer these
targets to the smaller lazy-identity candidate described below.

The original cold Init <=2.7s objective remains OPEN; last valid recorded median
~3.420s. Init optimization remains paused. No uncontrolled bootstrap Init result
can qualify as cold. Related #115/#108/#109/#110/#106/#102/#104/#100/#107.

## Read first: completed evidence and review corrections

Read all applicable AGENTS.md files (parents/root if present, benchmark/AGENTS.md).
Read `docs/general/benchmark_rules.md` and `benchmark/fs-bench-pro/QUICKSTART.md`.
Under `docs/roadmap/0.1/0.1.5/issue111/`, read:

- `commit-stage1-results.md`, `commit-stage1-contract.md`, `commit-stage1-handoff.md`
- `commit-baseline-results.md`, `commit-baseline-contract.md`
- `commit-optimization-direction.md` (earlier audit; superseded where corrected here)
- `fingerprint-index-results.md`, `metadata-proof-experiment-results.md`
- shared format context in parent `hybrid_mental_model.md`, `spec.md`,
  `compaction-removal.md`

Read completed Stage 1 comment:
https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5637855243
and latest issue review update. Stage1 contract7f264300e, report0650b4945.

Established: diagnostic K100 tree apply244.17ms, read/auth/decode240.33ms;
2053 logical pages read,1920 unchanged-range pages,132 entered pages;
101001 compact records decoded/identities derived;5990 physical-group fetches
and decompressions over Commit. Plain/diagnostic cohorts are separate; their
sequential n3/n2 median difference does not identify instrumentation overhead.

Mandatory corrections to the Stage1 recommendation:

1. **224ms is an inferred allocation, not a directly timed removable subset.**
   There are skipped-page counts but no skipped-only time measurement.
2. **30.1ms is composite compact decoding**, including allocations, canonical
   checks and identity derivation. It is not isolated hashing or guaranteed savings.
3. Parent compact branches store aggregate count and (max key,child ID), not each
   child's count/size/occupancy/level. Parent-minus-modified arithmetic cannot
   independently validate aggregate correctness or recover individual summaries.
4. Reading unchanged siblings currently checks canonical form/minfill, child
   level/key, parent aggregate and rebalance inputs. Keeping authentication only
   on pages still opened is NOT equivalent to preserving these checks.
5. Do not require logical reads to equal entered pages: boundary materialization
   can be needed for canonical redistribution. Reduce repeated physical work
   without removing required validation.
6. Edit publication5050 rows includes the final required100-row snapshot; only
   intermediate4950 rows are redundant candidates. Whole-cell92–104ms is not
   proven removable edit time; host consume is nested in backing round trips.
   ~86% edit wall remains unattributed. No edit optimization/latency promise here.
7. Retained/FUSE chain “edit+Commit#1” must not add all edits to just Commit#1.
   Pair each edit with its corresponding Commit, or sum the entire chain including
   all Commits. Correct the new analyzer; preserve archived results unchanged.

## Source trace and feasibility decision

Trace every caller/adapter in the real path before edits. Relevant sources:

- `crates/layerfs-content/src/tree/batch.rs`: Engine::read185, Node::existing150,
  check_child366, sibling487, branch loop579, CompactInodes::decode946.
- `crates/layerfs-content/src/tree/compact.rs`: branch representation290,
  canonical decode/re-encode validation406.
- `crates/layerfs-content/src/object/access.rs`: get_authenticated_batch and its
  default per-object implementation. Verify actual dispatch through ObjectBuffer
  and ObjectStore adapters; calling a batch-named method alone proves nothing.
- `crates/layerfs-layerstack-store/src/objects/read.rs`: read_metadata_values191,
  extract_record_group875, extract_demanded_group1062, visit_locations1671,
  visit_wave1725 and Metadata extraction branch. Trace dependency-pool reads too.
- `crates/layerfs-workspace/src/changes.rs`: namespace phase and FrontierInodes::finish;
  `objects.rs`/buffer adapters for authentication and lifetime; SDK/lifecycle entry.

Existing visit_locations sorts by(pack,group,record) and uses bounded waves;
visit_wave can decode a metadata record group once for multiple demands. However
dependency value-pool groups may still be decoded repeatedly. Stage1's5990 calls
do not reveal distinct groups or reuse distance. Do not expand the unrelated
SmallContent wave-local FULL-base cache as a substitute for evidence.

Phase A: freeze a small diagnostic screen first, recommended n=2 independent
fresh K100 cells and n=2 K10 cells in declared order. Reuse Stage1 inputs/proof.
Split physical reads into metadata-record groups, pooled-value groups and other
groups. Count repeats/opportunities and time extraction/decompression separately
from reconstruction/authentication, with matching operation boundaries.

Keep diagnostics bounded: fixed request-wave state or a charged bounded reuse
simulation (e.g. initial256KiB capacity, entry overhead included), no unbounded
history/ID map and no payload dumps. Report hits/misses/evictions/peak charge and
time/bytes potentially reusable. A simulation is not a measured candidate gain.
Prefer evidence of grouping within a known sibling wave over a speculative cache.

After Phase A, write and commit an evidence-based implementation contract before
product changes or candidate sampling. Select ONE mechanism:

- Prefer existing bounded authenticated batching when it demonstrably removes
  material repeated group work. Preserve key order/output identity even if the
  physical reader sorts demands. Account retained decoded pages before batching.
- Only consider operation-scoped group reuse if batching is insufficient and the
  capacity/reuse trace supports it. Require immutable Store/snapshot/pack/group/
  format identity, exact authentication, rollback/publication lifetime correctness,
  explicit eviction and bounded concurrency. Fit existing budgets; no global cache.
- If neither supports a useful result, report a blocker. A separate smaller
  follow-up is authenticated summary-only decoding of unchanged compact children,
  preserving compact::decode_inode and all existing checks while avoiding synthetic
  IDs. Its available envelope is LESS than30ms; do not silently substitute it
  under the200ms target or run successive unfrozen experiments.

## Environment, custody and how to run

At preparation main HEAD was0650b4945; this handoff adds docs only. Record actual
HEAD/status, full tracked binary diff and hashes of preexisting untracked files.
Preserve compaction-removal treatment and issue112/issue113 files byte-identically;
no stash/reset/revert/clean or accidental commits. Use isolated source snapshots
including the exact existing dirty treatment. A clean checkout alone is not control.

Historical main product:
`760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`;
plain campaign source:
`490938083f8a7d7ecec166ae2e20c7287d0f7c1e890c504fd5c5c05217be3a8e`.
Verify drift and rebuild identities; never assume binaries/images match a tag.

Read-only Stage1 root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z/`.
Contains collect.py/analyze.py/validate_fixtures.py/finalize.py, stage1-plain and
stage1-diagnostic sources, build scripts/logs, identities, raw cells and manifests.
Earlier baseline root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z/`.
All historical roots, sources and Stores remain read-only. NEVER run their scripts
in place: they write beside themselves. Create a new timestamped external root
and copy/adapt the needed source/scripts into it. Register an evidence pointer.

macOS owns SQLite, SDK/coordinator, canonical construction/admission/publication,
and physical spool. Docker Desktop owns daemon/live workspace/FUSE/workload only.
Containers:2CPUs,2GiB,no swap,256PIDs; record host resources separately. No Docker
SQLite, prebuilt Store mounts, changed topology or compile/profile treatments.

From EACH isolated source root, use the actual shared build entrypoints:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
```

Capture logs and exit codes separately; abort dependent execution on build failure
(old shell examples print exits but do not necessarily fail the script). Record
source/product/workload seals, binary SHA256 and immutable Docker image ID as well
as tag. Host jobs<=8 and existing Docker job policy stay matched between arms.
Runner builds own their locks: do not wrap them in a second measurement lock.

Archived Stage1 collector interface is `python3 collect.py plain|diagnostic`.
It expects stage1-<arm> directories, per-arm build-image logs, validation.json
named fixture-validation.json, and fresh arm/cell directories. **Adapt it first:**
the next campaign requires control/candidate and plain/diagnostic distinctions,
plus alternating paired order. Update imports, paths, identity selection, result
fields AND analyzer expectations. Do not invoke whole-control then whole-candidate
collections and claim alternating pairs. Freeze/document the actual new CLI.

The underlying real host command remains:

```text
<qualified-binary> commit-baseline <fresh-store-dir> <fixture-payload> \
  <live-container-id> namespace-100000 <cell>
```

This harness is in the archived campaign source, not a registered
`--family commit_baseline`. Copy/reuse it rather than invent that runner command
or substitute init_namespace/perf.sh. Keep harness bytes identical across paired
arms; instrumentation, if needed, is equal across diagnostic control/candidate.

Reuse runtime.start_sample/container cleanup and collect.py environment:
LAYERFS_EXEC_TRANSPORT=daemon, LAYERFS_FUSE_TRANSPORT=daemon, matching
LAYERFS_V013_IMAGE, workload path/usr/local/bin/fs-benchmark-workload, cell TMPDIR.
Parent collector owns runner-compatible layerfs-infra-measurement.lock at the
same TMPDIR/default path as builds. Child TMPDIR changes must not relocate the
parent lock. No child double-lock; no overlapping tests/builds/proofs/measurements.
Retain live command progress, logs and exits. Never interrupt another owner's run.

No crates/tools/benchmark edit after sealing through completed collection without
resealing/rebuilding. Do not time development instrumentation as plain candidate.

## Fixture and measurement protocol

Full original pseudorandom namespace-100000:100000 files,1000 data directories,
500000000 logical bytes, digest
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.
No structured-text variant, reduced workload or changed edit classes. Validate
full content/inventory and file0640/dir0750/mtime1700000000000000000ns including
root. Cache directory key in collector is not the content digest. Prefer original
immutable fixture; copying bytes alone previously changed directory metadata.

Fresh public Init+fork per independent Store outside Commit timer, no prepared
output Store substitute. Bootstrap112451 canonical objects/513026835 bytes/
100002 pool values. K100 final112684 objects/513774250 bytes/100 added pool values
are reference checks for this unchanged workload. Also verify exact roots/content
and comparison semantics; aggregate counts alone cannot establish correctness.

Cache policy remains commit-study-os-uncontrolled; retained/reopened is lifetime,
not OS cold. No untimed warm-up Commit. Report cache/lifetime/sequence/route and
process disk bytes per sample. No mixing with cold Init or diagnostic qualification.

Primary cells: nochange, retained K1 marker/repeat/revert/no-edit chain, k10, k100,
fuse-posix. Reuse exact old edits/markers/offsets and full changed-byte proofs.
Plain recommended n=3 pairs/cell in C1,T1;T2,C2;C3,T3 order. Diagnostic n=2 pairs
for retained/k10/k100 in C1,T1;T2,C2 order, separate from plain. Declare exact
phase/cell order and scope before results. For new shared reader changes, add
the applicable smaller-tier/reopened and shared-caller checks before promotion,
based on the actual changed call graph; reuse unaffected #104 evidence.

Never count sequential Commits on one Store as independent repetitions. Freeze
missing-evidence invalidity and at most one replacement of the entire affected
pair/cell for demonstrated infrastructure invalidity. Retain valid outliers,
failed attempts and all raw rows; no slower-arm retry or selective exclusions.

## CPU, memory, integrity and scaling gates

- Existing4MiB tree scratch ceiling remains. Charge new state/buffers before
  allocation; retain current reader/admission/expansion budgets and worker counts.
  No automatic extra8MiB allowance. Account coexistence and concurrency, including
  intermediate decoding and callback-retained pages. Direct bounded fallback
  must retain correctness and its work counts must be reported.
- K100 total user+system CPU must fall in matched plain pairs; preferred>=25%
  reduction (~240ms versus current~320.5ms). Freeze paired aggregation. For small
  cases use a predeclared tolerance, proposed max(10% of control,1ms), alongside
  public-wall and edit+Commit nonregression. Do not manufacture a gain by eager work.
- Measure charged scratch high-water plus host/daemon peaks, swaps/OOM, threads,
  spill/disk bytes and cleanup. Post-call RSS92MiB is not an operation-peak limit.
- Preserve4KiB, schema10, exact CAS, canonical authentication, pooled-group digest
  checks, pack/zstd/FULL/DELTA/CDC and transaction/rollback/publication semantics.
- Test unchanged malformed siblings: incorrect level/max key, parent count,
  noncanonical form, underfill, corruption, missing dependency. Require equivalent
  rejection behavior; valid-fixture root proofs alone are insufficient.
- Extend existing focused suites for generic directory/inode callers, boundary
  keys, split/merge, insert/delete/root growth/collapse, hardlink references,
  reversion, rollback, low scratch and fallback. Existing release test targets
  hit a preexisting debug_assertions gate: record limitation, use supported focused
  test profile and qualified release benchmark binaries; do not weaken tests.
- Verify complete changed-file bytes, unchanged sampled files, head/result,
  final root/content after reopen, physical counts and workspace/container cleanup.
- Mechanism counters must explain gain: same required logical validation, fewer
  physical extractions/decompressions/decoded bytes, bounded scratch, no new
  repeated scan under eviction. Add fixedN/varyK and fixedK/varyN work-count tests,
  clustered/spread keys and budget thresholds; do not infer asymptotics from3 timings.
- Keep the scaling ledger OPEN for triangular edit facts, quadratic spill merge,
  repeated-reopen history and untested range/fallback paths. This change alone
  does not eliminate all O(n²) mechanisms. No unrelated fixes in this experiment.

## Deliverables and stop

1. Commit a Phase A contract/evidence pointer before diagnostic edits; then a
   measured feasibility decision and Phase B candidate contract before optimization.
2. Produce `commit-stage2-read-results.md` here: repeated-group evidence, chosen
   mechanism, exact seals/commands, plain paired raw timings/medians/ranges and
   paired differences, separate diagnostic attribution, CPU/peak memory, physical
   receipts, adversarial correctness and scaling coverage, all failed attempts.
3. Report worthwhile-screen and preferred-target outcomes separately; state residual
   cost and next step if missed. No “224ms measured saving” or “30ms hashing” claim.
4. Update #111 with correction, outcome and #115/#108/provenance links. Keep cold
   Init target/open status intact. Commit only intended product/tests/docs in
   focused commits, preserving all preexisting dirty files. Keep diagnostics
   isolated unless separately justified as permanent telemetry.
5. Stop after this ONE experiment. If feasibility fails, finish the report with
   the bounded alternative rather than silently starting a second candidate.
   No release/tag/deployment, no broader storage-policy change or Init restart.

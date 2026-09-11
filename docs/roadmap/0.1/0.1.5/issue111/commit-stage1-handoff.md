# Handoff: Stage 1 — attribute Commit and edit cost at 100,000 files

Execute this task end to end: inspect, freeze a bounded protocol, prepare isolated
source snapshots, build, collect plain and separate diagnostic evidence, verify,
report, and update #111. This prompt authorizes execution and focused diagnostic
changes. Do not stop at a plan. **Stop after measured attribution and a concrete
Stage 2 experiment recommendation; do not implement an optimization.**

## Objective: be precise about what we are optimizing

Repository: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`.
Primary namespace: **100,000 files**, not one million. N is total namespace
size; K is the number of changed files before one Commit. Primary operation:
public `Client::commit_workspace_session` after K=1/10/100 changes in the same
100,000-file namespace. Secondary metric: all edit preparation plus that Commit,
computed as a per-sample sum before aggregation.

We have completed the current-main baseline. Stage 1 must identify what actually
consumes the **241 ms namespace phase of a 313 ms K=100 Commit**, and the
**659 ms SDK edit stage preceding it**, without moving work between timers.
The outcome is an evidenced optimization mechanism, its removable-cost ceiling,
and CPU/memory/scaling requirements—not another broad baseline report.

Proposed subsequent optimization targets, to assess and freeze before a later
candidate experiment (not Stage 1 performance acceptance gates):

| Metric at N=100000, retained Store | Baseline median | Proposed useful-win target |
|---|---:|---:|
| K=100 namespace | 241.33 ms | <=120 ms |
| K=100 public Commit | 313.29 ms | <=200 ms |
| K=10 namespace | 62.86 ms | <=31 ms |
| K=10 public Commit | 80.50 ms | <=50 ms |
| K=1 public Commit | 18.84 ms | No material regression |
| No-change / repeated change / revert | See baseline | No material regression |

Do not promise these gains. If attribution shows inadequate removable cost, say
so and recommend a defensible target before implementing a future candidate.
Do not set an edit-stage millisecond target before measuring its internal cost.
For edits, the required direction is linear useful work and lower total CPU.

Owner requirements: CPU and memory safe, small resource growth acceptable only
with an explicit bound, and eliminate quadratic scaling mechanisms. This means
linear or near-linear work in relevant changed items/bytes; it cannot mean
sublinear processing of K actual changes. Stage 1 identifies and quantifies
mechanisms; it cannot declare them eliminated before implementation/proof.

## Issue and rollout context

Read https://github.com/Ephemeral-AI-Lab/layerfs/issues/111 and completed baseline
comment https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5637126575.
Read the latest rollout update as well. #111 remains OPEN under its original
cold Init <=2.7 s objective. Last valid recorded cold Init median was ~3.420 s;
further Init optimization is paused during this Commit work. Never apply the
2.7 s Init gate to Commit or relabel uncontrolled bootstrap Init as cold.

Rollout: (1) attribution; (2) one measured namespace fix; (3) redundant edit
preparation; (4) spill/fallback/range scaling repairs; (5) reopened history work;
(6) combined qualification. Only Stage 1 is authorized by this handoff.
Related issues: #115 shared metadata/admission, #108 branch history, and
#109/#110/#106/#102/#104/#100/#107 for provenance and qualification context.

## Read first and preserve current state

Read all applicable AGENTS.md files, including parent/root if present and
`benchmark/AGENTS.md`. Root AGENTS.md was absent at handoff preparation; do not
invent instructions or assume it remains absent. Also read:

- `docs/roadmap/0.1/0.1.5/issue111/commit-baseline-results.md`
- `commit-baseline-contract.md`, `commit-measurement-handoff.md` in that directory
- `commit-optimization-direction.md` (three-subagent audit and corrected hypothesis)
- `fingerprint-index-results.md`, `metadata-proof-experiment-results.md`
- `docs/general/benchmark_rules.md`, especially timer, cache, paired arms and custody
- `benchmark/fs-bench-pro/QUICKSTART.md`, `shared/runner.py`, `shared/runtime.py`
- `docs/roadmap/0.1/0.1.5/hybrid_mental_model.md`, `spec.md`, `compaction-removal.md`

At preparation HEAD was `faaa03933` on main; this handoff adds documentation only.
Re-read HEAD/status. Never reset to this recorded hash. Preserve the existing
tracked compaction-removal changes and untracked compaction-removal.md,
issue112/issue113 content byte-identically. Record status, tracked binary diff,
and hashes of preexisting untracked files before work. Do not stash, clean,
revert, attribute or commit another owner's work. Treatment: promoted-uncompacted.

Use isolated source/worktrees with an exact copy of the declared dirty treatment;
a clean worktree alone would omit it. Do not blindly apply a historical patch
over later main changes. Detect and report drift before freezing sources.

Historical main product seal:
`760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`;
retained binary SHA256:
`0051058ac8e9ffca19fee65e595c19a43abc64ad315536aa14abc2f7e6983b63`.
The completed baseline plain harness had source `490938083f8a7d7e…`, binary
`a61ac03724277c9d…`; diagnostics `9218655581b187ad…`, binary `8f4ce307…`.
Read full identities from the evidence files. These are references, not permission
to skip current sealing. Plain must retain current product behavior; diagnostics
have their own product/source seal because instrumentation changes code.

## Existing evidence and how to reuse it

Read-only completed campaign root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z/`.
It contains:

- `collect.py`, `analyze.py`, `validate_fixtures.py`, `dev_smoke.py`, `finalize.py`
- `harness.patch`, `diagnostic-only-instrumentation.diff`, full instrumentation patch
- `source-plain/benchmark/fs-bench-pro/src/commit_baseline.rs` and main dispatch
- `source-diagnostic/` with historical diagnostic sources
- `plain-identity.json`, `diagnostic-identity.json`, image identities
- plain/diagnostic raw cells, summary.json, fixture-validation.json and manifests

Create a **new timestamped evidence root** outside the repository. Copy the needed
scripts/harness into it and adapt them there. Never execute archived collectors,
validators, analyzers or finalizers in place: they write beside their own scripts.
Never alter any old source snapshot, Store, manifest or result.

The archived collector's real interface is `python3 collect.py plain|diagnostic`.
It owns the measurement lock, resolves ROOT to its script directory, expects
`source-plain`/`source-diagnostic`, per-arm build-image logs, fixture-validation.json,
and new arm/cell output directories. It imports the runner/runtime from its source
snapshots, validates seals, starts containers, invokes the host harness and cleans
up. Its default matrix is the old full study: adapt BOTH collector and analyzer
to the bounded Stage 1 matrix before freezing them. Do not accidentally rerun
all old cells or leave an analyzer expecting cells you intentionally excluded.
Review the old plain eligibility field: this means plain study timing, never
cold-Init or release qualification. Nonce observations remain diagnostic only.

Keep #104/#109/#110 evidence and other issue111 evidence roots read-only too.

## Benchmark environment and actual commands

macOS host owns SQLite, SDK/coordinator, canonical construction, object admission,
Commit publication and physical spool. Docker Desktop provides Linux daemon,
live FUSE/workspace core and workload only. No Docker-owned SQLite, prepared Store
mount, alternative topology or historical fallback. Standard container limits:
2 CPUs, 2 GiB memory, no swap, 256 PIDs. Record host and container scopes separately.

Build from EACH isolated source root with the shared runner:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
```

Capture stdout/stderr, image tag, immutable image ID and exit codes into fresh
evidence files; preserve the collector's expected build-image log naming or
explicitly update its reader. Host jobs are at most eight; Docker compilation
uses its existing policy. Do not change profiles or jobs between matched arms.
The runner owns build locks. Do not wrap these commands in a second lock.

After harness/scripts/fixtures are frozen and built, from the NEW evidence root:

```bash
python3 collect.py plain
python3 collect.py diagnostic
python3 analyze.py
```

These commands apply to your reviewed/adapted copies, not the archived scripts.
The collector acquires the runner-compatible `layerfs-infra-measurement.lock` in
the parent process TMPDIR (or /tmp). Keep its resolved path consistent with the
runner; per-cell TMPDIR is assigned only to children after acquisition. No nested
lock acquisition in children, no overlapping builds/tests/proofs/measurements,
and no interruption of another owner's run.

The actual archived host harness invocation is:

```text
<qualified-host-binary> commit-baseline <fresh-store-directory> \
  <validated-fixture-payload> <live-container-id> <scenario-id> <cell>
```

This is a campaign-specific harness, not a registered `--family commit_baseline`.
Do not invent that family or run init_namespace/perf.sh as a substitute for these
Commit intervals. Reuse runtime.start_sample and collector lifecycle. Existing
environment includes `LAYERFS_EXEC_TRANSPORT=daemon`, `LAYERFS_FUSE_TRANSPORT=daemon`,
the matching `LAYERFS_V013_IMAGE`, workload `/usr/local/bin/fs-benchmark-workload`,
and per-cell TMPDIR. Read collect.py for full arguments/deadlines; retain exact
argv/environment identities without dumping unrelated secrets.

No crates/tools/benchmark edits between build and completed collection without
resealing/rebuilding. Capture commands with live progress and retained logs/exit
codes. Do not rerun valid slow samples for nicer numbers.

## Fixture, lifetime and timer contract

Full original pseudorandom `namespace-100000`: 100000 files, 1000 data directories,
500000000 logical bytes. Fixture digest:
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.
Do not select the newer structured-text variant or change file classes/sizes.

Validate full inventory, payload SHA256, file mode0640, directory mode0750 and
mtime1700000000000000000 ns, including payload root. Read the collector's fixture
registry/path resolution; its directory cache key is not the content digest.
Prefer the original immutable input. Prior cp -cR directory-mtime drift changed
canonical counts despite identical content; never infer validity from byte hashes
alone or repair a measured fixture retroactively.

Fresh public Init + fork per independent Store, outside Commit timing; no cloned
output Store substitute. Expected 100k bootstrap: 112451 canonical objects,
513026835 canonical bytes, 100002 pooled metadata values. Gate these identities.
Use unchanged baseline edit selections/markers/offsets: retained K1 marker chain,
K10/K100 spread ordinals, skipped empty files, and separately labeled FUSE route.
Read the old contract/harness rather than approximate them.

Cache profile remains `commit-study-os-uncontrolled`. Store reopen means dropping
all owners and reconnecting; it does not mean OS cold. No extra untimed warm-up
Commit. Report read/write bytes and lifetime/sequence per sample. Do not pool
plain/nonce, SDK/FUSE, retained/reopened or different change sets.

Timer: only the public Commit call. Record setup, reconnect, workspace create,
every edit, checks and cleanup separately. Aggregate edit+Commit per sample.
Never move final snapshot or index work into setup to improve the timer.

## Required investigation and minimum diagnostics

Correct the preceding report's hypothesis explicitly:

- `crates/layerfs-workspace/src/changes.rs:805` ends content; namespace starts806
  and includes apply_references and inodes.finish. Directory update work in the
  preceding loop is not this namespace timer; measured content edits have no
  directory binding mutations.
- At changes.rs:2414, inode deltas already enter one sorted batch.
- `crates/layerfs-content/src/tree/batch.rs:579` reads sibling children before
  unchanged-range return519; compact decode946–955 derives hashes per record.
  Measure this hypothesis; do not describe it as the proven hotspot yet.

Reuse TreeBatchCounters before adding new diagnostics. Expose batch calls,
nodes read/created/reused, changed keys, scratch peak; currently the finish caller
discards them. Add minimum counters for compact records decoded/IDs derived,
authenticated bytes, checkpoint validation/record encoding, fallback reasons
and mutations. Avoid unbounded sets of object IDs just to count unique reads;
if uniqueness is needed use bounded diagnostic state and report limits.

Provide disjoint inner namespace clocks: reference handling, checkpoint-related
work, record encoding, tree application, fallback, final root encoding, residual.
Avoid double-counting nested callbacks and parent intervals. Preserve outer
phase equations; do not add summed per-thread CPU into a wall-time partition.
Use aggregate counters and boundary clocks rather than high-volume per-record logs.

For edits, trace `crates/layerfs-fuse/src/live_owner.rs`: EDIT_BEGIN -> freeze ->
publish_facts (around2404–2627); cached-inode flush2344; retirement2949; range/folio
matching around1185. Count/time snapshot calls, serialized fact rows/bytes, cache
invalidations, scanned backing references and range tests across the full sequence.
Separate host and daemon clocks; correlate calls without subtracting timestamps
from different processes. Facts are consumed by frontier construction at
changes.rs:354; prove the consumer boundary before recommending skipped publication.

Confirmed algorithmic debt to retain in the report:

- growing full dirty snapshots can process triangular total rows;
- frontier spill merge changes.rs:2259/2288 explicitly O(K²/buffer);
- range×folio and growing barrier scans need their own counts;
- sorted-batch fallback2430–2450 must be tested, not automatically labeled K²;
- history replay is O(H) per reopen and can be quadratic cumulatively. This is
  a separate workstream, not a Stage 1 optimization target.

## Freeze a bounded matrix before instrumentation

Recommended core at N=100000: nochange, retained K1 marker/repeat/revert/noedit
chain, K10, K100, and labeled fuse-posix companion. Plain n=3 independent fresh
Stores/cell; diagnostic n=2. Freeze exact order, e.g. all plain then diagnostic,
each nochange/retained/k10/k100/fuse-posix and repetitions ascending. This measures
instrumentation overhead, not a candidate speedup. Reuse the existing reopened
and smaller-tier baseline unless a stated attribution question needs new samples.

Add bounded counter-only adversarial tests for spill threshold, forced fallback,
growing dirty/cache/range sets and overlapping ranges. Use existing tests with a
small policy to expose algorithmic growth; do not represent them as authentic
100k timing. Stage 1 need not launch the full future production-budget scaling
campaign. Freeze any required extension before inspecting its timings.

Create/commit `commit-stage1-contract.md` and an evidence pointer before code
edits/collection. Freeze fields, repetitions, ordering, resource limits, timeouts,
invalidity criteria and replacement rule. Retain every attempt. At most one
whole affected cell/pair replacement for demonstrated infrastructure invalidity;
never remove valid outliers or retry the slower observation alone.

## Correctness, resources and deliverables

Preserve schema10, 4KiB, authentication, exact CAS, zstd/pack/FULL/DELTA/CDC,
metadata/admission/cache limits and worker counts. No compaction/VACUUM/repack/GC.
Instrumented runs must preserve bootstrap identity, Created/UpToDate outcomes,
changed-file full bytes, unchanged sampled files, final head/root/content after
reopen, physical receipts, admitted metadata values and clean workspace/container
shutdown. Run focused existing workspace/content/fuse tests for touched paths.

Record user/system CPU, threads, disk/spool growth, swaps/OOM, host and container
memory. Distinguish post-call RSS, process-lifetime peaks and operation peaks.
Do not claim operation peak safety from the first two. Diagnostic scratch must
be explicitly bounded; no unbounded history-sized counter maps. For a future
candidate, prefer no extra allocation; proposed additional scratch ceiling8MiB
per active operation must also have an aggregate concurrency bound. CPU must
decrease on large changes and not materially regress small cases. Stage 1 must
recommend precise measurement/gating rules, not assert these conditions already pass.

Deliver `commit-stage1-results.md` under issue111 with:

1. Exact current identities and comparison to prior main; all raw rows, n,
   median/range, cache/lifetime/route, diagnostic overhead and any invalid attempts.
2. Reconciled inner namespace table and edit-stage table, operation counts,
   resources, correctness and a simple actual-path diagram.
3. Explanation of the 241ms namespace and 659ms edit costs, with measured versus
   unmeasured portions explicit. Correct the old directory-batching hypothesis.
4. Ranked measured removable costs and ONE concrete Stage 2 namespace experiment,
   expected ceiling, proposed targets, exact code boundaries, CPU/memory budgets,
   tests and falsification conditions. If no namespace experiment is justified,
   say so and recommend the measured alternative without silently implementing it.
5. A scaling ledger separating confirmed quadratic mechanisms, potential risks,
   bounded ordinary work and untested fallbacks; next proof required for each.
6. Evidence root/manifests, commands/exits, seals before/after, independent proof
   coverage and confirmation that preexisting dirty files stayed unchanged.

Update #111 with these results and links to #115/#108 and existing provenance;
leave the cold Init target/open status intact. Commit only intended diagnostic,
contract and report files. Keep nonce instrumentation isolated unless explicitly
justified as permanent telemetry; no optimization promotion, release/tag/deploy.
Finish with a handoff-ready Stage 2 recommendation, not an implementation.

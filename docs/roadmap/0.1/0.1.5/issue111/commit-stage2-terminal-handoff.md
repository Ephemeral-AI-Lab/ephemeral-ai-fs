# Continuation: bring Stage 2 to terminal pass with trustworthy calibration

Continue end to end until Stage 2 satisfies the terminal criteria below: audit
execution identity, resolve calibration drift, finish hardening/verification,
fix evidenced failures, measure and report. This authorizes necessary focused
code/harness/test repairs and qualification; do not stop after another exploratory
calibration. Do not manufacture PASS: if an external prerequisite makes progress
impossible, give the exact blocker and retained evidence, never relabel failure.

Repo `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`; issue #111. Current preparation
HEAD `2b1459fe3`; promoted Stage2 `bd9dc1600`, hardening `259a80a4b`. Re-read actual
HEAD/status and preserve later work. This is the continuation of Stage2, not a
new Init campaign or spill/edit optimization project.

## Objective, present evidence and interpretation correction

Optimize/qualify public Commit inside the ORIGINAL pseudorandom100000-file
namespace, N=100000, K=1/10/100 changed files. Preserve correctness, authentication,
4KiB, bounded memory/CPU and prior read-batching benefit.

Read `commit-stage2-hardening-results.md`, especially sections3.3/3.4/4/5/7/8.
Three-point calibration (post-hoc, n3) reported:

| Arm | Public Commit median | Namespace median |
|---|---:|---:|
| A: pre-Stage2 | 177.25ms | 108.32ms |
| B: promoted Stage2 | 177.31ms | 104.70ms |
| C: hardened | 174.72ms | 104.26ms |

Original Stage2 A namespace was240.68ms. The reported same-source A is now108.32ms,
an unexplained2.2x shift. Never label this OS-cache/host noise without evidence.
Calibration JSON only reports timing/CPU, so reconstruct full execution custody
from raw commands/identities if available; if missing, mark historical calibration
identity unverified and produce new correctly bound evidence.

The five-pair B/C hardening comparison (173.50/173.16ms public,105.09/105.36ms
namespace) shows parity, which is EXPECTED for correctness hardening. Requiring
another25% speedup over B uses the wrong comparator for preserving Stage2.
Do not use that failure to conclude hardening lost the original gain. However
the historical frozen gate actually contained it: retain its FAIL and explain
the design error in an append-only correction and a new prospective contract.
Do not silently rewrite old gates or erase real K10/small-case breaches.

Separately, A/B/C parity in the calibration leaves the original optimization
benefit unconfirmed in that session. Both questions must be resolved separately.

## Terminal PASS: all required conditions, never median-only blanket claims

Freeze exact sample counts, order and equations before new qualification:

1. **Execution identity and calibration:** A/B/C executable/image/harness/input
   bindings are proved; the apparent old-control drift is attributed to measured
   differences or corrected as invalid identity/measurement evidence. A changed
   absolute clock alone is not a failure to be retried until historical240ms
   returns. Explain any remaining absolute drift and do not claim it resolved
   merely because a new run looks familiar.
2. **Original optimization comparison:** freshly paired A/C (or A/B plus B/C)
   proves the read-work reduction and original worthwhile screen: K100 namespace
   ratio<=0.75 in every declared pair; public wall and CPU lower in every pair.
   If genuine conditions remove the benefit, report that and investigate a
   specific root cause; do not slow A, warm only C or choose a favorable workload.
3. **Hardening preservation comparison:** freshly paired B/C uses per-pair
   noninferiority for public wall and user+system CPU, allowance
   max(10% of B,1ms). Parity passes this comparison; no extra25% gain is required.
   Preserve existing explicit descriptive treatment of FUSE if inherited, and
   state its results separately; freeze any stronger final gate prospectively.
4. **Absolute C targets:** K100 public<=200ms and namespace<=120ms for every
   sample; K10 public median<=50ms and namespace median<=31ms. Report all samples,
   ranges, medians and breaches. Do not weaken targets because a control drifts.
5. **Correctness and resources:** all selected focused regressions, independent
   verification and affected shared-caller proofs pass; declared scratch/reader/
   concurrency bounds hold; no unexplained safety/accounting/fallback gap is
   covered by a generic “correctness qualified” statement.
6. **Custody and reproducibility:** final evidence is complete, append-only and
   bound to the exact final committed treatment. All disclosed historical losses
   remain visible. Stage2 can pass a new complete qualification without pretending
   deleted historical evidence has been recovered.

If all cannot pass yet, continue with a concrete root-cause hypothesis and a new
versioned attempt only after a material repair or documented infrastructure
correction. No unlimited unchanged reruns, selective slower-arm retries, relaxed
tolerances or discarded valid outliers. “Until terminal pass” authorizes completing
the work, not manipulating the measurement.

## Read-first context and preserved state

All applicable AGENTS.md (parent/root if present and benchmark/AGENTS.md),
docs/general/benchmark_rules.md, benchmark/fs-bench-pro/QUICKSTART.md, shared
runner/runtime, verify-selected.py and the actual family registry/help.
In this issue111 directory read:

- commit-stage2-hardening-contract/results/evidence-pointer/handoff.md
- commit-stage2-read-contract/results/evidence-pointer.md
- commit-stage1-results.md, commit-baseline-contract.md
- commit-stage2-read-handoff.md and prior format/authentication context

Historical roots, ALL read-only:

- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-hardening-evidence/20260912T120000Z`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z`
- baseline/earlier #104/#109/#110 roots cited by those reports

Known identities (reference only, verify): A product760eb0f2…;
B product `a608cd4edd25161584986b0f2885d2497a0c231a63a0b3dc7be73685bd0c0b38`;
C hardened product `a54ef6e3f6d94fb10eb131bbdabec52ae9b903572a42f1b685f7c2d1e0ee3be8`.
Hardening report B binary3b46be77…/C binary400e5b08…; read full recorded hashes.
Never infer actual execution from source contents or image tags alone.

Capture tracked diff and untracked hashes before work; preserve compaction-removal,
issue112/issue113, untracked web/ and all unrelated files. No stash/reset/clean/
revert or broad staging. Product changes overlapping objects.rs must be staged as
intended hunks only. Isolated control/candidate source must include the declared
dirty treatment; clean HEAD alone is not the measured product.

## 1. Execution-identity audit FIRST

The previous hardening used `cp -a -l`; candidate writes changed archived sources
through shared inodes. Restoration from git index is not proof that a retained
binary matches restored source. Do not repeat that copy strategy.

Create independently owned new snapshots, no hard links and no shared target
directory between arms. Check inode/link identity of files that will be written.
Record actual snapshot hashes against original seal inputs/manifests, not only
git status. Leave historical trees untouched even when inconsistencies are found.

For each A/B/C sample, bind:

- resolved executable path, SHA256 before/after, build identity and source/product/
  workload/compilation/dependency seals;
- exact argv, cwd, relevant environment, image immutable ID and daemon binary;
- fixture content AND metadata identity, Store initialization receipt and selected
  edit paths/classes/offsets, compiler/profile/features and SQLite settings;
- source/build timestamps as context, never as substitutes for hashes.

First determine whether the archived A binary actually predates batching. Inspect
build inputs and executed paths; rebuild independently with the shared runner if
binding is absent. Test actual route dispatch through ObjectBuffer/ObjectRead
adapters, not merely the presence/absence of an API declaration in a source tree.
Check copied identity files, mutable image tags, target reuse and collector arm
imports for aliasing. If a source is repaired, it needs a new build/seal.

## 2. Matched mechanism diagnostics to resolve the drift

Use identical diagnostic definitions on A/B/C and a frozen balanced/interleaved
sequence. Record logical nodes/records, physical selections by call site,
decompression count/time/bytes, object-location queries, authenticated bytes,
group layout/reuse, CPU/waits and scratch. No unbounded trace maps. Keep diagnostics
separate from plain qualification and do not subtract estimated overhead.

Expected old evidence: A~5210 physical selections, B~1317, same2053 logical pages
and101001 decoded records. These numbers are reference observations, not magic
values to force. Distinguish:

- A still performs far more physical work but it is cheaper now: measure where
  per-call cost changed (storage layout, read state, compilation, CPU/waits).
- All arms perform batched work: audit wrong executable/dispatch/harness first.
- Layout/logical work differs: establish identity and source of the difference.

Reconcile phase walls to public timer and use actual per-process CPU. Explicitly
declared cache/read state must be symmetric across arms. No assumption that the
host is quiet or “cold”; record load/resource state and prove any blamed factor.
Do not require the old absolute time to reappear in order to accept a correctly
explained condition-dependent improvement; report its scope honestly.

## 3. Finish hardening and missing proofs

Audit code/test claims, not just report labels. Already fixed: batch demand order
and entry bounds, per-chain pool allowance, adaptive chunking. Reproduce any new
failure and make the smallest shared fix with meaningful tests.

Prior report limits must be resolved or precisely bounded:

- Sparse pooled DELTA-chain exhaustion was not executed. “Too large for this
  campaign” does not disprove a valid-format hazard. Use a bounded targeted corpus
  or faithful configurable-limit test plus a documented production-limit proof;
  do not demand giant fixtures unnecessarily or claim synthetic proof is full E2E.
- The claimed point-versus-batch budget sweep may use the same batch engine with
  different storage adapters. Inspect dispatch before concluding equal peaks
  disprove retained sibling pressure. Exercise actual batch-specific pressure and
  real workspace fallback; adaptive chunks must avoid restarting failing whole
  candidates. Record deterministic read/attempt counts.
- Check allocation arithmetic with actual `size_of`/capacity values; do not assume
  an ObjectId is8B or a fetched tuple is24B. Account tree-owned vectors/copies and
  simultaneous source-reader allocations, with separate component and aggregate
  bounds. Existing4MiB tree, reader/pool limits and workers stay unchanged.
- No operation-peak claim from post-call RSS. Gather charged high-water and bounded
  external peak/footprint observations with explicit sampling limitations.
- Run missing independent verification, including proof-only workspace_reliability,
  using the real supported verification entrypoint. Not being a performance family
  is not a reason to omit its proof. Read help/parser; current generic parser has
  `--verification` and `--performance-rows`, not a universal `--mode verification`.
  Bind exact image/source/input and row IDs where required. Do not invent flags.
- Freeze the affected shared-family/case matrix from the actual changed call
  graph. Include smaller tiers, retained/reopened, structural edits, corruption,
  rollback, low-budget paths and ordered mixed ownership; reuse unaffected #104
  evidence with stated applicability. Mark omissions as remaining work.

## Benchmark environment and commands

macOS: SQLite, SDK/coordinator, canonical construction/admission/publication, spool.
Docker: daemon/live workspace/FUSE/workload only,2CPU/2GiB/no-swap/256PID. No
Docker SQLite, alternate topology or page/cache/worker increase.

From each independently owned source snapshot:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
```

Preserve checked exit codes and full logs, stop dependent work on build failure.
Runner owns build locks. Do not wrap it in another lock. No resource-sensitive
overlap, no interruption of another owner. No source edits from sealing through
collection without resealing/rebuilding.

Copy/adapt the existing collector/analyzer into NEW evidence. Stage2 collector
interface is `python3 collect.py <sequence.json>` with entries arm/cell/repetition;
inspect hardening copy's actual interface before execution. Update paths/arm
imports, identities and analyzer together. The host command is:

```text
<qualified-binary> commit-baseline <fresh-store-dir> <fixture-payload> \
  <live-container-id> namespace-100000 <cell>
```

This campaign harness is not a registered `--family commit_baseline`. Keep it
byte-identical across compared arms. Parent collector owns the runner-compatible
measurement lock at the SAME parent TMPDIR/default path. Child TMPDIR changes
must not relocate that lock; no child double-acquisition.

Full immutable pseudorandom fixture digest:
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`;
100000 files,1000 dirs,500000000 bytes. Validate full content/inventory/modes/
mtime including root (file0640,dir0750,1700000000000000000ns). No text variant,
reduced count or byte-only copy. Bootstrap fresh public Init+fork outside Commit;
no prepared output Store. Same existing K1/K10/K100 markers/offsets/paths.
Cache profile commit-study-os-uncontrolled; no cold claim or untimed warmup Commit.

## Final qualification protocol and evidence

Create `commit-stage2-terminal-contract.md` and evidence pointer before repairs
or new measurements; identify superseded comparison logic explicitly. Use new
timestamped attempt directories with create-exclusive writes. NEVER rm-rf a prior
attempt or hard-link writable source. Old phaseA deleted raw records cannot be
recovered by a rerun and must remain disclosed.

Recommended final screen: n3 pairs for each required comparison/cell, alternating
C1,T1;T2,C2;C3,T3; distinct pair identity for A/C versus B/C. Core cells nochange,
retained K1/repeat/revert/noedit,k10,k100,fuse-posix. Freeze which cells test
optimization benefit and which test preservation; do not reuse a row as a new
independent sample. Choose any needed higher n BEFORE observing outcomes, with
reason. A/B/C diagnostic calibration has its own predeclared balanced order.

Freeze invalidity/replacement rules (at most one complete affected-pair replacement
for demonstrated infrastructure invalidity); all valid slow rows remain. A failed
criterion requires investigation, not extra unchanged runs to make a median pass.
Any material product/harness repair starts a versioned treatment with its own seal
and declared checks. Do not widen gates after seeing results.

Keep analyzer self-tests for per-edit+matchingCommit and full-chain sums. Recheck
raw receipts, phase equations, exact head/canonical/content proofs, admitted pool
values, cleanup, no swap/OOM, resource and per-sample gates. JSON and Markdown
verdicts must agree. No aggregate PASS when any required verification is NOT_RUN.

Deliver `commit-stage2-terminal-results.md`: identity audit, measured drift
explanation, A/C benefit versus B/C preservation, all raw rows/paired differences,
CPU and peaks/bounds, fixes/tests, independent verification/coverage, custody,
exact final product/source/binary/image seals and terminal checklist. Update #111
through progress and final outcome; cross-link #115/#108/#109/#110/#106/#102/#104/
#100/#107. Commit only intended changes, preserving all unrelated dirty work.

Do not stop at another report with known repairable NOT_RUN or analyzer errors.
Continue through the evidence-backed repairs and final proof. If terminal pass
remains impossible due to a true external blocker, state which exact gate fails
and why; keep Stage2 open rather than asserting success. No release/tag/deploy.
Cold Init<=2.7s and broader quadratic spill/edit/history work stay open and are
not part of this Stage2 terminal claim.

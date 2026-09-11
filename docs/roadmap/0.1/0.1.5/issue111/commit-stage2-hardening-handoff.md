# Handoff: harden and qualify the promoted Stage 2 batch reader

Execute end to end: inspect, reproduce review findings, freeze protocol, make
focused fixes, test, build, measure paired treatments, qualify affected callers,
report and update #111. This prompt authorizes that work. Do not stop at a plan.
The objective is to preserve the measured performance win while restoring exact
shared-reader behavior and proving bounds. Do not start a new speed experiment.

## Current state and objective

Repository `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`, issue
https://github.com/Ephemeral-AI-Lab/layerfs/issues/111.
At preparation HEAD was `1ce1313ee`; product commit `bd9dc1600` is already on main.
The original Stage 2 comment says unpromoted, but the later promotion record
supersedes that status. Do not reapply candidate.patch or promote a second copy.
Re-read current HEAD and detect subsequent changes before freezing.

Main plus preserved dirty treatment matched measured candidate product seal:
`a608cd4edd25161584986b0f2885d2497a0c231a63a0b3dc7be73685bd0c0b38`.
Verify this; do not assume the host binary/image was rebuilt on promotion.
The clean committed tree alone omits the uncommitted compaction-removal treatment.

N=100000 total namespace files, K=1/10/100 changed files per public
`Client::commit_workspace_session`. Original pseudorandom fixture,500000000 bytes.
Stage 2 paired results: K100 Commit314.66→176.59ms, namespace240.68→105.40ms,
CPU302.12→164.71ms; all3 candidate K100 calls174.01–182.93ms. Group selections
5210→1317 while logical pages2053 and decoded records101001 stayed unchanged.
Tree charged scratch164556→449492B, unchanged4MiB ceiling, unchanged workers.

Hardening performance targets to freeze before measurements:

- K100 public Commit <=200ms and namespace <=120ms; report median AND every sample.
- K10 public Commit median <=50ms, namespace median <=31ms; these are explicitly
  median targets, not per-call promises. Previous Commit max56.46ms exceeded50ms.
- Compare hardened candidate against the CURRENT optimized product in fresh pairs,
  not against the pre-Stage2 product. Preserve CPU and small-case performance;
  proposed per-pair noninferiority tolerance max(10% of control,1ms) for CPU and
  public wall, frozen independently from absolute targets. Report every breach.
- No new workers, higher cache/page settings or additional scratch allowance.
  A correct partial outcome that misses performance gates remains unqualified
  for those gates; do not weaken correctness to save timing.

Cold Init <=2.7s remains OPEN and paused; last valid recorded median~3.420s.
This task must not restart Init optimization or claim uncontrolled Init is cold.

## Read first

All applicable AGENTS.md (parents/root if present, `benchmark/AGENTS.md`),
`docs/general/benchmark_rules.md`, `benchmark/fs-bench-pro/QUICKSTART.md`.
Under `docs/roadmap/0.1/0.1.5/issue111/`:

- `commit-stage2-read-results.md`, `commit-stage2-evidence-pointer.md` (promotion)
- `commit-stage2-read-contract.md` (including deleted Phase A evidence disclosure)
- `commit-stage2-read-handoff.md`, `commit-stage2-phasea-contract.md`
- `commit-stage1-results.md`, `commit-baseline-contract.md`, relevant format docs

Read issue comment5638943024 and latest review update. Review `git show bd9dc1600`
and `git show 1ce1313ee`. Trace all callers/adapters of each function changed below.
The findings are static review findings; valid sparse-corpus and deep-tree
reproductions have NOT yet been executed. Establish them before claiming a bug
reproduced, and report any finding disproved by actual constraints.

## Priority 1: separate metadata cache lifetime from work-budget lifetime

`crates/layerfs-layerstack-store/src/objects/read.rs:1767–1790` now shares one
`PoolRead` among targets in a metadata record-group wave. The reusable cache is
intentional. `objects/metadata.rs:360–366` still accumulates decoded_work16KiB per
miss and rejects after32MiB (2048 misses). Previously that counter lived for one
metadata_chain; now unrelated targets can exhaust it together.

Construct an ACTUALLY VALID pooled DELTA/scattered-ordinal corpus, respecting
all record group, canonical closure, chain-depth and admission constraints, where
point reads succeed but combined reads exhaust the shared counter. Analytical
starting example: two12-version chains,100 rows/version with disjoint missed
groups: per-chain canonical closure97728B<128KiB,11edges<16,1200misses18.75MiB;
combined2400misses37.5MiB. This is a hypothesis recipe, not a validated fixture.
Include cache eviction/reloads, and confirm both target records occupy a wave
that actually shares PoolRead. Synthetic counter mutation alone is not enough.

Smallest fix: retain bounded cached decoded values across the wave, but reset or
separately own the work allowance per metadata_chain. Preserve the existing
512KiB/128group cache and per-chain limits. Never merely raise32MiB globally.
If an additional aggregate wave work bound is needed, make exhaustion split or
fall back semantically safely; healthy objects must not become Integrity errors
because another object was requested alongside them. Bound total wave work by
the fixed request count and individual limits, and document it.

The moved192KiB logical_work counter is different: existing16edges+anchor,
100rows/leaf and94B/row imply159800B maximum, further limited by canonical closure.
Verify/document that proof and a maximum-chain test; do not call it an established
regression or remove guards speculatively.

## Priority 2: batch API ordering and entry bounds

`objects.rs:3509` ObjectBuffer emits owned objects immediately and fetches missing
ones afterward, violating the demand-order promise in `object/access.rs`.
Example `[sourceA, ownedB]` can callback `[B,A]`. The tree currently searches IDs,
so this does not prove wrong current Commit roots; it is a shared API violation.

Preserve the ordered contract with the smallest bounded implementation. Test
mixed memory/source and spill/source inputs, duplicates, empty batches, missing
objects, callback errors and original identity checking. Enforce the supported
batch-count ceiling at entry BEFORE callbacks/allocation; all-owned oversized
input currently bypasses CoreReader's missing-subset limit. Reuse existing slot/
ordering helpers if present; do not invent an unbounded result map.

## Priority 3: actual deep-tree fallback and complete allocation accounting

In `tree/batch.rs`, batch_children reserves copied payload bytes for up to32
siblings. That lease survives recursive descent; live sibling chunks accumulate
along the active branch spine. Workspace's individual-mutation fallback can
re-enter the same compact batch engine. Therefore a one-byte root-read error test
does NOT establish successful batch-exhaustion fallback.

Construct a bounded valid deep/large-page tree or a supported reduced-budget
equivalent where the point route fits but fixed32 batching fails after entering
the branch. Exercise the REAL workspace path, require identical successful roots,
and count batch attempts, fallback entries/mutations, pages and repeated work.
Do not require a physically impossible enormous tree merely to manufacture depth;
document validity and what a reduced-policy test proves versus production limits.

If reproduced, prefer reducing chunk width to available scratch, eventually a
one-child/point route, before restarting the entire candidate. Preserve validation,
error handling and bounded work; no recursive retry of the same failing batch.
Do not change32 for normal cases without evidence or add configurable tuning.

Account payload copies PLUS ids/fetched vector capacities and descriptors. Source
read results coexist with callback copies and its pool/decode buffers. Charge
tree-owned state to existing4MiB and explicitly account source-owned simultaneous
allocations under existing reader bounds. Neither449492B tree charge nor post-call
RSS is a whole-operation peak. Record high-water and coexistence; preserve current
concurrency so aggregate bounds remain explicit. Fixed32 position/remove loops
are bounded per chunk, not an unbounded K² defect; do not optimize them here.

## Environment, evidence and actual execution

Use new timestamped evidence outside the repo. Snapshot complete git status,
tracked diff and preexisting untracked hashes before edits. Preserve all existing
compaction-removal work and issue112/issue113 files. No stash/reset/clean/revert.
If objects.rs overlaps dirty work, stage only intended hunks via a verified index
patch; never stage the whole file blindly. Compare preserved hunks/content before
and after (blob hashes naturally change when HEAD changes).

Read-only reference root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z/`.
Reuse copies of collect.py/analyze.py, sequence files, build scripts and harness
from its stage2-* source directories. Also retain Stage1/baseline/older roots
read-only. NEVER run archived scripts in place, overwrite or delete attempts.
Use create-exclusive fresh output directories; existing directory must fail.

Prior first PhaseA raw cells were deleted, disclosed in the amended contract;
only extracted values survive. Do not claim full historical custody passed or
attempt to reconstruct deleted raw evidence. The retained PhaseB paired results
remain usable under their stated limits. Keep the violation visible in context.

macOS owns SDK/coordinator, SQLite, canonical construction/admission/publication
and spool. Docker Desktop provides daemon/live workspace/FUSE/workload only:
2CPUs,2GiB,no swap,256PIDs. No Docker SQLite or alternative topology.

Build each isolated source including the exact dirty treatment using:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
```

Builds own their lock. Retain stdout/stderr and checked exits; do not continue
after a failed command. Keep compiler/profile/job policy compatible. Record
source/product/workload seals, binary SHA256, image tag AND immutable image ID.

Stage2 collector real interface is:

```bash
python3 collect.py <sequence.json>
```

Run only the adapted copy in the NEW evidence root. JSON has `entries` containing
`arm`, `cell`, `repetition`. Existing collector reads stage2-<arm> directories,
image-<arm>.txt and writes cells/<sequence-stem>/stage2-<arm>/...; adapt paths/arm
identities coherently and update analyzer. Root controls = promoted optimized
product, candidate = focused hardening only. Do not use archived pre-Stage2 control.

Underlying host harness:

```text
<qualified-binary> commit-baseline <fresh-store-dir> <fixture-payload> \
  <live-container-id> namespace-100000 <cell>
```

This is not a registered `--family commit_baseline`; copy the actual harness.
Reuse runtime.start_sample, daemon/FUSE environment and cleanup from collector.
Parent owns runner-compatible layerfs-infra-measurement.lock at the same parent
TMPDIR/default path as builds. No child double-acquisition or sensitive overlap.
Per-cell TMPDIR belongs to child only. Stream progress and preserve logs/exits.
No crates/tools/benchmark edit between build and collection without resealing.

Fixture digest `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`;
100000 files,1000 data dirs,500000000 bytes, original pseudorandom (not text).
Full inventory/content plus file0640/dir0750/mtime1700000000000000000ns validation
including root. Cache directory key is not content digest. No byte-only fixture
copy or retroactive repair. Fresh public Init+fork per Store outside Commit timer;
no prepared output Store substitute. Bootstrap112451objects/513026835B/100002values;
K100 final112684objects/513774250B/100 added values. Gate exact semantic identities
and proof, not aggregate counts alone.

Cache policy commit-study-os-uncontrolled; retained/reopened means lifetime, not
cold. No extra untimed warm-up Commit. Report read bytes/cache/route/sequence.

## Frozen qualification and reporting

Before fixes/timing, commit `commit-stage2-hardening-contract.md` and evidence
pointer. Freeze reproductions, measurements, noninferiority/absolute gates,
test/profile limitations, cells/order/repetitions and invalid-attempt rules.

Core fresh paired plain cells: nochange, retained K1 marker/repeat/revert/noedit,
k10,k100,fuse-posix; n=3 pairs/cell C1,T1;T2,C2;C3,T3. Keep matching harness bytes.
Separate diagnostic pairs only where needed for budget/decompression/fallback
attribution. Never subtract diagnostic overhead or pool it with plain rows.
Retain all attempts; at most one complete affected-pair replacement for demonstrated
infrastructure invalidity, never slow timing. Raw evidence is append-only.

Fix analyzer chain totals in the NEW copy: previous analyze.py sums all edits or
exec writes then adds only Commit#1. Report each edit+matchingCommit or full chain
including everyCommit; compute per-sample sums before aggregates. Add a small
regression check with known intervals. Preserve archived wrong aggregates and
explain that standalone Stage2 Commit measurements were unaffected.

Qualify affected shared callers from actual call graph (content tree/store packed
reader/workspace/FUSE/SDK), including all registered Init tiers as applicable,
retained/reopened Commit, sparse pooled DELTA/eviction, mixed batch storage,
structural tree mutations, corruption, low-budget/deep fallback and rollback.
Freeze exact selected family/case IDs from current registry; reuse unaffected
#104 evidence instead of blanket reruns. Do not claim release qualification from
only the five existing performance cells. All timing has independent proof:
complete changed bytes, stated unchanged samples, head/result, reopened root/content,
physical receipts, clean workspace/container/spool, no swaps/OOM.

Focused debug tests are supported; existing release-test cfg(debug_assertions)
problem must be recorded, not silently bypassed. Qualified release binaries still
run public performance/proof. Test new regression fails on control where feasible
and passes on hardening; distinguish analytically suspected from reproduced.

Output `commit-stage2-hardening-results.md`: findings/reproductions/fixes, exact
source/product identities, all raw timing/paired differences and per-sample target
outcomes, CPU/peaks/charged bounds, fallback work counts, correctness/affected-case
coverage, custody disclosures and remaining blockers. Update #111 and link #115/
#108/#109/#110/#106/#102/#104/#100/#107. Commit only intended fixes/tests/docs;
report whether hardening is qualified or remains blocked. No release/tag/deploy.

Stop after hardening and qualification. Next queued work, NOT bundled here:
quadratic spill-merge removal; fresh edit-stage attribution (goal>=90% disjoint
wall accounted before choosing a latency target); triangular publication and
reopened-history scaling. Stage2 does not close the overall no-quadratic objective.

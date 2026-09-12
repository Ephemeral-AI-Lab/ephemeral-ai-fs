# v0.1.6 implementation and qualification handoff

Complete GitHub issue https://github.com/Ephemeral-AI-Lab/layerfs/issues/122 in
the repository `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`.

Implement the entire v0.1.6 benchmark roadmap, its shared infrastructure and
independent verifiers. Obtain successful runs for every declared case, fix
measured problems through their shared root causes, and post the actual numbers,
failures, fixes and evidence to issue #122. Continue through implementation and
runtime qualification; a plan, scaffold, static check or one passing family is
not completion. You are authorized to post progress and result comments to #122.
Do not create a release or tag as part of this task.

## Start with the actual roadmap and current product

Read all files in `docs/roadmap/0.1/0.1.6/`, especially `cases.json`,
`benchmark-families.md`, `fixtures.md`, `workloads.md`, and
`execution-and-verification.md`. Also read `benchmark/AGENTS.md`,
`docs/general/benchmark_rules.md`, current `benchmark/fs-bench-pro/QUICKSTART.md`
and the existing implementations before writing code.

The roadmap currently exists in this checkout as local uncommitted files. Do
not lose it by starting from an older clean worktree. Preserve it, run
`python3 docs/roadmap/0.1/0.1.6/check_plan.py`, then make a scoped specification
commit before benchmark implementation or sampling. Bind all six families to
#122 and record the exact specification commit there. Do not include unrelated
changes, disturb another agent's release work, or overwrite existing evidence.
Source-arm revisions and the reference-machine fingerprint must be frozen before
paired collection. Version any necessary contract correction prospectively.

## Required implementation scope

Implement all **33 regular cases** in these six families:

- `dedup_branch_history`: six additions.
- `file_size_transition`: seven new cases.
- `mixed_load_bearing`: four additions.
- `multi_workspace_development`: four new cases.
- `branch_development`: six new cases.
- `historical_access`: six additions.

Use exact IDs and configurations from `cases.json`; do not duplicate inherited
small-file history cases or silently replace existing family definitions.
Implement selection/registry, preparation, public operation orchestration,
multiple live sessions, timers/resources, cancellation/cleanup, receipts,
independent oracles and result reporting by extending the existing harness.
No parallel benchmark framework, per-case collector or product scenario-ID path.

The twelve load-bearing cases use:

- L100: cap5,000 non-directory names/100,000,000 logical path bytes; initial
  4,984 files/99,934,464 bytes.
- L500: cap30,000 names/500,000,000 bytes; initial29,984 files/499,934,464 bytes.
- Exact distributions,52/302 data directories and62/312 total directories,
  16-name/64KiB headroom, threshold files and content cohorts from fixtures.md.
- Sequential:10/100 new commits. Concurrent:two branches/workspaces with10/100
  local commits each,20/200 total. Historical forks:trunk10, fork children from
  trunk5,10/100 commits per child,30/210 total and ancestry15/105.
- Five-stage mixed cycles:bulk delete/create; small/large/boundary edits;
  populated directory moves and empty-directory churn; attributes/aliases/
  symlinks; recursive subtree replacement and atomic saves/open-inode lifetime.

Preserve exact operation counts, per-role recurrence, initial A/B/Z values,
hot/rotating cohorts and K10-as-prefix-of-K100. Every requested mutating commit
must be `Created`; count UpToDate/Busy/HeadMoved/presentation failures separately.
Verify real work, not dummy mutations inserted to force commit counts.

## Environment and runtime requirements

One macOS-host Store/SDK/coordinator/publication/spool; one Linux Docker
daemon/FUSE/helper container with **2 CPUs,2 GiB RAM,no swap,256 PIDs total**.
All concurrent workspaces share this resource budget and have distinct roots.
Report host resources separately. One active workspace lease per branch.

Use real public SDK and FUSE/POSIX operations. The SDK batch API is same-file
only: cross-file pairs use two ordered single-file calls, not an invented atomic
batch. Finish each stage helper and writable handles before Commit. Supported
positive attrs are chmod/mtime/truncate; do not claim xattr/chown/ACL persistence.
Assert inode relationships, not stable numeric inode values across remounts.

Regular perf and separate regular verify must each finish within **15 seconds
complete invocation wall**, with the12s worker deadline/3s cleanup reservation.
Include per-run validation/copy/readiness, every scheduled commit, receipts and
cleanup. Builds and pristine fixture preparation are explicit prerequisites;
never cache completed measured histories, live workspaces or verifier success.
Historical access explicitly consumes sealed producer artifacts and must not
silently auto-build them. Missing prerequisites are NOT_READY, never PASS.

No sleeps, time-filling loops, timed soak, reducing100 commits, weakening proof,
raising budgets after a miss, or moving failed regular cases into extended.
Use actual overlapping workers and record overlap; do not assume simultaneous
request submission proves parallel SQLite publication.

## Execute and qualify everything

Develop through small selected cases and meaningful correctness tests. Reuse
build/preparation caches. Serialize resource-sensitive builds/perf/verifiers
under the existing lock, including coordination with other agents. Parallel
code/review work is fine with disjoint ownership; do not parallelize independent
performance samples or run a verifier alongside perf.

First obtain a successful selected perf and independent verify for **every one
of33 regular cases**. Then complete the roadmap's seed1/2/3 qualification and
matched control/candidate schedule on the exact final source, using identical
harness/fixture/schedule across arms. Record all applicable cardinalities and
run affected inherited regressions. A selected diagnostic pass alone is not
admission. If a baseline cannot support identical semantics, state the reason
and report those results as unpaired; never fabricate a speedup.

This task also explicitly requests all three extended cases:

1. `v016-mixed-exhaustive-100mb-5000-k100-v1`: verify all101 states/bytes;120s cap.
2. `v016-mixed-exhaustive-500mb-30000-k100-v1`: same at L500;300s cap.
3. `v016-workspace-four-100mb-5000-k100-v1`: four live workspaces,100 commits each;
   perf and independent verify,60s cap each.

Keep extensions suppressed in regular defaults but explicitly execute them for
this issue. They are finite work, not durations to fill. Verify-only cases have
perf N/A. Preserve/seal producer artifacts needed by historical readers and
exhaustive proofs before sample cleanup. Keep artifact preparation/reuse honest
and declared; no silent rebuild of historical evidence.

Regular load verification must check every parent/head and affected path,
metadata/link/inode semantics, full changed small-file bytes, declared large
changed ranges/witnesses and initial/final namespace inventories. Clearly label
the affected-set scope; only explicit exhaustive proofs claim every byte/state.
Keep benchmark oracles/digests/reopens out of performance distributions.

## Report actual evidence to #122

Post an initial implementation checklist/specification commit, substantive
progress including failures and root causes, and final per-case tables with a
machine-readable matrix. For every case/seed/arm/mode report:

- Requested/observed file/byte caps, branch/workspace counts, local/total commit
  counts, ancestry, actual operation counters and concurrency overlap.
- Complete command wall, preparation/workload/cleanup, edit/Commit/fork timing;
  raw units, sample count, median/min–max and maximum Commit latency.
- Logical/physical bytes, Store growth, dedup/reuse, spool high water; host and
  container CPU/memory/I/O separately, with unavailable fields and precision.
- Separate correctness/timing/resource/cleanup/custody/coverage outcomes.
- Exact source/product/harness/fixture/oracle/image seals, raw evidence paths,
  manifests and reproducible commands. Link immutable GitHub artifacts when
  available; identify local-only artifacts honestly rather than inventing URLs.

Retain every failure/timeout and valid sample; append fixes and later results
without rewriting earlier evidence. No cherry-picking, baseline asymmetry,
unsupported zero counters, hidden omissions or success-shaped summaries.
Successful completion means all applicable required cases and gates pass on the
final candidate. If a genuine external blocker remains after available work is
exhausted, post exact failed/missing rows, actual numbers, root cause and needed
action, and leave the issue open. Do not claim completion or wait for routine
permission merely because a case needs sustained debugging.

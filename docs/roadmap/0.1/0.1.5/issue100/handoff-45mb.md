# Handoff prompt: bring the ten-snapshot Store toward 45 MB

> Draft for owner review. The scope-revision section below proposes permission
> for bounded base/depth changes that were not authorized in the original issue.
> Issuing this prompt as written authorizes that revision. Merely studying or
> storing this draft does not change the existing product contract.

Continue GitHub issue #100:
https://github.com/Ephemeral-AI-Lab/layerfs/issues/100

Implement and measure a materially more storage-efficient v0.1.5 on the existing
**ten-snapshot DeepSeek smoke**, aiming for **45,000,000 final allocated bytes**
(45 decimal MB), while preserving good public save, Commit and historical-read
speed. Continue through diagnosis, implementation, matched measurement, exact
same-Store verification, cleanup and reporting. Do not stop after planning,
instrumentation, an isolated codec result or a small improvement over 66 MB.

The owner selected approximately 45 MB because metadata/index overhead makes
exact Git parity unnecessarily restrictive. **45–46 MB is near-target**, not an
exact 45 MB achievement; report the exact allocation and tradeoffs. Do not silently
relax the target, round 46 MB down to 45 MB, or claim a release-admission PASS.

## 1. Start from the actual current implementation

Primary worktree:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100`
Branch: `codex/issue100-full157`
Known study/code head before this handoff document:
`e46d50e82f5dccda36091d15a5896cb7649c06dd`.

The product remains the v0.1.5 SmallContent implementation plus the tested
upper-range exact-CAS fix from `7ca59e244`. The original
`codex/v015-small-content` worktree is an earlier starting point, not the latest
issue100 work. Do not start from v0.1.4 and discard v0.1.5.

Released control worktree:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-control`
Known head: `26a80a8efae63b7606c845f20c69aa3a1ccef292`.
Its product is unchanged release
`101fa273d815f3aaedb0e06ba0de7b0777d83def`; only the harness/docs differ.

Inspect current status, branches, worktrees and applicable AGENTS.md first.
Preserve other tasks' edits. Use an isolated `codex/` worktree when needed,
carrying the relevant current implementation and reviewed docs. The untracked
`target` symlink intentionally reuses the existing cache at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-release-v014/target`.

All relative paths below are relative to the primary worktree. Read:

- Issue #100 and applicable AGENTS.md, especially `benchmark/AGENTS.md`.
- `docs/general/benchmark_rules.md`.
- `docs/roadmap/0.1/0.1.5/README.md`, `workflow.md`, `spec.md`,
  `implementation_plan.md`, and `implementation-notes.md`.
- `docs/roadmap/0.1/0.1.5/past_mistake.md`, `benchmark_success.md`,
  `smoke-report.md`, and `metadata-optimization.md`.
- `docs/roadmap/0.1/0.1.5/issue100/ten-snapshot-contract.md`,
  `ten-snapshot-baselines.md`, and `git-gap-directions.md`.
- The three supporting studies in that directory: `git-algorithm-study.md`,
  `layerfs-gap-study.md`, and `git-gap-budget-review.md`.
- The current full157 contract and released v0.1.4 benchmark closeout before
  final full157 confirmation. Historical settings are not current instructions.

## 2. Freeze the scorecard; reuse all three valid baselines

Case: `deepseek-ten`. Scenario: `deepseek-ten-spread-v1`.
Retain full157 snapshot indices exactly:
**1, 18, 36, 53, 70, 88, 105, 122, 140, 157**, in that order.
These are ten complete snapshots; skipped checkpoints are not replayed.
They are not ten consecutive upstream commits and not ten full157 runs.

Source fixture:
`/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data`
Manifest SHA256:
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`
Source tip: `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`.
Prepared ten-snapshot cache:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`.

Baseline evidence:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence`.
It contains `fixture.json`, `git/`, `v014/`, `v015/`, saved binaries/identities,
image inspections, dirty patches, exact command receipts and evidence manifests.
Read those artifacts instead of recollecting unchanged controls.

| Baseline | Final allocated bytes | Allocated growth |
|---|---:|---:|
| Git | 38,223,872 | 38,215,680 |
| Released v0.1.4 | 67,145,728 | 67,076,096 |
| Existing v0.1.5 | 66,105,344 | 66,035,712 |

Initial allocation is **8,192 bytes for Git / 69,632 bytes for LayerFS**, not MB.
All three arms verified ten states, 58,860 path states and 327,885,165 logical
bytes. Both LayerFS arms produced 10 Created / 0 UpToDate and passed same-Store
verification and cleanup. Keep actual outcomes; never manufacture Created counts.

Existing v0.1.5 public medians: save/Exec 2.982280354 s, Commit 0.485983625 s,
paired save+Commit 3.468263979 s. Performance/verification case walls:
56.145798708 / 26.492623625 s. Warm complete performance+verification commands
total about 89.417 s, excluding builds. Recorded performance host lifetime peak
RSS: 120,406,016 B. Raw min/max, sums, per-state values and resource scopes are
in the baseline JSON/CSV. Ten observations are dependent history steps.

Git 2.47.1 retains exactly these ten trees, not the upstream 15,632 commits.
Its packing is compression 6/window 10/depth 50/threads 2. Construction and later
packing costs are separate; do not claim their timers match LayerFS save latency.
Git's narrower metadata scope remains explicit.

Reuse a baseline only after checking its complete source/harness/fixture/timing/
environment applicability. If a material harness or measurement change invalidates
it, explain why and collect the affected matched control once. No forged seals.

## 3. Use the measured gap, not speculative compression percentages

The existing candidate's allocation is:

- File-content packs: **54,245,616 B**, versus Git blob entries **34,306,253 B**.
- Metadata/legacy packs: **6,559,114 B**, versus Git trees/commits **1,566,186 B**.
- Remaining indexes/structures/allocation: **5,300,614 B**, versus **2,351,433 B**.
- Total gap to Git: **27,881,472 B**; required net saving to 45 MB: **21,105,344 B**.

Matching Git content while retaining current non-content allocation gives
**46,165,981 B**. Reaching 45 MB then needs another **1,165,981 B** of saving.
This is a target budget, not proof that an online encoder can match Git.
Prioritize content; do not begin with a wholesale metadata/index redesign.

SmallContent FULL/DELTA frames are **30,191,026 / 19,361,953 B**. Deleting DELTA
frames hypothetically still leaves 46,743,391 B with other costs fixed. More DELTA
records alone do not prove lower allocation. Count physical FULL bases once.

Confirmed Git differences:

- Git sorts candidates partly by descending size and can encode old smaller
  versions against newer larger bases; it is not simply storing commit patches.
- **18,604 / 23,597 Git blob DELTAs require later-snapshot base closure**. This is
  whole-chain availability, not necessarily direct-base first appearance.
- Actual Git maximum blob/tree depths are 23/7. Do not copy these as our limits.
- Of our FULL frames, **16,494,159 B** map to Git DELTAs requiring future closure;
  same/earlier closure covers only **2,519,593 B**. Git target-entry differences
  exclude the cost of establishing alternate base representations.
- Objects both systems store FULL total 11,177,274 B in our frames versus
  10,605,176 B in Git entries. A codec swap is not the first evidenced repair.

Concrete real diagnostic:
`scripts/snapshots/translation-prompt-v4/request-response.expected.json`
shrinks from **133,273 B** at smoke step 7/full157 index 105 to **129,991 B** at
step 8/index 122. Git uses the preceding FULL blob and stores a **5,492 B**
depth-one DELTA. LayerFS stores a **50,626 B FULL frame** because the predecessor
is CDC-backed and ineligible as a SmallContent FULL base. This is a representation-
boundary restriction, not an absent predecessor or a depth-only issue.

Reuse `git-pack-attribution.json`, `small-full-git-attribution.json`,
`online-candidate-example.json`, and `git-gap-summary.json`. The prior full157
149–169 MB estimate used unmeasured reduction assumptions; it is not a forecast
and must not be transferred to this smoke.

## 4. Proposed scope revision when this prompt is issued by the owner

For this optimization, the previous rule requiring every new SmallContent DELTA
to reference exactly one predecessor-derived **FULL SmallContent** is relaxed
only as follows:

- You may investigate and implement **bounded immediate-predecessor delta chains**,
  bounded reuse of an already-available similar candidate, and **bounded logical
  predecessor reuse across the SmallContent/CDC boundary**.
- Choose the smallest evidence-supported design. Do not implement all three
  automatically or run parameter/candidate-selection sweeps.
- Before encoding alternatives or measuring a changed candidate, write a concrete
  design amendment specifying eligible base roles, maximum edges, maximum total
  decoded closure bytes including the target, simultaneously live buffers,
  authentication ownership, fallback, retention, rollback and format fencing.
- Reuse existing codec and memory ownership. Keep the existing per-active small
  reconstruction allowance (2 MiB) and small encoding allowance (3 MiB), including
  actual simultaneous buffers and static codec storage. Depth is not a substitute
  for a decoded-byte/ownership cap. Never copy Git's depth 50 or use an unbounded
  cache, recursive history walk or hidden heap fallback.
- Preserve whole-file SmallContent canonical identity where possible. Any new
  persisted interpretation needs explicit version/capability fencing and supported
  old readers/nonpromoting opens. Do not silently reinterpret existing formats.
- A near-cutoff CDC predecessor is not already an eligible small FULL: merely
  changing its pointer would violate current grammar/bounds. Implement and verify
  the complete bounded reader/dependency path if that design is selected.

Everything else in the original product contract stays fixed:

- Nonempty new/changed regular files strictly below **131072 B** use whole-file
  SmallContent; empty representation stays compact.
- Files at/above that boundary retain CDC **8/16/32 KiB** and existing extent-tree
  known-edit locality. No cutoff, CDC, codec or page-size sweep.
- Keep pinned Zstandard, its specified compression/window/frame settings, normal
  durability/acknowledgement semantics, pack bounds and SQL cohort limits.
- New SQLite pages remain 4 KiB; retain supported old layouts.
- Preserve shared Init/Commit construction, authoritative live FUSE/SDK state,
  POSIX behavior, handles/links/modes/symlinks, publication outcomes and rollback.
- Preserve #95 Init authenticated comparison reuse, #98 Workspace SQL coalescing
  and staging handoff, and ordered spill read-ahead <=64 KiB.
- No new dependencies, codec replacement, cloud storage, general GC/repacker,
  reverse physical rewriting project or broad metadata rewrite. Do not append an
  alternative encoding and subtract unreclaimed original bytes from allocation.

This is authorization for the bounded design work above, not permission to weaken
correctness, hide maintenance outside the measurement window or claim Git parity.
For a change outside this revised scope, prepare the concrete proposal and identify
which constraint requires an owner decision. Do not repeatedly ask permission for
already-authorized routine implementation/build/diagnostic steps.

## 5. Fast execution loop

1. Verify baseline applicability and current source. Keep all controls immutable.
2. Use a **handful of actual fixture file families** to isolate base eligibility,
   accumulated differences and matcher cost. Reuse original bytes/oracles. Include
   the cutoff-crossing example and an expensive growing history when relevant.
   These are labelled diagnostics, not a synthetic benchmark family or full proof.
3. Measure complete FULL/base/DELTA cost and bounded reconstruction before choosing
   a design. Record rejected outcomes. Do not optimize one frame while hiding new
   FULL retention, duplicate records, metadata or index cost.
4. Fix the shared owner. Start with `PreparedAdmission::prepare_small` in
   `crates/layerfs-layerstack-store/src/objects/admission.rs` and `StoreDb::small_anchor`
   / SmallContent readers in `objects/read.rs`; trace all callers and handoffs.
   Use existing code/dependencies, and avoid per-object transport/transactions,
   repeated authentication/readbacks and long Store/live-state locks.
5. Leave and run the smallest focused regression checks for changed depth/base,
   integrity and size-boundary behavior. The old >64-KiB CAS defect is already
   fixed; rerun its focused check only if reader/comparison changes invalidate it.
   Do not run broad Cargo test, Clippy, doctest or unrelated qualification suites.
6. Build only affected artifacts, then run **deepseek-ten performance + exact
   same-Store verification + census/cleanup** for each substantive candidate that
   warrants the complete smoke. Do not run full157 during the exploratory loop.
7. Compare against both released v0.1.4 and the existing v0.1.5 baseline. Keep an
   optimization only when measured storage benefits justify speed/resource costs.
   Revert unhelpful attempts while retaining their patch, identities and evidence.
8. Once the ten-snapshot candidate is stable near the target, run final full157
   confirmation once with exact source identity and its historical verifier.
   Reuse the released full157 control only if provenance, harness, inputs,
   environment and boundaries demonstrably match; otherwise obtain one matched
   control. No unchanged reruns for a nicer number.

The existing full157 evidence is at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-evidence`:
released control **184,582,144 B**, initial candidate **201,371,648 B**; both
verified 157 Created outcomes and same-Store history with clean teardown.
Ten-state improvements do not erase that long-history regression. The 45 MB
objective belongs only to the ten-snapshot smoke, not full157.

## 6. Build, runner and custody requirements

Reuse the existing entrypoints:

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-storage-smoke-image
```

Candidate performance (substitute freshly sealed paths; use a NEW output):

```sh
python3 benchmark/fs-bench-pro/shared/runner.py \
  --storage-smoke deepseek-ten \
  --source-arm candidate \
  --image "$CANDIDATE_IMAGE" \
  --host-binary "$CANDIDATE_BINARY" \
  --fixtures /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs \
  --output "$NEW_RUN"
```

After successful performance, freeze allocation and Store identity, collect the
read-only census, then run the same command with `--storage-verify-run "$NEW_RUN"`
in place of `--output`. Update census support for any admitted physical-format
change without altering the frozen observations. Do not weaken source checks or
relabel a stale binary/image to avoid building.

MacOS owns Store/SQLite/coordinator/canonical publication/spool. Docker owns only
Linux daemon/live core/FUSE/workload. Preserve 2 CPUs/2 GiB/no swap/256 PIDs.
Serialize resource-sensitive work using the existing lock; runner commands own
it and must not be wrapped in a second acquisition.

Reuse shared Cargo/BuildKit caches and prepared inputs. No manual cargo clean,
fresh target directories, Docker pruning or repeated fixture rebuilding/transfers.
The shared Cargo target has previously reused stale cross-worktree timestamps:
if needed, invalidate only genuinely changed source files without changing their
bytes or forging seals. Diagnose actual build/setup/lock/transport owners instead
of inflating timeouts.

Keep smoke watchdogs: 600 s phase/preparation, 120 s operation/setup/cleanup;
builds 900 s with two Cargo workers. Retain full157's separate existing bounds.

Before candidate measurement, record a prospective speed/resource working criterion.
A suggested starting criterion is <=10% regression against existing v0.1.5 in
save/Commit/paired medians and public-call sums, performance/verification walls,
and recorded host lifetime RSS (with a separately declared 8-MiB absolute RSS
allowance). This is **proposed, not previously user-approved**. Report actual
tradeoffs and uncertainty regardless; correctness/resource/cleanup failures are
never tolerated. No fabricated tail-confidence or numerical release PASS.

Each candidate uses a fresh Store and the public native Init, ordinary Exec/FUSE
imports and public Commit. Retain all ten actual mappings. After observations
are frozen, check the measured Store digest, reopen the SAME Store in a new
coordinator, and verify complete bytes, paths, types, modes and symlinks against
original oracles. Verification lifecycle mutations stay outside measurements.
Require successful shutdown and container cleanup. No second rebuilt history.

## 7. Delegation and completion

The owner permits subagents for focused Git/source/measurement studies. Delegate
only concrete independent subtasks with explicit file ownership. Tell workers
they are not alone; preserve others' edits. Do not duplicate full scans, builds,
fixture preparation, resource-sensitive measurements or already-completed research.

Finish with:

- Accepted implementation and updated specification/format/ownership docs.
- Exact final allocated bytes and growth; 45-MB target/near-target status stated
  honestly, with comparisons to Git, released v0.1.4 and existing v0.1.5.
- Per-snapshot save/Commit/paired timings, historical verification time, CPU/RSS,
  container categories, staging/spool and separate build/setup/transfer/cleanup.
- Byte-reconciled FULL/DELTA/base, content/metadata/index/pack/page attribution.
- Exact source revisions, dirty patches, harness/fixture/binary/image seals,
  commands, artifact paths, same-Store verification and cleanup results.
- Retained failed/rejected attempts and reasons for necessary reruns/reversions.
- Final full157 confirmation after the short-loop design stabilizes, with its
  own matched control and no claim that 45 MB applies to 157 retained states.
- Honest unresolved tradeoffs, limitations and unrun qualification. If evidence
  does not support the target under the revised scope, report that result; do not
  present speculative percentages, offline frame totals or unreclaimed bytes as
  achieved product allocation.

Update issue #100 with the final evidence and outcome. Do not publish/tag a
release. Do not claim completion merely because a reader test passes, one DELTA
gets smaller, or the candidate narrowly beats the 66-MB baseline.

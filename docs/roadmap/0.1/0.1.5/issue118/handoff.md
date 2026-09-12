# Ready-to-use handoff prompt: finish issue #118

Work in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs` on main. Continue and fully
close the remaining work of https://github.com/Ephemeral-AI-Lab/layerfs/issues/118
through terminal acceptance. The owner said “issue18” in the final handoff
request; the active umbrella is #118. The previous agent intentionally stopped
after finishing the fsync optimization because the owner requested a quick
handoff to conserve quota. The umbrella is **not complete**.

Use subagents for concrete independent work; the owner explicitly requested
them. Aggressively simplify and speed the development infrastructure, but retain
all correctness, authentication, resource, identity and cleanup protections.
Use the common runner and existing cases. Do not stop after a stage, attribution
report or checklist while actionable required work remains. Do not create another
Codex task or automation. No release, tag, deployment or unrelated architecture.

## Start here, then continue execution

1. Read current #118 and the applicable current linked issues, benchmark/AGENTS.md,
   docs/general/benchmark_rules.md and benchmark/fs-bench-pro/QUICKSTART.md.
   Read this handoff, execution.md, fsync-results.md, scope.md and history-handoff.md
   in this directory. Older diagnosis sections explicitly precede newer results;
   do not revive their stale PENDING statuses or superseded cold target.
2. Inspect git status and current source. Product change `f8fa59fab` is committed
   and qualified. Later handoff/report commits are documentation only. Keep all
   earlier intended commits and unrelated work. Do not repeat source recovery,
   attribution or passing tests without a relevant new change.
3. Freeze one concise updated selection/arm plan for the remaining shared
   qualification, then execute it. Existing stage4-frozen-plan.json names old
   arms; reuse its registry-validated rows, not its obsolete candidate identity.
   Investigate only concrete failures/material regressions. Then finish ordinary
   full157 storage/history/access work. Only afterwards begin #116.

## Mandatory owner instructions and dispositions

- **No external-library patches.** Do not edit registry/vendor/dependency sources
  or patch SQLite/rusqlite/zstd/Rust/libc/OS. Current changes are LayerFS-owned;
  Cargo manifests and lockfile were preserved.
- Preserve 4 KiB Store pages, authentication, exact CAS, pack and DELTA policies.
  No quota increases, extra workers, automatic Commits or unbounded caches to
  conceal restrictive representations or redundant work.
- **Cold Init 2.7 s is owner-WAIVED.** The owner explicitly said improvement is
  needed but 2.7 s is not mandatory because newer features may cost time.
  Stage2 K10 50/31 ms targets also remain owner-WAIVED. No other gate is waived.
- Unrelated-history500 remains **<15 s**, tiny-create100 **<1 s**, and all eleven
  historical-access performance commands plus eleven separate proofs retain
  their **complete 15-second envelopes**. The latest full500 screen passes;
  `--collection-mode` itself never waives an explicit stronger target.
- Ordinary preservation rule: freeze n3 alternating pairs, median paired wall
  slowdown >max(15% control median,3 ms) AND at least two of three slow;
  CPU >max(15%,1 ms) with the same condition. Keep every attempt. No unchanged
  retiming to fish for a passing median. Distinguish engineering goals from gates.
- Preserve explicit WARNs: cold openat change had paired wall +1.039% and CPU
  -2.855%; retained as a CPU tradeoff, not a cold wall win. Active K100 Commit
  paired +0.311 ms was below the material rule. Prior unused_mut compiler warning
  is documented; latest fsync/mounted checks had none. Old failed attempts remain.

## Completed work: reuse it

| Area | Current evidence and result |
|---|---|
| #117 infrastructure | Shared incremental host target, locked Docker dependency cache, independent immutable binaries, guarded retention, fewer readiness/image calls. Focused checks, smoke and n3 A/A passed. Fixed actual stale Linux binaries caused by copied-source mtimes; verified fresh daemon/Fuse for the latest build. No further infra rewrite or pruning campaign is required. |
| Stage3 spill | Highest-tier growing-prefix merge repaired; real failures/retry, actual capacities/FD/disk bounds, default-budget component and reduced-budget Workspace proofs PASS. Default public K32000 remains deferred to #116 capacity repair. |
| 100000-file reservation failure | Fixed in a36c60891/440584938: PreparedObject 184→112 bytes using a bounded signature slab, and stale input-association ledger ownership corrected. Required 2106936 bytes had exceeded 2097152 by 9784. Original full public case now PASS 4.708 s plus separate proof; no budget increase. See physical-reservation-root-cause.md. |
| SDK edit preparation | Fresh attribution covered 99.9167% of named daemon edit phases. Removed unnecessary full prior-fact publication only on EDIT_BEGIN and optional SDK sibling/payload prefetch. Both-cache K100 n3 pairs, exact work counts, independent proofs and mounted tests PASS. Inactive edit median 796→219 ms; edit+Commit 993→387 ms. See edit-attribution.md. |
| Reopened history | Exact greedy retained metadata window reconstructed without replaying discarded payload, complete header validation and exclusive-open FK check retained. 22 focused checks PASS; K1000×33 actual public reopened sequence and separate proof PASS across 131072 retention boundary. See reopen-retention-explanation.md. Header/FK scans remain linear; no universal sublinear reconnect claim. |
| Cold Init | Completed nine frozen H/B/T samples, reconstructed exact historical product with current harness, four namespace tiers and five proofs. No verified recovery of old 3.420 s absolute time. CPU tradeoff and owner waiver explicit. See cold-namespace-results.md; do not rerun cold Init for fsync-only changes. |
| Latest fsync | f8fa59fab: bounded batch acknowledgements and idle reservation-tail reuse. 15 focused checks + independent review PASS. n3 unrelated100 medians 4.492→2.677 s; 8007→1503 backing exchanges; host CPU 2.165→1.641 s. Full unrelated500 screen PASS 13.791443406 s; exact 5000 writes/500MiB/6000fsync/500Commits. Two independent sampled proofs, four current mounted reliability proofs and two freshly linked real Docker SDK tests PASS. See fsync-results.md. |

The latest 500-round result is one complete candidate screen, not a new full500
n3 paired comparison. Decide any remaining exact-final comparative coverage
prospectively; do not present it as already collected. The focused optimization
is complete, and no additional fsync tuning is required just to chase margin.

## Remaining optimization and qualification

Three of 21 stage4 rows were previously collected on pre-fsync qualified-v2:
distributed500 and recurring500 improved and independently verified; unrelated500
failed 15 s then and is now repaired by the latest change. Keep those failures.
Reassess only affected final coverage for the new shared code and complete the
following **18 NOT_RUN selections**, whose exact families/input flags are in
stage4-frozen-plan.json:

```text
dedup-cross-file-unique-500
dedup-cross-file-identical-10
store-footprint-large-object-500m
store-footprint-unique-100000
directory-content-scan-500-mixed-v4
git-tool-500-mixed-v4
workspace-dense-rewrite-100-mixed-v4
workspace-dense-rewrite-500-mixed-v4
workspace-distributed-sdk-edit-500-mixed-v4
workspace-fixed-move-500-mixed-v4
agent-episodes-500
dedup-workspace-exact-10-compact-v2
dedup-cdc-overwrite-10
payload-random-read-10-compact-v2
overwrite-middle-4k-on-1mib-ops-1
insert-middle-4k-on-1mib-ops-1
delete-middle-4k-on-1mib-ops-1
tiny-bulk-create-100-mixed-v3
```

Deduplicate fixes across #108/#112/#114/#100/#107/#102/#110; do not restart the
old 198-case matrix, old ratio ladders, page-size study, or completed hypotheses.
SDK edit cases use `--repetition 1` and their required performance-row binding;
other ordinary selections use `--seed 1`. Git has its existing 300/310 s contract.
Each selected case needs its applicable independent proof; performance PASS alone
does not supply one. Small selected proof coverage is explicitly sampled.

**Full157 ordinary storage/history/access is entirely NOT_RUN on current source.**
Follow [history-handoff.md](history-handoff.md): immutable dataset and all157
original oracles plus matching packed Git157 control exist. Helpers are repaired
and focused-tested. Freeze the arm choice before running; the old full157 plan
still names pre-edit-control. Per arm: ordinary stride-1 performance → freeze
closed measured Store → exact physical census → all157 original oracles against
the same measured Store. Then matching Git proof, candidate ordinary-access
fixture, all11 access cases and11 separate proofs. Watch the existing per-step
progress; this is an explicit long workload, not a default quick test.

For #107, publish current ordinary allocated/apparent byte attribution and Git
distance before choosing a new mechanism and prospective quantitative goal.
The old compacted <66 MB goal is obsolete. A possible 1% improvement floor is a
future experiment proposal, not retroactive acceptance. Savings must arise from
ordinary Init/Commit; no compaction, VACUUM or post-workload repacking. Keep
construction allocation, frozen-copy allocation and verification growth separate.

## Stage6: #116, only after optimization tracks finish

The audit has **not started**. Read its current full instructions. First publish
source-linked capability/resource inventory, exact public errors, reset/ownership
scopes, bounded boundary reproductions and recommendations. Only then implement
proven redundant restriction removals or bounded representation repairs.

Primary work: >4096 counted pending edits to one file where actual budgets fit;
repair the route-specific changed-file ceiling rather than raising the 2 MiB
piece budget. The old ~5461-file observation is not a universal fixed limit.
Preserve cancellation, partial writes, retry, alias/mmap/writeback semantics and
one requested Commit. Cover large files, a genuinely wide single directory,
deep paths, large pending sets, long histories, and a representative pinned
package installation/update plus Commit/reopen. No existing npm failure is known.

Then execute default-budget public `namespace-100000 --sequence 32000` spill
qualification and its independent proof. K5000 cannot prove B15873 spilling.
The current Stage3 contract requires two full-capacity crossings plus a third
partial run. The common sequence runner already supports this workload after
the capacity fix; K×N>1000 has an explicitly selected 600 s work/614 s complete
verification policy, while ordinary small proofs retain 45/59 s. Do not silently
extend unrelated verification deadlines. Rerun only affected final checks after
#116 changes, then finish the terminal source/evidence/outcome table.

## Artifacts, execution and preservation

All current evidence is under:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/issue118/20260912/`.
New attempts need fresh directories; preserve failures and referenced raw rows.
Keep successful scratch Stores only as required for proof; never delete fixtures,
unique source, current history masters or owner-retained controls.

Current reusable treatment: `fsync-qualified/fs-benchmark-pro`, its adjacent
identity JSON and image.txt. Host SHA256:
`440ae2c4ce03741676a2845becb9b1661dde341079cc62faad0998847d33a4e5`.
Source seal: `8053ccba0350395727a01788c7b36de508d4202de27095028473c94be226cc74`.
Product seal: `c2d5022b8be288304e3ca092eadc543eae8bdf4089198309813157a38e38951b`.
Immutable image:
`sha256:a04cb1d9cd8c2696fdd3b8a4f7deffe2c55c2d38f0aced40be0c213b2227f0e7`.
Pre-fsync control: qualified-v2, host SHA256
`dc07a23f98a77c4520c2f91464359c899167d39240ef50d669d18eecc7e3750d`, image.txt
`layerfs-bench-infra:e162aecd2b351811`. Earlier Stage4 comparison uses
pre-edit-control; do not interchange those comparator roles silently.

Use existing build entrypoints only after a relevant source change:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
```

The shared release cache has been rebuilt for current source; no old historical
binary remains at its top-level executable. No historical-source reconstruction
or dependency copy is needed. Docs-only changes need no rebuild. Archive actual
binary/image hashes, not just labels. The prior falsely fresh image is retained
as rejected evidence and must not be promoted.

All resource work is serial under the runner-owned lock at
`Path(TMPDIR)/layerfs-infra-measurement.lock`. Wrappers acquire it themselves;
direct Cargo/integration tests acquire the same lock without nesting. macOS owns
Store/SQLite, SDK, canonical publication and spool. Docker owns daemon/FUSE and
workloads only, with 2 CPUs/2 GiB/no swap/256 PIDs. Check command budget, progress
and disk headroom before long runs. No duplicate collector or full target per
revision. Use compact aggregate output; do not dump thousands of receipt rows or
walk the quarantined 10000-file fixture when searching for a few scripts.

Preserve unrelated untracked `web/`, issue112/issue113 docs and
docs/roadmap/0.2/cloud-sqlite-vfs.md. The owner explicitly authorized the separate
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-transition-experiments` worktree in another
task; it is an exception to the original sole-worktree criterion. Do not delete,
reset or incorporate that work. It shares resource constraints. The temporary
historical source clone was already removed after archiving; source recovery and
historical evidence retirement need no repeat.

Terminal completion requires every applicable #118 row resolved with committed
source, exact final treatment, independent proof, bounded resources, explicit
FAIL/WARN/owner-WAIVED dispositions and the final linked-issue outcome table.
No hidden required NOT_RUN. Resume useful work immediately; ordinary reversible
implementation choices do not require renewed owner approval.

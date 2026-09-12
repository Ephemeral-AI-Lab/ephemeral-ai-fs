# #120 family report: `git_tool_workflow` (4) + `workspace_reliability` (28 proof-only)

Candidate `c55daf13…` @ `1ff1f2ddd`. Timer `pure_call_sum_ns` (receipt op count
= 1 workload). References: published v0.1.3 single samples (profile undeclared ⇒
context). All registered harness targets PASS; all cleanups PASS; no S0/S1.

## `git_tool_workflow` (3 fresh + 1 reused)

| Selection | Candidate | v0.1.3 ref | Ratio | Δ/op | Disposition | Proof | #104 ns |
|---|---:|---:|---:|---:|---|---|---:|
| git-tool-1-compact-v2 | 337.08 ms | 318.66 ms | 1.06× | +18.43 ms | **WARN** | PASS | 375,171,792 |
| git-tool-10-compact-v2 | 705.41 ms | 611.38 ms | 1.15× | +94.02 ms | **WARN** | PASS | 700,293,585 |
| git-tool-100-mixed-v4 | 2,838.25 ms | 1,879.18 ms | 1.51× | +959.07 ms | **WARN** | PASS | 2,715,104,126 |
| git-tool-500-mixed-v4 | 8,803.03 ms | 4,689.31 ms | 1.88× | +4,113.72 ms | **REUSED-FROM** #118 `remaining-shared` (fsync-qualified `440ae2c4…`; treatment identity recorded, pre-#107/#116) | PASS (reused) | 7,042,397,499 |

Diagnosis (reused row, aggregate +4.11 s > 1 s, mandatory): git-tool-500 runs a
full git workflow (clone-scale working tree, status/diff/add/commit cycles)
through FUSE on the v0.1.5 ordinary path; the reused fsync-qualified value
(8.80 s) sits between #104's uncompacted 7.04 s and twice the pre-authentication
v0.1.3 value, consistent with the per-iteration #116/#107 mechanism costs
measured in the unrelated-500 RCA plus authentication/pack work on the object
path. Single-sample, no n3 comparator ⇒ Tier-3 context.

Fresh git-tool-100 aggregate +959 ms is under the 1 s diagnosis threshold; its
ratio (1.51×) matches the git family's v0.1.5-vs-v0.1.3 shape.

## `workspace_reliability` (28 proof-only rows)

27/27 runnable proofs **PASS** on the frozen candidate (walls 2.1–9.0 s, all
inside the 60 s verification envelope; cleanups PASS):

`workspace-invalid-sdk-edit`, `workspace-invalid-namespace`,
`workspace-lease-lifecycle`, `workspace-open-writer-busy`,
`workspace-live-execution-busy`, `workspace-candidate-failure-retry`,
`workspace-admission-batch-failure-retry`,
`workspace-final-publication-failure-retry`,
`workspace-published-presentation-failure-smoke-v3`, `workspace-dirty-end-discard`,
`workspace-dirty-net-zero`, `workspace-short-spool-write`,
`workspace-deferred-nospace`, `workspace-workload-cancel`,
`workspace-dirty-runtime-disconnect`, `workspace-corrupt-descendant`,
`workspace-missing-descendant`, `workspace-parallel-read-write`,
`workspace-shared-path-contention`, `workspace-hardlink-alias`,
`workspace-symlink-semantics`, `workspace-open-rename-unlink`,
`workspace-metadata-chmod`, `workspace-metadata-mtime`,
`workspace-metadata-xattr`, `workspace-exec-500`,
`workspace-repeat-publication` (all `-compact-v2-proof` unless noted).

`workspace-sustained-600s-compact-v2-proof`: **NOT_RUN_OPTIONAL** — excluded by
the frozen campaign declaration (`long_test_exclusion`: "Optional long test
under current QUICKSTART; unexecuted, no endurance qualification. Other five
extended reliability members and all depth500 histories required."). Recorded
as an explicit gap in the final report, with the exception receipt at
`verification/workspace_reliability/workspace-sustained-600s-compact-v2-proof/exception.json`.

## Tally

All 198 registered performance selections are now dispositioned (181 fresh +
17 reused). Proof-only: 28 of 29 terminal (27 PASS + 1 NOT_RUN_OPTIONAL); the
29th (`dedup-cdc-boundaries-proof`) PASSed in the group-6 report. Remaining for
the final report: family 8 (historical access 11 + 11 reuse) and the campaign
deliverables.

Receipts: `benchmark-results/host-store/issue120/{performance,verification}/{git_tool_workflow,workspace_reliability}/…`

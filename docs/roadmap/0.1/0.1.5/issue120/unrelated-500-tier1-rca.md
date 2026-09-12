# #120 root cause: `dedup-history-unrelated-500-mixed-v2` Tier-1 miss (< 15 s gate)

**Status: root cause identified within the 60-minute timebox. Cell disposition:
`FAIL — unrepaired` (S2); owner decision requested on #120 (repair vs waive).**

## Observation

| Treatment | Product timer | Gate (< 15 s) | exec mean ×500 | commit mean ×500 |
|---|---:|---|---:|---:|
| `fsync-qualified` (`440ae2c4…`, commit `f8fa59fab`, pre-#107/#116) — the #106 repair's acceptance evidence | 13.791 s | PASS | 12.42 ms | 15.14 ms |
| `6693224e…` (+#116 bounded pending, pre-#107) — diagnostic attribution run, this issue | **15.772 s** | **MISS** | 14.91 ms | 16.60 ms |
| **candidate** `c55daf13…` (+#116 + #107) — campaign cell | **16.107 s** | **MISS** | 15.30 ms | 16.88 ms |

Campaign receipt:
`benchmark-results/host-store/issue120/performance/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/perf.jsonl`
(complete command 19.057 s, inside the 20 s budget; cleanup PASS;
`historical_product_target_status=TARGET_MISS`).
Diagnostic receipt (single sample, not a campaign cell):
`benchmark-results/host-store/issue120/diagnostics/unrelated-500-on-6693224e/perf.jsonl`
— archived `6693224e…` binary (byte-identical SHA256 to the #118 affected-rerun
producer) with its seal-consistent image `6ccad14c…`.

## Attribution

The workload is 500 iterations of exec + commit over unrelated histories. The
per-phase means decompose the +2.316 s candidate regression exactly:

- **#116 bounded pending representation: +1.98 s** (exec 12.42→14.91 ms,
  +2.49 ms/call; commit 15.14→16.60 ms, +1.46 ms/call). The compact splice form
  and its materialization/conversion add per-write work on this
  unrelated-history shape (many small pending edits per iteration, each
  round-tripped through the bounded form and converted at Commit).
- **#107 pack coalescing: +0.33 s** (exec +0.39 ms, commit +0.28 ms per call) —
  consistent with #107's measured +1.1 % on full157 construction.
- `3e308a8f2` contributes nothing (behavior-neutral, decided separately).

## Why this was not caught when #116 landed

The #118 affected-rerun after the #116 source change covered only the three
SDK-edit stage4 cells (`overwrite/insert/delete-middle-4k-on-1mib-ops-1`);
`dedup-history-unrelated-500-mixed-v2` was not re-checked, so the gate breach
introduced by #116 (13.79 → 15.77 s) was invisible until this campaign collected
the cell on the frozen candidate.

## Why it is not a minimal-fix candidate

The regressing mechanism is the owner-required #116 bounded pending
representation itself (and to a lesser extent the owner-accepted #107 pack
coalescing) — the same mechanisms that delivered the #116 capacity repair
(5,461 → ≈29,959 spliced files per workspace) and the #107 footprint reduction
(e.g. −10.2 MB allocated on `store-footprint-unique-100000`, measured fresh in
this campaign). "Optimizing" the exec-phase cost of the compact form is an
optimization campaign, not a claim-preserving minimal fix; removing the bounded
form to recover 2 s would reinstate the #116 ceiling the owner required to be
removed. No assertion, fsync, byte or operation is being dropped to obtain a
pass.

## Disposition and requested owner decision

Per the frozen contract the cell is **`FAIL — unrepaired`** (S2: Tier-1
registered gate miss; no percentage leniency). Options requiring an owner
decision on #120:

1. **Waive** the 15 s gate for this cell (exact target: unrelated-history500
   family timer < 15 s; measured 16.107 s, single sample, seed 1; reason: the
   gate was set on the pre-#106 pre-#116 treatment and the #116 mechanism was
   owner-required; residual risk: the worst-case unrelated-history tier runs
   ~7 % over its historical gate).
2. **Direct a repair**: a bounded optimization of the #116 compact-form exec
   path (candidate budget: recover ≥ 2 ms/exec without changing the bounded
   representation's semantics or capacity), with its own impact-set re-runs
   (this cell + the SDK-edit and workspace-locality cells sharing the pending
   path) plus full157 + census + access per the fix rules.

The campaign continues collecting (S2 does not stop collection); every other
`dedup_branch_history` and `store_footprint` cell in this family report is
terminal and unaffected by this finding.

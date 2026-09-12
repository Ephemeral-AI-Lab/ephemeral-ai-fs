# v0.1.5 waivers and explicit warning dispositions

> **Status:** LayerFS 0.1.5 release record. Referenced from
> [acceptance](acceptance.md); every entry here is a disposition of an existing
> measured target, never a re-measurement.

A waiver retires one named target for v0.1.5 only. It does not convert the
measured result into a pass, does not change any published number, does not
lower that target for any other case, and does not bind a future release. Every
measured value below is copied from the #120 campaign evidence.

## 1. `unrelated-history500 < 15 s` — owner-waived (new in this closure)

| Field | Value |
|---|---|
| Exact target | `dedup_branch_history/dedup-history-unrelated-500-mixed-v2`, family timer `pure_call_sum_ns`, `< 15 s` (registered Tier-1 gate) |
| Measured | **16.107 s** (16,106,808,892 ns); complete command 19.057 s, inside the 20 s budget; cleanup PASS; `historical_product_target_status=TARGET_MISS` |
| Sample and scope | one complete public sample, seed 1, repetition 1, on candidate `c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295` @ `1ff1f2ddd`; this cell only |
| Receipt | `benchmark-results/host-store/issue120/performance/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/perf.jsonl` |
| Reason | the breach is produced by owner-required/owner-accepted mechanisms, not by an unrepaired defect: the #116 bounded pending representation (+1.98 s; exec 12.42→14.91 ms, commit 15.14→16.60 ms per call) which the owner required to remove the pending-edit ceiling, plus the owner-accepted #107 pack coalescing (+0.33 s). A diagnostic run on the archived byte-identical #116-without-#107 binary measured 15.772 s, proving the breach predates #107 and was introduced by #116. The cell is **0.89× v0.1.3** (18.164 s) — faster than the released predecessor; the 15 s gate is a v0.1.4/0.1.5-era absolute target that only the pre-#116 fsync-qualified treatment met (13.791 s). Repair would require optimizing the owner-required compact-form exec path or removing its semantics, i.e. an optimization campaign rather than a claim-preserving fix. |
| Residual risk | the worst-case unrelated-history tier runs **~7 % over its historical gate** (16.107 s vs 15 s); any future treatment that adds per-iteration cost on this shape widens that margin |
| Decision authority | owner direction recorded on #121 (close v0.1.5 and write the disposition as a waiver); root cause and options in `docs/roadmap/0.1/0.1.5/issue120/unrelated-500-tier1-rca.md` |
| Supersedes | `docs/roadmap/0.1/0.1.5/issue120/final-report.md` §6 finalization condition for this cell; #120 closes on this waiver |

The cell keeps its **FAIL — unrepaired (S2)** disposition everywhere it is
reported; the waiver records that v0.1.5 ships with it.

## 2. Cold Init `<= 2.7 s` — owner-waived in #118, carried forward verbatim

Source record:
`benchmark-results/host-store/issue118/20260912/owner-cold-target-waiver.json`.

| Field | Value |
|---|---|
| Exact target | `init_namespace/namespace-100000` cold Init wall, absolute `<= 2.7 s` (registered Tier-1 gate) |
| Measured on the v0.1.5 candidate | **4.3975 s** (4,397,542,208 ns), `VERIFIED_COLD` (0 of 125,169 resident pages after eviction; metadata VERIFIED; acquisition 25.942 s reported separately and inside the 31.615 s complete envelope) |
| Sample and scope | one complete public sample under the owner-authorized issue111 cold-qualification allowances (n=1, 300 s product / 310 s command / 600 s setup); this target only |
| Decision authority | owner message of 2026-09-12: "we need to get better but does not mean 2.7 is a must because in v0.1.5 we introduced authentication, pack, delta encoding which might increases time" |
| Remaining acceptance from that waiver (still binding) | matched current-code improvement; no unexplained material regressions; fixed authentic-cold acquisition and the full original fixture; all correctness/authentication/resource/evidence gates |
| Scope limit | the waiver covers the 2.7 s absolute target only — it does **not** extend to any other tier, family or target |

## 3. Stage2 K10 — owner-waived in #118, carried forward

| Field | Value |
|---|---|
| Exact target | Stage2 K10 sequence target from the #118 work |
| Disposition | owner-**WAIVED**; recorded in `docs/roadmap/0.1/0.1.5/issue118/terminal-outcome.md` row 28 |
| Scope | that target only; no other sequence or family target is affected |

## 4. K32000 historical-family `TARGET_MISS` — recorded as WARN

| Field | Value |
|---|---|
| Exact target | `namespace-100000 --sequence 32000` against the harness's historical 15 s **family** target (31.765 s measured) |
| Disposition | **WARN, not hidden.** The harness itself labels that target "reporting-only … not a collection acceptance gate"; the route is a 32,000-edit sequence, not an Init-scale namespace run (K5,461 previously took 7.31 s ≈ 1.34 ms/edit), so the family target is not a valid acceptance gate for it. |
| What passed instead | COMPLETE in one Commit: 32,000 edits, 92,821 pieces, charge 2,048,000 (= 32,000 × 64) under the unchanged 2,097,152 B budget; independent verification PASS (32,000 changed files byte-compared pre/post reopen, 68.576 s inside the 600 s scaled allowance); cleanup PASS |
| Evidence | `benchmark-results/host-store/issue118/20260912/issue116-compact-k32000/perf.jsonl`, `…/issue116-compact-k32000-proof/verification.json` |

## 5. Git allocation-layout WARN — retained as a WARN

| Field | Value |
|---|---|
| Observation | Git157 read-only control: 157 checkpoints and mapping verified; live allocation 56,197,120 B vs the recorded 56,373,248 B, with unchanged apparent/pack bytes |
| Disposition | **WARN retained** (not waived, not relabeled); the ordinary Store's allocated 83,951,616 B versus that Git live figure remains an accepted residual of the ordinary-write route (#100) |
| Evidence | `benchmark-results/host-store/issue118/20260912/issue107-storage-full157/git157-proof/result.json`, `…/census.json` |

## 6. Other #118 accepted WARNs, unchanged

The full157 single-sample +1.1 % wall WARN accepted by the owner, the
cold-openat paired WARN, the active-K100 Commit +0.311 ms WARN and the
documented prior `unused_mut` warning remain as recorded in
`docs/roadmap/0.1/0.1.5/issue118/terminal-outcome.md` row 28. The #107
allocated-byte target was not met numerically (0.02 % allocated / 1.13 %
apparent) and was accepted by explicit owner decision
(`owner-107-acceptance.json`) — acceptance, not a re-based figure.

## 7. Not waived

- The 125 WARN cells and the 139/198 ≥15 %-versus-v0.1.3 campaign context are
  published as measured; they are not targets and have no waiver.
- The #108 per-commit/per-edit reduction targets and the #112 G0–G6
  optimizations remain **unmet and unclaimed**; the owner retired that
  optimization scope for v0.1.5 rather than waiving a measured gate.
- The #114 timer-to-command-window gap and `slab_send_blocked_ns` mechanism
  remain **unattributed**, not waived.
- Endurance (600 s sustained) remains **unqualified** (`NOT_RUN_OPTIONAL`), not
  waived.

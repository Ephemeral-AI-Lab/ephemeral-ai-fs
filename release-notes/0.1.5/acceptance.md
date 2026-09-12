# Owner acceptance of the v0.1.5 stopping point

> **Status:** LayerFS 0.1.5 release record.

On 2026-09-13 the owner directed the v0.1.5 closure: close the v0.1.5 issue set
(#102, #108, #112, #114, #120) with honest closing statements, prepare the
release documents, bump the version, tag `v0.1.5` and publish the source-only
GitHub release — **using the inherited #120 campaign, with zero benchmark
collection, zero proof runs and zero test-suite executions**. That direction is
the acceptance decision recorded here.

This accepts the implemented stopping point and explicitly authorizes closing
its related issues. It supersedes further optimization obligations for this
release; it does not change data, comparison eligibility, old thresholds, sample
counts, provenance or verification coverage. No historical report is rewritten,
and no measured FAIL, WARN or miss is relabeled.

## What is accepted

- The measured product `c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295`
  at commit `1ff1f2dddeb60493953311de316fa5bec4634a1a` (clean, == `origin/main` at
  freeze), with the #120 campaign as its evidence base.
- 198/198 registered performance selections terminal (182 fresh: 56 PASS /
  125 WARN / 1 FAIL; 16 REUSED-FROM) and 29/29 proof-only terminal (28 PASS +
  1 NOT_RUN_OPTIONAL), 182/182 fresh independent proofs PASS, every cleanup
  PASS, no S0/S1, and the native gate PASS (530 tests, 118 s).
- Owner acceptance of the v0.1.5 storage/performance tradeoff as measured:
  the optimization scope is retired exactly as 0.1.4 retired it. The
  125 WARNs, the 139-of-198 ≥15 %-versus-v0.1.3 cells and the +33.115 s
  campaign total are **published as context, not relabeled as passes**, and
  v0.1.5 does not claim the optimization work.
- **Owner decisions recorded as dispositions, not re-litigated:**
  1. `unrelated-history500 < 15 s` — **owner-waived** (measured 16.107 s,
     single sample, seed 1; the breach predates this campaign and is produced by
     the owner-required #116 bounded pending representation plus the
     owner-accepted #107 pack coalescing; worst-case residual risk ≈7 % over the
     historical gate). See [waivers](waivers.md) §1.
  2. Cold Init 2.7 s — **already owner-waived** in #118; carried forward
     verbatim, covering that target only. See [waivers](waivers.md) §2.
  3. #102's optimization half — **retired by owner direction** (0.1.4-style).
     See [waivers](waivers.md) §7.
  4. #108/#112/#114 unmet gates — **closed with their precise unmet gates
     restated**; any follow-up must be a **new** issue explicitly scoped to
     v0.1.6+, never a reopened v0.1.5 item.
- Every other #118 waiver and accepted WARN carried forward unchanged:
  Stage2 K10 (owner-waived), the K32000 historical-family `TARGET_MISS`
  (reporting-only target, recorded as WARN), the Git allocation-layout WARN
  (retained WARN), the full157 single-sample +1.1 % wall WARN accepted by the
  owner, the cold-openat paired WARN, the active-K100 Commit +0.311 ms WARN, and
  the #107 allocated-byte target accepted by explicit owner decision at 0.02 %
  allocated / 1.13 % apparent. See [waivers](waivers.md) §3–§6.
- The explicitly new-Store compatibility transition; no schema-10 promotion of
  existing Stores, no downgrade, no mixed-version runtime guarantee and no new
  durability promise.

## Related issue dispositions

| Issue | Disposition | Basis and remaining limits |
|---|---|---|
| #120 | **Completed: campaign terminal, finalization condition satisfied by the owner waiver** | 198/198 performance + 29/29 proof-only terminal; 182/182 fresh proofs PASS; no S0/S1; the single unmet Tier-1 gate is dispositioned by the recorded waiver, which was #120's stated finalization condition. The FAIL keeps its FAIL label. |
| #102 | **Completed: campaign half delivered; optimization half retired** | Every registered selection terminal on the frozen candidate (single-sample n=1 per the frozen registry); full157/access/K32000 reused from the final treatment with per-row provenance. The 139/198 ≥15 %, median 1.34× and +33.115 s context is published for future target selection, never as an acceptance gate. No optimization was landed. |
| #108 | **Completed: current per-commit/per-exec means and attribution delivered; reduction targets unmet and stated** | Fresh per-commit means 6.22 ms (distributed-500) and 4.03 ms (recurring-500) against the 3.4/2.8 ms targets; per-edit path 7.6–15.4 ms against the 2.4/1.4 ms v0.1.3-era means; #116 ≈ +2.49 ms/exec + 1.46 ms/commit and #107 ≈ +0.4 ms/phase, with transfer visible across the tier-500, git-tool and workspace families. No fix from #108's list landed; follow-up belongs to a new v0.1.6+ issue. |
| #112 | **Completed: fresh current-treatment values and mechanism attribution delivered; G0–G6 unmet and unclaimed** | Every G-group vehicle/confirmation cell carries a fresh single-sample value plus the #116/#107 attribution. G0 (harness attribution of the v0.1.3 ratio) and all G1–G6 optimizations were not executed or claimed; follow-up belongs to a new v0.1.6+ issue. |
| #114 | **Completed: fresh ordinary-path baseline delivered; both unknowns stated** | scattered-100 timer 308.33 ms, orchestration matches to 0.07 %, command window 760.6 ms ⇒ unattributed gap ≈452 ms ≈2.5× the timer (down from ≈1.8 s ≈7× in the #113-era arms); the tier curve is monotone on this treatment. The gap's composition and the `slab_send_blocked_ns` mechanism remain unattributed; the counter is absent from ordinary-path receipts. |

Independent future durability/cloud/S3 work (#52, #69, #82), separately scoped
FUSE/dependency follow-ups (#70, #51) and other milestones remain outside this
closure and untouched. Closure signifies accepted scope, not a fictitious
performance pass.

## Final release authorization

The owner authorized preparing, committing, tagging and publishing v0.1.5 with
the existing #120 results and no re-measurement. The
[benchmark closeout](benchmark-closeout.md) reports all families and cases, the
failures of numerical targets, waived targets, ineligible comparisons and
omissions without reclassification. The actual publication identifiers are
recorded by the GitHub tag/release and in
[release-evidence.json](release-evidence.json); earlier preparation records
remain historical.

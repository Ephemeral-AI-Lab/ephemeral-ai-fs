# Owner acceptance of the v0.1.4 stopping point

> **Status:** Release candidate for LayerFS 0.1.4.

On 2026-09-09, after receiving every benchmark-family/case result, the owner said:

> i am good with what it is, report honestly and prepare for v0.1.4 release and close all related issues

This accepts the implemented stopping point and explicitly authorizes closing
its related issues. It supersedes further optimization obligations for this
release; it does not change data, comparison eligibility, old thresholds, sample
counts, provenance or verification coverage. No historical report is rewritten.

## What is accepted

- The implementation qualified at `c48bb4903f456136ccbcdba78de38b9042d2755a`,
  with immutable evidence at `36a5d9da612211cf26f2a23370e9d59afdceb8e2`.
- 198 performance executions, 226 routine proofs, all supplemental smokes and
  full157 complete. One optional 600-second endurance proof was not run.
- All original severity labels, the three absolute target misses and four
  INELIGIBLE Git historical comparisons remain. The reporter remains INCOMPLETE.
- Unique/scattered publication and native encoding costs, mixed-working-set
  misses, higher historical-proof wall time and bounded-reuse memory tradeoffs.
- Full157 allocation of 184,582,144 B; the 134,221,004 B exploratory aspiration
  was not achieved. The supplemental storage control is not the v0.1.3 timing
  baseline. Full157 timing comparisons are historical single observations.
- The explicitly new-Store compatibility transition; no migration from released
  schema 5, no mixed-version runtime guarantee and no new durability promise.

## Related issue dispositions

| Issue | Disposition | Basis and remaining limits |
|---|---|---|
| #18 | Completed: accepted v0.1.4 scope | Storage implementation/evidence accepted; release preparation is reviewable, publication remains separate. |
| #71 | Completed: accepted scoped Init repair | Cumulative structural-buffer restart repaired and bounded nested frontier tested. Hard-link fallback remains intentional; the original broader redesign and exact 1.434 GB real-fixture target are not claimed complete. |
| #72 | Completed: retained-history experiment | Final 157-state performance/history proofs, census, accounting and cleanup completed. Historical experiments retain their own identities and limits. |
| #83 | Not planned: remaining optional follow-ups retired | Do not describe every R1–R6 proposal as implemented. Smaller 4 KiB pages are present; further codec/schema/layout changes and causal attribution/general bounds on allocation excess remain unimplemented or unresolved. |
| #87 | Completed: diagnosis/experimentation | Full157 storage decomposition and optimization exploration complete; 134,221,004 B aspiration unmet. Offline/theoretical layouts are not production results. |
| #93 | Not planned: remaining regression optimization accepted | Broad removal of all v0.1.3 regressions remains unmet; owner accepts the stopping point with original classifications retained. |
| #95 | Completed: focused repair and terminal delivery | Exact-byte bounded reuse, paired benefits, correctness and storage qualification completed; unique/scattered/mixed residual costs accepted. |

Independent future durability/cloud/S3 work (#52, #69, #82), and separately
scoped FUSE/dependency follow-ups (#70, #51), remain outside this closure.
Existing closed milestone issues remain closed. Closure signifies accepted
scope, not a fictitious performance pass or a published release.

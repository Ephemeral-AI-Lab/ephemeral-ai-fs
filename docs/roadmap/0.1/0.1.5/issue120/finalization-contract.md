# #120 finalization contract (frozen before collection)

This is the in-repo copy of the issue body for
[#120](https://github.com/Ephemeral-AI-Lab/layerfs/issues/120), committed so the
disposition, budget, measurement-validity and severity rules are fixed **before**
any campaign family is collected. The issue body is the operative text; this file
is the versioned record and is updated only by the same edits.

Frozen candidate at creation: commit `bb51d5b7c`, product binary
`b5f089ebcd6fa2fa939feb9bccc5300ca8ede798820fe8fd9aace8799fe4ec0b`. A fix produces a
new candidate identity and voids only its impact set.

# v0.1.5 finalization: full benchmark campaign, honest dispositions, no measurement cheating

## Goal

Run the complete registered benchmark campaign on **one frozen candidate**, one family at a time; diagnose and fix obvious regressions; report the result **family → per-test in detail**; and update the remaining open v0.1.5 issues honestly (#102, #108, #112, #114 — closed only on their own evidence, otherwise restated with the precise remaining gate). v0.1.5 is finalized only when every registered selection has a terminal disposition and no release-blocking defect is open.

Candidate at creation: commit `bb51d5b7c`, product binary `b5f089ebcd6fa2fa939feb9bccc5300ca8ede798820fe8fd9aace8799fe4ec0b`. The candidate is frozen (commit/binary/image/fixture seals recorded) **before** collection; any fix produces a new candidate identity.

## Budget rules — fast iteration is mandatory

- **Every performance selection: complete command ≤ 20 s** (product timer + container lifecycle + cleanup), excluding the one-time prepared-input validation, which is prepared once, identity-checked and reused.
- **Every independent verification selection: complete command ≤ 60 s.**
- Environment preparation must be **fast and reusable**: prepared fixtures/inputs are acquired once per campaign and reused with identity checks; no per-selection fixture rebuild, no per-selection image rebuild, no new target directory per revision.
- A registered selection that cannot fit these budgets is either **(a) already qualified on this exact treatment and REUSED** with its evidence cited (never re-collected), or **(b) escalated as an explicit expensive-qualification exception that needs an owner decision before it runs.**
- **Family by family:** run one family, then commit the receipts and post the family summary on this issue (`family → per-test`). Do not accumulate several families before reporting.
- **Never re-run a selection that already passed on this treatment.** A pass is accepted in good faith by citing its receipt. Only these are re-run: cells that missed or failed, and cells whose shared mechanism was changed by a fix landed during this campaign (impact set by call path, not by family).
- No re-running unchanged arms for a nicer median, no retrying only the slower arm, no dropping valid outliers, no moving work outside the timer, no enlarging a timeout instead of diagnosing.

## Measurement validity — the warm-cache lesson is non-negotiable

#109 reported a 39.7 % `namespace-100000` Init improvement with `fixture_cache_profile = reused-subsequent-sample-uncontrolled` and an untimed warm-up before each timed block; #110 re-measured the **byte-identical** binary on the authentic cold path and the absolute result was materially different, because input acquisition was never inside the timer. Rules that follow:

- **Cache contract per row:** every ledger row declares `fixture_cache_profile`, timer name, sample count, seed, and whether input acquisition is inside the reported envelope. A row without a cache contract is **diagnostic only, never qualifying**.
- **Cold claims require positively-detected cold acquisition**: fixture identity validation, fsync, read-only shared mmap, `MS_SYNC | MS_INVALIDATE`, residency recheck without faulting payload pages, all descriptors/mappings closed before product execution. Acquisition wall is reported **separately from and inside** the complete envelope. Failed/stale/nonzero-residency acquisition is `UNVERIFIED`; **no warm fallback**, and no cache label or process disk-read count may substitute for residency evidence.
- The residency detector must pass a **live self-check that it detects warm data** before it is allowed to certify cold.
- **Comparability:** control and candidate must share fixture, seed, profile, harness/workload source, image and topology. Any mismatch **voids the ratio** — report the numbers side by side, never subtracted.
- At least one **verified-cold member for every family carrying an absolute target** (Init tiers, history, access), not only namespace Init.
- Historical/warm/uncontrolled samples stay labelled diagnostic and are **never rewritten or promoted** retroactively.
- The owner's cold-Init 2.7 s waiver applies to **that target only**; it may not excuse a warm profile, another tier or another family.

## Disposition rules — what passes and what must be fixed

**Tier 1 — registered absolute gates.** Pass/fail as registered: unrelated-history500 < 15 s, tiny-create100 < 1 s, each of the 11 historical-access performance commands and 11 separate proofs within their complete 15 s envelope, and any harness `TARGET_MISS` with a registered target. A miss is **FAIL** unless repaired or explicitly owner-waived. No percentage leniency and no per-operation excuse applies here.

**Tier 2 — ordinary comparative cells.** A cell is a material regression only if all three hold:

1. per-operation wall delta > `max(15 % of the control per-operation median, 3 ms)` (CPU: `max(15 %, 1 ms)`), with `delta = (candidate − control) / operations` using the receipt's operation count;
2. ≥ 2/3 alternating pairs slower, where a comparator exists (n3);
3. worse than the recorded regression-ledger entry for that cell beyond the noise band, **or** per-operation delta above the hard cap (wall 10 ms/op, CPU 5 ms/op).

**Tier 3 — accepted with a recorded WARN.** Not worse than its ledger entry, or per-op delta under the floor: record ratio + absolute delta + per-op delta and proceed.

**Context, never a gate:** the campaign-wide distribution ("N of 197 cells ≥ 15 % slower", median ratio). It exists to choose optimization targets (#112), not to decide acceptance.

Cells with no v0.1.3 comparator are judged on absolute targets only. Any accepted cell whose aggregate delta exceeds ~1 s carries a **mandatory short diagnosis** (e.g. "1 ms/op × 3,000 = 3 s") so an aggregate never hides a systematic fixed cost.

## When something is really wrong — investigation and fix rules

**Severity classes**

| Class | Meaning | Rule |
|---|---|---|
| **S0** release-blocking correctness | wrong bytes, lost/duplicated data, broken CAS/authentication, unreadable Store, cross-workspace leakage, success reported without publication | **Stop collection.** Freeze command + artifact, no waiver offered, move this issue to BLOCKED until reproducer + fix + regression test exist |
| **S1** resource/custody/lifecycle | bound breached, leaked descriptor/spool/container, unclean cleanup, invalidated evidence identity | Stop the affected family, publish the blocker; escalate to S0 if authentication/publication is implicated |
| **S2** performance gate miss | Tier-1 miss or Tier-2 material regression | Keep collecting; mark that cell `FAIL — unrepaired` until repaired or owner-waived |
| **S3** permitted WARN | under the floor, or not worse than the ledger | Record and proceed |
| **H** harness/test defect | product fine, measurement/test wrong or flaky | If identity/validity is affected the cohort is **void and re-run**; if only test stability, record it |

**Investigation loop (bounded):** one meaningful reproducer from the failing receipt → inspect shared callers before changing anything → minimal fix → focused check → one selected public screen → stable qualification. Declare purpose, expected scale, budget, progress signal and stop condition before any long command. **Timebox: 60 minutes to a root cause or a written blocker.** The deliverable is a root-cause note or an explicit blocker statement — "probably noise" is not an artifact.

**Fix rules:** minimal and claim-preserving. Never remove assertions, authentication, fsyncs, files, bytes or operations to obtain a pass; never loosen a target, floor or quota (owner-only, in writing); add a regression test that fails without the fix; re-run **only the impact set** (selections sharing the changed mechanism, by call path) — plus full157 + census + access when a fix touches storage/admission or the workspace Commit path; fixes never overwrite an attempt.

**Waivers:** owner-only, written, specific — exact target, measured value, sample count/scope, why it is acceptable, residual risk accepted.

## Deliverables

1. **Family-by-family execution:** after each family, one commit pinning that family's receipts plus a comment on this issue with the family summary. Progress visible per family, not only at the end.
2. **Final report: family → per-test detail.** Every selection as one row: id, timer, candidate value, reference value **with its profile**, ratio, absolute delta, per-operation delta, operation count, disposition (`PASS` / `WARN` / `FAIL` / `OWNER-WAIVED` / `REUSED-FROM`), evidence path. Reused rows name the run they are reused from.
3. **Bug ledger:** severity · evidence path · reproducer · root cause · fix commit · impact-set re-run · disposition.
4. **Explicit gaps:** every registered selection appears; any NOT_RUN/failed/unstable cell is listed with its reason — no silent omissions.
5. **Honest issue updates:** #102 (campaign), #108 (per-commit/per-edit means and transfer), #112 (worst-case grouping G0–G5), #114 (scattered-100 attribution) each updated with what this campaign actually shows; closed only on their own evidence, otherwise restated with the precise remaining gate.
6. **Finalization condition:** v0.1.5 closes only when every registered selection has a terminal disposition and no S0/S1 is open.

## Reuse — do not re-collect these

- Ordinary full157 stride-1 on this treatment: construction complete command 763.218 s, same-Store verification against all 157 original oracles (157 states / 904,143 entries / 4,936,693,030 B), physical census and read-only Git157 control. Stride-3/10 remain unselected optional profiles.
- Historical access: 11 performance cases + 11 separate proofs, complete commands 2.48–2.88 s, all inside 15 s.
- Default-budget `namespace-100000 --sequence 32000` route and its independent verification (two B = 15,873 frontier crossings), K6000 boundary, the default-budget frontier proof.
- The 18 remaining single-sample selections and their 18 proofs on `fsync-qualified`, and the three affected SDK-edit selections, for every mechanism this campaign's fixes do not touch — cited, not re-run.

## Explicitly forbidden

Warm-cache substitution for a cold claim; target/floor/quota loosening; silently re-running passed cells; deleting or rewriting failed attempts; lowering accounting multipliers from process RSS; new benchmark families or collectors; compaction/VACUUM/post-workload repacking; fixture rewriting; external-library patches; extra workers; release/tag/deployment work inside this issue.

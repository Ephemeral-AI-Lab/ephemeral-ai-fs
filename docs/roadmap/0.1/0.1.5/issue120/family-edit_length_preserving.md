# #120 family report: `edit_length_preserving` (12) + `edit_canonical_chunk_count` (12)

Candidate: `c55daf13…` @ `1ff1f2ddd` (see the freeze record). Protocol: one
complete public sample per selection (`--repetition 1`, clone setup,
`--perf-fast --collection-mode`, 300/310/600 s allowances) + one independent
identity-pinned proof per fresh selection. Timer `edit_commit_ns` = one public
SDK range-edit call plus its Commit (ops = 1 per cell).

**Disposition convention (used for every ordinary comparative cell in this
campaign, stated once here):** reference = published v0.1.3 single samples
(`docs/roadmap/0.1/0.1.3/checkpoint-evidence/performance.csv`, profile
undeclared ⇒ ratios are context, never paired claims). Candidate ≤ reference,
or per-operation delta under the Tier-3 floor (3 ms wall) → **PASS** (values
still recorded). Beyond-floor delta that is not Tier-2 material — at n=1 no
alternating-pair comparator exists, so Tier-2 condition 2 cannot hold — →
**WARN** with ratio + absolute + per-op delta. Per-op delta above the hard cap
(10 ms/op) with an n3 comparator, or a Tier-2 three-condition hit → FAIL.
Registered harness targets are Tier-1: every cell here reports
`historical_product_target_status=PASS`.

## `edit_length_preserving` (12 rows)

| Selection | Candidate ns | v0.1.3 ref ns | Ratio | Δ/op | Disposition | Proof | #104 context ns |
|---|---:|---:|---:|---:|---|---|---:|
| overwrite-head-4k-on-1mib-ops-1 | 10,947,833 | 5,777,000 | 1.89× | +5.17 ms | **WARN** | PASS | 9,213,083 |
| overwrite-head-4k-on-10mib-ops-1 | 11,791,084 | 6,154,125 | 1.92× | +5.64 ms | **WARN** | PASS | 8,951,083 |
| overwrite-head-4k-on-100mib-ops-1 | 7,881,083 | 6,651,583 | 1.18× | +1.23 ms | PASS | PASS | 8,398,250 |
| overwrite-head-4k-on-500mib-ops-1 | 8,782,667 | 8,411,084 | 1.04× | +0.37 ms | PASS | PASS | 9,757,375 |
| overwrite-middle-4k-on-1mib-ops-1 | 8,538,292 | 6,138,666 | 1.39× | +2.40 ms | **REUSED-FROM** #118 affected-rerun (treatment `6693224e…`, commit `2dbc75ecb` + the #116 bounded-pending source; predates #107 pack coalescing — recorded, not hidden) | PASS (reused: `affected-rerun/overwrite-proof`) | 8,000,750 |
| overwrite-middle-4k-on-10mib-ops-1 | 8,174,584 | 5,924,958 | 1.38× | +2.25 ms | PASS | PASS | 9,905,083 |
| overwrite-middle-4k-on-100mib-ops-1 | 8,825,083 | 6,188,333 | 1.43× | +2.64 ms | PASS | PASS | 9,574,000 |
| overwrite-middle-4k-on-500mib-ops-1 | 11,182,750 | 7,174,000 | 1.56× | +4.01 ms | **WARN** | PASS | 16,381,958 |
| overwrite-tail-4k-on-1mib-ops-1 | 8,385,500 | 6,112,375 | 1.37× | +2.27 ms | PASS | PASS | 8,735,167 |
| overwrite-tail-4k-on-10mib-ops-1 | 8,565,333 | 7,265,084 | 1.18× | +1.30 ms | PASS | PASS | 8,350,208 |
| overwrite-tail-4k-on-100mib-ops-1 | 9,800,000 | 7,631,041 | 1.28× | +2.17 ms | PASS | PASS | 9,367,875 |
| overwrite-tail-4k-on-500mib-ops-1 | 11,799,250 | 9,377,875 | 1.26× | +2.42 ms | PASS | PASS | 10,685,958 |

## `edit_canonical_chunk_count` (12 rows)

| Selection | Candidate ns | v0.1.3 ref ns | Ratio | Δ/op | Disposition | Proof | #104 context ns |
|---|---:|---:|---:|---:|---|---|---:|
| chunk-count-preserve-on-1mib-ops-1 | 11,020,416 | 6,854,625 | 1.61× | +4.17 ms | **WARN** | PASS | 10,996,042 |
| chunk-count-preserve-on-10mib-ops-1 | 13,045,625 | 8,506,500 | 1.53× | +4.54 ms | **WARN** | PASS | 11,530,666 |
| chunk-count-preserve-on-100mib-ops-1 | 9,271,500 | 7,335,375 | 1.26× | +1.94 ms | PASS | PASS | 10,755,500 |
| chunk-count-preserve-on-500mib-ops-1 | 10,036,917 | 8,269,583 | 1.21× | +1.77 ms | PASS | PASS | 13,646,834 |
| chunk-count-increase-on-1mib-ops-1 | 8,425,875 | 7,852,416 | 1.07× | +0.57 ms | PASS | PASS | 8,988,167 |
| chunk-count-increase-on-10mib-ops-1 | 10,311,708 | 7,515,667 | 1.37× | +2.80 ms | PASS | PASS | 10,558,791 |
| chunk-count-increase-on-100mib-ops-1 | 9,544,667 | 7,869,709 | 1.21× | +1.67 ms | PASS | PASS | 11,197,167 |
| chunk-count-increase-on-500mib-ops-1 | 12,256,500 | 8,466,250 | 1.45× | +3.79 ms | **WARN** | PASS | 12,815,209 |
| chunk-count-decrease-on-1mib-ops-1 | 7,590,750 | 7,175,375 | 1.06× | +0.42 ms | PASS | PASS | 9,499,167 |
| chunk-count-decrease-on-10mib-ops-1 | 11,765,500 | 7,013,292 | 1.68× | +4.75 ms | **WARN** | PASS | 10,662,417 |
| chunk-count-decrease-on-100mib-ops-1 | 10,909,958 | 7,582,042 | 1.44× | +3.33 ms | **WARN** | PASS | 8,684,458 |
| chunk-count-decrease-on-500mib-ops-1 | 15,376,625 | 8,095,500 | 1.90× | +7.28 ms | **WARN** (under the 10 ms/op hard cap; no n3 comparator ⇒ Tier-3) | PASS | 11,186,541 |

## Notes

- 23/24 cells collected fresh on the candidate; 1 (`overwrite-middle-4k-on-1mib-ops-1`)
  reused per the frozen contract from the #118 affected-rerun with its treatment
  identity recorded. Its mechanism (SDK pending-edit path) was re-qualified
  there after the #116 change; the #107 pack-coalescing delta between that
  treatment and this candidate is inside its Commit publication phase and is
  recorded here rather than re-collected, per the contract's reuse rule.
- All 24 registered targets report PASS (`historical_product_target_status`);
  23/23 fresh proofs PASS (walls 1.9–4.5 s); cleanups PASS; no S0/S1 findings.
- WARN set (9 cells): per-op deltas 3.3–7.3 ms on a single edit+commit; every
  one is below the 10 ms/op hard cap and none has an n3 comparator at n=1, so
  none is Tier-2 material. Context: on the same cells the #104
  schema10-uncompacted candidate measured 8.0–16.4 ms — the frozen candidate is
  within or below that band on 22/24 cells (diagnostic context, not a claim).
- Aggregate deltas are all ≤ 7.3 ms — no mandatory >1 s diagnosis applies.

Receipts: `benchmark-results/host-store/issue120/{performance,verification}/{edit_length_preserving,edit_canonical_chunk_count}/…`
(immutable local evidence; per-receipt SHA256 recorded in the campaign ledger
maintained with this report series).

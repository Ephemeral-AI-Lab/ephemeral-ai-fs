# #120 family report: `workspace_change_locality` (16) + `tiny_file_churn` (20)

Candidate `c55daf13…` @ `1ff1f2ddd`. Timer `pure_call_sum_ns` (the registered
per-family summed public-call timer; the receipt's operation count is 1 workload
per sample, so Δ/op = whole-workload delta). References: published v0.1.3 single
samples (profile undeclared ⇒ context, never paired claims). Disposition
convention as stated in the `edit_length_preserving` report. Registered harness
targets: every fresh cell reports `historical_product_target_status=PASS`.

## `workspace_change_locality` (16 rows)

| Selection | Candidate ns | v0.1.3 ref ns | Ratio | Δ/op | Disposition | Proof | #104 context ns |
|---|---:|---:|---:|---:|---|---|---:|
| clean-commit-1-compact-v2 | 10,495,167 | 11,315,960 | 0.93× | −0.82 ms | PASS | PASS | 14,216,541 |
| clean-commit-10-compact-v2 | 11,871,750 | 10,872,708 | 1.09× | +1.00 ms | PASS | PASS | 13,975,292 |
| clean-commit-100-mixed-v4 | 15,469,750 | 11,559,208 | 1.34× | +3.91 ms | **WARN** | PASS | 12,660,666 |
| clean-commit-500-mixed-v4 | 17,795,083 | 11,866,166 | 1.50× | +5.93 ms | **WARN** | PASS | 14,018,541 |
| fixed-move-1-compact-v2 | 26,557,334 | 19,708,292 | 1.35× | +6.85 ms | **WARN** | PASS | 24,249,042 |
| fixed-move-10-compact-v2 | 28,828,876 | 21,320,084 | 1.35× | +7.51 ms | **WARN** | PASS | 27,545,459 |
| fixed-move-100-mixed-v4 | 35,948,959 | 28,352,083 | 1.27× | +7.60 ms | **WARN** | PASS | 41,921,501 |
| fixed-move-500-mixed-v4 | 47,745,459 | 27,589,207 | 1.73× | +20.16 ms | **REUSED-FROM** #118 `remaining-shared` (treatment `fsync-qualified` `440ae2c4…`, commit `f8fa59fab`; predates #107/#116) | PASS (reused) | 56,561,501 |
| distributed-sdk-edit-1-compact-v2 | 25,273,042 | 16,937,333 | 1.49× | +8.34 ms | **WARN** | PASS | 20,085,625 |
| distributed-sdk-edit-10-compact-v2 | 42,314,374 | 39,179,873 | 1.08× | +3.14 ms | **WARN** | PASS | 56,623,250 |
| distributed-sdk-edit-100-mixed-v4 | 213,086,041 | 336,399,086 | 0.63× | −123.31 ms | PASS | PASS | 475,559,492 |
| distributed-sdk-edit-500-mixed-v4 | 666,100,943 | 2,849,182,039 | 0.23× | −2,183.08 ms | **REUSED-FROM** #118 `remaining-shared` (fsync-qualified) | PASS (reused) | 4,130,687,706 |
| dense-rewrite-1-compact-v2 | 109,375,707 | 84,118,125 | 1.30× | +25.26 ms | **WARN** | PASS | 105,865,166 |
| dense-rewrite-10-compact-v2 | 583,170,375 | 316,616,500 | 1.84× | +266.55 ms | **WARN** | PASS | 585,343,916 |
| dense-rewrite-100-mixed-v4 | 2,894,054,086 | 1,497,618,208 | 1.93× | +1,396.44 ms | **REUSED-FROM** #118 `remaining-shared` (fsync-qualified) | PASS (reused) | 2,667,615,916 |
| dense-rewrite-500-mixed-v4 | 10,499,040,917 | 5,688,591,958 | 1.85× | +4,810.45 ms | **REUSED-FROM** #118 `remaining-shared` (fsync-qualified) | PASS (reused) | 9,331,262,667 |

## `tiny_file_churn` (20 rows)

| Selection | Candidate ns | v0.1.3 ref ns | Ratio | Δ/op | Disposition | Proof | #104 context ns |
|---|---:|---:|---:|---:|---|---|---:|
| tiny-create-1-compact-v2 | 23,261,583 | 19,053,709 | 1.22× | +4.21 ms | **WARN** | PASS | 27,430,457 |
| tiny-create-10-compact-v2 | 30,161,376 | 26,711,667 | 1.13× | +3.45 ms | **WARN** | PASS | 33,293,583 |
| tiny-create-100-mixed-v4 | 78,975,251 | 53,616,291 | 1.47× | +25.36 ms | **WARN**; **Tier-1 gate `tiny-create100 < 1 s` PASS (79 ms)** | PASS | 75,412,250 |
| tiny-create-500-mixed-v4 | 242,659,707 | 201,739,625 | 1.20× | +40.92 ms | **WARN** | PASS | 242,208,792 |
| tiny-stat-1-compact-v2 | 21,807,583 | 15,833,917 | 1.38× | +5.97 ms | **WARN** | PASS | 18,303,166 |
| tiny-stat-10-compact-v2 | 28,626,292 | 21,194,250 | 1.35× | +7.43 ms | **WARN** | PASS | 21,341,709 |
| tiny-stat-100-mixed-v4 | 45,704,626 | 37,810,208 | 1.21× | +7.89 ms | **WARN** | PASS | 42,450,917 |
| tiny-stat-500-mixed-v4 | 69,836,625 | 55,349,375 | 1.26× | +14.49 ms | **WARN** | PASS | 67,105,458 |
| tiny-unlink-1-compact-v2 | 19,562,543 | 17,634,874 | 1.11× | +1.93 ms | PASS | PASS | 24,219,041 |
| tiny-unlink-10-compact-v2 | 28,359,917 | 25,205,291 | 1.13× | +3.16 ms | **WARN** | PASS | 31,726,625 |
| tiny-unlink-100-mixed-v4 | 63,784,084 | 50,918,082 | 1.25× | +12.87 ms | **WARN** | PASS | 76,475,582 |
| tiny-unlink-500-mixed-v4 | 119,579,792 | 114,278,709 | 1.05× | +5.30 ms | **WARN** | PASS | 154,616,999 |
| tiny-bulk-create-1-compact-v2 | 97,120,710 | 96,728,291 | 1.00× | +0.39 ms | PASS | PASS | 104,294,166 |
| tiny-bulk-create-10-compact-v2 | 316,833,541 | 295,897,791 | 1.07× | +20.94 ms | **WARN** | PASS | 372,819,459 |
| tiny-bulk-create-100-mixed-v3 | 983,329,501 | 989,887,791 | 0.99× | −6.56 ms | **REUSED-FROM** #118 `remaining-shared` (fsync-qualified); its registered `<1 s` gate PASS (0.983 s) | PASS (reused) | 994,845,248 |
| tiny-bulk-create-500-mixed-v3 | 5,732,993,668 | 5,054,054,042 | 1.13× | +678.94 ms | **WARN** | PASS | 4,734,482,291 |
| tiny-bulk-delete-1-compact-v2 | 110,066,333 | 93,346,584 | 1.18× | +16.72 ms | **WARN** | PASS | 107,374,084 |
| tiny-bulk-delete-10-compact-v2 | 209,748,123 | 168,254,959 | 1.25× | +41.49 ms | **WARN** | PASS | 210,378,417 |
| tiny-bulk-delete-100-mixed-v3 | 306,845,124 | 259,278,875 | 1.18× | +47.57 ms | **WARN** | PASS | 298,072,209 |
| tiny-bulk-delete-500-mixed-v3 | 1,185,336,667 | 933,000,457 | 1.27× | +252.34 ms | **WARN** | PASS | 1,072,292,585 |

## Diagnoses (aggregate deltas > ~1 s, mandatory)

- `dense-rewrite-100` (reused, +1.396 s) and `dense-rewrite-500` (reused,
  +4.810 s): the dense-rewrite workload rewrites whole files; its v0.1.3→v0.1.5
  cost is the batched-fsync write path plus authenticated pack admission. The
  reused fsync-qualified values (2.894 s / 10.499 s) sit between #104's
  uncompacted schema10 values (2.668 s / 9.331 s) and are the #118-qualified
  single samples; the candidate's own fresh dense-rewrite cells (1: +25 ms,
  10: +267 ms) show the same shape at smaller scale. No n3 comparator exists;
  recorded WARN-tier context per the frozen rule.
- No fresh cell exceeds 1 s aggregate delta (max +678.94 ms on
  tiny-bulk-create-500, ratio 1.13).

## Notes

- 31/36 cells collected fresh (12 workspace + 19 tiny); 5 reused per the frozen
  contract (4 workspace + tiny-bulk-create-100) with treatment identities
  recorded (all `fsync-qualified`, pre-#107/#116 — stated, not hidden).
- 31/31 fresh proofs PASS (walls 2.1–7.8 s); all cleanups PASS; no S0/S1.
- `distributed-sdk-edit-100/500` are materially **faster** than v0.1.3
  (0.63×/0.23×) and than #104 — the bounded pending representation (#116)
  shows its benefit exactly on the distributed-SDK-edit axis.
- WARN set (fresh): 24 cells with workload deltas 3.1–679 ms, all single-sample
  without an n3 comparator ⇒ Tier-3, none above the hard cap per the
  receipt's operation count.

Receipts: `benchmark-results/host-store/issue120/{performance,verification}/{workspace_change_locality,tiny_file_churn}/…`

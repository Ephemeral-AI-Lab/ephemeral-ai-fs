# #120 family report: `dedup_branch_history` (20) + `store_footprint` (6)

Candidate `c55daf13…` @ `1ff1f2ddd`. Timer `pure_call_sum_ns` /
`product_call_sum_ns` (receipt op count = 1 workload per sample). References:
published v0.1.3 single samples (profile undeclared ⇒ context). All registered
harness targets PASS except the one Tier-1 miss below. All cleanups PASS.

## `dedup_branch_history` (20 rows, all fresh)

| Selection | Candidate ns | v0.1.3 ref ns | Ratio | Δ/op | Disposition | Proof | #104 ns |
|---|---:|---:|---:|---:|---|---|---:|
| distributed-1 | 17,135,875 | 22,163,084 | 0.77× | −5.03 ms | PASS | PASS | 26,895,375 |
| distributed-10 | 69,572,246 | 82,265,877 | 0.85× | −12.69 ms | PASS | PASS | 99,183,459 |
| distributed-100 | 735,401,993 | 586,404,579 | 1.25× | +149.00 ms | **WARN** | PASS | 873,929,251 |
| distributed-500 | 4,303,859,826 | 2,925,218,405 | 1.47× | +1,378.64 ms | **WARN** | PASS | 5,249,372,173 |
| hotset-1 | 23,411,501 | 23,580,083 | 0.99× | −0.17 ms | PASS | PASS | 27,292,959 |
| hotset-10 | 125,411,582 | 118,461,251 | 1.06× | +6.95 ms | **WARN** | PASS | 129,265,373 |
| hotset-100 | 985,360,790 | 799,500,410 | 1.23× | +185.86 ms | **WARN** | PASS | 1,127,308,870 |
| hotset-500 | 4,914,477,906 | 3,683,754,881 | 1.33× | +1,230.72 ms | **WARN** | PASS | 5,758,411,288 |
| metadata-1 | 29,651,664 | 20,652,541 | 1.44× | +9.00 ms | **WARN** | PASS | 25,777,250 |
| metadata-10 | 101,610,504 | 76,473,917 | 1.33× | +25.14 ms | **WARN** | PASS | 98,584,418 |
| metadata-100 | 918,530,093 | 649,605,248 | 1.41× | +268.92 ms | **WARN** | PASS | 774,377,377 |
| metadata-500 | 4,442,021,910 | 3,231,966,647 | 1.37× | +1,210.06 ms | **WARN** | PASS | 3,890,784,774 |
| recurring-1 | 24,056,293 | 21,456,208 | 1.12× | +2.60 ms | PASS | PASS | 26,511,958 |
| recurring-10 | 101,618,039 | 62,918,081 | 1.62× | +38.70 ms | **WARN** | PASS | 83,436,585 |
| recurring-100 | 618,637,324 | 462,840,955 | 1.34× | +155.80 ms | **WARN** | PASS | 599,441,623 |
| recurring-500 | 2,787,071,655 | 2,145,416,191 | 1.30× | +641.66 ms | **WARN** | PASS | 3,215,736,027 |
| unrelated-1 | 453,818,625 | 880,163,541 | 0.52× | −426.34 ms | PASS | PASS | 1,066,194,083 |
| unrelated-10 | 5,665,896,792 | 9,543,474,916 | 0.59× | −3,877.58 ms | PASS | PASS | 10,306,428,834 |
| unrelated-100-mixed-v2 | 3,189,752,295 | 3,549,609,120 | 0.90× | −359.86 ms | PASS | PASS | 4,743,104,045 |
| **unrelated-500-mixed-v2** | **16,106,808,892** | 18,163,888,871 | 0.89× | −2,057.08 ms | **FAIL — unrepaired (S2)**: registered Tier-1 gate `unrelated-history500 < 15 s` missed (16.107 s); root cause + owner decision request in [unrelated-500-tier1-rca.md](unrelated-500-tier1-rca.md) | PASS | 23,781,066,044 |

## `store_footprint` (6 rows; 4 fresh + 2 re-collected)

The two cells that appear in the #120 reuse list (`store-footprint-unique-100000`,
`store-footprint-large-object-500m`) were **re-collected fresh on purpose**: their
fsync-qualified evidence predates #107, and #107's pack coalescing changes exactly
their measured quantity (pack layout/footprint). The reuse condition ("for every
mechanism this campaign['s candidate] does not change") therefore fails for them;
their #118 receipts remain cited as history. Measured effect of that decision:
allocated 530,358,272 → 520,142,848 B (−10,215,424 B / −1.9 %) on unique-100000 —
reusing the old number would have misstated the candidate by 10 MB.

| Selection | Candidate ns (alloc/apparent B) | v0.1.3 ref ns | Ratio | Δ/op | Disposition | Proof | #104 ns (alloc B) |
|---|---|---:|---:|---:|---|---|---:|
| unique-100-low-v1 | 49,652,000 (5,160,960 / 5,160,960) | 31,947,543 | 1.55× | +17.70 ms | **WARN** | PASS | 57,973,876 (5,156,864) |
| unique-100000 | 5,397,130,666 (520,142,848 / 515,538,944) | 2,982,459,252 | 1.81× | +2,414.67 ms | **WARN** | PASS | 5,041,895,791 (520,372,224) |
| metadata-cardinality-100-low-v1 | 60,355,958 (5,193,728 / 5,193,728) | 35,869,626 | 1.68× | +24.49 ms | **WARN** | PASS | 56,224,000 (5,201,920) |
| metadata-cardinality-100000 | 6,639,510,832 (554,524,672 / 552,144,896) | 4,570,429,959 | 1.45× | +2,069.08 ms | **WARN** | PASS | 7,190,754,707 (554,381,312) |
| large-object-10m-low-v1 | 63,198,040 (10,203,136 / 10,203,136) | 43,893,543 | 1.44× | +19.30 ms | **WARN** | PASS | 62,515,040 (10,203,136) |
| large-object-500m | 1,253,795,750 (509,149,184 / 505,753,600) | 491,905,625 | 2.55× | +761.89 ms | **WARN** | PASS | 1,359,544,542 (512,479,232) |

## Diagnoses (aggregate deltas > ~1 s, mandatory)

- `distributed-500` (+1.379 s), `hotset-500` (+1.231 s), `metadata-500`
  (+1.210 s): construction-dominated 500-tier history workloads on the v0.1.5
  ordinary path. The unrelated-500 RCA measured the per-iteration mechanism cost
  on this same family: #116 bounded pending ≈ +2.49 ms/exec +1.46 ms/commit and
  #107 pack coalescing ≈ +0.4 ms per phase; 500 iterations × ~2.4–2.8 ms ≈
  +1.2–1.4 s — matching these aggregates. v0.1.3 predates authentication/pack/
  delta encoding entirely; against #104 (same workload, schema10 uncompacted)
  the candidate is faster on distributed-500/hotset-500/recurring-500 and slower
  on metadata-500 (single-sample context).
- `unique-100000` (+2.415 s) and `metadata-cardinality-100000` (+2.069 s):
  100,000-entry store construction through the ordinary admission path
  (authentication + pack assembly + delta encoding). #104 context: 5.042 s /
  7.191 s. The candidate's footprints match #104's within noise (±0.04 %) and
  are 1.9 % *smaller* allocated than the superseded fsync-qualified numbers
  (#107's measured effect, now captured on the candidate).

## Notes

- 26/26 cells collected fresh (20 + 6); 26/26 proofs PASS (walls 2.2–17.6 s,
  all inside the 60 s envelope); cleanups PASS.
- One Tier-1 miss: `dedup-history-unrelated-500-mixed-v2` 16.107 s ≥ 15 s gate
  ⇒ `FAIL — unrepaired` (S2), root-caused within the timebox to the #116
  bounded pending representation (+1.98 s, dominant) and #107 pack coalescing
  (+0.33 s), with a diagnostic attribution run on the archived `6693224e…`
  binary; owner decision (waive vs repair) requested on #120. Note the cell is
  **faster than v0.1.3** (0.89×): the gate is a v0.1.4/0.1.5-era absolute
  target that fsync-qualified met at 13.791 s and the #116/#107 treatments
  moved past.
- Bug ledger entry (S2): see `unrelated-500-tier1-rca.md`.

Receipts: `benchmark-results/host-store/issue120/{performance,verification}/{dedup_branch_history,store_footprint}/…`;
diagnostic: `benchmark-results/host-store/issue120/diagnostics/unrelated-500-on-6693224e/`.

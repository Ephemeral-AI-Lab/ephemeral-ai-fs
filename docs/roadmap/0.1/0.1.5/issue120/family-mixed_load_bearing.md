# #120 family report: `mixed_load_bearing` (4) + `namespace_mutation` (4) + `directory_construction_traversal` (12)

Candidate `c55daf13…` @ `1ff1f2ddd`. Timer `pure_call_sum_ns` (receipt op count
= 1 workload per sample). References: published v0.1.3 single samples (profile
undeclared ⇒ context). All registered harness targets PASS; all cleanups PASS;
18/18 fresh proofs PASS (walls 2.3–13.5 s); no S0/S1; no aggregate delta > 1 s
(max +189.73 ms), so no mandatory >1 s diagnosis applies.

## Rows

| Family | Selection | Candidate ns | v0.1.3 ns | Ratio | Δ/op | Disposition | Proof |
|---|---|---:|---:|---:|---:|---|---|
| mixed_load_bearing | agent-episodes-1-compact-v2 | 34,382,915 | 26,241,042 | 1.31× | +8.14 ms | **WARN** | PASS |
| mixed_load_bearing | agent-episodes-10-compact-v2 | 62,125,917 | 89,910,918 | 0.69× | −27.79 ms | PASS | PASS |
| mixed_load_bearing | agent-episodes-100 | 1,030,079,667 | 908,411,042 | 1.13× | +121.67 ms | **WARN** | PASS |
| mixed_load_bearing | agent-episodes-500 | 8,218,714,834 | 7,535,400,625 | 1.09× | +683.31 ms | **REUSED-FROM** #118 `remaining-shared` (fsync-qualified `440ae2c4…`) | PASS (reused) |
| namespace_mutation | relocate-delete-1-compact-v2 | 29,545,292 | 21,310,793 | 1.39× | +8.23 ms | **WARN** | PASS |
| namespace_mutation | relocate-delete-10-compact-v2 | 73,478,126 | 59,144,541 | 1.24× | +14.33 ms | **WARN** | PASS |
| namespace_mutation | relocate-delete-100-mixed-v4 | 74,188,707 | 51,018,082 | 1.45× | +23.17 ms | **WARN** | PASS |
| namespace_mutation | relocate-delete-500-mixed-v4 | 274,960,582 | 182,893,833 | 1.50× | +92.07 ms | **WARN** | PASS |
| directory_construction_traversal | construct-1-compact-v2 | 22,384,083 | 17,022,374 | 1.31× | +5.36 ms | **WARN** | PASS |
| directory_construction_traversal | construct-10-compact-v2 | 38,598,917 | 39,346,916 | 0.98× | −0.75 ms | PASS | PASS |
| directory_construction_traversal | construct-100-mixed-v4 | 253,421,666 | 216,059,959 | 1.17× | +37.36 ms | **WARN** | PASS |
| directory_construction_traversal | construct-500-mixed-v4 | 1,140,328,543 | 1,030,580,667 | 1.11× | +109.75 ms | **WARN** | PASS |
| directory_construction_traversal | metadata-scan-1-compact-v2 | 87,967,374 | 70,526,833 | 1.25× | +17.44 ms | **WARN** | PASS |
| directory_construction_traversal | metadata-scan-10-compact-v2 | 137,608,209 | 100,738,749 | 1.37× | +36.87 ms | **WARN** | PASS |
| directory_construction_traversal | metadata-scan-100-mixed-v4 | 333,027,500 | 244,238,709 | 1.36× | +88.79 ms | **WARN** | PASS |
| directory_construction_traversal | metadata-scan-500-mixed-v4 | 694,664,875 | 504,932,250 | 1.38× | +189.73 ms | **WARN** | PASS |
| directory_construction_traversal | content-scan-1-compact-v2 | 105,525,085 | 84,759,585 | 1.24× | +20.77 ms | **WARN** | PASS |
| directory_construction_traversal | content-scan-10-compact-v2 | 348,615,250 | 308,870,416 | 1.13× | +39.74 ms | **WARN** | PASS |
| directory_construction_traversal | content-scan-100-mixed-v4 | 1,231,326,957 | 1,105,140,625 | 1.11× | +126.19 ms | **WARN** | PASS |
| directory_construction_traversal | content-scan-500-mixed-v4 | 4,434,140,708 | 3,906,595,000 | 1.14× | +527.55 ms | **REUSED-FROM** #118 `remaining-shared` (fsync-qualified `440ae2c4…`) | PASS (reused) |

## Notes

- 18/20 cells collected fresh; 2 reused per the frozen contract
  (`agent-episodes-500`, `directory-content-scan-500-mixed-v4`), both
  fsync-qualified single samples with treatment identity recorded (pre-#107/#116).
- WARN set (16 cells): workload deltas 5.4–189.7 ms, ratios 1.11–1.50×, all
  single-sample without an n3 comparator ⇒ Tier-3, none above the hard cap.
  Context: candidate vs #104 on the same cells is mixed (faster on
  agent-episodes-10/100, construct-100/500; slower on relocate-delete tiers;
  single-sample diagnostics).
- The reuse-condition deviation recorded for `store_footprint` in the previous
  family report does not extend here: these families' measured mechanism is
  workload latency, which #107/#116 affect only through the per-iteration costs
  already quantified in the unrelated-500 RCA (well under the material floor at
  these workload sizes).

Receipts: `benchmark-results/host-store/issue120/{performance,verification}/{mixed_load_bearing,namespace_mutation,directory_construction_traversal}/…`

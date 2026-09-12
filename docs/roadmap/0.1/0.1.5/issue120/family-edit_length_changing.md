# #120 family report: group 6 — `edit_length_changing` (32) + `payload_create_read` (8) + `dedup_workspace_reuse` (14) + `dedup_cross_file` (10) + `dedup_cdc_locality` (20 + 1 proof-only)

Candidate `c55daf13…` @ `1ff1f2ddd`. Timers: `edit_commit_ns` (SDK cells, ops=1)
and `pure_call_sum_ns`/`product_call_sum_ns` (receipt op count = 1 workload).
References: published v0.1.3 single samples (profile undeclared ⇒ context).
Disposition convention as before (≤ ref or Δ/op ≤ 3 ms floor → PASS; beyond-floor
non-Tier-2 → WARN). All registered harness targets PASS; all cleanups PASS;
77/77 fresh proofs PASS (walls 1.8–25.2 s, inside 60 s); the proof-only
`dedup-cdc-boundaries-proof` PASS (1.74 s); no S0/S1; no fresh aggregate
delta > 1 s (max +943.03 ms on `dedup-cdc-scattered-500`).

## `edit_length_changing` (30 fresh + 2 reused)

Fresh: 12 PASS / 18 WARN. PASS cells: insert-middle-4k-on-10/100/500mib (0.51–
1.07×), delete-middle-4k-on-10mib (+1.32 ms), prepend-head-4k-on-500mib
(+2.41 ms), replace-grow on 1/10/100/500 (hmm: 10mib +3.57 ms is WARN; PASS are
1mib +2.75, 100mib +2.05, 500mib +1.41), replace-shrink on 1/10/100mib,
truncate-tail-4k-on-10/100mib, zero-extend-tail-4k-on-1mib. WARN cells (18):
append-tail 1/10/100/500mib (+3.1–4.3 ms), delete-middle 100/500mib
(+3.27/+5.29 ms), prepend-head 1/10/100mib (+3.07/+4.37/+4.18 ms),
replace-grow-10mib (+3.57 ms), replace-shrink-500mib (+6.88 ms),
truncate-tail 1/500mib (+4.77/+4.00 ms), zero-extend 10/100/500mib
(+4.17/+3.04/+3.67 ms). All single-sample, all under the 10 ms/op hard cap ⇒
Tier-3. Reused (treatment `6693224e…` post-#116 affected-rerun, treatment
identity recorded): `insert-middle-4k-on-1mib-ops-1` 8.31 ms (1.49×),
`delete-middle-4k-on-1mib-ops-1` 8.11 ms (1.53×) — **REUSED-FROM**, proofs
reused from the affected-rerun.

## `payload_create_read` (7 fresh + 1 reused)

| Selection | Candidate | v0.1.3 ref | Ratio | Δ/op | Disposition | Proof |
|---|---:|---:|---:|---:|---|---|
| payload-create-1m-compact-v2 | 29.33 ms | 28.84 ms | 1.02× | +0.50 ms | PASS | PASS |
| payload-create-10m-compact-v2 | 82.57 ms | 91.56 ms | 0.90× | −8.99 ms | PASS | PASS |
| payload-create-100m | 575.91 ms | 682.77 ms | 0.84× | −106.86 ms | PASS | PASS |
| payload-create-500m | 2,418.82 ms | 3,068.25 ms | 0.79× | −649.43 ms | PASS | PASS |
| payload-random-read-1-compact-v2 | 20.46 ms | 14.58 ms | 1.40× | +5.88 ms | **WARN** | PASS |
| payload-random-read-100 | 69.68 ms | 56.40 ms | 1.24× | +13.29 ms | **WARN** | PASS |
| payload-random-read-500 | 232.30 ms | 219.69 ms | 1.06× | +12.61 ms | **WARN** | PASS |
| payload-random-read-10-compact-v2 | 25.17 ms | 17.42 ms | 1.44× | +7.75 ms | **REUSED-FROM** #118 fsync-qualified | PASS (reused) |

## `dedup_workspace_reuse` (13 fresh + 1 reused)

PASS (8): exact-100 (0.62×), exact-500 (0.91×), local-1 (0.99×), local-10
(0.77×), local-100 (0.66×), unique-10-compact-v2 (1.00×), unique-10-base128
(0.91×), unique-100 (0.83×). WARN (5): exact-1 (+3.22 ms), local-500
(+201.48 ms), unique-1-base128 (+5.68 ms), unique-1-compact-v2 (+10.47 ms),
unique-500 (+388.43 ms). REUSED-FROM (1): exact-10-compact-v2 81.92 ms (0.75×),
fsync-qualified. 13/13 fresh proofs PASS.

## `dedup_cross_file` (8 fresh + 2 reused)

All 8 fresh cells WARN: anchor-1 +3.80 ms (2.00×), identical-100 +25.42 ms
(1.88×), identical-500 +99.28 ms (1.90×), mixed-10 +11.06 ms (1.46×),
mixed-100 +162.23 ms (2.42×), mixed-500 +690.85 ms (2.27×), unique-10 +12.00 ms
(1.54×), unique-100 +219.25 ms (3.35×). REUSED-FROM (2, fsync-qualified):
identical-10 19.17 ms (1.04×), unique-500 1,290.93 ms (2.90×). 8/8 fresh proofs
PASS.

## `dedup_cdc_locality` (19 fresh + 1 reused + 1 proof-only)

PASS (2): delete-10 (+0.74 ms), insert-10 (+0.70 ms). WARN (17): the
remaining 17 fresh cells, ratios 1.04–3.45×, per-workload deltas +3.7 ms to
+943.03 ms (scattered-500, 2.95× — the largest fresh aggregate in the campaign
so far, still under the 1 s diagnosis threshold). REUSED-FROM (1):
overwrite-10 18.28 ms (0.89×), fsync-qualified. 19/19 fresh proofs PASS;
`dedup-cdc-boundaries-proof` (proof-only) PASS 1.74 s.

## Notes

- 77/84 cells collected fresh this group; 7 reused per the frozen contract with
  treatment identities recorded (2 × `6693224e…` affected-rerun post-#116, 5 ×
  fsync-qualified pre-#107/#116).
- The dedup families' large ratios vs v0.1.3 (1.5–3.5×) are the v0.1.5 ordinary
  path (authenticated CAS + CDC + pack assembly) measured against
  pre-authentication v0.1.3 references; the campaign context column for #112
  (worst-case grouping) is carried into the final report.
- Tally after families 1–6: 194 of 198 registered performance cells
  dispositioned (177 fresh + 17 reused), plus 1 of 29 proof-only cells run.

Receipts: `benchmark-results/host-store/issue120/{performance,verification}/{edit_length_changing,payload_create_read,dedup_workspace_reuse,dedup_cross_file,dedup_cdc_locality}/…`

# #120 final report — v0.1.5 finalization campaign: family → per-test

Status: **terminal collection complete** for every registered selection. One
registered Tier-1 gate is `FAIL — unrepaired` with its root cause published and
an owner decision requested; one proof-only selection is NOT_RUN_OPTIONAL by the
frozen campaign declaration. No S0/S1 defect is open. #120 itself cannot close
until the unrelated-500 owner decision is recorded and any directed repair is
qualified — the exact unmet gate is stated in §6.

## 1. Campaign identity and protocol

| Field | Value |
|---|---|
| Candidate commit | `1ff1f2dddeb60493953311de316fa5bec4634a1a` (clean, == origin/main) |
| `LAYERFS_SOURCE_SEAL` | `a555211fdc6e96e025764eba2c2c36f709baa7e9446840a07a8669935252fec3` |
| `LAYERFS_PRODUCT_SEAL` | `276c5970aabf485594d90ae30920b3cdb310134a7572589c558063a9d52ce093` |
| Host binary SHA256 | `c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295` |
| Image | `sha256:c83085b897e272204e5477e66b33ec9b328af568299448872441e7a676ce1761` |
| Applicability decision | `3e308a8f2` behavior-neutral (line-by-line: transparent type alias, identity reborrow ×4, identical control flow, identity conversion; remainder rustfmt reflow); final-treatment evidence reused with provenance. [applicability-3e308a8f2.md](applicability-3e308a8f2.md) |
| Build validation | host build 12.68 s (artifacts fresh; no stale-fingerprint anomaly); full native gate `tools/test-fast.sh` PASS, 530 tests, 118 s, exit 0 |
| Protocol | one complete public sample per selection (`--perf-fast --collection-mode`, 300/310/600 s allowances, seed 1 / repetition 1 per route, fresh/clone per setup policy) + one independent identity-pinned proof per fresh selection via `verify-selected.py`; runner/verifier serialize on the measurement lock; every receipt under `benchmark-results/host-store/issue120/` (immutable, never overwritten) |

**Cache contract per row (campaign-wide):** every fresh performance row is a
single complete sample on prepared-input reuse (`fixture_cache_profile`:
warm-prepared-reuse, clone per sample, cache identity recorded in the receipt)
except `namespace-100000`, which is **VERIFIED_COLD** under
`namespace-100000-cold-v2` (0/125,169 resident pages after eviction, metadata
VERIFIED, detector live self-check detected 32 warm pages, acquisition 25.942 s
reported separately and inside the complete envelope of 31.615 s). Reused rows
carry their source run's contract. v0.1.3 references are single samples with
**undeclared** cache profile ⇒ all ratios are historical context, never paired
claims; dispositions follow the frozen Tier-1/2/3 rules.

**Disposition convention (stated once, applied to all ordinary comparative
cells):** candidate ≤ reference, or Δ/op under the 3 ms wall floor → **PASS**
(values recorded anyway); beyond-floor non-Tier-2 (at n=1 no alternating-pair
comparator exists, so Tier-2 condition 2 cannot hold) → **WARN** with ratio +
absolute + per-op delta; Tier-2 material (all three conditions) → FAIL; Tier-1
registered targets → PASS/FAIL/OWNER-WAIVED directly.

**Tallies:** 198/198 registered performance selections dispositioned = **182
fresh + 16 reused**; 29/29 proof-only selections terminal = **28 PASS + 1
NOT_RUN_OPTIONAL**; 182/182 fresh proofs PASS; all cleanups PASS; harness
targets: 196 PASS, 1 TARGET_MISS (unrelated-500, FAIL—unrepaired), 1
NOT_APPLICABLE_COLD_CONTRACT (namespace-100000, 2.7 s target owner-waived).

## 2. Family → per-test (every registered selection)

| Family | Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof |
|---|---|---|---|--:|--:|--:|--:|--:|---|---|
| payload_create_read | `payload-create-1m-compact-v2` | performance | `pure_call_sum_ns` | 29.33 ms | 28.84 ms | 1.02× | +0.50 ms | 1 | PASS | PASS |
| payload_create_read | `payload-create-10m-compact-v2` | performance | `pure_call_sum_ns` | 82.57 ms | 91.56 ms | 0.90× | -8.99 ms | 1 | PASS | PASS |
| payload_create_read | `payload-create-100m` | performance | `pure_call_sum_ns` | 575.91 ms | 682.77 ms | 0.84× | -106.86 ms | 1 | PASS | PASS |
| payload_create_read | `payload-create-500m` | performance | `pure_call_sum_ns` | 2.419 s | 3.068 s | 0.79× | -649.43 ms | 1 | PASS | PASS |
| payload_create_read | `payload-random-read-1-compact-v2` | performance | `pure_call_sum_ns` | 20.46 ms | 14.58 ms | 1.40× | +5.88 ms | 1 | WARN | PASS |
| payload_create_read | `payload-random-read-10-compact-v2` | performance | `pure_call_sum_ns` | 25.17 ms | 17.42 ms | 1.45× | +7.75 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| payload_create_read | `payload-random-read-100` | performance | `pure_call_sum_ns` | 69.68 ms | 56.40 ms | 1.24× | +13.29 ms | 1 | WARN | PASS |
| payload_create_read | `payload-random-read-500` | performance | `pure_call_sum_ns` | 232.30 ms | 219.69 ms | 1.06× | +12.61 ms | 1 | WARN | PASS |
| dedup_workspace_reuse | `dedup-workspace-exact-1-compact-v2` | performance | `pure_call_sum_ns` | 34.79 ms | 31.56 ms | 1.10× | +3.22 ms | 1 | WARN | PASS |
| dedup_workspace_reuse | `dedup-workspace-exact-10-compact-v2` | performance | `pure_call_sum_ns` | 81.92 ms | 109.32 ms | 0.75× | -27.40 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| dedup_workspace_reuse | `dedup-workspace-exact-100` | performance | `pure_call_sum_ns` | 524.62 ms | 840.53 ms | 0.62× | -315.91 ms | 1 | PASS | PASS |
| dedup_workspace_reuse | `dedup-workspace-exact-500` | performance | `pure_call_sum_ns` | 3.900 s | 4.297 s | 0.91× | -396.20 ms | 1 | PASS | PASS |
| dedup_workspace_reuse | `dedup-workspace-local-1-compact-v2` | performance | `pure_call_sum_ns` | 30.20 ms | 30.61 ms | 0.99× | -0.41 ms | 1 | PASS | PASS |
| dedup_workspace_reuse | `dedup-workspace-local-10-compact-v2` | performance | `pure_call_sum_ns` | 78.99 ms | 103.15 ms | 0.77× | -24.16 ms | 1 | PASS | PASS |
| dedup_workspace_reuse | `dedup-workspace-local-100` | performance | `pure_call_sum_ns` | 532.42 ms | 808.39 ms | 0.66× | -275.97 ms | 1 | PASS | PASS |
| dedup_workspace_reuse | `dedup-workspace-local-500` | performance | `pure_call_sum_ns` | 4.509 s | 4.307 s | 1.05× | +201.48 ms | 1 | WARN | PASS |
| dedup_workspace_reuse | `dedup-workspace-unique-1-compact-v2` | performance | `pure_call_sum_ns` | 37.94 ms | 27.48 ms | 1.38× | +10.47 ms | 1 | WARN | PASS |
| dedup_workspace_reuse | `dedup-workspace-unique-10-compact-v2` | performance | `pure_call_sum_ns` | 96.02 ms | 95.67 ms | 1.00× | +0.35 ms | 1 | PASS | PASS |
| dedup_workspace_reuse | `dedup-workspace-unique-100` | performance | `pure_call_sum_ns` | 634.99 ms | 768.00 ms | 0.83× | -133.01 ms | 1 | PASS | PASS |
| dedup_workspace_reuse | `dedup-workspace-unique-500` | performance | `pure_call_sum_ns` | 4.708 s | 4.320 s | 1.09× | +388.43 ms | 1 | WARN | PASS |
| dedup_workspace_reuse | `dedup-workspace-unique-1-base128-v3` | performance | `pure_call_sum_ns` | 39.06 ms | 33.38 ms | 1.17× | +5.68 ms | 1 | WARN | PASS |
| dedup_workspace_reuse | `dedup-workspace-unique-10-base128-v3` | performance | `pure_call_sum_ns` | 93.55 ms | 103.01 ms | 0.91× | -9.46 ms | 1 | PASS | PASS |
| dedup_cross_file | `dedup-cross-file-anchor-1` | performance | `pure_call_sum_ns` | 7.61 ms | 3.81 ms | 2.00× | +3.80 ms | 1 | WARN | PASS |
| dedup_cross_file | `dedup-cross-file-unique-10` | performance | `pure_call_sum_ns` | 34.39 ms | 22.39 ms | 1.54× | +12.00 ms | 1 | WARN | PASS |
| dedup_cross_file | `dedup-cross-file-unique-100` | performance | `pure_call_sum_ns` | 312.51 ms | 93.27 ms | 3.35× | +219.25 ms | 1 | WARN | PASS |
| dedup_cross_file | `dedup-cross-file-unique-500` | performance | `pure_call_sum_ns` | 1.291 s | 444.95 ms | 2.90× | +845.98 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| dedup_cross_file | `dedup-cross-file-identical-10` | performance | `pure_call_sum_ns` | 19.17 ms | 18.40 ms | 1.04× | +0.77 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| dedup_cross_file | `dedup-cross-file-identical-100` | performance | `pure_call_sum_ns` | 54.37 ms | 28.96 ms | 1.88× | +25.42 ms | 1 | WARN | PASS |
| dedup_cross_file | `dedup-cross-file-identical-500` | performance | `pure_call_sum_ns` | 209.91 ms | 110.63 ms | 1.90× | +99.28 ms | 1 | WARN | PASS |
| dedup_cross_file | `dedup-cross-file-mixed-10` | performance | `pure_call_sum_ns` | 35.11 ms | 24.05 ms | 1.46× | +11.06 ms | 1 | WARN | PASS |
| dedup_cross_file | `dedup-cross-file-mixed-100` | performance | `pure_call_sum_ns` | 276.18 ms | 113.94 ms | 2.42× | +162.23 ms | 1 | WARN | PASS |
| dedup_cross_file | `dedup-cross-file-mixed-500` | performance | `pure_call_sum_ns` | 1.233 s | 541.99 ms | 2.27× | +690.85 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-overwrite-1` | performance | `pure_call_sum_ns` | 8.15 ms | 4.00 ms | 2.04× | +4.15 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-overwrite-10` | performance | `pure_call_sum_ns` | 18.28 ms | 20.45 ms | 0.89× | -2.17 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| dedup_cdc_locality | `dedup-cdc-overwrite-100` | performance | `pure_call_sum_ns` | 61.05 ms | 30.95 ms | 1.97× | +30.10 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-overwrite-500` | performance | `pure_call_sum_ns` | 243.22 ms | 122.04 ms | 1.99× | +121.18 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-insert-1` | performance | `pure_call_sum_ns` | 7.59 ms | 3.90 ms | 1.95× | +3.69 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-insert-10` | performance | `pure_call_sum_ns` | 18.76 ms | 18.06 ms | 1.04× | +0.70 ms | 1 | PASS | PASS |
| dedup_cdc_locality | `dedup-cdc-insert-100` | performance | `pure_call_sum_ns` | 64.35 ms | 34.18 ms | 1.88× | +30.17 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-insert-500` | performance | `pure_call_sum_ns` | 257.57 ms | 136.78 ms | 1.88× | +120.79 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-delete-1` | performance | `pure_call_sum_ns` | 8.25 ms | 4.41 ms | 1.87× | +3.84 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-delete-10` | performance | `pure_call_sum_ns` | 18.97 ms | 18.22 ms | 1.04× | +0.74 ms | 1 | PASS | PASS |
| dedup_cdc_locality | `dedup-cdc-delete-100` | performance | `pure_call_sum_ns` | 57.37 ms | 33.17 ms | 1.73× | +24.20 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-delete-500` | performance | `pure_call_sum_ns` | 243.00 ms | 130.86 ms | 1.86× | +112.14 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-common-body-1` | performance | `pure_call_sum_ns` | 8.86 ms | 4.13 ms | 2.15× | +4.73 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-common-body-10` | performance | `pure_call_sum_ns` | 24.89 ms | 20.40 ms | 1.22× | +4.50 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-common-body-100` | performance | `pure_call_sum_ns` | 131.23 ms | 58.21 ms | 2.25× | +73.02 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-common-body-500` | performance | `pure_call_sum_ns` | 552.10 ms | 244.35 ms | 2.26× | +307.75 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-scattered-1` | performance | `pure_call_sum_ns` | 11.48 ms | 4.65 ms | 2.47× | +6.83 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-scattered-10` | performance | `pure_call_sum_ns` | 40.93 ms | 23.47 ms | 1.74× | +17.46 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-scattered-100` | performance | `pure_call_sum_ns` | 308.33 ms | 89.37 ms | 3.45× | +218.96 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-scattered-500` | performance | `pure_call_sum_ns` | 1.426 s | 483.01 ms | 2.95× | +943.03 ms | 1 | WARN | PASS |
| dedup_cdc_locality | `dedup-cdc-boundaries-proof` | proof-only | — | 1.74 s wall | — | — | — | — | **PASS** | PASS |
| edit_length_preserving | `overwrite-head-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 10.95 ms | 5.78 ms | 1.90× | +5.17 ms | 1 | WARN | PASS |
| edit_length_preserving | `overwrite-head-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 11.79 ms | 6.15 ms | 1.92× | +5.64 ms | 1 | WARN | PASS |
| edit_length_preserving | `overwrite-head-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 7.88 ms | 6.65 ms | 1.19× | +1.23 ms | 1 | PASS | PASS |
| edit_length_preserving | `overwrite-head-4k-on-500mib-ops-1` | performance | `edit_commit_ns` | 8.78 ms | 8.41 ms | 1.04× | +0.37 ms | 1 | PASS | PASS |
| edit_length_preserving | `overwrite-middle-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 8.54 ms | 6.14 ms | 1.39× | +2.40 ms | 1 | **REUSED-FROM** 6693224e affected-rerun | reused |
| edit_length_preserving | `overwrite-middle-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 8.17 ms | 5.92 ms | 1.38× | +2.25 ms | 1 | PASS | PASS |
| edit_length_preserving | `overwrite-middle-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 8.83 ms | 6.19 ms | 1.43× | +2.64 ms | 1 | PASS | PASS |
| edit_length_preserving | `overwrite-middle-4k-on-500mib-ops-1` | performance | `edit_commit_ns` | 11.18 ms | 7.17 ms | 1.56× | +4.01 ms | 1 | WARN | PASS |
| edit_length_preserving | `overwrite-tail-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 8.39 ms | 6.11 ms | 1.37× | +2.27 ms | 1 | PASS | PASS |
| edit_length_preserving | `overwrite-tail-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 8.57 ms | 7.27 ms | 1.18× | +1.30 ms | 1 | PASS | PASS |
| edit_length_preserving | `overwrite-tail-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 9.80 ms | 7.63 ms | 1.28× | +2.17 ms | 1 | PASS | PASS |
| edit_length_preserving | `overwrite-tail-4k-on-500mib-ops-1` | performance | `edit_commit_ns` | 11.80 ms | 9.38 ms | 1.26× | +2.42 ms | 1 | PASS | PASS |
| edit_length_changing | `insert-middle-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 8.31 ms | 5.59 ms | 1.49× | +2.72 ms | 1 | **REUSED-FROM** 6693224e affected-rerun | reused |
| edit_length_changing | `insert-middle-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 7.93 ms | 7.40 ms | 1.07× | +0.54 ms | 1 | PASS | PASS |
| edit_length_changing | `insert-middle-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 8.65 ms | 12.44 ms | 0.70× | -3.79 ms | 1 | PASS | PASS |
| edit_length_changing | `insert-middle-4k-on-500mib-result-capped-v2-ops-1` | performance | `edit_commit_ns` | 9.24 ms | 18.01 ms | 0.51× | -8.77 ms | 1 | PASS | PASS |
| edit_length_changing | `delete-middle-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 8.11 ms | 5.29 ms | 1.53× | +2.82 ms | 1 | **REUSED-FROM** 6693224e affected-rerun | reused |
| edit_length_changing | `delete-middle-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 7.89 ms | 6.57 ms | 1.20× | +1.32 ms | 1 | PASS | PASS |
| edit_length_changing | `delete-middle-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 10.02 ms | 6.76 ms | 1.48× | +3.27 ms | 1 | WARN | PASS |
| edit_length_changing | `delete-middle-4k-on-500mib-ops-1` | performance | `edit_commit_ns` | 12.78 ms | 7.49 ms | 1.71× | +5.29 ms | 1 | WARN | PASS |
| edit_length_changing | `append-tail-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 9.70 ms | 6.26 ms | 1.55× | +3.44 ms | 1 | WARN | PASS |
| edit_length_changing | `append-tail-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 9.63 ms | 5.91 ms | 1.63× | +3.72 ms | 1 | WARN | PASS |
| edit_length_changing | `append-tail-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 9.34 ms | 6.25 ms | 1.49× | +3.09 ms | 1 | WARN | PASS |
| edit_length_changing | `append-tail-4k-on-500mib-result-capped-v2-ops-1` | performance | `edit_commit_ns` | 10.98 ms | 6.69 ms | 1.64× | +4.30 ms | 1 | WARN | PASS |
| edit_length_changing | `prepend-head-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 8.98 ms | 5.91 ms | 1.52× | +3.07 ms | 1 | WARN | PASS |
| edit_length_changing | `prepend-head-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 10.04 ms | 5.67 ms | 1.77× | +4.37 ms | 1 | WARN | PASS |
| edit_length_changing | `prepend-head-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 10.45 ms | 6.27 ms | 1.67× | +4.18 ms | 1 | WARN | PASS |
| edit_length_changing | `prepend-head-4k-on-500mib-result-capped-v2-ops-1` | performance | `edit_commit_ns` | 9.50 ms | 7.10 ms | 1.34× | +2.41 ms | 1 | PASS | PASS |
| edit_length_changing | `replace-grow-middle-2k-to-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 8.49 ms | 5.74 ms | 1.48× | +2.75 ms | 1 | PASS | PASS |
| edit_length_changing | `replace-grow-middle-2k-to-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 10.04 ms | 6.46 ms | 1.55× | +3.57 ms | 1 | WARN | PASS |
| edit_length_changing | `replace-grow-middle-2k-to-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 8.43 ms | 6.38 ms | 1.32× | +2.05 ms | 1 | PASS | PASS |
| edit_length_changing | `replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1` | performance | `edit_commit_ns` | 9.33 ms | 7.92 ms | 1.18× | +1.41 ms | 1 | PASS | PASS |
| edit_length_changing | `replace-shrink-middle-4k-to-2k-on-1mib-ops-1` | performance | `edit_commit_ns` | 7.82 ms | 5.88 ms | 1.33× | +1.94 ms | 1 | PASS | PASS |
| edit_length_changing | `replace-shrink-middle-4k-to-2k-on-10mib-ops-1` | performance | `edit_commit_ns` | 8.33 ms | 5.77 ms | 1.44× | +2.56 ms | 1 | PASS | PASS |
| edit_length_changing | `replace-shrink-middle-4k-to-2k-on-100mib-ops-1` | performance | `edit_commit_ns` | 8.48 ms | 7.01 ms | 1.21× | +1.47 ms | 1 | PASS | PASS |
| edit_length_changing | `replace-shrink-middle-4k-to-2k-on-500mib-ops-1` | performance | `edit_commit_ns` | 13.87 ms | 7.00 ms | 1.98× | +6.88 ms | 1 | WARN | PASS |
| edit_length_changing | `truncate-tail-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 10.07 ms | 5.29 ms | 1.90× | +4.77 ms | 1 | WARN | PASS |
| edit_length_changing | `truncate-tail-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 9.17 ms | 15.64 ms | 0.59× | -6.47 ms | 1 | PASS | PASS |
| edit_length_changing | `truncate-tail-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 9.39 ms | 6.53 ms | 1.44× | +2.86 ms | 1 | PASS | PASS |
| edit_length_changing | `truncate-tail-4k-on-500mib-ops-1` | performance | `edit_commit_ns` | 10.89 ms | 6.90 ms | 1.58× | +4.00 ms | 1 | WARN | PASS |
| edit_length_changing | `zero-extend-tail-4k-on-1mib-ops-1` | performance | `edit_commit_ns` | 7.73 ms | 5.63 ms | 1.37× | +2.10 ms | 1 | PASS | PASS |
| edit_length_changing | `zero-extend-tail-4k-on-10mib-ops-1` | performance | `edit_commit_ns` | 9.80 ms | 5.64 ms | 1.74× | +4.17 ms | 1 | WARN | PASS |
| edit_length_changing | `zero-extend-tail-4k-on-100mib-ops-1` | performance | `edit_commit_ns` | 9.45 ms | 6.40 ms | 1.48× | +3.04 ms | 1 | WARN | PASS |
| edit_length_changing | `zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1` | performance | `edit_commit_ns` | 10.33 ms | 6.66 ms | 1.55× | +3.67 ms | 1 | WARN | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1` | performance | `edit_commit_ns` | 11.02 ms | 6.85 ms | 1.61× | +4.17 ms | 1 | WARN | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1` | performance | `edit_commit_ns` | 13.05 ms | 8.51 ms | 1.53× | +4.54 ms | 1 | WARN | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1` | performance | `edit_commit_ns` | 9.27 ms | 7.34 ms | 1.26× | +1.94 ms | 1 | PASS | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1` | performance | `edit_commit_ns` | 10.04 ms | 8.27 ms | 1.21× | +1.77 ms | 1 | PASS | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1` | performance | `edit_commit_ns` | 8.43 ms | 7.85 ms | 1.07× | +0.57 ms | 1 | PASS | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1` | performance | `edit_commit_ns` | 10.31 ms | 7.52 ms | 1.37× | +2.80 ms | 1 | PASS | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1` | performance | `edit_commit_ns` | 9.54 ms | 7.87 ms | 1.21× | +1.67 ms | 1 | PASS | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1` | performance | `edit_commit_ns` | 12.26 ms | 8.47 ms | 1.45× | +3.79 ms | 1 | WARN | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1` | performance | `edit_commit_ns` | 7.59 ms | 7.18 ms | 1.06× | +0.42 ms | 1 | PASS | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1` | performance | `edit_commit_ns` | 11.77 ms | 7.01 ms | 1.68× | +4.75 ms | 1 | WARN | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1` | performance | `edit_commit_ns` | 10.91 ms | 7.58 ms | 1.44× | +3.33 ms | 1 | WARN | PASS |
| edit_canonical_chunk_count | `overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1` | performance | `edit_commit_ns` | 15.38 ms | 8.10 ms | 1.90× | +7.28 ms | 1 | WARN | PASS |
| init_namespace | `namespace-100-compact-v3` | performance | `layerstack_init_ns` | 28.70 ms | 8.90 ms | 3.22× | +19.80 ms | 1 | WARN | PASS |
| init_namespace | `namespace-1000-compact-v3` | performance | `layerstack_init_ns` | 112.73 ms | 38.08 ms | 2.96× | +74.66 ms | 1 | WARN | PASS |
| init_namespace | `namespace-10000` | performance | `layerstack_init_ns` | 1.020 s | 403.47 ms | 2.53× | +616.95 ms | 1 | WARN | PASS |
| init_namespace | `namespace-100000` | performance | `layerstack_init_ns` | 4.398 s | 2.603 s | 1.69× | +1,794.38 ms | 1 | WARN | PASS |
| store_footprint | `store-footprint-unique-100000` | performance | `product_call_sum_ns` | 5.397 s | 2.982 s | 1.81× | +2,414.67 ms | 1 | WARN | PASS |
| store_footprint | `store-footprint-metadata-cardinality-100000` | performance | `product_call_sum_ns` | 6.640 s | 4.570 s | 1.45× | +2,069.08 ms | 1 | WARN | PASS |
| store_footprint | `store-footprint-large-object-500m` | performance | `product_call_sum_ns` | 1.254 s | 491.91 ms | 2.55× | +761.89 ms | 1 | WARN | PASS |
| store_footprint | `store-footprint-unique-100-low-v1` | performance | `product_call_sum_ns` | 49.65 ms | 31.95 ms | 1.55× | +17.70 ms | 1 | WARN | PASS |
| store_footprint | `store-footprint-metadata-cardinality-100-low-v1` | performance | `product_call_sum_ns` | 60.36 ms | 35.87 ms | 1.68× | +24.49 ms | 1 | WARN | PASS |
| store_footprint | `store-footprint-large-object-10m-low-v1` | performance | `product_call_sum_ns` | 63.20 ms | 43.89 ms | 1.44× | +19.30 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-create-1-compact-v2` | performance | `pure_call_sum_ns` | 23.26 ms | 19.05 ms | 1.22× | +4.21 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-create-10-compact-v2` | performance | `pure_call_sum_ns` | 30.16 ms | 26.71 ms | 1.13× | +3.45 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-create-100-mixed-v4` | performance | `pure_call_sum_ns` | 78.98 ms | 53.62 ms | 1.47× | +25.36 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-create-500-mixed-v4` | performance | `pure_call_sum_ns` | 242.66 ms | 201.74 ms | 1.20× | +40.92 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-stat-1-compact-v2` | performance | `pure_call_sum_ns` | 21.81 ms | 15.83 ms | 1.38× | +5.97 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-stat-10-compact-v2` | performance | `pure_call_sum_ns` | 28.63 ms | 21.19 ms | 1.35× | +7.43 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-stat-100-mixed-v4` | performance | `pure_call_sum_ns` | 45.70 ms | 37.81 ms | 1.21× | +7.89 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-stat-500-mixed-v4` | performance | `pure_call_sum_ns` | 69.84 ms | 55.35 ms | 1.26× | +14.49 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-unlink-1-compact-v2` | performance | `pure_call_sum_ns` | 19.56 ms | 17.63 ms | 1.11× | +1.93 ms | 1 | PASS | PASS |
| tiny_file_churn | `tiny-unlink-10-compact-v2` | performance | `pure_call_sum_ns` | 28.36 ms | 25.21 ms | 1.12× | +3.15 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-unlink-100-mixed-v4` | performance | `pure_call_sum_ns` | 63.78 ms | 50.92 ms | 1.25× | +12.87 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-unlink-500-mixed-v4` | performance | `pure_call_sum_ns` | 119.58 ms | 114.28 ms | 1.05× | +5.30 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-bulk-create-1-compact-v2` | performance | `pure_call_sum_ns` | 97.12 ms | 96.73 ms | 1.00× | +0.39 ms | 1 | PASS | PASS |
| tiny_file_churn | `tiny-bulk-create-10-compact-v2` | performance | `pure_call_sum_ns` | 316.83 ms | 295.90 ms | 1.07× | +20.94 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-bulk-create-100-mixed-v3` | performance | `pure_call_sum_ns` | 983.33 ms | 989.89 ms | 0.99× | -6.56 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| tiny_file_churn | `tiny-bulk-create-500-mixed-v3` | performance | `pure_call_sum_ns` | 5.733 s | 5.054 s | 1.13× | +678.94 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-bulk-delete-1-compact-v2` | performance | `pure_call_sum_ns` | 110.07 ms | 93.35 ms | 1.18× | +16.72 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-bulk-delete-10-compact-v2` | performance | `pure_call_sum_ns` | 209.75 ms | 168.25 ms | 1.25× | +41.49 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-bulk-delete-100-mixed-v3` | performance | `pure_call_sum_ns` | 306.85 ms | 259.28 ms | 1.18× | +47.57 ms | 1 | WARN | PASS |
| tiny_file_churn | `tiny-bulk-delete-500-mixed-v3` | performance | `pure_call_sum_ns` | 1.185 s | 933.00 ms | 1.27× | +252.34 ms | 1 | WARN | PASS |
| namespace_mutation | `namespace-subtree-relocate-delete-1-compact-v2` | performance | `pure_call_sum_ns` | 29.55 ms | 21.31 ms | 1.39× | +8.23 ms | 1 | WARN | PASS |
| namespace_mutation | `namespace-subtree-relocate-delete-10-compact-v2` | performance | `pure_call_sum_ns` | 73.48 ms | 59.14 ms | 1.24× | +14.33 ms | 1 | WARN | PASS |
| namespace_mutation | `namespace-subtree-relocate-delete-100-mixed-v4` | performance | `pure_call_sum_ns` | 74.19 ms | 51.02 ms | 1.45× | +23.17 ms | 1 | WARN | PASS |
| namespace_mutation | `namespace-subtree-relocate-delete-500-mixed-v4` | performance | `pure_call_sum_ns` | 274.96 ms | 182.89 ms | 1.50× | +92.07 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-construct-1-compact-v2` | performance | `pure_call_sum_ns` | 22.38 ms | 17.02 ms | 1.31× | +5.36 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-construct-10-compact-v2` | performance | `pure_call_sum_ns` | 38.60 ms | 39.35 ms | 0.98× | -0.75 ms | 1 | PASS | PASS |
| directory_construction_traversal | `directory-construct-100-mixed-v4` | performance | `pure_call_sum_ns` | 253.42 ms | 216.06 ms | 1.17× | +37.36 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-construct-500-mixed-v4` | performance | `pure_call_sum_ns` | 1.140 s | 1.031 s | 1.11× | +109.75 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-metadata-scan-1-compact-v2` | performance | `pure_call_sum_ns` | 87.97 ms | 70.53 ms | 1.25× | +17.44 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-metadata-scan-10-compact-v2` | performance | `pure_call_sum_ns` | 137.61 ms | 100.74 ms | 1.37× | +36.87 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-metadata-scan-100-mixed-v4` | performance | `pure_call_sum_ns` | 333.03 ms | 244.24 ms | 1.36× | +88.79 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-metadata-scan-500-mixed-v4` | performance | `pure_call_sum_ns` | 694.66 ms | 504.93 ms | 1.38× | +189.73 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-content-scan-1-compact-v2` | performance | `pure_call_sum_ns` | 105.53 ms | 84.76 ms | 1.25× | +20.77 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-content-scan-10-compact-v2` | performance | `pure_call_sum_ns` | 348.62 ms | 308.87 ms | 1.13× | +39.74 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-content-scan-100-mixed-v4` | performance | `pure_call_sum_ns` | 1.231 s | 1.105 s | 1.11× | +126.19 ms | 1 | WARN | PASS |
| directory_construction_traversal | `directory-content-scan-500-mixed-v4` | performance | `pure_call_sum_ns` | 4.434 s | 3.907 s | 1.14× | +527.55 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| workspace_change_locality | `workspace-clean-commit-1-compact-v2` | performance | `pure_call_sum_ns` | 10.50 ms | 11.32 ms | 0.93× | -0.82 ms | 1 | PASS | PASS |
| workspace_change_locality | `workspace-clean-commit-10-compact-v2` | performance | `pure_call_sum_ns` | 11.87 ms | 10.87 ms | 1.09× | +1.00 ms | 1 | PASS | PASS |
| workspace_change_locality | `workspace-clean-commit-100-mixed-v4` | performance | `pure_call_sum_ns` | 15.47 ms | 11.56 ms | 1.34× | +3.91 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-clean-commit-500-mixed-v4` | performance | `pure_call_sum_ns` | 17.80 ms | 11.87 ms | 1.50× | +5.93 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-fixed-move-1-compact-v2` | performance | `pure_call_sum_ns` | 26.56 ms | 19.71 ms | 1.35× | +6.85 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-fixed-move-10-compact-v2` | performance | `pure_call_sum_ns` | 28.83 ms | 21.32 ms | 1.35× | +7.51 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-fixed-move-100-mixed-v4` | performance | `pure_call_sum_ns` | 35.95 ms | 28.35 ms | 1.27× | +7.60 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-fixed-move-500-mixed-v4` | performance | `pure_call_sum_ns` | 47.75 ms | 27.59 ms | 1.73× | +20.16 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| workspace_change_locality | `workspace-distributed-sdk-edit-1-compact-v2` | performance | `pure_call_sum_ns` | 25.27 ms | 16.94 ms | 1.49× | +8.34 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-distributed-sdk-edit-10-compact-v2` | performance | `pure_call_sum_ns` | 42.31 ms | 39.18 ms | 1.08× | +3.13 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-distributed-sdk-edit-100-mixed-v4` | performance | `pure_call_sum_ns` | 213.09 ms | 336.40 ms | 0.63× | -123.31 ms | 1 | PASS | PASS |
| workspace_change_locality | `workspace-distributed-sdk-edit-500-mixed-v4` | performance | `pure_call_sum_ns` | 666.10 ms | 2.849 s | 0.23× | -2,183.08 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| workspace_change_locality | `workspace-dense-rewrite-1-compact-v2` | performance | `pure_call_sum_ns` | 109.38 ms | 84.12 ms | 1.30× | +25.26 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-dense-rewrite-10-compact-v2` | performance | `pure_call_sum_ns` | 583.17 ms | 316.62 ms | 1.84× | +266.55 ms | 1 | WARN | PASS |
| workspace_change_locality | `workspace-dense-rewrite-100-mixed-v4` | performance | `pure_call_sum_ns` | 2.894 s | 1.498 s | 1.93× | +1,396.44 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| workspace_change_locality | `workspace-dense-rewrite-500-mixed-v4` | performance | `pure_call_sum_ns` | 10.499 s | 5.689 s | 1.85× | +4,810.45 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| dedup_branch_history | `dedup-history-distributed-1` | performance | `pure_call_sum_ns` | 17.14 ms | 22.16 ms | 0.77× | -5.03 ms | 1 | PASS | PASS |
| dedup_branch_history | `dedup-history-distributed-10` | performance | `pure_call_sum_ns` | 69.57 ms | 82.27 ms | 0.85× | -12.69 ms | 1 | PASS | PASS |
| dedup_branch_history | `dedup-history-distributed-100` | performance | `pure_call_sum_ns` | 735.40 ms | 586.40 ms | 1.25× | +149.00 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-distributed-500` | performance | `pure_call_sum_ns` | 4.304 s | 2.925 s | 1.47× | +1,378.64 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-hotset-1` | performance | `pure_call_sum_ns` | 23.41 ms | 23.58 ms | 0.99× | -0.17 ms | 1 | PASS | PASS |
| dedup_branch_history | `dedup-history-hotset-10` | performance | `pure_call_sum_ns` | 125.41 ms | 118.46 ms | 1.06× | +6.95 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-hotset-100` | performance | `pure_call_sum_ns` | 985.36 ms | 799.50 ms | 1.23× | +185.86 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-hotset-500` | performance | `pure_call_sum_ns` | 4.914 s | 3.684 s | 1.33× | +1,230.72 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-recurring-1` | performance | `pure_call_sum_ns` | 24.06 ms | 21.46 ms | 1.12× | +2.60 ms | 1 | PASS | PASS |
| dedup_branch_history | `dedup-history-recurring-10` | performance | `pure_call_sum_ns` | 101.62 ms | 62.92 ms | 1.61× | +38.70 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-recurring-100` | performance | `pure_call_sum_ns` | 618.64 ms | 462.84 ms | 1.34× | +155.80 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-recurring-500` | performance | `pure_call_sum_ns` | 2.787 s | 2.145 s | 1.30× | +641.66 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-metadata-1` | performance | `pure_call_sum_ns` | 29.65 ms | 20.65 ms | 1.44× | +9.00 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-metadata-10` | performance | `pure_call_sum_ns` | 101.61 ms | 76.47 ms | 1.33× | +25.14 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-metadata-100` | performance | `pure_call_sum_ns` | 918.53 ms | 649.61 ms | 1.41× | +268.92 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-metadata-500` | performance | `pure_call_sum_ns` | 4.442 s | 3.232 s | 1.37× | +1,210.06 ms | 1 | WARN | PASS |
| dedup_branch_history | `dedup-history-unrelated-1` | performance | `pure_call_sum_ns` | 453.82 ms | 880.16 ms | 0.52× | -426.34 ms | 1 | PASS | PASS |
| dedup_branch_history | `dedup-history-unrelated-10` | performance | `pure_call_sum_ns` | 5.666 s | 9.543 s | 0.59× | -3,877.58 ms | 1 | PASS | PASS |
| dedup_branch_history | `dedup-history-unrelated-100-mixed-v2` | performance | `pure_call_sum_ns` | 3.190 s | 3.550 s | 0.90× | -359.86 ms | 1 | PASS | PASS |
| dedup_branch_history | `dedup-history-unrelated-500-mixed-v2` | performance | `pure_call_sum_ns` | 16.107 s | 18.164 s | 0.89× | -2,057.08 ms | 1 | **FAIL** | PASS |
| git_tool_workflow | `git-tool-1-compact-v2` | performance | `pure_call_sum_ns` | 337.08 ms | 318.66 ms | 1.06× | +18.43 ms | 1 | WARN | PASS |
| git_tool_workflow | `git-tool-10-compact-v2` | performance | `pure_call_sum_ns` | 705.41 ms | 611.38 ms | 1.15× | +94.02 ms | 1 | WARN | PASS |
| git_tool_workflow | `git-tool-100-mixed-v4` | performance | `pure_call_sum_ns` | 2.838 s | 1.879 s | 1.51× | +959.07 ms | 1 | WARN | PASS |
| git_tool_workflow | `git-tool-500-mixed-v4` | performance | `pure_call_sum_ns` | 8.803 s | 4.689 s | 1.88× | +4,113.73 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| mixed_load_bearing | `agent-episodes-1-compact-v2` | performance | `pure_call_sum_ns` | 34.38 ms | 26.24 ms | 1.31× | +8.14 ms | 1 | WARN | PASS |
| mixed_load_bearing | `agent-episodes-10-compact-v2` | performance | `pure_call_sum_ns` | 62.13 ms | 89.91 ms | 0.69× | -27.79 ms | 1 | PASS | PASS |
| mixed_load_bearing | `agent-episodes-100` | performance | `pure_call_sum_ns` | 1.030 s | 908.41 ms | 1.13× | +121.67 ms | 1 | WARN | PASS |
| mixed_load_bearing | `agent-episodes-500` | performance | `pure_call_sum_ns` | 8.219 s | 7.535 s | 1.09× | +683.31 ms | 1 | **REUSED-FROM** fsync-qualified remaining-shared | reused |
| workspace_reliability | `workspace-invalid-sdk-edit-compact-v2-proof` | proof-only | — | 2.41 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-invalid-namespace-compact-v2-proof` | proof-only | — | 2.43 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-lease-lifecycle-compact-v2-proof` | proof-only | — | 2.57 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-open-writer-busy-compact-v2-proof` | proof-only | — | 2.72 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-live-execution-busy-compact-v2-proof` | proof-only | — | 2.36 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-candidate-failure-retry-compact-v2-proof` | proof-only | — | 2.38 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-admission-batch-failure-retry-compact-v2-proof` | proof-only | — | 3.51 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-final-publication-failure-retry-compact-v2-proof` | proof-only | — | 3.95 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-published-presentation-failure-smoke-v3-proof` | proof-only | — | 2.16 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-dirty-end-discard-compact-v2-proof` | proof-only | — | 2.31 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-dirty-net-zero-compact-v2-proof` | proof-only | — | 2.51 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-short-spool-write-compact-v2-proof` | proof-only | — | 2.34 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-deferred-nospace-compact-v2-proof` | proof-only | — | 1.96 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-workload-cancel-compact-v2-proof` | proof-only | — | 2.75 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-dirty-runtime-disconnect-compact-v2-proof` | proof-only | — | 2.61 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-corrupt-descendant-compact-v2-proof` | proof-only | — | 2.16 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-missing-descendant-compact-v2-proof` | proof-only | — | 2.09 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-parallel-read-write-compact-v2-proof` | proof-only | — | 2.20 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-shared-path-contention-compact-v2-proof` | proof-only | — | 2.16 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-hardlink-alias-compact-v2-proof` | proof-only | — | 2.44 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-symlink-semantics-compact-v2-proof` | proof-only | — | 2.39 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-open-rename-unlink-compact-v2-proof` | proof-only | — | 2.26 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-metadata-chmod-compact-v2-proof` | proof-only | — | 2.49 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-metadata-mtime-compact-v2-proof` | proof-only | — | 2.24 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-metadata-xattr-compact-v2-proof` | proof-only | — | 2.01 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-exec-500-compact-v2-proof` | proof-only | — | 8.90 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-repeat-publication-compact-v2-proof` | proof-only | — | 2.41 s wall | — | — | — | — | **PASS** | PASS |
| workspace_reliability | `workspace-sustained-600s-compact-v2-proof` | proof-only | — | — | — | — | — | — | **NOT_RUN_OPTIONAL** (campaign `long_test_exclusion`) | exception.json |


## 3. Reused evidence (cited, not re-collected)

| Item | Producing treatment | Value | Disposition |
|---|---|---|---|
| Ordinary full157 stride-1 construction | final treatment `b5f089eb…` | complete command 763.218 s, 157/157 steps PASS | REUSED (mechanism unchanged; `3e308a8f2` neutral) |
| full157 same-Store verification | final treatment | 735.749 s; 157 states / 904,143 entries / 4,936,693,030 B, all oracles PASS | REUSED |
| Physical census | final treatment | allocated 83,951,616 B; apparent 82,583,552 B; pack rows 1,058; `store_sha256 88b4fe70…` | REUSED |
| Git157 read-only control | final treatment | 157 checkpoints, mapping verified; allocation-layout WARN retained (56,197,120 vs recorded 56,373,248 B) | REUSED |
| Historical access 11 performance + 11 proofs | final treatment | performance outer walls 2.525–3.232 s, verification 2.48–2.87 s, all 22 PASS, all inside the 15 s envelope (Tier-1 gate PASS), cleanups PASS | REUSED |
| Default-budget K32000 route | final treatment | COMPLETE in one Commit; 32,000 edits / 92,821 pieces / charge 2,048,000 (= 32,000 × 64) under the unchanged 2 MiB budget; harness TARGET_MISS vs its historical 15 s family target recorded as the #118 WARN (reporting-only target); independent verification PASS (32,000 changed files byte-compared pre/post reopen, 68.576 s inside the 600 s scaled allowance) | REUSED |
| K6000 boundary + default-budget frontier proof | final treatment | K6000 PASS (command 14.61 s); frontier `production_budget_spill_boundaries_stay_bounded` PASS at exactly 2B (batch=15873 count=31746 flushes=2), 4B and 8B | REUSED |
| 16 registered cells (§2 table) | `6693224e…` affected-rerun (3 SDK-edit cells, post-#116) and `fsync-qualified` `440ae2c4…` (13 cells, pre-#107/#116) | see §2 rows | REUSED-FROM with treatment identity per row |

Deviations recorded honestly: (a) the two store-footprint cells that appear in
the #120 reuse list were **re-collected fresh** because #107 changes exactly
their measured quantity (pack layout); the effect was material (unique-100000
allocated 530,358,272 → 520,142,848 B, −1.9 %) and the old receipts remain
cited as history; (b) one archived-binary custody observation: the
`binary-archive/6693224e…` directory's `identity.json` (product seal
`0e5daa12…`) does not match the #118 affected-rerun's recorded product seal
(`08176eca…`) although the binary bytes hash to the same `6693224e…`; the
diagnostic attribution run used the archive copy with its seal-consistent image
(`6ccad14c…`), and because the bytes are identical the attribution is
byte-determined; recorded so the archive discrepancy is not silent.

## 4. Bug ledger

| Severity | Evidence | Reproducer | Root cause | Fix commit | Impact-set re-run | Disposition |
|---|---|---|---|---|---|---|
| **S2** (Tier-1 gate miss) | `performance/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/perf.jsonl` (16.107 s vs `< 15 s`; complete command 19.057 s; cleanup PASS) | the campaign receipt + diagnostic run `diagnostics/unrelated-500-on-6693224e/perf.jsonl` (15.772 s on the #116-without-#107 binary) | 500 × (exec 12.42→15.30 ms, commit 15.14→16.88 ms): #116 bounded pending ≈ +1.98 s (dominant; +2.49 ms/exec, +1.46 ms/commit), #107 pack coalescing ≈ +0.33 s; introduced when #116 landed, undetected because #118's affected-rerun covered only the 3 SDK-edit cells | none (not a minimal-fix candidate: would require optimizing the owner-required #116 compact-form exec path or removing its semantics) | none yet — if a repair is directed: this cell + the SDK-edit and workspace-locality cells sharing the pending path + full157 + census + access per the fix rules | **FAIL — unrepaired**; owner decision requested (waive vs repair); [unrelated-500-tier1-rca.md](unrelated-500-tier1-rca.md) |

No S0, no S1, no H-class finding occurred during the campaign: 182/182 fresh
performance samples COMPLETE, 182/182 fresh proofs PASS, 28/28 runnable
proof-only proofs PASS, every cleanup PASS, no custody/identity failures, no
resource-bound breaches.

## 5. Campaign-wide distribution (context for #112 — never a gate)

Across the 198 comparable registered cells (fresh + reused values vs published
v0.1.3): **139 of 198 cells ≥ 15 % slower**, median ratio **1.34×**, total added
time **+33.115 s**. Worst ratios: dedup-cdc-scattered-100 3.45×, cross-file
unique-100 3.35×, namespace-100 3.22×. Largest absolute deltas: unrelated-500
reused dense-rewrite-500 +4.81 s (reused value), fresh dense-rewrite group
+25–267 ms, fresh 500-tier history cells +0.64–1.38 s, git-tool-100 +0.96 s,
scattered-500 +0.94 s. Faster-than-v0.1.3: payload-create-* (0.79–1.02×),
dedup-workspace exact/local-100 (0.62–0.66×), distributed-sdk-edit-100/500
(0.63×/0.23×), unrelated-1/10/100 (0.52–0.90×). Per-commit means on the
candidate for #108: distributed-500 6.22 ms, recurring-500 4.03 ms, hotset-500
4.51 ms, metadata-500 4.67 ms (exec 4.18 ms), unrelated-500 16.88 ms (exec
15.30 ms) — against #108's v0.1.3 targets of 3.4/2.8 ms on distributed/recurring.

## 6. Explicit gaps and the finalization condition

1. `workspace-sustained-600s-compact-v2-proof` — **NOT_RUN_OPTIONAL**, excluded
   by the frozen campaign declaration (`long_test_exclusion`; optional long
   test, no endurance qualification; the other five extended reliability
   members all ran and PASSed). Explicit gap; not silently omitted.
2. `dedup-history-unrelated-500-mixed-v2` — **FAIL — unrepaired** (S2). This is
   the single unmet gate for closing #120: it needs either a written owner
   waiver (exact target, measured value, scope, reason, residual risk) or a
   directed repair with its impact-set qualification. Everything else in the
   registered campaign is terminal.
3. The two store-footprint reuse-list cells were re-collected (§3a) — a recorded
   deviation from the literal reuse list, made because the reuse condition
   (mechanism unchanged) fails for them; both old receipts remain cited.
4. Endurance (600 s sustained proof) remains unqualified for v0.1.5, as
   declared by the campaign.

**#120 finalization condition:** every registered selection has a terminal
disposition (198/198 performance, 29/29 proof-only) and no S0/S1 is open —
**except** the single S2 Tier-1 miss above, which requires an owner decision
before #120 can close. The precise unmet gate: *`dedup-history-unrelated-500-
mixed-v2` family timer 16.107 s against the registered `unrelated-history500
< 15 s` target — awaiting owner waiver or directed repair.*

## 7. Honest linked-issue updates

Posted as comments on #102, #108, #112, #114 with this campaign's measured
results; none is closed on this campaign's evidence alone — each keeps its
precise remaining gate (see the comments and §5).

## 8. Comparison against the earlier v0.1.5 baselines

The earlier v0.1.5-line baselines are the #104 campaign (2026-09-10,
promoted-uncompacted schema10), the #110 requalification (same day, source
`cc8025fcd` = the #109 Init-reduction candidate) and the #118 treatments
(fsync-qualified `f8fa59fab`, `6693224e…` = +#116, final treatment
`b5f089eb…` = +#107). The #120 candidate is that lineage **plus fsync batching,
#116 bounded pending, #107 pack coalescing and the behavior-neutral `3e308a8f2`**.
All comparisons are single unpaired samples, matched on exact family + case +
identical timer; reused values are marked and are not candidate measurements.

**Candidate vs #104 (198/198 cells matched; fresh-only n=182):** median ratio
**0.993**; ≥15 % slower **29 fresh** (31 including reused); ≥15 % faster **38
fresh** (40 including reused). For context, the #110 requalification recorded
median 1.013 vs #104 (30 slower / 11 faster) — the candidate is a slight net
improvement over the #110-era position relative to #104. Against the common
v0.1.3 reference the candidate is also modestly better than #110: median 1.34×
vs 1.379×, 139/198 vs 154/198 cells ≥15 % slower.

**Where the candidate improved vs #104** (fsync batching + #116 paying off):
`workspace-distributed-sdk-edit-500` 4.131 → **0.666 s (0.16×)** and `-100`
0.45× (the #116 bounded-pending win); the unrelated-history cells 0.43×–0.68×
(`unrelated-500` 16.107 vs 23.781 s); `agent-episodes-10` 0.50×;
`overwrite-middle-4k-on-500mib` 0.68×. Ten of sixteen family medians are ≤ 1.0
(best: `dedup_branch_history` 0.862, `mixed_load_bearing` 0.896,
`namespace_mutation` 0.920).

**Where the candidate regressed vs #104** (the measured fixed per-iteration
costs): the worst are ms-scale cells — `payload-random-read-1` 1.41× (+6 ms),
`chunk-count-decrease-on-100mib` 1.37×, `tiny-stat-10` 1.34× (+7 ms),
`namespace-100` 1.32× (+7 ms), `delete-middle-4k-on-500mib` 1.33× — consistent
with the #116/#107 fixed costs (+2.49 ms/exec, +1.46 ms/commit, ≈+0.4 ms/phase)
measured in the unrelated-500 RCA. Six family medians sit between 1.01× and
1.05×. (`git-tool-500` at 1.25× is the reused fsync-qualified value, not a
candidate measurement.)

**Tracked cells across the v0.1.5 chain:**

| Cell | #104 | #110 | fsync-qualified | #120 candidate |
|---|---:|---:|---:|---:|
| namespace-100000 Init (cold) | 4.841 s | 3.759 s | — | 4.398 s (0.91× #104; +0.64 s of the #109 win given back; 2.7 s target owner-waived) |
| unrelated-500 | 23.781 s | 23.042 s | 13.791 s | 16.107 s (0.68× #104; +2.32 s vs the fsync peak → the §4 FAIL) |
| distributed-500 | 5.249 s | 4.191 s | — | 4.304 s |
| hotset-500 | 5.758 s | 4.905 s | — | 4.914 s |
| store-footprint-unique-100000 (timer / allocated) | 5.042 s | 3.787 s | 3.962 s / 530,358,272 B | 5.397 s / 520,142,848 B (timer +36 % vs fsync-qualified = #107 pack-row UPDATE cost on one giant commit; **footprint −1.9 %**) |
| distributed-sdk-edit-500 | 4.131 s | 3.239 s | — | 0.666 s |
| dedup-cdc-scattered-100 | 306.3 ms | 380.3 ms | — | 308.3 ms (back at the #104 level) |

**One-line summary:** against the last full v0.1.5 baseline the candidate is
neutral-to-slightly-better in aggregate (median 0.993×, more cells faster than
slower), with the #116/#107 mechanisms trading large wins on SDK-edit-heavy and
capacity-bound cells and a −1.9 % store-footprint reduction against small fixed
per-iteration costs on ms-scale cells — plus the one material regression:
`unrelated-500` giving back 2.32 s of the fsync-qualified repair (the §4 S2,
owner decision pending).

Caveats: single unpaired samples across treatments that differ by three real
mechanisms; reused values marked; #110's per-case data was removed in the
#117-era cleanup, so only its recorded summary and the worst-case values quoted
in #112 survive for the middle column.

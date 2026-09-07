# v0.1.3 benchmark checkpoint

Status: **PASS**.

One fixed-seed observation per case. Timers retain their family-specific scopes. Setup and verification are separate. Across-case ranges are not latency distributions. [JSON report](report.json), [performance CSV](performance.csv), and [verification CSV](verification.csv) contain phases, identities, resources, coverage and evidence hashes.

| Family | Performance cases | Proof-only definitions | Performance outcomes | Verification outcomes | Target misses | Across-case range (ms) |
|---|---:|---:|---|---|---:|---|
| dedup_branch_history | 20 | 0 | {'PASS': 20} | {'PASS': 20} | 1 | 20.653–18163.889 |
| dedup_cdc_locality | 20 | 1 | {'PASS': 20} | {'PASS': 21} | 0 | 3.897–483.007 |
| dedup_cross_file | 10 | 0 | {'PASS': 10} | {'PASS': 10} | 0 | 3.808–541.992 |
| dedup_workspace_reuse | 14 | 0 | {'PASS': 14} | {'PASS': 14} | 0 | 27.475–4319.934 |
| directory_construction_traversal | 12 | 0 | {'PASS': 12} | {'PASS': 12} | 0 | 17.022–3906.595 |
| edit_canonical_chunk_count | 12 | 0 | {'PASS': 12} | {'PASS': 12} | 0 | 6.855–8.507 |
| edit_length_changing | 32 | 0 | {'PASS': 32} | {'PASS': 32} | 0 | 5.290–18.011 |
| edit_length_preserving | 12 | 0 | {'PASS': 12} | {'PASS': 12} | 0 | 5.777–9.378 |
| git_tool_workflow | 4 | 0 | {'PASS': 4} | {'PASS': 4} | 2 | 318.658–4689.306 |
| init_namespace | 4 | 0 | {'PASS': 4} | {'PASS': 4} | 0 | 8.902–2603.162 |
| mixed_load_bearing | 4 | 0 | {'PASS': 4} | {'PASS': 4} | 0 | 26.241–7535.401 |
| namespace_mutation | 4 | 0 | {'PASS': 4} | {'PASS': 4} | 0 | 21.311–182.894 |
| payload_create_read | 8 | 0 | {'PASS': 8} | {'PASS': 8} | 0 | 14.585–3068.250 |
| store_footprint | 6 | 0 | {'PASS': 6} | {'PASS': 6} | 0 | 31.948–4570.430 |
| tiny_file_churn | 20 | 0 | {'PASS': 20} | {'PASS': 20} | 0 | 15.834–5054.054 |
| workspace_change_locality | 16 | 0 | {'PASS': 16} | {'PASS': 16} | 0 | 10.873–5688.592 |
| workspace_reliability | 0 | 28 | {} | {'PASS': 27, 'EXCLUDED_LONG': 1} | 0 | —–— |

## dedup_branch_history

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [dedup-history-distributed-1](raw/performance/dedup_branch_history/dedup-history-distributed-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 22.163 | — / 6.799 / 4.076 | 11.86 / 5.14 | 15.778 | PASS / PASS | PASS |
| [dedup-history-distributed-10](raw/performance/dedup_branch_history/dedup-history-distributed-10/perf.jsonl.gz) | 10 / 200 / 1048576 | pure_call_sum_ns | 82.266 | — / 31.503 / 35.981 | 12.73 / 5.14 | 57.151 | PASS / PASS | PASS |
| [dedup-history-distributed-100](raw/performance/dedup_branch_history/dedup-history-distributed-100/perf.jsonl.gz) | 100 / 200 / 1048576 | pure_call_sum_ns | 586.405 | — / 238.382 / 337.385 | 15.08 / 11.10 | 397.228 | PASS / PASS | PASS |
| [dedup-history-distributed-500](raw/performance/dedup_branch_history/dedup-history-distributed-500/perf.jsonl.gz) | 500 / 200 / 1048576 | pure_call_sum_ns | 2925.218 | — / 1221.637 / 1692.532 | 23.62 / 41.85 | 2073.074 | PASS / PASS | PASS |
| [dedup-history-hotset-1](raw/performance/dedup_branch_history/dedup-history-hotset-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 23.580 | — / 6.857 / 4.553 | 12.22 / 4.65 | 17.330 | PASS / PASS | PASS |
| [dedup-history-hotset-10](raw/performance/dedup_branch_history/dedup-history-hotset-10/perf.jsonl.gz) | 10 / 200 / 1048576 | pure_call_sum_ns | 118.461 | — / 64.620 / 39.226 | 12.89 / 5.38 | 104.038 | PASS / PASS | PASS |
| [dedup-history-hotset-100](raw/performance/dedup_branch_history/dedup-history-hotset-100/perf.jsonl.gz) | 100 / 200 / 1048576 | pure_call_sum_ns | 799.500 | — / 508.240 / 280.116 | 15.25 / 11.98 | 771.554 | PASS / PASS | PASS |
| [dedup-history-hotset-500](raw/performance/dedup_branch_history/dedup-history-hotset-500/perf.jsonl.gz) | 500 / 200 / 1048576 | pure_call_sum_ns | 3683.755 | — / 2293.414 / 1379.460 | 23.48 / 44.62 | 3657.148 | PASS / PASS | PASS |
| [dedup-history-recurring-1](raw/performance/dedup_branch_history/dedup-history-recurring-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 21.456 | — / 6.661 / 4.544 | 12.48 / 4.64 | 16.680 | PASS / PASS | PASS |
| [dedup-history-recurring-10](raw/performance/dedup_branch_history/dedup-history-recurring-10/perf.jsonl.gz) | 10 / 200 / 1048576 | pure_call_sum_ns | 62.918 | — / 20.888 / 32.855 | 13.02 / 4.65 | 68.660 | PASS / PASS | PASS |
| [dedup-history-recurring-100](raw/performance/dedup_branch_history/dedup-history-recurring-100/perf.jsonl.gz) | 100 / 200 / 1048576 | pure_call_sum_ns | 462.841 | — / 151.687 / 302.302 | 13.89 / 4.88 | 498.935 | PASS / PASS | PASS |
| [dedup-history-recurring-500](raw/performance/dedup_branch_history/dedup-history-recurring-500/perf.jsonl.gz) | 500 / 200 / 1048576 | pure_call_sum_ns | 2145.416 | — / 712.461 / 1421.468 | 15.33 / 5.37 | 2188.923 | PASS / PASS | PASS |
| [dedup-history-metadata-1](raw/performance/dedup_branch_history/dedup-history-metadata-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 20.653 | 6.847 / — / 2.914 | 10.44 / 4.34 | 21.833 | PASS / PASS | PASS |
| [dedup-history-metadata-10](raw/performance/dedup_branch_history/dedup-history-metadata-10/perf.jsonl.gz) | 10 / 200 / 1048576 | pure_call_sum_ns | 76.474 | 40.258 / — / 26.112 | 10.77 / 4.68 | 72.656 | PASS / PASS | PASS |
| [dedup-history-metadata-100](raw/performance/dedup_branch_history/dedup-history-metadata-100/perf.jsonl.gz) | 100 / 200 / 1048576 | pure_call_sum_ns | 649.605 | 384.356 / — / 255.545 | 11.78 / 5.66 | 654.388 | PASS / PASS | PASS |
| [dedup-history-metadata-500](raw/performance/dedup_branch_history/dedup-history-metadata-500/perf.jsonl.gz) | 500 / 200 / 1048576 | pure_call_sum_ns | 3231.967 | 1946.106 / — / 1275.062 | 13.50 / 4.89 | 2935.202 | PASS / PASS | PASS |
| [dedup-history-unrelated-1](raw/performance/dedup_branch_history/dedup-history-unrelated-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 880.164 | 840.214 / — / 28.888 | 16.91 / 8.09 | 1107.731 | PASS / PASS | PASS |
| [dedup-history-unrelated-10](raw/performance/dedup_branch_history/dedup-history-unrelated-10/perf.jsonl.gz) | 10 / 200 / 1048576 | pure_call_sum_ns | 9543.475 | 9181.185 / — / 350.191 | 33.98 / 15.20 | 11029.951 | PASS / PASS | PASS |
| [dedup-history-unrelated-100-mixed-v2](raw/performance/dedup_branch_history/dedup-history-unrelated-100-mixed-v2/perf.jsonl.gz) | 100 / 10 / 1048576 | pure_call_sum_ns | 3549.609 | 2741.061 / — / 798.293 | 56.53 / 8.34 | 3492.535 | PASS / PASS | PASS |
| [dedup-history-unrelated-500-mixed-v2](raw/performance/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/perf.jsonl.gz) | 500 / 10 / 1048576 | pure_call_sum_ns | 18163.889 | 13835.773 / — / 4317.425 | 65.25 / 9.94 | 16997.841 | PASS / PASS | TARGET_MISS |

## dedup_cdc_locality

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [dedup-cdc-overwrite-1](raw/performance/dedup_cdc_locality/dedup-cdc-overwrite-1/perf.jsonl) | 1 / 2 / 2097152 | pure_call_sum_ns | 4.002 | — / — / — | 11.52 / 4.64 | 4.091 | PASS / PASS | PASS |
| [dedup-cdc-overwrite-10](raw/performance/dedup_cdc_locality/dedup-cdc-overwrite-10/perf.jsonl) | 10 / 11 / 11534336 | pure_call_sum_ns | 20.448 | — / — / — | 12.45 / 4.94 | 18.876 | PASS / PASS | PASS |
| [dedup-cdc-overwrite-100](raw/performance/dedup_cdc_locality/dedup-cdc-overwrite-100/perf.jsonl) | 100 / 101 / 105906176 | pure_call_sum_ns | 30.953 | — / — / — | 20.31 / 4.67 | 31.456 | PASS / PASS | PASS |
| [dedup-cdc-overwrite-500](raw/performance/dedup_cdc_locality/dedup-cdc-overwrite-500/perf.jsonl) | 500 / 501 / 525336576 | pure_call_sum_ns | 122.035 | — / — / — | 38.91 / 4.90 | 133.688 | PASS / PASS | PASS |
| [dedup-cdc-insert-1](raw/performance/dedup_cdc_locality/dedup-cdc-insert-1/perf.jsonl) | 1 / 2 / 2101248 | pure_call_sum_ns | 3.897 | — / — / — | 11.48 / 4.82 | 4.784 | PASS / PASS | PASS |
| [dedup-cdc-insert-10](raw/performance/dedup_cdc_locality/dedup-cdc-insert-10/perf.jsonl) | 10 / 11 / 11575296 | pure_call_sum_ns | 18.058 | — / — / — | 13.03 / 4.41 | 19.287 | PASS / PASS | PASS |
| [dedup-cdc-insert-100](raw/performance/dedup_cdc_locality/dedup-cdc-insert-100/perf.jsonl) | 100 / 101 / 106315776 | pure_call_sum_ns | 34.183 | — / — / — | 26.64 / 4.65 | 34.634 | PASS / PASS | PASS |
| [dedup-cdc-insert-500](raw/performance/dedup_cdc_locality/dedup-cdc-insert-500/perf.jsonl) | 500 / 501 / 527384576 | pure_call_sum_ns | 136.776 | — / — / — | 45.80 / 4.83 | 137.175 | PASS / PASS | PASS |
| [dedup-cdc-delete-1](raw/performance/dedup_cdc_locality/dedup-cdc-delete-1/perf.jsonl) | 1 / 2 / 2093056 | pure_call_sum_ns | 4.412 | — / — / — | 11.81 / 4.90 | 4.703 | PASS / PASS | PASS |
| [dedup-cdc-delete-10](raw/performance/dedup_cdc_locality/dedup-cdc-delete-10/perf.jsonl) | 10 / 11 / 11493376 | pure_call_sum_ns | 18.225 | — / — / — | 13.12 / 4.89 | 19.179 | PASS / PASS | PASS |
| [dedup-cdc-delete-100](raw/performance/dedup_cdc_locality/dedup-cdc-delete-100/perf.jsonl) | 100 / 101 / 105496576 | pure_call_sum_ns | 33.166 | — / — / — | 25.55 / 5.16 | 33.176 | PASS / PASS | PASS |
| [dedup-cdc-delete-500](raw/performance/dedup_cdc_locality/dedup-cdc-delete-500/perf.jsonl) | 500 / 501 / 523288576 | pure_call_sum_ns | 130.864 | — / — / — | 42.31 / 5.66 | 142.975 | PASS / PASS | PASS |
| [dedup-cdc-common-body-1](raw/performance/dedup_cdc_locality/dedup-cdc-common-body-1/perf.jsonl) | 1 / 2 / 2097152 | pure_call_sum_ns | 4.125 | — / — / — | 11.97 / 4.15 | 4.808 | PASS / PASS | PASS |
| [dedup-cdc-common-body-10](raw/performance/dedup_cdc_locality/dedup-cdc-common-body-10/perf.jsonl) | 10 / 11 / 11534336 | pure_call_sum_ns | 20.395 | — / — / — | 19.03 / 4.64 | 20.677 | PASS / PASS | PASS |
| [dedup-cdc-common-body-100](raw/performance/dedup_cdc_locality/dedup-cdc-common-body-100/perf.jsonl) | 100 / 101 / 105906176 | pure_call_sum_ns | 58.208 | — / — / — | 58.80 / 4.39 | 57.155 | PASS / PASS | PASS |
| [dedup-cdc-common-body-500](raw/performance/dedup_cdc_locality/dedup-cdc-common-body-500/perf.jsonl) | 500 / 501 / 525336576 | pure_call_sum_ns | 244.350 | — / — / — | 61.23 / 4.65 | 254.522 | PASS / PASS | PASS |
| [dedup-cdc-scattered-1](raw/performance/dedup_cdc_locality/dedup-cdc-scattered-1/perf.jsonl) | 1 / 2 / 2097152 | pure_call_sum_ns | 4.647 | — / — / — | 13.80 / 4.89 | 4.750 | PASS / PASS | PASS |
| [dedup-cdc-scattered-10](raw/performance/dedup_cdc_locality/dedup-cdc-scattered-10/perf.jsonl) | 10 / 11 / 11534336 | pure_call_sum_ns | 23.469 | — / — / — | 31.98 / 5.14 | 24.154 | PASS / PASS | PASS |
| [dedup-cdc-scattered-100](raw/performance/dedup_cdc_locality/dedup-cdc-scattered-100/perf.jsonl) | 100 / 101 / 105906176 | pure_call_sum_ns | 89.368 | — / — / — | 58.23 / 4.66 | 91.017 | PASS / PASS | PASS |
| [dedup-cdc-scattered-500](raw/performance/dedup_cdc_locality/dedup-cdc-scattered-500/perf.jsonl) | 500 / 501 / 525336576 | pure_call_sum_ns | 483.007 | — / — / — | 60.69 / 4.86 | 452.848 | PASS / PASS | PASS |

## dedup_cross_file

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [dedup-cross-file-anchor-1](raw/performance/dedup_cross_file/dedup-cross-file-anchor-1/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 3.808 | — / — / — | 10.89 / 4.64 | 4.182 | PASS / PASS | PASS |
| [dedup-cross-file-unique-10](raw/performance/dedup_cross_file/dedup-cross-file-unique-10/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 22.390 | — / — / — | 30.64 / 4.66 | 24.261 | PASS / PASS | PASS |
| [dedup-cross-file-unique-100](raw/performance/dedup_cross_file/dedup-cross-file-unique-100/perf.jsonl) | 100 / 100 / 104857600 | pure_call_sum_ns | 93.267 | — / — / — | 59.36 / 4.39 | 106.949 | PASS / PASS | PASS |
| [dedup-cross-file-unique-500](raw/performance/dedup_cross_file/dedup-cross-file-unique-500/perf.jsonl) | 500 / 500 / 524288000 | pure_call_sum_ns | 444.949 | — / — / — | 61.84 / 4.90 | 458.246 | PASS / PASS | PASS |
| [dedup-cross-file-identical-10](raw/performance/dedup_cross_file/dedup-cross-file-identical-10/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 18.395 | — / — / — | 11.58 / 4.40 | 18.808 | PASS / PASS | PASS |
| [dedup-cross-file-identical-100](raw/performance/dedup_cross_file/dedup-cross-file-identical-100/perf.jsonl) | 100 / 100 / 104857600 | pure_call_sum_ns | 28.960 | — / — / — | 14.36 / 4.65 | 29.554 | PASS / PASS | PASS |
| [dedup-cross-file-identical-500](raw/performance/dedup_cross_file/dedup-cross-file-identical-500/perf.jsonl) | 500 / 500 / 524288000 | pure_call_sum_ns | 110.630 | — / — / — | 16.22 / 4.91 | 111.170 | PASS / PASS | PASS |
| [dedup-cross-file-mixed-10](raw/performance/dedup_cross_file/dedup-cross-file-mixed-10/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 24.046 | — / — / — | 26.97 / 4.92 | 25.753 | PASS / PASS | PASS |
| [dedup-cross-file-mixed-100](raw/performance/dedup_cross_file/dedup-cross-file-mixed-100/perf.jsonl) | 100 / 100 / 104857600 | pure_call_sum_ns | 113.944 | — / — / — | 63.34 / 4.39 | 113.707 | PASS / PASS | PASS |
| [dedup-cross-file-mixed-500](raw/performance/dedup_cross_file/dedup-cross-file-mixed-500/perf.jsonl) | 500 / 500 / 524288000 | pure_call_sum_ns | 541.992 | — / — / — | 66.17 / 4.59 | 535.620 | PASS / PASS | PASS |

## dedup_workspace_reuse

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [dedup-workspace-exact-1-compact-v2](raw/performance/dedup_workspace_reuse/dedup-workspace-exact-1-compact-v2/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 31.563 | 10.358 / — / 9.724 | 14.83 / 7.50 | 34.458 | PASS / PASS | PASS |
| [dedup-workspace-exact-10-compact-v2](raw/performance/dedup_workspace_reuse/dedup-workspace-exact-10-compact-v2/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 109.325 | 43.102 / — / 54.571 | 39.06 / 12.31 | 124.068 | PASS / PASS | PASS |
| [dedup-workspace-exact-100](raw/performance/dedup_workspace_reuse/dedup-workspace-exact-100/perf.jsonl) | 100 / 128 / 134217728 | pure_call_sum_ns | 840.528 | 382.532 / — / 443.740 | 69.05 / 12.25 | 1039.093 | PASS / PASS | PASS |
| [dedup-workspace-exact-500](raw/performance/dedup_workspace_reuse/dedup-workspace-exact-500/perf.jsonl) | 500 / 128 / 134217728 | pure_call_sum_ns | 4296.598 | 2422.419 / — / 1858.818 | 71.59 / 13.09 | 5001.114 | PASS / PASS | PASS |
| [dedup-workspace-local-1-compact-v2](raw/performance/dedup_workspace_reuse/dedup-workspace-local-1-compact-v2/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 30.610 | 9.273 / — / 9.223 | 16.09 / 7.37 | 32.089 | PASS / PASS | PASS |
| [dedup-workspace-local-10-compact-v2](raw/performance/dedup_workspace_reuse/dedup-workspace-local-10-compact-v2/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 103.153 | 42.554 / — / 49.849 | 40.53 / 10.24 | 118.665 | PASS / PASS | PASS |
| [dedup-workspace-local-100](raw/performance/dedup_workspace_reuse/dedup-workspace-local-100/perf.jsonl) | 100 / 128 / 134217728 | pure_call_sum_ns | 808.387 | 373.829 / — / 414.616 | 67.69 / 10.55 | 944.918 | PASS / PASS | PASS |
| [dedup-workspace-local-500](raw/performance/dedup_workspace_reuse/dedup-workspace-local-500/perf.jsonl) | 500 / 128 / 134217728 | pure_call_sum_ns | 4307.360 | 2334.955 / — / 1956.752 | 72.53 / 13.12 | 5033.169 | PASS / PASS | PASS |
| [dedup-workspace-unique-1-compact-v2](raw/performance/dedup_workspace_reuse/dedup-workspace-unique-1-compact-v2/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 27.475 | 9.833 / — / 7.920 | 14.80 / 7.51 | 29.844 | PASS / PASS | PASS |
| [dedup-workspace-unique-10-compact-v2](raw/performance/dedup_workspace_reuse/dedup-workspace-unique-10-compact-v2/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 95.669 | 43.999 / — / 40.485 | 38.86 / 10.26 | 113.608 | PASS / PASS | PASS |
| [dedup-workspace-unique-100](raw/performance/dedup_workspace_reuse/dedup-workspace-unique-100/perf.jsonl) | 100 / 128 / 134217728 | pure_call_sum_ns | 768.004 | 376.460 / — / 377.740 | 66.28 / 12.73 | 889.122 | PASS / PASS | PASS |
| [dedup-workspace-unique-500](raw/performance/dedup_workspace_reuse/dedup-workspace-unique-500/perf.jsonl) | 500 / 128 / 134217728 | pure_call_sum_ns | 4319.934 | 2313.265 / — / 1988.328 | 71.41 / 11.96 | 5078.063 | PASS / PASS | PASS |
| [dedup-workspace-unique-1-base128-v3](raw/performance/dedup_workspace_reuse/dedup-workspace-unique-1-base128-v3/perf.jsonl) | 1 / 128 / 134217728 | pure_call_sum_ns | 33.382 | 12.250 / — / 8.868 | 37.12 / 7.39 | 33.569 | PASS / PASS | PASS |
| [dedup-workspace-unique-10-base128-v3](raw/performance/dedup_workspace_reuse/dedup-workspace-unique-10-base128-v3/perf.jsonl) | 10 / 128 / 134217728 | pure_call_sum_ns | 103.007 | 49.072 / — / 43.049 | 61.64 / 10.33 | 134.105 | PASS / PASS | PASS |

## directory_construction_traversal

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [directory-construct-1-compact-v2](raw/performance/directory_construction_traversal/directory-construct-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 17.022 | 4.825 / — / 2.527 | 9.41 / 5.14 | 19.679 | PASS / PASS | PASS |
| [directory-construct-10-compact-v2](raw/performance/directory_construction_traversal/directory-construct-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 39.347 | 23.145 / — / 4.571 | 10.42 / 4.38 | 43.811 | PASS / PASS | PASS |
| [directory-construct-100-mixed-v4](raw/performance/directory_construction_traversal/directory-construct-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 216.060 | 191.301 / — / 13.697 | 13.33 / 5.30 | 243.815 | PASS / PASS | PASS |
| [directory-construct-500-mixed-v4](raw/performance/directory_construction_traversal/directory-construct-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 1030.581 | 942.614 / — / 73.472 | 29.92 / 11.01 | 1232.776 | PASS / PASS | PASS |
| [directory-metadata-scan-1-compact-v2](raw/performance/directory_construction_traversal/directory-metadata-scan-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 70.527 | 59.086 / — / 1.434 | 9.98 / 4.91 | 83.738 | PASS / PASS | PASS |
| [directory-metadata-scan-10-compact-v2](raw/performance/directory_construction_traversal/directory-metadata-scan-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 100.739 | 88.606 / — / 1.954 | 23.16 / 5.02 | 121.407 | PASS / PASS | PASS |
| [directory-metadata-scan-100-mixed-v4](raw/performance/directory_construction_traversal/directory-metadata-scan-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 244.239 | 226.651 / — / 3.172 | 51.16 / 15.91 | 229.157 | PASS / PASS | PASS |
| [directory-metadata-scan-500-mixed-v4](raw/performance/directory_construction_traversal/directory-metadata-scan-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 504.932 | 479.976 / — / 5.306 | 55.22 / 33.58 | 410.723 | PASS / PASS | PASS |
| [directory-content-scan-1-compact-v2](raw/performance/directory_construction_traversal/directory-content-scan-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 84.760 | 73.138 / — / 1.492 | 10.42 / 6.97 | 107.854 | PASS / PASS | PASS |
| [directory-content-scan-10-compact-v2](raw/performance/directory_construction_traversal/directory-content-scan-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 308.870 | 294.491 / — / 2.817 | 24.69 / 28.52 | 379.844 | PASS / PASS | PASS |
| [directory-content-scan-100-mixed-v4](raw/performance/directory_construction_traversal/directory-content-scan-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 1105.141 | 1082.974 / — / 7.242 | 52.11 / 172.73 | 1600.999 | PASS / PASS | PASS |
| [directory-content-scan-500-mixed-v4](raw/performance/directory_construction_traversal/directory-content-scan-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 3906.595 | 3861.436 / — / 24.456 | 59.20 / 586.19 | 5306.648 | PASS / PASS | PASS |

## edit_canonical_chunk_count

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 6.855 | — / 2.256 / 4.599 | 29.64 / 4.43 | 7.344 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 8.507 | — / 2.525 / 5.981 | 29.59 / 4.65 | 7.627 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 7.335 | — / 2.053 / 5.283 | 29.59 / 4.65 | 7.847 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 8.270 | — / 2.323 / 5.947 | 29.91 / 4.64 | 9.105 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 7.852 | — / 2.482 / 5.371 | 29.66 / 4.91 | 7.253 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 7.516 | — / 2.101 / 5.415 | 29.52 / 4.64 | 7.746 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 7.870 | — / 2.351 / 5.518 | 29.42 / 4.83 | 8.519 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 8.466 | — / 2.244 / 6.222 | 29.58 / 4.90 | 8.662 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 7.175 | — / 2.428 / 4.747 | 29.50 / 4.66 | 8.122 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 7.013 | — / 2.069 / 4.944 | 29.31 / 4.59 | 7.612 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 7.582 | — / 2.204 / 5.378 | 29.55 / 4.65 | 8.010 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1](raw/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 8.095 | — / 2.213 / 5.883 | 29.56 / 4.64 | 7.998 | PASS / PASS | PASS |

## edit_length_changing

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [insert-middle-4k-on-1mib-ops-1](raw/performance/edit_length_changing/insert-middle-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 5.590 | — / 1.780 / 3.810 | 29.47 / 4.90 | 6.652 | PASS / PASS | PASS |
| [insert-middle-4k-on-10mib-ops-1](raw/performance/edit_length_changing/insert-middle-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 7.396 | — / 2.539 / 4.857 | 29.55 / 4.65 | 7.750 | PASS / PASS | PASS |
| [insert-middle-4k-on-100mib-ops-1](raw/performance/edit_length_changing/insert-middle-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 12.439 | — / 2.867 / 9.572 | 29.34 / 4.64 | 6.845 | PASS / PASS | PASS |
| [insert-middle-4k-on-500mib-result-capped-v2-ops-1](raw/performance/edit_length_changing/insert-middle-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524283904 | edit_commit_ns | 18.011 | — / 5.572 / 12.438 | 29.75 / 5.64 | 7.269 | PASS / PASS | PASS |
| [delete-middle-4k-on-1mib-ops-1](raw/performance/edit_length_changing/delete-middle-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 5.290 | — / 1.773 / 3.517 | 29.75 / 4.40 | 6.013 | PASS / PASS | PASS |
| [delete-middle-4k-on-10mib-ops-1](raw/performance/edit_length_changing/delete-middle-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 6.570 | — / 1.863 / 4.707 | 29.64 / 4.39 | 6.361 | PASS / PASS | PASS |
| [delete-middle-4k-on-100mib-ops-1](raw/performance/edit_length_changing/delete-middle-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 6.758 | — / 1.861 / 4.897 | 29.61 / 4.60 | 6.718 | PASS / PASS | PASS |
| [delete-middle-4k-on-500mib-ops-1](raw/performance/edit_length_changing/delete-middle-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 7.488 | — / 1.993 / 5.495 | 29.59 / 5.64 | 7.113 | PASS / PASS | PASS |
| [append-tail-4k-on-1mib-ops-1](raw/performance/edit_length_changing/append-tail-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 6.261 | — / 2.073 / 4.188 | 29.48 / 5.14 | 7.266 | PASS / PASS | PASS |
| [append-tail-4k-on-10mib-ops-1](raw/performance/edit_length_changing/append-tail-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 5.905 | — / 1.937 / 3.968 | 29.45 / 5.19 | 7.074 | PASS / PASS | PASS |
| [append-tail-4k-on-100mib-ops-1](raw/performance/edit_length_changing/append-tail-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 6.255 | — / 1.952 / 4.303 | 29.72 / 5.19 | 7.522 | PASS / PASS | PASS |
| [append-tail-4k-on-500mib-result-capped-v2-ops-1](raw/performance/edit_length_changing/append-tail-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524283904 | edit_commit_ns | 6.686 | — / 1.991 / 4.695 | 29.38 / 4.65 | 7.323 | PASS / PASS | PASS |
| [prepend-head-4k-on-1mib-ops-1](raw/performance/edit_length_changing/prepend-head-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 5.905 | — / 1.952 / 3.953 | 29.59 / 4.90 | 7.075 | PASS / PASS | PASS |
| [prepend-head-4k-on-10mib-ops-1](raw/performance/edit_length_changing/prepend-head-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 5.672 | — / 1.735 / 3.937 | 29.48 / 5.41 | 6.296 | PASS / PASS | PASS |
| [prepend-head-4k-on-100mib-ops-1](raw/performance/edit_length_changing/prepend-head-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 6.269 | — / 2.164 / 4.105 | 29.45 / 5.14 | 6.176 | PASS / PASS | PASS |
| [prepend-head-4k-on-500mib-result-capped-v2-ops-1](raw/performance/edit_length_changing/prepend-head-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524283904 | edit_commit_ns | 7.099 | — / 2.310 / 4.788 | 29.23 / 4.40 | 7.304 | PASS / PASS | PASS |
| [replace-grow-middle-2k-to-4k-on-1mib-ops-1](raw/performance/edit_length_changing/replace-grow-middle-2k-to-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 5.744 | — / 1.975 / 3.769 | 29.66 / 5.14 | 7.557 | PASS / PASS | PASS |
| [replace-grow-middle-2k-to-4k-on-10mib-ops-1](raw/performance/edit_length_changing/replace-grow-middle-2k-to-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 6.463 | — / 2.322 / 4.141 | 29.58 / 4.65 | 6.817 | PASS / PASS | PASS |
| [replace-grow-middle-2k-to-4k-on-100mib-ops-1](raw/performance/edit_length_changing/replace-grow-middle-2k-to-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 6.382 | — / 1.981 / 4.401 | 29.59 / 4.65 | 9.794 | PASS / PASS | PASS |
| [replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1](raw/performance/edit_length_changing/replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524285952 | edit_commit_ns | 7.922 | — / 2.165 / 5.756 | 29.31 / 5.14 | 7.902 | PASS / PASS | PASS |
| [replace-shrink-middle-4k-to-2k-on-1mib-ops-1](raw/performance/edit_length_changing/replace-shrink-middle-4k-to-2k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 5.880 | — / 2.002 / 3.878 | 29.39 / 4.89 | 6.608 | PASS / PASS | PASS |
| [replace-shrink-middle-4k-to-2k-on-10mib-ops-1](raw/performance/edit_length_changing/replace-shrink-middle-4k-to-2k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 5.770 | — / 1.780 / 3.990 | 29.59 / 4.57 | 5.983 | PASS / PASS | PASS |
| [replace-shrink-middle-4k-to-2k-on-100mib-ops-1](raw/performance/edit_length_changing/replace-shrink-middle-4k-to-2k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 7.008 | — / 2.022 / 4.985 | 29.44 / 5.39 | 6.422 | PASS / PASS | PASS |
| [replace-shrink-middle-4k-to-2k-on-500mib-ops-1](raw/performance/edit_length_changing/replace-shrink-middle-4k-to-2k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 6.997 | — / 2.044 / 4.953 | 29.28 / 4.40 | 6.912 | PASS / PASS | PASS |
| [truncate-tail-4k-on-1mib-ops-1](raw/performance/edit_length_changing/truncate-tail-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 5.293 | — / 1.930 / 3.364 | 29.39 / 4.59 | 5.454 | PASS / PASS | PASS |
| [truncate-tail-4k-on-10mib-ops-1](raw/performance/edit_length_changing/truncate-tail-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 15.641 | — / 5.806 / 9.835 | 29.64 / 4.93 | 6.264 | PASS / PASS | PASS |
| [truncate-tail-4k-on-100mib-ops-1](raw/performance/edit_length_changing/truncate-tail-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 6.528 | — / 2.088 / 4.439 | 29.80 / 4.90 | 6.847 | PASS / PASS | PASS |
| [truncate-tail-4k-on-500mib-ops-1](raw/performance/edit_length_changing/truncate-tail-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 6.895 | — / 1.913 / 4.983 | 29.66 / 4.89 | 7.534 | PASS / PASS | PASS |
| [zero-extend-tail-4k-on-1mib-ops-1](raw/performance/edit_length_changing/zero-extend-tail-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 5.633 | — / 1.914 / 3.719 | 29.61 / 4.65 | 7.085 | PASS / PASS | PASS |
| [zero-extend-tail-4k-on-10mib-ops-1](raw/performance/edit_length_changing/zero-extend-tail-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 5.638 | — / 1.845 / 3.793 | 29.52 / 5.15 | 6.263 | PASS / PASS | PASS |
| [zero-extend-tail-4k-on-100mib-ops-1](raw/performance/edit_length_changing/zero-extend-tail-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 6.404 | — / 2.307 / 4.096 | 29.73 / 4.64 | 5.945 | PASS / PASS | PASS |
| [zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1](raw/performance/edit_length_changing/zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524283904 | edit_commit_ns | 6.664 | — / 1.964 / 4.700 | 29.59 / 4.65 | 7.992 | PASS / PASS | PASS |

## edit_length_preserving

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [overwrite-head-4k-on-1mib-ops-1](raw/performance/edit_length_preserving/overwrite-head-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 5.777 | — / 2.051 / 3.726 | 29.62 / 4.59 | 6.942 | PASS / PASS | PASS |
| [overwrite-head-4k-on-10mib-ops-1](raw/performance/edit_length_preserving/overwrite-head-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 6.154 | — / 1.974 / 4.180 | 29.66 / 4.89 | 6.979 | PASS / PASS | PASS |
| [overwrite-head-4k-on-100mib-ops-1](raw/performance/edit_length_preserving/overwrite-head-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 6.652 | — / 1.960 / 4.692 | 29.72 / 4.42 | 7.292 | PASS / PASS | PASS |
| [overwrite-head-4k-on-500mib-ops-1](raw/performance/edit_length_preserving/overwrite-head-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 8.411 | — / 2.692 / 5.720 | 29.61 / 4.90 | 9.354 | PASS / PASS | PASS |
| [overwrite-middle-4k-on-1mib-ops-1](raw/performance/edit_length_preserving/overwrite-middle-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 6.139 | — / 1.897 / 4.241 | 29.64 / 4.67 | 6.264 | PASS / PASS | PASS |
| [overwrite-middle-4k-on-10mib-ops-1](raw/performance/edit_length_preserving/overwrite-middle-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 5.925 | — / 1.656 / 4.269 | 29.20 / 4.67 | 7.386 | PASS / PASS | PASS |
| [overwrite-middle-4k-on-100mib-ops-1](raw/performance/edit_length_preserving/overwrite-middle-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 6.188 | — / 1.773 / 4.415 | 29.48 / 4.42 | 6.626 | PASS / PASS | PASS |
| [overwrite-middle-4k-on-500mib-ops-1](raw/performance/edit_length_preserving/overwrite-middle-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 7.174 | — / 1.987 / 5.187 | 29.58 / 4.90 | 8.928 | PASS / PASS | PASS |
| [overwrite-tail-4k-on-1mib-ops-1](raw/performance/edit_length_preserving/overwrite-tail-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 6.112 | — / 2.018 / 4.095 | 29.58 / 5.64 | 6.207 | PASS / PASS | PASS |
| [overwrite-tail-4k-on-10mib-ops-1](raw/performance/edit_length_preserving/overwrite-tail-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 7.265 | — / 2.349 / 4.916 | 29.30 / 4.41 | 6.989 | PASS / PASS | PASS |
| [overwrite-tail-4k-on-100mib-ops-1](raw/performance/edit_length_preserving/overwrite-tail-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 7.631 | — / 2.410 / 5.221 | 29.55 / 4.90 | 8.001 | PASS / PASS | PASS |
| [overwrite-tail-4k-on-500mib-ops-1](raw/performance/edit_length_preserving/overwrite-tail-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 9.378 | — / 2.318 / 7.060 | 29.45 / 4.82 | 8.343 | PASS / PASS | PASS |

## git_tool_workflow

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [git-tool-1-compact-v2](raw/performance/git_tool_workflow/git-tool-1-compact-v2/perf.jsonl) | 1 / 51 / 1051076 | pure_call_sum_ns | 318.658 | 300.598 / — / 5.010 | 13.48 / 11.99 | — | PASS / PASS | PASS |
| [git-tool-10-compact-v2](raw/performance/git_tool_workflow/git-tool-10-compact-v2/perf.jsonl) | 10 / 207 / 4211804 | pure_call_sum_ns | 611.383 | 587.985 / — / 8.797 | 23.45 / 26.84 | — | PASS / PASS | PASS |
| [git-tool-100-mixed-v4](raw/performance/git_tool_workflow/git-tool-100-mixed-v4/perf.jsonl) | 100 / 2351 / 61491723 | pure_call_sum_ns | 1879.182 | 1830.354 / — / 27.289 | 51.16 / 70.57 | 1852.297 | PASS / PASS | TARGET_MISS |
| [git-tool-500-mixed-v4](raw/performance/git_tool_workflow/git-tool-500-mixed-v4/perf.jsonl) | 500 / 5351 / 73779723 | pure_call_sum_ns | 4689.306 | 4575.835 / — / 78.107 | 76.00 / 140.78 | 4635.514 | PASS / PASS | TARGET_MISS |

## init_namespace

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [namespace-100-compact-v3](raw/performance/init_namespace/namespace-100-compact-v3/perf.jsonl) | 100 / 100 / 5000000 | layerstack_init_ns | 8.902 | — / — / — | 21.80 / 4.40 | 10.105 | PASS / PASS | PASS |
| [namespace-1000-compact-v3](raw/performance/init_namespace/namespace-1000-compact-v3/perf.jsonl) | 1000 / 1000 / 20000000 | layerstack_init_ns | 38.077 | — / — / — | 50.89 / 4.64 | 39.891 | PASS / PASS | PASS |
| [namespace-10000](raw/performance/init_namespace/namespace-10000/perf.jsonl) | 10000 / 10000 / 300000000 | layerstack_init_ns | 403.468 | — / — / — | 67.20 / 4.42 | 402.721 | PASS / PASS | PASS |
| [namespace-100000](raw/performance/init_namespace/namespace-100000/perf.jsonl) | 100000 / 100000 / 500000000 | layerstack_init_ns | 2603.162 | — / — / — | 101.25 / 6.33 | 2734.598 | PASS / PASS | PASS |

## mixed_load_bearing

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [agent-episodes-1-compact-v2](raw/performance/mixed_load_bearing/agent-episodes-1-compact-v2/perf.jsonl) | 1 / 54 / 1081344 | pure_call_sum_ns | 26.241 | 12.084 / — / 4.346 | 9.14 / 4.65 | 29.921 | PASS / PASS | PASS |
| [agent-episodes-10-compact-v2](raw/performance/mixed_load_bearing/agent-episodes-10-compact-v2/perf.jsonl) | 10 / 540 / 10813440 | pure_call_sum_ns | 89.911 | 74.119 / — / 6.676 | 10.75 / 5.41 | 143.728 | PASS / PASS | PASS |
| [agent-episodes-100](raw/performance/mixed_load_bearing/agent-episodes-100/perf.jsonl) | 100 / 14800 / 83492864 | pure_call_sum_ns | 908.411 | 851.249 / — / 45.420 | 40.62 / 12.35 | 1449.353 | PASS / PASS | PASS |
| [agent-episodes-500](raw/performance/mixed_load_bearing/agent-episodes-500/perf.jsonl) | 500 / 14800 / 83492864 | pure_call_sum_ns | 7535.401 | 7345.760 / — / 172.414 | 74.94 / 40.68 | 10585.122 | PASS / PASS | PASS |

## namespace_mutation

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [namespace-subtree-relocate-delete-1-compact-v2](raw/performance/namespace_mutation/namespace-subtree-relocate-delete-1-compact-v2/perf.jsonl) | 1 / 250 / 551200 | pure_call_sum_ns | 21.311 | 8.515 / — / 3.151 | 9.78 / 4.93 | 23.850 | PASS / PASS | PASS |
| [namespace-subtree-relocate-delete-10-compact-v2](raw/performance/namespace_mutation/namespace-subtree-relocate-delete-10-compact-v2/perf.jsonl) | 10 / 700 / 1012000 | pure_call_sum_ns | 59.145 | 44.220 / — / 4.099 | 11.30 / 4.66 | 69.609 | PASS / PASS | PASS |
| [namespace-subtree-relocate-delete-100-mixed-v4](raw/performance/namespace_mutation/namespace-subtree-relocate-delete-100-mixed-v4/perf.jsonl) | 100 / 2400 / 105267200 | pure_call_sum_ns | 51.018 | 36.126 / — / 4.866 | 12.84 / 4.91 | 42.300 | PASS / PASS | PASS |
| [namespace-subtree-relocate-delete-500-mixed-v4](raw/performance/namespace_mutation/namespace-subtree-relocate-delete-500-mixed-v4/perf.jsonl) | 500 / 7000 / 526336000 | pure_call_sum_ns | 182.894 | 161.252 / — / 9.644 | 21.02 / 5.74 | 208.678 | PASS / PASS | PASS |

## payload_create_read

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [payload-create-1m-compact-v2](raw/performance/payload_create_read/payload-create-1m-compact-v2/perf.jsonl) | 1 / 0 / 0 | pure_call_sum_ns | 28.837 | 8.517 / — / 8.805 | 13.89 / 8.34 | 42.039 | PASS / PASS | PASS |
| [payload-create-10m-compact-v2](raw/performance/payload_create_read/payload-create-10m-compact-v2/perf.jsonl) | 10 / 0 / 0 | pure_call_sum_ns | 91.556 | 28.729 / — / 50.851 | 35.30 / 9.98 | 90.331 | PASS / PASS | PASS |
| [payload-create-100m](raw/performance/payload_create_read/payload-create-100m/perf.jsonl) | 100 / 0 / 0 | pure_call_sum_ns | 682.771 | 240.635 / — / 428.711 | 62.28 / 11.20 | 723.611 | PASS / PASS | PASS |
| [payload-create-500m](raw/performance/payload_create_read/payload-create-500m/perf.jsonl) | 500 / 0 / 0 | pure_call_sum_ns | 3068.250 | 1112.949 / — / 1941.935 | 70.89 / 12.38 | 3368.999 | PASS / PASS | PASS |
| [payload-random-read-1-compact-v2](raw/performance/payload_create_read/payload-random-read-1-compact-v2/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 14.585 | 3.195 / — / 1.379 | 8.55 / 5.64 | 15.508 | PASS / PASS | PASS |
| [payload-random-read-10-compact-v2](raw/performance/payload_create_read/payload-random-read-10-compact-v2/perf.jsonl) | 10 / 1 / 10485760 | pure_call_sum_ns | 17.420 | 6.405 / — / 1.575 | 10.02 / 4.56 | 20.241 | PASS / PASS | PASS |
| [payload-random-read-100](raw/performance/payload_create_read/payload-random-read-100/perf.jsonl) | 100 / 1 / 524288000 | pure_call_sum_ns | 56.396 | 43.874 / — / 1.611 | 28.56 / 4.89 | 65.573 | PASS / PASS | PASS |
| [payload-random-read-500](raw/performance/payload_create_read/payload-random-read-500/perf.jsonl) | 500 / 1 / 524288000 | pure_call_sum_ns | 219.687 | 206.590 / — / 2.072 | 48.86 / 11.94 | 278.839 | PASS / PASS | PASS |

## store_footprint

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [store-footprint-unique-100000](raw/performance/store_footprint/store-footprint-unique-100000/perf.jsonl) | 100000 / 100000 / 500000000 | product_call_sum_ns | 2982.459 | — / — / — | 103.61 / 5.09 | 2936.726 | PASS / PASS | PASS |
| [store-footprint-metadata-cardinality-100000](raw/performance/store_footprint/store-footprint-metadata-cardinality-100000/perf.jsonl) | 100000 / 100000 / 500000000 | product_call_sum_ns | 4570.430 | — / — / — | 121.38 / 4.66 | 5708.674 | PASS / PASS | PASS |
| [store-footprint-large-object-500m](raw/performance/store_footprint/store-footprint-large-object-500m/perf.jsonl) | 100000 / 100 / 500000000 | product_call_sum_ns | 491.906 | — / — / — | 62.47 / 4.89 | 495.413 | PASS / PASS | PASS |
| [store-footprint-unique-100-low-v1](raw/performance/store_footprint/store-footprint-unique-100-low-v1/perf.jsonl) | 100 / 100 / 5000000 | product_call_sum_ns | 31.948 | — / — / — | 23.78 / 4.67 | 34.060 | PASS / PASS | PASS |
| [store-footprint-metadata-cardinality-100-low-v1](raw/performance/store_footprint/store-footprint-metadata-cardinality-100-low-v1/perf.jsonl) | 100 / 100 / 5000000 | product_call_sum_ns | 35.870 | — / — / — | 23.98 / 5.16 | 33.524 | PASS / PASS | PASS |
| [store-footprint-large-object-10m-low-v1](raw/performance/store_footprint/store-footprint-large-object-10m-low-v1/perf.jsonl) | 10 / 10 / 10000000 | product_call_sum_ns | 43.894 | — / — / — | 30.84 / 4.65 | 53.083 | PASS / PASS | PASS |

## tiny_file_churn

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [tiny-create-1-compact-v2](raw/performance/tiny_file_churn/tiny-create-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 19.054 | 5.761 / — / 3.307 | 9.50 / 4.93 | 22.029 | PASS / PASS | PASS |
| [tiny-create-10-compact-v2](raw/performance/tiny_file_churn/tiny-create-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 26.712 | 10.518 / — / 4.762 | 10.94 / 4.67 | 35.749 | PASS / PASS | PASS |
| [tiny-create-100-mixed-v4](raw/performance/tiny_file_churn/tiny-create-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 53.616 | 36.491 / — / 7.753 | 12.06 / 4.40 | 91.401 | PASS / PASS | PASS |
| [tiny-create-500-mixed-v4](raw/performance/tiny_file_churn/tiny-create-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 201.740 | 164.055 / — / 24.284 | 20.50 / 6.49 | 322.892 | PASS / PASS | PASS |
| [tiny-stat-1-compact-v2](raw/performance/tiny_file_churn/tiny-stat-1-compact-v2/perf.jsonl) | 1 / 51 / 1048576 | pure_call_sum_ns | 15.834 | 4.650 / — / 1.357 | 9.36 / 4.39 | 18.053 | PASS / PASS | PASS |
| [tiny-stat-10-compact-v2](raw/performance/tiny_file_churn/tiny-stat-10-compact-v2/perf.jsonl) | 10 / 510 / 10502249 | pure_call_sum_ns | 21.194 | 9.018 / — / 1.526 | 12.03 / 5.14 | 24.621 | PASS / PASS | PASS |
| [tiny-stat-100-mixed-v4](raw/performance/tiny_file_churn/tiny-stat-100-mixed-v4/perf.jsonl) | 100 / 2500 / 105682050 | pure_call_sum_ns | 37.810 | 26.471 / — / 1.778 | 13.34 / 5.58 | 65.484 | PASS / PASS | PASS |
| [tiny-stat-500-mixed-v4](raw/performance/tiny_file_churn/tiny-stat-500-mixed-v4/perf.jsonl) | 500 / 5500 / 525112450 | pure_call_sum_ns | 55.349 | 42.781 / — / 1.995 | 16.08 / 6.34 | 218.263 | PASS / PASS | PASS |
| [tiny-unlink-1-compact-v2](raw/performance/tiny_file_churn/tiny-unlink-1-compact-v2/perf.jsonl) | 1 / 51 / 1048576 | pure_call_sum_ns | 17.635 | 5.617 / — / 2.755 | 9.70 / 4.64 | 20.226 | PASS / PASS | PASS |
| [tiny-unlink-10-compact-v2](raw/performance/tiny_file_churn/tiny-unlink-10-compact-v2/perf.jsonl) | 10 / 510 / 10502249 | pure_call_sum_ns | 25.205 | 11.478 / — / 3.615 | 12.75 / 4.65 | 29.633 | PASS / PASS | PASS |
| [tiny-unlink-100-mixed-v4](raw/performance/tiny_file_churn/tiny-unlink-100-mixed-v4/perf.jsonl) | 100 / 2500 / 105682050 | pure_call_sum_ns | 50.918 | 35.328 / — / 5.093 | 15.22 / 5.44 | 77.793 | PASS / PASS | PASS |
| [tiny-unlink-500-mixed-v4](raw/performance/tiny_file_churn/tiny-unlink-500-mixed-v4/perf.jsonl) | 500 / 5500 / 525112450 | pure_call_sum_ns | 114.279 | 93.342 / — / 9.092 | 19.69 / 5.55 | 300.545 | PASS / PASS | PASS |
| [tiny-bulk-create-1-compact-v2](raw/performance/tiny_file_churn/tiny-bulk-create-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 96.728 | 73.820 / — / 11.621 | 14.31 / 6.88 | 99.502 | PASS / PASS | PASS |
| [tiny-bulk-create-10-compact-v2](raw/performance/tiny_file_churn/tiny-bulk-create-10-compact-v2/perf.jsonl) | 10 / 50 / 1048576 | pure_call_sum_ns | 295.898 | 241.067 / — / 43.042 | 35.42 / 11.36 | 317.471 | PASS / PASS | PASS |
| [tiny-bulk-create-100-mixed-v3](raw/performance/tiny_file_churn/tiny-bulk-create-100-mixed-v3/perf.jsonl) | 100 / 200 / 1048576 | pure_call_sum_ns | 989.888 | 595.398 / — / 381.964 | 68.56 / 14.11 | 1037.675 | PASS / PASS | PASS |
| [tiny-bulk-create-500-mixed-v3](raw/performance/tiny_file_churn/tiny-bulk-create-500-mixed-v3/perf.jsonl) | 500 / 200 / 1048576 | pure_call_sum_ns | 5054.054 | 3036.146 / — / 1987.005 | 86.27 / 23.53 | 5172.216 | PASS / PASS | PASS |
| [tiny-bulk-delete-1-compact-v2](raw/performance/tiny_file_churn/tiny-bulk-delete-1-compact-v2/perf.jsonl) | 1 / 100 / 2097152 | pure_call_sum_ns | 93.347 | 79.582 / — / 3.860 | 10.52 / 4.91 | 147.886 | PASS / PASS | PASS |
| [tiny-bulk-delete-10-compact-v2](raw/performance/tiny_file_churn/tiny-bulk-delete-10-compact-v2/perf.jsonl) | 10 / 550 / 11534336 | pure_call_sum_ns | 168.255 | 152.264 / — / 5.079 | 23.48 / 4.67 | 240.293 | PASS / PASS | PASS |
| [tiny-bulk-delete-100-mixed-v3](raw/performance/tiny_file_churn/tiny-bulk-delete-100-mixed-v3/perf.jsonl) | 100 / 1200 / 105906176 | pure_call_sum_ns | 259.279 | 243.270 / — / 5.440 | 39.69 / 8.47 | 334.564 | PASS / PASS | PASS |
| [tiny-bulk-delete-500-mixed-v3](raw/performance/tiny_file_churn/tiny-bulk-delete-500-mixed-v3/perf.jsonl) | 500 / 5200 / 525336576 | pure_call_sum_ns | 933.000 | 905.347 / — / 15.148 | 57.70 / 25.04 | 1079.909 | PASS / PASS | PASS |

## workspace_change_locality

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [workspace-clean-commit-1-compact-v2](raw/performance/workspace_change_locality/workspace-clean-commit-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 11.316 | — / — / 1.543 | 8.25 / 4.65 | 11.059 | PASS / PASS | PASS |
| [workspace-clean-commit-10-compact-v2](raw/performance/workspace_change_locality/workspace-clean-commit-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 10.873 | — / — / 1.445 | 8.78 / 4.18 | 12.542 | PASS / PASS | PASS |
| [workspace-clean-commit-100-mixed-v4](raw/performance/workspace_change_locality/workspace-clean-commit-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 11.559 | — / — / 1.469 | 8.33 / 4.64 | 11.508 | PASS / PASS | PASS |
| [workspace-clean-commit-500-mixed-v4](raw/performance/workspace_change_locality/workspace-clean-commit-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 11.866 | — / — / 1.404 | 8.86 / 5.15 | 12.953 | PASS / PASS | PASS |
| [workspace-fixed-move-1-compact-v2](raw/performance/workspace_change_locality/workspace-fixed-move-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 19.708 | 5.916 / — / 2.942 | 10.39 / 4.65 | 21.614 | PASS / PASS | PASS |
| [workspace-fixed-move-10-compact-v2](raw/performance/workspace_change_locality/workspace-fixed-move-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 21.320 | 6.788 / — / 3.281 | 13.42 / 4.65 | 21.996 | PASS / PASS | PASS |
| [workspace-fixed-move-100-mixed-v4](raw/performance/workspace_change_locality/workspace-fixed-move-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 28.352 | 12.034 / — / 3.936 | 12.64 / 4.50 | 20.821 | PASS / PASS | PASS |
| [workspace-fixed-move-500-mixed-v4](raw/performance/workspace_change_locality/workspace-fixed-move-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 27.589 | 12.655 / — / 3.554 | 16.16 / 4.93 | 23.177 | PASS / PASS | PASS |
| [workspace-distributed-sdk-edit-1-compact-v2](raw/performance/workspace_change_locality/workspace-distributed-sdk-edit-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 16.937 | — / 3.008 / 3.588 | 10.48 / 4.91 | 18.322 | PASS / PASS | PASS |
| [workspace-distributed-sdk-edit-10-compact-v2](raw/performance/workspace_change_locality/workspace-distributed-sdk-edit-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 39.180 | — / 23.559 / 4.909 | 19.98 / 4.64 | 35.855 | PASS / PASS | PASS |
| [workspace-distributed-sdk-edit-100-mixed-v4](raw/performance/workspace_change_locality/workspace-distributed-sdk-edit-100-mixed-v4/perf.jsonl.gz) | 100 / 2000 / 104857600 | pure_call_sum_ns | 336.399 | — / 310.515 / 14.688 | 54.62 / 12.93 | 355.760 | PASS / PASS | PASS |
| [workspace-distributed-sdk-edit-500-mixed-v4](raw/performance/workspace_change_locality/workspace-distributed-sdk-edit-500-mixed-v4/perf.jsonl.gz) | 500 / 5000 / 524288000 | pure_call_sum_ns | 2849.182 | — / 2782.241 / 56.074 | 66.81 / 29.64 | 2818.321 | PASS / PASS | PASS |
| [workspace-dense-rewrite-1-compact-v2](raw/performance/workspace_change_locality/workspace-dense-rewrite-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 84.118 | 63.190 / — / 10.069 | 14.91 / 8.21 | 124.779 | PASS / PASS | PASS |
| [workspace-dense-rewrite-10-compact-v2](raw/performance/workspace_change_locality/workspace-dense-rewrite-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 316.616 | 226.868 / — / 76.749 | 47.61 / 21.51 | 445.117 | PASS / PASS | PASS |
| [workspace-dense-rewrite-100-mixed-v4](raw/performance/workspace_change_locality/workspace-dense-rewrite-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 1497.618 | 933.244 / — / 545.125 | 75.59 / 124.39 | 2434.446 | PASS / PASS | PASS |
| [workspace-dense-rewrite-500-mixed-v4](raw/performance/workspace_change_locality/workspace-dense-rewrite-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 5688.592 | 3032.658 / — / 2626.086 | 93.47 / 546.07 | 8334.002 | PASS / PASS | PASS |

## workspace_reliability

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|

SDK verification uses the versioned cold-projection proof on an explicitly compatible verifier-only source. All 56 SDK proofs were requalified; their original receipts, including ten resource failures from the pre-edit-lookup proof, remain in raw evidence. Product code, performance paths, fixtures, image and harness are unchanged. See sdk_requalification in the JSON report for both source and recipe seals.

Historical subsecond bulk targets are separate from the family threshold:
- tiny-bulk-create-100-mixed-v3: PASS against strict <1,000 ms.
- tiny-bulk-delete-100-mixed-v3: PASS against strict <1,000 ms.

## Git stages

| Test | Apply | First status | Diff | Add | Cached check | Git commit | Final status | LayerFS Commit |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| git-tool-1-compact-v2 | 3.877 | 118.801 | 85.742 | 36.216 | 4.703 | 13.079 | 30.733 | 5.010 |
| git-tool-10-compact-v2 | 10.896 | 201.073 | 268.597 | 44.717 | 5.151 | 16.623 | 30.209 | 8.797 |
| git-tool-100-mixed-v4 | 97.090 | 502.457 | 1019.667 | 101.550 | 14.611 | 40.066 | 35.236 | 27.289 |
| git-tool-500-mixed-v4 | 474.960 | 1135.062 | 2279.828 | 358.900 | 71.041 | 122.089 | 65.302 | 78.107 |

Historical native Git reference (three-run median):
- git-tool-100-mixed-v4: 249.184 ms, Apply + six Git commands. [Published source](../issue68-evidence/report.json).
- git-tool-500-mixed-v4: 634.505 ms, Apply + six Git commands. [Published source](../issue68-evidence/report.json).

Native references exclude LayerFS Create/Commit/visibility/End and are not matched total-lifecycle comparisons.

All Git stage values are milliseconds. Git commit and LayerFS Commit are separate operations.

## Verification

| Test | Result | Wall (s) | Coverage |
|---|---|---:|---|
| [payload-create-1m-compact-v2](raw/verification/payload_create_read/payload-create-1m-compact-v2/verification.json) | PASS | 1.6539817498996854 | changed data and selected unchanged witnesses; qualified roots |
| [payload-create-10m-compact-v2](raw/verification/payload_create_read/payload-create-10m-compact-v2/verification.json) | PASS | 1.8620274169370532 | changed data and selected unchanged witnesses; qualified roots |
| [payload-create-100m](raw/verification/payload_create_read/payload-create-100m/verification.json) | PASS | 3.2236744998954237 | changed data and selected unchanged witnesses; qualified roots |
| [payload-create-500m](raw/verification/payload_create_read/payload-create-500m/verification.json) | PASS | 9.37614162499085 | changed data and selected unchanged witnesses; qualified roots |
| [payload-random-read-1-compact-v2](raw/verification/payload_create_read/payload-random-read-1-compact-v2/verification.json) | PASS | 1.8159785000607371 | changed data and selected unchanged witnesses; qualified roots |
| [payload-random-read-10-compact-v2](raw/verification/payload_create_read/payload-random-read-10-compact-v2/verification.json) | PASS | 2.0634160828776658 | changed data and selected unchanged witnesses; qualified roots |
| [payload-random-read-100](raw/verification/payload_create_read/payload-random-read-100/verification.json) | PASS | 14.759463042020798 | changed data and selected unchanged witnesses; qualified roots |
| [payload-random-read-500](raw/verification/payload_create_read/payload-random-read-500/verification.json) | PASS | 8.67045862507075 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-exact-1-compact-v2](raw/verification/dedup_workspace_reuse/dedup-workspace-exact-1-compact-v2/verification.json) | PASS | 2.724054167047143 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-exact-10-compact-v2](raw/verification/dedup_workspace_reuse/dedup-workspace-exact-10-compact-v2/verification.json) | PASS | 2.1237384169362485 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-exact-100](raw/verification/dedup_workspace_reuse/dedup-workspace-exact-100/verification.json) | PASS | 7.579232375137508 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-exact-500](raw/verification/dedup_workspace_reuse/dedup-workspace-exact-500/verification.json) | PASS | 14.479497166816145 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-local-1-compact-v2](raw/verification/dedup_workspace_reuse/dedup-workspace-local-1-compact-v2/verification.json) | PASS | 1.6782422503456473 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-local-10-compact-v2](raw/verification/dedup_workspace_reuse/dedup-workspace-local-10-compact-v2/verification.json) | PASS | 2.011282874736935 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-local-100](raw/verification/dedup_workspace_reuse/dedup-workspace-local-100/verification.json) | PASS | 5.887250042054802 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-local-500](raw/verification/dedup_workspace_reuse/dedup-workspace-local-500/verification.json) | PASS | 14.32428654236719 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-1-compact-v2](raw/verification/dedup_workspace_reuse/dedup-workspace-unique-1-compact-v2/verification.json) | PASS | 1.613851415924728 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-10-compact-v2](raw/verification/dedup_workspace_reuse/dedup-workspace-unique-10-compact-v2/verification.json) | PASS | 1.9447309169918299 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-100](raw/verification/dedup_workspace_reuse/dedup-workspace-unique-100/verification.json) | PASS | 5.869120125193149 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-500](raw/verification/dedup_workspace_reuse/dedup-workspace-unique-500/verification.json) | PASS | 14.501097959000617 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-1-base128-v3](raw/verification/dedup_workspace_reuse/dedup-workspace-unique-1-base128-v3/verification.json) | PASS | 3.9754031249321997 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-10-base128-v3](raw/verification/dedup_workspace_reuse/dedup-workspace-unique-10-base128-v3/verification.json) | PASS | 3.9360759588889778 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-anchor-1](raw/verification/dedup_cross_file/dedup-cross-file-anchor-1/verification.json) | PASS | 1.5741568747907877 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-unique-10](raw/verification/dedup_cross_file/dedup-cross-file-unique-10/verification.json) | PASS | 1.842995208222419 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-unique-100](raw/verification/dedup_cross_file/dedup-cross-file-unique-100/verification.json) | PASS | 3.9533258751034737 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-unique-500](raw/verification/dedup_cross_file/dedup-cross-file-unique-500/verification.json) | PASS | 13.830102666746825 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-identical-10](raw/verification/dedup_cross_file/dedup-cross-file-identical-10/verification.json) | PASS | 1.8481676252558827 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-identical-100](raw/verification/dedup_cross_file/dedup-cross-file-identical-100/verification.json) | PASS | 3.2244128747843206 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-identical-500](raw/verification/dedup_cross_file/dedup-cross-file-identical-500/verification.json) | PASS | 9.846022041980177 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-mixed-10](raw/verification/dedup_cross_file/dedup-cross-file-mixed-10/verification.json) | PASS | 1.80004091700539 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-mixed-100](raw/verification/dedup_cross_file/dedup-cross-file-mixed-100/verification.json) | PASS | 3.9346542921848595 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-mixed-500](raw/verification/dedup_cross_file/dedup-cross-file-mixed-500/verification.json) | PASS | 13.190160249825567 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-overwrite-1](raw/verification/dedup_cdc_locality/dedup-cdc-overwrite-1/verification.json) | PASS | 1.9819096252322197 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-overwrite-10](raw/verification/dedup_cdc_locality/dedup-cdc-overwrite-10/verification.json) | PASS | 2.1752002499997616 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-overwrite-100](raw/verification/dedup_cdc_locality/dedup-cdc-overwrite-100/verification.json) | PASS | 4.911127625033259 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-overwrite-500](raw/verification/dedup_cdc_locality/dedup-cdc-overwrite-500/verification.json.gz) | PASS | 17.282971000298858 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-insert-1](raw/verification/dedup_cdc_locality/dedup-cdc-insert-1/verification.json) | PASS | 1.7598338327370584 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-insert-10](raw/verification/dedup_cdc_locality/dedup-cdc-insert-10/verification.json) | PASS | 1.9468696247786283 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-insert-100](raw/verification/dedup_cdc_locality/dedup-cdc-insert-100/verification.json.gz) | PASS | 4.800869207829237 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-insert-500](raw/verification/dedup_cdc_locality/dedup-cdc-insert-500/verification.json.gz) | PASS | 17.712656499817967 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-delete-1](raw/verification/dedup_cdc_locality/dedup-cdc-delete-1/verification.json) | PASS | 1.688861166127026 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-delete-10](raw/verification/dedup_cdc_locality/dedup-cdc-delete-10/verification.json) | PASS | 2.125135541893542 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-delete-100](raw/verification/dedup_cdc_locality/dedup-cdc-delete-100/verification.json.gz) | PASS | 5.0871890829876065 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-delete-500](raw/verification/dedup_cdc_locality/dedup-cdc-delete-500/verification.json.gz) | PASS | 18.953364375047386 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-common-body-1](raw/verification/dedup_cdc_locality/dedup-cdc-common-body-1/verification.json) | PASS | 1.6940456670708954 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-common-body-10](raw/verification/dedup_cdc_locality/dedup-cdc-common-body-10/verification.json) | PASS | 1.937447499949485 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-common-body-100](raw/verification/dedup_cdc_locality/dedup-cdc-common-body-100/verification.json.gz) | PASS | 4.916059957817197 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-common-body-500](raw/verification/dedup_cdc_locality/dedup-cdc-common-body-500/verification.json.gz) | PASS | 17.835566584020853 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-scattered-1](raw/verification/dedup_cdc_locality/dedup-cdc-scattered-1/verification.json) | PASS | 1.7866018330678344 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-scattered-10](raw/verification/dedup_cdc_locality/dedup-cdc-scattered-10/verification.json) | PASS | 2.0293574589304626 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-scattered-100](raw/verification/dedup_cdc_locality/dedup-cdc-scattered-100/verification.json) | PASS | 5.280157791916281 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-scattered-500](raw/verification/dedup_cdc_locality/dedup-cdc-scattered-500/verification.json.gz) | PASS | 19.78213316621259 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-boundaries-proof](raw/verification/dedup_cdc_locality/dedup-cdc-boundaries-proof/verification.json) | PASS | 1.5558000826276839 | registered verification; detailed checks in receipt |
| [overwrite-head-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-head-4k-on-1mib-ops-1/verification.json) | PASS | 4.893288208171725 | bounded bytes; payload.bin [0, 69632) |
| [overwrite-head-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-head-4k-on-10mib-ops-1/verification.json) | PASS | 2.007636707741767 | bounded bytes; payload.bin [0, 69632) |
| [overwrite-head-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-head-4k-on-100mib-ops-1/verification.json) | PASS | 2.074364125262946 | bounded bytes; payload.bin [0, 69632) |
| [overwrite-head-4k-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-head-4k-on-500mib-ops-1/verification.json) | PASS | 10.349498459137976 | bounded bytes; payload.bin [0, 69632) |
| [overwrite-middle-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-middle-4k-on-1mib-ops-1/verification.json) | PASS | 1.4846038329415023 | bounded bytes; payload.bin [456704, 591872) |
| [overwrite-middle-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-middle-4k-on-10mib-ops-1/verification.json) | PASS | 1.4658998753875494 | bounded bytes; payload.bin [5175296, 5310464) |
| [overwrite-middle-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-middle-4k-on-100mib-ops-1/verification.json) | PASS | 1.9178994167596102 | bounded bytes; payload.bin [52361216, 52496384) |
| [overwrite-middle-4k-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-middle-4k-on-500mib-ops-1/verification.json) | PASS | 3.9773077918216586 | bounded bytes; payload.bin [262076416, 262211584) |
| [overwrite-tail-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-tail-4k-on-1mib-ops-1/verification.json) | PASS | 1.6904152501374483 | bounded bytes; payload.bin [978944, 1048576) |
| [overwrite-tail-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-tail-4k-on-10mib-ops-1/verification.json) | PASS | 1.5769604998640716 | bounded bytes; payload.bin [10416128, 10485760) |
| [overwrite-tail-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-tail-4k-on-100mib-ops-1/verification.json) | PASS | 2.0943664158694446 | bounded bytes; payload.bin [104787968, 104857600) |
| [overwrite-tail-4k-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_preserving/overwrite-tail-4k-on-500mib-ops-1/verification.json) | PASS | 4.1155280829407275 | bounded bytes; payload.bin [524218368, 524288000) |
| [insert-middle-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/insert-middle-4k-on-1mib-ops-1/verification.json) | PASS | 1.645815750118345 | bounded bytes; payload.bin [458752, 593920) |
| [insert-middle-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/insert-middle-4k-on-10mib-ops-1/verification.json) | PASS | 1.629339958075434 | bounded bytes; payload.bin [5177344, 5312512) |
| [insert-middle-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/insert-middle-4k-on-100mib-ops-1/verification.json) | PASS | 2.126746834255755 | bounded bytes; payload.bin [52363264, 52498432) |
| [insert-middle-4k-on-500mib-result-capped-v2-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/insert-middle-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 10.1168696670793 | bounded bytes; payload.bin [262076416, 262211584) |
| [delete-middle-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/delete-middle-4k-on-1mib-ops-1/verification.json) | PASS | 1.6300312080420554 | bounded bytes; payload.bin [456704, 587776) |
| [delete-middle-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/delete-middle-4k-on-10mib-ops-1/verification.json) | PASS | 1.6265036249533296 | bounded bytes; payload.bin [5175296, 5306368) |
| [delete-middle-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/delete-middle-4k-on-100mib-ops-1/verification.json) | PASS | 2.165766208432615 | bounded bytes; payload.bin [52361216, 52492288) |
| [delete-middle-4k-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/delete-middle-4k-on-500mib-ops-1/verification.json) | PASS | 3.7909686248749495 | bounded bytes; payload.bin [262076416, 262207488) |
| [append-tail-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/append-tail-4k-on-1mib-ops-1/verification.json) | PASS | 1.598674583248794 | bounded bytes; payload.bin [983040, 1052672) |
| [append-tail-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/append-tail-4k-on-10mib-ops-1/verification.json) | PASS | 1.5596828749403358 | bounded bytes; payload.bin [10420224, 10489856) |
| [append-tail-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/append-tail-4k-on-100mib-ops-1/verification.json) | PASS | 2.0242148749530315 | bounded bytes; payload.bin [104792064, 104861696) |
| [append-tail-4k-on-500mib-result-capped-v2-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/append-tail-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 3.907536083832383 | bounded bytes; payload.bin [524218368, 524288000) |
| [prepend-head-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/prepend-head-4k-on-1mib-ops-1/verification.json) | PASS | 1.6000570417381823 | bounded bytes; payload.bin [0, 69632) |
| [prepend-head-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/prepend-head-4k-on-10mib-ops-1/verification.json) | PASS | 1.475805874913931 | bounded bytes; payload.bin [0, 69632) |
| [prepend-head-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/prepend-head-4k-on-100mib-ops-1/verification.json) | PASS | 2.0379828750155866 | bounded bytes; payload.bin [0, 69632) |
| [prepend-head-4k-on-500mib-result-capped-v2-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/prepend-head-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 3.723122708965093 | bounded bytes; payload.bin [0, 69632) |
| [replace-grow-middle-2k-to-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/replace-grow-middle-2k-to-4k-on-1mib-ops-1/verification.json) | PASS | 1.4701372082345188 | bounded bytes; payload.bin [457728, 592896) |
| [replace-grow-middle-2k-to-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/replace-grow-middle-2k-to-4k-on-10mib-ops-1/verification.json) | PASS | 1.5025674169883132 | bounded bytes; payload.bin [5176320, 5311488) |
| [replace-grow-middle-2k-to-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/replace-grow-middle-2k-to-4k-on-100mib-ops-1/verification.json) | PASS | 2.0294438749551773 | bounded bytes; payload.bin [52362240, 52497408) |
| [replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 9.569232542067766 | bounded bytes; payload.bin [262076416, 262211584) |
| [replace-shrink-middle-4k-to-2k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/replace-shrink-middle-4k-to-2k-on-1mib-ops-1/verification.json) | PASS | 1.4633545419201255 | bounded bytes; payload.bin [456704, 589824) |
| [replace-shrink-middle-4k-to-2k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/replace-shrink-middle-4k-to-2k-on-10mib-ops-1/verification.json) | PASS | 1.7834462500177324 | bounded bytes; payload.bin [5175296, 5308416) |
| [replace-shrink-middle-4k-to-2k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/replace-shrink-middle-4k-to-2k-on-100mib-ops-1/verification.json) | PASS | 2.0271498328074813 | bounded bytes; payload.bin [52361216, 52494336) |
| [replace-shrink-middle-4k-to-2k-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/replace-shrink-middle-4k-to-2k-on-500mib-ops-1/verification.json) | PASS | 4.094620666000992 | bounded bytes; payload.bin [262076416, 262209536) |
| [truncate-tail-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/truncate-tail-4k-on-1mib-ops-1/verification.json) | PASS | 1.5802297913469374 | bounded bytes; payload.bin [978944, 1044480) |
| [truncate-tail-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/truncate-tail-4k-on-10mib-ops-1/verification.json) | PASS | 1.5486287497915328 | bounded bytes; payload.bin [10416128, 10481664) |
| [truncate-tail-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/truncate-tail-4k-on-100mib-ops-1/verification.json) | PASS | 2.1211761250160635 | bounded bytes; payload.bin [104787968, 104853504) |
| [truncate-tail-4k-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/truncate-tail-4k-on-500mib-ops-1/verification.json) | PASS | 4.183337207883596 | bounded bytes; payload.bin [524218368, 524283904) |
| [zero-extend-tail-4k-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/zero-extend-tail-4k-on-1mib-ops-1/verification.json) | PASS | 1.429268625099212 | bounded bytes; payload.bin [983040, 1052672) |
| [zero-extend-tail-4k-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/zero-extend-tail-4k-on-10mib-ops-1/verification.json) | PASS | 1.6796865002252162 | bounded bytes; payload.bin [10420224, 10489856) |
| [zero-extend-tail-4k-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/zero-extend-tail-4k-on-100mib-ops-1/verification.json) | PASS | 2.3117653341032565 | bounded bytes; payload.bin [104792064, 104861696) |
| [zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1](raw/requalification-sdk-v2/verification/edit_length_changing/zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 4.307895042002201 | bounded bytes; payload.bin [524218368, 524288000) |
| [overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1/verification.json) | PASS | 1.693148166872561 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1/verification.json) | PASS | 1.6394168329425156 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1/verification.json) | PASS | 2.086416333913803 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1/verification.json) | PASS | 4.169705667067319 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1/verification.json) | PASS | 1.4702880419790745 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1/verification.json) | PASS | 1.5128514170646667 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1/verification.json) | PASS | 2.0101697077043355 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1/verification.json) | PASS | 3.829485333058983 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1/verification.json) | PASS | 1.5791225829161704 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1/verification.json) | PASS | 1.7167114582844079 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1/verification.json) | PASS | 2.053627416025847 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1](raw/requalification-sdk-v2/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1/verification.json) | PASS | 4.162201290950179 | bounded bytes; payload.bin [81920, 278528) |
| [namespace-100-compact-v3](raw/verification/init_namespace/namespace-100-compact-v3/verification.json) | PASS | 1.6314908331260085 | registered verification; detailed checks in receipt |
| [namespace-1000-compact-v3](raw/verification/init_namespace/namespace-1000-compact-v3/verification.json) | PASS | 1.965520250145346 | registered verification; detailed checks in receipt |
| [namespace-10000](raw/verification/init_namespace/namespace-10000/verification.json) | PASS | 6.707328666001558 | registered verification; detailed checks in receipt |
| [namespace-100000](raw/verification/init_namespace/namespace-100000/verification.json) | PASS | 35.46769508300349 | registered verification; detailed checks in receipt |
| [store-footprint-unique-100000](raw/verification/store_footprint/store-footprint-unique-100000/verification.json) | PASS | 32.86010116711259 | bounded bytes; d0000/f000000 [0, 1468) |
| [store-footprint-metadata-cardinality-100000](raw/verification/store_footprint/store-footprint-metadata-cardinality-100000/verification.json) | PASS | 38.551538666244596 | bounded bytes; d0000/f000000 [0, 1468) |
| [store-footprint-large-object-500m](raw/verification/store_footprint/store-footprint-large-object-500m/verification.json) | PASS | 5.6987985000014305 | bounded bytes; d0000/file-00000.bin [4375525, 4506607) |
| [store-footprint-unique-100-low-v1](raw/verification/store_footprint/store-footprint-unique-100-low-v1/verification.json) | PASS | 2.0881247082725167 | bounded bytes; d0000/f000000 [0, 1097) |
| [store-footprint-metadata-cardinality-100-low-v1](raw/verification/store_footprint/store-footprint-metadata-cardinality-100-low-v1/verification.json) | PASS | 2.1498197079636157 | bounded bytes; d0000/f000000 [0, 1097) |
| [store-footprint-large-object-10m-low-v1](raw/verification/store_footprint/store-footprint-large-object-10m-low-v1/verification.json) | PASS | 2.1416437081061304 | bounded bytes; d0000/file-00000.bin [396765, 527847) |
| [tiny-create-1-compact-v2](raw/verification/tiny_file_churn/tiny-create-1-compact-v2/verification.json) | PASS | 2.9295104583725333 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-create-10-compact-v2](raw/verification/tiny_file_churn/tiny-create-10-compact-v2/verification.json) | PASS | 10.674214290920645 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-create-100-mixed-v4](raw/verification/tiny_file_churn/tiny-create-100-mixed-v4/verification.json) | PASS | 13.17870608298108 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-create-500-mixed-v4](raw/verification/tiny_file_churn/tiny-create-500-mixed-v4/verification.json) | PASS | 23.838351749815047 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-stat-1-compact-v2](raw/verification/tiny_file_churn/tiny-stat-1-compact-v2/verification.json) | PASS | 1.6641851249150932 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-stat-10-compact-v2](raw/verification/tiny_file_churn/tiny-stat-10-compact-v2/verification.json) | PASS | 1.8630032921209931 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-stat-100-mixed-v4](raw/verification/tiny_file_churn/tiny-stat-100-mixed-v4/verification.json) | PASS | 4.687502583023161 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-stat-500-mixed-v4](raw/verification/tiny_file_churn/tiny-stat-500-mixed-v4/verification.json) | PASS | 15.140078791882843 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-unlink-1-compact-v2](raw/verification/tiny_file_churn/tiny-unlink-1-compact-v2/verification.json) | PASS | 1.5801415829919279 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-unlink-10-compact-v2](raw/verification/tiny_file_churn/tiny-unlink-10-compact-v2/verification.json) | PASS | 1.5121114579960704 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-unlink-100-mixed-v4](raw/verification/tiny_file_churn/tiny-unlink-100-mixed-v4/verification.json) | PASS | 2.215617834124714 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-unlink-500-mixed-v4](raw/verification/tiny_file_churn/tiny-unlink-500-mixed-v4/verification.json) | PASS | 4.306972874794155 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-create-1-compact-v2](raw/verification/tiny_file_churn/tiny-bulk-create-1-compact-v2/verification.json) | PASS | 1.857313958927989 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-create-10-compact-v2](raw/verification/tiny_file_churn/tiny-bulk-create-10-compact-v2/verification.json) | PASS | 1.859029124956578 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-create-100-mixed-v3](raw/verification/tiny_file_churn/tiny-bulk-create-100-mixed-v3/verification.json) | PASS | 3.2811177908442914 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-create-500-mixed-v3](raw/verification/tiny_file_churn/tiny-bulk-create-500-mixed-v3/verification.json) | PASS | 9.008737375028431 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-delete-1-compact-v2](raw/verification/tiny_file_churn/tiny-bulk-delete-1-compact-v2/verification.json) | PASS | 1.7963929162360728 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-delete-10-compact-v2](raw/verification/tiny_file_churn/tiny-bulk-delete-10-compact-v2/verification.json) | PASS | 2.326850458048284 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-delete-100-mixed-v3](raw/verification/tiny_file_churn/tiny-bulk-delete-100-mixed-v3/verification.json) | PASS | 4.3907437501475215 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-delete-500-mixed-v3](raw/verification/tiny_file_churn/tiny-bulk-delete-500-mixed-v3/verification.json) | PASS | 19.176117249764502 | changed data and selected unchanged witnesses; qualified roots |
| [namespace-subtree-relocate-delete-1-compact-v2](raw/verification/namespace_mutation/namespace-subtree-relocate-delete-1-compact-v2/verification.json) | PASS | 1.934355332981795 | changed data and selected unchanged witnesses; qualified roots |
| [namespace-subtree-relocate-delete-10-compact-v2](raw/verification/namespace_mutation/namespace-subtree-relocate-delete-10-compact-v2/verification.json) | PASS | 1.9511259589344263 | changed data and selected unchanged witnesses; qualified roots |
| [namespace-subtree-relocate-delete-100-mixed-v4](raw/verification/namespace_mutation/namespace-subtree-relocate-delete-100-mixed-v4/verification.json) | PASS | 4.428649625275284 | changed data and selected unchanged witnesses; qualified roots |
| [namespace-subtree-relocate-delete-500-mixed-v4](raw/verification/namespace_mutation/namespace-subtree-relocate-delete-500-mixed-v4/verification.json) | PASS | 15.900314709171653 | changed data and selected unchanged witnesses; qualified roots |
| [directory-construct-1-compact-v2](raw/verification/directory_construction_traversal/directory-construct-1-compact-v2/verification.json) | PASS | 1.8342067091725767 | changed data and selected unchanged witnesses; qualified roots |
| [directory-construct-10-compact-v2](raw/verification/directory_construction_traversal/directory-construct-10-compact-v2/verification.json) | PASS | 2.4709582081995904 | changed data and selected unchanged witnesses; qualified roots |
| [directory-construct-100-mixed-v4](raw/verification/directory_construction_traversal/directory-construct-100-mixed-v4/verification.json) | PASS | 4.403138792142272 | changed data and selected unchanged witnesses; qualified roots |
| [directory-construct-500-mixed-v4](raw/verification/directory_construction_traversal/directory-construct-500-mixed-v4/verification.json) | PASS | 16.01527929212898 | changed data and selected unchanged witnesses; qualified roots |
| [directory-metadata-scan-1-compact-v2](raw/verification/directory_construction_traversal/directory-metadata-scan-1-compact-v2/verification.json.gz) | PASS | 1.9332237499766052 | changed data and selected unchanged witnesses; qualified roots |
| [directory-metadata-scan-10-compact-v2](raw/verification/directory_construction_traversal/directory-metadata-scan-10-compact-v2/verification.json.gz) | PASS | 2.5388052500784397 | changed data and selected unchanged witnesses; qualified roots |
| [directory-metadata-scan-100-mixed-v4](raw/verification/directory_construction_traversal/directory-metadata-scan-100-mixed-v4/verification.json.gz) | PASS | 4.6578890001401305 | changed data and selected unchanged witnesses; qualified roots |
| [directory-metadata-scan-500-mixed-v4](raw/verification/directory_construction_traversal/directory-metadata-scan-500-mixed-v4/verification.json.gz) | PASS | 15.367040832992643 | changed data and selected unchanged witnesses; qualified roots |
| [directory-content-scan-1-compact-v2](raw/verification/directory_construction_traversal/directory-content-scan-1-compact-v2/verification.json.gz) | PASS | 1.8085595830343664 | changed data and selected unchanged witnesses; qualified roots |
| [directory-content-scan-10-compact-v2](raw/verification/directory_construction_traversal/directory-content-scan-10-compact-v2/verification.json.gz) | PASS | 2.3349620001390576 | changed data and selected unchanged witnesses; qualified roots |
| [directory-content-scan-100-mixed-v4](raw/verification/directory_construction_traversal/directory-content-scan-100-mixed-v4/verification.json.gz) | PASS | 3.204433625098318 | changed data and selected unchanged witnesses; qualified roots |
| [directory-content-scan-500-mixed-v4](raw/verification/directory_construction_traversal/directory-content-scan-500-mixed-v4/verification.json.gz) | PASS | 8.218886875081807 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-clean-commit-1-compact-v2](raw/verification/workspace_change_locality/workspace-clean-commit-1-compact-v2/verification.json) | PASS | 1.6036402923054993 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-clean-commit-10-compact-v2](raw/verification/workspace_change_locality/workspace-clean-commit-10-compact-v2/verification.json) | PASS | 2.133901999797672 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-clean-commit-100-mixed-v4](raw/verification/workspace_change_locality/workspace-clean-commit-100-mixed-v4/verification.json) | PASS | 1.983244125265628 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-clean-commit-500-mixed-v4](raw/verification/workspace_change_locality/workspace-clean-commit-500-mixed-v4/verification.json) | PASS | 4.171032125130296 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-fixed-move-1-compact-v2](raw/verification/workspace_change_locality/workspace-fixed-move-1-compact-v2/verification.json) | PASS | 1.7485374580137432 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-fixed-move-10-compact-v2](raw/verification/workspace_change_locality/workspace-fixed-move-10-compact-v2/verification.json) | PASS | 1.989767957944423 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-fixed-move-100-mixed-v4](raw/verification/workspace_change_locality/workspace-fixed-move-100-mixed-v4/verification.json) | PASS | 2.1229467089287937 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-fixed-move-500-mixed-v4](raw/verification/workspace_change_locality/workspace-fixed-move-500-mixed-v4/verification.json) | PASS | 4.110525917261839 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-distributed-sdk-edit-1-compact-v2](raw/verification/workspace_change_locality/workspace-distributed-sdk-edit-1-compact-v2/verification.json) | PASS | 1.6775912921875715 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-distributed-sdk-edit-10-compact-v2](raw/verification/workspace_change_locality/workspace-distributed-sdk-edit-10-compact-v2/verification.json) | PASS | 2.05084816692397 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-distributed-sdk-edit-100-mixed-v4](raw/verification/workspace_change_locality/workspace-distributed-sdk-edit-100-mixed-v4/verification.json.gz) | PASS | 2.5024316660128534 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-distributed-sdk-edit-500-mixed-v4](raw/verification/workspace_change_locality/workspace-distributed-sdk-edit-500-mixed-v4/verification.json.gz) | PASS | 7.130985875148326 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-dense-rewrite-1-compact-v2](raw/verification/workspace_change_locality/workspace-dense-rewrite-1-compact-v2/verification.json) | PASS | 1.7524517909623682 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-dense-rewrite-10-compact-v2](raw/verification/workspace_change_locality/workspace-dense-rewrite-10-compact-v2/verification.json) | PASS | 2.570778665598482 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-dense-rewrite-100-mixed-v4](raw/verification/workspace_change_locality/workspace-dense-rewrite-100-mixed-v4/verification.json) | PASS | 3.8838729159906507 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-dense-rewrite-500-mixed-v4](raw/verification/workspace_change_locality/workspace-dense-rewrite-500-mixed-v4/verification.json) | PASS | 10.18545279186219 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-history-distributed-1](raw/verification/dedup_branch_history/dedup-history-distributed-1/verification.json) | PASS | 2.1922978330403566 | all parent links; snapshots [0, 1] |
| [dedup-history-distributed-10](raw/verification/dedup_branch_history/dedup-history-distributed-10/verification.json.gz) | PASS | 4.5789663339965045 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-distributed-100](raw/verification/dedup_branch_history/dedup-history-distributed-100/verification.json.gz) | PASS | 4.233630375005305 | all parent links; snapshots [0, 1, 2, 49, 50, 99, 100] |
| [dedup-history-distributed-500](raw/verification/dedup_branch_history/dedup-history-distributed-500/verification.json.gz) | PASS | 8.538804166950285 | all parent links; snapshots [0, 1, 199, 200, 201, 250, 499, 500] |
| [dedup-history-hotset-1](raw/verification/dedup_branch_history/dedup-history-hotset-1/verification.json) | PASS | 2.239450457971543 | all parent links; snapshots [0, 1] |
| [dedup-history-hotset-10](raw/verification/dedup_branch_history/dedup-history-hotset-10/verification.json.gz) | PASS | 4.571824874728918 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-hotset-100](raw/verification/dedup_branch_history/dedup-history-hotset-100/verification.json.gz) | PASS | 4.604663415811956 | all parent links; snapshots [0, 1, 7, 8, 9, 50, 99, 100] |
| [dedup-history-hotset-500](raw/verification/dedup_branch_history/dedup-history-hotset-500/verification.json.gz) | PASS | 8.688416291959584 | all parent links; snapshots [0, 1, 7, 8, 9, 250, 499, 500] |
| [dedup-history-recurring-1](raw/verification/dedup_branch_history/dedup-history-recurring-1/verification.json) | PASS | 2.060230457689613 | all parent links; snapshots [0, 1] |
| [dedup-history-recurring-10](raw/verification/dedup_branch_history/dedup-history-recurring-10/verification.json.gz) | PASS | 4.071059083100408 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-recurring-100](raw/verification/dedup_branch_history/dedup-history-recurring-100/verification.json.gz) | PASS | 3.892683541867882 | all parent links; snapshots [0, 1, 2, 3, 99, 100] |
| [dedup-history-recurring-500](raw/verification/dedup_branch_history/dedup-history-recurring-500/verification.json.gz) | PASS | 7.011576457880437 | all parent links; snapshots [0, 1, 2, 3, 499, 500] |
| [dedup-history-metadata-1](raw/verification/dedup_branch_history/dedup-history-metadata-1/verification.json) | PASS | 1.9984951252117753 | all parent links; snapshots [0, 1] |
| [dedup-history-metadata-10](raw/verification/dedup_branch_history/dedup-history-metadata-10/verification.json.gz) | PASS | 4.163718957919627 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-metadata-100](raw/verification/dedup_branch_history/dedup-history-metadata-100/verification.json.gz) | PASS | 3.8464217078872025 | all parent links; snapshots [0, 1, 2, 49, 50, 99, 100] |
| [dedup-history-metadata-500](raw/verification/dedup_branch_history/dedup-history-metadata-500/verification.json.gz) | PASS | 6.678333332762122 | all parent links; snapshots [0, 1, 2, 249, 250, 499, 500] |
| [dedup-history-unrelated-1](raw/verification/dedup_branch_history/dedup-history-unrelated-1/verification.json) | PASS | 2.9486843328922987 | all parent links; snapshots [0, 1] |
| [dedup-history-unrelated-10](raw/verification/dedup_branch_history/dedup-history-unrelated-10/verification.json.gz) | PASS | 12.83805275009945 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-unrelated-100-mixed-v2](raw/verification/dedup_branch_history/dedup-history-unrelated-100-mixed-v2/verification.json.gz) | PASS | 5.244219375308603 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-history-unrelated-500-mixed-v2](raw/verification/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/verification.json.gz) | PASS | 20.745782082900405 | changed data and selected unchanged witnesses; qualified roots |
| [git-tool-1-compact-v2](raw/verification/git_tool_workflow/git-tool-1-compact-v2/verification.json) | PASS | 5.03639341564849 | full head/tree/parent and reopened custody |
| [git-tool-10-compact-v2](raw/verification/git_tool_workflow/git-tool-10-compact-v2/verification.json) | PASS | 6.69657937483862 | full head/tree/parent and reopened custody |
| [git-tool-100-mixed-v4](raw/verification/git_tool_workflow/git-tool-100-mixed-v4/verification.json) | PASS | 20.080943084321916 | full head/tree/parent and reopened custody |
| [git-tool-500-mixed-v4](raw/verification/git_tool_workflow/git-tool-500-mixed-v4/verification.json) | PASS | 36.49917087517679 | full head/tree/parent and reopened custody |
| [agent-episodes-1-compact-v2](raw/verification/mixed_load_bearing/agent-episodes-1-compact-v2/verification.json) | PASS | 2.0044421670027077 | changed data and selected unchanged witnesses; qualified roots |
| [agent-episodes-10-compact-v2](raw/verification/mixed_load_bearing/agent-episodes-10-compact-v2/verification.json) | PASS | 2.7333738333545625 | changed data and selected unchanged witnesses; qualified roots |
| [agent-episodes-100](raw/verification/mixed_load_bearing/agent-episodes-100/verification.json) | PASS | 18.099349500145763 | changed data and selected unchanged witnesses; qualified roots |
| [agent-episodes-500](raw/verification/mixed_load_bearing/agent-episodes-500/verification.json) | PASS | 19.736787791829556 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-invalid-sdk-edit-compact-v2-proof](raw/verification/workspace_reliability/workspace-invalid-sdk-edit-compact-v2-proof/verification.json.gz) | PASS | 2.3052737498655915 | case-specific lifecycle/fault/recovery contract |
| [workspace-invalid-namespace-compact-v2-proof](raw/verification/workspace_reliability/workspace-invalid-namespace-compact-v2-proof/verification.json.gz) | PASS | 2.0225165407173336 | case-specific lifecycle/fault/recovery contract |
| [workspace-lease-lifecycle-compact-v2-proof](raw/verification/workspace_reliability/workspace-lease-lifecycle-compact-v2-proof/verification.json.gz) | PASS | 2.088033334352076 | case-specific lifecycle/fault/recovery contract |
| [workspace-open-writer-busy-compact-v2-proof](raw/verification/workspace_reliability/workspace-open-writer-busy-compact-v2-proof/verification.json.gz) | PASS | 2.4069364997558296 | case-specific lifecycle/fault/recovery contract |
| [workspace-live-execution-busy-compact-v2-proof](raw/verification/workspace_reliability/workspace-live-execution-busy-compact-v2-proof/verification.json.gz) | PASS | 2.4131747079081833 | case-specific lifecycle/fault/recovery contract |
| [workspace-candidate-failure-retry-compact-v2-proof](raw/verification/workspace_reliability/workspace-candidate-failure-retry-compact-v2-proof/verification.json.gz) | PASS | 1.9956189999356866 | case-specific lifecycle/fault/recovery contract |
| [workspace-admission-batch-failure-retry-compact-v2-proof](raw/verification/workspace_reliability/workspace-admission-batch-failure-retry-compact-v2-proof/verification.json.gz) | PASS | 3.186245792079717 | case-specific lifecycle/fault/recovery contract |
| [workspace-final-publication-failure-retry-compact-v2-proof](raw/verification/workspace_reliability/workspace-final-publication-failure-retry-compact-v2-proof/verification.json.gz) | PASS | 3.505597583949566 | case-specific lifecycle/fault/recovery contract |
| [workspace-published-presentation-failure-smoke-v3-proof](raw/verification/workspace_reliability/workspace-published-presentation-failure-smoke-v3-proof/verification.json.gz) | PASS | 2.0126582919619977 | case-specific lifecycle/fault/recovery contract |
| [workspace-dirty-end-discard-compact-v2-proof](raw/verification/workspace_reliability/workspace-dirty-end-discard-compact-v2-proof/verification.json.gz) | PASS | 2.1298947501927614 | case-specific lifecycle/fault/recovery contract |
| [workspace-dirty-net-zero-compact-v2-proof](raw/verification/workspace_reliability/workspace-dirty-net-zero-compact-v2-proof/verification.json.gz) | PASS | 2.1398541247472167 | case-specific lifecycle/fault/recovery contract |
| [workspace-short-spool-write-compact-v2-proof](raw/verification/workspace_reliability/workspace-short-spool-write-compact-v2-proof/verification.json.gz) | PASS | 2.1939758746884763 | case-specific lifecycle/fault/recovery contract |
| [workspace-deferred-nospace-compact-v2-proof](raw/verification/workspace_reliability/workspace-deferred-nospace-compact-v2-proof/verification.json.gz) | PASS | 2.1209224998019636 | case-specific lifecycle/fault/recovery contract |
| [workspace-workload-cancel-compact-v2-proof](raw/verification/workspace_reliability/workspace-workload-cancel-compact-v2-proof/verification.json.gz) | PASS | 2.2967236251570284 | case-specific lifecycle/fault/recovery contract |
| [workspace-dirty-runtime-disconnect-compact-v2-proof](raw/verification/workspace_reliability/workspace-dirty-runtime-disconnect-compact-v2-proof/verification.json.gz) | PASS | 2.2926473328843713 | case-specific lifecycle/fault/recovery contract |
| [workspace-corrupt-descendant-compact-v2-proof](raw/verification/workspace_reliability/workspace-corrupt-descendant-compact-v2-proof/verification.json) | PASS | 1.7986803753301501 | case-specific lifecycle/fault/recovery contract |
| [workspace-missing-descendant-compact-v2-proof](raw/verification/workspace_reliability/workspace-missing-descendant-compact-v2-proof/verification.json) | PASS | 1.9173369999043643 | case-specific lifecycle/fault/recovery contract |
| [workspace-parallel-read-write-compact-v2-proof](raw/verification/workspace_reliability/workspace-parallel-read-write-compact-v2-proof/verification.json.gz) | PASS | 2.124951582867652 | case-specific lifecycle/fault/recovery contract |
| [workspace-shared-path-contention-compact-v2-proof](raw/verification/workspace_reliability/workspace-shared-path-contention-compact-v2-proof/verification.json.gz) | PASS | 1.964518416672945 | case-specific lifecycle/fault/recovery contract |
| [workspace-hardlink-alias-compact-v2-proof](raw/verification/workspace_reliability/workspace-hardlink-alias-compact-v2-proof/verification.json.gz) | PASS | 1.90862483298406 | case-specific lifecycle/fault/recovery contract |
| [workspace-symlink-semantics-compact-v2-proof](raw/verification/workspace_reliability/workspace-symlink-semantics-compact-v2-proof/verification.json.gz) | PASS | 1.8959957496263087 | case-specific lifecycle/fault/recovery contract |
| [workspace-open-rename-unlink-compact-v2-proof](raw/verification/workspace_reliability/workspace-open-rename-unlink-compact-v2-proof/verification.json.gz) | PASS | 1.863768707960844 | case-specific lifecycle/fault/recovery contract |
| [workspace-metadata-chmod-compact-v2-proof](raw/verification/workspace_reliability/workspace-metadata-chmod-compact-v2-proof/verification.json.gz) | PASS | 2.0118467081338167 | case-specific lifecycle/fault/recovery contract |
| [workspace-metadata-mtime-compact-v2-proof](raw/verification/workspace_reliability/workspace-metadata-mtime-compact-v2-proof/verification.json.gz) | PASS | 2.010027375072241 | case-specific lifecycle/fault/recovery contract |
| [workspace-metadata-xattr-compact-v2-proof](raw/verification/workspace_reliability/workspace-metadata-xattr-compact-v2-proof/verification.json.gz) | PASS | 1.8560614581219852 | case-specific lifecycle/fault/recovery contract |
| [workspace-exec-500-compact-v2-proof](raw/verification/workspace_reliability/workspace-exec-500-compact-v2-proof/verification.json.gz) | PASS | 9.041402082890272 | case-specific lifecycle/fault/recovery contract |
| [workspace-repeat-publication-compact-v2-proof](raw/verification/workspace_reliability/workspace-repeat-publication-compact-v2-proof/verification.json.gz) | PASS | 2.0348000419326127 | case-specific lifecycle/fault/recovery contract |
| workspace-sustained-600s-compact-v2-proof | EXCLUDED_LONG | — | excluded; not executed |

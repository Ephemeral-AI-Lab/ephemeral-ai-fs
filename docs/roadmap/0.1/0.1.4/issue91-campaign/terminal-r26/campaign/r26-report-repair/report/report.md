# v0.1.4 benchmark checkpoint against published v0.1.3

Status: **INCOMPLETE**.

One fixed-seed observation per case. Timers retain their family-specific scopes. Setup and verification are separate. Across-case ranges are not latency distributions. [JSON report](report.json), [performance CSV](performance.csv), and [verification CSV](verification.csv) contain phases, identities, resources, coverage and evidence hashes.

| Family | Performance cases | Proof-only definitions | Performance outcomes | Verification outcomes | Target misses | Across-case range (ms) |
|---|---:|---:|---|---|---:|---|
| dedup_branch_history | 20 | 0 | {'PASS': 20} | {'PASS': 20} | 1 | 28.195–25637.566 |
| dedup_cdc_locality | 20 | 1 | {'PASS': 20} | {'PASS': 21} | 0 | 5.958–1332.844 |
| dedup_cross_file | 10 | 0 | {'PASS': 10} | {'PASS': 10} | 0 | 6.508–1442.648 |
| dedup_workspace_reuse | 14 | 0 | {'PASS': 14} | {'PASS': 14} | 0 | 31.679–3973.390 |
| directory_construction_traversal | 12 | 0 | {'PASS': 12} | {'PASS': 12} | 0 | 25.454–5415.185 |
| edit_canonical_chunk_count | 12 | 0 | {'PASS': 12} | {'PASS': 12} | 0 | 8.238–13.068 |
| edit_length_changing | 32 | 0 | {'PASS': 32} | {'PASS': 32} | 0 | 7.474–14.170 |
| edit_length_preserving | 12 | 0 | {'PASS': 12} | {'PASS': 12} | 0 | 7.084–12.249 |
| git_tool_workflow | 4 | 0 | {'PASS': 4} | {'PASS': 4} | 2 | 396.925–5806.735 |
| init_namespace | 4 | 0 | {'PASS': 4} | {'PASS': 4} | 0 | 19.893–3803.760 |
| mixed_load_bearing | 4 | 0 | {'PASS': 4} | {'PASS': 4} | 0 | 33.485–8523.980 |
| namespace_mutation | 4 | 0 | {'PASS': 4} | {'PASS': 4} | 0 | 27.012–245.514 |
| payload_create_read | 8 | 0 | {'PASS': 8} | {'PASS': 8} | 0 | 17.124–2701.168 |
| store_footprint | 6 | 0 | {'PASS': 6} | {'PASS': 6} | 0 | 47.905–6897.714 |
| tiny_file_churn | 20 | 0 | {'PASS': 20} | {'PASS': 20} | 0 | 19.920–5319.530 |
| workspace_change_locality | 16 | 0 | {'PASS': 16} | {'PASS': 16} | 0 | 12.446–7665.527 |
| workspace_reliability | 0 | 28 | {} | {'PASS': 27, 'NOT_RUN_OPTIONAL': 1} | 0 | —–— |

## dedup_branch_history

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [dedup-history-distributed-1](../view/performance/dedup_branch_history/dedup-history-distributed-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 31.439 | — / 12.243 / 5.758 | 11.78 / 5.66 | 22.163 | PASS / PASS | PASS |
| [dedup-history-distributed-10](../view/performance/dedup_branch_history/dedup-history-distributed-10/perf.jsonl) | 10 / 200 / 1048576 | pure_call_sum_ns | 90.197 | — / 37.222 / 41.474 | 12.70 / 5.40 | 82.266 | PASS / PASS | PASS |
| [dedup-history-distributed-100](../view/performance/dedup_branch_history/dedup-history-distributed-100/perf.jsonl) | 100 / 200 / 1048576 | pure_call_sum_ns | 694.219 | — / 280.261 / 401.995 | 16.89 / 10.85 | 586.405 | PASS / PASS | PASS |
| [dedup-history-distributed-500](../view/performance/dedup_branch_history/dedup-history-distributed-500/perf.jsonl) | 500 / 200 / 1048576 | pure_call_sum_ns | 3175.247 | — / 1330.917 / 1832.303 | 26.75 / 42.20 | 2925.218 | PASS / PASS | PASS |
| [dedup-history-hotset-1](../view/performance/dedup_branch_history/dedup-history-hotset-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 29.028 | — / 10.594 / 5.772 | 11.78 / 5.14 | 23.580 | PASS / PASS | PASS |
| [dedup-history-hotset-10](../view/performance/dedup_branch_history/dedup-history-hotset-10/perf.jsonl) | 10 / 200 / 1048576 | pure_call_sum_ns | 140.844 | — / 84.523 / 44.986 | 13.05 / 5.42 | 118.461 | PASS / PASS | PASS |
| [dedup-history-hotset-100](../view/performance/dedup_branch_history/dedup-history-hotset-100/perf.jsonl) | 100 / 200 / 1048576 | pure_call_sum_ns | 1163.198 | — / 721.406 / 429.124 | 16.92 / 12.69 | 799.500 | PASS / PASS | PASS |
| [dedup-history-hotset-500](../view/performance/dedup_branch_history/dedup-history-hotset-500/perf.jsonl) | 500 / 200 / 1048576 | pure_call_sum_ns | 4143.199 | — / 2478.239 / 1651.523 | 25.25 / 43.91 | 3683.755 | PASS / PASS | PASS |
| [dedup-history-recurring-1](../view/performance/dedup_branch_history/dedup-history-recurring-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 35.136 | — / 12.533 / 7.668 | 12.84 / 4.95 | 21.456 | PASS / PASS | PASS |
| [dedup-history-recurring-10](../view/performance/dedup_branch_history/dedup-history-recurring-10/perf.jsonl) | 10 / 200 / 1048576 | pure_call_sum_ns | 91.314 | — / 32.319 / 47.475 | 13.16 / 5.66 | 62.918 | PASS / PASS | PASS |
| [dedup-history-recurring-100](../view/performance/dedup_branch_history/dedup-history-recurring-100/perf.jsonl) | 100 / 200 / 1048576 | pure_call_sum_ns | 734.401 | — / 239.792 / 481.867 | 15.33 / 7.11 | 462.841 | PASS / PASS | PASS |
| [dedup-history-recurring-500](../view/performance/dedup_branch_history/dedup-history-recurring-500/perf.jsonl) | 500 / 200 / 1048576 | pure_call_sum_ns | 3045.772 | — / 1019.229 / 2013.385 | 18.59 / 5.17 | 2145.416 | PASS / PASS | PASS |
| [dedup-history-metadata-1](../view/performance/dedup_branch_history/dedup-history-metadata-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 28.195 | 10.168 / — / 4.700 | 9.44 / 5.39 | 20.653 | PASS / PASS | PASS |
| [dedup-history-metadata-10](../view/performance/dedup_branch_history/dedup-history-metadata-10/perf.jsonl) | 10 / 200 / 1048576 | pure_call_sum_ns | 94.299 | 48.366 / — / 33.203 | 9.89 / 5.40 | 76.474 | PASS / PASS | PASS |
| [dedup-history-metadata-100](../view/performance/dedup_branch_history/dedup-history-metadata-100/perf.jsonl) | 100 / 200 / 1048576 | pure_call_sum_ns | 787.526 | 459.482 / — / 314.838 | 12.36 / 5.15 | 649.605 | PASS / PASS | PASS |
| [dedup-history-metadata-500](../view/performance/dedup_branch_history/dedup-history-metadata-500/perf.jsonl) | 500 / 200 / 1048576 | pure_call_sum_ns | 4041.859 | 2367.563 / — / 1658.595 | 15.02 / 4.98 | 3231.967 | PASS / PASS | PASS |
| [dedup-history-unrelated-1](../view/performance/dedup_branch_history/dedup-history-unrelated-1/perf.jsonl) | 1 / 200 / 1048576 | pure_call_sum_ns | 1171.338 | 1114.516 / — / 45.133 | 17.89 / 7.78 | 880.164 | PASS / PASS | PASS |
| [dedup-history-unrelated-10](../view/performance/dedup_branch_history/dedup-history-unrelated-10/perf.jsonl) | 10 / 200 / 1048576 | pure_call_sum_ns | 12093.766 | 11556.699 / — / 521.013 | 33.25 / 16.57 | 9543.475 | PASS / PASS | PASS |
| [dedup-history-unrelated-100-mixed-v2](../view/performance/dedup_branch_history/dedup-history-unrelated-100-mixed-v2/perf.jsonl) | 100 / 10 / 1048576 | pure_call_sum_ns | 5125.324 | 3283.278 / — / 1829.277 | 57.03 / 8.36 | 3549.609 | PASS / PASS | PASS |
| [dedup-history-unrelated-500-mixed-v2](../view/performance/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/perf.jsonl) | 500 / 10 / 1048576 | pure_call_sum_ns | 25637.566 | 16197.321 / — / 9425.461 | 66.42 / 10.57 | 18163.889 | PASS / PASS | TARGET_MISS |

## dedup_cdc_locality

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [dedup-cdc-overwrite-1](../view/performance/dedup_cdc_locality/dedup-cdc-overwrite-1/perf.jsonl) | 1 / 2 / 2097152 | pure_call_sum_ns | 7.247 | — / — / — | 14.62 / 5.40 | 4.002 | PASS / PASS | PASS |
| [dedup-cdc-overwrite-10](../view/performance/dedup_cdc_locality/dedup-cdc-overwrite-10/perf.jsonl) | 10 / 11 / 11534336 | pure_call_sum_ns | 19.061 | — / — / — | 17.56 / 5.60 | 20.448 | PASS / PASS | PASS |
| [dedup-cdc-overwrite-100](../view/performance/dedup_cdc_locality/dedup-cdc-overwrite-100/perf.jsonl) | 100 / 101 / 105906176 | pure_call_sum_ns | 143.659 | — / — / — | 27.73 / 5.39 | 30.953 | PASS / PASS | PASS |
| [dedup-cdc-overwrite-500](../view/performance/dedup_cdc_locality/dedup-cdc-overwrite-500/perf.jsonl) | 500 / 501 / 525336576 | pure_call_sum_ns | 769.799 | — / — / — | 38.66 / 5.15 | 122.035 | PASS / PASS | PASS |
| [dedup-cdc-insert-1](../view/performance/dedup_cdc_locality/dedup-cdc-insert-1/perf.jsonl) | 1 / 2 / 2101248 | pure_call_sum_ns | 5.958 | — / — / — | 16.06 / 5.66 | 3.897 | PASS / PASS | PASS |
| [dedup-cdc-insert-10](../view/performance/dedup_cdc_locality/dedup-cdc-insert-10/perf.jsonl) | 10 / 11 / 11575296 | pure_call_sum_ns | 19.471 | — / — / — | 19.34 / 5.40 | 18.058 | PASS / PASS | PASS |
| [dedup-cdc-insert-100](../view/performance/dedup_cdc_locality/dedup-cdc-insert-100/perf.jsonl) | 100 / 101 / 106315776 | pure_call_sum_ns | 158.100 | — / — / — | 29.42 / 5.64 | 34.183 | PASS / PASS | PASS |
| [dedup-cdc-insert-500](../view/performance/dedup_cdc_locality/dedup-cdc-insert-500/perf.jsonl) | 500 / 501 / 527384576 | pure_call_sum_ns | 775.264 | — / — / — | 42.30 / 5.34 | 136.776 | PASS / PASS | PASS |
| [dedup-cdc-delete-1](../view/performance/dedup_cdc_locality/dedup-cdc-delete-1/perf.jsonl) | 1 / 2 / 2093056 | pure_call_sum_ns | 6.298 | — / — / — | 15.88 / 5.14 | 4.412 | PASS / PASS | PASS |
| [dedup-cdc-delete-10](../view/performance/dedup_cdc_locality/dedup-cdc-delete-10/perf.jsonl) | 10 / 11 / 11493376 | pure_call_sum_ns | 19.955 | — / — / — | 20.30 / 5.09 | 18.225 | PASS / PASS | PASS |
| [dedup-cdc-delete-100](../view/performance/dedup_cdc_locality/dedup-cdc-delete-100/perf.jsonl) | 100 / 101 / 105496576 | pure_call_sum_ns | 150.030 | — / — / — | 29.06 / 5.17 | 33.166 | PASS / PASS | PASS |
| [dedup-cdc-delete-500](../view/performance/dedup_cdc_locality/dedup-cdc-delete-500/perf.jsonl) | 500 / 501 / 523288576 | pure_call_sum_ns | 765.470 | — / — / — | 39.66 / 5.17 | 130.864 | PASS / PASS | PASS |
| [dedup-cdc-common-body-1](../view/performance/dedup_cdc_locality/dedup-cdc-common-body-1/perf.jsonl) | 1 / 2 / 2097152 | pure_call_sum_ns | 6.692 | — / — / — | 15.86 / 5.39 | 4.125 | PASS / PASS | PASS |
| [dedup-cdc-common-body-10](../view/performance/dedup_cdc_locality/dedup-cdc-common-body-10/perf.jsonl) | 10 / 11 / 11534336 | pure_call_sum_ns | 25.391 | — / — / — | 24.48 / 5.43 | 20.395 | PASS / PASS | PASS |
| [dedup-cdc-common-body-100](../view/performance/dedup_cdc_locality/dedup-cdc-common-body-100/perf.jsonl) | 100 / 101 / 105906176 | pure_call_sum_ns | 189.510 | — / — / — | 57.78 / 5.14 | 58.208 | PASS / PASS | PASS |
| [dedup-cdc-common-body-500](../view/performance/dedup_cdc_locality/dedup-cdc-common-body-500/perf.jsonl) | 500 / 501 / 525336576 | pure_call_sum_ns | 990.060 | — / — / — | 60.94 / 5.17 | 244.350 | PASS / PASS | PASS |
| [dedup-cdc-scattered-1](../view/performance/dedup_cdc_locality/dedup-cdc-scattered-1/perf.jsonl) | 1 / 2 / 2097152 | pure_call_sum_ns | 8.233 | — / — / — | 17.47 / 5.39 | 4.647 | PASS / PASS | PASS |
| [dedup-cdc-scattered-10](../view/performance/dedup_cdc_locality/dedup-cdc-scattered-10/perf.jsonl) | 10 / 11 / 11534336 | pure_call_sum_ns | 34.553 | — / — / — | 33.47 / 5.69 | 23.469 | PASS / PASS | PASS |
| [dedup-cdc-scattered-100](../view/performance/dedup_cdc_locality/dedup-cdc-scattered-100/perf.jsonl) | 100 / 101 / 105906176 | pure_call_sum_ns | 306.409 | — / — / — | 60.80 / 5.92 | 89.368 | PASS / PASS | PASS |
| [dedup-cdc-scattered-500](../view/performance/dedup_cdc_locality/dedup-cdc-scattered-500/perf.jsonl) | 500 / 501 / 525336576 | pure_call_sum_ns | 1332.844 | — / — / — | 63.00 / 5.11 | 483.007 | PASS / PASS | PASS |

## dedup_cross_file

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [dedup-cross-file-anchor-1](../view/performance/dedup_cross_file/dedup-cross-file-anchor-1/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 6.508 | — / — / — | 13.98 / 5.34 | 3.808 | PASS / PASS | PASS |
| [dedup-cross-file-unique-10](../view/performance/dedup_cross_file/dedup-cross-file-unique-10/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 33.044 | — / — / — | 30.52 / 5.15 | 22.390 | PASS / PASS | PASS |
| [dedup-cross-file-unique-100](../view/performance/dedup_cross_file/dedup-cross-file-unique-100/perf.jsonl) | 100 / 100 / 104857600 | pure_call_sum_ns | 296.813 | — / — / — | 60.11 / 5.65 | 93.267 | PASS / PASS | PASS |
| [dedup-cross-file-unique-500](../view/performance/dedup_cross_file/dedup-cross-file-unique-500/perf.jsonl) | 500 / 500 / 524288000 | pure_call_sum_ns | 1442.648 | — / — / — | 63.33 / 5.39 | 444.949 | PASS / PASS | PASS |
| [dedup-cross-file-identical-10](../view/performance/dedup_cross_file/dedup-cross-file-identical-10/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 19.478 | — / — / — | 18.03 / 5.41 | 18.395 | PASS / PASS | PASS |
| [dedup-cross-file-identical-100](../view/performance/dedup_cross_file/dedup-cross-file-identical-100/perf.jsonl) | 100 / 100 / 104857600 | pure_call_sum_ns | 125.687 | — / — / — | 22.20 / 5.18 | 28.960 | PASS / PASS | PASS |
| [dedup-cross-file-identical-500](../view/performance/dedup_cross_file/dedup-cross-file-identical-500/perf.jsonl) | 500 / 500 / 524288000 | pure_call_sum_ns | 682.649 | — / — / — | 22.94 / 5.15 | 110.630 | PASS / PASS | PASS |
| [dedup-cross-file-mixed-10](../view/performance/dedup_cross_file/dedup-cross-file-mixed-10/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 30.755 | — / — / — | 27.80 / 5.41 | 24.046 | PASS / PASS | PASS |
| [dedup-cross-file-mixed-100](../view/performance/dedup_cross_file/dedup-cross-file-mixed-100/perf.jsonl) | 100 / 100 / 104857600 | pure_call_sum_ns | 296.843 | — / — / — | 60.31 / 5.15 | 113.944 | PASS / PASS | PASS |
| [dedup-cross-file-mixed-500](../view/performance/dedup_cross_file/dedup-cross-file-mixed-500/perf.jsonl) | 500 / 500 / 524288000 | pure_call_sum_ns | 1350.345 | — / — / — | 63.84 / 5.42 | 541.992 | PASS / PASS | PASS |

## dedup_workspace_reuse

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [dedup-workspace-exact-1-compact-v2](../view/performance/dedup_workspace_reuse/dedup-workspace-exact-1-compact-v2/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 36.174 | 13.923 / — / 9.502 | 12.03 / 7.47 | 31.563 | PASS / PASS | PASS |
| [dedup-workspace-exact-10-compact-v2](../view/performance/dedup_workspace_reuse/dedup-workspace-exact-10-compact-v2/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 94.424 | 49.605 / — / 32.667 | 25.41 / 10.24 | 109.325 | PASS / PASS | PASS |
| [dedup-workspace-exact-100](../view/performance/dedup_workspace_reuse/dedup-workspace-exact-100/perf.jsonl) | 100 / 128 / 134217728 | pure_call_sum_ns | 718.356 | 418.488 / — / 282.593 | 54.88 / 12.55 | 840.528 | PASS / PASS | PASS |
| [dedup-workspace-exact-500](../view/performance/dedup_workspace_reuse/dedup-workspace-exact-500/perf.jsonl) | 500 / 128 / 134217728 | pure_call_sum_ns | 3925.196 | 2614.095 / — / 1293.431 | 56.45 / 12.79 | 4296.598 | PASS / PASS | PASS |
| [dedup-workspace-local-1-compact-v2](../view/performance/dedup_workspace_reuse/dedup-workspace-local-1-compact-v2/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 31.679 | 11.498 / — / 9.400 | 12.39 / 7.20 | 30.610 | PASS / PASS | PASS |
| [dedup-workspace-local-10-compact-v2](../view/performance/dedup_workspace_reuse/dedup-workspace-local-10-compact-v2/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 95.430 | 49.562 / — / 33.063 | 26.48 / 10.31 | 103.153 | PASS / PASS | PASS |
| [dedup-workspace-local-100](../view/performance/dedup_workspace_reuse/dedup-workspace-local-100/perf.jsonl) | 100 / 128 / 134217728 | pure_call_sum_ns | 793.133 | 437.885 / — / 337.872 | 57.53 / 11.79 | 808.387 | PASS / PASS | PASS |
| [dedup-workspace-local-500](../view/performance/dedup_workspace_reuse/dedup-workspace-local-500/perf.jsonl) | 500 / 128 / 134217728 | pure_call_sum_ns | 3973.390 | 2576.972 / — / 1380.217 | 59.91 / 12.86 | 4307.360 | PASS / PASS | PASS |
| [dedup-workspace-unique-1-compact-v2](../view/performance/dedup_workspace_reuse/dedup-workspace-unique-1-compact-v2/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 37.899 | 13.682 / — / 10.871 | 14.08 / 7.25 | 27.475 | PASS / PASS | PASS |
| [dedup-workspace-unique-10-compact-v2](../view/performance/dedup_workspace_reuse/dedup-workspace-unique-10-compact-v2/perf.jsonl) | 10 / 10 / 10485760 | pure_call_sum_ns | 105.688 | 52.313 / — / 41.975 | 32.41 / 10.02 | 95.669 | PASS / PASS | PASS |
| [dedup-workspace-unique-100](../view/performance/dedup_workspace_reuse/dedup-workspace-unique-100/perf.jsonl) | 100 / 128 / 134217728 | pure_call_sum_ns | 831.454 | 455.038 / — / 360.944 | 59.23 / 12.39 | 768.004 | PASS / PASS | PASS |
| [dedup-workspace-unique-500](../view/performance/dedup_workspace_reuse/dedup-workspace-unique-500/perf.jsonl) | 500 / 128 / 134217728 | pure_call_sum_ns | 3936.558 | 2575.201 / — / 1343.396 | 61.39 / 12.93 | 4319.934 | PASS / PASS | PASS |
| [dedup-workspace-unique-1-base128-v3](../view/performance/dedup_workspace_reuse/dedup-workspace-unique-1-base128-v3/perf.jsonl) | 1 / 128 / 134217728 | pure_call_sum_ns | 32.976 | 10.414 / — / 10.118 | 17.16 / 7.14 | 33.382 | PASS / PASS | PASS |
| [dedup-workspace-unique-10-base128-v3](../view/performance/dedup_workspace_reuse/dedup-workspace-unique-10-base128-v3/perf.jsonl) | 10 / 128 / 134217728 | pure_call_sum_ns | 106.993 | 48.256 / — / 46.167 | 35.64 / 10.30 | 103.007 | PASS / PASS | PASS |

## directory_construction_traversal

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [directory-construct-1-compact-v2](../view/performance/directory_construction_traversal/directory-construct-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 25.454 | 6.998 / — / 5.174 | 8.69 / 5.68 | 17.022 | PASS / PASS | PASS |
| [directory-construct-10-compact-v2](../view/performance/directory_construction_traversal/directory-construct-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 47.305 | 24.295 / — / 9.573 | 9.91 / 7.40 | 39.347 | PASS / PASS | PASS |
| [directory-construct-100-mixed-v4](../view/performance/directory_construction_traversal/directory-construct-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 339.072 | 289.401 / — / 32.608 | 16.70 / 5.39 | 216.060 | PASS / PASS | PASS |
| [directory-construct-500-mixed-v4](../view/performance/directory_construction_traversal/directory-construct-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 1245.070 | 968.687 / — / 262.788 | 35.64 / 11.75 | 1030.581 | PASS / PASS | PASS |
| [directory-metadata-scan-1-compact-v2](../view/performance/directory_construction_traversal/directory-metadata-scan-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 95.407 | 80.682 / — / 1.870 | 8.28 / 5.15 | 70.527 | PASS / PASS | PASS |
| [directory-metadata-scan-10-compact-v2](../view/performance/directory_construction_traversal/directory-metadata-scan-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 98.781 | 83.640 / — / 2.295 | 11.33 / 5.47 | 100.739 | PASS / PASS | PASS |
| [directory-metadata-scan-100-mixed-v4](../view/performance/directory_construction_traversal/directory-metadata-scan-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 344.609 | 325.867 / — / 3.585 | 23.67 / 16.25 | 244.239 | PASS / PASS | PASS |
| [directory-metadata-scan-500-mixed-v4](../view/performance/directory_construction_traversal/directory-metadata-scan-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 676.262 | 651.623 / — / 6.054 | 47.94 / 33.70 | 504.932 | PASS / PASS | PASS |
| [directory-content-scan-1-compact-v2](../view/performance/directory_construction_traversal/directory-content-scan-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 108.694 | 96.494 / — / 1.941 | 9.64 / 7.47 | 84.760 | PASS / PASS | PASS |
| [directory-content-scan-10-compact-v2](../view/performance/directory_construction_traversal/directory-content-scan-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 394.741 | 379.681 / — / 3.289 | 20.14 / 29.10 | 308.870 | PASS / PASS | PASS |
| [directory-content-scan-100-mixed-v4](../view/performance/directory_construction_traversal/directory-content-scan-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 1406.812 | 1383.158 / — / 7.618 | 47.00 / 174.73 | 1105.141 | PASS / PASS | PASS |
| [directory-content-scan-500-mixed-v4](../view/performance/directory_construction_traversal/directory-content-scan-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 5415.185 | 5364.746 / — / 22.547 | 53.38 / 594.67 | 3906.595 | PASS / PASS | PASS |

## edit_canonical_chunk_count

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 10.110 | — / 2.603 / 7.507 | 29.48 / 5.19 | 6.855 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 10.518 | — / 2.455 / 8.063 | 29.17 / 5.18 | 8.507 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 10.190 | — / 2.274 / 7.917 | 29.34 / 5.65 | 7.335 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 13.068 | — / 3.405 / 9.663 | 29.39 / 5.41 | 8.270 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 9.833 | — / 2.575 / 7.258 | 29.55 / 5.17 | 7.852 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 12.497 | — / 3.188 / 9.309 | 29.62 / 5.40 | 7.516 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 11.893 | — / 2.836 / 9.057 | 29.41 / 5.16 | 7.870 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 11.197 | — / 2.243 / 8.954 | 29.33 / 5.66 | 8.466 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 8.413 | — / 1.988 / 6.425 | 29.69 / 5.40 | 7.175 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 8.238 | — / 2.152 / 6.086 | 29.69 / 5.14 | 7.013 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 9.331 | — / 2.410 / 6.921 | 29.53 / 5.40 | 7.582 | PASS / PASS | PASS |
| [overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1](../view/performance/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 12.063 | — / 2.689 / 9.374 | 29.47 / 5.65 | 8.095 | PASS / PASS | PASS |

## edit_length_changing

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [insert-middle-4k-on-1mib-ops-1](../view/performance/edit_length_changing/insert-middle-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 8.822 | — / 2.131 / 6.690 | 29.67 / 7.15 | 5.590 | PASS / PASS | PASS |
| [insert-middle-4k-on-10mib-ops-1](../view/performance/edit_length_changing/insert-middle-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 9.767 | — / 2.439 / 7.328 | 29.61 / 5.15 | 7.396 | PASS / PASS | PASS |
| [insert-middle-4k-on-100mib-ops-1](../view/performance/edit_length_changing/insert-middle-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 10.679 | — / 2.448 / 8.231 | 29.52 / 5.16 | 12.439 | PASS / PASS | PASS |
| [insert-middle-4k-on-500mib-result-capped-v2-ops-1](../view/performance/edit_length_changing/insert-middle-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524283904 | edit_commit_ns | 9.690 | — / 1.996 / 7.694 | 29.69 / 4.90 | 18.011 | PASS / PASS | PASS |
| [delete-middle-4k-on-1mib-ops-1](../view/performance/edit_length_changing/delete-middle-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 9.300 | — / 2.474 / 6.825 | 29.62 / 5.42 | 5.290 | PASS / PASS | PASS |
| [delete-middle-4k-on-10mib-ops-1](../view/performance/edit_length_changing/delete-middle-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 8.249 | — / 1.983 / 6.266 | 29.64 / 7.65 | 6.570 | PASS / PASS | PASS |
| [delete-middle-4k-on-100mib-ops-1](../view/performance/edit_length_changing/delete-middle-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 9.840 | — / 2.755 / 7.085 | 29.45 / 5.40 | 6.758 | PASS / PASS | PASS |
| [delete-middle-4k-on-500mib-ops-1](../view/performance/edit_length_changing/delete-middle-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 10.336 | — / 2.322 / 8.014 | 29.59 / 5.65 | 7.488 | PASS / PASS | PASS |
| [append-tail-4k-on-1mib-ops-1](../view/performance/edit_length_changing/append-tail-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 8.331 | — / 2.252 / 6.079 | 29.80 / 5.40 | 6.261 | PASS / PASS | PASS |
| [append-tail-4k-on-10mib-ops-1](../view/performance/edit_length_changing/append-tail-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 7.838 | — / 1.943 / 5.895 | 29.53 / 5.15 | 5.905 | PASS / PASS | PASS |
| [append-tail-4k-on-100mib-ops-1](../view/performance/edit_length_changing/append-tail-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 7.737 | — / 2.048 / 5.689 | 29.56 / 5.16 | 6.255 | PASS / PASS | PASS |
| [append-tail-4k-on-500mib-result-capped-v2-ops-1](../view/performance/edit_length_changing/append-tail-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524283904 | edit_commit_ns | 9.916 | — / 2.321 / 7.596 | 29.47 / 5.14 | 6.686 | PASS / PASS | PASS |
| [prepend-head-4k-on-1mib-ops-1](../view/performance/edit_length_changing/prepend-head-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 9.342 | — / 2.959 / 6.383 | 29.78 / 7.59 | 5.905 | PASS / PASS | PASS |
| [prepend-head-4k-on-10mib-ops-1](../view/performance/edit_length_changing/prepend-head-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 9.153 | — / 2.551 / 6.602 | 29.41 / 5.14 | 5.672 | PASS / PASS | PASS |
| [prepend-head-4k-on-100mib-ops-1](../view/performance/edit_length_changing/prepend-head-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 9.046 | — / 2.657 / 6.389 | 29.20 / 5.40 | 6.269 | PASS / PASS | PASS |
| [prepend-head-4k-on-500mib-result-capped-v2-ops-1](../view/performance/edit_length_changing/prepend-head-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524283904 | edit_commit_ns | 9.858 | — / 2.131 / 7.727 | 29.52 / 5.91 | 7.099 | PASS / PASS | PASS |
| [replace-grow-middle-2k-to-4k-on-1mib-ops-1](../view/performance/edit_length_changing/replace-grow-middle-2k-to-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 13.716 | — / 4.320 / 9.395 | 29.47 / 5.42 | 5.744 | PASS / PASS | PASS |
| [replace-grow-middle-2k-to-4k-on-10mib-ops-1](../view/performance/edit_length_changing/replace-grow-middle-2k-to-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 7.474 | — / 1.755 / 5.720 | 29.64 / 5.64 | 6.463 | PASS / PASS | PASS |
| [replace-grow-middle-2k-to-4k-on-100mib-ops-1](../view/performance/edit_length_changing/replace-grow-middle-2k-to-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 14.170 | — / 5.581 / 8.589 | 29.58 / 5.65 | 6.382 | PASS / PASS | PASS |
| [replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1](../view/performance/edit_length_changing/replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524285952 | edit_commit_ns | 10.596 | — / 2.483 / 8.113 | 29.06 / 5.40 | 7.922 | PASS / PASS | PASS |
| [replace-shrink-middle-4k-to-2k-on-1mib-ops-1](../view/performance/edit_length_changing/replace-shrink-middle-4k-to-2k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 7.777 | — / 2.048 / 5.729 | 29.28 / 5.15 | 5.880 | PASS / PASS | PASS |
| [replace-shrink-middle-4k-to-2k-on-10mib-ops-1](../view/performance/edit_length_changing/replace-shrink-middle-4k-to-2k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 9.975 | — / 2.631 / 7.344 | 29.52 / 5.40 | 5.770 | PASS / PASS | PASS |
| [replace-shrink-middle-4k-to-2k-on-100mib-ops-1](../view/performance/edit_length_changing/replace-shrink-middle-4k-to-2k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 9.390 | — / 1.968 / 7.422 | 29.67 / 5.43 | 7.008 | PASS / PASS | PASS |
| [replace-shrink-middle-4k-to-2k-on-500mib-ops-1](../view/performance/edit_length_changing/replace-shrink-middle-4k-to-2k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 12.270 | — / 2.543 / 9.727 | 29.48 / 5.39 | 6.997 | PASS / PASS | PASS |
| [truncate-tail-4k-on-1mib-ops-1](../view/performance/edit_length_changing/truncate-tail-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 8.425 | — / 1.733 / 6.691 | 29.59 / 5.65 | 5.293 | PASS / PASS | PASS |
| [truncate-tail-4k-on-10mib-ops-1](../view/performance/edit_length_changing/truncate-tail-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 8.465 | — / 3.067 / 5.398 | 29.38 / 5.68 | 15.641 | PASS / PASS | PASS |
| [truncate-tail-4k-on-100mib-ops-1](../view/performance/edit_length_changing/truncate-tail-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 10.382 | — / 3.250 / 7.132 | 29.72 / 7.65 | 6.528 | PASS / PASS | PASS |
| [truncate-tail-4k-on-500mib-ops-1](../view/performance/edit_length_changing/truncate-tail-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 9.531 | — / 2.231 / 7.300 | 29.58 / 5.40 | 6.895 | PASS / PASS | PASS |
| [zero-extend-tail-4k-on-1mib-ops-1](../view/performance/edit_length_changing/zero-extend-tail-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 10.496 | — / 3.188 / 7.308 | 29.39 / 5.39 | 5.633 | PASS / PASS | PASS |
| [zero-extend-tail-4k-on-10mib-ops-1](../view/performance/edit_length_changing/zero-extend-tail-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 7.955 | — / 2.205 / 5.750 | 29.55 / 5.43 | 5.638 | PASS / PASS | PASS |
| [zero-extend-tail-4k-on-100mib-ops-1](../view/performance/edit_length_changing/zero-extend-tail-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 8.437 | — / 2.288 / 6.149 | 29.56 / 5.39 | 6.404 | PASS / PASS | PASS |
| [zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1](../view/performance/edit_length_changing/zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1/perf.jsonl) | 500 / 1 / 524283904 | edit_commit_ns | 9.305 | — / 2.224 / 7.081 | 29.41 / 5.90 | 6.664 | PASS / PASS | PASS |

## edit_length_preserving

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [overwrite-head-4k-on-1mib-ops-1](../view/performance/edit_length_preserving/overwrite-head-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 7.945 | — / 1.926 / 6.019 | 29.98 / 5.14 | 5.777 | PASS / PASS | PASS |
| [overwrite-head-4k-on-10mib-ops-1](../view/performance/edit_length_preserving/overwrite-head-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 8.576 | — / 2.227 / 6.349 | 29.62 / 7.43 | 6.154 | PASS / PASS | PASS |
| [overwrite-head-4k-on-100mib-ops-1](../view/performance/edit_length_preserving/overwrite-head-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 7.945 | — / 2.194 / 5.750 | 29.58 / 5.14 | 6.652 | PASS / PASS | PASS |
| [overwrite-head-4k-on-500mib-ops-1](../view/performance/edit_length_preserving/overwrite-head-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 9.620 | — / 2.420 / 7.200 | 29.42 / 5.66 | 8.411 | PASS / PASS | PASS |
| [overwrite-middle-4k-on-1mib-ops-1](../view/performance/edit_length_preserving/overwrite-middle-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 7.084 | — / 1.986 / 5.098 | 29.23 / 5.40 | 6.139 | PASS / PASS | PASS |
| [overwrite-middle-4k-on-10mib-ops-1](../view/performance/edit_length_preserving/overwrite-middle-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 12.249 | — / 4.104 / 8.145 | 29.91 / 5.39 | 5.925 | PASS / PASS | PASS |
| [overwrite-middle-4k-on-100mib-ops-1](../view/performance/edit_length_preserving/overwrite-middle-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 9.383 | — / 2.277 / 7.106 | 29.72 / 5.68 | 6.188 | PASS / PASS | PASS |
| [overwrite-middle-4k-on-500mib-ops-1](../view/performance/edit_length_preserving/overwrite-middle-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 11.123 | — / 2.531 / 8.592 | 29.55 / 5.34 | 7.174 | PASS / PASS | PASS |
| [overwrite-tail-4k-on-1mib-ops-1](../view/performance/edit_length_preserving/overwrite-tail-4k-on-1mib-ops-1/perf.jsonl) | 1 / 1 / 1048576 | edit_commit_ns | 7.901 | — / 1.907 / 5.994 | 29.64 / 5.41 | 6.112 | PASS / PASS | PASS |
| [overwrite-tail-4k-on-10mib-ops-1](../view/performance/edit_length_preserving/overwrite-tail-4k-on-10mib-ops-1/perf.jsonl) | 10 / 1 / 10485760 | edit_commit_ns | 8.921 | — / 2.464 / 6.457 | 29.66 / 5.64 | 7.265 | PASS / PASS | PASS |
| [overwrite-tail-4k-on-100mib-ops-1](../view/performance/edit_length_preserving/overwrite-tail-4k-on-100mib-ops-1/perf.jsonl) | 100 / 1 / 104857600 | edit_commit_ns | 9.244 | — / 2.404 / 6.840 | 29.36 / 4.90 | 7.631 | PASS / PASS | PASS |
| [overwrite-tail-4k-on-500mib-ops-1](../view/performance/edit_length_preserving/overwrite-tail-4k-on-500mib-ops-1/perf.jsonl) | 500 / 1 / 524288000 | edit_commit_ns | 10.039 | — / 2.294 / 7.745 | 29.52 / 5.43 | 9.378 | PASS / PASS | PASS |

## git_tool_workflow

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [git-tool-1-compact-v2](../view/performance/git_tool_workflow/git-tool-1-compact-v2/perf.jsonl) | 1 / 51 / 1051076 | pure_call_sum_ns | 396.925 | 372.348 / — / 9.292 | 13.47 / 11.80 | — | PASS / PASS | PASS |
| [git-tool-10-compact-v2](../view/performance/git_tool_workflow/git-tool-10-compact-v2/perf.jsonl) | 10 / 207 / 4211804 | pure_call_sum_ns | 769.868 | 733.841 / — / 19.630 | 21.03 / 26.75 | — | PASS / PASS | PASS |
| [git-tool-100-mixed-v4](../view/performance/git_tool_workflow/git-tool-100-mixed-v4/perf.jsonl) | 100 / 2351 / 61491723 | pure_call_sum_ns | 2396.650 | 2312.376 / — / 54.785 | 45.66 / 70.51 | — | PASS / PASS | TARGET_MISS |
| [git-tool-500-mixed-v4](../view/performance/git_tool_workflow/git-tool-500-mixed-v4/perf.jsonl) | 500 / 5351 / 73779723 | pure_call_sum_ns | 5806.735 | 5600.568 / — / 166.783 | 71.34 / 136.28 | — | PASS / PASS | TARGET_MISS |

## init_namespace

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [namespace-100-compact-v3](../view/performance/init_namespace/namespace-100-compact-v3/perf.jsonl) | 100 / 100 / 5000000 | layerstack_init_ns | 19.893 | — / — / — | 26.19 / 5.39 | 8.902 | PASS / PASS | PASS |
| [namespace-1000-compact-v3](../view/performance/init_namespace/namespace-1000-compact-v3/perf.jsonl) | 1000 / 1000 / 20000000 | layerstack_init_ns | 81.556 | — / — / — | 47.84 / 5.65 | 38.077 | PASS / PASS | PASS |
| [namespace-10000](../view/performance/init_namespace/namespace-10000/perf.jsonl) | 10000 / 10000 / 300000000 | layerstack_init_ns | 997.947 | — / — / — | 68.95 / 5.09 | 403.468 | PASS / PASS | PASS |
| [namespace-100000](../view/performance/init_namespace/namespace-100000/perf.jsonl) | 100000 / 100000 / 500000000 | layerstack_init_ns | 3803.760 | — / — / — | 90.22 / 5.36 | 2603.162 | PASS / PASS | PASS |

## mixed_load_bearing

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [agent-episodes-1-compact-v2](../view/performance/mixed_load_bearing/agent-episodes-1-compact-v2/perf.jsonl) | 1 / 54 / 1081344 | pure_call_sum_ns | 33.485 | 15.161 / — / 7.017 | 10.25 / 5.41 | 26.241 | PASS / PASS | PASS |
| [agent-episodes-10-compact-v2](../view/performance/mixed_load_bearing/agent-episodes-10-compact-v2/perf.jsonl) | 10 / 540 / 10813440 | pure_call_sum_ns | 108.465 | 83.540 / — / 12.343 | 13.05 / 5.64 | 89.911 | PASS / PASS | PASS |
| [agent-episodes-100](../view/performance/mixed_load_bearing/agent-episodes-100/perf.jsonl) | 100 / 14800 / 83492864 | pure_call_sum_ns | 1169.385 | 1027.723 / — / 124.335 | 41.52 / 12.09 | 908.411 | PASS / PASS | PASS |
| [agent-episodes-500](../view/performance/mixed_load_bearing/agent-episodes-500/perf.jsonl) | 500 / 14800 / 83492864 | pure_call_sum_ns | 8523.980 | 8169.061 / — / 336.114 | 60.88 / 41.46 | 7535.401 | PASS / PASS | PASS |

## namespace_mutation

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [namespace-subtree-relocate-delete-1-compact-v2](../view/performance/namespace_mutation/namespace-subtree-relocate-delete-1-compact-v2/perf.jsonl) | 1 / 250 / 551200 | pure_call_sum_ns | 27.012 | 9.754 / — / 4.733 | 9.16 / 5.42 | 21.311 | PASS / PASS | PASS |
| [namespace-subtree-relocate-delete-10-compact-v2](../view/performance/namespace_mutation/namespace-subtree-relocate-delete-10-compact-v2/perf.jsonl) | 10 / 700 / 1012000 | pure_call_sum_ns | 75.996 | 56.514 / — / 6.515 | 10.86 / 5.43 | 59.145 | PASS / PASS | PASS |
| [namespace-subtree-relocate-delete-100-mixed-v4](../view/performance/namespace_mutation/namespace-subtree-relocate-delete-100-mixed-v4/perf.jsonl) | 100 / 2400 / 105267200 | pure_call_sum_ns | 66.062 | 44.257 / — / 10.606 | 14.14 / 5.90 | 51.018 | PASS / PASS | PASS |
| [namespace-subtree-relocate-delete-500-mixed-v4](../view/performance/namespace_mutation/namespace-subtree-relocate-delete-500-mixed-v4/perf.jsonl) | 500 / 7000 / 526336000 | pure_call_sum_ns | 245.514 | 211.860 / — / 19.891 | 28.11 / 5.43 | 182.894 | PASS / PASS | PASS |

## payload_create_read

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [payload-create-1m-compact-v2](../view/performance/payload_create_read/payload-create-1m-compact-v2/perf.jsonl) | 1 / 0 / 0 | pure_call_sum_ns | 32.794 | 9.791 / — / 10.851 | 12.84 / 8.14 | 28.837 | PASS / PASS | PASS |
| [payload-create-10m-compact-v2](../view/performance/payload_create_read/payload-create-10m-compact-v2/perf.jsonl) | 10 / 0 / 0 | pure_call_sum_ns | 84.416 | 33.570 / — / 39.713 | 28.20 / 9.65 | 91.556 | PASS / PASS | PASS |
| [payload-create-100m](../view/performance/payload_create_read/payload-create-100m/perf.jsonl) | 100 / 0 / 0 | pure_call_sum_ns | 662.003 | 282.778 / — / 361.895 | 54.61 / 12.02 | 682.771 | PASS / PASS | PASS |
| [payload-create-500m](../view/performance/payload_create_read/payload-create-500m/perf.jsonl) | 500 / 0 / 0 | pure_call_sum_ns | 2701.168 | 1215.713 / — / 1468.741 | 55.53 / 12.53 | 3068.250 | PASS / PASS | PASS |
| [payload-random-read-1-compact-v2](../view/performance/payload_create_read/payload-random-read-1-compact-v2/perf.jsonl) | 1 / 1 / 1048576 | pure_call_sum_ns | 17.124 | 3.689 / — / 1.873 | 7.70 / 5.64 | 14.585 | PASS / PASS | PASS |
| [payload-random-read-10-compact-v2](../view/performance/payload_create_read/payload-random-read-10-compact-v2/perf.jsonl) | 10 / 1 / 10485760 | pure_call_sum_ns | 22.836 | 8.952 / — / 1.713 | 9.72 / 5.38 | 17.420 | PASS / PASS | PASS |
| [payload-random-read-100](../view/performance/payload_create_read/payload-random-read-100/perf.jsonl) | 100 / 1 / 524288000 | pure_call_sum_ns | 64.856 | 52.621 / — / 1.799 | 29.48 / 5.40 | 56.396 | PASS / PASS | PASS |
| [payload-random-read-500](../view/performance/payload_create_read/payload-random-read-500/perf.jsonl) | 500 / 1 / 524288000 | pure_call_sum_ns | 259.410 | 245.250 / — / 2.357 | 43.33 / 11.48 | 219.687 | PASS / PASS | PASS |

## store_footprint

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [store-footprint-unique-100000](../view/performance/store_footprint/store-footprint-unique-100000/perf.jsonl) | 100000 / 100000 / 500000000 | product_call_sum_ns | 4082.417 | — / — / — | 94.00 / 5.40 | 2982.459 | PASS / PASS | PASS |
| [store-footprint-metadata-cardinality-100000](../view/performance/store_footprint/store-footprint-metadata-cardinality-100000/perf.jsonl) | 100000 / 100000 / 500000000 | product_call_sum_ns | 6897.714 | — / — / — | 104.50 / 5.91 | 4570.430 | PASS / PASS | PASS |
| [store-footprint-large-object-500m](../view/performance/store_footprint/store-footprint-large-object-500m/perf.jsonl) | 100000 / 100 / 500000000 | product_call_sum_ns | 1505.693 | — / — / — | 62.05 / 7.34 | 491.906 | PASS / PASS | PASS |
| [store-footprint-unique-100-low-v1](../view/performance/store_footprint/store-footprint-unique-100-low-v1/perf.jsonl) | 100 / 100 / 5000000 | product_call_sum_ns | 47.905 | — / — / — | 28.23 / 5.43 | 31.948 | PASS / PASS | PASS |
| [store-footprint-metadata-cardinality-100-low-v1](../view/performance/store_footprint/store-footprint-metadata-cardinality-100-low-v1/perf.jsonl) | 100 / 100 / 5000000 | product_call_sum_ns | 56.222 | — / — / — | 28.36 / 5.64 | 35.870 | PASS / PASS | PASS |
| [store-footprint-large-object-10m-low-v1](../view/performance/store_footprint/store-footprint-large-object-10m-low-v1/perf.jsonl) | 10 / 10 / 10000000 | product_call_sum_ns | 71.377 | — / — / — | 33.89 / 5.15 | 43.894 | PASS / PASS | PASS |

## tiny_file_churn

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [tiny-create-1-compact-v2](../view/performance/tiny_file_churn/tiny-create-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 23.598 | 6.894 / — / 4.726 | 9.05 / 5.14 | 19.054 | PASS / PASS | PASS |
| [tiny-create-10-compact-v2](../view/performance/tiny_file_churn/tiny-create-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 33.639 | 12.777 / — / 8.764 | 10.92 / 5.66 | 26.712 | PASS / PASS | PASS |
| [tiny-create-100-mixed-v4](../view/performance/tiny_file_churn/tiny-create-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 63.618 | 37.747 / — / 15.283 | 15.14 / 5.15 | 53.616 | PASS / PASS | PASS |
| [tiny-create-500-mixed-v4](../view/performance/tiny_file_churn/tiny-create-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 231.759 | 159.981 / — / 58.224 | 30.33 / 7.23 | 201.740 | PASS / PASS | PASS |
| [tiny-stat-1-compact-v2](../view/performance/tiny_file_churn/tiny-stat-1-compact-v2/perf.jsonl) | 1 / 51 / 1048576 | pure_call_sum_ns | 19.920 | 5.999 / — / 1.629 | 7.81 / 7.41 | 15.834 | PASS / PASS | PASS |
| [tiny-stat-10-compact-v2](../view/performance/tiny_file_churn/tiny-stat-10-compact-v2/perf.jsonl) | 10 / 510 / 10502249 | pure_call_sum_ns | 25.328 | 10.552 / — / 1.788 | 8.61 / 5.41 | 21.194 | PASS / PASS | PASS |
| [tiny-stat-100-mixed-v4](../view/performance/tiny_file_churn/tiny-stat-100-mixed-v4/perf.jsonl) | 100 / 2500 / 105682050 | pure_call_sum_ns | 60.027 | 45.987 / — / 2.324 | 15.75 / 5.43 | 37.810 | PASS / PASS | PASS |
| [tiny-stat-500-mixed-v4](../view/performance/tiny_file_churn/tiny-stat-500-mixed-v4/perf.jsonl) | 500 / 5500 / 525112450 | pure_call_sum_ns | 94.357 | 77.926 / — / 2.308 | 25.30 / 5.74 | 55.349 | PASS / PASS | PASS |
| [tiny-unlink-1-compact-v2](../view/performance/tiny_file_churn/tiny-unlink-1-compact-v2/perf.jsonl) | 1 / 51 / 1048576 | pure_call_sum_ns | 32.923 | 13.249 / — / 4.450 | 8.83 / 5.43 | 17.635 | PASS / PASS | PASS |
| [tiny-unlink-10-compact-v2](../view/performance/tiny_file_churn/tiny-unlink-10-compact-v2/perf.jsonl) | 10 / 510 / 10502249 | pure_call_sum_ns | 36.254 | 16.318 / — / 7.142 | 9.83 / 5.68 | 25.205 | PASS / PASS | PASS |
| [tiny-unlink-100-mixed-v4](../view/performance/tiny_file_churn/tiny-unlink-100-mixed-v4/perf.jsonl) | 100 / 2500 / 105682050 | pure_call_sum_ns | 85.590 | 60.323 / — / 12.634 | 16.95 / 5.43 | 50.918 | PASS / PASS | PASS |
| [tiny-unlink-500-mixed-v4](../view/performance/tiny_file_churn/tiny-unlink-500-mixed-v4/perf.jsonl) | 500 / 5500 / 525112450 | pure_call_sum_ns | 139.473 | 109.597 / — / 18.311 | 27.59 / 5.41 | 114.279 | PASS / PASS | PASS |
| [tiny-bulk-create-1-compact-v2](../view/performance/tiny_file_churn/tiny-bulk-create-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 107.711 | 77.417 / — / 16.224 | 14.16 / 6.72 | 96.728 | PASS / PASS | PASS |
| [tiny-bulk-create-10-compact-v2](../view/performance/tiny_file_churn/tiny-bulk-create-10-compact-v2/perf.jsonl) | 10 / 50 / 1048576 | pure_call_sum_ns | 354.671 | 260.384 / — / 79.685 | 34.48 / 9.00 | 295.898 | PASS / PASS | PASS |
| [tiny-bulk-create-100-mixed-v3](../view/performance/tiny_file_churn/tiny-bulk-create-100-mixed-v3/perf.jsonl) | 100 / 200 / 1048576 | pure_call_sum_ns | 1139.658 | 694.170 / — / 427.400 | 63.58 / 13.89 | 989.888 | PASS / PASS | PASS |
| [tiny-bulk-create-500-mixed-v3](../view/performance/tiny_file_churn/tiny-bulk-create-500-mixed-v3/perf.jsonl) | 500 / 200 / 1048576 | pure_call_sum_ns | 5319.530 | 3297.686 / — / 1986.453 | 75.19 / 24.52 | 5054.054 | PASS / PASS | PASS |
| [tiny-bulk-delete-1-compact-v2](../view/performance/tiny_file_churn/tiny-bulk-delete-1-compact-v2/perf.jsonl) | 1 / 100 / 2097152 | pure_call_sum_ns | 111.209 | 92.030 / — / 5.735 | 9.44 / 5.41 | 93.347 | PASS / PASS | PASS |
| [tiny-bulk-delete-10-compact-v2](../view/performance/tiny_file_churn/tiny-bulk-delete-10-compact-v2/perf.jsonl) | 10 / 550 / 11534336 | pure_call_sum_ns | 188.448 | 171.030 / — / 5.787 | 12.12 / 7.65 | 168.255 | PASS / PASS | PASS |
| [tiny-bulk-delete-100-mixed-v3](../view/performance/tiny_file_churn/tiny-bulk-delete-100-mixed-v3/perf.jsonl) | 100 / 1200 / 105906176 | pure_call_sum_ns | 320.918 | 299.496 / — / 7.284 | 20.62 / 8.75 | 259.279 | PASS / PASS | PASS |
| [tiny-bulk-delete-500-mixed-v3](../view/performance/tiny_file_churn/tiny-bulk-delete-500-mixed-v3/perf.jsonl) | 500 / 5200 / 525336576 | pure_call_sum_ns | 1143.209 | 1112.460 / — / 16.928 | 50.59 / 25.18 | 933.000 | PASS / PASS | PASS |

## workspace_change_locality

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|
| [workspace-clean-commit-1-compact-v2](../view/performance/workspace_change_locality/workspace-clean-commit-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 12.734 | — / — / 1.591 | 7.42 / 5.67 | 11.316 | PASS / PASS | PASS |
| [workspace-clean-commit-10-compact-v2](../view/performance/workspace_change_locality/workspace-clean-commit-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 12.446 | — / — / 1.863 | 7.77 / 5.39 | 10.873 | PASS / PASS | PASS |
| [workspace-clean-commit-100-mixed-v4](../view/performance/workspace_change_locality/workspace-clean-commit-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 14.589 | — / — / 1.802 | 10.42 / 5.11 | 11.559 | PASS / PASS | PASS |
| [workspace-clean-commit-500-mixed-v4](../view/performance/workspace_change_locality/workspace-clean-commit-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 18.544 | — / — / 1.972 | 19.42 / 5.14 | 11.866 | PASS / PASS | PASS |
| [workspace-fixed-move-1-compact-v2](../view/performance/workspace_change_locality/workspace-fixed-move-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 24.738 | 7.064 / — / 4.820 | 9.11 / 5.17 | 19.708 | PASS / PASS | PASS |
| [workspace-fixed-move-10-compact-v2](../view/performance/workspace_change_locality/workspace-fixed-move-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 23.146 | 7.261 / — / 4.577 | 10.00 / 5.33 | 21.320 | PASS / PASS | PASS |
| [workspace-fixed-move-100-mixed-v4](../view/performance/workspace_change_locality/workspace-fixed-move-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 35.468 | 18.348 / — / 5.289 | 15.03 / 5.18 | 28.352 | PASS / PASS | PASS |
| [workspace-fixed-move-500-mixed-v4](../view/performance/workspace_change_locality/workspace-fixed-move-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 36.072 | 18.539 / — / 5.993 | 24.89 / 5.40 | 27.589 | PASS / PASS | PASS |
| [workspace-distributed-sdk-edit-1-compact-v2](../view/performance/workspace_change_locality/workspace-distributed-sdk-edit-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 25.371 | — / 4.759 / 6.957 | 9.83 / 5.40 | 16.937 | PASS / PASS | PASS |
| [workspace-distributed-sdk-edit-10-compact-v2](../view/performance/workspace_change_locality/workspace-distributed-sdk-edit-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 54.443 | — / 32.150 / 9.430 | 13.28 / 5.42 | 39.180 | PASS / PASS | PASS |
| [workspace-distributed-sdk-edit-100-mixed-v4](../view/performance/workspace_change_locality/workspace-distributed-sdk-edit-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 430.541 | — / 386.897 / 32.121 | 31.67 / 12.99 | 336.399 | PASS / PASS | PASS |
| [workspace-distributed-sdk-edit-500-mixed-v4](../view/performance/workspace_change_locality/workspace-distributed-sdk-edit-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 3473.778 | — / 3349.117 / 111.816 | 64.09 / 30.13 | 2849.182 | PASS / PASS | PASS |
| [workspace-dense-rewrite-1-compact-v2](../view/performance/workspace_change_locality/workspace-dense-rewrite-1-compact-v2/perf.jsonl) | 1 / 50 / 1048576 | pure_call_sum_ns | 144.801 | 100.924 / — / 29.070 | 17.31 / 8.27 | 84.118 | PASS / PASS | PASS |
| [workspace-dense-rewrite-10-compact-v2](../view/performance/workspace_change_locality/workspace-dense-rewrite-10-compact-v2/perf.jsonl) | 10 / 500 / 10485760 | pure_call_sum_ns | 449.219 | 275.048 / — / 159.861 | 43.36 / 21.15 | 316.616 | PASS / PASS | PASS |
| [workspace-dense-rewrite-100-mixed-v4](../view/performance/workspace_change_locality/workspace-dense-rewrite-100-mixed-v4/perf.jsonl) | 100 / 2000 / 104857600 | pure_call_sum_ns | 2260.318 | 1152.372 / — / 1083.433 | 69.36 / 124.41 | 1497.618 | PASS / PASS | PASS |
| [workspace-dense-rewrite-500-mixed-v4](../view/performance/workspace_change_locality/workspace-dense-rewrite-500-mixed-v4/perf.jsonl) | 500 / 5000 / 524288000 | pure_call_sum_ns | 7665.527 | 3820.293 / — / 3804.454 | 80.02 / 546.10 | 5688.592 | PASS / PASS | PASS |

## workspace_reliability

| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |
|---|---|---|---:|---|---|---:|---|---|

Historical subsecond bulk targets are separate from the family threshold:
- tiny-bulk-create-100-mixed-v3: TARGET_MISS against strict <1,000 ms.
- tiny-bulk-delete-100-mixed-v3: PASS against strict <1,000 ms.

## Git stages

| Test | Apply | First status | Diff | Add | Cached check | Git commit | Final status | LayerFS Commit |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| git-tool-1-compact-v2 | 5.413 | 150.107 | 112.181 | 40.770 | 5.845 | 15.392 | 33.078 | 9.292 |
| git-tool-10-compact-v2 | 11.802 | 241.801 | 356.191 | 51.360 | 6.844 | 20.927 | 32.011 | 19.630 |
| git-tool-100-mixed-v4 | 129.904 | 629.383 | 1313.946 | 107.007 | 17.065 | 46.505 | 39.508 | 54.785 |
| git-tool-500-mixed-v4 | 610.194 | 1475.983 | 2756.498 | 424.031 | 70.025 | 133.792 | 59.635 | 166.783 |

Historical native Git reference (three-run median):
- git-tool-100-mixed-v4: 249.184 ms, Apply + six Git commands. [Published source](../issue68-evidence/report.json).
- git-tool-500-mixed-v4: 634.505 ms, Apply + six Git commands. [Published source](../issue68-evidence/report.json).

Native references exclude LayerFS Create/Commit/visibility/End and are not matched total-lifecycle comparisons.

All Git stage values are milliseconds. Git commit and LayerFS Commit are separate operations.

## Verification

| Test | Result | Wall (s) | Coverage |
|---|---|---:|---|
| [payload-create-1m-compact-v2](../view/verification/payload_create_read/payload-create-1m-compact-v2/verification.json) | PASS | 1.9868718750076368 | changed data and selected unchanged witnesses; qualified roots |
| [payload-create-10m-compact-v2](../view/verification/payload_create_read/payload-create-10m-compact-v2/verification.json) | PASS | 2.2637014580104733 | changed data and selected unchanged witnesses; qualified roots |
| [payload-create-100m](../view/verification/payload_create_read/payload-create-100m/verification.json) | PASS | 3.9128714580001542 | changed data and selected unchanged witnesses; qualified roots |
| [payload-create-500m](../view/verification/payload_create_read/payload-create-500m/verification.json) | PASS | 11.078752832996543 | changed data and selected unchanged witnesses; qualified roots |
| [payload-random-read-1-compact-v2](../view/verification/payload_create_read/payload-random-read-1-compact-v2/verification.json) | PASS | 2.1241928749950603 | changed data and selected unchanged witnesses; qualified roots |
| [payload-random-read-10-compact-v2](../view/verification/payload_create_read/payload-random-read-10-compact-v2/verification.json) | PASS | 2.488251750008203 | changed data and selected unchanged witnesses; qualified roots |
| [payload-random-read-100](../view/verification/payload_create_read/payload-random-read-100/verification.json) | PASS | 16.31882716700784 | changed data and selected unchanged witnesses; qualified roots |
| [payload-random-read-500](../view/verification/payload_create_read/payload-random-read-500/verification.json) | PASS | 9.802170583992847 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-exact-1-compact-v2](../view/verification/dedup_workspace_reuse/dedup-workspace-exact-1-compact-v2/verification.json) | PASS | 2.863152333011385 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-exact-10-compact-v2](../view/verification/dedup_workspace_reuse/dedup-workspace-exact-10-compact-v2/verification.json) | PASS | 2.3816247079957975 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-exact-100](../view/verification/dedup_workspace_reuse/dedup-workspace-exact-100/verification.json) | PASS | 8.367592041991884 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-exact-500](../view/verification/dedup_workspace_reuse/dedup-workspace-exact-500/verification.json) | PASS | 16.019578208986786 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-local-1-compact-v2](../view/verification/dedup_workspace_reuse/dedup-workspace-local-1-compact-v2/verification.json) | PASS | 1.837970624997979 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-local-10-compact-v2](../view/verification/dedup_workspace_reuse/dedup-workspace-local-10-compact-v2/verification.json) | PASS | 2.26244420799776 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-local-100](../view/verification/dedup_workspace_reuse/dedup-workspace-local-100/verification.json) | PASS | 6.718023249995895 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-local-500](../view/verification/dedup_workspace_reuse/dedup-workspace-local-500/verification.json) | PASS | 16.468694416005746 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-1-compact-v2](../view/verification/dedup_workspace_reuse/dedup-workspace-unique-1-compact-v2/verification.json) | PASS | 1.810568000000785 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-10-compact-v2](../view/verification/dedup_workspace_reuse/dedup-workspace-unique-10-compact-v2/verification.json) | PASS | 2.2860012499877485 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-100](../view/verification/dedup_workspace_reuse/dedup-workspace-unique-100/verification.json) | PASS | 6.692703166001593 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-500](../view/verification/dedup_workspace_reuse/dedup-workspace-unique-500/verification.json) | PASS | 17.145474666001974 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-1-base128-v3](../view/verification/dedup_workspace_reuse/dedup-workspace-unique-1-base128-v3/verification.json) | PASS | 4.6470338749932125 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-workspace-unique-10-base128-v3](../view/verification/dedup_workspace_reuse/dedup-workspace-unique-10-base128-v3/verification.json) | PASS | 4.78530824999325 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-anchor-1](../view/verification/dedup_cross_file/dedup-cross-file-anchor-1/verification.json) | PASS | 1.8088304170087213 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-unique-10](../view/verification/dedup_cross_file/dedup-cross-file-unique-10/verification.json) | PASS | 2.0889084999944316 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-unique-100](../view/verification/dedup_cross_file/dedup-cross-file-unique-100/verification.json) | PASS | 4.967885416990612 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-unique-500](../view/verification/dedup_cross_file/dedup-cross-file-unique-500/verification.json) | PASS | 16.35377650000737 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-identical-10](../view/verification/dedup_cross_file/dedup-cross-file-identical-10/verification.json) | PASS | 2.0977917080017505 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-identical-100](../view/verification/dedup_cross_file/dedup-cross-file-identical-100/verification.json) | PASS | 3.78707145799126 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-identical-500](../view/verification/dedup_cross_file/dedup-cross-file-identical-500/verification.json) | PASS | 11.414002584002446 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-mixed-10](../view/verification/dedup_cross_file/dedup-cross-file-mixed-10/verification.json) | PASS | 2.191850917006377 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-mixed-100](../view/verification/dedup_cross_file/dedup-cross-file-mixed-100/verification.json) | PASS | 4.619159499998204 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cross-file-mixed-500](../view/verification/dedup_cross_file/dedup-cross-file-mixed-500/verification.json) | PASS | 15.386793124998803 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-overwrite-1](../view/verification/dedup_cdc_locality/dedup-cdc-overwrite-1/verification.json) | PASS | 1.8937429999932647 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-overwrite-10](../view/verification/dedup_cdc_locality/dedup-cdc-overwrite-10/verification.json) | PASS | 2.3053038330108393 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-overwrite-100](../view/verification/dedup_cdc_locality/dedup-cdc-overwrite-100/verification.json) | PASS | 5.526330166001571 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-overwrite-500](../view/verification/dedup_cdc_locality/dedup-cdc-overwrite-500/verification.json) | PASS | 19.509087458005524 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-insert-1](../view/verification/dedup_cdc_locality/dedup-cdc-insert-1/verification.json) | PASS | 2.075602833996527 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-insert-10](../view/verification/dedup_cdc_locality/dedup-cdc-insert-10/verification.json) | PASS | 2.1921925000060583 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-insert-100](../view/verification/dedup_cdc_locality/dedup-cdc-insert-100/verification.json) | PASS | 5.522222375002457 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-insert-500](../view/verification/dedup_cdc_locality/dedup-cdc-insert-500/verification.json) | PASS | 19.590034375010873 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-delete-1](../view/verification/dedup_cdc_locality/dedup-cdc-delete-1/verification.json) | PASS | 1.823948291988927 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-delete-10](../view/verification/dedup_cdc_locality/dedup-cdc-delete-10/verification.json) | PASS | 2.2334594999992987 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-delete-100](../view/verification/dedup_cdc_locality/dedup-cdc-delete-100/verification.json) | PASS | 5.371818624989828 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-delete-500](../view/verification/dedup_cdc_locality/dedup-cdc-delete-500/verification.json) | PASS | 20.81625316699501 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-common-body-1](../view/verification/dedup_cdc_locality/dedup-cdc-common-body-1/verification.json) | PASS | 1.9369902499893215 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-common-body-10](../view/verification/dedup_cdc_locality/dedup-cdc-common-body-10/verification.json) | PASS | 2.2215032919921214 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-common-body-100](../view/verification/dedup_cdc_locality/dedup-cdc-common-body-100/verification.json) | PASS | 5.499138500003028 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-common-body-500](../view/verification/dedup_cdc_locality/dedup-cdc-common-body-500/verification.json) | PASS | 19.990675708992057 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-scattered-1](../view/verification/dedup_cdc_locality/dedup-cdc-scattered-1/verification.json) | PASS | 1.9648935000004712 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-scattered-10](../view/verification/dedup_cdc_locality/dedup-cdc-scattered-10/verification.json) | PASS | 2.218973625000217 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-scattered-100](../view/verification/dedup_cdc_locality/dedup-cdc-scattered-100/verification.json) | PASS | 6.17798487500113 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-scattered-500](../view/verification/dedup_cdc_locality/dedup-cdc-scattered-500/verification.json) | PASS | 22.8749262909987 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-cdc-boundaries-proof](../view/verification/dedup_cdc_locality/dedup-cdc-boundaries-proof/verification.json) | PASS | 1.7793910000036703 | registered verification; detailed checks in receipt |
| [overwrite-head-4k-on-1mib-ops-1](../view/verification/edit_length_preserving/overwrite-head-4k-on-1mib-ops-1/verification.json) | PASS | 1.8475475419982104 | bounded bytes; payload.bin [0, 69632) |
| [overwrite-head-4k-on-10mib-ops-1](../view/verification/edit_length_preserving/overwrite-head-4k-on-10mib-ops-1/verification.json) | PASS | 1.980406167000183 | bounded bytes; payload.bin [0, 69632) |
| [overwrite-head-4k-on-100mib-ops-1](../view/verification/edit_length_preserving/overwrite-head-4k-on-100mib-ops-1/verification.json) | PASS | 3.8586084579874296 | bounded bytes; payload.bin [0, 69632) |
| [overwrite-head-4k-on-500mib-ops-1](../view/verification/edit_length_preserving/overwrite-head-4k-on-500mib-ops-1/verification.json) | PASS | 10.954548583002179 | bounded bytes; payload.bin [0, 69632) |
| [overwrite-middle-4k-on-1mib-ops-1](../view/verification/edit_length_preserving/overwrite-middle-4k-on-1mib-ops-1/verification.json) | PASS | 1.7929103339993162 | bounded bytes; payload.bin [456704, 591872) |
| [overwrite-middle-4k-on-10mib-ops-1](../view/verification/edit_length_preserving/overwrite-middle-4k-on-10mib-ops-1/verification.json) | PASS | 1.8630845000006957 | bounded bytes; payload.bin [5175296, 5310464) |
| [overwrite-middle-4k-on-100mib-ops-1](../view/verification/edit_length_preserving/overwrite-middle-4k-on-100mib-ops-1/verification.json) | PASS | 2.149253249997855 | bounded bytes; payload.bin [52361216, 52496384) |
| [overwrite-middle-4k-on-500mib-ops-1](../view/verification/edit_length_preserving/overwrite-middle-4k-on-500mib-ops-1/verification.json) | PASS | 3.8273802079929737 | bounded bytes; payload.bin [262076416, 262211584) |
| [overwrite-tail-4k-on-1mib-ops-1](../view/verification/edit_length_preserving/overwrite-tail-4k-on-1mib-ops-1/verification.json) | PASS | 1.6417941660038196 | bounded bytes; payload.bin [978944, 1048576) |
| [overwrite-tail-4k-on-10mib-ops-1](../view/verification/edit_length_preserving/overwrite-tail-4k-on-10mib-ops-1/verification.json) | PASS | 1.7036557500105118 | bounded bytes; payload.bin [10416128, 10485760) |
| [overwrite-tail-4k-on-100mib-ops-1](../view/verification/edit_length_preserving/overwrite-tail-4k-on-100mib-ops-1/verification.json) | PASS | 2.2169682920066407 | bounded bytes; payload.bin [104787968, 104857600) |
| [overwrite-tail-4k-on-500mib-ops-1](../view/verification/edit_length_preserving/overwrite-tail-4k-on-500mib-ops-1/verification.json) | PASS | 3.964256541003124 | bounded bytes; payload.bin [524218368, 524288000) |
| [insert-middle-4k-on-1mib-ops-1](../view/verification/edit_length_changing/insert-middle-4k-on-1mib-ops-1/verification.json) | PASS | 1.7774745839997195 | bounded bytes; payload.bin [458752, 593920) |
| [insert-middle-4k-on-10mib-ops-1](../view/verification/edit_length_changing/insert-middle-4k-on-10mib-ops-1/verification.json) | PASS | 1.888940459000878 | bounded bytes; payload.bin [5177344, 5312512) |
| [insert-middle-4k-on-100mib-ops-1](../view/verification/edit_length_changing/insert-middle-4k-on-100mib-ops-1/verification.json) | PASS | 2.236307749990374 | bounded bytes; payload.bin [52363264, 52498432) |
| [insert-middle-4k-on-500mib-result-capped-v2-ops-1](../view/verification/edit_length_changing/insert-middle-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 10.592876084003365 | bounded bytes; payload.bin [262076416, 262211584) |
| [delete-middle-4k-on-1mib-ops-1](../view/verification/edit_length_changing/delete-middle-4k-on-1mib-ops-1/verification.json) | PASS | 1.7915914580080425 | bounded bytes; payload.bin [456704, 587776) |
| [delete-middle-4k-on-10mib-ops-1](../view/verification/edit_length_changing/delete-middle-4k-on-10mib-ops-1/verification.json) | PASS | 1.803129916996113 | bounded bytes; payload.bin [5175296, 5306368) |
| [delete-middle-4k-on-100mib-ops-1](../view/verification/edit_length_changing/delete-middle-4k-on-100mib-ops-1/verification.json) | PASS | 2.291404165996937 | bounded bytes; payload.bin [52361216, 52492288) |
| [delete-middle-4k-on-500mib-ops-1](../view/verification/edit_length_changing/delete-middle-4k-on-500mib-ops-1/verification.json) | PASS | 4.114484833000461 | bounded bytes; payload.bin [262076416, 262207488) |
| [append-tail-4k-on-1mib-ops-1](../view/verification/edit_length_changing/append-tail-4k-on-1mib-ops-1/verification.json) | PASS | 1.837592541007325 | bounded bytes; payload.bin [983040, 1052672) |
| [append-tail-4k-on-10mib-ops-1](../view/verification/edit_length_changing/append-tail-4k-on-10mib-ops-1/verification.json) | PASS | 1.6037975420040311 | bounded bytes; payload.bin [10420224, 10489856) |
| [append-tail-4k-on-100mib-ops-1](../view/verification/edit_length_changing/append-tail-4k-on-100mib-ops-1/verification.json) | PASS | 2.1850692499865545 | bounded bytes; payload.bin [104792064, 104861696) |
| [append-tail-4k-on-500mib-result-capped-v2-ops-1](../view/verification/edit_length_changing/append-tail-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 4.14192241699493 | bounded bytes; payload.bin [524218368, 524288000) |
| [prepend-head-4k-on-1mib-ops-1](../view/verification/edit_length_changing/prepend-head-4k-on-1mib-ops-1/verification.json) | PASS | 1.7200156250037253 | bounded bytes; payload.bin [0, 69632) |
| [prepend-head-4k-on-10mib-ops-1](../view/verification/edit_length_changing/prepend-head-4k-on-10mib-ops-1/verification.json) | PASS | 1.779393542004982 | bounded bytes; payload.bin [0, 69632) |
| [prepend-head-4k-on-100mib-ops-1](../view/verification/edit_length_changing/prepend-head-4k-on-100mib-ops-1/verification.json) | PASS | 2.28030545799993 | bounded bytes; payload.bin [0, 69632) |
| [prepend-head-4k-on-500mib-result-capped-v2-ops-1](../view/verification/edit_length_changing/prepend-head-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 4.245487124993815 | bounded bytes; payload.bin [0, 69632) |
| [replace-grow-middle-2k-to-4k-on-1mib-ops-1](../view/verification/edit_length_changing/replace-grow-middle-2k-to-4k-on-1mib-ops-1/verification.json) | PASS | 1.7398471250053262 | bounded bytes; payload.bin [457728, 592896) |
| [replace-grow-middle-2k-to-4k-on-10mib-ops-1](../view/verification/edit_length_changing/replace-grow-middle-2k-to-4k-on-10mib-ops-1/verification.json) | PASS | 1.7945018749887822 | bounded bytes; payload.bin [5176320, 5311488) |
| [replace-grow-middle-2k-to-4k-on-100mib-ops-1](../view/verification/edit_length_changing/replace-grow-middle-2k-to-4k-on-100mib-ops-1/verification.json) | PASS | 2.2856844580092 | bounded bytes; payload.bin [52362240, 52497408) |
| [replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1](../view/verification/edit_length_changing/replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 9.973089458988397 | bounded bytes; payload.bin [262076416, 262211584) |
| [replace-shrink-middle-4k-to-2k-on-1mib-ops-1](../view/verification/edit_length_changing/replace-shrink-middle-4k-to-2k-on-1mib-ops-1/verification.json) | PASS | 1.8320273749995977 | bounded bytes; payload.bin [456704, 589824) |
| [replace-shrink-middle-4k-to-2k-on-10mib-ops-1](../view/verification/edit_length_changing/replace-shrink-middle-4k-to-2k-on-10mib-ops-1/verification.json) | PASS | 1.5861151660064934 | bounded bytes; payload.bin [5175296, 5308416) |
| [replace-shrink-middle-4k-to-2k-on-100mib-ops-1](../view/verification/edit_length_changing/replace-shrink-middle-4k-to-2k-on-100mib-ops-1/verification.json) | PASS | 2.1557470839907182 | bounded bytes; payload.bin [52361216, 52494336) |
| [replace-shrink-middle-4k-to-2k-on-500mib-ops-1](../view/verification/edit_length_changing/replace-shrink-middle-4k-to-2k-on-500mib-ops-1/verification.json) | PASS | 4.055619624996325 | bounded bytes; payload.bin [262076416, 262209536) |
| [truncate-tail-4k-on-1mib-ops-1](../view/verification/edit_length_changing/truncate-tail-4k-on-1mib-ops-1/verification.json) | PASS | 1.6827613750065211 | bounded bytes; payload.bin [978944, 1044480) |
| [truncate-tail-4k-on-10mib-ops-1](../view/verification/edit_length_changing/truncate-tail-4k-on-10mib-ops-1/verification.json) | PASS | 1.6619550829927903 | bounded bytes; payload.bin [10416128, 10481664) |
| [truncate-tail-4k-on-100mib-ops-1](../view/verification/edit_length_changing/truncate-tail-4k-on-100mib-ops-1/verification.json) | PASS | 2.1448318750044564 | bounded bytes; payload.bin [104787968, 104853504) |
| [truncate-tail-4k-on-500mib-ops-1](../view/verification/edit_length_changing/truncate-tail-4k-on-500mib-ops-1/verification.json) | PASS | 3.731173708991264 | bounded bytes; payload.bin [524218368, 524283904) |
| [zero-extend-tail-4k-on-1mib-ops-1](../view/verification/edit_length_changing/zero-extend-tail-4k-on-1mib-ops-1/verification.json) | PASS | 1.5976098749961238 | bounded bytes; payload.bin [983040, 1052672) |
| [zero-extend-tail-4k-on-10mib-ops-1](../view/verification/edit_length_changing/zero-extend-tail-4k-on-10mib-ops-1/verification.json) | PASS | 1.9053765830030898 | bounded bytes; payload.bin [10420224, 10489856) |
| [zero-extend-tail-4k-on-100mib-ops-1](../view/verification/edit_length_changing/zero-extend-tail-4k-on-100mib-ops-1/verification.json) | PASS | 2.1319220410077833 | bounded bytes; payload.bin [104792064, 104861696) |
| [zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1](../view/verification/edit_length_changing/zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1/verification.json) | PASS | 4.079063582990784 | bounded bytes; payload.bin [524218368, 524288000) |
| [overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1/verification.json) | PASS | 1.8478067090036348 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1/verification.json) | PASS | 1.6475708330108318 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1/verification.json) | PASS | 2.2539729589916533 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1/verification.json) | PASS | 4.092843291000463 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1/verification.json) | PASS | 1.9106646250002086 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1/verification.json) | PASS | 1.8543050000007497 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1/verification.json) | PASS | 2.311351249998552 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1/verification.json) | PASS | 3.9996202500042273 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1/verification.json) | PASS | 1.7268055000022287 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1/verification.json) | PASS | 1.7514523749996442 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1/verification.json) | PASS | 2.112886624992825 | bounded bytes; payload.bin [81920, 278528) |
| [overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1](../view/verification/edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1/verification.json) | PASS | 4.0621043749997625 | bounded bytes; payload.bin [81920, 278528) |
| [namespace-100-compact-v3](../view/verification/init_namespace/namespace-100-compact-v3/verification.json) | PASS | 1.92011520799133 | registered verification; detailed checks in receipt |
| [namespace-1000-compact-v3](../view/verification/init_namespace/namespace-1000-compact-v3/verification.json) | PASS | 2.196545334008988 | registered verification; detailed checks in receipt |
| [namespace-10000](../view/verification/init_namespace/namespace-10000/verification.json) | PASS | 6.87217404099647 | registered verification; detailed checks in receipt |
| [namespace-100000](../view/verification/init_namespace/namespace-100000/verification.json) | PASS | 7.098156834006659 | registered verification; detailed checks in receipt |
| [store-footprint-unique-100000](../view/verification/store_footprint/store-footprint-unique-100000/verification.json) | PASS | 7.047733291008626 | bounded bytes; d0000/f000000 [0, 1468) |
| [store-footprint-metadata-cardinality-100000](../view/verification/store_footprint/store-footprint-metadata-cardinality-100000/verification.json) | PASS | 11.196098917003837 | bounded bytes; d0000/f000000 [0, 1468) |
| [store-footprint-large-object-500m](../view/verification/store_footprint/store-footprint-large-object-500m/verification.json) | PASS | 6.7499664170027245 | bounded bytes; d0000/file-00000.bin [4375525, 4506607) |
| [store-footprint-unique-100-low-v1](../view/verification/store_footprint/store-footprint-unique-100-low-v1/verification.json) | PASS | 2.124764124993817 | bounded bytes; d0000/f000000 [0, 1097) |
| [store-footprint-metadata-cardinality-100-low-v1](../view/verification/store_footprint/store-footprint-metadata-cardinality-100-low-v1/verification.json) | PASS | 2.1978761659993324 | bounded bytes; d0000/f000000 [0, 1097) |
| [store-footprint-large-object-10m-low-v1](../view/verification/store_footprint/store-footprint-large-object-10m-low-v1/verification.json) | PASS | 2.2874070830002893 | bounded bytes; d0000/file-00000.bin [396765, 527847) |
| [tiny-create-1-compact-v2](../view/verification/tiny_file_churn/tiny-create-1-compact-v2/verification.json) | PASS | 2.9827153329970315 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-create-10-compact-v2](../view/verification/tiny_file_churn/tiny-create-10-compact-v2/verification.json) | PASS | 10.8172775409912 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-create-100-mixed-v4](../view/verification/tiny_file_churn/tiny-create-100-mixed-v4/verification.json) | PASS | 12.470610000003944 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-create-500-mixed-v4](../view/verification/tiny_file_churn/tiny-create-500-mixed-v4/verification.json) | PASS | 20.35009816699312 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-stat-1-compact-v2](../view/verification/tiny_file_churn/tiny-stat-1-compact-v2/verification.json) | PASS | 1.8853386249975301 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-stat-10-compact-v2](../view/verification/tiny_file_churn/tiny-stat-10-compact-v2/verification.json) | PASS | 2.289218499994604 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-stat-100-mixed-v4](../view/verification/tiny_file_churn/tiny-stat-100-mixed-v4/verification.json) | PASS | 4.389714458986418 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-stat-500-mixed-v4](../view/verification/tiny_file_churn/tiny-stat-500-mixed-v4/verification.json) | PASS | 12.056146041999455 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-unlink-1-compact-v2](../view/verification/tiny_file_churn/tiny-unlink-1-compact-v2/verification.json) | PASS | 1.5237022079963936 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-unlink-10-compact-v2](../view/verification/tiny_file_churn/tiny-unlink-10-compact-v2/verification.json) | PASS | 1.9362801249953918 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-unlink-100-mixed-v4](../view/verification/tiny_file_churn/tiny-unlink-100-mixed-v4/verification.json) | PASS | 2.2852495840052143 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-unlink-500-mixed-v4](../view/verification/tiny_file_churn/tiny-unlink-500-mixed-v4/verification.json) | PASS | 4.326627874994301 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-create-1-compact-v2](../view/verification/tiny_file_churn/tiny-bulk-create-1-compact-v2/verification.json) | PASS | 2.0056307500053663 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-create-10-compact-v2](../view/verification/tiny_file_churn/tiny-bulk-create-10-compact-v2/verification.json) | PASS | 2.0800247919978574 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-create-100-mixed-v3](../view/verification/tiny_file_churn/tiny-bulk-create-100-mixed-v3/verification.json) | PASS | 3.5594885410100687 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-create-500-mixed-v3](../view/verification/tiny_file_churn/tiny-bulk-create-500-mixed-v3/verification.json) | PASS | 10.040339083003346 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-delete-1-compact-v2](../view/verification/tiny_file_churn/tiny-bulk-delete-1-compact-v2/verification.json) | PASS | 2.0774274169962155 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-delete-10-compact-v2](../view/verification/tiny_file_churn/tiny-bulk-delete-10-compact-v2/verification.json) | PASS | 2.5047305000043707 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-delete-100-mixed-v3](../view/verification/tiny_file_churn/tiny-bulk-delete-100-mixed-v3/verification.json) | PASS | 4.6276638340059435 | changed data and selected unchanged witnesses; qualified roots |
| [tiny-bulk-delete-500-mixed-v3](../view/verification/tiny_file_churn/tiny-bulk-delete-500-mixed-v3/verification.json) | PASS | 15.25484549999237 | changed data and selected unchanged witnesses; qualified roots |
| [namespace-subtree-relocate-delete-1-compact-v2](../view/verification/namespace_mutation/namespace-subtree-relocate-delete-1-compact-v2/verification.json) | PASS | 2.122025084012421 | changed data and selected unchanged witnesses; qualified roots |
| [namespace-subtree-relocate-delete-10-compact-v2](../view/verification/namespace_mutation/namespace-subtree-relocate-delete-10-compact-v2/verification.json) | PASS | 2.344779250008287 | changed data and selected unchanged witnesses; qualified roots |
| [namespace-subtree-relocate-delete-100-mixed-v4](../view/verification/namespace_mutation/namespace-subtree-relocate-delete-100-mixed-v4/verification.json) | PASS | 4.059544666990405 | changed data and selected unchanged witnesses; qualified roots |
| [namespace-subtree-relocate-delete-500-mixed-v4](../view/verification/namespace_mutation/namespace-subtree-relocate-delete-500-mixed-v4/verification.json) | PASS | 12.045600500001456 | changed data and selected unchanged witnesses; qualified roots |
| [directory-construct-1-compact-v2](../view/verification/directory_construction_traversal/directory-construct-1-compact-v2/verification.json) | PASS | 2.2043334170011804 | changed data and selected unchanged witnesses; qualified roots |
| [directory-construct-10-compact-v2](../view/verification/directory_construction_traversal/directory-construct-10-compact-v2/verification.json) | PASS | 2.806862750003347 | changed data and selected unchanged witnesses; qualified roots |
| [directory-construct-100-mixed-v4](../view/verification/directory_construction_traversal/directory-construct-100-mixed-v4/verification.json) | PASS | 4.407854957986274 | changed data and selected unchanged witnesses; qualified roots |
| [directory-construct-500-mixed-v4](../view/verification/directory_construction_traversal/directory-construct-500-mixed-v4/verification.json) | PASS | 12.381380874998285 | changed data and selected unchanged witnesses; qualified roots |
| [directory-metadata-scan-1-compact-v2](../view/verification/directory_construction_traversal/directory-metadata-scan-1-compact-v2/verification.json) | PASS | 2.1718460000120103 | changed data and selected unchanged witnesses; qualified roots |
| [directory-metadata-scan-10-compact-v2](../view/verification/directory_construction_traversal/directory-metadata-scan-10-compact-v2/verification.json) | PASS | 2.8906948329968145 | changed data and selected unchanged witnesses; qualified roots |
| [directory-metadata-scan-100-mixed-v4](../view/verification/directory_construction_traversal/directory-metadata-scan-100-mixed-v4/verification.json) | PASS | 4.262733458002913 | changed data and selected unchanged witnesses; qualified roots |
| [directory-metadata-scan-500-mixed-v4](../view/verification/directory_construction_traversal/directory-metadata-scan-500-mixed-v4/verification.json) | PASS | 11.782383249999839 | changed data and selected unchanged witnesses; qualified roots |
| [directory-content-scan-1-compact-v2](../view/verification/directory_construction_traversal/directory-content-scan-1-compact-v2/verification.json) | PASS | 1.812671332998434 | changed data and selected unchanged witnesses; qualified roots |
| [directory-content-scan-10-compact-v2](../view/verification/directory_construction_traversal/directory-content-scan-10-compact-v2/verification.json) | PASS | 2.687661541000125 | changed data and selected unchanged witnesses; qualified roots |
| [directory-content-scan-100-mixed-v4](../view/verification/directory_construction_traversal/directory-content-scan-100-mixed-v4/verification.json) | PASS | 3.5046589999983553 | changed data and selected unchanged witnesses; qualified roots |
| [directory-content-scan-500-mixed-v4](../view/verification/directory_construction_traversal/directory-content-scan-500-mixed-v4/verification.json) | PASS | 8.787645582997357 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-clean-commit-1-compact-v2](../view/verification/workspace_change_locality/workspace-clean-commit-1-compact-v2/verification.json) | PASS | 1.8640412090026075 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-clean-commit-10-compact-v2](../view/verification/workspace_change_locality/workspace-clean-commit-10-compact-v2/verification.json) | PASS | 2.3233794169937028 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-clean-commit-100-mixed-v4](../view/verification/workspace_change_locality/workspace-clean-commit-100-mixed-v4/verification.json) | PASS | 2.28293924999889 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-clean-commit-500-mixed-v4](../view/verification/workspace_change_locality/workspace-clean-commit-500-mixed-v4/verification.json) | PASS | 4.038838083986775 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-fixed-move-1-compact-v2](../view/verification/workspace_change_locality/workspace-fixed-move-1-compact-v2/verification.json) | PASS | 1.9129476250091102 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-fixed-move-10-compact-v2](../view/verification/workspace_change_locality/workspace-fixed-move-10-compact-v2/verification.json) | PASS | 2.318935584000428 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-fixed-move-100-mixed-v4](../view/verification/workspace_change_locality/workspace-fixed-move-100-mixed-v4/verification.json) | PASS | 2.216246042007697 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-fixed-move-500-mixed-v4](../view/verification/workspace_change_locality/workspace-fixed-move-500-mixed-v4/verification.json) | PASS | 4.073624915996334 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-distributed-sdk-edit-1-compact-v2](../view/verification/workspace_change_locality/workspace-distributed-sdk-edit-1-compact-v2/verification.json) | PASS | 1.8759311249887105 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-distributed-sdk-edit-10-compact-v2](../view/verification/workspace_change_locality/workspace-distributed-sdk-edit-10-compact-v2/verification.json) | PASS | 2.3366433339979267 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-distributed-sdk-edit-100-mixed-v4](../view/verification/workspace_change_locality/workspace-distributed-sdk-edit-100-mixed-v4/verification.json) | PASS | 2.614487792001455 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-distributed-sdk-edit-500-mixed-v4](../view/verification/workspace_change_locality/workspace-distributed-sdk-edit-500-mixed-v4/verification.json) | PASS | 7.340896000008797 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-dense-rewrite-1-compact-v2](../view/verification/workspace_change_locality/workspace-dense-rewrite-1-compact-v2/verification.json) | PASS | 1.95272699999623 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-dense-rewrite-10-compact-v2](../view/verification/workspace_change_locality/workspace-dense-rewrite-10-compact-v2/verification.json) | PASS | 2.787893749991781 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-dense-rewrite-100-mixed-v4](../view/verification/workspace_change_locality/workspace-dense-rewrite-100-mixed-v4/verification.json) | PASS | 4.341490666003665 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-dense-rewrite-500-mixed-v4](../view/verification/workspace_change_locality/workspace-dense-rewrite-500-mixed-v4/verification.json) | PASS | 11.095477333990857 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-history-distributed-1](../view/verification/dedup_branch_history/dedup-history-distributed-1/verification.json) | PASS | 2.3104635420022532 | all parent links; snapshots [0, 1] |
| [dedup-history-distributed-10](../view/verification/dedup_branch_history/dedup-history-distributed-10/verification.json) | PASS | 4.895279999997001 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-distributed-100](../view/verification/dedup_branch_history/dedup-history-distributed-100/verification.json) | PASS | 4.674999208000372 | all parent links; snapshots [0, 1, 2, 49, 50, 99, 100] |
| [dedup-history-distributed-500](../view/verification/dedup_branch_history/dedup-history-distributed-500/verification.json) | PASS | 9.918297666008584 | all parent links; snapshots [0, 1, 199, 200, 201, 250, 499, 500] |
| [dedup-history-hotset-1](../view/verification/dedup_branch_history/dedup-history-hotset-1/verification.json) | PASS | 2.121109667001292 | all parent links; snapshots [0, 1] |
| [dedup-history-hotset-10](../view/verification/dedup_branch_history/dedup-history-hotset-10/verification.json) | PASS | 4.758220750009059 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-hotset-100](../view/verification/dedup_branch_history/dedup-history-hotset-100/verification.json) | PASS | 4.867449708006461 | all parent links; snapshots [0, 1, 7, 8, 9, 50, 99, 100] |
| [dedup-history-hotset-500](../view/verification/dedup_branch_history/dedup-history-hotset-500/verification.json) | PASS | 9.105573875000118 | all parent links; snapshots [0, 1, 7, 8, 9, 250, 499, 500] |
| [dedup-history-recurring-1](../view/verification/dedup_branch_history/dedup-history-recurring-1/verification.json) | PASS | 2.3268663750059204 | all parent links; snapshots [0, 1] |
| [dedup-history-recurring-10](../view/verification/dedup_branch_history/dedup-history-recurring-10/verification.json) | PASS | 4.753103875002125 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-recurring-100](../view/verification/dedup_branch_history/dedup-history-recurring-100/verification.json) | PASS | 4.078387000001385 | all parent links; snapshots [0, 1, 2, 3, 99, 100] |
| [dedup-history-recurring-500](../view/verification/dedup_branch_history/dedup-history-recurring-500/verification.json) | PASS | 7.8799811249919 | all parent links; snapshots [0, 1, 2, 3, 499, 500] |
| [dedup-history-metadata-1](../view/verification/dedup_branch_history/dedup-history-metadata-1/verification.json) | PASS | 2.151772625002195 | all parent links; snapshots [0, 1] |
| [dedup-history-metadata-10](../view/verification/dedup_branch_history/dedup-history-metadata-10/verification.json) | PASS | 4.670538125006715 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-metadata-100](../view/verification/dedup_branch_history/dedup-history-metadata-100/verification.json) | PASS | 4.432802542010904 | all parent links; snapshots [0, 1, 2, 49, 50, 99, 100] |
| [dedup-history-metadata-500](../view/verification/dedup_branch_history/dedup-history-metadata-500/verification.json) | PASS | 8.003812166003627 | all parent links; snapshots [0, 1, 2, 249, 250, 499, 500] |
| [dedup-history-unrelated-1](../view/verification/dedup_branch_history/dedup-history-unrelated-1/verification.json) | PASS | 3.1005180409993045 | all parent links; snapshots [0, 1] |
| [dedup-history-unrelated-10](../view/verification/dedup_branch_history/dedup-history-unrelated-10/verification.json) | PASS | 13.448633166990476 | all parent links; snapshots [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |
| [dedup-history-unrelated-100-mixed-v2](../view/verification/dedup_branch_history/dedup-history-unrelated-100-mixed-v2/verification.json) | PASS | 6.536404583006515 | changed data and selected unchanged witnesses; qualified roots |
| [dedup-history-unrelated-500-mixed-v2](../view/verification/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/verification.json) | PASS | 26.312125584008754 | changed data and selected unchanged witnesses; qualified roots |
| [git-tool-1-compact-v2](../view/verification/git_tool_workflow/git-tool-1-compact-v2/verification.json) | PASS | 5.136487332987599 | full head/tree/parent and reopened custody |
| [git-tool-10-compact-v2](../view/verification/git_tool_workflow/git-tool-10-compact-v2/verification.json) | PASS | 6.751760874991305 | full head/tree/parent and reopened custody |
| [git-tool-100-mixed-v4](../view/verification/git_tool_workflow/git-tool-100-mixed-v4/verification.json) | PASS | 20.203167208004743 | full head/tree/parent and reopened custody |
| [git-tool-500-mixed-v4](../view/verification/git_tool_workflow/git-tool-500-mixed-v4/verification.json) | PASS | 36.88988016700023 | full head/tree/parent and reopened custody |
| [agent-episodes-1-compact-v2](../view/verification/mixed_load_bearing/agent-episodes-1-compact-v2/verification.json) | PASS | 2.023686542001087 | changed data and selected unchanged witnesses; qualified roots |
| [agent-episodes-10-compact-v2](../view/verification/mixed_load_bearing/agent-episodes-10-compact-v2/verification.json) | PASS | 2.9848968330043135 | changed data and selected unchanged witnesses; qualified roots |
| [agent-episodes-100](../view/verification/mixed_load_bearing/agent-episodes-100/verification.json) | PASS | 19.361520916994778 | changed data and selected unchanged witnesses; qualified roots |
| [agent-episodes-500](../view/verification/mixed_load_bearing/agent-episodes-500/verification.json) | PASS | 21.661483584000962 | changed data and selected unchanged witnesses; qualified roots |
| [workspace-invalid-sdk-edit-compact-v2-proof](../view/verification/workspace_reliability/workspace-invalid-sdk-edit-compact-v2-proof/verification.json) | PASS | 2.2472231249994365 | case-specific lifecycle/fault/recovery contract |
| [workspace-invalid-namespace-compact-v2-proof](../view/verification/workspace_reliability/workspace-invalid-namespace-compact-v2-proof/verification.json) | PASS | 2.128260958008468 | case-specific lifecycle/fault/recovery contract |
| [workspace-lease-lifecycle-compact-v2-proof](../view/verification/workspace_reliability/workspace-lease-lifecycle-compact-v2-proof/verification.json) | PASS | 2.226637125000707 | case-specific lifecycle/fault/recovery contract |
| [workspace-open-writer-busy-compact-v2-proof](../view/verification/workspace_reliability/workspace-open-writer-busy-compact-v2-proof/verification.json) | PASS | 2.511351666995324 | case-specific lifecycle/fault/recovery contract |
| [workspace-live-execution-busy-compact-v2-proof](../view/verification/workspace_reliability/workspace-live-execution-busy-compact-v2-proof/verification.json) | PASS | 2.322063125000568 | case-specific lifecycle/fault/recovery contract |
| [workspace-candidate-failure-retry-compact-v2-proof](../view/verification/workspace_reliability/workspace-candidate-failure-retry-compact-v2-proof/verification.json) | PASS | 2.1518283750046976 | case-specific lifecycle/fault/recovery contract |
| [workspace-admission-batch-failure-retry-compact-v2-proof](../view/verification/workspace_reliability/workspace-admission-batch-failure-retry-compact-v2-proof/verification.json) | PASS | 3.3047050000022864 | case-specific lifecycle/fault/recovery contract |
| [workspace-final-publication-failure-retry-compact-v2-proof](../view/verification/workspace_reliability/workspace-final-publication-failure-retry-compact-v2-proof/verification.json) | PASS | 3.2135843329888303 | case-specific lifecycle/fault/recovery contract |
| [workspace-published-presentation-failure-smoke-v3-proof](../view/verification/workspace_reliability/workspace-published-presentation-failure-smoke-v3-proof/verification.json) | PASS | 1.9959435829950962 | case-specific lifecycle/fault/recovery contract |
| [workspace-dirty-end-discard-compact-v2-proof](../view/verification/workspace_reliability/workspace-dirty-end-discard-compact-v2-proof/verification.json) | PASS | 1.9497576660069171 | case-specific lifecycle/fault/recovery contract |
| [workspace-dirty-net-zero-compact-v2-proof](../view/verification/workspace_reliability/workspace-dirty-net-zero-compact-v2-proof/verification.json) | PASS | 2.106014042001334 | case-specific lifecycle/fault/recovery contract |
| [workspace-short-spool-write-compact-v2-proof](../view/verification/workspace_reliability/workspace-short-spool-write-compact-v2-proof/verification.json) | PASS | 2.149599208001746 | case-specific lifecycle/fault/recovery contract |
| [workspace-deferred-nospace-compact-v2-proof](../view/verification/workspace_reliability/workspace-deferred-nospace-compact-v2-proof/verification.json) | PASS | 2.0626066669938155 | case-specific lifecycle/fault/recovery contract |
| [workspace-workload-cancel-compact-v2-proof](../view/verification/workspace_reliability/workspace-workload-cancel-compact-v2-proof/verification.json) | PASS | 2.599535083005321 | case-specific lifecycle/fault/recovery contract |
| [workspace-dirty-runtime-disconnect-compact-v2-proof](../view/verification/workspace_reliability/workspace-dirty-runtime-disconnect-compact-v2-proof/verification.json) | PASS | 2.5736062500072876 | case-specific lifecycle/fault/recovery contract |
| [workspace-corrupt-descendant-compact-v2-proof](../view/verification/workspace_reliability/workspace-corrupt-descendant-compact-v2-proof/verification.json) | PASS | 2.0514488750050077 | case-specific lifecycle/fault/recovery contract |
| [workspace-missing-descendant-compact-v2-proof](../view/verification/workspace_reliability/workspace-missing-descendant-compact-v2-proof/verification.json) | PASS | 1.8005512090021512 | case-specific lifecycle/fault/recovery contract |
| [workspace-parallel-read-write-compact-v2-proof](../view/verification/workspace_reliability/workspace-parallel-read-write-compact-v2-proof/verification.json) | PASS | 2.347034167003585 | case-specific lifecycle/fault/recovery contract |
| [workspace-shared-path-contention-compact-v2-proof](../view/verification/workspace_reliability/workspace-shared-path-contention-compact-v2-proof/verification.json) | PASS | 2.2403678330010734 | case-specific lifecycle/fault/recovery contract |
| [workspace-hardlink-alias-compact-v2-proof](../view/verification/workspace_reliability/workspace-hardlink-alias-compact-v2-proof/verification.json) | PASS | 2.0260079169966048 | case-specific lifecycle/fault/recovery contract |
| [workspace-symlink-semantics-compact-v2-proof](../view/verification/workspace_reliability/workspace-symlink-semantics-compact-v2-proof/verification.json) | PASS | 2.1404060409986414 | case-specific lifecycle/fault/recovery contract |
| [workspace-open-rename-unlink-compact-v2-proof](../view/verification/workspace_reliability/workspace-open-rename-unlink-compact-v2-proof/verification.json) | PASS | 2.2421420409955317 | case-specific lifecycle/fault/recovery contract |
| [workspace-metadata-chmod-compact-v2-proof](../view/verification/workspace_reliability/workspace-metadata-chmod-compact-v2-proof/verification.json) | PASS | 2.2962617500015767 | case-specific lifecycle/fault/recovery contract |
| [workspace-metadata-mtime-compact-v2-proof](../view/verification/workspace_reliability/workspace-metadata-mtime-compact-v2-proof/verification.json) | PASS | 2.1066658750060014 | case-specific lifecycle/fault/recovery contract |
| [workspace-metadata-xattr-compact-v2-proof](../view/verification/workspace_reliability/workspace-metadata-xattr-compact-v2-proof/verification.json) | PASS | 2.1019235410058172 | case-specific lifecycle/fault/recovery contract |
| [workspace-exec-500-compact-v2-proof](../view/verification/workspace_reliability/workspace-exec-500-compact-v2-proof/verification.json) | PASS | 10.43746691699198 | case-specific lifecycle/fault/recovery contract |
| [workspace-repeat-publication-compact-v2-proof](../view/verification/workspace_reliability/workspace-repeat-publication-compact-v2-proof/verification.json) | PASS | 2.169134709009086 | case-specific lifecycle/fault/recovery contract |
| workspace-sustained-600s-compact-v2-proof | NOT_RUN_OPTIONAL | — | excluded; not executed |

## Incomplete requirements

- ('git_tool_workflow', 'git-tool-1-compact-v2'): prepared fixture content differs
- ('git_tool_workflow', 'git-tool-10-compact-v2'): prepared fixture content differs
- ('git_tool_workflow', 'git-tool-100-mixed-v4'): prepared fixture content differs
- ('git_tool_workflow', 'git-tool-500-mixed-v4'): prepared fixture content differs

## Historical comparison and qualification

REVISE: supplemental storage/additional cases, repair ledger and independent review must also qualify

v0.1.3 release includes later FUSE correctness repair; complete historical performance campaign not rerun. Baseline operands are elapsed_ns, never older comparison fields. Resource scopes are individual; no overlapping sums. [All metric operands and raw n](comparison.csv).

| Family / case | Metric | v0.1.3 | v0.1.4 (n=1) | Change | Severity |
|---|---|---:|---:|---:|---|
| dedup_branch_history / dedup-history-distributed-1 | elapsed_ns | 22163084 | 31439417 | +41.85% | SEVERE |
| dedup_branch_history / dedup-history-distributed-1 | route.sample-complete.host_orchestration_ns | 24968667 | 34521334 | +38.26% | SEVERE |
| dedup_branch_history / dedup-history-distributed-1 | route.sample-complete.pure_call_sum_ns | 22163084 | 31439417 | +41.85% | SEVERE |
| dedup_branch_history / dedup-history-distributed-1 | sdk_edit_ns | 6799084 | 12242834 | +80.07% | SEVERE |
| dedup_branch_history / dedup-history-distributed-10 | elapsed_ns | 82265877 | 90197418 | +9.64% | OBSERVED_INCREASE |
| dedup_branch_history / dedup-history-distributed-100 | elapsed_ns | 586404579 | 694219077 | +18.39% | REVIEW |
| dedup_branch_history / dedup-history-distributed-500 | elapsed_ns | 2925218405 | 3175247074 | +8.55% | OBSERVED_INCREASE |
| dedup_branch_history / dedup-history-hotset-1 | elapsed_ns | 23580083 | 29027751 | +23.10% | REVIEW |
| dedup_branch_history / dedup-history-hotset-10 | elapsed_ns | 118461251 | 140843707 | +18.89% | REVIEW |
| dedup_branch_history / dedup-history-hotset-10 | sdk_edit_ns | 64619958 | 84522581 | +30.80% | SEVERE |
| dedup_branch_history / dedup-history-hotset-100 | commit_ns | 280116201 | 429124165 | +53.20% | SEVERE |
| dedup_branch_history / dedup-history-hotset-100 | elapsed_ns | 799500410 | 1163198164 | +45.49% | SEVERE |
| dedup_branch_history / dedup-history-hotset-100 | route.sample-complete.host_orchestration_ns | 943429458 | 1344411583 | +42.50% | SEVERE |
| dedup_branch_history / dedup-history-hotset-100 | route.sample-complete.pure_call_sum_ns | 799500410 | 1163198164 | +45.49% | SEVERE |
| dedup_branch_history / dedup-history-hotset-100 | sdk_edit_ns | 508239667 | 721405582 | +41.94% | SEVERE |
| dedup_branch_history / dedup-history-hotset-500 | elapsed_ns | 3683754881 | 4143199056 | +12.47% | OBSERVED_INCREASE |
| dedup_branch_history / dedup-history-metadata-1 | elapsed_ns | 20652541 | 28194584 | +36.52% | SEVERE |
| dedup_branch_history / dedup-history-metadata-1 | route.sample-complete.host_orchestration_ns | 22170791 | 30392208 | +37.08% | SEVERE |
| dedup_branch_history / dedup-history-metadata-1 | route.sample-complete.pure_call_sum_ns | 20652541 | 28194584 | +36.52% | SEVERE |
| dedup_branch_history / dedup-history-metadata-10 | elapsed_ns | 76473917 | 94298669 | +23.31% | REVIEW |
| dedup_branch_history / dedup-history-metadata-100 | elapsed_ns | 649605248 | 787525582 | +21.23% | REVIEW |
| dedup_branch_history / dedup-history-metadata-500 | commit_ns | 1275061660 | 1658594586 | +30.08% | SEVERE |
| dedup_branch_history / dedup-history-metadata-500 | elapsed_ns | 3231966647 | 4041859457 | +25.06% | REVIEW |
| dedup_branch_history / dedup-history-metadata-500 | route.sample-complete.orchestration_unattributed_ns | 423390312 | 569803918 | +34.58% | SEVERE |
| dedup_branch_history / dedup-history-recurring-1 | elapsed_ns | 21456208 | 35135625 | +63.76% | SEVERE |
| dedup_branch_history / dedup-history-recurring-1 | route.sample-complete.host_orchestration_ns | 24101625 | 38144667 | +58.27% | SEVERE |
| dedup_branch_history / dedup-history-recurring-1 | route.sample-complete.pure_call_sum_ns | 21456208 | 35135625 | +63.76% | SEVERE |
| dedup_branch_history / dedup-history-recurring-1 | sdk_edit_ns | 6660917 | 12533000 | +88.16% | SEVERE |
| dedup_branch_history / dedup-history-recurring-10 | commit_ns | 32854749 | 47474791 | +44.50% | SEVERE |
| dedup_branch_history / dedup-history-recurring-10 | elapsed_ns | 62918081 | 91313624 | +45.13% | SEVERE |
| dedup_branch_history / dedup-history-recurring-10 | route.sample-complete.host_orchestration_ns | 78055708 | 111717084 | +43.12% | SEVERE |
| dedup_branch_history / dedup-history-recurring-10 | route.sample-complete.orchestration_unattributed_ns | 15137627 | 20403460 | +34.79% | SEVERE |
| dedup_branch_history / dedup-history-recurring-10 | route.sample-complete.pure_call_sum_ns | 62918081 | 91313624 | +45.13% | SEVERE |
| dedup_branch_history / dedup-history-recurring-10 | sdk_edit_ns | 20887748 | 32318667 | +54.73% | SEVERE |
| dedup_branch_history / dedup-history-recurring-100 | commit_ns | 302301708 | 481866957 | +59.40% | SEVERE |
| dedup_branch_history / dedup-history-recurring-100 | container_cpu_ns | 74392000 | 125612000 | +68.85% | SEVERE |
| dedup_branch_history / dedup-history-recurring-100 | elapsed_ns | 462840955 | 734400949 | +58.67% | SEVERE |
| dedup_branch_history / dedup-history-recurring-100 | route.sample-complete.host_orchestration_ns | 605354583 | 924109583 | +52.66% | SEVERE |
| dedup_branch_history / dedup-history-recurring-100 | route.sample-complete.pure_call_sum_ns | 462840955 | 734400949 | +58.67% | SEVERE |
| dedup_branch_history / dedup-history-recurring-100 | sdk_edit_ns | 151687247 | 239791617 | +58.08% | SEVERE |
| dedup_branch_history / dedup-history-recurring-500 | commit_ns | 1421468082 | 2013385422 | +41.64% | SEVERE |
| dedup_branch_history / dedup-history-recurring-500 | container_cpu_ns | 297340000 | 446924000 | +50.31% | SEVERE |
| dedup_branch_history / dedup-history-recurring-500 | elapsed_ns | 2145416191 | 3045772393 | +41.97% | SEVERE |
| dedup_branch_history / dedup-history-recurring-500 | route.sample-complete.host_orchestration_ns | 2857950000 | 3971268875 | +38.96% | SEVERE |
| dedup_branch_history / dedup-history-recurring-500 | route.sample-complete.pure_call_sum_ns | 2145416191 | 3045772393 | +41.97% | SEVERE |
| dedup_branch_history / dedup-history-recurring-500 | sdk_edit_ns | 712460568 | 1019229263 | +43.06% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-1 | commit_ns | 28888417 | 45132666 | +56.23% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-1 | elapsed_ns | 880163541 | 1171338333 | +33.08% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-1 | exec_ns | 840214042 | 1114515625 | +32.65% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-1 | route.sample-complete.host_orchestration_ns | 882015167 | 1173343875 | +33.03% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-1 | route.sample-complete.pure_call_sum_ns | 880163541 | 1171338333 | +33.08% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-10 | commit_ns | 350191248 | 521013290 | +48.78% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-10 | elapsed_ns | 9543474916 | 12093765959 | +26.72% | REVIEW |
| dedup_branch_history / dedup-history-unrelated-100-mixed-v2 | commit_ns | 798292832 | 1829277170 | +129.15% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-100-mixed-v2 | elapsed_ns | 3549609120 | 5125323582 | +44.39% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-100-mixed-v2 | host.after-product-minus-before.disk_write_bytes | 106508288 | 173264896 | +62.68% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-100-mixed-v2 | host_cpu_ns | 1448705208 | 2498240625 | +72.45% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-100-mixed-v2 | route.sample-complete.host_orchestration_ns | 3645729584 | 5244703208 | +43.86% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-100-mixed-v2 | route.sample-complete.pure_call_sum_ns | 3549609120 | 5125323582 | +44.39% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-500-mixed-v2 | commit_ns | 4317425423 | 9425460782 | +118.31% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-500-mixed-v2 | elapsed_ns | 18163888871 | 25637566161 | +41.15% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-500-mixed-v2 | host.after-product-minus-before.disk_write_bytes | 532520960 | 865849344 | +62.59% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-500-mixed-v2 | host_cpu_ns | 7438216500 | 12641764876 | +69.96% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-500-mixed-v2 | route.sample-complete.host_orchestration_ns | 18665382917 | 26301081417 | +40.91% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-500-mixed-v2 | route.sample-complete.orchestration_unattributed_ns | 501494046 | 663515256 | +32.31% | SEVERE |
| dedup_branch_history / dedup-history-unrelated-500-mixed-v2 | route.sample-complete.pure_call_sum_ns | 18163888871 | 25637566161 | +41.15% | SEVERE |
| dedup_cdc_locality / dedup-cdc-common-body-1 | elapsed_ns | 4125125 | 6691667 | +62.22% | REVIEW |
| dedup_cdc_locality / dedup-cdc-common-body-10 | elapsed_ns | 20395333 | 25391000 | +24.49% | REVIEW |
| dedup_cdc_locality / dedup-cdc-common-body-100 | elapsed_ns | 58208209 | 189509833 | +225.57% | SEVERE |
| dedup_cdc_locality / dedup-cdc-common-body-100 | route.sample-complete.host_orchestration_ns | 58365500 | 189740333 | +225.09% | SEVERE |
| dedup_cdc_locality / dedup-cdc-common-body-100 | route.sample-complete.pure_call_sum_ns | 58208209 | 189509833 | +225.57% | SEVERE |
| dedup_cdc_locality / dedup-cdc-common-body-500 | elapsed_ns | 244350083 | 990060125 | +305.18% | SEVERE |
| dedup_cdc_locality / dedup-cdc-common-body-500 | route.sample-complete.host_orchestration_ns | 244506625 | 990276000 | +305.01% | SEVERE |
| dedup_cdc_locality / dedup-cdc-common-body-500 | route.sample-complete.pure_call_sum_ns | 244350083 | 990060125 | +305.18% | SEVERE |
| dedup_cdc_locality / dedup-cdc-delete-1 | elapsed_ns | 4411750 | 6297917 | +42.75% | REVIEW |
| dedup_cdc_locality / dedup-cdc-delete-10 | elapsed_ns | 18224917 | 19955209 | +9.49% | OBSERVED_INCREASE |
| dedup_cdc_locality / dedup-cdc-delete-100 | elapsed_ns | 33165583 | 150029916 | +352.37% | SEVERE |
| dedup_cdc_locality / dedup-cdc-delete-100 | route.sample-complete.host_orchestration_ns | 33317334 | 150283667 | +351.07% | SEVERE |
| dedup_cdc_locality / dedup-cdc-delete-100 | route.sample-complete.pure_call_sum_ns | 33165583 | 150029916 | +352.37% | SEVERE |
| dedup_cdc_locality / dedup-cdc-delete-500 | elapsed_ns | 130863875 | 765470333 | +484.94% | SEVERE |
| dedup_cdc_locality / dedup-cdc-delete-500 | route.sample-complete.host_orchestration_ns | 131013583 | 765743083 | +484.48% | SEVERE |
| dedup_cdc_locality / dedup-cdc-delete-500 | route.sample-complete.pure_call_sum_ns | 130863875 | 765470333 | +484.94% | SEVERE |
| dedup_cdc_locality / dedup-cdc-insert-1 | elapsed_ns | 3897167 | 5957917 | +52.88% | REVIEW |
| dedup_cdc_locality / dedup-cdc-insert-10 | elapsed_ns | 18057833 | 19470709 | +7.82% | OBSERVED_INCREASE |
| dedup_cdc_locality / dedup-cdc-insert-100 | elapsed_ns | 34183250 | 158100250 | +362.51% | SEVERE |
| dedup_cdc_locality / dedup-cdc-insert-100 | route.sample-complete.host_orchestration_ns | 34422875 | 158381583 | +360.11% | SEVERE |
| dedup_cdc_locality / dedup-cdc-insert-100 | route.sample-complete.pure_call_sum_ns | 34183250 | 158100250 | +362.51% | SEVERE |
| dedup_cdc_locality / dedup-cdc-insert-500 | elapsed_ns | 136775833 | 775264250 | +466.81% | SEVERE |
| dedup_cdc_locality / dedup-cdc-insert-500 | route.sample-complete.host_orchestration_ns | 136931333 | 775465458 | +466.32% | SEVERE |
| dedup_cdc_locality / dedup-cdc-insert-500 | route.sample-complete.pure_call_sum_ns | 136775833 | 775264250 | +466.81% | SEVERE |
| dedup_cdc_locality / dedup-cdc-overwrite-1 | elapsed_ns | 4002292 | 7247375 | +81.08% | REVIEW |
| dedup_cdc_locality / dedup-cdc-overwrite-10 | elapsed_ns | 20448459 | 19061417 | -6.78% | NO_INCREASE |
| dedup_cdc_locality / dedup-cdc-overwrite-100 | elapsed_ns | 30952625 | 143658542 | +364.12% | SEVERE |
| dedup_cdc_locality / dedup-cdc-overwrite-100 | route.sample-complete.host_orchestration_ns | 31085834 | 143931292 | +363.01% | SEVERE |
| dedup_cdc_locality / dedup-cdc-overwrite-100 | route.sample-complete.pure_call_sum_ns | 30952625 | 143658542 | +364.12% | SEVERE |
| dedup_cdc_locality / dedup-cdc-overwrite-500 | elapsed_ns | 122035083 | 769798750 | +530.80% | SEVERE |
| dedup_cdc_locality / dedup-cdc-overwrite-500 | route.sample-complete.host_orchestration_ns | 122184125 | 770030583 | +530.22% | SEVERE |
| dedup_cdc_locality / dedup-cdc-overwrite-500 | route.sample-complete.pure_call_sum_ns | 122035083 | 769798750 | +530.80% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-1 | elapsed_ns | 4647500 | 8233292 | +77.16% | REVIEW |
| dedup_cdc_locality / dedup-cdc-scattered-10 | elapsed_ns | 23469041 | 34552500 | +47.23% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-10 | route.sample-complete.host_orchestration_ns | 23614208 | 34747500 | +47.15% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-10 | route.sample-complete.pure_call_sum_ns | 23469041 | 34552500 | +47.23% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-100 | elapsed_ns | 89368333 | 306408500 | +242.86% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-100 | route.sample-complete.host_orchestration_ns | 89527625 | 306609333 | +242.47% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-100 | route.sample-complete.pure_call_sum_ns | 89368333 | 306408500 | +242.86% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-500 | elapsed_ns | 483006833 | 1332843667 | +175.95% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-500 | route.sample-complete.host_orchestration_ns | 483170208 | 1333050250 | +175.90% | SEVERE |
| dedup_cdc_locality / dedup-cdc-scattered-500 | route.sample-complete.pure_call_sum_ns | 483006833 | 1332843667 | +175.95% | SEVERE |
| dedup_cross_file / dedup-cross-file-anchor-1 | elapsed_ns | 3808291 | 6507500 | +70.88% | REVIEW |
| dedup_cross_file / dedup-cross-file-identical-10 | elapsed_ns | 18395375 | 19477583 | +5.88% | OBSERVED_INCREASE |
| dedup_cross_file / dedup-cross-file-identical-100 | elapsed_ns | 28959584 | 125687167 | +334.01% | SEVERE |
| dedup_cross_file / dedup-cross-file-identical-100 | route.sample-complete.host_orchestration_ns | 29114000 | 125890792 | +332.41% | SEVERE |
| dedup_cross_file / dedup-cross-file-identical-100 | route.sample-complete.pure_call_sum_ns | 28959584 | 125687167 | +334.01% | SEVERE |
| dedup_cross_file / dedup-cross-file-identical-500 | elapsed_ns | 110630125 | 682648958 | +517.06% | SEVERE |
| dedup_cross_file / dedup-cross-file-identical-500 | route.sample-complete.host_orchestration_ns | 110801750 | 682873375 | +516.30% | SEVERE |
| dedup_cross_file / dedup-cross-file-identical-500 | route.sample-complete.pure_call_sum_ns | 110630125 | 682648958 | +517.06% | SEVERE |
| dedup_cross_file / dedup-cross-file-mixed-10 | elapsed_ns | 24046375 | 30754875 | +27.90% | REVIEW |
| dedup_cross_file / dedup-cross-file-mixed-100 | elapsed_ns | 113943625 | 296842625 | +160.52% | SEVERE |
| dedup_cross_file / dedup-cross-file-mixed-100 | route.sample-complete.host_orchestration_ns | 114128083 | 297082708 | +160.31% | SEVERE |
| dedup_cross_file / dedup-cross-file-mixed-100 | route.sample-complete.pure_call_sum_ns | 113943625 | 296842625 | +160.52% | SEVERE |
| dedup_cross_file / dedup-cross-file-mixed-500 | elapsed_ns | 541992375 | 1350345000 | +149.14% | SEVERE |
| dedup_cross_file / dedup-cross-file-mixed-500 | route.sample-complete.host_orchestration_ns | 542163167 | 1350517583 | +149.10% | SEVERE |
| dedup_cross_file / dedup-cross-file-mixed-500 | route.sample-complete.pure_call_sum_ns | 541992375 | 1350345000 | +149.14% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-10 | elapsed_ns | 22389542 | 33043916 | +47.59% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-10 | route.sample-complete.host_orchestration_ns | 22540292 | 33259959 | +47.56% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-10 | route.sample-complete.pure_call_sum_ns | 22389542 | 33043916 | +47.59% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-100 | elapsed_ns | 93266750 | 296812834 | +218.24% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-100 | route.sample-complete.host_orchestration_ns | 93441084 | 296995833 | +217.84% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-100 | route.sample-complete.pure_call_sum_ns | 93266750 | 296812834 | +218.24% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-500 | elapsed_ns | 444949333 | 1442647875 | +224.23% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-500 | route.sample-complete.host_orchestration_ns | 445121292 | 1442916042 | +224.16% | SEVERE |
| dedup_cross_file / dedup-cross-file-unique-500 | route.sample-complete.pure_call_sum_ns | 444949333 | 1442647875 | +224.23% | SEVERE |
| dedup_workspace_reuse / dedup-workspace-exact-1-compact-v2 | elapsed_ns | 31563458 | 36174457 | +14.61% | OBSERVED_INCREASE |
| dedup_workspace_reuse / dedup-workspace-exact-10-compact-v2 | elapsed_ns | 109324875 | 94423792 | -13.63% | NO_INCREASE |
| dedup_workspace_reuse / dedup-workspace-exact-100 | elapsed_ns | 840527750 | 718355875 | -14.54% | NO_INCREASE |
| dedup_workspace_reuse / dedup-workspace-exact-500 | elapsed_ns | 4296598375 | 3925196458 | -8.64% | NO_INCREASE |
| dedup_workspace_reuse / dedup-workspace-local-1-compact-v2 | elapsed_ns | 30610293 | 31678749 | +3.49% | OBSERVED_INCREASE |
| dedup_workspace_reuse / dedup-workspace-local-10-compact-v2 | elapsed_ns | 103153458 | 95429584 | -7.49% | NO_INCREASE |
| dedup_workspace_reuse / dedup-workspace-local-100 | elapsed_ns | 808387126 | 793132584 | -1.89% | NO_INCREASE |
| dedup_workspace_reuse / dedup-workspace-local-500 | elapsed_ns | 4307360416 | 3973389666 | -7.75% | NO_INCREASE |
| dedup_workspace_reuse / dedup-workspace-unique-1-base128-v3 | elapsed_ns | 33382084 | 32976207 | -1.22% | NO_INCREASE |
| dedup_workspace_reuse / dedup-workspace-unique-1-compact-v2 | elapsed_ns | 27475043 | 37898958 | +37.94% | SEVERE |
| dedup_workspace_reuse / dedup-workspace-unique-1-compact-v2 | route.sample-complete.host_orchestration_ns | 29240500 | 39925917 | +36.54% | SEVERE |
| dedup_workspace_reuse / dedup-workspace-unique-1-compact-v2 | route.sample-complete.pure_call_sum_ns | 27475043 | 37898958 | +37.94% | SEVERE |
| dedup_workspace_reuse / dedup-workspace-unique-10-base128-v3 | elapsed_ns | 103006542 | 106992709 | +3.87% | OBSERVED_INCREASE |
| dedup_workspace_reuse / dedup-workspace-unique-10-compact-v2 | elapsed_ns | 95669332 | 105688210 | +10.47% | OBSERVED_INCREASE |
| dedup_workspace_reuse / dedup-workspace-unique-100 | elapsed_ns | 768004167 | 831453625 | +8.26% | OBSERVED_INCREASE |
| dedup_workspace_reuse / dedup-workspace-unique-500 | elapsed_ns | 4319934208 | 3936558000 | -8.87% | NO_INCREASE |
| directory_construction_traversal / directory-construct-1-compact-v2 | elapsed_ns | 17022374 | 25454167 | +49.53% | SEVERE |
| directory_construction_traversal / directory-construct-1-compact-v2 | route.sample-complete.host_orchestration_ns | 18616583 | 27812416 | +49.40% | SEVERE |
| directory_construction_traversal / directory-construct-1-compact-v2 | route.sample-complete.pure_call_sum_ns | 17022374 | 25454167 | +49.53% | SEVERE |
| directory_construction_traversal / directory-construct-10-compact-v2 | commit_ns | 4571375 | 9572875 | +109.41% | SEVERE |
| directory_construction_traversal / directory-construct-10-compact-v2 | elapsed_ns | 39346916 | 47304707 | +20.22% | REVIEW |
| directory_construction_traversal / directory-construct-100-mixed-v4 | commit_ns | 13697458 | 32607959 | +138.06% | SEVERE |
| directory_construction_traversal / directory-construct-100-mixed-v4 | elapsed_ns | 216059959 | 339071500 | +56.93% | SEVERE |
| directory_construction_traversal / directory-construct-100-mixed-v4 | exec_ns | 191301375 | 289401291 | +51.28% | SEVERE |
| directory_construction_traversal / directory-construct-100-mixed-v4 | route.sample-complete.host_orchestration_ns | 217819334 | 341099458 | +56.60% | SEVERE |
| directory_construction_traversal / directory-construct-100-mixed-v4 | route.sample-complete.pure_call_sum_ns | 216059959 | 339071500 | +56.93% | SEVERE |
| directory_construction_traversal / directory-construct-500-mixed-v4 | commit_ns | 73471542 | 262787917 | +257.67% | SEVERE |
| directory_construction_traversal / directory-construct-500-mixed-v4 | elapsed_ns | 1030580667 | 1245070418 | +20.81% | REVIEW |
| directory_construction_traversal / directory-construct-500-mixed-v4 | host_cpu_ns | 98714917 | 350108667 | +254.67% | SEVERE |
| directory_construction_traversal / directory-content-scan-1-compact-v2 | elapsed_ns | 84759585 | 108693500 | +28.24% | REVIEW |
| directory_construction_traversal / directory-content-scan-1-compact-v2 | exec_ns | 73138292 | 96493625 | +31.93% | SEVERE |
| directory_construction_traversal / directory-content-scan-10-compact-v2 | elapsed_ns | 308870416 | 394740751 | +27.80% | REVIEW |
| directory_construction_traversal / directory-content-scan-100-mixed-v4 | elapsed_ns | 1105140625 | 1406811707 | +27.30% | REVIEW |
| directory_construction_traversal / directory-content-scan-100-mixed-v4 | host_cpu_ns | 358585501 | 546252375 | +52.34% | SEVERE |
| directory_construction_traversal / directory-content-scan-500-mixed-v4 | elapsed_ns | 3906595000 | 5415184667 | +38.62% | SEVERE |
| directory_construction_traversal / directory-content-scan-500-mixed-v4 | end_ns | 11778708 | 18084833 | +53.54% | SEVERE |
| directory_construction_traversal / directory-content-scan-500-mixed-v4 | exec_ns | 3861436000 | 5364745958 | +38.93% | SEVERE |
| directory_construction_traversal / directory-content-scan-500-mixed-v4 | host_cpu_ns | 1516647376 | 2347432666 | +54.78% | SEVERE |
| directory_construction_traversal / directory-content-scan-500-mixed-v4 | route.sample-complete.host_orchestration_ns | 3908434333 | 5417803000 | +38.62% | SEVERE |
| directory_construction_traversal / directory-content-scan-500-mixed-v4 | route.sample-complete.pure_call_sum_ns | 3906595000 | 5415184667 | +38.62% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-1-compact-v2 | elapsed_ns | 70526833 | 95406834 | +35.28% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-1-compact-v2 | exec_ns | 59086250 | 80681625 | +36.55% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-1-compact-v2 | route.sample-complete.host_orchestration_ns | 72045792 | 97369500 | +35.15% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-1-compact-v2 | route.sample-complete.pure_call_sum_ns | 70526833 | 95406834 | +35.28% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-10-compact-v2 | elapsed_ns | 100738749 | 98781125 | -1.94% | NO_INCREASE |
| directory_construction_traversal / directory-metadata-scan-100-mixed-v4 | elapsed_ns | 244238709 | 344608875 | +41.10% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-100-mixed-v4 | exec_ns | 226650917 | 325866792 | +43.77% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-100-mixed-v4 | host_cpu_ns | 86358626 | 166934459 | +93.30% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-100-mixed-v4 | route.sample-complete.host_orchestration_ns | 246173792 | 346560584 | +40.78% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-100-mixed-v4 | route.sample-complete.pure_call_sum_ns | 244238709 | 344608875 | +41.10% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-500-mixed-v4 | elapsed_ns | 504932250 | 676262332 | +33.93% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-500-mixed-v4 | exec_ns | 479975833 | 651623333 | +35.76% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-500-mixed-v4 | host_cpu_ns | 184070458 | 370187209 | +101.11% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-500-mixed-v4 | route.sample-complete.host_orchestration_ns | 506663709 | 678321209 | +33.88% | SEVERE |
| directory_construction_traversal / directory-metadata-scan-500-mixed-v4 | route.sample-complete.pure_call_sum_ns | 504932250 | 676262332 | +33.93% | SEVERE |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1 | elapsed_ns | 7582042 | 9330750 | +23.06% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1 | elapsed_ns | 7013292 | 8238250 | +17.47% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1 | elapsed_ns | 7175375 | 8412959 | +17.25% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1 | elapsed_ns | 8095500 | 12063292 | +49.01% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1 | elapsed_ns | 7869709 | 11892666 | +51.12% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 11656583 | 17317958 | +48.57% | SEVERE |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1 | elapsed_ns | 7515667 | 12497250 | +66.28% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 11227125 | 17486542 | +55.75% | SEVERE |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1 | elapsed_ns | 7852416 | 9832542 | +25.22% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1 | elapsed_ns | 8466250 | 11197292 | +32.26% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1 | elapsed_ns | 7335375 | 10190125 | +38.92% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1 | elapsed_ns | 8506500 | 10518334 | +23.65% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1 | elapsed_ns | 6854625 | 10109542 | +47.48% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1 | elapsed_ns | 8269583 | 13067750 | +58.02% | REVIEW |
| edit_canonical_chunk_count / overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 12920542 | 18000375 | +39.32% | SEVERE |
| edit_length_changing / append-tail-4k-on-100mib-ops-1 | elapsed_ns | 6254875 | 7736833 | +23.69% | REVIEW |
| edit_length_changing / append-tail-4k-on-10mib-ops-1 | elapsed_ns | 5905416 | 7837959 | +32.72% | REVIEW |
| edit_length_changing / append-tail-4k-on-1mib-ops-1 | elapsed_ns | 6261125 | 8331125 | +33.06% | REVIEW |
| edit_length_changing / append-tail-4k-on-500mib-result-capped-v2-ops-1 | elapsed_ns | 6686041 | 9916500 | +48.32% | REVIEW |
| edit_length_changing / delete-middle-4k-on-100mib-ops-1 | elapsed_ns | 6758000 | 9839916 | +45.60% | REVIEW |
| edit_length_changing / delete-middle-4k-on-10mib-ops-1 | elapsed_ns | 6569875 | 8249333 | +25.56% | REVIEW |
| edit_length_changing / delete-middle-4k-on-1mib-ops-1 | elapsed_ns | 5290000 | 9299541 | +75.79% | REVIEW |
| edit_length_changing / delete-middle-4k-on-1mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 10015584 | 16758958 | +67.33% | SEVERE |
| edit_length_changing / delete-middle-4k-on-500mib-ops-1 | elapsed_ns | 7488083 | 10336083 | +38.03% | REVIEW |
| edit_length_changing / insert-middle-4k-on-100mib-ops-1 | elapsed_ns | 12439416 | 10678584 | -14.16% | NO_INCREASE |
| edit_length_changing / insert-middle-4k-on-10mib-ops-1 | elapsed_ns | 7396250 | 9766500 | +32.05% | REVIEW |
| edit_length_changing / insert-middle-4k-on-1mib-ops-1 | elapsed_ns | 5589750 | 8821542 | +57.82% | REVIEW |
| edit_length_changing / insert-middle-4k-on-500mib-result-capped-v2-ops-1 | elapsed_ns | 18010583 | 9689791 | -46.20% | NO_INCREASE |
| edit_length_changing / prepend-head-4k-on-100mib-ops-1 | elapsed_ns | 6269458 | 9046042 | +44.29% | REVIEW |
| edit_length_changing / prepend-head-4k-on-10mib-ops-1 | elapsed_ns | 5671583 | 9153166 | +61.39% | REVIEW |
| edit_length_changing / prepend-head-4k-on-1mib-ops-1 | elapsed_ns | 5905500 | 9342000 | +58.19% | REVIEW |
| edit_length_changing / prepend-head-4k-on-500mib-result-capped-v2-ops-1 | elapsed_ns | 7098667 | 9857667 | +38.87% | REVIEW |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-100mib-ops-1 | elapsed_ns | 6381708 | 14170208 | +122.04% | SEVERE |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-100mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 10269666 | 20179958 | +96.50% | SEVERE |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-100mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.edit_commit_ns | 6381708 | 14170208 | +122.04% | SEVERE |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-10mib-ops-1 | elapsed_ns | 6462958 | 7474292 | +15.65% | REVIEW |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-1mib-ops-1 | commit_ns | 3768666 | 9395333 | +149.30% | SEVERE |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-1mib-ops-1 | elapsed_ns | 5744083 | 13715708 | +138.78% | SEVERE |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-1mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 10544667 | 21762041 | +106.38% | SEVERE |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-1mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.commit_call_ns | 3768666 | 9395333 | +149.30% | SEVERE |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-1mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.edit_commit_ns | 5744083 | 13715708 | +138.78% | SEVERE |
| edit_length_changing / replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1 | elapsed_ns | 7921917 | 10596041 | +33.76% | REVIEW |
| edit_length_changing / replace-shrink-middle-4k-to-2k-on-100mib-ops-1 | elapsed_ns | 7007625 | 9389917 | +34.00% | REVIEW |
| edit_length_changing / replace-shrink-middle-4k-to-2k-on-10mib-ops-1 | elapsed_ns | 5769750 | 9974916 | +72.88% | REVIEW |
| edit_length_changing / replace-shrink-middle-4k-to-2k-on-1mib-ops-1 | elapsed_ns | 5880000 | 7777084 | +32.26% | REVIEW |
| edit_length_changing / replace-shrink-middle-4k-to-2k-on-500mib-ops-1 | elapsed_ns | 6996750 | 12269917 | +75.37% | SEVERE |
| edit_length_changing / replace-shrink-middle-4k-to-2k-on-500mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 11387792 | 17060875 | +49.82% | SEVERE |
| edit_length_changing / replace-shrink-middle-4k-to-2k-on-500mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.edit_commit_ns | 6996750 | 12269917 | +75.37% | SEVERE |
| edit_length_changing / truncate-tail-4k-on-100mib-ops-1 | elapsed_ns | 6527834 | 10381792 | +59.04% | REVIEW |
| edit_length_changing / truncate-tail-4k-on-10mib-ops-1 | elapsed_ns | 15641333 | 8465000 | -45.88% | NO_INCREASE |
| edit_length_changing / truncate-tail-4k-on-1mib-ops-1 | elapsed_ns | 5293459 | 8424666 | +59.15% | REVIEW |
| edit_length_changing / truncate-tail-4k-on-500mib-ops-1 | create_ns | 7822375 | 14478625 | +85.09% | SEVERE |
| edit_length_changing / truncate-tail-4k-on-500mib-ops-1 | elapsed_ns | 6895459 | 9530917 | +38.22% | REVIEW |
| edit_length_changing / truncate-tail-4k-on-500mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.workspace_create_ns | 7822375 | 14478625 | +85.09% | SEVERE |
| edit_length_changing / zero-extend-tail-4k-on-100mib-ops-1 | elapsed_ns | 6403792 | 8436791 | +31.75% | REVIEW |
| edit_length_changing / zero-extend-tail-4k-on-10mib-ops-1 | elapsed_ns | 5637917 | 7954875 | +41.10% | REVIEW |
| edit_length_changing / zero-extend-tail-4k-on-1mib-ops-1 | elapsed_ns | 5633000 | 10496292 | +86.34% | REVIEW |
| edit_length_changing / zero-extend-tail-4k-on-1mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 10095791 | 15587208 | +54.39% | SEVERE |
| edit_length_changing / zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1 | elapsed_ns | 6663791 | 9305416 | +39.64% | REVIEW |
| edit_length_preserving / overwrite-head-4k-on-100mib-ops-1 | elapsed_ns | 6651583 | 7944583 | +19.44% | REVIEW |
| edit_length_preserving / overwrite-head-4k-on-10mib-ops-1 | elapsed_ns | 6154125 | 8576458 | +39.36% | REVIEW |
| edit_length_preserving / overwrite-head-4k-on-1mib-ops-1 | elapsed_ns | 5777000 | 7944708 | +37.52% | REVIEW |
| edit_length_preserving / overwrite-head-4k-on-500mib-ops-1 | elapsed_ns | 8411084 | 9620167 | +14.37% | OBSERVED_INCREASE |
| edit_length_preserving / overwrite-middle-4k-on-100mib-ops-1 | elapsed_ns | 6188333 | 9383375 | +51.63% | REVIEW |
| edit_length_preserving / overwrite-middle-4k-on-10mib-ops-1 | elapsed_ns | 5924958 | 12248792 | +106.73% | SEVERE |
| edit_length_preserving / overwrite-middle-4k-on-10mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.cgroup_window_duration_ns | 9979875 | 17368625 | +74.04% | SEVERE |
| edit_length_preserving / overwrite-middle-4k-on-10mib-ops-1 | route.fs-bench-pro-sdk-edit-performance-v1.edit_commit_ns | 5924958 | 12248792 | +106.73% | SEVERE |
| edit_length_preserving / overwrite-middle-4k-on-1mib-ops-1 | elapsed_ns | 6138666 | 7083666 | +15.39% | OBSERVED_INCREASE |
| edit_length_preserving / overwrite-middle-4k-on-500mib-ops-1 | elapsed_ns | 7174000 | 11123125 | +55.05% | REVIEW |
| edit_length_preserving / overwrite-tail-4k-on-100mib-ops-1 | elapsed_ns | 7631041 | 9244042 | +21.14% | REVIEW |
| edit_length_preserving / overwrite-tail-4k-on-10mib-ops-1 | elapsed_ns | 7265084 | 8920709 | +22.79% | REVIEW |
| edit_length_preserving / overwrite-tail-4k-on-1mib-ops-1 | elapsed_ns | 6112375 | 7900875 | +29.26% | REVIEW |
| edit_length_preserving / overwrite-tail-4k-on-500mib-ops-1 | elapsed_ns | 9377875 | 10039333 | +7.05% | OBSERVED_INCREASE |
| git_tool_workflow / git-tool-1-compact-v2 | elapsed_ns | 318657624 | 396924918 | unavailable | INELIGIBLE |
| git_tool_workflow / git-tool-10-compact-v2 | elapsed_ns | 611383084 | 769867540 | unavailable | INELIGIBLE |
| git_tool_workflow / git-tool-100-mixed-v4 | elapsed_ns | 1879182458 | 2396650001 | unavailable | INELIGIBLE |
| git_tool_workflow / git-tool-500-mixed-v4 | elapsed_ns | 4689306499 | 5806735418 | unavailable | INELIGIBLE |
| init_namespace / namespace-100-compact-v3 | elapsed_ns | 8901875 | 19893333 | +123.47% | SEVERE |
| init_namespace / namespace-100-compact-v3 | route.fs-bench-pro-namespace-v3.layerstack_init_ns | 8901875 | 19893333 | +123.47% | SEVERE |
| init_namespace / namespace-1000-compact-v3 | elapsed_ns | 38077083 | 81556500 | +114.19% | SEVERE |
| init_namespace / namespace-1000-compact-v3 | route.fs-bench-pro-namespace-v3.layerstack_init_ns | 38077083 | 81556500 | +114.19% | SEVERE |
| init_namespace / namespace-10000 | elapsed_ns | 403467916 | 997947125 | +147.34% | SEVERE |
| init_namespace / namespace-10000 | route.fs-bench-pro-namespace-v3.initialization_user_cpu_ns | 662520625 | 1077342417 | +62.61% | SEVERE |
| init_namespace / namespace-10000 | route.fs-bench-pro-namespace-v3.layerstack_init_ns | 403467916 | 997947125 | +147.34% | SEVERE |
| init_namespace / namespace-100000 | elapsed_ns | 2603162083 | 3803760250 | +46.12% | SEVERE |
| init_namespace / namespace-100000 | route.fs-bench-pro-namespace-v3.initialization_user_cpu_ns | 2481059666 | 3873391625 | +56.12% | SEVERE |
| init_namespace / namespace-100000 | route.fs-bench-pro-namespace-v3.layerstack_init_ns | 2603162083 | 3803760250 | +46.12% | SEVERE |
| mixed_load_bearing / agent-episodes-1-compact-v2 | elapsed_ns | 26241042 | 33484959 | +27.61% | REVIEW |
| mixed_load_bearing / agent-episodes-10-compact-v2 | commit_ns | 6675542 | 12343208 | +84.90% | SEVERE |
| mixed_load_bearing / agent-episodes-10-compact-v2 | elapsed_ns | 89910918 | 108465083 | +20.64% | REVIEW |
| mixed_load_bearing / agent-episodes-100 | commit_ns | 45420125 | 124335250 | +173.74% | SEVERE |
| mixed_load_bearing / agent-episodes-100 | elapsed_ns | 908411042 | 1169385332 | +28.73% | REVIEW |
| mixed_load_bearing / agent-episodes-100 | host_cpu_ns | 247805708 | 421193625 | +69.97% | SEVERE |
| mixed_load_bearing / agent-episodes-500 | commit_ns | 172414000 | 336113625 | +94.95% | SEVERE |
| mixed_load_bearing / agent-episodes-500 | elapsed_ns | 7535400625 | 8523980209 | +13.12% | OBSERVED_INCREASE |
| namespace_mutation / namespace-subtree-relocate-delete-1-compact-v2 | elapsed_ns | 21310793 | 27011749 | +26.75% | REVIEW |
| namespace_mutation / namespace-subtree-relocate-delete-10-compact-v2 | elapsed_ns | 59144541 | 75995500 | +28.49% | REVIEW |
| namespace_mutation / namespace-subtree-relocate-delete-100-mixed-v4 | commit_ns | 4865667 | 10606334 | +117.98% | SEVERE |
| namespace_mutation / namespace-subtree-relocate-delete-100-mixed-v4 | elapsed_ns | 51018082 | 66061667 | +29.49% | REVIEW |
| namespace_mutation / namespace-subtree-relocate-delete-500-mixed-v4 | commit_ns | 9644333 | 19891000 | +106.25% | SEVERE |
| namespace_mutation / namespace-subtree-relocate-delete-500-mixed-v4 | elapsed_ns | 182893833 | 245514375 | +34.24% | SEVERE |
| namespace_mutation / namespace-subtree-relocate-delete-500-mixed-v4 | exec_ns | 161251750 | 211860292 | +31.38% | SEVERE |
| namespace_mutation / namespace-subtree-relocate-delete-500-mixed-v4 | host_cpu_ns | 56176624 | 167247833 | +197.72% | SEVERE |
| namespace_mutation / namespace-subtree-relocate-delete-500-mixed-v4 | route.sample-complete.host_orchestration_ns | 184437791 | 247845125 | +34.38% | SEVERE |
| namespace_mutation / namespace-subtree-relocate-delete-500-mixed-v4 | route.sample-complete.pure_call_sum_ns | 182893833 | 245514375 | +34.24% | SEVERE |
| payload_create_read / payload-create-100m | container_cpu_ns | 120017000 | 189974000 | +58.29% | SEVERE |
| payload_create_read / payload-create-100m | elapsed_ns | 682771167 | 662002582 | -3.04% | NO_INCREASE |
| payload_create_read / payload-create-10m-compact-v2 | elapsed_ns | 91556041 | 84415708 | -7.80% | NO_INCREASE |
| payload_create_read / payload-create-1m-compact-v2 | elapsed_ns | 28837459 | 32793624 | +13.72% | OBSERVED_INCREASE |
| payload_create_read / payload-create-500m | elapsed_ns | 3068249542 | 2701167542 | -11.96% | NO_INCREASE |
| payload_create_read / payload-random-read-1-compact-v2 | elapsed_ns | 14584500 | 17123751 | +17.41% | REVIEW |
| payload_create_read / payload-random-read-10-compact-v2 | elapsed_ns | 17420292 | 22835958 | +31.09% | SEVERE |
| payload_create_read / payload-random-read-10-compact-v2 | route.sample-complete.pure_call_sum_ns | 17420292 | 22835958 | +31.09% | SEVERE |
| payload_create_read / payload-random-read-100 | elapsed_ns | 56396250 | 64856375 | +15.00% | REVIEW |
| payload_create_read / payload-random-read-100 | host_cpu_ns | 36215459 | 87078416 | +140.45% | SEVERE |
| payload_create_read / payload-random-read-500 | elapsed_ns | 219687084 | 259409875 | +18.08% | REVIEW |
| payload_create_read / payload-random-read-500 | host_cpu_ns | 92612250 | 162618500 | +75.59% | SEVERE |
| store_footprint / store-footprint-large-object-10m-low-v1 | elapsed_ns | 43893543 | 71377250 | +62.61% | SEVERE |
| store_footprint / store-footprint-large-object-10m-low-v1 | route.fs-bench-pro-store-footprint-performance-v5.initialization_ns | 22268208 | 41195125 | +85.00% | SEVERE |
| store_footprint / store-footprint-large-object-10m-low-v1 | route.product-timing.initialization_ns | 22268208 | 41195125 | +85.00% | SEVERE |
| store_footprint / store-footprint-large-object-10m-low-v1 | route.product-timing.product_call_sum_ns | 43893543 | 71377250 | +62.61% | SEVERE |
| store_footprint / store-footprint-large-object-500m | elapsed_ns | 491905625 | 1505693374 | +206.09% | SEVERE |
| store_footprint / store-footprint-large-object-500m | host_cpu_ns | 1294280208 | 2203894292 | +70.28% | SEVERE |
| store_footprint / store-footprint-large-object-500m | route.fs-bench-pro-store-footprint-performance-v5.complete_ns | 1125911458 | 1934183000 | +71.79% | SEVERE |
| store_footprint / store-footprint-large-object-500m | route.fs-bench-pro-store-footprint-performance-v5.initialization_ns | 460296417 | 1453797916 | +215.84% | SEVERE |
| store_footprint / store-footprint-large-object-500m | route.fs-bench-pro-store-footprint-performance-v5.process_system_cpu_ns | 412516583 | 767315792 | +86.01% | SEVERE |
| store_footprint / store-footprint-large-object-500m | route.fs-bench-pro-store-footprint-performance-v5.process_user_cpu_ns | 881763625 | 1436578500 | +62.92% | SEVERE |
| store_footprint / store-footprint-large-object-500m | route.product-timing.create_ns | 14998917 | 31703250 | +111.37% | SEVERE |
| store_footprint / store-footprint-large-object-500m | route.product-timing.initialization_ns | 460296417 | 1453797916 | +215.84% | SEVERE |
| store_footprint / store-footprint-large-object-500m | route.product-timing.product_call_sum_ns | 491905625 | 1505693374 | +206.09% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100-low-v1 | elapsed_ns | 35869626 | 56222458 | +56.74% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100-low-v1 | route.fs-bench-pro-store-footprint-performance-v5.initialization_ns | 10913417 | 22125958 | +102.74% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100-low-v1 | route.product-timing.exec_ns | 9326500 | 15898708 | +70.47% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100-low-v1 | route.product-timing.initialization_ns | 10913417 | 22125958 | +102.74% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100-low-v1 | route.product-timing.product_call_sum_ns | 35869626 | 56222458 | +56.74% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100000 | elapsed_ns | 4570429959 | 6897713625 | +50.92% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100000 | route.fs-bench-pro-store-footprint-performance-v5.commit_ns | 7513417 | 14637125 | +94.81% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100000 | route.fs-bench-pro-store-footprint-performance-v5.complete_ns | 5264204250 | 7425311500 | +41.05% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100000 | route.fs-bench-pro-store-footprint-performance-v5.initialization_ns | 4515861292 | 6790337667 | +50.37% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100000 | route.product-timing.commit_and_visibility_ns | 7513417 | 14637125 | +94.81% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100000 | route.product-timing.exec_ns | 28308833 | 71585458 | +152.87% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100000 | route.product-timing.initialization_ns | 4515861292 | 6790337667 | +50.37% | SEVERE |
| store_footprint / store-footprint-metadata-cardinality-100000 | route.product-timing.product_call_sum_ns | 4570429959 | 6897713625 | +50.92% | SEVERE |
| store_footprint / store-footprint-unique-100-low-v1 | elapsed_ns | 31947543 | 47905166 | +49.95% | SEVERE |
| store_footprint / store-footprint-unique-100-low-v1 | route.fs-bench-pro-store-footprint-performance-v5.initialization_ns | 9387459 | 20124125 | +114.37% | SEVERE |
| store_footprint / store-footprint-unique-100-low-v1 | route.product-timing.initialization_ns | 9387459 | 20124125 | +114.37% | SEVERE |
| store_footprint / store-footprint-unique-100-low-v1 | route.product-timing.product_call_sum_ns | 31947543 | 47905166 | +49.95% | SEVERE |
| store_footprint / store-footprint-unique-100000 | elapsed_ns | 2982459252 | 4082417459 | +36.88% | SEVERE |
| store_footprint / store-footprint-unique-100000 | route.fs-bench-pro-store-footprint-performance-v5.initialization_ns | 2909436042 | 4004038625 | +37.62% | SEVERE |
| store_footprint / store-footprint-unique-100000 | route.fs-bench-pro-store-footprint-performance-v5.process_user_cpu_ns | 2579186583 | 3921724459 | +52.05% | SEVERE |
| store_footprint / store-footprint-unique-100000 | route.product-timing.initialization_ns | 2909436042 | 4004038625 | +37.62% | SEVERE |
| store_footprint / store-footprint-unique-100000 | route.product-timing.product_call_sum_ns | 2982459252 | 4082417459 | +36.88% | SEVERE |
| tiny_file_churn / tiny-bulk-create-1-compact-v2 | elapsed_ns | 96728291 | 107711042 | +11.35% | OBSERVED_INCREASE |
| tiny_file_churn / tiny-bulk-create-10-compact-v2 | commit_ns | 43041917 | 79685000 | +85.13% | SEVERE |
| tiny_file_churn / tiny-bulk-create-10-compact-v2 | elapsed_ns | 295897791 | 354670542 | +19.86% | REVIEW |
| tiny_file_churn / tiny-bulk-create-10-compact-v2 | host_cpu_ns | 76582500 | 144404292 | +88.56% | SEVERE |
| tiny_file_churn / tiny-bulk-create-100-mixed-v3 | elapsed_ns | 989887791 | 1139657665 | +15.13% | REVIEW |
| tiny_file_churn / tiny-bulk-create-500-mixed-v3 | elapsed_ns | 5054054042 | 5319529625 | +5.25% | OBSERVED_INCREASE |
| tiny_file_churn / tiny-bulk-delete-1-compact-v2 | elapsed_ns | 93346584 | 111209083 | +19.14% | REVIEW |
| tiny_file_churn / tiny-bulk-delete-10-compact-v2 | elapsed_ns | 168254959 | 188448499 | +12.00% | OBSERVED_INCREASE |
| tiny_file_churn / tiny-bulk-delete-100-mixed-v3 | elapsed_ns | 259278875 | 320917959 | +23.77% | REVIEW |
| tiny_file_churn / tiny-bulk-delete-100-mixed-v3 | host_cpu_ns | 70523124 | 122858250 | +74.21% | SEVERE |
| tiny_file_churn / tiny-bulk-delete-500-mixed-v3 | elapsed_ns | 933000457 | 1143208791 | +22.53% | REVIEW |
| tiny_file_churn / tiny-bulk-delete-500-mixed-v3 | host_cpu_ns | 199167958 | 423084625 | +112.43% | SEVERE |
| tiny_file_churn / tiny-create-1-compact-v2 | elapsed_ns | 19053709 | 23598085 | +23.85% | REVIEW |
| tiny_file_churn / tiny-create-10-compact-v2 | elapsed_ns | 26711667 | 33638917 | +25.93% | REVIEW |
| tiny_file_churn / tiny-create-100-mixed-v4 | commit_ns | 7752541 | 15282792 | +97.13% | SEVERE |
| tiny_file_churn / tiny-create-100-mixed-v4 | elapsed_ns | 53616291 | 63617626 | +18.65% | REVIEW |
| tiny_file_churn / tiny-create-500-mixed-v4 | commit_ns | 24284125 | 58224000 | +139.76% | SEVERE |
| tiny_file_churn / tiny-create-500-mixed-v4 | elapsed_ns | 201739625 | 231759166 | +14.88% | OBSERVED_INCREASE |
| tiny_file_churn / tiny-create-500-mixed-v4 | host_cpu_ns | 41676334 | 166864874 | +300.38% | SEVERE |
| tiny_file_churn / tiny-stat-1-compact-v2 | elapsed_ns | 15833917 | 19920041 | +25.81% | REVIEW |
| tiny_file_churn / tiny-stat-10-compact-v2 | elapsed_ns | 21194250 | 25328083 | +19.50% | REVIEW |
| tiny_file_churn / tiny-stat-100-mixed-v4 | elapsed_ns | 37810208 | 60026834 | +58.76% | SEVERE |
| tiny_file_churn / tiny-stat-100-mixed-v4 | exec_ns | 26470708 | 45986667 | +73.73% | SEVERE |
| tiny_file_churn / tiny-stat-100-mixed-v4 | route.sample-complete.host_orchestration_ns | 39327416 | 62092625 | +57.89% | SEVERE |
| tiny_file_churn / tiny-stat-100-mixed-v4 | route.sample-complete.pure_call_sum_ns | 37810208 | 60026834 | +58.76% | SEVERE |
| tiny_file_churn / tiny-stat-500-mixed-v4 | elapsed_ns | 55349375 | 94356709 | +70.47% | SEVERE |
| tiny_file_churn / tiny-stat-500-mixed-v4 | exec_ns | 42781125 | 77926417 | +82.15% | SEVERE |
| tiny_file_churn / tiny-stat-500-mixed-v4 | host_cpu_ns | 35523000 | 126985542 | +257.47% | SEVERE |
| tiny_file_churn / tiny-stat-500-mixed-v4 | route.sample-complete.host_orchestration_ns | 57017459 | 96518167 | +69.28% | SEVERE |
| tiny_file_churn / tiny-stat-500-mixed-v4 | route.sample-complete.pure_call_sum_ns | 55349375 | 94356709 | +70.47% | SEVERE |
| tiny_file_churn / tiny-unlink-1-compact-v2 | create_ns | 6615458 | 11859708 | +79.27% | SEVERE |
| tiny_file_churn / tiny-unlink-1-compact-v2 | elapsed_ns | 17634874 | 32923207 | +86.69% | SEVERE |
| tiny_file_churn / tiny-unlink-1-compact-v2 | exec_ns | 5616500 | 13248875 | +135.89% | SEVERE |
| tiny_file_churn / tiny-unlink-1-compact-v2 | route.sample-complete.host_orchestration_ns | 19280791 | 35246083 | +82.80% | SEVERE |
| tiny_file_churn / tiny-unlink-1-compact-v2 | route.sample-complete.pure_call_sum_ns | 17634874 | 32923207 | +86.69% | SEVERE |
| tiny_file_churn / tiny-unlink-10-compact-v2 | elapsed_ns | 25205291 | 36253667 | +43.83% | SEVERE |
| tiny_file_churn / tiny-unlink-10-compact-v2 | route.sample-complete.host_orchestration_ns | 27177917 | 38544833 | +41.82% | SEVERE |
| tiny_file_churn / tiny-unlink-10-compact-v2 | route.sample-complete.pure_call_sum_ns | 25205291 | 36253667 | +43.83% | SEVERE |
| tiny_file_churn / tiny-unlink-100-mixed-v4 | commit_ns | 5092875 | 12634041 | +148.07% | SEVERE |
| tiny_file_churn / tiny-unlink-100-mixed-v4 | elapsed_ns | 50918082 | 85589541 | +68.09% | SEVERE |
| tiny_file_churn / tiny-unlink-100-mixed-v4 | exec_ns | 35327541 | 60323208 | +70.75% | SEVERE |
| tiny_file_churn / tiny-unlink-100-mixed-v4 | route.sample-complete.host_orchestration_ns | 52647792 | 87829500 | +66.82% | SEVERE |
| tiny_file_churn / tiny-unlink-100-mixed-v4 | route.sample-complete.pure_call_sum_ns | 50918082 | 85589541 | +68.09% | SEVERE |
| tiny_file_churn / tiny-unlink-500-mixed-v4 | commit_ns | 9091667 | 18311000 | +101.40% | SEVERE |
| tiny_file_churn / tiny-unlink-500-mixed-v4 | elapsed_ns | 114278709 | 139472626 | +22.05% | REVIEW |
| tiny_file_churn / tiny-unlink-500-mixed-v4 | host_cpu_ns | 46524458 | 131266208 | +182.14% | SEVERE |
| workspace_change_locality / workspace-clean-commit-1-compact-v2 | elapsed_ns | 11315960 | 12733583 | +12.53% | OBSERVED_INCREASE |
| workspace_change_locality / workspace-clean-commit-10-compact-v2 | elapsed_ns | 10872708 | 12446083 | +14.47% | OBSERVED_INCREASE |
| workspace_change_locality / workspace-clean-commit-100-mixed-v4 | elapsed_ns | 11559208 | 14588749 | +26.21% | REVIEW |
| workspace_change_locality / workspace-clean-commit-500-mixed-v4 | create_ns | 8210666 | 13340500 | +62.48% | SEVERE |
| workspace_change_locality / workspace-clean-commit-500-mixed-v4 | elapsed_ns | 11866166 | 18544000 | +56.28% | SEVERE |
| workspace_change_locality / workspace-clean-commit-500-mixed-v4 | host_cpu_ns | 18101416 | 93222249 | +415.00% | SEVERE |
| workspace_change_locality / workspace-clean-commit-500-mixed-v4 | route.sample-complete.host_orchestration_ns | 13334250 | 20655625 | +54.91% | SEVERE |
| workspace_change_locality / workspace-clean-commit-500-mixed-v4 | route.sample-complete.pure_call_sum_ns | 11866166 | 18544000 | +56.28% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-1-compact-v2 | commit_ns | 10068833 | 29069500 | +188.71% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-1-compact-v2 | elapsed_ns | 84118125 | 144800667 | +72.14% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-1-compact-v2 | exec_ns | 63189792 | 100924500 | +59.72% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-1-compact-v2 | route.sample-complete.host_orchestration_ns | 85727542 | 147002166 | +71.48% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-1-compact-v2 | route.sample-complete.pure_call_sum_ns | 84118125 | 144800667 | +72.14% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-10-compact-v2 | commit_ns | 76749125 | 159861250 | +108.29% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-10-compact-v2 | elapsed_ns | 316616500 | 449219043 | +41.88% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-10-compact-v2 | host_cpu_ns | 128174666 | 286276000 | +123.35% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-10-compact-v2 | route.sample-complete.host_orchestration_ns | 318564875 | 451511125 | +41.73% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-10-compact-v2 | route.sample-complete.pure_call_sum_ns | 316616500 | 449219043 | +41.88% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-100-mixed-v4 | commit_ns | 545124875 | 1083433125 | +98.75% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-100-mixed-v4 | elapsed_ns | 1497618208 | 2260317667 | +50.93% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-100-mixed-v4 | host_cpu_ns | 702154916 | 1552525917 | +121.11% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-100-mixed-v4 | route.sample-complete.host_orchestration_ns | 1500189500 | 2264002500 | +50.91% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-100-mixed-v4 | route.sample-complete.pure_call_sum_ns | 1497618208 | 2260317667 | +50.93% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-500-mixed-v4 | commit_ns | 2626086500 | 3804453709 | +44.87% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-500-mixed-v4 | elapsed_ns | 5688591958 | 7665527084 | +34.75% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-500-mixed-v4 | end_ns | 22404666 | 31233125 | +39.40% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-500-mixed-v4 | host_cpu_ns | 3127638750 | 5345351250 | +70.91% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-500-mixed-v4 | route.sample-complete.host_orchestration_ns | 5691401125 | 7669509084 | +34.76% | SEVERE |
| workspace_change_locality / workspace-dense-rewrite-500-mixed-v4 | route.sample-complete.pure_call_sum_ns | 5688591958 | 7665527084 | +34.75% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-1-compact-v2 | elapsed_ns | 16937333 | 25370624 | +49.79% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-1-compact-v2 | route.sample-complete.host_orchestration_ns | 19266375 | 28304333 | +46.91% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-1-compact-v2 | route.sample-complete.pure_call_sum_ns | 16937333 | 25370624 | +49.79% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-10-compact-v2 | elapsed_ns | 39179873 | 54443417 | +38.96% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-10-compact-v2 | route.sample-complete.host_orchestration_ns | 41792958 | 57662500 | +37.97% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-10-compact-v2 | route.sample-complete.pure_call_sum_ns | 39179873 | 54443417 | +38.96% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-10-compact-v2 | sdk_edit_ns | 23558958 | 32150291 | +36.47% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-100-mixed-v4 | commit_ns | 14688250 | 32120709 | +118.68% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-100-mixed-v4 | elapsed_ns | 336399086 | 430540914 | +27.99% | REVIEW |
| workspace_change_locality / workspace-distributed-sdk-edit-100-mixed-v4 | host_cpu_ns | 149096500 | 236277750 | +58.47% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-500-mixed-v4 | commit_ns | 56073958 | 111815833 | +99.41% | SEVERE |
| workspace_change_locality / workspace-distributed-sdk-edit-500-mixed-v4 | elapsed_ns | 2849182039 | 3473777663 | +21.92% | REVIEW |
| workspace_change_locality / workspace-distributed-sdk-edit-500-mixed-v4 | route.sample-complete.orchestration_unattributed_ns | 17223419 | 34183296 | +98.47% | SEVERE |
| workspace_change_locality / workspace-fixed-move-1-compact-v2 | elapsed_ns | 19708292 | 24738083 | +25.52% | REVIEW |
| workspace_change_locality / workspace-fixed-move-10-compact-v2 | elapsed_ns | 21320084 | 23145542 | +8.56% | OBSERVED_INCREASE |
| workspace_change_locality / workspace-fixed-move-100-mixed-v4 | elapsed_ns | 28352083 | 35468083 | +25.10% | REVIEW |
| workspace_change_locality / workspace-fixed-move-100-mixed-v4 | exec_ns | 12033792 | 18348125 | +52.47% | SEVERE |
| workspace_change_locality / workspace-fixed-move-500-mixed-v4 | elapsed_ns | 27589207 | 36072125 | +30.75% | SEVERE |
| workspace_change_locality / workspace-fixed-move-500-mixed-v4 | exec_ns | 12654666 | 18538542 | +46.50% | SEVERE |
| workspace_change_locality / workspace-fixed-move-500-mixed-v4 | host_cpu_ns | 24628375 | 94617250 | +284.18% | SEVERE |
| workspace_change_locality / workspace-fixed-move-500-mixed-v4 | route.sample-complete.host_orchestration_ns | 29095666 | 37917875 | +30.32% | SEVERE |
| workspace_change_locality / workspace-fixed-move-500-mixed-v4 | route.sample-complete.pure_call_sum_ns | 27589207 | 36072125 | +30.75% | SEVERE |

## Independently scheduled proof preparation

Preparation and verifier wall are disjoint and shown separately. Their sum includes both processes. The verifier preparation scope changed; neither warmed verifier wall nor this sum establishes a historical speedup.

| Case | Preparation (ns) | Proof wall (s) | Disjoint total (s) | Evidence validation |
|---|---:|---:|---:|---|
| namespace-100000 | 29686743625 | 7.098156834006659 | 36.78490045900666 | PASS |
| store-footprint-unique-100000 | 29821026833 | 7.047733291008626 | 36.86876012400863 | PASS |
| store-footprint-metadata-cardinality-100000 | 31846418625 | 11.196098917003837 | 43.042517542003836 | PASS |

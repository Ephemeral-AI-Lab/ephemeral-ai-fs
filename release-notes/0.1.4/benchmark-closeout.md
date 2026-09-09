# v0.1.4 benchmark closeout

**Work stopping point accepted; no further optimization or benchmark rerun requested.** This report uses the existing issue #98 terminal campaign, qualified at `9cfb4be477116646258ea0621280ed13b1824c6d`, with immutable evidence at `856baab0caf0522db4757cb4dcbfb0d9c43e30d4`. The owner explicitly authorized merge, tag and release publication using these results. The [GitHub release](https://github.com/Ephemeral-AI-Lab/layerfs/releases/tag/v0.1.4) is the authoritative publication record.

## Coverage and limits

- **198/198 performance executions passed; 226 routine proofs passed.** One declared optional 600-second proof was not run.
- **411 native tests**, the separately activated ignored spill test, doctests, formatting and Clippy passed; GitHub CI passed.
- Full157: **157 performance states, 157 historical proofs, 158 checkpoint/accounting records and cleanup passed**.
- Full157 allocation: **184,582,144 B**, identical to the prior candidate and **15.374% below** the 218,116,096 B supplemental storage control. This control is not the published v0.1.3 timing baseline.
- Current elapsed classifications: **49 SEVERE, 95 REVIEW, 31 OBSERVED_INCREASE, 19 NO_INCREASE, 4 INELIGIBLE**. Three absolute latency targets remain missed.
- The unmodified comparison report remains **INCOMPLETE** for four Git fixtures whose historical identity incorporates the image. Execution success is not a performance-threshold pass. Owner acceptance does not reclassify historical outcomes.

Each performance case has one sample. Family sums below are descriptive sums of case timers, not campaign wall time, throughput, paired estimates or latency distributions. Families can contain different operation timers. Historical family comparisons are shown only when every case is eligible. CPU/RSS scopes and full identities remain in the CSV copies (values unchanged; line endings normalized to LF).

## Each family

| Family | Performance passes | Routine proof passes | Optional unrun | Published baseline sum (s) | Candidate sum (s) | Change |
|---|---:|---:|---:|---:|---:|---:|
| payload_create_read | 8 | 8 | 0 | 4.179502 | 3.872964 | -7.33% |
| dedup_workspace_reuse | 14 | 14 | 0 | 15.874997 | 14.074111 | -11.34% |
| dedup_cross_file | 10 | 10 | 0 | 1.402381 | 3.154912 | +124.97% |
| dedup_cdc_locality | 20 | 21 | 0 | 1.484589 | 3.187850 | +114.73% |
| edit_length_preserving | 12 | 12 | 0 | 0.082806 | 0.119696 | +44.55% |
| edit_length_changing | 32 | 32 | 0 | 0.230456 | 0.306447 | +32.97% |
| edit_canonical_chunk_count | 12 | 12 | 0 | 0.092536 | 0.135438 | +46.36% |
| init_namespace | 4 | 4 | 0 | 3.053609 | 4.903439 | +60.58% |
| store_footprint | 6 | 6 | 0 | 8.156506 | 12.273350 | +50.47% |
| tiny_file_churn | 20 | 20 | 0 | 8.529795 | 8.996516 | +5.47% |
| namespace_mutation | 4 | 4 | 0 | 0.314367 | 0.429782 | +36.71% |
| directory_construction_traversal | 12 | 12 | 0 | 7.628812 | 9.710653 | +27.29% |
| workspace_change_locality | 16 | 16 | 0 | 10.971227 | 13.240867 | +20.69% |
| dedup_branch_history | 20 | 20 | 0 | 47.049815 | 55.856741 | +18.72% |
| git_tool_workflow | 4 | 4 | 0 | — | 8.877153 | — |
| mixed_load_bearing | 4 | 4 | 0 | 8.559964 | 8.592193 | +0.38% |
| workspace_reliability | 0 | 27 | 1 | — | — | — |

## Each performance case

All rows below executed successfully and have passing routine verification. Historical comparisons use the published v0.1.3 elapsed value only when eligible. A “MISS” target remains a miss despite successful execution.

### payload_create_read

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| payload-create-1m-compact-v2 | pure_call_sum_ns | 28.837 | 45.630 | +58.23% | SEVERE | PASS |
| payload-create-10m-compact-v2 | pure_call_sum_ns | 91.556 | 94.836 | +3.58% | OBSERVED_INCREASE | PASS |
| payload-create-100m | pure_call_sum_ns | 682.771 | 606.224 | -11.21% | NO_INCREASE | PASS |
| payload-create-500m | pure_call_sum_ns | 3068.250 | 2758.559 | -10.09% | NO_INCREASE | PASS |
| payload-random-read-1-compact-v2 | pure_call_sum_ns | 14.585 | 14.896 | +2.14% | OBSERVED_INCREASE | PASS |
| payload-random-read-10-compact-v2 | pure_call_sum_ns | 17.420 | 21.713 | +24.64% | REVIEW | PASS |
| payload-random-read-100 | pure_call_sum_ns | 56.396 | 81.285 | +44.13% | SEVERE | PASS |
| payload-random-read-500 | pure_call_sum_ns | 219.687 | 249.821 | +13.72% | OBSERVED_INCREASE | PASS |

### dedup_workspace_reuse

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| dedup-workspace-exact-1-compact-v2 | pure_call_sum_ns | 31.563 | 35.442 | +12.29% | OBSERVED_INCREASE | PASS |
| dedup-workspace-exact-10-compact-v2 | pure_call_sum_ns | 109.325 | 91.429 | -16.37% | NO_INCREASE | PASS |
| dedup-workspace-exact-100 | pure_call_sum_ns | 840.528 | 663.691 | -21.04% | NO_INCREASE | PASS |
| dedup-workspace-exact-500 | pure_call_sum_ns | 4296.598 | 3928.647 | -8.56% | NO_INCREASE | PASS |
| dedup-workspace-local-1-compact-v2 | pure_call_sum_ns | 30.610 | 34.221 | +11.80% | OBSERVED_INCREASE | PASS |
| dedup-workspace-local-10-compact-v2 | pure_call_sum_ns | 103.153 | 98.345 | -4.66% | NO_INCREASE | PASS |
| dedup-workspace-local-100 | pure_call_sum_ns | 808.387 | 694.966 | -14.03% | NO_INCREASE | PASS |
| dedup-workspace-local-500 | pure_call_sum_ns | 4307.360 | 3699.743 | -14.11% | NO_INCREASE | PASS |
| dedup-workspace-unique-1-compact-v2 | pure_call_sum_ns | 27.475 | 33.818 | +23.09% | REVIEW | PASS |
| dedup-workspace-unique-10-compact-v2 | pure_call_sum_ns | 95.669 | 104.697 | +9.44% | OBSERVED_INCREASE | PASS |
| dedup-workspace-unique-100 | pure_call_sum_ns | 768.004 | 772.099 | +0.53% | OBSERVED_INCREASE | PASS |
| dedup-workspace-unique-500 | pure_call_sum_ns | 4319.934 | 3772.128 | -12.68% | NO_INCREASE | PASS |
| dedup-workspace-unique-1-base128-v3 | pure_call_sum_ns | 33.382 | 37.367 | +11.94% | OBSERVED_INCREASE | PASS |
| dedup-workspace-unique-10-base128-v3 | pure_call_sum_ns | 103.007 | 107.519 | +4.38% | OBSERVED_INCREASE | PASS |

### dedup_cross_file

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| dedup-cross-file-anchor-1 | pure_call_sum_ns | 3.808 | 6.356 | +66.90% | REVIEW | PASS |
| dedup-cross-file-unique-10 | pure_call_sum_ns | 22.390 | 33.418 | +49.26% | SEVERE | PASS |
| dedup-cross-file-unique-100 | pure_call_sum_ns | 93.267 | 283.365 | +203.82% | SEVERE | PASS |
| dedup-cross-file-unique-500 | pure_call_sum_ns | 444.949 | 1221.996 | +174.64% | SEVERE | PASS |
| dedup-cross-file-identical-10 | pure_call_sum_ns | 18.395 | 18.012 | -2.09% | NO_INCREASE | PASS |
| dedup-cross-file-identical-100 | pure_call_sum_ns | 28.960 | 36.683 | +26.67% | REVIEW | PASS |
| dedup-cross-file-identical-500 | pure_call_sum_ns | 110.630 | 117.390 | +6.11% | OBSERVED_INCREASE | PASS |
| dedup-cross-file-mixed-10 | pure_call_sum_ns | 24.046 | 29.416 | +22.33% | REVIEW | PASS |
| dedup-cross-file-mixed-100 | pure_call_sum_ns | 113.944 | 259.898 | +128.09% | SEVERE | PASS |
| dedup-cross-file-mixed-500 | pure_call_sum_ns | 541.992 | 1148.377 | +111.88% | SEVERE | PASS |

### dedup_cdc_locality

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| dedup-cdc-overwrite-1 | pure_call_sum_ns | 4.002 | 6.976 | +74.29% | REVIEW | PASS |
| dedup-cdc-overwrite-10 | pure_call_sum_ns | 20.448 | 17.761 | -13.14% | NO_INCREASE | PASS |
| dedup-cdc-overwrite-100 | pure_call_sum_ns | 30.953 | 42.643 | +37.77% | SEVERE | PASS |
| dedup-cdc-overwrite-500 | pure_call_sum_ns | 122.035 | 156.606 | +28.33% | REVIEW | PASS |
| dedup-cdc-insert-1 | pure_call_sum_ns | 3.897 | 6.173 | +58.39% | REVIEW | PASS |
| dedup-cdc-insert-10 | pure_call_sum_ns | 18.058 | 16.420 | -9.07% | NO_INCREASE | PASS |
| dedup-cdc-insert-100 | pure_call_sum_ns | 34.183 | 46.772 | +36.83% | SEVERE | PASS |
| dedup-cdc-insert-500 | pure_call_sum_ns | 136.776 | 178.864 | +30.77% | REVIEW | PASS |
| dedup-cdc-delete-1 | pure_call_sum_ns | 4.412 | 6.565 | +48.80% | REVIEW | PASS |
| dedup-cdc-delete-10 | pure_call_sum_ns | 18.225 | 19.333 | +6.08% | OBSERVED_INCREASE | PASS |
| dedup-cdc-delete-100 | pure_call_sum_ns | 33.166 | 47.801 | +44.13% | SEVERE | PASS |
| dedup-cdc-delete-500 | pure_call_sum_ns | 130.864 | 179.825 | +37.41% | REVIEW | PASS |
| dedup-cdc-common-body-1 | pure_call_sum_ns | 4.125 | 6.506 | +57.71% | REVIEW | PASS |
| dedup-cdc-common-body-10 | pure_call_sum_ns | 20.395 | 22.009 | +7.91% | OBSERVED_INCREASE | PASS |
| dedup-cdc-common-body-100 | pure_call_sum_ns | 58.208 | 114.210 | +96.21% | SEVERE | PASS |
| dedup-cdc-common-body-500 | pure_call_sum_ns | 244.350 | 516.631 | +111.43% | SEVERE | PASS |
| dedup-cdc-scattered-1 | pure_call_sum_ns | 4.647 | 9.010 | +93.88% | REVIEW | PASS |
| dedup-cdc-scattered-10 | pure_call_sum_ns | 23.469 | 46.855 | +99.65% | SEVERE | PASS |
| dedup-cdc-scattered-100 | pure_call_sum_ns | 89.368 | 284.406 | +218.24% | SEVERE | PASS |
| dedup-cdc-scattered-500 | pure_call_sum_ns | 483.007 | 1462.484 | +202.79% | SEVERE | PASS |

### edit_length_preserving

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| overwrite-head-4k-on-1mib-ops-1 | edit_commit_ns | 5.777 | 9.136 | +58.15% | REVIEW | PASS |
| overwrite-head-4k-on-10mib-ops-1 | edit_commit_ns | 6.154 | 12.845 | +108.71% | SEVERE | PASS |
| overwrite-head-4k-on-100mib-ops-1 | edit_commit_ns | 6.652 | 11.562 | +73.82% | REVIEW | PASS |
| overwrite-head-4k-on-500mib-ops-1 | edit_commit_ns | 8.411 | 11.164 | +32.73% | REVIEW | PASS |
| overwrite-middle-4k-on-1mib-ops-1 | edit_commit_ns | 6.139 | 7.755 | +26.33% | REVIEW | PASS |
| overwrite-middle-4k-on-10mib-ops-1 | edit_commit_ns | 5.925 | 9.631 | +62.55% | REVIEW | PASS |
| overwrite-middle-4k-on-100mib-ops-1 | edit_commit_ns | 6.188 | 10.646 | +72.04% | REVIEW | PASS |
| overwrite-middle-4k-on-500mib-ops-1 | edit_commit_ns | 7.174 | 11.009 | +53.45% | REVIEW | PASS |
| overwrite-tail-4k-on-1mib-ops-1 | edit_commit_ns | 6.112 | 7.579 | +24.00% | REVIEW | PASS |
| overwrite-tail-4k-on-10mib-ops-1 | edit_commit_ns | 7.265 | 8.989 | +23.73% | REVIEW | PASS |
| overwrite-tail-4k-on-100mib-ops-1 | edit_commit_ns | 7.631 | 8.793 | +15.23% | REVIEW | PASS |
| overwrite-tail-4k-on-500mib-ops-1 | edit_commit_ns | 9.378 | 10.586 | +12.89% | OBSERVED_INCREASE | PASS |

### edit_length_changing

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| insert-middle-4k-on-1mib-ops-1 | edit_commit_ns | 5.590 | 9.873 | +76.62% | REVIEW | PASS |
| insert-middle-4k-on-10mib-ops-1 | edit_commit_ns | 7.396 | 9.979 | +34.92% | REVIEW | PASS |
| insert-middle-4k-on-100mib-ops-1 | edit_commit_ns | 12.439 | 9.871 | -20.65% | NO_INCREASE | PASS |
| insert-middle-4k-on-500mib-result-capped-v2-ops-1 | edit_commit_ns | 18.011 | 12.187 | -32.34% | NO_INCREASE | PASS |
| delete-middle-4k-on-1mib-ops-1 | edit_commit_ns | 5.290 | 8.084 | +52.81% | REVIEW | PASS |
| delete-middle-4k-on-10mib-ops-1 | edit_commit_ns | 6.570 | 9.308 | +41.67% | REVIEW | PASS |
| delete-middle-4k-on-100mib-ops-1 | edit_commit_ns | 6.758 | 10.370 | +53.45% | REVIEW | PASS |
| delete-middle-4k-on-500mib-ops-1 | edit_commit_ns | 7.488 | 12.893 | +72.19% | SEVERE | PASS |
| append-tail-4k-on-1mib-ops-1 | edit_commit_ns | 6.261 | 9.518 | +52.01% | REVIEW | PASS |
| append-tail-4k-on-10mib-ops-1 | edit_commit_ns | 5.905 | 7.851 | +32.95% | REVIEW | PASS |
| append-tail-4k-on-100mib-ops-1 | edit_commit_ns | 6.255 | 9.157 | +46.40% | REVIEW | PASS |
| append-tail-4k-on-500mib-result-capped-v2-ops-1 | edit_commit_ns | 6.686 | 9.451 | +41.35% | REVIEW | PASS |
| prepend-head-4k-on-1mib-ops-1 | edit_commit_ns | 5.905 | 8.615 | +45.88% | REVIEW | PASS |
| prepend-head-4k-on-10mib-ops-1 | edit_commit_ns | 5.672 | 9.615 | +69.53% | REVIEW | PASS |
| prepend-head-4k-on-100mib-ops-1 | edit_commit_ns | 6.269 | 9.100 | +45.14% | REVIEW | PASS |
| prepend-head-4k-on-500mib-result-capped-v2-ops-1 | edit_commit_ns | 7.099 | 9.979 | +40.58% | REVIEW | PASS |
| replace-grow-middle-2k-to-4k-on-1mib-ops-1 | edit_commit_ns | 5.744 | 8.422 | +46.62% | REVIEW | PASS |
| replace-grow-middle-2k-to-4k-on-10mib-ops-1 | edit_commit_ns | 6.463 | 9.059 | +40.17% | REVIEW | PASS |
| replace-grow-middle-2k-to-4k-on-100mib-ops-1 | edit_commit_ns | 6.382 | 10.347 | +62.14% | REVIEW | PASS |
| replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1 | edit_commit_ns | 7.922 | 11.585 | +46.24% | REVIEW | PASS |
| replace-shrink-middle-4k-to-2k-on-1mib-ops-1 | edit_commit_ns | 5.880 | 9.896 | +68.30% | REVIEW | PASS |
| replace-shrink-middle-4k-to-2k-on-10mib-ops-1 | edit_commit_ns | 5.770 | 9.248 | +60.28% | REVIEW | PASS |
| replace-shrink-middle-4k-to-2k-on-100mib-ops-1 | edit_commit_ns | 7.008 | 9.705 | +38.50% | REVIEW | PASS |
| replace-shrink-middle-4k-to-2k-on-500mib-ops-1 | edit_commit_ns | 6.997 | 11.142 | +59.25% | REVIEW | PASS |
| truncate-tail-4k-on-1mib-ops-1 | edit_commit_ns | 5.293 | 8.637 | +63.16% | REVIEW | PASS |
| truncate-tail-4k-on-10mib-ops-1 | edit_commit_ns | 15.641 | 7.610 | -51.35% | NO_INCREASE | PASS |
| truncate-tail-4k-on-100mib-ops-1 | edit_commit_ns | 6.528 | 8.850 | +35.57% | REVIEW | PASS |
| truncate-tail-4k-on-500mib-ops-1 | edit_commit_ns | 6.895 | 10.315 | +49.59% | REVIEW | PASS |
| zero-extend-tail-4k-on-1mib-ops-1 | edit_commit_ns | 5.633 | 8.599 | +52.65% | REVIEW | PASS |
| zero-extend-tail-4k-on-10mib-ops-1 | edit_commit_ns | 5.638 | 8.378 | +48.60% | REVIEW | PASS |
| zero-extend-tail-4k-on-100mib-ops-1 | edit_commit_ns | 6.404 | 9.285 | +44.99% | REVIEW | PASS |
| zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1 | edit_commit_ns | 6.664 | 9.520 | +42.86% | REVIEW | PASS |

### edit_canonical_chunk_count

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1 | edit_commit_ns | 6.855 | 11.624 | +69.58% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1 | edit_commit_ns | 8.507 | 10.647 | +25.16% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1 | edit_commit_ns | 7.335 | 11.843 | +61.45% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1 | edit_commit_ns | 8.270 | 12.831 | +55.15% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1 | edit_commit_ns | 7.852 | 10.789 | +37.40% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1 | edit_commit_ns | 7.516 | 11.735 | +56.13% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1 | edit_commit_ns | 7.870 | 12.887 | +63.76% | SEVERE | PASS |
| overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1 | edit_commit_ns | 8.466 | 12.639 | +49.29% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1 | edit_commit_ns | 7.175 | 8.818 | +22.89% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1 | edit_commit_ns | 7.013 | 9.288 | +32.43% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1 | edit_commit_ns | 7.582 | 10.804 | +42.50% | REVIEW | PASS |
| overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1 | edit_commit_ns | 8.095 | 11.534 | +42.48% | REVIEW | PASS |

### init_namespace

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| namespace-100-compact-v3 | layerstack_init_ns | 8.902 | 19.965 | +124.28% | SEVERE | PASS |
| namespace-1000-compact-v3 | layerstack_init_ns | 38.077 | 81.724 | +114.63% | SEVERE | PASS |
| namespace-10000 | layerstack_init_ns | 403.468 | 939.236 | +132.79% | SEVERE | PASS |
| namespace-100000 | layerstack_init_ns | 2603.162 | 3862.514 | +48.38% | SEVERE | PASS |

### store_footprint

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| store-footprint-unique-100000 | product_call_sum_ns | 2982.459 | 3949.145 | +32.41% | SEVERE | PASS |
| store-footprint-metadata-cardinality-100000 | product_call_sum_ns | 4570.430 | 6910.795 | +51.21% | SEVERE | PASS |
| store-footprint-large-object-500m | product_call_sum_ns | 491.906 | 1258.095 | +155.76% | SEVERE | PASS |
| store-footprint-unique-100-low-v1 | product_call_sum_ns | 31.948 | 48.546 | +51.96% | SEVERE | PASS |
| store-footprint-metadata-cardinality-100-low-v1 | product_call_sum_ns | 35.870 | 54.944 | +53.18% | SEVERE | PASS |
| store-footprint-large-object-10m-low-v1 | product_call_sum_ns | 43.894 | 51.826 | +18.07% | REVIEW | PASS |

### tiny_file_churn

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| tiny-create-1-compact-v2 | pure_call_sum_ns | 19.054 | 22.766 | +19.48% | REVIEW | PASS |
| tiny-create-10-compact-v2 | pure_call_sum_ns | 26.712 | 31.788 | +19.00% | REVIEW | PASS |
| tiny-create-100-mixed-v4 | pure_call_sum_ns | 53.616 | 68.318 | +27.42% | REVIEW | PASS |
| tiny-create-500-mixed-v4 | pure_call_sum_ns | 201.740 | 224.958 | +11.51% | OBSERVED_INCREASE | PASS |
| tiny-stat-1-compact-v2 | pure_call_sum_ns | 15.834 | 19.672 | +24.24% | REVIEW | PASS |
| tiny-stat-10-compact-v2 | pure_call_sum_ns | 21.194 | 25.060 | +18.24% | REVIEW | PASS |
| tiny-stat-100-mixed-v4 | pure_call_sum_ns | 37.810 | 61.445 | +62.51% | SEVERE | PASS |
| tiny-stat-500-mixed-v4 | pure_call_sum_ns | 55.349 | 84.688 | +53.01% | SEVERE | PASS |
| tiny-unlink-1-compact-v2 | pure_call_sum_ns | 17.635 | 23.583 | +33.73% | SEVERE | PASS |
| tiny-unlink-10-compact-v2 | pure_call_sum_ns | 25.205 | 33.432 | +32.64% | SEVERE | PASS |
| tiny-unlink-100-mixed-v4 | pure_call_sum_ns | 50.918 | 82.583 | +62.19% | SEVERE | PASS |
| tiny-unlink-500-mixed-v4 | pure_call_sum_ns | 114.279 | 141.657 | +23.96% | REVIEW | PASS |
| tiny-bulk-create-1-compact-v2 | pure_call_sum_ns | 96.728 | 73.225 | -24.30% | NO_INCREASE | PASS |
| tiny-bulk-create-10-compact-v2 | pure_call_sum_ns | 295.898 | 329.992 | +11.52% | OBSERVED_INCREASE | PASS |
| tiny-bulk-create-100-mixed-v3 | pure_call_sum_ns | 989.888 | 1056.367 | +6.72% | OBSERVED_INCREASE | PASS |
| tiny-bulk-create-500-mixed-v3 | pure_call_sum_ns | 5054.054 | 5049.879 | -0.08% | NO_INCREASE | PASS |
| tiny-bulk-delete-1-compact-v2 | pure_call_sum_ns | 93.347 | 106.834 | +14.45% | OBSERVED_INCREASE | PASS |
| tiny-bulk-delete-10-compact-v2 | pure_call_sum_ns | 168.255 | 169.562 | +0.78% | OBSERVED_INCREASE | PASS |
| tiny-bulk-delete-100-mixed-v3 | pure_call_sum_ns | 259.279 | 298.070 | +14.96% | OBSERVED_INCREASE | PASS |
| tiny-bulk-delete-500-mixed-v3 | pure_call_sum_ns | 933.000 | 1092.636 | +17.11% | REVIEW | PASS |

### namespace_mutation

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| namespace-subtree-relocate-delete-1-compact-v2 | pure_call_sum_ns | 21.311 | 29.182 | +36.93% | SEVERE | PASS |
| namespace-subtree-relocate-delete-10-compact-v2 | pure_call_sum_ns | 59.145 | 70.781 | +19.67% | REVIEW | PASS |
| namespace-subtree-relocate-delete-100-mixed-v4 | pure_call_sum_ns | 51.018 | 77.177 | +51.27% | SEVERE | PASS |
| namespace-subtree-relocate-delete-500-mixed-v4 | pure_call_sum_ns | 182.894 | 252.642 | +38.14% | SEVERE | PASS |

### directory_construction_traversal

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| directory-construct-1-compact-v2 | pure_call_sum_ns | 17.022 | 23.119 | +35.82% | SEVERE | PASS |
| directory-construct-10-compact-v2 | pure_call_sum_ns | 39.347 | 46.970 | +19.37% | REVIEW | PASS |
| directory-construct-100-mixed-v4 | pure_call_sum_ns | 216.060 | 228.079 | +5.56% | OBSERVED_INCREASE | PASS |
| directory-construct-500-mixed-v4 | pure_call_sum_ns | 1030.581 | 1084.095 | +5.19% | OBSERVED_INCREASE | PASS |
| directory-metadata-scan-1-compact-v2 | pure_call_sum_ns | 70.527 | 84.228 | +19.43% | REVIEW | PASS |
| directory-metadata-scan-10-compact-v2 | pure_call_sum_ns | 100.739 | 112.910 | +12.08% | OBSERVED_INCREASE | PASS |
| directory-metadata-scan-100-mixed-v4 | pure_call_sum_ns | 244.239 | 304.099 | +24.51% | REVIEW | PASS |
| directory-metadata-scan-500-mixed-v4 | pure_call_sum_ns | 504.932 | 640.298 | +26.81% | REVIEW | PASS |
| directory-content-scan-1-compact-v2 | pure_call_sum_ns | 84.760 | 119.580 | +41.08% | SEVERE | PASS |
| directory-content-scan-10-compact-v2 | pure_call_sum_ns | 308.870 | 390.459 | +26.42% | REVIEW | PASS |
| directory-content-scan-100-mixed-v4 | pure_call_sum_ns | 1105.141 | 1424.410 | +28.89% | REVIEW | PASS |
| directory-content-scan-500-mixed-v4 | pure_call_sum_ns | 3906.595 | 5252.405 | +34.45% | SEVERE | PASS |

### workspace_change_locality

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| workspace-clean-commit-1-compact-v2 | pure_call_sum_ns | 11.316 | 13.871 | +22.58% | REVIEW | PASS |
| workspace-clean-commit-10-compact-v2 | pure_call_sum_ns | 10.873 | 16.208 | +49.07% | SEVERE | PASS |
| workspace-clean-commit-100-mixed-v4 | pure_call_sum_ns | 11.559 | 13.168 | +13.92% | OBSERVED_INCREASE | PASS |
| workspace-clean-commit-500-mixed-v4 | pure_call_sum_ns | 11.866 | 13.494 | +13.72% | OBSERVED_INCREASE | PASS |
| workspace-fixed-move-1-compact-v2 | pure_call_sum_ns | 19.708 | 22.848 | +15.93% | REVIEW | PASS |
| workspace-fixed-move-10-compact-v2 | pure_call_sum_ns | 21.320 | 24.470 | +14.78% | OBSERVED_INCREASE | PASS |
| workspace-fixed-move-100-mixed-v4 | pure_call_sum_ns | 28.352 | 37.965 | +33.91% | SEVERE | PASS |
| workspace-fixed-move-500-mixed-v4 | pure_call_sum_ns | 27.589 | 39.344 | +42.61% | SEVERE | PASS |
| workspace-distributed-sdk-edit-1-compact-v2 | pure_call_sum_ns | 16.937 | 20.263 | +19.64% | REVIEW | PASS |
| workspace-distributed-sdk-edit-10-compact-v2 | pure_call_sum_ns | 39.180 | 44.330 | +13.14% | OBSERVED_INCREASE | PASS |
| workspace-distributed-sdk-edit-100-mixed-v4 | pure_call_sum_ns | 336.399 | 412.087 | +22.50% | REVIEW | PASS |
| workspace-distributed-sdk-edit-500-mixed-v4 | pure_call_sum_ns | 2849.182 | 2947.457 | +3.45% | OBSERVED_INCREASE | PASS |
| workspace-dense-rewrite-1-compact-v2 | pure_call_sum_ns | 84.118 | 117.866 | +40.12% | SEVERE | PASS |
| workspace-dense-rewrite-10-compact-v2 | pure_call_sum_ns | 316.616 | 393.220 | +24.19% | REVIEW | PASS |
| workspace-dense-rewrite-100-mixed-v4 | pure_call_sum_ns | 1497.618 | 1935.727 | +29.25% | REVIEW | PASS |
| workspace-dense-rewrite-500-mixed-v4 | pure_call_sum_ns | 5688.592 | 7188.546 | +26.37% | REVIEW | PASS |

### dedup_branch_history

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| dedup-history-distributed-1 | pure_call_sum_ns | 22.163 | 35.024 | +58.03% | SEVERE | PASS |
| dedup-history-distributed-10 | pure_call_sum_ns | 82.266 | 97.468 | +18.48% | REVIEW | PASS |
| dedup-history-distributed-100 | pure_call_sum_ns | 586.405 | 679.296 | +15.84% | REVIEW | PASS |
| dedup-history-distributed-500 | pure_call_sum_ns | 2925.218 | 3404.998 | +16.40% | REVIEW | PASS |
| dedup-history-hotset-1 | pure_call_sum_ns | 23.580 | 32.683 | +38.60% | SEVERE | PASS |
| dedup-history-hotset-10 | pure_call_sum_ns | 118.461 | 135.254 | +14.18% | OBSERVED_INCREASE | PASS |
| dedup-history-hotset-100 | pure_call_sum_ns | 799.500 | 926.892 | +15.93% | REVIEW | PASS |
| dedup-history-hotset-500 | pure_call_sum_ns | 3683.755 | 3844.578 | +4.37% | OBSERVED_INCREASE | PASS |
| dedup-history-recurring-1 | pure_call_sum_ns | 21.456 | 29.096 | +35.61% | SEVERE | PASS |
| dedup-history-recurring-10 | pure_call_sum_ns | 62.918 | 89.001 | +41.46% | SEVERE | PASS |
| dedup-history-recurring-100 | pure_call_sum_ns | 462.841 | 600.233 | +29.68% | REVIEW | PASS |
| dedup-history-recurring-500 | pure_call_sum_ns | 2145.416 | 2878.276 | +34.16% | SEVERE | PASS |
| dedup-history-metadata-1 | pure_call_sum_ns | 20.653 | 30.097 | +45.73% | SEVERE | PASS |
| dedup-history-metadata-10 | pure_call_sum_ns | 76.474 | 97.801 | +27.89% | REVIEW | PASS |
| dedup-history-metadata-100 | pure_call_sum_ns | 649.605 | 751.688 | +15.71% | REVIEW | PASS |
| dedup-history-metadata-500 | pure_call_sum_ns | 3231.967 | 3481.995 | +7.74% | OBSERVED_INCREASE | PASS |
| dedup-history-unrelated-1 | pure_call_sum_ns | 880.164 | 1033.286 | +17.40% | REVIEW | PASS |
| dedup-history-unrelated-10 | pure_call_sum_ns | 9543.475 | 9477.130 | -0.70% | NO_INCREASE | PASS |
| dedup-history-unrelated-100-mixed-v2 | pure_call_sum_ns | 3549.609 | 4575.335 | +28.90% | REVIEW | PASS |
| dedup-history-unrelated-500-mixed-v2 | pure_call_sum_ns | 18163.889 | 23656.609 | +30.24% | SEVERE | TARGET_MISS |

### git_tool_workflow

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| git-tool-1-compact-v2 | pure_call_sum_ns | — | 359.825 | — | INELIGIBLE | PASS |
| git-tool-10-compact-v2 | pure_call_sum_ns | — | 692.451 | — | INELIGIBLE | PASS |
| git-tool-100-mixed-v4 | pure_call_sum_ns | — | 2320.390 | — | INELIGIBLE | TARGET_MISS |
| git-tool-500-mixed-v4 | pure_call_sum_ns | — | 5504.488 | — | INELIGIBLE | TARGET_MISS |

### mixed_load_bearing

| Case | Timer | Baseline (ms) | Candidate (ms) | Change | Elapsed classification | Absolute target |
|---|---|---:|---:|---:|---|---|
| agent-episodes-1-compact-v2 | pure_call_sum_ns | 26.241 | 32.438 | +23.62% | REVIEW | PASS |
| agent-episodes-10-compact-v2 | pure_call_sum_ns | 89.911 | 115.521 | +28.48% | REVIEW | PASS |
| agent-episodes-100 | pure_call_sum_ns | 908.411 | 996.743 | +9.72% | OBSERVED_INCREASE | PASS |
| agent-episodes-500 | pure_call_sum_ns | 7535.401 | 7447.490 | -1.17% | NO_INCREASE | PASS |

## Additional proof-only cases

| Family | Case | Result |
|---|---|---|
| dedup_cdc_locality | dedup-cdc-boundaries-proof | PASS |
| workspace_reliability | workspace-invalid-sdk-edit-compact-v2-proof | PASS |
| workspace_reliability | workspace-invalid-namespace-compact-v2-proof | PASS |
| workspace_reliability | workspace-lease-lifecycle-compact-v2-proof | PASS |
| workspace_reliability | workspace-open-writer-busy-compact-v2-proof | PASS |
| workspace_reliability | workspace-live-execution-busy-compact-v2-proof | PASS |
| workspace_reliability | workspace-candidate-failure-retry-compact-v2-proof | PASS |
| workspace_reliability | workspace-admission-batch-failure-retry-compact-v2-proof | PASS |
| workspace_reliability | workspace-final-publication-failure-retry-compact-v2-proof | PASS |
| workspace_reliability | workspace-published-presentation-failure-smoke-v3-proof | PASS |
| workspace_reliability | workspace-dirty-end-discard-compact-v2-proof | PASS |
| workspace_reliability | workspace-dirty-net-zero-compact-v2-proof | PASS |
| workspace_reliability | workspace-short-spool-write-compact-v2-proof | PASS |
| workspace_reliability | workspace-deferred-nospace-compact-v2-proof | PASS |
| workspace_reliability | workspace-workload-cancel-compact-v2-proof | PASS |
| workspace_reliability | workspace-dirty-runtime-disconnect-compact-v2-proof | PASS |
| workspace_reliability | workspace-corrupt-descendant-compact-v2-proof | PASS |
| workspace_reliability | workspace-missing-descendant-compact-v2-proof | PASS |
| workspace_reliability | workspace-parallel-read-write-compact-v2-proof | PASS |
| workspace_reliability | workspace-shared-path-contention-compact-v2-proof | PASS |
| workspace_reliability | workspace-hardlink-alias-compact-v2-proof | PASS |
| workspace_reliability | workspace-symlink-semantics-compact-v2-proof | PASS |
| workspace_reliability | workspace-open-rename-unlink-compact-v2-proof | PASS |
| workspace_reliability | workspace-metadata-chmod-compact-v2-proof | PASS |
| workspace_reliability | workspace-metadata-mtime-compact-v2-proof | PASS |
| workspace_reliability | workspace-metadata-xattr-compact-v2-proof | PASS |
| workspace_reliability | workspace-exec-500-compact-v2-proof | PASS |
| workspace_reliability | workspace-repeat-publication-compact-v2-proof | PASS |
| workspace_reliability | workspace-sustained-600s-compact-v2-proof | NOT_RUN_OPTIONAL |

## Supplemental observations

The small-files workload and all four SDK/FUSE frequent-edit variants passed performance and independent verification. The final frozen Torch `.venv` contains 16,395 files, 1,283 subdirectories, three symlinks and 581,658,413 regular-file bytes. All three final Workspace imports/Commits and exhaustive byte/metadata readbacks passed.

| Final .venv sample | Commit (s) | Exec + Commit (s) | Full proof |
|---|---:|---:|---|
| 1 | 4.643631 | 10.891184 | PASS |
| 2 | 4.578398 | 10.777544 | PASS |
| 3 | 4.620881 | 10.774716 | PASS |

The earlier adjacent optimization pair was **6.671061 → 4.152412 s Commit (37.8% lower)** and **12.667237 → 10.178917 s Exec + Commit (19.6% lower)**, with **5.1875 MiB higher peak RSS** and unchanged allocated Store size. This scoped result is not a guaranteed latency or universal speedup.

Full157 case wall observations were **481.976 s performance / 613.815 s verification**, versus prior **447.247 / 545.827 s**. These historical observations are unpaired; no whole-history speedup or controlled causal regression estimate is claimed.

## Source data and identities

- [Per-case performance CSV](benchmark-performance.csv), including CPU/RSS, their scopes, target status, raw evidence paths and identities.
- [Per-case verification CSV](benchmark-verification.csv), including coverage, omissions and preparation status.
- [Complete issue #98 report and lossless evidence](../../docs/roadmap/0.1/0.1.4/issue98/README.md).
- Qualified source: `9cfb4be477116646258ea0621280ed13b1824c6d`.
- Host benchmark SHA-256: `47b4d44e3e961f3b6a57d2c18dc6d195973133dbdfb87681a55f1b8e07abe3a2`.
- Image: `sha256:42eb806fcfe9db1dc28098eb423340de3e4eaf81fd487389128305e65606d347`.

No measurements were rerun to produce this closeout. Earlier failures, preparation retries and original performance reports remain preserved.

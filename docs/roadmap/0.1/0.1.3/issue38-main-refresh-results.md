# Current-main benchmark refresh: issue38 families plus tiny-churn

**138/138 performance cases and 36/36 selected independent proofs PASS for the requested fast scope.** One seed-1 or SDK repetition-1 observation per case, on one final implementation. This covers the nine families in issue38 (118 cases) plus tiny-file churn (20).

The implementation and matching host/Linux artifacts were held fixed for this corrected campaign. Two focused SDK performance/proof checks belong to this same source and are reused in the final cohort; they were not rerun to seek a favorable result. Every case has exactly one final-source performance observation. Historical results and the failed first campaign remain separate.

## Family results

Seconds, four decimal places; comparisons use unrounded nanoseconds. Min/max below span different registered cases and sizes, not a distribution or confidence interval. The family pass threshold is 15 seconds.

| Family | Cases passed | Declared timer | Across-case min s | Across-case max s |
|---|---:|---|---:|---:|
| init_namespace | 4/4 | `layerstack_init_ns` | 0.0101 | 2.7346 |
| edit_length_preserving | 12/12 | `edit_commit_ns` | 0.0062 | 0.0094 |
| edit_length_changing | 32/32 | `edit_commit_ns` | 0.0055 | 0.0098 |
| edit_canonical_chunk_count | 12/12 | `edit_commit_ns` | 0.0073 | 0.0091 |
| store_footprint | 6/6 | `product_call_sum_ns` | 0.0335 | 5.7087 |
| payload_create_read | 8/8 | `pure_call_sum_ns` | 0.0155 | 3.3690 |
| dedup_workspace_reuse | 14/14 | `pure_call_sum_ns` | 0.0298 | 5.0781 |
| dedup_cross_file | 10/10 | `pure_call_sum_ns` | 0.0042 | 0.5356 |
| dedup_cdc_locality | 20/20 | `pure_call_sum_ns` | 0.0041 | 0.4528 |
| tiny_file_churn | 20/20 | `pure_call_sum_ns` | 0.0181 | 5.1722 |

SDK timers measure the public edit call plus Commit. Namespace timers measure native initialization. Other families retain their declared product-call or complete-lifecycle sums. Setup, builds and independent verification are outside these operation timers. No new statistical scaling or blanket speedup claim is made.

## Every registered case

Host RSS is the reported process-lifetime high-water at/after product work; container memory is the sample-container lifetime peak, including setup/cache effects. They are separate scopes, not incremental per-operation allocations. Dashes mean the phase is not applicable or the metric is unavailable. Full phase observations, SDK resource details, CPU scopes, backing-byte counters, input identities and raw receipt hashes are in the JSON companion.

### init_namespace

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| namespace-100-compact-v3 | 0.0101 | — | — | — | 21.62 | 7.15 |
| namespace-1000-compact-v3 | 0.0399 | — | — | — | 50.98 | 5.34 |
| namespace-10000 | 0.4027 | — | — | — | 68.20 | 5.40 |
| namespace-100000 | 2.7346 | — | — | — | 97.41 | 5.17 |

### edit_length_preserving

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| overwrite-head-4k-on-1mib-ops-1 | 0.0069 | 0.0027 | — | 0.0042 | 29.81 | 5.65 |
| overwrite-head-4k-on-10mib-ops-1 | 0.0070 | 0.0024 | — | 0.0045 | 29.47 | 5.14 |
| overwrite-head-4k-on-100mib-ops-1 | 0.0073 | 0.0021 | — | 0.0052 | 29.62 | 5.65 |
| overwrite-head-4k-on-500mib-ops-1 | 0.0094 | 0.0026 | — | 0.0067 | 29.70 | 5.40 |
| overwrite-middle-4k-on-1mib-ops-1 | 0.0063 | 0.0023 | — | 0.0040 | 29.61 | 5.66 |
| overwrite-middle-4k-on-10mib-ops-1 | 0.0074 | 0.0030 | — | 0.0044 | 29.72 | 5.18 |
| overwrite-middle-4k-on-100mib-ops-1 | 0.0066 | 0.0021 | — | 0.0045 | 29.58 | 5.42 |
| overwrite-middle-4k-on-500mib-ops-1 | 0.0089 | 0.0028 | — | 0.0061 | 29.80 | 5.42 |
| overwrite-tail-4k-on-1mib-ops-1 | 0.0062 | 0.0020 | — | 0.0042 | 29.64 | 5.41 |
| overwrite-tail-4k-on-10mib-ops-1 | 0.0070 | 0.0027 | — | 0.0042 | 29.62 | 5.40 |
| overwrite-tail-4k-on-100mib-ops-1 | 0.0080 | 0.0027 | — | 0.0053 | 29.59 | 5.65 |
| overwrite-tail-4k-on-500mib-ops-1 | 0.0083 | 0.0026 | — | 0.0057 | 29.31 | 4.89 |

### edit_length_changing

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| insert-middle-4k-on-1mib-ops-1 | 0.0067 | 0.0023 | — | 0.0044 | 29.70 | 5.43 |
| insert-middle-4k-on-10mib-ops-1 | 0.0078 | 0.0027 | — | 0.0050 | 29.64 | 5.40 |
| insert-middle-4k-on-100mib-ops-1 | 0.0068 | 0.0023 | — | 0.0045 | 29.52 | 5.15 |
| insert-middle-4k-on-500mib-result-capped-v2-ops-1 | 0.0073 | 0.0023 | — | 0.0050 | 29.50 | 4.16 |
| delete-middle-4k-on-1mib-ops-1 | 0.0060 | 0.0019 | — | 0.0041 | 29.70 | 5.65 |
| delete-middle-4k-on-10mib-ops-1 | 0.0064 | 0.0021 | — | 0.0043 | 29.69 | 5.64 |
| delete-middle-4k-on-100mib-ops-1 | 0.0067 | 0.0023 | — | 0.0044 | 29.56 | 5.65 |
| delete-middle-4k-on-500mib-ops-1 | 0.0071 | 0.0020 | — | 0.0051 | 29.61 | 5.93 |
| append-tail-4k-on-1mib-ops-1 | 0.0073 | 0.0032 | — | 0.0041 | 29.75 | 5.43 |
| append-tail-4k-on-10mib-ops-1 | 0.0071 | 0.0024 | — | 0.0047 | 29.52 | 5.16 |
| append-tail-4k-on-100mib-ops-1 | 0.0075 | 0.0028 | — | 0.0047 | 29.47 | 5.68 |
| append-tail-4k-on-500mib-result-capped-v2-ops-1 | 0.0073 | 0.0025 | — | 0.0048 | 29.48 | 4.68 |
| prepend-head-4k-on-1mib-ops-1 | 0.0071 | 0.0027 | — | 0.0044 | 29.72 | 5.40 |
| prepend-head-4k-on-10mib-ops-1 | 0.0063 | 0.0022 | — | 0.0041 | 29.38 | 5.40 |
| prepend-head-4k-on-100mib-ops-1 | 0.0062 | 0.0020 | — | 0.0042 | 29.62 | 5.14 |
| prepend-head-4k-on-500mib-result-capped-v2-ops-1 | 0.0073 | 0.0025 | — | 0.0048 | 29.30 | 5.17 |
| replace-grow-middle-2k-to-4k-on-1mib-ops-1 | 0.0076 | 0.0027 | — | 0.0049 | 29.48 | 5.65 |
| replace-grow-middle-2k-to-4k-on-10mib-ops-1 | 0.0068 | 0.0021 | — | 0.0047 | 29.50 | 4.89 |
| replace-grow-middle-2k-to-4k-on-100mib-ops-1 | 0.0098 | 0.0032 | — | 0.0066 | 29.72 | 6.16 |
| replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1 | 0.0079 | 0.0022 | — | 0.0057 | 29.62 | 5.16 |
| replace-shrink-middle-4k-to-2k-on-1mib-ops-1 | 0.0066 | 0.0022 | — | 0.0044 | 29.47 | 5.17 |
| replace-shrink-middle-4k-to-2k-on-10mib-ops-1 | 0.0060 | 0.0021 | — | 0.0039 | 29.72 | 5.40 |
| replace-shrink-middle-4k-to-2k-on-100mib-ops-1 | 0.0064 | 0.0018 | — | 0.0046 | 29.41 | 5.93 |
| replace-shrink-middle-4k-to-2k-on-500mib-ops-1 | 0.0069 | 0.0020 | — | 0.0049 | 29.42 | 4.89 |
| truncate-tail-4k-on-1mib-ops-1 | 0.0055 | 0.0021 | — | 0.0034 | 29.50 | 4.90 |
| truncate-tail-4k-on-10mib-ops-1 | 0.0063 | 0.0023 | — | 0.0039 | 29.50 | 4.64 |
| truncate-tail-4k-on-100mib-ops-1 | 0.0068 | 0.0025 | — | 0.0044 | 29.72 | 4.65 |
| truncate-tail-4k-on-500mib-ops-1 | 0.0075 | 0.0023 | — | 0.0052 | 29.78 | 5.15 |
| zero-extend-tail-4k-on-1mib-ops-1 | 0.0071 | 0.0025 | — | 0.0046 | 29.59 | 4.69 |
| zero-extend-tail-4k-on-10mib-ops-1 | 0.0063 | 0.0024 | — | 0.0039 | 29.42 | 5.90 |
| zero-extend-tail-4k-on-100mib-ops-1 | 0.0059 | 0.0020 | — | 0.0040 | 29.50 | 4.67 |
| zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1 | 0.0080 | 0.0029 | — | 0.0051 | 29.83 | 4.39 |

### edit_canonical_chunk_count

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1 | 0.0073 | 0.0027 | — | 0.0047 | 29.58 | 4.84 |
| overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1 | 0.0076 | 0.0027 | — | 0.0049 | 29.64 | 5.65 |
| overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1 | 0.0078 | 0.0023 | — | 0.0056 | 29.75 | 6.98 |
| overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1 | 0.0091 | 0.0028 | — | 0.0063 | 29.50 | 5.15 |
| overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1 | 0.0073 | 0.0023 | — | 0.0049 | 29.55 | 5.39 |
| overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1 | 0.0077 | 0.0023 | — | 0.0055 | 29.56 | 5.44 |
| overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1 | 0.0085 | 0.0026 | — | 0.0059 | 29.61 | 5.39 |
| overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1 | 0.0087 | 0.0025 | — | 0.0062 | 29.55 | 5.39 |
| overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1 | 0.0081 | 0.0028 | — | 0.0053 | 29.58 | 4.65 |
| overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1 | 0.0076 | 0.0026 | — | 0.0051 | 29.72 | 5.14 |
| overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1 | 0.0080 | 0.0024 | — | 0.0056 | 29.64 | 4.90 |
| overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1 | 0.0080 | 0.0022 | — | 0.0058 | 29.48 | 5.39 |

### store_footprint

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| store-footprint-unique-100000 | 2.9367 | — | — | — | — | 5.18 |
| store-footprint-metadata-cardinality-100000 | 5.7087 | — | — | — | — | 5.60 |
| store-footprint-large-object-500m | 0.4954 | — | — | — | — | 5.14 |
| store-footprint-unique-100-low-v1 | 0.0341 | — | — | — | — | 5.68 |
| store-footprint-metadata-cardinality-100-low-v1 | 0.0335 | — | — | — | — | 5.43 |
| store-footprint-large-object-10m-low-v1 | 0.0531 | — | — | — | — | 5.65 |

### payload_create_read

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| payload-create-1m-compact-v2 | 0.0420 | — | 0.0116 | 0.0090 | 15.33 | 8.18 |
| payload-create-10m-compact-v2 | 0.0903 | — | 0.0330 | 0.0457 | 35.48 | 10.05 |
| payload-create-100m | 0.7236 | — | 0.2526 | 0.4563 | 64.53 | 12.46 |
| payload-create-500m | 3.3690 | — | 1.2475 | 2.1079 | 70.59 | 12.68 |
| payload-random-read-1-compact-v2 | 0.0155 | — | 0.0036 | 0.0016 | 8.61 | 5.68 |
| payload-random-read-10-compact-v2 | 0.0202 | — | 0.0081 | 0.0020 | 10.06 | 5.90 |
| payload-random-read-100 | 0.0656 | — | 0.0530 | 0.0017 | 27.98 | 5.65 |
| payload-random-read-500 | 0.2788 | — | 0.2643 | 0.0020 | 47.84 | 7.52 |

### dedup_workspace_reuse

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| dedup-workspace-exact-1-compact-v2 | 0.0345 | — | 0.0120 | 0.0100 | 14.58 | 6.98 |
| dedup-workspace-exact-10-compact-v2 | 0.1241 | — | 0.0527 | 0.0588 | 38.53 | 11.36 |
| dedup-workspace-exact-100 | 1.0391 | — | 0.5036 | 0.5192 | 69.52 | 12.54 |
| dedup-workspace-exact-500 | 5.0011 | — | 2.9000 | 2.0821 | 73.42 | 12.69 |
| dedup-workspace-local-1-compact-v2 | 0.0321 | — | 0.0104 | 0.0100 | 14.48 | 7.14 |
| dedup-workspace-local-10-compact-v2 | 0.1187 | — | 0.0496 | 0.0570 | 40.23 | 10.30 |
| dedup-workspace-local-100 | 0.9449 | — | 0.4471 | 0.4805 | 68.58 | 12.52 |
| dedup-workspace-local-500 | 5.0332 | — | 2.8648 | 2.1524 | 72.22 | 12.68 |
| dedup-workspace-unique-1-compact-v2 | 0.0298 | — | 0.0107 | 0.0084 | 15.94 | 7.07 |
| dedup-workspace-unique-10-compact-v2 | 0.1136 | — | 0.0580 | 0.0433 | 42.05 | 11.75 |
| dedup-workspace-unique-100 | 0.8891 | — | 0.4606 | 0.4121 | 67.66 | 12.27 |
| dedup-workspace-unique-500 | 5.0781 | — | 2.8238 | 2.2360 | 73.00 | 12.94 |
| dedup-workspace-unique-1-base128-v3 | 0.0336 | — | 0.0122 | 0.0096 | 37.52 | 6.11 |
| dedup-workspace-unique-10-base128-v3 | 0.1341 | — | 0.0584 | 0.0615 | 58.91 | 10.32 |

### dedup_cross_file

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| dedup-cross-file-anchor-1 | 0.0042 | — | — | — | — | 4.91 |
| dedup-cross-file-unique-10 | 0.0243 | — | — | — | — | 5.90 |
| dedup-cross-file-unique-100 | 0.1069 | — | — | — | — | 4.64 |
| dedup-cross-file-unique-500 | 0.4582 | — | — | — | — | 5.39 |
| dedup-cross-file-identical-10 | 0.0188 | — | — | — | — | 4.93 |
| dedup-cross-file-identical-100 | 0.0296 | — | — | — | — | 5.39 |
| dedup-cross-file-identical-500 | 0.1112 | — | — | — | — | 5.39 |
| dedup-cross-file-mixed-10 | 0.0258 | — | — | — | — | 4.95 |
| dedup-cross-file-mixed-100 | 0.1137 | — | — | — | — | 4.93 |
| dedup-cross-file-mixed-500 | 0.5356 | — | — | — | — | 5.43 |

### dedup_cdc_locality

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| dedup-cdc-overwrite-1 | 0.0041 | — | — | — | — | 5.64 |
| dedup-cdc-overwrite-10 | 0.0189 | — | — | — | — | 5.64 |
| dedup-cdc-overwrite-100 | 0.0315 | — | — | — | — | 4.64 |
| dedup-cdc-overwrite-500 | 0.1337 | — | — | — | — | 5.35 |
| dedup-cdc-insert-1 | 0.0048 | — | — | — | — | 5.92 |
| dedup-cdc-insert-10 | 0.0193 | — | — | — | — | 5.66 |
| dedup-cdc-insert-100 | 0.0346 | — | — | — | — | 5.90 |
| dedup-cdc-insert-500 | 0.1372 | — | — | — | — | 5.34 |
| dedup-cdc-delete-1 | 0.0047 | — | — | — | — | 5.65 |
| dedup-cdc-delete-10 | 0.0192 | — | — | — | — | 5.39 |
| dedup-cdc-delete-100 | 0.0332 | — | — | — | — | 4.41 |
| dedup-cdc-delete-500 | 0.1430 | — | — | — | — | 4.92 |
| dedup-cdc-common-body-1 | 0.0048 | — | — | — | — | 5.39 |
| dedup-cdc-common-body-10 | 0.0207 | — | — | — | — | 4.65 |
| dedup-cdc-common-body-100 | 0.0572 | — | — | — | — | 5.65 |
| dedup-cdc-common-body-500 | 0.2545 | — | — | — | — | 5.18 |
| dedup-cdc-scattered-1 | 0.0047 | — | — | — | — | 4.86 |
| dedup-cdc-scattered-10 | 0.0242 | — | — | — | — | 5.59 |
| dedup-cdc-scattered-100 | 0.0910 | — | — | — | — | 5.20 |
| dedup-cdc-scattered-500 | 0.4528 | — | — | — | — | 5.41 |

### tiny_file_churn

| Case | Measured s | SDK edit s | Exec s | Commit s | Host peak RSS MiB | Container lifetime peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| tiny-create-1-compact-v2 | 0.0220 | — | 0.0068 | 0.0038 | 9.27 | 5.69 |
| tiny-create-10-compact-v2 | 0.0357 | — | 0.0178 | 0.0056 | 10.34 | 4.68 |
| tiny-create-100 | 0.1520 | — | 0.0827 | 0.0548 | 54.09 | 5.43 |
| tiny-create-500 | 0.4802 | — | 0.3665 | 0.1023 | 71.97 | 7.05 |
| tiny-stat-1-compact-v2 | 0.0181 | — | 0.0053 | 0.0016 | 8.95 | 7.18 |
| tiny-stat-10-compact-v2 | 0.0246 | — | 0.0125 | 0.0015 | 10.84 | 5.42 |
| tiny-stat-100 | 0.0827 | — | 0.0685 | 0.0017 | 30.92 | 5.43 |
| tiny-stat-500 | 0.2802 | — | 0.2654 | 0.0017 | 40.00 | 5.39 |
| tiny-unlink-1-compact-v2 | 0.0202 | — | 0.0061 | 0.0029 | 9.59 | 5.64 |
| tiny-unlink-10-compact-v2 | 0.0296 | — | 0.0148 | 0.0040 | 12.19 | 5.40 |
| tiny-unlink-100 | 0.1495 | — | 0.0871 | 0.0491 | 52.28 | 4.66 |
| tiny-unlink-500 | 0.4192 | — | 0.3163 | 0.0825 | 62.41 | 5.17 |
| tiny-bulk-create-1-compact-v2 | 0.0995 | — | 0.0733 | 0.0144 | 14.17 | 6.49 |
| tiny-bulk-create-10-compact-v2 | 0.3175 | — | 0.2498 | 0.0540 | 34.33 | 8.89 |
| tiny-bulk-create-100-mixed-v3 | 1.0377 | — | 0.6400 | 0.3800 | 75.91 | 13.80 |
| tiny-bulk-create-500-mixed-v3 | 5.1722 | — | 3.1179 | 2.0267 | 94.38 | 23.24 |
| tiny-bulk-delete-1-compact-v2 | 0.1479 | — | 0.1279 | 0.0079 | 10.34 | 5.14 |
| tiny-bulk-delete-10-compact-v2 | 0.2403 | — | 0.2179 | 0.0104 | 22.98 | 6.15 |
| tiny-bulk-delete-100-mixed-v3 | 0.3346 | — | 0.3061 | 0.0166 | 37.80 | 5.16 |
| tiny-bulk-delete-500-mixed-v3 | 1.0799 | — | 1.0175 | 0.0498 | 53.30 | 6.14 |

## Accepted tier100 result and retained original classifier

| Case | Full lifecycle s | Original <1.0000 s classifier | Current acceptance |
|---|---:|---|---|
| tiny-bulk-create-100-mixed-v3 | 1.0377 | TARGET_MISS | Explicitly accepted by user |
| tiny-bulk-delete-100-mixed-v3 | 0.3346 | PASS | Meets original target |

The user explicitly accepted the current bulk-create-100 observation of 1.037675209 seconds (displayed as1.0377). This result is accepted for the requested integration; no further tuning is required to chase the earlier subsecond target. The original mathematical classifier is preserved in raw receipts, and no new general numerical threshold is inferred. Both tier100 independent proofs were added after this acceptance. No timing or failed receipt was discarded, and no additional issue is automatically closed.

## Independent verification

36 selected proofs passed. Total observed proof wall 128.3375 s; maximum 15.8288 s. Each proof retains the 45-second work /59-second hard budget. Separate preparation is recorded outside proof wall. The two same-source focused SDK proofs are included at their original measured wall and receipt identities.

| Family | Case | Proof wall s |
|---|---|---:|
| edit_canonical_chunk_count | overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1 | 1.5157 |
| edit_canonical_chunk_count | overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1 | 1.6386 |
| edit_canonical_chunk_count | overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1 | 1.5971 |
| edit_length_changing | append-tail-4k-on-1mib-ops-1 | 1.5874 |
| edit_length_changing | delete-middle-4k-on-1mib-ops-1 | 1.6049 |
| edit_length_changing | insert-middle-4k-on-1mib-ops-1 | 1.7056 |
| edit_length_changing | insert-middle-4k-on-500mib-result-capped-v2-ops-1 | 4.1741 |
| edit_length_changing | prepend-head-4k-on-1mib-ops-1 | 1.5968 |
| edit_length_changing | replace-grow-middle-2k-to-4k-on-1mib-ops-1 | 1.7714 |
| edit_length_changing | replace-shrink-middle-4k-to-2k-on-1mib-ops-1 | 1.7893 |
| edit_length_changing | truncate-tail-4k-on-1mib-ops-1 | 1.6450 |
| edit_length_changing | zero-extend-tail-4k-on-1mib-ops-1 | 1.6364 |
| edit_length_preserving | overwrite-head-4k-on-1mib-ops-1 | 1.5128 |
| edit_length_preserving | overwrite-middle-4k-on-1mib-ops-1 | 1.5401 |
| edit_length_preserving | overwrite-tail-4k-on-1mib-ops-1 | 1.6596 |
| dedup_cdc_locality | dedup-cdc-boundaries-proof | 1.5503 |
| dedup_cdc_locality | dedup-cdc-insert-500 | 10.7539 |
| dedup_cross_file | dedup-cross-file-mixed-500 | 9.3237 |
| dedup_workspace_reuse | dedup-workspace-unique-500 | 15.8288 |
| init_namespace | namespace-100-compact-v3 | 1.6891 |
| init_namespace | namespace-1000-compact-v3 | 1.4641 |
| init_namespace | namespace-10000 | 2.0537 |
| init_namespace | namespace-100000 | 5.3632 |
| payload_create_read | payload-create-500m | 10.1316 |
| store_footprint | store-footprint-large-object-500m | 2.4554 |
| store_footprint | store-footprint-metadata-cardinality-100000 | 7.5948 |
| store_footprint | store-footprint-unique-100000 | 5.5677 |
| tiny_file_churn | tiny-create-1-compact-v2 | 1.9378 |
| tiny_file_churn | tiny-stat-1-compact-v2 | 1.7826 |
| tiny_file_churn | tiny-unlink-1-compact-v2 | 1.8447 |
| tiny_file_churn | tiny-bulk-create-1-compact-v2 | 2.0913 |
| tiny_file_churn | tiny-bulk-delete-1-compact-v2 | 1.9127 |
| tiny_file_churn | tiny-bulk-create-500-mixed-v3 | 7.2813 |
| tiny_file_churn | tiny-bulk-delete-500-mixed-v3 | 5.3043 |
| tiny_file_churn | tiny-bulk-create-100-mixed-v3 | 3.0522 |
| tiny_file_churn | tiny-bulk-delete-100-mixed-v3 | 2.3794 |

Coverage includes all SDK operation/outcome shapes at 1 MiB plus capped 500 MiB insertion, all four namespace sizes, the three main storage controls, selected large payload/reuse/CAS/CDC cases, proof-only CDC boundaries, and nine tiny-churn shapes, including the user-accepted tier100 pair. Mixed-v3 tier500 creation covers three 64 KiB ranges (beginning/middle/end) of every large file plus declared small/medium/witness paths; deletion covers selected absence and witness retention.

These are sampled proofs. They do not establish exhaustive file-byte, namespace/object-census, alias/failure-injection, multi-seed scaling, other-platform or physical 100-workspace qualification. Exact sampled paths/ranges and omissions are retained in the JSON companion.

## Failures found and corrected before integration

The first attempt completed all 138 timings under 15 seconds, but all 56 SDK performance rows reported nested resource failure, and all 15 selected SDK proofs failed their resource gate despite semantic/canonical/route checks passing. The initial derived timing assessment omitted those nested fields; its correction and every original receipt remain preserved. That attempt is explicitly NOT QUALIFIED.

General owner/backing metadata and frozen-fact frames were incorrectly classified as FUSE-write frames. The implementation now attributes only append traffic to FUSE write-frame counters and separately records submitted request-body bytes for all backing calls as `live_backing_request_bytes`. This records the actual control/backing work rather than hiding it or relaxing the zero-FUSE-write SDK gate. SDK data-cache updates now run only for nodes that have had cached opens; metadata invalidation remains. This avoids streaming a cold file suffix solely to populate and invalidate cache pages. Existing resource, canonical, route and timing thresholds remain unchanged.

Pre-integration checks also fixed stale unused wrappers/strict Clippy issues and the Linux unmounted-owner path incorrectly requiring a kernel notifier. Unmounted direct/TCP owners now edit without a kernel session; mounted owners retain required cache update/error handling. The earlier Linux CI failure and focused regression evidence are retained.

## Validation, provenance and cleanup

- Implementation commit: `448a74bfef5947a0ad91594bf3e5c67865e9a731`; unmodified upstream fuser 0.18.0.
- Source seal: `659bd68f0669c9416465aaba99bdbf2d42063b60b68d32bd3e5028d0d47e8c96`.
- Product seal: `0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695`.
- Workload SHA-256: `2ec809798656deef9ac05903c9e3bd77921f6ef9c6c87f24060812a81b93bf76`.
- Host binary SHA-256: `8c0ef6c4773f0a1560acfa2476865631b1ab6330e03ffcf4d2a7beed4f82433c`.
- Linux image: `layerfs-bench-infra:659bd68f0669c941` / `sha256:f035fd216f6a298c762f86bc38af3f1bf2dbffe387a100f540cd04af20baaf11`.
- [Final implementation Linux CI](https://github.com/Ephemeral-AI-Lab/layerfs/actions/runs/34033533660): formatting, full native tests and warnings-denied Clippy PASS.
- Local full native suite passed in 114 seconds with two bounded test jobs, within 120 seconds; final changed-seam direct/TCP regression and strict host Clippy pass.
- Final-source real-FUSE ordinary writes + active mappings + SDK edits/concurrent Commit passed in 2.36 seconds, including peer Workspace lifecycle and cleanup.
- macOS owns SDK/coordinator, physical backing, SQLite and canonical publication; Linux owns daemon/FUSE/workload/live operation state. Observed limits: 2 CPUs, 2 GiB, no swap, 256 PIDs, no data mounts. Host CPU is uncapped and reported separately.
- Every applicable protected SQLite master remained unchanged; native initialization used fresh outputs from owned prepared inputs. All performance/proof cleanup and exact campaign-owned container/sample-directory inventory checks passed. Protected preparation is retained.
- Corrected campaign: `benchmark-results/host-store/campaigns/issue38-main-refresh-1788696142088555000-corrected`.
- Retained first attempt: `benchmark-results/host-store/campaigns/issue38-main-refresh-1788696142088555000`.
- [Machine-readable per-case statistics, resource scopes, source/input identities, raw receipt hashes and proof coverage](issue38-main-refresh-results.json).
- Raw receipts, preparation logs and failed attempts remain append-only in their local campaign directories; final report commits do not change the measured implementation.

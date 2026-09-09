# selected-chain-1: ten-snapshot measured result

Final allocated storage is **50,364,416 bytes**, growth **50,294,784 bytes** from **69,632 bytes**. The 45,000,000-byte objective is not met; the exact difference is **+5,364,416 bytes**. This is not near-target. No release-admission PASS or full157 confirmation is claimed.

Actual outcomes: **10 Created / 0 UpToDate**. Same-Store verification checked **10 states, 58,860 path states, 327,885,165 logical bytes**; cleanup was {'performance': 'PASS', 'verification': 'PASS'}. Frozen performance and post-verification manifests are separate lifecycle points. Verification checked the measured digest before reopening the SAME Store in a new coordinator; no replacement history was built.

The fixed full157 indices are **1, 18, 36, 53, 70, 88, 105, 122, 140, 157**. Skipped checkpoints are not replayed. Controls retain the [applicability audit](baseline-applicability-45mb.md). Git retains these ten trees with narrower metadata; its construction/packing timers do not match foreground save latency.

| Arm | Final allocated B | Candidate difference B |
| --- | --- | --- |
| Git | 38,223,872 | +12,140,544 |
| released v014 | 67,145,728 | -16,781,312 |
| original v015 | 66,105,344 | -15,740,928 |
| chain-1 | 56,668,160 | -6,303,744 |
| selected-full-1 | 54,562,816 | -4,198,400 |

**Rejected and reverted in `d18ae46fe`.** Content packs grew **1,854 B**, metadata packs grew **14,747 B**, and logical SQLite size grew **4,096 B** versus removed-base-1. Final allocation fell only **8,192 B** because filesystem/nonpack differences moved in the other direction. Commit median increased from 0.5805168755 to 0.6219250625 seconds and Commit sum from 7.167381335 to 7.912139627 seconds. The expansion does not provide useful measured storage benefit for its cost. All ten same-Store checks and cleanup succeeded, so the rejection is a measured optimization decision, not a correctness failure.

Retained [product patch](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-product.patch), source `0d2c0830f`, binaries/images, census and all command receipts remain available. Reverting restores the measured removed-base product; no unchanged run was repeated to manufacture a better observation.

## Public timings and resource tradeoffs

| Metric | Min s | Median s | Max s | Sum s | Median vs original v015 |
| --- | --- | --- | --- | --- | --- |
| exec_ns | 0.185946542 | 2.850479166 | 7.471900750 | 34.513543333 | -4.42% |
| commit_ns | 0.042207875 | 0.621925063 | 1.846278625 | 7.912139627 | +27.97% |
| paired_ns | 0.228154417 | 3.472404229 | 9.318179375 | 42.425682960 | +0.12% |
| verification_step_wall_ns | 0.191313334 | 2.301149313 | 4.378347000 | 22.926694168 | +3.73% |

These are ten dependent history steps. Min/max and per-step observations are descriptive; no tail confidence is inferred. The prospective <=10% speed/resource comparison and separate 8-MiB RSS allowance are engineering criteria, not owner-approved release gates.

| Scope | Seconds |
| --- | --- |
| performance_wall_ns | 58.017789500 |
| performance_work_wall_ns | 53.849927875 |
| verification_wall_ns | 27.341203250 |
| verification_work_wall_ns | 26.038122292 |
| setup_ns | 3.573834458 |
| verification_setup_ns | 0.852907834 |
| cleanup_ns | 0.547054958 |
| verification_cleanup_ns | 0.442446958 |
| preparation_ns | 2.950978208 |
| verification_preparation_ns | 3.280130792 |
| transfer_ns | 7.440725126 |

Preparation, setup, transfer, cleanup and case walls have their existing nested boundaries; do not add all rows as disjoint costs. Build and complete command receipts are below.

| Step | Save s | Commit s | Paired s | Verify step s | Allocated B |
| --- | --- | --- | --- | --- | --- |
| 1 | 0.185946542 | 0.042207875 | 0.228154417 | 0.191313334 | 688128 |
| 2 | 0.886300041 | 0.210683584 | 1.096983625 | 0.770694750 | 4227072 |
| 3 | 1.456601334 | 0.313634375 | 1.770235709 | 1.204012417 | 7372800 |
| 4 | 2.285955333 | 0.494515959 | 2.780471292 | 1.612300167 | 11567104 |
| 5 | 2.980360833 | 0.661826167 | 3.642187000 | 2.067902208 | 16809984 |
| 6 | 2.720597500 | 0.582023958 | 3.302621458 | 2.534396417 | 21004288 |
| 7 | 4.710852500 | 1.075163250 | 5.786015750 | 3.017107750 | 27295744 |
| 8 | 5.532083875 | 1.244467542 | 6.776551417 | 3.204601000 | 34635776 |
| 9 | 6.282944625 | 1.441338292 | 7.724282917 | 3.946019125 | 43024384 |
| 10 | 7.471900750 | 1.846278625 | 9.318179375 | 4.378347000 | 50364416 |

Full source-to-commit mappings, CPU/transfer values and original-oracle counts are in [selected-chain-1-comparison.csv](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-comparison.csv).

Resource values retain their actual scopes: host lifetime peak RSS, cumulative process CPU, and boundary samples for cgroup categories/staging/spool. Boundary maxima are not simultaneous samples or continuous phase-local peaks.

```json
{
  "performance": {
    "host_lifetime_peak_rss_bytes": 113033216,
    "host_max_boundary_rss_bytes": 113033216,
    "host_cpu_ns": 15420130796,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 86470656,
      "memory_peak": 142262272,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 452,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 10289254,
      "throttled_usec": 0,
      "usage_usec": 15872715,
      "user_usec": 5583461
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 63324160,
      "file": 7016448,
      "file_dirty": 36864,
      "file_writeback": 0,
      "kernel": 14782464,
      "shmem": 0,
      "slab": 14125248
    },
    "spool_max_boundary_bytes": 45056,
    "staging_max_boundary_bytes": 71393280
  },
  "verification": {
    "host_lifetime_peak_rss_bytes": 71270400,
    "host_max_boundary_rss_bytes": 71237632,
    "host_cpu_ns": 8399339791,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 137482240,
      "memory_peak": 242171904,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 262,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 5044393,
      "throttled_usec": 0,
      "usage_usec": 9562990,
      "user_usec": 4518596
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 133386240,
      "file": 1970176,
      "file_dirty": 8192,
      "file_writeback": 0,
      "kernel": 1359872,
      "shmem": 0,
      "slab": 650240
    },
    "spool_max_boundary_bytes": 40960,
    "staging_max_boundary_bytes": 1966080
  }
}
```

## Physical reconciliation

| Category | Bytes |
| --- | --- |
| all_pack_bytes | 44,854,800 |
| file_content_pack_bytes | 38,281,625 |
| filesystem_allocation_difference_bytes | 794,624 |
| metadata_legacy_pack_bytes | 6,573,175 |
| sqlite_logical_bytes | 49,569,792 |
| sqlite_nonpack_bytes | 4,714,992 |
| store_allocated_bytes | 50,364,416 |

Content + metadata + SQLite nonpack + filesystem allocation difference equals final allocated bytes. SQLite logical size and filesystem allocation are distinct. FULL bases below are subsets of retained physical records, counted once; never add their bytes again to total storage.

```json
{
  "counts": {
    "large_CDC_FULL": {
      "canonical_bytes": 7453425,
      "frame_bytes": 2097249,
      "objects": 381,
      "raw_bytes": 7445424,
      "record_bytes": 2099154,
      "record_header_bytes": 1905,
      "selected": 381
    },
    "large_CDC_PREFIX": {
      "canonical_bytes": 7271663,
      "frame_bytes": 1140960,
      "objects": 354,
      "raw_bytes": 7264229,
      "record_bytes": 1154058,
      "record_header_bytes": 13098,
      "selected": 354
    },
    "legacy_full_roles": {
      "LFS4CHK": 5,
      "LFS4DIR": 4008,
      "LFS4FSR": 11,
      "LFS4INO": 37289,
      "LFS4INT": 421,
      "LFS4LNK": 7,
      "LFS4MAP": 124,
      "LFS4MET": 4,
      "LFS4NSP": 4145
    },
    "metadata_legacy_DELTA": {
      "canonical_bytes": 1634948,
      "objects": 275,
      "raw_bytes": 1634948,
      "record_bytes": 997987,
      "selected": 275
    },
    "metadata_legacy_FULL": {
      "canonical_bytes": 8560054,
      "objects": 46014,
      "raw_bytes": 8560054,
      "record_bytes": 8606068,
      "selected": 46014
    },
    "pack_v1": {
      "bytes": 6573175,
      "decoded_group_bytes": 9792355,
      "encoded_group_bytes": 6558215,
      "groups": 786,
      "header_directory_bytes": 14960,
      "packs": 149,
      "record_directory_bytes": 188300
    },
    "pack_v2": {
      "bytes": 3259492,
      "decoded_group_bytes": 3256564,
      "encoded_group_bytes": 3256564,
      "groups": 103,
      "header_directory_bytes": 2928,
      "packs": 80,
      "record_directory_bytes": 3352
    },
    "pack_v3": {
      "bytes": 35022133,
      "decoded_group_bytes": 34482421,
      "encoded_group_bytes": 34482421,
      "groups": 33217,
      "header_directory_bytes": 539712,
      "packs": 515
    },
    "small_DELTA": {
      "canonical_bytes": 187268156,
      "frame_bytes": 11791592,
      "objects": 24413,
      "raw_bytes": 186706657,
      "record_bytes": 12792525,
      "record_header_bytes": 1000933,
      "selected": 24413
    },
    "small_FULL": {
      "canonical_bytes": 59303705,
      "frame_bytes": 21610660,
      "objects": 8804,
      "raw_bytes": 59101213,
      "record_bytes": 21689896,
      "record_header_bytes": 79236,
      "selected": 8804
    },
    "small_depth_counts": {
      "0": 8804,
      "1": 7004,
      "2": 5155,
      "3": 3690,
      "4": 2800,
      "5": 2166,
      "6": 1535,
      "7": 1151,
      "8": 912
    },
    "small_physical_DELTA_bases": {
      "count": 14971,
      "raw_bytes": 120331927,
      "record_bytes": 8970460
    },
    "small_physical_FULL_bases": {
      "count": 6066,
      "raw_bytes": 44592234,
      "record_bytes": 16435058
    },
    "small_physical_bases": {
      "count": 21037,
      "raw_bytes": 164924161,
      "record_bytes": 25405518
    },
    "small_record_kinds": {
      "0": 8804,
      "1": 1072,
      "2": 23341
    }
  },
  "pages": [
    {
      "bytes": 4096,
      "name": "branch_identity",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 37,
      "unused": 4048
    },
    {
      "bytes": 4096,
      "name": "branch_names",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 45,
      "unused": 4040
    },
    {
      "bytes": 4096,
      "name": "branches",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 113,
      "unused": 3972
    },
    {
      "bytes": 4096,
      "name": "commits",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 1327,
      "unused": 2722
    },
    {
      "bytes": 4096,
      "name": "layer_identity",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 53,
      "unused": 4032
    },
    {
      "bytes": 4096,
      "name": "layer_stack_names",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 33,
      "unused": 4052
    },
    {
      "bytes": 4096,
      "name": "layer_stacks",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 67,
      "unused": 4018
    },
    {
      "bytes": 4096,
      "name": "layers",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 89,
      "unused": 3996
    },
    {
      "bytes": 4096,
      "name": "layers_child",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 0,
      "unused": 4088
    },
    {
      "bytes": 4096,
      "name": "layers_genesis",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 53,
      "unused": 4032
    },
    {
      "bytes": 4096,
      "name": "layers_source",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 0,
      "unused": 4088
    },
    {
      "bytes": 4096,
      "name": "object_packs",
      "pages": 1,
      "pagetype": "internal",
      "payload": 0,
      "unused": 417
    },
    {
      "bytes": 1925120,
      "name": "object_packs",
      "pages": 470,
      "pagetype": "leaf",
      "payload": 1459202,
      "unused": 454495
    },
    {
      "bytes": 43466752,
      "name": "object_packs",
      "pages": 10612,
      "pagetype": "overflow",
      "payload": 43399238,
      "unused": 25066
    },
    {
      "bytes": 57344,
      "name": "objects",
      "pages": 14,
      "pagetype": "internal",
      "payload": 42240,
      "unused": 8013
    },
    {
      "bytes": 4055040,
      "name": "objects",
      "pages": 990,
      "pagetype": "leaf",
      "payload": 3385139,
      "unused": 424225
    },
    {
      "bytes": 4096,
      "name": "sqlite_schema",
      "pages": 1,
      "pagetype": "internal",
      "payload": 0,
      "unused": 3977
    },
    {
      "bytes": 8192,
      "name": "sqlite_schema",
      "pages": 2,
      "pagetype": "leaf",
      "payload": 5347,
      "unused": 2763
    },
    {
      "bytes": 4096,
      "name": "workspace_stages",
      "pages": 1,
      "pagetype": "leaf",
      "payload": 0,
      "unused": 4088
    }
  ],
  "pragmas": {
    "auto_vacuum": 0,
    "freelist_count": 0,
    "page_count": 12102,
    "page_size": 4096,
    "user_version": 9
  },
  "sqlite_residual_bytes": 0,
  "max_depth": 8,
  "max_canonical_closure": 521863,
  "max_encoded_closure": 71494
}
```

## Exact custody and commands

```json
{
  "source": {
    "LAYERFS_PRODUCT_SEAL": "021fff6efcce0bd909c5447ce3e9a69beafc3eb685081eb4d97136b0a25d08bd",
    "LAYERFS_SOURCE_COMMIT": "0d2c0830fb185832125095b9f7b2b023b6f8ae02",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "4a0cc09ec5da400883a975015701bdfa7f9a612e41f74b77e0d360002d8dd1c5",
    "LAYERFS_SOURCE_TREE": "11e6f413609eec7d754eeb4251c6dc2dccff0545",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127"
  },
  "host": {
    "LAYERFS_PRODUCT_SEAL": "021fff6efcce0bd909c5447ce3e9a69beafc3eb685081eb4d97136b0a25d08bd",
    "LAYERFS_SOURCE_COMMIT": "0d2c0830fb185832125095b9f7b2b023b6f8ae02",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "4a0cc09ec5da400883a975015701bdfa7f9a612e41f74b77e0d360002d8dd1c5",
    "LAYERFS_SOURCE_TREE": "11e6f413609eec7d754eeb4251c6dc2dccff0545",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127",
    "binary_sha256": "e3b6700fa8fe11331406204619ccadcfafbbdcb02820e3c25b1e47d0263f732a",
    "platform": "macOS-26.4.1-arm64-arm-64bit-Mach-O",
    "rust_toolchain": "1.85.1",
    "schema_sha256": "7ed3355be81cfdb82839d1651ee0919afc445ce6255c522f2b1bab54c9a92780"
  },
  "image": "sha256:d27b586c746b4299a537770254ca16a4d4b8c0dad455ef80774cd3523a489103",
  "fixture_digest": "eeb408e63b091aa7379aeefd1e3fbf819cdb4d882c67c03dd040c9fbe04feabb",
  "contract_digest": "9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50",
  "frozen_store_sha256": "f5419a2f34e512991f6939e05748cbfd49c7d0c29673d845e625a4e4485b3147",
  "census_script_sha256": "dcd343150bb137ad30ef2dd87ede9bb86abac8a74065e7db2622efd95cd30440"
}
```

Original fixture manifest SHA256: `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Prepared inputs: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`.

- [selected-chain-1-source.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-source.json); SHA256 `bafc965657db8891bcdd8a498a2dcf75c4e0c7796dc47ed7c52a66bfe24f2658`
- [selected-chain-1-dirty.patch](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-dirty.patch); SHA256 `3515545d955d5f30812c017469c5427fd10d8a5d46d29d61fc5b555868540d37`
- [selected-chain-1-fs-benchmark-pro.identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-fs-benchmark-pro.identity.json); SHA256 `ea63d0f23cd236992013f249f11cc80fd896c22df156d27112a1c7d559c6fc0a`
- [selected-chain-1-image-inspection.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-image-inspection.json); SHA256 `ce0ae3c3c2661c361bee4d818291370a541e6224f8ef2c83017b57635dc7493d`

**build-host**: 57.910489166 s, exit 0. [selected-chain-1-build-host-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-build-host-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-host"
]
```

**build-image**: 79.779340958 s, exit 0. [selected-chain-1-build-image-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-build-image-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-storage-smoke-image"
]
```

**performance**: 61.187336541 s, exit 0. [selected-chain-1-performance-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-performance-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:4a0cc09ec5da4008",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--output",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1"
]
```

**census**: 0.818139625 s, exit 0. [selected-chain-1-census-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-census-command.json)

```json
[
  "python3",
  "docs/roadmap/0.1/0.1.5/issue100/census.py",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1"
]
```

**verification**: 30.854155167 s, exit 0. [selected-chain-1-verification-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-verification-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:4a0cc09ec5da4008",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--storage-verify-run",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1"
]
```

- [identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1/identity.json)
- [performance-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1/performance-manifest.json)
- [verification-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1/verification-manifest.json)
- [census.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1/census.json)
- [selected-chain-1-comparison.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-chain-1-comparison.json)

No broad Cargo/Clippy/doctest or unrelated qualification suite, release/tag, GC/repacking or unchanged-control rerun is claimed. The premature chain full157 execution remains incomplete diagnostic evidence in [the correction](followup-disposition.md), not a completed confirmation. Ten-state gains do not erase the original full157 regression. Final full157 remains deferred until a stable verified ten-state candidate is near target.

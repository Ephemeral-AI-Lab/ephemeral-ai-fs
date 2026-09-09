# retained-chain-1: ten-snapshot measured result

Final allocated storage is **50,360,320 bytes**, growth **50,290,688 bytes** from **69,632 bytes**. The 45,000,000-byte objective is not met; the exact difference is **+5,360,320 bytes**. This is not near-target. No release-admission PASS or full157 confirmation is claimed.

Actual outcomes: **10 Created / 0 UpToDate**. Same-Store verification checked **10 states, 58,860 path states, 327,885,165 logical bytes**; cleanup was {'performance': 'PASS', 'verification': 'PASS'}. Frozen performance and post-verification manifests are separate lifecycle points. Verification checked the measured digest before reopening the SAME Store in a new coordinator; no replacement history was built.

The fixed full157 indices are **1, 18, 36, 53, 70, 88, 105, 122, 140, 157**. Skipped checkpoints are not replayed. Controls retain the [applicability audit](baseline-applicability-45mb.md). Git retains these ten trees with narrower metadata; its construction/packing timers do not match foreground save latency.

| Arm | Final allocated B | Candidate difference B |
| --- | --- | --- |
| Git | 38,223,872 | +12,136,448 |
| released v014 | 67,145,728 | -16,785,408 |
| original v015 | 66,105,344 | -15,745,024 |
| chain-1 | 56,668,160 | -6,307,840 |
| selected-full-1 | 54,562,816 | -4,202,496 |

**Rejected and reverted in `b77f29432` and `148fdc7f3`.** Relative to retained-FULL, allocation grew **1,040,384 B** and content packs grew **326,391 B**. Commit median rose from 0.571987792 to 0.6492450625 seconds; sum rose from 7.178636126 to 7.938355291 seconds. The five-family diagnostic established an eligible similar base, but the full product measurement did not justify this policy. Exact same-Store verification and cleanup succeeded; this is an optimization rejection.

The [retained product patch](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-product.patch), measured source `01204acaf`, binary/image identities and all evidence are retained. Product and harness now match measured `ee78028ba` byte-for-byte. No unchanged restored history was rerun.

## Public timings and resource tradeoffs

| Metric | Min s | Median s | Max s | Sum s | Median vs original v015 |
| --- | --- | --- | --- | --- | --- |
| exec_ns | 0.161509041 | 2.920919437 | 7.368790083 | 34.994654041 | -2.06% |
| commit_ns | 0.038052250 | 0.649245063 | 1.673612916 | 7.938355291 | +33.59% |
| paired_ns | 0.199561291 | 3.570164500 | 9.042402999 | 42.933009332 | +2.94% |
| verification_step_wall_ns | 0.164388667 | 2.440712438 | 4.387327500 | 23.363061208 | +10.02% |

These are ten dependent history steps. Min/max and per-step observations are descriptive; no tail confidence is inferred. The prospective <=10% speed/resource comparison and separate 8-MiB RSS allowance are engineering criteria, not owner-approved release gates.

| Scope | Seconds |
| --- | --- |
| performance_wall_ns | 58.821740083 |
| performance_work_wall_ns | 53.935805792 |
| verification_wall_ns | 28.067754209 |
| verification_work_wall_ns | 26.712818708 |
| setup_ns | 4.374753333 |
| verification_setup_ns | 0.971087875 |
| cleanup_ns | 0.474072167 |
| verification_cleanup_ns | 0.378720375 |
| preparation_ns | 2.950852166 |
| verification_preparation_ns | 3.108149500 |
| transfer_ns | 7.413329918 |

Preparation, setup, transfer, cleanup and case walls have their existing nested boundaries; do not add all rows as disjoint costs. Build and complete command receipts are below.

| Step | Save s | Commit s | Paired s | Verify step s | Allocated B |
| --- | --- | --- | --- | --- | --- |
| 1 | 0.161509041 | 0.038052250 | 0.199561291 | 0.164388667 | 684032 |
| 2 | 0.914257000 | 0.211931000 | 1.126188000 | 0.802434666 | 4222976 |
| 3 | 1.454444750 | 0.332169667 | 1.786614417 | 1.126600167 | 7368704 |
| 4 | 2.322179375 | 0.527705250 | 2.849884625 | 1.675528125 | 11563008 |
| 5 | 3.081478250 | 0.690813583 | 3.772291833 | 2.341079375 | 16805888 |
| 6 | 2.760360625 | 0.607676542 | 3.368037167 | 2.540345500 | 21000192 |
| 7 | 4.882348750 | 1.104961958 | 5.987310708 | 2.954585583 | 27291648 |
| 8 | 5.706535375 | 1.273067625 | 6.979603000 | 3.342784000 | 33583104 |
| 9 | 6.342750792 | 1.478364500 | 7.821115292 | 4.027987625 | 41971712 |
| 10 | 7.368790083 | 1.673612916 | 9.042402999 | 4.387327500 | 50360320 |

Full source-to-commit mappings, CPU/transfer values and original-oracle counts are in [retained-chain-1-comparison.csv](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-comparison.csv).

Resource values retain their actual scopes: host lifetime peak RSS, cumulative process CPU, and boundary samples for cgroup categories/staging/spool. Boundary maxima are not simultaneous samples or continuous phase-local peaks.

```json
{
  "performance": {
    "host_lifetime_peak_rss_bytes": 112132096,
    "host_max_boundary_rss_bytes": 112132096,
    "host_cpu_ns": 15813989915,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 83275776,
      "memory_peak": 141557760,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 454,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 10258383,
      "throttled_usec": 0,
      "usage_usec": 16032600,
      "user_usec": 5774216
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 60870656,
      "file": 7016448,
      "file_dirty": 32768,
      "file_writeback": 0,
      "kernel": 14749696,
      "shmem": 0,
      "slab": 14119032
    },
    "spool_max_boundary_bytes": 45056,
    "staging_max_boundary_bytes": 71393280
  },
  "verification": {
    "host_lifetime_peak_rss_bytes": 70139904,
    "host_max_boundary_rss_bytes": 70123520,
    "host_cpu_ns": 8546542873,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 140513280,
      "memory_peak": 244793344,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 268,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 5144527,
      "throttled_usec": 0,
      "usage_usec": 9758167,
      "user_usec": 4613639
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 135987200,
      "file": 1970176,
      "file_dirty": 8192,
      "file_writeback": 0,
      "kernel": 1363968,
      "shmem": 0,
      "slab": 649896
    },
    "spool_max_boundary_bytes": 40960,
    "staging_max_boundary_bytes": 1966080
  }
}
```

## Physical reconciliation

| Category | Bytes |
| --- | --- |
| all_pack_bytes | 44,780,122 |
| file_content_pack_bytes | 38,205,878 |
| filesystem_allocation_difference_bytes | 851,968 |
| metadata_legacy_pack_bytes | 6,574,244 |
| sqlite_logical_bytes | 49,508,352 |
| sqlite_nonpack_bytes | 4,728,230 |
| store_allocated_bytes | 50,360,320 |

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
      "LFS4INT": 415,
      "LFS4LNK": 7,
      "LFS4MAP": 124,
      "LFS4MET": 4,
      "LFS4NSP": 4145
    },
    "metadata_legacy_DELTA": {
      "canonical_bytes": 1647860,
      "objects": 279,
      "raw_bytes": 1647860,
      "record_bytes": 1010275,
      "selected": 279
    },
    "metadata_legacy_FULL": {
      "canonical_bytes": 8546926,
      "objects": 46008,
      "raw_bytes": 8546926,
      "record_bytes": 8592934,
      "selected": 46008
    },
    "pack_v1": {
      "bytes": 6574244,
      "decoded_group_bytes": 9791533,
      "encoded_group_bytes": 6559156,
      "groups": 794,
      "header_directory_bytes": 15088,
      "packs": 149,
      "record_directory_bytes": 188324
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
      "bytes": 34946386,
      "decoded_group_bytes": 34406690,
      "encoded_group_bytes": 34406690,
      "groups": 33217,
      "header_directory_bytes": 539696,
      "packs": 514
    },
    "small_DELTA": {
      "canonical_bytes": 189582139,
      "frame_bytes": 12468030,
      "objects": 24821,
      "raw_bytes": 189011256,
      "record_bytes": 13485691,
      "record_header_bytes": 1017661,
      "selected": 24821
    },
    "small_FULL": {
      "canonical_bytes": 56989722,
      "frame_bytes": 20845435,
      "objects": 8396,
      "raw_bytes": 56796614,
      "record_bytes": 20920999,
      "record_header_bytes": 75564,
      "selected": 8396
    },
    "small_depth_counts": {
      "0": 8396,
      "1": 6692,
      "2": 5023,
      "3": 3748,
      "4": 2853,
      "5": 2184,
      "6": 1720,
      "7": 1368,
      "8": 1233
    },
    "small_physical_DELTA_bases": {
      "count": 15089,
      "raw_bytes": 121670735,
      "record_bytes": 9447286
    },
    "small_physical_FULL_bases": {
      "count": 5793,
      "raw_bytes": 42764929,
      "record_bytes": 15818653
    },
    "small_physical_bases": {
      "count": 20882,
      "raw_bytes": 164435664,
      "record_bytes": 25265939
    },
    "small_record_kinds": {
      "0": 8396,
      "1": 1087,
      "2": 23734
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
      "unused": 381
    },
    {
      "bytes": 1941504,
      "name": "object_packs",
      "pages": 474,
      "pagetype": "leaf",
      "payload": 1456529,
      "unused": 473525
    },
    {
      "bytes": 43393024,
      "name": "object_packs",
      "pages": 10594,
      "pagetype": "overflow",
      "payload": 43327229,
      "unused": 23419
    },
    {
      "bytes": 57344,
      "name": "objects",
      "pages": 14,
      "pagetype": "internal",
      "payload": 42188,
      "unused": 8072
    },
    {
      "bytes": 4050944,
      "name": "objects",
      "pages": 989,
      "pagetype": "leaf",
      "payload": 3385294,
      "unused": 419985
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
    "page_count": 12087,
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
    "LAYERFS_PRODUCT_SEAL": "80626b3a096f62b9fa583a3e07b7c9a6d88650c977b4a4c24cf847ba087e61d5",
    "LAYERFS_SOURCE_COMMIT": "01204acaf1f4b0f3e2aae9ab27c0c578b249e4da",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "49992c0d2102fbc7aee1dee697ab2aa4eba76c362352817ae499990068343b29",
    "LAYERFS_SOURCE_TREE": "e9d6340467350b9624b69fe4b0e06464ba6d9854",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127"
  },
  "host": {
    "LAYERFS_PRODUCT_SEAL": "80626b3a096f62b9fa583a3e07b7c9a6d88650c977b4a4c24cf847ba087e61d5",
    "LAYERFS_SOURCE_COMMIT": "01204acaf1f4b0f3e2aae9ab27c0c578b249e4da",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "49992c0d2102fbc7aee1dee697ab2aa4eba76c362352817ae499990068343b29",
    "LAYERFS_SOURCE_TREE": "e9d6340467350b9624b69fe4b0e06464ba6d9854",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127",
    "binary_sha256": "bb61cbecb153792d9cb212733f586ca3570aa1a4b10764d0f5c181dd9390fbdf",
    "platform": "macOS-26.4.1-arm64-arm-64bit-Mach-O",
    "rust_toolchain": "1.85.1",
    "schema_sha256": "7ed3355be81cfdb82839d1651ee0919afc445ce6255c522f2b1bab54c9a92780"
  },
  "image": "sha256:602f93924c267e158f0dbd55b5be12c343cac9c7264f1e684357ce688d70a2d0",
  "fixture_digest": "eeb408e63b091aa7379aeefd1e3fbf819cdb4d882c67c03dd040c9fbe04feabb",
  "contract_digest": "9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50",
  "frozen_store_sha256": "9fdbb2d1eacd3ccace5ff9bd1e877c5fb3a06cec48cce9e2751b149ddb380852",
  "census_script_sha256": "dcd343150bb137ad30ef2dd87ede9bb86abac8a74065e7db2622efd95cd30440"
}
```

Original fixture manifest SHA256: `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Prepared inputs: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`.

- [retained-chain-1-source.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-source.json); SHA256 `8d6b24e26e2cd57bc5f4f7e227e3967fdd45f067754ab6be7d9f9c920b88a83b`
- [retained-chain-1-dirty.patch](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-dirty.patch); SHA256 `3515545d955d5f30812c017469c5427fd10d8a5d46d29d61fc5b555868540d37`
- [retained-chain-1-fs-benchmark-pro.identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-fs-benchmark-pro.identity.json); SHA256 `cf6653f615bae3c41d4d589328cafe0609592d21f6f80ec3975e401ee265d277`
- [retained-chain-1-image-inspection.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-image-inspection.json); SHA256 `f7df783e8feb4e354716c72c9bc14ca075628a4f1ea3e23a05b13d097e070f12`

**build-host**: 57.875506875 s, exit 0. [retained-chain-1-build-host-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-build-host-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-host"
]
```

**build-image**: 82.198234542 s, exit 0. [retained-chain-1-build-image-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-build-image-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-storage-smoke-image"
]
```

**performance**: 61.979930125 s, exit 0. [retained-chain-1-performance-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-performance-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:49992c0d2102fbc7",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--output",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1"
]
```

**census**: 0.755749334 s, exit 0. [retained-chain-1-census-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-census-command.json)

```json
[
  "python3",
  "docs/roadmap/0.1/0.1.5/issue100/census.py",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1"
]
```

**verification**: 31.401294917 s, exit 0. [retained-chain-1-verification-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-verification-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:49992c0d2102fbc7",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--storage-verify-run",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1"
]
```

- [identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1/identity.json)
- [performance-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1/performance-manifest.json)
- [verification-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1/verification-manifest.json)
- [census.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1/census.json)
- [retained-chain-1-comparison.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-chain-1-comparison.json)

No broad Cargo/Clippy/doctest or unrelated qualification suite, release/tag, GC/repacking or unchanged-control rerun is claimed. The premature chain full157 execution remains incomplete diagnostic evidence in [the correction](followup-disposition.md), not a completed confirmation. Ten-state gains do not erase the original full157 regression. Final full157 remains deferred until a stable verified ten-state candidate is near target.

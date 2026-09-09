# retained-candidate-1: ten-snapshot measured result

Final allocated storage is **49,319,936 bytes**, growth **49,250,304 bytes** from **69,632 bytes**. The 45,000,000-byte objective is not met; the exact difference is **+4,319,936 bytes**. This is not near-target. No release-admission PASS or full157 confirmation is claimed.

Actual outcomes: **10 Created / 0 UpToDate**. Same-Store verification checked **10 states, 58,860 path states, 327,885,165 logical bytes**; cleanup was {'performance': 'PASS', 'verification': 'PASS'}. Frozen performance and post-verification manifests are separate lifecycle points. Verification checked the measured digest before reopening the SAME Store in a new coordinator; no replacement history was built.

The fixed full157 indices are **1, 18, 36, 53, 70, 88, 105, 122, 140, 157**. Skipped checkpoints are not replayed. Controls retain the [applicability audit](baseline-applicability-45mb.md). Git retains these ten trees with narrower metadata; its construction/packing timers do not match foreground save latency.

| Arm | Final allocated B | Candidate difference B |
| --- | --- | --- |
| Git | 38,223,872 | +11,096,064 |
| released v014 | 67,145,728 | -17,825,792 |
| original v015 | 66,105,344 | -16,785,408 |
| chain-1 | 56,668,160 | -7,348,224 |
| selected-full-1 | 54,562,816 | -5,242,880 |

**Kept as the best measured implementation, with the target unresolved.** The retained-FULL handoff reduces content packs by **276,862 B** versus compact-candidate-1. Allocation falls **1,048,576 B**, including a 786,432-B decrease in filesystem allocation difference; do not attribute that whole reduction to compressed frames. The combined current product is 16,785,408 B smaller than original v0.1.5.

Commit median/sum exceed the prospective 10% working criterion, while save/paired medians remain slightly below baseline and performance/verification case walls increase about4.4%. The later retained-DELTA experiment was worse and reverted. The measured product is restored byte-for-byte; full157 is deferred because this result is outside near-target. See [the consolidated outcome](storage-optimization-results.md).

## Public timings and resource tradeoffs

| Metric | Min s | Median s | Max s | Sum s | Median vs original v015 |
| --- | --- | --- | --- | --- | --- |
| exec_ns | 0.155065000 | 2.877358249 | 7.983091375 | 35.702063250 | -3.52% |
| commit_ns | 0.038712167 | 0.571987792 | 1.657021500 | 7.178636126 | +17.70% |
| paired_ns | 0.193777167 | 3.449346042 | 9.640112875 | 42.880699376 | -0.55% |
| verification_step_wall_ns | 0.178329667 | 2.249809292 | 4.361261000 | 22.587782624 | +1.41% |

These are ten dependent history steps. Min/max and per-step observations are descriptive; no tail confidence is inferred. The prospective <=10% speed/resource comparison and separate 8-MiB RSS allowance are engineering criteria, not owner-approved release gates.

| Scope | Seconds |
| --- | --- |
| performance_wall_ns | 58.616814833 |
| performance_work_wall_ns | 54.569486208 |
| verification_wall_ns | 27.655753750 |
| verification_work_wall_ns | 26.187865416 |
| setup_ns | 3.403733083 |
| verification_setup_ns | 1.089283917 |
| cleanup_ns | 0.597444875 |
| verification_cleanup_ns | 0.370338750 |
| preparation_ns | 2.980092916 |
| verification_preparation_ns | 3.268211917 |
| transfer_ns | 7.578186084 |

Preparation, setup, transfer, cleanup and case walls have their existing nested boundaries; do not add all rows as disjoint costs. Build and complete command receipts are below.

| Step | Save s | Commit s | Paired s | Verify step s | Allocated B |
| --- | --- | --- | --- | --- | --- |
| 1 | 0.155065000 | 0.038712167 | 0.193777167 | 0.178329667 | 692224 |
| 2 | 0.817925000 | 0.190154125 | 1.008079125 | 0.737189833 | 4231168 |
| 3 | 1.411108542 | 0.292553625 | 1.703662167 | 1.088895833 | 7376896 |
| 4 | 2.274160417 | 0.443838500 | 2.717998917 | 1.628581042 | 11571200 |
| 5 | 3.122992333 | 0.624653709 | 3.747646042 | 2.059005125 | 16814080 |
| 6 | 2.631724166 | 0.519321875 | 3.151046041 | 2.440613458 | 21008384 |
| 7 | 4.737789625 | 0.970014291 | 5.707803916 | 2.688672750 | 27299840 |
| 8 | 5.857543167 | 1.105095750 | 6.962638917 | 3.211256583 | 33591296 |
| 9 | 6.710663625 | 1.337270584 | 8.047934209 | 4.193977333 | 41979904 |
| 10 | 7.983091375 | 1.657021500 | 9.640112875 | 4.361261000 | 49319936 |

Full source-to-commit mappings, CPU/transfer values and original-oracle counts are in [retained-candidate-1-comparison.csv](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-comparison.csv).

Resource values retain their actual scopes: host lifetime peak RSS, cumulative process CPU, and boundary samples for cgroup categories/staging/spool. Boundary maxima are not simultaneous samples or continuous phase-local peaks.

```json
{
  "performance": {
    "host_lifetime_peak_rss_bytes": 116424704,
    "host_max_boundary_rss_bytes": 116424704,
    "host_cpu_ns": 15183215417,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 80953344,
      "memory_peak": 138260480,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 464,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 10518073,
      "throttled_usec": 0,
      "usage_usec": 16211149,
      "user_usec": 5693076
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 65277952,
      "file": 45056,
      "file_dirty": 32768,
      "file_writeback": 0,
      "kernel": 14663680,
      "shmem": 0,
      "slab": 14023632
    },
    "spool_max_boundary_bytes": 45056,
    "staging_max_boundary_bytes": 71393280
  },
  "verification": {
    "host_lifetime_peak_rss_bytes": 69861376,
    "host_max_boundary_rss_bytes": 69844992,
    "host_cpu_ns": 8078623453,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 134541312,
      "memory_peak": 239165440,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 264,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 4844926,
      "throttled_usec": 0,
      "usage_usec": 9334604,
      "user_usec": 4489678
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 130674688,
      "file": 1982464,
      "file_dirty": 20480,
      "file_writeback": 0,
      "kernel": 1343488,
      "shmem": 0,
      "slab": 652856
    },
    "spool_max_boundary_bytes": 40960,
    "staging_max_boundary_bytes": 1966080
  }
}
```

## Physical reconciliation

| Category | Bytes |
| --- | --- |
| all_pack_bytes | 44,430,069 |
| file_content_pack_bytes | 37,879,487 |
| filesystem_allocation_difference_bytes | 106,496 |
| metadata_legacy_pack_bytes | 6,550,582 |
| sqlite_logical_bytes | 49,213,440 |
| sqlite_nonpack_bytes | 4,783,371 |
| store_allocated_bytes | 49,319,936 |

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
      "LFS4INT": 412,
      "LFS4LNK": 7,
      "LFS4MAP": 124,
      "LFS4MET": 4,
      "LFS4NSP": 4145
    },
    "metadata_legacy_DELTA": {
      "canonical_bytes": 1669092,
      "objects": 283,
      "raw_bytes": 1669092,
      "record_bytes": 1009533,
      "selected": 283
    },
    "metadata_legacy_FULL": {
      "canonical_bytes": 8525802,
      "objects": 46005,
      "raw_bytes": 8525802,
      "record_bytes": 8571807,
      "selected": 46005
    },
    "pack_v1": {
      "bytes": 6550582,
      "decoded_group_bytes": 9769620,
      "encoded_group_bytes": 6535686,
      "groups": 782,
      "header_directory_bytes": 14896,
      "packs": 149,
      "record_directory_bytes": 188280
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
      "bytes": 34619995,
      "decoded_group_bytes": 34080299,
      "encoded_group_bytes": 34080299,
      "groups": 33217,
      "header_directory_bytes": 539696,
      "packs": 514
    },
    "small_DELTA": {
      "canonical_bytes": 191644964,
      "frame_bytes": 12732768,
      "objects": 25392,
      "raw_bytes": 191060948,
      "record_bytes": 13773840,
      "record_header_bytes": 1041072,
      "selected": 25392
    },
    "small_FULL": {
      "canonical_bytes": 54926897,
      "frame_bytes": 20236034,
      "objects": 7825,
      "raw_bytes": 54746922,
      "record_bytes": 20306459,
      "record_header_bytes": 70425,
      "selected": 7825
    },
    "small_depth_counts": {
      "0": 7825,
      "1": 9602,
      "2": 6337,
      "3": 4097,
      "4": 2549,
      "5": 1550,
      "6": 788,
      "7": 341,
      "8": 128
    },
    "small_physical_DELTA_bases": {
      "count": 15703,
      "raw_bytes": 122563259,
      "record_bytes": 9576641
    },
    "small_physical_FULL_bases": {
      "count": 5524,
      "raw_bytes": 42261666,
      "record_bytes": 15634304
    },
    "small_physical_bases": {
      "count": 21227,
      "raw_bytes": 164824925,
      "record_bytes": 25210945
    },
    "small_record_kinds": {
      "0": 7825,
      "1": 4363,
      "2": 21029
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
      "unused": 263
    },
    {
      "bytes": 2002944,
      "name": "object_packs",
      "pages": 489,
      "pagetype": "leaf",
      "payload": 1485599,
      "unused": 505781
    },
    {
      "bytes": 43012096,
      "name": "object_packs",
      "pages": 10501,
      "pagetype": "overflow",
      "payload": 42948106,
      "unused": 21986
    },
    {
      "bytes": 61440,
      "name": "objects",
      "pages": 15,
      "pagetype": "internal",
      "payload": 42412,
      "unused": 11897
    },
    {
      "bytes": 4071424,
      "name": "objects",
      "pages": 994,
      "pagetype": "leaf",
      "payload": 3385018,
      "unused": 440713
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
    "page_count": 12015,
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
    "LAYERFS_PRODUCT_SEAL": "24cde1dce88104daebf2d01e6b711665cfa52c52880b31157fbcfa5c27214673",
    "LAYERFS_SOURCE_COMMIT": "ee78028ba56741002a627b3a8875d3c888c86708",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "2941cd53eab45065b5b2461479805852d5779a4d0c3515a2f2c04baa64097196",
    "LAYERFS_SOURCE_TREE": "c5187d4eccc631895ac942cc0411c70364b12f69",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127"
  },
  "host": {
    "LAYERFS_PRODUCT_SEAL": "24cde1dce88104daebf2d01e6b711665cfa52c52880b31157fbcfa5c27214673",
    "LAYERFS_SOURCE_COMMIT": "ee78028ba56741002a627b3a8875d3c888c86708",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "2941cd53eab45065b5b2461479805852d5779a4d0c3515a2f2c04baa64097196",
    "LAYERFS_SOURCE_TREE": "c5187d4eccc631895ac942cc0411c70364b12f69",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127",
    "binary_sha256": "98d7f8b99466a283b371cfb32b2113168ef44c939aa08cdc47d2b9e9e4d59086",
    "platform": "macOS-26.4.1-arm64-arm-64bit-Mach-O",
    "rust_toolchain": "1.85.1",
    "schema_sha256": "7ed3355be81cfdb82839d1651ee0919afc445ce6255c522f2b1bab54c9a92780"
  },
  "image": "sha256:158f34d7af6e21123e713a661a6e7644bdfcae15a00a76c663b7af04592c94bf",
  "fixture_digest": "eeb408e63b091aa7379aeefd1e3fbf819cdb4d882c67c03dd040c9fbe04feabb",
  "contract_digest": "9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50",
  "frozen_store_sha256": "70349c2f906c29f80102533342e302fd5ee00ebd78b8d2c9ae7e96054fe2145a",
  "census_script_sha256": "dcd343150bb137ad30ef2dd87ede9bb86abac8a74065e7db2622efd95cd30440"
}
```

Original fixture manifest SHA256: `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Prepared inputs: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`.

- [retained-candidate-1-source.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-source.json); SHA256 `820cc72f281e558c8db8a5197cdf21229c497ed58135f96bafb4df31a768e69c`
- [retained-candidate-1-dirty.patch](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-dirty.patch); SHA256 `3515545d955d5f30812c017469c5427fd10d8a5d46d29d61fc5b555868540d37`
- [retained-candidate-1-fs-benchmark-pro.identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-fs-benchmark-pro.identity.json); SHA256 `d11433b5f892191ccc7028664484be1f893e0e5c06addd92382f3acbc3e56202`
- [retained-candidate-1-image-inspection.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-image-inspection.json); SHA256 `6894e8a2c1a5f050872fa375c050425fc974e3c10fcf9fd7cd41366dc45f8296`

**build-host**: 57.111774709 s, exit 0. [retained-candidate-1-build-host-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-build-host-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-host"
]
```

**build-image**: 65.964735541 s, exit 0. [retained-candidate-1-build-image-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-build-image-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-storage-smoke-image"
]
```

**performance**: 61.823635208 s, exit 0. [retained-candidate-1-performance-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-performance-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:2941cd53eab45065",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--output",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1"
]
```

**census**: 0.813206209 s, exit 0. [retained-candidate-1-census-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-census-command.json)

```json
[
  "python3",
  "docs/roadmap/0.1/0.1.5/issue100/census.py",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1"
]
```

**verification**: 31.168617250 s, exit 0. [retained-candidate-1-verification-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-verification-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:2941cd53eab45065",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--storage-verify-run",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1"
]
```

- [identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1/identity.json)
- [performance-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1/performance-manifest.json)
- [verification-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1/verification-manifest.json)
- [census.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1/census.json)
- [retained-candidate-1-comparison.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-comparison.json)

No broad Cargo/Clippy/doctest or unrelated qualification suite, release/tag, GC/repacking or unchanged-control rerun is claimed. The premature chain full157 execution remains incomplete diagnostic evidence in [the correction](followup-disposition.md), not a completed confirmation. Ten-state gains do not erase the original full157 regression. Final full157 remains deferred until a stable verified ten-state candidate is near target.

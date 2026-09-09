# selected-full-1: ten-snapshot measured result

Final allocated storage is **54,562,816 bytes**, growth **54,493,184 bytes** from **69,632 bytes**. The 45,000,000-byte objective is not met; the exact difference is **+9,562,816 bytes**. This is not near-target. No release-admission PASS or full157 confirmation is claimed.

Actual outcomes: **10 Created / 0 UpToDate**. Same-Store verification checked **10 states, 58,860 path states, 327,885,165 logical bytes**; cleanup was {'performance': 'PASS', 'verification': 'PASS'}. Frozen performance and post-verification manifests are separate lifecycle points. Verification checked the measured digest before reopening the SAME Store in a new coordinator; no replacement history was built.

The fixed full157 indices are **1, 18, 36, 53, 70, 88, 105, 122, 140, 157**. Skipped checkpoints are not replayed. Controls retain the [applicability audit](baseline-applicability-45mb.md). Git retains these ten trees with narrower metadata; its construction/packing timers do not match foreground save latency.

| Arm | Final allocated B | Candidate difference B |
| --- | --- | --- |
| Git | 38,223,872 | +16,338,944 |
| released v014 | 67,145,728 | -12,582,912 |
| original v015 | 66,105,344 | -11,542,528 |
| chain-1 | 56,668,160 | -2,105,344 |
| selected-full-1 | 54,562,816 | +0 |

The selected-FULL cache cut allocation by **2,105,344 B** versus chain-1. Content-pack bytes fell by **1,324,387 B**; the remainder includes metadata/index/allocation changes. Its Commit median rose **12.02%** and Commit sum **13.34%** versus original v0.1.5, exceeding the prospective 10% working criterion. Save/paired medians, case walls and host lifetime RSS improved. This is a measured storage/speed tradeoff, not an unconditional performance pass.

## Public timings and resource tradeoffs

| Metric | Min s | Median s | Max s | Sum s | Median vs original v015 |
| --- | --- | --- | --- | --- | --- |
| exec_ns | 0.167518625 | 2.737773791 | 7.251036000 | 34.743671084 | -8.20% |
| commit_ns | 0.036202000 | 0.544395209 | 1.374976291 | 6.580561250 | +12.02% |
| paired_ns | 0.203720625 | 3.282169000 | 8.626012291 | 41.324232334 | -5.37% |
| verification_step_wall_ns | 0.172637916 | 2.056906979 | 4.203478167 | 21.041632124 | -7.28% |

These are ten dependent history steps. Min/max and per-step observations are descriptive; no tail confidence is inferred. The prospective <=10% speed/resource comparison and separate 8-MiB RSS allowance are engineering criteria, not owner-approved release gates.

| Scope | Seconds |
| --- | --- |
| performance_wall_ns | 54.892444000 |
| performance_work_wall_ns | 51.462467500 |
| verification_wall_ns | 25.196761750 |
| verification_work_wall_ns | 23.934586042 |
| setup_ns | 2.901082666 |
| verification_setup_ns | 0.876932709 |
| cleanup_ns | 0.491580000 |
| verification_cleanup_ns | 0.380912375 |
| preparation_ns | 11.026444500 |
| verification_preparation_ns | 3.057088459 |
| transfer_ns | 6.920401958 |

Preparation, setup, transfer, cleanup and case walls have their existing nested boundaries; do not add all rows as disjoint costs. Build and complete command receipts are below.

| Step | Save s | Commit s | Paired s | Verify step s | Allocated B |
| --- | --- | --- | --- | --- | --- |
| 1 | 0.167518625 | 0.036202000 | 0.203720625 | 0.172637916 | 692224 |
| 2 | 1.057553542 | 0.184907208 | 1.242460750 | 0.723760333 | 4231168 |
| 3 | 1.676204542 | 0.279066292 | 1.955270834 | 1.054476958 | 7376896 |
| 4 | 2.503547125 | 0.448163959 | 2.951711084 | 1.467209959 | 12619776 |
| 5 | 2.882816083 | 0.574376167 | 3.457192250 | 1.883891291 | 17862656 |
| 6 | 2.592731500 | 0.514414250 | 3.107145750 | 2.229922667 | 22056960 |
| 7 | 4.831217083 | 0.886857500 | 5.718074583 | 2.639801791 | 29396992 |
| 8 | 5.354449959 | 1.055676125 | 6.410126084 | 2.973384625 | 36737024 |
| 9 | 6.426596625 | 1.225921458 | 7.652518083 | 3.693068417 | 45125632 |
| 10 | 7.251036000 | 1.374976291 | 8.626012291 | 4.203478167 | 54562816 |

Full source-to-commit mappings, CPU/transfer values and original-oracle counts are in [selected-full-1-comparison.csv](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-comparison.csv).

Resource values retain their actual scopes: host lifetime peak RSS, cumulative process CPU, and boundary samples for cgroup categories/staging/spool. Boundary maxima are not simultaneous samples or continuous phase-local peaks.

```json
{
  "performance": {
    "host_lifetime_peak_rss_bytes": 114294784,
    "host_max_boundary_rss_bytes": 114294784,
    "host_cpu_ns": 13524653452,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 87732224,
      "memory_peak": 144166912,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 445,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 10831162,
      "throttled_usec": 0,
      "usage_usec": 16756880,
      "user_usec": 5925718
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 64700416,
      "file": 7028736,
      "file_dirty": 40960,
      "file_writeback": 0,
      "kernel": 14770176,
      "shmem": 0,
      "slab": 14125544
    },
    "spool_max_boundary_bytes": 45056,
    "staging_max_boundary_bytes": 71393280
  },
  "verification": {
    "host_lifetime_peak_rss_bytes": 70156288,
    "host_max_boundary_rss_bytes": 70139904,
    "host_cpu_ns": 7094554918,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 135540736,
      "memory_peak": 239599616,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 242,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 5123924,
      "throttled_usec": 0,
      "usage_usec": 9545615,
      "user_usec": 4421690
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 130920448,
      "file": 1974272,
      "file_dirty": 12288,
      "file_writeback": 0,
      "kernel": 1351680,
      "shmem": 0,
      "slab": 653400
    },
    "spool_max_boundary_bytes": 40960,
    "staging_max_boundary_bytes": 1966080
  }
}
```

## Physical reconciliation

| Category | Bytes |
| --- | --- |
| all_pack_bytes | 49,832,736 |
| file_content_pack_bytes | 43,277,929 |
| filesystem_allocation_difference_bytes | 12,288 |
| metadata_legacy_pack_bytes | 6,554,807 |
| sqlite_logical_bytes | 54,550,528 |
| sqlite_nonpack_bytes | 4,717,792 |
| store_allocated_bytes | 54,562,816 |

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
      "LFS4INT": 409,
      "LFS4LNK": 7,
      "LFS4MAP": 124,
      "LFS4MET": 4,
      "LFS4NSP": 4145
    },
    "metadata_legacy_DELTA": {
      "canonical_bytes": 1679992,
      "objects": 282,
      "raw_bytes": 1679992,
      "record_bytes": 1018936,
      "selected": 282
    },
    "metadata_legacy_FULL": {
      "canonical_bytes": 8514470,
      "objects": 46002,
      "raw_bytes": 8514470,
      "record_bytes": 8560472,
      "selected": 46002
    },
    "pack_v1": {
      "bytes": 6554807,
      "decoded_group_bytes": 9767704,
      "encoded_group_bytes": 6539783,
      "groups": 790,
      "header_directory_bytes": 15024,
      "packs": 149,
      "record_directory_bytes": 188296
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
      "bytes": 40018437,
      "decoded_group_bytes": 39478693,
      "encoded_group_bytes": 39478693,
      "groups": 33217,
      "header_directory_bytes": 539744,
      "packs": 517
    },
    "small_DELTA": {
      "canonical_bytes": 172820783,
      "frame_bytes": 10967881,
      "objects": 22370,
      "raw_bytes": 172306273,
      "record_bytes": 11885051,
      "record_header_bytes": 917170,
      "selected": 22370
    },
    "small_FULL": {
      "canonical_bytes": 73751078,
      "frame_bytes": 27496019,
      "objects": 10847,
      "raw_bytes": 73501597,
      "record_bytes": 27593642,
      "record_header_bytes": 97623,
      "selected": 10847
    },
    "small_depth_counts": {
      "0": 10847,
      "1": 9417,
      "2": 5542,
      "3": 3409,
      "4": 1989,
      "5": 1124,
      "6": 556,
      "7": 243,
      "8": 90
    },
    "small_physical_DELTA_bases": {
      "count": 12935,
      "raw_bytes": 105622381,
      "record_bytes": 7874999
    },
    "small_physical_FULL_bases": {
      "count": 5912,
      "raw_bytes": 45339119,
      "record_bytes": 16202810
    },
    "small_physical_bases": {
      "count": 18847,
      "raw_bytes": 150961500,
      "record_bytes": 24077809
    },
    "small_record_kinds": {
      "0": 10847,
      "1": 3824,
      "2": 18546
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
      "unused": 400
    },
    {
      "bytes": 1933312,
      "name": "object_packs",
      "pages": 472,
      "pagetype": "leaf",
      "payload": 1468830,
      "unused": 453033
    },
    {
      "bytes": 48439296,
      "name": "object_packs",
      "pages": 11826,
      "pagetype": "overflow",
      "payload": 48367556,
      "unused": 24436
    },
    {
      "bytes": 57344,
      "name": "objects",
      "pages": 14,
      "pagetype": "internal",
      "payload": 42225,
      "unused": 8028
    },
    {
      "bytes": 4055040,
      "name": "objects",
      "pages": 990,
      "pagetype": "leaf",
      "payload": 3384898,
      "unused": 424481
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
    "page_count": 13318,
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
    "LAYERFS_PRODUCT_SEAL": "606354a8a6ff2bdc041d06ee8bbe0c19b8810dbeb59b98e567535a55798dfd90",
    "LAYERFS_SOURCE_COMMIT": "45ec8918cd9da1899bed5181c5f5dc334b6f3d07",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "9a13bce557fcf895a8b0cce442899b1027dbc57a0a9eab521182d9ac45e24fbb",
    "LAYERFS_SOURCE_TREE": "f3f1a6bad47635892026fdffcfe7c7f2f592c10b",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127"
  },
  "host": {
    "LAYERFS_PRODUCT_SEAL": "606354a8a6ff2bdc041d06ee8bbe0c19b8810dbeb59b98e567535a55798dfd90",
    "LAYERFS_SOURCE_COMMIT": "45ec8918cd9da1899bed5181c5f5dc334b6f3d07",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "9a13bce557fcf895a8b0cce442899b1027dbc57a0a9eab521182d9ac45e24fbb",
    "LAYERFS_SOURCE_TREE": "f3f1a6bad47635892026fdffcfe7c7f2f592c10b",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127",
    "binary_sha256": "a4ad346e6a2acbb8c1769288076f879d90eddb351e18d89752f7725b79e22562",
    "platform": "macOS-26.4.1-arm64-arm-64bit-Mach-O",
    "rust_toolchain": "1.85.1",
    "schema_sha256": "7ed3355be81cfdb82839d1651ee0919afc445ce6255c522f2b1bab54c9a92780"
  },
  "image": "sha256:59558238261de001bb03a06e6e365e2f2b3f7ff416edc640f70cd11cb30a6f40",
  "fixture_digest": "eeb408e63b091aa7379aeefd1e3fbf819cdb4d882c67c03dd040c9fbe04feabb",
  "contract_digest": "9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50",
  "frozen_store_sha256": "336467ab956ab2501ad747d6a6e96f127bfec51d5ee33d2a7e543aee47659e61",
  "census_script_sha256": "dcd343150bb137ad30ef2dd87ede9bb86abac8a74065e7db2622efd95cd30440"
}
```

Original fixture manifest SHA256: `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Prepared inputs: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`.

- [selected-full-1-source.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-source.json); SHA256 `eec603d3d1340b0404bf6aa9b64a99fcf7d9c907cceeecf1f96df11bd1419006`
- [selected-full-1-dirty.patch](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-dirty.patch); SHA256 `b7151913d65ad42252389d24468af4d0b15dbd6728ae28a6f63677fcfc8aa07f`
- [selected-full-1-fs-benchmark-pro.identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-fs-benchmark-pro.identity.json); SHA256 `6a7ae078b327b18b51978856c3d9a385dd89c2b157d231530913eb6a4a73a74b`
- [selected-full-1-image-inspection.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-image-inspection.json); SHA256 `0f9f2f358a09f739de45ccfeaa12239d9203f98afa37ac2467e1bf2ad66baad9`

**build-host**: 56.144310750 s, exit 0. [selected-full-1-build-host-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-build-host-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-host"
]
```

**build-image**: 88.508335792 s, exit 0. [selected-full-1-build-image-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-build-image-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-storage-smoke-image"
]
```

**performance**: 66.260228000 s, exit 0. [selected-full-1-performance-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-performance-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:9a13bce557fcf895",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--output",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1"
]
```

**census**: 0.814003333 s, exit 0. [selected-full-1-census-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-census-command.json)

```json
[
  "python3",
  "docs/roadmap/0.1/0.1.5/issue100/census.py",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1"
]
```

**verification**: 28.490110791 s, exit 0. [selected-full-1-verification-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-verification-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:9a13bce557fcf895",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--storage-verify-run",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1"
]
```

- [identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1/identity.json)
- [performance-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1/performance-manifest.json)
- [verification-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1/verification-manifest.json)
- [census.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1/census.json)
- [selected-full-1-comparison.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/selected-full-1-comparison.json)

No broad Cargo/Clippy/doctest or unrelated qualification suite, release/tag, GC/repacking or unchanged-control rerun is claimed. The premature chain full157 execution remains incomplete diagnostic evidence in [the correction](followup-disposition.md), not a completed confirmation. Ten-state gains do not erase the original full157 regression. Final full157 remains deferred until a stable verified ten-state candidate is near target.

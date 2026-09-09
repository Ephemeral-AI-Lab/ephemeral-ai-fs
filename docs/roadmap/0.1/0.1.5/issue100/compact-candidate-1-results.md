# compact-candidate-1: ten-snapshot measured result

Final allocated storage is **50,368,512 bytes**, growth **50,298,880 bytes** from **69,632 bytes**. The 45,000,000-byte objective is not met; the exact difference is **+5,368,512 bytes**. This is not near-target. No release-admission PASS or full157 confirmation is claimed.

Actual outcomes: **10 Created / 0 UpToDate**. Same-Store verification checked **10 states, 58,860 path states, 327,885,165 logical bytes**; cleanup was {'performance': 'PASS', 'verification': 'PASS'}. Frozen performance and post-verification manifests are separate lifecycle points. Verification checked the measured digest before reopening the SAME Store in a new coordinator; no replacement history was built.

The fixed full157 indices are **1, 18, 36, 53, 70, 88, 105, 122, 140, 157**. Skipped checkpoints are not replayed. Controls retain the [applicability audit](baseline-applicability-45mb.md). Git retains these ten trees with narrower metadata; its construction/packing timers do not match foreground save latency.

| Arm | Final allocated B | Candidate difference B |
| --- | --- | --- |
| Git | 38,223,872 | +12,144,640 |
| released v014 | 67,145,728 | -16,777,216 |
| original v015 | 66,105,344 | -15,736,832 |
| chain-1 | 56,668,160 | -6,299,648 |
| selected-full-1 | 54,562,816 | -4,194,304 |

Index compaction saved **123,422 content-pack bytes** versus removed-base-1, but only **4,096 allocated bytes**. Commit median was 0.5813286455 seconds versus 0.5805168755 seconds, and Commit sum was 7.141591707 versus 7.167381335 seconds. This small standalone gain is not target progress sufficient to stop; the fixed compact layout is retained provisionally for the [retained-admission handoff](retained-candidate-amendment.md), which tests the missing cross-admission availability. It stores each candidate once within the same 128-KiB allowance rather than duplicating its ID/signature in up to eight lookup slots.

## Public timings and resource tradeoffs

| Metric | Min s | Median s | Max s | Sum s | Median vs original v015 |
| --- | --- | --- | --- | --- | --- |
| exec_ns | 0.155263667 | 3.235269042 | 7.734226750 | 36.422463292 | +8.48% |
| commit_ns | 0.037962083 | 0.581328645 | 1.592876167 | 7.141591707 | +19.62% |
| paired_ns | 0.193225750 | 3.816597687 | 9.327102917 | 43.564054999 | +10.04% |
| verification_step_wall_ns | 0.189750208 | 2.234937125 | 4.253856458 | 22.040650583 | +0.74% |

These are ten dependent history steps. Min/max and per-step observations are descriptive; no tail confidence is inferred. The prospective <=10% speed/resource comparison and separate 8-MiB RSS allowance are engineering criteria, not owner-approved release gates.

| Scope | Seconds |
| --- | --- |
| performance_wall_ns | 59.046753750 |
| performance_work_wall_ns | 54.900048250 |
| verification_wall_ns | 26.681905375 |
| verification_work_wall_ns | 25.251539167 |
| setup_ns | 3.570358458 |
| verification_setup_ns | 0.988035292 |
| cleanup_ns | 0.533447958 |
| verification_cleanup_ns | 0.437316833 |
| preparation_ns | 3.055430541 |
| verification_preparation_ns | 3.229816833 |
| transfer_ns | 7.568728960 |

Preparation, setup, transfer, cleanup and case walls have their existing nested boundaries; do not add all rows as disjoint costs. Build and complete command receipts are below.

| Step | Save s | Commit s | Paired s | Verify step s | Allocated B |
| --- | --- | --- | --- | --- | --- |
| 1 | 0.155263667 | 0.037962083 | 0.193225750 | 0.189750208 | 692224 |
| 2 | 0.840557667 | 0.193205375 | 1.033763042 | 0.755833875 | 4231168 |
| 3 | 1.522796916 | 0.283311000 | 1.806107916 | 1.057793583 | 7376896 |
| 4 | 2.481828750 | 0.451341250 | 2.933170000 | 1.578567917 | 11571200 |
| 5 | 3.417576084 | 0.636038333 | 4.053614417 | 2.088327375 | 16814080 |
| 6 | 3.052962000 | 0.526618958 | 3.579580958 | 2.381546875 | 21008384 |
| 7 | 4.977058292 | 0.978168625 | 5.955226917 | 2.774941042 | 27299840 |
| 8 | 5.630411291 | 1.139559916 | 6.769971207 | 3.129039209 | 34639872 |
| 9 | 6.609781875 | 1.302510000 | 7.912291875 | 3.830994041 | 43028480 |
| 10 | 7.734226750 | 1.592876167 | 9.327102917 | 4.253856458 | 50368512 |

Full source-to-commit mappings, CPU/transfer values and original-oracle counts are in [compact-candidate-1-comparison.csv](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-comparison.csv).

Resource values retain their actual scopes: host lifetime peak RSS, cumulative process CPU, and boundary samples for cgroup categories/staging/spool. Boundary maxima are not simultaneous samples or continuous phase-local peaks.

```json
{
  "performance": {
    "host_lifetime_peak_rss_bytes": 119373824,
    "host_max_boundary_rss_bytes": 119373824,
    "host_cpu_ns": 14788463666,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 83300352,
      "memory_peak": 139825152,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 470,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 11038508,
      "throttled_usec": 0,
      "usage_usec": 17314107,
      "user_usec": 6275598
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 67158016,
      "file": 16384,
      "file_dirty": 12288,
      "file_writeback": 0,
      "kernel": 14675968,
      "shmem": 0,
      "slab": 14019376
    },
    "spool_max_boundary_bytes": 45056,
    "staging_max_boundary_bytes": 71393280
  },
  "verification": {
    "host_lifetime_peak_rss_bytes": 70729728,
    "host_max_boundary_rss_bytes": 70713344,
    "host_cpu_ns": 7658918458,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 138547200,
      "memory_peak": 242606080,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 257,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 4867106,
      "throttled_usec": 0,
      "usage_usec": 9376215,
      "user_usec": 4509109
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 133943296,
      "file": 1990656,
      "file_dirty": 28672,
      "file_writeback": 0,
      "kernel": 1359872,
      "shmem": 0,
      "slab": 651600
    },
    "spool_max_boundary_bytes": 40960,
    "staging_max_boundary_bytes": 1966080
  }
}
```

## Physical reconciliation

| Category | Bytes |
| --- | --- |
| all_pack_bytes | 44,731,040 |
| file_content_pack_bytes | 38,156,349 |
| filesystem_allocation_difference_bytes | 892,928 |
| metadata_legacy_pack_bytes | 6,574,691 |
| sqlite_logical_bytes | 49,475,584 |
| sqlite_nonpack_bytes | 4,744,544 |
| store_allocated_bytes | 50,368,512 |

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
      "canonical_bytes": 1653832,
      "objects": 278,
      "raw_bytes": 1653832,
      "record_bytes": 1022450,
      "selected": 278
    },
    "metadata_legacy_FULL": {
      "canonical_bytes": 8540198,
      "objects": 46002,
      "raw_bytes": 8540198,
      "record_bytes": 8586200,
      "selected": 46002
    },
    "pack_v1": {
      "bytes": 6574691,
      "decoded_group_bytes": 9796902,
      "encoded_group_bytes": 6559779,
      "groups": 783,
      "header_directory_bytes": 14912,
      "packs": 149,
      "record_directory_bytes": 188252
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
      "bytes": 34896857,
      "decoded_group_bytes": 34357161,
      "encoded_group_bytes": 34357161,
      "groups": 33217,
      "header_directory_bytes": 539696,
      "packs": 514
    },
    "small_DELTA": {
      "canonical_bytes": 188676108,
      "frame_bytes": 12135191,
      "objects": 24855,
      "raw_bytes": 188104443,
      "record_bytes": 13154246,
      "record_header_bytes": 1019055,
      "selected": 24855
    },
    "small_FULL": {
      "canonical_bytes": 57895753,
      "frame_bytes": 21127657,
      "objects": 8362,
      "raw_bytes": 57703427,
      "record_bytes": 21202915,
      "record_header_bytes": 75258,
      "selected": 8362
    },
    "small_depth_counts": {
      "0": 8362,
      "1": 9466,
      "2": 6224,
      "3": 3972,
      "4": 2476,
      "5": 1508,
      "6": 762,
      "7": 326,
      "8": 121
    },
    "small_physical_DELTA_bases": {
      "count": 15303,
      "raw_bytes": 120546148,
      "record_bytes": 9132162
    },
    "small_physical_FULL_bases": {
      "count": 5836,
      "raw_bytes": 43915808,
      "record_bytes": 16140741
    },
    "small_physical_bases": {
      "count": 21139,
      "raw_bytes": 164461956,
      "record_bytes": 25272903
    },
    "small_record_kinds": {
      "0": 8362,
      "1": 3813,
      "2": 21042
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
      "unused": 289
    },
    {
      "bytes": 1990656,
      "name": "object_packs",
      "pages": 486,
      "pagetype": "leaf",
      "payload": 1484810,
      "unused": 494307
    },
    {
      "bytes": 43315200,
      "name": "object_packs",
      "pages": 10575,
      "pagetype": "overflow",
      "payload": 43249866,
      "unused": 23034
    },
    {
      "bytes": 57344,
      "name": "objects",
      "pages": 14,
      "pagetype": "internal",
      "payload": 42191,
      "unused": 8076
    },
    {
      "bytes": 4046848,
      "name": "objects",
      "pages": 988,
      "pagetype": "leaf",
      "payload": 3385021,
      "unused": 416188
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
    "page_count": 12079,
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
    "LAYERFS_PRODUCT_SEAL": "49106b92908eec9e7fcfc00bb08062d993b921a8d858fab66bce0989ac2facf5",
    "LAYERFS_SOURCE_COMMIT": "3fdd356c5ee06cbecd9e247d5891e7e7dcd6daa5",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "7c7cb6bf4cab65d4134ba4bb457586ce2067333b1509b19f942ca21a5271d82f",
    "LAYERFS_SOURCE_TREE": "d248bfc76898085eb3b2122d94e8e0ae0858866a",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127"
  },
  "host": {
    "LAYERFS_PRODUCT_SEAL": "49106b92908eec9e7fcfc00bb08062d993b921a8d858fab66bce0989ac2facf5",
    "LAYERFS_SOURCE_COMMIT": "3fdd356c5ee06cbecd9e247d5891e7e7dcd6daa5",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "7c7cb6bf4cab65d4134ba4bb457586ce2067333b1509b19f942ca21a5271d82f",
    "LAYERFS_SOURCE_TREE": "d248bfc76898085eb3b2122d94e8e0ae0858866a",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127",
    "binary_sha256": "16e81207361a5f992cf264058a8eebc4ab4cfbaa2d7604b64d022f00561dd563",
    "platform": "macOS-26.4.1-arm64-arm-64bit-Mach-O",
    "rust_toolchain": "1.85.1",
    "schema_sha256": "7ed3355be81cfdb82839d1651ee0919afc445ce6255c522f2b1bab54c9a92780"
  },
  "image": "sha256:cda9169f0b711b27231bb48d80dfc603802bb1686be086d5fa85ddc2295e4ac6",
  "fixture_digest": "eeb408e63b091aa7379aeefd1e3fbf819cdb4d882c67c03dd040c9fbe04feabb",
  "contract_digest": "9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50",
  "frozen_store_sha256": "c110adf15d93358971942258a48944e1cb671c65bab0d525e0945262ae46ceb0",
  "census_script_sha256": "dcd343150bb137ad30ef2dd87ede9bb86abac8a74065e7db2622efd95cd30440"
}
```

Original fixture manifest SHA256: `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Prepared inputs: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`.

- [compact-candidate-1-source.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-source.json); SHA256 `6542c39dbec7ac78365df824728bad39757a2114b1f2290553bc867deb56326a`
- [compact-candidate-1-dirty.patch](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-dirty.patch); SHA256 `3515545d955d5f30812c017469c5427fd10d8a5d46d29d61fc5b555868540d37`
- [compact-candidate-1-fs-benchmark-pro.identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-fs-benchmark-pro.identity.json); SHA256 `e4aa583c295c9a0ca3b50b8f7bbe1f55cf98a60b93b8ae0f23e3a9708d88fdcc`
- [compact-candidate-1-image-inspection.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-image-inspection.json); SHA256 `b8b8f12ae3db2cfa5fb0aa4f412c1d2876e9a1c4460fae7d19ddb7115a447b6c`

**build-host**: 58.138807417 s, exit 0. [compact-candidate-1-build-host-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-build-host-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-host"
]
```

**build-image**: 65.522227250 s, exit 0. [compact-candidate-1-build-image-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-build-image-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-storage-smoke-image"
]
```

**performance**: 62.322541833 s, exit 0. [compact-candidate-1-performance-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-performance-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:7c7cb6bf4cab65d4",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--output",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1"
]
```

**census**: 0.831711625 s, exit 0. [compact-candidate-1-census-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-census-command.json)

```json
[
  "python3",
  "docs/roadmap/0.1/0.1.5/issue100/census.py",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1"
]
```

**verification**: 30.145863458 s, exit 0. [compact-candidate-1-verification-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-verification-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:7c7cb6bf4cab65d4",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--storage-verify-run",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1"
]
```

- [identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1/identity.json)
- [performance-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1/performance-manifest.json)
- [verification-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1/verification-manifest.json)
- [census.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1/census.json)
- [compact-candidate-1-comparison.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/compact-candidate-1-comparison.json)

No broad Cargo/Clippy/doctest or unrelated qualification suite, release/tag, GC/repacking or unchanged-control rerun is claimed. The premature chain full157 execution remains incomplete diagnostic evidence in [the correction](followup-disposition.md), not a completed confirmation. Ten-state gains do not erase the original full157 regression. Final full157 remains deferred until a stable verified ten-state candidate is near target.

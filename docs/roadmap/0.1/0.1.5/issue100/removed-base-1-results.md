# removed-base-1: ten-snapshot measured result

Final allocated storage is **50,372,608 bytes**, growth **50,302,976 bytes** from **69,632 bytes**. The 45,000,000-byte objective is not met; the exact difference is **+5,372,608 bytes**. This is not near-target. No release-admission PASS or full157 confirmation is claimed.

Actual outcomes: **10 Created / 0 UpToDate**. Same-Store verification checked **10 states, 58,860 path states, 327,885,165 logical bytes**; cleanup was {'performance': 'PASS', 'verification': 'PASS'}. Frozen performance and post-verification manifests are separate lifecycle points. Verification checked the measured digest before reopening the SAME Store in a new coordinator; no replacement history was built.

The fixed full157 indices are **1, 18, 36, 53, 70, 88, 105, 122, 140, 157**. Skipped checkpoints are not replayed. Controls retain the [applicability audit](baseline-applicability-45mb.md). Git retains these ten trees with narrower metadata; its construction/packing timers do not match foreground save latency.

| Arm | Final allocated B | Candidate difference B |
| --- | --- | --- |
| Git | 38,223,872 | +12,148,736 |
| released v014 | 67,145,728 | -16,773,120 |
| original v015 | 66,105,344 | -15,732,736 |
| chain-1 | 56,668,160 | -6,295,552 |
| selected-full-1 | 54,562,816 | -4,190,208 |

Removed-name discovery reduced content packs by **4,998,158 B** versus selected-full-1, while final allocation fell **4,190,208 B**. The filesystem allocation difference grew from 12,288 to 806,912 B, so the serialized content saving cannot be reported as allocated saving. The post-verification framing comparison found 1,934 former FULLs changed to kind 2, saving 4,985,528 record bytes; 570 former kind-1 targets changed to kind 2, saving 55,325 B. Other redirected targets include regressions (93 kind-1 and five kind-2 targets became FULL); the aggregate physical census includes all of them.

Commit median/sum rose **19.45% / 23.44%** versus original v0.1.5, exceeding the prospective 10% criterion. Save median/sum rose 2.35% / 2.38%; paired median/sum rose 4.75% / 5.30%. Exact verification, cleanup and resource bounds passed, but this is not an unconditional speed PASS. Retain the implementation as an intermediate storage improvement while continuing toward the target and reporting the Commit tradeoff.

The read-only [framing comparison](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-quality.json) checks both post-verification Store digests and unchanged filesystem identity. Its first helper invocation failed before Store access because `__file__` was absent in the reused parser namespace; the failed source is retained as `removed-base-quality-failed-1.py`, and the corrected run/log use the same existing parser. No benchmark/control was rerun for this diagnostic failure.

## Public timings and resource tradeoffs

| Metric | Min s | Median s | Max s | Sum s | Median vs original v015 |
| --- | --- | --- | --- | --- | --- |
| exec_ns | 0.153887709 | 3.052408229 | 8.171625542 | 36.925741668 | +2.35% |
| commit_ns | 0.040438042 | 0.580516875 | 1.541591958 | 7.167381335 | +19.45% |
| paired_ns | 0.194325751 | 3.632925104 | 9.713217500 | 44.093123003 | +4.75% |
| verification_step_wall_ns | 0.187479667 | 2.240401230 | 4.169560500 | 21.787791667 | +0.99% |

These are ten dependent history steps. Min/max and per-step observations are descriptive; no tail confidence is inferred. The prospective <=10% speed/resource comparison and separate 8-MiB RSS allowance are engineering criteria, not owner-approved release gates.

| Scope | Seconds |
| --- | --- |
| performance_wall_ns | 58.934968459 |
| performance_work_wall_ns | 55.320258958 |
| verification_wall_ns | 26.622781917 |
| verification_work_wall_ns | 25.101836042 |
| setup_ns | 3.028198625 |
| verification_setup_ns | 1.142928708 |
| cleanup_ns | 0.536978209 |
| verification_cleanup_ns | 0.373269458 |
| preparation_ns | 8.856153375 |
| verification_preparation_ns | 3.216285125 |
| transfer_ns | 7.403669958 |

Preparation, setup, transfer, cleanup and case walls have their existing nested boundaries; do not add all rows as disjoint costs. Build and complete command receipts are below.

| Step | Save s | Commit s | Paired s | Verify step s | Allocated B |
| --- | --- | --- | --- | --- | --- |
| 1 | 0.153887709 | 0.040438042 | 0.194325751 | 0.187479667 | 696320 |
| 2 | 0.863367333 | 0.192534500 | 1.055901833 | 0.689006833 | 4235264 |
| 3 | 1.511650042 | 0.284673667 | 1.796323709 | 1.081875125 | 7380992 |
| 4 | 2.316469625 | 0.450906833 | 2.767376458 | 1.580925333 | 11575296 |
| 5 | 3.318137125 | 0.617993042 | 3.936130167 | 2.104692000 | 16818176 |
| 6 | 2.786679333 | 0.543040709 | 3.329720042 | 2.376110459 | 21012480 |
| 7 | 5.297707917 | 0.965814084 | 6.263522001 | 2.785624875 | 27303936 |
| 8 | 5.742012750 | 1.134316000 | 6.876328750 | 3.139933250 | 34643968 |
| 9 | 6.764204292 | 1.396072500 | 8.160276792 | 3.672583625 | 43032576 |
| 10 | 8.171625542 | 1.541591958 | 9.713217500 | 4.169560500 | 50372608 |

Full source-to-commit mappings, CPU/transfer values and original-oracle counts are in [removed-base-1-comparison.csv](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-comparison.csv).

Resource values retain their actual scopes: host lifetime peak RSS, cumulative process CPU, and boundary samples for cgroup categories/staging/spool. Boundary maxima are not simultaneous samples or continuous phase-local peaks.

```json
{
  "performance": {
    "host_lifetime_peak_rss_bytes": 115867648,
    "host_max_boundary_rss_bytes": 115867648,
    "host_cpu_ns": 15194981709,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 85766144,
      "memory_peak": 142319616,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 476,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 10776953,
      "throttled_usec": 0,
      "usage_usec": 17028696,
      "user_usec": 6251742
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 62668800,
      "file": 7110656,
      "file_dirty": 36864,
      "file_writeback": 0,
      "kernel": 14761984,
      "shmem": 0,
      "slab": 14123848
    },
    "spool_max_boundary_bytes": 45056,
    "staging_max_boundary_bytes": 71393280
  },
  "verification": {
    "host_lifetime_peak_rss_bytes": 70107136,
    "host_max_boundary_rss_bytes": 70090752,
    "host_cpu_ns": 7630944534,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 142434304,
      "memory_peak": 246292480,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 256,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 4932080,
      "throttled_usec": 0,
      "usage_usec": 9387400,
      "user_usec": 4455320
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 137842688,
      "file": 1966080,
      "file_dirty": 4096,
      "file_writeback": 0,
      "kernel": 1368064,
      "shmem": 0,
      "slab": 653360
    },
    "spool_max_boundary_bytes": 40960,
    "staging_max_boundary_bytes": 1966080
  }
}
```

## Physical reconciliation

| Category | Bytes |
| --- | --- |
| all_pack_bytes | 44,838,199 |
| file_content_pack_bytes | 38,279,771 |
| filesystem_allocation_difference_bytes | 806,912 |
| metadata_legacy_pack_bytes | 6,558,428 |
| sqlite_logical_bytes | 49,565,696 |
| sqlite_nonpack_bytes | 4,727,497 |
| store_allocated_bytes | 50,372,608 |

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
      "LFS4INT": 411,
      "LFS4LNK": 7,
      "LFS4MAP": 124,
      "LFS4MET": 4,
      "LFS4NSP": 4145
    },
    "metadata_legacy_DELTA": {
      "canonical_bytes": 1643184,
      "objects": 276,
      "raw_bytes": 1643184,
      "record_bytes": 992594,
      "selected": 276
    },
    "metadata_legacy_FULL": {
      "canonical_bytes": 8550846,
      "objects": 46004,
      "raw_bytes": 8550846,
      "record_bytes": 8596850,
      "selected": 46004
    },
    "pack_v1": {
      "bytes": 6558428,
      "decoded_group_bytes": 9777692,
      "encoded_group_bytes": 6543532,
      "groups": 782,
      "header_directory_bytes": 14896,
      "packs": 149,
      "record_directory_bytes": 188248
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
      "bytes": 35020279,
      "decoded_group_bytes": 34480583,
      "encoded_group_bytes": 34480583,
      "groups": 33217,
      "header_directory_bytes": 539696,
      "packs": 514
    },
    "small_DELTA": {
      "canonical_bytes": 187201025,
      "frame_bytes": 11752395,
      "objects": 24370,
      "raw_bytes": 186640515,
      "record_bytes": 12751565,
      "record_header_bytes": 999170,
      "selected": 24370
    },
    "small_FULL": {
      "canonical_bytes": 59370836,
      "frame_bytes": 21649395,
      "objects": 8847,
      "raw_bytes": 59167355,
      "record_bytes": 21729018,
      "record_header_bytes": 79623,
      "selected": 8847
    },
    "small_depth_counts": {
      "0": 8847,
      "1": 9315,
      "2": 6085,
      "3": 3894,
      "4": 2430,
      "5": 1468,
      "6": 740,
      "7": 319,
      "8": 119
    },
    "small_physical_DELTA_bases": {
      "count": 14971,
      "raw_bytes": 119556937,
      "record_bytes": 8867789
    },
    "small_physical_FULL_bases": {
      "count": 6167,
      "raw_bytes": 44831867,
      "record_bytes": 16514321
    },
    "small_physical_bases": {
      "count": 21138,
      "raw_bytes": 164388804,
      "record_bytes": 25382110
    },
    "small_record_kinds": {
      "0": 8847,
      "1": 3325,
      "2": 21045
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
      "unused": 307
    },
    {
      "bytes": 1982464,
      "name": "object_packs",
      "pages": 484,
      "pagetype": "leaf",
      "payload": 1488923,
      "unused": 482019
    },
    {
      "bytes": 43417600,
      "name": "object_packs",
      "pages": 10600,
      "pagetype": "overflow",
      "payload": 43352912,
      "unused": 22288
    },
    {
      "bytes": 61440,
      "name": "objects",
      "pages": 15,
      "pagetype": "internal",
      "payload": 42095,
      "unused": 12270
    },
    {
      "bytes": 4038656,
      "name": "objects",
      "pages": 986,
      "pagetype": "leaf",
      "payload": 3384799,
      "unused": 408228
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
    "page_count": 12101,
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
    "LAYERFS_PRODUCT_SEAL": "c7521be440f52f995fc3fa8ee2428ae97eff4080e3ade9c679ac2ebbfe7760d9",
    "LAYERFS_SOURCE_COMMIT": "6d0f6acbfb0406e0f38301d8122aa425aa985b81",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "72a0f268511506957ea4d65c3c87b6d4793b53d16681e79e2a0040b09b487edd",
    "LAYERFS_SOURCE_TREE": "a7a4fd24e932a69b489b581abfb96b65c1c11259",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127"
  },
  "host": {
    "LAYERFS_PRODUCT_SEAL": "c7521be440f52f995fc3fa8ee2428ae97eff4080e3ade9c679ac2ebbfe7760d9",
    "LAYERFS_SOURCE_COMMIT": "6d0f6acbfb0406e0f38301d8122aa425aa985b81",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "72a0f268511506957ea4d65c3c87b6d4793b53d16681e79e2a0040b09b487edd",
    "LAYERFS_SOURCE_TREE": "a7a4fd24e932a69b489b581abfb96b65c1c11259",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127",
    "binary_sha256": "02d82eaee64c6e26012535de913ff44de6fbebee896c53c27238526af5abd67e",
    "platform": "macOS-26.4.1-arm64-arm-64bit-Mach-O",
    "rust_toolchain": "1.85.1",
    "schema_sha256": "7ed3355be81cfdb82839d1651ee0919afc445ce6255c522f2b1bab54c9a92780"
  },
  "image": "sha256:514064d929ce4dac83d0d28f71c30e5980429812b88fbf125a8e89a4d5880b6f",
  "fixture_digest": "eeb408e63b091aa7379aeefd1e3fbf819cdb4d882c67c03dd040c9fbe04feabb",
  "contract_digest": "9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50",
  "frozen_store_sha256": "f9f6f58cd132698982a9e41e2724ec77644c7d1cb3bd6822f9b4004cb3f0fe04",
  "census_script_sha256": "dcd343150bb137ad30ef2dd87ede9bb86abac8a74065e7db2622efd95cd30440"
}
```

Original fixture manifest SHA256: `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Prepared inputs: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`.

- [removed-base-1-source.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-source.json); SHA256 `c184f51fe5e14bfb63707ac9d8005532c312578d838892359f61a0318136346c`
- [removed-base-1-dirty.patch](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-dirty.patch); SHA256 `3515545d955d5f30812c017469c5427fd10d8a5d46d29d61fc5b555868540d37`
- [removed-base-1-fs-benchmark-pro.identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-fs-benchmark-pro.identity.json); SHA256 `704d09de2850e5ed509b411d1093cf46f00ea294288164c279a0d4488d966d22`
- [removed-base-1-image-inspection.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-image-inspection.json); SHA256 `bfd37237a842fd90fcfb4b11854aa29bacfc52cfd6b68c8d7984e1c12d2fb5f9`

**build-host**: 57.462171250 s, exit 0. [removed-base-1-build-host-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-build-host-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-host"
]
```

**build-image**: 77.707234667 s, exit 0. [removed-base-1-build-image-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-build-image-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--build-storage-smoke-image"
]
```

**performance**: 68.025099917 s, exit 0. [removed-base-1-performance-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-performance-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:72a0f26851150695",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--output",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1"
]
```

**census**: 0.812525959 s, exit 0. [removed-base-1-census-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-census-command.json)

```json
[
  "python3",
  "docs/roadmap/0.1/0.1.5/issue100/census.py",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1"
]
```

**verification**: 30.067814667 s, exit 0. [removed-base-1-verification-command.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-verification-command.json)

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-ten",
  "--image",
  "layerfs-bench-infra:72a0f26851150695",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-fs-benchmark-pro",
  "--fixtures",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs",
  "--source-arm",
  "candidate",
  "--storage-verify-run",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1"
]
```

- [identity.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1/identity.json)
- [performance-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1/performance-manifest.json)
- [verification-manifest.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1/verification-manifest.json)
- [census.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1/census.json)
- [removed-base-1-comparison.json](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/removed-base-1-comparison.json)

No broad Cargo/Clippy/doctest or unrelated qualification suite, release/tag, GC/repacking or unchanged-control rerun is claimed. The premature chain full157 execution remains incomplete diagnostic evidence in [the correction](followup-disposition.md), not a completed confirmation. Ten-state gains do not erase the original full157 regression. Final full157 remains deferred until a stable verified ten-state candidate is near target.

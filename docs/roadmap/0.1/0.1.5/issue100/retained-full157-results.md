# Retained implementation: full157 confirmation

The owner explicitly requested this full157 run after the ten-state49,319,936-B result. That request supersedes the earlier near-target gate for this measurement. The45,000,000-B target remains specific to the ten-state smoke and remains unmet there; it does not apply to157 retained states. No release-admission PASS, tag or release is claimed.

Candidate final allocated storage: **134,246,400 B**, growth **134,176,768 B** from **69,632 B**. Actual outcomes: **157 Created / 0 UpToDate**. A new coordinator verified **157 states, 904,143 path states, 4,936,693,030 logical bytes** from the SAME measured Store after frozen digest checking. Performance and verification cleanup: **{'performance': 'PASS', 'verification': 'PASS'}**.

**Storage improves, foreground latency regresses.** Relative to released control, paired median is31.13% higher, Commit median18.30% higher and performance wall20.22% higher; historical verification wall is12.14% lower. Relative to initial v015, paired median is10.32% higher, Commit median19.16% higher and verification wall14.93% higher. Several prospective speed criteria are missed; this is not an unqualified speed improvement or release-admission PASS.

Candidate setup was15.630205542 seconds versus2.610920458 for released control. That contributes to case-wall difference, but does not explain the separate70.134479871-second increase in paired public-call sum. The cause of setup variation was not isolated; no timeout was changed or rerun collected to improve it.

## Matched comparison

Released control and initial v015 are the existing issue100 full157 arms, reused under [the applicability audit](baseline-applicability-45mb.md). The current harness differs only by the ten-state registration/routing addition; full157 operations, input ordering, timing boundaries and watchdogs are unchanged. The runner revalidated prepared fixtures and current binary/image/source seals. No unchanged control or build was repeated.

| Arm | Allocated B | Growth B | Save median s | Commit median s | Paired median s | Performance wall s | Verification wall s |
| --- | --- | --- | --- | --- | --- | --- | --- |
| control | 184,582,144 | 184,512,512 | 1.193853333 | 0.230891250 | 1.429884000 | 433.333684791 | 509.677223417 |
| initial | 201,371,648 | 201,302,016 | 1.482143250 | 0.229225625 | 1.699653250 | 485.452240250 | 389.626694458 |
| retained-full157-1 | 134,246,400 | 134,176,768 | 1.630228625 | 0.273142083 | 1.874999625 | 520.957421042 | 447.802943917 |

Against control: allocated difference **-50,335,744 B (-27.27%)**.

Against initial: allocated difference **-67,125,248 B (-33.33%)**.

Prospective <=10% speed/RSS comparisons and the separate8-MiB absolute RSS allowance remain working criteria, not owner-approved release gates. All157 observations are dependent history steps. Percentiles/min/max are descriptive; no tail confidence is inferred.

| Metric | Min s | Median s | p95 s | Max s | Sum s |
| --- | --- | --- | --- | --- | --- |
| exec_ns | 0.118120041 | 1.630228625 | 5.127202750 | 8.742677625 | 305.505782382 |
| commit_ns | 0.021044250 | 0.273142083 | 0.936116917 | 1.531790542 | 53.347171292 |
| paired_ns | 0.139164291 | 1.874999625 | 6.271788334 | 10.274468167 | 358.852953674 |
| verification_step_wall_ns | 0.205599041 | 2.541209750 | 4.877306959 | 5.071685167 | 394.210761289 |

| Comparison metric | vs released control | vs initial v015 |
| --- | --- | --- |
| exec_ns median | +36.55% | +9.99% |
| exec_ns p95 | +20.85% | +8.85% |
| exec_ns sum | +24.80% | +6.18% |
| commit_ns median | +18.30% | +19.16% |
| commit_ns p95 | +34.31% | +28.59% |
| commit_ns sum | +21.44% | +23.55% |
| paired_ns median | +31.13% | +10.32% |
| paired_ns p95 | +24.15% | +14.38% |
| paired_ns sum | +24.29% | +8.45% |
| performance_wall_ns | +20.22% | +7.31% |
| verification_wall_ns | -12.14% | +14.93% |

All157 source-to-commit mappings, public per-state save/Commit/paired times, historical verification times and counts, CPU/transfer/allocation values: [comparison CSV](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1-comparison.csv). Full derived data: [comparison JSON](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1-comparison.json).

## Physical attribution

| Category | Bytes |
| --- | --- |
| all_pack_bytes | 118,095,748 |
| file_content_pack_bytes | 68,607,962 |
| filesystem_allocation_difference_bytes | 3,256,320 |
| metadata_legacy_pack_bytes | 49,487,786 |
| sqlite_logical_bytes | 130,990,080 |
| sqlite_nonpack_bytes | 12,894,332 |
| store_allocated_bytes | 134,246,400 |

Content + metadata + SQLite nonpack + filesystem allocation difference equals final allocation. Logical bytes differ from allocated bytes. Physical FULL/DELTA bases below are subsets of existing retained records, counted once; their bytes must not be added to totals a second time.

```json
{
  "counts": {
    "large_CDC_FULL": {
      "canonical_bytes": 18524913,
      "frame_bytes": 4553023,
      "objects": 910,
      "raw_bytes": 18505803,
      "record_bytes": 4557573,
      "record_header_bytes": 4550,
      "selected": 910
    },
    "large_CDC_PREFIX": {
      "canonical_bytes": 48444632,
      "frame_bytes": 3444826,
      "objects": 2311,
      "raw_bytes": 48396101,
      "record_bytes": 3530333,
      "record_header_bytes": 85507,
      "selected": 2311
    },
    "legacy_full_roles": {
      "LFS4CHK": 5,
      "LFS4DIR": 13647,
      "LFS4FSR": 158,
      "LFS4INO": 89576,
      "LFS4INT": 5549,
      "LFS4LNK": 7,
      "LFS4MAP": 1058,
      "LFS4MET": 4,
      "LFS4NSP": 14429
    },
    "metadata_legacy_DELTA": {
      "canonical_bytes": 26757164,
      "objects": 4673,
      "raw_bytes": 26757164,
      "record_bytes": 5571112,
      "selected": 4673
    },
    "metadata_legacy_FULL": {
      "canonical_bytes": 52614944,
      "objects": 124433,
      "raw_bytes": 52614944,
      "record_bytes": 52739377,
      "selected": 124433
    },
    "pack_v1": {
      "bytes": 49487786,
      "decoded_group_bytes": 58852097,
      "encoded_group_bytes": 49371690,
      "groups": 6296,
      "header_directory_bytes": 116096,
      "packs": 960,
      "record_directory_bytes": 541608
    },
    "pack_v2": {
      "bytes": 8120302,
      "decoded_group_bytes": 8103038,
      "encoded_group_bytes": 8103038,
      "groups": 562,
      "header_directory_bytes": 17264,
      "packs": 517,
      "record_directory_bytes": 15132
    },
    "pack_v3": {
      "bytes": 60487660,
      "decoded_group_bytes": 59254684,
      "encoded_group_bytes": 59254684,
      "groups": 75398,
      "header_directory_bytes": 1232976,
      "packs": 1663
    },
    "small_DELTA": {
      "canonical_bytes": 659087564,
      "frame_bytes": 27686507,
      "objects": 66034,
      "raw_bytes": 657568782,
      "record_bytes": 30393901,
      "record_header_bytes": 2707394,
      "selected": 66034
    },
    "small_FULL": {
      "canonical_bytes": 81917405,
      "frame_bytes": 28776507,
      "objects": 9364,
      "raw_bytes": 81702033,
      "record_bytes": 28860783,
      "record_header_bytes": 84276,
      "selected": 9364
    },
    "small_depth_counts": {
      "0": 9364,
      "1": 15101,
      "2": 11896,
      "3": 9748,
      "4": 8161,
      "5": 6709,
      "6": 5578,
      "7": 4802,
      "8": 4039
    },
    "small_physical_DELTA_bases": {
      "count": 44010,
      "raw_bytes": 434248597,
      "record_bytes": 21027713
    },
    "small_physical_FULL_bases": {
      "count": 7429,
      "raw_bytes": 69305835,
      "record_bytes": 24312867
    },
    "small_physical_bases": {
      "count": 51439,
      "raw_bytes": 503554432,
      "record_bytes": 45340580
    },
    "small_record_kinds": {
      "0": 9364,
      "1": 8197,
      "2": 57837
    }
  },
  "decoder_library": "/opt/homebrew/lib/libzstd.dylib",
  "decoder_library_sha256": "69d8dee81bccdee425d167300d11431f3c5ecc34a3011400e536aa89f7756189",
  "elapsed_ns": 2264932792,
  "maximum_small_decoded_canonical_closure_bytes": 523366,
  "maximum_small_delta_depth": 8,
  "maximum_small_retained_encoded_record_bytes": 71494,
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
      "pagetype": "internal",
      "payload": 680,
      "unused": 3364
    },
    {
      "bytes": 24576,
      "name": "commits",
      "pages": 6,
      "pagetype": "leaf",
      "payload": 20639,
      "unused": 3282
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
      "bytes": 20480,
      "name": "object_packs",
      "pages": 5,
      "pagetype": "internal",
      "payload": 0,
      "unused": 5449
    },
    {
      "bytes": 7708672,
      "name": "object_packs",
      "pages": 1882,
      "pagetype": "leaf",
      "payload": 5816371,
      "unused": 1846591
    },
    {
      "bytes": 112488448,
      "name": "object_packs",
      "pages": 27463,
      "pagetype": "overflow",
      "payload": 112294254,
      "unused": 84342
    },
    {
      "bytes": 143360,
      "name": "objects",
      "pages": 35,
      "pagetype": "internal",
      "payload": 110373,
      "unused": 14556
    },
    {
      "bytes": 10543104,
      "name": "objects",
      "pages": 2574,
      "pagetype": "leaf",
      "payload": 8797002,
      "unused": 1110054
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
    "page_count": 31980,
    "page_size": 4096,
    "user_version": 9
  },
  "reconciliation": {
    "all_pack_bytes": 118095748,
    "file_content_pack_bytes": 68607962,
    "filesystem_allocation_difference_bytes": 3256320,
    "metadata_legacy_pack_bytes": 49487786,
    "sqlite_logical_bytes": 130990080,
    "sqlite_nonpack_bytes": 12894332,
    "store_allocated_bytes": 134246400
  },
  "schema": "issue100-census-v2",
  "scope": "Physical framing/locator/base accounting. Canonical authentication and exact visible bytes are qualified separately by same-Store historical verification.",
  "script_sha256": "dcd343150bb137ad30ef2dd87ede9bb86abac8a74065e7db2622efd95cd30440",
  "sqlite_residual_bytes": 0,
  "store": "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/host-runtime/store.sqlite",
  "store_inode": 780485869,
  "store_sha256": "04cd697f2f917fd10a36efb45afcee9bf38c58ffc9743af9acb4cab00b346246"
}
```

## Resource and orchestration scopes

Host lifetime RSS, recorded process CPU and boundary samples retain their original scopes. Cgroup category maxima are not simultaneous samples; boundary disk/memory observations are not continuous phase-local peaks. Topology remains macOS Store/SQLite/coordinator/publication/spool and Linux live core/FUSE/workload,2CPUs/2GiB/no swap/256PIDs.

```json
{
  "performance": {
    "host_lifetime_peak_rss_bytes": 127303680,
    "host_max_boundary_rss_bytes": 127270912,
    "host_cpu_ns": 156570685107,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 101097472,
      "memory_peak": 159420416,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 4478,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 71043202,
      "throttled_usec": 0,
      "usage_usec": 124315717,
      "user_usec": 53272515
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 72491008,
      "file": 12374016,
      "file_dirty": 57344,
      "file_writeback": 0,
      "kernel": 15200256,
      "shmem": 0,
      "slab": 14528920
    },
    "spool_max_boundary_bytes": 647168,
    "staging_max_boundary_bytes": 70705152
  },
  "verification": {
    "host_lifetime_peak_rss_bytes": 77266944,
    "host_max_boundary_rss_bytes": 77234176,
    "host_cpu_ns": 160470947662,
    "cgroup_max_boundary": {
      "burst_usec": 0,
      "high": 0,
      "low": 0,
      "max": 0,
      "memory_current": 148238336,
      "memory_peak": 252596224,
      "nice_usec": 0,
      "nr_bursts": 0,
      "nr_periods": 4461,
      "nr_throttled": 0,
      "oom": 0,
      "oom_group_kill": 0,
      "oom_kill": 0,
      "swap_current": 0,
      "system_usec": 77574319,
      "throttled_usec": 0,
      "usage_usec": 147341374,
      "user_usec": 69767054
    },
    "cgroup_memory_stat_max_boundary": {
      "anon": 143708160,
      "file": 1974272,
      "file_dirty": 12288,
      "file_writeback": 0,
      "kernel": 1388544,
      "shmem": 0,
      "slab": 653080
    },
    "spool_max_boundary_bytes": 643072,
    "staging_max_boundary_bytes": 1966080
  }
}
```

| Timer | Seconds |
| --- | --- |
| performance_wall_ns | 520.957421042 |
| performance_work_wall_ns | 504.605326917 |
| verification_wall_ns | 447.802943917 |
| verification_work_wall_ns | 445.951385000 |
| setup_ns | 15.630205542 |
| verification_setup_ns | 1.383971667 |
| cleanup_ns | 0.658257167 |
| verification_cleanup_ns | 0.447151208 |
| preparation_ns | 74.302836333 |
| verification_preparation_ns | 73.050181875 |
| transfer_ns | 92.883510203 |

Preparation, setup, transfer, work and cleanup scopes can overlap/nest; do not add them as disjoint costs. No new builds were required: the existing saved retained-candidate host/image pair was reused after strict seal checks. Original build costs remain in the ten-state report and are not charged as new builds here.

## Custody, commands and limitations

Current product/harness are byte-identical to measured product source ee78028ba. Current reporting head at execution is1955205db; raw identity records preserve that distinction rather than relabelling the saved binary. Source fixture manifest03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271, tip b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed, all157 checkpoints in original order.

```json
{
  "source": {
    "LAYERFS_PRODUCT_SEAL": "24cde1dce88104daebf2d01e6b711665cfa52c52880b31157fbcfa5c27214673",
    "LAYERFS_SOURCE_COMMIT": "1955205db82f4bf84b73efaf60b97c1d508320cc",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "2941cd53eab45065b5b2461479805852d5779a4d0c3515a2f2c04baa64097196",
    "LAYERFS_SOURCE_TREE": "e8e8e1ea0092c55162661a257ab7abee2c9335c1",
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
  "fixture_digest": "206b07d8100822f85578d743003cbf9f868405ca2e01c0acf0b20866bb0e1bed",
  "contract_digest": "e33e6ac09ed44cc5b9405800b55d5c16d09dd1ca3446a33298bb75124105200d"
}
```

**performance:** complete command wall 596.187291917 s, exit 0.

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-full",
  "--source-arm",
  "candidate",
  "--image",
  "sha256:158f34d7af6e21123e713a661a6e7644bdfcae15a00a76c663b7af04592c94bf",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-fs-benchmark-pro",
  "--data",
  "/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data",
  "--output",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1"
]
```

**census:** complete command wall 2.403865292 s, exit 0.

```json
[
  "python3",
  "docs/roadmap/0.1/0.1.5/issue100/census.py",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1"
]
```

**verification:** complete command wall 521.946669791 s, exit 0.

```json
[
  "python3",
  "benchmark/fs-bench-pro/shared/runner.py",
  "--storage-smoke",
  "deepseek-full",
  "--source-arm",
  "candidate",
  "--image",
  "sha256:158f34d7af6e21123e713a661a6e7644bdfcae15a00a76c663b7af04592c94bf",
  "--host-binary",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1-fs-benchmark-pro",
  "--data",
  "/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data",
  "--storage-verify-run",
  "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1"
]
```

Performance allocation/Store digest and census were frozen before verification. Verification lifecycle mutations remain outside those observations; performance and post-verification manifests therefore describe different lifecycle points. Saved raw receipts, identity, manifests, Store and census are under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1`.

The earlier premature full157 run remains incomplete historical evidence and was not resumed or reused as this proof. This run uses a fresh Store and the retained optimized implementation. No product edits, threshold/codec sweeps, broad tests, release qualification or release publication were part of this confirmation.

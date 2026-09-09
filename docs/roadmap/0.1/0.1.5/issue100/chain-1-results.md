# Intermediate chain-1 ten-snapshot result

**Chain-1 uses 56,668,160 final allocated bytes, with 56,598,528 bytes of growth. It misses the 45,000,000-byte objective by 11,668,160 bytes and is not near-target.** This is an intermediate bounded-predecessor candidate, not completion of issue #100 or release admission. Final full157 confirmation has not run for this candidate.

The measured product adds immediate SmallContent predecessor chains under the [prospective amendment](bounded-predecessor-amendment.md): at most eight edges, 512 KiB decoded canonical closure including the target, 256 KiB retained encoded records, and schema-9 kind-2 fencing. The cutoff, CDC and pinned codec settings stay fixed. This candidate does not admit CDC-backed predecessors across the SmallContent boundary.

## Storage and target

| Arm | Initial allocated B | Final allocated B | Growth B | Chain-1 difference B | Chain-1 change |
| --- | --- | --- | --- | --- | --- |
| Git | 8,192 | 38,223,872 | 38,215,680 | 18,444,288 | +48.253% |
| Released v0.1.4 | 69,632 | 67,145,728 | 67,076,096 | -10,477,568 | -15.604% |
| Existing v0.1.5 | 69,632 | 66,105,344 | 66,035,712 | -9,437,184 | -14.276% |
| Chain-1 | 69,632 | 56,668,160 | 56,598,528 | 0 | +0.000% |

Chain-1 removes 9,437,184 allocated bytes (14.276%) versus existing v0.1.5 and 10,477,568 bytes versus released v0.1.4. It remains 18,444,288 bytes above Git. Values are exact bytes; 56.668160 decimal MB is not 45 MB. Git retains these ten trees with a narrower filesystem metadata scope; construction and later packing costs are separate from LayerFS foreground save timers. [Baseline applicability](baseline-applicability-45mb.md) documents reuse of the unchanged controls.

## Public latency and prospective comparison

Ten dependent history steps, one fresh Store. Seconds below derive from retained integer nanoseconds. No independent-repetition or tail-confidence claim. The prospective working comparison is <=10% increase in medians, public-call sums, performance/verification walls and recorded host lifetime RSS, with a separately reported 8-MiB absolute RSS allowance. It is an engineering criterion, not an owner-approved numerical release gate.

| Public metric | Arm | Min s | Median s | Max s | Sum s |
| --- | --- | --- | --- | --- | --- |
| Save/Exec | v014 | 0.151556125 | 3.107363250 | 7.579190042 | 35.629309334 |
| Save/Exec | v015 | 0.152755042 | 2.982280354 | 7.510265708 | 36.068060415 |
| Save/Exec | chain-1 | 0.181988000 | 2.813470542 | 7.241390791 | 34.047000873 |
| Commit | v014 | 0.030421000 | 0.528708792 | 1.283256875 | 6.286720626 |
| Commit | v015 | 0.033685917 | 0.485983625 | 1.275150291 | 5.806243418 |
| Commit | chain-1 | 0.025653625 | 0.501543562 | 1.274562166 | 5.954278749 |
| Paired save+Commit | v014 | 0.181977125 | 3.636072042 | 8.832459834 | 41.916029960 |
| Paired save+Commit | v015 | 0.186440959 | 3.468263979 | 8.785415999 | 41.874303833 |
| Paired save+Commit | chain-1 | 0.207641625 | 3.315014103 | 8.515952957 | 40.001279622 |

| Compared with existing v0.1.5 | Chain-1/baseline ratio | Change |
| --- | --- | --- |
| exec_ns median | 0.943396 | -5.660% |
| exec_ns sum | 0.943965 | -5.603% |
| commit_ns median | 1.032017 | +3.202% |
| commit_ns sum | 1.025496 | +2.550% |
| paired_ns median | 0.955814 | -4.419% |
| paired_ns sum | 0.955270 | -4.473% |
| performance_wall_ns  | 0.969133 | -3.087% |
| verification_wall_ns  | 0.945914 | -5.409% |
| performance_host_lifetime_peak_rss_bytes  | 0.966390 | -3.361% |
| verification_host_lifetime_peak_rss_bytes  | 0.976883 | -2.312% |

Commit median rises by 3.202% and its public-call sum by 2.550%; save, paired and both case walls decrease in this observation. Performance RSS changes by -4,046,848 B and verification RSS by -1,654,784 B, so neither consumes the separate 8,388,608-B absolute allowance. These observations justify retaining this improvement for further investigation, but do not establish the storage objective.

## All ten retained snapshots

| Step | Full157 index | Save s | Commit s | Paired s | Historical verifier Exec s | Verification step wall s | Allocated B |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1 | 0.181988000 | 0.025653625 | 0.207641625 | 0.114714042 | 0.173742459 | 700,416 |
| 2 | 18 | 0.807279209 | 0.131803667 | 0.939082876 | 0.519166250 | 0.577619834 | 4,239,360 |
| 3 | 36 | 1.429347750 | 0.231856917 | 1.661204667 | 0.901421625 | 0.967807000 | 7,385,088 |
| 4 | 53 | 2.245475291 | 0.390759833 | 2.636235124 | 1.404284959 | 1.478791167 | 12,627,968 |
| 5 | 70 | 3.011629500 | 0.510245416 | 3.521874916 | 1.818653875 | 1.890056292 | 17,870,848 |
| 6 | 88 | 2.615311583 | 0.492841708 | 3.108153291 | 2.181541042 | 2.261581500 | 22,065,152 |
| 7 | 105 | 4.721107083 | 0.818866459 | 5.539973542 | 2.507632916 | 2.582095916 | 29,405,184 |
| 8 | 122 | 5.642521625 | 0.952917250 | 6.595438875 | 2.889833167 | 2.970966000 | 36,745,216 |
| 9 | 140 | 6.150950041 | 1.124771708 | 7.275721749 | 3.484734375 | 3.615455416 | 46,182,400 |
| 10 | 157 | 7.241390791 | 1.274562166 | 8.515952957 | 3.798716042 | 3.924404708 | 56,668,160 |

Every step returned Created; all ten exact commit IDs and full source SHAs are in [CSV](chain-1-results.csv) and [JSON](chain-1-results.json), together with baseline step rows, allocation, transfers, CPU and complete coverage counts. The paired median is the median of each actual step’s sum. Historical verifier Exec performs exhaustive reading and hashing through public FUSE, so it is not a pure-read latency benchmark.

## Lifecycle costs

| Scope | v0.1.4 s | Existing v0.1.5 s | Chain-1 s |
| --- | --- | --- | --- |
| Performance case wall | 56.191617292 | 56.145798708 | 54.412738000 |
| Performance work wall | 53.242828458 | 53.177959875 | 50.846944125 |
| Verification case wall | 31.014695667 | 26.492623625 | 25.059731500 |
| Verification work wall | 29.533861667 | 25.033151167 | 23.557253584 |
| Performance setup | 2.344705125 | 2.314562791 | 2.966688792 |
| Verification setup | 1.090992125 | 1.062320583 | 1.118069250 |
| Performance input acquisition/validation | 2.945910958 | 2.970530917 | 3.591694125 |
| Verification input acquisition/validation | 3.001159709 | 3.308235042 | 3.152552625 |
| Performance input transfer sum | 7.374326124 | 7.349587169 | 7.221912750 |
| Performance wrapper cleanup | 0.545260333 | 0.596251708 | 0.551338458 |
| Verification wrapper cleanup | 0.381377292 | 0.388955792 | 0.376948084 |

| Chain-1 external command | Wall s | Exit |
| --- | --- | --- |
| chain-1-build-host-command.json | 55.407264834 | 0 |
| chain-1-build-image-command.json | 73.480895500 | 0 |
| chain-1-census-command.json | 0.797766916 | 0 |
| chain-1-performance-command.json | 58.256949458 | 0 |
| chain-1-verification-command.json | 28.468043500 | 0 |

Native public empty Init took 0.001262875 s, separate from save timers. Warm performance+verification command wall totals 86.724992958 s, excluding builds and census. Census internal wall is 0.706387167 s; its wrapper is separately recorded above. Shared Cargo/BuildKit caches and prepared inputs were reused. These scopes overlap: do not add case/work wall and their setup/transfer/cleanup subdivisions twice.

## Host and container resources

| Host/boundary metric | Performance | Verification |
| --- | --- | --- |
| Recorded host phase CPU s | 14.345020999 | 7.058162832 |
| host_lifetime_peak_rss_bytes | 116,359,168 | 69,926,912 |
| host_max_boundary_rss_bytes | 116,359,168 | 69,894,144 |
| spool_max_boundary_bytes | 45,056 | 40,960 |
| staging_max_boundary_bytes | 71,393,280 | 1,966,080 |

Host CPU above is seconds summed from recorded phase receipts. RSS/spool/staging values are bytes. Performance host CPU is +3.878% versus existing v0.1.5; verification host CPU is +2.733%. Lifetime RSS is a process high-water observation; disk values are boundary samples, not continuous spool peaks.

| Container counter/category | Performance boundary maximum | Verification boundary maximum |
| --- | --- | --- |
| memory_current | 84,525,056 | 144,785,408 |
| memory_peak | 141,131,776 | 248,750,080 |
| usage_usec | 15,174,207 | 8,987,576 |
| user_usec | 5,334,827 | 4,343,244 |
| system_usec | 9,839,380 | 4,644,331 |
| oom | 0 | 0 |
| oom_kill | 0 | 0 |
| oom_group_kill | 0 | 0 |
| swap_current | 0 | 0 |
| nr_throttled | 0 | 0 |
| throttled_usec | 0 | 0 |
| memory_stat.anon B | 62,091,264 | 140,165,120 |
| memory_stat.file B | 7,114,752 | 1,990,656 |
| memory_stat.file_dirty B | 32,768 | 28,672 |
| memory_stat.file_writeback B | 0 | 0 |
| memory_stat.kernel B | 14,757,888 | 1,376,256 |
| memory_stat.shmem B | 0 | 0 |
| memory_stat.slab B | 14,121,232 | 656,144 |

Cgroup CPU counters are cumulative microseconds sampled at boundaries; memory values are bytes. Category maxima may occur at different times and must not be summed as one simultaneous peak. Host and container scopes remain separate. Both runs enforce 2 CPUs/2 GiB/no swap/256 PIDs with host Store/SQLite/coordinator/publication/spool and container-only daemon/live core/FUSE/workload. Recorded OOM, swap and throttling counters remain zero. Missing phase-local continuous peaks are unavailable, not fabricated zeros.

## Byte reconciliation and dependencies

| Additive component | Bytes |
| --- | --- |
| file_content_pack_bytes | 44,602,316 |
| metadata_legacy_pack_bytes | 6,577,457 |
| sqlite_nonpack_bytes | 4,751,107 |
| filesystem_allocation_difference_bytes | 737,280 |
| store_allocated_bytes | 56,668,160 |

44,602,316 + 6,577,457 + 4,751,107 + 737,280 = **56,668,160 allocated bytes**. All pack BLOBs total 51,179,773 B; SQLite logical length is 55,930,880 B (13,655 pages × 4,096 B), freelist zero, dbstat residual zero. The allocation difference is filesystem allocation versus SQLite length; it is not additional logical content.

| Pack/record population | Count | Frame B | Header/other B | Complete record/pack B |
| --- | --- | --- | --- | --- |
| small_FULL | 14,664 | 30,497,635 | 131,976 | 30,629,611 |
| small_DELTA | 18,553 | 9,412,620 | 760,673 | 10,173,293 |
| large_CDC_FULL | 381 | 2,097,249 | 1,905 | 2,099,154 |
| large_CDC_PREFIX | 354 | 1,140,960 | 13,098 | 1,154,058 |
| pack_v1 | 149 | 6,562,353 | 15,104 | 6,577,457 |
| pack_v2 | 80 | 3,256,564 | 2,928 | 3,259,492 |
| pack_v3 | 528 | 40,802,904 | 539,920 | 41,342,824 |

The pack rows use encoded group bytes and pack header/directory bytes; they are aggregates of the record rows, not additional storage. Pack-v2 record directories add 3,352 B. Pack-v1 metadata/legacy records total 9,601,650 decoded B plus 188,296 B record directories, compressed to 6,562,353 B encoded groups; its 15,104 B pack headers/directories produce 6,577,457 B. Five legacy chunk objects remain within this category.

| Unique physical SmallContent base subset | Objects | Raw B | Complete encoded record B |
| --- | --- | --- | --- |
| small_physical_FULL_bases | 7,551 | 49,113,185 | 17,415,165 |
| small_physical_DELTA_bases | 10,983 | 100,515,264 | 6,969,877 |
| small_physical_bases | 18,534 | 149,628,449 | 24,385,042 |

Physical bases are counted once per distinct selected dependency and are already included in the FULL/DELTA record totals. They must not be added again. All 18,553 small DELTAs use kind 2; 14,664 use FULL kind 0. Maximum observed chain depth is 8; maximum decoded canonical closure including the target is 521,863 B; maximum retained encoded record sum is 71,494 B.

| Depth | Objects |
| --- | --- |
| 0 | 14,664 |
| 1 | 7,554 |
| 2 | 4,858 |
| 3 | 2,903 |
| 4 | 1,676 |
| 5 | 901 |
| 6 | 397 |
| 7 | 188 |
| 8 | 76 |

SQLite object locator pages occupy 4,087,808 B; object-pack B-tree/overflow pages 51,781,632 B; other tables/schema/index pages 61,440 B. Full leaf/internal/overflow payload and unused-byte rows are retained in JSON. Pack-page overhead versus pack BLOBs is 601,859 B; together with locator/other pages this equals the 4,751,107-B SQLite non-pack budget.

Against existing v0.1.5, content packs change by -9,643,300 B, metadata/legacy packs by 18,343 B, and all remaining allocation by 187,773 B, reconciling to -9,437,184 B. Small DELTA frames fall from 19,361,953 to 9,412,620 B, while FULL frames rise from 30,191,026 to 30,497,635 B. Therefore more depth materially improves accumulated predecessor differences, but does not solve the remaining FULL population or establish the 45-MB objective.

## Same-Store proof and exact custody

The frozen Store was `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/chain-1/deepseek-ten/host-runtime/store.sqlite`, inode 780433098, SHA256 `f6644302aacbc03bed17ff5b2094fc3aab171e9aaec4486250eed75921b3d035`. Census matched the performance-manifest digest before and after read-only inspection. The runner checked the same digest before a new coordinator reopened that same path; no second history was constructed. All ten mappings verified 58,860 path states and 327,885,165 logical bytes, including complete content/path/type/mode/symlink oracles. Both performance and verification report successful End/coordinator shutdown and container removal. Verification lifecycle writes occur after frozen allocation. The final verification manifest was independently hash-checked: 68/68 files matched.

| Candidate identity | Value |
| --- | --- |
| LAYERFS_PRODUCT_SEAL | `1515c4fdc1b0e8951017d7f0f1843f639401a02803189b057915997dbff1102b` |
| LAYERFS_SOURCE_COMMIT | `f631667665167eeb92dc391850ab7969d5e952d6` |
| LAYERFS_SOURCE_DIRTY | `true` |
| LAYERFS_SOURCE_SEAL | `a9d2de398413a7a0bf6c19b034f9e77a38a841c585514927b11b053d19a1e47d` |
| LAYERFS_SOURCE_TREE | `0d49bd00b8a57404e36920562287a9d6475f8271` |
| WORKLOAD_SOURCE_SHA256 | `86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127` |
| Host binary SHA256 | `eb8db670898f6db5f63a3a2cba220314845fba0520065793bfb88913e7b14413` |
| Image ID | `sha256:9d9684cce970d9c29e3b4118570cfb6e94507139ee92b56a8dcd2d2fd827d088` |
| Census script SHA256 | `dcd343150bb137ad30ef2dd87ede9bb86abac8a74065e7db2622efd95cd30440` |
| Manifest SHA256 | `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271` |
| Original fixture.json SHA256 | `6a28059656b671edc6f068d578a0f4670bfc7519aa0f54ee9dda10941cc7f64a` |
| Ten contract SHA256 | `9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50` |

The saved dirty patch is empty; the intentional untracked target symlink accounts for dirty status. The image inspection, source record, binary identity, dirty patch, comparison inputs and every command receipt have SHA256 custody references in the JSON. Fixture indices/source SHAs are recorded per step; source tip is `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`.

Exact commands below ran from `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100`; complete argv/cwd/elapsed/exit receipts remain in JSON and the evidence directory.

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
```

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-storage-smoke-image
```

```sh
python3 docs/roadmap/0.1/0.1.5/issue100/census.py /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/chain-1
```

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-ten --image layerfs-bench-infra:a9d2de398413a7a0 --host-binary /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/chain-1-fs-benchmark-pro --fixtures /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs --source-arm candidate --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/chain-1
```

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-ten --image layerfs-bench-infra:a9d2de398413a7a0 --host-binary /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/chain-1-fs-benchmark-pro --fixtures /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs --source-arm candidate --storage-verify-run /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/chain-1
```

## Limitations and remaining work

This is one intermediate candidate with a complete successful ten-state measurement and same-Store proof. No unchanged arm was rerun for a nicer number. Focused chain and upper-range exact-CAS checks have successful command receipts; the original fixture-family diagnostic remains separate from public smoke evidence. This report does not erase earlier retained failures/rejected experiments or the initial full157 regression. No broad Cargo, Clippy, doctest or unrelated qualification ran for this report.

The 45-MB target remains unmet by 11,668,160 B. Further authorized storage diagnosis/implementation is needed; final full157 confirmation is still unrun and belongs after the short-loop design stabilizes. Ten-state improvement does not establish long-history storage or performance. No release/tag or issue-completion claim follows from this result.

Raw evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence`. Machine-readable [report JSON](chain-1-results.json) and [per-snapshot CSV](chain-1-results.csv) preserve the exact measurements and source references; producing this report performed no new product measurement.

# Ten-snapshot baselines: Git, v0.1.4 and existing v0.1.5

**All three histories completed and verified. These are baseline observations, not a new storage optimization or release admission.**

Exactly ten selected snapshots are retained: full157 indices **1, 18, 36, 53, 70, 88, 105, 122, 140, 157**, in that order. The skipped checkpoints are not replayed. The first contains 276 files / 1,499,109 logical bytes; the last contains 9,415 files / 65,020,822 bytes. Each arm verifies 58,860 path states and 327,885,165 logical bytes across the ten states.

[Frozen contract](ten-snapshot-contract.md). Existing v0.1.5 means the current SmallContent implementation plus the verified upper-range exact-CAS dispatch fix. No FULL-anchor policy or codec change was made. The compiled host session, importer and Linux workload are unchanged; the Python runner schedules the selected transitions through the existing DeepSeek session.

## Storage

| Metric | Git | v0.1.4 | Existing v0.1.5 |
|---|---:|---:|---:|
| Initial allocated bytes | 8,192 | 69,632 | 69,632 |
| Final allocated bytes | **38,223,872** | **67,145,728** | **66,105,344** |
| Final allocated MB | 38.224 | 67.146 | 66.105 |
| Allocated growth | 38,215,680 | 67,076,096 | 66,035,712 |
| Final apparent bytes | 37,372,675 | 67,104,768 | 65,572,864 |

v0.1.5 uses 1,040,384 fewer allocated bytes (**1.55%**) than v0.1.4 and **72.94% more** than Git. LayerFS allocation includes relevant sidecars (zero in both final observations); Git includes repository files, pack and indexes. Logical database length and filesystem allocation remain separate quantities.

## LayerFS timings

| Metric, seconds | v0.1.4 | Existing v0.1.5 |
|---|---:|---:|
| Save/Exec median (min–max) | 3.107 (0.152–7.579) | 2.982 (0.153–7.510) |
| Commit median (min–max) | 0.529 (0.030–1.283) | 0.486 (0.034–1.275) |
| Paired save+Commit median (min–max) | 3.636 (0.182–8.832) | 3.468 (0.186–8.785) |
| Performance case wall | 56.192 | 56.146 |
| Verification case wall | 31.015 | 26.493 |
| Performance setup | 2.345 | 2.315 |
| Verification setup | 1.091 | 1.062 |
| Input validation/acquisition before performance | 2.946 | 2.971 |
| Input validation/acquisition before verification | 3.001 | 3.308 |
| Total input transfers | 7.374 | 7.350 |
| Performance wrapper cleanup | 0.545 | 0.596 |
| Verification wrapper cleanup | 0.381 | 0.389 |
| Host build | 68.650 | 69.192 |
| Linux image build | 17.995 | 1.454 |
| Complete performance command | 59.410 | 59.356 |
| Complete verification command | 34.288 | 30.061 |

One source history per arm, ten dependent observations. Min/median/max are descriptive, not independent repetitions or tail-confidence estimates. Public Exec/Commit timers exclude fixture preparation, transfers, census and verification. Verification includes complete content hashing and lifecycle work, so its wall is not a pure-read throughput result. Cold OS caches were not established.

One-time selected-input preparation took **12.498 s**. Warm performance plus verification commands took **93.698 s** for v0.1.4 and **89.417 s** for v0.1.5. No unchanged successful arm was repeated.

Git construction took **26.215 s**, final delta packing **11.923 s**, and the complete native Git command **46.000 s**. Final original-oracle content verification took **1.552 s**, additionally to tree/membership/fsck work. Git 2.47.1 uses compression 6/window 10/depth 50/threads 2, as in the historical reference. Its final pack contains 24,897 deltified objects out of 40,763 total objects. Git construction and later packing have different boundaries from foreground LayerFS saves; no cross-system save-latency speedup is claimed.

## Storage attribution

| Physical component, bytes | v0.1.4 | Existing v0.1.5 |
|---|---:|---:|
| File-content packs | 48,973,657 | 54,245,616 |
| Metadata/legacy packs | 9,544,748 | 6,559,114 |
| Locator B-tree pages | 7,667,712 | 4,087,808 |
| All pack BLOBs | 58,518,405 | 60,804,730 |
| SQLite freelist pages | 0 | 0 |

The candidate really emits pack-v3 SmallContent: **14,646 FULL / 18,571 DELTA** objects. FULL/DELTA frame bytes are **30,191,026 / 19,361,953**. Its **7,548** selected physical FULL bases retain **17,300,067** record bytes; every new DELTA has one FULL-base edge. These base bytes are already included in FULL totals, not added twice.

The candidate adds 5,271,959 content-pack bytes, removes 2,985,634 metadata/legacy-pack bytes, and removes 3,579,904 locator-page bytes. Remaining SQLite/framing/allocation differences add 253,195 bytes, yielding the net 1,040,384-byte allocation reduction. Metadata/legacy packs include five old chunk objects. The complete census retains pack framing, SQLite leaf/internal/overflow payload/slack, canonical populations and physical base checks.

## Correctness, custody and resources

Both LayerFS arms completed **10 Created / 0 UpToDate** outcomes. All ten mappings were retained and verified against the original full-byte/path/type/mode/symlink oracles after checking the measured Store digest and reopening the SAME Store in a new coordinator. Both performance and verification reported clean coordinator shutdown and container removal. Verification lifecycle mutations occur after frozen allocation; no second history was substituted.

Git has ten deterministic linear commits with the exact selected root trees, strict fsck and exact object membership, no alternates and no original source commits. Every blob is SHA256-checked against the original oracles from that same retained repository. Its historical metadata scope is paths, executable bits, symlinks and content, not every LayerFS POSIX metadata field.

| Resource observation | v0.1.4 | Existing v0.1.5 |
|---|---:|---:|
| Performance host lifetime peak RSS, bytes | 110,837,760 | 120,406,016 |
| Verification host lifetime peak RSS, bytes | 68,124,672 | 71,581,696 |
| Performance boundary spool maximum, bytes | 45,056 | 45,056 |
| Performance boundary staging maximum, bytes | 71,393,280 | 71,393,280 |

No observed OOM or swap events. Container policy stays 2 CPUs/2 GiB/no swap/256 PIDs; host and container scopes remain separate. Lifetime RSS and boundary disk samples are not phase-local allocation peaks. Git RSS and continuous spool peaks were not collected. Full container category samples and image inspections remain in raw evidence. Builds used shared caches; candidate Linux image reused its compiled layers. No broad Cargo test/Clippy/doctest or unrelated benchmark ran.

## Source identities

| Identity | v0.1.4 | Existing v0.1.5 |
|---|---|---|
| Measured source commit | `26a80a8efae63b7606c845f20c69aa3a1ccef292` | `bb2474ae88fa60a0d62e689f419edfe1d568e255` |
| Product seal | `891067915b4152dd42e7510ef374a3d8a8827ead03a597051335c933d0485958` | `557a420d05d1b6cbd18e1b06c55a824c2fbe0ca2fdee596c3231398496a74f8a` |
| Combined source seal | `16159d88c1b8c0cf3b6887168690a8f3cf368a70893d9cedc2f9f3012f656987` | `aa8022757f47b42e69cf6d5315da30acce16c74bc5e40521e20d04231b44a4fc` |
| Host binary SHA256 | `ecb4d5bec62c061b32a92fb1d2b2956390ac55042ac0e7611286ce24d6c3d915` | `b1f8fc5b9a033e447f50596fe03178e0c34dbb53ef56c7d91a8b10832c68f780` |
| Linux image ID | `sha256:a84448f29032a904171ed50edb445dbbefb10074c974cfe79ddeeff00e571773` | `sha256:4bf528c07e96846c5d7ed6c10ff3f31db4eb0e4b5ba23e898daeb6aa194a2675` |
| Measured Store SHA256 before reopen | `54fb013c3bc18d4514d4c6bc88f0b4fe0dcfa051c914e06d608367443be52653` | `f25929903913a7848ddf1fa3335ba69a50bf03f3d011a06fe34676697bb9073e` |

Control product files, Cargo manifests/lock and tools are byte-identical to release `101fa273d815f3aaedb0e06ba0de7b0777d83def`. Candidate product files are byte-identical to the already verified CAS-fix candidate `7ca59e244`. Both arms share all 110 recorded harness source-file hashes. Dirty status is preserved: both have the shared `target` symlink; candidate also had the report-only summarize.py adaptation, retained in its dirty patch. No product patch was hidden or stale binary relabeled.

Fixture selection SHA256: `6a28059656b671edc6f068d578a0f4670bfc7519aa0f54ee9dda10941cc7f64a`. Original manifest SHA256 remains `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Every selected original tree and oracle is recorded in fixture.json.

## Selection and per-step results

| Smoke step | Full157 index | Source commit | Files | Logical bytes |
|---:|---:|---|---:|---:|
| 1 | 1 | `ca4e1c3a1c61` | 276 | 1,499,109 |
| 2 | 18 | `2a48b31306ce` | 1,335 | 9,776,685 |
| 3 | 36 | `e29db1456664` | 2,062 | 15,073,486 |
| 4 | 53 | `89d4643c26e9` | 3,526 | 22,364,863 |
| 5 | 70 | `9e6f251a4c0e` | 4,955 | 29,109,246 |
| 6 | 88 | `7248b5ec8f87` | 5,742 | 36,405,983 |
| 7 | 105 | `5ba7455091ba` | 6,554 | 42,587,599 |
| 8 | 122 | `37247ea70544` | 7,364 | 48,605,608 |
| 9 | 140 | `6770f76fdaf9` | 8,658 | 57,441,764 |
| 10 | 157 | `b0a7d2ce3b4c` | 9,415 | 65,020,822 |

[Per-snapshot allocation/latency CSV](ten-snapshot-comparison.csv) and [complete LayerFS comparison JSON](ten-snapshot-comparison.json) retain every observation and slow checkpoint. [Git results](ten-snapshot-git.json) retain its construction, packing and same-repository proof.

Raw evidence, Stores, Git repository, saved binaries, command logs, image inspections and dirty patches:

```text
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence
```

Each `*-command.json` records exact arguments, cwd, command wall and exit status. `run-layerfs.py` retains the serialized build/performance/census/verification sequence. Reuse the input cache at `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`; successful raw outputs must not be overwritten.

## Interpretation and remaining work

This is the requested three-baseline smoke. It is useful for fast implementation/correctness iteration and includes mature repository states, but its ten retained versions do not reproduce the long anchor lifetimes of full157. In the earlier complete history, unchanged v0.1.5 was 9.10% larger than released v0.1.4; this ten-state 1.55% reduction does not supersede that result or qualify full157 savings. No full157 run was repeated for this baseline task.

The selected baselines had no failed or rejected execution attempts. The earlier CAS compile failure, reproduced correctness failure and full157 regression remain preserved in `layerfs-issue100-evidence`; the old rejected metadata-anchor experiment remains preserved separately. No storage optimization is accepted from these baseline observations, no user-approved numeric PASS is claimed, and no release is published or tagged. The next optimization remains subject to the original design constraints and any explicit owner-approved revision.

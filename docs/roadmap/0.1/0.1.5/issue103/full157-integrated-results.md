66 MB storage target: **ACHIEVED**.

The promoted product's complete full157 Store occupies **55,476,224 allocated
bytes (55.476224 decimal MB)** after one supported public compaction, and **all157
original states verify successfully through the measured Store**. The margin
below66,000,000 bytes is **10,523,776 bytes (10.523776 MB)**. All11 historical-access
performance cases and their11 separate verification runs pass the unchanged
15-second contracts. There are **no read-limit failures**.

The owner authorized full157 after the preceding successful stride3 run. This is
one original full157 history, one compaction and one same-Store verification, on
the unchanged promoted product. No broad #102 campaign or release was performed.
[Prospective full157 contract](full157-integrated-compaction-v1.md),
[preceding stride3 result](stride3-integrated-results.md),
[machine-readable results](full157-integrated-results.json), and
[seal audit](full157-seal-audit.json).

## Product and execution

The same schema10 product integrates compact scoped serial inode identities,
inline inode records and compact directory references; authenticated shared
metadata-value groups and bounded deltas; whole-file FULL/prefix graphs and
authenticated native chunk slices; compact framing and4KiB SQLite pages. Public
initialization, ordinary writes/Exec, Commit, reopen, fork, reconciliation and
historical reads use these implementations. Schema6–9 keep the supported
nonpromoting compatibility behavior. Detailed format/version, allocation,
authentication, reference closure, scratch and recovery contracts are in the
[integration history](integration-progress.md).

**Explicit compaction is required for this measured result.** The supported
`LayerStackStore::compact_into(destination, CompactionOptions)` API, SDK exports
and `layerfs-store-compact` product command select from authenticated, already
published Store content. Larger-content-first processing and up to four existing
min-hash candidates replace the experiment's Git-selected graph. Runtime requires
no Git repository, fixture oracle or future snapshot. Whole-file owners cover
128KiB through2MiB; bounded native FULL fallback preserves larger-file semantics.
Every prefix base, full owner, original native slice identity and metadata group/
intermediate leaf remains authenticated. Canonical IDs and logical records are
preserved across compaction; the source and destination are independently usable.

No product code changed for full157. Only Python runner registration, profile
custody and access-fixture binding changed after the prospective contract commit.
The rebuilt host, compactor and all three Linux binaries are **byte-identical to
the stride3 executables**. The qualified linked probe again confirms schema10,
LFS6FSR compact namespace, metadata pools and content107 FULL/prefix/slices.
The earlier focused342-test proof (four existing ignored) and live Exec/FUSE
lifecycle proof apply to this unchanged product; the updated runner passes52
product-free tests and the repository-history registry self-check.

## Complete storage and original-state proof

| History/boundary | Allocated bytes | Decimal MB | Verified states and publications |
| --- | ---: | ---: | --- |
| Preceding integrated stride3, compacted | 46,202,880 | 46.202880 | 53/53 PASS;53 Created |
| Integrated full157, before compaction | 83,935,232 | 83.935232 | 157 original public save/Commit attempts |
| **Integrated full157, compacted** | **55,476,224** | **55.476224** | **157/157 PASS;157 Created** |
| Historical offline full157 reference | 65,957,888 | 65.957888 | Recorded selected optimized artifact |
| Historical Git157 reference | 56,373,248 | 56.373248 | Recorded matching history |

Full157 has **157 Created, zero UpToDate and zero presentation failures**. Original
checkpoints1 through157 are retained in order, with no hidden commits, skipped
states or workload reduction. All **904,143 path-states and4,936,693,030 logical
bytes** match the original content and metadata oracles through public reads of
the same measured product Store. Producer and verifier cleanup pass.

The complete measured directory contains one SQLite file, allocated and apparent
length both55,476,224 bytes, with no required sidecars. Indexes, pools, physical
bases, native adapters, all logical tables and SQLite allocation are included.
Pack payload alone is49,409,357 bytes and is not used as the storage headline.
The retained precompaction source is an independent evidence copy, not a required
dependency; retaining both source and compacted destination uses139,411,456
allocated bytes. Compaction saves28,459,008 allocated bytes.

Before verification, the measured Store was frozen at inode783807449/device16777234,
SHA256 `f4a7a7b5b02d1ed7d8444f68cb9dbb94406c5c5e28deb7ba46467c05739c3b97`.
A closed, byte-identical preimage is retained in `frozen-measured-store/` and is
the master for historical-access copies. Verification then reopened the actual
measured `compacted-store/store.sqlite` path and created ordinary verification
forks. Those writes increased allocation to56,524,800 bytes (+1,048,576) and
apparent length to55,513,088 bytes (+36,864). The post-verification SHA is
`d08220b119965bf62b381dca7d93ef70b1f7f779b7aa9cbb8dd8749d1f1d6236`.
This growth is reported separately; the frozen preimage and source remain intact.

## Offline/Git comparison and causes

The integrated Store saves **10,481,664 bytes (15.89%)** versus selected offline
full157 and **897,024 bytes (1.59%)** versus recorded Git157. These are historical
storage comparisons, not fresh paired speed measurements. The preceding stride3
comparisons remain46,202,880 versus54,382,592 offline and49,332,224 Git bytes.
The same product policy adds9,273,344 allocated bytes from53 to157 retained states;
this is a two-workload observation, not an asymptotic scaling claim.

| Complete physical accounting | Offline157 B | Product157 B | Product saving B |
| --- | ---: | ---: | ---: |
| Content pack payload | 50,710,265 | 40,797,486 | 9,912,779 |
| Other payload, including metadata/pools | 9,603,174 | 8,611,871 | 991,303 |
| All pack payload | 60,313,439 | 49,409,357 | 10,904,082 |
| SQLite allocation outside payload | 5,644,449 | 6,066,867 | -422,418 |
| **Complete allocated Store** | **65,957,888** | **55,476,224** | **10,481,664** |

Read-only comparison finds all **75,398 SmallContent IDs/lengths,523 whole-owner
IDs/lengths and3,221 offline native-slice IDs/lengths** present and identical.
The product content containers additionally preserve five native FULL records.
No file-state omission or payload-identity change accounts for the content saving.

Product selection yields69,650 Small prefixes versus63,272 offline, replacing
6,378 FULL records and saving10,055,283 Small record bytes. Whole-file records
are **142,366 bytes larger** despite504 prefixes versus498: the product-owned
bounded selection graph differs from the Git-selected graph. Six fewer whole
FULL records do not imply a globally better whole-file encoding. Additional
native FULL and container overhead are102 and36 bytes. Together these explain
the9,912,779-byte content-payload saving.

There are253,706 candidate trials. Maximum selected depth reaches the frozen
50-edge limit; maximum canonical closure is25,489,771 bytes and encoded closure
483,545 bytes, within the64MiB limits. The fallback/depth policy was not loosened.
Product pool construction has879 groups/89,575 values versus543/89,576 offline.
Public namespace construction, durable serial reservations and incremental pool
formation differ from offline namespace rewriting. The991,303-byte noncontent
payload saving is measured in aggregate; attribution among those metadata
mechanisms would need an isolated experiment and is not claimed. Additional
SQLite overhead is fully charged.

## Save, Commit, compaction and verification costs

| Full157 operation | Sum s | Minimum s | Median s | p95 s | Maximum s |
| --- | ---: | ---: | ---: | ---: | ---: |
| Exec | 314.027550 | 0.111955 | 1.712955 | 5.142147 | 8.637586 |
| Commit | 62.689607 | 0.027938 | 0.326090 | 0.966821 | 1.513716 |

Exec+Commit sum is376.717157 seconds. The save work wall is519.681190 seconds,
including transfer/orchestration. Required compaction takes626.313062 seconds
in the public API and626.518567 seconds process wall. Exec+Commit+compaction API
sum is1003.030219 seconds. Save work wall+compaction process wall is1146.199757
seconds. The full performance wrapper is1221.527622 seconds, including72.796147
seconds of immutable preparation and remaining setup/cleanup/receipt overhead.

Compaction phase seconds: inventory29.537726, owner construction13.999629,
encoding373.241033, VACUUM0.172365, intrinsic verification198.294967 and
publication0.025025. It byte-verifies **104,705 original objects and523 new owners**
and validates logical records/closure before publication. Source preservation,
publication, directory sync and cleanup are all true, with no publication notes.

Original-state verification costs778.616960 seconds of work; its full wrapper is
866.655963 seconds, including70.788461 seconds of original-input validation.
These are a single qualified run's dependent checkpoint observations, not a fresh
Git speed comparison or repeated-trial statistics.

Save host peak RSS is132,431,872 bytes, Linux cgroup peak155,566,080 bytes,
maximum spool allocation9,101,312 bytes and container staging70,705,152 bytes.
No OOM, OOM-kill or swap was observed. Compaction reports619.403201 CPU seconds,
**131,792,896-byte peak RSS**,140,869,632 disk-read bytes and139,276,288 disk-write
bytes. `/usr/bin/time -l` records626.47 real/512.20 user/107.31 system seconds,
the same peak RSS,42,369,528-byte peak footprint and zero swaps.

The temporary budget remains4,294,967,296 bytes. Named growth-boundary sampling
and6,031 open-descriptor samples both observe **407,638,016-byte temporary peak**;
no unlinked allocation was observed. These are sampled maxima, not proof that a
shorter-lived unlinked file could never exist. The preserved source is additional:
source plus observed temporary peak is491,573,248 bytes. Output construction is
included in temporary accounting and becomes the final Store on publication.

## Historical access at original full157 checkpoints

All11 performance cases and all11 verification runs PASS, including preparation,
receipts and cleanup within15 seconds. Performance totals range1.993534–2.225822
seconds; verification totals1.985693–2.224085 seconds. **Read-limit failures:0.**

The prospectively registered `historical-access-full157-integrated-v1` fixture
uses `-f157-v1` case IDs and the original checkpoints1/157/65/57, which equal their
retained ordinals. Paths, offsets, lengths and cache policies match the original
historical-access-v2 templates. All expected content/metadata values were derived
from original sealed oracles and checked equal to the original templates. The
stride3 mappings65→67 and57→58 are not used. Both earlier fixtures stay unchanged.

| Case (`ha-…-f157-v1`) | Checkpoint | Public access ms | POSIX operation ms | Performance total s | Verification total s | Decoded B |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| stat-old | 1 | 21.240 | 18.439 | 2.225822 | 2.097827 | 3,497,362 |
| stat-head | 157 | 37.239 | 33.286 | 2.015814 | 2.103016 | 8,526,828 |
| directory-old | 1 | 21.027 | 17.776 | 2.073356 | 1.998863 | 3,497,362 |
| directory-head | 157 | 34.142 | 31.076 | 2.012220 | 1.985693 | 8,526,828 |
| small-old | 1 | 20.345 | 17.038 | 2.017730 | 2.162140 | 3,497,362 |
| small-head | 157 | 33.639 | 30.870 | 1.993534 | 2.093518 | 8,526,828 |
| range-history-cold | 65 | 49.571 | 46.191 | 2.117008 | 2.110064 | 22,216,028 |
| range-history-warm | 65 | 2.987 | 0.074 | 2.107997 | 2.224085 | 0 |
| full-head-cold | 157 | 73.175 | 65.909 | 2.112542 | 2.154199 | 25,684,859 |
| full-head-warm | 157 | 7.278 | 0.402 | 2.023601 | 2.199983 | 0 |
| metadata-worst | 57 | 70.011 | 66.934 | 2.124474 | 2.063032 | 14,586,582 |

Public access includes execution/IPC around the POSIX operation; only the complete
end-to-end totals determine the15-second gate. Cold/warm follow the declared
reader-cache policy; OS caches remain uncontrolled. **Cold amplification remains a
material limitation:** the6421-byte range of `pnpm-lock.yaml` at checkpoint65,
offset537371, reads2,603,234 encoded and22,216,028 decoded bytes, including
3,229,988 pool-decoded bytes. Full-head cold reads5,508,571 encoded/25,684,859
decoded bytes for841,964 requested bytes. The original checkpoint57 metadata
case reads5,596,614 encoded/14,586,582 decoded bytes. Its path remains
`.agents/notes/implemented/architecture/2026-07-19-gui-layering-and-rpc-protocol.zh.md`.
Warm range/full cases have zero Store encoded/decoded work. Passing these time
and work limits does not establish generally low amplification or full release
qualification. Full physical counters are retained in the result JSON.

## Source, build, fixture and evidence custody

Starting worktree `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb`, branch
`codex/issue100-40mb-experiments`, clean HEAD`efda330a72e51c7925b9e11a0ba960fb432733a9`.
Branch/HEAD/dirty state, active processes, free space and measurement lock were
checked before work. No benchmark/build was active. Prior drafts, evidence and
binary arms remain preserved. Contract commit`937bc2e58c9060df13086be10c4d84b5b0a44b9c`
precedes the runner changes; the measured candidate is below. This final report
is a later documentation-only commit, not a replacement build identity.

| Identity | Exact value |
| --- | --- |
| Measured commit | `786d29575b1b7cf1123b5f9b8c97f1e4610c2bab` |
| Measured tree | `841f4f5e8d438a96e79ccab9c84c01b94a16d141` |
| Dirty at build | `false` |
| Source seal | `891e210cdbbe7218b0252410184a84a918f07b61d71368a938dc938fb0d2ecb9` |
| Product seal | `b1a94e2223b1cd6c0eedffa3c6c60eca7134727c45b9018e1cea518cdf6d3dd5` |
| Host SHA256 | `70edec49418aec3f70f5ddbce5743838fbba3e80766cd0940939659f085db417` |
| Compactor SHA256 | `82d94c206243d45e2c64e1fe73cc18f5eca7f0705ced8553633d63bc206fb7d8` |
| Linked schema SQL SHA256 | `1efbb2247d23efd645531e502512245b60ea7e8745e6c937000ca67ccdea7f5a` |
| Linux image | `sha256:8bb710bcd3736d161d32800b72ef1cc834b39a45cd2adf2987aacb9f39f9ec8c` |
| fs-benchmark-workload SHA256 | `e2ae9a6b0859e3542f58dbf93e9088fe9f94e047cd205da8d927e3219d21aa9a` |
| layerfs-daemon SHA256 | `982b20a100535363884ca02060c1db3e952fe7614f2d558e977f7fc64f2625b2` |
| layerfs-fuse SHA256 | `f0b194ab3dbfab50005d0cc0cd818f8b733e896869281dad472d2116f69c3686` |
| Original manifest SHA256 | `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271` |
| Original source tip | `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed` |
| Access fixture SHA256 | `f47663f4ca28162f93a897942aaa9c8f27e9a8e036f69444193af9217868bba7` |
| Frozen Store SHA256 | `f4a7a7b5b02d1ed7d8444f68cb9dbb94406c5c5e28deb7ba46467c05739c3b97` |

Host macOS26.4.1 arm64, Rust1.85.1, linked system SQLite3.51.0. Qualified builds
use the existing runner-owned lock and source-seal-isolated target. Host/compactor
and original identities were archived before switching; Linux daemon/FUSE/workload
bytes were archived from the exact image. SQLite, coordinator, canonical
construction/publication, spool and compaction remain on macOS; Docker runs only
the declared daemon/FUSE/workload roles. No previous schema9/initial-1 evidence
was relabeled to qualify this format.

The final audit passes **330 performance-manifest files,811 verification-manifest
files,157 original oracle seals and all22 historical-access manifests/completions**,
plus frozen Store/source preservation and archived executable identities. For the
performance manifest, the original measured Store hash is checked against its
retained identical preverification image; verification-only writes remain explicit.

Raw evidence root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/`.

- `full157-handoff-20260910/`: preserved starting identity, original manifest and
  prospective scope/checkpoint custody.
- `qualified-full157-786d29575/`: host, compactor, sidecar identities and exact
  Linux binaries/image identity; prior stride3 archives remain untouched.
- `full157-runner-tests-1.log`, `full157-qualified-host-build-1.log`,
  `full157-qualified-image-build-1.log`.
- `full157-integrated-1/`: complete producer/verification manifests and receipts,
  compaction invocation/result/time/trace, source Store, measured Store, frozen
  preimage,157 original-oracle observations and before/after allocation custody.
- `full157-integrated-1.log`, `full157-verification-1.log`,
  `historical-access-full157-1.json`, `full157-historical-access-prepare-1.log`,
  `full157-historical-access-{performance,verification}-1/` and corresponding logs.
- `full157-analysis/`: read-only SQL/object comparison, reproducible report
  analysis, summary and seal audit. Result JSON records retained evidence hashes.

There were no failed full157 producer, compaction, verification or access attempts,
and no unchanged-candidate rerun. Earlier integration/stride3 failures remain
preserved under their original names.

Exact executed entrypoints, from the worktree:

```sh
EVIDENCE=/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence
RUN="$EVIDENCE/full157-integrated-1"
STORE="$RUN/deepseek-full/frozen-measured-store/store.sqlite"
IMAGE=sha256:8bb710bcd3736d161d32800b72ef1cc834b39a45cd2adf2987aacb9f39f9ec8c
RUNNER=benchmark/fs-bench-pro/shared/runner.py
python3 -m unittest discover -s benchmark/fs-bench-pro/shared -p 'test_*.py'
python3 "$RUNNER" --family repository_history --self-check
python3 "$RUNNER" --build-host
python3 "$RUNNER" --build-image
python3 "$RUNNER" --family repository_history --profile stride-1 --storage-compact --image "$IMAGE" --output "$RUN"
python3 "$RUNNER" --family repository_history --profile stride-1 --image "$IMAGE" --storage-verify-run "$RUN"
python3 benchmark/fs-bench-pro/shared/integrated_storage.py --prepare-access "$RUN" --output "$EVIDENCE/historical-access-full157-1.json"
python3 "$RUNNER" --family historical_access --all --fixture "$EVIDENCE/historical-access-full157-1.json" --store "$STORE" --image "$IMAGE" --output "$EVIDENCE/full157-historical-access-performance-1"
# For all11 exact fixture IDs, serially:
python3 "$RUNNER" --family historical_access --case "$CASE_ID" --fixture "$EVIDENCE/historical-access-full157-1.json" --store "$STORE" --image "$IMAGE" --mode verification --performance "$EVIDENCE/full157-historical-access-performance-1/$CASE_ID/result.json" --output "$EVIDENCE/full157-historical-access-verification-1/$CASE_ID"
python3 "$EVIDENCE/full157-analysis/analyze.py"
python3 "$EVIDENCE/full157-analysis/audit.py"
```

The compaction receipt retains the exact `fs-benchmark-pro storage-compact`
argument list, source/destination and4294967296-byte temporary limit. It calls
one supported product API operation; Python only orchestrates and freezes custody.

## Issue103 completion and remaining qualification

The promoted integration, focused product correctness, stride3-first storage
proof, full15766MB criterion, original-state verification and historical-access
obligations are satisfied. This candidate and the exact evidence above are the
handoff to #102. Issue103 can close on these results; #102's full mandatory
registry and matched released-control comparison remain outstanding.

The required compaction cost,50-edge content chains and substantial cold-read
amplification remain explicit product tradeoffs. No full #102 benchmark campaign,
release, tag, deployment, routine cache pruning or unrelated cleanup was performed.

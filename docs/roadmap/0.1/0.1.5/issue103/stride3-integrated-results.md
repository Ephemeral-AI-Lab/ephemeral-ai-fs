66 MB storage target: **NOT QUALIFIED**.

The user revised execution to **full promoted-method integration and stride3 only**.
Full157 was intentionally not run; its allocated bytes, verified count and margin
against 66,000,000 bytes are unavailable. No full157 or release claim follows
from this 53-state result.

**Stride3 integration/storage verification: PASS.** The complete compacted product
Store occupies **46,202,880 allocated bytes (46.202880 decimal MB)** and verifies
**53/53 original states**. This is 8,179,712 bytes below the selected offline53
reference and 3,129,344 bytes below the recorded Git53 reference. Explicit public
compaction is required for this measured result and cost **339.133 seconds**.

Measured on 2026-09-10 at candidate `80bc489281735c889a4d62ed576135586be3365b`.
[Machine-readable results](stride3-integrated-results.json),
[seal audit](stride3-seal-audit.json),
[integration history and retained failures](integration-progress.md), and
[prospective measurement contract](stride3-integrated-compaction-v1.md).

## Integrated product and selection policy

New schema-10 Stores use compact scoped serial inode identities, inline inode
values and compact directory references through public empty/directory
initialization, ordinary writes/Exec, Commit, reconnect, fork, reconciliation and
historical reads. Durable serial reservations precede publication and burn unused
ranges; they do not use fixture-selected namespace identities. The existing
bounded tree, admission, spool, reconciliation and publication owners are reused.

Metadata pack v6 carries authenticated shared physical value groups and bounded
metadata deltas. The catalogue has one row per group, not per-value CAS/index
rows. Product-owned append-only ordinals preserve canonical leaf identities.
Every full group digest and intermediate reconstructed canonical leaf is checked.
Groups contain at most 165 values/16 KiB; metadata chains retain 16-edge/128-KiB
canonical limits. The bounded disposable macOS lookup index is charged as scratch
and rebuilt from authenticated published groups on reopen. Detailed encoded-work,
cache and fallback limits are recorded in the integration history.

The supported **`LayerStackStore::compact_into(destination, CompactionOptions)`**
API (exported by the SDK) and **`layerfs-store-compact SOURCE DESTINATION
[TEMPORARY_BYTE_LIMIT]`** product command produce a self-contained destination.
They inventory authenticated, already published Store content and build whole-file
owners for 128-KiB through 2-MiB regular files. Selection reuses the existing eight
min-hash signature, processes larger content first and trials at most four
same-role already selected candidates. It requires neither Git nor oracle/path
knowledge nor snapshots that have not been published. It can use any state
already present when the caller explicitly requests compaction.

LFCNT1/version-107 containers carry SmallContent and whole-file FULL/prefix records
plus native chunk slices. Reading a slice authenticates the whole owner and the
original reconstructed native chunk. Every intermediate prefix base is checked.
Native FULL fallback retains support for ordinary files outside the owner size
window. Content chains allow at most 50 edges and 64 MiB canonical/encoded
closure; the two-pass reader retains only bounded locations and has a 16-MiB
active scratch bound, including its bounded owner cache. Caller output batches
are separately charged. New SmallContent admission uses compact pack-v4 framing;
compaction uses four-byte starts and the selected **4096-byte SQLite layout**.

Compaction preserves canonical IDs and logical SQL rows, checks every original
object byte-for-byte and added whole owner, checks logical-table digests and
reachable closure, then atomically publishes a new destination without clobbering
an existing path. It syncs the file and parent directory and reports publication
and cleanup separately. Source and destination are independently usable. Future
ordinary writes/Commit, fork/reopen and repeated compaction are covered by public
tests. Compaction does not run secretly during fixture preparation.

Schema 6–9 retain their existing nonpromoting read/write behavior; the explicit
7/8-to-9 upgrade still targets 9. There is no silent schema9 identity migration,
and experimental schema9304 is not accepted. This remains a development candidate,
with qualification limited to the tests and stride3 cases reported here.

## Storage, state verification and publications

| Store/history | Complete allocated B | Decimal MB | Verification/publication |
| --- | ---: | ---: | --- |
| Integrated stride3, before compaction | 65,056,768 | 65.056768 | 53 Created; 0 UpToDate; 0 presentation failures |
| **Integrated stride3, compacted** | **46,202,880** | **46.202880** | **53/53 original states PASS**; compaction published, synced, cleaned |
| Selected offline stride3 reference | 54,382,592 | 54.382592 | Historical reference, 53 original states |
| Matching Git53 reference | 49,332,224 | 49.332224 | Historical storage reference |
| Integrated full157 | Not run | — | Outside revised scope; NOT QUALIFIED |

The workload retains original indices **1,4,7,…,157** through 53 direct transitions.
There are no hidden commits for skipped snapshots. Verification covers **306,861
path-states and 1,676,767,835 logical bytes**, including original content and
metadata oracles, through the measured public product Store. All 53 oracle files
were rechecked against their original seals. Producer and verifier cleanup passed.

The final measurement is the sum of allocated blocks of every file in the measured
Store directory. It contains one SQLite file, with apparent length also
46,202,880 bytes and no required sidecars. Indexes, pools, physical bases, adapters,
all logical tables and allocation overhead are included. Pack payload alone is
42,139,439 bytes and is reported separately below. The retained source is an
independent precompaction Store, not a dependency of the destination; it remains
preserved as evidence. Keeping both costs 111,259,648 allocated bytes.

Identity and allocation were frozen **before verification**. The measured inode
783743561 on device 16777234 had SHA256
`a91ff533ab0d3d48c0618983b911cc6638d711c3cc5760172a5ae76b1dd7cf1f`.
The closed byte-identical preimage was preserved at
`stride3-integrated-1/deepseek-stride3/frozen-measured-store/store.sqlite`.
All 53 states were then verified **in place on the actual measured Store**, with
public verification forks. Those verification-only writes grew its allocation to
47,251,456 bytes and apparent length to 46,211,072 bytes: +1,048,576 allocated and
+8,192 apparent. Both before/after manifests are retained. The frozen preimage
remains unchanged and supplies the historical-access copies. Verification growth
is not substituted for the frozen workload measurement.

## Difference from the offline result

| Physical accounting | Offline53 B | Integrated53 B | Integrated saving B |
| --- | ---: | ---: | ---: |
| Content pack payload | 45,494,501 | 37,402,796 | 8,091,705 |
| Other pack payload, including metadata/pools | 5,003,685 | 4,736,643 | 267,042 |
| All pack payload | 50,498,186 | 42,139,439 | 8,358,747 |
| SQLite allocation outside payload | 3,884,406 | 4,063,441 | -179,035 |
| **Complete allocated Store** | **54,382,592** | **46,202,880** | **8,179,712** |

A read-only comparison found **all 59,768 SmallContent IDs/lengths and all 224
whole-owner IDs/lengths identical** across product and offline Stores. All 1,998
offline native slice IDs/lengths are also present; the product content container
additionally includes five native FULL records. Content savings do not come from
omitting file states or changing these payload identities.

The product selects 54,076 Small prefixes versus 48,494 offline, replacing 5,582
FULL records; Small record bytes fall by 8,026,741. It selects 205 whole prefixes
versus 199, replacing six FULL records and saving 65,086 record bytes. The remaining
content difference is 102 additional native FULL bytes and 20 directory bytes.
The product's bounded four-candidate signature search over published Store content
replaces the offline Git-selected graph and produces this different encoding.
There were 191,876 trials; maximum selected depth was 45, canonical closure
16,911,255 bytes and encoded closure 483,545 bytes.

Namespace tree construction, durable serial allocation and incremental pool-group
formation also differ from offline rewriting. Product has 523 pool groups/66,528
values versus 404/66,529 offline. The aggregate 267,042-byte noncontent payload
saving is measured; assigning it among those metadata mechanisms would require
an isolated experiment and is not claimed here. SQLite overhead rises by 179,035
bytes, already charged in the final total.

Relative allocated savings are **15.04% against offline53** and **6.34% against
Git53**. Git53 is the previously recorded 49,332,224-byte reference; it was not
rerun as a fresh paired speed control. The historical offline full157/Git157
references remain 65,957,888/56,373,248 bytes; neither is a measurement of this
integrated candidate. Full157 margin to 66,000,000 bytes is **unavailable**.

## Costs and resource use

| Public save phase, 53 transitions | Sum s | Median s | p95 s | Maximum s |
| --- | ---: | ---: | ---: | ---: |
| Exec | 118.863256 | 1.989955 | 5.708112 | 6.119502 |
| Commit | 23.732028 | 0.399612 | 1.159194 | 1.282634 |

Exec plus Commit sum is 142.595284 seconds. The save phase wall time, including
transfer/orchestration, is 187.880254 seconds. Explicit compaction takes
339.132858 seconds in the API and 339.266211 seconds process wall. Save phase wall
plus compaction process wall is 527.146464 seconds; the entire performance wrapper
is 558.904392 seconds, including 29.849888 seconds of preparation and cleanup/
receipt overhead. These are one qualified run's observations, not repeated-trial
medians or a fresh Git speed comparison.

Compaction phase seconds: inventory 15.304159; owner construction 9.265679;
encoding 216.558465; VACUUM 0.139119; intrinsic verification 94.184399; publication
0.023998. The public operation verifies all **73,476 original objects and 224 new
owners** before publication. State verification adds 236.103555 seconds of work,
259.569175 seconds total wrapper time (including 8.831653 seconds preparation).

Save host peak RSS is 117,866,496 bytes; Linux cgroup peak is 125,497,344 bytes,
with zero OOM/OOM-kill/swap observations. Maximum physical spool allocation is
6,578,176 bytes and container staging allocation is 59,277,312 bytes.

Compaction reports 335.968232 CPU seconds, **135,069,696-byte peak RSS**, and
163,102,720/110,866,432 process disk read/write bytes. `/usr/bin/time -l` records
339.23 real, 292.27 user, 43.76 system seconds, the same peak RSS and zero swaps.
The configured temporary budget is 4,294,967,296 bytes. Peak named temporary
allocation sampled at growth boundaries is **281,907,200 bytes**. The 100-ms
open-descriptor stat sampler independently observes the same peak across 3,173
samples and no unlinked temporary allocation; this is a sampled observation,
not proof that a shorter-lived unlinked file could never exist. Source allocation
is additional: source plus observed temporary peak is 346,963,968 bytes. The final
46,202,880-byte destination is included in temporary construction accounting and
becomes the final Store after publication. Publication, directory sync, source
preservation and cleanup are all true, with no publication notes.

## Historical access: unchanged 15-second contracts

All **11 performance cases and their 11 verification runs PASS**. Performance
totals, including preparation/cleanup/receipts, range from 2.010149 to 2.145410
seconds; verification totals range from 2.048607 to 2.257637 seconds. There are
**no read-limit failures**. Seals and cleanup are checked for every case.

The explicit `historical-access-stride3-integrated-v1` profile uses case IDs ending
`-s3-v1`. Old/head cases remain original checkpoints 1/157 (retained ordinals1/53).
Original65 maps to **67 / ordinal23**, preserving `pnpm-lock.yaml`, offset537371
and length6421: retained64 is only541522 bytes and cannot supply that range.
Original57 metadata access maps to **58 / ordinal20**, preserving the original
path. Expected values come from the original sealed mapped checkpoints, not
product output. The existing full157 historical-access fixture is unchanged.

| Case (`ha-…-s3-v1`) | Original checkpoint | Public access ms | POSIX operation ms | Performance total s | Verification total s |
| --- | ---: | ---: | ---: | ---: | ---: |
| stat-old | 1 | 17.674 | 15.008 | 2.063881 | 2.053754 |
| stat-head | 157 | 32.586 | 29.498 | 2.077520 | 2.153049 |
| directory-old | 1 | 20.126 | 16.806 | 2.100000 | 2.086160 |
| directory-head | 157 | 34.598 | 31.835 | 2.145410 | 2.076923 |
| small-old | 1 | 17.808 | 15.289 | 2.010149 | 2.048607 |
| small-head | 157 | 33.649 | 31.035 | 2.108241 | 2.154539 |
| range-history-cold | 67 | 38.621 | 35.817 | 2.080916 | 2.166991 |
| range-history-warm | 67 | 3.297 | 0.058 | 2.103648 | 2.257637 |
| full-head-cold | 157 | 63.964 | 57.118 | 2.050118 | 2.157579 |
| full-head-warm | 157 | 7.199 | 0.510 | 2.100786 | 2.138834 |
| metadata-worst | 58 | 51.481 | 48.307 | 2.073953 | 2.173090 |

Public access includes execution/IPC around the POSIX operation; end-to-end totals
are the performance contract. Cold amplification remains substantial: the
6,421-byte range decodes **17,029,550 bytes** and reads 2,181,331 encoded bytes;
full-head cold decodes 19,245,707 bytes for an 841,964-byte file; the mapped metadata
case decodes 9,747,805 bytes. Pool-decoded components are respectively 2,286,950,
6,022,769 and 2,551,716 bytes. Warm range/full cases perform zero Store encoded/
decoded work. The cold cases pass their time/work bounds, but this is not a claim
of generally low amplification or full release qualification. Because the mapped
checkpoint differs, these numbers are not a paired improvement claim against the
old checkpoint65 reference. Complete physical counters are in the JSON report.

## Focused correctness and failed attempts

The affected native run passes **342 tests, four existing ignored**, covering
content, Store and workspace owners. It includes six public compaction tests,
eight public Store lifecycle tests, legacy staging and the existing doc test.
Coverage includes initialization/writes/Commit/reopen/fork, metadata/hardlinks,
file-size boundaries beyond 2 MiB, corrupt dependencies and malformed/bounded
reconstruction, admission/publication failure, actual mid-rewrite SQLite quota
failure, source preservation, existing destination refusal, cleanup and published
outcome reporting. SDK compile and compactor CLI checks also pass. Runner tests
pass 51 cases; the subsequent shell-composition regression test passes.

Qualified `live-integration-2` passes through four daemon Exec calls, two Created
commits, compaction and historical/fork/reopen reads/writes, with cleanup PASS.
Only then was the single stride3 producer/compaction run collected. No unchanged
storage candidate was rerun to obtain a better size.

Failures remain preserved: early compilation/test failures in the integration
history; Apple SQLite's VACUUM-into-existing-empty-file refusal (fixed by an absent
output in an owned private directory, retaining the actual linked SQLite);
`live-integration-1` shell separator failure before filesystem writes (fixed in
the shared builder); and `stride3-verification-1.log` Docker tag lookup failure
before opening the Store. Verification then used the exact already-built producer
image ID successfully. `compaction-invocation.json` is the immutable prelaunch
INCOMPLETE intent; `compaction-result.json` is the final PASS/source-preserved
receipt. Neither the failed attempt nor intent record was overwritten.

## Exact custody and retained commands

Worktree `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb`, branch
`codex/issue100-40mb-experiments`. Starting HEAD and dirty handoff were inspected
and preserved at `handoff-20260910T033600Z/`. Product implementation commit
`948068a48`, prospective freeze contract `4b6943623`, runner integration
`27677bbae`, and measured final shell-builder correction `80bc48928` remain
separate. This report is a later documentation-only commit; it does not replace
the following measured build identity.

| Identity | Exact value |
| --- | --- |
| Measured source commit | `80bc489281735c889a4d62ed576135586be3365b` |
| Measured Git tree | `5c0d353df36fbd994a056272b0ec0fce5e6f4020` |
| Dirty at build | `false` |
| Source seal | `cc9b90b7d5b09c9d7988e56c4c3e9cef80da00780836df840295f861fc9a0b57` |
| Product seal | `b1a94e2223b1cd6c0eedffa3c6c60eca7134727c45b9018e1cea518cdf6d3dd5` |
| Host binary SHA256 | `70edec49418aec3f70f5ddbce5743838fbba3e80766cd0940939659f085db417` |
| Compactor binary SHA256 | `82d94c206243d45e2c64e1fe73cc18f5eca7f0705ced8553633d63bc206fb7d8` |
| Linked schema SQL SHA256 | `1efbb2247d23efd645531e502512245b60ea7e8745e6c937000ca67ccdea7f5a` |
| Linux image | `sha256:315466b586327e7659d19d8636fc0244832271894a061e885e20ba67e12eb163` |
| fs-benchmark-workload SHA256 | `e2ae9a6b0859e3542f58dbf93e9088fe9f94e047cd205da8d927e3219d21aa9a` |
| layerfs-daemon SHA256 | `982b20a100535363884ca02060c1db3e952fe7614f2d558e977f7fc64f2625b2` |
| layerfs-fuse SHA256 | `f0b194ab3dbfab50005d0cc0cd818f8b733e896869281dad472d2116f69c3686` |
| Stride3 fixture SHA256 | `3b2c12b682e4892837fc810969e4e90eaa8d756c0d6f738df50746ad478fb73e` |
| Original oracle manifest SHA256 | `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271` |
| Fixture source tip | `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed` |
| Mapped access fixture SHA256 | `2b5a6e5cbdc48c321689c02e5a4458ff6d2795421ec6a7c9190b3d3a273f324d` |
| Frozen measured Store SHA256 | `a91ff533ab0d3d48c0618983b911cc6638d711c3cc5760172a5ae76b1dd7cf1f` |

Host is macOS26.4.1 arm64, Rust1.85.1, linked system SQLite3.51.0. The actual linked
probe observed schema10, compact LFS6FSR namespace, metadata pools and content107
FULL/prefix/slice emission. Host and compactor builds use source-seal-isolated
targets. Each switched host/compactor arm and original identity was archived;
Linux daemon/FUSE/workload bytes were archived from the exact image before use.
SQLite, coordinator, canonical construction/publication, spool and compaction
stay on macOS; Docker runs only daemon/FUSE/workload. Each measurement used the
existing runner-owned lock. The old schema9/invalid initial-1 evidence does not
qualify this build.

Raw evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/`.
The final audit checks 291 history-verification manifest files, all22 historical
access completion/manifests, all53 original oracle hashes, frozen Store bytes and
host/Linux binary identities. It is PASS. Machine-readable result evidence hashes
make the retained files identifiable without committing binaries or databases.

Executed from the worktree (variable names below abbreviate the exact paths):

```sh
EVIDENCE=/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence
CASE="$EVIDENCE/stride3-integrated-1/deepseek-stride3"
IMAGE=sha256:315466b586327e7659d19d8636fc0244832271894a061e885e20ba67e12eb163
TAG=layerfs-bench-infra:cc9b90b7d5b09c9d
RUNNER=benchmark/fs-bench-pro/shared/runner.py

cargo +1.85.1 test --locked -j2 -p layerfs-content -p layerfs-layerstack-store -p layerfs-workspace
cargo +1.85.1 check --locked -j2 -p layerfs-sdk
python3 "$RUNNER" --build-host
python3 "$RUNNER" --build-image
python3 "$RUNNER" --integration-smoke "$EVIDENCE/live-integration-2" --image "$TAG"
python3 "$RUNNER" --family repository_history --profile stride-3 --storage-compact --image "$TAG" --output "$EVIDENCE/stride3-integrated-1"
python3 "$RUNNER" --family repository_history --profile stride-3 --image "$IMAGE" --storage-verify-run "$EVIDENCE/stride3-integrated-1"
python3 benchmark/fs-bench-pro/shared/integrated_storage.py --prepare-access "$EVIDENCE/stride3-integrated-1" --output "$EVIDENCE/historical-access-stride3-1.json"
python3 "$RUNNER" --family historical_access --all --fixture "$EVIDENCE/historical-access-stride3-1.json" --store "$CASE/frozen-measured-store/store.sqlite" --image "$IMAGE" --output "$EVIDENCE/historical-access-performance-1"
# For each of the 11 exact IDs in historical-access-stride3-1.json, serially:
python3 "$RUNNER" --family historical_access --case "$CASE_ID" --fixture "$EVIDENCE/historical-access-stride3-1.json" --store "$CASE/frozen-measured-store/store.sqlite" --image "$IMAGE" --mode verification --performance "$EVIDENCE/historical-access-performance-1/$CASE_ID/result.json" --output "$EVIDENCE/historical-access-verification-1/$CASE_ID"
```

The actual measured compaction command is retained as a structured argument list
in `stride3-integrated-1/deepseek-stride3/compaction-result.json`: the qualified
host `fs-benchmark-pro storage-compact` calls the supported product API, with
source `host-runtime/store.sqlite`, destination `compacted-store/store.sqlite`,
temporary limit4294967296 and the external open-file trace path.

Retained evidence groups:

- `qualified-candidate-80bc48928/`: host/compactor binaries and identities;
  `linux/`: daemon/FUSE/workload binaries and image identity. Previous276 arm is
  retained in `qualified-candidate-27677bbae/` and runner archives.
- `whole-compaction-focused-suites-9.log`, `compaction-sdk-check-10.log`,
  `compaction-cli-help-10.log`, `integrated-runner-tests-7.log`,
  `integrated-harness-check-7.log`, `integration-script-tests-8.log`.
- `qualified-host-build-{1,2}.log`, `qualified-image-build-{1,2}.log`,
  `live-integration-{1,2}/` and their logs.
- `stride3-integrated-1/`: identity, sealed manifests, original save receipts,
  compaction receipt/time/resources/trace, original and compacted Stores, frozen
  preimage, all53 original-oracle observations, verification receipts and cleanup.
  Wrapper logs `stride3-integrated-1.log`, `stride3-verification-{1,2}.log`.
- `historical-access-stride3-1.json`, `historical-access-performance-1/`,
  `historical-access-verification-1/`, preparation/performance/verification logs.
- `final-analysis/summary.json`, `seal-audit.json` and read-only SQL analysis;
  committed result JSON retains all comparison totals and nonzero read counters.

The revised stride3 work is complete. **#103 remains open** because its original
full157 final-storage and #102 handoff obligations exceed the revised scope.
No full157 run, broad #102 campaign, release, tag, deployment, cache pruning or
unrelated cleanup was performed. Cold amplification remains the measured product
tradeoff to carry into any separately authorized qualification work.

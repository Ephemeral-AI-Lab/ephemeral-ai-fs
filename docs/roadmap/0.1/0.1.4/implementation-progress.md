# M4 independent review completed — stop before M5

2026-09-08. Three independent subagents reviewed physical encoding/admission/read
bounds, predecessor/capture/spill callers, and raw evidence/oracles. The review
found real source gaps in the earlier completion claim. They are corrected in
`de3a046f6a488ddd9d5f6710db8d98609b9983d6`, with fresh matching host/runtime
builds and all three approved smokes. **M4 implementation and scoped verification
are complete; performance diagnostics are not all passing.** No M5 work ran.

Current evidence: [review JSON](implementation-milestone-4-review.json).
The [original M4 JSON](implementation-milestone-4.json), M2/M3 JSON, and all earlier
raw runs remain unchanged. The historical ledger follows below.

## Review findings and disposition

| Finding | Shared-owner correction | Status |
| --- | --- | --- |
| Group-capacity accounting rescanned all groups for each group | Precompute fixed associations once; increment retained encoded capacity | Fixed |
| Directory predecessor misses still used scalar reader crossings | Batched unique directory-state and traversal-node waves; retain root/child validation and request order/duplicates | Fixed |
| Batch optional-fetch exhaustion reset on the next target | Persistent batch exhaustion separate from per-target reset; stop further locator/header reads | Fixed |
| Absent-predecessor metric meant empty hint list | Carry retained-predecessor presence through private hints/spill; separate `targets_without_hints` | Fixed |
| Correspondence reservation left spill/read-ahead capacities unchanged | Shrink existing/future spill buffers before correspondence, flushing pending data first | Fixed |

The directory helper uses the existing authenticated batch reader and canonical
codecs; no new cache/interface/backend. Workspace preparation charges 16 KiB per
request for additional wave/association ownership. This is not a standalone total
memory bound: one decoded node, canonical rewrap/validation reencoding and physical
reader scratch retain their existing separate charges. Producer reservation still
fits existing output partitions; the physical encoder's 2-MiB reserve and complete
late-validation reservation are unchanged.

Nine implementation files changed in this review: Store `objects.rs`,
`objects/{admission,read,spill}.rs`, `telemetry.rs`; content
`tree/directory/{read,mod}.rs`; Workspace `changes.rs`; and benchmark
`storage_smoke.rs` for the corrected diagnostic. Other M4 owners were reviewed
without further edits. No schema, canonical codec, CDC or workload change.

## Fresh corrected evidence

All 33 reopened FUSE mappings passed, covering 114,620,274 bytes and the original
path/type/mode/symlink oracles. All four frequent-edit no-change outcomes and every
performance/verification cleanup passed. There are still 269 selected DELTAs:
266 DeepSeek and 3 ordinary binary, all with selected FULL anchors. No unselected
physical records or retained stages were observed. Independent audit rehashed all
new manifests and compared every observed TSV directly to its original oracle.

Each row is one fresh corrected observation compared with historical controls.
These are not matched pairs, not a latency distribution and not final qualification.

| Case | Baseline / M3 / corrected M4 allocation B | Mutation + Commit ms, baseline / M3 / corrected M4 |
| --- | --- | --- |
| fuse-binary-8m | 11,206,656 / 10,354,688 / 10,289,152 | 263.891 / 266.834 / 234.130 |
| fuse-text-32k | 1,179,648 / 983,040 / 983,040 | 40.491 / 49.442 / 47.847 |
| sdk-binary-8m | 11,141,120 / 10,354,688 / 10,354,688 | 28.220 / 39.050 / 34.499 |
| sdk-text-32k | 983,040 / 983,040 / 983,040 | 32.937 / 33.138 / 32.473 |
| small-files | 1,441,792 / 1,114,112 / 2,162,688 | 22.108 / 30.301 / 34.236 |
| deepseek-five | 12,320,768 / 5,898,240 / 4,849,664 | 1095.992 / 1256.791 / 1365.948 |

**Retain bounded emission as a development checkpoint.** DeepSeek still saves
17.78% allocation versus M3 (60.64% versus baseline), with 8.69% additional replay
elapsed versus M3. Ordinary binary saves 64 KiB versus M3; SDK/text allocations
are unchanged. This recommendation is not a claim that every storage/performance
gate passes. Do not widen search or run M5 to force a more favorable result.

### Small-file storage regression retained

Init remains 1,114,112 B. Final allocation is **2,162,688 B**, versus previous M4/M3
1,114,112 B and baseline 1,441,792 B: a valid original storage-gate miss. It is not
replaced by the smaller logical SQLite size. The fresh canonical trajectory matches
the original baseline exactly:
`447/380991 →454/386031 →458/390818 →461/395507` objects/bytes. Previous M3/M4 ended
at 460/391367. This is consistent with the recorded randomized inode/COW population
variation; it is not evidence of incorrect retained contents.

The fresh final pack is 4,387 B versus 339 B previously; total pack BLOB bytes are
112,712 versus108,664 (+4,048 B), with the same 7 packs / 23 groups and no unselected
records. The pack table adds one 65,536-byte page; total SQLite pages rise 17→18.
Logical file size is 1,179,648 B, while filesystem allocation is 2,162,688 B,
983,040 B larger. Together this yields the measured 1,048,576-byte allocation jump.
The exact filesystem allocation mechanism remains unestablished. No repeat for
nicer allocation, truncation, VACUUM or SQLite tuning was performed.

Post-Init growth: DeepSeek 3,866,624 B; ordinary binary 0; SDK binary 65,536 B;
text 0; small files 1,048,576 B. These are separate from total-allocation denominators.

### Diagnostics and resource limits

Original diagnostic misses in this corrected run are SDK binary steps 2/3;
small-file final allocation, first read and steps 1/2/3; DeepSeek steps 3/5.
All raw operands and allowances are in the review JSON. Original M3 and initial
M4 misses remain in their own reports. Small-file Init is 8.037 ms, first/repeated
reads 26.775 / 7.071 ms; no cold-cache claim. Correctness/cleanup PASS does not relabel
these diagnostic misses.

DeepSeek: 292 base trials, 82.176 ms matching and 31.714 ms encoding; 3,719 performance
group fetches, 25,968,180 encoded group bytes, 53,234,935 decoded bytes and 3,705
decompressions. Absent predecessors now count 758 eligible targets while 984 have
no hints; 193 correspondence budget skips remain explicit. The fields no longer
conflate absence with optional discovery exhaustion. Group-byte counters include
record framing, exclude outer pack headers/directories; range counts include them.
Historical M3 lacks these counters, so no physical-read comparison ratio is claimed.

DeepSeek public-phase CPU 438.480 ms versus M3 313.462 ms (+39.88%) is separately
reported from 8.69% replay elapsed growth. Performance host lifetime RSS peaks range
10,747,904–30,130,176 B; container lifetime peaks 5,156,864–14,766,080 B; sampled
spool peaks 4,096–24,576 B. Resource comparisons and absolute caps pass; no swap/OOM.
These are sampled/lifetime scopes, not exact phase peaks. All per-case CPU/I/O,
cgroup categories, wall scopes and historical verification costs remain in JSON.

## Commands, custody and completion

Run directories under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs`:
`m4-review-edits-1`, `m4-review-small-1`, `m4-review-deepseek-1`. Each has separate
performance/verification raw JSONL, results, identities, manifests and cleanup.
One matching host/image build and one performance/verification observation per
smoke ran. No build or smoke failed. Prior successful runs were not overwritten.

- Product correction: `de3a046f6a488ddd9d5f6710db8d98609b9983d6`.
- Combined seal: `240c9f21007f3c3f724ce8c2ac6813b753c89532f738a3d5f9be7d48b8f533cb`.
- Product seal: `00e076f7711284cbc4a0144fc77f44537349f9a5d806fc61cab398015fc04b4b`.
- Host SHA-256: `0133fbe3b04b9164a5732904319b5701dd675bc238c9dd57140a8264939a28ee`.
- Host and sidecar: `builds/m4-review-host`, `builds/m4-review-host.identity.json`.
- Image tag: `layerfs-bench-infra:240c9f21007f3c3f`; exact image ID in review JSON.
- Patch/reporter: `builds/m4-review-source.patch`, `builds/m4-review-report.py`.
- Logs: `builds/m4-review-host-1.log`, `builds/m4-review-image-1.log`, and
  `builds/m4-review-{edits,small,deepseek}-{perf,verify}-1.log`.

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-storage-smoke-image
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke frequent-edits --source-arm candidate --repetition 1 --image layerfs-bench-infra:240c9f21007f3c3f --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-review-edits-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke frequent-edits --source-arm candidate --repetition 1 --image layerfs-bench-infra:240c9f21007f3c3f --storage-verify-run /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-review-edits-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke small-files --source-arm candidate --repetition 1 --image layerfs-bench-infra:240c9f21007f3c3f --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-review-small-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke small-files --source-arm candidate --repetition 1 --image layerfs-bench-infra:240c9f21007f3c3f --storage-verify-run /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-review-small-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-five --source-arm candidate --repetition 1 --image layerfs-bench-infra:240c9f21007f3c3f --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-review-deepseek-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-five --source-arm candidate --repetition 1 --image layerfs-bench-infra:240c9f21007f3c3f --storage-verify-run /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-review-deepseek-1
```

All applicable M4 completion items are checked after source correction and fresh
smokes. No unresolved source finding remains from the three scoped reviews.
Unexercised malformed-record, concurrency, small-budget and exhaustion branches
remain source-reviewed, not empirically qualified. The storage regression above
remains a valid measurement/acceptance limitation, not an unfinished optimization
campaign. No M5 implementation or final three-pair qualification ran.

Remaining M5: checkpoint/trusted-read and reconciliation cleanup/deletion review,
affected checks for later changes and final frozen three-pair qualification.
Stronger codec, SQLite tuning, migration, cloud and durability stay deferred.

---

# Storage architecture v3 — M4 checkpoint

Status: **M4 implemented and smoke-verified; stop before M5**, 2026-09-08.
This is not complete v0.1.4 qualification. Product commit
`1aa40785a7b4a96c78ba04a6af59af221b8f93bb`, branch
`codex/storage-v3-implementation`, existing draft [PR #81](https://github.com/Ephemeral-AI-Lab/layerfs/pull/81).
The exact committed product/harness bytes match the measured dirty-source build.
[Machine-readable M4 evidence](implementation-milestone-4.json) retains all raw
operands, source/binary/image identities, manifests, commands and limitations.

**Recommendation: retain bounded emission.** DeepSeek allocation falls a further
17.78% from M3, with replay elapsed +0.55% against that historical observation.
Ordinary binary saves one 64-KiB allocation unit. Other allocations are flat.
This is a useful workload-dependent increment, not universal savings or a reason
to widen matching, deepen chains, tune SQLite, or change the workloads.

## Completion and ownership

- [x] Preserved source custody, M2/M3 reports, finalized selection policy and prior failures.
- [x] Shared physical-order reads/counters, preserving request order and duplicates.
- [x] Same-inode, complete-build, localized-range and captured/tempfile predecessor handoff.
- [x] Forward metadata cursor, original first spans, four hints and authenticated admitted FULL anchors.
- [x] Bounded COPY/INSERT matching, optional exhaustion and required error propagation.
- [x] Reviewed physical base retention before emission; no object reclamation or locator replacement.
- [x] Fixed-membership A/B selection, selected-only persistence and prospective capacity checks.
- [x] All three approved smokes, 33 retained mappings, no-change and cleanup passed.
- [x] Allocation, elapsed, physical work and resource evidence reported with historical limitations.
- [x] Final product diff reviewed; no unrelated edits, new backend/cache, obsolete recursive extent visitor or M5 work.
- [x] Progress and M4 JSON recorded from actual retained evidence; existing draft branch checkpointed for push.

No implementation or smoke item remains incomplete. Commit/push custody is checked
at task completion; no merge or release qualification is authorized.

Actual files and responsibilities:

- Store `objects.rs` and `objects/spill.rs`: owned hint/span facts, bounded selected
  delivery and private spool frames. `objects/read.rs`: physical ordering,
  authenticated reconstruction, bounded hint-record inspection and read counters.
- Store `objects/pack.rs`: fixed seed matcher and FULL/mixed group encoding.
  `objects/admission.rs`: admitted-base trials, budgets, comparison operands,
  selection and finally admitted representation counters.
- Content `file/rope/{build,edit,read,mod,validate}.rs` and `object/access.rs`:
  original CDC positions, shared forward traversal and removed old visitor.
- Workspace `changes.rs`: batched retained-inode/path task preparation and physical-
  only predecessor handoff. `capture.rs` is unchanged: its existing CapturedFile
  already owns the now span-bearing DeferredObjectStore; no second capture pass.
- Store `layerstack.rs`/`workspace.rs`: pass Store access into shared preparation;
  snapshot correspondence diagnostics. Store `schema.rs`, `store.rs`, `telemetry.rs`,
  `lib.rs`, SDK `client.rs`, and benchmark `storage_smoke.rs`: existing receipt
  integration, Store-local counters and structured diagnostic rendering.
- Documentation: this ledger, M4 JSON and checked completion list in the M4 plan.
  No schema/SQL, canonical encoding, CDC profile, workload or oracle changes.

## Measured outcomes

Every control/candidate has one observation. Baseline and M3 are historical,
not freshly paired controls. New diagnostic rendering changes harness bytes;
fixtures, public operations, output-drain/finalization and timing boundaries are
unchanged. Random Init inode identity/order can affect canonical tree populations
and compressed bytes. All six M4 final canonical counts/bytes match M3; this is not
a claim that their exact IDs or grouping bytes are identical.

| Case | Baseline allocated B | M3 allocated B | M4 allocated B | Baseline reduction | M3 reduction | Mutation + Commit ms: baseline / M3 / M4 |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| fuse-binary-8m | 11,206,656 | 10,354,688 | 10,289,152 | 8.19% | 0.63% | 263.891 / 266.834 / 266.340 |
| fuse-text-32k | 1,179,648 | 983,040 | 983,040 | 16.67% | 0.00% | 40.491 / 49.442 / 48.906 |
| sdk-binary-8m | 11,141,120 | 10,354,688 | 10,354,688 | 7.06% | 0.00% | 28.220 / 39.050 / 34.686 |
| sdk-text-32k | 983,040 | 983,040 | 983,040 | 0.00% | 0.00% | 32.937 / 33.138 / 32.304 |
| small-files | 1,441,792 | 1,114,112 | 1,114,112 | 22.73% | 0.00% | 22.108 / 30.301 / 22.098 |
| deepseek-five | 12,320,768 | 5,898,240 | 4,849,664 | 60.64% | 17.78% | 1095.992 / 1256.791 / 1263.726 |

Allocation is complete SQLite filesystem allocation plus required sidecars at
acknowledgement, before verifier Branches. DeepSeek growth after Empty Init is
3,866,624 B (M3 4,915,200 B); ordinary binary growth is 0 (M3 65,536 B);
SDK binary growth remains 65,536 B; text and small-file growth remain 0.
These growth figures do not replace the primary total-allocation denominator.
DeepSeek pack BLOB bytes fall 3,140,288 →2,757,089 (383,199 B), while filesystem
allocation falls 1,048,576 B. SQLite/page/filesystem allocation granularity makes
those different quantities; no hidden reclamation, truncation, VACUUM or tuning ran.

Small-file Init: 8.765 ms versus M3 8.561 / baseline 7.029; first read 25.014 ms
versus 25.329 /19.658; repeated read 7.675 ms versus 7.666 /7.439. Neither read
is OS-cold. Binary Init is 18.535 ms SDK /19.491 ms ordinary. All public operations
and the full DeepSeek checkpoint/step timings remain in JSON, without pooled SDK
and ordinary-route claims.

Three original diagnostic misses remain: ordinary text step 2 is 15.238 ms versus
14.352-ms allowance; SDK binary return-to-A is 9.841 ms versus 9.403-ms allowance;
small-file first read is 25.014 ms versus 24.658-ms allowance. They are not relabeled
PASS under a new numerical gate. Historical M3 misses remain preserved independently.
All original storage/resource diagnostics hold for this single observation. The
owner's short-total-operation tradeoff and material DeepSeek gain support retaining
this checkpoint; no final three-pair qualification or aggregate-load claim follows.

## Representation, read and work evidence

All 33 reopened FUSE mappings passed independent path/type/mode/symlink/byte oracles,
reading 114,620,274 bytes. Frequent edits retain SDK calls/members 1/1/1/3/0 and
ordinary in-place, full rewrite, tempfile/rename and return-to-A routes. All four
no-change Commits return UpToDate. Cleanup passed in every performance/verification
case; no retained workspace stage or unselected physical record was observed.
The unrelated `layerfs-phase21-binary-build` container was preserved.

| Case | FULL / DELTA records | FULL anchors | RAW / Zstd groups | Pack BLOB B | Trials | Matching ms | Codec calls / ms |
| --- | --- | ---: | --- | ---: | ---: | ---: | --- |
| fuse-binary-8m | 478 / 3 | 3 | 397 / 11 | 8,449,219 | 5 | 2.933 | 411 / 6.680 |
| fuse-text-32k | 43 / 0 | 0 | 0 / 10 | 3,899 | 4 | 4.391 | 12 / 0.256 |
| sdk-binary-8m | 488 / 0 | 0 | 398 / 10 | 8,476,754 | 6 | 2.406 | 408 / 6.466 |
| sdk-text-32k | 46 / 0 | 0 | 0 / 11 | 4,301 | 6 | 2.636 | 11 / 0.283 |
| small-files | 460 / 0 | 0 | 3 / 20 | 108,664 | 0 | 0.000 | 23 / 0.696 |
| deepseek-five | 5290 / 266 | 172 | 5 / 417 | 2,757,089 | 292 | 75.872 | 498 / 30.940 |

Ordinary binary selected two DELTAs on complete rewrite and one on tempfile/rename.
DeepSeek selected 266 DELTAs with 172 distinct FULL anchors. Offline locator census
confirms all 269 selected DELTA dependencies resolve to selected FULL records;
public reopened readback exercises reconstruction/authentication. Verification
records 347 base fetches for DeepSeek and 10 for ordinary binary. This does not
qualify every malformed/deep-chain rejection or dependency-reclamation scenario.

DeepSeek A-alternative encoded sum is 3,129,618 B; eligible B sum is 256,644 B;
selected sum is 2,749,409 B. A and B totals have different group populations;
A minus B is not a storage-saving equation. Two ordinary-text mixed alternatives
were rejected, correctly retaining FULL. No raw per-record 12.5% acceptance gate
remains. Encoding calls include unsuccessful compression trials and both A/B routes.

DeepSeek performance phases fetch 3,572 groups, 24,700,984 encoded group bytes,
51,195,843 decoded bytes and make 3,557 decompressions. Verification separately
fetches 8,347 groups, 57,256,082 encoded and 118,677,689 decoded bytes. Group bytes
include record framing and exclude outer pack header/directory bytes; BLOB-range
counts include their reads. Later waves repeat groups explicitly, without a cache.
Matching charges 1,709,010 comparison bytes and 13,754,848 seed/hash bytes across
DeepSeek's batches; 292 trials take 75.872 ms, encoding 30.940 ms. Required read,
authentication and construction costs remain within public operation timers.

DeepSeek correspondence reports 193 optional budget skips and 66,617,088 reserved
bytes summed across separate Commits, each limited to 16 MiB. This is prospective
worst-case reservation, not actual metadata I/O. No matching/fetch/instruction or
memory-budget skip occurred in this smoke set. The smaller-partition memory-skip
path is source-reviewed, not empirically exercised. `absent_predecessors` currently
counts eligible targets with no prior IDs at encoder entry, including Init,
no-overlap and exhausted correspondence; it is not an exact count of files without
a predecessor. The report preserves that diagnostic limitation explicitly.

Read/matching counters were unavailable in historical M3, so no physical-read
speedup ratio is claimed. Snapshot intervals include shared-Store worker activity;
concurrent overlap would not be exclusive operation attribution. Membership SQL
counts and matching-only base-byte attribution remain unavailable. Metadata targets
stay FULL; current hints originate from payload spans. No global similarity search.

## Resource scopes and evidence custody

DeepSeek public-phase host CPU is 364.495 ms versus M3 313.462 (+16.28%); replay
elapsed is 1,263.726 ms versus1,256.791 (+0.55%). Historical-verification wall is
2.714 s versus2.600 (+4.40%); performance work wall is3.964 s versus4.786, an
unpaired observation including coordinator overhead, not attributed codec speedup.
Performance host lifetime RSS peaks range10,747,904–32,587,776 B, container lifetime
peaks4,870,144–14,966,784 B and sampled spool peaks4,096–24,576 B. All absolute and
comparative resource checks hold; no swap/OOM or incomplete cleanup occurred.
These are lifetime/boundary/sampled scopes, not precise codec or operation peaks.
Complete host CPU/I/O, cgroup categories and spool/staging observations are in JSON.

Artifact root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs`.
Final directories: `m4-checkpoint-edits-1`, `m4-checkpoint-small-1`,
`m4-checkpoint-deepseek-1`; each contains performance and verification manifests,
identity, exact child commands, raw JSONL, results, resource/cleanup records and Stores.
All baseline/M3/M4 final verification manifests were rechecked. Three host builds
were preparation during integration; all succeeded. Only the final build was used
for measurements. One image build and one performance/verification observation per
smoke ran; no failed smoke or rerun was discarded.

- Combined seal: `074900ce608a6fb6c7307a2ee873bdc9f459beacf7d6bd9e9b87f764222b1171`.
- Product seal: `2d430bcbf902e37b0a57d3116bec9c3591da199988a2118b9f2900ce511b9e23`.
- Retained host: `builds/m4-checkpoint-host`; SHA-256
  `d81f79e478cc6ba55a3a545fbc40aa20d0a6a00927d62665359a42be01d0aa98`.
- Image: `layerfs-bench-infra:074900ce608a6fb6`; exact Docker ID in run identities.
- Build logs: `builds/m4-host-{1,2,3}.log`, `builds/m4-image-1.log`.
- Source patch and reporter: `builds/m4-checkpoint-source.patch`,
  `builds/m4-checkpoint-report.py`; exact hashes in M4 JSON.
- Wrapper logs: `builds/m4-{edits,small,deepseek}-{perf,verify}-1.log`.

Build commands from the implementation worktree:

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-storage-smoke-image
```

Exact smoke commands (run serially, each performance followed by verification):

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke frequent-edits --source-arm candidate --repetition 1 --image layerfs-bench-infra:074900ce608a6fb6 --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-checkpoint-edits-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke frequent-edits --source-arm candidate --repetition 1 --image layerfs-bench-infra:074900ce608a6fb6 --storage-verify-run /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-checkpoint-edits-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke small-files --source-arm candidate --repetition 1 --image layerfs-bench-infra:074900ce608a6fb6 --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-checkpoint-small-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke small-files --source-arm candidate --repetition 1 --image layerfs-bench-infra:074900ce608a6fb6 --storage-verify-run /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-checkpoint-small-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-five --source-arm candidate --repetition 1 --image layerfs-bench-infra:074900ce608a6fb6 --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-checkpoint-deepseek-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-five --source-arm candidate --repetition 1 --image layerfs-bench-infra:074900ce608a6fb6 --storage-verify-run /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m4-checkpoint-deepseek-1
```

No unit/fuzz/property/race/crash/full-workspace/full-family/fourth-smoke suite,
new population, stronger codec or M5 final qualification ran. Remaining M5 work:
checkpoint fact/trusted-read cleanup, invocation-local reconciliation cleanup and
its deletion ledger, affected checks for subsequent changes and the frozen final
three-pair qualification. Stronger-codec and SQLite layout opportunities remain
separately deferred; this checkpoint does not silently authorize them.

## M4 pre-measurement ownership review (2026-09-08)

The writer uses common FULL-sized group membership, level-1 encoding of A and
at most one mixed B, independent 16-byte codec savings, and the finalized
`max(64, ceil(A/8))` encoded-group threshold. Matcher work is bounded by four
hints/trials per target, a 64-KiB fixed seed table, 512 batch trials and 16 MiB
seed/hash/comparison work. Missing untrusted hints are optional; fetched malformed
records and invalid selected FULL bases fail. Only admission winners contribute
FULL/DELTA selected counters. No same-batch anchor or extra durable FULL copy exists.

The existing output allowance remains inclusive: at most 6 MiB resident canonical/
prepared ownership and 2 MiB physical scratch inside 8 MiB. Existing upstream
slabs, bounded indexes and spool I/O keep their distinct charges. The shared
consumer normally closes missing canonical batches at 512 KiB (plus an incoming
256-KiB page); larger objects flush separately. Closed-episode late validation
still reserves the complete worst selected DELTA-program sum, up to 1 MiB,
inside physical scratch. No late split/recheck or weaker validation is introduced.

For one group, source-derived physical preflight charges actual retained current-
pack encoded Vec capacities and association capacities, 1 MiB codec context, RAW
and compressBound buffers (each conservatively 66,560 bytes), and, for B, another
66,560 bytes each for retained A and aggregate delta programs. The input vector
and preallocated prepared-object associations are charged; unused initial ID sets
are dropped before preparation. Base/table/current/best matching owners drop before
codec calls. Programs have capacity at most their FULL records, summing to at most
one FULL-sized group. Prepared backing outside this current pack is in the resident
allowance; assembly runs after codec contexts drop. Optional B is skipped before
search if its prospective capacity cannot fit; required encoding failure propagates.

Producer correspondence can overlap admission and other producers. Each capable
producer reserves 576 KiB (64 KiB forward cursor/frontier plus 512 KiB bounded
metadata decode/reconstruction) from its existing partitioned output allowance,
spilling existing canonical output if needed. Partitions with less than this plus
32 KiB output skip optional correspondence, preserve predecessor context and spans,
and report the memory reason. Worker counts and workloads are unchanged. This
conservative policy can significantly reduce opportunities in multi-file parallel
construction; it is a limit to report, not a claim that all potential bases were
searched. Correspondence also prospectively reserves 131,136 bytes per metadata
fetch against 1 MiB/file and 16 MiB/operation and caps descriptors at 4,096/file.
Reserved work is distinct from actual read counters and survives admission flushes.

Production admission locators/packs remain immutable and no production object
reclamation path exists. Stage discard deletes only its stage. Failed/fallback
construction retains admitted objects. Thus admitted FULL anchors remain readable
without a direct logical reference. Physical closure follows each DELTA base
ObjectId through its selected locator to its FULL record/group/pack; no new
persistent dependency index is needed. Export and garbage collection remain deferred.

Read ordering is shared by ordinary reads and admission comparisons. Store-local
atomic diagnostics include worker and mounted-host read activity; counter intervals
are shared-Store intervals and may overlap under concurrency. They do not establish
aggregate-load attribution or phase-exact memory peaks. New diagnostic rendering
changes harness bytes but no fixture, public call, oracle or timing boundary.
Baseline/M3 comparisons will be descriptive historical controls.

---

# Storage architecture v3 implementation progress

Status: **M3 complete and closed as a development milestone; follow-ups tracked below**. Updated 2026-09-08.
The complete v0.1.4 storage design is **not qualified**. M4/M5 remain deferred.
No merge, migration, cloud work, crash qualification or benchmark campaign ran.

M3 product commit: `1ac1ce4b56064a548929b070568d2373daa9e30d` on
`codex/storage-v3-implementation`, existing draft [PR #81](https://github.com/Ephemeral-AI-Lab/layerfs/pull/81).
Worktree `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3` was clean at recovered
M2 HEAD `0e18ec561d11934f2935e20832bf49eab1afab72`. All M2 evidence is preserved;
the complete prior ledger appears verbatim below as historical context.

## M3 result and completion checklist

Compression produces a material storage improvement at acceptable foreground cost
under the owner's prospective policy. DeepSeek allocation is 52.13% below the
unchanged baseline and 47.37% below M2; replay elapsed is +14.67% / +12.89%.
Small-file Init saves 22.73% with equal canonical counts/bytes; final allocation
saves 26.09% versus M2 with the COW-population caveat below. Incompressible binary
histories show little or no incremental allocated-space saving; this is not a
universal storage benefit claim. The owner reiterated after these observations
that storage optimization is the priority and timing margins are generous; no
historical result or frozen gate was relabeled because of that later message.

- [x] Source custody, current branch/HEAD, instructions, idle prior tasks and M2 evidence verified.
- [x] Prospective M3–M5 performance policy recorded before candidate measurements (`d941f86bc`).
- [x] Bounded Zstandard FULL groups and framing-aware RAW selection implemented in shared writer.
- [x] Bounded range extraction/decompression with ordinary canonical authentication implemented.
- [x] Shared Init/Commit lifecycle, localized COW, bulk SQL, permits and connection boundaries preserved.
- [x] All three approved smokes, 33 historical mappings, no-change and cleanup checks passed.
- [x] Source and result review found no unresolved M3 defect requiring a fix/rerun; no speculative tuning.
- [x] Allocation, elapsed, physical/resource accounting, historical controls and limitations recorded separately.
- [x] Separate M3 JSON evidence added; original M2 JSON and raw evidence unchanged.
- [x] Final product diff reviewed: obsolete unsupported-codec branch removed, no duplicate owner or M4/M5 work.
- [x] Product and evidence committed/pushed to existing implementation branch/PR; PR remains unmerged.

No M3 checklist item remains incomplete. This is a development milestone, not
final three-pair, aggregate-load or complete v0.1.4 qualification.

## Actual module ownership

- `objects/pack.rs` owns existing wire framing plus Zstandard trial/selection and
  bounded decode. Its private FFI module uses the bundled pinned Zstandard 1.5.7
  static-context APIs. `lib.rs` denies unsafe globally with a narrow allowance
  inside that module; documented alignment/lifetime rules bound this native call.
- `objects/admission.rs` retains canonical comparison operands by moving vectors
  for compressed records, and borrows RAW operands from immutable prepared packs.
  Its existing permit, single final recheck and bounded multirow INSERT owner stay
  intact; zero-winner/mixed-pack rules and synchronous partial flushes are unchanged.
- `objects/read.rs` extracts header/selected directory/group ranges under the
  connection guard, then delegates decode outside it. Existing target/base waves
  coalesce requested groups and canonical authentication remains in the same owner.
- Store `Cargo.toml`/workspace `Cargo.lock` pin `zstd-sys =2.0.16` (bundled 1.5.7),
  defaults disabled, experimental static APIs enabled. New transitive build
  dependencies are recorded; no external dependency source was modified.
- Canonical constructors, CDC/COW trees, public SnapshotReader cache, schema/SQL,
  Init discovery, staging, head checks and finalization code are unchanged in M3.
  No physical DELTA emission, shadow machinery, extra cache, scheduler or backend.

## Owner update for M3–M5 (prospective, 2026-09-08)

The owner authorizes M3 bounded group compression now, continuing from completed
M2 HEAD `0e18ec561d11934f2935e20832bf49eab1afab72` on the existing implementation
branch/PR #81, and requires this task to stop after M3. M4/M5 remain later work.
The existing schema-6/wire-1 new-Store-only scope is unchanged.

Before new M3 candidate measurements, record the owner's updated tradeoff policy:
substantial storage reduction takes priority with reasonable foreground cost.
Approximately sub-50 ms **total public-operation elapsed** can make a percentage
increase immaterial; this is not 50 ms extra per object lookup. For longer
foreground operations, approximately 30% added elapsed is acceptable for meaningful
storage improvement (1.0 s to 1.3 s is explicitly acceptable). These are tradeoff
criteria, not an allowance to spend on negligible savings. Repeated reads, total
workload time, CPU/resources and queueing still matter. Results outside these
examples require reporting the actual tradeoff and investigating concrete causes,
not inventing a passing threshold. Smoke evidence establishes no aggregate-load
qualification. Historical gates, observations and failures remain unchanged;
this policy changes no storage target and does not retrospectively pass old misses.

M3 follows the frozen one-observation-per-coherent-slice development procedure,
using all three approved smokes because shared Commit/read behavior is affected.
The M5 final three-pair qualification will not run. Baseline and M2 comparisons
are descriptive historical comparisons, not new matched timing controls.

### M3 custody and implementation decisions before measurements

The implementation task found the clean expected branch at M2 HEAD, with PR #81
open/draft at that same revision. Earlier implementation tasks and the later
read-only design-review task were idle. No active smoke/build process or measurement
lock owner was found. The unrelated `layerfs-phase21-binary-build` sleeping
container is preserved. Baseline/M2 final verification manifests and retained M2
host binary hashes were checked; frozen fixture/cache/resource definitions agree.

Keep existing 16/32-KiB role targets, 64-KiB ordinary decoded limit, 256-KiB decoded
pack cap, count/admission bounds and oversized FULL/RAW route unchanged. Encode
eligible closed groups once with level 1, checksum, content size and <=64-KiB
window. The complete Zstandard frame must save at least 16 bytes versus decoded
RAW framing; the pack directory has the same size for either codec. Compression
errors/resource failures propagate; RAW is only the explicit size/savings choice.

Use pinned Zstandard native static contexts to enforce actual allocator bounds:
compression workspace <=1 MiB and decoder workspace <=256 KiB, inside the existing
2-MiB physical reservation. A private byte-codec FFI boundary is necessary because
the safe binding lacks static-context APIs; deny unsafe elsewhere and document
pointer/alignment/lifetime invariants at that boundary. No codec cache, worker or
new memory-management subsystem. Pinned `compressBound(65536)` is 65,824 bytes (defensive trial cap 66,560).
Selected compressed Vecs retain trial capacity until assembly; truncation is not
reported as memory release. For a normal pack, the conservative group-capacity
sum is <=279,552 bytes. Shared admission still caps ordinary canonical batches at
512 KiB, plus <=256 KiB incoming; >256-KiB objects flush alone through RAW spool.
Existing source consumption moves canonical vectors out of the <=6-MiB resident
owner. Encoding's <=1-MiB context, <=~512-KiB accumulated encoded backing, one
<=64-KiB raw group and <=65-KiB trial leave room for bounded directories/associations
inside the 2-MiB reservation. Assembly happens after the context drops. Decoder
workspace plus encoded/decoded ordinary groups is <=384 KiB; the existing 1-MiB
late-validation reserve remains unchanged. These are conservative source-derived
bounds, not observed process peaks. Existing final-owned diagnostics do not include
codec/pack scratch and must not be described as measuring inclusive memory. Compressed records move their authenticated canonical
operands into prepared admission for late equality; RAW records keep borrowing
from their prepared pack. Encoding remains outside admission and connection locks.

## Integrated smoke evidence and comparisons

[Machine-readable M3 evidence](implementation-milestone-3.json) contains exact raw
operands, per-step routes/resources, manifests, group census and original diagnostic
checks. Artifact root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs`.
Every case has one observation per arm. Historical baseline/M2 timings are not
matched new controls; baseline predates untimed Init receipt rendering. M2 and M3
harness files match exactly. Frozen fixtures, interfaces, cache/resource profiles
and all final control/candidate manifests were checked. No OS cache purge or
completed Store reuse occurred. All final primary allocation receipts predate
verifier-created Branches and include required SQLite sidecars.

| Case | Baseline allocation B | M2 allocation B | M3 allocation B | Reduction baseline / M2 | Mutation + Commit ms: baseline / M2 / M3 |
| --- | ---: | ---: | ---: | --- | --- |
| DeepSeek first five | 12,320,768 | 11,206,656 | 5,898,240 | 52.13% / 47.37% | 1095.992 / 1113.313 / 1256.791 |
| SDK text-32k | 983,040 | 983,040 | 983,040 | 0% / 0% | 32.937 / 28.004 / 33.138 |
| SDK binary-8m | 11,141,120 | 10,354,688 | 10,354,688 | 7.06% / 0% | 28.220 / 35.820 / 39.050 |
| Ordinary text-32k | 1,179,648 | 1,179,648 | 983,040 | 16.67% / 16.67% | 40.491 / 45.286 / 49.442 |
| Ordinary binary-8m | 11,206,656 | 10,420,224 | 10,354,688 | 7.60% / 0.63% | 263.891 / 260.997 / 266.834 |
| Small files | 1,441,792 | 1,507,328 | 1,114,112 | 22.73% / 26.09% | 22.108 / 26.283 / 30.301 |

These sums contain the declared individual public mutation and Commit timers,
including required output drain/finalization; they are not whole smoke wall time.
Small-file after-Init allocation is 1,114,112 versus 1,441,792 in both controls.
Init elapsed is 8.561 ms versus 7.833 M2 / 7.029 baseline. First read is 25.329 ms
versus 23.719 / 19.658; repeated read is 7.666 ms versus 7.419 / 7.439. Neither
pass is described as OS-cold. Binary Init takes 20.234 ms SDK /18.903 ms ordinary
versus M2 14.174 /14.141; those percentage increases remain short total operations.

All changed frequent-edit Commits still use one object-admission transaction;
separate stage/publication transactions remain. DeepSeek admissions are unchanged
at 11/9/10/15/12. SDK member/call counts remain 1/1/1/3/0 and 1/1/1/1/0, with zero
SDK-caused FUSE writes. Ordinary in-place, truncate/rewrite and tempfile/rename
routes remain intact. Each unchanged edit Commit returns UpToDate.

| Artifacts | Historical mappings | Performance / verification / cleanup |
| --- | ---: | --- |
| `m3-checkpoint-small-1` | 4/4 | PASS / PASS / PASS |
| `m3-checkpoint-edits-1` | 24/24 | PASS / PASS / PASS |
| `m3-checkpoint-deepseek-1` | 5/5 | PASS / PASS / PASS |

Mounted reopened readers checked 114,620,274 bytes across the 33 mappings, plus
path/type/mode/symlink oracle comparisons. Every owned container was removed and
no workspace stage remained. No candidate failure or rerun occurred; the first
matching build and integrated smoke set succeeded. Source review and the below
read-only accounting are not additional product verification suites.

Original tighter diagnostic misses remain explicit: SDK binary steps 2/3/4,
small-file first read and small-file steps 2/3. All original storage/resource
checks hold in this single observation; final DELTA coverage is deferred to M4,
not passed. Under the prospective policy, short public calls and substantial
DeepSeek/small-file saving justify the actual cost. No attempt was made to shave
insignificant milliseconds or change the old diagnostic results into passes.

### Physical bytes, resource costs and attribution

| Case | Packs | RAW / Zstd groups | FULL records | Pack BLOB bytes | Encoded group savings vs their RAW framing |
| --- | ---: | --- | ---: | ---: | ---: |
| DeepSeek | 58 | 3 / 421 | 5,556 | 3,140,288 | 5,876,859 |
| SDK text | 5 | 0 / 11 | 46 | 4,305 | 58,558 |
| SDK binary | 61 | 398 / 10 | 488 | 8,476,890 | 21,302 |
| Ordinary text | 5 | 0 / 10 | 43 | 3,899 | 103,272 |
| Ordinary binary | 61 | 397 / 11 | 481 | 8,498,480 | 22,742 |
| Small files | 7 | 3 / 20 | 460 | 108,714 | 285,525 |

This offline census is after verification, outside all operation timers. It
counts all pack BLOBs, including partial packs; all records are FULL and no
unselected record was observed. There is no concurrent-race/zero-waste guarantee.
Encoded totals equal outer framing plus encoded groups. Decoded totals separately
equal canonical bytes plus record directories/kinds. The JSON retains both,
SQLite B-tree/index pages and unused allocation; compressed payload is never the
primary allocation numerator. No DELETE, VACUUM, repack or later packing occurred.
No per-record share of a compressed group is invented for hypothetical mixed waste.

Small-file canonical population is seed-dependent: Init always has 447 objects /
380,991 bytes; M3 final has 460/391,367 versus M2 461/396,275. Random Init-derived
inode ordering places f032/f040 in separate COW leaves in this M3 run; restoring
f032 reuses an additional leaf. The final 4,908 canonical-byte difference is not
attributed to compression. Equal-population Init already saves 22.73% allocation.
The bounded investigation is retained in `builds/m3-small-canonical-diagnostic.md`.
DeepSeek final canonical bytes differ by 108 and one record versus M2; the large
5,876,859-byte framed-group saving establishes the dominant compression effect
without adding overlapping CAS/COW percentages. All edit-history canonical totals
match M2 exactly.

Observed host lifetime RSS peaks are 10,420,224–31,375,360 bytes, container peaks
4,874,240–15,159,296 bytes, sampled spool peaks 4,096–24,576 bytes. All comparative
and absolute resource budgets hold; no swap/OOM occurred. These are lifetime or
sampled scopes, not precise phase maxima. Required source/output spools and owned
staging observations are retained; unseen instantaneous peaks are not zero.

DeepSeek host public-phase CPU rises 231.786 ->313.462 ms (+35.24%, +81.676 ms)
versus M2; replay elapsed rises 12.89%, total performance work wall 3.901 ->4.786 s
(+22.70%), and historical-verification work 2.554 ->2.600 s. All raw host CPU/I/O,
cgroup categories, staging/disk observations and other case costs remain in JSON.
SDK-text work wall rises 1.620 ->2.179 s (+34.52%) while mutation+Commit rises only
28.004 ->33.138 ms: about 554 ms of the difference lies outside those product
timers. The unchanged coordinator includes resource sampling, receipt processing
and Docker inspection there; individual overhead attribution is unavailable.
This is disclosed rather than called a codec regression, hidden in an average,
or judged against an invented wall threshold. No duplicate product work or added
per-object crossings were identified in M3 source review. Historical observations
do not isolate machine scheduling/noise or establish aggregate throughput.

Candidate membership-query counts and physical BLOB/decode call/byte counts remain
unavailable (logical SnapshotReader counters are not physical counts). Source review
establishes selected-range extraction and one decode per distinct group in a bounded
wave; later drains/dependent tree levels can repeat decoding. No new persistent
cache or second canonical verification pass was added. Static buffer bounds above
are source-derived, separately from observed RSS.

## Exact build and command custody

- Product commit: `1ac1ce4b56064a548929b070568d2373daa9e30d`.
- Prospective policy commit: `d941f86bcabe90da5f2d86ae29b2109257bd7a5f`.
- Combined source seal: `e5731ca3a5aca6dd44f00465814982e8bfc89a0780a4d1f2abda37b93dc53f99`.
- Product seal: `7ee60a1f0a7070d51cdd964fe52dcbe4324a99ac36d191020c50b5ef141efc7b`.
- Host SHA-256: `20064b510c637c39997234c65c8cf27bee2c98f36d75c42d0aca92d421db17b7`.
- Runtime image: `sha256:e43a1c6ba062b4811049d1252c39fa94f684619977640c9cec31a1db421d4dc3`.
- Image tag: `layerfs-bench-infra:e5731ca3a5aca6dd`.
- Retained host/sidecar: `builds/m3-checkpoint-host`, `builds/m3-checkpoint-host.identity.json`.
- Build logs: `builds/m3-host-1.log`, `builds/m3-image-1.log`.
- Built at policy commit plus exact product patch, now committed with matching seals.
  Patch `builds/m3-checkpoint-source.patch` SHA-256:
  `4a8d469d39fb261b378fc2b341e95f8e8f7ae66b0cfaecaee595ba6ee3847ade`.
- External reproducible reporter: `builds/m3-checkpoint-report.py`; exact hash and
  source-document/run hashes are in M3 JSON. Report-generation document hashes
  identify the revision read, not a claim that future ledger edits retain that hash.

From the implementation worktree, builds ran serially under the existing lock:

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-storage-smoke-image
```

For each exact pair `small-files/small`, `frequent-edits/edits`,
`deepseek-five/deepseek`, performance then verification ran with this command shape:

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke CASE --source-arm candidate --repetition 1 --image layerfs-bench-infra:e5731ca3a5aca6dd --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m3-checkpoint-TAG-1
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke CASE --source-arm candidate --repetition 1 --image layerfs-bench-infra:e5731ca3a5aca6dd --storage-verify-run /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/m3-checkpoint-TAG-1
```

Exact expanded commands and stream receipts remain in each run identity/results;
wrapper logs are `builds/m3-{small,edits,deepseek}-{perf,verify}-1.log`.
The image builder disabled its generic self-check as required. No unit, property,
fuzz, race, crash, full-workspace, fourth smoke, full benchmark, Git arm, 157-state
replay or M5 three-pair qualification ran.

## Post-M3 review findings and follow-up ledger

Recorded 2026-09-08 after three independent subagent reviews of
`c16855e0ef656704ed171a922660440fe0cee9aa`: codec/grouping, admission/pack formation,
and existing allocation evidence. The review used source, retained receipts/census
and read-only file metadata. No product changes, builds, smoke reruns, recompression
trials or new candidate measurements ran. No demonstrated M3 correctness defect,
skipped eligible compression, or large readily recoverable storage loss was found.

**Disposition: close M3, retain these findings, and recommend M4 next.** Closing
M3 does not merge PR #81, qualify the complete design, or authorize execution of
M4/M5 or these experiments in this documentation update. The owner prioritizes
storage savings with generous latency margins; that makes further compression
worth evaluating, but does not establish its unmeasured benefit.

| ID / priority | Finding and established evidence | Follow-up owner / timing | Required disposition or evidence |
| --- | --- | --- | --- |
| M3-R1 / first experiment | Writer uses fixed Zstandard level 1. DeepSeek has 3,121,104 encoded Zstandard bytes; 421/424 groups already compress. A modestly higher fixed level, such as 3, may reduce already-compressed content. No alternate level has been measured. | `objects/pack.rs`; consider after M4 establishes the FULL/DELTA mix, before final M5 qualification. | Prospectively record one candidate level; preserve one trial/group, frame/window/output limits, 1-MiB encoder cap, 2-MiB scratch and 8-MiB inclusive allowance. Use only affected approved smokes; report complete allocation and CPU/read/foreground costs, retain failures. Keep only for worthwhile measured savings, or explicitly record why deferred. Do not silently raise memory limits. |
| M3-R2 / concrete, low priority | `prepare_full` charges 20 bytes of possible group framing per record, although actual grouping needs that charge per group. This can close a pack early; the excess is a capacity estimate, not physically written padding. DeepSeek outer framing totals only 7,712 bytes. | `objects/admission.rs`; revisit when pack formation is next touched or evidence attributes material loss to these boundaries. | Maintain exact projected role-group counts/sizes incrementally in the existing owner. Preserve all caps, a forward pass and one encode per closed group. Demonstrate actual pack/page reduction; no storage saving is established by the conservative-charge arithmetic alone. |
| M3-R3 / layout investigation | Retained `dbstat` reports unused pack-table space: DeepSeek 724,874 bytes, SDK binary 958,335, ordinary binary 936,745, small files 87,788. | Existing pack/admission and SQLite layout owners; separate evidence-led investigation. | Attribute waste to concrete placement/overflow behavior before changing it. Larger/fewer packs can worsen SQLite overflow allocation. Unused bytes are not a reclaimable-byte promise; preserve synchronous operation-end flushes and count all allocation. |
| M3-R4 / requires design revision | Per-record worst-case late-validation reserve can constrain pack filling, but the closed-episode sum reservation is required by the approved admission protocol. Generic reader-wave draining does not override it. | Admission protocol/specification first, then `objects.rs`, `objects/admission.rs`, `objects/read.rs`; not a current-contract M3 fix. | Any decoupling needs a prospective ownership/synchronization revision preserving 2-MiB/8-MiB bounds, one final recheck, complete DELTA-reader safeguards, and no unplanned late split/retry. No compliant reserve-removal route was demonstrated; FULL-only smoke coverage is not grounds to weaken safeguards. |
| M3-R5 / separate layout work | Both text Stores occupy 15 x 64-KiB pages despite only 3,899/4,305 pack BLOB bytes. Required table/index root pages dominate. | Schema/page-layout design, beyond M3. | Smaller pages or consolidation need separately reviewed compatibility, constraints, index and I/O tradeoffs. More compression cannot eliminate a required root page under the present layout. Do not create a small-file backend or drop required indexes to improve a smoke. |
| M3-R6 / allocation question | DeepSeek logical file size is 5,111,808 bytes (78 pages), versus 5,898,240 filesystem-allocated bytes. The extra 786,432 bytes are outside logical SQLite length, not freelist/B-tree slack; retained receipts and read-only stat agree. | Existing SQLite/filesystem allocation boundary; bounded diagnosis before any proposed change. | Establish the actual allocation mechanism and whether it can safely be avoided. Preallocation is only a plausible explanation. Keep charging these bytes; no post-operation truncation/compaction or reduced accounting numerator is authorized. |

These are **open follow-ups, not incomplete M3 checklist items**. R1 should receive
an explicit measured or deferred disposition before final qualification; R2–R6
remain evidence-dependent and are not silently added as mandatory M4/M5 scope.
Do not hold M3 open for indefinite tuning or claim a large opportunity without
new attributable evidence. A subsequently demonstrated correctness/bounds defect
or material avoidable cost would justify reopening the relevant owner.

Source anchors: [codec parameters](../../../../crates/layerfs-layerstack-store/src/objects/pack.rs),
[pack accounting](../../../../crates/layerfs-layerstack-store/src/objects/admission.rs),
[closed-episode reservation](storage-architecture-spec.md#batched-duplicate-validation-and-memory-reservation),
[group/read batching](storage-architecture-spec.md#groupread-batching-and-single-pass-byte-work),
[format policy](sqlite-storage-format.md#4-groups-records-and-codec), and
[retained M3 evidence](implementation-milestone-3.json).

Low-value alternatives are explicitly deprioritized: reducing the 16-byte keep
threshold could recover at most 45 bytes across DeepSeek's existing RAW groups
with the same codec output; all six censuses have zero unselected records,
freelist pages and workspace stages. No cleanup windfall or missed RAW-compression
population is established. Small-file final COW-population variation and historical
timing comparability caveats remain as recorded above; no historical evidence is
rewritten by this review.

## Finalized M4 plan update (2026-09-08; documentation only)

The owner finalized [M4 scope and handoff](implementation-milestone-4-plan.md).
M3 remains closed and all preceding observations, JSON and review findings are
preserved. No M4 code, trials or smoke runs were performed for this update.

M4 includes physical read ordering before bounded drains, focused counters,
predecessor/first-span handoff, depth-one matching and two encoded group alternatives.
The prospective selection threshold is max(64 bytes, ceil(FULL alternative encoded
size / 8)); only the winner is stored. Keep level 1 and current memory/schema/wire
limits. Stop at M4 with evidence and retain/revise/remove disposition.

This update supersedes earlier suggestions to place stronger compression or SQLite
layout work ahead of/in M4: stronger-level R1 remains a separate later decision;
SQLite tuning R3/R5/R6 is deferred by the owner. R2/R4 are not automatic M4 scope.
FileState deletion, construction-reuse redesign, new caches and cloud implementation
are also deferred. Future SQLite/S3 hybrid is #82. Historical review text above
records its original proposals, not current implementation instructions.

## Remaining scope

M4 owns predecessor/first-span handoff, forward correspondence, shallow physical
delta selection/emission and anchor policy. M5 owns remaining checkpoint facts,
reconciliation/cleanup work and final integrated three-pair qualification. No
such implementation was started. Arbitrary malformed records, oversized singleton
matrices, selected DELTA paths, late duplicate races, FIFO/failure schedules,
nonempty-Init fallback errors, staging/head races and failure injection are not
covered by these smokes. Safeguards are implemented/source-reviewed where M3
requires them; broader empirical coverage is not claimed. Compatibility remains
schema 6/wire 1 for explicitly new Stores, legacy rejection with no conversion.

---

## Archived M2 ledger (verbatim; its stop instruction applied to that prior task)

# Storage architecture v3 implementation progress

Status: **milestones 0–2 complete as a FULL/RAW development checkpoint; stop here**.
Updated 2026-09-08. This is **not final storage-v3 qualification**. Three integrated
smokes passed public-operation completion, historical mounted readback and cleanup.
Three comparisons exceed the original smoke elapsed allowances; the owner accepts
the checkpoint under the subsequently stated timing tolerance below. Compression,
delta emission and milestones 3–5 are deferred; their original targets are unchanged.

The latest owner scope explicitly supersedes the older full-completion instruction:
finish milestone 2, commit/push to existing PR #81, then stop. No merge, deployment,
conversion, automatic/in-place migration or user-data operation is authorized.
The [approved smoke contract](implementation-smoke-contract-v1.md),
[implementation plan](implementation-plan.md), [architecture](storage-architecture-spec.md),
[format](sqlite-storage-format.md) and [review disposition](review-disposition.md)
retain their original definitions. Their final-v3 compressed-storage and representation
requirements are deferred/not applicable as **milestone-2 exit gates**, not passed,
weakened or replaced. The three-pair final qualification has not run.

Subsequent owner timing disposition: the 1.58% replay increase is acceptable;
**up to 10 ms added elapsed time or less than 30% degradation** is acceptable for
this milestone-2 checkpoint. All reported elapsed comparisons meet that tolerance.
This records owner acceptance separately from the unchanged frozen smoke contract
and its original diagnostic misses; it is not final-v3 qualification.

## Recovery and source custody

- Existing worktree preserved: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3`,
  branch `codex/storage-v3-implementation`, draft [PR #81](https://github.com/Ephemeral-AI-Lab/layerfs/pull/81).
- Recovered HEAD was `ad80cf43157ded12c140caadae53f21980f74599`. The complete diff
  contained only the five reported Store files: layerstack.rs, objects.rs,
  objects/admission.rs, objects/read.rs and workspace.rs. All changes were retained.
- The old task `01a07de3-57bf-7b31-abc2-93e4358f98f6` was idle with its last turn
  interrupted. It was not resumed. No concurrent build/smoke writer was found.
  No second agent was used. The original `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`
  checkout and its unrelated work were untouched.
- The shared lock at
  `/var/folders/s4/xpkmz7wn6yq97w1ls_4f_dfc0000gn/T/layerfs-infra-measurement.lock`
  was free; the existing runner acquired it for every image build, performance
  and verification invocation. No jobs overlapped. The unrelated running
  `layerfs-phase21-binary-build` container contained only `sleep infinity` and
  was left alone, as were unrelated stopped containers.
- `builds/m2-host-11.log` was reconciled against the current source seal, host
  sidecar and actual binary hash. The binary matched; its matching image was
  absent. Only that image was built, using `--build-storage-smoke-image` with
  build self-check disabled. No extra test suite ran.
- The preserved correction is committed as
  **`7cad3f603abf08b5e5021c6545854286a473f1b5`**. The smoke binaries were built at
  `ad80cf4` plus this exact dirty patch; the committed product/harness bytes have
  the same seals. Documentation-only reporting does not change these seals.
- Exact recovery patch: `builds/m2-checkpoint-source.patch`, SHA-256
  `15b13f2379c48728545edbd61812ed8a6d0c41f681f5cdbaf2e931f0b06b1641`.
  All paths beginning with `builds/` or `m2-` below are under the external evidence
  root `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs`.

Applicable `benchmark/AGENTS.md`, the controlling documents, shared admission,
packed framing/readers, spill membership and selected-output delivery, Init sinks
and fallback handoff, direct/Workspace publishers, staging and schema gate were
reviewed against their actual callers. The prior milestone source-review history
remains available in the ledger at `ad80cf4`. Ponytail and systematic-debugging
skills were used; the owner's three-smoke-only scope overrides generic advice to
add unit/property/fault tests. No external library/dependency source was changed.

## Resolved authority and completed milestones

The owner approved the narrow v0.1.4 schema exception: explicit **new-Store-only
schema 6 / wire 1 creation**, exact legacy rejection in this implementation,
existing compatible tools for old Stores, and no converter. This decision and the
smoke fixtures/numerical contract are resolved and must not be requested again.

| Milestone | Actual disposition |
| --- | --- |
| 0 | Complete: committed approved contract, three smoke entrypoints, protected fixtures, unchanged-product baseline and all 33 historical mappings |
| 1 | Complete: paged seen/offset operations, buffered/fallible ID-order seals, selected output ownership transfer and retained spill authentication |
| 2 | Complete FULL/RAW vertical slice and integrated three-smoke checkpoint; elapsed deviations and evidence limits below remain explicit |
| 3 | Deferred: Zstandard encoder/decoder, codec allocation/framing qualification and compressed groups |
| 4 | Deferred: whole-file predecessor/first-span handoff, forward extent cursor, shallow delta selection and emission |
| 5 | Deferred: remaining checkpoint/reconciliation cleanup, final module deletion review and final three-pair qualification |

Milestone 2 includes:

- Schema-6 pack BLOBs and selected canonical-length/locator rows; canonical IDs,
  current CDC, CAS and localized COW semantics remain independent of placement.
- One shared prepared admission owner for Init, direct candidate and Workspace
  candidate publication. Initial exact reuse is filtered **before** physical
  accumulation. Only initially missing objects consume pack/admission capacity.
  The private `MissingBatch` handoff carries that partition into final preparation.
- Child-first output and carried available-dependency facts avoid repeated
  dependency probes. Unknown external dependencies use bounded presence SQL;
  demanded stored bytes still authenticate. Unique candidates receive one initial
  membership lookup and at most one final recheck. Supplied duplicate occurrences
  still pay actual byte equality work; this is not a zero-I/O duplicate claim.
- Bounded bulk pack/locator INSERTs under one admission permit, one forward winner
  selection, no UPSERT/retry/shrinking-set loop, and immutable selected locations.
  Zero-winner packs are skipped; mixed-pack unselected records are retained.
- Native serial/parallel/fallback and Empty Init deliver to the same physical
  sink. Production parent payload collection and all whole-Store Init cleanup
  calls are removed. Fallback retains earlier admissions and reports attempted
  source work; it does not delete another writer's objects.
- Targeted FIFO successor channels replace normal broadcast wakeups. Construction,
  source waits and new encoding stay outside the permit. Late equality validation
  retains the permit but releases SQLite before decode/hash/base access.
- Workspace admission precedes its separately committed stage and conditional
  publication. Existing head/base checks, no-change outcomes, required checkpoint
  installation, resume and cleanup remain in the public operation.
- RAW readers validate the selected directory entry once to choose the oversized
  route, including the 65,528–65,536-byte FULL/DELTA overlap. Canonical reads hash
  after releasing SQLite. Existing DELTA reader code is unexercised here; **no
  compression or delta emission is enabled**. Zstandard reads reject explicitly.

## Exact integrated checkpoint artifacts

[Machine-readable observations and comparisons](implementation-milestone-2.json)
contain integer timings/allocation, per-step admissions/routes, resource operands,
physical census, source identities and receipt-manifest hashes. The final
verification manifests were checked against all retained files. The earlier
performance manifests preserve pre-verifier Store custody; verification adds
Branches, so their Store hashes are not substituted for final Store hashes.

| Identity | Exact value |
| --- | --- |
| Implementation correction | `7cad3f603abf08b5e5021c6545854286a473f1b5` |
| Combined source seal | `7f63b4bd3f355c90c7bfaa266c9b956ab8ea8ec9ba83f54b0faa0c6ac1e80938` |
| Product seal | `16ecf5880990894eaf15599bedcb6eea6b59b59fbee3c036df1bff85e06a6d74` |
| Host binary SHA-256 | `342510f7ca63e609f11c71a7fd4fcce50b6b6b7f6b29a26e06900f027da290b2` |
| Runtime image | `sha256:c51d2431b2b29898fa431e1ab843362e482c2b4d96b85da7410df560032ce574` |
| Image tag | `layerfs-bench-infra:7f63b4bd3f355c90` |
| Schema-6 DDL SHA-256 | `53bda8792a601683038af508b183986e1f880a3c0f2158dc3cc9f6bbf1fced50` |
| Independent harness seal | `f51a4906e4176a51e74c5af6272d3cd0dc11aa681fda9527949373e60e84ca0a` (definition/files in JSON) |
| Retained host binary and original identity sidecar | `builds/m2-checkpoint-host`, `builds/m2-checkpoint-host.identity.json` |
| Matching preparation | `builds/m2-host-11.log`, `builds/m2-checkpoint-image-1.log` |

Executed serially on fresh mutable Stores with protected inputs, existing OS
caches, host SQLite/SDK/coordinator/spool, managed authenticated Linux daemon/live
core and real FUSE; no data-sharing mount or container Store. Performance and
historical verification used separate invocations and timers.

| Smoke / artifact directory | Performance | Historical mappings | Cleanup |
| --- | --- | ---: | --- |
| `m2-checkpoint-edits-1` | PASS: four distinct SDK/ordinary histories | 24/24 PASS | PASS, both phases/all four cases |
| `m2-checkpoint-small-1` | PASS: Init, two reads, three mutations/Commits | 4/4 PASS | PASS, both phases |
| `m2-checkpoint-deepseek-1` | PASS: exactly first FIVE frozen entries | 5/5 PASS | PASS, both phases |

PASS above means complete public operations/oracles/cleanup, **not numerical
qualification**. All created states and no-change aliases were independently
read after reopen through mounted FUSE. SDK edit member counts remain 1/1/1/3/0,
with public edit calls 1/1/1/1/0 and zero SDK-caused FUSE write counts. Ordinary
Exec keeps in-place range writes, whole replacements and tempfile/rename. DeepSeek
keeps the original ordinary importer. All four final unchanged edit Commits report
UpToDate. No selected DELTA or compressed group is claimed.

## Allocation, transactions and elapsed observations

Each arm below has **one observation**, using the initial diagnostic unchanged
baseline in [implementation-baseline.json](implementation-baseline.json). These
are descriptive ratios, not final paired estimates or statistical claims. The
current harness includes added untimed Init receipt rendering; final qualification
still requires its frozen identical-harness three-pair schedule.

| Case | Baseline final allocated bytes | FULL/RAW final allocated bytes | Allocation change | Baseline mutation+Commit sum ns | FULL/RAW sum ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| DeepSeek first five | 12,320,768 | 11,206,656 | −9.04% | 1,095,992,417 | 1,113,312,833 |
| SDK text-32k | 983,040 | 983,040 | 0% | 32,936,917 | 28,003,959 |
| SDK binary-8m | 11,141,120 | 10,354,688 | −7.06% | 28,219,670 | 35,819,625 |
| Ordinary text-32k | 1,179,648 | 1,179,648 | 0% | 40,490,999 | 45,286,042 |
| Ordinary binary-8m | 11,206,656 | 10,420,224 | −7.02% | 263,891,416 | 260,996,792 |
| Small files | 1,441,792 | 1,507,328 | +4.55% | 22,108,291 | 26,282,708 |

Initial small-file allocation is 1,441,792 bytes in both arms. Both binary candidate
histories start at 10,289,152 bytes versus 11,075,584 baseline; growth is 65,536 bytes
for SDK and 131,072 for ordinary in both arms. Small-file final growth is one
64-KiB SQLite page; it is retained, not reclaimed to improve the result.

All changed edit Commits have **one object-admission transaction**, plus existing
stage and publication transactions. Earlier `m2-edits-1` ordinary binary replacement
steps 2/3/4 had 29 admissions; `m2-edits-2` and this checkpoint have one. The unchanged
baseline had four for those replacements and two for the other changed edit steps.
DeepSeek candidate admission counts are **11/9/10/15/12**, versus baseline 2 each:
new-object and worst-selected-representation validation bounds still split larger
batches. There is no claim that every workload has fewer transactions. Stage and
publication are never counted as eliminated by the admission reduction.

Original-contract diagnostic elapsed misses (retained unchanged; accepted under
the owner’s subsequent milestone-2 tolerance):

| Quantity | Baseline ns | Candidate ns | Approved diagnostic allowance ns | Disposition |
| --- | ---: | ---: | ---: | --- |
| SDK binary step 2 | 5,971,876 | 8,699,042 | 7,971,876 | Above allowance |
| SDK binary return-to-A | 7,402,667 | 10,628,125 | 9,402,667 | Above allowance |
| Small-file shrink step 3 | 7,294,833 | 10,056,667 | 9,118,541.25 | Above allowance |

Small-file Init is 7,832,458 ns versus 7,028,667 baseline; first/repeated reads are
23,719,084 / 7,419,458 ns versus 19,658,084 / 7,439,041. These and the other
small-file steps are within their diagnostic allowances. DeepSeek replay is +1.58%
and each checkpoint is within its allowance. All ordinary and SDK-text steps and
all six historical-verification wall comparisons are within diagnostic allowances.
The JSON preserves every operand and individual step, including passing values.

The remaining SDK binary return-to-A latency was inspected without speculative
RAW tuning. Compared with `m2-edits-2`, object admission fell from 724,500 to
549,375 ns; content construction rose from 2,211,499 to 3,151,583 ns, while
snapshot calls/bytes remained 8 / 30,123 and CDC scanned 12,288 bytes. Runtime
wait/fence/checkpoint timings also vary. These receipts do not establish a new
membership/batching defect or justify a broader rewrite. The elapsed deviation
remains visible against the original contract and is accepted under the owner’s
subsequent milestone-2 tolerance. Neither noise nor future compression is claimed
to have removed the elapsed difference.

## Physical accounting, resources and coverage limits

Read-only physical census was performed after historical verification and outside
all operation timers. It does not replace pre-verifier allocated-byte receipts.
Canonical object counts/bytes equal the performance final counts; verifier-created
Branches do not add object packs. The six Stores contain only selected FULL/RAW
records: no unused records or retained workspace stages were observed. Unreachable
selected canonical objects, if any, remain counted; no reachability/GC claim is made.

| Case | Packs | RAW groups | FULL records | Pack BLOB bytes | Framing bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| DeepSeek first five | 58 | 424 | 5,557 | 9,017,260 | 37,193 |
| SDK text-32k | 5 | 11 | 46 | 62,863 | 530 |
| SDK binary-8m | 61 | 408 | 488 | 8,498,192 | 11,576 |
| Ordinary text-32k | 5 | 10 | 43 | 107,171 | 495 |
| Ordinary binary-8m | 61 | 408 | 481 | 8,521,222 | 11,541 |
| Small files | 7 | 23 | 461 | 399,152 | 2,877 |

Every observed pack is below the 256-KiB cap, including final partial packs. This
unused capacity is **not padding or allocated payload**. Full pack-size ranges and
zero observed unselected-record bytes are in the JSON. SQLite pages, indices and
other overhead remain in the actual allocated totals. Concurrent mixed-pack waste
and failed Init/fallback admissions are source-reviewed behaviors, not exercised
zero-waste guarantees. No DELETE, VACUUM or repack was run.

Performance host lifetime RSS peaks range from 10,158,080 to 32,817,152 bytes;
container lifetime peaks range from 4,919,296 to 14,909,440 bytes; sampled spool
peaks are 4,096–24,576 bytes. All are within diagnostic comparative allowances and
absolute safety budgets. No container OOM/swap or forced coordinator stop was
reported. Host phase CPU/I/O, container category snapshots, temporary/staging and
cleanup receipts are retained separately. These are lifetime/boundary/sampled
scopes, not exact operation-phase memory maxima or aggregate capacity evidence.

Candidate membership-query counts and actual BLOB/group-read call/byte counts are
**unavailable**, not zero. Snapshot database counts are logical reader API counts;
they cannot prove physical SQL/range-call counts. Source inspection establishes the
bounded page/recheck algorithm, and smoke receipts establish actual admission
transaction counts. Synchronous physical preparation and finalization remain inside
the measured public operations even where subphase instrumentation is incomplete.

Not exercised: arbitrary malformed/oversized records (including the RAW/DELTA
overlap), Zstandard or selected DELTA reads, late duplicate races, FIFO poison or
abandonment schedules, nonempty-Store/native topology fallback failures, direct
reconciliation conflicts, stage/head races, transaction/finalization failure
injection, disk-seen threshold populations and truncated spill recovery. Safeguards
were not weakened to avoid these cases. No unit/property/fuzz/race/crash suite,
fourth smoke, full-family campaign, 157-state replay or final-v3 qualification ran.

## Retained earlier evidence

- Unchanged product baseline: `28177560c8f049c02192e18c263cdc5543c1ab52`, approved
  contract commit `96e796431964f7a00af7fd1d7cc647029f2c5eaa`, harness/baseline commit
  `b4b967b91`. `implementation-baseline.json` retains exact source/binary/image and
  all six histories. Baseline host SHA-256 is
  `ce6a81a629d7d1adafaea74933acc502edb21e4ca650a396172f3574f26a0746`.
- M1 commit `86d04b95ed360835f1a861fd7104218794deb142`; `m1-small-1`,
  `m1-deepseek-1`, `m1-edits-1` retain their own source-specific performance,
  historical verification and cleanup results. They are not reused as M2 proof.
- `m2-small-1`: first-read 29,530,000 ns; `m2-small-2`: 23,176,834 ns. Both retain
  their valid observations and full historical receipts. The latter Init was
  9,228,167 ns, above its prospective allowance. Neither is final qualification.
- `m2-edits-1` source `654c583fd3c6129c6c8093851d56a0bf46f1dcf3d6c0d82e36139469d1ef1568`;
  `m2-edits-2` source `3601b48c2cf1f6d72a405ccf4055722c308971db1b103a16df8657866f7f530b`.
  Both passed 24 historical mappings and cleanup. Ordinary binary allocation
  changed **11,403,264 → 10,420,224 bytes between these two intermediate RAW
  iterations**, not between final candidate and unchanged baseline.
- Build failures remain under `builds/`: preparation-2/3 (captured Cargo diagnostics
  and ToSql correction), m2-host-1 (producer/sink edit correction), m2-host-6
  (infallible emitter correction), plus all subsequent build logs. No failed or
  slow observation was overwritten, discarded or reclassified as a passing gate.
- Frozen DeepSeek manifest SHA-256 remains
  `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`, tip
  `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Only its first five manifest entries
  were executed; no Git footprint arm or Git-proximity claim was added.

The checkpoint is ready for owner review in PR #81. Work stops at milestone 2.
Milestones 3–5, the full completion contract and final compressed-storage targets
remain deferred; this report grants no merge, deployment or migration authority.

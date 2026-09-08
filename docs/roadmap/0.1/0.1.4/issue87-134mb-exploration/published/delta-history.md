# History representation: a credible route to a much smaller Store

**The strongest architectural candidate is a content-addressed whole-file blob
for bounded files, physically encoded against its actual previous version using
native Zstandard prefix compression and short dependency chains.** A second,
smaller candidate applies that physical encoding to the existing canonical
chunks. Neither has a measured LayerFS saving yet. These are alternatives to
screen, not additive percentages or simultaneously scheduled treatments.

The user now targets335,552,512→134,221,004 allocated bytes, a201,331,508-byte
reduction, and permits canonical/architectural changes and slower operations.
This changes the search space substantially: retaining every current abstraction
is not a requirement. Integrity, retained states and honest allocation/cost
accounting still are. This document researches source at
`91a830766ce117e8c3a0cc305129e44762bdb052`, existing sealed reports and primary codec
documentation. Only a deterministic synthetic codec check was executed; no
retained-data encoding, replay, Store rewrite or product implementation occurred.

## Why history encoding deserves the larger experiment

The historical matched Git control held identical110,081 object identities and
157 trees while changing its packing policy. The no-delta pack body is
275,324,594 bytes; the delta pack body is51,989,900 bytes—a223,334,694-byte
difference. Complete allocated repositories are289,480,704 and56,373,248 bytes.
Compression level6 and two threads are recorded; the no-delta window/depth are0,
the delta window10/depth50. This is direct evidence of substantial historical
redundancy in this workload, not a numerical prediction for LayerFS. The old
physical no-delta representation does not survive; its sealed measurements do.
[Pinned Git results](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/docs/roadmap/0.1/0.1.4/deepseek-history/git-control-results.json)

That same control contains75,929 blob objects and33,995 trees. Its delta pack has
12,159 FULL blobs and15,459 FULL trees, so the matched membership establishes
**63,770 deltified blobs and18,536 deltified trees** by subtraction. Those counts
do not allocate compressed bytes by logical type. Git retains less filesystem
metadata and performs an all-history repack; its outcome is not proof that an
append-only chronological LayerFS writer can obtain the same size or cost.

The current LayerFS role inventory contains86,417 payload chunks representing
705,162,954 canonical bytes, of which78,393 FULL payloads represent598,565,032
canonical bytes. Current physical encoding has only8,024 DELTA records.
The shared metadata-only role join for this exploration resolves the older
“mixed-role” label: all9,755 mixed groups are mixed **structural roles**, not
payload/structural mixtures. Payload encoded groups total216,448,341 bytes;
structural encoded groups total78,792,537 bytes. These are exact separate group
accounts from the shared inventory, not proportional attribution.

An illustrative target budget makes the scale clear. Holding current
post-verification non-payload logical bytes fixed at102,421,163
(78,792,537 structural group bytes +703,824 pack framing +22,924,802 database
bytes outside BLOBs), and assuming zero future allocation adjustment, leaves
only31,799,841 bytes for payload under134,221,004. That would require removing
184,648,500 of216,448,341 payload encoded bytes. This is **a deliberately fixed
overhead scenario**, not a forecast: actual new index/pages/framing and allocation
adjustment must be measured. It explains why aggressive payload history encoding
and structural simplification should be evaluated together as a final design,
while their incremental effects are measured separately during exploration.

## Current mechanisms that restrict the search space

- `file/cdc/gear.rs` fixes8 KiB minimum,16 KiB target and32 KiB maximum chunks.
  Whole-file construction at `file/rope/build.rs:109` runs CDC and emits chunk
  objects, an extent map and a FileState. Each small changed file still pays for
  multiple authenticated/indexed objects.
- `PredecessorCursor` relates target chunks to up to four old extent IDs by
  logical overlap. It does not compare the whole current file with the whole
  previous file. Coverage also shares the already diagnosed127 metadata-read
  reservations per Commit.
- `objects/read.rs:219` deliberately exposes the FULL anchor of a DELTA hint
  without reconstructing the actual immediate predecessor.
  `objects/admission.rs:489` then matches the new target against that anchor.
  A growing difference from an older anchor can therefore be encoded repeatedly.
  The source mechanism is proven; its workload bytes need measurement.
- `objects/pack.rs` limits ordinary decoded groups to65,536 bytes and stores
  oversized singleton objects RAW. `objects/read.rs:306` enforces that RAW form.
  **Changing CDC to whole-file objects alone can make large files uncompressed.**
  Whole-file construction requires a matching bounded large-frame physical
  format, not a single chunk-size constant change.
- Zstandard is already installed and pinned via `zstd-sys2.0.16+zstd1.5.7`.
  Current `pack.rs:637` uses level1 with at most64 KiB window and static encoder
  workspace at most1 MiB. Prefix/dictionary frames and larger windows need an
  explicit new record/reader contract and memory accounting.

## Candidate1: whole-file canonical content, short native prefix chains

Introduce a versioned canonical `FileContent` encoding whose identity depends
only on canonical framing and **the reconstructed file bytes**, never on path,
parent version, chosen base, delta program, compression level or pack location.
For files within a prospectively fixed size bound, an inode points directly at
that object. It replaces the small-file path
`inode→FileState→extent leaf→payload chunks` with `inode→FileContent`.
For larger files, retain the existing segmented/CDC representation until a
separate large-file range-read design wins its own evidence. A concrete initial
screening bound is256 KiB. The shared authenticated input-manifest profile now
shows75,743 of75,929 unique Git blobs and795,755,930 of891,893,320 payload bytes
within that bound. Removing the seven symlink blobs/253 bytes gives75,736 regular
file blobs/795,755,677 bytes in scope. Only186 unique larger blobs remain, carrying
96,137,390 bytes; the maximum is1,241,221 bytes. This is broad byte coverage,
not selected encoding savings. The profile is the root agent's one shared
`wholefile-size-profile.json`; no second input census was run here.

There is also a material counterweight: all unique regular-file blobs total
891,893,067 raw bytes, whereas existing file-content chunks total703,348,161 raw
bytes (705,162,813 canonical bytes minus21-byte framing for86,412 chunks).
Whole-file identity therefore exposes188,544,906 additional unencoded bytes
before compression/deltas because it gives up exact chunk reuse across files
and versions. That difference is not allocated bytes or a measured regression;
it is a debt the stronger history representation must repay. Existing75,927
FileState identities versus75,922 regular Git-blob identities also means the
report must not assume a one-to-one new-object mapping from the FileState count.

Physical representation is chosen independently of canonical identity:

```text
FULL_FRAME:  content bytes compressed without a prefix
PREFIX_FRAME: authenticated prior content ObjectId + reconstructed length
              + one Zstandard frame encoded using that prior content as prefix
```

Use the actual prior same-path file content as the first candidate; exact CAS
filtering happens first. If its selected physical representation is itself a
prefix frame, reconstruct it within a **depth4** cap and a prospectively fixed
cumulative reconstructed-byte cap, initially four times the file-size bound for
the complete read closure. If the proposed new edge would violate either cap,
choose a FULL frame. These are concrete screening settings, not claimed optimal
parameters. Depth counts and whether the final target belongs in the byte cap
must be declared exactly; the equations below use the whole closure including
target. No unbounded chain is proposed.

One independently decodable content frame per blob is a simple starting layout;
packs concatenate frames with explicit locators. It can lose current compression
sharing among small records, and its frame headers/checksums/base IDs may exceed
the present group overhead. Count every byte. Do not report only the patch-frame
length and omit the complete-frame alternative, base record or index. Structural
objects may retain their current grouping while the structural agent evaluates
their own representation changes.

Zstandard's primary API documents `ZSTD_CCtx_refPrefix` as a single-use prefix for
the next frame, requiring the same unmodified prefix during decompression and a
large enough window. Building prefix tables has nontrivial latency. These are
reasons to reuse the installed codec, not promises of free compression.
[Zstandard prefix API](https://github.com/facebook/zstd/blob/v1.5.7/lib/zstd.h)
The project's `--patch-from` mode demonstrates this intended old-file/new-file
use case; its published results use other workloads and are not transferred to
LayerFS forecasts. [Zstandard patching engine](https://github.com/facebook/zstd/wiki/Zstandard-as-a-patching-engine)

**What can disappear:** CDC scanning and extent construction for bounded files;
their FileState and extent-leaf wrappers; chunk-span handoff/cursor search for
those files; custom COPY/INSERT matching and instruction handling for new prefix
records. The current75,927 FileStates and75,927 extent leaves occupy15,385,370
canonical bytes. Only the subset actually replaced can disappear, and its
compressed/index footprint must come from the new image. These canonical bytes
are not additional encoded savings. Large-file fallback and old-format support
mean the whole CDC/legacy decoder subsystem cannot yet be deleted globally.

**What is added:** a direct-file canonical variant, one prefix-frame record
decoder, strict dependency traversal, full-content identity verification, and
bounded whole-file buffering/cache ownership. This is a genuine format/reader
change. It should simplify the new small-file path rather than append a second
general-purpose storage backend.

**Main failure modes:** cross-file shared chunks may stop deduplicating; many
new unrelated files have no useful prior; insertion/rename lineage can be missed;
full-file authentication increases tiny-range read work; shallow periodic FULL
resets can erase expected savings; and independently compressed frames can lose
group-level redundancy. Every one is measurable in an offline content image
before a costly product implementation.

## Candidate2: native prefix deltas on the existing canonical unit

Keep current chunk ObjectIds, extent/FileState objects and range-read structure,
but permit physical prefix frames against the **actual selected predecessor**
within the same depth/cumulative-work bounds. The predecessor is reconstructed
instead of replacing it with its old FULL anchor. This isolates the representation
engine and anchor policy from canonical granularity.

Start with existing legitimate candidates and complete measured correspondence;
do not assume a native codec can recover absent bases. The first screening
comparison uses one authenticated candidate and FULL fallback. A wider search
can then be tested as a separately named arm: up to four historical same-path
versions, followed only if needed by a small fixed window of compatible objects
already admitted before the target. Never allow future snapshots into an online
claim. Any persistent similarity/history index becomes Store bytes; a transient
index has a declared build/read/CPU/memory cost. Neither can be free off-book
infrastructure.

This retains the CDC cross-file reuse and bounded random-read units, but also
retains structural overhead and the need for correspondence. It may be cheaper
to implement and could disprove the need for a canonical break if it wins enough.
It cannot inherit the whole-file wrapper/index savings of Candidate1. In a fresh
format that retires the custom delta program, `pack::delta_record` and its COPY/
INSERT interpreter can be replaced by installed-codec prefix operations; while
old Stores remain readable, their decoder must remain or be isolated in an
explicit import tool.

The reason to compare these two designs is causal: if current chunks plus useful
actual predecessors reach the target envelope, a whole-file migration is needless;
if whole-file frames win materially after counting lost chunk reuse and wrappers,
the canonical break has measured justification. This does not authorize doing
both optimizations simultaneously in a product benchmark.

## Versioned file recipes: physical, not history-dependent identity

A file recipe such as `COPY(prior_file,offset,length)+INSERT(literal)` is another
way to express a physical delta. Placing that history-dependent recipe directly
inside the canonical identity creates different ObjectIds for identical content
constructed through different histories. It also turns predecessor versions into
logical references, complicating equality, deduplication, retention and merge
behavior. The user allows canonical changes, but those consequences buy no
inherent compression advantage over a physical representation of the same bytes.

Prefer a canonical whole-file content identity with the recipe below it. Native
prefix encoding delegates recipe construction/reconstruction to the existing
codec and avoids a new general patch language. If workloads later demonstrate
that authenticated tiny-range reads require multi-base slice recipes, that is a
different reader design: it needs authenticated block boundaries/proofs, a cap on
base fan-out and a cost comparison against the existing extent tree. It should
not be smuggled into the first whole-file experiment.

## Conservation and read/write cost model

For selected targets T, let L be retained logical objects, B the transitive
physical dependency closure of all selected prefix frames, and R=L∪B. For each
retained physical record, charge its selected representation once:

```text
content_record_bytes = sum(FULL frame bytes + FULL framing)
                     + sum(PREFIX frame bytes + base IDs + PREFIX framing)
pack_bytes = content_record_bytes + structural records + all directories/headers
database_logical_bytes = pack_bytes + SQL row encoding + index/metadata pages
                      + page unused/overhead + freelist + explicit residual
Store_allocated_bytes = database_allocated_bytes + sidecar_allocated_bytes
physical_base_only = B minus L
```

Do not add reconstructed target/base bytes to physical frame bytes. Shared bases
count once, even when many targets depend on them. Required base-only records and
their containing shared frames/groups remain in the image. Current base-only
count0 does not imply the new policy's base-only closure is0. Choosing new bases
can change it, especially when branches/versions are removed from logical roots.

For a chain with D delta edges and decoded content lengths n0…nD, a cold read
must fetch the distinct required frames and reconstruct at most
`sum(n_i)` bytes under the declared cap, plus authentication and wrapper work.
At mostD+1 content frames are needed when there is exactly one base per record;
shared pack fetches may reduce reads, but do not assume cache hits. Two alternating
output/base buffers can bound serial reconstruction near
`max(n_i+n_(i+1)) + encoded input + decoder workspace`, not all versions in memory,
provided prefix lifetime rules are obeyed. A256 KiB unit with a1 MiB whole-closure
cap allows four256 KiB decoded nodes, i.e. only three delta edges in that worst
case; the depth4 guard is an additional cap, not permission to exceed1 MiB.

Small range reads can require reconstructing the whole bounded file. Full-file
writes need previous-content reconstruction, prefix-table construction, candidate
compression, FULL alternative encoding, admission and verification before the
public acknowledgement. Per-target work across repeated old bases can dominate
unless a bounded admitted-content cache or batching reuses it. Such cache bytes
and read amplification belong to the foreground result. The new owner tolerance
is broader than the old roughly30% guidance; report absolute latencies, tails,
CPU/RSS and read workloads rather than inventing a new slowdown ceiling.

## Exact evidence needed to judge the60% target

Before any retained-data encoding, freeze a separate offline exploration contract
and input manifest with the root agent. Reuse one authenticated read/inventory
pipeline and one logical snapshot; do not independently decode the retained Store
for every arm. The necessary comparisons are:

1. **FULL baseline images per canonical unit.** Reconstruct unique file contents
   using existing authenticated decoders/oracles; make current-chunk and bounded-
   whole-file FULL-only images with the same declared codec settings. Preserve
   identical all157 selected states, not simply HEAD. Count unique content,
   lost/shared chunks, wrappers, indexes, frame/pack overhead and all retained
   bases. This separates granularity/serialization from delta opportunity.
2. **Actual-parent prefix images.** On the identical ordered admissions, permit
   only already admitted prior versions, with fixed depth/decoded-byte caps and
   FULL fallback. Include initial content and later newly created paths, not only
   favorable modified pairs. Compare whole images and reconstructed-state oracles.
   This determines whether native prefix+actual-parent encoding has material
   retained byte opportunity without a global search.
3. **Only if the parent arm misses:** a prospectively bounded historical same-path
   window, then an explicitly separate broader-similarity arm. Attribute its extra
   saving and candidate-fetch/match/memory work. An unrestricted all-history
   search is an offline opportunity ceiling, never an online public-path result.
4. **Reconstruction stress before product work.** Verify every ObjectId and state;
   wrong/missing bases, cycles, excessive depths/windows/output lengths and
   truncated frames must fail strictly. Read cold individual files, small random
   ranges, all157 historical states and fan-in-heavy bases. Record closure depth,
   cumulative decoded bytes, duplicate decodes and peak buffers. No full-data
   logging in timed operations.
5. **Complete allocation/public-path proof for the chosen design.** Only after an
   offline image fits the remaining target budget with credible index/metadata
   allowance, implement one declared design in a fresh prospective Store and
   measure original acknowledgement allocation including sidecars, canonical
   correctness, foreground writes/reads, cleanup and final pre-verification
   snapshot. Offline frame sizes alone do not establish134,221,004-byte product
   allocation or acceptable operation cost.

Retain every negative result and full parameter/source/library identity. If the
images cannot fit the target after complete accounting, reject that design rather
than present a successful individual patch as evidence for60% Store savings.
Canonical migration must rebuild all retained roots/references in a fresh Store,
with source→new mappings and unchanged filesystem semantics; old canonical IDs
cannot be preserved by declaration. A physical-only change can retain canonical
IDs but still requires a versioned reader/Store compatibility contract. Neither
path mutates historical evidence in place.

## Synthetic native-codec check executed

`delta-prefix-probe.py` generates nine256 KiB byte arrays with fixed seed87. Each
new version changes256 fresh bytes at a disjoint offset. Native Zstd1.5.7 level3,
single-thread, is run against no prefix, the initial version, and the immediate
previous version. All24 decompressions are byte-for-byte checked. FULL frames
are262,163 bytes; the final initial-prefix frame is2,141 bytes, while its immediate-
previous-prefix frame is309 bytes. This demonstrates the intended cumulative-edit
mechanism and native API availability on a deliberately favorable synthetic
fixture. **It measures no LayerFS savings or eligible workload fraction.** CLI
window decisions/static-workspace behavior are not the current product encoder.

The check and exact outputs are in `delta-prefix-probe.py` and
`delta-prefix-probe.json`. No temporary synthetic inputs survive the check.
Reproduce with:

```sh
python3 docs/roadmap/0.1/0.1.4/issue87-134mb-exploration/delta-prefix-probe.py
```

Git's primary pack documentation confirms the relevant alternative search model:
candidate sorting/windowing and depth limits, with deeper chains increasing
reconstruction work. Its pack specification permits a base that is itself
deltified. These explain why a short-chain comparison is reasonable; they do not
make Git's historical depth50 an appropriate default for synchronous LayerFS.
[Git pack-objects](https://git-scm.com/docs/git-pack-objects),
[Git pack format](https://git-scm.com/docs/gitformat-pack)

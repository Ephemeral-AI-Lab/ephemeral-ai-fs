# A credible architecture search for a 134.2 MB Store

**134.2 MB is a defensible experimental target, not yet a forecast.** The aggressive search found two large representation opportunities: file-version encoding and structural-page deltas. It also found that deleting wrappers or replacing SQLite alone cannot reach the target. The strongest candidate architecture combines bounded whole-file content with actual-parent native prefix compression, and obtains structural predecessor hints directly from tree edits. The pieces must be measured separately before claiming their combined effect.

The target is335,552,512→134,221,004 allocated bytes, requiring201,331,508 fewer bytes. The user's expanded authority permits exploring architecture/canonical breaks and slower foreground operations. This report therefore does not treat current CDC, FULL-only metadata, depth-one anchors or the custom matcher as sacred. All157 states, filesystem semantics, integrity, honest allocation and complete read/write cost remain required. No product patch, retained-data encoding or replay was executed: research used sealed metadata, one read-only native Git pack audit, primary documentation and small synthetic codec/API checks.

## The evidence now supports looking beyond a 10–20% target

The previous10–20% suggestion was not an evidence-derived expectation. No result here establishes that as a ceiling. The exact measured representation accounts are:

| Existing representation | Bytes | Scope |
|---|---:|---|
|LayerFS payload encoded groups|216,448,341|Post-verification group inventory; canonical content unchanged|
|LayerFS structural encoded groups|78,792,537|All remaining groups, including mixtures of structural roles|
|LayerFS outer pack framing|703,824|Nested inside pack BLOB total|
|LayerFS pack BLOB total|295,944,702|Payload+structure+framing|
|Historical Git non-delta pack|275,324,594|Same110,081 selected Git objects|
|Historical Git delta pack|51,989,900|Same objects; different packing policy|
|Git blob entries in delta pack|46,982,533|35,244,143 FULL +11,738,390 DELTA entry bytes|
|Git tree entries in delta pack|4,976,275|3,138,693 FULL +1,837,582 DELTA entry bytes|
|Git commit entries|31,060|157 FULL commits|
|Git header/trailer|32|Completes exact51,989,900-byte pack equation|

The Git FULL→delta pack difference is **223,334,694 bytes**, greater than the201,331,508-byte target reduction. That is strong workload-specific evidence of exploitable history redundancy; it is not a transferable LayerFS saving. Git's recorded final pack has63,770 blob DELTAs and18,536 tree DELTAs, with actual maximum depths50 and36 respectively. Native `verify-pack -v` validated exact membership against109,924 frozen snapshot object IDs plus157 mapped commits; pack/index/source hashes were unchanged. No new packing or historical timing pair was collected.

Git can search objects in a final batch using type/size/name heuristics. Its bases can include versions not available at an earlier checkpoint; the audit does not claim chronological prior-only encoding matches it. Its metadata differs from LayerFS, and packing costs are separate. [Official Git packing rules](https://git-scm.com/docs/git-pack-objects), [verify-pack fields](https://git-scm.com/docs/git-verify-pack.html).

The refined group join also corrects an ambiguous earlier description: the9,755 “mixed-role” groups are **structural mixtures**, not payload/structural mixtures. Payload and structure are separate physical lanes here. Structural compression is weak at95,800,627 decoded→78,792,537 encoded bytes; much of it is hash/reference-heavy. Payload is619,957,422 decoded-record bytes→216,448,341 encoded. Reconstructed canonical DELTA targets are another logical dimension and are not added to those decoded records.

## Opportunity1 — change the file-content unit and use actual-parent prefix compression

**Candidate architecture:** for bounded files, store one content-addressed `FileContent` object and let the inode point directly to it. Canonical identity depends only on the reconstructed bytes and a versioned canonical framing—not path, parent or chosen delta program. Physically store a normal compressed frame or a native Zstandard prefix frame referencing the actual previous content. Bound chain depth and cumulative reconstruction work. Larger files keep a segmented representation.

A concrete first screen is files with at most262,144 **payload bytes**, physical prefix compression of those payload bytes, and separate full canonical authentication. Cap dependency depth at4 edges and cumulative decoded payload across FULL base, intermediate versions and target at1,048,576 bytes; either violation forces FULL. At maximum file size the byte cap is stricter than four edges. Prefix+target window and buffer bounds must cover both live payloads; canonical framing, record headers and indexes are additional accounted bytes. These are frozen screening settings, not optimized thresholds.

**Why the coverage is large:**75,736 unique regular-file blobs/795,755,677 raw bytes fall within256KiB—most unique file count and about89% of unique source bytes. Only186 larger blobs/96,137,390 bytes remain. Whole-file construction directly supplies the prior file identity; it avoids reconstructing predecessor chunk correspondence and its127-reservation bottleneck for that path.

**What can disappear for bounded files:** CDC scanning, extent construction, FileState/extent wrappers, chunk-span correspondence, and custom COPY/INSERT matching for the new physical format. Native `ZSTD_CCtx_refPrefix`/matching can replace the custom patch search rather than adding another handwritten matcher. Old readers/large-file fallback mean legacy components cannot be deleted globally until a version transition is complete. [Zstd1.5.7 prefix API](https://github.com/facebook/zstd/blob/v1.5.7/lib/zstd.h), [official patching use case](https://github.com/facebook/zstd/wiki/Zstandard-as-a-patching-engine).

**The counterweight is substantial:** unique regular-file contents total891,893,067 raw bytes, versus703,348,161 unique current file-chunk payload bytes. Whole-file identity gives up188,544,906 bytes of raw chunk reuse before compression. This is not an allocated regression, but history encoding must repay it. Independent per-file frames can also lose group sharing. Merely raising CDC's maximum is insufficient: current oversized singleton records are RAW, so this requires a matching physical-format/reader change.

**Read and speed cost:** a tiny read may need a whole bounded file and its base closure, up to1MiB of decoded payload, instead of current≤64KiB group units; an existing DELTA may also require its separate FULL-base group, plus metadata traversal. Prefix table construction and parent reconstruction add CPU and memory; removing CDC and repeated structural traversal may offset some of it. The net effect is unmeasured. The user's willingness to sacrifice speed broadens the screen, but a60% size claim cannot hide severe random-read or tail-latency regressions.

The native prefix capability test performs24 synthetic byte-for-byte round trips. Its deliberately favorable generated mutations are not a LayerFS compression-ratio estimate. No real input was re-encoded.

## Opportunity2 — delta-encode structural pages using lineage the tree already knows

**This is larger than wrapper cleanup and may not require a canonical break.** Current inode-table leaves contain56,781,912 canonical bytes. Groups containing inode-table nodes occupy69,595,996 encoded bytes, including68,706,890 shared with other structural roles. That envelope is an affected upper bound, not bytes attributed exclusively to inode tables.

The metadata-only reference join finds880,418 inode-record reference occurrences in inode-table leaves;790,456 reference records already retained at earlier checkpoints. At32 bytes per CAS reference,25,294,592 bytes of those reference fields repeat older identities. The equally wide key fields and repeated page layouts provide further potential, but are not independently measured savings. About89.8% older references supports a version-delta hypothesis; it does not prove every one appears in one chosen base page.

There is a concrete source hook: the generic tree editor carries `Page.origin` into `Node.id`; edited nodes remember the old immutable ID, but ordinary `put_owned(canonical)` drops that origin at persistence. Supplying this existing ID as an optional physical predecessor for exact inode-table leaves avoids a global similarity search. Splits/merges without a single valid origin fall back to FULL. Existing physical record reconstruction authenticates generic canonical data; the current writer's candidate policy is explicitly chunk-only.

**Smallest treatment:** preserve inode-leaf canonical bytes/IDs and all logical graph semantics, pass existing origin, and permit a bounded physical delta/native-prefix alternative for that one exact role. Count the actual complete groups, FULL fallback/base closure, reads and matching work. It must not consume the file correspondence allowance or obtain a free unaccounted index.

This could be an important companion to payload history encoding, but their savings cannot simply be added: changed canonical file roots alter inode bytes and group membership. The complete combined image must be measured. No compression ratio is assigned to this opportunity before encoding it.

## Opportunity3 — simplify canonical wrappers and index ownership

For the current all-history graph,75,927 FileStates each have a distinct extent mapping root, and all maps are leaves. A hypothetical fused leaf representation reduces their15,385,370 canonical bytes to5,970,422 and removes75,927 selected objects. The raw difference9,414,948 bytes is **not** allocated savings. The whole FileState/extent group envelope is only9,136,919 encoded bytes, and that overlaps any whole-file conversion.

Likewise, a hypothetical dense40-byte locator row for366,141 objects is14,645,640 bytes before fanout, checksums, mutable tail and transaction costs, versus18,837,504 bytes of current objects-index pages. The4,191,864-byte arithmetic gap is not a proven reclaimable allocation. Reducing the object population may help more than changing the index backend.

Treat these as simplifications accompanying a winning representation, not independent headline optimizations. Even deleting **all** structure, indexes, framing and allocation overhead leaves216,448,341 payload encoded bytes, still82,227,337 above the target. Metadata-only or SQLite-only work cannot deliver60%.

## A concrete budget for134.2 MB

One design budget worth testing is:

| Future component budget | Bytes | Required change from current group bucket |
|---|---:|---|
|Payload representation|70,000,000|146,448,341 fewer encoded bytes, about67.7%|
|Structural representation|45,000,000|33,792,537 fewer encoded bytes, about42.9%|
|All indexes, SQL/container framing, sidecars and allocation adjustment combined|19,000,000|Must be measured as a complete allocated budget; not assumed from current stats|
|Total|134,000,000|Within134,221,004 target|

This is **a required allocation of the target budget, not a predicted outcome**. Seventy MB for payload is looser than the historical Git blob47MB reference;45MB for structure is far looser than Git's different5MB tree representation. That leaves room for LayerFS semantics and shallower history, but whether these allowances can be met simultaneously is unknown. The19MB non-group budget requires fewer objects and/or lower index/placement costs; it is not already satisfied by the18.8MB index alone.

`target-budget.json` records multiple explicit overhead/structural assumptions and the corresponding required payload sizes. It never splices the current post-verification22.9MB non-BLOB database bytes and pre-verification16.7MB signed allocation adjustment into an exact same-snapshot account. Original acknowledgement allocation remains the baseline.

## What I would not bet the target on

- **A stronger Zstd level alone.** Existing independent frames cannot exploit history they were never given. The installed1.5.7 library estimates1,791,447 bytes of encoder context for level19 at64KiB, above the current1MiB context cap. An8MiB/level19 profile needs about102,006,934 bytes for input, maximum output and context before application/decoder costs. Those are library estimates, not measured RSS or compression gains.
- **Large frames without a read model.** EightMiB is128× the current64KiB decode bound. Existing operation counters already sum12.45GB of decoded work over a715.8MB final decoded inventory; more coarse decoding can dominate read costs.
- **Inlining everything.** It can duplicate payloads used by several inodes and remove useful CAS sharing; base closure may keep old backing objects anyway.
- **History-dependent canonical recipe IDs.** Encoding a file's identity as “parent plus patch” can make identical contents have different IDs. Prefer history dependence in physical representation while preserving content identity.
- **Background repacking disguised as foreground savings.** A future architecture could synchronously re-encode old immutable content against a newly available version and atomically replace locator generations. That would relax locator immutability, require reader pinning/old-pack GC and add write amplification/peak space. It is a legitimate aggressive alternative, but only post-finalization allocation can be compared, and it is not the first screen proposed here.

## One next experiment: bounded version-aware file-content screen

The first empirical experiment should compare the chosen≤256KiB whole-file/actual-prior prefix representation with the current complete retained representation on the same frozen157 source chronology. It is one declared architectural treatment, not a level/window/chain sweep. An offline encoding screen measures potential only; an actual product result requires synchronous public-path construction and all157 verification afterward.

Freeze one codec level and exact prefix/window/resource profile before running; use only bases available before each checkpoint, CAS-filter before choosing a representation, enforce4-edge/1MiB closure bounds, retain FULL fallback, and verify every reconstructed byte and canonical identity. Preserve all regular-file modes, symlinks, inode/metadata semantics and every retained state. A representation that drops semantics is not a60% win.

Report complete candidate bytes: content frames, required bases, larger-file fallback, regenerated structural data, new locator/index/framing and allocation. If only a payload screen is implemented initially, label it payload-only and calculate the remaining structural/overhead budget—it is not a134MB Store. Separately screen origin-based inode-leaf deltas before integrating them into a combined architecture; do not attribute a combined result to either piece without that evidence.

Measure parent fetch/reconstruction, encoding, random-range cold/warm reads, full-file reads, repeated-history verification, memory/queue peaks, peak temporary disk, total writes and original acknowledgement/finalization boundaries. Require explicit source/binary/image/candidate-format custody and a disposable new output directory; do not rewrite the original Store. Retain negative results. Stop on identity, reconstruction, bounds, retention, cleanup or provenance failure rather than extending limits after observation.

The earlier roughly30% extra-foreground-time guidance is a useful initial product-feasibility comparison, but the expanded user request permits researching candidates outside it. Report the actual slowdown and absolute latency; do not call every speed sacrifice acceptable. No new storage/latency gate is invented beyond the stated target and retained correctness requirements.

## Status and deliverables

The aggressive opportunity research is complete. The target is plausible enough to justify a version-aware encoding experiment, but no evidence-backed optimization ratio is available yet. The main advance is a credible architecture and explicit budget, backed by exact existing pack decomposition and structural lineage—not a promise that lifting one limit will remove60%.

The subreports contain source mechanisms, deletion boundaries, compatibility/migration and negative-test requirements: delta-history.md, canonical-index.md and compression-packs.md. The external directory `layerfs-storage-v3-runs/issue87-134mb-exploration-91a8307-1` contains the target model, authenticated whole-file profile, structural metadata account, read-only Git pack account and synthetic/resource-estimate checks. Previous sealed artifacts and product code remain unchanged. #87 stays open; no PR merge, S3, M5 or release qualification.

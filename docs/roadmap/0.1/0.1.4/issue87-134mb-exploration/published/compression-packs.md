# Compression and pack construction against the134,221,004-byte target

**A60% reduction is neither demonstrated nor ruled out.** There is no measured basis for a10–20% ceiling, nor for promising that a stronger Zstandard level will reach the target. The strongest next architectural experiment is the sibling proposal: bounded whole-file canonical content, encoded physically against its actual prior content through native prefix compression and a short, byte-bounded dependency chain. This document does not schedule a separate codec/group sweep. It explains why history access is more promising than a level knob, what pure compression/construction alternatives would cost, and how to screen the chosen design honestly.

Scope: source `91a830766ce117e8c3a0cc305129e44762bdb052`, previously sealed full157 metadata and version-pinned primary documentation. One streaming aggregation of existing group CSV metadata and three native-library **resource-estimate queries** were performed. No Store was opened, payload was decoded, retained data was compressed, product code was changed or replay was launched. Any retained-data encoding needs the root owner's separately frozen contract and one shared input pipeline.

## Exact byte populations

The groups CSV SHA256 is `313e9ad1b198de9a6b626bbca6a2a8f9efd355d539488c7c14c7e7f73439cb2a`. Its hash was checked against the sealed analysis manifest before metadata aggregation. The sibling's exact group/role join establishes that all mixed-role groups mix **structural roles only**; none mixes payload with structure. Therefore these encoded populations are exclusive whole groups, not proportional record attribution.

| Population, post-verification retained original | Groups | Decoded group bytes | Encoded group bytes |
|---|---:|---:|---:|
|Payload-only groups|29156|619957422|216448341|
|Structural-only groups|9918|95800627|78792537|
|All groups|39074|715758049|295240878|
|Zstandard groups, overlapping codec partition|37733|703543472|283026301|
|RAW groups, overlapping codec partition|1341|12214577|12214577|

The role rows sum to the all-groups row; separately, the two codec rows sum to it. Do not add both partitions. The outer pack framing is703824 B, so complete pack BLOB bytes are295944702. Post-verification object_packs pages are299909120 B, including the BLOBs. Their difference is3964418 B of SQL row encoding, page overhead and unused allocation within this table—not a guaranteed removable quantity. Other SQLite pages remain separate.

Decoded-size histogram:4847 groups≤4096 B;11608 groups4097–16384 B;22410 groups16385–32768 B;209 groups32769–65536 B. Average decoded group size is about18318 B. Payload group compression is about2.864:1; structural group compression only about1.216:1. The poor structural ratio is observed, but is not proof those bytes are intrinsically random: repeated references across independently compressed historical objects may simply be outside the current frame.

The RAW bucket and outer framing are small relative to the201331508 B required allocation reduction. Making every RAW byte vanish or eliminating every pack header would still not explain the target. Pure container placement also cannot remove the216448341 B of encoded payload or78792537 B of encoded structures.

## What output ratio must be achieved

The owner target is `floor(0.4 * 335552512) =134221004` allocated bytes. These scenarios are arithmetic requirements, **not forecasts**, and explicitly vary assumptions about future overhead:

| Explicit scenario | Bytes reserved outside new pack representation | Remaining pack-byte budget | Required reduction from295944702 B pack BLOBs |
|---|---:|---:|---:|
|Keep post-verification non-BLOB logical bytes; assume future allocation adjustment0|22924802|111296202|62.39%|
|Additionally reserve the old final-ack adjustment as a hypothetical future allowance|39644674|94576330|68.04%|
|Unrealistic all-other-costs-zero optimistic bound|0|134221004|54.65%|

The second row combines quantities from different snapshots only as a declared hypothetical budget. It is not an observed decomposition of acknowledgement allocation. Future pages, index sizes, frame metadata, dictionaries, physical bases and signed filesystem adjustment must be measured in the candidate.

If all715758049 decoded record bytes remain unchanged, the first scenario requires about6.43:1 compression into the whole pack budget; the second requires about7.57:1. Current encoded groups achieve about2.42:1. These are demanding requirements. Canonical simplification or better physical delta programs can reduce the input itself, so these ratios are **not** lower bounds for a redesigned representation.

With structural bytes and current non-BLOB overhead held fixed, only31799841 B remain for payload. That would require an85.31% payload encoded-byte reduction. Conversely, deleting all current structural groups cannot achieve the60% whole-Store target by itself. The final design must budget payload and structure together; independently measured benefits cannot be naively added when they share frame/group/index effects.

## Why a stronger level alone is not the strongest mechanism

Current `objects/pack.rs` pins Zstd1.5.7 level1, `windowLog≤16`, a1MiB static encoder context, a256KiB decoder context, ordinary decoded groups≤65536 B and packs≤256KiB. `admission.rs` aims for32KiB payload groups and16KiB structural groups. Every group becomes an independent one-shot frame. Oversized canonical singleton records are RAW. Changing CDC to larger whole-file objects without changing this physical reader/writer contract can therefore **disable compression** for those larger objects.

Zstandard levels select different search strategies and memory parameters. Higher search effort may find better matches already visible in the input; increasing a window does not create historical bytes that were never supplied. Context reuse improves resource handling but does not itself improve compression ratio. [Zstd1.5.7 API manual](https://facebook.github.io/zstd/doc/api_manual_v1.5.7.html)

Zstd frames are independent, while blocks within a frame can depend on previous blocks. Concatenating the current compressed frames into a larger file preserves their reset boundaries. Merely raising `windowLog` while still compressing separate32KiB inputs cannot discover matches in preceding frames. To expose those bytes, construction must provide a larger decoded frame or an explicit prefix/dictionary. [Pinned Zstd format](https://github.com/facebook/zstd/blob/v1.5.7/doc/zstd_compression_format.md)

The historical matched Git control offers a workload-specific discriminator: FULL-only pack275324594 B versus delta pack51989900 B, an81.12% pack-body difference223334694 B within the same110081-object/157-tree membership. That is larger than the required201331508 B allocation cut. It establishes substantial history redundancy under a different representation and packing process; it does not prove chronological LayerFS can realize it or that metadata is free. It is stronger evidence for history-aware representation than an unsupported generic compression-percentage range. The allocation/metadata/interface/packing qualifications from the root report remain essential.

The root's additional read-only `verify-pack` accounting authenticates the exact110081-object membership and decomposes the51989900 B pack: blob records46982533 B (35244143 FULL +11738390 DELTA), tree records4976275 B (3138693 FULL +1837582 DELTA), commit records31060 B, plus32 B outer pack header/trailer. This is exact Git record accounting, unlike proportional attribution inside a shared Zstd group. Observed maximum chain depths are50 for blobs and36 for trees. The final pack may use bases from later selected snapshots; no chronological-base availability or shallow-chain size equivalence is inferred. These measurements reinforce the representation opportunity while also identifying why native short-prefix chains must be tested rather than inheriting Git's byte total.

## Concrete resource estimates without compressing data

`compression_profile_costs.py` calls only `ZSTD_getCParams`, `ZSTD_estimateCCtxSize_usingCParams` and `ZSTD_compressBound` on the installed library, refusing a version other than1.5.7. No input payload is supplied and no encoder/decoder context is created. `compression-profile-costs.json` retains exact parameters, library path and results.

| Declared profile | Input/frame cap | Estimated encoder context | Maximum output buffer | Input+context+maximum output |
|---|---:|---:|---:|---:|
|Current level1/64KiB cap|65536|304152|65824|435512|
|Level19/same64KiB cap|65536|1791447|65824|1922807|
|Level19/8MiB frame, windowLog23|8388608|85196950|8421376|102006934|

All quantities are integer bytes. These are library estimates, not measured RSS, CPU or compression output. They exclude application object ownership, retained FULL alternatives, dictionaries, SQLite, caches, worker queues and allocator effects. They do not size the chosen prefix architecture: its exact canonical-versus-payload frame bound, live prefix and selected CParams must be frozen before a separate estimate is meaningful. The inspected dynamic library is `/opt/homebrew/lib/libzstd.dylib`; matching version does not replace a future producer-build identity seal. The parameter-estimate interface is used here only for pinned diagnostic inspection, not proposed as an unversioned dynamic production ABI.

**A concrete implementation obstacle is already established:** level19 at the current64KiB cap estimates1791447 B of context, exceeding the existing1MiB hard context allowance. Replacing the level literal without redesigning resource ownership fails that otherwise valid64KiB input on the current bounded-workspace path; this is not a claim that every smaller group fails. The user permits a larger memory budget, but it must be explicit. Two aggressive8MiB encoders alone have an estimated input/output/context sum204013868 B before other application costs.

## Construction alternatives, their costs and compatibility

| Alternative | Affected measured bucket and mechanism | What remains unmeasured / costs / compatibility |
|---|---|---|
|Stronger level on identical groups|All295240878 encoded group B; stronger within-group match search|No retained-data output estimate exists. Preserves record membership and potentially canonical/pack wire format if frames remain within old reader limits; encoder workspace and CPU change. It cannot reach across frame boundaries. Not the chosen large-gain mechanism.|
|Larger independent frames, for example the fixed8MiB/level19 profile above|Potentially all715758049 decoded record B; expose repetition across many current groups and amortize frame resets|Gain depends on ordering and actual shared substrings. New group/pack bounds and locators/read buffers are required. An8MiB unit is128× the current64KiB maximum decode granularity; frame buffering/context costs are shown above. This is a researched alternative, not another scheduled run.|
|Group records by logical role and nearby history|Potentially both payload216448341 and structural78792537 encoded B; put similar versions/references into one codec window|Requires an authenticated lineage/order rule. ObjectId sorting is cryptographic order, not similarity. Sorting all retained history is an offline construction/compaction policy; it must not be presented as an unchanged chronological append writer. Sorting/storage/rewrites and concurrency costs belong in the result.|
|Shared trained dictionary while keeping small frames|Whole small-group population; reusable patterns can be visible without decoding preceding frames|Training, dictionary bytes/IDs/retention, loading and cache costs are added. This is not automatically a substitute for a specific previous file's content. A dictionary trained on future snapshots is an offline-only result until an online training protocol includes its cost.|
|Actual previous file as native prefix|The broad historical payload population and potentially repeated structured objects; complete previous bytes are available directly to native matching|This is the chosen architectural direction with the sibling's bounded whole-file design. Base reconstruction, depth/byte limits, FULL alternatives, identity verification and dependency retention are required; compressed bytes remain a measured future result.|
|Immutable flat pack files plus compact locator index|At unchanged pack bytes, the directly exposed current object_packs container difference is3964418 logical B; index redesign is a separate mechanism|Cannot make295944702 B of pack contents disappear. File allocation tails, orphan handling, publication barriers and index bytes can offset container savings. Canonical IDs may survive a physical migration, but transactional/read/cleanup design changes. Not selected merely to explain a201MB gap.|

Dictionary training is intended for many small similar samples; its content supplies repeated material and its header can supply entropy tables. The same dictionary is required for decompression. The primary documentation recommends representative samples and describes memory/sample-size costs; it provides no LayerFS saving percentage. [Pinned dictionary API and guidance](https://raw.githubusercontent.com/facebook/zstd/v1.5.7/lib/zdict.h)

A seek table provides frame-level random access by indexing independent frames. It does not restore small-object access inside one large dependent frame. LayerFS already has a group directory, so importing a standard seek table is not itself a new compression mechanism. [Pinned seekable format](https://raw.githubusercontent.com/facebook/zstd/v1.5.7/contrib/seekable_format/zstd_seekable_compression_format.md)

SQLite's official internal/external-BLOB measurements vary with BLOB size, page size and platform; they are historical experiments, not evidence that external packs are faster on this host. [SQLite BLOB comparison](https://www.sqlite.org/intern-v-extern-blob.html) Moving bytes outside the database also moves part of publication outside SQLite's database transaction. A new design must publish durable complete packs before references, preserve necessary reader generations and recover orphan/partial packs; SQLite's atomic-commit contract is not an automatic transaction over arbitrary external files. [SQLite atomic commit](https://www.sqlite.org/atomiccommit.html)

## Decoding cost must be part of the storage design

Existing performance-phase counters report869798 decompressions,880813 group fetches,12450972177 decoded-read bytes and7309561533 encoded-read bytes. Decoded work is about17.40 times the final715758049 decoded group bytes. These are reader-wave counters, not unique disk traffic, and cannot identify precise cache misses or the cause of each repeat.

Source `read.rs::visit_locations` groups requests by physical location, but bounded drains call `visit_wave` separately. A group repeated in another drain is decoded again. Current frame granularity is therefore a deliberate read-amplification control. Bigger frames could combine requests and reduce some calls, or repeatedly decode far more unrelated bytes. Multiplying today's call count by a proposed8MiB cap is not a prediction because grouping/cache behavior changes; ignoring this work is equally unjustified.

For a bounded whole-file prefix chain, retain the sibling's explicit whole-closure decoded-byte bound, including the target, rather than only a depth number. Prefix lifetimes require a live authenticated base while reconstructing its child. Every demanded content identity remains verified after reconstruction. A bounded admitted-content cache may reuse hot parents, but its memory and cache policy are part of the treatment. A dictionary/prefix dependency needs an authenticated persistent identity, not merely the codec's short dictionary ID. New readers must reject missing/wrong bases, cycles, oversized windows/output and invalid frames.

Larger frames across historical checkpoints introduce another cost: a synchronous writer must close and persist what it acknowledges. It cannot leave data in an unfinished frame until later checkpoints merely to obtain an attractive final ratio. Rewriting a partial frame or repacking history before acknowledgement incurs real read/encode/write/temporary-allocation cost. An offline final archive may demonstrate potential; it does not establish that per-Commit retained allocation or timing has the same behavior.

## One credible aggressive screen, with a falsifiable result

Select the sibling's **bounded whole-file canonical content plus actual-prior native-prefix physical representation**, with one fixed codec profile and declared depth/whole-closure cap. The FULL-only control for that same canonical unit uses the same codec settings; this separates native history reuse from stronger-level effects. Larger files retain the declared segmented fallback. The256KiB proposed bound must first be checked against the shared metadata/oracle size distribution, not assumed to cover all byte-bearing files.

Before retained-data encoding, freeze with the root owner: all157 workload/retention identities; the precise canonical field layout; chronological candidate/base admissibility; one fixed codec level/window; FULL/PREFIX framing and checksum/base identifiers; dependency limits; index/container allowance; and the permitted offline timing/resource envelope. Reuse one authenticated payload pipeline across controls. Do not train on future data, reorder future states into an online claim, omit an unfavorable new-file/initial-content cohort, or build only favorable patch examples.

For every complete candidate image, report actual frame/pack/dictionary/base/index/container bytes and reconstruct every selected state/ObjectId. Compare its fully costed budget with134221004 B. If it misses, retain the negative result and explain whether payload, structural representation, dependency anchors, or physical overhead consumes the budget; do not expand automatically into a parameter sweep. If it fits, the result is **offline representational potential**, followed by one separately frozen fresh-Store public-path experiment before claiming60% product allocation reduction. The original acknowledgement-time allocation remains the control.

This choice is aggressive because it changes the unit of canonical content and the available historical reference, not because it assumes unlimited codec effort. It can delete the bounded-file CDC/extent/FileState path and replace the new-format handwritten COPY/INSERT machinery with the installed codec, while preserving the old decoder or an explicit migration importer. Its compatibility and deletion boundary are detailed in `delta-history.md`. A codec-only adjustment remains a possible later simplification if measured evidence shows it reaches the same envelope, but there is currently no basis to schedule a broad sweep or to promise either10–20% or60%.

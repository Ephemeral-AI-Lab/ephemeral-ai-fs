# Issue88 empirical encoding experiments

**The experiments favor keeping existing content units and improving their physical history encoding. The134.2MB complete allocated-Store objective has not been demonstrated.** Whole-file conversion adds only2,481,574 bytes of framed-content improvement beyond existing-chunk native prefix encoding, while changing canonical units and increasing the raw unit population by130,552,367 bytes. Structural-origin deltas have a measurable21,935,435-byte offline benefit, but their public-path costs and allocation must be judged separately.

This report follows dedicated issue88, linked to #87/#18 and explorationeb7050603. The owner authorized empirical retained-content encoding and isolated prototypes. Original Stores, source inputs and previous evidence remain protected. Experiments were run sequentially under the existing measurement lock; no benchmark/build contention, external dependency edits, original-Store conversion, merge or rollout occurred.

## Prospective controls and identities

The contract was committed before retained-content encoding at2b7eb7541, with exact framing and per-pack details atded1d28d2. S1 tool build failed first on unsupported SQLite usize conversions; checked conversions produced sourcebb36062bf and binaryfa4323d2bb45889d78c89f2acfaf11605b083b9d32da3948d43bd9bdf49bc484. The failure is retained. S1 observes current tree-engine origins reconstructed from actual retained bindings; it does not pretend original producers logged those origins.

S2/S3 use exact existing FastCDC/canonical chunk code through extractor binary1a8cd26bcecc8fdc51ac64a0de935157129f91543b81d71e60b3ff68390e46d0. All75,922 regular-file source blobs were authenticated against frozen input seals and Git blob identities. The first extraction was superseded before encoding after independent review found a cross-threshold prior-selection bug; corrected input2 preserves S2 byte-identically and enforces S3 small→large FULL fallback. Both extractions remain.

Native screens use Zstandard1.5.7, level3, windowLog20, one thread, content size/checksum enabled, no dictionary ID, one actual selected prior from a strictly earlier checkpoint, depth≤4 edges and cumulative decoded payload≤1,048,576 bytes including target and all bases. Prefix wins only when its frame plus32-byte base ID beats FULL. No future/same-checkpoint base, alternate-base search, shadow FULL copy or parameter sweep. S3 changes only the declared unit/fallback construction under the same codec/base rules.

Each experimental file has16-byte header and56-byte per-record header, plus32 bytes per selected prefix. Raw content is encoded; canonical framing is deterministically reconstructed and authenticated separately. S2 preserves canonical chunk IDs from sealed extraction; S3 uses the frozen experimental content-only canonical domain. Full/candidate images share a diagnostic SQLite index whose bytes are recorded, not misrepresented as a production index for either arm.

## Offline results

| Screen | Population | Control bytes | Candidate bytes | Difference |
|---|---|---:|---:|---:|
|S1 actual inode-leaf origin, current matcher/codec|9,918 identical structural groups /279,724 objects|78,792,537|56,857,102|21,935,435|
|S2 native prior prefix on existing chunks|86,412 units /703,348,161 raw bytes|243,582,540|102,306,097|141,276,443|
|S3 bounded whole-file/hybrid, same prefix policy|77,487 units /833,900,528 raw bytes|267,114,049|99,824,523|167,289,526|

S1 values are encoded group bodies with identical original grouping, record ordering and canonical identities; complete pack/SQLite allocation is not in that subtraction. S2/S3 values include complete experimental container and record/base framing, but exclude regenerated LayerFS structures and a production locator/index layout. The joint analysis indexes are28,934,144 bytes forS2 and25,694,208 forS3; these contain both arms' bookkeeping and cannot be used as final product-index forecasts.

Every S1 FULL control matches the original encoded bytes by length and BLAKE3, every one of157 inode roots rebuilt identically, and all279,724 selected records reconstruct to the same canonical IDs. There are4,585 selected structural DELTAs. Of8,771 origins observed,4,591 were eligible under candidate-FULL-only depth-one policy;4,180 origins referred to a candidate DELTA and therefore fell back to FULL. No matcher-budget skips occurred. Origins absent from splits/merges remain FULL; no approximate predecessor was substituted.

S2 selected58,298 prefixes; S3 selected52,206. S2's8,702 depth-limit fallbacks and S3's7,644 depth-limit plus9 closure-limit fallbacks remain in the images. All target and dependency reconstruction checks pass, with maximum observed closures163,840 bytes forS2 and1,027,939 forS3. No required base is omitted or charged as reclaimable because it is shared.

**Interpretation:** both native-prefix screens show substantial encoded potential. However, the additional S3 benefit is only2,481,574 bytes (about2.43% ofS2's selected container). The unencoded population grows by130,552,367 bytes. This does not account for possible wrapper/index savings, but neither does it establish that those savings justify a canonical migration. The evidence revises the earlier architecture hypothesis: prioritize native prefix encoding on existing canonical chunks; defer whole-file conversion.

## Offline CPU, memory and read costs

S1 took59.14s wall,10.54s user CPU and47.26s system CPU, with143,818,752-byte peak RSS. Extraction was3.284s, actual tree reconstruction1.658s, and paired encoding/validation/output54.010s. This process includes observer SQLite writes and both alternatives; it is not public Commit latency.

S2 took36.357s wall,28.185s user CPU,6.982s system CPU and53,067,776-byte peak RSS. FULL encoding cost4.739s, prefix trials1.632s and candidate-base reconstruction6.476s; verification is separate nested work. S3 took35.078s wall,27.847s user CPU,6.306s system CPU and55,443,456-byte peak RSS; FULL encoding4.882s, prefix1.533s and base reconstruction6.175s. These sequential single screens are descriptive, not statistical speedup pairs. S3's measured codec encoder context reached1,303,576 bytes versus541,720 forS2; that exceeds the old1MiB static context assumption and must be budgeted in any implementation.

The fixed matched read follow-up uses first, largest and largest-closure units, fresh codec contexts and no decoded-object cache, three alternating pairs, OS cache uncontrolled. Representative median small-range costs:

| Screen / representative | FULL control ns | Selected ns |
|---|---:|---:|
|S2 first|26,208|51,375|
|S2 largest|44,833|54,500|
|S2 largest closure|68,000|340,125|
|S3 first|22,708|46,500|
|S3 largest|470,500|541,208|
|S3 largest closure|384,209|1,058,625|

These show visible dependency costs, not a latency distribution or OS-cold I/O. FULL and small-range results, all raw repeats, returned/decoded/encoded bytes and depths are retained inread-costs.json. A warmed Python one-object slice is explicitly a cache witness, not public API timing. A decoded-byte cap is not evidence of acceptable latency.

The deterministic preflight exercised localized64-byte edits, cross-file exact identity, return-to-A, and262143→262145→262143 threshold transitions using actual product CDC for fallback. All raw/canonical round trips passed. Wrong bases, truncated/concatenated frames, malformed container headers and future bases were rejected, with expected failures preserved. No favorable seed or parameter was selected after results.

## Public S1 prototype and smoke results

The isolated prototype reuses the tree editor's origin and existing hint fields. It accepts exact inode-table-leaf targets and same-role selected FULL origins; structural DELTA origins fall back FULL without anchor retargeting. Existing chunk policy, canonical format, spill row sizes and public batch budgets remain. Eligibility/trial counters now include inode leaves as well as chunks and must not be compared as identical payload populations.

The integrated test covers actual origin delivery, memory/spill, a first DELTA, second DELTA-origin FULL fallback, role validation and canonical readback. A pre-existing private-field access error in unrelated tests initially prevented compilation; two cfg(test)-only accessors fixed that without changing production behavior. Both failures and new candidate identities are retained.

Baseline uses authenticated accepted host/image source seals from the clean discovery checkout; candidate host SHA25689fc4cb9c87b4d4eb774f1747d1a6c5f3ea1c03fb0571864c6f9bdb5477f4869, product seal1649675c68a853f8a9da7debd36e30ba7cacd883c016e842dcfe99ba34f1e91f, image sha256:d95da176d7deca0d7ab9f5d7cd05fe983baf922ed00c25dd033e6e5bbb44cdbd. Host producer commit3afe76243 and image reporting commite33237627 differ, but measured source/product content seals match. All harness/input/import/timer definitions remain unchanged.

All three approved smokes, including four frequent-edit histories, passed independent historical verification and cleanup:33 mappings per arm. Single fresh baseline/candidate pairs are development evidence, not three-pair final qualification.

| Case | Baseline final allocated B | Candidate final allocated B | Baseline mutation+Commit ns | Candidate ns |
|---|---:|---:|---:|---:|
|DeepSeek-five|3,153,920|3,162,112|1,398,389,833|1,277,850,917|
|SDK binary8MiB|8,630,272|8,626,176|31,847,668|38,226,958|
|FUSE binary8MiB|9,650,176|9,646,080|207,486,042|236,606,540|
|FUSE text32KiB|69,632|69,632|40,463,376|49,133,208|
|SDK text32KiB|77,824|135,168|26,591,584|39,913,794|
|Small files|208,896|204,800|20,749,209|25,266,709|

The SDK text return-to-A step grew6.728→13.231ms. Its logical database remains77,824 bytes in both arms, with46 objects/62,333 canonical bytes; the57,344-byte allocation difference is a signed filesystem residual, not57KB extra encoded content. The candidate attempted one extra structural delta that the mixed group rejected. Extra matching work is observed, but SDK editing itself also slowed before Commit, so the entire elapsed difference cannot be attributed to that extra delta attempt. The negative results are retained; the original numerical smoke gates are not relabelled PASS.

The prospective continuation decision records why one full157 applicability run is necessary despite mixed tiny-case results: the measured structural opportunity concerns larger historical inode pages. It does not waive regression evidence or qualify the candidate for release.

## Public full157 result and final pre-verification census

The public S1 run completed all157 performance checkpoints, all157 historical
verification mappings and normal cleanup. Source index/SHA/tree/input and oracle
seals match the historical accepted workload; each verifier identity matches its
own run's LayerFS mapping. Both runs verify4,936,693,030 bytes across history.
The original final802-entry manifest, new324-entry performance manifest and
new802-entry verification manifest all reauthenticate. The performance Store
entry is checked against the separately preserved pre-verification logical copy,
whose SHA256 matches the original acknowledgement-time Store; the verifier's
later Store is checked against its own final manifest. No allocation is borrowed
from the copy.

| Nested account / quantity | Bytes |
|---|---:|
|Original acknowledgement Store allocation|335,552,512|
|S1 acknowledgement Store allocation|302,006,272|
|S1 logical SQLite pages (72,472 ×4,096)|296,845,312|
|S1 sidecar allocation|0|
|Signed allocation adjustment|5,160,960|
|object_packs B-tree pages, including overflow|277,884,928|
|Selected objects index B-tree pages|18,874,368|
|Other SQLite pages|86,016|
|Pack BLOB bytes, nested inside object_packs|273,908,421|
|Pack headers +directories|703,520|
|Payload group encoded bytes|216,448,341|
|Structural group encoded bytes|56,756,560|

SQLite payload/unused/overhead sum to the logical page length. Freelist is0;
object_packs SQL row encoding is24,347 bytes beyond actual BLOB payload. All page
unused bytes total5,648,035 and page/cell/pointer overhead1,519,716; neither is
presented as automatically recoverable. Exact per-table/page types are retained
infull157-census.json. Pack conservation is273,908,421 =78,608 headers +624,912
directories +273,204,901 encoded groups. Decoded conservation is694,540,462
=1,621,400 group count/directory bytes +666,524,298 FULL-record bytes
+26,394,764 DELTA-record bytes. These equations are nested, not additive savings.

Payload encoded bytes are exactly unchanged from the original216,448,341.
Structural encoded bytes decrease78,792,537→56,756,560, a22,035,977-byte
public-path reduction; pack framing decreases304 bytes. This is consistent with
S1's intended mechanism and more specific than the allocation delta alone.
Allocation decreases33,546,240 bytes, but logical SQLite decreases21,987,328;
the remaining11,558,912 difference is the changed signed allocation adjustment,
whose cause is not established. Do not call the entire allocation delta encoding.

The reused exact canonical/record decoder authenticated366,293 selected objects
with799,576,457 canonical bytes across4,913 packs. The role table contains4,674
inode-leaf DELTAs and8,024 payload DELTAs; all other selected roles remain FULL.
Every selected object lies in the union of layers, commits and retained workspace
stages. Branch roots derive from those layer/commit roots under the schema, so
there is no omitted independent branch-root field. All selected DELTA bases are
selected FULL records. There are zero base-only objects, zero selected objects
outside the required union, and zero physical records without selected locators.
The new canonical population differs from the historical run by152 objects and
51,168 bytes; fresh internal construction is not identical merely because all
filesystem oracles match. The complete role table and dependency counts are kept
infull157-census.json rather than duplicating the large inventory.

New public Exec238,364,965,586ns +Commit40,761,892,669ns =279,126,858,255ns.
Historical public sum305,991,448,835ns is descriptive, not a fresh paired speedup.
New performance case wall401,869,095,000ns and work400,419,705,208ns are separate
from enclosing invocation462,028,952,333ns and preparation59,737,252,916ns.
Verification invocation447,574,048,667ns includes42,306,572,959ns preparation;
its work is not performance latency. Per-checkpoint counters exclude Init and
include both payload and structural eligible targets on S1. Do not combine those
with old payload-only denominators.

The pre-verification snapshot/decoder custody and independent page/role/root
review now close the earlier report placeholder. The owner-selected combined
159,163,199-byte milestone is a subsequent prospective investigation; this full
S1 result does not measure native payload integration, qualify release or erase
smoke regressions.


## Disposition

- **S1 structural-origin deltas:** encoded hypothesis confirmed on fixed retained groups; canonical-preserving public prototype passes correctness. Public cost/placement evidence must govern adoption, not the offline21.9MB alone.
- **S2 existing-unit native prior-prefix:** retain as the next production investigation direction. It achieves most of the content reduction without a canonical migration. A real implementation must preserve chronology, authentication and explicit read/memory budgets; this report does not claim its offline image is a product Store.
- **S3 bounded whole-file rewrite:** defer/reject as the next architecture commitment under this fixed screen. Only2.48MB additional framed-content gain was measured; wrapper/index effects remain unmeasured and cannot be invented to reach134MB.

**Exactly one next production direction:** canonical-preserving native prior-version/prefix encoding on existing content units, with explicit bounded dependency costs and coverage. Structural-origin findings remain separate; no measured combined S1+S2 image exists, and adding their differences is not a134MB result. The70MB payload target is not met by either offline content screen. The aggressive complete allocated-Store goal remains open.

Original evidence and failed preparations are preserved. No existing-Store migration, silent conversion, dependency-source edit, benchmark population change, seed selection, rollout, M5, S3/cloud or merge occurred. The experiment branch and reports are for review, not release qualification.

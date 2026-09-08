# Candidate physical review against the fresh C+S1 control

**The candidate's acknowledged complete Store is184598528 bytes,33517568 bytes smaller than the fresh218116096-byte control.** The improvement is real in both pack content and logical SQLite size; it is not solely an allocation artifact. Accounting and158-row cohort validation report PASS, including86412 file objects /705162813 canonical bytes. The candidate's separate public historical verification is pending an external build lock, not recorded as a failed sample. Complete verification remains necessary before final acceptance.

This review reads only existing JSON/CSV reports under `issue88-SP-{control,candidate}-{accounting,validation,supplements}-1`. No Store, inventory database, hashing, census, build, encoding or new experiment was executed. All bytes below are exact integers; allocation is the original final acknowledgement and physical accounts describe the final pre-verification logical snapshot.

## Fresh matched physical difference

|Dimension|Control|Candidate|Candidate minus control|
|---|---:|---:|---:|
|Complete allocated Store|218116096|184598528|-33517568|
|Logical database|205524992|181710848|-23814144|
|Signed allocation adjustment|12591104|2887680|-9703424|
|Pack BLOBs|182782215|155353550|-27428665|
|Encoded group bodies|182078391|154975454|-27102937|
|All payload-group bodies|125298018|98256041|-27041977|
|Structural-group bodies|56780373|56719413|-60960|
|Pack headers/directories|703824|378096|-325728|
|Logical database outside BLOB content|22742777|26357298|+3614521|
|objects-index page bytes|18808832|18722816|-86016|
|Other metadata pages|86016|86016|0|
|object_packs pages minus BLOB content|3847929|7548466|+3700537|

Both have zero acknowledged sidecars, zero freelist pages and zero unexplained SQLite page residual. The candidate has44363 pages×4096 bytes. Its B-tree interior conserves170961088 payload +9290898 unused +1458862 overhead =181710848. Candidate unused bytes increase3746958 and page/cell/pointer overhead increases28915; these are not automatically removable waste. Pack SQL row encoding is38429 bytes versus24312. The larger page-space margin partly offsets the27428665-byte BLOB improvement.

`33517568 allocated improvement =23814144 logical improvement +9703424 reduction in signed allocation adjustment`. The adjustment's cause is still unattributed; do not call the last term codec savings, reclaimed preallocation or bytes beyond EOF.

Candidate8544 packs (+3627) and15087 groups (-23985) contain155353550 BLOB bytes =136704 pack headers +241392 directory +154975454 encoded groups. More packs do not mean more overall pack framing here: the much smaller group directory outweighs the added headers.

## Native frames and exact reference bridge

The candidate's file-payload lane is86412 native records in5335 RAW groups:

|Component|Bytes|
|---|---:|
|28106 FULL frames|68435250|
|FULL5-byte headers|140530|
|58306 PREFIX frames|27155798|
|PREFIX37-byte headers, including base IDs|2157322|
|Group count fields|21340|
|Record-end directories|345648|
|Native payload groups total|98255888|

Native frame bytes total95591048. Headers2297852 plus group framing366988 explain the rest. The five metadata-only chunks remain legacy FULL:141 canonical bytes and153 encoded group bytes. Thus all payload groups total98256041. Exact RAW native record components may be assigned to their records; legacy compressed structural group bytes may not be proportionally assigned to records.

The old offline component reference159163199 comprised102306097 experimental payload-container bytes plus56857102 S1 structural-group bytes. A transparent bridge is:

```
159163199
 - 4050209  payload-container-to-public-native-group difference
 -  137689  structural group difference
 +     153  previously omitted legacy metadata payload groups
 +  378096  actual complete pack headers/directories
=155353550 actual pack BLOB bytes
```

Of the4050209 payload difference, only10425 bytes come from the difference between old95601473 and current95591048 native frame totals. The remaining4039784 bytes arise from different record/container/group framing. The offline payload envelope had56-byte records plus separate32-byte PREFIX IDs and a16-byte container header; the public grammar uses5/37-byte records and actual group directories. Do not advertise the whole3809649 difference below159163199 as a codec gain or compare that encoded reference directly with184598528 allocated bytes.

At the fresh-pair level, structural group bytes differ by only60960. All non-inode-table role count/length aggregates agree; inode-table differences account for the total canonical-population difference: candidate366147 selected objects (-138) and799534513 canonical bytes (+39112). Inode-table leaves change by-142 objects/+49176 canonical bytes and branches by+4/-10064. This is not evidence of semantic loss, nor proof that object identities agree individually; that belongs to canonical and public verification. S1 has4583 selected leaf DELTAs versus4647 in control. Shared preparation/grouping effects and generated inode-table identities remain qualifications rather than a claim that the structural arm is byte-identical.

## Coverage, FULL residue and dependency shape

All366147 selected candidate objects are in L=R. Base-only objects, selected objects outside R and unselected physical records are all zero. There is no demonstrated garbage or physical-base-only deletion opportunity.

Native FULL targets partition exactly:

|Reason|Targets|Canonical target bytes|
|---|---:|---:|
|No delivered hint|19377|106954938|
|Depth limit|8704|88802966|
|Complete PREFIX loses to FULL|25|183974|
|Total native FULL|28106|195941878|

No-hint further comprises18584 no-predecessor targets /101492055 canonical bytes and793 completed no-overlap targets /5462883 bytes. Missing required spans and limited-empty outcomes are zero. Native budget, unavailable, unsupported legacy DELTA and wrong-role fallback counts are zero. The full157 Empty-Init/import route therefore does not exhibit the earlier Directory-Init native-provenance limitation in this validated file cohort.

The68435250-byte native FULL-frame bucket cannot be apportioned among these reasons from aggregate canonical-byte counters. In particular88802966 depth-limited canonical bytes are not88802966 missed compressed bytes. More depth is not justified automatically. Prepared FULL alternatives total238743452 encoded frame bytes for all86412 targets; those are trial alternatives, not another durable Store component.58331 complete PREFIX frames include25 FULL losses. Durable PREFIX frames total27155798, while completed trial PREFIX frames total27203295:47497 bytes belong to the rejected25 alternatives.

Native selected depth distribution:

|PREFIX edges|Records|Canonical target bytes|Frame bytes|Maximum raw closure bytes|
|---|---:|---:|---:|---:|
|0 FULL|28106|195941878|68435250|32768|
|1|19413|155243143|8548780|65536|
|2|15518|134380364|7228932|98304|
|3|12852|116689257|6046612|131072|
|4|10523|102908171|5331474|163840|

All native dependencies cross packs/groups.19413 PREFIX records point directly to native FULL bases;38893 point to native PREFIX bases. Distinct immediate bases are16683 FULL and32499 PREFIX. These are shared logically retained objects, not additional physical copies. Structural4583 DELTAs retain4579 FULL inode-leaf bases. The existing fixed SDK public-read probe observed depth0 native FULL edits only, so it does not qualify this58k-PREFIX distribution or its10523 depth4 targets.

## Ranked residual hypotheses, with ceilings rather than forecasts

1. **Native FULL representation and dependency policy remain the largest payload bucket:68435250 frame bytes, plus140530 record-header bytes.** Most no-hint canonical bytes have no predecessor; a hint-handoff explanation is contradicted by the measured zero missing-span/limited-empty counters. Depth policy affects8704 targets /88802966 canonical bytes, but their durable frame-byte subtotal and potential savings are unavailable. Initial/new content, raw-prefix utility and depth policy are distinct explanations. Zero budget skips do not establish that deeper chains would be cheap.
2. **Structural representation remains56719413 encoded group bytes.** Its canonical objects include large inode-map histories, and S1 already applies leaf deltas. Mixed structural groups dominate, so neither a FULL-leaf canonical total nor decoded DELTA savings supplies a per-record compressed ceiling. A canonical redesign would affect compatibility, construction and reads and is not justified merely by this residual's size. Base-only reclamation is contradicted by zero base-only objects.
3. **SQLite placement/indexing encloses26357298 logical bytes beyond pack BLOB content, plus a separate2887680 signed allocation adjustment.** The logical bucket contains the required18722816-byte selected index,86016 metadata pages and7548466 bytes within pack B-tree pages beyond BLOB content. Only part could ever be removable; the amount is unknown. Increased unused page bytes support a placement-efficiency hypothesis, but not a VACUUM/repack saving forecast or a reason to change the successful frozen treatment.

The candidate is below the owner's200MB/around190MB planning ranges, but those were not acceptance gates. It remains50377524 allocated bytes above the134221004 stretch reference. No requirement to chase that difference follows from this result.

## Exactly one potential next experiment

**After completing the pending correctness verification, a prospective matched public-read qualification on actual full157 native dependencies is better justified than another storage optimization.** Hypothesis: the frozen P representation's depth1–4 closures preserve correct public reads at an acceptable absolute foreground cost, which the depth0-only SDK probe cannot establish.

One variable: representation arm, frozen C+S1 versus frozen C+S1+P; no codec, depth, delivery, batching or memory changes. Before timing, select one retained file range for each observed depth0–4 by a fixed rule—earliest full157 checkpoint containing that depth, then lexicographically first file/offset—and freeze mappings, ranges, expected bytes/oracles and source/binary/environment identities. Use existing retained logical snapshots through disposable copies with separate custody; preserve original Stores and allocation evidence. Fix paired order/repetition count prospectively, fresh application/FUSE contexts, and disclose uncontrolled OS cache state. Measure synchronous public range-read elapsed, CPU/RSS, requested/parser/decoded bytes, fetch/decode calls, actual depth/closure, and correctness/cleanup; do not substitute codec microtime for public latency.

Stop on identity, output/oracle, dependency/resource or cleanup failure. Retain unfavorable latency as a valid result; do not move the range or deepen the policy after observation. This diagnostic could support or reject the current implementation's read-cost suitability. It promises no storage savings and does not authorize a replay, new workload family or optimization under this review.

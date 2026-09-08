# C+S1 control physical review

**The fresh control preserves C's exact aggregate payload encoding and realizes approximately21.96MB less structural group encoding.** It is a coherent C+S1 baseline for the pending P comparison; it is not itself the candidate outcome or a new optimization experiment.

Scope: read-only review of `issue88-SP-control-accounting-1` JSON/CSV, `issue88-SP-control-validation-1/validation.json`, historical `issue88-C-custody-1/C-accounting.json`, and analysis source. No Store, inventory database, SQL query, census, build or test was opened/run. Accounting and158-row graph validation report PASS; the latter identifies86412 unique file targets /705162813 canonical bytes. This review does not independently certify completion of the separately running public historical verifier. Candidate results remain pending.

## Exact nested final account

All figures are integer bytes unless labeled counts. Allocation is the original final acknowledgement; pages and physical encoding are the authenticated final pre-verification logical snapshot.

|Dimension|C+S1 control|
|---|---:|
|Store/database allocated|218116096|
|Logical database|205524992|
|Signed allocation adjustment|12591104|
|Allocated/logical sidecars|0 /0|
|Pack BLOBs|182782215|
|object_packs page allocation|186630144|
|Selected objects-index page allocation|18808832|
|Other metadata pages|86016|
|Pack headers/directories|78672 /625152|
|Encoded group bodies|182078391|
|Payload-only encoded group bodies|125298018|
|Structural-only encoded group bodies|56780373|

`205524992 = 186630144 +18808832 +86016`. The object_packs page allocation exceeds its BLOB content by3847929 bytes; this includes SQLite row/page space and is not a reclaimability estimate. Pack SQL row encoding alone is24312 bytes. All50177 pages are4096 bytes; freelist and unexplained page residual are zero. B-tree bytes conserve198551105 payload +5543940 unused +1429947 overhead. Overflow is included in table totals, not added again. The positive12591104 allocation adjustment remains unattributed.

There are4917 packs and39072 groups. Wire conservation is `182782215 =78672 +625152 +182078391`. Decoded groups total380717119, comprising156288 count framing +1465140 record directory +205155588 legacy FULL records +173940103 legacy DELTA records. The DELTA interior is2814076 headers +22305933 COPY +11637240 INSERT framing +137182854 literals. These decoded dimensions are not additional disk bytes. Native components are correctly zero in this control.

## Historical C comparison and S1 interpretation

|Control minus historical C|Bytes|
|---|---:|
|Original allocated Store|-16781312|
|Logical database|-21934080|
|Signed allocation adjustment|+5152768|
|Pack BLOBs|-21963287|
|Encoded payload groups|0|
|Encoded structural groups|-21963415|
|Pack header/directory change|+128|
|Logical bytes outside pack BLOBs|+29207|

The historical C payload aggregate125298018 is preserved exactly, including63989 payload DELTAs representing567996834 canonical bytes and168469732 decoded physical record bytes. Thus this run provides measured aggregate evidence that adding S1 did not change that payload bucket; do not extrapolate this observation into a general absence of shared-budget interactions.

The structural group reduction21963415 closely contextualizes the earlier offline S1 reduction21935435, but they are different populations/experiments. Current structural56780373 is not the offline56857102 forecast becoming an identical result. Current canonical totals differ from historical C by-5412 bytes and+5 objects, so these are not byte-identical structural populations. Allocation improves less than logical/BLOB encoding because the signed allocation adjustment increases5152768; calling the whole allocated difference codec savings would be wrong. Historical C remains a qualified historical control, not a fresh timing pair.

S1 now selects4647 inode-table-leaf DELTAs for26640628 canonical bytes, stored as5470371 decoded record bytes, while5387 inode leaves /30100964 canonical bytes remain FULL. Total inode-leaf canonical bytes56741592 compare with historical C56747708 across10029 FULL leaves. The decoded per-record difference is not compressed per-record savings: nearly all structural compressed space belongs to mixed structural-role groups. `mixed_roles` in this report does **not** mean mixed payload/structure; the exact composition CSV shows no payload record in those groups. The structural lane contains9917 groups; payload contains29155.

Selected canonical roles total366285 objects /799495401 bytes. Payload contributes86417 objects /705162954 canonical bytes:86412 file-only chunks /705162813 bytes plus five metadata-only chunks /141 bytes. There is no file/metadata overlap reported here. The remaining279868 structural objects contribute94332447 canonical bytes. This is separate from the physical56.78MB structural group account.

## Retention and dependencies

All366285 selected objects belong to L and R. Physical-base-only, selected-outside-R and unselected-record counts are zero. Therefore no residual/base-only deletion bucket explains the remaining allocation. This does not mean anchors have no encoding cost; every required anchor is also logically retained.

All4647 inode-leaf DELTAs use4642 selected FULL inode-leaf bases, all cross-pack/group. Of those bases4637 have one selected reference and five have two. Payload63989 DELTAs use12630 FULL payload bases, likewise cross-pack/group. Payload fan-in bins contain3409 bases with one reference,4572 with2–4,4101 with5–16,544 with17–64 and four with65–256. Canonical base totals are not compressed reclaimable bytes, and unique-base counts are not additive across arbitrary overlapping summaries.

The complete control remains218116096 allocated bytes. Neither the200MB/190MB planning ranges nor159MB component reference/134MB stretch establish an acceptance threshold. The pending candidate must be compared against this fresh control with its own complete allocation, timing, correctness and physical accounts. No target-chasing change follows from this control result.

## Existing-inventory supplements still needed

Source inspection confirms the current compact accounting omits a fixed role-size histogram, an explicit root-source count table and a native-depth distribution. The native-aware census already stores every required metadata column; **no census or decoding rerun is needed**. The following are suggested bounded queries for the parent to run only after the candidate reaches its allowed analysis slot. They have not been executed by this review. Output metadata should label integer counts/bytes, final pre-verification snapshot, existing inventory hash and derived status; empty bins are observed zero only after the required tables are present.

Fixed canonical-size histogram, unique selected objects; reuse these bins for both arms:

```sql
WITH bins(bin,lo,hi) AS (
 VALUES ('0..63',0,63),('64..255',64,255),('256..1023',256,1023),
 ('1024..4095',1024,4095),('4096..16383',4096,16383),
 ('16384..65535',16384,65535),('65536+',65536,NULL)
), combos AS (SELECT DISTINCT role,kind FROM records WHERE selected=1),
 observed AS (
 SELECT r.role,r.kind,b.bin,count(*) n,sum(r.bytes) canonical_bytes
 FROM records r JOIN bins b ON r.bytes>=b.lo AND (b.hi IS NULL OR r.bytes<=b.hi)
 WHERE r.selected=1 GROUP BY r.role,r.kind,b.bin
)
SELECT c.role,c.kind,b.bin,b.lo,b.hi,
 coalesce(o.n,0) objects_count,coalesce(o.canonical_bytes,0) canonical_bytes
FROM combos c CROSS JOIN bins b LEFT JOIN observed o
 ON o.role=c.role AND o.kind=c.kind AND o.bin=b.bin ORDER BY c.role,c.kind,b.lo;
```

Explicit collected root types, including zero staging roots. A root shared across source types is not counted twice in logical union L:

```sql
WITH types(source) AS (VALUES ('layers'),('commits'),('workspace_stages'))
SELECT t.source,count(r.id) root_rows_count,count(DISTINCT r.id) unique_roots_count
FROM types t LEFT JOIN roots r ON r.source=t.source GROUP BY t.source;
```

Native physical depth—not public read-call depth. Keep selected and unselected records distinct and retain native FULL/PREFIX kind; the control is expected to have no observed rows:

```sql
SELECT r.selected,r.kind,n.prefix_edges,count(*) physical_records_count,
 sum(r.bytes) canonical_bytes,sum(n.raw_bytes) target_raw_bytes,
 sum(n.header_bytes) header_bytes,sum(n.frame_bytes) frame_bytes,
 max(n.closure_raw_bytes) maximum_closure_raw_bytes
FROM native_records n JOIN records r USING(pack,grp,rec)
GROUP BY r.selected,r.kind,n.prefix_edges ORDER BY r.selected,r.kind,n.prefix_edges;
```

A fixed0..4 depth display can fill verified empty bins with zero counts/byte sums; the maximum closure of an empty bin is null, not zero. Missing tables or failed queries are invalid/unknown, not an empty native population.

Evidence JSON SHA256s:

- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-control-accounting-1/accounting-reconciliation.json`: `c3da2921072fdc95da477aa088a814a8611bb5944d328e648ae40ce1cbaec460`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-control-validation-1/validation.json`: `7917516ee58356a5471a886d32077e6a4b775ca249b803b8e79f2b2db9c5caf7`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-C-custody-1/C-accounting.json`: `cce8a16fff1effe8e27a20f7f63041642c306353696e56bbfa6358412e9f14e1`.

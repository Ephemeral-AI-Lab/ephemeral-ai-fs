# Candidate coverage and remaining payload review

TARGET IDENTIFIED, conditional on pending normal all157 historical verification and cleanup. The completed performance, native census and158-checkpoint graph/admission validator support retaining P as the research implementation. They do not replace normal historical verification, release/fault qualification or repeatability. This review inspected source and already generated JSON/CSV only; no Store/inventory was opened and no hashes, census, build or encoding were run.

Sources: runs/issue88-SP-candidate-validation-1/{validation.json,checkpoint-trajectory.csv}; candidate-accounting-1/{accounting-reconciliation.json,group-accounts.csv,object-roles.csv,payload-use.csv,retention.csv,dependencies.csv,dependency-fan-in.csv}; candidate-supplements-1/native-depth.csv. Comparisons use matching control-accounting-1 and parent-supplied public elapsed totals. Existing reports bind the immutable final pre-verification snapshot and actual decoder proof. This review relies on those authenticated upstream artifacts, not a fresh independent checksum claim.

## Exact cohort and fallback conservation

The unique authenticated initially missing retained file-payload cohort is86412 objects /705162813 canonical bytes (raw payload plus21-byte canonical framing). There are no admission races and no unlocated physical records. Every native physical record is selected and in the retained logical union. Exclusive final preparation outcomes reconcile as follows:

|Outcome|Targets count|Canonical bytes|
|---|---:|---:|
|No delivered hint|19377|106954938|
|Depth/closure-cap fallback|8704|88802966|
|Native FULL wins completed comparison|25|183974|
|Native PREFIX admitted|58306|509220935|
|Unavailable / legacy DELTA / wrong role / budget fallback|0|0|
|Total|86412|705162813|

FULL winners =19377+8704+25=28106 targets /195941878 canonical bytes. PREFIX winners =58306/509220935. No-hint is further explained by authenticated D observations:18584 no-predecessor targets /101492055B, and793 completed-empty correspondence targets /5462883B. Missing required span, limited-empty correspondence, optional budget and unavailable-base populations are zero. Thus the old operation-correspondence bottleneck is not a remaining cause in this run.

The depth counter combines two source conditions, but the fixed raw maximum32768B and five-record maximum imply existing closure<=163840B and proposed child<=196608B, below1MiB. With source constants and authenticated depth table, the88802966B fallback is attributable to `depth>=4`, not closure-byte exhaustion (source-supported inference with a strict bound). Source: objects/admission.rs307–318 checks Available prior depth/closure before a prefix trial. These targets have a selected, authenticated prior; no prefix trial is attempted because a child would exceed the frozen edge limit.

Attempt and selected populations remain distinct even without races. There are86412 FULL encodes (238743452 frame bytes),58331 PREFIX encodes (27203295 frame bytes), but58306 selected PREFIX frames (27155798B). The25 losing PREFIX trials account for47497 attempted frame bytes and183974 target canonical bytes. Do not add attempted frames to durable storage. Existing matcher events are overlapping observations and cannot be added to this exclusive table. The record-only native-FULL counterfactual is239175512B (238743452+5*86412), whereas selected native records total97888900B; their141286612B difference is a same-cohort native-record comparison, **not** measured reduction versus the legacy C+S1 Store or an allocation forecast.

## Remaining bytes are localized, not explained by delta count alone

|Nested physical payload account|Bytes|
|---|---:|
|Native FULL frames|68435250|
|Native FULL5-byte headers|140530|
|Native PREFIX frames|27155798|
|Native PREFIX37-byte headers|2157322|
|Native group count/end directories|366988|
|Native payload groups total|98255888|

FULL frames are71.5917% of native frame bytes. The25 raw comparisons lost are tiny; there is no evidence that deeper search, stronger matching, budgets or mixed-group rejection dominate this remaining bucket. Native uses strict record-size FULL/PREFIX comparison, not the old mixed-group12.5% threshold. The legacy metadata-only chunk remainder is5 objects /141 canonical bytes in153 encoded group bytes, immaterial to the tens-of-megabytes gap.

The exact68,435,250B FULL frame total is not split by fallback reason in current durable artifacts. The depth-blocked88802966 canonical bytes are45.3211% of FULL canonical bytes; that percentage must not be used as proportional measured compressed attribution. An affected native FULL-record ceiling is min(68575780B allFULLrecords,88802966B depth-target canonical plus applicable framing bounds). This is deliberately loose. It is not a credible removable forecast. The other large FULL cause is no-predecessor/no-overlap content:106954938 canonical bytes, which cannot be assumed compressible by predecessor search.

Structural legacy groups total56719413B; all legacy groups56719566B including153B metadata payload. Pack framing378096B gives total155353550B pack BLOBs. SQLite encloses those BLOBs:162902016B object_packs pages,18722816B objects-index pages,86016B other metadata pages,181710848B logical database. Original allocated184598528B includes an unattributed signed2887680B allocation adjustment. Native headers, the tiny legacy payload remainder or small framing cleanup cannot explain the remaining50,377,524B between actual allocation and the separate134221004B stretch.

All366147 selected objects /799534513 canonical bytes belong to retained graphs; physical-base-only, selected-outside-required and unselected-record counts are all0. FULL anchors are retained historical data, not garbage. Any anchor experiment must change representation while retaining every required historical state.

## Depth, dependencies and costs

Selected native PREFIX depths1/2/3/4 are19413/15518/12852/10523 records. Their frame bytes are8548780/7228932/6046612/5331474; depth4 targets represent102908171 canonical bytes but just5331474 frame bytes. Maximum authenticated closure sizes at those depths are65536/98304/131072/163840 raw bytes. All58306 PREFIX dependencies cross packs/groups and obey lower-pack chronology.19413 point to nativeFULL and38893 to nativePREFIX. Fan-in is mostly1 or2–4; one PREFIX base has24 selected references. There is no evidence of a small set of enormous fan-in roots accounting for the entire cost.

Performance native reader work:757114 record fetches/decode calls,978009318 requested bytes including outer framing,953781670 parser bytes,5083418918 raw decoded bytes,439255 dependency-edge visits,6513096956 native decode ns. These mix required public reads and optional candidate discovery; depth histogram counts completed chains, not unique objects. They are not a cold random-read benchmark, and must not be divided by target count and called a pure public per-object latency. Decoder work is already nested in public elapsed.

Native FULL encode4029032096ns and PREFIX encode1471176195ns are nested codec time. The later history101–157 contributes52215605 of88802966 depth-fallback canonical bytes and3629079069 of6513096956 decode ns. This supports the relevance of history anchoring, not a claim that depth fallback alone caused those CPU totals.

Fresh control→P observations: original allocation218116096→184598528B (-33517568B,-15.3668%); logical database205524992→181710848B (-23814144B); BLOBs182782215→155353550B (-27428665B). Exec245932122164→234598445543ns (-4.6085%); Commit79272777754→47805663491ns (-39.6947%); Exec+Commit325204899918→282404109034ns (-13.1612%). These are one fixed sequential pair, not repeatable speedup statistics; do not compare them to preparation-inclusive wall or historical controls. The reduction in signed allocation adjustment also contributes to allocated-byte differences; it is not itself proven codec savings. Earlier public read-cost outliers and depth0 smoke coverage remain limitations, even though real full157 runtime exercises deeper chains.

## Exactly ONE recommended next experiment

After this run's required verification/cleanup passes, the next optional optimization experiment should test **depth-triggered root-anchor substitution while preserving the maximum4-edge format and one trial per target**.

Falsifiable hypothesis: for a material portion of the8704 currently depth-blocked targets, the authenticated FULL root of the delivered prior's existing chain remains similar enough that one native-prefix encoding against that root beats nativeFULL and reduces complete Store allocation at acceptable synchronous/read costs. It may fail because the root is too stale; that negative result must be retained. This is a source-supported hypothesis, not proven missed savings.

One variable: only when the first delivered prior is valid but depth4, substitute that chain's already authenticated terminal nativeFULL root as the single candidate base. Control is current P (FULL fallback); candidate is that one root substitution. Keep CDC/canonical IDs, first-hint owner/order, codec/level/window, S1, C coverage, maximum4edges, all shared memory/read/trial limits, tie rule, grouping/publication and workload unchanged. Do not raise depth, search another hint, try multiple ancestors, introduce a cache, change Init provenance or sweep policies. Normal targets follow the existing path. An unavailable/over-budget root or a losing single comparison retains FULL.

Smallest implementation boundary would be native prior-read result ownership plus the existing admission depth fallback: make the bounded authenticated root available without a second Store scan and replace this one terminal decision. No legacy reader/matcher can be deleted because S1 and old-format reads remain. No schema/canonical migration; only existing native base IDs change. Stored lower-pack and canonical authentication invariants remain mandatory. Retaining the root through reconstruction may require one extra<=32789B transient canonical buffer; prove actual simultaneous capacities inside the existing owner cap, not an uncharged addition. Read amplification can decrease for substituted objects but descendant selection/history may redistribute work; measure it rather than promise it.

Prospectively bind depth-triggered target count/canonical bytes AND actual selected FULL/PREFIX header/frame bytes for that exclusive fallback cohort, trial bytes/time and final admission winners/races. This closes the current exact physical-byte denominator gap without per-path/raw-byte timed logs. Required checks: every frozen157 mapping/oracle, native chain/base/role/chronology/authentication, equivalent retained graphs and base closure, existing resource caps and cleanup. Preserve original acknowledgement Store+sidecar allocation, logical SQLite/pack decomposition, public Exec/Commit/finalization, transfer/preparation/observer/wall, host/container CPU/RSS/I/O and fixed read probes, including real selected deep chains where predeclared. Start with the existing matched smoke gate; promote only by explicit cost decision, not until a numerical target is forced. Stop on correctness/ownership/resource/cleanup failure; preserve valid losses and no savings. Offline encoding, if used as a preliminary screen, measures potential only and cannot substitute for public-path durable results.

No experiment is executed or authorized by this review. Pending current normal verification is a completion gate, not an optimization recommendation. If it passes, the implemented P result is already below the owner's200MB/around190MB guidance and below159163199B in actual complete pack BLOB bytes; no failure to reach the134221004B allocated stretch justifies automatic further tuning.

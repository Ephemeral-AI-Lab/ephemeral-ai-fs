# Combined C+S1+native payload experiment

TARGET IDENTIFIED AND IMPLEMENTED — RETAIN AS RESEARCH CHECKPOINT. Both frozen arms passed all157 historical checks, cleanup, census,158-row cohort validation, physical reconciliation and final resource reporting. No release or merge qualification is claimed.

The frozen candidate completed all157 performance checkpoints at184,598,528B of original acknowledgement allocation. The fresh C+S1 control is218,116,096B:33,517,568B less allocation (15.37%). Relative to original M4.5's335,552,512B, the storage reduction is44.99%; that historical result is not a fresh timing pair. Complete candidate pack BLOBs are155,353,550B, below the159,163,199B offline component reference. The reference was never a complete allocation requirement. Below200MB/around190MB planning guidance is met without changing the frozen treatment;134,221,004B remains an unproven stretch, not a further tuning instruction.

## Custody and correctness

Control measured producer2753453933c55ed7f93eb21619c235668a01ef4c; candidate measured producerd4f26f0d16f0f91c1f75767cf699012b8794ac01. Host SHA256s8158508de32cf1bcfbdddb7d181251ff25290491babc233d5bc62ac9e64e8b51 anded0fc0ea0a04a77651a46c1a156e313df94f3a2bbc7abec88dae497f56da5a32. Imagesbc5dacacb29c97134fefb55b3b0dbb7713dab9cc924833c48d087def4aa731f3 andf4b00c4555de1edca86f4a4d251c756355188853a8120927ffff54057e0cdcce. Frozen schedule binds exact commands, source/product/workload seals, binaries, images, contracts and reporter identities. Later documentation commits are not measured producer revisions.

P host was clean; its image build recorded a broad dirty checkout from authenticated analysis-only edits. The archived dirty patch and build-custody qualification preserve that fact; source/product seals match through strict normal identity checks. No producer identity override or fixture change was used. Ordered157 manifest03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271 remains frozen. Metadata normalization, public interface, host-owned SQLite/SDK/publication/spool and Docker daemon/FUSE/workload remain qualified exactly as in the benchmark contract.

Both arms independently verified904,143 historical entries and4,936,693,030 file bytes, with all157 resulting mappings matched and cleanup successful. Both final pre-verification logical snapshots were made after successful normal close/cleanup and lsof quiescence, before any verifier artifact existed. Control snapshot SHAaaf82576e7e0a51e1bee1004c7de2494f22ad81800b2d6bc7a648b0f5724b034; candidate2c1203974f6cbd249708e176c27990ce854fbc2587d52647c7ba9e61c5dcdc53. One shared native-aware census per arm authenticates every physical record, selected ObjectId, canonical role, reference and dependency. Its binary SHAdc7855a3149170f0a839d242bca15ab9523b17828abf9410613d30faec023718 is separately frozen. No old-format scanner was applied to native records. Both158-row graph/admission validations pass the identical86,412-object/705,162,813B unique eligible file cohort. Both SQLite quick_check and foreign_key_check pass. Copies preserve logical content, not allocation equivalence.

Performance Store seals are checked against the explicit pre-verification snapshots after normal verification adds Branch metadata; verification seals apply to originals. Snapshot/hash/census/integrity/report observer work is outside public timing and warms caches. Verification is not called cold. A separate task's image build acquired the shared lock between candidate analysis and verification; the first verifier launch was rejected before any Store open/sample. Its log is preserved. The successful launch is not a replacement timing sample. An earlier proof-helper launch likewise failed the lock before analysis; no census was repeated.

## Four nested byte accounts

All values below are integer bytes at final acknowledgement or the identified pre-verification logical snapshot. They are nested, not additive across sections.

| Filesystem / SQLite | Fresh control | Candidate |
|---|---:|---:|
| Original acknowledged Store allocation |218,116,096|184,598,528|
| Database allocation |218,116,096|184,598,528|
| Sidecar allocation |0|0|
| Logical database / page bytes |205,524,992|181,710,848|
| Signed allocation adjustment, unattributed |12,591,104|2,887,680|
| Pages at4096B |50,177|44,363|
| object_packs pages |186,630,144|162,902,016|
| objects index pages |18,808,832|18,722,816|
| Other metadata pages |86,016|86,016|
| Freelist / other / unexplained page residual |0 /0 /0|0 /0 /0|
| B-tree SQL payload |198,551,105|170,961,088|
| B-tree unused |5,543,940|9,290,898|
| B-tree page/cell/pointer overhead |1,429,947|1,458,862|
| object_packs actual BLOB bytes |182,782,215|155,353,550|
| object_packs row encoding |24,312|38,429|

Allocation =database+sidecars. Signed adjustment=allocated database−logical length. The adjustment is not attributed to preallocation, reclaimable waste or beyond-EOF bytes. Page bytes=exclusive table/index pages+freelist+identified other+residual. Payload+unused+overhead=page bytes, with overflow pages included once. Unused is not automatically recoverable.

| Pack / group component | Fresh control | Candidate |
|---|---:|---:|
| Complete pack BLOBs |182,782,215|155,353,550|
| Pack16-byte headers |78,672|136,704|
| Pack16-byte group directory entries |625,152|241,392|
| Encoded groups |182,078,391|154,975,454|
| Payload-only encoded groups |125,298,018|98,256,041|
| Structural encoded groups |56,780,373|56,719,413|
| Decoded groups |380,717,119|172,792,095|
| Group count framing |156,288|60,348|
| Record directories |1,465,140|1,464,588|
| Legacy FULL records |205,155,588|67,940,176|
| Legacy DELTA records |173,940,103|5,438,083|
| Native FULL headers / frames |0 /0|140,530 /68,435,250|
| Native PREFIX headers / frames |0 /0|2,157,322 /27,155,798|

Pack headers+directories+encoded groups=pack length. Decoded group count+directories+legacy FULL/DELTA+native headers/frames=decoded length. Legacy DELTA includes exact base/output/instruction framing, COPY, INSERT framing and literals in accounting JSON. Native FULL is5-byte header+frame; PREFIX is37-byte header including base ID+frame. Reconstructed canonical bytes are a separate logical dimension. Existing exact parsers checked versions, reserved fields, bounds, offsets, complete coverage and authentication.

Candidate native group98,255,888B comprises95,591,048B frames,2,297,852B record headers and366,988B group directories/counts. The remaining153B payload groups contain five metadata-only legacy chunks. Mixed-role compressed groups contain structural roles; no proportional compressed bytes are assigned to their records.

The reference bridge is exact:159,163,199−4,050,209(payload envelope)−137,689(structural groups)+153(metadata payload groups)+378,096(pack framing)=155,353,550B. The native frame total differs from S2's95,601,473B by only10,425B. Most of the improvement over the offline sum is real shared framing, not a new3.8MB codec gain. Public structural groups are measured independently, not frozen to an offline constant.

Fresh allocated reduction33,517,568=logical reduction23,814,144+signed-adjustment reduction9,703,424. Pack reduction27,428,665 is partly offset by3,614,521B more SQLite space around the BLOBs. Candidate complete allocation exceeds its BLOBs by29,244,978B; that includes index/metadata pages, page slack/encoding/overhead and the signed adjustment, not one removable bucket.

## Logical roles and retained state

Candidate unique selected roles; shared objects count once. FULL/PREFIX/customDELTA coverage and fixed size bins are in the accompanying CSVs.

| Exclusive role | Objects | Canonical bytes |
|---|---:|---:|
| FileState | 75,927 | 8,048,262 |
| directory_map_branch | 406 | 160,456 |
| directory_map_leaf | 14,023 | 11,240,607 |
| directory_state | 13,647 | 1,337,406 |
| extent_leaf | 75,927 | 7,337,108 |
| inode_record | 89,576 | 8,778,448 |
| inode_table_branch | 163 | 658,372 |
| inode_table_leaf | 9,892 | 56,790,768 |
| metadata_map_leaf | 4 | 572 |
| namespace_root | 158 | 19,118 |
| payload_chunk | 86,417 | 705,162,954 |
| symlink_state | 7 | 442 |

Total366,147 selected objects /799,534,513 canonical bytes. Control366,285 /799,495,401. Internal object populations differ by−138 objects/+39,112B despite matching filesystem histories; no unsupported attribution to randomness or a canonical-format change is made. The file-content cohort is identical. Exact outer framing and canonical decoders distinguish FileState/extent and user payload; role markers were never applied to raw file contents.

All selected objects in both arms are in L and R. R includes transitive physical bases, including intermediate native PREFIX records. Physical-base-only, selected-outside-R and physical-unselected counts are zero. Roots cover layers, commits and required staging roots, with source-specific rows and the distinct158-root union reported separately. No unreachable/base-only deletion bucket exists under these specified roots. Logically required FULL anchors still cost bytes; they are not garbage.

Candidate native representations:28,106 FULL;58,306 PREFIX. PREFIX depths1/2/3/4 have19,413 /15,518 /12,852 /10,523 records. Maximum actual raw dependency closure163,840B stays below1MiB. All physical dependency ordering, roles and allowed base kinds are validated. Canonical base bytes are not reclaimable compressed bytes.

## Checkpoint trajectory and costs

| Checkpoint | Control allocation | Candidate allocation | Control logical | Candidate logical |
|---|---:|---:|---:|---:|
|1|733,184|770,048|733,184|770,048|
|25|14,692,352|13,680,640|14,352,384|13,357,056|
|50|37,761,024|34,652,160|37,085,184|33,878,016|
|75|63,975,424|58,769,408|63,627,264|57,790,464|
|100|100,675,584|100,712,448|94,109,696|84,217,856|
|122|134,230,016|134,266,880|134,160,384|119,476,224|
|125|151,007,232|134,266,880|137,601,024|122,482,688|
|150|201,338,880|167,821,312|187,035,648|164,868,096|
|157|218,116,096|184,598,528|205,524,992|181,710,848|

These are descriptive checkpoints, not post-hoc timing samples. At100 and122 the candidate is logically smaller but slightly more allocated; the later allocation step is not evidence that one checkpoint alone created all savings. Acknowledgement pack-range provenance and graph validation establish first admissions/retentions; final pack ordering alone and final B-tree layout cannot reconstruct historical fragmentation.

Fresh checkpoint public Exec totals245,932,122,164→234,598,445,543ns; public Commit79,272,777,754→47,805,663,491ns. Combined325,204,899,918→282,404,109,034ns (13.16% lower observed total); Commit39.69% lower. This is one sequential pair, not a repeatable speedup. Preparation, transfer, Init/End, case/work/invocation, explicit observer durations and resource sampling are separately reported. Nested codec/matcher work is not added to public elapsed. Host runtime already includes Store/spool; do not sum overlapping scopes.

The fixed24-row public-read campaign remains part of the decision. Text-full median4,753,666→6,263,292ns (+31.76%,+1,509,626ns); binary-full58,773,416→64,999,875ns. Keep the first candidate text-range74,423,459ns versus control8,049,541ns; native decode6,917ns does not explain that outlier. These probes covered native FULL edits alongside legacy bulk Init content, not PREFIX-chain cost qualification. Directory Init's direct sink lacks reviewed FILE provenance and conservatively stays legacy; full157 Empty Init+Workspace imports use the native lane. No eligibility fix was bundled into the frozen treatment.

## Residual diagnosis and opportunities

Proven: native payload frames95,591,048B and structural groups56,719,413B dominate encoded storage. No residue or omitted physical-base deletion explains the remaining gap. SQLite objects-index18,722,816B is material but cannot alone reach134.2MB. Richer filesystem semantics do not excuse the entire128,225,280B allocation gap to historical matched Git56,373,248B; Git packing/global construction, representation policy, structural semantics, indexes and allocation differ, and the exact counterfactual share is unavailable. Git is a historical matched storage reference, not fresh timings or an interchangeable namespace/oracle.

1. Native FULL frames:68,435,250 measured bytes.8,704 targets/88,802,966 canonical bytes hit the depth cap;19,377 /106,954,938B have no hint (18,584 no predecessor;793 legitimate no-overlap),25 /183,974B complete comparisons select FULL. Other unavailable/role/legacy/budget/missing-span/race outcomes are zero in this validated cohort. Exact FULL frame bytes by fallback reason are unavailable;68.44MB is only an aggregate ceiling. A later narrowly scoped anchor-policy change could try the already delivered chain's authenticated FULL root when depth4 would otherwise force FULL. No deeper chain, extra trial or global search is justified. Stale roots may lose; no savings forecast, code-deletion claim or present implementation authorization follows.
2. Structural groups:56,719,413 measured bytes. Canonical inode-table leaves56,790,768B are a logical size, not their compressed share. A canonical simplification would require explicit semantic/ID/compatibility design and its own evidence; current S1 retains custom structural matching for live callers. Potential removable fraction, read/write/CPU costs and migration burden are unavailable. Do not add independently imagined savings to the payload ceiling.
3. SQLite/index space:objects index18,722,816B; all B-tree unused9,290,898B; signed allocation adjustment2,887,680B. These overlap differently with page/BLOB totals and are not all waste. A placement/index simplification may trade lookup locality, write amplification and compatibility; no measured removable forecast or VACUUM/repack is claimed. No backend change is recommended by this account.

Hypothesis review: small-file structural dominance is contradicted as the sole explanation (payload frames remain larger). SQLite placement is material, not dominant enough to explain the whole original gap. Missing required hints/spans and matcher/budget suppression are contradicted for this frozen validated cohort. Depth policy demonstrably creates FULL outcomes; its compressed opportunity is unavailable. FULL frames dominate the remaining payload cost, but FULL-only anchor garbage is absent. Good-raw-delta rejection is not established by native FULL-win counts or legacy overlapping A/B attempts. Residual payload being mostly genuinely new is unproven because the depth-limited cohort is large. Repeated decoding is observable and bounded; evidence that it constrains stronger encoding is unavailable without depth-stratified public costs.

## Next action and issue disposition

Exactly ONE next action is recommended, separately authorized: unchanged-format public-read qualification using actual full157 candidate depth0–4 dependencies and matched control file ranges. The measured population is58,306 PREFIX records, including10,523 depth4 records/102,908,171 canonical bytes/5,331,474 frame bytes. Existing read probes tested depth0 only. This is a read-cost diagnostic with zero proposed storage change, not a pretext for deeper chains or new encoding. The prospective contract is next-read-contract.md. It tests public range/full-read tail costs with exact selected-population checks and preserves every negative result. Anchor-root substitution remains an unselected opportunity, not a second recommended experiment.

The frozen pair's normal verification and cleanup passed, and its accounting and resource reports reconcile. The implementation milestone is complete. Retain the implementation as an isolated research checkpoint; do not merge or call it release-qualified. Keep issue88 open for explicit owner disposition and the precisely named depth-stratified public-read diagnostic; completion of this report does not close release/allocation issues. No additional encoding, replay, migration, rollout, merge, M5 or cloud work is launched by this report.

The full Store suite remains62PASS/9FAIL/1ignored. Independent accepted/D comparisons preserve nine matching failure names, distinct cleanup residues and initially opaque errors; these are not a green regression suite. Focused native/diagnostic/S1/reader/codec checks and public smoke/read correctness pass;672 disjoint codec cases are not exhaustive proof. Failures/outliers and fixture corrections remain sealed. This experiment supports a research decision, not release/fault qualification.

## Resource observations and implementation boundary

Candidate Commit host CPU falls78,462,690,590→48,298,782,291ns, while Commit host disk-write observations rise1,933,914,112→2,025,394,176B (+91,480,064B). Lower durable allocation is not lower write traffic. Peak sampled current host RSS across reported phases falls117,473,280→114,245,632B; process-lifetime high-water is reported separately. Scoped host runtime maxima218,771,456→185,253,888B already include Store/spool; spool max647,168B and container staging70,705,152B remain unchanged. Host Exec CPU rises54,031,968,422→56,935,452,885ns despite lower observed Exec elapsed; this is a tradeoff, not universal acceleration.

Preparation73,975,454,250→68,988,315,583ns is separate from enclosing invocation562,700,705,709→492,281,328,000ns and public elapsed. Resource CSVs preserve all315 performance/verification rows per arm, actual cumulative container boundaries, per-phase CPU/I/O and current/lifetime RSS scopes, plus null unrecorded quantities. Explicit snapshot/census/integrity times are observer work; validation/report observers are not mislabeled pure workload.

The final shared implementation preserves canonical IDs, CDC, namespace and COW semantics. Payload native encoding replaces custom payload delta construction for provenance-qualified new file chunks; S1 and legacy compatibility keep required custom-codec callers. There is one StoreDb, selected-index/CAS check and transactional publisher. Version2 records use static bounded Zstd contexts and a direct bounded iterative reader. No second Store, cache/service, similarity index or dependency-source patch was added. No entire legacy codec can be deleted safely: its structural and historical-format callers remain live. Required version dispatch is not presented as an independent storage engine. Physical format compatibility is additive for new readers; older readers cannot read native version2 Stores. No migration, rollout or merge was performed.

The all157 verification public Exec (full-tree read plus digest traversal, not pure read latency) rises395,368,430,006→487,045,982,584ns (+91,677,552,578ns,+23.19%). Its host CPU rises169,329,383,787→213,648,809,324ns (+26.17%), and sampled current RSS74,235,904→78,233,600B. Verification enclosing invocation532,819,404,667→618,709,495,750ns includes separate preparation and observer/cache effects; no causal cold-read or repeatability claim is made. This material read-side cost is part of retaining the research checkpoint and is the strongest reason to do the one depth-stratified public-read diagnostic before changing anchor policy.

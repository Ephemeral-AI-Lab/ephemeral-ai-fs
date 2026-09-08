# ADDITIONAL DIAGNOSTIC REQUIRED

The completed full-157 M4.5 Store retains **335,552,512 allocated bytes at final acknowledgement**, against historical matched delta-packed Git **56,373,248 bytes**: a **279,179,264-byte gap**. Post-verification pack BLOBs alone occupy **295,944,702 logical bytes**. SQLite bookkeeping and the filesystem allocation adjustment are too small to explain the whole gap. Existing counters identify missing predecessor coverage, but do not measure the affected payload bytes or why each target lacked a usable candidate. The justified next step is **one unchanged-policy, byte-weighted delta-coverage diagnostic**, separately authorized before execution. No optimization or replay was run for this analysis.

## Evidence and verification

Run: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/full157-m45-1`; analysis: sibling `storage-breakdown-014c0b9-full157-m45-1-issue87`. The original full157 contract and issue87 were read in full. A retained reporting script existed, but its expected final JSON/census did not; it was inspected rather than executed into historical paths. This report reconciles the sealed original receipts.

The initially clean discovery worktree was on `codex/storage-v3-implementation` at `014c0b9cb5d62bd50ded2052a2604d4aeab76dce`. The measured binary embeds dirty producer revision `452158572b72adb4a480740dbe13fe36e8338491`. Its retained patch is exactly the harness diff to the discovery checkpoint; product seal `5407203d4fe7adba5c4aa4c8222e825b24192a17ac3b726adce3d97d769f270f` and source seal `9406b4e21f9b21973e7675cd07931430c3bbee87e32daf0fa46af995cdf4a329` authenticate equivalent source content. The binary was not built at the later reporting checkpoint. Binary SHA256 `d3d6bdb3166f8d3211cf99b178e0226e6963291bb7ab7b60dc32cd033d2d140b` and retained image `sha256:8e609f2bd9e304fe3c36ef61b0c7771275b5247f987aeb9b79946ad34fb59ed6` match their seals.

All802 verification-manifest entries match. Of324 performance-manifest entries,323 match; only Store bytes changed during historical verification. The original pre-verification Store digest is preserved, but no corresponding logical snapshot survives. All157 source SHA/tree/order/mappings match the frozen manifest, all157 were Created, and independent comparison of all observed TSV dictionaries to oracles passes: **4,936,693,030 bytes and904,143 entries**. Both performance and verification cleanup pass and owned containers were removed. Store writer checks preceded immutable read-only access. Original Store content/stat consistency is checked around analysis.

MacOS owns SQLite, SDK/coordinator, canonical construction, admission/publication and spool; managed Docker owns daemon/FUSE/importer. Recorded limits are2 CPUs,2GiB memory/no swap,256PIDs; no data-sharing mounts. Current4KiB schema6, supported64KiB reading, wire1, level1 Zstandard, depth-one delta, canonical/CAS/CDC/COW and public acknowledgement behavior are unchanged. Existing uncontrolled OS caches remain uncontrolled. Normalized mtime is part of import semantics but is not independently checked by the historical oracle. Historical Git retains paths/content/executable bits/symlinks rather than all LayerFS metadata; its compression6/window10/depth50/threads2 packing and construction have separate costs. These are historical allocation controls, never fresh paired timing or release/M5 evidence.

## Four nested physical accounts

| Snapshot/account | Component | Exact bytes | Interpretation |
|---|---|---:|---|
| Final performance acknowledgement A | SQLite allocated |335,552,512|Primary allocation receipt|
| Final performance acknowledgement A | Sidecars allocated |0|Measured absent/zero receipt|
| Final performance acknowledgement A | SQLite logical |318,832,640|77,840 pages ×4,096|
| Final performance acknowledgement A | Signed allocation adjustment |+16,719,872|Allocated minus logical; cause unproven|
| Post-verification B | SQLite logical |318,869,504|77,849 pages ×4,096|
| Post-verification B | object_packs pages |299,909,120|Includes overflow exactly once|
| Post-verification B | objects index pages |18,837,504|Separate indexed locator allocation|
| Post-verification B | Other metadata pages |122,880|Includes schema/Branch/Commit/Layer tables|
| Post-verification B | Freelist / unexplained page residual |0 /0|Validated page partition|
| Post-verification B | Payload |311,738,751|Nested within B-tree bytes|
| Post-verification B | Unused |5,588,767|Not measured recoverable space|
| Post-verification B | Page/cell/pointer overhead |1,541,986|Derived remainder, unclamped|
| Post-verification B→C | object_packs SQL payload |295,969,063|BLOB plus SQL row encoding|
| Post-verification C | Actual pack BLOBs |295,944,702|Nested in SQL payload|
| Post-verification B→C | SQL row encoding |24,361|SQL payload minus actual BLOBs|

Equations: A_store=A_database+sidecars=335552512+0. Allocation adjustment=335552512−318832640=+16719872. B=299909120+18837504+122880+0+0=318869504, and payload311738751+unused5588767+overhead1541986=318869504. SQL payload295969063=BLOB295944702+row encoding24361. Neither SQLite unused bytes nor allocation adjustment is labelled waste or preallocation.

The current original stat is separately326,438,912 allocated bytes and318,869,504 logical bytes, adjustment+7,569,408. Verification increased logical size by36,864 bytes/nine pages. The changed stat does not replace final acknowledgement allocation or attribute a mechanism. Post-verification logical bytes outside packs total22,924,802; do not combine that number with the pre-verification adjustment into a purported exact decomposition. Runtime disk samples already include Store/spool and are not added again.

C (all4,915 packs): **295,944,702 =78,640 header bytes +625,184 group-directory bytes +295,240,878 encoded-group bytes**. The39,074 groups have validated complete offset coverage, codecs, reserved fields and bounds. No record-level compressed allocation is assigned.

D (decoded groups): **715,758,049 =156,296 count framing +1,464,564 record-directory bytes +693,285,484 FULL-record bytes +20,851,705 DELTA-record bytes**. FULL=358,117 kind bytes +692,927,367 canonical bytes. DELTA=328,984 header/base/output/instruction-count bytes +3,676,626 COPY instruction bytes +1,769,685 INSERT framing bytes +15,076,410 literal bytes. Reconstructed DELTA targets total106,597,922 canonical bytes, a logical dimension that is not added to D. All nested residuals are zero. See packs.csv, groups.csv and accounting-reconciliation.json for integer operands and source statuses.

## Authenticated logical roles and retention

Each selected ObjectId is authenticated with the existing canonical/record/reference decoders after outer framing. Exact FileState/extent decoders distinguish their shared family; raw user content is not prefix-classified. Shared objects count once. The following are **post-verification logical canonical bytes, not compressed byte assignments**.

| Exclusive role | Unique objects | Canonical bytes | FULL count | DELTA count |
|---|---:|---:|---:|---:|
|payload_chunk|86,417|705,162,954|78,393|8,024|
|inode_table_leaf|9,890|56,781,912|9,890|0|
|directory_map_leaf|14,023|11,240,607|14,023|0|
|inode_record|89,576|8,778,448|89,576|0|
|FileState|75,927|8,048,262|75,927|0|
|extent_leaf|75,927|7,337,108|75,927|0|
|directory_state|13,647|1,337,406|13,647|0|
|inode_table_branch|159|658,004|159|0|
|directory_map_branch|406|160,456|406|0|
|namespace_root|158|19,118|158|0|
|metadata_map_leaf|4|572|4|0|
|symlink_state|7|442|7|0|
|Extent branch / metadata-map branch / other supported / unknown-invalid|0 /0 /0 /0|0 /0 /0 /0|0 /0 /0 /0|0 /0 /0 /0|
|**Total**|**366,141**|**799,525,289**|**358,117**|**8,024**|

Payload canonical bytes total705,162,954 (FULL598,565,032; DELTA reconstructed106,597,922). All structural roles total94,362,335. The largest structural bucket is inode-table leaves56,781,912 canonical bytes, but that is not a measured compressed saving. Payload size histograms and overlapping file-content/metadata-value use labels are separate from exclusive role totals:86,412 objects/705,162,813 canonical bytes serve file content and5 objects/141 bytes serve metadata values. No payload serves both in this inventory.

Encoded group ownership is exact:29,156 homogeneous payload groups occupy216,448,341 bytes;9,755 mixed-role groups occupy77,593,027;163 other homogeneous structural groups occupy1,199,510. Mixed decoded composition is recorded exactly in group-role-composition.csv. Even assigning every mixed group's whole encoded size to structure as a generous ceiling gives78,792,537 structural-containing encoded bytes, still below homogeneous payload216,448,341. Thus small-file structure is not the dominant encoded bucket; no proportional allocation is used.

Retention enumerates1 LayerStack/layer initial root,157 Commit roots and158 Branch pointers, including157 verifier-created aliases; workspace stages/pending roots are absent (0), not silently omitted. It covers all retained roots, not only HEAD. L contains all366,141 selected objects/799,525,289 canonical bytes. B contains3,065 required authenticated FULL bases/34,133,561 canonical bytes (maximum anchor fan-in72); all8,024 DELTA dependencies cross both group and pack boundaries, and every base is already in L, so **B−L=0**, **R=L∪B=L**, and **selected rows outside R=0**. Physical records without selected locators are independently **0**. This is authenticated absence of these residue classes under the specified roots, not a generic garbage-collection safety rule. Verifier-created Branch pointers do not add new canonical objects. First logical retention is derived from retained graphs; pack order is never treated as first admission. Base fan-in and cross-group/pack dependencies are recorded in delta-dependencies.csv.

The selected payload count86,417 exactly matches the existing eligible-target count. Source inspection finds all selected payload sizes eligible and no recorded optional-memory exclusions; this supports a run-specific inference that705,162,954 canonical bytes represent the same initial-missing eligible universe. That inference is distinct from a directly recorded denominator: **bytes split by missing hints and mutually exclusive terminal causes still cannot be reconstructed**, so the diagnostic remains necessary. The broad final payload ceiling is known; the useful missed-byte opportunity is not.


## All157 trajectory and elapsed scopes

`checkpoint-trajectory.csv` has158 rows including Init; `checkpoint-fields.json` declares units, population, source, phase, status and null reasons. Every canonical growth delta equals the corresponding CandidateStats inserted count/bytes. Every source mapping and allocation equation passes. First admitted pack/location checkpoints remain null: final pack ordering is not authoritative admission provenance. Final B-tree placement cannot reconstruct historical fragmentation.

| Through checkpoint | Allocated bytes at acknowledgement | Canonical bytes | Selected objects |
|---:|---:|---:|---:|
|Init|69,632|1,050|12|
|1|729,088|1,633,857|1,350|
|5|4,202,496|8,980,067|5,557|
|16|11,542,528|35,664,406|16,726|
|32|27,271,168|79,534,686|36,487|
|64|83,894,272|212,012,203|95,417|
|96|134,225,920|329,207,129|148,471|
|128|218,112,000|545,715,959|248,648|
|157|335,552,512|799,525,289|366,141|

The first checkpoint adds1,632,807 canonical bytes and1,338 objects; its286 eligible-target counts all lack predecessors, consistent with initial import, not evidence of a hint defect. The largest canonical increment is checkpoint122:30,584,802 bytes/16,714 objects, alongside12,537,856 bytes of logical database growth. Checkpoint153 adds26,412,123 bytes/14,482 objects and11,341,824 logical database bytes. These are admitted growth facts, not proof that their contents are inherently new or poorly matched. Allocation steps must not be mistaken for content steps: checkpoint60 adds16,777,216 allocated bytes while logical growth is2,625,536; similarly checkpoint71 adds16,777,216 allocated versus1,245,184 logical bytes. The allocation mechanism remains unidentified. Late history (129–157) contributes253,809,330 canonical bytes and117,493 objects; this broad growth cannot be explained by the tiny initial state.

Performance timings, all integer ns:

| Scope | ns |
|---|---:|
|Preparation, enclosing invocation|76,762,192,750|
|Transfer across157 steps|97,305,740,715|
|Public Exec across157 steps|262,063,022,083|
|Public Commit including required finalization|43,928,426,752|
|Step-wall sum|404,654,712,835|
|Work wall|460,033,711,875|
|Case wall including setup/cleanup|463,745,728,334|
|Enclosing invocation|540,947,041,625|

Step wall minus transfer/Exec/Commit is1,357,523,285ns of allocation observation, IPC, formatting and other unattributed work; it is not a measured pure observer timer. Work wall outside step walls is55,378,999,040ns, including resource observers, pending/final receipts and orchestration. Setup3,004,296,167ns, cleanup602,275,208ns and ready/closed public phases are separately retained in counter-reconciliation.json. Verification has its own preparation75,848,485,084ns, case432,497,784,833ns and enclosing508,554,006,209ns. Historical performance phase644,755,990,000ns excludes its preparation51,964,027,833ns; no speedup ratio is claimed from incompatible scopes.

Host phase CPU and I/O remain operation interval observations; host RSS peaks are process lifetime, not synchronized phase peaks. Container CPU counters are preserved as cumulative ns and boundary deltas; the boundary includes transfer/orchestration and is not a public-operation-only cost. Container memory categories overlap. Spool/runtime/staging are boundary samples. Exact free-disk samples and pure observer durations were not persisted; the source enforces a50GiB reserve but the unavailable value remains null. Nested matching5,677,066,532ns and encoding2,566,416,665ns are not added to public elapsed.

## What explains the gap, and what remains unknown

The complete retained allocation is604,758,016 bytes below historical original LayerFS940,310,528, but remains279,179,264 above packed Git. The original and candidate have equal157 selected states, not identical internal metadata/canonical construction. The observed pack footprint locates the remaining problem principally inside representation of retained objects; it does not prove which better representation is achievable on the synchronous public path. Richer semantics do not justify assigning the entire gap to metadata.

Ready/checkpoint Store interval counters record86,417 eligible targets,77,949 without hints,18,589 absent predecessors,11,555 predecessor hints,11,293 usable-base trials,8,024 admitted DELTAs and358,117 admitted FULLs. These are not one exclusive funnel. In particular125,254 correspondence-budget skips are precursor/cursor events, can exceed eligible targets, and cannot be interpreted as125,254 lost DELTA candidates. Recorded fetch/match/instruction/memory budget skips are zero. There are332 compressed mixed-group rejections; their byte-weighted useful opportunity is unknown. Prepared full/mixed/selected encoding bytes include potential race losers. Subtracting315,555,931 attempted FULL bytes and295,240,878 selected encoding bytes would not establish durable savings. The durable group inventory provides a separate population.

Source supports a possible coverage choke: correspondence reserves131,136 bytes per metadata fetch allowance, with1MiB per-file and16MiB per-operation bounds; candidate search uses validated chunk payloads and falls back to FULL without usable hints. It does not prove lost-span handoff: the reviewed memory/spill paths preserve first_span. A has_predecessor flag without first_span is not currently reported separately. Legitimate no-overlap, descriptor limits, unavailable bases, marginal candidates and genuinely new content remain alternatives. Low DELTA count is not proof of weak matching. Repeated decoding is observed (869,798 decompressions,12,450,972,177 decoded bytes), but those are reader-wave work, not unique physical disk traffic or proof that a stronger codec is feasible.

The independent review challenges all eight requested hypotheses in `independent-review.md`; its final dispositions use authenticated role/base closure rather than guessing from FULL counts.

## Ranked opportunities (not scheduled changes)

1. **Payload predecessor coverage diagnostic.** Affected bytes: exact payload/group inventory above supplies the broad ceiling, while the portion belonging to no-hint initially missing targets is **unknown**. Credible removable portion is unknown; existing counts cannot forecast it. Mechanism: bounded correspondence and FULL fallback without hints. Smallest implementation boundary for a separately authorized diagnostic is existing correspondence→candidate→final-admission receipt ownership. No concepts/product code are deleted; no codec, canonical format or reader compatibility changes. Costs: fixed integer counters/histograms and bounded per-target terminal state; their CPU/memory overhead must be measured. No extra payload reading/matching solely for instrumentation; authoritative pack provenance recorded outside public timers. Read behavior remains control policy.
2. **SQLite/index/allocation accounting ceiling.** Post-verification22,924,802 bytes outside pack BLOBs, with18,837,504 in the objects index; separately final-ack signed adjustment16,719,872. Removable fraction is unestablished. Mechanism evidence is exact page ownership, not a demonstrated placement defect. Smallest boundary would be a specific index/layout diagnosis if larger payload opportunities fail. No deletion or compatibility change is presently justified. Format/index changes could incur migration, read/write amplification and memory costs, hence are not selected.
3. **Mixed candidate/group quality.**332 rejected mixed groups is measured, but their affected encoded/decoded target bytes and prospective removable portion are unknown. Mechanism: compressed mixed alternative must beat the FULL group by the format policy margin. Smallest boundary is group-level candidate analysis on existing bases; deeper chains/global search are not justified. No concepts deleted. Candidate analysis may add encode CPU and buffers; a later actual treatment must include reader reconstruction and memory costs. Existing format could remain compatible if only writer choice changes, but this has not been demonstrated.

## Exactly one next diagnostic — draft prospective contract, NOT EXECUTED

**Question and falsifiable hypothesis:** missing correspondence coverage affects a material byte-bearing share of unique initially missing eligible payload targets, rather than only small targets or genuinely new/unrelated content. The diagnostic supports that hypothesis only if exclusive target counts **and bytes** identify that bucket relative to total payload and group bytes. If no-hint bytes are small, predominantly no-predecessor/no-overlap, or cannot be tied to a legitimate predecessor opportunity, reject that hypothesis and retain the negative finding. No numerical storage acceptance gate is invented; owner judgment must weigh the measured ceiling and absolute operation costs before a subsequent optimization experiment.

**One variable:** fixed diagnostic receipt instrumentation enabled. Control is the authenticated accepted M4.5 policy in this sealed run; diagnostic candidate has the same product choices, limits, codec, group construction and canonical semantics plus counters/provenance. This is an unpaired diagnostic, not a timing speedup test. No budget increase, span alternative, matcher tweak or recompression treatment is bundled. No product optimization is authorized by this draft.

**Workload/oracle/environment:** freeze the same ordered157 checkpoint manifest SHA256 `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`, source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`, tree/oracle/input seals, whole-file ordinary Exec imports and public Commit routes. One fresh Store with empty initial LayerStack/Branch/Workspace; same MacOS host ownership, Docker2CPU/2GiB/no-swap/256PID profile, versions and uncontrolled-cache disclosure. Seal exact new source/patch/binary/image before execution and enforce normal identity checks. This requires new owner authorization.

**Denominator and funnel:** count each unique initially missing validated eligible payload target after CAS filtering, with its full canonical byte length. Preserve a bounded target outcome through admission; transient duplicate attempts and per-trial events are separate. Funnel: eligible→predecessor→required span/correspondence→selected prior record→authenticated FULL base→complete candidate→mixed group wins→final admission. The terminal outcomes are exclusive and exhaustive: no predecessor; missing required span (handoff defect); no overlap; correspondence/descriptor limit; unavailable/inadmissible base (subreason missing prior/invalid selected FULL); fetch/match/trial/instruction/memory budget (fixed subreasons); no useful raw delta; compressed mixed rejection; DELTA admitted; admission race selected existing representation. Define outcome precedence prospectively: final race overrides prepared choices; otherwise assign the furthest completed funnel stage's failure. Mixed groups with no admitted DELTA for a target retain the target's own terminal reason, not a duplicated group rejection count.

Collect counts and canonical bytes at each stage/outcome per public acknowledgement; fixed size bins [0,64),[64,256),[256,1024),[1024,4096),[4096,16384),[16384,65536),[65536,+∞) bytes; separate candidate-trial and group A/B outcomes. Capture cheap per-group attempted A/B encoded lengths and selected winner lengths during the existing work, then serialize them and authoritative newly persisted pack IDs/selected locators outside timed operations, distinguishing prepared race losers. Do not perform extra encoding to reconstruct discarded alternatives. No raw bytes, paths, per-match logs or every ObjectId in timed operations. Match/fetch/encode work and buffer maxima stay on existing receipt scopes. `delta-opportunity.csv` names the missing fields and decision; null is deliberate, not zero.

**Census checkpoints:** Init and final157 only. Existing cheap per-ack allocation/canonical/counter receipts already provide the trajectory; the current analysis found no reason to scan intermediate Stores. A final pre-verification logical snapshot resolves durable role/group/base attribution; Init identifies the fixed empty baseline. No whole-Store copy, graph traversal or decompression after each Commit. Record observer durations and cache effects; instrumentation outside public timers does not make the run an uncontaminated performance control.

**Snapshot/custody:** record allocation/logical size/sidecars on the original at acknowledgement. Close its owner normally, report End and cleanup separately, seal Store bytes/stat and performance receipts, preserve one final logical snapshot. Census and all157 historical verification use disposable copies with separately recorded origin/copy/hash custody. APFS clones/copies preserve content, not allocation equivalence. Keep the original manifests unchanged and identity checks strict. At Init, only fixed small logical census is allowed if its observer cost is recorded.

**Correctness and boundaries:** verify every source/mapping/oracle, same retention roots including stages/pending if present, authenticated ObjectIds, FULL bases/dependency bounds, pack/group/page equations, and outcome counts+bytes summing exactly to the denominator. Record final acknowledged original Store allocation including sidecars, all intermediate allocation observations, separate spool/staging/runtime/free disk, public Init/Exec/Commit-finalization/End, transfer/preparation/observer/case/invocation timers, host phase CPU/I/O and sampled/lifetime RSS, container CPU/memory and cleanup. No nested codec time summation. Any later optimization claim needs synchronous public-path footprint/read/resource evidence; offline encoding could show potential only.

**Stop and retain:** stop on identity/oracle/integrity/route failure, missing terminal accounting, budget violation, OOM/swap, active-writer snapshot conflict, timeout or incomplete cleanup. Preserve partial receipts, failed census, negative outcomes and exact instrumentation cost; do not extend budgets or silently change census checkpoints. Inherit300s per acknowledged step,4h preparation/performance/verification bounds,120s setup/cleanup, Store16GiB, scoped runtime/spool/staging16GiB, owned directory32GiB, host reserve50GiB and sampled host RSS8GiB. The owner allows roughly30% longer foreground operations for meaningful storage gains; that guidance is for a future actual optimization, must include absolute time for short calls, and is not a new gate or excuse to reinterpret historical failures.

## Limitations and issue disposition

This report is descriptive analysis with independent custody/causality review, not M5, release, crash/durability, aggregate load, S3 or Git parity qualification. Historical physical placement and missing pre-verification bytes cannot be recreated; exact durable admission chronology and byte-weighted delta terminal causes were not recorded. Objects outside the specified root closure are not declared safe garbage. Group-sharing prevents record-level compressed savings claims. Analysis artifacts are separate and original evidence is unchanged.

**Keep #87 open** for the single separately authorized byte-weighted delta-coverage diagnostic above. The analysis deliverables are complete when sealed and published; the unresolved diagnostic is a precise next step, not authority to execute it. No parent/release/allocation issue is closed and PR#81 is not merged.

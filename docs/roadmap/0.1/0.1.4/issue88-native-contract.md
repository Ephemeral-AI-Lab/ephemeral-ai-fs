# P: native payload pack version2 — prospective physical contract v1

Status: concrete design/interface contract, not implemented or executed. This binds the selected minimal grammar and physical policy for the issue88 combined route. Product implementation and samples wait for the D/C delivery decision and a committed candidate/source identity. If D/C changes delivery, both later public arms use the same accepted delivery foundation; no parameter below changes silently. This document alone launches no build, encoder, Store opening, replay or migration.

The public comparison is fresh S1 with legacy payload representation versus the same S1 foundation plus this complete native physical treatment. The treatment includes payload framing, codec, first-delivered selected-prior handling and bounded dependencies; it is not a codec-only experiment. S1 canonical/group codec/origin/FULL-base policy stays fixed. Shared resource and grouping interactions are measured, not assumed absent.

## 1. Population and physical ownership

Only an authenticated current canonical chunk with reviewed regular-file payload provenance is native-writer eligible. Decode outer framing and the exact chunk family; raw user-prefix matching is forbidden. Preserve existing21-byte canonical chunk framing, ObjectId,32,768-byte maximum raw chunk payload and CDC. Canonical lengths in `objects` and CAS comparisons always mean raw payload length plus21.

All other objects use the existing version1 path, including structural S1 groups and metadata-value-only chunks. A shared file/metadata chunk has one selected canonical ObjectId; CAS first surviving admission remains authoritative. No duplicate representation is emitted merely because another use class appears. D must keep exact file-payload and structural cohorts separate: an inode origin lacking `first_span` is intentional, not a payload handoff defect.

Native groups are batched into payload-only version2 packs; structural and legacy groups remain version1. Both pack versions use the existing SQLite `object_packs` and `objects` tables, existing membership/final recheck, collision comparison and successful publication transaction. No extra database, persistent content index, digest-translation table or per-chunk BLOB. No SQL schema change is required for this experiment's new BLOB grammar. Fresh experimental Stores only; no old-Store write conversion.

## 2. Exact wire grammar

All integers below are unsigned little-endian. Every addition/multiplication, slice endpoint and allocation size is checked before use. Values outside bounds, unknown tags, nonzero reserved bits, truncation and trailing bytes are integrity errors.

### Pack header and directory

The outer16-byte header retains the existing shape:

| Offset | Width | Value |
|---:|---:|---|
|0|8|ASCII `LFPACK` followed by two zero bytes|
|8|4|Version2, integer2|
|12|4|Group count,1..256|

Exactly `group_count`16-byte directory entries follow:

| Entry offset | Width | Value |
|---:|---:|---|
|0|4|Absolute group start within pack|
|4|4|Stored group length|
|8|4|Group framing length; exactly equal to stored length for version2|
|12|1|Outer codec0 RAW, mandatory|
|13|3|Zero reserved bytes|

Ranges are contiguous from `16 +16*group_count` through exact pack EOF; no holes, overlap or padding. Each RAW group is at most65,536 bytes. Pack length including header/directory and all groups is at most262,144 bytes, and total records per pack at most8,191. Version2 has no oversized singleton exception and no outer Zstandard compression. The duplicated group-length fields preserve the current outer directory shape; they must agree. They do **not** denote canonical reconstruction length. Whole-pack assembly/validation and the final census prove contiguous complete directory coverage. A demanded read checks the header and selected entry bounds as the existing point reader does; it does not read every unrelated directory entry or claim that one point read validated the entire pack. This distinction keeps the charged32-byte header/entry extraction honest.

### RAW group framing

The existing group directory shape is retained: `record_count:u32`, followed by `record_count` cumulative `u32` end offsets relative to the record area, followed by the records. Count is1..8,191. Ends strictly increase, every record is nonempty, and the last end equals record-area length. Directory/framing plus records must fit65,536 bytes.

### Native records

The pack version selects the record grammar. Version2 kind0 and kind1 are **not** the old version1 FULL/COPY-INSERT meanings.

**Native FULL:**

```text
kind:u8 =0
raw_payload_length:u32 =0..32768
zstandard_frame: all remaining record bytes
```

**Native PREFIX:**

```text
kind:u8 =1
raw_payload_length:u32 =0..32768
base_canonical_object_id:32 bytes
zstandard_frame: all remaining record bytes
```

Thus fixed FULL/PREFIX overhead is5/37 bytes. A frame is nonempty and at most33,024 bytes. Record boundaries already provide the frame length; do not store it again. Depth and closure length are derived by bounded traversal rather than duplicated in the wire. Target identity comes from the demanded selected ObjectId/locator, followed by canonical authentication; no duplicate raw-SHA256 or target ObjectId field is stored in every record. Base identity is a canonical ObjectId, not the raw digest used by the offline analysis container.

A native reader reconstructs the exact canonical chunk with the existing encoder and authenticates its demanded ObjectId. It checks locator canonical length equals raw length plus21. Validation of a whole group parses all record boundaries and fixed native headers; it decompresses only demanded records and required dependencies. Unselected records remain part of physical accounting and are validated fully by the final census.

## 3. Codec and candidate decision

Pinned library Zstandard1.5.7, compression level3, windowLog20, single thread (`nbWorkers=0`), content size enabled, checksum enabled, dictionary ID disabled. One independent ordinary frame per record. Reset/rebind prefix for every call; it is a borrowed single-use raw prefix whose exact bytes remain alive and unmodified through compression/decompression. Do not use a trained dictionary, long-distance mode, alternate seed, level sweep or outer recompression.

Strict decoder checks before trusting output: ordinary Zstd magic/frame, exactly one complete frame, no trailing/concatenated/skippable frames, required checksum, no dictionary-ID field (including explicit zero), expected content size equal to the native raw length, and frame window at most1,048,576 bytes. Reserved Zstd header bits remain invalid. Use a non-growing static context and bounded exact output capacity; normal canonical authentication is still required after successful codec decoding.

For each initially missing eligible target, encode native FULL. The candidate prior is the **first nonempty slot in the validated public-delivered payload hint array**, in its existing order. Do not search another slot if that prior is absent, incompatible, too deep, over budget or unhelpful. Do not consult frozen offline manifests or globally sort producers. An absent prior yields native FULL.

The prior must be a selected object already reachable through the operation's validated predecessor snapshot/context before this target's construction, and present at optional lookup. No current prepared-batch object or future-history base. Native PREFIX publication requires the base to have a lower immutable pack ID than the newly assigned target pack; demanded reads enforce that inequality for every native edge as well as cycle/depth bounds. Actual locator/admission provenance must establish selected-before-target publication. This rule uses public predecessor ownership, not final pack-ID order as a historical clock.

Supported prior representations: native FULL, native PREFIX, or legacy version1 FULL containing an exact authenticated chunk. A legacy custom DELTA prior is an inadmissible native candidate and yields FULL; do not retarget its old FULL anchor or add an unmeasured second chain mechanism. This is a declared integration fallback, not equivalent coverage to the offline S2 actual-selected-prior screen; report its count and canonical bytes. Any supported prior must decode to the exact chunk role. Wrong/missing/cyclic dependencies inside an admitted representation are integrity errors, not a quiet FULL retry. An unavailable optional hint is a candidate fallback; that does not legitimize a missing dependency in stored data.

A proposed PREFIX may have at most4 PREFIX edges to its root FULL. Its cumulative raw payload lengths, including target, must be at most1,048,576 bytes. With the fixed chunk size/depth this is necessarily at most163,840 bytes, but validate both declared rules. Fallback when adding the target exceeds a cap. A candidate is selected iff `37 + prefix_frame_length < 5 + full_frame_length`; equality chooses FULL. FULL and PREFIX use identical codec settings and target bytes. There is no old12.5% mixed-group threshold for the native lane. S1 retains its existing group decision unchanged.

All candidate prefixes are reconstructed and canonical-authenticated in deterministic unit tests and the final census; public writes retain existing authenticated canonical comparison operands through final CAS recheck. Successful selected representation counts are recorded only after commit. Attempts, optional fallbacks and late race losers remain separate populations.

## 4. Grouping and admission interface

Keep the existing initial CAS owner and canonical ordering. The native lane greedily batches records in that order using their complete **native FULL size** as the admission bound: `4 +4*record_count +sum(FULL_record_sizes) <=65536`. A PREFIX winner is strictly smaller, so substitution cannot overflow its group. Greedily pack resulting groups in order under the complete262,144-byte/256-group/8,191-record limits. Structural version1 group construction remains its own unchanged lane. Do not adapt group size after observing savings.

Implement at the existing physical boundary, with these concrete responsibilities rather than a new generic storage abstraction:

- `pack::header` returns a validated version plus group count; `GroupEntry`/read dispatch retains version. Version1 parsing remains its current grammar and limits. Version2 directory parsing requires RAW and the native bounds.
- `pack::native_record` borrows the fixed header and frame slice and returns `Full { raw_length, frame }` or `Prefix { raw_length, base, frame }`; it performs no database access or output allocation.
- Bounded native codec helpers accept raw payload and optional borrowed prefix, return an owned frame or exact raw payload, and expose codec-work counts/time. They do not acquire database locks or select bases.
- `StoreDb::read_native_payload` takes an already located demanded ObjectId plus the caller's budget, authenticates a bounded closure and returns one canonical chunk. It uses existing locator/BLOB-range extraction and normal reader connection ownership; no writer permit or recursive public API call.
- `PreparedAdmission` obtains at most one prior per native target, reconstructs it outside publication, prepares FULL/PREFIX and records, retains the original canonical CAS comparison operand, then uses the existing final recheck/publisher. Native output must never be compared to canonical input as if compressed bytes were canonical.
- Existing `insert` assigns immutable pack IDs and inserts both versions plus selected locators atomically. No base enters the candidate from an uncommitted prepared pack. Partially winning packs still persist all physical records; report their residue, not proportional compressed attribution.

These are implementation seams, not permission to delete the legacy matcher/reader: structural S1 and old Store compatibility still require them. No target hashes, frame lengths, depth/closure summaries or global trace table are added when existing framing/locators plus bounded validation suffice.

## 5. Fixed resource accounting

Process native reads and optional base reconstruction sequentially one chain at a time. No decoded-object cache in the first prototype. A chain has at most5 records. Discovery parses demanded record headers and copies only selected frame bytes/needed legacy FULL canonical bytes into bounded owned state; release each enclosing group before fetching another. Authenticate from the root FULL toward the target, retaining only the current raw prefix and next output. Do not retain five whole decoded groups or five codec contexts.

Per demanded native chain, including the demanded target:

| Bound | Ceiling / accounting |
|---|---|
|Native frame bytes per record|33,024 B|
|Enclosing group bytes per fetched record|65,536 B, including framing|
|Record lookups/group fetches|At most5; repeated enclosing groups count again if fetched again|
|Encoded group+outer-header/directory reads|393,216 B; every fetched group length plus32 B header/entry extraction charged|
|Decoded-work charge|524,288 B; each decoded/RAW enclosing group plus each reconstructed canonical chunk, including21-byte framing, charged|
|Cumulative raw payload closure|1,048,576 B contract, and implied163,840 B at fixed size/depth|
|Per-chain owned scratch|1,048,576 B maximum, covering frames/descriptors, enclosing buffers, prefix/output and decoder context|
|Native static decoder context|262,144 B maximum|
|Native static encoder context|1,048,576 B maximum|
|Shared physical encoding/read scratch|Existing2,097,152 B owner ceiling remains; reserve actual simultaneous capacities, not nominal lengths|

The per-chain read ceilings cover the five-record worst case:5×65,536 enclosing bytes plus5×32 outer bytes, and5×65,536 group work plus5×32,789 canonical reconstruction. They are logical extraction/decode budgets, not measured disk-page I/O; SQLite cache/pages and OS I/O remain separately measured. Every extra directory reread or duplicate group fetch must be charged. Avoid fetching entire pack BLOBs merely to extract one group.

Optional writer prefix discovery retains the current admission batch8MiB encoded and8MiB decoded hint-work allowances,512KiB encoded and512KiB decoded per-target limits, and8 lookup limit per target; its chain work is charged to the same shared owner, not a second hidden allowance. The native per-chain limits do not replace those existing per-target limits: both apply. The initial optional-hint lookup and every repeated lookup count toward the eight; reuse its returned locator rather than probing again merely to start the chain. Extra header/directory reads and canonical reconstruction work remain charged as specified above. One native prefix trial per target is also subject to the existing512 optional-trial batch ceiling. The custom matcher's16MiB comparison-work counter continues to govern its actual custom work; do not translate native CPU into fictitious byte comparisons. Native work is bounded by target count, declared read allowances, fixed frame/context capacities and the existing public operation deadline, and is reported separately. These budgets can make native or structural outcomes compete; do not silently split them to preserve offline totals.

Reader decoder context is dropped before the writer's encoder phase; only the authenticated raw prior survives that transition. Writer FULL/prefix buffers, complete native record directories, already prepared pack backing, vector capacities and comparison operands must fit existing declared ownership. Before any optional allocation, reserve its entire simultaneous upper bound; if optional work cannot fit, retain native FULL without changing the global allowance. If mandatory FULL encoding cannot fit the frozen static context/owner limits, stop preparation as a resource failure; do not secretly switch codec/window/format or allocate dynamically.

Actual compiled worst-case tests must prove these bounds for the pinned build. Offline observed contexts541,720/123,336 B are evidence only, not a substitute for static allocation checks. A valid admitted record exceeding a frozen read ceiling indicates an implementation/contract inconsistency and fails the read; do not return partial unauthenticated bytes. Budget overflow is not solved by retries with reset counters.

## 6. Compatibility and information security

New code reads version1 and version2 explicitly. Old version1 readers reject version2 at the existing header version check; no claim of backward readability by old binaries. Strictly reject native records in version1, version1 custom instruction records in version2, non-RAW version2 groups and mixed structural/native payload packs. No fallback reinterpretation after a parse failure.

Canonical IDs remain full cryptographic identities; do not truncate them or treat Zstd's checksum as authentication. CAS collision checks authenticate full canonical bytes both before admission and when a late selected representation wins. Stored PREFIX base role, selected locator, canonical length and reconstructed identity are all validated. Header-derived lengths are untrusted until bounded; checksum/frame failure, wrong base, cycles and malformed role are integrity errors. No file bytes, paths, ObjectIds or per-match traces are added to timed diagnostic logs; aggregate counters and outside-timer successful-location provenance suffice.

## 7. Required tests and execution gates

Before any retained encoding/public sample, freeze source/product/dirty patch, binary/codec identity and run command, and pass the following bounded tests:

1. Exact byte grammar fixtures for zero/max chunk, FULL/PREFIX, multiple records/groups, size/offset/EOF boundaries and version dispatch. Invalid tags, reserved fields, count/length arithmetic and outer-codec mismatches reject.
2. Native roundtrip reconstructs the existing canonical21-byte framing and exact ObjectId. Wrong base, self/cycle, missing dependency, cross-role base, truncated/concatenated/skippable frames, dictionary-ID fields, missing checksum, content-size/window overflow and too-large frame reject.
3. FULL tie fallback, no hint, first-hint-only behavior, unsupported legacy-DELTA base fallback, depth4 success/depth5 refusal, legacy FULL→native chain, resource-exhausted optional fallback, and mandatory codec resource failure.
4. Exact context/capacity ownership at worst frame/chain sizes; no context or prefix lifetime violation; all group/read/decoded-work charges include repeated fetches. Max-depth small-range/full reads return authenticated bytes and expose complete work counts.
5. Initial/final CAS and collision checks with native selected records; duplicate input, late race, shared/high-fan-in base, A→B→A recurrence and partial-pack admission preserve canonical identity and required bases.
6. Legacy S1 structural samples remain readable and obey FULL-base-only structural rules. Compare actual structural results; do not assume the offline56,857,102 B or S1 public results are invariant.

Then follow the approved smoke-first matched S1 versus S1+P workflow on the resolved D/C foundation, and a prospectively justified full157 pair. This contract creates no additional four-arm/full-history matrix. Hold the existing measurement lock, preserve every failure/negative result, and use original public resource/time/cleanup gates. No parameter sweep, adaptive regrouping, post-observation budget change, migration, rollout or merge.

## 8. Accounting and milestone bridge

Report native frame bytes,5-byte FULL headers,37-byte PREFIX headers, group count/end directories, pack headers/directories, legacy residual objects and S1 compressed group bodies exactly once. Base-ID bytes are header bytes, not another base-record copy. Required physical-only bases and unselected records remain separate populations but are not added twice if already enumerated in lane totals.

The reference102,306,097 payload bytes contains56-byte experimental per-record headers and a16-byte standalone header. Replace that envelope with actual measured native framing; do not compare it silently to the new5/37-byte grammar or claim every removed field is a storage result before execution. Add the previously omitted non-file payload objects through their legacy lane and use one real selected index. SQLite page/BLOB allocation and original acknowledgement Store+sidecars are enclosing accounts, not additive copies of packs.

Every result states units, population, source identity, phase/snapshot and measured/derived/unknown status. Invalid/unavailable is null with a reason, never zero. Measure public operation and matched small/full read latency, CPU/RSS/I/O, temporary disk, observer effects and exact allocation. The159,163,199-byte component milestone is not a complete Store target or lower bound;134,221,004 allocated bytes remains the separate stretch objective. Retain/revise/reject from actual combined size and explicit costs, preserving earlier regressions and owner absolute-time guidance.

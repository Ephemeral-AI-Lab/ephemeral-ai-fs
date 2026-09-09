# LayerFS source gap study: ten selected snapshots

Read-only product study; no source/settings changes, builds, new encoding experiments, or benchmarks. This report uses the already collected ten-state decomposition supplied by the coordinator. Repository root and parent AGENTS.md locations were checked; none existed. The rejected `../metadata-optimization.md` evidence was read. Existing unrelated untracked `target` was preserved.

## What the accounting requires

Existing candidate allocation is 66,105,344 bytes versus Git's 38,223,872 bytes. Candidate components:

| Component | Bytes |
|---|---:|
| SmallContent FULL frames | 30,191,026 |
| SmallContent DELTA frames | 19,361,953 |
| Large FULL frames | 2,097,249 |
| Large PREFIX frames | 1,140,960 |
| Metadata/legacy packs | 6,559,114 |
| Outside packs | 5,300,614 |
| Remaining content framing/pack overhead | 1,454,428 |
| Total | 66,105,344 |

SmallContent FULL is the largest component in this short history. Removing every DELTA frame, an impossible ideal held against the existing other costs, still leaves 46,743,391 bytes. Therefore an anchor-only DELTA optimization cannot reach 40 MB. Holding other components fixed, FULL+DELTA frames would have to fall from 49,552,979 to 23,447,635 bytes: a 52.68% reduction. Matching Git exactly requires 56.27% reduction in that combined population. These are required budgets, not forecasts of recoverable bytes.

## Exact current owners and behavior

All source links below are repository relative.

- `crates/layerfs-layerstack-store/src/objects.rs::build_checked_file_inner` (2842) builds one whole-file object for nonempty files strictly below 131072 bytes. `crates/layerfs-content/src/file/content.rs::build_bytes` retains CDC above the boundary. The shared completion path serves direct native and private Workspace candidates. SmallContent is not CDC with a changed chunk size.
- `objects.rs::DeferredObjectStore::consume_prevalidated_pages` (method at 2395) carries `small_predecessor` to `prior_ids[0]` (2435). `FinalizedOutputWriter` insertion also attaches this same predecessor (793). The four-slot generic hint field does not mean four SmallContent candidates.
- `crates/layerfs-layerstack-store/src/objects/admission.rs::PreparedAdmission::prepare_small` (212) batches predecessor locator lookup, but evaluates exactly one predecessor-derived anchor per object. It compresses the target as FULL and again using that FULL's bytes as Zstandard prefix. It selects DELTA only when `delta.len() + 32 < full.len()` (248). The 32-byte base reference is included; there is no missing elementary FULL-versus-DELTA cost comparison.
- `crates/layerfs-layerstack-store/src/objects/read.rs::StoreDb::small_anchor` (250) reads the predecessor's selected record. If DELTA, it follows the recorded base directly to its FULL. It does not reconstruct the predecessor to use its newer bytes; it does not search siblings, other versions, or later objects. This is deliberate one-level policy, not a discovered decoder defect.
- `read.rs::read_small_full` (239) rejects a DELTA base and authenticates the decoded canonical object. `read_small` (266) authenticates the reconstructed target and limits its per-read-wave FULL operand cache to 256 KiB. Chronology and self-cycle checks remain enforced. Simply pointing at a DELTA predecessor would fail these checks and violate existing design.
- `crates/layerfs-layerstack-store/src/objects/pack.rs::native_parameters` (943) pins SmallContent Zstandard level 3, windowLog 18, content size and checksum, no dictionary ID and no workers. `native_compress_in` uses `ZSTD_CCtx_refPrefix`. This is raw-prefix compression, rather than Git's separately constructed copy/insert delta program. Whether either matcher wins for identical bases is unknown without paired byte analysis; no codec sweep is warranted.
- `pack.rs::small_workspace` (964) estimates bounded static compression scratch; `prepare_small` accounts for operands, physical output and codec memory and destroys the encoder before decoding a new anchor. The latter can repeat setup/authenticated reads across objects; it is a possible speed owner, not a demonstrated size saving. Any reuse must remain bounded and preserve existing comparison/authentication ownership.
- `crates/layerfs-layerstack-store/src/objects/delta.rs::encode` writes 9-byte fixed record framing plus 32 bytes for a DELTA base. `pack.rs::assemble_small` writes a pack header and 16-byte group directory entries. Each SmallContent object is one group, with pack bounds 256 KiB/256 groups. Trimming this framing cannot plausibly recover the 26 MB required.
- `admission.rs::compare` and `read.rs::compare_singleton` now dispatch pack-v3 upper-range SmallContent through its reader while preserving legacy oversized reads. `admission/native_tests.rs::small_content_upper_range_exact_cas_reuse` (610) is the existing focused regression. No need to rediscover or reimplement the fixed CAS issue.

## Why there are many FULLs

`prepare_small` falls back to FULL when no hint resolves to an eligible SmallContent FULL anchor, or when the one prefix candidate fails to save more than its 32-byte reference cost. These causes include distinct new-path content, absent predecessor, crossing from the large-file representation, and an eligible but poor anchor. Existing exact CAS reuse occurs before missing-object physical admission.

Source establishes these possibilities, but the 30,191,026-byte FULL total alone does **not** establish their respective contributions. In particular, no evidence yet says how much consists of unavoidable first versions versus similar new files denied any cross-path candidate. That split is the highest-value missing diagnostic for the 40 MB question.

The greedy DELTA decision minimizes this object's present cost. It never elects a more expensive FULL merely because future objects may benefit from a refreshed anchor. Such a local decision can retain growing differences against an old FULL throughout a history. It explains a mechanism; actual recoverable bytes must include new anchors and cannot be inferred from the number of DELTAs.

## Metadata and locator costs

`tree/inode/record.rs` separates stable 32-byte inode identity from inode records containing content and metadata object IDs. `tree/inode/codec.rs::encode_inode_table_node` encodes inode-ID/object-ID pairs (64 bytes each) in leaves/branches, with at most 127 entries. `tree/directory/codec.rs` owns directory nodes, states and namespace roots. Thus LayerFS preserves separate namespace, inode identity, content and filesystem metadata structures, rather than just Git tree entry names/modes/object IDs. These structures serve hardlinks and POSIX behavior; deleting them is not a permissible shortcut.

`admission.rs::DeltaSearch::candidate` (1188) permits structural delta attempts only for inode-table leaves (or legacy chunk payloads). Other metadata is FULL/group compressed. For inode leaves it deliberately skips a predecessor that is DELTA. Ordinary groups already compare completely compressed FULL and mixed alternatives: `pack.rs::encode_group` (699) requires savings at least max(64 bytes, one eighth of FULL). It is wrong to say metadata compression is disabled.

The prior inode anchor-following patch converted FULLs to DELTAs but increased pack bytes by 76 and saved no allocated space. It was reverted; see `../metadata-optimization.md`. Its failure forbids treating more delta records as evidence of improvement. Expanding eligible structural roles requires origin provenance and type checks, plus measured complete-group savings; indiscriminate history search is not an appropriate first change.

`sql/schema/v8.sql` stores pack BLOBs and a WITHOUT ROWID object locator keyed by 32-byte object ID with canonical length, pack, group and record fields. `objects/read.rs::object_locations` is the lookup owner. The 5.30 MB outside-pack allocation also includes other tables and SQLite page effects; it is not all redundant locator data. Four-KiB page size is fixed. A new compact index is a schema/read-path project and cannot alone close 26 MB.

## Smallest useful next investigations, before choosing implementation

1. Read existing ten-state records/receipts and classify **FULL bytes** by absent/ineligible predecessor versus failed delta cost comparison. Attribute first appearances and transitions using the frozen ten-state oracles. Do not replay product history. If the original telemetry does not distinguish a cause, mark it unknown instead of inventing counts.
2. For the highest-byte real file lineage, compare existing chosen FULL versus the immediately preceding version under the **same pinned prefix codec**, with exact encoded FULL/base/reference costs and reconstruction depth recorded. This is a narrow diagnostic on retained bytes, not a replacement benchmark or a new synthetic workload. It would quantify the opportunity for bounded short chains; it must not be reported as accepted product storage.
3. If absent-candidate FULLs dominate, examine a small, byte-ranked set of similar real files against a bounded preceding FULL candidate pool. Record candidate provenance and cost including new lookup metadata. This tests whether Git's broader search captures material sharing unavailable to same-file predecessor policy. Do not start with an unbounded history index or parameter sweep.
4. Only after content economics are known, decompose the 6.56 MB metadata by canonical role and complete selected group. The 49 MB full157 metadata total must not be imported as the ten-state budget.

These are proposed diagnostics only; none was run by this source study.

## What requires changing the fixed design

| Direction | Current constraint affected | Scope and uncertainty |
|---|---|---|
| Bounded recent eligible FULL pool across paths/versions | One predecessor-derived candidate / fixed candidate selection | Can preserve one-level read depth and codec. Requires candidate lookup/provenance budget; unknown FULL saving. Best motivated if no-base FULL bytes dominate. |
| Bounded short DELTA chains | One eligible FULL base, depth one, read/ownership bounds | Could reduce both alternating FULLs and repeated accumulated deltas. Requires authenticated bounded reconstruction at shared read owner, corruption/depth tests and explicit design revision. Not a one-line pointer change. |
| Anchor refresh while remaining one-level | Fixed FULL-vs-DELTA selection | Smaller representation change but can add costly FULLs; old anchors cannot be removed because retained versions need them. Might address full157 more than ten states. |
| Wider metadata delta eligibility | Fixed structural policy and provenance | Existing copy/insert/group machinery can be reused only for authenticated appropriate origins. Maximum ten-state population is 6.56 MB, so cannot solve target alone. |
| Repack/rewrite old choices using future history | No GC/repacker project, foreground durability | Closest to Git's offline freedom, but explicitly out of scope. |
| Codec/page/boundary sweep | Explicit fixed settings | Out of scope and unsupported by current evidence. |

A 40 MB result is not yet demonstrated or predicted. The concrete first direction is **understand and recover the 30.19 MB FULL population together with the 19.36 MB DELTA population**. Restricting work to stale anchors would repeat the earlier mistake of optimizing too narrow a component.

## Follow-up: actual FULL-to-Git attribution and a concrete boundary case

The coordinator subsequently completed read-only target identity mapping against the retained Git pack. Evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence/small-full-git-attribution.json`; pack rows: `git-pack-attribution-raw.txt`. This refines the initially unknown FULL opportunity without inventing LayerFS fallback counters.

| Actual LayerFS FULL targets | Count | LayerFS FULL frame bytes | Git packed target bytes, excluding base closure |
|---|---:|---:|---:|
| Git also FULL | 5,025 | 11,177,274 | 10,605,176 |
| Git DELTA requiring later snapshot closure | 7,855 | 16,494,159 | 1,687,032 |
| Git DELTA with same-snapshot closure | 989 | 1,101,538 | 356,151 |
| Git DELTA with earlier closure | 777 | 1,418,055 | 265,491 |

Most FULL payload that Git deltifies lies in the **later-closure** group: 16.49 MB. Those exact dependencies are not available when the online Store first admits the target. Their compressed target sizes cannot be advertised as directly recoverable by a bounded earlier-only search. Another online base might exist, but that has not been demonstrated. Even same-snapshot availability does not prove availability within the current admission order/batch. Conversely, 2.52 MB of FULL frames has same/earlier Git closure, a concrete narrower population for bounded online diagnosis. None of these categories identifies whether LayerFS chose FULL due to unavailable hint, ineligible representation, or cost rejection; the actual fallback split remains missing.

The largest listed earlier-closure example is particularly informative:

| Same path: `scripts/snapshots/translation-prompt-v4/request-response.expected.json` | Smoke step 7 | Smoke step 8 |
|---|---:|---:|
| Git blob | `587d59a8bd1c112b5df89c0354c38d80244a72d6` | `627097be238ed52c9789369e0e7e3dba0db93e27` |
| Actual complete blob length | **133,273 B** | **129,991 B** |
| Git selected representation | FULL, 49,944 B | DELTA, 5,492 B |
| LayerFS size-class consequence | CDC/extent root | SmallContent |

Read-only `git ls-tree` against the retained ten commit mappings confirms these are consecutive versions of the same path; `git cat-file -s` establishes complete lengths. The stored `verify-pack` row confirms Git target depth one and the named base above. Its reported delta-program length 9,632 is **not** the complete target length; complete target is 129,991.

This file shrinks across the fixed 131072-byte boundary. `small_anchor` accepts only SmallContent extraction, whereas the immediately preceding 133,273-byte version uses the large-file representation. The current one-candidate policy therefore cannot use that predecessor as a SmallContent FULL base. Git can, and does, with no delta chain. The measured target frame comparison is LayerFS **50,626 B** versus Git **5,492 B**; the 45,134-byte difference is not a proven net LayerFS saving because Git's encoding and base storage differ.

This is a concrete diagnosable exclusion at the shared content-size/anchor interface, not a reason to change all boundary settings. The next smallest analysis would compare actual bytes against an earlier eligible small FULL, if one exists, or design a bounded way to use an authenticated reconstructed predecessor across representations. The latter changes the fixed eligible-FULL-base rule and reader grammar/role assumptions: a 133,273-byte predecessor cannot simply be named as a current pack-v3 SmallContent FULL (whose raw limit is 131071). It may add dependency/working-set/read costs, and an auxiliary copied FULL base could erase its saving. No such encoding, new dependency or experiment was performed.

# Improved offline structural format: stride3, 53 states

2026-09-10. **The same improved structural policy that measured 79,790,080 B on full157 produces a complete 59,760,640 B database on the captured 53-state history.** Matched Git53 occupies 49,332,224 allocated bytes. The gap is **10,428,416 B / 21.14%**, or **1.2114× Git**.

This corrects the scope of the previous stride3 comparison: that run measured the older runnable product at 100,700,160 B. Here the compact metadata, longer metadata chains, Git-selected content graph and bounded CDC improvement are actually encoded into a new complete offline database. The 53 public commits were reused, not rerun.

**Verification PASS:** all **53 original states / 306,861 path-states / 1,676,767,835 logical bytes** match through the actual candidate reader. Original fixture and performance oracle seals match, and the candidate hash is unchanged before/after. Complete object authentication, physical membership, transformed SQL identities, SQLite integrity/foreign keys and negative dependency checks also pass. The 91.08-second Python oracle pass is diagnostic verification time, not public Commit latency.

## Comparable results

| Implementation / format | Stride3: 53 states | Full157: 157 states |
| --- | ---: | ---: |
| Recorded public LayerFS | 100,700,160 B | 134,246,400 B |
| **Improved offline structural format** | **59,760,640 B** | **79,790,080 B** |
| Recorded matched Git | 49,332,224 B | 56,373,248 B |
| **Improved-format gap to matching Git** | **10,428,416 B** | **23,416,832 B** |

The two improved-format results now use the same policy, on their respective exact histories. The ten-state 41.65 MB prototype used earlier metadata/content policies and is not inserted into this same-policy row.

## Complete database accounting

| Component | Bytes |
| --- | ---: |
| Structural metadata packs | 6,877,152 |
| Structural SmallContent packs, including headers | 43,827,707 |
| Optimized native CDC packs | 4,996,438 |
| SQLite indexes, pack-table overhead, allocator and remaining SQL | 4,059,343 |
| **Complete logical and allocated database** | **59,760,640** |

There are **72,308 canonical objects** and **727 packs**. Full 32-byte object identities and their real locator index remain present; every selected physical base is counted. The scope allocator and all required history, namespace and branch rows are included.

The quiescent source includes verification fork branches and measures 84,852,736 logical / 100,700,160 allocated bytes. A fresh backup/VACUUM control measures **84,070,400 B** both logically and physically. Against that matched offline lifecycle, the structural result saves **24,309,760 B / 28.92%**. Its 40,939,520 B reduction from the public allocated result also includes the source's filesystem allocation and compaction differences; it is not all attributed to the new encoding.

## Same component policies, applied to this history

### Metadata

All 84,003 original metadata objects authenticate before rewriting. Compact D produces 10,542 canonical objects, with inline inode values, direct directory representation and eight-byte scoped inode serials. Source identities map to 25,390 serials, with a retained allocator. All 54 namespace states, including genesis, match the original inode/directory semantics.

The fixed chronological policy permits 16 edges / 128 KiB summed canonical closure, using the same previous-root greatest-shared-key candidate, COPY/INSERT matcher, codec, grouping and complete-group threshold as full157. Its matched chronological FULL packs occupy 12,350,890 B; selected delta packs occupy **6,877,152 B**, a 5,473,738 B reduction. There are 2,721 DELTAs and 7,821 FULL records, with 2,706 dependency bases retained once. Actual maximum depth is 15, with 130,223 B canonical closure. [Metadata report](experiments40/stride3-structural/metadata/report.md).

### SmallContent

All **59,768** source SmallContent identities and all selected output records authenticate over **521,421,951 unique raw bytes**. The matched Git53 base graph is re-encoded using the same pinned LayerFS prefix codec. This is the selected `git-small` policy only; the earlier losing forward/reverse policies and whole-file reference were not rerun.

Selected frames occupy 41,972,723 B. Complete records and compact directories occupy 43,822,603 B, and actual pack headers bring the contribution to **43,827,707 B**. There are 11,298 FULL and 48,470 DELTA records. All 24 bases outside the source SmallContent population fall back to FULL. Maximum selected depth is 32, cumulative canonical closure 2,409,470 B and encoded closure 87,786 B. Limits remain 50 edges / 64 MiB for this diagnostic policy. [Content report](experiments40/stride3-structural/content/report.md).

### Native CDC

The same preceding-file similarity policy processes 794 lockfile chunk identities from the actual selected-state provenance. It includes 265 improved, 67 worsened and 462 unchanged records, plus 66 newly FULL records totaling 339,827 B. All 1,998 native objects authenticate before and after treatment, and all 52 large-lockfile states restore exactly; the first selected state uses SmallContent.

Native packs decrease **5,696,523→4,996,438 B**, a net **700,085 B** reduction. No packs need splitting. The same depth-four and complete closure/work bounds apply. [CDC report](experiments40/stride3-structural/cdc/report.md).

## Assembly and verification boundary

The candidate is assembled directly from the captured source in a new file. Replaced metadata and SmallContent packs are removed only from the new copy; native packs receive the fixed CDC treatment. The metadata rewrite rederives every dependent commit/layer identity and SQL reference. Every expected physical record has exactly one locator and there are no extra unlocated records. Original file-content IDs and lengths remain identical. The source Store and Git53 artifacts remain unchanged.

The reader uses the same unsupported schema 9302 and kind103 SmallContent framing as the full157 structural reference. It authenticates complete dependency closures and rejects missing, corrupt, cyclic, wrong-role and over-depth content dependencies, plus invalid metadata dependencies. All 10,542 rewritten metadata, 59,768 SmallContent and 1,998 native content objects are read through the actual assembled copy.

The independent filesystem verifier reuses the previous oracle decoder, while selecting exactly the sealed 53-state fixture: original indices **1, 4, 7, …, 157**. It compares against original saved oracles and source performance identities. Its payload caches are 4 MiB canonical / 8 MiB packs; these do not bound total process memory or cumulative reconstruction work.

Unlike the public mounted verifier, this offline reader does not create fork branches. Candidate and source hashes must remain unchanged across the independent verification.

This remains **offline format evidence**. Longer chains, offline Git base selection, import/migration semantics and real public Commit performance are not product-qualified by this result. Offline script runtimes are not public save/Commit latency.

## Artifacts

Raw evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural`.

Candidate: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/combined/candidate.sqlite`.

SHA256: `ff2eb0bbff39cb822f5fe33514dd84adce817f2d86f4041296220211ba91a8f4`.

Source SHA256: `5c6ee04eee133539f043ee64d242c26769c77d30b434af2523666e326a8d999e`.

- [Frozen combination protocol](experiments40/stride3-structural/combined/protocol.md).
- [Complete layout result](experiments40/stride3-structural/combined/result.json).
- [Reader negative checks](experiments40/stride3-structural/combined/reader-checks.json).
- [Independent original-state verification](experiments40/stride3-structural/verification/result.json).
- [Previous full157 structural result](structural-investigations.md), [public stride3 / Git53 comparison](stride3-comparison-results.md), and [experiment ledger](optimization-checklist-and-experiment-ledger.md).

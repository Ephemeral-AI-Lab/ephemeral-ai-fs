# Long-history scaling: attribution and one bounded metadata-delta experiment

2026-09-10. Follow-up to the owner's concern that the gap to Git remains large and grows with retained history. The evidence supports that concern. The unchanged compact prototype grew from41,648,128bytes on ten selected states to107,958,272bytes on full157, while Git grew38,223,872→56,373,248bytes. Adding the intermediate states costs66,310,144bytes in that prototype versus18,149,376bytes in Git. These are workload-specific comparisons, not an asymptotic complexity claim.

Two bounded investigations were performed: a complete full157 content attribution, and one fixed metadata-delta experiment. **The new full157 offline copy occupies98,668,544bytes and passes all157original-oracle checks. It still exceeds recorded Git allocation by42,295,296bytes (75.03%).** This is an improvement, not evidence of acceptable Git-like scaling or production readiness.

## Complete-copy result

| Full157 offline layout | Metadata packs B | Complete logical/allocated B |
| --- | ---: | ---: |
| Previous hash-sorted compact D prototype | 34,917,104 | 107,958,272 |
| Matched chronological FULL control | 35,010,506 | 108,081,152 |
| **Same chronology with bounded depth-one metadata deltas** | **25,624,588** | **98,668,544** |
| **Matched reduction** | **9,385,918** | **9,412,608** |

The matched allocation reduction is8.71%. Difference from the previously published hash-sorted copy is9,289,728bytes; chronological grouping changes the control, so the matched and historical differences are reported separately.

All canonical metadata, content objects, namespace roots, typed history IDs and allocator rows remain identical to the previous compact prototype. Only physical metadata representations and locators changed. The24,748metadata objects,5,154group memberships and388metadata pack boundaries are the same in the matched FULL and DELTA treatments.

New allocation reconciles exactly:

| Category | Candidate B | Recorded Git B | Difference B |
| --- | ---: | ---: | ---: |
| Content packs / blob entries | 66,190,736 | 46,982,533 | +19,208,203 |
| Metadata packs / trees and commits | 25,624,588 | 5,007,335 | +20,617,253 |
| SQLite/index/other allocation | 6,853,220 | 4,383,380 | +2,469,840 |
| **Total** | **98,668,544** | **56,373,248** | **+42,295,296** |

The candidate's content consists of58,979,700SmallContent pack bytes and7,211,036native CDC pack bytes. No content encoding was changed in this metadata experiment. All78,619content locator rows and2,180content pack BLOBs were compared byte-for-byte with the prior verified copy.

## Metadata: repeated historical pages are a real, measured cost

The previous diagnosis found1.579physical copies per distinct inline inode value in ten states versus9.886in full157. Fixed D's FULL/group-compressed pages did not exploit historical metadata deltas. That made the short-history number an inadequate basis for accepting the design unchanged.

This experiment kept D's canonical format, compact identities, all full-width hashes and codec settings fixed. It reused the exact existing Rust `pack.rs::delta_record` COPY/INSERT matcher through a standalone wrapper, not a newly invented matcher. The source function text and binary/helper identities are saved.

The single policy:

- Only new compact inode-table leaves are eligible; target and base each have at most8KiBcanonical bytes.
- Choose one leaf from the immediately previous snapshot with the greatest number of shared stable inode serials, breaking ties by full object ID.
- The base must already have been selected FULL. A DELTA origin causes FULL fallback; no second candidate, older-anchor walk, duplicate FULL base or hidden rewrite.
- Maximum dependency depth1; canonical target+base closure at most16KiB. Actual maximum was16,288bytes.
- Match work is bounded to16MiB/512trials per root; there were no matcher/instruction-budget skips.
- Compare complete compressed FULL and MIXED groups with the existing threshold: saving at least max(64bytes,ceil(FULLencoded/8)). Publish the actual winner ledger only after this choice.
- Both treatments have identical chronological object/group/pack membership. All physical FULL bases are counted once.

Result across8,930eligible leaves:

| Outcome | Targets |
| --- | ---: |
| Selected DELTA | 4,242 |
| Chosen origin was already DELTA, so FULL fallback | **4,060** |
| No overlapping previous leaf | 465 |
| Complete group rejected the mixed alternative | 161 |
| Matcher produced no smaller candidate | 1 |
| Genesis had no previous root | 1 |

The experiment confirms that historical metadata compression recovers a material cost. It also exposes the next restriction: the one-edge rule forces4,060targets back to FULL. This is not evidence that removing all depth limits is safe, or that every such FULL is recoverable. No second depth/anchor/candidate policy was tried.

## Content: the favorable ten-state result did not carry over

The content attribution decoded and authenticated all75,398SmallContent objects from the previous compact157copy and joined their exact raw bytes to Git blob identities. No content encoding, repacking or policy trial was performed.

| Exact same-object comparison | LayerFS B | Git entry B | Difference B |
| --- | ---: | ---: | ---: |
| Ten-state compressed SmallContent frames | 32,968,802 | 32,534,916 | 433,886 |
| Full157 compressed SmallContent frames | **56,463,014** | **44,975,837** | **11,487,177** |
| Full157 complete compact SmallContent packs | **58,979,700** | **44,975,837** | **14,003,863** |

Git entry sizes include its framing; LayerFS frame sizes exclude its own framing. Current compact SmallContent framing costs2,516,686bytes and is not subtracted again from the complete-copy total.

Opposing representation assignments were reconciled. LayerFS has20,682,353bytes of FULL frames whose Git representations require future-dependent deltas, but in the opposite population9,203LayerFS DELTAs cost4,522,366bytes while Git stores them FULL using27,330,025bytes. The22,807,659-byte offset prevents treating the apparent future-FULL mismatch as a standalone savings forecast. The net11.487MBframe gap remains after those opposing choices cancel.

## A concrete SmallContent reset population

For1,528current FULL objects, the exact same-path predecessor's existing graph cannot be extended under the current depth/decoded-byte limits:

| Binding structural limit | FULL objects | Current FULL frame B |
| --- | ---: | ---: |
| Eight-edge limit only | 1,446 | 7,655,946 |
|512-KiBdecoded canonical closure only | 67 | 1,354,592 |
| Both | 15 | 244,422 |
| **Total** | **1,528** | **9,254,960** |

Their largest prior encoded closure is only56,629bytes. Raising encoded-byte capacity alone would not help this population. These are structural eligibility facts, not delivered-hint/fallback telemetry or proven savings; other candidates may have been tried. The population is distributed across real histories such as icons, generated documentation graphs/catalogs and request/response fixtures.

The narrower next content hypothesis is one newest existing predecessor ancestor that fits the current bounds, rather than dropping the complete chain and relying on a FULL-only cache. That may avoid resets but may also accumulate larger deltas against an older base. It requires a fixed-pair diagnostic followed by chronological whole-family net accounting. No such encoding was run here.

Larger files remain a separate issue:523large Git file versions occupy2,006,385entry bytes versus7,211,036native pack bytes, a5,204,651-byte difference. Another311Git entry bytes belong to symlinks/empty content. SmallContent and metadata changes do not explain away that large-file gap.

## Verification and custody

- All24,748metadata objects reconstruct to byte-identical canonical values and all158namespace states match the prior compact prototype.
- All4,242metadata deltas have an authenticated inode-leaf FULL base in an earlier root/pack. Missing, corrupt, wrong-role, future and non-FULL bases are rejected.
- Content packs/locators, canonical IDs/lengths, SQL history/roots and allocator rows are unchanged. No typed-ID rederivation is needed for this physical-only change.
- Both offline copies pass complete metadata physical membership, SQLite integrity and foreign keys.
- The independent actual-copy verifier passed **157/157 original snapshot oracles,904,143path states and4,936,693,030logical bytes**, including types/modes/content/symlinks and inode reference counts. Every oracle hash matches the original saved fixture seals. File digest reuse is keyed by authenticated content root, with4MiBcanonical and8MiBpack payload caches.
- Candidate SHA256 remained unchanged before and after verification: `2c9e44a55042ee0f7989dbfa5e7e88b09beb284ce5fbcfa7682f72d940cfd080`.
- Original source Store, previous compact copies, Git data and earlier evidence remain unchanged. No public-product or release qualification was run.

A verifier launch initially targeted its directory before that directory existed; no process or data read began. Directory preparation was completed and the single full verification then passed. Helper extraction preserved the exact matcher body; wrapper checks covered COPY/INSERT roundtrip, identical input and zero-budget fallback. No rejected candidate policy was silently replaced.

## Assessment

The concern about long-history efficiency is justified. Compact fields alone did not address repeated historical representations. The depth-one metadata experiment improves the complete layout by about9.4MB, but98.67MBremains far above56.37MBGit. Content and metadata now contribute roughly equal shares of the remaining gap; the database/index is secondary.

Do not accept the design as production-ready based on its ten-state footprint. The next questions are bounded historical reuse in compact metadata pages and avoiding unnecessary SmallContent resets while respecting decoding work and memory bounds. Each proposed graph must be measured with all FULL bases, changed descendants, group costs and original-state verification. Larger caches, wider limits or a new codec are not justified merely by the existence of the gap.

## Artifacts

Root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas`.

- [Complete layout report](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/combined/report.md) and [result](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/combined/result.json).
- [Metadata experiment](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/metadata/report.md) and [decision ledger](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/metadata/ledger.csv).
- [Full content attribution](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/content/report.md).
- [Independent157-state verification](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/verification/result.json) and [oracle custody](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/verification/oracle-custody.json).

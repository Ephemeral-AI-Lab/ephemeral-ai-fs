# Structural storage investigations

2026-09-10. Branch: `codex/issue100-40mb-experiments`. Decimal MB throughout.

## Result

The complete rebuilt full157 offline database is **79,790,080 B**, down from the matched **98,668,544 B** control. The **18,878,464 B** reduction closes **44.63%** of the previous gap to Git. It still exceeds Git's recorded **56,373,248 B** by **23,416,832 B / 41.54%**.

This is a measured complete database, including full-width object indexes and every retained dependency. It combines longer metadata chains with a repacked SmallContent graph selected from Git's actual dependency graph and re-encoded using LayerFS's existing codec. It is an unsupported format reference with different read bounds, not a product release candidate.

**Verification PASS:** all **157 original states / 904,143 path states / 4,936,693,030 logical bytes** match through the actual candidate reader. Every oracle hash matches its original saved fixture seal, and the candidate SHA256 is unchanged before/after. Canonical authentication, physical membership, SQL integrity and negative dependency checks also pass. [Original-state verification](experiments40/structural/verification/result.json), [oracle custody](experiments40/structural/verification/oracle-custody.json). The 182.34-second Python verification is diagnostic, not public-path latency.

## STRUCT-01 — metadata cost that grows with changes

Two fixed full-history representations were measured:

| Representation | Measured bytes | Accounting and result |
| --- | ---: | --- |
| Previous depth-one metadata packs | 25,624,588 | Control, before database/index overhead |
| Longer bounded metadata chains | **17,459,061** | Same canonical objects and matched grouping; **8,165,527 B pack reduction** |
| Checkpoints plus changed/deleted inode records | **14,032,896** | Actual separate metadata database including its indexes and retained non-table metadata; different representation and read model |

The chain experiment permits 16 edges and 128 KiB cumulative canonical reconstruction; observed maxima are 15 edges and 130,304 B. It authenticates all 24,748 metadata objects and preserves all 158 roots. A single cold reconstruction can touch 16 groups in 16 packs; 128 KiB is not a total I/O or decompression budget. Its compatible packs are included in the complete copy.

The checkpoint reference stores ten full checkpoints, one every 16 roots, plus exact changes between them. It replaces 9,087 inode-table CAS objects and retains 15,661 other metadata objects. All 158 tables and 139,247 directory comparisons restore exactly. It requires whole-table materialization and up to 15 state changes, with as much as 3,433,521 decoded stream bytes. Historical point lookup and ordinary table-CAS resolution are not implemented. Its 14.03 MB total includes a different index scope, so it must not be subtracted directly from a pack-only subtotal or combined into the current Store by arithmetic.

**Decision:** history-aware metadata representation is a material opportunity, but neither tested policy brings metadata near Git's 5.01 MB tree/commit subtotal. Longer chains are retained as a reference; checkpointing does not yet justify its change to read semantics. [Metadata experiment and limitations](experiments40/structural/metadata/report.md).

## STRUCT-02 — complete content dependency policy

All four SmallContent comparisons cover the same 75,398 canonical objects and count every FULL base and changed descendant. Each saved graph independently reconstructs and authenticates all objects.

| Policy | Records + directories B | Saving from current 58,953,092 B |
| --- | ---: | ---: |
| Forward same-path eligible ancestor | 69,828,498 | **−10,875,406** |
| Reverse same-path eligible ancestor | 60,439,704 | **−1,486,612** |
| Git-selected graph, restored to current depth/byte bounds | 53,825,229 | **5,127,863** |
| Git-selected graph, extended reconstruction bounds | **49,149,947** | **9,803,145** |

The forward policy saved 6.72 MB on the previously capped FULL subset but made the entire graph 10.88 MB larger. Existing DELTA-to-DELTA records worsened by 16.89 MB. This rejects using isolated FULL-reset savings to rank the next optimization.

The bounded Git-graph reference reaches eight edges, 523,950 B cumulative canonical bytes and 71,494 B encoded closure. It demonstrates an encoded opportunity within the existing reconstruction ceilings, but uses offline Git base selection; an online discovery mechanism is not implemented.

The larger Git-graph reference reaches 41 edges, 3,270,825 B cumulative canonical bytes and 89,597 B encoded closure. Its larger saving is included in the complete database below. The additional **4,675,282 B** relative to the bounded reference comes with a real reconstruction-cost tradeoff.

A separate whole-file reference spans all 75,929 Git blobs, including large files, and occupies **50,560,752 B of records + directories**. Every blob authenticates. It requires depth 50, about 29.81 MB modeled canonical closure, and some frames larger than current pack limits. It suggests additional opportunity across the SmallContent/native boundary, but omits replacement file-root/index/framing integration and is **not a whole-Store size**.

The first four policies were frozen together. The bounded Git-graph follow-up was frozen after the negative forward/reverse results and the extended small-graph measurement, while the whole-file trial was finishing. Initial executed source/protocol snapshots are preserved; raw initial summary source-hash timing is explicitly corrected in the content manifest. [Content report and complete trial evidence](experiments40/structural/content/report.md).

## STRUCT-03 — complete-copy integration

A fresh backup/VACUUM control remains exactly 98,668,544 B. The candidate replaces complete metadata and SmallContent pack populations, updates their locators, removes replaced pack rows and VACUUMs once. Canonical IDs and lengths, native content packs, typed history, namespace roots and scope allocator remain unchanged. The full object index contains **103,367 rows**. There are **1,294 packs**.

| Contribution | Previous copy B | Combined copy B | Reduction B |
| --- | ---: | ---: | ---: |
| Metadata packs | 25,624,588 | 17,459,061 | 8,165,527 |
| SmallContent packs | 58,979,700 | 49,156,171 | 9,823,529 |
| Native packs | 7,211,036 | 7,211,036 | 0 |
| SQLite/index/allocation remainder | 6,853,220 | 5,963,812 | 889,408 |
| **Complete allocated database** | **98,668,544** | **79,790,080** | **18,878,464** |

SmallContent records are packed in dependency order into bounded packs; pack grouping differs from the chronological source. The complete pack saving therefore includes 20,384 B of pack-header reduction beyond the record/directory experiment. The database remainder also changes with pack density. These measured contributions are included once; the whole-copy result is not claimed to isolate only base selection.

The candidate uses diagnostic schema 9302 and custom SmallContent pack version 103. Its reader checks the complete dependency closure, missing bases, cycles, roles, canonical lengths and full hashes. The metadata decoded-group limit is explicitly 16 KiB. Content permits the extended reference graph; it is not qualified under the product's existing eight-edge limit. Neither the 4 MiB canonical cache nor the 8 MiB pack cache is a bound on total process memory or cumulative I/O.

[Combination protocol](experiments40/structural/combined/protocol.md), [actual layout result](experiments40/structural/combined/result.json), [baseline audit](experiments40/structural/combined/audit.json), and [reader negative checks](experiments40/structural/combined/reader-checks.json).

## Remaining gap and next decisions

| Role comparison | Current B | Recorded Git B | Difference B |
| --- | ---: | ---: | ---: |
| Content packs / blob entries | 56,367,207 | 46,982,533 | **9,384,674** |
| Metadata packs / trees and commits | 17,459,061 | 5,007,335 | **12,451,726** |
| Database/index/allocation remainder | 5,963,812 | 4,383,380 | **1,580,432** |
| **Total** | **79,790,080** | **56,373,248** | **23,416,832** |

1. Metadata remains the largest gap. Next test sharing repeated inode values within history/change representations, with actual index costs and historical lookup. The checkpoint result prevents assuming that a change log alone solves it.
2. Content needs a practical way to find better bases. The bounded Git graph is a reference for that work; the two simple same-path policies are rejected. The whole-file reference motivates a separate cross-boundary design with explicit large-file read and pack limits.
3. Decide the acceptable storage/read tradeoff before promoting longer chains. Retain the bounded and extended reference results separately. Framing/index polish stays behind these decisions.

These comparisons preserve LayerFS metadata semantics; Git's roles are an accounting reference, not proof that identical storage is achievable. The campaign establishes a material 18.88 MB reduction, not a demonstrated route all the way to 56.37 MB.

## Fast iteration policy

The owner subsequently requested committing every third snapshot; the [fixed53-state contract](stride3-snapshot-contract.md) records selection, scope and qualification. Future development campaigns use **53 states: original indices 1, 4, 7, …, 157**, preserving both endpoints. Jump directly to each selected state; do not perform hidden commits for skipped states. This reduces commit count by 66.24%, but no threefold wall-time improvement has been measured. Input preparation, larger per-step changes, encoding, setup and verification can scale differently.

Use a fresh candidate and matched control with this exact selection, plus a separately measured Git53 baseline. Full157 and ten-state results retain their original scopes. The full157 verification already running for this structural campaign is preserved. Final qualification still uses all157 states.

## Artifacts

Raw root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural`.

Candidate: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural/combined/candidate.sqlite`; SHA256 `e5837e3484d561225fa464433ed94410705bdb6cde1f60b2c057faa834a73281`.

Source: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/combined/delta.sqlite`; unchanged SHA256 `2c9e44a55042ee0f7989dbfa5e7e88b09beb284ce5fbcfa7682f72d940cfd080`.

See the [optimization checklist and append-only experiment ledger](optimization-checklist-and-experiment-ledger.md). No production source, original Store or Git baseline was rewritten. No issue comment or release was published in this campaign.

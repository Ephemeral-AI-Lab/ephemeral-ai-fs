# Ordered optimization campaign

Started2026-09-10, following the owner's instruction: **metadata first, content history second, index/layout third**. The phases execute in that order. Previous reports and stores remain immutable.

## Starting point and objectives

| History | Improved offline LayerFS | Matched Git | Gap |
| --- | ---: | ---: | ---: |
| Stride3 /53 states |59,760,640 B|49,332,224 B|10,428,416 B|
| Full157 |79,790,080 B|56,373,248 B|23,416,832 B|

Use53 states for initial measurements and test promising or demonstrably history-sensitive changes on 157 states. The proposed next engineering milestones are≤55 MB for 53 and≤65 MB for 157; longer-term objectives≤52 MB/≤60 MB remain unproven. These are investigation targets, not release gates. Keep complete allocation, original metadata/content semantics and measured read-cost tradeoffs visible.

## Priority1 — repeated historical metadata

- [x] Encode the shared-value representation and count every new index: per-value CAS/ordinal indexing loses6,766,592 B on 53 and 3,698,688 B on 157; rejectit.
- [x] Test a separately frozen physical-group catalogue: **57,974,784 B** on 53 and **71,970,816 B** on 157, saving1,785,856 B /7,819,264 B against the improved baselines.
- [x] Same policies measured on 157; preserve both rejected per-value indexing results. Cold leaf reconstruction can require4.73 MB/5.79 MB decoded groups, so the retained layout is storage-focused with a substantial read-cost tradeoff.
- [x] All original metadata, physical values and 54/158 namespace semantics authenticate. Independent original-oracle checks **PASS on all 53 and all 157 states**, with unchanged candidate hashes ([53 proof](experiments40/ordered-optimization/metadata-proof-53/result.json), [157 proof](experiments40/ordered-optimization/metadata-proof-157/result.json)).
- [x] Retain the physical-group layout for storage experiments, reject the per-value indexed layout, and move the verified53-state copy into content work. [Metadata experiment report](experiments40/ordered-optimization/metadata/report.md).

Current hypothesis: D's73-byte inode values repeat across historical leaves. A physical value pool with compact references may remove repetition without losing full canonical authentication. Pool compression, lookup index rows, leaf reconstruction and random reads must all be charged. The previous full-hash-reference model was not a compressed full-Store result.

## Priority2 — content history

**53-state storage result verified:**54,382,592 B, another 3,592,192 B below the metadata candidate; all 53 originaloraclesPASS. The same-policy157 layout measures **65,957,888 B**, saving another 6,012,928 B; allnewcontentobjects authenticate and the final157 original-oracle pass is complete. [53 proof](experiments40/ordered-optimization/content-proof-53/result.json). Integrate the measured whole-file Git-selected graph: preserve existingSmallContent/chunk canonicalIDs, add authenticated large-file objects, and map nativechunks to verified ranges with FULL fallback. Freeze policy before encoding, count whole histories rather than isolated target wins, and carry all decoder/index/root costs into any retained candidate. Do not combine a reference-only byte sum into the complete database.

## Priority3 — index and physical layout

**Complete:** one fixed1024-byte SQLite-page policy tested after content onboth53/157. It grows53 allocation4,096 B and saves157 only 73,728 B(0.112%), with deeperBtrees and~4×overflowpages. Keep4096-bytepages forboth selectedlayouts. Exactlogicalrow/schema/BLOB equality andintegrity/FKsPASS; fresh157 variantcheckscoveroriginalstates1/79/157. [Layout report](experiments40/ordered-optimization/index/report.md). Measure the actual remaining index/layout population after the selected earlier changes. Preserve canonical length validation, full hashes, required uniqueness and lookup behavior. Count real allocated database pages, not only field-size projections.

## Read-cost disposition

The metadata layout is storage-focused: cold reconstruction can decode4.73 MB/5.79 MB of scattered pool groups. The content layout is **archive/reference-only**, not a hot-read recommendation: one6,421 Bnativechunk forced15,608,785 B of whole-file decoding (2,430.90×amplification). Other selected owner chains reach20.47 MB canonical closure on 53 states. The existingpoolcache retains4 MiBdecodedbodies plus up to4 MiBcopiedrecords, in addition to canonical/pack caches. No public latency, memory or release qualification is implied by size gains.

## Reporting

Every phase will append its measured outcome and decision here and to the [experiment ledger](optimization-checklist-and-experiment-ledger.md). Offline improvements remain separate from runnable-product Commit latency and final product qualification. Raw campaign root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization`.

See the [consolidated ordered results](ordered-optimization-results.md) for the complete size table, decisions and read-cost limitations.

**Final verification:** all53 and all157 original-state oracles PASS on the selected4KiB content copies; hashes unchanged. Final sizes54,382,592B /65,957,888B. The53≤55MB milestone is met;157 remains957,888B above≤65MB, and hot-read qualification remains unresolved.

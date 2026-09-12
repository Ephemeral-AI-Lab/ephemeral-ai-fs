# Issue #100 optimization checklist and experiment ledger

Updated: **2026-09-10**. Working branch: `codex/issue100-40mb-experiments`.

This is the working index for priorities, completed approaches, results, rejected ideas and remaining qualification. Update this file after each experiment. Keep the linked raw reports and manifests immutable; add a new run when the policy changes.

**Integrated full157 update:66 MB target ACHIEVED.** After the owner authorized
full157, the unchanged promoted product measured **55,476,224 allocated B
(55.476224 MB),157/157 original states verified**,10,523,776B below66MB. Supported
compaction costs626.313062s; all11 historical-access cases and11 verification runs
pass. Cold range amplification remains22,216,028 decoded B for6421 requested B.
[Final full157 report](../issue103/full157-integrated-results.md). No broad #102
campaign or release is claimed. Earlier checkpoint assessments below are retained.

**Preceding integrated stride3 update:** issue #103 now measures **46,202,880 allocated B
(46.202880 MB), 53/53 original states verified**, after339.132858 seconds of
supported public compaction. It is8,179,712B below offline53 and3,129,344B below
recorded Git53. All11 historical-access performance cases and11 verification runs
pass15-second contracts; cold read amplification remains substantial. Full157
was not run under the revised stride3-only scope. [Final integrated report](../issue103/stride3-integrated-results.md).

**Historical offline assessment:** the ordered campaign's selected archive/reference layouts are **54,382,592B on53 states** and **65,957,888B on157 states**, with all original states verified. They exceed matching Git by **10.24% /17.00%**. Metadata pooling and whole-file history encoding reduce storage, but cold metadata reads can decode5.79MB and a native chunk can incur3,700× amplification. These are **archive/reference results, not hot-filesystem recommendations**. [Ordered results and decisions](ordered-optimization-results.md).

## 1. Current scorecard — preserve the measurement boundary

Decimal bytes throughout. The ten-state objective is **40,000,000 B**. Full157 is a separate long-history comparison to Git; the ten-state size target does not apply to it.

| Result / boundary | Ten selected states | Full157 | Status and evidence |
| --- | ---: | ---: | --- |
| Recorded Git allocation | 38,223,872 | 56,373,248 | Existing baselines; [ten-state control](ten-snapshot-baselines.md), [full157 Git audit](experiments40/full157/git-baseline/report.md) |
| Prior measured product, before issue103 | 49,319,936 | 134,246,400 | Public save/Commit + same-Store verification; [ten](retained-candidate-1-results.md), [157](retained-full157-results.md) |
| Same-policy compact D + framing B + CDC offline copies | 41,648,128 | 107,958,272 | Both measured; [compact10](40mb-experiment-results.md), [compact157](40mb-offline-full157-results.md) |
| Matched offline controls for that compact experiment | 48,783,360 | 129,937,408 | Equally VACUUMed controls; do not blend with public allocation |
| Prior offline copy with depth-one metadata deltas | Not run | **98,668,544** | [History-delta report](history-scaling-and-metadata-deltas.md); 157 original-oracle checks PASS |
| Matched chronological FULL control for depth-one deltas | Not run | 108,081,152 | Same canonical objects, chronology, groups and pack membership |
| Earlier structural offline reference, extended read bounds | Not run | **79,790,080** | [Structural investigation](structural-investigations.md); full database, all157 original oracles PASS |
| Latest ordered archive/reference,4KiB pages | Not run | **65,957,888** | [Ordered results](ordered-optimization-results.md); all157 oracles PASS, severe read amplification |

**Earlier53-state result:** the [same improved structural policy](stride3-structural-results.md) occupies **59,760,640B**, versus matched Git53 **49,332,224B**. The older public53 implementation remains100,700,160B. These are explicitly different formats, not a regression.

The latest ten-state and latest full157 winners use different metadata policies. Do not infer a scaling rate by comparing those two best numbers. The same-policy compact pair, 41,648,128→107,958,272 B, remains the measured scaling comparison. Git's full157 live allocation is now 56,197,120 B with unchanged apparent sizes and pack contents; keep that later observation separate from the recorded 56,373,248 B.

| Remaining full157 category gap | Current offline B | Recorded Git B | Gap B |
| --- | ---: | ---: | ---: |
| Content packs / blob entries | 50,710,265 | 46,982,533 | **3,727,732** |
| Metadata and physical value-pool packs / trees and commits | 9,603,174 | 5,007,335 | **4,595,839** |
| Indexes, structures and allocation | 5,644,449 | 4,383,380 | **1,261,069** |
| **Total** | **65,957,888** | **56,373,248** | **9,584,640** |

## Ordered follow-up — completed in requested order

1. **Metadata:** reject the per-value indexed pool; retain physical group catalogues as a storage experiment. Complete copies57,974,784B /71,970,816B, all53/157states verified.
2. **Content:** integrate whole-file objects and native slice adapters. Complete copies54,382,592B /65,957,888B, all53/157states verified. Archive/reference-only because of severe small-read amplification.
3. **Index/layout:**1KiB pages grow53 by4,096B and save157 only73,728B, with deeper B-trees. Retain4KiB pages uniformly; preserve both trials.

The next design requirement is acceptable read amplification. The proposed53≤55MB milestone is met;157≤65MB is missed by957,888B. [Complete ordered results](ordered-optimization-results.md).

## 2. Structural investigations — highest priority

The previous queue ranked narrow mechanisms too highly for the then-remaining **42,295,296 B** gap. The completed campaign reduced it to **23,416,832 B**; the following motivation preserves the starting measurement. At campaign start, metadata accounted for **20,617,253 B**, content **19,208,203 B**, and database/index/allocation remainder **2,469,840 B**. The objective is to establish a measured route to savings in the tens of megabytes. Category differences are accounting comparisons, not guaranteed recoverable bytes; LayerFS preserves additional metadata semantics.

| Order | ID / investigation | Required evidence | Status |
| --- | --- | --- | --- |
| 1 | STRUCT-01 — metadata cost that grows mainly with changes | Compare bounded historical page reuse with a structurally different sharing/change representation. Encode full157, include bases, checkpoints and index costs, and reconstruct exact metadata. Establish how much of the 20.62 MB gap each mechanism addresses and its read cost. | COMPLETE — measured reference |
| 2 | STRUCT-02 — complete content dependency policy | Compare current histories with a better bounded base graph and an offline repacked reference where feasible. Charge FULL resets, shared bases, worse descendants and representation boundaries. Separate base-selection losses from the cost of read guarantees across the 19.21 MB gap. | COMPLETE — measured reference |
| 3 | STRUCT-03 — complete-copy integration and feasibility | Combine compatible measured winners in a fresh database; include every locator, base, checkpoint and allocator row. Verify all157 original states and report remaining gap plus resource limitations. Reference-only encodings remain separate until they have an actual reader and layout. | COMPLETE — 79,790,080 B, all157 oracles PASS |

- [x] STRUCT-01: freeze protocol, run full-history encoded comparisons, verify and report.
- [x] STRUCT-02: freeze protocol, run complete-history graph comparisons, verify and report.
- [x] STRUCT-03: audit existing allocation and matched controls while the independent experiments run.
- [x] STRUCT-03: assemble compatible winners, verify full157 and report the actual combined outcome.
- [x] Re-rank mechanisms from the resulting evidence; do not declare success from a sequence of small improvements.

Results and experiment protocols are collected in [the structural investigation report](structural-investigations.md). Earlier measured results in section 3 remain unchanged.

### Mechanism backlog — subordinate to the structural investigations

Statuses below distinguish completed structural trials from remaining hypotheses. The next priorities are shared inode values with efficient historical lookup, practical discovery of better content bases, and a whole-file representation with explicit read limits. See the structural report for the measured remaining gap. NEXT-01/05 belong to STRUCT-01; NEXT-02/03/04/07 to STRUCT-02. Physical replacement moves into the main content investigation as a reference comparison, even though product implementation remains substantial work. NEXT-06/08 remain lower priority until the structural measurements justify them. Product qualification in section 5 remains mandatory.

### NEXT-01 — bounded metadata chains: measured, retained as a reference

- [x] Fixed16-edge/128KiB policy, complete chronology, grouping, FULL resets and bases counted.
- [x] All24,748 canonical metadata objects and all158 roots verified; actual-copy all157 original states PASS.
- [x] Actual metadata packs **25,624,588→17,459,061 B**, saving **8,165,527 B**; record read costs and product limitations.

The structural checkpoint/change alternative is also measured at **14,032,896 B** of its own metadata database, including indexes. It has different historical lookup semantics and is not part of the79.79MB copy. Do not promote either without product read-cost qualification.

### NEXT-02 — eligible SmallContent ancestor: complete-history trial rejected

- [x] Supersede the proposed isolated sample with the complete75,398-object forward graph trial.
- [x] Include all new FULL records and changed descendants; every object authenticates.
- [x] Record **10,875,406 B growth**, despite6.72MB saving on the previously capped FULL subset.

The reverse same-path trial also loses1,486,612B. These policies are rejected. Git-selected graph references demonstrate **5,127,863 B** record/directory savings within original bounds and **9,803,145 B** with extended bounds. Practical base discovery is now the relevant question.

### Mechanism rank 3 — NEXT-03: large-file history attribution and one targeted mechanism

**Evidence:** optimized native packs are 7,211,036 B versus 2,006,385 B for the 523 corresponding large Git versions, a **5,204,651 B** difference. Existing lockfile similarity recovered 909,266 B in full157, so the remaining large-file gap needs separate attribution.

- [ ] Rank actual full157 families by physical retained bytes, counting shared chunks once.
- [ ] Identify whether the dominant owner is base locality, representation boundaries, depth/closure resets or matching/compression quality.
- [ ] Freeze one family and one mechanism to test; compare complete retained history with identical inputs/settings.
- [ ] Measure decoding, signature construction, candidate trials and public-path implications alongside space.
- [ ] Extend only after the fixed-family evidence supports it; record rejected outcomes too.

**Conditional alternative:** bounded whole-file physical encoding for medium-sized files. It needs explicit grammar, window/memory accounting and random-read analysis; changing a cutoff alone is not an implementation.

### Mechanism rank 4 — NEXT-04: better alternative when a valid SmallContent predecessor compresses poorly

**Evidence:** the writer consults its retained FULL candidate cache only when the predecessor is unavailable. It does not try that candidate after an eligible predecessor yields a poor delta. The full157 same-object frame gap is 11,487,177 B, but this specific restriction's contribution is unmeasured.

- [ ] Identify a fixed real sample where a valid predecessor is weak and a different already-selected base demonstrably exists.
- [ ] Compare at most one additional candidate, preserving the same complete cost and read bounds.
- [ ] Charge additional authentication, reads and encoding work; quantify net whole-history effects before broadening the search.
- [ ] Record whether this warrants implementation or should remain deferred.

**Do not repeat:** the rejected selected/retained DELTA-cache variants without evidence that this policy differs in a material way.

### Mechanism rank 5 — NEXT-05: compact-ID/shared-inode-value hybrid

**Evidence:** full157 inline leaves repeat each distinct inode value 9.886 times. A separate value-CAS representation could share those values, but introduces approximately 89,576 CAS/index rows and replaces inline data with 32-byte references. The 27.53 MB raw-byte model is not a compressed-storage forecast.

- [ ] Revisit only after NEXT-01 establishes what bounded page deltas recover.
- [ ] Encode a matched hybrid with real page repartitioning, compression, object IDs and all new index rows.
- [ ] Compare complete metadata-plus-index and whole-copy costs on both ten-state and full157 histories.
- [ ] Reject if random hash/reference or index costs exceed the sharing benefit.

### Mechanism rank 6 — NEXT-06: remaining pack/index overhead

**Evidence:** simple compaction/locator experiments are sub-megabyte on the ten-state case. Compact framing already forms part of the current offline candidate. Only 2,469,840 B of the current full157 gap is in the entire residual category.

- [ ] Identify a specific remaining redundant physical field or index structure, with its actual population and byte ceiling.
- [ ] Compare matched layouts including required authentication, length checks, lookup structures and allocator state.
- [ ] Integrate only if the measured benefit justifies the format/read complexity.

**Not pending quick wins:** deleting a redundant object index or a free-page backlog; neither exists in the inspected source. Do not truncate content hashes or remove preallocation/authentication length checks to improve a number.

### Mechanism rank 7 — NEXT-07: reverse encoding / physical replacement

**Evidence:** Git has useful later-base graphs, but opposing FULL/DELTA assignments cancel large apparent target savings. No complete replacement graph or reclaimed-store result has been measured for LayerFS.

- [ ] First quantify a whole-family/whole-graph opportunity including every newly FULL base and changed descendant.
- [ ] Specify locator publication, dependency retention, concurrent-reader lifetime, crash recovery and real old-byte reclamation.
- [ ] Run a bounded original-family experiment before considering a general repacker.

**Deferred:** this is a substantial storage-lifecycle change. Appending another encoding without reclaiming the original is not a saving.

### Mechanism rank 8 — NEXT-08: codec / matcher replacement

**Evidence:** the existing prefix encoder won four of five identical-base original-file tests; Git's matcher won the fifth by 49 B. That small study is not a universal codec ranking, but provides no basis for a wholesale replacement.

- [ ] Revisit only if matched-base measurements on a specifically attributed residual population demonstrate a codec deficit.
- [ ] Keep base graph, input bytes and compression parameters equal during that test.
- [ ] Charge complete encoded records, reconstruction, dependencies and format/ownership costs.

## Fast iteration track — default for subsequent development campaigns

- [x] Freeze **53 states**, original indices **1, 4, 7, …, 157**, under the [stride3 contract](stride3-snapshot-contract.md).
- [x] Add explicit `deepseek-stride3` harness profile; prepare and independently check all53 original fixture/oracle seals and direct transitions ([fixture result](stride3-fixture-results.md)).
- [x] Establish the public LayerFS53 foundation and separate matched Git53 baseline: **100,700,160B vs49,332,224B allocated**, both53-state proofs PASS ([comparison](stride3-comparison-results.md)). The first improved structural53-state run now passes at **59,760,640B** ([result](stride3-structural-results.md)); later policy changes still require their own run.
- [x] Run selected-state commits directly; all53 Created and original oracles PASS. Performance+verification wall330.023s vs968.760s for the historical157-state workload, **2.94× shorter**; not an identical-workload product speedup.
- [ ] Advance promising designs to final all157 validation. Do not relabel the existing full157 measurements as53-state results.

The selection reduces commit count by66.24%; the first completed public run measures2.94× shorter performance+verification wall against the earlier157-state workload. First-use builds and outer fixture preparation are separate. The already-running structural full157 proof finished and is retained.

## 3. Experiment ledger — completed approaches and their disposition

Rows are grouped by the priority domain above. Within a domain, retained approaches appear together with their rejected alternatives and supporting studies. IDs remain stable; append a new row for a changed policy rather than overwriting a rejected result.

**Status key:** `PUBLIC-KEPT` = measured runtime foundation, not release approval; `OFFLINE-KEPT` = useful diagnostic component, not product integration; `REJECTED` = tested but not retained; `MEASURED-OFFLINE` = measured diagnostic/control without a retained optimization; `DEFERRED` = measured or studied, with further work postponed; `REFERENCE-ONLY` = encoded architectural reference without complete Store integration; `STUDY-ONLY` = attribution/model without an alternative encoding; `INCOMPLETE` = unusable as final proof; `SUPERSEDED` = valid earlier result replaced by later evidence. A checked experiment means execution/reporting is complete, not that the idea won.

### Ordered optimization follow-up

| Done | ID / approach | Complete measured result | Decision / scope | Evidence |
| --- | --- | --- | --- | --- |
| [x] | ORDER-META-01 — per-value indexed pool |53:66,527,232B;157:83,488,768B; regress6,766,592B /3,698,688B | REJECTED; pool/index cost outweighs smaller payload | [Metadata](experiments40/ordered-optimization/metadata/report.md) |
| [x] | ORDER-META-02 — physical value-group catalogue |53:57,974,784B;157:71,970,816B; save1,785,856B /7,819,264B;all original states PASS | OFFLINE-KEPT storage reference;4.73/5.79MB cold group decoding | [Metadata](experiments40/ordered-optimization/metadata/report.md) |
| [x] | ORDER-CONTENT-01 — whole-file graph/native adapters |53:54,382,592B;157:65,957,888B; additional3,592,192B /6,012,928B saving;all original states PASS | OFFLINE-KEPT archive reference; worst3,700× native-read amplification | [Content53](experiments40/ordered-optimization/content/report.md), [content157](experiments40/ordered-optimization/content/full157/report.md) |
| [x] | ORDER-INDEX-01 —1KiB SQLite pages |53:54,386,688B (+4,096B);157:65,884,160B (−73,728B);alllogicalrows identical | REJECTED as default; tiny157gain/deeperBtrees,4KiB retained | [Layout](experiments40/ordered-optimization/index/report.md) |

### Structural campaign — completed after the original queue

| Done | ID / approach | Measured result | Decision / scope | Evidence |
| --- | --- | --- | --- | --- |
| [x] | STRUCT-META-01 — bounded16-edge metadata graph | Metadata packs17,459,061B;8,165,527B smaller than depth1 | OFFLINE-KEPT; extended read cost, same canonical IDs | [Metadata report](experiments40/structural/metadata/report.md) |
| [x] | STRUCT-META-02 — checkpoint/change metadata | Actual separate metadata database14,032,896B including indexes | REFERENCE-ONLY architectural reference; exact states, whole-table reconstruction | [Metadata report](experiments40/structural/metadata/report.md) |
| [x] | STRUCT-CONTENT-01 — forward same-path graph | Records/directories69,828,498B;10,875,406B growth | REJECTED; isolated cap savings erased by complete history | [Content report](experiments40/structural/content/report.md) |
| [x] | STRUCT-CONTENT-02 — reverse same-path graph | Records/directories60,439,704B;1,486,612B growth | REJECTED | [Content report](experiments40/structural/content/report.md) |
| [x] | STRUCT-CONTENT-03 — extended Git-selected small graph | Records/directories49,149,947B;9,803,145B saving | OFFLINE-KEPT reference; max41edges/3,270,825B canonical closure | [Content report](experiments40/structural/content/report.md) |
| [x] | STRUCT-CONTENT-04 — all-blob whole-file reference | Records/directories50,560,752B, all75,929blobs authenticate | REFERENCE-ONLY architectural reference; new roots/indexes/framing missing, large read costs | [Content report](experiments40/structural/content/report.md) |
| [x] | STRUCT-CONTENT-05 — bounded Git-selected small graph | Records/directories53,825,229B;5,127,863B saving | OFFLINE-KEPT reference; original reconstruction ceilings, offline base selection | [Content report](experiments40/structural/content/report.md) |
| [x] | STRUCT-VALID-01 — compatible combined copy | **79,790,080B**,18,878,464B saving; all157original states PASS | OFFLINE-KEPT reference; still23,416,832B above Git, extended read bounds | [Complete report](structural-investigations.md) |
| [x] | STRIDE3-META-01 — same compact D + metadata chains | **6,877,152B** metadata packs;10,542canonical objects;all54roots verified | OFFLINE-KEPT; same16edge/128KiB policy | [Metadata53](experiments40/stride3-structural/metadata/report.md) |
| [x] | STRIDE3-CONTENT-01 — same Git-selected small graph | **43,827,707B** actual packs;59,768objects authenticate;maxdepth32 | OFFLINE-KEPT; same extended reader policy | [Content53](experiments40/stride3-structural/content/report.md) |
| [x] | STRIDE3-CDC-01 — same lockfile similarity policy | Native packs **5,696,523→4,996,438B**,700,085B saving including worse descendants | OFFLINE-KEPT;all1,998native objects and52large-file states verified | [CDC53](experiments40/stride3-structural/cdc/report.md) |
| [x] | STRIDE3-VALID-01 — improved structural53 complete copy | **59,760,640B**,all53 original oracles PASS; **10,428,416B /21.14% above Git53** | OFFLINE-KEPT;24,309,760B saving versus matched84,070,400B offline control | [Improved53 result](stride3-structural-results.md) |

Earlier rows below retain their historical baselines and scopes; their savings are not added again to the structural result.

### Metadata and historical reuse — priority domain 1

| Done | ID / approach | Scope and measured result | Decision / limitation | Evidence |
| --- | --- | --- | --- | --- |
| [x] | META-01 — inline inode values | Ten-state matched metadata+all-object-index subtotal 10,795,395→7,747,531 B: **3,047,864 B smaller**; replacement pages included | OFFLINE-KEPT; repeated values grow with history; no standalone whole-Store/public claim | [A–C report](experiments40/metadata/report.md) |
| [x] | META-02 — remove directory wrappers | Same subtotal after inlining 7,747,531→7,408,359 B: **339,172 B further reduction** | OFFLINE-KEPT; cumulative C saving is 3,387,036 B, not additional to META-01 | [A–C report](experiments40/metadata/report.md) |
| [x] | META-03 — scoped eight-byte inode identities | Ten-state C subtotal 7,408,359→4,450,410 B, including allocator: **2,957,949 B further reduction**; cumulative A→D **6,344,985 B** | OFFLINE-KEPT; full content/object hashes preserved; allocator/import/product semantics remain unimplemented | [D report](experiments40/metadata/report-D.md) |
| [x] | META-04 — extend fixed D to full157 | Actual metadata packs 49,487,786→34,917,104 B versus source; **14,570,682 B smaller**. Canonical metadata grows 79,372,108→80,160,065 B | OFFLINE-KEPT as evidence, insufficient scaling; original online deltas/order differ from D's FULL-only packing | [Full157 metadata](experiments40/full157/metadata/report.md) |
| [x] | META-05 — depth-one metadata deltas | Matched chronological metadata packs **35,010,506→25,624,588 B**; complete copies **108,081,152→98,668,544 B**; **9,412,608 B allocated saving** | OFFLINE-KEPT; all 157 oracles PASS; 4,060 chosen DELTA origins forced FULL fallback | [History delta report](history-scaling-and-metadata-deltas.md) |
| [x] | META-06 — earlier FULL-anchor following | Separate 10-file/30-commit smoke: metadata 22,451→22,527 B; allocated final **221,184 B unchanged** | REJECTED and reverted; more DELTAs increased accumulated-difference cost | [Earlier rejected attempt](../metadata-optimization.md) |
| [x] | META-07 — local inode-ID dictionaries | Raw field accounting: group-local **176,050 B larger**, pack-local **76,818 B larger**, even before added dictionary framing | REJECTED at diagnostic/model stage; no encoded alternative/product run | [Metadata study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/metadata/report.md) |
| [x] | META-08 — global dictionary / separate value-CAS model | Global ID dictionary model removes 2,177,813 raw B before lookup/framing; full157 hybrid model removes 27,528,815 raw B before compression/index effects | STUDY-ONLY; neither is achieved storage; do not add these models to META-03/05 savings | [Scaling diagnosis](experiments40/full157/metadata/scaling-diagnosis.md) |

### SmallContent history and base selection — priority domains 2 and 4

These public ten-state rows are **cumulative candidates**, not independent savings that can be summed. Each valid candidate has its own same-Store verifier and cleanup evidence.

| Done | ID / approach | Public ten-state final allocation / timing | Decision / limitation | Evidence |
| --- | --- | --- | --- | --- |
| [x] | SMALL-01 — bounded predecessor chains | **56,668,160 B**, versus original v0.1.5 66,105,344 B; Commit median 0.501543562 s | PUBLIC-KEPT foundation; target still missed | [chain-1](chain-1-results.md) |
| [x] | SMALL-02 — selected FULL fingerprint cache | **54,562,816 B**; Commit median 0.544395209 s | PUBLIC-KEPT foundation; storage/Commit tradeoff | [selected-full-1](selected-full-1-results.md) |
| [x] | SMALL-03 — removed-name / moved-file base discovery | **50,372,608 B**; Commit median 0.580516875 s | PUBLIC-KEPT foundation; unique bounded removal hints, no fabricated inode identity | [removed-base-1](removed-base-1-results.md) |
| [x] | SMALL-04 — compact fingerprint references | **50,368,512 B**; Commit median 0.581328645 s | PUBLIC-KEPT foundation; only 4,096 B allocated gain versus prior candidate | [compact-candidate-1](compact-candidate-1-results.md) |
| [x] | SMALL-05 — retain FULL candidate cache across admissions | **49,319,936 B**; Commit median 0.571987792 s | PUBLIC-KEPT current foundation; allocation improves 1,048,576 B but content packs only 276,862 B versus SMALL-04 | [retained-candidate-1](retained-candidate-1-results.md) |
| [x] | SMALL-06 — session-local selected DELTA candidates | **50,364,416 B**; Commit median 0.621925063 s; content packs **1,854 B larger** than removed-name candidate | REJECTED/reverted; a small allocation difference did not justify content/CPU regression | [selected-chain-1](selected-chain-1-results.md) |
| [x] | SMALL-07 — retained DELTA candidates | **50,360,320 B**; Commit median 0.649245063 s; content packs **326,391 B larger** than retained FULL candidate | REJECTED/reverted; isolated matching example did not improve whole product | [retained-chain-1](retained-chain-1-results.md) |
| [x] | SMALL-08 — recent128 candidate ring | No eligible candidate in the fixed original-family diagnostic | REJECTED for that tested policy/sample; does not prove all broader discovery ineffective | [Disposition](followup-disposition.md), [results summary](storage-optimization-results.md) |

### Larger-file CDC — priority domain 3

| Done | ID / approach | Scope and measured result | Decision / limitation | Evidence |
| --- | --- | --- | --- | --- |
| [x] | CDC-01 — try remaining overlap hints | Same twelve expensive lockfile targets: **602 B saved from 144,549 record B** | REJECTED as the main direction; no full-family extension of this policy | [Overlap report](experiments40/cdc/report.md) |
| [x] | CDC-02 — one similarity candidate, fixed sample | Same twelve targets: **70,745 B potential record saving**; all selected candidates outside overlap hints | SUPERSEDED by chronological family proof; not extrapolated into a Store result | [Similarity sample](experiments40/cdc/similarity-report.md) |
| [x] | CDC-03 — chronological ten-state lockfile family | 249 identities; record sum **1,123,468→571,573 B**; native packs **551,895 B smaller** | OFFLINE-KEPT; 12 worsened records and 79,971 B of new FULL records included; all 735 native objects authenticate | [Ten-state family](experiments40/cdc/family-report.md) |
| [x] | CDC-04 — same policy across full157 | 1,347 lockfile identities; native packs **8,120,302→7,211,036 B**, net **909,266 B** | OFFLINE-KEPT; all 3,221 native objects authenticate; 130 new FULLs included; discovery cost/public latency unqualified | [Full157 CDC](experiments40/full157/cdc/report.md) |

### Framing and indexes — priority domain 6

| Done | ID / approach | Scope and measured result | Decision / limitation | Evidence |
| --- | --- | --- | --- | --- |
| [x] | PACK-01 — four-byte SmallContent group starts | Ten states: **398,604 pack B saved**, exactly predicted | OFFLINE-KEPT; all 514 packs restore byte-for-byte | [Framing report](experiments40/framing/report.md) |
| [x] | PACK-02 — derive redundant record lengths | Further **265,736 B**; combined ten-state framing **664,340 B**; full157 framing **1,507,960 B** | OFFLINE-KEPT; full hashes/frames retained; 41 malformed-input/dependency checks PASS; versioned product reader needed | [Framing](experiments40/framing/report.md), [full157](40mb-offline-full157-results.md) |
| [x] | INDEX-01 — VACUUM existing schema | Ten-state disposable copy: logical **49,213,440→48,783,360 B**, **430,080 B reduction** | MEASURED-OFFLINE control, not an online optimization; no free-page backlog and no pack-table saving | [Index study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/index/report.md) |
| [x] | INDEX-02 — varint locator encoding | Compact index **3,702,784→3,588,096 B**, **114,688 B saving** | DEFERRED; small benefit, no product path qualification | [Index study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/index/report.md) |
| [x] | INDEX-03 — combine group/record integers | Compact index **3,702,784→3,715,072 B**, **12,288 B larger** | REJECTED | [Index study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/index/report.md) |
| [x] | INDEX-04 — one packed integer locator | **86,016 B saving** versus compact index; adds a 42-bit pack ceiling/constraint implications | DEFERRED; insufficient justification for new restrictions | [Index study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/index/report.md) |
| [x] | INDEX-05 — omit canonical length | Optimistic compact-index model **303,104 B smaller** | REJECTED as a direct implementation; length supports preallocation, authentication and closure checks | [Index study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/index/report.md) |
| [x] | INDEX-06 — remove inode/wrapper locator rows | Optimistic compact-index reduction **1,896,448 B** for both sets, before replacement rows | STUDY-ONLY projection, superseded by actual META-01/02/03 inventories; never add it again | [Index study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/index/report.md) |

### Matcher, scope and root-cause studies — priorities 7–8 / supporting evidence

| Done | ID / approach | Finding | Decision / limitation | Evidence |
| --- | --- | --- | --- | --- |
| [x] | MATCH-01 — Git COPY/INSERT versus current prefix on identical bases | Current prefix wins **4/5 original pairs**; Git wins fifth by **49 B** | No wholesale matcher replacement justified by this sample; not a universal codec ranking | [Matcher results](git-matcher-results.md) |
| [x] | STUDY-01 — ten-state whole SmallContent/Git join | Same 33,217 objects: LayerFS frames **433,886 B above Git entries** | Valid ten-state fact; superseded for full157 prioritization by STUDY-02 | [Content study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/content/report.md) |
| [x] | STUDY-02 — full157 whole SmallContent/Git join and caps | Same 75,398 objects: frames **11,487,177 B above Git**; **9,254,960 B** of FULL frames associated with structurally ineligible predecessors | Supports NEXT-02; capped population is not a savings claim. Opposing FULL/DELTA choices fully reconciled | [Full content attribution](experiments40/history-deltas/content/report.md) |
| [x] | STUDY-03 — cross-CDC-to-small cohort | Six direct targets totaled **112,093 FULL frame B** in the earlier diagnostic | DEFERRED as the main target-closing route; not a bound on descendants or every cross-representation opportunity | [Consolidated outcome](storage-optimization-results.md) |
| [x] | STUDY-04 — timestamp diversity / missing persistent tree | Only four metadata-root values; existing code already has persistent local tree updates. Every prior leaf had at least one changed/deleted pair | Reject timestamp stripping and an assumed missing copy-on-write mechanism as easy fixes; no new encoder tested | [Metadata study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/metadata/report.md) |
| [x] | STUDY-05 — metadata repetition / hybrid model | Full157 **9.886 copies/value** versus ten-state **1.579**; hybrid raw model changes sign with history length | Supports history-aware experiments, not promised hybrid savings | [Scaling diagnosis](experiments40/full157/metadata/scaling-diagnosis.md) |

### Complete combinations and qualification history

| Done | ID / combination | Measured outcome | Status / scope | Evidence |
| --- | --- | --- | --- | --- |
| [x] | VALID-01 — supported retained candidate, ten states | **49,319,936 B**, 10 Created, same-Store verification and cleanup PASS; Commit median/sum **+17.70%/+23.64%** versus original v0.1.5 | PUBLIC-KEPT with disclosed regression; not release approval | [Public ten result](retained-candidate-1-results.md) |
| [x] | VALID-02 — supported retained candidate, full157 | **134,246,400 B**, 157 Created, original same-Store verification and cleanup PASS; paired median **+31.13%** versus released control | PUBLIC-KEPT with disclosed regression | [Public full157](retained-full157-results.md) |
| [x] | VALID-03 — compact D + framing B + CDC, ten states | Matched compact **48,783,360→41,648,128 B**; **7,135,232 B saving** | OFFLINE-KEPT; 1,648,128 B above 40 MB; metadata/content identity checks PASS | [Compact10](40mb-experiment-results.md) |
| [x] | VALID-04 — exact same compact policies, full157 | Matched compact **129,937,408→107,958,272 B**; **21,979,136 B saving** | OFFLINE-KEPT evidence; all 157 original oracles PASS; **51,585,024 B above Git** | [Compact157](40mb-offline-full157-results.md) |
| [x] | VALID-05 — add depth-one metadata deltas, full157 | Matched chronological **108,081,152→98,668,544 B**; **9,412,608 B saving** | Latest OFFLINE-KEPT; all 157 original oracles PASS; **42,295,296 B above Git** | [Current result](history-scaling-and-metadata-deltas.md) |
| [x] | VALID-06 — earlier premature chain full157 | Performance allocation **151,031,808 B**; historical verification interrupted at **150/157** | INCOMPLETE; forced shutdown/diagnostic cleanup, container removed; not final proof, superseded by VALID-02 for the retained product | [Correction](followup-disposition.md) |
| [x] | VALID-07 — public stride3 / matched Git53 | **100,700,160B vs49,332,224B allocated**;53Created, all53 original states and cleanup PASS | PUBLIC-KEPT development foundation;2.94× shorter phase wall versus historical157 workload, not structural format qualification | [Stride3 comparison](stride3-comparison-results.md) |

## 4. Checklist for every new experiment

Copy this checklist into the next run's protocol, then link its result back into the ledger. A completed negative result is valuable; do not change the policy or numerical target after seeing its result.

- [ ] Assign a stable experiment ID, owner and one concrete hypothesis; link the measured population motivating it.
- [ ] Record exact input/source/fixture hashes and baseline applicability. Use a fresh output path; preserve all earlier artifacts.
- [ ] Freeze one candidate-selection/encoding policy and its fallback before encoding; no unrecorded parameter sweep.
- [ ] Declare base roles, chronology, depth, summed decoded/encoded closure, live buffers and static codec scratch.
- [ ] Establish a matched control with the same input, ordering, grouping, compaction and measurement boundary.
- [ ] Trace the real source path or reproduce exact existing encodings/provenance where needed; no invented predecessor facts.
- [ ] Count actual selected FULL/DELTA winners after complete-group comparison. Charge physical bases once, replacement pages and all index/allocator costs.
- [ ] Propagate changed base choices chronologically; include worse descendants, cap fallbacks and new FULL records.
- [ ] Verify reconstruction/authentication, role and boundary failures, chronology and missing/corrupt dependencies.
- [ ] Measure complete layout allocation when warranted; label raw bytes, frames, pack BLOBs, logical DB bytes and allocated bytes distinctly.
- [ ] Verify every affected original state and retain the exact verifier/oracle custody result.
- [ ] Record extra lookup/read/encode work and actual timing/resource scope; diagnostic microtimings are not Save/Commit latency.
- [ ] Record status, rejection/acceptance rationale, limitations and next action. Update scorecard only with a comparable result.
- [ ] Seal artifacts and checkpoint source/protocol/report; retain failed attempts and rerun reasons.

## 5. Product integration and acceptance checklist — not yet complete

Offline wins do not authorize a numerical product or release PASS. The supported product and the experimental format remain separate until this work is performed.

- [ ] Select the supported design from full-history evidence; do not promote a ten-state winner solely because it is close to 40 MB.
- [ ] Write the concrete format/capability fence and old-reader/nonpromoting-open behavior.
- [ ] Implement transactional scope/serial allocation, branch behavior, foreign-scope imports and required migration/rollback semantics.
- [ ] Implement authenticated readers/writers and prove complete memory/work ownership on the real public path.
- [ ] Preserve shared Init/Commit construction, POSIX/links/modes/symlinks, #95 comparison reuse, #98 coalescing/handoff and bounded spill reads.
- [ ] Declare prospective speed/resource criteria before measurement; the earlier 10% criterion was a working criterion, not an owner-approved release gate.
- [ ] Build and seal only affected product artifacts using the existing entrypoints/caches.
- [ ] Run one fresh public ten-state save/Commit campaign for the actual integrated candidate; freeze census/allocation before same-Store verification and cleanup.
- [ ] Run the required public full157 campaign after the candidate is ready; compare only applicable matched controls and Git's proper storage boundary.
- [ ] Publish per-state timings, sums/medians, historical-read results, CPU/RSS, staging/spool, setup/build/cleanup and failure scopes.
- [ ] State target status and regressions honestly; an allocation win alone does not establish acceptable latency or release readiness.
- [ ] Update issue #100 and this tracker with the qualified outcome, linking exact evidence.

Historical verification completed for VALID-05; STRUCT-VALID-01 now independently passes the same scope: **157 states / 904,143 path states / 4,936,693,030 logical bytes**, exact original-oracle and custody PASS. That qualifies the offline copy/read path only; it does not check off the public integration tasks above.

## 6. Reusable experiment report entry

```markdown
### <EXPERIMENT-ID> — <approach>

- Priority / related backlog item:
- Date / owner:
- Status: TODO | PUBLIC-KEPT | OFFLINE-KEPT | MEASURED-OFFLINE | DEFERRED | REJECTED | REFERENCE-ONLY | STUDY-ONLY | INCOMPLETE | SUPERSEDED
- Hypothesis and measured motivation:
- Exact source, fixture, input Store and hash:
- Protocol / policy change / unchanged variables:
- Bounds, base roles, authentication and fallback:
- Baseline and measurement boundary:

| Metric | Matched control | Candidate | Difference |
| --- | ---: | ---: | ---: |
| Complete allocated bytes, if measured | | | |
| Logical database bytes | | | |
| Content pack bytes | | | |
| Metadata pack bytes | | | |
| Index/allocator/remaining bytes | | | |
| Selected FULL / DELTA count | | | |
| Extra read/lookup/encoding work | | | |
| Timing / resource metric with scope | | | |

- Complete-base and changed-descendant accounting:
- Verification: states, paths, logical bytes, authentication, negative checks, oracle seals:
- Failed attempts / rerun justification:
- Decision: retain / reject / defer, and why:
- Limitations / product qualification still missing:
- Next action:
- Protocol / raw result / report / manifest / checkpoint links:
```

## 7. Evidence and checkpoints

| Checkpoint | Contents |
| --- | --- |
| `5636fbc43` | Ten-state compact prototype, protocols and report |
| `a4c8de3dc` | Fixed-policy full157 extension and exact original-oracle verifier |
| `56e507d24` | Full content attribution and verified depth-one metadata-delta improvement |

Raw evidence roots, including immutable unsupported copies, selected frames and complete manifests:

```text
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas
```

Maintenance rule: preserve old rows and evidence; append changed-policy experiments. Re-rank the queue only when new evidence changes expected value or dependencies. Do not mark an untested idea completed because a raw-byte model looks favorable.

## #103 native integration checkpoint — 2026-09-10

[Implementation progress](../issue103/integration-progress.md) records native
schema-10/pack-v4 SmallContent framing and focused correctness checks. The rest
of the compact namespace, pooled metadata and whole-file/slice product path
remains unfinished. No integrated stride3/full157 allocation exists yet; the
66 MB product target is NOT QUALIFIED. Recorded offline results above are
unchanged. Stride3 remains the first permitted storage run after integration.

### #103 continuation 2 — native namespace foundation

[Progress and exact logs](../issue103/integration-progress.md#native-namespace-checkpoint--continuation-2)
now cover scoped inline namespace writes from empty initialization, ordinary
Commit/workspace paths, durable serial reservations and passing affected suites.
Directory-source initialization, compact reconciliation and the metadata/content
physical optimizations remain unfinished. No integrated history size is measured;
66 MB remains NOT QUALIFIED. Offline reference rows remain unchanged.


### 2026-09-10 — issue103 continuation 3 and revised scope

The user narrowed execution to full promoted-method integration followed by
**stride3 optimization/verification only**. Full157 is outside the revised scope;
no broad #102 campaign or release qualification is implied. Native directory
initialization and compact reconciliation now pass focused development checks.
Schema-10 metadata pack v5 integrates bounded 16-edge/128-KiB canonical delta
chains with authenticated intermediate bases and unchanged legacy contracts.
Shared metadata-value groups and whole-file graphs/native slices remain pending;
stride3 has not run and no integrated allocation is claimed. See
[issue103 integration progress](../issue103/integration-progress.md) for exact
commands, retained failures and evidence. Recorded offline/Git references are
unchanged and are not fresh product measurements.


### 2026-09-10 — issue103 shared metadata groups integrated

Schema-10 public initialization/admission/Commit now uses pack-v6 pooled inode
leaves and bounded metadata deltas. Append-only product-owned ordinals preserve
canonical IDs; full group digests and every intermediate leaf are authenticated.
A bounded disposable macOS lookup index replaces permanent per-value indexes and
rebuilds from published groups on reopen. Sharing, rollback/cache disposal,
corrupt pool dependencies and public lifecycle tests pass. Exact policy, memory/
work limits, retained failures and test commands are in
[issue103 integration progress](../issue103/integration-progress.md).
Whole-file graphs/native slices remain pending. **Stride3 is not yet measured**;
full157 and broad #102 execution remain outside the revised scope.


### 2026-09-10 — issue103 whole-file product compaction implemented

The public `LayerStackStore::compact_into` operation and `layerfs-store-compact`
binary now write authenticated whole-file prefix graphs and native chunk slices
using only already published Store content. Selection reuses product min-hash
signatures; canonical IDs/logical records are preserved, sources are retained,
and destination publication follows full intrinsic verification. Explicit
compaction cost, temporary storage and resources must accompany the final size.
The content container and four-KiB final layout are integrated with public reads,
future writes/Commit, fork, reopen and repeated compaction. The focused development
suites pass 342 tests (4 existing ignored), including corruption, quota failure
and publication recovery. See [issue103 integration progress](../issue103/integration-progress.md)
for exact policy, limits, commands and retained failed attempts.
**Stride3 remains unmeasured** pending runner/qualified-build/live-Exec gates.
No full157 or broad #102 campaign has run under the revised scope.


## 2026-09-10 — integrated stride3 qualification complete

The native schema10 candidate `80bc489281735c889a4d62ed576135586be3365b` integrates
scoped-inline namespaces, authenticated physical metadata-value groups/deltas,
compact framing, whole-file prefix graphs/native slices and4KiB final SQLite
layout. Public initialization/writes/Commit/reopen/fork use the product format;
`LayerStackStore::compact_into` explicitly selects from already published Store
content without Git/oracle dependencies. Focused342-test proof, qualified live
Exec/FUSE proof,53 direct saves and all53 original-oracle verifications pass.

| Stride3 boundary | Allocated B | Status |
| --- | ---: | --- |
| Integrated before compaction | 65,056,768 | 53 Created,0 UpToDate,0 presentation failures |
| Integrated after public compaction | **46,202,880** | **53/53 original states PASS** |
| Historical selected offline53 | 54,382,592 | Unchanged reference |
| Historical matching Git53 | 49,332,224 | Unchanged reference; no fresh speed control |

Complete physical accounting explains the8,179,712B offline gap: content packs
save8,091,705B, other payload saves267,042B, and SQLite nonpayload allocation
increases179,035B. All59,768 Small IDs/lengths and224 whole-owner IDs/lengths
match the offline reference. No workload/state was omitted for size.

Compaction339.132858s; peak RSS135,069,696B; sampled temporary peak281,907,200B,
with source charged separately. All publication/cleanup receipts pass. Historical
access11+11 cases pass; the cold6421-byte range decodes17,029,550B, so low read
amplification and release readiness are not claimed. Full157/66MB qualification
and broad #102 remain outside the revised scope. Exact identities, commands,
frozen Store and retained failures: [issue103 final report](../issue103/stride3-integrated-results.md).


## 2026-09-10 — promoted full157 qualifies the66MB target

The owner authorized full157 after stride3 completion. Product seal and all
executables remain identical to the qualified stride3 candidate; runner-only
registration/custody changes are frozen at`786d29575b1b7cf1123b5f9b8c97f1e4610c2bab`.

| Full157 boundary | Allocated B | Outcome |
| --- | ---: | --- |
| Product before compaction | 83,935,232 |157 Created,0 UpToDate,0 presentation failures |
| **After supported compaction** | **55,476,224** | **157/157 original oracles PASS** |
| Selected offline157 reference | 65,957,888 | Historical, unchanged |
| Matching recorded Git157 | 56,373,248 | Historical storage comparison |

Complete allocation saves10,481,664B against offline and897,024B against Git.
Content payload saves9,912,779B, other payload991,303B, while SQLite nonpayload
allocation adds422,418B. All75,398 Small and523 whole-owner IDs/lengths match the
offline reference. Product selection adds6378 Small prefixes, saving10,055,283B;
whole records cost142,366B more under the bounded product-owned graph. No states
were omitted to obtain the size.

Compaction626.313062s; peak RSS131,792,896B; sampled temporary407,638,016B plus
preserved source. All publication/cleanup and157-state oracle checks pass. All11
historical-access cases and11 verification runs pass15-second contracts; cold
range decoding22,216,028B for6421B remains a limitation. Exact sources, costs,
commands and immutable evidence: [full157 result](../issue103/full157-integrated-results.md).
The full #102 campaign and release remain incomplete.


## 2026-09-11 — explicit compaction dropped from the product

Owner decision: compaction is too slow and not useful enough for the intended
real workloads; existing ordinary deduplication and compression already meet the
owner's needs. The full157 saving remains valid historical evidence, but required
626.313062 seconds of additional API time. A smaller archived Store does not
justify that cost as a product step.

The local removal deletes the public compaction API/CLI and benchmark producer;
ordinary schema10 content/metadata admission stays enabled. Keep authenticated
read compatibility for previously compacted Stores. Future work must improve
ordinary writes without background rewrites or hidden compaction. The old 66 MB
compacted result does not automatically become an uncompacted acceptance gate.

See [the current decision and validation boundary](../compaction-removal.md).
This supersedes earlier compaction continuation items in this ledger, preserves
all original measurements, and does not close unrelated latency/storage targets.

# Issue #100 optimization checklist and experiment ledger

Updated: **2026-09-10**. Working branch: `codex/issue100-40mb-experiments`.

This is the working index for priorities, completed approaches, results, rejected ideas and remaining qualification. Update this file after each experiment. Keep the linked raw reports and manifests immutable; add a new run when the policy changes.

**Current assessment:** historical reuse is the largest opportunity. The best verified offline full157 copy is **98,668,544 B**, still **42,295,296 B / 75.03% above recorded Git**. A useful intermediate saving is not completion. The next experiment is **NEXT-01: bounded metadata delta chains**.

## 1. Current scorecard — preserve the measurement boundary

Decimal bytes throughout. The ten-state objective is **40,000,000 B**. Full157 is a separate long-history comparison to Git; the ten-state size target does not apply to it.

| Result / boundary | Ten selected states | Full157 | Status and evidence |
| --- | ---: | ---: | --- |
| Recorded Git allocation | 38,223,872 | 56,373,248 | Existing baselines; [ten-state control](ten-snapshot-baselines.md), [full157 Git audit](experiments40/full157/git-baseline/report.md) |
| Latest measured supported product | 49,319,936 | 134,246,400 | Public save/Commit + same-Store verification; [ten](retained-candidate-1-results.md), [157](retained-full157-results.md) |
| Same-policy compact D + framing B + CDC offline copies | 41,648,128 | 107,958,272 | Both measured; [compact10](40mb-experiment-results.md), [compact157](40mb-offline-full157-results.md) |
| Matched offline controls for that compact experiment | 48,783,360 | 129,937,408 | Equally VACUUMed controls; do not blend with public allocation |
| Latest offline copy with depth-one metadata deltas | Not run | **98,668,544** | [History-delta report](history-scaling-and-metadata-deltas.md); 157 original-oracle checks PASS |
| Matched chronological FULL control for depth-one deltas | Not run | 108,081,152 | Same canonical objects, chronology, groups and pack membership |

The latest ten-state and latest full157 winners use different metadata policies. Do not infer a scaling rate by comparing those two best numbers. The same-policy compact pair, 41,648,128→107,958,272 B, remains the measured scaling comparison. Git's full157 live allocation is now 56,197,120 B with unchanged apparent sizes and pack contents; keep that later observation separate from the recorded 56,373,248 B.

| Remaining full157 category gap | Current offline B | Recorded Git B | Gap B |
| --- | ---: | ---: | ---: |
| Content packs / blob entries | 66,190,736 | 46,982,533 | **19,208,203** |
| Metadata packs / trees and commits | 25,624,588 | 5,007,335 | **20,617,253** |
| Indexes, structures and allocation | 6,853,220 | 4,383,380 | **2,469,840** |
| **Total** | **98,668,544** | **56,373,248** | **42,295,296** |

## 2. Optimization queue — highest to lowest priority

An unchecked item means the proposed experiment has not completed. Priority is based on measured relevance and implementation scope, not a promised saving. Validation and product qualification in section 5 are mandatory gates, not lower-priority optional work.

### Priority 1 — NEXT-01: bounded metadata delta chains

**Evidence:** META-05 saved 9,412,608 B of matched complete-copy allocation, but **4,060 leaf targets fell back to FULL because their selected origin was already DELTA**. Current metadata still exceeds Git by 20,617,253 B.

**Hypothesis:** allowing a short, explicitly bounded metadata chain can retain successive small differences instead of periodically writing FULL pages. Keep canonical metadata, compact identities, candidate selection, matcher, codec and complete-group comparison fixed; change only permitted base roles/chain policy.

- [ ] Write the fixed protocol: maximum edges, summed decoded closure including target, retained encoded capacity, live buffers, authentication, chronology and fallback.
- [ ] Implement the smallest diagnostic reader/writer extension; use one policy, not a depth sweep.
- [ ] Simulate the complete chronological metadata graph, charging every FULL reset, base and changed descendant.
- [ ] Assemble matched complete copies and compare allocated bytes, not only delta records.
- [ ] Verify all canonical metadata, negative dependency/boundary cases and all 157 original state oracles.
- [ ] Record decision, work/resource costs and the exact new result under a new experiment ID.

**Stop/reject if:** complete storage does not improve, dependencies violate declared bounds, or apparent wins depend on omitted bases/old representations. The 4,060 fallbacks are a population, not estimated savings.

### Priority 2 — NEXT-02: newest eligible SmallContent ancestor

**Evidence:** full157 has **9,254,960 B of FULL frames** whose same-path predecessor cannot be extended within current bounds; 7,655,946 B hits only the eight-edge limit. These are structural eligibility facts, not delivered-hint telemetry or proven savings. See STUDY-02.

**Hypothesis:** one newest existing predecessor ancestor that fits the current limits may avoid a FULL reset without raising the limits or adding a new physical grammar.

- [ ] Freeze a small byte-ranked sample across the actual capped file families, with exact predecessor provenance.
- [ ] Compare actual FULL against one newest eligible ancestor using the same codec and complete reference cost.
- [ ] If useful, apply one fixed policy chronologically to complete original families, including larger accumulated deltas and worse descendants.
- [ ] Measure the complete copy and all affected historical content; retain the current format/depth/byte ceilings.
- [ ] Record results and whether a full157 integration experiment is justified.

**Stop/reject if:** older-base accumulation erases the initial savings. Do not repeat the old FULL-anchor failure while counting only the attractive targets.

### Priority 3 — NEXT-03: large-file history attribution and one targeted mechanism

**Evidence:** optimized native packs are 7,211,036 B versus 2,006,385 B for the 523 corresponding large Git versions, a **5,204,651 B** difference. Existing lockfile similarity recovered 909,266 B in full157, so the remaining large-file gap needs separate attribution.

- [ ] Rank actual full157 families by physical retained bytes, counting shared chunks once.
- [ ] Identify whether the dominant owner is base locality, representation boundaries, depth/closure resets or matching/compression quality.
- [ ] Freeze one family and one mechanism to test; compare complete retained history with identical inputs/settings.
- [ ] Measure decoding, signature construction, candidate trials and public-path implications alongside space.
- [ ] Extend only after the fixed-family evidence supports it; record rejected outcomes too.

**Conditional alternative:** bounded whole-file physical encoding for medium-sized files. It needs explicit grammar, window/memory accounting and random-read analysis; changing a cutoff alone is not an implementation.

### Priority 4 — NEXT-04: better alternative when a valid SmallContent predecessor compresses poorly

**Evidence:** the writer consults its retained FULL candidate cache only when the predecessor is unavailable. It does not try that candidate after an eligible predecessor yields a poor delta. The full157 same-object frame gap is 11,487,177 B, but this specific restriction's contribution is unmeasured.

- [ ] Identify a fixed real sample where a valid predecessor is weak and a different already-selected base demonstrably exists.
- [ ] Compare at most one additional candidate, preserving the same complete cost and read bounds.
- [ ] Charge additional authentication, reads and encoding work; quantify net whole-history effects before broadening the search.
- [ ] Record whether this warrants implementation or should remain deferred.

**Do not repeat:** the rejected selected/retained DELTA-cache variants without evidence that this policy differs in a material way.

### Priority 5 — NEXT-05: compact-ID/shared-inode-value hybrid

**Evidence:** full157 inline leaves repeat each distinct inode value 9.886 times. A separate value-CAS representation could share those values, but introduces approximately 89,576 CAS/index rows and replaces inline data with 32-byte references. The 27.53 MB raw-byte model is not a compressed-storage forecast.

- [ ] Revisit only after NEXT-01 establishes what bounded page deltas recover.
- [ ] Encode a matched hybrid with real page repartitioning, compression, object IDs and all new index rows.
- [ ] Compare complete metadata-plus-index and whole-copy costs on both ten-state and full157 histories.
- [ ] Reject if random hash/reference or index costs exceed the sharing benefit.

### Priority 6 — NEXT-06: remaining pack/index overhead

**Evidence:** simple compaction/locator experiments are sub-megabyte on the ten-state case. Compact framing already forms part of the current offline candidate. Only 2,469,840 B of the current full157 gap is in the entire residual category.

- [ ] Identify a specific remaining redundant physical field or index structure, with its actual population and byte ceiling.
- [ ] Compare matched layouts including required authentication, length checks, lookup structures and allocator state.
- [ ] Integrate only if the measured benefit justifies the format/read complexity.

**Not pending quick wins:** deleting a redundant object index or a free-page backlog; neither exists in the inspected source. Do not truncate content hashes or remove preallocation/authentication length checks to improve a number.

### Priority 7 — NEXT-07: reverse encoding / physical replacement

**Evidence:** Git has useful later-base graphs, but opposing FULL/DELTA assignments cancel large apparent target savings. No complete replacement graph or reclaimed-store result has been measured for LayerFS.

- [ ] First quantify a whole-family/whole-graph opportunity including every newly FULL base and changed descendant.
- [ ] Specify locator publication, dependency retention, concurrent-reader lifetime, crash recovery and real old-byte reclamation.
- [ ] Run a bounded original-family experiment before considering a general repacker.

**Deferred:** this is a substantial storage-lifecycle change. Appending another encoding without reclaiming the original is not a saving.

### Priority 8 — NEXT-08: codec / matcher replacement

**Evidence:** the existing prefix encoder won four of five identical-base original-file tests; Git's matcher won the fifth by 49 B. That small study is not a universal codec ranking, but provides no basis for a wholesale replacement.

- [ ] Revisit only if matched-base measurements on a specifically attributed residual population demonstrate a codec deficit.
- [ ] Keep base graph, input bytes and compression parameters equal during that test.
- [ ] Charge complete encoded records, reconstruction, dependencies and format/ownership costs.

## 3. Experiment ledger — completed approaches and their disposition

Rows are grouped by the priority domain above. Within a domain, retained approaches appear together with their rejected alternatives and supporting studies. IDs remain stable; append a new row for a changed policy rather than overwriting a rejected result.

**Status key:** `PUBLIC-KEPT` = measured runtime foundation, not release approval; `OFFLINE-KEPT` = useful diagnostic component, not product integration; `REJECTED` = tested but not retained; `MEASURED-OFFLINE` = measured diagnostic/control without a retained optimization; `DEFERRED` = measured or studied, with further work postponed; `STUDY-ONLY` = attribution/model without an alternative encoding; `INCOMPLETE` = unusable as final proof; `SUPERSEDED` = valid earlier result replaced by later evidence. A checked experiment means execution/reporting is complete, not that the idea won.

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

Current verification already completed for VALID-05: **157 states / 904,143 path states / 4,936,693,030 logical bytes**, exact original-oracle and custody PASS. That qualifies the offline copy/read path only; it does not check off the public integration tasks above.

## 6. Reusable experiment report entry

```markdown
### <EXPERIMENT-ID> — <approach>

- Priority / related backlog item:
- Date / owner:
- Status: TODO | PUBLIC-KEPT | OFFLINE-KEPT | MEASURED-OFFLINE | DEFERRED | REJECTED | STUDY-ONLY | INCOMPLETE | SUPERSEDED
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

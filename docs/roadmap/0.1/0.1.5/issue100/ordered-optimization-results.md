# Ordered optimization results: metadata, content, then layout

2026-09-10. The three requested priorities were investigated in order, using53 states first and applying the same retained storage policies to157 states.

**Selected archive/reference layouts now measure54,382,592 B on 53 states and 65,957,888 B on 157 states.** Relative to the prior improved offline format, this saves5,378,048 B and 13,832,192 B respectively. The storage gains carry significant cold-read amplification and are not a hot-filesystem or release recommendation.

**Verification PASS:** all **53 original states / 306,861 path-states / 1,676,767,835 logical bytes**, and all **157 original states / 904,143 path-states / 4,936,693,030 logical bytes** match through the selected candidates' actual readers. Both candidate hashes remain unchanged. Original oracle hashes match the saved fixture seals. [53 proof](experiments40/ordered-optimization/content-proof-53/result.json), [157 proof](experiments40/ordered-optimization/content-proof-157/result.json), [157 oracle custody](experiments40/ordered-optimization/content-proof-157/oracle-custody.json).

## Results in requested order

| Complete allocated database | 53 states | 157 states | Decision |
| --- | ---: | ---: | --- |
| Starting improved offline format |59,760,640 B|79,790,080 B|Matched starting point|
| Metadata V1: separately indexed pooled values |66,527,232 B|83,488,768 B|Reject: added indexes outweigh payload savings|
| **Metadata V2: authenticated physical value groups** |**57,974,784 B**|**71,970,816 B**|Retain for storage experiments; cold-read tradeoff|
| **Then whole-file content graph plus native slice adapters** |**54,382,592 B**|**65,957,888 B**|Archive/reference-only; severe small-read amplification|
| Then1 KiB SQLite-page trial |54,386,688 B|65,884,160 B|Do not adopt:53 grows;157 saves only 73,728 B with deeper B-trees|
| **Selected final layout, retaining4 KiB pages** |**54,382,592 B**|**65,957,888 B**|No production implementation or qualification|
| Recorded matching Git |49,332,224 B|56,373,248 B|Same selected histories|
| **Selected gap to Git** |**5,050,368 B /10.24%**|**9,584,640 B /17.00%**|Gap remains|

The proposed≤55 MB stride3 milestone is met. The proposed≤65 MB full157 milestone is missed by 957,888 B. These are engineering investigation milestones, not release gates; neither number erases the read-cost problem.

The53→157 storage increment falls from 20,029,440 B to11,575,296 B. Git adds7,041,024 B across those histories: the incremental-byte ratio improves from 2.84× to1.64×. This is a measured workload comparison, not an asymptotic complexity claim.

## 1. Historical metadata

The independent census found306,115 physical inode-value occurrences but66,529 distinct values on 53 states (4.60 copies/value), and 885,543 occurrences /89,576 distinct values on 157 states (9.89 copies/value). Raw repetition motivated a test; it was not treated as compressed savings.

V1 replaced repeated73-byte inline values with physical ordinals and retained all original canonical leaf IDs. It stored each distinct value as an extra indexed CAS object, plus an ordinal table and full-hash uniqueness index. Actual packed metadata became smaller, but those three lookup structures made the complete databases **6,766,592 B /3,698,688 B larger**. Both failed policies remain preserved.

A separately frozen V2 keeps the exact V1 packs, ordinals and metadata delta graph, but stores the pool as authenticated physical groups. It needs only 404/543 group-catalogue entries, with full SHA256 group digests, rather than per-value CAS and ordinal index rows. Original metadata leaves still reconstruct and authenticate against their full32-byte canonical IDs. No content bytes or typed SQL history changes.

Actual complete-copy savings against the starting layout are **1,785,856 B /7,819,264 B**. All original metadata and physical values,54/158 namespace states including genesis, physical catalogue coverage, SQL/constraint preservation and six malformed-input checks pass. Independent original53 and 157 state oracles pass with unchanged candidate hashes.

**Read cost:** hash-sorted values scatter across physical groups. A cold metadata leaf dependency can require4,725,604 B /5,785,404 B of decoded groups, involving as many as 288/353 pool groups. The128 KiB canonical dependency limit does not include that decompression amplification. The executed pool cache stores4 MiB of decoded bodies and up to another 4 MiB of copied record payload; this is separate from the canonical and encoded-pack caches. No reader was changed during verification to hide this cost.

[Metadata report](experiments40/ordered-optimization/metadata/report.md), [population census](experiments40/ordered-optimization/metadata-population.json), [53 cold-work audit](experiments40/ordered-optimization/metadata-cold-53.json), [157 cold-work audit](experiments40/ordered-optimization/metadata-cold-157.json).

## 2. Content history

The prior all-blob encoding reference was turned into an actual complete Store representation. Existing SmallContent IDs stay intact. New authenticated whole-file objects represent224/523 large regular-file versions. All1,998/3,221 original native chunk IDs become authenticated slices of those retained objects; every chunk has a proven range, so no fallback FULL chunks were required. Existing file roots, metadata, allocator, pool catalogue and typed SQL remain unchanged.

The actual matched Git base graph selects each regular-file prefix base. Every FULL object, delta reference, large-file index row and native slice record is charged. There is no subtraction of isolated target savings or hidden retention of old content packs.

Content packs decrease48,824,145→45,494,501 B on 53 states and 56,367,207→50,710,265 B on 157 states. Complete database savings are **3,592,192 B /6,012,928 B**, including changes in pack-table allocation and the new whole-file index rows.

All59,768/75,398 original SmallContent objects,224/523 new large objects and 1,998/3,221 original native identities authenticate. Complete original-ID retention, physical coverage, SQLite integrity/FKs, schema and noncontent SQL/payload preservation, and five corruption checks pass.

**The cold-read tradeoff is severe:**

| Actually reconstructed native chunk | Returned bytes | Raw content decoded | Amplification |
| --- | ---: | ---: | ---: |
| Worst ratio selected on 53 states |6,421|15,608,785|2,430.90×|
| Worst ratio selected on 157 states |6,421|23,758,968|3,700.20×|

The largest selected file dependency reaches20.47 MB /29.81 MB cumulative canonical data. In157, the worst-ratio cold diagnostic read fetched1,241,516 B of physical packs and took420.83 ms in the Python prototype. These are scoped diagnostic observations, not public API latency claims. Cached sequential reads can behave differently, but do not remove the cold dependency requirement. This layout should remain an archive/reference result until hot-read behavior is redesigned and measured.

[53 content report](experiments40/ordered-optimization/content/report.md), [157 content report](experiments40/ordered-optimization/content/full157/report.md). The157 internal object verifier grouped adapter checks by owner under a prospective protocol note; encoding, owner selection, cache limits and the parent's original-filesystem verification order remained unchanged.

## 3. Index and physical layout

One fixed native SQLite experiment changed4 KiB pages to1 KiB after a fresh VACUUM. It preserved the full logical schema, every typed row, every encoded pack BLOB, every full hash and every constraint. No locator truncation, canonical-length removal or parameter sweep was attempted.

On53 states, logical bytes grow3,072 and allocation grows4,096. On157 states, allocation falls only 73,728 B (0.112%). The objects index gains an extra B-tree level and grows131,072 B /197,632 B. The pack table also gains a level; overflow-page counts grow12,146→49,321 and 14,363→58,791.

**Decision: retain4 KiB pages uniformly.** The1 KiB full157 copy is preserved as a measured reference, but its tiny saving does not justify adopting the deeper layout without read evidence. This does not alter the selected54.38 MB /65.96 MB results.

Both page experiments pass exact logical schema/row/payload equality, source custody, integrity and FKs. The157 variant also passes cold actual-reader samples for every populated content kind and fresh original states 1,79,157. Full-history correctness is carried by equality with the fully verified4 KiB source; those three reads are not presented as a fresh157-state replay. [Layout report](experiments40/ordered-optimization/index/report.md).

## Evidence and custody

The original public Stores, earlier offline copies and Git baselines remain unchanged. Every new layout is in a fresh output path. Sources, actual databases, executed readers, protocols, selected graphs, comparisons and negative outcomes are retained.

The first full157 metadata verifier invocation had a malformed expected-SHA argument and failed before creating output or decoding candidate objects; the retry read the expected hash programmatically from the sealed result. Content's initial import collided with an existing generic codec module before encoding or Store creation; the local helper was renamed, with the failed source/log retained. Neither event caused an encoding-policy change, a hidden benchmark rerun or evidence replacement.

Raw root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization`.

- Selected53 copy: `content/candidate.sqlite`, SHA256 `dd4a3c7cd2260b487b570cb819f2799ff189c2470382d4481b0f2fd19f3360a3`.
- Selected157 copy: `content/full157/candidate.sqlite`, SHA256 `efacb9b832961ed7a2e675c26f113545622b75df684f9c11a36f3b1a121bec11`.
- [53 original-state proof](experiments40/ordered-optimization/content-proof-53/result.json).
- [Metadata53 proof](experiments40/ordered-optimization/metadata-proof-53/result.json), [metadata157 proof](experiments40/ordered-optimization/metadata-proof-157/result.json).
- [Ordered campaign checklist](ordered-optimization.md), [experiment ledger](optimization-checklist-and-experiment-ledger.md).

No product implementation, public Commit benchmark, issue publication or release was performed. The next design constraint is read amplification; further byte savings should not be mistaken for an acceptable hot filesystem.

# Executed storage experiments toward 40 MB

2026-09-10. The three requested experiments, conditional compact inode identities, and a combined offline layout have completed.

**The combined unsupported diagnostic copy occupies 41,648,128 bytes.** It is 1,648,128 bytes above 40,000,000. The equally VACUUMed unchanged control occupies 48,783,360 bytes; the combined reduction on that matched offline boundary is **7,135,232 bytes (14.63%)**. Both logical size and observed copy allocation have these values.

This is an executed encoding/layout result, not an achieved public LayerFS Store result. The supported measured product still has the previously recorded ten-state allocation of **49,319,936 bytes**. No product source was changed, public candidate built, ten-state performance replayed, or new full157 launched by these experiments. The diagnostic readers and formats are not supported by the Rust product. Online packing, allocator behavior, Commit cost, rollback, compatibility and historical API verification remain implementation/qualification work.

## What was actually measured

| Fixed complete-copy layout | Logical bytes | Observed copy allocation | Reduction vs matched compact control |
| --- | ---: | ---: | ---: |
| Unchanged source, equally VACUUMed | 48,783,360 | 48,783,360 | 0 |
| Metadata D + compact SmallContent framing B | 42,192,896 | 42,192,896 | 6,590,464 |
| D + B + fixed CDC similarity graph | **41,648,128** | **41,648,128** | **7,135,232** |

The source was the retained candidate's post-verification Store, SHA256 `713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4`, matching the saved verification manifest. Original Stores and fixtures remained unchanged. The existing offline compact control was reused after digest verification. Each combined treatment started from its own fresh disposable copy.

Final combined allocation reconciles exactly:

| Physical category | Bytes |
| --- | ---: |
| SmallContent packs, compact diagnostic framing B | 33,955,655 |
| Native CDC packs, fixed similarity graph | 2,707,597 |
| Metadata packs, scoped inline representation D | 2,639,978 |
| **All pack payloads** | **39,303,230** |
| SQLite nonpack bytes, including all indexes and allocator | 2,344,898 |
| Allocation above logical copy length | 0 |
| **Total** | **41,648,128** |

The final inventory has 38,852 objects and 619 packs. The object-index B-tree occupies 1,806,336 bytes; the new allocator occupies 4,096 bytes. These are already included in SQLite nonpack bytes and must not be added again.

## Metadata: the large reduction is demonstrated

Each treatment used actual replacement canonical objects, full-width authenticated content/object hashes, bounded pages, the same pinned codec, and the same offline FULL-only packing and compact-index policy. All eleven original namespace states were compared semantically. Replacement pages and locator rows were included.

| Metadata treatment | Encoded metadata packs | All-object index | Added allocator | Matched subtotal |
| --- | ---: | ---: | ---: | ---: |
| A: current canonical inventory | 7,063,939 | 3,731,456 | 0 | 10,795,395 |
| B: inline inode values | 5,748,683 | 1,998,848 | 0 | 7,747,531 |
| C: B + direct directory mappings | 5,593,831 | 1,814,528 | 0 | 7,408,359 |
| D: C + scoped eight-byte inode serials | **2,639,978** | **1,806,336** | **4,096** | **4,450,410** |

C saves 3,387,036 bytes versus A. Because that is insufficient for the target budget, the conditional D experiment was executed. D saves another **2,957,949 bytes versus C**, or **6,344,985 bytes versus A**. This includes actual reordering, repartitioning, compression and persisted allocator state rather than a raw field-width subtraction.

D stores one 32-byte origin scope in each namespace root and eight-byte serials for inode references. It allocates 17,922 distinct serials in fixed first-appearance order. All content and metadata CAS keys remain full 32-byte hashes. The old-ID mapping is a verification oracle, not a hidden runtime dictionary. D has 4,145 rewritten directory nodes, 604 inode-table nodes and eleven new roots; maximum canonical object size is 8,144 bytes.

All original 46,288 metadata objects authenticated, and all 782 original metadata compression groups were reproduced byte-for-byte under the pinned codec before alternatives were interpreted. Focused representation checks covered hardlinks, rename, distinct allocations and shared-ancestor branches. The modeled allocator is not a concurrent product implementation: global serial reservation, publication rollback, imports across scopes and old-format dispatch still require product design and tests.

A's 7,063,939 metadata pack bytes are a matched offline control, not the original online total of 6,550,582. The combined-copy experiment accounts for the actual original source separately; do not subtract the independent A-to-D saving directly from the public 49,319,936-byte result.

## CDC: similarity works where extra overlap hints do not

The twelve expensive lockfile targets were frozen before alternatives were encoded. The exact product CDC scanner reconstructed their source spans, identities and first-admission provenance. Pinned native encoding reproduced the existing selected frames byte-for-byte.

- Trying the other at-most-three same-offset overlap hints saved only **602 bytes out of 144,549**. It was rejected as the main optimization.
- One bounded previous-file similarity candidate, using the existing sixteen-byte window/eight-minhash policy on the same targets, offered **70,745 bytes** of record savings. Every selected candidate was outside the original overlap hints.
- The fixed policy was then simulated chronologically across all **249 lockfile chunks**, updating base eligibility after each encoding decision. It saved **551,895 bytes net**, including worse outcomes and new FULL resets.

The full family record sum falls from 1,123,468 to 571,573 bytes: 152 targets improve, twelve worsen and 85 remain unchanged. Twelve formerly PREFIX records become FULL and consume 79,971 bytes, already included in the final total. The experiment therefore does not add isolated target savings while ignoring changed dependency depths.

All nine large lockfile versions reconstructed exactly, and all 735 native objects authenticated under the final graph. There are no non-lockfile dependents on substituted lockfile chunks. Maximum depth is four, raw closure 163,840 bytes and lookup count five. The existing pack/group memberships still fit their size limits; native packs shrink by exactly 551,895 bytes, from 3,259,492 to 2,707,597. No split or extra locator is required for this corpus.

Discovery is not free. Eight previous-file signature indexes involved 575 dependency lookups, 1,865,480 encoded-work bytes and 14,488,535 decoded-work bytes. Their Python/ctypes construction took 3.428 seconds; this is not a Rust or public Commit measurement. Candidate trials retain cumulative target-read checks, while actual admission-batch remaining quotas and public read-wave ownership remain unqualified.

## Framing: exact modest gain, unchanged compressed data

All 514 original SmallContent packs and 33,217 objects were reencoded under two explicit diagnostic grammars:

- Four-byte group starts replace sixteen-byte directory entries: **398,604 bytes saved**.
- Deriving redundant raw/frame lengths from validated locator/range information saves another **265,736 bytes**.

Combined intrinsic saving is **664,340 bytes**, matching the prospective prediction exactly. Every original pack reconstructed byte-for-byte, and every object and dependency authenticated. Forty-one negative checks covered format refusal, malformed directories and lengths, invalid identities, missing/corrupt bases, cycles, chronology and closure limits.

The standalone equally compact SQLite comparison saved 716,800 bytes because page geometry changed. The final combined result was constructed and measured independently; neither 716,800 nor a generic VACUUM saving was simply added to the metadata result.

## Combined validation and limits

The metadata encoding changes canonical namespace roots, so it also changes derived LayerIds and CommitIds. The combined constructor first verified the original derivations, then rederived one genesis layer and all ten commits in dependency order, updating parent/base/head/root references. Keeping old typed IDs with new root fields would have been invalid even if SQL foreign keys passed. Full old-to-new mappings are retained.

Both combined copies passed:

- Exact metadata semantics for eleven retained namespace states, including the initial state.
- Authentication of all SmallContent objects and dependencies, plus all 735 native objects under the selected graph.
- Complete physical-record/locator membership with no missing or unlocated records.
- Typed identity derivations and all rewritten history/head references.
- SQLite integrity and foreign-key checks; unchanged original source digest.

These are standalone diagnostic checks. They do not substitute for supported Rust readers, old-format opens, concurrent allocation, transaction rollback, public FUSE/SDK reads, save/Commit timing or same-Store historical API verification. The diagnostic copies deliberately use unsupported format versions.

## Decision

The experiments identify a concrete candidate architecture: **inline inode values, direct directory mappings, scoped compact inode identity, compact SmallContent framing, and bounded previous-file CDC similarity**. The overlap-only policy is rejected. No codec sweep, hash truncation, general repacker, deleted historical state, or metadata-semantic removal was needed to obtain the measured offline result.

The combined result is close enough to 40 MB to justify product implementation, but it does not establish 40 MB or public speed. It remains 1,648,128 bytes above the target even with offline compact packing. Production allocation could differ materially. The next milestone is a supported, versioned implementation with allocator/import and reader ownership defined, followed by one fresh ten-snapshot public performance/census/same-Store-verification/cleanup run. An additional optimization must have its own measured mechanism; the remaining gap cannot be claimed away by rounding or by omitting online costs.

## Artifacts

Root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments`.

- [Combined layout report](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/combined/report.md) and [result](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/combined/result.json).
- [Metadata A–C](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/metadata/report.md) and [compact identity D](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/metadata/report-D.md).
- [CDC whole-family graph](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/cdc/family-report.md), with rejected overlap and fixed-sample similarity reports alongside it.
- [Framing and SQLite geometry](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/framing/report.md).
- Each lane includes a prospective protocol, executable scripts and raw results; shared hash/CDC helper source, build commands, input identities and runnable checks are in `tools/`.

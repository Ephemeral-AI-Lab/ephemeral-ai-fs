# Why inline metadata scales differently over157 states

Read-only diagnosis of the sealed metadata cache/root results. No new encoding, policy change, product operation or Store write was performed. `scaling-diagnosis.py` reproduces these counts and checks the source cache hash before/after; `scaling-diagnosis.json` retains the arithmetic and source identities. The full experiment's original manifest remains unchanged.

## Repetition grows much faster than distinct inode values

| Metadata fact | Ten-state D | Full157 D |
|---|---:|---:|
| Historical roots, including empty | 11 | 158 |
| Logical inode occurrences across roots | 58,871 | **904,301** |
| Inline value occurrences in unique physical leaf objects | 58,871 | **885,543** |
| Distinct inode values | 37,289 | **89,576** |
| Copies per distinct value | 1.579 | **9.886** |
| Repeated value occurrences beyond the first | 21,582 | **795,967** |
| Raw bytes occupied by73-byte inline values | 4,297,583 | **64,644,639** |
| Raw bytes in repeated copies of those values | 1,575,486 | **58,105,591** |

Physical inline occurrences grow **15.04×**, while distinct values grow only **2.40×**. The full-history leaf inventory has8,930 unique leaf objects plus157 branches; leaf canonical bytes alone total72,121,903B. Root-level logical repetition exceeds physical repetition by18,758 entries, so CAS sharing of whole leaves avoids only about2.07% of the logical occurrences.

There are52,724 stable inode identities and123,146 distinct `(inode identity, inode value)` pairs, but only89,576 distinct values. Different stable inodes can share the same value; keeping identity and value separate is semantically possible. The full value includes kind, namespace linkcount, content root and metadata root, so value sharing would not erase inode identity or hardlinks.

The measured design still saves metadata pack bytes:49,487,786→34,917,104B. However, canonical metadata grows79,372,108→80,160,065B. Removing separately indexed inode objects was profitable in the short history; repeating their values in changed leaf pages becomes a larger cost in the long history. Full-history D also rebuilds balanced partitions under fixed100-entry leaves, so changes in population can alter boundaries. This does not prove that a particular stable-partition replacement would save bytes; actual updates still change content and record references.

**58.11MB of repeated canonical value bytes is not58.11MB of recoverable compressed storage.** The current pinned group compressor already removes some repeated fields and values. The remaining high-entropy references repeat across independently compressed groups and historical versions.

## The full experiment also omitted a compression mechanism the source already uses

The original full Store has4,673 selected metadata delta records:

- Reconstructed canonical bytes: **26,757,164**.
- Selected COPY/INSERT record bytes: **5,571,112**.
- Difference: **21,186,052 bytes before whole-group compression**. Counting the alternative FULL record's one-byte tag gives21,190,725B instead.

Those are decoded representation differences, not21MB of allocated savings. FULL and mixed groups can compress differently, and base/reference/pack/index costs must remain in the accounting.

D deliberately used the same FULL-only offline packing policy in both the ten-state and full-history experiments. It retained compact identity/inlining but did **not** apply the original metadata-delta selection mechanism to the new pages. Therefore the result is not evidence that compact scoped identities intrinsically need34.92MB of metadata. It is evidence that this exact FULL-only representation is insufficient to reproduce Git's long-history cost.

## Most actionable next hypothesis: bounded metadata deltas on D pages

The first next experiment should retain D's canonical representation and test **one fixed bounded predecessor-leaf selection rule plus the existing authenticated metadata COPY/INSERT/group machinery**. Select candidates from the previous snapshot's overlapping stable-serial ranges, with a declared small candidate/base limit; account the actual selected FULL/DELTA ledger, base eligibility, decoding closure and full compressed groups. Do not assume every previous page remains a free FULL base after earlier decisions choose DELTA, or that21.19MB of source decoded savings transfers to D.

This attacks repeated unchanged content/hash ranges across independently encoded historical leaves. The source already demonstrates that bounded metadata deltas can find substantial repetition in this workload. D's canonical8KiB page bound keeps individual metadata operands small. This is **untested on D**, and it must retain bounded authenticated reads and existing retained-history semantics. Its value is a narrower next question than another namespace-format rewrite.

A hybrid—compact serial identity but a separate CAS object for each inode value—is a valid second hypothesis, with an important trap. Holding D leaf membership fixed and retaining98-byte canonical value records gives the raw model:

`full: 885,543 × (73 − 32) − 89,576 × 98 = 27,528,815 raw bytes removed`.

For the ten-state population the same calculation is **−1,240,611B**, meaning the hybrid adds raw bytes. This crossover explains why the short-history result cannot select a universal granularity.

But the hybrid model replaces each73-byte inline value with another32-byte random hash. D's kind, linkcount and mostly shared metadata-root fields already compress well; the32-byte content-root reference is a major residual cost. A FULL-only hybrid still repeats32-byte references in hundreds of thousands of entries and adds **89,576 CAS objects and index rows**, before any repartitioning. Consequently the27.53MB raw reduction could translate into much less compressed saving, or an outright loss. It is not an allocation forecast and should not be stacked with the metadata-delta model.

The measured outcome is that fixed D plus the other completed offline changes leaves the whole full157 layout materially above Git. Neither bounded D metadata deltas nor the compact-ID/value-CAS hybrid has been tested. The evidence supports testing metadata history compression first; it does not support describing either untested choice as a fix already found.

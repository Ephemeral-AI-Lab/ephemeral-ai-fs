# Priority 2 content: whole-file graphs with native chunk adapters

The actual 53-state Store shrinks **57,974,784 → 54,382,592 B allocated**, a **3,592,192 B (6.20%) saving** after the verified Priority 1 metadata changes. The copy preserves all original SmallContent and native chunk canonical IDs, metadata pack bytes, typed SQL rows, the metadata pool catalogue, allocator, and existing SQLite schema. It adds 224 authenticated whole-file object rows and the necessary physical content records and locators.

This is an **offline storage experiment with substantial read amplification**. It does not qualify a product format or public read performance. The parent experiment owns the separate original filesystem-oracle proofs and final retained scorecard.

The actual copy is `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/content/candidate.sqlite`, SHA256 `dd4a3c7cd2260b487b570cb819f2799ff189c2470382d4481b0f2fd19f3360a3`. [result.json](result.json) contains its allocation, component, and custody measurements. [protocol.md](protocol.md) froze the policy before encoding.

## What changed physically

The existing Git53 dependency graph now spans all 59,992 nonempty regular-file versions: 59,768 existing SmallContent identities and 224 new large-file identities. The prefix codec remains pinned to the existing parameters, with window 18 for small files and window 20 for large files. Graph bounds remain 50 edges and 64 MiB of canonical/encoded closure.

All 1,998 original native chunk identities become 41-byte slice adapters into retained, authenticated large-file objects. An adapter checks its extent, reconstructs and authenticates the owner, takes the requested slice, and authenticates the original chunk canonical identity. Actual coverage is **1,998/1,998**, with no fallback FULL chunks required. The fallback is still implemented for an unmapped chunk.

| Physical component | Bytes |
|---|---:|
| SmallContent compressed frames | 41,452,060 |
| SmallContent records, including kind/base fields | 43,063,636 |
| Large-file compressed frames | 2,090,507 |
| Large-file records, including kind/base fields | 2,097,099 |
| Native adapter records | 81,918 |
| Record directories and pack headers | 251,848 |
| **New complete content packs** | **45,494,501** |
| Previous complete content packs | 48,824,145 |
| **Content pack saving** | **3,329,644** |

Frames are subtotals within records and must not be added twice. The remaining **262,548 B** of the net Store saving comes from actual SQLite page/overflow/layout effects after replacing the content packs and running VACUUM. All necessary new index rows are included. This is not a separate index optimization or an independently additive estimate.

The new layout has 243 content packs. Its largest actual pack is **724,078 B**, and its largest frame is **483,544 B**. Both exceed legacy limits; the experimental grammar explicitly permits 4 MiB packs and frames up to 2 MiB + 1 KiB. Existing metadata packs keep their original bytes and 256 KiB limit.

Source authentication passed for 59,768 small and 1,998 native objects. Candidate authentication passed for 59,768 small, 224 large, and 1,998 native objects. Every original content identity and length remains indexed; the only additional objects are the 224 large-file versions. SQLite integrity/FK checks and all noncontent table, schema, and metadata pack comparisons passed. Original file roots and native map IDs need no rewrite because adapters reconstruct their exact original chunks.

## The read-amplification cost

We ranked every actual native adapter's owner graph, then decoded and authenticated the two worst selections with empty reader caches. OS cache was uncontrolled. These timings are diagnostic observations, not public product latency measurements.

| Worst selection | Returned chunk | Raw graph bytes decoded | Amplification | File edges + adapter | Diagnostic cold read |
|---|---:|---:|---:|---:|---:|
| Largest decode/output ratio | 6,421 B | 15,608,785 B | **2,430.90×** | 23 + 1 | 277.37 ms |
| Largest owner graph | 9,306 B | 20,466,017 B | 2,199.23× | 37 + 1 | 366.92 ms |

The first read consumed 318,892 B of encoded file records, plus its 41-byte adapter, and fetched **1,022,176 B of physical packs**. The second fetched **1,703,900 B**. Preserving a native object's canonical identity does not preserve its former chunk-read cost. The storage win therefore does not establish that this design is suitable for product integration.

Complete content verification used encoded file order followed by native chunk hash order. It decoded **12,037,267,088 canonical file bytes** and fetched **7,894,813,676 physical pack bytes** across 31,304 cache misses. Total experiment wall time was **344.02 seconds**, including source authentication, encoding, actual copy construction, and candidate authentication. Arbitrary native owner access caused substantial cache thrashing. This scope is not Commit latency.

The separately frozen full157 extension preserves encoding, owner selection, packing, and cache limits. Its exhaustive internal verifier groups native adapters by owner, preserving coverage and authentication while avoiding the arbitrary hash-order artifact. The parent's verifier still reads actual original filesystem states, and worst cold adapters are tested separately.

Reader cache accounting is 4 MiB canonical payload, 8 MiB physical packs, and the inherited metadata pool's 4 MiB decoded bodies plus up to approximately 4 MiB of copied records. That is about **20 MiB of retained payload**, before graph records, decoder contexts, Python/locator overhead, and temporary outputs. It is not an RSS cap. File graph closure bounds separately allow 64 MiB canonical and encoded bytes. [execution-notes.md](execution-notes.md) clarifies this inherited accounting without changing the executed protocol.

## Checks and reproducibility

Five in-memory corruptions were rejected without modifying the SQLite artifact: a damaged frame checksum, file-graph self-cycle, missing adapter owner, wrong owner role, and overflowing slice extent. [reader-checks.json](reader-checks.json) records adapter coverage and actual worst reads; [check_reader.py](check_reader.py) reproduces the checks.

[experiment.py](experiment.py), [reader.py](reader.py), and [content_codec.py](content_codec.py) preserve the executed implementation. The first attempt failed during imports, before encoding or Store construction, because a generic `codec` name collided with the inherited metadata codec. Renaming the helper to `content_codec` fixed the import collision. The attempted source and [failure log](failed-import/run.log) are retained; encoding and ownership policy did not change.

Raw mapping coverage, logs, checksums, and the actual Store are under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/content/`. The full157 extension has separate protocol, source copies, and artifacts under `full157/`.

# Priority 2 content: full157 physical-copy replication

The same content representation policy reduces the verified metadata baseline from **71,970,816 → 65,957,888 B allocated**, saving **6,012,928 B (8.35%)**. This is an actual SQLite copy, including all new whole-file identities, index rows, base references, slice adapters, pack framing, and allocation effects. It preserves every original content canonical identity and all metadata/typed SQL data.

Source and candidate content authentication passed. The parent experiment separately owns the original 157 filesystem-oracle proof; content authentication alone must not be reported as that proof or as public product qualification.

The candidate is `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/content/full157/candidate.sqlite`, SHA256 `efacb9b832961ed7a2e675c26f113545622b75df684f9c11a36f3b1a121bec11`. [result.json](result.json) records the complete measurements and custody; [protocol.md](protocol.md) froze this extension before encoding.

## Complete physical accounting

| Component | Before this content stage | After |
|---|---:|---:|
| Content packs | 56,367,207 B | **50,710,265 B** |
| Metadata packs | 9,603,174 B | **9,603,174 B** |
| SQLite nonpack contribution | 6,000,435 B | **5,644,449 B** |
| **Actual allocated Store** | **71,970,816 B** | **65,957,888 B** |

The content pack saving is **5,656,942 B**. The remaining **355,986 B** comes from measured SQLite page/overflow/layout effects after replacing content packs and running VACUUM, with all 523 necessary new index rows included. The index schema and policy did not change. These are reconciled contributions to one actual Store result, not independently additive projections.

| New content component | Bytes |
|---|---:|
| SmallContent frames | 45,856,792 |
| SmallContent records, including kind/base fields | 47,956,894 |
| Large-file frames | 2,283,323 |
| Large-file records, including kind/base fields | 2,299,782 |
| Original native IDs represented by slice adapters | 132,061 |
| Directories and pack headers | 321,528 |
| **Complete content packs** | **50,710,265** |

Frames are included in record subtotals and must not be added twice. There are 310 new content packs. Maximum actual pack size is **735,861 B**, and maximum frame size is **483,544 B**. Both require the explicitly unsupported grammar with its frozen 4 MiB pack and 2 MiB + 1 KiB frame limits; they do not fit the legacy native pack limits.

## Preservation and coverage

- Source authentication: **75,398 SmallContent + 3,221 native chunk identities**.
- Candidate authentication: **75,398 SmallContent + 523 large-file + 3,221 original native chunk identities**, totaling **79,142**.
- Native coverage: **3,221/3,221** adapters; zero unmapped chunks and zero FULL fallbacks needed. The fixed fallback remains implemented for an unmapped chunk.
- Original object identities and canonical lengths are unchanged. Additional indexed objects are exactly the 523 new large-file versions.
- Metadata pack bytes, pool catalogue, allocator, typed SQL rows, and the complete SQLite schema match the verified source.
- SQLite integrity and foreign-key checks passed; the source Store checksum remained unchanged.
- SmallContent selects 12,126 FULL and 63,272 prefix records. Large files select 25 FULL and 498 prefix records. All representations and dependencies remain counted once.

The new objects use the same raw-file Git-selected graph and codec as the 53-state trial. File roots and native map identities remain unchanged: each adapter reconstructs its exact original chunk and verifies the original canonical BLAKE3 identity after slicing. No original chunk was discarded based solely on a byte-range approximation.

## Read amplification remains a major cost

Every adapter's actual owner graph was ranked. The two worst selections were then decoded and authenticated with empty reader caches. OS cache was uncontrolled; elapsed times are diagnostic measurements, not public latency benchmarks.

| Selection | Returned chunk | Raw graph decoded | Amplification | File edges + adapter | Diagnostic cold read |
|---|---:|---:|---:|---:|---:|
| Largest decode/output ratio | 6,421 B | 23,758,968 B | **3,700.20×** | 35 + 1 | **420.83 ms** |
| Largest owner graph | 17,979 B | 29,807,446 B | 1,657.90× | 47 + 1 | 525.45 ms |

The first read required **329,803 B of encoded file records**, its **41-byte adapter**, and **1,241,516 B of physical packs**. The second fetched **2,211,294 B** of physical packs. Maximum observed file graph canonical closure is **29,808,454 B**; the complete file population reaches 50 edges, while the largest adapter owner in this dataset has 47 file edges plus its adapter.

The storage improvement does not preserve the original native chunk read cost. The format must remain an offline experiment unless that tradeoff is accepted or addressed. Five corruption checks passed: bad frame checksum, file self-cycle, missing owner, wrong owner role, and overflowing slice extent. [reader-checks.json](reader-checks.json) and [check_reader.py](check_reader.py) retain the exact worst selections and checks.

## Execution differences and custody

Encoding, owner selection, packing, fallback, codec, and read bounds are identical to the 53-state policy. Only the source/fixture/cardinality paths changed. The fixed owner rule remains first owner in Git OID order, then lowest offset. No content identities, frames, or selected bases from the 53-state run were reused.

One verification-only ordering change was declared before this run: native adapter checks are grouped by owner after the file-object checks. This keeps coverage and authentication identical while avoiding the arbitrary native chunk hash-order thrashing observed in the 53-state object proof. It does not change physical packing, owner assignments, cache limits, or the parent's original filesystem-order proof.

Total diagnostic wall time was **462.80 seconds**, covering source authentication, encoding, actual copy construction, and complete candidate content authentication. The verifier decoded **13,979,051,959 canonical file bytes** and fetched **10,435,269,869 physical pack bytes** across 45,328 cache misses. Those totals are specific to this verification ordering and uncontrolled OS cache; they are not Commit costs or throughput claims.

The reader retains the same cache limits and inherited metadata accounting described in [the 53-state execution notes](../execution-notes.md): approximately 20 MiB of retained payload before graph records, contexts, Python/locator overhead, and temporaries. This is not an RSS bound. Canonical and encoded graph closure limits remain 64 MiB each.

[experiment.py](experiment.py), [reader.py](reader.py), and [content_codec.py](content_codec.py) preserve the executed implementation. All raw output, coverage mappings, logs, and the complete SQLite artifact are in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/content/full157/`. No encoding attempt failed in this full157 replication.

# Stride3 structural metadata replay — 2026-09-10

**PASS: identical fixed structural policy on 53 captured states plus genesis.** This is an offline metadata pack measurement; complete database allocation and external path/content verification belong to the combined experiment.

| Representation | Actual metadata pack bytes |
|---|---:|
| Original captured product metadata | 20,094,239 |
| Fixed D inline/scoped canonical format, hash-sorted FULL packs | 12,314,839 |
| Matched D chronological FULL control | 12,350,890 |
| Same structural selected DELTA policy as full157 | **6,877,152** |

Structural deltas save **5,473,738 B (44.32%)** versus the matched chronological FULL control. The original-to-final metadata pack reduction is **13,217,087 B**. These exclude object-index replacement costs; do not subtract them directly from an allocated Store total.

The fixed policy permits16 edges and128KiB summed canonical closure, with previous-root greatest actual shared stable-key overlap selecting one candidate, existing matcher budgets, and the same actual group compression acceptance threshold. No parameter search or checkpoint representation was used. Actual maximum depth is15; maximum canonical closure130,223 B. Per-object canonical cap remains8KiB.

- Authenticated all84,003 original metadata objects, including exact recompression of2,572 original groups.
- Built10,542 D canonical objects totaling28,458,524 B;25,390 scoped inode identities and6,875 transformed directory objects.
- Compared all54 original namespace roots' inode and directory semantics; original IDs map to immutable serial identities.
- Selected2,721 DELTA and7,821 FULL records;2,706 physical dependency bases are retained in that same inventory and counted once.
- Encoded138 packs in each matched layout; parsed actual physical packs and reconstructed/authenticated all10,542 canonical objects.
- Verified all54 resulting inode tables and directory semantics again. Cached export self-check passes all pack hashes, locators, objects and roots.
- 70 candidates rejected by canonical closure bound;59 candidates rejected with their compressed mixed group;232 have no predecessor overlap. No matcher byte/instruction budget skips.
- D construction7.08s; structural encode and internal verification19.13s. Diagnostic Python timings, not Commit or public read timings.

Source Store SHA256: `5c6ee04eee133539f043ee64d242c26769c77d30b434af2523666e326a8d999e`, unchanged after both encodes. Source verification manifest is included in D result provenance. No failed attempts occurred.

Raw artifacts: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/metadata`. Import `chain_api.py` for `original_inventory`, `root_map`, `inode_oracle`, `decode_table`, `decode_directory`, `decode_record`, `pack_blobs` and `record_ledger`; D namespace chronology is in `d/roots.json`, selected record chronology in `cache.sqlite.records.first_step`. Parent must map complete typed SQL identities and check all53 independent original snapshot oracles.

Reproduce on this machine, in a fresh output directory after adapting only HERE/source artifact paths: run `d/full.py`, seal D artifacts as `d/manifest.json`, then `experiment.py` and `chain_api.py`. Existing output files cause refusal to overwrite. See [frozen protocol](protocol.md), [structural result](result.json), and [D result](d/result.json).

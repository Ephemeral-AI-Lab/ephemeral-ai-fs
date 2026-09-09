# Experiment 1 result: inline metadata saves 3.39 MB in the matched diagnostic

The three predeclared treatments completed. **Inline inode values plus direct directory mapping references reduced the matched metadata-pack-plus-object-index subtotal by 3,387,036 bytes (31.37%).** Inlining alone saved 3,047,864 bytes. This is a measured offline representation result; no product Store was optimized and no Commit latency was measured.

| Fixed treatment | Metadata objects | All locator rows | Complete encoded metadata packs | Compact object index | Matched subtotal | Reduction vs A |
|---|---:|---:|---:|---:|---:|---:|
| A: current canonical metadata, reconstructed | 46,288 | 80,240 | 7,063,939 B | 3,731,456 B | 10,795,395 B | — |
| B: inline 73-byte inode values, stable 32-byte IDs | 9,088 | 43,040 | 5,748,683 B | 1,998,848 B | 7,747,531 B | **3,047,864 B** |
| C: B plus direct directory mapping references | 5,080 | 39,032 | 5,593,831 B | 1,814,528 B | 7,408,359 B | **3,387,036 B** |

B saves **1,315,256 encoded pack bytes + 1,732,608 index bytes**. C saves **1,470,108 encoded pack bytes + 1,916,928 index bytes**. Removing directory wrappers adds **339,172 bytes** of saving beyond B, comprising154,852 pack bytes and184,320 index bytes. These totals already include the removed inode/directory locator rows: do not add the study's earlier projected index reductions again.

## What was actually encoded and counted

The original inventory contains37,289 separate inode CAS records,695 inode-table nodes and11 namespace roots. B removes all37,995 objects in those roles and replaces them with **784 table nodes and11 namespace roots**. Thus **89 extra table nodes** are charged instead of pretending the old leaf count survives larger entries. C additionally removes4,008 directory wrappers. Each replacement object has its real32-byte BLAKE3 identity, and all retained content objects keep their original full-hash locator rows.

Leaves hold the stable32-byte inode identity and exact73-byte inode value. Balanced leaves contain at most77 entries under the8,192-byte canonical page limit; branch capacity remains127. The largest new leaf is8,129 bytes. The largest retained canonical metadata page is8,191 bytes. All rehashed replacement branches and namespace roots are in the inventory, packs and index projection. The same4,145 directory mapping nodes, metadata/value support objects and large-file metadata survive unless explicitly removed by the chosen treatment.

The fresh diagnostic grammar uses `LFS5INT`/`LFS5FSR` magics and a treatment-specific profile ID. It is an executable proposed encoding, not a product-supported format. LayerFS's canonical object hash is BLAKE3 over `layerfs/object/v2\0` plus canonical bytes; the shared helper uses the repository's existing cached Rust BLAKE3 dependency. No dependency was added to the product.

## The matched baseline and its limits

All treatments use the protocol's identical offline rule: unique complete metadata inventory, role-magic then full-ID ordering, FULL records,16-KiB decoded groups, existing group/pack framing, maximum256-KiB encoded **and decoded** pack sizes, and the pinned RAW-versus-Zstandard16-byte threshold. Real compressed bytes, directories and pack headers were counted. There are703/571/545 groups and42/36/34 packs for A/B/C.

A's7,063,939 metadata bytes **are not** the retained Store's measured6,550,582 bytes. The original Store uses online admission order,149 packs,782 groups and283 selected metadata deltas; this diagnostic uses sorted FULL-only inventory packing for all alternatives. It therefore does not inherit actual mixed-group selection, predecessor search, staging, concurrency or runtime allocation. A is a sound control for this exact representation experiment, not a substitute benchmark baseline. **Do not subtract3,387,036 from49,319,936 and call the result achieved or expected Store allocation.**

Likewise, these are isolated SQLite4096-byte full-hash index projections after VACUUM. Content locators retain their real existing values; metadata locators point to actual diagnostic pack/group/record positions. All three remap metadata pack numbers beyond the original maximum, so their integer widths are consistently charged. The isolated index schema omits the foreign key to absent `object_packs`; this does not change the indexed row representation. The table/row checks and SQLite integrity check pass. Source metadata/other tables, WAL, file allocation granularity and publication work are excluded. Projected index bytes differ slightly from the earlier deletion-only floor because replacement objects and locator integer widths are now real.

## Verification and evidence

- Authenticated **all46,288** reconstructed original metadata objects against their stored full BLAKE3 IDs; all283 selected legacy deltas decoded through their original FULL bases.
- Re-encoded **all782 original decoded groups** and matched their exact stored codec choice and bytes. This establishes the ctypes compressor uses the actual pinned metadata policy, rather than merely compatible decompression or a system default. Loaded Zstandard1.5.7; exact library hash is in `result.json`.
- Used `ZSTD_getCParams(1,raw_size,0)`, windowLog capped16, estimated static context, explicit frame flags and `compress2`, matching `objects/pack.rs:1197`. Maximum codec workspace was218,136 bytes; every new compressed group was decompressed and compared exactly.
- Checked **all11 original namespace states** through B and C: identical stable inode IDs, kinds, reference counts, metadata roots and file content roots. C's directory roots were resolved to exactly the original directory mappings and authenticated level/count summaries; complete directory entry lists matched. Original metadata is shared unchanged rather than synthesized from fewer fields.
- Ran a focused **hardlink/rename representation check** with two names referencing one inode, reference count2, and a namespace rename that preserves the same inode and content/metadata value. This tests the proposed encoding's semantics; it is not a product rename implementation test.
- Checked every replacement canonical ID and page bound, counted every real replacement locator row, and ran SQLite integrity checks on all three new indexes.
- Original Store SHA-256 remained `713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4` before and after. The original measured Store, product source and old evidence were not changed.

Artifacts: `protocol.md` fixes the three treatments and policy before encoding; `run.py` reproduces the entire diagnostic and refuses to overwrite existing index results; `run.log` records execution; `result.json` contains byte accounting and integrity identities; `roots.json` contains per-state root mappings and semantic checksums; `A-index.sqlite`, `B-index.sqlite`, `C-index.sqlite` are newly created index-only projections, not Store copies. Re-run in a fresh experiment output directory with the shared `../tools/hashing.py` helper available, or preserve/move the old index outputs first.

## Decision

**C is the winner of this fixed experiment; the major return is eliminating separately indexed inode records.** The improvement is real under matched encoding and smaller than the5.75-MB noncontent saving needed even if content matches Git. This experiment provides no evidence that metadata restructuring alone reaches40MB.

The next product decision is whether to integrate C as a new explicitly versioned namespace format and run the public ten-snapshot workload once, measuring actual compressed mixed groups, index allocation, memory bounds, historical reads and Commit latency. It changes shared canonical readers/writers, batched inode resolution, object-reference traversal, reconciliation and profile dispatch; existing hardlink/rename behavior must pass through those actual owners. The product check should use a fresh Store and preserve old-format read behavior. Compact inode identity remains a separate experiment, not an unmeasured saving added to these results.

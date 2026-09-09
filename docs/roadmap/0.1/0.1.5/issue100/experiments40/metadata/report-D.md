# Conditional D result: compact scoped inode identities recover another2.96MB

The one predeclared D treatment passed and reduced the matched subtotal by **2,957,949 bytes versus C**, after charging a persistent allocator page. Together with inode inlining and directory-wrapper removal, D saves **6,344,985 bytes versus the original A representation control**. All original metadata semantics matched after mapping diagnostic serials back to their original stable inode identities.

| Matched diagnostic | A: current canonical inventory | C: inline/direct directory | D: C plus scoped8-byte serial |
|---|---:|---:|---:|
| Canonical metadata bytes | 10,194,894 | 8,571,111 | **6,277,255** |
| Metadata objects | 46,288 | 5,080 | **4,900** |
| All object locator rows | 80,240 | 39,032 | **38,852** |
| Complete encoded metadata packs | 7,063,939 B | 5,593,831 B | **2,639,978 B** |
| Compact object-index pages | 3,731,456 B | 1,814,528 B | **1,806,336 B** |
| Additional scope-allocator page | — | — | **4,096 B** |
| **Matched subtotal** | **10,795,395 B** | **7,408,359 B** | **4,450,410 B** |
| Saving versus A | — | 3,387,036 B | **6,344,985 B** |

The incremental C→D saving is **2,953,853 encoded metadata bytes +8,192 index bytes −4,096 allocator bytes**. The canonical reduction is2,293,856 bytes. This is more than a raw width subtraction: serials replace pseudorandom identity fields with small ordered values, alter inode-table ordering/partitioning, and change what the fixed compressor sees. The complete transformed inventory was actually encoded and decoded; no compression ratio was assumed.

The original measured online metadata total remains **6,550,582 B**, whereas A's matched FULL-only offline reconstruction costs7,063,939 B. None of these subtotals includes file-content packs or complete Store allocation. **D has not demonstrated a40MB product Store.** Parent's combined layout work may measure how these physical bytes fit together, but it remains a fresh unsupported diagnostic format until public APIs, history, allocator semantics and latency are implemented and qualified.

## The new identity representation is explicit and paid for

D assigns serials1–17,922 in the protocol's fixed first-appearance order. Each old identity receives one serial permanently throughout all eleven original states; serials are8-byte big-endian values. A32-byte origin scope is stored once in **each namespace root**. Canonical identity is scope+serial, rather than a truncated digest. All file-content IDs, metadata CAS IDs and object-locator keys remain their real authenticated32-byte hashes.

A new `scope_allocator` SQLite table stores the32-byte scope and current highwater17,922. Its actual dbstat payload is37 bytes and allocation isone4,096-byte leaf page. The table's logical scope+u64 state is40 bytes. D's complete index-only database is1,814,528 bytes:1,806,336 for `objects`,4,096 for `scope_allocator`,4,096 for `sqlite_schema`. C also has one4,096-byte schema page; the matched subtotal consistently excludes that common page, so allocator overhead is charged once and fixed schema overhead is not doubled.

The old32-byte→serial mapping is only the diagnostic verification oracle. A fresh new-format reader needs scope+serial and its authenticated namespace; it does not need that old mapping. Existing-Store migration would need translation and temporary storage, which this experiment does not implement. A global highwater is required across branches; resetting allocation from a historical snapshot's counter would be incorrect.

## Complete reference rewriting and validation

D retains140 original metadata/value support objects byte-for-byte and creates4,760 actual new canonical objects:

- **4,145 directory mapping nodes** replace the old directory nodes. Leaf references are8-byte serials; branch object references remain32-byte CAS IDs. Every changed ancestor was rehashed, and subtree-byte summaries now count10+name-length per leaf entry. Original directory page membership and levels were preserved.
- **604 inode-table nodes** replace C's784 nodes. Leaves hold81-byte entries and at most100 entries; branches hold40-byte entries and at most127 entries. Canonical pages are bounded by8,192 bytes; actual maximum is**8,144 bytes**. Serials determine table ordering, and all changed keys, branch references and summaries were encoded from their actual values.
- **11 namespace roots** contain the new profile, scope, root inode serial and actual rehashed table root. Each new namespace root is129 canonical bytes; the added scope bytes are included in pack/index accounting.

There are399 groups and25 packs. The same pinned legacy Zstandard policy, FULL-only role/full-ID sort,16KiB group target, frame-selection threshold and pack bounds used by A–C were retained. No codec/parameter sweep was performed.

Verification passed:

1. All46,288 original metadata objects re-authenticated; all782 original metadata groups matched their original encoded bytes under the exact pinned codec.
2. Every one of the11 transformed states decoded to its exact original inode-ID/kind/refcount/content-root/metadata-root tuples through the oracle mapping. Their semantic checksums equal the sealed C checksums.
3. All**4,145 directory objects** decoded to exact original ordered name/inode pairs, with valid counts, levels and encoded-byte summaries; all directory content pointers were rehashed and updated.
4. Focused checks preserved one serial across namespace rename and two hardlink names with refcount2, gave distinct serials to distinct inode allocations, retained a shared ancestor identity in two branches, and allocated different subsequent serials to new inodes in those branches.
5. Every generated object has its actual authenticated full hash; every locator row and allocator page was counted. SQLite integrity and persisted allocator-state checks passed.
6. All sealed ABC artifact hashes and the original Store's before/after SHA256 remain unchanged. No product source or measured Store was edited.

An initial Python syntax error occurred before any D encoding or index creation; it was corrected without changing treatment rules. There was one completed D treatment. A later extraction of callable helper functions rebuilt D once to verify its exact inventory and all25 recorded pack SHA256 hashes; it did not rerun A–C or create another index/treatment result. Both pre-refactor and current script hashes are retained in `result-D.json`.

## What is ready for reuse, and what is still product work

`protocol-compact-ids.md`, `compact_ids.py`, `result-D.json`, `roots-D.json`, `D-index.sqlite` and `run-D.log` preserve D independently of sealed ABC. `d_api.py` exposes `original_inventory()` and `pack_blobs(inventory, base_pack)` for combined offline layout accounting. `api-check.log` records exact agreement with all25 original D pack hashes. See `manifest-D.json` for integrity seals.

The diagnostic verifies a **single-origin fresh namespace**. It does not implement concurrent transactional serial reservation, publication rollback behavior, import of foreign inode scopes, existing-Store migration, public readers or persistent cross-branch reconciliation. The focused branch allocator check is a representation/allocation model, not a product concurrency test. Those are concrete integration requirements rather than compression uncertainties.

D is the strongest measured metadata direction so far. It changes stable identity representation and all namespace codecs, so integrating it is more substantial than C. Its measured bytes justify that product experiment; they do not justify blending a hypothetical content improvement into a claimed40MB result.

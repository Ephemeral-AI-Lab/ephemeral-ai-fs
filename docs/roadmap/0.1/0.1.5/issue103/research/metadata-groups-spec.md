# Historical metadata: authenticated value groups + bounded metadata deltas

Scope: **issue #103 / issue100 "historical metadata" only** — (1) authenticated shared metadata-value
groups ("Metadata V2") and (2) bounded metadata deltas (the depth-bounded metadata delta graph).
Content history, whole-file objects, native slice adapters and the SQLite page-size trial are out of
scope except where needed for the byte accounting of the two headline totals
(54,382,592 B / 65,957,888 B).

Status: **research note, not a plan and not an approved format.** The representation described here is
an unsupported offline prototype. Every number is quoted from a frozen artifact or is arithmetic on
quoted numbers (marked as such).

No product source file was modified to produce this report.

---

## 0. Provenance: which file is which

The prototype is a **genealogy of thin Python layers**, each importing the previous one. Getting the
layers right matters, because the same filename (`reader.py`, `experiment.py`, `codec.py`) exists at
several levels with different contents.

Raw roots (read-only evidence):

| Tag | Root |
| --- | --- |
| **P** | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb` (the Rust product repo) |
| **O** | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization` (V1/V2 + selected final layout) |
| **S** | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural` (depth-16 metadata delta chain; canonical `codec`/`api` package) |
| **F** | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157` (base reader: pack/group/cache mechanics) |
| **H** | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas` (depth-one metadata delta experiment + extracted Rust matcher) |

A **git-tracked, byte-identical source mirror** of `O` lives inside the product repo (verified by
SHA-256 on the files cited below; it excludes the large `.sqlite` artifacts):

```
docs/roadmap/0.1/0.1.5/issue100/experiments40/ordered-optimization/
```

Citations use the mirrored in-repo path where the file exists there (**`O/`**), and absolute paths for
the upstream roots `S`, `F`, `H`.

Import chain (each arrow is a runtime `import`/`importlib` load):

```
O/metadata/v2-53/{build,reader,negative-check}.py
  └─ O/metadata/{experiment,reader}.py                 (V1; protocol.md, result.json, report.md)
       ├─ O/metadata/reader.py:3-4   BASE = S ; import store_api as old   -> S/combined/store_api.py
       ├─ O/metadata/experiment.py:5 import experiment as e               -> S/metadata/experiment.py
       │     ├─ S/metadata/experiment.py:5  import api as D; import codec -> S/metadata/d/{api,codec}.py
       │     ├─ S/metadata/experiment.py:6  import matcher                 -> H/tools/matcher.py
       │     └─ O/metadata/experiment.py:86 import pack_api                -> S/metadata/d/pack_api.py
       └─ S/combined/store_api.py:12-14  old = F/combined/store_api.py
H/tools/matcher.py:7  ctypes -> H/tools/libdiagnostic_delta.dylib
H/tools/delta_ffi.rs:23 include!("delta_record.rs")   (exact source text of the product matcher)
```

`H/tools/source.json` records the matcher's origin:

```json
{"source": "/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb/crates/layerfs-layerstack-store/src/objects/pack.rs",
 "source_sha256": "a01f05a4…", "extracted_function_sha256": "9ffb462a…"}
```

i.e. **the prototype delta matcher is the product's own `pack.rs::delta_record`, copied verbatim**
(`H/tools/delta_record.rs:1-147`), wrapped only to replace `Result`/`ObjectId`/telemetry
(`H/tools/delta_ffi.rs:1-55`). The wrapper's own round-trip fixtures are at `H/tools/matcher.py:43-54`.

### 0.1 Headline artifacts

| Artifact | State | Bytes | SHA-256 (prefix) |
| --- | --- | ---: | --- |
| Improved matched baseline (53) | control | 59,760,640 | `ff2eb0bb…` |
| Improved matched baseline (157) | control | 79,790,080 | `e5837e34…` |
| V1 53 (`O/metadata/candidate.sqlite`) | **REJECT** | 66,527,232 | `5f22c403…` |
| V1 157 (`O/metadata/full157/candidate.sqlite`) | **REJECT** | 83,488,768 | `5b2725b0…` |
| V2 53 (`O/metadata/v2-53/candidate.sqlite`) | PROMISING | 57,974,784 | `54f47590…` |
| V2 157 (`O/metadata/v2-157/candidate.sqlite`) | PROMISING | 71,970,816 | `96cfd0ea…` |
| Selected+content 53 (`O/content/candidate.sqlite`) | reference only | **54,382,592** | `dd4a3c7c…` |
| Selected+content 157 (`O/content/full157/candidate.sqlite`) | reference only | **65,957,888** | `efacb9b8…` |

Decision wording is quoted from `O/metadata/report.md:28,30-46` and
`docs/roadmap/0.1/0.1.5/issue100/ordered-optimization-results.md:11-22`.

---

## 1. On-disk representation

### 1.1 Canonical object envelope — already byte-identical to the product

Prototype (`S/metadata/d/codec.py:10-15`):

```python
def oid(b): return blake3(b'layerfs/object/v2\0' + b)
def canonical(v): return b'LFSO\1' + struct.pack('>II', len(v)+4, len(v)) + v
def value(b):  assert b[:5]==b'LFSO\1' and struct.unpack_from('>II',b,5)==(len(b)-9,len(b)-13); return b[13:]
```

Product (`P/crates/layerfs-content/src/object/codec.rs:11-12,22-27,320-337`): `MAGIC = b"LFSO"`,
`HEADER_LEN = 9`, then `LFSO ‖ kind ‖ u32 BE (value.len()+4) ‖ u32 BE value.len() ‖ value`.
`blake3("layerfs/object/v2\0" ‖ canonical)` is the identity
(`P/.../object/digest.rs:5-7,44-52`).

⇒ **Nothing new is needed for the envelope or for identity.** The prototype's `canonical`/`value`/`oid`
are exactly the product's `encode_bytes_object`/`decode_bytes_object`/`ObjectId::for_bytes`.

### 1.2 Node header (31 bytes) — already byte-identical to the product

Prototype builds it inline (`S/metadata/d/scoped.py`; `header(...)` used at
`S/metadata/d/codec.py:110,118`):

```
(HBBB HQQ)  version=1, role, level, flags=0, count(u16), subtree_count(u64), subtree_bytes(u64)
```

Product `P/.../tree/directory/codec.rs:269-285` `node_header(magic,role,level,count,subtree_count,subtree_bytes)`
emits magic(8) ‖ version u16 BE ‖ role ‖ level ‖ flags=0 ‖ count u16 BE ‖ subtree_count u64 BE ‖
subtree_bytes u64 BE. `validate_node_header` (`:310-318`) caps `level ≤ 31`.
`NODE_LIMIT = 8192` (`:9`) is the canonical-node bound in both.

### 1.3 Metadata leaves and branches: `LFS6INT`

Role detection used by the prototype (`S/metadata/experiment.py:10`):

```python
def leaf(b): return b[13:21]==b'LFS6INT\0' and b[23]==7      # canonical offset 13 = start of value
```

Three magics matter for metadata (`S/metadata/experiment.py:12-35`):

| Magic | Role byte | Layout after the 31-byte header |
| --- | --- | --- |
| `LFS6FSR\0` | — (root) | profile_id(32) scope(32) root_inode(8 BE) inode_table(32) → value len 116 |
| `LFS6INT\0` | 7 = leaf | N × **81** B: serial(8 BE) ‖ kind(1) ‖ namespace_ref_count(8 BE) ‖ content_root(32) ‖ metadata_root(32) |
| `LFS6INT\0` | 8 = branch | N × **40** B: serial(8 BE) ‖ child ObjectId(32) |
| `LFS6NSP\0` | 1/2 | directory leaf/branch (name ‖ serial / name ‖ ObjectId) |

Bounds: leaf `1 ≤ N ≤ 100`, branch `1 ≤ N ≤ 127` (`S/metadata/experiment.py:43`, `:17`; and the
product's `P/.../tree/compact.rs:64,98,111`). Leaf canonical = 44 + 81·N ≤ 8144 B.
The **73-byte inode value** is `kind(1) ‖ namespace_ref_count(8) ‖ content_root(32) ‖ metadata_root(32)`
— corroborated by `O/metadata-population.json` field counts
(`type:3, link_count:2, content_or_directory:66529, attributes:4`).

⇒ The compact `LFS6INT` codec **already exists in the product**
(`P/crates/layerfs-content/src/tree/compact.rs:53-124`, `encode_inode`/`decode_inode`), with fixtures
under `P/crates/layerfs-content/tests/fixtures/compact/` sourced from the issue100 full157 compact
metadata cache (`.../manifest.json`). **Nothing in the product writes it yet** — the only non-test
caller is `P/.../object/references.rs:26-34`, added in the working tree (`git diff` shows the
`LFS6FSR/LFS6INT/LFS6NSP` arms added and `pub mod compact;` at `P/.../tree/mod.rs:1`;
`tree/compact.rs` is untracked). A writer for it does not exist.

### 1.4 The value pool

**Value object.** Each distinct 73-byte inode value becomes a canonical Bytes object whose value is
`b"LFSIVL1\0"` ‖ 73-byte value (`O/metadata/experiment.py:9,12-14`). Canonical length is therefore

```
13 (envelope) + 8 (magic) + 73 = 94 B
```

**Ordinal.** `ids = sorted(pool)` by canonical ObjectId; `ordinal = position+1`
(`O/metadata/experiment.py:28`). Ordinals are 32-bit **big-endian**, stored *inside* the leaf:

```
pooled leaf := leaf_canonical[:44] ‖ ( serial(8 BE) ‖ ordinal(4 BE) ) × N
```

(`O/metadata/experiment.py:29`; expansion at `O/metadata/reader.py:16-25`). A pooled leaf record is
therefore `44 + 12·N ≤ 1244` bytes, while the original object keeps its **original** `canonical_length`
(`44 + 81·N`, up to 8144) in `objects` — so **physical record length ≠ `canonical_length`** in this
format. This is a load-bearing fact for integration (see §5.1).

**V1 index form (rejected).** Two real SQLite structures were added:

```sql
shared_inode_values(ordinal INTEGER PRIMARY KEY CHECK(ordinal BETWEEN 1 AND 4294967295),
                    object_id BLOB UNIQUE NOT NULL REFERENCES objects(object_id))
```

plus the implicit UNIQUE auto-index on `object_id` (`O/metadata/experiment.py:100-101`,
`pragma user_version=9303`). These cost 2,748,416 B + 2,723,840 B of pages on 53 states and
3,698,688 B + 3,678,208 B on 157 states (`O/metadata/report.md:18-26`).

### 1.5 The "value group" pack format (V2)

**Group body** — built identically by `S/metadata/d/pack_api.py:16-26` and
`S/metadata/experiment.py:73-78`:

```
body     = u32 LE count ‖ u32 LE end_offset[count] ‖ record[0] ‖ … ‖ record[count-1]
record_i = 0x00 ‖ canonical_value_object(94 B)          # FULL only; pool packs carry no DELTA
```

* `len(body) ≤ 16,384` (hard assert, `S/metadata/experiment.py:77`).
* Measured max records per pool group = **165** (`select max(count) from pool_groups`), consistent with
  `4 + 4c + 95c ≤ 16384`.
* Each pool record is exactly **95 B** = tag(1) + 94 B canonical. The reader asserts `end−last == 95`,
  `record[0]==0`, `record[14:22]==b'LFSIVL1\0'`, `canonical[:5]==b'LFSO\1'`,
  `unpack('>II', canonical, 5) == (85,81)` (`O/metadata/v2-53/reader.py:35-38`).

**Group ordering.** Groups are filled from `sorted(inventory.items(), key=(role, oid))`
(`S/metadata/d/pack_api.py:22`). All pool values share one role, so groups follow **ObjectId order** —
which is also ordinal order, so **the ordinals in a group are a contiguous range**.

**Pack container** — `LFPACK` v1, the same container the product already uses
(`S/metadata/d/pack_api.py:27-40`; product `P/.../objects/pack.rs:11,44-56,791-855`):

```
"LFPACK\0\0" ‖ u32 LE version=1 ‖ u32 LE group_count
             ‖ group_count × { u32 LE offset, u32 LE encoded_len, u32 LE decoded_len, u32 LE codec }
             ‖ frame[0] ‖ … ‖ frame[group_count-1]
```

Prototype limits: pack ≤ **262,144 B** encoded and decoded, `group_count ≤ 256`, records ≤ 8191,
`codec ∈ {0=raw, 1=zstd}` (`S/metadata/d/pack_api.py:32,35`). These equal the product's `PACK_LIMIT`,
`GROUP_COUNT_LIMIT`, `RECORD_COUNT_LIMIT` (`P/.../objects/pack.rs:6-11`). The one **tighter** prototype
bound is the 16 KiB decoded group (product `GROUP_LIMIT` = 65,536).

**Compression.** zstd, level-1 params with `windowLog = min(windowLog,16)`, checksum + content-size +
no-dictID flags, static context (`S/metadata/d/codec.py:16-43`). A group is stored raw iff
`encoded + 16 ≤ raw` (`:43`).

**Group catalogue (V2 SQLite table)** — the whole point of V2 (`O/metadata/v2-53/build.py:25`):

```sql
CREATE TABLE pool_groups(
  first_ordinal INTEGER PRIMARY KEY CHECK(first_ordinal BETWEEN 1 AND 4294967295),
  count         INTEGER NOT NULL CHECK(count BETWEEN 1 AND 166),
  pack_id       INTEGER NOT NULL REFERENCES object_packs(pack_id),
  group_number  INTEGER NOT NULL CHECK(group_number BETWEEN 0 AND 255),
  group_sha256  BLOB NOT NULL CHECK(length(group_sha256)=32)
) STRICT
```

* **One row per physical group** — 404 rows / 24,576 B (53 states), 543 rows / 32,768 B (157 states).
* `group_sha256 = SHA-256(decoded group body)` — authenticates the count, the offset directory **and**
  every record. Computed at build time from the decompressed body (`O/metadata/v2-53/build.py:17`);
  verified at read time before any record boundary is trusted (`O/metadata/v2-53/reader.py:31`).
* Ordinal → `(pack, group, slot)`: `bisect_right(starts, ordinal)` then `slot = ordinal − first_ordinal`
  (`O/metadata/v2-53/reader.py:12-16,22-23`).
* The catalogue must be **contiguous and complete**: `first_ordinal` runs 1,2,… with no gaps and
  `Σ count < 2^32` (`O/metadata/v2-53/reader.py:14-16`; builder asserts the same at `build.py:13,19`).
* Catalogue-covered groups must be **exactly** the physical groups contributed by the removed pool
  rows and **disjoint** from ordinary CAS locators (`O/metadata/v2-53/build.py:44-45`).

V2 adds **no new pack format**: it reuses the exact V1 pack bytes (`O/metadata/v2-53/build.py:30`
asserts pack SHA-256 equality for every pack; result field `all_packs_byte_equal_v1 = "PASS"`).

### 1.6 The metadata delta graph (depth-bounded)

**Record grammar** (`H/tools/matcher.py:22-41`; identical to the product's
`P/.../objects/pack.rs:397-500`):

```
FULL   record = 0x00 ‖ canonical (1..8193 B)
DELTA  record = 0x01 ‖ base_id(32) ‖ u32 LE target_len ‖ u32 LE instruction_count ‖ instruction*
       instruction = 0x00 ‖ u32 LE start ‖ u32 LE len     (COPY from base)
                   = 0x01 ‖ u32 LE len ‖ bytes           (INSERT literal)
```

`target_len ≤ 8192`, `instruction_count ≤ 8191` (`RECORD_COUNT_LIMIT`, `P/.../objects/pack.rs:9`).
Producer: `delta_record` (`P/.../objects/pack.rs:549-697`; verbatim copy `H/tools/delta_record.rs`).
Consumer: `apply_delta` (`P/.../objects/pack.rs:475-500`) / `matcher.replay`
(`H/tools/matcher.py:22-41`).

**Bounds — two measured policies.**

| Bound | Depth-16 chain (`S`) | Depth-1 (`H`) | V1/V2 as built (`O`) |
| --- | --- | --- | --- |
| Max edges in a chain | 16 | **1** | 16 |
| Cumulative canonical closure | 131,072 B | 16,384 B (base+target ≤ 16 KiB) | 131,072 B |
| Per-record canonical | ≤ 8192 B | ≤ 8192 B | ≤ 8192 B |
| Encoded closure check | ≤ 17×8193 (`S/combined/store_api.py:69`) | — | — |
| Logical pool-value work | — | — | ≤ 196,608 B (192 KiB) |
| Match work budget per root | 16 MiB, ≤512 trials | 16 MiB, ≤512 trials | 16 MiB, ≤512 trials |
| Base chronology | base pack_id < target pack_id | previous snapshot only | base pack_id < target pack_id |
| Observed max depth | 15 | 1 | 15 |
| Observed max canonical closure | 130,223 B (53) / 130,304 B (157) | 16,288 B | 130,223 / 130,304 B |
| Matcher / instruction budget skips | 0 | 0 | **0** |

Sources: `S/metadata/experiment.py:80-90,170,198`; `O/metadata/experiment.py:31-40,63-64`;
`O/metadata/reader.py:35-38`; `O/metadata/result.json` (`maximum_depth:15`, `maximum_closure:130223`,
`maximum_pool_canonical_work:150306`, `counters.match_budget_skips:0`,
`counters.instruction_budget_skips:0`); `docs/.../issue100/history-scaling-and-metadata-deltas.md:37-58`.

**Candidate policy (single, frozen, not swept).**

1. Only inode-table **leaves** in the new snapshot are eligible; target and base each ≤ 8192 canonical.
2. The base must come from the **immediately previous root's** leaf list
   (`O/metadata/experiment.py:46,56-62`; `previous = rootleaves[step-1]`).
3. Ranking: greatest number of shared InodeSerials, tie-break `min((-overlap, origin))`
   (`O/metadata/experiment.py:59-61`); a key-range pre-filter rejects disjoint leaves early.
4. The base must already be in `records` and chronologically earlier
   (`assert first[origin] < step and origin in records`, `:62`).
5. The base's own chain must satisfy `depth < 16` and `base_closure + target ≤ 131072`; otherwise the
   target is counted `closure_rejected` and falls back to FULL (`:63`).
6. The matcher is called on the **physical (pooled) leaf bytes**, not the original 81-byte form
   (`O/metadata/protocol.md:7`). Evidence that this changes the outcome: 53-state `selected_delta` is
   2,237 (V1, pooled) vs 2,721 (chain experiment, unpooled).
7. Group accept rule — literally the product's rule (`P/.../objects/pack.rs:709-715`): a MIXED group is
   chosen iff `FULL_encoded − MIXED_encoded ≥ max(64, ceil(FULL_encoded/8))`
   (`O/metadata/experiment.py:71`). Otherwise **every** record in that group stays FULL — a pending
   candidate record is not kept per-object.
8. Base accounting: every retained FULL base is counted once, in the same pack population.

**Measured outcomes.** Depth-1 policy across 8,930 eligible leaves: 4,242 selected DELTA,
**4,060 forced back to FULL because the chosen origin was itself a DELTA**, 465 no overlapping
previous leaf, 161 group-rejected, 1 no matcher candidate, 1 genesis
(`docs/.../history-scaling-and-metadata-deltas.md:47-58`). The depth-16 chain removes most of that
4,060 fallback: on 53 states `selected_full_objects:7821`, `selected_delta_objects:2721`,
`status_origin_closure_bound:70` (`S/metadata/result.json`). Pool-based V1/V2 graph: 2,237 DELTA of
10,542 metadata objects (53) and 6,952 of 24,748 (157).

### 1.7 How a metadata leaf is reconstructed and authenticated

Read-time algorithm (`O/metadata/v2-53/reader.py:21-49`; chain walk inherited from
`S/combined/store_api.py:29-94`):

1. **Locate.** `objects(object_id) → (canonical_length, pack_id, group_number, record_number)`
   (SQL `P/.../sql/objects/get.sql`). `canonical_length` is the **original** canonical length.
2. **Catalogue lookup** (pool values only). `bisect` the group `first_ordinal` table; the covering row
   gives `(count, pack_id, group_number, group_sha256)`; `slot = ordinal − first_ordinal`.
3. **Fetch pack.** Whole pack BLOB (≤256 KiB) from `object_packs`; LRU-cache it (8 MiB).
4. **Group bounds.** `blob[:8]=="LFPACK\0\0"`, `u32@8 == 1`, `group_count ≤ 256`, `0 ≤ g < group_count`,
   `16+16·group_count ≤ offset`, `offset+encoded ≤ len(blob)`, `0 < decoded ≤ 16384`, `codec ∈ {0,1}`
   (`O/metadata/v2-53/reader.py:27-29`).
5. **Decompress** the whole group frame, assert `len(body) == decoded`, then
   **`SHA-256(body) == group_sha256`** — the group is authenticated *before* the record is sliced.
6. **Slice the record** `slot`: validate `u32(body[0]) == count`, the offset directory,
   `end − last == 95`, `record[0]==0`, `record[14:22]==b"LFSIVL1\0"`, `canonical[:5]==b"LFSO\1"`,
   `>II@5 == (85,81)` (`:32-39`).
7. **Chain walk** (metadata object): `record[0]==0` ⇒ FULL, body = `record[1:]`; `record[0]==1` ⇒ DELTA,
   `base = record[1:33]`, then enforce `depth ≤ 16`, `Σ canonical_length ≤ 131072`,
   `base.pack_id < node.pack_id`, `base ∈ objects` (`S/combined/store_api.py:60-77`).
8. **Replay** in reverse order: `matcher.replay(record, previous_id, previous)` with `previous` = the
   decoded base body; the result must still satisfy `leaf(body)` (`S/combined/store_api.py:79-87`).
   Replay is the product's `apply_delta`.
9. **Expand the pool.** `_expand(body)` yields `body[:44] ‖ (serial ‖ pool_value(ordinal)[-73:])×N`
   with `(len(body)−44) % 12 == 0`, `1 ≤ N ≤ 100`, and `Σ N·94 ≤ 196,608` (192 KiB logical pool bound)
   (`O/metadata/v2-53/reader.py:46-49`).
10. **Authenticate the original leaf.** The reconstructed canonical bytes must hash to the `objects` row
    key: `blake3(b"layerfs/object/v2\0" ‖ canonical) == object_id`, with
    `len(canonical) == canonical_length`. In the prototype this is enforced inside `_put`
    (`F/combined/store_api.py:162-167`) and then proven exhaustively
    (`O/metadata-proof-53/`, `O/metadata-proof-157/`): *all* original metadata objects and *all* pooled
    values were re-derived and compared.

The graph is a **forest of chains** (each node has exactly one physical base, pack_ids strictly
increase, no cycles), not a general DAG. **There is no checkpoint or skip pointer** — a cold read at
depth 15 reads 15 packs.

Negative fixtures that must fail (all PASSed on both copies): `missing_group`, `wrong_ordinal`,
`bad_group_digest`, `corrupt_group`, `wrong_value_role`, `canonical_leaf_hash`
(`O/metadata/v2-53/negative-check.py`, `negative-check.json`).

---

## 2. SQLite-level deltas vs the current product schema 9

### 2.1 What the product has today

`P/.../sql/schema/v9.sql` — tables `object_packs`, `objects`, `commits`, `branches`, `layer_stacks`,
`layers`, `workspace_stages`; indexes `layer_stack_names`, `layer_identity`, `layers_genesis`,
`layers_child`, `layers_source`, `branch_identity`, `branch_names`
(7 tables, 28 columns, 7 indexes — asserted at `P/.../src/statements.rs:255-290`).
`objects` is `(object_id BLOB PK, canonical_length, pack_id → object_packs, group_number 0..255,
record_number 0..8191) STRICT, WITHOUT ROWID` (`v9.sql:9-16`).

**`v9.sql` and `v8.sql` are byte-identical except `PRAGMA user_version`** (verified by `diff`), and
`migrate_to_v9.sql` is a single `PRAGMA user_version = 9;`. So schema 9 introduced **no DDL** — it only
unlocked `small_chain_format()` (`P/.../src/schema.rs:172`).

### 2.2 Tables / columns / indexes / packs: added, changed, removed

| # | Change | V1 | V2 | Evidence |
| --- | --- | --- | --- | --- |
| 1 | **+** `shared_inode_values(ordinal INTEGER PK, object_id BLOB UNIQUE NOT NULL REFERENCES objects)` `STRICT` | added | **removed** | `O/metadata/experiment.py:100`; `O/metadata/v2-53/build.py:24` |
| 2 | **+** `sqlite_autoindex_shared_inode_values_1` (UNIQUE on `object_id`) | added | **removed** | implicit |
| 3 | **+** `pool_groups(first_ordinal PK, count, pack_id → object_packs, group_number, group_sha256)` `STRICT` | — | **added** | `O/metadata/v2-53/build.py:25` |
| 4 | **~** `objects` — 66,529 / 89,576 pooled-value rows **deleted** | rows present | rows deleted | `O/metadata/v2-53/build.py:24`; builder also asserts the surviving rows are byte-equal to the source (`build.py:28`) |
| 5 | **~** `objects.canonical_length` semantics — column unchanged, but for pooled metadata leaves the stored physical record is shorter than `canonical_length` | yes | yes | §1.4 |
| 6 | **~** `object_packs` — old metadata packs deleted and replaced by the V1/V2 metadata + pool packs, pack_ids rebased above the prior `max(pack_id)` | yes | yes | `O/metadata/experiment.py:97-99`; V2 keeps identical bytes (`build.py:20,30`) |
| 7 | **=** all typed SQL history (`layers`, `commits`, `branches`, `layer_stacks`, `workspace_stages`, `scope_allocator`) byte-for-byte unchanged | yes | yes | `O/metadata/experiment.py:103`; `build.py:29` |
| 8 | **=** all content packs and content locators unchanged | yes | yes | `all_packs_byte_equal_v1`, `unchanged_content_locators_and_typed_SQL` = PASS |
| 9 | **~** `PRAGMA user_version` | 9303 | 9304 | diagnostic only; the product's value is 9 |
| 10 | **=** page size 4096, exactly one `VACUUM` | yes | yes | `O/metadata/v2-53/build.py:26` |

**Nothing is removed from the product schema.** V2 is purely additive at the DDL level (one table) plus
a change in *which* BLOBs live in `object_packs`.

### 2.3 Complete byte accounting

All figures are `dbstat` `sum(pgsize)` per object, or `st_size` on the actual files. Decimal bytes.

**(a) Metadata V2 stage — 53 states**

| dbstat object | Matched control | V1 (reject) | **V2 (retain)** |
| --- | ---: | ---: | ---: |
| `object_packs` (all packs) | 56,299,520 | 54,489,088 | 54,489,088 |
| `objects` (identity index) | 3,379,200 | 6,483,968 | 3,379,200 |
| `shared_inode_values` | — | 2,748,416 | — |
| `sqlite_autoindex_shared_inode_values_1` | — | 2,723,840 | — |
| `pool_groups` | — | — | **24,576** |
| typed SQL tables + `sqlite_schema` | 81,920 | 81,920 | 81,920 |
| **Database** | **59,760,640** | **66,527,232** | **57,974,784** |

**(b) Metadata V2 stage — 157 states**

| dbstat object | Matched control | V1 | **V2** |
| --- | ---: | ---: | ---: |
| `object_packs` | 74,833,920 | 66,981,888 | 66,981,888 |
| `objects` | 4,829,184 | 9,003,008 | 4,829,184 |
| `shared_inode_values` | — | 3,698,688 | — |
| `sqlite_autoindex_shared_inode_values_1` | — | 3,678,208 | — |
| `pool_groups` | — | — | **32,768** |
| typed SQL + `sqlite_schema` | 126,976 | 126,976 | 126,976 |
| **Database** | **79,790,080** | **83,488,768** | **71,970,816** |

Baseline `object_packs` and `objects` rows are derived by subtracting the published per-component deltas
(`O/metadata/report.md:18-26`: `object_packs −1,810,432 / −7,852,032`; `objects index +3,104,768 /
+4,173,824`; ordinal table `+2,748,416 / +3,698,688`; ordinal UNIQUE index `+2,723,840 / +3,678,208`;
total `+6,766,592 / +3,698,688`) from the V1 rows; every column sums. The decisive identity for V2:

```
53 :  −1,810,432 (object_packs pages) + 24,576 (pool_groups)  = −1,785,856   ✔ matches result.json
157:  −7,852,032 (object_packs pages) + 32,768 (pool_groups)  = −7,819,264   ✔ matches result.json
```

**Which packs.** Measured by scanning `object_packs` in the actual candidate files:

| Pack family | Magic / version | 53 packs | 53 bytes | 157 packs | 157 bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| Metadata CAS packs (inode tables, roots, directories, metadata, link/meta nodes) | `LFPACK\0\0` / 1 | 138 | **2,458,109** | 388 | **6,156,885** |
| Pool value packs (FULL-only, hash-sorted) | `LFPACK\0\0` / 1 | 26 | **2,545,576** | 34 | **3,446,289** |
| ⇒ metadata subtotal | | **164** | **5,003,685** | **422** | **9,603,174** |
| Content packs (whole-file / SmallContent / native) | `LFCNT1\0\0` / 107 | 243 | 45,494,501 | 310 | 50,710,265 |
| **All packs** | | **407** | **50,498,186** | **732** | **60,313,439** |

Group counts in the same files: metadata packs hold 1,827 groups (53) / 5,154 (157); pool packs hold
exactly the 404 / 543 catalogue groups. Metadata pack byte reduction vs baseline:
`6,877,152 − 5,003,685 = 1,873,467` (53) and `17,459,061 − 9,603,174 = 7,855,887` (157). The page
reductions above are slightly smaller (1,810,432 / 7,852,032) — the residual 63,035 B / 3,855 B is
page- and overflow-granularity, not payload.

**(c) Content stage and the two headline totals.** The content stage's source is the V2 candidate; it
replaces only content packs and adds 224 / 523 whole-file object rows (`O/content/report.md:3-31`,
`O/content/full157/result.json`).

| dbstat object | 53 final | 157 final |
| --- | ---: | ---: |
| `object_packs` | 50,860,032 | 60,919,808 |
| `objects` | 3,416,064 | 4,878,336 |
| `pool_groups` | 24,576 | 32,768 |
| `branches` / `commits` | 12,288 / 12,288 | 24,576 / 28,672 |
| `branch_identity`, `branch_names`, `layer_identity`, `layer_stack_names`, `layer_stacks`, `layers`, `layers_child`, `layers_genesis`, `layers_source`, `scope_allocator`, `workspace_stages` | 11 × 4,096 = 45,056 | 9 × 4,096 = 36,864 |
| `sqlite_schema` | 12,288 | 12,288 |
| **Total** | **54,382,592** | **65,957,888** |

Reconciliation:

```
53 :  59,760,640  −1,785,856 (metadata V2)  −3,592,192 (content)  = 54,382,592
157:  79,790,080  −7,819,264 (metadata V2)  −6,012,928 (content)  = 65,957,888
```

Content-stage internals: 53 — content packs `48,824,145 → 45,494,501` (−3,329,644) plus −262,548 B of
page/overflow effects = −3,592,192 (`O/content/report.md:24-27`); 157 — content packs
`56,367,207 → 50,710,265` (−5,656,942) plus −355,986 B = −6,012,928.

`objects` row counts: **72,532** (53) and **103,890** (157) = metadata objects (10,542 / 24,748) +
content objects (61,766 / 78,619) + new whole-file objects (224 / 523). Both headline copies were
verified end-to-end: 53 states / 306,861 path-states / 1,676,767,835 logical bytes and 157 states /
904,143 path-states / 4,936,693,030 logical bytes (`ordered-optimization-results.md:7`).

**Reminder carried by the same documents:** these totals are *archive/reference* results with severe
cold-read amplification; `ordered-optimization-results.md:5,57` state that no production implementation
or qualification was performed and that this must not be treated as a hot-filesystem recommendation.

---

## 3. Read path, amplification, caches

### 3.1 Runtime resolution of one metadata leaf

1. `SELECT canonical_length, pack_id, group_number, record_number FROM objects WHERE object_id = ?1` —
   one index seek in the `WITHOUT ROWID` `objects` B-tree (SQL `P/.../sql/objects/get.sql`; batch form
   `get_many_128.sql`; membership-only form `membership_128.sql`).
2–9. As in §1.7 — pack fetch, group decompress + digest check, record slice, chain walk (≤16 packs),
   reverse replay, pool expansion (≤100 ordinals → ≤100 catalogue lookups → ≤100 group fetches), final
   `blake3` identity check.

**Rows read:** 1 `objects` row + 1 `object_packs` row per distinct pack touched (up to 16 for the chain,
plus up to N for the pool expansion) + 1 `pool_groups` row per distinct pool group (the bisect itself is
in-memory, but every group hit costs a row read unless the whole catalogue is resident).

**Two independent amplification layers:**

* **Chain amplification** (metadata DELTA graph): bounded at 128 KiB *canonical*; measured max 130,304 B
  for one leaf.
* **Pool-group amplification** (new in V1/V2, bounded by no format rule): hash-sorted ordinals scatter
  one leaf's ≤100 values across many independent 16 KiB groups, so a single cold leaf can touch hundreds
  of groups. `O/metadata-cold-work.py:33-34` computes exactly this — the union of group `(pack,group)`
  pairs over every pooled value in the chain **and** every `(pack,group)` of the chain's own records,
  de-duplicated per target.

Measured cold cost for one leaf, counting each group once (`O/metadata-cold-53.json`,
`O/metadata-cold-157.json`, produced by `O/metadata-cold-work.py`):

| Cold single-leaf requirement | 53 states | 157 states |
| --- | ---: | ---: |
| Decoded pool-value group bytes | **4,705,632** | **5,765,786** |
| Decoded metadata **+** value group bytes | **4,725,604** | **5,785,404** |
| Encoded bytes of the same groups | 1,824,220 | 2,244,872 |
| Distinct pool-value groups | **288** | **353** |
| Distinct pool-value packs | 26 | 34 |
| Whole-pack fetch if each touched pack is read entirely | **2,854,074** | **3,744,557** |
| Chain depth / canonical closure at the worst target | 15 / 129,413 | 13 / 112,315 |
| Distinct pool values in that leaf | 487 | 570 |

In words: reconstructing **one** cold metadata leaf can require **4.7 MB / 5.8 MB of decompressed group
bodies across 288 / 353 groups in 26 / 34 packs**, against a 130 KiB canonical closure bound and a 192 KiB
logical pool-byte bound. The 128 KiB canonical dependency limit does **not** bound decompression or
pack-fetch amplification (`ordered-optimization-results.md:36`; `O/metadata/report.md:48-60`).

### 3.2 Caches and their limits

Prototype reader caches (`O/metadata/reader.py:7`; `O/metadata/v2-53/reader.py:17-19`;
`F/combined/store_api.py:132-137`):

| Cache | Limit | Notes |
| --- | --- | --- |
| Canonical reconstructed objects (LRU) | 4 MiB | objects larger than the limit are never inserted |
| Encoded pack payload (LRU, whole pack BLOBs) | 8 MiB | one pack ≤ 256 KiB |
| Decoded pool-group bodies (LRU) | 4 MiB | keyed by `first_ordinal` |
| Extracted pool record copies | up to ~4 MiB | **not** in the frozen protocol; recorded as a clarification in `O/metadata/v2-53/execution-notes.md` |
| ⇒ retained payload | **~20 MiB** | ~4 + 8 + 4 + 4, before Python container overhead |

`O/metadata/v2-53/execution-notes.md` states plainly that the 4 MiB pool budget is **not** a total memory
bound and that the 192 KiB logical value bound does not bound cold group amplification.
`O/content/report.md:48` gives the same inherited accounting as "about 20 MiB of retained payload …
not an RSS cap".

Diagnostic reader work actually executed during V2 verification (whole-population pass, warm caches):
53 — `pool_lookups 2,156,335`, `pool_canonical_logical_bytes 202,695,490`,
`decoded_pool_group_bytes 5,153,664,670`, `maximum_pool_canonical_work 150,306`,
`maximum_original_canonical_closure 130,223`; 157 — `pool_lookups 9,081,218`,
`pool_canonical_logical_bytes 853,634,492`, `decoded_pool_group_bytes 23,434,802,308`,
`maximum_pool_canonical_work 150,400`, `maximum_original_canonical_closure 130,304`
(`O/metadata/v2-53/result.json`, `O/metadata/v2-157/result.json`).

**The product read path has none of these caches.** `P/.../objects/read.rs` uses SQLite incremental BLOB
I/O (`connection.blob_open("main","object_packs","data",…)`, e.g. `read.rs:416,530,956,996,1309`) —
header (16 B) + directory entry (16 B) + one group frame, then it decompresses **one demanded group**,
explicitly documented: "One demanded group only; no unrelated record bodies or persistent cache"
(`read.rs:513`). The only read budgets are `HintReadBudget` (≤512 KiB encoded/decoded per target and
≤8 MiB per batch, for *optional predecessor discovery* only — `read.rs:33-58`) and `validation_reserve`
(`read.rs:111-120`). The `small_chain`/`native_chain` closure bounds are `CHAIN_EDGES = 8`,
`CHAIN_CANONICAL_LIMIT = 512 KiB`, `CHAIN_ENCODED_LIMIT = 256 KiB` (`P/.../objects/delta.rs:7-9`) —
**different from** the prototype's 16 edges / 128 KiB for metadata.

---

## 4. Mapping table: prototype feature → current Rust owner

Left column = what the prototype does; right column = where an implementer must change or mirror it in
**P** today. `P/` = `crates/layerfs-layerstack-store/`, `C/` = `crates/layerfs-content/`, relative to
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb`.

| Prototype feature (file:line) | Current Rust owner (file:line) | Role |
| --- | --- | --- |
| Canonical envelope `LFSO\1` + length pair; `value()` slice (`S/metadata/d/codec.py:11-14`) | `C/src/object/codec.rs:11-12,22-70` (`MAGIC`, `HEADER_LEN`, `encode_bytes_object`, `decode_bytes_object`) | both |
| Identity `blake3("layerfs/object/v2\0"‖canonical)` (`S/metadata/d/codec.py:10`) | `C/src/object/digest.rs:5-7,44-52`; `C/src/object/id.rs:16-22`; `C/src/object/canonical.rs:92-96` | both |
| Identity re-derivation + length check (`F/combined/store_api.py:162-167`) | `C/src/object/codec.rs:163` `authenticate_identity` | read |
| Node header (31 B) for `LFS6INT`/`LFS6NSP`/`LFS6FSR` (`S/metadata/d/codec.py:110,118,153`) | `C/src/tree/directory/codec.rs:269-285` `node_header`; `:310-318` `validate_node_header`; `:289,:343` `NODE_LIMIT = 8192`; `:235` `exact_value` | both |
| Compact inode leaf/branch `LFS6INT` role 7/8, 81-/40-byte entries, ≤100/≤127 (`S/metadata/experiment.py:10-17,41-43`) | `C/src/tree/compact.rs:53-124` (`InodeNode`, `encode_inode`, `decode_inode`); root `:26-50`; directory `:127-183` | **read-only today; no writer exists** |
| Inode *value* = 73 B (kind ‖ refcount ‖ content_root ‖ metadata_root) (`S/metadata/d/scoped.py`) | `C/src/tree/compact.rs:73-77,101-106` (`InodeRecordV1`); legacy equivalent `C/src/tree/inode/codec.rs:107+` (`LFS4INO`) | both (legacy path) |
| Legacy inode table `LFS4INT` (32-byte id ‖ 32-byte record id, 64-B entries) — what the prototype replaces | `C/src/tree/inode/codec.rs:18-105`; written from `C/src/filesystem/apply.rs:122,176,198,254,282,574,623,673`; `C/src/filesystem/reconcile.rs:199,348`; `C/src/tree/batch.rs:804` | write |
| **Pool value object** `LFSIVL1` + 73 B, canonical 94 B (`O/metadata/experiment.py:9,12-14`) | **no owner** — needs a new role in `C/src/tree/compact.rs` and/or `C/src/object/references.rs`, plus a decision on whether it is an `objects` row (V1) or catalogue-only (V2) | new |
| **Pooled leaf** = 44 B prefix ‖ (serial 8 ‖ ordinal 4)×N (`O/metadata/experiment.py:29`) | **no owner** — a new `InodeNode` variant; must interact with the canonical-length equality check at `C/src/tree/compact.rs:98` and with `C/src/object/references.rs:27-31` | new |
| **`pool_groups` catalogue + 32-byte group digest** (`O/metadata/v2-53/build.py:25`; reader `O/metadata/v2-53/reader.py:12-16`) | **no owner** — new DDL; must be added to a `v10.sql` successor plus the schema gate (`src/schema.rs:425-490`, `src/statements.rs:59-71`) and the `pragma_foreign_key_check` sweep (`sql/schema/foreign_key_check.sql`) | new |
| **Group body** = u32 count ‖ u32 ends ‖ records, ≤16 KiB (`S/metadata/experiment.py:73-78`) | `P/src/objects/pack.rs:699-789` `encode_group`/`group_alternative`; `:44-56` `header`; `:113-160` `Header`/`versioned_header`/`versioned_entry`; `:856` `decode_group` — same grammar, but `GROUP_LIMIT = 65,536` (`:6`) | both |
| **Pack assembly** `LFPACK` v1 + directory + frames; 256 KiB / 256 groups / 8191 records (`S/metadata/d/pack_api.py:27-40`; `S/metadata/experiment.py:118-124`) | `P/src/objects/pack.rs:791-855` `assemble`; limits `:6-10`; `:44-56` `header`; `:57` `entry`; `:108-160` versioned forms | both |
| **Delta record grammar** FULL/DELTA + COPY/INSERT (`H/tools/matcher.py:22-41`) | `P/src/objects/pack.rs:397-429` `record`; `:431-474` `walk_instructions`; `:475-500` `apply_delta`; `:501-539` `visit_records` | both |
| **Delta producer** = product matcher applied to metadata leaves (`O/metadata/experiment.py:65`; `H/tools/delta_record.rs`) | `P/src/objects/pack.rs:549-697` `delta_record` (**identical source text**) | both |
| **Delta base choice as it exists today** (compact candidate, predecessor hint, chain) | `P/src/objects/admission.rs:1231-1340` (`candidate` search); `:215-275` (predecessor + `small_chain_format`); `P/src/objects/delta.rs:3-46` (`Record`, `FRAME_LIMIT`, `CHAIN_*`, `encode`, `decode`) | both |
| **Group accept threshold** `max(64, ceil(FULL/8))` (`O/metadata/experiment.py:71`) | `P/src/objects/pack.rs:709-715` `64.max(full.bytes.len().div_ceil(8))` | both |
| **Depth/closure bounds**: 16 edges, 128 KiB canonical, 192 KiB pool work, 16 MiB/512 trials (`O/metadata/experiment.py:31-40,43,63-64`) | analogous but **different values** today: `P/src/objects/delta.rs:7-9` (`CHAIN_EDGES = 8`, `512 KiB`, `256 KiB`); enforced at `P/src/objects/read.rs:289-330` and `P/src/objects/admission.rs:273` | both |
| **Reference/dependency enumeration** (`S/metadata/experiment.py:12-35` `edges()`) | `C/src/object/references.rs:10-` `referenced_objects` (the `LFS6` arms are the working-tree addition at `:26-34`); consumers `P/src/objects.rs:2659` (cycle/closure validation), `:2743`, `:3526` (`close_dependencies`), `P/src/query.rs:473` | read |
| **Closure/cycle validation over new objects** (`S/metadata/experiment.py:80-90` `closure`) | `P/src/objects.rs:2650-2680` (cycle detection + encoded-byte accounting); `:3515-3535` `close_dependencies` | read |
| **Pack/object publication** (delete old packs, insert new packs + locators, rebase ids) (`O/metadata/experiment.py:97-99`) | `P/src/objects/admission.rs:1052-1150` (`MAX(pack_id)` → `INSERT INTO object_packs` → `INSERT INTO objects`); SQL `sql/objects/insert.sql`; constants `P/src/statements.rs:74-79` | write |
| **Commit publication** (prototype rewrites `layers`/`commits` ids; `F/combined/store_api.py:21-23,26-47`) | `P/src/workspace.rs:278-420` `commit_candidate`, `:423` `commit_workspace_candidate`; `INSERT_COMMIT` at `:336,:534` (`sql/workspace/insert_commit.sql`); `ADVANCE_BRANCH` at `:366,:564`; `P/src/layerstack.rs:208` `add_layer`; id derivation `P/src/ids.rs` | write |
| **Reopen / schema verification / recovery** (prototype checks `pragma user_version` at open — `O/metadata/reader.py:9`, `S/combined/store_api.py:23`) | `P/src/schema.rs:355` `preflight_connect` (accepts 6/7/8/9); `:425-441` `verify_schema` (page_size ∈ {4096,65536}, `schema_objects` equality, FK check); `:470-490` `schema_objects`/`expected_schema_objects`; `:794` `upgrade_format`; `:809` `MIGRATE_TO_V9`; public façade `P/src/store.rs:11` `upgrade_format`; pack-version probe `P/src/schema.rs:383` and `P/src/schema/compatibility.rs:48` | both |
| **Orphan-pack / abandoned-batch recovery** (prototype: none) | `P/src/objects.rs:2152-2185` (`DELETE FROM objects` then `DELETE FROM object_packs` above `baseline_pack`); spill/deferred machinery `P/src/objects/spill.rs`, `P/src/objects.rs:2968-3060` | write |
| **Page size / VACUUM decisions** (`O/metadata/v2-53/build.py:26`) | `P/src/schema.rs:14` `NEW_STORE_PAGE_SIZE_BYTES = 4096`, applied at `:223`; `verify_schema` accepts only 4096 or 65536 (`:434`) | both |

---

## 5. Open risks and unknowns the implementer must resolve

**5.1 `canonical_length` vs physical record length.** Pooled leaves and pool value objects have a stored
record that is *not* the canonical serialization of the key in `objects`. The product asserts length
equality in several places — `P/src/objects/read.rs:256-258`
(`canonical_length != parsed.raw_length + 23`), `read.rs:283-292`, `P/src/objects/delta.rs:31-34`,
`C/src/object/codec.rs:163` (`authenticate_identity`). **Every one of those checks has to be
re-specified**, and the semantic must be chosen explicitly: does `objects.canonical_length` mean
"reconstructed canonical length" (the V1/V2 choice) or "physical record length" (today's meaning)? The
prototype simply overrode its readers.

**5.2 `pool_groups` is a new table ⇒ a new schema version.** `verify_schema` compares the live
`sqlite_schema` against a freshly-executed DDL text (`P/src/schema.rs:437,479-490`); `preflight_connect`
whitelists versions (`:374`); `P/src/statements.rs:255-290` asserts the exact table/index inventory.
Adding V2 requires a new `vN.sql`, a migration, a new `SCHEMA_VERSION`, a new `expected_schema_objects`
arm, and updated inventory assertions. `pool_groups.pack_id` also participates in
`pragma_foreign_key_check` (`sql/schema/foreign_key_check.sql`).

**5.3 Ordinals are not identities.** The prototype's own protocol says so
(`O/metadata/protocol.md:5`: "Physical ordinals are locators, never truncated identities"). Every pooled
value's full 32-byte identity must stay reachable — which is exactly why V1's per-value CAS rows cost
3.1 MB / 4.2 MB of `objects` pages — and V2 sidesteps that by **deleting the rows entirely**, so a pooled
value is no longer individually addressable by `objects`. Nothing in the product today can look up an
object by anything but `object_id`. Whether any product path needs `value_id → value` lookup is **not
determinable from this evidence**.

**5.4 Cold-read amplification is bounded by no rule.** 4.7 MB / 5.8 MB of decompressed groups per cold
leaf, spread over 288 / 353 groups in 26 / 34 packs, against a 130 KiB canonical bound and a 192 KiB
logical bound. `ordered-optimization-results.md:5,57` and `O/metadata/report.md:48-60` refuse to call
this acceptable. **No product read budget models it**: `HintReadBudget` covers only optional predecessor
discovery, and there is no pool cache at all. A pool-aware budget (decoded group bytes, groups touched,
packs touched) does not exist and must be designed.

**5.5 Group-level authentication is not in the product.** The product authenticates whole objects and
validates pack/group framing, but `LFPACK` v1 has **no per-group digest field** (`P/src/objects/pack.rs:44-56`
carries only offset/lengths/codec). V2's `group_sha256` authenticates a group *before* its record
directory is trusted. Putting it in the pack format changes the container; putting it in SQL keeps the
container unchanged but authenticates *bytes already fetched* rather than bytes on disk independently of
SQLite.

**5.6 Ordinal stability is a whole-store invariant — the largest unknown.** `first_ordinal` must be
contiguous from 1 and cover every pooled value (`O/metadata/v2-53/reader.py:14-16`). The prototype builds
the pool **once, offline, from the complete history**. On a live incrementally-written store, adding a new
distinct value would renumber ordinals (they are hash-sorted) and invalidate every pooled leaf. **No
online ordinal-assignment, pool-growth, or write-path migration algorithm was designed or measured.**

**5.7 Depth-16 chains vs the product's depth-8 SmallContent chain.** The prototype allows 16 edges /
128 KiB for metadata; `P/src/objects/delta.rs:7-9` allows 8 / 512 KiB. An implementation either reuses the
SmallContent bounds or introduces per-role bounds enforced at `P/src/objects/read.rs:289-330` and
`P/src/objects/admission.rs:273`; neither was trialled as a product change.

**5.8 16 KiB metadata group vs 64 KiB product group.** The prototype's digest and 16 KiB bound are
asserted at `S/metadata/experiment.py:77` and checked at `S/combined/store_api.py:38`. **No experiment
measured group-size sensitivity**, so the §3 amplification numbers are specific to 16 KiB groups.

**5.9 The depth-one variant lost 4,060 of 8,930 targets** to the "base must be FULL" rule
(`docs/.../history-scaling-and-metadata-deltas.md:47-58`). The depth-16 chain recovered most of it, but the
same document warns this "is not evidence that removing all depth limits is safe". No depth other than 1
and 16, and no second candidate/anchor policy, was measured.

**5.10 Content coupling.** The V2 53-state `objects` index (3,379,200 B) is unchanged by V2, but the
content stage then adds 224 whole-file objects and rewrites all content packs; its 3,592,192 B saving
includes 262,548 B of page effects (`O/content/report.md:27`). **A metadata-only change cannot be
subtracted arithmetically from the 54,382,592 B total** — the stages interact through pack density and
VACUUM.

**5.11 No measured product-side latency or memory.** All cold numbers are Python-prototype diagnostics
with an uncontrolled OS cache; `ordered-optimization-results.md:57` calls them "scoped diagnostic
observations, not public API latency claims". The 4 MiB / 8 MiB / 4 MiB + 4 MiB cache accounting is
explicitly *not* an RSS cap and exists only in Python.

**5.12 Explicitly not determinable from this evidence:** whether `LFSIVL1` must become a first-class object
role with edges in `referenced_objects`; whether pooled values must stay reachable via
`object_ids_in_order` / `visit_locations`; whether `visit_wave` / `compare_singleton`
(`P/src/objects/read.rs:1009-1270`) can run over pooled leaves; and whether the graph's
"previous snapshot only" base choice is reproducible from a live store's publication order. Each needs
product paths outside this task's evidence.

---

## Appendix A — measured metadata population (why pooling was tried at all)

From `O/metadata-population.json` (`O/metadata-population.py`):

| | 53 states | 157 states |
| --- | ---: | ---: |
| Inode-table leaf objects | 3,084 | 8,930 |
| Canonical leaf bytes | 24,931,011 | 72,121,903 |
| Inline 73-byte value occurrences | **306,115** | **885,543** |
| Distinct values | **66,529** | **89,576** |
| Copies per value | **4.60×** | **9.89×** |
| Raw repeated-value bytes | 17,489,778 | 58,105,591 |

The source is explicit that "raw repetition is not compressed savings"
(`O/metadata-population.json`, `scope` field). The measured whole-database outcome after adding the pool
and removing the per-value indexes was −1,785,856 B / −7,819,264 B. V1 failed because it kept *both* the
per-value CAS rows and an ordinal→fullhash UNIQUE index; **the win comes from deleting those index
structures and keeping the pool data itself catalogue-only** — the pool payload (2,545,576 B / 3,446,289 B)
is a cost that the removed index pages more than pay for at these history lengths.

---

## Appendix B — evidence index

| Claim | Artifact |
| --- | --- |
| V1/V2 physical results, cold-read limitation, custody | `O/metadata/report.md` (mirror: `docs/.../issue100/experiments40/ordered-optimization/metadata/report.md`) |
| V2 catalogue DDL + build/verify assertions | `O/metadata/v2-53/build.py`; `O/metadata/v2-157/build.py` |
| V2 reader (catalogue, digest, record shape, pool expansion) | `O/metadata/v2-53/reader.py`; `O/metadata/v2-157/reader.py` |
| V2 negative fixtures | `O/metadata/v2-53/negative-check.py`, `negative-check.json` (+ 157) |
| Frozen policies (V1, V2, full157 extension) | `O/metadata/protocol.md`, `O/metadata/full157/protocol.md` |
| V1 result/counters; V2 result/dbstat | `O/metadata/result.json`, `O/metadata/full157/result.json`, `O/metadata/v2-53/result.json`, `O/metadata/v2-157/result.json` |
| Cache-accounting clarification (~8 MiB pool payload) | `O/metadata/v2-53/execution-notes.md`, `O/metadata/v2-157/execution-notes.md` |
| Depth-16 chain policy, bounds, ledger, counters | `S/metadata/experiment.py`, `S/metadata/chain_api.py`, `S/metadata/result.json`, `S/metadata/groups.json`, `S/metadata/ledger.csv` |
| Depth-one policy + measured outcome table | `H/metadata/protocol.md`, `H/metadata/report.md`, `docs/.../issue100/history-scaling-and-metadata-deltas.md:13-58` |
| Exact Rust matcher extraction + source hash | `H/tools/delta_record.rs`, `H/tools/delta_ffi.rs`, `H/tools/matcher.py`, `H/tools/source.json` |
| Pack/group/cache substrate the prototype inherits | `F/combined/store_api.py:130-200`, `S/combined/store_api.py` |
| Value population census | `O/metadata-population.py`, `O/metadata-population.json` |
| Cold-read accounting | `O/metadata-cold-work.py`, `O/metadata-cold-53.json`, `O/metadata-cold-157.json` |
| Content stage and headline totals | `O/content/report.md`, `O/content/result.json`, `O/content/full157/result.json` |
| Campaign decisions and scorecard | `docs/.../issue100/ordered-optimization-results.md`, `docs/.../issue100/optimization-checklist-and-experiment-ledger.md` |
| Prior structural (checkpoint vs long chain) context | `docs/.../issue100/structural-investigations.md`, `S/metadata/report.md` |

# Compact namespace map: scoped serial inode identities, inline inode values, compact directory references, namespace roots

Research report for issue #103 (compact-namespace-first integration). **No source file was
modified by this audit.** Paths without a leading `/` are relative to
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb` unless prefixed `EXT:`.

Working-tree state at audit time (this matters for any "is this fresh?" claim):
`crates/layerfs-content/src/tree/compact.rs` is **untracked/new**;
`crates/layerfs-content/src/object/references.rs` and `crates/layerfs-content/src/tree/mod.rs`
are **modified**; `crates/layerfs-content/tests/fixtures/compact/*.bin` is untracked. Nothing
else in the tree differs from `HEAD`. The draft wires the compact codecs in exactly two
places: `tree/mod.rs:1` (`pub mod compact;`) and `references.rs:26-35` (LFS6 magic arms) plus
`references.rs:112-114` (a magic-safety test list). No product writer, reader, allocator,
schema or dispatch path consumes it yet.

The intended integration order is already fixed in
`docs/roadmap/0.1/0.1.5/issue102/integrated-candidate-v2.md:12-15`: "**Public compact
namespace: scoped serial inode IDs, inline inode values, compact directory references,
durable allocation and legacy compatibility**" is step 1, before pooled values, whole-file
content graphs, and the full benchmark rerun.

---

## 1. The offline prototype's compact namespace

### 1.1 Where the writer and reader live

The compact namespace is prototype experiment **"D"** ("Conditional experiment D — scoped
8-byte stable inode serials"). In-repo evidence copies (immutable, easiest to cite):

- Protocol (frozen before execution): `docs/roadmap/0.1/0.1.5/issue100/experiments40/metadata/protocol-compact-ids.md`
- Writer (the whole experiment, single script): `docs/roadmap/0.1/0.1.5/issue100/experiments40/metadata/compact_ids.py`
- Result report: `docs/roadmap/0.1/0.1.5/issue100/experiments40/metadata/report-D.md`
- Earlier A/B/C control: `docs/roadmap/0.1/0.1.5/issue100/experiments40/metadata/report.md`

External raw evidence roots (larger than the in-repo mirrors, same content):
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/metadata/` (D writer,
`result-D.json`, `roots-D.json`, `D-index.sqlite`, `d_api.py`) and
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/` (the downstream
pooled-value/candidate-index work that reads the same compact objects).

Key writer/reader functions in `.../experiments40/metadata/compact_ids.py`:

| Role | Function | Lines |
| --- | --- | --- |
| LFS6INT table writer (leaf 7 / branch 8) | `build_table` | 7-20 (`put` at 8-10, balancing at 12-19) |
| LFS6INT table reader | `decode_table` | 21-34 |
| LFS6NSP directory rewriter (LFS4NSP → LFS6NSP) | `transform_directory` | 36-49 |
| LFS6NSP directory reader | `decode_directory` | 51-64 |
| Allocator primitive | `class Allocator` | 66-70 |
| Focused allocator/identity checks (rename, hardlink, two branches) | `fixture` | 72-83 |
| Serial allocation + full inventory build + LFS6FSR roots | `build_inventory` | 85-128 (serial loop 87-91, scope 92, root 109) |
| `scope_allocator` table creation + persistence | `main` | 139-143 |

The same code is replayed unmodified at 53 states in
`docs/roadmap/0.1/0.1.5/issue100/experiments40/stride3-structural/metadata/d/scoped.py`
(identical structure, 129 lines; scope/serial/root at lines 87-92, 109-110) and at 157
states in `.../experiments40/full157/metadata/scoped.py`. Readers in the ordered-optimization
prototype: `EXT:/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/metadata/reader.py:26-30`
(`read_canonical`), `.../index/full157/reader-check.py:22` (root/table/pool probe) and
`.../metadata-population.py:13-18` (81-byte leaf row census).

### 1.2 Wire layouts (what the draft `compact.rs` reproduces)

Canonical envelope is the product's `LFSO` bytes-object framing: 9-byte header + 4-byte
length + value (`crates/layerfs-content/src/object/codec.rs:11-12, 22-37`), so
`canonical_len = value_len + 13`.

- **LFS6FSR namespace root** — value 116 B → canonical 129 B:
  `magic8 | version2 | role=6,flags=0 | profile32 | scope32 | root_inode_serial8(BE) |
  inode_table_root32`. Written at `compact_ids.py:109`; Rust `compact.rs:33-40` (`encode_root`),
  `compact.rs:42-50` (`decode_root`, `exact_value(..., 116, 6)`). Prototype only: `profile` is
  a diagnostic constant (`compact_ids.py:93`), `scope` is 32 B (`:92`), and the old 32-byte
  root inode id is replaced by the 8-byte serial at value offset 76-84.
- **LFS6INT inode table/branch** — 31-B header
  `magic8 | version2 | role,level,flags3 | count2 | subtree_count8 | subtree_bytes8`, then rows:
  leaf role 7 → `serial8(BE) | kind1 | namespace_ref_count8 | content_root32 | metadata_root32`
  = **81 B/row**, ≤100 rows, `subtree_bytes = total*81`; branch role 8 → `serial8 | child_objid32`
  = **40 B/row**, ≤127 rows, level ≥1. Max leaf canonical = 31+100·81+13 = **8144 B**.
  Rust `compact.rs:58-89` (`encode_inode`), `compact.rs:91-124` (`decode_inode`).
- **LFS6NSP compact directory node** — same 31-B header; leaf role 1 → `u16 len | name | serial8`
  (= 10+len), `subtree_bytes = Σ(10+len)`; branch role 2 → `u16 len | name | child_objid32`
  (= 34+len), children keep full 32-byte object ids. Rust `compact.rs:132-154` (`encode_directory`),
  `compact.rs:156-183` (`decode_directory`).

Crucially, **the inode record is inlined into the table leaf** (fields `kind`,
`namespace_ref_count`, `content_root`, `metadata_root`) — there is no LFS4INO-equivalent
object — and **the directory leaf's inode reference is an 8-byte serial, not a 32-byte id**.
The LFS4DIR directory-state wrapper is also dropped: the inlined record's
`content_or_directory` field points **directly** at the LFS6NSP root
(`compact_ids.py:106` `rec=rec[:9]+dir_cache[mapping]+rec[41:]`; equivalently
`build_inventory:116-121`).

### 1.3 Identity and allocator rules — what "scoped" means exactly

From `protocol-compact-ids.md:5` and `compact_ids.py:85-128`:

- **Scope** = `blake3(b'layerfs/diagnostic/scoped-inode-origin/v1\0' ++ roots[0])`
  (`compact_ids.py:92`) — one 32-byte value derived from a fixed diagnostic descriptor plus
  the **first/genesis namespace root** of the diagnostic run. It is written into **every**
  namespace root (`:109-110`, asserted equal at `:110`). Scope is therefore
  **one-per-Store/origin**, not per-state and not per-branch.
- **Canonical inode identity = scope + serial.** The serial alone is not an identity
  (`protocol-compact-ids.md:5`: "Scope+serial is the new canonical stable identity").
- **Serials are 8-byte big-endian, start at 1, never recycled, monotonic.** `Allocator`
  (`compact_ids.py:66-70`) asserts `highwater < 2^63-1`; `protocol-compact-ids.md:9` fixes the
  positive signed-64-bit SQLite range.
- **Allocation order is global across the whole run, not per state.** `build_inventory:87-91`
  walks the eleven roots **in original admission order**, and within each root the original
  sorted inode-entry order, assigning `serial[inode] = allocator.reserve()` **at first
  appearance only**. The same original 32-byte inode id maps to the same serial in every
  state. Result: 17,922 identities for eleven states (ten-state run, `report-D.md:22`),
  25,390 for the 53-state replay (`.../stride3-structural/metadata/report.md:15`). There is
  **no per-state counter reset** — allocation is one monotonic domain across branches, which
  the protocol explicitly requires (`protocol-compact-ids.md:9`: "This is global monotonic
  allocation across branches, not a counter reset from a historical namespace root").
- **Semantics preserved by construction**: a serial is assigned to a *distinct original inode
  id*, so rename keeps identity, two hardlink names share one serial with refcount 2, distinct
  allocations differ, and a shared ancestor keeps the same serial in two branches. Those are
  the five focused checks in `compact_ids.py:73-83` (status string at `:83`), and they are a
  **representation/allocation model only** — `:83` states the limit: "global allocation
  primitive model only; no product concurrent allocator/import implementation".
- **Durable allocator state**: a new SQLite table
  `scope_allocator(scope BLOB PRIMARY KEY CHECK(length(scope)=32), highwater INTEGER NOT NULL
  CHECK(highwater>=0)) STRICT, WITHOUT ROWID` (`compact_ids.py:139`), persisting the final
  highwater (`:140`, asserted at `:143`). Charged as one real 4096-byte dbstat page;
  logical payload 40 B (`report-D.md:24`).
- **The old 32-byte → serial map is a verification oracle only**, explicitly "not a runtime
  dictionary" (`compact_ids.py:148` `mapping_purpose`; `report-D.md:26`).

### 1.4 Did it rewrite canonical namespace/commit identities? Yes.

Changing the namespace root's canonical bytes changes its object id, and the product derives
typed ids from it (`crates/layerfs-layerstack-store/src/ids.rs:92-118`: `CommitId::derive`
hashes `root_id`, `LayerId::derive` hashes `root_id`). The prototype therefore had to
**rederive** them. From `docs/roadmap/0.1/0.1.5/issue100/40mb-experiment-results.md:79`:

> "The metadata encoding changes canonical namespace roots, so it also changes derived
> LayerIds and CommitIds. The combined constructor first verified the original derivations,
> then rederived one genesis layer and all ten commits in dependency order, updating
> parent/base/head/root references. Keeping old typed IDs with new root fields would have been
> invalid even if SQL foreign keys passed. Full old-to-new mappings are retained."

The retained map is `EXT:.../layerfs-issue100-40mb-experiments/combined/identity-map.json`
(consumed as an oracle at `.../ordered-optimization/index/full157/reader-check.py:23,29`).
No product-side migration, translation or transient spooling exists
(`protocol-compact-ids.md:5`: "Migrating old Stores would need translation/transient spooling
and is unimplemented").

### 1.5 Measured byte effect

Ten-state diagnostic (eleven original roots), matched offline metadata packing
(`report-D.md:5-16` and `40mb-experiment-results.md:39-46`):

| Metric | A (current canonical) | C (inline + direct dir) | **D (C + scoped serials)** |
| --- | ---: | ---: | ---: |
| Canonical metadata bytes | 10,194,894 | 8,571,111 | **6,277,255** |
| Metadata objects | 46,288 | 5,080 | **4,900** |
| Encoded metadata packs | 7,063,939 | 5,593,831 | **2,639,978** |
| Compact object-index pages | 3,731,456 | 1,814,528 | **1,806,336** |
| Added `scope_allocator` page | — | — | **4,096** |
| **Matched subtotal** | 10,795,395 | 7,408,359 | **4,450,410** |

D's incremental C→D saving is **2,957,949 B** (2,953,853 pack + 8,192 index − 4,096
allocator); versus A it is **6,344,985 B** (`report-D.md:16`). D produced 4,760 new canonical
objects: 4,145 rewritten directory nodes, 604 table nodes, 11 roots; max canonical object
8,144 B (`report-D.md:30-34`).

53-state replay (`docs/roadmap/0.1/0.1.5/issue100/experiments40/stride3-structural/metadata/report.md:5-13`):
10,542 compact canonical objects / 28,458,524 B, 25,390 scoped identities, 6,875 directory
objects, 3,137 LFS6INT, 54 LFS6FSR; metadata packs 20,094,239 (original) → 12,314,839
(hash-sorted FULL) → **6,877,152** with the same selected-DELTA policy as full157.

Combined ten-state offline copy (`40mb-experiment-results.md:11-31`): total
**41,648,128 B** vs 48,783,360 B matched VACUUMed control (−7,135,232 B), of which metadata D
packs = 2,639,978 B, object index = 1,806,336 B, allocator = 4,096 B. The full157 downstream
lineage starting from D-based structural work measures 79,790,080 B
(`structural-investigations.md:7`), then 65,957,888 B with pooled values + whole-file content
(`ordered-optimization-results.md:9-19`).

**Correction on the task's cited evidence.** `compact-candidate-1-results.md` does **not**
measure this namespace. It measures a *product* change to the admission candidate index
(`crates/layerfs-layerstack-store/src/objects/small_candidates.rs`, `SLOTS=1024`,
`REFERENCES=8192`, `INDEX_BYTES=128*1024` at lines 4-6) — that is the "index compaction"
whose measured effect is 123,422 content-pack bytes and only **4,096 allocated bytes**
(`compact-candidate-1-results.md:17`). Do not attribute those bytes to the compact namespace.
The 4,096 B figure coincidentally equals D's allocator page; they are different objects.

---

## 2. Current product namespace representation, end to end

### 2.0 Object envelope and authentication (unchanged by any of this)

- `LFSO` framing: `crates/layerfs-content/src/object/codec.rs:11-12` (`MAGIC`, `HEADER_LEN=9`),
  `encode_bytes_object` `:22-37`, `encode_object` `:14-20`.
- Identity: `crates/layerfs-content/src/object/digest.rs:6` (`OBJECT_DOMAIN =
  b"layerfs/object/v2\0"`) and `:90-92` (`hash_object_bytes`) → full 32-byte BLAKE3, no
  truncation. `ObjectId` at `object/id.rs:8-35`. Authentication entry points
  `authenticate_identity` / `identify_canonical` re-exported at
  `crates/layerfs-content/src/lib.rs:16-24`.
- **Namespace objects are `Bytes` objects** (`ObjectKind::Bytes = 0x01`,
  `object/canonical.rs:7-10`) distinguished only by their 8-byte role magic.

### 2.1 Namespace root — `LFS4FSR`

- Value 108 B: `magic8 | version2 | role=6,flags=0 | profile_id32 | root_directory_inode32 |
  inode_table_root32` → canonical 121 B.
- Struct: `crates/layerfs-content/src/tree/root.rs:4-9` (`NamespaceRootV1`).
- Encode: `crates/layerfs-content/src/tree/directory/codec.rs:187-199`.
- Decode: `directory/codec.rs:201-212` (`exact_value(canonical, b"LFS4FSR\0", 108, 6)`).
- `profile_id()`: `directory/codec.rs:214-233` — a fixed BLAKE3 over
  `layerfs/namespace-profile/bplus/v1\0` plus the profile's numeric parameters. Every
  encode/decode of a namespace root or directory state rejects a mismatched profile
  (`codec.rs:129`, `:151`, `:188`, `:208`).
- Only reader dispatch point in the product:
  `crates/layerfs-content/src/filesystem/resolve.rs:26-27` (`namespace()` →
  `with_authenticated_canonical(root, decode_namespace_root)`).

### 2.2 Inode table nodes — `LFS4INT`

- 31-B header, then **64 B/row**: `inode_id32 | inode_record_object_id32`.
- Enum: `crates/layerfs-content/src/tree/inode/codec.rs:9-16` (`InodeTableNodeV1::Leaf/Branch`,
  role 7/8).
- Encode: `inode/codec.rs:18-63` (leaf role 7 level 0, branch role 8 level ≥1, `count ≤ 127`
  for both roles, `subtree_bytes = subtree_count * 64`).
- Decode: `inode/codec.rs:65-105`.
- Read/lookup: `tree/inode/table.rs:27-34` (`inode_table_lookup` → `Option<ObjectId>` = the
  **record object id**), `:36-131` (`inode_table_lookup_many`), `:211-247` (`lookup_from`),
  `:248-254` (`leaf_lookup`); cursor/streaming `tree/inode/cursor.rs:15, 96-109`
  (`StreamingInodeDiff`, `StreamingInodeCursor`).
- Write: `table.rs:11-25` (`inode_table_from_root`), `:277-287` (`inode_table_upsert`),
  `:288-347` (`inode_table_remove`), `:447-466` (`generated_inode_table_*`),
  `:467-536` (`inode_table_apply_insertions*`), `:478-496`
  (`build_initial_inode_table_from_pairs`), balancing helpers `:537-653`.
- Diff/reconcile: `table.rs:667-704` (`reconcile_inode_tables`), `:787-845`
  (`apply_inode_table_change`), `:863-944` (`reconcile_inode_records`), `:995-1008`
  (`diff_inode_table_entries`).

### 2.3 Inode records — `LFS4INO`

- Value 85 B: `magic8 | version2 | role=4,flags=0,kind1 | namespace_ref_count8 |
  content_root32 | metadata_root32` → canonical 98 B.
- Struct: `tree/inode/record.rs:45-51` (`InodeRecordV1`), validation `:53-67`.
- Encode: `inode/codec.rs:107-116`. Decode: `inode/codec.rs:118-126`.
- Read: resolved through the table then authenticated, e.g.
  `filesystem/resolve.rs:86-90`, `filesystem/diff.rs` node summaries,
  `tree/directory/validate.rs:115-150`.

### 2.4 Directory state and nodes — `LFS4DIR` / `LFS4NSP`

- **`LFS4DIR` directory state** (value 85 B): `magic8 | version2 | role=3,flags=0 |
  entry_count8 | tree_level1 | profile_id32 | mapping_root32`. Encode
  `directory/codec.rs:128-141`; decode `:143-158`. This is the wrapper the compact format
  removes.
- **`LFS4NSP` directory node**: 31-B header; leaf role 1 → `u16 len | name | inode_id32`
  (34+len), `subtree_encoded_bytes = Σ(34+len)`; branch role 2 → `u16 len | name | child32`
  (34+len). Enum `directory/codec.rs:11-23`; encode `:25-80`; decode `:82-126`;
  `verify_subtree_bytes` `:394-404`.
- Read: `directory/read.rs:16-27` (`empty_directory`), `:29-39` (`directory_lookup`),
  `:40-159` (`directory_lookup_many` + cache), `:237-250` (`directory_entries`),
  `:251-297` (`directory_page_after`), `:298-368` (`visit_directory_entries`),
  `:369-451` (`walk_directory_node`), streaming cursors `:452-783`.
- Write/edit: `directory/edit.rs:18-44` (`directory_insert`), `:45-74` (`directory_remove`),
  `:75-140` (`directory_rename`), `:256-318` (`remove_node`), `:319-453`
  (`rebalance_children`), `:527-591` (`insert_node`), `:592-612` (`split_leaf_if_needed`),
  `:613-685` (`split_branch_if_needed`), `:686-...` (`emit_branch_from_summaries`).
- Batched sorted updates: `tree/batch.rs:3-4` (imports both codec pairs), used at
  `batch.rs:750` (`encode_directory_node`) and `:804` (`encode_inode_table_node`), decode at
  `:699` / `:775`.
- Diff/reconcile: `directory/diff.rs:21-54` (`diff_directory_entries`), `:55-204`
  (`diff_directory_nodes`/`children`), `:224-274` (`reconcile_directory_roots`).

### 2.5 Metadata roots — `LFS4MET` (+ `LFS4ACL`)

- Node value: 31-B header; leaf role 9 → `u16 domain | u16 key | required_flag1 |
  value_file_root32` (37+len), `subtree_encoded_bytes = Σ(37+domain+key)`; branch role 10 →
  `u16 domain | u16 key | child32`.
- Encode: `tree/metadata/codec.rs:22-93`; decode `:95-...`.
- Build/lookup/rewrite: `tree/metadata/tree.rs:26-36` (`build_metadata_tree`), `:37-198`
  (`MetadataTreeBuilder`), `:199-210` (`metadata_tree_entries`), `:211-275`
  (`metadata_lookup*`), `:276-...` (`replace_metadata_entry`), `:511-...`
  (`visit_metadata_entries`).
- Apple ACL: `tree/metadata/apple_acl.rs:24, 43` (`LFS4ACL`).
- **Metadata roots are not part of the compact-namespace change.** In D, all metadata/value
  support objects are retained byte-for-byte ("retains 140 original metadata/value support
  objects", `report-D.md:30`; `LFS4MET` count 4 in the ten-state census). What changes is that
  the inode record's `metadata_root` field is **inlined** into the table leaf instead of living
  in a separate LFS4INO object — the referenced metadata tree itself is unchanged.

### 2.6 (context) file-content roles, so the namespace boundary is unambiguous

`LFS4MAP` extent state/node (`file/extent_codec.rs:9`), `LFS4CHK` native chunk
(`file/extent_codec.rs:10`), `LFS5SML` SmallContent (`file/content.rs:9`), `LFS4LNK` symlink
(`directory/codec.rs:160-185`). These are content, not namespace.

### 2.7 How the store allocates inode identities today

**There is no persistent inode counter, allocator table, or serial highwater in the product.**
`sql/schema/v9.sql` (read in full) contains only `object_packs`, `objects`, `commits`,
`branches`, `layer_stacks`, `layers`, `workspace_stages` + indexes — no allocator. Grep for
`scope_allocator|inode_serial|next_inode|inode_counter` over `crates/**/*.sql` and
`crates/**/*.rs` returns nothing in product code (`serial_initialize` at
`layerstack.rs:333-397` is the *serial* (non-parallel) directory-import fallback and is
unrelated).

Identity is instead a **pure deterministic function of a salt plus the logical path**:

- `tree/inode/record.rs:7-14`: `InodeId::allocate(store_id: [u8;32], serial: u64)` =
  `BLAKE3("layerfs/inode-id/v1\0" ++ store_id ++ serial_be)` → 32-byte `InodeId`.
  Note the **already-present but always-zero `serial` parameter** — a natural seam.
- `filesystem/change.rs:424-434`: `allocated_inode(seed, path)` =
  `InodeId::allocate(ObjectId::for_bytes(seed ++ path).to_bytes(), 0)`.
- Initial import: `layerstack.rs:28` `initialization_seed(&layer_stack_id, …)` →
  `:578-597` (`blake3(layer_stack_id)` unless the diagnostic override
  `LAYERFS_BENCH_INITIALIZATION_SEED_HEX` is set); root inode `InodeId::allocate(seed, 0)`
  (`filesystem/root.rs:11`, `filesystem/apply.rs:275`, `layerstack.rs:1494, 1918, 2192`);
  every other inode `filesystem::allocated_inode(self.seed, logical)` (`layerstack.rs:2113`
  regular files, `:2194` non-root directories, `:2241` symlinks).
- Live workspace mutation: `crates/layerfs-workspace/src/changes.rs:908-934`
  (`frontier_inode`) allocates new inodes as
  `allocated_inode(self.live.base_root.to_bytes(), path)` — i.e. **scoped by the current base
  snapshot root id, not by a counter**, with an explicit comment (`:928-929`) that identity is
  bound to the base snapshot so a replaced alias cannot reuse the still-live inode.
- Hardlinks are handled by native (dev,ino) memoization + `namespace_ref_count` bump, not by
  a separate identity store (`layerstack.rs:2094-2111`, `:2216-2235`).

So a durable 8-byte serial allocator is a genuinely **new** persisted structure, not a
resizing of an existing one.

### 2.8 How objects are admitted into packs

- Pack versions: `objects/pack.rs:98-135` (`Version::{Legacy, Native, Small}` from the
  big-endian u32 at offset 8: 1 / 2 / 3). Group limit 64 KiB (`pack.rs:6` `GROUP_LIMIT`).
  Metadata/namespace objects ride in **version 1** packs (legacy FULL-id-sorted groups,
  zstd-or-raw per 16 KiB group); content is version 2 (native CDC) / 3 (SmallContent).
- **Role classification decides the pack stream and the group size**:
  `objects/admission.rs:1365-1383` `is_content()` — an 8-magic allowlist
  (`LFS4FSR`, `LFS4INT`, `LFS4INO`, `LFS4DIR`, `LFS4NSP`, `LFS4MET`, `LFS4MAP`, `LFS4LNK`)
  returning `false`, everything else `true`; used at `admission.rs:662-671` to pick a 16 KiB
  (metadata) vs 32 KiB (content) group target. **LFS6 magic names are absent** — compact
  namespace objects would be misclassified as content today.
- Inode-leaf-aware delta reuse (the S1 path the offline D/structural numbers depend on):
  `objects.rs:235-244` `is_inode_table_leaf()` requires `value.starts_with(b"LFS4INT\0")` and
  then a real leaf decode; used at `objects.rs:3234` (`put_tree_origin` sets
  `prior_ids[0] = origin`) and `admission.rs:1243, 1308` (`DeltaSearch::candidate`,
  `objects/admission.rs:1231-1345`). `LFS6INT` leaves would silently take the non-leaf path.
- Admission pipeline entry: `objects/admission.rs:648-...` (`prepare_ordinary`),
  `:1230-1345` (`DeltaSearch`), objects index `sql/objects/insert.sql`,
  `crates/layerfs-layerstack-store/src/objects.rs:2732-2770` (`put_authenticated`),
  candidate cache `objects/small_candidates.rs:4-6, 66-110`.
- Dependency closure for a candidate batch: `objects.rs:2620-2689` (explicit stack + `active`
  cycle check) calling `referenced_objects` at `:2659`; also `:2742-2746`.

### 2.9 How branches / fork interact

- `crates/layerfs-layerstack-store/src/branch.rs:9-78` (`fork_branch`): copies
  `layer_id`/`commit_id`/`base_layer_id`/`head_commit_id` references into a new `branches`
  row. **No object is rewritten** — a fork points at an existing layer's already-published
  namespace root.
- `layerstack.rs:208-277` (`add_layer`) turns a branch head into a LayerRecord whose
  `root_id` is the existing (or newly committed) namespace root; `LayerId::derive` includes
  `root_id` (`ids.rs:105-118`).
- Commit publication: `workspace.rs:278-421` (`commit_candidate`) — `CommitId::derive`
  at `:300`, object admission `:306-324`, then `INSERT_COMMIT` + `ADVANCE_BRANCH` inside one
  publication transaction (`workspace.rs:335-393`); `commit_workspace_candidate` `:423-...`.
  Initial layer/stack publication `layerstack.rs:103-...` (`INSERT_LAYER`, `INSERT`, with
  `fail_transaction_statement` fault boundaries).
- Consequence: because a fork never rewrites the namespace, **one Store can hold branches
  whose roots use different namespace magics**, and every reader must dispatch per root.

---

## 3. Every place that must change to emit and consume the compact representation

Grouped by owner. "Emit" = writer/encoder side; "consume" = reader/closer side.

### 3.1 Content crate — codecs and logical tree

| Concern | File:line | Why |
| --- | --- | --- |
| Compact codecs (already drafted, needs profile + validation) | `crates/layerfs-content/src/tree/compact.rs:1-183` | No namespace-profile constant/validation; `decode_root` accepts any `profile_id`; no `scope`/serial cross-checks. |
| Inode table encode/decode | `tree/inode/codec.rs:18-63, 65-105` | Must dispatch to LFS6INT; leaf rows are 81 B with an inline record, not 64 B with a record id. |
| Inode record object | `tree/inode/codec.rs:107-126`, `tree/inode/record.rs:45-67` | `LFS4INO` objects disappear for compact roots; `InodeRecordV1` becomes an in-leaf value. |
| Inode table lookup API | `tree/inode/table.rs:27-34, 36-131, 211-254` | Today returns `Option<ObjectId>` (a separate object). Inline form must return the record (or serial→record) directly; changes every caller. |
| Inode table write/edit | `tree/inode/table.rs:277-347, 447-536, 537-653` | Balancing/capacity constants (`count ≤ 127`, 64 B/row, `subtree_bytes = count*64`) are wrong for 100×81 leaves and 127×40 branches. |
| Inode cursors / streaming inode diff | `tree/inode/cursor.rs:15-194` | Emits `(InodeId, ObjectId)` pairs. |
| Inode diff/reconcile | `tree/inode/table.rs:667-845, 863-1008` | Compares record objects by id; inline records must be compared by value, and serial↔id translation is needed for mixed-format roots. |
| Directory node codec | `tree/directory/codec.rs:25-126` | Leaf value width 34+len → 10+len with an 8-byte serial; `ensure_count_fits(..., 35)` at `:87` and `verify_subtree_bytes` at `:394-404` both assume 32-byte refs. |
| Directory state object | `tree/directory/codec.rs:128-158` | `LFS4DIR` wrapper is removed for compact directories; record `content_root` becomes the node root. |
| Directory read | `tree/directory/read.rs:16-27, 29-159, 237-368, 369-451, 452-783` | `directory_lookup*` returns `InodeId` from a leaf; must return/map serial; `empty_directory` builds LFS4DIR. |
| Directory edit | `tree/directory/edit.rs:18-140, 256-318, 319-453, 497-685` | Insert/remove/rebalance/split rewrite leaves whose refs are serials; subtree-byte accounting changes (10 vs 34). |
| Directory validate | `tree/directory/validate.rs:115-200` | Inode record validation is called on a fetched object id. |
| Directory diff/reconcile | `tree/directory/diff.rs:21-54, 55-204, 224-274` | Leaf entries carry serials; `reconcile_directory_roots` merges 32-byte refs. |
| Batched sorted updates | `tree/batch.rs:3-4, 699, 750, 775, 804` | Calls the codecs directly; must learn the compact shape and its capacities. |
| Metadata trees | `tree/metadata/codec.rs:22-...`, `tree/metadata/tree.rs:26-...` | **No wire change required**, but `metadata_root` is now an inlined field rather than an object reference. |
| Namespace root | `tree/root.rs:4-9`, `tree/directory/codec.rs:187-212`, `tree/mod.rs:1` | Add `NamespaceRoot` variant + profile id for the compact format (`directory/codec.rs:214-233` mints only the v1 profile). |

### 3.2 Content crate — filesystem semantics over the namespace

| Concern | File:line | Why |
| --- | --- | --- |
| Root/namespace dispatch | `filesystem/resolve.rs:26-27` | Single dispatch point: LFS4FSR vs LFS6FSR. Good news — it is one seam. |
| Path resolve | `filesystem/resolve.rs:29-56, 57-...` | Uses `record.content_root` as `DirectoryStateRoot`; for compact roots it is an LFS6NSP node root. |
| Read path (stat/list/read) | `filesystem/read.rs:36-45, 46-78, 79-99, 100-114, 115-...` | Goes through resolve + directory reads; must handle serial refs and the absent LFS4DIR. |
| Initial namespace build | `filesystem/apply.rs:270-286` (`build_initial_namespace`), `:369-497` (`build_initial_directory*`, `apply_initial_inode_upserts`) | Encodes inode records as separate objects (`apply.rs:282`); must inline them and allocate serials. |
| Initial root | `filesystem/root.rs:10-26` (`empty_root`) | Root inode = `InodeId::allocate(seed, 0)`; must become a serial + scope. |
| New inode allocation | `filesystem/change.rs:424-434` (`allocated_inode`) | Path-derived identity is the thing the compact format replaces. |
| Directory/symlink/hardlink create | `filesystem/apply.rs:498-519, 520-542, 543-...` | All take a pre-computed `InodeId`. |
| File replace / metadata | `filesystem/apply.rs:703-...` (`replace_file`), `filesystem/change.rs` (`write_file`, `metadata`, `build_portable_metadata`) | Inode values inline; `namespace_ref_count` handling moves into the leaf. |
| Rename / remove | `filesystem/apply.rs:155-269`, `change.rs` (`apply_changes`) | Identity stability across rename is a *new* invariant the current path-derived scheme does not have. |
| Diff | `filesystem/diff.rs:47-...` (`diff_roots`), `:359-...` (`walk_directory`) | `NodeSummary`/`DiffEntry` carry `InodeId`; cross-format diffs need serial↔id translation. |
| Reconcile | `filesystem/reconcile.rs:30-46, 47-398, 399-419` | `reconcile_inode_change`, `reconcile_roots`, `reconcile_with`. |
| Batch structural counters | `filesystem/apply.rs` (`StructuralBatchCounters`), `filesystem/resolve.rs:10-21` (`LogicalCounters`) | Counters/measurements assume the old object shapes. |

### 3.3 Store crate — admission, closure, publication, compatibility

| Concern | File:line | Why |
| --- | --- | --- |
| Reference enumeration | `crates/layerfs-content/src/object/references.rs:10-84` (`referenced_objects`) | Draft already adds LFS6FSR/LFS6INT/LFS6NSP arms at `:26-35`. This is the *only* graph-edge authority; every closure walk depends on it. |
| Magic safety test | `references.rs:109-158` | Draft adds the three magics at `:112-114`; keep the "safe as user file prefix" property. |
| Role classification (pack stream + group size) | `objects/admission.rs:1365-1383` (`is_content`), used `:662-671` | Must treat LFS6FSR/INT/NSP as metadata (else they land in content packs at a 32 KiB group target). |
| Inode-leaf delta reuse | `objects.rs:235-244` (`is_inode_table_leaf`), `:3228-3243` (`put_tree_origin`), `admission.rs:1243, 1308-1317` | Must recognize `LFS6INT` leaves or the measured structural DELTA savings do not materialize. |
| Dependency closure build | `objects.rs:2620-2689` (uses `:2659`), `:2732-2769` (`put_authenticated`, `:2742-2746`) | Walks `referenced_objects`; needs the LFS6 arms (already in the draft) **and** correct `active`-set/cycle behaviour for serial-keyed leaves (a leaf has up to 200 edges). |
| Closure walk #2 (canonical storage / integrity) | `query.rs:285-395` (`canonical_storage`) → `query.rs:456-487` (`traverse_root`, `:473` `referenced_objects`) | Used for store-level storage accounting; must accept both formats. |
| SmallContent chain closure | `objects/read.rs:265-370` (`small_anchor`, `small_predecessor`, `small_chain`), `query.rs:474-476` (`small_physical_base`) | Namespace objects are not in this chain, but the physical-base enumeration runs alongside it. |
| Compact-candidate index (unrelated to the namespace) | `objects/small_candidates.rs:4-6, 39-110` | The `compact-candidate-1` retention; do not confuse with LFS6. |
| Commit/layer id derivation | `ids.rs:92-118` (`CommitId::derive`, `LayerId::derive`) | `root_id` feeds both; a new root encoding re-derives all typed ids. |
| Commit publication | `workspace.rs:278-421, 423-...`; `layerstack.rs:103-200` (`initialize_layerstack` publication), `:208-277` (`add_layer`) | Must publish the allocator highwater atomically with the commit/layer row, or serials can be reused after a failed Commit. |
| Diff entry point | `query.rs:207-260` (`visit_diff` → `filesystem::diff_roots`) | Cross-format diff needs translation. |
| Snapshot/reconciliation reads | `workspace.rs:82-166` (`acquire_workspace_lease`, `pin_branch`, `snapshot_reader`, `prepare_reconciliation`) | Historical reads bind a root object id; must dispatch per root. |
| Fork | `branch.rs:9-78` | No rewrite needed, but a forked compact branch must not later be written by an LFS4 writer. |

### 3.4 Workspace crate (live mutation path)

- `crates/layerfs-workspace/src/changes.rs:908-934` (`frontier_inode`) — the live allocator
  seam; must call the durable serial allocator scoped to the Store/scope, not
  `allocated_inode(base_root, path)`.
- `changes.rs:177-330` (inode mutation records, 192-byte spilled frontier rows at `:1898-1929`),
  `:2287-2330` (`InodeMutation::Upsert` construction), `cow_tree.rs:409-488`
  (`ids`, `inode` fields), `changes.rs:1155-1200` (`records(...)` directory name→inode pairs).
- `crates/layerfs-workspace-core/src/namespace.rs:68` — `InodeId([9; 32])` test fixture.

### 3.5 SQLite schema / SQL

- New table for the durable allocator (prototype form: `scope_allocator(scope BLOB PK,
  highwater INTEGER)`) and therefore a **new `sql/schema/vN.sql`**, plus
  `statements::schema::{V9→new, MIGRATE_TO_*, SCHEMA_OBJECTS}` registrations at
  `crates/layerfs-layerstack-store/src/statements.rs:59-71`.
- Existing tables need no column change: `commits.root_id`, `layers.root_id`,
  `workspace_stages.root_id`, `branches.base_layer_id/head_commit_id` are all opaque
  references (`sql/schema/v9.sql:18-128`). `objects.canonical_length <= 16777216` already
  admits 8,144-byte compact leaves (`sql/schema/v9.sql:11-12`).
- Insert/lookup SQL for objects (`sql/objects/insert.sql`, `page.sql`, `get_many_128.sql`) is
  representation-agnostic.

### 3.6 Verification / reopen / integrity

- `schema.rs:355-391` (`preflight_connect`: accepts `6|7|8|9`), `:425-445` (`verify_schema`:
  exact `schema_objects` comparison, application_id, user_version, page size 4096|65536,
  foreign-key check), `:479-489` (`expected_schema_objects`), `:794-819` (`upgrade_format`,
  currently "requires schema 7, 8 or 9").
- Feature gating pattern to extend: `schema.rs:166-172` (`native_format = >=7`,
  `small_content_format = >=8`, `small_chain_format = >=9`).
- Reopen/limits tests: `crates/layerfs-layerstack-store/src/schema/compatibility.rs:56-187`
  (legacy reads without promotion, mixed pack versions, unsupported-version rejection without
  mutation, legacy-open contract).
- Store-level closure/accounting: `query.rs:285-395` (`canonical_storage`) — the closest thing
  to a whole-Store integrity walk in the product.
- Content-crate tests that will need compact analogues: `crates/layerfs-content/tests/namespace_codec.rs`,
  `namespace_model.rs`, `logical.rs`, `canonical_v2_fixture_oracle.rs`; store tests
  `tests/v4.rs`, `tests/v5.rs`, plus `objects/ingestion_tests.rs`, `objects/read/native_tests.rs`,
  `objects/admission/*_tests.rs`.

---

## 4. The compatibility boundary

### 4.1 What is supported today

- `crates/layerfs-layerstack-store/src/schema.rs:11-12`: `SCHEMA_VERSION = 9`,
  `LEGACY_SCHEMA_VERSION = 6`.
- `schema.rs:374-376`: `preflight_connect` accepts exactly `{6, 7, 8, 9}` and rejects anything
  else with `StoreError::WrongStoreSchema` **before** opening read-write.
- `schema.rs:381-389`: a schema-6 Store that already contains non-legacy pack headers is
  rejected (research binaries are evidence, not supported Stores).
- `schema.rs:425-445` + `:479-489`: for the declared version, the Store's actual schema objects
  (tables/indexes, via `sql/schema/schema_objects.sql`) must **equal exactly** the object set
  produced by that version's SQL (`statements::schema::V6|V7|V8|V9`). Page size must be
  4096 or 65536; application id must be `0x4c46_534c`; foreign keys must check clean.
- `schema.rs:794-819`: offline `upgrade_format` accepts only 7/8/9 and promotes to 9 with
  `MIGRATE_TO_V9`; it never touches 6.
- Format *behaviour* is gated on the schema version: `schema.rs:166-172` (>=7 native packs,
  >=8 SmallContent, >=9 small chain). Pack versions 1/2/3 are additionally self-describing
  per pack (`objects/pack.rs:113-135`), so a schema-9 Store legitimately holds v1 metadata
  packs, v2 native packs and v3 SmallContent packs simultaneously
  (`schema/compatibility.rs:107-110`).
- Object formats are **not** versioned anywhere: a namespace object is identified by its 8-byte
  magic inside the canonical bytes, and the reader dispatches on that magic
  (`object/references.rs:25-83`; `directory/codec.rs:82-126`; `inode/codec.rs:65-105`). The
  namespace *profile* is a separate 32-byte check (`directory/codec.rs:214-233`).

### 4.2 Can a compact namespace live inside schema 9?

**No — not while the durable allocator exists — and the reason is the schema-object equality
check, not the object formats.**

- *Object formats* can coexist inside schema 9: LFS4* and LFS6* are both `Bytes` objects
  distinguished by magic, the reader has a single root-dispatch seam
  (`filesystem/resolve.rs:26-27`), and pack v1 already carries both because it only frames and
  compresses groups (`objects/pack.rs:113-135`). Nothing in `verify_schema` inspects object
  bytes.
- *But* the prototype's allocator is a new SQLite table (`compact_ids.py:139`). Adding that
  table to `sql/schema/v9.sql` changes `expected_schema_objects(9)`, so **every existing
  schema-9 Store in the field would fail `verify_schema` and be rejected**
  (`schema.rs:437-439`). Leaving the table out of the version but creating it at write time
  fails the same comparison in the other direction. So the allocator state cannot be added to
  schema 9 without breaking one side of the boundary.
- *Also* there is no persisted "this Store contains compact namespaces" signal today. The
  gating pattern used by v7/v8/v9 is `format_version >= N` (`schema.rs:166-172`); the product
  deliberately does not scan object magics at connect time (see the schema-6 header probe
  being described as a special isolation case, `schema.rs:378-389`).
- **Therefore a compact namespace with a durable allocator must be a new schema version**
  (call it 10), which must:
  1. add `sql/schema/v10.sql` (+ `statements::schema::V10`) with the allocator table and any
     new indexes, and register it in `ALL` (`statements.rs:1-71`),
  2. extend `preflight_connect`'s accepted set (`schema.rs:374`) and `expected_schema_objects`
     (`schema.rs:479-489`),
  3. add a new `MIGRATE_TO_V10`-style path only if migration is intended — note that
     migration cannot be a pure SQL transform here, because the *objects* must be rewritten and
     typed ids re-derived (`40mb-experiment-results.md:79`); the prototype explicitly leaves
     this unimplemented (`protocol-compact-ids.md:5`),
  4. extend the `format_version >= N` feature gate so the writer emits compact objects only for
     v10 Stores, and keep v9 Stores writing LFS4* unchanged ("supported old opens do not
     promote" is the existing policy, `schema/compatibility.rs:57-95`).
- The one **escape hatch** that would allow schema 9: store no new allocator table and derive
  serials instead (e.g. from a per-root persisted highwater inside the namespace root, or from
  a deterministic identity→serial mapping). The evidence does **not** determine whether such a
  scheme can meet the prototype's global-monotonic, never-recycled, cross-branch requirement
  (`protocol-compact-ids.md:9`) — no experiment tried it, and the prototype's measured 4,096-B
  allocator cost is the only data point.

---

## 5. Blockers and unknowns for integrating the compact namespace first

Ordered roughly by how much they block a first end-to-end slice.

1. **No durable allocator exists, and its concurrency/rollback semantics are unimplemented.**
   Today identity is `BLAKE3(salt ++ serial)` with the serial parameter always zero
   (`tree/inode/record.rs:7-14`; `filesystem/change.rs:424-434`;
   `crates/layerfs-workspace/src/changes.rs:908-934`). The prototype models only a monotonic
   counter (`compact_ids.py:66-70`) and states its own limit at `:83`. Required: transactional
   reservation in the same publication transaction as `INSERT_COMMIT`/`ADVANCE_BRANCH`
   (`workspace.rs:335-393`) so a failed Commit cannot recycle serials; import-time reservation
   during `initialize_layerstack` (`layerstack.rs:21-200`); and a rule for concurrent writers
   (the Store already serializes writers via `enter_operation`/exclusive locking, which helps).
2. **What is the scope, in product terms?** The prototype's scope is
   `blake3(b'layerfs/diagnostic/scoped-inode-origin/v1\0' ++ genesis_root)`
   (`compact_ids.py:92`). The product must choose between Store, LayerStack, and Layer as the
   scope domain, decide which root is the genesis anchor, and decide what happens when a
   LayerStack imports a foreign inode scope — **explicitly unimplemented**
   (`protocol-compact-ids.md:5`; `report-D.md:53`).
3. **Typed identity churn.** A new root encoding changes `root_id` →
   `LayerId::derive`/`CommitId::derive` (`ids.rs:92-118`), so every published Commit/branch
   head/layer root would change if a Store were rewritten. The prototype simply rederived all
   of them (`40mb-experiment-results.md:79`). The product must instead decide: per-root
   dispatch with mixed formats in one Store (keeps history readable, but forces every reader
   and the diff/reconcile pair to handle both), or an explicit offline migration (not
   implemented, and not a pure SQL transform).
4. **The inode-table API changes shape, and it is consumed widely.**
   `inode_table_lookup` returns `Option<ObjectId>` (a record object) today
   (`tree/inode/table.rs:27-34`). Inlining the record means every caller changes:
   `filesystem/resolve.rs:86-90`, `filesystem/diff.rs` node summaries,
   `tree/inode/table.rs:667-1008` (reconcile/diff), `tree/inode/cursor.rs:15-194`,
   `tree/directory/validate.rs:115-150`, `tree/batch.rs:775-804`. This is the largest
   mechanical change and it touches hardlink/rename semantics that the task requires to keep
   passing through their real owners (`.../experiments40/metadata/report.md:45`).
5. **Directory leaves stop carrying object ids.** `directory_lookup*` return `InodeId`
   (`tree/directory/read.rs:29-159`); the compact leaf carries an 8-byte serial, and the
   LFS4DIR wrapper is gone so `record.content_root` becomes a node root
   (`filesystem/resolve.rs:45-55`). Callers: `read.rs:237-451`, `edit.rs:18-685`,
   `diff.rs:21-274`, `batch.rs:699-750`, `validate.rs`. Two separate byte-accounting constants
   change (34 → 10 per entry).
6. **Store-side classification and reuse must learn the new magic.** `is_content`
   (`objects/admission.rs:1365-1383`) and `is_inode_table_leaf` (`objects.rs:235-244`,
   `admission.rs:1243, 1308`) are magic/role-driven allowlists. Missing entries cause compact
   metadata to be packed as content (wrong stream, wrong group target) and silently disable the
   inode-leaf delta-reuse path that the measured structural savings depend on.
7. **The measured offline bytes are not a product forecast.** A's offline control (7,063,939
   metadata pack bytes) already differs from the live Store's measured 6,550,582 B
   (`metadata/report.md:25`), and D was measured under a separate FULL-only 16 KiB offline
   packing policy, not the live mixed-group/selected-delta admission
   (`report.md:3, 25`). The 41,648,128-B combined copy is an explicit diagnostic VACUUM copy,
   not a Store allocation (`40mb-experiment-results.md:7`). Only a fresh public ten-state run
   can establish the integrated number.
8. **No product validation exists for the draft codecs.** `compact.rs:185-228` has two unit
   tests over the fixtures; there is no test that a compact root survives `resolve`/`stat`/
   `list`/`edit`/`Commit`/reopen, no profile-id validation in `decode_root`
   (`compact.rs:42-50` does not check `profile_id`), and the compact module is referenced only
   by `references.rs`. The fixtures (`tests/fixtures/compact/{root,inode-leaf,inode-branch,
   directory-leaf,directory-branch}.bin` + `manifest.json`) are real prototype bytes and are a
   strong oracle for byte-compatibility, but they are not a product integration test.
9. **Hardlinks and refcount live inside the inlined record.** Today hardlinks are
   `(dev, ino)` memoization plus a `namespace_ref_count` bump on a shared record object
   (`layerstack.rs:2094-2111, 2216-2235`). With inlining, the count is a leaf field, so two
   directory entries sharing one serial must be updated consistently in the same edit — the
   prototype asserts this as a representation property only (`compact_ids.py:83`).
10. **Capacity/limit arithmetic.** Compact leaf = 31+100·81 = 8,131 value bytes / 8,144
    canonical, against `NODE_LIMIT = 8192` on the canonical object
    (`tree/directory/codec.rs:9`, checked in `decode_node_value` `:288-291` and `finish_node`
    `:341-348`) — it fits with 48 bytes to spare, but any future entry-width growth or a
    100-entry limit change collides with that hard limit. Branch = 31+127·40 = 5,111.
11. **Compatibility/migration decision is unowned.** See §4: a schema-10 bump, a
    no-promotion old-read policy, and refusal (or explicit translation) for mixed-format
    diff/reconcile are all still decisions, not implementations.

### What the evidence does **not** determine

- Whether the compact namespace can be introduced **without** a new schema version (no
  experiment stored allocator state inside schema 9; see §4.2 escape hatch).
- Real product scope semantics: the prototype's scope is a diagnostic descriptor over the
  genesis root; Store/LayerStack/Layer scoping, foreign-scope import and cross-store identity
  are untested.
- Online (mixed-group, selected-delta) byte outcomes. Every measured D number is offline,
  matched-policy, and for a different cardinality (11 roots / 53 states), and A's own control
  does not equal the live Store.
- Read latency and memory for serial-keyed inode tables and serial-valued directory leaves under
  the public APIs; the prototype states its own checks are representation-only.
- Whether a compact branch and a legacy branch can be **diffed or reconciled** in one Store
  (no experiment; the old→new id map is explicitly not a runtime dictionary).
- Whether `sort`-by-serial inode tables preserve the *existing* `LFS4INT`-keyed S1
  predecessor-reuse behaviour once ported online (the offline D/structural pipeline relies on
  it, but no product-path measurement exists).
- Whether `compact-candidate-1`'s product-side 4,096-byte allocation change composes with the
  namespace change; the two were measured in different campaigns and must not be added.

### Key correction to the task's framing

`compact-candidate-1-results.md` and the `small_candidates.rs` retention measure a
**product admission candidate-index** change, not the compact namespace. The compact
namespace's measured bytes come from `report-D.md` / `report.md` (metadata experiment D),
`stride3-structural/metadata/{report.md,d/result.json}`, and the combined copies in
`40mb-experiment-results.md`. Do not attribute the 4,096-byte allocation figure from
`compact-candidate-1-results.md` to the namespace allocator; it is the candidate index's
allocation delta and merely coincides with D's allocator page size.

# Whole-file content history + native chunk slice adapters — format spec and integration research

Research note for issue 103, scope: **the whole-file content history part only** — (1) whole-file FULL/prefix (base-graph) representations, (2) native chunk slice adapters, (3) the offline base-selection policy and its product-owned replacement.

Status: **research only — no source file was modified.** This document is the only file written.

## 0. Scope, sources, method

| Role | Path |
|---|---|
| Product (Rust, schema 9) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb` |
| Offline prototype (Python, read-only evidence) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization` |
| Executed 157-state content prototype | `…/layerfs-issue100-ordered-optimization/content/full157/` |
| Executed 53-state content prototype | `…/layerfs-issue100-ordered-optimization/content/` |
| Matching Git pack inventory | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/git-baseline/verify-pack.txt` (10,615,521 B) |
| Git object database | `/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data/git-snapshots-157-v1/snapshots.git` |

Every claim below is either (a) read out of the executed prototype scripts/readers, (b) read out of the actual `candidate.sqlite` artifacts, or (c) re-measured in this session and labelled as such. Statements that the evidence does **not** determine are called out explicitly in §7.

Re-measurements performed here (read-only): class of every prefix edge by base first-appearance ordinal across the 157 sealed manifests; exact per-class stored record bytes; owner-chain depth histogram; adapter-per-owner histogram; byte reconciliation of source and candidate SQLite; `dbstat` component deltas; a 200-sample-per-class FULL-vs-prefix encode re-measurement. All are labelled "measured here".

---

## 1. Exact on-disk representation

### 1.1 Canonical object envelope (unchanged product envelope)

Every object — SmallContent, whole-file, and native chunk — is stored as a product byte-object:

```
"LFSO" (4) | kind=1 (1) | payload_len u32 BE | value_len u32 BE | value
```

`payload_len = value_len + 4`, total header 13 B. This is the existing product envelope (`crates/layerfs-content/src/object/codec.rs:11` `MAGIC = b"LFSO"`, `:12` `HEADER_LEN = 9`; the extra 4 B is the u32 BE value length). The prototype writes it identically at `content/full157/content_codec.py:29-31`:

```python
body = tag + raw
return b'LFSO\1' + struct.pack('>II', len(body)+4, len(body)) + body
object_id(role, raw) = blake3(b'layerfs/object/v2\0' + canonical(role, raw))
```

Canonical identity is the full 32-byte BLAKE3 of the canonical envelope prefixed by the domain string `layerfs/object/v2\0` (`content_codec.py:33`), i.e. exactly the product identity rule. The prototype's reader re-checks it for every materialized object (`metadata/full157/reader.py:162-163` chain: `blake3(b'layerfs/object/v2\0'+b) == i`).

### 1.2 Role tags (value prefix)

| role | tag | tag len | canonical_len for raw R |
|---|---|---|---|
| small (existing product SmallContent) | `4C 46 53 35 53 4D 4C 00 00 01` = `"LFS5SML\0"` + u16 BE `1` | 10 | R + 23 |
| **large (new whole-file)** | `"LFSWFL1\0"` | 8 | R + 21 |
| native (existing product chunk) | `"LFS4CHK\0"` | 8 | R + 21 |

Sources: `content_codec.py:30`; reader length arithmetic `content/full157/reader.py:40` (`header = 23 if role=='small' else 21`).

Cross-checks against the product:
- `"LFS5SML\0"` is the product SmallContent magic (`crates/layerfs-content/src/file/content.rs:9`) with framing `MAGIC + u16 BE version(1) + raw` (`content.rs:44-53`, `:20-32`) — the prototype's small object *is* the current product SmallContent object, byte for byte.
- `"LFS4CHK\0"` is the product chunk magic (`crates/layerfs-content/src/file/extent_codec.rs:10`) with value `CHUNK_MAGIC + raw` (`extent_codec.rs:19-46`).
- `"LFSWFL1\0"` has **no product analogue**. There is no whole-file object for files ≥ 128 KiB today (see §5, row 1).

### 1.3 Physical container: new pack magic `LFCNT1\0\0`

```
offset 0  : "LFCNT1\0\0"        (8 B)
offset 8  : version u32 LE = 107
offset 12 : record count u32 LE   (1..=256)
offset 16 : offsets[count] u32 LE, pack-relative starts of each record
offset 16+4*count : records, concatenated
```

- Writer: `content/full157/experiment.py:104-107`.
- Pack-size/record-count bound: `experiment.py:113` — flush when `len(pending)==256` or `16+4*(len(pending)+1)+payload+len(row[2]) > 4 MiB`.
- Reader: `content/full157/reader.py:6` (`MAGIC=b'LFCNT1\0\0'`, `MAX_PACK=4*1024*1024`), `:11-13`, `:19-23`.

There is **no group layer**: the prototype's container is one flat record table per pack (`count` ≤ 256). The product's pack container is two-level — packs contain up to 256 *groups*, groups contain up to 8,191 *records* (`crates/layerfs-layerstack-store/src/objects/pack.rs:7-9`). The `objects` table row for a prototype content object always has `record_number = 0` and `group_number` = the flat record index (`experiment.py:105-110`); measured here in the candidate DB: `record_number` is 0 for all 79,142 content rows and `group_number` ranges 0..255, max 256 records per new pack.

### 1.4 Record grammar (kinds 0–5)

Single leading kind byte; format differs by kind.

| kind | meaning | layout | length formula |
|---|---|---|---|
| 0 | small FULL | `00 ‖ zstd_frame` | 1 + frame |
| 1 | small prefix | `01 ‖ base[32] ‖ zstd_frame` | 33 + frame |
| 2 | **large FULL (whole file)** | `02 ‖ zstd_frame` | 1 + frame |
| 3 | **large prefix (whole file)** | `03 ‖ base[32] ‖ zstd_frame` | 33 + frame |
| 4 | **native chunk slice** | `04 ‖ owner[32] ‖ offset u32 LE ‖ length u32 LE` | 41 (fixed) |
| 5 | native FULL fallback | `05 ‖ zstd_frame` | 1 + frame |

Writer: `experiment.py:84` (kinds 0–3), `:91` (kind 4), `:93` (kind 5). Reader: `content/full157/reader.py:23-28`.

Note the framing difference from the product: there are **no explicit raw_length / frame_length fields**. The frame length is implicit from the next directory offset (`reader.py:22`) and the raw length is implicit as `canonical_length − 21|23` (`reader.py:40`, `:51`). The product's own whole-file record *does* carry explicit lengths (`objects/delta.rs:20-48`: `kind ‖ u32 LE raw_length ‖ u32 LE frame_length ‖ [base 32] ‖ frame`). Measured here (candidate DB): kind 0 = 12,126 records, kind 1 = 63,272, kind 2 = 25, kind 3 = 498, kind 4 = 3,221, **kind 5 = 0 records** — no fallback was needed in full157 (matches `result.json` `native_coverage.fallback = 0` and report.md:36).

### 1.5 How a whole-file prefix object references its base

- The base reference is the **full 32-byte canonical object id** of the base, stored inline at bytes 1..33 (`experiment.py:84`; `reader.py:36` `node = r[1:33]`).
- Resolution is via the `objects` locator table (`reader.py:19`, `:33`); the base may itself be a prefix record, so a *chain* exists.
- Chain reconstruction: `reader.py:31-42`. Cold traversal walks to the FULL root checking cycle, role, and closure, then replays **in reverse order**, `previous` raw bytes feeding `codec.decode(frame, n-header, previous, window)` — i.e. the zstd **raw prefix** (`ZSTD_CCtx_refPrefix` / `ZSTD_DCtx_refPrefix`) is the immediately preceding node's raw bytes, not the base's canonical envelope.
- Window: 18 for small, 20 for large (`reader.py:41`). Encoder: window 18 for raw < 128 KiB else 20 (`content_codec.py:20`).
- **There is no chronology constraint on the file graph** (`reader.py:31-37` has no ordinal comparison), unlike the metadata reader (`metadata/full157/reader.py:39` requires `loc[base] < loc[node]`). This is deliberate and is the mechanism by which future-appearing bases are used (see §4).
- Closure limits enforced during traversal: depth ≤ 50 edges, canonical closure ≤ 64 MiB, encoded closure ≤ 64 MiB (`reader.py:34`, `:36`).

### 1.6 How a native chunk slice records (owner, offset, length) and is checked/authenticated

Record: `04 ‖ owner_object_id[32] ‖ offset u32 LE ‖ length u32 LE`, exactly 41 bytes (writer `experiment.py:91`; reader `reader.py:27`, `:51`).

Read path (`reader.py:50-53`):

```python
owner = r[1:33]; offset,length = struct.unpack_from('<II', r, 33)
v,body,_ = self._physical(owner)
check(v==107 and body[0] in (2,3),'slice owner role')          # owner must be a whole-file FULL or prefix
check(length == n-21 and 0<length<=32768 and offset+length <= self.loc[owner][3]-21, 'slice extent')
whole = self.read_canonical(owner)                              # full owner graph authenticated first
check(whole[13:21]==b'LFSWFL1\0','whole canonical role')
raw = whole[21+offset : 21+offset+length]
canonical = codec.canonical('native', raw)                      # re-wrap in the ORIGINAL chunk envelope
self._put(identity, canonical)                                  # blake3 domain-hash == original chunk id
```

Bounds actually enforced:
- `length == canonical_length − 21` — the slice must return exactly the chunk's own raw length.
- `0 < length ≤ 32,768` — the product CDC maximum chunk size (`crates/layerfs-content/src/file/cdc/gear.rs:9`).
- `offset + length ≤ owner_raw_length` — computed against the *owner object's* `canonical_length` from the locator table, so an overflowing extent is rejected before the owner is even decoded.
- owner role must be 2 or 3 (whole-file large).
- the reconstructed bytes are re-wrapped in `LFS4CHK\0` and hashed; `_put` rejects any mismatch (`metadata/full157/reader.py:162-163`). Low-level `_physical` bounds for kinds 4/5 are `22 ≤ canonical_length ≤ 32768+21` and `len(record) == 41` for kind 4 (`reader.py:26-27`).

Five negative checks are executed and passed (`content/full157/check_reader.py:12-28`, recorded in `reader-checks.json`): frame checksum corruption, file self-cycle, slice missing owner, slice wrong owner role, slice extent overflow.

**No adapter-to-adapter reference** is allowed by policy (`content/protocol.md:7`): a slice's owner must be kind 2/3.

### 1.7 Complete bounds/limits

| Limit | Value | Where |
|---|---|---|
| Whole-file (large) raw cap | 2,097,152 B (2 MiB) | `content_codec.py:12`; population assertion `experiment.py:31` |
| Frame cap (raw + 1024) | 2,098,176 B | `content_codec.py:12`, `:21`, `:24` |
| Native FULL (kind 5) raw cap | 32,768 B | `reader.py:26` (`22 ≤ n ≤ 32768+21`) |
| Slice length | ≤ 32,768 B, `== canonical_len − 21` | `reader.py:51` |
| Slice record size | exactly 41 B | `reader.py:27` |
| New pack cap | 4 MiB | `reader.py:6`; writer `experiment.py:106` |
| Records per new pack | ≤ 256 | `experiment.py:113`; `reader.py:21` |
| Legacy metadata pack cap | 262,144 B | `reader.py:12` |
| File graph edges | ≤ 50 (+1 node list bound, `check(len(nodes)<=50)` is checked before appending the 51st) | `reader.py:36` |
| File graph canonical closure | ≤ 67,108,864 B | `reader.py:34` |
| File graph encoded closure | ≤ 67,108,864 B | `reader.py:34` |
| Small canonical length | 24 ≤ n < 131,095 (raw 1..131,071) | `reader.py:24` |
| Large canonical length | 131,072+21 ≤ n ≤ 2 MiB+21 | `reader.py:25` |
| Reader canonical cache | 4 MiB | `metadata/v2-157/reader.py:8` |
| Reader physical-pack cache | 8 MiB | `metadata/v2-157/reader.py:8` |
| Inherited metadata pool cache | 4 MiB decoded bodies (+ ~4 MiB copied records) | `metadata/v2-157/reader.py:19`; `content/execution-notes.md:3` |
| Git-side object cache in the encoder | 64 MiB LRU | `experiment.py:43` |

Observed maxima in full157 (`content/full157/result.json`): max selected depth **50** (the bound is exactly touched), max canonical closure **29,808,454 B**, max encoded closure **483,545 B**, max frame **483,544 B**, max pack **735,861 B**.

### 1.8 Codec parameters (pinned)

`content_codec.py:8-21`: libzstd `ZSTD_versionNumber() == 10507` (asserted), compression level 3, `windowLog` 18/20, `contentSizeFlag=1`, `checksumFlag=1`, `dictIDFlag=0`, `nbWorkers=0`, raw prefix via `ZSTD_CCtx_refPrefix`, frame built by `ZSTD_compress2`.
Decoder `content_codec.py:23-27` re-validates the frame grammar: magic `28 B5 2F FD`, `frame[4] & 0x1b == 0`, `ZSTD_getFrameHeader` → `content == expected_size`, `window <= 1<<windowLog`, `type == 0` (single segment), `dictid == 0`, `checksum == 1`, and `findFrameCompressedSize == len(frame)` (no trailing frames). Re-measured here on a real kind-3 record: `content=150167`, `blocksize=131072`, `type=0`, `dictid=0`, `checksum=1`, `findFrameCompressedSize==len` → the executed artifacts satisfy every one of those checks.

---

## 2. Which Python files/functions write and read this, and the protocol/report files

### Writers

| Function / step | Location |
|---|---|
| Canonical envelope + id | `content/full157/content_codec.py:29-33` |
| Prefix-codec encode/decode | `content/full157/content_codec.py:18-27` |
| Population + 2 MiB assertion | `content/full157/experiment.py:24-31` |
| Git base graph parse | `content/full157/experiment.py:32-36` |
| `git cat-file --batch` raw fetch + SHA1/SHA256 checks + 64 MiB LRU | `content/full157/experiment.py:37-44` |
| CDC boundary scan + native coverage mapping (owner = first Git OID order, then lowest offset) | `content/full157/experiment.py:46-55` |
| Topological order over the base DAG | `content/full157/experiment.py:65-72` |
| Selection loop (FULL vs prefix, gates, record bytes) | `content/full157/experiment.py:73-86` |
| Native slice/fallback record emission | `content/full157/experiment.py:87-94` |
| Fresh SQLite copy, pack flush, locator update, pack deletion, VACUUM, integrity/FK/schema/typed-SQL equality | `content/full157/experiment.py:97-122` |
| Candidate re-read + authentication of all 79,142 content objects | `content/full157/experiment.py:123-132` |
| 53-state variant of all of the above (same code, different fixture/repo/inventory) | `content/experiment.py:7`, `:35`, `:66-94` |

### Readers

| Function | Location |
|---|---|
| Pack fetch + cache + LFCNT1 length bound | `content/full157/reader.py:9-17` |
| Locator/directory/kind/bounds validation | `content/full157/reader.py:18-29` |
| Whole-file chain traversal + reverse replay | `content/full157/reader.py:30-42` |
| Entry point, cache, slice reconstruction | `content/full157/reader.py:43-54` |
| Canonical identity check (`_put`) and legacy pack decoding | `metadata/full157/reader.py:130-192` (file→`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/combined/store_api.py:130-192`) |
| Shared value-group reader inherited by both | `metadata/v2-157/reader.py` |
| Corruption checks + worst-amplification measurement | `content/full157/check_reader.py:12-49` |

### Protocol and report files of record

- `content/protocol.md` (53-state frozen policy; 11 lines) — §"Content representation".
- `content/full157/protocol.md` (157 extension, frozen before encoding; 11 lines) — states the frozen policy, the verifier-ordering change, and the cache accounting.
- `content/report.md` (53-state results), `content/full157/report.md` (157-state results).
- `content/execution-notes.md` (cache-accounting clarification + the failed first import).
- `content/full157/result.json`, `content/full157/coverage.json`, `content/full157/reader-checks.json`, `content/full157/run.log`.
- Product-repo evidence docs: `docs/roadmap/0.1/0.1.5/issue100/ordered-optimization-results.md:40-59`, `optimization-checklist-and-experiment-ledger.md:28-41`, `structural-investigations.md:29-48`, `git-algorithm-study.md:20,24,34`, `git-gap-directions.md:36-41,71-88`, `git-matcher-results.md`.
- Git baseline provenance: `layerfs-issue100-40mb-full157/git-baseline/protocol.md:1-5` and `verify.py:24-31` (`git verify-pack -v <idx>` written to `verify-pack.txt`).

---

## 3. SQLite-level deltas vs product schema 9, and the byte accounting

### 3.1 Schema

The candidate is `PRAGMA user_version = 9304`. Its `objects` and `object_packs` DDL is **byte-identical to product schema 9** (`crates/layerfs-layerstack-store/sql/schema/v9.sql:4-16`), including `CHECK(canonical_length > 0 AND canonical_length <= 16777216)`, `group_number < 256`, `record_number < 8191`, `STRICT`, `WITHOUT ROWID`. The prototype's largest new canonical length is 1,241,242 B, comfortably inside the 16 MiB check — **no schema change is required for the whole-file rows**.

Tables present at 9304 but absent from schema 9 (`sql/schema/v9.sql` has 7 tables + 7 indexes):

| Table | DDL (verbatim from the artifact) | Introduced by |
|---|---|---|
| `pool_groups` | `CREATE TABLE pool_groups(first_ordinal INTEGER PRIMARY KEY CHECK(first_ordinal BETWEEN 1 AND 4294967295), count INTEGER NOT NULL CHECK(count BETWEEN 1 AND 166), pack_id INTEGER NOT NULL REFERENCES object_packs(pack_id), group_number INTEGER NOT NULL CHECK(group_number BETWEEN 0 AND 255), group_sha256 BLOB NOT NULL CHECK(length(group_sha256)=32)) STRICT;` | the **metadata** priority (V2 physical value groups), not the content stage |
| `scope_allocator` | `CREATE TABLE scope_allocator(scope BLOB PRIMARY KEY CHECK(length(scope)=32), highwater INTEGER NOT NULL CHECK(highwater>=0)) STRICT, WITHOUT ROWID;` | metadata/structural stage |

543 rows in `pool_groups` (895,576 pooled values across 157 states); 1 row in `scope_allocator`. Indexes are unchanged (the same 7 as schema 9, including `layers_source`, `layers_child`, `layers_genesis`, `layer_identity`, `branch_identity`, `branch_names`, `layer_stack_names`). No index was added for the 523 new rows.

### 3.2 Content-stage deltas (measured here against the source artifact)

Source: `metadata/v2-157/candidate.sqlite` (SHA256 `96cfd0ea…15825`, 71,970,816 B apparent = allocated).
Candidate: `content/full157/candidate.sqlite` (SHA256 `efacb9b8…bec11`, 65,957,888 B apparent = allocated).

| Change | Source | Candidate | Δ |
|---|---:|---:|---:|
| `object_packs` rows | 1,328 | 732 | −596 |
| `object_packs` payload bytes | 65,970,381 | 60,313,439 | −5,656,942 |
| `objects` rows | 103,367 | 103,890 | **+523** |
| Content packs (LFSTRC v103 + LFPACK v2 + all LFCNT1) | 906 packs / 56,367,207 B | 310 packs / 50,710,265 B | −596 packs / −5,656,942 B |
| Untouched metadata packs (LFPACK v1) | 422 packs / 9,603,174 B | 422 packs / 9,603,174 B | 0 |
| `objects` in new content packs | — | 79,142 | (75,398 small + 523 large + 3,221 native) |
| `objects` left in retained packs | 24,748 | 24,748 | 0 |

Source content packs split: 389 `LFSTRC\0\0` v103 packs = 49,156,171 B (SmallContent) + 517 `LFPACK\0\0` v2 packs = 7,211,036 B (native chunks). Both populations are deleted and replaced by the 310 new `LFCNT1\0\0` v107 packs, which now hold **all** content: small, large and the native slice records.

**Large-file index rows (the 523 new `objects` rows).** `object_id` = BLAKE3(`layerfs/object/v2\0` ‖ canonical `LFSO|1|…|"LFSWFL1\0"‖raw`); `canonical_length` ∈ [131,100, 1,241,242] (raw 131,079..1,241,221); `pack_id` = one of the 310 new packs; `group_number` = flat record index; `record_number` = 0. No existing row is updated except `pack_id`/`group_number`/`record_number=0` for the 79,142 relocated content rows (`experiment.py:108-110`); no row is deleted (`removed = 0`, measured here); all original identities and lengths are preserved (`result.json` `original_canonical_ids_lengths_retained = true`).

### 3.3 Complete byte accounting behind 65,957,888 B

Pack payload (`result.json.all_pack_bytes = 60,313,439`):

| Component | Bytes |
|---|---:|
| SmallContent records (75,398) = frames 45,856,792 + kind/base header 2,100,102 | 47,956,894 |
| Large whole-file records (523) = frames 2,283,323 + kind/base header 16,459 | 2,299,782 |
| Native slice records (3,221 × 41 B) | 132,061 |
| Pack headers (310 × 16 B) + record directories (79,142 × 4 B) | 321,528 |
| **New content packs** | **50,710,265** |
| Metadata packs (unchanged) | 9,603,174 |
| **All pack payload** | **60,313,439** |

Header-byte derivation: 12,126 small FULL × 1 B + 63,272 small prefix × 33 B = 2,100,102; 25 large FULL × 1 B + 498 large prefix × 33 B = 16,459. Both identities were re-derived here from the artifact and match exactly.

Non-pack bytes = apparent − pack payload = 65,957,888 − 60,313,439 = **5,644,449**, and `dbstat` reconciles it exactly:

| dbstat component | Source | Candidate | Δ |
|---|---:|---:|---:|
| `object_packs` | 66,981,888 | 60,919,808 | −6,062,080 (= −5,656,942 payload − 405,138 page/overflow) |
| `objects` | 4,829,184 | 4,878,336 | **+49,152** (12 × 4 KiB pages for 523 rows) |
| `pool_groups` | 32,768 | 32,768 | 0 |
| `commits` / `branches` / `branch_identity` / `branch_names` / `sqlite_schema` | 28,672 / 24,576 / 12,288 / 12,288 / 12,288 | same | 0 |
| 9 × single-page tables/indexes (`layers*`, `layer_stack*`, `scope_allocator`, `workspace_stages`) | 9 × 4,096 | same | 0 |
| **Total (= apparent = allocated, freelist 0, page_size 4096, page_count 17,571 → 16,103)** | **71,970,816** | **65,957,888** | **−6,012,928** |

So the report's "remaining 355,986 B from SQLite page/overflow/layout effects" decomposes exactly as **−405,138 B of `object_packs` page/overflow overhead + 49,152 B of `objects` page growth**. The content-pack saving (5,656,942) plus that 355,986 exactly equals the reported 6,012,928 B total. Nothing is double-counted and no component is missing.

---

## 4. Base selection: the Git mechanism, and what a product-owned policy needs

### 4.1 Exact offline mechanism

1. `git verify-pack -v <pack>.idx > verify-pack.txt` is run read-only against the single retained pack of the matching `snapshots.git` (`git-baseline/verify.py:24-25`). No repack is performed (`git-baseline/protocol.md:2`).
2. `experiment.py:33-36` parses each line with 5 or 7 whitespace fields whose field 0 is 40-hex and field 1 is `blob`; for 7-field lines **field 6 is the delta base SHA1**. 5-field lines → `None` (FULL).
3. `raw(oid)` pulls each blob from `git cat-file --batch`, verifying `sha1("blob <n>\0"+b) == oid` **and** `sha256(b) == file_seals[oid]` where the seal came from the sealed fixture oracle (`experiment.py:42`).
4. The population is restricted to non-empty regular-file blobs found in the 157 sealed `manifest.tsv` inputs (`experiment.py:24-31`).
5. `visit()` produces a topological order, base before descendant (`experiment.py:65-72`).
6. Per target, in that order: encode FULL; if the Git base is already chosen, try `encode(data, raw(base))`; accept the prefix iff `len(alt)+32 < len(full)` **and** `depth+1 ≤ 50` **and** `closure_canonical + n ≤ 64 MiB` **and** `closure_encoded + 33 + len(alt) ≤ 64 MiB` (`experiment.py:76-82`).

Data the mechanism needs: (i) the matching pack's delta-base index (`verify-pack -v` output, 10.6 MB for 157 states), (ii) raw blob access to the same object database, (iii) the sealed per-state file inventories, (iv) per-blob oracles for the SHA256 cross-check. All four are oracle/fixture artifacts — **none of them exist in the product at write time**.

### 4.2 Measured: the chosen graph *is* Git's pack delta graph, 1:1

`layerfs-issue100-40mb-full157/git-baseline/result.json`: 157-pack populations are `blob_full = 12,159` and `blob_delta = 63,770`, max blob delta depth **50**.

Candidate artifact (measured here): 12,151 FULL records (12,126 small + 25 large) and 63,770 prefix records (63,272 small + 498 large). 12,159 − 12,151 = 8 = the empty-file blobs excluded by `n` non-zero. And `result.json.statistics.candidate_encodes = 63,770`.

Furthermore the gate counters `structural_reject`, `cost_or_encoded_reject` and `outside_population_full` are **absent from `result.json.statistics`**, which in the writer (`experiment.py:79-83`) means they were never incremented. Conclusion: **every Git-deltified blob became a prefix record and every non-deltified blob became a FULL record; the 50-edge/64 MiB gates and the cost gate rejected nothing.** The prototype's whole-file graph is Git's delta graph with no independent policy of its own, and the prototype's 50-edge bound is exactly Git's repack depth (default `pack.depth = 50`, `git-algorithm-study.md:24`).

### 4.3 Measured here: 73.8% of prefix edges need a base that does not exist yet

For each prefix record, the target and base Git OIDs were recovered (`small` OIDs from `layerfs-issue100-40mb-history-deltas/content/attribution.json`, `large` OIDs from `content/full157/coverage.json`), and each OID's first-appearance ordinal was computed by scanning the 157 sealed `manifest.tsv` files in order.

| Base class | Edges | Share | Stored record bytes (exact) | Raw bytes of targets |
|---|---:|---:|---:|---:|
| base first appears in a **LATER** state | 47,033 | 73.76% | 8,991,558 | 597,230,333 |
| base first appears in an **EARLIER** state | 14,713 | 23.07% | 3,400,570 | 180,344,285 |
| base first appears in the **SAME** state | 2,024 | 3.17% | 595,812 | 8,969,207 |
| (FULL records) | 12,151 | — | 37,268,736 | 105,500,038 |
| **Total** | **75,921** | | **50,256,676** | |

50,256,676 = `small_records + large_records` from `result.json` exactly.

Byte weight of the future-base dependency: a random 200-sample per class was re-encoded here (seed 7) and compared against the same target encoded with no prefix.

| Class | samples | FULL record B | prefix record B | saving ratio |
|---|---:|---:|---:|---:|
| future | 200 | 831,103 | 37,955 | 0.9543 |
| past | 200 | 634,815 | 45,020 | 0.9291 |
| same | 200 | 225,490 | 53,625 | 0.7622 |

Scaling class means to the real edge counts: the prefix graph avoids ≈231 MB of FULL record bytes, of which ≈186 MB (**≈80%**) is attributable to future-appearing bases and ≈20% to past/same-state bases. This is a *sampling estimate* (200 per class, one seed, uncontrolled OS cache); the edge counts and stored bytes in the table above are exact.

### 4.4 What is available at write time in the product today

| Capability | Exists? | Exact owner |
|---|---|---|
| Same-path predecessor root per file version | Yes | `objects.rs:2426-2448`, `:794-795`, `:3235-3236`; set via `objects.rs:2915-2952` (`set_physical_predecessor`) |
| Bounded predecessor span hints (`prior_ids[0..4]`, `has_predecessor`) | Yes | `objects.rs:164-215`, `:2465` (`PredecessorCursor::hints`), serialized in `objects/spill.rs:473-478`, `:598-606` |
| Batch-local candidate trial engine | Yes, bounded | `admission.rs:1212-1346` `DeltaSearch`: `trials == 512` cap, 16 MiB `remaining` budget, hints only, **no record written in the batch can be a base** (`admission.rs:1210-1211`) |
| Session-local content-keyed candidate cache | Yes, tiny and ephemeral | `small_candidates.rs:4-9` (128 KiB index, 1,024 slots, 8,192 references), `:39-63` 16-byte rolling-hash signature (8 min-hashes), `:91-106` requires ≥2 overlapping hashes and only FULL winners |
| Exact chunk dedup | Yes | chunks are content-addressed by their own canonical id (`extent_codec.rs:19-46`) |
| Persistent cross-snapshot similarity index | **No** | — |
| Filename/size-based candidate ordering (Git's `type_size_sort` / name hash) | **No** | — |
| Deferred / repack / physical-replacement phase with byte reclamation | **No** (explicitly out of scope) | `git-gap-directions.md:110-115`; `structural-investigations.md:75` |
| Whole-file object for files ≥ 128 KiB | **No** | `content.rs:8` `SMALL_LIMIT = 131072`; `content.rs:181-218` send everything ≥ SMALL_LIMIT to the rope/extent path |

### 4.5 Feasibility assessment

- **The cost rule is already the product's.** The prototype's acceptance test `len(alt)+32 < len(full)` (`experiment.py:80`) is identical in form to `admission.rs:272` (`delta.len() + 32 < frame.len()`). What the prototype changes is only (a) the closure budget: `CHAIN_ENCODED_LIMIT = 256 KiB` (`delta.rs:9`) → 64 MiB, and (b) the chain depth: `CHAIN_EDGES = 8` (`delta.rs:7`) → 50.
- **Past-only candidates are reachable and bounded.** 14,713 edges / ≈20% of the avoided bytes rest on bases that already exist when the target is written, so a predecessor-driven or persistent-candidate-index policy could in principle obtain them without any repack. The measured same-path policies already tried in this repo were both *net losses* (`structural-investigations.md:35-41`: forward −10,875,406 B, reverse −1,486,612 B), so "reachable" is a budget statement, not a demonstrated saving.
- **The remaining ≈80% is not reachable online.** 47,033 edges require a base that first appears later. Reproducing them requires deferred physical encoding, repacking with replacement of previously written encodings, and actual byte reclamation; the current format has no reclamation path and the repo explicitly excludes that project. This is the single largest integration risk and it is a *policy/lifecycle* problem, not a matcher problem.
- **A product-owned replacement policy would have to supply**: (1) candidate generation without Git — a persisted similarity index (name/size signature or CDC-chunk-set signature) over already-written whole-file objects; (2) candidate ranking that survives the absence of Git's name hash and descending-size preference; (3) an explicit decision about whether future bases are permitted (if yes: a deferred/repack stage with reclamation; if no: accept a materially smaller saving); (4) the enlarged closure budget and a read-cost accounting for it; (5) a per-object admission gate that is not "one batch cannot anchor another" (`admission.rs:1210-1211`).

---

## 5. Mapping table: prototype feature → exact current Rust owner to change

| # | Prototype feature | Exact current Rust owner | Nature of change |
|---|---|---|---|
| 1 | Whole-file object for regular files ≥ 128 KiB (role `LFSWFL1\0`) | `crates/layerfs-content/src/file/content.rs:8` (`SMALL_LIMIT`), `:9` (MAGIC), `:181-218` (`build_bytes`/`build` route everything ≥ SMALL_LIMIT to rope); store side `objects.rs:624` (`build_small_file`), `:673` (`build_complete_file`) | New representation class + a new role tag; today no such object exists |
| 2 | Envelope / identity for the new role | `crates/layerfs-content/src/object/codec.rs:11-12`; `crates/layerfs-content/src/object/references.rs:80,124` (magic dispatch) | Add tag + reference-walk arm (leaf) |
| 3 | New pack magic `LFCNT1\0\0`, version 107, flat record table, 4 MiB pack | `objects/pack.rs:11` (`MAGIC`), `:6-10` (limits), `:44` `header`, `:57` `entry`, `:101` `Version`, `:113` `versioned_header`, `:137` `versioned_entry`, `:791` `assemble`, `:856` `decode_group` | New container version **or** a new flat-record container; the product is two-level (group→record), the prototype one-level |
| 4 | Record grammar kinds 0–5 with implicit lengths | `objects/delta.rs:19-74` (existing kinds 0/1/2 with explicit `u32 raw_length`, `u32 frame_length`, optional 32 B base) | New parser; the prototype derives raw/frame lengths from `canonical_length` and the offset directory |
| 5 | Frame limit 2 MiB + 1 KiB | `objects/delta.rs:6` (`FRAME_LIMIT = 135,168`); `objects/pack.rs:163` (`NATIVE_FRAME_LIMIT = 33,024`) | ~15× larger frame bound |
| 6 | Chain depth 50 | `objects/delta.rs:7` (`CHAIN_EDGES = 8`) | 6.25× deeper |
| 7 | Canonical/encoded closure 64 MiB each | `objects/delta.rs:8-9` (`CHAIN_CANONICAL_LIMIT = 512 KiB`, `CHAIN_ENCODED_LIMIT = 256 KiB`) | 128× / 256× larger budgets; also drives decoder workspace |
| 8 | Cost test `prefix+32 < FULL` | `objects/admission.rs:266-278` (already identical) | **No change needed** |
| 9 | Prefix selection for SmallContent (anchors = predecessor then session candidate cache) | `objects/admission.rs:212-303`; `objects/small_candidates.rs:4-106` | Extend to a persisted candidate source instead of a 128 KiB session cache |
| 10 | Native chunk slice: `kind 4 = owner[32] ‖ offset u32 ‖ length u32`, owner must be whole-file kind 2/3 | `objects/pack.rs:180-205` (`native_record` kind 0/1 prefix over *chunk* bases), `:243-259` (`native_encode_record`), `:210-241` (`native_record_range`) | New record kind whose base is a whole-file object, not a chunk; needs a range read *inside* a decoded whole-file object |
| 11 | Slice range check + owner graph authentication before slicing | `objects/read.rs:735` (`native_chain`), `:706` (`read_native_prior`), `:216` (`validate_small_packs`), `:246` (`small_physical_base`), `:266` (`small_anchor`), `:282` (`small_predecessor`), `:295` (`small_chain`), `:370` (`read_small`), `:950` (`singleton_range`), `:985` (`singleton`) | New read path: decode whole-file chain, slice, re-wrap, authenticate |
| 12 | Sub-range of a payload object referenced from an extent | `crates/layerfs-content/src/file/extent.rs:9-13` (`ExtentSliceV3{payload_object_id, source_offset, logical_length}`) | Precedent exists, but today the payload object is a ≤32 KiB chunk object |
| 13 | Per-target / per-batch optional-work budget | `objects/read.rs:23-31` `HintReadBudget`, `:43-53` `charge` (512 KiB target, 8 MiB batch), `:634`, `:722` (8 target fetches) | 64 MiB closure is 128× the target budget — must be raised or the design rejected |
| 14 | Reader canonical cache 4 MiB, pack cache 8 MiB | Product read path has **no** canonical-object LRU; only SQLite page cache `schema.rs:15` (`SQLITE_PAGE_CACHE_KIB = 32 MiB`) and `prepare_cached` statements (`read.rs:141,165,192`) | A cache must exist for the measured amplification to be tolerable at all |
| 15 | Zstd pinned parameters + strict frame contract | `objects/pack.rs:334-348` (`native_compress`/`native_decompress`/`small_decompress`), `:30` `Codec` | Window/checksum/content-size enforcement for the larger frames |
| 16 | 523 extra `objects` rows; content locators repointed; 906 packs deleted, 310 written | `sql/schema/v9.sql:9-16` (`objects`), `:4-7` (`object_packs`); packing/publish `objects/admission.rs:560` (`flush_native_group`), `:648` (`prepare_ordinary`), `:880-1050` (`publish`) | **No schema change**; largest new canonical length 1,241,242 « 16 MiB check |
| 17 | Pool/allocator tables (`pool_groups`, `scope_allocator`) | absent from `sql/schema/v9.sql` | Owned by the **metadata** priority, not this content stage; required for user_version 9304 |

---

## 6. Measured read amplification and what it implies for the product read path

### 6.1 Measured numbers (full157, `content/full157/reader-checks.json`; 53-state in `content/report.md:37-42`)

| Selection | Returned chunk | Owner raw | File-graph raw decoded | Amplification | Edges | Cold read (Python, OS cache uncontrolled) |
|---|---:|---:|---:|---:|---:|---:|
| Largest decode/output ratio | 6,421 B | 543,792 B | 23,758,968 B | **3,700.20×** | 35 + 1 | 420.83 ms |
| Largest owner graph | 17,979 B | 471,835 B | 29,807,446 B | 1,657.90× | 47 + 1 | 525.45 ms |
| 53-state worst ratio | 6,421 B | — | 15,608,785 B | 2,430.90× | 23 + 1 | 277.37 ms |
| 53-state largest graph | 9,306 B | — | 20,466,017 B | 2,199.23× | 37 + 1 | 366.92 ms |

Supporting measurements: the first read fetched 1,241,516 B of physical packs in 4 pack reads (encoded file records 329,803 B); the second fetched 2,211,294 B in 7 pack reads. Dataset maxima: file edges 50, canonical closure 29,808,454 B, encoded closure 483,545 B, frame 483,544 B, pack 735,861 B. Exhaustive verification decoded **13,979,051,959 canonical file bytes** and fetched **10,435,269,869 physical pack bytes** across 45,328 cache misses in 462.80 s. Reader-retained payload is ≈20 MiB (4 MiB canonical + 8 MiB packs + ≈8 MiB inherited pool), explicitly not an RSS bound (`content/execution-notes.md:3`).

Measured here, the structural reason for the amplification: of the 501 distinct slice owners, **only 25 are FULL records**; 476 owners are themselves prefix records with chain depths 1..50, so **3,196 of 3,221 adapters (99.2%) require a chain decode to obtain their owner**. The same owner is shared by up to **62 adapters** (75 owners have exactly one), so per-owner caching can amortise up to 62 visits — but only if the cache holds the decoded owner.

### 6.2 Implications for the product read path

- **Cold read cost is set by the owner chain, not by the chunk.** A ≤32 KiB chunk read now requires fully reconstructing an owner chain that can decode ~24–30 MB. There is no way to decode only part of the chain: each link's raw bytes are the zstd prefix of the next link (`reader.py:38-41`), so link *d* cannot be obtained without links *0..d−1*.
- **Slice boundary crossing is unconstrained.** The slice offset/length are arbitrary u32 values inside a ≤2 MiB raw owner (`reader.py:51`), so a 4 KiB page read at any file offset can land anywhere in the owner's raw bytes. There is no alignment rule that would let the reader restrict the decode. Chunk-level chunking was already the product's I/O unit (≤32 KiB, `gear.rs:9`); the prototype decouples the read unit from the storage unit entirely.
- **Cache limits are contradicted by measurement.** The prototype's 4 MiB canonical cache cannot hold a 23.8–29.8 MB closure: every miss evicts the entire chain it just decoded, so consecutive adapters sharing an owner thrash (`content/report.md:44` records 31,304 misses on 53 states and 12.0 GB decoded). The product has *no* canonical-object cache at all — only a 32 MiB SQLite page cache (`schema.rs:15`) — so this design cannot be adopted without adding one.
- **Pack/record bounds are exceeded.** Max actual pack 735,861 B > product `PACK_LIMIT = 262,144 B` (`pack.rs:7`); max frame 483,544 B > `FRAME_LIMIT = 135,168` (`delta.rs:6`) and ≫ `NATIVE_FRAME_LIMIT = 33,024` (`pack.rs:163`). Both need the explicitly-unsupported grammar the protocol froze (`content/protocol.md:9`).
- **The per-target work budget is 128× too small.** `HintReadBudget` allows 512 KiB encoded/decoded per target and 8 MiB per batch (`read.rs:43-53`); the measured closure permits 64 MiB each. Either the budget rises by two orders of magnitude or the amplification is a hard failure.
- **Workspace/reservation assumptions break.** Static decoder workspace and reservations are sized for 512 KiB chains (`delta.rs:8-9`, `pack.rs:165` `NATIVE_DECODE_WORKSPACE = 262,144`, `objects.rs:47-56`). A 2 MiB-frame decode plus a 64 MiB chain needs a different reservation model.
- **Direction of the fix.** Because the owner is shared by up to 62 adapters, a *retained-decoded-owner* cache keyed by owner id (rather than by chunk id) is the only measured behaviour that helps; the frozen protocol already had to reorder verification by owner to avoid "arbitrary chunk-hash-order thrash" (`content/protocol.md:9`, `content/full157/report.md:61`). That is a diagnostic ordering change that the product read path has no equivalent of today.

---

## 7. Open risks / unknowns

1. **Future-base dependency is the dominant risk.** 47,033/63,770 edges (73.8%) and ≈80% of the avoided bytes use a base that first appears in a later snapshot (measured here). The product's append-only, immutable-placement model cannot express this without deferred encoding plus physical replacement and reclamation, which the repo excludes (`git-gap-directions.md:110-115`). *The evidence does not determine* how much saving a past-only online policy could reach; that requires a policy experiment, not an inference.
2. **The prototype has no policy of its own.** Because `structural_reject = cost_or_encoded_reject = outside_population_full = 0`, the executed graph is Git's pack delta graph 1:1 and its 50-edge bound equals Git's `pack.depth`. Any reading of "the saving" as evidence about a *LayerFS* base-selection algorithm is unsupported by this artifact.
3. **Sampling estimate width.** The ≈80%/≈20% byte split in §4.3 comes from 200 encodes per class with one seed and uncontrolled OS cache. The edge counts and stored bytes are exact; the *byte-weighted split* is an estimate and could move by several points with a different sample.
4. **Read amplification is a Python diagnostic.** All timings (420.83 ms, 525.45 ms) are single Python reads with uncontrolled OS cache, explicitly not product latency (`content/full157/report.md:46`). No product read path was built or measured, so the real cost of the same graph under SQLite blob reads is unknown.
5. **No product-side memory/RSS bound exists for a 64 MiB closure.** The prototype's ≈20 MiB retained-payload accounting excludes graph records, decoder contexts, Python overhead and temporaries (`content/execution-notes.md:3`). The evidence does not determine product peak memory.
6. **Corruption coverage is five in-memory mutations** (`check_reader.py:24-28`). Not covered by any executed check: slice offset mis-targeting that still lands in range, base substitution between two same-length objects, chain-swap between two owners, pack-directory reordering, and any kind-5 path (zero instances exist in full157, so kind 5 was never exercised end to end).
7. **Owner selection is "first Git OID order, then lowest offset"** (`content/protocol.md:7`, `experiment.py:51-53`). That rule needs *all* owners' Git OIDs and *all* chunk offsets simultaneously — an offline, whole-population decision. The product's admission is streaming and batch-local (`admission.rs:1210-1211`), so this rule has no online equivalent and no measurement of how much owner choice matters for amplification was made.
8. **`canonical_length` CHECK vs 2 MiB cap.** The schema allows up to 16 MiB (`v9.sql:11-12`) but the frozen policy caps raw at 2 MiB; the cap's origin is the observed maximum (1,241,221 B raw) plus headroom, **not** a product limit or a read-cost analysis. Any future sweep of that bound is unmeasured.
9. **The 523 whole-file objects are new identities.** Every root/namespace pointer is unchanged because the native adapters and the file-content roots still reference the *original* objects (`content/report.md:31`); the 523 large objects are retained beside them, not substituted for anything. Which layer is supposed to own those extra objects in the product (and how they are reclaimed if a policy is reverted) is not determined by the evidence.
10. **The metadata-side additions are prerequisites, not part of this scope.** `pool_groups`/`scope_allocator` and user_version 9304 must land before any of this content grammar can be embedded in a real Store; and the 157-state content result is only reachable on top of the V2 metadata baseline (71,970,816 B), which itself carries a documented metadata cold-read cost (`ordered-optimization-results.md:36`).

---

## 8. Evidence index

| Item | Path |
|---|---|
| 157-state content writer | `layerfs-issue100-ordered-optimization/content/full157/experiment.py` |
| 157-state content reader | `…/content/full157/reader.py` |
| Pinned codec | `…/content/full157/content_codec.py` |
| Corruption + amplification checks | `…/content/full157/check_reader.py`, `reader-checks.json` |
| 157-state protocol / report / result / coverage | `…/content/full157/{protocol.md,report.md,result.json,coverage.json,run.log}` |
| 53-state protocol / report / result | `…/content/{protocol.md,report.md,result.json,execution-notes.md,experiment.py,reader.py,content_codec.py}` |
| Base Git pack inventory (157) | `layerfs-issue100-40mb-full157/git-baseline/{verify-pack.txt,verify.py,protocol.md,result.json,report.md}` |
| Product-repo evidence docs | `layerfs-issue100-40mb/docs/roadmap/0.1/0.1.5/issue100/{ordered-optimization-results.md,optimization-checklist-and-experiment-ledger.md,structural-investigations.md,git-algorithm-study.md,git-gap-directions.md,git-matcher-results.md}` |
| Product schema 9 | `layerfs-issue100-40mb/crates/layerfs-layerstack-store/sql/schema/v9.sql` |
| Product pack/delta/read/admission code | `…/crates/layerfs-layerstack-store/src/objects/{pack.rs,delta.rs,read.rs,admission.rs,small_candidates.rs,spill.rs}`, `src/objects.rs` |
| Product content code | `…/crates/layerfs-content/src/file/{content.rs,extent.rs,extent_codec.rs,cdc/gear.rs}`, `src/object/{codec.rs,references.rs}` |

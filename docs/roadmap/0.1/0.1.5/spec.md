# v0.1.5 specification: whole-file CAS and bounded delta storage

> **Issue #100 measured outcome, 2026-09-10:** The retained implementation allocates
> **49,319,936 bytes**, with **49,250,304 bytes** growth, ten Created outcomes,
> exact same-Store verification and clean teardown. It is **4,319,936 bytes above
> 45,000,000** and is **not near-target**. Commit median/sum exceed the prospective
> 10% working criterion; save/paired medians and historical-read wall remain close
> to the original baseline. See [the consolidated results](issue100/storage-optimization-results.md)
> and [complete retained-candidate evidence](issue100/retained-candidate-1-results.md).
> The owner subsequently authorized full157 despite the ten-state target miss.
> [Full157 confirmation](issue100/retained-full157-results.md) completed at
> **134,246,400 B**, with157 Created outcomes, same-Store verification and clean
> teardown. It saves27.27% versus released control but has31.13% higher paired
> median latency. The issue remains open; no release-admission PASS or release.

## 1. Baseline, scope and fixed settings

Continue the current v0.1.5 implementation in `codex/issue100-full157`, including
its upper-range exact-CAS repair. Released v0.1.4 commit
`101fa273d815f3aaedb0e06ba0de7b0777d83def` is the immutable comparison product,
not a replacement starting point. Preserve other tasks' changes and the released
#95/#98 behavior described in section 8.

| Setting | Fixed value |
| --- | --- |
| Small regular-file content | 1..131071 raw bytes |
| Large regular-file content | >=131072 raw bytes |
| Empty regular file | Existing compact empty extent/file-state representation |
| Large-file CDC profile | 8192 minimum / 16384 target / 32768 maximum bytes |
| New SQLite pages | 4096 bytes; supported existing 65536-byte layouts retained |
| New small codec | Existing Zstandard, level 3, windowLog 18, workers 0 |
| Frame flags | Content size and checksum enabled; dictionary ID disabled |
| Schema-9 new delta depth | At most 8 SmallContent dependency edges |
| Schema-9 decoded closure | At most 512 KiB canonical bytes, including target |
| Schema-9 retained encoded records | At most 256 KiB actual capacity |
| Candidate bases | At most one known eligible anchor per new small object |
| Ordinary pack limit | 256 KiB; existing maximum 256 groups |
| New small group / encoded-frame cap | 192 KiB / 132 KiB |

The fixed 256-KiB codec window covers the maximum base-plus-target span below
256 KiB. This replaces the obsolete draft's windowLog 17, which could exclude
shifted matches near the cutoff. It is a fixed match-coverage choice,
not an instruction to compare windows or compression levels. Keep existing
large-file encoding parameters unchanged. No setting sweeps or runtime tuning menu.

The new small representation applies to newly prepared regular-file content
through Init, SDK edits, and ordinary saves. It does not apply to metadata ropes,
arbitrary Bytes objects, or untouched historical file roots. Multi-workspace/
branch expansion stays v0.1.6; new GC, repacking and global similarity indexing
are outside scope.

## 2. Canonical small-content object and shared file dispatch

Released outer object kinds and canonical hashing stay unchanged. Introduce a
new logical role inside the existing `Object::Bytes` framing:

```text
SmallContent value:
  8 bytes: b"LFS5SML\0"
  u16 big-endian version: 1
  raw file bytes: 1..131071 bytes

Existing outer Bytes encoding adds 13 bytes.
Canonical length = 23 + raw length.
ObjectId = existing domain-separated canonical digest.
```

A regular inode's `content_root` points directly to this whole-content object.
No extra Small FileState, extent wrapper, or member directory is needed. Empty
files keep the existing empty representation; writers do not emit zero-length
SmallContent and readers reject it. Keep existing `LFS4CHK` encoding and its
32-KiB raw limit unchanged. Old object bytes/IDs are never reinterpreted.

Equal new small content has one ObjectId across names, Init and edit surfaces,
regardless of physical FULL/DELTA encoding. Its ID intentionally differs from an
old extent-backed file-state root containing the same visible bytes. Cross-format
visible equality does not imply canonical-root equality or automatic sharing.

Add a common regular-file content-root dispatch in `file/content.rs`. It owns
SmallContent role parsing/construction and dispatches length/read-range/stream
operations to either SmallContent or existing FileState/rope routines. Route all
regular-file consumers through it: filesystem reads/apply/reconcile, workspace
construction/materialization, backing reads, checkpoint validation, inspection
and content-accounting paths. Keep `FileStateRoot` restricted to actual extent
states; do not fabricate extents to hide new SmallContent roots. Generic metadata
rope APIs retain their existing semantics.

Extend canonical-role reference traversal: SmallContent has no canonical child
objects. Its physical delta base is handled by Store dependency traversal, not
invented as a canonical child. Validate magic, version, canonical framing and
raw length before interpreting data or allocating large operands.

## 3. One Init/Commit pipeline

```text
 Init: source readers/metadata       Commit: frozen pieces + predecessor facts
                  \                   /
                   v                 v
               Common regular-file preparation
                  canonical size/role decision
                              |
                  Exact CAS membership/equality
                              |
                  FULL/DELTA encoding selection
                              |
                  Bounded admission and packing
                              |
                 Namespace completion/publication
```

Use existing `run_finalized_output`, `FinalizedOutputWriter::build_complete_file`,
checked admission and Workspace construction seams. Init retains bounded native
discovery; Commit retains dirty-frontier capture. Both use one canonical builder,
codec policy, admission and publication mechanism. Lifecycle inputs/preconditions
are distinct: initial Layer/LayerStack creation is not a Branch advancement.
Do not force one SQL transaction or replay Init through FUSE to make them identical.

For changed small content, materialize/hash the bounded final target once through
shared preparation. Pending live pieces remain mutable until capture; do not
construct canonical small objects after every FUSE write. Carry ready target
bytes and trusted construction facts forward rather than rereading through POSIX,
spooling and rehydrating completed output, or enabling the unrelated old
CaptureState shortcut for remote live capture.

For large known edits, retain unchanged extents and process replacement ranges.
A complete large overwrite can require streaming its supplied bytes; no new
large-file full scan is introduced merely to obtain a delta base.

## 4. FULL/DELTA representation in pack version 3

Keep the seven existing tables and existing locator shape:

```text
objects(object_id, canonical_length, pack_id, group_number, record_number)
object_packs(pack_id, data)
commits / branches / layers / layer_stacks / workspace_stages
```

Use existing pack magic `LFPACK\0\0`, 16-byte header and 16-byte group directory.
Header version 3 introduces small-content groups; versions 1/2 remain unchanged.
Each v3 group stores exactly one SmallContent object; `record_number` must be 0.
Multiple groups share a pack. A group is not a transaction or separate disk file.

Outer group codec is RAW; encoded and decoded directory lengths equal the exact
group byte length. Dispatch version 3 before legacy 64-KiB group checks. Apply the
192-KiB group and 256-KiB total pack bounds only to the appropriate new grammar.

Group payload, no padding, little-endian integer fields:

```text
u8 kind                 0 = FULL, 1 = FULL-base DELTA, 2 = bounded-chain DELTA
u32 raw_length          1..131071
u32 frame_length        1..135168
[32 bytes base ObjectId] DELTA only
frame_length bytes      exactly one Zstandard frame
```

FULL frames decode without a dictionary. Kind 1 retains exactly its original
one-edge FULL SmallContent base interpretation, including in schema 9. Kind 2 is
admitted only in schema 9 and uses the raw bytes of its named SmallContent base,
FULL or bounded DELTA. A bounded removed-name hint can name another path's retained
SmallContent; this changes base discovery without changing canonical identity. Its complete closure has at most
8 edges, 512 KiB summed canonical bytes including the target, and 256 KiB retained
encoded record capacity. Unknown kinds and kind 2 in schema 8 are rejected.
The reconstructed canonical object uses
the fixed framing in section 2; its length must match the locator and its digest
must equal the requested ObjectId. Raw file length is not canonical length.

V3 writers emit 1..256 groups with zero reserved directory bytes and group ranges
contiguously covering the payload after the complete directory, ordered without
overlap or gaps. The last group ends exactly at pack EOF. Whole-pack validation
checks this complete structure. Point acquisition retains the existing bounded
selected-entry model: validate header/count, selected entry/reserved bytes, its
bounds and exact selected-group framing; do not reread every sibling group to
serve one authenticated object. Full integrity traversal checks the global
coverage rules, while trusted freshly prepared output carries construction facts.

Reject unknown tags, wrong versions/lengths, overflowing offsets, missing/trailing
bytes, multiple frames, invalid checksums/content size, excessive window/memory,
nonzero record ordinal and incompatible object roles. Check all sizes before
allocating. Check the pinned compressor's bound fits the fixed frame cap. FULL
and DELTA both retain exact decoded-size/EOF checks. Do not feed v3 bytes through
legacy/native grammar or widen old canonical limits globally.

## 5. Deterministic base selection and deduplication

Resolve exact CAS reuse through existing batched membership/equality first. If
the target exists, use its selected representation and emit no duplicate payload.
Do not encode first and then reread the target simply to rediscover that hit.

Schema 9 selects at most one eligible base for encoding:

- Schema 9: authenticate/reconstruct the immediate SmallContent predecessor and
  retain its depth, summed canonical closure and encoded closure facts. Select a
  new kind-2 DELTA only when adding the target satisfies all three fixed limits.
- Schema 8: keep the original FULL predecessor/direct FULL-anchor selection and
  kind-1 writer policy. Opening schema 8 does not enable kind 2.
- Before file production, schema-9 Workspace task planning may resolve frozen
  removed names and bounded removed subtrees into a temporary basename/root
  catalogue. A new nonempty small target with no genuine predecessor may use a
  unique distinct root for its basename. Ambiguities/cap exhaustion/low budgets
  skip discovery. Never fabricate a before inode. The [amendment](issue100/removed-base-amendment.md)
  fixes 16384 dirty-node, 32768 change-entry, 4096 removed-entry, 64-depth and
  8192 metadata-call bounds, <=1-MiB catalogue/queue ownership, and a 4-MiB
  planning allowance floor. Existing batched readers authenticate metadata.
- With no eligible hint, query the compact selected-FULL cache: 1024 records stored
  once plus 8192 u16 references, charged within the existing 128-KiB reservation
  from admission index ownership. Eight rolling 16-byte fingerprints produce at
  most eight probes and one chosen candidate with at least two matches. Preserve
  ObjectId tie-breaking, authenticate the exact selected FULL and emit kind 1
  only if economical. There is no second candidate encoding trial.
- Register only actual FULL winners after publication, outside Store locks, from
  already-owned canonical bytes; omit final-batch registration. A retained session
  moves the cache to one idle StoreDb slot while holding its writer permit; the
  next admission takes it without cloning. At most the same 128 KiB survives
  while idle. Rollback discards inherited/private hints; reopening starts empty.
  See [index ownership](issue100/compact-candidate-amendment.md) and
  [retained handoff](issue100/retained-candidate-amendment.md).
- Cached DELTA registration is not retained in the product: both tested variants
  increased content storage and Commit cost. CDC predecessor reuse remains
  unimplemented. No eligible base means FULL.

An optional hint whose eligibility cannot be established may fall back to the
prepared FULL. Missing/corrupt/wrong-role dependencies of a selected persisted
DELTA are integrity errors. Kind 1 still requires its directly named base to be
FULL. Kind 2 validates the entire bounded chain and authenticates each decoded
canonical object before using its raw bytes for the next step.

Acquire at most nine selected records iteratively, checking locator lengths,
cycles, chronological dependencies, complete decoded closure and actual encoded
capacities before reconstruction. There is no recursive history walk, copied
physical base, global similarity index, or rewrite of selected representations.
A base already has a stable selected location under the existing admission owner;
carry those verified facts forward without a second readback. At a prospective
size/depth/encoded ceiling, FULL is the complete fallback.

Reuse scoped predecessor facts from the existing workspace provenance/batched
lookup, including supported replace-by-rename relations. Do not infer relationships
through a new global path/similarity index or add a per-file transport negotiation.
If neither scoped facts nor the bounded selected-FULL cache provide an eligible
base, FULL is the complete fallback.

Prepare one compressed FULL alternative and at most one DELTA alternative, using
already owned target/base bytes. Choose DELTA only for strictly smaller complete
incremental serialized cost, including its 32-byte base reference and pack/group
framing. FULL wins ties; retain the prepared FULL buffer for fallback rather than
compressing it again. No fixed percentage savings threshold is added.

A late CAS duplicate discards private losing output and retains the winner. There
are no partial member wins: one new small object has one selected locator. Count
base retention separately from logical target ownership; no history census is
needed in the encoding hot path to speculate about future reclamation.

## 6. Reads, retention and transitions

Read small FULL directly; preserve the kind-1 authenticated FULL-base fast path.
Read kind 2 through the shared iterative bounded reader, authenticating each node
and the complete target. Exact-CAS comparison, including targets above 64 KiB,
uses that same reader. A short range may require reconstructing the bounded
complete object. Clear the old read-wave small-base cache before a kind-2 chain;
no unbounded raw cache or recursive history walk is introduced. Legacy v2 PREFIX
read support and its existing depth/codec rules remain unchanged.

Physical base closure must be included in admission/staging, integrity traversal,
accounting, rollback and any future reclamation. Canonical reachability alone is
insufficient. Do not delete a base retained only by a delta or claim a production
GC exists. Stable locations/owned bytes remain valid across codec work outside
shared locks; do not discover missing bases after publishing a dependent root.

| Transition for changed final content | Required behavior |
| --- | --- |
| Small to small | Assemble bounded final target; CAS reuse or FULL/DELTA. |
| Small to large | Stream resulting content into existing chunked construction; reuse matching chunks. |
| Large to small | Read only the retained result and necessary extent/decode dependencies; create/reuse SmallContent, normally FULL without an eligible small predecessor. |
| Large to large | Preserve known-range locality and existing extent algorithms. |
| Any to empty | Use existing compact empty file-state representation. |
| Metadata-only/unchanged | Preserve current content root, including legacy small extent roots. |

Length changes, shifted suffixes, insert/delete/prepend/append, unequal replacements,
truncate and zero extension must work. New small objects have no CDC count oracle;
large-file canonical chunk-count contracts stay fixed. Several boundary crossings
before one Commit cause one persistent representation decision from final facts.
Earlier versions remain readable, including alternating small/chunked episodes.

Live inode identity, hard links, open-unlinked lifetime, rename replacement,
read-after-write, SDK/FUSE cache reconciliation and checkpoint installation remain
correct. Before root publication, failures preserve the old published root;
a failure after successful publication is reported as a post-publication failure,
not falsely as rollback or permission to issue a duplicate Commit.

## 7. Bounded memory and authentication ownership

Serialize new small-object physical encoding, including FULL objects from Init,
at the existing admission consumer. Parallel Init producers prepare bounded
canonical SmallContent with existing queue limits; they do not each allocate a
new codec context. One admission-owned encoder at a time borrows the shared
allowance after CAS selection. Reserve at most 3 MiB for that operation: 2 MiB static aligned codec
workspace and 1 MiB total target/base/FULL/DELTA/output/handoff buffers. Fit it
within the existing 6-MiB data and 2-MiB physical-output ledgers alongside actual
producer/queue ownership; do not add those amounts as uncharged global buffers.
Existing pure-new Init scheduling stays bounded and does not reserve a base.

For small reconstruction reserve at most 2 MiB per active owner: <=1 MiB static
DCtx/DDict storage, <=256 KiB retained encoded records, four simultaneously live
raw/canonical-sized buffers of at most `131071 + 23` bytes, and <=16 KiB chain
associations. Four buffers account for the previous canonical base, decoded raw,
intermediate SmallContent framing and final canonical output. Admission's incoming
target is also charged while it survives predecessor reconstruction. The reader
checks actual Vec capacities and separately bounds acquisition of the next
<=192-KiB group before any decoder or raw operands are live. Existing read-wave
and producer/output ledgers retain their separate ownership charges.

Decoder and encoder never overlap during base acquisition. The encoding operation
keeps its existing 3-MiB total: an actual 2-MiB static encoder plus <=1 MiB combined
target/base/FULL/DELTA/output/handoff ownership. Validate pinned static estimates,
reset borrowed references on success/failure, and reject resource violations;
there is no heap codec fallback or parameter sweep.

Preserve authenticated canonical owners through private handoffs. Fresh persisted
or spilled operands are authenticated; a trusted in-flight buffer need not be
rehash-verified at each internal stage. Prepared physical spill uses its retained
exact-byte digest and EOF validation rather than a forced canonical decode loop.
Exact collision comparison remains required even when authentication is reused.
CPU-heavy compression/reconstruction runs outside the Store mutex and broad live
state locks where stable ownership permits. Never hold a write lock across host
transport to fetch a base.

## 8. Released speed invariants

- **#95 Init:** 2-MiB authenticated comparison reuse comes from the existing
  16-MiB index budget. Require current exact location, canonical length and byte
  equality on each comparison; charge Vec capacity plus 2048 bytes per entry/node,
  clear on saturation and bypass oversized entries. No new global cache.
- **#98 Workspace:** preserve SQL coalescing, same-owner pending reads and rollback
  across pending/already committed private batches. Comparison reuse remains 0;
  new bounded decode ownership does not silently enable the Init cache here.
- Physical batches stay separate from strict **<8192 objects / <4 MiB** SQL
  cohorts. Flush `commit_pending()` before staging opens its separate transaction.
  Small objects/groups do not create a transaction per object.
- Ordered spill visits keep `min(buffer_bytes, 64 KiB)` read-ahead; sequential
  scanning retains its separate buffer. A larger valid object does not justify
  restoring oversized random read-ahead.
- Preserve dirty-frontier work, borrowed/prepared output, paged object acquisition
  and host-owned Store/spool. No second capture pass, global workspace scan,
  per-object transport loop, status polling, or redundant predecessor readback.

See [past mistakes](past_mistake.md) and the source-bound [release report](benchmark-v0.1.4-report.md).

## 9. Store capability and explicit upgrade

New Stores use schema 9 with the same seven SQL tables and 4-KiB pages. Schema 8
continues to fence the SmallContent canonical role and original pack-v3 kinds;
schema 9 additionally fences kind 2. Supported schema 6/7/8 opens do not promote,
and writes retain each schema's supported construction/encoding policy. Carry the
format capability once per operation, not through a query for every file.

`LayerStackStore::upgrade_format(path)` explicitly upgrades schema 7 or 8 to 9.
Read-only preflight precedes exclusive revalidation and a real SQLite transaction
using DELETE journal/FULL synchronous. Require no live owners; change format
capability without rewriting payload/history or page size. Failure before COMMIT
preserves the original schema; failure after COMMIT reports promotion. Schema 9
is an idempotent no-op. Unsupported schema 5 and schema-6 upgrade requests remain
errors; existing supported schema-6 reads/writes retain their compatibility policy.

Pre-amendment binaries reject schema 9 before normal mutable connection setup.
A downgrade requires a pre-upgrade backup; changing the version integer is not a
supported rollback once new records may exist. Normal MEMORY/OFF acknowledgement
and durability semantics remain unchanged when the Store is reopened.

The owner-authorized canonical/Store evolution is a scoped v0.1.5 exception in
[release policy](../../../general/release-policy.md). Ordinary write/fsync/Commit
acknowledgements do not gain crash/power-loss durability; MEMORY journal and
synchronous-OFF normal operation remain their documented contract.

## 10. Current measurement and completion boundary

Use the [ten-snapshot contract](issue100/ten-snapshot-contract.md): `deepseek-ten`
with the exact retained full157 indices 1, 18, 36, 53, 70, 88, 105, 122, 140, 157.
Reuse the applicable immutable Git, released-v0.1.4 and existing-v0.1.5 baselines.
Run focused changed-owner checks and matched public performance, freeze allocation
and Store identity, collect the census, then reopen the same Store in a fresh
coordinator for exhaustive original-oracle verification and cleanup. Do not
rebuild a second history to substitute for that verification.

The [chain1 result](issue100/chain-1-results.md) is 56,668,160 B final allocation,
11,668,160 B above the 45,000,000-B objective. It is an improvement, not near-target
or task completion. The selected-FULL cache is a later measured implementation;
its separate report above supersedes chain-only status. Cross-CDC reuse remains
unimplemented. The subsequently owner-authorized full157 confirmation is
complete at134,246,400 B with exact same-Store verification. Its objective is
separate from the ten-snapshot45-MB target; preserve the earlier regression and
the new foreground-latency tradeoffs in [the report](issue100/retained-full157-results.md).

The original ten-file/thirty-commit smoke and its 31-state verifier are historical
first-round evidence in [smoke-report.md](smoke-report.md), not current execution
instructions. Broad Cargo/Clippy/doctest, unrelated qualification, parameter sweeps
and repeated unchanged measurements remain out of this issue's exploratory loop.
The smoke and focused checks do not establish exhaustive compatibility, POSIX,
corruption, failure or release qualification. No release is published or tagged.

## Released source anchors (historical control)

[Canonical roles and codec](https://github.com/Ephemeral-AI-Lab/layerfs/tree/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-content/src/object),
[existing file formats](https://github.com/Ephemeral-AI-Lab/layerfs/tree/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-content/src/file),
[shared output/admission](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-layerstack-store/src/objects.rs),
[Workspace producer](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-workspace/src/changes.rs),
[pack grammar](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-layerstack-store/src/objects/pack.rs),
[released storage contract](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/docs/versioned/0.1.4/storage-format.md).

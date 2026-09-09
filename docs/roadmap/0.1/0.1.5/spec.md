# v0.1.5 specification: whole-file CAS and bounded delta storage

> **Status:** Reconciled implementation contract, 2026-09-09. No runtime changes
> or new measurements are claimed. Implement the complete scope below; execute
> only the ten-file/thirty-commit smoke and its 31-state verifier in this round.
> [Implementation plan](implementation_plan.md) assigns files and sequencing;
> [workflow](workflow.md) explains the architecture to users.

## 1. Baseline, scope and fixed settings

Start from released v0.1.4 commit `101fa273d815f3aaedb0e06ba0de7b0777d83def`,
qualified product `9cfb4be477116646258ea0621280ed13b1824c6d`. Preserve other tasks'
changes and the released #95/#98 behavior described in section 8.

| Setting | Fixed value |
| --- | --- |
| Small regular-file content | 1..131071 raw bytes |
| Large regular-file content | >=131072 raw bytes |
| Empty regular file | Existing compact empty extent/file-state representation |
| Large-file CDC profile | 8192 minimum / 16384 target / 32768 maximum bytes |
| New SQLite pages | 4096 bytes; supported existing 65536-byte layouts retained |
| New small codec | Existing Zstandard, level 3, windowLog 18, workers 0 |
| Frame flags | Content size and checksum enabled; dictionary ID disabled |
| New delta depth | One edge to a FULL small-content base |
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
u8 kind                 0 = FULL, 1 = DELTA
u32 raw_length          1..131071
u32 frame_length        1..135168
[32 bytes base ObjectId] DELTA only
frame_length bytes      exactly one Zstandard frame
```

FULL frames decode without a dictionary. DELTA frames decode using the raw bytes
of their named FULL SmallContent base. The reconstructed canonical object uses
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

Only the already known predecessor's selected representation supplies a candidate:

- Predecessor SmallContent stored as v3 FULL: use it as the sole candidate base.
- Predecessor SmallContent stored as v3 DELTA: use its directly named FULL base;
  validate that the selected base really is v3 FULL SmallContent.
- Chunked, empty, unavailable or otherwise ineligible predecessor: emit FULL.

An optional predecessor hint that cannot establish eligibility may fall back to
FULL. Once a persisted DELTA record is selected, its named base is a required
dependency: missing, corrupt, wrong-role or non-FULL base is an integrity error,
not a reason to silently fall back. Apply that rule to reads, exact collision
comparison and candidate-anchor reuse.

No predecessor decoding just to find a base ID, no multi-part chunk anchor, no
recursive ancestry search, and no new base copy to create an artificial candidate.
A dependent base must already have a stable selected location or be selected
before its dependent under the same admission ownership. Never rewrite a globally
selected object to make it FULL. A locator claiming FULL must be verified through
the real selected record; ObjectId alone does not prove its encoding or integrity.

Reuse scoped predecessor facts from the existing workspace provenance/batched
lookup, including supported replace-by-rename relations. Do not infer relationships
through a new global path/similarity index or add a per-file transport negotiation.
If those facts do not provide an eligible base, FULL is the complete fallback.

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

Read small FULL directly or reconstruct small DELTA with its FULL base, then
validate the complete canonical object at the storage trust boundary. A short
range can require decoding the bounded complete object. Batch and deduplicate
base acquisition within valid read ownership; cache reuse never substitutes for
initial authentication. New-format dependency depth is one. Legacy v2 PREFIX
read support and its existing depth rules remain unchanged.

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

For small reconstruction, reserve at most 2 MiB per active owned decoder: 1 MiB
static decoder workspace and 1 MiB combined operands/output. Validate static
estimates against the fixed pinned codec configuration in the implementation;
reset borrowed references on success and failure. A failed budget check is an
implementation error to solve through reuse/lifetime/allocation structure, not
permission for a setting sweep or hidden heap fallback.

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

New Stores use schema 8 with the same seven SQL tables and 4-KiB pages. Schema 8
fences both the new canonical role and pack grammar. New binaries continue to
read supported schema 6/7 data using the old encodings. Opening old Stores does
not silently promote them; writes in those schemas stay on their supported old
construction policy. Expose format capability to the shared builder once per
operation, not through a database query for every file.

Implement explicit offline `LayerStackStore::upgrade_format(path) -> Result<()>`
for schema 7 to 8. Read-only preflight checks compatibility/metadata, then exclusive
access revalidates before changing only the format version. Use a real SQLite
transaction with DELETE journal and FULL synchronous during the upgrade; require
no live owners, do not rewrite payload/history or change page size. Restore the
normal Store connection profile when reopening. Failure before COMMIT leaves
schema 7; failure after COMMIT must report that promotion occurred. Schema 8 is
an idempotent no-op; unsupported schema 5 and schema-6 upgrade requests return
clear errors. Existing schema-6 compatibility behavior is otherwise retained.

Old binaries reject schema 8 before mutation. There is no header-only downgrade
once new objects may exist. Reverting requires a pre-upgrade backup, not changing
a version integer. This task does not add a history-export service or silently
claim compatibility with released v0.1.3's unsupported schema 5.

The owner-authorized canonical/Store evolution is a scoped v0.1.5 exception in
[release policy](../../../general/release-policy.md). Ordinary write/fsync/Commit
acknowledgements do not gain crash/power-loss durability; MEMORY journal and
synchronous-OFF normal operation remain their documented contract.

## 10. Implementation completion and verification scope

Implement all paths above, compile the required product/harness targets, and run
only `small_file_delta_smoke / small-file-delta-10x30-v1` as specified in
[delta-encoding benchmarks](delta-encoding-benchmarks.md#first-round-execution-scope).
Its integrated verification reopens the same retained Store and checks every byte,
path, mode and length in all 31 states. Runtime parser/budget checks are production
requirements even when malformed cases are not exercised by this smoke.

Do not run Cargo test/Clippy/doctest suites, separate codec/compatibility/failure
suites, old three-file reruns, SDK/FUSE matrices, 56-case campaigns or full157 in
this round. Do not repeatedly run a passed smoke on an unchanged relevant build.
Keep a matching baseline, source-bound observations, actual FULL/DELTA selection,
storage/latency changes, and a list of unexercised paths. A path silently falling
back to old small chunking does not count as implementation completion.

The smoke is local exploratory evidence, not release admission. Broader tests and
benchmarks remain later qualification; do not publish/tag a release or represent
smoke-only verification as exhaustive compatibility/POSIX/corruption assurance.

## Source anchors

[Canonical roles and codec](https://github.com/Ephemeral-AI-Lab/layerfs/tree/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-content/src/object),
[existing file formats](https://github.com/Ephemeral-AI-Lab/layerfs/tree/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-content/src/file),
[shared output/admission](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-layerstack-store/src/objects.rs),
[Workspace producer](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-workspace/src/changes.rs),
[pack grammar](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/crates/layerfs-layerstack-store/src/objects/pack.rs),
[released storage contract](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/docs/versioned/0.1.4/storage-format.md).

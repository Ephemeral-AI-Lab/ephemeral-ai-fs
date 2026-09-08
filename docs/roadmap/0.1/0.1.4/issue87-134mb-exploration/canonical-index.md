# Canonical structure, metadata DELTAs, and index exploration

**Metadata/index elimination with the current payload representation cannot reach 134,221,004 allocated bytes by itself, but structural compression is a material second arm.** The authenticated group inventory partitions exactly into **216,448,341 encoded payload bytes** and **78,792,537 encoded structural bytes**. No group mixes payload with structure. Even the impossible deletion of all structural storage, every index, every pack header and all filesystem overhead leaves the payload bodies 82,227,337 bytes over the goal.

This review expands the candidate space to physical structural DELTAs, versioned canonical forms and a different history representation. It does not implement them, encode retained data, replay a workload, or open the original Store. `canonical-metadata.py` reads only previously authenticated CSVs and the immutable analysis index, authenticates their seals, and emits `canonical-metadata.json` in the new exploration directory. Source checkpoint: `91a830766ce117e8c3a0cc305129e44762bdb052`.

## Exact budget and coupled physical buckets

| Existing quantity | Integer bytes / count | Interpretation |
|---|---:|---|
|Acknowledgement allocation|335,552,512 bytes|Primary pre-verification measurement|
|60% reduction target|134,221,004 bytes|Integer floor of 40% of baseline|
|Required allocation reduction|201,331,508 bytes|Derived objective, not a prediction|
|Payload-only groups|29,156 /216,448,341 encoded bytes|Exact exclusive physical group population|
|Structural-only groups|9,918 /78,792,537 encoded bytes|Exact exclusive physical group population|
|Structural canonical objects|279,724 /94,362,335 canonical bytes|Logical reconstruction dimension, not additional disk bytes|
|Objects index|18,837,504 page bytes|Post-verification; already WITHOUT ROWID|
|SQLite logical bytes outside pack BLOBs|22,924,802 bytes|Includes the objects index; do not add both|
|Ack signed allocation adjustment|+16,719,872 bytes|Mechanism unresolved; not a promised reclaimable pool|

The post-verification page layout cannot be spliced into an exact acknowledgement-time decomposition. These quantities constrain candidates, but only a new complete allocation measurement could establish combined savings.

Structural subfamilies share compressed groups. Their exact **whole-group envelopes** are:

| Target family | Groups containing only that family | Groups shared with other structural roles | Combined encoded envelope |
|---|---:|---:|---:|
|Inode-table leaf/branch|86 /889,106 B|5,655 /68,706,890 B|69,595,996 B|
|FileState plus extent nodes|3,880 /7,570,025 B|157 /1,566,894 B|9,136,919 B|
|Inode records|21 /6,504 B|5,745 /68,941,487 B|68,947,991 B|

The inode-table and inode-record envelopes substantially overlap. They cannot be added, and an envelope is not measured per-role compressed attribution. The structural union remains 78,792,537 bytes.

## 1. First structural experiment: physical DELTAs for inode-table leaves

**Material target; canonical break is unnecessary for the first experiment.** Current inode-table leaves contain 880,418 entries. Each canonical entry is a 32-byte InodeId plus a 32-byte inode-record ObjectId. Across 9,890 leaves, the exact canonical account is:

```text
56,781,912 = 9,890 ×44 header bytes +880,418 ×64 entry bytes
```

The metadata graph shows that **790,456 of those record references already existed at an earlier retained checkpoint**; 89,962 first appeared at the leaf's checkpoint. Consequently 25,294,592 bytes of record-ID fields alone refer to older records. This is concrete copied-structure evidence: approximately 89.8% of record-reference occurrences in new leaves point to older data. It does not prove that one particular predecessor leaf contains every repeated entry, nor that all those bytes can be removed after compression. The other 32-byte half of each entry is the inode identity; uniqueness/repetition of these keys was not retained in the existing census and is not invented here.

The physical reader already accepts generic canonical DELTAs. `objects/pack.rs::Record::Delta`, `apply_delta`, and `objects/read.rs::finish_deltas` reconstruct arbitrary bounded canonical bytes and authenticate their expected ObjectId. The current writer's `DeltaSearch::candidate` explicitly accepts only validated chunk payloads, and checks the base for that same role. Structural FULL-only behavior is therefore writer policy, not an inherent wire limitation.

There is also an existing origin mechanism to reuse rather than a new global similarity search. `tree/batch.rs` stores `Page.origin`, propagates it into `Node.id`, sets it to the prior node in `Engine::edit`, and carries it through pending pages. `Engine::persist` encodes the final page and currently calls ordinary `put_owned(canonical)` after checking exact reuse. An unchanged-partition leaf therefore already has an immutable prior-node candidate at the point where a physical hint could be emitted. Splits and merges may clear origin; they can safely remain FULL in the first experiment. There is no need to add key-range/global lookup before this existing origin path has been measured.

**Smallest implementation boundary, prospectively:** retain a validated optional origin hint through canonical admission for exact inode-table leaves, allow this one structural role in candidate selection, reuse the current FULL-anchor reader, depth-one record format, matcher and A/B group threshold. Ignore an unavailable or not-yet-admitted origin as the existing bounded fallback requires. Do not treat every `Node.id` as a guaranteed selected FULL anchor: origins can be absent or can reference a representation that needs the normal anchor resolution. Keep payload policy unchanged during that isolated structural experiment.

**Affected bytes:** at most the 69,595,996-byte inode-table group envelope, with only 889,106 bytes belonging to inode-table-only groups. **Credible removable portion:** unknown until paired A/B bytes on the selected origins are measured. The old-reference population makes substantial reduction plausible; it does not establish a percentage. Measure origin-present count/bytes, authenticated selected FULL bases, completed candidates, mixed acceptance, final selected bytes and all new physical-base closure.

**Costs and compatibility:** additional bounded base reads, matching and encoding during Commit; additional base read/decode on structural lookups. File lookup traverses inode metadata, so read amplification matters even when byte savings are good. Depth one and existing object/group bounds contain memory, but CPU and foreground time still need measurement. Canonical IDs and wire framing can stay identical; existing generic readers must nevertheless receive an explicit structural-DELTA compatibility test. New anchors may be physical-base-only even though baseline anchors were all logically retained. No product concept is deleted in this first arm; it reuses the existing physical DELTA machinery and avoids introducing a second encoder. A larger canonical redesign should not be a prerequisite for discovering whether this arm works.

## 2. If origin DELTAs are insufficient: dictionary or recipe-based structure

### Typed physical reference dictionary

Repeated 32-byte inode keys and ObjectId fields can be represented by compact physical ordinals, with full identifiers held once in an authenticated dictionary. This differs from truncating identity: the decoder reconstructs the original canonical bytes and verifies the original ObjectId. It can preserve the logical format while replacing repeated high-entropy identifiers in physical metadata. Git provides a primary-source precedent for a physical representation omitting canonical material and reconstructing it for hashing; its pack representation omits the loose-object type/size prefix while retaining the canonical object identity. This is a mechanism analogy, not a measured LayerFS saving. [Git pack format](https://git-scm.com/docs/gitformat-pack#_object_encoding)

The exact old-reference counts give this idea a real population, but the existing census does not retain all inode-key values or a same-base mapping. A store-wide dictionary also costs space and becomes a required authenticated physical dependency. A pack-local dictionary avoids global publication coupling but misses repetition across historical packs. Choosing the scope is a different experiment from structural DELTAs, not an extra switch in the same candidate.

**Boundary:** one typed structural physical codec with strict dictionary bounds, exact canonical reconstruction and normal authentication. **Costs:** dictionary lookup/read/cache pressure, atomic dictionary publication, retention accounting and corruption checks. It adds a mechanism rather than deleting code. It should beat the existing origin-DELTA approach on complete allocated footprint before adoption. Its affected envelope overlaps the full structural bucket; do not add dictionary savings to DELTA savings on the same bytes.

### Versioned snapshot recipes / compact namespace representation

A more aggressive canonical architecture would retain a periodic materialized namespace plus sorted per-checkpoint inode/path changes instead of persisting every historical COW node version. Existing final-state delta streams provide a natural construction source. This could remove much of the repeated 880,418-entry inode-table history rather than merely encoding it better.

This is a genuine architecture break: roots identify recipe/checkpoint structures; historical lookup may need to apply a bounded chain or consult a materialized cache. A generic `read_object(ObjectId)` API over all old structural nodes no longer has the same meaning. Branching, reconciliation, corruption detection, crash-consistent publication and cache limits need an explicit replacement. Preserve all157 observable states, executable/mode/mtime/symlink metadata, inode/hardlink semantics and supported operations; matching Git's smaller semantic surface is not an acceptable shortcut.

**Potential concepts removed:** per-checkpoint persistent inode-table/directory-map node publication and possibly a separate canonical object/index row for each structural wrapper. **Concepts added:** bounded recipe replay/materialization, checkpoint selection, root authentication and historical-cache policy. Compact path entries can eliminate global inode indirection only if the design still specifies hardlink identity and efficient updates; embedding the same inode state into every alias shifts work to all linked directories.

**Scale:** the entire existing structural group population is 78,792,537 encoded bytes, plus any index rows actually eliminated. This is material but still cannot reach134.2MB without a large payload change. The reference chronology alone cannot reconstruct exact recipe-operation count: an older record can be installed under a new inode key, and deletions have no new record. Any numerical recipe size is therefore presently unknown. A future bounded prototype must serialize the actual required change semantics and verify every retained state, not extrapolate from 89,962 new-reference events as if they were all mutations.

## 3. FileState/extent fusion and small inline files: useful simplification, smaller ceiling

There are exactly **75,927 FileStates and75,927 distinct mapping roots**, all leaves; no extent branch occurs in this workload. Of the leaves, 64,754 have one extent, one is empty, and all leaves together have 99,908 extent entries. This is a concrete redundant-wrapper opportunity, not a reason to discard support for large/range-edited files generally.

A versioned fused leaf-file object can place the extent list directly in the file-state object. An illustrative new canonical layout with a26-byte header and40-byte extent entries would change this workload's FileState-plus-leaf canonical bytes from15,385,370 to5,970,422, a **9,414,948-byte raw difference**, and remove75,927 selected objects. The header arithmetic is an explicitly hypothetical format; it is not encoded or allocated savings. Its complete current group envelope is only9,136,919 encoded bytes, and some of that belongs to other roles.

**Boundary:** one new file-content variant carrying a leaf list inline, with a branch-root form retained for larger files. The version/profile can make currently repeated profile identity implicit. **Concepts deleted:** the mandatory FileState→leaf lookup and separate leaf object for the leaf case. Large-file extent trees still exist; a universal wrapper cleanup should not be advertised as deletion of the whole rope subsystem. Canonical ObjectIds and enclosing roots change, requiring a new canonical profile and a declared compatibility/migration strategy. Reads lose one metadata lookup; writes build fewer objects. Spill/memory and partial-extent validation still require explicit bounds.

Small payload inlining is a related alternative, not another additive saving. Conservatively selecting one-extent states whose entire backing payload is within a threshold gives:

| Backing payload bound | FileStates | Current state+leaf canonical bytes | Backing canonical bytes across these uses |
|---|---:|---:|---:|
|256 B|864|164,160|144,472|
|1,024 B|15,992|3,038,480|8,531,459|
|4,096 B|33,537|6,372,030|50,270,807|

A slice cannot exceed its backing payload, so these are safe candidate cohorts even though source offsets were not retained in the metadata inventory. They are not complete file-size histograms. The underlying payload bytes still need storage after inlining. Other files, metadata values or DELTA base dependencies may retain the old chunk objects, so one must recompute logical and physical-base closure before claiming their index rows disappear. The 256-byte cohort is plainly too small to lead a201MB reduction effort.

Inlining full inode records into every inode-table entry also deserves skepticism. Current leaf entries store one32-byte record ID; the record itself is98 canonical bytes and shared. Replacing that reference with a73-byte bare value (kind, refcount, content root, metadata root) adds41 bytes per leaf entry before compression, roughly36.1MB across the current entry population, while removing only8,778,448 canonical inode-record bytes. The exact new partitioning would change as entries grow. Better compression of defaults might reverse the result, but “one less indirection” is not proof of less storage. The existing reference-based form or physical structural DELTAs may be cheaper.

## 4. Dense index / pack placement is a finishing step, not the main architecture

The `objects` table already uses `WITHOUT ROWID`, so adding that keyword cannot deliver SQLite's often quoted two-B-tree-to-one improvement. SQLite documents that the existing layout stores the primary key once; LayerFS already takes this path. The separate `object_packs` integer-primary-key table supports incremental BLOB I/O, which SQLite does not provide for WITHOUT ROWID tables. Blindly converting it would lose an existing read mechanism. [SQLite WITHOUT ROWID](https://www.sqlite.org/withoutrowid.html)

A hypothetical dense sorted index with full32-byte ObjectIds and8 compact locator/length bytes would occupy14,645,640 bytes for366,141 rows, versus18,837,504 existing index page bytes: a4,191,864-byte difference **before** fanout, checksums, mutable tail and publication costs. The40-byte model fits this run's compact locator domains; it is not a universal maximum-size Store layout. Merely retaining each full identifier once already costs11,716,512 bytes, so eliminating every other current index byte would save only7,120,992 bytes before replacement lookup structures.

Git's index format demonstrates sorted full object names, a fanout table, and physical offsets. A LayerFS design must additionally handle synchronous fresh admission, selected locators, collision checks, and concurrent readers; a periodically rebuilt static index is not equivalent to the current transaction path. [Git index format](https://git-scm.com/docs/gitformat-pack#_version_2_pack_idx_files_support_packs_larger_than_4_gib_and)

Using direct physical ordinals inside canonical parents would make identity depend on placement and make relocation propagate through ancestors. Prefer a physical representation/dictionary if canonical identity should survive repacking. A short-fingerprint lookup index could verify full identity after reading collision candidates, but it needs real collision handling and strict amplification bounds; it is not justified simply by discarding half the digest bytes.

Moving pack BLOBs out of SQLite has only3,964,418 current logical bytes between object_packs pages and actual pack BLOB bytes. It would introduce file/index publication, rollback and orphan-cleanup responsibilities presently handled by one SQLite transaction. The signed filesystem allocation adjustment is a separate unresolved mechanism, not guaranteed extra savings. SQLite's page/overflow representation explains why table pages and BLOB bytes differ; it does not imply that all overhead is reclaimable. [SQLite database file format](https://www.sqlite.org/fileformat.html)

## Selection and next evidence

For the metadata arm, rank **origin-hinted inode-leaf physical DELTAs first**, an authenticated structural dictionary or recipe architecture second if the first ceiling is insufficient, and FileState fusion/index compacting as smaller simplifications. These are alternatives over overlapping bytes, not a201MB sum.

Before any claim that134.2MB is reachable, combine measured output from an isolated payload arm with measured structural output and one complete Store allocation. A strong payload result with little metadata improvement may still miss the target; perfect metadata elimination alone certainly misses it. The smallest new structural evidence is paired unchanged-canonical leaf A/B encoding on actual origins, then synchronous public-path correctness/read/foreground/resource checks. No such encoding or product experiment was executed in this review.

The accompanying metadata tool verifies group/role conservation, exact format-derived leaf counts, FileState/mapping one-to-one ownership, and source hashes. Its query uses existing analysis indexes and an explicit bounded join order; no original database, payload census or compression path is involved. A first metadata-query plan was interrupted because SQLite chose a poor role-only join, then replaced by joins using the existing ID indexes; no evidence file was changed.

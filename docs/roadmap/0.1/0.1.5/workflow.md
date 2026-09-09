# v0.1.5: workspace storage and history workflows

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

Your workspace behaves like a normal filesystem. Applications and coding agents
read and edit files; a successful commit saves a workspace version. Small files
share storage through whole-file deduplication and delta encoding. Large files
share chunks and unchanged ranges. A file can change storage representation as
it grows or shrinks without making its retained earlier versions unreadable.

## 1. Components and the overall architecture

| Component | What it does for the user |
| --- | --- |
| FUSE interface / SDK operations | Deliver filesystem operations and explicit edits to the same live state owner. |
| Live workspace state (FUSE + SDK) | Owns the currently editable files, directories, inodes, and open-handle lifetimes. |
| Pending changes | Tracks edits within the live workspace and retains their bytes/backing references before a commit. |
| Capture pending changes | Freezes/drains a consistent set of live facts for the shared Commit pipeline. |
| Shared Init/Commit pipeline | Prepares file content and namespace metadata, stores required objects, and publishes a version. |
| Small-file storage | Gives each distinct complete content one CAS object, physically stored as FULL or DELTA. |
| Large-file storage | Maps file ranges to immutable chunks through an extent tree. |
| Object store | Holds encoded objects in packs; SQLite indexes their locations. |
| Version roots and history | Identify the files and metadata belonging to each retained version. |

CAS means content-addressed storage: an object's identity is derived from its
canonical type/framing and complete content. The same small-file content has the
same identity whether physically stored as FULL or DELTA. Names, permissions,
and other inode metadata are represented separately from the file payload.

```text
 Native directory                         Applications / coding agents
       |                                               |
 Namespace Init                              FUSE operations / SDK edits
       |                                               |
 Discover source files                         Live workspace state
 and metadata                                    (FUSE + SDK)
       |                                               |
       |                                         Pending changes
       |                                               |
       |                                         Workspace Commit
       |                                               |
       |                                      Capture pending changes
       |                                      (freeze / drain facts)
       |                                               |
       +-----------------------+-----------------------+
                               |
                  ONE SHARED INIT/COMMIT PIPELINE
                               |
                 Prepare content + namespace objects
                               |
                  +------------+-------------+
                  |                          |
             Below 128 KiB              128 KiB or above
                  |                          |
          Whole-file CAS object       CDC chunks + extent tree
                  |                          |
             FULL or DELTA            Reuse unchanged ranges
                  |                          |
                  +------------+-------------+
                               |
                   Shared admission and packing
                               |
                     Packs + SQLite locators
                               |
                  Publish the prepared version root
                               |
                    Retained workspace versions
```

The cutoff is **131,072 bytes**, independent of line count and SQLite page size.
Empty files keep a compact empty representation and need no delta payload.

## 2. Namespace Init and Commit: one pipeline, different inputs

**Init and Commit must use the same core pipeline.** Init supplies a discovered
namespace and its source content. Commit supplies a captured workspace state,
its predecessor, and known changes. Input acquisition differs because one reads
a source directory and the other captures a live workspace. It must not create
two implementations of content construction, delta selection, authentication,
batching, or publication.

```text
 Init input                              Commit input
 --------------------------              --------------------------
 Source files + metadata                 Frozen file/namespace facts
 No predecessor for new import           Previous root + dirty frontier
                \                        /
                 \                      /
                  v                    v
              +----------------------------------+
              | 1. Prepare the resulting state   |
              |    Reuse unchanged references    |
              +----------------+-----------------+
                               |
              +----------------v-----------------+
              | 2. Build new canonical objects   |
              |    Apply the same size policy    |
              +----------------+-----------------+
                               |
              +----------------v-----------------+
              | 3. Resolve CAS reuse and choose  |
              |    physical FULL/DELTA encoding  |
              +----------------+-----------------+
                               |
              +----------------v-----------------+
              | 4. Admit objects in bounded      |
              |    batches; make bases available |
              +----------------+-----------------+
                               |
              +----------------v-----------------+
              | 5. Complete namespace metadata   |
              |    and prepare publication      |
              +----------------+-----------------+
                               |
              +----------------v-----------------+
              | 6. Publish the version under     |
              |    its lifecycle preconditions   |
              +----------------------------------+
```

These are logical stages; implementations can stream bounded batches through
them. They do not require holding every file in memory or putting the entire
operation into one SQLite transaction. Initial creation and advancing an existing
workspace have different publication preconditions, enforced through the shared
publication machinery. Identical pipeline does not mean identical source reads
or identical work for a new namespace and a one-file change.

An import normally has no related predecessor for delta selection, so unique
small content starts FULL. Exact duplicates can still share CAS objects. A commit
can use its already known predecessor context for bounded delta selection; it
must not search all historical commits for a better base.

## 3. From a live edit to saved history

**FUSE is the filesystem interface; the live workspace owns the editable state.**
It is more than a change-capture buffer: it maintains current files and directory
bindings, inode and open-handle lifetimes, pending bytes, and references to
unchanged saved content. SDK edits route through that same owner so both access
paths remain consistent, including the required kernel-cache reconciliation.

```text
 Application / coding agent                    SDK edit
              |                                   |
 open / read / write / truncate / rename           |
              |                                   |
             FUSE                                 |
              |                                   |
              +----------------+------------------+
                               |
                               v
              +-----------------------------------+
              | LIVE WORKSPACE STATE (FUSE + SDK)  |
              | Current files, directories, inodes|
              | Open handles and file lifetimes   |
              | Pending edits + backing references|
              | Unchanged saved-content references|
              +----------------+------------------+
                               |
                  Commit requested: capture a
                  consistent set of pending facts
                               |
                               v
                    Shared Init/Commit pipeline
                               |
                   Store objects + publish root
                               |
                               v
                      Saved workspace version
```

For a one-line write, the live owner applies the edit and retains its new bytes;
subsequent reads through the supported live paths see the updated file before
Commit. Changes are tracked as operations are applied, rather than rediscovered
later by watching or rescanning the directory. Capture is the separate Commit
step that freezes/drains these facts for canonical construction and storage.
Namespace Init supplies discovered source files directly to the same pipeline;
it does not need to replay the import through FUSE or a live workspace first.

A write does not create a historical commit or a database transaction per syscall.
Pending file pieces can reference old content, new bytes, spool storage, or zeros.
At commit, small changed files are assembled into bounded complete content;
large files retain range sharing where known edits permit it. Unchanged files
keep their existing references rather than being reread or converted.

Storage policy does not change pathname, hard-link, open-handle, rename, or
read-after-write semantics. Files sharing a content object remain independently
editable unless they are actual hard links. Changes through one hard link remain
changes to the shared inode, as before.

Before publication succeeds, preparation must not expose a partially published
version. A failed preparation does not replace the previous published root;
cleanup follows the existing lifecycle. Publication, live-write acknowledgement,
`fsync`, and crash/power-loss durability remain distinct. This implementation does not
strengthen v0.1.4's durability contract.

## 4. Small files: whole-content identity, compact physical differences

Each distinct small-file content is **one logical CAS payload object**. Small
files do not first become CDC members in this representation.

```text
 Logical history                   Physical storage in schema-9 packs
 ---------------------             ---------------------------------
 Commit 1 -> object A ------------> A: FULL complete content
 Commit 2 -> object B ------------> B: DELTA(A -> B)
 Commit 3 -> object C ------------> C: DELTA(B -> C)
                                   C -> B -> A is the authenticated closure
 Commit 4 -> object B ------------> Reuse existing B; no new file payload
 Commit 5 -> object D ------------> D: FULL when bounds/cost require fallback
 Commit 6 -> object E ------------> E: DELTA(D -> E)
```

B identifies the complete reconstructed B content, not the patch bytes. A reader
gets B by decoding its delta using A, then validating the resulting canonical
object. The encoding can reuse matching byte sequences from the base; it does
not require storing a new 8-32 KiB chunk for a one-line change.

New schema-9 kind-2 deltas may follow the already selected immediate small
predecessor through at most **8 dependency edges**. Complete closure, including
the target, is also bounded to **512 KiB canonical bytes** and **256 KiB retained
encoded capacity**. Each decoded node is authenticated before becoming the next
base. If eligibility or any prospective bound fails, or complete DELTA cost is
not strictly smaller, the prepared FULL wins.

Kind 1 keeps its original one-edge FULL-only meaning. Existing schema-8 Stores
remain on that writer policy and are not promoted on open. Kind 2 is rejected
there; explicit offline upgrade to schema 9 is required. Old native/legacy formats
retain their own decoding rules. When no eligible predecessor exists, schema 9
can select one actual FULL from its bounded fingerprint cache and emit kind 1.
Retained admissions move the same cache to the next admission on the same Store;
rollback discards it and reopen starts empty. Frozen removed-name facts can also
supply a unique physical hint without changing the real inode or metadata. The older recent-128 ring was diagnostic only; logical CDC predecessor
reuse remains unimplemented.

Exact deduplication and delta encoding provide different benefits:

- Returning to identical content reuses its CAS object.
- Similar new content gets a new identity but may require only a compact delta.
- Unrelated new content uses FULL without forcing an expensive history search.

Each commit still has namespace/history metadata costs. Reusing a file payload
does not mean the entire commit consumes zero additional storage.

### Length-changing edits and shifted content

Insertions, deletions, and unequal-length replacements are first-class small-file
operations. Delta matching must reuse content at different offsets; it does not
require stable byte/line counts or a fixed number of changed regions.

```text
 Base A:   [ prefix ][ old region ][ suffix ]
 Target B: [ prefix ][ longer replacement ][ suffix ]
                                              |
                               Same suffix at a different offset

 B remains one whole-file CAS object.
 Its delta can reference matching prefix/suffix bytes in FULL base A.
```

This illustrates matching, not a literal delta wire format. Actual savings depend
on base similarity and codec settings, including the matching window. Target
content is still assembled/hashed below 128 KiB; small explicit edits can therefore
read unchanged bytes that the large-file extent path would simply retain.
Deletion, truncation, and zero extension may already be cheaper through extents;
we must measure their tradeoffs rather than promise a win for every operation.

File length and CDC chunk count are separate quantities. An equal-length edit
can change CDC chunk count in a chunked file. New small-file objects have no CDC
member list, so the corresponding payload changes require no small-file extent
count maintenance. The existing large-file count and locality contracts remain.

See [delta-encoding benchmarks](delta-encoding-benchmarks.md) for the three released
edit families, their small-file equivalents, and explicit complexity targets.
The frozen tiny-history case has only same-length edits and does not prove this
length-changing behavior.

## 5. Large files: share unchanged content and ranges

Large files use CDC chunks and an extent tree. An extent describes which slice
of an immutable payload supplies a logical file range. The persistent tree lets
versions share content without copying the complete file.

```text
 Earlier version:  [ A ][ B ][ C ][ D ][ E ]
                     |    |         |    |
                     |    |         |    |     Shared existing content
                     v    v         v    v
 New version:      [ A ][ B ][ X ][ D ][ E ]
                               ^
                               |
                        New changed content
```

This is a simplified illustration: an edit may split extents or produce several
chunks. The existing CDC profile uses an 8 KiB minimum, 16 KiB target, and 32 KiB
maximum, with a possible shorter final fragment. Those are chunk sizes, not a
fixed charge per edit. Existing supported pack compression remains available;
v0.1.5 does not require new whole-file deltas for large files or their chunks.

Known range edits can retain unchanged references and process replacement bytes.
A full-file overwrite may require reading and chunking the supplied stream to
establish reuse. Delta storage cannot turn an application's 100 MiB rewrite into
a one-line input operation. Append-only growth can retain earlier extents, but
new content and mapping/history metadata still require storage.

## 6. Growing and shrinking across the cutoff

The final size chooses the representation when preparing **new or changed** file
content. A mode belongs to a saved version; it is not a permanent property of a
pathname. Readers follow the recorded representation, including older chunked
small-file versions. Untouched history is never converted just to match policy.

```text
 Commit 1        Commit 2        Commit 3        Commit 4
 40 KiB          150 KiB         20 KiB          180 KiB
    |               |              |               |
 Small A         Chunked B       Small C         Chunked D
 FULL/delta      extent tree     FULL/delta      extent tree
    |               |              |               |
    +---------------+--------------+---------------+
                  All retained versions readable
```

| Transition | Work required for the new version |
| --- | --- |
| Small to small | Assemble/hash the bounded result; reuse CAS or choose FULL/DELTA. |
| Small to large | Produce the new chunked representation from the resulting content; reuse matching existing chunks. |
| Large to small | Read the resulting small content and necessary dependencies; create/reuse its whole-file object. Do not reconstruct the discarded large content. |
| Large to large | Preserve unchanged extents where the captured edits permit it. |

For example, truncating 200 MiB to 20 KiB needs the retained 20 KiB and its
necessary chunk/decode dependencies, not a scan of 200 MiB or its history. Without
an eligible SmallContent predecessor already available, the new small object starts FULL.
Later small versions can use deltas.

```text
 Live sizes before one commit: 127 -> 130 -> 126 -> 140 -> 120 KiB
                                                             |
                                                           Commit
                                                             |
                                           Prepare SMALL representation once
```

Repeated crossings across separate commits do incur conversion work. Previously
stored identical small objects or chunks can be reused, but overlapping bytes in
the two representations do not automatically share physical storage. The fixed
128 KiB rule has no hysteresis or threshold tuning in this version.

## 7. Reading current and historical versions

```text
 Select a saved version -> resolve path/inode -> recorded file representation
                                                        |
                                +-----------------------+------------------+
                                |                                          |
                           Small object                               Chunked file
                                |                                          |
                        +-------+-------+                         Locate extents for
                        |               |                         the requested range
                       FULL            DELTA                               |
                        |               |                         Read needed chunks
                        |        Read bounded base closure                 |
                        |        + reconstruct target                      |
                        +-------+-------+----------------------------------+
                                |
                   Validate objects at the storage trust boundary
                                |
                         Ordinary file bytes
```

A small delta read may reconstruct the bounded complete object even for a short
requested range. A large-file range read acquires the needed extents and payload
dependencies. Neither requires replaying every intervening commit. Live reads
also combine immutable base ranges with pending edits before a commit exists.

Retained versions keep their content references. Physical delta bases must also
remain available while dependent objects are retained, even if a base is no
longer directly referenced by a visible file. Missing or corrupt bases are errors,
not permission to return unchecked bytes. This is a retention requirement, not a
claim that a new garbage collector or repacker already exists.

## 8. Keeping the shared pipeline fast

The same rules apply to Init and Commit:

- Carry prepared content, namespace facts, and verification ownership forward;
  do not reread and reauthenticate them at every internal handoff.
- Verify persisted/untrusted content at the established boundary. Reuse already
  authenticated operands within their valid bounded ownership scope.
- Batch membership checks, object acquisition, pack writes, and SQL work. A file,
  chunk, delta, or syscall must not automatically mean another transaction.
- Keep compression and reconstruction bounded and outside the shared Store lock
  where ownership permits. Do not introduce transport round trips per object.
- Reuse predecessor context and unchanged namespace references. Avoid workspace
  rescans and history searches when the operation already supplies the facts.

Shared code still respects operation-specific resource budgets and lifecycle
rules; it does not erase the qualified Init and Commit optimizations. SQLite
creation stays at 4 KiB pages, with supported existing 64 KiB layouts readable.

New Stores retain 4-KiB pages and the same seven tables under schema 9. Supported
schema 6/7/8 opens do not promote; explicit offline 7/8→9 upgrade revalidates with
exclusive access and transactional DELETE/FULL promotion, without rewriting old
payloads. Rollback to an old binary requires a pre-upgrade backup.

The active reconstruction allowance stays 2 MiB, including static decoder and
dictionary storage, retained encoded records, four simultaneous base/raw/framing/
canonical buffers and any surviving admission target. Admission encoding stays
3 MiB including its actual 2-MiB static encoder and bounded operands/output.
Decoder and encoder do not overlap; neither may fall back to hidden heap growth.
Existing producer, index and read-wave ownership budgets remain authoritative.


The earlier schema-9 chain-only measurement reduces ten-snapshot allocation from the existing
v0.1.5 baseline's 66,105,344 B to **56,668,160 B**. This remains **11,668,160 B above**
the 45,000,000-B target, not near-target or a release PASS. The same measured Store
passed exhaustive historical verification and cleanup. See the
[chain-1 report](issue100/chain-1-results.md) for exact timing, resource and custody
scopes rather than transferring claims from an isolated codec diagnostic.

The current [ten-snapshot contract](issue100/ten-snapshot-contract.md) retains ten
complete selected snapshots, not ten consecutive upstream commits. Original
[tiny-case](tiny-history-baseline-v1.md) and [31-state smoke](smoke-report.md)
observations keep their separate fixtures and historical thresholds. The initial
full157 regression remains recorded; the subsequently owner-authorized
[full157 confirmation](issue100/retained-full157-results.md) is complete with
157 same-Store historical proofs and disclosed latency regressions. Its target is not 45 MB. Broader release qualification
is unrun, and the current target miss does not rule out all authorized bounded
designs.

For released behavior, consult [existing architecture](existing_architecture.md).
For performance lessons, read [past mistakes](past_mistake.md). The
[spec](spec.md) and [implementation plan](implementation_plan.md) define the
whole-file canonical object, compatibility, small-file partial edits and shared
Init/Commit implementation. See their fixed parameters; do not run tuning sweeps.

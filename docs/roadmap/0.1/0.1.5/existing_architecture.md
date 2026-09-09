# Existing architecture and the paths v0.1.5 must preserve

> **Status:** Source-bound inventory, 2026-09-09. This describes existing implementation,
> not a new architecture, a universal POSIX certification, or a release claim.

Read alongside [hybrid_mental_model.md](hybrid_mental_model.md),
[benchmark_success.md](benchmark_success.md), and [past_mistake.md](past_mistake.md).

## Release update before implementation

The structural inventory below is pinned to the historical R26 source. The
implementation baseline is now released v0.1.4 commit `101fa273d815f3aaedb0e06ba0de7b0777d83def`.
The [spec](spec.md#8-released-speed-invariants) supersedes
old admission details: Init has bounded authenticated comparison reuse; Workspace
also coalesces SQL transactions and closes them before staging; ordered spill
visits use 64-KiB read-ahead while sequential scanning remains separate.
See the [qualified #98 report](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/docs/roadmap/0.1/0.1.4/issue98/README.md).
The canonical graph, native encoding, seven tables and 4-KiB page policy remain.

## Source authority and scope

The authority is evidence head **cf3a058925c3012fda0fae922dc081766bb8fa99**, with
measured R26 product **861e388339ef5572659cb16ef0c8febbff0351df**. The isolated
tiny-history worktree uses those production crates plus benchmark additions.
The older main checkout and milestone status banners are not current format
specifications. All implementation links below pin the evidence head. This
inventory involved source reads only: no builds, tests, or benchmarks.

The baseline already includes canonical CAS, CDC, persistent extent sharing,
SQLite packs, and Zstandard FULL/PREFIX representations. It does not store every
object in a separate host file. The agreed v0.1.5 direction is bounded whole-file
similarity reuse for complete saves below **128 KiB**, preserving **4 KiB SQLite
pages** and existing explicit range edits. New format details belong in the spec.

## Owners and representations

| Owner or representation | Existing responsibility |
| --- | --- |
| SDK/coordinator on host | Workspace lifecycle, native import, canonical construction, admission and publication |
| Live operation owner | One authoritative mutable namespace/file state; runs beside Linux FUSE in the approved container topology; direct-host placement reuses the same core |
| Host backing service | Physical spool reservations/appends, authenticated immutable acquisition, retained frozen facts |
| Portable workspace core | Inode/name/link rules, prepared edits, persistent piece references and resource validation; no SQLite or Unix file ownership |
| Canonical object graph | Namespace root → inode table → inode records → directory/content/metadata roots |
| File state and extent tree | Logical length/profile plus bounded mapping nodes referencing slices of canonical payload chunks |
| SQLite physical store | Object locators and encoded packs, independent of canonical identity |

The host's frozen live facts are construction/recovery input, **not a second
independently mutable POSIX namespace**. [Placement and backing][backing]
[portable pieces][pieces] [canonical graph][refs]

## 1. Native directory Init: discover once, construct in bounded parallel work

```text
HOST public initialize_layerstack(native directory)
  |
  +-> bounded filesystem discovery / sorted namespace frontier
  |       supported fast plan, or ordinary serial fallback
  |
  +-> existing workers (CPU/task/budget capped, at most 8)
  |       read source bytes + metadata
  |       small-file completion / streaming CDC
  |       canonical payloads, file states, inode facts
  |       bounded checked owned output slabs
  |                         |
  |                         v
  +------------------ shared admission consumer
                          membership / exact duplicate comparison
                          FULL/PREFIX encoding / packs / locators
                          bounded SQL cohorts
                              |
               ordered namespace + final Layer/LayerStack publication
```

This path imports a host directory; it does not execute the workload through
FUSE or first create a live Workspace. The fast planner expands bounded nested
frontiers and keeps source ordering for final namespace assembly. Unsupported
shapes retain the serial importer; discovering a large tree does not authorize
an unbounded flattened manifest or a whole-tree byte preflight. Workers construct
file output, while the consumer/coordinator owns SQLite and final assembly.
[Init entry and fallback][init] [bounded frontier and driver][frontier]

Small-file construction below the CDC minimum uses bounded complete input with
length/EOF validation; larger streams use CDC. The shared output driver has
256 KiB slabs, at most 512 objects per slab and four queue slots. Completed-file
facts allow output delivery without mandatory whole-candidate payload
spill/readback. Worker results carry structural facts instead of making the
parent reread every payload. Native and Workspace construction share these
builders and admission machinery, **not their discovery mechanisms**.
[File construction][checkedfile] [output bounds/driver][output]

The unavoidable work includes reading supplied source bytes, assigning canonical
identities and validating inputs. Avoidable repetition includes rediscovering
completed facts, copying owned payloads into another delivery buffer, or
rehashing an immutable authenticated owner at every internal handoff. General
fallbacks and persisted/spilled reads still perform real I/O and validation.

## 2. Ordinary POSIX activity: mutate live pieces before publication

```text
LINUX application: open / read / write / truncate / rename / unlink
       |
       v
kernel page cache + FUSE callbacks
       |
       v
LiveOwner: admission + operation ordering + portable inode/piece state
       |                                      |
       | read local Inline/Zero/pending bytes | missing immutable fact/range
       |                                      +---------> HOST acquisition
       |
       +-> reserve backing / append-window bytes
               local retained frame -> grouped HOST spool append + ACK

       Live state is now changed; no new canonical Commit is implied.
```

A write prepares an edit against the current revision and validates resources,
retains bytes in its pending backing frame, then applies the prepared piece
change. A bounded append window amortizes physical transport. Flushing sends the
filled reservation to host backing and cancels unused capacity; failures mark
the owner failed. The acknowledgement therefore depends on retained resources
and applied live state, not on a fresh SQL transaction per syscall.
[Write preparation and retained bytes][livewrite] [append flush][append]

Reads clone a stable file/read plan before awaiting acquisition. Inline, Zero and
still-pending spool ranges can be served locally. Immutable Base ranges consult
an owner-scoped cache; misses request host `READ_BASE`. Backed spool ranges use
the retained backing owner. Existing grouped acquisition/prefill reduces repeated
host crossings, but a cache miss is not zero I/O and a fragmented plan can require
several pieces. [Live read plan][liveread] [host requests][hostread]

### POSIX-visible constraints carried by this design

| Operation | Behavior the storage change must preserve |
| --- | --- |
| OPEN / close | Opening establishes valid inode lifetime before success. Ordinary handles retain pins; optimized kernel-reference modes have explicit ownership preparation. RELEASE/FORGET must discharge the corresponding lifetime. |
| READ / WRITE | Offset operations act on the current live inode, with per-file ordering and retained read operands. Newly acknowledged bytes must be visible through the supported live paths before Commit. |
| Truncate / extension | Change logical ranges; extension can use Zero pieces. Truncating a live inode must not destroy bytes retained by old snapshots or read plans. |
| Rename / replacement | Resolve source/target under namespace serialization, validate target rules, then mutate shared inode bindings. An open descriptor must not be retargeted merely because its pathname was replaced. |
| Unlink / hard links | Remove a binding, not all references to the inode. Open-unlinked files and surviving aliases retain data; canonical construction must include dirty referenced inodes even without materialized paths. |
| mmap / writeback / SDK edit | Kernel pages and live state must agree. Dirty folio writes can contain pre-edit bytes, so acknowledged SDK ranges need explicit cache reconciliation. |
| FLUSH / fsync / Commit | Distinct callback and lifecycle boundaries. Writable callback/error ordering stays intact; none implies an unimplemented power-loss guarantee. |

[OPEN][open] [pin/truncate/link/unlink/rename][posix] [kernel edit reconciliation][mmap]

SDK edits on a remote placement route to that same live owner. Operation cuts
coordinate ordinary callbacks, folio reads/writeback and cache reconciliation;
reserved lifecycle progress prevents invalidation from waiting for callbacks
blocked behind its own gate. The implementation uses bounded cache-store
notifications for changed ranges and protects those ranges against stale dirty
folio writes. Simply deleting an OPEN acknowledgement, invalidation, or fence
would remove a correctness boundary, not just latency. [SDK routing][sdkedit]
[cut gates][cuts] [cache-store and reconciliation][mmap]

**Two uses of “capture” must stay separate.** Remote live capture drains/freezes
operation facts for host construction. The older host-side `CaptureState` is an
opportunistic one-stream CDC worker for eligible new sequential files, with a
bounded channel and invalidation fallback. The remote candidate path uses frozen
backing facts and explicitly passes no captured-file object; it does not rely on
that older streaming shortcut. [Candidate inputs][candidate] [stream capture][capture]

## 3. Public Workspace Commit: freeze facts, publish, install checkpoint

```text
HOST public Commit
  |
  +-> LIVE operation cut / kernel reconciliation / append drain
  |       freeze generation + dirty inode/name facts + retained backing
  |
  +-> HOST candidate construction
  |       dirty files once (including referenced aliases)
  |       range edits -> reuse Base pieces, chunk replacements
  |       full rewrites -> complete input construction
  |       finalized file output -> shared checked admission
  |       dirty directory/inode frontier -> final namespace root
  |
  +-> admit remaining objects -> retain Workspace stage
  |       check expected Branch/head/base
  |       publish Commit/head atomically, or retain stage for retry
  |
  +-> LIVE bounded checkpoint delivery / complete validation / installation
          retire no-longer-owned backing, resume, return public status
```

Commit consumes dirty bindings and file facts rather than rescanning the complete
workspace. Frozen inputs and generation/identity checks prevent construction from
mixing revisions. Existing retained range edits use `FileMutationBatch`; only
replacement gaps are rechunked. Same-length candidate ranges may be compared with
the base to detect no-ops. Complete overwrites can lose retained Base pieces and
still require full input processing; fallback `file_matches` reads/hashes whole
versions. There is no universal prefix/suffix recovery of an unknown tiny edit.
[Dirty frontier and worker budgeting][commitbuild] [file mutation and comparisons][mutation]

Worker budgeting is storage-sensitive: plans with a predecessor currently cap at
one producer so correspondence scratch fits; pure-new plans retain bounded
parallelism. Dropping this cap without replacing its budget proof previously
made all PREFIX attempts unreachable. Clean commits reuse roots but may still
need release/retirement work. [Predecessor cap][commitbuild]

Physical admission can precede final root publication. The session owns cleanup
until a complete Workspace stage is retained. Conditional Branch publication can
then fail while leaving that intentional stage available for retry. This differs
from leaking private rows after failed construction. Once publication succeeds,
pending-publication/checkpoint state prevents retry from creating another Commit.
Installation failure is a distinct public outcome; a stored root alone is not
successful presentation. [Staging/publication][publish] [transition/retry][transition]
[bounded checkpoint transport][checkpoint]

## 4. Rope algorithms operate on an extent tree; live pieces are another layer

CDC boundaries are **8/16/32 KiB min/target/max**; final fragments can be shorter,
and empty files need no payload chunk. The canonical file state stores length,
extent count, tree level, profile and mapping root. Leaves reference payload IDs
with source offsets and lengths; branches carry cumulative ends. Bounded nodes
have at most 128 entries. [CDC][cdc] [extent grammar][extents]

“Rope” names the build/read/split/concat/edit algorithms over this persistent
extent mapping. It is not another full-text blob beside the extent tree. The
mutable Workspace PieceTree separately references Base, Inline, Spool and Zero
ranges before canonical construction.

```text
Live pieces:     Base prefix | Inline/Spool replacement | Base suffix
                                  Commit
Persistent map:  old chunk slice | new chunk(s) | old chunk slice
                      old snapshot keeps its own mapping
```

Replacement scans new bytes, splits the old mapping and joins retained sides.
It avoids rechunking untouched data, but creates mapping metadata. A tiny retained
slice keeps its backing chunk and required decoding bases alive. Different extent
layouts can expose identical file bytes while having different file-state IDs.
[Split/concat implementation][rope]

## 5. Canonical identity, packed storage and authenticated reads

Object IDs are 32-byte domain-separated digests of **canonical framing plus
content**, not arbitrary raw-file hashes. FULL/PREFIX is physical representation;
reconstructed canonical bytes retain the same ID. New v0.1.5 grouping must not
silently redefine these framing or identity rules. [Identity][identity]

```text
objects[ObjectId] -> canonical_length, pack_id, group_number, record_number
                                             |
object_packs[pack_id].data -> group directory -> selected record
                                             |
                                  FULL / PREFIX + required base
                                             |
                        decode -> reconstruct canonical framing
                               -> authenticate ObjectId -> slice file range
```

Schema 7 has `object_packs`, locator `objects` (WITHOUT ROWID), `commits`,
`branches`, `layers`, `layer_stacks`, and `workspace_stages`. Physical pack IDs
are local integer locators. Canonical length differs from encoded length.
A SQLite page can hold many rows; packs span pages. Neither a 128 KiB file cutoff
nor a short delta promises one row or page. [Schema][schema]

The reader supports legacy pack grammars and native Zstandard FULL/PREFIX payload
records. Native PREFIX uses predecessor bytes as codec history, not a textual
patch script; raw payload records remain bounded at 32 KiB. Existing native
chains can reach **four edges**, subject to ordering, byte/work and memory guards.
This is distinct from the proposed **one-level whole-file** v0.1.5 policy.
[Pack grammar][packs] [native chain reconstruction][decode]

Locator membership can avoid content decoding. Demanded reads batch locations,
extract required group/record bytes, reconstruct needed bases and authenticate
requested canonical objects. Authentication of requested data does not imply a
whole-pack corruption census. Cache hits can avoid some acquisition, while repeated
base/group decoding across calls remains measurable work. Logical read bytes,
BLOB bytes, decoded bytes, hashes and device reads are different counters.
[Location batching][locations] [reconstruction][decode]

## 6. Preserve useful batching without erasing trust or publication boundaries

| Boundary | Existing fact or necessary work | Redundant work to avoid |
| --- | --- | --- |
| Checked private memory → shared consumer | Immutable owned canonical identity/framing can travel with bytes | Clone/rehash solely due to wrapper handoff |
| Private prepared-pack spill → owned encoded pack | Check exact length/EOF and retained exact-byte digest; prior canonical framing facts remain owned | Decode and rehash every record solely to recover facts retained by this owner |
| Persisted canonical input → checked operand | Reconstruct demanded bytes, authenticate ObjectId and bounds | Refetch the same valid retained operand for each sibling |
| Duplicate object → reuse | Exact equality/authentication, correct owner/snapshot | Repeated cross-batch decode when bounded operand reuse suffices |
| Missing ID → insertion | Exclusive-owner absence proof with publication epoch | Second negative query while that proof remains valid |
| Physical batch → SQL cohort | Fit codec/operand owners; advance epoch at insertion | Commit each physical batch when approved bounded coalescing applies |
| Stage → final Commit → checkpoint | Dependency availability, conditional publication, recovery, coherence | Rediscover already finalized namespace/file facts |

For example, the oversized singleton preparation path spills an already checked
canonical operand with its pack prefix, then verifies the reread against a retained
exact-byte digest. That private transport check differs from reconstructing a
demanded persisted canonical object. [Prepared singleton ownership][prepared]

Physical preparation is capped separately from SQL transactions. Init can
coalesce physical batches under existing **8,191-object / sub-4 MiB** public
ceilings without retaining all encoding buffers; other paths keep their own
publication behavior. There is no “every save is one transaction” guarantee.
Dependencies must actually be admitted, not merely present in a hint or input
stream. Error/drop cleanup preserves preexisting objects and required bases;
cleanup failure quarantines unsafe future writes. [Admission bounds][output]
[admission implementation][admission] [failure ownership][publish]

The selected schema uses MEMORY journaling and synchronous=OFF. Commit/reopen
correctness under the implemented failure model is not power-loss durability,
backup, or garbage collection. Canonical reachability and physical PREFIX-base
closure both matter; encoding changes must retain both until safe reclamation.
[Effective Store policy][policy]

## v0.1.5 preservation requirements

Keep the three paths distinct: native discovery, live POSIX mutation, and dirty
Commit construction. Add whole-save similarity at the existing shared construction/
admission seam, with actual predecessor provenance and reachable aggregate budgets.
Do not route explicit small range edits through whole-file reconstruction.

For every proposed extra read, query, copy or hash, identify its owner and whether
existing checked bytes/facts already suffice. Preserve necessary acquisition,
collision, publication and coherence acknowledgements; remove repeated work at its
shared cause. Validate actual selection/fallback counts, retained history,
POSIX-visible behavior and total allocated storage alongside complete public
latency. The [benchmark guidance](benchmark_success.md) and
[historical failures](past_mistake.md) define the concrete checks and evidence limits.

[backing]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/live_backing.rs#L62
[pieces]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace-core/src/file_edit.rs#L384
[refs]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-content/src/object/references.rs#L11
[init]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/layerstack.rs#L21
[frontier]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/layerstack.rs#L1162
[checkedfile]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/objects.rs#L2775
[output]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/objects.rs#L29
[livewrite]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-fuse/src/live_owner.rs#L1156
[append]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-fuse/src/live_owner.rs#L1331
[liveread]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-fuse/src/live_owner.rs#L1377
[hostread]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/live_backing.rs#L433
[open]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-fuse/src/filesystem.rs#L692
[posix]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-fuse/src/live_owner.rs#L2024
[mmap]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-fuse/src/live_owner.rs#L2741
[sdkedit]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/lifecycle.rs#L797
[cuts]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-fuse/src/live_runtime.rs#L294
[candidate]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/changes.rs#L357
[capture]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/capture.rs#L48
[commitbuild]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/changes.rs#L560
[mutation]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/changes.rs#L1537
[publish]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/workspace.rs#L423
[transition]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/lifecycle.rs#L163
[checkpoint]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-workspace/src/live_backing.rs#L1350
[cdc]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-content/src/file/cdc/gear.rs#L7
[extents]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-content/src/file/extent.rs#L1
[rope]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-content/src/file/rope/edit.rs#L47
[identity]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-content/src/object/id.rs#L16
[schema]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/sql/schema/v7.sql#L4
[packs]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/objects/pack.rs#L1
[decode]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/objects/read.rs#L523
[locations]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/objects/read.rs#L145
[admission]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/objects/admission.rs#L1
[policy]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/schema.rs#L373
[prepared]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/crates/layerfs-layerstack-store/src/objects/admission.rs#L734

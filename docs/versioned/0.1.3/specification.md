# LayerFS 0.1.3 specification

> **Status:** Release candidate for LayerFS 0.1.3.

## Product model

One SDK Client binds one local SQLite LayerStackStore, a passive Monitor and
Workspace management. A separate Store has an independent identity and
canonical-object namespace. LayerStacks contain immutable Layers; Branches
identify writable histories of immutable Commits. A Workspace is ephemeral
working state attached to one Branch. Commit publishes a snapshot; End releases
the Workspace and does not implicitly publish it.

Canonical content uses authenticated objects, content-defined chunking, ropes
and structural copy-on-write. Existing objects are reused by ObjectId. LayerFS
retains the established canonical identity domains and immutable entity-name
rules; schema-v5 staging does not change content roots or Commit identity
derivation. Names are immutable and do not participate in content identity.

Initialize creates a genesis Layer and named LayerStack from an empty root or
native directory. Fork creates a named Branch at a selected immutable state
without copying canonical payloads. Add Layer publishes an eligible Branch head
as a Layer using its root. Queries and Diff are bounded and paged; reconciliation
reports typed conflicts that must be resolved before publication. Exact request
and outcome types are documented in the [SDK reference](sdk.md).

## Shared live Workspace

The live Workspace core owns namespace, inode and file-mutation semantics. For
managed Linux/FUSE execution, the execution-side live owner handles filesystem
callbacks and coordinates SDK edits. The host retains physical backing,
canonical construction, SQLite and publication. Capability-authenticated
connections bind those responsibilities to the Workspace; another owner does
not acquire mutable state merely because immutable data are cached.

File edits use the common piece representation. A non-empty SDK range-edit batch
targets one regular file and prevalidates the batch before applying it.
Replacement data may be inline bytes, logical zeros or deletion. Ordinary write,
append, sparse growth and truncate use the same live mutation semantics. Shared
physical backing segments retain range ownership across reads, rollback and
open-unlinked file lifetimes. Failed append cleanup keeps retained physical
bytes accounted for.

FUSE write acknowledgement accepts bytes into live, bounded Workspace state;
it does not mean a canonical Commit or a host database sync has occurred.
Buffered backing transfer, filesystem flush/fsync and Commit are distinct
boundaries. None upgrades the Store's crash-durability profile. Materialization
remains a separate projection route; its existence does not imply every live
FUSE behavior applies to every projection.

## Commit and continuation

Commit establishes a filesystem-operation cut. It can drain admitted callbacks
without waiting for an entire command to exit; the mount, current directory and
open handles can remain live. Mutations admitted after the cut resume as new
Workspace work. This is a coherent filesystem snapshot boundary, not a promise
that an application's multi-syscall transaction is atomically captured.

The host constructs the selected canonical result from frozen live facts.
Ordinary construction coalesces final inode updates and transfers finalized
owned objects through bounded checked admission. Completed roots are recorded
in `workspace_stages`, then conditionally published against the expected Branch
base and head. A moved head returns the actual competing state rather than
overwriting it. Successful publication retires its stage. An unchanged root can
return `UpToDate` without adding history; a stage may still be created and retired
in that path.

Continuation installs construction's checkpoint facts into existing live nodes.
The published root, mutation generation and live attributes are checked before
installation. Publication success and presentation success are distinct outcomes:
if installation or presentation fails after publication, recovery uses the exact
published snapshot rather than creating another Commit. A rejected publication
can retain a stage for the owning Workspace's recovery or explicit discard.
These are live-process recovery semantics, not restart recovery of an arbitrary
ephemeral Workspace.

Reconciliation with kernel mappings protects SDK-installed ranges while stale
writeback drains, preserves unrelated mapped writes and clips obsolete tails
after resizing. Required writeback can proceed while ordinary mutations are
paused for reconciliation. New-file create handles retain the direct-I/O
restriction described in [limitations](limitations.md#live-filesystem-boundaries).

## Resource and storage boundaries

Resource policies bound Workspace spool and final-delta use. Canonical
construction, object admission, queries and reads use their own bounded buffers;
one buffer limit is not a whole-process memory limit. Immutable read caches are
bounded and optional acquisition falls back when admission is unavailable.
Cached directory pages establish absence only within their validated coverage,
and delayed replies must match the live namespace revision before installation.

Initialization fast paths and ordinary Commit share construction/output
machinery where applicable, but native fallback, incremental construction and
planned admission retain distinct routes. No universal worker-count or
all-input fast-path guarantee is implied.

The [storage contract](storage-format.md) defines schema, migration, publication
and acknowledgement semantics. The [limitations](limitations.md) state the
runtime, durability and evidence boundaries.

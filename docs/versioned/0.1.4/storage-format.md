# LayerFS 0.1.4 storage format

> **Status:** LayerFS 0.1.4 Developer Preview manual.

## File and connection

A Store is one local SQLite database file. Create refuses an existing path.
Connect validates the exact supported schema and obtains exclusive ownership;
never inspect a Store through another connection while its owner writes it.
SQLite must support STRICT tables (3.37.0 or later). New Stores use:

```text
application_id=0x4c46534c
user_version=7
page_size=4096
foreign_keys=ON
journal_mode=MEMORY
synchronous=OFF
temp_store=MEMORY
cache_size=-32768
cache_spill=OFF
mmap_size=0
threads=0
locking_mode=EXCLUSIVE
busy_timeout=5000ms
```

Connect accepts supported Stores with 4096- or 65536-byte pages and preserves
their page size. The page cache is a named SQLite allowance, not a total process
memory limit. The Store serializes SQLite access. For managed container
execution, the host owns SQLite, canonical construction/publication and physical
spool; the container owns daemon/FUSE/workload execution.

Publication is readable from the same live local Store process. MEMORY journaling
and synchronous OFF provide no process-crash, OS-crash or power-loss durability
guarantee. Filesystem fsync does not upgrade this database profile.

## Compatibility and migration

Create produces schema 7. Connect accepts exact schema 7 and supported legacy
schema 6; it does not migrate or promote the latter. Schema-6 Stores read and
write legacy version-1 packs. Research schema-6 Stores containing native packs
without the schema-7 writer fence are rejected during preflight. Schema 7 can
contain native version-2 packs alongside legacy packs used by fallback routes.

Published v0.1.3 uses schema 5, which v0.1.4 rejects. Schema 4 and other
unsupported versions, unexpected schema objects and WAL-mode Stores are also
rejected. There is no in-place migration, downgrade or retained-history transfer
command. Retain the original Store with its matching binaries; directory import
into a new Store copies a selected filesystem state and starts new history.
Canonical identity preservation does not imply file-format compatibility.

## Schema

Schema 7 has seven STRICT tables and 28 columns, with seven named unique indexes:

| Table | Columns | Role |
| --- | ---: | --- |
| `object_packs` | 2 | Physical pack ID and packed bytes |
| `objects` | 5 | ObjectId, canonical length and pack/group/record locator |
| `commits` | 4 | Immutable Commit root, parent and base Layer |
| `branches` | 5 | Named Branch and publication pointers |
| `layer_stacks` | 3 | Named LayerStack and head Layer |
| `layers` | 6 | Immutable Layer lineage and root |
| `workspace_stages` | 3 | Workspace-owned completed root awaiting retirement |

`object_packs` uses an INTEGER PRIMARY KEY; the other tables use WITHOUT ROWID.
The exact [schema-7 DDL](../../../crates/layerfs-layerstack-store/sql/schema/v7.sql),
[legacy schema-6 DDL](../../../crates/layerfs-layerstack-store/sql/schema/v6.sql)
and [connection validation](../../../crates/layerfs-layerstack-store/src/schema.rs)
are authoritative. A stage is not a Branch head or a restartable Workspace.

## Canonical identity and physical encoding

Content-defined chunking, canonical bytes, ObjectId domains, CAS/COW semantics
and immutable Layer/Commit identities are preserved. Physical pack representation
is separate from canonical identity. Locators identify records; reads validate
framing, reconstruct the canonical bytes and authenticate their ObjectId.
Native representations support independently encoded content and eligible
predecessor-based reconstruction; unsupported input retains the canonical
fallback. Dependency ordering and validation are required before publication.
See the [pack grammar](../../../crates/layerfs-layerstack-store/src/objects/pack.rs)
and [read implementation](../../../crates/layerfs-layerstack-store/src/objects/read.rs).

Ordinary native packs remain bounded at 256 KiB; their groups are bounded at
64 KiB. Oversized canonical objects use the explicitly bounded legacy fallback.
These physical limits are distinct from SQL transaction and operation ownership
limits. They do not authorize changing object bytes under an existing ObjectId.

## Admission, reuse and publication

Physical batches remain bounded. Coalesced SQL transactions contain at most
8,191 objects and no more than 4 MiB canonical payload. Other candidate, queue,
index, reconstruction and SQLite allocations have separate limits.

Directory Init can retain authenticated comparison bytes across admission
batches within its existing owner allowance: a 2 MiB reservation replaces part
of the previous allowance rather than raising it. Every reuse still performs
locator lookup, validates physical location and canonical length, and compares
exact bytes. An ID, membership-filter hit or earlier occurrence alone is never
sufficient collision validation. Ineligible or uncached records use the normal
authenticated read path. The cache is owner-scoped and bounded.

Admission validates dependencies and completes selected output before exposing
its final root. Pending and earlier committed batches owned by failed admission
are rolled back through the checked ownership path; failed cleanup quarantines
writes. Root publication remains atomic within the live Store transaction.

Workspace publication validates its retained stage and expected Branch base/head,
inserts or verifies the immutable Commit, conditionally advances the Branch and
retires the stage. No-op publication can retire a stage without creating history.
A stale head returns the competing state. Presentation failure after successful
publication does not undo that Commit; use the [SDK recovery API](sdk.md#commit-and-recovery).
Explicitly completed admission and subsequently rejected publication are distinct
boundaries: unreachable objects may remain, and automatic garbage collection is
not provided. These live-process integrity rules do not promise crash durability.

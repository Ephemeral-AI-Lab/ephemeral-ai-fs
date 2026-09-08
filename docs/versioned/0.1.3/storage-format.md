# LayerFS 0.1.3 storage format

> **Status:** Release candidate for LayerFS 0.1.3.

## File and connection

A Store is one ordinary local SQLite database file. Create refuses an existing
file; Connect requires an existing regular file, validates its exact supported
schema and acquires exclusive ownership. SQLite must support STRICT tables
(3.37.0 or later). Runtime settings are:

```text
application_id=0x4c46534c
user_version=5
page_size=65536
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

The Store owns the connection and serializes SQLite access. Managed container
execution leaves this database on the host and uses authenticated backing
services; it does not require the database to be bind-mounted into the container.
Ephemeral backing and journals are not additional authoritative databases.

The publication acknowledgement means the committed transaction is readable
from the same live local Store process. With MEMORY journaling and synchronous
OFF, it does **not** guarantee survival of a process crash, operating-system
crash or power loss. A Workspace Commit, staging row or successful filesystem
fsync does not change that profile. Maintain independent backups of important
data.

## Compatibility and migration

Version 0.1.3 creates schema-v5 Stores and accepts exact schema-v4 or schema-v5
Stores on Connect. A v4 connection first validates the previous schema, acquires
the Store lock, then executes the v4-to-v5 migration in an immediate transaction.
The migration adds `workspace_stages` and updates `user_version` to 5; the final
schema is validated again. WAL-mode inputs, unsupported versions and unexpected
schema objects are rejected rather than silently normalized into a supported
Store.

Migration is an in-place format change. The v4-only 0.1.2 binary rejects a
migrated v5 file, and 0.1.3 provides no downgrade. Close the old owner and preserve
an independent closed-Store backup before migration if rollback is required.
Do not treat Connect as a read-only inspection operation for a v4 file.

Canonical object encodings, ObjectId domains, CDC profile and immutable Layer
and Commit identities remain separate from this schema change. Existing
canonical objects are not re-encoded by migration. This is forward migration
support, not an unchanged five-table Store contract or a mixed-version daemon
compatibility promise.

## Schema

Schema v5 has six STRICT tables and 23 columns:

| Table | Columns | Role |
| --- | ---: | --- |
| `objects` | 2 | Authenticated canonical bytes keyed by 32-byte ObjectId |
| `commits` | 4 | Immutable Commit root, parent and base Layer |
| `branches` | 5 | Named Branch and publication pointers |
| `layer_stacks` | 3 | Named LayerStack and head Layer |
| `layers` | 6 | Immutable Layer lineage and root |
| `workspace_stages` | 3 | Workspace-owned completed root awaiting retirement |

The added table is exactly:

```sql
CREATE TABLE workspace_stages (
    workspace_id BLOB PRIMARY KEY CHECK (length(workspace_id) = 16),
    branch_id BLOB NOT NULL CHECK (length(branch_id) = 17)
        REFERENCES branches(branch_id),
    root_id BLOB NOT NULL CHECK (length(root_id) = 32)
        REFERENCES objects(object_id)
) STRICT, WITHOUT ROWID;
```

The existing five tables retain their v4 definitions and seven named unique
indexes. Metadata tables use WITHOUT ROWID; `objects` retains rowid storage and
its primary-key index. The exact [v5 DDL](../../../crates/layerfs-layerstack-store/sql/schema/v5.sql)
and [migration SQL](../../../crates/layerfs-layerstack-store/sql/schema/migrate_v4_to_v5.sql)
are part of this release. A stage is not a Branch head, a Commit history entry,
a persistent shell or a complete restartable Workspace.

## Objects, admission and publication

Canonical bytes remain append-only. Reads authenticate bytes against ObjectId;
retained immutable authenticated owners may be reused without repeating that
work, while fresh spill/storage reads authenticate again. Checked insertion
preserves collision checks. Successful reuse does not authorize replacing an
existing object's bytes.

Ordinary finalized output enters checked admission in bounded pages: at most
8,191 objects and 4 MiB of canonical payload per page. That limit excludes
separate candidate, index, queue, live-state and SQLite-cache costs. Other
fallback/query page limits retain their own contracts; the old blanket rule of
fewer than 128 objects per admission transaction is not the 0.1.3 contract.

Completed selected output is admitted before its root is staged. Publication
validates the retained stage and expected Branch base/head in a transaction,
inserts or verifies the immutable Commit, conditionally advances the Branch,
and retires the matching stage. No-op publication retires the stage without
creating history. A stale-head failure leaves the stage available to the owning
operation; explicit discard can remove it. Earlier admitted objects can remain
unreachable after failure and be reused later. This release provides no automatic
garbage collection.

Publication never intentionally exposes an incomplete canonical closure. That
live-process integrity rule must not be confused with crash durability. Exact
schema/runtime validation lives in [the Store implementation](../../../crates/layerfs-layerstack-store/src/schema.rs),
and stage ownership in [the staging implementation](../../../crates/layerfs-layerstack-store/src/staging.rs).

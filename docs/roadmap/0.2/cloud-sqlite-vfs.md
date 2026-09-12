# Cloud Store and LayerFS-VFS roadmap

> **Status:** Brainstorm / candidate architecture, 2026-09-12. Not a product
> contract, schema proposal, or release commitment. Nothing here is approved
> for implementation in 0.1.x.

This document answers one question: **the benchmark path runs with SQLite
journaling disabled, so what has to change before a LayerFS Store can live in
the cloud and expose a LayerFS VFS built on SQLite?**

It records the constraint, separates two different meanings of "VFS", and
proposes a staged roadmap whose first step is deliberately boring.

## 1. The starting constraint, stated precisely

Current Store connection policy
([`schema.rs`](../../../crates/layerfs-layerstack-store/src/schema.rs)) is:

| Pragma | Value | Why it is there |
| --- | --- | --- |
| `journal_mode` | `MEMORY` | No rollback journal file on the hot path |
| `synchronous` | `OFF` | No per-transaction fsync |
| `locking_mode` | `EXCLUSIVE` | Never acquire or release a POSIX file lock after open |
| `cache_size` | 32 MiB | Bound, and `cache_spill=OFF` keeps pages resident |
| `mmap_size` | `0` | Avoid an mmap the Store does not need |
| `temp_store` | `MEMORY` | No scratch file beside the Store |

Three consequences follow, and all three matter for cloud work:

1. **`preflight_connect` rejects any Store whose header says `wal`.** This is a
   deliberate compatibility boundary, not an accident, and it is tested.
2. **Mutual exclusion is a POSIX file lock.** `locking_mode=EXCLUSIVE` is
   correct on one host and meaningless across hosts. The one-writer invariant
   currently has no owner other than the local filesystem.
3. **A local process must not open the Store directly in cloud mode.** The
   exclusive-lock trick is what makes a *second* concurrent opener fail closed.
   Move the file to shared storage and that protection is transferred to
   whatever locking the network filesystem implements — which is exactly the
   thing that varies per provider.

`reserve_inode_serials` already shows the shape of the problem: it temporarily
raises the connection to `journal_mode=DELETE`, `synchronous=FULL` for one
durability-critical transaction, then restores `MEMORY`/`OFF` and removes the
zeroed rollback journal. So the codebase already accepts that **policy is
per-transaction, not per-Store** — the inode allocator is the one region where
"reserve before you publish" needs a real journal. Any cloud design inherits
that requirement for the allocator and for catalog publication.

Also relevant, and good news: `spill.rs` runs private scratch databases at
`journal_mode=OFF` with `locking_mode=EXCLUSIVE`. Those are derived, disposable,
and unlocked. They should **not** be dragged into any cloud policy change.

## 2. The two meanings of "VFS", and why they are not the same project

The phrase "LayerFS VFS built on SQLite" resolves to two different artifacts.
Naming them separately prevents a year of confusion.

### (a) LayerFS as the *client* of SQLite — the Store's VFS

LayerFS reaches its database through SQLite's VFS layer. A cloud build can be
implemented at the *SQLite VFS* seam: write `sqlite3_vfs` / `sqlite3_io_methods`
shims so the Store's pages live somewhere remote.

This buys something real: **a single sealed mechanism**, so no LayerFS code
learns about the network. But it inherits every SQLite locking and WAL rule,
including the ones LayerFS currently switches off, and it turns every pager miss
into a network round trip. It also cannot express LayerFS's actual invariants
(content addressing, immutability, append-only packs) — a VFS only sees pages.

### (b) LayerFS as the *provider* of a VFS — a filesystem for SQLite users

`sqlite3_vfs` registered as `layerfs`, so a stock `sqlite3` binary or any
application with `?vfs=layerfs` stores its database **inside a LayerFS
Workspace**, gaining commit, branch, fork, and diff per query for free.

This is the more interesting product, and it is a *projection* — it belongs on
the same axis as FUSE, OverlayFS, and reflink in
[roadmap architecture](../architecture.md). It requires no change to SQLite at
all. Its hard parts are not storage; they are file locking, POSIX advisory
locks, `fsync`, and concurrent access across processes.

### (c) The Store's *backend* seam — what the code already has

Independently of (a) and (b), `layerfs-content` already defines
[`ObjectRead` and `ObjectStore`](../../../crates/layerfs-content/src/object/access.rs).
`CoreReader` is the only production implementor that touches SQLite; tests use
an in-memory `MemoryStore`. The vision document already states the north star:
"the same object should remain valid in SQLite, OPFS, memory, an object store."
That trait pair is the narrowest place to put a remote backend.

**Recommendation:** pursue (c) first. Treat (a) as a bounded experiment with a
kill criterion. Treat (b) as a genuine 0.3-class product, scoped separately.

## 3. What the current schema already gives us for free

The v10 schema
([`v10.sql`](../../../crates/layerfs-layerstack-store/sql/schema/v10.sql))
splits cleanly into two very different regions:

```text
append-only, immutable, content-addressed          mutable catalog
─────────────────────────────────────────          ────────────────────────
object_packs   (pack_id INTEGER PRIMARY KEY,       commits, branches
                data BLOB)                          layers, layer_stacks
objects        (object_id PRIMARY KEY,              scope_allocator
                pack_id, group, record)             workspace_stages
metadata_value_groups
```

Pack IDs come from `SELECT COALESCE(MAX(pack_id),0) FROM object_packs` inside
the admission transaction, so they are dense and monotonic on the source of
truth. `objects` rows reference packs without `ON DELETE CASCADE`, and canonical
identity means an object's locator row is decided once, at admission.

This is the single most important fact in this document:

> **A LayerFS Store's object bytes are a logical append-only log keyed by dense
> integer pack IDs. Only the catalog is a mutable B-tree.**

That is exactly the shape object storage is good at. A replica can pull
`object_packs` by `pack_id > cursor` and `objects` rows by pack, in any order,
idempotently, and get byte-identical canonical objects. Nothing needs to
reconcile mutable pages. Only the catalog needs a real authority.

### The one exception, and why it matters

There is exactly one production path that removes published packs:
`AdmissionSession::rollback` in
[`objects.rs`](../../../crates/layerfs-layerstack-store/src/objects.rs) deletes
every `objects` and `object_packs` row above a recorded `baseline_pack`, up to
the current `MAX(pack_id)`. (Other `UPDATE object_packs` statements in the tree
are test-only corruption injection.)

Two consequences for any replicator:

1. **Pack insertion is not the durability boundary; transaction commit is.** A
   log that streams packs as they are inserted would publish bytes that an
   unwinding admission can later retract.
2. **A replication cursor cannot assume monotonic, never-retracted IDs.** Either
   the log is defined to extend only on commit, or it needs explicit retraction
   records. This is recorded as open question 7 below.

Note also that this deletion path is unrelated to the explicit compaction that
[v0.1.5 removed](../0.1/0.1.5/compaction-removal.md); it is retained, and it
is now the only mechanism that removes packs. It deletes in 512-row batches with
one transaction per batch, which is fine for a local disk and questionable for a
remote one — and it is the *abandonment* path, so it runs precisely when
something has already gone wrong.

## 4. Decision D1 — where the bytes live (pick one, deliberately)

The roadmap must not carry two of these at once. The choice gates everything
else.

### D1-A. Hosted Store endpoint; keep the file local

The Store stays on one machine's local disk and stays fast. Clients talk to it
over the existing authenticated daemon protocol; each client keeps a local
materialized or FUSE projection.

- Store policy changes: **none**. `MEMORY`/`EXCLUSIVE`/`OFF` stay valid.
- WAL question: **does not arise**.
- Real work: multi-tenant namespaces, authorization, streaming object transfer,
  session lifetime, backpressure, and a stable wire contract.
- Ceiling: single-writer per Store, i.e. exactly today's invariant, now
  explicit instead of implicit in a file lock.

This is the smallest delta with the largest share of cloud value. It is also the
only option that does not put the Store's performance on a network.

### D1-B. Store file on a shared network filesystem

Many hosts open one Store file over NFS/SMB/EFS/Filestore.

- `locking_mode=EXCLUSIVE` and `journal_mode=MEMORY` become **unsafe**, not
  merely slow. `MEMORY` means readers see committed state only through one
  process's page cache; two hosts would each have a private view.
- Requires `journal_mode=WAL` with a VFS that provides `xShmMap`/`xShmLock`,
  i.e. option (a), plus a network filesystem whose byte-range locking is
  actually correct. Provider support varies and must be proven per provider.
- Every pager miss is a round trip; `cache_spill=OFF` and a 32 MiB cache stop
  being obviously right.
- **Recommendation: do not choose this.** It spends the performance budget to
  buy a topology that D1-A and D1-C both express better.

### D1-C. Compute-local Store plus remote object/pack replication

Keep a local SQLite Store as the *write-optimized* representation. Ship the
append-only region to object storage; pull on demand elsewhere. Local reads and
writes stay local-disk fast; the cloud becomes a durability and fan-out layer.

- Store policy changes: `journal_mode=WAL` on local disk (still
  `locking_mode=EXCLUSIVE`, so shared memory stays per-connection and free),
  `synchronous=NORMAL`, and a real crash-recovery story.
- This is the option that *uses* the immutability in §3 rather than fighting it.
- It is also the option that eventually makes (a) unnecessary: a replica is a
  normal local SQLite database that received rows, not a remote-paged file.

**Recommendation: D1-C as the destination, D1-A as the first shippable step.**
D1-A is a subset of D1-C's client path, so nothing built for D1-A is wasted.

## 5. Decision D2 — what happens to WAL

WAL is currently rejected at connect. In a cloud trajectory it must return, and
it should return on its own merits rather than as a remote-paging workaround.

WAL is the right choice for a **local** Store that has a replication obligation:

- it removes the rollback-journal write-then-delete cycle that
  `reserve_inode_serials` has to work around;
- with `locking_mode=EXCLUSIVE` the WAL index lives in the connection's heap,
  so the shared-memory file is unnecessary — WAL does *not* inherently require
  `-shm` when only one process uses the database;
- it gives a durable, ordered, replayable byte stream, which is precisely the
  input a replicator wants.

WAL is the wrong choice for a **remote-paged** Store, where it adds a second
round-trip-heavy file to a pager that is already latency-bound.

So: adopt WAL under D1-C, keep rejecting it under D1-A, and under D1-B treat it
as an explicit, provider-specific, evidence-gated exception. The preflight
check should become a *declared policy assertion* rather than a hard-coded
rejection, so each topology can prove what it requires.

### Scheduling: does this block the 0.1.x benchmark?

No, and it should not. The
[architecture doc](../architecture.md) is explicit that "new projections,
platforms, remote topology, or incompatible contracts do not belong in this
phase," and the benchmark registry is frozen through 1.0.0. Changing Store
journal policy mid-campaign would invalidate comparability for a benefit the
current phase cannot use.

The correct current-phase action is to **keep the decision un-made and
reversible**, which costs four small changes:

1. Extract one function, `store_connection_policy()`, returning the pragma set,
   with a doc comment naming the benchmark rationale and the cloud blocker. No
   behavior change; one seam.
2. Replace the literal WAL rejection with a named error and a comment that
   states the invariant being protected ("no second writer, for any reason"),
   so the cloud work knows what it must re-establish.
3. Stop re-deriving the writer-exclusion rule in more than one place. If
   `locking_mode=EXCLUSIVE` is the mechanism, name it once and assert it as an
   invariant, not as a side effect of configuration.
4. Record the measured *cost* of the current policy: a benchmark row (already
   possible in the retained harness) that reports what `MEMORY`+`OFF` buys over
   `WAL`+`NORMAL`. Without that number, the cloud design will be argued from
   belief. This is observation, not a policy change.

## 6. Staged roadmap

Each stage has an exit gate. A stage that misses its gate does not advance.

### Stage 0 — Name the seams (0.1.x, days)

| Deliverable | Gate |
| --- | --- |
| `store_connection_policy()` extraction, behavior identical | Full existing suite passes unchanged |
| WAL rejection becomes a named, documented boundary | A test asserts the error identity and the invariant comment exists |
| Journal-policy cost row added to the harness | Reports `MEMORY/OFF` vs `WAL/NORMAL` on one fixed workload |
| Glossary: Store VFS vs projection VFS vs object backend | Reviewed and merged into `docs/general/concepts.md` |

No user-visible change. This stage exists purely so Stage 1 can be argued from
measurements and vocabulary instead of preference.

### Stage 1 — Backend seam without a network (0.2 line, weeks)

Prove the abstraction before paying for transport.

1. Define a `StoreBackend` boundary that owns *three* things, not one:
   **canonical object bytes**, **locator resolution**, and **catalog
   transaction commit**. Today all three are the same `rusqlite::Connection`;
   the cloud splits them (objects to blob storage, locators to an index,
   catalog to a coordinator).
2. Implement it twice: the existing SQLite path, and a pure `MemoryStore`
   backend. Run the existing content/filesystem/workspace test corpus against
   both. If the corpus cannot pass on `MemoryStore`, the seam is wrong.
3. Port the existing `ObjectRead`/`ObjectStore` batch methods through the seam.
   `get_authenticated_canonical_batch`, `objects_exist`, and `object_locations`
   are already batch-shaped — keep them that way. A remote backend's cost model
   is *round trips*, and the current paged `IN (...)` queries are the right
   units of work.
4. Add a fault-injecting backend that returns latency, partial failure, and
   duplicate delivery. Assert that authenticated reads still fail closed on
   `IdentityMismatch`.

**Gate:** one workspace lifecycle (init → fork → exec → commit → end) completes
on `MemoryStore` *and* on SQLite with identical resulting identities, and the
latency-injecting backend produces no correctness divergence.

### Stage 2 — Durability and replication primitives (0.2 line, weeks)

Still no cloud. Make the Store able to describe and ship its own history.

1. **WAL under `locking_mode=EXCLUSIVE`** with a real crash-recovery test:
   kill during admission, during `reserve_inode_serials`, and during catalog
   commit; reopen; verify oracles. The current documentation honestly says
   there is no crash or power-loss guarantee — this is where that changes, and
   it is a prerequisite for anything cloud.
2. **An exported log.** Because §3 holds, define the replication unit as
   `(pack_id, data)` plus the locator rows and catalog rows that reference it.
   Decide two things explicitly: the ordering contract (catalog rows may
   reference packs only after those packs are durably stored), and the
   retraction contract (`AdmissionSession::rollback` can delete packs above
   `baseline_pack`, so the log must either extend only at commit or carry
   retractions). Note also that `object_packs.pack_id` is currently allocated
   per-Store via `MAX(pack_id)+1`, so a *shared* log needs either one allocator
   authority or a `(store_id, pack_id)` pair. Choose explicitly.
3. **Verification of a replica.** The evaluator in `tools/layerfs-eval` already
   walks Store and Branch integrity. Extend it to verify a replica against a
   log cursor without trusting the transport.
4. Optionally evaluate rusqlite's `session` extension for catalog changesets.
   It is *not* currently enabled (`rusqlite` is built with
   `cache, hooks, trace, limits, blob`), and the linked SQLite is not the
   `bundled` build, so enabling it is a build-policy decision with a
   supply-chain surface — not a free win. An explicit row-diff log may be
   simpler and more auditable.

**Gate:** a killed writer recovers to a verified state; a replica rebuilt from
the log passes the evaluator byte-for-byte against the source; the log contains
no mutable-page assumptions.

### Stage 3 — Hosted endpoint (cloud line, months)

Ship D1-A. This is where "cloud" first becomes true and where most of the
non-storage engineering lives.

- Multi-tenant Store catalog; per-tenant authorization on the existing
  capability-authenticated daemon protocol.
- Object transfer: streaming, resumable, bounded memory, no full-Store download
  to read one file.
- Session and workspace lifetime; branch leases become server-enforced rather
  than process-local.
- **The one-writer invariant must be re-established as a server property.** The
  current guarantee is a file lock plus a documented error; the cloud version
  needs an authoritative, auditable answer for "who is writing this Store right
  now," including after a client disappears mid-transaction.
- Credentials, quotas, and abuse bounds are out of LayerFS's ownership per the
  vision document but must exist before exposure.

**Gate:** N concurrent clients on M hosts over one Store, with a reproducible
answer to writer contention, and a documented failure story for client death
mid-write.

### Stage 4 — Replicated lineage (cloud line)

Move from "a hosted Store" to D1-C: compute-local Stores that replicate the
append-only region and pull packs on demand.

- Ranged pulls, prefetch, and a local cache keyed by the existing content
  identity — `immutable_read_cache.rs` is the starting point.
- Promotion/fork/`Add` across replicas with compare-and-swap on a shared
  authority rather than a local row.
- Explicit conflict semantics when two replicas advance one Branch. **This is
  the same problem as
  [agent Branch reconciliation](agent-branch-reconciliation/README.md)**; the
  cloud work should reuse its result rather than inventing a second answer.

**Gate:** a Workspace reads a pack that no local host has ever seen, with
authentication intact, and Branch advancement across two replicas resolves
without silent divergence.

### Stage 5 — LayerFS as a provider: the `layerfs` VFS (0.3 line)

Only after Stages 1–4, and only because the locking answer now exists.

Register a `sqlite3_vfs` named `layerfs` where database file contents are
LayerFS objects inside a Workspace. Scope it honestly:

- **Locking is the whole problem.** A SQLite VFS must implement `xLock`,
  `xUnlock`, `xCheckReservedLock`, and `xFileControl`. In LayerFS, "the same
  file open in two processes" is a Workspace problem, not a byte-range problem.
  Decide whether `layerfs` VFS databases are single-process by construction.
  For many useful cases — an embedded app, a test, a CLI, an agent tool call —
  single-process is exactly correct, and saying so plainly is better than a
  partially correct lock implementation.
- **WAL must be handled explicitly.** Returning `SQLITE_READONLY` from
  `xShmMap` with `SQLITE_FCNTL_PERSIST_WAL` off lets a single-connection
  database run in WAL without a `-shm` file; anything else is guesswork.
- **The VFS must not be the storage engine.** It should talk to
  `ObjectStore`/the Stage 1 seam, never to a private file format. Otherwise the
  compatibility contract forks.
- **Do not build this to make LayerFS cloud-native.** It is a projection
  product for SQLite users; the cloud path runs through Stages 1–4.

**Gate:** an unmodified SQLite client creates, writes, commits, and reopens a
database through the `layerfs` VFS, and the resulting Workspace Commit is
readable by an ordinary LayerFS Workspace on another host.

## 7. Alternatives considered and rejected

| Option | Why not |
| --- | --- |
| Move LayerFS off SQLite to a client/server database | Throws away the frozen v10 format, the exact-read authentication story, and the benchmark baseline. The schema is not the problem; the transport is. |
| Put the Store file on S3 behind a FUSE mount | Pager-level round trips per 4 KiB page, no correct locking, no crash story. Strictly worse than D1-A. |
| WAL plus a shared `-shm` file over the network | Shared memory over a network filesystem is provider-defined behavior. Rejected as a correctness dependency. |
| Adopt a third-party cloud SQLite (serverless/replicated SQLite services) | Tempting, and worth a bounded spike — but it moves the Store's compatibility contract to a vendor and does not by itself provide object-level replication, which is where the append-only structure in §3 pays off. Evaluate as a D1-A implementation detail, not as the architecture. |
| Implement the `layerfs` VFS first, because it is the most visible | It has the hardest locking semantics and the least cloud value. It is Stage 5 for a reason. |

## 8. Invariants that must survive every stage

Restating these so no stage quietly trades one away:

1. **Canonical identity is independent of storage, projection, and transport.**
   `ObjectId::for_bytes` remains the authority; `IdentityMismatch` remains fatal.
2. **One writer per Store, for any reason.** The *mechanism* may change from a
   POSIX file lock to a server lease. The *guarantee* may not weaken, and it
   must be auditable.
3. **Required encoding finishes before public success.** No stage may report
   success and replicate later without saying so.
4. **No benchmark-only APIs, hidden caches, or shifted timing boundaries.**
   The cloud work does not get to weaken the measurement contract.
5. **The 0.1.x benchmark registry stays append-only.** Cloud work adds rows or
   starts a new registry; it never rewrites an existing one.
6. **`spill.rs` scratch policy is separate.** Derived, disposable, unlocked
   databases keep `journal_mode=OFF`. They are not a Store.

## 9. Open questions to settle before Stage 3

These are the ones that will otherwise be decided by accident:

1. **Pack ID allocation authority.** One Store, one `MAX(pack_id)+1` allocator.
   A shared replication log needs a single allocator or a namespaced ID. Which?
2. **Catalog ownership.** Does the catalog stay SQLite on a coordinator, or does
   it become a service? The answer determines whether Stage 4 is replication or
   federation.
3. **What is a tenant?** A Store per tenant is simple and probably right; a
   shared Store with per-tenant LayerStacks is cheaper and harder to authorize.
4. **Read path budget.** The current read path issues paged `IN (...)` queries
   against one local connection. What is the acceptable round-trip count for
   reading one 32 MiB file from a remote replica? That number should be written
   down before any transport is chosen.
5. **Does `synchronous=NORMAL` suffice, or is `FULL` required for catalog
   commit?** The inode allocator already answers this for itself; the catalog
   has not been asked.
6. **Crash-consistency of the append-only log.** Packs are inserted in the same
   transaction as their locators. A replicator that streams packs must know
   whether a pack is durable *before* the catalog row that references it is
   published, or it may ship the catalog first.
7. **Retraction semantics.** `AdmissionSession::rollback` deletes packs above
   `baseline_pack`. Is the replication log defined to extend only at commit, or
   does it carry explicit retractions? A puller that already fetched a
   retracted pack must not have made it reachable.
8. **Cost of the current policy.** Unmeasured. Measure it in Stage 0 so the
   cloud tradeoff has a denominator.

## 10. Summary

- The benchmark's `MEMORY`/`OFF`/`EXCLUSIVE` policy is a **single-host**
  optimization, and it silently owns the one-writer invariant. That ownership is
  the actual cloud blocker — more than WAL itself.
- The schema's append-only, densely-keyed pack region is a **major asset** that
  the roadmap has not yet exploited. It makes object-level replication,
  cheaply, a consequence of existing design rather than a new mechanism.
- Three different "VFS" projects are hiding in one phrase. Do the object-backend
  seam first, the hosted endpoint second, the replication third, and the
  `layerfs` SQLite VFS last.
- **Do nothing to the benchmark policy now** except name the seams, price the
  current choice, and keep the decision reversible.

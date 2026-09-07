# Storage architecture v3: implementation and rollout plan

Status: **draft implementation plan**, 2026-09-08. Based on the specification at
`555d91f0cd74148364331e24acf0ba14408d7c78` and source baseline
`28177560c8f049c02192e18c263cdc5543c1ab52`. This document plans work; it does not
implement product code, create a Store, run a smoke or authorize a migration.
Recheck actual source/PR heads before implementation and preserve unrelated edits.

**Owner verification boundary: use only the three agreed development smokes.**
Do not add/run unit, property, fuzz, race, crash, all-workspace or all-family suites
as part of this iteration. Building the exact host binaries/runtime image required
by a smoke is preparation, not an additional test campaign. Read-only source review,
compiler feedback and documentation checks support the work but are not substitute
smoke evidence. This limited iteration does not claim full release qualification or
silently change historical benchmark contracts.

Authority: [architecture v3](storage-architecture-spec.md), [physical format](sqlite-storage-format.md),
[owner boundary](storage-efficiency-boundary.md), and [finding disposition](review-disposition.md).
The [agreed smoke plan in PR #80](https://github.com/Ephemeral-AI-Lab/layerfs/blob/d9ec9c6714ca31adb7a337d2ac0f40976513908c/docs/roadmap/0.1/0.1.4/storage-smoke-test-plan.md)
owns fixture/lifecycle/topology decisions and its remaining prerequisites. Full
benchmark family and numerical qualification gates remain separate.

## 1. Mental model and invariants

```text
Immutable Layer/Commit root
  -> namespace/inode objects
  -> FileState and extent trees
  -> payload ObjectIds and slices

Mutable Workspace: pinned root + private COW state + temporary spool
  -> freeze/capture
  -> shared canonical construction (CAS + CDC + COW)
  -> exact reuse, then new FULL or depth-one DELTA records
  -> RAW or independently compressed groups
  -> immutable SQLite pack BLOBs + selected-object index
  -> existing operation-specific publication and finalization
```

The logical graph remains complete and authenticated. A physical delta is only an
encoding of a canonical object; it is not a Workspace edit or Commit definition.
Accurate localized edits keep old slices and affected-tree-path construction. A
whole-file rewrite discovers reuse and passes predecessor hints without pretending
the write was a known byte-level diff. Small/large files use one path; CDC minimum
is not allocation padding, and growing past it relocates no historical objects.

Init and Workspace Commit share physical construction/admission, not lifecycle
state machines. Preserve Init naming/publication, committed Workspace staging,
expected-head/base and no-change checks, pending-publication state, checkpoint
installation, spool retirement and resume. Required encoding and finalization
finish before public success. Multiple bounded transactions remain allowed.

All authoritative representations, locators, roots and history stay in SQLite.
ObjectIds are independent of local pack IDs. Full bases already exist and remain
available; no deep chains, later packer, global similarity index, separate small-file
backend, cloud service or new durability/crash contract is part of this work.

## 2. Read before changing code

Read applicable `AGENTS.md` in the actual worktree. The links below are existing
files unless explicitly identified as pinned historical material. Follow callers
and the affected helper bodies, including sibling paths; names alone are not proof
that a method batches, releases a lock or preserves predecessor information.

### Design and execution custody

1. [Architecture](storage-architecture-spec.md), especially admission, complexity/
   batching, predecessor handoff, all-Init ownership and reconciliation scope.
2. [Exact proposed format](sqlite-storage-format.md), including pre-allocation limits,
   the 65,528-65,536 RAW/DELTA overlap, and canonical versus admission byte limits.
3. [Boundary](storage-efficiency-boundary.md), [closure matrix](review-disposition.md),
   [evidence index](evidence.md), and [0.1 compatibility rule](../README.md#compatibility-boundary).
4. [PR #80 smoke plan](https://github.com/Ephemeral-AI-Lab/layerfs/blob/d9ec9c6714ca31adb7a337d2ac0f40976513908c/docs/roadmap/0.1/0.1.4/storage-smoke-test-plan.md),
   [benchmark hosting rules](../../../../benchmark/AGENTS.md),
   [general benchmark rules](../../../general/benchmark_rules.md), and
   [runner quick start](../../../../benchmark/fs-bench-pro/QUICKSTART.md).
   The owner's smoke-only verification instruction controls this iteration; do not
   turn the quick start's full-checkpoint instructions into a new task.

### Store ownership, schema and readers

| Read | Trace / reason |
| --- | --- |
| [objects.rs](../../../../crates/layerfs-layerstack-store/src/objects.rs) | Canonical/checked owners, finalized slabs, candidate planning, all admission variants, private spools, seen/offset indices and IdOrder; read the actual scalar callers too |
| [schema.rs](../../../../crates/layerfs-layerstack-store/src/schema.rs) | Exact schema dispatch, shared connection mutex, FIFO permit, branch leases and existing error behavior |
| [layerstack.rs](../../../../crates/layerfs-layerstack-store/src/layerstack.rs) | Empty/nonempty Init, parallel/serial/fallback construction, parent merges, four cleanup callers and final publication |
| [workspace.rs](../../../../crates/layerfs-layerstack-store/src/workspace.rs) | Direct/Workspace candidate publishers, SnapshotReader scalar/batch cache and authentication ownership |
| [staging.rs](../../../../crates/layerfs-layerstack-store/src/staging.rs), [branch.rs](../../../../crates/layerfs-layerstack-store/src/branch.rs) | Stage persistence/discard and sibling metadata permit scopes; prevent nested acquisition |
| [statements.rs](../../../../crates/layerfs-layerstack-store/src/statements.rs), [v5.sql](../../../../crates/layerfs-layerstack-store/sql/schema/v5.sql) | Statement inventory, FK targets, schema version and raw bytes assumptions; follow included object/query SQL |
| [query.rs](../../../../crates/layerfs-layerstack-store/src/query.rs), [records.rs](../../../../crates/layerfs-layerstack-store/src/records.rs), [telemetry.rs](../../../../crates/layerfs-layerstack-store/src/telemetry.rs) | Canonical storage counts, reachable-object lookup and receipts; do not relabel canonical bytes as compressed bytes |
| [Store Cargo.toml](../../../../crates/layerfs-layerstack-store/Cargo.toml), [workspace Cargo.toml](../../../../Cargo.toml), [Cargo.lock](../../../../Cargo.lock) | Existing dependencies/features; select one compatible Zstandard binding and BLOB/limit APIs without a backend framework |

### Construction, mutation and runtime callers

| Read | Trace / reason |
| --- | --- |
| [rope/build.rs](../../../../crates/layerfs-content/src/file/rope/build.rs), [rope/edit.rs](../../../../crates/layerfs-content/src/file/rope/edit.rs) | Full/replacement CDC callbacks and file-logical offsets, retained COW slices and canonical output ownership |
| [rope/read.rs](../../../../crates/layerfs-content/src/file/rope/read.rs), [rope/diff.rs](../../../../crates/layerfs-content/src/file/rope/diff.rs), [rope/state.rs](../../../../crates/layerfs-content/src/file/rope/state.rs) | Existing traversal primitives, touched-node reads and private forward-cursor placement; do not restart a whole visitor per chunk |
| [extent_codec.rs](../../../../crates/layerfs-content/src/file/extent_codec.rs), [limits.rs](../../../../crates/layerfs-content/src/limits.rs), [object/access.rs](../../../../crates/layerfs-content/src/object/access.rs) | Preserve canonical encoding/profile and distinguish trusted authentication from fresh source reads |
| [workspace-core/file_edit.rs](../../../../crates/layerfs-workspace-core/src/file_edit.rs), [namespace.rs](../../../../crates/layerfs-workspace-core/src/namespace.rs), [lib.rs](../../../../crates/layerfs-workspace-core/src/lib.rs) | Truncate preserves Edited.base; rename retains source inode and replaces bindings; no logical predecessor substitution |
| [changes.rs](../../../../crates/layerfs-workspace/src/changes.rs), [capture.rs](../../../../crates/layerfs-workspace/src/capture.rs) | Once-per-file task preparation, same-inode/path predecessor, complete-build fallback and captured-output first-span handoff |
| [filesystem/resolve.rs](../../../../crates/layerfs-content/src/filesystem/resolve.rs) | Distinguish authenticated absent entries from missing required objects in optional predecessor lookup |
| [lifecycle.rs](../../../../crates/layerfs-workspace/src/lifecycle.rs), [live_backing.rs](../../../../crates/layerfs-workspace/src/live_backing.rs) | Freeze/capture, finalization, installed checkpoint facts, post-publication failure and spool lifetime |
| [reconcile.rs](../../../../crates/layerfs-workspace/src/reconcile.rs) and fingerprint helpers in changes.rs | Shared lazy invalidation view, exact/ancestor/prefix selection and actual backing-byte fingerprints; overlapping scopes remain a limit |
| [FUSE live_owner.rs](../../../../crates/layerfs-fuse/src/live_owner.rs), [filesystem.rs](../../../../crates/layerfs-fuse/src/filesystem.rs) | Ordinary live write/rename/freeze callers; use source context rather than rewriting daemon/FUSE semantics |
| [SDK client.rs](../../../../crates/layerfs-sdk/src/client.rs) | Real Init, Exec, SDK edit and Commit surfaces used by the smoke coordinator |

### Smoke infrastructure and original import semantics

Read the existing host coordinator [main.rs](../../../../benchmark/fs-bench-pro/src/main.rs),
[Cargo.toml](../../../../benchmark/fs-bench-pro/Cargo.toml),
[runner.py](../../../../benchmark/fs-bench-pro/shared/runner.py), and
[runtime.py](../../../../benchmark/fs-bench-pro/shared/runtime.py).
The existing `--smoke` flag does not automatically select PR #80's three new smokes.

Read the frozen [manifest](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/docs/roadmap/0.1/0.1.4/deepseek-history/checkpoint-manifest.json)
and [DeepSeek helper](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/benchmark/fs-bench-pro/shared/deepseek_history.py)
from the pinned experiment commit if absent from the active checkout. Reuse preparation,
import and oracle logic with current public/runtime ownership; do not check out and
run historical product binaries to bypass current hosting or format rules. Preserve
ordinary truncate/rewrite and tempfile/rename behavior of the real importer.

## 3. Resulting file/folder structure

This is the **planned final structure**, not files created by this documentation
change. New names below are concrete private implementation locations, not new
public SDK APIs or frozen benchmark scenario identifiers. Move existing code into
these owners only where it serves the stated responsibility; do not scaffold an
empty module for a later phase or split every helper into its own file.

```text
crates/layerfs-layerstack-store/
├── Cargo.toml                         change: one codec binding / needed SQLite features
├── src/
│   ├── objects.rs                     keep: canonical construction facade and shared types
│   ├── objects/                       new private submodules of existing objects module
│   │   ├── pack.rs                    wire framing, codec and bounded delta matching
│   │   ├── admission.rs               one accumulator, final recheck, bulk insertion
│   │   ├── read.rs                    locator/group waves, reconstruction/authentication
│   │   └── spill.rs                   moved private spool, seen/offset index and ID-order code
│   ├── schema.rs                      exact format dispatch + existing gate's FIFO handoff
│   ├── layerstack.rs                  all Init producers share the output sink
│   ├── workspace.rs                   candidate publication + existing SnapshotReader cache
│   ├── staging.rs / branch.rs         keep lifecycle; audit permits, no new manager
│   ├── query.rs / records.rs           canonical accounting and locator-based queries
│   ├── telemetry.rs / statements.rs    extend existing receipts and SQL inventory
│   └── lib.rs                         preserve outward facade; private module wiring
└── sql/
    ├── schema/v6.sql                  proposed new schema, only after compatibility decision
    ├── schema/v4.sql / v5.sql          retain historical/reference definitions unchanged
    ├── objects/get*.sql               selected-locator queries replacing raw payload retrieval
    ├── objects/membership_128.sql      canonical_length, not length(pack bytes)
    ├── objects/page.sql               preserve page semantics through selected index
    ├── query/*.sql                    remove raw bytes-column assumptions
    └── workspace/ branch/ layerstack/ existing logical FK/publication responsibilities

crates/layerfs-content/src/file/rope/
├── build.rs / edit.rs                 emitted first-span and predecessor-context handoff
├── read.rs / state.rs / mod.rs        existing traversal and module wiring
└── cursor.rs                         new private forward extent traversal if not clean in read.rs

crates/layerfs-workspace/src/
├── changes.rs                        per-file predecessor, complete/captured output and fingerprints
├── capture.rs                        first-span facts at original emission
├── lifecycle.rs / live_backing.rs     publication-bound checkpoint facts and finalization
└── reconcile.rs                      one invocation-local invalidation view

benchmark/fs-bench-pro/
├── src/main.rs                       thin dispatch in existing macOS host coordinator
├── src/storage_smoke.rs              proposed smoke SDK flows + retained-state oracle use
└── shared/
    ├── runner.py / runtime.py         reuse lock, preparation, managed Docker and collection
    └── storage_smoke.py               only the needed plan-specific orchestration/import adaptation

Cargo.toml / Cargo.lock               pin dependency changes once the codec adapter is selected
```

`pack.rs` is pure byte/codec work: no Store locks or publication calls. `read.rs`
owns connection-only extraction; `admission.rs` owns the admission permit and uses
reader/codec functions without a reverse dependency. `spill.rs` remains private
temporary ownership, not another durable backend. Keep canonical codecs unchanged.
`cursor.rs` may be folded into the existing read traversal if equally clear; its
responsibility cannot disappear into per-chunk root rescans.

Do not add another crate, plugin trait, registry, persistent group cache, allocator
service, global similarity index, migration framework or separate Init/Commit encoder.
The Python smoke adapter is a small client of the existing runner/runtime, not a new
harness. Fold it into the existing owner if no distinct orchestration is needed.
No new `families/` qualification matrix or new `tests/` tree is part of this plan.

## 4. Change and deletion areas

| Area / owner | Required final change | Delete or avoid |
| --- | --- | --- |
| Physical objects: pack/read/admission | Exact schema/format, grouped reads, <=2 membership visits per candidate, one final permit and bounded bulk SQL | Raw object-row get/insert/equality choreography once packed creation is enabled; no permanent dual writer |
| Coordination: schema + all publishers | Targeted FIFO successor grant; all writer callers agree on permit ownership; connection released for validation | Broadcast-per-release fanout, nested permits and whole-Init admission exclusion |
| Init: layerstack + finalized output | Move authenticated producer ownership directly into shared accumulator for empty/nonempty/serial/fallback inputs | Parent merge_prevalidated payload cloning and all four whole-Store cleanup calls; no occupancy encoder fork |
| Temporary data: spill owner | Page seen/offset insertion and membership, absolute-location union, pending visibility, buffered sealed ID I/O | Per-ID SQL/implicit transactions, tiny field reads/per-record index-building seeks; no claimed fsync saving |
| Construction hints: content + Workspace | Same-inode predecessor; physical-only pinned-path fallback; first per-file spans and forward cursor | Dropped complete-build context, second source/CDC pass for hints, logical inode substitution or alias-times-chunk scans |
| Reads/finalization: SnapshotReader + live backing | Internal group/base batching, honest output memory, carried root-bound checkpoint facts | Temporary cache-argument clone, redundant trusted rehash and needless namespace readback; retain required installation |
| Reconciliation: existing owners | Lazy shared view and indexed exact/ancestor/prefix selection using existing fingerprint bytes | Per-conflict full-manifest rebuild/unrelated scans; do not claim overlapping exact fingerprint work solved |
| Queries/receipts: query + SQL/telemetry | Preserve canonical/public meanings; separately account physical packs, indices and unused records | Canonical_length inferred from compressed size; whole census or raw SQL tracing inside an operation timer |
| Smokes: existing benchmark owners | Only PR #80's three public-path flows and their observation/verification scopes | Broad unit/fuzz/race/recovery suites, inherited full-family campaigns, altered import operations or production scenario branches |

Keep existing public symbols where needed by real callers; moving private code is
not permission to remove SDK behavior. Retired helper-specific source references
must move or disappear with their owner rather than leave duplicate production
implementations. Historical SQL/report artifacts are not an active legacy writer.

## 5. Implementation milestones and smoke selection

Each milestone is a coherent change on an isolated implementation branch/worktree.
Read its affected callers before editing, build matching host/runtime artifacts,
then execute only the relevant agreed smoke. Do not make a new raw admission system
solely to delete it in the next milestone. Keep each checkpoint attributable and
intermediate limitations explicit.

| Milestone | Implement / resulting checkpoint | Executable verification only |
| --- | --- | --- |
| 0. Policy and smoke prerequisites | Record compatibility/release choice; finish only PR #80's outstanding fixtures, budgets, entrypoints and comparison rules. Wire the three smoke selectors into existing infrastructure and preserve the exact five manifest entries/import semantics. Record the unchanged current-code baseline. | The three agreed smokes once under their finalized development procedure; current raw baseline reports pack/delta coverage not applicable |
| 1. Reusable ownership and spill preparation | Move real private owners, batch seen/offset scratch paths and ID I/O, carry location facts. Preserve current placement/publication until the complete replacement is ready. No new codec trait or temporary shared raw engine. | Small-file Init/readback and first-five replay for affected construction/spill paths; localized-edit smoke only if its handoff changes |
| 2. Complete FULL/RAW packed vertical slice | Schema dispatch and explicit fresh packed creation; FULL framing and bounded reads; shared admission and bulk SQL; all Init/candidate publishers; canonical accounting and required finalization. Switch ownership/gates and delete unsafe cleanup together. | All three smokes: Init, changed/no-change Commit, current/historical mounted readback and truthful intermediate allocation |
| 3. Group compression | Add both Zstandard write/read support with specified framing/window/output and scratch rules. Coalesce group waves and count bytes/copies. | Small-file/readback and first-five replay; edit smoke when its capture/Commit/read path is affected |
| 4. Whole-file correspondence and shallow deltas | Carry prior-file/span context through complete and captured paths; implement one old cursor and bounded matcher. Enable delta emission only when FULL-base selection, reads and physical closure handling are complete. | Frequent-edit smoke and first-five ordinary import replay; use their declared representation observations, never forced SDK substitutions |
| 5. Final cleanup and integrated candidate | Finish checkpoint fact reuse, trusted read-copy/hash removal, invocation-local reconciliation view and named deletion ledger. Remove intermediate raw-only policy/dead paths. | Affected smoke while fixing; all three on the final source/binary/image candidate before calling this implementation iteration complete |

Milestone 2 is the indivisible switch for: every object writer using the new final
membership/equality owner; removal of outer/nested Init/direct/Workspace/callback
permits; removal of stale all-missing and all four cleanup assumptions; correct
stage/head/finalization behavior. Changing only half can deadlock or delete another
operation's admitted data. Keep sibling Fork/Add/stage-discard metadata scopes small.

FULL/RAW-only and compression-without-delta checkpoints are explicitly incomplete
development milestones, not release-ready support for every selected wire variant.
Use fresh independently created Stores and exact candidate identities. A known but
not-yet-implemented variant fails explicitly; never reinterpret it as RAW. Before
final rollout the implementation supports the full selected reader contract and
uses the required compression/selection policy. No runtime fallback or public flag
is added merely to preserve intermediate implementations.

Reconciliation's shared view is a prescribed source change, but the three smokes
are not a conflict matrix. Review its exact byte/ancestor/alias semantics against
its callers and state its executable coverage limitation in the result; do not
add a fourth reconciliation suite or claim a smoke proved all invalidation paths.
Likewise, ordinary smokes do not establish every FIFO/race schedule or malformed-
record failure. These are evidence limits, not permission to weaken implementation
invariants or silently describe uncovered behavior as verified.

## 6. Smoke-only iteration loop

```text
one coherent milestone/change
  -> build exact host binary and managed runtime image as needed
  -> smallest affected agreed smoke
  -> inspect correctness, actual route, allocation, elapsed and cleanup
  -> fix, keep, or revert the change
  -> retain source/input/result identities and failed observations
  -> all three smokes on the final integrated candidate
```

Use PR #80's host SQLite/SDK/coordinator/spool plus managed Docker daemon/live core
and real FUSE. Reuse the shared measurement lock, limits, prepared immutable inputs
and compatible builds. A source change invalidates matching binary/image seals as
required by existing infrastructure. Start each history with independent mutable
state; the DeepSeek smoke starts empty and uses the first **FIVE manifest entries**,
not five arbitrary Git commits, the full 157-history, or an already-completed Store.

The implementation agent must first finalize the still-open Smoke 2/3 fixtures and
execution prerequisites within that existing plan, before collecting baseline or
candidate observations. Keep SDK range edits and ordinary Exec/FUSE replacement
surfaces separate. Include the relevant ordinary replacement routes within that
agreed smoke scope; do not change the DeepSeek importer to manufacture better hints.
This plan assigns no new fixture population, thresholds, matrix or scenario IDs.

Use the smokes' existing independent file/tree/mode/symlink and historical-state
checks. Measure retained allocation before verifier-created records/projections;
keep verification and expensive accounting outside operation timers. Preserve
required output drain, Commit result, End/reopen and cleanup boundaries. No direct
SQL injection or private product API is a substitute for public smoke operations.

Stop on the first unresolved correctness, route/authenticity or cleanup failure;
retain the last acknowledged state and diagnostics. Correct the smallest shared
cause and rerun the affected smoke on fresh mutable state. Keep valid slow/regressing
results. Do not repeatedly rerun unrelated passing smokes for nicer numbers. After
all final-candidate smokes pass, stop; no full checkpoint, 157-state replay, unit-test
suite or additional benchmark is automatically triggered by this plan.

For each retained smoke observation report: exact source/binary/image/input IDs;
public surface and lifecycle; correctness/representation applicability; allocated,
canonical and physical bytes where available; operation/finalization/wall scopes;
host/container resources and temporary space; cleanup and limitations. Reuse compact
receipts, including query/page/group counts if implemented, rather than add per-object
logging or a complete object census to timed work. Unavailable counters are unavailable,
not zero or proof of linear behavior. Git is not a default smoke arm; any later
reference needs its own matched five-state scope under PR #80, not a 157-state divisor.

## 7. Rollout, rollback and completion boundary

1. **Compatibility first.** The owner must choose the new-Store-only transition,
   narrow 0.1 exception versus 0.2 placement, and any required legacy support. Keep
   current Store files unchanged. Implementation begins only under that explicit
   disposition; this plan does not grant an exception or conversion authority.
2. **Development isolation.** Keep milestone commits and fresh smoke Stores separate
   from user Stores. Restore code by selecting a known commit if a milestone fails;
   do not reset unrelated edits or attempt a reverse/in-place database conversion.
   Never open a new-format Store with a legacy reader and hope for fallback.
3. **Final candidate.** One shared physical encoder/admission/reader remains; all
   required synchronous work is included. Check the deletion ledger and source
   ownership, then retain the final three-smoke results and their limits.
4. **Owner review.** Report the final implementation revision, changed modules,
   executed smoke identities/results, baseline comparison, unmeasured/uncovered
   behavior and compatibility status. A smoke-only implementation checkpoint does
   not authorize broad rollout, migration, PR merge or full release qualification.

No performance ratio, hard latency ceiling, Git-closeness tolerance or aggregate
capacity is invented here. The architecture requires linear admission visits,
indexed/batched operations and bounded byte work; smoke observations establish only
what those exact runs exercised. Exact overlapping reconciliation fingerprints,
all race schedules, arbitrary malformed inputs and wider load remain outside this
iteration's executable evidence. Cloud transport/services, crash recovery, new
fsync policy and in-place migration remain excluded.

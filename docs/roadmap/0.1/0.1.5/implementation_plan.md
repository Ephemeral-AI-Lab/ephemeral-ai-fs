# v0.1.5 implementation plan

> **Status:** Reconciled 2026-09-09. Implement [spec.md](spec.md) completely.
> Verification in this task is only the fixed ten-file/thirty-commit FUSE smoke
> with its fixture self-check and 31-state history verification. No broad tests,
> release campaign, settings experiments or repetition of unaffected passes.

## Resulting file responsibilities

Start from release `101fa273d815f3aaedb0e06ba0de7b0777d83def`. Paths below refer to
that baseline, not the older main checkout. Reuse existing modules; add only the
two product modules that separate canonical content from physical encoding.

```text
crates/
  layerfs-content/src/
    file/content.rs                NEW: SmallContent role + common file-root dispatch
    file/mod.rs                    export the common regular-file entrypoints
    object/references.rs           SmallContent has no canonical children
    filesystem/read.rs             regular reads dispatch SmallContent/FileState
    filesystem/apply.rs            bounded small edits / existing large extent edits
    filesystem/reconcile.rs        representation-aware equality and reconciliation
    file/rope/                     retain extent algorithms and old formats
  layerfs-layerstack-store/src/
    objects/delta.rs                NEW: bounded small FULL/DELTA construction + decode
    objects/pack.rs                 pack-v3 single-object groups, strict validation
    objects/read.rs                 acquire/decode/authenticate targets and FULL bases
    objects/admission.rs            exact CAS selection, packing and base ordering
    objects/diagnostic.rs           new role/physical closure and byte accounting
    objects/spill.rs                support object bounds; preserve 64-KiB ordered read-ahead
    objects.rs                     shared builders/owned output and budget capability
    layerstack.rs                  Init input adapter to shared construction
    workspace.rs                   stage/publish and dependency lifecycle
    schema.rs, schema/compatibility.rs, statements.rs
                                   schema-8 fence, retain old readers/statements
    store.rs                       explicit offline upgrade_format(path)
  layerfs-layerstack-store/sql/schema/
    v6.sql, v7.sql                 preserved supported schemas
    v8.sql                         NEW: same seven tables, new format capability
    migrate_v7_to_v8.sql            NEW: explicit transactional version promotion
  layerfs-workspace/src/
    changes.rs                     frozen facts/predecessor -> shared small builder
    cow_tree.rs                    representation-aware materialization
    backing/read/checkpoint callers
                                   route actual regular-file roots through dispatch
  layerfs-workspace-core/           retain live pieces, POSIX and ordering semantics
  layerfs-fuse/                     retain transport; adapt shared content consumers only
benchmark/fs-bench-pro/
  families/small_file_delta_smoke/
    README.md, perf.sh, verify.sh   NEW: one case, thin wrappers over existing runner
  shared/small_file_delta_fixture.py
                                   NEW: ten files, thirty edits, independent oracles
  shared/runner.py, storage_smoke.py
                                   register/dispatch new case; reuse lifecycle
  src/storage_smoke.rs             existing importer/Created/history verification route
  families/tiny_rewrite_history/   unchanged historical case
  shared/tiny_history_*.py         unchanged fixture identity; reuse generic helpers
```

The tree names responsibilities, not permission to add placeholder files. Put
one-case dispatch in the existing smoke registry; no new benchmark engine, Store,
codec interface, global base service, or `whole_file.rs`. Existing backing/checkpoint
filenames must be found from actual callers rather than invented from this outline.

## Phase 0: bind the control and make the loop cheap

1. Preserve current and other-task changes. Use an isolated `codex/` worktree based
   on v0.1.4 if needed; reuse an appropriate existing implementation worktree.
2. Read workflow/spec, past mistakes and the released report. Freeze the one
   [smoke fixture](delta-encoding-benchmarks.md#first-round-execution-scope), add its
   minimal harness plumbing, and record source/harness/fixture/image identities.
3. On unchanged released product code, run one smoke performance sequence and
   verify all 31 states from that same Store. Keep this control separate from the
   old three-file R26 output. Carry identical harness and fixture artifacts to the
   candidate. The local smoke is owner-authorized exploratory work; it is not
   blocked on a new admission issue or a full benchmark registry campaign.
4. Reuse matching builds, prepared inputs and images. Record build/setup/run/
   verification wall time separately so orchestration overhead is visible.

## Phase 1: implement common canonical file handling

Add the `LFS5SML\0` Bytes role and direct regular inode content-root representation
from spec section 2. New nonempty small files use it; empties keep the old compact
representation. Old FileState/chunk codecs and IDs stay readable and unchanged.

Implement shared regular-file inspect/length/read-range/stream/build dispatch.
Search every caller assuming a regular inode root is a `FileStateRoot`, calling
`rope::read_state`, reading all extents, or deriving file size from an extent
summary. Route those assumptions through common dispatch; retain actual rope
functions for chunked content and metadata. Include filesystem read/apply/reconcile,
workspace cow materialization/changes, host backing, frozen facts, checkpoint
installation and diagnostics. Do not make each caller implement its own decoder.

For explicit small edits, prepare bounded final bytes once at capture/construction.
For large known edits keep replacement-only extent work. Wire final-size
transitions and unchanged-root preservation before exposing new roots to readers.
Carry Store format capability once through the operation so schema-6/7 writes
continue using their supported old construction until explicit upgrade.

## Phase 2: implement bounded physical small-object storage

Implement `objects/delta.rs` with fixed Zstandard level 3/windowLog 18/workers 0,
checksum/content size on and dictionary ID off. Implement v3 FULL and DELTA groups
exactly as specified: one complete SmallContent object per group, ordinal zero,
one optional FULL base ID, no member directory/chunk-base list.

Resolve existing target reuse first. Select at most one FULL anchor from the
known predecessor or its direct delta base. Prepare FULL once and DELTA at most
once; select strictly smaller complete encoded cost, with FULL winning ties.
Reuse operands and fallback output. Serialize physical FULL/DELTA encoding at
the admission consumer for Init and Commit; parallel producers do not each own
a codec context. Keep codec/base work outside Store/live locks
under valid ownership; enforce static workspace/operand budgets with fixed
parameters, not a compression/window sweep.

Extend existing pack dispatch/read/admission/physical-dependency traversal and
accounting. Validate bounds before allocation, exact frame EOF and identity,
selected base role/FULL encoding, and late duplicate handling. Use stable pending
or selected locations; preserve base closure through staging, rollback and retained
history. A new frame/group never implies its own SQL transaction or transport call.

## Phase 3: integrate both public paths and compatibility

Use the common producer seam instead of separate Init and Commit implementations:

| Released seam | Change |
| --- | --- |
| `objects.rs:344`, `run_finalized_output` | Keep shared bounded producer delivery. |
| `objects.rs:646`, `FinalizedOutputWriter::build_complete_file` | Apply canonical size dispatch once; carry ready output. |
| `layerstack.rs`, checked Init admission | Native readers/metadata enter the same policy and admission. |
| `changes.rs:334/1232`, candidate/produce_file | Frozen dirty facts and known predecessors enter shared preparation. |
| `objects.rs:3847/3942`, workspace construction/owned-page consumption | Reuse missing-object preparation and common publication. |
| `objects.rs:3912`, `admit_remaining` | Close pending admission transaction before staging. |
| `workspace.rs:488`, staging; lifecycle commit/checkpoint callers | Preserve conditional publication and post-publication outcome reporting. |

Line numbers are release anchors, not promises about post-edit positions. Preserve
#95 exact bounded Init comparison reuse, #98 Workspace coalescing with comparison
reuse zero, and graph-order spill read-ahead capped at 64 KiB. Preserve live FUSE/
SDK inode, alias, handle, cache and operation-cut semantics. New format data stays
host-owned; do not add per-file base-search negotiation to the daemon protocol.

Create schema 8 using the existing seven tables. Implement nonpromoting old-store
open behavior and explicit offline schema-7 upgrade with exclusive revalidation,
transactional version change and accurate failure reporting. Preserve old readers,
page layouts, and immutable histories. No automatic conversion or downgrade.
Update format/API documentation for the implementation and its limitations.

## Phase 4: smoke-only verification and performance iteration

Build the needed product/harness targets and run only
`small_file_delta_smoke / small-file-delta-10x30-v1`. Its fixture self-check and
verification belong to this same smoke. Verify the **same retained performance
Store**, after allocation observations are frozen, through a new coordinator
connection. Require 30 Created commits and exact path/mode/length/byte oracles for
all 31 states, plus successful cleanup. Observe that the candidate actually
produced new small objects and DELTA records; all-FULL/old-chunk fallback cannot
masquerade as exercising delta history reconstruction.

Report baseline/candidate initial/final allocated bytes, growth, per-step save
and Commit times, paired median/range, and available operation/resource costs.
Preserve failures and fix causes rather than changing the fixture or timer.
The ten-file case has no inherited three-file absolute storage/latency gate;
report improvement or regression honestly and optimize identified slowness.
Do not claim a numerical performance PASS from unfrozen thresholds.

Do not run Cargo test, Clippy, doctest, format-test suites, separate codec or
migration/failure/threshold suites, old tiny reruns, SDK/FUSE matrices, 56-case
families or full157. Production validation/error handling must still be complete;
record unexercised corner cases as later qualification. Read-only source review
and compilation are allowed and do not expand the verification workload.

After a meaningful change, rebuild only affected artifacts and rerun this smoke
when that change invalidates its last result. Do not rerun a passed smoke after
documentation-only edits or repeatedly sample an unchanged candidate for a better
number. A final result must correspond to the final relevant product/harness seal.

## Fast build and harness rules

The existing host build entrypoint runs
`cargo +1.85.1 build --locked --release -j2 -p fs-benchmark-pro` and records binary
identity in the repository's `target/release`. Use the runner, locked dependencies,
and existing target/BuildKit caches. Do not create a fresh target directory for
each phase, run `cargo clean`, prune Docker caches, update dependencies, or rebuild
for Markdown-only changes.

Use the existing `--build-storage-smoke-image` lane; it avoids the extra executable
workload self-check while retaining compilation and provenance. The Docker build
layer currently cleans product packages internally: avoid repeatedly invoking it
for unchanged inputs. If that clean step causes a measured bottleneck, fix the
invalidation/cache cause while retaining truthful source/binary/image identity;
do not remove custody checks or relabel stale binaries as current.

A host-only change may reuse a Linux image only through an accepted explicit
compatibility record. New canonical/read code affects shared consumers, so expect
a matching host and Linux build for those changes. Never forge a source seal.

When a build/smoke phase is slow, inspect whether the cause is compilation,
linking, lock contention, repeated preparation, container startup/transfer,
per-state process launch, or actual product work. Fix the shared cause and reuse
completed work. Avoid redundant `cargo check` followed immediately by the same
required release build. Register and preserve this smoke's fixed 600-second phase and 30-second
operation watchdogs; a slow phase is diagnosed, not hidden by increasing timeouts.

All resource-sensitive builds, smoke runs and verification are serial under the
existing measurement lock. Self-locking runner commands must not be nested under
a second acquisition. Respect another owner's run and avoid busy polling.

## Completion report

Deliver full code and documentation, a source-matched smoke control/candidate,
31-state verification/cleanup receipts, and a concise storage/latency/build-time
comparison. Name remaining slowness and all unrun qualification honestly. Do not
publish/tag a release or mark the historical v0.1.4 optimization backlog complete.

Broader verification is deferred, not deleted: malformed data/base lifetime,
upgrade interruption, old/new reads, all boundary transitions, POSIX concurrency,
large-file locality and full-family release evidence remain later obligations.
Use [benchmark success](benchmark_success.md), [delta benchmark plan](delta-encoding-benchmarks.md)
and [past mistakes](past_mistake.md) to retain those requirements without running
them in this implementation task.

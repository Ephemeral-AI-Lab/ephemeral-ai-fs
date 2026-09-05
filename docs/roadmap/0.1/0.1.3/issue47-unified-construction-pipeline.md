# Issue 47: one construction and persistence pipeline, different inputs

Design amendment, 2026-09-06, for [#47](https://github.com/Ephemeral-AI-Lab/layerfs/issues/47), under [#46](https://github.com/Ephemeral-AI-Lab/layerfs/issues/46) and [#39](https://github.com/Ephemeral-AI-Lab/layerfs/issues/39). Consolidates three completed read-only audits and two follow-up reviews. This is a plan, not a performance PASS; later source-bound results supersede the observations below.

## Current progress and evidence

Implementation owner: task `01a07283-4940-7ba1-9b4c-548ddc459e35`, worktree `/Users/yifanxu/.codex/worktrees/7311/layerfs`.

- `46463287c`: ordinary in-place checkpoint, ordered final inode processing, paged deletion/reference reads, shared checked admission and unseen-alias fixes are recorded.
- `8cbbb425b`: shared physical spool segments committed locally. Written ranges/read plans retain descriptors; ordinary per-node spool ownership and retirement methods are removed. The owner reports 51 Workspace tests, targeted lifetime/accounting regressions, 12 SDK file-edit integration tests and host build passed. The shared-spool performance sample is pending at review time.
- Separate Exec task retains FUSE/transport ownership. Keep builds, tests and measurements serialized through the existing host-wide lock. Do not edit its files or clean its artifacts.

Latest pre-shared-spool receipts in the owner's `benchmark-results/host-store/results/`:

| Receipt | Exec | Commit | Complete product |
|---|---:|---:|---:|
| `issue47-ordered-create100` | 15.338 s | 3.562 s | 18.941 s |
| `issue47-ordered-delete100` | 3.580 s | 0.460 s | 4.062 s |

Create Commit: content-labelled phase 597.276 ms, namespace 318.532 ms, candidate finish 91.944 ms, admission 431.738 ms, checkpoint 2,117.269 ms including 2,079.293 ms retirement. The content-labelled timer includes some directory/metadata/record work, not just parallelizable file construction. Preserve exact identities in the owner's append-only `issue47-subsecond-workspace-results.md`; these different-revision observations are not final qualification.

## Shared architecture

Create, delete, edit, rename and link already belong to one ordinary Commit flow. Initialization and Commit should share the actual construction/delivery/persistence implementation, with different input and lifecycle boundaries. Do not add benchmark-name, size, density or family policies selecting engines.

```text
Prepared native files                 Stable Workspace delta + base roots
         |                                           |
Native discovery                         Resolve changed inode inputs once
         |                                           |
         +---------------------+---------------------+
                               v
                 Existing canonical content builders
                   bounded independent producers
                               |
                      finalized owned output
                               v
                    bounded queue/backpressure
                               v
                    shared checked persistence
                               ^
                               |
                  ordered namespace finalization
                  existing directory/inode builders
                               |
                               v
                  lifecycle-specific publication
                   /                       \
          initial project           next Commit + small
                                    existing-node checkpoint
```

Finality must be established before each direct output handoff; the diagram is not permission to persist provisional objects. One coordinator combines content results with final metadata/reference facts. Deletion emits little or no file content and uses the same ordered finalization/persistence machinery.

macOS already owns Workspace bytes, canonical construction and SQLite. Docker owns workload/daemon/FUSE. Writes normally cross to host during Exec; live-FUSE Commit does not re-upload the workspace. Future buffering must account for drain costs across the full lifecycle. Host SQLite only, no data mounts, existing container limits and separate host resource reporting remain unchanged.

## Concrete reuse and responsibility transfer

| Files / mechanism | Transfer |
|---|---|
| Workspace `changes.rs`, `capture.rs`, existing file readers | Freeze each changed inode's generation, length and retained source description. Shared-spool inputs retain segment+offset+length and use positional reads; mixed edits retain PieceTree/base-root semantics. Do not send mutable Workspace state to workers. |
| Store `layerstack.rs` | Adapt native discovery to the shared producer driver. Keep native source identity checks, initial identities and initial publication with this caller. |
| Store `objects.rs` producer/slab machinery | Reuse bounded task claiming, owned slabs, channel backpressure, drain/join-on-error behavior. Extract the smallest seam used by both callers, not a generic scheduler framework. |
| Content `rope::build`, `FileMutationBatch`, PieceTree | Keep existing content/extent algorithms and incremental reuse. No new file-tree algorithm. |
| `PortableMetadataCache`, ordered directory/inode builders, `FrontierInodes` | Keep exact constructed metadata, final reference counts and coalesced sorted records. Finalize under one owner; do not restore all-node discovery or repeated intermediate inode roots. |
| `consume_checked_owned_page`, `insert_checked_object_batch` | Already shared by initialization and Commit. Feed finalized batches without mandatory full candidate payload spill/readback. A shared wrapper alone is not completion. |
| `ObjectBuffer`, `DeferredObjectStore` | Retain necessary readable provisional structures and existing reachability guarantees; remove redundant final payload retention once its responsibilities transfer. |
| Store `workspace.rs`, `staging.rs`; Workspace `lifecycle.rs` | Keep expected-head/base publication, retained-stage recovery, candidate-bound installation and retry without duplicate Commit. |

Remove obsolete wrappers and ordinary paths after transferring all callers; report the caller/deletion ledger. Shared code must serve actual native and Workspace callers. Existing resource-domain handling remains correct; it must not become a second tiny/large/dense/sparse product engine.

## Finality, deduplication and failure contract

1. Schedule each changed inode once, not once per hardlink path. Producers return input identity/generation, content root and counters. Namespace/reference mutation and SQLite connection ownership stay with the coordinator/consumer.
2. The native slab writer is append-only (`get` fails). Incremental builders can read base or newly constructed objects. Preserve their actual read capability with existing authenticated readers and bounded provisional ownership; do not blindly replace every object buffer with the native writer.
3. Distinguish canonical construction from final candidate membership. A completed producer or `put_owned` does not prove every emitted intermediate object is reachable. Reuse existing finality/selection facts; do not invent a competing reachability algorithm or admit every provisional tree object.
4. Before admitting earlier, preserve current construction/admission/publication failure and footprint semantics. Distinguish inserted from reused objects, retain candidates when required, preserve previous roots. Never transplant the empty-Store requirement or whole-Store initialization failure deletion. Specify failed-producer/consumer cleanup before implementation.
5. Dedup is already efficient in shape: prepared INSERT with conflict handling, bounded equality reads only for conflicts, no preliminary SELECT per new object or whole-Store scan. Preserve collision checks and selected = inserted + reused receipts.
6. Keep all simultaneous ownership bounded: producer scratch/partial slabs, queue, consumer batch, structural state, pending results and retained input descriptors. Start with existing slab/backpressure limits, not a full-workspace RAM buffer. Host CPU is not bounded by the container's two CPUs. Join every worker on failure before releasing source lifetimes.

## Actual overhead and non-priorities

- Stage is one `(workspace_id, branch_id, root_id)` metadata row, not a payload upload. Create publication was 0.223 ms; stage precedes that timer and is not separately measured. Live capture was 0.012 ms, zero copied files/bytes. Do not remove recovery semantics for an unmeasured tiny saving.
- Create reused 831 of 82,762 objects, about 1% of candidate bytes. Admission 431.738 ms includes insertion/conflicts 150.871 ms, transaction begin/commit 41.522 ms and about 239.345 ms other handling. It is not a dedup timer. Dedup-heavy siblings may differ; do not skip checks based on this case.
- Selected candidate spill readback was 113,434,447 bytes. This host-side temporary I/O is the major delivery opportunity after spools.
- Carrying fully validated immutable owned bytes across an in-memory handoff may avoid duplicate digest work. Existing authenticated-object names do not establish complete framing validation. Preserve authentication after fresh spill/durable reads; profile before prioritizing this secondary change.
- Delete Commit is 460 ms, with namespace 454 ms including record/reference work 347 ms. Content-production parallelism does not address that bottleneck.

## Implementation and experiment order

1. Measure the committed shared-spool slice first; reuse a newer complete result if available. Keep retirement, physical allocation, retained bytes and physical file count visible through Commit/End. Preserve the owner's current work.
2. Define final-output ownership and all failure paths for the shared seam. Replan if direct admission would broaden persistent garbage or lose incremental reads.
3. Connect finalized owned delivery to the shared consumer with one producer first, preserving compact results and ordered finalization. Demonstrate removed payload passes; merely extracting helpers is not a performance gain.
4. Overlap producer/consumer work using bounded backpressure. Add a small bounded producer set only if measured construction merits it, reusing the native driver rather than one thread per file. Observe producer wall/CPU, blocked sends, consumer idle and finalizer cost. Stop increasing concurrency when complete time stops improving.
5. Improve remaining delete reference processing only through the shared ordered algorithm if it remains worthwhile. Do not start a separate delete engine.

Run one selected complete performance sample per substantive experiment, seed 1, serial under the existing lock. Narrow timers may separate spill readback, admission authentication and conflict work. Retain bytes, objects, transactions, CPU/RSS, queue/scratch peaks and retained storage. Do not sum overlapping timers or move required work outside product timing.

Reuse focused tests for serial/shared-input canonical equivalence, unchanged-tree reuse, sparse/overlap data, aliases, corrupted spill, producer/consumer error, head movement, exact installation retry and retained reads. Add only missing checks. Independent sampled proof remains final-stage only, 45 seconds work / 59 seconds hard; no exhaustive per-iteration proof.

## Expectations and completion

Subsecond create-100 Commit is a plausible next objective, not another rigid per-phase gate. Subtracting old retirement leaves about 1.483 seconds; realistic delivery/overlap/finalization gains remain necessary. Delete already has a 460 ms sample. Do not impose a speculative 100 ms gate or spend a campaign on marginal gains.

Existing #47 acceptance still requires both complete original 20,000-file / 100 MiB create/delete lifecycles strictly below 1 second and final correctness. Commit-only progress does not close #47/#46/#39. Replan on no-go using evidence; never manufacture PASS or silently change the target.

Completion includes a real caller/reuse/deletion ledger: native initialization and ordinary Commit use shared production/delivery/persistence; create/delete/edit/rename retain one Commit implementation; superseded ordinary mechanisms cannot remain as agent-reusable fallback engines. Document expected sibling benefits separately from measured qualification. Preserve incremental small edits and initialization semantics.

## Implementation handoff — shared finalized output, 2026-09-06

The shared-segment create result is now recorded in the owner's results ledger: complete lifecycle 16.187 s, Commit 1.271 s, retirement 2.571 ms, and all physical spool bytes/descriptors retired by Commit. Selected candidate readback remained 113,434,447 bytes. That result supersedes the pending status above; full acceptance remains TARGET_MISS.

The next slice transfers the actual native queue/backpressure/drain/join loop to `run_finalized_output`. Native discovery and Workspace's immutable dirty-inode inputs both call it. Workspace starts one producer. Its `FrozenFile` retains existing PieceTree/segment and snapshot-reader ownership; the existing rope and FileMutationBatch builders remain the only canonical content algorithms. The source-generation/length and completed-root result journal is bounded by the dirty input count, and final metadata/reference/ordered inode processing remains with the Workspace coordinator.

Each readable file buffer calls existing `ObjectBuffer::finish(file_root)` selection before `FinalizedOutputWriter::send_selected`. Owned selected memory moves into native's bounded slabs; provisional file objects never enter the consumer. The consumer and final structural output both use `CheckedOutputAdmission` and `consume_checked_owned_page`. An opaque Store/Workspace-bound admission token carries exact globally unique selected/inserted/reused accounting across file outputs and final structure. Native keeps its own empty-Store precondition, identity/publication, and failure cleanup. Commit imports none of that cleanup.

Canonical producer scratch and each spill-delivery page are capped at 1 MiB, reusing the existing spill-buffer allowance. Together with the 1 MiB spill buffer, 256 KiB partial/sending slabs, four 256 KiB queue slots, and existing sub-4 MiB consumer batch, payload ownership stays bounded. Existing bounded reference/ID indexes and authenticated snapshot caches remain separately accounted structures. No full-workspace canonical payload collection is added. The finalizer runs after producer join and batch flush.

Failure in either side cancels production, drains queued owners, and joins all producers before returning. No stage/head/Commit is published before the final structure succeeds. Already admitted selected CAS rows may remain after failure, as permitted by the existing shared-construction staging contract; reused rows and prior roots are preserved. Publication retains the existing stage, expected-head check and recoverable checkpoint. Preview/reconciliation call the same file construction synchronously with private selected-output merging and never use early admission.

Caller/deletion ledger:

| Previous responsibility | Active shared path / retained lifecycle boundary |
|---|---|
| Native inline channel and scoped worker receive/join loop | `run_finalized_output`, called by native initialization and ordinary Workspace Commit |
| Initialization-only slab writer and segment accumulator names | `FinalizedOutputWriter`, `CheckedOutputAdmission`, `FinishedOutputAdmission` |
| Workspace whole-candidate file-payload retention until namespace completion | Per-file reachability selection followed by owned slab delivery; private final namespace buffer retained |
| Separate `admit_checked_objects` Workspace wrapper | Shared checked accumulator plus Store/Workspace-bound `WorkspaceAdmission` receipt continuity |
| Workspace-borrowing file reader/build methods | Retained `FrozenFile` inputs and the existing canonical builders |
| Re-read completed file state during checkpoint validation | Producer-checked canonical length plus identity/generation/length result handoff; final metadata/reference checks retained |

Validation before measurement: 53 pre-existing Workspace tests, the added private-preview/Commit-root equivalence test, 12 file-edit and two reconciliation integration tests, and v5 migration/staging pass. Store suite: 45 tests passed, one existing large-spill test ignored; one new test initially asserted the wrong missing-object error variant, corrected to exact absent membership and passed with an added failed-admission preservation check. New producer/consumer cancellation test verifies both source owners are joined. No independent benchmark proof has run, and performance for this delivery slice is pending.

The first source-bound delivery sample is now recorded as attempt 5 in `issue47-subsecond-workspace-results.md`: Commit 1.130 s, complete lifecycle 16.173 s, selected spill readback 4,256,258 bytes with unchanged canonical candidate/admission totals. The earlier performance-pending statement is superseded by this result. Full acceptance remains TARGET_MISS.

## Aggressive finalization slice — implementation, measurement pending

The exploration objective is now <=300 ms Commit for each original create-100 and delete-100. It is not demonstrated, does not add a mandatory phase gate, and does not change full-lifecycle acceptance.

`FrontierInodes::finish` no longer forces memory-resident changes into a physical journal. It consumes the bounded pending map into ordered inode/ObjectId pairs and feeds the existing sorted inode builder. When a spill exists, merges are sequential and buffered; final encoding attaches ObjectIds in bounded blocks and the builder consumes a buffered forward cursor. The supported-domain fallback still uses the same canonical mutation routine from the original root. Spill probes read keys, decoding a complete record only on a match.

Expected checkpoint NodeId/content identity now travels with the coalesced record through pending updates and spill merges. Final encoding validates the same kind/content/metadata/reference facts and writes the checkpoint row once. This removes the separate checkpoint-to-inode lookup pass; file length was already checked at the immutable file-result handoff. The private fixed spill row expands from144 to192 bytes to carry these facts; this is a candidate journal layout, not a canonical/schema change. Changed and unchanged dirty records both carry checkpoint facts, preserving ordinary no-op/alias/reference semantics through the same finalizer.

File-result/checkpoint I/O buffers grow only from budgeted slack, capped at64KiB each. Four buffer increments are subtracted before assigning pending-map/tree/deletion scratch allowances; 1KiB policies retain the existing small-buffer domain. Merge and encoding buffers reuse the unused pending-entry reservation. No memory limit increases or workload-specific route is introduced.

Retirement telemetry now separates the retain predicate's bookkeeping time and exact retired segment count from total retirement. The remainder includes Arc/segment/File drop, physical accounting and map iteration; it is explicitly not a syscall-only close timer. All release work remains inside the original timing boundary.

Validation:55 Workspace library tests pass, including a new memory/spill/zero-scratch final-root equivalence test and existing coalesced merge-failure, alias, sparse, rollback and partial checkpoint retry tests. The new test initially used a tombstone for an inode absent from the base; the existing generic fallback correctly rejected that artificial case, so the equivalence test now removes an existing inode. No product fallback was weakened. Full file-edit/reconciliation integration checks and the selected performance sample follow serially.

The12 file-edit integration tests passed. One of the two reconciliation tests initially failed before product setup with `StoreAlreadyExists`; their shared temporary-path helper used only process ID plus timestamp. A per-process atomic sequence now prevents same-timestamp collisions, and both reconciliation tests pass. The final-record equivalence test additionally checks that attached checkpoint identity survives memory/spill updates and is validated exactly once. No independent proof has run.

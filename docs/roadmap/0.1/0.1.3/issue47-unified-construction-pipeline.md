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

Attempt6 measures this slice on original create-100: Commit772.030ms, namespace/final-record work58.928ms, full lifecycle17.127s TARGET_MISS. Canonical totals and selected spill readback are unchanged. The detailed ledger preserves retirement variation and identifies shared consumer service as the next target.

## Shared consumer slice — implementation, measurement pending

New output objects now make one pending lookup and one exact seen-set insertion, then move directly into the shared pending batch. Duplicate pending objects compare against the already located owned value; flushed duplicates use bounded authenticated equality-read pages (128objects /256KiB payload, with the existing single-object ceiling). No preliminary durable lookup is added for new objects. Exact global candidate uniqueness and inserted/reused provenance remain unchanged.

The ID set's old linear-file membership scan beyond its memory allowance is removed. A private SQLite primary-key index reuses the existing payload-offset index's scratch connection configuration.4MiB of the original64MiB set allowance is reserved for its cache during migration; the completed derived index replaces the old memory set only after success. This high-cardinality fallback was not exercised by the83k-object create sample, and no current performance gain is attributed to it.

Validation:25 Store object tests with instrumentation and all12 file-edit integration tests pass. A new test proves300 flushed duplicates require exactly3 batched equality reads, leave candidate/inserted/reused counts unchanged, and still reject different bytes under an existing identity. Another forces the private ID index, checks primary-key lookup and4MiB cache configuration, verifies membership/counts, and confirms cleanup. Native/shared admission boundary tests remain passing. No independent proof has run.

Attempt7 measures the consumer slice: Commit679.934ms, pipeline431.588ms, consumer286.724ms, full lifecycle14.942s. Canonical totals are unchanged. Parent15s PASS does not satisfy #47; <=300ms exploration and full-lifecycle subsecond qualification remain unmet. The next measured slice targets shared deletion/namespace record processing.

## Shared edge/reference slice — implementation, measurement pending

The existing sorted directory tree engine now reports each original/final binding from its leaf merge. The ordinary API delegates to that same engine with a no-op observer; Workspace captures reference facts without looking up each binding again in later passes. New-inode counts still come from their final paths. Observer output is provisional: the private reference journal rewinds to its pre-directory prefix if the existing supported-domain fallback is required.

Workspace applies all added references before removals. Each bounded reference page coalesces aliases by inode, fetches authenticated base records in batches, and retains latest pending/spilled-reference precedence when applying those records. Top-level removals and recursive directory pages use the same base-record helper and release algorithm; coalesced initial removal amounts do not add another deletion engine. The helper decodes the already-authenticated canonical record directly instead of reconstructing its outer bytes framing.

The reference journal is lazy (no physical file when no existing references change), anonymous and buffered within the existing allowance. File-result buffers are dropped before reference processing; checkpoint allocation begins after it. Retained outer reference pages are reserved from recursive deletion scratch. All journal construction, consumption and cleanup remain inside Commit. Published roots, stage/recovery and shared spool lifetime are unchanged.

Validation so far:55 existing Workspace tests and8 sorted-tree tests pass. Added observer tests check exact original/final pairs, canonical-root equivalence and error propagation with old-root readability. A new Workspace regression verifies reference-journal prefix rollback, coalesced alias precedence and a regular-file deletion at the existing1KiB final-delta budget. Full file-edit/reconciliation checks and one original delete-100 performance sample follow serially. No independent proof has run.

The reference-journal regression and all14 file-edit/reconciliation integration tests pass. The next performance selection is original delete-100; create gains are not extrapolated to it. The historical delete assessment was regenerated from its unchanged raw receipt while checking the reference counters; no historical performance was rerun or relabeled.

## Located reference update — implementation, measurement pending

`change_references` now locates the current pending/spilled/base record once, checks the addition/removal, and updates that exact owned location. It preserves attached checkpoint facts and current-over-prefetched precedence. The ordinary release algorithm and recursive traversal are unchanged. The former general `set` wrapper is test-only; production reference changes no longer compose a read lookup with a second update lookup.

All56 Workspace library tests and the targeted hardlink/rename/replacement integration tests pass. The reference-journal test now forces a one-entry pending map so later alias removal exercises a previously spilled current count. One original delete-100 measurement follows; no independent proof.

## Completed full-file output — implementation, measurement pending

The existing `rope::build` streaming builder emits sealed prefixes, connects every emitted prefix to the final mapping root, then emits FileState. This is the append-only finality already used by native import. `ObjectBuffer::build_complete_file` applies that property to a private bounded buffer: successful full construction and exact final-length validation precede its existing all-reachable seal. Workspace's existing full-file-build branches use it; incremental FileMutationBatch and resumed capture paths retain readable buffering and final reachability selection. No size/density/workload policy selects a new engine, and the same rope/extent algorithm remains authoritative.

This removes per-file reference-index construction and the completed full-file DFS/seen/order selection pass. It does not skip canonical identity/framing checks in delivery/admission, expose an incomplete file to the consumer, or import native empty-Store cleanup. The existing1MiB per-producer scratch, spill delivery, shared queue/consumer, single producer and final publication boundaries remain unchanged.

A focused check compares roots, every selected object ID and encoded bytes against ordinary reachability selection for empty input, chunk boundaries, a branched extent tree, spill, and repeated/zero data. Length mismatch and source I/O failure return no candidate. A test initially assumed large logical zero data must spill; deduplication correctly kept it in memory, so mandatory spill is asserted only for varied data. The semantic equality checks remain for both patterns.56 Workspace tests and all14 file-edit/reconciliation integration tests pass. One original create-100 sample follows; final independent proofs remain deferred.

Attempt10 measures full-file finality on create-100: pipeline413.860ms and Commit710.826ms, with60.862ms retirement. This is not a net Commit win over the prior679.934ms result. Canonical totals/readback are unchanged, and all physical spools retire by Commit. Latest delete is304.405ms on `02af76b86`. Both exploration objectives and full-lifecycle acceptance remain unmet; the results ledger records the no-go/replan without pooling revisions or subtracting favorable cleanup timings.

# Issue 47 implementation results

Status: in progress; neither full subsecond target has passed. No independent benchmark proofs have run.

## First implementation slice

Ordinary Commit now installs candidate-bound backing facts into existing Workspace nodes. It retains paths, NodeIds, pins and directory-parent associations. A bounded anonymous metadata journal survives publication/install failure. Canonical construction checks final record kind/content/length/metadata/reference facts before publication; installation checks the exact candidate root, mutation generation and live attributes. Reconciliation still refreshes from its exact published snapshot.

47 Workspace library tests passed with `cargo +1.85.1 test -p layerfs-workspace --lib --no-default-features -j2 -- --test-threads=1`. This includes incremental file reuse, sparse/overlap/write rollback, alias/reference behavior, old-root use, exact returned snapshot, rejected publication, partial installation recovery, Busy guards and bounded policies. These are component regressions, not the required independent sampled benchmark proofs.

## Symbol and caller transfer ledger

| Status | Symbols | Actual callers/responsibility |
|---|---|---|
| Kept | PieceTree, FileMutationBatch, rope::build, PortableMetadataCache, sorted directory/inode builders | Frontier and localized content construction use the existing algorithms |
| Changed | build_candidate → PreparedCommit | Ordinary lifecycle retains checkpoint and passes BuiltRoot to Store; reconciliation consumes BuiltRoot and refreshes; WorkingTree resolution preview safely drops the whole prepared result; direct tests consume BuiltRoot |
| Added | CheckpointJournal, Checkpoint, install_checkpoint | Bounded construction-to-continuation facts and retryable installation |
| Deleted | rebase_committed, lookup_committed_path, rebase parent cache | Ordinary lifecycle now calls install_checkpoint |
| Changed | refresh_reconciled | Retains only reconciliation's exact-snapshot replacement responsibility |
| Deleted | General manifest ordinary construction, create_group, apply_one, set_metadata, groups_by_inode, groups_by_node, base_symlink, depth | Structural/root-only/unlinked cases transfer to frontier |
| Kept, awaiting consolidation | build_localized_candidate | Content-only low-budget supported domain; same checkpoint result, no reconstruction fallback |
| Kept | base_manifest, final_manifest | resolution_fingerprint still uses them |
| Kept | lookup_path | SDK edit path resolution |
| Kept | planned admission | Reconciliation/publication still requires it; no native initialization assumptions imported |
| Changed | Checkpoint/SpoolRetirement telemetry | Receipt, attribution total, SDK JSON consumer and Workspace receipt prose migrated together |

## Evidence handling

Retained create baseline: `issue46-parent-create100`, total 22.142764 s, Exec 15.086029 s, Commit 6.988436 s, reconstructed continuation 5.258958 s. Retained delete observation: `issue46-pages-delete100`, total 4.991831 s, Exec 3.297524 s, Commit 1.679852 s. Delete predates parent-cache source; they are not a same-source final pair.

New checkpoint measurements will be appended with source/product/image/harness identities. Historical raw receipts are immutable. The strict child assessment remains complete `pure_call_sum_ns < 1,000,000,000`, separately from the parent's 15-second classifier.

## Attempt 1 — in-place checkpoint, create-100

Receipt: `benchmark-results/host-store/results/issue47-checkpoint-create100/perf.jsonl`; child assessment beside it. Complete original workload, seed 1, 20,000 writes / 104,857,600 bytes, one publication. **TARGET_MISS: 19,916,002,292 ns**, strictly above the child limit. Create 9,009,625 ns; Exec 14,649,304,542 ns; Commit 5,225,464,125 ns; visibility 72,750 ns; End 32,151,250 ns.

Commit content 1,184,619,625 ns; namespace (including checked final-record handoff) 782,133,209 ns; candidate finish 98,398,917 ns; admission 432,169,750 ns; checkpoint 2,723,398,708 ns, including **2,691,903,129 ns spool retirement**. Only **5 Commit-scoped snapshot database calls** versus the retained create receipt's 108,006. Candidate admission still reads 113,424,403 spilled bytes and delivers 82,669 selected objects in 28 transactions. This demonstrates removal of snapshot rediscovery, not completion of Commit optimization.

Producing identities:

- source_identity: `d32305155dc66eb73607ac18b03c26efe396253f6c02842777824a69fa78e599`
- product_identity: `c30ebcb2adcd6f4a04c68247e704191dabbdb64c45e9355db7f4544ac8f3b5cc`
- image: `sha256:7e15e4d30bcb5231a85bab4a75edf8f930505baffb36d6a044338527f6fc618e`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `f3cb375cbb649463e80f8f06e66c69e9e2f56c67e0417f3a8521c1d33f40397e`

Source was dirty on documentation checkpoint `8aec76f76` when built; the following local implementation commit records that source. No push. Host macOS SQLite, real Linux FUSE, no mounts, 2 CPUs / 2 GiB / no swap / 256 PIDs were validated. Compatible protected create preparation was copied into isolated cache; cache hit, independent closed-byte-copy sample, master unchanged, cleanup PASS. No OOM or swap. Linux observed peak 41,447,424 bytes; host process observed peak 194,330,624 bytes. No independent proof ran.

No-go revision: removal of path reconstruction alone is insufficient. Per-file unlink/close remains the dominant checkpoint cost; retaining final coalesced inode records will remove intermediate construction and the 0.782 s record reread validation introduced by this conservative first handoff. Next: bounded ordered final inode deltas and paged deletion accounting, then shared checked admission consumer. Minimal storage-lifetime work is now evidenced as necessary for the remaining 2.692 s retirement cost, but broad Exec ownership/segment redesign remains behind the substantive Commit checkpoint.

## Second implementation slice — final inode stream and deletion pages

The ordinary localized builder and its selectors are now removed. All ordinary candidates use the existing frontier/content algorithms, including the retained 1 KiB supported domain. The existing dirty-node set now tracks metadata and directory/new-node changes as well as file edits, avoiding an all-materialized-node probe. Remaining spool accounting visits retained descriptors, not the complete live graph.

`FrontierInodes` now coalesces final records over the immutable base. Its bounded map spills sorted fixed-width records; reads use pending → spilled (including tombstones) → authenticated base precedence. Existing spilled keys update in place. New-key merges replace ownership only after complete writes, and a regression injects a failed merge before retry. After accounting, each final record is encoded once and one ordered ID stream feeds the existing sorted inode builder. Its supported-domain resource fallback replays final records from the original root. Superseded ordinary `FrontierInodes::flush` intermediate tree construction is deleted. The known large-change ceiling is repeated new-key spill merging; it is documented in code, not presented as a universal external sorter.

Deletion release now consumes bounded directory pages and batches immutable inode-table/record reads, checking latest reference mutations again before using prefetched records. Lookup batch width separately reserves raw/decoded canonical-page scratch; small policies retain single-key canonical reads. Added a regression with moves, repeated aliases, unseen surviving aliases and several pages under a 16 KiB policy. Nested `deletion_cursor_ns` and `deletion_records_ns` distinguish cursor work from reference/record work without double-counting total attribution.

`PortableMetadataCache::get_by_root` reuses existing exact constructed metadata during final-fact checking. `consume_checked_owned_page` now owns/sorts/authenticates bounded pages, commits checked insertion, and returns metrics to ordinary Workspace and native initialization. Deleted the duplicated native admission transaction wrapper and Workspace per-object outcome accumulation. Native empty-Store/cleanup/final-publication behavior and reconciliation planned admission stay with their real callers. This is code reuse, not removal of the remaining candidate payload spill/readback.

Regression discovery and repair: a rejected write can establish editable backing without semantic mutation. Retained-descriptor accounting preserves its PieceTree charge across another file's Commit. A stronger pinned/unseen-alias test then failed with published `abc` instead of edited `xbc`; construction had skipped every pathless node. Canonical inodes with remaining references now participate even when their materialized path set is empty, preserving the same NodeId and publishing edited bytes to the unseen alias. The regression checks both canonical and live bytes.

Validation so far: all 49 Workspace library tests passed before the additional paged-release test; that new test and the revised lookup-width test passed separately. All 20 object-store unit tests passed; explicit native multi-batch publication-failure cleanup and Workspace candidate/admission/publication failure retry integration tests passed. Builds/tests are serial. No independent benchmark proof ran. Next measurement is one original delete-100 sample, seed 1; its old protected preparation was evicted, so supported infrastructure will regenerate it once.

## Attempt 2 — ordered/paged Commit, delete-100

Receipt: `benchmark-results/host-store/results/issue47-ordered-delete100/perf.jsonl`; strict child assessment alongside. **Child TARGET_MISS: 4,061,720,833 ns**; the runner's parent 15-second PASS is not child PASS. Original 20,000 unlinks / 233 rmdirs, full traversal/metadata/sync, seed 1, one publication. Create 13,896,459 ns; Exec 3,580,485,958 ns; Commit 460,068,958 ns; visibility 89,250 ns; End 7,180,208 ns.

Commit namespace 454,455,959 ns, including cursor 15,309,616 ns and record/reference work 347,007,136 ns. Checkpoint 723,583 ns. Admission is still exactly nine selected objects / 22,318 bytes, with no spill readback. Commit snapshot calls **1,385** versus the retained older delete observation's **27,546**; observed Commit **0.460069 s** versus **1.679852 s**. The retained observation predates the parent-cache revision; this is not a controlled same-source final pair.

Producing identities:

- source_identity: `18954962c98ff1fb959a626820f0ad0df575950cc4e4e97aa53e568531b0f12f`
- product_identity: `35c9141e56b236f8af8377fba86d342ec31e83601d3b9fde758979ccb4645a6f`
- image: `sha256:b51dd5afb527c57f55c450f8f3c7ccd9c7dd5badcd842b986d7157c8c0011ec7`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `d4f9271acc1737f52491808ef1a53cdfdabef33bee5f0657e7b97320cf40660a`

Implementation commit `55c950979a81958dcd8d948dd5af6e3b177baf69`, local and not pushed. The build sidecar reports dirty=true (retained source seal above is authoritative); raw identities are unchanged. Host topology/container bounds were validated, no data mounts, no OOM/swap. Supported infrastructure regenerated the missing deletion preparation, then used an independent closed-copy sample; master unchanged and cleanup PASS. Linux observed peak 9,101,312 bytes. No independent proof ran.

This records a stable checkpoint for the major construction, checkpoint and deletion handoffs, not a standalone Commit acceptance gate. No-go revision: remaining delete time is now primarily Exec (3.580 s). Create's measured 2.692 s per-file retirement still requires the documented physical-lifetime dependency: bounded shared append segments referenced directly by existing PieceTree ranges, preserving rollback/read-plan/open-unlinked ownership and measured retirement. Carry forward to coherent Exec after recording this checkpoint; both full targets and final sampled proofs remain outstanding.

A follow-up regression extended unseen-alias coverage to **unpinned** edited files. It failed with published `abc` instead of `xbc`: `reclaim` removed the pathless dirty inode before Commit despite remaining unseen file links. Reclaim now retains unpublished non-directory inodes with surviving links; directory removal and clean canonical reclamation keep their existing behavior. Both pinned/unpinned forms and the focused construction regressions pass. This correctness repair follows the measured `55c950979` source and is not retroactively attributed to its receipt.

## Coordination boundary

A separately assigned Exec issue/task owns layerfs-fuse and agreed transport changes in another worktree. This task remains Commit and integration owner for Workspace, Store, content, shared spool lifetime and #47 guide/results. No uncommitted FUSE changes exist here. Cross-boundary projection/telemetry hooks require one named patch owner. All performance-sensitive builds/tests/samples use the same host measurement lock; no other task's caches, containers or artifacts are cleaned. Final #47 qualification must use the combined product.

## Attempt 3 — ordered Commit, create-100

Original complete create-100 / seed 1 on `46463287c`, before shared spools. Child **TARGET_MISS**: `18940567833 ns`. This directly measures the second Commit slice; gains are not inferred from delete. Receipt and strict assessment: `benchmark-results/host-store/results/issue47-ordered-create100/`.

- create: 9046375 ns
- exec: 15337713958 ns
- commit: 3561906833 ns
- visibility: 79500 ns
- end: 31821167 ns

Commit detail:

- content_ns: 597275834
- namespace_ns: 318532333
- candidate_finish_ns: 91943750
- object_admission_ns: 431737875
- checkpoint_ns: 2117268584
- spool_retirement_ns: 2079293345
- snapshot_database_calls: 6
- object_admission_spill_readback_bytes: 113434447

Producing identities:

- source_identity: `772a892b09a3d39e7aa82e5610da1bc4d3a82faf3c3beb96b07555cc1f3d967c`
- product_identity: `641afd33103ef39c1453ed8321101799567068eb1fb967836eabedcb4e40872f`
- image: `sha256:ac58ab1abda379bab689f66919c7889e6cd2f9a6ed162f16fb3ceac368dd6f43`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `28fcd85776a5f85c1b4919cea46374dcb3bc0424c51f1cd41a37fbc7334c2acf`

Complete host topology and container bounds validated; protected preparation reused through independent closed-copy sample, master unchanged, cleanup PASS, no OOM/swap. No independent proof. The next change targets shared physical segment ownership; snapshot rediscovery is already removed and no further lookup-cache tuning is planned.

## Shared physical spool ownership slice

Existing `Piece::Spool` ranges now carry an `Arc<SpoolSegment>` and absolute physical offset/length. The compact PieceTree case owns a one-word range handle instead of a private path/descriptor; the existing 100,000-file / 800,000-byte logical compact-piece charge regression remains valid. PieceTree split/replace and canonical FileMutationBatch/rope algorithms remain unchanged. Range binding metadata replaces per-logical-file physical ownership; it is bounded by the live range graph.

`SpoolSegment` uses a private create-new descriptor and removes its temporary name immediately. Workspace shares append capacity across files and checks physical high-water before writes. Nominal segment capacity is 1 MiB; a larger single write receives a segment large enough for that write within the unchanged payload allowance. There is one storage mechanism. New empty files, inline edits and logical zero ranges need no physical file. `edited_nodes` tracks logical editable state; the physical registry tracks unique segment descriptors.

Delayed `ReadPlan` and rollback pieces retain segment ownership directly. Short append rollback truncates only the current unpublished physical tail. If tail cleanup itself fails, observed retained bytes are added to segment charge while logical file state remains unchanged. Admission checks the existing logical charge and retained physical segment charge; partially dead shared segments remain charged. Checkpoint drops published ranges and retires unused segments, including on a clean subsequent Commit after a held read finishes. Reconciliation transfers still-held segments and charge into the refreshed Workspace. Physical allocation/peak/error observations remain tied to actual segment lifetime.

Retired ordinary production symbols/state: per-node `open_spools`, `FileData::Edited.spool`, `spool_file`, `create_spool`, `remove_spool_file`, `remove_spool_if_exists`, and zero-based `contiguous_spool_len`. Their callers transfer to segment-owned ranges, `edited_nodes`, direct descriptor reads and `retire_spool_segments`. The generic Workspace runtime `spool` directory remains for owned candidate journals/runtime artifacts. Capture eligibility follows the unchanged logical sequential-write checks and accepts physical segment transitions.

Validation: 51 Workspace library tests passed together after the main migration; targeted reconciliation-held-charge and failed-tail-cleanup regressions passed afterward. Existing tests still cover dense 100,000 compact ranges, sparse/overlap/zero data, edit limits, exact rollback, capture equivalence, canonical history, alias/reference behavior, partial-install recovery and repeated Commit. A new interleaved two-file regression holds a prepared read across overwrite, sparse changes, short append, unlink, rollover and two Commits, proving correct bytes and segment retirement. The first cleanup-failure test setup incorrectly placed the failed tail in an entirely unreferenced segment, which was correctly reclaimable; the test was corrected to retain a partial tail in the same live segment. No product weakening was made to satisfy it.

Normal benchmark observations now include passive maintained physical allocation, open physical file count and retained segment bytes before and after Commit. The directory walk is explicitly named-file-only; anonymous segment allocation is not inferred from that walk. These observations perform no oracle, namespace scan or independent proof. Performance remains unmeasured for this slice until the next source-bound receipt; no subsecond claim.

The full 12-test SDK file-edit integration suite also passed serially after the spool migration, including candidate/admission/publication failure retry, projection refresh/reopen, aliases, owner composition, stale heads and discard. No independent benchmark proof ran.

## Attempt 4 — shared segments, create-100

Original complete create-100 / seed 1 on `8cbbb425b`, child **TARGET_MISS**: `16186989166 ns`. Receipt and assessment: `benchmark-results/host-store/results/issue47-segments-create100/`.

- create: 8691500 ns
- exec: 14875371750 ns
- commit: 1270814792 ns
- visibility: 73541 ns
- end: 32037583 ns

Measured Commit/storage detail:

- content_ns: 435104125
- namespace_ns: 299912500
- candidate_finish_ns: 90672416
- object_admission_ns: 417636958
- checkpoint_ns: 23277250
- spool_retirement_ns: 2571041
- snapshot_database_calls: 6
- object_admission_spill_readback_bytes: 113434447
- spool_write_open_count: 101
- spool_write_ns: 217079511
- workspace_fence_ns: 198000

Physical segment observations:

- `{"allocated_bytes": 104914944, "kind": "workspace-physical-spool", "method": "verification_workspace_state", "observation_count": 20202, "observation_errors": 0, "open_physical_files": 101, "peak_bytes": 104914944, "phase": "after-workload-before-commit", "precision": "mutation-event-aggregate-allocation", "retained_segment_bytes": 104857600, "scope": "passive maintained Workspace counters; no independent verification"}`
- `{"allocated_bytes": 0, "kind": "workspace-physical-spool", "method": "verification_workspace_state", "observation_count": 20202, "observation_errors": 0, "open_physical_files": 0, "peak_bytes": 104914944, "phase": "after-commit", "precision": "mutation-event-aggregate-allocation", "retained_segment_bytes": 0, "scope": "passive maintained Workspace counters; no independent verification"}`

Producing identities:

- source_identity: `a1cff705acbc35adad92c3b7115ab42dde35a944e6a4396049e68b3e0377149f`
- product_identity: `b3f0333e3ce623aa59c8ce1312d2cb4f52aa55ad45905438ceeb22a5dabac5cb`
- image: `sha256:445fb85c77c344069325af3debd132b8240dd158e37be2f9ad18de1eff200366`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `f9796fea5d4552c28a77f9499f783a70c1235e686c7e855f356634e23a6ecdca`

Host topology/container bounds validated, protected preparation reused with independent closed-copy sample, master unchanged, cleanup PASS, no OOM/swap. No independent proof. No-go revision: physical per-file lifecycle is removed; next target is candidate payload retention/readback while Exec remains separately owned. Full subsecond acceptance remains unmet.

## Attempt 5 — shared finalized-output pipeline, create-100

Original complete create-100 / seed 1 on `de3c35387`, child **TARGET_MISS**: `16173168125 ns`. Receipt and strict assessment: `benchmark-results/host-store/results/issue47-output-create100/`. One selected sample; no independent proof.

- create: 9272917 ns
- exec: 14990186625 ns
- commit: 1129995125 ns
- visibility: 89792 ns
- end: 43623666 ns

Measured Commit/storage detail:

- content_ns: 636913499
- namespace_ns: 269526667
- candidate_finish_ns: 25910208
- object_admission_ns: 73278333
- checkpoint_ns: 112590041
- spool_retirement_ns: 90428458
- snapshot_database_calls: 11
- object_admission_spill_readback_bytes: 4256258
- object_admission_memory_owned_bytes: 109178189
- object_admission_borrowed_copy_bytes: 0
- output_pipeline_ns: 467859084
- output_admission_ns: 316901950
- output_blocked_ns: 198744293
- output_consumer_idle_ns: 158446244
- output_queue_peak_bytes: 1048576
- spool_write_open_count: 101
- spool_write_ns: 239242460
- workspace_fence_ns: 270542
- candidate_objects: 82762
- candidate_bytes: 113434447
- inserted_objects: 81931
- inserted_bytes: 112321855
- reused_objects: 831
- reused_bytes: 1112592

Selected spill readback fell by 109,178,189 bytes (96.25%), from 113,434,447 to 4,256,258. Exactly 109,178,189 bytes were instead delivered as final owned memory; borrowed payload copies remain zero. Candidate objects/bytes (82,762 / 113,434,447), inserted objects/bytes (81,931 / 112,321,855) and reused objects/bytes (831 / 1,112,592) match the shared-spool sample. This establishes actual removed delivery I/O, not only overlapping timers. Store live size after Commit remained 131,661,824 bytes, 2,009 pages, zero freelist pages.

Commit was 1,129,995,125 ns versus 1,270,814,792 ns in the prior single sample. Checkpoint retirement was slower in this sample, 90,428,458 ns versus 2,571,041 ns; retain both observations rather than subtracting the difference to claim another result. Both samples retire all 101 physical spool files and all 104,857,600 retained segment bytes by Commit. End was included at 43,623,666 ns.

The shared file pipeline occupied 467,859,084 ns, with 316,901,950 ns consumer work, 198,744,293 ns producer blocked time and 158,446,244 ns consumer idle time. These are overlapping/nested measurements, not additive phases. Queue peak reached its 1,048,576-byte bound. The existing content-labelled phase also includes the subsequent metadata/directory/record preparation; its 636,913,499 ns is not pure content construction. Final selection was 25,910,208 ns and remaining structural admission 73,278,333 ns.

Replan: retain the shared delivery slice. More producers are not justified while the single producer already spends substantial time blocked on the consumer. Remaining Commit opportunities are consumer ownership/dedup handling and ordered record/reference updates; the latter is shared with the measured delete bottleneck. Do not revisit checkpoint lookup caches or attribute the 14,990,186,625 ns Exec phase to this Commit change. The sibling-owned Exec path remains outside this worktree's edits. No sibling messages were sent during this slice.

Physical segment observations:

- `{"allocated_bytes": 104914944, "kind": "workspace-physical-spool", "method": "verification_workspace_state", "observation_count": 20202, "observation_errors": 0, "open_physical_files": 101, "peak_bytes": 104914944, "phase": "after-workload-before-commit", "precision": "mutation-event-aggregate-allocation", "retained_segment_bytes": 104857600, "scope": "passive maintained Workspace counters; no independent verification"}`
- `{"allocated_bytes": 0, "kind": "workspace-physical-spool", "method": "verification_workspace_state", "observation_count": 20202, "observation_errors": 0, "open_physical_files": 0, "peak_bytes": 104914944, "phase": "after-commit", "precision": "mutation-event-aggregate-allocation", "retained_segment_bytes": 0, "scope": "passive maintained Workspace counters; no independent verification"}`

Producing identities:

- source_identity: `b69042b744fd21748ba88917c5abe4b2cd34175040d41a5c5b013831e477bb75`
- product_identity: `7fc8975b47aafeb1d617e60835cb7a87166fd572bb5ad4107829cfb42733e0eb`
- image: `sha256:c0d0eae8b922539759ba684d076054cda903d8f8626a81e7c7580276924a8247`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `4f10c2489f344103bab40c75b9f233cb7ce6af443318c7f016814d835ba33de9`

Host topology/container limits validated, protected preparation reused and master unchanged, cleanup PASS, no OOM/swap. Linux container lifetime peak 41,172,992 bytes. Host process peak RSS 122,552,320 bytes (previous single sample 142,573,568); RSS sampler peak 122,109,952, 1,506 samples, maximum sample gap 14,071,500 ns. These are individual observations, not a statistical distribution. Both original full-lifecycle subsecond targets remain unqualified, and final independent proofs remain deferred.

## Attempt 6 — coalesced final-record handoff, create-100

Original create-100 / seed1 on `d09cc7dd7`, complete lifecycle **TARGET_MISS**, `17126516710 ns`; Commit `772030417 ns`. Receipt and strict assessment: `benchmark-results/host-store/results/issue47-final-records-create100/`. One selected sample, protected preparation reused unchanged, no independent proof. The <=300ms exploration objective remains unmet.

- create: 9563959 ns
- exec: 16308660709 ns
- commit: 772030417 ns
- visibility: 82541 ns
- end: 36179084 ns

Measured Commit detail:

- content_ns: 579296458
- namespace_ns: 58928084
- candidate_finish_ns: 26390250
- object_admission_ns: 78753583
- checkpoint_ns: 19332750
- spool_retirement_ns: 3723292
- spool_retirement_scan_ns: 3493
- spool_retired_segments: 101
- snapshot_database_calls: 11
- object_admission_spill_readback_bytes: 4256258
- object_admission_memory_owned_bytes: 109178189
- object_admission_borrowed_copy_bytes: 0
- output_pipeline_ns: 480500250
- output_admission_ns: 332000194
- output_blocked_ns: 213946792
- output_consumer_idle_ns: 157011998
- output_queue_peak_bytes: 1048576
- spool_write_open_count: 101
- spool_write_ns: 179983171
- workspace_fence_ns: 192292
- candidate_objects: 82762
- candidate_bytes: 113434447
- inserted_objects: 81931
- inserted_bytes: 112321855
- reused_objects: 831
- reused_bytes: 1112592

Namespace/final-record time decreased from269,526,667 to58,928,084 ns. The content-labelled phase's work outside the pipeline decreased from169,054,415 to98,796,208 ns. Final selection and remaining structural admission were26,390,250 and78,753,583 ns. These observations support eliminating per-record journal I/O and the checkpoint lookup pass; they do not attribute Exec variation to this slice.

Candidate/inserted/reused object and byte totals are unchanged from attempts4–5. Owned selected memory delivery remains109,178,189 bytes and selected spill readback4,256,258 bytes. All101segments and104,857,600 retained bytes are retired by Commit. Retirement was3,723,292 ns, including3,493 ns in retain-predicate bookkeeping; the3,719,799 ns remainder includes owner drop/physical close, physical accounting and map iteration, not only a close syscall. Retain the earlier2.6ms and90.4ms observations unchanged.

Replan: retain this substantive finalization gain. The shared pipeline now dominates at480,500,250 ns, including332,000,194 ns consumer work; producer blocked time213,946,792 ns and consumer idle time157,011,998 ns overlap other timers. Next target is exact consumer membership/ownership handling and batched duplicate equality reads; do not increase producer count before reducing consumer service time. Delete still requires a new measurement after its shared record/reference work changes; no gains are assigned to its retained460ms result from this create sample.

Producing identities:

- source_identity: `a76e6d9fbbc93abe18e41fad0d16f1e008a569be478b6d24fd390d68ddc75920`
- product_identity: `7187d124792e1e30648f63e605787b3687c1a76bc8a0494d9882fd97e4e5d171`
- image: `sha256:738e367096503caa76b37af996604d70f852cbd204a1dcb514569cd3d3e4bd22`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `72327bbcd5d3abb037aea79c02445706971d2ad8a56cb3e5eb960f58d0ce39b9`

Cleanup PASS; no OOM/swap; Linux container lifetime peak 41717760 bytes. Host SQLite, no data mounts, 2CPU/2GiB container limits and120s/130s timing allowances preserved. No full-lifecycle qualification or independent proof.

### Receipt-parser correction for attempts5–6

Commit snapshot database calls are **6**, not11, in both attempts5 and6. The derived assessment parser had selected the preceding WorkspaceCreate receipt's same-named field from concatenated operation details. It now prioritizes `WorkspaceCommitReceipt`; both derived assessments record the previous and corrected values explicitly. Raw `perf.jsonl` files and all timings/bytes/object counts are unchanged. This correction preserves the earlier5–6-call snapshot result; it is not another lookup-cache optimization.

## Attempt 7 — shared consumer ownership/membership, create-100

Original create-100 / seed1 on `ebb98dbe7`: complete lifecycle **TARGET_MISS** under #47, `14942030209 ns`; Commit `679934459 ns`. The parent15-second harness reports PASS; this is explicitly not #47 acceptance. Receipt and strict assessment: `benchmark-results/host-store/results/issue47-consumer-create100/`. One selected sample, no independent proof.

- create: 10433416 ns
- exec: 14218591084 ns
- commit: 679934459 ns
- visibility: 77458 ns
- end: 32993792 ns

Measured Commit detail:

- content_ns: 514723750
- namespace_ns: 52006291
- candidate_finish_ns: 21261167
- object_admission_ns: 64173041
- checkpoint_ns: 17021792
- spool_retirement_ns: 3402375
- spool_retirement_scan_ns: 4454
- spool_retired_segments: 101
- snapshot_database_calls: 6
- object_admission_spill_readback_bytes: 4256258
- object_admission_memory_owned_bytes: 109178189
- object_admission_borrowed_copy_bytes: 0
- output_pipeline_ns: 431588250
- output_admission_ns: 286724334
- output_blocked_ns: 191189916
- output_consumer_idle_ns: 151823170
- output_queue_peak_bytes: 1048576
- spool_write_open_count: 101
- spool_write_ns: 213158270
- workspace_fence_ns: 215250
- candidate_objects: 82762
- candidate_bytes: 113434447
- inserted_objects: 81931
- inserted_bytes: 112321855
- reused_objects: 831
- reused_bytes: 1112592

Pipeline wall time fell from480,500,250 to431,588,250 ns; consumer service from332,000,194 to286,724,334 ns. Namespace was52,006,291 ns, candidate finish21,261,167 ns and remaining structural admission64,173,041 ns. Canonical candidate/inserted/reused totals and4,256,258-byte spill readback remain unchanged. The private high-cardinality ID index was not triggered here; no measured gain is assigned to that fallback replacement.

All101segments and104,857,600 retained bytes were released by Commit. Retirement was3,402,375 ns, including4,454 ns retain-predicate bookkeeping. Cleanup PASS, no OOM/swap, protected preparation and master unchanged, validated host SQLite/no-data-mount topology. Linux container lifetime peak41,439,232 bytes.

Replan: retain the consumer gain. The <=300ms Commit exploration objective remains unmet and the pipeline still dominates. Next substantive slice is shared deletion/namespace record-reference processing and ordered edge facts, followed by one original delete-100 measurement; do not assign create-only gains to delete. Avoid prolonged minor tuning or added producers while the consumer remains dominant.

Producing identities:

- source_identity: `ddc5adf1a0fd786b831df2c1a385e0f1291526241464a34b0c56116f6ccfc5c1`
- product_identity: `729862bf7622306d82d75b874913b44e32398c059ffab8a0e25940ec12482244`
- image: `sha256:d8ffbdd27afb893c680d41e9806d4e1d9f5ac0a3c18727129cf51ac039775501`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `bb92170a6ab2b38f9385d2deeab3b22c1d0275072beacb0ffa59727dd2b6e7c9`

## Attempt 8 — cumulative shared finalization/reference changes, delete-100

Original delete-100 / seed1 on `6fbaf2f9e`: complete lifecycle **TARGET_MISS**, `3538963293 ns`; Commit `311255958 ns`. Parent15s PASS does not satisfy #47. Receipt and strict assessment: `benchmark-results/host-store/results/issue47-reference-delete100/`. This is the first delete sample after the shared-spool/output/final-record/consumer/reference slices, so the reduction from460,068,958 ns is cumulative, not isolated attribution to edge observations.

- create: 9684334 ns
- exec: 3212627167 ns
- commit: 311255958 ns
- visibility: 70875 ns
- end: 5324959 ns

Measured Commit detail:

- content_ns: 798792
- namespace_ns: 306671625
- deletion_cursor_ns: 13676300
- deletion_records_ns: 281150379
- candidate_finish_ns: 98667
- object_admission_ns: 451584
- checkpoint_ns: 577792
- spool_retirement_ns: 208
- spool_retirement_scan_ns: 0
- spool_retired_segments: 0
- snapshot_database_calls: 1393
- object_admission_spill_readback_bytes: 0
- object_admission_memory_owned_bytes: 22318
- object_admission_borrowed_copy_bytes: 0
- output_pipeline_ns: 340083
- output_admission_ns: 18625
- output_blocked_ns: 0
- output_consumer_idle_ns: 281208
- output_queue_peak_bytes: 0
- spool_write_open_count: 0
- spool_write_ns: 0
- workspace_fence_ns: 13917
- candidate_objects: 9
- candidate_bytes: 22318
- inserted_objects: 9
- inserted_bytes: 22318
- reused_objects: 0
- reused_bytes: 0

Candidate totals remain9objects /22,318bytes, all inserted. Payload spill readback is zero; there are no physical payload spool segments before or after Commit. The8-query snapshot increase (1,385 to1,393) is retained; no cache improvement is claimed.

Replan: record/reference processing remains281,150,379 ns out of306,671,625 ns namespace time. The shared release path still looks up a spilled record once in `record_with_base` and again in `set_value`. Combine that ownership/location decision into one checked reference update, preserving current-over-prefetched precedence and zero-reference traversal. Admission451,584 ns and checkpoint577,792 ns remain non-priorities. The300ms Commit exploration objective is close but still unmet; no threshold-based PASS is invented.

Producing identities:

- source_identity: `3e4ffc3953ed322c84cf86d5956d5aa3ab782becfb8c318f9d1eac741af00118`
- product_identity: `59e706e9ffe59e57e66991511decee12997e91baca0e91f0d9868c002b77b6d9`
- image: `sha256:9a2016d153ccedc68237b34d707fed178ef0d3ed70d2fdfe8b2d395e11ebe1fe`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `43525efe0eb21bad661d9aadfbb1000c86086f0d03487b76bdbfa78e5c389f55`

Cleanup PASS; protected preparation and master unchanged; host SQLite/no data mounts validated; no OOM/swap. Linux container lifetime peak9,166,848 bytes. No independent proof.

## Attempt 9 — located reference update, delete-100

Original delete-100 / seed1 on `02af76b86`: complete lifecycle **TARGET_MISS**, `3531075500 ns`; Commit `304405125 ns`. Receipt and strict assessment: `benchmark-results/host-store/results/issue47-located-reference-delete100/`. One selected sample, no independent proof.304.4ms is not rounded into the <=300ms exploration objective.

- create: 11085208 ns
- exec: 3208255125 ns
- commit: 304405125 ns
- visibility: 75625 ns
- end: 7254417 ns

Measured Commit detail:

- content_ns: 662625
- namespace_ns: 299466542
- deletion_cursor_ns: 14840300
- deletion_records_ns: 270915432
- candidate_finish_ns: 147500
- object_admission_ns: 515375
- checkpoint_ns: 609708
- spool_retirement_ns: 250
- spool_retirement_scan_ns: 0
- spool_retired_segments: 0
- snapshot_database_calls: 1393
- object_admission_spill_readback_bytes: 0
- object_admission_memory_owned_bytes: 22318
- object_admission_borrowed_copy_bytes: 0
- output_pipeline_ns: 309250
- output_admission_ns: 16167
- output_blocked_ns: 0
- output_consumer_idle_ns: 254917
- output_queue_peak_bytes: 0
- spool_write_open_count: 0
- spool_write_ns: 0
- workspace_fence_ns: 14208
- candidate_objects: 9
- candidate_bytes: 22318
- inserted_objects: 9
- inserted_bytes: 22318
- reused_objects: 0
- reused_bytes: 0

The single-sample difference is modest: Commit311.3→304.4ms and record processing281.2→270.9ms. Keep the simpler exact-location update, but this disproves treating the repeated locator alone as the main remaining record cost. No unchanged reruns will chase the300ms line. Further deletion work needs a larger reduction in record loading/decoding, not admission/checkpoint tweaks.

Canonical totals remain9objects /22,318bytes, all inserted; snapshot calls remain1,393; payload spools and readback remain zero. Cleanup PASS, protected preparation/master unchanged, host SQLite/no data mounts and resource bounds validated, no OOM/swap; Linux container lifetime peak9,670,656 bytes.

Replan toward create's larger pipeline cost: prove the existing full-file streaming builder emits only final reachable objects before omitting its per-file selection scaffolding. Incremental/provisional paths keep selection, and no incomplete file output may enter admission. More producers alone still cannot meet300ms with the observed consumer and finalizer costs.

Producing identities:

- source_identity: `38e6b3a3a31c08c33f7a7837cd3acdbba5c6773e3445fe43e5218ab7960f2cc1`
- product_identity: `0f4dfec7a71defe81e020665702da5b508120317893422fd828393020b335f2d`
- image: `sha256:9dbb39e99f81b276dac1a8bd0ba0f010a33db2611fae9f8fefbf047991d21e87`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `80db9050a07847d0659431069f675da6d93076362ea03b80d20882507c6dde63`

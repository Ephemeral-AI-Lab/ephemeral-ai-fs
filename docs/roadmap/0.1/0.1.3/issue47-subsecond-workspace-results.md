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

## Attempt 10 — completed full-file finality, create-100

Original create-100 / seed1 on `c6683078c`: complete lifecycle **TARGET_MISS**, `16102944458 ns`; Commit `710825625 ns`. Receipt and strict assessment: `benchmark-results/host-store/results/issue47-complete-file-create100/`. This is the first create sample after edge/reference changes as well as full-file finality; whole-Commit changes are cumulative. One selected sample, no independent proof.

- create: 10952833 ns
- exec: 15350047042 ns
- commit: 710825625 ns
- visibility: 101666 ns
- end: 31017292 ns

Measured Commit detail:

- content_ns: 500782249
- namespace_ns: 40152167
- deletion_cursor_ns: 0
- deletion_records_ns: 0
- candidate_finish_ns: 21877125
- object_admission_ns: 62682708
- checkpoint_ns: 75893958
- spool_retirement_ns: 60861916
- spool_retirement_scan_ns: 9161
- spool_retired_segments: 101
- snapshot_database_calls: 6
- object_admission_spill_readback_bytes: 4256258
- object_admission_memory_owned_bytes: 109178189
- object_admission_borrowed_copy_bytes: 0
- output_pipeline_ns: 413859583
- output_admission_ns: 291501415
- output_blocked_ns: 204800792
- output_consumer_idle_ns: 129881794
- output_queue_peak_bytes: 1048576
- spool_write_open_count: 101
- spool_write_ns: 224506025
- workspace_fence_ns: 202208
- candidate_objects: 82762
- candidate_bytes: 113434447
- inserted_objects: 81931
- inserted_bytes: 112321855
- reused_objects: 831
- reused_bytes: 1112592

The full-file builder no longer constructs a reference index or performs per-file DFS/seen/order selection. Focused boundary checks establish identical roots and selected object sets, including repeated data and spill. Candidate/inserted/reused objects and bytes remain identical to attempts4–7, and selected spill readback remains4,256,258 bytes. This is actual removed construction work, not admission of provisional trees.

Pipeline wall was413,859,583 ns versus431,588,250 ns in attempt7; consumer service291,501,415 ns, producer blocked204,800,792 ns and consumer idle129,881,794 ns. The observed pipeline reduction is modest. Whole Commit710.8ms is **not a net performance win** over the prior679.9ms sample: checkpoint was75,893,958 ns, including60,861,916 ns retirement. Only9,161 ns was retain-predicate bookkeeping; the remainder includes owner drop/physical close, accounting and map iteration. Preserve this observation alongside earlier2.6/3.4/90.4ms retirement; do not subtract a favorable close time to claim a hypothetical result. All101segments and104,857,600 retained bytes were released by Commit.

Replan: keep the proven construction simplification and earlier substantial final-record/consumer gains, but stop further threshold-chasing micro-tuning. Create still needs a structural reduction in consumer work or overlap with finalization; more producers alone cannot supply the missing budget at the observed291.5ms consumer service plus remaining work. Delete remains304.4ms in its latest source-bound sample, with270.9ms record processing. Neither <=300ms exploration objective is demonstrated, and both complete lifecycle targets remain TARGET_MISS. Independent bounded proofs stay deferred until full performance gates pass.

Producing identities:

- source_identity: `d81d7d170760524589246e84976f4451b1fbfedae8736bdebf48835a39df9cdd`
- product_identity: `cf7bec19be0fa50a456aa80b926ba118fa8aa345e9d69aaf0cedb8114d3d026a`
- image: `sha256:3d2d4d8579a37adc8e73bfce94abefd773ddacb338015282c10ad67fd8447222`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `d675b23abbb5b11449f11c0d6e4e843d26ae69b47c458f502eed1ef603dd6f07`

Cleanup PASS, protected preparation/master unchanged, validated host SQLite/no-data-mount topology and existing resource/timing bounds, no OOM/swap. Linux container lifetime peak41,443,328 bytes. No sibling messages or Exec-owned source edits were made during these slices.

## Attempt 11 — narrow consumer attribution for the sub400 plan

Original create-100 / seed1 on `32b24ccd1`, instrumenting the unchanged checked consumer: Commit `644927625 ns`, full lifecycle **TARGET_MISS** `15351515333 ns`. This is attribution, not an optimization claim for adding timers. Receipt: `benchmark-results/host-store/results/issue47-consumer-attribution-create100/`.

- create: 9981833 ns
- exec: 14663940125 ns
- commit: 644927625 ns
- visibility: 94250 ns
- end: 32571500 ns

- content_ns: 495834333
- namespace_ns: 39965250
- deletion_cursor_ns: 0
- deletion_records_ns: 0
- candidate_finish_ns: 21515375
- object_admission_ns: 63269750
- object_admission_authentication_ns: 130870335
- object_admission_sort_ns: 4620210
- object_admission_begin_ns: 508544
- object_admission_insert_ns: 137456333
- object_admission_commit_ns: 33823247
- checkpoint_ns: 20390083
- spool_retirement_ns: 5776709
- spool_retirement_scan_ns: 3751
- spool_retired_segments: 101
- snapshot_database_calls: 6
- object_admission_spill_readback_bytes: 4256258
- object_admission_memory_owned_bytes: 109178189
- object_admission_borrowed_copy_bytes: 0
- output_pipeline_ns: 411530458
- output_admission_ns: 286477125
- output_blocked_ns: 201266541
- output_consumer_idle_ns: 131874958
- output_queue_peak_bytes: 1048576
- spool_write_open_count: 101
- spool_write_ns: 201624107
- workspace_fence_ns: 185334
- candidate_objects: 82762
- candidate_bytes: 113434447
- inserted_objects: 81931
- inserted_bytes: 112321855
- reused_objects: 831
- reused_bytes: 1112592

Authentication130,870,335 ns is material; sorting4,620,210 ns is small. Choose exactly follow-upA from the approved sub400 plan: carry complete identity+framing validation in immutable owned output so memory delivery does not repeat the pass. Construction must still compute identity and validate framing; spill/durable reads still authenticate; collisions remain exact. No trust toggle, provisional admission, producer framework or memory-limit increase. After this follow-up and one selected sample, reassess against the practical400–450ms milestone, not the superseded300ms ambition.

All canonical totals/readback and physical retirement checks remain unchanged; cleanup PASS, no OOM/swap, protected preparation/master unchanged and topology/resource limits validated. No independent proof. Delete304.405ms is retained without a timing-only rerun.

Producing identities:

- source_identity: `e4f4b99df56401bf882e471d55af3e6315c9de64b68147eb69fb86368635efff`
- product_identity: `48022cae098aa049299cf727fdedaebf98946f6a4cb3dec08408a1591b77d858`
- image: `sha256:f96306930d3930dd83e7c75813de9ac95db2187b2dbbc0c7dc48d8ecf4aa6f68`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `fbb9fc021c7dbfb46f202932b8eb3ba2bfdc98b5d8c139a5c9c7416c39687422`

## Attempt 12 — validated owned output, sub400 reassessment

Original create-100 / seed1 on `a07f053bb` (follow-upA, including `49dd521fe`): Commit `636195334 ns`, full lifecycle **TARGET_MISS** `15893156752 ns`. Receipt and strict assessment: `benchmark-results/host-store/results/issue47-validated-owned-create100/`. This does **not** meet strict sub400 or the user-accepted approximate400–450ms Commit milestone. One selected sample; no independent proof.

- create: 10377000 ns
- exec: 15217984000 ns
- commit: 636195334 ns
- visibility: 81709 ns
- end: 28518709 ns

Measured Commit detail:

- content_ns: 455958791
- namespace_ns: 39262875
- deletion_cursor_ns: 0
- deletion_records_ns: 0
- candidate_finish_ns: 21711042
- object_admission_ns: 85498041
- object_admission_authentication_ns: 0
- object_admission_storage_authentication_ns: 8010009
- object_admission_sort_ns: 4471206
- object_admission_begin_ns: 695041
- object_admission_insert_ns: 160799664
- object_admission_commit_ns: 45873041
- checkpoint_ns: 28999209
- spool_retirement_ns: 15870750
- spool_retirement_scan_ns: 4793
- spool_retired_segments: 101
- snapshot_database_calls: 6
- object_admission_spill_readback_bytes: 4256258
- object_admission_memory_owned_bytes: 109178189
- object_admission_borrowed_copy_bytes: 0
- output_pipeline_ns: 374065834
- output_admission_ns: 179085332
- output_blocked_ns: 72107460
- output_consumer_idle_ns: 200205459
- output_queue_peak_bytes: 1048576
- spool_write_open_count: 101
- spool_write_ns: 230743046
- workspace_fence_ns: 188833
- candidate_objects: 82762
- candidate_bytes: 113434447
- inserted_objects: 81931
- inserted_bytes: 112321855
- reused_objects: 831
- reused_bytes: 1112592

The duplicated memory authentication loop is removed. Construction still computes identity once and now retains complete framing validation in immutable ownership; no duplicate hash was moved into construction. Required fresh selected-spill authentication is separately8,010,009 ns. Candidate/inserted/reused totals remain82,762/81,931/831 objects and113,434,447/112,321,855/1,112,592 bytes. Memory-owned delivery remains109,178,189 bytes; spill readback4,256,258 bytes; borrowed delivery copies zero.

Compared with attribution attempt11, consumer service fell286,477,125→179,085,332 ns and pipeline wall411,530,458→374,065,834 ns. Consumer idle grew131,874,958→200,205,459 ns. Whole Commit644,927,625→636,195,334 ns is only a modest single-sample change, not the practical milestone. Outside-pipeline time is262,129,500 ns. SQL begin/insertion/commit totals and retirement were higher in this observation; retain those costs rather than substituting favorable prior values. Retirement15,870,750 ns includes4,793 ns retain-predicate bookkeeping; all101physical segments and104,857,600 retained bytes are released by Commit.

**Required reassessment after the active finality slice and one evidenced follow-up:** retain the validated ownership/consumer improvement, but stop this focused tuning attempt without declaring the milestone complete. The remaining critical path is producer availability/overlap plus the serial preparation/structural-delivery tail. Reaching the practical range requires a structural reduction or bounded overlap of completed-file preparation/final delivery; eliminating another small lookup, changing transaction/cache knobs, or rerunning unchanged work will not establish it. No additional producer framework, worker count, all-tier/seed/median campaign, #49 prerequisite, or second automatic follow-up is started.

Delete's retained304.405ms is sufficient for the revised Commit objective and is not rerun solely for timing. This is not a current-source final qualification pair. Full-lifecycle #47 acceptance and final independent proof obligations remain distinct and unmet.

Producing identities:

- source_identity: `961f73e6f08e3868935c469d34c5e77dd8d906832ca167ece0d1f378844c15a5`
- product_identity: `4d1f3e9348a516fa4815defe8f6279c9ada45f20d76285b5d56e14a629bf74c8`
- image: `sha256:8c96fff17e32ef1da16a4fd2f97e8e6e6b305a4cec03a894260363fa54bb1b5f`
- harness_identity: `c3610164231c40bfa3e77c27e5c1526b5e9656c046c9e382d095b0d6c75d1abc`
- input_identity: `5bdab025fd1ebdf02a8ccd5d7840cf42ad543b521accf2694375a5ed1c54e545`

Cleanup PASS, protected preparation/master unchanged, validated host SQLite/no-data-mount topology and existing120s/130s allowances; no OOM/swap. Linux container lifetime peak41,185,280 bytes. No sibling messages or Exec-owned edits.

## Attempts 13–14: approved mixed bulk v3 migration and tier100 pair

The original-workload campaign stops at create Commit636.195ms and retained delete Commit304.405ms. Those results and all earlier raw receipts remain unchanged. Specification commit `66983181a` freezes the approved replacement workload before implementation; migration commit `0ac1ebcf4` changes benchmark registration, bulk fixture/expected/apply, sampled verification, fixture/input/cache identity, classifiers and examples. No product code changes in this migration. These are new-workload absolute observations, not a speedup attributable to product optimization.

Active high-tier IDs are `tiny-bulk-{create,delete}-{100,500}-mixed-v3`; family cardinality remains20. Tier100 affects1,000files/104,857,600B (1x52,428,800B,800x4096B,194x246,995B,5x246,994B). Tier500 affects5,000files/524,288,000B (3x104,857,600B,4000x4096B,936x193,913B,61x193,912B). Both keep200files/1,048,576B witness. Populated namespace totals are1,200/5,200files and105,906,176/525,336,576B, with273/293directories including root. Existing shard paths use5/25target shards; other families and low-tier/individual cases retain their definitions.

### Focused checks and custody

Both tier distributions, all three recipe seeds, exact ordinal assignment, create/delete equivalence, witness, shared path shape, registry20 and every large-file range/absence passed the focused Rust contract check. The canonical/native sampled-reader check passes, including non-prefix byte corruption rejection.15 runner checks and8 proof-selection checks pass. Released-binary registry and fixture-info checks confirm all four IDs/counts/bytes and matching create/delete populated manifests. This is product-free tier500 definition/identity work, not a tier500 performance run.

The populated independent manifest includes paths, per-file lengths/digests, directories and metadata. Its digest joins the prepared plan and selected input identity. Fixture profile `tiny-bulk-mixed-v3` prevents reuse of old target preparations. An immutable oracle-identity cache bound to host binary/family/case/seed avoids full large-file oracle generation during later bounded proofs; missing/corrupt proof custody fails closed. Large files have64KiB ranges at beginning, midpoint and end; every large file is included. Tier100 create selects7files/340,992B, tier500 selects9files/734,208B per view, including witness; delete checks corresponding target absence and retained witness. Unselected paths/bytes and exhaustive inode/object/reference census remain omitted. Independent proof execution is still NOT_RUN.

Exact seed1 fixture identities (create/delete share the populated manifest but have different initial trees):

| Tier | Populated manifest SHA-256 | Create input-plan SHA-256 | Delete input-plan SHA-256 |
|---|---|---|---|
|100|`fcf05c8961a938ec8d9853901c0c555f8cc5df7afe98af2c31261b85869e6c5d`|`fda015d62663ae9f90f0191faa7353b26de3a5418667aa711c833790408106ad`|`4b4db51ede8725c2837bfef088e9f5a811b49ead803d4b38cca87cf77ebe7aa5`|
|500|`f9bf606a551c00deaa3d0ca01346bf174739205f985e2ac04cf555f2c5163b2c`|`2a50f86875c8a84d7fbb0693899c18dae6e2558f61f2448c59e4e780f76ab13e`|`cf51b7e2b45e3907cb96ec3dd92c654e7c2b8f88d00d6742b01f7747dd5ed969`|

Machine-readable definition evidence: `benchmark-results/host-store/results/issue47-mixed-v3-definition/definition.json`; paired assessment: `.../pair-assessment.json`.

### One serial tier100 sample per operation

Both samples use source `0ac1ebcf43b2cfcd266703b4bb40bfbea8be7e5a`, source seal `e80a564bcdbdaf38b3b8f18a8b8b4e50b4161a2dc3f63c50a4308c8132960417`, product seal `4d1f3e9348a516fa4815defe8f6279c9ada45f20d76285b5d56e14a629bf74c8`, host binary SHA-256 `bf38adb473a5b87973a38a77267095af357032c28bf615797abc7841fbfe9480`, and image `layerfs-bench-infra:e80a564bcdbdaf38` / `sha256:9b4638f099f0d2e0ed8a739a533c2e4f964f53bc9eec6538181e2f08ba70ed50`. Product seal is unchanged from the final original-workload Commit slice. This does not claim adoption of additional unpublished Exec-track work.

| Revised tier100 case | Create session ns | Exec ns | Commit ns | Visibility ns | End ns | Complete lifecycle ns | Strict #47 performance |
|---|---:|---:|---:|---:|---:|---:|---|
|create|8,932,250|956,095,375|370,395,375|75,542|6,622,875|1,342,121,417|TARGET_MISS|
|delete|10,784,792|280,367,875|15,033,084|68,708|2,991,291|309,245,750|PASS|

Each row is one observation (n=1; median=min=max), not a distribution. Both parent15s classifiers PASS. Both Commit observations satisfy the practical400ms aim; the pair does not satisfy strict complete-lifecycle qualification because create exceeds1,000,000,000ns. Final proofs remain deferred. #46/#47/#48/#39 are not closed; #49 is not started.

Raw receipts:

- `benchmark-results/host-store/results/issue47-mixed-v3-create100/perf.jsonl`, SHA-256 `93a116a6281d2da42b7230cc9ca335d0d0df8034dbac8728a30f3b6f85ad23af`, selected input `94478fdc58ea9c3e677dc734ed375029e30b84958775a24a5b43ade4c3624027`.
- `benchmark-results/host-store/results/issue47-mixed-v3-delete100/perf.jsonl`, SHA-256 `af70bce5b3d2e4c83013ac8758c510e7c8973370ef67b3c5e48e55fbd547fe1b`, selected input `57afdc6a47c2bf3c2d694be5667c04a8dc60c74843993c3a7034350bcf2a151a`.

Each launch initially encountered the occupied shared measurement lock and started no sample; the later serial launch collected the sole observation. No overlapping build/measurement was introduced. Host SQLite/Workspace/spools, daemon/workload-only Docker, no data mounts,2CPU/2GiB/no-swap/256PID container and120s/130s allowances remain. Both samples report zero benchmark verifier/reopen/injection in performance, successful cleanup, unchanged prepared masters and0spoolfiles/bytes after Commit. Container peaks11,390,976B create and4,874,240B delete; OOM/swap0. Samples used closed-quiescent byte copies, not APFS clones or a cold-cache claim. Host resources are separately retained in raw records and the pair assessment.

Create performed exactly1,000filewrites/104,857,600B,1,049workload pwrite calls and1,139metadata normalizations. Delete performed exactly1,000unlinks and138rmdirs. Create Commit: pipeline264.614459ms, nested consumer73.407908ms, consumer idle191.688968ms; checkpoint92.428458ms including physical retirement91.601959ms of103segments. Selected canonical data is53,065,971B memory-owned and52,595,430B spill-readback (the new50MiB file exceeds the existing bounded per-file private buffer); required spill authentication47.951924ms. Candidate9,600objects/105,661,401B, inserted9,586/105,660,222B, reused14/1,179B. No cleanup is moved outside Commit. Delete Commit namespace12.107084ms, deletion records10.106586ms, cursor0.798833ms, checkpoint0.157459ms.

The remaining complete-create gap is342.121417ms. Exec metadata normalization alone is548.364209ms in this new sample. This identifies remaining work for reassessment; this migration does not begin another original-workload Commit tuning campaign or claim those nested timers are additive recoverable savings.

## Attempts 15–16: authorized complete tier500 mixed-v3 pair

The user's request to increase the limit and obtain full tier500 runs supersedes the earlier tier500 deferral. Specification amendment `6fea9d2b5` precedes harness change `459eefaa0`. The explicit `--product-timeout` defaults to120seconds; the selected tier500 runs use600seconds product/630seconds outer/600seconds preparation. The existing watchdog, cumulative checks and phase receipts all use the configured limit. No pass threshold, product engine, workload size, resource cap or independent proof deadline changes. Two focused watchdog/boundary checks and16 runner checks passed, including live600second-budget enforcement using a bounded10ms remaining allowance. Host and image builds were serial under the shared lock. Two initial lock rejections started no tests.

One seed1 sample per operation, on source `459eefaa01db3fc08c7b40c930da5981d8b04bc1` / seal `b6159377eadfc07806aaba102525a8b5d7b10c8ef2c0b48c834f3b2fb60731ca`, host binary SHA-256 `ba22f2936ef28e07a74fc6a5a6dc5abc822b36da59c52958e1c16ae4290e20cf`, image `layerfs-bench-infra:b6159377eadfc078` / `sha256:ff14c13a66f1224fe5879d682b0345015e52a626eab381a5f9f27995ce985002`. Product seal remains `4d1f3e9348a516fa4815defe8f6279c9ada45f20d76285b5d56e14a629bf74c8`. The fixture recipes and exact populated manifest are unchanged from attempts13–14's tier500 definition checks.

| Revised tier500 case | Create session ns | Exec ns | Commit ns | Visibility ns | End ns | Complete lifecycle ns | Family15s performance |
|---|---:|---:|---:|---:|---:|---:|---|
|create|10,247,667|4,307,807,708|1,862,498,042|96,875|19,426,417|6,200,076,709|PASS|
|delete|9,871,667|874,646,083|50,838,875|70,291|4,588,333|940,015,249|PASS|

Every phase receipt confirms `limit_ns=600000000000`. Each row is n=1, median=min=max; no distribution, simple cross-tier scaling or cross-profile product-speedup claim. No tier500 subsecond or400ms Commit gate is introduced. This does not qualify the full20-case family or close the tier100 #47 target, whose retained create lifecycle is1,342.121417ms. No unchanged tier100 rerun or independent proof was performed after the allowance-only harness change.

Full target work is retained: create5,000files/524,288,000B via5,297workload pwrite calls,5,159metadata normalizations and required root sync; delete5,000unlinks/158rmdirs and required sync. The separate200-file/1MiB witness remains in the fixture/oracle contract. Both runs report0benchmark verifier/reopen/injection work in performance, cleanupPASS, unchanged prepared masters, OOM/swap0 and0physical spoolfiles/bytes after Commit. Container lifetime peaks18,952,192B create and5,255,168B delete; host resources are retained separately. Host SQLite/no data mounts and2CPU/2GiB/256PID container constraints hold.

Create Commit: output pipeline1,500.741917ms, nested consumer452.420386ms, consumer idle1,051.685092ms; checkpoint306.409708ms including299.854334ms physical retirement of516segments. Selected spill/readback316,687,246B and memory-owned211,344,309B; required fresh-spill authentication291.970257ms. Exec metadata normalization2,558.220918ms. These identify remaining costs without starting a new tuning campaign or treating nested times as additive savings. Delete Commit namespace48.105875ms, deletion records41.410426ms, cursor3.544041ms.

Raw receipts and exact identities:

- `benchmark-results/host-store/results/issue47-mixed-v3-create500/perf.jsonl`, SHA-256 `fc19ed12c45fe6f1a6cb3f6bd0f07c519578aa5b918072782105117410411394`, selected input `f9524311661812fd26332de47f379b053121602ab33a059043b4b1d5522acce3`.
- `benchmark-results/host-store/results/issue47-mixed-v3-delete500/perf.jsonl`, SHA-256 `1832a640eefa649fcb515b11d20771ce9954d00187a9532a9a7456f59d8b0965`, selected input `2a752bda5e8a7022f2f7e97a4a1bf4ffb12399a52d74df8187cf3a1b67a791a5`.
- `benchmark-results/host-store/results/issue47-mixed-v3-definition/tier500-pair-assessment.json` balances all lifecycle phases, verifies configured limits, exact operation counts, shared source/binary/image identities, physical retirement and cleanup. Independent proofs remain NOT_RUN; #49 remains unstarted.

## Attempts 17–20: tier1/tier10 shared-path coverage refresh

The user requested four missing low-tier observations before #49. One seed1 complete performance sample was collected per `tiny-bulk-{create,delete}-{1,10}-compact-v2` on the delivered product, serially through the global lock. No product, workload, low-tier definition or benchmark source changed; no unpublished Exec-track work was adopted. Normal120second product/130second outer allowances are confirmed by headers and phase receipts. Tier100/500 were not rerun. No independent proof, new low-tier target, optimization campaign or issue closure was performed.

The unchanged compact witness is50files/1MiB. Tier1 affects50files/1MiB; tier10 affects500files/10MiB. Complete populated trees therefore contain100files/2MiB and550files/11MiB, plus directories. This differs intentionally from the200-file witness in the high-tier mixed-v3 profile. Creation byte counts are measured writes; deletion byte counts below describe the affected input payload, not content reads or rewritten bytes.

| Case (all compact-v2) | Affected files / bytes | Exec ns | Commit ns | Complete lifecycle ns | Cleanup | Family15s performance |
|---|---:|---:|---:|---:|---|---|
|create1|50 / 1,048,576|206,973,125|9,743,625|230,565,041|PASS|PASS|
|delete1|50 / 1,048,576|96,636,750|7,095,500|114,032,375|PASS|PASS|
|create10|500 / 10,485,760|550,552,083|40,627,167|604,193,624|PASS|PASS|
|delete10|500 / 10,485,760|199,131,625|10,010,417|220,390,332|PASS|PASS|

Each row is one observation (n=1, median=min=max), not a distribution. All four phase sums balance exactly. Create performs50/500filewrites and1,048,576/10,485,760B; delete performs50/500unlinks. Required metadata normalization and root sync remain. All selected operation counts checked against historical receipts match. Both creation samples and both deletion samples have0physical spool files/bytes after Commit, unchanged prepared masters, cleanupPASS, benchmark verifier/reopen/injection0 and OOM/swap0. Tier10 create reused the compatible preparation populated during tier1 create; other worktree preparations were cache misses. All writable samples use closed-quiescent byte copies, not a cold-cache claim. Host resources are recorded separately.

### Shared mechanism adoption

All four use ordinary `lifecycle.rs::build_candidate(Commit)` → `changes.rs::build_frontier_candidate` → `LayerStackStore::construct_workspace_files`, one ordered inode/reference finalizer, shared checked structural admission and `install_checkpoint`. There is no tier-selected product engine.

Creation uses the existing PieceTree backed by shared segment extents, `FrozenFile::build`/`ObjectBuffer::build_complete_file`, the shared bounded `run_finalized_output` driver, immutable authenticated owners and checked consumer. Tier1 retires1physical segment for50files; tier10 retires11for500files. Selected output is1,130,913B/10,780,341B memory-owned,0spill-readback,0borrowed copies and0repeat-memory authentication in both. Pipeline walls are2.136916/21.536000ms, with checkpoint0.763792/9.749459ms including retirement0.605250/9.318958ms. These are nested counters, not additive savings.

Deletion enters the same driver but has no file content output: output queue bytes0 and retired payload segments0. It exercises the shared directory/reference deletion algorithm, ordered final inode updates and checked consumer for7structural objects/12,502B in each case. Record/reference processing is4.053753/5.984241ms; checkpoint0.140334/0.162208ms. Thus shared finalization/checkpoint/consumer adoption is demonstrated for delete without claiming a payload-spool benefit on a payload-free operation.

### Historical comparison and limits

The four saved references have product `77aff139adefb45e5175cddaffb6e4e9acb9a5ceaae027c05ae4a9eae6271802`; the current product is different. Initial fixture compatibility dictionaries (including exact input-plan SHA, byte/file totals, schema and seed) match, and observed write/unlink/mkdir/rmdir/metadata-normalization/root-sync counts match. Definitions are compatible; product/source/harness/binary/image differ. These are historical unpaired observations, not a controlled product-speedup estimate. Current absolute results are primary. Old raw receipts are read-only and unchanged.

| Case | Historical Exec ms | Current Exec ms | Historical Commit ms | Current Commit ms | Historical full ms | Current full ms |
|---|---:|---:|---:|---:|---:|---:|
|create1|187.551125|206.973125|45.502000|9.743625|245.388250|230.565041|
|delete1|114.423167|96.636750|6.740833|7.095500|132.160375|114.032375|
|create10|481.299250|550.552083|193.929583|40.627167|687.995042|604.193624|
|delete10|195.842792|199.131625|23.885833|10.010417|232.338708|220.390332|

Retain the increases: tier1 delete Commit rises0.354667ms; create Exec rises19.422000ms at tier1 and69.252833ms at tier10, and tier10 delete Exec rises3.288833ms. Full lifecycle is lower in all four observations, but these single historical comparisons do not isolate causality or justify reruns to select a favorable result.

### Exact custody

All four: source `459eefaa01db3fc08c7b40c930da5981d8b04bc1`, source seal `b6159377eadfc07806aaba102525a8b5d7b10c8ef2c0b48c834f3b2fb60731ca`, product `4d1f3e9348a516fa4815defe8f6279c9ada45f20d76285b5d56e14a629bf74c8`, host binary SHA-256 `ba22f2936ef28e07a74fc6a5a6dc5abc822b36da59c52958e1c16ae4290e20cf`, harness `9cbde969b686b82163e16dfee411cead2831b76b1a95375218da8bd757a49813`, image `layerfs-bench-infra:b6159377eadfc078` / `sha256:ff14c13a66f1224fe5879d682b0345015e52a626eab381a5f9f27995ce985002`. This is the same delivered product used for the retained mixed-v3 observations.

- **tiny-bulk-create-1-compact-v2**: raw `benchmark-results/host-store/results/issue47-shared-create1-compact-v2/perf.jsonl`; SHA-256 `8de24c5e7c73461f3e6af3557a2eccc55ed5f2a345408dcfd1ca8096608b3ea4`; selected input `969cb46947836e7e12d47bd0734ecb9c92656aea13c9d05215c7971f61a4efe6`; fixture plan `9efa8bc2a72d30d2ae27c02cd5970cf5334a2beb142360e1178436154f5b0246`; container peak7,782,400B. Historical receipt: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/results/issue46-collection-tiny-bulk-create-1-compact-v2/perf.jsonl`.
- **tiny-bulk-delete-1-compact-v2**: raw `benchmark-results/host-store/results/issue47-shared-delete1-compact-v2/perf.jsonl`; SHA-256 `774b8c1f2fed2e20eec6cfd7dd093ca37784efca5550b4940fbe24376c9bf36d`; selected input `46a11657d71387afa6e49330900dc469491d355d744df09983ca6aaae4a1cd19`; fixture plan `6db2ad97220cf1bc6b6651e8ecbc550d3e5cb517b1f15f7ed5dd97a640cbdc7b`; container peak4,866,048B. Historical receipt: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/results/issue46-collection-tiny-bulk-delete-1-compact-v2/perf.jsonl`.
- **tiny-bulk-create-10-compact-v2**: raw `benchmark-results/host-store/results/issue47-shared-create10-compact-v2/perf.jsonl`; SHA-256 `5d3d40158306e9505452f2864aeeabb5cc259b6aa22d0766dbe8cf6140718869`; selected input `537dafab30733ef6e54ee79d847d178ce0c4ef0c0b44217330676dd1edabd093`; fixture plan `9efa8bc2a72d30d2ae27c02cd5970cf5334a2beb142360e1178436154f5b0246`; container peak11,608,064B. Historical receipt: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/results/issue46-finalrefs-create10/perf.jsonl`.
- **tiny-bulk-delete-10-compact-v2**: raw `benchmark-results/host-store/results/issue47-shared-delete10-compact-v2/perf.jsonl`; SHA-256 `e014b223443e6d0691b7281477983ca34572c110c77983ff56faa6490e93c8f4`; selected input `ff36f18861202f2f9ebc231ef85116621d0180fdd6272d28dadce25786b4cbc9`; fixture plan `000598b5116beebbc5cf8f60759c27a8d5885f966fcab3008be8a36a1298bd8d`; container peak4,837,376B. Historical receipt: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/results/issue46-collection-tiny-bulk-delete-10-compact-v2/perf.jsonl`.

Machine-readable assessment: `benchmark-results/host-store/results/issue47-compact-low-tier-assessment.json`. It retains all current and historical identities/raw hashes, full phase sums, exact fixture compatibility, operation counts, resource/cleanup evidence and mechanism mapping. Coverage refresh is recorded; #49 remains unstarted. Independent proofs and full-family qualification remain pending; retained tier100 create still misses the strict #47 complete-lifecycle target.

## Issue49 shared-producer refactor (no new performance attempt)

After the four low-tier observations, the user authorized #49. Plan `cc6d3b0ef`, completion-facts slice `2570c0d81` and shared task/result slice `da40c431c` are implemented. Native initialization (direct and fallback) and ordinary Workspace Commit now share task claim/cancel/step/result-finish/writer-finish/drain/join behavior. Completed full-file root/length facts come from the existing rope builder; native duplicated file construction and Workspace single-result/whole-dirty-worker loops are removed. Workspace generation-bound result ordering uses bounded worker journals and fixed ordinal metadata slots. Production remains one Commit worker and native's existing cap; preview, incremental/capture semantics and lifecycle owners remain separate.

The [#49 implementation/adoption/deletion/test ledger](issue49-shared-producers-plan.md#implementation-and-adoption-ledger) records resource accounting, exact tests, the resolved intermediate compiler obstacle and retained semantic boundaries. Store suite51passed/1existing ignored plus the additional partitioned construction check; final Workspace58passed; file-edit12/reconciliation2passed; targeted core/native checks and SDK compilation pass. No benchmark performance or independent proof run was added. Attempts1–20 remain immutable evidence for their recorded sources; they are not re-attributed to the new refactor. No parent issue is closed and no unpublished Exec work is claimed.

## Attempts 21–22: current-source numbers after issue49

After #49 implementation, the user asked whether it was fully done and requested numbers. One complete seed1 sample per revised tier100 operation was therefore collected on the delivered refactor, with unchanged one-worker Commit default and normal120/130second allowances. This is a scoped post-refactor pair, not an all-tier campaign or independent proof. Host/image builds and both measurements were serial under the shared lock.

| Case | Create session ns | Exec ns | Commit ns | Visibility ns | End ns | Complete lifecycle ns | Strict #47 performance |
|---|---:|---:|---:|---:|---:|---:|---|
|tiny-bulk-create-100-mixed-v3|8,974,334|917,203,209|358,675,583|75,750|6,462,000|1,291,390,876|TARGET_MISS|
|tiny-bulk-delete-100-mixed-v3|12,330,750|268,385,792|14,803,083|70,625|2,417,875|298,008,125|PASS|

Each is n=1, median=min=max. Both observed Commit durations are below400ms; complete create remains291.390876ms above the strict1second threshold. #49's implementation/tests are complete; overall #47 performance/proof qualification is not. No unpublished #48 implementation was adopted and no issue was closed.

Exact matching source: `6de381837d1c22e5eb21dfaf446f85c9c30ea6ec`; source seal `4ce40e2c3c3573902e3820d8e16c5f1be793f42dbeb14e4bb72ed4274557fc1f`; product seal `3fb2f18f1c636b8c3aa8b2a901bb3ca2a56cb075e674b6f520e8c685ca2b1053`; host binary SHA-256 `4de1c0e44903299865434e94c1378f00d03545e930719e0a81e205f792ac9be7`; image `layerfs-bench-infra:4ce40e2c3c357390` / `sha256:47d0d03f9a7079292c6ab5fa076a5392c7aa6e79a7f1e47a76467f89eed15e0e`.

- Create raw `benchmark-results/host-store/results/issue49-shared-producers-create100/perf.jsonl`, SHA-256 `93f5f439f80f30c0ca6efa472582995cd5be664890bbe337a6ffa55218f67e28`; selected input `0f74e6e62a5729f144e1f29e84d94c40984e4f9ebd6b947114d6d72659b6d580`.
- Delete raw `benchmark-results/host-store/results/issue49-shared-producers-delete100/perf.jsonl`, SHA-256 `391df7cabf8ff78707c417dbbfacc2d38b71cb57c571510d27a19ead2fef0756`; selected input `35beeec41c290dafcf98142cddde294d857b078e15682537f2c54e8e686919c2`.
- Machine assessment: `benchmark-results/host-store/results/issue49-shared-producers-assessment.json`.

Both retain exact fixture-info identity from attempts13–14,1,000affectedfiles/100MiB plus200-file/1MiB witness, complete operations, cleanupPASS, unchanged prepared masters and0physical spoolfiles/bytes after Commit. OOM/swap0; container peaks11,468,800B create/4,866,048B delete, with host resources separately retained. Performance verifier/reopen/injection counts are0. Independent proofs remain NOT_RUN.

Historical create Commit370.395375→358.675583ms and delete15.033084→14.803083ms are unpaired observations with different product/source/harness/image identities, not a controlled speedup claim. Create pipeline is essentially unchanged264.614459→264.543417ms; retirement91.601959→80.705000ms accounts for most of the Commit difference. Selected memory/spill bytes remain53,065,971/52,595,430. This supports reporting current absolute behavior without claiming a material pipeline acceleration from code reuse. The empty delete pipeline now measures0.033875ms with no producer work, consistent with its zero-task path.

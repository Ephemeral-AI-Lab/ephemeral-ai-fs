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

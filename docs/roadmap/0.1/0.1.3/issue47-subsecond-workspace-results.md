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

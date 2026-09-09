# Issue 95 comparison reuse correctness review

Review scope: project-owned initialization comparison reuse in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95`. Initial reviewed product commit: `1d365f5a89b5cddd20cb2cd7132f11cfcb8cb36c`. Follow-up source observed when writing this record: `c48bb4903f456136ccbcdba78de38b9042d2755a`, with `COMPARISON_REUSE_ENTRY_BYTES = 2048`. This is a source review and test mapping, not benchmark or terminal execution evidence. The review agent ran no builds, tests, benchmarks, or database analysis and opened no Store outside test source construction.

## Finding and correction

The original 512-byte per-entry B-tree charge did not conservatively cover a singleton node allocation. A Rust B-tree node allocates multiple slots immediately; for the current 32-byte ObjectId key and Location/Vec value, a node can exceed 512 bytes even with one live entry. Initialization supports oversized singleton canonical inputs, so a single operand near the 2 MiB retention limit could pass the nominal charge while exceeding that reservation.

The follow-up uses 2048 bytes per retained entry, plus the Vec's actual capacity. This conservatively covers the present full node allocation even for a singleton. The limit remains 2 MiB and comes from the existing 16 MiB uniqueness/filter reservation. No reusable exact B-tree allocation helper was found nearby. The simple conservative charge avoids dependence on asymptotic occupancy. The near-limit test also checks spare Vec capacity, overflow bypass, and replacement after saturation.

## Reviewed invariants

1. Activation is initialization-only: `new_for_initialization` creates a coalescing session, and `with_session` reserves comparison reuse only for that session. Ordinary admission retains its original comparison route and no retained payload allocation.
2. ObjectId membership never suffices for reuse. `probe_incoming` still obtains current object locations. A retained hit must match the complete Location, then canonical length and every supplied byte. A different length or same-length collision fails admission.
3. First-use storage authentication is preserved. A miss passes through `admission::compare`, which checks canonical length and uses authenticated packed reads. Only after that comparison succeeds are equal supplied bytes moved into retained state. The retained bytes do not originate from an unverified ID match.
4. Native FULL/PREFIX reconstruction and legacy FULL/DELTA authentication are unchanged. Packed extraction and decoder checks are bypassed only for an operand already validated at the same immutable physical location, under the same owner.
5. The cache is owner-local. The session holds the operation permit, published pack locations are immutable, and a full Location mismatch forces rereading. No global cache, independent lifetime, synchronization, or Store-identity sharing was added.
6. Publication epochs remain authoritative for absence proofs. `PreparedAdmission::publish` still performs late positive collision validation when its absence epoch is stale. Retained comparison data is not consulted by that path.
7. Dependency validation remains separate: `close_dependencies` still checks pending IDs and database existence. A retained comparison entry cannot establish the presence or ordering of a dependency.
8. Transaction and physical batch limits are unchanged. At maximum 512 incoming operands, the additional misses-only locator map is charged alongside the original known map in the existing comparison scratch calculation. This leaves capacity for an ordinary validation wave; oversized comparisons keep their existing singleton route.
9. Retained payload capacity and conservative node ownership are deducted from the existing uniqueness allowance. Clear-on-full eviction releases the map entries; oversized operands bypass retention. A successful hit keeps its existing operand instead of growing retained state. No unbounded workload-dependent allocation was added.
10. Error resolution, explicit abort, and drop retain their rollback behavior. Rollback removes private pending and previously committed batches while preserving the baseline pack boundary. Closed sessions reject further admission. Retained bytes cannot authorize publication after rollback.
11. Final admission still passes through normal dependency checking, preparation, and final publication. Empty final publication, final metadata failure, and final-root atomicity require the existing terminal checks below; they were not replaced by cache-specific tests.

No additional correctness blocker was found after identifying the per-entry charge issue. This conclusion is bounded to the reviewed source and does not substitute for the execution owner's terminal validation.

## Diagnostic interpretation

- `collision_checks` includes successful exact byte checks on retained hits plus checks performed by `admission::compare` for misses.
- `conflict_read_rows` and `conflict_read_bytes` now include only misses actually processed by the authenticated-read comparison function. They remain canonical operand counts/bytes, not physical I/O bytes.
- `conflict_read_ns` times `admission::compare` on those misses. It excludes the exact comparison work done on retained hits and excludes the locator SQL lookup, as before for lookup. Comparing that timer across revisions is not a measurement of all collision handling CPU.
- Cross-batch occurrence accounting and first-preexisting-object reuse accounting still use the full known-locator map, independent of hits versus misses.
- Native decode counters should fall on retained hits; that is a useful causal check. End-to-end adjacent timings, CPU, and RSS remain necessary to establish the performance benefit and its memory implications.

## New focused checks

Module: `crates/layerfs-layerstack-store/src/objects/comparison_reuse_tests.rs`.

| Test | Property |
| --- | --- |
| `comparison_reuse_reads_once_and_checks_every_occurrence_and_location` | First known occurrence decodes storage; retained hit performs no native decode; a deliberately stale Location forces reread; a same-length collision is rejected; preexisting object survives rollback. |
| `comparison_reuse_authenticates_storage_before_retaining_an_operand` | Test-only corrupted locator pointing to another same-length valid frame fails storage authentication before any operand is retained. |
| `comparison_reuse_bounds_resident_operands_and_eviction_keeps_exact_checks` | Many unique 32 KiB chunks trigger bounded eviction, preserve conservative capacity accounting, and force authenticated reread of an evicted operand. |
| `comparison_reuse_single_operand_boundary_reserves_a_complete_tree_node` | Private accounting seam covers exact-limit allocation, one-byte overflow bypass, logical-length versus Vec-capacity distinction, and clear-on-full replacement. It is an allocation test, not an authentication test. |
| `comparison_reuse_collision_rolls_back_pending_and_committed_private_batches` | A retained-hit collision removes both uncommitted and committed private objects, preserves baseline content, restores autocommit, and closes admission. |

The execution owner reported that the initial four checks and the old fresh-filter collision check passed before the allocation-bound follow-up. This review record does not claim a terminal pass or a pass for the fifth check; authoritative execution logs belong to the execution owner.

## Existing terminal correctness mapping

These names were read from the current source. Run them through the normal serialized affected suite rather than repeatedly rerunning already passing filters without a source change or failure.

| Required property | Existing tests |
| --- | --- |
| Duplicate accounting before/after physical boundaries | `consumer_batches_flushed_duplicate_checks_without_recounting`; `segment_admission_deduplicates_across_pending_segments`; `segment_admission_deduplicates_after_a_batch_boundary` |
| Fresh-filter negative/false-positive and deliberate collisions | `fresh_filter_negatives_false_positives_and_collisions_keep_exact_admission`; `fresh_filter_nonempty_and_orphaned_stores_keep_sql_fallback` |
| Stale/publication epochs | `unchanged_absence_proof_avoids_reprobe_but_intervening_publication_rechecks`; `streaming_absence_proofs_advance_only_over_disjoint_owned_batches`; `native_admission_late_races_compare_canonical_full_and_prefix` |
| Bounded input/output and collision ownership | `direct_admission_honors_every_frozen_count_boundary`; `shared_admission_keeps_every_object_transaction_below_the_frozen_bounds`; `native_admission_peak_reservations_reject_unowned_buffers`; `native_admission_batches_bound_output_and_stream_late_collision_waves`; `collision_comparison_preserves_the_physical_reserve_and_unique_operands`; `native_reader_maximum_owned_capacity_bound` |
| Dependency correctness and authentication | `admission_watermark_preserves_preexisting_counts_and_dependency_authentication`; `native_admission_actual_prior_depth_first_hint_and_readback`; `native_reader_missing_same_pack_and_wrong_identity_reject`; `native_reader_depth_budget_and_legacy_hint_separation` |
| Pending/committed rollback and cleanup failure | `init_cohorts_bound_sql_commits_flush_empty_final_and_rollback_pending`; `init_cohorts_final_metadata_and_commit_failures_restore_baseline`; `failed_admission_reclaims_private_packs_and_preserves_waiting_owners_and_bases`; `failed_cleanup_quarantines_writes_and_keeps_preexisting_reads` |
| Empty final publication and final-root atomicity | `init_cohorts_bound_sql_commits_flush_empty_final_and_rollback_pending`; `init_cohorts_final_metadata_and_commit_failures_restore_baseline`; `empty_composite_admission_uses_no_object_point_selects`; `late_direct_initialization_failure_publishes_nothing`; `multi_batch_publication_failure_removes_admitted_objects` |
| Packed read scope, authentication, and legacy compatibility | `native_reader_point_scope_and_exact_ranges`; `native_reader_legacy_full_root_and_delta_rejection`; `native_reader_empty_chunk_same_pack_and_cross_role`; `native_batch_shares_directory_but_only_reads_requested_bodies` |
| Canonical import identity and fallback | `batched_directory_import_matches_legacy_canonical_root`; `parallel_root_import_and_cross_directory_hard_link_match_legacy`; `parallel_large_spill_matches_legacy_after_fresh_store_reopen`; `compact_inode_cross_task_hard_link_falls_back_before_admission`; `mixed_root_hard_link_falls_back_before_publication` |

The affected Store/Workspace/native suites, formatting, warning-denying Clippy, all affected performance/independent verification members, full benchmark, and DeepSeek full157 terminal storage/correctness qualification remain execution-owner responsibilities. Preserve exact source/binary/image identities and failed attempts with their applicability boundaries.

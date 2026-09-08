# Phase-1 repair ledger

**Resolved: all D01–D09 and A01–A09 have passing final proof on candidate593f4ad01.**
See [verification and exact custody](verification.md).

This is additive to the immutable #88 ledger. Historical P/D/accepted results and
unequal cleanup residues remain unchanged. Proof names below identify required
checks; the final verification report supplies executed results and source seals.

| ID | Guarantee and cause | Repair | Focused proof |
|---|---|---|---|
| D01 | Late Init failure returns an error and publishes nothing. Stale fixture estimated SQL ordinal from canonical object count; packed SQL has different cardinality. | Inject at actual pre-INSERT_LAYER boundary, retain exact error and zero-publication checks. | `late_direct_initialization_failure_publishes_nothing` |
| D02 | Failed final publication must not leave private admitted residue. Product independently committed batches lacked lifetime cleanup ownership. | Existing writer gate spans admission through retention; bounded deletion of owner-new selected rows/packs on error/drop; preexisting/required bases preserved. Retained Workspace stage survives later branch failure. | `multi_batch_publication_failure_removes_admitted_objects`; `failed_admission_reclaims_private_packs_and_preserves_waiting_owners_and_bases`; staging integration tests |
| D03 | Count-boundary inputs must fit real2MiB physical/6MiB data owners. Product accumulated hint/vector allocations for8191 records beside codec workspace. | Form512-object batches inside unchanged public ceilings; retain all original8190/8191 cases and add511/512/513. | `direct_admission_honors_every_frozen_count_boundary` |
| D04 | Aggregate partitioned spill owner must fit1MiB and keep failed candidates private. Stale fixture omitted local/shared ID reservations. | Assert exact sum of payload buffers plus four local and one shared64KiB ID buffers. | `partitioned_completed_files_share_direct_facts_and_keep_failures_private` |
| D05 | Shared admission obeys count and byte/resource boundaries. Same product batching cause asD03. | Shared512 cap, original small-payload case retained, incompressible1000-byte pressure case added. | `shared_admission_keeps_every_object_transaction_below_the_frozen_bounds` |
| D06 | Selection/consumption preserve authenticated graph order and exact subset. Fixture expected physical order after requesting reverse graph order. | Expect the supplied child-first graph order; retain exact identities, membership and excluded middle object. | `spilled_candidate_visits_selected_objects_in_graph_order` |
| D07 | Workspace selection/dedup and failed admission privacy require an actual SQL failure. One-object token remained pending; no SQL ran during construction. | Prove pending row absent, consume token through real admission with armed statement hook, require error and unchanged Store/stage. Consumer already runs on caller thread; no new global hook needed. | `workspace_delivery_selects_before_admission_and_deduplicates_across_phases` |
| D08 | Exactly one executable SQL statement plus exact manifest/schema/header/parameters. Test counted a comment semicolon. | SQLite Batch parser counts executable statements; SQL behavior unchanged. | `exact_manifest_prepares_against_exact_v6_schema` and exact v7/schema compatibility checks |
| D09 | Requested-only authentication, unrelated corruption handling and cache reuse. Obsolete two-argument INSERT never reached assertions. | Admit through shared packed fixture, corrupt unrelated identity afterward; restore deleted real dependency before large structural admission. | `snapshot_cache_reads_only_requested_authenticated_objects_and_reuses_them` |
| A01 | Unsupported writers must reject native Stores before mutation. Same schema6 header admitted old writers. | Fresh native7 fence, dual6/7 validation, legacy6 writes stay legacy, research-native6 rejected without mutation. | Four schema compatibility tests, native legacy-base reopen test, actual older/newer binary probe |
| A02 | Directory regular payload provenance must reach shared native admission without metadata promotion. Both fast and serial sinks lost FILE/span context. | Scope checked-file producer context; transport hints through both sinks. | `file_provenance_is_scoped_and_restored_before_metadata_and_after_failure`; `directory_and_fallback_import_encode_only_regular_payloads_and_reopen` |
| A03 | Exported content Write/Splice must retain actual regular-file provenance while generic metadata ropes remain unmarked. `store::apply_changes` lacked context. | Scope inside regular-file build/replace before metadata callback; restore on errors. | `public_file_changes_scope_payload_provenance_and_restore_after_errors` |
| A04 | Exact integration schema, corruption authentication and every publication boundary must be tested. v4/v5 fixtures retained older schema5 layout, removed objects.bytes SQL, assumed v4 migration and guessed fault ordinals. | Exact7/4KiB/packed layout; valid RAW singleton then same-length corruption; nonmutating4 rejection with genuine legacy6 staging; stable commit/advance sentinels. | Full `tests/v4.rs` and `tests/v5.rs` |
| A05 | Shared Init uses batched selected-index probes and pack inserts. #71 SQL trace fixture matched obsolete objects(id,bytes) insertion. | Validate actual executable selected-locator/pack statements and reject object point reads; retain one-worker applicability. | `empty_composite_admission_uses_no_object_point_selects` |
| A06 | Late duplicate diagnostics must test whole partial-pack retention without deadlocking the new owner contract. Test held two independent owners on one thread. | Share explicit test ownership while retaining both prepared forms and exact race/selected/unlocated populations. | `late_race_retains_whole_partial_pack_and_invalidates_unique_cohort` |
| A07 | Cleanup failure must not silently allow subsequent unsafe writes. Drop cannot return a cleanup error. | Return combined errors on fallible paths and quarantine writes for that Store owner; existing authenticated reads remain usable. | `failed_cleanup_quarantines_writes_and_keeps_preexisting_reads` |
| A08 | Demand-read batching must hash unique rows once and move bytes at final use. Feature-enabled fixture used obsolete2-parameter SQL and expected one old row query for packed reads. | Admit valid packed objects, inject unrelated locator corruption afterward, assert one selected-locator query separately from necessary pack work, preserve exact hash/copy/error checks. | `durable_batches_hash_unique_rows_once_and_move_on_last_use` |

| A09 | Public Workspace retry test must reach both logical publication failures. It guessed SQL ordinals from inserted-object count and unexpectedly succeeded. | Use the actual commit/advance sentinels; strengthen exact cleanup versus retained-stage object counts and preserve one retry/one Commit/no-op checks. | `group_5_candidate_admission_and_publication_failures_retry_once` |

Additional quality repairs remove unused superseded helpers, mark retained test
oracles as test-only, and resolve warning-denying Clippy findings without lint
waivers. Benchmark source changes are formatting and two redundant borrows;
workloads, timers and sample definitions are unchanged. The Docker correctness
stage now runs all FUSE library tests, including #71 checkpoint ownership tests.

The ignored `parallel_large_spill_matches_legacy_after_fresh_store_reopen` is
applicable on this host and passed its explicit lane. It constructs100,004,100 actual
file bytes and checks complete canonical identity/content after reopen against
the legacy construction oracle. It is correctness coverage, not a performance
sample or a substitute for #91's full workloads.

## Retained development failures

- Initial focused run: seven ledger repairs passed; the new D09 structural
  fixture exposed its missing dependency after deliberate cache-test deletion.
  The dependency was restored through normal admission.
- Before D02 repair, focused reproduction still left19,571 selected objects.
- Compile receipts preserve the cleanup-query iterator lifetime fix, SQLite
  signed count correction, and a concurrent file-edit reconciliation error.
- First complete Store attempt found A05 and stopped on A06's blocked owner.
  The owned test process was sampled, then terminated; it is a partial failed
  suite, not a pass. Its source, output and stack sample remain preserved.
- The additional corruption fixture first expected Core IdentityMismatch;
  the public Store error mapping is precisely Integrity("object identity").
  The corrected assertion preserves that exact authentication failure.
- An initial route assertion expected two source passes for a preflight-rejected
  root symlink. That preflight did not construct/read regular files; the receipt
  correctly reports one construction. The strengthened test directly checks
  Some/None from the real fast initializer, drops the unpublished owner and
  checks cleanup, then separately runs the public operation and authenticates
  contents/metadata after reopen.

The older D07 D-arm actual result remains historically unavailable. No new run
fills it retroactively. The old184,598,528B figure remains an original-producer
measurement and is not a repaired-candidate result.

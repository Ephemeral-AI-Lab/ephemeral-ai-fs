# Independent P Store regression comparison

**The full Store suite remains FAILED: P 62 passed / 9 failed / 1 ignored; D 49 / 9 / 1; accepted control 43 / 9 / 1.** All nine failing names match across the three logs. Seven panic payloads match exactly after removing only source line/column numbers. No failure is waived, and this comparison is not a whole-suite pass.

This reviewer read the existing logs and the relevant current fixture source only. No rerun, build, Store opening or baseline source change was performed.

## Assertion-level reconciliation

| Failure | Observed comparison |
|---|---|
| `layerstack::tests::late_direct_initialization_failure_publishes_nothing` | All three print unexpected `Ok(InitializeLayerStackResult { ... })`; generated LayerStackId and LayerId differ. Same result variant, not identical IDs. |
| `layerstack::tests::multi_batch_publication_failure_removes_admitted_objects` | Expected object residue 0; actual P **19,571**, D **19,573**, accepted **19,572**. Same failed cleanup assertion, numerically different outcomes. |
| `objects::tests::direct_admission_honors_every_frozen_count_boundary` | Exact panic payload match apart from source line/column. |
| `objects::tests::partitioned_completed_files_share_direct_facts_and_keep_failures_private` | Exact panic payload match apart from source line/column. |
| `objects::tests::shared_admission_keeps_every_object_transaction_below_the_frozen_bounds` | Exact panic payload match apart from source line/column. |
| `objects::tests::spilled_candidate_visits_selected_objects_in_graph_order` | Exact panic payload match apart from source line/column. |
| `objects::tests::workspace_delivery_selects_before_admission_and_deduplicates_across_phases` | Exact panic payload match apart from source line/column. **Opaque actual result:** the matching `matches!` assertion prints neither the returned error nor an unexpected success value. |
| `statements::tests::exact_manifest_prepares_against_exact_v6_schema` | Exact panic payload match apart from source line/column. |
| `workspace::tests::snapshot_cache_reads_only_requested_authenticated_objects_and_reuses_them` | Exact panic payload match apart from source line/column. |

The seven matched payloads include both `physical encoding reservation` errors, the same reversed pair of selected ObjectIds, `720896 != 1048576`, two statements instead of one for `objects/insert.sql`, `InvalidParameterCount(2, 5)`, and the opaque injected-failure expectation. The differing cleanup count is retained exactly; these logs do not prove that its difference is solely randomness or that all nine underlying executions are equivalent. The initialization fixture creates filesystem-backed sources and returns generated namespace identities, making run-specific identities unsurprising, but that is not a license to normalize the cleanup residue.

## Finite disposition

The logs establish no new failing test name and no newly different printed error class among the payloads that expose their actual result. They cannot establish “no hidden different error” for the opaque workspace-delivery assertion: its fixture asserts `matches!(failed, Err(StoreError::Integrity("injected transaction failure")))` without printing `failed`. The precise actual result for that assertion is unavailable in all three receipts. Do not upgrade matching assertion text to matching error behavior.

These failures remain unresolved baseline fault-path/boundary/schema/order defects. The evidence supports an explicitly scoped research continuation decision only when the new native/reader/admission/diagnostic/S1 gates pass and the parent preserves the failed full-suite receipts and qualifications. It does not establish fault-path correctness, release readiness or a blanket waiver. No additional rerun is requested by this report.

## Exact log provenance

- P: [store-regression-1.log](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-P-custody-1/store-regression-1.log); SHA-256 `38f714ac4dfa8650fb0e3038c1429f7e0de08f8f0c8bf102c6dca3308f2c9efe`.
- D: [store-regression-1.log](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-diagnostic-custody-1/store-regression-1.log); SHA-256 `9617f8bd310e818e64137b8c1d902bc1af6f4bd2a82476e34c4a6aa96d57068d`.
- accepted: [accepted-store-regression-1.log](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-diagnostic-custody-1/accepted-store-regression-1.log); SHA-256 `9e40aae8f594b004a9eaa9a5ca4a8150b0c0af270c1cf82906a9dcb74343075b`.

## Addendum: bounded assertion-detail comparison

The parent reran only the previously opaque workspace-delivery test in P and an isolated accepted-control checkout after adding test-only assertion detail. The reviewed source retains the expected injected-failure predicate and adds `failed.as_ref().err()` to its failure message; the assertion is not weakened. Both existing receipts report `actual error (None means unexpected success): None`. Thus both targeted executions returned unexpected success, rather than a newly hidden resource error in P.

Each receipt contains one failed test and no passed tests (P: 71 filtered out; accepted: 52 filtered out). This additional evidence resolves the unknown actual result for the P-versus-accepted comparison only. The original D actual result remains unavailable: D was not rerun and its older opaque message is not retroactively filled in. The full-suite 62/9/1 result remains failed, and the cleanup residual differences and all other qualifications above remain unchanged. No assertion repair or waiver is established. This reviewer read the receipts and current assertion source only and performed no execution.

- [P-delivery-failure-detail-1.log](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-P-custody-1/P-delivery-failure-detail-1.log); SHA-256 `fd337b882a39d5daa2f87d54b5b322b4fa7dc6aee8329d77c4877edef95050dd`.
- [accepted-delivery-failure-detail-1.log](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-P-custody-1/accepted-delivery-failure-detail-1.log); SHA-256 `111bc612f71a20ceda6c77a40e593ce723ab4001b325d596f9bc0adcb84c768a`.

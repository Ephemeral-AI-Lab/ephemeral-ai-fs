# v0.1.5 benchmark family scope

Owner-directed scope, 2026-09-10. Split into two benchmark issues:

- [#101: historical_access](https://github.com/Ephemeral-AI-Lab/layerfs/issues/101) owns the access family and its15-second test/verification requirements.
- [#102: full benchmark run and optimization](https://github.com/Ephemeral-AI-Lab/layerfs/issues/102) owns optional repository-history profiles, the full campaign and optimization of measured regressions.

This is the agreed family plan, not an implemented registry or a completed benchmark campaign.

## Exactly two new families

Add only `historical_access` and `repository_history`. Existing deduplication and other families remain in the ordinary regression suite; extending their case matrices is outside this follow-up. Reuse the existing runner, fixture preparation, public operations and verification machinery.

## repository_history — optional, explicitly selected

Owned by #102; it is not part of the #101 implementation scope.

| Profile | Original checkpoint selection | Selected states |
| --- | --- | ---: |
| Stride 1 | 1,2,…,157 | 157 |
| Stride 3 | 1,4,7,…,157 | 53 |
| Stride 10 | 1,11,21,…,151,157 | 17 |

Preserve the first and final original state in each profile: `sorted(set(range(1,158,stride)) | {157})`. Stride10 therefore has17states, not10; it is different from the earlier ten-state spread fixture. Prepare direct transitions between selected states, with no hidden commits for skipped checkpoints. Preserve original source/manifest/oracle seals and each `full157_index`.

All three repository-history profiles are optional because they can take substantial time. No routine test, quick verification, default family invocation or default full-regression command should silently launch them. Require explicit profile selection or an explicit optional-history campaign flag. Report unselected profiles as `NOT_RUN_OPTIONAL`, never PASS. A routine regression report may be complete for its declared mandatory scope without claiming repository-history qualification.

Reuse the existing DeepSeek history machinery. Measure public save/Commit and actual allocated retained storage, per-state growth, content/metadata/index/base costs, historical verification and cleanup with distinct scopes. Each stride needs its own matching Git/control history. Existing Git53/Git157 evidence retains its scope; no Git17 baseline has been measured. Preserve all original oracles for whichever history is selected.

The15-second historical-access limit does not apply to repository-history construction or exhaustive history verification. Those long operations remain explicit optional work with separately frozen budgets.

## historical_access — 15-second end-to-end tests

Owned by #101; hand the completed family and its known regressions to #102.

**Every selected historical-access test must finish within15seconds total, including preparation.** This is a hard owner requirement, not merely an inner-operation latency target.

The test clock starts before the first test-specific acquisition/preparation step. Include fixture acquisition/validation, any copy or staging, reader/Store/container/mount readiness when required by the declared surface, cache-state establishment and warm-up, the operation, receipt collection and teardown. Do not assign15seconds to preparation plus another15seconds to execution. Do not start timing after preparation, leave cleanup in an unbounded background task or report an operation-only timer as the complete test duration.

Use one outer deadline and allocate preparation/operation/cleanup within it, reserving time for cancellation and cleanup. Failure or unavailable prerequisites must also be reported promptly; waiting for another benchmark's lock is not an unmeasured extension. An overrun is a failure and retained evidence, not a reason to increase the budget after observing the result.

Use reusable sealed input Stores with compatibility/source identities. Existing repository-history artifacts may be explicit inputs produced by separately selected history runs, but a historical-access test must never auto-run a long history to prepare itself. All per-test validation/acquisition/copy/open work still counts inside15seconds. If required input is absent or incompatible and cannot be prepared inside the budget, fail fast with a clear not-ready result; do not hide preparation elsewhere or call that a passing test. First-use toolchain/product builds are explicit prerequisites, not secretly launched by the selected test.

Initial access coverage should use a small fixed set of bounded operations:
- Metadata lookup/stat and bounded directory listing.
- Small-file reads.
- Small range reads from large files, including the measured native-adapter worst cases.
- Bounded full-file reads.

Distinguish current-head and selected historical states, and application-cold versus warm access. Cache-state setup and warm-up count toward the test total. Do not call an uncontrolled OS cache a cold-disk test. Avoid a full Cartesian product of operation, size, age, history length and cache state.

Whole-snapshot traversal/checkout and exhaustive history replay do not belong in a routine historical-access case if they cannot meet15seconds. Keep exhaustive history checks in optional `repository_history`; selected access checks must validate exactly the paths/ranges and metadata named by their case.

Report outer wall plus preparation, operation and cleanup breakdown; returned/fetched/decoded bytes; groups, packs and dependency edges; and scoped CPU/memory counters. Freeze exact case IDs, cardinalities, operation surfaces, fixture selections and cancellation boundaries before implementation. No product scenario-specific behavior.

## Verification target

Prefer a separate verification invocation to finish within15seconds end to end as well, including its preparation and cleanup. **15seconds is the requested verification target**, while the test limit above is mandatory. Do not silently equate the two or inherit the old45/59-second verifier settings as the new desired duration.

Verify the selected case's exact content, metadata and declared access semantics using its original sealed oracle. Keep performance and verification timers separate. If a necessary verification cannot fit the target, report its actual time and explicit target miss and identify the exceptional coverage; do not truncate verification, silently relax the target or label a sampled check exhaustive. Any hard watchdog/exception for verification must be specified prospectively in the case contract, not invented after a miss.

## Regression campaign and implementation boundary

Under #102, after the selected product candidate and #101 access family are ready, rerun the existing mandatory registry plus mandatory historical-access cases. Diagnose measured regressions, optimize their shared causes through selected cases, then freeze the final candidate for a complete final campaign. Repository-history profiles run only when explicitly selected and remain separately reported. Do not broaden existing dedup families in this scope.

The54.38MB/65.96MB offline archive/reference layouts remain evidence for design and fixture selection, not proof of a shipped public implementation. Freeze which supported product/surface is tested and applicable controls; preserve host-owned SQLite/coordinator and the registered SDK/FUSE operation boundaries. Read amplification and the15-second end-to-end limits must be measured on that actual implementation.

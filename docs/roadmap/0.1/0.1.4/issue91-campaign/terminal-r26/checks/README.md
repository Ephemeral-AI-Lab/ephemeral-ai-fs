# Terminal R26 checks and source applicability

The recorded final native suite passed **405 tests across 37 binaries, with zero failures and one ignored test** on source `56470d418a5cad5d27616e995f88f845573854da`. Clippy and formatting checks passed on that source. Final source `861e388339ef5572659cb16ef0c8febbff0351df` changes only the observation helper and campaign documentation: Rust product, tests, manifests, and dependency sources are unchanged. These are copied execution receipts, not newly executed checks. Overlapping focused checks and suites must not be added together.

Each receipt directory contains the exact original command, result, source identity, source patch, and output. [provenance.json](provenance.json) binds each copied file to its original absolute path, byte count, and SHA-256. [SHA256SUMS](SHA256SUMS) additionally covers the packaged files, including this explanatory document; it excludes itself. The original absolute references inside qualification JSON remain historical custody references; corresponding receipts are copied here under the same directory names.

| Evidence | Recorded outcome and scope |
| --- | --- |
| [Final native suite](r26-native-suite-01/output.log), [summary](r26-native-suite-01/summary.json) | `env RUSTUP_TOOLCHAIN=1.85.1 tools/test-fast.sh`; 405 passed, zero failed, one ignored; exit 0. |
| [Final Clippy](r26-clippy-01/output.log) | `cargo +1.96.0 clippy --workspace --locked -- -D warnings`; exit 0. |
| [Final formatting check](r26-format-check-01/result.json) | `cargo +1.96.0 fmt --all --check`; exit 0, empty output. |
| [Focused filter](r25-focused-filter-03/output.log) | Two filter tests passed, subsequently included in final suite. |
| [Focused cohorts](r25-focused-cohorts-01/output.log) | Two cohort tests passed, subsequently included in final suite. |
| [R26 before](r26-before-01/output.log) and [after](r26-after-01/output.log) | Same targeted predecessor regression failed before repair and passed after repair. Both historical runs used dirty `d6a7b4` with their exact source patches preserved. Final clean suite contains the committed regression. |
| [Large spill](r25-large-spill-01/output.log) | Explicit ignored `parallel_large_spill_matches_legacy_after_fresh_store_reopen` passed. Reused Store-only receipt with unchanged source, not a claim it ran in the final suite. |
| [Doc tests](r25-doc-tests-01/output.log) | Prior Store telemetry compile-fail doctest passed; Workspace had zero doctests. Reused under documented unchanged-source applicability. |
| [Initial filter compile failure 1](r25-focused-filter-01/output.log) | Unsupported `u64` SQLite test reads and unused import; exit 101. Preserved as an unsuccessful test-source compilation attempt. |
| [Initial filter compile failure 2](r25-focused-filter-02/output.log) | Attempted move through authenticated-object dereference; exit 101. Corrected before the passing filter run. |
| [Prior suite](r25-native-suite-01/output.log), [prior Clippy](r25-clippy-01/output.log), [prior format check](r25-format-check-01/result.json) | Historical supporting receipts; final R26 suite/checks are authoritative where overlapping. |
| [Formatting action](r25-format-01/command.json) | Historical mutating `cargo fmt` action and its exact source patch. This is separate from the successful formatting checks. |

## Required correctness coverage

The following references are exact passing records in [the final suite log](r26-native-suite-01/output.log). [coverage-records.json](coverage-records.json) preserves their one-based log lines for machine inspection.

| Requirement | Final suite evidence |
| --- | --- |
| Filter definite negatives, forced false positives, duplicates/collisions | `fresh_filter_negatives_false_positives_and_collisions_keep_exact_admission`, line 612; nonempty/orphan fallback, line 613. |
| SQL cohort object/byte limits, pending rollback, empty final flush | `init_cohorts_bound_sql_commits_flush_empty_final_and_rollback_pending`, line 614. |
| Final root/metadata atomicity and actual COMMIT failure | `init_cohorts_final_metadata_and_commit_failures_restore_baseline`, line 615: partial final callback failure and commit-hook veto restore baseline after prior committed work; pending transaction cleanup and subsequent ownership are covered. |
| Physical resource ownership and transaction bounds | Collision reserve test, line 568; peak reservation rejection, line 575; shared admission frozen transaction bounds, line 626. |
| Rollback of private packs and late publication failures | Private-pack cleanup, line 569; direct initialization failure, line 552; multi-batch publication failure, line 556. |
| Collision rechecks and absence-proof epochs | Native FULL/PREFIX late races, line 574; streaming absence proof, line 576; intervening publication recheck, line 577. |
| Authentication and spill delivery | Checked framing/identity, line 602; authenticated fresh spill handoff, line 617; separate large-spill fresh reopen receipt above. |
| Predecessor budget repair preserves correspondence and native PREFIX | `parallel_predecessor_plan_keeps_correspondence_and_native_prefixes`, line 324, together with targeted before/after failure-to-pass receipts. |

## Qualification and fixed constraints

[Original R26 qualification](r26-preparation/candidate-check-qualification.json) records all commands, execution-source custody, and Store-only reuse. [Final 861 applicability](r26-preparation/candidate-check-applicability-861e.json) binds the clean passing checks to the final helper/document-only correction. [Final helper guard receipt](r26-preparation/reuse-guard-preflight-final.json) preserves the separately checked observation binding. Original [test-only](r26-preparation/test-only.patch), [product repair](r26-preparation/product-fix.patch), and [patch custody](r26-preparation/patch-custody.json) records preserve the narrow R26 treatment. Build qualification is included for provenance, not counted as a test suite.

The packaging pass independently inspected Git objects and parsed lockfiles; [source-audit.json](source-audit.json) records the result. Final schema declares `NEW_STORE_PAGE_SIZE_BYTES: i64 = 4096`. Root Cargo manifests and lockfile are unchanged against campaign base `46308986`. The only changed Cargo lockfile is the repository-owned analysis-role harness lockfile: all 42 imported package records, including source/version/checksum/dependency fields, are identical to that base. No imported library source, version, or feature change is introduced by this final treatment. Store/content/root manifests are unchanged from the source of the reused large-spill and doctest checks.

This folder establishes check execution and source applicability. It does not independently establish benchmark, storage-gate, or full-157 acceptance; those have separate terminal receipts. No tests, builds, measurements, or Store opens were performed while assembling this package.

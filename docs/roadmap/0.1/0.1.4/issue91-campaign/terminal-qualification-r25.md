# R25 terminal qualification

The user authorized freezing the retained candidate, focused correctness checks,
affected Store/Workspace checks, final Init performance/resource comparison,
full157 and SQLite analysis, and finishing PR94 only when applicable gates pass.
R19's deferred-verification restriction is now lifted for terminal qualification.

Retained treatments: R15 new-file Commit streaming, fresh-session negative lookup
filter, and R23 Init-only bounded SQL commit coalescing. R16 lookahead, R17 direct
assembly, and R24 larger locator statements are not present. Keep existing native
encoding/dependency semantics, 4KiB pages, physical512-object/512KiB batches,
8191-object/4MiB-minus-one-byte Init SQL ceilings and unchanged imported libraries.

Sequence: add focused regression coverage, format/freeze source, run focused tests
and affected native checks plus repository-required formatting/Clippy; repair any
failure and freeze revised source before further qualification. Preserve all
failed attempts. Build final host/image/census identities from the frozen source.
Compare final Init performance in adjacent pairs against declared archived
controls, separating incremental treatment comparisons from published-baseline
regression assessment. Report RSS, CPU, transaction maxima and storage honestly.

Full157 reuses the existing sealed schedule and native public workflow, with
control then candidate, preverification snapshots/census, independent158-receipt
accounting and157 historical proofs per arm. It must authenticate equal retained
state and at least10% lower original acknowledged allocation, not copied-file
allocation. Run SQLite analyzer only on owned quiescent snapshots; retain hashes
and report page/index/pack/overflow/slack without double-counting.

All heavy builds/tests/measurements serialize through the existing nonblocking
host measurement lock. Host Store/SDK/SQLite stay on macOS; Docker serves only
its existing runtime role. No imported library patch/version/feature changes.

The existing campaign's hard/strict/severe gates and required selected-matrix
coverage still apply. Passing these terminal checks alone cannot waive an
unresolved campaign regression or mark v0.1.4 qualified. Update PR evidence and
readiness only to the extent supported by completed gates; no automatic release
or tag is part of this task.

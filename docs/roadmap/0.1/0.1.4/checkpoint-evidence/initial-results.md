# Initial campaign results — qualification incomplete

The complete initial G0 campaign executed all 198 declared performance cases. Independent verification recorded 222 passes, three timeouts and one failure across 226 required attempts. The explicitly optional 600-second endurance member was not run. Execution success is not performance acceptance.

The published v0.1.3 baseline is checkpoint `9f5a641d223606c45e5e6aa8a20094c12f9139a1`, preserved in release tag `49f0f5be911e7f5b92afc2becab45144ad7a1307`. The tag includes a later FUSE correctness repair after which the full performance campaign was not rerun. Comparisons use historical `elapsed_ns`, not its older-comparison columns. These are fixed-seed historical observations, not a fresh paired experiment or statistical speedup claim.

G0 candidate: `f484a79861a11c5100ad6c1ee41df59af7648601`; source seal `dcfddc7a6b25051e78ac61102a0996aa5e330821f61b6ea0d3ac89fe9dcc869d`. Exact build/image identities and raw outcomes are in `raw/generations/g0/`. The frozen specification and complete mapping are in `../issue91-campaign/`.

The frozen elapsed-time severity rule identified 44 cases across 11 families; 25 were at least 2× slower. See `g0-severe-total-time.csv` for every case, exact operands, timers, proof status and receipts. The separate strict tiny-bulk-create-100 target also failed: 0.989888 s historically versus 1.220231 s G0, exceeding the unchanged <1 s gate.

## Repairs and diagnostic status

- Native encoder scratch reuse was implemented in `5eb1414f0`. Focused native admission, bounds, import/reopen and rollback tests passed. Four diagnostic performance cases and their independent verifiers passed, but severe Init regressions remain. This is a partial improvement, not final qualification.
- 100,000-file Init: v0.1.3 2.603162083 s; G0 13.565385375 s; G1 diagnostic 12.155402250 s. G1 remains 4.67× slower (+366.95%). Comparing G1 only with G0 conceals the regression.
- 500 MiB creation: v0.1.3 3.068249542 s; G0 4.051680834 s; G1 diagnostic 3.825552959 s (+24.68% historically). The G0 increase concentrates in Commit: 1.941935 s to 2.930548 s; Exec is approximately unchanged. Host CPU increased 52.96%, while acknowledged allocation fell from 637,599,744 to 536,875,008 bytes. Storage savings do not establish acceptable foreground cost.
- Init diagnostics show serial admission work and producer backpressure, with transactions increasing from 130 to 2,519 for 100,000 files. A bounded collision-validation batching repair is in progress; it has no passing repaired benchmark result yet.
- The 500-tier directory content scan increased from 3.906595 s to 5.275265667 s. Authentication and native extraction are visible in sampling. A bounded shared extraction repair is in progress; authentication must remain intact. No repaired read result is claimed.
- Three independent proof timeouts occurred after expensive preparation consumed most of the unchanged deadline (namespace100000 and the two 100,000-object Store cases). Explicit preparation scheduling was added without raising proof bounds. These three proofs still require complete revalidation.
- The corrupted-object proof used an obsolete inline-SQL payload fixture. It now positively validates an isolated supported payload before corrupting it; focused corrupt/missing payload FUSE proofs pass. This is a harness repair, not evidence that every hard failure is resolved.

G1 is source `5eb1414f0` with the exact identity recorded in its diagnostic declaration. Its build reported a dirty working tree because of a benchmark-results cache symlink; tracked product code had an empty patch. Preserve that custody qualification. G1 is a limited diagnostic and is not substituted into the complete G0 matrix.

## Outstanding qualification

Recommendation: revise; keep #91 open. Finish feasible product repairs, seal one final candidate, requalify all affected performance and verifier members, complete equal-retained-state full157 storage evaluation, phase-1 compatibility/resource/correctness checks and independent review, then publish the final reconciled tables. The old #88 storage experiment is not new-candidate evidence. Optional endurance, general cold-cache and tail behavior remain unqualified. This interim record does not create a release or waive any gate.

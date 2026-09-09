# R26 derived report binding: diagnosis and safe packaging

The original failed report at `../r26-full-benchmark-report/` remains intact.
The original ordinary campaign at `../r26-full-benchmark-final/` is unchanged.
No product, collector, report generator, thresholds or raw receipt contents have
been changed. This is a post-collection binding, not a newly invented prospective
declaration. The preparation task has not invoked the report or opened a Store.

## Cause

The v0.1.4 reporter's `--frozen issue91-campaign` argument was correct. It loads
the prospective campaign plan, baseline manifest, mapping and registry there.
Separately, its reused v0.1.3 `derive()` requires a generation-local
`CAMPAIGN/declaration.json` containing host/image/harness, order and seed bindings.
The collector writes `selections.json`, registry and ledgers, but never creates
that declaration. The earlier invocation plan omitted this packaging prerequisite.

That missing declaration produced 637 cascading errors: expected identities
became None, seed became None, and expected orders were absent. Parsed observed
and prospective registries are exactly equal (227 definitions). All 198 captured
performance ledger identities agree with the pre-existing R26 preflight; all use
the same harness identity and seed 1. The proof ledger records 226 PASS and one
NOT_RUN_OPTIONAL. These checks do not replace the unchanged reporter's full checks.

## Prepared derived files

- `derived-declaration.json`: explicitly marked
  `POST_COLLECTION_DERIVED_NOT_PROSPECTIVE`. Membership/seed/exclusion come from
  the prior campaign declaration, cross-checked against actual selections/registry.
  Host/image/source come from the pre-existing R26 identity preflight and commands,
  cross-checked against every performance ledger row. Harness identity is the
  unanimous recorded value, expressly identified as a post-collection binding.
- `derived-verification-ledger.json`: a separately derived copy of the original
  ledger. Only six path strings change: the preparation and proof receipt paths
  for the three independently prepared cases become generation-relative paths.
  Every receipt hash, outcome, identity, command, timing and resource value remains
  unchanged. The original ledger is not overwritten.
- `binding-provenance.json`: input paths/hashes, observed counts, field sources
  and every before/after path rebase. Use this beside any repaired report.

The path rebases are necessary because the existing reporter verifies both that
prepared evidence resolves within the generation and that ledger receipt paths
resolve to their registered locations. A symlink view of the original directories
would fail its containment checks. Do not weaken those checks.

## Packaging to perform when resource-sensitive work is quiescent

Create a new `view/` directory under this repair folder. Project only the required
JSON/JSONL evidence into it, using hard links or byte-identical copies, not symlinks
that escape the view. No Store, runtime, fixture or snapshot is needed:

1. Install a copy of `derived-declaration.json` as `view/declaration.json`.
2. Install a copy of `derived-verification-ledger.json` as
   `view/verification-ledger.json`, keeping the original ledger at its original path.
3. Link/copy the original `selections.json`, `registry.jsonl`,
   `performance-ledger.json` and `terminal-assessment.json` into `view/`.
4. For each of the 198 performance registry rows, link/copy exactly
   `performance/FAMILY/CASE/perf.jsonl` to the same relative location in `view/`.
5. For each verification-supported registry row, link/copy exactly
   `verification/FAMILY/CASE/verification.json` to its relative location in `view/`.
   Keep the optional exclusion metadata if present; no absent verifier is invented.
6. For each of the three preparation-ledger cases, link/copy its original
   `preparation/FAMILY/CASE/preparation.json` and `runner.json`. Commands/stderr may
   be included byte-identically for provenance, although the report needs only
   the two JSON files. All six rebased ledger paths then resolve inside `view/`.
7. Record the projection paths and source/destination hashes. A hard-linked raw
   file must never be edited through the view. The report only reads these files.

This is an evidence projection with derived binding metadata, not a rewritten
raw generation. Keep the derivation label visible in the publication narrative.

## Unchanged report invocation after packaging

```sh
cd /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91
python3 docs/roadmap/0.1/0.1.4/checkpoint-evidence/report.py \
  /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/r26-report-repair/view \
  --registry /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/r26-full-benchmark-final/registry.jsonl \
  --frozen docs/roadmap/0.1/0.1.4/issue91-campaign \
  --baseline docs/roadmap/0.1/0.1.3/checkpoint-evidence \
  --output /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/r26-report-repair/report
```

Inspect `errors` before interpreting status. Any remaining content, operation,
custody or correctness mismatch remains a real blocker to investigate. Do not
filter errors or rewrite identities to force success. If errors are empty but
severe performance regressions remain, the existing reporter returns REVISE/exit1;
preserve that historical classification and attach the separate user selection
amendment accepting the candidate's performance tradeoffs.

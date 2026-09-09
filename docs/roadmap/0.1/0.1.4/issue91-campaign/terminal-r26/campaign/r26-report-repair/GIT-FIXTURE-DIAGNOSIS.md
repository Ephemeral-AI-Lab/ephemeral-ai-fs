# Four Git comparison errors: confirmed image-salted fixture metadata

The only differing field inside the historical/current prepared fixture records
is `input_plan_sha256`. Fixture bytes, regular-file counts, profile and input mode
match for all four `git_tool_workflow` cases.

`workspace_bench.rs::fixture_info` computes this Git-specific digest from:

1. The entry-recipe digest.
2. The literal `ordinary_workloads.rs` source.
3. The `LAYERFS_V013_IMAGE` environment value supplied by the runner.

The historical and candidate image IDs necessarily differ. This digest therefore
binds image provenance in addition to fixture recipe, despite its placement under
`preparation.fixture`. The unchanged report currently compares it as content.

The same sealed R26 binary was invoked only through `workspace-fixture-info` for
the four cases, seed 1, under each recorded image ID. All eight resulting fixture
records exactly match their corresponding recorded historical/candidate records.
This path evaluates metadata only: no fixture was prepared, no Store was opened,
and no Git workload, performance run or verifier was executed.

The four relevant generator files are byte-identical between the published
baseline commit and candidate `861e388339ef5572659cb16ef0c8febbff0351df`:

- `benchmark/fs-bench-pro/src/workspace_bench.rs`
- `benchmark/fs-bench-pro/workload/ordinary_workloads.rs`
- `benchmark/fs-bench-pro/workload/workspace_common.rs`
- `benchmark/fs-bench-pro/families/git_tool_workflow/mod.rs`

Exact commands, environment values, source hashes, eight metadata outputs and
matching results are in `git-fixture-diagnosis.json`.

## What this establishes

The fingerprint differences are reproduced entirely by changing the explicit
image salt. They are not evidence of a demonstrated differing working-tree
payload or volatile Git-index timestamp. The generator recipe/source is identical.

This does not establish byte equality of the previously materialized `.git`
directories across images. Real fixture preparation runs Git inside each image;
the metadata-only function does not read those generated repository bytes.
No unavailable historical `.git` bytes or cross-image Git equivalence is inferred.

## Reporting and PR scope

Keep the four comparisons INELIGIBLE and the unchanged reporter's INCOMPLETE
status. Preserve the original failed report as well as the repaired derived-view
report. Do not erase these errors or present 198 eligible historical comparisons.
The derived report separately establishes 198 performance executions PASS and
226 routine proofs PASS, plus one declared optional exclusion; its remaining four
errors concern cross-release comparability, not those execution/proof outcomes.

This limitation does not itself demonstrate a correctness or storage failure.
A review-ready PR can clearly present the completed runs, the user-accepted
performance tradeoffs, and four unavailable historical Git comparisons. It must
not claim that the original all-comparisons qualification gate passed. If readiness
is defined as satisfying that original gate without limitations, this remains an
unresolved limitation rather than an automatic waiver supplied by this diagnosis.

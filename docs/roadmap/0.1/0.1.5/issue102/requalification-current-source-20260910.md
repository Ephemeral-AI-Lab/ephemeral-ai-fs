# Issue102 requalification on the current source (cc8025fcd + uncommitted compaction removal)

**Campaign executed 2026-09-10T21:00Z–21:41Z. Overall outcome: FAIL on two families.**
17 mandatory families and `historical_access` were attempted as the full #104
matrix: **209/209 performance selections and 237/237 routine-verification
outcomes recorded**. Performance: 208 PASS/TARGET_MISS and 1 hard product error
(INCOMPLETE). Verification: 236 PASS and 1 FAIL (the same product-error case).
One open timing miss remains. This is a requalification on the current source
(`cc8025fcd`, #109) **including** the preserved uncommitted compaction-removal
work; the declared treatment is `promoted-uncompacted`.

**Unpaired fixed-seed single-sample comparisons against recorded v0.1.3 and
v0.1.4 (and v0.1.5 #104) results across different source states; no paired or
statistical speedup/regression claim is made. repository_history and the #102
additional cases remain pending.**

## Scope run

The exact #104 mandatory family set, one collector invocation per family in the
declared smoke-first order, with the frozen
`issue104/mandatory-campaign.json` declaration (seed 1, n=1, product 300 s /
outer 310 s / setup 600 s, verification 45 s work + 59 s hard, long-test
exclusion `workspace-sustained-600s-compact-v2-proof`). No control arm was run
(the released v0.1.3/v0.1.4 controls are incompatible with the current complete
harness); the collector's required retained-inapplicability input was passed.
`repository_history`, the six #102 additional cases, `small_file_delta_smoke`,
and every paired control arm were out of scope.

## Family completion

| Family | Performance completed/expected | Target pass | Verification pass/expected | Failures | Slow | All-attempt wall s | Outcome |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| namespace_mutation | 4/4 | 4 | 4/4 | 0 | 0 | 32.096 | PASS |
| edit_canonical_chunk_count | 12/12 | 12 | 12/12 | 0 | 0 | 76.278 | PASS |
| payload_create_read | 8/8 | 8 | 8/8 | 0 | 0 | 70.146 | PASS |
| dedup_cross_file | 10/10 | 10 | 10/10 | 0 | 0 | 83.829 | PASS |
| edit_length_preserving | 12/12 | 12 | 12/12 | 0 | 0 | 63.747 | PASS |
| edit_length_changing | 32/32 | 32 | 32/32 | 0 | 0 | 162.531 | PASS |
| dedup_workspace_reuse | 14/14 | 14 | 14/14 | 0 | 0 | 133.644 | PASS |
| init_namespace | 4/4 | 4 | 4/4 | 0 | 0 | 56.293 | PASS |
| store_footprint | 6/6 attempted | 5 | 5/6 | 1 product error + 1 verification FAIL | 0 | 99.477 | **FAIL** |
| tiny_file_churn | 20/20 | 20 | 20/20 | 0 | 0 | 177.833 | PASS |
| directory_construction_traversal | 12/12 | 12 | 12/12 | 0 | 0 | 101.651 | PASS |
| workspace_change_locality | 16/16 | 16 | 16/16 | 0 | 0 | 118.739 | PASS |
| git_tool_workflow | 4/4 | 4 | 4/4 | 0 | 0 | 93.820 | PASS |
| mixed_load_bearing | 4/4 | 4 | 4/4 | 0 | 0 | 58.868 | PASS |
| dedup_cdc_locality | 20/20 | 20 | 21/21 | 0 | 0 | 196.920 | PASS |
| dedup_branch_history | 20/20 | 19 | 20/20 | 1 TARGET_MISS | 1 | 235.384 | **FAIL** |
| workspace_reliability | 0/0 | — | 27/27 | 0 | 0 | 77.623 | PASS |
| historical_access | 11/11 | 11 (15 s contract) | 11/11 | 0 | 0 | 64.331 | PASS |

All executed resource/cleanup/custody gates passed. All routine proofs passed
except `store-footprint-metadata-cardinality-100000`. Total executed family wall
≈ 36 min (excluding preflight/builds and reporting). Ambient host load during
the campaign was 4–6 (load average; interactive shared macOS host) — small
timing deltas are noise per the unpaired-comparison rules and were not rerun.

## Hard failure finding: store_footprint (current-source product error)

`store_footprint/store-footprint-metadata-cardinality-100000`
(100,000 metadata-cardinality values; `product_call_sum_ns`) **FAILED** with a
product error, and its verification also failed:

```
fs-benchmark-pro failed: exit 1
fs-benchmark-pro: Store(Io(Custom { kind: Other, error: "physical encoding reservation" }))
```

- Origin: `crates/layerfs-layerstack-store/src/objects/admission.rs:823-826`,
  the hard `2 MiB` conservative reservation check in the metadata value-group
  encoding path.
- This case **PASSED in #104** (`elapsed_ns` 7,190,754,707; allocated
  554,381,312 B) on the preceding source state. The failure was not seen there
  because #109 only qualified `init_namespace/namespace-100000` and explicitly
  left every other family/tier unmeasured (`issue109/optimization-results.md`).
- Root cause candidate (not a fix): #109 Zone A added
  `small_signature: Option<[u64; 8]>` to `PreparedObject`
  (`admission.rs`), enlarging each prepared slot. The `fixed_associations`
  operand of the reservation includes
  `self.objects.capacity() * std::mem::size_of::<PreparedObject>()`
  (`admission.rs:798-804`), so the same workload now trips the hard budget.
  #109's own note for Zone A states it "now explicitly charges the
  prepared-vector capacity of the enlarged slots" — the metadata reservation
  inherits the same enlargement.
- There was **no retry**; the error is a deterministic arithmetic budget
  rejection, not infrastructure. Evidence:
  `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-family-campaign-20260910T210014Z/store_footprint/`
  (`candidate/attempt-1/performance/store_footprint/store-footprint-metadata-cardinality-100000/failure.log`
  and `perf.jsonl`; the retained #109 admission diff is
  `store_footprint-109-admission-diff.txt`).

This is a functional regression on a metadata-heavy corner of the shared
admission path and is the primary item for #108/#109 follow-up.

## Timing comparison (unpaired, single sample)

Per-case, exact `family` + `case` + identical `timer` only. `ratio` is
`current / recorded`. No difference column was used as an operand.

| Comparison | Matched cases | Median ratio | Min | Max | ≥15% slower | ≥15% faster |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| current vs #104 (v0.1.5, immediately preceding source) | 197 | 1.013 | 0.655 | 1.468 | 30 (see below) | 11 |
| current vs v0.1.3 `elapsed_ns` | 198 | 1.379 | 0.579 | 4.256 | 154 | 8 |
| current vs v0.1.4 G0 `g0_elapsed_ns` | 198 | — | — | — | — | — |

The v0.1.3/v0.1.4 baselines are older source states and harness generations;
their absolute ratios are reported as historical context only (v0.1.3 sets a
default 15 s reporting target, with 3 of its own target misses: one
`dedup_branch_history`, two `git_tool_workflow`). v0.1.4 G0 qualification was
incomplete (3 timeouts + 1 failure); its timeouts are not product failures and
its values are cited as recorded, not as a gate.

### Notable improvements vs #104 (≥15% faster, `elapsed_ns`, unpaired)

| Case | Current | #104 | Ratio |
| --- | ---: | ---: | ---: |
| init_namespace/namespace-10000 (`layerstack_init_ns`) | 0.905 s | 1.253 s | 0.722 |
| store_footprint/store-footprint-unique-100000 | 3.787 s | 5.042 s | 0.751 |
| init_namespace/namespace-100000 (`layerstack_init_ns`) | 3.759 s | 4.841 s | 0.776 |
| workspace_change_locality/workspace-distributed-sdk-edit-500-mixed-v4 | 3.239 s | 4.131 s | 0.784 |
| dedup_branch_history/dedup-history-distributed-500 | 4.191 s | 5.249 s | 0.798 |
| init_namespace/namespace-1000-compact-v3 | 0.0752 s | 0.0941 s | 0.798 |
| edit_length_changing/truncate-tail-4k-on-1mib-ops-1 | 8.674 ms | 10.816 ms | 0.802 |
| tiny_file_churn/tiny-bulk-create-10-compact-v2 | 0.310 s | 0.373 s | 0.832 |
| dedup_cdc_locality/dedup-cdc-insert-10 | 19.547 ms | 23.350 ms | 0.837 |
| dedup_cross_file/dedup-cross-file-unique-100 | 0.304 s | 0.359 s | 0.846 |
| edit_length_preserving/overwrite-middle-4k-on-500mib-ops-1 | 10.735 ms | 16.382 ms | 0.655 |

The Init wins are consistent with, and independently reproduce, the #109
declared `namespace-100000` reduction. The current host binary is exactly
#109's retained candidate (`sha256 dbbf1259…`).

### Review exceptions vs #104 (relative >15% AND absolute >1 ms) — 30 cases

Flagged for #108/#109 follow-up. 21 of 30 are small edit-commit cases whose
absolute deltas are ~1.6–4.1 ms; the largest absolute items are
`directory-content-scan-500-mixed-v4` (+973.8 ms) and the
dedup-history/dedup-cdc cases. This is consistent with a shared-path cost from
the #109 admission/signature changes (Zone A slot enlargement and signature
handling), not with a correctness change.

| Case | Current | #104 | Rel | Abs | Timer |
| --- | ---: | ---: | ---: | ---: | --- |
| edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1 | 12.753 ms | 8.684 ms | +46.9% | +4.069 ms | edit_commit_ns |
| edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1 | 12.354 ms | 8.988 ms | +37.5% | +3.366 ms | edit_commit_ns |
| dedup_branch_history/dedup-history-recurring-1 | 33.976 ms | 26.512 ms | +28.2% | +7.464 ms | pure_call_sum_ns |
| edit_length_changing/insert-middle-4k-on-1mib-ops-1 | 9.359 ms | 7.333 ms | +27.6% | +2.026 ms | edit_commit_ns |
| edit_length_changing/replace-shrink-middle-4k-to-2k-on-1mib-ops-1 | 10.148 ms | 7.966 ms | +27.4% | +2.182 ms | edit_commit_ns |
| edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1 | 16.247 ms | 12.815 ms | +26.8% | +3.431 ms | edit_commit_ns |
| edit_length_changing/prepend-head-4k-on-1mib-ops-1 | 9.751 ms | 7.696 ms | +26.7% | +2.055 ms | edit_commit_ns |
| dedup_branch_history/dedup-history-hotset-10 | 162.803 ms | 129.265 ms | +25.9% | +33.538 ms | pure_call_sum_ns |
| dedup_cdc_locality/dedup-cdc-delete-1 | 8.275 ms | 6.591 ms | +25.5% | +1.683 ms | pure_call_sum_ns |
| edit_length_changing/prepend-head-4k-on-500mib-result-capped-v2-ops-1 | 10.891 ms | 8.736 ms | +24.7% | +2.155 ms | edit_commit_ns |
| edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1 | 13.372 ms | 10.755 ms | +24.3% | +2.616 ms | edit_commit_ns |
| dedup_cdc_locality/dedup-cdc-scattered-100 | 380.341 ms | 306.337 ms | +24.2% | +74.004 ms | pure_call_sum_ns |
| payload_create_read/payload-random-read-500 | 294.519 ms | 242.045 ms | +21.7% | +52.474 ms | pure_call_sum_ns |
| edit_length_changing/replace-grow-middle-2k-to-4k-on-10mib-ops-1 | 10.324 ms | 8.534 ms | +21.0% | +1.790 ms | edit_commit_ns |
| tiny_file_churn/tiny-stat-10-compact-v2 | 25.770 ms | 21.342 ms | +20.7% | +4.428 ms | pure_call_sum_ns |
| edit_length_changing/delete-middle-4k-on-100mib-ops-1 | 10.129 ms | 8.403 ms | +20.6% | +1.727 ms | edit_commit_ns |
| edit_canonical_chunk_count/overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1 | 12.726 ms | 10.559 ms | +20.5% | +2.167 ms | edit_commit_ns |
| edit_length_changing/insert-middle-4k-on-500mib-result-capped-v2-ops-1 | 11.911 ms | 9.902 ms | +20.3% | +2.009 ms | edit_commit_ns |
| directory_construction_traversal/directory-content-scan-500-mixed-v4 | 5971.005 ms | 4997.212 ms | +19.5% | +973.793 ms | pure_call_sum_ns |
| edit_length_changing/append-tail-4k-on-100mib-ops-1 | 10.118 ms | 8.501 ms | +19.0% | +1.617 ms | edit_commit_ns |
| payload_create_read/payload-random-read-10-compact-v2 | 23.193 ms | 19.500 ms | +18.9% | +3.693 ms | pure_call_sum_ns |
| workspace_change_locality/workspace-clean-commit-100-mixed-v4 | 15.036 ms | 12.661 ms | +18.8% | +2.375 ms | pure_call_sum_ns |
| edit_length_changing/delete-middle-4k-on-500mib-ops-1 | 11.408 ms | 9.635 ms | +18.4% | +1.773 ms | edit_commit_ns |
| dedup_cdc_locality/dedup-cdc-common-body-100 | 142.923 ms | 121.091 ms | +18.0% | +21.833 ms | pure_call_sum_ns |
| dedup_cdc_locality/dedup-cdc-scattered-1 | 12.227 ms | 10.366 ms | +18.0% | +1.861 ms | pure_call_sum_ns |
| dedup_cdc_locality/dedup-cdc-common-body-10 | 27.251 ms | 23.232 ms | +17.3% | +4.018 ms | pure_call_sum_ns |
| dedup_cdc_locality/dedup-cdc-scattered-10 | 43.170 ms | 36.817 ms | +17.3% | +6.353 ms | pure_call_sum_ns |
| dedup_branch_history/dedup-history-hotset-1 | 31.944 ms | 27.293 ms | +17.0% | +4.651 ms | pure_call_sum_ns |
| dedup_cdc_locality/dedup-cdc-delete-10 | 21.579 ms | 18.442 ms | +17.0% | +3.137 ms | pure_call_sum_ns |
| dedup_cdc_locality/dedup-cdc-common-body-500 | 628.782 ms | 544.468 ms | +15.5% | +84.314 ms | pure_call_sum_ns |

## Target misses and slow-case diagnostics

| Family/case | Classification | Timer | Current s | Threshold s | #104 s |
| --- | --- | --- | ---: | ---: | ---: |
| dedup_branch_history/dedup-history-unrelated-500-mixed-v2 | TARGET_MISS | pure_call_sum_ns | 23.041778 | 15 | 23.781066 |

The known open item persists but improved slightly versus #104 in this
unpaired single sample. No workload/timeout relaxation was made.

Command-wall cases >5 s (existing diagnostic, not an admission failure): 10
cases — `dedup-history-distributed-500` 5.573 s, `dedup-history-hotset-500`
6.133 s, `dedup-history-unrelated-10` 9.906 s,
`dedup-history-unrelated-100-mixed-v2` 5.005 s,
`dedup-history-unrelated-500-mixed-v2` 24.162 s,
`directory-content-scan-500-mixed-v4` 6.369 s, `git-tool-500-mixed-v4`
6.990 s, `agent-episodes-500` 8.775 s, `tiny-bulk-create-500-mixed-v3`
5.699 s, `workspace-dense-rewrite-500-mixed-v4` 10.444 s. (The #104 list had
13 cases; the `init_namespace/namespace-100000` and both `store_footprint`
100k slow cases are no longer >5 s, except the newly failing metadata case
which does not reach a timing.)

The issue47 strict gate (`tiny-bulk-create-100-mixed-v3` < 1 s): **PASS**
(915,595,083 ns).

## Allocated-store findings vs #104

Only two cases differ by >1% in allocated bytes; both have essentially
identical apparent bytes, indicating pack/SQLite page-layout drift rather than
data change:

| Case | Current allocated B | #104 allocated B | Delta | Current apparent B | #104 apparent B |
| --- | ---: | ---: | ---: | ---: | ---: |
| init_namespace/namespace-10000 | 320,675,840 | 305,426,432 | +4.99% | 304,926,720 | 304,881,664 |
| init_namespace/namespace-100000 | 516,861,952 | 530,272,256 | −2.53% | 515,371,008 | 515,485,696 |

All other cases are within <1% (e.g. `store-footprint-unique-100000`
+0.897%, `store-footprint-large-object-500m` +0.261%). The `namespace-100000`
apparent bytes and canonical object/byte counts were previously verified
identical under #109. No storage-semantics change is inferred.

## historical_access

All 11 performance cases and all 11 verification runs PASS within the 15 s
contract (current outer wall 2.60–3.34 s performance, 2.75–2.91 s
verification; cleanup PASS). The current fixture is the v2 fixture pinning the
closed157 schema9 store (SHA256
`f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7`), which
differs from #104's `…-uncompacted104-v1` case IDs, so no case-level ratio is
claimed; the family is reported by its own 15 s gate.

## Build, source and custody identities

| Item | Value |
| --- | --- |
| Source commit (HEAD) | `cc8025fcd029c75c8a55003e5b72560806739864` |
| Source tree | `8f6736e64039795fc898b6407cd65a7aa4cdaf84` |
| Source dirty | `true` (21 tracked modified/deleted + untracked `docs/roadmap/0.1/0.1.5/compaction-removal.md`) |
| Source seal | `17334f5e6900bdaf73e1c49ccb9bc0ad7ec3b710eef3ef5efef9b9841c6975ba` |
| Product seal | `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4` |
| Compilation seal | `07eba1aa674a13dcbf6ae3a03f5aca814fca6d69af5e26ca09a7d5ca13c8830e` |
| Dependency seal | `cce6389a9d87e365bf11750870bf27444e8f52d8a981d054661004cb5ced0e79` |
| Host binary | `target/release/fs-benchmark-pro` sha256 `dbbf1259186c60122286bb2a0503d6dcccfd0a41f49eb82799b5c3a4d6e6a36b` (identical to #109's retained candidate) |
| Host build wall | 36.116 s (fresh seal-isolated target; no dependency reuse) |
| Linux image | `layerfs-bench-infra:17334f5e6900bdaf`, inspect Id `sha256:0ccd9886961edbd8186c9aea1055faa86bf8d65937ac9c20e69a39a6efb458b4` |
| Image built | 2026-09-10T21:01Z (workload at 86a12224…, source-tree `8f6736e6…`) |
| Access fixture | `benchmark/fs-bench-pro/families/historical_access/fixture.json`; access store SHA256 `f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7` |
| Unit tests | `python3 -m unittest discover -s benchmark/fs-bench-pro/shared -p 'test_*.py'` → 57 tests OK |
| Disk free at start | ~317 GiB (≥50 GiB reserve) |

Exact commands (preflight and per-family template):

```sh
cd /Users/yifanxu/Ephemeral-AI-Lab/layerfs
python3 -m unittest discover -s benchmark/fs-bench-pro/shared -p 'test_*.py'
python3 -u benchmark/fs-bench-pro/shared/runner.py --build-host
python3 -u benchmark/fs-bench-pro/shared/runner.py --build-image
# per family (collector acquires the measurement lock itself):
python3 -u benchmark/fs-bench-pro/issue102_collect.py \
  --family <FAMILY> \
  --candidate-binary target/release/fs-benchmark-pro \
  --candidate-image layerfs-bench-infra:17334f5e6900bdaf \
  --control-inapplicable <EV>/control-inapplicable.json \
  --output <EV>/<FAMILY> --resume
# historical_access additionally:
#   --access-store /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/host-runtime/store.sqlite
#   --access-fixture benchmark/fs-bench-pro/families/historical_access/fixture.json
```

The `run-family-task1.sh` wrapper and every per-family `<family>.log`, the
per-family `ledger.jsonl`, raw receipts and `commands/` stdout/stderr/exit
records are retained.

## Evidence paths

- Campaign root (create-once):
  `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-family-campaign-20260910T210014Z/`
- Consolidated data: `current-cases.csv`, `comparison.csv`,
  `family-summary.csv`, `no-historical-comparator.json`, `analyze.py`.
- Build/custody: `task1-build-host.log`, `task1-build-image.log`,
  `task1-host-identity.json`, `task1-image-id.txt`, `task1-image-labels.json`.
- Inapplicability: `control-inapplicable.json`,
  `control-inapplicable-audit.txt`.
- store_footprint failure: `store_footprint/…`,
  `store_footprint-109-admission-diff.txt`.
- Orchestration record:
  `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-requalification-orchestration-20260910T205721Z/orchestration.md`

No write occurred under `layerfs-issue104-evidence/`,
`layerfs-issue103-evidence/`, `layerfs-issue109-evidence/`, or historical
`benchmark-results/host-store/campaigns/`. No product-code change, release,
tag, deployment, or unrelated cleanup was made. No file under `crates/`,
`tools/` or `benchmark/` was edited during the campaign.

## Deviations

1. The task prompt's per-family sample command omits `--control-inapplicable`,
   but `issue102_collect.py:85` hard-requires a qualified control or retained
   inapplicability evidence. A fresh inapplicability document was generated
   under the evidence root (with a concrete audited incompatibility against the
   released control `101fa273…`) rather than weakening or bypassing validation.
   No control arm was run. This mirrors the #104 invocation pattern.
2. The preflight image rebuild produced inspect Id
   `sha256:0ccd9886…` (new manifest-list digest) for the same
   `17334f5e6900bdaf` tag; the tag itself is used for all runs. This differs
   from the tag-store-cached Id observed at handoff and is recorded rather than
   treated as an identity change (labels/seals match the current tree).

## Follow-up required

- **#108/#109 (primary):** fix and requalify
  `store-footprint-metadata-cardinality-100000` on the current shared admission
  path; review the 30 timing review-exceptions (notably the small-edit and
  dedup-history/dedup-cdc clusters).
- **#104 timing target stays open:**
  `dedup_branch_history/dedup-history-unrelated-500-mixed-v2`.
- `repository_history` and the six #102 additional cases remain pending (owned
  by the companion recheck / #102 lane).

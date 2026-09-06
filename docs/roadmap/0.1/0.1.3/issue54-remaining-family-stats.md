# Issue 54 — remaining-family statistics collection

Status: **collection complete**. This is a one-sample statistics campaign, not a 15-second qualification, not a three-seed distribution, and not an Exec/Commit optimization pass. #39 stays open.

Raw receipts: `benchmark-results/host-store/campaigns/issue54-c8013528cc49547b/` (local, gitignored). Machine-readable per-case table: [issue54-remaining-family-stats.json](issue54-remaining-family-stats.json).

## Frozen identities

| Item | Value |
|---|---|
| Product seal | `0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695` |
| Image | `layerfs-bench-infra:c8013528cc49547b` |
| Topology | host-store: macOS SDK/SQLite/spool; Docker Linux daemon/FUSE; 2 CPU / 2 GiB / no swap / 256 PIDs; no data mounts |
| Seed / samples | seed 1, one sample per performance case |
| Performance allowance | 300 s product / 310 s outer; 15 s family target is **reporting-only** |
| Verification allowance | 45 s work / 59 s hard (not 300 s) |

A later harness-only patch (Git case cache isolation + 16 MiB receipt cap) changed the host source seal to `d85e2a2bab2dacbd…` for five recollected performance rows. Product/image bytes did not change. Those five rows are `git-tool-500` and `dedup-history-{distributed,hotset,recurring,metadata}-500`.

## Inventory

Exact registry membership matches the issue-54 expected remaining set. Obsolete capped-family IDs are not counted again.

| Family | Expected | Performance attempted | Completed workload | Incomplete |
|---|---:|---:|---:|---:|
| `namespace_mutation` | 4 | 4 | 4 | 0 |
| `directory_construction_traversal` | 12 | 12 | 10 | 2 |
| `workspace_change_locality` | 16 | 16 | 14 | 2 |
| `dedup_branch_history` | 20 | 20 | 19 | 1 |
| `git_tool_workflow` | 4 | 4 | 4 | 0 |
| `mixed_load_bearing` | 4 | 4 | 4 | 0 |
| `workspace_reliability` | 28 proof-only | n/a | n/a | accounted |
| **Total** | **60 + 28** | **60** | **55** | **5** |

Completed nine issue-#38 families plus tiny-file churn remain the existing 138-case campaign. They were not rerun.

## Performance family table

Timer is `pure_call_sum_ns`. Ranges span different registered cases, not a confidence interval. One seed, one sample: not a median or distribution.

| Family | Completed | Across-case min s | Across-case max s | Historical 15 s TARGET_MISS |
|---|---:|---:|---:|---|
| `namespace_mutation` | 4/4 | 0.0238 | 31.4600 | `namespace-subtree-relocate-delete-500` |
| `directory_construction_traversal` | 10/12 | 0.0197 | 13.3827 | none among completed |
| `workspace_change_locality` | 14/16 | 0.0111 | 3.2270 | none among completed |
| `dedup_branch_history` | 19/20 | 0.0158 | 90.1450 | `dedup-history-unrelated-100` |
| `git_tool_workflow` | 4/4 | 0.4866 | 21.0850 | `git-tool-100`, `git-tool-500` |
| `mixed_load_bearing` | 4/4 | 0.0299 | 10.5851 | none |

A completed 21 s or 90 s row is a successful collection. It is not a 15-second PASS.

## Incomplete performance workloads

These are not completed timings and are not PASS.

| Case | Outcome | Evidence |
|---|---|---|
| `directory-metadata-scan-500` | FAIL, `readdir wide: ENOSPC` | 32k-entry `wide` listing in the 2 GiB container |
| `directory-content-scan-500` | FAIL, `readdir wide: ENOSPC` | same `wide` listing after 136000 preads |
| `workspace-dense-rewrite-100` | FAIL after Commit | `Commit published with failed FUSE presentation` |
| `workspace-dense-rewrite-500` | FAIL during Exec | `fsyncdir .: ENOSPC` after 100000 pwrites |
| `dedup-history-unrelated-500` | TIMEOUT at 300 s | stopped around commit 106/500; partial phases retained |

Cleanup passed on these rows. Partial receipts are under the campaign directory, including first-attempt history-500 truncation receipts in `attempts/`.

## Verification

Proofs do **not** use the 300 s performance timeout. `verify-selected.py` uses 45 s work / 59 s hard, and the runner keeps a 4 s cleanup reserve, so large proofs are killed around 41 s.

This campaign does not rerun those TIMEOUT proofs to chase a 300 s proof budget. Small-tier proofs of the same kind already cover the workload shape in 2–6 s.

| Verification outcome | Count | Meaning |
|---|---:|---|
| PASS | 56 | independent proof finished, cleanup PASS |
| TIMEOUT | 17 | 45/59 s proof ceiling; not a failed 300 s performance run |
| FAIL | 9 | assertion/product mismatch; see below |
| SKIPPED | 5 | matching performance workload did not complete |
| INCOMPLETE | 1 | `workspace-sustained-600s-compact-v2-proof` duration-incompatible |

Passing proof wall times: 1.7–39.8 s. Reliability short proofs were ~2 s; `exec-500` was 11.4 s.

Large-tier TIMEOUT proofs (`*-100` / `*-500` history, git, namespace, directory-construct, dense-sdk-edit, clean-commit-500, fixed-move-500) are mapped to the passing compact/tier-1/10 proof of the same kind. That mapping is coverage of the shape, not a passing 100/500 proof.

### Proof FAILs (not timeouts)

| Case | Observed |
|---|---|
| `git-tool-1-compact-v2`, `git-tool-10-compact-v2` | `canonical metadata mismatch: .` |
| `workspace-lease-lifecycle-compact-v2-proof` | second create returned `InvalidRequest`, not `WorkspaceBusy` |
| `workspace-open-writer-busy-compact-v2-proof` | open-writer Commit was not independently `Busy` |
| `workspace-live-execution-busy-compact-v2-proof` | live-exec Busy/cleanup mismatch under current live owner |
| `workspace-admission-batch-failure-retry-compact-v2-proof` | injected Commit did not surface the exact historical error |
| `workspace-final-publication-failure-retry-compact-v2-proof` | spill/early-admission/fault boundary not reached |
| `workspace-short-spool-write-compact-v2-proof`, `workspace-deferred-nospace-compact-v2-proof` | faulted write did not produce the exact errno proof |

These are current live-owner/FUSE semantics versus historical Busy/fault assertions. They were not weakened.

`workspace-sustained-600s-compact-v2-proof` is preserved and not shortened. It cannot fit a 60 s proof ceiling or a 300 s performance allowance.

## Setup reuse and cleanup

Compatible prepared masters were reused by fixture/schema/seed identity. Git references are now keyed by case as well as seed, because `git-tool-500` previously cache-hit `git-tool-100`'s reference (`Git reference case/seed identity mismatch`). Clone method remains `closed-quiescent-byte-copy`. Completed rows recorded `prepared_master_unchanged=true` and cleanup PASS. Container snapshots showed no data mounts.

Unavailable metrics are omitted or recorded as the runner emitted them; zeros are not invented for missing host/FUSE counters.

## Follow-up ownership

Leftover failed and slow rows are tracked in one ticket: [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61). Split follow-ups #55–#59 were closed as duplicates of that issue.

Large-tier independent proofs that hit the 45/59 s ceiling are not rerun at 300 s. Compact/tier-1/10 proofs of the same kind already cover those shapes. This campaign does not start Exec+Commit optimization and does not close #39.

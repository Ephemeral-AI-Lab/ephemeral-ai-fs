# LayerFS 0.1.3

> **Status: Closed — benchmark and engineering checkpoint completed 2026-09-08.**
> Delivered to `main` through [PR #76](https://github.com/Ephemeral-AI-Lab/layerfs/pull/76)
> at [`9f5a641d2`](https://github.com/Ephemeral-AI-Lab/layerfs/commit/9f5a641d223606c45e5e6aa8a20094c12f9139a1).
> This records roadmap completion. Package versions, a release tag, binary
> publication and an immutable versioned manual are separate release actions.

## Completed scope

v0.1.3 completes the Workspace, filesystem-tool, CAS/CDC, bounded single-Branch
history and reliability benchmark checkpoint, including inherited SDK edit,
namespace initialization and Store-footprint controls. The active runner registry
contains **17 families, 198 performance cases and 29 proof-only definitions**.
Every performance case also has a routine proof: **198/198 performance PASS and
226/226 routine verification PASS**, with one optional 600-second test explicitly
excluded and unexecuted.

[Full per-test tables](checkpoint-evidence/report.md) ·
[JSON report](checkpoint-evidence/report.json) ·
[Performance CSV](checkpoint-evidence/performance.csv) ·
[Verification CSV](checkpoint-evidence/verification.csv) ·
[Evidence and reproduction guide](checkpoint-evidence/README.md)

The [frozen registry](checkpoint-evidence/raw/registry.jsonl) and
[checkpoint contract](checkpoint-74-75.md) own the final inventory and coverage.
They supersede the original 12-family/130-case planning totals and Phase 1
suppression policies. Historical specifications and failed attempts remain
available; no obsolete case or failed receipt was relabeled as passing.

| Family | Performance PASS | Routine verification PASS | Excluded long proof |
|---|---:|---:|---:|
| `dedup_branch_history` | 20 | 20 | 0 |
| `dedup_cdc_locality` | 20 | 21 | 0 |
| `dedup_cross_file` | 10 | 10 | 0 |
| `dedup_workspace_reuse` | 14 | 14 | 0 |
| `directory_construction_traversal` | 12 | 12 | 0 |
| `edit_canonical_chunk_count` | 12 | 12 | 0 |
| `edit_length_changing` | 32 | 32 | 0 |
| `edit_length_preserving` | 12 | 12 | 0 |
| `git_tool_workflow` | 4 | 4 | 0 |
| `init_namespace` | 4 | 4 | 0 |
| `mixed_load_bearing` | 4 | 4 | 0 |
| `namespace_mutation` | 4 | 4 | 0 |
| `payload_create_read` | 8 | 8 | 0 |
| `store_footprint` | 6 | 6 | 0 |
| `tiny_file_churn` | 20 | 20 | 0 |
| `workspace_change_locality` | 16 | 16 | 0 |
| `workspace_reliability` | 0 | 27 | 1 |
| **Total** | **198** | **226** | **1** |

## Engineering delivery

- Simplified verification and shared preparation/collection across all families;
  preserved independent writable samples and separated setup, workload, proof and
  cleanup timing.
- Replaced expensive full-history payload replay with deterministic snapshots,
  retaining every Commit, every parent link and final-head checks. Flattened the
  recursive expected-content oracle that caused high-tier history timeouts.
- Repaired presentation recovery and failed-owner Discard, corrected obsolete
  Busy expectations, and reached actual streaming-admission and spool-failure
  boundaries with recovery and cleanup checks.
- Fixed stale folio writeback overwriting SDK edits during kernel-cache
  reconciliation; retained concurrent Workspace, command and mmap behavior.
- Preserved #73's corrected Git fixtures and full head/tree/parent/reopened-custody
  proof. Git and LayerFS Commit timings remain separately visible.
- Aligned routine SDK verification with the declared cold projection, retaining
  resource limits, canonical inode/root and payload-retention checks, and a
  post-commit FUSE boundary read. Versioned proof coverage and source/recipe
  bindings preserve the original failed proofs and subsequent requalification.

The supported measurement topology is macOS SDK/Store/SQLite/spool plus Linux
Docker daemon/FUSE/workloads, with the existing 2 CPU / 2 GiB / no-swap / 256 PID
container profile and separate host resource accounting.

## Final results and accepted limitations

Git-100 measured **1,879.182 ms** and Git-500 **4,689.306 ms** for the complete
LayerFS lifecycle. Each final performance case has one fixed-seed observation;
these are not percentile distributions or statistical speedup claims.

The checkpoint is closed with the following explicit boundaries:

- The historical 500/1,000 ms Git targets remain missed. Unrelated-history-500
  also exceeds the historical 15-second target at 18,163.889 ms. All active
  performance workloads pass the unchanged 300-second collection allowance.
- Nineteen proofs exceed the aspirational 15-second wall time; all routine
  proofs pass the unchanged 45-second work / 59-second hard deadline.
- The 600-second sustained definition is optional extended coverage, not PASS.
  Sampled history/content checks are not exhaustive historical object censuses.
- Cold SDK verification omits pre-edit FUSE inode-number stability; canonical
  inode preservation remains checked. It does not establish a warm-cache/mmap
  resource bound. Separate live coherence tests retain their own coverage.
- Existing crash/power-loss durability limitations remain unchanged. This
  checkpoint introduces no broader durability or multi-history scaling claim.

## Closure evidence

- [x] Freeze and reconcile the active runner inventory and inherited controls.
- [x] Complete the per-family simplification review and supported fast entrypoints.
- [x] Pass all 198 performance cases and 226 routine proofs in one shared campaign,
  retaining pre-work refusals, failures, explicit requalification and exclusions.
- [x] Publish per-family/per-test Markdown, JSON and CSV, phases, resources,
  comparisons, coverage and reproducible raw evidence.
- [x] Execute the three gated live Docker tests on the final product, pass shared
  runner/report checks, and pass required Rust CI on the exact PR head.
- [x] Merge the implementation and evidence to main and close
  [#74](https://github.com/Ephemeral-AI-Lab/layerfs/issues/74) and
  [#75](https://github.com/Ephemeral-AI-Lab/layerfs/issues/75).

[Green exact-head CI](https://github.com/Ephemeral-AI-Lab/layerfs/actions/runs/34151299860)
and the [published evidence guide](checkpoint-evidence/README.md) record validation
and source compatibility. The central [v0.1.3 roadmap issue #21](https://github.com/Ephemeral-AI-Lab/layerfs/issues/21)
is closed. The [original planning checklist](planning-history.md) is archival.

## Next scope

[v0.1.4](../0.1.4/README.md) prioritizes storage efficiency through the shared
Init/Commit pipeline. The previously drafted multi-history expansion moves to
[v0.1.5](../0.1.5/README.md), which owns multi-Layer/multi-Branch history, Fork, Add, Diff,
conflicts, fan-out and broader history-query scaling. Additional FUSE/Git
optimization or extended endurance qualification is follow-up work; it does not
reopen this completed checkpoint or turn unmet stretch targets into passes.

# LayerFS v0.1.3

**Status: Release candidate for LayerFS 0.1.3 Developer Preview.**

LayerFS v0.1.3 grew out of a benchmark-driven optimization cycle across the
Workspace lifecycle: payload creation, directory traversal, bulk changes, SDK
edits, Git workflows, content reuse, repeated checkpoints, and recovery. The work
reduced repeated construction, temporary-file management, copying, spill readback,
and remote acquisition, while repairing correctness failures uncovered by those
workloads.

The final benchmark checkpoint was merged through
[PR #76](https://github.com/Ephemeral-AI-Lab/layerfs/pull/76) at
[`9f5a641d223606c45e5e6aa8a20094c12f9139a1`](https://github.com/Ephemeral-AI-Lab/layerfs/commit/9f5a641d223606c45e5e6aa8a20094c12f9139a1).
[PR #78](https://github.com/Ephemeral-AI-Lab/layerfs/pull/78) subsequently recorded
roadmap closure. This changelog describes the development cycle. The
[release record](../../../release-notes/0.1.3/README.md) and
[versioned manual](../../versioned/0.1.3/README.md) define release verification,
artifacts, compatibility, and supported behavior.

Read in order: **[Highlights](#highlights) → [Engineering details](engineering.md)
→ [Benchmark families and every case](#benchmark-families-and-every-case).**

## Highlights

1. **Commit without Workspace reconstruction.** Candidate-bound checkpoint facts
   are installed into existing live nodes instead of rediscovering their state
   after publication. In the original 20,000-file / 100 MiB development
   investigation, snapshot database calls fell from **108,006 to six**. Across
   that optimization sequence, recorded create Commit time fell from **6.99 s to
   0.64 s (~11×)** and delete Commit from **1.68 s to 0.30 s (~5.5×)**. These are
   historical development observations, not final qualified speedup guarantees.[^commit]
   [How it works →](engineering.md#1-commit-without-workspace-reconstruction)

2. **Shared temporary backing segments.** Logical files now reference shared
   append ranges rather than owning separate physical spools. The original
   20,000-write workload used **101 physical segments**; recorded retirement time
   fell from **2.08 s to 2.57 ms** in successive development observations, with
   reference lifetime, rollback, and retained-byte accounting preserved.[^commit]
   [How it works →](engineering.md#2-shared-temporary-backing-segments)

3. **Ordered namespace finalization.** Bounded final-inode coalescing, paged
   deletion, and batched reference processing reduce intermediate tree work.
   Original bulk-delete snapshot queries fell from **27,546 to approximately
   1,400**. Add-before-remove reference updates and repaired unseen-hard-link
   handling preserve surviving aliases.[^commit]
   [How it works →](engineering.md#3-ordered-namespace-finalization)

4. **Owned output and checked object admission.** Finalized authenticated buffers
   move into admission without redundant borrowed copies. Original bulk-create
   spill readback fell **96%, from 113.43 MB to 4.26 MB**, with approximately
   **109 MB delivered memory-owned** and **zero borrowed delivery copies** in the
   measured path. Spilled objects still receive fresh validation.[^commit]
   [How it works →](engineering.md#4-owned-output-and-checked-object-admission)

5. **Payload and content-reuse scaling.** Reused valid contiguous capture,
   corrected spill ordering, preserved compact sequential backing, and improved
   bounded native task partitioning. A payload-500 development comparison fell
   from **3.63 s to 2.45 s**. Four-family qualification covered **114 samples and
   30 scaling comparisons in each of two topologies**, under the recorded
   acceptance criterion.[^scaling]
   [How it works →](engineering.md#5-payload-and-content-reuse-scaling)

6. **Shared live Workspace execution.** FUSE operations and SDK edits use one
   live core. Paged fact transfer and checkpoint installation preserve live
   nodes and handles; Commit uses filesystem-operation boundaries so commands
   can continue on the same live-FUSE Workspace. The final mixed-file create-100 and
   delete-100 lifecycle observations were **0.990 s and 0.259 s**.[^final]
   [How it works →](engineering.md#6-shared-live-workspace-execution)

7. **Authenticated read reuse and kernel caching.** Bounded immutable acquisition
   and optional read-only page prefill reduced high-tier Git backing requests by
   **about 92%** and FUSE READ callbacks by **97%–98%**. Complete Git lifecycle
   time improved from **5.83 s to 1.85 s** and **14.12 s to 4.64 s**, approximately
   **3×**. The implementation uses unmodified upstream `fuser` 0.18.0.[^git]
   [How it works →](engineering.md#7-authenticated-read-reuse-and-kernel-caching)

8. **Writeback coherence and recovery.** Fixed delayed mapped-page writeback
   overwriting SDK edits, stale tails undoing truncation, failed-owner Discard,
   and recovery after publication succeeded but updating the live view failed.
   All **three required live Docker tests executed and passed** on the final
   checkpoint product.[^final]
   [How it works →](engineering.md#8-writeback-coherence-and-recovery)

9. **Reusable benchmark preparation.** Consolidated setup, collection, and
   verification while retaining independent writable samples and separate
   producer/executor identities. Removed **10,377 net lines** of legacy benchmark
   infrastructure.[^infra]
   [How it works →](engineering.md#9-reusable-benchmark-preparation)

10. **Bounded history verification.** Flattened recursive expected-content
    recipes and selected explicit high-tier snapshots while retaining every
    Commit identity, parent link, and final head. Five previously timed-out
    development proofs completed in **3.66–9.14 s**; sampled coverage remains
    explicit.[^verification]
    [How it works →](engineering.md#10-bounded-history-verification)

11. **Complete routine benchmark checkpoint.** Published **198/198 performance
    passes and 226/226 routine verification passes across 17 families**, with
    source identities, resource and cleanup results, comparison limits, and
    retained failures. One separately registered long proof was excluded.[^final]
    [How it works →](engineering.md#11-final-benchmark-checkpoint)

## Final checkpoint results

The following are **one fixed-seed observation per case** at the final checkpoint.
They are distinct from the historical optimization comparisons above. Performance,
routine verification, resource, and cleanup checks passed for each listed case.

| Workload | Work performed | Primary time | Timer |
|---|---|---:|---|
| Mixed bulk create, tier 100 | Create 1,000 files / 100 MiB; retain witness tree | **0.990 s** | Complete Workspace lifecycle |
| Mixed bulk delete, tier 100 | Delete 1,000 files / 100 MiB; retain witness tree | **0.259 s** | Complete Workspace lifecycle |
| Mixed bulk create, tier 500 | Create 5,000 files / 500 MiB; retain witness tree | **5.054 s** | Complete Workspace lifecycle |
| Mixed bulk delete, tier 500 | Delete 5,000 files / 500 MiB; retain witness tree | **0.933 s** | Complete Workspace lifecycle |
| Namespace initialization | Registered 100,000-file namespace | **2.603 s** | Native initialization |
| Payload creation | Create 500 MiB payload | **3.068 s** | Complete Workspace lifecycle |
| Git workflow, tier 100 | Registered file changes and six Git commands | **1.879 s** | Complete Workspace lifecycle |
| Git workflow, tier 500 | Registered file changes and six Git commands | **4.689 s** | Complete Workspace lifecycle |

Complete Workspace lifecycle is the declared sum of Create, workload execution,
LayerFS Commit, visibility, and End. Git's own `git commit` is part of workload
execution and is separate from LayerFS Commit. Setup and independent verification
are outside performance timing. These sums are not the complete benchmark command
wall time.

The measurement topology is **macOS SDK/Store/SQLite/spool plus Linux Docker
daemon/FUSE/workloads**. Containers use 2 CPUs, 2 GiB memory, no swap, and a 256-PID
limit, without data-sharing mounts. Host CPU and memory are separately accounted;
the container limits do not cap the host. See the
[checkpoint environment and evidence guide](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/README.md).

## Benchmark families and every case

**Performance PASS** means the complete registered workload succeeded within the
checkpoint's declared execution, resource, and cleanup gates. It does not mean
every historical latency target was met. **Routine verification PASS** means the
declared proof checks succeeded, with sampled coverage and omissions recorded.

Each family link opens its full case table in the immutable checkpoint report.
The report retains family-specific timers, exact case IDs, phases, resources,
comparisons, proof coverage, and raw evidence links.

| Family and full case table | Performance PASS | Routine verification PASS | Excluded long proof |
|---|---:|---:|---:|
| [Single-Branch history](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#dedup_branch_history) | 20 | 20 | 0 |
| [CDC locality](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#dedup_cdc_locality) | 20 | 21 | 0 |
| [Cross-file CAS deduplication](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#dedup_cross_file) | 10 | 10 | 0 |
| [Workspace content reuse](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#dedup_workspace_reuse) | 14 | 14 | 0 |
| [Directory construction and traversal](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#directory_construction_traversal) | 12 | 12 | 0 |
| [Canonical chunk-count edits](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#edit_canonical_chunk_count) | 12 | 12 | 0 |
| [Length-changing SDK edits](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#edit_length_changing) | 32 | 32 | 0 |
| [Length-preserving SDK edits](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#edit_length_preserving) | 12 | 12 | 0 |
| [Git workflow](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#git_tool_workflow) | 4 | 4 | 0 |
| [Namespace initialization](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#init_namespace) | 4 | 4 | 0 |
| [Mixed agent work episodes](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#mixed_load_bearing) | 4 | 4 | 0 |
| [Namespace mutation](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#namespace_mutation) | 4 | 4 | 0 |
| [Payload creation and random reads](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#payload_create_read) | 8 | 8 | 0 |
| [Store footprint](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#store_footprint) | 6 | 6 | 0 |
| [Tiny-file operations and bulk churn](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#tiny_file_churn) | 20 | 20 | 0 |
| [Workspace change locality](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#workspace_change_locality) | 16 | 16 | 0 |
| [Workspace reliability](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md#workspace_reliability) | — | 27 | 1 |
| **Total** | **198** | **226** | **1** |

The inventory has 198 performance cases, each with a routine proof, plus 29
proof-only definitions: one CDC boundary proof and 28 reliability definitions.
One reliability definition is the excluded 600-second test; 198 + 1 + 27 = 226
executed routine proofs. Earlier planning totals and retired case definitions
are not additional final-checkpoint passes.

## Known limits and measurement scope

- **Historical investigations are not final release measurements.** The original
  0.636-second create Commit observation had no independent benchmark proof at
  that stage, and its complete lifecycle remained 15.893 seconds. Spool-retirement
  observations varied; the recorded millisecond result is not a universal bound.
- **Workload revisions are explicit.** Original 20,000/100,000-file tiny-file
  recipes, mixed-v3 bulk cases, mixed-v4 trees, and history-v2 snapshots are
  different definitions. Reducing snapshot writes from 200 files to ten is
  benchmark load reduction, not a 20× product speedup.
- **Final timings are observations, not distributions.** The create-100 result
  is 0.989887791 seconds, only about 10 ms below one second. It does not establish
  a repeated subsecond guarantee. Git's before/after study has one corrected
  baseline observation and three final runs; the separate checkpoint has one
  observation per case.
- **Targets remain separate from successful execution.** The historical Git
  targets of 500/1,000 ms remain missed. Unrelated-history-500 took 18.164 seconds,
  above the historical 15-second target but within the unchanged 300-second
  collection allowance. The earlier native initialization 2.2-second/10% CPU
  improvement target was not achieved by the shared-construction refactor.
- **Verification is scoped.** High-tier history uses selected snapshots and
  omits exhaustive historical object census. Cold SDK proofs omit pre-edit FUSE
  inode-number stability while retaining canonical inode preservation. Nineteen
  routine proofs exceed the aspirational 15-second wall; all pass the unchanged
  45-second work / 59-second hard deadline. The separate 600-second endurance
  test was not executed on this checkpoint.
- **Architecture and durability boundaries remain.** Shared construction helpers
  do not mean every native fallback and Commit route has converged; ordinary
  Commit retained one worker. Upstream `fuser` is integrated, while
  [#51](https://github.com/Ephemeral-AI-Lab/layerfs/issues/51) retains separate
  qualification obligations. This checkpoint does not establish new crash or
  power-loss durability, broad multi-Branch scaling, or physical 100-Workspace
  qualification. Open research PRs #36/#37 are not counted as delivered features.

## Details and evidence

- [Numbered engineering explanation](engineering.md)
- [Checkpoint scope, environment, and reproduction guide](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/README.md)
- [Full family, case, and proof tables](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md)
- [Performance CSV](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/performance.csv)
- [Verification CSV](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/verification.csv)
- [Machine-readable JSON report](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.json)
- [Frozen registry, declarations, and raw receipts](https://github.com/Ephemeral-AI-Lab/layerfs/tree/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/raw)

The full tables remain in their existing published location. Links are pinned to
the checkpoint revision so subsequent development cannot silently change the
evidence behind this changelog.

[^commit]: [Original-workload Commit investigation and source-bound attempt ledger](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/issue47-subsecond-workspace-results.md). Counters, phase timings, proof status, and source identities belong to their recorded attempts.
[^scaling]: [Payload/CAS/CDC/Workspace scaling investigation](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/issue38-progress.md). The final criterion was normalized adjacent-tier median growth strictly below 1.25; Docker and host-owned Store results retain distinct topology/resource scopes.
[^git]: [Git implementation and corrected comparison](https://github.com/Ephemeral-AI-Lab/layerfs/blob/5c9cce92b446ba8d953c30c9df41b4feccb497df/docs/roadmap/0.1/0.1.3/issue68-git-optimization-results.md), [all measured counters](https://github.com/Ephemeral-AI-Lab/layerfs/blob/5c9cce92b446ba8d953c30c9df41b4feccb497df/docs/roadmap/0.1/0.1.3/issue68-evidence/report.md).
[^infra]: [Infrastructure cleanup and net deletion](https://github.com/Ephemeral-AI-Lab/layerfs/issues/45#issuecomment-5546984074).
[^verification]: [History oracle, selected snapshots, and recovery proof results](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-74-75.md).
[^final]: [Final checkpoint results, checks, and coverage](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/README.md).

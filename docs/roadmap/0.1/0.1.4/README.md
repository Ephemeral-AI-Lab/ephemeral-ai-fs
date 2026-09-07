# LayerFS 0.1.4 — Storage efficiency

> **Status (2026-09-08):** Planned storage-efficiency phase following the
> completed v0.1.3 benchmark checkpoint. Optimization design, numerical targets,
> implementation scope, and new benchmark admission remain open. The proposed
> architecture is recorded below; no implementation has started.
>
> **Compatibility:** Existing compatibility and acknowledgement requirements
> remain in force. This roadmap decision authorizes no Store-format or public
> semantic change.

## Problem statement

LayerFS is intended to retain filesystem state at agent tool-call boundaries.
Localized-edit experiments demonstrate useful checkpoint latency, while the
DeepSeek Harness retained-history experiment and matched Git control show a
substantial storage-efficiency gap. Frequent checkpoints need both economical
retention and acceptable foreground operations.

The next phase therefore prioritizes storage efficiency. The previously drafted
multi-agent/multi-Branch and multi-Layer benchmark expansion moves to
[v0.1.5](../0.1.5/README.md). Existing semantics and their required correctness
checks remain obligations in v0.1.4; only the broader benchmark expansion moves.

## Goal

Reduce storage cost through the shared filesystem capture/construction and
SQLite publication path used by namespace initialization and Workspace Commit.
Preserve correctness, ordinary and historical reads, and demonstrated operation
performance. Workspaces per tool call are an expected agent usage flow. The direction is an outcome, not a selected storage design.
No speedup, storage ratio, or universal advantage over Git is promised.

## Research boundary

The [storage-efficiency boundary](storage-efficiency-boundary.md) records the
SQLite-only storage constraint, synchronous shared Init/Commit scope,
read/write cost concerns, exclusion of new durability/crash-recovery work,
conditional speed/storage tradeoffs,
and preference for a new benchmark family. The benchmark definition and test
environment are placeholders for a separate discussion. No optimization
implementation is selected.

## Proposed architecture

The [storage architecture specification](storage-architecture-spec.md) records
the proposed shared Init/Commit design: exact CAS/COW reuse, current CDC,
shallow delta records, bounded compression groups, and immutable SQLite pack
BLOBs. Small and large files use one pipeline. It includes read/write costs,
multiple-transaction publication, Git-comparison limits, and boundary checklists.
Prototype parameters are proposals; binary format, compatibility, benchmark and
environment decisions remain open.

The [SQLite storage-format walkthrough](sqlite-storage-format.md) illustrates
the proposed database, pack/group/record layouts, shallow deltas, and shared
read/write flows with SQL examples and diagrams. Its schema is conceptual.

## Supporting evidence and tracking

- [Issue #18 — v0.1.4 storage-efficiency planning](https://github.com/Ephemeral-AI-Lab/layerfs/issues/18)
  carries the reprioritized storage discussion and its historical research.
- [Issue #72 — DeepSeek Harness checkpoint-history experiment](https://github.com/Ephemeral-AI-Lab/layerfs/issues/72)
  supplies retained-history evidence and related controls. Its original contract,
  comments, candidate identities, and results remain unchanged.
- [Evidence index](evidence.md) records exact report links, measurements, and
  limitations. Supporting experiments are not qualification of a future candidate.
- [v0.1.3 checkpoint](../0.1.3/checkpoint-evidence/README.md) records the merged
  baseline, including target misses and exclusions.

## Scope of this planning decision

- Prioritize retained storage efficiency for v0.1.4.
- Preserve historical evidence, existing public behavior, and required integrity
  and lifecycle checks, including shared content across Branches.
- Assess foreground operations, historical reads, resource use, maintenance, and
  temporary space alongside retained allocation when the evaluation is defined.
- Preserve the append-only benchmark contract and define any new measurements
  before collecting candidate evidence.
- Discuss optimization approaches, format compatibility, numerical budgets, and
  implementation/release gates separately before implementation begins.

The durable storage boundary is SQLite-only; packs inside SQLite are permitted
as research candidates. No encoding, packing layout, compaction policy, chunk
profile, or small-file representation is selected. It starts no implementation or
benchmark campaign and does not import historical footprint targets as new gates.

## Follow-up planning

- [ ] Agree the optimization proposal and compatibility scope.
- [ ] Freeze evaluation populations, accounting, budgets, and acceptance gates.
- [ ] Implement and validate only the agreed scope.
- [ ] Publish candidate-specific storage and operation results, limitations, and
  required regression evidence before release closure.

## Subsequent release

[v0.1.5](../0.1.5/README.md) owns the deferred multi-history scaling plan:
Commit depth, Branch fan-out, Fork, Add, Diff, Query, conflicts, historical reads,
reopen, and cross-Branch storage reuse. This resequencing changes no admitted
scenario identity or released contract.

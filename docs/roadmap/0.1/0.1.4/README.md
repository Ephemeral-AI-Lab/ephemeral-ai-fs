# LayerFS 0.1.4 — Storage efficiency

> **Status (2026-09-08):** Planned storage-efficiency phase following the
> completed v0.1.3 benchmark checkpoint. Architecture v3 now specifies physical
> framing, shared ownership, admission races and read/write bounds. Compatibility/release policy requires an owner
> decision. Development smokes have a separate agreed plan; full qualification
> remains open. This revision starts no product implementation or execution.
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
performance. Workspaces per tool call are an expected agent usage flow. The
proposal targets that outcome without claiming measured qualification.
The storage objective is to get sufficiently close to matched Git allocation
with a worthwhile storage/latency tradeoff; exact parity is not required or ruled
out. No speedup, storage ratio, or universal advantage over Git is promised.

## Research boundary

The [storage-efficiency boundary](storage-efficiency-boundary.md) records the
SQLite-only storage constraint, synchronous shared Init/Commit scope,
read/write cost concerns, exclusion of new durability/crash-recovery work,
conditional speed/storage tradeoffs,
and preference for a new benchmark family. Full-family benchmark and evaluation
details remain open. The separately agreed development
smokes and topology are linked below; no implementation or execution begins here.

## Proposed architecture

The [storage architecture specification](storage-architecture-spec.md) records
the proposed shared Init/Commit design: exact CAS/COW reuse, current CDC,
shallow delta records, bounded compression groups, and immutable SQLite pack
BLOBs. Small and large files use one pipeline. It includes read/write costs,
multiple-transaction publication, Git-comparison limits, and boundary checklists.
Exact proposed wire framing and engineering bounds are specified; they are not
measured settings. The [compatibility transition](storage-architecture-spec.md#compatibility-transition)
proposes a new versioned Store with no conversion on open and leaves the release/legacy
policy to the owner. Full qualification remains open; the development-smoke plan
owns its confirmed scope/topology and remaining execution prerequisites.

The [SQLite storage-format walkthrough](sqlite-storage-format.md) illustrates
the proposed database, pack/group/record layouts, shallow deltas, and shared
read/write flows with SQL examples and diagrams. Its proposed schema and wire
format are precise but not an executable migration.

The [development-smoke scope and topology](storage-efficiency-boundary.md#development-smokes-and-qualification)
link PR #80's first-five-checkpoint replay, edit and small-file cases. This is
separate from the still-open full benchmark family and numerical gates.

The [review disposition](review-disposition.md) records each original finding,
its correction, the subsequent linear-work/batching audit and remaining
policy/measurement limitations. Design confidence
does not qualify storage ratios or speed.

The [implementation plan](implementation-plan.md) lists required source reading,
the proposed final module tree, change/deletion ownership and staged rollout. Its
only executable verification is the three agreed development smokes; this planning
change runs nothing and does not authorize migration or full release qualification.

Use the [implementation handoff prompt](implementation-handoff-prompt.md) to assign
the implementation with persistent milestone/smoke/fix iteration, a joint storage/
speed completion contract and explicit evidence limits. Creating the prompt does
not start that work or approve the pending compatibility policy.

Use the [design-review prompt](design-review-prompt.md) for an independent
review of clarity, storage/speed tradeoffs, aggregate multi-project load, future
cloud compatibility, and minimal final components and operation paths.

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
as research candidates. The revised encoding/layout is a concrete recommendation,
not implementation authorization. It changes no canonical chunk profile or
small-file representation. It starts no implementation or benchmark campaign and
does not import historical footprint targets as new gates.

## Follow-up planning

- [ ] Agree the optimization proposal and compatibility scope.
- [ ] Complete PR #80's remaining development-smoke prerequisites in that workstream;
      discuss the separate full benchmark family and numerical qualification gates.
- [ ] Implement and validate only the agreed scope.
- [ ] Publish candidate-specific storage and operation results, limitations, and
  required regression evidence before release closure.

## Subsequent release

[v0.1.5](../0.1.5/README.md) owns the deferred multi-history scaling plan:
Commit depth, Branch fan-out, Fork, Add, Diff, Query, conflicts, historical reads,
reopen, and cross-Branch storage reuse. This resequencing changes no admitted
scenario identity or released contract.

# v0.1.4 storage-efficiency research boundary

Status: owner-directed planning boundary, 2026-09-08. This records storage scope
and tradeoff guidance. It is not a benchmark contract, selected implementation,
or permission to collect qualification samples. Benchmark and test-environment
sections are deliberately placeholders for the next discussion.

Related: [v0.1.4 scope](README.md), [supporting evidence](evidence.md),
[issue #18](https://github.com/Ephemeral-AI-Lab/layerfs/issues/18), and
[issue #72](https://github.com/Ephemeral-AI-Lab/layerfs/issues/72).

## Objective

Improve storage efficiency in the shared filesystem capture/construction and
SQLite publication pipeline used by namespace initialization and Workspace
Commit. Preserve most of LayerFS's demonstrated operation performance,
correctness, and historical readability. Storage efficiency and operation cost
must be assessed together. No universal advantage over Git is claimed.

A Workspace per agent tool call is an expected application workflow, not the
core storage abstraction or a required checkpoint cadence. Initialization and
Commit must benefit through the shared product path; avoid a separate
agent-specific encoder or storage pipeline.

The broader multi-agent/multi-Branch scaling campaign remains v0.1.5. Required
correctness of existing shared-object and Branch behavior is not deferred.

## Durable storage boundary

All authoritative durable Store content remains inside SQLite for this phase:
Branches, Commits, roots, object identities, lookup information, and stored
object representations. Packs are acceptable candidates if stored inside
SQLite, such as pack BLOBs. External durable pack files and a second durable
object backend are out of scope.

```text
SQLite Store
  ├── Branches, Commits, roots
  ├── Object identities and lookup information
  └── Object representations
        ├── Individual values, if selected
        └── Packs inside SQLite, if selected
```

This boundary selects no schema, pack size, encoding, or compaction lifecycle.
It does not prohibit existing runtime spools or legitimate temporary maintenance
files. Required SQLite journals/sidecars, spools, and temporary space must be
accounted for in their declared scopes; they must not conceal retained storage
or become an unacknowledged external durable object store.

## First-design execution boundary: synchronous and blocking

The owner prefers a fast synchronous shared pipeline for namespace
initialization and Workspace Commit: capture or construct the filesystem state,
admit new objects and required metadata, and publish into SQLite. Selected
encoding must finish before the corresponding public operation returns success.
No background encoder, packer, or later compaction pass may be required to
achieve that operation's reported storage efficiency.

Include the work wherever it occurs in the declared foreground lifecycle. Work
performed during mutation must not disappear from accounting because the final
Commit call is short. Synchronous completion does not mean holding a SQLite
write transaction during all preparation. Synchronous completion refers to
operation completion, not a new durability guarantee.

Prefer incremental work on newly admitted content and required metadata. This
boundary does not require rewriting or globally repacking all retained history
on every Commit. Any bounded base search, compression, or pack construction
selected later must fit the agreed foreground budget. Internal parallel work is
not prohibited, but required work must complete before acknowledgement and its
resources must be counted.

Measure the primary retained footprint at acknowledgement under this policy.
Do not substitute a separately compacted footprint for it. Future maintenance
or reclamation remains a separately scoped decision; it must not be used to
hide costs or rescue the first design's storage claim.

## Scope exclusion: durability and crash recovery

The owner excludes new durability guarantees, crash-recovery design, and crash
or power-loss qualification from this storage-efficiency phase. Do not expand
the research into those workstreams. Existing behavior is not intentionally
weakened; ordinary correctness, reported failures, object authentication, and
readability of successfully retained states remain required.

## Read-heavy and write-heavy use

Neither reads nor writes may be assumed rare. Evaluate namespace initialization,
change capture and Commit, ordinary filesystem reads, and historical reads as
separate costs. Compression and representation decoding must not be hidden by
a faster publication phase or by a favorable average workload mix.

Record canonical encoding/decoding, compression/decompression, authentication,
object lookup, and SQLite I/O distinctly where measurement permits. Determine
which paths reach stored objects versus already available Workspace data; do
not assume every read decompresses or every write recompresses an entire file.

Research should consider avoiding re-encoding reused objects, excessive decode
size for small reads, repeated decoding of shared metadata, and unnecessary
compression of tiny or incompressible values. These are questions for a shared
implementation, not selected cache, codec, threshold, or layout policies.
Any cache benefit must include its memory cost and behavior on misses. Read and
write amplification and any delta-base reconstruction must remain visible.

## Research candidates, not implementation commitments

| Area | Question |
| --- | --- |
| Compression within SQLite | How much content saving is possible while retaining SQLite publication and stable logical object identities? |
| Packing inside SQLite | Does grouping encoded objects reduce overhead enough to justify the indexing and lifecycle work? |
| Delta encoding | Do similar retained versions offer additional savings that justify base selection, reconstruction, and dependency management? |
| Small-file representation | Does avoiding unnecessary per-file objects or mappings materially reduce overhead? |

These are possible investigations, not four required implementations or a frozen
execution order. External-pack comparison is excluded by the SQLite-only
boundary. Algorithm, codec, thresholds, schema, and implementation choices remain
open. Representation changes require explicit compatibility review rather than
being treated as physical encoding changes automatically.

## Performance versus storage tradeoff

The owner accepts some performance sacrifice for a substantial storage gain:

- Around 50% more elapsed time can be acceptable if retained storage becomes as
  efficient as Git under the eventual matched comparison.
- Around 50% more elapsed time for only 10% less retained allocation is not
  acceptable.
- Most of the useful checkpoint performance should be preserved. A 50% increase
  is a conditional tradeoff example, not a blanket allowance for every operation.

For this discussion, interpret “50% slower” as 1.5 times elapsed latency
(for example, 4 ms to 6 ms), not half the throughput. Final operation-specific
limits and aggregation rules remain to be agreed.

Use explicit quantities when the measurement contract is defined:

```text
storage_reduction = 1 - candidate_allocation / baseline_allocation
latency_increase = candidate_latency / baseline_latency - 1
git_storage_multiple = candidate_allocation / matched_git_allocation
```

The denominator must have a declared, matching scope. Undefined quantities remain
unavailable with a reason. Retained allocation includes required storage overhead;
canonical bytes and candidate reuse are separate measurements, not substitutes.

| Outcome | Planning disposition |
| --- | --- |
| Modest storage gain with negligible latency impact | Potentially worthwhile if complexity is small |
| 10% less storage with 50% more elapsed latency | Unacceptable tradeoff |
| Substantial storage gain with modest latency impact | Worth evaluating against the full set of requirements |
| Git-comparable retained storage with about 50% more foreground latency | Potentially acceptable, subject to correctness, reads, resources, and the final contract |
| Greater slowdown or a materially different tradeoff | Requires a separate decision; no blanket authorization |
| Corruption, lost retained states, weakened acknowledgements, or violation of agreed hard limits | Unacceptable regardless of savings |

“Git-comparable” is deliberately not assigned a numerical tolerance yet. The
previously suggested 1.25× band was a proposal, not an agreed boundary. No
historical absolute MB target is adopted as a v0.1.4 gate.

## Accounting and behavioral safeguards

- Compare unchanged LayerFS and the candidate under the same eventual operation
  and environment contract. Keep Git's retained-history and workflow comparisons
  separately scoped and explicitly matched where a comparison is claimed.
- Report namespace initialization, capture/Commit, ordinary reads, and historical
  reads separately. Complete tool-call lifecycle is an additional application
  measurement when selected. Do not hide a severe case regression in an average.
- Report absolute elapsed time as well as ratios. Numerical latency, tail-latency,
  resource, and noise/repetition rules remain to be specified.
- Count immediate storage and any post-maintenance storage separately. Include
  the CPU, elapsed time, I/O, and peak temporary allocation required to reach the
  claimed compact state, plus any interference with foreground operations.
- Preserve public filesystem behavior, historical content, isolation, and
  authentication. Preserve existing acknowledgement behavior; new durability
  and crash-recovery work is outside this phase.
- Physical encoding should preserve logical object identities. Any schema,
  canonical representation, or compatibility change needs a separately agreed
  contract and existing-Store handling before implementation.
- Preserve historical evidence and failed outcomes. Existing exploratory reports
  do not qualify a future candidate and must not be relabeled as new samples.

## New benchmark family — placeholder

**Owner preference: create a new benchmark family for this research.** Existing
families and reports are supporting evidence; they are not automatically the new
family's benchmark baseline or acceptance population. Existing infrastructure
may be reused without inheriting old scenario identities or silently changing
old contracts. Required product regression obligations remain separate.

To discuss next:

- Family name, purpose, exact claim, and scenario IDs: **TBD**.
- Baseline/candidate revisions and Git comparison scope: **TBD**.
- Fixtures, file populations, edit schedules, checkpoint counts, and tiers: **TBD**.
- Public operation surfaces, lifecycle, and timing boundaries: **TBD**.
- Repetitions, ordering, cache treatment, and decision statistics: **TBD**.
- Verification coverage, independent oracle, and failure classification: **TBD**.
- Numerical storage/performance/resource gates and artifact contract: **TBD**.

No earlier proposed family list, tier sequence, repeat count, or latency result
is adopted here as the new family's specification. Freeze the new contract
before benchmark implementation or qualification sampling.

## Test environment — placeholder

The environment will be discussed separately. This document specifies no new
topology, hardware profile, container limits, timeout, or preparation policy.
Existing repository rules remain in force until explicitly amended; leaving
this section open does not authorize a conflicting environment.

- Host/container ownership and hardware/runtime identities: **TBD**.
- Resources, timeouts, measurement coordination, and background activity: **TBD**.
- Input preparation, transfer, build reuse, cache policy, and sample isolation: **TBD**.
- Resource sampling, Store/temporary allocation measurement, and cleanup: **TBD**.

## Next decision

Define the new benchmark family and its consistent test environment, then agree
numerical tradeoff gates before implementing or measuring storage candidates.

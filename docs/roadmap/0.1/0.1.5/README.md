# LayerFS v0.1.5: bounded delta storage

> **Status:** Implementation and fixed smoke completed, 2026-09-09. See the
> [implementation notes](implementation-notes.md) and [matched smoke report](smoke-report.md).
> Both arms verified all 31 retained states; the candidate exercised 10 FULL and
> 30 DELTA SmallContent objects. This is exploratory evidence, not release qualification.

The current issue #100 follow-up has a [three-arm ten-snapshot baseline](issue100/ten-snapshot-baselines.md)
for fast iteration: Git 38.22 MB, released v0.1.4 67.15 MB, existing v0.1.5 66.11 MB
allocated, with all ten states verified. This smaller history does not supersede
the initial full157 regression. See the [frozen smoke contract](issue100/ten-snapshot-contract.md)
and [growth analysis](issue100/growth-estimate.md); no storage optimization or release
qualification is claimed from the baselines.

The ready-to-use [implementation prompt](implementation_prompt.md) combines this
scope, fixed settings and fast smoke-only loop.

## What to implement

New or changed nonempty file content below **131,072 bytes** becomes one
whole-file canonical CAS payload, physically stored as FULL or a one-level DELTA
against a FULL small-content object. Exactly 128 KiB and larger files retain the
existing CDC/extent representation and known-range edit locality. Empty files
retain their existing compact representation. Unchanged old objects and histories
are not rewritten to adopt the new policy.

Namespace Init and workspace Commit use the same content construction, admission,
packing, dependency, and publication machinery. Init discovers source files;
Commit captures live FUSE/SDK state and supplies predecessor/dirty-frontier facts.
Those input adapters remain distinct; no duplicated storage pipeline is added.

CAS identifies complete canonical content, independently of FULL/DELTA storage.
Small-file length-changing edits may assemble/hash the complete bounded target;
large-file known edits must not scan untouched content. Keep live POSIX semantics,
trusted ownership, batching, rollback, and publication behavior intact.

## Read in this order

1. [Workflow](workflow.md): user-facing architecture and ASCII workflows, including
   live workspace state, shared Init/Commit, history, and size transitions.
2. [Specification](spec.md): fixed representation, encoding, schema/compatibility,
   ownership, transitions, and completion contract. This is the technical authority.
3. [Implementation plan](implementation_plan.md): concrete file responsibilities,
   order of work, fast builds, and smoke-only verification.
4. [Delta-encoding benchmarks](delta-encoding-benchmarks.md): the fixed first-round
   smoke and explicitly deferred broader qualification.
5. [Past mistakes](past_mistake.md) and [benchmark success](benchmark_success.md):
   source-bound performance pitfalls and existing-family contracts.

[Hybrid mental model](hybrid_mental_model.md) is the compact conceptual companion.
[Existing architecture](existing_architecture.md) describes the released/historical
baseline; [v0.1.4 benchmark report](benchmark-v0.1.4-report.md) records its results
and later optimization backlog. Neither is a command to run a broad campaign now.

The earlier chunk-member FILE_DELTA proposal has been replaced, including its
multiple-member directory, base-part descriptor, and all-size extent-only policy.
It is not an alternative implementation route.

## Fixed settings and scope

- Small-file boundary: **128 KiB**, strictly below is small.
- Existing CDC: **8 KiB minimum / 16 KiB target / 32 KiB maximum**.
- SQLite: **4 KiB pages** for new Stores; preserve supported existing 64-KiB layouts.
- Codec/depth/buffer parameters: fixed in the spec; no parameter search.
- Use existing Zstandard, Store, ownership, transport, and runner facilities.
- No global similarity index, delta chains for new small objects, background GC,
  repacker, new storage service, or dependency hunt.
- Multi-Branch/multi-Workspace scope remains [v0.1.6](../0.1.6/README.md).

Implement the complete v0.1.5 scope, including readers, transitions, compatibility,
base lifetime, failure handling and documentation. A small verification campaign
does not authorize omitting these production requirements.

## Baseline and repairs to preserve

Start from released **v0.1.4**, commit
`101fa273d815f3aaedb0e06ba0de7b0777d83def`, qualified product
`9cfb4be477116646258ea0621280ed13b1824c6d`. Preserve other tasks' work and use an
isolated checkout when necessary; do not overwrite this older main checkout with
release files piecemeal.

Retain #95 Init's bounded authenticated comparison reuse, #98 Workspace SQL
coalescing and staging handoff, and the 64-KiB ordered spill read-ahead cap. These
are released optimizations, not gains attributable to v0.1.5.

The release's [benchmark closeout](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/release-notes/0.1.4/benchmark-closeout.md)
records remaining regressions and ineligible comparisons. Do not infer universal
speed gains from release acceptance or omit those limitations in future claims.

## First-round implementation loop

Use the single ten-file/thirty-commit ordinary-FUSE smoke specified in
[the benchmark document](delta-encoding-benchmarks.md#first-round-execution-scope).
Collect a released control for the exact same fixture before measuring the
candidate. Then implement, build the needed targets, run that smoke, verify its
31 states, inspect the actual cost, and fix the measured cause.

Build incrementally and reuse valid host/image artifacts. Diagnose slow builds,
lock waits, setup, or verification rather than repeatedly paying the same cost.
Do not run already-passed checks again without a change that invalidates them.
No Cargo test/Clippy/doctest suite, three-family matrix, full157, cutoff sweep,
page-size experiment, or separate read campaign in this implementation round.

Keep the original [three-file tiny baseline](tiny-history-baseline-v1.md) immutable.
Its 88-KiB initial / 216-KiB final allocation and 13.709125-ms observation are not
ten-file results. Its 64-KiB-growth/15-ms candidate gates remain attached to that
original case and are deferred with its rerun. The new smoke has its own matched
comparison and evidence scope.

## Completion versus release

Implementation completion requires all specified paths wired, the matching build
and single smoke passing, its history verified, and a candid record of storage,
latency, limitations and unrun qualification. It is not permission to publish a
release or claim exhaustive correctness.

The v0.1.5 canonical/physical format extension is an explicit owner-authorized
exception to the normal patch-format rule; see [release policy](../../../general/release-policy.md).
Old bytes/identities stay readable, new format writes are fenced, and upgrade is
explicit. Existing MEMORY-journal/synchronous-OFF acknowledgements do not become a
power-loss durability guarantee. Broader format, failure, POSIX and performance
qualification follows separately before release.

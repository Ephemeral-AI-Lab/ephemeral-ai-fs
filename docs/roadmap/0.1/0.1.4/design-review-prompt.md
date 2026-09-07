# v0.1.4 storage design review prompt

Copy the prompt below into a review task. It requests a review and recommendations,
not implementation, test-verification planning, or a new benchmark campaign. The review must distinguish
mechanistic reasoning from measured proof; document review cannot guarantee
storage ratios or latency.

---

Act as an independent reviewer of LayerFS's proposed v0.1.4 storage design.
Determine whether it is clear, internally consistent, likely to materially improve
storage efficiency with reasonable operation overhead, and capable of evolving
toward cloud deployment. Challenge assumptions rather than endorsing the proposal.

Optimize for the smallest clean final architecture, not the smallest patch.
A substantial rewrite or deletion can be the correct recommendation if the final
system has fewer concepts, procedures, interfaces, and round trips. Conversely,
do not recommend a rewrite merely for aesthetics or speculate about future needs.

## Task boundary and source custody

This is a read-only review. Do not modify product code, benchmark definitions,
GitHub issues, or the reviewed design. Do not launch a benchmark campaign or
collect candidate samples. Produce the review report in your response; if the
owner requests a file, write a separate report without changing the reviewed
contract. Do not repeatedly ask for approval for normal read-only investigation.

Read applicable AGENTS.md and review skills. Record the repository revision,
branch, and reviewed document identities. Preserve unrelated working-tree edits.
Inspect the proposed PR/branch if the files are not on main; do not substitute an
older roadmap silently. PR #77 merged at `28177560c8f049c02192e18c263cdc5543c1ab52`; the revised
specification builds on its independent audit. Use the actual revision under
review, not the original worktree as an authority. The original proposal is in PR #77:
https://github.com/Ephemeral-AI-Lab/layerfs/pull/77

The original documentation worktree is:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-roadmap`
Branch: `codex/storage-roadmap`.
Treat these as discovery hints, not permission to reset or change a checkout.

## Required reading

Read these documents in full:

- `docs/roadmap/0.1/0.1.4/README.md`
- `docs/roadmap/0.1/0.1.4/storage-efficiency-boundary.md`
- `docs/roadmap/0.1/0.1.4/storage-architecture-spec.md`
- `docs/roadmap/0.1/0.1.4/sqlite-storage-format.md`
- `docs/roadmap/0.1/0.1.4/evidence.md`
- `docs/roadmap/0.1/0.1.4/review-disposition.md`

Read relevant compatibility and architecture context:

- `docs/roadmap/0.1/README.md`
- `docs/roadmap/architecture.md`
- The logical-identity-versus-SQLite-identity section of
  `docs/research/vision/layerfs:ai-native-file-system.md`
- Current schema/admission/publication and public reader implementation.
- `docs/roadmap/0.2/agent-branch-reconciliation/README.md` and relevant current-model
  constraints, distinguishing future reconciliation from present behavior.

Read issues #18 and #72 and the immutable reports linked by the evidence index.
Read issue #52 for future cloud intent:
https://github.com/Ephemeral-AI-Lab/layerfs/issues/52
Its referenced `docs/roadmap/0.3/README.md` may not be published; report that gap
rather than inventing its contents. Do not make v0.3 implementation a v0.1.4 gate.

## Product mental model you must preserve

LayerFS is a filesystem with immutable retained state and mutable COW Workspaces,
not merely a repository archive compressor.

- CAS gives canonical identity and exact object sharing.
- CDC discovers reusable content regions during construction/capture.
- COW extent and namespace trees preserve unchanged content ranges and structure.
- Agents mount a Workspace through FUSE, read files, make edits, and capture/Commit
  changes. A Workspace per tool call is an expected application workflow, not a
  mandatory low-level storage cadence or a separate storage engine.
- Namespace Init and Workspace Commit must converge on the same shared canonical
  construction, physical encoding, admission, and SQLite publication pipeline.
- “Commit deltas” can mean captured filesystem changes or COW differences. These
  are distinct from physical binary delta records; do not conflate them.
- Accurate range edits can preserve extents directly. Whole-file replacement
  requires discovery of reusable content; observed writes do not automatically
  identify which bytes equal an earlier version.
- Small and large files use the same storage path. CDC minimum is not allocation
  padding. Growing across 8 KiB does not require relocating historical objects.
- Init and Commit have distinct public/lifecycle semantics even when they share
  construction and storage. Do not erase staging/no-change/head-check behavior
  merely to make their diagrams identical.

## Agreed scope and open decisions

All authoritative stored data remains in SQLite, including pack BLOBs and object
locations. No external durable packs, cloud service, or second backend in this
phase. Existing temporary spools remain separately accounted.

Required encoding completes synchronously before the corresponding public
operation returns. No background packer or later compaction may be required to
achieve the claimed footprint. Multiple bounded SQLite admission transactions
are allowed; synchronous does not mean one giant transaction or holding the DB
writer during compression.

New durability guarantees, fsync policy, crash recovery, failover, and power-loss
qualification are out of scope. Preserve ordinary live-operation correctness,
authentication, retained-state readability, and existing acknowledgement behavior.

The user prefers a NEW benchmark family. Benchmark population and environment
are intentionally TBD. Review whether the design states what needs validation,
but do not fill those placeholders, adopt old family populations, or turn this
review into benchmark implementation. Existing benchmark rules still apply.

The revised architecture/format specify proposed wire fields, bounds, codec,
hint provenance, admission races and selection approximation. Review those concrete
rules rather than treating them as unspecified; they are not measured conclusions.
The compatibility transition remains an explicit owner policy decision. The new
benchmark family/population, environment, numerical gates, test-verification plans
and all execution are deferred until specification discussion; do not fill them
or classify their intentional deferral as an architecture defect. Identify only
remaining architectural contradictions or owner-required decisions. A large code
change is acceptable; a silent incompatible format or public-semantic change is not.

## A. Clarity and end-to-end consistency

Trace the actual current implementation through callers and shared helpers:

1. Namespace source discovery to canonical construction and SQLite publication.
2. Mounted Workspace reads/writes to capture, construction, admission, staging,
   final Commit publication, and required finalization.
3. Object lookup to authentication and serving filesystem bytes.

Starting points include `crates/layerfs-layerstack-store/src/objects.rs`,
`layerstack.rs`, `workspace.rs`, `staging.rs`, `schema.rs`, SQL statements,
`crates/layerfs-content/src/file/`, and Workspace/FUSE callers. Follow references
as needed; do not assume a planning document describes the implemented path.

For each proposed diagram and operation, identify its owner, input, output,
identity, mutable state, memory ownership, lock/transaction lifetime, failure
boundary, and required result. Flag ambiguous terms or contradictions between
the SQL example, binary layout, architecture spec, and boundary document.

Check that logical ObjectIds remain independent of pack IDs and rowids; full
versus delta records remain independent of raw versus compressed groups; and
object, group, pack, operation, and transaction boundaries are not conflated.

## B. Storage benefit and performance risks

Assess the incremental contribution of:

- Exact CAS reuse and COW structural/range reuse.
- Group compression of full payload, delta records, and metadata.
- Bounded shallow deltas and periodic new full anchors.
- Packed SQLite placement and its remaining per-object index cost.
- Small-file structural overhead that physical packing cannot remove.

Do not add overlapping savings estimates. Include object indexes, group/record
framing, base references, full anchors, SQLite allocation, partial packs, and
unreferenced records from failed or racing admissions. Address the case where
one synchronous operation admits only a handful of objects and cannot wait for
future calls to improve group compression.

Review whether the proposed delta-selection heuristic can choose sensibly after
group compression without excessive trial encoding. Trace availability and cost
of base hints for both Init and Commit. Do not assume a global similarity index
or an accurate byte-level diff exists. Ask whether shallow chains sacrifice so
much compression that the Git objective becomes unlikely, and propose the
smallest alternative only when justified.

Review reads as seriously as writes: partial BLOB access, directory/index reads,
small random reads, metadata traversal, sequential reads, base fetches, decode
amplification, hashing, copies, allocation, cache misses and cache memory. Flag
whole-pack reads hidden behind a nominal object API, repeated group decoding,
unbounded decompression, accidental deep delta chains, and cache-dependent claims.

The owner accepts roughly 50% MORE elapsed latency for genuinely Git-comparable
storage, but rejects 50% more latency for only 10% less storage. This is a tradeoff
example, not a blanket regression allowance. Do not adopt the earlier unapproved
1.25× Git tolerance. Distinguish foreground operation latency, reads, queue time,
Init throughput, and total resource consumption. Give uncertainty and validation
needs instead of invented speed or storage guarantees.

Use the matched Git control, not an unmatched repository-history denominator.
Git's separately packed size does not mean packing was included in commit time.
Retain metadata, interface, candidate, and host/container qualifications from the
reports. Synthetic localized-edit advantages do not establish a universal win.

## C. Aggregate load and database contention

LLM latency means an individual agent may issue calls slowly: approximately
1 QPS typical and up to about 10 QPS is the owner's expectation. Do NOT turn this
into a hard aggregate Store or machine ceiling. Multiple agents and projects on
the same machine can overlap. The rate's per-agent/per-project/per-Store meaning
is unresolved and must be stated as an assumption in any calculation.

Analyze both possible deployments without preselecting one:

- Multiple projects sharing one Store: aggregate writer, connection, admission,
  lookup, compression, and publication contention.
- Separate project Stores on one machine: separate DB locks but shared CPU, disk,
  memory, cache and scheduler pressure; no assumed cross-Store deduplication.

Use explicit estimates such as aggregate arrival rate = sum of project/agent
rates, and utilization of a serialized stage approximately equals arrival rate
multiplied by its total service time per operation. Label assumptions. Tool-call
rate is not SQL transaction rate: each call may cause multiple admission batches,
reads, staging and publication transactions. A large Init or formatter call can
produce far more work than a small edit at the same QPS.

Examine writer lock duration, shared reader/writer connection locking, compression
outside locks, bounded queueing, fairness, peak memory across active operations,
and whether a large Init can delay small Commits or reads. Reuse existing
coordination first. Do not invent worker pools, schedulers, per-Branch mutex maps,
read replicas, or sharding just because concurrency is mentioned.

## D. Future cloud compatibility without cloud implementation

Determine whether the local format preserves a credible evolution path toward
#52, without claiming SQLite can simply be placed on shared object storage or
that replication merges independently writable databases.

Review these seams:

- Canonical identity and filesystem semantics do not depend on a local path,
  SQLite rowid, process pointer, or pack placement.
- A future exporter can translate local locators into portable pack references;
  clarify whether packs require reindexing, reframing, or repacking. Do not assume
  numeric local pack IDs are globally meaningful or demand all packs be globally
  content-addressed now without a concrete benefit.
- Pack/group boundaries support bounded range retrieval and explicit versioning.
  Small groups and depth-one bases may still cause multiple remote requests;
  describe request fan-out and cross-pack dependencies rather than claiming
  bounded decode bytes automatically imply low network latency.
- Physical delta-base closure can be enumerated for transfer and retention.
  A selected root's logical graph alone may omit required physical bases.
- Future remote bytes can be bounded, decoded, and authenticated without trust
  in a location index. Required framing and codec dependencies remain explicit.
- The publication boundary can later sit behind an authoritative metadata service
  without changing logical object identity. Do not implement that service now.
- Future tenant/access boundaries must not assume global CAS identity is access
  authorization or that cross-project deduplication is automatically permitted.
- A SQLite-replication route and an object-storage route have different migration
  costs; assess lock-in and evidence gaps without requiring both implementations.

Classify each cloud concern as a format blocker now, a documented migration cost,
or future cloud work. Local durability/crash work remains excluded. Versioning,
portable identity, ordinary validation, and physical dependency descriptions may
be necessary now; replication, distributed consensus, remote recovery, tenant
services, and cloud benchmarks are not v0.1.4 deliverables.

## E. Minimal final implementation, not minimal patch

Evaluate the complete resulting system, including what can be deleted.
Recommend a clean shared implementation even if it changes many files or removes
large legacy paths. Preserve unrelated user edits and required public behavior;
“destructive refactor” does not authorize deleting user data or silent breakage.

Produce a KEEP / MERGE / REPLACE / DELETE / DEFER ledger for components and steps.
For each proposed addition, identify its indispensable responsibility, why an
existing component cannot own it, and what obsolete code it replaces. Reject
one-implementation plugin interfaces, parallel Init/Commit encoders, unnecessary
layers, redundant caches/indexes, a separate small-file backend, and speculative
cloud abstractions unless current evidence justifies them.

Trace at least one small Commit, one bulk Init, and one object-read miss. Count
logical operations and crossings: public methods, process/Exec calls, daemon/RPC
round trips, DB connection-lock acquisitions, SQL statements, transaction commits,
object lookups, BLOB range reads, compression/decompression calls, and buffer
copies. Distinguish a local SQL statement from a network round trip.

For every stage or pass ask:

- Can it be removed, fused, batched, or handled by an existing owner?
- Are we hashing, traversing, copying, decoding, or checking membership twice?
- Is metadata duplicated in the object index, pack directory, group directory,
  and record header? Is each copy necessary for access or validation?
- Can construction carry trusted facts forward rather than rediscover them?
- Does a smaller API remove call choreography without weakening semantics?
- Does combining steps increase writer lock duration, memory, or read amplification?
- Are extra transactions required by the existing lifecycle or merely an artifact
  of the proposed container layout?

Do not optimize for the fewest lines or calls at the expense of correctness or
bounded work. Batch appropriately; do not hold one giant transaction simply to
minimize transaction count. Avoid benchmark-specific paths and silent fallbacks.

## Required review output

Lead with **READY FOR DETAILED DESIGN / NEEDS REVISION / NOT JUSTIFIED**, with
separate confidence assessments for storage benefit, read/write overhead, and
cloud evolution. Do not treat “ready” as measured qualification or implementation
approval while benchmark/environment and compatibility decisions remain open.

Then provide:

1. **Findings, highest severity first.** Cite document/code file and line, explain
   the concrete trigger and impact, identify evidence versus inference, and give
   the smallest corrective recommendation. No vague warnings.
2. **Boundary matrix.** PASS / GAP / CONTRADICTION / INTENTIONALLY TBD for shared
   Init/Commit, CAS/CDC/COW, small files, synchronous SQLite packs, multiple
   transactions, read/write costs, load assumptions, tradeoff policy, compatibility,
   cloud seams, and benchmark/environment placeholders. Do not mark intentional
   placeholders as design failures merely because they are incomplete.
3. **One recommended architecture diagram.** Show only necessary final components,
   with ownership and read/write/publication boundaries. Explain material changes
   from the current proposal.
4. **Minimalism ledger and operation counts.** Current versus proposed required
   steps/crossings for small Commit, bulk Init and read miss. Use unknown where
   implementation detail is not available; do not invent measured counts.
5. **Storage/performance assessment.** Expected benefit mechanism, likely dominant
   overhead, aggregate-load risks, and exactly what evidence is missing. No promise
   of Git parity based only on the presence of compression and deltas.
6. **Cloud assessment.** Format blockers now versus acceptable migration costs and
   explicitly deferred cloud responsibilities.
7. **Concrete document corrections.** Quote replacement wording or provide small
   proposed text snippets for the important ambiguities; do not edit the source.
8. **Finding closure and next decisions.** Map every original finding to resolved,
   remaining decision, or deferred measurement. Identify only necessary remaining
   architecture/policy decisions; leave family/population, environment, numerical
   gates and test-verification plans for the separate discussion. Assess design
   clarity, consistency, bounded mechanisms, minimalism and cloud seams separately
   from empirical storage/speed confidence. Do not manufacture very-high empirical
   confidence from improved prose.

If a proposed mechanism does not justify its complexity, recommend removing or
simplifying it explicitly. If a larger rewrite yields a cleaner final system,
state the replacement boundary and deletion scope. The review should leave us
with a clearer, smaller design and precise bounded claims, not a larger wish list or a new verification plan.

# Commit optimization direction and scaling requirements

Analysis dated 2026-09-12 against main `faaa03933` plus the preserved dirty
compaction-removal treatment. Read-only product investigation with three
independent subagents: Commit/tree construction; metadata/admission resource
bounds; eager SDK/FUSE edit work. No product optimization or new performance
measurement was performed for this analysis.

Source evidence: [completed measurement](commit-baseline-results.md),
[frozen measurement protocol](commit-baseline-contract.md), and
[owner-linked issue comment](https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5637126575).
All recorded Commit timings use uncontrolled OS cache. Store reopening is a
lifetime distinction, not a cold-cache claim. The original cold Init gate stays open.

## Decision

Continue with Commit optimization, but correct the earlier proposed mechanism.
First attribute the expensive inode-finalization work inside `namespace_ns`;
do not implement speculative directory batching. In parallel as a design
priority, make removal of the identified quadratic edit/spill mechanisms a
required acceptance condition, even where K=100 does not trigger them.
Implement and measure each mechanism separately, rather than stack fixes.

The user requires CPU and memory bounds and elimination of quadratic scaling.
That means preserving linear or near-linear useful work in changed items and
bytes, not promising sublinear processing of K real changes. A fixed memory
limit alone does not satisfy the CPU requirement.

## What the measurements establish

At 100,000 files, public Commit medians are 18.84 ms for one retained change,
80.50 ms for K=10, and 313.29 ms for K=100. Namespace phases are 5.97,
62.86, and 241.33 ms respectively. K=100 object admission is 42.53 ms.
No-change is 2.12 ms. First changed Commit after Store reopen is 142.64 ms,
with approximately 127 ms spent rebuilding the metadata index.

K=100 SDK edit preparation costs another 659.08 ms. Adding its median to
the Commit median gives about 972 ms of context, not a measured median of
per-sample end-to-end sums. Future reports must compute those sums per sample.
K=1/10/100 timing growth is not itself proof of quadratic behavior.

### Correction to the preceding report's hypothesis

`crates/layerfs-workspace/src/changes.rs:805` finishes the content timer;
the namespace timer starts at 806 and covers reference application and
`inodes.finish` (812–820). The preceding dirty-directory loop is therefore
not the measured namespace phase. Content-only edits also introduce no
directory-binding deltas.

`FrontierInodes::finish` already submits all sorted inode deltas together to
`inode_table_apply_sorted_with_budget` at `changes.rs:2414`. The claim that
K measured changes require K independent directory-tree reconstructions is
unsupported. The old ~230 ms saving ceiling is not a defensible prediction.

The batch engine does have potentially unnecessary work: it reads each child
before the no-delta early return (`crates/layerfs-content/src/tree/batch.rs:579`,
519), while compact-leaf decode derives a canonical inode-value hash for
every decoded record (946–955). Untouched sibling records can therefore pay
derivation cost. This is a code-supported hypothesis, not yet an internally
measured explanation for the 241 ms.

Minimum separate diagnostics: reference application; checkpoint/record
validation; individual record encoding; batch tree application; fallback.
Reuse existing `TreeBatchCounters` (reads, created/reused nodes, changed keys,
scratch peak), currently discarded by `FrontierInodes::finish`.
Within the batch, count reads/authenticated bytes, decoded leaf records,
synthetic record hashes, changed keys, emitted nodes/bytes and fallback causes.
Preserve page authentication, subtree-summary checks, canonical chunking,
split/merge rules, roots and reference counts.

## Scaling findings and required remedies

Use K for changed items, B for pending-buffer capacity, H for metadata history,
C for successive reopen/Commit cycles, E for edit ranges, and F for dirty folios.

| Mechanism | Evidence and classification | Required direction |
|---|---|---|
| Full dirty-fact publication before every SDK edit | `crates/layerfs-fuse/src/live_owner.rs:2627` calls `freeze`; `freeze` at 2404 invokes `publish_facts` at 2416. Publication clones/re-encodes accumulated dirty state at 2420–2565. For K distinct edits, cumulative fact rows can grow as 0+1+...+(K-1): confirmed quadratic mechanism; contribution to 659 ms is unmeasured. | Separate the edit barrier from full Commit snapshot publication. Keep full publication for wire FREEZE/actual snapshot consumers. Prove host edit handlers need authoritative live state and immutable backing, not a freshly republished whole dirty snapshot. Preserve acknowledgements, failure recovery, fencing and spool ownership. |
| Growing cache/retirement scans at edit barriers | `live_owner.rs:2344` scans cached inodes; `retire_ranges` at 2949 scans retained backing references. Repeated barriers over growing sets can be quadratic. | Count visits first; maintain incremental dirty/retirement worklists or a correctly scoped edit barrier. Never simply remove kernel invalidation/writeback safety. |
| Range-by-folio cross product | `live_owner.rs:1175` tests edit ranges during folio writeback. E ranges × F folios is a quadratic family when both grow; a 4096-edit cap only bounds damage. | Normalize overlapping intervals once and use indexed overlap lookup/ordered traversal, preserving original edit order and semantics where they matter. Test mmap/writeback, overlapping edits and range boundaries. |
| Repeated whole-run spill merge | `crates/layerfs-workspace/src/changes.rs:2259`; explicit O(N² / buffer capacity) comment at 2288. Each full pending buffer merges with the entire growing spill: confirmed O(K²/B) record traffic. K=100 does not hit the default spill threshold. | Size-tiered sorted runs using the existing record format and journal. Merge similarly sized runs so each record is rewritten O(log(K/B)) times; preserve lookup/update semantics and failure-atomic installation. |
| Batch failure followed by sequential mutations | `changes.rs:2430–2450` falls back on Unsupported/ObjectLimitExceeded and applies each mutation separately. Repeated traversal/rewrite risk is confirmed; exact asymptotic behavior depends on the fallback implementation and shape. | Instrument and force both fallback causes. Provide a bounded streaming/batched route with an explicit work bound; do not silently restore a growing full-tree scan per mutation. No blanket claim that this fallback is quadratic before tracing it. |
| Repeated metadata history replay | `crates/layerfs-layerstack-store/src/objects/metadata.rs:117–152` initializes and synchronizes an index across H values. One reopening is O(H log W), with fixed W=131072. Growing history plus reopen after each Commit gives a quadratic aggregate sum of H. | Separate later workstream. A bounded predecessor proof can accelerate supported shapes, but its unknown-value full-history fallback does not satisfy a universal history-scaling requirement. A persistent exact lookup/recovery design needs its own schema/storage/correctness assessment. |

Store connection also validates the metadata catalogue (`metadata.rs:47`),
so removing replay from Commit alone does not make reconnect+Commit independent
of history. Retaining the Store avoids repeated replay for that lifetime; it
is not a general fix for the reopened workflow.

The current fingerprint lookup is not a discovered K² problem: it deduplicates
requested hashes, sorts candidate ordinals, authenticates each implicated group
once per lookup, and performs exact set comparisons (`metadata.rs:191–265`).
Do not replace it or remove authentication without a new measured reason.

## Proposed implementation order

1. Add the minimal inner namespace and edit-work counters in a separate
   diagnostic treatment. Reproduce the current K=1/10/100 shape and identify
   the dominant inner operation. This resolves the incorrect directory hypothesis.
2. Remove redundant intermediate fact snapshots if the consumer/fencing tests
   support it. Audit the other edit-barrier scans in the same trace, but keep
   independently measurable changes in separate experiments.
3. Optimize the measured inode-finalization operation. Lazy synthetic record-ID
   derivation is a candidate only if counters establish it is material; preserve
   authenticated canonical pages and all tree validation.
4. Replace quadratic spill merging and qualify low-budget/fallback paths. This
   is a required scaling repair, even if small-cell latency is unchanged.
5. Address repeated-reopen history work separately. Do not promote the previous
   guarded proof automatically: its retained noninferiority/storage screen is
   unresolved, and it retains an O(H) fallback. See
   [the rejected screen](metadata-proof-experiment-results.md).

## CPU and memory contract to freeze before implementation

These are proposed design/experiment limits, not measured candidate properties.

- No new workers, larger SQLite pages, higher cache settings or weaker encoding
  and authentication. Keep 4 KiB pages and current storage treatment.
- Prefer removing allocation/work. Do not start by adding a tree cache.
  If extra scratch is necessary, use one explicitly accounted aggregate budget,
  proposed at at most 8 MiB additional memory per active operation. Account for
  capacity, buffers, descriptors and concurrency; a per-operation cap is not
  a process-wide cap. Bound concurrent owners or use a shared charged budget.
- Tiered spill retains the existing pending allowance. Use checked run lengths
  and offsets, bounded run descriptors (at most 64 levels for u64 counts), and
  a fixed number of I/O buffers. Charge temporary output disk space before
  merging; preserve the prior runs until success. Never fall back to the old
  quadratic merge when the memory budget is exhausted.
- Retain existing metadata limits: 4 MiB scratch cache, 32 MiB scratch file,
  131072 retained fingerprints, 512 KiB candidate-ordinal vector; all existing
  admission, DELTA trial/read, expansion and predecessor-chain budgets remain.
  These component limits do not by themselves establish process peak memory.
- Measure stage user+system CPU and operation peak memory on both host and
  daemon. Post-call RSS cannot prove a peak bound. Check swaps, OOM, threads,
  spill disk growth and cleanup. At matched concurrency, incremental observed
  peak must stay within the declared additional-memory budget.
- For unchanged/no-change/K=1/revert cases, proposed paired CPU noninferiority
  tolerance is max(10% of control, 1 ms); evaluate alongside operation counts
  to avoid hiding growth in noise. K=100 CPU must decrease with wall time.
  Freeze the paired aggregation and uncertainty rule before seeing results.

## Targets and proof

First changed-tree experiment targets (aspirational, not forecasts):

| Metric, matched uncontrolled-cache retained Store | Baseline | Proposed useful-win target |
|---|---:|---:|
| K=100 namespace | 241.33 ms | <=120 ms |
| K=100 public Commit | 313.29 ms | <=200 ms |
| K=10 namespace | 62.86 ms | <=31 ms |
| K=10 public Commit | 80.50 ms | <=50 ms |
| K=1 / no-change / repeat / revert | See baseline | No material paired regression |

The targets ask for approximately half the namespace cost, not its complete
elimination. The inner attribution may show that a smaller improvement is the
honest limit; report that without moving the frozen promotion gate. Do not
claim a reopened-Commit target from the obsolete ~250 ms control. For edit work,
first require linear fact publication/visit counts and lower edit+Commit CPU;
set a millisecond target only after attributing the 659 ms edit interval.

Retain the exact original K=1/10/100 cells, plain/diagnostic separation, paired
control/candidate order, receipts, seals, runner lock and independent proofs.
Add separate adversarial cells so the main fixture remains unchanged:

- Independently vary N, K, directory width and depth; same-directory versus
  spread changes; content edits versus binding mutations, hardlinks and deletion.
- Cross spill thresholds with K around B and then 2B/4B/8B; use small-budget
  deterministic tests for the algorithm and at least one production-budget run.
- Force split/merge, Unsupported and memory-limit fallbacks. Check failed I/O,
  rollback, retry, complete changed bytes, canonical roots and reopen results.
- Repeated edits with growing dirty/cache/backing sets; many edits to one file;
  overlapping range unions and growing folio count. Keep mmap/FUSE writeback proof.
- Vary H and reopen count independently; cross the 131072 index window;
  include unknown/reverted metadata and collision tests.

Deterministic work counters are the complexity gate: O(K) fact records for a
fixed number of actual snapshots, near-linear/logarithmic spill traffic,
bounded indexed range queries, and no hidden fallback full-scan multiplier.
Include actual authenticated/output bytes and depth in tree bounds. Ten times
the changed work may require about ten times the useful processing; it must
not trigger one hundred times the same repeated scan. Timings alone at three
small K values cannot establish that property.

This analysis identifies concrete quadratic mechanisms and a roadmap to remove
them. It does not claim that the code is already CPU/memory safe under all
workloads, that all paths have been exhaustively audited, or that any of the
proposed gains has been achieved. The no-quadratic acceptance condition remains
open until the fixes and adversarial work-count proofs pass.

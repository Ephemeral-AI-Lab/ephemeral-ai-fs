# Building LayerFS v0.1.3 through benchmarks

> Status: Release candidate for LayerFS 0.1.3 Developer Preview. This engineering
> account accompanies the [v0.1.3 changelog](CHANGELOG.md). It distinguishes
> exploratory measurements from the final published development checkpoint.

The v0.1.3 cycle put LayerFS through whole-Workspace workloads: large payloads,
dense directories, bulk changes, repeated checkpoints, content reuse, Git and
recovery. Those workloads exposed repeated construction, expensive temporary-file
cleanup, serial work hidden inside apparently parallel code, and coherence bugs
between live filesystem operations and SDK edits. Improving the shared machinery
underneath those workloads became the central task.

The sections below explain the changes and the evidence behind them. Historical
measurements retain their original workload and source boundaries. The final
checkpoint is [PR #76, source
`9f5a641d223606c45e5e6aa8a20094c12f9139a1`](https://github.com/Ephemeral-AI-Lab/layerfs/pull/76).
Writing this document did not execute new benchmarks or tests.

## 1. Commit without Workspace reconstruction

An early bulk-create measurement revealed that publishing a checkpoint was only
part of Commit's cost. After publication, LayerFS reconstructed the continuing
Workspace from the canonical snapshot, rediscovering information construction
had just produced. Creating 20,000 files containing 100 MiB led to 108,006
Commit-scoped snapshot database calls. Reconstructed continuation consumed
5.259 seconds inside a 6.988-second Commit.

The replacement carries final backing facts through construction. A bounded,
buffered checkpoint journal records live NodeIds, canonical inode identities,
content roots and attributes. Installation updates existing nodes while retaining
their paths, pins and directory relationships. It checks that the returned
publication root and mutation generation match the journal, validates every
handoff before changing live backing, and supports retry after partial
installation. Carrying facts forward therefore removes rediscovery without
weakening the publication boundary.

The first implementation reduced snapshot calls to five; later observations
recorded six. Commit initially fell to 5.225 seconds, exposing a different
bottleneck: 2.692 seconds spent retiring temporary spools. The important result
was the removal of an entire class of repeated work. See the [historical attempt
ledger](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/issue47-subsecond-workspace-results.md)
and the retained [publication identity and installation
checks](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/crates/layerfs-workspace/src/lifecycle.rs#L260).

## 2. Shared temporary backing segments

Temporary storage originally coupled each edited logical file to its own
physical spool. Thousands of small files consequently meant thousands of
descriptor and pathname lifetimes to finish at Commit. Making content
construction faster left that cleanup cost visible on the critical path.

The existing PieceTree now describes ranges in shared backing segments. The host
owns each segment's descriptor, physical length and accounting; logical ranges,
delayed reads and acknowledged remote references retain its lifetime. Segment
creation uses an exclusively created file and immediately removes its temporary
name. A segment retires only after its last external reference disappears.
Partially unused shared segments remain charged until then.

Rollback protects other files sharing the segment. An append must match the
expected physical tail. A failed append truncates only its unpublished tail; if
that cleanup also fails, retained physical bytes remain accounted for and the
error propagates. The original 20,000-file workload used 101 physical segments
after this change. In adjacent development observations, retirement fell from
2.079 seconds to 2.571 milliseconds and Commit from 3.562 to 1.271 seconds.
Later retirement measurements varied, so these are individual observations of
the mechanism, not a guaranteed cleanup latency. The final source retains the
[segment ownership](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/crates/layerfs-workspace/src/file_io.rs#L113)
and [retirement and rollback
checks](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/crates/layerfs-workspace/src/file_io.rs#L1241).

## 3. Ordered namespace finalization

Bulk changes also exposed repeated intermediate inode-table construction and
record lookups. The revised `FrontierInodes` coalesces final values in a bounded
ordered map, spills sorted fixed-width records when necessary, and resolves
reads through pending changes, spilled changes and then the immutable base.
Tombstones participate in that precedence. One ordered final stream feeds the
existing canonical inode builder.

Deletion now consumes bounded directory pages and batches authenticated inode
reads. Reference processing applies additions before removals, coalesces aliases
within each page, and lets current changes override prefetched base facts. This
prevents a move or alias update from temporarily reaching zero references and
incorrectly reclaiming live content. Tests also exposed edited pathless inodes
with unseen hard links; repairs preserved those edits for both pinned and
unpinned cases.

The original delete workload performed 20,000 unlinks and 233 directory removals.
Historical Commit observations fell from 1.680 to 0.460 and then 0.304 seconds;
snapshot calls fell from 27,546 to 1,385, later 1,393. These are cumulative
development observations, not a controlled final comparison. The implementation
remains bounded rather than claiming an ideal external sorter: repeated
new-key spill merges are a known ceiling. See the [ordered
overlay](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/crates/layerfs-workspace/src/changes.rs#L1541)
and [batched reference
processing](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/crates/layerfs-workspace/src/changes.rs#L2038).

## 4. Owned output and checked object admission

Construction could own complete validated objects yet still pay to select,
copy, spill, reread and authenticate them again. The revised fast paths transfer
immutable authenticated object owners through a bounded output queue to checked
admission. Completed full-file construction also avoids redundant per-file
graph-selection scaffolding where final reachable output is already established.
Incremental and provisional paths retain their required selection.

Memory-owned bytes move into admission without another copy or authentication
pass. Storage reads still authenticate: selected spilled objects are ordered for
physical delivery, read back and checked against their identities. Admission
pages remain bounded to 8,191 objects and 4 MiB, collision handling remains
checked, and publication follows successful checked selected-object admission.

| Original create100 diagnostic | Earlier | Later |
| --- | ---: | ---: |
| Selected spill readback | 113,434,447 B | 4,256,258 B |
| Consumer service, authentication slice | 286.477 ms | 179.085 ms |
| Output pipeline, same slice | 411.530 ms | 374.066 ms |

The later sample delivered 109,178,189 bytes memory-owned with zero borrowed
delivery copies. Its complete Commit was 0.636 seconds, but this was **one
historical exploratory sample with no independent benchmark proof**; the full
lifecycle remained 15.893 seconds. It is not a release speedup claim. Shared
output scheduling also does not mean every initialization and Commit route has
converged: ordinary Commit retained one worker, and native fallback and planned
admission retain distinct responsibilities. The final [output driver and checked
consumer](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/crates/layerfs-layerstack-store/src/objects.rs)
preserve these boundaries.

## 5. Payload and content-reuse scaling

The four-family campaign covered payload creation, cross-file CAS reuse, CDC
locality and Workspace unique-content reuse. A failing payload Commit first
revealed that spilled-object reads assumed insertion order while reachability
selection used graph order. Reusing the existing offset index corrected the
failure. Another predicate rejected legitimate multi-piece sequential capture,
discarding canonical content already constructed during writes. Correct capture
reuse and compact contiguous ranges removed repeated work at Commit.

Native initialization had a separate scheduling problem: flat CAS directories
and CDC variants could remain on one producer. Eligible sorted file ranges are
now divided into bounded tasks across existing workers, preserving canonical
ordering and fallback rules. The final heuristic balances file counts, so uneven
file sizes can still leave a producer tail.

Historical payload500 candidate time fell from 3.626 to 2.446 seconds, with
admission falling from 2.386 to 1.232 seconds. A native directory/inode
finalization diagnostic fell from 29.375 to 0.523 milliseconds. The completed
campaign recorded 114 samples and 30 adjacent-tier comparisons in each of two
separate topologies. All passed the amended normalized-growth gate below 1.25;
earlier proportional and tighter-gate misses remain recorded. Host and Docker
resource scopes differ, so their differences are not pure algorithm speedups.
The [diagnosis ledger](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/issue38-progress.md)
and [host qualification](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/issue38-host-store-results.md)
retain those distinctions. CAS and CDC themselves predate this cycle.

## 6. Shared live Workspace execution

The optimization campaign increasingly depended on coherent live state. FUSE
operations, SDK changes, open handles and Commit could not safely optimize their
own copies of that state independently. The [shared-live-Workspace integration,
PR #53](https://github.com/Ephemeral-AI-Lab/layerfs/pull/53), moved live filesystem
semantics into one owner and a reusable Workspace core.

Linux filesystem operations and SDK edits now coordinate through that live
owner. Host responsibilities remain explicit: physical backing, SQLite and
canonical publication stay host-owned. The core works with backing references
and immutable facts rather than owning host storage directly. This structure
lets filesystem operations and SDK edits preserve common inode, namespace and
file-mutation semantics while their transport and persistence responsibilities
remain distinct.

Commit drains filesystem callbacks at an operation boundary while commands,
mounts, current directories and handles can remain live. Writes after that cut
resume as new Workspace work rather than requiring the command to terminate.

The earlier optimizations survived that integration: checkpoint installation
updates live-core records, shared segment lifetimes remain host-owned, and
ordinary construction retains ordered finalization. This is an architectural
achievement with correctness implications; code reuse alone is not evidence of
a latency improvement. The final [live core](https://github.com/Ephemeral-AI-Lab/layerfs/tree/9f5a641d223606c45e5e6aa8a20094c12f9139a1/crates/layerfs-workspace-core/src)
and [host backing adapter](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/crates/layerfs-workspace/src/live_backing.rs)
make that division concrete.

## 7. Authenticated read reuse and kernel caching

Git exposed the cost of repeated dependent acquisition across the host/runtime
boundary. The revised read path reuses authenticated structural objects and
acquires bounded groups of directory facts and small-file content. A charged
daemon cache keys immutable facts by owner scope and relevant roots; delayed
responses must still match namespace identity and revision before installation.
Partial directory pages cannot establish absence outside their validated range.
The host structural cache is bounded to 8 MiB; the daemon's charged shared cache
is bounded to 32 MiB. Small-file prefetch allows at most 8 KiB per file and
512 KiB per grouped response. Recorded backing-request counts fell from 10,704
to 784 for Git-100 and from 24,440 to 1,999 for Git-500.

Eligible immutable read-only opens can prefill already-acquired bytes through
upstream `fuser`'s public `Notifier::store` API. A dedicated worker uses a
zero-capacity try-only handoff: optional work is skipped when busy. Accepted
jobs retain their operation guards until completion, and writable opens,
mutations and SDK reconciliation exclude conflicting prefill.
The final path retains explicit OPEN; read-only descriptors can omit FLUSH,
while writable descriptors preserve their flush path.

| Complete Git lifecycle | Corrected baseline, one run | Final three-run median |
| --- | ---: | ---: |
| Git-100 | 5.827 s | 1.852 s |
| Git-500 | 14.118 s | 4.636 s |

These descriptive improvements are about 3.15× and 3.05× for the measured
workloads. They include Create, cold acquisition, file changes, six Git commands,
LayerFS Commit, visibility and End. The 500/1,000 ms targets remain missed.
Source [#73, `5c9cce92b446ba8d953c30c9df41b4feccb497df`, records the corrected
qualification](https://github.com/Ephemeral-AI-Lab/layerfs/blob/5c9cce92b446ba8d953c30c9df41b4feccb497df/docs/roadmap/0.1/0.1.3/issue68-git-optimization-results.md);
the later checkpoint measured 1.879 and 4.689 seconds. No kernel or dependency
fork was required.

## 8. Writeback coherence and recovery

Live tests found a concrete coherence failure: stale kernel folio writeback could
overwrite bytes installed by an SDK edit. The repair retains the installed view
and protects SDK-written ranges while kernel reconciliation drains. Incoming
stale data is corrected over those ranges while unrelated mapped writes remain
preserved; obsolete tails are clipped after resizing.

Reconciliation must also allow the writeback needed to finish while pausing
ordinary mutations. Treating every operation identically at that boundary can
block the work required to make progress. Recovery repairs addressed failed-owner
Discard and publication-success/presentation-failure handling, preserving a
published result without creating a duplicate Commit.

All three required live Docker tests executed and passed on the final product,
covering concurrent Workspaces, live commands, dirty mappings, ordinary writes
and SDK resize/write coherence. The [checkpoint repair record and retained live
logs](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/README.md)
include the original failure. These results strengthen the tested live-operation
contract; they do not expand crash or power-loss durability guarantees.

## 9. Reusable benchmark preparation

Repeatedly constructing and checking large fixtures made iteration expensive.
The harness now reuses protected prepared inputs while giving each measurement
independent mutable state. Closed SQLite masters produce independent writable
copies; preparation checks preserve starting-state identity and master ownership.
Native imports reuse prepared source files but create fresh output Stores.

Setup, product execution and independent proof have separate timing and identity
records. This matters when interpreting improvement: removing fixture copies or
repeated preparation hashes saves engineering time without making a public
product operation faster. Host CPU and storage scope also remain distinct from
container limits. The [preparation and topology
qualification](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/issue38-host-store-results.md)
records the isolation, reuse and cleanup checks.

Consolidating the benchmark entrypoints and shared machinery also removed a net
10,377 lines in the [infrastructure migration
record](https://github.com/Ephemeral-AI-Lab/layerfs/issues/45#issuecomment-5546984074).
That is a maintenance improvement, separate from product performance.

Benchmark defects were treated as defects. Git qualification repaired copied
root metadata and a proof-routing mistake, preserved the old receipts, then
collected a predeclared corrected cohort. Successful performance observations
were not silently replaced to improve a table.

## 10. Bounded history verification

Repeated-history workloads exposed expensive verification as well as product
costs. The expected-content oracle accumulated recursive Slice/Concat ancestry;
repeated evaluation could revisit an increasingly deep expression graph. For
expected edited files of at most 48 KiB, the verifier now materializes the edited
result with `Vec::splice` and stores a literal, avoiding that recursive ancestry.

Routine history proof checks every Commit identity, parent and final head while
selecting six to eight snapshots for deeper content checks. Coverage and omitted
exhaustive history/object work remain explicit. The five original high-tier
history timeout cases subsequently passed routine proofs in approximately
3.66–9.14 seconds. This is improved verification practicality, not
faster product ancestry lookup or a new production history index. The [final
checkpoint contract](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-74-75.md)
and [per-proof coverage](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md)
define the evidence. Open PRs #36 and #37 are excluded from delivered claims.

## 11. Final benchmark checkpoint

The final inventory covers 17 admitted families: 16 performance families plus
reliability. It records **198/198 performance PASS and 226/226 routine
verification PASS**. These are fixed-seed observations with declared coverage,
not a latency distribution or exhaustive release qualification.

| Selected final checkpoint observation | Time |
| --- | ---: |
| Initialize 100,000-file namespace | 2.603 s |
| Payload create, 500 MiB | 3.068 s |
| Bulk create100, mixed-v3 lifecycle | 0.990 s |
| Bulk delete100, mixed-v3 lifecycle | 0.259 s |
| Git-100 / Git-500 lifecycle | 1.879 s / 4.689 s |

Mixed-v3 bulk workloads affect 1,000 files / 100 MiB at tier100 and differ from
the original 20,000-file recipe. Their times must not be compared as a product
speedup. The original 2.2-second namespace objective and 500/1,000 ms Git targets
remain missed. Unrelated-history-500 measured 18.164 seconds, above its historical
15-second target; collection PASS does not erase those targets. One separate
600-second endurance definition was explicitly excluded and not executed.

The [full checkpoint report](https://github.com/Ephemeral-AI-Lab/layerfs/blob/9f5a641d223606c45e5e6aa8a20094c12f9139a1/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md)
publishes per-case timings, resources, source identities and proof coverage,
including preserved failures and verifier-only requalification. It provides a
concrete basis for release preparation while keeping the remaining publication
and release-policy obligations separate from development completion.

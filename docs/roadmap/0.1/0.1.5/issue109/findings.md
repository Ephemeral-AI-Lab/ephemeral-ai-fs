# Namespace Init slowness findings (#109)

Recorded 2026-09-11 (Asia/Shanghai; raw UTC timestamps are retained).

**Status: investigated; fixes and controlled measurements pending.**
Owner: [#109](https://github.com/Ephemeral-AI-Lab/layerfs/issues/109).
Coordinate shared admission work with [#108](https://github.com/Ephemeral-AI-Lab/layerfs/issues/108), performance tracking with [#106](https://github.com/Ephemeral-AI-Lab/layerfs/issues/106), and qualification with [#102](https://github.com/Ephemeral-AI-Lab/layerfs/issues/102).
The primary goal remains Init <=2.7 s, preferably approximately 2.60 s.
The [combined #108/#109 implementation plan](implementation-plan.md) maps shared
SQL costs, isolated candidates, open questions and proposed transfer checks.
It is a plan only; execution is pending a separate request.
The machine-readable timings, attempt commands, source identities and evidence
hashes are in [measurements.json](measurements.json).

Two independent read-only subagents traced the two stages against the historical product source. A single intrusive macOS stack profile of the exact archived #104 executable corroborates two concrete sources of avoidable work. No product code has been changed or optimization measured in this investigation.

## Clear direction and remaining uncertainty

**Ready now:** implement signature reuse and metadata-index statement reuse as
separate candidates. Their duplicated work is confirmed by source inspection
and profiling; another broad exploratory profile is not a prerequisite.
**Not established:** their actual latency savings, combined benefit, or whether
they restore the <=2.7-second goal while preserving storage and resource behavior.
All candidates still require correctness checks and controlled measurements.

| Priority | Issue | Proposed optimization | Readiness and required experiment |
| --- | --- | --- | --- |
| 1 | Preparation discards a SmallContent signature that FULL publication recomputes. | Retain/reuse the existing bounded 64-byte signature; preserve fallback computation and memory accounting. | Ready to implement. Prove publication reuses the signature without changing lookup/winner behavior; compare pipeline and total Init against an unchanged control. |
| 2 | Metadata index prepares the same INSERT per value and SELECT per lookup. | Prepare INSERT once per existing transaction; reuse cached SELECT. | Ready to implement. Measure final-tree, index sync/lookup and total Init; verify metadata sharing, authentication, reopen and rollback. |
| 3 | Scratch metadata index commits each synchronized group of at most 165 values. | Group compatible synchronization work in a bounded scratch transaction. | Measure remaining cost after priority 2 first. Count groups and scratch commits, then test grouping with cursor/error/invalidation checks. Store commit clocks do not cover these transactions. |
| 4 | FULL compression and candidate-triggered DELTA trials may provide little benefit for this pseudorandom fixture. | Reduce provably unnecessary encoding/trial work only after quantifying its benefit. | Exploratory evidence required: FULL/DELTA selections, trial success rate, codec time, encoded bytes saved and complete allocated Store size. Any encoding-policy change is an explicit treatment, not a silent benchmark shortcut. |
| 5 | Candidate hits can add locator queries, authenticated base decoding and codec teardown/setup. | Reuse already available facts or batch compatible calls within existing authentication and memory bounds. | Profile/count the remaining calls after earlier fixes. Implement only demonstrated reuse; do not add speculative caches or allow decoder/encoder memory to overlap beyond the budget. |
| 6 | Four producers feed a mostly busy serial consumer and encounter backpressure. | Reassess worker count or bounded parallel preparation after reducing serial work. | Reprofile first. A separate worker-count treatment must show latency benefit with consumer idle, producer blocked time, CPU and memory evidence. More producers are not currently justified as the first fix. |

Priority order follows evidence, expected effort and correctness risk, not a
predicted number of seconds saved. Once the first two fixes are measured, follow
the remaining dominant cost rather than mechanically implementing every item.
The 23.6% redundant signature share and 76.8% metadata-index share are stack
observations; the latter includes necessary execution/I/O. Neither is a promised
wall-time reduction.

### Questions that the next experiments must answer

| What is not clear | Evidence needed to resolve it |
| --- | --- |
| Actual benefit of signature reuse | Unchanged-control versus signature-only candidate operation/pipeline clocks; signature computation/reuse counts or equivalent focused evidence; memory and selection correctness. |
| SQL preparation versus necessary lookup, execution and I/O cost | Statement-only candidate final-tree/index timings, lookup counts and residual profile if needed. Keep scratch commits separate from authoritative Store commits. |
| Whether scratch transaction grouping is both useful and safe | Remaining scratch commit count/time after statement reuse, then a separate grouped candidate with cursor/error-state and invalidation checks. |
| Delta/compression benefit on this exact corpus | Actual FULL/DELTA distribution, trial costs, encoded savings and allocated Store bytes. Pseudorandom input suggests low opportunity but is not a measured selection result. |
| Whether combined fixes recover v0.1.3-level performance | Fresh current-source control and combined candidate with matched harness, fixture, flags, cache policy and bounded ordered repetitions; report operation and command wall, all attempts, median/range and correctness. |
| Whether improvements transfer beyond the selected case | All four registered Init tiers and independent proofs after fixes stabilize, plus shared admission/publication callers affected by the actual changes, coordinated with #108. |

The current instrumented observation is 4.702218500 s. Reaching 2.7 s requires
2.002218500 s (42.58%) less latency relative to that observation. This is target
gap arithmetic, not a prediction or a valid substitute for a fresh control. The
historical 2.603162083-second campaign sample and 2.537925917-second historical
product diagnostic retain their different harness/custody and unpaired scope.

### Bounded execution order

```text
Freeze current source, compatibility and measurement protocol
  |
  +--> unchanged control <-> A: signature reuse only -> check + measure
  |
  +--> unchanged control <-> B: SQL statement reuse only -> check + measure
                                  |
                                  v
                        combine validated A + B
                                  |
                                  v
                         measure remaining cost
                                  |
                 +----------------+----------------+
                 |                                 |
          target reached                    target still missed
                 |                                 |
                 v                                 v
       qualify all Init tiers          investigate remaining dominant cost
       and affected shared callers      among priorities 3 through 6
```

Before the first build/edit/timed execution, freeze exact source arms, a small
repetition count and alternating arm order, fixture acquisition/preconditioning,
cache policy, output locations, timing boundaries and acceptance checks. The
execution protocol has not yet been frozen or run; this is the implementation
order. Use the existing measurement lock and fresh independent Stores, retaining
every attempt. Keep profiler runs separate from clean timing. Add only the
missing counters needed to distinguish a tested mechanism; reuse existing hooks.

An improvement is retained only with correct behavior, measured benefit and
reported storage/memory tradeoffs. Do not weaken FULL/DELTA eligibility,
authentication, publication, complete workload, benchmark compiler flags or
verifier coverage to obtain a faster number. Source custody must distinguish
any separately pending compaction-removal changes. No broad campaign rerun is
needed for this report-only update.

## Recorded operation and command boundaries

Case: `init_namespace / namespace-100000`, 100,000 files, 1,000 data
directories, 500,000,000 decimal bytes. `layerstack_init_ns` surrounds the public
`Client::initialize_layerstack` call through its acknowledgement. Fixture
preparation, Store/client setup, monitoring, canonical accounting and teardown
are outside that clock. SQLite and SDK execution are on the macOS host.

| Observation | Historical v0.1.3 product | v0.1.5 #104 product |
| --- | ---: | ---: |
| Original archived Init | 2.603162083 s | 4.840907708 s |
| New plain diagnostic Init | 2.776088125 s | 4.957007833 s |
| New instrumented diagnostic Init | 2.537925917 s | 4.702218500 s |
| New instrumented complete command wall | 2.995606583 s | 5.037236250 s |

One sample per cell; median and min/max equal that single value. Do not pool
plain, instrumented, profiled or original-campaign observations. No
instrumentation-overhead estimate follows from their differences.

The original historical executable was not recovered. The new historical
diagnostic used a retained later harness with exactly matching product-source
seal; it is not a decomposition of the exact old 2.603-second observation.
The v0.1.5 diagnostic uses the exact archived #104 executable. Neither arm
measures the later, separately pending compaction-removal patch.

## Fixture distribution and hybrid storage routes

A read-only inventory of the retained fixture verified these actual sizes.
There are 1,000 data directories with 100 regular files each. All MB are decimal.

| Fixture class | Files | Size per file, bytes | Total bytes | Share of all bytes |
| --- | ---: | ---: | ---: | ---: |
| Empty | 1,000 | 0 | 0 | 0% |
| Tiny | 78,998 | 13-95 | 4,250,404 | 0.85% |
| Small | 15,000 | 377-3,005 | 25,363,467 | 5.07% |
| Medium | 5,000 | 12,018-96,136 | 270,386,129 | 54.08% |
| Large fixture anchors | 2 | 100,000,000 | 200,000,000 | 40% |
| Total | 100,000 | | 500,000,000 | 100% |

The fixture's category labels are not product thresholds. SmallContent accepts
non-empty files strictly below 131,072 bytes (128 KiB), so all 98,998 tiny,
small and medium files use that route: 300 MB, or 60% of the bytes. The two
100-MB files use the large-file CDC/extents route; empty files are separate.
The 5,000 medium files carry 90.13% of all SmallContent bytes, whereas the
78,998 tiny files emphasize per-file and metadata overhead.

```text
100,000 files / 500 MB
  +-- 98,998 non-empty files <128 KiB: SmallContent FULL/eligible DELTA, 300 MB
  +-- 2 files of 100 MB: CDC chunks and extents, 200 MB
  +-- 1,000 empty files: empty representation and namespace metadata

Both content routes retain exact CAS identity/authentication and packed storage.
SmallContent eligibility does not imply DELTA selection.
```

The generator seeds deterministic pseudorandom content separately per file;
this is not a corpus deliberately composed of related file versions. The exact
compression ratio and FULL/DELTA selection distribution remain unmeasured in
these Init diagnostics. The fixture's "anchor" label does not mean those large
files are SmallContent delta bases. The read-only inventory and its original
hash are retained in [measurements.json](measurements.json).

## Clean clocks already recorded

| Stage | Historical product diagnostic | v0.1.5 diagnostic |
| --- | ---: | ---: |
| Parallel import/admission | 2.380754875 s | 3.449673500 s |
| Final root/inode construction | 0.138150625 s | 1.185147250 s |
| Complete public Init | 2.537925917 s | 4.702218500 s |

These remain separate unpaired n=1 diagnostics, with different historical harnesses and uncontrolled reused OS-cache input. The new profile is not a replacement for these clean measurements.

```text
Public Init                         historical        v0.1.5
  |
  +-- import setup/handoff            0.010 s          0.006 s
  +-- parallel import/admission       2.381 s          3.450 s  <-- +1.069 s
  +-- final root/inode construction   0.138 s          1.185 s  <-- +1.047 s
  +-- publication/outer remainder     0.009 s          0.061 s
  v
Return                                2.538 s          4.702 s
```

The four components balance exactly before display rounding. Setup/handoff is
`prepare_import - pipeline - final_tree`; the outer remainder is
`Init - prepare_import`. Producer and SQL clocks overlap these stages and must
not be added to them. v0.1.5's unpopulated `sql_prepare_ns` and
`sql_bind_step_returning_ns` are unavailable, not zero SQL cost.

## 1. Pipeline: SmallContent signatures scanned twice

The serial admission consumer computes the SmallContent candidate signature in `PreparedAdmission::prepare_small`, uses it for candidate lookup, discards it, then recomputes it from the same bytes in `publish_inner` when adding a selected FULL winner to the candidate index.

The signature algorithm scans overlapping 16-byte windows and keeps the eight smallest distinct mixed hashes. This is substantial CPU work over the source content, not a cheap object-ID lookup. Historical admission inserted canonical blobs directly; this physical candidate-search work was added later. Commit `45ec8918cd9da1899bed5181c5f5dc334b6f3d07` introduced the matching and both call sites.

| Disjoint pipeline main-thread stack branch | Samples | Share of 2,903 pipeline samples |
| --- | ---: | ---: |
| Signature during preparation | 685 | 23.60% |
| Signature recomputed during publication | 686 | 23.63% |
| Both signature scans | 1,371 | 47.23% |
| Store commit through begin_batch | 598 | 20.60% |
| PreparedAdmission::insert | 418 | 14.40% |

```text
producer 1 --+
producer 2 --+--> bounded output queue --> one admission consumer
producer 3 --+             |                        |
producer 4 --+             +-- full: workers wait <--+

Consumer path:
  SmallContent bytes
    -> scan 16-byte windows; compute similarity signature
    -> candidate lookup / physical encoding
    -> discard computed signature                    <-- repeated work
    -> physical admission and publication
    -> selected FULL winner: scan the SAME bytes AGAIN
    -> candidate-index insertion

Candidate fix:
  compute signature -> lookup
          |
          +-> retain bounded 64-byte signature -> reuse at publication
```

The v0.1.5 clean pipeline took 3.450 s; consumer idle waiting for output was
62.683 ms. Producer blocked time totaled 4.684621 thread-seconds across four
workers, not additional operation wall time. The queue's observed peak was four.

The two signature branches are disjoint; their parent stacks must not be added again. Sample counts are not phase milliseconds. Eliminating the redundant scan does not eliminate the first required scan, and no latency saving is claimed yet.

Source at the measured v0.1.5 checkpoint:

- [prepare_small signature](https://github.com/Ephemeral-AI-Lab/layerfs/blob/84eaa5b619fb015dbc5b0175836a2e5451461328/crates/layerfs-layerstack-store/src/objects/admission.rs#L253)
- [publish_inner recomputation](https://github.com/Ephemeral-AI-Lab/layerfs/blob/84eaa5b619fb015dbc5b0175836a2e5451461328/crates/layerfs-layerstack-store/src/objects/admission.rs#L1015)
- [signature algorithm](https://github.com/Ephemeral-AI-Lab/layerfs/blob/84eaa5b619fb015dbc5b0175836a2e5451461328/crates/layerfs-layerstack-store/src/objects/small_candidates.rs#L39)

Smallest candidate: carry the already computed 64-byte signature with the bounded prepared object and reuse it for publication. Preserve the algorithm, lookup time/order, candidate selection, eviction, and publication-only visibility for actual FULL winners. Explicit-predecessor paths that did not compute a lookup signature must still compute one if publication needs it. Account for the added struct/Option/vector capacity in the existing memory ledgers; do not retain another payload copy. Existing tests cover selected candidates, rollback, late CAS, prepared-only invisibility, eviction, retained handoff and reopen.

This shared admission path serves Init and other publication operations. Increasing producer count is not the first justified correction: the consumer is already busy and producers experience backpressure.

## 2. Final inode construction: derived metadata index SQL work

The final compact inode builder streams its leaves through physical admission. The profile locates most of this stage in metadata pooling's disposable SQLite index, not sorting or directory construction.

| Profile subtree | Samples |
| --- | ---: |
| compact_inode_table_from_sorted | 1,030 |
| Persistence through direct writer and checked admission | 976 |
| prepare_ordinary to metadata pooling | 860 |
| ValueIndex::sync, all branches | 416 |
| ValueIndex::find, all branches | 375 |
| Inode serialization/hash iterator | 32 |

Most rows are nested and MUST NOT be summed. The disjoint sync/find branches total 791/1,030 (76.80%) of compact-table samples.

Confirmed repeated work:

- `ValueIndex::sync` opens/commits one scratch transaction per metadata group of at most 165 values and prepares the same INSERT separately for each value through `transaction.execute`.
- `ValueIndex::find` prepares the same SELECT again for each lookup through `Connection::query_row`.
- `prepare_values` synchronizes this index and looks up every value absent from the publication-local pending map.

Source:

- [prepare_values](https://github.com/Ephemeral-AI-Lab/layerfs/blob/84eaa5b619fb015dbc5b0175836a2e5451461328/crates/layerfs-layerstack-store/src/objects/admission/metadata_values.rs#L46)
- [sync transactions and per-value INSERT preparation](https://github.com/Ephemeral-AI-Lab/layerfs/blob/84eaa5b619fb015dbc5b0175836a2e5451461328/crates/layerfs-layerstack-store/src/objects/metadata.rs#L102)
- [per-value SELECT preparation](https://github.com/Ephemeral-AI-Lab/layerfs/blob/84eaa5b619fb015dbc5b0175836a2e5451461328/crates/layerfs-layerstack-store/src/objects/metadata.rs#L133)

```text
Compact inode builder
  -> emitted leaf -> physical metadata admission
       |
       +-> synchronize derived metadata index
       |     for each missing group (at most 165 values):
       |       BEGIN scratch transaction
       |       for each value:
       |         PREPARE identical INSERT -> execute -> discard statement
       |       COMMIT -> page writes
       |
       +-> for each value absent from publication-local pending map:
             PREPARE identical SELECT -> lookup -> discard statement

All of these are local SQLite calls, not network round trips.

Two separate transaction domains:
  authoritative Store admission COMMITs -> counted by final-build commit clock
  temporary metadata-index COMMITs     -> NOT counted by that clock
```

**Correction to the preceding diagnostic interpretation:** the 9.671 ms final-build commit counter covers Store admission commits, not all commits performed during final-tree construction. The separate scratch metadata-index transactions are excluded. The profile contains 151 samples at scratch commit within ValueIndex::sync, including 130 at guarded_pwrite_np. Thus transaction/I/O overhead does contribute to this stage; the prior counter alone could not rule it out.

Smallest first candidate: prepare the INSERT once per existing sync transaction and reuse a cached SELECT in `find`. Preserve identical SQL, lookup ordering, first-ordinal INSERT OR IGNORE behavior, authentication, eviction bounds, scratch limits and rollback invalidation. Transaction coalescing should be a separate experiment because scratch uses journal/synchronous OFF and cursor/entries state must remain valid on errors. Do not remove pooling or change storage formats.

The fixture's 1,000 root directories remain worker tasks; only their root parent is constructed serially at the end. The profile has one main-thread sample in final directory construction. Moving directory work to the serial stage is not supported as the cause here.

## Evidence and next bounded implementation experiments

Exact executable SHA256: `60caa06fd67f0883148bc49c0f320a1f75cbec7fb5ba2770c9c82fc356203c67`, matching #104; product seal `b1a94e2223b1cd6c0eedffa3c6c60eca7134727c45b9018e1cea518cdf6d3dd5`. Source and build identities are retained in the previous v015-clock evidence. Same 100,000-file / 500,000,000-byte fixture, digest `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.

New profile evidence:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue109-evidence/profile-corrected-20260910T173335Z`

- `sample.txt`: pipeline branches at lines39 and714; final inode subtree at1222.
- `init.log`, `result.json`, `internal-clocks.json`: actual profile-run clocks and full scan receipt.
- `run.py`, `declaration.md`, `attempts.jsonl`: prospective protocol, absolute argv, timestamps and exits.
- `/usr/bin/sample PID 6 1 -mayDie -file sample.txt` ran with the existing Init nonce hooks. The successful profiled Init took5.734524166s, pipeline4.328713375s, final-tree1.313786667s. These include profiling overhead and are not clean performance results.
- An initial launch failed cache-profile argument validation before initialization. It is retained in sibling `profile-20260910T173247Z`; the corrected supported argument was declared before the one successful profile run. No valid sample was discarded or selected for speed.
- Parent owned the existing runner measurement lock; no other resource-sensitive run overlapped. Both subagents performed read-only analysis. No source changes/builds, compaction, or independent verifier run occurred. Diagnostic scan counts pass; admission_eligible=false.
- Historical and clean v0.1.5 evidence manifests were rechecked successfully. The investigation did not alter the unrelated compaction-removal work or sealed #104 campaign.

The prioritized implementation order and evidence needed for each open question
are recorded above under [Clear direction and remaining uncertainty](#clear-direction-and-remaining-uncertainty).
No optimization has been measured yet; #109 remains open.


## Correctness and qualification still required

Signature reuse must retain selected-candidate ordering, explicit-predecessor
fallback, late-CAS handling, rollback, eviction, prepared-only invisibility,
retained handoff and cold reopen. Account for actual struct/vector capacity,
including the mirrored prepared-object layout check, within existing bounds.
Metadata statement reuse must retain first-ordinal deduplication, group
authentication, scratch limits and rollback/cache invalidation. Transaction
coalescing requires a separate cursor/error-state design.

Freeze current-source control and each candidate before measuring. Keep
benchmark flags, full fixture, public operation and uncompacted treatment.
Use the existing runner lock, fresh independent Stores and retained attempts.
After selected-case improvement, qualify all four registered Init tiers and
independent proofs, plus shared callers affected by the actual change. No
successful #104 cell is rerun solely because this documentation changes a source
seal. This findings report does not claim a fix, release admission, or issue
completion.

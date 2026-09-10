# Namespace Init slowness findings (#109)

Recorded 2026-09-11 (Asia/Shanghai; raw UTC timestamps are retained).

**Status: investigated; fixes and controlled measurements pending.**
Owner: [#109](https://github.com/Ephemeral-AI-Lab/layerfs/issues/109).
Coordinate shared admission work with [#108](https://github.com/Ephemeral-AI-Lab/layerfs/issues/108), performance tracking with [#106](https://github.com/Ephemeral-AI-Lab/layerfs/issues/106), and qualification with [#102](https://github.com/Ephemeral-AI-Lab/layerfs/issues/102).
The primary goal remains Init <=2.7 s, preferably approximately 2.60 s.
The machine-readable timings, attempt commands, source identities and evidence
hashes are in [measurements.json](measurements.json).

Two independent read-only subagents traced the two stages against the historical product source. A single intrusive macOS stack profile of the exact archived #104 executable corroborates two concrete sources of avoidable work. No product code has been changed or optimization measured in this investigation.

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

Next implementation should isolate the two candidates: (A) reuse SmallContent signatures, (B) reuse metadata-index SQL statements. Freeze fresh current-source control/candidate identities and a small sample order before each experiment, use fresh Stores and the existing fixture, check shared-call correctness, and measure existing phase clocks. Add only missing signature-reuse and index lookup/sync accounting needed to distinguish the mechanisms. A combined candidate can follow their individual validation. Remaining scratch transaction coalescing is secondary. No speedup is promised from stack shares; #109 remains open pending implementation, measured improvement, and qualification.


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

# Shared metadata index attribution — measured findings

The study confirms two distinct costs:

1. **Cold Init spends most index time executing SQLite statements, not binding
   parameters.** Whole-Init medians: INSERT execution 187.640 ms versus binding
   3.813 ms; lookup execution122.807 ms versus binding 3.473 ms. The scratch index
   shows substantial cache misses and spills within its unchanged 4 MiB cache.
2. **The first one-file Commit after reopening a 100000-file Store reconstructs
   100002 metadata values.** It takes 244.064–273.890 ms, versus 16.087–28.256 ms
   for the retained Store. Synchronization explains almost all of the difference.
   The next Commit synchronizes exactly one value in every tested cell.

These are instrumented diagnostic findings, not a new product improvement or
acceptance result. No storage optimization was implemented in this stage.

## Protocol, custody and diagnostic limits

Followed [the committed protocol](index-attribution-contract.md), 808817edb.
Isolated source: current product including the bounded endpoint query, plus the
preserved dirty compaction-removal work. Main-workspace product code was not
edited. Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-index-attribution-evidence/20260911T074548Z`.

Added private diagnostic counters and a nonce-only harness in that worktree:
index creations, replay groups/values/evictions, INSERT bind/execute and VM work,
lookup prepare/bind/step/extraction and VM work, per-index SQLite cache counters.
Used raw binding APIs to separate binding from execution without changing values,
SQL, statement reuse, ordering or transaction boundaries. 15 focused metadata tests
passed, including sharing, reopen, rollback, corruption and intermediate-base
authentication. The private cache-status helper uses narrowly scoped SQLite FFI
on the borrowed, owned connection; none of this instrumentation is promoted.

Fine-grained clocks add overhead, especially once per inserted value. Their time
is included in the enclosing operation; no speculative overhead subtraction was
performed. These absolute times must not be compared with older plain or less
instrumented samples as a product speedup. SQLite execution includes its B-tree,
constraint, allocation, cache-management and I/O work; this study does not split
those internal costs into pure CPU versus physical device latency.

Qualified host and matching Linux image builds passed. Builds/tests/collection
were serialized under the runner-owned measurement lock; no double acquisition.
The full fixed collection completed: 2 cold Init runs,16 Store-lifetime cells,
32 successful public Commit calls. No failed sample, replacement or timing-based
retry. Every command, raw log, exit, fixture reference and source identity remains.

## New Store: full cold namespace-100000 Init

Both independent acquisitions validated 100000 files/500000000 bytes, 125169
pages and zero resident pages; source allocation 865730560 bytes. Each launch
followed acquisition within the fixed allowance. Both report
`fixture_cache_profile=reused-first-sample-uncontrolled`; qualification is false
because these are nonce diagnostics. Cold plain <=2.7 s remains the absolute gate.

| Sample | Init ns | Pipeline ns | Final-tree ns | Import remainder ns | Outer remainder ns | initialization_disk_read_bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 3,440,627,667 | 2,775,137,958 | 569,738,791 | 57,749,668 | 38,001,250 | 866,279,424 |
| 2 | 3,655,285,000 | 2,971,993,083 | 592,299,167 | 55,108,083 | 35,884,667 | 874,536,960 |

n=2, median Init 3.547956334 s, range 3.440627667–3.655285000 s. Both preserve
112451 canonical objects/513026835 canonical bytes and 100002 admitted metadata
values. This cohort supplies phase attribution, not an acceptance pass.

The following counters cover **whole Init**, not only final construction:

| Work | Median ms | Count in each run |
| --- | ---: | --- |
| Metadata preparation, enclosing interval | 476.166 | 80 preparations |
| ValueIndex synchronization, nested | 247.522 | 648 groups /98,557 values |
| INSERT loop, nested within synchronization | 197.574 | 98,557 executions |
| INSERT parameter binding, nested within loop | 3.813 | Two parameters per execution |
| INSERT execution, nested within loop | 187.640 | 2,168,254 SQLite VM steps |
| Group read/authentication/decode, inside sync | 20.509 | 648 groups |
| Scratch COMMIT, inside sync | 27.659 | Existing sync transaction boundaries |
| Catalogue work, inside sync | 1.522 | Existing bounded endpoint/group lookups |
| Value lookup, enclosing interval | 133.719 | 100,072 inputs /70 hits |
| SQL construction/prepared statement acquisition, inside lookup | 6.072 | 232 queries |
| Lookup parameter binding, inside lookup | 3.473 | Existing bounded/padded IN lists |
| Lookup execution, inside lookup | 122.807 | 740,719 SQLite VM steps |
| Result extraction, inside lookup | 0.047 | 70 matched values |

Enclosing and nested rows must not be added together. INSERT execution amortizes
to about 1.90 microseconds/value; binding to 0.039 microseconds/value. Lookup
execution amortizes to 1.23 microseconds/input value, not per SQL query.

Other whole-Init metadata preparation intervals remain: ordinal assignment/
physical-leaf rewrite 27.141 ms, metadata group encoding 26.531 ms, and encoded-
group decode/digest 16.775 ms. Their purposes and integrity guarantees remain.

Scratch index cache counts were identical in both cold runs:

| Scope | Cache hits | Cache misses | Cache writes | Cache spills |
| --- | ---: | ---: | ---: | ---: |
| Synchronization | 326,650 | 24,546 | 34,283 | 21,295 |
| Lookup | 179,990 | 18,373 | 0 | 0 |

These establish SQLite cache churn, not physical-disk read/write totals. A cache
miss can be served from the OS cache; a cache write is not necessarily a device
write during the timer. They do not justify increasing the 4 MiB cache allowance
without a separate resource/benefit experiment. They do explain why further
binding-only changes target very little measured work. The rejected INSERT
batching experiment removed executions but retained much of this execution work.

## Existing Store: one-file public SDK edit and Commit

At each registered tier, ran R1,O1,O2,R2 with independent fresh Stores initialized
from the same complete immutable fixture. R retains the original Store/Client;
O drops every owner and reconnects before workspace creation. Reopened means a
new derived index; **OS cache state is uncontrolled, not independently cold**.

Both modes create the same FUSE workspace, then use public
`Client::edit_workspace_file_range` to replace 10 bytes in d0000/f000000 at the
fixture's edit offset. The marker is I000000001, then I000000002 for the separate
second-Commit cohort. This is an SDK edit contract, not the historical normalized-
mtime POSIX namespace-edit contract. Public `commit_workspace_session` alone is
timed; visibility, file verification and cleanup occur afterwards.

Workspace creation and both edits created/synchronized **zero** index values in
all 16 cells. Thus the replay cost was inside Commit, not hidden in setup.
Every first/second Commit admitted exactly one new pooled metadata value.

| Namespace files | R first Commit ms, median [range] | O first Commit ms, median [range] | R second Commit median ms | O second Commit median ms |
| --- | --- | --- | ---: | ---: |
| 100 | 5.217 [4.939,5.495] | 5.093 [4.961,5.225] | 3.742 | 4.072 |
| 1,000 | 9.679 [9.142,10.217] | 9.890 [8.819,10.961] | 6.645 | 4.943 |
| 10,000 | 16.269 [12.913,19.625] | 30.322 [29.515,31.129] | 7.563 | 8.037 |
| 100,000 | 22.171 [16.087,28.256] | 258.977 [244.064,273.890] | 10.465 | 8.457 |

n=2 for every table cell. Both independent O−R first-Commit differences at 100000
files are positive: 245.633 ms and 227.977 ms. This is a Store-lifetime comparison
on one instrumented product, not a candidate/control optimization claim.

| Namespace files | R first synchronized values, repetitions 1/2 | O first synchronized values | O first sync median ms |
| --- | --- | ---: | ---: |
| 100 | 102 /102 | 102 | 0.176 |
| 1,000 | 1,002 /1,002 | 1,002 | 1.211 |
| 10,000 | 350 /4,121 | 10,002 | 14.508 |
| 100,000 | 2,040 /200 | 100,002 | 243.346 |

O creates one new index during first Commit; R creates none. After that, all 32
first/second Commit observations follow the expected ownership transitions:
each of the 16 second Commits synchronizes exactly **one** value and creates no
index. No retention-window eviction occurred in this cohort.

Retained does not mean fully populated: at the two smallest tiers, Init leaves
all values for the next synchronization. At larger tiers a pending tail remains,
and batching affects its size. This explains why the retained first Commit is
not equivalent to a fully caught-up second Commit.

At 100000 files, first-Commit synchronization took 258.695/227.997 ms in O versus
12.971/1.504 ms in R. After subtracting that measured interval, the rest of Commit
was 15.194/16.067 ms in O versus 15.285/14.583 ms in R. The extra historical index
reconstruction is therefore directly localized, not merely inferred from total
latency. O replay alone spent 218.358/189.101 ms executing INSERTs, 3.915/3.764 ms
binding, 24.422–25.397 ms reading groups (raw receipts give exact values), and
about 1.5 ms in the scratch COMMIT.

This confirms linear growth in **replayed value counts** at fixed one-file change.
It is not a fitted universal wall-time complexity claim. We did not test long
histories beyond 131072 retained values, arbitrary K, or multiple historical DELTA
depths. The second cohort contains one prior edit, not a long-history simulation.

## Correctness, resources and remaining direction

All 32 calls returned Created. After each Commit, visible-head checks and complete
canonical bytes of the edited file passed. Each cell ended its workspace cleanly,
then reopened the Store and verified the final root/content. All 16 containers
were removed successfully. The full immutable input was scanned at every bootstrap;
the post-Commit proof is exhaustive for the edited file, not the entire namespace.

Host Store/SDK/spool/publication remain on macOS. Docker runs daemon/FUSE/workload
only, with 2 CPUs/2 GiB per container; host CPU is uncapped. Cold Init peak RSS
was 82,919,424/85,803,008 bytes. Observed stage-end RSS across Commit cells ranged
23,920,640–87,949,312 bytes; these are snapshots, not exact per-operation peaks.
The existing 4 KiB pages, 4 MiB scratch cache and format/work budgets were unchanged.

**Recommended next step:** design and evaluate a restartable exact metadata index
that avoids replaying unrelated history on first use, while preserving sharing,
authentication, rollback and existing memory bounds. Its validity check must not
itself scan all history. Measure additional disk/write cost; a global in-memory
map or a smaller deduplication window is not an equivalent free replacement.

For cold Init, the separate remaining problem is the index's execution/cache
working set. Binding and result extraction are too small to lead. A compact
index representation or other locality improvement needs its own collision/
equality proof and bounded experiment; this study does not select or implement
one. Authentication removal is not supported by these measurements.

## Seals and retained artifacts

- Instrumented product: `3c9b3cb7794504b8dc4a8efac72c81e76e2f11d804aac27b150e071af0daa644`
- Source: `e921511d9c4a6e9bf9e420b493d294c9c2fde2f5b6ee950e7ac2c98daea4b02b`
- Binary SHA-256: `ce9060d6465c45349adf5df402b1b53514329297664d71c52a11ff414e1e45e5`
- Image: `layerfs-bench-infra:e921511d9c4a6e9b`
- Image identity: `sha256:ceeee9a329f0f2ecb610db0863a6b4550df2fe90c8488c24bbc0be8301ec28d9`
- namespace-100000 fixture:
  `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`

All tier fixture references/digests, source-before/after identities, patch,
harness, build logs, stage receipts, container cleanup, raw cold acquisitions,
`analyze.py` and `summary.json` are retained. The analyzer verifies cold residency,
full Init canonical counts, phase equations, nested clocks, replay/binding counts,
index ownership, one-value follow-up synchronization and source custody.
Unused inherited diagnostic fields are not interpreted as measured zero costs.

All samples remain nonce, qualification_eligible=false. No warm/cold pooling,
performance percentage, release or deployment. #111 remains open; context:
#115/#109/#110/#108/#106/#102/#104/#100/#107.

Issue outcome: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5631365034

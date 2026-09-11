# Metadata preparation study after #115

Read [#115's post-closure assessment](https://github.com/Ephemeral-AI-Lab/layerfs/issues/115#issuecomment-5630420769)
and compared the current shared admission path with the retained
[final-tree RCA](final-tree-rca-results.md). This study makes no product change
or performance claim. Keep 4 KiB, schema10, exact equality, pooling, compression,
DELTA, authentication and existing resource limits.

## What transfers from #115

The useful lesson is to measure frequency and per-call cost before modifying a
path. #115 reports zero attributable improvement. Its metadata-read and BLOB
experiments did not recover the expected time. It reports mixed encoding at
4.0% of FULL where a base exists (472 groups); that is a strong reason to preserve
the useful DELTA policy while investigating execution overhead.

Its ~85% search/matching/encode finding concerns object admission on the
distributed-history workload. It is not a measurement of Init's ValueIndex or
proof that metadata pooling cannot be improved. The unresolved ~1.24 ms/op
DeltaSearch question is a separate experiment. Do not combine it with this one
or extrapolate Init improvements to 500-commit history timing.

## Existing evidence, normalized by work

Recomputed directly from the two retained `v015-*/result.json` receipts under
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-final-tree-rca-evidence/20260911T045909Z`.
These remain nonce diagnostics, not qualification.

| Final-tree operation | Sample 1 count | Sample 2 count | Amortized microseconds per item, sample 1 / 2 | Median total ms |
| --- | ---: | ---: | ---: | ---: |
| Scratch INSERT loop, values | 86,976 | 86,082 | 1.792 / 2.126 | 169.402 |
| Batched lookup, input values | 93,270 | 92,381 | 1.376 / 1.454 | 131.333 |
| Existing group read/authentication/decode | 572 | 566 | 31.464 / 34.651 | 18.805 |

Lookup costs are amortized per input value, **not per SQL query**. Existing
receipts do not count lookup statement executions or split binding, stepping,
SQL-text construction and result extraction. Do not invent that attribution.
Lookup hits were only 63/65 (0.0675%/0.0704%). Different group composition and
cache conditions prevent transferring #115's 3.37 microseconds/read here.

The two SQL intervals total 300.735 ms of final construction, or 368.765 ms
across whole Init. These totals are upper bounds on work a hypothetical complete
elimination could remove, not expected gains. Metadata preparation as a whole
was 439.392 ms in final construction. Improving it alone does not establish
that the cold Init operation can reach 2.7 s.

## What the code already does

`objects/admission/metadata_values.rs::prepare_values` synchronizes the
Store-scoped derived index, deduplicates lookups against pending values, calls
`find_batch`, assigns ordinals in first-encounter order, rewrites physical
leaves, encodes value groups and authenticates their decoded representation.

`objects/metadata.rs::ValueIndex::sync` already uses one scratch transaction
per synchronization and one prepared INSERT statement. It still executes that
statement once per value. `find_batch` already queries up to 512 values per
IN-list page. These are the accepted #109 B/S changes; proposing statement
reuse or batched SELECTs again would repeat completed work.

The scratch database is already journal_mode=OFF, synchronous=OFF,
cache_size=-4096, cache_spill=ON, mmap_size=0, and locking_mode=EXCLUSIVE.
Scratch COMMIT's 24.834 ms therefore cannot simply be called fsync latency.
Authoritative Store durability is separate. There is no justification here
for changing its PRAGMAs or page size.

The index preserves complete 73-byte keys, existing ordinals and a bounded
131,072-value retention window. Eviction happens before a whole metadata
group, and rollback clears the derived index. A replacement must preserve
those semantics, including duplicate-value first-winner behavior.

## First experiment: multi-row scratch INSERT, one change

Hypothesis: amortizing repeated bind/execute/reset overhead reduces the
169.402 ms INSERT interval without changing exact lookup or physical format.
This is plausible from frequency and existing code, but unproven: all B-tree
insertions remain and parameter bindings still scale with values.

Use the existing bounded multi-row INSERT pattern in `objects/admission.rs`.
Apply it only to `ValueIndex::sync`: insert the already decoded group's
value/ordinal pairs with one bounded `INSERT OR IGNORE ... VALUES ...` statement
per chunk, respecting the connection's parameter limit (two per row). Groups
are at most 165 values. With room for 330 parameters, final-phase executions
could fall from 86,976/86,082 to 572/566, with the same number of inserted values.
Count actual statements in the diagnostic; do not report these conditional
counts as measured results.

Keep original pair order, group-boundary eviction, transaction boundary,
entry/cursor accounting and rollback invalidation. Reuse statement shapes;
do not add a generic batching framework or a new dependency. Do not bundle
lookup changes, sorting, new caches, worker changes or encode changes.

Measure total INSERT wall, executions, rows and scratch cache misses/spills
where available. Accept the hypothesis only if the measured interval and
whole-operation paired timings improve beyond control variation. If they do
not, revert it and investigate B-tree/cache behavior before another edit.

## Conditional second question: why do mostly missed lookups cost 131 ms?

Only after the first result, split `find_batch` into SQL construction/cache
acquisition, parameter binding, stepping and result extraction, with statement,
input and hit counts. Use the exact bundled SQLite and a retained workload;
do not infer an execution plan or performance from another SQLite build.

An exact negative-membership accelerator might avoid most SQL lookups on this
fixture, but the present evidence does not justify implementing one yet. Any
filter would need a measured size/hit-rate tradeoff, no false negatives for
the retained index, exact SQL confirmation for possible hits, and correct
rebuild/eviction/rollback handling. It must help redundant Commit workloads too.
It is more state and proof burden than the first experiment.

A full in-memory HashMap is not a free substitution. At the current 131,072
entry cap, just 73-byte keys and 4-byte ordinals require 10,092,544 bytes before
alignment, spare capacity and table metadata, compared with the current 4 MiB
SQLite cache allowance plus bounded disk scratch. Do not silently increase RAM
or shrink the deduplication window to obtain a speedup.

## Lower-priority or rejected directions

- Group-read caching or direct insertion of newly admitted decoded values:
  could avoid some reread work, but the measured group-read interval is only
  18.805 ms in final construction and scratch inserts/lookups remain. Direct
  seeding also introduces publication/rollback coordination. Not first.
- Remove DELTA, compression or metadata pooling: changes the storage tradeoff
  rather than improving execution. #115 measured useful DELTA savings on its
  history vehicle; Init's unique corpus does not invalidate them.
- Skip searches whenever Init is running or when data looks random: violates
  shared semantics and exact reuse requirements. No scenario-specific path.
- Reuse an unverified warm fixture or enlarge pages: outside the fixed contract.

## Proof and qualification for an implementation

Before a product experiment, freeze its control/candidate identities, repetition
count, alternating arm order, cold acquisition and invalid-run policy under the
runner lock. Keep diagnostics separate. Cold plain qualification still requires
independently verified acquisition before every arm; warm remains paired-delta
only. Retain every attempt.

The focused correctness check must compare exact value-to-ordinal results across
single/multi-row insertion, duplicate keys, a short final chunk, a lowered SQL
parameter limit, group-boundary eviction, reopen and publication rollback.
Run existing metadata/admission/small-candidate suites; then all four Init tiers
and the independent selected proof on matching candidate binary/image.

Shared Commit coverage must include unchanged workspace, one small first commit
after Store reopen, repeated small commits on a retained Store, a larger changed
set and redundant history with DELTA opportunities. Report canonical and physical
bytes alongside timing so loss of reuse or DELTA savings cannot masquerade as
progress. First-use synchronization remains expected after reopen; batching
would reduce its execution overhead, not remove the need to construct the index.

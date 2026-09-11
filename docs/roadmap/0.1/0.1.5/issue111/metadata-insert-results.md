# Bounded metadata INSERT experiment — rejected for promotion

The bounded multi-row INSERT implementation passed correctness checks and
reduced its execution count sharply. Across whole Init, the diagnostic metadata
preparation median decreased by **37.513 ms**. It did **not** demonstrate a
whole-operation improvement: one pair was 101.540 ms faster and the other
88.259 ms slower. The frozen mechanism screen failed, so the candidate is not
promoted and no plain acceptance or Commit performance qualification is claimed.

This is a measured partial improvement in an internal diagnostic interval,
not proof that batching resolves the metadata bottleneck. Cold <=2.7 s remains
open. No performance percentage is calculated from these nonce samples.

## Protocol and implementation

Followed [the committed contract](metadata-insert-contract.md), b19ebed03, and
the [#115-informed study](metadata-preparation-study.md). Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-metadata-insert-evidence/20260911T065242Z`.
The control is the current d316cb592 product plus the preserved dirty patch;
the subsequent contract commit changes documentation only. Its pre-diagnostic
product seal is `5d7f1feea049846633129a08be605397e86470b9c4f17967e0f4bd4d975e8e4c`.
This is not the older 95e796f8 product; no historical row is pooled into this A/B.

Both isolated worktrees have the same prior nonce phase counters and added
INSERT execution counter. The candidate changes only
`objects/metadata.rs`: a bounded helper emits multi-row INSERT OR IGNORE, with
two parameters per value, at most 165 values per statement and the actual
connection variable limit respected. Values retain original order. The scratch
transaction, exact 73-byte keys, first-winning ordinal, retention-window/group
eviction boundaries, index cursor and rollback invalidation are unchanged.
No schema, Store page size, producer count, lookup, cache or DELTA change.

The candidate INSERT clock includes helper SQL construction, parameter-pair
allocation, cached-statement acquisition and execution; these costs were not
moved outside the operation. The control's already-prepared statement remains
outside its per-group INSERT loop clock, as before. Both enclosing sync/metadata
and whole-operation clocks include their complete respective work.

Added focused tests for duplicates/earliest ordinal, full and tail chunks,
parameter limits 2/6/330, too-small limit rejection, whole-group eviction and
idempotent synchronization. The eviction test places the index entry counter
at the real boundary without allocating 131070 irrelevant entries, then runs
the real group read/sync/lookup path. Existing tests cover sharing, reopen,
authentication, publication failure and rollback.

## Attempts and validation

- The pre-change focused check failed as expected: 165 executions versus the
  expected 55 under a six-parameter limit.
- One implementation compile attempt failed because this rusqlite version does
  not implement ToSql for ValueRef. Reused the existing borrowed-parameter pattern
  instead; that failure and its exit 101 are retained.
- Store library suite: 129 passed, 0 failed, 4 pre-existing ignored tests. The
  subsequently added boundary check and INSERT check both passed (2/2).
- Both arms were built separately with `shared/runner.py --build-host` under
  its measurement lock; both linked schema10/storage-format probes passed.
- Initial control attempt used nonce 111i1, which is not hexadecimal. The public
  operation completed, but no phase receipt was emitted and the collector failed.
  Its raw 3.736587916 s and 875962368 read bytes remain in `invalid-control-1/`.
  It is ineligible for the diagnostic comparison, not discarded for timing.
  The candidate of that pair had not started. Used the one allowed affected-pair
  replacement, correcting only the collector nonce; no source or binary change.
- Completed replacement C1,T1, then original T2,C2. No further replacement,
  warm-up or selected-arm rerun. Every sample used the same shared measurement
  lock and independently acquired cold input before a fresh host Store.

`analyze.py` validates every completed comparison row's cold receipt, binary
identity, source-before/after seals, complete canonical counts, execution counts,
phase sums and nested clock bounds. It reproduces the failed screen from the
frozen criterion. Main-workspace product changes were never applied.

## Cold diagnostic rows

Every row below has `fixture_cache_profile=reused-first-sample-uncontrolled`,
`measurement_mode=init-only-diagnostic`, `qualification_eligible=false`.
Independent acquisition validated 100000 files / 500000000 logical bytes,
125169 pages, zero resident pages and the warm-page backend positive control.
Source allocation was 865730560 bytes. Read count alone is not cold evidence.

| Order | Arm | Init ns | Final-tree ns | Final INSERT ns | INSERT executions / values | initialization_disk_read_bytes |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1 | C1 | 3,824,702,625 | 601,829,542 | 155,413,290 | 85,441 / 85,441 | 875,487,232 |
| 2 | T1 | 3,723,162,458 | 605,130,958 | 150,108,693 | 588 / 89,449 | 874,340,352 |
| 3 | T2 | 3,748,799,542 | 624,473,125 | 156,964,629 | 590 / 89,697 | 866,082,816 |
| 4 | C2 | 3,660,540,167 | 605,499,041 | 160,109,210 | 87,371 / 87,371 | 871,088,128 |

n=2 per arm. Cold plain remains the absolute acceptance gate; warm remains
paired-delta-only. No diagnostic row can pass the 2.7 s target.

| Interval | Control median ms [min,max] | Candidate median ms [min,max] | C1−T1 ms | C2−T2 ms |
| --- | --- | --- | ---: | ---: |
| Whole Init | 3742.621 [3660.540,3824.703] | 3735.981 [3723.162,3748.800] | +101.540 | −88.259 |
| Final-tree | 603.664 [601.830,605.499] | 614.802 [605.131,624.473] | −3.301 | −18.974 |
| Final metadata preparation | 420.469 [416.531,424.406] | 429.465 [421.256,437.673] | −4.725 | −13.267 |
| Final INSERT | 157.761 [155.413,160.109] | 153.537 [150.109,156.965] | +5.305 | +3.145 |
| Whole-Init INSERT | 212.534 [204.361,220.707] | 177.592 [172.047,183.137] | +32.314 | +37.570 |
| Whole-Init metadata preparation | 507.480 [499.413,515.546] | 469.966 [460.222,479.711] | +39.191 | +35.835 |

Positive differences mean the candidate was faster. The whole-Init control
range was 164.162 ms, larger than either paired reduction, with the second pair
negative. Final INSERT's control range was 4.696 ms; its second reduction was
only 3.145 ms. Both required screen conditions therefore fail.

## Interpretation and limits

Batching reduced SQL execution count, but most insertion work remained. Whole
Init executed 97962/99851 scalar INSERTs in control versus 629/631 candidate
statements covering 95684/95932 values. The roughly 35 ms whole-INSERT reduction
is real as an observation of these clocks; it does not justify claiming that
the entire 169 ms previously measured INSERT interval was removable overhead.
Per-value work, B-tree operations, bindings, cache behavior and candidate batching
overhead remain inside the measured path. Their individual shares are unresolved.

Final-phase and whole-Init counters must remain distinct. All arms admitted
100002 pooled metadata values, but the number synchronized before each boundary
varied with batching. In particular the candidate synchronized more values in
final construction and fewer across the entire operation. The derived index
need not synchronize the final tail if there is no later lookup. These counters
cannot be interpreted as identical per-phase work counts or evidence of missing
canonical input. There was no code change to move work outside the Init timer.

The whole metadata-preparation interval improved while its final-phase subset
did not. Reporting only the final subset would conceal the partial benefit;
reporting only the 6.640 ms median whole-Init difference would conceal the opposite
paired outcomes and much larger control variation. Neither supports promotion.

No further candidate variants were attempted. The next useful investigation is
the remaining index/B-tree/cache and lookup execution cost, with explicit counts
and measured subintervals. Do not stack a cache or lookup rewrite on this rejected
patch. The #115 DeltaSearch question remains a separate shared-caller problem.

## Correctness, storage and resources

All four comparison Stores contain 112451 canonical objects/513026835 canonical
bytes, zero object reuse, 100000 scanned files / 500000000 scanned bytes and schema10
with 4096-byte pages. Four producers; process threads return to one; zero swaps.

| Arm | Apparent Store bytes | Allocated Store bytes | Peak process RSS bytes |
| --- | ---: | ---: | ---: |
| C1 | 515,379,200 | 518,664,192 | 85,884,928 |
| T1 | 515,514,368 | 527,056,896 | 85,950,464 |
| T2 | 515,489,792 | 522,555,392 | 91,291,648 |
| C2 | 515,506,176 | 524,505,088 | 85,262,336 |

The helper owns bounded SQL text and up to 165 borrowed-value/ordinal pairs;
it adds transient allocations, not a persistent cache. RSS is reported rather
than presumed unchanged. No compaction/repack/VACUUM/GC was performed.

Because screening failed, the conditional plain campaign, four-tier candidate
qualification, independent matching-image verifier and Commit performance
campaign were not launched. Existing tests passing is not a substitute for
those promotion gates. No product fix is committed; the experimental candidate,
tests, exact arm diff and diagnostic binaries remain in the evidence root.

## Seals and custody

Fixture digest:
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.

| Identity | Control | Candidate |
| --- | --- | --- |
| Product | c9228589c4252fca7b91a11391968cfb58dae76f6ecd7303b690659760fcc970 | 90bdc864ba173c71807bbcb14b370ecfe239e7b0ec483c131e12357be0d63977 |
| Source | a7bda1459de5eafc89ca2d1a730ee03f68f7efce345c8eb2de4a437df6a34793 | 9d33cc919a96d5851c8c768746fffed45fd74e738d4e5dc28bffaf90a2a2f5d7 |
| Binary SHA-256 | 2d901ba79c98f531ee3027927408897afd196dc42497ff16488ef95f6a271033 | b8baf33c55c1637b5b0af226c2815a1443531e1ba6840fb10ee86e3f61bfc154 |

Before/after source identities match each built arm. Both binaries were qualified
by the host runner; no Docker image is needed for these direct host diagnostics.
The initial dirty patch, control/candidate source patches, candidate-vs-control
patch, both build logs/identities, every attempt's raw output and exits, analysis
and runnable checks are retained. Older sealed evidence roots were only read.

This is #111's bounded follow-up to #115, with context in #109/#110 and shared
qualification references #108/#106/#102/#104/#100/#107. No release/tag/deployment.

Issue outcome: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5630789309

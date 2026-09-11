# Aggressive removal and scaling review

Three read-only subagent reviews examined authentication elimination, metadata
index elimination, and integrity/scaling tradeoffs. No product changes, builds
or timing runs were performed. Findings use the current source and the retained
[RCA](final-tree-rca-results.md), [INSERT experiment](metadata-insert-results.md)
and [#115 assessment](https://github.com/Ephemeral-AI-Lab/layerfs/issues/115#issuecomment-5630420769).

The owner's scaling requirement reveals two structural issues that resource
bounds alone do not solve: repeated full-catalogue aggregates and full-history
reconstruction of a disposable index after reopen. Actual stored-data and
intermediate-base authentication should remain under the current contract.

## Precise scaling requirement

Let N be existing metadata values, G catalogue groups, K changed/emitted values,
B newly ingested or necessarily authenticated bytes, R the retained index bound,
and D the bounded dependency depth.

- Full Init must examine its new input: at least Ω(B + K). Sublinear work in
  those inputs would require skipping some of the required operation.
- A small incremental Commit should avoid scanning unrelated N/G: target work
  proportional to changed bytes and metadata, touched tree paths and indexed
  lookups, such as O(B + K log N), with explicitly bounded dependency expansion.
  This is a design target, not a proven bound on every current Commit path.
- Keeping K fixed while increasing N should not introduce a linear historical
  scan. Reopened and retained Store cases must be measured separately.
- Bounded memory does not imply bounded total work. R=131072 does not help if
  initialization scans millions of values only to evict most of them again.

## 1. Remove repeated complete catalogue aggregation

Both `objects/metadata.rs::next_metadata_ordinal` and publication code in
`objects/admission.rs` use:

```sql
SELECT COALESCE(MAX(first_ordinal+count),1) FROM metadata_value_groups
```

`sql/schema/v10.sql` indexes first_ordinal, not the summed expression. Reviewers'
illustrative SQLite 3.51.2 EXPLAIN checks showed Rewind/AggStep/Next traversal;
the exact product-linked SQLite 3.51.0 plan still needs a runnable check before
implementation. The EQP label SEARCH alone is not evidence of a single seek.

Each aggregate traverses G groups. Repeated as bounded admissions grow the
catalogue, this can accumulate ΣG_i and quadratic total catalogue-row visits.
It also adds an O(G) term to retained-Store incremental preparation.

For a valid contiguous append-only catalogue, the endpoint is in its final
primary-key row. An ordered tail seek can avoid this traversal without an extra
index. Preserve empty-catalogue 1, ordinal limits, rollback behavior and catalogue
integrity requirements. The schema alone does not prove contiguity/nonoverlap;
do not silently exchange MAX for a last-row assumption on malformed data.
Tests must cover gaps, overlaps, altered counts and endpoint exhaustion, and
establish which validation owns those invariants.

Prior final-phase catalogue time was only 2.699 ms. This is a structural scaling
target, not a demonstrated route to hundreds of milliseconds at 100k.

## 2. Remove full-history startup replay

`ValueIndex::new` starts next=1 after Store reopen. Synchronization then reads,
authenticates, decodes and inserts all historical groups. Whenever accumulated
entries exceed 131072, it clears the scratch table and continues. Work discarded
by later eviction was still performed. Approximate first-use cost includes
O(G log G + N log R), plus the bytes decoded/authenticated; exact implementation
costs require measurement.

A truly sublinear-in-history first small Commit cannot reconstruct this entire
derived accelerator. Alternatives need an explicit storage-policy decision:

- Preserve current reuse semantics using a restartable or persistent searchable
  derivation, with measured extra space/write cost and robust validity/rollback
  handling. This is a design change, not a free cache.
- Reconstruct only a bounded recent window. This can preserve canonical integrity
  but may change which values deduplicate. Reproducing the current exact suffix
  is nontrivial: eviction boundaries depend on group sizes and greedy resets,
  so simply subtracting 131072 from the last ordinal is not equivalent.
- Remove the global ValueIndex entirely, retaining publication-local pending
  values, fresh ordinal allocation, packs and authentication. Canonical correctness
  can survive, but cross-admission/reopen metadata sharing does not remain the same.

The last option is the most aggressive removal reviewed. Its old final-phase
envelope was sync 215.984 + lookup 131.333 = 347.317 ms. This is not a savings
prediction: duplicate values create more physical groups, encoding, writes and
potentially less effective pooled-metadata DELTA representations.
`metadata_values_share_across_prepared_packs_and_reopen` and
`metadata_batched_value_lookup_pages_and_sharing` explicitly require sharing
that this deletion would lose. Treat it as an alternative storage-policy
experiment requiring byte-growth evidence, not an equivalent optimization.

## 3. Remove encode-time self-decompression, retain authentication

`prepare_values` encodes metadata groups, clones the encoded bytes, decodes
them, then hashes the resulting group body for its persisted digest. The pack
encoder already constructs those exact uncompressed bytes before zstd.

Providing that body's digest to the metadata caller could remove the clone and
decompression while preserving the same persisted identity and reader checks.
Hash the complete framed body (count, offsets, record tags and values), not just
the 73-byte values or compressed bytes. Avoid imposing new hashing/retention on
unrelated content groups or unused encoding alternatives.

Measured decode+digest envelope: 16.442 ms in the old final-phase RCA. Hashing
remains, so the recoverable part is smaller. Verify independent decode/digest
equality for RAW/zstd, 1/164/165 values and tail groups; preserve corruption tests.

## 4. Other removals and their limits

Passing newly published values directly to the retained index could avoid their
readback, but does not remove index insertion/lookup or reopen reconstruction.
The measured read/authenticate/decode envelope was 18.805 ms. Correct publication,
late-winner, eviction and rollback handoff adds lifecycle complexity.

Do not remove publication-local pending_values: it provides sharing before
new groups become database-visible. Removing its lookup-deduplication seen set
may preserve output but amplify SQL inputs; measure frequency before choosing it.

Do not remove stored-group digest checks or authenticate only final DELTA output.
`metadata_pool_authenticates_unused_base_values` modifies a base value and its
group digest together; the target DELTA replaces that value. Target-only identity
checking can pass while the required intermediate-base identity check fails.
Those checks enforce distinct guarantees.

Canonical creation hashing supplies ObjectIds; it is not an optional redundant
check. AuthenticatedCanonicalObject establishes identity/outer framing, not all
inode-value semantic constraints. Exact positive CAS byte comparisons, bounds,
dependency checks and rollback invalidation likewise have distinct purposes.

## Required evidence before saying scaling is ensured

1. Full Init across existing tiers, then a separately declared larger tier if
   needed: count metadata groups/values, catalogue rows visited, SQL VM steps,
   authenticated bytes and index inserts. Cumulative visits must not follow G².
2. Fixed K=1/10/100 with growing existing namespace/history: separately test first
   Commit after reopen and retained-Store Commit. Count historical replay, not
   just wall time. Proposed larger vehicles require a frozen protocol first.
3. Grow history at fixed namespace/K past 131072 values; report evictions, replay
   and physical metadata growth. A smaller retained index is not a free speedup.
4. Compare repeated references to the same immutable group with distinct groups.
   Within a safe bounded ownership lifetime, verification should follow distinct
   bytes/groups where reuse is possible; disclose eviction and dependency costs.
5. Preserve corruption, exact CAS, sharing, reopen, failed-publication and rollback
   tests. Validate SQL plans against the pinned product engine. Keep plain cold
   qualification separate from diagnostics and warm paired comparisons.

Recommendation: address the repeated aggregate as a scaling defect; design away
whole-history startup replay; consider the self-decompression deletion as a small
compatible simplification. Complete index removal remains a storage-efficiency
tradeoff. Authentication removal is not supported by the current measurements
or correctness contract. No claim that current scaling is already ensured.

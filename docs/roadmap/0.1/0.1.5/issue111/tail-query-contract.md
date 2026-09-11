# Bounded metadata endpoint query protocol

Replace the two full-catalogue endpoint aggregates with one shared bounded query.
Because first_ordinal is a unique integer and count is in1..165, the maximum end
must belong to a row whose first_ordinal is greater than MAX(first_ordinal)-165.
Aggregate this indexed suffix instead of assuming the last row has the largest
end. Preserve exact MAX semantics for schema-conforming gaps/overlaps as well as
valid contiguous catalogues, empty result1, ordinal limits and transaction scope.
Retain all read/group authentication, sequential catalogue validation and rollback
checks. Arbitrary rows created by bypassing CHECK constraints are outside the
equivalence proof; full validation must still reject malformed count/group data.

No cache, schema/index, page-size, thread, pooling, DELTA or reopen-index change.
Evidence root is frozen in `/tmp/layerfs-tail-query-root` before edits; preserve
the original dirty patch and all commands/logs/exits. Use the runner lock for
resource-sensitive tests/builds/verification, no overlapping runs or double locks.

This is a CPU-work/scaling fix, not a new Init wall-time acceptance claim. Use
the linked product SQLite and actual schema in a deterministic runnable test:
catalogue sizes100/1000/10000/100000; execute old and new queries once per cell
on identical data, count VM steps, fullscan steps, sorts and statement memory.
Verify equal endpoints; bounded query <=1200 VM steps,0 fullscan steps,0 sorts,
statement memory <=64KiB. Include a dense165-row worst-case suffix. These are
work-counter checks, not timing samples; do not turn them into speedup percentages.

Focused tests cover empty/single, gaps, overlaps where last-row endpoint is wrong,
strict suffix boundary, ordinal exhaustion, altered counts, transaction rollback,
reopen and existing catalogue corruption/authentication tests. Run the existing
Store library suites. Build host through the qualified runner after final source
edits, then run registered Init verification on the matching candidate and image.
Keep cold plain <=2.7 s acceptance explicitly open; this query test cannot pass it.

Commit only intended shared query/test files and documentation. Preserve existing
uncommitted compaction-removal work. No release/tag/deployment.

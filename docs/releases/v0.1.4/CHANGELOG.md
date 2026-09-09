# LayerFS v0.1.4 changelog

> **Status:** LayerFS 0.1.4 release changelog.

1. **Packed SQLite storage and authenticated native representations.** Immutable
   packs contain bounded groups; native FULL/PREFIX records preserve canonical
   identity, authentication and dependency order. New schema-7 Stores use 4 KiB
   pages. The schema-7 writer fence rejects unsupported mixed research formats.
2. **Bounded shared Init/Commit construction and publication.** Streaming output,
   fresh membership filtering, bounded SQL coalescing and predecessor budgeting
   reduce unnecessary work while preserving final-root atomicity and rollback.
3. **Authenticated Init comparison reuse.** A 2 MiB owner-local allowance, carved
   from the existing index/filter budget, avoids repeated decode/authentication.
   Every hit still checks the actual locator, length and bytes; larger working
   sets retain ordinary fallback validation.
4. **Completed, honestly classified validation.** The accepted campaign has 198
   successful performance executions, 226 routine proofs, supplemental smokes and
   full157 retained-history verification. Original performance misses, four
   INELIGIBLE comparisons and one optional omission remain visible.
5. **Explicit compatibility exception.** Schema 5 from released v0.1.3 is rejected
   without migration. Supported development schema 6 remains legacy; new Stores
   use schema 7. No downgrade or retained-history migration command is supplied.

The current measured product is `9cfb4be477116646258ea0621280ed13b1824c6d`, including the Workspace follow-up below. Earlier issue #95 measurements at `c48bb4903f456136ccbcdba78de38b9042d2755a` and evidence at `36a5d9da612211cf26f2a23370e9d59afdceb8e2` remain historical. Subsequent reporting and packaging do not relabel observations.

Read the [release record](../../../release-notes/0.1.4/README.md),
[acceptance](../../../release-notes/0.1.4/acceptance.md),
[versioned manual](../../versioned/0.1.4/README.md) and
[all-case evidence](../../roadmap/0.1/0.1.4/issue95/README.md).

## Qualified Workspace admission follow-up

Reuse bounded SQL cohorts for Workspace admission and cap graph-ordered candidate-spool read-ahead at 64 KiB. The adjacent `.venv` pair reduced Commit 6.671 → 4.152 seconds with unchanged allocated storage. Complete native, benchmark, independent verification, full157 and final `.venv` qualification passed. [Measured benefits, RSS tradeoff, historical misses and exact identities](../../roadmap/0.1/0.1.4/issue98/README.md).

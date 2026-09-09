# R8 prospective scratch configuration repair

R7's five-second late namespace100000 profile contains3,897 main-thread samples
inside Init. Admission index insertion accounts for2,519 of those samples;
PreparedAdmission preparation152, publication845, object lookup452 (nested scopes,
not additive elapsed percentages). Scratch writes enter pagerAddPageToRollbackJournal
and guarded_pwrite_np. The10k profile lacked this spill-dominated late phase.

The focused before-check proves scratch_index requests OFF but actually receives
DELETE on this host. The isolated configuration check proves OFF can be configured
on a newly created empty private database, then the original DEFENSIVE setting
restored before schema/data access. SQLite documents that DEFENSIVE blocks the
journal_mode=OFF statement: https://www.sqlite.org/c3ref/c_dbconfig_defensive.html
The existing v0.1.3 scratch helper also made the unchecked request, but its64MiB
seen set did not spill at this fixture. Packed admission's two16MiB sets cross
their12MiB entry thresholds, exposing disk journal amplification. This is a
migration-triggered scratch bottleneck, distinct from native codec cost.

Repair only scratch_index: read its default defensive flag, temporarily disable
it solely while issuing the fixed OFF pragma on the freshly owned empty scratch
file, restore the original flag even if configuration fails, and verify effective
OFF before any schema/data use. Preserve4MiB page cache, FILE temp storage, cache
spill, no mmap, exclusive ownership and all original memory thresholds. This is
not a change to the durable Store's MEMORY journal/publication/durability policy,
not an OFF-journal customer Store, and not a permanently disabled defensive flag.
OFF scratch is discarded/poisoned after any failed mutation, as already required;
no partial scratch state may be consulted or published. No in-memory journal,
raised memory bound, new index/cache/backend, encoding or pack-selection change.

Tests: effective OFF and restored default flag; no disk journal sidecar while
writing a multi-page scratch index; exact dedup/membership and failure poison;
existing spill/import/collision/rollback/cleanup and Store policy checks. Retain
before failure and configuration probe. Freeze G3 after checks, build isolated
host/image identities, then the same four R6 cells and independent proofs in the
same order, one seed1 each. All affected final matrix/full157 obligations remain.
No promised speedup until measured; preserve G2's insufficient results. Storage
allocation must retain v0.1.4 savings alongside speed, including final equal-state
full157. All codepaths still authenticate and reconstruct canonical objects.

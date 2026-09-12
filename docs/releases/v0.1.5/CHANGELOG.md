# LayerFS v0.1.5 changelog

> **Status:** LayerFS 0.1.5 release changelog.

1. **Ordinary schema-10 Init/Commit storage.** New Stores use schema 10 with
   4 KiB pages, exact content-addressed admission, compression, bounded
   whole-file small-content representations with bounded delta chains,
   large-file CDC/extents, compact scoped namespaces and authenticated pooled
   metadata. Canonical identity, ObjectId domains and Commit derivation are
   unchanged.
2. **Explicit compaction is removed (owner decision, 2026-09-11).** The owner
   judged compaction too slow and insufficiently useful for the intended
   workloads: the #103 full157 Store shrank from 83,935,232 B to 55,476,224 B,
   but public compaction took 626.313062 s (626.518567 s process wall) in
   addition to ordinary construction. `LayerStackStore::compact_into`, the
   compaction options/receipt exports and `layerfs-store-compact` are gone;
   previously compacted Stores remain readable through the retained
   authenticated LFCNT1/version-107 path. Those historical measurements are
   history, not a v0.1.5 claim.
3. **Bounded pending representation for live edits (#116).** An equal-length
   overwrite of committed content is retained as one base root plus one bounded
   splice descriptor — one 64-byte descriptor (72 B for a spool replacement)
   instead of three 128-byte nodes — and converted to canonical pieces at
   Commit. Pending workspace capacity moves from 5,461 to ≈29,959 spliced files
   at the same budget, and the public default-budget route accepts 32,000 edits
   (32,000 × 64 B = 2,048,000 charge under the unchanged 2,097,152 B budget);
   the route's independent verification compared 32,000 changed files before
   and after a fresh reopen.
4. **Coalesced ordinary pack rows (#107).** Ordinary publication updates pack
   rows instead of appending one row per pack. On the matching full157 history:
   pack rows 3,457 → 1,058, SQLite pack overflow slack 2,142,701 → 1,222,187 B
   (−920,514 B, −43 %), apparent Store 83,525,632 → 82,583,552 B (−1.13 %), and
   allocated Store 83,935,232 → 83,951,616 B (+0.02 %) because the live file's
   allocation residue grew. The self-declared ≥1.5 % allocated target was not
   met numerically and is superseded by explicit owner acceptance
   (`owner-107-acceptance.json`), not re-based.
5. **Bounded fsync batching on ordinary write bursts.** The live path bounds the
   number of backing fences a write burst performs. This does not change the
   MEMORY-journal/synchronous-OFF durability contract.
6. **Honest qualification summary.** 227 registered selections are terminal:
   198 performance selections (182 fresh — **56 PASS / 125 WARN / 1 FAIL** — plus
   16 reused) and 29 proof-only (**28 PASS + 1 NOT_RUN_OPTIONAL**), with 182/182
   fresh independent proofs PASS, every cleanup PASS and no S0/S1. One registered
   Tier-1 gate is a **FAIL** dispositioned by an owner waiver
   (`dedup-history-unrelated-500-mixed-v2` 16.107 s vs `< 15 s`); the cold-Init
   2.7 s target, the Stage2 K10 target, the K32000 historical-family
   `TARGET_MISS` and the Git allocation-layout WARN remain explicit WARNs or
   waivers. Campaign context against published v0.1.3: 139/198 cells ≥15 %
   slower, median 1.34×, +33.115 s. Endurance is not qualified. See the
   [release record](../../../release-notes/0.1.5/README.md),
   [waivers](../../../release-notes/0.1.5/waivers.md) and
   [#120 final report](../../roadmap/0.1/0.1.5/issue120/final-report.md).

The measured product is host binary `c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295`
at commit `1ff1f2dddeb60493953311de316fa5bec4634a1a`, with the qualified
final-treatment evidence (`b5f089eb…`) reused where the frozen contract allows
it. The `v0.1.5` tag tree differs from that commit only by documentation and the
workspace version bump. No benchmark was re-run to publish this release.

Read the [release record](../../../release-notes/0.1.5/README.md),
[acceptance](../../../release-notes/0.1.5/acceptance.md),
[verification](../../../release-notes/0.1.5/verification.md),
[versioned manual](../../versioned/0.1.5/README.md) and
[all-case evidence](../../../release-notes/0.1.5/benchmark-closeout.md).

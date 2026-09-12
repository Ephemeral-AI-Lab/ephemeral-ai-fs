# LayerFS 0.1.5 Developer Preview

> **Status:** LayerFS 0.1.5 Developer Preview release record.
> [Tag and downloads](https://github.com/Ephemeral-AI-Lab/layerfs/releases/tag/v0.1.5).

The owner authorized this closure on 2026-09-13: close the v0.1.5 issue set
honestly, prepare the release documents, and commit, tag and publish v0.1.5 using
the inherited benchmark campaign — **no re-measurement**.

## What v0.1.5 is

1. **Ordinary schema-10 Init/Commit storage.** New Stores use schema 10 with
   4 KiB pages: exact content-addressed admission, compression, bounded
   whole-file small content below 128 KiB with bounded delta chains, large-file
   CDC/extents, compact scoped namespaces and authenticated pooled metadata.
   Canonical identity, ObjectId domains and Commit derivation are unchanged.
2. **Explicit compaction is removed** by the 2026-09-11 owner decision (the #103
   full157 Store shrank 83,935,232 → 55,476,224 B but compaction itself took
   626.313062 s). No `compact_into`, no compaction options/receipts, no
   `layerfs-store-compact`. Previously compacted Stores remain readable through
   the retained authenticated LFCNT1/version-107 path.
3. **Bounded pending representation (#116).** An equal-length overwrite of
   committed content retains one base root plus one bounded splice descriptor
   instead of one pending node per write. Pending workspace capacity moves from
   5,461 to ≈29,959 spliced files, and the public default-budget route accepts
   32,000 edits under the unchanged 2 MiB budget with independent verification.
4. **Coalesced ordinary pack rows (#107).** Pack rows 3,457 → 1,058, pack
   overflow slack −43 %, apparent Store −1.13 %, allocated Store +0.02 %; the
   self-declared allocated target was missed numerically and is accepted by
   explicit owner decision, not re-based.
5. **Bounded fsync batching** on ordinary write bursts, with the durability
   contract unchanged.

## Compatibility

New Stores use **schema 10** with 4 KiB pages. Supported schema-6/7/8/9 Stores
connect without promotion; published v0.1.3/schema-5 Stores are rejected.
`LayerStackStore::upgrade_format` promotes a closed schema-7/8/9 Store to schema
9 only — there is no in-place promotion to schema 10 and no downgrade. Preserve
old Stores with matching old binaries. See the
[release contract](release-contract.md) and the
[storage manual](../../docs/versioned/0.1.5/storage-format.md).

## Qualification at a glance

227 registered selections are terminal on the measured candidate
`c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295` @
`1ff1f2ddd`: **198/198 performance** (182 fresh — **56 PASS / 125 WARN / 1 FAIL**
— plus 16 reused) and **29/29 proof-only** (**28 PASS + 1 NOT_RUN_OPTIONAL**),
with 182/182 fresh independent proofs PASS, every cleanup PASS and no S0/S1. The
native gate passed with 530 tests in 118 s on that tree.

**Published as measured, never relabeled:** `dedup-history-unrelated-500-mixed-v2`
**16.107 s vs the registered `< 15 s` gate — FAIL**, dispositioned by an owner
[waiver](waivers.md); cold Init `namespace-100000` **4.3975 s** against the
owner-waived 2.7 s target; **125 WARN** cells; **139/198** cells ≥15 % slower
than published v0.1.3 (median 1.34×, +33.115 s total — context, never a gate);
**+2.49 ms/exec and +1.46 ms/commit** from #116 and **≈+0.4 ms/phase** from #107;
scattered-100's **≈452 ms** unattributed gap; ordinary full157 allocated
**83,951,616 B** versus Git live **56,197,120 B**; endurance **unqualified**.
Owner acceptance retires optimization scope; it does not relabel results.

## Read the release record

- [Release contract and compatibility boundary](release-contract.md)
- [Owner acceptance and issue disposition](acceptance.md)
- [Waivers and explicit warning dispositions](waivers.md)
- [Verification and exact-source applicability](verification.md)
- [Benchmark closeout: every family and case](benchmark-closeout.md)
- [Derived performance table](benchmark-performance.csv) · [derived proof table](benchmark-verification.csv)
- [Machine-readable release evidence](release-evidence.json)
- [Artifact preparation and checksums](artifacts.md)
- [GitHub release announcement](github-release.md)
- [Versioned manual](../../docs/versioned/0.1.5/README.md)
- [Limitations](../../docs/versioned/0.1.5/limitations.md)
- [Changelog](../../docs/releases/v0.1.5/CHANGELOG.md)
- [#120 finalization campaign report](../../docs/roadmap/0.1/0.1.5/issue120/final-report.md)

This is a **source-only Developer Preview** with no crash/power-loss durability
promise. No crates.io package, prebuilt executable or public runtime image is
part of this release; the GitHub release carries the tagged source and its
checksums. Existing qualified campaign results were reused; no benchmark was
re-run for publication.

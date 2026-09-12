# LayerFS 0.1.5 release contract

> **Status:** LayerFS 0.1.5 release record.

This is a source-only Developer Preview. The owner accepted the v0.1.5 stopping
point and directed honest closure; [acceptance](acceptance.md) and
[waivers](waivers.md) limit release claims to supported observations and
preserve every failed or missed numerical gate. Execution, correctness, custody
and authentication requirements are not waived.

## Compatibility boundary

The public SDK/CLI grammar and canonical identity remain as documented by the
[versioned manual](../../docs/versioned/0.1.5/README.md). Use matching SDK, CLI,
owner, daemon and FUSE builds; mixed-version live sessions are unsupported.

**New Stores use ordinary schema 10** with 4 KiB SQLite pages. The schema-10
format is the ordinary Init/Commit route: exact CAS, compression, bounded
whole-file small content below 128 KiB with bounded FULL/DELTA chains
(`LFS5SML\0`, one SmallContent object per pack-v3 group, at most 8 dependency
edges, 512 KiB decoded closure and 256 KiB retained encoded capacity),
large-file CDC/extents, compact scoped namespaces, pooled authenticated metadata
values and per-scope non-recycled serial allocation.

Supported schema-6 (legacy, writer-fenced), schema-7, schema-8 and schema-9
Stores connect and keep their own construction/encoding policy; opening a Store
does not promote it. `LayerStackStore::upgrade_format` is the explicit offline
promotion of a closed schema-7/8/9 Store to **schema 9** without rewriting
payloads, history or page size — it is not a schema-10 promotion. Schema-10
Stores are created by `create`. Released schema 5 and other unsupported versions
(including schema 4) are rejected without mutation or migration. There is no
downgrade and no retained-history transfer command; importing a directory into a
new Store starts new history at schema 10.

**Explicit compaction is removed.** The product no longer exposes
`LayerStackStore::compact_into`, compaction options/receipts or
`layerfs-store-compact`, and compaction must not be reintroduced as background
or asynchronous rewriting, mandatory maintenance or hidden benchmark
preparation. Stores compacted by an earlier build remain readable through the
retained authenticated LFCNT1/version-107 whole-file read path (read
compatibility only; its writers exist only in tests). No old data is deleted or
reinterpreted. The owner decision and its historical measurements are in
`docs/roadmap/0.1/0.1.5/compaction-removal.md`.

SQLite publication and supported live-process recovery do not establish
process-crash, OS-crash or power-loss durability. The MEMORY journal and
synchronous-OFF profile is unchanged, and bounded fsync batching on ordinary
write bursts does not upgrade it.

## Evidence and release decision

The measured product is host binary
`c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295` at commit
`1ff1f2dddeb60493953311de316fa5bec4634a1a`, with source seal
`a555211fdc6e96e025764eba2c2c36f709baa7e9446840a07a8669935252fec3`, product seal
`276c5970aabf485594d90ae30920b3cdb310134a7572589c558063a9d52ce093` and image
`sha256:c83085b897e272204e5477e66b33ec9b328af568299448872441e7a676ce1761`. The
build identity probe on that tree reports `schema_version 10` with
`storage_policy: ordinary`; that probe record is carried in every campaign
receipt.

The #120 campaign is the evidence base and was **not** re-run: 198/198 registered
performance selections and 29/29 proof-only selections terminal, 182/182 fresh
proofs PASS, native gate PASS (530 tests, 118 s). The ordinary full157
construction/verification/census/Git157 control, the historical access 11 + 11
cases, the default-budget K32000 route with its independent verification, the
K6000 boundary and the default-budget frontier proof are reused from the
[final treatment](#exact-source-applicability) under the frozen applicability
decision.

The release is neither an all-speed-gates pass nor a storage-target pass. One
registered Tier-1 gate is a FAIL dispositioned by an owner waiver; the 125 WARN
cells, the 139/198 ≥15 %-versus-v0.1.3 campaign context, the #108/#112/#114
unmet gates, the endurance gap and every retained WARN stay visible in the
[closeout](benchmark-closeout.md) and [waivers](waivers.md). The optimization
scope is retired for this release; owner acceptance does not convert any
measured miss into a pass.

## Exact-source applicability

The tag tree differs from the measured product commit `1ff1f2ddd` only by
documentation and the workspace version bump (`0.1.4` → `0.1.5`), so no product
byte changes between the measured source and the released source. The reused
final-treatment evidence (`b5f089eb…`) is applicable because `3e308a8f2` was
decided behavior-neutral before the campaign (line-by-line identity
transformations plus rustfmt reflow) — see
[verification](verification.md) and
`docs/roadmap/0.1/0.1.5/issue120/applicability-3e308a8f2.md`.

A release tag must identify reviewed source. Final artifacts/checksums are made
from that exact tagged commit as described in [artifacts](artifacts.md); local
benchmark binaries and images remain under their original seals and are not
relabeled as release binaries. The owner authorized merge, tag and publication
using these existing results.

# LayerFS 0.1.5 manual

> **Status:** LayerFS 0.1.5 Developer Preview manual.

This is the versioned manual for the 0.1.5 Developer Preview. The release is
source-only: the [GitHub release](https://github.com/Ephemeral-AI-Lab/layerfs/releases/tag/v0.1.5)
carries the tagged source and its checksums, and no package, prebuilt executable
or public runtime image is published.

LayerFS provides local versioned Workspaces, shared FUSE/SDK editing and
explicit snapshot publication. v0.1.5 keeps the ordinary schema-10 Init/Commit
storage path (exact CAS, compression, bounded small-file FULL/DELTA, large-file
CDC/extents, compact scoped namespaces and pooled metadata), adds a bounded
pending representation for live edits and coalesced ordinary pack rows, and
**removes explicit compaction** by owner decision. Canonical identity is
unchanged; the SQLite format boundary moves to schema 10.

## Read this manual

1. [Quickstart](quickstart.md)
2. [Product specification](specification.md)
3. [CLI reference](cli.md)
4. [Rust SDK reference](sdk.md)
5. [Container runtime](container-runtime.md)
6. [Storage format](storage-format.md)
7. [Limitations](limitations.md)

## Compatibility boundary

New Stores use **schema 10** with 4 KiB pages and pooled metadata tables.
Supported schema-6 (legacy), schema-7, schema-8 and schema-9 Stores connect
without promotion and keep the construction/encoding policy of their own
schema. Published v0.1.3/schema-5 Stores and other unsupported versions are
rejected without mutation; there is no in-place promotion to schema 10, no
downgrade and no retained-history transfer command. Keep existing Stores with
their matching binaries. Importing a directory into a new Store copies the
selected filesystem state and starts new history. See
[storage compatibility](storage-format.md#compatibility-and-migration).

**Explicit compaction is removed.** New Stores and every ordinary write use the
schema-10 ordinary path; there is no `LayerStackStore::compact_into`, no
compaction options/receipt export and no `layerfs-store-compact` binary in this
release. Stores that were compacted by an earlier build remain readable through
the retained authenticated LFCNT1/version-107 read path, and no old data is
deleted or reinterpreted. The removal decision and its historical measurements
are recorded in the [compaction removal decision](../../roadmap/0.1/0.1.5/compaction-removal.md).

Use matching SDK, CLI owner, daemon and runtime components. Mixed-version live
sessions are unsupported. The [0.1.4 manual](../0.1.4/README.md) remains the
historical 0.1.4 contract.

## Qualification and acceptance

The [v0.1.5 release record](../../../release-notes/0.1.5/README.md) records the
measured product source (`c55daf13…` at `1ff1f2ddd`), the inherited #120
campaign, every waiver and every unmet target. 227 registered selections are
terminal: 198 performance selections (182 fresh — 56 PASS / 125 WARN / 1 FAIL —
plus 16 reused) and 29 proof-only (28 PASS + 1 not run by the frozen campaign
declaration). Owner acceptance retires optimization scope for this release; it
does not relabel the one FAIL, the 125 WARNs or the unmet #108/#112/#114 gates as
passes. See [acceptance](../../../release-notes/0.1.5/acceptance.md) and
[evidence limits](limitations.md#scale-and-evidence).

# LayerFS 0.1.4 manual

> **Status:** LayerFS 0.1.4 Developer Preview manual.

This manual prepares the 0.1.4 Developer Preview. It does not assert that a
release tag, packages, binaries or runtime image have been published.

LayerFS provides local versioned Workspaces, shared FUSE/SDK editing and
explicit snapshot publication. This release adds packed native object storage
and bounded reuse of authenticated comparison bytes during directory Init.
Canonical identities remain unchanged; the SQLite format boundary changes.

## Read this manual

1. [Quickstart](quickstart.md)
2. [Product specification](specification.md)
3. [CLI reference](cli.md)
4. [Rust SDK reference](sdk.md)
5. [Container runtime](container-runtime.md)
6. [Storage format](storage-format.md)
7. [Limitations](limitations.md)

## Compatibility boundary

New Stores use schema 7 and 4 KiB pages. Exact supported legacy schema-6 Stores
remain schema 6 and retain legacy writes. Published 0.1.3 schema-5 Stores are
rejected; there is no in-place migration, downgrade or retained-history import
command. Keep existing Stores with their matching binaries. Importing a directory
into a new Store creates new history; it does not migrate the previous Store.
See [storage compatibility](storage-format.md#compatibility-and-migration).

Use matching SDK, CLI owner, daemon and runtime components. The
[0.1.3 manual](../0.1.3/README.md) remains the historical 0.1.3 contract.

## Qualification and acceptance

The [issue #95 terminal report](../../roadmap/0.1/0.1.4/issue95/README.md)
records the qualified product source, evidence custody, complete benchmark and
full157 results, remaining regressions and storage comparison. Owner acceptance
permits preparation at that stopping point; it does not turn performance misses
or ineligible comparisons into passes. See [evidence limits](limitations.md#scale-and-evidence).

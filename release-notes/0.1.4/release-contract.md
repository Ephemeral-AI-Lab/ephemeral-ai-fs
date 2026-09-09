# LayerFS 0.1.4 release contract

> **Status:** Release candidate for LayerFS 0.1.4.

This is a source-only Developer Preview. The owner accepts the recorded
storage/performance tradeoff; [acceptance](acceptance.md) limits release claims
to supported observations and explicitly preserves failed numerical gates.
Execution/correctness/custody requirements are not waived.

## Compatibility boundary

The public SDK/CLI grammar and canonical identity remain as documented by the
[versioned manual](../../docs/versioned/0.1.4/README.md). Use matching SDK, CLI,
owner, daemon and FUSE builds; mixed-version live sessions are unsupported.

New Stores use schema 7 and 4 KiB SQLite pages. Supported legacy schema 6 is
preserved with its legacy writer; schema 6 is not promoted on open. The
schema-7 writer fence protects native version-2 packs. A research schema-6 Store
containing native packs without that fence is rejected. Released schema 5 and
other unsupported versions are rejected without mutation or migration.

This is an explicit pre-1.0 Developer Preview exception to the general patch
Store-format promise, continuing the approved new-Store-only storage scope.
It is not silent compatibility with v0.1.3. Preserve old Stores and matching
binaries. Importing a chosen filesystem state into a new Store does not migrate
retained Commit/Layer/Branch history. There is no downgrade or history-transfer
command.

SQLite publication and supported live-process recovery do not establish
process-crash, OS-crash or power-loss durability. The MEMORY journal and
synchronous-OFF profile is unchanged.

## Evidence and release decision

Benchmarks were collected on product source
`c48bb4903f456136ccbcdba78de38b9042d2755a`; package version metadata and release
documents are prepared afterward. Original binary/image identities are retained.
No timing is relabeled as a new measurement of version-bumped release artifacts.

The full family registry and independent routine coverage executed. The optional
600-second proof remains not run. Four historical Git comparisons remain
INELIGIBLE; numerical regressions and target misses remain visible. Release
claims exclude invalid historical Git speed comparisons. The complete reporter
still says INCOMPLETE, and must not be advertised as an all-speed-gates pass.

A release tag must identify reviewed source after the stacked implementation and
preparation PRs are integrated. Preparation does not create a tag or publish a
GitHub release. Final artifacts/checksums must be made from that exact reviewed
commit as described in [artifacts](artifacts.md).

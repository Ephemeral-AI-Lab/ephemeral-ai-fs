# LayerFS 0.1.3 Developer Preview

> **Status:** Released for LayerFS 0.1.3 Developer Preview.

LayerFS 0.1.3 delivers a shared live Workspace core, lower Commit and temporary-storage overhead, faster authenticated reads, and improved recovery. Its completed benchmark checkpoint covers **198 performance cases and 226 routine proofs across 17 families**.

- [Numbered changelog and family index](../../docs/releases/v0.1.3/CHANGELOG.md)
- [Detailed engineering account](../../docs/releases/v0.1.3/engineering.md)
- [Full benchmark tables](../../docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md)
- [Versioned manual](../../docs/versioned/0.1.3/README.md)
- [Release contract and upgrade boundary](release-contract.md)
- [Verification and source binding](verification.md)
- [Artifact manifest](artifacts.md)
- [GitHub announcement](github-release.md)

The release is source-only: no crates.io packages, prebuilt executables, or public runtime image are published. Version 0.1.3 upgrades schema-4 Stores to schema 5 on connect. Use matching SDK, owner, and daemon builds; retain a separate pre-upgrade backup because old binaries cannot open schema 5.

The benchmark source is `9f5a641d223606c45e5e6aa8a20094c12f9139a1`. Release preparation changes package versions and documentation and repairs an admitted-writeback drain race found by live qualification; historical measurements retain their original source identities. The release verification record binds the final checks without relabeling those timings as newly collected samples.

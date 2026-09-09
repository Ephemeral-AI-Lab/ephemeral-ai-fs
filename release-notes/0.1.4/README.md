# LayerFS 0.1.4 Developer Preview

> **Status:** LayerFS 0.1.4 Developer Preview release record. [Tag and downloads](https://github.com/Ephemeral-AI-Lab/layerfs/releases/tag/v0.1.4).

The owner accepted the measured storage/performance tradeoff on 2026-09-09 and
requested honest reporting, release preparation and closure of related work.
This release contains packed SQLite storage, native FULL/PREFIX encoding,
bounded shared Init/Commit publication and authenticated Init comparison reuse.

The measured full157 Store uses **184,582,144 allocated bytes versus 218,116,096
bytes for its supplemental control: 15.374% lower**. Adjacent duplicate-heavy
Init observations improved by 80–82% against the prior R26 candidate. These are
specific observations, not a universal speed or storage guarantee.

**Compatibility changes:** new Stores use schema 7 with 4 KiB pages. Supported
legacy schema-6 Stores remain schema 6. Published v0.1.3/schema-5 Stores are
rejected without migration. Preserve old Stores with matching old binaries;
importing a filesystem snapshot into a new Store starts new history. See the
[storage manual](../../docs/versioned/0.1.4/storage-format.md).

**Known performance limits remain:** the current reporter records 49 SEVERE,
95 REVIEW, 31 OBSERVED_INCREASE, 19 NO_INCREASE and four INELIGIBLE elapsed
comparisons. Historical reports retain their original classifications. Three absolute latency targets miss. Four Git fixture-binding
limitations leave the current overall report INCOMPLETE. Owner acceptance
retires the optimization scope; it does not relabel these results as passes.

- [Release contract and accepted exceptions](release-contract.md)
- [Owner acceptance and issue disposition](acceptance.md)
- [Verification and exact-source applicability](verification.md)
- [Artifact preparation and remaining publication steps](artifacts.md)
- [GitHub release announcement draft](github-release.md)
- [Versioned manual](../../docs/versioned/0.1.4/README.md)
- [Changelog](../../docs/releases/v0.1.4/CHANGELOG.md)
- [Current benchmark/root-cause report](../../docs/roadmap/0.1/0.1.4/issue98/README.md)
- [Historical issue #95 report](../../docs/roadmap/0.1/0.1.4/issue95/README.md)

This is a source-only Developer Preview with no crash/power-loss durability promise. No crates.io package, prebuilt executable or public runtime image is part of this release. The GitHub release records the actual tag, commit and asset checksums.

- [Benchmark closeout: every family and case](benchmark-closeout.md)

The subsequent [issue #98 Workspace repair](../../docs/roadmap/0.1/0.1.4/issue98/README.md) is fully qualified and ready for review: adjacent Commit time improved 37.8%, final allocated storage is unchanged, and all benchmark/proof/full157 checks passed. Final `.venv` observations and higher unpaired full157 wall times are reported explicitly. Remaining historical performance misses and four ineligible Git comparisons stay visible.

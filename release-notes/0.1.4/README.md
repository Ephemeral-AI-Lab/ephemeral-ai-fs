# LayerFS 0.1.4 Developer Preview

> **Status:** Release candidate for LayerFS 0.1.4. Prepared; not tagged or published.

The owner accepted the measured storage/performance tradeoff on 2026-09-09 and
requested honest reporting, release preparation and closure of related work.
This candidate contains packed SQLite storage, native FULL/PREFIX encoding,
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

**Known performance limits remain:** the frozen reporter retains 56 SEVERE,
86 REVIEW, 35 OBSERVED_INCREASE, 17 NO_INCREASE and four INELIGIBLE elapsed
comparisons. Three absolute latency targets miss. Four Git fixture-binding
limitations leave the unchanged overall report INCOMPLETE. Owner acceptance
retires the optimization scope; it does not relabel these results as passes.

- [Release contract and accepted exceptions](release-contract.md)
- [Owner acceptance and issue disposition](acceptance.md)
- [Verification and exact-source applicability](verification.md)
- [Artifact preparation and remaining publication steps](artifacts.md)
- [GitHub release announcement draft](github-release.md)
- [Versioned manual](../../docs/versioned/0.1.4/README.md)
- [Changelog](../../docs/releases/v0.1.4/CHANGELOG.md)
- [Complete benchmark/root-cause report](../../docs/roadmap/0.1/0.1.4/issue95/README.md)

This remains a source-only Developer Preview with no crash/power-loss durability
promise. No crates.io package, prebuilt executable or public runtime image is
claimed published. Merge, reviewed tag creation and GitHub publication are
separate from this prepared candidate.

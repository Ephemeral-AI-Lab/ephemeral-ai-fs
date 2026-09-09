# LayerFS v0.1.4 — storage efficiency Developer Preview

> **Status:** LayerFS 0.1.4 release announcement.

v0.1.4 adds packed SQLite storage, native FULL/PREFIX representations, bounded
shared Init/Commit publication, and reuse of authenticated Init comparison bytes.

The accepted full157 observation uses **184,582,144 allocated bytes**, **15.374%
less** than its 218,116,096-byte supplemental control. Adjacent duplicate-heavy
Init observations improve by **80–82%** against the prior R26 candidate while
preserving exact collision checks. These are scoped observations, not universal
performance guarantees or statistically established distributions.

Validation covers 198 performance cases, 226 routine proofs, supplemental
small-file/SDK/FUSE workloads, 157 performance states and 157 retained-history
proofs. The optional 600-second endurance proof was not run.

**Known tradeoffs:** substantial regressions versus v0.1.3 remain. Three absolute
latency targets miss; the four historical Git comparisons are INELIGIBLE and
the current overall report remains INCOMPLETE. Owner acceptance does not turn
those outcomes into passes. Full157 historical verification also took longer
than the prior candidate. Read the complete tables and measurement boundaries.

**Upgrade boundary:** create a new schema-7 Store. Supported development
schema-6 Stores retain the legacy format. Released v0.1.3/schema-5 Stores are
not migrated or opened; preserve them with their original binaries. Directory
import starts new history. Use matching SDK/owner/daemon builds.

This is a source-only Developer Preview, not production storage. Crash and
power-loss durability are not promised. The source assets and checksums are attached to the GitHub release.

See the versioned manual, accepted-tradeoff record and full terminal evidence
in the accompanying source tree.

The subsequent [issue #98 Workspace repair](../../docs/roadmap/0.1/0.1.4/issue98/README.md) is fully qualified and ready for review: adjacent Commit time improved 37.8%, final allocated storage is unchanged, and all benchmark/proof/full157 checks passed. Final `.venv` observations and higher unpaired full157 wall times are reported explicitly. Remaining historical performance misses and four ineligible Git comparisons stay visible.

[Every benchmark family and case](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.4/release-notes/0.1.4/benchmark-closeout.md). Existing qualified results were reused; no benchmark was rerun for publication.

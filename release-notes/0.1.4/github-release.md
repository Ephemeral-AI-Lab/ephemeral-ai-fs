# LayerFS v0.1.4 — storage efficiency Developer Preview

> **Status:** Release candidate announcement draft; not published.

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
the unchanged overall report remains INCOMPLETE. Owner acceptance does not turn
those outcomes into passes. Full157 historical verification also took longer
than the prior candidate. Read the complete tables and measurement boundaries.

**Upgrade boundary:** create a new schema-7 Store. Supported development
schema-6 Stores retain the legacy format. Released v0.1.3/schema-5 Stores are
not migrated or opened; preserve them with their original binaries. Directory
import starts new history. Use matching SDK/owner/daemon builds.

This is a source-only Developer Preview, not production storage. Crash and
power-loss durability are not promised. Final asset and tag links must be
inserted from the reviewed publication, not guessed during preparation.

See the versioned manual, accepted-tradeoff record and full terminal evidence
in the accompanying source tree.

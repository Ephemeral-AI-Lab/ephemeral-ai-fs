# v0.1.4 verification and source applicability

> **Status:** Release candidate for LayerFS 0.1.4.

The authoritative measured product is
`c48bb4903f456136ccbcdba78de38b9042d2755a`; its final evidence-only head is
`36a5d9da612211cf26f2a23370e9d59afdceb8e2`. The
[terminal qualification](../../docs/roadmap/0.1/0.1.4/issue95/evidence/terminal-qualification.json)
and [report](../../docs/roadmap/0.1/0.1.4/issue95/README.md) preserve:

| Check | Result |
|---|---|
| Full benchmark | 198 performance executions passed |
| Independent routine verification | 226 passed; optional 600-second proof not run |
| Focus families | 30 performance cases and 31 proof members passed |
| Native suite | 410 passed; ignored spill/reopen member passed separately |
| Additional checks | Doctest, formatting, warning-denying Clippy and CI passed |
| Supplemental workloads | Small-files plus four SDK/FUSE frequent-edit variants passed performance and verification |
| Full157 | 157 performance states, 157 retained-history proofs, 158 validation/accounting records and cleanup passed |
| Full157 allocation | 184,582,144 B vs 218,116,096 B supplemental control; 15.374% lower |

The four pre-start measurement-lock refusals and their targeted retries remain
archived. No passing performance cases were recollected to improve numbers.
The standalone census decoder and supplemental control have explicit source,
contract and custody applicability. Routine sampled-content verification is
not reclassified as an exhaustive byte census.

## Preparation delta

The preparation changes workspace package identities from 0.1.3 to 0.1.4 and
adds release documentation. Rust implementation, schema, native codec, public
operations, benchmark workloads, timers and verifier assertions remain unchanged
from the measured source. The root lockfile changes only the 12 project-owned
package versions; imported packages/versions/features/checksums are unchanged.
Historical research lockfiles remain tied to their original recorded source.

Preparation-specific build, native, CLI-version, formatting, Clippy, dependency
and documentation checks are recorded in `release-evidence.json` when complete.
Those checks qualify packaging and source applicability; they do not manufacture
new performance observations. Published source archives must bind to the final
reviewed commit, not merely a version string.

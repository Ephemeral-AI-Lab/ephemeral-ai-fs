# v0.1.4 verification and source applicability

> **Status:** LayerFS 0.1.4 release record.

The current qualified product is `9cfb4be477116646258ea0621280ed13b1824c6d`; see the [issue #98 terminal report](../../docs/roadmap/0.1/0.1.4/issue98/README.md). It passed 411 native tests, all 198 performance executions, 226 routine proofs, full157 and three final full `.venv` proofs.

The earlier issue #95 product was `c48bb4903f456136ccbcdba78de38b9042d2755a`, with evidence head `36a5d9da612211cf26f2a23370e9d59afdceb8e2`. Its historical
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

## Versioned preparation checks

The workspace release build, 410 native tests, formatting, warning-denying Clippy, CLI `layerfs 0.1.4`, imported-dependency audit and documentation checks passed on `176e4e83863aa743298263235974413078016495`. The archive helper's runnable check is `python3 release-notes/0.1.4/check_artifact_helper.py`; run it under the shared measurement lock. It checks candidate archives, missing-tag rejection, overwrite rejection, untracked-file exclusion and checksum corruption.

A clean matching host/image pair built from `70fdd839dd68c91491524590ed034b9b516431bd` passed the small-files mounted performance and independent verification smoke. Exact identities and receipts are in [release-evidence.json](release-evidence.json) and [qualification](qualification/). One image-build invocation was refused before launch while the host build held the measurement lock; the preserved retry ran after host completion. Subsequent changes add only documentation, artifact-tool checking and evidence. These are packaging checks, not a replacement full benchmark or new speedup claim.

## Completed Torch .venv Workspace acceptance

The formerly paused full `.venv` Workspace proof is now complete: three fresh imports/Commits and three exhaustive independent content/metadata verifications passed on the unchanged v0.1.4 implementation. The frozen fixture contains 17,682 entries and 581,658,413 file bytes. Healthy presentation, clean End, zero active sessions/executions, no swap/OOM and owned-container cleanup passed. See the [full report and exact receipts](../../docs/roadmap/0.1/0.1.4/issue71-venv-acceptance/README.md). This is supplemental correctness acceptance with descriptive timings, not a new historical speedup claim or a change to the formal campaign counts.

## Qualified Workspace follow-up

Issue #98 adds bounded Workspace transaction coalescing and a 64-KiB cap on graph-ordered spill read-ahead. The adjacent original-control/candidate pair reduced Commit 6.671061 → 4.152412 seconds; final three-sample confirmations were 4.578398–4.643631 seconds. Full157 allocation remains exactly 184,582,144 B. Full157 historical wall observations increased and are explicitly reported without a paired causal claim. The current comparison report remains INCOMPLETE for four Git fixtures; [all current results and limitations](../../docs/roadmap/0.1/0.1.4/issue98/README.md) supersede no historical evidence.

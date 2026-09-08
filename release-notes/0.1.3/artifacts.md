# LayerFS 0.1.3 artifacts

> **Status:** Release candidate artifact specification for LayerFS 0.1.3.

| Asset | Contents |
|---|---|
| `layerfs-0.1.3.tar.gz` | Exact tagged source under `layerfs-0.1.3/` |
| `layerfs-0.1.3.zip` | Exact tagged source under `layerfs-0.1.3/` |
| `layerfs-0.1.3-benchmark-data.tar.gz` | Published checkpoint report, registry, declarations, and raw evidence |
| `Cargo.lock` | Tagged dependency lockfile |
| `LICENSE` | Tagged license |
| `SHA256SUMS` | SHA-256 checksums for the five assets above |

The evidence bundle retains the checkpoint's original source identities and does not imply benchmarks were rerun for the release. Source archives also contain the versioned manual, changelog, engineering account, and release records. GitHub's automatic source downloads are additional platform-generated archives, distinct from these checksummed assets.

No prebuilt executables, crates.io packages, or runtime images are published. Locally built runtime images used for verification are recorded separately.

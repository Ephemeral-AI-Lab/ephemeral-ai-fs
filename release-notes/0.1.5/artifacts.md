# v0.1.5 artifacts and checksums

> **Status:** Source-only v0.1.5 artifact record.
> [Published assets](https://github.com/Ephemeral-AI-Lab/layerfs/releases/tag/v0.1.5).

The release follows the source-only v0.1.3/v0.1.4 distribution model. The
`v0.1.5` tag resolves to the final release-documentation commit, and the tagged
Git tree is the only source of the published assets.

| Asset | Contents |
|---|---|
| `layerfs-0.1.5.tar.gz` | Tagged source under `layerfs-0.1.5/` (every tracked file) |
| `layerfs-0.1.5.zip` | The same tagged source |
| `layerfs-0.1.5-benchmark-data.tar.gz` | The tracked evidence bundle: `docs/roadmap/0.1/0.1.5/**`, the v0.1.3 checkpoint baseline, the benchmark family registry, the benchmark rules, `docs/versioned/0.1.5/**`, `docs/releases/v0.1.5/**` and `release-notes/0.1.5/**` |
| `Cargo.lock` | Tagged root lockfile |
| `LICENSE` | Tagged license |
| `SHA256SUMS` | Actual SHA-256 checksums of the preceding five assets |

GitHub's automatically generated source archives are additional
platform-generated downloads and are not part of this list.

**What the evidence bundle does and does not contain.** The campaign receipts
(raw `perf.jsonl` and `verification.json` files, ≈196 MB) live under the
untracked local evidence root `benchmark-results/host-store/issue120/`; they are
cited by path from the tracked reports and are deliberately **not** part of the
archive. The bundle therefore ships the complete tracked report set — including
every per-case value, disposition, receipt path and receipt fact — but not the
untracked raw bytes. That limitation is stated here rather than implied.

## Published checksums

<!-- CHECKSUMS:BEGIN -->
Checksums are recorded here after the assets are built from the tag, so that no
hash is invented before it exists.
<!-- CHECKSUMS:END -->

## Publication sequence

1. Create the annotated `v0.1.5` tag on the final release-documentation commit
   and record the resolved tag and commit IDs here and in
   [release-evidence.json](release-evidence.json). Never move an existing tag.
2. Prepare the six assets from the tagged Git tree with
   `release-notes/0.1.5/prepare_artifacts.py`, which archives tracked content
   only (untracked files are excluded), validates archive membership against the
   tagged tree, writes `SHA256SUMS` and re-validates the complete set.
3. Attach the six assets to the GitHub release with the
   [announcement](github-release.md), keeping the Developer Preview status, the
   schema-10 compatibility boundary, the compaction removal, the FAIL/waiver,
   the 125 WARNs and the unqualified endurance gap visible.
4. Copy the resulting checksums into this file. That update is a documentation
   commit after the tag; the tag tree itself differs from the measured product
   commit only by documentation and the version bump.

No crates.io publication, prebuilt executable or public runtime image is part of
this release. Local benchmark binaries and images remain under their original
seals and are not relabeled as release binaries.

## Reproducible preparation

Run `python3 release-notes/0.1.5/prepare_artifacts.py FRESH_OUTPUT` to prepare
the six assets from the actual `refs/tags/v0.1.5`; an absent tag or a commit
whose workspace version is not `0.1.5` is rejected. Repeat with `--verify` to
validate an existing directory without writing. `--candidate --ref COMMIT`
allows a clearly labeled untagged run for review; candidate assets are never
published.

`release-notes/0.1.5/check_artifact_helper.py` is a self-contained fixture check
for that helper (missing-tag rejection, overwrite rejection, untracked-file
exclusion, checksum corruption). It was **not executed** in this closure: the
issue's mandate allows only the four static gates, and the helper's validation
path is exercised for real by the asset preparation above.

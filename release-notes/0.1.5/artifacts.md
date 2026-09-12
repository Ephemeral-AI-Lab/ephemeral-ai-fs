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
The `v0.1.5` tag is an annotated tag (`5c7c9b0b01107461c3c144bd910539a6d73b12af`)
resolving to commit **`6ee1ec94cfdcb7bc8c55830e8348d553c20e2f13`**. The six assets
were prepared from that tag with `release-notes/0.1.5/prepare_artifacts.py`
(validation PASS: 4,801 tagged source members, 1,307 tracked evidence members,
untracked files excluded, `SHA256SUMS` re-verified) and published at
<https://github.com/Ephemeral-AI-Lab/layerfs/releases/tag/v0.1.5>.

| Asset | Bytes | SHA-256 |
|---|---:|---|
| `layerfs-0.1.5.tar.gz` | 63,465,344 | `b7439c3421cb627374f17a46ab1d03d288bd46554a7ccd59e9730250237df5b2` |
| `layerfs-0.1.5.zip` | 69,459,204 | `99b5c5332aa689063225950c98fcd8896dff369fcd7e81da8224dc443bbbd245` |
| `layerfs-0.1.5-benchmark-data.tar.gz` | 11,786,849 | `608c553978a559db520ea8d8aaa4682d98ef4ce62f9d32336553ee7d28748628` |
| `Cargo.lock` | 27,680 | `ddef31ec251492102f76c51f8916006bcf35f12818ffd864c2500192e262f4f4` |
| `LICENSE` | 1,069 | `e20a92efe4b92c0460bd0c475395166e37b6450ed410af155fa0ed99d479f676` |

`SHA256SUMS` contains exactly those five checksums in that order. GitHub's
automatically generated source archives are additional platform downloads and
are not covered by this list.

This checksum record is a documentation commit **after** the tag; the tag tree
itself differs from the measured product commit only by documentation and the
version bump, as stated in [verification](verification.md).
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

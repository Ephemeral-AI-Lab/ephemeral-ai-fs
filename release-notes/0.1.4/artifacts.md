# v0.1.4 artifact preparation

> **Status:** Release candidate for LayerFS 0.1.4. No final release assets published.

The candidate follows the source-only v0.1.3 distribution model. After the
stacked PRs are reviewed and integrated, use their exact accepted commit for
an annotated `v0.1.4` tag and the following release assets:

| Planned asset | Contents |
|---|---|
| `layerfs-0.1.4.tar.gz` | Reviewed tagged source under `layerfs-0.1.4/` |
| `layerfs-0.1.4.zip` | The same reviewed source |
| `layerfs-0.1.4-benchmark-data.tar.gz` | Accepted terminal evidence, raw references, declaration, reports and limitations |
| `Cargo.lock` | Tagged root lockfile |
| `LICENSE` | Tagged license |
| `SHA256SUMS` | Actual checksums of the preceding five assets |

Do not invent final archive hashes or claim they exist before tagging. GitHub's
automatic archives are additional platform-generated downloads. No crates.io
publication, prebuilt executable or public runtime image is part of this scope.

## Publication sequence

1. Review/integrate PR #94, then PR #96, then this preparation PR; preserve the
   measured source and immutable evidence references.
2. Confirm the final reviewed tree has only the documented packaging delta from
   the accepted implementation, and that preparation checks/CI pass. A product
   change invalidates blanket reuse and requires affected requalification.
3. Create the annotated tag on that exact reviewed commit. Record the resolved
   tag and commit IDs in the release record; never move an existing tag.
4. Create the source/evidence archives from the tagged Git tree, verify archive
   members and SHA-256 checksums, and attach them with the actual lock/license.
5. Publish the [announcement](github-release.md), keeping Developer Preview,
   schema compatibility, performance regressions, optional omission and the
   INCOMPLETE comparison report visible.

This preparation intentionally leaves merge, tag creation, final archive
construction and publication as explicit subsequent actions. Local benchmark
binaries/images remain under their original seals; they are not mislabeled as
new release binaries.

## Reproducible preparation

Run `python3 release-notes/0.1.4/prepare_artifacts.py FRESH_OUTPUT --candidate --ref COMMIT` under the shared measurement lock to prepare six clearly named candidate assets. The helper archives tracked Git content only and validates membership and checksums. Repeat with `--verify` to validate without writing. For the eventual authorized release, omit `--candidate` and use the actual `refs/tags/v0.1.4`; an absent tag is rejected. Candidate archives are not published release assets.

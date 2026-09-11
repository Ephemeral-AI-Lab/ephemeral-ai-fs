# Shared cold fixture metadata guard completed

Implemented in local commit `8def17a7b`; the Init product optimization is unchanged.
This closes the missing directory metadata check exposed by the fingerprint-index
experiment's invalid cloned fixture. No Init performance sample was run here.

The shared cold acquisition now checks regular-file mode 0640, directory mode 0750
and mtime 1700000000000000000 ns. It includes the payload root, derives the exact
directory inventory from file paths, rejects extra empty directories and symlinks,
checks file metadata before/after content acquisition and during page observation,
and rechecks directories before finalizing the receipt. Inputs are not repaired.

Qualification is versioned as `namespace-100000-cold-v2`; the underlying Darwin
invalidation/mincore method and 2.7 s target are unchanged. `metadata_validation`
records the fixed policy, completion status,100000 files and 1001 directories
(including the root). The report assessor requires this evidence. Missing, partial,
incorrect or old-v1 receipts are ineligible for current v2 qualification, and
incompatible pairs cannot produce improvement percentages. Historical raw receipts
and their original results remain untouched; they are not upgraded automatically.

Validation:

- A regression first failed against the old implementation: metadata-incomplete
  rows with a saved PASS and fast 2.6 s timing were still counted as valid.
- All 70 shared-harness tests then passed. New coverage includes the unchanged-byte/
  changed-directory-mtime copy error, root and nested directory metadata, file
  mode/mtime drift, unexpected empty directories, symlinked root, missing/partial/
  wrong metadata receipts, raw diagnostic retention and pair rejection.
- One untimed native acquisition of the original immutable namespace-100000
  fixture passed:100000 files,1001 directories,500000000 bytes,125169 checked
  data pages and 0 resident pages. Metadata status VERIFIED; acquisition status
  VERIFIED_COLD. Acquisition took 21.732 s, outside any Init timer.
- The retained bad fixture copy was rejected at its payload root for wrong mtime,
  with 0 payload files/pages acquired and metadata status UNVERIFIED. The bad copy
  and original fixture were not modified.

The shared runner already retains any subsequently measured raw timing when
acquisition is unverified; this change does not silently skip, repair or relabel
it. No new performance result or speedup is claimed. The prior 3.420 s result remains
historical under its recorded protocol and extra metadata audit; the 2.7 s target
remains open. Future qualification must provide the new v2 evidence.

Evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-cold-metadata-evidence/20260911T134416Z`.
The original dirty compaction-removal patch is preserved and excluded from commits.
Host/image build identities are recorded after resealing below. The product seal
must stay `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`.
No release, tag, deployment or further product optimization. Context #111/#115.

Resealed host/image build: PASS. Product seal and host binary are byte-identical
to before this harness-only change.

- Source seal: `371d5dc40336492ed4aa969f4d210536bfbf740a8e0ac0f8a7c44e1ada1ac38b`
- Product seal: `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`
- Host binary: `0051058ac8e9ffca19fee65e595c19a43abc64ad315536aa14abc2f7e6983b63`
- Image: `sha256:b394c02bcb0605b568464baac21bb0bbc3fa9b620a1e8b8c4940374d6eb61c0b`
- Image tag: `layerfs-bench-infra:371d5dc40336492e`

Issue update: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5635464693

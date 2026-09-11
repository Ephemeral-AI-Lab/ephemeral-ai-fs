# Commit baseline evidence pointer

Evidence root for the #111 current-main Commit baseline and phase attribution
([contract](commit-baseline-contract.md)):

`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z`

Created 2026-09-11T14:26:06Z, before any instrumentation or collection.

Initial custody snapshot captured in the root:

- HEAD: `3e22876be13fa185652e28f9568b439d66db955e` (docs-only handoff commit)
- Working tree: 27 modified/deleted tracked paths plus untracked
  compaction-removal/issue112/issue113 content — the preserved uncommitted
  compaction-removal work, treatment `promoted-uncompacted`, kept byte-identical
  and excluded from all commits made by this campaign.
- Full binary diff: `before.patch`, SHA-256
  `44fdf679f24deead8e5f0d7dc8b2d0dd3a06346d56d8477058ebc69db0f6c2cc`
- Status list: `before-status.txt`

The root is append-only evidence: every attempt (valid or invalid), command,
log, exit, source patch, binary/image identity, fixture validation, raw
receipt, and analyzer is retained there. Older evidence roots
(index-attribution, metadata-proof, fingerprint-index, cold-metadata,
final-tree-rca, restart-index-design) remain read-only.

Campaign outcome (see
[commit-baseline-results.md](commit-baseline-results.md)): 45/45 plain cells
(one full cohort retained as an invalid first attempt with a harness
expectation bug) and 30/30 diagnostic cells passed all gates;
`evidence-manifest.json` hashes all 21,767 retained files, and
`final-custody.json` records unchanged source seals after collection.

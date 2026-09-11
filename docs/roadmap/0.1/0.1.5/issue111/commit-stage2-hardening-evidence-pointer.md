# Commit Stage 2 hardening evidence pointer (#111)

> **Status:** Custody registration for the Stage 2 hardening campaign. Complete:
> the campaign ran end to end. See
> [commit-stage2-hardening-results.md](commit-stage2-hardening-results.md) for
> the outcome. Read-only registration; no measurement claim is made here.

## Roots

| Role | Path |
|---|---|
| Stage 2 hardening root (this campaign) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-hardening-evidence/20260912T120000Z` |
| Stage 2 root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z` |
| Stage 1 root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z` |
| Completed baseline root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z` |

No archived script, analyzer, validator or finalizer was executed in place. The
collector, analyzer and build script are copies adapted to this root's arm names.
Raw evidence is append-only.

## Source custody at freeze

| Item | Value |
|---|---|
| Main HEAD | `6f7a8b8d54e9855b42f0572555252c2420e2da2d` |
| Main worktree PRODUCT_SEAL, promoted Stage 2 only (before hardening) | `a608cd4edd25161584986b0f2885d2497a0c231a63a0b3dc7be73685bd0c0b38` |
| Main worktree SOURCE_SEAL, promoted Stage 2 only | `4c46c963298e1eb5d4dacad062d8fcf886ef27ae7d90c5db910920f92a7e7ce5` |
| Pre-freeze status / tracked diff / untracked hashes | `custody/before-status-all.txt`, `custody/before-tracked-scoped.patch`, `custody/before-untracked-hashes.txt` |

The promoted product seal was re-verified from source, not assumed:
`stage2-candidate` (the Stage 2 base arm, product seal `760eb0f2…`) plus
`candidate.patch` (sha256
`ffbe1fd1a793ab4c47901cd46cebb3a766b706e67c15e2534937f0974df2ea53`) reproduces
the working tree's promoted source and product seal `a608cd4e…` exactly. The
main worktree carries the preserved uncommitted compaction-removal treatment; the
clean committed tree alone does not.

## Isolated arms

| Arm | Tree | Treatment | SOURCE_SEAL | PRODUCT_SEAL |
|---|---|---|---|---|
| `control` | `hardening-control` | promoted optimized Stage 2 product | `4c46c963298e1eb5…` | `a608cd4edd251615…` |
| `candidate` | `hardening-candidate` | control + the three focused hardening fixes | recorded in `identity-candidate.json` | recorded in `identity-candidate.json` |

Both trees are independent self-contained repositories created inside this root by
copy from the main worktree at freeze, each carrying the campaign harness
(`benchmark/fs-bench-pro/src/commit_baseline.rs`) and the exact dirty treatment.
The two arms differ in exactly four crate files
(`layerfs-content/src/tree/batch.rs`, `layerfs-layerstack-store/src/objects.rs`,
`objects/metadata.rs`, `objects/read.rs`); this is asserted before each build.

Per-arm binary SHA256, image tag and immutable image ID are in
`identity-<arm>.json`, `image-<arm>.txt` and `image-id-<arm>.txt`.

## Custody breach during arm preparation and its repair

The first arm copies were made with `cp -a -l` (hard links). Writing the candidate
treatment then modified files **through the shared inode**, which changed the
Stage 2 root's `stage2-candidate` and `stage2-control` worktrees in place. This is
a violation of the read-only rule for that root and is disclosed here rather than
hidden:

- Affected: five crate files in each of `stage2-candidate` and `stage2-control`
  (`layerfs-content/src/object/access.rs`, `layerfs-content/src/tree/batch.rs`,
  `layerfs-layerstack-store/src/objects.rs`, `objects/metadata.rs`,
  `objects/read.rs`). No other file, and no other root, was affected.
- Repair: both archived trees were restored from their own git index and now
  report a clean `git status`; every crate file is byte-identical to its own `HEAD`
  commit, and no shared inode remains between those trees and this root
  (`find … -links +1` returns zero).
- Reported consequence: the Stage 2 root's live worktrees no longer carry the
  applied `candidate.patch`. The measured Stage 2 evidence (`cells/`,
  `candidate.patch`, the build logs and the recorded identity files) is untouched,
  and the seal chain above was re-derived from `candidate.patch` against the
  restored base, so the promoted identity is verified rather than assumed.
- This root's arms were rebuilt from verified sources afterwards, not from the
  altered trees.

## Campaign artefacts

| Artefact | Purpose |
|---|---|
| `sequence-paired-plain.json` (30 cells) | frozen paired plain measurement |
| `cells/<sequence>/stage2-<arm>/…` | raw cells, one fresh Store and container each |
| `collect.py`, `analyze.py` | collector and analyzer copies |
| `build-arm.sh`, `build-both.sh`, `build-host-*.log`, `build-image-*.log` | build entrypoints, logs and exit codes |
| `custody-sealcheck.py`, `custody/before-seals.txt` | seal re-derivation |
| `summary.json`, `analysis.log` | derived summaries from retained raw evidence |
| `test-control.log`, `test-candidate.log` | focused suite outcomes per arm |
| `evidence-manifest.json` | sha256 of every retained file |

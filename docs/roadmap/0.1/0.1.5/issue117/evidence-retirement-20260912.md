# Worktree and evidence retirement — 2026-09-12

Owner explicitly requested removal of all LayerFS linked worktrees, then also
instructed: “drop evidences as well, and mention they are dropped.” This newer
instruction authorizes the evidence deletion, including roots previously marked
sealed/read-only. It does not alter historical measurements into new results.

## Completed

- Removed all seven registered linked worktrees under layerfs-*; `git worktree
  list` now contains only `/Users/yifanxu/Ephemeral-AI-Lab/layerfs` on main.
- Every removed HEAD was already an ancestor of main. No branch merge was needed.
  Historical diagnostic/experimental dirty source was recovered, not silently
  promoted into the product.
- Deleted45 explicitly listed LayerFS evidence/run/archive directories below.
  This includes Stage1/Stage2/Stage3, #104/#109 and related historical campaigns.
- Main HEAD before documentation was a258c4936. Its tracked diff, staged diff,
  status and14 preexisting untracked file hashes were verified unchanged after
  cleanup. Existing compaction-removal, issue112/113, cloud design and web work
  remain in place and uncommitted.
- Free space on the host volume: about106GiB before,346GiB after (~240GiB gained).
- No tests, benchmarks, rebuild, Docker pruning, release or deployment performed.
  Main target/dependency caches and prepared fixture/input directories were kept.
  Existing fixture damage was not repaired or requalified by this cleanup.

## What remains recoverable, and what was intentionally dropped

Source-only recovery backup:
`/Users/yifanxu/Ephemeral-AI-Lab/worktree-source-recovery-20260912T031258Z`
(about1.0GiB; verified source archives, per-worktree HEAD/status/staged/unstaged
patches and source manifests). It includes code/harness scripts and source
snapshots, not a complete benchmark evidence archive. The initial full-contents
archive attempt was abandoned and removed when the owner authorized dropping
raw evidence. No raw run/binary preservation claim is made.

**Raw timing/proof receipts, benchmark binary/image archives, sample Stores and
other artifacts in the deleted roots are intentionally gone.** Historical reports
remain as summaries, but references into those roots no longer resolve. Those
reports cannot now be independently audited solely from the retained local
artifacts, or reused as exact-binary current qualification. Future claims require
new measurement/verification. Old PASS/waiver/failure values are not rewritten;
their artifact-availability status is now DROPPED by owner authorization.

The machine-readable exact list and completion checks are in
[retirement-manifest.json](retirement-manifest.json). This is a deletion record,
not a reconstruction of the deleted evidence.

## Deleted roots

- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-cold-contract-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-cold-metadata-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-hardening-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-terminal-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage3-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-db-inspection-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-family-campaign-20260910T210014Z`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-final-tree-rca-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-fingerprint-index-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-index-attribution-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-init-breakdown-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue101-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue102-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue104-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue105-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue109-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue111-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue90-runs`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95-runs`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-metadata-insert-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-metadata-proof-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-namespace-content-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-namespace100-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-real-source-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-repeated-source-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-repository-history-recheck-20260910T210014Z`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-requalification-orchestration-20260910T205721Z`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-restart-index-design-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-tail-query-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v015-smoke-evidence`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v015-tiny-history-runs`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-workspace-admission-runs`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/results`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/campaigns`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/binary-archive`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/image-archive`

## Next work

See [workflow-simplification-plan.md](workflow-simplification-plan.md). The infra
changes are studied, not implemented by this cleanup. Issue117 remains open for
the prospective workflow/rule changes and source consolidation.

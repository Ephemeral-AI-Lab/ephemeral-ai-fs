# Commit Stage 2 terminal evidence pointer (#111)

> **Status:** Custody registration for the Stage 2 terminal campaign. Written
> before the measurements it registers. Read-only registration; no measurement
> claim is made here. Outcome: see
> [commit-stage2-terminal-results.md](commit-stage2-terminal-results.md).

## Roots

| Role | Path |
|---|---|
| Terminal root (this campaign) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-terminal-evidence/20260912T180000Z` |
| Stage 2 hardening root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-hardening-evidence/20260912T120000Z` |
| Stage 2 root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z` |
| Stage 1 root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z` |
| Baseline root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z` |

No archived script, analyzer, validator or finalizer was executed in place. No
archived tree was modified. Historical inconsistencies found during the audit are
recorded, not repaired.

## Source custody at freeze

- Main HEAD `91bebcc4d906c7c4cbef06fc80834b1ae3382a9f`; worktree dirty with the
  preserved compaction-removal treatment, `issue112/`, `issue113/` and untracked
  `web/`.
- `custody/before-head.txt`, `custody/before-status-all.txt`,
  `custody/before-tracked-all.patch`, `custody/before-untracked-hashes.txt`.
- Main worktree seal at freeze in `custody/before-seals.txt`.

## Arms

Each arm is an independent `git worktree` snapshot of this repository with link
count 1 on every file. No hard links, no shared `target/`, no shared writable
state.

| Arm | Source commit | Product seal | SOURCE_SEAL | binary sha256 | image tag | image id |
|---|---|---|---|---|---|---|
| A | `15e3d48e0` | `760eb0f2093488a6…` | `df636ab1dd4e8906…` | `1f3bbaff89eb99fb…` | `layerfs-bench-infra:df636ab1dd4e8906` | `sha256:b5801412309b952e…` |
| B | `a6f25eafe` | `a608cd4edd251615…` | `312235380f6609df…` | `f871b1173e01431b…` | `layerfs-bench-infra:312235380f6609df` | `sha256:d1dc3a7217ce83be…` |
| C | `259a80a4b` | `a54ef6e3f6d94fb1…` | `ed9b05c634818d08…` | `4f167e9164f69ec2…` | `layerfs-bench-infra:ed9b05c634818d08` | `sha256:d532c626f055b7f6…` |

Full per-arm identity JSON is `arm-identity-<arm>.json`; build logs and exit codes
are `build-host-<arm>.log` and `build-image-<arm>.log`.

## Campaign artefacts

| Artefact | Purpose |
|---|---|
| `arm-A/`, `arm-B/`, `arm-C/` | independently owned source snapshots |
| `build-arm.sh`, `build-all.out`, `build-host-*.log`, `build-image-*.log` | builds and exit codes |
| `probe/arm-<arm>/`, `probe-build*.log`, `route-probe.py`, `route-probe-<cell>.json` | compiled route-dispatch audit |
| `sequence-*.json` | frozen sequences |
| `cells/<sequence>/stage2-<arm>/…` | raw cells |
| `collect.py`, `analyze.py` | adapted collector and analyzer |
| `custody/` | pre-work head, status, diff and untracked hashes |

# Commit Stage 3 evidence pointer (#111)

> **Status:** Custody registration for the Stage 3 spill-scaling campaign. Written
> before the measurements it registers. Read-only registration; no measurement
> claim is made here. Outcome: see
> [commit-stage3-results.md](commit-stage3-results.md).
> Contract: [commit-stage3-contract.md](commit-stage3-contract.md).

## Roots

| Role | Path |
|---|---|
| Stage 3 root (this campaign) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage3-evidence/20260912T000000Z` |
| Stage 2 terminal root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-terminal-evidence/20260912T180000Z` |
| Stage 2 hardening root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-hardening-evidence/20260912T120000Z` |
| Stage 2 root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z` |
| Stage 1 root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z` |
| Baseline root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z` |

No archived script, analyzer, validator or finalizer is executed in place. No
archived tree is modified. Every attempt in this root is append-only.

## Source custody at freeze

- Main HEAD `987af93671e60b885504acc056ab5dbaf4f11d89`; worktree dirty with the
  preserved compaction-removal treatment, `issue112/`, `issue113/` and untracked
  `web/`.
- `custody/before-head.txt`, `custody/before-status-all.txt`,
  `custody/before-tracked-all.patch`, `custody/before-untracked-hashes.txt`.
- Main worktree seal at freeze in `custody/before-seals.txt`.

## Arms

Each arm is an independent `git worktree` snapshot of this repository with link
count 1 on every file: no hard links, no shared `target/`, no shared writable
state. The only intended product difference is the declared spill treatment.

| Arm | Role | Treatment |
|---|---|---|
| A | control | current accepted optimized product + preserved dirty compaction-removal treatment |
| B | candidate | A + the Stage 3 tiered spill runs, byte-identical elsewhere |

Per-arm product seal, source seal, compilation seal, binary sha256, image tag and
immutable image id are recorded in `arm-identity-<arm>.json`, `identity-<arm>.json`,
`binary-sha256-<arm>.txt`, `image-<arm>.txt` and `image-id-<arm>.txt`, with build
logs and exit codes in `build-host-<arm>.log` and `build-image-<arm>.log`.

The retained harness module `commit_baseline.rs` and its `main.rs` dispatch are
copied byte-identically into both arms from the Stage 2 terminal arm-C snapshot
(read-only) and sealed per arm; the harness is not part of the tracked product.

## Campaign artefacts

| Artefact | Purpose |
|---|---|
| `arm-A/`, `arm-B/` | independently owned source snapshots |
| `build-arm.sh`, `build-all.out`, `build-host-*.log`, `build-image-*.log` | builds and exit codes |
| `sequence-*.json` | frozen sequences |
| `cells/<sequence>/stage3-<arm>/…` | raw cells, one fresh Store and container per entry |
| `collect.py`, `analyze.py` | adapted collector and analyzer |
| `custody/` | pre-work head, status, diff and untracked hashes |
| `evidence-manifest.json` | hashes of every retained evidence file in the declared scope |
| `custody/fixture-damage.json` | the two missing 100 MB fixture payload files and both validator receipts |
| `custody/per-cell-store-bytes.json` | byte totals of the per-cell Store copies pruned from evidence scope |

**Declared evidence scope.** Per-cell `store/` copies and `layerfs-runtime/` scratch
are reconstructed artifacts, not evidence: every derived number comes from
`result.json`, the merged `output.log`, `command.json`, `exit.json` and
`cleanup.json`, all of which are retained. Their byte totals are recorded in
`custody/per-cell-store-bytes.json` before pruning (84 920 MiB across 279
directories), which was required at 99 % root-volume capacity. The two arm build
trees remain excluded and are identified by `arm-identity-<arm>.json`.

**Fixture custody failure: repaired.** The immutable `namespace-100000` fixture lost
two 100 MB payload files at `2026-09-12T02:13:32Z`; `fixture-validation.json` (before)
passes and `fixture-validation-final.json` (after the damage) fails. The content is
produced by a deterministic generator, so it was regenerated, proved reproducible
against two independent generations and the registered digests, and re-validated:
`fixture-validation-restored.json` reports `VALIDATION PASS` / `VERIFIED_IDENTITY`
with the registered digest `6fc793a9…1ac80a7e`. The damaged tree is retained in
quarantine (`benchmark-results/host-store/fixtures/.damaged-…-20260912T023240Z`) and
the run it rejected is retained as failed evidence. Both public campaigns therefore
measured byte-identical fixture content. Full detail: `custody/fixture-damage.json`
and `commit-stage3-results.md` §5.3.

# Commit Stage 2 evidence pointer (#111)

> **Status:** Custody registration for the Stage 2 campaign. Complete: the
> campaign ran end to end. See
> [commit-stage2-read-results.md](commit-stage2-read-results.md) for the outcome.
> Read-only registration; no measurement claim is made here.

## Roots

| Role | Path |
|---|---|
| Stage 2 evidence root (this campaign) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z` |
| Stage 1 root (read-only, unchanged) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z` |
| Completed baseline root (read-only) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z` |

No archived script, analyzer, validator or finalizer was executed in place. The
Stage 2 root was created new and populated by copy; the Stage 1 root and all
earlier roots remain byte-identical.

## Source custody at freeze

- Main HEAD `dcc3b939b6f9780e3600a15f3c6a7c8c94f11ede`; worktree dirty.
- Full preexisting tracked diff (`git diff HEAD`) sha256
  `44fdf679f24deead8e5f0d7dc8b2d0dd3a06346d56d8477058ebc69db0f6c2cc` — identical
  to the Stage 1 record, so the retained compaction-removal treatment is
  byte-identical.
- Untracked `compaction-removal.md`, `issue112/`, `issue113/` hashes identical to
  the Stage 1 record (`custody/before-untracked-hashes.txt`).
- Pre-modification seals of the working repository: SOURCE_SEAL
  `371d5dc40336492ed4aa969f4d210536bfbf740a8e0ac0f8a7c44e1ada1ac38b`, PRODUCT_SEAL
  `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`.

## Isolated arms

Each arm is an independent self-contained repository created inside this root by
copying the Stage 1 isolated source (which already carries the campaign harness
`benchmark/fs-bench-pro/src/commit_baseline.rs`, sha256
`9128dfe2ca27e3972679b4093f686a25ab319100f1563fde17f872d17951ffdb`) and then
`git init` + one base commit. The archived Stage 1 worktree metadata is not
reused or rewritten.

| Arm | Tree HEAD | SOURCE_SEAL | PRODUCT_SEAL |
|---|---|---|---|
| `stage2-control` | `52281435cbb29bf7f7ea8a6220488c6c2d0568be` | `490938083f8a7d7ecec166ae2e20c7287d0f7c1e890c504fd5c5c05217be3a8e` | `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932` |
| `stage2-control-diagnostic` | `e412c7538c80791fa6f250a704b36e570caad789` | `6ca0b4576927d97459907c924454e29d4951f80204188b5766d6fd0da4ed7520` | `45488fd2eb6081a1f1686d5a12cd6cbf09a7aafb8cac0eba7e28e276bc9492a2` |
| `stage2-candidate` | `d8a6eb8f77935e507fd44c1c11eeb41bfff12a0f` | `490938083f8a7d7ecec166ae2e20c7287d0f7c1e890c504fd5c5c05217be3a8e` (base) | `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932` (base) |
| `stage2-candidate-diagnostic` | `d2d62536fbbf958b4231b5fac519b85890211eeb` | `6ca0b4576927d97459907c924454e29d4951f80204188b5766d6fd0da4ed7520` (base) | `45488fd2eb6081a1f1686d5a12cd6cbf09a7aafb8cac0eba7e28e276bc9492a2` (base) |

The control and candidate source seals reproduce the Stage 1 plain seal exactly,
and the two diagnostic seals reproduce the Stage 1 diagnostic seal exactly, so
the Stage 2 arms start from the same measured treatment. The candidate arms are
byte-identical to their controls until the Phase B treatment is applied, which is
recorded separately after the Phase A decision.

Full per-arm seals, including the path-dependent compilation and dependency
seals and native build identities, are in `custody/tree-seals.json` and in each
arm's `target/release/fs-benchmark-pro.identity.json`.

## Measured arms at completion

| Arm | SOURCE_SEAL | PRODUCT_SEAL | binary sha256 | image tag | image id |
|---|---|---|---|---|---|
| `control` | `490938083f8a7d7ecec166ae…` | `760eb0f2093488a6…` | `6f7272ba4d50ef1b…` | `layerfs-bench-infra:490938083f8a7d7e` | `fe4b0de076452698…` |
| `control-diagnostic` | `a614765dc559c8d597d3d458…` | `5a32b6fb1152266d…` | `e9fc1bb47f2f5e8c…` | `layerfs-bench-infra:a614765dc559c8d5` | `9074109aec1d8375…` |
| `candidate` | `4c46c963298e1eb5d4dacad0…` | `a608cd4edd251615…` | `0e75635f297344ac…` | `layerfs-bench-infra:4c46c963298e1eb5` | `437762a8b35072de…` |
| `candidate-diagnostic` | `e67b60d179ee164ab18b9064…` | `7a9bfe71ca696a55…` | `b4fefd9aaa9ac260…` | `layerfs-bench-infra:e67b60d179ee164a` | `ab0a79f937318c17…` |

The two diagnostic arms differ from their controls by exactly the instrumentation
files and from each other by exactly the same five product files as the plain
pair (`phasea-instrumentation.patch`, `candidate.patch`,
`candidate-diagnostic.patch`).

## Campaign artefacts

| Artefact | Purpose |
|---|---|
| `sequence-phasea.json` (6 cells) | frozen Phase A feasibility screen |
| `sequence-paired-plain.json` (30 cells) | frozen paired plain measurement |
| `sequence-paired-diagnostic.json` (12 cells) | frozen paired diagnostic attribution |
| `dev/dev-candidate-k100` | selected development smoke, `admission_eligible=false` |
| `attempts/aborted-paired-diagnostic` | cells of the aborted first diagnostic run (collector cell-path collision), retained unmodified |
| `cells/<sequence>/stage2-<arm>/…` | raw cells, one fresh Store and container each |
| `collect.py`, `analyze.py`, `validate_fixtures.py`, `finalize.py`, `dev_smoke.py` | collector, analyzer, fixture validator, manifest, smoke |
| `summary.json`, `analysis.log` | derived summaries from retained raw evidence |
| `build-arm.sh`, `build-host-*.log`, `build-image-*.log` | build entrypoints, logs and exit codes |
| `fixture-validation.json` | full fixture identity validation (`VALIDATION PASS`) |
| `candidate.patch` | exact Phase B product treatment for promotion review |
| `evidence-manifest.json` | sha256 of every retained file |

Every campaign cell passed its gates; the analyzer reported zero problems. The
outcome, the paired statistics, the mechanism counters, the resource and
correctness evidence and the limitations are in
[commit-stage2-read-results.md](commit-stage2-read-results.md).

## Fixture

The immutable original pseudorandom `namespace-100000` fixture
(`100000` files, `1000` directories, `500000000` logical bytes, digest
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`) is consumed
from the main repository's prepared cache at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/fixtures`,
read-only, and validated by `validate_fixtures.py` before each collection.

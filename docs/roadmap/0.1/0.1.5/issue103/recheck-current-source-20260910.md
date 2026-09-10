# Issue103 repository_history recheck on the current source (cc8025fcd + uncommitted compaction removal)

**stride-3 (53 states) and stride-1/full157 (157 states): PASS.**
Both profiles were run as one performance (history-build) run plus a separate
original-state verification run on the current source, with no compaction path
(ordinary/uncompacted). All states Created, zero presentation failures, all
states PASS on verification, cleanup PASS.

Executed 2026-09-10T21:44Z–22:11Z. This is a re-check of the
`repository_history` family on the current source (`cc8025fcd`, #109)
**including** the preserved uncommitted compaction-removal work.

**Unpaired single-sample comparison against the recorded v0.1.5 results in
#103/#100 across different source states; not a paired benchmark result. The
historical-access 11-case set is not re-runnable without the removed compaction
flow and remains pending.**

## Results

| Metric | stride-3 recorded (#103, ordinary pre-compaction) | stride-3 current | Current/recorded | full157 recorded (#103, ordinary pre-compaction) | full157 current | Current/recorded |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Save (history-build) work wall | 187.880254 s | 206.723243 s | 1.100 | 519.681190 s | 548.432 s | 1.055 |
| Verification work wall | 236.103555 s | 165.271453 s | 0.700 | 778.616960 s | 481.050 s | 0.618 |
| Ordinary allocated (before verification writes) | 65,056,768 B | 65,064,960 B | +0.0126% | 83,935,232 B | 83,943,424 B | +0.0098% |
| Ordinary apparent (before verification writes) | — (not in #103 table) | 64,630,884 B | — | 83,697,664 B (authenticated #104 master) | 83,644,516 B | −0.0635% |
| Allocation after verification writes | 47,251,456 B (compacted #103 case) | 65,064,960 B | — | 56,524,800 B (compacted #103 case) | 83,943,424 B | — |
| States Created / verified | 53 / 53 PASS | 53 / 53 PASS | — | 157 / 157 PASS | 157 / 157 PASS | — |
| Path-states verified | 306,861 | 306,861 | exact | 904,143 | 904,143 | exact |
| Logical bytes verified | 1,676,767,835 | 1,676,767,835 | exact | 4,936,693,030 | 4,936,693,030 | exact |
| Compacted allocated (HISTORICAL ONLY) | 46,202,880 B | not applicable (no compaction) | — | 55,476,224 B | not applicable (no compaction) | — |
| Compaction cost (HISTORICAL ONLY) | 339.133 s | not applicable | — | 626.313062 s | not applicable | — |

Historical references from #100 (not re-measured): offline stride3
54,382,592 B; offline full157 65,957,888 B; Git53 49,332,224 B; Git157
56,373,248 B. These are compacted/optimized prototypes and are not comparable
to the current ordinary Store; they are listed as clearly-labeled historical
context only. The non-re-runnable historical-access 11-case set over the frozen
full157 store remains untouched at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/full157-integrated-1/deepseek-full/frozen-measured-store/store.sqlite`.

## Interpretation

- **Storage semantics are preserved.** Ordinary allocated bytes match the #103
  pre-compaction baselines to within +8,192 B (+0.013% stride3, +0.010%
  full157) — one page-level difference, well inside the <1% expected
  allocation drift. The verified path-state and logical-byte totals are
  **exactly** the recorded values (306,861 / 1,676,767,835 and 904,143 /
  4,936,693,030), and every original state verified against its sealed oracle.
  A fresh single-run Store is not byte-identical to the #104 master
  (page-level nondeterminism); this is not a semantics change.
- **Timing deltas are unpaired observations, not measured speedups.**
  Save-phase wall was +10.0% (stride3) and +5.5% (full157) versus the recorded
  #103 single samples; verification wall was 0.700× and 0.618×. The host is an
  interactive shared machine (load averages ~5–8 during these runs), the #103
  source state differed (compaction enabled; candidate `80bc4892`), and n=1 vs
  n=1. No rerun was performed for better numbers. The save-phase observation is
  directionally consistent with the small shared-path admission costs seen in
  the companion #102 requalification, but neither direction is a
  controlled/paired claim.

## Acceptance per profile

- stride-3: performance PASS, 53/53 Created, 0 presentation failures, cleanup
  PASS; verification PASS, 53/53 original states, cleanup PASS.
- full157: performance PASS, 157/157 Created, 0 presentation failures, cleanup
  PASS; verification PASS, 157/157 original states, cleanup PASS.
- Phase budgets 14,400 s each were not approached.

## Build, source and custody identities

| Item | stride-3 run | full157 run |
| --- | --- | --- |
| Source seal | `17334f5e6900bdaf73e1c49ccb9bc0ad7ec3b710eef3ef5efef9b9841c6975ba` | same |
| Product seal | `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4` | same |
| Host binary sha256 | `dbbf1259186c60122286bb2a0503d6dcccfd0a41f49eb82799b5c3a4d6e6a36b` | same |
| Host build | warm no-op `--build-host` (0.23 s; binary byte-identical to Task 1 / #109's retained candidate) | same |
| Linux image | `layerfs-bench-infra:17334f5e6900bdaf`, inspect Id `sha256:c5947d926e08e7d035d87c3352f467965478d1d8a429c2ad53419880752da680` | same |
| Host commit recorded in identity | `f4c67f376904278639ae095283f5ecd214466130` | same |
| Fixture manifest | `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271` (157 checkpoints; tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`) | same |
| Performance store sha256 (as measured) | `eb9f40f0ba0ac14ca6f8f22b227916c2bc36e94ea402d5b4bf2c62f3bf061b72` | `576d7ff8f55d4fe733c14c9034f04b1680fb8a7a2c44cef7d601e5fbca098767` |
| Store on disk after verification writes | `0519f327fdca878e90ad2ab192a971a489fa884e33b57e25bd32f8fa8724ab40` (64,638,976 B) | `482410a3b793ae40526e252d64375237a1039e813b356dcec1bab9033d16b17a` (83,681,280 B) |
| Performance manifest entries | 116 | 324 |
| Verification manifest entries | 282 | 802 |
| Preparation wall | 22.16 s | 68.54 s |

Preflight: `runner.py --family repository_history --self-check` → PASS
(profiles 3, states [157,53,17]); product-free runner unit tests → 57 OK.
Free disk ≥317 GiB (≥50 GiB reserve). Ambient load observed 5.9–7.8 at start;
recorded in evidence.

Exact commands:

```sh
cd /Users/yifanxu/Ephemeral-AI-Lab/layerfs
python3 -m unittest discover -s benchmark/fs-bench-pro/shared -p 'test_*.py'
python3 benchmark/fs-bench-pro/shared/runner.py --family repository_history --self-check
python3 -u benchmark/fs-bench-pro/shared/runner.py --build-host
python3 -u benchmark/fs-bench-pro/shared/runner.py --build-image

IMAGE=layerfs-bench-infra:17334f5e6900bdaf
RUNNER=benchmark/fs-bench-pro/shared/runner.py
EV=/Users/yifanxu/Ephemeral-AI-Lab/layerfs-repository-history-recheck-20260910T210014Z

# stride-3
python3 -u "$RUNNER" --family repository_history --profile stride-3 --image "$IMAGE" \
  --output "$EV/stride3-integrated-current"
python3 -u "$RUNNER" --family repository_history --profile stride-3 --image "$IMAGE" \
  --storage-verify-run "$EV/stride3-integrated-current"
# stride-1 / full157
python3 -u "$RUNNER" --family repository_history --profile stride-1 --image "$IMAGE" \
  --output "$EV/full157-integrated-current"
python3 -u "$RUNNER" --family repository_history --profile stride-1 --image "$IMAGE" \
  --storage-verify-run "$EV/full157-integrated-current"
```

No `--storage-compact` was passed (the path is removed).

## Evidence paths

- Evidence root (create-once):
  `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-repository-history-recheck-20260910T210014Z/`
- Logs: `stride3-performance.log`, `stride3-verification.log`,
  `full157-performance.log`, `full157-verification.log`,
  `task2-selfcheck.log`, `task2-unit-tests.log`.
- Runs: `stride3-integrated-current/`, `full157-integrated-current/` (identity,
  performance/verification results, manifests, summaries, per-state records,
  measured stores).
- Identities: `task2-host-binary.sha256`, `task2-host-identity.json`,
  `task2-image-id.txt`, `task2-image-labels.json`, `task2-disk.txt`,
  `task2-load-start.txt`.
- Orchestration record:
  `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-requalification-orchestration-20260910T205721Z/orchestration.md`

No write occurred into `layerfs-issue103-evidence/`, the #104 campaign
directory, or any `layerfs-issue109-evidence/` directory; the retained #103
frozen store is untouched. No product-code change, release, tag, deployment,
campaign restart, or unrelated cleanup was made. No file under `crates/`,
`tools/` or `benchmark/` was edited during the run.

## Deviations

1. `--build-host` and `--build-image` were re-run as required preflight against
   the post-Task-1 docs commit. The source seal was unchanged, the host binary
   was byte-identical (warm no-op), and only the image `revision`/`source-tree`
   labels changed (inspect Id `sha256:0ccd9886…` → `sha256:c5947d92…` for the
   same `17334f5e6900bdaf` tag and identical cached layers). No rebuild of
   product bytes occurred.
2. The historical-access 11-case set is unavailable without the removed
   compaction flow (as anticipated by the prompt) and remains pending.

## Beyond scope / pending

- `historical-access` 11 cases (removed compaction prerequisite).
- stride-10 profile, the #104/#102 campaigns, and any compaction were not run.
- Companion #102 requalification (Task 1) completed with
  `store_footprint/store-footprint-metadata-cardinality-100000` failing with a
  product error and the known `dedup_branch_history` timing miss; its evidence
  root is
  `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-family-campaign-20260910T210014Z/`.

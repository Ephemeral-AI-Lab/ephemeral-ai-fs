# Issue118 terminal outcome table

Status: terminal reconciliation of the applicable #118 work. Every row names its
disposition, the exact treatment it was measured on, and where the raw evidence
lives. Nothing here is inherited from retired historical evidence.

Evidence root: `benchmark-results/host-store/issue118/20260912/`.

## Exact treatments

| Name | Commit | Host binary SHA256 | Source seal | Product seal | Image |
|---|---|---|---|---|---|
| `fsync-qualified` (fsync optimization) | `f8fa59fab` | `440ae2c4ce03741676a2845becb9b1661dde341079cc62faad0998847d33a4e5` | `8053ccba0350395727a01788c7b36de508d4202de27095028473c94be226cc74` | `c2d5022b8be288304e3ca092eadc543eae8bdf4089198309813157a38e38951b` | `sha256:a04cb1d9cd8c2696fdd3b8a4f7deffe2c55c2d38f0aced40be0c213b2227f0e7` |
| `qualified-v2` (pre-fsync control) | pre-`f8fa59fab` | `dc07a23f98a77c4520c2f91464359c899167d39240ef50d669d18eecc7e3750d` | — | — | `layerfs-bench-infra:e162aecd2b351811` |
| `final-treatment` (current source) | `2dbc75ecb` | `6693224ee0939a7b6e18489259cdd08ab380a1a0f2877d8a4c6caf2334abafc1` | `e46df7b0463158def12ad4768eab8f7b795c917ada24906ad3804dac3a097dd1` | `08176ecae86ea6e9cf63a053061dc0d34603996be21f2ade603ea3a59c7dabcd` | `sha256:94b3eed3078c90f6ea4573d6806cc2fc1125c90c7e7f044f3168af1dc9a4cf00` |

The product executable is byte-identical across `fsync-qualified` and
`final-treatment` (`440ae2c4…` at `f8fa59fab` → `6693224e…` after the #116
repair); the source/product seals differ because the #116 product change and the
harness seals are part of the identity.

## Terminal rows

| # | Required work | Disposition | Treatment | Evidence |
|---|---|---|---|---|
| 1 | #117 infrastructure simplification | PASS (prior run) | `fsync-qualified` lineage | `infra-focused-checks.log`, `infra-pair-*.json`, `binary-archive/` |
| 2 | Stage3 highest-tier merge / failure / resource proofs | PASS | prior + current | `stage3/`, `physical-reservation-root-cause.md` |
| 3 | Edit attribution, dominant fix, paired qualification | PASS | qualified-v1/v2 | `edit-attribution.md`, `edit-pair-summary.json` |
| 4 | Reopened-history exact tail recovery + public sequence proof | PASS | qualified-v2 | `reopen-retention-explanation.md`, `reopen-retention-public/` |
| 5 | Cold Init and four namespace tiers | PASS with explicit owner **WAIVER** of the 2.7 s absolute target and a documented wall WARN (paired wall +1.039%, CPU −2.855%) | qualified-v2 | `cold-namespace-results.md`, `owner-cold-target-waiver.json` |
| 6 | Fsync optimization (bounded batch + reservation tail) | PASS | `fsync-qualified` | `fsync-results.md`, `fsync-qualified/` |
| 7 | Unrelated-history500 **<15 s** (unwaived) | PASS 13.791443406 s (single full-workload screen) | `fsync-qualified` | `fsync-qualified/500-candidate/perf.jsonl` |
| 8 | 18 remaining stage4 selections | PASS: 18/18 complete public samples + 18/18 independent proofs | `fsync-qualified` | `remaining-shared/plan.json`, `remaining-shared/*/perf.jsonl`, `remaining-shared/*-proof/verification.json` |
| 9 | Affected recheck of the three SDK-edit selections after the #116 source change | PASS: 3/3 performance + 3/3 proofs | `final-treatment` | `affected-rerun/` |
| 10 | Ordinary full157 performance (stride-1, 157 states) | PASS: 766.577 s complete, 157/157 steps | `final-treatment` at harness state matching the current tree | `full157-candidate-1/performance-summary.json`, `deepseek-full/performance-result.json` |
| 11 | Ordinary full157 same-Store verification against all 157 original oracles | PASS: 712.570 s | same Store | `full157-candidate-1/verification-summary.json` |
| 12 | Frozen measured Store identity | PASS: `0e767a7f4078f040de165e6138b64ed7ed5ce1ea1f1c88ed8fc8df6f3cc523f5` | — | `full157-candidate-1/deepseek-full/frozen-measured-store/` |
| 13 | Ordinary storage physical census (#107 attribution) | PASS: allocated 83,935,232 B; apparent 83,525,632 B; pack bytes 75,695,515 B; file-content pack 67,084,005 B; metadata pack 8,611,510 B; SQLite non-pack 7,830,117 B; filesystem allocation difference 409,600 B | same Store | `full157-candidate-1/census.json` |
| 14 | Matching Git157 comparison | PASS: 157 commits/trees verified read-only, no repack or checkout | existing packed control | `full157-candidate-1/git157-proof/result.json` |
| 15 | 11 historical-access cases | PASS: 11/11, each complete envelope 2.33–3.18 s, deadline and cleanup PASS | `final-treatment` | `full157-candidate-1/access-performance/*/result.json` |
| 16 | 11 separate access proofs | PASS: 11/11 independent verifications | `final-treatment` | `full157-candidate-1/access-verification/*/result.json` |
| 17 | #116 Phase-1 capability/resource audit published before capacity changes | PASS: source-linked inventory, capability matrix, bounded public-API reproductions, dispositions | `f8fa59fab` + `ead812e78` | `docs/roadmap/0.1/0.1.5/issue118/issue116-audit.md` |
| 18 | #116 bounded repair: >4096 counted pending edits to one file | PASS: 10,000 counted edits accepted where budgets fit; the redundant counter ceiling removed, real budgets retained, counter widened to `u64` with checked arithmetic | `ead812e78` | `issue116-audit.md` §5, `issue116-audit/probe-fixed-a.log`, `crates/layerfs-workspace/tests/issue116_capacity.rs` |
| 19 | #116 default-budget public K32000 spill coverage | **BLOCKED, explicit**: `namespace-100000 --sequence 32000` fails at accepted edit 5,641 with `workspace piece allocation limit`; the sequence needs distinct files and the splice shape costs 384 B/file in the 2 MiB budget, admitting exactly 5,461 files (K5461 PASS at charge 2,097,024 B, K6000/K32000 FAIL). Reaching the 15,873-key spill-scale crossing needs ≈5.8 MiB, i.e. a compact pending-splice representation, not a quota increase. | `final-treatment` | `issue116-audit.md` §7, `k32000-spill-1/`, `k6000-diag3/`, `k5461-boundary/`, `k5000-boundary/` |
| 20 | #116 representative pinned package installation + Commit/reopen | **NOT_RUN, explicit**: the benchmark container carries only `layerfs-daemon`, `layerfs-fuse`, `fs-benchmark-workload`; no package manager or interpreter exists in it, and adding one would be an unrelated image/network change. The container execution surface itself is qualified by the registered 500-execution reliability proof. | — | `workspace-exec-500-compact-v2-proof` (registered proof, prior run) |
| 21 | Exact identities, command costs, compact retention, cleanup | PASS: this table plus per-run `commands.json`/`tail-commands.json`; failures and superseded attempts retained | — | `remaining-shared/plan.json`, `full157-candidate-1/tail-commands.json`, `k-boundary.log` |
| 22 | Owner waivers and permitted warnings explicit | PASS: cold Init 2.7 s owner-WAIVED; Stage2 K10 50/31 ms owner-WAIVED; cold-openat paired WARN retained; active-K100 Commit +0.311 ms below the material rule; prior `unused_mut` warning documented | — | `execution.md`, `scope.md`, `fsync-results.md` |

## Retained failures and rejected evidence

| Item | Why retained |
|---|---|
| `stage4/…unrelated-500…` pre-fsync attempts (23.228–24.120 s) | Original unwaived failure on the pre-fsync control and candidate |
| `image-freshness-failure.json` | Falsely fresh image rejected; its two runs are not candidate qualification |
| `fsync-batch-pressure-before.*` | Real before-fix starvation failure |
| `mounted/proof1/verification.json` | Obsolete input identity rejected before any product workload |
| `reopen-k1000-screen/` | Harness-only observer failure (`database is locked`), not a product failure |
| `issue116-audit/probe.log` (probe A/B before the repair) | The measured pre-repair rejections |
| `k32000-spill-1/`, `k6000-diag*`, `k6000-diag3/` | The blocked spill-scale attempts, including the preserved owner-side error |
| `affected-rerun/overwrite-proof` first attempt | Superseded by the corrected-identity proof; original attempt preserved as `INCOMPLETE` in the run log |

## #107 follow-up decision

The current ordinary baseline is published above (row 13) together with the
matching Git157 control (row 14). No new ordinary-storage mechanism is
implemented in #118. A future #107 experiment must prospectively declare its
mechanism, quantitative target (the ≥1% ordinary allocated-byte floor is a
proposal, not a retroactive PASS) and latency/resource constraints, and must
derive any saving from ordinary Init/Commit — never compaction, VACUUM or
post-workload repacking.

## Linked-issue outcome

| Issue | Outcome |
|---|---|
| #118 (umbrella) | Applicable optimization, qualification, ordinary-storage, access and #116 audit/repair rows are terminal as tabulated. Two rows are explicitly unsatisfied and must not be read as PASS: the default-budget public K32000 spill coverage (row 19, BLOCKED with its exact boundary) and the representative pinned package installation (row 20, NOT_RUN with its reason). |
| #116 | Phase-1 audit published; the redundant per-file pending-edit counter ceiling removed with real budgets retained; the changed-file ceiling located exactly and recorded as a representation follow-up; spill-scale public coverage blocked and documented. |
| #100/#107 | Current ordinary attribution and matching Git distance published; no new storage mechanism claimed. |
| #108/#112/#114/#102/#110 | Deduplicated into the single qualification above; no historical matrix rerun. |
| #111 | Stage3 spill and reopened-history work complete; the deferred spill-scale public selection is blocked as row 19. |
| #117 | Infrastructure simplification complete and reused throughout. |

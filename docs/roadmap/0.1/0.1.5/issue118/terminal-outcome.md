# Issue118 terminal outcome table

Status: terminal reconciliation of the applicable #118 work. Every row names its
disposition, the exact treatment it was measured on, and where the raw evidence
lives. Nothing here is inherited from retired historical evidence. Corrections to
the earlier record are in [record-corrections.md](record-corrections.md).

Evidence roots: `benchmark-results/host-store/issue118/20260912/` (earlier work)
and the runs listed per row.

## Exact treatments

| Name | Commit | Host binary SHA256 | Source seal | Product seal | Image |
|---|---|---|---|---|---|
| `qualified-v2` (pre-fsync control) | pre-`f8fa59fab` | `dc07a23f98a77c4520c2f91464359c899167d39240ef50d669d18eecc7e3750d` | — | — | `layerfs-bench-infra:e162aecd2b351811` |
| `fsync-qualified` | `f8fa59fab` | `440ae2c4ce03741676a2845becb9b1661dde341079cc62faad0998847d33a4e5` | `8053ccba…` | `c2d5022b…` | `sha256:a04cb1d9…` |
| `408f3e2a…` full157/access producer | `226cfeea9` + dirty tree | `408f3e2a762e059a5efc8f3634355ad259de16a95624427610e8f94fb7b7fc64` | `0574853ae6cdb50c9be79316417146cb10e72d4fd08ddb9dab8038316a9badfb` | `6e9e2840d7478c35e4370ab38c009914dd7ec43921aab4322624840c9e822768` | `sha256:08082f1af0927d64af9fa525e7dee041f02bdee0d8e84baf4cf6ecbfcd22660d` |
| `6693224e…` later treatment | `2dbc75ecb` | `6693224ee0939a7b6e18489259cdd08ab380a1a0f2877d8a4c6caf2334abafc1` | `e46df7b0…` | `08176eca…` | `sha256:94b3eed3…` |
| **final treatment** | `345f75ad9` + this work | `b5f089ebcd6fa2fa939feb9bccc5300ca8ede798820fe8fd9aace8799fe4ec0b` | `5ef2ec80620a9402693bc08af5024e64217103b88d55cb5811e7b35d4cac083b` | see run identities | `sha256:2fd77e0674de65519685b826a6028c74a02272343efafcaccefc923374ec3c58` |

Different executable hashes are never byte-identical, and a reused path proves
nothing about which bytes ran. Row 10 onwards therefore names its own producer.
The final treatment's source tree differs from the qualified binary only in
`crates/layerfs-workspace/tests/issue116_capacity.rs` (a test file); the product
binary hash is unchanged.

## Terminal rows

| # | Required work | Disposition | Treatment | Evidence |
|---|---|---|---|---|
| 1 | #117 infrastructure simplification | PASS (prior run) | `fsync-qualified` lineage | `infra-focused-checks.log`, `infra-pair-*.json` |
| 2 | Stage3 highest-tier merge / failure / resource proofs | PASS | prior + current | `stage3/`, `physical-reservation-root-cause.md` |
| 3 | Edit attribution, dominant fix, paired qualification | PASS | qualified-v1/v2 | `edit-attribution.md`, `edit-pair-summary.json` |
| 4 | Reopened-history exact tail recovery + public sequence proof | PASS | qualified-v2 | `reopen-retention-explanation.md`, `reopen-retention-public/` |
| 5 | Cold Init and four namespace tiers | PASS with explicit owner **WAIVER** of the 2.7 s absolute target and a documented wall WARN (paired wall +1.039%, CPU −2.855%) | qualified-v2 | `cold-namespace-results.md`, `owner-cold-target-waiver.json` |
| 6 | Fsync optimization (bounded batch + reservation tail) | PASS | `fsync-qualified` | `fsync-results.md`, `fsync-qualified/` |
| 7 | Unrelated-history500 **<15 s** (unwaived) | PASS 13.791443406 s (single full-workload screen) | `fsync-qualified` | `fsync-qualified/500-candidate/perf.jsonl` |
| 8 | 18 remaining stage4 selections | PASS: 18/18 complete public samples + 18/18 independent proofs, **single-sample n=1 qualification, not n3 comparative evidence** | `fsync-qualified` | `remaining-shared/plan.json`, `remaining-shared/*-proof/verification.json` |
| 9 | Affected recheck of the three SDK-edit selections after the #116 source change | PASS: 3/3 performance + 3/3 proofs | `6693224e…` + `440ae2c4…` control | `affected-rerun/` |
| 10 | Ordinary full157 performance (stride-1, 157 states) | PASS: complete command 763.218 s, 157/157 steps | **final treatment** | `issue107-storage-full157/performance-summary.json`, per-step `deepseek-full/performance-step-*.json` |
| 11 | Ordinary full157 same-Store verification against all 157 original oracles | PASS: complete command 735.749 s, 157 states, 904,143 entries, 4,936,693,030 B | same frozen Store | `issue107-storage-full157/verification-summary.json`, `verification-result.json` |
| 12 | Frozen measured Store identity | PASS | — | `issue107-storage-full157/deepseek-full/frozen-measured-store/`, `census.json` (`store_sha256` `88b4fe70…`) |
| 13 | Ordinary storage physical census | PASS: allocated 83,951,616 B; apparent 82,583,552 B; pack overflow slack 1,222,187 B; pack rows 1,058 | final treatment | `issue107-storage-full157/census.json` |
| 14 | Matching Git157 comparison | PASS: 157 commits/trees, 110,081 objects, 904,143 entries, no repack or checkout; **allocation-layout WARN retained** (live 56,197,120 B vs recorded 56,373,248 B, unchanged apparent/pack bytes) | existing packed control | `issue107-storage-full157/git157-proof/result.json` |
| 15 | 11 historical-access performance cases | PASS: 11/11, complete commands 2.48–2.88 s inside the unwaived 15 s envelope, cleanup PASS | final treatment | `issue107-storage-full157/access-performance/*/result.json` |
| 16 | 11 separate access proofs | PASS: 11/11 independent verifications | final treatment | `issue107-storage-full157/access-verification/*/result.json` |
| 17 | #116 Phase-1 capability/resource audit published before capacity changes | PASS | `f8fa59fab` + `ead812e78` | `issue116-audit.md` (corrected) |
| 18 | #116 >4096 counted pending edits to one file | PASS: 10,000 counted edits accepted, **exact final contents asserted, Commit and reopen byte-compared** | final treatment | `issue116-probe-a-extended.log` (complete command 8.31 s) |
| 19 | #116 bounded compact pending representation | PASS: one equal-length overwrite of a committed base charges one 64-byte descriptor (72 B for a spool replacement) instead of three 128-byte nodes, and is encoded as one bounded wire splice | final treatment | `file_edit.rs`, `live_wire.rs`, focused tests, row 21 |
| 20 | #116 default-budget public `namespace-100000 --sequence 32000` | PASS: COMPLETE in one requested Commit, 32,000 edits, `edit_count=32000`, `edit_piece_count=92821`, `edit_piece_logical_charge=2048000` (= 32,000 × 64) against the unchanged 2,097,152 B budget; cleanup PASS. **TARGET_MISS is reported by the harness against its historical 15 s *family* product target (31.765 s); the harness itself labels that target "reporting-only … not a collection acceptance gate", and 32,000 SDK edits cannot meet a target calibrated for Init-scale namespace runs (K5,461 previously took 7.31 s, i.e. 1.34 ms/edit). Recorded as an explicit WARN, not hidden.** | final treatment | `issue116-compact-k32000/perf.jsonl`, `failure.log` |
| 21 | #116 independent Commit/reopen/content verification of the 32,000-edit route | PASS: 32,000 changed files verified before and after a fresh reopen, 100 unchanged-file samples, history snapshot PASS, cleanup PASS; complete verification command 68.576 s inside the 600 s scaled allowance | final treatment | `issue116-compact-k32000-proof/verification.json` |
| 22 | Default frontier capacity proof (B = 15,873) | PASS: `production_budget_spill_boundaries_stay_bounded` at exactly 2B (`batch=15873 count=31746 flushes=2 new_writes=63492 merges=1 deepest_level=1`), 4B and 8B | final treatment | `issue116-compact-k32000/default-budget-frontier-proof.log` (2.82 s) |
| 23 | Owner concurrency requirement: two running commands, mappings, descriptors, cwd and continued writes across Commit | PASS: both live-cut tests plus the container lifecycle test, each with the compact form and its conversion proven live (`bounded_charge=64 bounded_pieces=3 converted_charge=640 converted_pieces=5`) and same-workload barrier throughput measured (`pre_spins`/`barrier_spins`, `edit_ns`) | final treatment | `issue116-compact-live/live-cut-1.log`, `…/ordinary_writes_queue_during_mapped_commit.log`, `…/managed_container_lifecycle…log` |
| 24 | Pinned package workflow: deterministic project/lockfile, clean install and representative pinned update, each with Commit/reopen and content/metadata/module checks | PASS: pinned pip 24.0 bootstrap verified by SHA256 (`pip.whl: OK`); `--require-hashes --only-binary=:all:` install of idna 3.6 / packaging 23.2 / six 1.16.0 then a pinned update to idna 3.7 / packaging 24.0 / six 1.17.0; application run and installed-module inventory hashed in-session and after reopen; host-side committed-snapshot bytes and vendor listing verified for both sets; **no lifecycle script executes (wheels only)** | final treatment | `issue116-compact-live/pinned-package.log` (complete command 15.9 s) |
| 25 | #107 mechanism and measured ordinary-storage saving | **Mechanism PASS / frozen allocated-byte target FAIL (explicit)**: pack rows 3,457 → 1,058, SQLite pack overflow slack 2,142,701 → 1,222,187 B (−920,514 B), apparent Store 83,525,632 → 82,583,552 B (−1.13%); **allocated Store bytes 83,935,232 → 83,951,616 B (+0.02%)**, because the live file's allocation residue grew 409,600 → 1,368,064 B. The frozen ≥1.5 % target is missed and not waived. | final treatment | [issue107-result.md](issue107-result.md), [issue107-declaration.md](issue107-declaration.md), `issue107-storage-full157/census.json`, superseded attempt `issue107-coalesce-full157/` |
| 26 | Acceptance-record corrections and bounded applicability assessment | PASS | — | [record-corrections.md](record-corrections.md) |
| 27 | Exact identities, command costs, retained failures and cleanup | PASS: this table plus per-run `identity.json`, `*-summary.json`, `census.json`, run logs; every failed or superseded attempt retained | — | this document, `record-corrections.md` |
| 28 | Owner waivers and permitted warnings explicit | PASS: cold Init 2.7 s owner-WAIVED; Stage2 K10 owner-WAIVED; Git allocation-layout WARN; K32000 historical-family TARGET_MISS WARN; full157 single-sample +1.1 % wall WARN accepted by the owner; cold-openat paired WARN; active-K100 Commit +0.311 ms; prior `unused_mut` warning documented | — | `execution.md`, `scope.md`, `fsync-results.md` |

## Retained failures and rejected evidence

| Item | Why retained |
|---|---|
| `stage4/…unrelated-500…` pre-fsync attempts (23.228–24.120 s) | Original unwaived failure on the pre-fsync control and candidate |
| `image-freshness-failure.json` | Falsely fresh image rejected; its two runs are not candidate qualification |
| `fsync-batch-pressure-before.*` | Real before-fix starvation failure |
| `mounted/proof1/verification.json` | Obsolete input identity rejected before any product workload |
| `reopen-k1000-screen/` | Harness-only observer failure (`database is locked`), not a product failure |
| `issue116-audit/probe.log` (probe A/B before the repair) | Measured pre-repair rejections |
| `k32000-spill-1/`, `k6000-diag*`, `k5461-boundary/`, `k5000-boundary/` | The pre-repair capacity boundary and its rejected attempts |
| `affected-rerun/overwrite-proof` first attempt | Superseded by the corrected-identity proof; original attempt preserved |
| `issue107-coalesce-full157/` | First full157 of the mechanism: construction/verification-equivalent content but its image manifest was replaced by a later build, so the strict `--storage-verify-run` custody check could not reuse the identical image. Superseded by `issue107-storage-full157`; its census is retained as corroboration of the same mechanism. |
| shared-target stale `layerfs-content` artifact | Build-cache integrity finding with the exact probe and repair; see `record-corrections.md` §3 |
| `layerfs-workspace` `tiered_spill_partial_writes…` under default parallel test threads | Pre-existing process-global FD-count interference: the test passes in isolation and with `--test-threads=1` (75/75) but can observe another test's descriptor; not a product failure |

## Linked-issue outcome

| Issue | Outcome |
|---|---|
| #118 (umbrella) | **Remains OPEN.** Applicable optimization, correction, #116 representation/capacity, package and final qualification rows are terminal as tabulated. One required item is explicitly unsatisfied and must not be read as PASS: **#107's prospective allocated-byte target (row 25)**.  |
| #116 | Phase-1 audit corrected and published; the redundant per-file pending-edit counter removed; the route-specific changed-file ceiling repaired by a bounded compact pending representation with its wire form, focused proofs, live concurrency proofs, the exact 32,000-edit public route and independent verification; default-budget frontier crossings proven at exactly 2B. |
| #107 | Mechanism implemented and measured; page-slack and apparent-byte reductions demonstrated; the frozen ≥1.5 % *allocated*-byte target is missed (0.02 % allocated / 1.13 % apparent) and remains an open quantitative gate with its cause and the failed prediction recorded. |
| #100/#102/#108/#112/#114/#110 | Deduplicated into the single qualification above; no historical matrix rerun. |
| #111 | Stage3 spill and reopened-history work complete; the public spill-scale route is now qualified end to end at 32,000 edits. |
| #117 | Infrastructure simplification complete and reused throughout (plus the recorded cache-integrity repair). |

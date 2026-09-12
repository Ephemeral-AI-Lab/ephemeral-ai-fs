# v0.1.5 benchmark closeout — every family and case

> **Status:** LayerFS 0.1.5 release record. Generated report-only from the #120
> campaign; **no benchmark or proof was run for this release**. The per-case
> values and dispositions are the published ones from
> `docs/roadmap/0.1/0.1.5/issue120/final-report.md` §2, re-verified row by row
> against the immutable receipts and expanded with the receipt facts
> ([benchmark-performance.csv](benchmark-performance.csv),
> [benchmark-verification.csv](benchmark-verification.csv)). Dispositions are
> copied, never recomputed: FAIL stays FAIL, WARN stays WARN, and a reused value
> is never presented as a candidate measurement.

## Protocol

One complete public sample per registered selection (`--perf-fast
--collection-mode`, 300/310/600 s allowances), seed 1, repetition 1, per the
frozen #120 campaign contract; one independent identity-pinned proof per fresh
selection. The runner and verifier serialized on the measurement lock and every
receipt is immutable under `benchmark-results/host-store/issue120/`. Reused
rows carry their source run's contract and name their producing treatment.

**Disposition convention (ordinary comparative cells):** candidate ≤ reference,
or Δ/op under the 3 ms wall floor → PASS (values recorded either way);
beyond-floor non-Tier-2 material (at n=1 no alternating-pair comparator exists,
so Tier-2 condition 2 cannot hold) → WARN with ratio, absolute and per-op delta;
Tier-2 material (all three conditions) → FAIL. Registered harness targets are
Tier-1 and are PASS/FAIL/OWNER-WAIVED directly.

**References:** published v0.1.3 checkpoint single samples with an **undeclared
cache profile**, so every ratio is historical context, never a paired claim.

## Tallies

| Tally | Value |
|---|---|
| Registered performance selections | **198/198 terminal** = 182 fresh + 16 reused |
| Fresh dispositions | **56 PASS / 125 WARN / 1 FAIL** |
| Proof-only selections | **29/29 terminal** = 28 PASS + 1 NOT_RUN_OPTIONAL |
| Proof dispositions (227 registered selections) | 210 PASS + 16 reused + 1 NOT_RUN_OPTIONAL |
| Fresh independent proofs | **182/182 PASS**; every cleanup PASS; no S0/S1, no H-class finding |
| Verified-cold member | `namespace-100000` (0/125,169 resident pages, metadata VERIFIED, acquisition inside the envelope) |
| Native gate on the measured tree | `tools/test-fast.sh` PASS — 530 tests/benchmarks, 118 s, exit 0 |
| Cache contract | warm-prepared-reuse with a clone per sample and the cache identity in the receipt, except the VERIFIED_COLD member; reused rows inherit their source run's contract |

## Family → per-case tables

The tables below list **every** registered selection. `Source` is `fresh` for a
candidate measurement on `c55daf13…` @ `1ff1f2ddd`, or the producing treatment
for a reused value.


### 1. `payload_create_read` — 8 registered selections


All four `payload-create-*` cells **PASS and are faster than v0.1.3** (0.79–1.02×); the three random-read cells WARN (+5.9–13.3 ms) and one is reused from the fsync-qualified treatment.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `payload-create-1m-compact-v2` | performance | pure_call_sum_ns | 29.33 ms | 28.84 ms | 1.02× | +0.50 ms | 1 | PASS | PASS | fresh |
| `payload-create-10m-compact-v2` | performance | pure_call_sum_ns | 82.57 ms | 91.56 ms | 0.90× | -8.99 ms | 1 | PASS | PASS | fresh |
| `payload-create-100m` | performance | pure_call_sum_ns | 575.91 ms | 682.77 ms | 0.84× | -106.86 ms | 1 | PASS | PASS | fresh |
| `payload-create-500m` | performance | pure_call_sum_ns | 2.419 s | 3.068 s | 0.79× | -649.43 ms | 1 | PASS | PASS | fresh |
| `payload-random-read-1-compact-v2` | performance | pure_call_sum_ns | 20.46 ms | 14.58 ms | 1.40× | +5.88 ms | 1 | WARN | PASS | fresh |
| `payload-random-read-10-compact-v2` | performance | pure_call_sum_ns | 25.17 ms | 17.42 ms | 1.45× | +7.75 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `payload-random-read-100` | performance | pure_call_sum_ns | 69.68 ms | 56.40 ms | 1.24× | +13.29 ms | 1 | WARN | PASS | fresh |
| `payload-random-read-500` | performance | pure_call_sum_ns | 232.30 ms | 219.69 ms | 1.06× | +12.61 ms | 1 | WARN | PASS | fresh |


### 2. `dedup_workspace_reuse` — 14 registered selections


The exact/local reuse tiers are the strongest v0.1.3 improvements in the campaign: `exact-100` 0.62× and `local-100` 0.66×. Eight PASS, five WARN, one reused.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `dedup-workspace-exact-1-compact-v2` | performance | pure_call_sum_ns | 34.79 ms | 31.56 ms | 1.10× | +3.22 ms | 1 | WARN | PASS | fresh |
| `dedup-workspace-exact-10-compact-v2` | performance | pure_call_sum_ns | 81.92 ms | 109.32 ms | 0.75× | -27.40 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `dedup-workspace-exact-100` | performance | pure_call_sum_ns | 524.62 ms | 840.53 ms | 0.62× | -315.91 ms | 1 | PASS | PASS | fresh |
| `dedup-workspace-exact-500` | performance | pure_call_sum_ns | 3.900 s | 4.297 s | 0.91× | -396.20 ms | 1 | PASS | PASS | fresh |
| `dedup-workspace-local-1-compact-v2` | performance | pure_call_sum_ns | 30.20 ms | 30.61 ms | 0.99× | -0.41 ms | 1 | PASS | PASS | fresh |
| `dedup-workspace-local-10-compact-v2` | performance | pure_call_sum_ns | 78.99 ms | 103.15 ms | 0.77× | -24.16 ms | 1 | PASS | PASS | fresh |
| `dedup-workspace-local-100` | performance | pure_call_sum_ns | 532.42 ms | 808.39 ms | 0.66× | -275.97 ms | 1 | PASS | PASS | fresh |
| `dedup-workspace-local-500` | performance | pure_call_sum_ns | 4.509 s | 4.307 s | 1.05× | +201.48 ms | 1 | WARN | PASS | fresh |
| `dedup-workspace-unique-1-compact-v2` | performance | pure_call_sum_ns | 37.94 ms | 27.48 ms | 1.38× | +10.47 ms | 1 | WARN | PASS | fresh |
| `dedup-workspace-unique-10-compact-v2` | performance | pure_call_sum_ns | 96.02 ms | 95.67 ms | 1.00× | +0.35 ms | 1 | PASS | PASS | fresh |
| `dedup-workspace-unique-100` | performance | pure_call_sum_ns | 634.99 ms | 768.00 ms | 0.83× | -133.01 ms | 1 | PASS | PASS | fresh |
| `dedup-workspace-unique-500` | performance | pure_call_sum_ns | 4.708 s | 4.320 s | 1.09× | +388.43 ms | 1 | WARN | PASS | fresh |
| `dedup-workspace-unique-1-base128-v3` | performance | pure_call_sum_ns | 39.06 ms | 33.38 ms | 1.17× | +5.68 ms | 1 | WARN | PASS | fresh |
| `dedup-workspace-unique-10-base128-v3` | performance | pure_call_sum_ns | 93.55 ms | 103.01 ms | 0.91× | -9.46 ms | 1 | PASS | PASS | fresh |


### 3. `dedup_cross_file` — 10 registered selections


All eight fresh cells WARN against v0.1.3 (ratios 1.46–3.35×, +11 ms to +691 ms); two reused. These ratios compare the v0.1.5 ordinary path (authenticated CAS + CDC + pack assembly) against pre-authentication v0.1.3 references — #112 context, not a gate.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `dedup-cross-file-anchor-1` | performance | pure_call_sum_ns | 7.61 ms | 3.81 ms | 2.00× | +3.80 ms | 1 | WARN | PASS | fresh |
| `dedup-cross-file-unique-10` | performance | pure_call_sum_ns | 34.39 ms | 22.39 ms | 1.54× | +12.00 ms | 1 | WARN | PASS | fresh |
| `dedup-cross-file-unique-100` | performance | pure_call_sum_ns | 312.51 ms | 93.27 ms | 3.35× | +219.25 ms | 1 | WARN | PASS | fresh |
| `dedup-cross-file-unique-500` | performance | pure_call_sum_ns | 1.291 s | 444.95 ms | 2.90× | +845.98 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `dedup-cross-file-identical-10` | performance | pure_call_sum_ns | 19.17 ms | 18.40 ms | 1.04× | +0.77 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `dedup-cross-file-identical-100` | performance | pure_call_sum_ns | 54.37 ms | 28.96 ms | 1.88× | +25.42 ms | 1 | WARN | PASS | fresh |
| `dedup-cross-file-identical-500` | performance | pure_call_sum_ns | 209.91 ms | 110.63 ms | 1.90× | +99.28 ms | 1 | WARN | PASS | fresh |
| `dedup-cross-file-mixed-10` | performance | pure_call_sum_ns | 35.11 ms | 24.05 ms | 1.46× | +11.06 ms | 1 | WARN | PASS | fresh |
| `dedup-cross-file-mixed-100` | performance | pure_call_sum_ns | 276.18 ms | 113.94 ms | 2.42× | +162.23 ms | 1 | WARN | PASS | fresh |
| `dedup-cross-file-mixed-500` | performance | pure_call_sum_ns | 1.233 s | 541.99 ms | 2.27× | +690.85 ms | 1 | WARN | PASS | fresh |


### 4. `dedup_cdc_locality` — 21 registered selections


Two PASS (`insert-10`, `delete-10`), seventeen WARN with ratios 1.04–3.45×; `scattered-500` carries the largest fresh aggregate (+943.03 ms, still under the mandatory >1 s diagnosis threshold) and `scattered-100` is the campaign's worst ratio (3.45×) with an **≈452 ms unattributed gap** between its timer and command window (#114). One reused, plus the proof-only `dedup-cdc-boundaries-proof` PASS (1.74 s).


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `dedup-cdc-overwrite-1` | performance | pure_call_sum_ns | 8.15 ms | 4.00 ms | 2.04× | +4.15 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-overwrite-10` | performance | pure_call_sum_ns | 18.28 ms | 20.45 ms | 0.89× | -2.17 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `dedup-cdc-overwrite-100` | performance | pure_call_sum_ns | 61.05 ms | 30.95 ms | 1.97× | +30.10 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-overwrite-500` | performance | pure_call_sum_ns | 243.22 ms | 122.04 ms | 1.99× | +121.18 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-insert-1` | performance | pure_call_sum_ns | 7.59 ms | 3.90 ms | 1.95× | +3.69 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-insert-10` | performance | pure_call_sum_ns | 18.76 ms | 18.06 ms | 1.04× | +0.70 ms | 1 | PASS | PASS | fresh |
| `dedup-cdc-insert-100` | performance | pure_call_sum_ns | 64.35 ms | 34.18 ms | 1.88× | +30.17 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-insert-500` | performance | pure_call_sum_ns | 257.57 ms | 136.78 ms | 1.88× | +120.79 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-delete-1` | performance | pure_call_sum_ns | 8.25 ms | 4.41 ms | 1.87× | +3.84 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-delete-10` | performance | pure_call_sum_ns | 18.97 ms | 18.22 ms | 1.04× | +0.74 ms | 1 | PASS | PASS | fresh |
| `dedup-cdc-delete-100` | performance | pure_call_sum_ns | 57.37 ms | 33.17 ms | 1.73× | +24.20 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-delete-500` | performance | pure_call_sum_ns | 243.00 ms | 130.86 ms | 1.86× | +112.14 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-common-body-1` | performance | pure_call_sum_ns | 8.86 ms | 4.13 ms | 2.15× | +4.73 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-common-body-10` | performance | pure_call_sum_ns | 24.89 ms | 20.40 ms | 1.22× | +4.50 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-common-body-100` | performance | pure_call_sum_ns | 131.23 ms | 58.21 ms | 2.25× | +73.02 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-common-body-500` | performance | pure_call_sum_ns | 552.10 ms | 244.35 ms | 2.26× | +307.75 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-scattered-1` | performance | pure_call_sum_ns | 11.48 ms | 4.65 ms | 2.47× | +6.83 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-scattered-10` | performance | pure_call_sum_ns | 40.93 ms | 23.47 ms | 1.74× | +17.46 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-scattered-100` | performance | pure_call_sum_ns | 308.33 ms | 89.37 ms | 3.45× | +218.96 ms | 1 | WARN | PASS | fresh |
| `dedup-cdc-scattered-500` | performance | pure_call_sum_ns | 1.426 s | 483.01 ms | 2.95× | +943.03 ms | 1 | WARN | PASS | fresh |


| Selection | Kind | Proof status | Proof wall (s) | Omissions | Evidence |
|---|---|---|---:|---|---|
| `dedup-cdc-boundaries-proof` | proof-only | PASS | 1.7386965829937253 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/dedup_cdc_locality/dedup-cdc-boundaries-proof/verification.json` |


### 5. `edit_length_preserving` — 12 registered selections


One public SDK range-edit plus its Commit per cell (`edit_commit_ns`, ops=1). Eleven fresh, one reused from the `6693224e…` affected-rerun; all 24 registered targets PASS. Nine WARN cells sit at +3.3–7.3 ms per-op, every one under the 10 ms/op hard cap with no n3 comparator at n=1.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `overwrite-head-4k-on-1mib-ops-1` | performance | edit_commit_ns | 10.95 ms | 5.78 ms | 1.90× | +5.17 ms | 1 | WARN | PASS | fresh |
| `overwrite-head-4k-on-10mib-ops-1` | performance | edit_commit_ns | 11.79 ms | 6.15 ms | 1.92× | +5.64 ms | 1 | WARN | PASS | fresh |
| `overwrite-head-4k-on-100mib-ops-1` | performance | edit_commit_ns | 7.88 ms | 6.65 ms | 1.19× | +1.23 ms | 1 | PASS | PASS | fresh |
| `overwrite-head-4k-on-500mib-ops-1` | performance | edit_commit_ns | 8.78 ms | 8.41 ms | 1.04× | +0.37 ms | 1 | PASS | PASS | fresh |
| `overwrite-middle-4k-on-1mib-ops-1` | performance | edit_commit_ns | 8.54 ms | 6.14 ms | 1.39× | +2.40 ms | 1 | REUSED-FROM 6693224e affected-rerun | reused | reused: 6693224e… |
| `overwrite-middle-4k-on-10mib-ops-1` | performance | edit_commit_ns | 8.17 ms | 5.92 ms | 1.38× | +2.25 ms | 1 | PASS | PASS | fresh |
| `overwrite-middle-4k-on-100mib-ops-1` | performance | edit_commit_ns | 8.83 ms | 6.19 ms | 1.43× | +2.64 ms | 1 | PASS | PASS | fresh |
| `overwrite-middle-4k-on-500mib-ops-1` | performance | edit_commit_ns | 11.18 ms | 7.17 ms | 1.56× | +4.01 ms | 1 | WARN | PASS | fresh |
| `overwrite-tail-4k-on-1mib-ops-1` | performance | edit_commit_ns | 8.39 ms | 6.11 ms | 1.37× | +2.27 ms | 1 | PASS | PASS | fresh |
| `overwrite-tail-4k-on-10mib-ops-1` | performance | edit_commit_ns | 8.57 ms | 7.27 ms | 1.18× | +1.30 ms | 1 | PASS | PASS | fresh |
| `overwrite-tail-4k-on-100mib-ops-1` | performance | edit_commit_ns | 9.80 ms | 7.63 ms | 1.28× | +2.17 ms | 1 | PASS | PASS | fresh |
| `overwrite-tail-4k-on-500mib-ops-1` | performance | edit_commit_ns | 11.80 ms | 9.38 ms | 1.26× | +2.42 ms | 1 | PASS | PASS | fresh |


### 6. `edit_length_changing` — 32 registered selections


Thirty fresh (12 PASS / 18 WARN, +3.0–6.9 ms) plus two reused from the `6693224e…` affected-rerun. All registered targets PASS.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `insert-middle-4k-on-1mib-ops-1` | performance | edit_commit_ns | 8.31 ms | 5.59 ms | 1.49× | +2.72 ms | 1 | REUSED-FROM 6693224e affected-rerun | reused | reused: 6693224e… |
| `insert-middle-4k-on-10mib-ops-1` | performance | edit_commit_ns | 7.93 ms | 7.40 ms | 1.07× | +0.54 ms | 1 | PASS | PASS | fresh |
| `insert-middle-4k-on-100mib-ops-1` | performance | edit_commit_ns | 8.65 ms | 12.44 ms | 0.70× | -3.79 ms | 1 | PASS | PASS | fresh |
| `insert-middle-4k-on-500mib-result-capped-v2-ops-1` | performance | edit_commit_ns | 9.24 ms | 18.01 ms | 0.51× | -8.77 ms | 1 | PASS | PASS | fresh |
| `delete-middle-4k-on-1mib-ops-1` | performance | edit_commit_ns | 8.11 ms | 5.29 ms | 1.53× | +2.82 ms | 1 | REUSED-FROM 6693224e affected-rerun | reused | reused: 6693224e… |
| `delete-middle-4k-on-10mib-ops-1` | performance | edit_commit_ns | 7.89 ms | 6.57 ms | 1.20× | +1.32 ms | 1 | PASS | PASS | fresh |
| `delete-middle-4k-on-100mib-ops-1` | performance | edit_commit_ns | 10.02 ms | 6.76 ms | 1.48× | +3.27 ms | 1 | WARN | PASS | fresh |
| `delete-middle-4k-on-500mib-ops-1` | performance | edit_commit_ns | 12.78 ms | 7.49 ms | 1.71× | +5.29 ms | 1 | WARN | PASS | fresh |
| `append-tail-4k-on-1mib-ops-1` | performance | edit_commit_ns | 9.70 ms | 6.26 ms | 1.55× | +3.44 ms | 1 | WARN | PASS | fresh |
| `append-tail-4k-on-10mib-ops-1` | performance | edit_commit_ns | 9.63 ms | 5.91 ms | 1.63× | +3.72 ms | 1 | WARN | PASS | fresh |
| `append-tail-4k-on-100mib-ops-1` | performance | edit_commit_ns | 9.34 ms | 6.25 ms | 1.49× | +3.09 ms | 1 | WARN | PASS | fresh |
| `append-tail-4k-on-500mib-result-capped-v2-ops-1` | performance | edit_commit_ns | 10.98 ms | 6.69 ms | 1.64× | +4.30 ms | 1 | WARN | PASS | fresh |
| `prepend-head-4k-on-1mib-ops-1` | performance | edit_commit_ns | 8.98 ms | 5.91 ms | 1.52× | +3.07 ms | 1 | WARN | PASS | fresh |
| `prepend-head-4k-on-10mib-ops-1` | performance | edit_commit_ns | 10.04 ms | 5.67 ms | 1.77× | +4.37 ms | 1 | WARN | PASS | fresh |
| `prepend-head-4k-on-100mib-ops-1` | performance | edit_commit_ns | 10.45 ms | 6.27 ms | 1.67× | +4.18 ms | 1 | WARN | PASS | fresh |
| `prepend-head-4k-on-500mib-result-capped-v2-ops-1` | performance | edit_commit_ns | 9.50 ms | 7.10 ms | 1.34× | +2.41 ms | 1 | PASS | PASS | fresh |
| `replace-grow-middle-2k-to-4k-on-1mib-ops-1` | performance | edit_commit_ns | 8.49 ms | 5.74 ms | 1.48× | +2.75 ms | 1 | PASS | PASS | fresh |
| `replace-grow-middle-2k-to-4k-on-10mib-ops-1` | performance | edit_commit_ns | 10.04 ms | 6.46 ms | 1.55× | +3.57 ms | 1 | WARN | PASS | fresh |
| `replace-grow-middle-2k-to-4k-on-100mib-ops-1` | performance | edit_commit_ns | 8.43 ms | 6.38 ms | 1.32× | +2.05 ms | 1 | PASS | PASS | fresh |
| `replace-grow-middle-2k-to-4k-on-500mib-result-capped-v2-ops-1` | performance | edit_commit_ns | 9.33 ms | 7.92 ms | 1.18× | +1.41 ms | 1 | PASS | PASS | fresh |
| `replace-shrink-middle-4k-to-2k-on-1mib-ops-1` | performance | edit_commit_ns | 7.82 ms | 5.88 ms | 1.33× | +1.94 ms | 1 | PASS | PASS | fresh |
| `replace-shrink-middle-4k-to-2k-on-10mib-ops-1` | performance | edit_commit_ns | 8.33 ms | 5.77 ms | 1.44× | +2.56 ms | 1 | PASS | PASS | fresh |
| `replace-shrink-middle-4k-to-2k-on-100mib-ops-1` | performance | edit_commit_ns | 8.48 ms | 7.01 ms | 1.21× | +1.47 ms | 1 | PASS | PASS | fresh |
| `replace-shrink-middle-4k-to-2k-on-500mib-ops-1` | performance | edit_commit_ns | 13.87 ms | 7.00 ms | 1.98× | +6.88 ms | 1 | WARN | PASS | fresh |
| `truncate-tail-4k-on-1mib-ops-1` | performance | edit_commit_ns | 10.07 ms | 5.29 ms | 1.90× | +4.77 ms | 1 | WARN | PASS | fresh |
| `truncate-tail-4k-on-10mib-ops-1` | performance | edit_commit_ns | 9.17 ms | 15.64 ms | 0.59× | -6.47 ms | 1 | PASS | PASS | fresh |
| `truncate-tail-4k-on-100mib-ops-1` | performance | edit_commit_ns | 9.39 ms | 6.53 ms | 1.44× | +2.86 ms | 1 | PASS | PASS | fresh |
| `truncate-tail-4k-on-500mib-ops-1` | performance | edit_commit_ns | 10.89 ms | 6.90 ms | 1.58× | +4.00 ms | 1 | WARN | PASS | fresh |
| `zero-extend-tail-4k-on-1mib-ops-1` | performance | edit_commit_ns | 7.73 ms | 5.63 ms | 1.37× | +2.10 ms | 1 | PASS | PASS | fresh |
| `zero-extend-tail-4k-on-10mib-ops-1` | performance | edit_commit_ns | 9.80 ms | 5.64 ms | 1.74× | +4.17 ms | 1 | WARN | PASS | fresh |
| `zero-extend-tail-4k-on-100mib-ops-1` | performance | edit_commit_ns | 9.45 ms | 6.40 ms | 1.48× | +3.04 ms | 1 | WARN | PASS | fresh |
| `zero-extend-tail-4k-on-500mib-result-capped-v2-ops-1` | performance | edit_commit_ns | 10.33 ms | 6.66 ms | 1.55× | +3.67 ms | 1 | WARN | PASS | fresh |


### 7. `edit_canonical_chunk_count` — 12 registered selections


Twelve fresh; five PASS, seven WARN (+3.3–7.3 ms), all under the hard cap. The candidate is within or below the #104 uncompacted band on 22 of these 24 edit cells (diagnostic context).


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `overwrite-fixed-64k-chunk-count-preserve-on-1mib-ops-1` | performance | edit_commit_ns | 11.02 ms | 6.85 ms | 1.61× | +4.17 ms | 1 | WARN | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-preserve-on-10mib-ops-1` | performance | edit_commit_ns | 13.05 ms | 8.51 ms | 1.53× | +4.54 ms | 1 | WARN | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-preserve-on-100mib-ops-1` | performance | edit_commit_ns | 9.27 ms | 7.34 ms | 1.26× | +1.94 ms | 1 | PASS | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-preserve-on-500mib-ops-1` | performance | edit_commit_ns | 10.04 ms | 8.27 ms | 1.21× | +1.77 ms | 1 | PASS | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-increase-on-1mib-ops-1` | performance | edit_commit_ns | 8.43 ms | 7.85 ms | 1.07× | +0.57 ms | 1 | PASS | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-increase-on-10mib-ops-1` | performance | edit_commit_ns | 10.31 ms | 7.52 ms | 1.37× | +2.80 ms | 1 | PASS | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-increase-on-100mib-ops-1` | performance | edit_commit_ns | 9.54 ms | 7.87 ms | 1.21× | +1.67 ms | 1 | PASS | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-increase-on-500mib-ops-1` | performance | edit_commit_ns | 12.26 ms | 8.47 ms | 1.45× | +3.79 ms | 1 | WARN | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-decrease-on-1mib-ops-1` | performance | edit_commit_ns | 7.59 ms | 7.18 ms | 1.06× | +0.42 ms | 1 | PASS | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-decrease-on-10mib-ops-1` | performance | edit_commit_ns | 11.77 ms | 7.01 ms | 1.68× | +4.75 ms | 1 | WARN | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-decrease-on-100mib-ops-1` | performance | edit_commit_ns | 10.91 ms | 7.58 ms | 1.44× | +3.33 ms | 1 | WARN | PASS | fresh |
| `overwrite-fixed-64k-chunk-count-decrease-on-500mib-ops-1` | performance | edit_commit_ns | 15.38 ms | 8.10 ms | 1.90× | +7.28 ms | 1 | WARN | PASS | fresh |


### 8. `init_namespace` — 4 registered selections


All four tiers WARN against v0.1.3 (2.53–3.22×). `namespace-100000` is the campaign's VERIFIED_COLD member: 100,000 files, 125,169 expected pages, **0 resident** after eviction, metadata VERIFIED, acquisition 25.942 s reported separately and inside the 31.615 s complete envelope. Its 2.7 s absolute target is **owner-waived** ([waivers](../0.1.5/waivers.md) §2) and its 4.3975 s measurement stands as a TARGET_MISS. The catalogued >1 s diagnosis: +1.794 s is one Init of 100,000 files on the ordinary path (authenticated CAS + pack assembly + delta encoding); the same cell measured 4.841 s on #104 (candidate −9.1 % versus it).


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `namespace-100-compact-v3` | performance | layerstack_init_ns | 28.70 ms | 8.90 ms | 3.22× | +19.80 ms | 1 | WARN | PASS | fresh |
| `namespace-1000-compact-v3` | performance | layerstack_init_ns | 112.73 ms | 38.08 ms | 2.96× | +74.66 ms | 1 | WARN | PASS | fresh |
| `namespace-10000` | performance | layerstack_init_ns | 1.020 s | 403.47 ms | 2.53× | +616.95 ms | 1 | WARN | PASS | fresh |
| `namespace-100000` | performance | layerstack_init_ns | 4.398 s | 2.603 s | 1.69× | +1,794.38 ms | 1 | WARN | PASS | fresh |


### 9. `store_footprint` — 6 registered selections


Six WARN rows (timers 1.44–2.55× v0.1.3) with footprints matching #104 within ±0.04 %. **Recorded reuse deviation:** the two cells that appeared in the #120 reuse list were re-collected fresh because #107 changes exactly their measured quantity; `unique-100000` allocated 530,358,272 → **520,142,848 B (−1.9 %)** — reusing the old number would have misstated the candidate by 10 MB. Its construction timer 5.397 s is +36 % versus fsync-qualified: the #107 pack-row UPDATE cost on one giant commit.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `store-footprint-unique-100000` | performance | product_call_sum_ns | 5.397 s | 2.982 s | 1.81× | +2,414.67 ms | 1 | WARN | PASS | fresh |
| `store-footprint-metadata-cardinality-100000` | performance | product_call_sum_ns | 6.640 s | 4.570 s | 1.45× | +2,069.08 ms | 1 | WARN | PASS | fresh |
| `store-footprint-large-object-500m` | performance | product_call_sum_ns | 1.254 s | 491.91 ms | 2.55× | +761.89 ms | 1 | WARN | PASS | fresh |
| `store-footprint-unique-100-low-v1` | performance | product_call_sum_ns | 49.65 ms | 31.95 ms | 1.55× | +17.70 ms | 1 | WARN | PASS | fresh |
| `store-footprint-metadata-cardinality-100-low-v1` | performance | product_call_sum_ns | 60.36 ms | 35.87 ms | 1.68× | +24.49 ms | 1 | WARN | PASS | fresh |
| `store-footprint-large-object-10m-low-v1` | performance | product_call_sum_ns | 63.20 ms | 43.89 ms | 1.44× | +19.30 ms | 1 | WARN | PASS | fresh |


### 10. `tiny_file_churn` — 20 registered selections


Nineteen fresh (4 PASS / 15 WARN, +3.2–679 ms) plus one reused. The registered Tier-1 gate `tiny-create100 < 1 s` **PASSES at 79 ms**.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `tiny-create-1-compact-v2` | performance | pure_call_sum_ns | 23.26 ms | 19.05 ms | 1.22× | +4.21 ms | 1 | WARN | PASS | fresh |
| `tiny-create-10-compact-v2` | performance | pure_call_sum_ns | 30.16 ms | 26.71 ms | 1.13× | +3.45 ms | 1 | WARN | PASS | fresh |
| `tiny-create-100-mixed-v4` | performance | pure_call_sum_ns | 78.98 ms | 53.62 ms | 1.47× | +25.36 ms | 1 | WARN | PASS | fresh |
| `tiny-create-500-mixed-v4` | performance | pure_call_sum_ns | 242.66 ms | 201.74 ms | 1.20× | +40.92 ms | 1 | WARN | PASS | fresh |
| `tiny-stat-1-compact-v2` | performance | pure_call_sum_ns | 21.81 ms | 15.83 ms | 1.38× | +5.97 ms | 1 | WARN | PASS | fresh |
| `tiny-stat-10-compact-v2` | performance | pure_call_sum_ns | 28.63 ms | 21.19 ms | 1.35× | +7.43 ms | 1 | WARN | PASS | fresh |
| `tiny-stat-100-mixed-v4` | performance | pure_call_sum_ns | 45.70 ms | 37.81 ms | 1.21× | +7.89 ms | 1 | WARN | PASS | fresh |
| `tiny-stat-500-mixed-v4` | performance | pure_call_sum_ns | 69.84 ms | 55.35 ms | 1.26× | +14.49 ms | 1 | WARN | PASS | fresh |
| `tiny-unlink-1-compact-v2` | performance | pure_call_sum_ns | 19.56 ms | 17.63 ms | 1.11× | +1.93 ms | 1 | PASS | PASS | fresh |
| `tiny-unlink-10-compact-v2` | performance | pure_call_sum_ns | 28.36 ms | 25.21 ms | 1.12× | +3.15 ms | 1 | WARN | PASS | fresh |
| `tiny-unlink-100-mixed-v4` | performance | pure_call_sum_ns | 63.78 ms | 50.92 ms | 1.25× | +12.87 ms | 1 | WARN | PASS | fresh |
| `tiny-unlink-500-mixed-v4` | performance | pure_call_sum_ns | 119.58 ms | 114.28 ms | 1.05× | +5.30 ms | 1 | WARN | PASS | fresh |
| `tiny-bulk-create-1-compact-v2` | performance | pure_call_sum_ns | 97.12 ms | 96.73 ms | 1.00× | +0.39 ms | 1 | PASS | PASS | fresh |
| `tiny-bulk-create-10-compact-v2` | performance | pure_call_sum_ns | 316.83 ms | 295.90 ms | 1.07× | +20.94 ms | 1 | WARN | PASS | fresh |
| `tiny-bulk-create-100-mixed-v3` | performance | pure_call_sum_ns | 983.33 ms | 989.89 ms | 0.99× | -6.56 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `tiny-bulk-create-500-mixed-v3` | performance | pure_call_sum_ns | 5.733 s | 5.054 s | 1.13× | +678.94 ms | 1 | WARN | PASS | fresh |
| `tiny-bulk-delete-1-compact-v2` | performance | pure_call_sum_ns | 110.07 ms | 93.35 ms | 1.18× | +16.72 ms | 1 | WARN | PASS | fresh |
| `tiny-bulk-delete-10-compact-v2` | performance | pure_call_sum_ns | 209.75 ms | 168.25 ms | 1.25× | +41.49 ms | 1 | WARN | PASS | fresh |
| `tiny-bulk-delete-100-mixed-v3` | performance | pure_call_sum_ns | 306.85 ms | 259.28 ms | 1.18× | +47.57 ms | 1 | WARN | PASS | fresh |
| `tiny-bulk-delete-500-mixed-v3` | performance | pure_call_sum_ns | 1.185 s | 933.00 ms | 1.27× | +252.34 ms | 1 | WARN | PASS | fresh |


### 11. `namespace_mutation` — 4 registered selections


Four fresh WARN rows (+8.2–92.1 ms, ratios 1.24–1.50×); no aggregate exceeds 1 s.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `namespace-subtree-relocate-delete-1-compact-v2` | performance | pure_call_sum_ns | 29.55 ms | 21.31 ms | 1.39× | +8.23 ms | 1 | WARN | PASS | fresh |
| `namespace-subtree-relocate-delete-10-compact-v2` | performance | pure_call_sum_ns | 73.48 ms | 59.14 ms | 1.24× | +14.33 ms | 1 | WARN | PASS | fresh |
| `namespace-subtree-relocate-delete-100-mixed-v4` | performance | pure_call_sum_ns | 74.19 ms | 51.02 ms | 1.45× | +23.17 ms | 1 | WARN | PASS | fresh |
| `namespace-subtree-relocate-delete-500-mixed-v4` | performance | pure_call_sum_ns | 274.96 ms | 182.89 ms | 1.50× | +92.07 ms | 1 | WARN | PASS | fresh |


### 12. `directory_construction_traversal` — 12 registered selections


Eleven fresh (2 PASS / 9 WARN, +5.4–189.7 ms) plus one reused; the largest fresh delta is +189.73 ms, below the >1 s diagnosis threshold.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `directory-construct-1-compact-v2` | performance | pure_call_sum_ns | 22.38 ms | 17.02 ms | 1.31× | +5.36 ms | 1 | WARN | PASS | fresh |
| `directory-construct-10-compact-v2` | performance | pure_call_sum_ns | 38.60 ms | 39.35 ms | 0.98× | -0.75 ms | 1 | PASS | PASS | fresh |
| `directory-construct-100-mixed-v4` | performance | pure_call_sum_ns | 253.42 ms | 216.06 ms | 1.17× | +37.36 ms | 1 | WARN | PASS | fresh |
| `directory-construct-500-mixed-v4` | performance | pure_call_sum_ns | 1.140 s | 1.031 s | 1.11× | +109.75 ms | 1 | WARN | PASS | fresh |
| `directory-metadata-scan-1-compact-v2` | performance | pure_call_sum_ns | 87.97 ms | 70.53 ms | 1.25× | +17.44 ms | 1 | WARN | PASS | fresh |
| `directory-metadata-scan-10-compact-v2` | performance | pure_call_sum_ns | 137.61 ms | 100.74 ms | 1.37× | +36.87 ms | 1 | WARN | PASS | fresh |
| `directory-metadata-scan-100-mixed-v4` | performance | pure_call_sum_ns | 333.03 ms | 244.24 ms | 1.36× | +88.79 ms | 1 | WARN | PASS | fresh |
| `directory-metadata-scan-500-mixed-v4` | performance | pure_call_sum_ns | 694.66 ms | 504.93 ms | 1.38× | +189.73 ms | 1 | WARN | PASS | fresh |
| `directory-content-scan-1-compact-v2` | performance | pure_call_sum_ns | 105.53 ms | 84.76 ms | 1.25× | +20.77 ms | 1 | WARN | PASS | fresh |
| `directory-content-scan-10-compact-v2` | performance | pure_call_sum_ns | 348.62 ms | 308.87 ms | 1.13× | +39.74 ms | 1 | WARN | PASS | fresh |
| `directory-content-scan-100-mixed-v4` | performance | pure_call_sum_ns | 1.231 s | 1.105 s | 1.11× | +126.19 ms | 1 | WARN | PASS | fresh |
| `directory-content-scan-500-mixed-v4` | performance | pure_call_sum_ns | 4.434 s | 3.907 s | 1.14× | +527.55 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |


### 13. `workspace_change_locality` — 16 registered selections


Twelve fresh (4 PASS / 8 WARN) plus four reused from the fsync-qualified treatment. Highlight: `distributed-sdk-edit-100` **0.63×** and `-500` **0.23×** — the #116 bounded-pending win. The two reused >1 s diagnoses (`dense-rewrite-100` +1.40 s, `-500` +4.81 s) are the batched-fsync write path plus authenticated pack admission, measured on the reused treatment.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `workspace-clean-commit-1-compact-v2` | performance | pure_call_sum_ns | 10.50 ms | 11.32 ms | 0.93× | -0.82 ms | 1 | PASS | PASS | fresh |
| `workspace-clean-commit-10-compact-v2` | performance | pure_call_sum_ns | 11.87 ms | 10.87 ms | 1.09× | +1.00 ms | 1 | PASS | PASS | fresh |
| `workspace-clean-commit-100-mixed-v4` | performance | pure_call_sum_ns | 15.47 ms | 11.56 ms | 1.34× | +3.91 ms | 1 | WARN | PASS | fresh |
| `workspace-clean-commit-500-mixed-v4` | performance | pure_call_sum_ns | 17.80 ms | 11.87 ms | 1.50× | +5.93 ms | 1 | WARN | PASS | fresh |
| `workspace-fixed-move-1-compact-v2` | performance | pure_call_sum_ns | 26.56 ms | 19.71 ms | 1.35× | +6.85 ms | 1 | WARN | PASS | fresh |
| `workspace-fixed-move-10-compact-v2` | performance | pure_call_sum_ns | 28.83 ms | 21.32 ms | 1.35× | +7.51 ms | 1 | WARN | PASS | fresh |
| `workspace-fixed-move-100-mixed-v4` | performance | pure_call_sum_ns | 35.95 ms | 28.35 ms | 1.27× | +7.60 ms | 1 | WARN | PASS | fresh |
| `workspace-fixed-move-500-mixed-v4` | performance | pure_call_sum_ns | 47.75 ms | 27.59 ms | 1.73× | +20.16 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `workspace-distributed-sdk-edit-1-compact-v2` | performance | pure_call_sum_ns | 25.27 ms | 16.94 ms | 1.49× | +8.34 ms | 1 | WARN | PASS | fresh |
| `workspace-distributed-sdk-edit-10-compact-v2` | performance | pure_call_sum_ns | 42.31 ms | 39.18 ms | 1.08× | +3.13 ms | 1 | WARN | PASS | fresh |
| `workspace-distributed-sdk-edit-100-mixed-v4` | performance | pure_call_sum_ns | 213.09 ms | 336.40 ms | 0.63× | -123.31 ms | 1 | PASS | PASS | fresh |
| `workspace-distributed-sdk-edit-500-mixed-v4` | performance | pure_call_sum_ns | 666.10 ms | 2.849 s | 0.23× | -2,183.08 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `workspace-dense-rewrite-1-compact-v2` | performance | pure_call_sum_ns | 109.38 ms | 84.12 ms | 1.30× | +25.26 ms | 1 | WARN | PASS | fresh |
| `workspace-dense-rewrite-10-compact-v2` | performance | pure_call_sum_ns | 583.17 ms | 316.62 ms | 1.84× | +266.55 ms | 1 | WARN | PASS | fresh |
| `workspace-dense-rewrite-100-mixed-v4` | performance | pure_call_sum_ns | 2.894 s | 1.498 s | 1.93× | +1,396.44 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |
| `workspace-dense-rewrite-500-mixed-v4` | performance | pure_call_sum_ns | 10.499 s | 5.689 s | 1.85× | +4,810.45 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |


### 14. `dedup_branch_history` — 20 registered selections


Seven PASS, twelve WARN, and the campaign's single FAIL. The 500-tier history cells (+0.64–1.38 s) match 500 × the per-iteration #116/#107 mechanism costs measured in the unrelated-500 RCA. `unrelated-1/10/100` are 0.52–0.90× v0.1.3 (faster) while `unrelated-500` is **0.89×** yet still misses its own absolute gate — see the bug ledger below.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `dedup-history-distributed-1` | performance | pure_call_sum_ns | 17.14 ms | 22.16 ms | 0.77× | -5.03 ms | 1 | PASS | PASS | fresh |
| `dedup-history-distributed-10` | performance | pure_call_sum_ns | 69.57 ms | 82.27 ms | 0.85× | -12.69 ms | 1 | PASS | PASS | fresh |
| `dedup-history-distributed-100` | performance | pure_call_sum_ns | 735.40 ms | 586.40 ms | 1.25× | +149.00 ms | 1 | WARN | PASS | fresh |
| `dedup-history-distributed-500` | performance | pure_call_sum_ns | 4.304 s | 2.925 s | 1.47× | +1,378.64 ms | 1 | WARN | PASS | fresh |
| `dedup-history-hotset-1` | performance | pure_call_sum_ns | 23.41 ms | 23.58 ms | 0.99× | -0.17 ms | 1 | PASS | PASS | fresh |
| `dedup-history-hotset-10` | performance | pure_call_sum_ns | 125.41 ms | 118.46 ms | 1.06× | +6.95 ms | 1 | WARN | PASS | fresh |
| `dedup-history-hotset-100` | performance | pure_call_sum_ns | 985.36 ms | 799.50 ms | 1.23× | +185.86 ms | 1 | WARN | PASS | fresh |
| `dedup-history-hotset-500` | performance | pure_call_sum_ns | 4.914 s | 3.684 s | 1.33× | +1,230.72 ms | 1 | WARN | PASS | fresh |
| `dedup-history-recurring-1` | performance | pure_call_sum_ns | 24.06 ms | 21.46 ms | 1.12× | +2.60 ms | 1 | PASS | PASS | fresh |
| `dedup-history-recurring-10` | performance | pure_call_sum_ns | 101.62 ms | 62.92 ms | 1.61× | +38.70 ms | 1 | WARN | PASS | fresh |
| `dedup-history-recurring-100` | performance | pure_call_sum_ns | 618.64 ms | 462.84 ms | 1.34× | +155.80 ms | 1 | WARN | PASS | fresh |
| `dedup-history-recurring-500` | performance | pure_call_sum_ns | 2.787 s | 2.145 s | 1.30× | +641.66 ms | 1 | WARN | PASS | fresh |
| `dedup-history-metadata-1` | performance | pure_call_sum_ns | 29.65 ms | 20.65 ms | 1.44× | +9.00 ms | 1 | WARN | PASS | fresh |
| `dedup-history-metadata-10` | performance | pure_call_sum_ns | 101.61 ms | 76.47 ms | 1.33× | +25.14 ms | 1 | WARN | PASS | fresh |
| `dedup-history-metadata-100` | performance | pure_call_sum_ns | 918.53 ms | 649.61 ms | 1.41× | +268.92 ms | 1 | WARN | PASS | fresh |
| `dedup-history-metadata-500` | performance | pure_call_sum_ns | 4.442 s | 3.232 s | 1.37× | +1,210.06 ms | 1 | WARN | PASS | fresh |
| `dedup-history-unrelated-1` | performance | pure_call_sum_ns | 453.82 ms | 880.16 ms | 0.52× | -426.34 ms | 1 | PASS | PASS | fresh |
| `dedup-history-unrelated-10` | performance | pure_call_sum_ns | 5.666 s | 9.543 s | 0.59× | -3,877.58 ms | 1 | PASS | PASS | fresh |
| `dedup-history-unrelated-100-mixed-v2` | performance | pure_call_sum_ns | 3.190 s | 3.550 s | 0.90× | -359.86 ms | 1 | PASS | PASS | fresh |
| `dedup-history-unrelated-500-mixed-v2` | performance | pure_call_sum_ns | 16.107 s | 18.164 s | 0.89× | -2,057.08 ms | 1 | FAIL | PASS | fresh |


### 15. `git_tool_workflow` — 4 registered selections


Three fresh WARN rows (1.06×/+18 ms, 1.15×/+94 ms, 1.51×/+959 ms) plus `git-tool-500` reused from fsync-qualified (8.803 s, 1.88× v0.1.3; the reused row's >1 s diagnosis is the per-iteration mechanism cost plus authentication/pack work on the object path).


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `git-tool-1-compact-v2` | performance | pure_call_sum_ns | 337.08 ms | 318.66 ms | 1.06× | +18.43 ms | 1 | WARN | PASS | fresh |
| `git-tool-10-compact-v2` | performance | pure_call_sum_ns | 705.41 ms | 611.38 ms | 1.15× | +94.02 ms | 1 | WARN | PASS | fresh |
| `git-tool-100-mixed-v4` | performance | pure_call_sum_ns | 2.838 s | 1.879 s | 1.51× | +959.07 ms | 1 | WARN | PASS | fresh |
| `git-tool-500-mixed-v4` | performance | pure_call_sum_ns | 8.803 s | 4.689 s | 1.88× | +4,113.73 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |


### 16. `mixed_load_bearing` — 4 registered selections


Three fresh (1 PASS / 2 WARN) plus one reused; largest fresh delta +121.67 ms.


| Selection | Kind | Timer | Candidate | v0.1.3 ref (profile undeclared) | Ratio | Δ/op | Ops | Disposition | Proof | Source |
|---|---|---|---:|---:|---:|---:|---:|---|---|---|
| `agent-episodes-1-compact-v2` | performance | pure_call_sum_ns | 34.38 ms | 26.24 ms | 1.31× | +8.14 ms | 1 | WARN | PASS | fresh |
| `agent-episodes-10-compact-v2` | performance | pure_call_sum_ns | 62.13 ms | 89.91 ms | 0.69× | -27.79 ms | 1 | PASS | PASS | fresh |
| `agent-episodes-100` | performance | pure_call_sum_ns | 1.030 s | 908.41 ms | 1.13× | +121.67 ms | 1 | WARN | PASS | fresh |
| `agent-episodes-500` | performance | pure_call_sum_ns | 8.219 s | 7.535 s | 1.09× | +683.31 ms | 1 | REUSED-FROM fsync-qualified remaining-shared | reused | reused: fsync-qualified 440ae2c4… |


### 17. `workspace_reliability` — 28 registered selections


Twenty-seven runnable proofs **PASS** (2.1–9.0 s) and one is `NOT_RUN_OPTIONAL`. These are proof-only selections: they carry no performance timer and none is reported as a speed result.


| Selection | Kind | Proof status | Proof wall (s) | Omissions | Evidence |
|---|---|---|---:|---|---|
| `workspace-invalid-sdk-edit-compact-v2-proof` | proof-only | PASS | 2.4101969999901485 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-invalid-sdk-edit-compact-v2-proof/verification.json` |
| `workspace-invalid-namespace-compact-v2-proof` | proof-only | PASS | 2.4250047920213547 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-invalid-namespace-compact-v2-proof/verification.json` |
| `workspace-lease-lifecycle-compact-v2-proof` | proof-only | PASS | 2.5743714999989606 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-lease-lifecycle-compact-v2-proof/verification.json` |
| `workspace-open-writer-busy-compact-v2-proof` | proof-only | PASS | 2.7236085000040475 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-open-writer-busy-compact-v2-proof/verification.json` |
| `workspace-live-execution-busy-compact-v2-proof` | proof-only | PASS | 2.356204000010621 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-live-execution-busy-compact-v2-proof/verification.json` |
| `workspace-candidate-failure-retry-compact-v2-proof` | proof-only | PASS | 2.376748291979311 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-candidate-failure-retry-compact-v2-proof/verification.json` |
| `workspace-admission-batch-failure-retry-compact-v2-proof` | proof-only | PASS | 3.511054790986236 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-admission-batch-failure-retry-compact-v2-proof/verification.json` |
| `workspace-final-publication-failure-retry-compact-v2-proof` | proof-only | PASS | 3.9517992500041146 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-final-publication-failure-retry-compact-v2-proof/verification.json` |
| `workspace-published-presentation-failure-smoke-v3-proof` | proof-only | PASS | 2.156578291003825 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-published-presentation-failure-smoke-v3-proof/verification.json` |
| `workspace-dirty-end-discard-compact-v2-proof` | proof-only | PASS | 2.3135516670008656 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-dirty-end-discard-compact-v2-proof/verification.json` |
| `workspace-dirty-net-zero-compact-v2-proof` | proof-only | PASS | 2.509194040991133 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-dirty-net-zero-compact-v2-proof/verification.json` |
| `workspace-short-spool-write-compact-v2-proof` | proof-only | PASS | 2.3350951250176877 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-short-spool-write-compact-v2-proof/verification.json` |
| `workspace-deferred-nospace-compact-v2-proof` | proof-only | PASS | 1.9623064159823116 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-deferred-nospace-compact-v2-proof/verification.json` |
| `workspace-workload-cancel-compact-v2-proof` | proof-only | PASS | 2.748615582997445 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-workload-cancel-compact-v2-proof/verification.json` |
| `workspace-dirty-runtime-disconnect-compact-v2-proof` | proof-only | PASS | 2.6058751250093337 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-dirty-runtime-disconnect-compact-v2-proof/verification.json` |
| `workspace-corrupt-descendant-compact-v2-proof` | proof-only | PASS | 2.156250125000952 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-corrupt-descendant-compact-v2-proof/verification.json` |
| `workspace-missing-descendant-compact-v2-proof` | proof-only | PASS | 2.092963624978438 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-missing-descendant-compact-v2-proof/verification.json` |
| `workspace-parallel-read-write-compact-v2-proof` | proof-only | PASS | 2.195508415985387 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-parallel-read-write-compact-v2-proof/verification.json` |
| `workspace-shared-path-contention-compact-v2-proof` | proof-only | PASS | 2.158009000006132 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-shared-path-contention-compact-v2-proof/verification.json` |
| `workspace-hardlink-alias-compact-v2-proof` | proof-only | PASS | 2.4398577910033055 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-hardlink-alias-compact-v2-proof/verification.json` |
| `workspace-symlink-semantics-compact-v2-proof` | proof-only | PASS | 2.3873098749900237 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-symlink-semantics-compact-v2-proof/verification.json` |
| `workspace-open-rename-unlink-compact-v2-proof` | proof-only | PASS | 2.2619809999887366 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-open-rename-unlink-compact-v2-proof/verification.json` |
| `workspace-metadata-chmod-compact-v2-proof` | proof-only | PASS | 2.4940547500154935 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-metadata-chmod-compact-v2-proof/verification.json` |
| `workspace-metadata-mtime-compact-v2-proof` | proof-only | PASS | 2.2410823329992127 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-metadata-mtime-compact-v2-proof/verification.json` |
| `workspace-metadata-xattr-compact-v2-proof` | proof-only | PASS | 2.0055110410030466 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-metadata-xattr-compact-v2-proof/verification.json` |
| `workspace-exec-500-compact-v2-proof` | proof-only | PASS | 8.90395483301836 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-exec-500-compact-v2-proof/verification.json` |
| `workspace-repeat-publication-compact-v2-proof` | proof-only | PASS | 2.4142839169944637 | no exhaustive Phase 1 replay | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-repeat-publication-compact-v2-proof/verification.json` |
| `workspace-sustained-600s-compact-v2-proof` | proof-only | NOT_RUN_OPTIONAL | — | Optional long test under the frozen campaign declaration (long_test_exclusion); unexecuted, no endurance qualification | `benchmark-results/host-store/issue120/verification/workspace_reliability/workspace-sustained-600s-compact-v2-proof/exception.json` |


## Reused evidence (cited, never claimed as new)

| Item | Producing treatment | Value | Disposition |
|---|---|---|---|
| Ordinary full157 stride-1 construction | final treatment `b5f089eb…` | complete command 763.218 s, 157/157 steps PASS | REUSED (mechanism unchanged; `3e308a8f2` behavior-neutral) |
| full157 same-Store verification | final treatment | 735.749 s; 157 states / 904,143 entries / 4,936,693,030 B; all oracles PASS | REUSED |
| Physical census | final treatment | allocated 83,951,616 B; apparent 82,583,552 B; pack rows 1,058; `store_sha256 88b4fe70…` | REUSED |
| Git157 read-only control | final treatment | 157 checkpoints, mapping verified; allocation-layout WARN retained (56,197,120 vs recorded 56,373,248 B) | REUSED with its WARN |
| Historical access 11 performance + 11 proofs | final treatment | outer walls 2.525–3.232 s, verification 2.48–2.87 s, all 22 PASS inside the unwaived 15 s envelope; cleanups PASS | REUSED |
| Default-budget K32000 route | final treatment | COMPLETE in one Commit; 32,000 edits / 92,821 pieces / charge 2,048,000 under the unchanged 2 MiB budget; independent verification PASS (68.576 s) | REUSED; its historical-family `TARGET_MISS` is the recorded [waiver](../0.1.5/waivers.md) §4 WARN |
| K6000 boundary + default-budget frontier proof | final treatment | K6000 PASS (14.61 s); frontier PASS at exactly 2B (batch=15873 count=31746 flushes=2), 4B and 8B | REUSED |
| 16 registered campaign cells | `6693224e…` affected-rerun (3 SDK-edit cells) and `fsync-qualified` `440ae2c4…` (13 cells) | see the per-case tables; each row names its treatment | REUSED-FROM |

Deviation recorded honestly: the two `store_footprint` cells on the #120 reuse
list were **re-collected fresh** because #107 changes exactly their measured
quantity; the old receipts remain cited as history. One archived-binary custody
observation is also retained: `binary-archive/6693224e…/identity.json` records a
product seal that differs from the #118 affected-rerun record although the binary
bytes hash to the same `6693224e…`; the diagnostic attribution run used the
archive copy with its seal-consistent image, and because the bytes are identical
that attribution is byte-determined.

## Bug ledger

| Severity | Evidence | Reproducer | Root cause | Fix | Impact-set re-run | Disposition |
|---|---|---|---|---|---|---|
| **S2** (Tier-1 gate miss) | `performance/dedup_branch_history/dedup-history-unrelated-500-mixed-v2/perf.jsonl` — 16.107 s vs `< 15 s`; complete command 19.057 s; cleanup PASS | the campaign receipt plus the diagnostic run `diagnostics/unrelated-500-on-6693224e/perf.jsonl` (15.772 s on the #116-without-#107 binary) | 500 × (exec 12.42→15.30 ms, commit 15.14→16.88 ms): #116 bounded pending ≈ +1.98 s (dominant), #107 pack coalescing ≈ +0.33 s; introduced when #116 landed and undetected because #118's affected-rerun covered only the 3 SDK-edit cells | none (not a minimal-fix candidate) | none | **FAIL — unrepaired**, dispositioned by the owner [waiver](../0.1.5/waivers.md) §1; the cell is 0.89× v0.1.3 |

No S0, no S1 and no H-class finding occurred: 182/182 fresh samples COMPLETE,
182/182 fresh proofs PASS, 28/28 runnable proof-only proofs PASS, every cleanup
PASS, no custody/identity failure and no resource-bound breach.

## Campaign-wide distribution (context for #112 — never a gate)

Across the 198 comparable registered cells: **139 of 198 cells ≥15 % slower**
than published v0.1.3, median ratio **1.34×**, total added time **+33.115 s**.
Worst ratios: `dedup-cdc-scattered-100` 3.45×, `dedup-cross-file-unique-100`
3.35×, `namespace-100` 3.22×. Largest absolute deltas: reused
`dense-rewrite-500` +4.81 s, fresh 500-tier history cells +0.64–1.38 s,
`git-tool-100` +0.96 s, `scattered-500` +0.94 s. Faster than v0.1.3:
`payload-create-*` (0.79–1.02×), `dedup-workspace exact/local-100` (0.62–0.66×),
`distributed-sdk-edit-100/500` (0.63×/0.23×), `unrelated-1/10/100` (0.52–0.90×).

Against the earlier #104 baseline the candidate's median ratio is **0.993×** with
38 fresh cells ≥15 % faster (best: `workspace-distributed-sdk-edit-500`
4.131 → 0.666 s = 0.16×) and 29 fresh cells ≥15 % slower — consistent with the
#116/#107 fixed per-iteration costs on ms-scale cells.

## Explicit gaps and omissions

1. `workspace-sustained-600s-compact-v2-proof` — **NOT_RUN_OPTIONAL**, excluded
   by the frozen campaign declaration (`long_test_exclusion`): optional long
   test, **no endurance qualification in v0.1.5**. The other five extended
   reliability members and all 27 runnable reliability proofs ran and PASSed.
2. `dedup-history-unrelated-500-mixed-v2` — FAIL, dispositioned by waiver
   ([waivers](../0.1.5/waivers.md) §1).
3. The `store_footprint` reuse deviation recorded above.
4. The archived-binary seal observation recorded above.
5. `namespace-100000`'s 2.7 s absolute cold target — TARGET_MISS under the
   carried-forward owner waiver ([waivers](../0.1.5/waivers.md) §2).
6. The K32000 historical-family `TARGET_MISS` — reporting-only target recorded
   as WARN ([waivers](../0.1.5/waivers.md) §4).

## What this closeout does not say

It does not claim an all-gates pass, a speedup, a storage-target pass or
endurance. It does not present a reused value as a measurement of the candidate's
bytes, and it does not convert the FAIL, the 125 WARNs, the waived targets or the
unmet #108/#112/#114 gates into passes. Owner acceptance retires optimization
scope; the classifications above are unchanged.

# Issue 62 — mixed-v4 tiny-op background results

Status: **background-only tiny ops collected**. This finishes the last #62 membership slice. Operations stay N tiny 0–8 KiB files; the mixed tree is untouched background. Compact 1/10 and `tiny-bulk-*-mixed-v3` were not changed or rerun.

Machine-readable table: [issue62-mixed-v4-tiny-background-results.json](issue62-mixed-v4-tiny-background-results.json).
Raw receipts: `benchmark-results/host-store/campaigns/issue62-tiny/`.

## Campaign identities

| Item | Value |
|---|---|
| Product seal | `0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695` |
| Source seal | `8c43327fcc9beb5418d62a00cffe8b19145c99b91194b2f7ca4a6d14a5c6a499` |
| Image | `layerfs-bench-infra:8c43327fcc9beb54` (`sha256:fd2d4e0024c4cd61480a22d49c5cb3a83c1240375a4fa1bf2f77a43e24acb3cd`) |
| Topology | host-store: macOS SDK/SQLite/spool; Docker Linux daemon/FUSE; 2 CPU / 2 GiB / no swap / 256 PIDs |
| Seed / samples | seed 1, one sample per case |
| Performance allowance | 300 s product / 310 s outer; 15 s is reporting-only |
| Verification | sampled only; 45 s work / 59 s hard |

Product bytes are unchanged.

## Inventory

`tiny_file_churn` cardinality remains 20. All six new 100/500 `-mixed-v4` tiny-op IDs were attempted once.

| Attempted | COMPLETE | INCOMPLETE | Proof PASS |
| ---: | ---: | ---: | ---: |
| 6 | 6 | 0 | 6 |

## Performance table

Timer is `pure_call_sum_ns`. Historical 15 s is reporting-only.

| Case | Background files / bytes | Status | pure_call_sum_ns | create / exec / commit / vis / end (ms) | Hist. 15 s | Proof |
|---|---|---|---:|---|---|---|
| `tiny-create-100-mixed-v4` | 2,000 / 100 MiB | COMPLETE | 91,400,959 | 9.404 / 70.234 / 8.998 / 0.068 / 2.696 | PASS | PASS 2.3 s |
| `tiny-create-500-mixed-v4` | 5,000 / 500 MiB | COMPLETE | 322,892,250 | 7.625 / 287.631 / 23.913 / 0.075 / 3.648 | PASS | PASS 4.4 s |
| `tiny-stat-100-mixed-v4` | 2,000 / 100 MiB + 500 tiny | COMPLETE | 65,483,625 | 8.708 / 52.643 / 1.749 / 0.075 / 2.309 | PASS | PASS 2.3 s |
| `tiny-stat-500-mixed-v4` | 5,000 / 500 MiB + 500 tiny | COMPLETE | 218,262,625 | 7.832 / 207.022 / 1.084 / 0.073 / 2.251 | PASS | PASS 4.2 s |
| `tiny-unlink-100-mixed-v4` | 2,000 / 100 MiB + 500 tiny | COMPLETE | 77,793,082 | 9.000 / 60.408 / 6.025 / 0.067 / 2.292 | PASS | PASS 2.3 s |
| `tiny-unlink-500-mixed-v4` | 5,000 / 500 MiB + 500 tiny | COMPLETE | 300,545,416 | 8.393 / 278.448 / 11.179 / 0.062 / 2.463 | PASS | PASS 4.6 s |

Create/unlink Commit `Created`; stat `UpToDate`. All `presentation_failed: false`. Swap current 0.

## What this does not close

Waves 1–3 plus this background slice complete the #62 mixed-v4 membership. Old 100k-file product leftovers stay on [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61). Product Exec/Commit optimization stays on #46 / #47 / #48 / #50. #39 stays open.

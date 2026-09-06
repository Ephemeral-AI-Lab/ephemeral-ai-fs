# Issue 62 — mixed-v4 Wave 1 results

Status: **Wave 1 collected**. This is a one-sample statistics campaign on the new `workspace-mixed-v4` trees. It is not a 15-second qualification, not a three-seed distribution, and not an Exec/Commit product optimization. [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61) stays open for leftover product rows on the **old** 100k-file trees. [#39](https://github.com/Ephemeral-AI-Lab/layerfs/issues/39) stays open.

Machine-readable table: [issue62-mixed-v4-wave1-results.json](issue62-mixed-v4-wave1-results.json).
Raw receipts (local, gitignored): `benchmark-results/host-store/campaigns/issue62-004448cf25a350a0/`.

## Campaign identities

| Item | Value |
|---|---|
| Product seal | `0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695` |
| Source seal | `004448cf25a350a08ea0eab6d7e1e617285f6e5d7bde5b51696466cb467d3666` |
| Image | `layerfs-bench-infra:004448cf25a350a0` (`sha256:042f3f377ec9879b0c36644a878ceef1c1b17205ee5f9809558e63278f889b3b`) |
| Topology | host-store: macOS SDK/SQLite/spool; Docker Linux daemon/FUSE; 2 CPU / 2 GiB / no swap / 256 PIDs; no data mounts |
| Seed / samples | seed 1, one sample per Wave 1 case |
| Performance allowance | 300 s product / 310 s outer; 15 s family target is **reporting-only** |
| Verification | sampled only; 45 s work / 59 s hard |

Product bytes are unchanged by the migration commit. The source seal covers harness/docs/fixtures only.

## Disclaimer (do not read as a speedup)

New times on 2k/5k mixed trees are a **different workload** from the old 20k/100k tiny-file rows. Do not compare as a product speedup from fewer files. Do not relabel [#54](https://github.com/Ephemeral-AI-Lab/layerfs/issues/54) 100k-file receipts as mixed-v4 results.

| Old row | Old tree | Old outcome |
|---|---|---|
| `workspace-dense-rewrite-100` | 20,000 files / 100 MiB / ≤48 KiB | Exec+Commit then FUSE presentation FAIL |
| `workspace-dense-rewrite-500` | 100,000 files / 500 MiB | `fsyncdir .: ENOSPC` after 100k pwrites; 8.36 ms published is Create only |
| `workspace-clean-commit-500` | 100k untouched tiny files | 14.5 ms PASS |
| `git-tool-100/500` | 6,750 files / 34 MiB | ~20.9 / 21.1 s; Wave 3, not this report |

## Wave 1 inventory

Family `workspace_change_locality` cardinality remains 16. Compact 1/10 IDs were not rerun. All eight new 100/500 IDs were attempted once.

| Attempted | COMPLETE | INCOMPLETE | Proof PASS | Proof SKIPPED |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 8 | 0 | 8 | 0 |

## Performance table

Timer is `pure_call_sum_ns`. One seed, one sample: not a median. Historical 15 s is reporting-only. Phases in milliseconds.

| Case | Files / bytes / large | Status / cleanup | pure_call_sum_ns | create / exec / commit / vis / end (ms) | Hist. 15 s | command_wall (ms) | container peak |
|---|---|---|---:|---|---|---:|---:|
| `workspace-dense-rewrite-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 2,434,445,875 | 8.759 / 1774.171 / 641.995 / 0.087 / 9.433 | PASS | 2,802 | 121,815,040 |
| `workspace-dense-rewrite-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 8,334,001,500 | 6.654 / 5584.181 / 2727.789 / 0.091 / 15.287 | PASS | 8,858 | 536,043,520 |
| `workspace-clean-commit-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 11,507,583 | 8.026 / — / 1.388 / 0.088 / 2.005 | PASS | 389 | 4,603,904 |
| `workspace-clean-commit-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 12,952,667 | 8.826 / — / 1.634 / 0.073 / 2.420 | PASS | 384 | 4,620,288 |
| `workspace-fixed-move-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 20,821,250 | 7.934 / 6.262 / 4.055 / 0.065 / 2.505 | PASS | 384 | 4,616,192 |
| `workspace-fixed-move-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 23,177,458 | 8.731 / 6.413 / 5.106 / 0.075 / 2.852 | PASS | 386 | 5,394,432 |
| `workspace-distributed-sdk-edit-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 355,760,339 | 9.330 / 320.448 sdk / 23.422 / 0.090 / 2.470 | PASS | 758 | 4,911,104 |
| `workspace-distributed-sdk-edit-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 2,818,321,215 | 7.699 / 2712.760 sdk / 95.083 / 0.082 / 2.697 | PASS | 3,254 | 10,280,960 |

Commit results: dense-rewrite / fixed-move / sdk-edit `Created` with `presentation_failed: false`. Clean-commit `UpToDate`. Swap current was 0 on every row.

A historical 15 s PASS on these mixed trees is **not** a 15 s PASS claimed from fewer files of the old workload.

## Dense-rewrite extra counters

| Case | file writes | write bytes | pwrite | open | fsyncdir | spool peak |
|---|---:|---:|---:|---:|---:|---:|
| `workspace-dense-rewrite-100-mixed-v4` | 2,000 | 104,857,600 | 2,049 | 2,000 | 1 | 104,927,232 |
| `workspace-dense-rewrite-500-mixed-v4` | 5,000 | 524,288,000 | 5,398 | 5,000 | 1 | 524,632,064 |

Dense-500 is 500 MiB tree + ~500 MiB rewrite spool inside the 2 GiB / no-swap container. That is the allowed peak-bytes case. It is not the old 500 MiB tiny parent + 500 MiB rewrite + 100k inodes failure.

## Sampled proofs

Proofs ran only after COMPLETE performance. Coverage: every large file, three 64 KiB ranges (begin, midpoint, end), plus declared small/medium paths and topology. Omissions: unselected paths, bytes beyond selected ranges, exhaustive inode/object/reference census, alias and failure injection.

| Case | Proof | Wall s |
|---|---|---:|
| `workspace-dense-rewrite-100-mixed-v4` | PASS | 5.003 |
| `workspace-dense-rewrite-500-mixed-v4` | PASS | 13.616 |
| `workspace-clean-commit-100-mixed-v4` | PASS | 2.149 |
| `workspace-clean-commit-500-mixed-v4` | PASS | 4.084 |
| `workspace-fixed-move-100-mixed-v4` | PASS | 2.233 |
| `workspace-fixed-move-500-mixed-v4` | PASS | 3.995 |
| `workspace-distributed-sdk-edit-100-mixed-v4` | PASS | 2.462 |
| `workspace-distributed-sdk-edit-500-mixed-v4` | PASS | 6.722 |

No exhaustive 100/500 walks were rerun at 300 s.

## What this does not close

- Wave 2 directory 100/500 mixed-v4 collection is reported separately in [issue62-mixed-v4-wave2-results.md](issue62-mixed-v4-wave2-results.md). Wave 3 (`namespace-subtree-relocate-delete` and `git-tool` mixed-v4) is not collected. Git mixed-v4 is not registered until the 256 MiB `.git`+tree bound is proven.
- [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61) remains the home of old 100k-file ENOSPC/presentation leftovers.
- Product Exec/Commit optimization remains [#46](https://github.com/Ephemeral-AI-Lab/layerfs/issues/46) / [#47](https://github.com/Ephemeral-AI-Lab/layerfs/issues/47) / [#48](https://github.com/Ephemeral-AI-Lab/layerfs/issues/48) / [#50](https://github.com/Ephemeral-AI-Lab/layerfs/issues/50).

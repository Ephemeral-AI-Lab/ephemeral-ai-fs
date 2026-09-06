# Issue 62 — mixed-v4 Wave 3 results

Status: **Wave 3 collected**. Namespace uses the Wave 1/2 `workspace-mixed-v4` tree. Git uses the git-safe `workspace-mixed-v4-git` tree (not the 500 MiB workspace tree). This is a one-sample statistics campaign, not a 15-second qualification and not an Exec/Commit product optimization. [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61) stays open.

Machine-readable table: [issue62-mixed-v4-wave3-results.json](issue62-mixed-v4-wave3-results.json).
Raw receipts (local, gitignored): `benchmark-results/host-store/campaigns/issue62-wave3/`.

## Campaign identities

| Item | Value |
|---|---|
| Product seal | `0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695` |
| Source seal | `b7b5b22ccc73eccb11600f98109c62992f6eaaabbe95c9eb18bb1d4bc86db147` |
| Image | `layerfs-bench-infra:b7b5b22ccc73eccb` (`sha256:64a3e1d7889f82f1452f2d3d22fdc453a6a5b51d36048b21654d3277a8e9b1df`) |
| Topology | host-store: macOS SDK/SQLite/spool; Docker Linux daemon/FUSE; 2 CPU / 2 GiB / no swap / 256 PIDs; no data mounts |
| Seed / samples | seed 1, one sample per Wave 3 case |
| Performance allowance | 300 s product / 310 s outer; 15 s family target is **reporting-only** |
| Verification | sampled only; 45 s work / 59 s hard |

Product bytes are unchanged. Compact 1/10 IDs were not rerun. Source seal differs from Waves 1–2 because Wave 3 added the git-safe fixture and sampled-proof adjustment; product seal is the same.

## Disclaimer (do not read as a speedup)

New times are a **different workload** from the old 100k-file namespace trees and the old 6,750-file / 34 MiB git trees. Do not compare as a product speedup. Do not relabel [#54](https://github.com/Ephemeral-AI-Lab/layerfs/issues/54) receipts as mixed-v4 results.

| Old row | Old tree | Old outcome |
|---|---|---|
| `git-tool-100` / `git-tool-500` | 6,750 files / 34 MiB | ~20.9 / 21.1 s; status+diff ≈ 20 s FUSE metadata; Commit 71–136 ms |

Git mixed-v4 working trees are 61.5 MiB / 73.8 MiB with one ignored 50 MiB blob. A fixture self-check proved the conservative `.git`+tree bound stays ≤ 256 MiB and `git ls-files` does not contain `wide/s000-f000.dat`.

## Wave 3 inventory

`namespace_mutation` cardinality remains 4. `git_tool_workflow` cardinality remains 4.

| Attempted | COMPLETE | INCOMPLETE | Proof PASS | Proof SKIPPED |
| ---: | ---: | ---: | ---: | ---: |
| 4 | 4 | 0 | 4 | 0 |

## Performance table

Timer is `pure_call_sum_ns`. One seed, one sample: not a median. Historical 15 s is reporting-only. Phases in milliseconds.

| Case | Files / bytes / large | Status / cleanup | pure_call_sum_ns | create / exec / commit / vis / end (ms) | Hist. 15 s | command_wall (ms) | container peak |
|---|---|---|---:|---|---|---:|---:|
| `namespace-subtree-relocate-delete-100-mixed-v4` | 2,400 / 105,267,200 / 50 MiB | COMPLETE / PASS | 42,299,833 | 10.470 / 20.447 / 7.027 / 0.102 / 4.255 | PASS | 387 | 4,870,144 |
| `namespace-subtree-relocate-delete-500-mixed-v4` | 7,000 / 526,336,000 / 300+100 MiB | COMPLETE / PASS | 208,677,542 | 8.147 / 178.622 / 18.507 / 0.074 / 3.328 | PASS | 607 | 4,882,432 |
| `git-tool-100-mixed-v4` | 2,351 / 61,491,723 / 50 MiB ignored | COMPLETE / PASS | 6,862,018,751 | 12.844 / 6790.348 / 43.761 / 0.135 / 14.930 | PASS | 7,239 | 48,726,016 |
| `git-tool-500-mixed-v4` | 5,351 / 73,779,723 / 50 MiB ignored | COMPLETE / PASS | 18,446,083,292 | 13.849 / 18287.907 / 119.170 / 0.083 / 25.075 | TARGET_MISS | 18,820 | 102,277,120 |

All four Commit `Created` with `presentation_failed: false`. Swap current was 0. `git-tool-500-mixed-v4` is a completed collection row whose historical 15 s classifier is TARGET_MISS; that is reporting-only.

## Namespace extra counters

| Case | rename | unlink | rmdir | move_ns | delete_ns | fsyncdir |
|---|---:|---:|---:|---:|---:|---:|
| `namespace-subtree-relocate-delete-100-mixed-v4` | 1 | 200 | 2 | 2,335,292 | 13,282,416 | 1 |
| `namespace-subtree-relocate-delete-500-mixed-v4` | 1 | 1,000 | 6 | 2,345,958 | 171,393,333 | 1 |

Affected subtrees are 200 / 1,000 files per tree (hundreds to low thousands), not 20k/100k, and there is no second 100k-file parent.

## Git extra counters

| Case | git processes | status ns | diff ns | add ns | git commit ns | LayerFS commit ns |
|---|---:|---:|---:|---:|---:|---:|
| `git-tool-100-mixed-v4` | 6 | 2,312,779,752 | 4,007,004,835 | 166,017,292 | 83,031,875 | 43,761,000 |
| `git-tool-500-mixed-v4` | 6 | 5,724,367,586 | 10,708,469,088 | 744,803,417 | 192,479,292 | 119,170,000 |

Exec is still dominated by `git status` + `git diff --binary` through FUSE. LayerFS Commit is 44–119 ms. Isolation env and `GIT_CONFIG` were unchanged. The 50 MiB blob was not git-added.

## Sampled proofs

Proofs ran only after COMPLETE performance. Coverage: every large file, three 64 KiB ranges (begin, midpoint, end), plus declared small/medium paths and topology. Git samples omit workspace-root `.` metadata because Git updates that directory while writing `.git`. Omissions: unselected paths, bytes beyond selected ranges, exhaustive inode/object/reference census, alias and failure injection.

| Case | Proof | Wall s |
|---|---|---:|
| `namespace-subtree-relocate-delete-100-mixed-v4` | PASS | 2.600 |
| `namespace-subtree-relocate-delete-500-mixed-v4` | PASS | 5.369 |
| `git-tool-100-mixed-v4` | PASS | 13.423 |
| `git-tool-500-mixed-v4` | PASS | 28.426 |

## What this does not close

- Background-only `tiny-{create,stat,unlink}-{100,500}-mixed-v4` is still later work.
- [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61) remains the home of old 100k-file ENOSPC/presentation leftovers.
- Product Exec/Commit optimization remains #46 / #47 / #48 / #50.

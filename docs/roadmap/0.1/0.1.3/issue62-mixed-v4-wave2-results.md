# Issue 62 — mixed-v4 Wave 2 results

Status: **Wave 2 collected**. Same campaign identities as [Wave 1](issue62-mixed-v4-wave1-results.md). This is a one-sample statistics campaign on the new `workspace-mixed-v4` trees. It is not a 15-second qualification and not an Exec/Commit product optimization. [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61) stays open for leftover product rows on the **old** 100k-file trees.

Machine-readable table: [issue62-mixed-v4-wave2-results.json](issue62-mixed-v4-wave2-results.json).
Raw receipts (local, gitignored): `benchmark-results/host-store/campaigns/issue62-004448cf25a350a0/wave2/`.

## Campaign identities

| Item | Value |
|---|---|
| Product seal | `0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695` |
| Source seal | `004448cf25a350a08ea0eab6d7e1e617285f6e5d7bde5b51696466cb467d3666` |
| Image | `layerfs-bench-infra:004448cf25a350a0` (`sha256:042f3f377ec9879b0c36644a878ceef1c1b17205ee5f9809558e63278f889b3b`) |
| Topology | host-store: macOS SDK/SQLite/spool; Docker Linux daemon/FUSE; 2 CPU / 2 GiB / no swap / 256 PIDs; no data mounts |
| Seed / samples | seed 1, one sample per Wave 2 case |
| Performance allowance | 300 s product / 310 s outer; 15 s family target is **reporting-only** |
| Verification | sampled only; 45 s work / 59 s hard |

Product bytes are unchanged. Compact 1/10 directory IDs were not rerun.

## Disclaimer (do not read as a speedup)

New times on 2k/5k mixed trees are a **different workload** from the old 20k/100k tiny-file rows. Do not compare as a product speedup from fewer files. Do not relabel [#54](https://github.com/Ephemeral-AI-Lab/layerfs/issues/54) 100k-file receipts as mixed-v4 results.

| Old row | Old tree | Old outcome |
|---|---|---|
| `directory-metadata-scan-500` | 100,000 files / 32k-entry `wide/` | FAIL, `readdir wide: ENOSPC` |
| `directory-content-scan-500` | 100,000 files / 32k-entry `wide/` | FAIL, `readdir wide: ENOSPC` after 136,000 preads |

Wave 2 mixed-v4 wide-directory fan-out is 640 entries at tier 100 and 1,600 at tier 500.

## Wave 2 inventory

Family `directory_construction_traversal` cardinality remains 12. Compact 1/10 IDs were not rerun. All six new 100/500 IDs were attempted once.

| Attempted | COMPLETE | INCOMPLETE | Proof PASS | Proof SKIPPED |
| ---: | ---: | ---: | ---: | ---: |
| 6 | 6 | 0 | 6 | 0 |

## Performance table

Timer is `pure_call_sum_ns`. One seed, one sample: not a median. Historical 15 s is reporting-only. Phases in milliseconds.

| Case | Files / bytes / large | Status / cleanup | pure_call_sum_ns | create / exec / commit / vis / end (ms) | Hist. 15 s | command_wall (ms) | container peak |
|---|---|---|---:|---|---|---:|---:|
| `directory-metadata-scan-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 229,157,292 | 9.310 / 213.280 / 1.805 / 0.081 / 4.681 | PASS | 603 | 9,084,928 |
| `directory-metadata-scan-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 410,722,625 | 8.233 / 390.220 / 1.491 / 0.083 / 10.695 | PASS | 798 | 15,036,416 |
| `directory-content-scan-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 1,600,999,042 | 20.082 / 1567.923 / 7.033 / 0.081 / 5.879 | PASS | 1,990 | 116,830,208 |
| `directory-content-scan-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 5,306,648,124 | 8.846 / 5255.829 / 26.159 / 0.172 / 15.643 | PASS | 5,707 | 544,616,448 |
| `directory-construct-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 243,814,916 | 8.629 / 215.158 / 16.572 / 0.094 / 3.362 | PASS | 637 | 5,210,112 |
| `directory-construct-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 1,232,776,041 | 8.063 / 1114.490 / 101.484 / 0.104 / 8.635 | PASS | 1,607 | 11,333,632 |

Commit results: metadata/content scans `UpToDate` with `presentation_failed: false`. Construct `Created` with `presentation_failed: false`. Swap current was 0 on every row.

A historical 15 s PASS on these mixed trees is **not** a 15 s PASS claimed from fewer files of the old workload.

## Scan and construct extra counters

| Case | visited paths / files | dir entries | lstat | opendir | pread | read bytes | open | mkdir / chains | fsyncdir |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `directory-metadata-scan-100-mixed-v4` | 2,143 / 2,000 | 2,142 | 2,143 | 143 | 0 | 0 | 0 | 0 / 0 | 0 |
| `directory-metadata-scan-500-mixed-v4` | 5,158 / 5,000 | 5,157 | 5,158 | 158 | 0 | 0 | 0 | 0 / 0 | 0 |
| `directory-content-scan-100-mixed-v4` | 2,143 / 2,000 | 2,142 | 2,143 | 143 | 4,049 | 104,857,600 | 2,000 | 0 / 0 | 0 |
| `directory-content-scan-500-mixed-v4` | 5,158 / 5,000 | 5,157 | 5,158 | 158 | 10,398 | 524,288,000 | 5,000 | 0 / 0 | 0 |
| `directory-construct-100-mixed-v4` | 0 / 0 | 0 | 0 | 0 | 0 | 0 | 0 | 550 / 100 | 1 |
| `directory-construct-500-mixed-v4` | 0 / 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2,750 / 500 | 1 |

Content-scan-500 read the full 500 MiB mixed tree through FUSE without the old 32k-entry `wide/` ENOSPC. Opendir counts (143 / 158) match a few-hundred-to-low-thousands wide directory, not 32,000.

## Sampled proofs

Proofs ran only after COMPLETE performance. Coverage: every large file, three 64 KiB ranges (begin, midpoint, end), plus declared small/medium paths and topology. Omissions: unselected paths, bytes beyond selected ranges, exhaustive inode/object/reference census, alias and failure injection.

| Case | Proof | Wall s |
|---|---|---:|
| `directory-metadata-scan-100-mixed-v4` | PASS | 2.581 |
| `directory-metadata-scan-500-mixed-v4` | PASS | 4.811 |
| `directory-content-scan-100-mixed-v4` | PASS | 3.916 |
| `directory-content-scan-500-mixed-v4` | PASS | 9.806 |
| `directory-construct-100-mixed-v4` | PASS | 2.705 |
| `directory-construct-500-mixed-v4` | PASS | 5.551 |

No exhaustive 100/500 walks were rerun at 300 s.

## What this does not close

- Wave 3 (`namespace-subtree-relocate-delete` and `git-tool` mixed-v4) is not collected. Git mixed-v4 is not registered until the 256 MiB `.git`+tree bound is proven.
- [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61) remains the home of old 100k-file ENOSPC/presentation leftovers, including the historical `readdir wide: ENOSPC` rows.

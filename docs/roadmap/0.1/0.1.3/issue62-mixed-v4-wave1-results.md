# Issue 62 — mixed-v4 Wave 1 results

Status: **Wave 1 collected**. Shared fixture `workspace-mixed-v4` is registered for `workspace_change_locality` 100/500. Family cardinality remains 16. Compact 1/10 IDs are unchanged. This is **benchmark infrastructure**. It does not close #39, #46, #47, #48, #50, or #61.

Machine-readable table: [issue62-mixed-v4-wave1-results.json](issue62-mixed-v4-wave1-results.json).

Raw receipts (local, gitignored): `benchmark-results/host-store/campaigns/issue62-58e1e5118572c834/`.

**Do not compare these times with the old 20k/100k tiny-file rows as a product speedup.** The new 2k/5k mixed trees are a different workload. Historical 15-second family status is reporting-only.

## Campaign identities

| Item | Value |
|---|---|
| Product seal | `0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695` (unchanged vs #54; product crates were not edited) |
| Source seal | `58e1e5118572c834a20736ff59d261a31c0aafcd023d520c207811c4109333ea` |
| Image | `layerfs-bench-infra:58e1e5118572c834` (`sha256:e2078498d4ade952ccfa18fbaf40464d271156773e0bd860870face00b6080bf`) |
| Topology | host-store: macOS SDK/SQLite/spool; Docker Linux daemon/FUSE; 2 CPU / 2 GiB / no swap / 256 PIDs; no data mounts |
| Seed / samples | seed 1, one sample per Wave 1 case |
| Performance | 300 s product / 310 s outer; collection-mode |
| Proofs | 45 s work / 59 s hard; sampled large-file ranges only |

Compatible prepared masters were reused across same-tier mixed-v4 cases after the first 100 MiB and 500 MiB preparations. Clone method remains closed-store copy. Cleanup PASS on every row.

## Wave 1 performance

Timer is `pure_call_sum_ns`. Phases are milliseconds. SDK-edit rows have no Exec; the `sdk-edit` column is the sum of N singular public SDK calls.

| Case | Files / bytes / large | Status / cleanup | `pure_call_sum_ns` | create / exec / sdk-edit / commit / vis / end (ms) | 15 s reporting | command_wall | container peak |
|---|---|---|---:|---|---|---:|---:|
| `workspace-dense-rewrite-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 2,698,577,751 | 10.294 / 2005.855 / — / 672.786 / 0.073 / 9.570 | PASS | 3.047 s | 120.6 MiB |
| `workspace-dense-rewrite-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 8,750,610,124 | 7.679 / 6005.751 / — / 2717.426 / 0.093 / 19.660 | PASS | 9.263 s | 534.7 MiB |
| `workspace-clean-commit-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 13,306,165 | 9.491 / — / — / 1.534 / 0.086 / 2.195 | PASS | 0.390 s | 5.4 MiB |
| `workspace-clean-commit-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 12,215,710 | 8.275 / — / — / 1.616 / 0.068 / 2.258 | PASS | 0.390 s | 5.1 MiB |
| `workspace-fixed-move-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 23,403,626 | 9.145 / 6.816 / — / 4.734 / 0.068 / 2.641 | PASS | 0.384 s | 5.2 MiB |
| `workspace-fixed-move-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 25,382,792 | 9.900 / 6.961 / — / 5.646 / 0.068 / 2.809 | PASS | 0.393 s | 4.9 MiB |
| `workspace-distributed-sdk-edit-100-mixed-v4` | 2,000 / 100 MiB / 50 MiB | COMPLETE / PASS | 263,750,617 | 8.567 / — / 231.342 / 21.591 / 0.067 / 2.183 | PASS | 0.645 s | 5.1 MiB |
| `workspace-distributed-sdk-edit-500-mixed-v4` | 5,000 / 500 MiB / 300+100 MiB | COMPLETE / PASS | 2,694,829,043 | 9.401 / — / 2588.073 / 94.814 / 0.070 / 2.470 | PASS | 3.101 s | 10.0 MiB |

A reporting-only 15 s PASS on these mixed trees is not a 15-second qualification of the old 100k-file IDs.

### Dense-rewrite Exec accounting

Existing-file writes at offset 0, no truncate, no per-file sync, then mtime `1700000000` and one root `fsyncdir`.

| Case | file writes | write bytes | pwrite | open | fsync | fsyncdir | spool peak |
|---|---:|---:|---:|---:|---:|---:|---:|
| `workspace-dense-rewrite-100-mixed-v4` | 2,000 | 104,857,600 | 2,049 | 2,000 | 0 | 1 | 104,927,232 |
| `workspace-dense-rewrite-500-mixed-v4` | 5,000 | 524,288,000 | 5,398 | 5,000 | 0 | 1 | 524,632,064 |

Dense-500 is a 500 MiB tree plus ~500 MiB rewrite spool inside the 2 GiB / no-swap container. That is allowed. It is not the old 100k-inode ENOSPC row.

## Sampled proofs

Every large file was covered with three 64 KiB ranges (begin, midpoint, end), plus declared small/medium paths and topology. Proofs did not read every large-file byte. Omissions: unselected paths, bytes beyond selected ranges, exhaustive inode/object/reference census, alias and failure injection.

| Case | Proof | Wall s | Cleanup |
|---|---|---:|---|
| `workspace-dense-rewrite-100-mixed-v4` | PASS | 4.99 | PASS |
| `workspace-dense-rewrite-500-mixed-v4` | PASS | 13.30 | PASS |
| `workspace-clean-commit-100-mixed-v4` | PASS | 2.45 | PASS |
| `workspace-clean-commit-500-mixed-v4` | PASS | 4.37 | PASS |
| `workspace-fixed-move-100-mixed-v4` | PASS | 2.30 | PASS |
| `workspace-fixed-move-500-mixed-v4` | PASS | 4.51 | PASS |
| `workspace-distributed-sdk-edit-100-mixed-v4` | PASS | 2.52 | PASS |
| `workspace-distributed-sdk-edit-500-mixed-v4` | PASS | 6.81 | PASS |

No Wave 1 proof was SKIPPED. None hit the 45/59 s ceiling.

## Old vs new (not a speedup)

| Row | Old #54 tree | Old outcome | New mixed-v4 |
|---|---|---|---|
| dense-rewrite-100 | 20,000 files / 100 MiB / ≤48 KiB | Exec+Commit then FUSE presentation FAIL | 2,000 files / 100 MiB / 50 MiB object; COMPLETE 2.70 s; sampled proof PASS |
| dense-rewrite-500 | 100,000 files / 500 MiB | `fsyncdir .: ENOSPC` after 100k pwrites; 8.36 ms published was Create only | 5,000 files / 500 MiB / 300+100 MiB objects; COMPLETE 8.75 s Exec+Commit; sampled proof PASS |
| clean-commit-500 | 14.5 ms PASS on 100k untouched tiny files | COMPLETE | 12.2 ms on 5k mixed files; different workload |
| git-100/500 | 6,750 files / 34 MiB; ~20.9 / 21.1 s | not in Wave 1 | Git exception is Wave 3; 500 MiB mixed tree cannot be the Git working tree |

#61 stays open for leftover product failures on the **old** 100k-file trees. A mixed-v4 PASS does not close those rows.

## Not in this wave

Wave 2 directory 100/500 IDs and Wave 3 namespace/git IDs are not collected here. The shared generator is in place so those families can emit `-mixed-v4` later. Git 256 MiB bound is proven in the fixture self-check (1×50 MiB + remaining 4 KiB files + 2,500-byte change schedule; conservative `.git`+tree ≤ 256 MiB) but git mixed-v4 IDs are not registered.

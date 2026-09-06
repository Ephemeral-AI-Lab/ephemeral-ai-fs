# Issue 65 — history-unrelated-mixed-v2 results

Status: **collected**. This is a one-sample statistics campaign on the new
`history-unrelated-mixed-v2` trees. It is not a 15-second qualification, not a
three-seed distribution, and not an Exec/Commit product optimization.
[#21](https://github.com/Ephemeral-AI-Lab/layerfs/issues/21) and
[#39](https://github.com/Ephemeral-AI-Lab/layerfs/issues/39) stay open.

Machine-readable table: [issue65-unrelated-mixed-v2-results.json](issue65-unrelated-mixed-v2-results.json).
Raw receipts (local, gitignored): `benchmark-results/host-store/campaigns/issue65-unrelated-mixed-v2/`.

## Campaign identities

| Item | Value |
|---|---|
| Product seal | `0999a1259161c245110e525e22b6db888cf4241872e190b36e2dcb617790e695` |
| Source seal | `cab364a18273c0ce8cbcdf726ed375974e0ea5eb5438d143a471e8164a09052a` |
| Image | `layerfs-bench-infra:cab364a18273c0ce` (`sha256:bb1c1c92b901498073e60ae4a211dcae84f6cfe25c3bbe8dce4a367ce8daf391`) |
| Topology | host-store: macOS SDK/SQLite/spool; Docker Linux daemon/FUSE; 2 CPU / 2 GiB / no swap / 256 PIDs; no data mounts |
| Seed / samples | seed 1, one sample per mixed-v2 case |
| Performance allowance | 300 s product / 310 s outer; 15 s family target is **reporting-only** |
| Verification | sampled only; 45 s work / 59 s hard |

Product bytes are unchanged by the migration commit. The source seal covers harness/docs/fixtures only.

## Disclaimer (do not read as a speedup)

New times on the 10-file mixed tree are a **different workload** from the old
200-file unrelated-100/500 rows. Do not compare as a product speedup from fewer
file writes. Do not relabel [#54](https://github.com/Ephemeral-AI-Lab/layerfs/issues/54)
90.1 s / timeout-at-106 receipts as mixed-v2 results.

| Old row | Old tree | Old outcome |
|---|---|---|
| `dedup-history-unrelated-100` | 200 files / 1 MiB, 20,000 writes | 90.1 s COMPLETE, historical 15 s TARGET_MISS |
| `dedup-history-unrelated-500` | 200 files / 1 MiB, 100,000 writes | TIMEOUT at 300 s around commit 106/500 |

Those receipts stay historical on [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61).

## Inventory

Family `dedup_branch_history` cardinality remains 20. Only unrelated-100/500 IDs
are versioned. Unrelated-1/10 and the other four history kinds were not rerun.

| Attempted | COMPLETE | INCOMPLETE | Proof PASS | Proof SKIPPED |
| ---: | ---: | ---: | ---: | ---: |
| 2 | 2 | 0 | 2 | 0 |

## Performance table

Timer is `pure_call_sum_ns`. One seed, one sample: not a median. Historical 15 s
is reporting-only. Phases in milliseconds. Fixture is 10 files / 1,048,576 bytes
/ one 640 KiB file.

| Case | Files / bytes / large | Status / cleanup | pure_call_sum_ns | create / exec / commit / vis / end (ms) | Hist. 15 s | command_wall (ms) | file writes / write bytes | Commits |
|---|---|---|---:|---|---|---:|---|---:|
| `dedup-history-unrelated-100-mixed-v2` | 10 / 1 MiB / 640 KiB | COMPLETE / PASS | 3,492,535,202 | 8.019 / 2698.839 / 783.353 / 0.070 / 2.254 | PASS | 3,949 | 1,000 / 104,857,600 | 100 Created |
| `dedup-history-unrelated-500-mixed-v2` | 10 / 1 MiB / 640 KiB | COMPLETE / PASS | 16,997,840,520 | 6.831 / 12926.866 / 4060.541 / 0.076 / 3.527 | TARGET_MISS | 17,979 | 5,000 / 524,288,000 | 500 Created |

Last Commit on both rows: `Created` with `presentation_failed: false`. Swap current was 0. Container peak was 9,256,960 bytes. Unique 100/500 MiB lives on the host Store, not as 100/500 live copies.

A historical 15 s PASS on mixed-v2 100, or TARGET_MISS on mixed-v2 500, is **not** a 15 s result claimed from fewer files of the old 200-file workload.

## Sampled proofs

Proofs ran only after COMPLETE performance. Coverage: the one 640 KiB file, three
64 KiB ranges (begin `0..65536`, midpoint `327680..393216`, end `589824..655360`);
declared small paths `mixed/f001.dat` and `mixed/f006.dat`; declared medium paths
`mixed/f007.dat` and `mixed/f009.dat`; directories `.` and `mixed`; Commit query
membership and parent topology. Omissions: unselected paths, bytes beyond selected
ranges, exhaustive unique 1 MiB snapshots, inode/object/reference census, alias
and failure injection.

| Case | Proof | Wall s |
|---|---|---:|
| `dedup-history-unrelated-100-mixed-v2` | PASS | 5.260 |
| `dedup-history-unrelated-500-mixed-v2` | PASS | 19.317 |

## What this does not close

- [#21](https://github.com/Ephemeral-AI-Lab/layerfs/issues/21) v0.1.3 umbrella / release qualification.
- [#39](https://github.com/Ephemeral-AI-Lab/layerfs/issues/39) Phase 2 restore/optimize; do not restore the old 200×500 unrelated recipe.
- Product Exec/Commit [#46](https://github.com/Ephemeral-AI-Lab/layerfs/issues/46) / [#47](https://github.com/Ephemeral-AI-Lab/layerfs/issues/47) / [#48](https://github.com/Ephemeral-AI-Lab/layerfs/issues/48) / [#50](https://github.com/Ephemeral-AI-Lab/layerfs/issues/50).
- Proof/oracle leftovers on [#61](https://github.com/Ephemeral-AI-Lab/layerfs/issues/61) (git-tool `.` metadata, Busy, fault/errno, presentation TIMEOUT, sustained-600s).

# Issue 49 final requested tiny-churn benchmark

**20/20 performance cases PASS and 7/7 selected independent proofs PASS for the requested benchmark scope.** Collected on 2026-09-06. One seed-1 observation per registered case: 19 new samples and the exact matching create-100 result reused unchanged. The implementation and harness were held fixed throughout collection.

At the user's explicit request, this report is the closure handoff for **#49 only**. It does not close #46, #47, #50, #51 or other issues. The separate strict tier100 create target still misses; this report does not assert that every broader redesign/checklist, recovery or scale obligation has been independently qualified. The implementation history and remaining limits are retained in [the live Workspace ledger](issue49-live-workspace-ledger.md).

## Per-case timing

Seconds to four decimal places; status uses unrounded nanoseconds. Total is the declared complete-lifecycle `pure_call_sum_ns`, including Create + Exec + Commit + visibility + End. Each row is one observation, not a median distribution or percentile estimate. The family threshold is 15 seconds.

| Case | Create | Exec | Commit | Visibility | End | Total | Family |
|---|---:|---:|---:|---:|---:|---:|---|
| tiny-create-1-compact-v2 | 0.0108 | 0.0068 | 0.0034 | 0.0001 | 0.0027 | 0.0238 | PASS |
| tiny-create-10-compact-v2 | 0.0089 | 0.0179 | 0.0050 | 0.0001 | 0.0025 | 0.0343 | PASS |
| tiny-create-100 | 0.0104 | 0.0646 | 0.0594 | 0.0001 | 0.0034 | 0.1378 | PASS |
| tiny-create-500 | 0.0078 | 0.3030 | 0.1050 | 0.0001 | 0.0040 | 0.4199 | PASS |
| tiny-stat-1-compact-v2 | 0.0075 | 0.0053 | 0.0015 | 0.0001 | 0.0027 | 0.0171 | PASS |
| tiny-stat-10-compact-v2 | 0.0079 | 0.0124 | 0.0015 | 0.0001 | 0.0021 | 0.0240 | PASS |
| tiny-stat-100 | 0.0077 | 0.0646 | 0.0016 | 0.0001 | 0.0021 | 0.0761 | PASS |
| tiny-stat-500 | 0.0088 | 0.2613 | 0.0015 | 0.0001 | 0.0027 | 0.2743 | PASS |
| tiny-unlink-1-compact-v2 | 0.0084 | 0.0069 | 0.0032 | 0.0001 | 0.0030 | 0.0215 | PASS |
| tiny-unlink-10-compact-v2 | 0.0077 | 0.0182 | 0.0043 | 0.0001 | 0.0029 | 0.0332 | PASS |
| tiny-unlink-100 | 0.0108 | 0.0785 | 0.0474 | 0.0001 | 0.0034 | 0.1402 | PASS |
| tiny-unlink-500 | 0.0096 | 0.3205 | 0.0816 | 0.0001 | 0.0034 | 0.4152 | PASS |
| tiny-bulk-create-1-compact-v2 | 0.0094 | 0.0685 | 0.0122 | 0.0001 | 0.0030 | 0.0931 | PASS |
| tiny-bulk-create-10-compact-v2 | 0.0090 | 0.2482 | 0.0511 | 0.0001 | 0.0035 | 0.3119 | PASS |
| tiny-bulk-create-100-mixed-v3 | 0.0106 | 0.6402 | 0.4133 | 0.0001 | 0.0055 | 1.0697 | PASS |
| tiny-bulk-create-500-mixed-v3 | 0.0070 | 3.0903 | 1.9054 | 0.0004 | 0.0139 | 5.0170 | PASS |
| tiny-bulk-delete-1-compact-v2 | 0.0099 | 0.1296 | 0.0080 | 0.0001 | 0.0026 | 0.1501 | PASS |
| tiny-bulk-delete-10-compact-v2 | 0.0086 | 0.2115 | 0.0104 | 0.0001 | 0.0033 | 0.2338 | PASS |
| tiny-bulk-delete-100-mixed-v3 | 0.0091 | 0.2988 | 0.0147 | 0.0001 | 0.0028 | 0.3253 | PASS |
| tiny-bulk-delete-500-mixed-v3 | 0.0076 | 0.9825 | 0.0506 | 0.0001 | 0.0028 | 1.0436 | PASS |

Low tiers retain compact-v2 definitions; high-tier bulk cases use mixed-v3. Bulk tier100 creates/deletes 1,000 files /100 MiB and tier500 5,000 files /500 MiB, with the separate 200-file /1 MiB witness. Individual-operation cases retain their registered large base fixtures. These are distinct recipes, not a uniform scaling series.

**Strict tier100 assessment:** create 1.069666876 s = TARGET_MISS by 0.069666876 s; delete 0.325349709 s = PASS. The family PASS does not satisfy the separate subsecond pair. The tier100 strict-gate proof pair remains deferred; no favorable repeat was taken.

## Per-case resource observations

Host RSS is the host process peak reported after product work; container memory is the sample-container lifetime peak, including setup/file-cache effects. CPU scopes are host before-to-after-product process CPU and the separate Linux container command window. These are not exact per-phase memory peaks. Spool is separately accounted physical host backing; retained backing returns to zero after Commit in these samples.

| Case | Host CPU s | Container CPU s | Host peak RSS MiB | Container lifetime peak MiB | Physical spool peak MiB |
|---|---:|---:|---:|---:|---:|
| tiny-create-1-compact-v2 | 0.0275 | 0.0206 | 9.53 | 5.65 | 0.00 |
| tiny-create-10-compact-v2 | 0.0329 | 0.0253 | 10.42 | 5.89 | 0.02 |
| tiny-create-100 | 0.0880 | 0.0458 | 53.25 | 5.41 | 0.16 |
| tiny-create-500 | 0.1582 | 0.1574 | 71.44 | 6.86 | 0.79 |
| tiny-stat-1-compact-v2 | 0.0267 | 0.0203 | 8.88 | 5.90 | 0.00 |
| tiny-stat-10-compact-v2 | 0.0290 | 0.0223 | 11.34 | 5.41 | 0.00 |
| tiny-stat-100 | 0.0535 | 0.0300 | 31.19 | 5.38 | 0.00 |
| tiny-stat-500 | 0.1478 | 0.0703 | 40.02 | 4.92 | 0.00 |
| tiny-unlink-1-compact-v2 | 0.0280 | 0.0207 | 9.41 | 4.93 | 0.00 |
| tiny-unlink-10-compact-v2 | 0.0337 | 0.0233 | 12.44 | 7.65 | 0.00 |
| tiny-unlink-100 | 0.1078 | 0.0400 | 52.69 | 5.16 | 0.00 |
| tiny-unlink-500 | 0.2387 | 0.0991 | 62.39 | 5.65 | 0.00 |
| tiny-bulk-create-1-compact-v2 | 0.0369 | 0.0591 | 14.30 | 6.80 | 1.00 |
| tiny-bulk-create-10-compact-v2 | 0.0905 | 0.1581 | 35.38 | 8.75 | 10.01 |
| tiny-bulk-create-100-mixed-v3 | 0.5004 | 0.3462 | 74.59 | 16.15 | 100.15 |
| tiny-bulk-create-500-mixed-v3 | 2.3839 | 1.5771 | 95.22 | 22.91 | 500.23 |
| tiny-bulk-delete-1-compact-v2 | 0.0710 | 0.0609 | 10.52 | 5.39 | 0.00 |
| tiny-bulk-delete-10-compact-v2 | 0.0896 | 0.0954 | 23.03 | 5.43 | 0.00 |
| tiny-bulk-delete-100-mixed-v3 | 0.1136 | 0.1374 | 37.77 | 5.14 | 0.00 |
| tiny-bulk-delete-500-mixed-v3 | 0.3174 | 0.4365 | 53.61 | 6.40 | 0.00 |

All sample containers were validated at 2 CPUs /2 GiB /no swap /256 PIDs, with no host data mounts. macOS owns SDK/coordinator, physical spool and SQLite; Linux owns daemon/FUSE/workload and live operation state. Host CPU is uncapped. All master-isolation and cleanup checks pass; exact campaign-owned container and host-sample inventories are empty. Protected preparations remain available.

## Selected independent proofs

| Case | Result | Proof wall s |
|---|---|---:|
| tiny-create-1-compact-v2 | PASS | 1.5870 |
| tiny-stat-1-compact-v2 | PASS | 1.6090 |
| tiny-unlink-1-compact-v2 | PASS | 1.6561 |
| tiny-bulk-create-1-compact-v2 | PASS | 1.8890 |
| tiny-bulk-delete-1-compact-v2 | PASS | 1.7230 |
| tiny-bulk-create-500-mixed-v3 | PASS | 6.8302 |
| tiny-bulk-delete-500-mixed-v3 | PASS | 5.1849 |

Total proof wall 20.4793 s; maximum 6.8302 s. Each proof retains its 45-second work /59-second hard deadline; separate preparation precedes the proof. All seven bind the corresponding performance source/product/input/image identities.

Coverage: individual create/stat/unlink, compact bulk create/delete, and both mixed-v3 tier500 bulk cases. The tier500 create proof samples beginning/middle/end 64 KiB ranges of every large file, selected small/medium paths and witness; delete verifies selected target absence and witness retention. Exact paths/ranges are retained in the accompanying JSON. Omissions include unselected paths/bytes, exhaustive namespace/object census, aliases, failure injection, physical 100-workspace qualification and other platforms. Existing focused mapping/SDK/Commit tests remain separate evidence; these seven proofs do not replace the entire redesign checklist.

## Provenance and retained evidence

- Implementation commit: `e6d66d676b398e9ba941caba3cdcd34e74e8e608`; official unmodified fuser 0.18.0.
- Source seal: `af3a4790d8d11007f1df8bee2e0f29b1747e75044a3b40eaace02b2d0f9366e6`.
- Product seal: `22e4ad3e355eabef447b2aabf568990e114a95a33d76707d5a876c98daebf60d`.
- Workload SHA-256: `2ec809798656deef9ac05903c9e3bd77921f6ef9c6c87f24060812a81b93bf76`.
- Image: `layerfs-bench-infra:af3a4790d8d11007` / `sha256:5054ec704fec09e95c1db8b898484f665ad9ab8444df51cc541400e9a99e6c03`.
- Host binary SHA-256: `2e4d2ba7894ead636b4e8e258b8d5448eaa4a02c59fdbd64ea3daa51ceae7f7f`.
- The sealed build identity retains its original dirty-source flag; source/product content seals, not that flag alone, bind the tested implementation. Later commits contain reports only.
- Raw campaign: `benchmark-results/host-store/campaigns/issue49-tiny-final-1788695216299617000`.
- Reused current-source create100 receipt: `benchmark-results/host-store/results/run-48e8fff4646c/perf.jsonl`.
- [Machine-readable per-case timings, resources, input identities, raw SHA-256 hashes and sampled proof paths](issue49-tiny-final-results.json).
- Raw receipts, commands, preparation logs and assessments remain append-only in the local campaign directory. The JSON contains exact raw-file identities; raw receipts are not rewritten into a new source or pooled with historical products.

# Hybrid transition experiment results

Exploratory observations on unchanged product code; no optimality or release claim.

| Case | Samples | Commits/sample | Median edit ms | Median Commit ms | Median edit+Commit ms | Edit+Commit range ms |
|---|---:|---:|---:|---:|---:|---:|
| edit-64k | 3 | 1 | 2.231 | 6.263 | 8.424 | 8.339–8.589 |
| edit-127k | 3 | 1 | 2.111 | 6.074 | 8.312 | 8.185–8.593 |
| edit-128k | 3 | 1 | 1.740 | 5.637 | 7.223 | 6.699–7.716 |
| edit-129k | 3 | 1 | 1.733 | 5.371 | 7.067 | 6.145–7.522 |
| edit-16m | 3 | 1 | 1.816 | 6.316 | 8.216 | 7.247–8.360 |
| oscillate | 3 | 6 | 11.851 | 28.635 | 40.486 | 39.532–41.510 |
| batched-crossings | 3 | 1 | 6.439 | 6.048 | 12.639 | 11.506–13.198 |
| shrink-1m | 3 | 1 | 1.736 | 6.311 | 8.016 | 7.676–8.453 |
| shrink-16m | 3 | 1 | 2.171 | 6.129 | 8.300 | 7.889–8.871 |
| chain-64k | 3 | 10 | 20.333 | 46.345 | 66.678 | 61.883–67.488 |

Multi-Commit rows are sums per sample, not per-Commit latency.

Correctness: 10 independent case replays; 34 saved versions checked byte-for-byte after reopen, including via new FUSE workspaces.

Timing scope: public SDK edit and Commit acknowledgements. Setup, FUSE verification, representation inspection, reopen and cleanup are excluded. Raw receipts include complete command/lifecycle walls and host/container resources. Fresh Store/process/container for each sample; OS cache uncontrolled. Representation inspection between history steps may warm subsequent reads. Host lifetime RSS includes fixture/oracle data.

CDC input is not total hashing or I/O. Commit decode/encoding counters include metadata. Separate correctness replay timings do not enter performance summaries.

Performance command wall total: 40.239 s. Verification command wall total: 15.362 s. Build/setup timings are retained separately.

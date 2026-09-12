# Current edit preparation attribution — issue118

Nonce diagnostics only, one 100-edit/one-Commit sequence for each declared cache condition. Both public operations and cleanup passed; raw sample status is `DIAGNOSTIC` and wrapper summary is `INCOMPLETE`, deliberately ineligible for performance acceptance. These are not matched optimization arms.

Raw receipts: [inactive](/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/issue118/20260912/edit-diagnostic-inactive/perf.jsonl), [active](/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/issue118/20260912/edit-diagnostic-active/perf.jsonl); [machine-readable attribution](/Users/yifanxu/Ephemeral-AI-Lab/layerfs/benchmark-results/host-store/issue118/20260912/edit-attribution.json).

Both use host SHA256 `fc9192f13c125c0663cd9e8bb53ae2e7361c7af61fccb11a7ffece9afdd89764`, image `sha256:32fa0eca8fbd8d7790411be6f86943b50f27ee82a03675f82851c54a9ee0328b`, product seal `f8db9e4ea64708ff211332096d4bb31b5d2a20b3582728ecd35fd39eecaabe25`. Full source/input/compiler identities and raw-file hashes are retained in the JSON.

| Measured scope | Inactive ms | Active-cache ms |
| --- | ---: | ---: |
| Public edits | 765.465 | 192.904 |
| Matching Commit | 181.844 | 188.999 |
| Edits + matching Commit | 947.309 | 381.903 |
| Complete chain | 969.275 | 1217.291 |
| Declared cache preparation | 0.000 | 812.543 |
| Host command (includes namespace bootstrap) | 5763.700 | 5736.680 |
| Sample preparation | 691.850 | 761.467 |
| Complete sample | 7052.142 | 7166.262 |

The active case first reads all 100 selected files through one real FUSE tool execution. Its812.543ms cache preparation is inside the complete chain and explains why the smaller edit timer is not an end-to-end improvement.

| Sequential daemon phase | Inactive ms | Active-cache ms |
| --- | ---: | ---: |
| `freeze_gate_ns` | 0.047 | 0.042 |
| `kernel_flush_ns` | 0.160 | 12.403 |
| `append_ns` | 0.038 | 0.025 |
| `facts_ns` | 97.993 | 88.745 |
| `retire_ns` | 0.072 | 0.029 |
| `lookup_ns` | 560.670 | 0.435 |
| `prepare_ns` | 0.718 | 0.319 |
| `apply_ns` | 0.556 | 0.336 |
| `reconcile_ns` | 8.311 | 12.036 |
| Unattributed daemon remainder | 0.557 | 0.270 |

Named sequential phases explain **99.9167%** and **99.7642%** of daemon control wall. No row has a negative phase remainder. This independently exceeds 90%; an algebraically complete transport remainder is not used to manufacture coverage.

| Disjoint edit interval | Inactive ms | Active-cache ms |
| --- | ---: | ---: |
| `host_api_envelope_ns` | 2.438 | 1.960 |
| `host_backing_dispatch_ns` | 440.054 | 5.434 |
| `host_backing_queue_ns` | 2.978 | 1.875 |
| `daemon_exclusive_ns` | 63.031 | 28.826 |
| `backing_transport_scheduling_ns` | 163.058 | 78.506 |
| `control_transport_scheduling_ns` | 93.906 | 76.303 |

These intervals sum to the public edit timer using W=API wall, G=host control-group wall, D=daemon handler wall, B=backing wait, H=host dispatch, Q=host queue. Transport/scheduling terms are bracket differences, not pure network latency. Backing wait is nested and is never added again. It starts before a shared stream mutex and can overlap during concurrent callbacks; both current runs have nonnegative D-B, B-H-Q and G-D on all 100 rows. Do not extrapolate that property to arbitrary concurrent workloads.

Both runs publish exactly 0,1,…,99 fact nodes: **4950 total**, with **1,340,395 request-wire bytes**. At this case size, every nonempty prefix fits one facts page: edit index i (zero-based) emits34+270i+5[i>0] bytes including request frame headers and excluding replies. Active backing calls are 299, exactly the facts BEGIN/pages/END traffic; inactive has 464 calls, adding 165 backing calls consistent with the lookup-only remaining route. Opcode-specific counts were not collected, so that split is a code-path inference.

Inactive lookup dominates: **560.670ms,73.25%** of public edit wall. Fact publication is still material at**97.993ms,12.80%** inactive and**88.745ms,46.00%** active. The active case scans 100 cached nodes per edit (10,000 total), executes 100 kernel flushes and 100 cache reconciliations; the inactive case scans none and has no kernel flush, while retaining 100 notifier reconciliations.

Proceed with the small EDIT_BEGIN-only snapshot omission because redundant growing-prefix publication is measured. Keep the operation gate, kernel flush, append acknowledgement, range retirement and fullFREEZE/fsync/preview/Commit publication. Final Commit already snapshots all 100 edits; this removes repeated work instead of relocating the required final snapshot. Independently investigate dominant inactive host lookup/read reuse. **Fact removal alone does not close edit preparation.**

Required follow-up: exact-source nonce-free alternating pairs for edits, matching Commit and complete chain; fresh fullFREEZE/preview/fsync visibility; local/socket failure/retry and spool-retention proofs; active mmap/FUSE ordering checks. No external library is modified.

Facts-only implementation qualification: two diagnostic/failure-retry checks PASS (6.201s command), two local/socket backing proofs PASS (4.447s), private-preview/root-parity PASS (0.134s), and stale-folio SDK-range/truncation proof PASS (0.128s). The first socket proof FAIL (7.339s) asserted positive backing traffic for the removed pre-edit snapshot; its replacement asserts zero cached-edit traffic and positive fullFREEZE publication, with latest edited facts present. Raw commands/exits/walls, including the failure, are in `edit-facts-*.json` beside the attribution JSON. The Workspace compile reports an unrelated Stage3 `private_interfaces` warning; its owner was notified. Actual mounted mmap/FUSE and nonce-free combined performance qualification remain pending with the root runner.

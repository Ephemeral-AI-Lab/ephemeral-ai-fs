# Issue 95 Init comparison reuse

Follow-up branch starts at unmerged PR #94 head `cf3a058925c3012fda0fae922dc081766bb8fa99`. Control product: `861e388339ef5572659cb16ef0c8febbff0351df`; host SHA-256 `c4be654eb87cf76a16536e832a0767e620f1e1647c547e17b9a3d3c57db3984f`; image `sha256:8e74a58284a6e8c22ac9cbf60231f69217c3cae31c231ef17b0afd81726b8249`. Evidence-only changes between product and PR head preserve product/harness applicability. Historical evidence is unchanged.

## Diagnosis

[r26-diagnostics.csv](r26-diagnostics.csv) extracts all 30 cases with exact raw paths and SHA-256. Durations are nanoseconds; RSS is bytes. CPU is after-minus-before host-process CPU; sampled RSS is the existing broader sampling window, and lifetime peak is not incremental. Concurrent producer blocking is not additive wall time. Canonical comparison bytes are not physical disk reads. These are individual historical observations, not latency distributions.

Short sealed-control profiles live in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95-runs`. `p01-identical-control` sampled the waiting launcher; `p02-identical-control` did not match the actual worker command. Both are preserved as unsuccessful profile attempts. `p03-identical-control` sampled the authentic workspace-run child: a main-thread comparison branch has 529 samples, including 357 in native canonical identity authentication. This supports eliminating repeated reconstruction/authentication across incoming pages while retaining actual byte comparisons. Profiled timings include profiler overhead and are not paired performance claims.

`p04-unique-control` has 708 main-thread samples, 628 in Init. Commit has 293, including 267 guarded pwrite samples; insertion branch has 81; native compression has 160. Counts are nested and must not be added indiscriminately. Unique publication is chiefly page-write syscall work and encoding, not repeated-object validation. Existing 128-row locator statements, bounded pack INSERTs and SQL coalescing already address batching. R24's larger locator statements were rejected. No unique-path implementation is justified by this profile within the frozen limits.

## Selected experiment and prospective timing protocol

Only initialization admission retains already authenticated comparison operands. Reuse requires an unchanged full physical Location and exact canonical length/bytes on every occurrence. Misses keep ordinary authenticated packed reads. SQL lookup, dependency validation, session/publication epochs and publication are unchanged. The 2 MiB reuse reservation (payload capacity plus conservative 2048-byte per-entry B-tree charge) comes from the existing 16 MiB uniqueness/filter allowance; oversize operands bypass retention, saturation drops all retained entries. No resource limits increase. Non-initialization admission follows the original comparison path.

Before candidate collection: one adjacent control/candidate pair for each of identical-500, CDC overwrite-500, and unique-500, seed 1, unchanged harness/workload/fixtures/timers and 2 CPU/2048 MiB Docker limits. Control first, then candidate; one full sample per arm, normal OS caches, fresh output. Existing runner 300/310/600-second product/command/setup allowances; selected diagnostic only, admission_eligible=false. Preserve every attempt and report absolute elapsed, CPU, lifetime/sampled RSS, storage, and identities. Add a reversed pair only if noise/order effects require it. A measurable gain plus correctness is required to retain the change. No historical single observation is a paired control.

Terminal qualification remains pending: affected families, Store/Workspace/native tests, fmt/Clippy, full benchmark and full157 equal-retained-state storage proof. Historical Git INELIGIBLE/INCOMPLETE limitations and original severity labels remain unchanged.

Pre-measurement review rejected the first 512-byte entry charge because a singleton B-tree allocates a full node. Candidate c1 was built but never timed; its binary remains in `c1-host`. The retained 2048-byte charge covers a full node even with one live entry.

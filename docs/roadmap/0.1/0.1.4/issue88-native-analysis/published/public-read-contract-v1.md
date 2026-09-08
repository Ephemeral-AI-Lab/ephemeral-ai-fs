# Issue88 P public read-cost probe — source-only proposed supplement

Status: plan only; no implementation, build, Store open, copy, mount, or read probe was performed for this note. Freeze this supplement before collecting the observations. It is a bounded development diagnostic of the two already approved frequent-edits smoke members, admission_eligible=false, not a new benchmark family or a fixture/normal-mode change.

## Decision and precise public surface

Use a small standalone host-side analysis binary under the issue88 documentation directory, linked to the exact arm's existing LayerStackStore and SDK crates, plus a Python wrapper importing the existing shared runtime. Do not add a mode to fs-benchmark-pro, edit the workload helper, register a family, or build a new runtime image for this probe. This is necessary because the current storage-smoke control protocol only offers whole-tree `storage-smoke-observe` for historical verification, and `storage-smoke-read` only for the small-files performance case; it has no fixed-file range command.

The SDK's public `Client::exec_workspace_session` and terminal output drain expose the required real FUSE read route. There is no direct public SDK file-range read method in current client.rs. Measure an explicitly named **public Exec read-and-count** operation, not raw read(2) latency or pure decoder latency. Shell launch, dd, wc, pipe transfer, public execution acknowledgement and output drain belong inside this boundary. Match all of them between arms. A separate digest invocation verifies content outside that timer.

Source seams to reuse rather than invent storage machinery:

- benchmark/fs-bench-pro/src/main.rs:4567 `benchmark_container_binding`: ContainerManager::open(sample_parent/container-control), connect(container_id), binding(). These helpers are private to the benchmark binary; the standalone host can call the same public SDK methods in a few lines, without copying a new container manager.
- main.rs:4579 `benchmark_client`: Client::connect_with_container(Arc<LayerStackStore>, binding).
- main.rs:4636 `execute`: exec_workspace_session, workspace_output, blocking read until terminal output; reject truncation, nonzero exit, non-daemon transport, absent daemon timing, Docker Engine calls or unbalanced execution receipt. Reproduce that short public-call sequence and mandatory checks, not its entire surrounding benchmark.
- src/storage_smoke.rs:356 request selects WorkspacePlacement::Container at /workspace/storage-smoke with explicit WorkspaceProjection::Fuse.
- src/storage_smoke.rs:703–735 historical fork/mount/observe/end lifecycle; use LocalForkSource::Branch with the precise stored CommitId from the source receipt, not HEAD.
- shared/runtime.py:340 start_sample owns authenticated daemon/container setup and fixed2CPU/2GiB/no-swap/256PID resource profile. Its ordinary no-data-sharing-mount topology remains mandatory.
- shared/runtime.py:521 closed_store_copy makes an independent fsynced byte copy, rejects original sidecars/symlinks and preexisting destination, checks source and copy SHA-256. Use it once per arm/case read campaign, after original producer and verifier cleanup/seals PASS.
- src/storage_smoke.rs:240 timed defines process CPU/current/lifetime RSS/I/O and before/after physical receipt separation. Reuse process snapshot API and report scope accurately.

## Fixed minimal population

Only sdk-text-32k and sdk-binary-8m from the paired approved frequent-edits smoke runs. File relative path is literal `file`. Only retained checkpoint index3. Existing synthetic_inputs at shared/storage_smoke.py:166–178 constructs three4096B updates at quarters, followed by steps4/5 returning to initial A; checkpoint3 exercises retained changed content while avoiding final A recurrence. This is a source-based choice made before read observations, not a search for maximal measured depth. It does not guarantee a native PREFIX or maximum-depth target. Record actual read work/depth; do not move the range if native counts are zero.

For each case, freeze exactly two operations:

|Case|File bytes|Operation|Offset bytes|Requested bytes|
|---|---:|---|---:|---:|
|sdk-text-32k|32768|small range crossing third edit boundary|22528|4096|
|sdk-text-32k|32768|full file|0|32768|
|sdk-binary-8m|8388608|small range crossing third edit boundary|6289408|4096|
|sdk-binary-8m|8388608|full file|0|8388608|

Range offset = 3*file_length/4 -2048. Full request is exact known length. Three repetitions each, no warm-up target read:12rows per arm,24 total. Every row uses a fresh host process/Client/Store connection, fresh container/daemon, fresh fork and FUSE Workspace; no warmed application context shared between rows. This means fresh application/FUSE context with uncontrolled OS caches, **not cold OS/disk/cache**. Copy/hash, previous history, build and ordinary OS cache effects remain disclosed. One sealed disposable Store copy per arm/case may accumulate probe-only Branch metadata between rows; each timed context is newly opened and no target content is read during setup. The original Store remains unopened. Do not make another full Store copy after every read or Commit.

Fixed order: case text then binary, operation range then full; within each pair repetition order C+S1,P / P,C+S1 / C+S1,P. The exact source Store pair must have passed performance, logical identity and cleanup; missing P counterpart blocks that pair. Never compare P's Store through the old control reader. Each arm's matching reader/image pair reads its own produced Store, with the same probe source and commands. Keep raw timings, count3, median and min/max; do not market six small reads as a general read-latency claim.

## Command, timing, correctness

Use the exact existing image's existing /bin/bash, /usr/bin/dd, /usr/bin/wc, /usr/bin/sha256sum; verify paths/features and hash binaries outside measured time before the campaign. No install or runtime image adjustment; missing utility is a concrete preflight blocker requiring a new frozen supplement, not silent substitution.

Timed argv skeleton, passed via the public SDK as literal arguments:

`/bin/bash -o pipefail -c '/usr/bin/dd if=/workspace/storage-smoke/file bs=65536 iflag=skip_bytes,count_bytes,fullblock skip=OFFSET count=LENGTH status=none | /usr/bin/wc -c'`

OFFSET/LENGTH are the fixed nonnegative integers above, prepared before timing. GNU dd counts/skips bytes independent of buffer size; wc consumes all bytes and outputs only an integer. Public route completion requires exit0 and integer exactly LENGTH. Pipefail ensures a dd failure cannot be hidden by wc. No raw file bytes enter logs. The sink/count work and shell/pipeline topology are deliberately part of Exec read-and-count, not subtracted using another machine's measurement.

Start monotonic timer immediately before exec_workspace_session; end immediately after last blocking output read reporting terminal exit, before output parsing/digest/oracle work. Record public exec count1, workspace_output reader count1, actual output read/poll count, process count1shell+1dd+1wc, daemon receipt fields, FUSE read count/bytes, forbidden FUSE writes0, mount identity, physical receipt delta (legacy reads plus native request/parser/raw/decode/dependency/depth histogram), and exact read-and-count wall ns. Native frame authentication remains intrinsic and included.

After timing, on the same disposable session, run a separate verification Exec with the same dd range piped to sha256sum. Hashing/second Exec/output verification is a separate phase and cannot enter read timing or its physical delta. Before any product read, derive expected SHA-256 from the already sealed `state-3` fixture for both range and full file; cross-check the full result against the existing oracle-3.json `file` digest/size and record manifest identities. The fixture/oracle is never passed as a product-side shortcut. Verify returned bytes count and digest; any mismatch invalidates the pair and blocks promotion.

Expose first demanded read only; do not pool post-digest warm reads with it. The digest phase will warm caches but that session/container is then ended/removed before the next row.

## Custody, resources, cleanup and stop conditions

Before copying, require source performance/verification statusPASS, cleanupPASS and retained manifest equality. Original allocation remains the smoke's acknowledgement allocation. Record source/copy logical hash and byte length, source branch/CommitId, checkpoint index, fixture/oracle hashes, host/image/product/probe/runner/utility identities, original sidecar absence and original seal before/after campaign. Copies are writable only for public fork/runtime metadata; do not infer allocated Store or savings from copy stat/clone allocation. No Commit is required or performed during read probes.

Phase rows distinguish source validation+copy, context start/bind/fork/mount, timed public read-and-count, output validation, digest verification, end/unmount/client/store close, container removal and enclosing invocation wall. Source Store disk, disposable copy disk, host runtime/spool and container staging remain separate. Copy allocation_comparison_eligible=false.

Use existing measurement lock and no simultaneous builds/census/other runs. Before/after host CPU and read/write I/O include exact read action; RSS is current/lifetime as actually available, not fabricated incremental peak. Cgroup snapshots around the operation remain broader observation windows; report anon/file/cache/kernel/total separately, no swap/OOM, and record sampling time separately. Enforce inherited2CPU/2GiB/no-swap/256PID container and8GiB host ceiling. Freeze120s setup,30s read-and-count,30s digest and30s normal cleanup per row; these are bounded diagnostic stops, not relaxed existing smoke acceptance gates. If inheriting stricter existing limits, use those. Retain timeout, negative and incomplete outcomes; no repeated valid sample for a nicer number.

End each successful read-only Workspace with EndWorkspaceMode::Clean; verify zero active Workspaces/executions, expected read-only state and no dirty/committed result; discard only as failure cleanup with the reason preserved. Remove only owned container and disposable runtime artifacts according to shared runtime. Source seals remain unchanged. Unknown identity, missing binding, non-FUSE route, malformed native dependency, resource overrun, mismatched bytes, timeout or incomplete cleanup stops the probe; do not proceed to full157 promotion based on partial favorable rows.

## Limits of this result

This supplement answers the public cost of these two fixed small-range/full-file reads under fresh application contexts. It does not prove OS-cold cost, arbitrary random-read throughput, mmap behavior, deep-history tail latency, maximum depth4 product selection, or general file-size scaling. Compile-time reader tests already cover valid depth4/corrupt depth5; measured smoke reads report whichever depths the actual frozen workload produced. If process/IPC dominates the small range, report the absolute Exec latency and physical read amplification without asserting a pure decoder speed ratio. This is a useful cost check for the approved optimization path, not another open-ended diagnostic loop.

## Pre-observation API scope correction — frozen before probe builds/samples

Independent source review established that public FUSE read receipts are collected
only at Workspace end (layerfs-workspace/src/lifecycle.rs970), and write receipts
only during Commit. Monitor snapshots during Exec cannot supply per-action kernel
read/write counts. The earlier per-phase counter/zero-write requirements above are
superseded by this precise available API scope; no probe observations existed.

Per-action kernel FUSE fields are null with a reason, never fabricated zero. After
successful Clean end, require a present public WorkspaceEnd read receipt and report
its count/bytes across the whole setup+read+digest+end session. Kernel write counts
remain null because no Commit is performed. Source-fixed read-only commands, byte
count/digest correctness, no Commit, successful Clean end and unchanged fork head
provide the functional nonmutation proof. Clean end pauses/quiesces and rejects
unpublished state or projection dirtiness at lifecycle.rs922–954. This is not a
claim that unavailable write-request metrics measured zero. Per-action Store
physical counters remain actual sampled interval deltas; asynchronous read-ahead
can cross boundaries, so they are not exclusive kernel attribution.

Require the actual mountinfo filesystem type fuse/fuse.* using Bash builtins, not
only a matching mount path. Require equality of utility hashes across both arms
and every row, and enforce observed lifetime host RSS peak as well as current
RSS≤8GiB. Attempt owned-container removal independently of staging/log/host
observation errors; any incomplete cleanup remains a failed row. All fixed input
selection,24-row order, timer/limits, oracle, source/copy and no-migration rules
remain. This correction adds no product API, Commit, fixture or benchmark mode.

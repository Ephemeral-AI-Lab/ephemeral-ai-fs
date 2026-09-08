# Issue88 frozen public read probe

Separate analysis tool for the committed public-read-contract-v1.md. It neither builds nor constructs benchmark history. It requires the exact completed, verified frequent-edits source runs for control C+S1 and candidate C+S1+P. Original Stores are opened only as ordinary read-only files for identity/copying, never via SQLite/SDK. Four disposable independent copies receive probe-only public fork metadata. Copy allocation is explicitly ineligible for storage comparison.

Source-only implementation is not evidence of a passed build or sample. The owner must build, seal, review and execute in the shared serialized slot. Do not run this wrapper concurrently with normal builds, smoke construction, verification, census or another probe. Runtime acquires the same `layerfs-infra-measurement.lock` as normal smokes; do not wrap it with a second owner holding that same lock.

## Build identity

Build this standalone manifest once against each arm's own crate sources. Copy this complete directory byte-for-byte into the control worktree; do not substitute the P reader against a control image. Keep Cargo.lock byte-identical and use locked builds after the first lockfile is generated. No new product/workspace dependency or benchmark module is added. serde_json is analysis output formatting only.

The serialized build owner writes `PROBE_BINARY.identity.json` containing:

- `binary_sha256`: actual executable SHA-256.
- `LAYERFS_SOURCE_SEAL` and `LAYERFS_PRODUCT_SEAL`: exactly the matching smoke producer's source/product seals, recomputed from the build checkout with the existing runner.
- `tool_files`: relative file names to SHA-256 for every regular file in this read-probe directory, excluding target/ and __pycache__/. Include Cargo.lock after generation. This is the mapping returned by run.py's `tool_seal` function.
- Additional commit/tree/dirty patch, command, compiler/library, resource log and normal host/image build identities should be retained by the enclosing source custody owner. A report commit is not the measured binary revision.

`resources.rs` preserves the existing benchmark's private macOS native process-resource helper. It is copied because the benchmark executable exposes no library API for that helper. The probe is macOS-only; the coordinator and SQLite must not execute in Docker. Other lifecycle work calls the existing public SDK, and Python imports the existing runtime for container/copy ownership.

## Invocation schema

An arm configuration JSON contains exactly `control` and `candidate`. Each has `source_run` (completed frequent-edits run root), `probe_binary` (absolute executable path with identity sidecar), and `image` (matching image ID/tag). Example with explicit placeholders:

```json
{
  "control": {"source_run": "/ABS/VERIFIED-CS1-RUN", "probe_binary": "/ABS/CONTROL/issue88-public-read-probe", "image": "sha256:CONTROL_IMAGE"},
  "candidate": {"source_run": "/ABS/VERIFIED-P-RUN", "probe_binary": "/ABS/NATIVE/issue88-public-read-probe", "image": "sha256:CANDIDATE_IMAGE"}
}
```

After serialized permission and frozen exact binary identities, invoke the same script source:

```text
python3 /ABS/read-probe/run.py --arms /ABS/arms.json --output /ABS/NEW-READ-RUN
```

The output must not exist and must lie outside both originals. No subset, repetitions override, offset switch, changed checkpoint or retry mode is provided. It executes the fixed24 rows from the contract or preserves an incomplete campaign and stops. Original source manifests, fixture/oracle3 identities, checkpoint mappings, host probe/image identities and both source seals are checked. GNU utility identities/features are checked on each new container outside timing; no utility installs or image changes are performed.

Each row owns a fresh process/container/daemon/FUSE context. The stdin protocol lets the wrapper observe cgroup resources before and after each action without a Docker operation inside the measured public Exec boundary. The host timer covers exec_workspace_session through terminal output drain. Scalar output validation and receipt checks follow timing. Digest verification has its own action, receipts and resources. Public mount proof is setup-only and does not read the target file.

## Evidence interpretation

Every raw host row identifies the physical counters as per-operation deltas, durations as integer ns, count/byte fields as integers, CPU/RSS/I/O scopes by field name, and depth histograms as completed-chain counts. Cgroup values are boundary snapshots and lifetime peak fields, not exact phase maxima; observer_ns is separate. Original acknowledgement allocation is never replaced by copy allocation. `copy_disk` is diagnostic storage occupation only. Native parser counters describe requested directories/records, not whole-pack validation or physical disk-page I/O.

Outputs preserve command/config/tool/source identities, four copy receipts, pending row records, host JSONL/stderr, per-row results,24-row completeness, unchanged-original seals and an output SHA-256 manifest. Failed rows are not overwritten or automatically retried. Full/range output carries only integer counts or digests, never file bytes. No Commit occurs. Cleanup requires successful public session end, zero active Workspace/execution counts, unchanged fork head and removed owned container. Error cleanup uses Discard; incomplete cleanup cannot pass.

Aggregate exactly the frozen three repetitions by case/operation/arm; report raw integer ns, median and min/max and sample count. The wrapper intentionally preserves raw observations rather than claiming product speedup. A higher-level report must not label Exec read-and-count latency as read(2)/decoder-only or OS-cold latency, and must retain negative outcomes.


## Reviewed FUSE observation boundary

Per-action FUSE read/write counters are **null**, with an explicit unavailable reason. The public implementation publishes WorkspaceRead metrics only during Workspace end, and write metrics only during Commit. This probe does not add a Commit or a product instrumentation endpoint. After successful Clean end it requires an actual WorkspaceRead receipt and records its counts/bytes with the whole-session scope: setup mount proof + measured read + digest verification + end. It cannot divide those counts into read versus digest phases. Write counters remain null. Fixed read-only commands, successful Clean end, unchanged fork head and no Commit calls establish read-only correctness; they are not relabeled as a measured zero kernel-write counter.

Mount readiness parses `/proc/self/mountinfo` with sealed Bash builtins, requires exactly one matching mount path and a `fuse` or `fuse.*` filesystem type, and does not use an unsealed grep executable. Utility binary hashes must match all rows and both arms. Both current and lifetime peak host RSS must stay within8GiB. Container removal is attempted independently even when host cleanup, staging observation or log collection fails; those failures remain explicit and cannot yield PASS.

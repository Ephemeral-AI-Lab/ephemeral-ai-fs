# Independent public read-probe source review

Source-only review of the standalone read-probe Rust host, Python wrapper, README and frozen public-read contract in the native candidate checkout. No build, run, Store opening, copy or probe was performed by this reviewer.

## Blocking measurement finding

The initial `fuse()` implementation sums storage receipts from `Client::monitor_snapshot` before and after each Exec. That API does not collect live FUSE transport counters. The product calls `projection::record_read_metrics` from workspace end (`lifecycle.rs`), while write-metric collection occurs during Commit. `exec_workspace_session` and `workspace_output` record their SDK operation receipts but do not invoke either collection boundary. Consequently the initial phase FUSE fields would be zero because no relevant receipt exists, not because measured read/write traffic is zero. This makes the read/write claims invalid despite otherwise correct elapsed timing.

Use an existing supported live collector if one exists, or freeze an explicit amendment before observations: per-phase FUSE metrics are null with a reason, and the after-Clean-end read receipt is a whole-session population including setup, timed read and digest. Do not add a Commit or product instrumentation merely to manufacture per-phase receipts. Zero forbidden writes needs an actual available counter or a precisely identified source-supported read-only invariant; absent FuseWrite receipts cannot prove zero. The clean/unchanged-head checks remain useful separate correctness checks.

## Blocking cleanup finding

The initial wrapper executes staging observation, Docker log retrieval and `sample.remove` in one try block. An observation timeout or error skips container removal. Removal of the owned container must be attempted in an independent finally/cleanup path, preserving the observation failure and failed row. Cleanup cannot be made conditional on successful diagnostic collection.

## Resource and comparability findings

The initial host gate checks only end-of-phase current RSS despite recording lifetime peak RSS. Because each row has a fresh process, an observed lifetime peak above the eight-GiB ceiling must also fail; otherwise a transient measured breach can pass. Cgroup before/after snapshots are broader observation windows and must stay labeled that way. They are not phase maxima.

Installed utility hashes are recorded each row but initially are not compared across arms. Assert equal Bash/dd/wc/sha256sum identities across the matched rows to substantiate identical command implementation; merely recording four lines is not that comparison. No utility replacement or image adjustment is authorized by a failed preflight.

## Source-supported properties

The schedule is exactly 24 rows: two cases, range then full, three repetitions, both arms in control/candidate, candidate/control, control/candidate order. Each row creates a new host process, Client, container/daemon, fork and FUSE workspace; one independent copy is reused for each arm/case. Checkpoint3 and literal file path/ranges match the frozen contract. No adaptation toward a deeper measured prefix is provided.

The timed command uses GNU dd byte skip/count flags, fullblock, explicit count/offset, and Bash pipefail with wc consuming all requested bytes. Its timer begins before public execution and ends after terminal output draining. Output parsing, SHA-256 verification and wrapper resource observations occur outside that timer. The digest is a second real public Exec through FUSE; fixture bytes are used only to compute the expected host-side digest, not as product-side read output. Native decoding/authentication remains included in the demanded read path. Shell/pipeline/output costs are part of this named Exec read-and-count measurement, not pure read or codec latency.

Source performance/verification manifests, fixtures/oracle3, mapped CommitId, producer/probe source/product seals, binary hash and image labels are checked. Copies use the shared closed-store copy helper; original Store is never opened via SDK. Path/sidecar/quiescence and external build seals remain custody preconditions. The probe source file map and pinned arm reader must be sealed after source fixes and lockfile generation. No original allocation comparison comes from writable probe-copy stat.

The public end path uses Clean on success, verifies zero active workspaces/executions and an unchanged fork head, and does not call Commit. Failure uses Discard where reachable; wrapper timeouts retain incomplete results and terminate only the owned host/container. Any incomplete cleanup or source-seal change prevents campaign PASS.

Disposition: do not execute the probe until measurement scope and unconditional container cleanup are corrected, the peak/utility checks are reconciled, and any required scope amendment is frozen. This source review does not claim compiled API compatibility or an empirical read-cost result.

## Final pre-observation source reconciliation

Status: **no unresolved blocking finding from this bounded source review; proceed to serialized build/preflight gates**. No probe observation was used to choose these corrections.

The amended contract explicitly supersedes unavailable per-action FUSE counters. Current Rust output uses nulls with a reason for every phase FUSE read/write field, obtains the actual WorkspaceEnd read receipt after cleanup, labels its whole-session scope and requires receipt presence. Kernel write counters remain unavailable. Successful Clean end, unchanged fork head, no Commit, fixed read-only commands and count/digest checks are identified as functional nonmutation evidence rather than measured zero write requests. Store physical counters remain sampled interval deltas with possible asynchronous read-ahead crossing; they are not exclusive per-phase kernel attribution.

The source now verifies a unique matching mount path with actual fuse/fuse.* filesystem type using Bash builtins. Both current and lifetime host RSS must remain within eight GiB. Utility hashes are compared against one campaign identity list across all rows and arms. Owned-container removal is attempted from a separate finally block even when staging/log/host cleanup observation fails, and those failures preserve FAIL status.

The fixed 24-row order, expected fixture/oracle digest, original/copy identity rules, public Exec-to-terminal timer and separate digest action remain intact. The parent plans identical analysis source/lockfiles and pinned common product dependency versions in both arm builds; actual binary/dependency identity remains a build custody gate, not something this source-only review can certify. This review permits progression to that gate and does not assert compiled compatibility, a successful read campaign, or full157 promotion.

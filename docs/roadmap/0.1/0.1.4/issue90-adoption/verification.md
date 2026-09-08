# Phase-1 verification and source custody

**All local required correctness and quality gates passed.** The independent
review found no unresolved adoption blocker. The verified product candidate is
[`593f4ad018bf34b3f180baf66e1ae5cf40c36647`](https://github.com/Ephemeral-AI-Lab/layerfs/commit/593f4ad018bf34b3f180baf66e1ae5cf40c36647),
tree `755ad78b2fa090d5afda6c27a1086dd628b3ec91`, clean at final execution.
Subsequent report-only commits do not change the tested product/harness inputs.
This is phase-1 correctness qualification, not phase-2 performance qualification
or a release/merge decision.

## Executed final gates

| Gate | Actual result | Receipt |
|---|---|---|
| Workspace build, Rust1.85.1, locked | PASS | [build](verification/workspace-build-final-01/output.log) |
| Full native workspace, all features, four coordinated jobs | 389 reported PASS /0 FAIL /1 ignored;37 executables;88s warm execution within existing120s ceiling | [workspace](verification/workspace-final-02/output.log) |
| Complete Store library in that run | 88 PASS /0 FAIL /1 ignored; all original nine checks executed and passed | [workspace](verification/workspace-final-02/output.log) |
| Explicit ignored large-spill lane | 1 PASS;100,004,100 actual file bytes; every expected canonical object/identity after fresh reopen | [large spill](verification/large-spill-final-01/output.log) |
| Store integration | v4 suite7 PASS, legacy6 staging suite1 PASS; public Workspace file-edit suite12 PASS | [workspace](verification/workspace-final-02/output.log) |
| Static SQL/schema/manifest | SQLite parsed cardinality, exact manifest/parameters/schema6, exact native7 layout and supported-version tests PASS | [workspace](verification/workspace-final-02/output.log) |
| Rust formatting and warning-denying workspace Clippy, Rust1.96.0 | PASS, no lint waivers | [quality](verification/quality-final-01/output.log) |
| `git diff --check` | PASS | [quality](verification/quality-final-01/output.log) |
| Workspace doctests | One actual compile-fail doctest PASS; other crates contain zero selected doctests, not extra coverage | [doctests](verification/docs-final-01/output.log) |
| Linux daemon/runtime component lane | 5 daemon tests,1 selected proxy test,31 FUSE library tests PASS; host-fuse feature compile PASS | [Linux checks](verification/runtime-check-final-02/output.log) |
| Linux runtime image build and workload self-check | PASS; source-bound release daemon/proxy binaries; existing workload self-check unchanged | [image](verification/runtime-image-final-01/output.log) |
| Enabled managed SDK/Docker/FUSE lifecycle | 3 actual PASS; normal Commit, mmap/edit coherence, live commands, retry/cleanup and clean shutdown | [live](verification/live-final-02/output.log), [empty container inventory](verification/live-final-02/containers-after.txt) |
| Actual older/newer implementation compatibility | PASS on disposable8KiB Directory fixtures: native7 old-open rejection leaves bytes unchanged; legacy6 alternating writes/full readback and reopen | [probe](verification/old-binary-final-01/output.log), [source](verification/helpers/compatibility-probe/src/main.rs) |

The Store library's sole ignored test was executed successfully in its explicit
lane. It is not left untested or silently counted in the normal88. The seven
opt-in live tests inside the native389 count return early without activation;
that reported count is not389 independent live guarantees. The three managed
Docker tests were separately enabled and actually executed.

Four direct-host Linux FUSE tests are not applicable to this macOS coordinator:
their local Linux mount path requires a capable Linux host. They were not
activated or called passed live. The required approved topology here keeps
SQLite/SDK/canonical construction/spool on macOS and executes real Linux
FUSE/daemon/workloads in Docker. The enabled managed lane and Linux component
lane verify that topology; Docker never executes a Store. This is not a claim
that all other host/mount configurations were exercised.

The Linux31 include the >16,384-record streamed checkpoint, duplicate/invalid
records, scratch overflow, pre-install validation, partial-install retry and
staging cleanup introduced by #71. These component checks use transient live
metadata and no SQLite Store. The managed containers use2CPUs,2GiB memory and
256PIDs; the Docker build uses two Cargo jobs. No full benchmark family ran.

## Exact identity

Full fields and executable SHA256 values are in
[identity.json](verification/identity.json). In particular:

- Source seal: `55c0f273b8ffb5612a0c907f4c4692084ef006e740c917bf34d82b07a3c315d7`.
- Product seal: `e279fce6024490e04833038dd834b21e2f5f31077730c9e255477eb648d2804c`.
- Dirty patch: empty; SHA256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- Runtime image: `layerfs-issue90:55c0f273b8ffb561`, immutable image ID
  `sha256:bcdef310f25f5a430f4e6e8c8c312213d5561cd18d0d23955dcfbec5b8029299`.
- Executed Store test binary SHA256:
  `aab68c8b2d832102e8587e5e149b7aa04942b443e669082e7aa2b4242493fc85`.
- Executed live SDK test binary SHA256:
  `4acf89851e8a7165b5c37889ae59b35341b4845a8b07e83ab166c8df01415d6d`.
- Older source in compatibility probe:
  `bfbc46c11dbb4d8dd949b54845e222cf42cd822f`, clean and unchanged. The verifier links
  that actual code with the candidate using its separately sealed Cargo.lock;
  it is not an archived shipping executable or a performance comparison.

Host: Apple M3 Max,14 logical CPUs,38,654,705,664B RAM,macOS26.4.1/arm64.
Native debug build/test toolchain: Rust1.85.1. Quality toolchain: Rust1.96.0.
Linux release runtime: Rust1.85.1, aarch64-unknown-linux-gnu, pinned Bookworm base.
Root Cargo.lock and schema7 hashes are recorded; dependencies were not edited.
Phase2 must build/seal its own performance host binary/profile and comparable
control, rather than calling these debug correctness binaries performance-qualified.

Cargo package versions remain0.1.3 from the integration base. This source-identified
unreleased candidate does not redefine the released0.1.3 artifact or its manual.
Any eventual release tag/version must follow release policy; none is created here.

## Failures, corrections and review

The [resolved defect ledger](defect-ledger.md) distinguishes product defects,
stale fixtures, missed injections, and explicit compatibility/ownership behavior.
All D01–D09 and A01–A09 have passing final proof. No assertion was disabled, limit
raised, failed guarantee waived, or original Store migrated to achieve a pass.
The [independent review](independent-review.md) reconciles the final receipts,
source scope and earlier findings; the coordinator separately reviewed the
reviewer's small content-layer provenance changes.

Failed and partial attempts remain under [verification](verification/README.md),
including the19,571-object before-repair residue, compile corrections, stale
feature/integration fixtures, the interrupted late-race fixture and stack sample,
wrong route-count inference and failed full-workspace injection attempt.
The final complete workspace run followed the causal repair; no passing full
benchmark campaign was repeated or replaced.

Docker disappeared once before runtime build and again before the first live
attempt. The [backend log](verification/live-final-01/docker-quit-evidence.log)
records explicit GUI `/app/quit`. These remain failed infrastructure attempts.
After restoring Docker, Linux component checks and the enabled live lane passed;
no failed row was relabeled. Final live containers were absent after cleanup.
Early development wrapper/dirty-patch custody limitations are explicit in the
[receipt README](verification/README.md); only clean final-source receipts qualify
this candidate. [manifest.json](verification/manifest.json) hashes retained evidence.

Historical #88 storage savings and unfavorable read/CPU/write observations retain
their original producers and qualifications. This work collected no fresh
Store-footprint comparison, general cold-cache/large-file/tail result, or broad
time-performance qualification. See the [phase-2 handoff](phase2-handoff.md).

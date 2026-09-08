# LayerFS 0.1.3 verification

> **Status:** Released verification record for LayerFS 0.1.3.

## Source binding

The accepted benchmark checkpoint is `9f5a641d223606c45e5e6aa8a20094c12f9139a1`.
Release preparation starts from `28177560c8f049c02192e18c263cdc5543c1ab52`.
All tracked implementation and harness files are byte-identical to the checkpoint except `crates/layerfs-fuse/src/live_owner.rs` and `crates/layerfs-fuse/src/live_runtime.rs`, which contain the release-only guard-retirement correction and deterministic regression extensions. Workspace package versions in root `Cargo.toml` and the 12 local lockfile package records advance from 0.1.2 to 0.1.3; third-party dependency records do not change. Manual, release documentation, entry-point links, and the explicit preview migration policy are updated separately.

The initial candidate matched checkpoint product seal
`3c797bc6dbfd9b03b919c270b609cad839000b68d67e34f3b00d24717e07f39a`.
After the correction, the existing runner computes product seal
`0bda13cfd3e8a9900fe3b66f0c41e922581806970bc6d42cfb2bef5f5af46c7d`.
The release is therefore a narrow correctness descendant of the benchmark source, not a byte-identical product. Original benchmark identities and timings are not relabeled. See [machine-readable release evidence](release-evidence.json).

## Checks

| Check | Release preparation result |
|---|---|
| Rust formatting | `cargo +1.96.0 fmt --all --check`: PASS |
| Full workspace native tests | `RUSTUP_TOOLCHAIN=1.85.1 tools/test-fast.sh`: 348 passed, zero failed, one pre-existing ignored; 74 seconds, four bounded jobs (initial candidate: 79 seconds) |
| Workspace build | `cargo +1.85.1 build --workspace --locked`: PASS |
| Warning-denying Clippy | `cargo +1.96.0 clippy --workspace --locked -- -D warnings`: PASS |
| Shared benchmark infrastructure | 40 Python checks: PASS |
| Report regression checks | Two Python checks: PASS |
| Checkpoint report derivation | Rebuilt from existing raw receipts: PASS, no report errors; no performance reruns |
| Managed Docker/FUSE tests | Repaired source: three predeclared full runs, 3/3 tests each; **9/9 executed tests PASS**, in 8.83 / 8.27 / 8.47 seconds |
| Documentation and whitespace | 18 maintained release/entry documents: local links resolve; whitespace check PASS |
| GitHub CI | Required on the final PR head and merged release source before publication |

The native suite includes platform-gated tests that return without live-Docker execution unless enabled. Its native PASS count does not establish FUSE qualification; the three tests below are executed separately with `LAYERFS_LIVE_DOCKER=1` and the source-bound release runtime image:

- `managed_container_lifecycle_and_disconnect_cleanup_are_exact`
- `running_commands_and_dirty_mappings_continue_across_commit`
- `ordinary_writes_queue_during_mapped_commit`

The image is local verification infrastructure, not a published release asset. Build provenance preserves the base commit/tree, dirty candidate status, source seal, product seal, and workload hash. A final identity comparison ensures subsequent documentation changes do not change the tested runtime inputs. The GitHub release announcement records the exact final commit and successful CI run, avoiding a self-referential commit hash inside its own tree.

## Initial release-runtime failure

The explicitly enabled live suite completed in 9.35 seconds with two passes and one failure. `ordinary_writes_queue_during_mapped_commit` observed byte 777 as zero rather than the SDK-written `S` in `held-a`; the diagnostic Commit preserved that wrong byte. Container cleanup succeeded. [The initial log](qualification/live-docker-initial.log) is retained with checkout paths redacted. This failed result is not replaced by the earlier checkpoint's passing live proofs. This failure triggered the release correction below.

## Release correction and requalification

The deterministic regression reproduced an already-admitted WRITE waiting for inode ordering while `KernelEditGuard` was retired. It produced `Q0Z` instead of `QSZ`. The correction reuses `CacheFlush::finish` to drain admitted writeback, explicitly clears SDK protection while the reacquired cut still holds both operation lanes, then resumes normal operations. It adds no sleep, dependency, protocol operation, per-file descriptor mechanism, or weakened assertion. Notification failures retain the failed-owner/error path.

- [Failing-before deterministic regression](qualification/guard-regression-before.log)
- [One instrumented live diagnostic](qualification/live-diagnostic.log): passed, so it did not by itself establish the initial failure's precise interleaving or erase it.
- [All 26 FUSE library tests after correction](qualification/fuse-repair-tests.log)
- Predeclared full live runs [1](qualification/live-docker-repaired-1.log), [2](qualification/live-docker-repaired-2.log), and [3](qualification/live-docker-repaired-3.log): all three named tests passed in every run, cleanup included. No failed repaired-source attempt was repeated for a favorable result.

The correction establishes draining of callbacks already admitted to the writeback lane. These tests do not prove a universal drain for kernel work not yet admitted or a new crash-durability guarantee. Exact scope is retained instead of presenting the successful finite runs as exhaustive concurrency proof.

## Benchmark coverage and limits

The [published checkpoint](../../docs/roadmap/0.1/0.1.3/checkpoint-evidence/README.md) contains 198 successful performance cases and 226 successful routine proofs across 17 families. One 600-second endurance definition is excluded and unexecuted. Routine proofs include explicitly sampled content/history coverage; all declared checks passed within the unchanged 45-second work / 59-second hard deadline. Nineteen proofs exceed the aspirational 15-second wall.

The final tier100 mixed-v3 bulk observations are below one second, with narrow create margin; Git's 500/1,000 ms targets and unrelated-history500's historical 15-second target remain missed. The performance checkpoint has one fixed-seed sample per case. Historical optimization studies preserve their own source, sample, and proof scopes. No old failure or changed recipe is relabeled as a release-source result.

Native tests cover Store schema migration, SQL structural checks, public SDK operations, and failure recovery. Live tests cover their named concurrency/coherence/lifecycle paths, not exhaustive platform or physical 100-Workspace qualification. Crash/power-loss durability remains unsupported.

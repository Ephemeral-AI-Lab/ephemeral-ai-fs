# LayerFS 0.1.3 verification

> **Status:** Release candidate verification record for LayerFS 0.1.3.

## Source binding

The accepted benchmark checkpoint is `9f5a641d223606c45e5e6aa8a20094c12f9139a1`.
Release preparation starts from `28177560c8f049c02192e18c263cdc5543c1ab52`.
All tracked files under `crates/`, `tools/`, and `benchmark/fs-bench-pro/` are byte-identical to the checkpoint. Workspace package versions in root `Cargo.toml` and the 12 local lockfile package records advance from 0.1.2 to 0.1.3; third-party dependency records do not change. Manual, release documentation, entry-point links, and the explicit preview migration policy are updated separately.

The existing runner computes product seal
`3c797bc6dbfd9b03b919c270b609cad839000b68d67e34f3b00d24717e07f39a`, identical to the measured checkpoint. The build source seal includes root Cargo metadata and is different; original benchmark identities are not relabeled. See [machine-readable release evidence](release-evidence.json).

## Checks

| Check | Release preparation result |
|---|---|
| Rust formatting | `cargo +1.96.0 fmt --all --check`: PASS |
| Full workspace native tests | `RUSTUP_TOOLCHAIN=1.85.1 tools/test-fast.sh`: 348 passed, zero failed, one pre-existing ignored; 79 seconds, four bounded jobs |
| Workspace build | `cargo +1.85.1 build --workspace --locked`: PASS |
| Warning-denying Clippy | `cargo +1.96.0 clippy --workspace --locked -- -D warnings`: PASS |
| Shared benchmark infrastructure | 40 Python checks: PASS |
| Report regression checks | Two Python checks: PASS |
| Checkpoint report derivation | Rebuilt from existing raw receipts: PASS, no report errors; no performance reruns |
| Managed Docker/FUSE tests | Initial release-runtime run: two passed, one failed; mapped-write/SDK coherence investigation in progress |
| Documentation and whitespace | Local links and whitespace checked; final review required after repair |
| GitHub CI | Required on the final PR head and merged release source before publication |

The native suite includes platform-gated tests that return without live-Docker execution unless enabled. Its native PASS count does not establish FUSE qualification; the three tests below are executed separately with `LAYERFS_LIVE_DOCKER=1` and the source-bound release runtime image:

- `managed_container_lifecycle_and_disconnect_cleanup_are_exact`
- `running_commands_and_dirty_mappings_continue_across_commit`
- `ordinary_writes_queue_during_mapped_commit`

The image is local verification infrastructure, not a published release asset. Build provenance preserves the base commit/tree, dirty candidate status, source seal, product seal, and workload hash. A final identity comparison ensures subsequent documentation changes do not change the tested runtime inputs. The GitHub release announcement records the exact final commit and successful CI run, avoiding a self-referential commit hash inside its own tree.

## Initial release-runtime failure

The explicitly enabled live suite completed in 9.35 seconds with two passes and one failure. `ordinary_writes_queue_during_mapped_commit` observed byte 777 as zero rather than the SDK-written `S` in `held-a`; the diagnostic Commit preserved that wrong byte. Container cleanup succeeded. [The initial log](qualification/live-docker-initial.log) is retained with checkout paths redacted. This failed result is not replaced by the earlier checkpoint's passing live proofs. Release publication is blocked until the cause is repaired and checked.

## Benchmark coverage and limits

The [published checkpoint](../../docs/roadmap/0.1/0.1.3/checkpoint-evidence/README.md) contains 198 successful performance cases and 226 successful routine proofs across 17 families. One 600-second endurance definition is excluded and unexecuted. Routine proofs include explicitly sampled content/history coverage; all declared checks passed within the unchanged 45-second work / 59-second hard deadline. Nineteen proofs exceed the aspirational 15-second wall.

The final tier100 mixed-v3 bulk observations are below one second, with narrow create margin; Git's 500/1,000 ms targets and unrelated-history500's historical 15-second target remain missed. The performance checkpoint has one fixed-seed sample per case. Historical optimization studies preserve their own source, sample, and proof scopes. No old failure or changed recipe is relabeled as a release-source result.

Native tests cover Store schema migration, SQL structural checks, public SDK operations, and failure recovery. Live tests cover their named concurrency/coherence/lifecycle paths, not exhaustive platform or physical 100-Workspace qualification. Crash/power-loss durability remains unsupported.

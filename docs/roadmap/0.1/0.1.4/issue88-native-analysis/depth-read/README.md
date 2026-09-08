# Frozen depth-stratified public read adapter

This is an additive adaptation of the sealed issue88 public read probe. It has no product changes and performs no history import, Commit, encoding, migration or Store copy creation. `select_cohort.py` and `proof/` have separate ownership and seals. The probe's tool seal includes only `Cargo.toml`, `Cargo.lock`, `run.py`, this README and `src/{main,resources}.rs`.

The root task builds/tests/runs under the infrastructure lock. New probe binaries must link the two exact frozen product sources separately; retain the same normal images/host environment. Do not reuse the old probe binary hashes after adapting input. Example build command shape (root chooses frozen checkout and isolated target/custody paths):

```sh
cargo build --locked --release --manifest-path docs/roadmap/0.1/0.1.4/issue88-native-analysis/depth-read/Cargo.toml --target-dir ABSOLUTE_TARGET
```

The executable is `issue88-depth-read-probe`. The sidecar `<binary>.identity.json` has `binary_sha256`, `tool_files` as returned by `run.tool_seal`, and the exact frozen `LAYERFS_SOURCE_SEAL`/`LAYERFS_PRODUCT_SEAL`. Root retains all actual build command/source/dirty-patch/dependency/binary identities. No build or test is launched by this README.

Runner CLI:

```text
python3 run.py --arms ARMS.json --cohort COHORT.json --cohort-sha256 SHA256 \
  --copies COPIES.json --copies-sha256 SHA256 --output NEW_CAMPAIGN_DIRECTORY
```

`ARMS` has exactly `control` and `candidate`, each with `source_run`, `probe_binary`, and `image`. Original full157 performance and normal verification evidence must be complete, sealed and quiescent. Producer source/product identity remains distinct from the new tool identity.

`COHORT` schema `issue88-depth-read-cohort-v1`, status PASS, contains exactly five `selections` ordered depth0..4. Each selection supplies checkpoint1..157, source SHA/tree, `source_oracle_sha256` and `source_manifest_sha256`, `path_hex`, file length4096..8388608, offset, range length4096, expected range/full SHA256s, each arm's branch/commit mapping, and `proof:{path,sha256}`. Candidate additionally supplies `target_id`, `pack/group/record`, `raw_bytes`, `depth`, `closure_raw_bytes`, and `target_span:{logical_offset,object_offset,length}`. The complete4KiB range must lie inside that exact authenticated extent span; source offset+span length must fit the target raw chunk. Path bytes are canonical relative components with no NUL, empty component, `.` or `..`. Proof and input hashes are checked; no timed per-ObjectId logs are added.

`COPIES` schema `issue88-depth-read-copies-v1` has `copies:{control:ENTRY,candidate:ENTRY}`. Each ENTRY supplies `path` to prepared `store.sqlite`, `source_path` to its **pre-verification logical snapshot**, `source_sha256`, `post_proof_sha256`, and `snapshot_custody:{path,sha256}`. Copy hash must remain equal to the inventory-bound snapshot after the bounded proof. Snapshot custody must bind `copy_sha256`, exact `primary_final_ack` and the original `performance_manifest_sha256`. Post-verification originals may contain verifier Branch metadata and therefore have a different Store hash; their normal verification manifests are authenticated independently, never substituted for the snapshot's identity. Copies are sibling `copies/control` and `copies/candidate` directories in one disjoint preparation directory. They are reused for the entire campaign; subsequent fork/session metadata changes are permitted only there and final hashes are recorded. No allocation equivalence is claimed.

Schedule: five strata × range/full × three repetitions × two arms =60 observations. Per stratum/operation order C/P, P/C, C/P. A fresh host process, Client, fork, container/daemon and FUSE workspace is used per observation. The fixed shell pipeline reads through public Exec and drains byte-count output inside the timer; a second digest Exec is outside that timer. Byte paths are passed as literal `OsString` positional arguments to Bash `"$1"`, never shell text. A selected target's depth is a static property; dynamic timed-read native depth/decode/edge aggregates corroborate the proof without pretending to identify a record by counter alone. Candidate timed reads must show the declared depth and enough decoding/dependency work. Digest can reuse warm FUSE data and is not required to repeat those counters. Control must not report native record/decode activity. Whole-session FUSE receipts remain whole-session; missing per-action FUSE counts stay null.

Limits remain30s/read or digest,120s lifecycle,4h campaign, frozen2CPU/2GiB/no-swap/256PID container,8GiB host RSS,16GiB scoped runtime,32GiB owned preparation+campaign outputs and50GiB free reserve. The measurement lock covers validation through final seals. Every row records timing, phase deltas, current/lifetime RSS, container current/peak resources, copy/temp/control runtime scopes, source identity and cleanup. Digest/observers/cache effects are separate; no cold-OS-cache claim. Failures stop without replacements and retain row JSON/logs; originals are re-sealed after the campaign. Correctness/depth/resource failures are not reported as fast or zero-cost observations.

Root-only focused checks: `python3 run.py --self-test` exercises scheduling and dynamic-coverage rejection without Store/container access; `cargo test --locked --manifest-path .../depth-read/Cargo.toml` includes the byte-path argument check. They have not been executed by the source author.

Reviewed preflight additions: governing `native-design/depth-read-contract-v1.md` must match `cohort.inputs.contract`; the command seals both it and the old prospective reference. Exact proof copy paths/hashes must match the prepared copies; inode/device checks reject hardlinks to any original or immutable snapshot. The proof selection identity, per-arm retained-path facts, exact candidate span, target locator, ordered native chain and raw closure must agree with cohort fields. Ready receipts expose absolute fresh-Store physical counters through setup; any native fetch/decode before the timed read invalidates fresh-application coverage. Timed candidate reads also require at least depth+1 native record fetches. These remain aggregate corroboration of the static proof, not per-ID traces.

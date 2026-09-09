# Full Torch .venv Workspace acceptance

**PASS — all three frozen-workload samples and exhaustive independent readbacks completed.** This finishes the previously paused issue #71 Workspace acceptance on the current v0.1.4 implementation. No product changes, resource increases or assertion relaxations were needed.

The frozen archive matched the original source in all 17,682 entries, 581,658,413 regular-file bytes, modes, nanosecond mtimes and symlink targets before execution. Every resulting Store then passed independent full readback of 16,395 files, 1,283 subdirectories and three symlinks, including the root metadata and exact directory entry sets. The `.venv` here is the original paused task's frozen 581,658,413-byte fixture; it does not establish the distinct older 1.434 GB / 40,831-file aspiration.

## Observations

| Sample | Exec (s) | Commit (s) | Exec + Commit (s) | Host CPU (s) | Host peak RSS (MiB) | Container peak (MiB) | Full proof |
|---|---:|---:|---:|---:|---:|---:|---|
| 1 | 6.279578 | 7.088442 | 13.368019 | 8.287519 | 121.672 | 65.324 | PASS |
| 2 | 6.081551 | 7.469642 | 13.551193 | 8.550364 | 120.922 | 57.031 | PASS |
| 3 | 6.237479 | 7.083119 | 13.320598 | 8.561944 | 122.828 | 56.988 | PASS |

Median Exec + Commit: **13.368019 seconds**. These are three descriptive candidate observations with uncontrolled caches and no successful untouched-release comparator. They are not an eligible comparative performance claim or an addition to the formal 198-case registry. Original native Init measurements are a different operation.

## Contract and correctness

The [plan](plan.json) retains the original tar extraction through public `Client::exec_workspace_session` into a real FUSE Workspace, followed by `commit_workspace_session_with_status`. It requires a Created Commit, healthy presentation, End(Clean), zero active sessions/executions and owned-container removal. All samples passed, with zero swap and zero OOM/OOM-kill events under 2 CPUs, 2 GiB RAM and 256 PIDs. macOS owns SDK, SQLite and publication; Docker owns daemon/FUSE/workload.

Setup and archive delivery are excluded from Exec/Commit timers. Independent verification executes only after the workload process has exited and closed its Store. The shared measurement lock serializes preparation, workload and readback. Outer workload and verification deadlines remain 480 and 180 seconds, with the original 300-second Exec deadline. Earlier attempts and the paused task's uncommitted evidence remain untouched.

## Source and custody

- Frozen acceptance source: `da2d474afd2f0e4fcd11a5311481c6b7e68c1a6a`; Rust product, schema, root lockfile and workload runtime are unchanged from the prepared v0.1.4 candidate.
- Proof binary SHA-256: `10cd62c0c78780beb374a07a701161725babfc646fe880facc17f40b129984c6`.
- Runtime image: `sha256:7679d908d50aa7cac722d0e5fe9f3c5dd1a8e54a1cb3f1728c97a26b26200974`, previously sealed from release source `70fdd839dd68c91491524590ed034b9b516431bd`; subsequent changes are documentation/evidence/proof tooling.
- Input tar SHA-256: `fdc1d8f63eba66c44af192929544c72d7d56126f155107e29de1c8c0de0e3314`.
- Full source archive bytes: 613,027,840; file payload bytes are reported separately above.
- [Results](results.json), [input custody](receipts/input-custody.json), [full identities](receipts/identity.json), [raw receipts](receipts/) and [SHA-256 manifest](manifest.sha256.json). Original fresh Stores, frozen binary and complete raw files remain at `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v014-venv-acceptance`.

The copied existing probe compiled against current crates and passed formatting and warning-denying Clippy. Imported package versions, checksums and resolved dependencies match the root release lockfile. The first preparation audit compared raw dependency-reference text and rejected Cargo's removal of redundant `windows-sys` version qualifiers; resolved dependency identities were then checked and passed. This was a preparation-check correction, not a product/workload failure.

To reproduce, build `probe/Cargo.toml` with Rust 1.85.1, `--release --locked --offline -j2` and `CARGO_TARGET_DIR` set to the repository target directory under the shared lock; then run `python3 run.py FRESH_OUTPUT`. The runner acquires the lock itself. It refuses an existing output, verifies the original archive/source, performs three fresh samples, preserves observations and runs the independent oracle.

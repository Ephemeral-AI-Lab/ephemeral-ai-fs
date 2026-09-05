# LayerFS benchmark quick note

- **Permanent policy:** Docker-owned SQLite and container-side benchmark coordinators are prohibited. Use host-owned Stores for preparation, performance, and verification; migrate unsupported families to the host instead of restoring a Docker fallback.
- **Environment:** macOS runs the SDK, Workspace processing, and embedded SQLite. Docker Linux runs the daemon, workload helper, and real FUSE.
- **Limits:** container **2 CPUs / 2 GiB RAM / no swap / 256 PIDs**. Host CPU is uncapped. No Docker data mounts.
- **Iteration:** reuse preparation and run **one explicit performance sample** per experiment. Collect selected independent proofs only after performance collection and the candidate are stable. Keep builds, samples, and proofs serial.
- **Performance allowance:** Workspace calls may execute for **120 seconds** to expose complete timings; the outer command defaults to 130 seconds. The complete product-call pass target stays **15 seconds**. Completed slower runs are `TARGET_MISS`; timeouts are incomplete. Optimize the lowest failing tier before advancing.
- **Timing:** SDK edit results measure **edit + Commit in milliseconds**. Setup and verification are separate.
- **Verification:** bounded SDK edit checks, sampled namespace checks, and storage accounting with bounded edit-region checks. **No SDK full-byte option.**

## Run one case

Docker Desktop must be running. The runner manages sample containers and FUSE; no separate SQLite service is required.

```bash
cd /Users/yifanxu/Ephemeral-AI-Lab/layerfs

# Build the host coordinator and the daemon/workload-only Linux image.
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
export LAYERFS_BENCH_IMAGE="$(python3 benchmark/fs-bench-pro/shared/runner.py --build-image)"

# List cases without running benchmarks.
target/release/fs-benchmark-pro infra-list edit_length_changing

# Run one complete sample.
python3 benchmark/fs-bench-pro/shared/runner.py \
  --topology host-store \
  --family edit_length_changing \
  --case insert-middle-4k-on-500mib-result-capped-v2-ops-1 \
  --repetition 1 \
  --perf-fast
```

Use `--repetition 1` for the three SDK edit families; use `--seed 1` for the other families. Results go to a new directory under `benchmark-results/host-store/results/` by default. Use `--output PATH` to choose a new output directory; existing evidence is not overwritten.

After performance collection, run separate verification through `verify-selected.py`, using the exact family, case, source, input, image, and seed/repetition identities from `perf.jsonl`. For SDK proofs, also bind the performance record's `row_id` with `--performance-rows`. Reuse compatible protected preparation; each proof has a 45-second work allowance and 59-second hard end-to-end deadline. Verification writes `verification.json` to its own new output directory.

## Rebuild after source changes

Requires Rust toolchain `1.85.1`, Python 3, and Docker Desktop. Builds use release mode and two build jobs.

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
export LAYERFS_BENCH_IMAGE="$(
  python3 benchmark/fs-bench-pro/shared/runner.py --build-image
)"
```

## Recorded baseline

Nine families: `payload_create_read`, `dedup_workspace_reuse`, `dedup_cross_file`, `dedup_cdc_locality`, `edit_length_preserving`, `edit_length_changing`, `edit_canonical_chunk_count`, `init_namespace`, and `store_footprint`.

**118/118 performance cases and 27/27 selected proofs passed.** One performance sample per case establishes a baseline, not a statistical distribution. The proofs use their explicitly recorded coverage; they do not establish exhaustive byte/namespace verification. Other families are deferred to **#39**.

See the [baseline report and exact verification coverage](../../docs/roadmap/0.1/0.1.3/nine-family-fast-baseline.md). During normal iteration, rerun the affected case rather than replaying the whole baseline.

## Tiny-file churn mixed bulk v3

The approved [mixed-v3 contract](../../docs/roadmap/0.1/0.1.3/tiny-file-churn-mixed-v3.md) replaces only the high-tier bulk rows with `tiny-bulk-create-100-mixed-v3`, `tiny-bulk-delete-100-mixed-v3`, `tiny-bulk-create-500-mixed-v3`, and `tiny-bulk-delete-500-mixed-v3`. Family membership remains 20. Tier 100 affects 1,000 files / 100 MiB; tier 500 affects 5,000 files / 500 MiB. Both retain the separate 200-file / 1 MiB witness. Low-tier compact and individual create/stat/unlink cases are unchanged. Old IDs and receipts are historical; fewer operations do not establish a product speedup.

After building the host binary and workload image, select one revised case:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --topology host-store --family tiny_file_churn --case tiny-bulk-create-100-mixed-v3 --seed 1 --setup clone --perf-fast
```

Use `tiny-bulk-delete-100-mixed-v3` for the separate serial delete sample on the same source. The broader family target remains 15 seconds; the sample also records the separate strict #47 assessment (`pure_call_sum_ns < 1,000,000,000`) for the revised tier-100 pair. Tier-500 performance and independent final proofs are deferred. Proof selection includes every large file with three 64 KiB ranges (beginning, midpoint, end), declared small/medium paths, and the witness; delete checks corresponding absence. Report omissions explicitly.

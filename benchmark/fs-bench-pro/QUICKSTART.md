# LayerFS benchmark quick note


- **Permanent policy:** Docker-owned SQLite and container-side benchmark coordinators are prohibited. Use host-owned Stores for preparation, performance, and verification; migrate unsupported families to the host instead of restoring a Docker fallback.
- **Environment:** macOS runs the SDK/benchmark coordinator, canonical construction/Commit publication, physical spool backing, and embedded SQLite. Docker Linux runs the daemon, workload helper, and real FUSE; #49 moves live operation state into that daemon while keeping backing and Store publication on the host.
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

## Full tier500 mixed-v3 diagnostic run

The user-authorized tier500 extension uses600seconds product time,630seconds outer command time and600seconds preparation. This changes execution allowance only; the15second family PASS target and resource caps remain. Select create and delete separately and serially:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --topology host-store --family tiny_file_churn --case tiny-bulk-create-500-mixed-v3 --seed 1 --setup clone --perf-fast --product-timeout 600 --timeout 630 --setup-timeout 600
```

Then select `tiny-bulk-delete-500-mixed-v3` with the same options and source. This authorization supersedes the earlier tier500-performance deferral above. Other runs retain the120second product/130second outer defaults. Independent proof budgets remain45/59seconds.

## Issue49 create-100 fast iteration: prepare once, clone each sample

The primary selection is `tiny-bulk-create-100-mixed-v3`, seed1: 1,000 created files /100MiB including one50MiB file, plus the unchanged separate200-file/1MiB witness. Use the family scripts below; they delegate to the existing host-store runner. Do not add a benchmark engine or direct benchmark execution inside Docker.

Build a matching pair after product-source changes, using the existing incremental Cargo and Docker caches. Reuse an already matching sealed pair when no relevant source changed; documentation-only changes do not require a rebuild. The runner rejects a stale host binary or differing host/image product seals.

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
export LAYERFS_BENCH_IMAGE="$(python3 benchmark/fs-bench-pro/shared/runner.py --build-image)"

# Initial preparation, or ensure compatible preparation after an input/schema change.
bash benchmark/fs-bench-pro/families/tiny_file_churn/setup.sh \
  --topology host-store --case tiny-bulk-create-100-mixed-v3 \
  --seed 1 --setup clone --image "$LAYERFS_BENCH_IMAGE"

# One full, serial sample after a substantive implementation change.
bash benchmark/fs-bench-pro/families/tiny_file_churn/perf.sh \
  --topology host-store --case tiny-bulk-create-100-mixed-v3 \
  --seed 1 --setup clone --perf-fast --image "$LAYERFS_BENCH_IMAGE"
```

`setup.sh` is preparation-only and does not run a performance sample. It creates a protected host master only when a compatible one is absent. The performance script also acquires preparation automatically, so do not run setup before every sample when it adds no value. Compatibility uses the existing fixture/schema/seed rules and content identity; a new product revision alone does not require deleting a compatible master. Preserve the master's producing identity and the new candidate's identity separately.

**Use `--setup clone` for normal create-100 reset.** The actual clone method is `closed-quiescent-byte-copy`: the runner copies a closed validated host master into a fresh disposable host sample Store and checks master isolation. It is not an APFS clone/reflink, not a live SQLite file copy, and not permission to reuse the previously mutated sample. The runner still starts and cleans up each sample's container/FUSE session. Do not prune Docker/build caches, restart Docker, delete protected preparation or recreate the base namespace as routine reset. The CLI alternative is `fresh`, not `refresh`; use fresh preparation only for an explicit preparation/invalidation investigation. Native initialization's fresh-output policy remains unchanged.

Inspect preparation `cache_hit`, compatibility, setup mode/clone method, source/image identity, `prepared_master_unchanged`, complete product timings and cleanup. A new input/schema incompatibility should create/acquire the appropriate master through the runner; never force an incompatible cache hit or weaken isolation checks. Setup and cleanup retain their declared timer scopes.

During implementation: one hypothesis, smallest changed-seam checks, matching build artifacts, one create-100 sample, inspect, retain/revise. Keep serial shared-lock coordination. Component checks supplement real FUSE execution; a host-only synthetic test cannot establish Docker/FUSE performance. Do not multiply performance seeds or rerun passing suites without a relevant change. Independent sampled proofs stay final-only through `verify.sh`/`verify-selected.py`, with exact receipt identities and45-second work/59-second hard limits. Physical100-workspace qualification remains deferred. These are execution instructions, not a request to run a benchmark during documentation work.

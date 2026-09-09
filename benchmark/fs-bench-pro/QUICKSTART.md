# LayerFS benchmark quick start

Use the family scripts or the shared runner. macOS owns the SDK/coordinator,
SQLite, canonical publication and spool. Linux Docker owns the daemon, real FUSE
and workloads. No Docker-owned SQLite, data-sharing mounts or fallback topology.
See [benchmark rules](../../docs/general/benchmark_rules.md) and
[the checkpoint contract](../../docs/roadmap/0.1/0.1.3/checkpoint-74-75.md).

## Build once per relevant change

From the repository root, with Docker Desktop running:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
export LAYERFS_BENCH_IMAGE="$(python3 benchmark/fs-bench-pro/shared/runner.py --build-image)"
```

Reuse matching builds and protected prepared inputs. Source/product/image seals
are checked. Builds, performance and verification share a measurement lock: do
not overlap resource-sensitive work or interrupt another owner's live run.
The standard container has 2 CPUs, 2 GiB RAM, no swap and 256 PIDs. Host CPU and
memory remain separate resource scopes.

## Select one case

```bash
target/release/fs-benchmark-pro infra-list git_tool_workflow
bash benchmark/fs-bench-pro/families/git_tool_workflow/perf.sh \
  --case git-tool-100-mixed-v4 --seed 1 --setup clone \
  --image "$LAYERFS_BENCH_IMAGE" --perf-fast --collection-mode \
  --product-timeout 300 --timeout 310 --setup-timeout 600
```

Fast performance means one complete sample, not less workload. The checkpoint
uses 300 seconds for the complete product workload and 310 seconds outer wall.
The normal selected runner defaults remain 120/130 seconds. Collection mode
reports historical latency targets separately from execution success.

Use `--repetition 1` for SDK edit families and `--seed 1` for other families.
Initialization uses `--setup fresh`; post-initialization cases normally use
`--setup clone`. Clone means a closed, validated, independent writable byte copy,
not an APFS clone and not a cold-OS-cache claim. Each run gets a fresh live owner.

Preparation is automatic on a cache miss. Run a family's `setup.sh` only when
explicit preparation is useful; do not repeat setup before every sample. Keep
master hashes/isolation, fixture versions and cleanup checks. Never reuse mutated
samples, clear protected caches routinely, or move cold product work into setup.
Use `--output` with a fresh path; raw receipts must not be overwritten.

## Verify separately

Use the family's `verify.sh` with the exact case, seed/repetition, source, input,
image and setup identity from the performance receipt. SDK proofs also bind the
performance `row_id` using `--performance-rows`. Verification has a 45-second
work allowance and 59-second hard deadline. Prepare compatible inputs separately
when necessary; report preparation time outside verification.

The normal verifier uses the family's practical coverage: bounded SDK regions,
selected paths/ranges, or selected historical snapshots. Git keeps its full
semantic head/tree/parent and reopened-custody checks. Every receipt must state
actual coverage and omissions; sampled PASS is not exhaustive qualification.
The 600-second sustained proof is a separately accounted optional long test.
Existing bounded parallel-read/write and repeated-publication proofs cover the
routine sustained-operation smoke; do not relabel either as 600 seconds.

## Full checkpoint — one shared campaign

After repairs, freeze source and the registry. This single campaign serves both
#74 (passing suite) and #75 (performance tables):

```bash
python3 benchmark/fs-bench-pro/issue54_collect.py --checkpoint \
  --image "$LAYERFS_BENCH_IMAGE" \
  --output benchmark-results/host-store/campaigns/checkpoint-final
```

The collector reuses the shared runner, lists admitted families once, collects
one sample per active case and runs routine proofs. Obsolete capped-edit entries
outside the host runner are not additional active cases. Keep all failures and
requalify affected results after a fix; do not repeat successful cells for nicer
numbers. During implementation use selected cases, not repeated full campaigns.

Publish Markdown, JSON and CSV grouped by family and exact test ID. Report the
original timer, phases, setup/proof/cleanup wall, resources and coverage. Compare
only matching workload profiles/topologies/timers. Mixed-v3/v4 and history-v2
replaced older workloads; fewer files are not evidence of a product speedup.
Previous campaign reports retain their original source and coverage limitations.

### Bounded historical access (#101)

`historical_access` opens explicitly supplied retained history through public
SDK/FUSE. Build host and image separately with the commands above. It never
constructs history or builds prerequisites during a selected test.

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access --list
python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access \
  --case ha-small-head-v1 --store /absolute/path/to/sealed/store.sqlite \
  --image "$LAYERFS_BENCH_IMAGE" --output /absolute/path/to/new/performance
python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access \
  --case ha-small-head-v1 --store /absolute/path/to/sealed/store.sqlite \
  --image "$LAYERFS_BENCH_IMAGE" --mode verification \
  --performance /absolute/path/to/new/performance/result.json \
  --output /absolute/path/to/new/verification
```

The v1 manifest pins the existing closed53-state schema9 Store and original
checkpoints1/58/157. Any different Store fails compatibility validation; it is
not silently rebuilt. See `families/historical_access/fixture.json` and the
[contract](../../docs/roadmap/0.1/0.1.5/issue101/historical-access-v1.md).
Each selected invocation has one15second deadline including preparation and
teardown. Verification has its own15second watchdog. `--all` explicitly runs
all ten performance cases serially with separate envelopes; verification binds
one selected performance receipt. These are diagnostic qualifications, not
release admission or a paired product speedup campaign. #102 owns that campaign.

# Ordinary full157 and access handoff

Status at this handoff: current full157 performance, census, same-Store history
verification, current Git157 binding/verification, and 11 access cases plus their
11 separate proofs are **NOT_RUN**. The helpers are implemented and their seven
focused grammar/custody tests passed (`668b5c73a`). No #116 audit has started.

Freeze comparator choice, exact artifact identities, output paths and applicable
criteria before running. `qualified-v2` is the available pre-fsync control;
`fsync-qualified` is the current fsync candidate. Both have captured host binary,
identity and `image.txt` files under the evidence root below. The earlier
`full157-attribution-plan.json` still names `pre-edit-control` and is explicitly
pending final treatment binding: do not execute that arm choice implicitly.
The next agent may use the qualified fsync treatment as the ordinary-storage
baseline for a later #107 experiment. A matched comparison requires a fresh
history on each declared arm; historical output is not a new construction result.

Read-only readiness checked all 157 input directories, receipts, manifests and
original oracle paths; none were missing. Pinned manifest SHA256 is
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`, source tip
`b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. The matching existing
`deepseek-history-data/git-snapshots-157-v1` repository, mapping, results and exact
object-ID list are present. The public runner must still authenticate complete
input contents and executable/image identities; path presence is not that proof.

## Per-arm commands

Run from the repository. Set these variables for one prospectively selected arm;
use `baseline` for an archived comparator and `candidate` for the final current
treatment. Use fresh output names. Read the captured image rather than copying an
old source seal or guessing an image tag.

```bash
set -euo pipefail
I118_REPO=/Users/yifanxu/Ephemeral-AI-Lab/layerfs
I118_EVIDENCE="$I118_REPO/benchmark-results/host-store/issue118/20260912"
I118_DATA=/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data
I118_ARM=baseline                         # candidate for the final arm
I118_ARTIFACT="$I118_EVIDENCE/qualified-v2" # fsync-qualified for the current candidate
I118_BIN="$I118_ARTIFACT/fs-benchmark-pro"
I118_IMAGE="$(cat "$I118_ARTIFACT/image.txt")"
I118_RUN="$I118_EVIDENCE/full157-${I118_ARM}-1"
cd "$I118_REPO"

python3 benchmark/fs-bench-pro/shared/runner.py \
  --family repository_history --profile stride-1 --source-arm "$I118_ARM" \
  --repetition 1 --data "$I118_DATA" --host-binary "$I118_BIN" \
  --image "$I118_IMAGE" --output "$I118_RUN"

python3 benchmark/fs-bench-pro/shared/integrated_storage.py --freeze-access "$I118_RUN"
python3 docs/roadmap/0.1/0.1.5/issue100/census.py "$I118_RUN" --output "$I118_RUN/census.json"

python3 benchmark/fs-bench-pro/shared/runner.py \
  --family repository_history --profile stride-1 --source-arm "$I118_ARM" \
  --repetition 1 --data "$I118_DATA" --host-binary "$I118_BIN" \
  --image "$I118_IMAGE" --storage-verify-run "$I118_RUN"
```

The freeze and census must happen **after successful producer closure and before
verification**. The verifier opens that same measured Store and checks all 157
original oracles. Its bookkeeping can grow the Store. Keep the original measured
allocation, frozen copy and post-verification growth separate; never use an
independent copy's allocated blocks as a construction saving. Do not add
`--perf-fast`, `--collection-mode`, `--seed` or `--verification` to these specialized
history commands. Stride-3/10 remain unselected optional profiles.

## Matching Git and candidate access

After the final candidate's complete history verification, retain that candidate's
`I118_RUN`, `I118_BIN` and `I118_IMAGE` values for this sequence. Access requires the
current matching candidate executable/image. The Git helper reads the existing
packed 157-state control; it does not build, repack, check out or garbage-collect it.

```bash
I118_ACCESS_FIXTURE="$I118_RUN/ordinary-access-fixture.json"
I118_ACCESS_STORE="$I118_RUN/deepseek-full/frozen-measured-store/store.sqlite"
I118_ACCESS_PERF="$I118_RUN/access-performance"

python3 docs/roadmap/0.1/0.1.5/issue100/experiments40/full157/git-baseline/verify.py \
  --run "$I118_RUN" --data "$I118_DATA" --output "$I118_RUN/git157-proof"
python3 benchmark/fs-bench-pro/shared/integrated_storage.py \
  --prepare-access "$I118_RUN" --data "$I118_DATA" --output "$I118_ACCESS_FIXTURE"
python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access --all \
  --fixture "$I118_ACCESS_FIXTURE" --store "$I118_ACCESS_STORE" \
  --host-binary "$I118_BIN" --image "$I118_IMAGE" --output "$I118_ACCESS_PERF"

for I118_CASE in \
  ha-stat-old-f157-ordinary-v1 ha-stat-head-f157-ordinary-v1 \
  ha-directory-old-f157-ordinary-v1 ha-directory-head-f157-ordinary-v1 \
  ha-small-old-f157-ordinary-v1 ha-small-head-f157-ordinary-v1 \
  ha-range-history-cold-f157-ordinary-v1 ha-range-history-warm-f157-ordinary-v1 \
  ha-full-head-cold-f157-ordinary-v1 ha-full-head-warm-f157-ordinary-v1 \
  ha-metadata-worst-f157-ordinary-v1; do
  python3 benchmark/fs-bench-pro/shared/runner.py --family historical_access \
    --case "$I118_CASE" --mode verification \
    --performance "$I118_ACCESS_PERF/$I118_CASE/result.json" \
    --fixture "$I118_ACCESS_FIXTURE" --store "$I118_ACCESS_STORE" \
    --host-binary "$I118_BIN" --image "$I118_IMAGE" \
    --output "$I118_RUN/access-verification/$I118_CASE"
done
```

## Budgets, retention and #107 decision

Existing history limits remain: 14400 seconds for immutable input validation and
each performance/verification phase, 300 seconds per operation, and 120 seconds
for setup/cleanup. Freeze copying has 120 seconds. Proposed outer allowances for
the read-only census and Git proof are 120 and 300 seconds; use the existing
deadline driver, not a new collector. Watch stdout and
`deepseek-full/{performance,verification}-{pending,step}-N.json` progress at least
every 60 seconds; inspect stalled work rather than blindly waiting for the limit.
Each access performance selection and each separate access verification retains
its **complete 15-second envelope**, including preparation, receipts and cleanup.

Keep the runner-owned measurement lock and existing container/host resource caps;
do not double-lock. The most recent free-space observation was above 50 GiB, but
recheck the normal runner gate. Preserve all prepared inputs, original/frozen
Stores, raw receipts, failures, manifests and helper identities. No automatic
input eviction, output rewriting, compaction, VACUUM or post-workload repacking.
The history cache profile is fresh Store with existing uncontrolled OS caches;
it supplies no verified-cold Init result.

Before any new ordinary-storage optimization, publish the current baseline's
exact allocated/apparent byte attribution and current matching Git distance.
Census separates frames, pack framing, metadata/value groups, SQLite table/index
pages, free/unused pages and allocation differences. Physical bases are subsets,
not additive bytes; per-record compressed metadata shares and unlocated compact
records' raw lengths remain explicitly unavailable. Product history/access
verification supplies semantic authentication, not the physical census.

Only after that attribution, prospectively declare the concrete #107 mechanism,
quantitative target and latency/resource constraints. A proposed **at least 1%**
ordinary allocated-byte benefit is a future experiment's floor, not a retroactive
PASS for baseline collection. The old compacted 66 MB criterion is obsolete.
Report live Git allocation and its recorded reference separately if blocks drift;
Git preserves narrower metadata and packs offline, so its packing time is not a
foreground LayerFS save comparator. No current history/access gate is waived by
the owner's separate cold-Init 2.7-second waiver.

# Handoff prompt: finish v0.1.6

You are taking over issue [#122](https://github.com/Ephemeral-AI-Lab/layerfs/issues/122)
in the repository `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`.

**Read this whole file before running anything.** It contains the state, the
exact commands, and the four traps that already cost the previous agent most of
a session.

## 1. Mission and stop condition

Finish v0.1.6: implement every case of the frozen roadmap, obtain a passing
performance run **and** a passing independent verification run for every
applicable case, and publish the actual measurements to issue #122.

You are **not done** until all of the following hold simultaneously:

1. All **33 regular cases** in `docs/roadmap/0.1/0.1.6/cases.json` have a
   retained `performance` receipt with `status=PASS` and a retained
   `verification` receipt with `status=PASS`, both on the same final source and
   image, both showing every requested Commit returned `Created` with
   `presentation_failed=false`.
2. All **three extended cases** have a terminal result:
   `v016-mixed-exhaustive-100mb-5000-k100-v1` (verify, 120 s cap),
   `v016-mixed-exhaustive-500mb-30000-k100-v1` (verify, 300 s cap), and
   `v016-workspace-four-100mb-5000-k100-v1` (perf **and** verify, 60 s cap each).
   Verify-only cases have performance explicitly `N/A`, never zero and never PASS.
3. Seeds **1, 2 and 3** qualification is complete for every case.
4. No required row is `FAIL`, `TIMEOUT`, `INCOMPLETE`, `NOT_READY`,
   `NOT_RUN` or `NOT_RUN_EXTENDED`, and no row is silently omitted.
5. Every failure encountered on the way either has a root-cause fix with the
   before/after evidence retained, or is reported as an exact unresolved row
   with the reason and the required action.
6. Issue #122 carries the final per-case tables plus a machine-readable
   complete matrix.

**Do not stop for routine permission.** Approval prompts are disabled in this
session; actions needing approval are rejected automatically. Do not ask before
long debugging runs, rebuilds, or posting to the issue — the owner has already
authorized implementation progress and results on #122.

**Do not stop after a scaffold, a static check, or one passing family.** The
previous agent stopped there and it was not completion. The five-stage M1 mixed
workload is the centre of this roadmap and it does not exist yet.

You **may not**: create a release or tag (explicitly out of scope); weaken a
workload, shorten K100, raise a timeout after observing a miss, move a failed
regular case to extended, add sleeps or duration-filling loops, delete or relabel
retained evidence, or claim a pass you did not measure.

## 2. Ground truth: read these first

Committed specification (frozen, do not edit retroactively):

- `docs/roadmap/0.1/0.1.6/README.md`
- `docs/roadmap/0.1/0.1.6/benchmark-families.md`
- `docs/roadmap/0.1/0.1.6/fixtures.md`
- `docs/roadmap/0.1/0.1.6/workloads.md`
- `docs/roadmap/0.1/0.1.6/execution-and-verification.md`
- `docs/roadmap/0.1/0.1.6/review-decisions.md`
- `docs/roadmap/0.1/0.1.6/cases.json` (all 36 expanded cases)
- `docs/roadmap/0.1/0.1.6/check_plan.py` — run it; it must print `PASS`

Rules and topology:

- `benchmark/AGENTS.md` (host owns SQLite/SDK/coordinator/spool; Docker runs
  only Linux daemon/FUSE/workload helper)
- `docs/general/benchmark_rules.md` (normative)
- `benchmark/fs-bench-pro/QUICKSTART.md`

## 3. Exact current state

Specification commit `63d8c6c04`. Implementation commits `3f38e82ca`,
`12c047ca1`, `dcd8ed591`. All pushed to `main`; working tree clean.

| Family | Regular cases | State |
| --- | ---: | --- |
| `file_size_transition` | 7 | **6 pass perf + verify; 1 perf PASS / verify FAIL** |
| `dedup_branch_history` | 6 additions | not implemented |
| `mixed_load_bearing` | 4 | not implemented |
| `multi_workspace_development` | 4 | not implemented |
| `branch_development` | 6 | not implemented |
| `historical_access` | 6 additions | not implemented |
| (extended) | 3 | `NOT_RUN_EXTENDED` |

So: **6 of 33 fully green, 1 half-qualified, 26 never run, 3 never run.**

What exists and works, reuse it rather than rewriting:

- `benchmark/fs-bench-pro/workload/v016_common.rs` — the single implementation
  of the v0.1.6 numbers: L100/L500 class tables and byte equations, reserved
  capacity, 100-name directory cap, five-stage M1 counter table, refresh-cohort
  recurrence algebra, topology cardinalities. Product-free, with three unit
  tests, all passing.
- `benchmark/fs-bench-pro/families/file_size_transition/mod.rs` — the seven
  boundary cases with an independent oracle, `plan`, `operations`,
  `expected`, `declared_length`, `pre_replacement_shared`, `self_check`.
- Registry wiring in `workload/workspace_registry.rs` (139 timed IDs) and host
  orchestration in `src/workspace_bench.rs`.

Latest measurements are in the last two comments on #122. Raw evidence is
local-only under `benchmark-results/v016/` (gitignored by design).

## 4. Environment and how to run

Host is macOS; Docker Desktop must be running. One Linux container per
invocation: 2 CPUs, 2048 MiB, `--memory-swap` equal to memory (no swap), 256
PIDs, real FUSE.

```bash
cd /Users/yifanxu/Ephemeral-AI-Lab/layerfs

# 1. product-free self-checks first (seconds, no Docker)
TMPDIR=/tmp rustc --edition=2021 -C opt-level=2 --test \
  benchmark/fs-bench-pro/workload/main.rs -o /tmp/wltest && /tmp/wltest
./target/release/fs-benchmark-pro workspace-self-check

# 2. build the host binary (~12 s incremental)
python3 benchmark/fs-bench-pro/shared/runner.py --build-host

# 3. build the Linux image (minutes; prints the tag on the LAST line)
python3 benchmark/fs-bench-pro/shared/runner.py --build-image

# 4. one selected performance invocation
IMG="layerfs-bench-infra:<tag>"
python3 benchmark/fs-bench-pro/shared/runner.py --family file_size_transition \
  --case v016-boundary-exact-v1 --seed 1 --image "$IMG" --perf-fast \
  --output benchmark-results/v016/perf-v016-boundary-exact-v1

# 5. its separate verification invocation. Take the two identities from the
#    performance receipt; --performance must point at the perf.jsonl.
python3 benchmark/fs-bench-pro/verify-selected.py --family file_size_transition \
  --case v016-boundary-exact-v1 --seed 1 \
  --source <source_identity> --input <input_identity> --image "$IMG" \
  --performance benchmark-results/v016/perf-v016-boundary-exact-v1/perf.jsonl \
  --output benchmark-results/v016/verify-v016-boundary-exact-v1
```

Helper for step 5 that avoids shell word-splitting bugs on the identities:

```bash
cat > /tmp/verify_one.py <<'PY'
import json, pathlib, subprocess, sys
IMAGE = sys.argv[1]
for c in sys.argv[2:]:
    perf = pathlib.Path(f"benchmark-results/v016/perf-{c}/perf.jsonl")
    ident = next(json.loads(l)["identities"] for l in perf.open()
                 if json.loads(l).get("kind") == "sample")
    out = f"benchmark-results/v016/verify-{c}"
    subprocess.run(["rm", "-rf", out])
    r = subprocess.run([sys.executable, "benchmark/fs-bench-pro/verify-selected.py",
        "--family", "file_size_transition", "--case", c, "--seed", "1",
        "--source", ident["source_identity"], "--input", ident["input_identity"],
        "--image", IMAGE, "--performance", str(perf), "--output", out],
        capture_output=True, text=True)
    p = pathlib.Path(out) / "verification.json"
    status = json.loads(p.read_text())["status"] if p.exists() else "NO-RECEIPT"
    print(f"{c}: {status} rc={r.returncode} {r.stderr.strip()[:200]}")
PY
```

**Measurement lock.** Every selected run takes
`$TMPDIR/layerfs-infra-measurement.lock` non-blocking and fails fast when it is
held. Never delete or steal that file, never run two selected invocations at
once, and never run a build while a sample is live. Serialize builds, perf
samples and verifications. If it is held, wait and retry — do not bypass it.

## 5. Four traps that will cost you hours

**Trap 1 — any source change invalidates every collected receipt.** `resolve_selection`
compares the host's compilation seal against the image label; a mismatch aborts
with `host and Linux image compilation seals differ`. So after *any* edit that
touches `benchmark/fs-bench-pro/src/**` or `families/**` you must rebuild the
image *and* re-collect every performance sample. Do not mix rows from different
seals in one table. Plan to batch edits, then collect once, rather than
collecting after each small fix.

**Trap 2 — the workload helper cannot see the SDK crates.** The Linux image
compiles `/usr/local/bin/fs-benchmark-workload` with a plain
`rustc workload/main.rs`. Anything under `workload/` or `families/` may not
reference `layerfs_sdk`, `layerfs_workspace`, `layerfs_content`,
`layerfs_layerstack_store`, or `rusqlite`. The previous agent lost two image
builds to `use of undeclared crate or module layerfs_workspace`. Keep
`WorkspaceFileRangeEdit`, `WorkspaceId`, `ObjectId` and all `Client` calls on
the host side (`src/workspace_bench.rs`); keep fixture/oracle code inside
`workload/`-visible modules only.

**Trap 3 — the generic verifier assumes the final state equals the initial
fixture, and assumes one inode class per declared path.** These are separate
bugs and both bit the boundary family:

- `registry::fixture` is the *initial* state. Any family whose declared final
  state differs (length transitions, alias splits, mixed edits) must route
  `expected` to its own oracle. `workspace_registry::expected` now does this for
  `file_size_transition`; do the same for every mixed family.
- `src/workspace_verify.rs:verify_root` derives reference counts from hard-link
  edges and requires one count per class for the whole snapshot. This cannot
  express an alias that separates from its target inside one schedule.
  `verify_root_split` now accepts one declared split class; that is a partial
  fix, not a general one.
- `fast-verify-v2` compares against the initial fixture. Families with a
  changed final state must be refused in the fast path. See the lists in
  `src/infra.rs` (`run_selected`) and `src/workspace_bench.rs` (`run_case`).

**Trap 4 — the one remaining known failure must be fixed by RCA, not by
loosening the check.** `v016-boundary-alias-roundtrip-v1` fails verification
with `canonical hard-link reference count: data/target.bin`. The observed edits
are `start=131071 del=0 len=1`, `start=131072 del=0 len=1`,
`start=131072 del=1 len=0`, `start=131071 del=1 len=0`,
`start=0 del=131071 len=131072`. Declared final state: target 131072 B,
surviving alias 131071 B, two inode classes.

To finish this one properly: add a dedicated alias oracle that records the two
classes' reference counts *before and after the replacement commit*, then
assert (a) both names hold one shared inode before, (b) two distinct inodes
after, (c) each class's reference count is what the recipe says. Do not delete
the reference-count check, do not special-case the path to zero, and do not
mark the row PASS because the lengths happen to match.

## 6. What still has to be built

Work in this order; each step unblocks the next.

1. **The v0.1.6 verify route** for families whose final state differs from the
   fixture, plus the split-class oracle from Trap 4. This is the prerequisite
   for every mixed family.
2. **Multi-live-workspace orchestration**: several concurrent host workers
   sharing one Store, one container, one resource budget, distinct mount roots
   (`/workspace/a`, `/workspace/b`, and `c`/`d` for the four-workspace
   extension), one active lease per branch. Keep one daemon and the live
   sessions alive across the schedule.
3. **The five-stage M1 workload** exactly as `workloads.md` specifies, including
   the per-cycle counters, deterministic target rotation, refresh cohorts, the
   discard/reopen event at local commit 5 for the concurrent cases, the
   historical Fork at trunk commit 5 for the branch cases, and the requirement
   that every requested Commit returns `Created`. Read the counter table in
   `workloads.md` §"Exact per-branch operation totals" and assert it.
4. **`dedup_branch_history` 6 additions** (large-hotset, namespace-inode,
   boundary-cycle, each K10/K100) — reuse the existing small-file history cases,
   do not duplicate them.
5. **`historical_access` 6 additions**: these consume *sealed producer
   artifacts* from step 3/4. Missing input is `NOT_READY`, never a
   constructed history and never a PASS. Seal the producer artifacts before any
   sample cleanup, and never relabel historical evidence.
6. **The three extended cases** with their declared caps.
7. **Seeds 2 and 3** for everything, then the matched control/candidate
   schedule if a genuinely comparable baseline exists. If it does not, say so
   and report unpaired rows instead of inventing a speedup.

## 7. Rules that are easy to get wrong

- **15 000 000 000 ns complete-command deadline** for each regular performance
  *and* each regular verification invocation, measured from entry, including
  authentication, independent copy, runtime readiness, all scheduled operations,
  receipt publication and cleanup. Plan the 12 s worker deadline plus a 3 s
  cleanup reserve. This is a complete-command gate; the inner `pure_call_sum_ns`
  is reported separately and never substitutes for it.
- **Performance and verification are separate invocations** with separate
  receipts. A fast perf run never implies a verification pass.
- **The path cap counts hardlink names, symlinks and temporary save files.**
  Charge a hardlink name its referent length; a symlink its target length.
  Report distinct-inode bytes separately.
- **Same-file SDK batches only.** `edit_workspace_file_ranges` rejects
  cross-file members. Cross-file pairs are two ordered single-file calls and
  must never be described as one atomic batch.
- **Finish every stage helper and close every writable handle before Commit.**
  One helper execution per POSIX stage via `Client::exec_workspace_session`; no
  per-file shells or Docker Exec.
- **One lease per branch.** A second same-branch lease is expected to fail; that
  is inherited correctness coverage, not an extra workspace.
- **Supported attributes are chmod, mtime and truncate.** Do not claim
  persistent xattrs, chown or ACLs. Verify inode relationships and link counts,
  never stable numeric inode values across remounts.
- **Perf receipts must prove zero added benchmark verifier/oracle/reopen/
  materialization/failure-injection work.** Reads intrinsic to the declared
  POSIX workload stay in and are named.
- **Container resource domains and host resource domains are reported
  separately.** The container quota does not constrain the host.
- **Record real concurrency overlap** from independent worker clocks. Two
  requests submitted at the same time is not evidence of parallel publication;
  if there is no observed overlap, report concurrency coverage failure rather
  than claiming parallelism. SQLite publication may legitimately serialize.
- **Contract corrections are prospective and versioned.** The roadmap is
  already committed. If you must change a frozen contract after observing
  results, give it a new scenario/schema identity and say so; never edit the
  committed roadmap retroactively and never rewrite retained evidence.

### Two known discrepancies you must resolve explicitly

1. `fixtures.md` states 62/312 directories excluding root. The implementation
   that satisfies the declared file counts, byte totals and 100-name cap needs
   53/303 data directories (63/313 excluding root). Either correct the document
   prospectively or make the layout match it — but state which, in the issue.
2. `benchmark-families.md` declares the roundtrip length sequence
   `131071→131072→131073→131072→131071`. That only holds if "remove 1" is a real
   removal. It now is; confirm and record it.

## 8. Evidence and reporting

Per run, retained under `benchmark-results/v016/performance/<case>/<seed>/<arm>/<run>/`
and `benchmark-results/v016/verification/<case>/<seed>/<arm>/<run>/`, each with
the raw JSONL/receipt, bounded failure logs, identity and manifest. Reuse the
runner's unique output directories; do not add a second collector.

For every case/seed/arm/mode report:

- exact configuration, requested vs observed paths/bytes, branch/workspace
  counts, local commits, total commits, ancestry;
- actual operation counters and observed concurrency overlap;
- complete command wall plus separate preparation, workload and cleanup;
- edit/Commit/fork timings in raw units, sample count, median, min–max, and
  maximum Commit latency;
- logical/physical bytes, Store growth, dedup/reuse, spool high water;
- host and container CPU/memory/I/O separately, with unavailable fields marked
  `null` and a reason — never fabricated zero;
- separate correctness, timing, resource, cleanup, custody and coverage
  statuses;
- source/product/harness/fixture/oracle/image seals and unique evidence paths.

Publish the final per-case tables and a machine-readable complete matrix to
#122. Keep every failure and every valid sample; append fixes and later results
without overwriting earlier evidence. If a genuine external blocker survives all
available work, post the exact failed or missing rows with actual numbers, the
root cause and the required action — and leave the issue open. Do not stop
early, and do not claim success while any required row is failing, timed out,
unsupported or unrun.

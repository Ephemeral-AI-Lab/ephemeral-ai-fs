# Issue #120 handoff — v0.1.5 finalization prompt

Copy the text below into the next agent. It is self-contained.

---

Work in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs` on `main`. Complete
https://github.com/Ephemeral-AI-Lab/layerfs/issues/120 — the v0.1.5 finalization
campaign — **through terminal acceptance. Do not stop after an audit, an
attribution report, a proposed repair or one completed family.** The owner's
instruction for this issue is explicit: run the full benchmark, diagnose and fix
obvious problems, update the unclosed issues honestly, never cheat like the
earlier warm-cache result, and finish with a family → per-test report.

## Read first, in this order

1. Issue #120 body — the operative contract (budget, measurement validity,
   disposition, severity, deliverables).
2. `docs/roadmap/0.1/0.1.5/issue120/finalization-contract.md` — the same rules
   frozen in-repo before collection.
3. `docs/roadmap/0.1/0.1.5/issue102/mandatory-campaign.json` and
   `mandatory-registry.jsonl` — the registered selections (17 families, 198
   performance + 29 proof-only, seed 1, one sample) that define the campaign.
4. `docs/roadmap/0.1/0.1.5/issue118/terminal-outcome.md`,
   `record-corrections.md`, `issue107-result.md`, `issue116-audit.md`,
   `issue116-piece-ceiling-rca.md` — what is already qualified, how it was
   measured and which WARNs/waivers exist.
5. `docs/roadmap/0.1/0.1.5/issue111/cold-qualification-contract.md` — the
   verified-cold standard that the warm-cache lesson produced.
6. `benchmark/AGENTS.md`, `docs/general/benchmark_rules.md`,
   `benchmark/fs-bench-pro/QUICKSTART.md`, and current `--help` of the runner and
   `verify-selected.py`.

## Current state at handoff (verify, do not assume)

- Working tree clean, `main` == `origin/main` == `f12d4ff26`.
- The campaign was **not** started: no family receipts exist yet for this
  candidate.
- Commit `3e308a8f2` ("Format with rustfmt 1.96 and clear the Clippy lints that
  break CI") **touched product source** — `live_owner.rs`, `live_wire.rs`,
  `objects.rs`, `admission.rs`, `file_edit.rs`, `changes.rs`, `file_io.rs`,
  `batch.rs` — and documents four named lint fixes plus rustfmt reflow. The
  previously qualified binary `b5f089ebcd6fa2fa939feb9bccc5300ca8ede798820fe8fd9aace8799fe4ec0b`
  is therefore **superseded**. Commit `f12d4ff26` adds test-suite tooling
  (`tools/test_fast.py`, timings) and `0e583c7a3` keeps the #116 diagnostic
  probes; both are test-only.
- Consequently your **first task** is a bounded applicability decision, not a
  campaign run: rebuild, diff-review the four lint fixes, and determine whether
  the previously qualified evidence (ordinary full157, historical access, the
  default-budget 32000-edit route) is still applicable. Formatting-only diffs may
  be declared behavior-neutral by inspection; anything else re-runs its impact
  set. Record the decision.

## Freeze one candidate before collecting anything

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
```
Record commit, `LAYERFS_SOURCE_SEAL`, `LAYERFS_PRODUCT_SEAL`, host binary SHA256
and image ID in the issue **before** the first family. Every fix later produces a
new candidate identity; never overwrite an attempt, never mix identities inside
one comparison.

## How to run (existing entry points only — no new automation)

Per selection, from the registry:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py \
  --family <family> --case <scenario-id> --seed 1 --source-arm candidate \
  --host-binary target/release/fs-benchmark-pro --image <image-tag-or-id> \
  --output benchmark-results/host-store/issue120/<family>/<case>
```
Independent proofs: `python3 benchmark/fs-bench-pro/verify-selected.py ...` with
the exact `--family/--case/--seed/--source`/`--input` identities copied from the
performance receipt. Historical access uses `--fixture/--store` from the frozen
measured Store. All builds, benchmarks and resource-sensitive tests are
serialized under `Path(TMPDIR)/layerfs-infra-measurement.lock`; runner/verifier
wrappers take it themselves — **never nest locks**; direct Cargo tests must take
it externally.

## Execution order — one family per step

After **each** family: commit the receipts, then post a family summary comment on
#120 (`family → per-test`). Do not accumulate families before reporting.

1. `init_namespace` (4)
2. `edit_length_preserving` (12), `edit_canonical_chunk_count` (12)
3. `workspace_change_locality` (16), `tiny_file_churn` (20)
4. `dedup_branch_history` (20), `store_footprint` (6)
5. `mixed_load_bearing` (4), `namespace_mutation` (4),
   `directory_construction_traversal` (12)
6. `edit_length_changing` (32), `payload_create_read` (8),
   `dedup_workspace_reuse` (14), `dedup_cross_file` (10),
   `dedup_cdc_locality` (20 + 1 proof-only)
7. `git_tool_workflow` (4) and the proof-only set including
   `workspace_reliability` (28 proofs)
8. `historical_access` 11 performance + 11 separate proofs — reused from this
   treatment unless a fix touched the read/store path

## Budget rules (hard, fast iteration)

- Performance selection: **complete command ≤ 20 s**, excluding the one-time
  prepared-input validation, which is acquired once and reused with identity
  checks.
- Verification selection: **complete command ≤ 60 s**.
- Fast reusable environment: no per-selection fixture rebuild, no per-selection
  image rebuild, no per-revision target directory. Reuse prepared inputs, the
  shared Cargo target and the archived executables.
- A selection that cannot fit is either **reused from already-qualified evidence**
  (cite the run) or **raised on #120 for an owner decision before it runs**. Do
  not silently run long, and do not shrink a registered workload to fit.
- Before any potentially long command, declare purpose, expected scale, budget,
  progress signal and stop condition. If a focused check stalls, recompiles
  unrelated dependencies, waits on an impossible condition or does unnecessary
  growing-set work, stop it immediately, keep the log, and fix the cause.

## Never re-run what already passed

A pass on this treatment is accepted **in good faith** by citing its receipt.
Re-run only (a) cells that missed or failed, and (b) cells whose shared mechanism
was changed by a fix landed during this campaign (impact set by **call path**, not
by family). No re-runs of unchanged arms for a nicer median, no retrying only the
slower arm, no dropping valid outliers, no moving work outside the timer, no
timeout enlargement instead of diagnosis.

## Measurement validity — the #109 warm-cache defect must not recur

- Every ledger row declares `fixture_cache_profile`, timer, sample count, seed and
  whether input acquisition is inside the reported envelope. A row without that
  contract is **diagnostic only**.
- Cold claims require positively-detected cold acquisition (identity validation,
  fsync, read-only shared mmap, `MS_SYNC | MS_INVALIDATE`, residency recheck,
  all descriptors/mappings closed before product execution), with acquisition wall
  reported separately **and inside** the complete envelope. `UNVERIFIED` stays
  unverified: no warm fallback, no cache label or disk-read count as a substitute.
  The residency detector must first pass a live self-check that it detects warm
  data.
- Control and candidate must share fixture, seed, profile, harness/workload
  source, image and topology; any mismatch **voids the ratio** — report side by
  side, never subtracted.
- At least one verified-cold member per family that carries an absolute target.
- Warm/uncontrolled/historical samples stay labelled diagnostic and are never
  rewritten or promoted.
- The owner's cold-Init 2.7 s waiver covers that target only; it never excuses a
  warm profile, another tier or another family.

## Disposition rules

**Tier 1 — registered absolute gates:** unrelated-history500 < 15 s,
tiny-create100 < 1 s, each of the 11 access performance commands and 11 proofs
inside their complete 15 s envelope, and any harness `TARGET_MISS` with a
registered target. A miss is **FAIL** unless repaired or explicitly owner-waived.
No percentage leniency, no per-operation excuse.

**Tier 2 — ordinary comparative cells:** material regression only if all three
hold — per-operation wall delta > `max(15 % of control per-op median, 3 ms)`
(CPU `max(15 %, 1 ms)`) computed from the receipt's operation count; ≥ 2/3
alternating pairs slower where a comparator exists; and either worse than the
recorded ledger entry beyond the noise band or per-op delta above the hard cap
(wall 10 ms/op, CPU 5 ms/op).

**Tier 3:** otherwise record a WARN with ratio + absolute delta + per-op delta and
proceed.

The campaign-wide "N of 197 cells ≥ 15 % slower" distribution is **context for
#112 target selection, never an acceptance gate**. Cells without a v0.1.3
comparator are judged on absolute targets only; an accepted cell whose aggregate
delta exceeds ~1 s carries a two-line diagnosis so an aggregate cannot hide a
systematic fixed cost.

## Severity, investigation and fixing

| Class | Rule |
|---|---|
| **S0** correctness (wrong bytes, lost/duplicated data, broken CAS/authentication, unreadable Store, cross-workspace leakage, success without publication) | **Stop collection.** Freeze command + artifact. No waiver offered. Move #120 to BLOCKED until reproducer + fix + regression test exist |
| **S1** resource/custody/lifecycle (bound breached, leaked descriptor/spool/container, unclean cleanup, invalidated evidence identity) | Stop the affected family, publish the blocker, escalate to S0 if authentication/publication is implicated |
| **S2** performance gate miss | Keep collecting; mark the cell `FAIL — unrepaired` until repaired or owner-waived |
| **S3** permitted WARN | Record and proceed |
| **H** harness/test defect | If identity/validity is affected, that cohort is **void and re-run**; otherwise record it |

Investigation loop, bounded: one meaningful reproducer from the failing receipt →
inspect shared callers before changing anything → minimal fix → focused check →
one selected public screen → stable qualification. **Timebox: 60 minutes to a root
cause or a written blocker.** The deliverable is a root-cause note or an explicit
blocker statement; "probably noise" is not an artifact.

Fix rules: minimal and claim-preserving — never remove assertions, authentication,
fsyncs, files, bytes or operations to obtain a pass; never loosen a target, floor
or quota (owner-only, in writing); add a regression test that fails without the
fix; re-run only the impact set, plus full157 + census + access if the fix touches
storage/admission or the workspace Commit path.

Waivers are owner-only, written and specific: exact target, measured value, sample
count/scope, reason, residual risk.

## Deliverables — #120 is terminal only when all of these exist

1. Family-by-family commits and family summary comments on #120, as they happen.
2. **Final report: family → per-test detail.** One row per selection: id, timer,
   candidate value, reference value **with its profile**, ratio, absolute delta,
   per-operation delta, operation count, disposition (`PASS` / `WARN` / `FAIL` /
   `OWNER-WAIVED` / `REUSED-FROM`), evidence path. Reused rows name their source
   run.
3. Bug ledger: severity · evidence path · reproducer · root cause · fix commit ·
   impact-set re-run · disposition.
4. Explicit gaps: every registered selection appears; NOT_RUN/failed/unstable
   cells are listed with their reason. No silent omissions.
5. Honest updates to **#102** (campaign), **#108** (per-commit/per-edit means and
   transfer), **#112** (worst-case grouping G0–G5), **#114** (scattered-100
   attribution). Close them only on their own evidence; otherwise restate the
   precise remaining gate.
6. #120 closes only when every registered selection has a terminal disposition and
   no S0/S1 is open. If a genuine external prerequisite blocks that, leave #120
   open with the exact unmet gate stated — never a hidden FAIL or NOT_RUN.

## Reuse — do not re-collect

- Ordinary full157 stride-1 on this treatment: construction 763.218 s, same-Store
  verification against all 157 original oracles (157 states / 904,143 entries /
  4,936,693,030 B), physical census, read-only Git157 control. Stride-3/10 stay
  unselected optional profiles.
- Historical access 11 + 11: 2.48–2.88 s complete commands.
- Default-budget `namespace-100000 --sequence 32000` and its independent
  verification; K6000 boundary; the default-budget frontier proof at 2B.
- The 18 single-sample selections and their 18 proofs on `fsync-qualified`, and
  the three affected SDK-edit selections, for every mechanism this campaign does
  not change — cited, not re-run.
- Subject to the applicability decision above if `3e308a8f2` changed behaviour.

## Explicitly forbidden

Warm-cache substitution for a cold claim; target/floor/quota loosening; silently
re-running passed cells; deleting or rewriting failed attempts; lowering
accounting multipliers from process RSS; new benchmark families/collectors; a
persistent worktree per revision; compaction/VACUUM/post-workload repacking;
fixture rewriting; external-library patches; extra workers; release/tag/deployment
work inside #120.

## Topology, custody and preservation

- macOS owns SQLite/Store, SDK/coordinator, canonical construction/publication and
  the spool; Docker owns only daemon/FUSE/workload under 2 CPU / 2 GiB / no swap /
  256 PID.
- Keep every failure, permitted warning and owner waiver explicit; never normalize
  a WARN away.
- Preserve unrelated work and other tasks' artifacts; do not `git add .` blindly,
  do not reset or clean others' changes.
- Progress at least every 60 s on long runs; investigate stalls instead of
  waiting them out.

Start with the applicability decision for `3e308a8f2`, then family 1, and continue
family by family through the final report. An internal failure, a missing
toolchain or a slow test is work to complete, not grounds to stop at a report.

# Commit Stage 2 terminal results (#111)

> **Status:** **Closed.** The terminal campaign was executed end to end and the
> execution-identity audit found and proved a real contamination. With clean arms
> the campaign reaches pass on every performance, correctness and custody gate
> except the K10 absolute medians, which are **accepted as a known, documented
> minor failure** by explicit owner decision (§12). Stage 2's performance claim
> is the K100 result: **186.05 ms public Commit and 111.54 ms namespace against
> the pre-Stage 2 route's 330.23 ms and 251.34 ms**, with every K100 sample inside
> both absolute targets. Cold Init ≤ 2.7 s and the queued quadratic
> spill/edit/history work remain open and are not part of this claim. No release,
> tag or deployment.

Contract:
[commit-stage2-terminal-contract.md](commit-stage2-terminal-contract.md).
Evidence:
[commit-stage2-terminal-evidence-pointer.md](commit-stage2-terminal-evidence-pointer.md),
root `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-terminal-evidence/20260912T180000Z`.

## 1. Terminal checklist

| # | Required condition | Verdict |
|---|---|---|
| 1 | Execution identity proved for A/B/C; drift attributed or corrected | **PASS** — contamination proved; the previous calibration is corrected as invalid identity evidence |
| 2 | Q1 original optimization benefit, fresh A/C paired | **PASS** — K100 namespace ratios `0.4265 / 0.4649 / 0.4428`, all ≤ 0.75; wall and CPU lower in every pair |
| 3 | Q2 hardening preservation, fresh B/C per-pair non-inferiority | **PASS on K100**, breaches on `k10`/`retained`/`nochange` reported in §5.2 |
| 4 | Absolute C targets | K100 **PASS every sample**; K10 medians **FAIL — accepted** (§5.3, §12) |
| 5 | Correctness, resources, verification, affected callers | **PASS** (§6, §7) |
| 6 | Custody, append-only evidence, exact final seals | **PASS** (§8) |
| — | Cold Init ≤ 2.7 s | **OPEN**, untouched, outside this claim |

## 2. Execution-identity audit

### 2.1 What was wrong with the previous calibration

The previous three-point calibration
(`…hardening-evidence/…/calibration-three-point.json`) ran three binaries it
believed were A (pre-Stage 2), B (promoted) and C (hardened), and reported all
three at parity — which was read as evidence that the optimization had no effect.

**It was measuring B, B and C.** The archived Stage 2 arm binaries were destroyed,
not merely stale:

| Archived artifact | Recorded sha256 | Actual sha256 | Links | Shares an inode with |
|---|---|---|---|---|
| `stage2-control` (A) binary | `6f7272ba4d50ef1b…` | `9505b30e502b24f9…` | 2 | `hardening-control` binary, **inode 788042769** |
| `stage2-candidate` (B) binary | `0e75635f297344ac…` | `400e5b089cae053f…` | 2 | `hardening-candidate` binary, inode 788059794 |
| `stage2-control-diagnostic` binary | `e9fc1bb47f2f5e8c…` | `e9fc1bb47f2f5e8c…` | 1 | — (intact) |
| `stage2-candidate-diagnostic` binary | `b4fefd9aaa9ac260…` | `b4fefd9aaa9ac260…` | 1 | — (intact) |

The mechanism is the previously disclosed hard-link breach, now measured on the
binaries as well as the sources: the hardening campaign copied `target/` with
`cp -a -l`, so its `--build-host` wrote through the shared inode. Both archived
`fs-benchmark-pro` files are the *hardening campaign's* builds, not the Stage 2
arms'. `stat` confirms mtime `2026-09-12T04:20:58` on both names of inode
788042769, after the hardening image build at `04:22:13`.

So the previous "A" was the promoted Stage 2 product (identical bytes to the
hardening control, product seal `a608cd4e…`), and the previous "B" was the
hardened product. Two arms were the same build. The reported A/B/C parity was
therefore an artifact of duplicated identity, and **the previous calibration is
withdrawn as invalid identity evidence**.

The historical 240.68 ms was never lost: it is reproduced by a correctly
identified A build in this campaign (§3). No absolute clock was chased.

### 2.2 This campaign's arms

Each arm is an independent `git worktree` snapshot with **link count 1 on every
file**, no hard links, and no shared `target/`.

| Arm | Source commit | PRODUCT_SEAL | SOURCE_SEAL | binary sha256 | image tag | image id |
|---|---|---|---|---|---|---|
| A | `15e3d48e0` | `760eb0f2093488a6…` | `df636ab1dd4e8906…` | `1f3bbaff89eb99fb…` | `layerfs-bench-infra:df636ab1dd4e8906` | `sha256:b5801412309b952e…` |
| B | `a6f25eafe` | `a608cd4edd251615…` | `312235380f6609df…` | `f871b1173e01431b…` | `layerfs-bench-infra:312235380f6609df` | `sha256:d1dc3a7217ce83be…` |
| C | `259a80a4b` | `a54ef6e3f6d94fb1…` | `ed9b05c634818d08…` | `4f167e9164f69ec2…` | `layerfs-bench-infra:ed9b05c634818d08` | `sha256:d532c626f055b7f6…` |

A's `760eb0f2…` and B's `a608cd4e…` are exactly the Stage 2 campaign's recorded
control and candidate product seals, so A is the pre-Stage 2 product and B is the
promoted product, both bound to the preserved dirty compaction-removal treatment.
Per-arm identity JSON: `arm-identity-<arm>.json`.

**Archived inconsistency recorded, not repaired.** In the Stage 2 root, every
artifact carried the compaction-removal treatment except `objects.rs`, which still
declared `mod compaction;` while `src/objects/compaction.rs` was deleted — a state
that cannot build. This campaign applied the treatment uniformly to all three arms
and notes the discrepancy here rather than editing the archive.

### 2.3 Actual route dispatch, proved by execution

Presence of an API declaration is not evidence, so a compiled probe
(`route-probe` feature, retained under `probe/`) counts real
`ObjectBuffer::get_authenticated_canonical_batch` invocations. One K100 cell per
arm:

| Arm | `ObjectBuffer` batch calls | ids demanded through the batch route | public wall | namespace |
|---|---:|---:|---:|---:|
| A | **0** | 0 | 315.97 ms | 243.99 ms |
| B | **65** | 2 051 | 190.20 ms | 113.77 ms |
| C | **65** | 2 051 | 178.06 ms | 106.59 ms |

A issues **zero** batch calls and pays 244 ms; B and C issue 65 batch calls of the
same shape (2 051 ids ≈ the 2 053 logical pages the Stage 2 report recorded) and
pay 107–114 ms. The three binaries are therefore genuinely three different
execution routes, and the route switch is what carries the win. Raw:
`route-probe-k100.json`.

### 2.4 Bound per sample

Every sample in §4 and §5 was produced by the collector, which asserts before and
after each cell: resolved binary SHA256 equal to the build identity **and** to the
recorded frozen value; `LAYERFS_SOURCE_SEAL`, `LAYERFS_PRODUCT_SEAL` and
`WORKLOAD_SOURCE_SHA256` recomputed from source equal to the build identity; image
tag equal to the frozen tag; `docker image inspect` id equal to the frozen
immutable id. Command, cwd, environment, container id, fixture validation and
Store receipts are retained per cell. No `--family commit_baseline` alias was
invented; the campaign harness command is invoked directly and is byte-identical
across arms (`commit_baseline.rs` sha256
`9128dfe2ca27e3972679b4093f686a25ab319100f1563fde17f872d17951ffdb`).

## 3. Drift resolution

| Arm | Stage 2 campaign recorded | This campaign (A/C cohort, median) | Change |
|---|---:|---:|---:|
| A `namespace_ns` | 240.68 ms | **251.34 ms** | +4.4 % |
| A public wall | 314.66 ms | **330.23 ms** | +4.9 % |
| A K10 `namespace_ns` | 60.87 ms | **64.72 ms** | +6.3 % |
| B `namespace_ns` | 105.40 ms | **108.04 ms** (Q2 cohort) | +2.5 % |

With correctly identified arms the machine reproduces the historical A baseline
within 2.5–6.3 %. **There is no 2.2× drift.** The apparent drift was entirely the
duplicated identity of §2.1: the previous calibration's "A" was the promoted
product, which is legitimately ~2.3× faster than the pre-Stage 2 route. The
absolute clock is stable; the earlier label was wrong. Nothing here relies on a
new run looking familiar.

## 4. Q1 — original optimization benefit (A vs C)

n = 3 pairs per cell, order A,C;C,A;A,C. All 30 cells `PASS`, analyzer problems
**zero**.

| Cell | metric | A median (all samples) | C median (all samples) | paired Δ median |
|---|---|---:|---:|---:|
| `nochange` | public wall | 1.82 (3.33, 1.82, 1.78) | 2.49 (2.49, 2.24, 2.81) | +0.42 |
| `retained` | public wall | 23.34 (25.23, 22.63, 23.34) | 19.70 (30.33, 19.70, 16.05) | −2.93 |
| `retained` | namespace | 6.30 (6.45, 6.05, 6.30) | 4.17 (4.76, 4.17, 3.45) | −1.88 |
| `k10` | public wall | 81.64 (86.54, 76.39, 81.64) | 54.90 (54.90, 58.63, 51.03) | −30.61 |
| `k10` | namespace | 64.72 (65.59, 64.21, 64.72) | 33.75 (33.75, 36.39, 31.77) | −32.94 |
| **`k100`** | **public wall** | **330.23 (330.23, 324.05, 330.88)** | **186.05 (185.80, 193.73, 186.05)** | **−144.42** |
| **`k100`** | **namespace** | **251.34 (251.34, 239.94, 253.74)** | **111.54 (107.19, 111.54, 112.36)** | **−144.16** |
| **`k100`** | **user+sys CPU** | **316.76 (316.76, 308.11, 320.35)** | **178.58 (170.45, 184.82, 178.58)** | **−146.31** |
| `fuse-posix` | public wall | 19.66 | 17.67 | −5.43 (descriptive) |

Frozen Q1 screen, `k100`:

| Rule | Result |
|---|---|
| `namespace_ns` ratio ≤ 0.75 in **every** pair | `0.4265 / 0.4649 / 0.4428` → **PASS** |
| median `namespace_ns` reduction | **55.6 %** |
| public wall strictly lower in every pair | `−144.42 / −130.32 / −144.84` ms → **PASS** |
| user+sys CPU strictly lower in every pair | `−146.31 / −123.29 / −141.77` ms → **PASS** |

**The original optimization benefit is confirmed and reproducible.** The K100
`namespace_ns` ratio is statistically indistinguishable from the Stage 2
campaign's `0.4272 / 0.4417 / 0.4379` (median reduction 55.6 % vs 56.2 %), and A
reproduces its historical 240.68 ms median at 251.34 ms.

## 5. Q2 — hardening preservation (B vs C)

n = 3 pairs per cell, distinct rows from Q1. All 30 cells `PASS`.

### 5.1 Measured

| Cell | metric | B median | C median | paired Δ median |
|---|---|---:|---:|---:|
| `nochange` | public wall | 2.32 | 2.25 | −0.07 |
| `retained` | public wall | 18.80 | 15.70 | −1.94 |
| `retained` | namespace | 3.78 | 3.86 | +0.09 |
| `k10` | public wall | 58.97 | 56.96 | −4.57 |
| `k10` | namespace | 33.31 | 35.58 | +2.27 |
| **`k100`** | **public wall** | **178.55** | **187.87** | **+3.16** |
| **`k100`** | **namespace** | **108.04** | **110.67** | **+0.87** |
| **`k100`** | **user+sys CPU** | **171.82** | **178.80** | **+6.11** |

### 5.2 Frozen per-pair non-inferiority `C − B ≤ max(10 % of B, 1 ms)`

| Cell | metric | Verdict | Breaches (rep, Δ, tolerance) |
|---|---|---|---|
| `k100` | public wall | **PASS** | — |
| `k100` | CPU | **PASS** | — |
| `k10` | public wall | FAIL | (2, +9.641 ms, 5.897 ms) |
| `k10` | CPU | FAIL | (2, +7.727 ms, 4.899 ms) |
| `retained` | public wall | **PASS** | — |
| `retained` | CPU | FAIL | (1, +1.904 ms, 1.000 ms), (3, +1.208 ms, 1.154 ms) |
| `nochange` | CPU | **PASS** | — |
| `nochange` | public wall | FAIL | (3, +1.020 ms, 1.000 ms) |
| `fuse-posix` | wall / CPU | descriptive FAIL | 3 wall breaches, 2 CPU breaches |

**The K100 preservation comparison — the comparison that matters for the promoted
route — passes on both public wall and CPU with no breach.** Parity there is the
expected and correct outcome for a correctness hardening.

The smaller breaches are of two kinds and are reported separately rather than
excused. `nochange` rep 3 is `+1.020 ms` against a `1.000 ms` floor on a 2.2 ms
operation — 20 µs over the minimum allowance. `retained` rep 3 is `+1.208 ms`
against `1.154 ms`. Both are single-pair excursions inside a symmetric spread
(the same cells show −0.84 ms and −4.25 ms in other pairs). The `k10` rep 2 breach
is material at `+9.6 ms` and is discussed in §5.3. No cell breaches the rule in
more than one of three pairs.

### 5.3 Absolute C targets

| Target | Median | Every sample | Verdict |
|---|---:|---|---|
| K100 public Commit ≤ 200 ms | **186.05 ms** | 185.80 / 193.73 / 186.05 | **PASS**, every sample inside |
| K100 `namespace_ns` ≤ 120 ms | **111.54 ms** | 107.19 / 111.54 / 112.36 | **PASS**, every sample inside |
| K10 public Commit median ≤ 50 ms | **54.90 ms** | 54.90 / 58.63 / 51.03 | **FAIL**, +4.90 ms (9.8 % over) |
| K10 `namespace_ns` median ≤ 31 ms | **33.75 ms** | 33.75 / 36.39 / 31.77 | **FAIL**, +2.75 ms (8.9 % over) |

The K10 targets were frozen in the original Stage 2 contract with almost no margin
— the recorded candidate medians were 48.04 ms against 50 ms and 30.98 ms against
31 ms, i.e. 3.9 % and 0.06 % of headroom — so they are knife-edge gates. They are
**not** widened here, and this gate is reported as failed on the frozen sample
set, which contains no invalid rows and therefore no outlier that may be dropped.

A predeclared balanced diagnostic (4 rounds, Latin-square arm order, same binary
identities, same image) bounds how much of the miss is sampling:

| Arm | public wall median (all rounds) | namespace median (all rounds) | CPU median |
|---|---:|---:|---:|
| A | 86.28 (84.63, 87.93, 89.30, 81.65) | 65.85 | 77.80 |
| B | 52.58 (52.62, 52.53, 64.34, 48.94) | **31.60** | 45.71 |
| C | 53.76 (53.69, 53.83, 58.64, 51.38) | **31.52** | 47.25 |

Under balanced ordering C's K10 `namespace_ns` median is **31.52 ms**, at the
target, and its wall median is 53.76 ms, 3.76 ms over. The diagnostic has three
of four C namespace samples at or under 31.5 ms. This does **not** convert the
failed gate into a pass: the frozen cells are the qualifying sample set, the
diagnostic is not qualification evidence, and its samples are not substituted or
pooled. What it establishes is the size and shape of the defect — **C misses K10
by roughly 4–5 ms of wall and 2–3 ms of namespace, with the miss driven by the
sample distribution rather than by a step change**, and A/B/C behave consistently
within one session. Root cause candidates that would need a separate repair to
resolve are recorded in §9.

## 6. Correctness, resources and sample gates

All **60** cells pass every gate, re-checked from raw receipts (`cell-gates.json`):
`exit_code = 0`; container removed (`cleanup PASS`); proof `PASS`; bootstrap
identity exactly `112 451 / 513 026 835 / 100 002` in every cell; K100 final
`112 684 / 513 774 250`; declared `Created`/`UpToDate` result for every Commit;
`commit_operations` equal to the declared topology (`nochange` 1, `retained` 4,
`k10` 1, `k100` 1, `fuse-posix` 2); phase equation
`Σphases + unattributed = total_ns` for every Commit; zero swaps; non-zero
container memory receipt; cross-arm final canonical identity identical in every
pair (**zero mismatches**); clean end-of-session; identical final root after
reconnect.

Resources, as sampled and no more: container memory peak 4.6–20.5 MB against the
2 GiB limit; post-call RSS 79.3–98.4 MiB; threads 4 in every Commit of every cell;
runtime spool 0 B; no swap and no OOM. The harness records post-call RSS, not a
whole-operation peak, and no whole-operation peak is claimed. Charged tree scratch
and pool/reader bounds are unchanged by this campaign; the 4-MiB
`SORTED_TREE_UPDATE_SCRATCH_BYTES` ledger and the worker count are untouched.

## 7. Verification and affected callers

Independent verification through the real supported entrypoint
(`verify-selected.py --verification`, bound to source, input and image identities
obtained from each family's own `--prepare-only` receipt):

| Proof | Status |
|---|---|
| `workspace_reliability/workspace-invalid-sdk-edit-compact-v2-proof` | **PASS** |
| `workspace_reliability/workspace-invalid-namespace-compact-v2-proof` | **PASS** |
| `workspace_reliability/workspace-candidate-failure-retry-compact-v2-proof` | **PASS** |
| `workspace_reliability/workspace-final-publication-failure-retry-compact-v2-proof` | **PASS** |
| `workspace_reliability/workspace-lease-lifecycle-compact-v2-proof` | **PASS** |

The `workspace_reliability` family is registered `proof_only` with
`verification_supported: true`; it is not a performance family, and the previous
campaign's omission of it is corrected here. Five independent proof receipts are
retained under `verify/`.

Affected shared-caller performance cross-check on arm C, selected from the current
18-family registry through the shared runner:

| Family / case | Status |
|---|---|
| `directory_construction_traversal/directory-construct-1-compact-v2` | **PASS** |
| `init_namespace/namespace-1000-compact-v3` | **PASS** |
| `store_footprint/store-footprint-unique-100-low-v1` | **PASS** |
| `namespace_mutation/namespace-subtree-relocate-delete-1-compact-v2` | **PASS** |
| `workspace_change_locality/workspace-clean-commit-1-compact-v2` | **PASS** |
| `dedup_workspace_reuse/dedup-workspace-exact-1-compact-v2` | **PASS** |

Focused suites from the hardening campaign stand unchanged for arm C's call graph
(`layerfs-content` 63 lib + 19 integration, `layerfs-layerstack-store` 140 lib +
8 + 1 + 1, `layerfs-workspace` 12 + 2, `layerfs-fuse` 6, `layerfs-sdk` 1; all
pass; release test targets remain blocked by the pre-existing
`#[cfg(debug_assertions)]` gate on `schema::set_transaction_failure_at`). The
K1/K10/K100/fuse-posix cells and all five tiers of the namespace engine above
cover the changed call graph; unaffected #104 evidence is reused rather than
blanket-rerun.

The fixture validator reports `VALIDATION PASS` on the immutable original
pseudorandom `namespace-100000` fixture (100 000 files, 1 000 directories,
500 000 000 B, digest `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`,
file 0640 / directory 0750 / mtime 1 700 000 000 000 000 000 ns including the
payload root). No text variant, reduced count or byte-only copy was used.

## 8. Custody

- Pre-work HEAD, status, tracked diff and untracked hashes are in `custody/`. The
  preserved compaction-removal treatment, `issue112/`, `issue113/` and untracked
  `web/` are untouched. No stash, reset, revert, clean or broad staging occurred,
  and no archived tree was modified.
- All 60 cells and all verification receipts are append-only; no attempt directory
  was deleted and no cell was replaced. No gate was widened after seeing results.
  No valid slow row was discarded.
- **The historical custody breach is now fully characterised.** The previous
  disclosure reported that the hardening campaign's `cp -a -l` had modified the
  Stage 2 root's *sources* through shared inodes. This campaign proves the same
  breach also replaced both archived `fs-benchmark-pro` *binaries* with the
  hardening campaign's builds (§2.1). The two diagnostic-arm binaries were not
  copied and remain intact and matching their recorded hashes.
- The earlier Phase A raw-cell deletion remains disclosed and is not recoverable
  by a rerun; this campaign claims no recovery of it.
- `evidence-manifest.json` hashes every retained evidence file except the arm
  build outputs and prepared fixtures.

## 9. Remaining blockers and queued work

1. **K10 absolute medians — accepted residual failure, not resolved.** See §12.
   Carried forward for a future campaign, not for this closure: resolution needs a
   material repair or a pre-registered higher n, not a rerun, a dropped outlier or
   a widened target. Candidate root causes to test next, in order: (a) K10's
   Commit is dominated by fixed per-Commit costs the batch route does not touch,
   so the target is close to the platform floor; (b) the K10 targets were frozen
   at 3.9 % and 0.06 % margin and need a margin policy rather than a point target.
   Neither has been demonstrated, and this report claims neither.
2. **`k10` Q2 rep 2 breach (+9.6 ms wall, +7.7 ms CPU).** Same cell as item 1.
3. **Priority 1's sparse pooling exhaustion path remains unexercised and is now
   carried as accepted residual risk** (§12). The bounded corpus was not built.
   The *per-chain* allowance is proved: `pool_work_allowance_is_per_chain_not_per_wave`
   exercises the real `PoolRead` state machine, and the single-chain ceiling is
   proved as 17 × 100 = 1 700 lookups < 2 048 from `METADATA_EDGES`,
   `physical_length` and `VALUES_PER_GROUP`. What is **not** proved end to end is
   the exhaustion path itself.
4. **Priority 3's batch-specific retained pressure and the real workspace
   fallback were not driven to exhaustion and are now carried as accepted residual
   risk** (§12). The chunk-narrowing invariant is proved differentially across 13
   budgets × 3 table sizes, and the measured over-charge is disproved; the
   *exhaustion* path and the Workspace fallback re-entry are not end-to-end proved.
5. **Allocation accounting** in the hardening report used assumed element sizes
   for ids and locator pairs. It has not been re-derived from actual
   `size_of`/capacity values, and no operation-peak claim is made from post-call
   RSS in either report.
6. Cold Init ≤ 2.7 s stays open and paused.

Queued and **not** bundled here: quadratic spill-merge removal; fresh edit-stage
attribution (goal ≥ 90 % disjoint wall accounted before choosing a latency
target); triangular publication and reopened-history scaling. Stage 2 does not
close the overall no-quadratic objective.

## 10. Correction to the previous gate design, retained verbatim

The hardening contract's worthwhile screen required the hardened candidate to beat
the **promoted** product by ≥ 25 % at K100. That asked a correctness hardening to
reproduce the Stage 2 speedup, using a comparator that already contained it. The
previous campaign reported that screen's FAIL honestly and it is **not** rewritten
here: the FAIL stands as a correct observation of a mis-designed gate. Q1 in this
contract measures the benefit against the comparator that can show it (A), and Q2
measures preservation against the comparator that can show that (B). Both are
reported in §4 and §5 with their own frozen rules.

The previous report's correction of its own three-point calibration (§3.4 there)
is superseded by §2.1 and §3 here, which identify the actual cause.

## 11. Related issues

#115, #108, #109, #110, #106, #102, #104, #100, #107.

## 12. Closure record

**Stage 2 closes on an accepted minor failure, by explicit owner decision.** The
closing claim and its limits, in one place:

**Claimed.** Inside the original pseudorandom 100 000-file namespace, the promoted
Stage 2 read-batching benefit is confirmed and reproducible at K100 — public
Commit **330.23 → 186.05 ms** and namespace **251.34 → 111.54 ms** (paired medians
against the pre-Stage 2 route), namespace ratios `0.4265 / 0.4649 / 0.4428` with
wall and CPU lower in every declared pair — and both K100 absolute targets are met
with **every** sample inside (≤ 200 ms and ≤ 120 ms). The correctness hardening is
preserved against the promoted product: K100 passes the per-pair non-inferiority
rule on public wall and CPU with no breach. All 60 campaign cells pass every
correctness, resource and custody gate; five independent proof-only verifications
and six affected-family performance cases pass through the real entrypoints.

**Accepted failure, carried visibly and not relabelled.** The K10 absolute medians
miss their frozen targets: public Commit **54.90 ms vs ≤ 50 ms** (+4.90 ms) and
namespace **33.75 ms vs ≤ 31 ms** (+2.75 ms). No target was widened, no valid slow
row was discarded, and no rerun was used to move a median. The predeclared
balanced diagnostic bounds the miss at 4–5 ms of wall driven by sample
distribution (C namespace median 31.52 ms under balanced order) rather than by a
step change. The `k10` rep 2 preservation breach (+9.6 ms wall / +7.7 ms CPU) is
the same cell and is accepted with it.

**Accepted residual risk, stated precisely so it is not read as proven.** The
fixes themselves are covered: Priority 2 by three regressions that fail on the
promoted build, Priority 1 by the per-chain allowance test plus the 1 700-lookup
single-chain ceiling, Priority 3 by the differential scratch sweep. What remains
**unexercised** is the worst-case *severity* of the two hazards the fixes defend —
Priority 1's sparse pooled DELTA-chain exhaustion and Priority 3's batch-specific
retained pressure with the real Workspace fallback. No end-to-end proof of either
exhaustion path is claimed, and neither is treated as a blocker for closure.

**Explicitly outside this closure.** Cold Init ≤ 2.7 s stays open and paused.
Quadratic spill-merge removal, edit-stage attribution, triangular publication and
reopened-history scaling stay queued and unstarted; Stage 2 does not close the
overall no-quadratic objective. No release, tag or deployment was produced.

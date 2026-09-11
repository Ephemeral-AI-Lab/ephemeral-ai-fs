# Commit Stage 2 hardening: findings, fixes and qualification (#111)

> **Status:** Executed end to end. All three review findings are addressed with
> focused fixes and regression tests; the paired hardening campaign is complete
> and gate-checked. **Correctness is qualified. The K100 absolute targets are
> met. The hardening is NOT qualified for the performance-preservation gate:**
> a five-pair measurement of the K100 cell shows the hardened candidate at parity
> with the promoted product (`namespace_ns` ratios `0.9968–1.0360`), so the
> Stage 2 reduction no longer reproduces and the frozen worthwhile screen fails.
> Cold Init ≤ 2.7 s remains **open** and paused. No release, tag or deployment.

Contract:
[commit-stage2-hardening-contract.md](commit-stage2-hardening-contract.md).
Evidence:
[commit-stage2-hardening-evidence-pointer.md](commit-stage2-hardening-evidence-pointer.md),
root `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-hardening-evidence/20260912T120000Z`.

## 1. Exact identities

| Item | Value |
|---|---|
| Main HEAD at freeze | `6f7a8b8d54e9855b42f0572555252c2420e2da2d` |
| Contract + pointer commit | `a6f25eafe` |
| Promoted Stage 2 product commit | `bd9dc1600` |
| Promoted product seal (control) | `a608cd4edd25161584986b0f2885d2497a0c231a63a0b3dc7be73685bd0c0b38` |
| Promoted source seal (control) | `4c46c963298e1eb5d4dacad062d8fcf886ef27ae7d90c5db910920f92a7e7ce5` |
| Control binary sha256 | `3b46be77cb8386c4ef888d08c7efdf2749793f258786e67b28a1de8a6d17a16e` |
| Control image / immutable id | `layerfs-bench-infra:4c46c963298e1eb5` / `sha256:9f9ad1488be9876ed83b9fcbdfc5bc169276d9110d264bc267fc95c163e661e7` |
| Hardened candidate product seal | `a54ef6e3f6d94fb10eb131bbdabec52ae9b903572a42f1b685f7c2d1e0ee3be8` |
| Hardened candidate source seal | `2fe0b96b73c06682224fbd3152fb6f67490dc8402b0dfb7a6bf6912b7f56bcff` |
| Candidate binary sha256 | `400e5b089cae053f76a472b894a75338cdaa3579fe27b976aa5995463a8f145a` |
| Candidate image / immutable id | `layerfs-bench-infra:2fe0b96b73c06682` / `sha256:4832be19f563cc4496b85873ced34c83a89a815013a1aff34f15c9b0ab4bfab9` |

**The promoted seal is verified, not assumed.** `stage2-candidate` (the Stage 2
base arm, product seal `760eb0f2…`) plus `candidate.patch` (sha256
`ffbe1fd1a793ab4c47901cd46cebb3a766b706e67c15e2534937f0974df2ea53`) reproduces
the promoted source and product seal `a608cd4e…` byte for byte. That
reconstruction is the control arm of every pair. The clean committed tree alone
omits the preserved uncommitted compaction-removal treatment, so it was never
used as an arm.

## 2. Findings, reproductions and fixes

### 2.1 Metadata cache lifetime versus work-budget lifetime (Priority 1)

**Finding as reviewed.** `visit_wave` serves one metadata record group with a
single `PoolRead`, so the per-chain physical work allowance
(`decoded_work`, 16 KiB per value-group miss, ceiling 32 MiB) was shared by
unrelated targets in the wave.

**Reproduced?** Not by measurement, and the review recipe is disproved by actual
constraints. A valid corpus that spent the ceiling would need about 3 400 distinct
value groups (roughly 560 000 admitted metadata values): the 512-KiB retention
bound clears the value cache after about 43 groups (`retained + 165×73 + 256 >
512 KiB`), so every later lookup is charged again, and the per-chain ceilings
(`METADATA_EDGES + 1 = 17` nodes × at most 100 rows via `physical_length`) cap a
chain at 1 700 lookups. Two disjoint chains would be needed for 3 400, which is
not constructible within this campaign's fixture budget. **Reported as an
analytically bounded hazard, not as a reproduced regression.** The bound is
recorded in a test rather than asserted in prose.

**Fix (smallest bounded form).** The reusable decoded value cache stays shared
across the wave; the work allowance becomes the property of one
`metadata_chain` and is reset by `PoolRead::begin_chain` when a chain starts. No
capacity, retention bound, eviction rule, cache type or worker changed. The 32-MiB
ceiling was **not** raised, and the 192-KiB per-`expand` logical-work guard is
unchanged.

**Bound documented.** Total wave work is bounded by the fixed request count
(`OBJECT_PAGE_COUNT = 128`) times the single-chain ceiling of 1 700 lookups —
217 600 lookups × 16 KiB worst case — while each chain independently sees at most
1 700. One chain cannot reach the ceiling anyway: 1 700 × 16 KiB = 26.6 MiB <
32 MiB.

**The moved 192-KiB `logical_work` counter is not a regression.** 17 nodes ×
100 rows × 94 B = 159 800 B < 192 KiB, further limited by the 128-KiB canonical
closure. Recorded as tests (`one_chain_cannot_exceed_the_decoded_work_ceiling`,
`group_lookup_spacing_controls_the_miss_count`,
`pool_work_allowance_is_per_chain_not_per_wave`); the guard was not removed.

### 2.2 Batch API ordering and entry bounds (Priority 2)

**Finding as reviewed.** `ObjectBuffer`'s batch override emitted owned objects
immediately and fetched missing ones afterwards, so `[sourceA, ownedB]` could
callback `[B, A]`; and an all-owned batch never reached `CoreReader`, so it
bypassed the `OBJECT_PAGE_COUNT` page bound.

**Reproduced.** Three focused tests fail on the control arm and pass on the
hardened arm:

| Test | Control | Hardened |
|---|---|---|
| `owned_objects_do_not_overtake_earlier_source_demands_in_a_batch` | FAIL (`[A]` delivered, then `MissingObject` for the reordered demand) | PASS |
| `oversized_all_owned_batch_is_rejected_before_any_callback` (129 owned ids) | FAIL (no error, callbacks ran) | PASS |
| `mixed_batch_reports_callback_and_identity_failures_after_the_source_call` | FAIL (ordering) | PASS |

**Fix.** The ceiling is enforced at entry, before the source call and before any
callback. One pass records which demands the buffer owns and which must be
fetched; the single bounded source call runs for the unowned subset only; a second
pass answers every demand in its place. When the request is wholly unowned — the
sorted-tree case — the source call streams straight to the callback exactly as the
promoted route did, so the common path keeps its original allocation profile.
Retained state is one position per owned demand plus one fetched copy per unowned
demand, bounded by the request; no unbounded result map was introduced.
Duplicates are answered once per demand, empty batches perform no callback, and
missing/identity/callback errors keep their existing semantics.

### 2.3 Deep-tree fallback and allocation accounting (Priority 3)

**Finding as reviewed.** `batch_children` reserved worst-case retained bytes for up
to 32 siblings, and the lease survived recursive descent; the Workspace
individual-mutation fallback re-enters the same compact batch engine.

**Reproduced?** The over-charge is **disproved for every measured table shape**,
and the "one-byte root-read error" framing does not establish the fallback. An
instrumented sweep of point-route and batch-route peaks (`N = 1 000 / 2 600 /
13 000`, budgets from 1 B to 4 MiB) shows **identical peaks on both routes** — for
a single-key update at `N = 13 000`, both must hold exactly 316 264 B, and each
fails below it. The batch route reserves the chunk's worst case but immediately
releases it to the bytes actually retained, so it does not demand more than the
point route at the same point in the walk. No valid tree was found in which the
batch route fails where a point read survives.

**Fix as a defensive invariant.** `batch_children` now narrows the chunk to what
the remaining ledger can actually hold, down to a one-child chunk read under the
same per-page ceiling the point route enforces, instead of failing on the
full-width worst-case charge. `TREE_BATCH_CHILDREN = 32` is unchanged for normal
cases, no configurable tuning was added, and the 4-MiB ledger is unchanged.
Regression test
`stage2_reduced_scratch_never_fails_a_batch_that_a_point_read_survives` sweeps 13
budgets × 3 table sizes and records the measured finding above in its doc
comment. The same test passes on the control arm, which is reported here rather
than presented as a control-failing regression.

**Allocation accounting.** The chunk charge is payload bytes only. Per chunk, ids
and fetched vectors add `entries × 8` (id) plus `entries × 24` (locator pair) plus
one 8-KiB-bounded payload per child; the tree scratch peak recorded for the K100
campaign cell is unchanged at 449 492 B against the unchanged
`SORTED_TREE_UPDATE_SCRATCH_BYTES` 4-MiB ceiling. Source-owned simultaneous
allocations (store decode buffers, pool group vectors) live under the existing
reader bounds and the 128-object page bound, not under the tree ledger. The
449 492 B figure and post-call RSS are **not** whole-operation peaks; no
independently sampled whole-operation peak is claimed.

## 3. Paired plain campaign

Frozen protocol: `nochange`, `retained` (K1), `k10`, `k100`, `fuse-posix`;
n = 3 pairs per cell, order C1,T1;T2,C2;C3,T3; one fresh Store and container per
cell. 30 cells, all `PASS`, analyzer problems **zero**, every gate re-checked
from raw evidence (bootstrap `112 451 / 513 026 835 / 100 002`, K100 final
`112 684 / 513 774 250`, 10 unchanged sampled files per cell, phase equation per
Commit, zero swaps, non-zero container memory receipt, clean cleanup, identical
reconnected root, cross-arm final identity equal in every pair).

Milliseconds; median with all three samples. `d` is the paired difference.

| Cell | metric | control | candidate | d median |
|---|---|---:|---:|---:|
| `nochange` | public wall | 2.355 `[2.355, 2.425, 2.216]` | 1.836 `[2.285, 1.836, 1.785]` | −0.431 |
| `retained` | public wall | 16.683 `[15.109, 21.456, 16.683]` | 20.297 `[11.686, 20.297, 23.639]` | −1.158 |
| `retained` | namespace | 3.863 | 3.529 | −0.333 |
| `k10` | public wall | 51.877 `[51.877, 51.229, 52.317]` | 51.354 `[48.615, 51.354, 53.123]` | +0.125 |
| `k10` | namespace | 31.279 | 34.109 | +2.837 |
| `k100` | public wall | 195.944 `[194.682, 197.483, 195.944]` | 184.157 `[184.157, 193.821, 180.660]` | −10.525 |
| `k100` | namespace | 121.080 | 110.046 | −5.816 |
| `k100` | user+sys CPU | 182.236 | 176.228 | −5.976 |
| `fuse-posix` | public wall | 16.054 | 15.654 | +2.425 (descriptive) |

### 3.1 Absolute targets (declared before measurement)

| Target | Median | Every sample | Verdict |
|---|---:|---|---|
| K100 public Commit ≤ 200 ms | 184.157 | 184.157 / 193.821 / 180.660 | **PASS** |
| K100 `namespace_ns` ≤ 120 ms | 110.046 | 110.046 / 115.264 / 106.935 | **PASS** |
| K10 public Commit median ≤ 50 ms | 51.354 | 48.615 / 51.354 / 53.123 | **FAIL** (median) |
| K10 `namespace_ns` median ≤ 31 ms | 34.109 | 29.977 / 34.116 / 34.109 | **FAIL** (median) |

The K10 rows were frozen as median targets and are reported as such; two of three
K10 samples exceed 50 ms and two exceed 31 ms.

### 3.2 Per-pair non-inferiority `max(10 % of control, 1 ms)`

| Cell | CPU | Public wall |
|---|---|---|
| `k100` | PASS, no breach | PASS, no breach |
| `k10` | **BREACH** rep 3: +5.340 ms vs +4.306 ms tolerance | PASS |
| `retained` | **BREACH** rep 3: +3.016 ms vs +1.037 ms | **BREACH** rep 3: +6.956 ms vs +1.668 ms |
| `nochange` | PASS | PASS |
| `fuse-posix` (descriptive) | BREACH rep 2 +4.554 ms, rep 3 +3.614 ms | BREACH rep 2 +2.425 ms, rep 3 +4.341 ms |

### 3.3 Five-pair K100 power measurement (decisive)

Because the three-pair campaign's control drifted (K100 `namespace_ns` median
121.08 ms here against 105.39 ms in the two earlier measurement sessions), the
K100 cell was re-measured with five pairs under the frozen rule continued, in one
sequence: C1,T1;T2,C2;C3,T3;C4,T4;T5,C5.

| rep | control wall | candidate wall | d wall | control ns | candidate ns | d ns | ns ratio | d CPU |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 166.43 | 170.83 | +4.40 | 102.50 | 104.15 | +1.66 | 1.0162 | +1.67 |
| 2 | 173.50 | 174.80 | +1.30 | 105.09 | 104.75 | −0.34 | 0.9968 | +6.37 |
| 3 | 184.85 | 182.65 | −2.20 | 105.92 | 108.77 | +2.85 | 1.0270 | +1.41 |
| 4 | 177.19 | 173.16 | −4.03 | 105.27 | 105.36 | +0.10 | 1.0009 | −7.83 |
| 5 | 173.15 | 173.09 | −0.05 | 102.69 | 106.39 | +3.70 | 1.0360 | +1.21 |
| **median** | **173.50** | **173.16** | **−0.34** | **105.09** | **105.36** | **+0.27** | — | **+1.21** |

The hardened candidate is **at parity** with the promoted optimized product on
this cell: median public wall −0.20 %, median namespace +0.26 %, median CPU
+0.74 %. The absolute K100 targets still pass in this cohort (≤ 200 ms and
≤ 120 ms in every sample), but the frozen worthwhile screen fails:
`k100_namespace_at_least_25_percent` is false, no pair is below 0.75, and
`k100_namespace_reduction` is −0.26 % rather than the Stage 2 measurement's
56.2 %.

**Interpretation, stated within the evidence.** The Stage 2 campaign measured the
unhardened candidate at 105.40 ms namespace against a 240.68 ms control; this
measurement has the promoted control at 105.09 ms and the hardened candidate at
105.36 ms. The measurement therefore **cannot demonstrate that the Stage 2 win
survives the hardening**. Whether the treatment lost the win or the win is
conditional on a load state this campaign could not reach is **not resolvable
from this evidence**. Three runs of the same frozen protocol in this root bracket
the effect: the first (superseded treatment, before the Priority 2 rewrite)
measured the candidate **slower** by 10.4–13.4 % on namespace; the retained
three-pair campaign measured a 9.1 % median reduction; the five-pair measurement
found parity. The spread of those three results is larger than any one of them,
so what is established is that **no reproducible K100 reduction was demonstrated
for the hardened build under the frozen protocol**. The first run is retained
unmodified under `attempts/run1-paired-plain-superseded-treatment` with its own
`collect`/`summary`/`analysis` artefacts.

### 3.4 Three-point calibration: no measurable improvement, and a 2.3× baseline drift

**Post-hoc diagnostic, not qualification evidence.** The five-pair result above
raised the question of how much of the original win survives at all, so the
pre-Stage 2 control was measured directly against the promoted product and the
hardened build in one interleaved session, three repetitions each, same image
topology and workload. The pre-Stage 2 arm is the archived `stage2-control` tree,
whose worktree contains no `get_authenticated_canonical_batch` and is therefore
the pre-Stage 2 source.

| Arm | public wall median (all samples) | `namespace_ns` median (all samples) | CPU median |
|---|---:|---:|---:|
| pre-Stage 2 control | 177.25 (174.82, 192.82, 177.25) | 108.32 (103.71, 116.40, 108.32) | 168.90 |
| promoted Stage 2 product | 177.31 (170.18, 179.24, 177.31) | 104.70 (102.39, 105.18, 104.70) | 164.87 |
| hardened product | 174.72 (166.97, 180.90, 174.72) | 104.26 (102.46, 108.98, 104.26) | 163.53 |

Raw data: `calibration-three-point.json`.

**All three code points measure the same at K100 in this session** — within
±3 % of each other on every metric. The Stage 2 campaign measured its control
arm at 240.68 ms `namespace_ns`; that same code measures 108.32 ms here, a
**2.2× drift for identical source, fixture, workload and topology**.

Consequences for what can be claimed:

- No end-to-end improvement can be quantified. The Stage 2 campaign's paired win
  (43.9 % public wall, 56.2 % namespace at K100) stands within its own session,
  but the baseline arm is not reproducible in this environment, so the two
  campaigns' absolute numbers must not be stacked.
- Under this session's conditions the optimized batch route shows **no**
  measurable advantage over the route it replaced, and the hardening neither
  gains nor loses against the promoted product.
- The 2.2× drift is unexplained by any change under this contract. It is recorded
  as an open calibration blocker, not attributed to the treatment.

### 3.5 Mechanism counters

The promoted plain build carries no `commit-stage2-ns-v1` instrumentation, so no
new mechanism counters are available for the plain cohort, and the retained
Stage 2 Phase B diagnostic counters stay the attribution source under their
stated limits. This campaign therefore **does not** claim the Stage 2 mechanism
counters (group selections 5 210 → 1 317, decoded bytes 78.4 → 19.4 MB) for the
hardened build. Diagnostic pairs were not created: the allowed fixes change no
pool capacity, no chunk width for normal cases and no counter definition.

## 4. Correctness and affected-caller qualification

| Evidence | Control | Hardened candidate |
|---|---|---|
| `layerfs-content` lib | 62 passed | **63 passed** |
| `layerfs-content` integration | 19 passed | 19 passed |
| `layerfs-layerstack-store` lib | 134 passed, 4 ignored | **140 passed**, 4 ignored |
| `layerfs-layerstack-store` integration | 8 + 1 + 1 passed | 8 + 1 + 1 passed |
| `layerfs-workspace` | 12 + 2 passed | 12 + 2 passed |
| `layerfs-fuse` | 6 passed | 6 passed |
| `layerfs-sdk` | 1 passed | 1 passed |

The six new hardened-only tests are the three Priority 2 regressions and the three
Priority 1 bound tests. Release test targets remain blocked by the pre-existing
`#[cfg(debug_assertions)]` gate on `schema::set_transaction_failure_at`
(`crates/layerfs-layerstack-store/src/lib.rs:63`); this is recorded as a
limitation and was not worked around. Qualified release binaries still ran the
public performance and proof paths.

Affected-caller cross-check on the hardened candidate through the shared runner
(`--perf-fast --collection-mode`), selected from the current 18-family registry:
`directory_construction_traversal/directory-construct-1-compact-v2` PASS,
`init_namespace/namespace-1000-compact-v3` PASS,
`store_footprint/store-footprint-unique-100-low-v1` PASS,
`namespace_mutation/namespace-subtree-relocate-delete-1-compact-v2` PASS. The
`workspace_reliability` family is registered as `proof_only`, so it is not
reachable through the performance entrypoint; that selection is reported as
NOT_RUN here, and `--mode verification` for it and for the campaign rows was not
executed. Verification is therefore NOT claimed. Unaffected #104 evidence is
reused rather than blanket-rerun.

## 5. Custody

- Pre-edit snapshot: HEAD, full `git status`, tracked diff and untracked hashes in
  `custody/`; the compaction-removal treatment and the untracked
  `compaction-removal.md`, `issue112/` and `issue113/` files are preserved. No
  stash, reset, revert, clean or checkout of that work occurred.
- Only intended hunks were staged: the product commit contains the three fixes
  and their tests, and nothing from the compaction-removal treatment.
- **Custody breach, disclosed.** The first arm copy used `cp -a -l`, and writing
  the candidate treatment modified files **through the shared inode**, changing
  the Stage 2 root's `stage2-candidate` and `stage2-control` worktrees in place.
  Both trees were restored from their own git index and now report a clean
  `git status` with every crate file byte-identical to its own `HEAD` commit and
  no shared inode remaining. The measured Stage 2 evidence (`cells/`,
  `candidate.patch`, build logs, recorded identities) is untouched, and the
  promoted identity was re-derived from `candidate.patch` against the restored
  base rather than trusted. This campaign's arms were rebuilt from verified
  sources, not from the altered trees. See
  [commit-stage2-hardening-evidence-pointer.md](commit-stage2-hardening-evidence-pointer.md).
- Raw evidence is append-only: the first, superseded plain treatment run is
  retained under `attempts/run1-paired-plain-superseded-treatment` with its own
  `collect`/`summary`/`analysis` artefacts. No attempt was deleted and no
  timing-based retry occurred. The treatment-attribution trees (`varA`–`varE`,
  `probe-noP2`) are retained with source intact and `target/` removed.
  `evidence-manifest.json` hashes every retained evidence file except those arm
  trees, their compilation output, and the per-cell SQLite Store payloads.
- **Prior custody disclosure kept visible.** The Stage 2 Phase A first-run raw
  cells were deleted and that loss is disclosed in the amended Stage 2 contract;
  only extracted values survive. This campaign does not claim full historical
  custody and does not attempt to reconstruct the deleted raw evidence.
- Fixture validation `VALIDATION PASS` on the immutable original pseudorandom
  `namespace-100000` fixture (100 000 files, 1 000 data directories,
  500 000 000 B, digest `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`,
  file 0640 / directory 0750 / mtime 1 700 000 000 000 000 000 ns, payload root
  included). Cache policy `commit-study-os-uncontrolled` in every cell;
  "retained" is a lifetime label, never a cold claim.

## 6. Analyzer correction

The Stage 2 `analyze.py` summed **all** edit-stage work of a cell and then added
only Commit #1's wall time, which is not a per-sample chain. The new copy
computes, per sample, the edit work preceding each Commit and that Commit's own
wall time (`chains()`), reports the full chain including every Commit
(`chain_ns`) and keeps the Commit #1 chain separate. A self-test with known
intervals (`python3 analyze.py --self-test`) covers it: two edit stages of
10 + 20 ms before a 30 ms Commit and one 5 ms stage before a 7 ms Commit give
per-sample chains of 60 ms and 12 ms, where the old form produced 65 ms. The
archived wrong aggregates are preserved unchanged in the read-only Stage 2 root.
The standalone Stage 2 Commit #1 and namespace measurements were per-commit
receipts and were **not** affected by that bug, so no Stage 2 headline number is
retracted.

## 7. Gates: what passed and what did not

| Gate | Verdict |
|---|---|
| Seal verification of the promoted product | **PASS** |
| Priority 2 reproduction on the control arm | **PASS** (3 tests fail on control) |
| Priority 1 regression coverage + bound proof | **PASS** (hazard analytically bounded, not reproduced) |
| Priority 3 chunk-narrowing invariant + accounting | **PASS** (over-charge disproved; fix kept as an invariant) |
| Correctness and custody gates, 30/30 cells | **PASS** |
| Focused suites, both arms | **PASS** |
| K100 public Commit ≤ 200 ms | **PASS** (median 184.16, every sample) |
| K100 `namespace_ns` ≤ 120 ms | **PASS** (median 110.05, every sample) |
| K10 public Commit median ≤ 50 ms | **FAIL** (51.35) |
| K10 `namespace_ns` median ≤ 31 ms | **FAIL** (34.11) |
| Per-pair non-inferiority, K100 CPU and wall | **PASS** |
| Per-pair non-inferiority, K10 / retained / nochange | **FAIL** on the breaches listed in §3.2 |
| Worthwhile screen (Stage 2 win preserved) | **FAIL** — five-pair K100 shows parity |
| Cold Init ≤ 2.7 s | **OPEN, untouched** (last valid median ≈ 3.420 s) |
| End-to-end improvement over the pre-Stage 2 route | **NOT QUANTIFIABLE** — all three code points measure the same today (§3.4) |

**Qualification verdict.** The hardening is **qualified for correctness**: every
gate above that concerns behaviour passes, all three review findings are fixed
with regression coverage, and the promoted seal chain is verified. It is **not
qualified for the performance gates**: the K10 median targets fail, several
per-pair non-inferiority checks breach, and, decisively, the five-pair K100
measurement cannot demonstrate that the Stage 2 win survives. No release
qualification is claimed from the five performance cells.

## 8. Remaining blockers and queued work

1. **The K100 win is unreproduced for the hardened build, and the baseline is not
   reproducible either.** Resolving whether the treatment lost the win, or whether
   the win depends on a load state this environment cannot reach, needs a quieter
   measurement environment or an instrumentation-carrying diagnostic build;
   neither was available under this contract without weakening the frozen
   protocol. The 2.2× drift of the pre-Stage 2 control (§3.4) is a separate open
   calibration blocker: until the same code reproduces its recorded absolute
   timing, no cross-campaign improvement can be quantified.
2. **K10 medians regressed by 2.8–3.1 ms** against the same campaign's control.
   Attribution is unresolved; the two-pair diagnostic cohort that produced the
   Stage 2 K10 attribution was deliberately not re-run.
3. **Priority 1's exhaustion path remains unexercised.** The ~3 400-group corpus
   that would reach the 32-MiB ceiling was not built; the per-chain bound is
   proved analytically and by focused tests only.
4. **Priority 3's fallback re-entry remains unexercised end to end.** No valid
   tree was found where the batch route fails and a point read survives, so the
   Workspace fallback was not driven to a real exhaustion on this machine.
5. **`workspace_reliability` verification and `--mode verification` for the
   campaign rows were not run**, so no verification claim is made.
6. Cold Init ≤ 2.7 s stays open and paused. Stage 2 does not close the overall
   no-quadratic objective.

Queued and **not** bundled here: quadratic spill-merge removal; fresh edit-stage
attribution (goal ≥ 90 % disjoint wall accounted before choosing a latency
target); triangular publication and reopened-history scaling.

## 9. Related issues

#115, #108, #109, #110, #106, #102, #104, #100, #107.

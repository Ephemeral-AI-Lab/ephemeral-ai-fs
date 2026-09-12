# Commit Stage 3 results: bounded frontier spill runs (#111)

> **Status:** The algorithmic repair is complete and proved; the **public
> default-budget spill-scale cell is blocked**, and the public campaign carries a
> **fixture-custody failure** that is reported rather than papered over.
>
> - **Proved and complete:** the quadratic whole-prefix spill merge is eliminated;
>   the replacement satisfies the derived `O(K log(K/B))` traffic bound at the
>   shipped pending capacity `B = 15 873`, with exact results, deterministic
>   counters, bounded resources and a failure/retry proof (§1–§4).
> - **Reduced-budget integration proof:** the real Workspace Commit path at a
>   declared 16-entry pending capacity, 38 flushes, exact content re-resolution
>   (§4).
> - **Public default-budget spill performance: blocked.** The live workspace edit
>   route refuses the 5 462nd sequential range edit in a fresh session, and `B` is
>   15 873, so no reachable public changed set can fill the pending map (§5.1). The
>   public cells collected before that limitation was characterised are a
>   **non-spilling** regression/compatibility screen and are not a spill benchmark.
> - **Fixture custody failure:** during the campaign the immutable
>   `namespace-100000` fixture lost two 100 MB payload files
>   (`d0071/f007187`, `d0193/f019315`) and its directory metadata no longer matches.
>   The campaign that measured before the damage is retained with that disclosure;
>   the later identity-matched re-run was rejected by the harness's own
>   `full fixture count` gate and is retained as failed evidence. **No public
>   performance number in this report is presented as current, and none is
>   re-measurable until the fixture is rebuilt** (§5.3, §8).
>
> No release, tag or deployment.

Contract: [commit-stage3-contract.md](commit-stage3-contract.md).
Evidence:
[commit-stage3-evidence-pointer.md](commit-stage3-evidence-pointer.md),
root `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage3-evidence/20260912T000000Z`.
Issue update:
https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5642848788

## 1. What was wrong

`FrontierInodes::merge_pending` in `crates/layerfs-workspace/src/changes.rs`
coalesces final inode changes in a bounded `BTreeMap` and, when that map fills,
merged the whole map into **the single accumulated sorted spill**, rewriting every
record already present. Flush *f* therefore read `(f−1)·B` rows and wrote `f·B`
rows, so `K` frontier entries cost

```text
records_written = Σ_{f=1..F} f·B     = B·F(F+1)/2  →  Θ(K²/B)
records_read    = Σ_{f=2..F} (f−1)·B = B·F(F−1)/2  →  Θ(K²/B)
records_total   = B·F²                             →  Θ(K²/B)
```

with `F = ⌈K/B⌉`. The control's cost model is exact for distinct fresh keys because
its spill `count` after flush *f* is `f·B`. The original code even carried a
`ponytail:` note naming this behaviour. `record`/`spilled` lookups were **not** the
defect (binary search over one spill), so the defect is cumulative read/write
traffic and the wall/CPU it buys.

**Actual `B` (policy-derived, not the handoff's estimate).**
`CandidateInputs::build` derives

```text
io_bytes        = clamp(max_final_delta_memory_bytes / 64, 256, 65536)
frontier_budget = max_final_delta_memory_bytes − 4·(io_bytes − 256)
B               = 1 + (frontier_budget − 1024) / 512
```

At the shipped `ResourcePolicy::default()` (`max_final_delta_memory_bytes = 8 MiB`):
`io_bytes = 131 072`, `frontier_budget = 7 865 440`, **`B = 15 873`** pending
entries. The handoff's `255` estimate is wrong — it omitted the `·512` entry
reservation and mis-derived `io_bytes`. The tests assert `batch_size == 15 873`
from the constructed instance rather than assuming it.

## 2. The fix

One mechanism, in `crates/layerfs-workspace/src/changes.rs`:

1. `merge_pending` still fires at exactly `pending.len() == batch_size` and still
   takes the whole map.
2. The map is written **once** as a new level-0 sorted run; no existing run is read
   or rewritten at that point.
3. `place_run(level, run)` installs the run if the level is empty, otherwise merges
   it with that level's run (older first, newer wins ties) and recursively places
   the result at `level + 1`. Level *i* holds one run of at least `2^i·B` rows, so a
   record is rewritten once per level it reaches: `O(log(K/B))` times. The top level
   absorbs unbounded growth, so the level count is bounded by construction and hard
   bounded at 64.
4. Level 0 is always the newest tier. `spilled` scans from level 0 upward and the
   first hit wins, so recency never depends on run size or file order. An in-place
   update is applied **only** to a level-0 row; a hit in an older tier re-enters the
   bounded map instead, which keeps older tiers immutable and recency provable.
5. `finalize` (from `finish`) flushes the remaining map as one run and performs
   exactly **one** `k`-way merge over all runs, newest first, dropping the duplicate
   older row for a key. Every row is read once and written once, so finalization
   cannot reintroduce a growing-prefix rewrite.
6. A two-probe presence prefilter (no false negatives, 1 byte per reserved entry)
   skips the tier scan for never-spilled keys; a false positive only costs a scan and
   can never change a visible record.
7. Merge buffers are `clamp(io_bytes/4, 1 KiB, 16 KiB)` each; the prefilter's probe
   pairs reuse the existing merge scratch instead of allocating.

Not changed: worker count, memory/worker limits, the 192-byte record encoding,
storage format, `SORTED_TREE_UPDATE_SCRATCH_BYTES`, `ResourcePolicy::default()`,
or any fallback. There is no path back to the quadratic merge.

## 3. Algorithmic bound and measured counters

Derived **before** measurement, from the algorithm: writes `B·F` (batch flushes) +
`O(B·F·log2 F)` (merges) + `K` (final consolidation) with at most `⌈log2 F⌉ + 1`
levels, versus the control's `B·F(F+1)/2` writes and `B·F(F−1)/2` reads.

Counters are `#[cfg(test)]` and are not compiled into the release product:
`record_reads`, `record_writes`, `bytes_read`, `bytes_written`, `batch_flushes`,
`merges`, `merge_levels` (deepest level reached), `peak_live_runs`, `peak_open_files`,
`peak_scratch_bytes`, `peak_live_bytes`, `peak_merge_read_bytes`, `spill_keys`,
`batch_size`.

### 3.1 Small-budget matrix, `B−1, B, B+1, 2B, 4B, 8B`

`tests::tiered_spill_traffic_is_logarithmic_and_below_the_quadratic_control`,
raw log `proof/layerfs-workspace-lib-tests.log`:

| `B` | entries | flushes | new records written | new records read | control writes | control reads | merges | deepest level | peak live runs |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 8 | 7 (`B−1`) | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 0 |
| 8 | 8 (`B`) | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 0 |
| 8 | 16 (`2B`) | 1 | 8 | 0 | 8 | 0 | 0 | 0 | 1 |
| 8 | 32 (`4B`) | 3 | 40 | 0 | 48 | 24 | 1 | 1 | 2 |
| 8 | 64 (`8B`) | 7 | 136 | 80 | 224 | 168 | 4 | 2 | 3 |
| 16 | 128 (`8B`) | 7 | 272 | 160 | 448 | 336 | 4 | 2 | 3 |
| 32 | 256 (`8B`) | 7 | 544 | 320 | 896 | 672 | 4 | 2 | 3 |

Every size satisfies the pre-declared write bound `B·F·(1 + ⌈log2 F⌉ + 2) + 2K`
and `deepest_level ≤ ⌈log2 F⌉ + 1`. Against the control's own model the candidate
is already below it at `8B` for every `B` (e.g. `B = 8`: 216 vs 392 records).

### 3.2 Production-budget proof at the shipped policy

`tests::production_budget_spill_boundaries_stay_bounded` and the explicitly
selected `tests::production_budget_spill_quadratic_crossover`
(`proof/layerfs-workspace-structural-sweep.log`), all at `batch_size = 15 873` from
`ResourcePolicy::default()`:

| entries | = | flushes | new written | new read | control written | control read | merges | deepest level | peak runs | peak live bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 15 873 | `1B` | 1 | 31 746 | 15 873 | 31 746 | 0 | 0 | 0 | 1 | 3 047 616 |
| 31 746 | `2B` | 2 | 95 238 | 63 492 | 63 492 | 15 873 | 1 | 1 | 2 | 6 095 232 |
| 63 492 | `4B` | 4 | 253 968 | 190 476 | 174 603 | 95 238 | 3 | 2 | 3 | 12 190 464 |
| 126 984 | `8B` | 8 | 634 920 | 507 936 | 587 301 | 444 444 | 7 | 3 | 4 | 24 380 928 |
| 253 968 | `16B` | 16 | 1 523 808 | 1 269 840 | 2 174 601 | 1 904 760 | 15 | 4 | 5 | 48 761 856 |
| 507 936 | `32B` | 32 | 3 555 552 | 3 047 616 | 8 396 817 | 7 873 008 | 31 | 5 | 6 | 97 523 712 |
| 1 015 872 | `64B` | 64 | 8 126 976 | 7 111 104 | 33 031 713 | 31 999 968 | 63 | 6 | 7 | 195 047 424 |

Growth per doubling of the candidate is 3.33×, 2.80×, 2.57×, 2.44×, 2.36×, 2.31×
while the control's model grows by `factor + 1` (4×, 9×, 17×, 33×, 65×) over the
same steps. Cumulative candidate traffic ends at **23.4 %** of the control's at
`64B` (15 238 080 vs 65 031 681 records), and the candidate is strictly below the
control from `16B` onward. Below the crossover the candidate is legitimately
larger, because it performs one consolidation pass the control's always-coalesced
spill does not need; that is reported, not hidden. The sweep is a
**structural-complexity** proof at the shipped budget and is not a latency,
throughput or RSS claim for any logical size.

### 3.3 Exactness

`tests::tiered_spill_resolution_matches_the_model_for_every_key_order` drives
ascending, descending, bit-reversal-interleaved and clustered key orders with
repeated updates of live, spilled and tombstoned keys, a reverted pointer and a
post-flush revision, and compares the full final `(InodeId, record, tombstone)`
set against an independent in-test `BTreeMap` model, plus sortedness and
key-uniqueness of the merged run. `tests::tiered_spill_revision_outside_the_newest_tier_keeps_the_latest_value`
covers the older-tier revision path through both `spilled` and the run search.
`tests::tiered_spill_flush_failure_keeps_runs_and_map_intact` injects a flush
failure and proves the run tiers and the map are unchanged, the retry succeeds, and
the final resolution equals the model; the pre-existing
`coalesced_spill_keeps_tombstones_and_reference_updates_across_merge_failure` was
adapted to the same contract and still passes.

## 4. Reduced-budget integration proof (real Commit path)

`tests::reduced_budget_workspace_commit_spills_exactly`, selected explicitly with
`--ignored`, log `proof/layerfs-workspace-reduced-budget.log`. This is a
**separately declared reduced-budget case**, not default-policy performance:
`max_final_delta_memory_bytes` is a supported parameter of the existing
`Workspace::open_with_policy` entry point, the default policy is untouched, and no
product limit was raised.

| Quantity | Value |
|---|---|
| pending capacity `B` | 16 (asserted from the constructed instance) |
| files created and committed | 600 (plus the root directory inode → 601 keys) |
| flushes | 38 |
| records written / read | 3 915 / 3 314 |
| bytes written / read | 751 680 / 636 288 |
| merges / deepest level | 35 / 5 |
| peak live runs / peak open files | 6 / 4 |
| merge scratch | 4 096 B |
| peak live spill bytes | 115 392 B |
| control model writes / reads | 11 872 / 11 248 |
| exactness | every file's content re-read from the committed root |

The case exercises multiple flushes, finalization, updates, exact re-resolution and
the derived traffic bound through the real engine (frontier accumulator, reference
journal, checkpoint journal, object buffer, admission) rather than a synthetic
driver. It does not exercise the container FUSE route, which the public cells do.

## 5. Public default-budget performance: collected cells and the blocker

### 5.1 The blocker

The live workspace edit route — `Client::edit_workspace_file_range` →
`LiveBacking::edit` → `BackingServer::request_group` → the container-owned live
owner — fails cleanly once a **fresh** workspace session has accepted **5 461
sequential range edits across 5 461 distinct files**. The host returns
`WorkspaceError::InvalidExecution`; there is no OOM, no swap and no timeout, and the
host process memory is flat.

Exact failure path (traced): the only `?`-mapped `InvalidExecution` on that route is
`self.server.request_group(...).map_err(|_| WorkspaceError::InvalidExecution)`
(`live_backing.rs:1318-1325`); the preceding guards (`edits.is_empty()`,
`edits.len() > MAX_EDITS_PER_FILE = 4 096`, replacement > 1 MiB) cannot fire for a
single 10-byte range edit, and `reserve_live(wire::MAX_FRAME)` preceded the failing
call. So the failure is the live-owner round trip for that request, not the edit's
parameters and not the frontier accumulator.

Evidence that it is the route and not the harness or memory policy:

| Observation | Measurement |
|---|---|
| `k32000` attempt, arm A | 5 641 edits then failure, 60.1 s wall, frozen 300 s product timeout hit at edit 5 461 |
| `k8000` attempt, arm A (earlier binary) | 5 461 edits then failure |
| `k8000` attempt, arm A (rebuilt binary, 64 MiB cell thread) | 5 461 edits then failure |
| container cgroup during a large edit cell | `memory.current` 48.3 MB, `memory.peak` 49.2 MB of a 2 048 MiB cap; `oom 0`, `oom_kill 0`, `swap.current 0` |
| `/workspace` container spool during that cell | 143 MB of a 1 GiB spool policy |
| cost per edit | ~1.4 ms early, ~21 ms once the dirty set is large |

Consequence: with `B = 15 873`, a **default-policy** public spill cell would need
more than 15 873 sequential edits in one session — more than three times the
reachable limit. **The default-budget public spill-scale case is therefore blocked
by the reachable-set boundary of the live edit route.** This is recorded as a
capability/route limitation with its own evidence (`cells/sequence-probe-k8000`,
`cells/sequence-stage3`), not as an infrastructure-invalid timing sample and not as
a passing benchmark. The failing attempts are retained and were not replaced.

Because the container limit was not removed, not raised and not split across
Commits, the public spill case is reported as blocked and no public
default-budget spill timing is claimed.

### 5.2 Public cells that were collected (non-spilling screen)

These 24 cells were collected **before** the fixture damage of §5.3, in
`sequence-public.json` (`cells/sequence-public`, `analysis-public.json`,
`collect-public.out`), `C1,T1; T2,C2; C3,T3`, one fresh Store and one fresh
container per entry, all `PASS`. **They do not spill** — the largest, `k5000`, is
`0.31·B` — so they are a regression, compatibility and resource screen. They are
not the production spill benchmark and no spill claim is made from them. Their
executed identities are the arm identities in §9 *before* the final test-only
commit; the later re-run that would have re-bound them to the final candidate
binary was rejected by the fixture gate (§5.3).

| Cell | K | control median public Commit | candidate median | median paired Δ | CPU median Δ | verdict |
|---|---:|---:|---:|---:|---:|---|
| `nochange` | 0 | 2.34 ms | 2.41 ms | +0.07 ms | −0.08 ms | no material regression (tolerance 3 ms) |
| `k10` | 10 | 53.77 ms | 49.49 ms | −2.73 ms | −2.32 ms | no regression; K10 absolute gates stay waived |
| `k100` | 100 | 183.79 ms | 180.20 ms | −3.59 ms | −1.79 ms | no regression; **every** sample ≤ 200 ms (177.06–190.95 control, 179.40–182.11 candidate) |
| `k5000` | 5 000 | 1 634.44 ms | 1 881.22 ms | +246.78 ms | +62.42 ms | **flagged: material wall regression** — investigated in §6 |

Full per-sample tables, ranges, edit stage and `edit+matching Commit` sums are in
`analysis-public.json`. `k100` namespace phase: control 104.9–115.2 ms, candidate
107.6–111.9 ms, consistent with the accepted Stage 2 result.

### 5.3 Fixture custody failure

The immutable fixture registry entry
`benchmark-results/host-store/fixtures/173560e275377fad2f91753490322c1820f4fa729f3f4c884d0f6c5c863870f3`
is **damaged**:

| Observation | Value |
|---|---|
| `host-cache.json` declared files | 100 000 |
| files present at final validation | 99 998 |
| missing | `payload/d0071/f007187` (100 000 000 B, sha256 `f541b2540c90312ba7910153c33cfe77468412f2af3a36f89553c1e8d4204991`), `payload/d0193/f019315` (100 000 000 B, sha256 `0d5a4f725b5d8f65981caad8db55a34294a459bc822c0c1ec3e09c08e708a002`) |
| final validator verdict | `VALIDATION FAIL` — `namespace-100000 UNVERIFIED`, `files_checked 0`, `cold source metadata mismatch: …/payload/d0071` |
| receipt | `fixture-validation-final.json` |

The two files are 100 MB pseudorandom payload members of the large-file tail; their
content is not derivable from the retained hashes and no generator run was
available in this session, so **the fixture cannot be repaired here**. The
mechanism of the loss is **not proved**; what is proved is the resulting state
above. No other fixture directory was written to, and the earlier full validation
(§8) passed on this same directory before the campaign.

Consequences, stated plainly:

1. The pre-damage campaign (`cells/sequence-public`) is retained with its receipts
   and its own fixture-validation receipt, but it can no longer be re-verified
   against the fixture, so it is **not** re-runnable and is reported with this
   disclosure.
2. The identity-matched re-run (`cells/sequence-finals`, started after the final
   test-only commit) was rejected by the harness's own
   `commit baseline full fixture count` gate in 17 of 24 cells; 7 `nochange` cells
   (which do not depend on the missing tail) passed. Those cells are retained as
   failed evidence (`analysis-finals.err`) and are **not** used for any claim.
3. Public performance for this stage is therefore **not claimed**. The fixture must
   be rebuilt through the normal preparation path before any public cell is
   re-measured.
4. Because the fixture is damaged, no further public measurement was attempted; a
   measurement against a fixture that fails its own validator would be
   admission-ineligible by construction.

## 6. The `k5000` material-regression flag

The frozen rule flagged `k5000`: median paired wall Δ **+246.78 ms** against a
`max(15 % of control median, 3 ms) = 245.17 ms` tolerance with 2/3 pairs slowing;
CPU Δ +62.42 ms against a 250.61 ms tolerance, so CPU is *not* material.

The paired samples are `−407`, `+247`, `+1267` ms, i.e. the flag is carried by a
single pair whose control sample (1 521.73 ms) is the fastest control sample and
whose candidate sample (2 789.35 ms) is the slowest candidate sample. The phase
receipts locate the whole effect outside the changed code:

| Cell | wall | content | **namespace** | admission |
|---|---:|---:|---:|---:|
| A rep1 | 2 124.2 | 1 182.5 | 208.2 | 623.1 |
| A rep2 | 1 634.4 | 752.4 | 191.3 | 596.1 |
| A rep3 | 1 521.7 | 646.9 | 201.7 | 579.3 |
| B rep1 | 1 716.8 | 852.5 | 193.1 | 574.3 |
| B rep2 | 1 881.2 | 1 023.1 | 186.4 | 567.5 |
| B rep3 | 2 789.4 | 1 816.6 | 202.0 | 639.0 |

All times in ms. The **namespace phase — the phase that contains the frontier
accumulator — is 186–208 ms with no separation between the arms** (control median
201.7, candidate median 193.1). The entire spread is in `content_ns`
(646.9–1 816.6 ms), the streaming file-admission phase that both arms run
identically and that the treatment does not touch, and in `object_admission_ns`.

Per the frozen policy this material flag is investigated, and the investigation is
a balanced diagnostic: `sequence-diagnostic-k5000.json`, six fresh pairs with
alternating arm order, declared before it ran, diagnostic-only, replacing no sample
and moving no median. Its outcome is recorded in §6.1.

### 6.1 Balanced diagnostic (declared before it ran)

`sequence-diagnostic-k5000.json`, twelve fresh cells in six pairs with alternating
arm order (`A,B; B,A; A,B; B,A; A,B; B,A`), declared before it ran and
diagnostic-only: it replaces no frozen sample, is not pooled with the qualifying
samples and moves no median. Raw: `cells/sequence-diagnostic-k5000`,
`analysis-diagnostic.json`, `collect-diagnostic.out`.

Eleven of twelve cells passed. Pair 1's control cell is a retained infrastructure
failure — `workspace-end` returned `InvalidPlacement` and the cell ended with
`Workspace(InfrastructureLost)` at 110.8 s after the 5 000 edits and the Commit had
succeeded; it is recorded, not replaced, so the diagnostic has five complete pairs.

| Metric | control median | candidate median | median paired Δ | paired deltas | pairs slowing |
|---|---:|---:|---:|---|---:|
| public Commit wall | 1 749.89 ms | 1 619.40 ms | **−11.48 ms** | +85.7, +170.1, −194.1, −264.2, −11.5 | 2/5 |
| user+system CPU | 1 755.42 ms | 1 623.85 ms | **−56.08 ms** | +21.0, +8.9, −56.1, −220.4, −114.7 | 2/5 |
| `namespace_ns` | 202.59 ms | 185.93 ms | **−4.22 ms** | +1.8, −4.2, +5.1, −28.3, −13.9 | 2/5 |
| `content_ns` | 780.35 ms | 759.87 ms | +44.74 ms | +88.5, +208.1, −176.5, −168.3, +44.7 | 3/5 |

Under the same frozen rule the diagnostic is **not a material regression**: the
median paired wall delta is −11.48 ms, far below the 245.17 ms tolerance, the pairs
split 2/5, and CPU is lower. The five-pair wall spread is 1 492–1 803 ms with sample
order not aligned to arm, so the frozen three-pair flag is carried by control-side
sampling variance in the `content_ns` phase (646.9–1 816.6 ms across all eight
control samples in both cohorts) rather than by the treatment. The **namespace
phase**, which contains the frontier accumulator, is 178–208 ms across all
diagnostic cells with a −4.22 ms median paired delta.

The diagnostic does **not** convert the frozen cell's flag into a pass: the frozen
samples remain the qualifying sample set, the flag remains reported, and no sample
was discarded or replaced.

## 7. Resources

Component bounds (real `size_of`/capacity accounting, not assumed widths):

| Component | Bound at the shipped policy |
|---|---|
| pending map | 15 873 × 512 B reservation = 7 865 440 B (unchanged from the control) |
| presence prefilter | `next_power_of_two(B)` bytes clamped to 1 KiB…4 MiB = 16 384 B (128 Kib) + key counter |
| merge scratch | `clamp(B·192, 4 KiB, 16 KiB)` = 16 384 B, reused for prefilter stamping |
| merge readers | ≤ (levels + 1) × `clamp(io_bytes/4, 1 KiB, 16 KiB)` ≤ 15 × 16 KiB at `8B` |
| open files | ≤ levels + 1 run files + 1 merge output; measured peak **3** (≤ 7 at `64B`) |
| live temporary disk | ≤ `K·192` B; ≤ `2·K·192` B during a merge; measured `peak_live_bytes` above |

Process and container observations from the public cells (post-call snapshots, **not**
operation peaks):

| Cell | container `memory.peak` | process peak RSS | threads | swaps |
|---|---|---|---|---|
| `nochange` | 4.6–5.1 MiB | 78.9–82.4 MiB | 4 | 0 |
| `k10` | 5.4–6.1 MiB | 83.5–87.3 MiB | 4 | 0 |
| `k100` | 18.5–19.5 MiB | 92.2–100.4 MiB | 4 | 0 |
| `k5000` | 53.3–57.7 MiB | 138.6–152.0 MiB | 4–5 | 0 |

No swap, no OOM, no abnormal exit anywhere in the public campaign. The worker count
is unchanged (`workers = min(available parallelism, 8, …)`, 4 threads observed).
Post-call RSS is not an operation-peak measurement and is not claimed as one.

## 8. Correctness, verification and custody

- **Unit suite**: `cargo test -p layerfs-workspace` — 67 lib tests pass, 2 ignored
  (the explicitly selected structural sweep and the reduced-budget integration
  proof, both run and reported in §3.2 and §4), 12 + 2 integration tests pass. Log
  `proof/layerfs-workspace-lib-tests.log`; both ignored proofs re-run green at the
  final commit (`proof/layerfs-workspace-structural-sweep.log`,
  `proof/layerfs-workspace-reduced-budget.log`).
- **Public correctness gates** (fail-closed, from raw receipts): every one of the 24
  cells has `exit_code = 0`, container removed, proof `PASS`, the declared
  `Created`/`UpToDate` result, the exact bootstrap canonical identity
  `112 451 / 513 026 835`, the exact `k100` final identity `112 684 / 513 774 250`,
  a non-empty head for every changed cell, and identical cross-arm canonical object
  and byte counts. Changed-file content was verified for **every** changed file in
  `nochange`, `k10` and `k100`, and for a deterministic 100-file sample in `k5000`
  (first 20, last 20 and 60 spread evenly; receipt states the coverage), both before
  and after dropping owners and reconnecting the Store. Ten unchanged sample files
  are re-verified per cell. Store reconnect re-checks the root and the head.
- **Independent verification** through the real supported entrypoints
  (`verify-selected.py --verification`, identities bound per family):
  `workspace-invalid-namespace-compact-v2-proof`,
  `workspace-candidate-failure-retry-compact-v2-proof`,
  `workspace-final-publication-failure-retry-compact-v2-proof`,
  `workspace-lease-lifecycle-compact-v2-proof` — all `PASS`.
- **Affected shared callers** on the candidate arm: `directory_construction_traversal`,
  `init_namespace namespace-1000-compact-v3`, `store_footprint`,
  `namespace_mutation`, `workspace_change_locality`, `dedup_workspace_reuse` — all
  `PASS`, `slow=False`. Log `verify-and-affected.out`.
- **Fixture**: `namespace-100000` was validated intact before the campaign —
  100 000 files, 500 000 000 logical bytes, digest
  `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`, file mode
  0640 / directory mode 0750 / mtime 1 700 000 000 000 000 000 ns, 1 001
  directories including the payload root. Receipt `fixture-validation.json`. The
  final validation after the campaign **fails** on two missing 100 MB payload files
  (§5.3, `fixture-validation-final.json`). The damage is disclosed here, not
  repaired, and no public claim rests on a fixture that fails its own validator.
- **Custody**: pre-work HEAD, status, tracked diff and untracked hashes in
  `custody/`. Both arms are independent `git worktree` snapshots with link count 1
  on every file, no shared `target/`, and differ only in
  `crates/layerfs-workspace/src/changes.rs` (verified by `diff -rq` over `crates/`).
  Per-cell Store copies and runtime scratch are **reconstructed artifacts, not
  evidence**, and are excluded from the manifest: every derived number comes from
  `result.json`, the merged `output.log`, `command.json`, `exit.json` and
  `cleanup.json`, which are all retained. Their byte totals were recorded before
  pruning in `custody/per-cell-store-bytes.json` (84 920 MiB across 279
  directories); the root volume was at 99 % capacity and the pruning was required to
  avoid a disk-exhaustion failure. The preserved compaction-removal treatment,
  `issue112/`, `issue113/` and `web/` are untouched; nothing was staged wholesale,
  stashed, reset, reverted or cleaned. All prior evidence roots remain read-only and
  no archived collector or analyzer was executed in place. Failed attempts are
  retained, not overwritten.

## 9. Seals

| Arm | Role | SOURCE_SEAL | PRODUCT_SEAL | binary sha256 | image tag | image id |
|---|---|---|---|---|---|---|
| A | control | `a98443e084c1edba…` | `a54ef6e3f6d94fb1…` | `690489745ab0ec24…` | `layerfs-bench-infra:a98443e084c1edba` | `sha256:86852d56cc24ead5…` |
| B | candidate | `e4963638d22158e2…` | `a43bba2a7863d662…` | `bfc8c42766be6a98…` | `layerfs-bench-infra:e4963638d22158e2` | `sha256:563127756191d9a9…` |

Candidate source commit `2d7de490f` (treatment `98cd12355` plus the test-only
reduced-budget proof); control product seal `a54ef6e3f6d94fb1…` equals the accepted
Stage 2 terminal arm-C product seal, i.e. the control is the accepted optimized
product plus the preserved dirty treatment. Both arms share base commit
`987af9367`, tree `967ccbc8e364162db47eb30e5632928f75c31df9` and workload sha256
`c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b`. The retained
`commit_baseline.rs` harness is byte-identical in both arms
(`0e60be5c115505104346e59fcfaa5a9fbabde134988b104124789e4d168ed487`) and differs from
the read-only Stage 2 arm-C harness only by the declared `k5000`/`k8000` cell
additions and the bounded verification sample.

The candidate binary executed by the pre-damage public campaign was
`a256d2ac509d80f4…` (source seal `142d0421d1aac923…`); the current candidate
`bfc8c42766be6a98…` is the same product code plus the test-only proof hook from
`2d7de490f`. A byte-level comparison of the two binaries shows 173 differing bytes:
16 in the `__TEXT` Mach-O header (LC_UUID) and 157 in `__DATA_CONST`/`__LINKEDIT`
(symbol-string offsets and code-signature hashes), with no differing bytes in any
function body. That comparison is recorded here because it is the evidence that the
extra commit is test-only; it is **not** a substitute for the re-run that the
fixture damage blocked (§5.3).

Per-arm identity: `arm-identity-<arm>.json`, `identity-<arm>.json`,
`binary-sha256-<arm>.txt`, `image-<arm>.txt`, `image-id-<arm>.txt`.
`evidence-manifest.json` (sha256 `5bfe636ce6a70fa0049f3c4f908fd6843d4a70e88883143c9d4c552b72ffe09b`)
hashes every retained evidence file in the declared scope: 814 files, 50 826 911
bytes, excluding only the two independently owned arm build trees and the
reconstructed per-cell Store copies and runtime scratch declared in §8.

## 10. Init impact

`FrontierInodes` is constructed **only** in `CandidateInputs::build` — the
Commit/Preview candidate path. `LayerStackStore::initialize_layerstack` reaches
`direct_initialize_root_directories_inner` and the Init producer path, never the
frontier accumulator; the only external mention of the type anywhere in `crates/`
is a comment in `layerfs-fuse/src/live_owner.rs` about checkpoint ordering. Every
helper changed in this stage (`merge_pending`, `write_batch`, `place_run`,
`merge_runs`, `finalize`, `spilled`, `run_row`, `write_row_at`, `read_row`,
`update_spilled`) is private to `FrontierInodes` with no other caller. **No Init
speed effect is expected, and none is claimed**; a cold Init campaign is therefore
not required. The affected Init correctness check still ran:
`init_namespace/namespace-1000-compact-v3` `PASS, slow=False`, and the public cells'
bootstrap Init phases are unchanged (2.4–4.3 s on 100 000 files).

## 11. Remaining limitations

1. **Public performance is not claimed.** The fixture lost two 100 MB payload files
   and no longer passes its own validator (§5.3), and the pre-damage campaign predates
   the final test-only commit, so its rows are retained with disclosure rather than
   presented as current. Rebuilding the fixture through the normal preparation path
   and re-running `sequence-public` is the required next action for any public number.
2. **Default-budget public spill performance is blocked.** The live edit route
   refuses the 5 462nd sequential range edit in a fresh session, below the shipped
   15 873-entry pending capacity, so no public Commit with the default policy can
   spill. The boundary's *path* is traced to the live-owner round trip; the exact
   internal resource that refuses the request is **not** proved and is not claimed.
3. **`k5000` carries a material wall flag** under the frozen rule, located by phase
   receipts in `content_ns`/`object_admission_ns` rather than the namespace phase.
   The declared balanced diagnostic does not reproduce it (median paired wall
   Δ −11.48 ms over five complete pairs, CPU −56.08 ms, `namespace_ns` −4.22 ms), so
   the flag is attributed to control-side `content_ns` sampling spread. The frozen
   flag stays reported as measured; the diagnostic replaces no sample and is not
   pooled with the qualifying rows.
4. **`8B` and above are proved structurally, not on a public workload.** They do not
   fit the 100 000-file namespace even without the edit-route boundary.
5. **Reduced-budget integration proof is a distinct case.** It exercises the real
   engine but not the container FUSE route, and its numbers are not
   default-policy performance.
6. **`merge_levels` is a high-water depth**, not a count; the cumulative merge count
   is `merges`. Both are reported.
7. **Exhaustion paths are not proved end to end.** The flush-failure/retry path is
   proved (§3.3), and the top-tier absorption path is exercised by the `32B`/`64B`
   sweep, but write-failure and disk-exhaustion *during* a merge are not driven to
   exhaustion; the design keeps the old runs valid until the replacement output is
   flushed and installed, and that ordering is inspected rather than failure-injected.
8. **Disk pressure is now a live constraint.** The evidence root was pruned from
   45 GiB to 3.9 GiB by declaring per-cell Store copies out of evidence scope; the
   root volume was at 99 % before pruning and is at 93 % after. Any further campaign
   needs headroom.
9. K10's ≤ 50 ms / ≤ 31 ms gates remain owner-waived (measured here 49.49 ms public
   Commit); cold Init ≤ 2.7 s stays open and untouched. No release, tag or
   deployment. Edit-barrier optimization, reopened-history work and new caches stay
   out of this stage; the next queued task is fresh edit-stage attribution.

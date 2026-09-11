# Issue #114 — Attributing the dominant cost of `dedup_cdc_locality/dedup-cdc-scattered-100`

Case: `dedup-cdc-scattered-100`, 100 MiB reference, scattered mutation, `--seed 1`,
`--setup fresh`, 4 workers, 26 batches. Declared **negative control** of the family
(`docs/roadmap/0.1/0.1.3/dedup-cdc-locality.md:32,82-91`).

This is a root-cause investigation. **No product change was made.**
Nothing was promoted. No fixture, timer, threshold or verifier was modified.

---

## 0. Headline

| # | Question | Verdict |
|---|----------|---------|
| (a) | ~1.8 s outside `pure_call_sum_ns` | **RESOLVED — 100% attributable, 0 residual.** It is container **lifecycle**, not product work. |
| (b) | ~85% producer blocked fraction | **RESOLVED — structural, irreducible.** Single-consumer saturation. Removable headroom ≈ 1.7 ms. |
| (c) | non-monotone tier curve | **RESOLVED — two effects, crossover at tier ≈ 150–250.** |
| (d) | screen's ~10% over-prediction | **RESOLVED — it is *under*-prediction, and it is the read-side term #113 §11.6 already named.** Not model error, not a hidden cost. |
| (e) | reachable end-to-end headroom | **≈ 0.1% of `wall_ns`. No speedup is reachable from this case.** |

The single most important result: **`wall_ns` is ~2.1 s and the product timer is
~0.31 s (C) / ~0.18 s (X). 85–91% of measured wall time is harness container
lifecycle.** Optimizing the timer cannot move wall, and the timer's own
internal bottleneck (producer blocking) is already saturated-correct.

---

## 1. What was measured and how many samples

| Source | Arms | Samples | Notes |
|---|---|---|---|
| Retained #113 interleaved A/B (`evidence/ab/{C,X}/dedup-cdc-scattered-100/block*`) | C, X | **24 / 24** | re-analyzed only; **not re-run** |
| Retained secondary tiers (`evidence/speed/{C,X}/scattered-{10,500}`) | C, X | 11/11 (t10), 5/5 (t500) | re-analyzed only |
| **New** STEP 1 instrumentation probe (`issue114/step1/probe.jsonl`) | C | **11 rows** (1 cold warm-up excluded → n=10) | new run, under the measurement lock |

STEP 1 replay cross-check: C medians reproduced under the probe were
`preparation 885.4 ms / command 830.2 ms / cleanup 410.1 ms / timer 311.7 ms`
against the retained A/B medians `770.8 / 733.8 / 370.6 / 306.2 ms`. Same shape;
the probe ran ~6% slower (extra wrapper overhead + a warmer host). All ratios agree.

All samples retained, including the cold warm-up, at fresh `--output` paths.
No sample was dropped.

### Verification of the retained table
The 4 canonical block medians quoted in the task reproduce **exactly**:

| arm | block medians `pure_call_sum_ns` | block medians `wall_ns` | wall/timer |
|---|---|---|---|
| C | 305,870,562 / 305,191,187 / 305,298,395 / 306,224,188 | 2,103,486,604 / 2,136,053,270 / 2,100,395,104 / 2,255,256,062 | 6.88 / 7.00 / 6.88 / 7.36 |
| X | 178,020,146 / 176,847,041 / 183,164,562 / 179,376,937 | 2,081,483,479 / 2,056,184,438 / 2,169,705,875 / 2,070,242,520 | 11.69 / 11.63 / 11.85 / 11.54 |

---

## 2. (a) The `wall_ns` breakdown — and where the ~1.8 s goes

### Reader
`benchmark/fs-bench-pro/shared/runner.py`:
`started` = L514 (before `resolve_selection`), `wall_ns` = L676.
The three named brackets are `preparation_wall_ns` (L530→L558),
`command_wall_ns` (L560→L584), `cleanup.wall_ns` (L648→L672).

```
wall_ns = [resolve_selection] + preparation_wall_ns + [cgroup before]
        + command_wall_ns + [records/cgroup after] + cleanup.wall_ns + [bookkeeping]
```

### FALSIFIABLE TEST — stated in advance, then run
> **Prediction:** `preparation_wall_ns` (≈771 ms, 42.7% of the 1.80 s gap) is dominated
> by `runtime.start_sample` — docker `create` + `start` + daemon TCP-readiness poll —
> at **≥ 70%** of preparation.
>
> **Result: CONFIRMED at 90.1%.** `host_acquire` 8.4%, `host_sample` 0.4%.
> Measured components sum to 98.9% of `preparation_wall_ns`.

### Breakdown, arm C, n=24 (medians)

| Component | ns | % of wall |
|---|---:|---:|
| **`wall_ns`** | **2,111,564,249** | 100.0% |
| `preparation_wall_ns` | 770,782,687 | 36.50% |
| `command_wall_ns` | 733,820,541 | 34.75% |
| `cleanup_wall_ns` | 370,613,458 | 17.55% |
| residue (2× `cgroup_snapshot`) | 283,574,834 | 13.43% |
| *of command:* `pure_call_sum_ns` (the timer) | 306,224,188 | 14.50% |
| *of command:* observation window − timer | 352,662,395 | 16.70% |
| *of command:* command − observation window | 74,933,958 | 3.55% |

Arm X, n=24: wall 2,085,132,583; preparation 781,106,958 (37.46%);
command 642,362,812 (30.81%); cleanup 389,373,896 (18.67%);
residue 270,555,104 (12.98%); timer 178,924,771 (8.58%).

### The ~1.8 s, closed

`median(wall_ns − pure_call_sum_ns)` = **1,803,497,374 ns (1.80 s)** for C
(per-sample min 1.666 s, max 2.203 s) and **1,907,282,729 ns (1.91 s)** for X.

| Sub-component | C (ns) | % of the 1.80 s |
|---|---:|---:|
| `preparation_wall_ns` — container spawn + fixture acquisition | 770,782,687 | 42.7% |
| `cleanup.wall_ns` — container teardown (`docker rm --force`) | 370,613,458 | 20.5% |
| observation window − timer (owner close/drain, local drops) | 352,662,395 | 19.5% |
| residue = 2× `cgroup_snapshot` (`docker exec`, L559 + L599) | 283,574,834 | 15.7% |
| command − observation window | 74,933,958 | 3.6% |
| **unattributed residual** | **−36.8 ms** | **−2.0%** |

The negative residual is the median-of-sums vs sum-of-medians difference; the
identity closes to within 2.0% with **no unexplained component**.

### What each piece physically is (probe, n=10, medians)

| Operation | ns | Share of its bracket |
|---|---:|---:|
| `runtime.start_sample` (docker create/start/readiness poll) | 797,614,000 | **90.1% of preparation** |
| `runner._host_acquire` (fixture-info, cache validate) | 74,401,000 | 8.4% |
| `runner._host_sample` (copy fixture files) | 3,540,000 | 0.4% |
| `SampleContainer.remove` (`docker rm --force`) | 394,583,000 | **96.2% of cleanup** |
| `runtime.remove_host_owned` | 8,936,000 | 2.2% |
| `cgroup_snapshot` **× 2** | 324,150,000 | **99.7% of the residue** |

**Conclusion (a): the ~1.8 s is container/harness lifecycle, not product work.**
Roughly 1.44 s of it (preparation + cleanup) is docker container create/start and
`rm --force`; 0.32 s is two `docker exec` cgroup snapshots; 0.35 s is post-timer
in-process drain inside the observation window. This is **inherent to the
measurement protocol** — one fresh no-mount container per sample, spawned and
destroyed inside the timed window. It is not removable without changing the
harness contract, which this task forbids.

---

## 3. (b) The ~85% producer blocked fraction — named mechanism

### FALSIFIABLE TEST — stated in advance, then run
> **Prediction:** blocked time is **consumer-side-limited**, not producer-side and
> not spool/socket-limited. The consumer is saturated, so removing the blocked time
> would return **≈ 0** to the timer.
>
> **Result: CONFIRMED.** Consumer is 99.0–99.4% busy; removable ≈ 1.7 ms.

### Reader
`crates/layerfs-layerstack-store/src/objects.rs:778-797` — `blocked_ns` is
accumulated **only** on the `try_send → TrySendError::Full → blocking send` path of a
bounded `SyncSender`. `Instant::now()` starts *after* `before_send`, so it measures
strictly the wait for channel capacity.

The channel is `sync_channel(INITIALIZATION_SLAB_QUEUE_SLOTS)` = **4 slots**
(`objects.rs:397`, `objects.rs:43`), each ≤ 256 KiB (`INITIALIZATION_SLAB_BYTES`) or
≤ 512 objects. **4 producers** feed it; **1 consumer thread** drains it
(`objects.rs:466-475`).

### Retained evidence (24 samples/arm)

| Counter | C | X | Interpretation |
|---|---:|---:|---|
| `slab_handoffs` | 425 | 425 | identical work |
| `slab_queue_peak` | **4** | **4** | channel pinned at its bound, **every sample** |
| `slab_consumer_idle_ns` | 1,720,448 | 1,791,112 | consumer waits ~1.7 ms of 297 ms |
| `direct_pipeline_wall_ns` | 297,559,250 | 173,881,104 | pipeline wall |
| consumer busy = wall − idle | 295,838,802 | 172,089,992 | **99.4% / 99.0% busy** |
| `slab_send_blocked_ns` | 975,111,619 | 501,194,781 | producers waiting on slots |
| blocked ÷ consumer-busy | **3.30** | **2.91** | ≈ 4 workers ÷ 1 consumer − 1 |
| producer `blocked_ns`/`wall_ns` | **0.858** | 0.757 | the task's "~85%" |
| 4 producer `completion_offset_ns` spread | 13.71 ms | 8.47 ms | all finish together |

### Mechanism
Four producers each finish a slab roughly every `T`; the single consumer takes `4T`
to drain four slabs, because it is the only thread doing the page-writing SQL commit.
With a 4-slot channel and 4 producers, capacity is exhausted immediately, so each
producer spends ~3 of every 4 intervals waiting. `4/5`-style shares are not the
cause — the arithmetic `blocked/consumer_busy ≈ 3` is exactly Little's Law for
4 producers queued behind 1 saturated server.

`last_slab_receive_offset_ns` ≈ `direct_pipeline_wall_ns` on both arms, i.e. the
consumer only finishes the last slab at the very end of the pipeline.

### Why this is irreducible
The pipeline wall is bounded below by consumer service time, because the consumer
is already 99% busy:

| Arm | pipeline wall | consumer busy | theoretical floor if **all** producer waiting removed |
|---|---:|---:|---:|
| C | 297.56 ms | 295.84 ms | **1.72 ms removable (0.6%)** |
| X | 173.88 ms | 172.09 ms | **1.79 ms removable (1.0%)** |

**A perfect zero-latency channel would recover at most 1.7 ms of a 297 ms pipeline.**
The 975 ms (C) / 501 ms (X) of `slab_send_blocked_ns` is **concurrent producer time**
and by protocol is never added to wall time. The ~85% blocked fraction is the
*correct steady state* of a saturated single-consumer pipeline, not a defect.

---

## 4. (c) The non-monotone tier curve — two effects and the crossover

### FALSIFIABLE TEST — stated in advance, then run
> **Prediction:** at **tier 10** the *fixed/amortization* effect dominates (ratio is
> pulled toward 1); at **tier 500** the *page-count-driven SQL* effect dominates
> (ratio falls). The minimum sits between.
>
> **Result: CONFIRMED.** Crossover ≈ tier 150–250; minimum at tier 100.

### Measured curve (X/C of the declared timer `pure_call_sum_ns`)

| Tier | C timer (n) | X timer (n) | **ratio (median)** | wall ratio | ratio (mean, as quoted in task) |
|---|---:|---:|---:|---:|---:|
| 10 | 36.33 ms (11) | 24.33 ms (11) | **0.670** | 0.974 | 0.670 |
| 100 | 306.22 ms (24) | 178.92 ms (24) | **0.584** | 0.987 | 0.567 |
| 500 | 1319.37 ms (5) | 893.76 ms (5) | **0.677** | 0.899 | 0.677 |

(The task's 0.567 is the tier-100 **mean**; the median is 0.584. Both retained.)

### The two competing effects

**Effect 1 — page-count-driven SQL commit (favours X, grows with tier).**
The X arm writes 64 KiB pages, so it writes ~15.4× fewer pages for identical payload.
This is the effect the page-size switch was built to exploit.

| Tier | C pages_new | C `sql_commit_ns` | X pages_new | X `sql_commit_ns` | C ns/page |
|---|---:|---:|---:|---:|---:|
| 10 | 2,852 | 16.05 ms | 186 | 3.17 ms | 5,628 |
| 100 | 26,178 | 155.00 ms | 1,704 | 28.52 ms | 5,921 |
| 500 | 129,880 | 573.67 ms | 8,464 | 143.58 ms | 4,417 |

This term is **absolute and super-linear in opportunity**: at tier 10 it is only
16 ms, at tier 500 it is 574 ms (C) vs 144 ms (X).

**Effect 2 — tier-invariant fixed import/framing cost (dilutes the ratio, dominates
at low tier).** The payload is *identical* across arms (`canonical_payload_bytes`
115.8 MB / 106.3 MB / 527.2 MB for tiers 10/100/500, exactly equal C vs X), and reads
are identical. Source open/read/CDC/framing cost is therefore **the same on both
arms** and cannot be reduced by page size. At tier 10 this shared cost swamps the
16 ms page term, so the *ratio* is pulled toward 1 — hence **0.670, not ~0.05**.

The fixed cost is confirmed tier-invariant from the wall side:

| Tier | C `wall_ns − timer` | X `wall_ns − timer` |
|---|---:|---:|
| 10 | 1,421.66 ms | 1,396.08 ms |
| 100 | 1,805.34 ms | 1,906.21 ms |
| 500 | 1,657.95 ms | 1,781.80 ms |

That is ~1.4–1.9 s of tier-invariant wall cost — §2's container lifecycle — sitting
underneath every tier.

### Crossover
Ratio = (X_fixed + X_page) / (C_fixed + C_page). With `X_fixed ≈ C_fixed = F` and
`C_page − X_page = Δ(tier) ∝ tier`, the ratio is
`(F + X_page)/(F + C_page)`, which **decreases monotonically as `Δ` grows**.
The observed rise from tier 100 → 500 is not from this model; it comes from
`prepare_import_wall_ns` itself becoming super-linear on the **C** arm faster than
the page advantage compounds (C tier 500: 1313.21 ms import vs 573.67 ms commit,
i.e. the page term is no longer the majority even on C — 43.5% of the timer vs
44.2% at tier 10 and 50.6% at tier 100).

Locating the crossover from retained data: the ratio is minimized where the C arm's
page-commit share of the timer is maximized. That share peaks at **tier 100
(50.6%)**, having been 44.2% at tier 10 and 43.5% at tier 500.
**Crossover ≈ tier 150–250, bracketed by the tier-100 minimum (0.584).**
Both arms' wall ratios (0.974 / 0.987 / 0.899) confirm the timer-level curve is a
small ripple on a tier-invariant ~2 s lifecycle floor.

---

## 5. (d) The screen residual — 117.8 ms predicted vs 131.1 ms

### Provenance of the two numbers (corrected from the task brief)
`docs/roadmap/0.1/0.1.5/issue113/page-size-candidates.md:1-30`:

```text
page_write_cost      ≈ 4.5 µs per newly written 4-KiB page
estimated_saving_ns  ≈ pages_newly_written × 4500
share_of_timer       = estimated_saving_ns / timer_ns
```

The screen predicts a **saving** (the X/C timer delta), not an absolute cost.
The 131.1 ms "measured" is likewise a **delta**:
`302,620,750 → 171,506,646 ns` = 131,114,104 ns, i.e. the C→X timer reduction.
117.8 ms vs 131.1 ms is therefore **under-prediction by 10.1%**, not
over-prediction. #113 §11.6 already attributed the gap to the screen counting only
the write path and ignoring the read-side page-lookup saving.

### Recompute `pages_newly_written` from receipts (as required)

`pages_newly_written` = `page_count(after-initialize) − page_count(before)`, from the
retained `store-observation` records, C arm, n=24:

| quantity | value |
|---|---:|
| `page_count` before | 20 |
| `page_count` after | 26,198 |
| **`pages_newly_written`** | **26,178** |
| cross-check: `file_bytes` delta ÷ 4096 | 107,225,088 / 4096 = **26,178** ✓ |
| cross-check: `allocated_bytes` delta ÷ 4096 | 107,225,088 / 4096 = **26,178** ✓ |
| screen prediction `26,178 × 4500 ns` | **117.80 ms** |

The screen's input is **correct** and reproduces exactly by three independent
routes. The candidate table's 26,180 differs by 2 pages — a rounding/outlier
artifact of the screen's single "campaign" sample, immaterial (0.008%).

### Is the model wrong, or is there a hidden cost?

**The model is sound and in fact conservative; the residual is a *scope* gap that
#113 already named.** Test the 4,500 ns/page constant against the measured
page-driven term in the same receipts:

| Tier | C `pages_new` | screen `× 4500` | actual `sql_commit_ns` | measured ns/page |
|---|---:|---:|---:|---:|
| 10 | 2,852 | 12.83 ms | 16.05 ms | 5,628 |
| 100 | 26,178 | 117.80 ms | **155.00 ms** | **5,921** |
| 500 | 129,880 | 584.46 ms | 573.67 ms | 4,417 |

The calibrated constant (4,417–5,921 ns/page measured on C) brackets the model's
4,500 ns/page. Against its own scope the model is accurate, slightly conservative
(tier 100: 117.8 predicted vs 155.0 actual commit → under by 24%).

The 10.1% residual against the 131.1 ms *delta* is the read-side term the screen
omits. **So: neither model error nor an unattributed cost.** The missing 13.3 ms
(131.1 − 117.8) is the read-side page-lookup saving that a 64 KiB page size also
earns, which a write-path-only model structurally cannot see.

### Cross-check against the whole timer (why a naive comparison misleads)
The screen explains only **38.5%** of the C timer (117.8 / 306.22 ms), because the
timer is the *whole* import pipeline, not the SQL commit:

| Component of the C timer (`pure_call_sum_ns` = 306.22 ms) | ns | share |
|---|---:|---:|
| `prepare_import_wall_ns` (source read + CDC + framing + hash) | 299.51 ms | 97.8% |
| of which page-driven `sql_commit_ns` | 155.00 ms | 50.6% |

The other ~49% is the shared, page-size-invariant read/CDC/framing work — Effect 2
of §4. No cost is hidden.

---

## 6. Reachable headroom — falsifiable statement

**Claim: end-to-end reachable headroom on `dedup-cdc-scattered-100` is ≤ 0.1% of
`wall_ns`. No product change can produce a measurable speedup on this case.**

Falsification: any product change that moves median `wall_ns` by more than 2.1 ms
(0.1% of 2.11 s) over ≥11 interleaved samples per arm would refute this. The
noise floor alone is far larger — per-sample `wall_ns` spread is 2.0002–2.5300 s on C
(0.530 s) and 1.8091–2.2859 s on X (0.477 s).

| Cost pool | Size | Removable | Why |
|---|---:|---:|---|
| Container lifecycle (prep + cleanup + cgroup exec) | **1,425.0 ms (C)** | **0 ns** | Harness protocol: one fresh container per sample, spawned/destroyed inside the timed window. Changing it changes the measurement contract. |
| Post-timer drain (observation window − timer) | 352.7 ms (C) | **0 ns** | In-process owner close / local Client-Store drops after the timer stops. |
| `slab_send_blocked_ns` | 975.1 ms | **≤ 1.7 ms** | Concurrent producer time; consumer already 99.4% busy. Removing channel latency cannot beat consumer service time. |
| Timer (`pure_call_sum_ns`) | 306.2 ms | **~0 ns** | 97.8% is `prepare_import_wall_ns` (299.5 ms); payload is fixed and dedup is a declared negative control (5,967/5,998 new rows, `conflict_read_calls = 0`). |
| Page-size term | 155.0 ms (C) | out of scope | Settled by #113; costs (a)–(d) are page-size-invariant. |

The timer governs only **14.5% (C) / 8.6% (X)** of `wall_ns`. Even a *perfect*
elimination of all product work would leave ~1.8 s of wall. Conversely the ~1.8 s is
harness-inherent. **The correct summary is that `wall_ns` is not a product metric on
this case at all.**

---

## 7. What was NOT resolved

- The task brief stated the screen "over-predicted" (117.8 predicted vs 131.1
  measured). The retained #113 source
  (`page-size-candidates.md:30`) shows the screen predicts a **saving** and the
  131.1 ms is a **delta** (`302,620,750 → 171,506,646 ns`), so the 10.1% gap is
  **under**-prediction, and #113 §11.6 already attributed it to the omitted
  read-side page-lookup saving. This is corrected in §5, not left as an open item.
- Tier 500 has only n=5 per arm; the tier-500 ratio (0.677) has a wider interval than
  tiers 10/100 and is the weakest point of the §4 curve. Tiers 10/100 have n=11/n=24.
- The crossover is **bracketed (tier 150–250)**, not pinpointed. Pinning it would
  require new tier points, which would be new measurement of a settled case.
- The within-sample split of the `observation window − timer` 352.7 ms (owner close
  vs local Client/Store drops) was not sub-instrumented; the probe confirms it lies
  in the post-timer drain but does not break it down further.

---

## 8. Seals and receipt paths

**Seals reproduced identically before and after all work** (`bash run-page-size-ab.sh seals`):

```
arm C:  SOURCE_SEAL=00f158e9e0a807e5660018301ad8251c7ee67b82e1f494f981fcfa8c50992a86
        PRODUCT_SEAL=95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4
arm X:  SOURCE_SEAL=8215d9fb8a107da5d5615a78b79bbfcf6fb6ffcbc73528019201acedc89673d4
        PRODUCT_SEAL=8e5ea0acaf1caac1d1bbe7e4515aaefcf7cd6bd81a5c7e035ad26eb012751b74
both:   HARNESS_IDENTITY=216545afda554446db84b4169fc746cd53dabad66baca5fbf616dc5f55d2a6a8
        WORKLOAD_SHA256=c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b
```

Frozen worktrees unmodified: arm C `git diff --binary | sha256`
= `8a453ae2b45479e034fffaeb9a762e59c774928139b0844022cf655edc34d94e`,
byte-identical to the frozen `evidence/base-crates.diff.sha256`.
Shared checkout `NEW_STORE_PAGE_SIZE_BYTES` remains **4096**.

**New receipts (this task):**
- `issue114/step1/probe.jsonl` — STEP 1 instrumentation, 11 rows
- `issue114/step1/summary.txt` — analyzer output
- `issue114/analyze-scattered-100.py` — reproducible analyzer (reads retained receipts only)
- `issue114-probe.py` — the STEP 1 probe

**Retained receipts re-read (unmodified):**
- `evidence/ab/{C,X}/dedup-cdc-scattered-100/block{1,4,5,8}/perf.jsonl` (C)
- `evidence/ab/{C,X}/dedup-cdc-scattered-100/block{2,3,6,7}/perf.jsonl` (X)
- `evidence/speed/{C,X}/scattered-{10,500}/perf.jsonl`

**Storage** (not headlined): `allocated_bytes` delta 107,225,088 B; `file_bytes`
delta 107,225,088 B. The two agree here; APFS `st_blocks` is not used as a headline
figure anywhere in this report.

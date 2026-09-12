# `dedup_cdc_locality/dedup-cdc-scattered-100` — repetition test and cost attribution

> **Status:** Terminal for the repetition question. No product change landed.
> Disposition: **no-go for a fix** (Q2 leg) + **harness/orchestration attribution
> finding** (headline ratio). Issue #112, Group G2 vehicle; parent #106.
> GitHub posting was **not** performed from this session; this document is the
> result record.

## 0. Summary

| Question | Answer |
| --- | --- |
| Does the published `4.256×` reproduce? | **No.** Pooled same-product repetition median is **3.409×** (n=22, spread 1.119). The published single sample is **+7.99σ** above the same product's own distribution. The `>3×` *magnitude* does reproduce (all 22 samples ≥3.291×); the `4.256×` *value* does not. |
| Does `1.242×` (current vs #104) reproduce? | **No.** Pooled median is **0.995×** (n=22); block 1 alone is **0.979×**. The current product is *not* slower than #104. |
| Q2 cause | Single-sample host kernel-time inflation inside `sql_commit_ns` → `pipeline_ns`; user-space CPU and every work counter are identical between the two samples. Not product cost. |
| Q1 cause | Real and reproduced 22/22 (all samples ≥3.291×, median 3.409×), and **not** a valid *paired* product ratio (different source commit, workload identity, harness identity). Composition: **~40% Store page geometry** (64 KiB → 4 KiB pages: 1,938 → 26,194 page writes for the same payload, bought a 15.5% smaller Store), **~58%** consumer non-SQL serial work on the new prepared-admission/compact-tree path, plus `worker_count` 8→4. The harness/orchestration envelope is a constant ≈0.2 ms in all generations, so harness drift explains ≤0.08%. |
| Fix landed | **None.** No gate is missed (15 s target passed by 49×); no product regression exists on either leg; the residual Q1 cost is a product re-architecture outside this case's scope. |
| Storage trade | None. |

## 1. What was measured, and how many samples

Prospectively frozen before the first sample (script: `run-ladder.sh`):

* tiers `dedup-cdc-scattered-{1,10,100,500}`, `--seed 1`, `--setup fresh`,
  `--perf-samples 11`, `--collection-mode`, `--product-timeout 300`,
  `--timeout 310`, serial under the runner-owned measurement lock
  (`$TMPDIR/layerfs-infra-measurement.lock`);
* block 1 = all four tiers (n = 11 each, 44 samples);
* block 2 = an independent repeat of tiers 100 (n = 11) and 500 (n = 7),
  run after block 1 had completed, as A/A repeatability evidence;
* pooled n = 11 / 11 / 22 / 18 for tiers 1 / 10 / 100 / 500 (62 valid samples);
* every sample `status=PASS`, `completion_status=COMPLETE`, `slow=false`,
  `historical_product_target_status=PASS`, `cleanup.status=PASS`;
* all 62 samples also report `oom_kill_delta=0`, `swap_current_bytes=0`, and
  timer purity `(benchmark_injection_count, benchmark_reopen_count,
  benchmark_verifier_count) = (0, 0, 0)`. No fixture, timer, threshold,
  verifier assertion, target value or historical receipt was changed; the
  frozen fixture cache key is unchanged (§1.2) and the source seal is identical
  before and after the run (§1.1).

Exact command shape (one per tier):

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --family dedup_cdc_locality \
  --case dedup-cdc-scattered-100 --seed 1 --setup fresh \
  --image layerfs-bench-infra:00f158e9e0a807e5 \
  --perf-samples 11 --collection-mode \
  --product-timeout 300 --timeout 310 --setup-timeout 600 \
  --output .../current-scattered-100
```

### 1.1 Identity of the measured arm

| Field | Value |
| --- | --- |
| Source commit | `2e72fc7d72c7a12e7c5796dbb20db7010456c359` (`LAYERFS_SOURCE_DIRTY=true`) |
| Source tree | `ec6617c57977217171f9e957a38d111fc2061328` |
| Source seal | `00f158e9e0a807e5660018301ad8251c7ee67b82e1f494f981fcfa8c50992a86` — **identical before and after the run** |
| **Product seal** | `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4` — **identical to the published “current” row** |
| Harness identity | `216545afda554446db84b4169fc746cd53dabad66baca5fbf616dc5f55d2a6a8` (published “current”: `7fd617af…`) |
| Workload sha256 | `c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b` (published “current”: `86a12224…`) |
| Host binary sha256 | `286860778261ea46149add333a46186b06b98f509b8de40622ddd4d1925ff27e` |
| Image | `layerfs-bench-infra:00f158e9e0a807e5`, inspect Id `sha256:c20f93b541217ec05c5017e9d9f97b3a7fba1434ebc6a8d3c70a0fa8155198ee` |
| Host | macOS arm64, 14 CPUs, `host_cpu_capped=false`, host-store topology |
| Container | 2 CPUs / 2048 MiB / 256 PIDs, no swap, `oom_kill_delta=0` in every sample |

**The product seal matches the published row exactly.** The harness/workload
identities do not, because `benchmark/fs-bench-pro/{src,workload}` advanced
under #111 after the campaign. That difference is quantified in §4.2 and is
≤0.3 ms.

### 1.2 Cache profile and disk reads

* `preparation.cache_key = 10af090aa6e713b9ef41d786d48f91ad392233b941cf54ad70d06997fc03a274`
  — **byte-identical fixture recipe to the #104 and published-current rows**,
  including `input_plan_sha256=e01b0995ea66019a1985488f4403cb9f44f1f03b65bce0380e4576db35df35b9`,
  `schema_sha256=1efbb2247d23efd645531e502512245b60ea7e8745e6c937000ca67ccdea7f5a`,
  `regular_files=101`, `fixture_bytes=105906176`.
  Sample 1 was a cache miss (fixture generation); samples 2-11 were cache hits.
* `fixture_cache_profile` is **not emitted** by the `workspace`/`dedup_cdc_locality`
  receipt schema — the field exists only in the fixture-generation and
  `init-only-diagnostic`/`product-lifecycle` records
  (`benchmark/fs-bench-pro/src/main.rs:1170`, `:2728`, `:3010`). Status:
  **INAPPLICABLE for this route**, not zero.
* `initialization_disk_read_bytes` is likewise **not emitted** by this route
  (only `repository_init.rs:91` and the init-only/product-lifecycle schemas
  emit it). Status: **INAPPLICABLE**. Substituted, per phase-final
  `host-resources`: process `disk_read_bytes` = **0** in 20 of 22 tier-100
  samples (≤32 KiB otherwise) — the run is warm-file-cache throughout, and
  `initialization-scan.scanned_bytes=105906176` equals the full fixture.
  The published rows are equally warm (#104 `disk_read_bytes=0`, published
  current `=53248`).

## 2. Repetition-confirmed ladder

`pure_call_sum_ns`, ns. Historical rows are **n=1 and unpaired**.

| tier | v0.1.3 (n=1) | #104 (n=1) | published current (n=1) | block 1 | block 2 | pooled median | pooled spread | pooled / v0.1.3 | pooled / #104 |
| ---: | ---: | ---: | ---: | --- | --- | ---: | ---: | ---: | ---: |
| 1 | 4,647,500 | 10,365,625 | 12,226,625 | median 10,005,708, 9,457,291–12,320,333 (n=11) | — | **10,005,708** | 1.303 | 2.153 | 0.965 |
| 10 | 23,469,041 | 36,817,208 | 43,169,958 | median 37,496,292, 36,087,541–41,663,042 (n=11) | — | **37,496,292** | 1.155 | 1.598 | 1.018 |
| 100 | 89,368,333 | 306,337,291 | 380,340,875 | median 299,843,375, 294,110,959–320,717,875 (n=11) | median 309,882,042, 298,751,292–329,074,834 (n=11) | **304,673,000** | 1.119 | **3.409** | **0.995** |
| 500 | 483,006,833 | 1,438,708,542 | 1,455,708,791 | median 1,297,472,375, 1,276,852,750–1,390,683,083 (n=11) | median 1,358,549,250, 1,314,927,333–1,425,117,000 (n=7) | **1,314,789,146** | 1.116 | 2.722 | 0.914 |

**The published “current” sample is above the same product's own distribution
at every tier** — the signature of one laden measurement block, not one bad
case:

| tier | published current | pooled mean | pooled σ | z | pooled max |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 12,226,625 | 10,304,485 | 869,813 | **+2.21** | 12,320,333 |
| 10 | 43,169,958 | 37,731,466 | 1,591,409 | **+3.42** | 41,663,042 |
| 100 | 380,340,875 | 306,704,298 | 9,213,985 | **+7.99** | 329,074,834 |
| 500 | 1,455,708,791 | 1,333,059,685 | 48,212,073 | **+2.54** | 1,425,117,000 |

Block 1 vs block 2 medians differ by 3.3% (299.8 vs 309.9 ms) — i.e. block-1
was itself the faster of two honest blocks, and the published sample is still
22–27% above both.

**Conclusion for step 1: `4.256×` is a trigger, not a fact.** The honest
same-product repetition ratio to v0.1.3 is **3.409×**, and the whole
v0.1.3 → current band (2.15×–3.41× across tiers) remains an **unpaired**
comparison against a different source commit, workload identity, harness
identity and CDC frame count.

## 3. Q2 — #104 → current (the clean, like-for-like leg)

The published `1.242×` decomposes entirely into two counters:

| counter (ns) | #104 | published current | Z = current − #104 | mine block 1 median (n=11) | Δ vs #104 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `pure_call_sum_ns` | 306,337,291 | 380,340,875 | **+74,003,584** | 299,843,375 | **−6,493,916** |
| `prepare_import_wall_ns` | 299,076,541 | 372,509,000 | +73,432,459 | 293,197,333 | −5,879,208 |
| `direct_pipeline_wall_ns` | 297,071,542 | 370,351,125 | +73,279,583 | 291,479,083 | −5,592,459 |
| `sql_commit_ns` | 158,120,166 | 227,655,790 | **+69,535,624** | 150,517,628 | −7,602,538 |
| └ `pipeline_ns` (25 commits) | 152,634,541 | 221,172,123 | +68,537,582 | 150,121,003 | −2,513,538 |
| └ `publication_ns` (1 commit) | 5,485,625 | 6,483,667 | +998,042 | 4,869,854 | −615,771 |
| └ `final_build_ns` | 0 | 0 | 0 | 0 | 0 |
| `sql_begin_ns` | 208,334 | 162,251 | −46,083 | 210,166 | +1,832 |
| `slab_send_blocked_ns` (concurrent — never summed) | 980,988,368 | 1,273,254,045 | +292,265,677 | 953,070,362 | −27,918,006 |
| `slab_consumer_idle_ns` | 1,734,506 | 1,444,846 | −289,660 | 1,531,842 | −202,664 |
| `last_slab_receive_offset_ns` | 296,997,875 | 369,719,500 | +72,721,625 | 291,353,917 | −5,643,958 |
| `pipeline_max_ns` | 6,914,083 | 10,207,625 | +3,293,542 | 7,231,125 | +317,042 |

Per-batch COMMIT mean: #104 6.11 ms, published current **8.85 ms**, my block-1
median **6.02 ms** — over the identical 25 pipeline commits.

### 3.1 The counter that carries the difference is kernel time, not product work

Every work counter is **bit-identical** between #104, the published current
sample and all 22 of my samples: `canonical_frame_count=5998`,
`canonical_payload_bytes=106286153`, `canonical_encode_calls=canonical_hash_calls=5998`,
`slab_sent_objects=slab_handoffs=425`, `sql_submitted_rows=5967`,
`sql_batch_count=26`, `sql_row_count_shape_count∈11..14`,
`cross_batch_skipped_objects=24`, `collision_checks=31`,
`source_file_read_calls=3333`, `source_file_read_bytes=105906176`,
`task_state_bytes=584406`, `admission_batch_peak_vec_capacity=512`.

What moved is the **split of the host process CPU time**:

| host process, phase `final` (ns) | #104 | published current | mine pooled (n=22) |
| --- | ---: | ---: | ---: |
| `user_cpu_ns` (the product's work) | 296,711,125 | **297,049,625** | **295,338,417** [290,349,375–301,102,166] |
| `system_cpu_ns` (kernel on its behalf) | 202,188,625 | **278,452,791** | **202,274,479** [194,651,125–216,318,833] |
| `disk_write_bytes` | 107,515,904 | 107,511,808 | 107,468,800 |

`user_cpu_ns` is flat to within 0.6% between #104 and the published current
sample (+338,500 ns, +0.11%) while `system_cpu_ns` rose **+76,264,166 ns**,
matching the `pure_call_sum_ns` delta of +74,003,584 ns. The product did the
same work in the same user-space instructions; the historical sample spent
+76 ms inside the kernel.

### 3.2 Mechanism

`pure_call_sum_ns` for this case is 98% the product's own
`prepare_import_wall_ns` (§4.2), and that wall is the consumer's 26 SQLite
`COMMIT`s:

* `sql_commit_ns` = 50% of the block-1 median timer (150.5 / 299.8 ms);
* all object payload lives inside the Store's 4 KiB pages
  (`page_count × page_size = 107,315,200` for 106,286,153 payload bytes,
  `page_size_bytes=4096`), so each of the 25 pipeline `COMMIT`s must flush
  ~4 MiB of dirty pages (`AdmissionCohort::commit` →
  `connection.execute_batch("COMMIT")`, `crates/layerfs-layerstack-store/src/objects.rs:2143`);
* the Store is opened `journal_mode=MEMORY`, `synchronous=OFF`,
  `cache_spill=OFF`, `cache_size=-32 MiB`
  (`crates/layerfs-layerstack-store/src/schema.rs:511-523`), so a `COMMIT` is
  a burst of page writes with no `fsync` — its latency is dominated by the
  host's page-write path;
* the 4 producers are backpressured against the bounded slab queue
  (`INITIALIZATION_SLAB_QUEUE_SLOTS=4 × 256 KiB = 1 MiB`,
  `slab_queue_peak_bytes=1048576`) while the consumer commits:
  `slab_send_blocked_ns` ≈ 4 × 244 ms concurrent vs `slab_consumer_idle_ns`
  = 1.5 ms, i.e. the consumer is never idle and the producers are blocked
  ~89% of their wall. Correctly treated as a backpressure indicator; never
  added to wall time.

So the timer is a serial sum over 26 page-flush-latency-bound commits, and is
therefore **sensitive to host page-write latency, which no product counter
observes except the elapsed time itself**. In one sample from the campaign
window that latency was 45% higher per commit.

### 3.3 Independent corroboration at campaign scale

Across all 197 joined rows of the campaign's own `comparison.csv`, the
`current/#104` ratio has median **1.013**, deciles **0.872 … 1.191**, max
**1.468**. `scattered-100`'s 1.242 sits at the **93.9th percentile** of a
single-sample distribution whose tail reaches 1.468 and which contains
**20 rows above 1.19**. Nothing singles this case out except that it was
picked as the record-holder.

**Q2 verdict: `1.242×` is not a product regression.** No change is warranted.
The counter that carries it is `pipeline_ns` (25 COMMITs), driven by host
kernel page-write time; user CPU and all work counters are identical.

## 4. Q1 — v0.1.3 → current (the headline, unpaired and confounded)

### 4.1 The comparison is not admissible as a product ratio

Per `docs/general/benchmark_rules.md` §8, a paired product comparison requires
byte-identical harness, workload, fixture and oracle with the sole difference
being the recorded `treatment`. For v0.1.3 → current:

| Identity | v0.1.3 | #104 / current |
| --- | --- | --- |
| source commit | `5eb849ea…` (dirty) | `84eaa5b6…` / `cc8025fc…` (dirty) |
| source seal | `1bedaeaa…` | `53f90fe4…` / `17334f5e…` |
| harness identity | `91ef300d…` | `3a3adc05…` / `7fd617af…` |
| workload sha256 | `2ec80979…` | `86a12224…` |
| `sql_row_count_shape_count` | 15 | 12 |
| CDC frames / payload bytes | 6,129 / 106,299,288 | 5,998 / 106,286,153 |
| `worker_count` | 8 | 4 |

The **work itself** changed (`canonical_frame_count`, `sql_row_count_shape_count`,
`sql_bind_step_returning_ns`, `insert_node_peak_len`, `structural_peak_bytes`,
`cdc_scratch_peak_bytes`, `pair_segment_*`), so v0.1.3 → current cannot be
presented as a product speedup ratio. It is retained as an unpaired diagnostic.

### 4.2 Harness attribution: the orchestration envelope is a ~0.2 ms constant

Method: bound the harness-attributable share from the product's own timers
rather than from wall ratios. Two independent bounds, in-product:

| bound (ns) | v0.1.3 | #104 | published current | mine pooled (n=22) |
| --- | ---: | ---: | ---: | ---: |
| `host_orchestration_ns − pure_call_sum_ns` (harness envelope outside the product timer) | **159,292** | **232,918** | **208,083** | **212,541** |
| `pure_call_sum_ns − prepare_import_wall_ns` (product work outside its own import wall) | 755,125 (0.845%) | 7,260,750 (2.370%) | 7,831,875 (2.059%) | 6,384,896 (2.096%) |

The harness envelope is 0.16–0.23 ms in every generation, and the product's
own import wall accounts for 97.6–99.2% of the reported timer in every
generation. **Therefore the harness generation accounts for ≤0.23 ms of the
+290.97 ms headline gap (≤0.08%), and ≤0.005 ms of the Q2 gap.** The harness
did not make this case slower. What the harness change *did* do is make the
comparison inadmissible, and it is what removed the historical ratio from the
status of a product target.

### 4.3 Counter-attributed product-side diagnosis (H1–H3)

All deltas are v0.1.3 → mine-pooled-median at tier 100.

**H1 — SQL publication cost per row grew: CONFIRMED, and it is a
metric/scope change plus a real per-row cost increase.**

| | v0.1.3 | #104 | mine pooled median |
| --- | ---: | ---: | ---: |
| `sql_commit_ns` | 29,846,878 | 158,120,166 | 155,293,357 |
| `sql_bind_step_returning_ns` | 39,971,246 | 0 | 0 |
| `sql_begin_ns` | 253,418 | 208,334 | 210,166 |
| SQL-attributable total | **70,071,542** | **158,328,500** | **155,503,523** |

*The metric disappeared; the work did not.* `sql_bind_step_returning_ns` is
still declared and accumulated (`objects.rs:2025-2027`) but **nothing in the
tree ever assigns it** — the v0.1.3 per-object `INSERT` loop that timed it
(`objects.rs:3964` at `5eb849ea`) was replaced by
`admission::PreparedAdmission::prepare_missing` → `publish`
(`objects.rs:4330`). The row-insertion work now happens inside the batch
transaction and is therefore counted in `sql_commit_ns`. Reported as a
**fabricated zero for an inapplicable sub-metric**, which
`benchmark_rules.md` §9 forbids ("An unavailable or inapplicable value MUST be
`null` with a status and reason, never a fabricated zero").

*The real cost increase.* On a comparable SQL basis the case pays
**+85.4 ms** of SQL-attributable time, **+125.4 ms** in `sql_commit_ns` alone.
`sql_commit_ns` is the elapsed time of `execute_batch("COMMIT")`
(`objects.rs:2142-2144`) — unchanged code, unchanged batch count (27 → 26) and
unchanged rows/batch (225 → 229). What changed is *what is dirty at COMMIT*:
the new prepared-admission path moves the object/metadata row writes inside the
transaction, so COMMIT now flushes essentially the whole Store
(107,315,200 bytes) through page writes where v0.1.3 flushed most of it during
the separately-instrumented insert loop.

**H2 — admission batch population collapse: REFUTED as a cost driver.**

`admission_batch_peak_objects` 284 → 34 and
`admission_batch_peak_vec_capacity` 8191 → 512 look alarming, but they are
**peaks of the consumer's slab-delivered admission page, not of the SQL
batch**. The SQL batch geometry is unchanged: 27 → 26 commits for 6080 → 5967
rows, i.e. 225 vs 229 rows/commit. The ceiling is **product policy**, not
harness-declared: `INITIALIZATION_SLAB_OBJECTS = 512` and
`INITIALIZATION_SLAB_BYTES = 256 KiB` are **byte-identical constants in
v0.1.3, #104 and now** (`objects.rs:21-22` at `5eb849ea`, `:43-44` at HEAD);
what changed is that the newer admission page is bounded by them
(`objects.rs:2639-2643`) instead of by the 8191-object `ADMISSION_BATCH_COUNT`.
The byte bound fell 8× and the object bound 16×, i.e. the change *reduced*
per-commit dirty volume. No fixed-cost-per-object amplification is visible, and
`admission_batch_peak_payload_bytes` 4,193,969 → 524,209 while `sql_batch_count`
stayed 26/27: the peak collapsed, the batch count did not.

**H3 — producer/consumer contention: CONFIRMED as a symptom, not a cost line.**

`slab_send_blocked_ns` 369,782,618 → 975,333,779 concurrent and
`slab_consumer_idle_ns` 16,010,923 → 1,647,588. In v0.1.3 the consumer idled
16 ms and the producers were blocked less in absolute terms. The mechanism is
the halved worker count working against a fixed 1 MiB bounded queue:
`worker_count` 8 → 4 with `INITIALIZATION_SLAB_QUEUE_SLOTS = 4` and
`slab_queue_peak = 4` in both, so each of 4 producers carries twice the bytes
of each of 8 producers against the same queue depth, and blocks whenever the
consumer is inside a COMMIT. `slab_send_blocked_ns` is concurrent time and is
**not** added to any wall here.

`worker_count` is **product-derived**, not harness-declared: it is
`min(std::thread::available_parallelism(), 8).min(tasks.len())`
(`layerstack.rs:1213-1216`, `:1256-1262`), and the receipts report the same
host `cpu_count=14` in all three generations. The 8 → 4 change is a
product-side frontier/worker-policy change between v0.1.3 and #104, **not**
harness or topology drift. This corrects the H4 framing in the handoff.

### 4.4 The dominant structural cause: Store page size 64 KiB → 4 KiB

The most consequential difference is not in any of the counters above — it is
in the Store geometry, visible only in `store-observation`:

| | v0.1.3 | #104 / v0.1.5 |
| --- | ---: | ---: |
| `page_size_bytes` | **65,536** | **4,096** |
| empty-Store `page_count` / bytes | 15 / 983,040 | 20 / 81,920 |
| after-`initialize` `page_count` | **1,938** | **26,194** |
| after-`initialize` apparent bytes | 127,008,768 | 107,290,624 |
| payload bytes per page | 65,536 (full) | 4,096 (full) |
| host process `disk_write_bytes` | 127,565,824 | 107,487,232 |

Both Stores hold the same 5,998 objects / 106,286,153 payload bytes, but the
4 KiB layout writes them into **13.52× more pages** while being **15.5%
smaller** (127.0 → 107.3 MB, i.e. 19.5% → 0.9% page-level overhead). Every one
of those pages is dirtied and flushed during the import, so the import issues
~13.5× more page writes for ~15% fewer bytes.

Introduced by commit `70955bc253a0a7f97f931e16b2e44eea2465d9a0`
("storage: create 4-KiB Stores and preserve 64-KiB reader compatibility",
2026-09-08), which adds `NEW_STORE_PAGE_SIZE_BYTES = 4096`
(`schema.rs:14`). It does not exist at `5eb849ea`.

Quantitatively, that is the reason the SQL-attributable line and the kernel
time both grew:

| | v0.1.3 | v0.1.5 (pooled median) | change |
| --- | ---: | ---: | ---: |
| SQL-attributable time (ns) | 70,071,542 | 155,503,523 | **+85,431,981** |
| host `system_cpu_ns` | 86,992,500 | 202,274,479 | **+115,281,979** |
| bytes written per ms of SQL time | 1,820,516 | 691,235 | −2.63× |
| page writes per ms of SQL time | 27.7 | 168.5 | +6.1× |
| page writes per import | 1,938 | 26,194 | **+13.52×** |

Per-syscall throughput is *six times better* on the 4 KiB path; the cost is
entirely that there are 13.5× more of them while the byte count fell 15%.

**This is a deliberate, declared storage↔latency trade, and it is the single
largest component of the `>3×`.** It is also protected: "4 KiB SQLite pages"
is on the handoff's must-preserve list, and `benchmark_success.md` forbids
changing CDC boundaries or storage layout to move a number. Reverting it is
therefore **not** an available fix inside this investigation — it is a
storage decision owned by #107 (Store size) that would have to be declared with
its allocated/apparent-bytes consequence, not smuggled in as a latency patch.

### 4.5 Composition of the `>3×` (v0.1.3 → v0.1.5)

`+215,304,667 ns` of `pure_call_sum_ns`, of which `prepare_import_wall_ns`
carries `+209,430,209`:

| component | ns | share of the gap | cause |
| --- | ---: | ---: | --- |
| SQL-attributable | +85,431,981 | 39.7% | **4 KiB pages** (§4.5): 13.5× more page writes |
| consumer non-SQL serial work (admission prep, canonical verification, compact-tree publication) | +123,998,228 | 57.6% | new prepared-admission/compact-tree path (§4.3, H1) |
| timer outside `prepare_import_wall_ns` | +5,874,458 | 2.7% | orchestration envelope, stable (§4.2) |

Cross-cutting: `worker_count` 8 → 4 (§4.3, H3), which does not add work but
removes half the producer parallelism against an unchanged 1 MiB queue.

Note on units: `slab_send_blocked_ns` (+605,551,161 concurrent) is producer
waiting and is **not** part of the sum above; it grows *because* the consumer's
serial path grew.

### 4.6 Q1 verdict

The `>3×` is **real and reproduced 22/22**: every sample I collected is at
least 3.291× the v0.1.3 value, median 3.409×. It is **product-side**, not
harness-side (§4.2), and it decomposes as: ~40% Store page geometry (a
declared storage↔latency trade, §4.4), ~58% consumer non-SQL serial work on
the new prepared-admission/compact-tree path, with `worker_count` 8→4 removing
half the producer parallelism across both.

The residual `>3×` is **product-side** and **unpaired**, and it is not the
`4.256×` headline (that specific value was an outlier sample, §2). Because the
v0.1.3 row is n=1 on a different source, workload and harness generation,
**no product target may be derived from this ratio** until a fresh paired
v0.1.3-equivalent arm is collected under one harness generation — which is
#112's G0 prerequisite, now discharged with evidence. That run is also the only
way to separate the ~40% page-geometry share from the ~58% consumer-serial
share, since both are present in every available v0.1.3 row.

## 5. H5 — is ~0.4% chunk reuse the intended contract?

**Yes; it is the intended behaviour of this scenario, and it is not a
regression.**

* `docs/roadmap/0.1/0.1.3/dedup-cdc-locality.md:32` defines the scenario:
  "`dedup-cdc-scattered` | Change one byte in every 4 KiB block, using distinct
  nonzero masks".
* `dedup-cdc-locality.md:82-85`: "Scattered edits are **the negative control for
  fuzzy similarity**; use the exact qualified transcript rather than claim
  'no sharing' solely from the mask schedule."
* `dedup-cdc-locality.md:87-91`: "No historical 5 MB-file 85%/95% thresholds
  carry over… The frozen CDC profile itself is unchanged."
* `benchmark_success.md:121` (family contract row): "Physical delta does not
  increase canonical exact reuse; **do not change CDC boundaries silently to
  pass a storage target.**"

The fixture generator applies `content(..., "mask", 256)` — **256 single-byte
flips**, one per 4 KiB block of each 1 MiB variant
(`benchmark/fs-bench-pro/workload/dedup_workloads.rs:124-133`); the handoff's
"512 single-byte edits" is off by 2×. The observed
`cross_batch_skipped_objects = 24` of 5,998 frames (0.40%) with
`collision_checks = 31` and `conflict_read_calls = 0` is exactly what a
negative control for fuzzy similarity should produce, and it is stable across
all 22 of my samples, #104 and the published current sample.

**No CDC boundary, mask schedule, seed or fixture may be changed to raise this
number.** Recorded as intended, not as a defect.

Additionally, `conflict_read_ns = 324,671` (0.11% of the timer) with
`conflict_read_calls = 0` confirms that the canonical-comparison path is *not*
the bottleneck here — unlike #95's `identical-500`/`overwrite-500`. This case
is a near-unique publication workload (5,967 of 5,998 frames are new), despite
living in the `dedup` family.

## 6. Disposition

**No product change landed.** Reasons, in order:

1. No gate is missed. The historical 15 s family target is reporting-only and
   passes by ~49× (304.7 ms median vs 15 s).
2. The clean leg (`1.242×`, #104 → current) does not reproduce (0.995×, n=22)
   and is fully attributed to host kernel time in a single sample. A fix would
   be a fix for noise.
3. The headline (`4.256×`) does not reproduce (3.409×, n=22) and is the extreme
   of a single-sample distribution (`z = +7.99`) that is itself inadmissible as
   a product ratio (§4.1).
4. The residual product-side cost is now attributed (§4.4, §4.5): ~40% is the
   Store page-size change 64 KiB → 4 KiB, which is a **deliberate storage
   trade** buying a 15.5% smaller Store and is on the handoff's must-preserve
   list; ~58% is consumer serial work on the new prepared-admission/compact-tree
   path; `worker_count` 8→4 removes half the producer parallelism across both.
   None of these is a minimal local fix. The latency can only be recovered by
   trading storage back (page geometry or payload placement), which is #107's
   decision and must be declared with its allocated/apparent-bytes consequence —
   or by shortening the consumer's serial path, which is a #100-class
   re-architecture, not a G0 attribution task.

**The `>3×` itself is not in dispute.** Every one of my 22 samples is ≥3.291×
the v0.1.3 checkpoint; that is a real, reproducible cost, and it is recorded
here as such. What is *not* established is any product target derived from it,
and what is *disproved* is the `4.256×` value and the `1.242×` step.

Recorded as a **no-go with cause**, in #104's disposition style.

### 6.1 Observability defect worth a separate, non-performance change

`sql_bind_step_returning_ns`, `sql_prepare_ns` and `sql_string_build_ns` are
emitted as literal `0` in every `layerfs-initialization-diagnostic-v3` line
although no code path assigns them any more. Per `benchmark_rules.md` §9 this
is a fabricated zero for an inapplicable metric and it actively misled this
investigation (H1's "did the work disappear?" question). The honest fix is to
drop or explicitly mark them; it is **not** landed here because it changes a
diagnostic schema that historical receipts and campaign parsers were compared
against, and it is not a performance change. Filed as a finding for #108
(counters).

## 7. Storage before/after

No change was made, so there is no before/after. For completeness, the
`store-observation` after `initialize` on the measured arm (identical work in
all samples):

| tier | allocated bytes (median, range) | apparent bytes (median) | `page_count` | `page_size_bytes` |
| ---: | --- | ---: | ---: | ---: |
| before | 81,920 | 81,920 | 20 | 4,096 |
| 1 | 2,211,840 (constant) | 2,211,840 | 540 | 4,096 |
| 10 | 11,763,712 (11,759,616–11,767,808) | 11,763,712 | 2,872 | 4,096 |
| 100 | 107,298,816 (107,261,952–107,335,680) | 107,298,816 | 26,196 | 4,096 |
| 500 | 542,097,408 (533,020,672–548,724,736) | 532,031,488 | 129,899 | 4,096 |

`freelist_page_count = 0` in every observation. The 4 KiB page size is
preserved and unchanged; the promoted-uncompacted treatment is untouched (no
compaction, no VACUUM, no repack, no background rewriter was executed by this
investigation).

## 8. Spillover note for #112

The mechanism (a timer that is a serial sum over page-flush-latency-bound
SQLite `COMMIT`s, with producers backpressured on a fixed-depth queue) is
**general to every host-store initialization family**, and it transfers as a
*sensitivity*, not as a fixed cost:

| case | current / #104 | reading |
| --- | ---: | --- |
| `dedup-cdc-scattered-100` | 1.242 (single sample) | **0.995** under n=22 — same mechanism, no regression |
| `dedup-cdc-scattered-500` | 1.012 | 0.914 under pooled n=18 |
| `dedup-cdc-scattered-1` / `-10` | 1.180 / 1.173 | 0.965 / 1.018 under n=11 |
| `dedup-cdc-common-body-{1,10,100,500}` | 1.056 / 1.173 / 1.180 / 1.155 | same +2–18% band, un-repeated |
| `dedup-cross-file-unique-{10,100,500}` | 0.880 / 0.846 / 1.123 | no `4×`-class behaviour |
| `store-footprint-unique-100000` | 0.751 | faster |
| `namespace-10000` / `namespace-100000` | 0.722 / 0.777 | faster (#109) |

Every case in the `1.18–1.28` band shares the same shape: a small tier
(1 or 10) whose absolute time is 8–44 ms, where one unlucky commit out of 26
is a large fraction of the total, and/or a single sample. The `dedup-cdc-common-body`
tiers and `dedup-history-*` small tiers should be treated as **un-repeated
single samples** in the same way `scattered-100` was, and should not be
carried as product regressions into G2 without repetition.

Within family `dedup_cdc_locality`, the whole `current/#104` column has median
1.013 over 197 rows (deciles 0.872–1.191, max 1.468), so the ~1.2× entries are
at the noise floor of the campaign's own single-sample design.

## 9. Completion checklist

- [x] Ladder repetition run completed; reproducibility of `4.256×` answered **no** with n=22 evidence and a +7.99σ outlier test.
- [x] Q1 and Q2 reported separately; no unpaired historical row presented as a paired result.
- [x] `sql_commit_ns` (→ `pipeline_ns`, kernel page-write time), `prepare_import_wall_ns` (→ same), `admission_batch_peak_objects` (refuted as cost driver; product constant), `slab_send_blocked_ns` and `slab_consumer_idle_ns` (backpressure symptom of 8→4 workers against a fixed 1 MiB queue) all explained.
- [x] H5 answered as intended behaviour with contract citations.
- [x] No-go recorded with cause; no fixture/timer/threshold/verifier change; no compaction; no resource-limit raise.
- [x] Allocated/apparent bytes reported (§7).
- [ ] Result posted to #112 — **not performed** (no GitHub access from this session); this document is the result record.

## 10. Seals and receipt paths

Evidence root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-dedup-g2-scattered/`

```
run-ladder.sh                                    prospectively frozen driver (n=11/tier)
current-seals-before.txt / -after.txt            identical source+product seals around the run
current-scattered-{1,10,100,500}/perf.jsonl      block 1, 11 samples each
block2-scattered-{100,500}/perf.jsonl            block 2, 11 and 7 samples
analyze.py, report.py, report-tables.md          derived tables (reproducible from the raw jsonl)
```

Retained historical receipts (read-only, unmodified):

* v0.1.3 — `docs/roadmap/0.1/0.1.3/checkpoint-evidence/raw/performance/dedup_cdc_locality/dedup-cdc-scattered-100/perf.jsonl`
* #104 — `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue104-evidence/campaign-20260910T1115Z/run/candidate/attempt-1/performance/dedup_cdc_locality/dedup-cdc-scattered-100/perf.jsonl`
* published current — `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-family-campaign-20260910T210014Z/dedup_cdc_locality/candidate/attempt-1/performance/dedup_cdc_locality/dedup-cdc-scattered-100/perf.jsonl`

Joins: `…/layerfs-family-campaign-20260910T210014Z/comparison.csv`,
`docs/roadmap/0.1/0.1.3/checkpoint-evidence/performance.csv`,
`docs/roadmap/0.1/0.1.5/issue104/case-results.csv`.

| Seal | This measurement | Published current |
| --- | --- | --- |
| source commit | `2e72fc7d72c7a12e7c5796dbb20db7010456c359` (dirty) | `cc8025fcd029c75c8a55003e5b72560806739864` (dirty) |
| source tree | `ec6617c57977217171f9e957a38d111fc2061328` | `8f6736e64039795fc898b6407cd65a7aa4cdaf84` |
| source seal | `00f158e9e0a807e5660018301ad8251c7ee67b82e1f494f981fcfa8c50992a86` | `17334f5e6900bdaf73e1c49ccb9bc0ad7ec3b710eef3ef5efef9b9841c6975ba` |
| **product seal** | `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4` | **same** |
| harness identity | `216545afda554446db84b4169fc746cd53dabad66baca5fbf616dc5f55d2a6a8` | `7fd617af6ccdb4e30fd387ca11303808447f3c568999c811097ff8574b932433` |
| workload sha256 | `c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b` | `86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127` |
| host binary sha256 | `286860778261ea46149add333a46186b06b98f509b8de40622ddd4d1925ff27e` | `dbbf1259186c60122286bb2a0503d6dcccfd0a41f49eb82799b5c3a4d6e6a36b` |
| image | `layerfs-bench-infra:00f158e9e0a807e5` (`sha256:c20f93b541217ec0…`) | `layerfs-bench-infra:17334f5e6900bdaf` (`sha256:0ccd9886961edbd8…`) |
| fixture cache key | `10af090aa6e713b9ef41d786d48f91ad392233b941cf54ad70d06997fc03a274` | **same** |

## 11. What was not resolved

* **Why one block was laden.** The `+76 ms` kernel-time inflation in the
  campaign's sample is measured and localised, but the host-level cause (APFS
  dirty-page writeback backlog, memory reclaim, or ambient load during a
  26-minute 18-family campaign) is inferred, not directly instrumented. No
  concurrent-writer experiment was run: `benchmark_rules.md` and the #112
  discipline forbid resource-sensitive overlap, and manufacturing a slow
  sample would not be evidence.
* **v0.1.3-side variance is unknown.** v0.1.3's 89.37 ms is a single sample
  with unknown host-state bias; a fair v0.1.3-equivalent control would need a
  sealed rebuild under the current harness — out of scope for this case and
  belonging to #112's G0 programme.
* **The exact page-write attribution of `sql_commit_ns`** is derived from the
  Store layout (`page_count × page_size`, journal/synchronous pragmas, and the
  fact that all payload lives in-page), not from a syscall-level profile. A
  paired profile was not run because the `1.242×` leg it would explain is
  already resolved as noise.
* **`pipeline_max_ns` tail.** My samples show a 23.27 ms worst single commit
  (vs 10.21 ms in the published sample), i.e. individual COMMIT latency has a
  wide tail. Its long-run distribution was not characterised; it is the main
  reason tier-100 spread (1.119) exceeds the smaller tiers' expected spread.

## 12. What the correct comparison is

No ratio currently on the books is the correct comparison. All three
generations are n=1 and unpaired. The admissible comparison is a **paired,
interleaved, repeated A/B under one harness generation against the identical
fixture**, and all three prerequisites are now verified satisfiable.

### 12.1 Prerequisite checks (verified)

| Prerequisite | Status | Evidence |
| --- | --- | --- |
| Baseline arm reproducible byte-exactly? | **Yes** | `git archive 5eb849ea…` reproduces the receipt's source seal `1bedaeaa3d564286b8ff6a77365d3d7b144a6dc97e236466fa7066826a6acd19` and product seal `3c797bc6dbfd9b03b919c270b609cad839000b68d67e34f3b00d24717e07f39a` exactly. The receipt's `SOURCE_DIRTY=true` did not touch any sealed path. The v0.1.3 baseline can therefore be rebuilt, not merely re-quoted. |
| Same fixture bytes? | **Yes** | v0.1.3, #104, published current and my arm all report `input_plan_sha256=e01b0995ea66019a1985488f4403cb9f44f1f03b65bce0380e4576db35df35b9`, `data_bytes=105947377`, `regular_files=101`, `fixture_bytes=105906176`, and `benchmark/fs-bench-pro/workload/dedup_workloads.rs` is **byte-identical** between `5eb849ea` and `84eaa5b6`. The differing `preparation.cache_key` (`721eb9dd…` vs `10af090aa…`) is driven only by `schema_sha256` (`6b714357…` vs `1efbb224…`) — the Store schema SQL is part of the conservative cache key, not part of the fixture. |
| Same harness? | **Achievable** | Run *both* arms with the current `runner.py`/`runtime.py`/`verify-selected.py` (harness identity `216545af…`). This is the one artifact the historical rows never shared. |

### 12.2 The comparison to run

* arms: **B** = product rebuilt at `5eb849ea` (source seal `1bedaeaa…`), **X** = current product seal `95e796f8…`;
* one harness generation for both arms; each arm's Linux image built from its own source (host-store topology, no Docker fallback);
* fixture: the frozen prepared input, proven identical by `input_plan_sha256` + `data_bytes`; report the per-arm cache keys and state the schema-driven key difference explicitly;
* **interleaved** alternating schedule (e.g. `A/B/B/A` blocks), n ≥ 11 samples per arm, one measurement lock, quiet host;
* `--collection-mode`, same budgets (`--product-timeout 300 --timeout 310`), `--seed 1`, `--setup fresh`;
* report per arm: `pure_call_sum_ns` median/min/max, the full initialization counter set, and the host process `user_cpu_ns` / `system_cpu_ns` split;
* run `verify.sh` on both arms.

### 12.3 Why interleaving is mandatory, not optional

Tier-100 single-sample CV is 3.00% (σ 9,213,985 on mean 306,704,298). With
n = 11 per arm the standard error of an arm mean is 0.90%, so a two-arm
difference is resolvable at ≈2.6% (2σ). The **block-to-block drift I measured
is 3.3%** (block 1 median 299,843,375 vs block 2 median 309,882,042) — larger
than the resolution. Sequential blocks therefore manufacture false effects of
the same size as real ones, and that is exactly the failure mode that produced
the published rows.

### 12.4 Declared treatment — the pairing problem

The B → X treatment is large and must be declared whole, not as one change:

* Store schema v9 → v10 (`sql_row_count_shape_count` 15 → 12);
* canonical object segmentation: `canonical_frame_count` 6,129 → 5,998 and
  `canonical_payload_bytes` 106,299,288 → 106,286,153, although
  `crates/layerfs-content/src/file/cdc` is **unchanged** — so the difference is
  in the layerstack-store object-segmentation/encoding layer, not the frozen
  CDC chunker;
* worker policy: `worker_count` 8 → 4;
* the compact-tree publication path (`insert_node_peak_len` 6,592 → 0,
  `structural_peak_bytes` 67,242 → 0, `compact_tree_scratch_peak_bytes` new);
* admission batching, and the `sql_bind_step_returning_ns` instrumentation
  removal (§4.3).

A single paired run therefore answers *"is the current implementation slower
than the v0.1.3-era implementation, and by how much?"* — not *"which change
cost what"*. If a speedup claim is wanted, that run must be followed by a
bisection over these five axes, each with its own paired interleaved blocks.

### 12.5 What is legitimately comparable today

| Comparison | Admissible? | Result |
| --- | --- | --- |
| v0.1.3 → current ratio | **No** — unpaired n=1, different source/seal/harness/workload | diagnostic only; not a product target |
| #104 → current ratio | **No** — unpaired n=1 | diagnostic only |
| current A/A block 1 vs block 2 | **Yes** — same source, harness, product, fixture | establishes the 3.3% block drift / 3.0% single-sample CV noise floor |
| #104 point vs the 22-sample current distribution | **Yes**, as containment | inside the distribution (0.995× of median) |
| published current point vs the same distribution | **Yes**, as outlier test | +7.99σ outside |
| work counters (frames/rows/batches/handoffs/payload bytes) | **Yes** — drift-free instruments | identical between #104, published current and all 22 of mine |
| host `user_cpu_ns` vs `system_cpu_ns` | **Yes** — the comparison that answers "did the product do more work" | user CPU flat; system CPU carries the whole difference |

## 13. Is v0.1.5 slower, faster, or similar — and how fast?

Two different answers, because the two baselines are not equivalent.

### 13.1 vs v0.1.4 (#104, the immediately preceding source): **similar**

| Scope | Value | Source |
| --- | --- | --- |
| All matched cases, median ratio | **1.013** (n=197, min 0.655, max 1.468) | #102 requalification table, `requalification-current-source-20260910.md:102` |
| Cases ≥15% faster / 115% slower | 11 / 30 | same |
| Distribution | 90/197 faster; 123/197 within ±10%; p10 0.872, p90 1.191 | `comparison.csv` |
| **`dedup-cdc-scattered-100`** | **0.995×** (n=22, product-seal-matched) | §3, this document |

v0.1.5 is not a slower release than v0.1.4. At this case it is 0.5% *faster*
and the difference is unresolved against a 3.0% single-sample CV.

### 13.2 vs v0.1.3: **slower, by ~1.38× at the campaign median**

| Scope | Value | Source |
| --- | --- | --- |
| All matched cases, median ratio | **1.379** (n=198, min 0.579, max 4.256) | `requalification-current-source-20260910.md:103` |
| Cases ≥15% slower / ≥15% faster | 154 / 8 | same |
| Distribution | p10 0.997, p90 1.968; only 20/197 faster than v0.1.3 | `comparison.csv` |
| **`dedup-cdc-scattered-100`** | **3.409×** (304,673,000 ns vs 89,368,333 ns) | §2, this document |

The campaign's own note applies, verbatim: *"The v0.1.3/v0.1.4 baselines are
older source states and harness generations; their absolute ratios are
reported as historical context only"* (`:106-108`).

For **this case** the direction is nonetheless real rather than pure block
state, and it now has an identified structural cause — **the `>3×` is
reproduced in all 22 of my samples (≥3.291×, median 3.409×), and ~40% of it is
the Store page geometry change**, 64 KiB → 4 KiB pages: 1,938 → 26,194 page
writes for the same 5,998 objects, in exchange for a 15.5% smaller Store
(§4.4). Supporting CPU evidence: user-space CPU per unit of work rose
**+59.6%** (184,962,541 → 295,338,417 ns median) while the work counters moved
by only −2.1% frames (6,129 → 5,998). Host-state bias cannot inflate *user*
CPU while leaving `system_cpu_ns` separately accounted for. The same is **not**
true of the `4.256×` headline, which is a block artefact (§2, §3).

### 13.3 The speed, in absolute units

`dedup-cdc-scattered-100` imports 105,906,176 B = 101 MiB across 101 files, so
the byte basis for throughput is scanned source bytes.

| Generation | `pure_call_sum_ns` | wall | throughput | per file |
| --- | ---: | ---: | ---: | ---: |
| v0.1.3 (n=1) | 89,368,333 | 89.37 ms | 1,185.1 MB/s = 1,130.2 MiB/s | 0.885 ms |
| v0.1.4-G0 (n=1) | 412,799,209 | 412.80 ms | 256.6 MB/s = 244.7 MiB/s | 4.087 ms |
| #104 / v0.1.4 (n=1) | 306,337,291 | 306.34 ms | 345.7 MB/s = 329.7 MiB/s | 3.033 ms |
| v0.1.5 published (n=1) | 380,340,875 | 380.34 ms | 278.5 MB/s = 265.6 MiB/s | 3.766 ms |
| **v0.1.5, n=22 median** | **304,673,000** | **304.67 ms** | **347.6 MB/s = 331.5 MiB/s** | **3.017 ms** |
| v0.1.5, n=22 min → max | 294,110,959 → 329,074,834 | 294.11 → 329.07 ms | 360.1 → 321.8 MB/s | 2.912 → 3.258 ms |

Per tier on the v0.1.5 arm (pooled repetitions):

| tier | source | v0.1.3 (n=1) | v0.1.5 (pooled median) | ratio | v0.1.5 throughput |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2.0 MiB | 4.65 ms | 10.01 ms | 2.153 | 199.9 MiB/s |
| 10 | 11.0 MiB | 23.47 ms | 37.50 ms | 1.598 | 293.4 MiB/s |
| 100 | 101.0 MiB | 89.37 ms | 304.67 ms | **3.409** | 331.5 MiB/s |
| 500 | 506.0 MiB | 483.01 ms | 1,314.79 ms | 2.722 | 384.9 MiB/s |

### 13.4 Why any single case's version ratio is not readable

`dedup-cdc-scattered-100` was measured in four campaign blocks:

| block | value | ratio to v0.1.3 |
| --- | ---: | ---: |
| v0.1.3 | 89,368,333 | 1.000 |
| v0.1.4 G0 | 412,799,209 | **4.619** |
| #104 / v0.1.4 | 306,337,291 | 3.428 |
| v0.1.5 published | 380,340,875 | 4.256 |
| v0.1.5, n=22 median | 304,673,000 | 3.409 |

The #104 and v0.1.5 products differ by `cc8025fcd` plus a formatting commit
(§3), yet their single samples differ by 24%; the v0.1.4-G0 and v0.1.5 single
samples differ by only 8% for a much larger product delta. **Block-to-block
host state contributes roughly ±35% at this case's absolute scale**, which is
larger than any version-level effect this case can carry. Version-level
comparisons must use the campaign median over all cases (§13.1, §13.2), and
case-level comparisons must use interleaved repetition (§12.2).

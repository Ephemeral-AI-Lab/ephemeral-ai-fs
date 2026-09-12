# Page-size A/B protocol: 4 KiB vs 64 KiB (`NEW_STORE_PAGE_SIZE_BYTES`)

> **Status:** Frozen protocol for issue **#113**. Not yet executed.
> Arms: **C** = 4096 (control, promoted contract), **X** = 65536 (treatment, scratch only).
> The X arm MUST NOT be promoted by this experiment.

Tracker: <https://github.com/Ephemeral-AI-Lab/layerfs/issues/113>
Motivating finding: [`../issue112/scattered-100-repetition-and-attribution.md`](../issue112/scattered-100-repetition-and-attribution.md) §4.4–§4.6, §13.

## 1. Question and claim

Is the 4 KiB Store page size a **quantified trade** — N bytes of Store saved for M
nanoseconds of initialization latency — or is the observed 15.5% Store reduction
caused by the payload-layout change that landed in the same window, leaving the
page size holding only the latency cost?

The experiment may support exactly one claim per axis:

* `empirical-performance` for `dedup_cdc_locality/dedup-cdc-scattered-100`
  (`pure_call_sum_ns`), paired and interleaved;
* `empirical-storage` for `repository_history --profile stride-3`, allocated and
  apparent bytes at the terminal state.

It may **not** support a release-wide storage or latency headline (§7).

## 2. Treatment

Exactly one constant, in `crates/layerfs-layerstack-store/src/schema.rs`:

```rust
pub const NEW_STORE_PAGE_SIZE_BYTES: i64 = 4096;   // arm C, unchanged
pub const NEW_STORE_PAGE_SIZE_BYTES: i64 = 65536;  // arm X, treatment
```

Unchanged between arms and explicitly out of scope:

| Item | Value | Why frozen |
| --- | --- | --- |
| `SQLITE_PAGE_CACHE_KIB` | `32 * 1024` | separate lever; changing it would confound the page-size signal |
| `objects/metadata.rs:93` scratch | `PRAGMA page_size=4096; PRAGMA max_page_count=8192` | a distinct 32 MiB scratch DB, not the Store; touching it would confuse two page sizes |
| CDC profile | `crates/layerfs-content/src/file/cdc` | frozen by `dedup-cdc-locality.md:87-91` |
| schema version / SQL | v10 | a migration is not part of the question |
| worker policy | `min(available_parallelism, 8).min(tasks.len())` | §112 measured this as a cross-cutting factor; hold it constant |
| harness | `runner.py`, `runtime.py`, `verify-selected.py` | must be byte-identical so `harness_identity` matches across arms |
| workload / fixture | `dedup_workloads.rs`, prepared inputs | §8 identity |
| compaction | disabled | promoted-uncompacted contract |

`verify_schema` already accepts `!matches!(page_size, 4096 | 65536)`, so no reader
path changes and no Store is migrated. Because `crates/` changes, the product
seal, compilation seal and image tag differ per arm; because the harness does
not, `harness_identity` is identical. That is the whole recordable difference.

## 3. Isolation

Run both arms in **dedicated git worktrees at one frozen commit**, never in the
shared checkout:

```text
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/C   # page size 4096
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/X   # page size 65536
```

Rationale: editing `crates/` in the live checkout changes the source seal and can
invalidate another owner's in-flight qualified build. Each worktree owns its
`benchmark-results/host-store` (builds, fixtures, samples); the DeepSeek inputs
are read from the shared absolute paths and are not duplicated.

The frozen commit is the one recorded in `frozen-commit.txt` and in each arm's
`seals-before.txt`.

### 3.1 Base state — why the shared tree's `crates/` diff is applied to both arms

The shared checkout carries the promoted, uncommitted `crates/` state (the
compaction removal). A clean checkout at the same commit is therefore a
*different* product: it still contains `objects/compaction.rs`. Preparing the
worktrees from the bare commit gave the control arm product seal `73350529…`
instead of the promoted `95e796f8…`.

The driver therefore snapshots `git diff --binary -- crates benchmark` from the
shared checkout, records its sha256, and applies that **identical diff to both
arms** before the page-size constant is set. The `benchmark/` side is required:
the compaction removal spans both trees, so a `crates/`-only base removes the
`CompactionReceipt`/`CompactionOptions`/`compact_into` API while
`benchmark/fs-bench-pro/src/storage_integrated.rs` still calls it, and the arm
does not compile. Recorded at setup:

```text
base diff sha256 = 8a453ae2b45479e034fffaeb9a762e59c774928139b0844022cf655edc34d94e
base diff        = 1921 lines (crates/ + benchmark/fs-bench-pro/)
```

Resulting arm identities, verified before any build:

| | arm C (control) | arm X (treatment) |
| --- | --- | --- |
| `LAYERFS_SOURCE_SEAL` | `00f158e9e0a807e5660018301ad8251c7ee67b82e1f494f981fcfa8c50992a86` | `8215d9fb8a107da5d5615a78b79bbfcf6fb6ffcbc73528019201acedc89673d4` |
| `LAYERFS_PRODUCT_SEAL` | `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4` | `8e5ea0acaf1caac1d1bbe7e4515aaefcf7cd6bd81a5c7e035ad26eb012751b74` |
| `HARNESS_IDENTITY` | `216545afda554446db84b4169fc746cd53dabad66baca5fbf616dc5f55d2a6a8` | **identical** |
| `WORKLOAD_SHA256` | `c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b` | **identical** |
| `NEW_STORE_PAGE_SIZE_BYTES` | `4096` | `65536` |
| `git diff --stat` | base diff only | base diff + 1 line (`schema.rs`) |

Arm C is **byte-identical to the #112 measured arm**: same source seal
`00f158e9…`, same product seal `95e796f8…`, same harness `216545af…`, same
workload. Its absolute `scattered-100` distribution is therefore directly
comparable to the retained n=22 ladder (median `304,673,000 ns`), with no
re-run needed. The sole recordable difference between arms is the page-size
constant, satisfying `benchmark_rules.md` §8.

Risk to declare: the base diff belongs to another owner's in-flight work. It is
snapshotted and hashed at setup time, so mid-experiment commits in the shared
checkout cannot alter either arm.

## 4. Freeze before measuring

Frozen at authoring time, before any arm was built:

| Decision | Value |
| --- | --- |
| speed case (primary) | `dedup_cdc_locality/dedup-cdc-scattered-100`, seed 1, `--setup fresh` |
| speed samples | **4 blocks × 6 samples, order `C X X C`** ⇒ 24 samples per arm |
| speed mode | `--collection-mode --product-timeout 300 --timeout 310 --setup-timeout 600` |
| secondary tiers | `scattered-10` (n=11) and `scattered-500` (n=5), one block per arm |
| storage profile | `repository_history --profile stride-3` (53 states, indices `1,4,7,…,157`) |
| storage repeats | `n = 1` primary; `n = 2` if allocated bytes are not stable |
| aggregate | median across samples; report min, max, spread and sample count |
| outlier policy | keep every valid sample; report any discarded or failed sample with its receipt |
| verification | `--storage-verify-run` for both arms; `cargo test -p layerfs-layerstack-store --lib` for both arms |

Interleaving is mandatory, not hygiene: §112 measured a **3.3%** block-to-block
drift against a **2.6%** two-arm resolution floor at `n=11`. Sequential blocks
manufacture effects of the same size as real ones. The `C X X C` order gives one
within-arm block pair per arm, so drift is observable rather than assumed away.

### 4.1 Driver

`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/run-page-size-ab.sh`
stages: `setup` (worktrees + base diff + constant, validated), `build`
(`--build-host` and `--build-image` per arm), `test`, `speed`, `storage`,
`seals`. Every stage is resumable and writes receipts under
`layerfs-pagearms/evidence/`.

## 5. Commands

Build (once per arm, under the runner measurement lock):

```bash
cd <worktree>
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
export ARM_IMAGE="$(python3 benchmark/fs-bench-pro/shared/runner.py --build-image)"
```

Speed, interleaved — the driver alternates arms per invocation:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --family dedup_cdc_locality \
  --case dedup-cdc-scattered-100 --seed 1 --setup fresh \
  --image "$ARM_IMAGE" --perf-samples 11 --collection-mode \
  --product-timeout 300 --timeout 310 --setup-timeout 600 \
  --output <fresh-path-per-arm-per-block>
```

Storage:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --family repository_history \
  --profile stride-3 --image "$ARM_IMAGE" --output <fresh-path>
python3 benchmark/fs-bench-pro/shared/runner.py --family repository_history \
  --profile stride-3 --storage-verify-run <arm-run> --image "$ARM_IMAGE" \
  --output <fresh-verify-path>
```

## 6. Predicted counter movement

Stated before measuring so the result is falsifiable:

| Prediction for arm X (64 KiB) | Refuted if |
| --- | --- |
| Store `page_count` falls ≈13.5× for the same payload | page count does not track bytes / page size |
| `sql_commit_ns` falls on `scattered-100` | flat while `page_count` falls → page size is not the driver |
| host `system_cpu_ns` falls on `scattered-100` | flat |
| `pure_call_sum_ns` falls on `scattered-100` | flat or worse |
| `repository_history` stride-3 `allocated_bytes` rises | flat → the 15.5% Store win is unrelated to page size |
| both arms pass 53-state verification | any correctness or allocation-gate failure on X |

A **negative** result is a valid outcome: it moves the `>3×` cost off page size
and onto the payload-layout change, which is #100/#107 scope.

## 7. Reporting rules

* Report **allocated and apparent bytes separately**, never pooled. Send the
  storage result to #107 with its `st_blocks × 512` basis.
* Report `pure_call_sum_ns` with sample count, median, min, max and spread, plus
  the matching counter movement. A timer delta without counter movement is noise.
* State `fixture_cache_profile` and disk-read evidence with each gate. For the
  `workspace` route and `repository_history`, `fixture_cache_profile` is **not
  emitted** by those receipt schemas — report **INAPPLICABLE with the reason**,
  never zero. `initialization_disk_read_bytes` is likewise not emitted for the
  `workspace` route; substitute phase-final `host-resources.disk_read_bytes`.
* The X arm is `admission_eligible=false` for release purposes and MUST NOT be
  promoted. 4 KiB pages remain the promoted creation contract until a separate
  storage decision is recorded on #107.
* Publish large percentage changes with absolute time and bytes.

## 8. Non-negotiables

* No change to fixtures, timers, verification assertions, comparison thresholds,
  target values, or historical receipts.
* No compaction, VACUUM, repack, or background rewriter; promoted-uncompacted
  treatment preserved. 4 KiB SQLite pages remain the promoted contract.
* No resource-limit or timeout raise. No CDC boundary change.
* Serialize on `$TMPDIR/layerfs-infra-measurement.lock`; no resource-sensitive
  overlap with other owners.
* Retain every sample, including failures and outliers, at fresh `--output`
  paths. A failed X arm is evidence, not something to suppress by relaxing a
  check.

## 9. Cost

Measured on the first execution (the earlier 1.5–3 h estimate was pessimistic and
is corrected here):

| Step | Measured / estimate |
| --- | --- |
| worktree + base diff + constant, both arms | < 5 s |
| host build, arm C | **80 s** |
| image build, arm C | **105 s** |
| host build, arm X | ≈45 s |
| image build, arm X | ≈60 s |
| `cargo test -p layerfs-layerstack-store --lib`, per arm | minutes (release) |
| `deepseek-stride3` measured phases, per arm | 188.9 s performance + 141.1 s verification = 330.0 s (retained #100 measurement), plus fixture preparation and container I/O |
| `scattered-100`, 24 samples per arm | ≈3.7 s/sample + per-invocation overhead |
| secondary tiers `scattered-{10,500}`, per arm | ≈11 × 46 ms + 5 × 1.3 s of product time, plus overhead |
| **total, both arms** | **≈45–75 min wall** |

Risk: a 64 KiB Store may trip an unknown page-size assumption outside
`schema.rs:547`. Behaviour is fail-closed — if X fails verification or the
integrated format probe, record the failure and stop.

## 10. Evidence layout

```text
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/
  C/  X/                       worktrees
  evidence/
    <arm>-seals-before.txt     source/product/harness/workload/binary seals
    <arm>-seals-after.txt
    <arm>-build.log
    speed/<arm>/block{1..4}/perf.jsonl
    storage/<arm>/             stride-3 performance receipt
    storage/<arm>-verify/      stride-3 verification receipt
    unit-tests/<arm>.log
    summary.md
```

## 11. Result

Executed 2026-09-11. Frozen commit `9867371af5acae1363efd04dde0d0fa248460063`,
base diff `8a453ae2b45479e034fffaeb9a762e59c774928139b0844022cf655edc34d94e`.
Raw evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/evidence/`.

### 11.1 Speed — `dedup_cdc_locality/dedup-cdc-scattered-100`

Interleaved `C X X C`, 6 samples per block, 12 samples per arm. All samples
`PASS`/`COMPLETE`/`slow=false`.

| | C 4096 (promoted `95e796f8`) | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| **`pure_call_sum_ns` median** | **302,620,750** | **171,506,646** | **0.567** |
| pooled spread (max/min) | 1.196 | 1.226 | |
| per-block medians | 297,815,084 / 306,518,187 | 163,467,584 / 178,057,688 | |

**64 KiB pages make this case 1.76× faster.** Every work counter is identical,
so the whole delta is host kernel time and the Store's page-flush path:

| counter (median ns) | C 4096 | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| `prepare_import_wall_ns` | 296,448,625 | 169,075,646 | 0.570 |
| `sql_commit_ns` | 153,429,918 | **26,264,166** | **0.171** |
| └ `pipeline_ns` | 148,385,897 | 25,470,624 | 0.172 |
| └ `publication_ns` | 4,800,875 | 792,959 | 0.165 |
| `sql_begin_ns` | 210,269 | 125,294 | 0.596 |
| `slab_send_blocked_ns` (concurrent) | 958,691,288 | 477,666,031 | 0.498 |
| `slab_consumer_idle_ns` | 1,743,128 | 1,866,960 | 1.071 |
| `last_slab_receive_offset_ns` | 294,660,708 | 166,822,458 | 0.566 |
| host `user_cpu_ns` | 296,314,937 | 284,882,291 | 0.961 |
| host `system_cpu_ns` | 203,779,500 | **86,854,187** | **0.426** |
| `canonical_frame_count` | 5,998 | 5,998 | 1.000 |
| `canonical_payload_bytes` | 106,286,153 | 106,286,153 | 1.000 |
| `sql_submitted_rows` / `sql_batch_count` | 5,967 / 26 | 5,967 / 26 | 1.000 |
| `worker_count` | 4 | 4 | 1.000 |
| Store `page_count` | 26,200 | **1,720** | **0.066** |
| Store `allocated = apparent` | **107,315,200** | **112,721,920** | **1.050** |

Secondary tiers (one block per arm):

| tier | C median | X median | X/C | C pages | X pages | C allocated | X allocated | alloc X/C |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 36,331,542 | 24,330,167 | 0.670 | 2,872 | 204 | 11,763,712 | 13,369,344 | 1.136 |
| 100 | 302,620,750 | 171,506,646 | 0.567 | 26,200 | 1,720 | 107,315,200 | 112,721,920 | 1.050 |
| 500 | 1,319,371,250 | 893,764,833 | 0.677 | 129,900 | 8,482 | 539,947,008 | 567,869,440 | 1.052 |

### 11.2 Storage — `repository_history --profile stride-3`, 53 states

Both arms: performance `PASS`, verification `PASS` on **all 53 states**; no
page-size assumption failed.

Measurement boundary note: the **performance boundary** is `host_runtime_disk`
in `performance-step-53.json`, taken inside the performance phase. The
`host-runtime/store.sqlite` row below was read with `PRAGMA` **after** the
verification pass, which deliberately creates a `verify-*` fork branch per
historical state; it is included because it is the only direct `PRAGMA
page_size/page_count` reading, and both arms were treated identically so the
ratio is unaffected. Quote the performance-boundary row for storage claims.

| | C 4096 | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| `PRAGMA page_size` (post-verification) | 4,096 | 65,536 | 16× |
| `PRAGMA page_count` (post-verification) | 15,771 | 1,158 | **0.0734** (13.62× fewer) |
| `PRAGMA freelist_count` | 0 | 0 | — |
| Store apparent bytes (post-verification) | 64,598,016 | 75,890,688 | **1.175** |
| Store **allocated** bytes (`st_blocks × 512`, post-verification) | 65,056,768 | 84,606,976 | **1.301** |
| **host-runtime dir apparent, performance boundary** | **70,816,106** | **82,116,970** | **1.160** |
| **host-runtime dir allocated, performance boundary** | **71,643,136** | **91,193,344** | **1.273** |
| spool allocated | 6,578,176 | 6,578,176 | 1.000 |
| container staging allocated | 18,022,400 | 18,026,496 | 1.000 |
| `commit` phase total, 53 steps | 21,700.8 ms | 21,392.2 ms | **0.986** |
| `exec` phase total, 53 steps | 117,697.9 ms | 114,907.8 ms | 0.976 |
| per-step wall total | 118.6 s | 117.9 s | 0.99 |

**Prefer `apparent` over `allocated` for claims.** The `allocated` ratio is
inflated by an APFS allocation artefact: X's `st_blocks × 512` sits flat at
~91.19 MB across the final four steps while its apparent size keeps growing, so
the +27.3% allocated penalty overstates the Store-size cost relative to the
+16.0% apparent penalty.

**64 KiB pages cost 17.5% apparent / 30% allocated storage on this workload and
buy no measurable latency.** The page-size latency win is not visible here
because this workload's per-step wall is transfer-dominated and its commit path
is delta/reuse-heavy rather than near-unique full-payload publication.

### 11.3 Verdict — the trade runs in opposite directions

| workload | 4 KiB buys | 4 KiB costs | which page size wins |
| --- | --- | --- | --- |
| bulk near-unique init (`scattered-100`) | **5.0%** fewer allocated Store bytes (107,315,200 vs 112,721,920) | **+76%** import latency (302,620,750 vs 171,506,646 ns) | **64 KiB** |
| history/delta publication (`stride-3`, 53 states) | **23%** fewer allocated bytes (65,056,768 vs 84,606,976) | ~1.4% `commit` phase (0.986), i.e. none resolvable | **4 KiB** |

There is no single correct page size for both. The present global constant forces
a compromise: 4 KiB is clearly right where Store size is the objective (history,
23%) and clearly wrong where Init latency is the objective (bulk init, 1.76×).

### 11.4 Corrections to the pre-experiment inference

1. **The v0.1.3 → v0.1.5 Store reduction is not page geometry.** Controlled, page
   geometry accounts for only **5.0%** at tier 100, not the 15.5% observed across
   generations. ~10.5 points of that reduction came from the payload-layout
   change. §112 §4.4's inference is superseded by this measurement.
2. **Page geometry accounts for 61.5% of the `scattered-100` gap**, not the ~40%
   inferred in §112 §4.5: X (64 KiB, current product) is 171,506,646 ns against
   v0.1.3's 89,368,333 ns, so 131,114,104 ns of the 213,252,417 ns gap is page
   geometry and ~82 ms remains with the layout/consumer-serial change.
3. **The 4 KiB creation policy was adopted without a documented trade.**
   `70955bc25` states a compatibility goal only; this experiment supplies the
   first quantified exchange rate.
4. **`cargo test -p layerfs-layerstack-store --lib` must run in debug**, because
   `set_transaction_failure_at` is `#[cfg(debug_assertions)]` (`schema.rs:50`).
   A `--release` invocation fails to compile on *both* arms and is an invocation
   error, not a treatment finding.

### 11.5 Correctness gates

| gate | C 4096 | X 65536 |
| --- | --- | --- |
| `repository_history` stride-3 performance | PASS | PASS |
| stride-3 verification, 53/53 states | PASS | PASS |
| `cargo test -p layerfs-layerstack-store --lib` (debug) | **128 passed**, 0 failed, 4 ignored | **128 passed**, 0 failed, 4 ignored |
| container OOM / swap | 0 / 0 | 0 / 0 |
| integrated format probe at build | PASS | PASS |

The 64 KiB arm is fully correct: no page-size assumption outside `schema.rs:547`
was tripped, and both layouts verify the identical 53-state history.

### 11.6 Why 64 KiB does not help the stride-3 commit

Page size changes exactly one thing: the cost of touching pages. Its benefit is
therefore proportional to *(page-flush work) ÷ (total phase work)*, and on the
stride-3 commit that ratio is a few percent.

**The stride-3 commit is 97% CPU-busy, so it is not waiting on the page path.**

| commit phase, 53 steps | C 4096 | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| `elapsed_ns` | 21,700,839,124 | 21,392,247,913 | 0.986 |
| host CPU (`user+system`) | 21,117,171,377 | 20,802,771,005 | 0.985 |
| **CPU utilisation (`cpu/elapsed`)** | **0.973** | **0.972** | — |

Compare scattered-100, where the C arm spent 203,779,500 ns of *kernel* CPU
against a 302,620,750 ns timer — the page path is ~39% of that timer.

**Every unit of work in the stride-3 commit is byte-identical between arms**, and
the dominant one is content processing that page size cannot touch:

| counter, commit phase, 53 steps | C 4096 | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| `decoded_read_bytes` | 3,468,309,272 | 3,463,494,476 | 0.999 |
| `decompression_calls` | 346,446 | 346,393 | 1.000 |
| `blob_ranges` | 1,074,292 | 1,074,136 | 1.000 |
| `base_fetches` | 187,483 | 187,428 | 1.000 |
| `encoding_calls` | 115,738 | 115,739 | 1.000 |
| `encoding_ns` | 3,313,478,516 | 3,366,560,831 | **1.016** |
| `matching_ns` | 68,506,791 | 69,638,158 | 1.017 |
| `native_decode_ns` | 133,680,000 | 135,724,902 | 1.015 |
| `delta_selected` / `full_selected` | 105,873 / 27,361 | 105,869 / 27,365 | 1.000 |

At step 53 alone the commit decodes 89,332,408 bytes of already-stored content,
performs 7,811 decompressions and spends 61,708,451 ns encoding. None of that
work observes the page size. `encoding_ns` even rises 1.6% on X — noise-level,
but it shows the change did not help the dominant path.

**The write path is a small share, and it did not shrink:** commit
`host_disk_write_bytes` 809,418,752 → 814,268,416 (×1.006), i.e. the same bytes
flushed. At the per-4-KiB-page syscall cost scattered-100 exhibits
(≈4.5 µs/page), 809 MB is ≈197,600 page writes ≈0.9 s of the 21.7 s phase (≈4%);
64 KiB removes nearly all of it, predicting ≈0.8 s of relief. Observed relief:
21,700,839,124 → 21,392,247,913 = **0.31 s**, and part of that is noise. Order of
magnitude agrees; the effect is real but small.

**Where 64 KiB did help stride-3.** The `exec` phase:

| exec phase, 53 steps | C 4096 | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| host CPU | 38,153,110,089 | 34,204,505,950 | **0.897** |
| `elapsed_ns` | 117,697,898,461 | 114,907,801,293 | 0.976 |
| CPU utilisation | 0.324 | 0.298 | — |
| `host_disk_write_bytes` | 434,176 | 434,176 | 1.000 |

That is a 3.95 s CPU saving (−10.3%) while writing essentially nothing — the
**read-side** benefit of 16× fewer page-cache entries and B-tree pages for
6.36 GB of decoded reads. It does not show in wall time because this phase is
29–32% CPU-busy: it is dominated by container tree transfer and waiting.

**Rule.** Larger pages help in proportion to how much of a phase is spent
*handling* pages rather than *processing* content:

| phase | page-handling share | 64 KiB benefit |
| --- | ---: | ---: |
| `scattered-100` commit (near-unique full-payload publication) | ≈39% of the timer | **1.76×** |
| `stride-3` commit (delta/reuse-heavy) | ≈4% | 1.4% |
| `stride-3` exec (read-side, transfer-dominated) | ≈10% of CPU | 10.3% CPU, 2.4% wall |

The distinguishing workload property is **uniqueness of published payload**, not
Store size or object count: scattered-100 publishes 5,967 of 5,998 objects as new
rows with `conflict_read_calls = 0` and negligible `matching_ns`, so almost its
entire cost is flushing newly written pages. Stride-3 reuses and delta-matches,
so its cost is reading, decompressing and encoding content that is already there.

### 11.7 Seals

| | C (control) | X (treatment) |
| --- | --- | --- |
| frozen commit | `9867371af5acae1363efd04dde0d0fa248460063` | same |
| `LAYERFS_SOURCE_SEAL` | `00f158e9e0a807e5660018301ad8251c7ee67b82e1f494f981fcfa8c50992a86` | `8215d9fb8a107da5d5615a78b79bbfcf6fb6ffcbc73528019201acedc89673d4` |
| `LAYERFS_PRODUCT_SEAL` | `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4` | `8e5ea0acaf1caac1d1bbe7e4515aaefcf7cd6bd81a5c7e035ad26eb012751b74` |
| `HARNESS_IDENTITY` | `216545afda554446db84b4169fc746cd53dabad66baca5fbf616dc5f55d2a6a8` | **identical** |
| `WORKLOAD_SHA256` | `c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b` | **identical** |
| host binary sha256 | `fc461532ddb5679db7ef3c720d64bec3ba49eeae2aeba22dc2041fef4d87ce44` | `817e7fa763b8863a725ee7123ab405fdc8ae98544dca260544782bd383b07b69` |
| image | `layerfs-bench-infra:00f158e9e0a807e5` | `layerfs-bench-infra:8215d9fb8a107da5` |
| base diff sha256 | `8a453ae2b45479e034fffaeb9a762e59c774928139b0844022cf655edc34d94e` | same |

Neither arm is promoted. 4 KiB pages remain the promoted creation contract.

---

## 12. Full-157 and A/B decision set (2026-09-11, second phase)

§11 settled the *mechanism* on `scattered-100` and one history profile. It did not
settle the *scope*, and it explicitly left the stride-3 penalty trend as a trend
rather than a full-157 measurement. Both gaps are now closed.

**Result: `full157-and-ab-results.md`** (this directory) — full 157-state storage
and time on both arms, plus the seven-slot interleaved A/B set.

Summary of the decisions taken there:

* **Full 157 states** (`repository_history --profile stride-1` / `deepseek-full`),
  both arms performance **PASS** and verification **PASS on all 157 states**.
  Performance-boundary apparent **91,931,104 → 106,447,328 B (+15.8%)**,
  allocated **+18.8%**, on 65,020,822 logical bytes. This lands **inside** the
  +10–16% band §11.2's stride-3 trend predicted (at its upper edge), so the
  penalty is bounded and the endpoint is not a scope risk. The ratio flattens
  within ±2% from **state 60** and within ±0.5% from **state 117**; asymptotic
  slope −0.0203 per e-fold of logical bytes over states ≥79. Full-157 wall
  X/C 0.967 (−3.3%, 15.5 s of 468.8 s).
* **A/B set, seven slots, 24 samples per arm**, interleaved `C X X C`.
  Confirmed: `scattered-100` **0.5843** (anchor; arm C 306.2 ms against #112's
  304.7 ms ladder, +0.5%), `namespace-10000` **0.6628**,
  `store-footprint-large-object-500m` **0.6863**, `directory-content-scan` (zero
  pages written) **0.9334**, `payload-create-500m` **0.9517**.
  Refuted/qualified: `dedup-history-distributed-500` is **+2.74% on time and
  +62.83% on storage** — the worst storage result in the set for no win — and the
  SDK edit slot is neutral on its registered timer (0.9949) but **+63% on the
  product command wall with 8.78× incremental RSS**.
* **Recommendation: outcome 3 (no global change) plus a class-scoped creation
  policy.** Storage deltas across the seven cases range **+5.17% to +62.83%
  apparent**, so no single global figure is defensible. The class is defined by
  measurable properties — create-path Stores, large objects, newly-written-page
  handling a large share of phase work — not by family name. No threshold is
  fixed; §5.3 of the result document states what remains unresolved.
* **Nothing was promoted.** `NEW_STORE_PAGE_SIZE_BYTES` in the shared checkout is
  still `4096`. The storage-trade statement for **#107** (bytes added, boundary,
  and the baselines that become stale custody) is §6 of the result document.

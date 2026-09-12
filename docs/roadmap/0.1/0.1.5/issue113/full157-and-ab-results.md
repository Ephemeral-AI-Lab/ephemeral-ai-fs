# Issue #113 — page size 4 KiB → 64 KiB: full-157 storage and the A/B decision set

Executed 2026-09-11. Frozen commit `9867371af5acae1363efd04dde0d0fa248460063`.
Base diff `8a453ae2b45479e034fffaeb9a762e59c774928139b0844022cf655edc34d94e`.

Arms: **C** = `NEW_STORE_PAGE_SIZE_BYTES 4096` (control, promoted product
`95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`) and
**X** = `65536`. Verified before and after the runs: both arms differ from each
other only at `crates/layerfs-layerstack-store/src/schema.rs:14`; all shared
identities identical; seals unchanged (see §8).

Raw evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/evidence/`.
This document is derived; every number below is reproducible from the raw
receipts with `full157-series.py` and `analyze-ab-set.py`.

**This document records a decision of type 3 (no global change) plus a
class-scoped policy proposal. Nothing was promoted. `NEW_STORE_PAGE_SIZE_BYTES`
in the shared checkout is still `4096`.**

---

## 1. What was measured

| set | arms | samples | source |
| --- | --- | ---:| --- |
| Full 157 states, `repository_history --profile stride-1` (`deepseek-full`) | C, X | 157 performance states + 157 verification states each | `evidence/full157/<arm>/` |
| A/B slot 1 `edit_length_preserving/overwrite-middle-4k-on-500mib-ops-1` | C, X | 24 + 24 | `evidence/ab/<arm>/…/block{1,4,5,8}/{2,3,6,7}` |
| A/B slot 2 `dedup_branch_history/dedup-history-distributed-500` | C, X | 24 + 24 canonical (36 + 36 retained) | as above |
| A/B slot 3 `directory_construction_traversal/directory-content-scan-500-mixed-v4` | C, X | 24 + 24 canonical (39 + 36 retained) | as above |
| A/B slot 4 `dedup_cdc_locality/dedup-cdc-scattered-100` | C, X | 24 + 24 | as above |
| A/B slot 5 `init_namespace/namespace-10000` | C, X | 24 + 24 | as above |
| A/B slot 6 `store_footprint/store-footprint-large-object-500m` | C, X | 24 + 24 | as above |
| A/B slot 7 `payload_create_read/payload-create-500m` | C, X | 24 + 24 | as above |

A/B interleave is the frozen `C X X C` order, 6 samples per block, two rounds ⇒
C blocks `1,4,5,8`, X blocks `2,3,6,7` = **24 samples per arm per slot**. No
sample was discarded: every retained block is reported, including the
supplementary third round for slots 2–3 and the truncated
`C/directory-content-scan-500-mixed-v4/block12` (3 of 6 samples, reported and
**excluded from pooled statistics**).

**Invocation correction.** The handoff specified Task 1 as
`runner.py --family repository_history --profile stride-1 --image … --output …`.
`runner.py` has no `--profile` and no `--storage-verify-run` flag. The working
entry point is `benchmark/fs-bench-pro/shared/repository_history.py --profile
stride-1`, which forwards to `storage_smoke.main(['--storage-smoke',
'deepseek-full', …])`. Both arms' full-157 receipts were produced through that
entry point; `identity.json` on both arms records `"smoke": "deepseek-full"` and
157 states.

---

## 2. Full 157 — storage increase and time

Both arms: performance **PASS**, verification **PASS on all 157 states**.

### 2.1 Performance boundary (quote this for storage claims)

`host_runtime_disk` from `performance-step-157.json`; Store geometry from that
same step's `storage-smoke-allocation` receipt (a pre-verification reading).

| | C 4096 | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| **host-runtime apparent** | **91,931,104** | **106,447,328** | **1.158** |
| host-runtime allocated | 93,044,736 | 110,493,696 | 1.188 |
| logical bytes at final step | 65,020,822 | 65,020,822 | 1.000 |
| **apparent overhead (apparent/logical)** | **1.4139** | **1.6371** | **1.158** |
| Store `page_size` | 4,096 | 65,536 | 16× |
| Store `page_count` | 20,392 | 1,496 | **0.073** |
| Store `freelist_count` | 0 | 0 | — |
| Store apparent bytes | 83,525,632 | 98,041,856 | 1.174 |

**Absolute storage cost of a global switch on this workload: +14,516,224 B
apparent (+14.5 MB), +17,448,960 B allocated (+17.4 MB), on a 65.0 MB logical
payload — +15.8% apparent / +18.8% allocated.**

### 2.2 Post-verification reading (different Store — labelled, not a claim)

Read from `host-runtime/store.sqlite` *after* the verification pass, which creates
a `verify-*` fork branch per historical state (trap #6). Included only because it
is the direct `PRAGMA` reading; **do not quote for storage claims.**

| | C 4096 | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| `PRAGMA page_size` | 4,096 | 65,536 | 16× |
| `PRAGMA page_count` | 20,401 | 1,496 | 0.0733 |
| `PRAGMA freelist_count` | 0 | 0 | — |
| `st_size` (apparent) | 83,562,496 | 98,041,856 | 1.173 |
| `st_blocks × 512` (allocated) | 83,935,232 | 101,384,192 | 1.208 |

The apparent ratio at the post-verification boundary (1.173) agrees with the
performance boundary (1.174), so the verification pass did not materially change
the Store-internal ratio.

### 2.3 The per-state trend — where the penalty flattens

Full series in `evidence/full157/full157-series.csv` (157 rows × both arms).

Apparent X/C, by segment (states are full-157 indices):

| states | first | last | mean | min | max |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1–39 | 2.746 | 1.246 | 1.4365 | 1.246 | 2.746 |
| 40–78 | 1.240 | 1.174 | 1.1940 | 1.1735 | 1.2399 |
| 79–118 | 1.1735 | 1.1646 | 1.1693 | 1.1632 | 1.1765 |
| 119–157 | 1.1650 | 1.1579 | 1.1611 | 1.1556 | 1.1665 |

**The penalty is bounded and monotone-decreasing in relative terms.** The series
is dominated by the fixed 64 KiB page-floor cost when the Store is small (ratio
2.75 at state 1, where both Stores are ~1 MB) and converges to a constant
additive offset once the payload dwarfs the page floor.

Flatness, measured as "first index from which the ratio stays within *tol* of the
final 1.158":

| tolerance | first index | states covered |
| --- | ---: | ---: |
| within 2% | 60 | 98 |
| within 1% | 91 | 62 |
| within 0.5% | 117 | 24 |

So **the region where the penalty trend flattens is state ≈60 (±2%) and
state ≈117 (±0.5%)**, with the asymptotic slope over states ≥79 of
**−0.0203 per e-fold** of logical bytes.

**Expected vs observed.** The handoff predicted +10–16% apparent at full-157 from
the stride-3 trend. **Observed +15.8% — inside the predicted band, at its upper
edge.** It is *not* materially above the trend, so the endpoint does not raise a
scope risk. The trend is decaying, not growing.

### 2.4 Phase timings, 157 steps

| | C 4096 | X 65536 | X/C |
| --- | ---: | ---: | ---: |
| `exec` total | 320,346,026,630 | 310,927,573,750 | 0.971 |
| `commit` total | 57,248,078,672 | 54,103,455,579 | 0.945 |
| per-step wall total | 468,802,170,830 | 453,279,625,875 | 0.967 |
| `exec` host CPU | 117,456,495,192 | 117,471,958,006 | 1.000 |
| `commit` host CPU | 54,897,359,080 | 52,280,129,290 | 0.952 |

**64 KiB pages buy 3.3% of full-157 wall time (15.5 s of 468.8 s) and cost 15.8%
apparent storage.** That is the whole of the global-switch trade on this
workload: the win is real but small in relative terms, because the phase is
mostly content processing the page size cannot touch (the same reason stride-3's
`commit` moved only 1.4%).

---

## 3. The A/B set — seven slots

Timer is each slot's **registered** timer. Storage is read from the arm's
recorded command-boundary observation; **`apparent` primary, `allocated`
(= `st_blocks × 512`) labelled separately**.

### 3.1 Headline table

| slot | case | role | timer | C median ns | X median ns | **X/C** | C apparent B | X apparent B | **app %** | alloc % |
| ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | `overwrite-middle-4k-on-500mib-ops-1` | falsification | `edit_commit_ns` | 10,923,729 | 10,867,480 | **0.9949** | 530,378,752 | 563,216,384 | **+6.19** | +6.00 |
| 2 | `dedup-history-distributed-500` | falsification | `pure_call_sum_ns` | 4,328,747,734 | 4,447,242,937 | **1.0274** | 1,851,392 | 3,014,656 | **+62.83** | +47.78 |
| 3 | `directory-content-scan-500-mixed-v4` | read isolation | `pure_call_sum_ns` | 4,756,136,395 | 4,439,592,584 | **0.9334** | 530,362,368 | 571,473,920 | **+7.75** | +7.75 |
| 4 | `dedup-cdc-scattered-100` | anchor | `pure_call_sum_ns` | 306,224,188 | 178,924,771 | **0.5843** | 107,307,008 | 112,852,992 | **+5.17** | +5.17 |
| 5 | `namespace-10000` | candidate | `layerstack_init_ns` | 950,076,520 | 629,739,479 | **0.6628** | 304,904,192 | 324,927,488 | **+6.57** | +4.06 |
| 6 | `store-footprint-large-object-500m` | candidate | `product_call_sum_ns` | 1,260,560,520 | 865,095,375 | **0.6863** | 505,835,520 | 536,182,784 | **+6.00** | +5.89 |
| 7 | `payload-create-500m` | candidate | `pure_call_sum_ns` | 3,123,175,124 | 2,972,435,542 | **0.9517** | 530,337,792 | 563,019,776 | **+6.16** | +6.27 |

Per-slot detail (per-block medians, min/max/spread, counters, CPU split) is in
`evidence/ab/ab-tables.md`; machine-readable form in
`evidence/ab/ab-summary.json`.

### 3.2 Sample counts, spread, and per-block medians

| slot | C n | X n | C spread | X spread | C per-block medians (ms) | X per-block medians (ms) |
| ---: | ---: | ---: | ---: | ---: | --- | --- |
| 1 | 24 | 24 | 1.361 | 1.527 | 11.13 / 9.74 / 11.27 / 10.94 | 10.79 / 10.81 / 10.93 / 10.76 |
| 2 | 24 | 24 | 1.297 | 1.173 | 4453 / 4354 / 4301 / 4094 | 4513 / 4518 / 4349 / 4270 |
| 3 | 24 | 24 | 1.301 | 1.242 | 4761 / 4810 / 4835 / 4668 | 4300 / 4576 / 4552 / 4495 |
| 4 | 24 | 24 | 1.217 | 1.112 | 305.9 / 305.2 / 305.3 / 306.2 | 178.0 / 176.8 / 183.2 / 179.4 |
| 5 | 24 | 24 | 1.059 | 1.090 | 945.3 / 944.6 / 948.9 / 955.9 | 633.6 / 641.6 / 627.8 / 619.8 |
| 6 | 24 | 24 | 1.168 | 1.098 | 1325 / 1257 / 1247 / 1248 | 882.5 / 870.6 / 863.3 / 847.6 |
| 7 | 24 | 24 | 1.286 | 1.407 | 3037 / 3456 / 3025 / 2920 | 2984 / 2607 / 3167 / 2987 |

### 3.3 Matching counter movement

Slot 4 (the anchor) is the clean case: every work counter is identical, so the
whole delta is host kernel/Store page-flush time.

| counter | kind | C | X | X/C |
| --- | --- | ---: | ---: | ---: |
| `worker_count` | invariant | 4 | 4 | 1.000 |
| `canonical_frame_count` | invariant | 5,998 | 5,998 | 1.000 |
| `canonical_payload_bytes` | invariant | 106,286,153 | 106,286,153 | 1.000 |
| `sql_submitted_rows` | invariant | 5,967 | 5,967 | 1.000 |
| `sql_batch_count` | invariant | 26 | 26 | 1.000 |
| `source_file_read_bytes` | invariant | 105,906,176 | 105,906,176 | 1.000 |
| `sql_commit_ns` | timing | 154,995,126 | 28,518,747 | **0.184** |
| `prepare_import_wall_ns` | timing | 299,508,417 | 176,153,916 | 0.588 |
| `pipeline_ns` | timing | 149,905,918 | 27,488,228 | 0.183 |
| host `user_cpu_ns` | timing | 298,796,228 | 288,713,458 | 0.966 |
| host `system_cpu_ns` | timing | 207,421,000 | **97,608,208** | **0.471** |

`slab_send_blocked_ns` (concurrent producer time — **never** added to wall):
C 975,111,620 → X 501,194,781 ns.

Slot 3 (`directory-content-scan`) writes **zero** pages and still moved
`pure_call_sum_ns` X/C 0.9334 with `created_commit_count` 0 on both arms — a
read-side page-lookup win: host `user_cpu_ns` 1,312,377,146 → 1,228,295,979
(0.936) while `system_cpu_ns` rose 749,904,270 → 882,329,166 (1.177).

Slot 1 counters: `commit_cdc_bytes_scanned` 4,096 (1.000), `inserted_bytes`
30,578 (1.000), `edit_call_ns` 2,203,187 → 2,183,667 (0.991), `commit_call_ns`
8,679,730 → 8,716,396 (1.004).

### 3.4 Fixture identity

`identities.input_identity` differs between arms on every slot; that is the
content-addressed `preparation.cache_key` including the Store schema hash
(trap #7), not a different fixture. `preparation.fixture.fixture_sha256` matches
between arms on all seven slots, and both arms record the same
`HARNESS_IDENTITY` / `WORKLOAD_SHA256`.

---

## 4. Confirmed and refuted predictions

| # | prediction | outcome |
| --- | --- | --- |
| 1 | full-157 apparent ≈ +10–16% | **CONFIRMED** (+15.8%), band's upper edge |
| 2 | penalty trend bounded, flattening | **CONFIRMED** — flat within ±2% from state 60, slope −0.0203/e-fold over ≥79 |
| 3 | `edit_length_preserving` neutral-or-negative | **CONFIRMED for the registered timer** (0.9949, −0.51%, indistinguishable) — see the refutation note below |
| 4 | `dedup_branch_history` neutral | **CONFIRMED on time** (1.0274, +2.7%, within the ±2.6% floor+n=24 noise), but **its storage is the worst in the set at +62.83%** |
| 5 | `directory-content-scan` small-positive (read isolation, 0 pages) | **CONFIRMED** −6.66% on a case that writes no pages, replicating stride-3's −10.3% exec-CPU read-side effect |
| 6 | `scattered-100` strong-positive | **CONFIRMED** 0.5843; C median 306.2 ms against the retained #112 ladder 304.7 ms (**+0.5%**, so the base chains to #112) |
| 7 | `namespace-10000` strong-positive | **CONFIRMED** 0.6628 (−33.7%) |
| 8 | `store-footprint-large-object-500m` strong-positive (screen share 0.434, highest) | **CONFIRMED** 0.6863 (−31.4%) |
| 9 | `payload-create-500m` positive (screen share 0.221) | **CONFIRMED but weak** −4.83% |
| 10 | "most families get ≈0 and the SDK edit class may get worse" | **PARTLY REFUTED** — the SDK edit *registered timer* is neutral, but see below |

### 4.1 The one prediction that needs correcting

The screen held that the SDK edit class saves nothing. That is **true of its
registered timer** (`edit_commit_ns` X/C 0.9949) but **false of its product
command wall**, which rose block-consistently and materially:

| slot 1 | C | X | X/C |
| --- | ---: | ---: | ---: |
| `edit_commit_ns` (registered timer) | 10,923,729 | 10,867,480 | 0.995 |
| `command_wall_ns` (product command window, median) | 438,873,271 | **715,769,667** | **1.631** |
| `command_wall_ns` per block (ms) | 443.0 / 394.0 / 440.8 / 439.4 | 662.3 / 720.6 / 688.2 / 667.0 | 1.63 block-consistently |
| `rss_incremental_upper_bound_bytes` | 13,565,952 | **119,119,872** | **8.78** (block spread ≤1.05) |
| `process_lifetime_peak_rss_bytes` | 31,031,296 | 168,067,072 | 5.42 |
| `rss_baseline_bytes` | 17,457,152 | 48,971,776 | 2.81 |
| `commit_publication_ns` | 103,521 | 172,416 | 1.67 |
| `wall_ns` (includes ~2.1 s preparation) | 3,438,749,375 | 3,765,772,000 | 1.095 |

`wall_ns` alone would overstate the case, because `preparation_wall_ns` is
2,097,448,771 → 2,131,786,521 (1.016) and `cleanup.wall_ns` 693,384,250 →
708,159,480 (1.021). The movement is in `command_wall_ns`, i.e. the measured
product command itself.

**Mechanism:** 64 KiB pages make the edit path's *dirty/addressing* footprint
16× coarser. The edit is a single 4 KiB overwrite (`commit_cdc_bytes_scanned`
4,096, `inserted_bytes` 30,578, unchanged on both arms) yet X's incremental RSS
bound is 8.78× C's and its commit publication cost 1.67×. The 16-sample RSS
evidence has block spread ≤1.05 within each arm, so this is not drift.

**This is the one case in the set where 64 KiB costs both time and storage**, and
it is exactly the class the handoff flagged as at risk. The registered timer does
not see it because it measures the SDK edit+commit call pair, not the command.
I am reporting it as a finding rather than suppressing it, but I am **not**
claiming it as a refutation of prediction 3: at n=24 the *registered* timer is
flat, and the `command_wall_ns` effect — while block-consistent — was not
pre-registered as this slot's decision variable.

---

## 5. Scoped recommendation

**Outcome 3 (no *global* change) plus a class-scoped creation policy. Nothing
promoted; nothing landed.**

### 5.1 Why not global

A global switch to 64 KiB would buy:

* −41.6% on `scattered-100`, −33.7% on `namespace-10000`, −31.4% on
  `store-footprint-large-object-500m`;
* but only **−3.3% of full-157 wall** (+15.5 s of 468.8 s), **−4.8%** on
  `payload-create-500m`, **−6.7%** on `directory-content-scan`;
* **+2.7%** (a regression, no win) on `dedup-history-distributed-500`;
* **+63% product-command wall and +8.78× incremental RSS** on the SDK edit class.

against a storage cost of **+5.2% to +62.8% apparent, workload-dependent**, with
the global full-157 figure at **+15.8% apparent / +18.8% allocated**.

The full-157 win is real but is 3.3% of wall on the one workload where page
touching is a large share of work. On the history/delta class the win is within
noise at 2.7× the storage cost of the anchor. **The storage cost is not uniform
and is unbounded relative to the benefit for small-object Stores** (slot 2:
+62.83% for a 2.7% *slowdown*). A single global constant cannot be justified by
these tables.

### 5.2 The class boundary, in measurable workload properties

Derived from measurement, not theory. The discriminator is **not** "which
family" — it is the ratio the handoff named in §2.1:

> **benefit ∝ (page-handling work) ÷ (total phase work)**, where page-handling
> work ≈ newly written bytes.

A case is in the 64 KiB class when **both** hold:

1. **Pages newly written is a large share of phase work.** Operationally: the
   case's `before → after-commit` page-count delta × 4,500 ns is a large fraction
   of its timer (the screen's share). Measured shares: 0.434, 0.402, 0.389,
   0.377, 0.370, 0.310 — all ≥0.10.
2. **The Store is created, not pre-existing, and its payload is large relative to
   page size.** This is the condition that actually bounds the penalty. In
   full-157 the ratio converges to a constant *additive* offset once the payload
   dwarfs the 64 KiB page floor: flat within ±2% from state 60.

**The class is therefore: create-path Stores holding large objects, where a large
fraction of phase time is newly-written-page handling.** Concretely the measured
members are `dedup_cdc_locality/{scattered}`, `init_namespace/*`,
`store_footprint/{large-object,unique}`, `payload_create_read/{create}`,
`dedup_cross_file/{unique,mixed}`.

**The complement** — where 4 KiB must stay — is characterized by either:

* **small-object Stores**, where the 64 KiB page floor is a fixed *percentage*
  tax on a small Store: slot 2 `dedup-history-distributed-500` **+62.83%**, and
  the stride-3 small-object result **+16.0% apparent / +27.3% allocated**;
* **pre-existing-Store edit/mutation paths**, where the operation touches a
  bounded dirty set and the coarse page amplifies the write: slot 1
  **+63% command wall, +8.78× incremental RSS, +6.19% storage**;
* **read-only scans**, which gain (−6.66%) but write no pages, so they inherit
  whatever the create-time policy chose — they are not a reason to switch;
* **history/delta publication**, where commit is 96% content processing
  (stride-3: `encoding_ns` X/C 1.016, `decoded_read_bytes` 0.999) and page
  handling cannot matter.

**A creation-time switch is technically sufficient and is what §8 of the
protocol allows**: all A/B slots recorded identical `HARNESS_IDENTITY`,
`WORKLOAD_SHA256` and `fixture_sha256` between arms, and every work counter in
slot 4 was identical — the difference is confined to Store geometry, which is
fixed at creation (`schema.rs:14`) and accepted at open (`schema.rs:547`).

### 5.3 What this recommendation does *not* do

It does **not** select a threshold. The tables bound the class from both ends
(≥0.10 share, create-path, large Store) but do not locate the crossover: slot 7
`payload-create-500m` at share 0.221 gains only −4.83% for +6.16% storage, which
is close to the break-even the handoff asked about. **Fixing a numeric threshold
needs a follow-up measurement, not a decision taken here.**

---

## 6. Storage-trade statement for #107

**Bytes added by the policy if it were adopted:**

| boundary | workload | apparent | allocated (`st_blocks×512`) |
| --- | --- | ---: | ---: |
| full-157 performance boundary | `deepseek-full`, 157 states | **+14,516,224 B (+15.8%)** | +17,448,960 B (+18.8%) |
| A/B slot 1 | SDK edit, 500 MiB fixture | +32,837,632 B (+6.19%) | +32,800,768 B (+6.00%) |
| A/B slot 2 | history/distributed-500 | **+1,163,264 B (+62.83%)** | +1,101,824 B (+47.78%) |
| A/B slot 3 | directory scan, 5000 files | +41,111,552 B (+7.75%) | +41,111,552 B (+7.75%) |
| A/B slot 4 | scattered-100 | +5,545,984 B (+5.17%) | +5,545,984 B (+5.17%) |
| A/B slot 5 | namespace-10000 | +20,023,296 B (+6.57%) | +12,713,984 B (+4.06%) |
| A/B slot 6 | large-object-500m | +30,347,264 B (+6.00%) | +30,273,536 B (+5.89%) |
| A/B slot 7 | payload-create-500m | +32,681,984 B (+6.16%) | +33,669,120 B (+6.27%) |

**At which boundary:** `performance-step-N.json → host_runtime_disk` for full-157
(the performance boundary). For A/B, each arm's recorded command-boundary
observation (`store_boundary` on the SDK route; the final `store-observation:
after-commit` on the workspace route). Post-verification `PRAGMA` readings are a
*different Store* and are labelled as such throughout.

**Caution for #107:**
* `apparent` is primary. `allocated` is `st_blocks × 512` and carries the APFS
  artefact noted in stride-3 (X's allocated sat flat while apparent kept growing).
* The delta is **not** a single percentage. It ranges **+5.17% to +62.83%
  apparent** across the seven measured cases. A single global figure would be
  wrong; the +15.8% full-157 figure applies only to `deepseek-full`.

**Previously recorded allocation baselines that become stale custody if the
policy changes** (listed, not re-derived):

* `issue100/stride3-comparison-results.md` — Git53 49,332,224 allocated /
  48,951,284 logical. **Recorded under a different product seal and boundary;
  cited as recorded, no ratio re-derived.**
* `issue113/page-size-ab-protocol.md` §11.2 — the stride-3 53-state allocation
  table (C 65,056,768 allocated / 64,598,016 apparent post-verification;
  X 84,606,976 / 75,890,688; performance-boundary dir apparent 70,816,106 →
  82,116,970).
* §11.1 — the `scattered-100` tier rows (tier 10/100/500 allocated
  11,763,712 / 107,315,200 / 539,947,008 for C and 13,369,344 / 112,721,920 /
  567,869,440 for X).
* `issue112/scattered-100-repetition-and-attribution.md` — the n=22 ladder whose
  C median is 304,673,000 ns. **The anchor measured here (306,224,188 ns) is
  +0.5% against it, so the ladder is still valid for arm C at 4 KiB and becomes
  stale only for whichever class a switch is applied to.**

---

## 7. Correctness, protocol, and what was not resolved

### 7.1 Gates

| gate | C 4096 | X 65536 |
| --- | --- | --- |
| full-157 performance (157 states) | PASS | PASS |
| full-157 verification (157 states) | PASS | PASS |
| A/B slots 1–7 | all samples `status: pass`, `completion_status: COMPLETE` | same |

No fixture, timer, verification assertion, comparison threshold, target value or
historical receipt was changed. No compaction, VACUUM, repack, background
rewriter, resource-limit or timeout change. `slab_send_blocked_ns` is reported
separately and never added to wall time.

### 7.2 Not resolved

1. **Where exactly the class boundary sits.** The set bounds it (share ≥0.10,
   create-path, large Store) but does not locate the crossover. Slot 7 at
   −4.83% / +6.16% is near break-even and is the closest thing to a boundary
   datum; it is one point, not a threshold.
2. **Slot 1's mechanism is identified but not fully attributed.** The 8.78×
   incremental RSS and 1.63× command wall are block-consistent, but I did not
   instrument the allocation site, so I cannot say whether the RSS is dirty
   page-cache pages, page-addressing structures, or the spool.
3. **`edit_length_changing` / `edit_canonical_chunk_count` were not run.** The
   screen marks them ≈0-or-negative like slot 1; given slot 1's wall/RSS finding
   they should be measured before any edit-class policy is fixed (see the
   unverified-member note below).
4. **The `store-footprint` route did not expose `page_count`**, so slots 1, 5, 6
   show `—` for Store geometry. Their apparent/allocated deltas come from the
   recorded boundary observation and are complete; the page geometry is not.
5. **Only `namespace-10000` was run**, not 100/1000/100000. The screen's
   `namespace-100000` share (0.151) is the lowest candidate in the family and was
   not re-measured.

### 7.3 One thing I want to flag explicitly

The screen predicted the SDK edit class would be "≈0 or negative". On its
registered timer it is ≈0. On the product command wall it is a **+63% regression
with 8.78× incremental RSS**. I did not pre-register `command_wall_ns` as slot
1's decision variable, so I am reporting it as a finding and **not** promoting it
to the headline. But if a future policy applies 64 KiB to *any* edit path, that
number is the one that falsifies it, and it needs a dedicated run.

---

## 8. Custody — seals and receipt paths

Verified before the runs and re-verified after: **unchanged**.

| | arm C | arm X |
| --- | --- | --- |
| worktree | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-pagearms/C` | `.../X` |
| frozen commit | `9867371af5acae1363efd04dde0d0fa248460063` (detached) | same |
| `LAYERFS_SOURCE_SEAL` | `00f158e9e0a807e5660018301ad8251c7ee67b82e1f494f981fcfa8c50992a86` | `8215d9fb8a107da5d5615a78b79bbfcf6fb6ffcbc73528019201acedc89673d4` |
| `LAYERFS_PRODUCT_SEAL` | `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4` | `8e5ea0acaf1caac1d1bbe7e4515aaefcf7cd6bd81a5c7e035ad26eb012751b74` |
| host binary sha256 | `fc461532ddb5679db7ef3c720d64bec3ba49eeae2aeba22dc2041fef4d87ce44` | `817e7fa763b8863a725ee7123ab405fdc8ae98544dca260544782bd383b07b69` |
| image | `layerfs-bench-infra:00f158e9e0a807e5` (`sha256:4d644db6…`) | `layerfs-bench-infra:8215d9fb8a107da5` (`sha256:e3dfac23…`) |
| `HARNESS_IDENTITY` | `216545afda554446db84b4169fc746cd53dabad66baca5fbf616dc5f55d2a6a8` | **identical** |
| `WORKLOAD_SHA256` | `c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b` | **identical** |
| base diff sha256 | `8a453ae2b45479e034fffaeb9a762e59c774928139b0844022cf655edc34d94e` | **identical** |

Both worktrees re-verified as differing from each other only at
`schema.rs:14`. The shared checkout `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`
still reads `NEW_STORE_PAGE_SIZE_BYTES = 4096`.

**Retained receipts**

```text
layerfs-pagearms/evidence/
  full157/C/{identity,performance-summary,performance-manifest,verification-summary,
             verification-manifest}.json + deepseek-full/performance-step-{1..157}.json
           + deepseek-full/verification-step-{1..157}.json
  full157/X/… same shape
  full157/full157-series.csv              per-state series, both arms
  full157/full157-summary.json            derived summary (flattening, segments, slope)
  ab/C/<case>/block<N>/perf.jsonl         slot 1,2,3,4,5,6,7
  ab/X/<case>/block<N>/perf.jsonl         same
  ab/ab-tables.md                         derived per-slot tables
  ab/ab-summary.json                      machine-readable per-slot summary
```

Analysers (reproduce every number in this document):
`full157-series.py`, `analyze-ab-set.py` (`evidence/ab/ab-tables.md`),
`run-page-size-ab.sh seals` (seal reproduction).
The full-157 driver is `run-full157.sh`; the A/B remainder driver is
`run-ab-remainder.sh`.

**Samples retained, none discarded:** slots 2 and 3 additionally retain a third
interleaved round (blocks 9,10,11,12) and slot 3 retains `C/block13` as the
round-3 top-up for the truncated `C/block12` (3 of 6 samples, reported and
excluded from pooled statistics). All windows are listed in §3.2.

---

## 9. Post-decision follow-up — unexplained costs on `scattered-100`

Recorded after the decision, from the retained slot-4 receipts. **These do not
revise any result above and do not change the recommendation**; they are the
things an optimization pass would have to explain first.

### 9.1 `wall_ns` is ~7× the timer and nearly invariant to page size

| arm | `pure_call_sum_ns` (4 canonical blocks) | `wall_ns` (4 blocks) | wall/timer |
| --- | --- | --- | ---: |
| C 4096 | 305,870,562 / 305,191,187 / 305,298,396 / 306,224,188 | 2,103,486,604 / 2,136,053,270 / 2,100,395,104 / 2,252,556,062 | 6.88 / 7.00 / 6.88 / 7.36 |
| X 65536 | 178,020,146 / 176,847,042 / 183,164,562 / 179,376,938 | 2,081,483,479 / 2,056,184,438 / 2,169,705,876 / 2,070,242,520 | 11.69 / 11.63 / 11.85 / 11.54 |

**The timer moves 41.6%; `wall_ns` moves ~1.3%.** `host_orchestration_ns` matches
the timer to ±0.2% and `orchestration_unattributed_ns` is ~200,000 ns, so the
~1.8 s gap is entirely outside the orchestration scope. The measured timer
governs ~15% of wall (C) / ~8.6% (X). **A perfect timer improvement on this case
is worth ~1.3% of wall** — which is the fact that should scope any optimization
attempt. The composition of that ~1.8 s is not resolved here.

### 9.2 `slab_send_blocked_ns` ~3× the timer, ratio-invariant to the treatment

Producer rows are **per-worker** (4 per sample). Pooled ranges (ns):

| arm | producer `blocked_ns` | producer `wall_ns` | blocked/wall | aggregate `slab_send_blocked_ns` |
| --- | --- | --- | ---: | ---: |
| C 4096 | 214M … 283M | 260M … 332M | ~0.85 | 975,111,620 |
| X 65536 | 97M … 140M | 159M … 187M | ~0.85 | 501,194,781 |

Producers spend ~85% of their wall blocked **on both arms**, so this is a
structural producer/consumer property, not a page-size artifact — and at ~1.0 s
it is ~3× the timer. **No mechanism identified.** It should be explained before
any small timer-level win on this case is trusted.

### 9.3 Tier trend is non-monotone

§11.1: X/C is 0.670 (tier 10), 0.567 (tier 100), 0.677 (tier 500). The win
shrinks on both sides of 100, which normally indicates two competing effects
crossing over; they were not identified. Tuning at tier 100 will not transfer
to tier 500, and tier 100 is the most favorable point rather than a typical one.

### 9.4 Screen residual

The screen model (`pages_newly_written × 4,500` ns) over-predicted on this case:
117.8 ms predicted vs 131.1 ms measured. That ~10% residual is unexplained and
lies inside the quantity an optimization would target.

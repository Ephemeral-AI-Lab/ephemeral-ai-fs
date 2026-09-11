# Direction 3 — profiling the consumer's commit

Issue #114, recommended ordering step 3. **Diagnosis only: no product change, no
receipt/timer/threshold/verifier change.** Reads retained receipts and product source.

## 1. Why this direction exists
`run_finalized_output` (`objects.rs:372-509`) is **4 producer threads → 1 consumer
thread**. The consumer is provably saturated (`slab_consumer_idle_ns` 1.7–6.6 ms;
consumer busy 99.0–99.6%; `slab_queue_peak` pinned at the 4-slot bound every sample).
On scattered-500 the consumer's `sql_commit_ns` is 573.67 ms of a 1,319.37 ms timer
(**43.5%**), with four producers blocked behind it. That is the one place the machine
is idle and the work is serial.

## 2. The consumer path, from source
`layerstack.rs:1410` → `|page| admission.admit_page(page)` → `objects.rs:3802`:

```text
admit_page(page)
  └─ for each object: admit_object()
       └─ probe_incoming()                      (objects.rs:3837)
            ├─ session.retain_possible_ids()    SQL
            ├─ db.object_locations(&ids)        SQL
            ├─ seen.insert_page()               SQL
            ├─ admission::compare(...)          SQL (+ read of object bytes)
            └─ db.note_physical(stats)
       └─ push_pending(...)  → flush_batch() when full
            └─ consume_checked_owned_page()     SQL commit   <-- sql_commit_ns
```

All of this runs on the single consumer thread. `flush_batch` (`objects.rs:4118`) is
the term measured as `sql_commit_ns`.

## 3. The flush trigger — measured, not assumed
`push_pending` (`objects.rs:4052`) flushes when **either**:

| trigger | constant | value |
|---|---|---|
| object count | `PHYSICAL_ADMISSION_BATCH_COUNT` = `INITIALIZATION_SLAB_OBJECTS` | 512 objects |
| payload bytes | `2 × INITIALIZATION_SLAB_BYTES` | 524,288 B |

**Measured: the byte cap is the binding one.**
`admission_batch_peak_payload_bytes = 524,281` on both arms at tier 500 — within
**7 bytes** of the 524,288 B trigger (ratio 1.0000). `admission_batch_peak_objects` is
only **36–41** objects, i.e. ~8% of the 512-object cap.

So batches are byte-bound at ~512 KiB, not object-bound. Raising the object cap would
change nothing; the byte cap would still fire.

## 4. The decisive measurement: what drives `sql_commit_ns`

| tier | arm | batches | rows | `sql_commit_ns` | commit/batch | commit/row |
|---|---:|---:|---:|---:|---:|---:|
| 10 | C | 3 | 662 | 16.05 ms | 5,350 µs | 24.24 µs |
| 100 | C | 26 | 5,967 | 155.00 ms | 5,961 µs | 25.98 µs |
| 500 | C | 129 | 29,544 | 573.67 ms | 4,447 µs | 19.42 µs |
| 10 | X | 3 | 662 | 3.17 ms | 1,056 µs | 4.79 µs |
| 100 | X | 26 | 5,967 | 28.52 ms | 1,097 µs | 4.78 µs |
| 500 | X | 129 | 29,544 | 143.58 ms | 1,113 µs | 4.86 µs |

Two independent controls isolate the driver:

1. **Same batches, same rows, different arms.** At tier 100 both arms run **26 batches
   and 5,967 rows**, yet C costs 155.00 ms and X costs 28.52 ms — **5.4×**. So the cost
   is neither batch count nor row count.
2. **X arm per-batch cost is flat.** 1,056 / 1,097 / 1,113 µs across 3 / 26 / 129
   batches — within 5% across a 43× range in batch count.

**The driver is pages written**, and the cost is **~4,417–5,921 ns per newly-written
4 KiB page** — the same constant #113's screen calibrated (`4,500 ns/page`), now
confirmed *independently*, from the consumer side, at three tiers.

This is a genuinely useful convergence: #113 derived 4,500 ns/page from the A/B *speed*
delta; §4 here derives it from the consumer's own `sql_commit_ns` per page. **Two
different measurements, same constant.**

## 5. Consequence for the parallelization hypothesis

**The consumer is not doing parallelizable CPU work.** Its dominant cost is
**per-page SQLite I/O** (page writes at ~5 µs/page), not computation. This materially
weakens the "add consumer threads" direction:

- Splitting one SQL session across threads is not obviously safe (`admit_page` closes
  over a single `CheckedOutputAdmission` with a shared `session` and `&mut self` batch).
- Even if it were, the bound is **the page-write path through one writer connection**.
  `configure_connection` opens the writer as a single connection with `journal_mode =
  MEMORY` and `synchronous = OFF`; threads sharing it would serialize on that connection's
  mutex, and threads with separate connections would collide on SQLite's write lock.
  Either way the serial work does not disappear.
- The cost is in-memory page work (§8), so it also cannot be hidden behind I/O wait.

**Verdict: parallelizing the consumer is unlikely to pay.** The consumer is a
serialization point by design, and its cost is the page-write cost the 64 KiB treatment
already targets.

## 6. Where the real opportunity is (and it is #113's, not a new one)

The `sql_commit_ns` term **is** the page-write cost. Reducing it means writing fewer
pages — which is exactly the page-size lever:

| tier | C pages | C commit | X pages | X commit | pages removed |
|---|---:|---:|---:|---:|---:|
| 10 | 2,852 | 16.05 ms | 186 | 3.17 ms | 15.3× |
| 100 | 26,178 | 155.00 ms | 1,704 | 28.52 ms | 15.4× |
| 500 | 129,880 | 573.67 ms | 8,464 | 143.58 ms | 15.3× |

So Direction 3's result is that **the consumer bottleneck and the page-size lever are the
same cost** — not two targets. The consumer is not a separate optimization opportunity;
it is the *mechanism* by which 64 KiB wins.

## 7. Honest headroom statement

- **Removable by parallelizing the consumer: ~0.** The bound is page-writing I/O behind
  SQLite's single writer, not thread-starved CPU.
- **Removable by writing fewer pages:** 155.00 → 28.52 ms at tier 100 (C→X), i.e.
  **126.5 ms of the 306.22 ms timer (41.3%)** — but that is the page-size treatment #113
  already scoped, with its +5% storage penalty at 18 KB objects, and it is **already
  measured**. No new product change is implied.
- **On `scattered-100` specifically:** the timer is 14.4% of wall, so even a full
  126.5 ms recovery is ≤6% of `wall_ns`. Not worth pursuing on the negative control.

### Falsifiable test, had a consumer change been proposed
> A consumer-side change must reduce `sql_commit_ns` **while holding pages written
> constant**. If `sql_commit_ns` tracks pages written and not the change, the change is
> not addressing the bound. Conversely, any change that reduces `sql_commit_ns` only by
> writing fewer pages is the page-size lever, not a consumer optimization.

## 8. Why the ~5 µs/page is CPU/btree work, not fsync

Verified from `schema.rs`, not assumed. The normal publication path is configured by
`configure_connection` (`schema.rs:507-520`):

| pragma | value | consequence |
|---|---|---|
| `journal_mode` | **MEMORY** | no journal file on disk; rollback log lives in RAM |
| `synchronous` | **OFF** | **no fsync per commit** |
| `cache_spill` | **OFF** | dirty pages are not evicted mid-transaction |
| `cache_size` | `-SQLITE_PAGE_CACHE_KIB` = **32 MiB** | page cache |
| `temp_store` | MEMORY | |

WAL is **explicitly rejected** on connect (`schema.rs:483-485`,
`StoreError::WrongStoreSchema`), with a dedicated test at `schema.rs:711-749`.

So `sql_commit_ns` is **not** durable-I/O bound: there is no per-commit fsync to
amortize, and no journal-file append. The ~5 µs/page is **btree insert + page-cache
allocation + in-memory rollback bookkeeping**, plus the eventual spill-free write of
dirty pages at close.

**This makes the §5 verdict stronger, not weaker.** If the cost were fsync, more threads
(more concurrent writers) would lose badly to SQLite's single-writer lock. Since it is
in-memory page work under a *single writer connection*, the bound is the one writer's
serial page-write path — which is exactly what the object-shape/`synchronous` model says,
and exactly what writing fewer pages addresses.

Corollary: the ~5 µs/page is a *page-fill + index-insert* constant, so it should be
sensitive to object size and btree fan-out — which is why the X arm (64 KiB pages,
15.3× fewer pages) sees 1,056–1,113 µs/batch instead of 4,447–5,961 µs.

## 9. What was NOT established
- The internal split of `sql_commit_ns` between page allocation, btree insert and the
  in-memory rollback log. `sql_begin_ns` is only 0.50 ms total (0.09% of commit), so the
  time is inside the commit, not the begin — but commit is not further decomposed by any
  existing counter.
- **No consumer change was attempted**, so the "unlikely to pay" verdict in §5 is a
  mechanism argument from the measured cost shape plus the pragma configuration, **not**
  an A/B result. It is falsifiable by the test in §7.
- Whether the eventual dirty-page spill at Store close is inside or outside
  `sql_commit_ns` was not determined.

## 10. Receipts
Retained only; no new measurement was run for this direction.
- `evidence/ab/{C,X}/dedup-cdc-scattered-100/block*/perf.jsonl` (tier 100, n=24/arm)
- `evidence/speed/{C,X}/scattered-{10,500}/perf.jsonl` (n=11/11, 5/5)
- Source: `crates/layerfs-layerstack-store/src/{objects.rs,layerstack.rs}`

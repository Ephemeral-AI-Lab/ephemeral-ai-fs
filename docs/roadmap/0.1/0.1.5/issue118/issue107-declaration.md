# #107 prospective target and mechanism declaration

Status: **declared before implementation and measurement** (owner requirement: the
mechanism, numerical target and latency/resource constraints are frozen before
the experiment; the earlier ≥1% figure was a proposal, not a gate).

Frozen measured baseline (ordinary schema10 `deepseek-full` full157 Store,
`full157-candidate-1/census.json`, Store SHA256
`0e767a7f4078f040de165e6138b64ed7ed5ce1ea1f1c88ed8fc8df6f3cc523f5`):

| Quantity | Bytes |
|---|---:|
| Store allocated (`store_allocated_bytes`) | 83,935,232 |
| Store apparent | 83,525,632 |
| Matching Git157 allocated | 56,197,120 |
| LayerFS excess | 27,738,112 (≈49.36%) |

## Census selection: which avoidable cost

The census decomposes the allocated bytes as follows (`reconciliation`,
`allocated_components`, and a direct `dbstat` read of the frozen Store):

| Component | Bytes | Share | Avoidable by a bounded ordinary-write change? |
|---|---:|---:|---|
| `object_packs` B-tree pages | 77,987,840 | 92.9% | — |
| — of which encoded pack payload | 75,695,515 | 90.2% | no: it is the authenticated content/metadata itself |
| — of which SQLite overflow-page slack (`sqlite_pack_page_unused_bytes`) | 2,142,701 | 2.55% | **yes** |
| `objects` locator table | 5,382,144 | 6.4% | no: 104,705 rows at ≈51 B (32-byte exact CAS id + length + locator) is already minimal |
| other trees + allocation rounding | ≈430,000 | 0.5% | no |

The remaining distance to Git is dominated by *content* costs that the census
shows are **not** avoidable by a small change:

- `small_DELTA` frames (27,702,575 B) versus Git's `blob_delta` (11,738,390 B):
  the LayerFS delta is a bounded Zstandard `refPrefix` frame, and a direct
  measurement of the workload's own changed-file pairs (24 consecutive-state
  pairs, 411,096 B of source) gives level-3 patch sizes of 23,382 B against
  20,283 B at level 19 — i.e. only ≈13% of theoretical headroom, at a large CPU
  cost, and any change here is a codec/format change.
- `small_FULL` frames (28,743,674 B for 9,340 objects) versus Git's `blob_full`
  (35,244,143 B for 12,159 objects): LayerFS is already *better* and finds more
  deltas (75,398 stored objects versus Git's 75,929 unique blobs, with only
  9,340 without a predecessor).
- metadata packs (8,611,510 B) encode per-entry authenticated values plus
  namespace pages that Git's tree format does not carry; changing them is a
  format change.

So the largest **avoidable** cost in the census is the 2,142,701 B of SQLite
overflow-page slack. Its cause is structural and measured: the ordinary path
creates **3,457 pack rows** (`pack_v1` 702 at 4,643 B avg, `pack_v2` 517 at
15,707 B, `pack_v4` 1,663 at 35,456 B, `pack_v6` 575 at 9,307 B) although the
existing, unchanged `pack::PACK_LIMIT` is 256 KiB. Each admission batch — a
bounded worker slab, not a Commit — opens a fresh pack row, so the row tail of
every batch leaves part of an overflow page unused. 3,236 of 3,457 packs are
≤64 KiB.

Bounded simulation of the mechanism on the frozen Store's real pack-byte
distribution (SQLite insert of the same 75,695,515 B as per-Commit packs filled
to the existing 256 KiB bound): 785 rows → 18,628 pages versus the current
19,100, i.e. **−472 pages = −1,933,312 B = −2.30% of allocated Store bytes**.

## Mechanism (smallest justified fix)

Fill the **existing** 256 KiB pack bound within one admission session instead of
opening a fresh pack row per bounded admission batch:

- keep the group vector of the last pack of each framing lane, and when the same
  lane is admitted again inside the same `AdmissionSession`, append its groups to
  the session's still-open pack row (one `UPDATE` in the same transaction that
  inserts the new object locators, with the locator group numbers offset by the
  open pack's group count);
- close and replace the open pack as soon as the merged pack would exceed
  `pack::PACK_LIMIT`, `GROUP_COUNT_LIMIT` (256) or `RECORD_COUNT_LIMIT`;
- otherwise behave exactly as today.

Unchanged by this change: pack version bytes, pack/record framing, the
`objects` locator (pack, group, record) semantics, exact CAS/dedup, canonical
authentication, DELTA base selection, legacy reads, schema version 10, 4 KiB
pages, `PACK_LIMIT`, every quota, the 2 MiB physical-output budget, the 6 MiB
data reserve, writer/transaction coalescing and failure atomicity (session
rollback still deletes every pack above the session baseline pack).

Explicitly out of scope and not used: compaction, VACUUM, post-workload
repacking, fixture rewriting, quota changes, external-library patches, codec or
compression-level changes.

## Frozen target and constraints

- **Target: ≥1.5% reduction of complete allocated Store bytes on the affected
  ordinary history** (full157: 83,935,232 B → ≤82,676,203 B), with the expected
  value from the census simulation being ≈2.3%. The focused development case is
  the registered `storage-smoke --storage-smoke deepseek-five` ordinary case.
- **Latency/resource constraints:** ordinary wall and CPU preservation use the
  contract's frozen n3 alternating-pair rule when a comparative screen is
  required (wall: median paired slowdown > max(15% of control median, 3 ms) and
  ≥2/3 pairs slower is a material regression; CPU: same rule with max(15%, 1 ms)).
  Commit/Init envelopes, the 2 MiB physical-output budget, the 6 MiB data
  reserve, the 8 MiB final-delta policy, the spool quota and the daemon's
  2 CPU / 2 GiB / no-swap / 256-PID benchmark policy are unchanged. The session
  holds at most one open pack per framing lane, each bounded by the unchanged
  256 KiB `PACK_LIMIT` (≤4 × 256 KiB = 1 MiB retained at any time).
- **Correctness gates (no waiver):** exact CAS, authentication, reference
  closure, legacy/compact/v1–v6 pack reads, DELTA reconstruction, publication
  atomicity and cleanup. Affected existing Store tests must pass unchanged.
- **Qualification plan:** one focused ordinary case during development; then one
  build, the affected matched ordinary history (full157) with its original
  transitions/oracles, a fresh physical census on the frozen new Store, and the
  matching Git control re-read read-only. No n3 full157 replay. Git allocation
  layout WARN (measured 56,197,120 B versus the recorded 56,373,248 B, with
  unchanged apparent/pack bytes) is preserved, never normalized.

Any ineffective variant is rejected promptly and retained with its log; no
unchanged reruns for a nicer median.

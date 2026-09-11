# Commit Stage 2 Phase A result and Phase B candidate contract (#111)

> **Status:** Frozen before the Phase B product edit and before any Phase B sample.
> Amends nothing in Stage 1. Phase A raw evidence stays byte-identical.

## 1. Phase A outcome — feasibility SUPPORTED

Frozen rule: [commit-stage2-phasea-contract.md](commit-stage2-phasea-contract.md) §5.
Screen: control-diagnostic arm (control product + Stage 1 diagnostic instrumentation
+ the bounded store group trace), `namespace-100000`, four independent fresh Stores
and containers in the declared order `k100#1, k100#2, k10#1, k10#2`
(`sequence-phasea.json`, sha256 `d1c014ddda436c6779738adaec736ff718ae3ca338e8ce53a9556522e55e0e99`). Evidence:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z/cells/stage2-control-diagnostic/`.

The contract requires the store trace to split physical reads into
metadata-record groups, pooled-value groups and other groups. The first screen run
split them only by pack version; that is not the declared split, so the screen was
re-run with the declared call-site split before the decision was taken. Both runs
are retained. The first run's frozen rule outcome was already PASS, and the second
run reproduces it with the correct split:

| Quantity (tree apply only) | k100 #1 | k100 #2 | k10 #1 | k10 #2 |
|---|---:|---:|---:|---:|
| `namespace_ns` | 252 033 125 | 253 572 541 | 63 179 042 | 71 009 916 |
| tree `read_auth_ns` (A) | 246 683 844 | 247 950 881 | 61 612 719 | 69 336 844 |
| group selections (R) | 5 206 | 5 203 | 1 282 | 1 275 |
| distinct `(pack, group)` (D) | 807 | 804 | 278 | 275 |
| repeated selections | 4 399 | 4 399 | 1 004 | 1 000 |
| — record-group selections (`site_wave`) | 1 967 | 1 967 | 485 | 485 |
| — pooled-value group reads (`site_pool`) | 3 239 | 3 236 | 797 | 790 |
| — metadata delta-chain selections (`site_chain`) | 0 | 0 | 0 | 0 |
| `object_locations` SQL lookups | 1 967 | 1 967 | 485 | 485 |
| table overflow | 0 | 0 | 0 | 0 |
| decoded bytes over selections | 78 573 840 | 78 540 071 | 19 236 762 | 19 257 892 |
| authenticated canonical bytes obtained | 8 038 029 | 8 038 029 | 1 970 640 | 1 970 640 |
| selection clock | 9 294 186 | 9 282 493 | 2 174 171 | 2 535 384 |
| decompression clock | 74 205 750 | 74 624 423 | 17 763 068 | 19 426 961 |
| authentication clock | 11 270 950 | 11 382 385 | 2 720 048 | 2 825 574 |
| engine leaf-decode clock | 31 148 698 | 30 910 085 | 9 619 539 | 10 191 670 |
| Commit `physical_delta.group_fetches` | 5 965 | 6 007 | 1 354 | 1 332 |

Frozen rule, both `k100` cells:

- **F1** `(R − D)/R` = **0.845**, 0.845 ≥ 0.40 — PASS.
- **F2** `R / group_fetches` = **0.873**, 0.866 ≥ 0.25 — PASS.
- **F3** `A·(R − D)/R / namespace_ns` = **0.827**, 0.826 ≥ 0.25 — PASS.

`k10` shows the same shape: `(R − D)/R` = 0.783 and 0.784.

### What the trace says

The tree apply authenticates and decodes 2 053 pages (2 000 compact inode leaves,
32 level-1 branches, 1 root) and obtains 8.04 MB of authenticated canonical bytes,
but performs **5 206 physical group selections over 807 distinct groups** and
decompresses **78.6 MB**, i.e. **9.8 decoded bytes per byte it needs**. The
repetition is not a hypothetical cache opportunity: it is the same immutable
`(pack, group)` selected again inside one bounded tree apply.

Attribution of the 5 206 selections (measured, not inferred):

- **3 239 (62 %) are pooled-value group reads** issued one per inode leaf by the
  per-object bounded pool reader `metadata.PoolRead`, which is constructed fresh
  for every `metadata_chain` call and therefore never carries its decoded value
  groups across sibling leaves. These are not wave selections: they happen one
  object at a time inside the store's chain walk (`site_chain` is 0 because the
  chain is one hop deep).
- **1 967 (38 %) are record-group selections**, one per single-object demand wave
  (`locations.len() == 1` in every wave; `within-call-shared demands` is 0 for
  every cell).
- **0 selections come from a metadata delta chain**, and no call in the tree apply
  reads more than one object at a time: `object_locations` is invoked once per
  demanded page, 1 967 SQL lookups for 1 967 pages.

The physical layout supports reuse within a bounded sibling wave: 2 202 objects
with inode-tree page lengths occupy only **230 distinct groups** (≈ 9.6 pages per
group), and the 100 102 pooled metadata values occupy **662 pool groups**
(≈ 151 values per group), while the committed pool groups are read 3 239 times.

### Ceiling, stated honestly

`R − D` is an **upper bound**, not a forecast: it counts repeats of a group
anywhere in the tree apply, whereas the candidate can only reuse inside one
bounded wave. Phase A does not measure an implemented optimization and sets no
performance number. Only the paired Phase B arms do that.

## 2. Chosen mechanism (ONE)

**A bounded authenticated batch is the unit of physical group work for the sorted
inode-tree engine.** Two coupled parts of one mechanism, not two candidates:

1. **Primary — bounded authenticated batch read of a branch node's children.**
   `Engine::edit`'s branch loop currently performs one point read per child:
   `ObjectStore::with_authenticated_canonical` → `CoreReader::with_authenticated_canonical`
   → `StoreDb::read_object_rows(&[id])` → the single-id fast path → one
   `visit_locations` wave of length 1 → one group selection. The candidate reads
   the children of one node in bounded chunks through a **canonical** batch read
   (`ObjectSource::read_authenticated_objects`, the existing bounded packed-read
   path used by `CoreReader`, wave-bounded by `VALIDATION_RESERVE` and
   `OBJECT_PAGE_COUNT`), so `object_locations` is resolved once per chunk and
   `visit_wave` selects and decompresses each physical group once for all demands
   that share it.
2. **Coupled — the existing bounded pool reader serves the whole batch.**
   `visit_wave` already decodes a metadata record group once for every target in
   that group; the candidate also constructs **one** `metadata.PoolRead` per
   record-group wave instead of one per object, so the already-existing bounded
   pool value cache carries sibling leaves instead of one leaf. No new cache
   type, no new capacity: the existing `≤128 groups` / `≤512 KiB retained`
   eviction, the existing `16 KiB`-per-miss decoded-work ceiling and the existing
   per-object logical-work guard are unchanged. The logical-work guard is applied
   per `expand` call exactly as today, so no per-object work ceiling is relaxed.

This follows the handoff's stated order: batching is primary, and the measured
trace shows batching alone cannot remove the 62 % pooled-value repetition because
those reads happen inside a per-object call, while the capacity/reuse trace does
support wave-scoped reuse. No speculative skipping is introduced, no cache is
invented, and no validation is weakened.

## 3. Exact boundaries

```text
crates/layerfs-content/src/object/access.rs
    + ObjectStore::get_authenticated_canonical_batch (default = per-object loop)
crates/layerfs-layerstack-store/src/objects.rs
    + ObjectBuffer override: deferred objects first, then one CoreReader batch
crates/layerfs-content/src/tree/batch.rs
    ~ Engine::edit branch loop: chunked batch prefetch, same per-child work
crates/layerfs-layerstack-store/src/objects/read.rs
    ~ visit_wave metadata branch: one PoolRead per record-group wave
    ~ metadata_chain takes the wave pool
crates/layerfs-layerstack-store/src/objects/metadata.rs
    ~ PoolRead::expand: per-call logical-work guard, shared value cache
```

## 4. Invariants the candidate must preserve

- **Every demanded page is still read, authenticated and decoded.** The batch
  replaces N point reads with one bounded batch read of the same N objects; the
  unchanged-range early return at `batch.rs:519` is untouched and no child is
  skipped speculatively.
- Canonical identity (`ObjectId::for_bytes`), page fill/minfill, child level and
  max-key (`check_child`), parent subtree count/bytes, canonical re-encode
  (`compact::decode_inode` round trip), root summary, ordering, and the
  `NonCanonicalPagePartition` / `InvalidRecord` rejection paths are unchanged.
- Key order, `changes` callback order, sibling merge order and encoded output are
  unchanged: children are still processed strictly in ascending key order.
- Bounded waves only: at most `OBJECT_PAGE_COUNT` ids per store call and a fixed
  child chunk; retained canonical bytes are charged to the existing 4 MiB
  `SORTED_TREE_UPDATE_SCRATCH_BYTES` ledger before allocation and released per
  chunk. No new worker, no unbounded history, no process-wide cache, no change to
  SQLite settings, page size or codec.
- Failure semantics are unchanged in kind: a missing, aliased or corrupt page
  fails the same operation with the same error class, and `changes.rs` keeps its
  existing `ObjectLimitExceeded | Unsupported` fallback.
- Resource: no swap, no OOM, no added threads; container limits, host job policy
  and worker counts unchanged.

## 5. Measurement (frozen before sampling)

Fresh paired arms. `control` = current dirty main product (SOURCE_SEAL
`490938083f8a7d7e…`, PRODUCT_SEAL `760eb0f20…`); `candidate` = the same tree plus
this one treatment. Harness, workload, fixture, collector, analyzer and report
generator are byte-identical across arms; the only difference is the product
treatment.

- Plain pairs, `n = 3` per cell, alternating order `C1,T1;T2,C2;C3,T3`, cells
  `nochange`, `retained`, `k10`, `k100`, `fuse-posix` — 30 cells.
- Diagnostic pairs, `n = 2`, alternating `C1,T1;T2,C2`, cells `retained`, `k10`,
  `k100` — 12 cells, reported separately and never pooled with plain rows.
- One fresh Store and one fresh container per cell; no untimed warm-up Commit; no
  sequential Commit on one Store counted as an independent repetition; cache
  policy `commit-study-os-uncontrolled`.
- Paired statistics: per-cell median of the per-pair difference
  (`candidate − control`), the median of paired ratios, and the raw per-sample
  values with ranges. Aggregates never substitute for per-sample gates.

### Gates

Worthwhile screen (all must hold in the plain cohort):

- ≥ 25 % reduction in paired K100 `namespace_ns` (receipt phase, per sample);
- a clear public-wall improvement in paired K100 Commit wall;
- lower K100 total user+system CPU in matched pairs;
- K1 `retained` Commit wall and `nochange` Commit wall within
  `max(10 % of control, 1 ms)` per pair (non-regression);
- every correctness, resource and custody gate below.

Preferred absolute targets (proposed experimental targets, not forecasts):
K100 namespace ≤ 120 ms, K100 public Commit ≤ 200 ms, K10 namespace ≤ 31 ms,
K10 public Commit ≤ 50 ms, K1 and nochange public Commit no material regression.
A result that misses 200 ms is recorded as a partial improvement, never
relabelled as the preferred target passing.

Correctness gates per cell: proof PASS; bootstrap identity 112 451 canonical
objects / 513 026 835 bytes / 100 002 pool values; expected `Created`/`UpToDate`
outcomes and head movement; complete changed-file bytes for all K files; 10
deterministic unchanged sampled files; visible head; `end_workspace_session(Clean)`
with zero active workspaces and executions; identical final root, canonical
object/byte counts and edited content after dropping owners and reconnecting;
zero swap and no OOM; container removed.

Resource gates: charged tree scratch high-water inside the 4 MiB ceiling and
reported before/after; host RSS/peak/threads; container memory current/peak,
pids, swap, OOM; spool bytes; disk read/write bytes per sample.

Mechanism counter gates (from the diagnostic cohort): fewer physical group
selections, fewer decompressions, fewer `object_locations` lookups, fewer decoded
bytes for the same authenticated canonical bytes, and the same number of pages
read/authenticated/decoded. A gain that requires reading *more* pages, decoding
more records, or skipping validation is a failure.

Adversarial correctness (focused suites on both arms): generic directory/inode
callers, boundary keys, split/merge, insert/delete, root growth/collapse,
hardlink references, reversion, rollback, low-scratch fallback, unchanged
malformed siblings (wrong level, wrong max key, wrong parent count, noncanonical
form, underfill, corruption, missing dependency) with equivalent rejection, and
fixed-N/vary-K plus fixed-K/vary-N work-count tests.

Falsification: if the paired K100 `namespace_ns` reduction is under 25 %, if any
canonical identity, verification or reopen proof differs, if CPU rises, or if any
charged scratch exceeds the declared budget, the candidate is a valid no-go and
the report records the measured blocker and a bounded alternative.

## 6. Non-claims

- No edit-stage optimization; the Stage 1 conclusion that ~86 % of the edit stage
  is unattributed stands, and no edit latency is promised.
- No claim of a "224 ms measured saving" or "30 ms of hashing": Stage 1's
  corrections 1–2 stand unchanged.
- No cold claim of any kind; cache state is uncontrolled and declared.
- Cold Init ≤ 2.7 s stays open and Init optimization stays paused.
- No release, tag, deployment or broader storage-policy change.

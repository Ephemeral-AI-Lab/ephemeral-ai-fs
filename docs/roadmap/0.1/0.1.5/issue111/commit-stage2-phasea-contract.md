# Commit Stage 2 Phase A contract — physical-group reuse feasibility (#111)

> **Status:** Frozen before any Stage 2 diagnostic edit or sample.
> Amends nothing in Stage 1; Stage 1 raw evidence and reports stay byte-identical.

Frozen at main `dcc3b939b` plus the preserved uncommitted compaction-removal
treatment whose full preexisting patch hashes to
`44fdf679f24deead8e5f0d7dc8b2d0dd3a06346d56d8477058ebc69db0f6c2cc` (identical to
the Stage 1 record). Stage 1 plain and diagnostic source seals reproduce exactly
(`490938083f8a7d7e…`, `6ca0b4576927d974…`), so the Stage 2 arms start from the
same measured treatment.

## 1. Question

Does the Commit namespace phase perform enough **repeated physical-group
retrieval/decompression** to support the bounded authenticated batch-read
candidate, and does that repetition carry a material share of `namespace_ns`?

This is a *feasibility* question. Phase A measures repetition and reuse
opportunity. It does not measure an implemented optimization and makes no
performance claim.

## 2. Why repetition is expected (unverified prior, not evidence)

- `Engine::read` (`crates/layerfs-content/src/tree/batch.rs:185`) reads exactly
  one page through `ObjectStore::with_authenticated_canonical`.
- `ObjectBuffer` implements `ObjectStore` but **not** `ObjectRead`, so the
  blanket `impl<T: ObjectStore> ObjectRead for T` supplies the per-object
  default `get_authenticated_batch` (`crates/layerfs-content/src/object/access.rs:17`).
- `ObjectBuffer::with_authenticated_canonical` falls through to
  `CoreReader(..)::with_authenticated_canonical` →
  `StoreDb::read_object_rows(&[id])` → the **single-id fast path** of
  `object_locations` plus one `visit_locations` call with one location
  (`crates/layerfs-layerstack-store/src/objects.rs:3618`, `read.rs:1682`).
- Each such call selects and decompresses one whole physical group
  (`extract_demanded_group`/`extract_record_group` + `pack::decode_group` or
  `decode_metadata_group`), even though a compact-namespace metadata group holds
  up to 16 KiB of canonical sibling pages and an ordinary group up to 16 KiB.

So N single-page reads inside one physical group may cost N group selections and
N decompressions instead of one. **This prior is explicitly not evidence.**

## 3. Instrumentation (diagnostic arm only)

Bounded, exact, aggregate. Fixed-size state only; nothing scales with namespace
size, delta count or history.

1. **Store group trace** (`layerfs-layerstack-store`), reset immediately before
   and snapshotted immediately after the sorted inode-table apply inside
   `FrontierInodes::finish`:
   - per `visit_locations`/`visit_wave` call: demands, selected groups and
     **within-call shared demands** (`demands − selected groups`), number of
     single-demand calls, and the single-demand group-selection count;
   - per group selection (`extract_record_group`, the single authoritative
     `group_fetches: 1` site): the `(pack_id, group_number)` key, its pack lane
     (metadata / pooled-metadata / legacy / native / small / whole /
     compact-small), encoded bytes, decoded bytes and an extraction clock
     (blob open, header, directory, encoded read);
   - decompression clock and decoded bytes at each `decode_group` /
     `decode_metadata_group` boundary, separately from the extraction clock;
   - authentication clock (`authenticate`, `check_canonical_format`) and emitted
     canonical bytes, separately from extraction and decompression;
   - **distinct-group table**: one fixed open-addressing table of 16 384 slots
     (`(pack_id, group_number) → fetch count`), 12 bytes per slot, 196 608 bytes
     total, charged and reported. It counts, per Commit-tree-phase, selections,
     distinct groups, repeats and table overflow. No unbounded history, no
     object-ID map, no payload retention, no eviction heuristic that changes a
     result.
2. **Engine counters** carried over unchanged from Stage 1 (`nodes_read`,
   `pages_skipped_unchanged`, `read_auth_ns`, `decode_ns`, `decode_records`,
   `value_ids_derived`, `read_bytes`, `peak_scratch_bytes`). Stage 1 numbers are
   not reused as Stage 2 measurements; the counters are re-collected.
3. **Phase boundaries.** `namespace_ns` remains the unchanged product clock.
   Store-trace clocks are nested inside it and are never added to a wall-time
   partition together with their parent. Instrumentation overhead is reported
   and never subtracted from a plain row.

## 4. Screen (declared before sampling)

Diagnostic arm only, `namespace-100000`, four independent fresh Stores and
containers, in this exact order:

```text
k100 #1, k100 #2, k10 #1, k10 #2
```

Each cell is a fresh public Init + fork plus the frozen K edit sequence and one
public `Client::commit_workspace_session`, exactly as Stage 1's collector ran
them. No untimed warm-up Commit, no sequential Commit repetition on one Store,
cache policy `commit-study-os-uncontrolled` (uncontrolled OS page cache; never
described as cold).

## 5. Frozen feasibility decision rule

Let, for the tree-phase store trace of one cell:

```text
S = single-demand group selections     (one page selected per call)
R = total group selections in the tree phase
D = distinct (pack, group) keys in the tree phase
W = within-call shared demands
A = read_auth_ns of the same tree apply (engine clock)
N = namespace_ns of the same Commit (unchanged product clock)
```

Feasibility is **supported** only if, in **both** `k100` cells:

- **F1** `R − D ≥ 0.40 · R` — at least 40 % of tree-phase group selections are
  repeats of a group already selected in the same phase;
- **F2** tree-phase `R` is at least 25 % of all group selections observed in the
  same Commit's namespace phase (the mechanism addresses a material share);
- **F3** the repeated-selection share of the tree's read clock,
  `A · (R − D) / R`, is at least 25 % of `N` (the ceiling can plausibly reach the
  worthwhile screen).

`k10` is reported for shape only; it is not a gate.

If F1–F3 do not all hold in both K100 cells, Phase A **fails** and the campaign
stops with the measured blocker plus a concrete bounded alternative. No cache is
invented and no validation is weakened to reach a target.

If F1–F3 hold, Phase B selects exactly **one** mechanism — bounded authenticated
batch reading of a node's children, preserving key order, authentication, decode
and every existing canonical/page/parent-child/partition check — and freezes a
separate candidate contract before any product edit.

## 6. Non-claims

- No optimization is implemented, promoted or timed in Phase A.
- Nothing here supports a removal of unchanged-sibling validation, a decoded-page
  cache, or any weakening of canonical/authentication checks.
- Phase A numbers are never pooled with plain rows and never used as a paired
  delta.
- The cold Init ≤ 2.7 s objective stays open; Init optimization stays paused.
- No release, tag, deployment or broader storage-policy change.

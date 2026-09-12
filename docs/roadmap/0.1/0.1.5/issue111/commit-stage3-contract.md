# Commit Stage 3: bounded frontier spill merge — measurement and qualification contract

> **Status:** Frozen before implementation sampling and before any performance
> collection. Written 2026-09-12 (UTC) in the #111 Commit line; the original
> cold-Init ≤ 2.7 s objective stays open. Repository
> `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`.
>
> Control is the **current accepted optimized product** (main `987af9367` plus the
> preserved uncommitted compaction-removal treatment, byte-identical), not the
> pre-Stage 2 product. Stage 2 is closed; its K10 absolute waiver stands and is
> neither reopened nor re-litigated here.

This contract freezes the defect, the treatment, the counters, the resource
bounds, the test matrix, the timing policy and the custody rules **before** the
candidate was measured. Numbers observed after this freeze cannot change the
questions or the gates, only the results.

## 1. Question and claim

One mechanism only: `FrontierInodes::merge_pending` in
`crates/layerfs-workspace/src/changes.rs` rewrites the entire accumulated sorted
spill on every pending flush. Claim to be tested:

> **structural-complexity**: replacing the single accumulated spill with
> size-tiered sorted runs reduces cumulative frontier spill record traffic from
> `Θ(K²/B)` to `O(K log(K/B))`, with unchanged results, bounded RAM, bounded
> temporary storage, and no path back to the quadratic merge.

A second, secondary claim is allowed only if measured:
**empirical-performance** for the declared spill-scale cases (§6), reported as
paired medians without a retro-fitted absolute target.

Stage 3 is an algorithmic scaling repair. K100 never spilt, so **no K100 speedup
is promised**; K100 ≤ 200 ms remains the accepted engineering goal and is
reported as a warning if a sample exceeds it. K10's ≤ 50 ms / ≤ 31 ms absolute
gates remain owner-waived and are not re-run as gates.

## 2. Defect statement (traced, not assumed)

`FrontierInodes` coalesces final inode changes in a bounded `BTreeMap` before
touching the immutable base inode table. Insertion into a full map calls
`merge_pending`, which in the control:

1. opens a new anonymous journal,
2. reads **every** record of the current spill (`self.count` rows) and merges the
   whole pending map into it,
3. writes all `count + new` rows back,
4. installs the new file as the only spill.

Flush *f* (1-based) therefore reads `B + (f−1)·B` records and writes `f·B`
records. Summing over `f = 1..K/B`:

```text
records_read    = Σ (f·B)          ≈ K²/(2B)
records_written = Σ (f·B)          ≈ K²/(2B)
records_total   = Σ (f·B + f·B)    ≈ K²/B
```

for `K` distinct frontier entries and pending capacity `B`. `record`/`spilled`
lookups are unaffected by the defect (binary search over the one spill,
`O(log K)`), so the defect is purely cumulative write/read traffic and the
resulting wall/CPU cost. The same file also carries a `ponytail:` note that
already names this behaviour.

## 3. Actual `B` (policy-derived, not an earlier estimate)

`CandidateInputs::build` sizes the frontier from the live policy:

```text
io_bytes        = journal_io_bytes(max_final_delta_memory_bytes)
                = clamp(max_final_delta_memory_bytes / 64, 256, 65536)
frontier_budget = max_final_delta_memory_bytes
                  − 4·(io_bytes.saturating_sub(256))
tree_scratch    = min((frontier_budget − 1024)/2, SORTED_TREE_UPDATE_SCRATCH_BYTES)
batch_size (B)  = 1 + (frontier_budget − 1024)/512
```

The `512` is the accounted reservation for one pending entry (256 B `BTreeMap`
entry + 256 B result/scratch allowance); the fixed 1 KiB term covers the first
map entry. At the shipped default `max_final_delta_memory_bytes` of 8 MiB
(`ResourcePolicy::default`):

```text
io_bytes        = 8 388 608/64 = 131 072 B                   (clamped: no)
frontier_budget = 8 388 608 − 4·(131 072 − 256) = 7 865 440 B
B               = 1 + (7 865 440 − 1024)/512 = 15 873
```

**B = 15 873 pending entries** at the shipped policy. `B` is a *derived* value,
recomputed from the live policy at every construction; the stage records it as a
measured quantity (`batch_size`) in the counters, never as a constant. The
`255`-entry estimate in the handoff is **wrong** and is not used anywhere in this
stage: it omitted the `·512` divisor and mis-derived `io_bytes`. Consequences
that the test matrix must cover:

| Multiple | Entries | Fits one public Commit in namespace-100000? |
|---|---:|---|
| `B−1` | 15 872 | yes |
| `B` | 15 873 | yes |
| `B+1` | 15 874 | yes |
| `2B` | 31 746 | yes |
| `4B` | 63 492 | yes |
| `8B` | 126 984 | **no** — 126 984 > 100 000 files |

`8B` therefore cannot be realised as a single public Commit in the original
namespace. It is covered by the deterministic production-budget matrix (§5), with
the largest declared public spill case being `k20000` (≈ `1.26·B`), and the
limitation is stated rather than worked around.

## 4. Treatment (the only intended product difference)

Replace the single accumulated spill with a bounded binary counter of sorted
runs inside `crates/layerfs-workspace/src/changes.rs`:

1. `merge_pending` still fires at exactly `pending.len() == batch_size` and still
   takes the whole map.
2. The map is written once as a new level-0 sorted run; no existing run is read
   or rewritten at that point.
3. `place_run(level, run)` installs the run if its level is empty, otherwise
   merges it with that level's run (older first, newer wins ties) and recursively
   places the result at `level + 1`. Level *i* holds at most one run whose size
   is between `2^i·B` and `2^(i+1)·B`, so a record is rewritten once per level it
   reaches: `O(log(K/B))` times. The top level absorbs unbounded growth, so the
   level count is bounded by construction (`log2(max_keys/B) + 1`, ≤ 32 in
   practice, hard-bounded at 64).
4. Level 0 is always the newest tier. `spilled` scans from level 0 upward and
   returns the first hit, so recency never depends on a run's size or file order.
   In-place updates (already-spilled key touched again) are applied **only** to a
   level-0 row; a hit in an older tier re-enters the bounded map instead, which
   keeps that tier's rows immutable and the recency order provable.
5. `finalize` (called from `finish`) flushes the remaining map as one run and
   performs exactly one `k`-way merge of all runs, newest first, dropping the
   duplicate older row for any key. Every row is read once and written once:
   finalization cannot reintroduce a growing-prefix rewrite.
6. A small presence prefilter (2 probes / key, 1 byte per reserved entry, no
   false negatives) skips the tier scan for keys that were never spilled, so
   ordinary commits that never spill pay one RAM test per lookup. A false
   positive only costs a scan; it can never change the visible record.
7. Where runs are opened, buffers are `clamp(io_bytes/4, 1 KiB, 16 KiB)` each.

Explicitly **not** part of the treatment: new workers, changed worker counts,
raised memory/worker limits, changed record encoding (the 192-byte record layout
is unchanged), a general sorting framework, storage-format changes, or any
fallback that restores the quadratic merge.

## 5. Deterministic counters and the pre-declared bound

New test-only instrumentation (`SpillStatCounters`) reports, for one
`FrontierInodes` instance:

```text
record_reads, record_writes, bytes_read, bytes_written
batch_flushes, merges, merge_levels
peak_live_runs, peak_open_files, peak_scratch_bytes, peak_live_bytes,
peak_merge_read_bytes, spill_keys, batch_size
```

`record_reads`/`record_writes` count every 192-byte row read/written, including
the final merge. Counters are `#[cfg(test)]` and are not compiled into the
release product.

**Bound derived before measurement.** With `F = ⌈K/B⌉` flushes:

```text
old: records_written = Σ_{f=1..F} f·B       = B·F(F+1)/2   → Θ(K²/B)
     records_read    = Σ_{f=2..F} (f−1)·B   = B·F(F−1)/2   → Θ(K²/B)
     records_total   = B·F²                                → Θ(K²/B)

new: batch flushes write B rows each                        → B·F
     merges  write (older + newer) rows; run sizes follow the
       binary counter, so Σ merge output = O(B·F·log2 F)     → O(K log(K/B))
     final merge reads every live row once and writes every
       unique key once                                      → ≤ 2·K
```

Qualification requires the measured candidate counters at **every** tested size
to satisfy `records_written ≤ B·F·(1 + ⌈log2 F⌉ + 2)`,
`merge_levels ≤ ⌈log2 F⌉ + 1`, and a growth factor below `2·` per size doubling,
while the control's model reproduces `B·F(F+1)/2` written and `B·F(F−1)/2` read.
The control's traffic is reported by the campaign's retained legacy model, which
is arithmetic on the control's own `count`/`batch_size` accounting
(`count` after flush `f` is exactly `f·B` for distinct fresh keys).

### Small-budget matrix (deterministic, unit level)

`B ∈ {8, 16, 32}` chosen through the real `ResourcePolicy`
(`max_final_delta_memory_bytes = B·512 + 1024 + 4·(io_bytes−256)`), so identity
`batch_size == B` is asserted, never assumed. Sizes `B−1, B, B+1, 2B, 4B, 8B`,
and a larger forced-flush size. Key orders: ascending, descending, interleaved
(bit-reversal), clustered (blocks), with repeated updates of live, spilled and
tombstoned keys; updates interleaved *with* flushes so post-flush revisions are
exercised. Deletion/reversion and hardlink reference adjustments are covered
through the existing `set`/`change_references` paths.

### Production-budget proof (at the shipped policy)

Because `B = 15 873`, the production-budget proof drives the **real
`FrontierInodes::new`** with the exact allocation accounting of
`CandidateInputs::build` at `ResourcePolicy::default()`, records `batch_size ==
15 873` from the constructed instance, and inserts `1B, 2B, 4B, 8B` entries (plus
`16B, 32B, 64B` in the explicitly selected structural sweep). This is a
**structural-complexity** proof (row traffic at the shipped budget), not a
latency or throughput claim, and it never relabels a reduced budget as production
performance.

The comparison rule against the control's model is frozen as follows, because the
candidate deliberately performs one extra coalescing pass that the control's
single always-coalesced spill does not need:

- `1B` and `2B`: no comparison claimed. The control has almost no accumulated
  spill to rewrite, so the quadratic mechanism is not yet dominant.
- `4B` and `8B`: candidate traffic must stay below `2×` the control model.
- above `8B` (structural sweep only): candidate traffic must be strictly below the
  control model, and the crossover ratio must fall monotonically across sizes.
- at every size: candidate traffic must grow by less than `3.5×` per doubling
  while the control's model grows by `factor + 1` (`4×` at `4B`, `9×` at `8B`,
  `65×` at `64B`), and must satisfy the derived `B·F·(1 + ⌈log2 F⌉ + 2) + 2K`
  write bound with `deepest_level ≤ ⌈log2 F⌉ + 1`.

## 6. Public-workspace cases (production budget, real namespace)

Fixture `namespace-100000` unchanged: 100 000 files, 1 000 data directories,
500 000 000 logical bytes, original pseudorandom content, digest
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`,
file mode 0640 / directory 0750 / mtime 1 700 000 000 000 000 000 ns including
the payload root. Frozen cells:

| Cell | K | Purpose |
|---|---:|---|
| `nochange` | 0 | ordinary regression screen |
| `k10` | 10 | ordinary regression screen (waived absolute gate, not re-gated) |
| `k100` | 100 | accepted Stage 2 cell; regression screen and ≤ 200 ms goal |
| `k32000` | 32 000 | declared spill-scale case: `≈ 2.02·B`, crossing two capacity boundaries and producing a third partial run at final flush with the shipped `B` |

`k32000` is the smallest public size that crosses **more than one** flush boundary
(`2B = 31 746`), which the declared spill-scale case requires; it costs ≈ 32 000
public `edit_workspace_file_range` calls at ~4.5 ms each (≈ 2.5 minutes of edit
preparation per cell, reported separately from Commit). `8B = 126 984 > 100 000`
files still cannot be realised publicly and is not attempted; it is covered by
the production-budget matrix (§5), which crosses 8 flushes and, in the explicitly
selected structural sweep, 64 flushes. No synthetic fixture is introduced and no
reduced-budget timing is reported as production performance.

Sequence shape: fresh public `Init` + `fork_branch` outside all Commit timers,
one fresh Store and one fresh container per entry, `K` single public
`Client::edit_workspace_file_range` calls (marker `C{K:05}{j:06}`, 10 bytes,
`namespace_edit_offset` rule), then the full public
`Client::commit_workspace_session`. Edit preparation is reported separately and
per-sample `edit + matching Commit` sums are computed. Commit cache state stays
declared uncontrolled (`commit-study-os-uncontrolled`); no cold claim, no extra
untimed warm-up Commit, no pooling of diagnostic and plain rows.

## 7. Evidence, harness and custody

New evidence root, created exclusively:

```text
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage3-evidence/20260912T000000Z
```

Arms are independently owned `git worktree` snapshots of this repository with
link count 1 on every file, no hard links, no shared `target/`:

| Arm | Treatment |
|---|---|
| `A` | current accepted optimized product + preserved dirty compaction-removal treatment (control) |
| `B` | `A` + the §4 spill treatment, and nothing else |

The collector is a fresh copy of the retained Stage 2 terminal collector, adapted
in the new root for the §6 cells and pointed at the new arms. The harness module
`benchmark/fs-bench-pro/src/commit_baseline.rs` (plus its `main.rs` dispatch) is
copied byte-identically into both arms from the Stage 2 terminal arm-C snapshot
(sha256 `9128dfe2ca27e3972679b4093f686a25ab319100f1563fde17f872d17951ffdb`), because
that file is retained-harness-only, is not part of the tracked product, and is
identical across every previous arm. `k20000` changes only the K loop bound and
the verification shape; the harness edit is identical in both arms and is sealed.

Per-cell custody: command, environment, container id, exit code, merged stdout,
container log, image id, binary sha256, source/product/workload/compilation
seals, fixture validation receipt, proof record and cleanup receipt. Identity is
re-verified before and after collection. The runner-owned
`layerfs-infra-measurement.lock` is held once by the collector parent; no child
double-acquires it. All prior evidence roots stay read-only and unexecuted; no
archived collector or analyzer runs in place; no attempt is deleted or
overwritten.

## 8. Timing policy (frozen now, applies to Stage 3 only)

Hard gates that timing can never waive: correctness, authentication, elimination
of the quadratic spill/fallback work, declared resource bounds, evidence
integrity.

For the ordinary regression screen (`nochange`, `k10`, `k100`) and the declared
spill-scale case (`k20000`): **three fresh matched pairs per case**, order
`C1,T1; T2,C2; C3,T3` with `C` = arm A and `T` = arm B. A **material wall
regression** requires the median paired slowdown to exceed
`max(15 % of the control median, 3 ms)` **and** at least two of the three pairs
to slow down. For user+system CPU, `max(15 % of the control median, 1 ms)`
analogously. Every sample, absolute miss, breach and allocator/resource effect is
reported; a material regression is investigated, not waived.

K10's absolute targets stay waived; a K10 miss is a reported warning only. A K100
Commit above 200 ms is a reported warning, not a failure of this stage. No
timing-based replacement of a sample: at most one complete affected-pair
replacement is allowed for demonstrated infrastructure invalidity, with the
original attempt retained. No median is moved by rerunning.

Any absolute target for the spill-scale case is set **from the measured
baseline** after collection and is reported as prospective context, never as a
retro-fitted gate.

## 9. Correctness and verification coverage

- All `layerfs-workspace` unit tests, including the extended spill suite.
- The real supported verification entrypoints through
  `benchmark/fs-bench-pro/verify-selected.py --verification` for the affected
  proof-only families with matching source/input/image identity.
- Public-cell correctness per case: declared `Created`/`UpToDate` result, visible
  head after every Commit, complete bytes of **every** changed file (`k20000`
  included: all 20 000), stated unchanged samples, final root and edited content
  re-verified after dropping owners and reconnecting the Store, clean workspace
  end, zero active workspaces/executions, container removed, zero swap/OOM.
- Cross-arm final canonical identity must be identical in every pair.
- Spill-scale resolution equivalence: the small-budget matrix compares the full
  final `(InodeId, record, tombstone)` set against an independent in-test
  `BTreeMap` model, and the public cells compare final canonical roots across
  arms.

## 10. Resource bounds (declared, then measured)

Accounting uses real `size_of`/capacity values:

| Component | Bound |
|---|---|
| pending map | `B` entries × (256 B reservation); unchanged from the control |
| presence prefilter | `next_power_of_two(B)` bytes, clamped to 1024..4 MiB, plus the filter key counter |
| merge scratch | `clamp(B·192, 4 KiB, 16 KiB)` bytes, reused for prefilter bit stamping |
| merge readers | ≤ (levels + 1) readers × `clamp(io_bytes/4, 1 KiB, 16 KiB)` |
| open files | ≤ levels + 1 run files + 1 merge output (≤ 34 in practice) |
| live temporary disk | Σ run sizes ≤ `K·192` B, ≤ `2·K·192` B during any merge |

`peak_live_runs`, `peak_open_files`, `peak_scratch_bytes`, `peak_live_bytes` and
`peak_merge_read_bytes` are reported from the counters; process post-call RSS is
reported as a snapshot only and is **never** called an operation peak.

## 11. Init overlap

Trace required before claiming no effect. `FrontierInodes` is constructed only in
`CandidateInputs::build`; Init (`initialize_layerstack`) constructs the canonical
namespace through the Init producer path and does not use the frontier
accumulator. If the trace confirms that, no Init speed effect is claimed and no
cold Init campaign is required; the affected Init correctness checks are still
run. If the trace instead shows shared use, a separately frozen cold Init
comparison would be required before any Init statement.

## 12. Completion

Deliverables: `commit-stage3-results.md` with the reproduced defect, the exact
fix, the derived bound, measured counters, the production-budget proof, the
resource accounting, paired timings/CPU with warnings versus material-regression
verdicts, correctness/verification coverage, exact seals, the append-only
evidence manifest, the Init impact assessment and the remaining limitations.
Commit only intended product/tests/docs; update #111 with #115/#108 and
provenance links. No release, tag or deployment; no bundling of edit-barrier,
reopened-history, cache or unrelated work.

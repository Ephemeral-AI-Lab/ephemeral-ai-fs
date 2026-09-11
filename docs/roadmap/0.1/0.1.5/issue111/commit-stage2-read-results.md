# Commit Stage 2: bounded authenticated batch reading — #111

Measured current-main product (treatment **promoted-uncompacted**, source seal
`490938083f8a7d7e…`, product seal `760eb0f2…`) against the same product plus the
one Stage 2 treatment frozen in
[commit-stage2-read-contract.md](commit-stage2-read-contract.md) (commit
`5490cf9bf`). The original cold-Init ≤ 2.7 s objective remains **open** and Init
optimization stays paused. No release, tag, deployment or broader storage-policy
change was produced.

Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z`.

## 1. Result summary

| Metric | Plain paired control (median) | Candidate (median) | Paired Δ median | Preferred target | Status |
|---|---:|---:|---:|---:|---|
| K100 namespace | 240.68 ms | **105.40 ms** | −135.28 ms | ≤ 120 ms | **PASS** |
| K100 public Commit | 314.66 ms | **176.59 ms** | −138.07 ms | ≤ 200 ms | **PASS** |
| K10 namespace | 60.87 ms | **30.98 ms** | −31.01 ms | ≤ 31 ms | **PASS** (margin 0.02 ms) |
| K10 public Commit | 82.33 ms | **48.04 ms** | −32.35 ms | ≤ 50 ms | **PASS** |
| K1 `retained` Commit #1 | 25.88 ms | 12.59 ms | −13.44 ms | no material regression | PASS |
| `nochange` Commit #1 | 1.99 ms | 2.10 ms | +0.11 ms | no material regression | PASS (≤ 1 ms per pair) |

**Worthwhile screen (frozen §5): PASS.**

- K100 paired `namespace_ns` ratios `0.4272 / 0.4417 / 0.4379` — every pair below
  0.75, median reduction **56.2 %**.
- K100 public wall fell in every pair: `−136.28 / −138.07 / −139.20 ms`.
- K100 total user+system CPU fell in every pair:
  `−134.29 / −139.47 / −138.27 ms`, median reduction **45.5 %**
  (302.12 → 164.71 ms).
- All correctness, resource and custody gates below pass in all 42 campaign cells.

The worthwhile screen and the preferred absolute targets are reported separately.
Both were met on the frozen rule as written; no target was relabelled and no
tolerance was widened after the fact.

## 2. What the treatment is

One mechanism: **a bounded authenticated batch is the unit of physical group work
for the sorted inode-tree engine.**

1. `Engine::edit`'s branch loop reads a node's children in fixed chunks of 32
   through a new `ObjectStore::get_authenticated_canonical_batch`, whose default
   is exactly the previous per-object loop and whose `ObjectBuffer` override routes
   the demands of one chunk into a single bounded
   `ObjectSource::read_authenticated_objects` call
   (`crates/layerfs-content/src/object/access.rs`,
   `crates/layerfs-layerstack-store/src/objects.rs`,
   `crates/layerfs-content/src/tree/batch.rs`).
2. `visit_wave`'s metadata branch builds **one** bounded `metadata.PoolRead` for a
   record-group wave instead of one per object, so the existing pool value cache
   carries sibling leaves (`objects/read.rs`, `objects/metadata.rs`). No new cache
   type, capacity, eviction rule or worker was added; the logical-work guard stays
   per `expand` call and only the decoded value cache is shared within the wave.

The exact diff is `candidate.patch` (sha256
`ffbe1fd1a793ab4c47901cd46cebb3a766b706e67c15e2534937f0974df2ea53`).

## 3. Mechanism counters, measured (diagnostic cohort, K100, medians of n=2)

| Counter (one tree apply) | Control | Candidate | Change |
|---|---:|---:|---:|
| pages read (`nodes_read`) | 2 053 | **2 053** | 0 |
| pages skipped as unchanged-range | 1 920 | **1 920** | 0 |
| pages entered (`in_range_checks`) | 132 | **132** | 0 |
| authenticated canonical bytes (`read_bytes`) | 8 353 409 | **8 353 409** | 0 |
| leaf records decoded | 101 001 | **101 001** | 0 |
| synthetic inode-value IDs derived | 101 001 | **101 001** | 0 |
| leaf-decode clock | 29.75 ms | 29.66 ms | ≈ 0 |
| canonical authentication calls / bytes | 1 967 / 8 038 029 | 1 956 / 8 007 945 | −0.6 % |
| authentication clock | 10.99 ms | 10.90 ms | ≈ 0 |
| physical demand waves | 1 967 | 127 | −94 % |
| waves with a single demand | 1 967 | 0 | −100 % |
| demands already covered inside their wave | 0 | 1 694 | new |
| **physical group selections** | **5 210** | **1 317** | **−75 %** |
| selections serving more than one demand | 0 | 242 | new |
| repeated selections (same group again) | 4 407 | 517 | −88 % |
| distinct groups | 804 | 800 | ≈ 0 |
| pooled-value group reads | 3 243 | 1 054 | −67 % |
| record-group selections | 1 967 | 263 | −87 % |
| `object_locations` SQL lookups | 1 967 | 64 | −97 % |
| group decompression calls | 5 210 | 1 317 | −75 % |
| group decompression clock | 72.21 ms | 14.22 ms | −80 % |
| decoded bytes over selections | 78.44 MB | 19.41 MB | −75 % |
| charged tree scratch peak | 164 556 B | 449 492 B | +285 kB, ceiling 4 MiB |

The same shape holds at K10 (selections 1 284 → 386, decompression
17.81 → 4.16 ms, SQL lookups 485 → 21, `nodes_read` 673 → 673).

**The candidate removes repeated physical group work without removing a single
required read, decode or check.** Every page the point route opened is still
opened, authenticated and decoded; the same 8 353 409 canonical bytes and the same
101 001 leaf records are decoded; page-skip, fill, minfill, child level, child max
key, parent subtree summary and canonical re-encode behaviour is unchanged.

Two small honest qualifications:

- `group_auth_calls` is 1 967 → 1 956 (−11) and authenticated bytes
  8 038 029 → 8 007 945. Eleven of 1 967 demands named an object that a sibling in
  the same bounded wave already named; the store's existing wave path resolves
  distinct IDs once and returns the same authenticated bytes to both demands, as
  it already does inside any wave. Every distinct demanded page is still
  authenticated; total decoded canonical bytes are identical.
- `group_selections` for the candidate (1 317) still exceeds distinct groups
  (800) because a wave-local pool reader is bounded: a pool group whose values
  span more than one 32-leaf chunk is read once per chunk. That is the declared
  bound, not an unbounded cache.

## 4. Plain paired cohort (n = 3 per cell, 30 cells, alternating C1,T1;T2,C2;C3,T3)

Milliseconds; median with [min, max] of the three independent fresh Stores.

| Cell | Commit #1 wall control | Commit #1 wall candidate | namespace control | namespace candidate | user+sys CPU control | CPU candidate |
|---|---:|---:|---:|---:|---:|---:|
| `nochange` | 1.99 [1.92, 2.08] | 2.10 [1.79, 2.13] | 0.00 | 0.00 | 0.55 | 0.50 |
| `retained` (K=1) | 25.88 [23.21, 26.42] | 12.59 [11.60, 12.98] | 5.70 | 3.44 | 16.39 | 8.13 |
| `k10` | 82.33 [80.39, 84.77] | 48.04 [43.68, 56.46] | 60.87 | 30.98 | 76.47 | 40.41 |
| `k100` | 314.66 [310.29, 322.13] | 176.59 [174.01, 182.93] | 240.68 | 105.40 | 302.12 | 164.71 |
| `fuse-posix` | 19.25 [18.94, 22.14] | 16.45 [12.82, 21.19] | 5.92 | 3.56 | 15.62 | 11.24 |

Per-pair wall and namespace differences:

| Cell | pair 1 | pair 2 | pair 3 |
|---|---:|---:|---:|
| `k100` wall / namespace | −136.28 / −138.02 | −138.07 / −133.87 | −139.20 / −135.28 |
| `k10` wall / namespace | −28.30 / −33.24 | −38.65 / −29.89 | −32.35 / −31.01 |
| `retained` wall / namespace | −13.44 / −2.53 | −10.62 / −2.36 | −14.27 / −2.21 |
| `nochange` wall / namespace | +0.11 / 0.00 | +0.21 / 0.00 | −0.29 / 0.00 |
| `fuse-posix` wall / namespace | +2.25 / −2.68 | −9.32 / −2.36 | −2.81 / −2.68 |

**Attribution, stated within what was measured.** For `k100` and `k10` the
namespace phase accounts for essentially all of the public-wall change
(`k100`: −138.0 / −133.9 / −135.3 ms of −136.3 / −138.1 / −139.2 ms). For
`retained` (K=1) it does not: the namespace phase improves by 2.2–2.5 ms in every
pair, while the wall difference is 10.6–14.3 ms and comes from
`object_admission_ns`, `checkpoint_ns` and `pause_fence_ns` — phases this
treatment cannot touch. In the separate diagnostic cohort those same phases moved
in the **opposite** direction on the same cell
(`object_admission_ns` +2.35/+4.73 ms, `checkpoint_ns` +1.85/+2.51 ms). The
K=1 Commit wall therefore carries uncontrolled session variance larger than the
mechanism's effect, and the report claims only the namespace-phase change there.
The frozen non-regression rule (`max(10 % of control, 1 ms)` per pair) passes for
both `retained` and `nochange` in the plain cohort; `fuse-posix` is reported
descriptively because the frozen allowance covers only those two cells
(+2.25 / −9.32 / −2.81 ms, median −2.81 ms).

## 5. Diagnostic cohort (n = 2 per cell, separate, never pooled)

| Cell | wall control | wall candidate | namespace control | namespace candidate | namespace ratios |
|---|---:|---:|---:|---:|---|
| `retained` | 18.75 | 22.52 | 5.85 | **3.78** | 0.681 / 0.616 |
| `k10` | 87.65 | 51.81 | 64.52 | **32.93** | 0.474 / 0.549 |
| `k100` | 318.11 | 172.02 | 240.83 | **106.01** | 0.449 / 0.431 |

Plain and diagnostic rows are never pooled and instrumentation overhead is never
subtracted. The diagnostic cohort reproduces the namespace reduction; its K=1
`retained` Commit wall is discussed above and is not a measurement cohort.

## 6. Correctness, resources and custody

All **42** campaign cells passed every gate; there were **zero problems** in the
analyzer's re-check of raw evidence. Per cell: `exit_code = 0`, container removed
(`cleanup PASS`), proof `PASS`, bootstrap identity exactly
112 451 canonical objects / 513 026 835 bytes / 100 002 pooled metadata values,
expected `Created`/`UpToDate` outcomes and head movement, complete bytes of every
changed file (1/10/100), 10 deterministic unchanged sampled files, visible head,
`end_workspace_session(Clean)` with zero active workspaces and executions,
`commit_operations` equal to the declared topology, phase equation
`Σphases + unattributed = total` for every Commit receipt, zero swaps, non-zero
container memory receipt, and an identical final root after dropping both owners
and reconnecting the Store. Every pair additionally required the control and
candidate arms to agree exactly on final canonical object count and encoded bytes;
K100 ends at the recorded reference `112 684 / 513 774 250`, K10 at
`112 483 / 513 140 485` (identical in all six K10 cells of both arms; no recorded
K10 reference exists, so this is a cross-arm identity gate, not a historical one).

Resources (plain K100): user CPU 263.99/266.22/269.72 → 146.60/142.71/148.52 ms;
system CPU 35.00/35.90/37.58 → 18.11/19.94/20.51 ms; post-call RSS 94.2/89.1/95.0
→ 88.7/89.1/90.5 MiB; threads 4 → 4 in every cell; disk reads 0 B in every cell;
container memory peak 19.8–20.5 MB against the 2 GiB limit; container pids 9;
runtime spool 0 B; no swap and no OOM in any cell. Phase-local operation peaks are
not claimed: the reported RSS is the harness's post-call snapshot, exactly as the
unchanged harness records it. Charged tree scratch peak rose 164 556 → 449 492 B
against the unchanged 4 MiB `SORTED_TREE_UPDATE_SCRATCH_BYTES` ceiling, and the
chunk's worst-case bytes are charged before the batch allocates.

Focused suites on both arms, debug profile (release test targets remain blocked by
the pre-existing `#[cfg(debug_assertions)]` gate in `layerfs-layerstack-store`,
recorded as a limitation, never worked around):

| Suite | Control | Candidate |
|---|---|---|
| `layerfs-content` | 57 passed, 0 failed | **62 passed, 0 failed** (5 new Stage 2 tests) |
| `layerfs-workspace` | 62 + 12 + 2 passed | 62 + 12 + 2 passed |
| `layerfs-layerstack-store` | 134 + 8 + 1 + 1 passed | 134 + 8 + 1 + 1 passed |
| `layerfs-fuse` | 31 + 6 passed | 31 + 6 passed |
| `layerfs-sdk` | all passed | all passed |

New work-count and adversarial coverage in
`tree::batch::tests` of `layerfs-content` (all counter-only, no timing claim):

- `stage2_batched_children_read_every_child_and_match_the_point_route` — a
  counting store proves the batch route reads the same number of pages, creates
  the same nodes and produces a byte-identical root as the point route, with no
  batch wider than the declared chunk and no child demanded twice inside a node.
- `stage2_batch_work_counts_follow_changed_keys_not_chunk_size` — fixed N, K =
  1/10/100; the demanded-page count equals the point route's `nodes_read`.
- `stage2_batch_work_counts_follow_namespace_size_at_fixed_k` — fixed K = 1 at
  N = 1 000 and 13 000.
- `stage2_insufficient_batch_scratch_is_the_declared_fallback_error` — a one-byte
  scratch ceiling returns exactly `ObjectLimitExceeded`, the error the workspace
  fallback matches.
- `stage2_malformed_siblings_are_rejected_identically_on_both_routes` — healthy
  control plus wrong child level, wrong child maximum key, inflated parent
  subtree count, underfilled child, corrupted child (`IdentityMismatch`), missing
  dependency, and a correctly addressed wrong-level page; each tampering is
  applied to two identical stores and the point and batch routes must return the
  **same** error. Clustered, spread and strided key sets are compared for equal
  roots.

Scaling ledger, kept open: the chunk charge is `chunks × 8 KiB` per live branch
level, so a deeper tree can charge more than one 4 MiB ledger and would take the
existing `ObjectLimitExceeded` → per-object fallback path; that path is asserted
but not measured at a deeper namespace. Triangular edit publication, quadratic
spill merge, repeated-reopen history and untested range/fallback paths are
untouched and remain in the ledger.

Custody: the preexisting tracked diff over `crates`/`benchmark`/`tools` is
byte-identical before and after the campaign
(`642ec707dce3e01a9ca0b91cc17a45ef040217835fca636a252a77331dbf2f61`), and the
untracked `compaction-removal.md` / `issue112/` / `issue113/` hashes are
byte-identical to the Stage 1 record. No stash, reset, revert, clean or
attribution of another owner's work occurred. The Stage 1, baseline and earlier
roots were read-only; no archived script ran in place. Raw cells live outside the
isolated source trees, and the campaign source trees were never modified by a
collection.

## 7. Limitations and non-claims

- The candidate is **not** promoted to `main` in this campaign. `objects.rs` is
  also modified by the preserved uncommitted compaction-removal treatment, so
  staging that file would have committed another owner's in-progress work. The
  exact treatment is preserved as `candidate.patch` for the owner's promotion
  step; committing it is a follow-up decision, not part of this bounded work.
- K10 namespace (30.98 ms) and K10 public Commit (48.04 ms) pass their preferred
  targets by 0.02 ms and 1.96 ms. The per-pair namespace ratios are
  `0.4837 / 0.5089 / 0.4901`, so the reduction is robust, but the absolute target
  margin is small and the K10 absolute pass should be read as a pass with little
  headroom, not as a comfortable result.
- No edit-stage optimization: the Stage 1 conclusion that ~86 % of the edit stage
  is unattributed stands, and no edit latency is promised. Nothing here removes
  the redundant per-barrier fact publication.
- No "224 ms measured saving" or "30 ms of hashing" claim: Stage 1 corrections 1–2
  stand. The measured saving is the paired difference of the frozen metric at the
  frozen boundary.
- No cold claim of any kind. Cache policy is `commit-study-os-uncontrolled` for
  every row; "retained" is a Store/index lifetime label.
- Cold Init ≤ 2.7 s stays open (last valid recorded median ~3.420 s) and Init
  optimization stays paused.
- The Stage 2 diagnostic instrumentation stays isolated in the diagnostic arms and
  is not promoted to permanent telemetry.

## 8. Next step

Promotion review of `candidate.patch` by the owner, then a rerun of every family
member that shares the changed call graph (`layerfs-content` tree engine,
`layerfs-layerstack-store` packed reader, `layerfs-workspace` Commit namespace
phase, `layerfs-fuse` callers) plus the reopened/retained and smaller-tier
sibling checks, before any release evidence is claimed. The scaling ledger's
deeper-tree chunk charge is the first follow-up proof.

Issue update:
https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5638943024

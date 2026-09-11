# Commit Stage 2 hardening contract (#111)

> **Status:** Frozen before the fixing edits and before every hardening
> measurement. Supersedes nothing: the Stage 2 read contract
> ([commit-stage2-read-contract.md](commit-stage2-read-contract.md)) and its
> report ([commit-stage2-read-results.md](commit-stage2-read-results.md)) keep
> their recorded status, including the promotion record. The original comment
> that called the treatment unpromoted is superseded by
> [commit-stage2-evidence-pointer.md](commit-stage2-evidence-pointer.md).

## 1. Question, claim and scope

The promoted Stage 2 product (`bd9dc1600`, product seal
`a608cd4edd25161584986b0f2885d2497a0c231a63a0b3dc7be73685bd0c0b38`) became the
per-object route's replacement for the sorted inode-tree engine. Three static
review findings say the promoted route departs from shared-reader behaviour in
ways the point route did not:

1. one `metadata.PoolRead` now serves a whole record-group wave, so the
   per-chain physical work allowance is shared by unrelated targets;
2. `ObjectBuffer`'s batch override emits owned objects before earlier unowned
   demands and skips the reader's page bound for an all-owned batch;
3. a fixed 32-child batch chunk reserves worst-case retained bytes without
   regard to the remaining ledger, and the Workspace per-object fallback
   re-enters the same compact batch engine.

This contract freezes how each finding is reproduced (or disproved), which
minimal fix is allowed, what must not change, and the exact measurement protocol.

**Claim under test:** the hardening restores exact shared-reader behaviour and
proves its bounds **without losing the measured Stage 2 win**.

Explicitly out of scope, not restarted and not claimed:

- Cold `init_namespace/namespace-100000` ≤ 2.7 s stays **OPEN** (last valid
  recorded median ≈ 3.420 s). No Init work, no cold claim.
- No new speed experiment, no broader storage-policy change, no new worker,
  cache size, page size or scratch allowance.
- Quadratic spill-merge removal, fresh edit-stage attribution, triangular
  publication and reopened-history scaling stay queued and untouched.
- No release, tag or deployment.

## 2. Source identity and custody at freeze

| Item | Value |
|---|---|
| Repository | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs` |
| HEAD at freeze | recorded in `custody/before-head.txt` |
| Pre-freeze status / tracked diff / untracked hashes | `custody/before-status-all.txt`, `custody/before-tracked-scoped.patch`, `custody/before-untracked-hashes.txt` |
| Main worktree PRODUCT_SEAL, promoted Stage 2 only | `a608cd4edd25161584986b0f2885d2497a0c231a63a0b3dc7be73685bd0c0b38` |
| Measured candidate PRODUCT_SEAL (Stage 2) | identical |
| Main worktree SOURCE_SEAL, promoted Stage 2 only | `4c46c963298e1eb5d4dacad062d8fcf886ef27ae7d90c5db910920f92a7e7ce5` |

Verified rather than assumed: the Stage 2 base arm (`stage2-candidate`, product
seal `760eb0f2…`) plus `candidate.patch` (sha256
`ffbe1fd1a793ab4c47901cd46cebb3a766b706e67c15e2534937f0974df2ea53`) reproduces
the promoted source and product seal `a608cd4e…` exactly, and the promoted
control arm of this campaign is that reconstruction. A custody breach during the
first arm copy, and its repair, are disclosed in
[commit-stage2-hardening-evidence-pointer.md](commit-stage2-hardening-evidence-pointer.md).

The dirty worktree is the promoted product **plus** the preserved uncommitted
compaction-removal treatment. The clean committed tree alone omits that
treatment, so every isolated arm is copied from the main worktree, not from a
clean checkout. The compaction-removal treatment and the untracked
`compaction-removal.md`, `issue112/` and `issue113/` files are preserved
byte-identically; no stash, reset, revert, clean or checkout occurs.

`objects.rs` carries both the Stage 2 hunks and preexisting compaction-removal
hunks. Only intended hunks may be staged, via a verified index patch, never the
whole file blindly. Preserved hunks and content are compared before and after;
git index blob hashes naturally change when HEAD changes and are excluded from
that comparison.

## 3. Evidence roots

| Role | Path | Mode |
|---|---|---|
| Stage 2 hardening root (this campaign) | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-hardening-evidence/20260912T120000Z` | new, create-exclusive |
| Stage 2 root | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-evidence/20260911T172337Z` | read-only, reused by copy |
| Stage 1 root | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z` | read-only |
| Baseline root | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z` | read-only |

No archived script, analyzer, validator or finalizer is executed in place. The
collector, analyzer and build script are copies adapted to this root's arm names.
Raw evidence is append-only; every attempt is retained, including failures.

**Custody disclosure, kept visible.** The first Phase A raw cells of the Stage 2
campaign were deleted and that loss is disclosed in the amended Stage 2 contract;
only extracted values survive. This campaign does not claim full historical
custody and does not attempt to reconstruct deleted raw evidence. The retained
Stage 2 Phase B paired results remain usable under their stated limits.

## 4. Frozen arms

| Arm | Tree | Treatment |
|---|---|---|
| `control` | `hardening-control` | promoted optimized product (Stage 2) only |
| `candidate` | `hardening-candidate` | control **plus** the three focused hardening fixes |

Both trees are independent self-contained repositories created inside this root
by copy from the main worktree at freeze, each carrying the campaign harness
(`benchmark/fs-bench-pro/src/commit_baseline.rs`) and the exact dirty treatment.
The root control is the **current optimized product**, never the pre-Stage 2
product.

Builds use exactly the shared entrypoints, one arm at a time, under the
runner-compatible `layerfs-infra-measurement.lock`:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
```

Compiler, profile and job policy are unchanged (`rust-1.85.1;release`,
`LAYERFS_HOST_BUILD_JOBS` = 8, Docker image jobs = 2). No crates/tools/benchmark
edit occurs between a build and the collection that uses it; a post-collection
`source_build_args()` re-check asserts the seals did not drift.

Recorded per arm: SOURCE_SEAL, PRODUCT_SEAL, COMPILATION_SEAL, binary SHA256,
image tag, immutable image ID, build wall, build stdout/stderr and exit codes.
**No dependent step runs after a failed build.**

## 5. Allowed fixes

### 5.1 Metadata pool work allowance (finding 1)

The decoded value cache stays shared across the wave: one bounded
`metadata.PoolRead` for a record-group wave is intentional and is the mechanism
that produced the measured win. The physical work allowance
(`decoded_work`, one 16-KiB unit per value-group miss, ceiling 32 MiB) becomes
the property of **one `metadata_chain`**, reset when a chain starts.

- The 512-KiB retention bound, the 128-group bound and the eviction rule are
  unchanged.
- The 192-KiB logical-work guard is unchanged and stays per `expand` call.
- The 32-MiB ceiling is **never** raised globally.
- Total wave work stays bounded by the fixed request count
  (`OBJECT_PAGE_COUNT`) times the single-chain ceiling; that product is
  documented rather than asserted as a new runtime limit.
- Healthy objects must not become `Integrity` errors because another object was
  requested alongside them.

A wave-aggregate work bound was considered and **rejected** for this change: it
would need new decomposition or fallback semantics, and the per-chain allowance
already restores the pre-Stage-2 property without adding one.

The moved 192-KiB `logical_work` counter is **not** treated as a regression:
`METADATA_EDGES + 1 = 17` nodes × 100 rows/leaf × 94 B/row = 159 800 B < 192 KiB,
further limited by the 128-KiB canonical closure. The proof is recorded as a
test; the guard is not removed speculatively.

### 5.2 Batch entry ordering and bound (finding 2)

`ObjectBuffer`'s batch override keeps one bounded source batch call for the
unowned subset, but:

- callbacks run in the declared demand order, so an owned object never overtakes
  an earlier unowned demand;
- the batch-count ceiling is enforced at entry, before any callback and before
  the source call, including for an all-owned batch that never reaches the
  source;
- duplicates are served once per demand; an empty batch performs no callback;
- a missing object, a callback error and an original-identity mismatch keep the
  same error semantics the point route has.

No unbounded result map is introduced: retained state is one bounded page plus
one locator pair per demanded object.

### 5.3 Chunking against the remaining ledger (finding 3)

`Engine::batch_children` narrows its chunk to what the remaining tree ledger can
actually hold, down to a one-child chunk read under the same per-page ceiling the
point route enforces, instead of failing on the full-width worst-case charge.

- Validation, error handling, per-page checks and bounded work are preserved.
- No recursive retry of the same failing batch is introduced.
- `TREE_BATCH_CHILDREN = 32` is unchanged for normal cases; no configurable
  tuning is added; the 4-MiB ledger is unchanged.
- The fixed 32-position search/remove loops stay as they are: they are bounded
  per chunk, not an unbounded K² defect, and are not optimised here.

### 5.4 What must not change

No new worker, no cache or page-size increase, no additional scratch allowance,
no relaxed correctness check, no weakened identity or canonical validation. A
correct partial outcome that misses the performance gates stays unqualified for
those gates; correctness is never traded for timing.

## 6. Reproduction rules

A finding is **reproduced** only by a test that fails on the control arm and
passes on the hardened arm, through the real code path. Static review alone,
synthetic counter mutation, or a test-only hook is not a reproduction. Findings
that actual constraints disprove are reported as disproved with the constraint
that disproves them, and are not called regressions.

Each fix is covered by a focused regression test in the isolated candidate tree,
and the same test is run against the control tree to record its control outcome.
Debug-profile `cargo test` is used; the existing `#[cfg(debug_assertions)]` gate
in `layerfs-layerstack-store` release test targets is recorded as a limitation
and is never worked around. Release binaries still run the public performance and
proof paths.

## 7. Measurement protocol

### 7.1 Fixture and workload

- N = 100 000 namespace files, K = 1 / 10 / 100 changed files per public
  `Client::commit_workspace_session`.
- Immutable original pseudorandom `namespace-100000` fixture, 100 000 files,
  1 000 data directories, 500 000 000 logical bytes, digest
  `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`,
  validated by a copy of `validate_fixtures.py` before each collection
  (full inventory/content plus file mode 0640, directory mode 0750, mtime
  1 700 000 000 000 000 000 ns, including the payload root). The fixture cache
  directory key is not the content digest. No byte-only fixture copy, no
  retroactive repair.
- Fresh public Init + fork per Store, outside the Commit timer. No prepared
  output Store substitute.
- Bootstrap identity 112 451 canonical objects / 513 026 835 B / 100 002 pooled
  metadata values; K100 final 112 684 objects / 513 774 250 B / 100 added values.
  Exact semantic identities are gated, not aggregate counts alone.
- Cache policy `commit-study-os-uncontrolled` for every row. "Retained" and
  "reopened" mean Store/index lifetime, never a cold claim. No extra untimed
  warm-up Commit.
- Read bytes, cache, route and sequence are reported per cell.

### 7.2 Plain paired cells and order

Frozen cells: `nochange`, `retained` (K1 marker/repeat/revert/noedit), `k10`,
`k100`, `fuse-posix` — n = 3 pairs per cell, order per repetition
C1,T1; T2,C2; C3,T3, cells in the declared order. The harness bytes are identical
across arms; the frozen sequence file is committed before collection.

Separate diagnostic pairs are created only where budget/decompression/fallback
attribution needs them. Diagnostic overhead is never subtracted from a plain row
and never pooled with one. **This campaign runs no diagnostic cohort**: the
hardening changes no pool capacity, chunk width for normal cases, or counter
definition, and the retained Stage 2 Phase B diagnostic counters stay the
attribution source under their stated limits.

### 7.3 Frozen gates

**Absolute targets** (declared before measurement, independent of the
control arm):

| Cell | Metric | Target |
|---|---|---|
| `k100` | public Commit #1 wall | ≤ 200 ms |
| `k100` | `namespace_ns` | ≤ 120 ms |
| `k10` | public Commit #1 wall median | ≤ 50 ms |
| `k10` | `namespace_ns` median | ≤ 31 ms |

Every sample is reported as well as the median. The K10 rows are explicitly
median targets, not per-call promises; a single K10 sample above 50 ms is
reported and does not by itself fail the median target.

**Per-pair non-inferiority**, frozen independently of the absolute targets:
for CPU (`user_cpu_ns + system_cpu_ns`) and public wall, the candidate must not
exceed `max(10 % of that pair's control value, 1 ms)`. Every breach is reported
individually. `fuse-posix` is reported under the same rule and additionally
marked descriptive.

**Worthwhile screen** (the measured Stage 2 win must survive): K100 namespace
paired ratio ≤ 0.75 in every pair, K100 public wall lower in every pair, K100 CPU
lower in every pair.

**Correctness and custody gates** per cell, re-checked from raw evidence:
`exit_code = 0`; container removed; proof `PASS`; bootstrap and final canonical
identities; every Commit's declared result and complete changed-byte count;
10 deterministic unchanged sampled files; visible head; `end_workspace_session`
clean with zero active workspaces/executions; `commit_operations` equal to the
declared topology; phase equation for every Commit; zero swaps; non-zero
container memory receipt; identical final root after dropping both owners and
reconnecting the Store; cross-arm equality of final canonical object count and
encoded bytes in every pair; runtime spool 0 B; no OOM.

### 7.4 Invalid attempts

Attempts are retained. At most one complete affected-pair replacement is allowed,
and only for demonstrated infrastructure invalidity (container, FUSE, daemon,
host or fixture failure), never for slow timing. No timing-based retry, no
selective deletion. A replacement reruns the whole affected pair in the declared
order and is recorded with its reason.

## 8. Analyzer correctness

The Stage 2 `analyze.py` summed all edit-stage work of a cell and then added only
Commit #1's wall time, which is not a per-sample chain. The new copy computes, per
sample, the edit work preceding each Commit and that Commit's wall time
(`chains()`), reports the full chain including every Commit, and keeps the
Commit #1 chain separate. A self-test with known intervals
(`python3 analyze.py --self-test`) covers it. The archived wrong aggregates stay
in the read-only Stage 2 root and are explained in the results; the standalone
Stage 2 Commit #1 and namespace measurements were per-commit receipts and were
unaffected.

## 9. Affected-caller qualification

Beyond the five performance cells, the changed call graph is qualified by
focused release-profile and debug-profile tests plus a static caller trace:
`layerfs-content` tree engine, `layerfs-layerstack-store` packed reader, metadata
pool reader, `layerfs-workspace` Commit namespace phase and per-object fallback,
`layerfs-fuse` callers and the `layerfs-sdk` public entrypoints, including all
registered `init_namespace` tiers where the namespace engine is exercised,
retained/reopened Commit, sparse pooled DELTA and eviction, mixed batch storage,
structural tree mutations, corruption rejection, low-budget/deep fallback and
rollback. Exact family/case IDs are frozen from the current registry; unaffected
#104 evidence is reused rather than blanket-rerun. Release qualification is
**not** claimed from the five performance cells alone.

## 10. Outputs

- `commit-stage2-hardening-results.md` in
  `docs/roadmap/0.1/0.1.5/issue111/`: findings, reproductions, fixes, exact
  source/product identities, every raw timing and per-sample target outcome,
  CPU/peaks/charged bounds, fallback work counts, correctness and affected-case
  coverage, custody disclosures and remaining blockers.
- An issue #111 update linking #115/#108/#109/#110/#106/#102/#104/#100/#107.
- Only intended fixes, tests and documentation are committed.

Hardening is reported **qualified** or **blocked**, per gate, with no rounding
up: a correctness-qualified result that misses a performance gate is reported as
unqualified for that gate.

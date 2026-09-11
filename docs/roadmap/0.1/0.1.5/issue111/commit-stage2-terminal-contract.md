# Commit Stage 2 terminal contract: trustworthy calibration and qualification (#111)

> **Status:** Frozen before any new measurement of this campaign and before any
> repair that this contract authorises. This contract **supersedes the comparison
> logic** of
> [commit-stage2-hardening-contract.md](commit-stage2-hardening-contract.md)
> §7.3 for the two questions it answers, and it identifies the design error in
> that frozen gate. Old gates and old results are **not** rewritten: the previous
> FAIL verdicts are retained verbatim and explained.

Cold Init ≤ 2.7 s and broader quadratic spill/edit/history work stay **open** and
are outside this claim. No release, tag or deployment.

## 1. What this campaign must settle

Two separate questions were conflated by the previous gate:

| # | Question | Correct comparator | What parity means |
|---|---|---|---|
| Q1 | Does the optimized read-batching still deliver its original benefit? | A (pre-Stage 2) vs C (hardened) | a failure of the optimization, not of the hardening |
| Q2 | Did the correctness hardening preserve the promoted product? | B (promoted Stage 2) vs C (hardened) | **success** — parity is the expected outcome |

**Design error in the previous frozen gate, stated plainly.** The hardening
contract's worthwhile screen required the hardened candidate to beat the
*promoted* product by ≥ 25 % in every K100 pair. The promoted product already
contains the read-batching optimization, so that screen asked the correctness
hardening to reproduce the Stage 2 speedup a second time. It is the wrong
comparator for "preserve the measured win". The previous campaign reported that
screen's FAIL honestly; this contract retains that FAIL as a true observation of
a mis-designed gate, and replaces it for Q2 with per-pair non-inferiority.
Q1 is answered by a fresh A/C comparison, which the previous campaign never ran.

## 2. Arms

| Arm | Meaning | Source | Product seal |
|---|---|---|---|
| **A** | pre-Stage 2 baseline | `15e3d48e0` (parent of the promotion commit) + preserved dirty treatment | `760eb0f2093488a6…` |
| **B** | promoted Stage 2 product | `a6f25eafe` + preserved dirty treatment | `a608cd4edd251615…` |
| **C** | hardened product | `259a80a4b` + preserved dirty treatment | `a54ef6e3f6d94fb1…` |

Each arm is an independently owned `git worktree` snapshot under this evidence
root. **No `cp -a -l`, no hard links, no shared `target/` directory**; every file
that could be written has link count 1. The campaign harness
(`benchmark/fs-bench-pro/src/commit_baseline.rs`, sha256
`9128dfe2ca27e3972679b4093f686a25ab319100f1563fde17f872d17951ffdb`) and the
harness `main.rs` are byte-identical across the three arms.

The preserved uncommitted compaction-removal treatment is applied to all three
arms, including its `objects.rs` module-declaration hunk. The Stage 2 evidence
root's `objects.rs` had been left inconsistent with the rest of that treatment;
this campaign binds the treatment uniformly and records the discrepancy in the
execution-identity audit rather than silently repairing it.

## 3. Execution identity is a gate, not an assumption

Before any measurement, each arm proves:

- resolved executable path and SHA256 recorded before and after its samples;
- build identity JSON (`LAYERFS_SOURCE_SEAL`, `LAYERFS_PRODUCT_SEAL`,
  `LAYERFS_COMPILATION_SEAL`, `LAYERFS_DEPENDENCY_SEAL`, `binary_sha256`,
  source commit/tree, workload sha256), image tag **and** immutable image ID;
- exact argv, cwd, relevant environment, and the live container id;
- fixture content and metadata identity, Store initialization receipt, and the
  selected edit paths/classes/offsets;
- **actual route dispatch**: a compiled probe counts real
  `ObjectBuffer::get_authenticated_canonical_batch` invocations. The presence or
  absence of an API declaration in a source tree is not evidence; only an
  observed call count is.

A recorded absolute clock value is **not** a gate. A drift is a failure only when
two runs of the same identified binary disagree; when a *different* binary is
shown to have produced an earlier number, the earlier number is corrected as
invalid identity evidence rather than chased.

## 4. Frozen measurement protocol

- Public operation: `Client::commit_workspace_session` through the campaign
  harness command
  `<qualified-binary> commit-baseline <fresh-store-dir> <fixture-payload> <live-container-id> namespace-100000 <cell>`.
- Fixture: immutable original pseudorandom `namespace-100000`, 100 000 files,
  1 000 data directories, 500 000 000 B, digest
  `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`, validated
  for full content/inventory plus file 0640, directory 0750 and mtime
  1 700 000 000 000 000 000 ns including the payload root, before collection.
- One fresh Store and one fresh container per cell. Fresh public Init + fork
  outside the Commit timer. No prepared output Store.
- Cache profile `commit-study-os-uncontrolled`. No cold claim, no untimed warm-up
  Commit.
- Cells: `nochange`, `retained` (K1 marker/repeat/revert/noedit), `k10`, `k100`,
  `fuse-posix`.
- **n = 3 pairs per comparison per cell**, order C1,T1; T2,C2; C3,T3, where the
  distinct pair identities are A/C (Q1) and B/C (Q2). Each comparison has its own
  sequence file and its own cells; a row is never reused as an independent sample
  of another comparison.
- `fuse-posix` is descriptive for Q1 and Q2 — the frozen non-inferiority
  allowance covers `nochange`/`retained`/`k10`/`k100`; its results are stated
  separately and never pooled.

## 5. Frozen gates

### Q1 — original optimization benefit (A vs C)

For `k100`, in **every** declared pair: `namespace_ns` ratio ≤ 0.75; public wall
strictly lower; user+system CPU strictly lower. Median `namespace_ns` reduction
is also reported. A failure here means the optimization's benefit is not
reproducible under these conditions and must be root-caused, not re-run.

### Q2 — hardening preservation (B vs C)

Per-pair non-inferiority for public wall and user+system CPU:
`C − B ≤ max(10 % of B in that pair, 1 ms)`.
**Parity passes.** No speedup over B is required or expected.

### Absolute targets (C)

| Cell | Metric | Target |
|---|---|---|
| `k100` | public Commit | ≤ 200 ms, **every** sample |
| `k100` | `namespace_ns` | ≤ 120 ms, **every** sample |
| `k10` | public Commit | ≤ 50 ms **median** |
| `k10` | `namespace_ns` | ≤ 31 ms **median** |

Every sample, range, median and breach is reported. Targets are not weakened
because a control drifts.

### Correctness, resources and custody

Per cell from raw receipts: `exit_code = 0`; container removed; proof `PASS`;
bootstrap identity `112 451 / 513 026 835 / 100 002`; K100 final
`112 684 / 513 774 250`; every Commit's declared result; complete changed bytes;
10 deterministic unchanged sampled files; visible head; clean end-of-session with
zero active workspaces/executions; phase equation per Commit; zero swaps; non-zero
container memory receipt; identical final root after reconnect; runtime spool 0 B;
no OOM. Cross-arm final canonical identity equal in every pair. Resource bounds
claim no more than what was sampled: charged tree scratch against the unchanged
4-MiB ledger, unchanged worker count, and pool/reader limits unchanged.

## 6. Invalidity and replacement rules

- Raw evidence is append-only; no attempt directory is deleted or overwritten.
- At most **one complete affected-pair replacement** per comparison, and only for
  demonstrated infrastructure invalidity (container, FUSE, daemon, host or fixture
  failure) recorded with its reason. Never for slow timing.
- All valid slow rows are retained and reported.
- A failed criterion is investigated and root-caused; it is never resolved by
  additional unchanged runs.
- Any material product or harness repair starts a versioned treatment with its own
  seal and its own declared checks, and invalidates earlier rows of that arm.
- Gates are not widened after seeing results. Higher n is chosen here, before
  outcomes, with the reason recorded: three pairs is the minimum that gives a
  median and a spread and keeps one full comparison group affordable; it was
  sufficient for the original Stage 2 claim and is the frozen rule inherited here.

## 7. Repairs this contract authorises

Focused, minimal repairs to product, harness or tests where a reproduced failure
requires them, each with a meaningful test and its own seal. Specifically in scope:

- the sparse pooled DELTA-chain exhaustion path: a bounded targeted corpus **or** a
  faithful configurable-limit test plus a documented production-limit proof, with
  the distinction between the two stated explicitly and no claim that a synthetic
  proof is end-to-end evidence;
- batch-specific retained-pressure and workspace-fallback exercises that use the
  real batch engine and the real fallback entry point, with deterministic read and
  attempt counts;
- allocation accounting using actual `size_of`/capacity values rather than assumed
  element sizes;
- any evidenced defect found while running the campaign.

Out of scope: new workers, cache/page-size increases, additional scratch
allowance, relaxed validation, Cold Init work, and the queued quadratic
spill/edit/history work.

## 8. Independent verification

The proof-only and verification-capable families are exercised through the real
supported entrypoints (`verify-selected.py`, family `verify.sh`, `--verification`
/ `--performance-rows` as the parser actually defines them), bound to the exact
image, source, input and row identities. An unavailable or unsupported flag is
reported, never invented. Any required verification that cannot run is listed as
remaining work, and **no aggregate PASS is claimed while a required verification
is NOT_RUN**.

## 9. Outputs

- `commit-stage2-terminal-results.md`: execution-identity audit, drift
  explanation, Q1 and Q2 outcomes with all raw rows and paired differences, CPU
  and charged bounds, fixes and tests, verification coverage, custody, exact final
  seals, and a terminal checklist in which JSON and Markdown verdicts agree.
- Issue #111 progress and final updates, cross-linked to
  #115/#108/#109/#110/#106/#102/#104/#100/#107.
- Only intended changes are committed; all unrelated dirty work is preserved.

## 10. Addendum: waiver of the K10 gate (owner decision)

**Added after the campaign ran and after its results were read.** This section
does not change any gate threshold, any measured value or any raw sample. It
records the disposition the terminal criteria did not cover.

The criteria above say a failed criterion must be investigated and root-caused and
must not be fixed by a rerun; they do not say what to do when the investigation
concludes and a residual miss remains. The K10 absolute medians came in at
54.90 ms against ≤ 50 ms and 33.75 ms against ≤ 31 ms, root-caused to a
knife-edge target frozen at 3.9 % and 0.06 % margin with a distribution-driven
spread.

**Owner decision, recorded as authored:** the K10 gate is **waived** for this
release and does **not** block Stage 2 closure. The owner has determined the K10
miss to be a minor, acceptable result, and Stage 2 is treated as meeting its
objectives with that waiver in place. Consequences:

- the K10 rows are recorded as **WAIVED**, not as `FAIL` and not as `PASS`. The
  measured values and the frozen thresholds stay printed next to the waiver;
- the waiver's authority is the owner, **not the measurement**. No gate threshold
  was relaxed, no valid sample was discarded, no rerun was used to move a median.
  This is therefore a disposition record, not a methodology change, and is
  **not** a precedent: the general rules in
  `docs/general/benchmark_rules.md` still govern every other campaign;
- Stage 2's measured performance claim remains the K100 result;
- the accepted residual risks named in the results (§9, §12) are carried forward
  explicitly and are **not** treated as resolved;
- **issue #111 is not closed by this stage.** #111 is the cold-cache
  `namespace-100000` **Init** gap (≤ 2.7 s), which remains open, paused and
  untouched. Stage 2 closure is a project-state disposition recorded in the
  results and on the issue timeline, not the closure of #111;
- Cold Init ≤ 2.7 s, quadratic spill-merge removal, edit-stage attribution,
  triangular publication and reopened-history scaling all remain outside this
  claim.

If a future campaign wants the K10 gate to read `PASS` on its own terms, that
requires a material repair or a pre-registered higher n — not a rerun, an outlier
removal, or a widened target. The waiver does not substitute for any of those.

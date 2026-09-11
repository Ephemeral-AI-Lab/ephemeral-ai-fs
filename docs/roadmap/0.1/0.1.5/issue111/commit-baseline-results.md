# Current-main public Commit baseline and phase attribution — #111

Measured current main (HEAD `3e22876be` + the preserved uncommitted
compaction-removal work, treatment **promoted-uncompacted**) after the compact
fingerprint index (`441be212e`). This is measurement and attribution only: no
optimization was implemented, no storage policy changed, the
guarded-predecessor candidate was not promoted, and the cold-Init <=2.7 s
objective remains open. Protocol:
[commit-baseline-contract.md](commit-baseline-contract.md) (commit `70be22868`).

Evidence root (all attempts, commands, logs, exits, patches, binaries, images,
fixture validation, raw receipts, Stores, analyzer, manifest):
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-baseline-evidence/20260911T142606Z`.

## What was measured

Plain cohort (n=3 independent fresh Stores per cell, product byte-identical to
current main: product seal `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`;
only the benchmark harness differs, source seal `490938083f8a7d7e…`, binary
`a61ac03724277c9d…`, image `layerfs-bench-infra:490938083f8a7d7e`). Diagnostic
cohort (n=2 per cell, product + metadata-index diagnostic counters, source seal
`9218655581b187ad…`, binary `8f4ce3070637607a…`, image
`layerfs-bench-infra:9218655581b187ad`); rows are nonce diagnostics, never
pooled with plain. Both arms share one byte-identical harness; the retained
main build reproduced exactly the recorded identities (source seal
`371d5dc40336492ed4aa969f4d210536bfbf740a8e0ac0f8a7c44e1ada1ac38b`, product
`760eb0f2…`, binary `0051058ac8e9ffca19fee65e595c19a43abc64ad315536aa14abc2f7e6983b63`).

All four registered fixtures were validated once per campaign (complete
inventory, file mode 0640 / directory mode 0750 / mtime 1700000000000000000 ns
including the payload root, full content SHA-256 against the prepared
manifest; namespace-100000 digest `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`).
Every cell bootstrapped through full public `Client::initialize_layerstack`
plus `fork_branch` outside the Commit timer, with the scan receipt and
bootstrap canonical/pool identity gated per tier (365/2,023/25,158/112,451
objects; 100,002 pooled values at 100,000 files). 45/45 plain and 30/30
diagnostic cells passed every gate on the final attempt; the analyzer rechecked
all gates from raw evidence (proof PASS, Created/UpToDate outcomes, phase
equations, candidate equations, canonical monotonicity, complete changed-file
bytes, 10 unchanged sampled files, final root/content after Store reopen,
clean workspace end, container cleanup, no swap/OOM).

The timer is only `Client::commit_workspace_session` (public wall). OS page
cache is **uncontrolled** for every row (`commit-study-os-uncontrolled`);
reopened means Store/index lifetime only. A first plain attempt (45 cells,
harness seal `07985007…`) is retained in full under `plain-invalid-attempt-1/`:
its fuse-posix expectation was wrong (harness bug, product content proven
correct by probe); the whole cohort was rerun once under a single new seal —
no timing from that attempt is used.

## Plain baseline: public Commit wall (milliseconds, median [min,max], n=3)

| Files | No change (`UpToDate`) | Retained first change | Reopened first change |
|---:|---:|---:|---:|
| 100 | 1.52 [1.29, 1.63] | 5.73 [5.63, 6.22] | 6.46 [6.29, 7.04] |
| 1,000 | 1.75 [1.28, 1.80] | 9.46 [9.40, 9.66] | 9.26 [9.23, 10.13] |
| 10,000 | 1.99 [1.71, 2.06] | 15.02 [12.40, 15.43] | 25.08 [24.10, 25.25] |
| 100,000 | 2.12 [1.93, 3.01] | 18.84 [15.11, 21.32] | 142.64 [140.49, 146.59] |

Repeated change/reversion in the same workspace (retained and reopened chains,
Commit #1 marker / #2 different marker / #3 revert / #4 no edit, all `Created`
except #4 `UpToDate`):

| Files | Retained 1/2/3/4 | Reopened 1/2/3/4 |
|---:|---|---|
| 100 | 5.73 / 4.08 / 3.57 / 1.58 | 6.46 / 4.28 / 4.14 / 1.52 |
| 1,000 | 9.46 / 4.58 / 4.50 / 1.56 | 9.26 / 4.62 / 4.20 / 1.42 |
| 10,000 | 15.02 / 7.11 / 6.62 / 2.08 | 25.08 / 7.76 / 6.54 / 1.79 |
| 100,000 | 18.84 / 6.89 / 6.67 / 2.71 | 142.64 / 8.89 / 8.02 / 2.65 |

Edit preparation (public SDK `edit_workspace_file_range`, 10 bytes at the
registered offset of `d0000/f000000`), outside the Commit timer:

| Files | Edit 1 | Edit 2 | Edit 3 (revert) |
|---:|---:|---:|---:|
| 100 | 5.16 | 2.70 | 2.54 |
| 1,000 | 4.56 | 2.63 | 2.96 |
| 10,000 | 16.38 | 3.96 | 4.61 |
| 100,000 | 20.93 | 4.38 | 4.39 |

Setup context (medians): bootstrap Init 0.024 / 0.085 / 1.000 / 2.589 s across
tiers (OS-cache uncontrolled — not comparable with cold-gate numbers);
branch fork 0.10–0.22 ms; workspace create 9.9–13.3 ms; Store reconnect (drop
every owner, `LayerStackStore::connect`, new Client) 3.60 / 7.07 / 191.97 /
336.64 ms across tiers.

## Changed-set scaling at 100,000 files (retained lifetime, first change)

| Changed set | Edit stage | Commit #1 | Candidate objects/bytes | Admission values admitted |
|---:|---:|---:|---:|---:|
| K=1 | 20.9 ms | 18.84 [15.11, 21.32] | 5 obj / 10 kB | 1 |
| K=10 (10 directories) | 104.7 ms | 80.50 [79.81, 85.30] | 32 obj / 114 kB | 10 |
| K=100 (100 directories) | 659.1 ms | 313.29 [310.18, 315.09] | 233 obj / 747 kB | 100 |

K files were spread evenly over the namespace (ordinal `floor(j·N/K)`, empty
files skipped deterministically); each sits in a different data directory.
Edit-stage per-call walls: first call ~20–22 ms, subsequent ~6 ms
(K=100 total 656–661 ms).

## Phase partition (plain Commit #1 medians, ms; disjoint top-level phases + residual)

| Cell | pause_fence | quiesce | capture | cand_plan | content | namespace | cand_finish | obj_admission | publication | checkpoint | resume | unattributed | wall |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 100/nochange | 0.84 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.01 | 0.06 | 0.01 | 0.21 | 0.36 | 1.52 |
| 100/retained | 1.07 | 0.00 | 0.00 | 0.00 | 0.60 | 0.47 | 0.05 | 0.79 | 0.10 | 2.10 | 0.26 | 0.36 | 5.73 |
| 100/reopened | 1.24 | 0.00 | 0.00 | 0.00 | 0.75 | 0.47 | 0.04 | 1.05 | 0.10 | 2.19 | 0.29 | 0.39 | 6.46 |
| 1,000/nochange | 0.88 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.06 | 0.01 | 0.28 | 0.41 | 1.75 |
| 1,000/retained | 1.15 | 0.00 | 0.01 | 0.00 | 0.59 | 2.49 | 0.07 | 1.82 | 0.11 | 2.45 | 0.31 | 0.38 | 9.46 |
| 1,000/reopened | 1.07 | 0.00 | 0.01 | 0.00 | 0.66 | 2.30 | 0.05 | 2.11 | 0.10 | 2.38 | 0.27 | 0.36 | 9.26 |
| 10,000/nochange | 1.05 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.01 | 0.06 | 0.01 | 0.31 | 0.44 | 1.99 |
| 10,000/retained | 1.27 | 0.00 | 0.01 | 0.00 | 0.60 | 5.19 | 0.07 | 3.76 | 0.11 | 3.42 | 0.37 | 0.41 | 15.02 |
| 10,000/reopened | 1.78 | 0.00 | 0.01 | 0.00 | 0.75 | 5.18 | 0.06 | 12.00 | 0.11 | 4.16 | 0.34 | 0.45 | 25.08 |
| 100,000/nochange | 1.31 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.01 | 0.07 | 0.01 | 0.34 | 0.44 | 2.12 |
| 100,000/retained | 1.39 | 0.00 | 0.00 | 0.00 | 0.66 | 5.97 | 0.09 | 4.87 | 0.11 | 2.81 | 0.32 | 0.42 | 18.84 |
| 100,000/reopened | 2.23 | 0.00 | 0.00 | 0.00 | 0.62 | 5.96 | 0.11 | 128.79 | 0.10 | 3.90 | 0.53 | 0.66 | 142.64 |
| 100,000/k10 | 1.22 | 0.00 | 0.01 | 0.00 | 0.79 | 62.86 | 0.24 | 11.96 | 0.11 | 3.26 | 0.33 | 0.40 | 80.50 |
| 100,000/k100 | 1.61 | 0.00 | 0.00 | 0.01 | 15.32 | 241.33 | 1.39 | 42.53 | 0.12 | 7.40 | 0.70 | 0.49 | 313.29 |
| 100,000/fuse-posix | 1.88 | 0.00 | 0.00 | 0.00 | 0.89 | 6.12 | 0.09 | 6.95 | 0.11 | 3.32 | 0.33 | 0.54 | 22.23 |

`dirty_compare_ns` and `local_admission_ns` are zero in every row: the schema
fields exist but no current-main path clocks them; their work is inside the
residual/other phases. Phases + residual sum exactly to the receipt total
(analyzer-gated); the SDK outer wrapper (receipt total vs public wall) is
0.01–0.03 ms — the public call is the product operation.

### Nested details (never added into the enclosing phases)

Object-admission nested SQL clocks are tiny at K=1 (retained 100,000 files:
begin 0.003 ms, insert 0.132 ms, commit 0.056 ms, authentication 0.000 ms,
storage-authentication 0.000 ms, sort 0.000 ms; publication begin 0.001 ms,
payload 0.000 ms, insert 0.000 ms, metadata 0.057 ms, commit 0.011 ms;
reopened similar: insert 0.169 ms, commit 0.069 ms). At
K=100 they grow to insert 1.332 ms / commit 1.061 ms inside a 42.53 ms
admission phase. The dominating nested cost is the metadata-pool ValueIndex
work (below). Checkpoint nests spool retirement (all rows ~0). Diagnostics:
K=1 commits touch 1 dirty node / 1 candidate probe / 3 pieces; K=10: 10/10/30;
K=100: 100/100/300 — namespace visits track the changed set, not the namespace.

Metadata-index events (diagnostic cohort, n=2, sums across both runs):

| Cell | Commit | Index creates | Syncs | Replayed values | Lookup inputs / candidates / authenticated groups / hits | Sync wall (median) |
|---|---|---:|---:|---:|---|---:|
| 100,000/reopened | #1 | 2 | 2 | 200,004 (2×100,002) | 100 / 98 / 2 / 98 | 123.0 ms |
| 100,000/reopened | #2 | 0 | 2 | 2 | 100 / 98 / 1 / 98 | ~0.08 ms |
| 100,000/retained | #1 | 0 | 2 | 6,508 (2×3,254 Init tail) | 100 / 98 / 3 / 98 | 5.6 ms |
| 10,000/reopened | #1 | 2 | 2 | 20,004 | 100 / 98 / 2 / 98 | 10.9 ms |
| 100,000/k100 | #1 | 0 | 4 | 6,618 | 9,914 / 9,714 / 260 / 9,714 | 5.7 ms |
| 100,000/nochange | #1 | 0 | 0 | 0 | — | 0 |

Every marker-change Commit admitted exactly one new pooled metadata value per
changed file (K=100: 100; reverted Commit #3 admits none — its metadata value
equals the original already-pooled value); the no-change commit performs no
metadata preparation at all. No eviction occurred (100,002 ≤ 131,072 retained
window). Counter snapshots confirm the work is at Commit, not hidden in
setup: pool counters move only across edit/commit boundaries; workspace
creation adds ~2 pool group fetches.

## SDK edit versus ordinary POSIX/FUSE write (100,000 files)

| Route | Mutation interval | Commit #1 wall | Commit #1 admission / sync | Result |
|---|---:|---:|---|---|
| SDK range edit (`I000000001` @ 961) | 20.9 ms (median edit 1) | 18.84 [15.11, 21.32] | 4.87 ms / 3.88 ms | `Created` |
| FUSE write, registered `namespace-edit` workflow (`E000000001` @ 961, normalized mtime) | 27.9 / 25.7 / 25.9 ms (exec incl. process spawn) | 22.23 [18.67, 23.74] | 6.95 ms / 5.88 ms | `Created` |
| FUSE write #2, workload `edit` (`E000000002` @ 464) | 4.8–7.7 ms | 7.03 ms median | — | `Created` |

Both routes converge on the same Commit shape: capture is a no-op for the
Docker FUSE projection (0 files, 0 bytes — mutations are live-backed), the
same phase partition applies, and the FUSE-route commit sits within the
retained SDK range (its slightly higher admission/sync reflects the
normalized-mtime write path and its first-write-after-exec state, visible as
spool_alloc 10 bytes in diagnostics). The SDK route performs real eager work
at edit time (first edit 20.9 ms vs 4.4 ms warm; K=100 edit stage 659 ms
exceeds its 313 ms Commit) — that work is outside the Commit timer and must
not be described as free. A POSIX/FUSE write followed by the same public
Commit therefore pays a comparable Commit cost plus a different (process-spawn
and write-path) mutation cost.

## Resources and physical receipts

Commit #1 at 100,000 files (medians): retained user 9.3 ms / system 3.3 ms
CPU, 0 B process disk reads, ~0.01 MB writes, RSS 84 MiB (post-call snapshot);
reopened user 126.2 ms / system 9.8 ms (CPU-bound replay), RSS 69 MiB. No
swap/OOM in any row; container peak memory 5 MiB (20 MiB for K=100); container
limits 2 CPUs / 2 GiB / 256 PIDs. Post-cell Stores: 112,461 canonical objects
after the K=1 chains (112,483 K=10, 112,684 K=100), apparent ~515.4–515.6 MB,
allocated 518–527 MB; runtime spool cleaned to 0 at Clean end. Full per-stage
user/system CPU, disk reads/writes, RSS/peak-RSS/footprint, context-switch,
thread and cgroup receipts are in each cell's `result.json`/`output.log`.

## Answers to the owner questions

1. **Public main Commit cost**: no-change `UpToDate` ~1.5–2.1 ms at every
   namespace size (generation-zero fast path); first one-file changed Commit
   5.7 / 9.5 / 15.0 / 18.8 ms at 100 / 1k / 10k / 100k files (retained);
   repeated changes ~4–9 ms and revert ~4–8 ms; larger changed sets 80.5 ms
   (K=10) and 313.3 ms (K=100) at 100k. Setup context above.
2. **Retained vs reopened**: identical except the first changed Commit after
   reopen: 142.64 vs 18.84 ms at 100k (25.08 vs 15.02 at 10k; ~equal at
   100/1k). The entire delta is ValueIndex reconstruction inside object
   admission (sync 127.15 vs 3.88 ms nested; replayed 100,002 values). So yes
   — index reconstruction still dominates the reopened first Commit (89% of
   it) after the fingerprint-index change, at roughly half its pre-change cost
   (~243 ms → ~123 ms replay; ~1.2 µs/value), and it is CPU-bound, not
   I/O-bound (0 B process disk reads).
3. **Where time goes**: for changed Commits the largest top-level phase is
   namespace/tree construction (6.0 ms of 18.8 at 100k K=1; 62.9/80.5 K=10;
   241.3/313.3 = 77% K=100), then object admission (4.9 ms K=1; 42.5 ms K=100,
   of which metadata-index sync 2–6 ms retained), then fixed overheads:
   pause fence 1–2 ms, checkpoint 2.8–7.4 ms, resume ~0.3 ms, publication
   ~0.1 ms, residual ~0.4–0.7 ms. Discovery/reconciliation of dirty state is
   not a measured cost on this path: SDK and FUSE mutations are live-backed,
   capture is a no-op, and the candidate is built from recorded mutations
   (candidate_plan ~0; content 0.7–15.3 ms only with changed data).
4. **Scaling**: with total namespace cardinality — the one-file namespace
   phase (0.47 → 5.97 ms from 100 to 100k files), checkpoint, and the
   out-of-Commit Store reconnect (3.6 → 336.6 ms). With the changed portion —
   namespace phase per changed directory (~6 ms each at 100k), content,
   admission, candidate objects/bytes, admitted pool values. With total
   physical history, independent of the changed set — the reopened first
   Commit's index replay (10.9 ms at 10k / 123 ms at 100k for 20k / 100k
   values). Flat — no-change Commit, pause/resume/publication.
5. **SDK edit does eager work**: the first SDK edit costs 20.9 ms at 100k
   (4.4 ms warm) and the K=100 edit stage 659 ms — more than its Commit. At
   Commit time both mutation routes take the same discovery/reconciliation
   branch (live dirty state; capture no-op), so an ordinary POSIX/FUSE write
   followed by the same public Commit pays a comparable Commit wall plus its
   own write-path cost; the two routes are labeled separately and neither
   timing covers the other.
6. **Single bottleneck**: for the changed-set Commit path the measured
   dominant cost is the namespace/tree construction phase scaling with the
   changed portion and amplified by namespace size; for the reopened first
   Commit it remains index reconstruction (a known, separately designed cost —
   see recommendation).

## Ranked measured costs (100,000 files) and recommendation

| Rank | Measured cost | Magnitude (median) | Share |
|---|---|---:|---:|
| 1 | Reopened first-Commit ValueIndex reconstruction (history-proportional replay of 100,002 values) | 127.2 ms of 142.6 ms | 89% of that Commit |
| 2 | Namespace/tree construction for changed sets (K=100) | 241.3 ms of 313.3 ms | 77% |
| 3 | Namespace/tree construction for changed sets (K=10) | 62.9 ms of 80.5 ms | 78% |
| 4 | Object admission incl. metadata pool (K=100) | 42.5 ms of 313.3 ms | 14% |
| 5 | Fixed per-Commit overheads (fence+checkpoint+resume+publication+residual) | ~4–10 ms | 21–53% at K=1 |
| 6 | SDK edit eager preparation (outside Commit) | 20.9 ms first / 659 ms K=100 | not in Commit timer |

**Recommended next experiment (one): changed-leaf namespace/tree construction
in `Workspace::commit`.** The K-scaling axis is the only top-2 cost that
affects every ordinary changed-set Commit on a retained Store, and it is a new
measured finding of this study; the reopened replay cost already has a
designed candidate with prior screen evidence (not promoted, with open
retained-mode/storage-screen questions), and restarting it is out of scope
here. Hypothesis: each changed file in its own directory triggers a separate
frontier-directory update and tree re-encode (~6 ms per changed leaf at 100k);
batching or amortizing these per-candidate (e.g. sharing unchanged-subtree
encodings across the dirty leaves of one candidate, or coalescing
`apply_frontier_directory` work) can cut the namespace phase substantially.
Expected ceiling from these measurements: ~55 ms of 80.5 ms at K=10 and
~230 ms of 313.3 ms at K=100 (the K=1-equivalent fixed portion remains).
Correctness risks: leaf split/merge, namespace refcounts, identical canonical
roots (physical placement may legitimately differ — must be declared and
measured), DELTA/pack reuse, and unchanged-file byte preservation.
Falsifiable test: freeze control/candidate arms under the runner lock, rerun
this study's retained K=1/10/100 cells plus the no-change and revert cells,
and require the Commit wall reduction to appear specifically in
`namespace_ns` with unchanged canonical object counts, admitted pool values,
complete changed-file bytes, sampled unchanged files and reopen proofs —
reusing this campaign's harness, collector and analyzer without modification.
If the evidence does not support it, the honest alternative is no
optimization: the retained one-file Commit (18.8 ms) is already dominated by
fixed overheads and small phases with no single dominant target.

## Seals, custody and remaining work

Contract `70be22868`; evidence root `20260911T142606Z` with `evidence-manifest.json`
hashing all 21,767 retained files; source seals verified unchanged after
collection; focused store suites passed in both worktrees (134 tests, 4
preexisting ignored, each arm). SQLite/SDK/admission/publication/spool ran on
the macOS host; Docker supplied only daemon/FUSE/workload (2 CPUs / 2 GiB).
The original uncommitted compaction-removal work is preserved byte-identical
in the main workspace and excluded from all commits made by this campaign.
No release, tag or deployment. #111 remains open with its original cold-Init
objective; related context: #115 (shared admission/publication), #108
(branch-history tier-500), prior evidence under
[index-attribution](index-attribution-results.md),
[metadata-proof](metadata-proof-experiment-results.md),
[fingerprint-index](fingerprint-index-results.md) and
[final-tree-rca](final-tree-rca-results.md).

Issue update: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5637126575

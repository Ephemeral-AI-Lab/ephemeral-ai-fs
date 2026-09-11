# Commit Stage 1: inner namespace and edit-stage attribution — #111

Measured current main (Stage 1 HEAD `e5ea7ffca` plus the preserved uncommitted
compaction-removal work, treatment **promoted-uncompacted**) against the bounded
Stage 1 matrix frozen in [commit-stage1-contract.md](commit-stage1-contract.md)
(commit `7f264300e`). This is measurement and attribution only: no optimization
was implemented or promoted, no storage policy changed, and the original
cold-Init <=2.7 s objective remains **open**. Further Init optimization stays
paused; last valid recorded cold Init median ~3.420 s.

Evidence root (all attempts, commands, logs, exits, patches, binaries, images,
fixture validation, raw receipts, Stores, collector, analyzer, manifest):
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage1-evidence/20260911T163220Z`.

## 1. Question and what was measured

Stage 1 asks what actually consumes the **241.33 ms `namespace_ns` of a 313.29 ms
K=100 Commit** and the **659.08 ms SDK edit stage** preceding it at
N=100,000 files, without moving work between timers. Frozen matrix: the original
pseudorandom `namespace-100000` fixture only, cells `nochange`, `retained`
(K1 marker/repeat/revert/no-edit chain), `k10`, `k100`, `fuse-posix`; plain n=3
then separate diagnostic n=2, each an independent fresh Store and container.
Plain and diagnostic rows are never pooled and instrumentation overhead is never
subtracted.

The timer remains only public `Client::commit_workspace_session`. The secondary
metric is the per-sample sum of all edit preparation plus that Commit, computed
per sample before aggregation. OS page cache is **uncontrolled** for every row
(`commit-study-os-uncontrolled`); "retained" is a Store/index lifetime label,
never a cold-cache claim. No untimed warm-up Commit was added.

## 2. Identities, drift and custody

| Identity | Stage 1 plain | Stage 1 diagnostic | Completed baseline |
|---|---|---|---|
| Product seal | `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932` | `760eb0f2…` + Stage 1 diagnostics | `760eb0f2…` |
| Source seal | `490938083f8a7d7ecec166ae2e20c7287d0f7c1e890c504fd5c5c05217be3a8e` | see `diagnostic-identity.json` | plain `490938083f8a7d7e…` |
| Harness source | `commit_baseline.rs` sha256 `9128dfe2ca27e3972679b4093f686a25ab319100f1563fde17f872d17951ffdb` | identical | identical |
| Host binary | `0141eb60…` (fresh independent build) | see `diagnostic-identity.json` | `a61ac037…` |
| Linux image | `layerfs-bench-infra:490938083f8a7d7e` (`sha256:edb29e96…`) | `layerfs-bench-infra:<diagnostic seal>` | `sha256:48922c21…` (tag reused) |
| Workload source | `c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b` | identical | identical |

**Drift check before freezing sources.** The current dirty worktree *without* the
campaign harness hashes to source seal
`371d5dc40336492ed4aa969f4d210536bfbf740a8e0ac0f8a7c44e1ada1ac38b` and product
seal `760eb0f2…` — exactly the retained-main build recorded by the completed
baseline — so the Stage 1 treatment is byte-identical to the measured baseline
treatment. The only commits since the baseline evidence are documentation
(`faaa03933`, `e5ea7ffca`), and `docs/` is outside the sealed input set. Both
isolated worktrees reproduce SOURCE_SEAL `490938083f8a7d7e…` and the identical
harness bytes from a clean checkout of `e5ea7ffca` plus the recorded dirty patch,
so no historical patch was applied over later main changes. The host binary and
Linux image differ from the baseline only by non-deterministic build identity;
the product seal — the behavioral binding — is unchanged.

Preexisting work preserved byte-identically: the tracked compaction-removal diff
(`custody/before-tracked-binary.patch`, sha256
`44fdf679f24deead8e5f0d7dc8b2d0dd3a06346d56d8477058ebc69db0f6c2cc`) and the
untracked `compaction-removal.md` / `issue112/` / `issue113/` files (hashes in
`custody/before-untracked-hashes.txt`). Nothing was stashed, cleaned, reverted,
attributed or committed for another owner.

## 3. Plain cohort — frozen matrix, n=3 (milliseconds, median [min,max])

| Cell | Commit #1 wall | Commit #1 `namespace_ns` | Edit stage | Edit + Commit #1 (per-sample sum) |
|---|---:|---:|---:|---:|
| `nochange` | 2.26 [1.84, 2.73] | 0.00 | — | — |
| `retained` (K=1) | 20.47 [16.74, 26.28] | 5.87 [5.72, 5.94] | 31.14–35.79 (3 edits) | 47.88–59.24 |
| `k10` | 82.33 [77.27, 84.46] | 59.87 [59.14, 60.19] | 115.76–121.63 | 197.71–206.09 |
| `k100` | 329.76 [316.94, 336.14] | 254.09 [244.05, 257.04] | 650.46–749.56 | 986.61–1066.50 |
| `fuse-posix` | 24.22 [21.65, 26.17] | 6.15 [6.12, 6.46] | 34.27–39.88 (2 exec writes) | 56.83–66.05 |

The frozen matrix reproduces the completed baseline shape: K=1/10/100 public
Commit 20.5 / 82.3 / 329.8 ms against the recorded 18.84 / 80.50 / 313.29 ms,
with namespace 5.87 / 59.87 / 254.09 against 5.97 / 62.86 / 241.33. The uniform
~3–7 % upward shift is consistent with a different uncontrolled-cache session and
is not attributed to any code change; no baseline row is used as a Stage 1
measurement.

## 4. Diagnostic cohort — overhead, inner namespace, edit stage

Diagnostic arm (n=2, independent fresh Stores), product plus Stage 1 diagnostics
only, source seal `6ca0b4576927d974…`, product seal `45488fd2eb6081a1…`, binary
`55ae588e…`, image `layerfs-bench-infra:6ca0b4576927d974`
(`sha256:a8fa450ba0031816…`). All 10/10 cells passed every gate.

### 4.1 Instrumentation overhead (diagnostic minus plain, medians)

| Cell | Commit #1 wall | `namespace_ns` |
|---|---:|---:|
| `nochange` | 2.26 → 2.77 ms (+0.51) | 0.00 → 0.00 |
| `retained` | 20.47 → 22.81 ms (+2.34) | 5.87 → 6.15 ms (+0.28) |
| `k10` | 82.33 → 84.55 ms (+2.22) | 59.87 → 62.98 ms (+3.11) |
| `k100` | 329.76 → 316.25 ms (−13.51) | 254.09 → 245.47 ms (−8.62) |
| `fuse-posix` | 24.22 → 20.65 ms (−3.57) | 6.15 → 6.51 ms (+0.36) |

Overhead is not resolvable from session noise at n=2/3 (the K=100 diagnostic
median is *lower* than plain). Every counter below is therefore read as a
structural decomposition of the phase, and no diagnostic millisecond is
subtracted from or added to a plain row. Plain rows remain the reported
performance numbers.

### 4.2 Inner namespace partition (diagnostic medians, ns; disjoint, never added to their parents)

| Inner clock / counter | K=1 (first) | K=1 (repeat) | K=10 | K=100 |
|---|---:|---:|---:|---:|
| *n=2 medians: the exact mean of the two samples, half-nanoseconds truncated* | | | | |
| `namespace_ns` (unchanged public phase) | 6,115,625 | 1,515,896 | 62,922,770 | **245,420,334** |
| reference handling (`apply_references`) | 417 | 1,104 | 438 | 542 |
| checkpoint journal creation | 180,271 | 127,958 | 157,312 | 202,396 |
| `inodes.finish` total | 5,755,063 | 1,198,313 | 62,274,271 | 244,725,458 |
| — spill merge | 0 | 0 | 0 | 0 |
| — record encoding | 36,042 | 42,271 | 64,271 | 549,666 |
| — namespace root read | 1,729 | 4,188 | 2,646 | 1,084 |
| — **batched tree application** | 5,713,688 | 1,147,583 | **62,202,084** | **244,171,312** |
| — fallback | 0 | 0 | 0 | 0 |
| — final root encoding | 0 | 0 | 0 | 0 |
| — residual | 542 | 1,709 | 542 | 562 |
| checkpoint record validation | 30,042 | 25,333 | 50,394 | 445,059 |
| namespace-phase residual | 179,874 | 188,520 | 490,750 | 491,938 |

Inside the batched tree application (nested in the row above):

| Counter / nested clock | K=1 (first) | K=1 (repeat) | K=10 | K=100 |
|---|---:|---:|---:|---:|
| **`read_auth_ns`** (canonical fetch + authentication + decode) | 5,536,250 | 1,054,746 | **61,516,122** | **240,334,812** |
| — `decode_ns` (compact leaf decode, nested in the above) | 994,686 | 980,192 | 9,626,306 | 30,083,552 |
| nodes read | 97 | 97 | 673 | **2,053** |
| nodes read then skipped as unchanged-range | 93 | 93 | 651 | **1,920** |
| pages actually entered (`in_range`) | 3 | 3 | 21 | 132 |
| **leaf records decoded** | 3,200 | 3,200 | 32,000 | **101,001** |
| synthetic inode-value IDs derived (encode + hash) | 3,200 | 3,200 | 32,000 | **101,001** |
| authenticated canonical bytes read | 345,464 | 345,464 | 2,703,608 | **8,353,409** |
| `leaf_value` calls (leaf entries merged) | 50 | 50 | 500 | 5,000 |
| changed keys (`delta_keys`) | 1 | 1 | 10 | 100 |
| pages created / reused | 3 / 0 | 3 / 0 | 21 / 0 | 132 / 0 |
| tree encode (`F::encode`) | 5,916 | 5,562 | 47,460 | 278,686 |
| tree encoded bytes | 7,982 | 7,982 | 68,264 | 492,848 |
| scratch peak (bytes, 4 MiB budget) | 82,768 | 82,768 | 117,596 | 164,556 |
| delta count / spilled / fallback taken | 1 / 0 / 0 | 1 / 0 / 0 | 10 / 0 / 0 | 100 / 0 / 0 |
| dirty **directory** nodes | 0 | 0 | 0 | 0 |

Independent cross-check from the product `PhysicalStorageReceipt` delta of the
same Commits: K=100 reads 5,990 storage groups, 31.1 MB encoded / 87.9 MB
decoded, **5,990 decompression calls**, 17,970 blob ranges. The 8.35 MB the tree
engine authenticates is a subset of that; every canonical object read is
decompressed, which is where the per-object read cost comes from.

### 4.3 Edit stage (diagnostic, K=100)

| Quantity | Rep 1 | Rep 2 |
|---|---:|---:|
| SDK edit stage (host, 100 public calls) | 647.6 ms | 701.7 ms |
| per-edit wall: first / median / min / max | 21.1 / 5.87 / 4.13 / 21.1 ms | 22.9 / 6.22 / 4.43 / 22.9 ms |
| daemon `freeze()` barriers | 102 | 102 |
| — Σ `freeze` wall | 92.4 ms (14.3 %) | 103.5 ms (14.7 %) |
| — Σ `publish_facts` wall | 92.1 ms | 103.2 ms |
| — Σ synchronous backing round trips (`publish_call_ns`), 304 calls | 88.3 ms | 99.6 ms |
| — Σ clone + re-encode of the published set | 3.75 ms | 3.52 ms |
| published rows, cumulative | **5,050** (= 1+2+…+100) | **5,050** |
| published pages / bytes | 100 / 1,363,600 | 100 / 1,363,600 |
| host fact groups received / rows / bytes | 102 / 5,050 / 1,343,300 | 102 / 5,050 / 1,343,300 |
| host consume wall (all 102 groups) | 5.9 ms | 6.7 ms |
| kernel-cache flushes / invalidations | 0 / 0 | 0 / 0 |
| retirement scans / released ranges | 0 / 0 | 0 / 0 |
| range × folio tests / overlaps / kernel-edit hits | 0 / 0 / 0 | 0 / 0 / 0 |
| unattributed edit-stage remainder | ≈ 555 ms (86 %) | ≈ 598 ms (85 %) |

Interpretation. The published set is *not* the whole dirty set: `publish_rows`
per barrier is exactly the number of edited files so far (1, 2, … , 100), not
files plus their directories, and `dirty_directories` is 0 for every measured
Commit. The cumulative row traffic over one K=100 edit sequence is therefore
still triangular (5,050 rows) exactly as the earlier audit predicted — but its
measured cost is 3.7 ms of re-encode plus 88–100 ms of 304 synchronous backing
round trips, and the host consumes all 5,050 rows in 5.9–6.7 ms. Removing the
redundant intermediate publication is a real, bounded saving of roughly 14 % of
the edit stage, **not** the dominant term. Kernel-cache flushing, retirement
scans and range × folio work are compiled out or inactive in this topology, so
they cannot be blamed for the 659 ms either. About 86 % of the edit stage
remains unattributed by Stage 1 instrumentation.

## 5. What actually consumes the two phases

### 5.1 Actual path (measured stages only)

```text
public Client::commit_workspace_session                       (timer)
└── Workspace::commit
    ├── pause fence / quiesce / capture                       ~1–2 ms, capture no-op
    ├── build_candidate
    │   ├── candidate plan + content (streaming file admission) 0.5–17 ms with changed data
    │   └── namespace phase  = apply_references + frontier finish
    │       ├── apply_references                              < 1 µs
    │       ├── CheckpointJournal::new + validate + push       ~0.45 ms (K=100, 100 records)
    │       ├── record encode + put_owned                      ~0.55 ms (100 records, 493 KB)
    │       └── inode_table_apply_sorted_with_budget          244.2 ms  ← 99.5 % of the phase
    │           └── Engine::read  x 2,053                    240.3 ms  ← 98.4 % of the tree apply
    │               ├── with_authenticated_canonical(fetch + hash + decompress) ~210 ms
    │               └── CompactInodes::decode (+ inode_value_id per record)     30.1 ms
    │                   └── 101,001 records decoded and 101,001 IDs derived for 100 changed keys
    │                       (1,920 of 2,053 reads are entered by no delta and skipped immediately)
    ├── object admission / publication                        ~46 ms (K=100)
    └── checkpoint install / resume                           ~8 ms
```

```text
public Client::edit_workspace_file_range   (each of K calls, outside the timer)
└── daemon EDIT_BEGIN → freeze()
    ├── gate/flush_append                                     ~0.1–0.7 ms
    ├── publish_facts: clone + encode + serialize             3.7 ms total over 100 barriers
    ├── 2–3 synchronous backing round trips per barrier       88–100 ms total (304 calls)
    └── retire_ranges                                         0 (no backing ranges)
    → host live_backing consumes 5,050 rows in 5.9–6.7 ms
≈ 86 % of the 648–702 ms stage is host SDK/coordinator plus daemon edit application,
  which Stage 1 did not instrument; no edit-stage target can be set from this evidence.
```

### 5.2 The corrected hypothesis

The preceding report's hypothesis — per-changed-directory frontier updates and
tree re-encodes — is **rejected by measurement**. `dirty_directories` is 0 in
every Stage 1 Commit, all deltas enter one sorted batch, tree encoding totals
0.28 ms, and page creation is exactly one page per entered changed page
(`nodes_created` = 132 at K=100, `tree_encoded_bytes` = 493 KB). The namespace
phase cost is not in mutation at all.

The measured explanation for the 241–254 ms namespace phase is a **read-side
amplification in the sorted-batch engine**:

1. `Engine::edit` reads every child of an entered page *before* testing whether
   the delta stream enters that child (`batch.rs:579` before `:519`). At K=100
   the engine reads 2,053 pages and enters only 132 of them: **1,920 reads
   (93.5 %) are authenticated, decompressed and decoded only to be discarded as
   an unchanged sibling**.
2. Each discarded page is still fully decoded, and `CompactInodes::decode`
   derives a synthetic canonical inode-value identity (encode + hash) for every
   leaf record (`batch.rs:946-955`). That is **101,001 record decodes and
   101,001 hash derivations for 100 real changes** — 30.1 ms, a deterministic
   0.31 µs per record.
3. The remaining ~210 ms is canonical fetch + authentication + decompression of
   the 8.35 MB those pages occupy inside the 240.3 ms `read_auth_ns`.

Reads scale with the number of *entered parent pages*, and each entered parent
pulls in its whole ~96-child fan-out: 97 reads at K=1 (one entered page), 673 at
K=10 (~7), 2,053 at K=100 (~21). That is linear in the changed set with a large
constant, plus a full-namespace term once the changed set spreads across every
parent. It is not quadratic in K at this N; it is not mutation; and it is 99 %
of the phase.

For the **659 ms edit stage** the honest answer is partial: the confirmed
triangular fact-publication traffic costs 92–104 ms (14–16 %), the flagged
kernel-cache flush and retirement scans are inactive, and **~86 % of the stage is
not yet attributed**. Stage 1 does not claim the audit's "redundant intermediate
snapshot" is the edit hotspot; the measurement contradicts that.

Measured versus unmeasured, explicitly:

| Phase | Measured | Unmeasured remainder |
|---|---:|---:|
| `namespace_ns` K=100 (245.4 ms) | 99.8 % (references, finish, checkpoint, residual) | 0.2 % |
| — of which tree apply (244.2 ms) | 98.4 % attributed to page reads | 1.6 % |
| — of which reads (240.3 ms) | 12.5 % decode, 87.5 % fetch/auth/decompress | — |
| SDK edit stage K=100 (648 ms) | 14.3 % (barrier publication) | **85.7 %** |
| public Commit K=100 (316 ms) | namespace 77.6 %, admission ~14 %, rest ~8 % | within attributed phases |

## 6. Ranked measured removable costs and the one Stage 2 experiment

| Rank | Measured cost at K=100 (N=100,000) | Magnitude | Removable by |
|---|---|---:|---|
| 1 | Sibling pages read for deltas that never enter them (1,920 of 2,053 reads) | ≈ 196 ms of `read_auth_ns` + 28 ms of decode ≈ **224 ms of the 245.4 ms namespace** | not reading them (Stage 2, conditional) |
| 2 | Per-record synthetic ID derivation on every decoded leaf record | 30.1 ms (101,001 records) | decoding only modified pages |
| 3 | Redundant fact publication on every edit barrier | 92–104 ms of 648–702 ms (14–16 %); 5,050 cumulative rows | one publication per Commit instead of per barrier (Stage 3, conditional on the consumer boundary) |
| 4 | Object admission + metadata pool at K=100 | ~46 ms of 316 ms | separate design work, not a Stage 1 result |
| 5 | Kernel-cache flush / retirement / range × folio | 0 ms measured in this topology | not a lever here |
| 6 | Spill merge, batch fallback, record encode, checkpoint | 0 / 0 / 0.55 / 0.45 ms at K=100 | not levers at this K |

**ONE recommended Stage 2 experiment — do not read sibling pages whose key range
the delta stream does not enter.**

- Exact code boundaries: `crates/layerfs-content/src/tree/batch.rs`,
  `Engine::edit` branch loop (`:575-585`, the unconditional
  `self.read(child_id, false)?` before `self.edit(...)`), the early return at
  `:519`, `Node::existing`, and `apply_budgeted_root`'s summary accumulation
  (`old_count` / `old_bytes`).
- Mechanism to qualify: for a child whose bound range contains no pending
  delta, reuse the child's summary instead of reading the page. The parent's own
  stored subtree count/bytes already describe the sum of its unchanged children,
  and the entered children's old counts are known from their own reads, so the
  new parent summary is derivable without opening untouched siblings. The
  candidate must keep full authentication and decode for every page it does
  open, and must not weaken `check_child` / `NonCanonicalPagePartition`
  validation on the modified path.
- Measured ceiling: 1,920 of 2,053 reads and ≈ 94,000 of 101,001 record decodes
  are structurally unnecessary at K=100; the inner clocks put 224 ms of the
  245.4 ms namespace on that path and 59 ms of the 63.0 ms K=10 phase. That is
  the honest upper bound, not a forecast.
- Proposed targets to assess (not promised, not Stage 1 gates): the recorded
  `<=120 ms` K=100 namespace and `<=31 ms` K=10 namespace are consistent with the
  ceiling. The recorded `<=200 ms` K=100 public Commit should be re-derived from
  a per-phase budget before freezing: this session's plain K=100 Commit median is
  329.76 ms, of which namespace is 254 ms, so the Commit target is a 39 %
  reduction, not the 36 % implied by the baseline's 313.29 ms.
- CPU and memory: remove work, do not add caches. No new workers, no SQLite
  page-size or cache-setting change, no encoding/authentication weakening. If a
  bounded sibling-summary scratch is genuinely required, `<= 8 MiB` additional
  per active operation **and** an explicit aggregate concurrency bound; a
  per-operation cap is not a process-wide cap. CPU must fall at K=100 and must
  not regress K=1/no-change/repeat/revert beyond `max(10 % of control, 1 ms)`.
- Tests and falsification: paired control/candidate arms under the runner lock on
  this exact Stage 1 matrix; require the reduction to appear specifically in
  `namespace_ns` with unchanged canonical objects/bytes (112,684 / 513,774,250 at
  K=100), unchanged admitted pool values (100), complete changed-file bytes,
  unchanged sampled files, rewritten root, and identical reopen proof; counters
  must show `nodes_read` falling to the entered-page count with `delta_keys`,
  `nodes_created` and encoded bytes unchanged. Run the focused content/workspace/
  fuse suites plus split/merge and `NonCanonicalPagePartition` cases. **Falsify
  the experiment if** the paired namespace reduction is under 25 %, any canonical
  identity, verification or reopen proof differs, CPU rises, or any added scratch
  exceeds the declared budget. If it fails, report the honest alternative: a
  smaller lazy-decode-only change (Rank 2, ~28 ms) or no optimization.

**No edit-stage experiment is justified yet.** The measured removable cost there
is 14–16 %, and 86 % of the stage is unattributed; the required next step for
edits is measurement (host SDK edit-call internals and daemon EDIT_PART/EDIT_END
application), not an optimization.

## 7. Scaling ledger

| Mechanism | Stage 1 evidence | Class | Next proof required |
|---|---|---|---|
| Cumulative dirty-fact rows per edit barrier | 5,050 rows for K=100 (1+2+…+100), 100 % host-consumed; cost 92–104 ms, of which 88–100 ms is RPC, 3.7 ms re-encode | **confirmed triangular traffic, bounded cost** | K=200/400 row growth with a consumer-boundary proof before any removal |
| Frontier spill merge O(K²/B) | `spilled=0` in every Stage 1 cell (K=100 ≪ threshold); new counter test shows written records triple when inserted count doubles | **confirmed quadratic mechanism, untriggered at K=100** | production-budget K around B, 2B, 4B with run-descriptor bounds |
| Sorted-batch fallback (`changes.rs:2430-2450`) | never taken; forced-error path asserted with a 1-byte scratch ceiling (`ObjectLimitExceeded`) | **untested at scale** | forced fallback with mutation counts and root equality |
| Sibling read fan-out | 97 → 673 → 2,053 reads; 93.5 % entered by no delta at K=100 | **linear in entered parents with ~96× fan-out constant; dominant measured cost** | the Stage 2 experiment above |
| Per-object read cost vs working set | 57 µs/object cold (K=1 first) vs 11 µs/object warm (K=1 repeat) for identical work; 117 µs/object at K=100 | **cache-sensitive, uncontrolled OS cache** | paired runs with declared cache state; never claim coldness |
| Range × folio / barrier scans | 0 range tests, 0 retirement scans, 0 kernel-edit hits in this topology; flush compiled out | **inactive here, needs its own counts in a FUSE-writeback topology** | mmap/writeback and overlapping-range cells (rollout step 4) |
| Metadata history replay per reopen | not re-measured (baseline 127 ms of a 142.6 ms reopened Commit) | **O(H) per reopen, quadratic cumulatively** | separate workstream (#108-adjacent), not a Stage 1 target |
| Ordinary mutation work (encode, checkpoint, persist, admission, publication) | 0.28 / 0.45 / 0.55 ms, admission ~46 ms, publication ~0.1 ms | **bounded ordinary work** | unchanged paired gates |

## 8. Correctness, resources and custody

Every Stage 1 cell passed: proof PASS, expected `Created`/`UpToDate` outcomes and
head movement, complete bytes of all 1/10/100 changed files, 10 deterministic
unchanged sampled files, visible head after every Commit, final root and
edited-content verification after dropping owners and reconnecting the Store,
`end_workspace_session(Clean)`, zero active workspaces/executions, container
removed, no swap and no OOM. Bootstrap identity was gated in every cell
(112,451 canonical objects / 513,026,835 canonical bytes / 100,002 pooled
metadata values); K=100 ends at 112,684 objects and 513,774,250 bytes, exactly
the baseline's `112,684`. The analyzer re-checked the phase equation
(phases + `unattributed` = receipt total) for all 33 Commit rows and reported
**zero problems**. Plain collection 15/15 PASS in 95.2 s; diagnostic 10/10 PASS
in 58.7 s. No invalid attempt occurred, so the contract's one-replacement rule
was never used.

Resources (Commit #1 medians): plain K=100 user 278.7 ms / system 41.8 ms CPU,
RSS 92.1 MiB, threads 4, 0 B disk reads, 61 kB writes, no swap; diagnostic
K=100 user 268–270 ms / system 37–39 ms, RSS 93–96 MiB, container peak memory
19.8–20.4 MB against 2 GiB. Post-cell Store apparent 515.6–515.7 MB, allocated
516.9–520.1 MB, runtime spool cleaned to 0. Diagnostic scratch is bounded by
construction: fixed-size `Copy` accumulators, one emitted line per Commit or per
edit barrier (102 lines for a K=100 cell), no per-record logging and no
object-ID maps. Post-call RSS is reported as a snapshot and is never presented
as an operation peak.

Focused existing tests for touched paths, both arms, debug profile (release test
targets are blocked by a pre-existing `#[cfg(debug_assertions)]` gate in
`layerfs-layerstack-store`, unrelated to this work): diagnostic arm content 58 +
workspace 63 + fuse 31 = 152 passed, 0 failed; plain arm content 58 + workspace
62 + fuse 31 = 151 passed, 0 failed. The two new bounded counter-only tests are
`tree::batch::tests::stage1_sibling_reads_are_unconditional_and_scratch_limit_forces_the_fallback`
and `changes::tests::stage1_spill_merge_rewrites_the_whole_growing_run`; both are
work-count tests and make no timing claim.

Commands and exits are retained in the evidence root: `build-host-{plain,diagnostic}.log`
(both `BUILD_*_EXIT=0`), `build-image-{plain,diagnostic}.log`,
`collection-{plain,diagnostic}.json`, `plain-collection.log`,
`diagnostic-collection.log`, `analysis.log`, `fixture-validation.json`
(`VALIDATION PASS`, 100,000 files, digest `6fc793a9…`),
`{plain,diagnostic}-{identity,image-id}-summary.txt`,
`{plain,diagnostic}-focused-tests.log`. Source and product seals were re-checked
by the collector after collection and were unchanged. No `crates/`, `tools/` or
`benchmark/` edit occurred between either build and its completed collection.

Custody: the archived `layerfs-commit-baseline-evidence/20260911T142606Z` root and
all #104/#109/#110 roots were read only; no archived collector, validator,
analyzer or finalizer was executed in place, and no old snapshot, Store,
manifest or result was altered. The preexisting tracked compaction-removal diff
and the untracked `compaction-removal.md` / `issue112/` / `issue113/` files are
byte-identical to their recorded hashes.

## 9. Status and next step

#111 remains **open** under its original cold-Init <=2.7 s objective; further
Init optimization stays paused. The optimization targets in the rollout table
remain proposals: Stage 1 supports the namespace targets as reachable in
principle but does not promise them, and explicitly declines to set any
edit-stage millisecond target. No optimization was implemented or promoted, no
release, tag or deployment was produced, and the nonce-style Stage 1
instrumentation stays isolated in the diagnostic worktree rather than being
promoted to permanent telemetry.

Handoff-ready Stage 2: implement the sibling-read elimination at
`crates/layerfs-content/src/tree/batch.rs:575-585` under the frozen control/
candidate protocol above, with the stated falsification conditions, on this
exact array — nothing else.

Issue update: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5637855243

# Issue118 execution ledger

Started 2026-09-12 on main at `42599d4f7`. Current issue118 supersedes
historical campaign repetition requirements. This run stays on main; unrelated
web/cloud source preserved. The owner subsequently requested a separate
`layerfs-transition-experiments` worktree in another task; it is preserved as an
explicit owner exception to the original sole-worktree criterion. Its resource
work shares the measurement lock and is not our source or qualification. Historical raw evidence was owner-retired under
issue117 and supplies no current qualification.

Fresh raw records: `benchmark-results/host-store/issue118/20260912/`.
Starting dirty patch and untracked hashes are retained there. This run keeps
compact logs/receipts, independently copied active control binaries, and at most
two failed sample Stores needed for diagnosis; successful scratch Stores are
removed after proof. Fixtures and unique source are not scratch. Build retention
is enforced by the shared runner. Initial host free space: 345 GiB.

| Sequence | Work / required acceptance | Status |
|---|---|---|
| 1 | Review/consolidate intended dirty compaction removal; preserve unrelated source | Product/harness committed; unrelated web/cloud preserved |
| 1 | Shared build cache, immutable executable custody, bounded retention, focused infra checks, smoke and small pair | 47 infra checks + public smoke + n3 A/A + independent proof PASS; retention follow-up in progress |
| 2 | Highest-tier carry, merge failure atomicity, actual RAM/FD/disk accounting; focused and production-budget proofs | Component/default-budget/reduced-Workspace proofs PASS; see below |
| 2 | Complete namespace fixture validation; reduced-budget Workspace/public non-spilling qualification | Fixture + reduced Workspace PASS; public sequence qualification next |
| 3 | Fresh disjoint edit attribution >=90%, dominant fix, edit + matching Commit + chain qualification | Attribution 99.9% PASS; facts + demand-only lookup implemented/focused PASS; public pairing next |
| 4 | Reopen/history fix and deduplicated #108/#112/#114/#100/#107/#102/#110 scope | IN_PROGRESS |
| 5 | Authentic namespace-100000 cold Init <=2.7s and affected final checks | Current control FAIL 4.162195375s; optimization pending |
| 6 | After optimization: #116 capability audit published before capacity changes | PENDING |
| 6 | Bounded capacity fixes, boundaries/package workflow, default-budget public spill and affected rechecks | PENDING |
| Final | Exact identities/commits, independent proofs, command costs and issue outcome table | PENDING |

Performance disposition frozen before new collection: ordinary regression
screens use three fresh alternating pairs, median paired wall slowdown greater
than max(15% of control median, 3ms) and at least two of three pairs slower;
CPU uses max(15%, 1ms). Every attempt remains. Stronger applicable explicit
requirements remain. Stage2 K10 50ms/31ms absolute targets remain owner-WAIVED;
K100 near200ms is an engineering goal. Cold Init <=2.7s, correctness,
authentication, resources and evidence validity are hard gates. A selected
development row alone is not terminal admission.

Subagents own infrastructure, spill correctness, and history scope. Root owns
source consolidation, edit attribution and serialized public measurements.
No overlapping resource-sensitive work or nested runner lock acquisition.

## Current results (not terminal acceptance)

- `b232d3fad`: compaction removal, authenticated legacy reads retained; five
  compatibility checks and two ordinary probe/script checks PASS (7.73s/14.03s
  complete commands). Pre-existing unused_mut warning remains.
- `ce371d6bc`: shared host/Linux build caches, independent atomic executable
  copies, guarded build retention and fewer runtime round trips. 47 checks PASS
  in0.50s; qualified host build21.63s, first Linux image91.64s, real ordinary
  storage smoke3.54s. One earlier build was safely rejected by the measurement
  lock (0.08s); the fixture validator still owned it.
- `infra-pair-summary.json`: three alternating A/A pairs, original100-file
  namespace; every sample retained. Product19.318–23.847ms; runner1.254–1.366s.
  Median paired wall delta−0.062ms/CPU−0.028ms; no material regression. This
  measures repeatability and workflow cost, not a product speedup. Independent
  selected proof PASS in1.81s. The first proof invocation rejected missing
  explicit source/input arguments before execution; corrected invocation binds
  the exact performance identities.
- Original100000-file fixture fully validated:100000 files/500000000B,
  1001 directories, original digest/modes/mtime,125169 pages/zero resident.
  Full validation20.67s. Fresh cold control:4.162195375s,875929600 physical
  readB,127 admission transactions,112451 canonical objects/513026835B,
  Store apparent515481600B,zero swap/OOM,cleanup PASS; complete command24.82s
  including18.70s cold acquisition. Hard2.7s gate FAIL.
- Cold attribution is explicitly ineligible for performance acceptance:
  pipeline3.492643792s, consumer idle1.772718887s (nested), SQL commit0.749224298s
  (nested), final tree0.444217375s. Separate intrusive stack sample found read
  and open dominating producer stacks; no candidate improvement is inferred.
- `a98a157ae`/`9930130fb`: old highest-tier regression reproduced; six focused
  candidate checks PASS, real partial-write failure/retry/temp/FD proof, default
  B15873 at1/2/4/8B PASS,600-file reduced-budget Workspace Commit PASS. The
  old startup growth assertion failed after the one-run final copy was removed;
  it was replaced with exact theoretical traffic counts, retaining the failure.
- `a0ea64404`: dropped batch-memory lease reproduced (8392 retainedB charged0),
  then fixed with actual retained capacities. Seven batch checks, initial
  builder, real1024-byte-policy Workspace fallback and subsequent600-file spill
  integration PASS. Pending BTreeMap allowance remains conservative, not an
  allocator-exact whole-process memory claim. See stage2-pressure/ and stage3/.
- `16c027804`/`cd7cfba22`: exact retained metadata-window recovery and one
  exclusive-open FK integrity scan.22 focused checks PASS. Catalogue-header
  audit and remaining FK scan are still linear; public reopen proof pending.
- `4a79c100c`: optional edit attribution on the existing control transaction;
  no additional round trips and no optimization yet. Two selected protocol/
  mutation/retry tests and Workspace library check PASS. Test-only compile
  mistakes and the earlier Store integer-conversion compile failures remain in
  raw logs; none is a waived product failure.
- Current metadata-cardinality public control reproduces physical reservation
  failure in0.939s product command after31.394s normal fixture preparation.
  Full command32.950s; cleanup PASS. Required2106936B exceeds2097152B by9784B.
  The old prepared slot is184B;483 is reserved Vec capacity, not an observed
  occupied count. The batch-signature-vector repair has passed the full existing
  input in a debug check; optimized public confirmation is pending.
- That preparation exposed legacy automatic input eviction: it removed exactly
  `fixtures/10af090aa6e713b9ef41d786d48f91ad392233b941cf54ad70d06997fc03a274`.
  This generated input eviction is disclosed. Automatic input eviction is now
  disabled; acquisition must preserve qualified fixtures/prepared masters.
  No raw result/source was removed and the original namespace fixture survives.

Every raw path above is relative to the fresh evidence root at the top. Existing
ignored production proofs were explicitly executed where reported PASS. Other
required public/terminal checks remain pending, not silently inherited from
retired historical evidence.

## Reservation and edit follow-through

- `a36c60891`/`440584938` fix excess inline prepared-slot storage and stale
  input-association accounting without enlarging the2MiB physical budget.
  The original100000-file metadata-cardinality public case now PASSes in4.708s;
  complete performance/proof commands8.36s/7.52s. Independent verification is
  the registered bounded storage/reopen/edit proof, not exhaustive100000-file
  readback. Full root cause, numbers and identities: `physical-reservation-root-cause.md`.
- Fresh inactive K100 attribution found560.670ms unnecessary lookup work and
  97.993ms prior-fact publication in765.465ms edits. Named daemon coverage99.9167%;
  active preparation moved812.543ms into setup, so it is no optimization claim.
  See `edit-attribution.md` for disjoint intervals and all residuals.
- `158d8fad2` skips full prior-fact publication only for EDIT_BEGIN; snapshot
  consumers retain publication. `14ef14262` acquires only demanded SDK path
  metadata. Ordinary FUSE grouped prefetch and authenticated caches remain.
  Focused100-edit proof:200 requested nodes, zero optional sibling/content
  exports, real Commit/reopen and full bytes of100changed+100unchanged files PASS.
  Old one-lookup optional work99siblings/409600exportB falls to0; canonical
  Store bytes454334→6144 with identical requested metadata. Public timing pending.
- No external-library patches: all implementation is LayerFS-owned, with
  Cargo manifests/lockfile and dependency sources unchanged.

## Updated owner cold-target disposition

The owner explicitly superseded the2.7s absolute gate during this run:
"we need to get better but does not mean2.7 is a must because in v0.1.5 we
introduced authentication, pack, delta encoding which might increases time".
The absolute target is now **owner-WAIVED**; prior hard-gate descriptions above
record the earlier contract. Require measured current-code improvement and
no unexplained material regression, preserving full verified-cold acquisition,
authentication, packing, DELTA encoding and resource protections. Existing
runner TARGET_MISS remains visible and receives this explicit owner disposition.
See `owner-cold-target-waiver.json` in the fresh evidence root.

## Build freshness failure, retained explicitly

The post-lookup Linux image declared new source but contained byte-identical
old daemon/FUSE binaries. Cargo reused the shared target after COPY changed
source. The diagnostic caught unchanged4950facts/1340395wireB for100edits.
The new image and its two runs are rejected as exact candidate qualification;
all artifacts remain. The host cold screen4.150→3.900s is exploratory only.
Repair is in LayerFS-owned build invalidation, preserving cached external
dependencies. See `image-freshness-failure.json`; no performance retries were
launched against this falsely fresh image.

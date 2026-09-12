# Issue118 execution ledger

Started 2026-09-12 on main at `42599d4f7`. Current issue118 supersedes
historical campaign repetition requirements. One persistent worktree; unrelated
web/cloud source preserved. Historical raw evidence was owner-retired under
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
| 2 | Complete namespace fixture validation; reduced-budget Workspace/public non-spilling qualification | PENDING |
| 3 | Fresh disjoint edit attribution >=90%, dominant fix, edit + matching Commit + chain qualification | PENDING |
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

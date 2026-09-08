# Issue88 finalization work plan

Status: reconciled next-work plan; no new build, Store opening, read campaign or
optimization is executed by this document. This plan is separate from the sealed
full157 report. The current assignment authorizes planning; campaign execution
remains a separately authorized prospective task.

## Outcome to preserve and finish line

The combined experiment is complete:184,598,528B original Store allocation,
155,353,550B complete pack BLOBs, all157 histories verified in both arms,158-row
cohort proofs and reconciled accounting. Retain it as a research checkpoint.
The open question for finalizing #88 is ordinary public-read cost on actual native
dependencies, followed by a finite issue disposition. No134.2MB target, second
optimization or release qualification is added to the finish line.

Read the [sealed findings](../issue88-native-analysis/published/findings-and-next-experiment.md),
[independent review](../issue88-native-analysis/published/independent-review.md),
[prospective read contract](../issue88-native-analysis/published/next-read-contract.md)
and [regression qualifications](regression-review.md) before implementation.
Current issue body remains authoritative. Do not modify those sealed artifacts.

## One experiment, five work packets

| Packet | Owner and scope | Concrete output / exit condition |
|---|---|---|
|1. Custody and cohort | Analysis/cohort owner | Frozen paired file/range manifest with actual retained depth proof; no measured rows before this passes |
|2. Public probe adaptation | Probe owner | Same minimal analysis-tool source built separately against both unchanged product arms; reviewed identities and tiny protocol/selection checks |
|3. Fixed execution | Single measurement owner |60 declared observations, or preserved failed/incomplete evidence; originals unchanged and all owned resources cleaned |
|4. Decision and test debt | Independent reviewer plus coordinator | Scoped read-cost retain/revise/reject decision and explicit disposition of all existing failures |
|5. Seal, publish and close | Coordinator | Pushed report, additive issue summary, final issue disposition; no merge or release claim |

Source inspection and review may run in parallel. Only the coordinator runs builds,
checks that consume the measurement slot, Store/copy operations or observations.
Use the existing shared lock; do not nest locks or stop another task's owner.
Assign exclusive file ownership and preserve unrelated edits.

### 1. Establish exact custody and a usable cohort

Resume from actual worktree HEAD/status and active owners. Freeze the finalization
reporter revision separately from measured product revisions. The control producer
is2753453933c55ed7f93eb21619c235668a01ef4c; candidate is
d4f26f0d16f0f91c1f75767cf699012b8794ac01. Reuse their authenticated normal host
binaries/images and unchanged product sources. Archive source/product/tool/dirty
patch, compiler, dependency and image identities. Preserve the candidate's
analysis-only image dirty qualification; do not retroactively call it clean.

Use existing sealed full157 manifests, snapshots, inventories and validation
results. Authenticate any older extraction cache before using it. Final inventory
reference adjacency and native depth alone do not prove a historical path or file
offset: path labels and extent source offsets/lengths are absent. Reuse existing
source-manifest/chunk-extraction metadata to shortlist candidates, then prove each
selected retained file path and extent mapping through existing exact canonical
and reference decoders. Perform only a bounded selected-path walk, not another
whole-Store census, generic filesystem decoder or construction replay.

Freeze a finite preparation budget (candidate-path attempts, structural object
reads/bytes, elapsed time and spill/output bytes) from the existing source bounds
before selection; cap exhaustion is a preparation limitation, not an empty stratum.
No execution-ready contract may leave these budget fields unspecified.

Choose one deterministic source checkpoint/file/range per candidate selected depth
0,1,2,3,4. Preserve the prospective source-index, path-byte, offset ordering and
4096B range / at-most8MiB containing-file limits. The4KB range must actually touch
the selected native target through its authenticated extent mapping; where the
claim is a single-target range, require the whole range inside that target span.
Use native FULL for depth zero, not a legacy FULL. Preserve raw path bytes as hex
in the cohort, reject unsafe path components and pass decoded paths as separate
arguments to the fixed Bash/dd command, never interpolated shell source. Prove
admission chronology from authoritative acknowledgement provenance together with
the chosen retained-root traversal; final pack order alone is not a timestamp.
Record exact logical offset, extent source offset, target canonical identity,
locator, actual chain, depth and raw closure. A source-blob match alone is not
retained-path proof. Preserve a bounded exclusion tally; no raw file dumps.

Bind each selected case to the corresponding control state, source SHA/tree/oracle
and identical expected range/full-file bytes. Full-file results belong to a file
selected by one target's depth, not a claim that every chunk in that file has that
depth. Freeze file size and its available dependency composition. Missing strata
or insufficient mapping data are explicit cohort blockers before timing; do not
silently choose a different range or claim a zero-cost read.

Create a new prospective contract amendment and new output directory. Preserve the
sealed earlier contract unchanged. Amendments must explain the necessary cohort
proof and probe-build distinctions before any observations; they cannot loosen
encoding, read bounds or replace the frozen workload.

### 2. Adapt the existing public probe, not the product

Use a versioned analysis directory `issue88-native-analysis/depth-read/` for the
cohort-aware `run.py`, `src/main.rs` and `select_cohort.py`, based on the existing
`read-probe` implementation. Preserve the earlier probe source and sealed evidence.
Reuse its public SDK/runtime/resource helpers; do not create a second product read
implementation. Add a small structural-proof entrypoint only if existing decoders
cannot be invoked directly. Give the cohort owner exclusive selection/proof files,
the probe owner exclusive protocol/orchestration files, and the reviewer report-only
ownership. Do not put inventory traversal inside timed operations.

The old executable is hardcoded to the earlier SDK fixtures/checkpoint3 and is not
an executable solution to full157 selection. Feed an explicit authenticated cohort
to the existing probe and remove the obsolete fixture assumptions from the new
invocation path. Build and seal the identical adapted probe source separately
against each arm's unchanged product sources. These are new analysis binaries;
the existing normal host binaries/images remain the frozen product references.
Never label an old probe hash as the hash of the adapted executable. Keep matching
Cargo.lock/dependency versions and separate target directories. Do not add product
instrumentation, cache, encoder policy, private read path or benchmark family.

Before timing, review the smallest meaningful checks: malformed/mismatched cohort
rejection, bounds and path/offset handling, correct source mappings, lifecycle
cleanup on protocol errors and unambiguous row order. Passing checks are not new
storage/performance evidence. Reuse existing successful reader/codec tests; no
blanket full-suite rerun is required for an analysis-only adaptation.

### 3. Execute the frozen read campaign once

Five candidate depth strata × two operations (4096B range/full containing file) ×
three repetitions × two arms =60 observations,30 per arm. Order arms C/P,P/C,C/P
within each fixed stratum/operation. Freeze exact commands and identities first.
Use one logical disposable Store copy per arm, fresh process/container/session per
row and the existing public fork/mount/Exec/terminal-drain/End/cleanup route. No
per-row Store copy, Commit, full157 replay or recompression. Copy allocation is not
a storage comparison. Original Stores stay unopened by mutable SDK/SQLite owners.

Time read/count Exec through terminal output drain. Digest verification is separate.
Record setup, read, digest, End, cleanup, case/invocation and observer scopes. Check
expected counts/digests and that per-read native dependency counters demonstrate
the selected target's depth was exercised. A depth-zero event alone need not prove
a native FULL decode; validate representation and actual decode/fetch activity.
FUSE counters exposed only at End retain whole-session scope; missing phase-local
metrics stay null. Do not invent zero writes from an unavailable write counter.

Container limits are2CPU/2GiB/no-swap/256PID; host observed RSS limit is8GiB. Preserve
30s read/digest,120s lifecycle,4h campaign,16GiB scoped runtime,32GiB owned outputs
and50GiB free reserve. Current/lifetime peaks and cumulative/sample scopes remain
separate. Fresh processes do not make OS-cache cold; do not flush caches or claim
cold/random-read qualification. A depth/byte bound is not a latency measurement.

Stop on identity, cohort, correctness, resource or cleanup failure. Preserve failed
rows and cleanup evidence. Never replace an outlier or continue with a new cohort
because a result is unfavorable. A lock rejection before launch is not a sample;
record it and coordinate, without bypassing the lock.

### 4. Reconcile one decision and disposition existing debt

Report all raw integer values and each cell's n=3 median/min/max plus paired absolute
and percentage changes. Do not report p95, population tails, universal worst-case
bounds or statistical confidence from three repeats of one selected file. Selection
is descriptive coverage across depths, not a representative workload sample.
Different strata also differ in content, locality and history: compare arms within
matched cases, not a causal latency-versus-depth slope across five different files.
Retain the existing23.19% full-history read/digest elapsed increase,26.17% CPU
increase, increased Commit write traffic and prior74.42ms read outlier.

Owner's roughly30% foreground guidance is an interpretation reference, not a new
automatic gate. Judge absolute time for short calls. No numeric threshold is to be
chosen after seeing the new observations. The decision must explicitly distinguish
acceptable selected-case costs from unqualified cache/workload/tail coverage.

| Evidence outcome | Research disposition | Issue action |
|---|---|---|
| Valid complete campaign, acceptable scoped costs | Retain research checkpoint with limitations | Final report and close research issue after explicit disposition |
| Valid complete campaign, unacceptable costs | Revise recommendation or reject adoption; keep evidence | Close experiment with negative decision and name a separately scoped follow-up; do not silently optimize |
| Valid but insufficient evidence for broader adoption | Retain research-only or decline adoption; state uncertainty | May finalize research with explicit owner disposition; no universal qualification claim |
| Demonstrated candidate corruption, invalid dependency or resource-bound violation stops a valid campaign early | Reject candidate adoption, or recommend separately scoped revision; preserve the scientific negative | Complete owned cleanup and seal the failure, then close research after explicit negative disposition; no automatic fix/rerun |
| Invalid cohort/identity, infrastructure or incomplete cleanup | No product cost conclusion | Keep open with exact blocker/remaining step; do not convert invalid evidence into a negative product result |

Existing62PASS/9FAIL/1ignored remains failed. Reconcile a finite ledger from the
existing regression reports: each of the nine names, P/D/accepted behavior,
source/receipt links, limits of equivalence, research disposition and responsible
subsystem/follow-up. Preserve P/D/accepted cleanup residues19571/19573/19572 and
unexpected-success details; D's older opaque error remains unknown. Matching
failure names do not waive defects. Fix or explicitly disposition adoption risks
before adoption; closing this investigation does not require repairing all baseline
fault/schema/order defects. Any newly demonstrated candidate correctness defect is
a blocker to retaining that candidate and requires separate source/evidence custody.

No product cleanup is bundled. Keep legacy readers and structural matching because
live callers need them. Directory Init's missing FILE provenance remains a recorded
coverage limitation; fixing it would change the treatment. Any future integration
must address compatibility: old readers cannot read version2 Stores. Pack-level
read rejection alone does not prove that an old writer is prevented from opening
a same-SQL-schema Store; writer/downgrade behavior needs a separate adoption decision. No migration,
rollout, merge, M5 or release qualification follows from research closure.

### 5. Publish the finite result

Create a new immutable finalization report with contract/cohort/schedule, exact
producer and probe identities, raw60-row evidence or explicit incomplete state,
resource/correctness/cleanup checks, original-seal checks, decision, test-debt ledger,
independent review and manifest. Link the existing184.60MB report; do not rewrite
or re-census it. Separate protocol/tooling corrections and their identities from
product evidence. Preserve failed attempts and all negative results.

Commit/push only relevant analysis/probe/report documentation through the existing
workflow; verify published bytes against seals. Add a concise #88 closure summary
with evidence links, scoped recommendation, limitations and remaining separately
owned adoption work. Close #88 only when the explicit research closure criteria
above hold. Do not close #87, parent allocation/release issues or merge a PR merely
because this work ends. Do not start anchor substitution, canonical simplification
or SQLite optimization as part of finalization.

## Independent planning inputs and reconciliation

Three bounded planning reviews were completed without measurements, builds or Store
scans: [read/probe feasibility](finalization-reviews/read-plan.md),
[integration and exact test-debt ledger](finalization-reviews/integration-plan.md),
and [independent closure review](finalization-reviews/closure-review.md).

The coordinator reconciles their wording as follows: range reads require full
containment in the authenticated selected span, not merely intersection; native
depth bins supplement static target/path proof and are not identity proof alone;
full-file strata are mixed-depth; extrema mean largest observed values, never a
worst-case guarantee. New probe binaries have new identities, while frozen product
sources and normal images remain unchanged. A valid negative research result may
close the generation; unresolved identity/infrastructure/cleanup cannot be hidden
by closure. Recording baseline debt does not repair it or authorize adoption.

This plan is the requested deliverable. The next execution assignment can authorize
implementation, the one frozen campaign, reporting and issue disposition together;
no redundant per-step permission should then be introduced within that scope.
No experiment, product edit or issue closure occurred while producing this plan.

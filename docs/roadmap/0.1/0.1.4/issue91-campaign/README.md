# Prospective v0.1.4 qualification campaign — issue 91

Status: frozen before execution tooling changes, builds or new samples. This is
an implementation and execution plan, not a result or permission to release.
The authoritative assignment is issue 91 as retrieved 2026-09-09. It supersedes
phase1's encoding-only primary-control proposal. Execute, diagnose, repair,
requalify and publish; a severe repairable regression is not a reporting waiver.

## Sources, workspace and custody

Integration base/candidate G0 is 46308986aec091337573227ede6c35aff7db11f2.
Verified phase1 product is 593f4ad018bf34b3f180baf66e1ae5cf40c36647;
reports are 930d35ab573256598f14fe0d5a3a1512244e0fb5. Work uses isolated
`codex/issue91-v014-qualification` at `layerfs-issue91`. Original main remains
c374f8c3b25e923831d25af2304c0d6bf8ac0d9e with unrelated dirty documentation.
No existing worktree is reset. Initial process/lock/container inspection found
no LayerFS execution, measurement lock holder or running container. Repeat
ownership checks under the lock before execution; never open an active Store.

[Baseline manifest](baseline-manifest.json) hashes all 654 historical evidence
files and release verification directly from immutable tag v0.1.3, resolving to
49f0f5be911e7f5b92afc2becab45144ad7a1307. Accepted checkpoint is
9f5a641d223606c45e5e6aa8a20094c12f9139a1. The original receipt producers remain
as recorded inside each receipt. Release source includes a later FUSE guard
retirement repair; the complete performance campaign was not rerun afterward.
Read the baseline README, complete per-case report/JSON/CSV, registry/declaration,
raw performance/proof resources and release-notes/0.1.3/verification.md.

[Source contracts](source-contracts.json) bind the starting execution sources.
Seal each generation after tooling/repairs: clean commit/tree, dirty patch bytes
and SHA256 (empty when clean), runner source/product/workload/harness identities,
Cargo.lock, build profile/toolchain, executable hashes, immutable Docker image
ID/configuration and reporter hashes. A plan/report-only commit does not change
an executable's producer. Relevant changes require a new generation, never
rewriting old samples. Build each relevant source once; use explicit unchanged
source/contract applicability for any retained evidence on the final candidate.

## Matrix and comparisons

[Registry](registry.jsonl) contains every current ordinary definition in source
order; [mapping](baseline-mapping.csv) retains every historical admitted ID;
[declaration](declaration.json) freezes exact membership, counts and order.
Independent source audit establishes 198 performance IDs /16 performance
families, plus 29 proof definitions /17 total families. Family definitions,
fixture generators, ordinary operation/timer modules and verifiers are identical
between v0.1.3 and G0. Runtime registry, recipe/content, public route and timer
must confirm this match before a comparison is eligible. Matching names alone
never establishes comparability. Preserve incompatible/new/retired entries with
explicit contract reasons; no unfavorable exclusion.

One complete seed1 sample per ordinary case; SDK repetition1. This deliberately
continues the current checkpoint profile, not the earlier v0.1.2 paired
five-repetition release experiment. Available registry ranges (SDK1–5, other
performance1–3) are inventoried, not claimed executed: 706 selectable performance
slots/736 supported verifier slots exist, but this fixed-observation campaign
selects198 performance/226 routine verifier slots. It makes no multi-seed,
statistical speedup or population-tail qualification. Aggregate for n=1 is the
observation itself; show raw n/min/median/max honestly. Additional diagnosis
must prospectively name its cases, seed/order/repetitions and remain separate.

All five non-endurance extended reliability members and every depth500 history,
tier500 mixed workload and large Store/namespace case are REQUIRED. The
600-second definition remains optional under current QUICKSTART and is explicitly
not selected; retain its mapping and NOT_RUN_OPTIONAL row. No 600s endurance
claim. No short proof is relabeled endurance. Retired five uncapped500MiB growth
IDs violate the current result-size cap and are not current applicable IDs;
active versioned replacements remain required, with historical IDs preserved.

Additional current entrypoints: execute small-files and all four frequent-edit
SDK/FUSE text/binary cases once with independent proofs. DeepSeek-five is a
selected readiness subset, not another qualification family; required full157
executes all157 states and verifies all157 mappings. No new benchmark family.
These have no v0.1.3 ordinary-row baseline and receive no invented ratios.

Primary comparison uses published v0.1.3 `elapsed_ns` and available corresponding
phase/resource values, NEVER previous_elapsed_ns/difference columns. At every
case/metric, difference=C-B; percentage=100*(C-B)/B for B>0, otherwise unavailable
with reason. Retain baseline and candidate identity, size/profile, seed, timer,
operands, units, actual n, aggregate, separate correctness/resource/cleanup/
custody/target/severity statuses and raw evidence. Historical single observations
are not a fresh paired experiment. Overall means no conflation of distinct
operation surfaces or sum of overlapping resource scopes; show per-family and
worst-case costs rather than concealing them behind one grand average.

## Operation, execution and observation contracts

Use unchanged public SDK/CLI/real-FUSE family operations, fixtures and independent
oracles bound by source-contracts.json and the registry. SDK uses one public
range-edit plus Commit; no edit-caused FUSE or spool writes. Preserve Init,
Create/Exec/SDK edits/Commit/publication-finalization/reads/End boundaries.
Namespace timer=layerstack_init_ns; SDK=edit_commit_ns; Store=product_call_sum_ns;
other ordinary families=pure_call_sum_ns. Nested phases/counters are diagnostic,
not additive to their enclosing time. Preserve fixed workload sizes and seeds.

macOS26.4.1 arm64, Apple M3 Max/Mac15,10,14 logical CPUs,38654705664 bytes RAM;
Docker Desktop Linux arm64 daemon/FUSE/workloads only. Host owns SQLite,
SDK/coordinator, canonical construction/publication and spool. Release Rust1.85.1,
locked Cargo graph, two build jobs; use existing runner Dockerfile pinned base
and source-bound image. Container2CPU/2GiB/no swap/256PIDs/no data-sharing mounts.
Record actual host/container versions and profiles at generation sealing.

One coordinator serializes builds, heavy tests, live checks, censuses and all
samples with `$TMPDIR/layerfs-infra-measurement.lock`. Reviewers only inspect
source/receipts. Reuse authenticated closed protected preparation; fresh writable
sample ownership and fresh runtime each time. Init uses fresh output. OS cache
is uncontrolled/warmed by ordinary preparation: no OS-cold claim. Keep source
producer and candidate identities distinct for compatible cached masters.

Readiness: selected payload-create-1m, insert-middle-4k-on-1mib,
namespace-100-compact-v3 and deepseek-five, each once, separate diagnostic paths.
Then ordinary performance in declaration order, all independent verifiers after
performance, additional smokes, full157 control then candidate. Preserved failures
trigger a focused repair loop before expanding an affected cohort. Never rerun
successful cells to improve numbers. An unchanged successful diagnostic is not
silently promoted to a full campaign sample.

Ordinary collection budgets retain300s product/310s outer/600s preparation;
verification45s work/59s hard. No after-result increase. Historical15s and Git
500/1000ms targets retain their reporting-only checkpoint classification;
strict bulk tier100 subsecond assessment and accepted SDK20/20/30ms ceilings
remain visible and binding where their original contracts apply. Frozen
resource/route/accounting correctness assertions remain unchanged.

## Prospective severity and acceptance

Hard failure: timeout/hang/OOM/swap, wrong result, missing required resource,
forbidden route, source/fixture/timer incompatibility, Store ownership violation,
failed cleanup or custody. Execution PASS alone is not qualification.

For every comparable elapsed phase/public-operation metric, review when increase
is >=15% AND >=1ms. Severe when >=30% AND the absolute increase is >=5ms for
baseline<100ms, >=50ms for100ms<=baseline<1s, >=250ms for baseline>=1s. Also severe:
new breach of a binding original gate, regardless of these added thresholds.
These are diagnostic/repair triggers, not permission for universal30% slowdowns.

CPU severe threshold: >=50% and >=50ms extra CPU in the same scope. Memory:
>=50% and >=64MiB extra in matching scope; physical writes: >=50% and >=16MiB
extra. Review any measured allocation increase; severe storage increase >=10%
and >=1MiB. Always retain phase/resource limitations and all unfavorable rows.

Accept only with complete selected matrix, valid evidence, all applicable hard
and binding original gates passed, all severe entries causally resolved and
revalidated, and >=10% lower acknowledged full157 allocation at equal retained
state against the prospectively named supplemental control. This is not a
promise of184.60MB or134.2MB and is not an encoding-only headline. Revise if a
feasible repair remains; reject only for demonstrated in-scope unrepairable
tradeoff/platform/contract blocker. No average overrides a severe member.

## Supplemental equal-retained-state full157 storage comparison

Workload manifest03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271,
Git tipb0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed,157 states and4936693030 logical
blob bytes. Authenticate existing deepseek-history-data input/receipt/oracle
files. Empty Init, same public import/Commit route, all pinned retained states.
Use current candidate G0 (or repaired final generation), never old #88 candidate.

Supplemental fresh control sourcebfbc46c11dbb4d8dd949b54845e222cf42cd822f is the
existing C+S1 legacy producer with the full157 public harness. Preserve that
original worktree; isolated control if changes are necessary. This is explicitly
NOT the v0.1.3 baseline and NOT an encoding-only treatment: it lacks later
initialization/ownership/batching/compatibility fixes. Compare equal logical
state and report every source difference. Its new observations must not replace
the published v0.1.3 primary198-row record. v0.1.3 lacks this additive harness;
its full157 storage metrics remain unavailable unless separately frozen and
implemented as a diagnostic backport. Old #88 numbers remain historical only.

Control then candidate, one fresh Store per arm. Build/seal both before collection.
300s public steps,4h each performance/verification phase,120s setup/cleanup,
16GiB Store and16GiB scoped runtime/spool,32GiB owned outputs,50GiB free host,
8GiB sampled host RSS; standard container caps. Initial free space approximately
409GiB. Original Stores/customer data never opened for mutation.

After normal acknowledgement/End and authenticated owner cleanup, make exactly
one pre-verification logical snapshot per arm. Reuse snapshot_combined,
census_combined, roles, prepare_combined_validation, account_combined and resource
report logic with minimal metadata/schema adaptation; build native-aware census
against final candidate. No per-Commit copy/traversal. Preserve allocation of
original acknowledged Store and sidecars, SQLite logical size/page partitions,
pack/index/metadata/freelist/unused overhead, temporary space and signed allocation
adjustment without overlapping sums. Authenticate all selected IDs and physical
dependency closure, including unselected retained bases. Then all157 independent
historical verifications, normal cleanup, original seals unchanged. Copy/census
warming and verifier CPU/I/O remain separate from performance.

Historical adverse context remains: #88 verification read/traversal/digest+23.19%,
CPU+26.17%, increased write traffic, read outliers, depth3/full25.70ms vs12.38ms.
Its60-read five-small-file diagnostic did not qualify general large/cold/tail
behavior. Current random-read/mixed-file families retain their exact measured
scope; no unmeasured cold-cache or tail assertion.

## Implementation, repair, review and publication

1. Commit this plan and complete mapping before editing execution/collecting.
2. Extend existing issue54 collector to consume declaration membership/exclusions
   and reject changed/missing/duplicate cells; preserve historical mode. Reuse
   existing report.py derivation without changing v0.1.3 files; add v0.1.4 metadata,
   mapping/deltas and ledger. Focused product-free tests exercise malformed
   comparisons, units/timers/resources/custody, counts, and historical/diagnostic
   separation. No new runner engine.
3. Build/seal; selected readiness; complete declared performance/proof campaign.
   Record attempts under checkpoint-evidence/raw/generations/<generation>/;
   immutable receipts/manifests plus declaration and exact commands per attempt.
4. For each severe/hard failure retain operands, establish comparability and
   environmental effects, prospectively freeze smallest diagnostic, trace every
   caller through the shared cause, repair within scope and add meaningful check.
   Preserve canonical auth/CAS/CDC/COW/retention/compatibility/bounded ownership/
   synchronous publication/existing durability. No dependency-source patch,
   benchmark-specific product shortcut/cache/backend/deeper search/fsync promise.
5. Revalidate affected phase1 native/Store/schema/compatibility/large-spill/live
   SDK/FUSE/resource/quality guarantees. New sealed generation reruns every
   affected required case/proof; explicit applicability binds unaffected evidence
   to final source. Retain failures/outliers, no threshold changes.
6. Independently review complete mapping, comparison operands/timers/resources,
   storage equality, repairs and final custody. Resolve findings. Commit/push
   implementation/tests/raw evidence/CSV/JSON/Markdown/ledger, create reviewable
   PR. No merge/tag/release/deploy or unrelated issue closure. Close91 only for
   fully published successful qualification. Concrete blockers keep it open with
   exact evidence and smallest unblock action; feasible repair work continues.

Command templates (run from isolated repo; SEALED_* values MUST be replaced and
saved in the immutable generation declaration before any corresponding sample):

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
python3 benchmark/fs-bench-pro/issue54_collect.py --checkpoint \
  --campaign docs/roadmap/0.1/0.1.4/issue91-campaign/declaration.json \
  --image "$SEALED_IMAGE" --host-binary "$SEALED_HOST" --phase inventory --output "$GEN"
# Verify generated registry against frozen registry, seal exact identities, then:
python3 benchmark/fs-bench-pro/issue54_collect.py --checkpoint \
  --campaign docs/roadmap/0.1/0.1.4/issue91-campaign/declaration.json \
  --image "$SEALED_IMAGE" --host-binary "$SEALED_HOST" --phase performance --output "$GEN"
python3 benchmark/fs-bench-pro/issue54_collect.py --checkpoint \
  --campaign docs/roadmap/0.1/0.1.4/issue91-campaign/declaration.json \
  --image "$SEALED_IMAGE" --host-binary "$SEALED_HOST" --phase verification --output "$GEN"
python3 benchmark/fs-bench-pro/shared/runner.py --deepseek-full \
  --source-arm candidate --repetition 1 --image "$SEALED_IMAGE" \
  --host-binary "$SEALED_HOST" --output "$FULL157"
# Snapshot/census before this independent verifier:
python3 benchmark/fs-bench-pro/shared/runner.py --deepseek-full \
  --source-arm candidate --repetition 1 --image "$SEALED_IMAGE" \
  --host-binary "$SEALED_HOST" --storage-verify-run "$FULL157"
```

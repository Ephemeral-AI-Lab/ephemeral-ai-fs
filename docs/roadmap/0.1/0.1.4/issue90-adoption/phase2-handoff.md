# Exact candidate handoff to #91

Phase1 supplies one stabilized, verified source candidate:
`593f4ad018bf34b3f180baf66e1ae5cf40c36647` (clean tree
`755ad78b2fa090d5afda6c27a1086dd628b3ec91`). Use the
[verification report](verification.md), [identity](verification/identity.json),
[compatibility contract](README.md), [resolved ledger](defect-ledger.md) and
[independent review](independent-review.md). Later commits in the adoption PR
contain reports/evidence only; compare the product/source seals before reuse.

This candidate reconciles research9a2b34851, actual main49f0f5be9, and committed
#71 work8d15ebc9b. It preserves shared CAS/index/publication, canonical identity,
CDC/COW, legacy/S1 readers/matching and bounded native dependencies. It adds
failure cleanup ownership, resource-safe batch formation, explicit format7
compatibility and supported ingestion provenance. These are material source
changes from the recorded #88 producers.

## Control basis and permitted differences

Freeze a control from the same repaired integration source. Keep #71 construction
and checkpoint changes, admission ownership/cleanup,512-object batching, format7
fence, FILE/span transport, all correctness fixes and every harness/oracle/input
identical. The intended encoding control uses the shared legacy C+S1 admission
lane while the candidate enables the existing native FULL/PREFIX lane. A narrow,
prospectively sealed source change at the native partition decision in
`crates/layerfs-layerstack-store/src/objects/admission.rs` is the proposed sole
product treatment. Preserve native/legacy readers in both arms and verify this
control's behavior before collection. No control branch/build/measurement is
created automatically by phase1.

Do not substitute main's schema5 implementation or the original historical
control as an automatically matched time pair. If #91 instead compares a broader
adoption treatment, record every differing repair and do not attribute its full
result to encoding alone. Avoid a benchmark-specific product switch or altered
public operation. Both arms need fresh independently owned Stores; format7
adoption does not migrate legacy6 or research-native6 artifacts.

## Required phase-2 preparation

1. Inventory the current active family/case/size/seed/repetition/verifier and
   extended-lane registry on this exact source; freeze cardinalities and
   applicability before collection. Phase1's finite tests do not replace it.
2. Freeze the comparable control commit, candidate commit, release host binaries,
   Linux image IDs, workload/oracle/fixture seals, cache treatment, execution order,
   timing boundaries and existing acceptance gates. Reuse this candidate's
   source-bound Linux image only if source/product/environment seals still match.
3. Revalidate any source change against the affected phase1 correctness proofs.
   A new eligibility/batching/ownership change produces a new candidate identity.
4. Run the full declared public-operation matrix under the existing measurement
   lock and host-owned Store topology. Preserve failures, partial runs and outliers.
   No full-family collection has started in this assignment.

## Questions that remain for evaluation

Measure storage and elapsed/CPU/memory/I/O together on equal retained state.
Directory Init now has native eligibility; the encoder sees a different initial
representation/base population. Smaller batches and admission-lifetime writer
serialization can change foreground throughput, contention and pack layout.
Measure these costs rather than inferring them from correctness or lower memory
reservations. Preserve synchronous finalization and the existing durability
contract; no new fsync guarantee, cache, backend or deeper search is implied.

Repeat the applicable retained-history comparison on this source. The old
184,598,528B allocation /155,353,550B pack result is historical only. Keep the
original335,552,512B M4.5 and218,116,096B fresh-control references separate from
new paired measurements. Keep the historical verification elapsed+23.19%,
CPU+26.17%, greater Commit write traffic, earlier outliers and depth3/full25.70ms
versus12.38ms adverse pair. The60-read diagnostic covered five small single-extent
files; it did not qualify general large-file, cold-cache, mixed-depth or tail work.

The outcome of #91 may be adopt, revise or reject. Phase1 correctness completion
is not an affirmative performance/release decision. No PR merge, release tag,
rollout, parent-issue closure, original-Store rewrite or phase2 launch is authorized
as a side effect of this handoff.

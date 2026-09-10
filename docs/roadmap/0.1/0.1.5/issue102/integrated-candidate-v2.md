# Corrected candidate prerequisite: integrate the 65.96 MB experiment

Tracked in [issue #103](https://github.com/Ephemeral-AI-Lab/layerfs/issues/103).
#103 owns implementation and focused tests; #102 follows with full benchmarks.

The owner corrected the execution order: integrate the selected offline work,
then run benchmarks on that runnable implementation. The schema9 initial2
campaign is diagnostic evidence for the previous product, not qualification of
the intended candidate. Stop further broad schema9 collection. Preserve its
failures and the applicable receipt/verifier/build-custody repairs.

**Current prerequisite status (2026-09-10): integration, stride3 and full157
storage/oracle checks are complete.** The owner authorized full157 after stride3.
The unchanged promoted product measures **55,476,224 allocated B /157 of157
original states verified**,10,523,776B below66MB. All11 full157 historical-access
performance cases and11 verifiers pass15-second contracts. Measured source
`786d29575b1b7cf1123b5f9b8c97f1e4610c2bab` changes only runner registration/custody
from the earlier candidate; its executable bytes/product seal are unchanged.
[Final result and exact candidate handoff](../issue103/full157-integrated-results.md).
Explicit compaction cost and substantial cold amplification remain recorded
tradeoffs. The broad #102 campaign and matched released-control run remain open.
Earlier checkpoint paragraphs below are retained chronology.

Implementation order:

- [x] Public compact namespace: scoped serial inode IDs, inline inode values,
      compact directory references, durable allocation and legacy compatibility.
- [x] Authenticated physical inode-value groups and bounded metadata deltas.
- [x] Whole-file content prefix graphs and authenticated native chunk slices,
      with all bases/index/pool bytes charged and bounded decoding.
- [x] Public creation, edits, Commit, reconnect, fork and historical reads use
      the integrated formats; no Python reader or fixture-specific converter
      substituted for the product implementation.
- [x] Qualify exact bytes, metadata, hardlinks, branching, failure recovery and
      corruption rejection before collecting the full mandatory registry.
- [x] Measure integrated stride3 first, then full157 and historical_access,
      retaining original-state verification and complete allocation custody.
- [ ] Run the complete mandatory benchmark campaign and matched released control
      on the frozen integrated candidate.

The offline 65,957,888-byte database is the reference, not a promised online
allocation. It rewrites canonical namespace/commit identities and chooses a
whole-file base graph from a matched Git pack. The live implementation must
choose bases without fixture/oracle access and must preserve existing supported
Stores. Record differences and their measured storage/read/write cost explicitly.
Keep the 15-second historical_access limits and original state oracles. Preserve
all earlier results; do not relabel them as integrated-candidate evidence.

## #103 implementation checkpoint — 2026-09-10

[Native integration progress](../issue103/integration-progress.md): new
schema-10/pack-v4 SmallContent framing is under focused correctness tests. This
is only one format component, not the selected integrated candidate. Public
compact namespace, shared metadata groups and whole-file/slice integration
remain open. No integrated stride3/full157 or historical_access run exists yet.
The broad #102 campaign remains deferred; schema9 diagnostics stay preserved.

### Namespace foundation update

[Continuation 2](../issue103/integration-progress.md#native-namespace-checkpoint--continuation-2)
adds real scoped-inline namespace construction from empty initialization and
workspace Commit, with durable allocation and passing affected suites. Directory
initialization, compact reconciliation and the remaining physical storage
mechanisms still gate the integrated candidate. No history campaign or size
qualification has run; the broad #102 campaign remains deferred.


### 2026-09-10 — issue103 continuation 3 and revised scope

The user narrowed execution to full promoted-method integration followed by
**stride3 optimization/verification only**. Full157 is outside the revised scope;
no broad #102 campaign or release qualification is implied. Native directory
initialization and compact reconciliation now pass focused development checks.
Schema-10 metadata pack v5 integrates bounded 16-edge/128-KiB canonical delta
chains with authenticated intermediate bases and unchanged legacy contracts.
Shared metadata-value groups and whole-file graphs/native slices remain pending;
stride3 has not run and no integrated allocation is claimed. See
[issue103 integration progress](../issue103/integration-progress.md) for exact
commands, retained failures and evidence. Recorded offline/Git references are
unchanged and are not fresh product measurements.


### 2026-09-10 — issue103 shared metadata groups integrated

Schema-10 public initialization/admission/Commit now uses pack-v6 pooled inode
leaves and bounded metadata deltas. Append-only product-owned ordinals preserve
canonical IDs; full group digests and every intermediate leaf are authenticated.
A bounded disposable macOS lookup index replaces permanent per-value indexes and
rebuilds from published groups on reopen. Sharing, rollback/cache disposal,
corrupt pool dependencies and public lifecycle tests pass. Exact policy, memory/
work limits, retained failures and test commands are in
[issue103 integration progress](../issue103/integration-progress.md).
Whole-file graphs/native slices remain pending. **Stride3 is not yet measured**;
full157 and broad #102 execution remain outside the revised scope.


### 2026-09-10 — issue103 whole-file product compaction implemented

The public `LayerStackStore::compact_into` operation and `layerfs-store-compact`
binary now write authenticated whole-file prefix graphs and native chunk slices
using only already published Store content. Selection reuses product min-hash
signatures; canonical IDs/logical records are preserved, sources are retained,
and destination publication follows full intrinsic verification. Explicit
compaction cost, temporary storage and resources must accompany the final size.
The content container and four-KiB final layout are integrated with public reads,
future writes/Commit, fork, reopen and repeated compaction. The focused development
suites pass 342 tests (4 existing ignored), including corruption, quota failure
and publication recovery. See [issue103 integration progress](../issue103/integration-progress.md)
for exact policy, limits, commands and retained failed attempts.
**Stride3 remains unmeasured** pending runner/qualified-build/live-Exec gates.
No full157 or broad #102 campaign has run under the revised scope.


### 2026-09-10 — source-sealed integrated stride3 result

[Final report](../issue103/stride3-integrated-results.md) records the qualified
source/build/fixture identities, public operation policy and all retained commands.
The53 direct public saves yield53 Created and no presentation failures. Complete
allocated storage is65,056,768B before supported compaction and **46,202,880B**
after it; all53 original content/metadata oracles pass on the measured Store.
The frozen preverification image remains archived; subsequent verification-only
fork growth is reported separately. Full157 has not run, so66MB/full157 storage
is **NOT QUALIFIED**.

Compaction costs339.132858s,135,069,696B peak RSS and281,907,200B sampled temporary
allocation; save Exec/Commit sums are118.863256s/23.732028s. All11 mapped
historical-access cases and all11 verification runs pass15-second contracts,
including preparation and cleanup. No read-limit failures occurred; cold range
amplification remains17,029,550 decoded bytes for6421 requested bytes.

The final result is8,179,712B below historical offline53 and3,129,344B below
historical Git53. These comparisons are storage-only recorded references, not
fresh paired speed measurements. Product min-hash candidate selection improves
content encoding without removing any of the matching Small/whole identities.

This is the candidate and evidence available for later authorized #102 work.
It does not qualify the full mandatory registry or a release. Preserve old
schema9 campaign qualifications and initial-1 invalid custody; neither is promoted
by this result. #103 remains open for its original full157/handoff obligations.


### 2026-09-10 — full157 completed; candidate handed off to #102

[Final full157 report](../issue103/full157-integrated-results.md) supplies the
source-sealed candidate, exact build/fixture/Store identities, commands, complete
physical attribution and retained evidence. Full157 is55,476,224 allocated B,
157/157 original states verified,157 Created and no presentation failures. The
66MB target is ACHIEVED with10,523,776B margin. Source preservation, publication,
sync and cleanup all pass; verification-only Store growth is separate.

The API compaction costs626.313062s,131,792,896B peak RSS and407,638,016B sampled
temporary allocation. Exec/Commit sums are314.027550s/62.689607s. All11 original
historical-access cases and11 verifiers pass; the cold6421B range decodes22,216,028B.
Storage success does not remove this amplification or qualify the full registry.

Use measured source`786d29575b1b7cf1123b5f9b8c97f1e4610c2bab`, product seal
`b1a94e2223b1cd6c0eedffa3c6c60eca7134727c45b9018e1cea518cdf6d3dd5`, and the archived
binaries/identities identified in that report as the completed #103 prerequisite.
No broad #102 campaign, matched released-control speed comparison or release has
been performed. Preserve earlier schema9/initial-1 qualification boundaries.

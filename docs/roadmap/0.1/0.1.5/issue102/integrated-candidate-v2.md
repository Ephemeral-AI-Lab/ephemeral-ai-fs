# Corrected candidate prerequisite: integrate the 65.96 MB experiment

Tracked in [issue #103](https://github.com/Ephemeral-AI-Lab/layerfs/issues/103).
#103 owns implementation and focused tests; #102 follows with full benchmarks.

The owner corrected the execution order: integrate the selected offline work,
then run benchmarks on that runnable implementation. The schema9 initial2
campaign is diagnostic evidence for the previous product, not qualification of
the intended candidate. Stop further broad schema9 collection. Preserve its
failures and the applicable receipt/verifier/build-custody repairs.

Implementation order:

- [ ] Public compact namespace: scoped serial inode IDs, inline inode values,
      compact directory references, durable allocation and legacy compatibility.
- [ ] Authenticated physical inode-value groups and bounded metadata deltas.
- [ ] Whole-file content prefix graphs and authenticated native chunk slices,
      with all bases/index/pool bytes charged and bounded decoding.
- [ ] Public creation, edits, Commit, reconnect, fork and historical reads use
      the integrated formats; no Python reader or fixture-specific converter
      substituted for the product implementation.
- [ ] Qualify exact bytes, metadata, hardlinks, branching, failure recovery and
      corruption rejection before collecting the full mandatory registry.
- [ ] Measure integrated stride3 first, then full157 and historical_access;
      run the complete mandatory benchmark campaign on the frozen candidate.

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

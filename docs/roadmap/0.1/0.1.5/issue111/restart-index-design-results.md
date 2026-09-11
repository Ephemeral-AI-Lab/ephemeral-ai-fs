# Restartable index design: correctness findings and revised proposal

The direct "save ValueIndex and reopen it" proposal is not ready for a product
prototype under the current guarantees. Schema10 has no durable Store generation
binding a saved index to the current complete catalogue. Native Store tests now
demonstrate that endpoint, last-group identity and data_version can agree while
the correct value-to-ordinal mappings differ.

A better first implementation candidate is **proof-driven avoidance of the global
index for common changed-leaf commits**, using authenticated predecessor ordinals
and exact new-dependency evidence. Two focused native tests validate the required
building blocks and a critical same-operation fallback condition. This candidate
is not wired into production and has no new latency measurement yet.

## Work completed

Three subagent reviews examined Store lifetime/durability, index restart semantics,
and existing authenticated predecessor/dependency mechanisms. Read the actual
schema verification, exclusive lock, scratch ownership, admission writer boundary,
rollback invalidation and metadata DELTA authentication paths.

Created an isolated worktree at fa8f00c62 plus the preserved dirty patch. Added
only two design-proof tests in its existing native metadata test module. Both
passed against actual Store/admission/group-authentication code. No product
behavior changed in main, no performance sample, and no test hook was used to
claim a public-operation speedup.

Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-restart-index-design-evidence/20260911T082028Z`.
The two test sources, full source patch, commands/logs/exits and original dirty
snapshot are retained. One initial command had an incorrect working-directory
path and did not start; the corrected test executions completed successfully.

## Why clean-close caching needs more than a stamp

Current StoreDb opens an exclusive SQLite connection and initializes ValueIndex
to None. The canonical Store uses MEMORY journaling/synchronous OFF; scratch uses
OFF journaling/synchronous OFF. A clean cache snapshot can be made disposable and
crash-tolerant, but that does not establish correspondence to a restored/replaced
Store or identify all indexed values.

`PRAGMA data_version` is connection-relative. The last group digest authenticates
one group, not the full catalogue or index coverage. A checksum of the sidecar
authenticates its own contents, not which Store state those contents describe.
File metadata and header counters are useful rejection checks, not substitutes
for a proven Store/cache generation protocol.

The native counterexample creates two schema10 Stores with different first
metadata values, then appends an identical final value. Both complete catalogues
pass authentication/validation. The observed stamps agree:

```text
next ordinal = 102
last group first ordinal = 101
last group count = 1
last group pack = 3
last group digest/group number = equal
data_version = 1
```

Nevertheless, Store A's index misses a value that Store B correctly maps to
ordinal 1. A positive mapping from A also points at different bytes in B, and
authenticated current-Store equality rejects it. This is a concrete counterexample
to those stamps, not a demonstration that every possible filesystem provenance
scheme is indistinguishable. No timestamp-forging test was performed.

Checking positive sidecar hits on demand can preserve canonical correctness.
It does not prove that negative lookups are complete or that a returned duplicate
ordinal is the existing index's first winner. A stale miss may allocate extra
physical metadata. Rebuilding on every uncertain miss preserves current sharing
but recreates the startup cost for changed values.

A durable index/table updated with the authoritative Store is a different design:
schema10 compares its complete schema against a fixed manifest, so an additional
table/index is a format-contract change. No such change was made or assumed.

## Existing evidence can avoid some need for reconstruction

The tree editor supplies the old leaf's ObjectId through `prior_ids`.
`metadata_predecessor` already returns both its fully authenticated canonical
leaf and reconstructed physical ordinal leaf, after checking all DELTA bases
and intermediate identities. Matching the complete 73-byte target value against
those bytes proves a positive ordinal reuse.

A metadata value includes content_root and metadata_root. If either dependency
was absent at the admission-session baseline and newly admitted in the held
session, an equal metadata value cannot occur in the pre-session catalogue under
the existing dependency-closure invariant. `AdmissionSession.baseline_pack` is
already the exact ownership boundary used by other admission code.

**Important:** this does not prove absence from earlier batches in the same
session. The native proof test demonstrates both parts:

```text
100-value changed leaf:
  99 values resolved from the authenticated predecessor
   1 value has a genuinely new dependency
Before publishing metadata in this session: fresh-negative proof applies
After publishing the first metadata batch: the same value already exists at 101
```

The test retains one real admission session across dependency and metadata
publication, confirms exact canonical readback, then rolls it back and confirms
the original base remains readable. It establishes the building-block proof;
it is not an implementation of the proposed fast path.

## Narrow proposed fast path

1. Keep publication-local pending_values matching first.
2. For a bounded changed leaf with an origin, authenticate its predecessor and
   recover exact value-to-ordinal hints. Match complete values and validate row
   correspondence; never trust a hint alone.
3. Classify unresolved values using exact dependency membership and the held
   session baseline. Use the fresh-negative proof only while no earlier metadata
   group has been published by that session; earlier prepared groups must be
   covered by pending_values. Otherwise retain the normal fallback.
4. Dependencies still in a prevalidated MissingBatch require their own explicit
   pending-closure proof. A missing object-location result alone is insufficient.
   The current test covers an already-published new dependency, not this additional
   pending-dependency case.
5. If every value has a valid positive or negative proof, assign/encode through
   the existing pipeline without constructing ValueIndex. If any value remains
   unknown, use the existing exact index path. Do not mark a partial map as a
   complete synchronized index.
6. Reuse the authenticated predecessor in the existing DELTA search so the
   optimization does not add a second full fetch/decode. Charge its retained bytes,
   maps and lookup vectors to existing physical/read budgets; exhausted budgets
   fall back without suppressing integrity errors.

This has no sidecar validity problem and introduces no persistent index format.
The successful path depends on changed leaves, bounded predecessor/dependency
work and indexed probes. The fallback still has O(history) startup cost: it is
not a universal sublinear-restart guarantee.

## Compatibility boundary and required qualification

An authenticated predecessor ordinal can differ from ValueIndex's first winner
in its 131072-value retained window, particularly after eviction or duplicate
physical values. Exact canonical bytes/authentication remain possible, but the
physical leaf bytes and DELTA choice can differ. A future experiment must declare
that placement difference and measure sharing, encoded/allocated bytes and DELTA
behavior. It must not claim byte-identical physical winner policy without proof.

Required cases before promotion include old-content reversion, repeated values
across two batches, pending dependencies, missing/unpooled origins, leaf split/
merge, corruption of unused base values, duplicate historical ordinals, eviction
boundaries and rollback. Existing read, allocation and work bounds remain.
Then repeat the exact retained/reopened public SDK cohort, with coverage counters
for positive proofs, fresh negatives, unresolved values and full-index fallback.
Plain acceptance remains separate from nonce diagnostics.

The prior measured opportunity remains 259 ms median first Commit after reopen
versus 22 ms retained at 100000 files, largely index reconstruction. It is not a
savings prediction for this unimplemented fast path. Cold Init generally lacks
predecessors, so its separate SQLite execution/cache-working-set problem remains.

## Decision

Do not promote a timestamp/endpoint-based persistent index as an exact authority.
Investigate the guarded predecessor/dependency fast path before adding a new
durable index lifecycle. It uses existing authenticated evidence and an exact
fallback instead of accepting stale negative cache answers. A universal restartable
index preserving identical global winner behavior remains a separate persistence/
format design question.

No new product optimization or latency improvement is claimed here. Main product,
original dirty work and older sealed evidence were preserved. Cold plain <=2.7 s
remains open; no release/tag/deployment. Context: #111/#115/#109/#110/#108/#106/
#102/#104/#100/#107.

Issue outcome: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5631753418

# Storage architecture v2: independent review disposition

Status: **READY FOR OWNER POLICY DISCUSSION / DETAILED DESIGN**, 2026-09-08.
The original review returned NEEDS REVISION. This document records its corrections,
not product qualification. Implementation, migration, new benchmark definitions,
verification planning and execution have not been authorized by this revision.

## Custody and revision scope

The original review used PR #77 at
`41b3143d08c20419879cc10dce66acab5a21e565`; its later merge
`b1ef84913783306267b5d6df6016fb5827bb966b` changed neither the five reviewed design
documents nor product code. This documentation revision starts from merged main
`28177560c8f049c02192e18c263cdc5543c1ab52` in an isolated
`codex/storage-spec-revision` worktree. The older local main checkout and its
unrelated edits were preserved. Source tracing uses the merged product code,
including live FUSE, rather than the older local checkout.

Authoritative revised documents:

- [Scope](README.md)
- [Owner boundary](storage-efficiency-boundary.md)
- [Architecture v2](storage-architecture-spec.md)
- [Proposed SQLite/wire format](sqlite-storage-format.md)
- [Evidence custody](evidence.md)
- [Independent review prompt](design-review-prompt.md)

The 0.1 roadmap still retains its compatibility rule, with a link to the proposed
exception/transition decision. The reconciliation current-model context corrects
the old unconditional active-execution Busy diagram to the implemented distinction
between non-remote and live remote-backed capture. Neither correction authorizes
a different public lifecycle.

Historical DeepSeek reports remain pinned to documentation commit
`1c7c9235115d1b4f21bc2eae7af822552b7be3ed`, with their distinct measured candidate,
interface and environment identities unchanged. The evidence-index addition is
only arithmetic/interpretation. No historical result, experiment definition,
benchmark contract or sample was modified.

## Finding-by-finding closure

“Resolved” means the revised specification assigns a concrete rule and owner;
it does not mean code implements or has verified that rule.

| Original finding | Disposition | Concrete correction |
| --- | --- | --- |
| Packed schema conflicts with unchanged 0.1 schema promise | **Remaining owner decision; design ambiguity resolved** | [Compatibility transition](storage-architecture-spec.md#compatibility-transition): explicit proposed schema 6/wire 1, new-Store-only creation, legacy rejection in packed implementation, no conversion on open; choose narrow 0.1 exception or 0.2 placement and required legacy scope |
| Compression outside transaction still leaves whole-Init operation serialization | **Resolved** | [Admission protocol](storage-architecture-spec.md#admission-protocol) distinguishes permit, connection and transaction; construction/encoding outside both guards, batches interleave using existing FIFO coordination |
| Shortening Init permit makes whole-Store cleanup unsafe | **Resolved** | Delete cleanup helper and all four fallback/publication callers in the future replacement; no trusted all-missing observation after interleaving; retain/account earlier admissions |
| New permit-taking helper can deadlock nested current callers | **Resolved** | Explicit replacement scope includes Init, direct candidate, Workspace candidate and delivery/final-flush callbacks; metadata siblings retain short permits without nesting |
| Locator SQL omits canonical length / root FK destination | **Resolved** | [Format SQL](sqlite-storage-format.md#2-proposed-sqlite-structures) retains logical index name objects, adds canonical_length and keeps root FKs there; rowid only needed for pack BLOB table |
| Reader/collision path may decode or fetch bases while connection is held | **Resolved** | [Read ownership](storage-architecture-spec.md#8-object-reads-and-ownership): close handles and release connection before decode/hash/base use; replace raw transaction-held collision reads |
| Stale prepared batches can overwrite or misclassify equal concurrent admissions | **Resolved** | Monotonic late-duplicate validation/recheck, immutable selected locators, no UPSERT, skip zero-winner packs and count mixed-pack redundant records |
| Init final-tail optimization can bypass new race handling | **Resolved** | Final tail explicitly executes shared preparation/recheck and retains the successful permit into combined final insertion/publication |
| Generic canonical output does not carry prior-file delta hints | **Resolved** | [Hint provenance](storage-architecture-spec.md#hint-provenance): optional already-visited prior extents/pages, no new graph walk; Init supplies no invented predecessor |
| Four candidates do not bound fetch/decode/trial work | **Resolved** | Per-target and per-batch fetch/decode/trial/comparison limits, anchor deduplication, one active base/index and charged buffers |
| Group-optimal delta selection conflicts with one compression pass | **Resolved as explicit approximation** | Bounded greedy COPY/INSERT, raw-record savings threshold, one selected-group compression; possible loss to compressed FULL is disclosed |
| Phase flushes unnecessarily split tiny Commit packing/admission | **Resolved without a fixed speed/count promise** | [Flush rules](storage-architecture-spec.md#7-flush-publication-and-finalization) carry one accumulator only when pending dependencies and memory permit; otherwise flush before dependent read |
| Newly encoded metadata immediately reread in finalization | **Resolved** | Carry authenticated inode-table-root fact in existing checkpoint tied to published root; bounded authenticated read when no matching fact exists; installation/resume preserved |
| Scalar temporary clone / repeated trusted authentication | **Resolved** | Remove temporary cache-argument clone; carry authentication through trusted internal read boundary, retaining untrusted-source checks; no additional cache hierarchy |
| Format fields and oversized roles not concretely bounded | **Resolved** | [Wire framing](sqlite-storage-format.md#3-exact-proposed-pack-framing) specifies fields, ends, counts, opcodes, codecs and raw singleton route; general 16-MiB codec limit is distinct from normal 4-MiB-minus-one admission |
| Compressed encoded length can allocate before a decoded bound protects it | **Resolved** | RAW equality / compressed encoded<=decoded<=64KiB acceptance before fetch; BLOB-wide and singleton classification checked before extraction |
| “Existing operation memory allowance” invents a shared current owner | **Resolved as an explicit proposed ownership refactor** | [Memory ownership](storage-architecture-spec.md#5-grouping-framing-and-memory) names separate existing budgets, inclusive candidate/output allowance, scratch reservation, optional-trial skip and required-codec resource failure |
| Sequential large reads mistaken for small total output memory | **Resolved** | Distinguish extraction scratch from retained Vec outputs; disclose theoretical batch sum and preserve public output semantics |
| Local numeric pack IDs / logical-only export closure | **Resolved at format seam** | [Cloud seams](storage-architecture-spec.md#13-cloud-evolution-seams): portable locator manifest, logical plus physical FULL-base closure, no extra local dependency index |
| Bounded decode presented as low remote latency | **Resolved** | Explicit locator/framing/group and cross-pack base fan-out; selective transfer overfetch/repack and large raw-object cost |
| Missing published v0.3 plan / future reconciliation treated as present | **Resolved as source-status distinction** | #52 supplies future intent, missing document remains a gap; no invented cloud contract or v0.1.4 cloud gate; current HeadMoved/lease behavior preserved |
| Git-parity/speed confidence inferred from design prose | **Deferred measurement** | Historical matched controls remain motivation; no candidate allocation, overhead or Git-parity qualification exists |
| New benchmark family/population, environment, gates, verification plans | **Intentionally deferred** | Separate discussion after specification review; no scenarios, matrices, counts, test plan or execution added |

## Focused independent rereview

Three bounded subagent passes contributed: current caller/admission ownership,
physical format/reader bounds, and compatibility/evidence/cloud custody. The
architecture/lifecycle and evidence/cloud passes independently reread the revised
documents; the format author checked cross-document consistency and the primary
reviewer read the resulting format. This is source/document review, not an
independent implementation proof.

The rereview found the final-Init-tail recheck, pre-allocation encoded-length cap,
explicit scratch budget owner, hint-inspection authentication wording and retained
batch-output memory distinctions. Those were corrected before publication.
Review did not treat deferred measurement/verification planning as an architectural
defect or fill those placeholders. Documentation checks cover whitespace, local
links/anchors, retained checklist/evidence scope and the documentation-only diff.
No product tests or benchmarks were run.

## Remaining policy and disclosed tradeoffs

The owner must choose the compatibility transition/release placement and whether
the recommended new-Store-only implementation is sufficient. Required same-binary
legacy access or separate-destination conversion needs explicit scope. There is
no automatic/in-place migration and no silently granted compatibility exception.

The admission recheck has a finite but quadratic adversarial case: one new equal
object appearing per round repeatedly queries the shrinking missing set. At
8,191 objects that can approach 33.55 million membership-ID probes. The design
prefers immutable facts and releasing Store permits over a reservation service or
holding the permit through conflict decoding. This supports bounded work and
interleaving opportunities, not a tail-latency guarantee. Do not hide it behind
the expected per-agent call rate.

Depth-one anchors, bounded approximate matching, no Init predecessor hints,
partial synchronous packs, persistent per-object indexes and request-local-only
group reuse can leave a material storage/read-cost gap. Actual codec allocator
requirements must fit the named reservation or require a prospective design change;
a 64-KiB window alone does not prove that fit. No silent RAW fallback, extra cache
or higher memory allowance is justified merely to make implementation convenient.

## Confidence assessment

| Dimension | Assessment and reason |
| --- | --- |
| Identity, scope and lifecycle boundaries | **Very high design confidence:** canonical/physical identity, shared paths, synchronous SQLite publication, staging/no-change/head/finalization and excluded cloud/durability work are explicit and source-traced |
| Architectural clarity and internal consistency | **High design confidence:** concrete owners/protocols/framing and cross-document authority replace generic TBDs; focused rereview corrected discovered exceptions; compatibility remains an explicit owner gate |
| Bounded ownership and cost mechanisms | **High design confidence, not a resource qualification:** encoded/decoded/canonical/count/work/resident limits and lock lifetimes are explicit; adversarial retry and public output sums are disclosed; actual allocator/caller integration remains implementation work |
| Minimal final architecture | **High design confidence:** one replacement object admission/access path, existing coordination/spools/caches and explicit deletion scope; no second service, encoder family or small-file backend |
| Cloud evolution seams | **High confidence in absence of the identified format dead ends:** portable IDs/framing, translated manifest and enumerable physical closure; actual cloud topology, authorization and operation costs remain future work |
| Material storage improvement | **Moderate expectation from historical evidence:** compression opportunity is substantial, but the proposed candidate has not been measured |
| Getting close to matched Git allocation | **Not yet measured:** the objective is sufficiently close allocation for a worthwhile storage/latency tradeoff, not exact parity; online shallow similarity and index/anchor costs remain unknown |
| Normal small-edit agent read/write overhead | **Moderately confident design expectation:** exact CAS/COW reuse avoids unchanged work; codec/base work is bounded and outside Store serialization; actual net latency remains unmeasured |
| Aggregate-load capacity | **Uncertain pending workload and service-demand evidence:** multiple projects/agents add CPU, I/O, memory and serialized work; the individual call-rate expectation is not a Store ceiling |
| Particular latency, throughput or resource result | **Not yet measured:** no candidate result establishes a numerical regression, capacity or acceptance claim |

“Not yet measured” does not mean overhead is expected to be unacceptable. The
revised architecture gives high confidence that avoidable serialization is removed
and codec work has explicit bounds. Manageable overhead is a reasonable expectation
for normal small-edit agent workloads, without guaranteeing it. On reads, group
and base decoding add work while compression can reduce physical I/O and the
replacement removes redundant copies/hashes. On writes, unchanged content is reused
while new content pays bounded compression and optional delta preparation. Their
net elapsed cost has not been established. Aggregate capacity depends on the sum
of service demand across calls and Stores, not LLM delay alone.

Likewise, Git parity has neither been demonstrated nor ruled out. The owner's
objective is getting sufficiently close with an acceptable storage/latency tradeoff;
no exact-parity requirement or numerical closeness tolerance is introduced.

The documents now support a much stronger architectural assessment. They do not
justify a very-high empirical rating, implementation approval, or a promised ratio.
The next conversation is the remaining owner policy and, separately,
the deferred evaluation/verification definitions.

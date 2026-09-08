# Storage architecture v3: independent review disposition

Status: **READY FOR OWNER POLICY DISCUSSION / DETAILED DESIGN**, 2026-09-08.
The original review returned NEEDS REVISION. This document records its corrections,
not product qualification. This revision also responds to the independent audit
of head `628fc7191f549a3b0df061ed3b0cb4b72dfbde2e`: its quadratic recheck is rejected,
and scalar batching deficiencies are addressed separately. PR #80 now owns the
agreed development-smoke direction/topology; this specification revision neither
implements/runs those smokes nor authorizes product code or migration.

## Custody and revision scope

The original review used PR #77 at
`41b3143d08c20419879cc10dce66acab5a21e565`; its later merge
`b1ef84913783306267b5d6df6016fb5827bb966b` changed neither the five reviewed design
documents nor product code. This documentation revision starts from merged main
`28177560c8f049c02192e18c263cdc5543c1ab52` in an isolated
`codex/storage-spec-revision` worktree. The older local main checkout and its
unrelated edits were preserved. Source tracing uses the merged product code,
including live FUSE, rather than the older local checkout. Revision v3 builds on
PR #79 head `628fc7191` without rewriting that earlier review history. The subsequent
scope authority is [PR #80's pinned smoke plan](https://github.com/Ephemeral-AI-Lab/layerfs/blob/d9ec9c6714ca31adb7a337d2ac0f40976513908c/docs/roadmap/0.1/0.1.4/storage-smoke-test-plan.md);
linking it changes no frozen experiment or smoke population.

Authoritative revised documents:

- [Scope](README.md)
- [Owner boundary](storage-efficiency-boundary.md)
- [Architecture v3](storage-architecture-spec.md)
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
| Compression outside transaction still leaves whole-Init operation serialization | **Resolved for the specified admission path** | [Admission protocol](storage-architecture-spec.md#admission-protocol) distinguishes permit, connection and transaction; construction/encoding outside both guards, batches interleave using existing FIFO coordination |
| Shortening Init permit makes whole-Store cleanup unsafe | **Resolved** | Delete cleanup helper and all four fallback/publication callers in the future replacement; no trusted all-missing observation after interleaving; retain/account earlier admissions |
| New permit-taking helper can deadlock nested current callers | **Resolved** | Explicit replacement scope includes Init, direct candidate, Workspace candidate and delivery/final-flush callbacks; metadata siblings retain short permits without nesting |
| Locator SQL omits canonical length / root FK destination | **Resolved** | [Format SQL](sqlite-storage-format.md#2-proposed-sqlite-structures) retains logical index name objects, adds canonical_length and keeps root FKs there; rowid only needed for pack BLOB table |
| Reader/collision path may decode or fetch bases while connection is held | **Resolved** | [Read ownership](storage-architecture-spec.md#8-object-reads-and-ownership): close handles and release connection before decode/hash/base use; replace raw transaction-held collision reads |
| Stale prepared batches can overwrite or misclassify equal concurrent admissions | **Resolved** | One initial probe and one final recheck; retain the admission permit through late validation with the connection released; no retry, UPSERT or recompression; count mixed-pack redundant records |
| Init final-tail optimization can bypass new race handling | **Resolved** | Final tail explicitly executes shared preparation/recheck and retains the successful permit into combined final insertion/publication |
| Generic canonical output does not carry prior-file delta hints | **Resolved** | [Hint provenance](storage-architecture-spec.md#hint-provenance-and-producer-handoff): same-inode and pinned-final-path predecessor rules, first-span capture handoff, complete-build propagation and one forward old-extent cursor; Init supplies no invented predecessor |
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
| Development smokes versus full qualification | **Planning reconciled** | PR #80 specifies first FIVE frozen #72 checkpoints plus edit/small-file smokes and host SQLite/SDK + managed Docker/real FUSE; remaining smoke prerequisites and full-family qualification remain open; nothing run here |

## Subsequent complexity and batching audit

The earlier v2 closure overreached on two points. A finite shrinking-set retry still
permitted O(N^2) membership visits, and merely optional whole-file hints did not
establish the producer handoff. These are proposed-design defects, not claims of
observed current-product regressions. The owner rejected worse-than-linear admission
processing; revision v3 changes the mechanisms, not only their confidence labels.

| Audit finding at 628fc7191 | Revised disposition |
| --- | --- |
| N shrinking-set rounds: N(N+1)/2 probes, 33,550,336 at N=8,191 and 266,176 128-ID query pages | **Removed:** at most two membership visits per distinct candidate; one final permit-held recheck followed by connection-unlocked validation, no retry |
| Complete-build fallback drops available same-inode predecessor | **Specified handoff:** preserve physical predecessor in the shared file context through the fallback and payload offset callback |
| Tempfile/rename-over and prebuilt captured output have no same-inode hint | **Specified handoff:** once-per-file frozen final-path lookup against pinned namespace, separate from logical before; first-span facts travel with original private captured output; one old cursor, no re-CDC or alias/chunk cross-product |
| `insert_checked_object_batch` executes one SQL INSERT per object | **Replace:** bounded multirow pack and five-column locator INSERTs; row/byte/parameter limits and indexed work stated |
| Scratch `insert_page`, streaming direct insert and `order_missing` still use scalar SQL | **Replace:** transactional page INSERT/RETURNING, page membership and order restoration; preserve duplicate flags/counts and invalidate operation after active scratch failure |
| IdOrder spill emits one unbuffered read/write per 32-byte ID | **Replace:** bounded buffered/page I/O, explicit flush/seal before read, exact order and truncated-tail detection |
| Object-level loops can repeatedly decode groups or re-encode growing packs | **Replace:** target/base group waves with one parse/decode per group per wave; direct slots, incremental counters and one compression per closed group |
| Earlier blanket smoke/test-environment deferral is stale | **Corrected:** reference PR #80's confirmed plan/topology; keep full qualification separate; no new execution |

### Owner follow-up: all eight items

| Item | Disposition / future replacement boundary |
| --- | --- |
| 1. Quadratic admission/recheck | **Resolved in spec:** <=2U membership visits, one final permit-held decision, no shrinking retry; byte validation and SQLite index costs separate |
| 2. Reconciliation C*F manifest work | **Partially resolved / remaining scope explicit:** one lazy invocation-local invalidation view and exact/ancestor/prefix selection remove per-conflict rebuilds and unrelated scans. Exact v2 fingerprinting of overlapping scopes can still repeat bytes; no all-Commit linearity claim or silently changed validity semantics |
| 3. Nonempty-Store Init parent copying | **Resolved in spec:** all source strategies stream owned finalized output to one physical sink regardless of occupancy; remove finish_parallel_candidate/merge_prevalidated payload replay, preserve hardlinks/seeds and at most one source fallback |
| 4. Scalar Store and scratch SQL | **Resolved in spec:** bounded bulk pack/locator INSERTs; seen and offset-index flush/page transactions and membership; streaming direct callers covered; preserve pending visibility, first-occurrence flags and order. These are not per-object fsync claims |
| 5. Tiny spill I/O | **Resolved in spec:** sealed buffered IdOrder; build disk offset index from the deduplicated known absolute-location union, including converted pending offsets and threshold record; delete tiny header/seek reconstruction in that transition |
| 6. FIFO notify_all fanout | **Resolved in spec:** explicit per-waiter one-shot FIFO handoff inside existing Store gate; normal release notifies one successor, abandonment/failure work amortized once per waiter; no shared-condvar notify_one shortcut |
| 7. Read batching | **Resolved with scope:** one target/base group parse per bounded internal batch/wave; coalesce anchors, count repeats across drains/dependent levels and retained output memory; no persistent cache hierarchy |
| 8. Whole-file hint handoff | **Resolved in spec, implementation coverage unproven:** retained same-inode base and pinned-path replacement predecessor travel through complete/captured producers with per-file first-span facts; one old extent cursor, legitimate absence and explicit budgets, no forced SDK edits |

The only remaining technical scope in this table is repeated exact fingerprint work
for overlapping reconciliation scopes. Its indexed-view correction is selected,
but a global distinct-bytes linear guarantee would require additional fingerprint
representation work not authorized by this spec. Admission itself has no such retry
or conflict-count multiplier. The compatibility/release decision remains separate.

The [complexity contract](storage-architecture-spec.md#complexity-and-batching-contract)
separates candidate visits, bytes, unique groups, Store/scratch index sizes, touched
nodes and retained history. O(N+B+T+G) orchestration excludes explicitly disclosed
indexed-access and required canonical sorting costs. B-trees are not called O(1),
and per-key index updates do not vanish when SQL statements are batched.

## Focused independent rereview

The first revision's independent passes corrected final-Init-tail race handling,
encoded-length allocation, scratch ownership, hint inspection and retained output
memory. The later owner audit reopened the algorithmic recheck and whole-file hint
claims. Read-only follow-up source traces established same-inode truncate preservation,
tempfile replacement behavior, complete-build/captured-output handoff gaps and the
scalar Store/scratch/IdOrder callers, occupancy-split Init ownership, reconciliation
manifest rebuilding and broadcast FIFO wakeups. The v3 rereview assesses the actual replacement
protocol and its work/lock tradeoff, not the rejected finite-cap argument.

Documentation checks cover whitespace, links/anchors, retained checklists/evidence
scope and docs-only changes. No product tests, development smokes or benchmarks are
run by this revision. PR #80's agreed direction is acknowledged without copying or
expanding its campaign.

## Remaining policy and disclosed tradeoffs

The owner must choose the compatibility transition/release placement and whether
the recommended new-Store-only implementation is sufficient. Required same-binary
legacy access or separate-destination conversion needs explicit scope. There is
no automatic/in-place migration and no silently granted compatibility exception.

The new admission permit lasts through **late-duplicate** group/base extraction and
validation plus final insertion. Connection handles are released before decode/hash,
so ordinary reads retain access, while other coordinated writers/publications wait.
This is a deliberate bounded-batch serialization cost in exchange for eliminating
the rejected quadratic retry. There is no permit over source discovery, whole Init
construction, private-spool loading or new representation encoding. Queue delay is
not a claim of linear wall time, high throughput or unlimited aggregate capacity.

Before acquiring that permit, each episode reserves canonical comparison operands,
prepared bytes and two-wave validation scratch. Pack/episode formation is a forward
pass with memory-derived boundaries, not a repeated shrink/rescan workaround. A RAW
singleton can share its canonical backing and use streamed comparison of stored
bytes. No public memory bound is silently raised to fit the protocol.

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
| Bounded ownership and cost mechanisms | **High design confidence, not a resource qualification:** encoded/decoded/canonical/count/work/resident limits and lock lifetimes are explicit; linear recheck visits, internal grouped validation, bulk/page I/O and public output sums are explicit; overlapping reconciliation fingerprints remain scoped separately; actual allocator/caller integration remains implementation work |
| Minimal final architecture | **High design confidence:** one replacement object admission/access path, existing coordination/spools/caches and explicit deletion scope; no second service, encoder family or small-file backend |
| Cloud evolution seams | **High confidence in absence of the identified format dead ends:** portable IDs/framing, translated manifest and enumerable physical closure; actual cloud topology, authorization and operation costs remain future work |
| Material storage improvement | **Moderate expectation from historical evidence:** compression opportunity is substantial, but the proposed candidate has not been measured |
| Getting close to matched Git allocation | **Not yet measured:** the objective is sufficiently close allocation for a worthwhile storage/latency tradeoff, not exact parity; online shallow similarity and index/anchor costs remain unknown |
| Normal small-edit agent read/write overhead | **Moderately confident design expectation:** exact CAS/COW reuse avoids unchanged work; new encoding/hint work stays outside Store serialization; late-duplicate validation retains only the admission permit, with explicit writer queue cost; actual latency remains unmeasured |
| Aggregate-load capacity | **Uncertain pending workload and service-demand evidence:** multiple projects/agents add CPU, I/O, memory and serialized work; the individual call-rate expectation is not a Store ceiling |
| Particular latency, throughput or resource result | **Not yet measured:** no candidate result establishes a numerical regression, capacity or acceptance claim |

“Not yet measured” does not mean overhead is expected to be unacceptable. The
revised protocol removes the identified quadratic recheck and bounds byte work;
it intentionally serializes late-duplicate validation under the admission permit.
This is not a claim that all avoidable lifecycle overhead has been eliminated. Manageable overhead is a reasonable expectation
for normal small-edit agent workloads, without guaranteeing it. This does not
extend a linear admission bound to existing rename work or overlapping reconciliation
fingerprints; the remaining scope is explicitly retained above. On reads, group
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
PR #80's remaining smoke prerequisites and the separate full qualification definitions.

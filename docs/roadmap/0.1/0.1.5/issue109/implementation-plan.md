# Namespace Init optimization implementation plan (#109 / #108)

**Status: plan only; implementation and performance experiments are not approved
by this planning request.** This document combines the
[#109 findings](findings.md) with
[the #108 dedup-history diagnostic](https://github.com/Ephemeral-AI-Lab/layerfs/issues/108#issuecomment-5623008277).
Its implementation sequence and measurements are proposed future work, not
completed fixes, observed speedups or a currently executing campaign.

## Objective and evidence

Optimize the full `init_namespace / namespace-100000` case: 100,000 files,
1,000 data directories and 500,000,000 bytes. Primary target: public Init
`layerstack_init_ns` <=2.7 s, preferably approximately 2.60 s. Preserve the
hybrid SmallContent FULL/DELTA and large-file CDC/CAS routes, packing,
compression, metadata pooling, authentication, bounded resources and ordinary
uncompacted operation. Exact CAS applies to both content routes.

The 98,998 non-empty files below 128 KiB hold 300 MB; the two 100-MB files hold
200 MB; 1,000 files are empty. The 5,000 medium files carry 90.13% of SmallContent
bytes. This stresses both content scanning and per-file/metadata admission.

Unprofiled, nonce-enabled diagnostic observations: pipeline 3.449673500 s, final root/inode
construction 1.185147250 s, total Init 4.702218500 s. The corresponding historical
product diagnostic was 2.537925917 s; the original published v0.1.3 sample was
2.603162083 s. These historical observations are unpaired, n=1, with different
harness custody. They explain the investigation, not a future candidate speedup.

## What transfers from #108, and what does not

| Finding | Relationship to Init | Planning consequence |
| --- | --- | --- |
| Metadata lookup and admission prepare identical SQL repeatedly. | Shared `StoreDb` metadata reads, `ValueIndex` and publication code serve both workloads. | Borrow the existing `prepare_cached` pattern; fix the shared functions, not a family runner. |
| Metadata predecessors cause repeated group fetches and pool expansion as history grows. | The read helpers overlap, but fresh Init does not start with the same version-history chain. | Statement reuse can reduce parsing per fetch; it does not reduce chain depth, fetch count or decoded bytes. Consider a read cache only after separate evidence. |
| #108 object-admission growth is predominantly reads; Store publication writes were flat. | #109 final-tree work also includes a separate scratch-index writer. | These findings are compatible. Keep authoritative Store commits, scratch commits, SQL preparation and read amplification distinct. |
| Checkpoint, pause, resume and SDK edit wait on daemon round trips. | Direct Init's public-call timer does not contain the same repeated edit/Commit protocol. | Do not make daemon/protocol changes to solve this Init problem or claim their costs transfer. |
| #108 measured codec work around 65 microseconds/commit, with flat matching cost. | Init processes 300 MB of SmallContent and has a different cost distribution. | Do not assume codecs dominate both. Retain packing/compression and measure encoding benefit before proposing codec policy changes. |
| #109 computes SmallContent search signatures again during FULL publication. | Shared admission code also serves other operations, when those paths execute. | Reuse the computed signature; expect benefit only for callers doing that repeated work. |

A shared implementation applies wherever those functions are called. It does
not prove a latency benefit for every caller or the same percentage gain. Demonstrate transfer on named cases and report unaffected or slower cases.

## Evidence gates before each optimization zone

Reviewed independently for evidence, shared-code correctness and experiment
design. **Evidence sufficient to begin a change is not evidence that the change
is fast or correct.** A/B still need a current-source control, focused checks
and post-change measurements when execution is requested. No new run is needed
merely to rediscover an unchanged mechanism already established below.

### Evidence available for reuse

| ID | Retained evidence | What it establishes / what it does not |
| --- | --- | --- |
| E1 | [#109 findings and source links](findings.md#1-pipeline-smallcontent-signatures-scanned-twice); profile `profile-corrected-20260910T173335Z/sample.txt`, lines 39 and 714 | Two signature scans over the same bytes; publication scan 686/2,903 pipeline samples. Not expected seconds saved. |
| E2 | [Final-inode profile and source](findings.md#2-final-inode-construction-derived-metadata-index-sql-work); same profile from line 1222 | `ValueIndex` sync/find are hot, identical SQL is prepared repeatedly; 791/1,030 compact-inode samples include required execution/I/O. |
| E3 | [#108 diagnostic](https://github.com/Ephemeral-AI-Lab/layerfs/issues/108#issuecomment-5623008277) | Shared SQL parsing, history read amplification and daemon waits in the named history case. Not proof of the same depth/wait costs in fresh Init. |
| E4 | [Measurements and fixture inventory](measurements.json), with raw paths and hashes | Exact fixture, clean/unprofiled versus nonce/profile scopes, identities and existing counter limitations. Not a fresh matched performance control. |
| E5 | Current caller and test audit described in A/B below | Ownership, locking, fallback, chronology and existing behavioral coverage. Recheck relevant code when the implementation source is frozen; documentation-only drift does not require historical replay. |

The profile path is under
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue109-evidence/`.

### Zone entry decisions

| Zone | Evidence already sufficient | Required before code changes | New pre-change experiment? | Required after a change |
| --- | --- | --- | --- | --- |
| A: signature reuse | E1 confirms duplicate work and material profile presence. | Audit every prepared-object constructor, usable-predecessor/FULL fallback, winner gating and actual memory capacity (E5). | **No** new broad profile, historical replay or signature microbenchmark. | Existing selection/rollback tests, the one missing fallback test, memory accounting check, isolated clean C/A comparison. |
| B-core: ValueIndex INSERT/SELECT | E2 confirms repeated preparation in Init's hot path. | Scope statement/transaction lifetimes; retain ordinal, eviction and rollback semantics (E5). | **No** new SQL microbenchmark or baseline profile. | Existing metadata tests and isolated clean C/B comparison. |
| B-shared: locator/ordinal/publication SQL | E3 plus source establishes stable repeated SQL and shared callers. Individual MAX-query impact is not independently measured. | Verify each listed site is still uncached and used; scope non-reentrant connection guards safely. | **No** extra profile to prove identical SQL. | Existing metadata/read/publication checks; report B-bundle savings only. Named history transfer is separate evidence. |
| B-query: eager ordinal fallback | Source proves `unwrap_or` eagerly executes the fallback. | Keep `index.sync` chronology validation; skip only the redundant second query when `next` is supplied. | **No** experiment to prove language evaluation; do not add a query-count framework. | Existing multi-pack sharing/rollback checks; include in B attribution, not a standalone speedup claim. |
| Scratch transaction grouping | E2 proves scratch commits/writes existed before B. Their remaining cost is unknown. | Inspect post-B/AB residual evidence and design cursor/entries/error behavior under journal OFF. | **Conditional:** reuse sufficient residual evidence; otherwise collect groups and scratch commit count/time before choosing transaction extent. | Separate grouping candidate, failure/eviction/invalidation checks and matched timing. |
| Codec / DELTA policy | E4 establishes corpus composition, not encoding benefit. E3's codec cost describes a different workload. | Obtain actual Init selections, trials, time, savings and Store size with correct scope. | **Yes if those facts are absent:** a targeted diagnostic, before any policy change. | Matched time/space/resource comparison and format/selection/authentication checks. |
| Read / decoded-group / codec-state reuse | E3 proves history amplification; Init applicability is not established. | Identify repeated work escaping existing caches and define identity, authentication, invalidation and memory bounds. | **Conditional:** existing residual receipts/profile may suffice; otherwise measure repeated fetch/decode/setup and their cost first. | Separate bounded candidate and tests for the exact affected read/admission paths. |
| Parallelism | E4 shows a busy consumer and producer backpressure before serial fixes. | Inspect residual idle/blocked time, CPU and memory after A/B. | **Conditional:** reuse sufficient residual clocks; otherwise one targeted diagnostic. No worker sweep now. | Separate worker treatment only if supported; latency plus CPU/memory/queue evidence. |

Deferred-zone gate: **a missed 2.7-second target alone is not permission to
implement the next item.** Begin a deferred zone only when current-candidate
evidence identifies its cost as material and the proposal addresses that cost.
A single existing residual diagnostic may satisfy multiple zone gates. If the
evidence is absent, collect only the missing facts with a bounded protocol. If
it contradicts the hypothesis, leave that zone deferred rather than trying it.

## Proposed implementation sequence

### A. Reuse SmallContent signatures

Issue: `prepare_small` computes a content signature for candidate lookup, then
FULL-winner publication recomputes it. The second scan accounted for 686/2,903
pipeline main-thread samples (23.6%); this is not a predicted wall-time saving.

Proposed change in `crates/layerfs-layerstack-store/src/objects/admission.rs`:

- Carry the fixed 64-byte signature with the bounded prepared object.
- Reuse it only when an actual selected FULL winner enters the candidate index.
- Compute lazily at publication if explicit-predecessor handling did not require
  a search signature during preparation.
- Preserve candidate lookup time/order, eviction, winner selection, late CAS,
  rollback, retained handoff and prepared-only invisibility.
- Include actual struct/Option/vector capacity in existing memory ledgers and
  the mirrored prepared-object layout check; retain no extra payload copy.

Before editing, check every `PreparedObject` constructor and the non-final-batch,
SmallContent pack-role and late-CAS winner gates. A usable explicit predecessor
can skip search-signature calculation; if its DELTA trial loses and FULL wins,
publication still must compute the signature lazily. An unusable predecessor
already falls through to ordinary search.

Memory check: `Option<[u64; 8]>` may exceed 64 bytes with discriminant/alignment.
Adding it to the common struct increases **every allocated object slot**, even
native/metadata slots containing `None`, up to the 512-object batch bound.
Existing `size_of::<PreparedObject>()` charges adapt in several paths, but the
SmallContent preparation/assembly preflight must explicitly include actual
prepared-vector capacity; do not assume all stages already charge it. Preserve
limits and the mirrored layout assertion. No per-object heap allocation or
additional payload copy is needed.

Existing coverage to reuse in `objects/admission/`:

- `selected_small_candidate_reuse_late_cas_and_rollback`;
- `selected_small_candidate_retained_handoff_rollback_and_cold_reopen`;
- `selected_small_candidate_fingerprint_bounds` and
  `selected_small_candidate_compact_references_and_eviction`;
- `small_chain_depth_closure_integrity_and_exact_reuse`;
- `native_admission_peak_reservations_reject_unowned_buffers` and bounded
  collision-wave checks for the affected memory paths.

The one missing behavior check: a usable explicit predecessor yields a FULL
winner, which is published in a non-final batch; a later no-predecessor
near-match can discover that winner. This covers lazy signature fallback.
Extend existing reservation coverage only if needed for the newly retained
capacity. Do not add global counters merely to mirror the implementation.
No additional exploratory experiment is required before A.

### B. Reuse SQL statements in shared metadata/admission paths

Borrow #108's shared overlap and extend #109's statement-reuse plan.
Distinguish the Init-hot **B-core** from **B-shared** stable queries and the
**B-query** eager fallback in the evidence table above. They may remain one
coherent B arm; do not automatically create three more experiments. Attribute
savings to B as a bundle unless a later, separately declared contrast isolates
a site. Reuse compiled statements, never query results or unauthenticated data.
Statement reuse leaves fetch/decode counts and chain depth unchanged; conditional
fallback evaluation removes a query. These are different mechanisms.

| Shared location | Proposed change | Expected scope |
| --- | --- | --- |
| `objects/metadata.rs`: `metadata_group` | Cache the group-locator SELECT. | Pool expansion and validation during reads, historical access and admission. |
| `objects/metadata.rs`: `next_metadata_ordinal` | Cache the ordinal-boundary SELECT. | Metadata synchronization and admission. |
| `objects/metadata.rs`: `ValueIndex::find` | Cache the per-value SELECT. | Init and other metadata-pooling publications. |
| `objects/metadata.rs`: `ValueIndex::sync` | Prepare INSERT once per existing transaction, reusing it for all values. | Derived metadata-index population for all callers. |
| `objects/admission.rs`: `insert` | Cache stable MAX queries and metadata-catalogue INSERTs. | Shared physical publication. |
| `objects/admission/metadata_values.rs`: `prepare_values` | Replace eager `next.unwrap_or(db.next_metadata_ordinal()?)` evaluation with a conditional fallback. | Avoid an unnecessary ordinal query when the caller already supplies it; retain `index.sync` chronology checks. |

Audit each shared caller and existing cached-statement usage before editing.
Keep SQL text/results, parameter binding, first-ordinal deduplication,
authentication, transaction boundaries, failure injection and publication order.
The scratch database's 4-MiB cache / 32-MiB file limits and index eviction bounds
remain unchanged. Query results must still reflect inserts and rollback.

Connection-lifetime check: `StoreDb::reader()` and `writer()` share one
non-reentrant `Mutex<Connection>`. A cached statement borrows that guard;
materialize a result and drop statement/guard before calling another `db.*`
helper that acquires it. Scope ValueIndex's INSERT statement so it is dropped
before `transaction.commit()`. Keep existing per-group transactions in B.

Reuse `metadata_values_share_across_prepared_packs_and_reopen`,
`metadata_pool_catalogue_corruption_and_publication_rollback`, and metadata
chain/unused-base authentication tests. The catalogue/rollback test already
warms the index, rolls back, checks invalidation and ordinal reuse, then
corrupts/restores/deletes catalogue rows and checks subsequent reads. Do not
invent a generic cached-SELECT freshness test duplicating that coverage.
Preserve failpoint statement numbering. Extra tests are justified only for an
actual uncovered behavior introduced by the final diff.

### AB. Combine validated changes, then follow remaining cost

Measure A and B independently before combining. Do not attribute their joint
result entirely to either mechanism. Keep a candidate only when its measured
benefit and storage/memory tradeoffs justify it.

```text
Confirm execution request; freeze current source and protocol
  |
  +--> C versus A: signature reuse only ------> verify + measure
  |
  +--> C versus B: shared SQL reuse only -----> verify + measure
                            |
                            v
                    combine validated A + B
                            |
                            v
                  measure remaining Init cost
                            |
               +------------+-------------+
               |                          |
         target reached             target still missed
               |                          |
     qualify Init tiers       investigate the remaining dominant cost;
     and shared callers       keep unresolved target explicit
```

## Measurement availability: avoid inventing phase evidence

| Source | Available information | Limit |
| --- | --- | --- |
| Plain Init receipt | Public Init, setup/teardown, process resources and canonical counts | Pipeline/final-tree phase clocks are not emitted by plain acceptance runs. |
| Existing nonce-enabled Init diagnostic | Pipeline, final-tree, consumer idle, producer blocked and Store commit clocks | Diagnostic overhead; keep separate from plain results. Store commit clocks exclude scratch-index commits. |
| `PhysicalStorageReceipt` in product telemetry | Metadata index sync, fetch/decode, trial/selection and encoding counters | Current Init JSON does not expose this receipt. Sync time is not SQL-prepare time; aggregate encoding time is not SmallContent-only time. |
| Existing macOS profile | Function-stack attribution | Sample shares are not elapsed times or removable percentages. |

Keep isolated A/B acceptance based on clean public operation timing and focused
mechanism/correctness evidence. The proposed nonce C/AB pair explains combined
residual cost only; it cannot establish A-only pipeline or B-only final-tree
deltas. If isolated phase attribution becomes a required question, prospectively
declare that exact diagnostic contrast instead of profiling every arm by default.

Before adding instrumentation, inspect retained outputs and existing telemetry.
If a deferred zone needs a missing `PhysicalStorageReceipt`, expose only a
diagnostic receipt delta around the exact Init call, with reporting outside its
timer and the same diagnostic harness on compared arms. A new diagnostic-only
harness must not be silently mixed with clean qualification. Scratch commit
count/time is a separate instrumentation gap. Raw-zero `sql_prepare_ns` and
`sql_bind_step_returning_ns` in archived v0.1.5 output remain unavailable (`null`
with reason), not zero SQL cost. No new telemetry framework is proposed.

The target gap from the prior instrumented observation is 2.002218500 s
(42.58%). This arithmetic does not predict A+B savings or replace a fresh C.

## Proposed measurement and qualification protocol

This protocol is a proposal to freeze against the then-current source when
implementation is requested. Do not treat previous exploratory build artifacts
as an automatically qualified performance control.

1. Inspect HEAD, dirty work, active processes, free disk and the runner lock.
   Preserve unrelated work. Resolve the compaction-removal source state and
   record it identically across C/A/B/AB; do not attribute its effects to A/B.
2. Build lazily: archive C, then each isolated candidate when its zone is
   entered. Build AB only after the isolated decisions. Reuse qualified builds
   by actual compatible inputs; build matching Linux images when transfer or
   qualification first requires them. Preserve Rust 1.85.1, release flags,
   linked schema/format checks, source-isolated outputs and real identities.
3. Validate/seal the immutable 100,000-file fixture once per acquisition/run,
   preserving generator, byte/metadata digest and cache-profile identity; do
   not rehash all input per sample. Use independent empty Stores for Init.
   Record ordinary per-initialization identities/seeds separately; do not
   silently activate a diagnostic seed override or require equal binary Store
   hashes when normal runtime identities differ.
4. Proposed clean blocks: C,A,A,C; C,B,B,C; C,AB,AB,C. Within each block the
   adjacent comparisons are C1/X1 and X2/C2 (X=A, B or AB); reductions are
   C1-X1 and C2-X2. Keep blocks separate;
   never pool six C observations against one candidate's two. Preserve all
   raw results, n=2 per arm, median, min/max, both paired differences and
   `(median(C)-median(X))/median(C)*100`. This is bounded descriptive evidence,
   not statistical proof. Freeze source, order and preconditioning before runs.
5. Proposed screening margin: 5,000,000 ns, an engineering materiality floor,
   **not** a measured host-noise estimate. To label an isolated Init gain
   consistent, require both paired reductions to exceed
   `max(5,000,000 ns, max(C)-min(C))`, with required correctness/resource checks
   passing. Mixed signs or gains within control variation are inconclusive;
   report that status, not a favorable selected sample. Keep a known regression
   out of the combined candidate. A useful change on another family may be
   reported separately without calling it an Init gain.
6. Proposed final Init goal gate: every declared plain final **candidate**
   acceptance observation (AB, or the retained final candidate) for the 500-MB
   case must be <=2,700,000,000 ns; report median/range
   separately. Any valid miss remains visible. Nonce/profile observations do
   not determine this gate. The matched 500-MB result does not imply a speedup
   at smaller tiers or in every family.
7. Freeze timeout/invalid-run rules before entering a zone. Propose a
   60-second external process watchdog for direct Init diagnostics; for registered
   collection propose the existing 600-second setup,
   300-second product and 310-second command allowances; independent selected
   verification keeps its 45-second work / 59-second hard limit. These do not
   relax the 2.7-second objective or historical 15-second targets. A product
   error/valid timeout is a failed candidate. For demonstrated infrastructure
   invalidity only, retain the attempt and allow one replacement of the whole
   affected pair/cell; a recurring failure leaves it blocked. Never rerun only
   the slower arm. One bounded follow-up may be proposed for a named ambiguity,
   with its protocol frozen first; no automatic repeat-until-fast loop.
8. Check combined transfer on the full `dedup-history-distributed-500`,
   `dedup-history-recurring-500`, and `dedup-history-unrelated-500-mixed-v2`
   cases: proposed C,AB,AB,C per case, seed 1, plus AB independent proof.
   This demonstrates only combined transfer on those named cases, not SQL-only
   causality. Report commit/edit phases, actual group/blob/decode counters and
   remaining daemon costs separately. Reuse the same qualified pristine Store
   artifact through fresh writable clones. Compare cache compatibility and
   artifact/digest identity; source-bound selection IDs may differ legitimately.
   Unknown preparation compatibility makes a paired claim invalid.
9. Before terminal qualification, freeze a final-diff-to-affected-family/member/
   verifier matrix, following benchmark rules sections 7 and 15. Minimum named
   coverage is all four Init tiers and the three history cases, but it is not
   automatically complete shared-path qualification. Enumerate every actually
   affected member/proof once, including relevant read/reuse regression guards;
   preserve demonstrably unaffected #104 evidence. Missing matrix/coverage means
   selected diagnostics only and pending final qualification.
10. Reuse exact-candidate results within their valid scope. AB-only checks at
    100/5 MB, 1,000/20 MB and 10,000/300 MB establish threshold/correctness
    coverage, not comparative gains. If the final 500-MB comparison uses the
    registered qualification command with exact terminal custody, reuse its
    AB rows and run only missing tiers/proofs. Lightweight direct diagnostic
    rows cannot replace registered qualification. Reuse already completed
    exact-candidate transfer proofs rather than rerunning them for a report.

After A/B, inspect the residual evidence once. A safe partial improvement may
still receive its required affected-path verification and be reported with the
goal missed and #109 open. Do not enter deferred zones merely to fill a tuning
checklist. A correctness, resource, custody or valid timeout failure blocks
promotion regardless of latency. No fresh historical replay, fixture generation,
broad pre-A/B profile, mandatory A/A campaign or full #104 restart is needed when
existing evidence and compatibility remain applicable.

Use the existing runner-owned measurement lock with no overlapping sensitive
work or double-acquisition in children. macOS owns SQLite/SDK/coordinator and
publication; Docker owns daemon/FUSE/workload (2 CPUs, 2 GiB). Existing collection
allowances may be used, but historical 15-second targets and the Init 2.7-second
objective are not relaxed. Preserve workload, fixture, verifier and compiler
contracts. Do not replay #104 merely because documentation or source seals change.

## Deliverables and completion boundary

When execution is requested, deliver shared product changes and focused tests,
all attempt logs and custody, isolated and combined timing tables, phase/read
counter comparisons, Store size/memory tradeoffs, Init-tier and transfer proof
coverage, and remaining blockers. Update #109 and coordinate with #108/#106/#102.
Keep #109 open if its target or required qualification remains incomplete.

Planning correction: an earlier interpretation started control builds and one
signature-only build before the owner clarified plan-only scope. The temporary
product patch was restored, the active benchmark binary was restored to the
unchanged-product control, and existing dirty files were checked unchanged.
No Init/dedup performance comparison ran. Those build artifacts/logs remain at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue109-evidence/implementation-20260910T180813Z`
for audit only; they are not optimization results. No experimental product edit
is retained by this plan.

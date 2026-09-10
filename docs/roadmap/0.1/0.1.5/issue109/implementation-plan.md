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

Clean instrumented observations: pipeline 3.449673500 s, final root/inode
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

Generic implementation means a shared function benefits every applicable caller.
It does not mean every family must become faster or receive the same percentage
gain. Demonstrate transfer on named cases and report unaffected or slower cases.

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

Use existing SmallContent candidate tests and add only missing fallback/reuse
coverage. This candidate is ready to implement once execution is requested;
no additional broad exploratory profile is needed first.

### B. Reuse SQL statements in shared metadata/admission paths

Borrow #108's strongest low-risk overlap and extend #109's statement-reuse plan.
Reuse compiled statements, never cached query results or unauthenticated data.

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

Existing metadata pool sharing, chain authentication, chronology, reopen and
rollback/invalidation tests cover the main behavior. Add only a missing
cached-query freshness regression if those tests do not exercise it.

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

## What is unclear and which experiments answer it

| Open question | Required evidence | Decision enabled |
| --- | --- | --- |
| Actual time saved by signature reuse | C/A clean Init and pipeline clocks, equivalent reuse evidence, winner behavior and memory checks. | Retain A or investigate why removing the scan did not help. |
| SQL preparation versus necessary SQLite execution/I/O | C/B final-tree and total clocks; metadata sync/lookup cost; residual profile only if needed. | Retain B and identify whether scratch commits are still material. |
| Does generic SQL reuse improve dedup history? | Matched C/AB runs of the three #108 tier-500 cases; commit/edit phases and physical read counters. | Establish measured transfer without assuming fewer group fetches or daemon calls. |
| Is scratch transaction grouping worthwhile and safe? | Remaining scratch commit count/time after B, then a separately proposed grouping experiment with error/cursor-state checks. | Decide whether to extend scope beyond statement reuse. |
| Is compression/delta trial work worthwhile on this corpus? | FULL/DELTA selections, successful trials, codec time, encoded savings and complete allocated Store size. | Propose a bounded encoding optimization only if justified. |
| Is a decoded-group cache justified? | Residual repeated group/chain fetches, decode time, authentication needs and bounded memory accounting. | Separate read-cache proposal; not part of A/B. |
| Would more parallelism help? | Residual consumer idle, producer blocked time, CPU and memory after serial reductions. | Separate worker-count experiment only if evidence supports it. |
| Can combined changes reach <=2.7 s? | Fresh matched C/AB operation results, median/range and correctness/resource/storage checks. | Claim success only after measurement and required qualification. |

Reaching 2.7 s from the existing 4.702218500-second instrumented observation
requires 2.002218500 s (42.58%) less latency. This is target-gap arithmetic.
Stack shares cannot establish that A+B will remove that amount, and the
historical v0.1.3 executable is not a substitute for a fresh current-source C.

## Proposed measurement and qualification protocol

This protocol is a proposal to freeze against the then-current source when
implementation is requested. Do not treat previous exploratory build artifacts
as an automatically qualified performance control.

1. Inspect HEAD, dirty work, active processes, free disk and the runner lock.
   Preserve unrelated work. Resolve the compaction-removal source state and
   record it identically across C/A/B/AB; do not attribute its effects to A/B.
2. Build and archive C, A, B and AB with existing qualified helpers and separate
   native-input outputs. Preserve Rust 1.85.1, release flags, linked schema/format
   checks and real binary identities. Build matching Linux images for C and AB;
   never relabel an incompatible image or share incompatible mutable targets.
3. Use the existing verified 100,000-file fixture with fresh independent Stores.
   Record its digest and reused/uncontrolled OS-cache state. Proposed clean
   order: C,A,A,C; C,B,B,C; C,AB,AB,C, two observations per candidate and paired
   control. Freeze repetition/order/preconditioning before running; retain every
   attempt and do not choose the best result. One separate instrumented C/AB
   pair may explain phases; do not pool it with clean observations.
4. Report public `layerstack_init_ns`, external command wall and preparation
   separately, plus canonical counts, allocated/apparent Store bytes, CPU/RSS,
   output/source identities and correctness. Proposed n=2 is bounded diagnostic
   confirmation, not statistical proof. Retain a missed <=2.7-second target.
5. Check generic transfer using the registered full `dedup-history-distributed-500`,
   `dedup-history-recurring-500`, and `dedup-history-unrelated-500-mixed-v2` cases:
   proposed C,AB,AB,C per case, seed 1, plus AB independent proof per case.
   Compare actual prepared-input compatibility keys and artifact/digest identity;
   source-bound selection IDs may legitimately differ because their recipes
   include the source seal. Do not confuse those IDs with different input bytes.
6. Qualify AB once on all four registered Init tiers (100/5 MB, 1,000/20 MB,
   10,000/300 MB, 100,000/500 MB), using existing performance selections and
   independent proofs. Add shared-caller coverage justified by the final diff.
   Preserve existing payload/read and reuse improvements as regression guards
   when the touched paths apply; publish exact coverage rather than claiming
   universal qualification from the selected cases.

Use the existing runner-owned measurement lock with no overlapping sensitive
work or double-acquisition in children. macOS owns SQLite/SDK/coordinator and
publication; Docker owns daemon/FUSE/workload (2 CPUs, 2 GiB). Existing collection
allowances may be used, but historical 15-second targets and the Init2.7-second
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

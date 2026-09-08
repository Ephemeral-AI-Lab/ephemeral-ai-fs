# Correspondence observations without changing correspondence policy

**The current cursor already exposes enough state to distinguish its stopping
reasons without editing its traversal algorithm.** Capture the first rejection
inside the existing reservation callback, compare `cursor.counters()` before and
after the existing `hints` call, and retain the first exhaustion reason for later
calls on that cursor. A partial hint result is not a terminal budget failure:
the existing matcher can still produce an admitted DELTA from those hints.

This is a prospective instrumentation design at source checkpoint
`363324b3c2ffc12452f6910d5693d4fffa655c74`. No product files, retained evidence or
Store were changed. The standalone probe imports the actual current cursor and
reuses the pinned previous synthetic fixture. It does not replay the workload or
encode candidate alternatives. Source research is bounded to delivery, cursor,
private spill, and admission interfaces.

## Minimal observation contract

For an occurrence whose authenticated object is about to leave deferred delivery,
preserve the existing `has_predecessor`, `first_span` and four `prior_ids` exactly.
Add a diagnostic tag describing only work that actually occurred:

| Field | Type / units | Source owner and meaning |
|---|---|---|
|`stop`|enum: unobserved, complete, memory_limit, file_limit, operation_limit, descriptor_limit|`objects.rs:2037–2064`, recorded after actual `hints` returns; complete means traversal did not exhaust, not that the whole file was traversed|
|`inherited_exhaustion`|Boolean|Whether `cursor.counters().1` was already nonzero before this call; a later span inherits the sticky original reason, without attempting another reservation|
|`has_predecessor`|existing Boolean|`objects.rs:2036`; do not derive this from hint count|
|`span_present`|derived existing Boolean|`first_span.is_some()`; no raw offsets exported in timed logs|
|`hint_count`|integer count0–4|Count of existing nonempty `prior_ids`; no extra base lookup, candidate generation or overlap scan|
|`triggered_reservation_grants`|integer count0–7, explicit u8|Number of successful existing reservation callbacks triggered by this occurrence's `hints` call; inherited exhaustion and no-request occurrences use0. Reserved work bytes are grants×131,136, not target bytes|
|`canonical_bytes`|integer bytes|Authenticated full canonical object length; project onto the eligible population after initial CAS filtering, not at raw file-cursor creation|

`unobserved` is valid before optional correspondence or when no predecessor/span
request exists. For an eligible file-content payload with both predecessor and
span present, surviving `unobserved` is an instrumentation/handoff error, not
completed no-overlap. A chunk role alone does not prove file-span provenance:
metadata-value chunks need explicit supported source handling. At initial CAS
absence the parent diagnostic validates eligibility/provenance and counts that
unique representative exactly once. Occurrence counters collected before CAS use
a separate denominator and cannot be added to this target population.

The mutually exclusive **post-CAS cursor buckets** are:

1. `no_predecessor` (regardless of absent span).
2. `missing_required_span` (predecessor present, required file span absent).
3. `complete_empty` (predecessor/span present, complete,0 hints).
4. `limited_empty` (predecessor/span present, a recorded limit,0 hints).
5. `complete_with_hints` (complete,1–4 hints).
6. `limited_with_hints` (a recorded limit,1–4 hints).

Each bucket carries **unique target count and full canonical bytes**, with the
same fixed canonical-size histograms as the outer diagnostic. Buckets1–4 identify
the correspondence-level terminal reason unless final admission selected an
already existing representation. Buckets5–6 proceed unchanged to prior/base,
matching, mixed-choice and final-admission outcomes. `limited_with_hints` is a
side label on any later terminal outcome, including DELTA admitted. Assigning all
limited observations to a budget-loss terminal would falsely count successful
DELTAs as missed opportunities.

A small aggregate cross-tab `stop × inherited_exhaustion × hint_count` can supply
counts and canonical bytes for the same unique eligible population. It is a
partition, not an additional population. Cursor/denial events are recorded
separately as counts and reserved bytes; they have no target-byte interpretation.

## Exact reason detection and collision precedence

The source has only three writes setting `exhausted=true`: the4096-descriptor
guard and two failed metadata-reservation sites. The wrapper can therefore use:

```text
before = cursor.counters().exhausted
denied = none
hints = cursor.hints(... existing callback, with diagnostic denial capture ...)?
after = cursor.counters().exhausted
if !before && after:
    sticky = denied if present else descriptor_limit
observation = (hints unchanged, sticky if after else complete, inherited=before)
```

Preserve the original callback's short-circuit order exactly:

- `!available` → `memory_limit`; do not attempt file/global accounting afterward.
- Otherwise the next131,136 bytes exceeding1 MiB/file → `file_limit`.
- Otherwise the existing single `fetch_update` fails16 MiB/operation admission →
  `operation_limit`; preserve its atomic ordering and checked arithmetic.
- Otherwise reserve the same bytes and return true.

When memory, file and operation limits coincide, record the first actual rejecting
guard: memory precedes file, file precedes operation. Do not inspect later guards
merely to claim multiple causes. An arithmetic overflow of the operation counter
is rejected by the same checked-add callback; under the invariant that the counter
starts0 and never exceeds16 MiB it cannot occur, and a violated counter invariant
is an integrity/diagnostic failure rather than normal workload exhaustion.

An already exhausted cursor returns empty hints without invoking the callback.
Its cause must come from the sticky per-cursor reason. Inferring descriptor limit
from every empty callback history would mislabel all those later spans.

Descriptor exhaustion can occur after one or more usable IDs were found in the
current call. Reservation failure can likewise occur when moving to the next
leaf after finding a first overlapping ID. The probe reproduces both partial
cases. Exactly four hints does not prove there were only four overlapping IDs:
the current policy returns the first four unique IDs and keeps advancing. Do not
invent an exact distinct-overlap count or rejected-best-base metric without doing
additional work; that would require separate instrumentation/traversal justification.

Any `hints` error propagates exactly as before. Span-order, identity, summary and
I/O errors are failed operations, not silently recoded to a successful terminal
bucket. Diagnostic aggregate overflow likewise invalidates the diagnostic; do not
saturate it and continue claiming conservation.

## Handoff, duplicates and fixed representation

The tag must follow the **same first representative** as existing `PhysicalHints`.
No duplicate should get a better prior, merged hints, a different first span, or
additional matching because instrumentation is enabled. At collisions where both
objects are already materialized, aggregate that their existing diagnostic states
differ; do not fetch prior discarded objects or resurrect alternate candidates.
Cross-batch deduplication that only retains IDs cannot reconstruct an exact
discarded-representative tag without extra state: record that detail unavailable
rather than allocating a new per-object index. Duplicated occurrence counts are
not unique missed-byte opportunity.

Minimal prospective ownership boundaries:

| Location | Smallest required change | Policy-preservation requirement |
|---|---|---|
|`objects.rs:147` `PhysicalHints`|Two explicit u8 fields: diagnostic progress tag and triggered reservation grants, default0|Preserve prior IDs, span and predecessor; verify layout, derived equality uses, and copy paths|
|`objects.rs:2023–2064` deferred delivery|One sticky reason per existing cursor; wrap the existing callback and inspect before/after counters|No cursor algorithm change, reservation refund, extra atomics, retry, reordered object or altered batching|
|`objects/spill.rs:468–481`,`:584–603`|Carry tag through existing private hint frame when objects are spilled/remerged|Keep144-byte hints and184-byte row framing; validate every tag/reserved bit explicitly|
|`objects/admission.rs` initial-missing eligible ownership, before optional matching exclusion|Consume diagnostic state into fixed aggregate counters, retaining tag through later final outcome ownership as required|No optional-policy change; include eligible targets skipped for optional memory, not only those reaching `DeltaSearch::candidate`|
|`workspace.rs:657–673` and receipt schema|Publish independent cursor/target aggregates at acknowledgement|New counters do not change existing physical receipt semantics or sum nested elapsed timers|

The existing spill frame has zero-reserved bytes13–15; a prospective diagnostic
could assign byte13 to the explicit tag and14 to triggered grants, keeping15 zero. Old zero means
unobserved. This is an ephemeral private-spool schema, not a Store format change.
Normal frame validation must allow only declared tag values and reject invalid
combinations such as inherited-complete or inherited with nonzero grants; it must not simply disable reserved-byte
checks. Original sealed files are not migrated or rewritten.

A concrete tag coding is low bits0–2:0 unobserved,1 complete,2 memory,3 file,
4 operation,5 descriptor; bit3 means inherited exhaustion. Bits4–7 and the
remaining reserved frame byte15 stay zero. Values6/7 are invalid; inherited is
valid only with reasons2–5 and grants0. Grants must not exceed7. Use named fields
and normal serialization, never reads/writes into native Rust padding.

Transport alone must not create a second observation. Preserve a generated tag
and grants through rebuffer/spill; every deduplication drop owner (including
`DeferredObjectStore::put_authenticated`) accounts its discarded occurrence work
before dropping it. The authoritative initial CAS probe consumes surviving grants
once into its actual preexisting/missing work category and clears only grants;
the reason tag continues with the missing object for terminal classification.
A second active correspondence cursor acting on an already tagged occurrence
requires an explicit diagnostic-validity failure, not silent tag overwrite or a
second claim on the same work. `inherited_exhaustion` describes cursor state,
not the fact that a tag was transported. The actual measured Commit route and all
reachable collision/drop paths require prospective handoff tests before a run.

The probe shows both explicit byte fields fit existing `PhysicalHints` layout on the
current compiler/target: **160 bytes before and160 after**, with the corresponding
authenticated-object model **216 bytes before and216 after**, equal alignments,
and unchanged144-byte private hint frame/184-byte row overhead. This is a test result,
not a Rust ABI guarantee. A prospective patch must assert matching size/alignment
for `PhysicalHints` and the containing authenticated-object representation, and
verify capacity/memory-accounting decisions stay identical. The code uses
`size_of::<AuthenticatedCanonicalObject>()` for page capacity and optional
encoding memory decisions: casually growing the struct would change the policy
being diagnosed. If the tag cannot fit under those guards, stop and revise the
instrumentation contract; do not silently lower capacities or relax budgets.

## Reconciliation and validity checks

For a complete, uniquely validated, initially missing eligible cohort E:

```text
sum(cursor_bucket.count) = E.count
sum(cursor_bucket.canonical_bytes) = E.canonical_bytes
sum(stop/inherited/hint_count cross-tab) = corresponding requested-span buckets
limited_with_hints + complete_with_hints = targets proceeding with hints
new_cursor_exhaustions = sum(memory/file/operation/descriptor denial events)
reserved_bytes = successful_metadata_reservations * 131136
sum(delivered_occurrence.triggered_reservation_grants) = successful_metadata_reservations
```

The last three equalities belong to cursor-work events across all producer
occurrences, not to E. A failed reservation does not increment reserved bytes;
an inherited limited call does not increment new cursor exhaustion. A cursor
may exhaust after spending zero bytes. The first denial may occur on FileState
initialization, not only on extent pages. The grant count belongs to the occurrence
that triggered the read, even if another later occurrence benefits from the cursor's
already decoded leaf. Assigning those grants to a later CAS disposition measures
where work was triggered; it does not prove all work spent on a reused occurrence
was useless, because a later initially missing target may reuse the traversal state.
Capture discarded duplicate occurrence grants before dropping them; aggregating
only survivor tags cannot reconcile all producer work.

For target buckets, zero-byte sums with nonzero counts must be justified by the
canonical format; file-content canonical objects have nonzero framing and are
not empty canonical records. The sums cannot use logical payload length for some
rows and canonical length for others. Race/duplicate validity and final winner
precedence follow the admission diagnostic contract: retain attempted observations
but do not call them a once-per-run unique cohort when the declared invariants fail.

## Complete-empty correspondence lemma

For a valid contiguous predecessor, positive requested length, correctly mapped
monotonic first spans, and a nonexhausted traversal, empty hints imply
`target_start >= predecessor_logical_len`. Extents have positive length; their
positions are contiguous; skipped subtrees end at/before target start; and the
first overlapping extent necessarily inserts an ID into the initially empty
local hint array. Repeated IDs cannot suppress that first ID. A retained current
extent from a previous call cannot jump past an in-bounds target: traversal stops
on the extent whose end reaches the prior target end and advances contiguously.

This supports an oracle-only EOF-growth ceiling when separate source proof ties
the selected span to the same-path predecessor and to the full canonical payload
length. It does not turn missing spans or limited traversals into no-overlap and
does not establish similarity/compressibility of in-bounds bytes. The probe
exhaustively checks monotonic two-query sequences with gaps across a small valid
file, including prior/current extent state and past-EOF spans.

## Runnable evidence and limitations

```sh
python3 docs/roadmap/0.1/0.1.4/issue87-diagnostic-design/correspondence/run.py
```

The runner pins the real cursor/reservation source and previous synthetic fixture,
constructs a disposable offline Cargo project, and builds into a separate temporary
target directory. The probe verifies original-vs-observed hint outputs, cursor
counters, reservation accounting and representative metadata-read counts;
collision precedence; inherited exhaustion without retry; partial file/descriptor
limits; completed no-overlap; and same-size tag layout. All checks pass. No
benchmark measurements, candidate samples, physical savings or timing equivalence
are claimed. Instrumentation still has foreground CPU/cache/receipt costs and can
perturb concurrent scheduling; a future run must disclose and measure those costs.

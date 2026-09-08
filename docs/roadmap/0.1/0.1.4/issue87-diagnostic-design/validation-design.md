# Prospective diagnostic validation design

This is an analysis-only schema and validator. It collects no samples, opens no Store and changes no product or benchmark code. Run its synthetic check with:

```sh
python3 docs/roadmap/0.1/0.1.4/issue87-diagnostic-design/validate_diagnostic.py --self-test
```

Validate a JSON receipt, or a JSON array containing Init plus all157 receipts, against a separately frozen expected identity:

```sh
python3 docs/roadmap/0.1/0.1.4/issue87-diagnostic-design/validate_diagnostic.py receipts.json expected-identity.json
```

The expected identity has `source_seal`, `product_seal` and `workload_manifest_sha256`, each a lowercase SHA256. It must come from independently authenticated producer/build/workload custody. A receipt cannot authorize its own expected identity. The validator prints JSON and exits nonzero on an invalid or incomplete diagnostic. It never edits the input, so failed/racing attempt evidence remains intact.

## What validation can establish

The validator checks aggregate schema, count/byte conservation, complete versus incomplete causal state, phase identity, checkpoint chain and cohort equalities. It **does not open the referenced provenance manifest, authenticate locators, inspect the producer binary, or prove the source invariants**. Those are mandatory upstream custody/review steps. `new_location_provenance.status="authenticated"` records their outcome; the validator does not turn that assertion into independent authentication. Its successful result explicitly says provenance authentication was required upstream and that the unique-cohort interpretation depends on reviewed source premises.

For the prior sealed run, source review and per-checkpoint checks establish: retained logical prefix L_i is a subset of selected admitted objects S_i; their acknowledgement cardinalities agree; therefore L_i=S_i. Every new selected eligible payload necessarily passed the eligible observation site; size exclusions and unobserved optional routes are absent; eligible counts equal new selected eligible counts at each checkpoint. This is a bijection between observed attempts and newly admitted targets for that run. Equal counts without source-proven inclusion would not suffice.

The proposed diagnostic preserves this cheap gate instead of adding a new global deduplication subsystem. At every checkpoint, eligible attempt count **and full canonical bytes** must equal the authenticated newly selected eligible cohort. Final admitted DELTA count/bytes must agree too. Any late race or mismatch makes the **unique target table unavailable**, even if all attempted-event counters remain internally consistent. A loser never overrides a sibling winner's unique ObjectId outcome. Race-free source premises and these equalities allow unique interpretation; they are not promised for arbitrary concurrent workloads.

## Collection boundaries

1. **Inside the existing public operation:** update fixed integer counters and bounded histograms from information the existing correspondence/CAS/admission owners already possess. No paths, raw payload, ObjectId list or per-match trace is written by this schema. Pre-CAS occurrences and overlapping trial events remain separate from post-CAS eligible attempts. No codec, matcher, hint choice, canonical format, spill layout or budget change is bundled.
2. **At acknowledgement:** record ordinary selected-count/bytes/allocation/resource receipts, cheap authoritative admission-transaction pack provenance, and serialize the fixed aggregates outside the public timer. The source owner must establish the publication barrier and prevent an unrelated writer from crossing the operation boundary.
3. **At the fixed final logical snapshot:** one authenticated pack/object/graph inventory supplies selected locators, roles, immutable graph edges and retained roots. Reconstruct chronological L_i with one incremental visited-set union over those already inventoried edges. Populate `retained_prefix` and the independent new-location aggregates retrospectively. **Do not traverse a Store, decode all groups, copy a Store or rescan the objects table after every Commit.**

For location provenance, prefer bounded transaction pack-interval/count observations during admission, authenticated against the final inventory. Fold first/last/count from existing successful insertion transactions into a fixed per-operation range; do not issue extra SQL or log individual objects. A per-ack min/max interval is usable only when source review and checks prove append-only allocation, `count = last − first + 1`, interval completeness, disjoint operation ownership and no external writer. The final join must reject missing/duplicate packs or cross-operation ranges. Otherwise require explicit bounded transaction ranges or fail provenance. Final pack-ID order by itself proves nothing. Do not accumulate an unbounded per-object log in memory, move such a log into timed I/O, or perform an unindexed full `objects` scan each acknowledgement merely to export locators. The admission-owner design supplies the exact interval/transaction evidence; this validator consumes its independently authenticated aggregate result.

## Receipt fields and units

All target/occurrence weights use exactly `{ "count": integer, "canonical_bytes": integer }`. Canonical bytes mean the **entire canonical target object**, including framing, counted once per member of the named population. They never mean DELTA program bytes, compressed group bytes or savings. Each `occurrence_routes` weight additionally carries `grants_count` and `reserved_bytes`; the latter must equal131136 times the grants. Route grants/reservations must sum to the independently observed `correspondence_grant_totals`. The observed operation reservation sum cannot exceed the unchanged16MiB bound. Credits are consumed exactly once, including discarded duplicate occurrences; a zero-occurrence route cannot own grants. These quantify which occurrence triggered the work, not avoidable work or savings. All integers are nonnegative; booleans are rejected where a count is required. Zero count and zero bytes must occur together.

| Field | Population and provenance | Snapshot/status |
|---|---|---|
| `schema` | Exact `issue87-coverage-diagnostic-v1` | Frozen design identity |
| `identity` | Independently frozen producer/source/workload identity | Authenticated upstream |
| `checkpoint`, `phase` | Ordinal0 Init;1–157 Commit | Recorded operation boundary |
| `complete` | Whether the acknowledged operation/diagnostic is complete | Recorded; false fails completeness |
| `eligible_shape_occurrences` | Every eligible-shaped payload delivery, including duplicate/reused occurrences | Measured delivery work |
| `occurrence_routes` | Exclusive `existing`, `duplicate`, `initially_missing` occurrence weights | Measured existing CAS/dedup route; sum equals occurrence total |
| `correspondence_grant_totals` | Measured `grants_count` and `reserved_bytes` from existing cursor reservation owner; sum of exclusive occurrence credits must agree | Operation interval observation |
| `eligible_attempts` | Initially missing eligible target attempts after existing CAS filtering, before optional search exclusions | Measured; equals `initially_missing` route |
| `cursor_histogram` | Fixed exclusive post-CAS eligible-attempt state cells | Measured from preserved first-owner correspondence state |
| `search_histogram` | Fixed exclusive cells for attempts having at least one hint | Measured candidate/group outcome state |
| `terminals` | Exactly one final reason per eligible attempt | Measured attempt outcome; unique interpretation only after cohort gate |
| `event_counts` | Overlapping trial/fetch/limit/duplicate-observation events, values are integer counts; names are restricted to the fixed `EVENT_COUNTS` set | Measured event population; deliberately not summed against target counts |
| `late_recheck_existing_count` | Late admission conflict events affecting eligible target attempts | Measured; any positive value fails unique-cohort gate |
| `previous_selected`, `ack_selected` | All selected objects and canonical bytes, including structural objects | Original adjacent acknowledgement receipts |
| `retained_prefix` | Union of all logically retained roots through this acknowledgement, including required initial/stage roots | Derived once from the final authenticated metadata graph |
| `new_location_provenance` | All new selected objects plus eligible FULL/DELTA target weights and custody reference | Retrospective authenticated join to per-ack transaction/pack evidence |

`new_location_provenance` contains `checkpoint`, `status="authenticated"`, `source="successful_transaction_pack_ranges_plus_final_authenticated_locator_join"`, `manifest_sha256`, zero `duplicate_object_count`, zero `duplicate_locator_count`, and weight fields `all_objects`, `eligible_FULL`, `eligible_DELTA`. The manifest is external provenance evidence, not a timed per-target log. The validator verifies its hash syntax/custody label and aggregate equalities; the upstream reviewer authenticates its actual contents and record membership.

A full array requires exactly158 ordered receipts, starting at checkpoint0 with empty `previous_selected` and no predecessor/search outcomes, followed by a joining previous/acknowledged count-and-byte chain. Individual-receipt mode cannot by itself establish whole-run cardinality or continuity.

## Cursor states detect missing spans and incomplete search

Each cursor histogram row has exactly these state fields plus a count/byte weight:

```text
has_predecessor: boolean
span_present: boolean
stop: not_applicable | complete | memory_limit | file_limit |
      operation_limit | descriptor_limit
hint_count: integer0..4
inherited_exhaustion: boolean
```

Rows with the same state key must already have been aggregated; duplicate cells are rejected. The finite key space bounds the number of rows. Extra per-target fields such as paths or ObjectIds are rejected. The cursor owner records the first real stopping cause at the existing reservation short circuit and carries its reason into later inherited exhaustion. `inherited_exhaustion` requires a limit stop.

The validator derives the first four terminal reasons:

- No predecessor requires zero hints/no cursor traversal and maps to `no_predecessor`. Span absence here is not a handoff defect.
- Predecessor present but span absent requires zero hints/no traversal and maps to `missing_required_span`. A positive count generates an explicit `handoff_defect` finding. It remains valid diagnostic evidence of the defect; it is never silently relabelled no-overlap.
- Valid predecessor+span, complete cursor and zero hints maps to `no_overlap`.
- Valid predecessor+span, limit-stopped cursor and zero hints maps to `correspondence_limit`.
- Any positive hint count proceeds to the search histogram, including partial hints from a limit-stopped cursor. Partial correspondence does not force a failure terminal if an available candidate later wins.

Thus a receipt calling an operation-limited empty cursor “no overlap” fails causal-state reconciliation even when its overall counts add up. A predecessor-present missing span cannot be counted as normal no-overlap. These rules only validate faithfully collected states: instrumentation must actually observe the construction/delivery boundary and real cursor stop, not invent labels after the fact.

## Search and admission states

Each search histogram row has exactly these fields plus a count/byte weight:

```text
usable_base: boolean
search_complete: boolean
raw_candidate_complete: boolean
mixed: not_run | rejected | selected
```

`search_complete` means all reached/allowed search work completed without the target being abandoned to a fetch/match/trial/instruction/memory bound. It does not claim a global search or optimal matching. `raw_candidate_complete` means an admissible complete DELTA program exists, **not** that its raw size or compressed size saves bytes individually. A target with a complete candidate can retain incomplete-search flags when another trial hit a bound.

- No complete raw candidate + incomplete search → `search_budget`, whether a usable base was already found or the fetch budget stopped first.
- No complete candidate + complete search + no usable base → `unavailable_base`.
- No complete candidate + complete search + usable base → `no_useful_raw_delta` (no complete admissible program from the allowed search).
- Complete candidate requires a usable base and a completed same-membership mixed-group comparison. Rejected → `compressed_mixed_rejection`; selected → provisional DELTA outcome, accepted as `delta_admitted` only when final winner accounting passes.

The histogram covers only hinted attempts. Targets with no hints retain their specific correspondence/handoff reason; coincident optional-search budget events remain overlapping events rather than replacing an earlier sufficient reason. The admission/cursor owners must freeze that terminal precedence consistently.

The complete terminal map always includes ten reasons: `no_predecessor`, `missing_required_span`, `no_overlap`, `correspondence_limit`, `unavailable_base`, `search_budget`, `no_useful_raw_delta`, `compressed_mixed_rejection`, `delta_admitted`, `admission_race`. Its counts and canonical bytes sum exactly to eligible attempts. If any race exists, attempt evidence is preserved but the unique table fails closed before comparing prepared causal reasons to persisted winners. No race, no missing term and complete population identities are prerequisites for a unique result.

A/B group comparisons and encoding-cost counters are a separate group/event population, outside this target validator. They need paired FULL/mixed lengths for identical FULL membership, with B-not-attempted distinguished, and final admitted/partial/discarded pack evidence. No target byte attribution can turn those compressed group numbers into per-record measured savings.

## Failures and tests

The runnable synthetic test passes a valid receipt containing a real missing-span finding, a completed empty correspondence, and a partial-cursor/incomplete-search target that nevertheless admits DELTA. Event counts exceed target counts deliberately, proving the validator does not conflate them. It validates a complete158-checkpoint chain, then rejects17 cases covering changed identity, byte conservation, limit masquerading as no-overlap, required correspondence outcome missing, mixed result without a candidate, late race, winner-byte mismatch, retained-prefix mismatch, duplicate selected identity, duplicate histogram cell, boolean count, per-ObjectId histogram field, dynamic per-object event-counter names, double-counted grant credits, operation reservation overflow, missing checkpoint and duplicate/reordered checkpoint.

These tests are entirely synthetic and product-free. They do not prove the instrumented implementation correct before one exists. Before any prospective run, the source reviewer must confirm first-owner preservation through CAS/dedup, coverage of every eligible route including optional-memory skips, faithful cursor-completion state, and admission/counter ownership. Existing oracle/custody/cleanup/resource checks remain mandatory and independent; this validator does not replace them or permit a replay.

## Review of the tighter no-overlap discriminator

The additional source/oracle bound is defensible only on the frozen ordinary-import path: the predecessor is the same path's previous checkpoint regular-file state; each new target's span uses absolute replacement-file coordinates and its length equals its full CHUNK payload; prior extents cover the file continuously; and a genuinely completed empty correspondence for a positive-length target therefore lies beyond prior EOF. Under those premises, charge each unique no-overlap target to one actual occurrence in the positive-growth tail. CAS deduplication reduces the unique set relative to the occurrence sum, so it cannot enlarge this upper bound. Add21 canonical-framing bytes per target; enforce per-checkpoint target-count/size ceilings rather than summing incompatible extrema.

The root-owned implementation reports the tightened per-checkpoint upper bound65446350 canonical bytes for legitimate no-overlap and a residual lower bound240892334 canonical bytes requiring another explanation. This review accepts the arithmetic and set reasoning under the stated source premises; it has not repeated the oracle scan. Partial slices, relative coordinates, holes without payload coverage, predecessors from intermediate writes or a different path would invalidate the bound and must be excluded by source review. The residual is **not** automatically useful-delta opportunity: missing span, reservation/descriptor limits and other incomplete correspondence remain alternative causes. The fixed histogram above is designed to separate them without changing the search policy.

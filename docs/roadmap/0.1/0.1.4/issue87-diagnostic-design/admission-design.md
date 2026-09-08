# Admission and pre-CAS work attribution design

This is a prospective design for the single unchanged-policy full157 diagnostic. It implements no product instrumentation, optimization, replay, or Store scan. Source ownership and ordering were traced through `objects.rs`, `objects/spill.rs`, `objects/admission.rs`, `workspace.rs`, `layerstack.rs`, and `changes.rs`. The accompanying `admission-accounting-check.py` runs synthetic accounting assertions only.

## One minimal representation

Use the two explicit diagnostic `u8` fields agreed with the cursor design: **cursor result tag** and **successful reservation grants triggered by this occurrence**. Do not create a global ObjectId registry or a parallel page sidecar. The independent layout witness found that two named fields preserve `PhysicalHints` size/alignment and the enclosing authenticated object size/alignment on this host: 160→160 and 216→216 bytes respectively. These are ordinary named fields, not unsafe reads or writes of Rust padding.

Prospective implementation must enforce those equalities for the actual compiled types and verify all capacity expressions, canonical batching thresholds, physical-encoding association charges, and spill row lengths before a run. Candidate spill already has a 144-byte hint frame and 184-byte row overhead. Allocate its currently reserved bytes 13 and 14 to tag and grant count; byte 15 remains zero. Apply the cursor document's complete tag validity rules, reject unknown bits/invalid combinations, and bound a call's grant count to 0..7. An inherited exhausted-cursor result must have zero newly triggered grants. The grant byte is a count; its reservation charge is derived as `count × 131,136` bytes.

This changes a private temporary diagnostic artifact's interpretation, not Store or canonical encoding. Old/new binaries are not interchangeable readers of these temporary candidate files. Original evidence remains untouched. If actual layout/capacity equivalence fails, stop diagnostic preparation and amend the design; do not silently enlarge objects, reduce canonical budgets, or introduce another transport architecture.

Only the grant count is needed for work attribution. Keep descriptors and read-reservation-cause counts at cursor/operation scope; carrying per-target descriptor arrays or per-file identity logs is unnecessary.

## Four populations, four different meanings

| Population | Exact owner / denominator | Bytes to record |
|---|---|---|
| Cursor work | Each attached predecessor cursor and its metadata reservation callback | Granted reservation bytes; explicitly not actual I/O bytes |
| Delivered occurrence work | Every delivered authenticated payload occurrence, including occurrences later deduplicated or reused | Occurrence canonical bytes, which can count the same ObjectId more than once; associated triggered reservation grants |
| Initially missing eligible attempts | Unique IDs within each existing admission owner after its initial CAS probe; cross-owner repeats remain attempts | Canonical target bytes once per attempt |
| Selected new payload cohort | Newly inserted selected eligible FULL/DELTA locators at the acknowledgement | Canonical bytes once per selected ObjectId |

Cursor counts include attachment, first actual query, successful initialization, completion, and each limiting reason, with fixed histograms for grants per cursor and descriptors per cursor. They do not purport to be file counts when no cursor was attached. File logical input bytes, if reported from existing producer metrics, remain a separate quantity. Neither occurrence canonical bytes nor unique CAS bytes are interchangeable with logical file lengths.

Do not derive per-cursor I/O by subtracting shared Store physical counters around a call: other producers may contribute during that interval. The exact local quantity is the reservation callback's successful grant count. Actual Store read/decode counters remain shared phase measurements unless separately instrumented with a justified, bounded owner attribution mechanism.

## Grant lifecycle and the initial CAS boundary

The measured Commit route is:

`ObjectBuffer completion → consume_prevalidated_pages → FinalizedOutputWriter::send_selected → bounded slab queue → CheckedOutputAdmission::admit_object → probe_incoming → MissingBatch → prepare/publish`.

The hint computation occurs **after** selected candidate spill readback, but **before** page/slab delivery and initial CAS filtering. Capture the local grant difference around that exact cursor call and attach it to the occurrence. Do not measure around a whole page: the current `push` closure can compute hints for the next object before flushing the preceding page. Page timing would assign that work to the wrong population.

At existing ownership decisions, consume the grant credit exactly once:

| Existing route | Work bucket | Unique-target action |
|---|---|---|
| First incoming occurrence, initial probe finds selected row | `fresh_probe_preexisting` | Not an initially missing target |
| First incoming occurrence, initial probe finds no row | `fresh_probe_missing` | Create one eligible attempt if exact role/size eligibility holds |
| Same ID already incoming or pending | `duplicate_occurrence` | Compare bytes as today; do not add a target |
| Seen-set repeat after earlier flush | `duplicate_occurrence` | Preserve existing comparison; do not reinterpret it as a new pre-existing target |
| Producer/consumer error or drained output never classified | `unclassified_failure` or explicit conservation residual | No successful cohort claim |

For the two fresh-probe routes, the metadata byte remains with the existing incoming object until `probe_incoming` supplies the result. There is no new membership query. For a duplicate, aggregate its credit before the existing byte-comparison/drop path loses its metadata. The first occurrence's predecessor IDs and span continue to win exactly as they do now: diagnostic accounting must not merge hints or alter that choice.

After aggregation, clear **only the grant credit** in the admitted owner's object. Preserve its cursor result tag for post-CAS reason classification. Prepare/publish consumes the tag to classify the eventual attempt result; it must not count the pre-CAS grant again. The grant field means outstanding attribution credit, not a persistent physical property of the canonical object.

The declared full157 route delivers once to the admission consumer. Instrument route checks for unexpected rebuffering, repeated active correspondence on an already tagged occurrence, or pending credit crossing an unsupported ownership boundary. Record a diagnostic invalidity flag, preserve the product's existing result, and fail the diagnostic coverage gate outside the public timer. Do not reset a live credit, silently overwrite the earlier cause, or create a new persistent trace index to handle an unrequested route. If later scope includes rebuffering, its additional duplicate/drop paths need explicit review before that separate run.

On the successful operation, after all producers join and incoming/final batches drain:

```text
successful cursor reservation grants
  = grants triggered by fresh_probe_preexisting occurrences
  + grants triggered by fresh_probe_missing occurrences
  + grants triggered by duplicate occurrences
  + unclassified grants

unclassified grants = 0
```

The outer shared reservation counter is the authoritative grant total and must be retained on failures too. A callback/read failure can occur after a successful grant but before an object is delivered. Preserve that delta in the failure bucket when possible; a panic or interrupted lifecycle must leave an explicit residual and invalid/incomplete status, never an invented zero. Do not modify public success/error behavior to make telemetry balance.

All bucket rows carry occurrence count, occurrence canonical bytes, and grant count/derived reserved bytes. They are exclusive **occurrence** routes. Structural objects without a payload query have zero grant credit; eligibility uses exact outer/chunk decoders rather than a raw content prefix.

## Triggered by reused content is not avoidable work

The strongest exact label is **“reservations triggered by an occurrence whose eventual initial probe found an existing selected object.”** It is not “wasted reservations” or “recoverable budget.”

For example, a reused first chunk can trigger two reads to initialize a leaf predecessor cursor. A later missing chunk in the same cursor can then receive useful hints without another grant. Skipping the reused first chunk merely moves the same two startup reads to the missing chunk. The synthetic accounting check proves that two grants attributed to a reused trigger can coexist with **zero avoidable grants**. Equally, the missing chunk's zero triggered grants do not imply zero benefit from correspondence.

This design does not track a global set of files/cursors to decide which entire files were eventually reused. Such a result is deliberately unavailable. The exact triggered-work buckets, cursor saturation/completeness counters, and post-CAS byte-weighted causes suffice to decide whether deferring correspondence deserves a later one-variable experiment. Net work avoided and additional useful coverage remain counterfactual quantities until that experiment. Do not subtract reused-triggered bytes from the 16 MiB allowance and translate the difference into extra useful chunks.

## Attempt outcomes and cohort validity

At initial CAS absence, count eligible attempt count and canonical bytes independently of the optional physical-memory gate. This corrects the denominator vulnerability of placing eligibility telemetry only inside optional candidate search. It changes no actual eligibility or candidate decision. Later memory exclusions are an overlapping attempt event; they must not erase a target's already determined no-predecessor/no-hint reason.

Use the cursor review's exclusive post-CAS states: no predecessor; missing required span; complete empty correspondence; limited empty correspondence; complete with hints; limited with hints. The independent validator maps these to the declared attempt terminal categories. A missing required span is a diagnostic correctness failure. Incomplete correspondence is not proven no-overlap. A target can have a complete candidate and an incomplete later trial; preserve fixed overlapping completeness flags rather than forcing those facts into an invented linear funnel.

The final late recheck has two separate consequences:

- A selected winner receives its actual FULL/DELTA terminal outcome only after transaction commit.
- A losing prepared attempt receives `admission_race`; it does not override another owner's successful outcome for the same ObjectId.

Require terminal attempt count/bytes to conserve the eligible attempt denominator. For this frozen single-owner diagnostic, retain the simplest validated cohort gate:

```text
eligible attempt count/bytes = newly selected eligible count/bytes
late race attempt count = 0
all selected eligible objects have one necessary eligibility event
all ownership/coverage/provenance checks pass
```

These conditions establish the attempt/selected bijection for the accepted run. Equality of numbers alone is insufficient without the source-path and provenance checks. If they fail, retain attempt counters and mark the unique terminal table unavailable/invalid. Do not add a global dedup subsystem or quietly accept double-counted target bytes. The synthetic check rejects two missing attempts for one selected object, even though their attempt terminal counts conserve.

## A/B group and admission provenance

Capture the A/B/selected lengths already computed by `encode_group`, with fixed FULL membership, codec choice, candidate-bearing count/bytes, and its existing threshold decision. A/B values are available as local counter deltas or local return observations; no additional encoding is needed. Maintain aggregate group counters in separate diagnostic storage so changing `EncodedGroup` size does not alter its existing association reservation. Detailed per-target IDs or paths are not serialized in timed operations.

Preparation and successful persistence are different populations. At publish, classify each prepared group using existing winner locators and pack ownership:

- no pack winner: group not persisted;
- a pack winner exists: every group in that pack persists, including groups with no selected locator;
- within persisted groups, selected-location coverage may be complete, partial, or zero.

A partial pack still writes its whole BLOB. Never allocate its compressed bytes in proportion to winner count. Keep attempted A/B totals, persisted chosen bytes, and selected FULL/DELTA object count/bytes separate. Final census verifies physical unselected records rather than assuming they cannot occur.

For authoritative checkpoint provenance, use successful transaction **pack ranges**, not timed per-object logs. `insert` already reads the prior maximum pack ID and assigns successive IDs. After successful commit, fold first/last/count into a fixed operation receipt. Under the diagnostic's single Store writer, no interleaved public operations, immutable locators, and append-only pack IDs, the operation range is contiguous; validate count against range length and fail provenance if it is not. On failure, never label uncommitted assigned IDs as persisted.

At the one final pre-verification logical census, enumerate selected locators once and join each pack ID to the sealed successful-operation ranges. This yields authoritative per-checkpoint selected-location count/byte manifests without scanning the unindexed `objects.pack_id` predicate or decoding the Store after every acknowledgement. Validate disjoint ranges, complete pack coverage, no unexpected writer or relocation, and correspondence between each operation's winner totals and final locator join. Provenance is **successful transaction pack ranges plus a final authenticated locator join**, not final pack-ID order alone.

## Cost and prospective checks

The two-field design adds no canonical object size, spill row bytes, queue slots, payload buffering, new database probe, or codec invocation when the required layout gates pass. It still adds instructions, branches and fixed counters, and can change producer scheduling; record this as diagnostic instrumentation, not a free or clean timing control. Fixed histograms and per-phase aggregate maps remain bounded independently of workload length. Include their actual memory in process/RSS observations and preserve original safety limits.

Before authorizing execution, the implementation must pass: actual-type layout/alignment and capacity equivalence; exact private spill roundtrip/invalid-bit tests; identical original hints and grant acceptance under scripted cursor inputs with telemetry on/off; duplicate-credit conservation; inherited sticky-cause/grant-zero behavior; late-race attempt/selected rejection; partial-pack group accounting; and successful pack-range/final-locator provenance. No malformed telemetry should silently change a product decision or manufacture an accepted diagnostic result.

The standalone check in this directory currently verifies only the accounting distinctions: shared cursor benefit, occurrence versus unique target bytes, race cohort rejection, and whole-group persistence under partial admission. It does not certify a future instrumentation implementation. All raw measures use integer count/byte units, operation/checkpoint and cursor/occurrence/attempt/selected scopes, explicit measured/derived/unknown status, and null reasons where a counterfactual or failure residual is unavailable.

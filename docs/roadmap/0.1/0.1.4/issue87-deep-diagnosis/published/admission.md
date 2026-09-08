# Independent candidate/admission diagnosis

Source checkpoint: `bd220ae4a78a4923289af1a582a914a590c562d2`. This is a source review and one deterministic synthetic matcher check. It does not open the retained Store, collect benchmark samples, call a codec, replay a workload, or change product code. The prior sealed full157 analysis remains immutable. Counter operands below come from its `counter-reconciliation.json`, `roles-summary.json`, and authenticated object inventory; the root analysis independently establishes per-checkpoint first-retention/growth equalities.

## Finding

**The admission/matcher stage is not the demonstrated cause of the large uncovered payload population.** The frozen run has 86,417 unique selected payload objects and exactly 86,417 eligibility events. Of these, 77,949 received no hint, 8,468 received at least one hint, and 8,024 became DELTA. There are therefore exactly 444 hinted targets that remained FULL. All fetch, matching, instruction, and physical-memory budget skip counters are zero. The only nonzero budget category is predecessor correspondence, owned by the separate correspondence review.

This is stronger than a low DELTA-count observation. Most targets never reached the matcher. A synthetic exact-source check also demonstrates that the matcher can miss a shorter valid program, but establishes no material full157 savings from changing it.

## Production path and uniqueness proof

All production prepared-admission entry points were inspected:

1. `objects.rs::consume_checked_owned_page` publishes early batches from `CheckedOutputAdmission::flush_batch`.
2. `workspace.rs` prepares the remaining checked batch and publishes its objects with Commit metadata.
3. `layerstack.rs` prepares the remaining checked batch and publishes initialization objects with Layer/LayerStack metadata.

`CheckedOutputAdmission::admit_object` suppresses duplicates already pending or incoming, while comparing actual canonical bytes. `probe_incoming` uses its spillable seen set to exclude repeated occurrences across flushed pages, compares pre-existing selected objects, and only passes initially missing objects to `push_pending`. `take_batch` closes logical dependencies. `PreparedAdmission::prepare_missing` additionally rejects duplicate ObjectIds within the handed-off `MissingBatch`.

`PreparedAdmission::publish` rechecks locations under the existing operation permit, compares newly present canonical objects, selects absent rows as winners, and inserts each pack only if it contains at least one winner. A partially winning pack retains all its physical records. FULL/DELTA counters are incremented only after transaction commit and count winners; A/B/selected encoded bytes were incremented during preparation. A generic concurrent caller could therefore have surplus attempted encoding, and a partially winning pack could contain unselected records. The current inventory has zero unselected records.

For this particular run, the stronger payload bijection is justified:

- Every selected payload has canonical length at most 32,789 bytes. The singleton bypass requires `canonical_length + 9 > 65,536`, so no selected payload takes it.
- Every remaining payload goes through `prepare_ordinary`. Its physical-memory optional-work bypass increments `memory_budget_skips` for every bypassed object. The measured count is zero.
- `DeltaSearch::candidate` authenticates the outer bytes-object framing and exact chunk payload encoding before incrementing `eligible_targets`. It increments once for each such object passed through this path, before no-hint or matcher exits.
- Each newly selected payload therefore requires an eligibility event. Initially missing duplicate preparations across independent owners or later admission race losses would add events without adding a second selected object.
- Root's per-checkpoint new-payload/eligibility equality, together with the above no-omission path, leaves no room for extra payload preparation events in the recorded intervals. It establishes an event-to-new-payload bijection for this evidence. It does not change the general meaning of these telemetry counters.

`has_predecessor == false` also implies no prior IDs in production: `PhysicalHints` defaults to false/empty; its only production prior-ID assignment is `objects.rs:2043`, conditional on an attached predecessor, after `objects.rs:2036` sets `has_predecessor` true. Spill preserves the fields. First-occurrence deduplication can discard later hints but cannot violate this implication. Thus the 18,589 absent-predecessor targets are a subset of the 77,949 no-hint targets, leaving **59,360 targets with a predecessor but no hint**, rather than an overlapping subtraction with unknown meaning.

## Counter reconciliation and resulting limits

All quantities in this table are integer counts or bytes for the full157 ready/public-checkpoint intervals. Derived rows identify their operands. Encoded bytes are group-level quantities, not filesystem allocation.

| Quantity | Value | Population / status |
|---|---:|---|
| Eligible payload targets | 86,417 | Measured eligibility events; bijection to selected payloads established above |
| No-hint targets | 77,949 | Measured unique targets in this evidence |
| Absent-predecessor targets | 18,589 | Measured; subset of no-hint |
| Predecessor-present/no-hint targets | 59,360 | Derived `77,949 − 18,589` |
| Hinted targets | 8,468 | Derived `86,417 − 77,949` |
| DELTA-admitted targets | 8,024 | Measured committed winners |
| Hinted-but-FULL targets | 444 | Derived `8,468 − 8,024` |
| Candidate trials / usable bases | 11,293 / 11,293 | Measured per authenticated distinct base trial, not distinct targets |
| Predecessor hints considered | 11,555 | Measured hint loop events, not distinct targets |
| Rejected mixed groups | 332 | Measured groups, not targets or trials |
| Matcher/fetch/instruction/memory skips | 0 / 0 / 0 / 0 | Measured; does not imply exhaustive matching |
| Prepared selected encoded bytes | 295,240,878 | Measured sum of selected encoded group bodies |
| Census encoded group bytes | 295,240,878 | Independent decoded inventory's exact encoded body sum |
| Prepared FULL alternative encoded bytes | 315,555,931 | Contemporaneously encoded A groups under fixed membership |
| A minus selected encoded body bytes | 20,315,053 | Derived group-level difference; not a SQLite allocation saving |
| Admitted FULL plus DELTA records | 366,141 | Measured winners, equals unique selected census objects |
| Physical unselected records | 0 | Authenticated inventory measurement |

The exact prepared-selected/census tie-out corroborates the absence of excess prepared output in this run. General race caveats should remain in the field definition, but should not be presented as observed loss. The A-minus-selected quantity is evidence of smaller chosen encoded bodies on the actual group membership. A separately persisted FULL-only Store would have different page placement and allocation; this quantity does not establish a filesystem-space counterfactual or fresh performance control.

The root's checkpoint-specific target-size order-statistic bounds put the 444 hinted-but-FULL targets at no more than **13,091,077 canonical bytes**. This is an affected ceiling, not recoverable compressed bytes. Matcher changes could also shorten already admitted DELTAs: the entire selected mixed-group encoded population is at most the 11,639,986 bytes of all attempted B alternatives, since that counter includes both winning and rejected B groups. Neither observation establishes large savings, and neither is a per-record compressed attribution. The rejected-group count alone gives no measured byte opportunity.

## Source mechanisms: proven behavior versus policy and defects

### Eligibility and first-occurrence hints

**Proven source behavior; opportunity incidence unavailable.** `DeferredObjectStore::put_authenticated`, incoming/pending duplicate handling, and cross-batch deduplication retain the first canonical occurrence's `PhysicalHints`; they compare duplicate bytes but do not merge alternate predecessors or spans. A target first encountered in a new file can therefore retain no hints even if a later occurrence in another file has a useful predecessor. If the first occurrence is already admitted, CAS correctly excludes re-encoding it.

This is a bounded first-occurrence policy, not demonstrated corruption or an established full157 byte loss. The canonical identity authenticates content, not the quality of its optional hints. The existing target count does not reveal differing duplicate-occurrence hints. A future field could count initially missing duplicate targets whose later occurrence offers a previously absent eligible hint, with canonical-byte weights, but it should not become a second concurrent optimization before the correspondence bottleneck is addressed.

Smallest hypothetical boundary: bounded hint union before the first admission ownership decision, preserving four-ID and memory limits. Costs: additional duplicate bookkeeping and potentially more reads/trials; no on-disk compatibility change. Concepts deleted: none. No such change is implemented or recommended now.

### Depth-one anchors and search budgets

**Proven source behavior; not evidence of budget-induced full157 loss.** `DeltaSearch` owns one batch-local 512-trial cap, a 16 MiB charged matcher-work allowance, and `HintReadBudget`. It considers only supplied prior IDs and selected immutable records. A DELTA predecessor exposes its selected FULL anchor; that FULL canonical object is fetched and authenticated before matching. No record in the current prepared batch can become an anchor. Per-target hint reads are capped at eight, with 512 KiB encoded/decoded limits; batch encoded and decoded allowances are each 8 MiB. Repeated groups are charged repeatedly.

The frozen run has zero fetch/match/instruction/memory skips, so increasing those caps has no demonstrated affected-byte population here. Older FULL anchors can still be a worse similarity match than the immediate DELTA predecessor; the counters do not prove perfect bases. All 3,065 distinct anchors are already in the retained logical union, so retaining them does not add a physical-base-only population. Deeper chains would change reads and dependency policy without evidence that they address the major uncovered population.

### Greedy bounded matcher

**Proven bounded policy miss, not a demonstrated production defect.** `pack.rs::delta_record` indexes 16-byte seeds at 16-byte base strides in 4,096 buckets, retaining the first four offsets in each bucket. It scans target positions one byte at a time, tries those offsets, greedily takes the longest match of at least 16 bytes, and breaks equal-length ties by smaller offset. It does not verify seed identity before extending a bucket candidate; mismatches are bounded charged work. It limits the record to the corresponding FULL-record length and rejects incomplete budget-exhausted trials. Among completed bases, `DeltaSearch` chooses the smallest raw delta record, then smallest ObjectId; it does not rank candidates by compressed group bytes.

The standalone `admission-probe` includes the exact unchanged `pack.rs`. On a 1,045-byte synthetic canonical base and 149-byte target, four earlier identical seeds occupy the bucket and hide the fifth seed's longer match. The actual matcher emits a valid **85-byte** DELTA record. A manually specified valid instruction witness reconstructs the same target in **76 bytes**. Both programs are verified by the exact `apply_delta` implementation. The witness is not an alternative matcher, and no codec is called. The test also confirms that an exhausted inner trial increments `match_budget_skips` while generic `budget_skips` is a caller-level event, reinforcing that event counters are not mutually exclusive target outcomes.

This proves the matcher is not globally optimal, as its source comment already says. It does not show how often this occurs in full157 or whether a shorter raw program wins compressed group selection. The measured affected production bytes and removable portion are **unknown**, bounded by the much smaller hinted-target population. No stronger matcher or wider table is justified as the next experiment by this witness.

### Mixed group selection

**Proven policy tradeoff.** Membership is fixed using FULL sizes. A is the all-FULL group; B uses the single best raw delta found for every target with a candidate. Both are encoded using the same fixed codec rule. B wins only when its encoded saving is at least `max(64, ceil(A_bytes/8))`, a 64-byte or 12.5% threshold. Compression itself must save at least 16 bytes versus RAW.

A raw-shorter candidate can compress worse because it replaces repeated canonical material with base IDs and instruction framing. The all-candidate B can also lose when a proper subset would win; the implementation does not evaluate subsets. These are deliberate bounded costs, not proof that the raw matcher failed. Existing telemetry lacks rejected groups' A/B byte pairs and byte-weighted terminal targets, so it cannot distinguish slightly subthreshold useful savings from a materially worse B. The tight hinted-but-FULL canonical-byte ceiling and the absence of budget skips prevent the rejection count being used as evidence of hundreds of MB of missed target opportunity.

Smallest hypothetical boundary: one prospective fixed A/B rejection histogram, with encoded A/B bytes and number/canonical bytes of candidate-bearing targets per bin, while retaining the exact decision threshold. Costs: fixed aggregate updates only, no new codec calls. Concepts deleted: none; wire compatibility unchanged. This is a possible diagnostic if rejection becomes the leading residual after coverage improves, not a second next experiment.

## Disposition

No canonical corruption, unsafe admission race, incorrect DELTA reconstruction, or violated declared candidate budget was found in this bounded source review. The concrete runtime diagnostic limitation is that most targets have no hint before candidate search; the separate correspondence analysis explains the demonstrated pre-CAS reservation mechanism. Candidate/matcher limits and group selection are real quality/cost tradeoffs, but their measured reachable population is much smaller.

**Recommendation to the reconciler:** keep the single next experiment at predecessor correspondence coverage, with post-CAS unique-target counts and bytes. Do not concurrently change matcher tables, budgets, group thresholds, anchor depth, or duplicate-hint union. Preserve this negative admission-stage result and the synthetic policy witness when interpreting a later experiment.

## Reproduction and references

Run the deterministic check from repository root:

```sh
cargo run --offline --manifest-path docs/roadmap/0.1/0.1.4/issue87-deep-diagnosis/admission-probe/Cargo.toml --target-dir /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/builds/issue87-admission-probe-target -j 2
```

Result artifact: `issue87-deep-diagnosis-bd220ae-1/admission-probe.json`, under the existing runs root. It includes source hashes and exact integers. Existing dependency warnings were emitted during compilation; the check passed, and no product files were edited.

Reviewed source anchors: `objects.rs:169`, `objects.rs:2006`, `objects.rs:2291`, `objects.rs:2956`, `objects.rs:3027`, `objects.rs:3105`, `objects.rs:3296`; `objects/admission.rs:34`, `:76`, `:118`, `:267`, `:339`, `:422`, `:443`; `objects/read.rs:23`, `:175`, `:221`; `objects/pack.rs:260`, `:410`, `:435`; `telemetry.rs:5`; production publication callers `workspace.rs:311` and `layerstack.rs:116`. Line numbers refer to the stated checkpoint.

# Deep independent review of issue87 diagnosis

Scope: additive review at discovery HEAD `bd220ae4`; original full157 Store and previously sealed reports remain unchanged. This reviewer inspected the existing canonical/pack inventory tool, report/retention code, receipt parser, source admission flow, source identity qualifications and the new `coverage_bounds.py`. No original Store opening, second payload census, replay or alternative encoding was performed. The systematic-debugging skill was applied to distinguish demonstrated defects from proposed instrumentation and unproved product causes.

## The earlier report left useful evidence on the table

The previous blanket statement that first-admitted checkpoint and no-hint byte magnitude were unavailable was too conservative. The new prefix comparison uses an actual set-inclusion proof, rather than pack-ID order or matching aggregate counts alone.

Let S_i be the selected immutable ObjectId set at historical acknowledgement i, and L_i the union of the authenticated graphs retained through i. The complete graph of any retained root must already have been admitted, so L_i is a subset of S_i. The sealed acknowledgement counter is `SELECT count(*), COALESCE(sum(canonical_length),0) FROM objects`, not a payload-only count. The new analysis establishes equality of each checkpoint’s retained-growth object count and canonical bytes with the corresponding acknowledgement increments, including Init. Therefore the cumulative cardinalities agree at every checkpoint. Inclusion plus equal finite cardinality gives **L_i=S_i**.

The required source premises are explicit: the measured public path is append-only for selected objects and immutable locators; retained ObjectIds/graph bytes are authenticated; no deletion, migration or location replacement occurred; final selected data retains every historical graph. The only discovered `DELETE FROM objects` source occurrence is a unit-test corruption/recovery setup, outside the benchmark product path. No receipt replay or prior page reconstruction is needed for this set proof.

Consequently an object’s **first-admitted checkpoint equals its first-retained checkpoint for this run**. This does not establish time/order inside a Commit or preparation attempt. A future run with transient unreachable selected objects would not inherit the conclusion; the prefix equality check must pass again. Pack admission checkpoints could only be derived additionally from source-proven atomic pack insertion and agreement of all selected records’ checkpoints, never merely monotonic pack IDs.

Every new selected payload has canonical length at most32789 B, below the optional delta eligibility limit. `prepare_missing`→`prepare_full`→`prepare_ordinary` necessarily calls `candidate()` for these payloads unless optional memory exclusion occurs; its measured exclusion count is0. At each checkpoint the new selected payload count exactly equals measured eligible count, and new DELTA count exactly equals admitted DELTA count. Each selected payload therefore accounts for one necessary eligible event; the equality leaves no additional eligible attempts in these intervals. The final705162954 canonical bytes of86417 payloads are now a source-and-receipt-derived initial eligible cohort for this run, rather than a directly recorded byte counter.

## No-hint bytes have sharp bounds, not an unconstrained unknown

`PhysicalHints` defaults to empty prior IDs and no predecessor. The only production assignment of nonempty `prior_ids` occurs inside `consume_prevalidated_pages` when a predecessor exists, after `has_predecessor` is set. Memory and spill transport preserve those fields. Thus on this measured path absent predecessor implies no prior IDs; such targets return no DELTA candidate and are stored FULL. Every selected DELTA necessarily had hints. It is valid to exclude known DELTA lengths before bounding no-hint FULL targets.

For each checkpoint with N no-hint targets and known newly admitted FULL payload sizes, the N smallest sizes give the minimum possible canonical bytes, and the N largest give the maximum. Summing checkpoint extrema preserves the checkpoint population. These bounds are individually sharp without per-target reason IDs. Subcategories must not be added as if their extrema were jointly attainable.

Reviewed outputs of the shared root-owned tool:

| Exclusive/overlapping population | Count | Lower canonical bytes | Upper canonical bytes |
|---|---:|---:|---:|
| All no-hint FULL payload targets |77949|585473955|598419082|
| Predecessor-present, no-hint FULL targets, a subset above |59360|298550682|570033483|
| Hinted targets finally FULL, disjoint from no-hint |444|145950|13091077|

These are bytes of canonical targets, not compressed savings. They establish that missing hint coverage concerns a large byte-bearing population and that improving matching on the already-hinted FULL remainder has a much smaller canonical ceiling. They do not establish that missing hints would find useful matches. New content, absent overlap, unavailable predecessor descriptors and reservation exhaustion remain causally distinct. The strongest next diagnostic is therefore narrower than the old generic funnel: explain byte-weighted terminal causes within the demonstrated predecessor-present no-hint population while preserving complete accounting of all eligible targets.

## Actionable diagnostic defects in the old draft

### 1. Unique targets and prepared attempts need different race accounting

The original draft combined a unique initially-missing ObjectId denominator with per-target bounded state and a final-admission-race override, without defining ownership across duplicate concurrent preparations. Two attempts may both probe one ObjectId as initially missing. One admits DELTA; the other loses the race. Counting attempts doubles one unique target’s bytes. Applying the loser’s override globally erases the admitted outcome. The synthetic counterexample in `review-check.py` demonstrates this without running product code.

Before any later instrumentation implementation, freeze the cohort as **(public operation, ObjectId)** and keep attempt outcomes separate. A same-operation winning admission determines the unique target’s persisted outcome; its losing sibling remains an overlapping attempt event. A target that loses to a representation admitted outside that operation has a unique race outcome. The existing admission/CAS owners should supply deduplication and winner identity. If they cannot prove a bounded unique owner, do not silently implement ordinary counters and call them unique: explicitly use bounded spillable post-ack provenance deduplication or report attempts separately with the unique denominator unavailable. No raw payload bytes or per-match logs are required.

This ambiguity does not invalidate the current run’s newly proved cohort: per-checkpoint count equality excludes additional eligible attempts in its observed intervals. It does matter to a prospective run because instrumentation can change scheduling and reveal races.

### 2. A linear funnel loses branch completeness information

A target can have partial correspondence, multiple predecessor IDs, several usable anchors, one useful raw candidate, one budget-exhausted trial and a rejected compressed mixed group. “Assign the furthest completed stage’s failure” is insufficient unless stage meaning is frozen. In particular, no-overlap requires a completed correspondence traversal with no overlap; an exhausted traversal is not evidence of no overlap. “No useful raw delta” requires all allowed reached trials completed without a useful candidate; an uncompleted trial belongs to a budget outcome, with partial-success flags retained separately.

Use the exclusive **selected terminal outcome** for conservation, and separate fixed overlapping flags for coverage incomplete, span missing, bases seen, candidate produced, trial exhaustion and mixed-group rejection. The target’s selected outcome must follow its own candidate state: being in a mixed group that wins does not mean every target is DELTA. Conversely, a raw candidate losing to the compressed margin is not a matcher failure. This is a diagnostic schema correction, not authority to raise limits or change matching.

### 3. Handoff loss must be checked before benign fallback attribution

The old source review only established that explicit memory-owned and spill transport preserve `PhysicalHints`. That does not prove every construction entrypoint attaches `first_span`. The diagnostic must check a predecessor-present eligible payload’s span at the actual candidate-delivery boundary, not infer it from the preserved field layout. A missing required span is a handoff defect, never “no overlap.” Parent-owner source tracing owns the construction-path root-cause investigation and its minimal pure-source check; this review does not duplicate that work or prejudge its result.

### 4. Per-group byte comparisons require the same candidate population

Attempted FULL and mixed bytes must be recorded as paired lengths for the same group membership; groups where B was never constructed must remain distinguishable. A/B aggregate subtraction across unmatched groups is invalid. Final persisted pack/location provenance must bind a selected winner and identify partial-pack residue or discarded preparations. Serialize information already collected by the product after the timer; do not rerun encoding or read additional payloads to reconstruct lost alternatives. This preserves the single instrumentation variable.

## Actual analysis-tool defect reproduced, with scope bounded

`issue87-analysis/trajectory.py::stats` uses a dictionary comprehension keyed by phase name. Duplicate phase receipts silently overwrite earlier elapsed/counter values. `one()` correctly rejects duplicate receipts of a singleton kind, but `stats()` lacks the equivalent guard. A synthetic pair of Commit receipts with7ns and11ns yields only11ns rather than an integrity error. This is a latent fail-closed defect in the analysis tool, not a product defect.

The smallest later correction is to assert phase-key uniqueness before constructing the mapping; repeated intentional phases would instead require an explicit sequence/aggregation schema. This review did not modify the already sealed tool. Independently checking all318 original performance/verification receipt lists (ready, closed and157 steps each) found no repeated phase names, so **the current full157 results are unaffected**. `review-check.py` is an executable defect witness rather than a regression test approving that behavior.

## Canonical decoder trust and test limits

The inventory first calls `identify_canonical`, validating complete outer framing and computing the ObjectId, before `role()` reads the kind byte. For byte objects it decodes outer byte framing before examining supported markers, and runs exact role decoders. Chunk payload containing an inode-like prefix remains a chunk. FileState and extent nodes share a family marker, but `decode_file_state` success is checked before the exact extent decoder; raw prefix alone is not accepted. Pack headers/directories, selected FULL bases, DELTA reconstruction and COPY bounds use existing product decoders. Final selected-row cardinality and complete page/pack/group/record equations cross-check the authenticated inventory.

The standalone command `cargo test --offline --manifest-path docs/roadmap/0.1/0.1.4/issue87-analysis/roles/Cargo.toml roles_do_not_inspect_user_prefix -- --exact` passes its one existing test. It does not amount to an independent malformed-encoding suite or validation of every hypothetical supported future role. The previous report correctly relies on exact current decoders plus exhaustive current-inventory checks; unsupported/invalid data would stop the scan or appear explicitly unknown. No new full payload scan is justified merely to broaden that claim.

The analysis-binary custody explicitly records that the final source added an analysis index/formatting after the executed binary. That qualification must remain; a sealed final source hash is not silently substituted for the executing binary’s identity. Product decoder source hashes are separately retained.

## Recommendation after deeper review

The new byte bounds and prefix proof materially improve the diagnosis. Amend the old report additively rather than rewriting its immutable artifacts. Keep the next action to **one byte-weighted correspondence/handoff diagnostic**, focused on the large predecessor-present no-hint population and with the race/branch-population corrections above. A separately proven handoff or reservation defect can justify a more specific single treatment only after its affected bytes and source mechanism are reconciled; do not bundle matching, codec, anchor or backend changes.

The reviewer’s product-free checks pass. Original Store contents, historical evidence, benchmark definitions and product sources were not modified. No replay, kernel trace, optimizer or candidate sample was executed.

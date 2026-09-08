# Correspondence/handoff diagnostic: concrete design and new no-overlap bound

**The remaining question is now narrower:** how much of the large predecessor-present/no-hint population is blocked by the operation allowance or incomplete handoff, rather than genuinely outside the predecessor's range? New oracle-only analysis bounds completed legitimate no-overlap at **65,446,350 canonical bytes**. At least **240,892,334 canonical bytes** require another explanation under the validated span/same-path premises below. These are affected canonical bytes, not recoverable compressed storage.

The requested fuller exploration is complete as a source/evidence investigation, executable observer/validator prototypes and a prospective implementation contract. No product instrumentation, workload replay, optimization or new performance sample was executed. Earlier sealed evidence remains unchanged.

## What the four causes now mean

| Cause | What is established | What the diagnostic must record |
|---|---|---|
|Global operation budget|All152 affected Commits saturate127×131136 reservations under16MiB; sticky exhaustion prevents further work|First actual operation-limit denial and inherited-limit counts/bytes among initially missing targets, separate from exhausted-cursor events|
|Allowance spent before CAS reuse|Hints are constructed before existing admission/CAS filtering|Grants triggered by occurrences later classified existing/duplicate/missing, with separate occurrence counts/bytes; this is not automatically avoidable work|
|Missing required span|No defect reproduced in full-file, replacement, batch or captured route; preservation alone is not sufficient proof of every future entrypoint|Predecessor-present eligible file payload with no first_span at actual delivery is an explicit handoff-defect finding; it must not become no-overlap|
|Legitimate no-overlap|For valid positive-length monotonic spans over contiguous prior extents, a nonexhausted empty result implies start at/after prior EOF|Completed empty correspondence only; exhausted/incomplete empty must use its actual limit reason|

**Important correction to interpretation of pre-CAS expenditure:** two grants triggered by a reused first chunk may initialize a cursor used by a later missing chunk. Skipping the first chunk would merely move the same two reads to the later one. The synthetic accounting witness demonstrates reused-triggered grants2 with avoidable grants0. The experiment therefore measures “triggered by reused,” never calls that quantity “wasted,” and does not forecast savings from a refund or reordered search.

## New oracle-only bound

The same frozen workload importer overwrites changed regular files at the same path, removes missing/type-transition paths, and does not create hardlinks or perform rename operations. Source predecessor selection uses the prior checkpoint's base inode or same-path base namespace. The independent full157 oracle verification already established the actual states.

For checkpoint i define G_i as the sum of positive file-length growth across same-path regular-file pairs:

`G_i = sum(max(current_file_length - prior_file_length, 0))`.

A valid whole-payload target span with no predecessor overlap must start at or beyond prior EOF. Its entire payload lies in the positive tail. Summing all file occurrences is a safe upper bound on unique targets: CAS deduplication can only reduce their total. Canonical chunk framing is21 bytes (9-byte outer header +4-byte value length +8-byte chunk family marker). Let N_i be the already proven predecessor-present/no-hint unique-target count, and L_i/U_i its canonical-byte bounds from the preceding investigation:

```text
completed_no_overlap_upper_i = min(U_i, G_i + 21*N_i)
not_completed_no_overlap_lower_i = max(0, L_i - completed_no_overlap_upper_i)
```

| Quantity across157 checkpoints | Exact integer |
|---|---:|
|Same-path positive tail growth, payload bytes|69,687,199|
|Growing file occurrences|67,733|
|Predecessor-present/no-hint unique targets|59,360|
|Prior canonical-byte lower/upper bounds|298,550,682 /570,033,483|
|Completed no-overlap upper bound, canonical bytes|65,446,350|
|Not completed legitimate no-overlap lower bound, canonical bytes|240,892,334|

The tighter final numbers sum **per-checkpoint** extrema, rather than subtracting unrelated global totals. A lower bound and upper bound from different subsets are not added as if jointly attained.

Source premises are explicit and reviewed. `build.rs::scan_mapping` supplies full chunk length and absolute scanned position. `scan_replacement_mapping_with` supplies `origin + payload_bytes_written`; `FrozenFile::mutate_existing_file` passes final-file coordinates and verifies final length. `FileMutationBatch` forwards the span-aware hook to the real store. Captured content also builds through ObjectBuffer. First-occurrence deduplication keeps one actual occurrence; it does not create a larger payload. Prior extents have positive lengths and cover the logical file contiguously. The actual cursor probe exhaustively checks small monotonic query pairs and confirms that a completed empty result cannot occur for an in-bounds span.

This bound does **not** classify all remaining240,892,334 bytes as operation-budget victims or as useful DELTAs. Missing/invalid spans and other incomplete coverage belong outside legitimate no-overlap too. The bound is invalid for an unproven span-coordinate mapping, alternate predecessor source, or another workload; the prospective run must retain those provenance checks. No original Store or payload bytes were read for this calculation: only sealed oracle dictionaries and prior aggregate bounds, with their hashes authenticated. No paths or ObjectIds are emitted.

## One chosen instrumentation design

Keep traversal, four-hint selection, all budgets, candidate order, CAS probes, matching, compression and publication unchanged. Observe existing decisions instead of performing extra searches.

At each existing `hints` call, capture the first rejected reservation guard inside the existing callback, compare cursor exhaustion before/after, and retain the first reason per cursor. The original guard precedence is memory, then file, then operation. Exhaustion arising without a callback denial is the descriptor limit. Later calls inherit the sticky reason and perform no retry. Partial hints remain valid input to ordinary matching; a limit side-label does not automatically become terminal failure.

Carry only two explicit diagnostic bytes alongside the existing hint owner: a reason/flags tag and reservation grants triggered by that occurrence. The standalone layout witness gives PhysicalHints160→160 bytes and authenticated object216→216 bytes with identical alignments on the current target. The private spill hint frame remains144 bytes and row overhead184 bytes; reserved slots carry explicitly validated values while unused bits remain strict. This is a **prospective choice, not a committed product patch**. It is admissible only if the actual product build proves the same layouts, capacities, spill thresholds, serialized lengths, canonical bytes and normal decisions. If those assertions fail on the target environment, the run is blocked; silently reducing candidate memory or changing grouping would violate unchanged policy.

At existing duplicate/discard owners, attribute that occurrence's pending grant count before the owner drops it. At the existing authoritative initial CAS probe, classify the surviving occurrence as already existing or initially missing and consume its grant credit exactly once. Preserve the causal tag through matching/final admission. Do not merge better hints, resurrect dropped candidates, add early CAS probes, refund reservations, or add a new global trace index. Rebuffering paths outside the reviewed frozen157 route must invalidate diagnostic coverage rather than invent attribution. The detailed ownership and conservation rules are in admission-design.md.

The prototype models and tests these observations; actual product instrumentation must still be implemented and reviewed before a replay can be launched.

## Receipt populations and validation gates

Keep cursor counts, descriptors and successful reservation bytes at their own work scope; they are not file logical bytes or target counts. Separately track three object populations:

1. **Occurrence work before CAS:** exclusive existing/duplicate/initially-missing routes with occurrence counts, canonical bytes, triggered grant counts and131136×grants reserved bytes. These include repeated or already stored content and are not the unique opportunity denominator.
2. **Initially missing eligible attempts:** fixed cursor/search histogram cells carry counts and canonical bytes. Cursor flags distinguish predecessor/span presence,0–4 hints, complete versus memory/file/operation/descriptor stop, and inherited exhaustion. Search flags distinguish base availability, complete/incomplete search, complete raw candidate and group selection. No timed per-path/ObjectId/match logs.
3. **Authenticated new selected winners:** authoritative new-location provenance with object count/canonical bytes and eligible FULL/DELTA count/canonical bytes, separate from attempted A/B work. Fold first/last/count of already assigned pack IDs into the receipt only after each successful transaction. Require contiguous, disjoint operation ranges under the frozen single-writer path, then join them with one final authenticated selected-locator inventory. This supplies authoritative checkpoints without timed per-object logs, new membership SQL, or per-Commit census; it is not an inference from final pack-ID order alone.

Derive exclusive terminal outcomes from these causal states rather than trusting a prelabelled “no overlap.” Missing span is explicitly reported as a defect. Partial limited hints and incomplete extra trials can coexist with DELTA admission when a complete candidate wins. No-useful-raw-delta must not mask an incomplete trial. Pair A/B bytes only for the same group membership; unattempted alternatives remain absent, not zero.

The minimal frozen157 unique-cohort gate remains retrospective. Require all158 retained-prefix/acknowledgement equalities and eligible-attempt/new-eligible-winner equalities, plus DELTA count/byte equality. If a race, extra attempt or missing observation breaks the equalities, preserve attempt receipts and report unique accounting unavailable/failed. Do not add an unbounded global dedup subsystem to force a PASS. The validator checks the entire158-row chain as well as each receipt.

`validate_diagnostic.py` is an executable **aggregate validator**, not an authenticator of arbitrary asserted manifests. Its expected identity is supplied from the independently frozen contract. An upstream reader must hash and authenticate the new-location manifests/records and original receipts; a string saying authenticated is not proof. Retained-prefix values are filled after the final snapshot from one chronological graph-union traversal. **No Store traversal or copy occurs after each Commit.**

## Prospective run contract — not executed

**One variable:** diagnostic observations enabled, with all current M4.5 policy settings unchanged. The control is the sealed accepted run; this diagnostic is unpaired and cannot establish a speedup. Before execution, freeze exact implementation/source patch/binary/image identity and demonstrate the policy-preservation checks. Current work authorizes this exploration and analysis tooling; it does not silently execute the previously excluded replay.

**Falsifiable question:** does the large predecessor-present/nohint cohort primarily report operation-limit or inherited operation-limit causes, with significant grants triggered by later reused occurrences, or does direct observation instead identify missing spans/no-overlap/other limits? The no-overlap bound is a cross-check under its stated source premises, not a compression gate. Preserve a negative result or a cohort-invalid result.

**Workload/environment:** identical ordered157 source states and manifest `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`, same oracles/input seals, ordinary whole-file Exec/Commit route, MacOS Store/coordinator/spool and Docker daemon/FUSE2CPU/2GiB/no-swap/256PID topology. Same4KiB creation, supported64KiB reading, codec/group/pack/memory/correspondence/matcher bounds and acknowledgement semantics. Keep normal identity checks strict.

**Observation:** cheap per-ack counters, original Store allocation/logical/sidecars and authoritative new pack/location provenance; detailed census only Init/final157. Collect actual observer durations and constant telemetry memory, separate host/container CPU/RSS/I/O and spool/staging/runtime/free disk. Record public Init/Exec/Commit-finalization/End, transfer, preparation, observer, work, case and invocation elapsed independently. No nested time sums or double-counted runtime allocation.

**Snapshot:** acknowledge allocation on original; close normally and record End/cleanup; seal source Store/receipts; preserve one final pre-verification logical snapshot. Census/verification use disposable copies with separate custody. Copies/clones cannot replace original allocated bytes. Preserve all originals and negative results.

**Stop:** identity/oracle/route/integrity failure, incomplete cleanup, original-bound budget breach, OOM/swap, layout/policy preservation failure, missing causal tags, grant double counting, invalid histograms, or broken unique-cohort chain. Keep partial evidence; do not change budgets/checkpoints after observing failures. Inherit300s steps,4h preparation/performance/verification,120s setup/cleanup, Store16GiB, scoped runtime/spool/staging16GiB, owned32GiB, free50GiB and host sampled RSS8GiB. Roughly30% foreground-time owner guidance concerns a later actual storage improvement, not a new diagnostic or storage acceptance gate.

## Disposition

The proposed diagnostic is now implementable and reviewable, with concrete source owners, field representation, causal classifications, a validator and executable witnesses. Its production instrumentation and replay remain unexecuted. There is no reason to expand to matcher/codec/anchor/backend experiments. Keep #87 open for this one focused diagnostic. No product change, benchmark-definition change, original-evidence rewrite, release/M5/S3 work or PR merge occurred.

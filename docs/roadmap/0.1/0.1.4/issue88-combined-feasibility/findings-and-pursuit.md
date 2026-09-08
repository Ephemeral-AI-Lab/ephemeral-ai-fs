# Combined payload and structural encoding: feasibility and selected route

**PURSUE THE COMBINATION.** All three independent reviews support a canonical-preserving native payload representation alongside S1 structural-origin deltas. Logical compatibility is strongly supported; public-path size and cost remain unmeasured. The selected implementation is payload-only version2 packs beside existing structural version1 packs, in the same SQLite Store and selected-object index. The159,163,199-byte encoded reference remains a milestone to test, not a guaranteed result or an allocated-Store forecast.

This is a source/evidence/design review of the updated #88 scope. No Store access, census, encoding, replay, build or product change occurred. The independent reports are physical.md, path.md and review.md. Source hashes and the issue snapshot accompany this report. Earlier full-run census/report finalization remains incomplete; this review does not certify it.

## Why this route is credible

Admission already separates payload and structural objects into groups (`objects/admission.rs:125–143`). Canonical ObjectIds name logical bytes, independent of their physical representation. Existing pack BLOBs and the ObjectId/length/pack/group/record locator schema can describe both formats. Existing CAS checks and transactional publication can remain the common owner. These are source-supported reasons to integrate the treatments without changing CDC, FileState, inode/namespace trees or the SQL index schema.

Keep structural S1 origin/FULL-base rules and its group codec fixed. Put independently compressed native FULL/PREFIX payload records into RAW groups in separately versioned payload packs. Avoid a second outer compression pass over those native frames. Batch records into bounded groups/packs; do not create one SQLite BLOB per chunk. Five metadata-value chunks omitted from the regular-file screen and all unsupported/non-file objects must remain represented through an explicit legacy path.

This is the minimal recommended first design, not proof that its packing is globally optimal. Separate packs simplify dispatch and accounting at the cost of possible extra headers/page slack. Measure that cost before considering mixed-format packs.

## Required implementation boundary

| Owner | Necessary work | Preserve |
|---|---|---|
|Physical format: objects/pack.rs|Explicit version2 grammar, FULL/PREFIX framing, size/window/bounds validation and version dispatch|Legacy version1 decoder, bounded group/pack containers|
|Read path: objects/read.rs|Iterative bounded actual-prior reconstruction; authenticate every base and reconstructed canonical chunk; reject cycles/missing bases|Existing locators, BLOB range access, strict legacy FULL-base rules for structural DELTA|
|Admission: objects/admission.rs|Select declared public hint; encode native alternative with complete framing; retain canonical operands for CAS checks; account successful winners and dependencies|Existing initial/final CAS checks, publication transaction, S1 policy and declared shared resource ownership|
|Delivery diagnostic: objects.rs/spill and receipts|Exact file-content cohort, existing reason/grant design, ownership/race/layout validation|Actual search decisions, ordering, budgets, spill sizes and canonical bytes|

Method ownership must be explicit where files overlap; source work may be parallel but builds and measurements remain serialized. No second database, custom persistent index, global similarity search, whole-file migration or unbounded cache is selected.

Canonical preservation does not mean old binaries can read the new packs. New readers retain version1 support; unsupported version2 must fail cleanly on old readers. Use fresh experimental Stores, not an in-place migration. A Store-wide capability marker is a compatibility design choice to freeze, not an automatic SQL redesign. No legacy decoder should be deleted while old-format compatibility is required. The experiment does not yet justify deleting the custom matcher used by structural deltas.

## A finite path to execution

1. **D: finish the bounded delivery diagnostic on accepted M4.5.** Freeze producer identity, unchanged policy and exact post-CAS file-content counts/bytes. Distinguish first/inherited limits, required-span defects and completed no-overlap; exclude structural origins and metadata-value chunks from file-span defects. Validate the158-row cohort/provenance chain. Source/format design and review can advance in parallel with D; D is not the project endpoint.
2. **Choose coverage once from D.** If delivery is sufficient for a useful encoding test, keep it. If D identifies one material defect or policy restriction, declare and evaluate that specific coverage change separately before the native treatment. Reused-triggered grants alone do not justify refunds. If the change fails, preserve it and either test native encoding with existing delivery or reject feasibility under the declared limits. Do not start another general diagnostic or silently increase budgets to reproduce offline results.
3. **Freeze P, one coherent native physical treatment.** Use existing canonical chunks, one declared actual prior from public hints, explicit codec/framing/grouping and bounded reconstruction. The prior is the actual selected object, not silently its old FULL anchor. Starting settings may reuse the fixed S2 parameters, but are prospective choices, not measured public results. FULL fallback is normal for absent/inadmissible/nonwinning candidates; corrupt identities/frames/dependencies are errors. Codec, record grammar and selected-base reconstruction form one named physical treatment; do not call it a codec-only change.
4. **Measure S1 versus S1+P.** Use the same accepted foundation and hold S1 policy fixed. The selected minimal public comparison is a fresh S1 legacy-payload Store versus the combined version2-payload Store. This measures the whole physical replacement, including framing, codec, base-policy and read costs. It does not isolate a compression knob. Follow approved smokes before a justified frozen full157 comparison.

This resolves the reviewers' control alternatives in favor of the direct whole-treatment pair. Same-envelope FULL/PREFIX checks remain useful for the new format's deterministic validation and reuse the offline mechanism evidence. Four full157 arms are not required. Without comparable four-arm evidence, do not report a statistical interaction term; report actual changes in payload and structural groups instead. If an accepted coverage change alters the foundation, both public arms must use it. Historical S1 measurements cannot substitute for a fresh timing control.

## Reconcile the milestone before judging it

The reference is exactly102,306,097 +56,857,102 =159,163,199 bytes, but the operands have different framing:

| S2 reference component | Bytes |
|---|---:|
|Native frame bytes|95,601,473|
|Prefix base identifiers|1,865,536|
|Experimental per-record headers|4,839,072|
|Standalone container header|16|
|S2 total|102,306,097|

The S1 figure is structural compressed group bodies, without outer pack framing. Build a measured bridge from these reference operands: replace experimental envelopes with actual native envelopes; add missing objects and actual group/pack framing; count all required physical records/bases once; use one production index. Do not count both diagnostic indexes or assume every removed experimental header field is unnecessary.

All physical records must be accounted for, including retained physical-only bases and unselected residue where present. Base identifiers and base records are different costs. A base record already in a lane total is not added again. Compressed group bytes cannot be proportionally attributed to records. SQLite page totals and original acknowledgement allocation are enclosing accounts, not extra numbers to add to the pack totals.

Removing the experimental envelope arithmetically yields154,324,111 known native-frame/base plus structural-body bytes, before replacement framing and omitted objects. This is only a reconciliation intermediate, not a revised target or a savings claim. Likewise159,163,199 is24,942,195 above the134,221,004 stretch goal before remaining costs; do not invent overhead savings to close that gap.

## Costs and decision gates

Public predecessor availability, first-surviving occurrence, shared admission budgets and pack formation may change either component. Keeping S1 policy fixed does not guarantee56,857,102 structural bytes. Such changes are measured interactions, not automatically correctness failures or permission to import the old result.

With maximum32KiB chunks and four prefix edges, raw closure is at most163,840 bytes; the offline1MiB raw cap is not the active bound. Public group/page reads, codec contexts, prefixes, output buffers and concurrent pending work must also fit the declared ownership. A sequential iterative chain reader is the first design; no cache is required to prove correctness. Existing fixed S2 probes show68,000 to340,125ns for one largest-closure small read, not workload percentiles or OS-cold latency. Measure matched public small/full reads and write costs.

Stop on identity/oracle/frame/chronology/closure/resource/cleanup/custody failures. Preserve unfavorable and partial results. Use the existing owner guidance of roughly30% longer foreground operations for meaningful gain with absolute-time judgment on short calls; do not invent a new storage/latency gate or erase prior smoke regressions. Record original acknowledgement allocation and one pre-verification snapshot with separate custody, observer/cache and timing scopes.

**Confidence:** high in canonical/logical coexistence; moderate in retaining substantial combined encoded benefit; unestablished for exact159.16MB encoded or134.22MB allocated achievement. Success for this pursuit is an authenticated combined representation with complete measured size/cost evidence and an honest retain/revise/reject decision. No new performance sample or implementation is claimed by this review.

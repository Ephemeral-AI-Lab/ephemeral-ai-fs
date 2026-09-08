# Independent review: pursuing the combined S2 + S1 representation

**Assessment: technically credible and worth pursuing;159163199 encoded bytes remain an unmeasured component budget, not a predicted complete Store.** The updated issue88 explicitly selects this path and supersedes a diagnostic-only endpoint. The optimal next route is a bounded coverage prerequisite followed by one matched incremental combined prototype, not a broad architecture or codec sweep.

Read the updated issue88 body and latest additive comments in full. Evidence also includes the issue88 prospective contracts, S1/S2/S3 results and prior independent review at `76b2443270e1ace1b9d6cb8c5e3fb587e55b64d6`. No original Store access, census, encoding, build, benchmark or product mutation was performed. This report is design review, not new experimental evidence.

## What can and cannot coexist by arithmetic

S2 selected content container102306097 B and offline S1 structural group bodies56857102 B sum159163199 B. Their logical populations are substantially disjoint: S2 covers86412 regular-file payload chunks; S1 covers279724 non-payload structural objects. Both preserve existing canonical units/identities for those populations. This is a strong reason a combined logical graph need not require a canonical rewrite.

But physical accounting is heterogeneous. S2 already includes16 container-header bytes,56 per-record header bytes and32 per-prefix base-ID bytes. S1's total is compressed group bodies without outer pack framing. S2 excludes five metadata-value chunks; original S1 excludes payload chunks of either use. The sum therefore omits those five objects unless separately carried. It excludes a production locator/index implementation, SQL row/page costs, sidecars and signed filesystem allocation adjustment. Its shared diagnostic SQLite is not a product index. Wrapping the already framed S2 records in new packs can add framing; replacing old framing can remove some. Neither is known until the physical format is fixed.

The canonical populations are compatible in principle, but coexistence also depends on different base rules: S1 structural origins must select FULL depth-one bases; S2 payload prefixes can reference selected prior prefixes through4 edges and1MiB decoded payload closure. Readers must dispatch by explicit record kind/role and enforce the correct policy. A generic old “every delta base is FULL” validator cannot silently validate both. Dictionaries/prefix bases must be retained and authenticated once, with base-only objects distinguished from already logical objects. Current all-history base membership is favorable; it is not a license to omit retention validation for the combined image.

The milestone is not a lower bound. A compatible format could be smaller or larger than159163199. Conversely,159163199 exceeds the134221004 stretch allocation target by24942195 B before remaining costs. The first combined milestone may succeed while the stretch goal remains unmet. Owner-selected pursuit does not require inventing unmeasured savings to erase that gap.

## Minimum attributable controls

Keep one exact canonical/state population and one fixed new physical-format reader/writer throughout a comparison. The cleanest incremental public comparison is:

- **C10:** S1 structural-origin policy enabled; payload prefix disabled, with the new physical container/reader infrastructure otherwise identical.
- **C11:** same S1 policy and infrastructure; declared payload prefix treatment enabled.

Their difference measures the incremental payload treatment **conditional on S1**. Report payload bytes, structure bytes, full representation bytes, allocation and costs for both; a change in the structural lane exposes an interaction instead of importing the old56857102 B constant. Existing public S1 controls are reusable only if exact format, harness, canonical population and other declared parameters are equivalent. If installing native-prefix support changes framing, reserve sizes or generic readers even when disabled, the old control cannot stand in for C10: obtain the missing matched control prospectively.

A full four-arm full157 campaign is not necessary for this milestone. A statistical interaction term `C11−C10−C01+C00` requires same-environment no-treatment/payload-only/S1-only/combined arms. Old offline S1/S2 figures are not those four matched cells. If that interaction number is useful, obtain the remaining controls in a cheap fixed identical-inventory offline or smoke design first; do not collect four expensive full-history runs merely to give the report a factorial label. Without those cells, honestly report incremental C11−C10 and observed lane changes, not an isolated interaction estimate.

The frozen whole-file S3 screen is already adequate to defer canonical migration: only2481574 B additional framed-content gain over S2. Repeating it is unnecessary unless the chosen representation changes a material assumption. Likewise, re-running successful decoder/inventory controls without a change or unresolved failure buys no causal information.

## Diagnostic is a bounded prerequisite, not a new programme

One accepted-M4.5 unchanged-policy file-payload diagnostic is justified because offline S2 obtained complete first-overlap candidates from authenticated source history after global CAS, whereas public correspondence currently runs before CAS under shared allowances. It resolves whether public delivery can supply the predecessor population needed by the frozen payload treatment. The diagnostic must end with a finite disposition: existing delivery adequate; one identified coverage correction required; or the current S2 opportunity not transferable under that policy.

Use exact validated chunk role plus file-content source/use, post-CAS unique target count AND canonical bytes, explicit first_span and completed/incomplete cursor reasons, and successful admission provenance. Structural S1 origins intentionally lack file spans and must never be counted as missing-span defects. Reused-triggered grants are not automatically avoidable: one reused chunk can initialize a cursor needed by a later missing chunk. Preserve occurrence/event populations and race-invalid unique-cohort evidence separately.

The already designed bounded counters, sticky cursor-reason state, exact histogram partitions and one final metadata traversal suffice. No per-Commit full Store copies/traversals, raw bytes, path/ObjectId logs in timed work or unbounded similarity index is needed. If this identifies a coverage-policy change, declare it separately before encoding; do not covertly lift a budget while claiming the sole treatment is native compression. A correct outcome can also be “adequate existing hints, proceed directly with the explicit codec/base policy.” No mandatory second diagnostic is implied.

## One explicit physical treatment and no hidden policy changes

Freeze native versioned PREFIX versus existing FULL/custom-DELTA representation, exact record/header/group organization, codec settings and allocation ceilings. Preserve canonical IDs/CDC and filesystem semantics. The payload treatment may intentionally encompass native encoding plus actual-prior selected-base reconstruction and its depth/decoded-byte policy; if so name it as that coherent physical representation change, not a codec-only tweak. Keep predecessor availability fixed to the reviewed public delivery rule.

S1 must remain policy-fixed in C10 and C11. Shared batch memory, matching budgets, read caches and optional search ordering can still interact. Either retain the current shared ownership and measure its effects or explicitly freeze a changed allocation before both arms; do not add a hidden special budget to ensure the combined arm looks like offline numbers. Do not weaken integrity/closure checks to make the new format pass old FULL-base assumptions.

Migration compatibility is an explicit decision. Old Stores stay untouched. A new Store/pack version or explicit per-record capability must cause old readers to reject unsupported prefix records cleanly; new readers may retain the legacy decoder. A successful fresh-format prototype does not imply safe in-place migration or automatic old binary readability. No separate backend is necessary merely to hold native prefix frames.

## Success, failure and read-cost standards

Hard failure remains identity/record/state reconstruction, chronology, missing/cyclic bases, declared depth/window/memory bound violation, retained-state mismatch, cleanup or evidence-custody failure. Preserve partial and failed outputs. Exact134221004 allocated bytes is the stretch goal, **not** the pass gate for the159163199 component milestone. The combined milestone reports the actual component sum and its difference from159163199, even if adverse.

No new unrequested storage or latency threshold is justified. Approximately30% extra foreground time is owner guidance for meaningful gain, with absolute-time judgment for short calls; it is not a blanket acceptance or rejection rule. Existing SDK-text recurrence regressions remain part of the evidence. Require actual read/write/CPU/RSS/I/O and temporary disk results, not a claim that a1MiB closure bound guarantees acceptable latency.

Reuse the fixed read representatives and add only cases required by new record behavior: full and small ranges through permitted chains, shared/high-fan-in bases, recurrence selecting an existing representation, missing/wrong base, truncation, depth/window overflow and cycle rejection. Whole-file threshold cases are not a new requirement for canonical-preserving chunk integration. Fresh application-decoder/cache versus warmed application cache must be labelled separately with OS cache uncontrolled. The existing selected maximum-closure read results show real amplification; they do not constitute a population latency distribution. Use matched C10/C11 logical requests, not different largest objects across arms.

Execute the three approved smokes first. Promote to one fixed full157 comparison only when correctness/cleanup pass and recorded size/cost evidence merits history applicability under the prospective decision. Retain unfavorable smoke values; a larger history can test a different size-dependent opportunity without erasing those regressions. Seal a final pre-verification logical snapshot, verify all157 states, and preserve original acknowledgement allocation as primary. Successful offline composition proves representational compatibility only; synchronous public-path evidence establishes actual product footprint and cost.

## Confidence and selected route

Confidence is **high** that the two logical populations can coexist while preserving canonical identity: they use different exact roles and individually reconstructed correctly. Confidence is **moderate** that much of their encoded benefit can survive one physical implementation: framing, origin/payload coverage and shared resource interactions remain measurable unknowns. Confidence that the first implementation reaches159163199 encoded bytes is **unestablished**, and confidence that it reaches134221004 allocated bytes is **unestablished with a known positive budget gap**.

Selected minimal route: finish existing report custody, perform the single bounded unchanged-policy file-payload delivery diagnostic, freeze one canonical-preserving native-prefix physical treatment, then compare S1-fixed C10 against combined C11 through the approved smoke-first path and a justified full157 proof. This actively pursues the combined milestone while preserving causal attribution. It neither refuses integration because its budget is hypothetical nor turns that hypothesis into an achieved size.

Primary task authority: [updated issue88](https://github.com/Ephemeral-AI-Lab/layerfs/issues/88). Existing evidence: `issue88-s1-bb36062-1/result.json`, `issue88-s2-b9abb13-1/result.json`, `issue88-s3-b9abb13-1/result.json`, `issue88-s1-full157-1/deepseek-full/performance-result.json`; source contracts `issue88-experiments/contract-v1.md`, `screen-details-v1.md`, `public-s1-contract.md`, `read-cost-contract.md`, and [prior independent review](https://github.com/Ephemeral-AI-Lab/layerfs/blob/76b2443270e1ace1b9d6cb8c5e3fb587e55b64d6/docs/roadmap/0.1/0.1.4/issue88-next-path-diagnosis/review.md).

## Normalized accounting bridge and the simplest compatible container

Retain the historical159163199 B reference, but report a bridge to a consistent native format rather than comparing heterogeneous quantities silently. S2 contains95601473 native frame B +1865536 base-reference B (=58298×32) +4839088 experimental header B (=16+86412×56). Subtracting those experimental headers gives97467009 known payload frame/base B; plus56857102 S1 structural group-body B gives154324111 known body/base B. This number is **not a new goal or complete lower-cost representation**. The56-byte envelope includes fields such as identity, lengths and closure that a valid new format may still need. The five metadata-value payload objects also remain omitted from this bridge.

For a measured combined image use:

```text
E_bodies = all selected payload representation bodies (including metadata values)
         + all selected structural group bodies
         + required physical-only base bodies not already counted in either lane
E_representation = E_bodies
                 + all actual record/group/pack framing exactly once
                 + dictionary objects not already included
                 + production index and other metadata not already included
```

If the lane totals already enumerate every required physical record, the separate base-only addend is zero by accounting definition, not because such bases cannot exist. A required base that is already selected logically must never be charged twice. Whether a base ID is classified as body or framing is a frozen convention; it occurs once. Actual SQLite logical pages and complete original acknowledgement allocation are additional nested accounts, not quantities to add to E_representation blindly. Show bridge adjustments from159163199 explicitly: remove/replace measured experimental envelopes, add measured native envelopes and omitted populations. Do not assume every removed56-byte header is free savings.

The physical owner's proposed payload-only wire2 packs beside unchanged structural wire1 packs, within existing `object_packs` and `objects`, is the simplest credible first implementation. Avoid compressing the already compressed native payload frames again at an outer layer. Extra pack headers, alignment/page slack and request grouping are real treatment costs. A separate backend or canonical rewrite is unnecessary.

The fixed-S1 control can be defined in either of two honest ways; freeze one, do not mix interpretations:

1. Same wire2 envelope/reader infrastructure in both arms, payload FULL versus PREFIX. This isolates prefix value conditional on that container, but does not itself compare the full new physical treatment with legacy payload DELTAs. Old S1 is a separate historical reference.
2. Existing S1 wire1 payloads versus combined S1 + native wire2 payload representation. This directly measures the coherent physical-replacement treatment, including codec/framing/base-policy costs. It is not a codec-only comparison. S1 policy remains fixed, but its bytes and latency may interact through shared resources and must be measured.

Option2 is sufficient for the selected integration milestone and avoids mandatory extra full-history controls. The already available same-unit offline FULL/PREFIX screen supports mechanism attribution, with its offline limitations retained. Use the same source/harness apart from the declared native physical treatment, fresh equal-state Stores, and identical public workloads. Obtain a new S1-only control if source/container/read-path differences make the old control noncomparable.

Compatibility review must include every caller of pack/header and object authentication: version dispatch cannot assume all packs use wire1; v2 reconstructs the exact21-byte canonical chunk frame before authenticating the original canonical ObjectId; CAS collision checks and canonical lengths use canonical bytes, not raw payload length. The new PREFIX chain rule must remain payload-specific, while S1 structural origins continue to require candidate FULL bases. Legacy readers reject unsupported wire2 safely; new readers retain necessary wire1 decoding. These are bounded format-integration requirements, not grounds to postpone the chosen route indefinitely.

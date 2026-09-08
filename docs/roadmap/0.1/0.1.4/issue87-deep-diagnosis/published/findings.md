# TARGET IDENTIFIED: correspondence coverage is the choke

The deeper investigation establishes **operation-wide correspondence-budget saturation**, and establishes that the missing-hint population carries hundreds of MB of canonical payload. It does **not** establish recoverable compressed bytes or authorize a product optimization. The next unchanged-policy diagnostic can now be focused on why predecessor-present targets miss coverage, rather than starting from an unbounded list of storage hypotheses.

Three independent investigations covered correspondence, candidate/admission/matching, and custody/diagnostic correctness; the parent traced span construction/transport and derived byte bounds from the existing authenticated metadata index. No original Store was opened, no payload census repeated, and no workload replay or retained-data encoding was performed. Small synthetic probes exercise the existing cursor/matcher APIs; they are algorithm checks, not benchmark samples.

## Findings, ranked by evidence and scale

### 1. Every affected Commit saturates the shared correspondence allowance

**Proven measured mechanism.** All152 Commit checkpoints with correspondence skips end at exactly16,654,272 reserved bytes:127 reservations ×131,136 bytes. That is the last possible slot under the16,777,216-byte operation allowance. The other checkpoints—1,6,12,93,121—have no exhaustion and lower reservations. Exec has zero correspondence activity. These facts are independently checked against all157 receipts with pinned source hashes.

The operation owns one shared atomic reservation counter across file producers (`changes.rs:586`, `:1260`). Each nonempty leaf predecessor normally needs FileState plus mapping, two reads. The actual cursor reproducer fully initializes63 of128 sequential one-extent predecessors; cursor64 consumes the last slot on its FileState, and all later cursors lack enough allowance. Concurrent producers may distribute partial starts differently, so63 is a capacity bound, not an assertion about measured file order.

The final all-history authenticated graph has no extent branches. Source resolves predecessor roots from the base namespace/inode graph. For roots within that retained scope, neither the4096-descriptor ceiling nor the seven-read file ceiling can bind: a leaf has at most128 descriptors and needs two metadata reads. The shared operation allowance is the directly observed limiting mechanism. Reservations are conservative work charges, not actual bytes fetched or storage waste.

**Impact:** nohint targets are not a tiny bucket; see exact bounds below. **Unresolved alternative:** some absent hints would still be useless because of no overlap, genuinely changed content or poor candidate quality. Budget saturation alone does not prove all such bytes could delta-encode. **Smallest candidate boundary after diagnosis:** correspondence scheduling at initial CAS filtering, retaining existing budgets. No cap increase, stronger matcher or new backend is justified by saturation alone.

### 2. Scarce correspondence is consumed before CAS filtering

**Source-proven ordering; workload-specific wasted fraction remains unknown.** `DeferredObjectStore::consume_prevalidated_pages` sets predecessor context and calls `cursor.hints` before sending the page to the admission visitor (`objects.rs:2036–2074`). The selected-index/CAS filtering occurs later. Hence correspondence for content ultimately reused can consume allowance and prevent subsequent initially missing targets from receiving hints. Cache hits and later CAS reuse do not refund reservations. Allocation depends on producer progress, rather than which missing payload bytes can benefit.

This is an efficiency issue, not a demonstrated data-integrity defect. Its byte scale is bounded by the predecessor-present nohint cohort, but the fraction specifically starved by pre-CAS reuse is not yet recorded. The one diagnostic should count reservations/descriptors consumed for targets later classified reused, and exclusive causes for initially missing targets. Deferring correspondence until CAS absence is known is the smallest candidate concept to investigate; preserving streaming spans/cursors and bounded memory makes that an ownership change, not a one-line budget tweak. No implementation is included.

### 3. Missing hints affect at least585,473,955 canonical bytes

**Derived, independently reviewed bounds.** The original report was too conservative when it treated all nohint byte magnitude and all first-admission checkpoints as unavailable. Existing evidence supports stronger conclusions without replay.

Let L_i be retained logical objects through acknowledgement i, and S_i the selected admitted index at i. Every retained object must already be admitted, so L_i⊆S_i. At **all158 checkpoints including Init**, retained-prefix count equals the acknowledgement selected count; canonical bytes independently reconcile too. With authenticated immutable graph data and this append-only selected index, finite-set inclusion plus cardinality equality proves L_i=S_i. Therefore selected first-admission checkpoint equals first-retention checkpoint for this run. This is not inferred from pack-ID order and does not establish intra-Commit timing.

Every new payload is within eligibility size bounds (maximum32,789 canonical bytes); recorded optional-memory exclusions are zero. Every new selected payload must visit candidate(). At each checkpoint, new payload count equals eligible count, leaving no extra eligible attempts after accounting for these necessary visits. Thus the initially missing eligible cohort is86,417 unique targets/705,162,954 canonical bytes for this run. Cross-owner payload race losses would add eligible events without selected targets; the equality excludes them here, although future diagnostics must handle them.

Known DELTA targets necessarily had hints. For each checkpoint, choose N smallest or N largest **newly admitted FULL payload lengths** for its nohint count. Summed checkpoint extrema give these bounds:

| Population | Unique targets | Lower canonical bytes | Upper canonical bytes |
|---|---:|---:|---:|
|All eligible payloads|86,417|705,162,954|705,162,954|
|No hints; FULL admitted|77,949|585,473,955|598,419,082|
|Predecessor present but no hints, subset above|59,360|298,550,682|570,033,483|
|Absent predecessor, subset of nohint|18,589|27,027,426|294,480,565|
|Hinted but FULL admitted, disjoint from nohint|444|145,950|13,091,077|
|DELTA admitted|8,024|106,597,922|106,597,922|

Bounds are individually valid; overlapping subcategory extrema must not be summed as jointly attainable values. **Canonical bytes are not compressed bytes or forecast savings.** The exact bytes assigned to each missing-hint reason remain unknown. Nevertheless, absence of hints covers a substantial byte-bearing population, while the hinted-but-FULL bucket has a13,091,077-byte canonical ceiling. Tuning matching or group rejection on that existing hinted remainder cannot directly explain hundreds of MB of unhinted targets.

All4,915 physical packs have records with the same derived first-admission checkpoint, and all366,141 physical records are selected. Together with source-proven atomic pack insertion and immutable locators, this derives pack admission checkpoints too. `derived-pack-checkpoints.csv` records that provenance; it does not reinterpret final pack-ID order as a clock. Historical page fragmentation remains unreconstructable.

An exact-source synthetic matcher probe did reproduce a bounded-quality miss: the current first-four-seed policy produces an85-byte valid DELTA where a76-byte valid witness program exists for the synthetic149-byte canonical target. This is a demonstrated heuristic tradeoff, not a measured full157 storage loss. Matching could also shorten already-admitted DELTAs, but all winning mixed encoded groups are contained within the11,639,986-byte attempted-B population. That additional ceiling does not explain the much larger nohint population. No matcher change is selected.

### 4. The planned diagnostic needs stricter accounting before implementation

**Demonstrated specification defects, not current data corruption.** A unique target cohort and a prepared-attempt cohort are different under races. Two attempts can see one ObjectId missing; one admits DELTA and the other loses. A global race override must not erase the same-operation winner or double the target's bytes. Freeze the cohort as(public operation,ObjectId), assign a unique owner/winner, and keep attempt events separate. Existing CAS/admission owners should provide deduplication; if bounded ownership cannot be proved, use explicit spillable post-ack provenance rather than mislabelled counters.

A linear “furthest stage” funnel also loses multi-base completeness. No-overlap requires completed correspondence, not exhausted traversal. No-useful-raw-delta requires completion of the allowed attempted trials; an incomplete search needs a separate budget flag. Keep exclusive selected terminal outcomes for conservation and fixed overlapping flags for partial coverage, candidate production and trial exhaustion. A/B lengths must describe the same group membership; serialize already collected values after timing, without extra encoding.

**Actual latent tool defect:** the old `trajectory.py::stats()` silently overwrites duplicate phase names. The independent witness reproduces7ns+11ns becoming11ns without error. All318 original receipt lists have unique phase names, so the sealed result is unaffected. The smallest future tool correction is a uniqueness guard; old sealed files remain unchanged. The new diagnosis reports this issue instead of silently editing historical tooling.

## Span handoff: no loss demonstrated on the measured routes

The parent traced all `put_file_payload` callers and ObjectStore wrappers. Full-file `scan_mapping` and the small `build_bytes` path call the span-aware hook. Single replacement calls that hook; `FileMutationBatch::replace` explicitly forwards to the underlying store rather than allowing its deferred wrapper's default hook to erase spans. `ObjectBuffer` attaches first_span, while memory transport preserves the typed object and spill serializes/restores start/len/prior IDs/predecessor state. Captured whole-file content also builds through ObjectBuffer before resume. Same-path replacement obtains its predecessor from the base namespace when the inode changes.

`put_authenticated` keeps the first identical object, so repeated identical chunks retain their first stored span; this is a bounded correspondence choice, not evidence of a dropped required span. Direct initialization writers use the default non-span hook, but initial construction has no predecessor. No measured-route handoff loss was reproduced. The later diagnostic must still explicitly count predecessor-present eligible objects missing required spans at delivery; transport preservation alone cannot guarantee every future entrypoint's behavior.

## Exactly one next action, now narrowed

Retain the **unchanged-policy byte-weighted correspondence/handoff diagnostic** as the single next action. Its question is now precise: **how much of the59,360-target, at-least298,550,682-canonical-byte predecessor-present nohint cohort is blocked by the shared correspondence allowance, how much allowance goes to later-CAS-reused content, and how much is legitimate no-overlap or a missing-span defect?** No separate matcher, codec, anchor or placement experiment is scheduled.

One variable remains diagnostic instrumentation. Keep all product limits and policy unchanged. For this frozen157 diagnostic, prefer the minimal cohort-validity gate: require the retained-prefix/eligible/DELTA equalities to hold again; when they do, attempt events map bijectively to unique targets without a new global dedup subsystem. If a race breaks equality, retain the attempt evidence and mark unique accounting unavailable or failed rather than inventing a unique result. General concurrent winner ownership is needed only if that gate cannot serve the intended diagnostic. Use the same frozen157 workload/oracles/environment and normal binary/image/source checks. Implement the unique-cohort/race rules above; record counts and bytes for missing span, no-overlap, operation/file/descriptor limits, unavailable bases, complete candidate, group rejection and final admission, with per-trial overlaps separate. Distinguish exact source reasons at the reservation callback rather than combining all into a single exhausted Boolean. Cheap per-ack receipts and authoritative new-location provenance suffice; detailed census remains Init/final157 only, with one final pre-verification logical snapshot and independent disposable-copy verification.

The prior proposed timing/allocation/cleanup/stop boundaries and negative-result retention remain. Product savings require synchronous public-path evidence and read/CPU/memory cost; owner roughly30% foreground-time guidance is not a storage gate. No replay is executed in this deeper source/evidence diagnosis, and the original experiment remains admission-ineligible for release/M5 claims.

## Deliverables and custody

- `coverage_bounds.py`: read-only metadata/receipt proof and bounds; scalar sizes bounded by one checkpoint, no retained payloads.
- `correspondence-receipts.py`, `.json` and `correspondence-probe`: original receipt saturation checks and actual-cursor synthetic probes.
- `admission.md` and `admission-probe`: independent candidate/matcher population review and synthetic checks.
- `independent-review.md`, `review-check.py`: independent proof audit, diagnostic race/branch defects and old reporter defect witness.
- External new directory `layerfs-storage-v3-runs/issue87-deep-diagnosis-bd220ae-1`: byte bounds, derived pack checkpoints and final additive seal. The earlier analysis directory and all original Stores/receipts are preserved unchanged.

This addendum supersedes only the earlier overly broad uncertainty about checkpoint admission chronology and aggregate nohint byte magnitude. It preserves the essential limitation: **useful missed compressed savings are not yet measured**. Keep #87 open for the narrowed diagnostic. No product fix, replay, recompression, benchmark change or PR merge occurred.

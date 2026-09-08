# C: fixed operation correspondence allowance

Prospective contract, frozen before C builds or samples. Worktree
`layerfs-issue88-coverage`, branch`codex/issue88-correspondence-coverage`, starts
from accepted-M4.5-plus-D source`265cfe9632f17b4bed652f477c20327d7dc19919`.
Parent confirms D all157 verification/cleanup and the unique158-row graph gate
PASS. Original D inputs, Store, snapshots, receipts and failed preparations remain
immutable. C output is exclusively created under the existing runs root.

## One treatment and falsifier

Change only the operation correspondence reservation ceiling from16777216 to
1073741824 bytes. The named constant is
`CORRESPONDENCE_OPERATION_RESERVATION_BYTES` in`objects.rs`; only the existing
checked atomic reservation predicate uses it. This is cumulative optional work,
**not1GiB resident allocation or cache**. No other16MiB limit changes.

Keep131136 bytes per metadata reservation,1MiB per-file allowance,4096 cursor
descriptors, actual file-source markers, D counters, pre-CAS delivery, first-owner
deduplication, predecessor selection, memory/queue/worker bounds, canonical
construction/IDs, legacy FULL-anchor matching, codec/group/pack format, reader,
SQL publication and public acknowledgement behavior unchanged. No S1 or native
prefix P is bundled. The saturation test derives its last admissible reservation
from the actual named constant instead of hardcoding the old127 grants.

D observed a maximum3763 attached predecessor cursors in Commit157. The shared
authenticated prior-root graph has no extent branches, so each attached cursor
needs at most FileState+leaf, two reservations:

`3763 * 2 * 131136 = 986929536 bytes`.

The fixed1GiB ceiling permits8188 reservations/4094 two-read cursors, exceeding
that bound by86812288 bytes (about8.80%). This source-bounded workload model,
not a parameter sweep, selects the single value. D measured59219 unique initially
missing file payloads /489185498 canonical bytes with operation-limited empty
hints. These are affected logical bytes, not predicted physical savings.

Falsifiable prediction: under the same flat-root and cursor population,
operation-limited empty coverage disappears without being displaced into
file/descriptor/memory exclusions. More hints need not produce useful deltas;
the unchanged legacy matcher/anchor/group outcomes remain part of C's measured
result. Continued exhaustion means the prediction or its population assumptions
failed; do not increase the cap during this experiment.

## Cost acknowledged prospectively

The allowance is64 times D's old cap. Across D's134997 attached cursors, the
two-grant bound permits269994 lookup reservations versus19638 granted in D:
up to13.75 times the observed granted work. Reservations are not actual physical
reads, decoded bytes, live memory or allocated Store bytes. Existing caches and
sharing affect actual work, which must be measured. Grants triggered by eventual
CAS reuse are not automatically avoidable; no refund or post-CAS deferral occurs.

The owner permits consideration of roughly30% longer foreground calls for
meaningful storage gain, with absolute-time judgment. This is not automatic
acceptance of a64-fold allowance, a promise of equal RSS or permission to exceed
hard execution bounds. Preserve negative cost results and the baseline suite's
independently reproduced failures; no assertion or resource limit is weakened.

## Source review, tests and serialized preparation

Before build, review the exact diff: one production constant/predicate change,
the existing real-cursor endpoint test update. The arithmetic is retained in
this contract and source review instead of a test that merely restates constants. Existing
compile-time charged layouts remain enforced. Run focused diagnostic tests in a
parent-granted serialized slot and preserve all output. Product source and
dependency code outside this treatment stay unchanged.

Freeze exact C source/product/dirty-patch, executable/image/toolchain, workload,
input and codec identities after compilation and before samples. Reuse the
validated D executable/image only when normal identity checks pass. Build C
through the unchanged normal host/image workflow with two Cargo jobs and900s
build limit. Record exact commands, arm order, output paths and identity hashes
in an immutable pre-run manifest before executing each stage. No build or
measurement runs concurrently; use the existing measurement lock.

## Three approved smokes first

Use the existing DeepSeek-five, frequent-edits and small-files smokes, with their
original data, seeds, routes, normalization, oracle and timer definitions. Freeze
one initial fresh D/C pair per approved smoke in order: DeepSeek-five D then C,
frequent-edits D then C, small-files D then C. Each run owns a fresh Store and
performs its normal independent historical verification and cleanup. These are
development/applicability pairs, not the original three-pair final qualification.
Retain all raw values and original numerical gate comparisons.

No C full157 run launches automatically after smoke correctness. Record an
explicit cost/usefulness decision after the three pairs. The decision must
consider measured added correspondence/read/match/encode work, actual allocated
and logical size, absolute and relative public-call costs, observed peaks and
remaining uncertainty about history-sized operations. If smokes reject usefulness
or leave an unresolved correctness/resource failure, stop C. Do not discard or
repeat a slow arm alone to obtain a favorable comparison.

## Conditional full157

Only after that written decision, freeze one full157 C performance plus all-state
verification run using manifest
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`, the same ordered
157 source states and ordinary public import/Commit routes. Historical D full157
receipts are descriptive controls, not a fresh matched timing pair. If a new
paired timing claim is required, prospectively obtain its missing control;
do not relabel the historical timer. No new benchmark family or seed.

Retain2CPU/2GiB/no-swap/256PID container profile, host ownership of Store/SDK/spool,
300s per public step,4h preparation/performance/verification,120s setup/cleanup,
Store<=16GiB, scoped runtime/spool/staging<=16GiB, owned output<=32GiB,
host-free>=50GiB and sampled host RSS<=8GiB. Normal lock, identity, filesystem
and cleanup rules remain strict. No timeout extension or resource relaxation.

Measure exact D cohort counters, attached/query/grant maxima, first/inherited
causes, candidate/base/budget/mixed outcomes and admitted FULL/DELTA bytes/counts.
Require the same ownership and158-row count/byte conservation gates, authoritative
successful pack ranges and selected-locator proof. Observe preparation, transfer,
Exec, Commit/finalization, End, observer and enclosing wall scopes separately;
retain host/container CPU/RSS/I/O, spool/staging/free-disk and cache qualifications.
Nested codec/matcher times are not added to public elapsed.

Record original acknowledged Store allocation and sidecars as primary. Preserve
one final pre-verification logical snapshot, seal it and receipts, and use the
normal verified custody process for final census/historical verification. Copies
preserve logical content, not allocation equivalence. No whole-Store traversal or
copy after every Commit, and no change to original D/issue87 manifests.

## Stop, retain and next decision

Identity/oracle/canonical reconstruction, ownership/provenance, resource or cleanup
failure stops the affected run and preserves partial evidence under its own
identity. An instrumentation/correctness correction requires a new explicit
source identity before a justified rerun. Ordinary FULL fallback, continued
limited coverage, insufficient storage gain and unfavorable timings are retained
results, not authority for a larger cap or bundled treatment.

Retain or reject C from the measured coverage, storage and foreground/read costs.
Successful C supplies a frozen public predecessor-delivery foundation for P;
it does not implement actual-prior native chains or reproduce S2's offline
first-owner manifest. If C fails usefulness, either test P with original delivery
when still justified or reject the constrained milestone. No diagnostic loop,
whole-file rewrite, in-place Store migration, release/M5 claim, rollout or merge.

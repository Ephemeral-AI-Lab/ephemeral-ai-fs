# One C experiment: bounded operation correspondence allowance

Implementation handoff update: the parent subsequently confirmed D all157
historical verification/cleanup and the unique158-row graph gate PASS. The text
below preserves the original pre-verification recommendation and its conditional
language; `contract-v1.md` freezes the current C plan. No C sample is implied.

**Recommend changing only the operation correspondence reservation ceiling from
16,777,216 bytes (16 MiB) to 1,073,741,824 bytes (1 GiB).** Retain the 131,136-byte
per-fetch reservation, 1 MiB per-file ceiling, 4096-descriptor cursor bound,
producer memory/queue limits, pre-CAS placement, first-owner ordering, legacy
matcher/codec/base policy and all public safety bounds. This recommendation is
conditional on D's remaining all157 historical verification and cleanup passing;
it authorizes no C execution or code change in this review.

The larger number is a **cumulative optional-work allowance**, not a 1 GiB
allocation or cache. It is nevertheless a material resource-policy change:
64 times the old allowance. It must be named and measured as C, separately from
native-prefix P. It is not an unchanged-policy diagnostic or a storage forecast.

## Evidence scope and exact demand calculation

Source evidence is the completed D performance receipt:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-D-full157-1/deepseek-full/performance-result.json`,
SHA256 `3f4b367a4638ee418ee93da89cf2c7145f605d80a7a63fc295692bab728f0ba4`.
I read its full parsed receipt population and derived the quantities below.
There are157 public Commit intervals; all nonzero correspondence work is in
Commit. Counts are interval observations, summed only once. No Store or payload
was scanned or decoded by this review.

Parent's completed graph validator validates all158 Init/checkpoint cohort rows,
with86412 unique file-payload targets /705162813 canonical bytes. Parent supplied
`issue88-diagnostic-custody-1/D-roles-summary.json` from the shared authenticated
inventory. It contains75927 FileStates and75927 extent leaves, **no extent
branches**. This is a logical graph-shape fact, not a compressed-byte allocation.
Final historical verification was still in progress at recommendation time.

| Quantity | Whole-run sum | Maximum one operation | Maximum location |
|---|---:|---:|---|
|Attached predecessor cursors|134997|3763|Commit157|
|Actual cursor queries|170877|4627|Commit157|
|Queries inheriting exhaustion|27826|794|Commit157|
|Granted metadata reservations|19638|127|First maximum Commit2|
|Operation-limit exhaustion events|125254|3700|Commit157|
|File/descriptor/memory limit events|0 /0 /0|0 /0 /0|All phases|

An attached cursor is an existing producer/candidate-delivery owner with a
predecessor, not a unique target and not necessarily a cursor that needs a read.
A file can issue multiple queries. Therefore `queries*reservation` is not the
right capacity model. Likewise observed grants are censored by the old cap and
cannot estimate unconstrained demand on their own.

The source plus authenticated graph supplies a conservative bound:

- `changes.rs::StableFileInputs::prepare_page` resolves the predecessor from the
  base inode table or same-path base namespace; `FrozenFile::build` attaches that
  existing FileState. Under this frozen ordinary full157 workload these are the
  retained prior-state graphs, not future or reconstructed hypothetical roots.
- `PredecessorCursor::next_extent` reads the FileState once, then its mapping.
  The authenticated retained mappings are single leaves, with at most128
  descriptors. Further monotonic queries reuse the loaded leaf/frontier rather
  than refetching it.
- Consequently **at most two metadata grants per attached cursor** suffice for
  the predecessor lookup in this workload. Empty or past-EOF cursors may use
  fewer. The seven-grant per-file bound and4096-descriptor bound do not bind on
  these graphs.
- Use the largest measured attached-cursor count, not an average:
  `3763 * 2 = 7526` grants and `7526 * 131136 = 986929536` reserved bytes.
- A 1 GiB ceiling permits `floor(1073741824 / 131136) = 8188` grants, sufficient
  for4094 completely initialized two-read cursors. It exceeds the measured
  conservative operation bound by86812288 bytes, approximately8.80%.

Thus 1 GiB is a simple fixed power-of-two ceiling just above source-bounded
observed demand. It is not an arbitrary sweep point. The calculation is
workload-specific: do not claim it covers every future file graph or operation.
If the actual C run exhibits more cursors, branching/transient predecessors or
continued operation exhaustion, retain that discrepancy as a failed prediction;
do not automatically raise the ceiling again.

## Why this addresses a material observed bucket

D reports59219 initially missing eligible targets /489185498 canonical bytes
with empty hints due to the operation allowance. Of these:

- First exhaustion:51390 targets /379852297 canonical bytes.
- Inherited exhaustion:7829 targets /109333201 canonical bytes.

Their sum is exact and approximately69.37% of the validated eligible canonical
bytes. Missing required spans are0. Completed benign no-overlap accounts for
1084572 canonical bytes. This is now byte-weighted evidence of a concrete public
coverage barrier, not an inference from a low DELTA count.

**489185498 affected canonical bytes are not recoverable physical bytes.** Extra
hints may refer to unhelpful content, inadmissible bases, older FULL anchors or
candidates rejected by the existing matcher/group decision. C does not change
those policies. It measures how much source-bounded correspondence survives into
usable candidate and final-admission populations, and its actual storage/cost
result. The S2 offline102306097-byte container is not a C forecast.

## Why not move correspondence after CAS first

D classifies8165 of19638 grants (about41.58%) as triggered by occurrences whose
initial CAS probe later found an existing selected object;11453 grants were
triggered by missing occurrences and20 by duplicates. These are trigger labels,
not avoidable-work quantities. A reused first chunk can pay two startup reads
that serve a later missing chunk without another grant. Removing that first
query can merely move the same startup reads to the missing chunk.

Post-CAS deferral also crosses producer/consumer ownership: the current cursor
and reader belong to file delivery, while the authoritative initial probe happens
later in CheckedOutputAdmission. Deferral requires moving or retaining per-file
root/span/cursor context across bounded slabs and deduplication, with new ordering,
lifetime and accounting decisions. It is a larger change than one ceiling and
cannot be justified as reclaiming41.58% of reservations. Even a maximal accounting
refund does not demonstrate enough coverage for thousands of files under the
existing127-grant ceiling.

C therefore tests the simpler mechanism first: keep ownership and semantics intact,
provide a fixed sufficient cumulative allowance, and measure the cost. If its
cost is unacceptable, retain that result; a later source-supported ownership
optimization may be considered separately, not silently added to this arm.

## Foreground and memory consequences

The per-file cursor/read scratch remains the existing576 KiB reservation within
its producer partition, with the required residual memory and unchanged queue/
worker limits. The operation counter does not retain all predecessor payloads
or allocate memory in proportion to the 1 GiB number. Existing bounded snapshot
cache, group fetch, read validation and encoder ownership still apply. Raising
cumulative work can nevertheless increase CPU, I/O/cache pressure and lifetime
peaks; an unchanged per-owner bound is not proof of unchanged process RSS.

A whole-run source bound is `134997*2 = 269994` metadata lookup reservations,
versus19638 granted in D: up to250356 additional reservations and **13.75 times
D's observed granted lookup count**. This is not a prediction of physical disk
reads: existing caches and repeated group access matter. The associated total
35,405,933,184 reserved bytes is a run-wide sum, never simultaneous memory or
Store allocation. The largest per-operation allowance is64 times the previous
cap; neither number should be hidden behind the word “bounded.”

D public Commit elapsed sums to42826245585 ns; maximum is1062291583 ns at
Commit155. These are descriptive D observations, not new timing samples. Extra
metadata reads and additional existing matcher/encoding trials can materially
increase foreground time. The owner permits consideration of roughly30% longer
foreground operations for meaningful storage gain, with absolute-time judgment;
that guidance is not an automatic acceptance or a permission to overrun frozen
resource/time gates. A single ceiling change can be correct and still fail the
cost/benefit test.

## Prospective C contract and decision gates

1. **Gate D:** require all157 verification,158-row cohort/provenance conservation,
   final snapshot identity and cleanup PASS before C. Keep the separately reported
   baseline Store-library failures visible; do not repair or waive them in C.
2. **One source change:** modify only the operation-reservation predicate in
   `objects.rs::consume_prevalidated_pages` from16 MiB to1 GiB. Do not replace
   other16 MiB constants (matcher/read/batch limits). Keep D diagnostics enabled
   for equal observation scope and freeze source/product/binary/image/contract.
3. **Small smoke first:** use the existing approved smoke workflow with the same
   seeds, fixture paths, import routes and timers. Obtain missing comparable
   D-control samples prospectively; existing valid controls may be reused with
   their exact qualifications. Localized edits, recurrence, shared content and
   integrity/cleanup must pass before full157. No parameter search or repeated
   favorable sampling.
4. **Full157 when justified:** same frozen ordered workload/oracles/environment,
   fresh Store, serial measurement lock, host/container topology. Preserve300s
   public step,4h preparation/performance/verification,120s setup/cleanup, existing
   Store/spool/runtime/output limits,50GiB host-free reserve and8GiB sampled host
   RSS bound. No timeout extension or memory-limit change to rescue C.
5. **Measure:** repeat exact post-CAS count/byte reasons and first/inherited limits,
   attached/query/grant counts, useful bases, candidate/budget/mixed outcomes and
   final FULL/DELTA selections. Separate preparation, transfer, public Exec,
   Commit/finalization, observers and wall time; retain host/container CPU/RSS/I/O,
   cache qualifications, spools, complete acknowledged Store allocation and
   sidecars. Preserve one final pre-verification logical snapshot; census/verify
   with correct separate custody and original allocation as primary.
6. **Falsifier:** under the flat-root/attached-cursor bound, operation-limit empty
   bytes should disappear without moving the same targets into file/descriptor/
   memory exclusions. If coverage remains limited, explain the violated bound
   rather than raising the cap. If coverage improves but current matcher/anchor
   policy gains little, retain that valid result; it still measures the bridge
   for P. If work/read/foreground costs outweigh the benefit under owner judgment,
   reject C and either test bounded P with existing delivery or reject the
   proposed milestone under those limits.
7. **Stop failures:** identity, canonical/oracle reconstruction, cohort ownership,
   malformed dependency, resource or cleanup failure stops the affected run.
   Ordinary FULL fallback and a valid size/cost miss are results, not reasons to
   mutate policy or discard evidence.

## What C enables for P

Successful complete correspondence makes the same *candidate class* available as
S2: a prior same-file overlapping chunk, from actual retained history. Public
first-owner scheduling and up-to-four hints still differ from offline path order;
C does not reproduce S2's candidate manifest exactly. Current legacy matching still
substitutes FULL anchors for DELTA hints. Actual-prior reconstruction and native
prefix records remain the separately frozen P treatment. Keep this accepted C
foundation fixed for subsequent S1-only and S1+P comparisons; do not add their
independent component figures and call159163199 encoded bytes achieved.

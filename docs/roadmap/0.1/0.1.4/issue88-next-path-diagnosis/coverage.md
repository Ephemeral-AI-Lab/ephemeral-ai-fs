# Next-path coverage review

Read-only source/report review at issue88 checkout HEAD
6f1c120c68f1532a88eb866c2af593b74c3cc772. Existing uncommitted report edits are
preserved. No Store, census, replay, build or encoder was opened/executed.
Reviewed the complete issue88 contract-v1, screen supplement, public-S1 and
full-history continuation contracts; current issue88 findings; prior issue87
correspondence and admission designs. Applicable repository AGENTS found only
under benchmark; no benchmark files are touched. Applied systematic-debugging
source-flow investigation and the existing Ponytail minimal-change principle.

## Principal finding

The S2 offline result establishes substantial representation potential but does
not establish that its exact base choices reach the current public encoder.
There is enough source evidence to identify the bridge that needs measurement:
file predecessor -> span/cursor -> first surviving CAS representative -> actual
selected base. Public correspondence is globally capped before CAS filtering;
S2's offline source-manifest predecessor lookup has no such cap and uses a
different first-occurrence order. A codec-only patch cannot claim the offline
102306097-byte framed image.

## Current call path and budget proof

- `crates/layerfs-workspace/src/changes.rs:586` creates one shared atomic
  correspondence allowance for the candidate build. `:1075` resolves predecessor
  inode/file roots from base inodes, then base namespace path lookup when needed;
  `:1180` saves a regular-file root, `:1238` decodes it, and `:1461` attaches it to
  the file ObjectBuffer. Same-path replacement can obtain a legitimate old file
  even when the newly created inode lacks a direct canonical predecessor.
- `objects.rs:2713` sets `first_span` when actual file payloads are constructed.
  Private spill saves/reloads it at `objects/spill.rs:469` and `:590`. There is no
  reproduced missing-span handoff defect in this reviewed route.
- `objects.rs:628` `send_selected` invokes `consume_prevalidated_pages` at `:637`.
  Its `:2047–2073` computes hints before handing pages/slabs to admission. The
  first CAS probe is later, `CheckedOutputAdmission::probe_incoming` at `:2978`,
  with known IDs looked up at `:3017` and missing objects pushed at `:3039`.
- Each optional metadata fetch consumes131136 reserved bytes. The1MiB file cap
  admits7 grants;16MiB operation cap admits127 grants. Current retained single-
  leaf predecessor graphs need FileState+mapping =2 grants. At most63 such file
  cursors can initialize completely under one saturated shared operation.
  Concurrent producer progress distributes those slots; it is not a deterministic
  first63 paths policy. The allowance is not actual I/O bytes or live memory.
- Prior sealed receipts establish152 exhausted full157 Commits all at exactly
 16654272=127*131136 reserved bytes;125254 events are exhausted cursors, not
 missing unique targets. The source choke remains in the current code.
- `objects.rs:2436–2437` separately reserves576KiB cursor/read memory and requires
 32KiB remaining. Memory exclusion is another reason, not global capacity.

The pre-CAS design can spend grants on occurrences later found to exist or be
duplicates. However, a reused first chunk may initialize a cursor that serves a
later missing chunk. Therefore **triggered by reused content != avoidable work**.
Simply refunding those grants is not a justified optimization.

## Missing span, no overlap and incomplete traversal

`file/rope/read.rs:394` rejects zero/reordered spans. `:399` permanently returns
empty after exhaustion; `:419` stops at4096 descriptors. The callback failures
at `:442`/`:472` also set exhaustion. A valid nonexhausted contiguous predecessor
with a positive in-bounds monotonic target must contribute a first hint: repeated
IDs cannot suppress insertion into the initially empty four-slot list. Thus
**complete empty correspondence implies start>=old EOF**, under exact file-span
provenance. Missing spans and limited traversals are not legitimate no-overlap.
Partial hints can survive a limit reached during the same call; they may still
lead to a selected DELTA and must not be counted as terminal budget failures.

There is a new S1 qualification absent from the older issue87 design:
`objects.rs:2720–2729` intentionally assigns inode-leaf origin hints and
`has_predecessor=true` without a file span. `admission.rs:455–472` now counts both
inode leaves and chunks as eligible. A diagnostic that labels every eligible
`has_predecessor && first_span==None` as a handoff bug would falsely report valid
structural origins. **Filter the payload cohort using exact canonical role
decoders and keep S1 structural-origin counters separate.** Metadata-value chunk
encodings also need explicit source provenance; their absent file span is not
automatically a file-payload defect.

## Why S2 candidates are available in principle, not proven delivered

`issue88-experiments/extract_content.py:43` sorts input paths. `:29–37` selects
the first global content occurrence and checks an earlier selected checkpoint.
`:51–53` picks the first overlapping old same-path chunk. These are legitimate
past-content candidates and require neither future history nor global similarity.
The product knows corresponding old file roots and can derive those IDs.

But the offline candidate pipeline differs materially:

1. Offline CAS filtering happens before optional correspondence; public hint
   work happens before CAS and competes for127 metadata grants.
2. Offline first occurrence is checkpoint/path/chunk ordered. Public dirty inode
   tasks/producers and first surviving hint ownership can pick another occurrence
   of the same content. Current duplicate paths retain first hints, not merged
   best hints. A shared chunk may have different valid predecessors by path.
3. Offline chooses one first overlap. Public returns up to four overlaps and
   tries candidates under different limits; this can help some targets but cannot
   recover a missing cursor result.
4. Offline uses the actual prior selected representation with up to4 delta edges.
   Public chunk matching at `admission.rs:492–496` replaces a DELTA predecessor
   with its FULL anchor. Native-prefix+actual-prior changes both codec and physical
   base/dependency policy. It is not a matcher-only or level-only treatment.

The offline58,298 selected prefixes are not58,298 established usable public
predecessors. The public-delivered subset's counts, canonical bytes, selection
results and cost remain unmeasured. S2 includes86412 file-content units; older
86417 payload figures additionally include5 metadata-value chunks.

## Minimum unchanged-policy diagnostic

Reuse the existing issue87 two-byte design, adapted for the current role split:

- Per existing cursor, capture the first rejecting callback guard in unchanged
  short-circuit order memory -> file -> operation. Compare actual cursor counters
  before/after: a new exhaustion with no denied callback is descriptor limit.
  Keep that reason sticky; inherited exhaustion performs no retry.
- Carry a reason/inherited tag and successful grants count0..7 in two explicit
  fields whose actual compiled size/alignment/capacity must remain unchanged.
  Earlier synthetic witnesses found PhysicalHints160 and containing object216
  bytes unchanged. Recheck actual current types, not assumed padding. Private
  spill uses bytes13/14, byte15 remains strict zero;144/184-byte framing stays.
- At initial CAS absence, independently of optional search-memory gating
  (`admission.rs:176–183`), count exact eligible file-payload targets and full
  canonical bytes once in the existing admission owner. Partition into no
  predecessor; missing required span; complete empty; limited empty; complete
  with hints; limited with hints. Add fixed stop/hint-count/size bins, no timed
  paths/ObjectId dumps or extra matching/decoding.
- Attribute triggered grants through existing preexisting/missing/duplicate
  branches once, before discarded metadata disappears. Clear grants after
  accounting; preserve cause to terminal classification. Unclassified failure
  residuals are null/invalid, not zero.
- Preserve terminal attempt outcomes through prior/base/search/mixed selection
  and successful publication. Partial hints continue normally; final late-race
  attempts are separate. Persist cheap transaction pack ranges after commit and
  validate selected locators once at a final pre-verification census.

Cohort gate: attempt count/bytes equals newly selected eligible count/bytes;
zero late-race attempts; source ownership proves one eligibility event per
selected eligible object; all role/transport/pack-range/locator provenance checks
pass. Equality of totals alone is insufficient. If the gate fails, preserve
attempt observations but mark unique terminal attribution unavailable; do not
add a global tracking subsystem just to force a complete table.

## One next feasible action

**Implement and review that unchanged-policy payload coverage diagnostic on
accepted M4.5 `eb7050603`, not on the current S1 candidate,
before claiming S2's offline gain is reachable from the public path.** The
immediate deliverable is the minimal instrumentation patch and deterministic
handoff/layout/duplicate/race tests; its separately frozen public full157 run
then measures the byte-bearing missing/capped subset and delivery costs. No codec,
base budget, ordering, canonical unit or extra search changes belong in that run.

This is a bounded bridge to the already justified native-prefix direction, not a
return to undirected analysis. It decides whether the first public native-prefix
prototype can retain existing correspondence or must declare a separate coverage
treatment. The whole-file rewrite is weaker after S3 measured only2481574 more
container bytes saved; no existing evidence justifies skipping public coverage
and importing offline path-ordered candidate manifests as a product feature.

This baseline choice preserves applicability of the original M4.5 cohort bounds
and isolates instrumentation as the sole treatment. S1 changes eligible role
populations and shared search/group work; silently making it the control would
conflate that change with the diagnostic. Exact payload role/source filtering is
required even on M4.5 to separate file-content chunks from metadata-value chunks.
If a later investigation intentionally uses S1, freeze that as a separate
diagnostic control, keep origin-bearing inode leaves outside the file-span
denominator, and do not reuse M4.5 numeric cohort equalities without validation.

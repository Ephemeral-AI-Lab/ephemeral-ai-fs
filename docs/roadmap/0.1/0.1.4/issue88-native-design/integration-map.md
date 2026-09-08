# P integration map — source design only

D source is frozen while the parent builds the normal host. This is read-only
interface tracing in`layerfs-issue88-diagnostic`; no P implementation, build,
Store read, census or encoding occurred. Physical codec/grammar settings are
owned by the parallel physical-design review. Candidate-dependent choices wait
for the bounded D/C decision.

## Every production parser call site

The production `crates/` search finds one external header caller, one external
directory-entry caller and three record-stream callers, all in
`layerfs-layerstack-store/src/objects/read.rs`:

| Function / location | Existing call | Required v2 handling |
|---|---|---|
|`extract_hint_group`,`:187`|`pack::header(&header,length)` returns group count only|Obtain explicit version plus count before allocating or reading the selected entry; preserve strict v1 acceptance and reject unsupported versions.|
|`extract_hint_group`,`:193`|`pack::entry(&directory,count,length)`|Pass/retain version so v2 RAW outer-group semantics cannot enter the legacy oversized-singleton path.|
|`read_hint`,`:263`|`pack::visit_records(&decoded,false,...)`|Keep legacy hint/anchor behavior for S1/custom DELTA; add a separate actual-prior path for native payload candidate selection.|
|`visit_wave` target pass,`:438`|`pack::visit_records` emits FULL or stores pending custom DELTA|Dispatch v2 records to bounded canonical reconstruction; preserve complete group parsing and selected-record coverage checks.|
|`visit_wave` base pass,`:512`|`pack::visit_records`;`:516` insists `Record::Full`|Do not weaken this invariant globally. Legacy custom DELTAs continue to require their existing FULL base; native dependencies use the explicit native reader.|

Inside `pack.rs`, `visit_records` calls `record` at`:238`; there are no direct
production calls to `pack::record` outside that module. `header` presently rejects
all versions except1. `entry` permits RAW or Zstd outer groups and the legacy
oversized RAW exception. Version2 must therefore be an explicit dispatch, not
reuse kind1 with a different payload and hope existing callers understand it.

Some analysis tools include`pack.rs` by path. Their existing sealed sources and
binaries are historical evidence, not production dispatch owners. New native
census tooling must use the new version-aware parser; do not alter/rebuild sealed
old census tools in place. A thin explicit `Header{version,groups}` or equivalent
parsed result at the existing physical boundary is sufficient; no generic format
registry or new backend is needed.

## Reader split with minimum disturbance

`extract_group(:171)` delegates to`extract_hint_group(:176)`; this common extraction
owner performs selected directory bounds checks, connection-scoped BLOB reads and
optional read-budget charges. Carry the checked version into its returned entry.
New v2 ordinary groups remain bounded RAW containers of native frames; their
directory decoded length describes the RAW record stream, not reconstructed
canonical chunk length. Keep those two lengths distinct in validation/accounting.

Add one iterative native/actual-prior reconstruction helper at this boundary.
Input: selected canonical ObjectId/location and explicit required/optional read
mode. Output: authenticated canonical bytes plus validated native depth and
cumulative raw-payload closure facts. Required reads treat corruption as errors;
optional candidate reads may return unavailable for an explicit budget or
unsupported-base policy, never turn malformed data into ordinary FULL fallback.

The helper needs bounded per-chain IDs/locators/frames, cycle and depth checks,
raw and canonical output-length checks, exact chunk-role decoding, native frame
validation, canonical reconstruction and ObjectId authentication. Start with
serial alternating base/target buffers, no speculative persistent cache. Count
both group fetch/decode costs and actual native reconstruction work. Four32KiB
delta edges imply163840 raw closure bytes but do not bound enclosing groups,
codec workspace or concurrent validation-wave ownership.

Keep the existing v1 `visit_wave` target/base batching for legacy records. For v2
targets, the ordinary public `visit_locations(:382)` route must emit canonical
objects through the same callback contract, including CAS comparison reads.
When selecting a target from a decoded group, fully validate the group stream
before treating its result as trusted; unrequested malformed records and duplicate/
missing locators must not become invisible. Avoid retaining a full group while
fetching a chain unless that overlap is explicitly in the memory budget.

`validation_reserve(:67)` and`visit_locations` currently split requests under the
legacy fixed validation reserve. Native frames cannot silently exceed that bound.
The integration owner must recalculate the native worst case and use a safe
per-request drain strategy; do not silently lower shared capacities or hide
additional allocations. The physical-design reviewer supplies static codec
workspace requirements and frame bounds for this calculation.

Legacy oversized `singleton_range(:293)`, `singleton(:328)` and
`compare_singleton(:352)` remain v1-only. Native file chunks are<=32768 raw bytes
and do not need the multi-MiB singleton path.

## Admission and canonical retention: critical ownership requirement

Current `PreparedAdmission::prepare_full` splits ordinary/oversized objects;
`prepare_ordinary` builds role groups, evaluates alternatives and calls
`pack::encode_group` (current D`:266`) and`pack::assemble` (`:301`). Preserve the
structural v1 path. Route only exact source-qualified file chunks into native
v2 payload-only packs; canonical metadata-value chunks remain explicitly legacy.
Do not interleave v1 and v2 records under one pack version.

The smallest change is a native preparation branch/helper using the existing
`self.packs` and`self.objects` outputs and the existing SQL publisher. Share normal
pack-size/count flushing infrastructure where its semantics match; do not copy
the analysis container/index into the product. Freeze native grouping/pack order
before samples. Existing logical roles and source identity remain unchanged.

**All native records, including native FULL, require retained canonical operands
until the final recheck.** Today `PreparedObject.retained` is present only for a
custom DELTA or a Zstd-compressed outer group. A native FULL inside a RAW outer
group is still compressed internally. Reusing the old condition would leave
`retained=None`, and `publish` would compare a pack byte slice containing a native
frame against a caller's canonical bytes. That would break CAS equality.

Set native `PreparedObject.retained=Some(original_canonical)` for both native
kinds, with correct object ownership and memory charges. Its `canonical` byte
range is only a legacy RAW-FULL optimization; do not give native encoded ranges
that interpretation. Use an explicit representation fact if needed rather than
pretending native FULL is a logical DELTA just to force retention.

`publish` keeps its same sequence: selected-location recheck, exact canonical
`compare`, winning-pack selection, transactional inserts, metadata publication,
commit, then successful diagnostic/location receipts. Existing
`admission::compare(:710 onward)` calls `db.visit_locations` for ordinary known
objects, so extending that reader correctly covers both initial CAS and late
collision checks. No second hash-only collision shortcut is introduced.

The insert path should remain format-agnostic: BLOBs plus unchanged canonical
length/pack/group/record locators. Preserve whole-pack persistence when only some
records win. Native depth/base facts must reflect actually selected earlier
representations; prepared objects from the same uncommitted batch cannot serve
as silently available bases. Record FULL fallbacks for absent/unsupported/over-
budget/nonwinning candidates, and preserve errors for corrupt dependencies.

Keep S1's origin-leaf selected-FULL rule unchanged. Legacy structural custom
DELTA cannot start depending on native prefix records merely because the reader
learns a more general reconstruction path. Native payload base kinds must be
explicitly frozen; supporting authenticated legacy chunk FULL/custom DELTA as
actual prior may be desirable, but must account for their extra FULL-anchor read
and define dependency-cap semantics rather than erase the legacy edge.

## File-content provenance independent of D telemetry

Reusable source provenance is the **owner marker**, not a telemetry count:

- The actual regular-file owners mark their ObjectBuffer: complete-file builders,
  `FrozenFile::build` and capture construction.
- Generic `rope::build_bytes` also builds mode/mtime metadata and stays unmarked.
- The marker follows existing first-owner ObjectId deduplication; a later file
  occurrence must not promote the original metadata owner silently.
- Existing private hint transport already carries the D source bit4 through
  memory/resume/spill. Canonical bytes/IDs are unaffected.

For P, expose a small `PhysicalHints`/authenticated-object source predicate in
`objects.rs` and an owner-facing file-context method. Production eligibility must
read that stable source fact plus exact canonical role validation, **not call
`diagnostic::file` or depend on diagnostic outcome tags/counter enablement**.
Rename/extract the source-marker API if necessary while retaining the bit's
transport and tests. The two D observer bytes can remain on the chosen foundation
for comparable controls; source provenance must remain valid if observer counting
is later disabled. Do not add another global lookup or persistent source index.

For combined measurements, both S1-only and S1+P controls need the same accepted
source-provenance/D-or-C foundation. Cherry-pick the marker/handoff independently
of native policy where appropriate, then keep S1's actual-origin hooks fixed.
Different D outcome counters under P cannot be compared as unchanged-policy
diagnostic evidence: the product treatment has changed, even if the source bit
continues to mean the same thing.

## Implementation split after D/C

1. Physical owner: version/entry/record parsing, native frame encode/decode and
   exact static-memory limits in`pack.rs`, with version1 compatibility tests.
2. Reader owner: extraction version propagation, bounded actual-prior helper,
   `visit_locations` native canonical emission, legacy FULL-base invariant and
   required-versus-optional error semantics in`read.rs`.
3. Admission owner: file-source predicate bridge, native grouping/preparation,
   canonical operand retention, selected-prior/fallback policy and output facts
   in`admission.rs`; publisher SQL remains unchanged except necessary counters.

Freeze these interfaces before concurrent edits. Test native FULL collision
comparison as well as PREFIX, mixed legacy/native reading, final-race partial
packs, missing/cyclic/cross-role bases, length/window/depth overflow and exact
canonical identities. No prospective159163199-byte component milestone is
reported as achieved until the actual combined image is measured.

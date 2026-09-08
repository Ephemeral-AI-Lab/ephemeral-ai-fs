# Issue #88 implementation working note: pursue 159.16 MB

Status: implementation plan, not an executed treatment or final measurement contract.
Owner direction: pursue the combined native-payload + S1 structural representation.
This note is the agent's resumption checklist and decision policy. Creating it does
not launch a build, encoder, replay or Store mutation. On a later implementation
assignment, apply the existing issue/user authority without repeatedly asking for
already authorized routine work; freeze the required concrete experiment contract
before collecting samples. Do not infer migration, rollout or merge authority.

## North star and honest completion

Build one authenticated, canonical-preserving retained representation combining:

- Native prior-version compression of existing file-content chunks.
- S1 structural-origin deltas with their current FULL-base rule.
- The same 157 filesystem states, metadata and public acknowledgement semantics.

The reference is **102,306,097 + 56,857,102 = 159,163,199 encoded bytes**.
These are separately measured offline components with different framing. They are
not a combined Store, a lower bound or a promise. Measure the common representation
and explain every adjustment from the reference. The **134,221,004-byte complete
allocated Store** goal remains a separate stretch objective.

Do not end this work at another general diagnostic. The delivery diagnostic is a
finite prerequisite; native-format design can advance alongside it. End with a
measured combined outcome and retain/revise/reject decision, or a concrete blocker.
An honest size miss is useful evidence. It does not authorize arbitrary tuning.

## Resume from actual state, not an assumed checkout

1. Read issue [#88](https://github.com/Ephemeral-AI-Lab/layerfs/issues/88), applicable
   AGENTS, [benchmark rules](../../../general/benchmark_rules.md), the exact smoke/
   full157 contracts and the [reconciled feasibility review](issue88-combined-feasibility/findings-and-pursuit.md).
   Latest explicit owner instructions take precedence over this working note.
2. Inspect actual HEAD, branch, staged/uncommitted edits, process/Store owners and
   active measurement lock. Preserve unrelated edits; never reset to these anchors.
3. Current experimental checkout:
   `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue88-experiments`, branch
   `codex/issue88-encoding-experiments`. Feasibility report checkpoint:
   `0192e143b`. Accepted M4.5 diagnostic foundation:
   `eb7050603c5ee97a02dfa0d5357619079d25ab51`; reporting revision is not measured
   product identity. Use an isolated branch/worktree for the diagnostic foundation.
4. Runs root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs`.
   Protect original `full157-m45-1` and every sealed prior analysis/screen.
   Original Store SHA256:
   `2036b98181dd3fa47788619896cb50daa27ac5c85ea8cc482c20d4b447573ed9`.
   Frozen157 manifest SHA256:
   `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`.
   Reauthenticate actual required seals before use; this note is not authentication.
5. Resolve known unfinished reporting before relying on it: existing issue88
   `findings.md` has a full-history placeholder; the final census/report summaries
   and complete sealing are unfinished at this checkpoint. Inspect any retained
   census for completion/custody and reuse valid outputs. Do not launch another
   census merely because an aggregation report is missing. Preserve and finish
   only relevant outstanding edits after establishing their ownership.

Baseline facts, with scopes: original acknowledgement allocation335,552,512 B;
S1 public acknowledgement302,006,272 B; S1 logical database296,845,312 B.
The allocation difference includes11,558,912 B of changed signed allocation
adjustment. Neither that adjustment nor the entire33,546,240 B difference is
proven structural-encoding savings. Historical full-run timing is not a fresh pair.
Candidate canonical totals differ from the original internal population despite
verified equal filesystem states; compare exact populations within each analysis.

## Chosen architecture: smallest first implementation

Use the existing SQLite Store, `object_packs`, ObjectId selected index, CAS checks
and transactional publisher. Admission already separates content and structural
groups. Preserve structural version1 groups and write file-payload-only version2
packs containing native FULL/PREFIX records in RAW outer groups. Batch records;
do not create a SQLite BLOB per chunk. Avoid recompressing native frames again.

Keep canonical framing/IDs, CDC, FileState, extents, inode and namespace trees.
Retain all other supported objects through the explicit legacy path, including
the five metadata-value chunks omitted from S2's regular-file screen. Do not
classify arbitrary bytes by magic prefix: use exact outer/role decoding and
reviewed file-content provenance.

Required new code belongs at the existing physical boundary:

| Responsibility | Source ownership | Required behavior |
|---|---|---|
| Diagnostic delivery | `objects.rs`, `objects/spill.rs`, existing receipt owners | Preserve decisions/layout; carry bounded reason/grant state to authoritative CAS/admission |
| Format | `objects/pack.rs` | Version dispatch; strict native FULL/PREFIX grammar; checked sizes, frame/window/output validation |
| Reader | `objects/read.rs` | Iterative bounded actual-prior reconstruction; authenticate canonical bases/targets; enforce role-specific dependency rules |
| Admission | `objects/admission.rs` | Declared public hint selection, complete alternative costs, canonical CAS comparison, successful-location provenance |
| Independent review | Existing analysis tools and reports | Conservation, root/base closure, cohort validity, timing/custody and negative results |

Parallel owners must have explicit method/file boundaries. Do not concurrently
edit shared methods. Source design/review may run concurrently; builds, encoders,
smokes and benchmarks must serialize under the existing measurement lock.

No new database, generic storage abstraction, persistent similarity index, whole-
file canonical migration, unbounded cache, global sort to imitate offline input
order, dependency-source edit or backend change. Reuse installed Zstd and existing
decoders/locators. Keep legacy readers and the structural custom matcher; they
cannot be deleted merely because payload PREFIX is added. Remove only code made
unreachable by the final measured treatment, with compatibility accounted for.

## Work sequence and gates

### 0. Close evidence gaps and prepare the next contract

- [ ] Establish current custody and finish relevant existing aggregate/report
  finalization without rerunning valid measurements.
- [ ] Read the [diagnostic design](issue87-diagnostic-design/findings-and-contract.md)
  and [admission ownership design](issue87-diagnostic-design/admission-design.md).
- [ ] Create a short append-only decision ledger in the new run/report directory:
  question, evidence, alternatives, choice, permitted differences, expected
  falsifier, source identity and result. One row per material decision; no new
  tracking service or general experiment framework.

### D. One bounded unchanged-policy delivery diagnostic

- [ ] Implement the reviewed two-byte reason/grant design on accepted M4.5.
  Prove actual compiled type size/alignment, capacities, spill bytes, ordering,
  cursor decisions and all original resource limits remain unchanged. Synthetic
  layout witnesses are not proof of the product implementation.
- [ ] Count unique initially missing eligible file payloads and canonical bytes
  at the initial CAS owner, before optional matching-memory exclusions. Separate
  occurrence work, cursor grants, attempts and admitted winners.
- [ ] Preserve exact first/inherited memory/file/operation/descriptor causes;
  no predecessor; missing required span; complete/limited empty; complete/limited
  hints. A partial hint can still win. S1 origins intentionally lack file spans;
  structural objects and metadata values are not missing-span defects.
- [ ] Test handoff/spill, duplicate-credit conservation, reused cursor benefit,
  sticky reasons, partial hints, late races, partial-pack persistence and
  successful transaction provenance. No extra search or encoding in D.
- [ ] Require source-proven ownership plus counts AND bytes to reconcile across
  all158 Init/checkpoint rows and selected locators. A late-race/ownership failure
  invalidates the unique-cohort interpretation; preserve attempt evidence.
- [ ] Freeze and execute only the authorized diagnostic contract. Record observer
  cost; it cannot support a speedup claim. One final logical inventory is enough.

**D ends with a decision:** existing delivery supports a useful P test; one precise
coverage correction is justified; or the opportunity is not transferable under
the declared limits. Counts of missed hints, saturated budgets or reused-triggered
grants alone do not measure useful compressed savings. No mandatory second general
diagnostic. A demonstrated instrumentation failure can justify a scoped correction
and new identity; preserve the invalid result and do not optimize to force its gate.

### C. Conditional coverage correction, only if justified

- [ ] Name the byte-bearing mechanism and smallest shared-path correction.
  Missing required spans imply fixing that handoff. A genuine allowance problem
  requires one prospectively declared bounded policy change, not an invisible
  budget increase or automatic grant refund.
- [ ] Keep codec/format/dependency policy unchanged. Measure added delivered
  candidates and actual work/cost; reserve bytes are not actual I/O.
- [ ] Retain or reject C. If rejected, test P with existing delivery when still
  useful, or report the constrained opportunity as rejected. Do not restart D
  indefinitely. Freeze the same accepted foundation for both later public arms.

### P. Freeze and implement the native physical treatment

Design/interfaces can be reviewed while D is pending; candidate-dependent choices
wait for D/C. Before encoding/product samples, commit a real contract containing:

- [ ] Exact version2 grammar and pack/group/record size meanings; native FULL and
  PREFIX headers; base ID is canonical ObjectId; canonical output length includes
  the exact chunk framing, not just raw payload. No ambiguous old kind1 reuse.
- [ ] One public candidate-selection rule. Default proposal: first delivered
  chronological payload hint, actual selected prior reconstruction, no search
  through offline manifests or forced producer ordering. Explicitly freeze which
  legacy/native base kinds are supported and report unsupported-base fallback.
- [ ] Fixed starting codec proposal from S2: Zstd1.5.7, level3, windowLog20,
  one thread, content size/checksum enabled, no dictionary ID; at most4 prefix
  edges and1MiB cumulative raw payload closure; compare complete native PREFIX
  record against native FULL including base/framing. These are proposed settings
  until the contract binds the actual library/build and complete policy.
- [ ] Group/pack assembly and FULL fallback policy. The public comparison treats
  changed codec, framing and actual-prior reconstruction as one coherent physical
  replacement, not a codec-only optimization. Do not tune parameters mid-run.
- [ ] Actual scratch/context/association/group-read bounds. Four edges on32KiB
  chunks imply at most163,840 raw closure bytes, but do not bound enclosing reads,
  codec contexts or concurrent buffers. Recalculate existing1MiB validation and
  2MiB encoding ownership; never silently exceed or shrink shared budgets.
- [ ] Compatibility: new reader supports legacy version1; unsupported version2
  fails cleanly for old readers. Decide whether early Store capability rejection
  is necessary. A SQL schema redesign is not intrinsically required for new BLOB
  bytes. Fresh experimental Stores only; no migration or silent conversion.
- [ ] Tests of exact canonical readback, FULL/PREFIX fallback, recurrence/CAS,
  shared bases, bounded maximum-depth reads, wrong/missing/cyclic/cross-role bases,
  malformed/truncated/concatenated frames, length/window/depth overflow, legacy
  dispatch, and canonical collision comparisons during final recheck. Corruption
  is an integrity error, never a quiet FULL retry.

Use the smallest meaningful regression tests, reuse existing fixtures and expand
only on changed behavior/failure. Do not copy the offline Python analysis index
into the product. Start with sequential iterative chain reads and bounded state;
do not add a cache just to hide a poor first read result.

### SP. Demonstrate combination through the public path

- [ ] Freeze exact source/product/dirty-patch, executable/image/codec, harness,
  workload/oracle, reporter, environment and cache identities before each arm.
  Reuse validated builds/preparation only where compatible; no mutated Store reuse.
- [ ] Compare fresh **S1 legacy payload** against **same foundation + S1 + P**.
  Hold S1 policy fixed, not its assumed byte result. Shared work/grouping may
  change structural output; record that interaction. Obtain a fresh matching
  control when old source/harness/framing/custody is not comparable.
- [ ] Use the existing deepseek-five, frequent-edits and small-files smokes with
  separate verification. Discover exact current runner arguments from its checked
  help/source; do not paste commands from another benchmark family. Freeze arm
  order, repetitions/statistics and resource/time limits in the contract.
- [ ] Fix the smallest demonstrated correctness defect; preserve old identity
  and failures, rerun only affected checks. For an invalid paired sample, rerun
  the complete affected pair under the normative rules, not just the slower arm.
- [ ] Promote to a fixed full157 comparison only after correctness/cleanup and
  recorded storage/read/foreground evidence justify it. A full-history applicability
  decision may explain size-dependent effects; it cannot erase smoke regressions.
- [ ] Reuse same-unit offline FULL/PREFIX mechanism evidence with its limitations.
  Do not require four full-history arms merely for a factorial label. Without
  matching four-arm data, report incremental treatment and observed group changes,
  not an isolated statistical interaction term.

## Decision policy: how I should exercise judgment

| Observation | Judgment / next action | Do not do |
|---|---|---|
| Few public hints, reasons valid | Select one evidenced delivery change or test P under current constraints | Assume every missing hint is a useful delta |
| Grants triggered by reused chunks | Check shared cursor benefit and unique missing-byte causes | Label all grants wasted or automatically refund them |
| Good bases but native FULL wins | Retain fallback and measure actual group/frame effect | Increase depth/search/codec level after seeing losses |
| Large raw savings, weak allocated gain | Reconcile framing, groups, bases, pages and signed adjustment | Count canonical bytes as physical savings or propose VACUUM |
| Combined structure grows | Explain shared budgets/grouping/CAS and compare valid controls | Import the offline56,857,102-byte figure as fixed |
| Read/foreground slowdown | Compare absolute and relative cost with storage gain, workload scope and uncertainty | Use a raw byte cap or average to excuse bad tails |
| Valid miss of159.16MB | Report measured size, component bridge and strongest evidenced next lever | Relabel the target, discard the result or start a sweep |
| Meets encoded milestone, allocated Store larger | Report both as distinct outcomes | Call it a159MB Store or claim134MB achieved |
| Identity/oracle/closure/resource/cleanup fails | Stop affected run; seal partial evidence; diagnose cause | Relax bounds or bypass identity to finish |
| Missing authority or another active Store owner | Continue independent design/report work; state exact blocker | Open a written Store, reset checkout or invent authorization |

The owner allows roughly30% longer foreground operations for meaningful storage
gain, judged in absolute time for short calls. This is guidance, not automatic
acceptance, a universal read-latency limit or permission to loosen frozen gates.
Keep original numerical failures visible. Do not invent a storage acceptance gate.

Before adding complexity ask: which measured bytes/work does this remove, what
evidence identifies the mechanism, what is the smallest falsifier, and what code
could be removed? If those answers are absent, keep the simpler bounded treatment.
Allow exactly the declared treatment; further optimization needs a new evidenced
decision and prospective contract, not a "while here" change.

## Measurement and accounting contract

Maintain integer bytes/ns/counts with population, phase/snapshot, provenance and
measured/derived/unknown status. Missing values are null with reasons, never zero.

Reference framing bridge:

```text
S2 =95,601,473 native frame bytes
   +1,865,536 prefix base-ID bytes
   +4,839,072 experimental record-header bytes +16 container-header bytes
   =102,306,097 bytes
reference +S1 structural group bodies56,857,102 =159,163,199 bytes
```

Replace experimental envelopes with actual version2 envelopes; include omitted
objects and version1/version2 pack/group framing exactly once. Header removal is
not free savings if the new decoder/index still needs those fields. Enumerate all
required physical records: logically retained, base-only and unselected residue
separately. A base already in a lane total is not another additive copy. One real
selected index, not the sum of diagnostic indexes. Compressed bytes belong to
groups, never proportionally assigned as measured per-record compression.

Reconcile nested accounts: pack bytes within SQLite BLOBs/pages; page partition
including overflow once, freelist, row encoding, unused and residual; original
acknowledgement database allocation plus sidecars; signed allocation adjustment.
Do not add SQLite size to pack size or call unused/residual bytes reclaimable.
Keep spool/staging/runtime scopes separate; runtime can already contain the Store.

Timing: preparation, transfer, public Init/Exec/Commit-finalization/End, observer,
case and enclosing invocation separately. Nested fetch/match/codec time is not
additional public elapsed. Keep attempted A/B work separate from admitted winners.
Measure host/container CPU/RSS/I/O, temporary disk and actual sampling scopes.
No historical timing pair, OS-cold claim from fresh application contexts, or
fresh paired speedup from changed harnesses. If new telemetry requires a harness
change, use the same frozen harness for product arms; D remains diagnostic.

At final acknowledgement record original allocation; close normally and seal
receipts/source; retain one final pre-verification logical snapshot. Follow normal
verifier identity checks with explicit copy custody where needed. Copies/clones
preserve logical content, not allocation equivalence. Report cache warming.
Reuse one bounded final canonical/reference/pack inventory for all analysis owners;
no full Store copy/traversal/decompression after each Commit.

Host owns SQLite/coordinator/canonical publication/spool. Keep the established
Docker daemon/FUSE/workload topology and applicable2CPU/2GiB/no-swap/256PID limits.
Use exact existing storage-smoke/full157 contract limits, not generic quickstart
timeouts for another family. Freeze changes prospectively if authorized. No
concurrent resource-sensitive work, favorable seeds or timeout extension after miss.

## Final artifacts, publication and resumption

- [ ] New unique output directories; preserve original Stores and all failed runs.
- [ ] Committed prospective contracts, exact commands/settings and identity seals.
- [ ] Raw performance and verification receipts, cleanup, all157 mappings/oracles.
- [ ] Combined component bridge, four nested accounts, roles/root/base closure,
  admitted provenance and read/foreground/resource costs with limitations.
- [ ] Independent review reconciled; hash manifest over retained artifacts and
  byte-identical compact committed reports. No report-only rerun of the product.
- [ ] Final verdict: encoded reference met/missed with exact scope; allocated
  Store achieved; costs; retain/revise/reject; at most one justified next experiment.
- [ ] Commit/push relevant work preserving unrelated edits; additive #88/#87
  reporting as relevant. No merge, rollout, migration, M5, cloud or release closure.

At each handoff/compaction leave: current phase; HEAD and dirty owners; exact
running process/session/lock owner; latest frozen contract; completed valid checks
not to repeat; failure/blocker; next concrete action. Mark checkboxes only from
actual evidence. This working plan may receive dated additive progress notes;
do not rewrite frozen contracts or historical results to match later outcomes.

**Immediate next action when implementation is assigned:** inspect and finish the
existing evidence aggregation/custody, then implement the scoped D instrumentation
and review the version2 grammar concurrently under separate ownership. This note
does not claim either implementation has started.

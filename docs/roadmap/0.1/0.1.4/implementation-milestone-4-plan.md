# Milestone 4: bounded deltas and encoded-group selection

Status: **M4 implemented and smoke-verified**, 2026-09-08.
Evidence: [reviewed M4 report](implementation-milestone-4-review.json),
[original M4 report](implementation-milestone-4.json), and [progress](implementation-progress.md).
Independent review gaps were corrected and the three smokes rerun; storage/timing
diagnostic misses remain explicit.
This development checkpoint stops before M5 and is not complete v0.1.4 qualification.
Tracking: [M4 issue #84](https://github.com/Ephemeral-AI-Lab/layerfs/issues/84).
Continue from closed M3 on `codex/storage-v3-implementation`, draft PR #81.
Discovery HEAD: `4626583c72c0e4cbe0dd818c58fbe784c453dbbc`; M3 product commit:
`1ac1ce4b56064a548929b070568d2373daa9e30d`. Inspect actual HEAD, active writers and
working-tree edits before starting; never reset a checkout to these anchors.
This documentation change starts no product implementation or verification run.

This plan supersedes the earlier raw-record-only delta decision and one-encoding-
per-group rule for M4. M3 results and its one-attempt policy remain historical facts.
The existing schema-6/wire-1 new-Store-only compatibility scope is approved; no
migration or reinterpretation of old Stores is authorized. No wire or canonical
format change is expected for M4.

## Objective and mental model

Evaluate whether predecessor-guided physical deltas materially improve residual
storage after CAS/CDC/COW and M3. DELTA emission alone is not evidence of success.
M4 is not necessary for existing localized-edit correctness or COW reuse.

```text
Workspace capture / Init input
  -> shared canonical construction: CAS + CDC + COW
  -> new canonical objects + bounded predecessor hints
  -> matching against admitted authenticated FULL bases
  -> fixed group membership:
       A: all FULL -> Zstandard or RAW
       B: selected FULL/DELTA -> Zstandard or RAW (only if eligible)
  -> worthwhile smaller encoding, discard unselected alternative
  -> existing bounded SQLite admission
  -> existing staging / conditional publication / required finalization
```

Logical ObjectIds, inode semantics and retained filesystem graphs do not depend
on the physical choice. Accurate range edits retain old extents directly; no new
whole-file scan is introduced for a small edit. Init shares the encoder without
invented Workspace predecessors. Whole-file replacement requires actual discovery.
All authoritative data remains in SQLite; both trials complete before public
acknowledgement, outside Store permits/connection locks/transactions. Stage,
no-change, head-check and finalization behavior remain distinct and unchanged.

## Read first

Read applicable AGENTS.md and implementation/debugging skills, then:

- [Architecture](storage-architecture-spec.md), especially predecessor handoff,
  matcher limits, encoded-group selection, memory ownership and closed admission.
- [Wire format](sqlite-storage-format.md), including depth-one reader validation.
- [Boundary](storage-efficiency-boundary.md), [general rollout](implementation-plan.md),
  [progress](implementation-progress.md), [M3 evidence](implementation-milestone-3.json),
  [smoke contract](implementation-smoke-contract-v1.md) and
  [smoke plan](storage-smoke-test-plan.md).
- Store: `objects.rs`, `objects/pack.rs`, `objects/admission.rs`, `objects/read.rs`,
  `objects/spill.rs`, `workspace.rs`, `schema.rs`, `staging.rs`, and included SQL.
- Content: `file/rope/{build,edit,read,state,mod}.rs`, `file/extent.rs`,
  `file/extent_codec.rs`, and `filesystem/resolve.rs`.
- Workspace: `changes.rs`, `capture.rs`, and relevant lifecycle/backing callers;
  Workspace core `file_edit.rs`/`namespace.rs`; follow SDK/FUSE callers as needed.

Resolve code paths relative to their existing crates, as detailed in the general
plan. Trace siblings; do not assume helper names establish ownership or batching.

## Resulting structure and ownership

```text
crates/layerfs-layerstack-store/src/
  objects.rs                 carry hints/owned canonical output; diagnostics
  objects/pack.rs            bounded matcher, DELTA records, alternative encoding
  objects/admission.rs       eligible bases, budgets, selected packs, publication
  objects/read.rs            physical order before draining; depth-one reads/counters
  objects/spill.rs           extend existing private span storage only if needed
crates/layerfs-content/src/file/rope/
  build.rs / edit.rs         original-emission spans and prior-range context
  read.rs / mod.rs           shared traversal and private wiring
  cursor.rs                  optional new private resumable extent cursor
crates/layerfs-workspace/src/
  changes.rs                 once-per-file predecessor and complete-build handoff
  capture.rs                 original first-span facts through captured output
```

Keep the cursor in the existing reader if that is cleaner; do not duplicate
traversal. No new crate, backend, plugin interface, delta service, durable hint
index, shadow table or public API is needed. Only selected packs are stored.
Existing telemetry/receipt plumbing may expose counters; workload definitions,
interfaces, timing and accounting boundaries must not change.

## Iteration order

1. **Read ordering and diagnostics.** Arrange selected locators by `(pack, group,
   record)` before bounded internal draining in the shared owner. Preserve result
   order/duplicates and memory limits. Count physical fetches/decode work; no cache.
2. **Predecessor handoff.** Preserve same-inode context through complete fallback;
   resolve one final path against the pinned previous namespace for eligible
   tempfile replacement. Its old destination supplies physical hints only, never
   logical `before`, inode identity or metadata. Preserve first-emitted spans from
   original capture before cross-file deduplication; no second read/CDC pass.
3. **Bounded matcher.** Use the architecture's forward cursor, at most four hints,
   authenticated admitted FULL bases, fixed seed table and existing work budgets.
   A prior DELTA can supply its FULL anchor; it cannot become another dependency
   layer. No speculative same-batch bases, alias/history search, or deeper chains.
4. **Encoded alternatives.** Implement the selection and memory contract below
   in the existing pack/admission owners. Keep Zstandard level 1 to isolate M4.
5. **Integrated verification and fixes.** Run the three approved smokes, fix root
   causes, rerun affected cases, and report retain/revise/remove disposition.
   Stop at M4, without starting M5 final qualification.

## Encoded-alternative policy

Form common group membership using existing FULL decoded sizes, role targets and
pack limits. For each target choose the smallest complete admissible raw DELTA
(tie: base ObjectId) only if no larger than its FULL record, so mixed group framing
fits the same bounds. This candidate choice is an approximation, not a compressed-
optimal search. Do not retain the old per-record 12.5% acceptance gate.

Encode A (all FULL) once. If useful candidate deltas exist, encode B (one selected
mixed assignment) once. For each alternative independently retain Zstandard only
if its complete frame is at least 16 bytes smaller than its decoded RAW bytes.
Codec errors propagate. Compare selected RAW/compressed encoded lengths, including
record/group framing and frame overhead; equal-size pack directory entries cancel.

Choose B only when `A_bytes - B_bytes >= max(64, ceil(A_bytes / 8))`.
Otherwise choose A. This prospective 12.5%/64-byte encoded-group threshold is an
engineering default, not a measured optimum, whole-Store saving or benchmark gate.
A group without an eligible delta and an oversized RAW singleton have one encoding
route. At most two codec attempts per eligible group, never all combinations or
recompression of growing pack prefixes. No extra durable FULL copy is stored for
a selected DELTA. Only finally selected and admitted FULL representations become potential later
anchors; a racing existing representation remains authoritative.

### Buffer lifetime and failures

Keep the existing 2-MiB physical reservation inside the inclusive 8-MiB allowance
(or smaller caller bound), plus unchanged closed-episode validation requirements.
Compare one group at a time. Retain only its chosen A encoding while constructing
B; release base/match-table scratch before compression where possible. Retain only
the winning group in prepared output, not two complete alternative packs. Account
for Vec capacity, canonical equality operands, programs, directories, codec context,
trial output and already-prepared backing. Do not equate truncation with release.

Prove simultaneous live ownership fits before wiring the writer; do not raise caps.
Optional delta work that cannot fit its budget is skipped and counted before
starting its trial. Stop further search on exhaustion; retain completed eligible
candidates if the group alternative still fits, discard incomplete trials, and
use FULL where no complete mixed alternative fits. Required FULL encoding/resource failure and detected integrity
failure propagate. Never disguise them as optimization fallback. A bounded reader
wave does not authorize dropping the complete late-validation reservation.

## Evidence and acceptance

Use only first-five DeepSeek, frequent-edit SDK/ordinary, and small-file smokes.
Build matching host/runtime artifacts as preparation. No unit/fuzz/property/race/
crash/full-workspace/new-family campaign, new populations or candidate sampling.
No M5 final three-pair qualification in M4. Preserve old artifacts and failed runs.

Record through existing diagnostics:

- eligible/missing-base objects, hints/trials, budget skips, DELTA selections and
  FULL selections (including no useful predecessor and rejected mixed alternative);
- A/B encoded bytes and selected bytes, compression calls/time and matching work;
- physical group/base fetches, encoded/decoded bytes and decompression calls;
- total SQLite allocation at acknowledgement, retained-history growth, foreground
  elapsed, reads, CPU/memory/spools and cleanup, keeping each scope distinct.

Compare original baseline -> M4 and M3 -> M4, with historical control limitations.
Verify all 33 historical mappings, no-change and cleanup using existing oracles.
Show selected DELTA coverage when the unchanged workloads supply the opportunity;
if they do not, disclose unexercised behavior rather than forcing a fixture or
claiming delta-read qualification. Source review is not empirical coverage.

The owner accepts roughly 30% extra elapsed for longer foreground operations when
storage gains justify it; approximately sub-50-ms total public operations can make
percentage changes immaterial. This is not extra 50 ms per lookup, an aggregate-
load guarantee or permission to spend on negligible savings. Preserve original
diagnostic misses and historical gates. No numerical M4 storage target is imposed.
The existing binary histories add only 64 KiB after Init, and text/small allocation
is flat across edits; do not promise large total reductions there.

## Completion and stop checklist

- [x] Source custody and preceding evidence preserved; revised policy recorded before measurements.
- [x] Shared read ordering/counters implemented without new cache or interface.
- [x] Same-inode, complete, range and captured/tempfile predecessor handoffs connected.
- [x] Matcher/base authentication and depth-one limits implemented within existing budgets.
- [x] Before DELTA emission, source-review admission/cleanup so required FULL bases
      remain readable even without direct logical references. Describe physical
      closure through existing record/base locations; export/GC remain deferred.
- [x] Two alternatives, threshold, selected-only persistence and buffer bounds implemented.
- [x] All three approved smokes and historical/no-change/cleanup checks completed; affected failures fixed.
- [x] Allocation/latency/resource evidence and DELTA coverage/limits recorded honestly.
- [x] Final diff removes obsolete policy/code and has no unrelated or M5 work.
- [x] Progress and new `implementation-milestone-4.json` record actual results, not fabricated placeholders.
- [x] Commit/push existing draft PR; do not merge. Stop with retain/revise/remove recommendation.

If gains are weak, finish a correct reviewable checkpoint and disclose that result.
Do not escalate search or rewrite benchmarks to satisfy an invented target. Any
subsequent removal/disablement must preserve readability of retained DELTA records
and explicitly disposition the format; no silent incompatible rollback.

## Deferred

Stronger codec-level experiments, larger groups, context pools/caches, prefix/suffix
construction redesign, FileState-wrapper removal, CDC changes and M5 cleanup stay
outside M4. SQLite allocation tuning is deferred. S3 hybrid is future issue #82;
M3 review opportunities remain tracked separately in #83. No new durability,
recovery, migration or cloud implementation enters this milestone.

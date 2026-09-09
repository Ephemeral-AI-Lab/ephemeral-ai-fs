# v0.1.5 delta-encoding benchmarks

> **Status:** First-round smoke contract and deferred benchmark design, reviewed
> 2026-09-09. No new harness or results are created here. The owner-authorized
> local smoke is exploratory (`admission_eligible=false`); broader release
> benchmarks still require their issue, committed contracts and frozen gates.

Read [workflow.md](workflow.md) for the current architecture and
[benchmark_success.md](benchmark_success.md) for the existing family inventory.
Use [benchmark rules](../../../general/benchmark_rules.md),
[benchmark hosting](../../../../benchmark/AGENTS.md), and the
[runner guide](../../../../benchmark/fs-bench-pro/QUICKSTART.md).

## First-round execution scope

**Implement the full product; verify only one ten-file/thirty-commit smoke.**
Family **`small_file_delta_smoke`** has exactly one case:
**`small-file-delta-10x30-v1`**. Use ordinary complete-file saves through public
Exec/FUSE and public Commit. No SDK/FUSE matrix, parameter sweep, old tiny rerun,
separate read campaign or broad test suite during this implementation round.

```text
 Freeze one ten-file / thirty-commit smoke
                     |
          Collect released-v0.1.4 control
                     |
          Implement -> run same smoke
                     |          |
                     |    Reopen SAME Store
                     |    Verify all 31 states
                     |          |
                     +--- inspect / fix / repeat
                                |
                      Later release qualification
```

### Fixed fixture and edit schedule

Use one Store, Branch and Workspace, ten flat regular files `source00.ts` through
`source09.ts`, with sizes in KiB **[4, 8, 12, 16, 20, 24, 28, 32, 40, 56]**.
Total genesis content is **245,760 bytes (240 KiB)**, with 64-896 initial lines
per file. Every source line is exactly 64 ASCII bytes, including its newline.

Reuse the existing tiny fixture's construction recipe with this new domain:
for file index `i` and line index `j`, let `value` be the first 36 lowercase hex
characters of SHA-256 over UTF-8 `small-file-delta-10x30-v1/{i}/{j}`. The line is
`export const v{j:04d} = "{value}";`, padded with ASCII spaces to 63 bytes, followed
by `\n`. Genesis has `N = initial_size/64` lines, numbered 0 through N-1.
All files use mode 0644 and the existing importer's fixed metadata normalization;
source directory mode 0755 and timestamp 1,000,000,000 follow the tiny recipe.

For step `s` from 1 through 30, choose `i=(s-1)%10` and edit only that file:

| Steps | Fixed operation |
| --- | --- |
| 1-10 | On original line `(17*(i+1)) % N`, rotate the eight hex digits starting at byte offset 22 within the line, each by +1 modulo 16. |
| 11-20 | Insert one new 64-byte line immediately before original midpoint line `N/2`. Generate it with the same recipe and `j=9000+i`. |
| 21-30 | Delete original quarter-position line `N/4`, which lies before the inserted midpoint and still starts at offset `64*(N/4)`. |

All N values are divisible by four. Deleting a different original line keeps the
final content distinct rather than simply undoing the insertion. Every step
changes exactly one file; nine remain unchanged. Maximum file length is **57,408
bytes**, and final file lengths equal genesis. Keep all versions; no threshold
crossing or settings comparison is hidden in this first fixture.

Generate manifests and SHA-256 path/mode/length/content oracles for genesis and
all 30 results before timing. Reuse existing fixture output/manifests/importer
helpers. The fixture self-check proves exact counts, sizes, unique changed steps,
hex-edit positions, inserted/deleted lines, unchanged siblings, deterministic
output and all 31 oracles. This self-check is part of the smoke, not another suite.

### Public route, measurement and verification

Initialize natively from the ten source files. Preconstruct/transfer changed-file
inputs outside operation-local timers. Each step calls the existing compiled
full-file importer through public Exec/FUSE, then public Commit. The actual full
write stays inside Exec; no SDK substitution, direct Store mutation, alternate
save algorithm or new per-step status query is allowed. Expect **30 Exec saves,
30 Created commits and 31 retained states**.

Record native Init separately and collect post-acknowledgement initial/per-step/
final allocated and apparent Store bytes with existing observers. Record integer
Exec and Commit nanoseconds and paired `Exec+Commit` per step, median and range.
Preserve applicable sidecars/base/metadata costs; report spool/staging separately.
Use existing receipts before adding new instrumentation. Observe candidate new
small-object and DELTA selection through bounded diagnostics after timed storage
measurements; the candidate must actually exercise delta reconstruction, not
silently use only FULL or old chunked output.

Run the integrated verifier **against the same retained performance Store** after
its allocation measurements are frozen, reopening it through a new coordinator
connection. Verify all ten files' bytes, paths, modes and lengths at all 31 states,
then complete the existing lifecycle cleanup. Do not substitute verification of a
freshly regenerated history. Verification has its own time/resource scope and is
not part of save/Commit timers. No extra read-performance campaign in this round.

Use the same fixed fixture/harness/timers on released v0.1.4
`101fa273d815f3aaedb0e06ba0de7b0777d83def` and candidate. Collect the released
control first; retain its exact source/binary/image/fixture seals. The historical
three-file result is not this control and its numerical gates do not apply.
Do not alter or rerun that old fixture during this round.

This smoke's pass is **correctness/route/cleanup plus actual delta-path execution**,
not a fabricated numeric performance qualification. Report storage/latency changes
against its control and fix demonstrated slowness. There is no new absolute
storage/latency threshold or comparative tolerance authorized by this draft.
If future admission adds them, freeze them before its candidate measurements.

Register the new case with 600-second phase and 30-second operation watchdogs
and use the approved
host-owned Store topology, with container 2 CPUs/2 GiB/no swap/256 PIDs. Reuse the
shared measurement lock and matching build/input artifacts. One source-bound
history is one exploratory observation, not 30 independent repetitions.

The owner explicitly selected this local smoke-only implementation loop. Missing
release-admission issue/committed-contract prerequisites do not block this local
work; mark `admission_eligible=false` and do not make a release claim. Broader
SDK/FUSE scenarios below, all 56 large-file cases, format/failure/POSIX suites,
realistic workspace and full157 campaigns are deferred until after this task.
Runtime validation and the full production implementation remain required.

## 1. Questions this qualification must answer

For new/changed file content below **128 KiB**, the proposed representation is one
whole-file canonical CAS payload, physically FULL or a bounded one-level DELTA
against a FULL base. Larger files retain CDC chunks and extent sharing. Both
namespace Init and workspace Commit use the same construction/admission/publication
pipeline, with different source inputs.

Qualification must establish:

1. Insertions, deletions, unequal-length replacements, and shifted content remain
   correct and can reuse matching base bytes. Equal-length overwrites alone are
   insufficient evidence.
2. Similar retained histories reduce actual storage without excessive save,
   Commit, reconstruction, memory, database, or transport costs.
3. Repeated content reuses its CAS payload; similar content may use a delta;
   unrelated content can fall back to FULL within the bounded policy.
4. Whole-file small-object preparation does not damage existing large-file range
   locality, metadata-only behavior, or shared Init/Commit batching.

Do not claim that every delta is the size of the latest edit. Deltas reference a
FULL anchor; differences can accumulate relative to that anchor. Returning to an
existing object saves file payload, not necessarily commit/namespace metadata.

## 2. What the three released edit families prove

Source authority is released v0.1.4, commit
`101fa273d815f3aaedb0e06ba0de7b0777d83def`.

| Family | Cases | Operation matrix |
| --- | ---: | --- |
| `edit_length_preserving` | 12 | 4-KiB overwrite at head, middle, tail; four file-size tiers. |
| `edit_length_changing` | 32 | Middle insert/delete, append/prepend, middle replacement 2-to-4/4-to-2 KiB, tail truncate/zero extension; four tiers. |
| `edit_canonical_chunk_count` | 12 | 64-KiB overwrite at offset 147,456; replacement payloads selected to preserve/increase/decrease canonical extent count; four tiers. |

The tiers are 1, 10, 100, and 500 MiB; growth at the last tier uses the existing
result-capped definitions. There are **56 cases**, each one explicit public SDK
range edit followed by Commit, not a 30-version history or ordinary editor save.
The older `edit_length_changing_capped` directory is not another active family.

Sources: [preserving][preserving], [changing][changing], [chunk count][count],
[shared sizes and payload generation][common], [timed SDK calls][calls].

"Count changing" must name its quantity. File byte/line count and canonical
chunk/extent count are different. The third family holds file length constant
while changing the canonical extent count. Physical pack-record count is another
quantity and must not substitute for either one.

The [released verifier][verify] checks canonical roots, mappings, lengths,
retention, and boundary bytes through the Store and FUSE. It is explicitly a
canonical-roots-and-boundaries proof, not exhaustive full-byte verification.
Its resource assertions include replacement-only CDC scans, no SDK-edit-caused
FUSE payload writes or edit-spool allocation, and zero Commit payload reads in
the length-changing family. This is direct extent reuse, not merely CDC finding
new boundaries after a full-file scan.

Keep these large-file definitions, coverage, and gates unchanged. They stay on
the large-file route and remain regression requirements. Do not insert small
sizes into their frozen registries or change old expected roots to suit the new
small-file representation.

## 3. The small-file tradeoff we must measure

```text
 Existing known range edit             Proposed small-file Commit
 -------------------------            --------------------------
 Retain old extents                   Assemble complete small target
 Process replacement bytes            Hash its canonical content
 Rebuild affected mapping             Resolve exact CAS reuse
                                      Use bounded FULL base if eligible
                                      Encode DELTA or store FULL
```

All three families' byte transformations are representable by the new small-file
model. Shifted matches can be reused without equal offsets. However, preparing
the whole-content identity generally requires processing the complete small
target and may acquire unchanged/base bytes. It cannot inherit a universal
zero-old-payload-read assertion from the large-file extent path.

Deletion, truncation, and zero extension can already be very cheap through
extents; delta storage need not win each operation. Report per-step costs, not
just an aggregate that hides these regressions. A compact encoded result does
not establish encoding time proportional to changed bytes.

The new small-file representation has no CDC member list. Content that would
alter CDC chunk count instead creates one new whole-file content identity.
Test its bytes, encoding, and resource behavior; do not demand the old extent
count. The new canonical format needs explicit compatibility and old-history
read tests, not silently updated historical oracles.

## 4. Preserve the original tiny baseline for later comparison

Keep `tiny_rewrite_history / tiny-history-30` exactly as recorded in the
[frozen case](tiny-history-baseline-v1.md): three 4/16/40-KiB source-like files,
30 eight-byte same-length edits saved as complete files through FUSE, 31 retained
states. It remains one exploratory case, separate from the family proposed below.

Original R26 baseline: 88 KiB initial allocated Store, 216 KiB final, 128 KiB
growth, 13.709125 ms median paired save + Commit. It is not a released-v0.1.4
control. For later qualification, carry its unchanged harness to released v0.1.4
and collect a separately sealed control; do not relabel the original run. This is
not a prerequisite for the first ten-file smoke loop.

The existing candidate gates apply **only to that unchanged case**:

| Metric | Fixed candidate gate |
| --- | ---: |
| Initial allocated Store | <= 90,112 bytes (88 KiB) |
| Allocated growth over 30 commits | <= 65,536 bytes (64 KiB) |
| Final allocated Store | <= 155,648 bytes (152 KiB) |
| Median paired save + Commit | <= 15,000,000 ns (15 ms) |
| Correctness | 30 Created commits; all 31 states verified; cleanup passes |

These targets do not prove length-changing performance and must not be copied to
larger or different fixtures without a separately specified contract.

## 5. Later qualification: one new family, two explicit operation surfaces

Proposed family: **`delta_encoding_history`**, exactly two scenarios:

- `small-mixed-edits-30-sdk-v1`
- `small-mixed-edits-30-fuse-rewrite-v1`

Both use the same initial bytes and the same 30 resulting file-content changes.
They answer different operation questions and are never pooled or used as the
opposite arms of a speedup ratio. Compare released baseline versus candidate
within each scenario only.

Use one Store, Branch, and Workspace per run, with native directory Init and
three flat regular files:

| File | Initial bytes | Role |
| --- | ---: | --- |
| `small.txt` | 16,384 | Edited on odd commits. |
| `medium.txt` | 40,960 | Untouched witness for dirty-frontier/content-read checks. |
| `near-limit.txt` | 98,304 | Edited on even commits. |

Initial total is 155,648 bytes (152 KiB). This file-content total is unrelated to
the tiny case's coincidentally equal final-Store target. Use the existing
source-like 64-byte-line fixture construction, with fixed per-file seed inputs
and exact generator/manifest seals to be recorded before harness implementation.
Line count describes genesis only; edits operate on bytes, and later versions
need not preserve line length or UTF-8 structure.

Each edited file receives the following 15 operations, interleaved: step `2*j-1`
edits `small.txt`, step `2*j` edits `near-limit.txt`. `L` means that file's current
length immediately before the operation; offsets below are bytes. Every step is
one mutation call/save followed by one successful Created Commit.

| Visit | Operation | Offset | Delete bytes | Replacement |
| ---: | --- | ---: | ---: | --- |
| 1 | Overwrite head | 0 | 4096 | Released head-overwrite payload, 4096 B |
| 2 | Overwrite middle | L/2 - 2048 | 4096 | Released middle-overwrite payload, 4096 B |
| 3 | Overwrite tail | L - 4096 | 4096 | Released tail-overwrite payload, 4096 B |
| 4 | Insert middle | L/2 | 0 | Released middle-insert payload, 4096 B |
| 5 | Delete middle | L/2 - 2048 | 4096 | Empty; reverses visit 4 |
| 6 | Append | L | 0 | Released append payload, 4096 B |
| 7 | Prepend | 0 | 0 | Released prepend payload, 4096 B |
| 8 | Replace-grow | L/2 - 1024 | 2048 | Released grow payload, 4096 B |
| 9 | Replace-shrink | L/2 - 2048 | 4096 | Released shrink payload, 2048 B |
| 10 | Truncate tail | L - 4096 | 4096 | Empty |
| 11 | Zero-extend | L | 0 | 4096 zero bytes |
| 12 | Change matching pattern A | 4096 | 4096 | First 4096 B of released chunk-count seed-4 payload |
| 13 | Change matching pattern B | 4096 | 4096 | First 4096 B of released chunk-count seed-2 payload |
| 14 | Zero-filled replacement | 4096 | 4096 | 4096 zero bytes, as in seed-0 payload style |
| 15 | Return to original content | 0 | L | Exact genesis bytes for that file |

Reuse the frozen payload seeds/generator from the released definitions, not new
random seeds per sample. Visits 12-14 adapt the payload styles, **not the original
64-KiB span or its proven count effects**. They carry no preserve/increase/decrease
CDC-count claim at these new sizes. All division is exact for this schedule.
Before the final restoration, maximum file lengths are 26,624 and 108,544 bytes
(26 and 106 KiB), so neither crosses the 128-KiB threshold.

The fixture/oracle self-check must prove ranges, lengths, changed-content steps,
untouched witness, restoration at visit 5 to visit 3, restoration at visit 15 to
genesis, and exact expected bytes/digests for all 31 states. Visit 9 is a shorter
replacement, not an inverse of visit 8. Previous intermediate versions remain
retained throughout; FULL anchor choices remain product decisions, not fixture
instructions injected into the encoder.

```text
 Same source bytes + edit schedule
                 |
       +---------+----------+
       |                    |
 Public SDK range edit   Ordinary full rewrite through FUSE
       |                    |
       +---------+----------+
                 |
         Same live state owner
                 |
        Capture pending changes
                 |
        Shared Init/Commit pipeline
                 |
      Record time + total storage growth
```

SDK scenario: use `Client::edit_workspace_file_range` once per step, including
an explicit zero replacement for zero-extension. No Exec/POSIX mutation or
FUSE write substitute. Reuse the existing SDK timing/lifecycle helpers.

FUSE scenario: supply the complete resulting changed file through the existing
compiled importer/public Exec route, followed by public Commit. Preconstruct
and transfer inputs outside operation-local timing as in the frozen tiny case;
the actual full-file write remains inside the measured Exec. Reuse the same
metadata-normalization policy across baseline and candidate. Do not convert
this scenario into an SDK range edit or tempfile/rename case.

## 6. Measurements and correctness

Record Init separately from history mutation: source bytes read, files discovered,
initial allocation, initialization time, and shared admission/batching counters.
Do not time native Init inside an edit. It exercises the same pipeline with no
predecessor; it is also a control against unnecessary delta-base searching.

For each of 30 commits, record:

- SDK edit or FUSE Exec time, Commit time, and their contiguous paired duration,
  in integer nanoseconds. Lifecycle/setup/transfer/cleanup have separate scopes.
- Total allocated and apparent Store bytes after acknowledgement; cumulative
  growth equals current total minus the post-Init total. Include applicable
  sidecars and separately report spool/staging occupancy. Retain negative step
  differences if allocation accounting produces them; do not clamp them away.
- New/reused canonical payloads, encoded FULL/DELTA bytes, required base bytes,
  metadata/index/pack overhead where available, and FULL-fallback reason.
- Base candidates examined, base reads/decodes, target bytes assembled/hashed,
  delta attempts, dependency depth, SQL statements/transactions, transport
  requests, Store-lock time, and unchanged-file reads.
- Host process memory and codec/buffer reservations separately from daemon/cgroup
  memory and disk. Report sampled versus exact accounting honestly.

Use existing receipts first; add only missing counters needed to distinguish the
actual causes. Missing metrics are null with a reason, never invented zeros.
Owner-validated in-flight reuse and persisted authentication are distinct events;
repeated exact collision comparison is not automatically redundant hashing.

Verification is a separate pass/mode and receipt stream over the same retained
performance Store after its allocation observations are frozen. Reconnect and
verify every file's bytes, length, and declared metadata at
all 31 versions against the independent oracle, including the unchanged witness.
This is the new family's bounded history verifier; do not add a full-byte mode to
the frozen large-file SDK families. Prove FUSE visibility and route authenticity
in verification, outside operation timers. Verify exact visible-content restoration on visits 5 and 15 in both arms. For
the candidate, require reuse of the earlier whole-file payload ObjectId without
a duplicate payload record. For the released chunked arm, report its actual
chunk reuse without demanding history-independent file/extent-root identity.
Cross-arm canonical root equality is not required because the new small-file
canonical representation is part of the treatment.

After the performance sequence, a separately scoped read phase measures first
acquisition after process reconnect and repeated reads of selected early/late
versions, complete files and short ranges. Freeze exact selections/order before
collection. A process reconnect does not establish a cold OS cache. Validate
returned bytes after the read timer, with product authentication still measured.

A one-level delta must reconstruct using its FULL base without history replay.
Exercise missing/corrupt bases, invalid lengths, old/new-format reads, rollback,
and base lifetime in focused correctness checks; do not inject failures into
performance rows. No GC, VACUUM, background repacking, page-size changes, or
unaccounted postprocessing may manufacture storage savings.

## 7. Complexity targets and the counters that make them useful

Let `S` be small target size, `B` its eligible FULL base size, `H` retained history
length, `W/F` workspace bytes/file count, `E` large-file extents, `K` dirty inodes,
`M` affected namespace/tree nodes, and `U` newly admitted objects.

| Operation | Required scaling target | Observable evidence |
| --- | --- | --- |
| Base selection | Fixed candidate count independent of H; no history search | Candidates, predecessor/base lookups, history traversal |
| Small construction/encoding | Bounded passes over S and B; fixed codec settings/attempt count; no W/F/H scan | Assembled/hashed/decoded bytes, attempts, CPU |
| Small reconstruction | One new-format dependency edge; O(S+B) operand bytes, independent of H | Depth, records fetched, decoded/returned bytes |
| Memory | O(S+B+codec workspace) per active small operation; total constrained by existing ownership/concurrency budgets | Reserved capacities, active operations, host peak |
| Incremental Commit | Changed-content work + affected M/K work + admission; no routine W/F scan | Dirty/visited files, tree nodes, unchanged-file reads |
| Init | Bounded passes over imported bytes/entries, plus required sorting/index work | Source reads, discovery visits, spill/readback |
| Large range read | O(log E + visited extents + decoded payload bytes), allowing physical dependencies | Nodes visited, requested/physical bytes |
| Known large edit | Changed-range/tree-path work; no whole-file scan introduced by delta | Existing 56-case locality receipts |
| SQL/transport | Bounded count/byte cohorts; no automatic transaction/request per U object | Cohort occupancy, statements, transactions, requests |
| Large-to-small transition | Retained output plus needed tree/decode dependencies; no discarded-file/history scan | Retained versus physical bytes and traversal |

`O(1)` candidate count does not imply zero index lookup cost. A codec's strict
runtime bound must not be asserted solely from its interface; require bounded
configuration and measure scaling. Total memory includes codec workspace and
concurrent buffers, not just the 128-KiB file limit.

Payload history storage is `sum(FULL anchors) + sum(delta records)`. If one anchor
and similarly sized deltas suffice, the simplified form is `O(S + H*D)`, compared
to `O(H*S)` uncompressed distinct FULL versions. Worst case remains `O(H*S)`.
Count only distinct contents for CAS payload accounting; commit metadata grows
separately. Released v0.1.4 is already compressed/chunk-shared, so it is the real
control, not an invented unshared full-copy baseline.

The 30-step fixture diagnoses these mechanisms; it does not prove asymptotic
scaling in workspace size or history length. Use existing locality/history
families and bounded algorithm inspection for those obligations.

## 8. Focused coverage outside the first timed family

Keep these as small correctness/resource checks rather than multiplying the
first timed matrix:

- Cutoff results 131,071 / 131,072 / 131,073 bytes; repeated small-large-small
  transitions; preserve all histories and handle growth/shrinkage within one
  live session with one final persistent conversion.
- Near-cutoff shifted matches, including prepend and deletion: verify the
  configured raw-prefix matching window does not accidentally exclude relevant
  base matches. Compression savings are measured, not guaranteed by the API.
- Empty and very small content, unrelated/incompressible replacements, scattered
  edits, and repetitive content; bound FULL fallback and reconstruction.
- Metadata-only and no-change operations: no content hydration or extra payload
  encoding. Hard links, rename/replacement, and open handles retain their normal
  live semantics.
- A whole small-file read and a one-byte read; a short request may legitimately
  reconstruct the complete bounded small object. Count that amplification.

The larger realistic source-workspace campaign and full157 remain later
qualification under their own contracts. Do not claim this three-file synthetic
case establishes realistic many-file or long-horizon results by itself.

## 9. Later qualification sequence, gates, and evidence custody

1. Reconcile the detailed product spec/plan with the whole-file canonical model.
   Preserve the original tiny evidence. After the first ten-file smoke loop,
   collect the original case's separate released tiny control for its later comparison.
2. Before implementing this new benchmark, create its required GitHub issue and
   commit an executable contract revision. Seal the actual fixture/oracle and
   schedule manifests, exact metadata, public call counts, read selections,
   source-arm treatment, cache/preconditioning profile, resource budgets and
   sampling, timeouts, repetition/ordering policy, and artifact schemas.
   This draft does not claim those prerequisites are already satisfied.
3. Reuse existing runner, lifecycle, SDK edit, importer, and verification helpers.
   Add one canonical definition for the new family with its two thin entrypoints
   for performance/verification; do not build a parallel benchmark engine.
4. Collect a released-v0.1.4 baseline per scenario before candidate measurement.
   Freeze comparative/absolute numerical gates before candidate optimization or
   sampling. Do not borrow the tiny-case 64-KiB/15-ms gates for this new fixture.
5. Implement/verify the candidate and compare identical public operations and
   input bytes using identical harness artifacts across arms. Preserve all
   attempts; use the smallest failing case while diagnosing a cause.
6. Run required affected-family proofs and report no-go outcomes, not just gains.

Format treatment needs care: old/new native Init may produce different canonical
small objects and Store formats. For this family, create fresh Stores from the
same source files under each arm and include initial physical allocation in the
comparison. Do not force an incompatible prepared Store across arms. An upgrade
control is a distinct declared experiment, not silently mixed into native Init.

Mandatory correctness/mechanism outcomes are already fixed: all 30 mutations
create commits, all 31 histories verify, candidate whole-file exact-content reuse works, delta depth
is at most one for the new encoding, old supported histories remain readable,
no history/base hunt or unchanged witness content read is introduced by Commit,
public routes are authentic, budgets are respected, and cleanup succeeds.

New-scenario numeric latency/storage/read/memory ceilings remain **unfrozen**;
this document supplies no PASS classification for them. Freeze resource ceilings
before baseline collection and any baseline-informed performance gates before
candidate optimization. A useful storage claim must report initial/final total
allocation and growth, as well as per-operation regressions and read costs; a
smaller encoded delta alone is insufficient.

Use the approved topology: macOS owns SDK/coordinator, canonical construction,
spool and SQLite; Docker owns the Linux live core, FUSE and workload. Retain
2 CPUs / 2 GiB / no swap / 256 PIDs for the container and the existing shared
measurement lock. Builds, measurements, and proofs remain serial.

Proposed result layout, using existing receipt conventions:

```text
<new-run-directory>/
  identity.json                 source/product/harness/image/contract identities
  fixture-manifest.json         input, schedule and oracle seals
  performance/                  baseline/candidate, separated by scenario
  verification/                 separate history/visibility/resource proofs
  reads/                        separately timed first/repeated acquisitions
  growth.csv                    per-step allocation, payload and base accounting
  summary.json                  raw operands, comparisons, gate applicability
  README.md                     claim scope, failures, limitations and results
```

One history is not 30 independent repetitions. Preserve raw integer units and
per-step results; use medians/ranges with their sample counts. Unavailable
mandatory evidence prevents admission. No existing benchmark result is changed
by this plan.

[preserving]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/benchmark/fs-bench-pro/families/edit_length_preserving/mod.rs
[changing]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/benchmark/fs-bench-pro/families/edit_length_changing/mod.rs
[count]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/benchmark/fs-bench-pro/families/edit_canonical_chunk_count/mod.rs
[common]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/benchmark/fs-bench-pro/workload/sdk_edit_common.rs
[calls]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/benchmark/fs-bench-pro/src/sdk_file_edit.rs
[verify]: https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/benchmark/fs-bench-pro/src/sdk_edit_verify.rs

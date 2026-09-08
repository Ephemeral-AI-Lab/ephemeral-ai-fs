# Storage architecture v3 implementation progress

Status: **milestones 0–1 implemented; milestone 2 in progress; implementation incomplete**.
Updated 2026-09-08. Unchanged-product baseline built; all three smokes and their
independent mounted historical verification passed. The FULL/RAW schema-6 switch has an intermediate small-file correctness observation; integrated v3 is not complete.
This ledger is the single disposition of this implementation task, not a copy
of the historical release matrix.

Authority: [implementation plan](implementation-plan.md),
[architecture](storage-architecture-spec.md), [format](sqlite-storage-format.md),
[boundary](storage-efficiency-boundary.md), [review disposition](review-disposition.md),
[evidence custody](evidence.md), [PR #80 smoke plan](storage-smoke-test-plan.md),
and [prospective smoke decision](implementation-smoke-contract-v1.md).

## Source and document custody

| Item | Actual identity / disposition |
| --- | --- |
| Original user checkout | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`, local main `c374f8c3b25e923831d25af2304c0d6bf8ac0d9e`; unrelated tracked/untracked edits preserved |
| Selected current product source | Freshly fetched origin/main `28177560c8f049c02192e18c263cdc5543c1ab52`, exactly the source trace baseline; original local main is 97 commits behind |
| Isolated worktree | `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3` |
| Implementation branch | `codex/storage-v3-implementation` |
| Initial worktree correction | Initially created at original local main, then switched to fresh origin/main before reading/changing product or executing any binary; no historical product run |
| PR #79 | OPEN, `codex/storage-spec-revision`, head `9f748514e651c322b16bd7cdb78813e492039dbf` |
| Architecture correction | `555d91f0cd74148364331e24acf0ba14408d7c78`; six requested reviewed documents unchanged between correction and PR #79 head except implementation-plan.md, which was added subsequently |
| Implementation plan | `34336e5b798a1d59efccd83b2412cea3944963fa`; implementation-plan.md byte-identical at selected PR #79 head |
| Later PR #79 differences | README additions and implementation/handoff plan additions; no product change or alteration to the five architecture/format/boundary/review/evidence files |
| PR #80 | OPEN, `codex/storage-smoke-plan`, head exactly pinned `d9ec9c6714ca31adb7a337d2ac0f40976513908c` |
| Planning access | Reviewed docs copied using git object reads/restores into isolated worktree; neither planning PR merged into main |
| Review decision search | Issue comments, PR review summaries and inline review comments for #79/#80 returned empty lists at inspection; no pre-existing owner format or numerical approval found there |
| Frozen experiment docs/helpers | `1c7c9235115d1b4f21bc2eae7af822552b7be3ed`, read via git show only; historical product not checked out/executed |
| Frozen source tip | `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed` |
| Manifest SHA-256 | `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; local copy matches pinned bytes |

## Owner decisions

- **Approved in this task:** narrow 0.1.4 exception with the proposed new-Store-only
  scope. Explicit schema 6 / wire 1 creation, legacy rejection in the new binary,
  existing compatible tools retained for old Stores. No converter, same-binary
  legacy reader/writer, automatic/in-place migration, merge or deployment.
- **Approved in this task:** the concrete smoke fixtures, repetition/resource policy
  and numerical gates in implementation-smoke-contract-v1.md. Subsequent owner
  instruction confirms autonomous implementation without further routine questions.
  No acceptance criterion has been chosen after observing candidate results.

## Completed independent preparation and source findings

Read all six requested reviewed documents in full, the pinned smoke plan,
benchmark/AGENTS.md, benchmark rules, quick start, 0.1 compatibility rule and
the original experiment's reproduction/immutable-input/lifecycle contract.
No other AGENTS.md exists in this worktree or the inspected parent chain.
Read the applicable Ponytail skill; the task's smoke-only instruction overrides
its generic self-test advice. No agent delegation was requested or used.

Read current runner.py and runtime.py in full and their Cargo manifest, plus
the complete pinned Python history helper, host history driver and workload
importer/observer. SDK public call bodies through execution/output dispatch
were inspected; the remaining implementation-plan source list stays mandatory
before edits to each owner. Partial schema/main inspection is not marked a
completed owner review.

Concrete adaptation findings:

1. Current runner has 17 historical family selectors, no PR #80 entrypoints.
   Existing `--smoke` selects a registered minimum, not these histories.
2. `_host_acquire` SDK setup qualifies all three edit families for a size.
   Do not invoke that path for this limited task or inherit its verification
   populations. Reuse runtime/build/lock primitives and exact native inputs.
3. `execute_selected` removes owned host samples during cleanup. These history
   smokes must retain their Store and evidence for independent reopen verification.
4. #72 `history::import` uses ordinary `fs::copy` into retained regular files
   and preserves unchanged entries; it does not use SDK edits or tempfile saves.
   Tempfile/rename coverage belongs to the separately declared frequent-edit smoke.
5. #72 Python `preparation` validates source tree and blob identities, but its
   cache creation needs protected publication/coordination under current rules.
   Preserve original inputs and oracles; do not assume their existence proves
   full acquisition validation.
6. #72 `history::observation` omits `-journal` in its sidecar loop; new accounting
   must include every existing required sidecar, outside operation timing.
7. #72 host driver calls presentation recovery after recording a publication.
   New results must retain that failure and recovery timing, never turn the
   original public failure into an unqualified successful sample.
8. Current schema has existing v4-to-v5 conversion on open. The packed switch
   must reject legacy before configuration/migration; the approved new-Store-only
   policy does not authorize carrying that migration into the new reader.

Input preflight is read-only preparation, not executable product verification:
all five local Git `ls-tree -rlz --full-tree` digests match the frozen manifest.
All five prepared input directories and oracle files exist, but complete cache
content validation remains pending acquisition. The five selected SHAs/counts
are recorded in the smoke draft. No other checkpoint was selected or executed.

Available external environment at inspection: macOS Darwin arm64, Docker server
29.5.2 responding, 419318792 KiB free on the workspace filesystem. No runtime
was created. Runtime image identity, binary identity and actual resource peaks
are **not available** until matching builds/runs; Docker availability is not
proof of mounted FUSE correctness.

Pinned helper SHA-256 values:

| Helper at experiment docs commit | SHA-256 |
| --- | --- |
| shared/deepseek_history.py | ae924199b4f8ee14ec830d93334a272663e3c7719afc6943cf41d7a35a1caff1 |
| src/history.rs | 49b9228aa1523628d63afdd0f658f069a96203c232c3f0bec6453f0aaa37c44f |
| workload/history.rs | f88e1c1129836e5e3eff7c26eb4bcc0bc3894ef28a231245ba9564bd2d00c596 |

## Compact implementation / deletion / execution ledger

States: DONE-PREP is documentary/source preparation only; PENDING is neither
implemented nor source-reviewed as a completed change; NOT-RUN supplies no
executed verification. An implemented item will separately record source review
and exact smoke evidence, or explicitly record that the three-smoke scope does
not exercise it.

| Milestone / item | Status | Evidence or next owner |
| --- | --- | --- |
| 0: source/PR/doc identity and isolation | DONE-PREP | Custody above |
| 0: release/new-Store/legacy decision | DONE-PREP | Owner reply approving narrow exception |
| 0: fixture/budget/comparison decision | DONE-PREP | Owner approved draft before observations |
| 0: three entrypoints, protected input/oracle custody | DONE-PREP + EXERCISED | storage_smoke host/workload/shared runner; exact Git inputs and synthetic manifests |
| 0: unchanged baseline all three smokes | PASS | implementation-baseline.json; full FUSE historical oracles and owned cleanup |
| 1: reusable canonical/finalized ownership, selected ID/location transfer | IMPLEMENTED / SOURCE-REVIEWED | 86d04b95e; M1 observations below; all-Init switch remains M2 |
| 1: page seen insert/membership/order/duplicate semantics | IMPLEMENTED / SOURCE-REVIEWED | objects.rs spill owner; streaming scalar callers included |
| 1: page offset flush/location lookup, pending visibility and failed-owner handling | IMPLEMENTED / SOURCE-REVIEWED | Existing spool and pending/absolute union |
| 1: sealed buffered IdOrder; delete tiny read/seek index reconstruction | IMPLEMENTED / SOURCE-REVIEWED | Preserve EOF/truncation and memory accounting |
| 2: exact schema 6/new creation/legacy rejection and SQL/FKs | IN PROGRESS / PARTIAL SMOKE | schema.rs, statements, v6.sql and all SQL assumptions |
| 2: FULL/RAW pack framing, bounded reader and integrity/authentication | IN PROGRESS / PARTIAL SMOKE | objects/{pack,read}.rs, SnapshotReader and generic sources |
| 2: shared <=2U admission, closed reservation, connection-unlocked late validation | IN PROGRESS / PARTIAL SMOKE | objects/admission.rs and every writer |
| 2: byte/parameter bounded pack/locator INSERTs, incremental counters | IN PROGRESS / PARTIAL SMOKE | Shared transaction owner and effective SQLite limits |
| 2: all Init routes move finalized outputs, no parent payload clone | IN PROGRESS / PARTIAL SMOKE | layerstack.rs serial/parallel/empty/nonempty/fallback |
| 2: delete whole-Init/nested permits and all four unsafe cleanup calls | IN PROGRESS / PARTIAL SMOKE | Indivisible all-writer switch; metadata siblings retained |
| 2: targeted FIFO handoff and poison/abandonment behavior | IN PROGRESS / PARTIAL SMOKE | Existing schema gate, no new scheduler |
| 2: stage/head/base/no-change/publication/finalization semantics | IN PROGRESS / PARTIAL SMOKE | workspace/staging/branch/lifecycle/live backing |
| 2: canonical/query/accounting preservation and physical receipts | PENDING | query/records/telemetry/SQL |
| 3: Zstandard exact framing/window/checksum/output and actual scratch bound | PENDING | One codec binding, no silent RAW fallback |
| 3: target/base internal group waves, slot/order/duplicate/output accounting | PENDING | Shared reader and duplicate validation |
| 4: same-inode and pinned-path physical predecessor | PENDING | changes/content/workspace-core/resolve callers |
| 4: original-emission first spans, complete/captured transfer | PENDING | capture + rope builders; no second CDC pass |
| 4: one forward old-extent cursor and localized seeks, bounded work | PENDING | rope traversal and content constructors |
| 4: shallow delta matcher/emission/read and authenticated FULL-base closure | PENDING | Shared pack/admission/read and publication |
| 5: root-bound checkpoint facts, remove redundant trusted rehash/cache clone | PENDING | workspace SnapshotReader + live backing |
| 5: lazy reconciliation view and indexed exact/ancestor/prefix scopes | PENDING | changes/reconcile; overlapping fingerprint work remains |
| 5: obsolete encoder/SQL/helper deletion and final module ledger | PENDING | Implementation plan KEEP/MERGE/REPLACE/DELETE |
| Each slice: matching builds and smallest affected agreed smoke | M0/M1 PASS; M2 NOT-RUN | Source-specific M1 evidence below; no packed writer enabled |
| Final integrated candidate: all three repeated comparisons | NOT-RUN | Diagnostic baseline exists; final three pairs remain open |
| Final correctness/history/routes/resources/cleanup | NOT-RUN | Source review cannot tick these |
| Final evidence/report/source agreement and implementation PR | PENDING | No completed implementation PR claim |

Eight review follow-ups map respectively to admission; reconciliation view;
all-Init ownership; Store/scratch batching; spill I/O/location transfer; FIFO;
grouped reads; whole-file handoff. **All eight remain open as complete integrated obligations**; spill/page components are implemented and the gate/reader/admission switch is in progress. The reviewed design's “resolved” labels are not implementation status.

## Current result table and coverage

| Smoke | Baseline allocated bytes / elapsed / resources | Final candidate | Correctness/history/route/cleanup |
| --- | --- | --- | --- |
| DeepSeek first five | 12,320,768 allocated bytes; 1,095,992,417 ns foreground; resource receipt in implementation-baseline.json | Not built/run | Baseline and M1 PASS |
| Frequent edits, SDK and ordinary separately | Four separate histories in implementation-baseline.json | Not built/run | Baseline and M1 PASS |
| Small-file Init/readback | 1,441,792 final allocated bytes; Init 7,028,667 ns; separate read timings in implementation-baseline.json | Not built/run | Baseline and M1 PASS |

No material-storage, latency, Git-proximity, universal correctness or capacity
claim exists. Broad parser/race/conflict, oversized-record and failure matrices,
157-state replay and release qualification remain outside executable scope.
Their specified safeguards still require implementation/source review.

All seven completion conditions in the user request remain open: complete
implementation/deletion; final matching builds; final three-smoke correctness;
material joint storage/speed evidence; complete coverage/identity reporting;
final docs/source/receipts agreement; committed/pushed implementation PR.
Compatibility policy alone is now resolved and must not be requested again.

## Milestone 0 baseline receipt

[Compact baseline values and raw-evidence links](implementation-baseline.json)
record all six separately reported histories. All 33 historical mappings verified
(5 DeepSeek +24 edit mappings +4 small-file mappings), including initial states
and no-change aliases. These are mappings, not 33 created Commits.

Host binary SHA-256: `ce6a81a629d7d1adafaea74933acc502edb21e4ca650a396172f3574f26a0746`.
Runtime image: `sha256:134cab8527a2cb686565bdab0aa749d4617ab7d85ef8aab14c0a10be60609c89`.
Product seal: `3c797bc6dbfd9b03b919c270b609cad839000b68d67e34f3b00d24717e07f39a`.
Combined source seal: `3564c1a834fc76f2157c2308c9bdaf6a0f05439a6098638b8af75be232138117`.
Built at docs commit `96e796431964f7a00af7fd1d7cc647029f2c5eaa` plus the
subsequently committed smoke harness. Product crates remain byte-identical to
selected origin/main. The dedicated smoke image build skips the Dockerfile
workload self-check; its unused runtime-check stage was not built. No unit,
fuzz, race, crash or historical family suite ran.

One mistyped image tag (`3564c1a834fc76f21` instead of `3564c1a834fc76f2`) failed
image lookup before runtime/Store creation; the corrected command was used. No
product sample was discarded. Original build log is under the external runs
folder, builds/baseline-image-1.log. All other initial smoke attempts passed.

Source reading advanced through current host main.rs in full; schema.rs,
workspace.rs, staging.rs, branch.rs, query.rs, records.rs, statements.rs and v5.sql;
objects.rs production owners through admission and layerstack.rs production Init
paths through NativeImport. The exact additional construction/runtime source list
remains mandatory before touching those owners. Systematic-debugging skill read;
task-specific smoke-only verification overrides generic extra-test advice.

## Milestone 1 — implemented and source reviewed

Moved actual derived spill/seen/location/ID-order owners to objects/spill.rs,
retaining the construction facade. Page SQL respects effective parameter and
statement limits; seen RETURNING results restore first input occurrence order.
Streaming admission callers now consume page flags, including a pending-duplicate
flag before a possible later drain. OFF-journal mutation failures invalidate the
owner. Offset transfer drains the known absolute/pending union without payload
header replay and includes the threshold-triggering record. Pending locations
remain readable until successful data/index flush. Selected location/member and
borrowed batch visitors resolve pages and preserve order/duplicates.

IdOrder uses 64-KiB buffers, explicit fallible seals and truncated-tail rejection.
The private doc-hidden into_resumable handoff is now fallible, propagated through
capture; no public SDK signature changed. Producer spill-buffer allocations reserve
ID-buffer space within the existing aggregate I/O allowance. Owned selected output
moves through merge_prevalidated; no canonical producer Vec clone is needed there.
All-Init direct sink/occupancy selection remains milestone 2, not completed by this.

Executed: m1-small-1 and m1-deepseek-1 performance plus full historical mounted
verification/cleanup PASS at source seal
335a06d6912fc62e926fbfd33da696493aca3e7d61d8401bf5243d30f6a2c415.
The final borrowed-batch scalar-lookup correction (not used by those smoke flows)
then built at source seal
ccc17643cac2fe1ffa315a97e32a1f1a5555dab9daa071a1ea2ad5735dbf278e;
m1-edits-1 performance and all 24 initial/retained/no-change mappings passed
historical FUSE verification and cleanup. All evidence lives under the previously
recorded external runs root. These are intermediate source-specific observations,
not the required final integrated candidate comparisons.

The subsequent fallible ID seal propagation is a source-reviewed error-handling
completion; its matching host build m1-host-4.log passed. Its next matching runtime/smoke remains pending the coherent M2 switch. No oversized
ID/seen-index population was injected: disk-seen thresholds, truncated ID failures
and OFF-journal failure schedules are not exercised by these smokes. Their code
and callers were reviewed; no unit or extra fault suite was run. Single-sample
operation times show no actionable regression: 8-MiB SDK steps remain roughly
6–8 ms and ordinary complete replacements roughly 76–86 ms. Retained allocation
is intentionally still raw-schema storage at this milestone; no material-storage
improvement is claimed.

Draft implementation PR: https://github.com/Ephemeral-AI-Lab/layerfs/pull/81.
It remains unmerged and explicitly incomplete. All eight review obligations that
require packed admission, hints, grouped reads, gate replacement or reconciliation
remain open; only the milestone-1 spill/page portions are implemented.


## Milestone 2 preparation — no format switch or smoke claim yet

Work in progress adds the prescribed private pack/read/admission modules. Wire
framing and bound checks, FULL/RAW construction, grouped target/FULL-base reads,
streamed oversized comparisons, one-probe/one-recheck prepared admission and bulk
pack/locator insertion are being connected to existing writers. These are not
reported as complete or exercised while callers still use schema 5. Compression
is not enabled; its required codec is milestone 3. FIFO successor channels replace
broadcast wakeups; complete permit scope changes remain open with the writer switch.

Source review found a payload-sized copy in referenced_objects; the Bytes branch
now borrows the existing decoder's slice while retaining exact structured-role
parsing. All production callers were traced. Oversized comparison reuses
ObjectId::from_reader and its existing domain-separated hash; no external hash or
SQLite dependency source was patched.

Build preparation observations (not smoke verification): m2-host-preparation-1
compiled the pack/read code before later gate/admission edits and is not a
matching final artifact. m2-host-preparation-2 failed; the host builder swallowed
Cargo diagnostics. The existing host build error branch now prints captured
stderr, matching its image-build behavior. m2-host-preparation-3 retained the
actual compiler error: rusqlite ValueRef does not implement ToSql. The correction
uses borrowed ToSql parameters, avoiding a pack-BLOB copy or dependency patch.
All failed build logs remain under the declared runs root. No smoke was executed
against any of these intermediate sources.

Owner steering: implementation patches stay within this isolated repository;
external library/dependency sources are not modified. Existing declared evidence
and fresh smoke-state locations retain their original custody and accounting.


### M2 first FULL/RAW small-file observation and correction

`m2-small-1` performance and all four independent historical mounted oracles
passed, with authenticated runtime/FUSE route and owned cleanup PASS. This is a
raw-only checkpoint, not v3 qualification. Identity: source commit
`86d04b95ed360835f1a861fd7104218794deb142` plus the retained implementation diff;
source seal `c2a8646c3f52bdc50b711a262a22ed4e58eeedc64aceec66c94c4a03fcac2972`;
product seal `8630a6a28ed5cbf3b6c8257336fe36ea366b544899985f17b09c490883582d93`;
host SHA-256 `b135c4d8c0cb4b236c817b392b05ed2d375d2a675255fec73e70bf5309be342f`;
image tag `layerfs-bench-infra:c2a8646c3f52bdc5` (immutable image ID in the run's
identity.json). The exact schema-6 DDL hash is
`53bda8792a601683038af508b183986e1f880a3c0f2158dc3cc9f6bbf1fced50`.

| Diagnostic small-file case | Unchanged baseline | First FULL/RAW | Disposition |
| --- | ---: | ---: | --- |
| Initial/final allocated bytes | 1,441,792 / 1,441,792 | 1,441,792 / 1,441,792 | No storage improvement; compression still required |
| Init ns | 7,028,667 | 7,842,750 | +11.6% single observation |
| First read ns | 19,658,084 | 29,530,000 | Meaningful regression; exceeds prospective read allowance |
| Repeated read ns | 7,439,041 | 7,722,917 | +3.8% single observation |
| Step 1 Exec+Commit ns | 8,016,916 | 8,917,084 | +11.2% single observation |
| Step 2 Exec+Commit ns | 6,796,542 | 9,803,666 | +44.2%; unresolved |
| Step 3 Exec+Commit ns | 7,294,833 | 10,962,250 | +50.3%; unresolved |

These valid samples are retained. They do not satisfy the final frozen gates and
are not statistical baseline/candidate pairs. Host first-read CPU rose from
4,891,416 to 7,242,500 ns. Source trace found redundant singleton batch-slot
planning, SnapshotReader's temporary cache-argument copy, and ObjectBuffer's
rehash of an already-authenticated base source. The correction keeps real spill
reauthentication, uses the existing trusted source boundary, restores scalar
request handling, and retains the existing demanded-object cache. The affected
smoke will be rerun on matching artifacts; no acceptance gate changes.

Additional build failure `m2-host-1.log`: an overly broad textual edit changed
both direct-sink and producer-only get methods. The producer-only implementation
was restored; only the coordinator sink can resolve pending/database objects.
`m2-host-2.log`, `m2-host-3.log`, and `m2-host-4.log` passed after the respective
recorded source corrections. No failed smoke has been discarded.

Named scope still open: complete codec/delta/hint handoffs, fallback source-pass
work receipts, remaining obsolete construction helper removal, physical/group
receipts, reconciliation cleanup, and final three paired smokes. Further source
reading completed changes.rs production through line 2352 and workspace-core
file_edit.rs production through 912, lib.rs production including its post-test
LiveWorkspace body, and namespace.rs production including rename. It found the
existing FrontierInodes growing-prefix merge; this remains an explicit correction
item, not an accepted finite-cap exception. No new verification suite was run.


M2 correction observation `m2-small-2`: performance and all four historical FUSE
oracles passed; cleanup passed. Source seal `2d092b67688af77c` (full identity in
receipt), immutable image `sha256:1595d1979fe58d966903b7a1efea1f1d0b116a3547341ab4e1bbefd8e4bfe4ac`.
First/repeated read: 23,176,834 / 7,348,500 ns. Exec+Commit steps:
8,986,333 / 7,422,917 / 8,621,459 ns. These diagnostic values are inside the
prospective read/step allowances versus the initial baseline. Init was 9,228,167 ns,
above the 8,785,834-ns prospective limit; this valid result remains retained and
qualification stays open. Initial/final allocation remains 1,441,792 bytes.
No numerical gate or repetition policy changed.

Subsequent M2 source cleanup records attempted fallback file reads before errors
escape, merges worker source counters once, and reports source construction passes
in the existing Init receipt. The stopped parallel attempt plus one serial fallback
retains admitted records; the final receipt includes attempted file/byte work.
The former parent-collection Init constructors now compile only as existing test
reference helpers, not production routes. The small smoke's host prints the
existing Init receipt outside operation timing; the same harness change will be
used for final baseline pairs. No external library/dependency source was edited.
An Init name conflict is classified only after the name INSERT fails, preserving
admission/integrity errors from earlier phases.

`m2-host-6.log` retained a compile failure from mistakenly applying `?` to the
existing infallible JSON emitter. Its actual body was read and the call corrected;
`m2-host-7.log` passed. The later name-error classification fix still needs its
matching build. These are preparation observations, not executed verification.

Full second-observation host custody: `a936569f25425d493ee7a47ef77ec4e9c14103ef2226be674d3d0bae49caa645`; source `2d092b67688af77ccfee5920b8c314af454d82d1a211556fd9e3d6f75f6263a1`.

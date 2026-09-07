# Storage architecture v3 implementation progress

Status: **milestone 0, source custody/preparation; implementation incomplete**.
Updated 2026-09-08. No product build, smoke, test or storage observation yet.
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
| 0: three entrypoints, protected input/oracle custody | PENDING | Existing runner/runtime and pinned helpers traced |
| 0: unchanged baseline all three smokes | NOT-RUN | Await approved smoke contract and matching harness/build |
| 1: reusable canonical/finalized ownership, selected ID/location transfer | PENDING | objects.rs + all Init/Workspace callers |
| 1: page seen insert/membership/order/duplicate semantics | PENDING | objects.rs spill owner; streaming scalar callers included |
| 1: page offset flush/location lookup, pending visibility and failed-owner handling | PENDING | Existing spool and pending/absolute union |
| 1: sealed buffered IdOrder; delete tiny read/seek index reconstruction | PENDING | Preserve EOF/truncation and memory accounting |
| 2: exact schema 6/new creation/legacy rejection and SQL/FKs | PENDING | schema.rs, statements, v6.sql and all SQL assumptions |
| 2: FULL/RAW pack framing, bounded reader and integrity/authentication | PENDING | objects/{pack,read}.rs, SnapshotReader and generic sources |
| 2: shared <=2U admission, closed reservation, connection-unlocked late validation | PENDING | objects/admission.rs and every writer |
| 2: byte/parameter bounded pack/locator INSERTs, incremental counters | PENDING | Shared transaction owner and effective SQLite limits |
| 2: all Init routes move finalized outputs, no parent payload clone | PENDING | layerstack.rs serial/parallel/empty/nonempty/fallback |
| 2: delete whole-Init/nested permits and all four unsafe cleanup calls | PENDING | Indivisible all-writer switch; metadata siblings retained |
| 2: targeted FIFO handoff and poison/abandonment behavior | PENDING | Existing schema gate, no new scheduler |
| 2: stage/head/base/no-change/publication/finalization semantics | PENDING | workspace/staging/branch/lifecycle/live backing |
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
| Each slice: matching builds and smallest affected agreed smoke | NOT-RUN | No binaries/images built |
| Final integrated candidate: all three repeated comparisons | NOT-RUN | No baseline/candidate metrics |
| Final correctness/history/routes/resources/cleanup | NOT-RUN | Source review cannot tick these |
| Final evidence/report/source agreement and implementation PR | PENDING | No completed implementation PR claim |

Eight review follow-ups map respectively to admission; reconciliation view;
all-Init ownership; Store/scratch batching; spill I/O/location transfer; FIFO;
grouped reads; whole-file handoff. **All eight remain unimplemented** in this
worktree. The reviewed design's “resolved” labels are not implementation status.

## Current result table and coverage

| Smoke | Baseline allocated bytes / elapsed / resources | Final candidate | Correctness/history/route/cleanup |
| --- | --- | --- | --- |
| DeepSeek first five | Not run | Not built/run | Not exercised |
| Frequent edits, SDK and ordinary separately | Not run | Not built/run | Not exercised |
| Small-file Init/readback | Not run | Not built/run | Not exercised |

No material-storage, latency, Git-proximity, universal correctness or capacity
claim exists. Broad parser/race/conflict, oversized-record and failure matrices,
157-state replay and release qualification remain outside executable scope.
Their specified safeguards still require implementation/source review.

All seven completion conditions in the user request remain open: complete
implementation/deletion; final matching builds; final three-smoke correctness;
material joint storage/speed evidence; complete coverage/identity reporting;
final docs/source/receipts agreement; committed/pushed implementation PR.
Compatibility policy alone is now resolved and must not be requested again.

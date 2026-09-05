# Issue 47: credible plan for sub-400 ms Commit

User-approved Commit objective, 2026-09-06: **below 400 ms is sufficient**. This replaces the earlier 300 ms exploration ambition for the current Commit work. Review by three read-only subagents and the parent inspected task `01a07283-4940-7ba1-9b4c-548ddc459e35`, source `c090f4684` plus active full-file-finality changes in `/Users/yifanxu/.codex/worktrees/7311/layerfs`.

## Scope and sequencing

Use the original `tiny-bulk-create-100` and `tiny-bulk-delete-100`, 20,000 affected files / 100 MiB, seed 1. Keep the existing host SQLite, Docker daemon/FUSE, no-data-mount topology. Hold mixed-v3 tier implementation and #49 producer simplification until this focused Commit attempt is assessed. No tier-change notification accompanies this review.

This is a Commit-phase milestone, not replacement of #47's full-lifecycle acceptance. Do not close #47/#46/#39 on a Commit-only result. No new per-phase budgets, percentage gain, multi-seed campaign or sub-300 ms requirement. Preserve useful intermediate changes. Final independent proofs remain at the agreed final stage, not after every experiment.

## Verified progress

| Original case | Latest retained Commit | Implication |
|---|---:|---|
| Create, `issue47-consumer-create100` | 679.934 ms | Needs about 280 ms less elapsed time for the new objective |
| Delete, `issue47-located-reference-delete100` | 304.405 ms | Already satisfies the Commit objective in this sample; stop deletion timing-only work |

Results and source identities are in the implementation owner's append-only `issue47-subsecond-workspace-results.md` and corresponding `benchmark-results/host-store/results/` receipts. These are individual observations from different revisions, not a same-source final qualification pair.

Already completed: shared spools; common bounded output driver/checked consumer; removal of most candidate spill/readback; sequential final-record processing; checkpoint validation at final record emission; consolidated exact membership and batched duplicate comparisons; replacement of linear-scan seen-set overflow; shared directory edge facts; located reference updates. Do not propose those again as unimplemented work.

Latest create timing:

| Component | Time |
|---|---:|
| File output pipeline | 431.588 ms |
| Consumer work, inside pipeline | 286.724 ms |
| Post-pipeline directory/metadata preparation | approximately 83.136 ms |
| Namespace/reference phase | 52.006 ms |
| Final candidate selection | 21.261 ms |
| Remaining structural admission | 64.173 ms |
| Checkpoint | 17.022 ms, including 3.402 ms retirement |
| Other | approximately 10.748 ms |

The pipeline contains consumer work and waiting; nested times are not additive. Outside-pipeline work totals about 248.346 ms. Even a hypothetical pipeline equal to today's consumer service plus an unchanged tail gives about 535 ms. Thus producer acceleration or fresh-file finality alone cannot be promised to achieve 400 ms.

## Step 1: finish the active full-file finality change

`ObjectBuffer::build_complete_file` in Store `objects.rs` uses the existing complete-file rope builder without constructing a reference index, checks final length, then marks its output reachable by the builder's finality contract. `FrozenFile::build` in Workspace `changes.rs` uses it for fresh/full rebuilds; captured and incremental/provisional paths retain selection.

The important boundary is unchanged: incomplete file output stays private and bounded until construction and length checking succeed. This removes reference parsing/indexing and per-file graph selection, not canonical chunking, deduplication or storage checks.

Finish existing equivalence checks for empty, chunk/tree-boundary, varied/repetitive data and spilled output; check identical root, object-ID set/count/bytes, wrong length and read failure. A repeated zero-filled file may deduplicate enough not to spill; test actual spill on varied data without weakening the equivalence checks.

Then run one original create-100 performance sample using the existing serial host lock. Preserve exact source/input identities, candidate/inserted/reused totals, Store footprint, resource bounds and cleanup. Reuse a newer matching result if the owner already completed this step.

**Decision:** if complete Commit is below 400 ms and checks hold, stop performance tuning this path. Otherwise identify the remaining gap from the new receipt; do not reuse old timings as the new implementation's breakdown.

## Step 2: choose one evidenced follow-up

Do not automatically implement every possibility. Add only the missing narrow attribution needed to pick a substantial remaining cost.

### A. Validated owned output, if authentication remains material

Construction establishes identity, but ordinary `CanonicalObject` delivery loses provenance and the consumer hashes/checks framing again. Measure consumer authentication separately from SQL, sorting and ownership handling.

Reuse and strengthen the existing private authenticated-object representation to guarantee complete identity and framing with immutable owned bytes. Carry that guarantee across in-memory delivery so admission need not recompute it. Do not merely rename a type or move the same duplicate hash to another phase. Fresh spill/durable reads must authenticate; collision comparisons against existing Store bytes remain exact. Avoid a trust toggle or bypass for a benchmark.

Keep this slice only if it removes the identified repeated pass with equivalent malformed-input, corrupted-spill and conflict behavior. If authentication is too small to explain a meaningful part of the gap, skip this as the primary performance experiment.

### B. Shorter finalization tail, if preparation/structural delivery remains material

Use already-established completed-file facts once, with bounded result records and one ordered namespace/reference owner. Check for remaining result I/O, repeated validation without new evidence and structural delivery passes; preserve checks by moving them to the fact-producing boundary.

If sequential removal is insufficient, consider consuming bounded completed-file result blocks while later files are still produced. Prepare only facts whose dependencies are final. Final reference counts, roots and publication wait for their true dependencies and all worker joins. Measure total Commit: doing preparation on the consumer can also delay SQLite work, so no additive claim from consumer idle plus preparation timers.

Do not admit provisional records, remove unseen-alias accounting, add an unbounded completion map, or start a broad scheduler rewrite. Explicitly replan if overlap requires a larger change than this scope supports.

## Conditional concurrency and #49

Keep one producer for the first two steps. If the remaining profile shows producer starvation after consumer/tail reductions, a small bounded concurrency experiment may be proposed separately. The current serial dirty loop must not simply be invoked multiple times: that duplicates work and violates result order. General task-driver/configuration simplification belongs to #49 and remains deferred rather than becoming a prerequisite here.

## Acceptance and stopping

1. Stop further delete optimization: its retained 304.405 ms meets the revised objective. Recheck on the delivered source only when shared changes or final integration require it; no unchanged reruns to chase 300 ms.
2. Target complete original create Commit below 400 ms with physical retirement included. Do not subtract the fastest retirement observation or defer cleanup to End/background work.
3. After the active finality slice and one directly justified follow-up, reassess. If a substantial gap remains, document the measured required mechanism; do not launch a marginal-tuning campaign or assert the target is guaranteed.
4. Before calling the shared Commit milestone complete, retain current-source evidence for both affected cases and the applicable focused semantic checks. Respect the existing no-repeat rule for unchanged passing work and final-stage-only independent proof schedule.
5. Report Commit success separately from Exec/full-lifecycle status. Parent closure, tier migration and #49 are separate subsequent decisions.

The credible expectation is that below 400 ms is worth pursuing through identified repeated work and a shorter critical path. It is not yet established by measurement. No product code, benchmark run, issue-body change or task notification is performed by this planning review.

## Owner update after completed finality slice

The latest user clarification accepts a sound result around400–450ms for this Commit milestone, while preserving exact timings and not labeling a450ms receipt strict sub400 PASS. The300ms ambition is superseded. Full-lifecycle #47 acceptance is unchanged.

Step1 is complete on `c6683078c`: equivalence/error checks pass and `issue47-complete-file-create100` records710.826ms Commit,413.860ms pipeline and291.501ms nested consumer service. The296.966ms outside the pipeline includes60.862ms physical retirement. Reuse this matching sample; do not rerun step1 unchanged. Delete's retained304.405ms is sufficient for the revised Commit milestone and receives no timing-only tuning.

Next: add only the missing shared consumer authentication/sort attribution, then choose one follow-up from that measured split. No producer framework, worker-count increase, tier changes, #49 prerequisite, or independent proof campaign is authorized by this focused plan. After the evidenced follow-up, reassess explicitly rather than continuing a marginal campaign.

The missing attribution is now measured in attempt11: authentication130.870ms, sorting4.620ms, Commit644.928ms. Follow-upA is selected; other follow-ups and workers are not automatically added. The exact result, rather than favorable retirement subtraction, will decide the milestone.

## Follow-up A implementation

The existing private authenticated object now owns a transparent wrapper around canonical bytes, exposes immutable access only, and requires both identity and complete outer framing. The shared core codec provides one identity-assignment/framing path; expected-identity authentication retains its previous error ordering. ObjectBuffer construction replaces its old identity computation with this checked construction, rather than adding another hash there.

Memory candidate rows, output slabs and the shared consumer preserve the checked owner. The SQL consumer borrows immutable views without cloning payloads and no longer hashes owned memory again. Fresh selected spill reads establish new checked ownership and still authenticate; their time is reported separately as `object_admission_storage_authentication_ns`, including producer handoff. Durable conflict reads and byte-for-byte comparisons remain unchanged. Borrowed legacy delivery still uses its ordinary checked admission.

Native final publication distinguishes checked output and ordinary planned batches by type; initialization's existing preconditions and failure cleanup stay with initialization. No trust flag, benchmark bypass, provisional admission, additional producer, or scheduler rewrite was introduced. Queue/batch/header bounds are unchanged, and candidate memory charging includes the retained identity without increasing memory limits. Selection/finality remains independent of authentication.

Validation:50 Store tests passed (one existing large-spill test ignored), plus a new selected-spill corruption-before-handoff check.13 core framing tests,56 Workspace tests and14 file-edit/reconciliation integration tests pass. Malformed owners cannot be constructed, memory handoff retains the payload pointer, exact conflict comparisons remain tested, and native initialization/publication failure behavior remains covered. The structural ownership fixture now uses actual canonical framing; malformed framing has explicit rejection cases. One original create-100 sample follows, then explicit reassessment; independent proofs remain deferred.

The same ownership transfer also covers ObjectBuffer's existing authenticated-read callback: an in-memory checked owner is borrowed directly, avoiding a clone and repeat hash when the builder reads its own just-created output. Spill/base reads retain authentication. The pointer-preservation test covers both this callback and delivery. This is part of follow-upA's immutable-memory contract, not another cache policy or producer framework.

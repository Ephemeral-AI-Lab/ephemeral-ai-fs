# Checkpoint task continuation state

Active goal: complete #74/#75, publish/merge and close only after full validation.
Worktree /Users/yifanxu/Ephemeral-AI-Lab/layerfs-checkpoint, branch codex/benchmark-checkpoint.
Original dirty checkout untouched. Base main 5c9cce92b446.
Commits: f25321fa0 contract; b17a72963 history/reliability/fault fixes + generalized collector;
976cd3e15 concise quickstart and draft report generator.
No subagents (not authorized by goal). No PR yet. Issues remain open.

Initial host infra-list (old matched #73 binary) in initial-registry.jsonl:
232 definitions total; host runner admits 198 performance + 29 proof definitions,
including 1 optional sustained-600s. 5 capped legacy definitions not host-admitted.
Final inventory needs regeneration from checkpoint binary.

Builds completed for source seal a847ad83ab8cea09... (see host identity file).
Image layerfs-bench-infra:a847ad83ab8cea09. Host target/release/fs-benchmark-pro.
Build host-r3/image-r3 logs. Later commit only changed markdown + docs reporting,
so product/harness build source contents unchanged. Don't rebuild merely for docs.

LIVE queued process: functions exec_command session 73732, Python PID 43214.
It waits using flock for current measurement, then runs in order:
presentation-failure, lease-lifecycle, open-writer-busy, live-execution-busy,
admission-batch-failure-retry, final-publication-failure-retry, short-spool-write,
deferred-nospace; stops at first non-PASS. Receipts development-r2/<kind>/verification.json.
RE-POLL THIS SESSION before restarting anything. Last confirmed waiting behind
PID42815, a separate DeepSeek full-history benchmark (do not interrupt).
Earlier development-r1 and before-presentation attempts were refused by lock;
these INCOMPLETEs are not product failures. Shared lock at $TMPDIR/layerfs-infra-measurement.lock.
Avoid overlapping CPU-heavy tests/builds/performance with the other run.

Confirmed baseline repro: before-presentation-r2/verification.json TIMEOUT,
Created/presentation_failed + fault hit + canonical content succeeded, then recovery
failed. Root cause: recovery unconditionally pauses an already-ended remote owner.
Changed recovery to pause only if projection_handle Some; attach makes new owner.
This still needs real Docker proof. Stage markers added in reliability runner.

History: fast_verify_branch returns file roots; bounded steps in family module;
all Commit parent links checked before skipping unsampled snapshots; sampled A/B
root comparison and metadata all-file-root preservation added. Raw marker
checkpoint-fast-v1 with verified_steps and omissions.

Reliability: competing mount must reject, original owner still works (not exact old
WorkspaceBusy). Open writer and running command now require Created, live data,
UpToDate afterward. Release running holder even on Commit failure.

Faults: real live_backing APPEND now consumes NoSpace/ShortAppend scoped test hooks.
NoSpace can surface on write or fsync; exact errno still required. Streaming checked
admission now has checkpoint/early-transaction counters; verification_candidate is
exported test-only and activated before construction in Workspace commit.
Final publication no longer demands irrelevant candidate spill. Later admission
still requires actual earlier committed transaction. Needs live fault proofs.

Passed focused tests (session26048 terminal): history_samples_cover_cycles_and_final_states;
append_consumes_only_its_exact_reservation (now also both real backing fault hooks);
published_install_failure_recovers_without_second_commit. These are not Docker proof.
Report test_report.py passes negative missing/duplicate/mismatched-input checks and
keeps target MISS separate from execution PASS. Draft report.py emits JSON/CSV/MD.

Outstanding implementation/report work:
- Run/fix eight pending live reliability proofs, then history100/500 focused proofs.
- Remaining routine reliability suite must pass (including any newly revealed gaps).
- Collector issue54_collect.py --checkpoint extends all HOST_FAMILIES, lists once,
  SDK repetition vs seed, native fresh vs clone, all proofs, nonzero failure return.
  Still needs SDK --performance-rows binding from sample record row_id.
  Its resume early-return paths currently omit parsed identities; fix before final
  campaign. Assert source/input/image matches on receipt reuse; don't silently reuse
  unknown prior source. Handle rejected/incomplete performance -> no fake proofPASS.
- Final reporting still needs comparable older rows and richer readable phase/resource
  columns, actual resource/custody checks, frozen inventory completeness, raw evidence
  packaging/links. report.py currently handles source/product/input/image proof match,
  1sample, statuses, targetmisses, phases/counters/host/resources in JSON. Raw resources
  need actual audited availability. Source manifests not yet frozen for final campaign.
- All-family review contract exists; prove implemented fast routes and remove any
  additional genuinely redundant work found. Preserve full Git verifier.
- Keep sustained600 optional; existing parallel-read-write/repeated-publication tests
  can serve bounded routine sustained smoke, no new duplicate case necessary.
- Required CI, live Docker tests, single shared final 198performance+routineproof
  campaign, MD/JSON/CSV, PR green exacthead, merge, issues74/75 closure all outstanding.

Do not declare goal complete. Previous goal turn made code/contract/report progress;
current external measurement wait is live and verified, not a blocked goal.

# Continuation update (September 8)

Previous turn made concrete progress: 5 real Docker proofs PASS; final-publication
FAIL diagnosed as verifier issuing Exec during retained stage, now moved after
supported Commit retry. Not an unproven product regression. Workload and prior
publication checks retained.

Old session73732 is TERMINAL exit1. Five development-r2 proofs passed (presentation,
lease, open-writer, live-command, later-admission), final-publication failed. Spool
faults were NOT STARTED. Do not re-poll or restart73732.

NEW LIVE SESSION35021: queued host build, last confirmed waiting under flock behind
PID47426 (other DeepSeek full-history verification; do not interrupt). Wrapper PID47968.
When it completes, inspect build-host-r4.log and identity; then BUILD NEW IMAGE
because the presentation family fixture changed on both host and workload.
Current old image a847ad83... remains available for reference but cannot run new
smoke-v3 ID. Avoid changes to source while new build is active; compare source seal
from runner.source_build_args to host identity if uncertain about queued build timing.

New active proof ID: workspace-published-presentation-failure-smoke-v3-proof.
It replaces only old compact-v2 presentation ID; family still28. fixture_for(case)
provides one4096-byte witness, work/a directory, then4096-byte published.dat.
Contract committed BEFORE implementation (9ebe18e29). Registry and expected-state
route updated, selfcheck expects new suffix only for this kind.

Collector implementation updated with:
- load_performance parses exactly1 sample and summary; checks listed source/image,
  family/case/harness, preserves complete identity + SDK performance_rows.
- native verification maps fresh-output identity to CLI --setup fresh.
- verify_row no longer precreates output directory (real verifier rejects existing).
- proof reuse checks source/product/input/image/harness.
- only PASS performance supplies proof identity; checkpoint --proofs all.
- _run waits shared flock before invoking runner (queue outside measured clocks).
- test_checkpoint_collector.py passes, verifies SDK binding and immutable output.

Report generator now includes existing published compatible observations from #38,
#54,#62,#65 and corrected Git #73 medians, phases, SDK route metrics, host/container
peaks and CSV columns. It rejects missing environment/OOM/swap/master evidence and
mismatched proof input/image; synthetic test passes. Need inspect real rows before
final reporting, improve exact resource metrics/links as needed. No final samples yet.

NEXT: finish host35021, build image, run ONLY final-publication retry, short-spool,
deferred-nospace, newpresentation smoke (unique development-r3 output). Then five
high-history proofs and remaining routine proof definitions. Need source-bound
performance selections for other families before their final proofs. Full final
198performance+226routineproofs+1longexclusion campaign still unstarted.

Commit latest contains collector/recovery/report changes; git log identifies it.
All goal requirements (CI/fullcampaign/PR/merge/#74/#75 closure) remain outstanding.

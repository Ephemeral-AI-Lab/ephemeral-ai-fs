# R26: reuse the completed control, collect only the repaired candidate

No Store, snapshot or inventory was opened, copied or rescanned during preparation.
Existing R25control PASS summaries,158-receipt validation, snapshot/census/account
and their small metadata seals were checked. The original files are unchanged.
control-reuse-applicability.json records their exact paths/hashes and explicitly
inherits the original completed census custody rather than asserting a new scan.

The pipeline owner confirms the repair changes only Workspace producer scheduling:
any-predecessor plan selects one worker; new-file plans and Init retain parallelism.
No wire, native dependency chronology, decoder or admission API changes are intended.
Root owns application/tests/freeze and the final decision.

The newly prepared `r26-preparation/run_frozen.py` is a minimal copy of the R25
wrapper with explicit original-control schedule/applicability seal validation.
It skips only control fresh-output checks and control collection, retaining
candidate collection and the full two-arm resource report. Use this R26copy after
sealing; the warning below applies to the unchanged R18/R25wrappers only.

Actual observer-block preflight found a remaining snapshot-branch schedule check
in the first draft. `resource-report-snapshot-reuse-fix.patch` is the one-line
follow-up; root must apply/freeze it. `reuse-guard-preflight-draft.json` preserves
the current-source rejection and the draft-patched valid/rejection test outcomes.

## Why a new schedule is required

R25's failed all-FULL candidate remains a completed, unfavorable storage observation.
Its original frozen schedule and every PASS/failure receipt must remain immutable.
R26 uses a new schedule/host/image/census declaration and a fresh candidate Store.
The control entry in schedule.template.json is EXACTLY the old R25control entry,
including original output, producer identity and argv. It is descriptive evidence,
not authorization to execute those control commands again.

The additive reused_arm_schedules.control field references the original R25schedule
by absolute path and SHA. control_reuse_applicability references the completed
control proof ledger. The new candidate retains the same fixture manifest and
full157 contract. No statistical paired-speedup claim follows from reusing an older
control observation. Original acknowledged allocation remains the storage baseline.

## Narrow reporter adaptation (draft only)

resource-report-control-reuse.patch is NOT applied to the repository. It changes
only the project-owned resource_report.py and retains its existing interfaces.
It selects an original observer schedule only for an explicitly reused control:

- Require allowed reuse key control, absolute original path, exact SHA and schema.
- Require the entire original control arm row equals the new schedule's control row.
- Require matching workload manifest and contract content hashes.
- Add original schedule to the report's authenticated input set.
- Validate old census custody against that ORIGINAL schedule hash and census binary.
- Record each arm's observation_schedule path/SHA/reused flag in output.

All other source/custody/run/manifest/decoder-command checks remain unchanged. The
candidate continues to validate against the new R26schedule and census. Missing or
tampered original schedule, changed control producer/output/argv, wrong workload,
wrong contract or attempted candidate reuse must fail closed. Root should review
and apply this narrow patch before freezing the final helper/source identity.

Do not rewrite old snapshot/census custody to replace its schedule hash. Do not
copy the old observer records while relabeling them as new observations. No new
observer manifest is needed: preserve the old chain and explicitly link its owner.

## Candidate-only sequence

candidate-commands.template.json gives exact command argv/cwd/lock ownership for
the existing coordinator. It is not executable while final identities are missing.
After root freezes R26 and seals both repaired producer and final-source roles:

1. Fill schedule.template.json into new schedule.frozen.json; preserve oldcontrol
   row exactly. Fill candidate host/image/source/census/check/buildqualification,
   helper hashes (including patched reporter), current free space and final UTC.
2. Finalize candidate-commands.template.json from that frozen candidate entry.
   Do NOT invoke either old R18/R25run_frozen.py main function: those wrappers
   require fresh control output and unconditionally execute both arms.
3. Execute candidate performance -> exactly one preverification snapshot -> one
   census -> validation preparation ->158-receipt validator ->account ->all157
   independent historical verifications and normal cleanup. Preserve any failures.
4. Run patched resource_report with original R25control output AND original
   snapshot/census custody, repaired R26candidate output/custody and newR26schedule.
   The explicit per-arm schedule association retains full observer timing fidelity.

Expected accounting input generation reuses account_expected/save definitions
from the existing R25wrapper through runpy.run_path; main is not invoked. It only
hashes already-produced snapshot/inventory/performance/proof/manifest inputs and
writes a new expected JSON. Root's existing coordinator holds the shared lock for
that command and other helpers without internal locks.

## Preserved controls and gates

R25control snapshot observer time1,097,683,125ns and census20,280,793,625ns are
reused measurements under original custody, never remeasured or double-counted.
R25control allocation218,116,096bytes stays the comparison operand. Repaired
candidate must achieve at least10% lower original acknowledged allocation at the
same157retained states;4KiBpages, authenticated dependencies, bounded resources and
all applicable final correctness/performance obligations remain required.

This is not permission to run a new control, mutate old evidence, skip candidate
historical verification, waive the failed R25storage result, or mark the full
campaign accepted merely because the new storage result improves.

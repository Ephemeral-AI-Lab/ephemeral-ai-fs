# M4 implementation handoff prompt

Copy the following into the implementation task. This document starts no task or
verification run. The prior M3-only and full-M0–M5 handoff versions are retained in
Git history; they do not override this milestone's stop condition.

---

Implement only LayerFS milestone 4 using the finalized
[implementation-milestone-4-plan.md](implementation-milestone-4-plan.md).
Read that plan in full, the architecture/format/boundary, current progress and M3
evidence, applicable AGENTS.md/skills, and all relevant caller paths in its reading
list before changing code.

Worktree discovery: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3`, branch
`codex/storage-v3-implementation`, draft PR #81. M3 product commit is
`1ac1ce4b56064a548929b070568d2373daa9e30d`; latest reviewed documentation checkpoint
is `4626583c72c0e4cbe0dd818c58fbe784c453dbbc`. Inspect actual state and active writers,
preserve unrelated edits, and never reset to discovery anchors. Existing schema-6/
wire-1 new-Store-only compatibility is approved; no converter or silent breakage.

You have authority for routine implementation/refactoring decisions within M4.
Do not ask for repeated approvals or stop at a plan/build/partial patch. Complete
implementation, the approved smokes, root-cause fixes and evidence. Use subagents
for bounded independent work when helpful, with explicit editing ownership and no
duplicate smoke runs. A genuine external blocker must be reported precisely.

Implement in this order:
1. Physical locator ordering before bounded read draining, plus existing-owner
   counters for group fetches/decode bytes and later delta opportunity/selection.
2. Predecessor and original first-span handoff through same-inode, complete-build,
   range and captured/tempfile paths. Preserve logical inode and COW semantics.
3. Bounded matching against already-admitted authenticated FULL bases, depth one,
   existing candidate/metadata/CPU/memory limits, no global search.
4. Two fixed-membership group alternatives: all FULL and one selected mixed
   FULL/DELTA assignment. Independently apply Zstandard/RAW; keep mixed only if
   encoded savings reach max(64 bytes, ceil(FULL alternative encoded bytes / 8)).
   Keep level 1 and store only the winner. Follow the plan's buffer-lifetime and
   error rules; do not retain two entire alternative packs or enlarge limits.
5. All three approved smokes, necessary fixes, evidence, commit/push and stop.

Only the DeepSeek first-five, frequent-edit SDK/ordinary and small-file smokes are
executable verification. Builds needed for them are preparation. Do not run extra
unit/fuzz/property/race/crash/workspace/full-family suites, collect new candidates,
change fixtures/interfaces/timing/accounting, or run M5 final three-pair qualification.
Keep required encoding/finalization inside acknowledgement. Never use later packing,
VACUUM, reduced accounting, benchmark-specific code, dependency patches or a skipped
integrity check to manufacture improvement. Preserve failed observations.

Your performance policy prioritizes meaningful storage savings. Roughly 30% longer
foreground operations can be acceptable when the gain justifies it; approximately
sub-50-ms TOTAL public operations can make percentage changes immaterial. This is
not an added allowance per read, aggregate-load qualification, or a new storage gate.
Historical diagnostic gates/misses remain unchanged. Compare baseline and M3 with
actual control limitations, not as fresh matched controls when they are historical.

Finish the plan's checklist. Fix correctness or demonstrated quadratic/repeated/
unbatched work, rather than tuning insignificant milliseconds. If valid results
show weak savings, report them and give a retain/revise/remove recommendation;
do not expand matching or alter workloads to force a target. Missing empirical
DELTA coverage is a limitation, not a pass. Any later emission removal must keep
retained DELTA objects readable.

No M5, stronger-codec trial, new cache/pool, construction-policy rewrite, canonical
wrapper deletion, SQLite tuning, S3, migration or durability work. Update progress
and create implementation-milestone-4.json only from actual evidence. Commit/push
the existing implementation branch/PR without merging. Final report: exact HEAD,
worktree/PR state, changed owners, checklist, commands/artifacts, allocation and
elapsed/resource comparisons, representation coverage and remaining limitations.
Stop at the reviewable M4 checkpoint. Complete v0.1.4 qualification remains later.

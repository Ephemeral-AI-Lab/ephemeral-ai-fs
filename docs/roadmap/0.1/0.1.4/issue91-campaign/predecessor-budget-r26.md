# R26 preserve predecessor correspondence within producer budgets

R25 completed both 157-state performance and historical-verification arms, both
158-receipt validators, and storage accounting. Content and history passed, but
candidate storage failed: 318,803,968 acknowledged allocated bytes versus
218,116,096 for the control (+46.16%). Both used 4 KiB pages.

All 86,412 file-eligible objects, representing 705,162,813 canonical bytes, selected
native FULL; PREFIX attempts and admissions were zero. All 134,997 attached
predecessor cursors were denied by the memory guard. The control had no such
denials and received 269,994 grants. Preserve the failed candidate and every
original observation. This storage regression is not an accepted speed tradeoff.

## Cause and correction

Parallel Workspace output partitions divide 1 MiB among producers. Predecessor
correspondence requires 576 KiB plus 32 KiB of output headroom. Two or more
producers therefore deny every correspondence cursor. This incompatibility
predates R15; it does not originate in Init coalescing or the 4 KiB page policy.
Predecessor roots survive planning but cannot receive grants.

The correction records whether the existing task-planning lookup found any
regular-file predecessor, then caps that plan's already bounded worker count
with min(1). Every memory guard, reservation and zero-worker check remains. The
single 1 MiB producer reserves 576 KiB and retains 448 KiB for output ownership.
Mixed plans serialize all file tasks. Pure-new, no-predecessor Workspace plans
and native Init retain their existing parallelism.

Native wire format, dependency ordering, SQLite pages and imported library
sources, versions and features remain unchanged. No cache, worker pool or memory
allowance is added.

The focused regression requests four workers for two rewritten predecessor
files. It fails before the fix because correspondence receives no grants. After
the fix it passes with grants, no memory denials, native PREFIX records and exact
current and retained file bytes. Commands, logs and patches are retained in
r26-before-01 and r26-after-01 under layerfs-issue91-runs. Affected native checks,
formatting and Clippy must pass before corrected candidate collection.

## Qualification and control reuse

The user selected the best available candidate and requested the full benchmark
plus the DeepSeek 157 harness. Preserve original performance severity labels and
report accepted performance tradeoffs separately. Correctness, 4 KiB pages and
at least 10% lower equal-state acknowledged storage remain requirements.

Freeze and build this correction, run a fresh candidate full157, then collect
198 performance cases, 226 routine proofs and the required supplemental checks.
Do not add further optional performance optimization in this pass.

Reuse the unchanged passing R25 control performance, snapshot, census, accounting
and all 157 historical proofs. The new frozen R26 schedule references the
original schedule and hash and preserves its entire control-arm entry. The
resource reporter validates both old snapshot and census custody against that
original schedule and decoder. It does not rewrite custody or remeasure control.
Candidate observers remain bound to the new frozen schedule.

The reporter adaptation changes evidence binding only; accounting equations and
storage thresholds remain unchanged. Its original draft missed the snapshot
schedule check. A focused metadata check rejected that draft, accepted the
corrected binding, and rejected altered schedule hashes, control identities,
workloads, contracts and attempted candidate reuse. Retain that check as well.

Final source, host, image, decoder and helper seals must precede candidate
collection. Preserve all build failures, the failed R25 storage result and the
original reports.

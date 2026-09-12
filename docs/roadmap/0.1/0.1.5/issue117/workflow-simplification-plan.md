# Main-only workflow and test-infrastructure simplification

Study date: 2026-09-12. Scope is LayerFS; other repositories, Docker volumes,
Codex session data and prepared fixture inputs are outside this cleanup.
Owner subsequently authorized deleting LayerFS worktrees and their benchmark
evidence, including previously protected evidence. Exact executed cleanup is
recorded separately in the retirement record; this document is the infra plan,
not a claim that the infra changes below have already been implemented.

## Findings

At study start LayerFS had main plus seven detached registered worktrees. All
seven HEADs were ancestors of main: no committed changes required merging.
Each worktree still contained dirty source/harness changes, so source recovery
was required before removal. Main itself had 23 modified/deleted tracked files
and 14 untracked files, including compaction-removal, issue112/113 notes and web
work; consolidating those into clean commits is separate from merging old arms.
Main was83 commits ahead of its local origin/main tracking ref; no fetch/push
was performed, so this is not a statement about the live remote tip.

Measured directory sizes before cleanup: main~78.29GiB, main target~65.58GiB,
host-store/builds~6.37GiB, seven linked worktrees~10.05GiB combined. Free space
was~106GiB. Removing worktrees alone could not reach issue117's150GB objective.

The current runner creates a new qualified target directory per compilation
seal; the seal includes source content and absolute source paths. Dependency
seeding copies another build tree. This is safe against some stale builds but
multiplies disk/codegen costs across experimental arms. Other costs came from
copied per-stage collectors, repeated setup and qualification during iteration,
heavy tests selected too broadly, and paperwork rather than useful testing.

The existing benchmark rules already prescribe a fast selected loop in section15.
We should make that the normal workflow and reserve terminal rigor for the final
claim, instead of making every experimental iteration a full campaign.

## Execution order

1. Finish authorized cleanup and record dropped evidence explicitly. Preserve
   source recovery, current main changes and prepared fixtures. No blind merging
   of historical control/diagnostic code into main.
2. Review main's dirty work by coherent feature, integrate intended product/docs
   changes in focused commits with the smallest relevant checks. Archive obsolete
   experiments rather than promote them. A clean main commit becomes the next
   measurement baseline. Never run git add -A over unrelated work.
3. Update benchmark_rules.md and QUICKSTART together with the minimal runner
   changes below. Owner's practical timing policy is prospective; historical
   failures and waivers are not rewritten.
4. Run focused runner/build-reuse/identity/cleanup regression tests, one selected
   product smoke and one small paired qualification example. No whole18-family
   rerun for an infra-only change without a demonstrated affected path.
5. Report measured complete command time, reused work, retained disk and exact
   preservation/identity checks; update117. No release/tag/deployment implied.

## Two workflows, one existing runner

Development (default):
- One relevant unit test or bounded group; long sweeps explicitly selected.
- One complete selected public case when product execution is needed.
- Reuse a compatible build and immutable prepared fixture. Do not rebuild Linux
  binaries for docs or host-only Python changes when the existing build mapping
  can prove the executable compatibility.
- Record command, exit, actual executable/image/input identity and raw result.
  Mark exploratory timing as such. No per-iteration contract commit, separate
  GitHub issue, full family matrix, or mandatory phase instrumentation.
- Correctness failures fail immediately. Minor isolated latency misses are
  warnings; no repeated unchanged runs to make a median pass.

Qualification (explicit, once the relevant code is stable):
- One concise configuration declares operation, case/input, comparator, cache
  state, repetitions and meaningful thresholds. Reuse one analyzer/schema.
- Matched alternating pairs for performance claims, with all samples retained.
- Independent verification of the affected public paths; expensive scaling,
  endurance and boundary tests selected where the change warrants them.
- Distinguish absolute target, material regression, correctness and evidence
  validity. Preserve documented owner waivers rather than forcing all into PASS.
- No new bespoke Python collector per stage unless the shared runner genuinely
  cannot express the workload; add the smallest reusable operation support once.

These are workflow names, not invented current CLI flags. Prefer existing
--perf-fast, family scripts and verify-selected entrypoints; only add a flag if
needed to express a real missing distinction.

## Build and storage policy

- Keep only main as a persistent Git worktree. Build candidate from main.
- Capture a control executable/identity by independent copy before product edits.
  Bind it to the same harness/input contract as candidate; do not mutate it.
- If a historical source build is essential, use one temporary independently
  owned source directory, then remove it after extracting the required binaries
  and identity. Never hard-link writable source/target files or switch a dirty
  main checkout underneath active work.
- Prefer Cargo's ordinary incremental dependency cache in one owned target per
  toolchain/profile, and separate immutable executable snapshots from that cache.
  Validate that source/build inputs and executed binaries match before retiring
  the current per-source-target strategy. Do not trade speed for stale artifacts.
- Keep bounded recent executable/build retention; no full dependency tree per
  sample or stage. Preserve existing main build cache now if free-space needs
  are met; cargo clean should not be the default between iterations.
- Temporary sample Stores and containers should be cleaned automatically after
  their required proof and receipt capture. Retain failed Stores only when needed
  for diagnosis under a bounded explicit policy. Never classify a unique input
  fixture or irreplaceable Store as disposable output.
- Before expensive work, check free space against declared output needs plus
  headroom. Cleanup must select owned scratch paths, never broad globs over
  fixture/evidence/source parents.
- One manifest written once after closure; do not keep recommitting a report's
  self-hash. Keep compact summaries/raw receipts and the few binaries needed for
  active comparisons. The owner has explicitly dropped the old evidence; new
  retention policy must be declared before collection.

## Rules to keep

Correct final contents, authentication, bounded memory/disk/CPU and failure
safety are not optional. Keep actual executable identity checks (the hard-link
incident demonstrated why), the public operation/timer boundary, the host SDK/
SQLite versus Docker daemon/FUSE topology, and serialization of resource-sensitive
measurements. Preserve the fixed verified-cold Init acceptance policy; warm timing
must not silently satisfy it. Keep complete original workload when claiming that
case, all valid outliers, and separate diagnostics from plain performance.

Relax administrative and exploratory-loop restrictions, not these protections.
No promise of a seconds-long cold build or a full million-record sweep: measure
those as explicit setup/extended work. Target quick incremental selected runs and
report actual wall time instead of advertising only the subsecond product timer.

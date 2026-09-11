# Bounded metadata INSERT experiment

Freeze before implementation: use the current d316cb592 source plus preserved
dirty compaction-removal work as the control, in isolated worktrees. Exclude
subsequent concurrent edits. Candidate changes only ValueIndex scratch INSERT
execution to bounded multi-row SQL, plus focused tests. Keep 4 KiB, all format
and equality rules, group/ordinal order, retention, transactions and rollback.

Evidence root is recorded before edits in `/tmp/layerfs-metadata-insert-root`;
retain its original patch/status, both source patches, qualified build identities,
commands/logs/exits and every sample. Original evidence roots are read-only.

Stage 1 is mechanism screening, not acceptance: identical nonce phase/physical
instrumentation in both arms; n=2 per arm, order C1,T1,T2,C2. Full
namespace-100000, seed1, fixture digest6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e.
Fresh independent host Stores and fixed verified cold acquisition before every
arm, no warm-up. Zero-residency evidence is required; high read counts alone
cannot validate cold. Build with shared runner under its lock; diagnostics use
that same lock once, no overlapping resource-sensitive work.

Record INSERT values/executions/wall and existing final-tree/pipeline/whole Init
clocks. Reconcile canonical112451 objects/513026835 bytes, full100000 files and
500000000 logical bytes, phases, resource costs and source/fixture identities.
Keep both paired differences and median/range. Proceed to qualification only
if both paired whole-Init and INSERT reductions exceed their respective control
ranges, with correctness/format checks passing. Otherwise reject this candidate;
do not stack a lookup/cache change onto it. One whole-pair replacement is allowed
only for demonstrated infrastructure invalidity; retain the original attempts.

Stage 2, only if screening passes: new plain identities, n=2 cold paired runs
in the same C,T,T,C order under independent fixed acquisition, plus all four
registered Init tiers, matching-image selected independent verifier and affected
Commit coverage (unchanged, reopened first-small, retained repeated-small, larger
change, redundant history). Plain cold remains the absolute2.7 s gate; diagnostic
and warm observations cannot pass it. No claim of improvement before plain proof.

Focused tests cover duplicate earliest-ordinal retention, multiple/tail chunks,
low variable limits, group-boundary eviction, reopen and rollback, using existing
metadata/admission suites where applicable. No release/tag/deployment, page-size
experiment, speculative cache, DELTA-policy change or compaction.

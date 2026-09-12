# Hybrid file-representation experiments

This is an evidence-only report. Start with [the findings](report/findings.md)
and [the timing table](report/results.md). Complete performance and verification
receipts and execution identities are included under [evidence](evidence/).
The archived harness is not registered in this PR's benchmark executable.

Prospective exploratory scope, 2026-09-12. User requested a separate worktree and
the benchmark Docker/FUSE environment. Base: f8bb6dcb676f73a14746357d98489b0691398c26.
No product source changes, cutoff treatment, release claim or new official family.

Reuse the benchmark host build/image pipeline, authentication, resource guards,
global measurement lock, public SDK calls and real FUSE sessions. macOS owns
SQLite/SDK/Commit/spool; Linux runs FUSE/daemon and read-only proof commands.
No host filesystem projection, shared data mount, Docker SQLite or compaction.

Deterministic splitmix fixture seed 71503; appended bytes use seed 71504.
One regular file per fresh input/Store/container. Performance does not pre-read
its payload. OS cache is uncontrolled; these are not cold-source claims.
Post-Commit representation inspection is outside call timers but can warm later
steps in the same history. Host lifetime RSS includes fixture/oracle allocation.

Cases: one-byte SDK replacements at 64/127/128/129 KiB and 16 MiB; ten one-byte
edits/Commits at 64 KiB; six alternating 127/129 KiB saved versions; four edits
127->130->126->140->120 KiB before one Commit; shrink 1 MiB and 16 MiB to 80 KiB.
All mutations use Client::edit_workspace_file_range. Public edit and Commit
timers exclude preparation, oracle work, content inspection and verification.
The full per-step lifecycle separately includes session creation, a FUSE
filesystem-type check, edits, Commit and end. Both modes keep original failures.

Three performance repetitions, alternating forward/reverse case order, followed
by one independent verification replay of each case. This establishes scoped
observations, not optimality or a product speedup. No latency pass threshold.
Correctness: Created outcome, exact final size and saved representation, no
presentation failure, host swap or container OOM/swap. Proof mode compares every
byte before/after Commit through FUSE and every retained version after Store
reopen, using both authenticated reads and newly forked FUSE workspaces.
Performance/proof data remain separate; proof timers do not enter medians.

Read/encoding counters cover the complete Commit, including metadata. CDC bytes
count chunker input, not total hashing or physical device reads. Whole small-file
assembly is not directly timed or counted by CDC. SQLite allocated bytes include
page overhead; repeated hashes/roots establish exact content-object reuse.
All raw receipts remain; successful sample Stores/inputs are removed after checks.
Failure Stores remain for diagnosis. The worktree and experiment sources remain.

## Recompute the published results

The analyzer uses only the included receipts and checks the representation,
CDC-input and exact-reuse observations. Use a new output directory:

```sh
python3 experiments/transitions/analyze.py \
  experiments/transitions/evidence/performance \
  experiments/transitions/evidence/verification \
  /tmp/layerfs-transition-report-recomputed
```

## Reproduce the product experiment

Measurements apply to development revision
`f8bb6dcb676f73a14746357d98489b0691398c26`, not this report-only PR's base.
At report preparation, that revision was available in the author's local
repository and was 98 commits ahead of GitHub main. This PR does not import
those unrelated changes or claim that its base was measured. Full reproduction
requires that development revision and the benchmark Docker environment.

On a separate checkout of the measured revision, copy the two files from
`harness/` into `benchmark/fs-bench-pro/src/transition_experiment.rs` and
`benchmark/fs-bench-pro/shared/transition_experiment.py`, respectively, then
apply `harness/main.patch` at the repository root. The archived files are
byte-identical to the measured experiment sources; see
[measured source hashes](evidence/measured-source-and-report-manifest.json).
Rebuilding records a new compilation identity; do not overwrite the evidence
published here or relabel new measurements as the original observations.

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
python3 benchmark/fs-bench-pro/shared/transition_experiment.py --image IMAGE --mode performance --repetitions 3 --output NEW_OUTPUT
python3 benchmark/fs-bench-pro/shared/transition_experiment.py --image IMAGE --mode verify --output NEW_PROOF_OUTPUT
```

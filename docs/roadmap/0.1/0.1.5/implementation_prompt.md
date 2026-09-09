# v0.1.5 implementation prompt

Implement the complete LayerFS v0.1.5 design. Continue through integration and the
smoke verification below; do not stop after planning, a codec prototype, or a
partial storage path.

Read these documents under `docs/roadmap/0.1/0.1.5/`:
1. `README.md` and `workflow.md`
2. `spec.md` and `implementation_plan.md`
3. `delta-encoding-benchmarks.md`, especially its first-round execution scope
4. `past_mistake.md`, `benchmark_success.md`, and `benchmark-v0.1.4-report.md`

Use released v0.1.4 (`101fa273d815f3aaedb0e06ba0de7b0777d83def`) as the product
baseline. Preserve other tasks' edits and use an isolated `codex/` worktree when
needed. Carry the reviewed roadmap docs into that worktree without overwriting
unrelated changes. Prefer existing code and dependencies; keep the implementation
small and direct.

Implement the full architecture:
- New/changed nonempty regular files below 131072 bytes use one whole-file
  SmallContent CAS object, physically FULL or a one-level DELTA against a FULL
  SmallContent base. Empty files retain the specified compact representation.
- At/above 131072 bytes, keep existing CDC chunks and extent trees, including
  locality for known large-file edits. Unchanged historical roots remain readable.
- Add the shared regular-file content dispatch and route all regular-file readers,
  edits, materialization, reconciliation, backing and checkpoint consumers through
  it. Do not leave assumptions that every file root is an extent FileState.
- Namespace Init and workspace Commit use the same construction, CAS selection,
  encoding, admission and publication machinery, with distinct source acquisition
  and lifecycle preconditions. Preserve the authoritative live FUSE/SDK state.
- Implement exact CAS reuse, deterministic single-base selection, pack-v3
  FULL/DELTA grammar, bounded reconstruction/authentication, physical base lifetime,
  all length-changing operations and small/large/empty transitions.
- Implement schema 8 with the existing seven tables, supported old-format reads,
  nonpromoting old-store opens and the explicit offline upgrade from schema 7.
  Preserve rollback, publication outcomes, POSIX behavior and existing durability
  semantics. Do not implement the superseded chunk-member delta design.

Keep settings fixed. Do not run experiments or sweeps over alternatives:
- Small/large boundary: 128 KiB, strictly below is small.
- Existing CDC: 8 KiB minimum / 16 KiB target / 32 KiB maximum.
- New SQLite pages: 4 KiB; retain supported old 64-KiB layouts.
- New small codec: existing Zstandard, level 3, windowLog 18, workers 0,
  checksum/content size enabled and dictionary ID disabled.
- One new-format delta level, one eligible anchor candidate, and the pack,
  buffer and ownership limits in the spec. Keep existing large-file codec settings.

Verification for this task is ONLY the fixed smoke:
`small_file_delta_smoke / small-file-delta-10x30-v1`.

Reuse the existing storage-smoke runner and importer. Implement its exact ten-file
fixture and thirty-step schedule from the benchmark doc: ten source-like files,
eight-byte overwrites, one-line insertions and one-line deletions; one file changes
per commit. Use native Init and ordinary full-file saves through public Exec/FUSE,
followed by public Commit. No SDK substitute or extra operation-surface matrix.

First collect a source-sealed control on unchanged released v0.1.4 with this same
harness/fixture. Then run the candidate with identical inputs and timing. Require
30 Created commits, successful cleanup, and exact verification of all ten files
at all 31 retained states by reopening the SAME measured Store after allocation
observations are frozen. Its fixture self-check and integrated history verifier
are part of this smoke. Confirm that the candidate actually emits the new small
representation and DELTA records rather than silently bypassing the feature.

Compile the required targets, but do not run Cargo test, Clippy, doctest, separate
codec/migration/failure/threshold suites, the old three-file tiny case, the 56-case
edit families, full157, or any broad benchmark campaign in this task. Implement
runtime validation and error handling fully; report unexercised corner cases as
later qualification. The smoke is exploratory, not release admission, and does
not require creating an admission issue. Do not publish or tag a release.

Iterate fast. Build once per relevant change and reuse matching host binaries,
Linux images, incremental Cargo/BuildKit caches and protected fixture preparation.
Use the existing host build and storage-smoke image entrypoints. Avoid redundant
cargo check/build passes, cargo clean, fresh target directories, Docker pruning,
dependency updates, full rebuilds for documentation changes, and forged source
seals. Register the case's fixed 600-second phase and 30-second operation watchdogs.

If builds, setup, smoke or verification are slow, diagnose and fix the actual
cause: lock contention, unnecessary rebuilds, repeated preparation/transfers,
per-state process startup, or product work. Preserve required provenance and
correctness checks. Serialize resource-sensitive commands under the existing
measurement lock; do not double-acquire locks already owned by the runner.

Preserve #95 Init's bounded authenticated comparison reuse, #98 Workspace SQL
coalescing/staging handoff, and the 64-KiB ordered spill read-ahead cap. Avoid
repeated authentication, readbacks, full workspace/history scans, per-object
transport calls, unnecessary SQL transactions, ineffective batches and long
Store/live-state lock holds. Fix a shared cause at its real owner instead of
adding special cases at individual callers.

After a substantive change, rebuild only affected artifacts and rerun the same
smoke when the change invalidates the previous result. Do not repeatedly run
already-passed checks or an unchanged candidate to obtain a better number. Keep
failed evidence and record why a rerun was needed.

Finish with the complete implementation, updated docs, exact code/artifact
identities, smoke verification and cleanup results, and a baseline/candidate
comparison of initial/final allocated storage, thirty-commit growth, save/Commit
latency and build/setup/verification time. Use the new ten-file control, not the
historical three-file numbers or gates. Report improvements, regressions and
unrun qualification honestly; do not claim numerical PASS against thresholds
that were never specified.

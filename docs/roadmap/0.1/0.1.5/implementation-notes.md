# v0.1.5 implementation notes

This work is based on released `101fa273d815f3aaedb0e06ba0de7b0777d83def` in the isolated `codex/v015-small-content` worktree. The separate `codex/v015-smoke-control` worktree changes only the benchmark harness and reviewed roadmap documents. Main and other tasks' source edits are preserved.

## Implemented paths

- `file/content.rs` owns SmallContent parsing, construction and regular-file root inspection/range/stream dispatch. The outer Bytes frame and identity algorithm stay unchanged. Empty output keeps the compact extent state. Generic metadata ropes and old chunk framing remain unchanged.
- `FileContentRoot` travels through live pieces, backing transport, frozen files and checkpoint installation. Filesystem reads, edits, materialization, reconciliation and namespace validation use the common dispatch; actual large extents still use `FileStateRoot` and the existing mutation batch.
- Init and Commit share checked file completion and checked admission. Small finalized targets carry one predecessor hint before slab delivery. Format capability comes from the operation's Store. Old schemas retain old construction; legacy history is not rewritten on open.
- Small physical admission runs after exact CAS selection. It batches predecessor locators, considers only the known predecessor's FULL representation or its direct FULL anchor, prepares FULL once and DELTA at most once, and chooses DELTA only when its complete serialized cost is strictly smaller. There is no history/similarity search, chunk-member delta format or new service.
- Pack v3 has one SmallContent per RAW group and ordinal zero. Parsing checks lengths, framing, reserved bytes, version, selected bounds, codec frame EOF/checksum/content size/window and canonical length/identity. Complete integrity traversal checks contiguous pack coverage and EOF. New deltas require authenticated v3 FULL bases; native v2 PREFIX remains on its old grammar and codec settings.
- Compression uses fixed Zstandard level 3/windowLog 18/workers 0, checksum/content-size flags on and dictionary ID off. A static aligned 2-MiB encoder and bounded operand/output reservations replace heap codec growth. Reconstruction uses static decoder/dictionary storage within 1 MiB plus bounded operands. Decoder and encoder ownership do not overlap during anchor acquisition. Store/Blob mutex guards end before codec work.
- The existing publication session keeps selected base locations stable. Rollback deletes only the session's private objects/packs; retained stages/publication retain their owning session's output. Explicit reachability accounting includes small physical bases. No GC, repacker or header-only downgrade is introduced.
- New Stores have schema 8, the same seven tables and 4096-byte pages. Supported existing 65536-byte layouts remain unchanged. `LayerStackStore::upgrade_format(path)` performs read-only preflight, exclusive revalidation and a DELETE/FULL SQLite transaction for schema 7 to 8. Normal Store opens retain MEMORY/OFF semantics and do not gain power-loss durability.
- #95 Init comparison reuse, #98 Workspace SQL cohorts/staging handoff, strict SQL cohort limits and the 64-KiB ordered spill read-ahead cap remain in place.

## Verification boundary

Only `small_file_delta_smoke / small-file-delta-10x30-v1` is authorized here, including its fixture self-check and same-Store 31-state verifier. Product/harness compilation is allowed. No Cargo test, Clippy, doctest, codec/migration/failure/threshold suite, SDK matrix, old tiny case, 56-case family or full157 campaign was run.

Later qualification must exercise malformed/truncated/multiple-frame inputs, missing/corrupt/non-FULL bases, late collisions, rollback and retained-stage failure boundaries, interrupted upgrades and old-format combinations, empty/131071/131072/131073 transitions, hard links/open-unlinked/rename/concurrency, very large known-edit locality, maximum codec/queue ownership and broad performance. The smoke cannot establish those properties exhaustively.

## Retained attempts

The first control completed 30 Created commits and all 31 retained states with clean teardown. It remains under `layerfs-v015-smoke-evidence/control`. Review then found the inherited verifier used its full phase timeout per state and did not check the saved measured-Store digest before reopening. The harness now uses the fixed 30-second operation watchdog for that step and checks the Store against the frozen performance manifest. `control-2` is the matched control for this harness revision; the first observation is not selected for its numerical result.

Candidate build logs retain the compile failures (a predecessor root type still using the extent type, and a SQLite length decoded as `usize` rather than checked from an SQL integer). Subsequent source changes consolidated small construction and completed integrity/accounting/capture paths before candidate measurement. No failed evidence was overwritten.

The final smoke report records code/artifact seals, initial/final allocation, growth, per-step save/Commit times, setup/build/verification time and cleanup. Numerical values are descriptive comparisons, with no inherited three-file threshold or invented PASS gate. No release is published or tagged.

The first candidate performance pass created 30 commits and its post-measurement census found 10 FULL and 30 DELTA SmallContent records. The integrated verifier failed at genesis: batched inode acquisition still used the extent-only decoder while individual inode acquisition used the new dispatch. The shared batch length parser fixes this path without an extra object read or authentication. The failed attempt remains under `candidate`; the source-matched rerun is `candidate-2`. Schema-8 producer concurrency is bounded at four to fit simultaneous small operands/queues and static codec storage inside the existing ledgers; predecessor-bearing scheduling retains its tighter existing bound.

Reusing one target directory across worktrees exposed Cargo's timestamp-based freshness behavior after rebuilding the control. Candidate build 5 selected a stale control dependency, producing missing-module/trait compile errors. Refreshing timestamps of the actually changed candidate sources invalidated those artifacts without deleting caches or changing source seals; build 6 succeeded.

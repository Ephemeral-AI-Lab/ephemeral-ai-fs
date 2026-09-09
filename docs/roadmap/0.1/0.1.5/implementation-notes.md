# v0.1.5 implementation notes

> **Issue #100 measured outcome, 2026-09-10:** The retained implementation allocates
> **49,319,936 bytes**, with **49,250,304 bytes** growth, ten Created outcomes,
> exact same-Store verification and clean teardown. It is **4,319,936 bytes above
> 45,000,000** and is **not near-target**. Commit median/sum exceed the prospective
> 10% working criterion; save/paired medians and historical-read wall remain close
> to the original baseline. See [the consolidated results](issue100/storage-optimization-results.md)
> and [complete retained-candidate evidence](issue100/retained-candidate-1-results.md).
> The owner subsequently authorized full157 despite the ten-state target miss.
> [Full157 confirmation](issue100/retained-full157-results.md) completed at
> **134,246,400 B**, with157 Created outcomes, same-Store verification and clean
> teardown. It saves27.27% versus released control but has31.13% higher paired
> median latency. The issue remains open; no release-admission PASS or release.

The retained product is byte-identical to measured source `ee78028ba`. Later
revert commits remove the two rejected DELTA-cache experiments; reporting changes
do not replace their saved source/binary/image identities. Accepted additions are
bounded SmallContent chains, removed-name discovery, a compact selected-FULL
fingerprint cache, and transfer of that cache after retained admissions.

The original first-round work was based on released `101fa273d815f3aaedb0e06ba0de7b0777d83def` in the isolated `codex/v015-small-content` worktree. The separate `codex/v015-smoke-control` worktree changes only the benchmark harness and reviewed roadmap documents. Main and other tasks' source edits are preserved.

## Implemented paths

- `file/content.rs` owns SmallContent parsing, construction and regular-file root inspection/range/stream dispatch. The outer Bytes frame and identity algorithm stay unchanged. Empty output keeps the compact extent state. Generic metadata ropes and old chunk framing remain unchanged.
- `FileContentRoot` travels through live pieces, backing transport, frozen files and checkpoint installation. Filesystem reads, edits, materialization, reconciliation and namespace validation use the common dispatch; actual large extents still use `FileStateRoot` and the existing mutation batch.
- Init and Commit share checked file completion and checked admission. Small finalized targets carry one predecessor hint before slab delivery. Format capability comes from the operation's Store. Old schemas retain old construction; legacy history is not rewritten on open.
- Small physical admission runs after exact CAS selection. Schema 9 batches known predecessor locators and reconstructs/authenticates the immediate SmallContent predecessor, carrying its bounded closure facts forward. Schema 8 keeps the original FULL/direct-FULL-anchor selection. FULL is prepared once and DELTA at most once, winning only on strictly smaller complete cost and admitted bounds. The schema-9 selected-FULL cache supplies one fallback candidate from actual
  retained winners, with 128 KiB reserved from the existing index allowance. The
  compact cache stores 1024 records once and 8192 u16 lookup references; it moves
  between retained admissions through one idle StoreDb slot, never cloning arrays.
  Rolled-back sessions discard all hints; reopen starts empty. Removed-name
  discovery uses frozen directory removals/removed subtrees and only a unique
  basename/root, preserving the real before inode and dropping its bounded
  catalogue before producers. No
  global history search, chunk-member delta format or new service is implemented.
- Pack v3 has one SmallContent per RAW group and ordinal zero. Parsing checks lengths, framing, reserved bytes, version, selected bounds, codec frame EOF/checksum/content size/window and canonical length/identity. Complete integrity traversal checks contiguous pack coverage and EOF. Kind 1 still requires authenticated v3 FULL bases; schema-9 kind 2 uses the shared iterative reader with <=8 edges, <=512 KiB summed canonical closure including target and <=256 KiB retained encoded capacity. Every reconstructed node is authenticated. Native v2 PREFIX remains on its old grammar and codec settings.
- Compression uses fixed Zstandard level 3/windowLog 18/workers 0, checksum/content-size flags on and dictionary ID off. A static aligned 2-MiB encoder and bounded operand/output reservations replace heap codec growth. Reconstruction has an actual 2-MiB active allowance: <=1 MiB static decoder/dictionary storage, <=256 KiB encoded records, four simultaneous base/raw/framing/canonical buffers, bounded associations and a surviving admission target. The old read-wave small-base cache is cleared for chains. The 3-MiB encoding allowance includes its actual 2-MiB static encoder plus bounded target/base/FULL/DELTA/output handoff; surrounding existing ledgers remain authoritative. Decoder and encoder ownership do not overlap during anchor acquisition. Store/Blob mutex guards end before codec work.
- The existing publication session keeps selected base locations stable. Rollback deletes only the session's private objects/packs; retained stages/publication retain their owning session's output. Explicit reachability accounting includes small physical bases. No GC, repacker or header-only downgrade is introduced.
- New Stores have schema 9, the same seven tables and 4096-byte pages. Supported existing 65536-byte layouts and nonpromoting schema 6/7/8 writer policies remain unchanged. `LayerStackStore::upgrade_format(path)` performs read-only preflight, exclusive revalidation and a DELETE/FULL SQLite transaction for schema 7/8 to 9. Kind 2 is rejected in schema 8; old binaries reject schema 9 before normal mutable open. Normal Store opens retain MEMORY/OFF semantics and do not gain power-loss durability.
- #95 Init comparison reuse, #98 Workspace SQL cohorts/staging handoff, strict SQL cohort limits and the 64-KiB ordered spill read-ahead cap remain in place.

## Verification boundary

The first-round `small_file_delta_smoke / small-file-delta-10x30-v1` and 31-state
verifier are historical evidence below. The current issue #100 scope uses the
[frozen ten-snapshot contract](issue100/ten-snapshot-contract.md), focused changed-owner
checks, public performance, frozen census and exact same-Store verification and
cleanup. The subsequently owner-authorized full157 confirmation is complete; see the
[current full157 report](issue100/retained-full157-results.md) for storage gains
and public-latency regressions.
No broad Cargo/Clippy/doctest, unrelated family or release qualification campaign
is claimed. Detailed focused-check/measurement receipts belong to the chain report.

Later qualification must exercise malformed/truncated/multiple-frame inputs, missing/corrupt/non-FULL bases, late collisions, rollback and retained-stage failure boundaries, interrupted upgrades and old-format combinations, empty/131071/131072/131073 transitions, hard links/open-unlinked/rename/concurrency, very large known-edit locality, maximum codec/queue ownership and broad performance. The smoke cannot establish those properties exhaustively.

## Historical first-round retained attempts

The first control completed 30 Created commits and all 31 retained states with clean teardown. It remains under `layerfs-v015-smoke-evidence/control`. Review then found the inherited verifier used its full phase timeout per state and did not check the saved measured-Store digest before reopening. The harness now uses the fixed 30-second operation watchdog for that step and checks the Store against the frozen performance manifest. `control-2` is the matched control for this harness revision; the first observation is not selected for its numerical result.

Candidate build logs retain the compile failures (a predecessor root type still using the extent type, and a SQLite length decoded as `usize` rather than checked from an SQL integer). Subsequent source changes consolidated small construction and completed integrity/accounting/capture paths before candidate measurement. No failed evidence was overwritten.

The final smoke report records code/artifact seals, initial/final allocation, growth, per-step save/Commit times, setup/build/verification time and cleanup. Numerical values are descriptive comparisons, with no inherited three-file threshold or invented PASS gate. No release is published or tagged.

The first candidate performance pass created 30 commits and its post-measurement census found 10 FULL and 30 DELTA SmallContent records. The integrated verifier failed at genesis: batched inode acquisition still used the extent-only decoder while individual inode acquisition used the new dispatch. The shared batch length parser fixes this path without an extra object read or authentication. The failed attempt remains under `candidate`; the source-matched rerun is `candidate-2`. Schema-8 producer concurrency is bounded at four to fit simultaneous small operands/queues and static codec storage inside the existing ledgers; predecessor-bearing scheduling retains its tighter existing bound.

Reusing one target directory across worktrees exposed Cargo's timestamp-based freshness behavior after rebuilding the control. Candidate build 5 selected a stale control dependency, producing missing-module/trait compile errors. Refreshing timestamps of the actually changed candidate sources invalidated those artifacts without deleting caches or changing source seals; build 6 succeeded.

## Issue #100 chronology

The shared oversized exact-CAS comparison now dispatches pack-v3 SmallContent to
its existing bounded authenticated reader and retains legacy streaming comparison.
A 96-KiB FULL/DELTA/legacy exact-reuse regression reproduced the old failure and
passed after the fix. The first release-mode unit compilation failed because
existing tests reference debug-only failure hooks; the focused debug-mode check
then ran alone. Original logs are retained in `layerfs-issue100-evidence`.

Both matched full157 histories completed 157 Created outcomes, exhaustive same-Store
historical verification and cleanup. The initial candidate allocated 201,371,648 B
versus released control 184,582,144 B: a regression, not accepted storage optimization.
At the owner's request, subsequent iteration uses the [ten-snapshot baseline](issue100/ten-snapshot-baselines.md)
with matched Git/released/current arms. Its new fixture and verification passed before the chain implementation. That
baseline-stage statement does not describe the later schema-9 candidate.

The current chain candidate's 56,668,160-B allocation improves on both LayerFS
baselines but does not satisfy the owner's objective. The [chain report](issue100/chain-1-results.md)
preserves exact source/binary/image/fixture seals, command receipts, FULL/DELTA/
metadata/index attribution and timing/resource/cleanup evidence. The original
full157 regression is preserved as historical evidence. The subsequent
[full157 confirmation](issue100/retained-full157-results.md) reverses its storage
regression but shows foreground-latency costs. The ten-state target miss is not
proof that every permitted bounded design is impossible.

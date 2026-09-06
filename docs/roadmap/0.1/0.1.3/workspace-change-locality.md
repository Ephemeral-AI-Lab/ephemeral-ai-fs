# Workspace change locality

> **Status:** v0.1.3 implementation plan; no measurements or passing claim.
> **Family ID:** `workspace_change_locality`. Sixteen timed cases; no separate
> proofs. [Shared testing rules](testing-rules.md) govern common setup, seeds,
> custody, timing, verification, resource bounds, and admission.

## Question and fixture

Measure total Workspace size and dirty-work size independently. A clean Commit,
one small namespace change, sparse owner SDK edits, and dense ordinary writes
exercise different paths. Each is a four-tier curve, not a Cartesian matrix.

Tiers 1/10 reuse [`workspace-shards-v1`](testing-rules.md#shared-workspace-fixture)
through the compact-v2 subset: one shard has 200 regular files,
`128*1 KiB + 64*8 KiB + 8*48 KiB = 1 MiB`, including the wide directory,
regular siblings, 128-component spine, and empty `dest/`.

Tiers 100/500 use [`workspace-mixed-v4`](workspace-mixed-v4.md) as the **whole
Workspace**. No second 100k-file parent. Compact 1/10 file lists are unchanged.

| Active ID suffix | Files | Logical bytes | Largest file |
| --- | ---: | ---: | ---: |
| `-1-compact-v2` | 200 | 1,048,576 | 48 KiB |
| `-10-compact-v2` | 2,000 | 10,485,760 | 48 KiB |
| `-100-mixed-v4` | 2,000 | 104,857,600 | 50 MiB |
| `-500-mixed-v4` | 5,000 | 524,288,000 | 300 MiB |

Owner SDK edits preserve length and target only small/medium mixed-v4 files.
Dense writes replace files in place without a coexisting tree copy, including
the 50/300/100 MiB objects. Receipts stay outside the workload with separately
reported storage. Old unversioned 100/500 IDs (20,000/100,000 files, max 48 KiB)
are historical; see [#62](https://github.com/Ephemeral-AI-Lab/layerfs/issues/62).

## Exact member expansion

| Scenario ID | Fixture | Measured dirty work | Expected Commit |
| --- | --- | --- | --- |
| `workspace-clean-commit-1-compact-v2` | compact 1 shard | None; untouched Workspace | `UpToDate` |
| `workspace-clean-commit-10-compact-v2` | compact 10 shards | None; untouched Workspace | `UpToDate` |
| `workspace-clean-commit-100-mixed-v4` | mixed-v4 2,000 / 100 MiB | None; untouched Workspace | `UpToDate` |
| `workspace-clean-commit-500-mixed-v4` | mixed-v4 5,000 / 500 MiB | None; untouched Workspace | `UpToDate` |
| `workspace-fixed-move-1-compact-v2` | compact 1 shard | Move one fixed 1 KiB file | `Created` |
| `workspace-fixed-move-10-compact-v2` | compact 10 shards | Move the same fixed 1 KiB file | `Created` |
| `workspace-fixed-move-100-mixed-v4` | mixed-v4 2,000 / 100 MiB | Move `regular/s000/f064.dat` (4 KiB) to `dest/moved.dat` | `Created` |
| `workspace-fixed-move-500-mixed-v4` | mixed-v4 5,000 / 500 MiB | Move the same 4 KiB file | `Created` |
| `workspace-distributed-sdk-edit-1-compact-v2` | compact 1 shard | 1 singular SDK 4 KiB overwrite | `Created` |
| `workspace-distributed-sdk-edit-10-compact-v2` | compact 10 shards | 10 singular SDK 4 KiB overwrites | `Created` |
| `workspace-distributed-sdk-edit-100-mixed-v4` | mixed-v4 2,000 / 100 MiB | 100 singular 4 KiB overwrites on distinct small/medium files | `Created` |
| `workspace-distributed-sdk-edit-500-mixed-v4` | mixed-v4 5,000 / 500 MiB | 500 singular 4 KiB overwrites on distinct small/medium files | `Created` |
| `workspace-dense-rewrite-1-compact-v2` | compact 1 shard | Rewrite 200 files / 1 MiB | `Created` |
| `workspace-dense-rewrite-10-compact-v2` | compact 10 shards | Rewrite 2,000 files / 10 MiB | `Created` |
| `workspace-dense-rewrite-100-mixed-v4` | mixed-v4 2,000 / 100 MiB | Rewrite all 2,000 files / 100 MiB in place, including the 50 MiB object | `Created` |
| `workspace-dense-rewrite-500-mixed-v4` | mixed-v4 5,000 / 500 MiB | Rewrite all 5,000 files / 500 MiB in place, including the 300 MiB and 100 MiB objects | `Created` |

There are **16 timed cases and 48 performance samples**, three per case, with
separate verification. No fault, metadata, or link subcases are hidden here.

## Operations and attribution

**Clean Commit:** create the ready real-FUSE Workspace and immediately Commit,
without Exec, listing, stat, payload reads, or mutations. Measure Create,
Commit, and End separately. Require unchanged Branch/head/root, no Commit
insertion, no write transaction, and unchanged Store counts. Whole-tree scan
families instead visit namespace or payload; their clean Commit does not replace
this untouched control.

**Fixed move:** one fresh managed process moves the same prepared path,
`regular/s000/f064.dat`, to `dest/moved.dat` using one ordinary rename. On
compact 1/10 that source is 1 KiB; on mixed-v4 100/500 it is the 4 KiB small
file at ordinal 64. Changed file count, payload size, and endpoint directories
remain constant while the background grows. Complete the declared sync, close
all handles, and exit before Commit. Do not enumerate the tree merely to locate
this known path.

**Distributed SDK edits:** compact 1/10 keep the existing eligible-file rule
(index j >= 128 in each selected compact shard). Mixed-v4 100/500 select N
distinct eligible small/medium files from the mixed tree; never the 50/300/100
MiB blobs. The deterministic N-element prefix spreads targets across
directories; each target is edited once. Invoke
`Client::edit_workspace_file_range` N times, replacing exactly 4 KiB in place
with independently generated different bytes. No managed Exec remains active.
Report aggregate SDK edit wall and one Commit wall separately. The public batch
API is same-file only; do not invent a multi-file batch. This measures a dirty
file frontier across a Workspace; inherited SDK families own single-file size
and edit geometry.

**Dense rewrite:** compact 1/10 rewrite the compact shard files. Mixed-v4 100/500
rewrite **every** mixed-v4 file in place, including the 50/300/100 MiB objects,
through ordinary FUSE open/write/close on existing descriptors at offset 0, no
truncate, no per-file sync, then mtime 1700000000 and one root fsyncdir.
Preserve each logical length and use independently generated new content. This
is a dense ordinary-filesystem workload, not thousands of SDK edit calls.
Count every call in the workload. Finish all writes and exit before one Commit.

All curves use one Branch, one final unpromoted Commit attempt, and End. Cached
preparation may reuse an identical input Store but cannot precompute output.
Metadata normalization needed for the oracle is explicit measured work limited
to affected paths; the clean curve performs none. Report it separately without
removing it from the operation's wall.

## Independent verification and meaningful gates

Replay transformations against the frozen input manifest outside performance.
Compare every reopened path, bytes, length, type, mode, timestamp, and topology.
The move changes one binding; SDK oracles splice explicit 4 KiB replacements;
dense oracles replace exactly the selected files. Check unchanged files and
subtree identities, not just changed-file hashes. The clean control preserves
the complete input root. Candidate-produced roots are not their own oracle.

Record total and dirty files/bytes separately, SDK calls, FUSE operation/byte
counts, metadata/payload reads, candidate and inserted/reused objects,
transaction maxima, all phase walls, RSS, spool/Store growth, and cleanup.
Sparse dirty work must be distinguishable from full-tree capture or hashing.
Dense work may scale with all selected files/bytes.

The selected-case 1–5 second goal after cached preparation is provisional.
Dense 100,000-file rewriting and full verification may need larger qualified
budgets. Freeze per-case target/hard walls before admission; do not claim the
entire 48-sample campaign is a few seconds or silently skip large tiers.

## Source grounding and completion

- [`sdk_edit_common.rs`](../../../../benchmark/fs-bench-pro/families/sdk_edit_common.rs)
  owns the released four labels and bounded generator infrastructure.
- [`lifecycle.rs`](../../../../crates/layerfs-workspace/src/lifecycle.rs),
  `Workspace::commit` and `Workspaces::edit_workspace_file_ranges`, distinguish
  clean admission, owner edits, publication, and presentation.
- [`changes.rs`](../../../../crates/layerfs-workspace/src/changes.rs),
  `try_build_localized_candidate`, `base_manifest`, and `final_manifest`,
  motivates measuring whole-tree and dirty-frontier dependence.
- [`file_io.rs`](../../../../crates/layerfs-workspace/src/file_io.rs) owns ordinary
  dirty file state; [`client.rs`](../../../../crates/layerfs-sdk/src/client.rs)
  defines the public singular SDK entrypoint.

Completion requires all 16 members, full-tree proofs, qualified fixture/source
identities, bounded samples, cleanup, and pre-admission budgets. Product changes
follow measured causes rather than a prescribed optimization.

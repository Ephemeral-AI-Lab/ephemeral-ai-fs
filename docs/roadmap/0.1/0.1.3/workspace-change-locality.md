# Workspace change locality

> **Status:** v0.1.3 implementation plan; no measurements or passing claim.
> **Family ID:** `workspace_change_locality`. Sixteen timed cases; no separate
> proofs. [Shared testing rules](testing-rules.md) govern common setup, seeds,
> custody, timing, verification, resource bounds, and admission.

## Question and fixture

Measure total Workspace size and dirty-work size independently. A clean Commit,
one small namespace change, sparse owner SDK edits, and dense ordinary writes
exercise different paths. Each is a four-tier curve, not a Cartesian matrix.

Tiers 1/10 reuse compact-v2 slices of [`workspace-shards-v1`](testing-rules.md#shared-workspace-fixture).
Tiers 100/500 use the separate [`workspace-mixed-v4`](workspace-mixed-v4.md)
profile as the **whole Workspace**. Do not put a second 100k-file shards-v1
parent under sparse or dense mixed-v4 work. Do not mutate shards-v1 in place.

| Active ID suffix | Files | Logical bytes | Large objects |
| --- | ---: | ---: | --- |
| `-1-compact-v2` | compact 50-file shard | 1 MiB | none; max 48 KiB |
| `-10-compact-v2` | 10 compact shards | 10 MiB | none; max 48 KiB |
| `-100-mixed-v4` | 2,000 | 100 MiB | 1 × 50 MiB |
| `-500-mixed-v4` | 5,000 | 500 MiB | 1 × 300 MiB + 1 × 100 MiB |

Owner SDK edits preserve length. Dense writes replace files in place without a
coexisting tree copy. Receipts stay outside the workload with separately
reported storage. Old unversioned 100/500 IDs (`workspace-dense-rewrite-100`,
`workspace-clean-commit-500`, …) and their 20k/100k-file receipts are historical.

## Exact member expansion

| Scenario ID | Fixture | Measured dirty work | Expected Commit |
| --- | --- | --- | --- |
| `workspace-clean-commit-1-compact-v2` | compact 1 shard | None; untouched Workspace | `UpToDate` |
| `workspace-clean-commit-10-compact-v2` | compact 10 shards | None; untouched Workspace | `UpToDate` |
| `workspace-clean-commit-100-mixed-v4` | mixed-v4 2,000 / 100 MiB | None; untouched Workspace | `UpToDate` |
| `workspace-clean-commit-500-mixed-v4` | mixed-v4 5,000 / 500 MiB | None; untouched Workspace | `UpToDate` |
| `workspace-fixed-move-1-compact-v2` | compact 1 shard | Move one fixed 1 KiB file | `Created` |
| `workspace-fixed-move-10-compact-v2` | compact 10 shards | Move the same fixed 1 KiB file | `Created` |
| `workspace-fixed-move-100-mixed-v4` | mixed-v4 2,000 / 100 MiB | Move `regular/s000/f064.dat` (4 KiB) | `Created` |
| `workspace-fixed-move-500-mixed-v4` | mixed-v4 5,000 / 500 MiB | Move the same 4 KiB file | `Created` |
| `workspace-distributed-sdk-edit-1-compact-v2` | compact 1 shard | 1 singular SDK 4 KiB overwrite | `Created` |
| `workspace-distributed-sdk-edit-10-compact-v2` | compact 10 shards | 10 singular SDK 4 KiB overwrites | `Created` |
| `workspace-distributed-sdk-edit-100-mixed-v4` | mixed-v4 2,000 / 100 MiB | 100 singular 4 KiB overwrites on small/medium files | `Created` |
| `workspace-distributed-sdk-edit-500-mixed-v4` | mixed-v4 5,000 / 500 MiB | 500 singular 4 KiB overwrites on small/medium files | `Created` |
| `workspace-dense-rewrite-1-compact-v2` | compact 1 shard | Rewrite compact shard files / 1 MiB | `Created` |
| `workspace-dense-rewrite-10-compact-v2` | compact 10 shards | Rewrite 10 compact shards / 10 MiB | `Created` |
| `workspace-dense-rewrite-100-mixed-v4` | mixed-v4 2,000 / 100 MiB | Rewrite every mixed-v4 file, including the 50 MiB object | `Created` |
| `workspace-dense-rewrite-500-mixed-v4` | mixed-v4 5,000 / 500 MiB | Rewrite every mixed-v4 file, including the 300 MiB and 100 MiB objects | `Created` |

There are **16 timed cases and 48 performance samples**, three per case, with
separate verification. No fault, metadata, or link subcases are hidden here.

## Operations and attribution

**Clean Commit:** create the ready real-FUSE Workspace and immediately Commit,
without Exec, listing, stat, payload reads, or mutations. Measure Create,
Commit, and End separately. Require unchanged Branch/head/root, no Commit
insertion, no write transaction, and unchanged Store counts. Whole-tree scan
families instead visit namespace or payload; their clean Commit does not replace
this untouched control.

**Fixed move:** one fresh managed process moves the frozen path
`regular/s000/f064.dat` to `dest/moved.dat` using one ordinary rename. Compact
1/10 that source is 1 KiB; mixed-v4 100/500 it is the 4 KiB small file at
ordinal 64. Changed file count, payload size, and endpoint directories remain
constant while the background grows. Complete the declared sync, close all
handles, and exit before Commit. Do not enumerate the tree merely to locate
this known path.

**Distributed SDK edits:** N singular 4 KiB overwrites. Compact 1/10 keep the
historical shard `j >= 128` selection on the compact tree. Mixed-v4 100/500
select N distinct eligible **small or medium** files on the mixed tree; never
the 50/300/100 MiB blobs. Invoke `Client::edit_workspace_file_range` N times,
replacing exactly 4 KiB in place with independently generated different bytes.
No managed Exec remains active. Report aggregate SDK edit wall and one Commit
wall separately. The public batch API is same-file only; do not invent a
multi-file batch.

**Dense rewrite:** compact 1/10 rewrite the selected compact shards. Mixed-v4
100/500 rewrite **every** mixed-v4 file, including the large objects, through
ordinary FUSE existing-file writes at offset 0, no truncate, no per-file sync,
then mtime `1700000000` and one root `fsyncdir`. Preserve each logical length
and use independently generated new content. This is a dense ordinary-filesystem
workload, not 2,000/5,000 SDK edit calls. Count every call in the workload.
Finish all writes and exit before one Commit.

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
Mixed-v4 dense rewrite and large-file proofs use sampled ranges, not exhaustive
100k-file walks. Freeze per-case target/hard walls before admission; do not
claim the entire 48-sample campaign is a few seconds or silently skip large
tiers. Historical 15-second family status is reporting-only for mixed-v4
collection.

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

# Directory construction and whole-workspace traversal

> **Status:** Current v0.1.3 planning specification; no release candidate or
> measured result is implied.

Family ID: `directory_construction_traversal`. This v0.1.3 implementation
contract contains **12 new timed cases and no standalone proofs**. Its family
adapters are not implemented yet.

## Purpose and scope

Measure directory construction, complete metadata enumeration, and complete
content scans as distinct operations. A scan must visit the whole selected
workspace, including its wide directory and deep path witness; visiting a few
independent short chains does not establish whole-tree behavior.

Use the [shared testing rules](testing-rules.md) for seeds, sample preparation,
source custody, exact manifests, byte accounting, timing, and resource gates.
All cases use public Workspace Create, managed execution through real FUSE,
one final Commit attempt, and clean End. Separate verification reconnects to
the Store and mounts a new Workspace.

## Cases and load units

Expand `N` over exactly `1, 10, 100, 500`; the three rows below define four
scenario IDs each.

| Scenario IDs | Primary load | Measured operation | Expected Commit |
| --- | --- | --- | --- |
| `directory-construct-{N}-compact-v2` (N=1,10) | N directory chains on compact background | Create each missing chain with ordinary root-to-leaf `mkdir` calls | `Created` |
| `directory-construct-{N}-mixed-v4` (N=100,500) | N directory chains on mixed-v4 background | Create each missing chain with ordinary root-to-leaf `mkdir` calls | `Created` |
| `directory-metadata-scan-{N}-compact-v2` (N=1,10) | compact tree | Enumerate the complete workspace, `lstat` every entry, and count completed entries | `UpToDate` |
| `directory-metadata-scan-{N}-mixed-v4` (N=100,500) | mixed-v4 2,000/5,000 files | Enumerate the complete mixed tree, `lstat` every entry, and count completed entries | `UpToDate` |
| `directory-content-scan-{N}-compact-v2` (N=1,10) | compact tree | Enumerate the complete workspace, open/read/close every regular file, and count completed bytes | `UpToDate` |
| `directory-content-scan-{N}-mixed-v4` (N=100,500) | mixed-v4 2,000/5,000 files / 100/500 MiB | Enumerate the complete mixed tree, open/read/close every regular file, and count completed bytes | `UpToDate` |

Tiers 100/500 use the [`workspace-mixed-v4`](workspace-mixed-v4.md) tree as the
whole Workspace (Wave 2 of [#62](https://github.com/Ephemeral-AI-Lab/layerfs/issues/62)).
Wide-directory fan-out is hundreds to low thousands, not 32,000. Compact 1/10
IDs and file lists are unchanged. Old unversioned 100/500 IDs are historical.

Construction uses an untouched shared workspace, with new chains under
`new-directories/`. Compact 1/10 keep the compact background; mixed-v4 100/500
use the mixed tree as background and do not stack a second 100k-file parent.
Its 500-chain seed-bound schedule uses domain `directory-construction`; each
smaller case is an exact prefix. Chain depths repeat by scheduled ordinal:

```text
1, 4, 2, 8, 3, 10, 5, 7, 6, 9
```

Names, modes, and normalized mtimes derive from the sealed fixture schedule.
Report chain units and actual `mkdir`/affected-directory counts separately.
Creation does not rewrite or copy the background payload.

Both scan curves use the complete N-shard
[shared workspace fixture](testing-rules.md#shared-workspace-fixture), including
its fixed wide-directory layout and 128-component spine. Each shard contains
200 files and exactly 1 MiB: tiers are 200/2,000/20,000/100,000 files and
1/10/100/500 MiB. Scan every directory component as well as every file; retain
actual entry count, fan-out, and maximum depth. Ordered path transcripts and
their digests are collected only in separate verification executions.

Traversal uses ordinary directory handles and consumes every pagination page
until EOF. Resolve and visit entries in bytewise name order. Performance mode
retains cheap counts; separate verification records the exact transcript and
rejects missing or repeated entries even when a total count matches.
Metadata scans do not open/read file payloads. Content scans use bounded buffers,
read each entire file through EOF, and count completed bytes. Only verification
mode computes per-file hashes and an ordered whole-tree content digest. Metadata
and payload scans remain distinct because metadata work can pass while payload
delivery fails or scales badly.

The largest scan or construction workspace contains 500 MiB of logical file
payload; added construction paths carry no payload. The shared profile's
largest file is 48 KiB. Fixed directory witnesses add no regular-file bytes.
No secondary materialized tree is created in the tested namespace.

## Timing, metrics, and independent oracle

Measure the entire traversal or construction inside the workload timer.
Directory reads, per-entry metadata calls, and content reads are measured work.
Full hashes, transcript checks, and tree oracles never run in performance mode.
Keep Workspace Create, ordinary sync for construction,
Commit, visibility, End, and complete lifecycle walls separately attributable.
Independent full verification has its own wall, receipts, and deadline, and
never contributes to a performance distribution.

Record directories and regular files processed, resolved components, payload
bytes, fan-out/depth, FUSE lookup/getattr/mkdir/opendir/readdir/releasedir/read
counts, CPU, memory, swap/OOM, Store/object changes, and cleanup. Content scans
report logical payload throughput; metadata scans report entries per second.
Do not divide directory-only work by payload bytes to invent throughput.

The expected manifest is sealed independently from the product and timed
traversal. The separate verifier compares every path, type, size, mode, declared
mtime, content digest, and relevant link identity after fresh Store reconnect
and real-FUSE remount. Construction adds exactly the scheduled directories and
preserves all unrelated paths. Scans preserve the whole manifest, Branch head,
and canonical root, return `UpToDate`, and create no durable payload state.
Verify exact traversal membership, EOF behavior, full-content read transcripts,
transient byte bounds, and absence of leaked handles or runtime resources.

## Execution and completion

Three fresh performance samples per case produce **36 timed executions**;
every case receives separate independent verification. Reuse the shared shard
manifests and prepared input Stores through the existing custody/sample-clone
machinery; each timed execution still owns independent mutable runtime state.
The benchmark binary, workload helper, runner, and report machinery are reused.

Prospective selection is one case and one seed, followed by its focused
verification; the selector is not implemented yet. Start with the affected
`directory-construct-1`, `directory-metadata-scan-1`, or
`directory-content-scan-1`. The provisional ordinary selected-run target is
1–5 seconds. Large scans, whole families, and exhaustive verification use the
longer lane with baseline-derived budgets; the 100,000-file scan is never
advertised as a guaranteed few-second test.

Completion requires all 12 cases and seed samples, exact independent full-tree
proofs, all pages and file bytes visited, bounded resource and size evidence,
and clean teardown. Existing historical shell-directory controls keep their
own meanings and are not additional members of this family.

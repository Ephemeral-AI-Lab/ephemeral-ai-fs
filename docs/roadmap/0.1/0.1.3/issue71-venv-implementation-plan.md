# Issue 71: bounded shared construction and real-environment import

Issue: https://github.com/Ephemeral-AI-Lab/layerfs/issues/71

## Target implementation plan: shared construction improvements, native scheduling, and Workspace Commit

Proceed on the separate `codex/fix-nested-init` worktree, retaining the tested buffer fix `71d255505`. The result will be submitted as a pull request after validation; **no merge to main**. This plan supersedes the separate-mode-cache suggestion with reuse of the existing eight cache slots.

### Implementation slices

1. **Shared metadata component reuse:** extend each existing `PortableMetadataCache` entry with its already-constructed mode root. Keep exactly eight slots, existing FIFO eviction, validation, and operation/Store-bound lifetime. On a complete tuple miss, reuse a matching validated normalized mode from those same slots, build the exact timestamp and final metadata. No separate/global/unbounded cache; no timestamp normalization. Preserve `get_by_root` validation used by Workspace Commit. Both native import and changed Workspace metadata already use this cache.
2. **Known-sorted directory construction:** reuse the existing bounded sorted-tree engine for sorted native directory input, preserving arbitrary-order builder semantics and exact canonical output. Do not simply remove the 512-entry preview guard. Retain the sorted scratch and deferred ownership limits, failure handling, and the existing sorted update path for changed Workspace directories.
3. **Bounded nested native frontier:** generalize root/single-flat planning into bounded contiguous preorder subtree/file tasks plus compact expanded-parent records. Reuse existing workers, bounded output, pair streams and checked admission. Preserve canonical preorder with ordered ancestor splices and validate coverage. Cap tasks/descriptors/skeleton ownership; stop expansion and leave a subtree unsplit rather than trigger a whole-import restart. Keep existing hard-link fallback. No full-tree byte preflight, whole-input flattening, new thread pool, or change to CDC boundaries.
4. **Shared small-file completion:** below the frozen minimum chunk length, read into a bounded buffer, validate exact length and EOF, and reuse the existing one-chunk construction. Larger files retain streaming CDC; incremental range edits retain localized reuse. Share the helper used by native initialization and full-file Workspace construction, preserving completed-file facts and failure atomicity.
5. **Workspace worker budgeting:** replace the unconditional single-worker production selection with a bounded allowance using existing CPU, eligible task and aggregate memory limits. Keep frozen dirty inputs, existing private full-file validation/selection, sparse edits, capture reuse, and Commit publication/head checks. Native traversal and Workspace dirty-node discovery remain separate adapters over shared builders and producer/admission machinery.

### Correctness/resource gates

- Exact baseline/candidate canonical roots and object bytes at fixed seeds, including 512/513/wide-directory boundaries, nested parent preorder, reversed worker completion, metadata validation/eviction, symlinks and hard-link fallback.
- Small-file parity at 0/1/8191/8192 bytes; short, fragmented and erroring reads; source growth/truncation must not be accepted.
- Keep slab payload <=256 KiB, <=512 objects, queue <=4 slabs, existing transaction bounds and aggregate partitioned worker allowances. Cache entry count stays eight; total process memory is measured rather than claimed constant.
- Producer cancellation/drain/join, late publication failure, clean Store recovery, unchanged roots on failure, and Workspace candidate finality must pass.
- Build, formatting, warning-denying Clippy, relevant crate tests, and live Workspace/FUSE checks on the final source. Preserve existing ignored-test and coverage boundaries explicitly.

### Measurement and success criteria

- Native: unchanged real `.venv` (16,395 files /581,658,413bytes) plus all four current namespace profiles; fresh processes and Stores, default-options paired samples, fixed sample count/order, same compiler/dependencies and no concurrent builds or workloads. Keep all outcomes. Prior results remain immutable.
- Workspace: equivalent source bytes/entries through ordinary public Exec and real FUSE, then ordinary public Commit; host owns SQLite/coordinator/publication/spool, Linux container only daemon/FUSE/workload. Record Exec, Commit, Exec+Commit and lifecycle separately; no relocation of work outside timing to claim a gain.
- Verify full real-source file bytes, lengths, inventories and symlink targets outside timing, with metadata/canonical parity covered explicitly. Do not import host `.git` or benchmark output into the fixture.
- Demonstrate reduced algorithmic work (mode object reconstruction, transient directory work, streaming setup for tiny files, producer skew) and lower same-input native median. Measure Workspace effects independently; never infer its speedup from native results. Check existing namespace inputs for regressions; report noisy or unmet targets honestly.
- Approximately0.74–1.03s for581.7MB at historical low-cardinality throughput is an investigation reference, **not a promised universal subsecond result**. A final candidate that fails correctness/resources or materially regresses affected cases is not ready for the PR.

Commit the implementation specification before collection. Keep compact reproduction commands/results in the PR and link this issue. Retain rejected attempts instead of silently dropping them. Make no new API/format change or broad issue-closure claim: remaining hard-link, extreme-shape or general scheduling limitations must be explicit.

## Approved checkpoint extension (Workspace proof revision 3)

The full 17,682-node environment exceeded the live checkpoint receiver’s fixed 16,384-record map. The released baseline saved its Store but failed presentation and clean shutdown; that attempt remains failed evidence. The user approved fixing this shared Workspace checkpoint bottleneck in the same PR on September 8, 2026. See the [issue update](https://github.com/Ephemeral-AI-Lab/layerfs/issues/71#issuecomment-5587697624).

Replace retained checkpoint records with bounded-memory staging using transient live-operation metadata scratch bounded by admitted live nodes. Validate the complete checkpoint before applying records; preserve duplicate/identity validation, generation checks, retry, view isolation, and cleanup. Keep canonical construction, SQLite, publication, and payload spool on the host. Do not increase the record limit or caches to fit this fixture.

Before final sampling, build the matching runtime and pass focused checks for more than 16,384 records, malformed/duplicate records, interrupted application/retry, and cleanup. The public probe must use Commit status and reject presentation failure, then verify all source content and metadata and require End(Clean). Collect three candidate full-environment runs with unchanged source and version-2 tar records, fresh Stores/processes, and the same container bounds. Report them as successful candidate timings only: the untouched release cannot complete this case, so there is no valid full-environment Workspace speedup ratio. Preserve the completed independent native pairs and report namespace controls as exploratory one-sample checks.

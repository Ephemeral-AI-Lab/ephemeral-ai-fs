# Benchmark success before v0.1.5 implementation

> **First-round scope (owner update, 2026-09-09):** Use only one **10-file,
> 30-commit smoke test** for the initial implementation/measurement loop, including
> verification of its 31 retained states. Broader benchmark matrices and release
> qualification follow after that loop is stable. The original three-file tiny
> baseline remains unchanged and is not the new smoke or its numerical control.
> See [the current execution scope](delta-encoding-benchmarks.md#first-round-execution-scope).

> **Status:** Source/history review, 2026-09-09; implementation guidance, not a new
> benchmark contract, benchmark execution, or release qualification.

Read this and [past_mistake.md](past_mistake.md) before finalizing the v0.1.5
specification. Success means less real storage/work with correct public behavior,
not a smaller timer obtained by moving work or weakening coverage.

## Fixed direction and current first loop

The agreed file cutoff is **128 KiB**; new SQLite Stores retain **4 KiB pages**.
The current [workflow](workflow.md) extends the canonical representation for new
small content: even explicit small-file edits require bounded complete-target
preparation. Large-file range edits retain extent sharing. Old canonical objects
and histories remain readable; new small representations need explicit versioning.
Do not reopen a threshold sweep during implementation.

The [delta benchmark plan](delta-encoding-benchmarks.md) adds proposed small-file
mixed-length history coverage without changing existing cases. The 56 released
SDK edit cases are all large files: 12 length-preserving, 32 length-changing, and
12 canonical chunk-count cases. Byte/line count, extent count, and physical record
count are distinct. The last family changes extent count at constant file length.
The released verifier checks roots and boundary bytes; it is not a full-file
history oracle. Its zero-payload-read/retained-extent assertions still apply to
those large cases, not automatically to new whole-file small objects.

Earlier all-size locality advice below must be read with that scope. New small
history qualification checks complete bytes and one-level reconstruction, measures
base/target work, and does not impose a CDC member-count oracle. The new plan also
records the expected storage versus processing tradeoff for deletion, truncation,
and zero extension. No small-case speed or storage win has been measured yet.

The first implementation loop is now only the **ten-file/thirty-commit smoke**
and its own 31-state verification; see [execution scope](delta-encoding-benchmarks.md#first-round-execution-scope).
Use a separately sealed released control for those same ten files. Defer the
broader matrix and existing-family reruns until this smoke loop is stable.

The earlier **tiny_rewrite_history / tiny-history-30** uses three 4/16/40-KiB files,
30 eight-byte line changes saved as complete files, and 31 retained states. Its
recorded baseline stays immutable and remains a separate later comparison.

| Existing tiny baseline | Observed value |
| --- | ---: |
| Median paired ordinary save + Commit | 13.709 ms |
| Initial / final allocated Store | 88 / 216 KiB |
| Allocated history growth | 128 KiB |
| Created commits / verified states | 30 / 31 |

This is one synthetic-history observation, not 30 independent repetitions or a
release target. The baseline already includes packed FULL/PREFIX representations.
The [case contract](tiny-history-baseline-v1.md) and the local baseline evidence
at `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v015-tiny-history-runs/baseline-01/`
retain exact identities, raw receipts, growth.csv and harness.patch. No new
benchmarks, builds or product tests were run for this document review.

## Released control now required

The inventory below describes the historical packed snapshot and tiny harness.
The v0.1.5 implementation baseline is now released v0.1.4 at `101fa273d815f3aaedb0e06ba0de7b0777d83def`.
Use its [complete case-by-case closeout](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/release-notes/0.1.4/benchmark-closeout.md)
for release results and the updated [past mistakes](past_mistake.md) for #95/#98.
The existing tiny observation is retained exactly; a separate control on the
released product is required before delta changes, using the same frozen tiny
case. Keep absolute candidate gates; do not credit earlier released repairs to
v0.1.5 or claim a fresh control was collected by this documentation update.

## Source and inventory authority

Review source: packed schema-7 evidence head `cf3a058925c3012fda0fae922dc081766bb8fa99`
(measured R26 product `861e388339ef5572659cb16ef0c8febbff0351df`) plus the isolated
tiny-history harness additions. This is not the older main checkout's row-BLOB
Store. Later issue/commit work needs its own validation; historical numbers do
not qualify different bytes. Issue #18 moved v0.1.4 to storage work and the older
multi-Branch scope onward; consult [that issue history](https://github.com/Ephemeral-AI-Lab/layerfs/issues/18)
rather than inferring implemented release scope from the stale main README.

Static inventory at the reviewed source:

- **19 family directories** inspected, including the new tiny family.
- **17 ordinary host families**: 132 timed Workspace IDs, 56 SDK edit IDs,
  four initialization IDs and six footprint controls = **198 timed IDs**.
- **28 reliability proof definitions plus one CDC boundary definition** are
  separate. The normal selected verifier excludes sustained-600s, leaving
  28 routine proof-only executions in the described campaign. Together with
  198 case verifications, this explains 226 routine verifications; these are
  different count dimensions, not interchangeable success claims.
- The **one tiny rewrite case** is a special Python dispatch. The ordinary
  checkpoint collector does not automatically include it.
- The older **edit_length_changing_capped** directory has five legacy cases;
  Python HOST_FAMILIES excludes it. Active capped-result coverage is already
  incorporated in edit_length_changing. Do not revive the legacy runner.
- Storage smokes and DeepSeek five/full157 use another explicit entrypoint.

Sources: [benchmark/fs-bench-pro/shared/runner.py:26](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/shared/runner.py#L26),
[benchmark/fs-bench-pro/workload/workspace_registry.rs:12](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/workspace_registry.rs#L12), and
[benchmark/fs-bench-pro/issue54_collect.py:423](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/issue54_collect.py#L423). The top-level benchmark README's
old 18-folder/universal-mod.rs description is not the actual registry.

## Complete family matrix

All family definition references below start in `benchmark/fs-bench-pro/families/`. Hot-path guidance is inference from the measured operation, not a newly imposed numerical gate.

| Family (case IDs) | Actual operation and key source | What implementation must preserve/optimize | Trap for delta work |
|---|---|---|---|
| payload_create_read (8) | Four sequential create sizes 1/10/100/500 MiB and four random-4-KiB-read counts. [payload_create_read/mod.rs:6](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/payload_create_read/mod.rs#L6); [workload/ordinary_workloads.rs:2087](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/ordinary_workloads.rs#L2087) | Stream construction/admission; bound buffers; map random ranges without whole-file traversal or decode. Larger read cases use a 500-MiB payload while low compact cases use their tier size. | Small read must not fetch/decode complete pack/base unnecessarily; no similarity search on unique initial creation. |
| tiny_file_churn (20) | create/stat/unlink/bulk-create/bulk-delete × four tiers, compact-v2 low cases and mixed-v3/v4 high cases. [tiny_file_churn/mod.rs:6](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/tiny_file_churn/mod.rs#L6); [ordinary_workloads.rs:2114](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/ordinary_workloads.rs#L2114) | Cheap namespace lookup; share staging; coalesce writes; batch admission; avoid transaction/query per inode or unchanged metadata. | A smaller record may lose to extra SQL lookups, pack framing, or one base dependency per tiny metadata object. Stat/unlink do not need payload decode. |
| directory_construction_traversal (12) | mkdir chains, metadata scan, full content scan × tiers. [directory_construction_traversal/mod.rs:6](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/directory_construction_traversal/mod.rs#L6); [ordinary_workloads.rs:2136](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/ordinary_workloads.rs#L2136), scan implementation `:1160` | Efficient lazy directory lookup/readdir and bounded metadata fetch; content scans batch payload reads. | Do not conflate metadata scan with payload decoding; preserve exact path and symlink semantics and clean commit behavior. |
| namespace_mutation (4) | subtree rename then recursive delete. [namespace_mutation/mod.rs:6](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/namespace_mutation/mod.rs#L6); [ordinary_workloads.rs:2173](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/ordinary_workloads.rs#L2173) | Share subtree roots, update affected ancestors, metadata-only operations avoid content hydration. Recursive POSIX delete still performs declared real calls. | Delta base bookkeeping must not turn rename into file reconstruction or global dependency scan. |
| workspace_change_locality (16) | clean Commit, fixed move, distributed singular SDK edits, dense full rewrites × tiers. [workspace_change_locality/mod.rs:6](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/workspace_change_locality/mod.rs#L6); contract [docs/roadmap/0.1/0.1.3/workspace-change-locality.md:59](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/workspace-change-locality.md#L59) | Clean Commit: unchanged head/root, no write transaction. Dirty publication scales with changed frontier; explicit edits preserve extents; dense rewrite must stream. | Do not enumerate background to find known paths; do not invent a cross-file batch API; current batch API is same-file. Whole rewrite and range edit are different surfaces. |
| git_tool_workflow (4) | real Git status/diff/add/cached-diff/commit/status with normalized exact frozen changed targets. [git_tool_workflow/mod.rs:6](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/git_tool_workflow/mod.rs#L6); [ordinary_workloads.rs:1575](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/ordinary_workloads.rs#L1575); config `:1370` | Low read/lstat/write/rename overhead through actual FUSE; exact Git metadata and head/tree/parent checks. | Git already does intrinsic hashing/read work; must remain measured. This is not a raw delta-codec test. Never replace Git mutations with SDK writes or cache output. |
| mixed_load_bearing (4) | dependent agent episodes: read, in-place edit, alias read, append, truncate, rename, tempfile save, symlink read, scratch unlink, output. [mixed_load_bearing/mod.rs:6](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/mixed_load_bearing/mod.rs#L6); [ordinary_workloads.rs:1245](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/ordinary_workloads.rs#L1245) | Correct read-after-write, alias/open-handle coherence, efficient mixed reads/writes, bound lock scope. | Deferral/cache optimization must preserve intermediate results; a final-tree pass alone misses stale read/alias failures. |
| dedup_cross_file (10) | public initialize_layerstack import: one anchor + unique/identical/mixed × 10/100/500 one-MiB files. [dedup_cross_file/mod.rs:5](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/dedup_cross_file/mod.rs#L5); [workload/dedup_workloads.rs:431](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/dedup_workloads.rs#L431) | Exact CAS reuse with batched membership/admission, full input scan measured; unique negative control stays bounded. | Do not infer dedup from candidate saved_fraction (coalescing hides emissions); distinguish canonical savings from SQLite bytes. |
| dedup_cdc_locality (20 + 1 proof) | reference one-MiB file + overwrite/insert/delete/common-body/scattered variants × 1/10/100/500, imported independently. [dedup_cdc_locality/mod.rs:5](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/dedup_cdc_locality/mod.rs#L5) | Frozen CDC transcripts/IDs/boundaries; middle sharing and honest resynchronization; chunk-level read efficiency. | Physical delta does not increase canonical exact reuse; do not change CDC boundaries silently to pass a storage target. Boundary proof uses 0/1/8191/8192/16384/32768/32769 B (`mod.rs:33`). |
| dedup_workspace_reuse (14) | append exact/local/unique content to already populated Store; low compact controls and explicit base128-v3 controls. [dedup_workspace_reuse/mod.rs:5](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/dedup_workspace_reuse/mod.rs#L5) | Reuse preexisting objects without reading/rebuilding unrelated base; distinguish source bytes scanned vs preexisting content reused. | Compact base is not the same scaling baseline as 128-file base. No database-wide similarity search for new unique files. |
| dedup_branch_history (20) | distributed/hotset/recurring SDK edits, metadata-only, unrelated whole rewrites × 1/10/100/500 commits. [dedup_branch_history/mod.rs:5](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/dedup_branch_history/mod.rs#L5); [dedup_workloads.rs:274](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/dedup_workloads.rs#L274) | Stable per-step cost as history grows, bounded base dependencies, accurate history allocation, immutable parent snapshots, exact/no-change reuse. | Range-edit history already benefits from extents; recurring A/B is exact reuse. Neither alone proves whole-file delta gains. High unrelated cases are versioned mixed-v2; routine historical reads are sampled at depth >10 (`mod.rs:23`). |
| edit_length_preserving (12) | one public SDK 4-KiB overwrite at head/middle/tail × 1/10/100/500 MiB. [edit_length_preserving/mod.rs:20](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/edit_length_preserving/mod.rs#L20); [workload/sdk_edit_common.rs:4](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/sdk_edit_common.rs#L4); [src/sdk_file_edit.rs:36](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/src/sdk_file_edit.rs#L36) | Edit+Commit should depend on changed span/tree path, not base size; retain untouched extents with bounded spool/candidates/CDC work. | No delta discovery by whole-file materialization on known edits; no FUSE write substitute. |
| edit_length_changing (32) | insert/delete/append/prepend/grow/shrink/truncate/zero-extend × four size tiers; final size capped at 500 MiB for growing last-tier cases. [edit_length_changing/mod.rs:33](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/edit_length_changing/mod.rs#L33), `:104` | Persistent piece tree and zero ranges; avoid tail copies, full rehash/rechunk, proportional spools. | Flat whole-file identity/reconstruction must not reintroduce O(file size) writes for localized insertions. Keep case identity and exact output size. |
| edit_canonical_chunk_count (12) | fixed 64-KiB overwrite at fixed offset, resulting canonical chunk count preserve/increase/decrease × four tiers. [edit_canonical_chunk_count/mod.rs:7](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/edit_canonical_chunk_count/mod.rs#L7), `:23` | Efficient extent mapping even when chunk cardinality changes; exact frozen roots/maps. | Delta physical format must preserve canonical bytes/IDs and logical chunk-count contract; physical record count is a separate quantity. |
| init_namespace (4) | public native namespace import 100/1k/10k/100k regular files; 5/20/300/500 decimal MB with anchors and small-heavy mix. [init_namespace/mod.rs:39](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/init_namespace/mod.rs#L39) | Fast per-file producer, prevalidated ownership, batch SQL admission, streaming hashing/content construction, bounded memory. | No base means little delta gain; base hunting/compression overhead must not regress bulk initialization; count table/index/page cost. |
| store_footprint (6) | 3 high unique/metadata-cardinality/large-object controls at 500 decimal MB; 3 explicit low controls at 5/5/10 MB. [store_footprint/mod.rs:27](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/store_footprint/mod.rs#L27) | Total physical SQLite cost, payload + structural + index + page slack; stable bounded import. | Small delta size is not DB savings; required full bases, record directories, empty roots, page size/freelist matter. No post-hoc VACUUM credit. |
| workspace_reliability (28 proof definitions) | lifecycle/busy/retry/publication/dirty/no-space/cancel/corruption/concurrency/link/metadata/repeated-publication. [workspace_reliability/mod.rs:5](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/workspace_reliability/mod.rs#L5) | Errors preserve previous published state; retry idempotence; exact full/base decode, lifetime and alias correctness; cleanup. | One-level/dependency correctness needs tests; decoder cannot trust malformed offsets or lengths. `sustained-600s` is explicitly rejected by selected infra at [src/infra.rs:209](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/src/infra.rs#L209); not routine PASS. |
| tiny_rewrite_history (1 special case) | native Init of 4/16/40-KiB source-like files, 30 full rewrites through real FUSE, each changes 8 ASCII bytes in one line, 31 retained states. [shared/tiny_history_fixture.py:6](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v015-tiny-history/benchmark/fs-bench-pro/shared/tiny_history_fixture.py:6); [families/tiny_rewrite_history/README.md:1](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v015-tiny-history/benchmark/fs-bench-pro/families/tiny_rewrite_history/README.md:1) | First iteration: reduced allocated growth with comparable Exec+Commit latency, all 31 states exact after reconnect. | Single observation/exploratory, not percentile evidence. Does not cover 128-KiB boundary, length-changing/scattered edits, multi-level paths, near-limit small files or many-file scale. Keep unchanged for baseline comparison; add later tests separately. |
| edit_length_changing_capped (5 legacy module) | prior capped-prefix replacements; [edit_length_changing_capped/mod.rs:1](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/families/edit_length_changing_capped/mod.rs#L1); registry inherited [workspace_registry.rs:31](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/workload/workspace_registry.rs#L31) | Consult to understand prior bounds and identities. | Excluded from HOST_FAMILIES; not extra active success coverage. |

## Special storage runners (not extra ordinary family directories)

[shared/storage_smoke.py:24](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/shared/storage_smoke.py#L24) defines `deepseek-five` (1 case), `deepseek-full` (1 case), `small-files` (1), `frequent-edits` (4), and the tiny case already counted above. Dispatch is [shared/runner.py:609](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/shared/runner.py#L609).

- **Frequent edits**: sdk-text-32k, sdk-binary-8m, fuse-text-32k, fuse-binary-8m. Five steps: three localized changes, return to original, no-change; generated at storage_smoke.py:167. SDK and FUSE must stay separated. FUSE complete rewrites/tempfile behavior finds lost base provenance; binary negative control guards useless delta work. Preserve exact prior-content reuse and UpToDate count.
- **Small files**: 128 regular files, empties/tiny/text/duplicates/incompressible and symlinks; baseline read passes, 8191→8193→8191 length crossing and metadata edit. storage_smoke.py:181 and :293. Guard page/framing overhead and requested-vs-decoded bytes; warm read success does not bound cold reconstruction.
- **Five/full157 DeepSeek**: real Git-derived frozen source manifest, ordinary whole-file importer/removal/type/mode transitions, one Branch, repeated public Exec+Commit; no Add, no SDK substitution. Shared smoke.py:94 and :305; full contract [docs/roadmap/0.1/0.1.4/deepseek-full-157-m45-contract.md:1](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/deepseek-full-157-m45-contract.md#L1). Count total Store allocation after each acknowledgement before verification creates extra branches. Full157 is extended later confirmation, not a per-patch default. Historical comparison to delta-packed Git is not matched timing or metadata equivalence.

## Cross-cutting implementation and evidence requirements

1. **Preserve useful algorithmic locality.** Known large-file edit: no work proportional to untouched payload. New small-file construction may assemble/hash the bounded complete target and decode a bounded base. Metadata-only move/clean Commit: no payload decode; history accumulation: no global base search or linear prior-history scan. Keep SDK edits and ordinary FUSE saves as distinct measured surfaces.
2. **Optimize whole pipeline, not codec alone.** Track selection/read/decode/match/encode/canonical validation/SQL write/publication separately. Keep codec CPU out of shared SQL critical sections; batch index rows and base reads; don't repeatedly fetch/decode same pack group for siblings. These are proposed implementation tactics; benchmark results must establish effect.
3. **Measure real storage.** Total allocated/apparent DB + required sidecars, SQLite page_count/freelist, canonical and encoded bytes by role, retained full bases, physical full/delta/group/pack counts. Fragment slices retain entire source chunks; never count only referenced byte lengths as actual stored payload. Definitions: [docs/roadmap/0.1/0.1.3/dedup-cross-file.md:82](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/dedup-cross-file.md#L82) and storage-smoke plan section6.
4. **Protect read costs.** Range requests, many tiny opens, scans, Git and historical reopen all hit decoding. Track requested vs fetched/decoded bytes, base/group fetch counts, per-operation reuse, cold-vs-warm status, CPU/RSS. A compact hotset may hide worst-case cold read amplification.
5. **Actual public surface and acknowledgements.** Host SDK edits through edit_workspace_file_range(s), no POSIX replacement; ordinary filesystem/Git importer must really traverse FUSE. No data-sharing mount, direct SQL writes, historical Docker-owned Store, or benchmark scenario branch in product. benchmark/AGENTS.md is hosting authority, rules sections2–4.
6. **Pure timing + counts.** Count public calls, RPCs, FUSE reads/writes, transactions, polls, fallback. Setup/fixtures/oracle/storage census/verification outside operation timer. Intrinsic CAS/Git authentication remains product work. Matching arms have identical harness/workload/fixture/cache/timer, with only product treatment different. rules sections4–8.
7. **Evidence authenticity is equal to speed.** Predeclared contract/issue needed for admission; tiny intentionally exploratory without issue. Every expected member/proof, exact source/product/image/fixture seals, unique retained output and explicit failure class. Missing metric is unavailable, never zero. Do not claim single history-step medians are independent repetitions; no baseline changed workload ratios. rules sections1,7–14.
8. **Resource and correctness gates cannot be traded away.** Separate host RSS, container anonymous/cache, spool disk, DB allocation and verifier resources; no swap/OOM. Verify current and historical paths/bytes/modes/types/links independently after timing, plus malformed/missing/corrupt base and publication failure behavior in focused product tests. rules sections10–12.
9. **Fast iteration already specified.** Product-free self-check → smallest relevant selected case + focused correctness → inspect → shared root fix → relevant sibling/size → final affected-family proofs. Shared lock excludes overlapping builds/measurements. Reuse qualified immutable preparations/binaries, never mutated result Stores. No repeated full campaigns to diagnose one miss. rules section15, QUICKSTART.
10. **Frozen physical vs canonical contracts.** FULL/DELTA encoding preserves the identity of a given canonical object. The newly agreed whole-file small-object model also introduces a canonical representation change: explicitly version it and test old/new reads. Preserve existing large-file CDC transcripts/roots; use separate small-case oracles rather than altering old expected values. Tiny does not settle compatibility details.


## Signals to inspect before changing code

| Signal | Question it answers |
| --- | --- |
| Unique IDs versus candidate/reused occurrences | Is exact content being revisited across batches? |
| Locator probes, pack/group/base reads | Is SQL batching hiding an N+1 fetch/decode path? |
| Canonical hash/comparison bytes | Are already authenticated immutable operands processed repeatedly? |
| Requested, encoded-fetched and decoded bytes | How much read amplification does a small request cause? |
| Eligible targets, usable bases, attempts, selections and fallback reasons | Is the new encoding actually reached under real resource budgets? |
| Physical batch maxima, SQL transaction maxima and statement rows | Which separate batch limit is binding? |
| Producer blocked/consumer idle time and lock occupancy | Is parallelism helping or only queuing more work? |
| Initial allocation, per-Commit growth, pack/index/slack/base bytes | Are smaller frames reducing the complete Store? |
| Exec, edit, Commit, End and full-case wall | Did work disappear or merely move to another phase? |

No single counter proves speed. Concurrent producer/consumer durations cannot be
summed as serial wall time. Zero device reads with warm OS cache does not mean
zero pack fetches or decoding. Unavailable counters stay unavailable.

## Implementation obligations and later qualification

- Name the actual public save/edit route and the full-file reconstruction mapping.
- State the baseline product, packed format, physical page policy and migration
  boundary; use a separate version when an encoding changes reader semantics.
- Account for simultaneous base, target, delta, FULL alternative, queue and decode
  ownership; prove that intended delta reservations can succeed.
- Specify authenticated base lifetime and batched reconstruction without holding
  broad SQLite/live-state locks across codec work or waiting I/O.
- Preserve existing publication, retry, alias/mmap coherence and cleanup contracts.
- Use only the [ten-file smoke](delta-encoding-benchmarks.md#first-round-execution-scope)
  and its integrated checks for implementation verification. The old tiny case,
  boundary/scattered/failure suites and affected-family campaigns are later work.
- The new local smoke reports numerical comparisons without declaring numerical
  PASS thresholds. Freeze numerical gates before any later admission campaign's
  candidate collection; do not impose that campaign as a first-round prerequisite.

## Reading order

1. [Current ten-file smoke contract](delta-encoding-benchmarks.md#first-round-execution-scope)
   and [implementation plan](implementation_plan.md).
2. [Historical pitfalls](past_mistake.md), especially ineffective delta budgets,
   repeated comparison, physical-versus-SQL batching and read amplification.
3. [benchmark/AGENTS.md](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/AGENTS.md), [benchmark/fs-bench-pro/QUICKSTART.md](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/benchmark/fs-bench-pro/QUICKSTART.md), and
   [docs/general/benchmark_rules.md](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/general/benchmark_rules.md) for execution/admission policy.
4. [Released v0.1.4 report](benchmark-v0.1.4-report.md) for the implementation
   baseline and [original tiny contract](tiny-history-baseline-v1.md) for separate
   historical evidence; R26 is not the release control.

The tiny family has no admission issue and remains explicitly exploratory.
A completed performance execution, a valid correctness proof, a comparable
baseline, and an accepted regression gate are four separate facts.

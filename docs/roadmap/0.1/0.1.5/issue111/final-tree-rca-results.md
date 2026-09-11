# Final root/inode-tree RCA — 2026-09-11

The dominant measured cost is **metadata-pool preparation: 439.392 ms of
625.874 ms final-tree wall time (70.2%, medians of two nonce diagnostics)**.
Its scratch SQLite value-index synchronization and batched value lookup account
for 347.317 ms together. This is shared admission work that can also affect
Workspace Commit. It is not evidence that every Commit rebuilds the entire
namespace or pays the complete Init final-tree cost.

The owner requires 4 KiB pages. They remain 4 KiB; four producers, schema10,
pack/zstd/FULL/DELTA policy, authentication and bounded scratch storage remain
unchanged. This is diagnosis, not an optimization or acceptance result.

## Custody and protocol

Followed [the frozen protocol](final-tree-rca-contract.md), committed in
71a5c847b. Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-final-tree-rca-evidence/20260911T045909Z`.
The detached `source/` worktree is commit 463beacb3 plus the preserved
compaction-removal patch. Before instrumentation it reproduced product seal
`95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`.
Concurrent metadata-cache work in the main workspace is excluded and untouched.

Added aggregated clocks and a final-phase physical-counter snapshot only in the
isolated source. `diagnostic-source.patch`, `instrumented-files.txt`, identities,
before/after seals, commands, stdout/stderr, exit codes and `analyze.py` are
retained. The qualified `shared/runner.py --build-host` succeeded under the
measurement lock and passed its linked schema10/storage-format probes. Initial
build and collection lock refusals are retained; neither started a sample.
Then exactly two full diagnostics ran under one shared measurement lock, with
independent cold acquisition and fresh host Stores. No timing-based replacement,
warm-up, resource-sensitive overlap within that lock, or intrusive profile.

Instrumented identities:

- Product: `b75a712f56983879115e7a659864fdcede9c4c39fa9f4125e4fe03b96ddf1d56`
- Source: `966ff486ed8445e9e23ef03ee5d80e1c229929d48f44a099078d1451141f41e9`
- Binary SHA-256: `10cc6cdd597d4b9facc6067b28352132548702e93d6cf78bc57cc364c3c7d674`
- Fixture: `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`

Source/product/workload seals match the qualified identity before and after both
samples. Store, SDK, construction and publication ran on macOS; direct Init
diagnosis requires no Docker daemon or FUSE mount.

## Cold observations, not qualification

Both rows report `fixture_cache_profile=reused-first-sample-uncontrolled`.
More importantly, each independently passed the fixed cold acquisition:
100,000 files, 500,000,000 logical bytes, 125,169 checked pages and **zero
resident pages**, with the backend warm-page positive control. Source allocation
was 865,730,560 bytes. Labels and process read counts alone do not establish
cold eligibility. Launch-gap and acquisition checks passed in the collector.

| Nonce | Init ns | Pipeline ns | Final-tree ns | Import remainder ns | Outer remainder ns | initialization_disk_read_bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 111f1 | 3,745,887,292 | 3,044,375,625 | 605,929,750 | 58,360,625 | 37,221,292 | 876,138,496 |
| 111f2 | 3,612,191,416 | 2,845,911,417 | 645,817,333 | 72,476,750 | 47,985,916 | 865,783,808 |

n=2; median Init 3.679039354 s, range 3.612191416–3.745887292 s.
Median final-tree 625.873542 ms, range 605.929750–645.817333 ms.
Both are **nonce diagnostics, qualification_eligible=false**. No target pass,
paired speedup or candidate improvement is calculated. Cold plain measurements
remain the absolute gate; warm data remain paired-delta-only. The 2.7 s target
is still open. The original plain 3.435659584 s and historical plain
2.776088125 s had no phase clocks; these rows cannot retrospectively split them.

## Exclusive final-phase accounting

The new physical snapshot encloses exactly the existing final-tree timer.
It ends before `admission.finish()` and final publication. Existing commit
diagnostics identify the two authoritative transactions committed during final
construction. Transaction COMMIT clocks are not INSERT clocks or CPU clocks.

| Component | Sample 1 ms | Sample 2 ms | Median ms |
| --- | ---: | ---: | ---: |
| Metadata-pool preparation | 419.228 | 459.555 | 439.392 |
| Authoritative admission INSERT | 9.408 | 11.189 | 10.299 |
| Authoritative transaction COMMIT | 17.918 | 9.385 | 13.652 |
| Other final work, by subtraction | 159.375 | 165.688 | 162.531 |
| **Final-tree wall** | **605.930** | **645.817** | **625.874** |

The residual includes tree assembly, pair-stream consumption, canonical
encoding/hashing, generic admission preparation and bookkeeping. It is not a
measured pure tree-algorithm or pure CPU interval. Its internal split remains
unresolved; this study does not claim the full regression has been eliminated
or causally isolated by an ablation.

The following intervals are **nested inside** metadata preparation; do not add
them again to the table above:

| Metadata preparation component | Median ms |
| --- | ---: |
| ValueIndex synchronization | 215.984 |
| Build lookup set | 21.699 |
| Batched value lookup | 131.333 |
| Assign ordinals and rewrite physical leaves | 26.237 |
| Build canonical metadata values and encode their packs | 25.081 |
| Decode encoded groups and calculate authentication digest | 16.442 |
| Preparation remainder | 2.616 |
| **Metadata preparation total** | **439.392** |

Within the 215.984 ms synchronization interval, catalogue work is 2.699 ms,
group read/authentication/decode 18.805 ms, scratch-index INSERT loops
169.402 ms, scratch transaction COMMIT 24.834 ms, and remainder 0.244 ms.
These are wall intervals around the named operations; SQL insertion cost is
not further separated into CPU, cache misses and scratch-file I/O.

## Mechanism and historical comparison

`prepare_values` obtains the Store-scoped `ValueIndex`, synchronizes previously
admitted metadata values into its disposable SQLite index, constructs lookup
values, calls `find_batch`, assigns stable physical ordinals, then encodes and
authenticates metadata groups. The scratch table uses the complete 73-byte value
as its primary key. `sync` executes an INSERT OR IGNORE for each recovered value;
`find_batch` executes batched IN lookups against that index.

Final construction called this preparation 74 times in each run. It synchronized
86,976 / 86,082 existing values from 572 / 566 groups, then looked up
93,270 / 92,381 values with only 63 / 65 hits. This explains substantial serial
index-maintenance and mostly unsuccessful lookup work in this unique fixture.
It does not establish that the same hit rate applies to redundant workloads.

The phase admitted 93,207 / 92,316 values; whole Init admitted 100,002 in both
runs. Small differences in phase counts reflect batching across timer boundaries,
not different canonical output. Whole-Init metadata preparation was
493.562 / 552.724 ms. Do not assign its full cost to final construction.

The previous [matched-cold diagnostic study](version-breakdown-results.md)
measured final-tree medians of 141.947 ms for v0.1.3 and 613.791 ms for the
preceding v0.1.5 diagnostic product. The historical schema5 path did not use schema10
pooled metadata/value indexing. The additional isolated clocks show 439.392 ms
in that schema10 path, on the scale of the observed 471.844 ms increase.
This is measured attribution plus a source-path explanation, not a controlled
claim that all 439 ms can be removed or that every other format difference is
irrelevant. No page-size comparison is proposed.

Metadata group reread/decode is only 18.805 ms here; removing that alone cannot
recover hundreds of milliseconds. Encoding the metadata packs takes 25.081 ms.
The earlier physical census found no selected small-content DELTAs in this
fixture. The evidence therefore prioritizes **scratch value-index insertion
and lookup**, rather than disabling compression or changing the 4 KiB format.

## Implication for Workspace Commit

Commit persists both changed content and the new namespace version referencing
it. It is more than a final SQLite COMMIT statement:

1. `Workspace::commit` builds a candidate from recorded mutations. Streaming
   content admission can occur during candidate construction.
2. `changes.rs` applies sorted inode deltas to the existing inode-table root via
   `inode_table_apply_sorted_with_budget`, reusing unchanged subtrees and encoding
   the resulting namespace root. This is incremental construction.
3. `commit_workspace_candidate` admits remaining objects and conditionally
   publishes the commit/branch update.

Both paths reach shared metadata preparation/admission. Consequently index
lookup, encoding and admission regressions can slow changed-workspace Commit.
But Init constructs the initial complete namespace; a one-file commit should
not be assigned the whole 626 ms Init final-tree cost. An unmodified workspace
has a generation-zero fast path using the existing root without candidate-tree
construction.

There is a separate first-use risk: opening a Store initializes `metadata_index`
to None; the first metadata preparation creates `ValueIndex` with next ordinal 1
and synchronizes existing metadata groups. A small first commit after reopen can
therefore pay index synchronization proportional to preexisting metadata. On a
retained Store, the cursor normally advances only over newly admitted groups;
rollback invalidation can reset the derived index. This behavior is established
by code inspection, **not a measured Commit latency regression in this study**.

The next shared-path experiment should reduce measured scratch index/lookup
cost while preserving exact equality, ordinal order, authentication, rollback
handling and current scratch limits. It must include a small commit after
reopen, repeated small commits on a retained Store, a larger changed set and
the unchanged fast path. Changing only a metadata read cache does not directly
remove the measured scratch INSERT or find_batch costs. No speculative product
change was made in this RCA.

## Validation and remaining work

`analyze.py` passed: full fixture counts; 112,451 canonical objects and
513,026,835 canonical bytes in both Stores; zero object reuse; all phase sums
and nested remainders nonnegative and reconciled; group counters consistent;
4 KiB/schema10/four producers; matching frozen seals. Host build probes passed.
The operation remained inside its original timer. Diagnostic-only clocks do
not replace the preceding independent semantic proofs, and no new candidate
qualification is claimed.

Post-Init apparent Store bytes were 515,399,680 / 515,420,160; allocated bytes
519,385,088 / 527,048,704; logical content was 500,000,000. Allocation is an
observation of these fresh diagnostic Stores, not a compression ratio or new
per-tier qualification. Existing allocation/format findings remain in the
version study. Peak process RSS was 83,591,168 / 84,426,752 bytes, no swaps,
and process thread count returned to one after four-producer initialization.

The shared metadata bottleneck is now measured. Its achievable speedup,
incremental Commit impact, residual 162.531 ms final work and the cold pipeline
gap remain open. Fixing final-tree work alone is not proof of reaching 2.7 s.
This RCA updates #111 and follows #109/#110, with shared-caller qualification
context in #108/#106/#102/#104/#100/#107.

Issue update: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5630041256

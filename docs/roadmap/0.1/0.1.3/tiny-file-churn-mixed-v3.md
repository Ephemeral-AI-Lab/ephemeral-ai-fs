# Tiny-file churn: mixed bulk profile v3

Approved workload amendment, 2026-09-06. Tracking: [#46](https://github.com/Ephemeral-AI-Lab/layerfs/issues/46), Commit/integration [#47](https://github.com/Ephemeral-AI-Lab/layerfs/issues/47), Exec [#48](https://github.com/Ephemeral-AI-Lab/layerfs/issues/48), parent [#39](https://github.com/Ephemeral-AI-Lab/layerfs/issues/39).

**Status: documentation/issue contract updated; benchmark registration, generator and verifier migration are not implemented by this amendment.** Do not run the proposed IDs until they appear in the migrated registry. Existing raw results remain immutable historical evidence.

## Scope and exact distribution

Replace the tier-100 and tier-500 **bulk create/delete target trees** with the profile below. Family ID remains `tiny_file_churn`. Counts refer to affected regular files; directories and the separate untouched witness are excluded. Tier payloads use MiB (1,048,576 bytes), not decimal MB.

| Group | Tier 100 | Tier 500 |
|---|---:|---:|
| Large files | 1 × 50 MiB = 52,428,800 bytes | 3 × 100 MiB = 314,572,800 bytes |
| Small files | 800 × 4 KiB = 3,276,800 bytes | 4,000 × 4 KiB = 16,384,000 bytes |
| Medium files | 199 files, 49,152,000 bytes combined | 997 files, 193,331,200 bytes combined |
| **Affected regular files** | **1,000** | **5,000** |
| **Affected payload** | **104,857,600 bytes / 100 MiB** | **524,288,000 bytes / 500 MiB** |

Exact medium sizes: tier 100 has 194 files of 246,995 bytes and five of 246,994 bytes; tier 500 has 936 files of 193,913 bytes and 61 of 193,912 bytes. Equivalently divide the remaining medium bytes by the medium count and assign one extra byte to the first remainder-count ordinals.

The existing untouched witness remains 200 regular files / 1 MiB under `witness`. Thus the complete populated regular-file namespace has 1,200 files / 101 MiB at tier 100 and 5,200 files / 501 MiB at tier 500, excluding directories. Do not accidentally include the witness in the requested affected counts or resize it. Bulk create starts with only the witness; bulk delete starts with witness plus target and ends with witness only.

## Identity and deterministic layout

New active IDs after implementation:

- `tiny-bulk-create-100-mixed-v3`
- `tiny-bulk-delete-100-mixed-v3`
- `tiny-bulk-create-500-mixed-v3`
- `tiny-bulk-delete-500-mixed-v3`

Fixture profile: `tiny-bulk-mixed-v3`. These replace the four corresponding unversioned bulk IDs in active membership; family cardinality stays 20. Low-tier `-compact-v2` cases and individual `tiny-create`, `tiny-stat`, `tiny-unlink` cases retain their existing definitions. The previous 20,000/100,000 affected-file bulk records are archived workload evidence, not measurements of v3.

Reuse the existing bulk path shape for five 200-file path shards at tier 100 and 25 at tier 500, rooted under `bulk`, including its wide/deep directory layout and destination directory. Only target content lengths change; do not change the shared shard generator used by unrelated families. Use `(shard ordinal, file ordinal)` ascending order to assign file ordinal 0..count-1. First assign the one/three large files, then the 800/4,000 small files, then medium files in that order. Derive bytes through the existing bounded generator with the new profile, seed, and path/ordinal as domain-separated inputs. Both create and delete preparation/oracles must use identical mapping for a given seed.

Seeds remain 1..3; initial iteration uses seed 1 only. Exact file lists, sizes, digests, directory counts and metadata enter the new input identity. Do not require tier 100 to be a byte-identical prefix of tier 500: the new distributions intentionally differ. Different file count, large-file count and sizes invalidate a simple linear-scaling expectation.

## Execution, verification and migration

Continue ordinary POSIX/FUSE create/write/close, traversal/unlink/rmdir, metadata normalization and required sync through managed Exec, followed by Commit, visibility and End. Large files use bounded writes, not a 50/100 MiB allocation or a special product route. Preserve one publication and witness semantics. The benchmark describes inputs; LayerFS product code must not branch on profile/case IDs, file-size buckets or workload density.

All prior topology, byte limits, alias/open-unlinked behavior, cleanup and source-custody obligations remain. macOS owns Workspace/spools/construction/SQLite; Docker runs Linux daemon/workload/FUSE, 2 CPUs / 2 GiB / no swap / 256 PIDs, no data mounts. Host resources are reported separately. No Docker SQLite mode.

Migration must change registration, bulk fixture/expected/apply paths, sampled verification (including explicit coverage of every large file), case classifiers/assessment, setup/cache identity and relevant command examples together. Keep independent sampled proof bounded: sample ranges from each large file plus declared medium/small paths; do not read every large-file byte in the proof iteration. Report exact coverage/omissions. Preserve the current final-stage-only 45-second work / 59-second hard proof contract.

Before collecting v3 performance, commit this specification/implementation under the existing issues and verify exact file/byte totals for both tiers, create/delete fixture equivalence, witness retention, deterministic sizes/content and registry cardinality. Use focused fixture checks; no full benchmark run is required for a documentation change. Do not reuse old prepared target trees, or relabel, overwrite or pool old perf/proof receipts. Unchanged witness preparation may only be reused through the existing compatibility checks.

## Targets and interpretation

The user-authorized workload revision supersedes older instructions demanding the original 20,000/100,000 affected files for these four rows. It does not authorize claiming a product speedup from reduced work.

- #47's two-case complete-lifecycle target remains strictly below 1,000 ms, now for the two tier-100 `-mixed-v3` IDs once implemented. Historical-ID diagnostic runs cannot qualify that new contract.
- #48 Exec work supports the same revised IDs and combined product; separate branch/phase results do not establish combined PASS.
- The aggressive Commit objective is approximately 300 ms for each revised tier-100 bulk operation. It remains exploratory, not an additional hard phase gate or promised result. No new tier-500 300 ms target is introduced.
- #46 remains the broader 20-case family task with its current 15-second complete-product classifier and final-proof amendment. Tier 500 remains later work; no immediate all-family/scaling campaign is required.

File-operation/metadata costs should fall with count, while content work still processes 100/500 MiB. The 50 MiB file dominates half of tier-100 bytes; the three 100 MiB files dominate 60% of tier-500 bytes. One producer is an initial implementation choice, not a benchmark requirement. Add bounded parallelism only when producer/consumer metrics and complete timings justify it, through the shared init/Commit pipeline.

## Integration authorization clarification

The migration authorization retains the practical Commit aim around 400 ms with approximately 50 ms tolerance; this supersedes the exploratory 300 ms sentence above for this integration campaign. Complete-lifecycle qualification remains strictly below 1,000 ms for both revised tier-100 IDs. Implement and validate both tiers, then collect one selected tier-100 create and delete sample on the same source, serially. Tier-500 performance, #49, and independent proofs remain deferred. Original create 636.195 ms and retained delete 304.405 ms Commit results remain historical; workload migration does not establish a product speedup.

## Implemented migration

Registration, fixture/expected/apply recipes, sampled range verification, registry profiles, fixture/input/cache custody and the separate strict tier100 assessment are migrated. The populated-tree manifest includes every path, file length/digest and metadata; its SHA-256 joins the input identity. Untimed oracle identities are cached by exact host binary/family/case/seed, checked before reuse, and required by later bounded proofs so proof selection never generates a full large-file oracle. Prepared Stores use the new fixture profile and manifest-bound plan; historical target trees cannot match. Product code is unchanged.

Focused checks cover all three recipe seeds and both tiers, exact ordinal assignment and distributions, shared path shape, create/delete equivalence, witness retention, registry20, every large-file range/absence, and non-prefix corruption rejection through both canonical and native sampled readers. Runner tests cover the strict1,000,000,000ns boundary, old/tier500 exclusion, preparation invalidation, cached-oracle corruption and bounded-proof reuse. No tier500 performance or independent benchmark proof is run by these checks. Revised tier100 receipts are recorded separately after the migration commit.

## Tier500 full-run authorization

The user now requests an increased limit to obtain a full tier500 run, superseding the earlier tier500-performance deferral. Use one selected seed1 sample for each tier500 mixed-v3 bulk operation, serially. Increase the diagnostic product allowance to600seconds, outer command allowance to630seconds, and selected preparation allowance to600seconds. These are execution allowances, not relaxed PASS criteria: retain the family15second complete-product target, with no new tier500 subsecond or Commit-phase gate. Resource caps, input distribution, host SQLite topology and proof deadlines are unchanged. Implement an explicit benchmark diagnostic timeout option (default120seconds) rather than hard-coding product behavior by tier. Preserve the already-recorded tier100 results; no unchanged tier100 reruns are required for this harness allowance change.

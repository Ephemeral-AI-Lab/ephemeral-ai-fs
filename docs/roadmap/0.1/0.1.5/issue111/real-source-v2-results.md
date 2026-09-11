# Refreshed real LayerFS source Init results (#111)

**Actual source: 5,005,457 logical bytes → 1,572,864–1,576,960 allocated Store bytes, saving68.50–68.58%. Median cold public Init time51,611,208ns (51.611ms); range51,379,542–60,197,958ns, n3. All three independently reopened Stores fully verified all387 files /5,005,457 bytes.**

This rerun uses the current source snapshot selected by the owner, not either synthetic generator mode. Scope matches the prior real-source run: tracked crates/, tools/, benchmark/, containers/, .github/, declared root build files/README/license. Excludes .git, generated/untracked outputs, docs and release-notes archives. Includes original dirty working-tree edits; none were modified. Snapshot source commit2e72fc7d72c7a12e7c5796dbb20db7010456c359;387 files,382 small and5 large. New manifest/case version reflects the changed snapshot. This is one real source-code corpus, not general production or100k/500MB scaling qualification.

[Prospective contract](real-source-v2-contract.md) committed before timings as c65309a49. Manifest SHA256:`7ec68f2c10df421f4df6d1136bae643574ec34ee9bbc57b9cf67a543e346aca2`. All input hashes rechecked unchanged after execution.

## Measurements

Each independent fresh Store was created on macOS. Timer starts immediately before public Client::initialize_layerstack(Directory(input)) and stops on acknowledgement; preparation, physical counters/census, closure and verification stay outside. Product seal unchanged. No pseudorandom/structured-text generator, compaction, repack, VACUUM, GC or product optimization.

| Sample | layerstack_init_ns | initialization_disk_read_bytes | Allocated B | Apparent B | Full-file proof |
|---|---:|---:|---:|---:|---|
| 1 | 60,197,958 | 6,045,696 | 1,572,864 | 1,572,864 | PASS |
| 2 | 51,379,542 | 6,049,792 | 1,576,960 | 1,576,960 | PASS |
| 3 | 51,611,208 | 6,045,696 | 1,572,864 | 1,572,864 | PASS |

Cache profile:`reused-first-sample-uncontrolled` plus recorded explicit fsync/shared-mmap/touch/MS_SYNC|MS_INVALIDATE/unmap/close acquisition before every sample. Input file allocation6,041,600B; observed reads6,045,696–6,049,792B validate cold input read volume. All three belong to this acquisition profile; no warm/cold pooling. No timing retries/outlier deletion. Host peak RSS27,230,208–29,032,448B; swaps0. This native Init/reader proof does not create a container or claim FUSE verification. Host build requalified under runner lock (native build0.22s); no resource-sensitive overlap.

Canonical output:2,456 objects /5,246,495 bytes in every run. Allocated median1,572,864B; about3.18× smaller than logical input. The4KiB allocation difference in sample2 reflects physical layout; no content difference. Relative to the prior385-file snapshot, these are consistent observations of roughly50–60ms and68.6% storage savings, not a paired speedup/regression claim because the source snapshot changed.

## Compression and encoding

Read-only census of all three measured Stores confirms compact small packs (version4), native large-content packs (version2), ordinary and pooled metadata packs.382 small source files become380 unique small records:4,052,731 unique raw B encoded into994,515–1,002,111B of Zstandard FULL/DELTA frames. Small DELTA records by sample:104,89,79; small FULL:276,291,301. Five large source files use normal raw-size-based CDC/CAS dispatch. Total physical pack blobs1,344,836–1,351,506B. SQLite/index/page overhead is included in the final allocated Store size.

These are combined compression/DELTA/deduplication/packing savings, not an isolated pure-compression benchmark. Exact duplicate small content saves only278 raw bytes; the major savings here come from compressible source and encoded similarity. Different FULL/DELTA choices can arise from parallel admission order. This run measures latency with all normal features enabled, not the incremental cost of turning compression on or off.

## Verification and custody

Three fresh-process full-file proofs PASS: persisted layer/root identity, complete stored file-path enumeration, streamed byte counts and SHA256 of every file against the independent Python hashlib oracle. Each verifies387 files /5,005,457B. Verification-only time85.424–87.080ms; not included in Init. No directory metadata/xattr/crash/concurrency/FUSE/history qualification is implied.

- Product seal:`95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`.
- Source/harness seal:`00f158e9e0a807e5660018301ad8251c7ee67b82e1f494f981fcfa8c50992a86`.
- Host binary SHA256:`286860778261ea46149add333a46186b06b98f509b8de40622ddd4d1925ff27e`.
- Evidence:`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-real-source-evidence/run-20260911T010317Z`.

Raw performance/verification stdout/stderr/command/exit records, cache receipts, source manifest/oracle, all three measured Stores, physical censuses, qualified host identity, collector and summary.json are retained. manifest.json seals final artifacts. Original namespace100000 cold2.7s target remains open and is not evaluated by this smaller real-data case. No release/tag/deployment.

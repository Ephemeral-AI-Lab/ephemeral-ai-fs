# Repeated LayerFS source scaling results (#111)

**All nine Init runs and all nine independent full-content verifiers passed.** This is the owner-requested synthetic repetition test at1,000/10,000/100,000 files, not a real unique-data production workload. Product code/settings remained unchanged.

## What was repeated

Use the sealed387-file LayerFS source snapshot from real-source-v2. Cycle its sorted file list; destination paths are copy-000000/original-path, copy-000001/original-path, and so on, truncating the last copy to the exact target count. Preserve file bytes, modes/mtimes and corresponding directory metadata. Every destination is written as an independent ordinary file: no hardlinks, clonefile, sparse padding or generated replacement content. Output counts and complete bytes were independently verified.

Every tier contains the same **385 distinct file contents /5,005,179 unique logical bytes**. The repeated logical totals below are real materialized inputs but deliberately duplicate that content. Claim kind: structural-complexity/synthetic deduplication evidence. Do not present the logical totals as unique-data throughput or general production-size qualification, and do not calculate headline logical-MB/s.

[Prospective contract](repeated-source-contract.md), committed9867371af before measurement, freezes n3 per tier, public operation, copy order, cache acquisition, full proofs, budgets and retention. No valid attempt rerun or discarded.

## Observations

Each Init uses a fresh independent empty Store and exactly one public Client::initialize_layerstack Directory call on macOS. The operation timer includes intrinsic reads/hashing, construction/admission, compression/deduplication and namespace publication. Fixture construction/cache acquisition, oracle parsing, physical observations, closure and verification are outside it.

Individual times are shown to avoid pooling differing observed cache categories. Storage savings use repeated logical input bytes as the denominator and include SQLite and filesystem allocation overhead; they are combined deduplication/compression/packing savings, not a pure codec ratio.

| Files | Directories excluding root | Repeated logical input B | Init times, runs1/2/3, ms | Allocated Store range B | Effective storage saved |
|---|---:|---:|---|---:|---:|
| 1,000 | 237 | 12,584,955 | 96.082 / 93.652 / 94.375 | 1,593,344–1,601,536 | 87.274–87.339% |
| 10,000 | 2,322 | 129,176,473 | 647.634 / 691.482 / 661.028 | 1,970,176–1,986,560 | 98.462–98.475% |
| 100,000 | 23,262 | 1,292,971,697 | 6107.051 / 5833.996 / 6121.596 | 6,422,528–6,426,624 | 99.503–99.503% |

Allocated medians:1,597,440 /1,978,368 /6,426,624B respectively. Canonical objects/bytes remain constant across repeats of each tier:2,619/5,332,119B;4,929/6,577,808B;28,123/19,046,463B. Physical packing/layout can vary without content differences. The larger namespace adds directory/inode/index metadata even when file contents repeat.

## Cache and resource accounting

All runs used the same prospectively declared fsync/shared-mmap/touch/MS_SYNC|MS_INVALIDATE/unmap/close procedure. The wrapper profile is reused-first-sample-uncontrolled. Counter-based classification is deliberately conservative: cold-read-volume means process read bytes at least the source files' allocated bytes; otherwise a nonzero read count is mixed. This is an observation of read volume, not proof of every device-cache level. Some input pages remained cached. Do not relabel all nine samples cold.

| Files | Run | initialization_disk_read_bytes | Input allocated B | Observed category | User CPU ns | System CPU ns | Host peak RSS B | Store apparent B |
|---|---:|---:|---:|---|---:|---:|---:|---:|
| 1,000 | 1 | 13,144,064 | 15,253,504 | mixed | 123,107,334 | 70,246,250 | 32,243,712 | 1,601,536 |
| 1,000 | 2 | 15,007,744 | 15,253,504 | mixed | 121,954,584 | 58,875,667 | 29,147,136 | 1,597,440 |
| 1,000 | 3 | 14,876,672 | 15,253,504 | mixed | 121,789,041 | 59,361,709 | 30,277,632 | 1,593,344 |
| 10,000 | 1 | 155,967,488 | 155,967,488 | cold-read-volume | 1,156,774,500 | 518,659,417 | 35,274,752 | 1,970,176 |
| 10,000 | 2 | 153,964,544 | 155,967,488 | mixed | 1,181,375,750 | 525,243,417 | 38,060,032 | 1,978,368 |
| 10,000 | 3 | 153,632,768 | 155,967,488 | mixed | 1,162,017,458 | 503,758,250 | 35,684,352 | 1,986,560 |
| 100,000 | 1 | 1,565,642,752 | 1,560,727,552 | cold-read-volume | 11,101,614,583 | 5,311,307,250 | 81,772,544 | 5,685,248 |
| 100,000 | 2 | 1,565,741,056 | 1,560,727,552 | cold-read-volume | 10,407,269,042 | 5,420,057,125 | 82,247,680 | 5,599,232 |
| 100,000 | 3 | 1,543,458,816 | 1,560,727,552 | mixed | 11,359,716,167 | 5,252,732,459 | 79,904,768 | 5,783,552 |

Within-category timing summaries (no cold/mixed pooling):

| Files | Category | n | Median ns | Min–max ns |
|---|---|---:|---:|---|
| 1,000 | mixed | 3 | 94374542.0 | 93651625–96082208 |
| 10,000 | cold-read-volume | 1 | 647633708.0 | 647633708–647633708 |
| 10,000 | mixed | 2 | 676254937.5 | 661027750–691482125 |
| 100,000 | cold-read-volume | 2 | 5970523687.5 | 5833996125–6107051250 |
| 100,000 | mixed | 1 | 6121596333.0 | 6121596333–6121596333 |

Host swaps0 in all samples. Init creates no live Workspace or runtime; no Docker/FUSE claim is made. The full proof uses the authenticated canonical reader. Builds, preparation, performance, proof and census are serialized under the runner measurement lock, with no child double acquisition. No other user's processes were stopped. All source/product seals checked unchanged at completion.

## Evidence of deduplication and its limits

A read-only physical census of the first Store at each tier finds **380 unique small-content records representing4,052,731 raw bytes at every tier**. Their compressed FULL/DELTA frames occupy1,001,608 /1,000,190 /1,000,816B respectively—approximately1MB, despite increasing file counts. This directly demonstrates content sharing and normal compression/DELTA encoding rather than inferring deduplication from Store length alone. Five unique large source files use the unchanged CDC/CAS path. Complete source manifest has two exact duplicate small files, hence382 small source files→380 unique small records.

Small FULL/DELTA counts in those three inspected Stores:301/79,295/85,305/75. Pack payload totals1,367,039 /1,613,799 /4,083,143B; remaining Store space covers SQLite/index/page/filesystem overhead. Compact small packv4, native packv2, ordinary and pooled metadata packs remain active. The operation's reused_objects=0 counts preexisting Store reuse in a fresh Store; it does not mean repeated input content was stored repeatedly.

**Deduplication keeps stored payload small, but does not remove the need to open/read/hash each input file and build its namespace entries.** This explains why measured Init time grows with repetition while stored payload stays near the unique-content size. The experiment does not isolate the incremental cost of enabling/disabling deduplication or compression. Nor can these observations be directly compared with the original random namespace tiers: this100k case has1,292,971,697 logical bytes and23,262 directories, whereas the original100k case has500,000,000 bytes and1,000 directories.

## Full verification

All three Stores per tier were closed and reopened in independent processes. Every file path was enumerated, its complete bytes streamed and SHA256-compared with the independently generated frozen oracle. Verified per Store:1,000 files/12,584,955B;10,000 files/129,176,473B;100,000 files/1,292,971,697B. No file verification entered performance timing. Largest-tier proof durations34.521/36.119/36.116s, below the prospectively declared180s extended proof limit. This does not claim crash/concurrency/xattr/directory-metadata/FUSE qualification.

## Custody and reproduction

- Source snapshot manifest:7ec68f2c10df421f4df6d1136bae643574ec34ee9bbc57b9cf67a543e346aca2.
- Product seal:`95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`.
- Source/harness seal:`00f158e9e0a807e5660018301ad8251c7ee67b82e1f494f981fcfa8c50992a86`.
- Host binary SHA256:`286860778261ea46149add333a46186b06b98f509b8de40622ddd4d1925ff27e`.
- Qualified native host rebuild:PASS,0.11s native build; schema10/ordinary-format probes PASS.
- Evidence:`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-repeated-source-evidence/20260911T011119Z`.

plan.json and plan-N.json freeze exact source-to-destination mappings; each tier's fixture.json binds allocation/oracle hashes/preparation time. collect.py/run.py/cache.py record the procedure, collection.log streams all commands/results/progress, ledger.jsonl retains every completed operation/proof, and per-run stdout/stderr/command/exit/cache receipts plus all nine Stores remain. summary.json and per-tier pack census support the tables; manifest.json seals artifacts. Existing uncommitted work—including compaction removal and the unrelated issue112 folder—was not modified or committed. No release, tag, deployment, compaction, VACUUM/repack/GC or product optimization. Original #111 cold target remains open.

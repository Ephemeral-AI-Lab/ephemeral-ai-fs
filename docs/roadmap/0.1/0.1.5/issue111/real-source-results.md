# Real LayerFS source Init results (#111)

**Actual source content: 4,989,091 B → 1,564,672 B allocated Store, a 68.638% reduction including SQLite overhead.** Three independent cold Init samples and all three fresh-process full-content verifiers passed. Product compression was already enabled; changing the input from pseudorandom data to the owner's selected real source corpus exposes its practical benefit.

This is one real repository snapshot, not a general production-readiness claim, a history/Commit benchmark, or a replacement for namespace-100000. The original cold <=2.7s target remains open. No product optimization, compaction, repack, VACUUM or GC was used. The rejected advisory-prefetch experiment was removed before this source snapshot/build.

## Frozen workload and boundaries

See [prospective contract](real-source-contract.md), committed as ae9ad2118 before harness implementation/timing and announced on #111. Source commit 6d42d0e3b plus preserved original dirty working tree. 385 existing tracked files from crates/, tools/, benchmark/, containers/, .github/, and the declared root build/README/license files. Includes current user compaction-removal edits. Excludes .git, binaries/caches/untracked files, docs and release-notes. Historical benchmark reports account for most of the excluded checkout bytes; no compressibility-based selection or repetition/padding was used.

- Total input:385 files /4,989,091 logical bytes /6,025,216 allocated filesystem bytes.
- Small content:380 files /4,036,780 bytes; five large files /952,311 bytes; no empty files.
-383 distinct file SHA256 values. Exact duplicate small-file contents account for only 278 logical bytes.
- Input manifest SHA256:`c6534deb6a187a3a7da09a062684f6658a78575958b55b6d676e86b60cc0f2c6`. Every source file hash rechecked after execution.

Each sample creates a separate empty Store on macOS and times exactly public `Client::initialize_layerstack(Directory(input))` through return. The normal product performs scanning, CAS hashing, encoding/admission, metadata construction and publication inside that timer. Oracle parsing, cache acquisition, counters/census, Store closure and verification are outside it. No runtime/Workspace/FUSE is created by Init; Docker is not involved in this operation or its canonical-reader proof. The unused existing compatible Linux image is not claimed as an executed proof.

Cold acquisition uses read-only shared mmap/touch/MS_SYNC|MS_INVALIDATE/unmap on every input file before each invocation; no descriptors/mappings remain and no input bytes are changed. Every sample has `fixture_cache_profile=reused-first-sample-uncontrolled` in its cache receipt and6,025,216 process read bytes—exactly the source files' allocation. This is explicit cache invalidation, not an assumed first-use label. No warm samples or paired comparator; no speedup claim.

## Plain cold samples

| Sample | layerstack_init_ns | initialization_disk_read_bytes | Store allocated B | Store apparent B | Process peak RSS B | Verification |
|---|---:|---:|---:|---:|---:|---|
| 1 | 55,388,583 | 6,025,216 | 1,564,672 | 1,564,672 | 27,869,184 | PASS |
| 2 | 53,343,583 | 6,025,216 | 1,564,672 | 1,564,672 | 26,476,544 | PASS |
| 3 | 50,492,166 | 6,025,216 | 1,564,672 | 1,564,672 | 27,082,752 | PASS |

n=3; median **53,343,583 ns (53.344ms)**; range 50,492,166–55,388,583ns. All samples have2,445 canonical objects /5,229,065 canonical bytes. Process swaps=0; user CPU54.423–57.950ms, system CPU24.890–28.419ms, peak RSS26.477–27.869MB. These are host process measurements, not container limits. All results retained; no timing retries/outlier removal.

## Why the real source shrinks

Read-only census of the exact retained Stores confirms schema10 compact small packs (on-disk pack version4, the compact framing successor of small-content packv3), native content packv2, ordinary packs and pooled metadata packv6. All378 unique small-content records contain Zstandard frames; FULL/DELTA choices differ slightly with parallel admission order.

| Sample | Small FULL records | Small DELTA records | Small Zstandard FULL/DELTA frame bytes | All physical pack bytes |
|---|---:|---:|---:|---:|
| 1 | 273 | 105 | 986,607 | 1,335,622 |
| 2 | 266 | 112 | 986,302 | 1,335,856 |
| 3 | 301 | 77 | 995,799 | 1,344,143 |

The380 input small files become 378 unique small records containing 4,036,502 raw bytes. Their compressed FULL/DELTA frames occupy986,302–995,799B: approximately75.3–75.6% less than the unique raw content, before small record/pack framing. This is the combined Zstandard/FULL/DELTA representation benefit, not a separately isolated compression-only versus delta-only saving. The two exact duplicate files contribute only 278B of raw deduplication savings. The much larger reduction comes from compressible source text and similarity exploited by the encoded records.

For sample1:4,989,091 logical input B →5,229,065 canonical B (content framing and namespace/metadata) →1,335,622 physical pack B →1,564,672 SQLite/apparent/allocated B. SQLite tables/indexes/page overhead beyond pack blobs is229,050B. Canonical bytes are logical identity bytes, not disk usage. The source filesystem itself allocates6,025,216B; do not confuse that separate baseline with the4,989,091B logical-content denominator used for 68.638% savings.

The earlier pseudorandom fixture's300,000,000B of small content became301,310,582B of frames with 98,998 FULL /0 DELTA records. Those figures explain its lack of shrinkage; they are a different corpus/size, not a paired performance comparison. No compression setting or product data path was changed for this real-source run.

`reused_objects=0` in the Init operation receipt means no preexisting Store object was reused; it must not be interpreted as proof of no within-input deduplication. The frozen SHA256 manifest plus physical-record census demonstrates the two duplicate source files independently.

## Independent verification and limits

All three measured Stores were closed, reopened in independent processes, their persisted layer/root identities checked, their complete stored namespace enumerated, and every file streamed through the authenticated reader and SHA256-compared with the independently generated Python hashlib oracle. Each proof verified 385 files /4,989,091 bytes; verification-only time83.331–84.205ms. No file-content verification entered performance timing. A negative proof using an intentionally wrong expected SHA256 correctly failed on `.dockerignore`. The original oracle and measured Stores were not changed.

This proves persisted contents and exact file-path coverage for this snapshot. It does not claim live FUSE, concurrency, crash-recovery, Commit/history growth, directory metadata/xattr fidelity, or100,000-file/500MB scaling qualification. Existing product metadata/admission tests and the #104/#110 family receipts remain at their own recorded scope.

## Reproduction and custody

Harness commit 37f752f38 adds only a generic `repository-init ROOT INPUT performance|verification ORACLE.tsv` entrypoint and streaming verification. The product seal is unchanged from #109/#110. The original dirty files remain byte-identical, except main.rs contains the separately committed4-line dispatcher addition; subtracting it reproduces the exact original dirty-file hash. No compaction-removal work was staged/committed/reverted.

- Qualified host binary SHA256:`51fb9e01e8990746a1ba1624e46e64446bc84a6451c35c851a3fbded21392f72`.
- Product seal:`95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`.
- Source/harness seal:`216f31304407442e18b6f32672099ac1a2590f245681834290deccb9cb581133`.
- Native compilation seal:`4019c7e9678c6642f8357f8bb28b777bf9d23603d6ca35eb4a72531143650975`.
- Qualified runner build:PASS; linked schema10 and ordinary integrated-format probes PASS. Source code was frozen from build through all samples/proofs.
- Evidence root:`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-real-source-evidence/run-20260911T001109Z`.

`collect.py` contains the exact lock-owned sequential acquisition/performance/verification procedure. It intentionally refuses existing sample/log paths; reuse the sealed input/oracle with fresh output paths for any new campaign. `performance-*.command.json`, raw stdout/stderr, exit records and cache receipts preserve every command. `packs-*.json`, `negative-proof.json`, `summary.json`, input manifest/oracle and `measured-host-identity.json` support the report. Store bytes remain available for inspection. `manifest.json` seals the final evidence files.

The existing namespace-100000 target stays cold-absolute; warm observations remain delta-only. Related work: #109/#110/#108/#106/#102/#104/#100/#107. No release/tag/deployment.

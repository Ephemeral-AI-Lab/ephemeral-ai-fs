# P initialization provenance and public-read scope

**The three native decodes in the8MiB public read are explained by a mixed-format checkpoint, not by failure to account for a deep PREFIX chain.** The selected SDK checkpoint has three4096-byte post-initialization edits, each admitted as native FULL. The initialized bulk remained legacy. The fixed read probe therefore observes native FULL at depth0 and legacy bulk reads; it does not measure PREFIX-chain read cost.

This review reads existing JSON receipts and source only. No Store, census, build, codec, replay or product edit was performed. Receipt hashes below identify this review's inputs; producer/binary/copy custody authentication remains the parent campaign's responsibility.

## Receipt evidence

The candidate `sdk-binary-8m` Init row reports451 physical records,431 chunks /8397679 canonical bytes in the diagnostic **non-file-provenance** bucket, zero eligible targets and zero native admissions. Do not rename that bucket “metadata-only”: source semantics and this fixture show it can contain actual initialized regular-file content whose FILE marker was not transported.

For each of candidate SDK binary checkpoints1,2 and3:

- One initially missing eligible target,4117 canonical bytes (4096 raw bytes plus21-byte canonical framing).
- One complete native FULL frame and one complete candidate PREFIX frame.
- One native FULL-win fallback,4117 canonical bytes.
- One native FULL admission; zero native PREFIX admissions and zero legacy-DELTA fallback.

Checkpoint3's preparation counters sharpen the explanation in both SDK cases: one base event /4117 canonical bytes, one successful depth0 prior reconstruction, but zero native record fetches and zero native decodes. The reader dispatch counts native record fetches only after the version2 branch (`objects/read.rs:242–262`), while the prior reader also accepts legacy FULL and records a depth0 completion (`read.rs:493–519,614`). This is evidence for a supported legacy FULL prior, not an unavailable hint or unsupported legacy DELTA. The FULL frame is23 bytes and PREFIX frame22 bytes: complete record sizes are28 versus59 bytes, so the one-byte frame improvement loses to the32-byte base-ID difference by31 bytes. This is a measured per-target decision here, not a statement about all payload deltas. A `native_depth_0` prior event alone must not be mislabeled a version2 frame decode.

Thus three edits produce three native FULL records at checkpoint3. A completed prefix attempt is not an admitted dependency. At checkpoint4 each SDK case admits three additional native FULLs from recurrence-related construction; checkpoint5 admits none. Neither later checkpoint is the prescribed read target.

All three candidate8MiB full-read repetitions (rows20,21,24) report `native_decode_calls=3`, `native_raw_decoded_bytes=12288`, `native_dependency_edges=0`, and native depth histogram `[3,0,0,0,0]`. All three candidate8MiB range repetitions (rows14,15,18) report one4096-byte native decode, no dependency edge, depth histogram `[1,0,0,0,0]`. The native raw bytes in a full8MiB read are only0.146484375% of logical file length. This is a logical read-work fraction, not physical compressed-byte attribution.

The three candidate32KiB full-read repetitions (rows8,9,12) likewise report three native FULL decodes /12288 raw bytes /zero edges:37.5% of that file's logical bytes. Their remaining data is still read through legacy representation. The corresponding range probes are depth0 as well.

For a concrete complete boundary, candidate row20 reads8388608 logical bytes and reports476 general group fetches,9907323 general encoded-read bytes,9926015 general decoded-read bytes,12 general decompression calls, plus separately projected native counters (3 native calls,204 native requested bytes,108 native parsed bytes,12288 native raw decoded bytes). These scopes must retain their source definitions; do not add overlapping counters into a fabricated physical I/O or whole-file native decoding total. OS reads/cache remain separate. Native decoder elapsed alone cannot explain total public read time.

## Exact source mechanism

Sources are relative to `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue88-native-payload`:

1. `benchmark/fs-bench-pro/src/storage_smoke.rs:531` selects `Empty` initialization for `deepseek-five` and `deepseek-full`; other smoke cases use `Directory(input/initial)`.
2. `crates/layerfs-layerstack-store/src/layerstack.rs:1056` constructs an `InitializationTaskObjectBuffer` for **structure** and passes a separate object sink to `NativeImport::new_split_with_cache`. `regular_file_with_metadata` calls `build_checked_file(self.objects,...)` at1979. The regular file payload is sent to the object sink, not automatically to that structural buffer.
3. `objects.rs:2491` forwards file construction to the canonical rope builder. `crates/layerfs-content/src/file/rope/build.rs:116` delivers chunks through `ObjectStore::put_file_payload`.
4. The trait default at `crates/layerfs-content/src/object/access.rs:66` discards the optional span arguments and forwards to `put_owned`. The direct initialization sink `FinalizedOutputWriter` implements put/put_owned at `objects.rs:772–783` but does not override `put_file_payload`. Its `push_owned` at715 builds an authenticated object via `AuthenticatedCanonicalObject::new`; that constructor initializes `PhysicalHints::default` at200–213. No FILE provenance survives this route. InitializationTaskObjectBuffer also uses default hints, but attributing the regular-file payload route specifically to that structural buffer would be inaccurate.
5. Native eligibility at `objects.rs:191` requires the FILE provenance bit **and** exact canonical chunk decoding and shape bounds. `objects/admission.rs:122` partitions by this predicate. Unmarked initialization chunks stay in the legacy lane despite being ordinary user file content.
6. Post-initialization file owners enable provenance: `crates/layerfs-workspace/src/changes.rs:1459` invokes `diagnostic_file_payloads`; captured file construction also does so (`capture.rs:197`, `objects.rs:2677`). The ObjectBuffer override at2799 stores the real span and sets FILE when that owner context is enabled. It is these marked newly missing chunks that enter P.

The distinction is source-supported and receipt-consistent: this is a producer-provenance coverage limitation for Directory initialization, not evidence that raw user bytes were misclassified by prefix or that the native reader skipped dependencies. No mutation is proposed under this review.

## Scope of the full157 path

The full157 harness uses Empty Init, so it has no8MiB or other workload file imported through the Directory-initialization path before checkpoint1. The paired `deepseek-five` candidate confirms the relevant route: Init has only12 physical records, zero file-eligible/native targets, and two non-file-provenance chunks /58 canonical bytes. Checkpoint1 then admits283 native FULL file targets /1495276 canonical bytes. Checkpoints2 and3 admit107 and117 native PREFIX targets respectively, with82 and69 native FULL targets. This demonstrates actual predecessor-prefix activity on the Empty-Init/import path, unlike the fixed SDK read probe.

Therefore this initialization limitation does not remove the first full157 workload state from P **under the frozen Empty-Init followed by public imports route**. It is not a claim that every public initialization interface is covered, that all future workloads use this route, or that all157 opportunity/correctness gates have already passed for P. The final native census and158-row graph/provenance validator remain the authority for the eventual full run. Pure raw smoke-counter PASS cannot establish graph cohorts at Directory Init: its eligible denominator deliberately follows producer provenance, not every chunk with file use.

## Contract and interpretation

The native contract requires authenticated regular-file payload provenance before native admission. Current behavior obeys that eligibility guard but covers fewer Directory-Init file chunks than an informal claim that “all payload is native.” Preserve this qualification explicitly. Changing the initialization sink's provenance transport would change which objects are encoded, affect both early representation and later bases, and require a separately frozen treatment; it must not be silently added after these samples.

The public-read contract chose checkpoint3 and fixed ranges before observation and explicitly did not promise maximum depth. These completed results are valid evidence for that declared mixed-format SDK state and public path. They are not a worst-case native-chain, full-native8MiB, or general read-latency qualification. Synthetic depth4 correctness/resource tests remain distinct from public timing evidence. Do not move the probe range/checkpoint, repeat until a PREFIX appears, or describe three native FULL decodes as three PREFIX links. The material unknown is public read cost for the native dependency distribution actually produced by the full157 import path, not whether these SDK counters omitted an observed chain.

## Evidence references

- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-smoke-custody-1/counter-validation.json` — SHA256 `b08a9b539d92c56774fa7cd81e7e699582b8a67eb26b935a9d97bc232d73068a`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-08/result.json` — SHA256 `9c3a5da3131a8dc07cfce2c12ca1fc0291a19d7915566b37092d7c7a04e5eb29`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-09/result.json` — SHA256 `38ae56eb238d37dad30c3c7a524b1dcfdb54f34f536e698addae4a84c664dab9`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-12/result.json` — SHA256 `e18d6eee680c68772a36afb56d81d867748c42c3c4230fbe2ef1f6cbefca6918`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-14/result.json` — SHA256 `c6866e02e614e4de627ce6f50462428c4817ac2c5b5a63f5435c41fd216844d2`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-15/result.json` — SHA256 `0c70b5b046987f0e3c19592eed317c470290b3c25d73e5cc3e2aa06f4c05f246`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-18/result.json` — SHA256 `c4baef309152358daa6cff23d0d996db913d5c0d347ef278f4bb6851bae1f436`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-20/result.json` — SHA256 `f23850abfda3c2c53d56457c49cbf425b1bce019ad23ec7b6b9120e289df5422`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-21/result.json` — SHA256 `528546ef2c0422d0c88c77f05c3f8b60d0b6c5c49635ec331ddd966ba6afb8c3`.
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-SP-public-read-1/row-24/result.json` — SHA256 `2796daec0977de70925aca6ffb6e12770cab49c860a6618233d65db1f2eb4f31`.

# P static native API review — source only, Zstd1.5.7

No build, codec call, test, Store access or benchmark was run. Inspected the pinned zstd-sys2.0.16+zstd1.5.7 vendor sources and generated experimental Rust bindings under `/Users/yifanxu/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zstd-sys-2.0.16+zstd.1.5.7/`, plus the installed matching header. No dependency source was edited.

## Finding

The encoder can use the exact S2 requested parameter sequence with a caller-owned static CCtx and a borrowed raw prefix. Static CCtx prevents workspace growth; whether every supported bounded input fits1MiB still needs compiled proof.

**The apparently symmetric decoder shortcut is not allocation-free:** `ZSTD_DCtx_refPrefix` in1.5.7 creates a DDict through a heap allocation. A static DCtx alone does not prevent that separate object allocation. Use a static DCtx plus an explicit caller-owned static DDict in by-reference/raw-content mode, charged together within the existing256KiB decoder workspace cap. This is a temporary decoding handle, not a new persistent dictionary or trained-dictionary treatment.

The prospective `issue88-native-contract.md` now states this exact route without changing its wire grammar, codec settings, depth, grouping or resource ceilings.

## Encoder path and exact requested settings

`zstd/lib/compress/zstd_compress.c:126–152` initializes a CCtx inside aligned caller storage and sets staticSize. `:1346–1363` implements CCtx_refPrefix by clearing dictionary references and storing the caller's pointer/length/type; it does not allocate a CDict. Later initialization passes the raw prefix directly to compression. `:2148–2168` computes needed workspace and returns memory_allocation before any resize when staticSize is nonzero.

Recommended sequence:

1. Validate raw target/prefix≤32768 bytes and reserve the caller's fixed1,048,576-byte eight-byte-aligned CCtx workspace, within existing physical ownership. Initialize with `ZSTD_initStaticCCtx`; NULL is failure.
2. `ZSTD_CCtx_reset(..., ZSTD_reset_session_and_parameters)`.
3. Set only S2's declared requested parameters: compressionLevel3, windowLog20, contentSizeFlag1, checksumFlag1, dictIDFlag0, nbWorkers0. Check every returned size_t through `ZSTD_isError`.
4. Bind the authenticated raw prefix with `ZSTD_CCtx_refPrefix`; FULL binds null/zero. Keep it alive/unmodified through compression. Do not call loadDictionary, createCDict, enable workers/LDM, or set hidden strategy/hash/search overrides.
5. Obtain checked `ZSTD_compressBound(raw_length)`, require it within the frozen33024-byte frame bound, allocate bounded output, and call `ZSTD_compress2`. It uses one-shot stable input/output buffers and determines the actual source size on its final call (`:6568–6598`, `:6358–6375`), avoiding streaming input/output buffers.
6. Check completion/error and actual frame length. Drop external workspace normally; never free a static CCtx using the C free function. Prefix, target, output and workspace are distinct simultaneously live regions.

This reproduces the S2 *requested* settings. It deliberately allows the same library to derive its internal compression parameters from actual source and raw-prefix size. Exact resulting frames remain a compiled equivalence question, especially across build/CPU feature differences; the report does not claim a byte-for-byte result from source inspection.

## Why a naïve estimate/setCParams transplant is wrong

`ZSTD_getCParamsFromCCtxParams` (`zstd_compress.c:1637–1651`) derives parameters from source size and dictionary/prefix size, applies requested overrides, and adjusts them again. The compress2 path supplies actual source and raw-prefix sizes (`:6367–6375`). However `ZSTD_estimateCCtxSize_usingCCtxParams` (`:1754–1766`) resolves parameters with unknown source and zero dictionary, and estimates token/block work with unknown source size. It can overestimate a small-input one-shot call.

`ZSTD_getCParams` (`:7787–7790`) additionally translates source-size zero into UNKNOWN and uses cpm_unknown, whereas compress2's raw-prefix path uses cpm_noAttachDict (`:5959–5964`). Deriving CParams with a zero prefix, overriding windowLog, then installing every CParam is not demonstrated equivalent to S2's setter/default policy. The existing product's Zstd1/window16 helper must therefore not simply be copied with3 substituted for1.

Use estimate APIs as checked advisory preflight measurements where relevant. Do not allocate a smaller workspace on the assumption that an approximate estimate is exact. Do not interpret an unknown-input conservative estimate above1MiB as proof that all bounded32KiB calls fail, or the observed dynamic541720-byte S2 context as a universal upper bound. The fixed1MiB static allocation is an enforceable hard ceiling; actual parameter-regime tests must establish its applicability. If mandatory FULL encoding cannot fit, stop the frozen implementation rather than allocating dynamically or reducing the window without a new contract.

## Decoder allocation trap and correction

Exact call chain in `zstd/lib/decompress/zstd_decompress.c`:

```text
ZSTD_DCtx_refPrefix :1732
  -> ZSTD_DCtx_refPrefix_advanced :1725
  -> ZSTD_DCtx_loadDictionary_advanced :1699
  -> ZSTD_createDDict_advanced :1707
```

`zstd/lib/decompress/zstd_ddict.c:145–158` allocates the DDict itself with `ZSTD_customMalloc(sizeof(ZSTD_DDict), customMem)`. By-reference mode avoids copying prefix bytes but does not remove the handle allocation. Thus “static DCtx + refPrefix is heap-free” is false for this pinned implementation. The general static-context documentation's dictionary-creation limitation matters here even though the high-level API is called refPrefix.

Use these already bound APIs instead:

1. Check/align `ZSTD_estimateDCtxSize()` and `ZSTD_estimateDDictSize(prefix_length, ZSTD_dlm_byRef)`. The sum of actual caller capacities must be≤262144 bytes, included inside the1MiB per-chain scratch.
2. Initialize `ZSTD_initStaticDCtx` in its own region and reset its parameters. Enforce windowLogMax20 plus all contract frame/checksum/content-size/dictionary-field checks before decoding.
3. For a nonempty prefix, initialize `ZSTD_initStaticDDict` with its separate caller region, `ZSTD_dlm_byRef`, and **ZSTD_dct_rawContent**, then call `ZSTD_decompress_usingDDict` with that handle. FULL/no prefix uses no DDict.
4. Keep prefix and static handle alive/unmodified through the call. Reinitialize/rebind on each independent record. Do not call the heap free functions on static regions.

`zstd_ddict.c:187–208` places the DDict in supplied storage; by-reference initialization (`:120–137`) does not enter the prefix-copy allocation branch. Explicit rawContent (`:96–102`) prevents file bytes beginning with the dictionary magic from being misinterpreted as a trained dictionary. Merely replacing refPrefix with decompress_usingDict would use automatic dictionary interpretation and is not the right raw-prefix substitute for arbitrary payloads.

`decompress_usingDDict` follows the existing one-shot frame reader. Validate one complete ordinary frame first: the C helper can otherwise process multiple frames. Preserve direct canonical reconstruction/authentication after output, and never treat checksum success as ObjectId validation.

The needed functions are exported in the pinned generated `bindings_zstd_experimental.rs`: CCtx_refPrefix at609; decompress_usingDDict at557; estimateDDictSize at966; initStaticCCtx at973; initStaticDCtx at985; initStaticDDict at1008. No external binding/dependency patch is necessary.

## Future compiled proofs, not executed here

- Static versus S2 dynamic requested-parameter/frame equality for the fixed deterministic cases and source/prefix size regime boundaries, including zero/max lengths. Do not change levels or select favorable samples after seeing output.
- Peak actual simultaneous capacities: fixed CCtx, static DCtx+DDict, output bound, raw prefix/target, retained frame descriptors and enclosing-group buffers; honor existing owner limits.
- Allocator instrumentation or equivalent narrow proof that the selected static API path never calls malloc/realloc, including dictionary-magic-prefixed raw bytes and error paths. No kernel tracing is required.
- Wrong/truncated/concatenated/dictionary-field/checksum/window/content-size failures, context reset and borrowed-prefix lifetime, depth/closure and canonical identity roundtrip.
- Deliberately undersized static workspaces return resource errors without heap fallback or parameter changes. Optional prefix resource failure retains FULL under the contract; mandatory FULL failure rejects the candidate build/run. Corrupt stored data is always integrity failure.

The D build may continue independently. This review identifies a concrete P implementation requirement; it does not certify a native encoder/reader or launch P before the D/C decision.

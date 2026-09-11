# Real LayerFS source Init: prospective diagnostic contract

Owner requested a real production-data run and selected LayerFS actual source files after reviewing the pseudorandom namespace fixture. This is a separate `layerfs-source-init-v1` diagnostic case under #111, not a replacement, relabeling, or relaxation of `namespace-100000`. No new release family or comparative performance claim is admitted.

## Frozen input

385 existing tracked regular files, 4,989,091 bytes, from commit `6d42d0e3b430690815ae4fcb077a48872bba2433` plus the preserved original dirty working tree. Rejected advisory-prefetch experiment removed before the snapshot. Include crates/, tools/, benchmark/, containers/, .github/, Cargo.toml, Cargo.lock, README.md, LICENSE, .gitignore, .dockerignore and .gitattributes. Exclude .git, build outputs, untracked files, docs and release-notes (the latter are dominated by historical benchmark evidence). No repeated/generated content, padding, filtering by compressibility, or file-size adjustment. File bytes, modes and mtimes copied to an independent snapshot.

Input manifest SHA256: `c6534deb6a187a3a7da09a062684f6658a78575958b55b6d676e86b60cc0f2c6`.

Evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-real-source-evidence/run-20260911T001109Z`. `input-manifest.json` records each path, SHA256, length, mode and mtime; `oracle.tsv` is independently produced by Python hashlib.

## Operation, samples and verification

Three sequential plain performance samples, one untouched independent empty Store per sample. Timer `layerstack_init_ns` begins immediately before public `Client::initialize_layerstack(Directory(input))` and ends on return. Setup, physical snapshots, oracle work, reporting, Store closure and verification remain outside it. The original promoted-uncompacted product remains unchanged. SDK, SQLite and physical storage stay on macOS; no Docker-owned Store. Init creates no runtime/FUSE activity.

Cold acquisition before each sample: read-only shared mmap, touch mapped input pages, msync(MS_SYNC|MS_INVALIDATE), unmap and close. Preserve input metadata and hashes. Report acquisition method, `fixture_cache_profile=reused-first-sample-uncontrolled`, actual initialization disk reads, process CPU/RSS/swaps, and Store allocated/apparent/canonical bytes per sample. Require disk reads >= logical input bytes for the cold label; retain any failure as cache-invalid without relabeling. No warm-up for cold. No control arm or speedup claim. Report n, median and range; the synthetic2.7s gate does not apply to this smaller real-data case.

Independent fresh-process verification of each retained Store after performance: enumerate the complete stored namespace, require exact file paths, stream and SHA256-check every file against the frozen Python oracle, verify byte counts and persisted layer/root identity, and report verification separately. This is exhaustive file-content/namespace verification for the selected source corpus, not a live FUSE or general production qualification.

Read-only physical pack census outside timing: identify actual pack versions, FULL/DELTA counts, raw small-content versus Zstandard frame bytes, canonical and total pack bytes. Distinguish compression, exact deduplication, representation overhead, SQLite overhead and filesystem allocation.

Use the runner-owned measurement lock for builds, each run, census and proof; no overlapping resource-sensitive work, no child double acquisition. Requalify host through shared/runner.py --build-host and retain exact source/product/binary/image identities. Product300s/outer310s; proof45s. Every attempt/log/exit retained. No selective retry or valid outlier removal; one whole affected sample/proof replacement only for demonstrated infrastructure failure. Product failures remain failures. No compaction/VACUUM/repack/GC or work shifted outside Init.

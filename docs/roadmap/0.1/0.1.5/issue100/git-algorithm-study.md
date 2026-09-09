# Git 2.47.1 packing study: ten-snapshot baseline

Scope: read-only study of pinned upstream sources and retained `git-pack-attribution.json`; no new benchmark, encoding, repack, product edit, dependency installation, or Store rewrite. Snapshot selection and measured Git repository are unchanged. Sizes are decimal MB unless explicitly KiB.

## Measured facts

Source: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence/git-pack-attribution.json`, describing the retained pack in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence/git/snapshots.git`.

| Git population | Objects | Packed bytes |
|---|---:|---:|
| Blob FULL | 9,684 | 28,143,165 |
| Blob DELTA | 23,597 | 6,163,088 |
| Tree FULL | 6,172 | 1,312,933 |
| Tree DELTA | 1,300 | 251,323 |
| Commit FULL | 10 | 1,930 |
| Total object entries | 40,763 | 35,872,439 |

These are packed object-entry bytes, not whole-repository allocated storage (38,223,872 bytes). Do not compare entry totals directly with LayerFS allocated bytes without accounting for indexes and filesystem overhead.

Of Git's 23,597 deltified blobs, 18,604 (78.84%) have a dependency closure requiring a later selected snapshot; 2,762 need only earlier checkpoint closure and 2,231 same-checkpoint closure. Later-dependent entries occupy 4,519,458 bytes. This is dependency-closure availability, not a claim that every direct base is a later version of the same path. Actual maximum chain depth is 23 for blobs and 7 for trees.

## Proven Git source behavior

`type_size_sort` orders by object type, filename hash and descending size, with additional preferred-base/island ordering. The source deliberately prefers larger bases for smaller targets: deletions can be cheap and larger versions are often newer. `try_delta` rejects different types, enforces depth, filters by sizes, and seeks a better delta; equal-size replacements must be shallower. Its depth-aware threshold discourages expensive deep deltas. `find_deltas` uses a bounded candidate window with data/index caching and optional memory limits. Defaults are window 10 and depth 50. Delta data is compressed per object using zlib; the entire pack is not one shared compression stream. [Pinned pack-objects implementation](https://github.com/git/git/blob/v2.47.1/builtin/pack-objects.c#L2446-L2820)

The filename hash emphasizes trailing non-whitespace characters. It is a cheap locality heuristic, not a content-similarity index or a restriction to the same path. [Pinned name hashing](https://github.com/git/git/blob/v2.47.1/pack-objects.h#L190-L207)

Git's binary delta engine indexes base bytes using Rabin fingerprints and scans target bytes for matches, emitting copy ranges and inserted literals. It bounds pathological hash buckets and can stop when a delta exceeds its budget. It does not use Git commit patches or CDC chunk object identities to represent these differences. [Pinned delta engine](https://github.com/git/git/blob/v2.47.1/diff-delta.c)

Pack records distinguish FULL objects from reference/offset DELTAs. Bases may themselves be deltified; readers reconstruct the base before applying copy/insert instructions. Object IDs/index lookups are separate from physical representation. Consequently, changing a pack representation does not require changing the canonical object's identity or snapshot references. Trees participate in the same physical delta mechanism as blobs. [Pinned pack format](https://github.com/git/git/blob/v2.47.1/Documentation/gitformat-pack.txt#L88-L111)

## What this means for LayerFS (inferences, not measured improvements)

**Future-base access is a material advantage in this actual Git pack.** An online immutable representation selected when an object first appears cannot reproduce a dependency on a future snapshot. Reproducing this specific Git graph requires deferred physical encoding or replacement of previously written encodings. Adding another encoding without reclaiming the old physical bytes does not deliver the corresponding allocated-space saving.

This does **not** establish that comparable total bytes require the same graph, depth 23, or offline repacking. A different online graph might be competitive; the current data does not quantify that possibility.

The growing-file example exposes direction independently of chain depth. Start with 10 KiB, append 1 KiB of incompressible new bytes per version, and retain 20 versions (last is 29 KiB). Ignoring framing:

- Forward DELTAs to the original FULL: 10 + (1 + … + 19) = 200 KiB.
- Forward chain to immediate predecessor: 10 + 19 = 29 KiB.
- Reverse DELTAs directly to the final FULL: approximately 29 KiB plus small COPY instructions for 19 older prefixes, still depth one.

Thus one-level deltas are not inherently unable to compress append histories. The combination of **past-only FULL anchors, immutable old placement, and repeated accumulation** causes the simple example's loss. Reverse depth-one encoding needs the final base first or replacement of older encodings. Real edits/deletions are less ideal than the example.

Git's 1.56 MB of tree entries is small even though most tree entries are FULL. Metadata efficiency therefore cannot be explained solely by counting DELTAs: canonical representation, exact sharing and per-object size need separate comparison. LayerFS preserves additional POSIX metadata, so equal metadata bytes cannot be assumed.

## Recommended direction and scope boundary

1. Keep canonical content identities and historical roots unchanged. Investigate physical base choice and reconstruction cost separately from logical object identity; a storage optimization need not redefine CAS keys.
2. Under the original fixed policy, quantify avoidable FULL fallback and repeated payload first. Do not promise 40 MB from copying Git's delta matcher; Git's advantages include base direction, candidate availability, chain freedom and compact trees.
3. A bounded recent-candidate search or bounded forward chain is an online design hypothesis. Short chains violate the original one-level constraint; broader candidate search changes the specified selection policy. Study is authorized, but these are proposed design changes, not implemented behavior or qualified results.
4. Reverse replacement could retain depth one and favor recent reads, but requires safe physical replacement, dependency handling, atomic locator publication and actual byte reclamation. This is outside the explicitly excluded repacker/GC scope; do not smuggle it into an anchor tweak.
5. Before selecting an implementation, compare ten-snapshot LayerFS content FULL/DELTA totals and metadata/index bytes with the Git populations above. The 40 MB ambition leaves only about 4.13 MB beyond Git's existing 35.87 MB object entries; it requires a concrete whole-Store budget, not DELTA count parity.

No claim of matched foreground performance, feasible 40 MB allocation, or release admission follows from this source study. Git packing cost and LayerFS public save/Commit cost retain their different measurement boundaries.

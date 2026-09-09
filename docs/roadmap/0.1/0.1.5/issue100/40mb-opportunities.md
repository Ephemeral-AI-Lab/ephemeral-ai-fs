# Storage opportunities toward 40 MB

2026-09-10. Three parallel diagnostic studies requested by the owner: metadata representation, SQLite/index/pack framing, and current content versus Git. Reviewed source `1955205db82f4bf84b73efaf60b97c1d508320cc`; retained measured product `ee78028ba56741002a627b3a8875d3c888c86708`.

**Current allocation remains 49,319,936 bytes. No implementation or new product measurement in this study achieves 40 MB.** The useful change in diagnosis is that whole-file compressed frames are already close to Git on the same objects. Metadata representation and object granularity now dominate the remaining total gap.

## Current budget

All values are decimal bytes for the same ten selected snapshots. Existing performance allocation is reused; the diagnostic Store is the post-verification same Store, with SHA256 `713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4`, matching its verification manifest. Its earlier frozen performance digest differs by lifecycle and is not substituted.

| Category | LayerFS | Git | Difference |
| --- | ---: | ---: | ---: |
| Content packs / blob entries | 37,879,487 | 34,306,253 | 3,573,234 |
| Metadata packs / trees and commits | 6,550,582 | 1,566,186 | 4,984,396 |
| Indexes, structures and allocation | 4,889,867 | 2,351,433 | 2,538,434 |
| Total allocated | 49,319,936 | 38,223,872 | 11,096,064 |

Reaching 40,000,000 requires **9,319,936 B**, or 18.90%, of net saving. Content explains 32.20% of the remaining Git gap, metadata 44.92%, and other allocation 22.88%. The earlier content-dominant diagnosis of the 66-MB implementation is obsolete for prioritizing this candidate.

Matching Git content while holding everything else constant gives **45,746,702 B**. Reaching 40 MB on that basis requires **5,746,702 B**, or 50.23%, less non-content allocation. Conversely, holding all current non-content costs fixed requires content of 28,559,551 B, 16.75% below Git's measured blob entries. Neither counterfactual is a forecast.

An illustrative budget is 34,306,253 B content + 2,500,000 B metadata + 3,193,747 B remaining allocation = 40,000,000 B. No alternative metadata encoding yet demonstrates those numbers. Git's narrower filesystem semantics and offline packing remain explicit.

## Whole-file delta encoding is no longer the principal deficit

The content study decoded and joined all 33,217 current SmallContent objects to exact Git blob identities:

| Same objects | Bytes |
| --- | ---: |
| LayerFS compressed frames | 32,968,802 |
| Git blob entries, including Git record framing | 32,534,916 |
| Difference before LayerFS framing | 433,886 |
| LayerFS complete SmallContent packs | 34,619,995 |
| Difference including LayerFS framing | 2,085,079 |

There are still 12,274,145 B of LayerFS FULL frames whose Git objects are DELTAs requiring future base closure. That is **not an 11-MB savings pool**: the opposite population contains 6,313 objects for which LayerFS DELTA frames cost 3,998,588 B while Git FULL entries cost 19,527,515 B. The latter offsets 15,528,927 B of apparent assignment differences. Any reverse-encoding proposal must charge the entire graph, changed FULL bases, retained dependencies and actual old-byte reclamation.

The current bounded forward graph is substantially more competitive than the original one-level implementation. Further SmallContent search and reverse packing remain hypotheses; the rejected retained-DELTA cache must not be repeated without new evidence.

## Larger-file CDC has a concentrated mismatch

Outside the 33,217 small objects, Git has 56 large regular-file versions occupying 1,771,026 entry bytes; eight other blobs are symlinks/empty content totaling 311 B. LayerFS CDC packs occupy 3,259,492 B. The small semantic remainder is disclosed rather than silently assigned to CDC.

The study decoded all 735 current CDC records and matched their bytes to original large files. Physical chunks shared by multiple paths are counted once.

| Family | LayerFS CDC frame bytes | Git entry bytes | Difference |
| --- | ---: | ---: | ---: |
| Nine pnpm-lock.yaml versions | 1,116,495 | 237,570 | 878,925 |
| Eight api-catalog.ts versions across three package locations | 455,478 | 146,699 | 308,779 |
| One PDF version | 503,618 | 492,305 | 11,313 |

The lockfile versions range from 308,097 to 841,964 raw bytes. Most retained frame bytes are already PREFIX deltas: 70 FULL chunks cost 354,164 B and 179 PREFIX chunks cost 762,331 B. Increasing DELTA count is not the diagnosis.

The code finds up to four predecessor chunks overlapping the target's **absolute file offset**, then admission tries only the first. Insertions/reordering can move useful content beyond that span; alternative overlapping hints are also ignored. This is a source-confirmed restriction, but current telemetry does not attribute all 878,925 B to it. Depth/closure caps and Git's future-base freedom can also matter.

First experiment: a fixed subset of expensive original lockfile chunks, comparing the selected base with the other at-most-three existing hints under the same codec and memory/depth limits. Only broaden to content-based CDC discovery if this demonstrates poor base selection. A bounded 128-KiB–1-MiB whole-file representation is a more aggressive second option; it requires a new grammar, window/memory accounting and historical random-read analysis, not a threshold-only edit.

## Destructive simplification worth investigating: metadata indirection

LayerFS indexes **80,240 objects**, of which **46,288 are metadata**. Git has 40,763 total entries, including only 7,482 trees/commits. LayerFS content-object count is already similar to Git's blob count. The additional independently addressed metadata structures explain most of the object-count difference.

The measured metadata includes **37,289 separate inode records**, 4,008 directory-state wrappers, and 695 inode-table nodes. Only **four metadata-root values** occur; timestamp or permission diversity is not the dominant cause. Separate records serve a meaningful interface, but the representation may be changed without deleting hardlink or inode-identity semantics.

The largest candidate is to inline inode-record fields into authenticated inode-table leaves and stop storing/indexing the individual inode records. A second candidate folds directory-state wrappers into their owning representation. This changes canonical metadata encoding and must preserve stable identities, shared hardlinks, modes, references, historical state, bounded page reads and old-format support.

There is a real tradeoff: 37,289 distinct inode records occur in 58,871 leaf entries. Inlining repeats fields across changed leaves, and larger leaves can create more pages. Deleted object bytes or locator rows alone are not net savings. The next experiment must encode actual replacement leaves, including partitioning, and measure complete compressed metadata plus replacement locators.

With current membership, replacing each 32-byte record reference by its 73-byte inode value gives a modeled raw reduction of only **1,240,611 canonical bytes**: `37,289 × 98 − 58,871 × 41`. This is not compressed saving. Entries grow from 64 to 105 bytes, reducing the existing 8-KiB page capacity from 127 entries to at most 77; replacement page growth must be charged. Eliminating directory wrappers removes another **392,784 canonical bytes** before compression and reader effects.

An independently constructed compact full-hash index containing surviving current rows would save **1,716,224 B** if inode rows disappeared, or **1,896,448 B** if inode and directory-wrapper rows disappeared. These are optimistic index projections against an equally compact baseline; they exclude additional replacement objects. They are not additive to another projection already including the same rows.

The metadata study also found zero inode-table node reuse across these retained roots. However, the implementation already uses local path-copy B-tree updates. For every transition, every old leaf has at least one changed or deleted old pair; a stable-boundary tree alone cannot reuse those existing leaves unchanged. This is not evidence that an existing persistent-tree mechanism was omitted.

Repeated high-entropy inode keys merit a separate format study. There are 94,873 canonical inode-ID occurrences, totaling 3,035,936 raw bytes. An 8-byte identity would remove 2,276,952 raw bytes before rehashing, compression, partition and delta changes. It is not a measured pack saving. Production allocation derives IDs from seed/path, not an available recoverable serial. A new stable compact identity allocator needs persisted provenance and branch/import semantics. Keeping full canonical identities with a group/pack-local physical dictionary was tested arithmetically and increases raw size; a global translation dictionary has substantial lookup and authentication ownership costs.

## Smaller, well-bounded format cuts

The object table is already WITHOUT ROWID, with no redundant secondary object index and no free pages. Disposable-copy VACUUM saves only **430,080 B** in logical Store size; tighter varint locator encoding saves a further **114,688 B** against an equally compact layout. Pack overflow geometry remains. These are offline layout measurements, not public-operation allocation gains.

SmallContent has one record per group. A new directory grammar containing only four-byte group starts can derive end offsets and remove redundant directory fields: **398,604 B** across 33,217 groups. Deriving raw/frame lengths from validated locator length and record range offers another **265,736 B** ceiling. Combined framing opportunity is **664,340 B**, before SQLite effects and after implementing explicit format fencing and bounds checks.

Do not treat all 812,544 B of delta base hashes as free overhead. The current base reader independently authenticates full base IDs. A simple global full-hash dictionary plus three-byte references saves only 57,104 B before its own lookup/framing costs. Sharing existing object identities through a compact physical ordinal system requires a real publication and authentication design.

## Recommended order

1. **Metadata coalescing economics:** encode replacement inode leaves and wrapper layout from actual retained metadata; compare full compressed metadata and index inventory. This is the first substantial architectural experiment because the 40-MB budget needs multi-megabyte non-content savings.
2. **Fixed-family CDC base experiment:** pnpm-lock.yaml first, then the renamed API catalog only if the first mechanism holds. Compare identical targets/bases with unchanged codec/bounds before expanding search globally.
3. **Compact SmallContent directory:** a small independent physical-format change with an exact 398,604-B pack target; optionally remove redundant length fields after preserving preallocation/authentication checks.
4. **Compact stable inode identities:** investigate if coalescing plus the smaller cuts cannot support the budget. Do not replace full content hashes with truncated hashes.
5. **Reverse/repacking only if still evidenced:** preserve all roots and reclaim old physical bytes atomically. Current whole-small parity does not justify prioritizing a general repacker.

Do not sum ceilings, raw-byte models, offline compaction and overlapping index projections into a promised final allocation. A concrete design must state exact old/new object inventory, memory/decoder ownership, authentication, rollback, format compatibility, foreground maintenance cost and read locality. A substantive implementation then needs the same ten-state public performance, frozen census, same-Store verifier and cleanup. The current Commit median/sum regressions of 17.70%/23.64% remain the speed baseline for tradeoffs, not something to hide through untimed maintenance.

## Evidence

The new study root is `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study`. It contains `budget.json`, the three independent reports, source-referenced scripts and machine-readable diagnostics. The index experiment's disposable database is excluded. Original Stores, fixtures, product source, controls and benchmark results were not changed. No new product build, smoke or full157 run was launched.

- [Metadata study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/metadata/report.md)
- [SQLite and framing study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/index/report.md)
- [Content/Git study](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/content/report.md)
- [Budget arithmetic](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-study/budget.json)

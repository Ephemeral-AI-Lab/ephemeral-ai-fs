# Full157 metadata experiment: fixed D verified on all158 roots

The exact ten-state D design completed on the entire original history: **158 namespace roots (empty+157 historical states), all129,106 original metadata objects, and every14,429 rewritten directory object**. The new metadata packs cost **34,917,104 bytes**, compared with the original selected metadata's49,487,786 bytes. This is an actual full-history encoding result, not a ten-state extrapolation.

| Full-history measure | Original source | Fixed D |
|---|---:|---:|
| Metadata canonical bytes | 79,372,108 | **80,160,065** |
| Metadata objects | 129,106 | **24,748** |
| Encoded metadata groups | 6,296 | 5,047 |
| Metadata packs | 960 | 314 |
| Complete metadata pack bytes | 49,487,786 | **34,917,104** |
| All object locator rows | 207,725 | **103,367** |
| Objects index allocation | 10,686,464 B in source | **4,820,992 B in isolated compact projection** |
| Additional scope allocator | — | **4,096 B** |
| Metadata+index+allocator subtotal | 60,174,250 B | **39,742,192 B** |

**Metadata pack bytes fall14,570,682 B (29.44%). Canonical metadata bytes increase787,957 B.** Inlining removes separately indexed inode records but repeats their values within changed table pages. Compact serials and altered page boundaries still reduce the actual compressed inventory, but the ten-state reduction rate does not transfer to this longer history. The original history already stores4,673 metadata deltas; D retains the fixed FULL-only inventory encoding policy. The page-cost comparison also includes compacting the projected index. The subtotal is therefore a measured difference between the source layout and this explicit counterfactual, not an isolated causal estimate for one code change and not a complete Store allocation.

No A/B/C alternatives, public history replay, Docker, product build, Git operation, parameter sweep or product integration ran. Parent's combined offline layout experiment owns complete Store accounting and content changes; this report does not add estimated content savings or claim a40MB product Store.

## Exactly the same D representation

The original inode identities receive52,724 stable serials in the fixed first-appearance order across all158 roots. Serial assignments never change or recycle. Canonical identity is a32-byte origin scope plus8-byte serial, with the scope stored once in each namespace root. No content or canonical object digest is shortened; generated metadata objects retain actual32-byte BLAKE3 IDs.

D removes128,032 original metadata objects and creates23,674 new objects. It retains1,074 original support objects unchanged:

| Retained/new role | Objects |
|---|---:|
| Original large-file metadata | 1,058 |
| Original metadata nodes / legacy value chunks / symlinks | 4 / 5 / 7 |
| Rewritten directory mapping nodes | **14,429** |
| New inline inode-table nodes | **9,087** |
| New scoped namespace roots | **158** |

The same leaf/branch rules apply:100-entry inode leaves with81-byte entries;127-entry inode branches with40-byte entries; balanced deterministic partitions;81-byte-per-entry subtree summaries; original directory page membership and levels preserved while inode references shrink32→8bytes and directory ancestor hashes/summaries are recalculated. Separate inode objects and directory state wrappers are removed, and every affected content/table/root reference is rewritten. Maximum generated canonical page size is **8,144 bytes**, below the8,192-byte cap.

D uses the same profile/scope descriptors as the ten-state experiment, FULL-only role-magic/full-ID order,16KiB group target, pinned legacy Zstandard policy and complete pack framing/bounds. This was one fixed treatment, not a search for settings that look better on full157.

## Verification and source integrity

The first attempt completed in20.57seconds. Its append-only log is `attempt-01.log`; there were no failed or discarded encoding attempts.

- Matched the post-verification source Store SHA256 to `verification-manifest.json`: `f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7`, checked before and after.
- Decoded all124,433 original FULL metadata records and4,673 legacy deltas. Every reconstructed canonical object authenticated against its actual full BLAKE3 identity.
- Recreated **all6,296 original selected groups byte-for-byte**, including their exact RAW/Zstandard choice, using the pinned static-context parameter sequence. Original encoded group bytes49,371,690 plus116,096 framing bytes reconcile to49,487,786 pack bytes.
- Verified **all158 states** by mapping decoded D serials back to original stable inode identities and comparing exact kind, namespace reference count, file content root and metadata root. Directory content roots resolve to the rewritten mappings and preserve original namespace meaning.
- Verified **all14,429 directory objects**, including objects shared between snapshots: identical ordered original name/inode pairs, correct subtree counts, recalculated encoded-byte summaries, preserved levels and rewritten full-hash branch references.
- Reused the unchanged focused serial/hardlink/rename/shared-ancestor/global-branch-allocation checks; they passed. This is a representation and allocation-model check, not product concurrent allocator testing.
- Authenticated every generated object, charged every replacement locator and allocator page, and passed SQLite integrity checks for the diagnostic index and cache.
- The independent cached API check verified all314 pack hashes and the158root mapping without re-encoding history. All ten-state artifact seals remained unchanged.

The codec made11,343 calls:6,296 original equivalence checks plus5,047 new groups. Maximum static codec workspace remained218,136 bytes. Runtime duration is diagnostic execution time, not a Save/Commit performance measurement.

## Index, allocation state and cached exports

The projected `objects` index includes78,619 unchanged content locators and24,748 actual D metadata locators. It stores full hashes and their actual canonical lengths and physical positions. Its1,177 pages occupy4,820,992 bytes. The new persistent `scope_allocator` has scope32/highwater52,724,38bytes of SQLite payload andone4,096-byte page. The complete index-only database is4,829,184 bytes, including one shared4,096-byte SQLite schema page. The subtotal excludes the schema page consistently and charges the allocator page once.

The original32-byte→serial mapping in `inode-map.json` is the diagnostic equivalence oracle. Fresh-format D readers do not need an old-ID translation dictionary. Transactional global allocation, rollback/publication, foreign-scope imports and existing-Store migration remain product work. A snapshot-local counter must not replace the global highwater because branches could reuse identities.

`cache.sqlite` contains only the **new diagnostic metadata inventory**, its actual encoded packs/locators, and content locator rows for projection; it is not a copy or mutation of the original Store. `api.py` exposes:

- `original_inventory()` → authenticated cached inventory,158new roots, original content locator rows, scope and highwater.
- `pack_blobs(inventory, base_pack)` → actual unchanged pack bytes and locators with physical pack numbers rebased for composition.
- `root_map()` → all158 old→new root IDs.
- `inode_oracle()` → original32-byte inode IDs→8-byte serials.
- `decode_table` and `decode_directory` → exact D semantic decoders.

The combined-layout agent has these APIs and mapping artifacts. Cached exports avoid another complete encoding run. Original file-oracle verification can use the root/serial mappings and actual decoded content roots independently.

Artifacts are sealed by `manifest.json`: protocol, generalized local helpers, executable full treatment, result, root and inode mappings, cache, index-only projection, API check and report. The helpers were copied/generalized locally; sealed ten-state files were not edited. `result.json` records source, library and script identities and all314 actual pack hashes.

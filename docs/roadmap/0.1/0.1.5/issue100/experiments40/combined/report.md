# Combined offline result: 41,648,128 bytes

**The three fixed changes combine to a complete diagnostic SQLite copy of 41,648,128 logical and observed allocated bytes.** This is 1,648,128 bytes above the 40,000,000-byte target. It is an actual assembled layout with all pack/index/root/allocator state charged, rather than a sum of separately projected savings.

| Equally compact offline layout | Logical bytes | Observed copy allocation | Saving vs baseline |
| --- | ---: | ---: | ---: |
| Original, existing verified VACUUM control reused | 48,783,360 | 48,783,360 | 0 |
| D scoped inode metadata + SmallContent grammar B | 42,192,896 | 42,192,896 | 6,590,464 |
| D + B + fixed chronological CDC family graph | **41,648,128** | **41,648,128** | **7,135,232** |

The final reduction versus the matched compact baseline is 14.63%. Adding the fixed CDC graph reduces the assembled D+B copy by 544,768 B, compared with its 551,895-byte intrinsic pack saving; SQLite page geometry absorbs the difference. No format parameter sweep was performed.

## Exact reconciliation

| Final component | Bytes |
| --- | ---: |
| SmallContent packs, diagnostic grammar B | 33,955,655 |
| Native CDC packs, including 12 new FULL records | 2,707,597 |
| D metadata packs, all 25 packs | 2,639,978 |
| **All pack BLOB bytes** | **39,303,230** |
| SQLite bytes outside pack BLOB payloads | 2,344,898 |
| Filesystem allocation minus logical size | 0 |
| **Complete offline copy** | **41,648,128** |

The final database has 38,852 selected objects and 619 packs. The full 32-byte-hash object index occupies 1,806,336 B. The 32-byte namespace-scope/highwater allocator table occupies 4,096 B, with highwater 17,922. Pack-table B-tree allocation is 39,776,256 B, including 419,859 B of unused page space. Every residual SQL table and index is included in the total.

Even the final pack payload alone occupies 39.303 MB; there is only 696,770 B of room for indexes and all other storage under 40 MB, whereas the preserved full-hash index alone uses 1.806 MB. Reaching 40 MB needs additional payload reduction or another substantial representation change, not just accounting cleanup.

## Correct identity and root rewrite

Changing canonical namespace roots requires changing typed history identities. The script first recomputed and checked the source's one genesis LayerId and all ten CommitIds using the repository's current BLAKE3 domains and optional-parent encoding. It then derived the new genesis layer and ten commits in parent DAG order, including their changed roots, parents and base layer.

The copy rewrites all corresponding layer-stack heads, branch base-layer/head-commit references, commit parent/base/root fields and layer references within one deferred-foreign-key transaction. Branch/stack random IDs, names and other fields are preserved. Workspace-stage references were checked; this fixture has no staged rows. SQL rows were compared against the complete expected transformed records, and SQLite foreign-key and integrity checks passed. The full mapping is in `identity-map.json`.

**The old externally visible LayerId/CommitIds are not preserved.** D changes the canonical metadata format, so new identities are expected. The comparison preserves snapshot meaning and content identities; it is not a transparent migration of old external handles.

## Verification performed on assembled copies

- The fixed D API authenticated all 46,288 source metadata objects, reproduced its recorded 25 pack hashes, and checked all 11 namespace states and directory transformations. Every new physical metadata record was decoded from the assembled copy, authenticated and compared with the exact 4,900-object D inventory. New namespace tables, directory inode references, root inode and metadata/content references were checked again against the copy.
- Every 33,217 SmallContent object and its dependencies was decoded and authenticated from the first combined copy, retaining depth 8, 512 KiB canonical and 256 KiB encoded closure bounds. The second copy's SmallContent pack graph and unchanged locator fields exactly match that authenticated graph, so identical decoding was not repeated. Each compact pack reconstructs the original v3 pack byte-for-byte.
- All 735 native objects were decoded and authenticated for each applicable graph. The final graph retains maximum depth 4, raw closure163,840 B, encoded read work29,239 B and decoded work171,781 B, within the existing limits. All 249 selected family records include their actual new frames and full base IDs; all other native records remain unchanged. Existing pack/group/ordinal locations remain legal, with no repartitioning or extra pack omitted.
- Every selected object locator maps to exactly one physical record, and every physical record is selected. All 33,952 content canonical IDs, canonical lengths and locator fields remain unchanged. Metadata locator replacement, all pack pages and the allocator row are charged.
- Both copies passed SQLite integrity and foreign-key checks. The measured source SHA-256 remained 713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4 before and after. The previously saved matched baseline was hash-verified and reused; no new baseline run was performed.

The combined construction and checks took 24.83 seconds as an offline script. This time is **not a Save/Commit performance result**.

## Scope and remaining qualification

These complete copies use unsupported diagnostic schema versions 9201/9202 and the isolated SmallContent diagnostic grammar. They are intentionally unsuitable for the current product to open. No product code, measured Store, runtime or Cargo state was modified; no product build, Docker run or full smoke was performed.

The 41,648,128-byte result qualifies this particular compact offline layout and standalone semantic reconstruction. It does not qualify online append allocation, actual Rust readers/writers, concurrent scoped-ID allocation, branch/import behavior, migration, rollback, cold reopening, historical checkout APIs or Save/Commit latency. The original 49,319,936-byte performance Store had a different growth lifecycle; using it as this experiment's matched baseline would conflate compaction with representation savings.

Artifacts: `protocol.md` was written before construction; `combine.py` is the executed script; `result.json` contains input/source/output hashes and per-table reconciliation; `identity-map.json` records 11 namespace-root, one layer and ten commit mappings. `D-B.sqlite` and `D-B-CDC.sqlite` are the unsupported offline copies, not replacement benchmark Stores.

# Depth1 metadata deltas: complete full157 copy falls to 98,668,544 bytes

**The fixed depth1 metadata delta treatment produces a complete offline copy of 98,668,544 logical and allocated bytes.** Against its matched chronological FULL control, it saves 9,412,608 B (8.71%). Against the prior hash-sorted FULL-metadata copy, it saves 9,289,728 B. All canonical objects, content, namespace roots, typed history identities and allocator state are unchanged.

| Offline full157 layout | Metadata pack bytes | Complete logical/allocated bytes |
| --- | ---: | ---: |
| Prior hash-sorted D metadata + B + CDC | 34,917,104 | 107,958,272 |
| New matched chronological FULL control | 35,010,506 | 108,081,152 |
| **Same chronological groups, selected depth1 deltas** | **25,624,588** | **98,668,544** |
| **Matched FULL→delta saving** | **9,385,918** | **9,412,608** |

Both new copies use exactly 388 metadata packs and 5,154 group memberships, with the same 24,748 canonical metadata objects and identical locator positions between treatments. The prior hash-sorted baseline has different packing order, so the chronological FULL control is the appropriate comparison for isolating the delta policy. No new parameter variants or encoding sweeps were run.

## Final storage reconciliation

| Component | Bytes |
| --- | ---: |
| SmallContent framing B packs, unchanged | 58,979,700 |
| Native CDC packs, unchanged | 7,211,036 |
| Selected depth1 metadata packs | 25,624,588 |
| **All pack BLOB bytes** | **91,815,324** |
| Full 32-byte canonical-object index | 4,820,992 |
| Pack-table SQLite overhead above BLOB bytes | 1,905,252 |
| Remaining SQL/schema/indexes, including allocator | 126,976 |
| **All SQLite outside pack payloads** | **6,853,220** |
| Allocation difference | 0 |
| **Complete copy** | **98,668,544** |

The final copy contains 103,367 selected objects and 2,568 packs. Of 24,748 metadata objects, 20,506 remain FULL and 4,242 use deltas. Those deltas reference 4,209 distinct selected FULL bases; each base is counted once among stored FULL objects, never added as a second overhead charge. Maximum depth is 1 and maximum target+base canonical closure is 16,288 B, within the 16 KiB bound.

## What was verified

Every new pack came from the sealed cached metadata export; all cached pack hashes and the fixed expected totals matched. No matcher, compression or candidate-selection run was repeated. Every physical metadata record in both assembled copies was decoded and authenticated, and reconstructed canonical bytes matched the actual prior D source and unchanged sealed inventory exactly. Each delta base is a selected FULL inode leaf in an earlier pack, with the required role, identity and size bounds.

Every metadata locator maps to exactly one physical record and no metadata record is unlocated. All 78,619 content locator rows and every one of 2,180 SmallContent/native pack BLOBs compare byte-for-byte with the prior independently verified full157 copy. Therefore its authenticated content and physical membership proof carries directly, without decoding 75,398 unchanged SmallContent frames again. Every canonical object ID and canonical length remains identical across the entire Store.

Every nonphysical SQL row—including 158 branches, 157 commits, the genesis layer, names, roots, parent/head/base references and scope/highwater allocator—compares exactly with the source. No typed identity cascade is necessary because no canonical identity changes. The prior identity-map file was copied byte-for-byte, SHA256 4873a9f13f5e90c05b83c7fd9439ce5a87f7c0f2b1668398047774a190d21e58. SQLite integrity and foreign-key checks passed for both copies after one VACUUM each.

The minimal `StoreReader` subclass changes only v1metadata kind1 handling; all content parsing and bounded caches are inherited from the previous reader. A valid delta fixture passed; missing, corrupt, non-FULL, future-pack and wrong-role bases were rejected. Actual-copy reader checks covered all 158 namespace roots in both copies and 32 selected delta targets. The reader retains unsupported diagnostic schema 9301 because both canonical representation and existing v1 wire grammar are unchanged.

The source copy retained SHA256 20bc3581f29ecab3168712b8972f2eb24898f92d98e94d402657e953048efa6b. Its prior independent proof covered 157 states, 904,143 path states and 4,936,693,030 logical bytes. Exact canonical metadata/content/root equality preserves those logical states. Parent separately runs the existing 157-oracle verifier through the new reader; that result should be cited separately when complete.

## Remaining gap

This result confirms that historical metadata redundancy is a material part of the problem. Nevertheless, 98,668,544 B remains 42,295,296 B above the matched Git baseline's 56,373,248 originally recorded allocation, about 1.750 times Git. The current live Git allocation is 56,197,120 B; its difference from the recorded allocation is already documented in the previous baseline verification and was not remeasured here.

File-content packs remain 66,190,736 B, 19,208,203 B above Git blob entries. Metadata packs now exceed Git's tree/commit entries by 20,617,253 B, down from 29,909,769 B. LayerFS and Git retain different metadata semantics, so those are physical role comparisons, not claims their models are interchangeable. The tested fixed depth1 policy closes roughly 9.4 MB of the full-copy gap; it does not establish Git-like storage or product performance.

## Artifacts and scope

Final candidate: `delta.sqlite`, SHA256 2c9e44a55042ee0f7989dbfa5e7e88b09beb284ce5fbcfa7682f72d940cfd080. Control: `full.sqlite`, SHA256 a783506d0a9d3e547231ffb945a8f51324f07dac6a27e1a16350f38a67b34fcb. `result.json` uses `results['delta']` and `results['full']`; `protocol.md`, `combine.py`, `store_api.py` and unchanged `identity-map.json` are adjacent.

After construction the parent requested the namespace export alias `d_api=old.d_api`. Both exported decoder functions were asserted to be the identical functions already exposed through delta_api; no StoreReader parsing code changed. The originally executed source is retained in `reader-executed.py`, and result.json records both its hash and the final reader hash 3a1b7f5d2aa97203a087b887079a5cfe597b6d2e171f7bd1dbfe5effec8497ef for the parent's oracle pass.

These remain unsupported, compact offline diagnostic copies. This experiment does not measure online append allocation, Save/Commit latency, cache behavior, product rollback, import/concurrency or migration. No prior artifact, product source, original Store, Git repository or public comment was changed. The two-copy construction and qualification took 16.01 seconds as an offline script, not a performance benchmark.

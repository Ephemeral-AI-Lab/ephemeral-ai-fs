# One bounded metadata-delta policy saves9.39MB on the full history

The fixed depth-one experiment completed successfully. **Complete metadata pack bytes fell from35,010,506 to25,624,588, saving9,385,918 bytes (26.81%) against its matched chronological FULL control.** The D canonical inventory and all158 namespace roots are unchanged. No second policy, deeper chain, additional candidate search or anchor carry was tried.

| Metadata layout | Groups | Packs | Actual complete pack bytes |
|---|---:|---:|---:|
| Published D, global role/hash order | 5,047 | 314 | 34,917,104 |
| Matched chronological FULL control | **5,154** | **388** | **35,010,506** |
| Same chronology/groups/packs, fixed depth-one policy | **5,154** | **388** | **25,624,588** |

Chronological ordering and root-boundary flushes add93,402B compared with the published D layout. Therefore replacing that published layout with the candidate has a **9,292,516B metadata-pack difference**, while the properly matched policy effect is9,385,918B. These are compressed pack measurements including record/base references, group directories, pack headers and all unchanged metadata. They are not complete Store allocations; parent combined-layout work owns SQLite page effects and the complete original-file-oracle check.

## What was fixed before encoding

The protocol admits every canonical object once at its first use among158 ordered namespace roots, following typed metadata references. All24,748 D metadata objects are reachable; **zero objects were dropped or parked as unrooted extras**. Each root's newly admitted objects are sorted by role magic/full ID and grouped by FULL decoded sizes at16KiB. Both treatments share the exact object order, group membership and pack membership, with groups/packs flushed at root boundaries.

Only D inode leaves are eligible. For each new leaf, select **one** immediately preceding snapshot leaf with the greatest number of shared stable-serial keys among overlapping ranges; break ties by full ID. The selected origin must already have won FULL in the earlier group ledger. If it won DELTA, the target falls back to FULL: no alternative origin, older anchor, duplicate base, refresh or chain is introduced. The previous snapshot descriptor set is bounded to128 leaves; this history uses at most110, with at most100 keys per leaf.

Matching uses the product's exact `objects/pack.rs::delta_record` body through the standalone Rust FFI:4096 buckets, four16-byte seed positions, identical COPY/INSERT grammar and output/instruction limits. The diagnostic resets its16MiB matching charge budget and512-trial limit per root. This is an explicit root-scoped experimental budget rather than a claim to reproduce every product admission boundary. The FFI reuses cached BLAKE3 and was built without a product Cargo build or new dependency.

For each fixed group, the complete FULL alternative and, when available, MIXED alternative are compressed under the existing pinned metadata codec. MIXED wins only if encoded saving is at least `max(64,ceil(FULL/8))`, matching product `encode_group`. The group winner is committed to the encoding ledger only afterward. All later candidate eligibility decisions use that actual selected ledger.

## Selection results and the remaining restriction

| Outcome among8,930 new inode leaves | Leaves |
|---|---:|
| **Selected DELTA after complete-group comparison** | **4,242** |
| One chosen origin was already DELTA; required FULL fallback | **4,060** |
| No shared serial keys with the previous snapshot | 465 |
| Candidate existed, but complete MIXED group did not meet threshold | 161 |
| Matcher produced no candidate | 1 |
| Empty/genesis root had no previous snapshot | 1 |

There were4,404 matcher calls,4,403 completed candidates,2,957 selected MIXED groups and155 rejected MIXED groups. **No match-budget or instruction-budget skips occurred.** The4,060 DELTA-origin fallbacks are a measured consequence of this fixed depth-one policy. They are not evidence that relaxing it would save a particular number of bytes; no alternate policy was encoded.

The retained inventory has20,506 FULL objects and4,242 DELTA objects, including unchanged metadata roles. The DELTAs depend on4,209 distinct selected FULL leaves. Those bases'33,991,839 raw record bytes are a **subset** of the45,894,219 selected FULL record bytes; they were counted once and never added again as extra base overhead. Selected DELTA records occupy8,116,105 raw bytes before group compression. Neither raw-record subtotal is reported as allocated storage or as a per-role compressed attribution.

## Reconstruction and bounds passed

- Reconstructed **all24,748 objects from actual serialized pack/group records**, authenticated every full BLAKE3 identity, and compared the entire resulting canonical dictionary byte-for-byte with the sealed D cache.
- Decoded **all158 namespaces**, comparing exact D inode tuples and every referenced directory mapping with the sealed D inventory. Because canonical bytes and roots are identical, the previous full157 original-inode semantic proof is preserved; this run does not claim to be a new public filesystem replay.
- Verified all4,242 persisted base dependencies point to an **earlier root and an earlier physical pack**, with actual selected FULL records and the correct inode-leaf role.
- Maximum reconstructed target+base canonical closure was **16,288 bytes**, below16KiB. Each operand is at most8KiB; maximum physical delta depth is exactly1.
- Focused reader tests rejected a missing base, corrupted base, authentic wrong-role base and DELTA-as-base/depth-two case. Matcher roundtrip and zero-budget fallback checks passed.
- Reparsed and verified the entire matched FULL layout as well. The cached API independently checked all**776 pack hashes** across both layouts and all24,748 selected reconstructions.
- The loaded Zstandard1.5.7 library hash matches the prior exact-original-group equivalence run. Its full setter sequence and bounded static workspace remain unchanged. Maximum static codec workspace was218,136B. Product source and all prior artifact seals, including the original Store hash, are unchanged.

`environment-physical-audit.json` records the persisted chronology/FULL-base audit and codec-library identity. `states.json` records per-root verification; `ledger.csv` records every eligible leaf's chosen origin, intersection, candidate/selected sizes and final outcome; `groups.json` records both whole-group costs and actual decisions. The append-only `attempt-01.log` is the sole completed attempt; no failed/rejected treatment was omitted.

Diagnostic execution took31.30seconds, with8,266 codec calls and4,404 matcher calls. These are diagnostic costs, not Save/Commit benchmarks. The analysis holds the complete canonical inventory and pack alternatives in memory; the8KiB/16KiB bounds describe per-target authenticated operands/closure, not total diagnostic-process RAM.

## Reuse and decision

`delta_api.py` exposes unchanged D inventory/root/identity mappings, actual `full` and `delta` pack bytes/locators, the persisted winner ledger and strict metadata record decoder. `cache.sqlite` contains the two new diagnostic pack layouts and their common locators, plus selected records; it is not an original Store copy. Combined-layout work received these exports and can preserve canonical and typed history IDs while changing only physical metadata representation.

**This tested policy materially improves full-history metadata while retaining depth-one reads.** It does not resolve the full Git gap, and no additional candidate/depth policy was tested. The concrete next decision is based on the complete combined allocation and independent157-state file-oracle verification, rather than extrapolating from raw duplicate bytes or the4,060 skipped origins.

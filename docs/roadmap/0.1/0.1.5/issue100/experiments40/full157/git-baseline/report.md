# Matched Git full157 baseline

**The existing Git baseline represents the same 157 snapshots and passes read-only live verification.** The recorded allocation is 56,373,248 B; the live files currently occupy 56,197,120 B. The 176,128-byte allocation difference is filesystem lifecycle drift: apparent bytes 56,170,070, pack bytes 51,989,900, index bytes 3,083,340 and nine files all exactly match the recorded result. All live file hashes/stat values were unchanged before and after this investigation. No repack, checkout, rebuild or repository write occurred.

All 157 index/source-SHA/tree entries in mapping.json match both the original checkpoint manifest and the retained LayerFS fixture. Manifest SHA is 03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271; source tip is b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed. Actual synthetic Git commit objects point to the mapped trees and form the expected 157-commit chain. Live `git verify-pack -v` passed; the 110,081 objects are exactly the 109,924 snapshot objects plus 157 synthetic commits.

| Component | Bytes |
| --- | ---: |
| Blob FULL entries | 35,244,143 |
| Blob delta entries | 11,738,390 |
| **All file-content pack entries** | **46,982,533** |
| Tree FULL entries | 3,138,693 |
| Tree delta entries | 1,837,582 |
| Commit entries | 31,060 |
| **All tree/commit pack entries** | **5,007,335** |
| Pack header and checksum | 32 |
| SHA1 v2 pack index | 3,083,340 |
| Other Git files, apparent bytes | 1,096,830 |
| **Total apparent bytes** | **56,170,070** |
| Live allocation difference | 27,050 |
| **Current live allocation** | **56,197,120** |
| **Originally recorded allocation** | **56,373,248** |

The index is exactly 1,072 fixed bytes + 28 bytes per object. Maximum observed delta depth is 50 for blobs and 36 for trees. The index uses 20-byte SHA1 identities, while LayerFS keeps 32-byte canonical hashes. Git's saved command performs an offline global repack (`window 10`, `depth 50`); this differs from LayerFS online append selection and its bounded dependency rules.

Artifacts: `verify.py`, `verify-pack.txt`, `result.json`, `protocol.md`. The result's `verify_pack_reported_size_bytes` must not be read as canonical reconstructed bytes for delta entries; Git reports the delta instruction stream size there. Packed-byte attribution above uses the distinct on-disk size field. The field label was corrected after the successful read-only pass without rerunning Git commands.

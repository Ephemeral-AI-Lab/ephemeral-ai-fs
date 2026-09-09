# Full157 combined result: 107,958,272 bytes

**The full157 offline combined copy is 107,958,272 bytes. The fixed treatment improves storage, but remains far from the matched Git baseline of 56,373,248 originally recorded allocated bytes.** The ten-state result did not predict the long-history ratio: D's fixed FULL-only metadata pages still occupy 34.9 MB, and file-content packs still occupy 66.2 MB.

| Layout | Logical/apparent bytes | Allocated bytes | Scope |
| --- | ---: | ---: | --- |
| Retained LayerFS performance census | 130,990,080 | 134,246,400 | Previously measured online-built Store |
| Same full source, one fresh VACUUMed control | 129,937,408 | 129,937,408 | Matched offline compact control |
| **D metadata + framing B + fixed full157 CDC graph** | **107,958,272** | **107,958,272** | Complete unsupported diagnostic copy |
| Matched Git, originally recorded | 56,170,070 | 56,373,248 | Existing offline global Git repack |
| Same Git files, live read-only verification | 56,170,070 | 56,197,120 | Allocation lifecycle drift; bytes/membership unchanged |

The final combined copy saves **21,979,136 B / 16.92%** versus the equally compact LayerFS control. It is **51,585,024 B above recorded Git allocation**, about **1.915 times** that baseline. Do not claim the difference from the online-built performance census as an online optimization result: compaction and filesystem growth differ.

## Exact combined storage accounting

| Final component | Bytes |
| --- | ---: |
| SmallContent packs, grammar B | 58,979,700 |
| Native CDC packs, selected fixed graph | 7,211,036 |
| D metadata packs | 34,917,104 |
| **All pack BLOB bytes** | **101,107,840** |
| Full32-byte object-hash index | 4,820,992 |
| Pack-table SQLite overhead above BLOB bytes | 1,902,464 |
| All remaining SQL tables/indexes/schema, including allocator | 126,976 |
| **All SQLite outside pack BLOB payloads** | **6,850,432** |
| Allocation difference | 0 |
| **Complete copy** | **107,958,272** |

There are **103,367 selected objects / 2,494 packs**: 24,748 metadata objects in 314 packs, 75,398 SmallContent objects in 1,663 packs, and 3,221 native objects in 517 packs. The scoped-inode allocator is included as a 4,096-byte table; its highwater is 52,724. Full hash width, unchanged content canonical IDs/lengths/locators, branch records and all root/head/parent relationships are charged.

Pack reductions are exact: D metadata saves 14,570,682 B versus original metadata packs; framing B saves 1,507,960 B (20 bytes × 75,398 objects); the fixed CDC graph saves 909,266 B. Their total intrinsic pack saving is 16,987,908 B. The full-database result also reflects smaller locator indexes and page geometry; it does not add separate optimistic index projections.

## Why the gap to Git remains

The independently verified Git pack contains **46,982,533 B of blob entries**, **5,007,335 B of trees/commits**, and a 32-byte header/checksum. LayerFS's final file-content packs occupy **66,190,736 B**, a **19,208,203-byte gap**. D metadata occupies **34,917,104 B**, a **29,909,769-byte gap** against Git's tree/commit entries. LayerFS metadata semantics are richer than Git trees, so this is role attribution rather than a claim the representations are interchangeable.

D's scoped inode IDs and inline records reduce locator count substantially, but the fixed D treatment repacks canonical metadata pages as FULL records. Across 158 namespace states it produces **80,160,065 canonical metadata bytes**, slightly more than the original **79,372,108 B**, and then compresses them into 34.9 MB. It does not exploit historical similarity using metadata-page deltas. That limitation is much more visible over 157 snapshots than over ten.

Framing B deliberately retains every compressed SmallContent frame, so it cannot improve the remaining frame compression. The tested CDC family saves 0.909 MB; its ceiling on this workload is measured. Git's global offline repack permits blob delta depth 50 and tree depth 36, while the preserved LayerFS limits are SmallContent depth 8 and native depth 4. Git can also choose bases that chronological publication cannot use. Deleting SQL indexes would not solve the problem: the final pack payload alone exceeds 101 MB.

The next evidence-supported physical experiment is bounded historical deltas for D metadata pages, preserving the new scoped inode semantics; another is the remaining file-content frame gap. These are follow-up directions, not achieved savings or product implementation approval.

## Verification completed

The source was opened immutable read-only and retained SHA256 `f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7` before and after. Metadata reused the authenticated full158 export rather than rerunning its construction. Every physical D metadata record in the assembled copy was authenticated and compared with the exact exported inventory. All 158 namespace table/directory/root-inode/reference checks passed, preserving the supplied metadata semantic checksums.

All **75,398 SmallContent targets and their dependencies** decoded and authenticated; every diagnostic pack restores its original v3 bytes exactly. Maximum depth was 8, canonical closure 523,366 B, original encoded closure 71,494 B, and static decoder/dictionary workspace 123,336 B. The workspace number excludes output buffers and Python corpus structures and is not an RSS claim.

All **3,221 native targets** authenticated after the fixed 1,347-object family substitution. Maximum depth was 4, raw closure 163,840 B, encoded read work 39,667 B and decoded read work 173,899 B. All existing group/pack layouts fit; no hidden split, locator or pack charge was omitted. Every one of the 78,619 content locator rows remains unchanged.

Every selected locator corresponds to exactly one physical record, and every physical record is selected. SQLite `foreign_key_check` and `integrity_check` passed. The original genesis LayerId and all 157 CommitIds were verified against the repository derivation, then rederived with the changed namespace roots and rewritten through every dependent SQL reference in one deferred-FK transaction. The 158 branch records, names and random branch/stack identities were preserved. The root/layer/commit map is recorded; **external LayerId/CommitIds change with the canonical metadata format**.

`store_api.py` exposes `StoreReader(path).read_canonical(id)` against the actual combined SQLite file, with bounded 32 MiB canonical and 8 MiB pack payload caches by default. It uses actual physical record decoding and authentication rather than returning the external metadata inventory. Its self-check passed 182 objects, including all 158 namespace roots, with smaller enforced cache budgets. The parent task separately owns independent full file/type/mode/symlink comparison for all157 original snapshot oracles; that result must be reported separately rather than inferred from this storage script.

## Provenance and limits

One initial attempt reached the final category assertion after physical authentication but failed because the combining script expected a top-level CDC export key. The actual full export stores the value under `summary.simulated_v2_pack_bytes`. A minimal reproduction confirmed the mismatch; only that lookup was corrected. The incomplete copy and failure note were retained, and the corrected qualification was executed. No encoding, selected graph or protocol parameter changed. The final successful construction/qualification took 97.93 seconds as a diagnostic script, not a Save/Commit benchmark.

The final file is `D-B-CDC.sqlite`, SHA256 `20bc3581f29ecab3168712b8972f2eb24898f92d98e94d402657e953048efa6b`. It uses unsupported diagnostic schema9301 and framing magic. It is not a current product Store and was never passed to current product open, rollback or historical checkout APIs. Online append allocation, latency, migration, concurrent scoped-ID allocation and import/branch semantics still require product work. No product build, Docker run, Cargo/runtime change, source Store mutation or ten-state result alteration occurred.

Artifacts: `protocol.md`, `combine.py`, `store_api.py`, `baseline.json`, `result.json`, `identity-map.json`; the matched Git report and raw attribution are in sibling `git-baseline/`. The incomplete reporting-key attempt is explicitly excluded from the final result.

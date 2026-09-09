# Saved prototype: full157 offline experiment versus Git

2026-09-10. The owner selected extending the offline prototype to all157 states before product integration. The ten-state prototype was first saved as commit `5636fbc43` on `codex/issue100-40mb-experiments`. Its original outputs and measured41,648,128-byte copy remain unchanged.

**The fixed prototype occupies107,958,272 bytes for all157 states. Every state passed exact original-oracle verification. It remains51,585,024 bytes larger than Git's recorded56,373,248-byte result (1.915× Git).** The ten-state near-Git result does not extend to this longer history.

These are unsupported offline diagnostic formats. There was no product integration, public Save/Commit benchmark, FUSE checkout campaign, Docker run or release claim. Original Stores and the sealed ten-state evidence were not changed.

## Exact full-history results

| Full157 artifact / boundary | Logical or apparent B | Allocated B |
| --- | ---: | ---: |
| Previously measured retained LayerFS product | 130,990,080 | 134,246,400 |
| Same retained source, fresh offline VACUUMed control | 129,937,408 | 129,937,408 |
| **Fixed D metadata + compact framing B + CDC similarity** | **107,958,272** | **107,958,272** |
| Git's originally recorded packed repository | 56,170,070 | 56,373,248 |
| Same Git files during this read-only verification | 56,170,070 | 56,197,120 |

The prototype saves **21,979,136 bytes (16.92%)** against the matched compact LayerFS control. Do not describe the larger difference from the earlier134,246,400-byte public result as an isolated implementation gain; lifecycle, compaction and filesystem allocation differ.

Git's actual matched baseline is about56.37decimalMB, rather than exactly60MB. Its current apparent bytes, pack/index bytes, nine files and exact object membership match the recorded baseline. Current allocated blocks are176,128bytes lower. Both recorded and live allocation observations are retained; no old receipt is rewritten. Main comparisons below use the recorded baseline.

| Physical category | Full157 prototype B | Recorded Git B | Difference B |
| --- | ---: | ---: | ---: |
| Content packs / blob entries | 66,190,736 | 46,982,533 | +19,208,203 |
| Metadata packs / tree and commit entries | 34,917,104 | 5,007,335 | +29,909,769 |
| Indexes, other structures and allocation | 6,850,432 | 4,383,380 | +2,467,052 |
| **Total** | **107,958,272** | **56,373,248** | **+51,585,024** |

The prototype's content consists of **58,979,700 SmallContent pack bytes +7,211,036 native CDC pack bytes**. Total pack payload is101,107,840bytes. Add6,850,432SQLite nonpack bytes and zero copy allocation difference to obtain107,958,272exactly. All103,367objects and2,494packs are accounted for. The full-hash object index occupies4,820,992bytes and the scope allocator4,096bytes; both are already included in nonpack bytes.

The index/database is not the principal obstacle. Metadata and content payload explain49,117,972bytes of the51,585,024-byte allocated gap. Even eliminating every SQLite overhead byte would leave101.11MB of pack payload.

## What was held fixed

The exact ten-state policies were extended, with no tuning to obtain a better157-state number:

- **D metadata:** inline73-byte inode values, direct directory mappings, scoped eight-byte stable inode serials, full32-byte authenticated content/object IDs, the same page capacities, FULL-only role/ID-sorted offline groups and pinned metadata codec.
- **Framing B:** four-byte SmallContent group starts and inferred redundant record lengths, preserving every existing compressed SmallContent frame and full base hash.
- **CDC:** one previous-file minhash candidate alongside the original first-overlap candidate, fixed128-entry/1MiB index bounds, unchanged codec/depth/closure limits, chronological graph updates, and all new FULL fallback costs included.

Input source is the previously verified retained-full157 Store at `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/host-runtime/store.sqlite`, post-verification SHA256 `f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7`. It is a different documented lifecycle from the frozen performance digest. The full157 fixture manifest remains `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`, pinned source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`.

## Why the longer history is much farther from Git

**Inlining is less favorable as retained history grows.** Original metadata has89,576distinct inode records across this history. Inlining repeats their values in changed inode-table pages. Total canonical metadata grows from79,372,108 to80,160,065bytes, despite removing104,358metadata objects. Compact IDs and compression still reduce physical metadata, but do not eliminate the repeated historical values.

The actual physical D leaves contain885,543inode-value occurrences but only89,576distinct values: **9.886copies per value**, versus **1.579copies per value** in the ten-state experiment. Occurrences grow15.04× while distinct values grow only2.40×. This is measured duplication, not a claim that every repeated canonical byte can be removed from compressed storage. The [separate scaling diagnosis](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/metadata/scaling-diagnosis.md) also records the original4,673metadata deltas; the fixed D packing policy does not use corresponding historical metadata deltas.

| Metadata metric | Original full-history source | Fixed D |
| --- | ---: | ---: |
| Metadata objects | 129,106 | 24,748 |
| Canonical metadata B | 79,372,108 | 80,160,065 |
| Actual metadata packs B | 49,487,786 | 34,917,104 |

D saves14,570,682actual metadata pack bytes (29.44%), much less proportionally than the ten-state result. The diagnostic D format uses independently FULL/group-compressed pages; it has not implemented bounded historical metadata deltas. The source implementation's existing selected metadata deltas and its online ordering are distinct from this fixed diagnostic policy. We cannot assume a new delta policy will recover a particular percentage without encoding it.

**The remaining content gap also grows.** The small-file frames are unchanged by this experiment; compact framing saves1,507,960bytes across75,398SmallContent objects. A conclusion about near-parity for the ten-state33,217-object population cannot be applied to this larger population. The actual Git full-history pack uses blob delta depth up to50 and tree delta depth up to36; that is evidence of different physical graphs, not proof LayerFS should copy those limits or that they alone explain the gap.

**CDC similarity helps, but is not the major remaining lever.** The same policy was applied to1,347lockfile chunk identities across156large-file checkpoint states; checkpoint1 remains SmallContent. Record bytes fall2,968,670→2,059,404, a net909,266-byte saving. There are433improved,133worsened and781unchanged records. All130newly FULL records, totaling592,845bytes, are included. Original native pack/group memberships still fit, so native packs shrink8,120,302→7,211,036bytes without splits or locator reordering.

The142previous-file indexes require11,199dependency lookups,29,484,509encoded-work bytes and270,712,900decoded-work bytes. Python/ctypes index construction took60.03seconds; this is diagnostic work, not a public Commit timing result. Product discovery overhead remains unqualified.

## Verification: all157 original oracles passed

The combined-copy constructor checked every physical record, full canonical identity, dependency bound and locator. It verified the original typed identities, rederived one genesis layer and157CommitIds after metadata root changes, and rewrote parent/base/head/root references. External historical IDs change with the new canonical metadata format; snapshot semantics remain comparable through explicit mappings.

The independent verifier then read **the assembled candidate copy**, resolved its new commits and traversed each namespace against the original fixture oracles. It did not substitute source bytes for candidate reads or rebuild a second history.

- **157/157 snapshot states: PASS.**
- **904,143path states:** exact path membership, type, mode, logical length and SHA256 match.
- **4,936,693,030logical bytes:** covered by the original per-state oracles.
- All directory and inode-reference counts agree; symlink targets match, with actual0777link permissions checked before normalizing the original Git-style120000oracle mode.
-75,922distinct regular-file roots were hashed from actual candidate content, totaling891,893,067bytes. Reused file digests are keyed only by identical authenticated content roots; every state's complete namespace and metadata are still checked.
- Decoded canonical cache bounded to4MiB and encoded-pack cache to8MiB, plus locator and digest metadata. This is not a process-RSS or public performance measurement.
- Original metadata129,106objects and6,296compression groups authenticated/reproduced byte-for-byte before D encoding; all158namespace states including initial empty state and14,429directory objects passed the metadata transformation checks.
- All75,398SmallContent objects/dependencies and3,221native objects authenticated. SQLite integrity and foreign keys passed.

The complete original-oracle pass took112.13seconds in this standalone verifier. The final copy SHA256 was identical before and after: `20bc3581f29ecab3168712b8972f2eb24898f92d98e94d402657e953048efa6b`.

All157oracle SHA256 values used by that completed pass were then checked against the **original saved fixture identity**, including checkpoint/source identities and logical sizes. The custody check passed without rerunning the candidate verification. See [oracle custody](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/verification/oracle-custody.json).

## Git baseline applicability

Read-only checks verified all157source/tree mappings, the linear synthetic commit ancestry, no alternates, exact110,081-object membership and the live pack. The saved packing settings remain compression6/window10/depth50/threads2; no Git baseline was rebuilt or repacked. Git preserves a narrower metadata scope and packs offline. No comparison of Git repack timers to LayerFS Save/Commit latency is made.

Git payload attribution:46,982,533blob-entry bytes,5,007,335tree/commit-entry bytes,32global pack bytes,3,083,340primary-index bytes,1,096,830other apparent repository bytes. Recorded allocation above apparent length is203,178bytes; live allocation above apparent length is27,050bytes. Both totals reconcile.

## Disposition

Keep the saved prototype and evidence as a demonstrated space improvement, but **do not call this full-history Git parity or extrapolate the ten-state41.65MB result**. The full157 experiment is useful precisely because it exposed the historical-page cost before product integration.

The first next hypothesis is bounded metadata deltas for the new pages, with a fixed prior-base rule and complete retained-base/group accounting. A hybrid retaining independently shared inode values can be compared afterwards, but it adds CAS/index rows and its raw-byte reduction is not a compressed-storage forecast. These are untested designs, not promised savings. Content needs a separate full-history graph attribution; another index cleanup cannot address the observed payload gap. No new policy or additional optimization run was performed in this comparison.

Product readers/writers, transactional allocation, foreign-scope imports, compatibility/migration, rollback, public Save/Commit latency and historical API qualification remain outside the selected offline task.

## Saved artifacts

Evidence root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157`.

- [Combined layout report](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/combined/report.md) and [exact result](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/combined/result.json).
- [Independent157-state oracle result](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/verification/result.json) and [runnable verifier](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/verification/verify.py).
- [Git baseline verification](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/git-baseline/report.md).
- [Metadata result](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/metadata/report.md) and [CDC result](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/cdc/report.md).

The first combined construction completed physical/authentication checks but failed a reporting assertion because the full-history CDC export nested its pack total differently. That incomplete copy/source/receipt was retained. The corrected final qualification changed only the reporting-key access, not the representation or chosen frames. The CDC diagnostic also retained a failed import-wiring attempt. Neither is hidden as a successful result. All original input data and sealed ten-state evidence remain intact.

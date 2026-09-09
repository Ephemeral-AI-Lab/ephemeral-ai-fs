# Shared inode values: physical metadata experiments — 2026-09-10

Priority1 metadata only. Both experiments preserve every original metadata canonical ID, content ID and content pack, and all typed SQL history. These are offline formats, not supported public Store operations. The exact same rules run on53and157states; content and general index optimization did not run here.

## V1: one logical pooled-value object and ordinal locator per distinct value — rejected

| Actual bytes | 53 states | 157 states |
|---|---:|---:|
| Improved matched baseline database | 59,760,640 | 79,790,080 |
| V1 complete database | 66,527,232 | 83,488,768 |
| **Regression** | **6,766,592** | **3,698,688** |
| Original metadata packs | 6,877,152 | 17,459,061 |
| Shared-reference leaf/other metadata packs | 2,458,109 | 6,156,885 |
| Pooled value packs | 2,545,576 | 3,446,289 |
| Distinct values | 66,529 | 89,576 |
| Inline value occurrences | 306,115 | 885,543 |

Each physical leaf stores8-byte stable serial plus4-byte value ordinal. V1 indexes every pooled94-byte canonical object by full32-byte ID in the ordinary objects table, then additionally stores ordinal→fullhash rows with a UNIQUEfullhashindex. Its payload savings are real, but these extra indexes dominate. Exact allocated-page changes versus matched baseline:

| SQLite component change | 53 states | 157 states |
|---|---:|---:|
| object_packs | −1,810,432 | −7,852,032 |
| objects index | +3,104,768 | +4,173,824 |
| ordinal table | +2,748,416 | +3,698,688 |
| ordinal UNIQUE fullhash index | +2,723,840 | +3,678,208 |
| **Total** | **+6,766,592** | **+3,698,688** |

This rejects V1's representation, not shared values in general. The157extension was declared after observing initial53allocation and before157encoding, because repeated-value reuse rises4.60→9.89times; no parameters were changed. Every original canonical metadata object and every pooled value authenticates; all54/158namespace inode and directory semantics pass. Content locators/packs and all typed SQL rows remain byte-for-byte equal. Missing ordinal, remapped ordinal and corrupt pool rejection fixtures pass on both copies. Exact canonical equality plus unchanged content establishes the rejected format's content preservation without another full path-oracle campaign.

## V2: one authenticated physical catalogue row per value group

A separate prospective policy removes V1's artificial per-value logical CAS/index rows. V2 retains **exactly the same physical packs, ordinals, leaf encodings and DELTA graph**. An indexed pool_groups table stores firstordinal, count, packID, groupnumber and full32-byte SHA256 of each independently compressed framed group. Resolveordinal by range lookup and recordslot; authenticate its entire group and then the full original canonical leaf. All catalogue/index bytes count. Ordinary metadata/content IDs and locators remain unchanged; catalogue-covered records must be complete and disjoint from ordinary CAS records.

| Actual bytes | 53 states | 157 states |
|---|---:|---:|
| Improved matched baseline database | 59,760,640 | 79,790,080 |
| V2 complete database | 57,974,784 | 71,970,816 |
| **Physical saving** | **1,785,856** | **7,819,264** |
| Physical pool catalogue rows | 404 | 543 |
| Catalogue allocated bytes | 24,576 | 32,768 |

The above is complete physical allocation including ordinary fullhash CAS indexing and the new per-group catalogue; no pooled bytes or retained DELTA bases are omitted. Internal canonical/namespace completion status is recorded in each result, and independent path-oracle acceptance belongs to the parent campaign.

Both V2 copies now pass every original canonical metadata object, every physical value, all54/158namespace semantics and complete catalogue/CAS membership checks. Six negative fixtures pass on each copy: missinggroup, wrongordinal, badgroupdigest, corruptgroup, wrongvaluerole and fullcanonicalleafhash. V2-53 candidate SHA256 is `54f47590d3f1189395b109901ab6b403c45074a9634cf7be3e28384365dfdb59`; V2-157 is `96cfd0ea89d7e3cfe85940ddbc863f1f417e86d8b86f6aee3cc0c79154b15825`.

See [V2-53 result](v2-53/result.json) and [V2-157 result](v2-157/result.json) for complete physical results. Parent owns the independent original path-oracle verification and final retention decision. Any storage improvement here remains subject to the cold-read cost below.

## Cold-read cost is a material limitation

Hash-sorted value groups scatter one reconstructed leaf's values across many independent groups. V2 intentionally retains V1 geometry. Parent's exact deduplicated dependency accounting measures these worst cold-chain costs:

| Cold chain requirement | 53 states | 157 states |
|---|---:|---:|
| Decoded pooled-value group bytes | 4,705,632 | 5,765,786 |
| Decoded metadata plus value group bytes | 4,725,604 | 5,785,404 |
| Distinct pooled-value groups | 288 | 353 |
| Distinct pooled-value packs | 26 | 34 |
| Maximum distinct whole-pack fetch bytes | 2,854,074 | 3,744,557 |

These are several megabytes per cold leaf chain, substantially above the preceding metadata layout. They are separate from the192KiB **logical value-byte** bound, which does not bound decompression-group or whole-pack amplification. A cached sequential oracle pass does not establish acceptable random-read performance. V2 reader uses4MiB canonical and8MiB encoded-pack payload budgets. Its pool cache accounts4MiB of decoded group bodies but also retains extracted record byte copies, so pooled cached byte payload can approach8MiB total (4MiB bodies plus up to4MiB records), before Python container overhead. This pool cache is additional to the preceding reader;4MiB is not its total memory bound. No public latency claim follows from these results.

## Reproduction and custody

- Raw V1-53: this folder. Source improved Store SHA256 `ff2eb0bbff39cb822f5fe33514dd84adce817f2d86f4041296220211ba91a8f4`; candidate `5f22c40374474df50136ed9fb927975693c7509fca4eb337d5a91e11e5942eb0`.
- Raw V1-157: `full157/`. Source improved Store SHA256 `e5837e3484d561225fa464433ed94410705bdb6cde1f60b2c057faa834a73281`; candidate `5b2725b002823c5e8624b315fbc087ab8f98a0cd1baabece9e7e08232760b170`.
- Raw V2 copies: `v2-53/` and `v2-157/`. Each has prospective protocol, build.py, reader.py, result and negative-check evidence. Builders refuse existing outputs. Run in a fresh artifact directory on this machine with the original sealed dependencies.
- `dependencies.json` and per-experiment manifests preserve source/decoder/codec/matcher provenance. Both V1 baseline hashes remained unchanged. V2 baseline hashes are pinned to validated V1 results; packs are identical.
- Parent cold-work artifacts: [53-state accounting](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/metadata-cold-53.json) and [157-state accounting](/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/metadata-cold-157.json).

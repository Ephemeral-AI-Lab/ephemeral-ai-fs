# Structural content investigation: graph quality matters more than isolated reset wins

**A globally selected dependency graph saves 5,127,863 B on all 75,398 SmallContent records while retaining the existing eight-edge/512-KiB/256-KiB reconstruction bounds.** Allowing the longer original Git graph saves another 4,675,282 B. Merely choosing an older eligible same-path version or reversing chronological order loses space after the whole graph is counted.

These are measured, independently authenticated **encoded-record** results. They are not allocated LayerFS Store sizes. The bounded result exports the original canonical identities and selected frames for the combined-copy investigation; the parent experiment must rebuild packs, retain index rows, and verify all 157 original content/metadata oracles before claiming a complete-copy saving.

## Complete graph results

All rows include compressed frames, one kind byte per object, every 32-byte delta base reference, and four directory bytes per object. Pack headers and database/index costs are excluded consistently. The current compact SmallContent packs are 58,979,700 B including 26,608 B of pack headers; their matched record/directory subtotal is **58,953,092 B**.

| Graph | Objects | Frames B | Record + directory B | Saving versus current matched subtotal | Maximum edges | Maximum canonical closure B |
|---|---:|---:|---:|---:|---:|---:|
| Current LayerFS | 75,398 | 56,463,014 | 58,953,092 | — | 8 | ≤524,288 |
| Forward same-path eligible ancestor + eligible original physical base | 75,398 | 67,323,316 | 69,828,498 | **−10,875,406** | 8 | 524,260 |
| Reverse same-path eligible ancestor + eligible original physical base | 75,398 | 58,118,970 | 60,439,704 | **−1,486,612** | 8 | 524,252 |
| Git-selected graph, current bounds restored through nearest eligible original-graph ancestor | 75,398 | 51,424,591 | 53,825,229 | **5,127,863** | 8 | 523,950 |
| Git-selected graph, extended bounds | 75,398 | 46,749,309 | 49,149,947 | **9,803,145** | 41 | 3,270,825 |

Every complete saved SmallContent graph reconstructs and authenticates **75,398 identities / 739,270,815 unique raw bytes**. Verification checks both Git blob SHA1 and original LayerFS canonical BLAKE3. The maximum encoded closure for the bounded winning graph is **71,494 B**. The extended small graph reaches 89,597 B. Encoded closure is not the dominant restriction in these trials.

The globally selected graph is taken from the already fixed Git reference. This demonstrates a representation/selection opportunity; it does not implement an independent LayerFS candidate discovery algorithm. The bounded graph still uses future-dependent bases and needs offline physical replacement/topological repacking. It is not a drop-in online admission policy.

## Why the isolated ancestor experiment was misleading

The forward trial saves **6,723,874 B** on the exact 1,528 objects previously classified as FULL because their preceding SmallContent graph exceeded a structural bound. That looks compelling in isolation. Across the complete graph, however:

| Original → selected representation | Objects | Net saving B |
|---|---:|---:|
| FULL → DELTA | 1,756 | 7,558,441 |
| DELTA → FULL | 1,284 | −1,541,182 |
| DELTA → DELTA | 64,750 | **−16,892,665** |
| FULL → FULL | 7,608 | 0 |
| **Whole population** | **75,398** | **−10,875,406** |

Avoiding FULL resets by repeatedly selecting a shallow, older base accumulates larger descendant deltas. The complete measurement disproves summing the attractive reset savings as an expected net gain for this policy.

Reverse order also has attractive isolated wins: FULL→DELTA saves 19,787,602 B. But displaced DELTA→FULL anchors cost 24,976,797 B, offsetting that and the 3,702,583 B improvement among DELTA→DELTA objects. Net result is a **1,486,612 B regression**. Direction alone does not capture Git's base selection quality.

For the bounded Git-selected graph, all opposing effects are counted: FULL→DELTA saves 20,330,111 B, DELTA→FULL costs 24,738,616 B, and DELTA→DELTA saves 9,536,368 B. The actual net is **5,127,863 B**, not the much larger isolated FULL conversion figure.

## How much do reconstruction limits cost?

The two Git-small trials share the exact canonical population and original Git DAG. The bounded trial traverses that original DAG to the nearest already-selected ancestor whose newly selected closure fits the original depth/canonical limits, then applies the same cost and encoded-closure check. There are **50,788 ancestor traversal steps** over the complete run. It does not search for the best compressed candidate among all ancestors.

Restoring current bounds costs **4,675,282 B** relative to keeping the extended graph. Both select 12,159 FULL and 63,239 DELTA records; in this experiment the difference comes from less similar shallower bases rather than additional FULL resets. This is a measured tradeoff for these fixed graph policies, not a universal lower bound or proof that larger reader limits are necessary.

Next useful work is to discover a comparably good graph without Git's precomputed choices, and compare bounded candidate selection against an offline compaction implementation. Expanding a candidate search should earn its encoding/read costs by closing a meaningful fraction of the remaining graph difference. Repeating narrow reset experiments without complete descendant accounting is low priority.

## Whole-file and representation-boundary reference

The separate `git-all` trial includes all **75,929 Git blobs**, using whole-file prefixes even for the 523 large regular-file versions. It also counts the seven symlinks and empty blob; their metadata placement in LayerFS differs.

| Selected graph component | Frames B | Record + directory B |
|---|---:|---:|
| Original 75,398 SmallContent identities | 45,856,792 | 48,258,486 |
| Other 531 blob identities | 2,283,675 | 2,302,266 |
| **All content** | **48,140,467** | **50,560,752** |

Every saved record was reconstructed and Git-authenticated: **75,929 identities / 891,893,320 unique bytes**, including the original canonical BLAKE3 checks for all SmallContent identities. The graph reaches **50 edges, 29,808,550 B canonical closure, and 483,553 B encoded closure**. The whole-file trial models closure with raw length plus 23 B per node; a new large-file canonical format is not specified by this diagnostic. Its largest compressed frame is **483,544 B**, exceeding the current 256-KiB pack bound. This is a structural reference with substantial read/format changes, not a runtime-compatible candidate.

Allowing cross-boundary dependencies improves the SmallContent subtotal by **891,461 B** compared with `git-small`, where 33 Git bases outside the SmallContent population become FULL. The other-blob encoded subtotal is **4,908,770 B below** the existing 7,211,036 B native-pack contribution, but this comparison includes different framing and the eight nonregular/empty identities. It is evidence that large-file history deserves redesign experiments; it is not an exact independently reclaimable native-pack saving.

The all-blob encoded subtotal is **15,629,984 B below** current complete content packs of 66,190,736 B, before replacement pack headers, index/locator changes, and file-root mapping. Existing native chunk/map objects cannot be removed until original historical file roots are redirected to a verified replacement representation. **Do not subtract 15.63 MB from the Store size or add the large-file number to the bounded-small saving as a validated combined outcome.**

Git's corresponding blob entry total is 46,982,533 B. Even on its actual graph, the current zstd prefix frames alone total 48,140,467 B, **1,157,934 B more**, before our 2,420,285 B record/directory fields. Thus a better graph closes much of the content gap, while codec/framing differences remain after graph alignment. The whole-file reference is still not a like-for-like physical Store comparison.

## Protocol, checks, and reproducibility

- Initial policy was frozen in [protocol-initial.md](protocol-initial.md); the bounded follow-up was frozen after the two simple graph policies lost in [protocol.md](protocol.md).
- [experiment-initial-executed.py](experiment-initial-executed.py) preserves the initial four-mode implementation. [experiment.py](experiment.py) includes the bounded follow-up. Candidate lists are deterministic: path histories in path order, nearest historical occurrence first, duplicate candidate identities removed; same-step admission ties use Git OID order. This is not exhaustive candidate search.
- [codec-check.py](codec-check.py) reproduces every 100th existing physical SmallContent record: **754/754 byte-exact matches**, comprising 89 FULL, 79 legacy delta, and 586 chain delta frames. Pinned zstd 1.5.7 requested parameters match the existing writer. Dynamic allocation in this diagnostic does not establish runtime scratch-memory equivalence.
- Saved-record graph verification is independent of the source-byte compression cache. A 64-MiB decoded payload LRU bounds that cache; it is not a process-RSS cap. No source Store or Git mutation is performed. All original 157 manifest seals are checked during input enumeration.
- Per-mode execution including authentication took 59.37 s forward, 54.96 s reverse, 61.58 s extended Git-small, 68.16 s Git-all, and 51.74 s bounded Git-small. These are diagnostic timings, not Commit or read-latency benchmarks, and two follow-up processes overlapped.
- [manifest.json](manifest.json) records all artifact checksums/results and exact source custody. The initial process computed its source hash at completion after the follow-up extension had been added; the manifest explicitly supplies the preserved initial executed snapshot rather than treating that completion-time hash as its executed source.
- [comparison.json](comparison.json) and [summarize.py](summarize.py) retain whole-population transitions, capped subsets, and largest positive/negative path families. All negative policy results remain recorded.

Raw artifacts: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural/content/`. Each `<mode>.sqlite` contains `records(oid, id, base, raw, frame, depth, canonical, encoded)`. For SmallContent, `id` is the unchanged original 32-byte canonical identity; join `base` Git OID to `records.oid` to recover its canonical identity. Emit FULL kind0 or chain-prefix kind2 records using the stored frame and raw length. The bounded winning artifact SHA256 is `06cd378e1233b71429dceb2b6430ed37d323616b98ecb87f951bd42d5f6e242c`.

The SQLite export is an experiment container with redundant diagnostic keys and statistics. Its file size is not LayerFS index overhead and must not enter the Store scorecard. Product qualification, actual pack allocation, same-Store lifecycle behavior, and all157 metadata/content oracles remain the responsibility of the combined-copy experiment.

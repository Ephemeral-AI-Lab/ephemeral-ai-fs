# Priority3: fixed1024-byte SQLite pages —53and157states

**Keep4096-byte pages for both histories.** The one fixed1024-byte experiment loses space at53states and saves only73,728allocated bytes (0.112%) at157while adding a B-tree level. Both measured copies are retained; no page-size sweep or further index rewrite was performed. All fullhashes, canonical lengths, constraints, schemas and physical pack bytes remain unchanged.

| Actual database bytes | 53 states | 157 states |
|---|---:|---:|
| Matched4096-byte control logical/allocated | 54,382,592 | 65,957,888 |
|1024-byte candidate logical | 54,385,664 | 65,882,112 |
|1024-byte candidate allocated | 54,386,688 | 65,884,160 |
| **Allocated saving (negative means regression)** | **−4,096** | **73,728** |
| Logical saving | −3,072 | 75,776 |
| Disposition | Reject | Retain measured reference; keep4096default |

## Why the page-size change does not justify adoption

| SQLite component |53:4096→1024pages |157:4096→1024pages |
|---|---:|---:|
| objects B-tree bytes |3,416,064→3,547,136 (+131,072) |4,878,336→5,075,968 (+197,632) |
| objects maximum B-tree depth below root |2→3 |2→3 |
| object_packs B-tree/overflow bytes |50,860,032→50,774,016 (−86,016) |60,919,808→60,695,552 (−224,256) |
| object_packs maximum B-tree depth below root |1→2 |1→2 |
| object_packs overflow pages |12,146→49,321 |14,363→58,791 |
| pool_groups bytes |24,576→21,504 |32,768→28,672 |

Smaller pages reduce packed-BLOB fragmentation but make the objects index larger. B-tree depths and overflow-page counts are observed SQLite layout costs; they are not measured I/O requests, RSS or public latency. No speed benefit is claimed.

## Exact logical preservation and verification scope

A deterministic typed streaming SHA256 and row count for **every table**, including every pack BLOB byte, match source, control and candidate on each history. Logical schemaSQL and all table/index/constraint definitions, user_version, application_id and encoding match exactly. Physical schema rootpage numbers are intentionally excluded because page relocation is the experiment. Both copies pass SQLite integrity and foreign-key checks; both source hashes remain unchanged.

- 53-state source already passed all53original filesystem oracles. Complete logical equality carries its content/metadata proof across nativeSQLite page relayout. The losing53candidate did not receive another read campaign.
- 157-state candidate additionally passed cold actual-reader representatives for every populated content record kind (smallFULL/prefix, largeFULL/prefix, native slice and nativeFULL), namespace/inode table and pooled-value group. It freshly matched original states **1,79and157**, including all paths, content and metadata, using the unchanged content reader. Those three oracle hashes also match the retained original full157verification proof. The selected-state campaign took16.35s diagnostically.
- The parent is responsible for completing all157original states on the4096-byte content source. **This page experiment did not freshly reread all157states.** Exact complete logical equality carries that source proof once it passes; selected-state results are not presented as all-state verification.

## Custody and reproduction

| Artifact | SHA256 |
|---|---|
|53content source |`dd4a3c7cd2260b487b570cb819f2799ff189c2470382d4481b0f2fd19f3360a3` |
|53page candidate |`6009ea777216f0afbb321fb66760e5f6f4174e2a3b1d2fd31421fa8dbe3aec09` |
|157content source |`efacb9b832961ed7a2e675c26f113545622b75df684f9c11a36f3b1a121bec11` |
|157page candidate |`0c33871669c5ab685aaf1d47270a0c35898611e25f47c64cbe635061d1fb18c3` |

Each experiment backs up the immutable source into fresh4096control and1024candidate files, applies the specified page_size andVACUUM, and refuses existing output files. Exact checks/accounting took2.01s for53and2.57s for157. No code/codec/index-policy changes or failed attempts occurred.

See [53protocol](protocol.md), [53result including table hashes/dbstat](result.json), [157protocol](full157/protocol.md), [157result](full157/result.json), [fresh reader/three-state checks](full157/reader-check.json), [oracle custody](full157/oracle-custody.json), and [default disposition](disposition.json). Raw database copies/logs remain under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ordered-optimization/index`.

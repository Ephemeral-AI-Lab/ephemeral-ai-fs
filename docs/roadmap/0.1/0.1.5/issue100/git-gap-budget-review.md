# Ten-snapshot Git gap: accounting and target review

2026-09-09. Read-only review of existing measurements; no builds, repacking,
new corpus census, payload decoding, benchmark run, or product modification.
The only new artifact from this review is this document. Decimal MB throughout.

## Measured scope

All arms retain precisely full157 snapshot indices 1, 18, 36, 53, 70, 88, 105,
122, 140, 157. The fixture records the original manifest SHA256
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271` and
selected-input provenance. Git retains ten reconstructed commits, not the
15,632 upstream commits. Both LayerFS arms report ten Created outcomes and
same-Store verification of all ten original oracles. Git reports the same
selected-tree and byte proof. Scope differences remain: Git does not preserve
all LayerFS POSIX metadata and its final allocation follows offline repacking.

Source records reviewed under
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence`:
`comparison-v014-v015.json`, `git/results.json`, `git-pack-attribution.json`,
and `fixture.json`. Lifecycle and definitions are from
[the frozen contract](ten-snapshot-contract.md) and
[the baseline report](ten-snapshot-baselines.md).

## Byte-exact accounting

| Physical category | Existing v0.1.5 bytes | Git bytes | Difference |
|---|---:|---:|---:|
| File content: SmallContent + CDC packs / Git blob records | 54,245,616 | 34,306,253 | +19,939,363 |
| Metadata/legacy packs / Git tree and commit records | 6,559,114 | 1,566,186 | +4,992,928 |
| Remaining indexes, database/repository structures, allocation | 5,300,614 | 2,351,433 | +2,949,181 |
| **Final allocated total** | **66,105,344** | **38,223,872** | **+27,881,472** |

These buckets reconcile exactly. They are useful physical comparisons, not
claims of semantic equivalence between a LayerFS inode and a Git tree.
LayerFS metadata/legacy includes five legacy chunk objects. LayerFS content
includes its pack framing while Git content is the verify-pack packed-record
size; Git's 32-byte global pack envelope is in the residual bucket.

LayerFS content details:

- SmallContent FULL frames: 30,191,026 B (14,646 objects).
- SmallContent DELTA frames: 19,361,953 B (18,571 objects).
- SmallContent record and pack headers: 1,433,145 B.
- Large CDC/PREFIX packs: 3,259,492 B.
- Sum: 54,245,616 B.

The 7,548 physical FULL bases occupy 17,300,067 record bytes already included
in the FULL total. Adding them again would double-count retention.

Git content: 28,143,165 B in 9,684 FULL blob records plus 6,163,088 B in
23,597 DELTA blob records. The difference between these totals and LayerFS
FULL/DELTA totals does not isolate a codec effect: object partition, base
selection, delta assignment, and framing differ.

LayerFS residual: 4,087,808 B locator pages + 680,326 B remaining logical
SQLite allocation above pack BLOBs and locator pages + 532,480 B filesystem
allocation above logical database length = 5,300,614 B. The 680,326 B includes
pack-table storage overhead and other tables; it is not all free slack.
Freelist count is zero. Thus VACUUM/free-page cleanup is not an evidenced
multi-megabyte solution.

Git residual: 32 B global pack envelope + 1,142,436 B primary index +
357,768 B other apparent repository files + 851,197 B filesystem allocation
above apparent length = 2,351,433 B.

File content explains **71.51%** of the allocated gap. Metadata/legacy explains
17.91%, and remaining structures/allocation 10.58%. These are allocation-gap
shares, not shares of avoidable waste.

## What reaching 40 MB requires

A 40,000,000-byte final Store requires a net reduction of **26,105,344 B**, or
39.49% of current allocation. It is 1,776,128 B above measured Git allocation.

Two counterfactuals establish the scale without predicting an implementation:

1. Make every current SmallContent DELTA frame free and hold everything else
   constant: 66,105,344 - 19,361,953 = **46,743,391 B**. A DELTA-frame-only
   optimization cannot reach 40 MB under those assumptions.
2. Match all Git blob bytes and retain current LayerFS metadata/overhead:
   34,306,253 + 6,559,114 + 5,300,614 = **46,165,981 B**. Even Git-level
   content compression requires another **6,165,981 B** of non-content savings
   to reach 40 MB. That is 51.99% of current non-content allocation.

A target budget, deliberately NOT a forecast, could be:

| Category | Illustrative budget | Current bytes | Required saving |
|---|---:|---:|---:|
| File-content packs | 34,306,253 | 54,245,616 | 19,939,363 |
| Metadata/legacy packs | 2,500,000 | 6,559,114 | 4,059,114 |
| Remaining allocation | 3,193,747 | 5,300,614 | 2,106,867 |
| **Total** | **40,000,000** | **66,105,344** | **26,105,344** |

This assumes Git-equivalent aggregate content bytes, 61.89% less metadata,
and 39.75% less residual allocation. None is demonstrated achievable under
the fixed one-level online SmallContent policy. The metadata target has more
room than Git's metadata representation, but no measured encoder yet supports
it. The residual target requires structural/index savings, not merely removal
of the 532 KB allocation-length difference.

There is no defensible narrow prediction interval around 40 MB yet. A credible
40 MB plan must explain savings in all these buckets or demonstrate content
compression materially better than Git to compensate for unchanged metadata.

## Git advantages actually present in this pack

The saved pack attribution reports:

| Blob DELTA base availability | Objects | Packed DELTA bytes |
|---|---:|---:|
| Earlier selected checkpoint closure | 2,762 | 959,661 |
| Same selected checkpoint closure | 2,231 | 683,969 |
| Later selected checkpoint required | 18,604 | 4,519,458 |

**78.84% of Git blob DELTA records require a complete base closure containing
an object first available in a later selected checkpoint**; those records
account for 73.33% of Git DELTA blob packed bytes. Availability recursively
takes the latest first-seen checkpoint across the complete dependency chain;
this does not mean the direct base itself necessarily first appears later. Maximum observed blob depth is
23; tree depth is 7. Ten selected snapshots do not limit Git chains to ten:
its candidate relationships need not follow one path's chronological versions.

This is strong evidence that offline access to later versions and a broader
base graph materially participates in the measured layout. It does not quantify
how many bytes would be lost if those choices were forbidden: that requires an
alternative encoding. It also does not prove depth 23 is necessary; a different
bounded representation could achieve similar bytes.

For append-growing files, storing a later large version FULL and representing
older versions with copy-only or mostly-copy DELTAs can avoid retaining the
appended literals many times. Forward old-FULL anchoring retains those literals
in separate records. That explains a direction to investigate, but mimicking
backwards packing in a foreground append-only Store would require a lifecycle
or representation change. Studying Git is not authorization to violate the
original no-repacker/fixed-depth constraints silently.

## Audit of the previous full157 estimate

[growth-estimate.md](growth-estimate.md) correctly labels its 114,013,422 B
non-DELTA term as current measured cost, with explicit adjustment terms, and
labels 149–169 MB as conditional arithmetic. It is not a fundamental lower
bound, forecast confidence interval, or measured optimization.

The assumed 60–85% / 30–65% reductions across size-ratio populations have no
measured alternative encodings behind them. Target/base size ratio identifies
expensive populations, but cannot establish recoverable bytes. Large differences
can contain genuinely new information. Calling the resulting central number a
"best estimate" overstates its evidence; it is a scenario budget.

Refreshing forward FULL anchors changes both DELTA and FULL costs and does not
capture Git's later-base/reverse-layout advantage. Git can change which object
is FULL; a forward anchor policy typically adds new FULLs without rewriting the
old ones. The full157 model is therefore not a forecast of approaching Git.

Do not transfer full157 proportions directly to the ten-state smoke. FULL
frames already cost 30.19 MB in ten states versus 37.54 MB in full157, whereas
DELTA frames are 19.36 MB versus 87.29 MB. The smoke gap is predominantly content,
but much less of its total is old-anchor DELTA accumulation. Focusing only on
the long-history DELTA problem can miss the short-smoke target.

## Smallest evidence needed next

The parent task has now completed the mapping of current SmallContent FULL
records to Git assignments and complete base-closure availability. The saved
read-only artifact is `small-full-git-attribution.json` in the evidence root.
No alternative encoding was performed. Its complete population accounting is:

| Git representation of our FULL object | Objects | LayerFS FULL frame bytes | Git packed target bytes |
|---|---:|---:|---:|
| Also FULL | 5,025 | 11,177,274 | 10,605,176 |
| DELTA requiring later base closure | 7,855 | 16,494,159 | 1,687,032 |
| DELTA with same-snapshot base closure | 989 | 1,101,538 | 356,151 |
| DELTA with earlier base closure | 777 | 1,418,055 | 265,491 |
| **Total** | **14,646** | **30,191,026** | **12,913,850** |

The largest observed assignment mismatch is 16.49 MB of our FULL frames whose
Git representation uses only 1.69 MB of target records but requires later
base-closure objects. That 14.81 MB record-size difference is not a standalone
online saving: required base-closure storage and reconstruction work are not
charged to the target column. Earlier/same-closure cases total 2,519,593 B
of our FULL frames versus 621,642 B of Git target records, a 1,897,951 B
record-size difference. These provide bounded diagnostic candidates available
without future snapshots, although same-snapshot dependency ordering and
chain-depth constraints still matter. The also-FULL difference is only
572,098 B, further pointing toward representation/base policy as the larger
issue than simply changing compression settings. These are measured
representation differences, not newly measured replacement costs.

Then, if experimentation is authorized, encode a handful of those real file
families with the existing codec under (a) current FULL anchor, (b) immediate
predecessor as a diagnostic, and (c) Git-selected/later base as a diagnostic.
Keep every version and account for FULL retention, records, and decoding bounds.
This isolates matcher efficiency from candidate-graph limits without a new
benchmark family, repack, or full history run. Options outside the fixed graph
remain diagnostics until a policy change is explicit.

Separately reuse the existing metadata role inventory and the rejected
metadata-anchor attempt to identify the largest concrete metadata structure.
A 40 MB forecast needs measured savings here or an explicit compensating
content budget. Merely comparing DELTA counts between Git trees and LayerFS
inodes is insufficient.

Only a substantive product change with matched ten-snapshot measurement,
same-Store exact verification, and cleanup can turn this budget into a result.
Full157 remains necessary afterwards for long-anchor behavior and final history
qualification. No additional experiment was performed for this review.

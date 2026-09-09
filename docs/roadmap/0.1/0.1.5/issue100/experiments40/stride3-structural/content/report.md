# Stride3 selected SmallContent structural rerun

Same `git-small` policy as the selected full157 structural result; no new policy search. All **59,768** source SmallContent objects were decoded and authenticated, joined to matched Git53 blobs, and re-encoded. All exported identities were independently reconstructed and authenticated over **521,421,951 unique raw bytes**.

| Measurement | Bytes / count |
|---|---:|
| Original source SmallContent frames | 46,993,759 B |
| Original source SmallContent packs | 50,132,007 B |
| Selected structural frames | 41,972,723 B |
| Selected records plus compact directories, excluding pack headers | 43,822,603 B |
| Matched Git entries for identical SmallContent population | 40,626,601 B |
| Selected FULL / DELTA objects | 11,298 / 48,470 |
| Outside-population Git bases, encoded FULL | 24 |
| Actual maximum depth | 32 edges |
| Actual maximum canonical / encoded closure | 2,409,470 / 87,786 B |

Frame saving is **5,021,036 B** against original source frames. The structural frames remain **1,346,122 B** above the matched Git entries; selected records plus directories remain **3,196,002 B** above them, before LayerFS pack headers. These are component measurements: the SQLite record export is a diagnostic graph, not the final Store footprint. All bases are members of the exact source population and counted once. No omitted external bases.

Pinned zstd1.5.7 parameters reproduced all **598** fixed every100th source frames (95 FULL,64 one-base deltas,439 chain deltas). Candidate encoding plus exported-graph verification took **36.31 s**; the full script including inventory and source authentication took **65.41 s**. These are offline diagnostic costs, not public Commit timings.

The unchanged policy allows50 edges and64MiB canonical/encoded closure. Actual observed32edges and2.41MB canonical closure exceed current product limits; this remains the same extended-bounds offline reference used in the full157 comparison.

Protocol: [protocol.md](protocol.md). Executed source: [experiment.py](experiment.py). Results: [git-small.json](git-small.json), [summary.json](summary.json), [source attribution](source-attribution-summary.json), [independent closure/integrity audit](audit.json), [pre-encoding seals](manifest.json).

Raw artifact: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/content/git-small.sqlite`; SHA256 `aae8475235e363d643e9ce6a68f6e584151cb18237a00c48aabf4ceca751f62e`. Table `records(oid,id,base,raw,frame,depth,canonical,encoded)` preserves original32-byte canonical identities and uses Git SHA1 strings for dependency references. Final kind103pack assembly and original53state oracle verification are tracked by the combined experiment.

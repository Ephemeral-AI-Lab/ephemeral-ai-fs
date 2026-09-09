# Structural metadata investigation: bounded chains and change-based state storage

Status: two completed offline full157 experiments, 2026-09-10. Neither is product qualification. The source has158 roots including genesis.

## Results

| Representation | Measured bytes | Scope |
|---|---:|---|
| Matched chronological D FULL |35,010,506| Metadata pack bytes, all24,748canonical objects |
| Previously retained depth1 D |25,624,588| Metadata pack bytes |
| Fixed depth16 /128KiB closure D |**17,459,061**| Metadata pack bytes, same canonical objects and exact locators |
| Checkpoint + per-state changes reference |**14,032,896**| Actual allocated and logical metadata SQLite database, including packs, segments, all its indexes and allocator |
| Git trees and commits |5,007,335| Recorded Git metadata entry bytes; different semantics/accounting |

Depth16 saves **8,165,527bytes** against retained depth1 (31.87%), and17,551,445against the matched FULL control. It addresses a substantial fraction of the20.62MB metadata gap, but leaves12.45MB above Git's metadata entry bytes before LayerFS indexes. The semantic reference costs11,591,692less than the old depth1 **pack-only** subtotal even with its own metadata indexes included. Its total exceeds Git's metadata entry bytes by9,025,561; these are different scopes and not a full-store savings assertion. Neither result supports a route to56.37MB by itself.

Protocol was written before either run; see [protocol.md](protocol.md). Both policies were run once, no depth/checkpoint/codec sweep. Exact scripts and raw results are preserved. Content encodings remained untouched.

## Compatible bounded-chain experiment

Reused the existing previous-root leaf origin, stable-key overlap ranking, one matcher candidate, budget, canonical format, FULL-size group membership, pinned codec, and ≥max(64,ceil(FULLencoded/8)) group selection threshold. Changed only whether a chosen DELTA base can be used: at most16edges and128KiB summed canonical reconstruction closure. No candidate hunt or global future history search.

Observed7,958selected DELTA objects, maximum15edges and130,304canonical closure bytes.375eligible leaves hit the closure bound. Physical dependencies are retained once in the unchanged inventory. All24,748canonical objects reconstructed byte-for-byte and authenticated, and all158inode tables and their directories matched the sealed original D inventory. API exports the real388packs and24,748locators, rebasing only physical pack IDs.

Worst-case read costs, measured independently from actual stored dependency records:

| Metric | Maximum |
|---|---:|
| Summed encoded record bytes |53,798|
| Unique compressed group bytes |66,135|
| Unique decoded group bytes |108,051|
| Dependency groups / physical packs |16 /16|
| Reconstructed canonical closure |130,304|

These are maxima across different objects, not one common target. Pack-granularity readers may read substantially more than compressed group bytes; no latency/RSS claim is made. The parent combined-copy investigation measures public-oracle restoration with its bounded reader.

The inherited fixture label `depth2_base` constructs a cyclic invalid base, rather than a valid depth2 chain. Its rejection is a cycle test. The separate [verify.py](verify.py) explicitly rejects17edges and oversized closure, authenticates the complete selected inventory, and checks malformed checkpoint records. Valid multi-edge chains are authenticated throughout the real fixture.

Export interface: `chain_api.original_inventory()`, `pack_blobs(inventory, base_pack, layout='delta')`, `root_map()`, `inode_oracle()`, `record_ledger()`, `decode_record(identity, records)`. Cache schema is unchanged from the previous metadata delta experiment. Metadata-only cache stores both controls and diagnostic copies and is **not** a projected production database size.

## Structurally different checkpoint/change reference

Replace9,087LFS6INT inode-table CAS objects with full table checkpoints at roots0,16,...,144 plus exact changed/deleted stable-serial rows between checkpoints. Each value retains all73bytes of D inode metadata. Keep **every other15,661canonical metadata object**, even when potentially redundant. Retained root objects keep their original bytes; the states index explicitly binds each original root/table identity to the replacement checkpoint stream. Thus the reference preserves semantic states, not the original table CAS representation.

Each state record is split into≤16KiB independently compressed segments with the same pinned codec. SQLite stores actual segment bodies and ordinals, state roots, original table IDs, checkpoint links, authentication hashes, decoded lengths, retained object packs and their full-hash locator index, and scope/allocator. Its ordinary SQLite page allocation is counted without subtracting empty space.

| Actual database contribution | Bytes |
|---|---:|
| Retained object pack table allocation |2,682,880|
| Checkpoint/change segment table allocation |10,575,872|
| Retained full-hash object index |729,088|
| State table |24,576|
| State-root uniqueness index |12,288|
| Allocator |4,096|
| SQLite schema |4,096|
| **Total** |**14,032,896**|

Actual pack bodies are2,655,034bytes and compressed checkpoint/change segment bodies8,571,980bytes. Segment-table unused space alone is1,967,888bytes; it remains included. There are10checkpoints and158state records. Every state independently restores from its checkpoint, compares its exact serial/value table with the original D decoder, validates the stored SHA-256 state digest, and verifies all directory references using authenticated retained canonical objects:139,247directory comparisons. SQLite integrity passes. Source seals are unchanged.

The price is a different read model: up to15state changes, up to3,433,521decoded stream bytes per reconstruction, and a materialized table of up to884,925row bytes (Python/container overhead excluded). It has no point-lookup index over historical changes. Branch traversal, concurrent writes, crash recovery, GC and public Store integration are unimplemented. Content objects and content indexes are excluded from this metadata database. The states map substitutes for table-CAS resolution; original table hashes cannot be authenticated by the ordinary CAS decoder without rebuilding that representation. This is a measured architectural reference, not a format ready to integrate.

## Decision

- Preserve the17.46MB compatible chain candidate for full-copy integration and independent content/metadata oracle checks. Its16-pack dependency fan-out must be assessed before product adoption.
- Keep the14.03MB change-stream reference as evidence that reducing repeated FULL tables helps, but checkpointing alone does **not** solve the whole metadata gap. It buys only3.43MB compared with chain pack bytes, despite changing random-read semantics; accounting includes different index scopes, so that difference is descriptive only.
- A further metadata redesign must target repeated values in change records and retained non-table metadata while retaining efficient reads. Do not assume the missing9MB can be recovered by minor framing changes or simply longer chains.

## Reproduction and evidence

Raw immutable outputs: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural/metadata/`.

Copy the four scripts plus protocol to a fresh output directory; then run `python3 experiment.py`, `python3 checkpoint.py`, `python3 verify.py`. Scripts require the existing sealed full157 metadata source and history-delta Rust matcher/codec dependencies at their recorded absolute paths. They refuse to overwrite existing experiment databases. Use `python3 chain_api.py` to verify cached export hashes/locators and reconstructions without regenerating results.

Reports: [result.json](result.json), [checkpoint-result.json](checkpoint-result.json), [read-cost.json](read-cost.json), [manifest.json](manifest.json). Raw `states.json`, `ledger.csv`, `groups.json`, `checkpoint-states.json`, actual caches and attempt logs remain in the raw output directory. Encoding/reconstruction scripts took56.53seconds and13.32seconds respectively; these Python diagnostic times are not production Commit or read benchmarks.

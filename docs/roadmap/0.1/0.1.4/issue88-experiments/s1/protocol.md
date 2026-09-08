# S1 exact offline engine-origin protocol

Supplement to committed contract-v1; frozen before build/encoding. Arguments:
`issue88-s1 ORIGINAL_STORE PRIOR_ANALYSIS_INDEX NEW_OUTPUT_DIRECTORY`.

The original Store is opened SQLite read-only immutable, after parent quiescence
and seal checks. The previous authenticated analysis index supplies exact selected
locators, role labels and first-checkpoint provenance. Both original and analysis
seals are checked by the run owner. No payload canonical data is output.

One pack scan loads at most256KiB per pack and decodes only the9918 structural
groups. All279724 FULL records authenticate to their selected ObjectIds. The new
`structural.sqlite` holds those canonical metadata records and their original
locations, never raw file payload. It uses an8MiB SQLite cache; no decoded-group
cache is needed. Metadata directories/IDs are finite for the frozen run and
bounded by its366141 selected rows. Original groups must contain only selected
structural records with one authenticated checkpoint; malformed mixtures stop.

For each checkpoint1..157, enumerate prior/current inode bindings through exact
canonical decoders, bounded to65536 entries each BEFORE growing the vectors.
Compute sorted binding differences and run current `inode_table_apply_sorted`.
The isolated default `ObjectStore::put_tree_origin` hook calls ordinary put_owned;
only the analysis Store overrides it. Engine::persist passes actual Page.origin.
Reads by the rebuilding engine are restricted to earlier retained nodes or nodes
it actually emitted in this invocation. Rebuilt roots must exactly equal retained
roots at every checkpoint. A mismatch is a preserved failure, not permission to
substitute an inferred neighboring leaf. Origins are actual OFFLINE tree-engine
observations, not historical producer logs.

For each original structural group in checkpoint/pack/group order, control A uses
all original FULL records and current pack::encode_group. A encoded length and BLAKE3 must
match the original group's encoded bytes. Candidate B keeps identical canonical
records/order/membership and substitutes only origin-hinted inode-table leaves.
The origin must be an exact leaf from an earlier checkpoint and must remain FULL
in the candidate arm's already selected representation. Missing, same-checkpoint,
future, nonleaf, or candidate-DELTA origin uses FULL; no shadow base or retargeting.

Matcher scratch is the exact current implementation:16MiB charged work and512
trials PER ORIGINAL PACK, reset only on next original pack. Each target has one
actual origin candidate. This is a declared offline bound, not historical
admission-batch scheduling. Existing mixed threshold and Zstd1 parameters apply.
Every chosen group is decoded immediately and each FULL/DELTA target is compared
byte-for-byte with the original canonical bytes and authenticated. Every DELTA
must resolve to an earlier candidate FULL. Candidate group frames are streamed
to `candidate-groups.bin`, offsets/lengths/BLAKE3 to candidate-frames.csv.

Outputs: structural.sqlite, tree-reconstruction.csv, groups.csv,
candidate-groups.bin, candidate-frames.csv, result.json. Original pack framing is
not emitted anew because payload groups remain fixed and pack offsets would
change; original header/directory byte COUNT is unchanged. Report that constant
separately. Encoded group potential is not complete allocated Store savings.
All failed partial directories are preserved; a source fix requires a new sealed
identity and new output directory. No automatic rerun after observed mismatch.

Parent wrapper captures total wall/CPU/peakRSS/I/O,4h bound/free disk and source,
contract, executable/codec hashes, existing nonblocking measurement lock and
serialized slot. Internal phases are extraction, tree reconstruction and group
encoding/validation. Public operation/cold read latency remains unavailable in
this offline S1 screen; fresh decoder correctness is not a cold-disk timing claim.

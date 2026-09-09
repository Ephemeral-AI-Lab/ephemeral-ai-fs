# Compact selected-FULL candidate index

Prospective design before changed encoding/measurement, 2026-09-10.
The selected-DELTA cache expansion was rejected and reverted: content packs grew
1,854 B, Commit median grew from 0.581 to 0.622 seconds, and its 8,192-B allocation
reduction came from other categories. Preserve its complete successful ten-state
verification/cleanup evidence, source patch and measured tradeoff.

Inspection identifies wasted candidate-index ownership: insert copies a complete
ObjectId plus eight-fingerprint signature into up to eight of 1,024 slots. The
same 128-KiB reserved index allowance can retain each selected FULL once and use
small references for lookup. This is a structural duplication repair, not a
cache-budget, codec, cutoff, page-size or parameter sweep.

Use exactly 1,024 candidate records in a fixed ring and 8,192 direct-mapped u16
slot references, with u16::MAX empty. Each candidate record contains its real
ObjectId and the same eight 64-bit fingerprints. On replacement clear the old
record's index references only when they still point to its ring position, then
write the new record and its references. On lookup probe the same maximum eight
fingerprints, require the referenced record to contain the queried fingerprint,
and preserve the >=2 total matching fingerprints and ObjectId tie-break.
Choose at most one candidate. No raw content cache, history traversal, secondary
candidate encoding trial, global/persisted index, artificial base or dependency.

Actual boxed-array storage is 1,024 * size_of<Option<Entry>> + 8,192 * size_of<u16>,
plus the owning optional Mutex<Candidates>; assert this fits the unchanged
128-KiB charge deducted from the existing admission index budget. Arrays have
fixed lengths and no growth. Fixed local fingerprint/entry operands fit remaining
index headroom. This improves representation efficiency within the same budget;
no storage saving is assumed before measurement.

All other selected-FULL policy is retained: actual selected FULL winners only,
nonfinal batches, after Store publication and outside the Store lock, session
lifetime/rollback ownership, genuine and removed-name predecessor priority,
schema-9-only enablement, exact authentication of the selected FULL and existing
kind 1. Decoder/encoder lifetimes and 2-MiB/3-MiB allowances do not change.
SmallContent chain limits remain 8 edges/512-KiB canonical/256-KiB encoded closure.
No persisted format or old-reader policy changes; supported old opens do not
promote. Missing/corrupt required dependencies remain errors, and FULL fallback,
complete encoded-cost comparison, retained-base accounting and late CAS stay fixed.

Extend focused cache checks with index collisions, replacement/eviction and
self/unrelated refusal. Reuse the existing real tool-schema diagnostic and
selected-FULL publication/rollback check; do not repeat an unchanged codec study.
Then build affected artifacts and run one fresh ten-state performance, census,
exact same-Store verification and cleanup. Controls remain immutable and matched.
Record exact source/patch/binary/image/fixture/command custody and actual bytes.
Keep the prospective speed/RSS criteria and report all tradeoffs; 45–46 MB means
near-target only, not exact 45-MB achievement. No full157 until stable verified
near-target; no release-admission PASS, tag or release.

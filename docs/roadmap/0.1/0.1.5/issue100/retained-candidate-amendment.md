# Retained admission candidate handoff

Prospective amendment, 2026-09-10, before changed encoding/measurement.
Compact-candidate-1 verified ten states with clean teardown at 50,368,512 B.
Content packs fell 123,422 B versus removed-base-1, but allocation fell only
4,096 B. This is not near-target. The fixed compact representation is retained
as the bounded owner for this next lifetime experiment, not declared sufficient.

Current cache state drops at every AdmissionSession end, even when the admission
retains all its output. Thus a new path cannot discover a cached similar FULL
from an earlier retained admission unless a separate namespace hint finds it.
Prior metadata studies expose still-existing same-basename candidates as well as
ambiguous removed ones; names alone did not establish good bases. Reuse the known
fingerprints rather than add a namespace-wide search, payload scan or size heuristic.

Move the existing cache into one optional idle slot on the SAME StoreDb only
when its AdmissionSession drops in retained state. The operation permit remains
held during this transfer. The next admission acquires that permit and takes
the cache out of the idle slot; otherwise it constructs the normal empty cache.
No cloning, second live cache, cross-Store transfer, persistence, history replay,
global object index, recursive search or reopen warm-up is introduced. A reopened
Store begins with an empty cache. This is a bounded cache of previously observed
selected FULLs, not a database-wide similarity search.

Keep exactly 1,024 candidate records and 8,192 u16 references within the existing
128-KiB admission-index reservation. Active admission index capacity remains its
existing 16-MiB allowance less that reservation for the other index owners. While
idle, at most the same 128-KiB cache survives; its lifetime extends, and this idle
footprint is explicit. Transfer ownership without copying arrays before releasing
the writer permit, so active and idle cache arrays do not overlap. The cache is
freed when the StoreDb closes. Per-active 2-MiB reconstruction/3-MiB encoding,
their static codecs/buffers and all other existing resource owners remain fixed.

Only actual selected SmallContent FULL winners enter, after publication and outside
the Store lock; skip final-batch registration as before. Return the cache only
from retained state. An abandoned/rolled-back/failed admission discards its entire
cache, including inherited hints, so private rolled-back IDs cannot leak forward.
Retained output already survives that session's rollback boundary. A subsequent
session's rollback removes only packs newer than its own high-water mark and
cannot delete those inherited bases. Existing operation serialization stabilizes
selected locators; copied raw bases or alternative physical representations are
not created. Optional poisoned idle-cache state is discarded, not used as proof.

Genuine inode/same-path and removed-name hints retain priority. The same eight
fingerprint probes select at most one FULL; authenticate its exact canonical ID
at its selected location once. A selected cached object must remain FULL; missing
or corrupt required dependencies remain errors. Existing FULL-vs-one-DELTA complete
cost selection, kind-1 grammar and physical dependency retention remain unchanged.
Schema-9 only; supported schema 6/7/8 opens do not promote. No new persisted format
or canonical identity, no codec/settings changes, and no release downgrade claim.
Existing kind-2 chains keep 8-edge/512-KiB canonical/256-KiB encoded bounds.

Leave a focused cross-admission check: retained FULL is reused as a base in the
next admission, exact bytes authenticate, rollback discards private cache state
while preserving earlier retained objects, and Store close/reopen starts cold.
Then build affected artifacts and run one fresh ten-state performance, frozen
census, exact SAME-Store historical verifier and cleanup. Retain exact source,
patch, binary/image/fixture seals and commands. Existing prospective <=10% speed/
RSS comparison and separate 8-MiB RSS allowance remain working criteria only;
report all actual costs. Full157 is deferred until a stable verified candidate
is near 45 MB; 45–46 MB is near-target, not exact achievement or release admission.

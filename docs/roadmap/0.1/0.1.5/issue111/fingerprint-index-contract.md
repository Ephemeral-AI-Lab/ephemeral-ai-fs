# Compact fingerprint index: one bounded cold Init experiment

Freeze before implementation. Control is current main43cf85a78 plus preserved
uncommitted compaction-removal work; treatment remains promoted-uncompacted.
Use isolated worktrees, preserve the original patch byte-for-byte. Evidence root
is recorded before edits in /tmp/layerfs-fingerprint-index-root. Older evidence
roots remain read-only. No guarded-predecessor candidate or other optimization.

Hypothesis: the full73-byte scratch B-tree key produces measured working-set churn
(24546 sync cache misses/21295 spills;18373 lookup misses). Replacing scratch keys
with a64-bit fingerprint plus ordinal should reduce SQLite page work enough to
justify a product change. Fingerprints are filters, never equality authorities.

Candidate only changes shared ValueIndex representation/lookup. Hash using the
existing ObjectId digest and deterministic64-bit extraction. Retain all ordinals,
including hash duplicates, with the exact existing group-boundary131072-entry
retention/eviction accounting. Deduplicate query fingerprints globally, collect
at most131072 candidate ordinals (512KiB), sort, authenticate each selected group
once in ordinal order, and compare complete73-byte values. Return the minimum
matching ordinal, preserving the old retained first-winner rule even on collisions.
No new group cache: retain one <=165-value decoded group. Account bounded query
state within existing index/decoder budgets. No K-by-history repeated scans.
Extreme collisions can scan the retained window; no universal sublinear claim.

Keep schema10,4KiB pages,4MiB scratch cache,32MiB scratch file, compact scopes,
pack/zstd/FULL/DELTA/CDC, exact CAS, authentication, public operation and timers.
No source special case, persistent cache, warm option, compaction or off-timer work.

Screen: identical existing nonce phase/counter harness plus identical added
fingerprint-candidate/authentication/scratch-size counters in both arms. Full
namespace-100000, seed1,100000 files/1000 directories/500000000 B, fixture digest
6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e.
n=2 per arm, order C1,T1,T2,C2. Independently acquire/validate zero resident input
pages before EVERY run using shared cold acquisition. Require full125169 pages,
100000 files and fixed launch allowance. No warm-up/fallback. Preserve profile
reused-first-sample-uncontrolled and initialization_disk_read_bytes per row.
Nonce rows are qualification-ineligible regardless of cold evidence.

Continue beyond screening only when both paired Init reductions are positive,
median paired reduction >=150ms, total metadata-preparation median decreases,
all correctness/format checks pass and candidate apparent Store bytes are within
1% of control maximum, allocated bytes within2% of control maximum. Canonical
112451 objects/513026835 B and100002 pooled values must agree exactly. Bounds
are fixed before results; randomized scope/admission placement means whole-Store
physical bytes need not be identical. Report all raw bytes, including small growth.

If screen passes, reseal separate plain control/candidate source arms (remove nonce
instrumentation equally, retain only product treatment). Plain cold n=3 per arm,
order C1,T1,T2,C2,C3,T3 through registered runner cold guard. Eligibility requires
verified acquisition and matched operation/fixture/procedure/paired conditions;
missing evidence fails closed. Meaningful gain requires median paired reduction
>=150ms, at least2/3 pairs >=150ms, and all pairs positive. Absolute acceptance is
still <=2700000000ns; a useful reduction does not close a missed target. Also
qualify all four registered Init tiers with matching-image independent verifier,
and affected retained/reopened/redundant Commit coverage before promotion.

All attempts retained, no outlier deletion or slower-arm retry. One replacement
of an entire affected pair allowed only for demonstrated infrastructure invalidity.
If the screen or qualification fails, reject this treatment and report the blocker;
do not stack another optimization in this experiment.

Use shared runner --build-host/--build-image under its measurement lock. Direct
tests/diagnostics acquire it once in the parent; children do not reacquire. No
resource-sensitive overlap. SQLite/SDK/publication/spool on macOS; Docker only
for daemon/FUSE/workload. Source edits after a build require resealing/rebuilding.
Retain command/log/exit, binary/image/source/product/fixture identities and checks.

Tests before sampling: forced fingerprint collisions, equal duplicate earliest
winner, real eviction (including duplicate-heavy groups), low parameter limits,
repeated hashes across query pages, candidate/group visit bounds, corrupted/missing
pool groups, rollback/invalidation/reopen. Existing metadata/admission suites.
No release/tag/deployment. Update #111 with both positive and negative results;
context #115/#109/#110/#108/#106/#102/#104/#100/#107.

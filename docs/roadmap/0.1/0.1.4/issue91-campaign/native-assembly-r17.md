# R17 prospective direct native pack assembly

This amendment follows R15 and R16 in the user's ordered sequence. Development
starts in an isolated worktree at 548c3663c; integration and qualification wait
for the R16 decision. It reuses the campaign README, declaration, source contracts,
R6 four cells/order/seed1, public timers, independent proofs, resource ceilings,
measurement lock, and generation/source/binary/image custody requirements.

Remove the intermediate native encoded-group byte buffer by assembling native
record bytes directly into the final pack. Retain the individual encoded records
initially. Preserve canonical order, FULL-alternative group boundaries, native
record/frame bytes, group and pack limits, native FULL/PREFIX choice, predecessor
authentication/dependency closure, 4KiB SQLite pages, and publication behavior.
Keep the old native_group plus assemble_native implementation as a test oracle.
Require exact byte equality against that oracle across valid group/pack boundaries,
FULL/PREFIX records and multiple groups. Invalid records and oversized/count-limit
inputs must remain rejected. Resource accounting must use actual retained Vec
capacities, including in-progress final pack and record ownership; no increased
physical/canonical/association allowance, dependencies or codec changes.

Run focused byte-equivalence, parser/error and resource-bound checks and the
existing native/authentication/collision/rollback checks before a clean generation
is built. No new benchmark fixture or instrumented timing may replace the R6
public observations. Collect only after the earlier treatment decision and sealed
identity, then repeat the same four R6 cells and their independent proofs once.
Retain all attempts and allocation/CPU/RSS/I/O/temporary-space/cleanup receipts.

Keep the treatment only if observed public-operation performance improves without
an unresolved binding gate or resource/storage regression; otherwise revert the
treatment and retain the failed evidence. Exact native bytes and all existing
resource bounds are hard gates, regardless of timing. The selected final matrix,
full157 equal-retained-state allocation gate, historical verification, and
independent review remain required. This amendment grants no new acceptance
allowance and does not substitute an encoding-only result for public performance.

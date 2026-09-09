# Issue100 bounded predecessor investigation

Prospective amendment, 2026-09-09, before alternative encoding or changed-candidate measurement.
Owner's 45,000,000-byte ten-snapshot objective and revised scope supersede the
earlier one-level-only restriction and historical first-round execution instructions.
45–46 million bytes is near-target, never exact achievement or release admission.

First investigate immediate-predecessor SmallContent chains, with **8 maximum
edges**, **512 KiB total decoded canonical closure including the target**, and
**256 KiB retained encoded record capacity**. This is one bounded policy, not a
parameter sweep. Use a handful of original fixture families, including the
133273 -> 129991-byte cutoff crossing, to compare complete retained FULL/base/
DELTA costs and reconstruction. The crossing remains ineligible in this design;
do not manufacture a SmallContent copy of the CDC predecessor. A cross-CDC
implementation needs a separate concrete amendment after evidence justifies it.

Eligible base: the one already selected immediate SmallContent predecessor,
FULL or bounded DELTA, named by existing scoped provenance. No global candidate
index, future base, reverse rewrite, new base copy, or alternative codec. Exact
CAS selection precedes encoding. At the depth/closure/encoded-cap ceiling emit
FULL; prepare FULL once and at most one eligible DELTA and choose strictly lower
complete record cost (32-byte reference included); FULL wins ties.

Persist a new kind 2 in existing pack-v3 framing, with unchanged FULL kind 0 and
FULL-base-only kind 1 semantics. New schema 9 explicitly fences kind 2; supported
schema 6/7/8 opens do not promote and retain their respective writer policies.
Canonical SmallContent v1 bytes and identity stay unchanged. Old binaries reject
schema 9 before mutation. Explicit offline upgrade accepts schema 7/8 to 9 with
existing exclusive revalidation and transactional DELETE/FULL profile; no payload
rewrite or downgrade. Rollback to old binaries requires a pre-upgrade backup.

Reader iteratively acquires at most nine selected records, validates role,
locator length, chronological dependencies, cycles, edge count and decoded-byte
sum before reconstruction, and authenticates every decoded canonical node.
Kind 1 must still terminate directly at kind 0. Missing/corrupt persisted bases
are integrity errors. Optional absent/ineligible hints can fall back to FULL;
corruption is not masked. Admission carries verified closure facts without a
second readback. No recursive history walk or heap codec fallback.

At reconstruction clear the old read-wave operand cache before the chain path.
Simultaneous ownership: <=1 MiB static decoder/dictionary storage, <=256 KiB
retained encoded records, <=4*(131071+23) bytes for previous canonical base,
decoded raw and canonical framing/conversion temporaries, plus <=16 KiB bounded
associations. The sum is below 2 MiB. Check actual Vec capacities and reject
oversized ownership before allocating/decoding. During admission the <=128 KiB
target also fits remaining headroom. Decoder is dropped before allocating the
2 MiB static encoder; target/base plus FULL/DELTA/output handoff remain inside
the existing 1 MiB operand and 3 MiB total encoding allowance. Existing data/
physical-output ledgers remain authoritative, including producer queues.

Existing admission session stabilizes selected locations and keeps base packs
older than dependent publication. Physical reference traversal follows kind 2
as well as kind 1, retaining full transitive closure; private rollback deletes
only its owned rows/packs and preserves retained stages/preexisting bases.
No GC/repacker or hidden post-measurement maintenance is introduced.

Prospective working speed/resource criterion (engineering comparison, not an
owner-approved release gate): aim for <=10% increase versus existing v0.1.5 in
save/Commit/paired medians and sums, performance/verification case walls, and
host lifetime RSS; report RSS also against a separate 8 MiB absolute allowance.
Keep actual tradeoffs if storage justifies them; correctness, ownership,
resource-limit and cleanup failures cannot be accepted. Ten dependent history
steps provide no tail-confidence or independent repetition claim.

All three unchanged baselines remain immutable. Build and focused checks are
serialized under the existing lock. Public ten-state performance, frozen census,
exact same-Store verification and cleanup are required for substantive candidates.
Full157 is final confirmation only after the short-loop design stabilizes.

## Second diagnostic: bounded recent FULL candidates

After chain-1's complete 56,668,160-byte result, inspect a **128-entry,
16-KiB session-local ring of already selected SmallContent FULLs**, only when
no eligible immediate predecessor exists. This targets the remaining FULL
population; it is not a database-wide similarity index or history walk.
Select at most one candidate by overlap of the eight smallest distinct hashes
of lines >=16 bytes (hash each complete line with Rust's existing DefaultHasher).
Require at least two matching hashes; ties prefer the most recently selected.
No raw bytes remain cached. No parameter sweep or fixture-path rule.

First test this bounded window on the same three actual file families, using
read-only selected record order from chain-1. Report whether candidates were
within the same snapshot/session, never treat an earlier-session ring member as
available. Reconstruct only selected FULLs under the existing static decoder and
assert identities/original target bytes. Compare complete FULL/base/DELTA cost;
these diagnostic totals do not predict allocated Store bytes.

Only if evidenced, a product ring would live exactly for AdmissionSession,
include only selected FULL winners after publication (never speculative output),
and clear with rollback/drop. Reserve 16 KiB from the existing seen-index allowance;
no decoder/encoder or other data budget increases. Signature computation occurs
outside the Store lock. The selected FULL must be authenticated at its actual
location using the existing bounded owner. The fallback remains FULL and one
DELTA trial. Use existing kind1 FULL-base grammar in schema9; schema8 writer
policy remains unchanged. No new persisted interpretation or added retention
rule is needed. Missing/corrupt required persisted bases remain errors.

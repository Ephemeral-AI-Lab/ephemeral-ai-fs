# R12 prospective reuse of owned absence proofs

G5 (d1032b154) records namespace1000006,040,561,000ns and payload500
3,764,907,208ns; all four proofs pass. Foreground qualification is still unresolved.
R10's profile shows a substantial second location lookup at publication after
all batch objects already received exact missing results under an exclusive writer
owner. Do not remove positive collision authentication or rely on timing luck.

Carry a bounded absence-proof epoch with the existing MissingBatch typestate.
The AdmissionSession owns a monotonically increasing publication epoch; every
successful object publication increments it before releasing the connection.
A batch records the epoch of its negative membership probes. If this epoch still
matches at publication, absence remains proven by exclusive ownership, so the
final validation does not need another SQL query. If it differs or a batch has no
proof, retain the existing lookup and exact length/byte authentication. Synthetic
late-race batches remain unproven unless created by the real accumulator.

When the accumulator itself flushes a disjoint previous batch while consuming a
unique incoming page, advance that page's proof only when the observed epoch moved
by exactly that one publication. Pending/incoming duplicate guards establish the
disjointness. Any intervening publication invalidates the proof and selects the
ordinary late comparison. Mixed probe epochs remain conservative. Overflow fails
closed; rollback closes the session. No public writer bypass, SQL uniqueness
relaxation, extra cache/index/service, or memory/batch-limit change.

Regression before repair: trace an ordinary prepared missing batch on a nonempty
Store and require no duplicate location query when its owner has not published
anything since the negative probe. Retain the failure. After repair require the
SQL recheck after a same-owner intervening publication, exact canonical equality,
late same-length corruption failure and rollback, preexisting and pending duplicate
accounting, Store writer serialization and unchanged resource ceilings. Existing
native late-race/rollback tests remain binding.

Freeze G6 after checks; execute the same four R6 cells/order/seed1 and independent
proofs once with distinct source/binary/image hashes. Native encoding, physical
pack bytes,4KiB pages, synchronous publication and durability remain unchanged.
Retain all earlier failures/unfavorable samples; final affected matrix/full157 and
independent review remain required. This is a proof-preserving removal of repeated
negative lookup, not permission to skip authentication or accept the G0 baseline.

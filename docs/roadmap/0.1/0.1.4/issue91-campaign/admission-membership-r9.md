# R9 prospective removal of redundant admission indexes

G3 (fa18ebde8) keeps four passing proofs and all observations. Init100000 improves
from G2's12,462,528,625ns to9,171,439,833ns, still severe against2,603,162,083ns.
Creation500 remains3,833,256,250ns. Fixing scratch journaling alone is insufficient.
G3 Init still builds two indexes containing almost the entire422k-object stream.
Both reproduce facts already owned by the serialized Store publication session.

Bounded repair: remove the separate available-dependency index; retain the same
bounded dependency-page collection and require Store.objects_exist for every
external dependency not in the pending batch. Published existence is authoritative.
No dependency may be accepted merely because its ID was encountered as input.

The session already records baseline_pack and owns the only writer permit. A
found immutable location with pack>baseline_pack was published by this session;
its canonical bytes must still compare/authenticate, but no second seen-index
entry is needed. Missing objects are first admissions, with existing pending and
incoming duplicate checks covering not-yet-published objects. Retain the original
16MiB seen allowance only for found preexisting objects (pack<=baseline_pack), to
count unique preexisting candidate occurrences exactly. Do not increase any
limit, expand caches or change canonical/physical encoding, batching, publication,
rollback, or public timers. Reuse the existing location lookup and watermark.

Before repair: strengthen the existing flushed-duplicate test to require zero
seen-index entries for fresh objects after real admission while retaining exact
collision/rollback/counter assertions; retain the failure. After repair: verify
first/repeated preexisting objects counted once, newly published duplicates
compared exactly, pending duplicates, missing dependency rejection, late races,
rollback, ownership serialization, resource bounds and spill paths. This is a
shared-path deletion, not a fixture-specific empty-Store shortcut.

Freeze G4 and distinct binary/image identity after checks. Reuse the unchanged
R6 four cells/order/seed1/public timers and independent proofs. If headline severe
residuals remain, diagnose before expanding; all final matrix/full157 and
independent review obligations stay required. Storage savings remain binding.

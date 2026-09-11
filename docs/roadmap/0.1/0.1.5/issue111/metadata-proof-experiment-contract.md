# Guarded metadata proof experiment

Freeze before implementation. One shared-code candidate: lazy global index
construction when a bounded single changed leaf can resolve every value through
pending values, authenticated predecessor ordinals and exact fresh-dependency
evidence. No persistent cache, schema/page/cache-size/worker change or unrelated
lookup optimization. Keep all authentication and encoding/DELTA limits.

Only use fresh-negative evidence when the held admission session has not published
metadata groups. A missing dependency lookup is not proof by itself; it must also
be present in authenticated pending preparation or known newly admitted after the
session baseline. Unknowns, budgets, missing/unpooled origins and prior published
metadata retain the original fallback. Reuse the predecessor in DeltaSearch and
charge retained bytes/read work to existing budgets.

Canonical identity/equality must remain exact. Reuse of an authenticated prior
ordinal may differ from the global index's physical winner; report encoding and
storage effects explicitly, never claim identical physical placement by assumption.

Use isolated control/candidate snapshots of the current product plus preserved
dirty work, with identical existing diagnostic harness/counters. Evidence root
recorded before edits in `/tmp/layerfs-metadata-proof-root`. Retain every attempt,
source patch, binary/image identity, command/log/exit and correctness result.
Build through shared runner and serialize resource-sensitive work under its lock.

First screen: namespace-100000, full500MB bootstrap per fresh Store; retained and
reopened modes separately, n=2 per arm, order C1,T1,T2,C2 in each mode. Same public
SDK10-byte edit and Commit, then a second marker/Commit separately. No warm-up,
no source edits after sealing, no timing-selected retry. OS cache uncontrolled;
reopened denotes Store/index lifetime. All nonce samples are qualification-ineligible.

Require the reopened first Commit to avoid global replay in both candidate runs,
both paired latency reductions greater than the control range, no systematic
retained-mode regression beyond control variation, and successful full edited-file/
reopen proofs. Reject if coverage fails, integrity fails, new pooled values increase,
metadata DELTA disappears or material encoded-byte growth exceeds control variation.
One whole affected pair replacement only for proven infrastructure invalidity.

Focused correctness must exercise positive proof, fresh/pending dependencies,
old-content reversion, same-session earlier metadata, unavailable/corrupt origin,
duplicate/evicted ordinals, rollback and budget fallback. Existing metadata/admission
tests remain. If the screen fails, retain/reject this candidate without stacking
another optimization. If it passes, report the candidate separately from any
subsequent broader/plain qualification. No cold Init <=2.7 s or release claim.

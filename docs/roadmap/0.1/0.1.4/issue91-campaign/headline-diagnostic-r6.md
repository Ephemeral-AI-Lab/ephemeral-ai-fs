# R6 prospective headline diagnosis (before new collection)

This amendment reuses README.md, declaration.json, source-contracts.json and
repair-r2-r3.md. It changes diagnostic order only: validate R2/R3 first, then
namespace100 and payload100m readiness, namespace100000 and payload500m with
independent proofs, before metadata100000/directory500/bulk100 and the remaining
affected matrix. One seed1 observation per selected cell, unchanged original
fixtures, public timers, limits, preparation policy, and fresh writable owners.
Retain every attempt/outlier. Final qualification still requires every affected
member, full157 and independent review; no new acceptance allowance.

## Inputs and routes established from frozen source

`families/init_namespace/mod.rs` and `workload/main.rs::namespace_plan` define
namespace100000 as 500,000,000 bytes, 100,000 regular files, 1,000 data directories
(100 files each, plus root): 1,000 empty, 78,998 tiny, 15,000 small, 5,000 medium,
and two 100,000,000-byte anchors. Content streams are seeded by scenario, path,
class and length; portable modes and mtimes are fixed. This is not identical
content to the creation fixture. G0's raw performance sample reports 422,057
canonical objects /542,897,294 canonical bytes. Physical packs/groups are separate
quantities, not interchangeable with canonical objects or input files.

`ordinary_workloads.rs::expected/apply` defines payload500 as one newly written
payload.bin of 500*1,048,576 = 524,288,000 bytes using Content::Seed/FLAT_SEED.
Its preexisting fixture is retained; creation is a FUSE Exec followed by Commit,
not a native import. Existing R1 receipts record 27,223 canonical objects /
525,955,698 canonical bytes. Public total is pure_call_sum_ns, with Exec and
Commit separately reported; Init's timer is layerstack_init_ns.

Init discovers native metadata/directories, builds complete files and namespace
structure, then publishes genesis. Workspace creation records live writes during
Exec, constructs its frozen dirty frontier during Commit, publishes the candidate,
and installs/finalizes its checkpoint. Both reach objects::run_finalized_output,
CheckedOutputAdmission::{admit_page,probe_incoming,push_pending,flush_batch},
PreparedAdmission::{prepare_missing,publish} and the same packed storage reader.
Lookup/dedup/collision authentication precede publication; pack construction and
native encoding precede SQLite insertion. Directory/metadata construction and
finalization differ. Canonical work inside Exec remains in its public timer.

Comparable byte volume does not imply comparable cost: Init has about 15.5 times
the canonical objects, many tiny file/metadata/tree objects and native filesystem
calls, while creation has one large new file. Repeated metadata can deduplicate,
but each supplied occurrence still follows authenticated admission contracts.

## Hypotheses and attribution

Leading H1: migration coupled SQL transaction formation to the sum of worst-case
collision-read scratch, reducing effective batches. G0 transactions are 2,519
for Init (historical130) and1,800 for creation (historical127). The retained
R2 before-check fails at12 versus120 objects. R2 separates bounded read waves
from output ownership while retaining512 objects/512KiB ordinary transactions,
maximal-object isolation,1MiB pending reserve plus active reconstruction budget,
2MiB physical and6MiB canonical ownership limits. Validate actual retained
capacity accounting, late collisions, rollback, and exhausted reserve first.

H2: native encoding/scratch and pack/group construction remain distinct costs.
R1 scratch reuse already improves only partly: Init12,155,402,250ns versus
published2,603,162,083ns; creation3,825,552,959ns versus3,068,249,542ns. G1's cache
symlink dirty-tree qualification remains; never relabel it a clean final source.
H3: repeated native record extraction adds collision/read cost; validate R3
separately at unit level, preserve mandatory canonical authentication.
H4: serial admission backpressure limits useful producer overlap even with the
unchanged min(available_parallelism,8) workers and four-slab queue. Summed producer
blocked/wall/CPU durations overlap; they must not be added to pipeline/public wall.
H5: directory shape, metadata work, and finalization may explain residuals after
H1. Do not infer their contribution by subtracting overlapping diagnostic phases.

Use existing physical, encoder, SQL, output-admission, idle, queue, producer,
metadata/directory and finalization telemetry first. No new instrumentation or
shape fixture is currently authorized by this amendment; freeze a further bounded
supplement if existing observations cannot distinguish residual hypotheses.
Collect selected public operations using the existing runner/collector command
forms and original300/310/600 and45/59 budgets. Save exact commands and sealed
source/binary/image declarations before each new generation's samples. Compare
published elapsed_ns and matching phase/resource scopes; report elapsed, allocation,
CPU/RSS/I/O/temporary space and cleanup together. No G0 acceptance baseline.

## Related work and custody

Ancestry checks show5db5eb5a0 and8d15ebc9b already ancestors of campaign73746e916.
Phase1 handoff explicitly integrates #71: bounded nested frontier scheduling,
small-file complete construction, metadata reuse, directory construction and
bounded checkpoint installation. No cherry-pick is needed. Namespace's1,000 root
tasks already exceed the nested-expansion trigger; the dominant nested .venv case
is a different distribution. Neither its .venv speedup nor its namespace control
measures a new treatment on this packed candidate. Keep namespace1000's unfavorable
5.3ms result and use only replacement evidence with distinct binary hashes.

The task "Create uv Torch repo commit" latest completed update confirms361 tests,
five checkpoint tests and three enabled Docker SDK tests passed; full .venv
Workspace proof remains pending. Its worktree has evidence/probe dirty changes,
which remain untouched; no workload there will be launched or task resumed.

Campaign dirty changes at takeover: objects.rs, admission.rs/native_tests.rs,
read.rs/native_tests.rs, report.py and test_report.py. Preserve and validate them.
All builds/heavy checks/samples use the existing TMPDIR measurement lock with
nonblocking acquisition and no overlapping Store writers. Use isolated Cargo
outputs; final candidate must have an unambiguous clean source and executable
identity. Evidence-only amendment is committed separately from those repairs.

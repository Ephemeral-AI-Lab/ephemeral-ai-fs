# historical_access v1 — issue #101

Preregistered before implementation and collection. Public host SDK opens a
byte-copy of the closed schema9 stride3 Store, forks the retained Commit and
mounts through Linux FUSE. No private Store read or offline9304 reader is a
substitute. The original source checkpoints are 1,58,157 (selected 1,20,53).

## Fixed ordered cases

| ID suffix (prefix `ha-`, suffix `-v1`) | Checkpoint | Operation | Cache |
| --- | ---: | --- | --- |
| stat-old | 1 | lstat README.md | application cold |
| stat-head | 157 | lstat README.md | application cold |
| directory-old | 1 | root readdir, at most128 entries | application cold |
| directory-head | 157 | root readdir, at most128 entries | application cold |
| small-old | 1 | full README.md (843 bytes) | application cold |
| small-head | 157 | full README.md (2201 bytes) | application cold |
| range-history-cold | 58 | pnpm-lock.yaml [528315,534736),6421 bytes | application cold |
| range-history-warm | 58 | same range | one identical untimed warm-up |
| full-head-cold | 157 | pnpm-lock.yaml,841964 bytes | application cold |
| full-head-warm | 157 | same full file | one identical untimed warm-up |

Ten cases; one operation each, seed0, repetition1 for selected qualification.
Cold means a new coordinator, Store copy, container and mount, without access
warm-up; OS caches are uncontrolled and mount may populate application caches.
Warm-up occurs inside the outer clock, before the measured access. The range
is the measured worst-amplification adapter from the53-state offline experiment;
public-format results cannot qualify that unintegrated adapter representation.

## Time and evidence

Performance hard limit15seconds from Python entry before argument parsing,
lock acquisition, source/build/fixture validation, Store copy/quick_check,
container readiness, mount, warm-up, operation, receipts and teardown.
Reserve3seconds for cleanup: working deadline12seconds. No waiting on lock.
Independent verification has the same prospective15second watchdog and target;
no inherited45/59second watchdog. A miss fails; never change workload or budget
because of results. Builds and the already completed history are explicit
prerequisites, never invoked by this family. Missing prerequisites are NOT_READY.

One selected run is diagnostic, admission_eligible=false. Explicit --all runs
these ten selections serially, each with its own envelope; no other family or
history construction runs. Performance and verification are separate invocations.
Verifier must bind the exact performance receipt, fixture, binary, image and
source seal, execute exactly the same selected access on a fresh copy and compare
with immutable original source oracles. No new expected result may be learned
from the tested product. Stat checks mode,size,mtime; directory checks exact
sorted immediate child names; reads check exact length and SHA256. Range expected
bytes come from the source Git blob, authenticated against its original oracle.

Retain outer and phase wall_ns, workload operation_ns, attempted/completed count,
returned bytes, Store physical receipt (encoded/decoded bytes, group/base fetches,
native decode/dependency counters), host CPU/current RSS/lifetime peak and cgroup
observations. Physical counters cover the host execute window, not pure POSIX
syscall time. Warm-up is a separate phase. No amplification ratio with zero
returned bytes. Unavailable unique-pack counts and exact phase memory peaks are
null with reasons; lifetime peaks are not access peaks. No speedup claim.

Retain exact source/product/build/image/fixture/oracle/contract seals, raw process
output and failures, immutable per-run evidence manifest, ownership/cleanup
receipts and original checkpoint/Commit mapping. Correctness, deadline, resource,
cleanup and custody statuses must all pass for a qualified selected result.
Check cgroup swap/OOM and confirm no active executions/workspaces before host
exit; force-remove only the benchmark-owned container on failure. Never delete
input Store or failed-run evidence. Cleanup failure leaves the issue open.

Selected development starts with small-head, then its independent proof. Once
correct, run each remaining sibling once and its proof; retain all attempts.
#102 owns full regression statistics, controls and optimization. No baseline
improvement claim or release admission is made by this unpaired implementation
qualification. Worst metadata paths and full157-specific native graphs remain
explicit #100/#102 integration qualifications, not silently passed here.

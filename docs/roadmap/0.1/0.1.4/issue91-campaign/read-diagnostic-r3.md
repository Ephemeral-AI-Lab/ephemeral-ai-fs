# R3 prospective read-path profile

G0 directory-content-scan500 public total3906595000→5275265667ns (+35.035%).
Exec3861436000→5231232250ns accounts for the increase; host local read/auth
1054520501→1675935136ns and host CPU1516647376→2275207709ns increase. Preparation
2.410/2.413s is excluded. Both workloads make15248 snapshot DB calls,5478 kernel
read requests,9478 payload batches (max10),35126 payload IDs,22324 rope nodes,
5000 opens,25714 workload syscalls and read529530880 bytes. No extra public
FUSE round trips are demonstrated. Async physical decoder counters are not
captured by ordinary creator-thread telemetry; their zero does not mean no work.

After G0 ordinary proofs, before choosing a read-path repair, run exactly ONE
additional G0 `directory-content-scan-500-mixed-v4`, seed1, clone, normal
300/310/600 budgets. During that owned host coordinator's performance Exec,
collect one macOS `sample PID 3 1 -file profile.txt`. Identify the process by
exact sealed binary and infra-run family/case under this invocation, never by
an unrelated Store/process. Retain process identity, sampler command/exit, full
profile and raw sample. Profiling overhead makes this diagnostic ineligible for
performance qualification; it cannot replace the unprofiled G0 result. Follow
with one independent verifier using matching receipt identities and45/59 limits.

Use the profile to distinguish per-record native decoder allocation/initialization,
SQLite blob/directory extraction, actual decompression and unrelated waits.
Any later bounded reuse must preserve selected-record scope, authentication,
frame/closure/depth limits and physical ownership. No new cache service, global
cache, dependency patch or changed workload/timer is permitted. Freeze the
chosen repair and its correctness/memory regression before a new source run.

# Final root/inode-tree RCA protocol

Owner authorized updating #111, then root-cause analysis before optimization.
Keep 4 KiB, four current producers, complete namespace-100000 input, encoding,
authentication, publication semantics and memory bounds unchanged.

Use an isolated detached source snapshot at463beacb3 plus the preserved
compaction-removal patch from version-breakdown evidence. Before instrumentation,
its product seal must equal95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4.
This explains the measured pre-cache-change product. Do not include or alter the
concurrent metadata-cache work in the main workspace.

Add diagnostic counters only in the isolated source: final-tree-specific
physical-counter snapshot; metadata preparation total/calls, synchronization
catalogue/read/insert/commit, lookup-set construction, batched lookup,
ordinal assignment, group encode and decode/digest, plus authoritative admission
INSERT wall. Reuse existing counters and phase clocks. Enclosing and nested
intervals must be explicitly distinguished; do not double-count their sums.

Freeze n=2 nonce diagnostics on this one instrumented source, same full immutable
fixture/digest, fresh independent host Stores, the shared fixed cold acquisition
before each run, no warm-up or timing-based retry. Record source residency/read
bytes, phase balances, row/group counts, source/product/binary seals, source patch,
commands/stdout/stderr/exits. All observations remain non-admission diagnostics;
the preceding matched-cold version study supplies historical context only.

Build with the isolated shared runner under the measurement lock. Docker is
unneeded by direct Init diagnosis; Store/SDK/publication stay on macOS. No
overlapping resource-sensitive work, no double lock acquisition in children.
Do not modify snapshot measurement inputs between building and finishing.
Validate complete scan counts, counters, byte conservation and final-phase
reconciliation; reuse prior semantic proofs for unchanged product behavior.
Only if the residual cannot be explained may one separate intrusive profile be
collected; never mix it into the phase/timing summary.

No performance fix, page-size/worker experiment, ablation, compaction, repack,
VACUUM, GC, release or deployment is part of this protocol. Deliver measured
root-cause attribution and a bounded next-step recommendation first.

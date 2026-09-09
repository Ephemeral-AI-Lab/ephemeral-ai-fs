# Tiny rewrite history v1

Status: exploratory baseline specification, 2026-09-09; not release admission.
Family `tiny_rewrite_history` has exactly one case, `tiny-history-30`.
No v0.1.5 product implementation is introduced. This local exploratory family
has no GitHub admission issue and cannot support a release claim.

## Fixed case

Case `tiny-history-30`: one Store, one Branch, one real-FUSE Workspace, three
source-like files of exactly 4/16/40 KiB (64/256/640 physical lines), no nesting.
Each line is exactly 64 bytes, with independently SHA-256-derived literal content.
Native-directory Init creates genesis. Each of 30 steps selects files round-robin
and changes eight ASCII bytes within one line; all sizes stay fixed. The ordinary
compiled importer writes the complete changed file through real FUSE, normalizes
metadata and returns through public Exec; explicit public Commit retains the state.
No SDK range-edit substitution. One source-only benchmark case and one seed.

## Boundaries and evidence

Use the unchanged packed schema-7/4-KiB-page production crates from
`cf3a058925c3012fda0fae922dc081766bb8fa99`, with only benchmark harness additions.
Record exact product/source/binary/image seals. Fresh Store, existing OS cache
uncontrolled. One observation is an iteration baseline, not a latency guarantee.
The existing storage-smoke runner owns topology, serial measurement lock, timeouts,
resource receipts and cleanup. Setup/container transfer are outside Exec/Commit
timers; physical Store receipts follow each acknowledgement.

Record initial/final and per-step allocated/apparent Store bytes, delta growth,
canonical/encoded accounting when available, Exec and Commit time separately,
CPU/RSS, staging, and cleanup. Keep all histories and failed attempts. No VACUUM,
GC, packing intervention, or changed page policy. Use 600 seconds complete
performance/verification phase and 30 seconds per operation; inherited memory,
PID, disk and container limits remain. No numeric speed target before baseline.

Run performance once, then reconnect and independently verify every byte/path in
genesis and all 30 commits against SHA-256 fixture oracles. Require exactly 30
Created commits, all 31 histories correct, and complete cleanup. A failure is
retained and fixed before the baseline is called complete. Add a fixture self-check
for sizes, line counts, one-file/eight-byte localized changes, and all oracles.

Baseline precedes v0.1.5 implementation. After implementing and verifying the fixed
128 KiB design, rerun this identical case and use it as the first optimization loop.
The larger realistic workspace and existing full157 cases remain later validation.

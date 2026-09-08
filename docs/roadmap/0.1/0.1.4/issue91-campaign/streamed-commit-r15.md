# R15 prospective streamed new-file Commit repair

Continue R6/README qualification from clean G7 product240609e6a. User authorizes
ordered investigation: new-file Commit streaming; bounded CPU preparation and
publication overlap; byte-identical direct pack assembly; pack sealing/leaf
occupancy; BLOB/write coalescing only when measured evidence warrants it.
Keep4KiB pages fixed, native FULL/PREFIX, canonical authentication, dependency
chronology, format/history, existing aggregate memory/transaction/queue limits,
public-operation timing and publication/durability contracts. No release waiver.

G7 payload500 total3,631,767,208ns includes Commit2,437,060,292ns;
output admission1,206,962,111ns, consumer idle983,754,160ns and admission
spill readback525,954,441bytes. Whole-file construction precedes selected output
emission. Reuse complete-file finality and the bounded FinalizedOutputWriter for
new files with no before inode, predecessor or captured construction. Keep edit
selection/predecessor paths unchanged. Root/result coverage may only succeed
after EOF/length validation; any read/producer/consumer failure drains and joins
workers and rolls back unpublished admission. Preview uses its existing private
sink. Do not remove authentication at persisted-spill boundaries.

First add a focused structural check demonstrating output reaches the consumer
before EOF without spill, exact root/canonical equivalence to private construction,
original slab/queue bounds, and late read/length failure with unchanged Store counts.
Validate Workspace tests, Store completion/rollback/resource tests, formatting and
warning-denying Clippy. Retain failing checks and exact dirty patches/source IDs.

Freeze distinct G8 product/source, host binary and image seals after correctness.
Use the unchanged R6 four cells/order, seed1, one observation and independent proof
per cell: namespace100, payload100m, namespace100000, payload500m. Preserve
300/310/600s and45/59s budgets and R4 proof preparation. Existing fixtures,
public SDK/FUSE operations, normalization and timers remain unchanged. Retain
all earlier observations/failures; no statistical claim or improved acceptance
baseline. Compare published v0.1.3 and separate G7 diagnostic operands, including
elapsed/Commit/CPU/RSS/I/O/spill/allocation/cleanup together. All builds/checks and
measurements use the nonblocking shared measurement lock, fresh writable owners,
macOS Store/coordinator and Linux daemon/FUSE/workload ownership.

Later treatments require separately committed amendments defining their concrete
change, resource accounting, correctness gate and selected observations before
collection. Reject unsafe/infeasible overlap rather than changing limits. Final
qualification still requires every affected original member, strict gates,
phase1 guarantees, equal-retained-state new-candidate full157 and independent
review.4KiB/storage savings are binding, even if64KiB diagnostic is faster.

# Reopened sequence: retention crossing and bounded cost cycles

The current public sequence completed 33 workspaces, 33 Commits and 33000 SDK
edits over the unchanged 100000-file / 500000000-byte namespace input. Performance
command wall was **115.304552792 s**, with **109.409753707 s** in the complete
workspace chains. Its independent replay proof passed in **166.364111625 s**
command wall under the explicit sequence-only 600/614-second verification policy.
Cleanup passed in both runs. This is current-code qualification, not a paired
before/after tail-recovery latency comparison.

Source, commands and raw observations are recorded in
`benchmark-results/host-store/issue118/20260912/reopen-retention-commands.json`,
`reopen-retention-public/perf.jsonl` and
`reopen-retention-proof/verification.json`. The executable/image source seal is
`e162aecd2b351811b7fd49aa1bb57638c9a89681cda59df4ca97dc32e172ca17`.

The proof checked every changed file's bytes and length before and after a fresh
reopen on all 33 steps, plus 100 declared unchanged-file samples per step. It also
checked the initial snapshot and three retained snapshots, with 1000 changed
files per snapshot. It does not claim an exhaustive unchanged-namespace oracle.

## Measured observations

| Step | Values before reopen | SDK edits | Commit |
| --- | ---: | ---: | ---: |
| 0 | 100002 | 1.519595477 s | 0.723436792 s |
| 16 | 116002 | 2.664069732 s | 1.625540458 s |
| 17 | 117002 | 1.459093495 s | 0.842712709 s |
| 31 | 131002 | 2.430608106 s | 1.680824833 s |
| 32 | 175383 | 1.507783933 s | 0.752157000 s |

Every observation through step 31 equals `100002 + 1000 × step`. Step 31's
Commit then adds **44381 physical metadata values**, exceeding the preceding
1000-value increments by **43381**. The final reopen therefore demonstrably
crosses the 131072-value retention boundary. Catalogue observation time is
recorded explicitly and remains inside the measured reopen/chain lifetime.

Logical work stays fixed: 1000 dirty/probed nodes, zero clean namespace visits,
569 snapshot database calls, 2321 rows, 9253481 returned canonical bytes,
4177577 spill-readback bytes and two admission transactions per step. These
counters do not count every internal metadata pool fetch or decoded DELTA base.

## Source-based explanation and limits

`objects/metadata.rs::ValueIndex::sync_to` deliberately clears the entire scratch
fingerprint index when adding the next whole group would exceed 131072 indexed
values. `objects/admission/metadata_values.rs::prepare_values` then assigns new
ordinals to values absent from both the remaining index and the current pending
map. A changed inode leaf contains unchanged neighboring values as well as the
edited inode. Thus eviction can re-intern old values when those leaves are
published. The code explicitly documents duplicate physical values after
eviction as the tradeoff for bounded indexing; canonical reconstruction is
unchanged. The 44381-row jump is measured. Attribution of its 43381 excess rows
to re-interning follows from this mechanism and the fixed edit pattern; the run
did not independently compare every duplicated value's bytes.

The tail-recovery change preserves that existing eviction policy and first
winner. At 175383 historical values, with group sizes at most 165, the first
discarded prefix contains 130908–131072 values. A fresh index therefore needs
44311–44475 retained values, approximately one quarter of the old full replay.
The exact retained count requires group headers not recorded by this public
observer. Earlier component proofs compared the full original recurrence and
exact resulting index rows. This public run supplies the actual crossing and
reopen correctness; it does not supply a paired wall-time speedup. The header
scan remains O(groups), and the surviving Store foreign-key scan remains linear.

`objects/read.rs` limits metadata DELTA chains to 16 edges and 128 KiB of canonical
closure. Admission accepts a metadata predecessor only while its depth is below
16. Reading the deepest predecessor and emitting a new FULL representation
explains the first high-cost boundary followed by the step-17 reset: namespace
time falls from 0.446748333 s to 0.167701708 s, and object admission from
0.887228334 s to 0.485103334 s. This is a source-supported interpretation of the
bounded cycle; per-leaf depth counters were not collected.

The step-31/32 drop also coincides with index eviction. Reassigned pool ordinals
change physical leaf bytes and can make existing DELTA candidates less useful,
causing earlier FULL selections, while tail recovery reduces initial index
payload work. These are consistent explanations, not separately measured shares
of that second drop. There is no evidence here of unbounded namespace traversal,
and no justification to raise chain, cache, memory or capacity limits.

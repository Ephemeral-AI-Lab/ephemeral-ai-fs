# Shared admission optimization experiment (#109 / #108)

Prospective declaration, 2026-09-11. Owner request: borrow the shared mechanisms
from [the #108 diagnostic](https://github.com/Ephemeral-AI-Lab/layerfs/issues/108#issuecomment-5623008277),
implement namespace Init improvements, and check transfer to other families.

## Applicable findings and treatment

Repeated SQL parsing is common to Init metadata pooling and historical
read/admission. Reuse existing rusqlite prepared-statement caching in
`metadata_group`, `next_metadata_ordinal`, `ValueIndex::find`, publication MAX
queries, and pool catalogue INSERTs; prepare scratch INSERT once per existing
transaction. Avoid the eagerly evaluated ordinal fallback when the caller
already supplied a next ordinal. Keep SQL values/results and transaction order.

Combine this with #109's duplicate SmallContent signature fix: retain the
already computed signature for selected FULL publication, including lazy
fallback for explicit-predecessor paths. Charge added prepared-object capacity
under existing limits. Preserve authentication, CAS, candidate order, rollback,
formats, compression/DELTA selection and worker count.

#108's daemon round trips and existing-history chain depth do not transfer
unchanged to fresh Init. No daemon protocol, codec policy, decoded-content cache,
transaction grouping, storage format or workload change is included. Benefits
are generic at the shared functions; this does not imply every family gets a
speedup, or predict an exact speedup from profile shares.

## Source and build custody

Starting HEAD: `22e9b46bda4da500fbfca9242f87eac9e32573e0` on `main`.
The working tree also contains the separately requested, uncommitted compaction
removal. Freeze it unchanged as part of all arms; this comparison does not
attribute its effects to the new optimization. Starting patch, status and dirty
file hashes are retained outside the repository. Only this task's changes will
be committed. Refuse or record a custody failure if unrelated native inputs move
during a qualified build.

Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue109-evidence/implementation-20260910T180813Z`.

Build and archive four host arms with the existing qualified runner:
C (unchanged working-tree product), A (signature only), B (SQL only), AB (both).
Each native-input seal owns an independent target; compiler flags/profile and
Rust 1.85.1 remain unchanged. Retain actual binary hashes, source/native seals,
linked schema/ordinary-format probes, exact commands, logs and wall time. Build
Linux images for C and AB through the existing runner; never relabel an image
with a different product seal. Builds/selected runs own the existing measurement
lock; a direct diagnostic parent owns it only when the child does not.

## Selected Init experiment, frozen before edits/timings

Use the existing verified 500-MB / 100,000-file fixture at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue109-evidence/historical-clock-20260910T171735Z/fixture`,
digest `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.
Every invocation receives a fresh independent Store. Source bytes are reused;
OS cache is uncontrolled, with no cold-cache claim and no cache deletion.
Use existing `namespace-init-diagnostic` with the supported
`reused-subsequent-sample-uncontrolled` label and no seed override.

After all four archived host binaries exist, run clean ordered comparisons:
C,A,A,C; C,B,B,C; C,AB,AB,C. This is two observations per candidate and two
corresponding controls, preserving the declared alternating pair order. Keep
every attempt; do not select the best number. Then one nonce-instrumented C and
one AB run, separately labelled to explain remaining phases, not pooled with
clean measurements. No broad profiler run is planned unless a new unexplained
failure warrants a separately declared diagnostic.

Report `layerstack_init_ns`, setup/teardown and external command wall separately;
record file/byte counts, canonical counters, allocated/apparent Store bytes,
CPU/RSS, binary and fixture identities. Primary target <=2.7 s remains open if
unmet. Report pair observations and median/range; n=2 is bounded diagnostic
confirmation, not statistical proof. No performance claim derives from build
wall, instrumented observations or the historical unpaired v0.1.3 receipt.

## Generic transfer and correctness

Use existing qualified C/AB host+Linux pairs and the shared runner. All selected
cases use seed 1, full workload, ordinary uncompacted Stores, existing topology
(macOS SDK/SQLite; Linux daemon/FUSE, 2 CPUs/2 GiB), and existing timers/verifiers.
Reuse protected prepared input using the runner's compatibility and digest
checks; record the same input identities across arms or mark comparison invalid.

For each of `dedup-history-distributed-500`, `dedup-history-recurring-500`, and
`dedup-history-unrelated-500-mixed-v2`: run C,AB,AB,C in that order. Use
`--perf-fast --collection-mode --product-timeout 300 --timeout 310
--setup-timeout 600`; these are collection allowances, not relaxed historical
15-second targets. Report deadline misses and per-operation counters. Verify AB
once per case using the existing independent family verifier.

Qualify AB once on every registered Init tier (100/5 MB, 1,000/20 MB,
10,000/300 MB, 100,000/500 MB) with its existing performance selection and
independent proof. These qualification rows are separate from selected ABBA
comparisons. Run focused Store admission tests covering SmallContent
selection/fallback/late-CAS/rollback/reopen and metadata pool authentication,
chronology, sharing and invalidation. Add only missing focused coverage; no
benchmark-case branch or relaxed contract is permitted in product code.

This is a focused optimization/qualification, not a restart of #104. Preserve
all sealed evidence. If any comparison cannot be matched or proof cannot run,
retain the evidence, mark that claim pending, and state the exact limitation.

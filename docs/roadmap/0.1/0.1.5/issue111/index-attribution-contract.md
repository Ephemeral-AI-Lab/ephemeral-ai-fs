# Shared metadata index attribution protocol

Diagnostic-only study after the bounded endpoint fix. Isolate current ee3545650
source plus preserved dirty work; no performance optimization. Instrument index
creation, synchronization groups/values/evictions, INSERT binding/execution,
lookup prepare/bind/step/result extraction, SQLite VM work and cache hit/miss/
write/spill counters. Keep exact values, policies, bounds and public operations.
Fine-grained clocks have overhead; do not compare their absolute timings with
plain historical samples or subtract invented overhead estimates.

Evidence root is frozen before edits in `/tmp/layerfs-index-attribution-root`.
Original evidence is read-only. Build qualified host/image from the isolated
source under the runner measurement lock. Serialize all resource-sensitive work.
All commands, raw stdout/stderr, exits, attempts and source/fixture seals retained.

Two cohorts:

1. New Store, full namespace-100000 Init: n=2 nonce diagnostics, fresh outputs,
   independent fixed zero-residency cold acquisition each time, full fixture
   digest6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e.
   Record cache profile/read bytes, phase clocks and detailed index counters.
2. Incremental Commit: all four current registered Init tiers, n=2 independent
   Stores per mode, retained/reopened order R1,O1,O2,R2 at each tier ascending.
   Each Store starts from full public Init/fork. O drops every Client/Store owner
   and reconnects before workspace creation; R retains them. Both create a Fuse
   workspace and use public SDK range edit to replace10 bytes of d0000/f000000,
   with the same offset and marker. Public Commit alone is timed. A second edit
   with a different marker and Commit on the same session is a separate retained
   follow-up cohort, not a replacement for the first pair. Thus32 Commit calls.

Reopened means a fresh derived index, not cold OS pages. Declare OS cache state
uncontrolled for Commit comparisons. Snapshot create/edit/Commit separately to
detect any index work before Commit. Verify Created results, visible heads and
the edited file's complete canonical bytes after each Commit, outside timing;
end cleanly and verify final root/content after Store reopen. Reuse the same
immutable fixtures; no direct fixture mutation or Store clone shortcut.

Retain full namespace count/byte receipts. Report index work versus existing
namespace size and first versus second Commit; do not claim an unmeasured long-
history or >131072-retention-bound result. No warm/cold pooling, target passes or
speedup percentages: all samples are nonce, ineligible for qualification.
Cold plain <=2.7 s remains open.

No timing-based retry. One whole affected pair/cell replacement only for proven
infrastructure invalidity, retaining originals. Failures in correctness or
source custody stop collection. Run focused metadata/admission checks for raw
binding instrumentation equivalence; retain all correctness checks and resource
bounds. SDK/SQLite/spool remain on macOS, Docker only daemon/FUSE/workload.

Deliver measured attribution and one evidence-led next-step recommendation.
No restartable index, new cache, schema/page-size/DELTA/authentication change,
product promotion, release or deployment in this diagnostic stage.

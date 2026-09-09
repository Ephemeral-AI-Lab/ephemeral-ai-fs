# v0.1.5 issue #100 implementation and verification plan

> **Issue #100 measured outcome, 2026-09-10:** The retained implementation allocates
> **49,319,936 bytes**, with **49,250,304 bytes** growth, ten Created outcomes,
> exact same-Store verification and clean teardown. It is **4,319,936 bytes above
> 45,000,000** and is **not near-target**. Commit median/sum exceed the prospective
> 10% working criterion; save/paired medians and historical-read wall remain close
> to the original baseline. See [the consolidated results](issue100/storage-optimization-results.md)
> and [complete retained-candidate evidence](issue100/retained-candidate-1-results.md).
> The owner subsequently authorized full157 despite the ten-state target miss.
> [Full157 confirmation](issue100/retained-full157-results.md) completed at
> **134,246,400 B**, with157 Created outcomes, same-Store verification and clean
> teardown. It saves27.27% versus released control but has31.13% higher paired
> median latency. The issue remains open; no release-admission PASS or release.

The original ten-file/thirty-commit implementation loop is historical; its
[31-state report](smoke-report.md) and retained attempts remain valid for that case.
They do not override the current [ten-snapshot contract](issue100/ten-snapshot-contract.md).
Continue `codex/issue100-full157` with existing v0.1.5 and its upper-range exact-CAS
fix. Do not restart from the released v0.1.4 control or overwrite other tasks.

## Accepted source ownership

| Owner | Responsibility |
| --- | --- |
| `layerfs-content/src/file/content.rs` | Unchanged canonical SmallContent v1, strict nonempty <131072-byte decision and regular-file dispatch. |
| `layerfs-layerstack-store/src/objects/admission.rs` | Exact CAS first; genuine/removed-name predecessor or one bounded cached FULL; one FULL and at most one DELTA; complete serialized cost and prospective closure limits. |
| `objects/read.rs` | Shared authenticated historical/exact-CAS reader; iterative kind-2 chain; old kind-1 FULL-only fast path. |
| `objects/delta.rs` | Pack-v3 FULL kind 0, original DELTA kind 1, schema-9 bounded DELTA kind 2; explicit tags and fixed limits. |
| `objects/pack.rs` | Existing fixed Zstandard settings/static workspaces and strict pack/group/frame grammar. |
| `schema.rs`, `statements.rs`, `sql/schema/` | New schema 9; supported nonpromoting schema 6/7/8 opens/writers; explicit offline 7/8→9 upgrade. |
| `query.rs` and existing admission session | Transitive physical-base retention/integrity, stable selected locations and private rollback. |
| Existing Init/Workspace/live-state owners | Shared construction, authoritative FUSE/SDK facts, POSIX behavior and existing public outcome semantics. |

Paths above are relative to `crates/` except module shorthand following its crate.
Existing namespaces, file roots, inode metadata, CDC/extents and public transport
remain intact. Preserve #95 bounded Init comparison reuse, #98 Workspace SQL
cohorts/staging handoff, and <=64-KiB ordered spill read-ahead.

## Accepted format and resource limits

Canonical identity is unchanged. Kind 1 always points directly to FULL; kind 2
requires schema 9 and an already selected bounded SmallContent base. The retained
fingerprint cache admits only FULLs, emitting kind 1; removed-name hints may
reach bounded DELTA predecessors through kind 2.
At most 8 dependency edges, 512 KiB summed canonical closure including target,
and 256 KiB actual retained encoded capacity are allowed. A missing/ineligible
optional predecessor or prospective bound refusal selects the prepared FULL.
Corrupt/missing required persisted dependencies remain integrity errors.

The reader iteratively checks roles, lengths, cycles, chronology and complete
closure, then authenticates every reconstructed node. Clear the old small-base
read cache for this path. Its 2-MiB active allowance includes <=1 MiB static
decoder/dictionary storage, <=256 KiB encoded records, four raw/canonical-sized
conversion/base buffers, bounded associations, and a surviving admission target.
Acquisition of another group is checked separately before decoder allocation.
The 3-MiB encoding allowance retains the actual 2-MiB static encoder and <=1 MiB
combined target/base/FULL/DELTA/output handoff. Preserve the existing surrounding
producer/index/read-wave/physical-output ownership ledgers.

Supported old schemas remain unchanged on open. Schema 8 writes the original
one-level policy; explicit upgrade uses read-only preflight, exclusive revalidation
and DELETE/FULL transactional promotion to schema 9 without payload/page rewrite.
Failures retain accurate pre/post-publication outcomes. Normal Store durability
semantics do not change; downgrade requires a pre-upgrade backup.

## Current continuation

1. Reuse source/harness/fixture/environment-applicable immutable three-arm controls.
   The exact ten retained full157 indices remain 1, 18, 36, 53, 70, 88, 105, 122,
   140, 157. Do not replay skipped states or collect an unchanged control again.
2. Diagnose measured FULL population using a handful of original fixture families.
   The removed-name catalogue, compact selected-FULL cache and retained-admission
   handoff are implemented. Both DELTA-cache variants were rejected and reverted.
   Cross-CDC reuse remains unimplemented. Amend eligibility, memory, authentication,
   fallback, retention and format contracts before encoding a changed alternative.
3. Keep the smallest evidence-supported change and focused regressions. Shared
   historical/exact-CAS reader changes require the existing upper-range CAS check.
   No broad Cargo, Clippy, doctest or unrelated qualification suite is requested.
4. Build affected artifacts through the existing host/image runner entrypoints.
   Use shared Cargo/BuildKit caches, two Cargo workers, existing serialization and
   900-second build watchdog. Runner commands own their lock; do not nest another.
   Markdown-only edits do not require rebuilding.
5. For substantive candidates run fresh public `deepseek-ten` performance, freeze
   allocation/Store identity, collect read-only census, and reopen the same Store
   with a new coordinator for original-oracle history verification and cleanup.
   Preserve actual outcomes, all paths/types/modes/symlinks/bytes, and exact seals.
6. Compare both LayerFS baselines plus Git with its narrower metadata/timer scope.
   Report exact allocation/growth, per-step save/Commit/paired timings and sums,
   historical verification, CPU/RSS, container/staging/spool scopes, and separate
   build/setup/transfer/cleanup. Retain failed/rejected patches and evidence.
7. The owner explicitly superseded the near-target gate and requested full157.
   That confirmation is now complete with qualified released-control reuse; see
   [the report](issue100/retained-full157-results.md). Do not repeat unchanged
   passing history. The45-MB objective remains specific to ten retained states.

Keep the smoke's 600-second phase/preparation and 120-second operation/setup/
cleanup watchdogs; full157 retains its separate current bounds. Diagnose the real
build, lock, transport or product owner instead of inflating timeouts. Do not
rerun unchanged passing evidence for a better observation.

## Evidence and qualification boundary

The chain improvement is material but insufficient for this target. Preserve the
initial full157 regression (201,371,648 B versus184,582,144 B) as historical
evidence. The new independent full157 confirmation is134,246,400 B; its
foreground speed regressions remain explicit. A ten-state result cannot replace it.
No claim is made that all other bounded designs are incapable of meeting the goal.

The [chain result](issue100/chain-1-results.md) owns raw timing/resource/custody
observations. The prospective speed/RSS working criteria in the amendment are
engineering comparisons, not owner-approved release gates. A 45–46-million-byte
result would be near-target, not exact 45-million achievement. Correctness,
resource and cleanup failures are never tolerated. No fabricated tail confidence,
release-admission PASS, release tag or publication follows from this smoke.

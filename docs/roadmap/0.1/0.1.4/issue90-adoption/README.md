# Storage adoption phase 1 (#90)

Status: repairs and validation in progress. This is a new adoption candidate,
not the producer of #88's storage measurements. Issue #91 owns performance
qualification. No PR merge, migration, rollout or full benchmark campaign is
part of this work.

## Integration and source ownership

The isolated `codex/issue90-storage-adoption` worktree begins at research
`9a2b34851`, incorporates `origin/main` at `49f0f5be9`, and reconciles the committed
#71 work through `8d15ebc9b`. The original research branch and evidence remain
unchanged. #71's uncommitted proof/report work is excluded. Its bounded frontier,
shared metadata components, small-file construction, sorted directory builder
and streaming live checkpoint repair are retained. Their measurements do not
qualify this combined source.

The initial inspection found unrelated documentation changes in the main
worktree and #71 proof changes in its worktree. Neither was reset or edited.
No LayerFS build/workload or Store owner appeared in the initial process/lsof
snapshot. One coordinator owns resource-sensitive checks through the existing
`$TMPDIR/layerfs-infra-measurement.lock`; workers perform source edits/review only.

## Supported compatibility

| Store | New adoption binary | Earlier schema-6 binary | Released schema-5 binary |
|---|---|---|---|
| Legacy schema6, version1 packs, 4KiB or64KiB pages | Read/authenticate/write with legacy encoding; no header change | Existing supported read/write | Reject unsupported schema |
| Fresh native schema7, 4KiB pages | Read/authenticate/write native and legacy records | Reject at read-only version preflight | Reject unsupported schema |
| Research native packs labeled schema6 | Reject without mutation; preserve isolated evidence | Unqualified historical research behavior | Reject unsupported schema |
| Unknown schema, incompatible layout, WAL | Reject without mutation | Outside supported matrix | Outside supported matrix |
| Schema4/5 | Reject; no automatic migration | Outside this adoption contract | Historical release-specific contract only |

Native adoption is **fresh-Store-only**. Existing supported legacy6 Stores remain
legacy on every write; there is no automatic promotion, recompression, repack,
header upgrade or downgrade. Schema7 changes the explicit Store-format marker;
its SQL objects match6, and canonical bytes/IDs remain unchanged. The explicit
v7 creation SQL is sealed independently of historical v6 SQL. Preflight rejects
unknown versions before writable connection configuration. On legacy6, a
bounded-memory SQL header check rejects native/research or unknown pack versions.
Native7 readers preserve legacy FULL/DELTA dispatch and native dependencies;
legacy FULL can be a native PREFIX base, while unsupported base kinds still fail.
Manually relabeling a native Store as6 is not a supported conversion.

## Publication and resource behavior

One existing Store, canonical CAS/selected-index check and synchronous publisher
remain. An admission owns the existing writer operation permit from the first
probe through final publication or retained Workspace staging. Other writers
queue; reads can continue. Complete or drop an admission token before making
another synchronous mutation on the same Store from the owning thread; mutation
ownership is non-reentrant. Independently committed batches remain owned until
publication succeeds. On failure/abandonment, selected IDs and packs newer than
that owner's starting pack maximum are reclaimed in bounded512-row transactions.
Preexisting immutable packs and their physical bases are preserved. Workspace
staging intentionally retains its root when a later branch publication fails.

Cleanup errors are surfaced on fallible paths and quarantine further writes on
the affected Store owner. Existing readable state remains accessible. This is
not a new crash/power-loss recovery or fsync promise; MEMORY journal/synchronous
OFF and the existing durability limitation remain. No physical file shrink is
promised: reclaimed SQLite pages can remain on the freelist.

The effective admission batch is capped at the existing512-object slab count,
inside the public8191-object ceiling. The physical2MiB and canonical/data6MiB
reservation guards remain enforced. Spill partition accounting includes each
local ID buffer and the shared ID buffer, within the existing1MiB allowance.
Selected consumption follows authenticated child-first graph order.

## Ingestion coverage

Regular-file context is explicit and scoped. Both direct initialization sinks
transport FILE provenance and span facts. Shared checked-file construction
restores context before metadata construction and after errors. Public content
Write and localized Splice do the same inside regular-file rope construction;
metadata callbacks remain outside that scope. Existing Workspace capture,
complete-file and predecessor paths retain their authenticated hints.

Generic rope chunks, metadata values, symlinks and arbitrary user byte prefixes
never establish FILE eligibility. Empty initialization has no regular payload.
Hard-link/special-shape fallback uses the same checked admission and scoped file
construction; it is an explicit construction fallback, not a hidden legacy
encoding omission. Public namespace metadata, symlinks, hard links, canonical
identity, CAS/dedup, retained roots and ordinary reopen/read remain required gates.

## Historical evidence and phase-2 questions

The original research results remain attributed to their recorded producers:
335,552,512B original M4.5 allocation;218,116,096B fresh C+S1 control;
184,598,528B candidate allocation;181,710,848B logical SQLite;
155,353,550B complete packs. Both arms passed157 historical checks. The separate
60-read diagnostic covered five small single-extent files, not general large
files, cold caches or tails. None of these quantities is a measurement of this
repaired candidate.

Keep the historical23.19% increase in verification read/traversal/digest elapsed,
26.17% host CPU increase, greater Commit write traffic, earlier read regressions
and outliers, and25.70ms depth3/full versus12.38ms paired control. Phase2 must
measure the new eligibility, batch formation and admission serialization as well
as encoding. A recommended matched control shares all correctness/#71/format
repairs and differs only in prospective, explicitly sealed native-encoding
selection. The old C+S1 producer alone is not an automatically matched control.
The exact matrix, acceptance, source differences and binaries must be frozen by
#91 before collection; this task starts none of that campaign.

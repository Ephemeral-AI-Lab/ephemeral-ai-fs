# LayerFS v0.1.5: bounded delta storage

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

Read the [bounded predecessor amendment](issue100/bounded-predecessor-amendment.md),
[the removed-name amendment](issue100/removed-base-amendment.md),
[compact cache](issue100/compact-candidate-amendment.md),
[retained cache handoff](issue100/retained-candidate-amendment.md), and
[frozen ten-snapshot contract](issue100/ten-snapshot-contract.md) for the current
work. The earlier [31-state smoke report](smoke-report.md) remains historical
first-round evidence. Its restrictions and the old [implementation prompt](implementation_prompt.md)
do not supersede the current issue #100 owner instructions.

## Current implementation

New or changed nonempty file content strictly below **131,072 bytes** uses one
whole-file canonical SmallContent CAS object. Empty files retain their compact
representation; files at/above the boundary retain CDC **8/16/32 KiB**, the existing
extent tree and known-range edit locality. Canonical identity stays independent
of physical FULL/DELTA representation. Existing objects and histories are not
rewritten.

New Stores use **schema 9** and unchanged 4-KiB pages/seven-table layout. Pack-v3
kind 0 is FULL; kind 1 still means one DELTA edge to FULL. New kind 2 permits an
immediate SmallContent predecessor chain bounded by **8 edges**, **512 KiB total
canonical closure including target**, and **256 KiB retained encoded capacity**.
The iterative reader authenticates each reconstructed node. Exact CAS runs before
encoding, FULL is prepared once, and one eligible DELTA wins only on strictly
smaller complete encoded cost.

Supported schema 6/7/8 opens remain nonpromoting and retain their respective
writer policies, including schema 8's one-level FULL-anchor grammar. Explicit
offline upgrade accepts schema 7/8 to 9; old binaries reject schema 9 before normal
mutable open. Reverting requires a pre-upgrade backup, not a header downgrade.

Init and Commit retain their shared construction/admission/publication pipeline,
authoritative live FUSE/SDK state, POSIX behavior, retained physical dependencies,
rollback and actual publication outcomes. Preserve #95 Init comparison reuse,
#98 Workspace SQL coalescing/staging handoff and ordered spill read-ahead <=64 KiB.
The existing 2-MiB active reconstruction and 3-MiB encoding allowances include
static codec storage and simultaneous operands; there is no hidden heap fallback.

## Evidence and remaining work

| Ten-snapshot arm | Final allocated bytes |
| --- | ---: |
| Git | 38,223,872 |
| Released v0.1.4 | 67,145,728 |
| Existing one-level v0.1.5 | 66,105,344 |
| Schema-9 chain candidate | 56,668,160 |
| Chains plus session-local selected-FULL cache | 54,562,816 |
| Removed-name discovery | 50,372,608 |
| Compact cache | 50,368,512 |
| Retained FULL cache (kept) | 49,319,936 |

Chain allocation is **11,668,160 B above the target**. All values are allocated
bytes, not compressed-frame totals. The detailed report retains exact growth,
per-state timings, resource scopes, attribution and custody. Ten dependent history
steps do not establish tail confidence or a release-admission PASS.

The initial full157 candidate regressed to **201,371,648 B**, versus released
control **184,582,144 B**, with both histories verified. The ten-state improvement
does not erase that result. The subsequently owner-authorized full157 run
verified the retained implementation at **134,246,400 B**, reversing the storage
regression while showing higher foreground latency. Its45-MB target remains
inapplicable; see [full157 results](issue100/retained-full157-results.md).

The earlier recent-128 ring was diagnostic only. The compact content-keyed
selected-FULL cache is implemented and measured. It stores 1024 candidate records
and 8192 u16 references within 128 KiB from the existing admission index allowance,
with no raw content. Retained admissions transfer this same cache to the next
admission on the same StoreDb; failed admissions discard it and reopen starts
empty. Both session-only and retained DELTA-cache expansions were measured and
reverted because they increased content packs and Commit cost.
Cross-CDC predecessor reuse also remains unimplemented. The target miss does not
prove every owner-authorized bounded design impossible. No global similarity
index, GC/repacker, reverse rewrite, new dependency, codec/page/cutoff sweep or
release publication is authorized by these results.

## Working documents

- [Specification](spec.md): canonical/physical grammar, compatibility and ownership.
- [Workflow](workflow.md): shared public paths, history and size transitions.
- [Implementation plan](implementation_plan.md): current continuation and measurement boundaries.
- [Implementation notes](implementation-notes.md): accepted paths and retained historical attempts.
- [Ten-snapshot baselines](issue100/ten-snapshot-baselines.md) and
  [Git gap study](issue100/git-gap-directions.md): immutable controls and measured opportunities.
- [Past mistakes](past_mistake.md), [benchmark success](benchmark_success.md), and
  [released closeout](benchmark-v0.1.4-report.md): invariants and qualification limitations.

Continue the current `codex/issue100-full157` implementation and upper-range exact-CAS
fix. Released v0.1.4 is the immutable control, not a replacement starting point.
The original [tiny baseline](tiny-history-baseline-v1.md) and first-round reports
retain their own fixtures and numerical scopes. Broader release qualification
remains unrun; ordinary MEMORY/OFF acknowledgements gain no power-loss guarantee.

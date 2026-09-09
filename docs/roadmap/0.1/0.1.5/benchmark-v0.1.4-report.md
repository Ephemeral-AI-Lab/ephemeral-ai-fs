# v0.1.4 benchmark report and deferred v0.1.5 optimization

> **Status:** Recorded v0.1.4 results; optimization follow-up scheduled after the
> main v0.1.5 implementation task is complete. Added 2026-09-09. No new benchmarks
> were run for this document.

## Decision and execution order

v0.1.4 improved storage efficiency and removed several expensive redundant paths,
but its optimization outcome is incomplete. Substantial latency regressions
against v0.1.3 remain. Successful execution, correctness verification and owner
acceptance of the release do not establish satisfactory performance across the
benchmark families.

**Finish the main v0.1.5 implementation first.** Follow [spec.md](spec.md) and
[implementation_plan.md](implementation_plan.md): implement the fixed whole-file
delta policy, prove correctness and ownership, and perform the required comparison
against the already collected tiny-history baseline. Fix correctness failures and
satisfy that task's existing acceptance gates. Then address the broader performance
backlog recorded here on the resulting implementation.

This follow-up must not displace the main implementation with another v0.1.4
optimization campaign. It also does not waive correctness, memory, storage or
latency gates already required by the v0.1.5 spec. Preserve qualified #95/#98
repairs while implementing; see [past_mistake.md](past_mistake.md). This document
schedules investigation and measurement, not a promise that every historical
regression has a safe fix or a new set of numerical acceptance thresholds.

## Source and comparison boundaries

- Released v0.1.4 commit: `101fa273d815f3aaedb0e06ba0de7b0777d83def`.
- Qualified product source: `9cfb4be477116646258ea0621280ed13b1824c6d`.
- Qualified evidence head: `856baab0caf0522db4757cb4dcbfb0d9c43e30d4`.
- Host benchmark SHA-256: `47b4d44e3e961f3b6a57d2c18dc6d195973133dbdfb87681a55f1b8e07abe3a2`.
- Sealed benchmark image: `sha256:42eb806fcfe9db1dc28098eb423340de3e4eaf81fd487389128305e65606d347`.

The released tree preserves the qualified production code; later changes package
reports and release documentation. Existing v0.1.5 tiny-history observations keep
their own original source and contract. They must not be relabeled as measurements
of this release. The full157 supplemental storage control is separate from the
published v0.1.3 timing baseline.

Sources: [released case-by-case report](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/release-notes/0.1.4/benchmark-closeout.md),
[qualified #98 evidence](https://github.com/Ephemeral-AI-Lab/layerfs/blob/856baab0caf0522db4757cb4dcbfb0d9c43e30d4/docs/roadmap/0.1/0.1.4/issue98/README.md).

## What passed and what did not

| Scope | Recorded result |
|---|---|
| Performance execution | 198/198 successful executions |
| Routine independent proofs | 226 passed |
| Optional 600-second endurance proof | Not run, as declared |
| Native checks | 411 tests passed, plus explicit ignored spill check; doctests, formatting and warning-denying Clippy passed |
| Supplemental workloads | Small-files and all four SDK/FUSE frequent-edit variants passed execution and verification |
| Full157 | 157 performance states, 157 historical proofs, 158 checkpoint/accounting records and cleanup passed |
| Elapsed comparisons | 49 SEVERE, 95 REVIEW, 31 OBSERVED_INCREASE, 19 NO_INCREASE, 4 INELIGIBLE |
| Absolute targets | Unrelated-history-500, Git-100 and Git-500 still missed |
| Overall comparison report | INCOMPLETE: four historical Git fixtures have image-bound identity mismatches |

The four Git cases ran and verified successfully; their historical speed
comparisons remain ineligible. Do not repair that classification by ignoring
fixture identities or editing old observations. Historical severity labels remain
valid even though the owner accepted and published v0.1.4.

## Results by family

Each performance case has one recorded sample. Values below are descriptive sums
of individual case timers, not campaign wall time, throughput, paired speedup
estimates or latency distributions. Some families contain different operation
timers. Positive changes mean slower than the eligible published v0.1.3 checkpoint.
The store-footprint row reports elapsed time, not allocated bytes.

| Family | Performance cases | Passing proofs | v0.1.3 sum (s) | v0.1.4 sum (s) | Change |
|---|---:|---:|---:|---:|---:|
| payload_create_read | 8 | 8 | 4.179502 | 3.872964 | -7.33% |
| dedup_workspace_reuse | 14 | 14 | 15.874997 | 14.074111 | -11.34% |
| dedup_cross_file | 10 | 10 | 1.402381 | 3.154912 | +124.97% |
| dedup_cdc_locality | 20 | 21 | 1.484589 | 3.187850 | +114.73% |
| edit_length_preserving | 12 | 12 | 0.082806 | 0.119696 | +44.55% |
| edit_length_changing | 32 | 32 | 0.230456 | 0.306447 | +32.97% |
| edit_canonical_chunk_count | 12 | 12 | 0.092536 | 0.135438 | +46.36% |
| init_namespace | 4 | 4 | 3.053609 | 4.903439 | +60.58% |
| store_footprint | 6 | 6 | 8.156506 | 12.273350 | +50.47% |
| tiny_file_churn | 20 | 20 | 8.529795 | 8.996516 | +5.47% |
| namespace_mutation | 4 | 4 | 0.314367 | 0.429782 | +36.71% |
| directory_construction_traversal | 12 | 12 | 7.628812 | 9.710653 | +27.29% |
| workspace_change_locality | 16 | 16 | 10.971227 | 13.240867 | +20.69% |
| dedup_branch_history | 20 | 20 | 47.049815 | 55.856741 | +18.72% |
| git_tool_workflow | 4 | 4 | — | 8.877153 | INELIGIBLE |
| mixed_load_bearing | 4 | 4 | 8.559964 | 8.592193 | +0.38% |
| workspace_reliability | 0 | 27 | — | — | — |

Workspace reliability additionally has one declared optional unrun proof. Every
individual case, timer, target and severity is retained in the
[complete report](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/release-notes/0.1.4/benchmark-closeout.md),
[performance CSV](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/release-notes/0.1.4/benchmark-performance.csv) and
[verification CSV](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/release-notes/0.1.4/benchmark-verification.csv).

## Measured inefficiencies and repairs already retained

| Mechanism | Evidence | Disposition for v0.1.5 |
|---|---|---|
| Repeated duplicate reconstruction/authentication across Init batches | #95 identical-500 native decodes fell 24,883→56 while exact collision checks remained 29,999; adjacent elapsed 704.759→123.357 ms. CDC overwrite 740.774→146.270 ms. | Preserve bounded authenticated operand reuse and all exact validation. Recheck mixed working sets after implementation; do not reimplement the completed repair. |
| Unique-content SQLite publication | Unique-500 profile: 293 commit samples, 267 in `guarded_pwrite_np`; original diagnostic commit about 679 ms of 1,443 ms elapsed. | Remaining measured cost. Profile the resulting v0.1.5 path before choosing a write-pattern change. No proven SSD-latency or B-tree root cause. |
| Native compression | Unique-500 profile included 160 native-compression samples. | Separate remaining cost; storage savings must be evaluated with encoding time and memory. A delta addition is not automatically a speed fix. |
| Workspace transactions bypassing existing coalescing | Diagnostic: 164,150 inserted objects and 1,324 SQL admission transactions. Applying existing coalescing reduced selected Commit by 7.2%, and 6.7% in reversed order. | Preserve the shared bounded cohort and clean handoff to staging for the new encoding path. |
| Excessive spill read-ahead during dependency-ordered seeks | 1,431/1,816 sampled late-admission stacks in spill reads. Capping ordered read-ahead at 64 KiB improved the selected coalescing control 6.266→4.194 s. | Preserve access-pattern-specific buffering; measure new base/group access amplification. |
| Bounded reuse saturation and mixed validation | #95 mixed-500 still spent about 209 ms in storage comparison. The 2-MiB reuse reservation can clear on saturation. | Investigate actual distinct-group/base working sets; no automatic larger budget or unbounded cache. |

The profile counts contain overlapping parent/child stacks. Canonical comparison
bytes and read stacks are not physical disk-byte measurements. Blocked producers
showed consumer backpressure, not evidence that more workers would help. The
predecessor-bearing Workspace worker cap was outside the timed directory-Init
path. Larger locator INSERT statements had already been rejected as slower and
more memory-intensive; batching is not a single knob.

The combined #98 `.venv` pair reduced Commit 6.671→4.152 s (37.8%) and Exec+Commit
12.667→10.179 s (19.6%). Host CPU fell 8.250→5.715 s, peak RSS rose 5.1875 MiB,
and both paired Stores allocated 218,107,904 B. These scoped observations establish
useful repairs, not a universal speedup or a return to v0.1.3 performance.

Sources: [#95 profiles, pairs and limitations](https://github.com/Ephemeral-AI-Lab/layerfs/blob/856baab0caf0522db4757cb4dcbfb0d9c43e30d4/docs/roadmap/0.1/0.1.4/issue95/README.md),
[#98 profiles and pairs](https://github.com/Ephemeral-AI-Lab/layerfs/blob/856baab0caf0522db4757cb4dcbfb0d9c43e30d4/docs/roadmap/0.1/0.1.4/issue98/README.md).

## Storage benefit and remaining whole-history cost

Full157 allocation was **184,582,144 B versus 218,116,096 B**, a **15.374% reduction**
against the supplemental equal-retained-state control. Logical database size was
176,152,576 B, with 4-KiB pages, no sidecar allocation or unexplained page residual.
The retained representations were 58,306 PREFIX and 28,106 FULL.

Final full157 performance/verification wall observations were **481.976/613.815 s**,
versus previous **447.247/545.827 s**. The higher observations remain reported;
they are unpaired history and do not isolate a regression caused by #98. Selected
Init/Commit improvements cannot certify whole-history read or verification speed.

## Optimization follow-up after the main implementation

Start with the completed v0.1.5 code and a source-matched control for each proposed
repair. Reuse valid evidence, but establish source/contract applicability before
calling an older observation a current control. The table below is an investigation
order; the old family sums do not establish priority by themselves.

| Follow-up | Initial scope | Required evidence before retaining a change |
|---|---|---|
| Unique and mixed Init | Unique-500, mixed-500 and representative CDC locality cases | Separate publication/page-write, insertion, compression, locator and validation work. Show adjacent end-to-end benefit with CPU/RSS and storage accounting. |
| Delta/group read amplification | Small range reads, batched objects sharing FULL bases, whole files and retained histories | Count distinct groups/bases, requested versus reconstructed bytes and authentication work. Preserve bounded reuse and exact validation. |
| Workspace admission and spill traversal | Full-file rewrites, mixed new/rewritten files and `.venv`-scale traversal | Confirm the new route retains SQL coalescing and appropriate spill buffering; measure complete Commit and Exec+Commit together. |
| Other elapsed regressions | Namespace Init, directory traversal, edits, branch history and other remaining severe/review cases | Profile representative cases on the completed implementation. These families have recorded regressions, not fully established root causes. |
| Git comparison eligibility | Four historical Git cases | Establish valid matched fixture/source contracts for future comparisons. Keep original ineligible results immutable. |

Use short profiles and selected adjacent control/candidate timings during the
follow-up, with reversed order when needed to resolve noise. Track process CPU,
RSS, storage and public elapsed time together. Do not add workers just because
producers wait, enlarge limits, change page size, weaken authentication/collision
checks, or alter workloads/timers/thresholds to obtain a pass. Preserve canonical
identity, dependency ordering, retained history, rollback and final-root atomicity.

Keep implementation iteration efficient: do not repeatedly run complete campaigns
for each experiment. Reuse passing unaffected checks, preserve failed attempts,
and reject unhelpful changes without reverting another task's work. Serialize
resource-sensitive operations with the existing measurement lock. After freezing
the retained candidate, run the affected correctness/performance scope and required
terminal qualification under [benchmark_success.md](benchmark_success.md) and the
repository benchmark rules. Fix real failures and requalify their affected scope.

Close out this follow-up with measured retained improvements, CPU/RSS/storage
tradeoffs, exact source/binary/image identities, independent correctness results,
and an explicit list of remaining regressions or evidence-backed no-change
conclusions. Do not mark this backlog complete merely because v0.1.5's main delta
implementation or its tiny benchmark succeeds.

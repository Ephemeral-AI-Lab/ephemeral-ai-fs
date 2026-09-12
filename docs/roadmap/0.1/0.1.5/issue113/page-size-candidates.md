# Page-size candidates: which benchmark families would benefit from 4 KiB → 64 KiB

> Companion to [`page-size-ab-protocol.md`](page-size-ab-protocol.md) (issue #113).
> Screen only — no case here has been A/B-measured except `dedup-cdc-scattered-100`.

## 1. The model, and its calibration

Issue #113 measured on `dedup-cdc-scattered-100` that 26,200 newly written 4-KiB
pages cost ~117 ms of extra kernel time, and that 64 KiB removed 131.1 ms from a
302.6 ms timer. That gives a screen with one parameter:

```text
page_write_cost      ≈ 4.5 µs per newly written 4-KiB page
estimated_saving_ns  ≈ pages_newly_written × 4500
share_of_timer       = estimated_saving_ns / timer_ns
```

`pages_newly_written` is taken from each case's own `store-observation`
records (`before` → `after-commit` page-count delta), which is the volume of
pages the operation actually allocated. **Store size is deliberately not used**:
for edit/read/mutation cases the Store pre-exists and is mostly untouched, and
using its size inflates the estimate by 100×. For the fresh-init routes with no
`before` observation (`namespace`, `store-footprint`) every page of the result is
new, so `pages_newly_written = apparent_bytes / page_size`.

Calibration check, the one case measured end to end:

| | screen | measured (C arm, issue #113) |
| --- | ---: | ---: |
| predicted saving | 117.8 ms | **131.1 ms** (302,620,750 → 171,506,646 ns) |
| share of timer | 0.310 (of the campaign's 380.3 ms outlier sample) | **0.433** (of the C arm's 302.6 ms median) |

The screen captures ~90% of the effect; it under-predicts because it counts only
the write path and ignores the read-side page-lookup saving (issue #113 §11.6).

## 2. Candidates, ranked by estimated relative benefit

Cases with `share ≥ 0.10` and a timer ≥ 20 ms. Estimated saving is
`pages_newly_written × 4.5 µs`.

| family | case | timer ms | pages written | est. saving ms | share | payload uniqueness |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `store_footprint` | `store-footprint-large-object-500m` | 1,281.0 | 123,499 | 555.7 | **0.434** | — |
| `dedup_cdc_locality` | `dedup-cdc-scattered-500` | 1,455.7 | 129,882 | 584.5 | **0.402** | 0.999 |
| `dedup_cross_file` | `dedup-cross-file-unique-500` | 1,500.6 | 129,594 | 583.2 | **0.389** | 0.999 |
| `dedup_cross_file` | `dedup-cross-file-unique-100` | 303.9 | 25,910 | 116.6 | **0.384** | 0.996 |
| `dedup_cross_file` | `dedup-cross-file-mixed-500` | 1,158.8 | 97,196 | 437.4 | **0.377** | 0.750 |
| `init_namespace` | `namespace-10000` | 904.7 | 74,445 | 335.0 | **0.370** | — |
| `dedup_cross_file` | `dedup-cross-file-unique-10` | 33.0 | 2,592 | 11.7 | 0.353 | 0.995 |
| `dedup_cross_file` | `dedup-cross-file-mixed-100` | 265.1 | 19,424 | 87.4 | 0.330 | 0.752 |
| `dedup_cdc_locality` | `dedup-cdc-scattered-100` | 380.3 | 26,180 | 117.8 | 0.310 | 0.996 |
| `init_namespace` | `namespace-1000-compact-v3` | 75.2 | 4,989 | 22.5 | 0.299 | — |
| `dedup_cdc_locality` | `dedup-cdc-scattered-10` | 43.2 | 2,853 | 12.8 | 0.297 | 0.985 |
| `init_namespace` | `namespace-100-compact-v3` | 20.7 | 1,260 | 5.7 | 0.274 | — |
| `dedup_cdc_locality` | `dedup-cdc-common-body-500` | 628.8 | 36,285 | 163.3 | 0.260 | 0.377 |
| `dedup_cdc_locality` | `dedup-cdc-common-body-100` | 142.9 | 7,482 | 33.7 | 0.236 | 0.404 |
| `payload_create_read` | `payload-create-500m` | 2,640.0 | 129,460 | 582.6 | 0.221 | — |
| `store_footprint` | `store-footprint-large-object-10m-low-v1` | 55.0 | 2,492 | 11.2 | 0.204 | — |
| `payload_create_read` | `payload-create-100m` | 650.6 | 25,884 | 116.5 | 0.179 | — |
| `dedup_workspace_reuse` | `dedup-workspace-unique-100` | 665.6 | 25,910 | 116.6 | 0.175 | — |
| `init_namespace` | `namespace-100000` | 3,759.1 | 125,823 | 566.2 | 0.151 | — |
| `store_footprint` | `store-footprint-unique-100000` | 3,787.4 | 125,849 | 566.3 | 0.150 | — |
| `dedup_workspace_reuse` | `dedup-workspace-unique-500` | 4,097.6 | 129,590 | 583.2 | 0.142 | — |
| `payload_create_read` | `payload-create-10m-compact-v2` | 90.9 | 2,591 | 11.7 | 0.128 | — |
| `tiny_file_churn` | `tiny-bulk-create-100-mixed-v3` | 915.6 | 25,925 | 116.7 | 0.127 | — |
| `dedup_workspace_reuse` | `dedup-workspace-unique-10-base128-v3` | 97.1 | 2,591 | 11.7 | 0.120 | — |
| `tiny_file_churn` | `tiny-bulk-create-500-mixed-v3` | 5,102.9 | 129,643 | 583.4 | 0.114 | — |
| `dedup_workspace_reuse` | `dedup-workspace-unique-10-compact-v2` | 106.7 | 2,592 | 11.7 | 0.109 | — |
| `store_footprint` | `store-footprint-unique-100-low-v1` | 52.2 | 1,261 | 5.7 | 0.109 | — |

Just below the line, but watched, because their **absolute** saving is large even
at a low relative share:

| family | case | timer ms | pages written | est. saving ms | share |
| --- | --- | ---: | ---: | ---: | ---: |
| `workspace_change_locality` | `workspace-dense-rewrite-500-mixed-v4` | 9,764.7 | 129,452 | 582.5 | 0.060 |
| `dedup_branch_history` | `dedup-history-unrelated-500-mixed-v2` | 23,041.8 | 129,418 | 582.4 | 0.025 |
| `workspace_change_locality` | `workspace-dense-rewrite-100-mixed-v4` | 2,619.7 | 25,829 | 116.2 | 0.044 |
| `tiny_file_churn` | `tiny-bulk-delete-500-mixed-v3` | 1,057.9 | 129,928 | 584.7 | 0.553 → see note |
| `directory_construction_traversal` | `directory-construct-500-mixed-v4` | 1,073.6 | 59 | 0.3 | 0.000 |

Note on `tiny-bulk-delete-*`: its `after-commit` page count is a *deletion*
workload, so the delta is not a write volume in the same sense; it is listed for
completeness and would need its own check.

## 3. Structural pattern — the useful simplification

Because the screen is linear in bytes written, the ranking collapses to one rule:

> **Estimated saving ≈ 1.1 ns per newly written byte**, i.e. ≈ **583 ms per
> 530 MB** of new pages and ≈ **117 ms per 106 MB**.

So the candidate set is simply *the cases that write a lot of new pages*:
`dedup-cdc-scattered-*`, `dedup-cross-file-unique-*`, `dedup-cross-file-mixed-*`,
`init_namespace` (all tiers), `store-footprint-unique-*` and
`store-footprint-large-object-*`, `payload-create-*`, `dedup-workspace-unique-*`,
`tiny-bulk-create-*`, `workspace-dense-rewrite-*`,
`dedup-history-unrelated-500-mixed-v2`.

The *relative* benefit then depends on how much of the timer that constitutes —
which is why `namespace-10000` (0.370) outranks `namespace-100000` (0.151):
both write proportionally similar page volumes, but the 100 000 case spends a
larger share of its budget elsewhere.

## 4. Predicted non-candidates

| family / class | why | observed |
| --- | --- | --- |
| `edit_length_preserving`, `edit_length_changing`, `edit_canonical_chunk_count` (SDK) | a single 4 KiB range edit dirties a handful of pages; the timer is `edit_commit_ns` ≈ 8–16 ms | page volume not observable in the receipt; predicted **≈0, and possibly negative** — with 64 KiB a dirty page write is 16× larger |
| `directory_construction_traversal` | traversal/mutation on a pre-existing 530 MB Store | page delta **+0 … +59** |
| `namespace_mutation` | subtree metadata move/delete | page delta **+0 … +2** |
| `tiny_file_churn` `tiny-{create,unlink,stat}-*-mixed-v4` | single-file operations on a prepared Store | page delta **+1 … +217** |
| `payload_create_read` `payload-random-read-*`, `directory-content-scan-*` | reads | page delta **0** |
| `git_tool_workflow` | clone/commit workflow | estimated share **0.011–0.030** |
| `dedup_branch_history` (all but `unrelated-500`) | delta/reuse-heavy, exactly the stride-3 shape measured to gain 1.4% | share **≤ 0.026** |
| `dedup-cdc-{overwrite,insert,delete}-*` | low payload uniqueness (0.13–0.53) → reuse wins, little new page volume | share **0.054–0.156** |
| `dedup-cross-file-identical-*` | nearly all content reused | share **0.002–0.064** |

This is corroborated by the direct stride-3 measurement: a delta/reuse-heavy
publication workload gained **1.4%** on its commit phase, against the screen's
~4% estimate.

## 5. The cost side is not in these receipts

The screen is purely the *speed* benefit. The *storage* cost was measured at two
points, and it depends on the Store's record-size distribution, not on bytes
written:

| measured case | record size | 64 KiB allocated penalty |
| --- | --- | --- |
| `dedup-cdc-scattered-100` (tier 100) | ~18 KB objects | **+5.0%** |
| `dedup-cdc-scattered-10` | ~18 KB objects | +13.6% |
| `dedup-cdc-scattered-500` | ~18 KB objects | +5.2% |
| DeepSeek stride-3 (53 states) | many small objects | **+27.3%** (host boundary) / +30.0% (Store file) |

The small-file candidates above — `init_namespace`, `tiny_file_churn`,
`store-footprint-unique-100000`, `directory_*` — are expected to sit near the
stride-3 end (+25–30%), not the scattered end (+5%). **That cost is not derivable
from the campaign receipts and must be measured per candidate family before any
decision**, because it is the term that decides the trade.

## 6. Recommended A/B set

Cheapest design that covers the whole mechanism space, using the #113 harness:

| slot | case | why this one |
| --- | --- | --- |
| 1 | `dedup-cdc-scattered-100` | already measured: the 18 KB-object, near-unique, high-share anchor |
| 2 | `init_namespace/namespace-10000` | highest-share small-file init (0.370) and the #106/#109 vehicle |
| 3 | `store_footprint/store-footprint-large-object-500m` | highest estimated share overall (0.434); large-object, large-store |
| 4 | `payload_create_read/payload-create-500m` | independent family, 0.221, 530 MB written |
| 5 | one SDK edit case, e.g. `edit_length_preserving/overwrite-middle-4k-on-500mib-ops-1` | predicted **zero or negative** — the falsification arm |
| 6 | `dedup_branch_history/dedup-history-distributed-500` | predicted **≈0** — the reuse-heavy falsification arm |

Slots 5 and 6 are the important ones: if 64 KiB is neutral or harmful there, the
result is a clean "creation policy by workload class" argument rather than a
global switch.

Storage for each arm is read directly from `host-runtime/store.sqlite`
(`PRAGMA page_size/page_count/freelist_count` plus `st_blocks × 512`), exactly as
in issue #113 §11.2.

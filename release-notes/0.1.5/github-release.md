# LayerFS v0.1.5 — ordinary storage, capacity, and honest limits

> **Status:** LayerFS 0.1.5 release announcement draft. Published as the GitHub
> release for tag [`v0.1.5`](https://github.com/Ephemeral-AI-Lab/layerfs/releases/tag/v0.1.5).

v0.1.5 ships **ordinary schema-10 Init/Commit storage** as the only storage path
and removes explicit compaction. New Stores use schema 10 with 4 KiB pages: exact
content-addressed admission, compression, bounded whole-file small content below
128 KiB with bounded delta chains, large-file CDC/extents, compact scoped
namespaces and authenticated pooled metadata. Canonical identity is unchanged.

**What got better, measured:**

- **Pending-edit capacity (#116):** an equal-length overwrite of committed
  content is retained as one base root plus one bounded splice descriptor — one
  64-byte descriptor instead of three 128-byte nodes. Pending workspace capacity
  moves from 5,461 to **≈29,959 spliced files** at the same budget, and the
  public default-budget route now accepts **32,000 edits** (32,000 × 64 B under
  the unchanged 2 MiB budget) with independent verification of all 32,000
  changed files before and after a fresh reopen.
- **Distributed SDK-edit workloads:** `workspace-distributed-sdk-edit-500`
  4.131 s → **0.666 s (0.16×)** and `-100` 0.45× against the earlier v0.1.5
  baseline — the same #116 mechanism.
- **Packed metadata (#107):** pack rows 3,457 → **1,058**, pack overflow slack
  **−43 %**, apparent Store **−1.13 %**, and allocated Store **−1.9 %** on
  `store-footprint-unique-100000` (530,358,272 → 520,142,848 B). The
  self-declared ≥1.5 % *allocated* figure was **missed numerically** (+0.02 %
  allocated) and is accepted by explicit owner decision, not re-based.
- **Stable wins:** `payload-create-*` 0.79–1.02× and `dedup-workspace
  exact/local-100` 0.62–0.66× versus published v0.1.3.

**What did not get better — published, not relabeled:**

- One registered Tier-1 gate **fails**: `dedup-history-unrelated-500-mixed-v2`
  measured **16.107 s** against `unrelated-history500 < 15 s`, dispositioned by
  an explicit owner waiver. Root cause: the owner-required #116 bounded pending
  representation (+1.98 s) plus the owner-accepted #107 pack coalescing
  (+0.33 s); the breach predates this campaign, and the cell is 0.89× v0.1.3.
  Residual risk: the worst-case unrelated-history tier runs ~7 % over its
  historical gate.
- **125 WARN cells**; worst ratios `dedup-cdc-scattered-100` 3.45×,
  `dedup-cross-file-unique-100` 3.35×, `namespace-100` 3.22×.
- **139 of 198** comparable cells are ≥15 % slower than published v0.1.3,
  median **1.34×**, **+33.115 s** total. This is context for future target
  selection, never an acceptance gate.
- The #116/#107 mechanisms add fixed per-iteration costs of **+2.49 ms/exec and
  +1.46 ms/commit** (#116) and **≈+0.4 ms/phase** (#107).
- `store-footprint-unique-100000` construction takes **5.397 s** (+36 % versus
  fsync-qualified): the #107 pack-row UPDATE cost on one giant commit.
- Ordinary full157 allocated **83,951,616 B** versus a Git control live
  **56,197,120 B** (accepted residual).
- `dedup-cdc-scattered-100` leaves an **≈452 ms** unattributed gap between timer
  and command window, and the `slab_send_blocked_ns` mechanism remains
  unattributed.
- **Endurance is not qualified:** the 600 s sustained proof was
  `NOT_RUN_OPTIONAL` under the frozen campaign declaration.

**Validation:** 227 registered selections terminal — 198/198 performance (182
fresh: 56 PASS / 125 WARN / 1 FAIL; 16 reused) and 29/29 proof-only (28 PASS +
1 not run) — with 182/182 fresh independent proofs PASS, every cleanup PASS, no
S0/S1, a VERIFIED_COLD `namespace-100000` member, and the native gate PASS (530
tests, 118 s) on the measured tree. The campaign is inherited from issue #120;
**no benchmark was re-run for this release.**

**Upgrade boundary:** create a **new schema-10 Store**. Supported
schema-6/7/8/9 Stores connect without promotion; published v0.1.3/schema-5 Stores
are rejected. `LayerStackStore::upgrade_format` promotes a closed schema-7/8/9
Store to schema 9 only — there is no in-place promotion to schema 10 and no
downgrade. Compacted Stores from earlier builds remain readable (read-only
compatibility path). Use matching SDK/owner/daemon builds.

This is a **source-only Developer Preview**, not production storage. Crash and
power-loss durability are not promised. No crates.io package, prebuilt
executable or public runtime image is part of this release.

Start with the [release record](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.5/release-notes/0.1.5/README.md),
the [versioned manual](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.5/docs/versioned/0.1.5/README.md),
the [acceptance and waivers](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.5/release-notes/0.1.5/acceptance.md) and
[every benchmark family and case](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.5/release-notes/0.1.5/benchmark-closeout.md).

## Existing benchmark results by family

Times below are descriptive sums of the individual case timers, not campaign
wall time and not statistical speedup estimates. Positive changes mean slower
than the published v0.1.3 checkpoint, whose cache profile is undeclared. Rows
mixing fresh and reused values are marked in the release record; this table
reports all registered performance cases in the family.

| Family | Performance cases | Reused | v0.1.3 seconds | v0.1.5 seconds | Change |
|---|---:|---:|---:|---:|---:|
| payload_create_read | 8 | 1 | 4.179260 | 3.454420 | -17.34% |
| dedup_workspace_reuse | 14 | 1 | 15.875100 | 15.301500 | -3.61% |
| dedup_cross_file | 10 | 2 | 1.402390 | 3.473250 | +147.67% |
| dedup_cdc_locality | 20 | 1 | 1.484610 | 3.510380 | +136.45% |
| edit_length_preserving | 12 | 1 | 0.082800 | 0.114680 | +38.50% |
| edit_length_changing | 32 | 2 | 0.230460 | 0.303170 | +31.55% |
| edit_canonical_chunk_count | 12 | 0 | 0.092550 | 0.129570 | +40.00% |
| init_namespace | 4 | 0 | 3.053450 | 5.559430 | +82.07% |
| store_footprint | 6 | 0 | 8.155620 | 13.464210 | +65.09% |
| tiny_file_churn | 20 | 1 | 8.529740 | 9.714270 | +13.89% |
| namespace_mutation | 4 | 0 | 0.314360 | 0.452180 | +43.84% |
| directory_construction_traversal | 12 | 1 | 7.629500 | 8.826820 | +15.69% |
| workspace_change_locality | 16 | 4 | 10.971850 | 15.227050 | +38.78% |
| dedup_branch_history | 20 | 0 | 47.049480 | 45.614220 | -3.05% |
| git_tool_workflow | 4 | 1 | 7.498040 | 12.683490 | +69.16% |
| mixed_load_bearing | 4 | 1 | 8.559560 | 9.345510 | +9.18% |
| **registered total** | **198** | **16** | **125.108770** | **147.174150** | **+17.64%** |

The five families that are faster than v0.1.3 in aggregate are exactly the ones
the #116/#107 mechanisms and the pre-existing ordinary-path wins touch; the
dedup families' large aggregate ratios compare the v0.1.5 ordinary path
(authenticated CAS + CDC + pack assembly) against pre-authentication v0.1.3
references and are published as #112 context, never as gate results. Read the
[complete per-case tables](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.5/release-notes/0.1.5/benchmark-closeout.md).
Existing qualified results were reused; no benchmark was run for this release.

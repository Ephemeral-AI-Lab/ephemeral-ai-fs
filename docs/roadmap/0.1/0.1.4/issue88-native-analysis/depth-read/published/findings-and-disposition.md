# Issue #88 finalization — completed research generation

**RETAIN AS AN ISOLATED RESEARCH CHECKPOINT.** The authorized fixed depth-stratified read campaign completed all **60 observations** with matched bytes/digests, intended dependency coverage, resource checks, successful owned cleanup and unchanged original evidence. This supports retaining the184,598,528-byte implementation for research with explicitly limited selected-case read evidence. It does not establish release readiness, universal read equivalence or a worst-case latency bound.

The previously completed full157 construction/retention experiment remains unchanged:184,598,528 B complete candidate allocation and155,353,550 B pack BLOBs;44.99% less allocation than original M4.5 and15.37% less than the fresh C+S1 control. Neither construction nor the full157 census/verification was repeated. The new diagnostic changes no product, codec, representation, canonical ID, budget or stored payload. No subsequent optimization is proposed or executed in this finalization.

## Actual workload and proof

The prospective deterministic rule selected the earliest eligible source checkpoint, raw path bytes and file offset for native depths0–4. The existing authenticated extraction cache supplied candidates; public LayerStackStore ObjectSource plus exact filesystem::stat and rope::visit_extents proved retained paths and complete descriptor containment on the two disposable copies. Selected locators and chains were tied to the existing authenticated inventory and158-row admission trajectory. Static proof supplies target identity; runtime counters corroborate that the public path exercised native dependencies.

| Selected depth | Checkpoint | File bytes | Range offset / length | Raw dependency closure bytes | Actual file |
|---:|---:|---:|---|---:|---|
|0|1|17,819|0 /4,096|17,819|AGENTS.md|
|1|2|9,128|0 /4,096|12,609|.agents/skills/dsh-code-review/SKILL.md|
|2|3|11,235|0 /4,096|23,844|same path, next source version|
|3|4|12,408|0 /4,096|36,252|same path, next source version|
|4|5|4,320|0 /4,096|17,392|.github/workflows/ci.yml|

All five selected files have exactly one authenticated payload extent. The contract allowed mixed-depth full files, but this actual cohort does not contain them. The five selections cover three distinct paths and checkpoints1–5. All ranges begin at zero, wholly inside the selected descriptor. These observations do not qualify8MiB files, multi-chunk or mixed-depth streaming, maximum163,840-byte retained dependency closures, the1MiB format ceiling, random offsets or general workloads. Differences across strata also differ in content/locality/history; depth alone cannot be inferred as their cause.

Selection succeeded in five path attempts, read54,910 source bytes for digest work, and completed within the frozen64-attempt/64MiB/1800-second limits. The supervising preparation invocation lasted11,321,342,125 ns, with sampled process-tree RSS maximum345,423,872 B and child-lifetime high-water308,035,584 B. Those are separate observed preparation scopes, not exact phase peaks. Minimum sampled free space443,132,878,848 B exceeded the50GiB reserve. Two independent logical copies were created once, proved byte-identical to their pre-verification source snapshots after proof, then reused for the campaign. Only the copies received public fork/session metadata; copy allocation is not a new storage result.

## Read observations

Five strata ×range/full ×three repetitions ×two arms =60 timed reads,30 per arm. Arm order within each cell was C/P, P/C, C/P. Every row used a fresh host process, container, daemon and FUSE session. Read/count public Exec was timed through terminal output drain; digest verification was separate. There was no warm-up, replacement row or discarded outlier. Values below are integer nanoseconds; each arm/cell has n=3. Signed median differences compare arm medians, not the median of paired differences; the latter operands are also preserved in the machine report.

| Depth | Operation | Control min / median / max (ns) | Candidate min / median / max (ns) | Candidate − control median (ns) |
|---:|---|---:|---:|---:|
| 0 | range | 7,985,292 / 8,110,750 / 10,164,750 | 8,531,833 / 8,597,792 / 10,772,625 | +487,042 |
| 0 | full | 9,771,250 / 10,083,083 / 10,942,250 | 8,884,000 / 9,073,042 / 9,170,583 | -1,010,041 |
| 1 | range | 10,518,292 / 12,635,709 / 13,744,250 | 10,292,333 / 11,473,375 / 11,754,583 | -1,162,334 |
| 1 | full | 10,737,292 / 12,990,458 / 15,240,458 | 9,995,166 / 10,460,542 / 10,610,166 | -2,529,916 |
| 2 | range | 10,055,708 / 10,718,792 / 14,017,166 | 10,867,166 / 11,432,166 / 11,570,042 | +713,374 |
| 2 | full | 10,790,625 / 11,502,000 / 11,821,625 | 11,061,459 / 11,739,250 / 12,029,459 | +237,250 |
| 3 | range | 11,917,000 / 13,619,667 / 14,398,791 | 11,262,459 / 13,080,625 / 13,127,042 | -539,042 |
| 3 | full | 12,127,625 / 12,377,792 / 12,984,125 | 11,722,833 / 13,643,084 / 25,695,625 | +1,265,292 |
| 4 | range | 12,009,542 / 12,493,292 / 13,636,875 | 10,337,708 / 10,920,958 / 14,809,375 | -1,572,334 |
| 4 | full | 10,585,875 / 11,464,625 / 12,558,417 | 9,665,875 / 11,242,625 / 12,039,542 | -222,000 |

Candidate range medians are8,597,792–13,080,625 ns and full-file medians9,073,042–13,643,084 ns. Cell median differences are mixed, not a uniform speedup: the largest positive median difference is1,265,292 ns at depth3/full. That is useful limited evidence for research retention given the material storage gain and short absolute calls; roughly30% owner guidance is context, not an automatic pass threshold.

**Preserved unfavorable observation:** depth3/full candidate repetition3 took25,695,625 ns versus its paired control12,377,792 ns: +13,317,833 ns (approximately107.59%). It remains in both raw data and summaries. No cause is proven. The maximum is the largest observed value, not a worst-case guarantee. Three repetitions of one file/range per stratum cannot establish population tails, p95/p99, confidence or performance equivalence.

## Mechanism, resources and timing scopes

All candidate rows reported zero native fetch/decode activity in setup before the requested read. Every timed candidate read met its selected-depth completion and minimum fetch/decode/edge criterion; control reads contained no native records. Every count and separately computed digest matched its frozen oracle. All60 normal ends confirmed unchanged fork heads, no active owners and successful container removal. Fixed source-reviewed commands and no Commit call establish nonmutation; unavailable FUSE write counters are not relabeled measured zero.

Observed candidate timed native decode calls by selected depth were18,29,39,51,58, with aggregate native raw decoded bytes41,501 /50,101 /75,488 /108,785 /115,653 B respectively; range/full have the same reported work within each selected stratum. These exceed the selected target's1–5-record chain. They are aggregate work of the public operation, not per-ObjectId attribution. The identities and causes of every extra decode were not logged; do not claim all this work belongs only to the target chain or specifically to read-ahead. Generic group/encoded/decoded counters and native counters retain their distinct source definitions and must not be added as disjoint physical disk I/O.

Across the30 read phases per arm, recorded host CPU totals are176,440,544 ns control and164,532,123 ns candidate; these totals describe only this fixed artificial schedule, not a workload-weighted speedup. Maximum sampled current/lifetime host RSS during read phases is41,926,656 B control and41,779,200 B candidate. Observed host disk-read bytes are0 in both arms, while read-phase host disk writes total122,880 B in each. Cached host I/O is not proof of no decoded/fetched data and does not support a cold-read claim. Maximum observed container lifetime total-memory peak across read/digest boundaries was15,073,280 B; minimum recorded campaign free space442,568,417,280 B. All frozen resource gates passed.

Public read and digest timings, host phase CPU/I/O/current/lifetime RSS, broader cgroup observation windows, setup/End/cleanup, copy/runtime/disk observations and enclosing wall are separate in results/observations.csv and report.json. FUSE metrics are available at session End and retain whole-session scope; per-action read/write values remain null. OS caches are uncontrolled after copying, hashing and prior operations. Campaign invocation206,243,691,708 ns includes60 fresh lifecycles, observers, digests and custody work; it is not the sum of user-visible read latencies.

The previous23.19% full-history verification read/traversal/digest elapsed increase,26.17% host CPU increase, higher construction write traffic, earlier text-full regression and74.42ms outlier remain valid unfavorable context. This small-file campaign does not erase or explain them. It also does not authorize a new cache, anchor substitution or altered codec to improve them.

## Custody and implementation scope

Normal control producer2753453933c55ed7f93eb21619c235668a01ef4c and candidate d4f26f0d16f0f91c1f75767cf699012b8794ac01 are unchanged. Existing product/source/workload, binaries, images, report manifests, snapshots, inventories and the prior extraction cache were reauthenticated. Candidate image's analysis-only dirty-patch qualification remains explicit. Old probe binaries were not relabeled: identical cohort-aware analysis source was built separately against each arm with Rust1.85.1 and pinned dependency identities. Actual new hashes, commands, source/tool maps and dirty analysis-build provenance are in identity-and-scope.json and the artifact registry. Later report commits are not measured producers.

The old probe, original Stores, snapshots and all sealed historical evidence remain intact. No product source or benchmark definition was changed. The new tools reuse the public SDK/runtime and existing canonical reader; no independent storage implementation, backend, generic benchmark family, persistent cache or payload dump was added. Analysis input checks passed, including malformed paths, span/file bounds and exact60-row schedule/dependency gates; one Rust byte-path test passed. No full Store/codec suite was rerun to regenerate evidence.

Preserved preparation history includes lock-rejected launches before any selection/sample, an offline lockfile prune retaining registry versions/checksums, and the proof helper's SQLite u64 conversion compile error corrected to checked i64 conversion. The corrected proof built successfully before selection. These are analysis/build corrections, not product failures or replaced timing samples. The actual cohort and read campaign each executed once and passed.

## Test debt, adoption limits and terminal disposition

The accompanying test-debt-ledger.md records all nine exact unresolved Store failures with P/D/accepted evidence and subsystem obligations. Full-suite status remains62PASS/9FAIL/1ignored. The P/D/accepted cleanup residues19,571 /19,573 /19,572 are not normalized; P/accepted unexpected-success clarification is preserved and the older D actual result remains unknown. Recording or comparing these failures is not a repair or waiver. Fault/schema/order/resource-boundary obligations must be resolved or explicitly dispositioned before adoption.

Directory Init still lacks the native FILE provenance transport on its direct sink; the full157 Empty-Init/import path is validated, not universal initialization coverage. New readers support the required legacy/native formats, but old readers cannot decode native version2 records. Old-writer/downgrade safety for the unchanged SQL schema is unqualified and needs explicit adoption design. Required legacy readers and S1 structural matching remain live; no deletion, migration or compatibility rollout was performed.

The valid completed diagnostic, explicit costs and independent review support **RETAIN AS AN ISOLATED RESEARCH CHECKPOINT**, with acceptable observed median costs for these selected small single-chunk cases and unresolved tail/general-workload suitability. **Close #88 as a completed research generation**, linking the sealed result and remaining adoption debt. No additional optimization or experiment is launched; no PR is merged, and no related release/allocation issue is closed. The184.60MB storage result is preserved;134.2MB remains an unproven future stretch outside this task.

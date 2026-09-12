# #118 shared optimization scope

Current issue bodies/comments read on 2026-09-12: #108, #112, #114,
#100, #107, #102, #110, #106 and umbrella #118. This is the deduplicated
scope map, not qualification evidence. Historical raw roots were retired by
the owner; old reports supply hypotheses and dispositions only. Runtime
execution and the current checklist belong to `execution.md`.

Current handoff update: reopened-history, physical reservation, edit and cold
qualification below have since completed as described in execution.md. Cold2.7s
is owner-WAIVED (see the final section). The fsync fix now passes a full500
13.791s screen; full157/access and18 other shared selections remain pending.
Use handoff.md for continuation; old diagnosis text below is retained provenance.

| Track | Current disposition and one owner/proof |
| --- | --- |
| #108 shared publication/read work; #112 G1/G6; #106 history deadline | One Commit/edit/history track. Stage 2 bounded tree reads and Stage 3 spill changes already exist; do not implement the old directory-rebuild hypothesis. Measure current tier-500 distributed/recurring/unrelated vehicles, then use the same results as G6 transfer. Preserve the unresolved unrelated-500 15 s target. |
| #108 SQL preparation; #109 signature preparation; #111 endpoint/index work | Prepared SQL, producer-prepared small signatures, bounded exact metadata endpoint query and fingerprint-based exact metadata lookup already exist. They are completed code changes, not fresh optimization opportunities. Their relevant current checks can be grouped with shared admission qualification. |
| #111 reopened metadata/history reconstruction | Still present: the first changed Commit builds an index from all historical metadata values after every Store reopen. Separate from warm/cold source acquisition. See the exact diagnosis below. |
| #112 G2 duplicate validation | Keep exact CAS and authenticated comparisons. Scattered-100 is the negative control with almost no conflicts; its repetition/attribution question already has a no-go disposition. Do not relaunch the historical ratio ladder. Any actual duplicate-read change needs an identical/overwrite case and workspace-reuse spillover. |
| #112 G3 unique-content admission; #114 consumer | One ordinary admission/publication track. Scattered SQL/page-cost attribution is complete. The 4 KiB policy remains binding; no global/class 64 KiB change or added writer threads follows from the old study. Cohort compression/raw-skip are hypotheses, not mandatory implementations independent of current evidence. |
| #112 G4/G5 reads, coalescing and dense rewrites | Inspect current shared paths, use focused scan/dense-rewrite cases and one other-family guard. Stage 2's bounded authenticated reader may satisfy part of G4, but its original raw evidence is retired and it needs current transfer qualification. No duplicate fixed-cost/readahead implementation. |
| #114 lifecycle observation overhead | Attribution and timer/lifecycle derived view are complete. Remaining Docker exec/image/readiness overhead is the #117 infrastructure task, using the existing fresh-container/resource/identity contracts. Container pooling was not authorized by the attribution. |
| #110 metadata-cardinality failure | Required current product check: `store_footprint/store-footprint-metadata-cardinality-100000`. Original failure was `physical encoding reservation`; `PreparedObject` still carries a full inline small signature in every role. Diagnose actual current reservation/capacity before changing it; never increase the 2 MiB limit to hide it. |
| #100/#107 ordinary storage | Attribute complete ordinary Store bytes, then prospectively set any new storage-improvement target with latency/resource constraints. Old compacted <66 MB goal is explicitly superseded, and Git numbers are historical context until a matching treatment exists. Savings must arise from ordinary Init/Commit, never compaction/VACUUM/repack. Preserve legacy compacted reads. |
| #102/#110 qualification | One final deduplicated qualification over affected paths, exact final identities and independent proofs. The registry is authoritative. Optional stride-1/3/10 histories stay explicit selections (157/53/17 states), not automatic quick/default work. Changed ordinary-storage representation requires affected history/storage proofs; no general replay of retired matrices. |
| #111 cold Init | Still a hard gate: original `namespace-100000`, 100000 files/500000000 bytes, complete metadata and verified-cold v2 acquisition, Init <=2.7 s. Latest historical 3.420 s is not current evidence. Readback/index changes plausibly affect it; measure after a relevant change stabilizes. |
| #116 capability restrictions | Deferred until optimization tracks. Audit before removal; repair restrictive representations with bounded streaming/spill. Then complete the default-budget public spill workload that the current SDK capacity blocked. |

## Hard gates and permitted dispositions

- Correctness/authentication/collision handling/reference closure/publication,
  recovery, actual RAM/file/disk bounds, cleanup and evidence validity are hard.
- Cold Init <=2.7 s is hard. Historical-access retains separate complete 15 s
  performance and verification envelopes, including preparation/cleanup.
- `dedup-history-unrelated-500-mixed-v2` retains its frozen 15 s target; the old
  explicit-failure reporting option does not make it a PASS.
- Issue47 `tiny-bulk-create-100-mixed-v3` <1 s remains an explicit stronger gate.
- Stage 2 K10 absolute 50/31 ms targets are owner-WAIVED. Near-200 ms K100 is an
  engineering goal; do not treat a hardening patch as needing a second 25% win.
- For ordinary regression screens freeze #118's n3 alternating-pair rule:
  median paired wall slowdown >max(15% control median,3 ms), and >=2/3 pairs
  slower; CPU analogue max(15%,1 ms). Applicable stronger requirements prevail.
  Isolated minor noise can be WARN; unexplained material regressions cannot.
- Preserve all attempts. Do not rerun an unchanged arm/cell for a better median.

## Reopened-history diagnosis and minimum exact design

Current call path: `LayerStackStore::connect` creates a `StoreDb` whose
`metadata_index` is None. First changed admission calls
`objects/admission/metadata_values.rs::prepare_values`, creates
`metadata::ValueIndex::new` with `next=1`, and calls `ValueIndex::sync`.
Synchronization authenticates every historical value group and inserts every
fingerprint/ordinal, clearing the scratch index whenever the next whole group
would exceed 131072 entries. Replaying H values costs O(H log 131072) plus group
reads; reopening after each growing Commit repeats the prefix and gives a
quadratic aggregate. Retaining the Store fixes only that lifetime.

Current schema10 has an ordinal-to-group catalogue and canonical-object locator
index, but **no persistent value-to-ordinal index**. The endpoint query is already
bounded to at most 165 catalogue rows, so changing that query again does not fix
replay. The old guarded-predecessor experiment was not promoted and retains an
O(H) unknown-value fallback. Reconstructing the latest 131072 ordinals does not
preserve the current greedy whole-group eviction boundary or first winner.

Saving a scratch sidecar with endpoint, last-group digest, SQLite data_version
or file timestamps is insufficient: the existing
`issue111/restart-index-design-results.md` documents two distinct valid Stores
with matching endpoint/last-group/data_version but different exact mappings.
Positive authentication detects false hits; it does not prove negative-cache
completeness. Do not revive this rejected sidecar as an exact index.

Minimal schema10 candidate: stream ordered `(first_ordinal,count)` catalogue
headers once, preserve range/gap/end checks and reproduce the exact greedy
whole-group eviction boundaries; only then authenticate/decode/insert the final
surviving <=131072 values. This eliminates payload work and SQL insertion for
values the legacy index inevitably discarded, without narrowing the dedup window
or changing its first winner. Required referenced values and DELTA bases still
authenticate normally, and `reachable_storage` retains complete catalogue
authentication. Decoding an unrelated group solely to discard all its index rows
is incidental index-construction work, not a value consumed by the new lookup.
Corruption handling at this precise boundary must be documented and tested.

This has a strict limitation: the header audit remains O(G) per reopen, and there
is no gain at H<=131072, including the 100002-value namespace workload. It is a
bounded-payload reconstruction improvement, **not** universally sublinear total
reopen work. A predecessor fast path also cannot close that universal requirement
because unknown values retain the fallback. A durable bounded table plus cursor
could remove repeated reconstruction generally, but requires a real format
extension: schema10 checks its complete schema and old writers must be fenced.
No schema extension or index-persistence change is authorized by this scope note.

Proposed minimal ownership: `objects/metadata.rs` and existing metadata tests.
Proof: real authenticated groups across repeated 131072 boundaries, mixed group
lengths and reopen+append; compare old/unknown/reverted/duplicate values and
winning ordinals to the legacy eviction recurrence, assert <=131072 values are
decoded/indexed per fresh index, check collisions, surviving-group corruption,
header gaps/overlaps/count errors, rollback and unchanged incremental sync.
Public proof must measure reconnect, edit, Commit and their per-sample complete
sum separately, with independent reopened content checks.

There is a separate reconnect cost: `preflight_connect` and the exclusive open
both call `verify_schema`, including full `pragma_foreign_key_check`. Direct
connect does **not** decode the metadata catalogue; that happens in
`reachable_storage` and index sync. Existing `tests/v4.rs` requires connect to
reject dangling foreign keys. Do not remove that check or claim universally
sublinear reconnect from eliminating metadata replay alone. A redundant
preflight scan can be assessed separately while preserving the exclusive-open
check, but still leaves an explicit linear integrity scan.

Component implementation is now committed separately: `16c027804` implements
exact tail recovery; `cd7cfba22` removes the duplicate preflight foreign-key scan
while preserving complete validation under exclusive ownership. Focused checks
passed 22 tests (6 fingerprint, 9 schema/compatibility, 7 metadata admission).
At 131229 / 262459 / 524864 historical values, actual decoded group counts fell
from 801 / 1603 / 3200 to 6 / 7 / 9, and the resulting index rows and exact first
winners matched the original full replay. These are deterministic component work
counts, not public latency measurements. Test command walls were 13.129 s
(including 3.28 s compilation), 5.285 s (including the deliberate 5 s lock timeout)
and 0.554 s. Logs, exact commands/exits, failures and limitations are retained in
`benchmark-results/host-store/issue118/20260912/reopened-history-checks/`.

Two initial shared compilations exposed unsupported `u64`/`usize` SQLite reads;
the first local check then exposed unsupported `u64` test parameters. All were
fixed before successful execution; no failed test or sample was hidden. The
pre-existing unused-mut warning in metadata admission tests remains explicit.
The two new production-retention proofs ran successfully and were then marked
ignored by default, without logic changes, to keep scaling boundaries explicit:
select `metadata_fingerprint_reopen_tail -- --ignored --nocapture`. Public
reopened-chain qualification and residual-cost assessment remain pending.

## Current public command surfaces

Use current CLI help after #117 edits; these are existing entrypoint shapes,
not old identity values or prepared-output claims:

```sh
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
python3 benchmark/fs-bench-pro/shared/runner.py --family FAMILY --list
python3 benchmark/fs-bench-pro/shared/runner.py --family FAMILY --case CASE \
  --seed 1 --image IMAGE --perf-fast --collection-mode --output NEW_OUTPUT
bash benchmark/fs-bench-pro/families/FAMILY/verify.sh --case CASE \
  --seed 1 --image IMAGE --output NEW_PROOF
```

SDK edit families use `--repetition 1`, not `--seed`. Init selects fresh setup;
namespace-100000 automatically enforces verified-cold acquisition. Verification
binds source/input and, for SDK cases, `--performance-rows` as required by the
selected family's current help. Historical-access uses its specialized
`--mode verification --performance RESULT` path and an explicit compatible
retained Store; repository history uses explicit `--profile stride-N` and the
same measured run for `--storage-verify-run`. An absent retired history is a
missing prerequisite to resolve, never a retained exact-candidate proof.

## Updated owner cold-target disposition

The owner explicitly superseded the2.7s absolute gate during this run:
"we need to get better but does not mean2.7 is a must because in v0.1.5 we
introduced authentication, pack, delta encoding which might increases time".
The absolute target is now **owner-WAIVED**; prior hard-gate descriptions above
record the earlier contract. Require measured current-code improvement and
no unexplained material regression, preserving full verified-cold acquisition,
authentication, packing, DELTA encoding and resource protections. Existing
runner TARGET_MISS remains visible and receives this explicit owner disposition.
See `owner-cold-target-waiver.json` in the fresh evidence root.

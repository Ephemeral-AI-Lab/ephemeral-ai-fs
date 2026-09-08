# M4.5: 4-KiB SQLite creation and allocation evaluation

Status: **completed; owner accepted 4-KiB creation default**, 2026-09-08.
Evidence: [M4.5 JSON](implementation-milestone-4.5.json) and [progress](implementation-progress.md).
The default is accepted; original diagnostic misses and full release qualification
are unchanged. Unresolved allocation amplification remains tracked in
[issue #83 (M3-R6)](https://github.com/Ephemeral-AI-Lab/layerfs/issues/83).
Tracking: [issue #86](https://github.com/Ephemeral-AI-Lab/layerfs/issues/86).
Complete this isolated iteration after M4 and stop before M5. This supersedes the
prior blanket SQLite-tuning deferral only for the scope below. S3 remains future
issue #82; no hybrid implementation or existing-Store migration is authorized.

## Custody and required reading

Worktree: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3`.
Branch: `codex/storage-v3-implementation`, draft PR #81.
Control checkpoint: `d6c8d565170c1015b6e9c6cae3917d4e344bce2b`;
M4 corrected product: `de3a046f6a488ddd9d5f6710db8d98609b9983d6`.
Inspect actual HEAD, active writers and working-tree edits; never reset to anchors.
Preserve all prior Stores, source identities, reports and failed observations.

Read applicable AGENTS.md/skills, this plan, implementation-progress.md,
implementation-milestone-4-review.json, the architecture/format/boundary, and the
existing smoke contract/plan. Trace `src/schema.rs` in the Store crate, its public
Create/Connect callers and SQL schema validation. Read existing smoke observation
and verification owners before touching their receipt plumbing. Respect benchmark
hosting rules: host SQLite/SDK/spools and managed Docker daemon/FUSE.

## Goal and evidence motivating the experiment

Evaluate one 4-KiB page candidate against corrected M4's 64-KiB layout while
preserving the filesystem and physical encoding. This is allocation granularity,
not a 4-KiB CDC chunk or compression-group change.

Corrected small-file allocation is 2,162,688 B versus prior 1,114,112 B. Pack BLOBs
increased only 4,048 B. A legitimate new COW inode-table leaf made a 4,391-byte SQLite
row exceed the rightmost leaf's 1,218 free bytes, adding one 65,536-byte B-tree page.
Logical SQLite size is 1,179,648 B; allocated bytes exceed it by 983,040 B. The
remaining allocation mechanism is unestablished. `stat` does not establish physical
allocation placement beyond EOF. Smaller pages are plausible mitigation, not a
proven causal fix. Small-file storage acceptance remains open; DeepSeek savings do
not cancel it. Preserve historical diagnostic misses without relabeling them.

## Exact product scope

1. Separate new-Store creation policy from accepted page sizes in existing
   `crates/layerfs-layerstack-store/src/schema.rs`.
2. Create new Stores with `PRAGMA page_size=4096`, only in Create mode and before
   schema creation/cache configuration.
3. Explicitly accept page sizes `{4096, 65536}` for otherwise supported schema-6
   Stores in the existing verifier used by read-only preflight and post-open checks.
   Preserve exact application ID, schema objects, foreign keys, old-schema and WAL
   rejection. Do not accept every SQLite page size or create a configuration framework.
4. Preserve ordinary access to supported 64-KiB Stores at their original page size.
   Never convert, VACUUM, rewrite page size, or replace a Store on Connect.
5. Document direction: new M4.5 code supports both layouts; old M4 binaries reject
   new 4-KiB Stores. Schema 6 and pack wire 1 remain unchanged; this is an explicit
   minimum-reader capability change, not bidirectional old-binary compatibility.

Keep unchanged: canonical encoding/ObjectIds, CDC profile, COW construction, M4
FULL/DELTA selection and depth, Zstandard level 1, group/pack/admission bounds,
2-MiB physical/8-MiB inclusive allowances, publication/finalization, journal and
synchronous policy, mmap/locking/spill settings. Keep `cache_size=-32768` KiB, not
an equal page count. That cache setting is not a total-process memory guarantee.

Expected product change is schema.rs plus directly affected constant references.
Do not bulk-replace 65536: group limits, read buffers and CDC-related sizes are
independent. No SQL table/index redesign, new crate, cache, backend or migration.
Existing stale schema-5 tests are preexisting maintenance gaps, not evidence that
M4.5 caused them; do not expand into unrelated test repair.

## Verification and evidence custody

Only the existing three smoke workloads are authorized: first-five DeepSeek,
frequent edits with distinct SDK/ordinary routes, and small files. Matching builds
are preparation. Run one coherent candidate observation under the existing
development procedure; fix root causes and rerun only invalidated/affected cases.
No page-size sweep, new population, unit/fuzz/race/crash/full-workspace suite,
benchmark campaign, or M5 final three-pair qualification. A negative result is valid.

Fresh 4-KiB candidate Stores must verify all 33 historical mappings, no-change and
cleanup with existing independent oracles. Retain corrected M4 as the historical
control; comparisons are not fresh matched timing pairs. Do not rerun controls
merely to obtain favorable allocation. Preserve original receipts and manifests.

### Explicit cross-version compatibility verification

Fresh candidate smokes do not prove existing 64-KiB readability. The ordinary
runner correctly rejects a verifier with a different binary/image than its saved
producer. Do not bypass or weaken that check.

A narrowly scoped compatibility-verification receipt/mode is authorized within the
existing smoke verification owners. Use disposable copies of retained corrected
M4 smoke Stores and their existing retained-history oracles, including selected
DELTA content. Record original producer binary/image/source and actual new verifier
binary/image/source separately, plus source/copy hashes. Do not overwrite the old
run directory, manifests or Stores. Permit normal verification-created records only
in disposable copies. Confirm page size stays 65536. Copies are for compatibility,
never for allocation comparisons. Keep this outside performance timing; it is not
a fourth workload or a new benchmark family. If implementation cannot supply this
proof, report the explicit incomplete compatibility item rather than claiming it
from source alone. Do not run conversion to manufacture a compatible copy.

### Measurements

Reuse existing observer/receipt owners (`workspace_bench.rs`, `storage_smoke.rs`,
shared smoke orchestration as needed). At existing observation points report actual
page size, page count/freelist, logical DB size, filesystem allocated bytes and
applicable sidecars; retain total allocation at acknowledgement as primary.
Outside timed product work report available table/index/overflow/unused-page census,
pack/record bytes and counts. Record first/repeated reads, foreground mutation/Commit,
Init, CPU/RSS/I/O and spools. Logical BLOB calls are not physical pager/disk calls.

Explain randomized inode/COW population differences and canonical/encoded-byte
variation; do not attribute them all to page size. Do not freeze a favorable seed
or change fixtures. No promised 16x Store shrink, reclaimable-unused-byte total,
5% pack-overhead gate, or invented numerical storage threshold.

## Allocation diagnosis

Start read-only from retained evidence and relevant SQLite/VFS/file-growth paths.
If necessary, one explicitly identified instrumented execution of the same
small-file smoke is authorized for causal diagnosis, serialized under the same
measurement lock. Record it separately from clean performance observations; do
not treat traced elapsed time as an uncontaminated performance result.

Seek evidence of allocation-related fcntl requests/results, write offsets/lengths,
truncation, caller identity and file size/allocated-block changes. Preallocation is
a hypothesis until observed. Kernel tracing may require administrator access;
do not bypass permissions or claim fs_usage alone proves the request parameters.
If privileges prevent the causal trace, continue independent candidate work and
record the limitation. Do not add operations beyond the unchanged smoke to claim
later reuse of spare allocation or a general bound.

A favorable 4-KiB result does not automatically close unexplained allocation
amplification. Distinguish observed mitigation, proven cause and unmeasured bounds.
No retained-Store modification, post-operation truncation/VACUUM/repacking, dependency
source patch, custom VFS or durability-policy change is part of this experiment.

## Acceptance, rollback and stop

Assess complete allocation against corrected M4 and original baseline with honest
comparison scopes. Owner guidance accepts roughly 30% extra longer-operation
elapsed for meaningful storage improvement; approximately sub-50-ms TOTAL public
operations can make percentage increases immaterial. Not 50 ms extra per lookup,
not permission for unexplained aggregate pressure, not a new historical gate.

- Retain 4-KiB creation if observed allocation benefit warrants read/write/resource
  cost and compatibility verification succeeds.
- If benefit is weak or cost unacceptable, report the negative result and recommend
  retaining the 64-KiB creation default. Preserve support for any newly created
  supported 4-KiB Stores; reverting the default must not strand their data.
- If behavior or evidence remains unresolved, report a development checkpoint with
  the explicit open acceptance issue, not release readiness or a fabricated pass.

Do not automatically try 8/16-KiB, adjust packs/codecs, or start M5 to rescue results.
Fix demonstrated errors within scope; no indefinite search for a favorable run.

## Completion checklist

- [x] Actual source custody and prospective single-candidate policy recorded.
- [x] Creation/accepted-layout separation implemented; all other settings unchanged.
- [x] Existing constant assumptions reviewed; minimum-reader direction documented.
- [x] Three candidate smokes, historical reads/no-change/cleanup completed.
- [x] Candidate reader verifies disposable 64-KiB M4 copies under separate custody.
- [x] Allocation layers, canonical variation and performance/resource costs reported.
- [x] Allocation diagnosis has a proven result or precise unresolved limitation.
- [x] No old evidence, original Store, workload or accounting boundary altered.
- [x] Actual results written to new `implementation-milestone-4.5.json`; progress
      records retain/reject/unresolved disposition and remaining M5 work.
- [x] Diff reviewed, commit/push existing draft PR, no merge. Stop at M4.5.

A task may report a genuine external blocker or unqualified result, but must not
mark missing empirical compatibility or unresolved acceptance as passed. No M5,
S3, conversion tooling, schema consolidation or new durability work is authorized.

## Final owner disposition (2026-09-08)

The owner accepts 4096-byte SQLite pages as the default for new Stores and closes
#86 as completed. Preserve support for existing 65536-byte schema-6 Stores, with
no page conversion on Connect. Old M4 binaries reject new 4-KiB Stores; upgraded
M4.5 code supports both layouts. All raw evidence and original allocation-growth,
timing and resource observations retain their original status. This decision is
not full release qualification and does not establish the cause or bounds of
allocation amplification. That unresolved work remains in #83, M3-R6, with the
latest M4.5 evidence. No new optimization, measurement, migration, merge or M5 work
is part of this disposition.

# Handoff: measure current-main Commit and attribute its cost

Work end to end on **measurement and root-cause attribution only**: inspect the
current code, freeze the protocol, build, collect a plain baseline, collect separate
phase diagnostics, verify correctness, and report the dominant measured cost.
This handoff authorizes that execution. Do not stop at a proposed plan.

**Stop after the baseline, breakdown and one evidence-led next-step recommendation.**
Do not implement a performance optimization, change a storage policy, promote the
old guarded-predecessor candidate, or restart the Init optimization campaign.

## Objective and issue context

Repository: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`.
Coordinate/report through https://github.com/Ephemeral-AI-Lab/layerfs/issues/111,
with #115 and #108 as related metadata/Commit context. Keep #111's original cold
Init objective and open status; this task is the next Commit investigation, not a
claim that the 2.7 s Init target was achieved. Do not retitle or close that issue.

Answer these owner questions with current measurements:

1. What does the public main Commit path cost for no changes, one small change,
   repeated changes/reversion, and a larger changed set?
2. How does a retained Store differ from a Store reopened before workspace creation?
   Does index reconstruction still dominate after the fingerprint-index change?
3. Where does time go: workspace change discovery/reconciliation, canonical/tree
   construction, metadata-pool preparation, admission/encoding, and publication?
4. Which work scales with total namespace/history versus the changed portion?
5. Does the public SDK edit already perform work before Commit? How does that
   differ from ordinary filesystem/FUSE writes followed by the same public Commit?
6. What single measured bottleneck, if any, warrants the next optimization experiment?

## Current state: verify, then preserve

At handoff preparation, local HEAD was `ec653be07` on main. The handoff itself may
add a docs-only commit. Re-read HEAD/status and record the actual starting point;
never reset to the recorded hash or discard later work.

Relevant landed commits:

- `441be212e`: compact fingerprint metadata value index, with full authenticated
  value comparison and the same earliest retained ordinal behavior.
- `8def17a7b`: shared cold fixture metadata guard; current qualification contract
  is `namespace-100000-cold-v2`.
- `ec653be07`: metadata guard verification report.

Expected current retained build identities (verify rather than assume):

- Product: `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`
- Source: `371d5dc40336492ed4aa969f4d210536bfbf740a8e0ac0f8a7c44e1ada1ac38b`
- Host `target/release/fs-benchmark-pro` SHA256:
  `0051058ac8e9ffca19fee65e595c19a43abc64ad315536aa14abc2f7e6983b63`
- Linux image: `layerfs-bench-infra:371d5dc40336492e`
- Immutable image identity:
  `sha256:b394c02bcb0605b568464baac21bb0bbc3fa9b620a1e8b8c4940374d6eb61c0b`

Uncommitted compaction-removal changes remain across about 21 tracked modified/
deleted files, plus `compaction-removal.md` and untracked issue112/issue113 content.
Capture the complete initial status and binary diff; preserve others' files
byte-identically. Do not commit, attribute, revert, rebase, stash or clean that work.
The declared storage treatment remains **promoted-uncompacted**.

Use isolated worktrees/snapshots for instrumentation. Check all applicable
AGENTS.md files, including parent/root instructions if present and
`benchmark/AGENTS.md`. Do not modify any sealed evidence root.

## Read first

Under `docs/roadmap/0.1/0.1.5/issue111/`:

- `fingerprint-index-results.md` and `fingerprint-index-contract.md`
- `cold-metadata-results.md` and `cold-metadata-contract.md`
- `index-attribution-results.md` and `index-attribution-contract.md`
- `metadata-proof-experiment-results.md` and its contract
- `restart-index-design-results.md`
- `final-tree-rca-results.md` and `metadata-preparation-study.md`

Also read:

- `docs/general/benchmark_rules.md` (especially authentic operations, timer
  boundaries, cache state, paired arms, source custody and independent proof)
- `benchmark/fs-bench-pro/QUICKSTART.md`, `shared/runner.py`, `shared/cold.py`,
  `verify-selected.py`, and relevant existing family collectors/registries
- `crates/layerfs-sdk/src/client.rs`: `commit_workspace_session` and edit APIs
- `crates/layerfs-workspace/src/lifecycle.rs`: public Commit routing and lifecycle
- `crates/layerfs-layerstack-store/src/workspace.rs`: reconciliation/publication
- `objects/admission.rs`, `objects/admission/metadata_values.rs`,
  `objects/metadata.rs`, and `telemetry.rs` (`WorkspaceCommitReceipt`,
  `PhysicalStorageReceipt` and nested publication clocks)

Read-only reference evidence:

- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-index-attribution-evidence/20260911T074548Z/`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-metadata-proof-evidence/20260911T085037Z/`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-fingerprint-index-evidence/20260911T092737Z/`
- `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-cold-metadata-evidence/20260911T134416Z/`

The older index diagnostic harness (`source/benchmark/fs-bench-pro/src/index_diagnostic.rs`)
and the fingerprint experiment's external `commit-smoke` can inform implementation.
Copy/adapt source into a new evidence root; never run archived collectors in place
or modify their outputs. Preserve the original #104/#109/#110 evidence as well.

## Do not carry stale conclusions forward

Before the fingerprint-index change, first Commit after Store reopen measured
roughly 250 ms and replayed 100002 metadata values. An isolated guarded-predecessor
candidate measured roughly 13 ms, but was not promoted. Neither number is a
current-main baseline. That candidate also had retained-mode and storage-screen
uncertainty; do not cherry-pick it into this study.

The last valid Init comparison measured about 3.420 s candidate median, and still
missed 2.7 s. It predates the shared v2 metadata receipt and had an explicit metadata
audit. Treat it under its recorded protocol, not as a new v2 qualification.

## 1. Trace and freeze before edits or timing

Trace the complete public call path and every caller of any function to instrument.
Reuse existing receipts before adding clocks. Identify SDK-prepared versus ordinary
filesystem dirty-state branches; do not assume both perform the same work at Commit.

Create a new timestamped evidence root outside the repository and record it in a
new pointer file. Commit a concise measurement contract and update #111 before
instrumentation/collection. Freeze:

- source snapshots, operation routes, timer start/end, fixture versions/digests;
- exact cell matrix, independent repetitions, traversal/arm order, edit paths,
  offsets, markers, file classes, changed-file counts and byte counts;
- Store/index lifetime, OS-cache policy, bootstrap and workspace setup sequence;
- correctness/resource/cleanup requirements and missing-evidence rejection;
- diagnostic fields and how exclusive versus nested phases will be reconciled;
- invalid-attempt rules: retain everything; at most one bounded replacement of
  the entire affected pair/cell for demonstrated infrastructure invalidity.

Recommended bounded minimum: plain n=3 independent Stores per core cell; separate
nonce diagnostics n=2 per cell needed for attribution. Do not count several commits
on one Store as several independent repetitions. Freeze any necessary scope change
before seeing results; do not grow a campaign in response to attractive timings.

## 2. Measure current-main public Commit

Use the four actual registered namespace fixtures:

| Case | Files | Logical input bytes |
|---|---:|---:|
| namespace-100-compact-v3 | 100 | 5000000 |
| namespace-1000-compact-v3 | 1000 | 20000000 |
| namespace-10000 | 10000 | 300000000 |
| namespace-100000 | 100000 | 500000000 |

Read the registry to confirm these IDs; there is no `namespace-10000-compact-v3`.
For the largest fixture, digest is
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.
Use full public `Client::initialize_layerstack` and branch creation for bootstrap,
outside the Commit timer. Keep all files/bytes; no reduced fixture or cloned Store
substitute. Reuse immutable input bytes, not a prebuilt SQLite output Store.

Core cells across namespace sizes:

- **No change:** fresh workspace, no edit, public Commit; expect the actual
  `UpToDate` result and unchanged head. Keep this separate from first-change cells.
- **Retained first small change:** retain Client/Store from bootstrap, create the
  workspace, make one deterministic 10-byte public SDK range edit, then Commit.
- **Reopened first small change:** drop every Client/Store owner, reconnect the
  same SQLite Store, create the workspace, perform the same edit, then Commit.
  Do not recreate the index while inspecting its state.
- **Repeated change/reversion:** after the first changed Commit in retained and
  reopened chains, apply a different marker and Commit, then revert and Commit.
  Report each stage separately. A later no-edit Commit may be an additional check;
  it must not become an untimed warm-up before the first changed measurement.

At namespace-100000 also measure a deterministic larger changed set, preferably
K=10 and K=100 files in addition to K=1, spread across declared directories/leaf
regions. Fix file classes and total supplied bytes; use the public batch edit API
where applicable. Report edit preparation separately from the one public Commit.
This adds a changed-set scaling axis without restarting every benchmark family.

Primary comparisons should match the public SDK edit route used by prior evidence.
If ordinary POSIX/FUSE writes take a different discovery/reconciliation branch,
include a bounded, separately declared companion at namespace-100000 using an
existing registered filesystem/workspace workflow. Label the two mutation routes
separately. Never substitute POSIX/FUSE writes for an SDK-edit claim, or describe
SDK-prepared Commit timings as covering an unmeasured ordinary-filesystem branch.

Time **only `Client::commit_workspace_session`** through its promised return/
publication acknowledgement. Record observed public-call counts and Created versus
UpToDate outcomes. Capture Init/bootstrap, Store reconnect, workspace creation,
edit and post-Commit checks in separate intervals. Report edit+Commit totals as
secondary context so eagerly prepared SDK work is visible. Do not relocate work.

## Cache and fixture contract

Store reopen and OS page-cache coldness are independent axes. The primary Commit
study may declare OS cache **uncontrolled**; repeated retained Commit is a genuine
workload. Do not call it a cold result, infer cold from read volume, warm it with
extra untimed Commits, or mix lifetime/cache/sequence cells in an aggregate.
Report lifetime, sequence position, cache profile, process disk reads/writes and
any actual acquisition evidence per sample. Do not invent a Commit latency target
or reuse Init's 2.7 s gate.

Validate fixture bytes AND file/directory modes/mtimes, including the root, even
when bootstrap is outside timing. Prefer the original immutable fixture. A prior
`cp -cR` copy changed directory timestamps and produced 116454 instead of 112451
canonical objects at 100000 files. Byte hashes alone did not catch that error.
Use the shared metadata guard/inventory logic; if copying is necessary, preserve
and verify metadata rather than trusting a copy flag. Never repair a measured
input afterward to relabel its run. Keep full bootstrap counts/bytes and reject
wrong-fixture cells. Any actual cold Init qualification would require v2 evidence;
this task does not need another Init acceptance campaign.

## 3. Attribute actual Commit time

Keep plain baseline and nonce/profile measurements separate, with distinct seals
and folders. Do not subtract guessed instrumentation overhead or pool them.
Use minimum operation-scoped diagnostics, preferring phase boundaries over a
clock per row. Preserve the plain public operation and output semantics.

Measure a disjoint top-level wall-time partition based on the actual code, including:

- SDK/coordinator queueing, cut/freeze and required waiting;
- dirty-state discovery/reconciliation or already-prepared candidate handling;
- canonical content/tree construction, including final root/inode updates;
- admission preparation and encoding;
- transaction/publication and acknowledgement;
- explicit remaining outer/unattributed time.

Within those enclosing phases, separately report relevant nested work:

- metadata-pool preparation, ValueIndex creation/sync/replayed values/eviction;
- fingerprint candidate counts, exact group authentication, lookup and assignment;
- pack/zstd/FULL/DELTA/CDC work, canonical/reused/admitted object counts and bytes;
- final tree nodes/leaves touched versus namespace size and changed-file count;
- Store SQL/page writes, transaction begin/insert/metadata/commit work;
- input reads, consumer waiting and other exposed waits where measurable.

Nested costs must not be added to their parent totals. Concurrent CPU/work sums
are not wall-time partitions. If existing clocks overlap, label that explicitly
and add only the minimum needed to establish a critical-path breakdown. Reconcile
top-level intervals to the actual public timer and keep the residual visible.

Compare counters before workspace creation, after creation, after edit and after
Commit. Establish whether work moved naturally into setup/edit rather than claiming
it disappeared. Explain unexpected scaling with measured counts, not curve-fitting
alone. Distinguish namespace cardinality, total physical history, retained index
window and touched leaves. Do not claim an unmeasured long-history bound.

## Correctness, resources and custody

- SQLite, SDK/coordinator, admission/publication and spool remain on macOS.
  Docker supplies daemon/FUSE/workload only. Read benchmark/AGENTS.md.
- Keep 4 KiB pages, scratch/main cache settings, worker counts, schema10, exact CAS,
  pooled metadata, pack/zstd/FULL/DELTA/CDC and authentication unchanged.
- Use `shared/runner.py --build-host` and matching `--build-image`. Retain exact
  source/product/fixture seals and binary/image identities for every cohort.
- Use the runner-owned measurement lock; serialize builds, tests, benchmarks and
  resource-heavy proof. No double-acquisition in children or overlapping runs.
- No edits under crates/, tools/ or benchmark/ between a build and completion of
  its measurements without resealing/rebuilding. Keep plain/diagnostic work isolated.
- Record user/system CPU, disk reads/writes, resource limits, swaps/OOM, threads,
  spool/index bytes and available memory observations. Distinguish RSS snapshots
  from process-lifetime peaks; do not label post-call RSS as phase peak memory.
- Verify expected head/result, every changed file's complete bytes where bounded,
  unchanged sampled files, final root/content after Store reopen, and cleanup.
  Keep verification outside timing and state sample versus exhaustive coverage.
- Compare canonical/physical counts, pooled values and encoding outcomes so a
  missing persistence or encoding step cannot masquerade as faster Commit.
- Run focused existing suites for instrumented paths; do not restart unrelated
  #104 families. No weakening of independent proof, budgets or invariants.

## Deliverables and stopping point

1. `docs/roadmap/0.1/0.1.5/issue111/commit-baseline-results.md`: current-main plain
   timing tables with n/median/range/raw samples, setup/edit context, cache/lifetime
   labels, resource/physical receipts, correctness and exact custody.
2. A measured phase table/diagram with disjoint totals, nested details and residual,
   plus namespace-size and changed-set scaling tables. Retain every attempt,
   command, log, exit, source patch, binary and analyzer in the new evidence root.
3. Answer the six owner questions. Rank the measured costs and select **one** next
   optimization hypothesis, with expected ceiling, correctness risks and falsifiable
   test—or explicitly recommend no optimization if the evidence does not justify it.
4. Update #111, linking #115/#108 and relevant existing evidence. Keep Init's target
   open. Commit only intended measurement/instrumentation/docs files, preserving
   the original dirty work. No optimization commit, release, tag or deployment.

Do not stop early with only a trace or plan. Complete measurement and attribution,
then hand back the report and recommended next experiment without implementing it.

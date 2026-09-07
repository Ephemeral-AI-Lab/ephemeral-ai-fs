# Storage implementation handoff prompt

Copy the prompt below into the implementation agent's task. This file is a prompt
artifact; creating it starts no implementation, smoke execution or migration.
Its executable verification scope is the owner's three agreed development smokes.

---

Implement LayerFS's agreed storage architecture v3 using the implementation plan.
Work autonomously through implementation, smoke verification, root-cause fixes and
justified optimization until the completion contract below is satisfied. Do not
stop after a plan, a partial patch, a compiling build, one passing smoke, or a
correct-but-unimproved candidate. The goal is correct production-path implementation
with strong, attributable smoke evidence of material retained-storage improvement
while preserving useful speed, especially localized edits.

## Read, establish custody, and resolve actual prerequisites

Read applicable AGENTS.md and relevant implementation skills. Then read in full:

- `docs/roadmap/0.1/0.1.4/implementation-plan.md`
- `docs/roadmap/0.1/0.1.4/storage-architecture-spec.md`
- `docs/roadmap/0.1/0.1.4/sqlite-storage-format.md`
- `docs/roadmap/0.1/0.1.4/storage-efficiency-boundary.md`
- `docs/roadmap/0.1/0.1.4/review-disposition.md`
- `docs/roadmap/0.1/0.1.4/evidence.md`
- The agreed development-smoke plan from PR #80, plus its referenced hosting,
  runner and immutable input contracts.

Follow the implementation plan's exact source-reading list before changing each
owner. Trace callers and siblings; do not trust a method name such as batch, commit
or authenticated without reading its behavior. Reuse the current public SDK,
shared content constructors, runtime, readers, spools and measurement infrastructure.

Discovery references:

- Specification and implementation plan: https://github.com/Ephemeral-AI-Lab/layerfs/pull/79
- Smoke plan: https://github.com/Ephemeral-AI-Lab/layerfs/pull/80
- Pinned smoke-plan revision: `d9ec9c6714ca31adb7a337d2ac0f40976513908c`
- Original source trace baseline: `28177560c8f049c02192e18c263cdc5543c1ab52`
- Architecture v3 correction: `555d91f0cd74148364331e24acf0ba14408d7c78`
- Implementation-plan revision: `34336e5b798a1d59efccd83b2412cea3944963fa`

Record the actual selected source/branch/doc identities and any later differences.
If the planning PRs remain unmerged, read their actual reviewed documents rather
than substituting old main. Work in an isolated codex/ branch/worktree; preserve
unrelated changes and user Stores. Do not merge a planning PR into main just to
access its files. Start no historical product checkout or obsolete container-Store
route to bypass current rules.

Use already-approved decisions without requesting them again. If the documented
format/release/legacy policy has not actually been decided, prepare one concrete
request for the missing decision: narrowly scoped 0.1 exception versus 0.2 placement,
new-Store-only scope and any required legacy access/conversion. This prompt grants
no implicit exception or in-place migration. Complete useful source/document
preparation while the necessary policy is unresolved; do not start an incompatible
product switch without it.

Before observations, finalize and record PR #80's outstanding smoke fixtures,
entrypoints, execution budgets and comparison/repetition rules within the agreed
three-smoke scope. Do not silently invent a new population or accept a moving
contract. If numerical storage/speed acceptance remains unagreed, present a concrete
smoke-specific decision before candidate optimization/qualification; do not set
favorable gates after seeing candidate results. Ask only genuinely missing policy
questions, not permission for routine authorized code, build or smoke work.

## Product model and non-negotiable implementation requirements

LayerFS retains complete immutable filesystem graphs and provides mutable COW
Workspaces through real FUSE. Canonical ObjectIds are independent of SQLite rowids,
pack placement and codec choices. Init and Workspace capture/Commit share one
canonical/physical admission pipeline while preserving distinct public lifecycles.

Preserve:

- CAS exact reuse, the current CDC profile, extent/namespace COW and canonical bytes.
- Localized edits that retain unchanged payload slices and touch affected tree paths.
- Ordinary whole-file replacement semantics; writes do not magically reveal a diff.
- One small/large-file storage path, with no allocation padding or migration at 8 KiB.
- All authoritative representations and locations in SQLite, including pack BLOBs.
- Synchronous required encoding and finalization before public success.
- Bounded multiple transactions; staging, head/base checks, no-change and honest
  published-but-finalization-failed results.
- Ordinary integrity, authentication, historical readability, isolation and cleanup.
- Explicit full-base dependencies, portable identity/framing and physical closure.

Linear admission/recheck work and proper batching are correctness-of-design
requirements, not optional tuning after a small smoke happens to pass. Implement
one initial membership probe and at most one final recheck per distinct candidate.
Retain the batch permit during late-duplicate validation but release SQLite handles
and the connection before decode/hash/base access. No nested permits, shrinking-set
retry, whole-Init permit or unsafe whole-Store cleanup.

Implement the named bulk/page and ownership corrections together with their callers:
parameter/byte-bounded pack/locator INSERTs, scratch seen/offset page operations,
order/duplicate/pending visibility, buffered sealed ID I/O, direct location transfer,
internal target/base group waves, incremental pack counters and targeted FIFO handoff.
All Init source strategies share finalized-output ownership; do not clone producer
payloads into a second parent owner because the Store is nonempty.

Carry legitimate prior-file context through truncate/rewrite, complete-build and
captured-tempfile/rename routes. Use a pinned-path predecessor only for physical
similarity, never as the replacement inode's logical identity. Retain per-file
first-span facts and one forward old-extent cursor; no repeated old-tree walk,
second source/CDC pass merely for hints, or alias-times-chunk lookup.

State indexed O(log M), required canonical ordering and actual byte work honestly.
No quadratic orchestration becomes acceptable because it has a finite cap or agents
usually issue calls slowly. Read repeats across internal drains and dependent tree
levels count. Queue delay and aggregate CPU/disk/memory pressure are separate costs.
Preserve the documented remaining scope of overlapping reconciliation fingerprints;
do not claim every Commit path is globally linear.

Optimize for the smallest clean final system. Follow the proposed module tree and
KEEP/MERGE/REPLACE/DELETE ledger. No parallel Init/Commit encoders, plugin framework,
new scheduler/service, global similarity index, persistent cache hierarchy or separate
small-file backend without an indispensable current responsibility and explicit
scope justification. No cloud implementation, new durability/fsync contract, crash
campaign or in-place migration.

## Fast implementation and iteration loop

Follow the implementation plan's milestones:

0. Policy/smoke prerequisites and an unchanged current-code baseline.
1. Reusable canonical ownership and spill/page-I/O preparation.
2. One complete packed FULL/RAW read/write/publication slice; switch all writer
   callers, permit scopes and unsafe cleanup assumptions as one coherent unit.
3. Specified group compression and its bounded read/write ownership.
4. Whole-file correspondence plus shallow delta emission/read/base closure.
5. Remaining prescribed cleanup and one integrated final-candidate smoke pass.

Intermediate raw-only/no-delta checkpoints are development milestones, not the final
contract. Do not introduce a temporary raw backend interface simply to delete it
later. Never enable a writer before the matching reader/publication path is complete.

For each coherent slice:

1. Read the touched code and all relevant callers. State the intended change and
   preserved behavior briefly. Maintain the implementation checklist as you work.
2. Implement the smallest clean replacement, including deletion of obsolete paths
   when their replacement becomes usable. Do not leave correctness debt for later.
3. Build only affected matching host binaries/runtime artifacts needed by the smoke.
4. Run the smallest affected agreed smoke with fresh mutable state and declared
   timing/accounting. Do not rerun unrelated passing smokes after every patch.
5. Inspect correctness, operation-route authenticity, retained allocation, elapsed
   phases, resource/cleanup status and available compact query/group/byte receipts.
6. On failure or a meaningful regression, identify the root cause before editing
   again. Make one justified correction and rerun the affected smoke. Preserve all
   failed observations and valid slow results. Do not repeat a disproven hypothesis.
7. Once that slice is correct and has no unresolved implementation-contract violation
   or actionable smoke regression, proceed to the next milestone.

Continue this loop until the whole implementation and joint storage/speed evidence
meet the completion contract. Do not stop simply because the first implementation
is functional, or keep tuning after the agreed objectives are met.

## Only the three agreed smokes are executable verification

Use:

1. **DeepSeek first FIVE frozen manifest entries**, starting with empty retained
   history and preserving ordinary Exec/FUSE import, repeated Commit and historical
   oracle semantics. These are manifest entries, not the first five Git commits.
2. **Frequent edits and Commit**, with SDK range edits and ordinary Exec/FUSE writes/
   complete replacements separately reported, retaining recurring/no-change behavior.
3. **Small-file Init and mounted readback**, with the agreed changes and retained-state
   checks from PR #80.

The topology is host SQLite/SDK/coordinator/spool plus managed Docker daemon/live
core and real FUSE. Use the current shared lock, resource configuration, authenticated
runtime, artifact custody, cleanup and immutable preparation. No host materialization,
container-owned SQLite, direct SQL injection or private helper replaces a smoke's
public operation. Existing --smoke is not proof that these entrypoints already exist.

Do not add/run unit, property, fuzz, race, crash, all-workspace, full-family or extra
reconciliation suites during this task. Builds required to run the smokes are
preparation. Source review protects unexercised invariants but must not be reported
as executed verification. The full 157-state campaign and full release qualification
are not automatically triggered when these smokes pass.

Use the agreed independent correctness and historical-read checks. Keep verification
and expensive accounting outside operation timing, and measure primary retained
allocation before verifier-created records can change it. Required mutation,
encoding, admission, publication and finalization work stays in its declared scope.

## When further optimization is justified

Fix a violated design requirement immediately, even if a tiny smoke hides its cost.
Beyond that, optimize only when source inspection establishes a concrete redundant
or faulty mechanism, or the smoke evidence identifies a meaningful cost:

- Growing-prefix or shrinking-set rescans instead of forward/indexed access.
- One-by-one SQL, scratch transactions, I/O or group decoding where known work can batch.
- Repeated full-file/namespace work instead of the intended localized path.
- Duplicate copies, hashing, traversals or unnecessary process/RPC crossings.
- Excessive base preparation, poor encoding decisions or unnecessary anchors with
  attributable storage/time impact.

Use existing receipts and bounded diagnostics before adding instrumentation. Do
not put per-object logs/censuses into operation timers or build another performance
framework. Treat the component and its shared callers, not a specific fixture.
A large rewrite is acceptable if it removes the demonstrated root cause and leaves
a smaller final system; aesthetics or speculative flexibility are insufficient.

## Strong evidence and accepted degradation

Success requires both correctness and a worthwhile storage/speed outcome, not merely
correctness PASS. Compare the unchanged baseline and final candidate on identical
frozen inputs, public operation surfaces, cache/preparation treatment and timing.
Use the prospectively declared independent repetition/comparison policy inside these
same smokes to distinguish the improvement from noise. Do not tune the repetition
count, discard valid samples or change workloads to rescue a failing result.

Report absolute allocated bytes and elapsed time alongside:

- storage reduction = 1 - candidate allocation / baseline allocation;
- elapsed increase = candidate elapsed / baseline elapsed - 1;
- phase and case differences, including localized edits, Init and readback separately.

Count pack/record/index/framing overhead, FULL anchors, partial packs, redundant and
unpublished admitted records, required SQLite allocation and separately scoped
spools/temporary peaks. No post-run repack, background completion or payload-only
numerator. Do not add overlapping CAS/COW/delta/compression savings percentages.

The owner accepts some degradation for substantial storage improvement. Roughly
50% more elapsed time for genuinely Git-close allocation may be worthwhile; 50%
more time for only 10% less allocation is unacceptable. This is not a universal
1.5x allowance, an approved Git-closeness tolerance, or permission to hide a severe
read/localized-edit regression behind an average. Apply the agreed operation-specific
smoke decision criteria; preserve localized editing as a core advantage.

Getting close to matched Git allocation is the direction, not exact parity. Git is
not a default extra smoke arm. Never divide five candidate states by the historical
157-state Git footprint, or treat separately packed Git size as its foreground commit
cost. Where no matched reference is authorized/available, report improvement against
the unchanged LayerFS baseline and label Git proximity unmeasured.

If storage improves only slightly at disproportionate cost, continue diagnosing and
fixing the shared design/implementation within scope. Do not declare success by
lowering gates, concealing overhead, swapping operation surfaces or weakening integrity.
If the frozen objective demonstrably cannot be reached by this design, present the
specific evidence and smallest justified revision; do not fabricate a passing result.

## Checklists, stopping rule and final delivery

Maintain one compact progress ledger mapping every **in-scope** implementation,
deletion, smoke execution and reporting item to its status and evidence. Link the
implementation plan, architecture obligations and eight-item review disposition.
For each correctness obligation distinguish:

- implemented and source-reviewed;
- exercised by the named smoke and verified there;
- not exercised by the smoke scope, with its limitation explicitly recorded.

Do not tick an execution/verification claim based on code review, or mark unresolved
correctness as complete. “All checklists” means the agreed implementation and these
three-smoke task checklists, not every historical release matrix. Unexercised broad
parser/race/conflict cases remain honest coverage limits; do not silently add another
suite or use that limit to avoid implementing the specified safeguards.

Honor later owner stop instructions or scope changes. Otherwise, do not end the
task while safe, authorized work can advance it. Normal bugs, failed
smokes, manageable regressions and incomplete milestones are work to resolve, not
reasons to hand back a TODO list. Resume the same task after context compaction.
Stop only when all of the following hold:

- [ ] Every in-scope implementation/deletion item is finished; no temporary encoder,
      unsafe cleanup, nested permit or quadratic admission mechanism remains.
- [ ] The final source builds into matching host/runtime artifacts.
- [ ] All three smokes pass their agreed correctness, route and cleanup requirements
      on the final integrated candidate, with required historical readback.
- [ ] Prospectively declared smoke evidence supports material allocated-storage
      improvement and the accepted joint speed/resource tradeoff, including preserved
      useful localized-edit performance; no severe case is hidden by aggregation.
- [ ] Actual representation coverage, metrics, candidate/input identities, failures,
      comparisons and unexercised behavior are recorded honestly.
- [ ] Docs, receipts and the final source agree; the compatibility policy is followed.
- [ ] Coherent changes are committed/pushed in an implementation PR with the evidence
      and remaining release/coverage limitations. No merge/deployment/migration occurs
      without separate instruction.

A genuinely missing owner policy, unavailable required external resource, or a
substantiated design limitation that cannot be resolved within authority is the
only exception to continued work. First finish independent work, then report the
exact blocker, evidence, remaining checklist items and minimal decision/action
needed. Do not label that state complete, ask the same permission repeatedly, or
invent success to satisfy this stopping rule.

Final response: exact source/PR/binary/image identities; concise resulting architecture
and deletion summary; complete checklist disposition; baseline versus final smoke
storage/time/resource table; correctness and historical-read evidence; executed
coverage and omissions; compatibility status and any remaining owner decisions.
Make no universal correctness, Git parity or aggregate-capacity claim from these
three development smokes.

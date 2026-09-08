# D: unchanged-policy file-payload delivery diagnostic v1

Owner authorization: the explicit request to proceed with the #88 implementation
working note. This contract is frozen before diagnostic samples, after source and
deterministic test review. Implementation commits/binary/image seals are recorded
in each run's pre-execution identity, not inferred from this document's revision.
The original accepted M4.5 foundation is
`eb7050603c5ee97a02dfa0d5357619079d25ab51` in isolated branch
`codex/issue88-delivery-diagnostic`. No S1 or native-payload policy is enabled here.

## Question and one variable

Measure which actual public-path causes explain predecessor-present/no-hint
**unique initially missing regular-file payload targets**, in counts and full
canonical bytes. Distinguish first/inherited operation exhaustion, reservations
triggered by eventual reuse/duplication, missing required spans and completed
legitimate no-overlap. These are opportunity/delivery observations, not measured
compressed savings or a speedup claim.

One variable: bounded diagnostic observation and outside-timer receipt projection.
Preserve all actual cursor decisions, guard precedence, four-hint ordering, CAS
probes, duplicate first-owner choice, canonical bytes, current group/pack formation,
codec, FULL-base policy, matcher/trial/read limits and public operation route.
No refund, retry, new lookup, increased allowance, hint merge or changed scheduling
policy. Extra observer instructions and constant memory may perturb scheduling;
disclose that instead of calling D a clean paired performance control.

This is the finite prerequisite to the owner-selected S1+native-payload milestone,
not the endpoint. A valid outcome selects existing delivery, one separately declared
coverage treatment, or rejection under the existing constraints. It need not
predict how well the later native codec compresses a target.

## Exact observation ownership

Reuse two named u8 values in PhysicalHints: tag and outstanding reservation grants.
Tag bits0..2: unqueried0, complete1, memory2, file3, operation4, descriptor5;
bit3 is inherited exhaustion; bit4 is actual regular-file source context. All other
bits and combinations are rejected by diagnostic spill validation. Grants0..7;
inherited exhaustion carries zero new grants. The private144-byte hint frame and
184-byte row overhead remain, using reserved slots13/14 and zero byte15. Preserve
the charged PhysicalHints160-byte/authenticated object216-byte layouts and the
actual original PreparedObject layout/capacity. Fail preparation on a violated gate.

File provenance originates at the real file-construction/capture owner, including
new files without predecessors. Generic `put_file_payload` is also used for mode/
mtime metadata and cannot alone establish file use. A constant owner context flag
is explicitly part of D observer memory; no batching/capacity policy depends on
that flag or changes with it. Exact role validation remains mandatory. Metadata
values and structural objects are excluded from the required file-span cohort.

Tag/grants survive actual memory/spill delivery. At the existing duplicate or
initial CAS owner, credit each occurrence exactly once as preexisting, initially
missing, or duplicate, then clear its outstanding credit. The grant byte can carry
a terminal diagnostic label only after the final MissingBatch ownership transition,
where no further candidate spill/handoff occurs. An unsupported rebuffer/lost
credit invalidates diagnostic conservation, not product output.

Eligibility counts occur at initial CAS absence before optional matching-memory
exclusion. Capture specific fetch/match/instruction/memory budget events as well
as general skips; a budget-limited trial cannot become a claim of exhaustive
unsuccessful matching. A complete candidate may win despite incomplete extra
trials or limited correspondence. State scopes remain explicit.

No per-object log or global ObjectId tracking subsystem is added. Existing receipt
macros contain fixed counters/histograms. The source field list and serializer are
sealed with the implementation. Fixed stack/Store counter memory is disclosed.
Keep all original physical counter meanings intact. `diag_selected_pack_last_id`
is an absolute post-commit high-water gauge; never sum it across phases.
`diag_invalid` is sticky. Other diag counters are cumulative and use phase deltas.

## Required conservation and provenance

Record count/byte pairs for six exclusive cursor states: no predecessor, missing
span, complete empty, limited empty, complete hints, limited hints. Fixed size and
hint-count histograms partition the eligible cohort. First/inherited reason pairs
and empty-specific reason pairs separate limited-empty from limited-with-hints;
do not fabricate joint histogram cells from unrelated marginal counts.

Terminal attempt outcomes plus late races conserve eligible count/bytes. Admitted
FULL/DELTA counts/bytes describe winners only. Trial/event counters are overlapping,
not exclusive terminal targets. Occurrence grant totals equal actual successful
cursor reservation grants; reservation bytes =131,136 ×grants, not actual I/O or
avoidable waste. A reused chunk can initialize a cursor needed by a missing chunk.

After successful transaction commit, observe new pack count/bytes/groups/records,
unlocated records and last assigned pack ID from existing insertion facts. No extra
SQL query. For an operation with count>0, range first=last-count+1. Require a single
Store writer, contiguous disjoint operation ranges, complete final pack coverage,
and matching actual locator records. Empty operations have no new range; their
high-water gauge does not imply an addition. An aborted transaction contributes no
successful pack count/range. Pack IDs alone are not historical provenance.

The final analysis joins these successful ranges to the shared authenticated
selected-locator inventory. Incremental retained-root union verifies every Init+
157 prefix against acknowledgement canonical object counts/bytes. Exact file-use
traversal starts at inode content edges to FileState, excluding metadata FileStates.
Require eligible attempts and newly selected file-use chunks to match in count and
bytes, with DELTA equality and zero late races for the simple unique-cohort claim.
Ownership and source provenance must also pass; equal totals alone do not prove
a bijection. If invalid, preserve all raw attempts, output unique attribution null
with the exact reason, and do not invent zeros or extra measurements to force PASS.

## Preparation and bounded tests

Before sample collection: source review; exact layout/spill invalid-tag checks;
real metadata versus file-source construction; new-file and predecessor memory/
spill delivery; sticky first/inherited reasons; grant-credit duplication/shared
cursor conservation; actual late-race and partially selected pack publication;
specific trial-budget exits; gauge behavior and serializer-field completeness.
Use the pinned normal Rust toolchain1.85.1 and existing locks. Retain failed builds
and corrected identities. The baseline test-only private-field compilation defect
may use the already reviewed cfg(test) accessors; no production behavior change.

The aggregate validator self-check exercises negative conservation/gauge/race
cases and uses an explicit raw-D schema. Reuse the prior validator's arithmetic
where appropriate without labelling unmatched raw fields as its old joint schema.

## Frozen sample sequence and commands

Use the existing runner, original fixtures/import/oracles and public surfaces.
One deepseek-five diagnostic performance+verification run first checks actual
receipt transport, mapping, cleanup and aggregate behavior. If valid, one full157
performance+verification diagnostic follows. No three-arm optimization campaign,
new family, four-arm matrix or fresh baseline timing claim is made by D.
Repetition1; source-arm candidate denotes instrumented source, not optimization.

Build host and storage-smoke image sequentially with the normal runner:

```text
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-storage-smoke-image
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-five --source-arm candidate --repetition 1 --image EXACT_IMAGE_ID --host-binary EXACT_HOST_BINARY --output NEW_RUN_DIRECTORY
python3 benchmark/fs-bench-pro/shared/runner.py --storage-smoke deepseek-five --source-arm candidate --repetition 1 --image EXACT_IMAGE_ID --host-binary EXACT_HOST_BINARY --storage-verify-run SAME_RUN_DIRECTORY
python3 benchmark/fs-bench-pro/shared/runner.py --deepseek-full --source-arm candidate --repetition 1 --image EXACT_IMAGE_ID --host-binary EXACT_HOST_BINARY --output NEW_FULL_RUN_DIRECTORY
python3 benchmark/fs-bench-pro/shared/runner.py --deepseek-full --source-arm candidate --repetition 1 --image EXACT_IMAGE_ID --host-binary EXACT_HOST_BINARY --storage-verify-run SAME_FULL_RUN_DIRECTORY
```

Replace placeholders only with authenticated pre-execution identities; retain exact
commands. The snapshot/census steps below occur after performance close and before
normal verification when required. Record preparation and build/image cache use.
Do not bypass normal identity checks or wrap a runner that already holds the
measurement lock in a second lock. Unit tools use the common lock separately.

Frozen157 manifest SHA256
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`;
same source/tree/oracle identities and all157 mappings, including Init allocation.
Host owns SQLite/coordinator/spool; Docker daemon/FUSE/workload uses the existing
2CPU/2GiB/no-swap/256PID topology. Full157 preparation/performance/verification
budgets4h each; step300s; setup/cleanup120s; build900s/two jobs. Deepseek-five retains
its existing600s phase and120s step bounds. Preserve original16GiB Store and scoped
runtime/spool/staging limits,32GiB owned storage,50GiB free reserve and8GiB sampled
host RSS limits. No timeout/resource increase after observing a miss.

Stop on policy/layout, integrity/oracle/route, identity, memory/resource, cleanup or
diagnostic ownership/conservation failure. Retain failed/partial data. Correct a
demonstrated instrumentation defect with a new source identity and scoped rerun;
no parameter sweep. A valid no-opportunity result is not a failure to be rerun.

## Timing, snapshot and sealing

Cheap diag projection occurs outside the existing public timer. Product observer
instructions remain inside where they execute. Record public Init/Exec/Commit/
End, transfer, preparation, observer, case and invocation time separately; nested
codec/matcher work is not added to public elapsed. Separate host/container CPU,
RSS/lifetime/sampling scopes, I/O, spool/staging/runtime and free disk. Original
physical counters may include attempted work that loses admission; report it apart
from durable winners. No clean paired speedup or storage improvement claim from D.

At final acknowledgement retain original allocation/logical/pages/sidecars. Close
normally, authenticate receipts/source, preserve one final pre-verification logical
snapshot with source/copy hashes and custody. Run the reused exact canonical/pack
decoder and one incremental graph/locator analysis on the quiescent snapshot.
Detailed census is Init/final only; no Store copy/traversal after each Commit.
Normal verifier identity checks remain strict. Copies/APFS clones are not allocation
controls; disclose cache warming and observer time. Original historical Stores and
manifests remain untouched. Seal new raw evidence, validator source, outputs and
manifest; publish an additive D result and the finite delivery decision for P.

No S1 enablement, native encoding, candidate-policy change, migration, rollout,
merge, M5 or cloud work is performed as part of D.

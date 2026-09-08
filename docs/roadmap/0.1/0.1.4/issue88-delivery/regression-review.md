# D Store-library regression review

The D Store-library run reports **49 passed,9 failed,1 ignored**, not an overall
passing regression suite. All six new D tests pass. The previously run focused
filter also included one existing diagnostic test and reported7 passes.

Evidence: `layerfs-storage-v3-runs/issue88-diagnostic-custody-1/store-regression-1.log`.
This review reads the failed assertions and accepted `eb7050603` source directly.
No tests, product decisions or resource limits were changed to obtain a pass.
The parent is preparing an independent accepted-baseline run with only the known
cfg(test) private-field accessor repair; **matching baseline failure results are
pending at the time of this initial review**. Source comparison alone does not
establish runtime equivalence for every failure.

## Exact failures and evidence

| Test | Observed D failure | Accepted-source evidence and current disposition |
|---|---|---|
|`statements::tests::exact_manifest_prepares_against_exact_v6_schema`|Counts2 semicolons where1 expected in `objects/insert.sql`|The accepted test uses `sql.matches(';').count()`. Accepted SQL has one real INSERT terminator and a semicolon in the comment `one inserted; conflicts fail`. This is lexical test parsing drift, **not two SQL statements**. The SQL and test are unchanged by D.|
|`workspace::tests::snapshot_cache_reads_only_requested_authenticated_objects_and_reuses_them`|`InvalidParameterCount(2,5)`|The accepted test binds `(id,bytes)` to the accepted v6 locator INSERT, which requires ID,length,pack,group,record. This is directly visible test/schema drift; D does not alter the statement.|
|`objects::tests::partitioned_completed_files_share_direct_facts_and_keep_failures_private`|720896 versus1048576|The failing quantity is `spill_buffer_bytes*4`, **not the canonical memory limit**. Accepted `partition_output` subtracts the64KiB shared ID buffer and another64KiB per partition: `((1048576−65536)/4−65536)*4=720896`. The old assertion expects the full1048576 spill allowance. D does not alter that formula.|
|`objects::tests::spilled_candidate_visits_selected_objects_in_graph_order`|The same two selected IDs appear in the opposite order|Accepted test installs order `[ids[2],ids[0]]`, verifies its direct visitor in that order, then expects consumed ownership in `[ids[0],ids[2]]`. Accepted `consume_prevalidated_pages` follows the supplied reachable order. This contradictory expectation predates D; no canonical object is missing in the logged comparison.|
|`objects::tests::direct_admission_honors_every_frozen_count_boundary`|`physical encoding reservation`|Accepted source contains the same2MiB encoding reservation guard and count-boundary fixture. D compile-time guards establish unchanged charged authenticated/prepared-object sizes; canonical/group construction is unchanged. The guard rejection needs baseline reproduction before attributing it to stale tests or an existing implementation limitation.|
|`objects::tests::shared_admission_keeps_every_object_transaction_below_the_frozen_bounds`|`physical encoding reservation`|Same accepted reservation mechanism and fixture. This could expose a real pre-existing count-versus-memory constraint; do not raise the limit or dismiss it merely because other tests pass. Baseline runtime comparison is required.|
|`objects::tests::workspace_delivery_selects_before_admission_and_deduplicates_across_phases`|Expected injected transaction error is absent|The accepted test configures statement1 then calls threaded output construction; failure injection is thread-local in accepted `schema.rs`. Initial-probe/flush ownership and when a statement executes matter. This is source-supported suspicion of an outdated failure-injection assumption, not yet a proven complete root cause.|
|`layerstack::tests::late_direct_initialization_failure_publishes_nothing`|Initialization returns success despite expected injected error|Accepted test sets failure at `candidate.objects.len()+1`, assuming a relationship between objects and SQL statement count. Accepted packed admission executes batched pack and locator INSERTs rather than one statement per object. Exact triggering behavior must be compared with baseline; success here does not test rollback because no expected failure fired.|
|`layerstack::tests::multi_batch_publication_failure_removes_admitted_objects`|19573 objects remain where0 expected|Accepted test injects at `u64::MAX` after admission and demands cleanup of already admitted objects. D does not alter `layerstack.rs` cleanup/fault logic or transaction policy. This is a material unresolved baseline retention/cleanup expectation until reproduced; do not relabel it harmless merely because the higher-level publication failed.|

The exact ObjectId pair in the ordering failure is preserved in the log; it is
not reproduced as a raw-data inventory. The ignored large-spill test was not run
or inferred to pass.

## What D did and did not change

D's changes add source-tagged fixed counters, private hint observation bytes,
post-CAS classification and post-commit diagnostic pack provenance. The actual
charged `PhysicalHints`, authenticated-object and prepared-object size/alignment
checks compile successfully. The diagnostic terminal observation reuses an
already consumed diagnostic byte; it does not add an encoding-group heap vector
or change the reservation formula. File-owner context adds fixed observer state
outside those charged per-object arrays and is disclosed separately.

The accepted `schema.rs`, `layerstack.rs`, `statements.rs` and locator INSERT SQL
are unchanged by D. The two cfg(test) accessors only allow previously invalid
test accesses to `SpillDiskIndex` private fields. They neither change runtime
connection behavior nor weaken an assertion. Removing the unused cfg(test)
`OptionalExtension` import is likewise test compilation cleanup.

These facts make the accepted-baseline comparison an appropriate discriminator;
they do not justify calling a failing suite passed or modifying its limits.

## Baseline comparison requirements

Parent should retain the baseline checkout identity/patch, exact toolchain,
feature flags, full command, test thread setting, log, exit status and cleanup.
Use accepted `eb7050603` plus only the same private test accessor repair. Exclude
all D instrumentation and actual-file source markers. The baseline has six fewer
tests, so counts are expected to differ while the failure-name set can match.

Compare all nine names and the actual failure values/messages, not only the
number9. Matching reservation errors, ordering IDs,720896 spill value, parameter
counts and19573 retained objects would establish those failures predate D under
that execution profile. Differences require investigation before a no-regression
claim. Retain nondeterministic fault/timing evidence instead of forcing an exact
match by rerunning until favorable.

If all failures reproduce, report **“D-specific tests pass; the full library suite
retains nine independently reproduced baseline failures.”** Do not call the full
suite green, waive the original failures, remove tests, or count this as crash/
durability/release qualification. Any existing defect repair belongs to its own
scope and identity rather than silently changing accepted M4.5 during an
unchanged-policy diagnostic.

## Addendum: accepted-baseline comparison completed

The independent accepted-baseline library run now reports **43 passed,9 failed,
1 ignored**. Its compilation paths identify the separate
`layerfs-issue88-diagnostic-control-tests` checkout; the parent retained only the
known cfg(test) accessor repair on accepted source. D reports49 passed because
it adds six passing tests. **The nine failure names match.**

Comparison of the two complete logs gives:

- Seven failures have identical substantive messages/values after accounting
  for shifted source line numbers.
- `late_direct_initialization_failure_publishes_nothing` returns successful
  initialization in both runs. The generated LayerStack/Layer IDs differ; exact
  identifier equality was never a valid expectation for these independent runs.
- `multi_batch_publication_failure_removes_admitted_objects` retains **19572
  objects in the baseline and19573 in D**, both against expected0. The failure
  class reproduces, but the numeric residual does **not** match exactly.

The two logs are `accepted-store-regression-1.log` and`store-regression-1.log`
under the custody directory above. The D log SHA256 is
`9617f8bd310e818e64137b8c1d902bc1af6f4bd2a82476e34c4a6aa96d57068d`.
The accepted-baseline log SHA256 is
`9e40aae8f594b004a9eaa9a5ca4a8150b0c0af270c1cf82906a9dcb74343075b`.

### What explains the publication failure, and what remains uncertain

The common failure mechanism is visible in unchanged accepted code:
`initialize_layerstack` completes earlier admission batches before its final
publication closure (`layerstack.rs:60–116`), injects the`u64::MAX` failure inside
that final closure (`:123`), and returns the error (`:152–167`). The error branch
does not delete prior committed object batches. Rolling back the final
transaction does not undo those earlier transactions. Thus both runs contradict
the test's zero-object cleanup expectation through the same pre-existing path.
This does not establish that the test's cleanup requirement is unnecessary or
that the underlying failure behavior is acceptable for release.

The **one-object difference is not causally attributed to D**. These are not
equal canonical-input populations: `LayerStackId::new()` obtains runtime/random
identity (`ids.rs:97` and`:153–173`); absent an explicit benchmark override, the
namespace seed hashes that new identity (`layerstack.rs:570–577`). The fixture
also creates fresh native files without normalizing their mtimes, and the native
importer incorporates actual `mtime`/`mtime_nsec` (`layerstack.rs:1999–2000`,
`:2095–2096`). Different canonical metadata reuse, inode identities/order,
intermediate structures or final-batch boundaries are consequently plausible
sources of a small retained-object-count difference. The logs do not identify
which object differs or prove one particular explanation. No Store census or
new fixed-seed experiment was run for this review, and no such attribution is
invented.

D changes neither the SQL fault-injection locations nor the cleanup/error branch.
Its charged per-object layouts and batching arithmetic remain guarded, and all
new diagnostic tests pass. This supports the narrower conclusion **“no new
failing test name; all nine failure classes reproduce on accepted source”**,
not complete behavioral equivalence under these unequal native test inputs.

### Research continuation decision

Continuing the **normal-path, fresh-Store, unchanged-policy D research** is
justified with these baseline failures explicitly retained: the D-specific
handoff, source, spill, cohort, race and pack-provenance tests pass; the broader
suite adds no new failure name; and accepted-baseline failures are independently
reproduced. This is not a waiver or a green full-suite result. The normal public
preflight must still pass its own identity, canonical, resource and cleanup
checks before full157 diagnostic execution. Stop on a new failure, a violated
D cohort/provenance gate, or a normal-path cleanup/integrity failure.

Do not alter the accepted transaction policy, fault fixtures, resource limits or
assertions within D merely to make this baseline suite pass. Baseline rollback/
retention semantics and the exact19572/19573 difference remain outside this
diagnostic's qualification claim. No crash/durability/release assurance follows
from proceeding with the research.

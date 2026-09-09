# R24 profile-guided locator statement experiment

User requested one short profile, one project-owned optimization, and an adjacent
candidate/control performance pair. Keep4KiB pages, exact native encoding and
imported dependencies unchanged. Verification/full157 remain deferred.

Profile the archived R23 coalescing binary using macOS sample,2seconds at1ms,
starting about1second after launch. Host measurement wrapper owns the shared
lock. Raw command/output and summary reside in r19-fast/019-profile-coalescing.
This instrumented timing is not an unprofiled performance comparison.

Main consumer sample counts (1373 total):
- publication: 964
- commit: 426
- insert: 514
- native_compression: 165
- ordinary_compression: 38
- guarded_pwrite: 337

These are inclusive stack counts: commit/insert are parts of publication, and
pwrite is part of commit. Do not add them. Native/ordinary compression are separate.
The profile also shows statement-journal copying under locator B-tree insertion.

Selected minimal treatment: allow up to512 rows per locator INSERT only when
Init coalescing is enabled; retain runtime SQLite variable/SQL-length limits.
Existing physical batch already has at most512 globally sorted locators.
Pack statements and other callers retain128-row caps. No additional queue,
canonical/encoded buffer, schema, native encoding or dependency changes.
Hypothesis: fewer statement starts and statement-journal boundaries reduce some
insertion overhead. This does not claim to eliminate B-tree traversal or page writes.

Rejected before implementation: INSERT OR ROLLBACK. Whole-admission failure
semantics would be compatible, but immediate foreign keys still require abort
handling, so omission of statement journaling is not established. See official
[conflict rules](https://www.sqlite.org/lang_conflict.html) and
[statement journal documentation](https://www.sqlite.org/tempfiles.html#statement_journal_files).
Local SQLite source inspection by admission agent supports this caveat but was
not inspection of the exact linked Apple SQLite implementation.

Treatment source and binary custody: r19-fast/build-020-locator-statements.
Control: archived r19-fast/build-015-coalescing/candidate. One adjacent pair first;
retain only a measurable gain. No post-treatment profile or verification required
for rejecting an unfavorable timing. Original source backup r24-admission-before.rs
allows removing only this treatment while preserving all earlier work.

## Rejected result

| Metric |512-row candidate|128-row coalescing control|
|---|---:|---:|
|Init ns|3568710625|3441064459|
|Peak RSS bytes|95584256|92897280|
|Admission commits|137|137|
|Max transaction objects|5582|5632|
|Max transaction bytes|4193036|4192646|
|Database bytes|550973440|551047168|
|User CPU ns|3866376334|3759193666|
|System CPU ns|6209736917|6269199416|

Candidate is 3.71% slower and
peak RSS is 2686976bytes higher in the adjacentpair.
Reject: no measurable win. Restored only objects/admission.rs from its exact
pre-experiment backup; retained R23 coalescing, filter and deferred regression.
Both timings are faster than historical R23 pairs, demonstrating why the adjacent
control matters. Do not claim the candidate improved from the earlier3.95s result.

Release build and both performance commands exited successfully. No correctness
suite, independent verification or full157 ran. No imported libraries, dependency
features, SQLite pages or native encoding changed. No further timing/profile is
needed to reject this treatment. Artifacts and unfavorable result are retained.

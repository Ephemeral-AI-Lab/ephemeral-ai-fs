# Past mistakes to avoid in v0.1.5

> **Status:** Source, issue and commit-history review, 2026-09-09; implementation
> guidance, not a claim that every historical defect remains in current code.

Read with [benchmark_success.md](benchmark_success.md) before the v0.1.5 spec.
Each lesson identifies the failure mechanism, the historical disposition, and a
concrete check for the new implementation. Do not copy historical resource limits
or accepted exceptions into a new contract without checking the current source.

## Evidence boundary

The historical packed baseline is evidence head `cf3a058925c3012fda0fae922dc081766bb8fa99`,
with R26 product `861e388339ef5572659cb16ef0c8febbff0351df`, plus our tiny-family
harness. The final v0.1.4 release is commit
`101fa273d815f3aaedb0e06ba0de7b0777d83def`, with qualified product source
`9cfb4be477116646258ea0621280ed13b1824c6d` and qualified evidence head
`856baab0caf0522db4757cb4dcbfb0d9c43e30d4`. The release adds documentation and
packaging without changing that qualified production code. The #95 Init and #98
Workspace repairs below are qualified, not pending investigations.

The existing [tiny-history baseline](tiny-history-baseline-v1.md) retains its
original source, harness and observations. Do not relabel it as the released
v0.1.4 source or silently replace its numerical comparison gates. Historical
R26, final release and tiny-case evidence are separate source-bound records;
a commit title or closed issue does not establish benchmark qualification.

The review covered the family implementations, retained #47/#49/#68/#71 and
#88/#90/#91 reports, selected patches, and GitHub issue histories including
[#18](https://github.com/Ephemeral-AI-Lab/layerfs/issues/18),
[#48](https://github.com/Ephemeral-AI-Lab/layerfs/issues/48),
[#49](https://github.com/Ephemeral-AI-Lab/layerfs/issues/49),
[#88](https://github.com/Ephemeral-AI-Lab/layerfs/issues/88),
[#90](https://github.com/Ephemeral-AI-Lab/layerfs/issues/90),
[#91](https://github.com/Ephemeral-AI-Lab/layerfs/issues/91), and
[#95](https://github.com/Ephemeral-AI-Lab/layerfs/issues/95), plus the qualified
[#98 repair](https://github.com/Ephemeral-AI-Lab/layerfs/blob/856baab0caf0522db4757cb4dcbfb0d9c43e30d4/docs/roadmap/0.1/0.1.4/issue98/README.md).
No product changes, builds, tests or new benchmark samples were performed for
this review. Historical observations below remain tied to their original sources.

## SQLite queries, object ownership and batches

### 1. Rediscovering a namespace and revalidating immutable bytes at every handoff

**Failure:** Earlier #47 create/Commit work performed 108,006 snapshot database
calls. Even after frontier repairs, separate membership/equality stages and
repeated authentication remained expensive. A memory buffer changing wrappers
was treated like newly untrusted content.

**Repair:** Keep dirty-frontier/final-fact information, coalesce exact membership,
batch positive equality reads, and carry checked immutable ownership through
construction and delivery. Commits `ebb98dbe7` and `a07f053bb` address distinct
parts. One checked-owner observation reduced consumer time from 286.477 to
179.085 ms, but complete Commit only changed 644.928 to 636.195 ms: a subphase win
was not an equal end-to-end win. Fresh persisted/spilled input still authenticates.

**v0.1.5 check:** Count identities, occurrences, hash bytes, equality bytes and
SQL probes. Keep corruption and collision checks at actual trust boundaries;
preserve checked ownership instead of repeating work or removing validation.
Source: [docs/roadmap/0.1/0.1.3/issue47-subsecond-workspace-results.md:294](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue47-subsecond-workspace-results.md#L294).

### 2. Spilling finalized output and immediately reading it back

**Failure:** About 113 MB of selected output was written through staging and
reread on a roughly 100 MiB create, even after its finality was established.

**Repair:** Bounded owned slabs deliver finalized file output to the shared
consumer; provisional structural output retains separate ownership. One retained
observation reduced readback from 113,434,447 to 4,256,258 bytes with equal
canonical candidate/inserted/reused accounting. This did not authorize buffering
an entire workspace in RAM.

**v0.1.5 check:** Follow base, canonical target, FULL alternative and delta buffers
end to end. Move owned data where possible. Charge simultaneously live operands,
queues and scratch; prove failed candidates cannot become published output.
Source: [docs/roadmap/0.1/0.1.3/issue47-subsecond-workspace-results.md:194](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue47-subsecond-workspace-results.md#L194).

### 3. Assuming a requested SQLite PRAGMA actually took effect

**Failure:** Disposable scratch requested `journal_mode=OFF` under SQLite
DEFENSIVE mode, but the effective mode remained DELETE. The large input crossed
an index-spill threshold that the smaller diagnostic missed, exposing rollback
journal costs. Initial namespace100000 elapsed was 12.4625 s.

**Repair:** On a newly owned empty scratch database only, temporarily adjust
DEFENSIVE, set the fixed scratch mode, restore the prior defensive value even
on failure, and check the effective result before data insertion. `fa18ebde8`
made a real correction; the next 9.1714 s observation still required more work.
Customer Store journaling was not weakened.

**v0.1.5 check:** Assert effective settings on the actual linked SQLite library.
Test beyond scratch/spill crossover, include sidecars, and preserve failure
poisoning/restoration. A configuration string is not evidence of runtime behavior.
Sources: [admission-profile-r7.md:3](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/admission-profile-r7.md#L3), [scratch-journal-r8.md:3](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/scratch-journal-r8.md#L3).

### 4. Maintaining redundant indexes for facts the publication owner already knows

**Failure:** Separate available-dependency and seen indexes processed almost the
same approximately 422k-object stream twice. Reducing transaction count did not
remove that work.

**Repair:** Reuse the serialized writer's pack watermark and actual published
locations. Keep bounded exact tracking only where it is still needed, rather
than duplicating a full-stream index. `1658f1b76` improved the selected Init to
6.7616 s; it still lagged the published 2.6032 s reference.

**v0.1.5 check:** A base hint or an encountered object is not a published dependency.
Reuse valid owner facts, but test pending versus admitted bases, rollback,
preexisting counts and cross-owner changes.
Sources: [admission-membership-r9.md:3](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/admission-membership-r9.md#L3), [residual-profile-r10.md:3](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/residual-profile-r10.md#L3).

### 5. Losing an absence proof and querying the same missing IDs again

**Failure:** Prepared objects already known missing received a second locator
query at admission because that proof did not cross the batch boundary.

**Repair:** `e218653f4` carried a publication epoch with the missing batch. Reuse
is valid only under the same exclusive owner and unchanged epoch. Physical
insertion advances the epoch immediately, because the same connection can see it
before SQL COMMIT. Mixed/stale proofs fall back to exact checks.

**v0.1.5 check:** Exercise unchanged-epoch reuse, insertion-before-recheck, late
duplicates, collision/corruption and rollback. Never turn a negative cache into
unconditional permission to skip dependency validation.
Source: [absence-proof-r12.md:3](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/absence-proof-r12.md#L3).

### 6. Treating physical batches, SQL transactions and statement rows as one knob

**Failure:** Preparing 8,191 records together exceeded real ownership budgets
when hint/vector allocations coexisted with codec scratch. After shrinking
physical batches, committing every batch caused excessive SQL commits.

**Repair:** #90 used 512-object physical batches within the existing public
transaction ceilings. R23 later coalesced physical insertions into bounded SQL
cohorts without keeping all released buffers alive. Exploratory Init comparisons
showed about 17.78% lower time and roughly 137/138 rather than 1,155/1,158 admission
commits, but peak RSS increased about 12–14 MB. A later 512-row locator INSERT
experiment was 3.71% slower than its adjacent control and was reverted.

**Later recurrence and repair:** Workspace still used 1,324 SQL admission
transactions for 164,150 inserted objects in the `.venv` diagnostic. #98 connected
Workspace to the existing bounded SQL coalescer and closed the cohort before
staging. Physical batches and strict <8,192-object/<4-MiB transaction bounds
remained unchanged. Adjacent and reversed-order observations reduced Commit by
7.2% and 6.7%, respectively. A shared helper existing in Store did not mean every
public operation used it.
Source: [qualified Workspace measurements](https://github.com/Ephemeral-AI-Lab/layerfs/blob/856baab0caf0522db4757cb4dcbfb0d9c43e30d4/docs/roadmap/0.1/0.1.4/issue98/README.md#adjacent-observations).

**v0.1.5 check:** Trace both Init and Workspace through FILE_DELTA admission.
Preserve cohort handoff to staging, pending-object visibility, exact collision
rejection, and rollback after earlier batches have committed. Measure preparation bytes, transaction objects/bytes, statement
rows, actual commits and process RSS separately. Preserve 511/512/513 and
8,190/8,191 boundary tests, empty-final flush and rollback. Bigger statements are
not automatically faster; unchanged buffer settings do not prove unchanged RSS.
Sources: [docs/roadmap/0.1/0.1.4/issue90-adoption/defect-ledger.md:13](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue90-adoption/defect-ledger.md#L13),
[bounded-commit-r23.md:1](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/bounded-commit-r23.md#L1), [locator-statements-r24.md:41](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/locator-statements-r24.md#L41).

### 7. Committing batches before publication without keeping cleanup ownership

**Failure:** A late publication failure left privately admitted rows behind;
one reproduction retained 19,571 selected objects. Individually committed
batches had no owner responsible through the final retention decision.

**Repair:** #90 retained writer/cleanup ownership through admission and publication,
removed only owner-new private rows/packs on failure, and preserved preexisting
objects, required bases and intentionally retained stages. Cleanup failures
quarantine further writes rather than silently allowing unsafe continuation.

**v0.1.5 check:** Inject failure at real publication boundaries. Assert exact
private-residue counts, base retention, retry-once behavior and cleanup-error
handling. A staged candidate retained for retry is different from a leak.
Source: [docs/roadmap/0.1/0.1.4/issue90-adoption/defect-ledger.md:12](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue90-adoption/defect-ledger.md#L12).

### 8. One transaction still repeated hundreds of megabytes of comparisons

**Failure:** The #95 starting evidence identified duplicate-heavy Init
regressions even with one SQL transaction. Identical-500 changed 110.6 to
682.6 ms; the conflict-comparison timer was 619.2 ms, about 90.7% of the total.
It revisited 24,320 cross-batch occurrences and about 453 MB of canonical comparison
operands. Unique-500 instead spent 679.5 ms in SQL commit work over 128 transactions.

**Measured cause and repair:** The duplicate-heavy short profile was dominated by
canonical authentication and native decompression/reconstruction. #95 retained
successfully authenticated comparison operands across incoming Init batches.
Every reuse hit still requires a fresh SQL locator lookup, equal full physical
location, canonical length and exact bytes. Session/publication epochs and
final-root atomicity remain intact. The 2-MiB reservation comes from the existing
16-MiB allowance, includes capacity and entry overhead, and clears on saturation;
oversized operands bypass retention. It introduces no extra canonical payload
clone and is not enabled for generic Workspace admission.

In adjacent control/candidate observations, identical-500 improved
704.759→123.357 ms and CDC overwrite 740.774→146.270 ms. Identical-500 native
decodes fell 24,883→56 while exact collision checks remained 29,999. These are
selected single-observation pairs, not latency distributions or comparisons
against the published v0.1.3 baseline.

**Remaining costs:** Unique-500's separate profile had 293 commit samples,
including 267 in the page-write syscall `guarded_pwrite_np`, and 160 samples in
native compression. Parent/child samples overlap. This supports page-write-path
and encoding costs, not a proven SSD-latency or B-tree root cause. No safe small
unique-file repair was retained under the encoding, page and resource constraints.
Mixed working sets can exceed bounded reuse; mixed-500 still spent about 209 ms
in storage comparison in the #95 terminal observation. Canonical comparison bytes
and sampled read stacks do not measure physical device bytes.

**v0.1.5 check:** Count distinct bases/groups, repeated occurrences and requested
members separately. Reuse bounded, authenticated reconstruction within valid
ownership; do not decode a whole-file delta separately for each canonical member.
Preserve exact collisions, location/epoch checks, eviction accounting and fresh
persisted-input authentication. Measure locator, extraction, decode, hash, byte
comparison, compression and publication work separately before selecting a repair.
These families time directory Init: the predecessor-bearing Workspace worker cap
is outside their timed path. Blocked producers indicate consumer backpressure;
more workers alone do not remove consumer work.
Source: [qualified #95 diagnosis, pairs and remaining costs](https://github.com/Ephemeral-AI-Lab/layerfs/blob/856baab0caf0522db4757cb4dcbfb0d9c43e30d4/docs/roadmap/0.1/0.1.4/issue95/README.md).

## Encoding, physical storage and budgets

### 9. The codec existed but the real operation never supplied its provenance

**Failure:** Generic builders and fast/serial sinks lost file/span context, so
native payload routing did not reliably receive predecessor hints. Codec tests
could pass while ordinary operations never exercised the intended encoding.

**Repair:** #90 carried scoped file context through actual build/replace sinks,
restoring it before metadata callbacks and after errors. Native schema-7 fencing
also stopped old writers from misinterpreting native-format Stores.

**v0.1.5 check:** Assert candidates, usable bases, attempts, selections and fallback
reasons on actual SDK and full-file save routes. Metadata ropes must not silently
become file payloads. Test old-reader/writer rejection before mutation.
Source: [docs/roadmap/0.1/0.1.4/issue90-adoption/defect-ledger.md:21](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue90-adoption/defect-ledger.md#L21).

### 10. More workers silently disabled every storage-saving attempt

**Failure:** R25 passed content/history verification but grew allocation 46.16%:
318,803,968 versus 218,116,096 bytes. All 86,412 eligible payload objects used FULL;
all 134,997 predecessor-cursor reservations were denied.

**Root cause and repair:** Dividing 1 MiB among multiple producers left each below
576 KiB of correspondence workspace plus 32 KiB headroom. The fix capped
predecessor-bearing plans at one existing worker; pure-new/native Init retained
parallelism. No budget was raised. `56470d418` added the meaningful before/after
check. R26 restored 58,306 PREFIX records and reported 15.37% lower allocation
than its retained control.

**v0.1.5 check:** Prove the fixed 128 KiB design can obtain all simultaneous
reservations at actual worker counts. Include mixed new/rewritten files. A correct
FULL fallback is still a storage failure when every intended delta attempt is
unreachable. Keep encoding-selection and total-allocation assertions together.
Sources: [predecessor-budget-r26.md:3](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/predecessor-budget-r26.md#L3),
[docs/roadmap/0.1/0.1.4/issue91-campaign/terminal-r26/storage/report.md:1](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/terminal-r26/storage/report.md#L1).

### 11. Smaller frames were mistaken for smaller database allocation

**Failure:** Required table/index pages dominated small Stores; some text Stores
occupied 15 x 64-KiB pages with only about 4 KiB of pack BLOB data. More compression
could not remove those root pages. Other runs had allocation beyond logical EOF
whose filesystem cause was not established.

**Disposition:** The newer workstream retained 4-KiB creation with supported old
64-KiB readers. The separate R14 64-KiB control was a diagnostic, not an adopted
reversion. Its prospective contract alone is not proof of a speed improvement.

**v0.1.5 check:** Keep 4-KiB pages fixed; report allocation, logical length, sidecars,
pack data, locator/index pages, slack and retained bases. Do not attribute all
allocation differences to delta size or label unproven preallocation hypotheses
as root causes. No uncounted VACUUM/GC/repacking credit.
Sources: [docs/roadmap/0.1/0.1.4/implementation-progress.md:882](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/implementation-progress.md#L882),
[page-geometry-diagnostic-r14.md:1](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/page-geometry-diagnostic-r14.md#L1).

### 12. Compact or selected reads hid broader reconstruction costs

**Evidence:** #88's selected depth campaign passed 60 reads on five small
single-extent files, while broader historical verification showed +23.19% elapsed
and +26.17% host CPU. Observed decode counts exceeded target-chain length; not all
extra work was attributed. Warm-cache zero device reads did not mean zero decoding.
R26 later had 15.37% smaller storage and 35.27% lower summed public Commit elapsed
against its supplemental control, but historical verification wall was 3.57% higher.
These are different source-bound comparisons, not a pooled speedup.

**v0.1.5 check:** Reuse batched base/group decoding for sibling reads, and measure
requested versus fetched/decoded bytes. Test small range reads, whole files,
Git scans and retained history. Tiny-history success cannot certify OS-cold reads
or worst-case reconstruction. Do not add overlapping phase durations together.
Sources: [docs/roadmap/0.1/0.1.4/issue88-native-analysis/depth-read/published/findings-and-disposition.md:46](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue88-native-analysis/depth-read/published/findings-and-disposition.md#L46),
[docs/roadmap/0.1/0.1.4/issue91-campaign/terminal-r26/storage/report.md:35](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/terminal-r26/storage/report.md#L35).

## FUSE round trips, coherence and lifecycle

### 13. Invalidating mapped pages let stale dirty bytes overwrite an SDK edit

**Failure:** In #49, an acknowledged SDK edit at byte 777 was overwritten by a
later 4,096-byte cached WRITE. A widened 10 ms race window established the ordering;
the application had not modified that byte. Three earlier passing attempts did
not resolve the original failure.

**Repair:** `e6d66d676` used public upstream cache-store notifications for the
changed immutable ranges before invalidation, with bounded buffers/reservation
and failure propagation. Length-changing edits include affected shifted suffixes.
Both mapped/ordinary-write and mapping-only focused checks passed afterward.

**v0.1.5 check:** Read an acknowledged edit through mmap, ordinary pread and the
committed snapshot while writeback competes. Preserve inode/alias coherence across
new decode paths. An invalidation call returning success is not the full proof.
Source: [docs/roadmap/0.1/0.1.3/issue49-live-workspace-ledger.md:351](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue49-live-workspace-ledger.md#L351).

### 14. Removing an OPEN round trip removed the ownership acknowledgement

**Failure:** Fire-and-forget read-only pinning exposed a handle before the backend
confirmed the inode lifetime. Replacement/unlink could make the handle invalid.

**Repair:** `957b40c7b` awaited the normal pin acknowledgement. Its regression
holds the ACK to prove OPEN cannot return, then covers success and NotFound.

**v0.1.5 check:** Classify each round trip by the ownership, visibility or error
boundary it establishes before deleting it. Batch optional base reads; preserve
required handle pin and publication acknowledgements.
Source: [read-only pin fix](https://github.com/Ephemeral-AI-Lab/layerfs/commit/957b40c7b).

### 15. No logical mutation was confused with no lifetime work

**Failure:** After an open-unlinked file survived Commit correctly, its last close
could leave backing retained because the next unchanged-generation fence skipped
retirement. Separately, final owner destruction could release the Branch lease
before dirty projection teardown completed.

**Repair:** `9c4862813` performed last-release retirement at no-op fences;
`03d4914ee` ended/discarded active projections before owner/lease release. Retained
read references stayed protected; cleanup failures did not become success.

**v0.1.5 check:** Exercise close-after-Commit then no-op sync, in-flight reads holding
old bases, final-owner drop, and second-owner reacquisition. Distinguish zero
payload change from pending cleanup, base lifetime and resource release.
Sources: [no-op retirement](https://github.com/Ephemeral-AI-Lab/layerfs/commit/9c4862813),
[owner teardown](https://github.com/Ephemeral-AI-Lab/layerfs/commit/03d4914ee).

### 16. Broad gates blocked the callbacks needed to finish their own operation

**Failure mechanism:** Lifecycle cuts, ordinary callbacks and writeback cannot all
wait behind one saturated admission gate. Blanket write pausing around invalidation
can prevent the dirty-page progress that invalidation itself needs.

**Correction:** #49 kept waiting callback ownership outside the receive loop and
reserved bounded lifecycle progress separately. File-operation cuts replaced
process-count Busy assumptions; handles and commands can span Commit.

**v0.1.5 check:** Do not await base I/O or encoding under broad live-state/cache
locks. Preserve control-progress capacity and error/cancellation propagation.
Test concurrent commands, dirty writeback, SDK edits and Commit rather than only
serial final-state equality.
Sources: [docs/roadmap/0.1/0.1.3/issue49-live-workspace-ledger.md:249](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue49-live-workspace-ledger.md#L249),
[docs/roadmap/0.1/0.1.3/issue49-restart-handoff.md:80](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue49-restart-handoff.md#L80).

### 17. Sequential immutable acquisition made Git expensive; unsafe caching was not the fix

**Evidence:** #68 Git100/500 lifecycle observations moved from 5.827/14.118 s to
corrected final medians 1.852/4.636 s after bounded authenticated sibling/group
acquisition and owner/root-scoped caches. Original 500/1,000 ms targets still
missed. This was one corrected baseline versus three candidate observations,
not a statistical confidence claim.

**Disposition:** 60-second TTLs and recursive/BFS acquisition were removed or
rejected; explicit OPEN and writable FLUSH were preserved. A partial directory
cache cannot prove absence beyond its covered range. Fewer callbacks alone did
not justify consistency or error-delivery changes.

**v0.1.5 check:** Reuse immutable decoded bases under valid authorization and
bounded lifetime. Measure lookup/read/decode round trips, public Git time and
memory together. Preserve writable error delivery and cache coverage semantics.
Source: [docs/roadmap/0.1/0.1.3/issue68-git-optimization-results.md:51](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue68-git-optimization-results.md#L51).

### 18. Commit publication succeeded but checkpoint installation could not finish

**Failure:** #71's 17,682-node checkpoint exceeded a 16,384-record receiver map.
Store publication succeeded while presentation/End failed.

**Repair:** `8d15ebc9b` used bounded metadata scratch with validation and retry,
instead of simply raising the map cap.

**v0.1.5 check:** Include full public status, visible state, checkpoint delivery
and clean End. A stored root alone is not complete user-visible success; test
metadata cardinality beyond the receiver's in-memory threshold.
Source: [docs/roadmap/0.1/0.1.3/issue71-venv-implementation-plan.md:35](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue71-venv-implementation-plan.md#L35).

## Tests, benchmark custody and interpretation

### 19. A green wrapper did not prove that the intended verifier ran

Git cases accidentally entered generic sampled-proof routing. Separately,
`docker cp` changed the prepared Git root's mtime. #68 corrected the proof route,
restored metadata and versioned preparation compatibility on both arms. In the
mapped test, substring marker matching let an error containing “after SDK
boundary” satisfy an “after” progress condition; exact-line matching fixed it.

**v0.1.5 check:** Assert actual proof contents, public routes and exact Created
counts. Use exact markers/structured responses. For the tiny case require all
31 actual historical states, not an outer PASS or codec round-trip alone.
Sources: [docs/roadmap/0.1/0.1.3/issue68-git-optimization-results.md:24](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue68-git-optimization-results.md#L24),
[docs/roadmap/0.1/0.1.3/issue49-live-workspace-ledger.md:355](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.3/issue49-live-workspace-ledger.md#L355).

### 20. Old tests stopped reaching their intended failures after the schema changed

Packed storage changed SQL cardinalities and deferred insertion. Guessed statement
ordinals, obsolete objects(id,bytes) fixtures and two owners held on one test thread
either tested nothing useful, failed setup, or deadlocked. #90 replaced them with
actual publication sentinels, valid packed fixtures, and explicit test ownership.
It kept collision, rollback, selected-read and exact residue assertions.

**v0.1.5 check:** Demonstrate the failure injection reaches the new path. Count
selected-locator queries separately from necessary pack reads. Update fixture
mechanics when format changes; do not delete the invariant because the old test
broke. An opt-in live test that skipped its body is not executed FUSE coverage.
Source: [docs/roadmap/0.1/0.1.4/issue90-adoption/defect-ledger.md:1](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue90-adoption/defect-ledger.md#L1).

### 21. Source drift and execution PASS were mistaken for complete qualification

At the original review, the main checkout's schema and scope lagged newer worktrees. Issue titles
alone miss sequencing changes recorded inside #18. R26 executed 198 performance
cases and 226 routine proofs, yet its original comparison report retained 64
SEVERE elapsed rows, four ineligible Git comparisons and INCOMPLETE status.
Missing generation bindings caused 637 report errors; later derived bindings
were explicitly separate. An image-dependent fixture hash explained some mismatch
but did not prove all historical Git bytes equal. Owner acceptance of a stopping
point did not retroactively pass the original gates.

**Final release disposition:** v0.1.4 retained the qualified #95/#98 repairs.
Its 198 performance executions and 226 routine proofs passed; the declared
optional 600-second proof remained unrun. Final elapsed labels were 49 SEVERE,
95 REVIEW, 31 OBSERVED_INCREASE, 19 NO_INCREASE and four INELIGIBLE. Three absolute
targets still missed and the comparison report remained INCOMPLETE. Owner approval
authorized release with those tradeoffs; it did not turn them into threshold passes.

Full157 allocated storage was 184,582,144 B versus the 218,116,096 B supplemental
storage control, a 15.374% reduction. That control is not the published v0.1.3
performance baseline. Final full157 performance/verification wall observations
were 481.976/613.815 s versus previous 447.247/545.827 s. These unpaired historical
observations do not establish a whole-history speedup or attribute the increase
to one patch. Publication reused existing results without benchmark reruns.
Sources: [final case-by-case closeout](https://github.com/Ephemeral-AI-Lab/layerfs/blob/101fa273d815f3aaedb0e06ba0de7b0777d83def/release-notes/0.1.4/benchmark-closeout.md),
[qualified terminal evidence](https://github.com/Ephemeral-AI-Lab/layerfs/blob/856baab0caf0522db4757cb4dcbfb0d9c43e30d4/docs/roadmap/0.1/0.1.4/issue98/README.md#final-terminal-qualification).

**v0.1.5 check:** Freeze source/product/harness/image/input identities; retain the
exact baseline and raw failures. Distinguish execution, proof, comparability,
performance acceptance and release qualification. No mixing old Exec with new
Commit times, no single-observation p99, no unmeasured background-work credit.
Our tiny baseline is exploratory and already uses packed FULL/PREFIX storage;
it is not a raw-CAS or delta-disabled comparator.
Sources: [#18 history](https://github.com/Ephemeral-AI-Lab/layerfs/issues/18),
[docs/roadmap/0.1/0.1.4/issue91-campaign/terminal-r26/README.md:41](https://github.com/Ephemeral-AI-Lab/layerfs/blob/cf3a058925c3012fda0fae922dc081766bb8fa99/docs/roadmap/0.1/0.1.4/issue91-campaign/terminal-r26/README.md#L41).

## Spill access order and read amplification

### 22. Sequential read-ahead was used for dependency-ordered seeks

**Failure:** Late Workspace admission visits spill records in graph order and can
jump around the spool. The existing read-ahead buffer could reach approximately
896 KiB, repeatedly reading and copying large windows across those seeks. A short
profile placed 1,431 of 1,816 sampled `admit_remaining` stacks in spill
`read_exact`/`File::read`. These samples do not quantify physical device bytes or
a complete elapsed-time decomposition.

**Repair:** #98 capped ordered spill visitation at the existing 64-KiB I/O bound;
sequential ID scanning retained its buffer. Frame order, hints, authentication and
identity were unchanged. With coalescing as the control, the selected adjacent
Commit observation improved 6.266→4.194 s (33.1%). Both repairs together improved
6.671→4.152 s (37.8%) against the original control, with host CPU 8.250→5.715 s.
Peak RSS increased 5.1875 MiB; both paired Stores allocated 218,107,904 B. This
`.venv` pair is separate from full157 storage and is not a universal speed claim.

**v0.1.5 check:** Choose buffering from the actual access pattern of delta bases
and prepared output. Measure seeks, requested/read-ahead bytes and buffer capacity
before changing the buffer. Preserve sequential scanning behavior and test ordered
visitation across the read-ahead boundary. A codec improvement cannot compensate
for repeatedly reading large unused windows from a spool.
Source: [qualified #98 spill diagnosis and pairs](https://github.com/Ephemeral-AI-Lab/layerfs/blob/856baab0caf0522db4757cb4dcbfb0d9c43e30d4/docs/roadmap/0.1/0.1.4/issue98/README.md).

## Checklist for the v0.1.5 spec

- [ ] Name the packed baseline and preserve the fixed 128-KiB / 4-KiB policies.
- [ ] Carry known-base provenance through every real full-file save route.
- [ ] Bound and measure query, decode, comparison and transaction work separately.
- [ ] Preserve qualified Init operand reuse and Workspace SQL coalescing in new admission paths.
- [ ] Prove sibling-member reads reuse bounded group/base reconstruction with exact validation.
- [ ] Measure spill access order and read amplification before changing buffers or worker counts.
- [ ] Prove delta selection remains reachable under aggregate worker/memory limits.
- [ ] Preserve current canonical identity, authentic read boundaries and FULL fallback.
- [ ] Define physical base retention through publication, retry, cancellation and End.
- [ ] Preserve required OPEN, error, mmap/writeback and lifecycle acknowledgements.
- [ ] Make focused tests hit real new-format failure points and meaningful size boundaries.
- [ ] Compare total allocated storage and full public operation costs on identical inputs.
- [ ] Keep tiny iteration cheap, then rerun affected families and required proofs once stable.

The useful lesson is to remove repeated work at its shared cause while preserving
ownership, identity, publication and evidence. A faster codec cannot compensate
for an N+1 read path, an unreachable delta policy, or a lost-update race.

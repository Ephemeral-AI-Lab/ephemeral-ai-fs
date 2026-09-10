# Issue #103 integration progress — stride3 complete

**Current result:** integrated stride3 is **46,202,880 allocated bytes / 53 of 53
original states verified**, after measured public compaction. All 11 historical
access performance cases and all 11 verification runs pass their 15-second
contracts. [Final results, custody, costs and limitations](stride3-integrated-results.md).
Full157/66 MB qualification remains **NOT QUALIFIED**, outside the revised scope.

Scope amended by the user on 2026-09-10: finish integration of the promoted
method and optimize/verify **stride3 only**. Full157 is outside this execution
scope; the earlier 66 MB/full157 requirement remains historical context, not a
claim this stride3 run can qualify.

The dated checkpoints below preserve the implementation chronology and failed
attempts; their pending statements describe those earlier checkpoints. The final
continuation supersedes their stride3 status. No full157, broad #102 campaign,
release, tag or deployment was performed.

## Handoff custody

Worktree: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb`.
Branch: `codex/issue100-40mb-experiments`.
Starting HEAD: `c9d6be0df4cb2b2c9131381bf0d44ab6534e577f`.
The requested branch and dirty work were inspected before editing. No active
LayerFS benchmark/build was found; existing Docker service processes were left
running. The draft compact codec/reference traversal, prototype fixtures and
research were preserved. The complete dirty handoff, tracked diff, file hashes
and #100/#103 issue text are archived at:

`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/handoff-20260910T033600Z/`

The initial `gh issue view --comments` command failed because this installed gh
version requested deprecated Projects Classic fields. Explicit JSON fields
successfully retrieved both issues; this did not block or change product work.

## Native physical-framing slice

New Stores use development schema 10 with the existing 4096-byte SQLite page
policy. Schema 6–9 retains its existing schema definitions and nonpromoting
read/write behavior. The existing explicit schema 7/8 to 9 upgrade still targets
9; it does not silently migrate namespace identities or opt old Stores into 10.
Experimental schema 9304 is not accepted or reused.

Schema-10 SmallContent admission writes product pack version 4: four-byte group
starts, one kind byte, optional full 32-byte base identity, and the existing
checksummed Zstandard frame. The raw length comes from the indexed canonical
length; frame length comes from the selected group boundary. The native reader
restores the existing decoder record in bounded scratch, validates the Zstandard
frame and authenticates the complete original canonical identity. No hash,
checksum, canonical-length index field or dependency check was removed.

This removes exactly 20 framing bytes per SmallContent record at unchanged pack
membership: 12 directory bytes plus 8 redundant record-length bytes. It is not a
measurement of allocated SQLite savings. Pack construction retains the previous
conservative packing cut points and all previous chain/resource limits.

Actual admission, exact-CAS comparison, selected-base cache population, point
reads, demanded batch reads, dependency reads, reopen, and explicit integrity
walks understand pack v4. The parser validates the complete bounded starts
directory (at most 1024 bytes); point reads authenticate their selected records
and dependencies, as before. New pack types are rejected by legacy read paths.

## Tests and retained failures

Commands, run on macOS from the above worktree:

```sh
cargo +1.85.1 test --locked -j2 -p layerfs-content --lib
cargo +1.85.1 test --locked -j2 -p layerfs-layerstack-store --lib
cargo +1.85.1 test --locked -j2 -p layerfs-layerstack-store small --lib
cargo +1.85.1 test --locked -j2 -p layerfs-layerstack-store --test v4
cargo +1.85.1 test --locked -j2 -p layerfs-layerstack-store compact_framing_multiple_groups_and_authenticated_locator_failures --lib
cargo +1.85.1 test --locked -j2 -p layerfs-layerstack-store
```

Logs are under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/`.
These are development correctness tests, not source-isolated benchmark arms or
performance qualification. Benchmark binaries/images have not been switched.

- `content-handoff-tests-1.log`: 54 content library tests pass, including the
  retained compact-wire fixtures; this does not establish public compact
  namespace integration.
- `framing-small-tests-3.log`: 10 pass, one pre-existing fixture-dependent
  diagnostic ignored. Covers chain limits, exact reuse, late CAS, rollback,
  selected-base handoff, cold reopen, upper-range SmallContent, schema 8 upgrade
  and schema 9 nonpromoting reads/writes.
- `framing-public-tests-1.log`: all 8 tests pass. The new public lifecycle test
  imports sizes 0/1/131071/131072/131073/2097153 bytes and a hardlink, asserts
  actual pack-v4 emission, reconnects, writes/Commits, forks/edits, reconnects
  again, verifies retained states and runs canonical storage accounting.
- `framing-negative-tests-5.log`: selected multi-group directory/body corruption
  and incorrect indexed canonical length fail authenticated reads; restored
  bytes read successfully.
- `framing-store-final-tests.log`: final full affected Store invocation passes:
  112 library tests (four pre-existing ignored), eight public v4 tests, one
  legacy v5 staging test, and one compile-fail doc test.

Failures were retained, not overwritten:

1. `framing-build-1.log`: an accounting edit inadvertently touched native-chunk
   code without a Store argument; corrected the edit scope.
2. `framing-tests-1.log`: selected-base cache population recognized only pack
   v3 and dropped compact FULL candidates. The shared version check now accepts
   v4. SQL manifest registration and format-specific test byte offsets were
   updated without removing their assertions.
3. `framing-small-tests-2.log`: a new test called a private constructor; changed
   it to use the existing ObjectBuffer/admission helpers.
4. `framing-lib-tests-4.log`: 111 pass, one new negative fixture fails, four
   existing tests ignored. That fixture changed an unrelated group's boundary
   and incorrectly expected a point read of another group to fail. The corrected
   fixture changes the selected group's start, preserving the original
   point-read authentication contract; the focused retry passes.

During review, compact-record expansion was changed to reserve exactly eight
additional bytes before extraction, avoiding Vec capacity doubling within the
existing scratch/chain budgets.

## Remaining integration and measurement gates

At the first framing checkpoint, the public compact namespace and the other
storage mechanisms were not integrated. The following namespace checkpoint
records subsequent implementation; it still does not complete the selected
product format. Schema 10 remains a development format until the full contract
and candidate are frozen.

The selected offline graph depended substantially on bases from later original
snapshots and on a matched Git pack. Product base selection must be defined over
product-owned information, without a Git/oracle dependency. Do not infer online
allocation from the offline graph. If supported compaction is used, implement
and measure its complete time/resources/transient storage and publication and
recovery behavior before qualification.

After full integration and focused correctness: stride3 (53 direct retained
states, original indices 1,4,...,157) first; explain any size gap and fix evidenced
shared integration causes; then freeze the candidate and run full157 and
historical_access with the existing 15-second contracts. Use runner-owned locks,
source-isolated builds, linked-schema checks and archived arm identities. Freeze
complete allocated Store bytes and identity before same-Store verification.

| History | Integrated allocated bytes | Verified states | Publications | Recorded offline B | Recorded Git B |
| --- | --- | --- | --- | ---: | ---: |
| stride3 | Not measured | Not run | Not run | 54,382,592 | 49,332,224 |
| full157 | Not measured | Not run | Not run | 65,957,888 | 56,373,248 |

Full157 margin to 66,000,000 bytes, save/Commit/read timings, resource usage,
compaction costs and historical_access outcomes are unavailable. These recorded
references are historical storage measurements, not fresh paired speed controls.
#103 remains open; #102 and release qualification remain incomplete.

The framing source diff, dirty-file manifest and development test binaries are
archived at `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/framing-checkpoint/`.
`source.json` describes the exact source snapshot; `binaries.json` records both
SHA-256 identities and the full Rust 1.85.1 toolchain identity. This archive
precedes the final documentation-only result annotation.


## Native namespace checkpoint — continuation 2

Status: **partial product integration; 66 MB target NOT QUALIFIED**. No
repository_history or historical_access measurement has run.

Implemented after the framing checkpoint:

- The existing sorted B+ tree engine now carries inline leaf values. Legacy
  formats use a zero-sized value; compact inode leaves carry the complete inode
  record. The same balancing, path-copying, deferred publication and scratch
  accounting code handles 50–100-entry inode leaves, 64–127-entry branches,
  8-byte inode keys, and compact directory references. Child entry counts are
  explicit instead of inferred from a fixed 64-byte width.
- Authenticated logical inode-value lookup handles separately stored legacy
  values and inline compact values. Filesystem resolution and workspace
  acquisition/reference handling consume this API. The legacy batched lookup
  reuses its prefetched root instead of fetching it again for format dispatch.
- Compact directories use the existing lookup, pagination, traversal, diff,
  insert/remove/rename and deferred-tree owners. Their representation is an
  explicit node flag/magic, not inferred from arbitrary legacy inode hashes.
  Deferred pruning/publication understands direct directory roots without a
  separate directory-state wrapper.
- Namespace roots carry the full 32-byte allocation scope. Runtime tree keys
  are namespace-local 32-byte keys containing the 8-byte serial; comparisons
  across namespace roots also check the complete scope. The product profile is
  distinct from the diagnostic prototype profile. Retained wire fixtures still
  exercise the original diagnostic codec without pretending that their profile
  is a supported product profile.
- Actual public **empty initialization**, ordinary public writes/Commit and
  workspace candidate/checkpoint paths now emit compact namespaces. Snapshot
  readers choose namespace behavior from their actual root, allowing the
  development schema-10 Store to read its still-legacy directory-initialization
  roots. **Directory-source initialization is not promoted yet.**
- New compact metadata magics use the metadata admission stream and inode-leaf
  origin selection. Final reachability excludes temporary legacy record values;
  the compact logical lifecycle proof finds no separate LFS4INO/LFS4DIR objects
  in the retained compact roots' authenticated closure.

### Allocation policy and recovery

`scope_allocator(scope BLOB(32) PRIMARY KEY, highwater INTEGER)` is now part of
schema 10. Scope derives from the initialization seed with a product domain;
ordinary seeds come from the existing new LayerStack identity. Allocation does
not consult Git, paths in the sealed fixture, original-state oracles or future
snapshots.

Reservations acquire the existing Store operation gate and writer, require an
autocommit boundary before admission, and commit under DELETE/FULL before IDs
escape. They then restore MEMORY/OFF and retain EXCLUSIVE locking. Returned
ranges are never rolled back with failed candidate admission/publication.
Overflow/zero requests fail, and allocation in schema 6–9 is refused.

A live workspace reserves one 2^32-wide serial range on its first candidate with
new inodes, before content admission opens a coalesced transaction. A shared
range survives preview/retry/checkpoint construction, so the same live NodeId
has the same canonical identity. Rebuilt/reopened workspaces get a fresh range;
existing canonical nodes retain their previous identity. This explicitly bounds
one workspace's NodeId lifetime below 2^32, with chunked reservation as the
upgrade path for longer lifetimes. Unused serials are deliberately burned;
reservation gaps can differ from the offline globally dense allocation.

The first durable-reservation product test exposed a leftover rollback journal
under EXCLUSIVE locking. SQLite documents that exclusive-mode rollback journals
can remain after commit ([SQLite journal policy](https://www.sqlite.org/pragma.html#pragma_journal_mode)).
The local pinned amalgamation confirms that switching to MEMORY closes the disk
journal handle. The reservation owner retains the exclusive DB lock, verifies
autocommit and a zero/inactive journal header, then removes that owned inactive
journal. A nonzero header or cleanup/restoration error is retained and quarantines
writes. The existing no-extra-sidecar assertion was preserved and now passes.
This is required operation cleanup, not cache pruning.

### Executed checks

Raw logs remain under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/`:

- `namespace-engine-tests-4.log`: all ten selected tree-engine tests pass,
  including a 13,000-inode, level-2 compact tree and a 4,000-name directory;
  updates/deletes, retained roots, inline closure, and scratch limits checked.
- `namespace-lifecycle-tests-2.log`: compact logical filesystem lifecycle passes:
  ordinary write/overwrite, hardlinks, rename, metadata time/mode changes,
  delete/recreate identity, retained content, full authenticated reference closure,
  foreign-scope rejection and corrupted inode-table rejection.
- `namespace-allocator-tests-3.log`: durable disjoint reservations, abandoned
  ranges, concurrent clones, separate scopes, exhaustion, reopen and legacy
  refusal pass. Later full Store suite also includes this check with the Store
  operation gate enabled.
- `namespace-product-tests-2.log`: all eight public Store tests pass, including
  no-op and injected-publication rollback checks, concurrent CAS and exact schema.
- `namespace-focused-suites-1.log`: all content suites pass: 56 library tests and
  67 integration tests. Workspace initially has 61 passes and one test failure:
  its metadata probe still used the separate-record lookup API on a compact
  table. The probe now reads authenticated logical values and preserves its
  metadata-dedup and batched-read assertions.
- `namespace-workspace-suite-3.log`: all 62 workspace library tests pass,
  including preview/Commit identity, checkpoint/reference handling, retained
  reads, mutation boundaries and recovery paths.
- `namespace-store-suite-4.log`: 113 Store library tests pass (four existing
  ignored); eight public tests, one legacy staging test and one compile-fail
  doc test also pass.
- `namespace-counter-tests-5.log`: all 19 namespace-model tests pass after
  accounting for format-dispatch reads and retaining the empty-update fast path.
- `namespace-scope-tests-6.log`: latest focused compact logical lifecycle,
  including foreign-scope snapshot replacement refusal.
- `namespace-checkpoint-build.log`: compilation of affected development test
  artifacts for the checkpoint; not a benchmark-arm build.

Compilation failures, the journal-sidecar failure, and the stale separate-record
probe remain in their numbered logs. No benchmark workload was changed or run.

### Still required before stride3

1. Promote directory-source initialization through the existing bounded producer,
   comparison-reuse, pair-spool and publication owners; avoid admitting separate
   inode records that compact leaves no longer reference.
2. Complete and directly test compact three-way inode reconciliation and snapshot
   replacement, branch/restart behavior, cross-format guards and the remaining
   format-specific corruption/resource boundaries. Current old-format tests do
   not substitute for those compact-format cases.
3. Implement authenticated shared metadata-value groups and the selected bounded
   metadata deltas, whole-file prefix graphs and native chunk slices, including
   a supported product-owned base policy and any explicitly measured compaction.
4. Freeze the complete candidate, run focused public Exec/FUSE checks with the
   prescribed macOS/Linux ownership, then stride3, full157 and historical_access
   in the required order using source-isolated qualified builds and sealed stores.

No checkbox for the complete compact namespace or selected storage format is
closed by this checkpoint. #103, full #102 qualification and release remain open.


### Higher-level reconciliation failure retained

`namespace-reconciliation-tests-1.log` runs the two existing public workspace
reconciliation integration cases against fresh compact empty-initialized
histories. Both fail with `Storage(Core(Unsupported))` at
`create_reconciliation_workspace`, before their resolution/publication assertions.
This is an actual incomplete product path, not a verifier or benchmark failure.
It must be fixed at the native inode traversal/reconciliation owners before
history qualification. The earlier 62 workspace-library passes do not cover it.

The continuation-2 source snapshot and development binaries are archived at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/namespace-checkpoint/`.
`source.json` records the dirty-file hashes over base HEAD
`c9d6be0df4cb2b2c9131381bf0d44ab6534e577f`; `binaries.json` records the test artifacts
from `namespace-checkpoint-build.log`. These are not benchmark arms and do not
replace the required source-isolated host/image builds or linked-schema probes.


## Continuation 3 — native initialization, reconciliation and metadata chains

The previous blockers above are retained as the chronology of failed attempts.
Directory-source initialization now writes compact leaves through the existing
bounded producer/frontier/admission owners and an 81-byte inline pair spool.
The stream starts without an admitted seed object; no temporary LFS4INO/LFS4DIR
objects remain indexed. Allocation uses disjoint durable serial lanes, including
parallel initialization tasks and the cross-task-hardlink fallback.

Compact three-way reconciliation now uses the existing field/metadata/directory/
refcount conflict algebra over typed inode values. Snapshot replacement and full
scope comparisons use the same logical owners. Directory lookups, cursors,
validation and diffs reject mixed child formats.

A real workspace memory-budget regression was fixed in the common frontier
lookup: compact dispatch had returned before path accounting. Original budget
assertions were retained. SmallContent provenance also now recognizes the
representation-level predecessor hint without requiring a nonexistent CDC cursor;
the paired schema-7/schema-10 test retains each format's actual counters.

New schema-10 compact inode leaves now use metadata pack version 5. The physical
group/matcher grammar is reused; metadata groups are at most 16 KiB, chains at
most 16 edges and 128 KiB of canonical closure, records at most 8193 bytes.
Only admitted, chronologically earlier compact leaves supplied by the product's
tree-origin hints can be bases. Every intermediate canonical base is hashed and
validated before replaying its child. Exact CAS and storage closure traversal
use these same reads. Old pack-v1/schema-6–9 rules remain unchanged.

This is the bounded-delta portion of metadata integration. Shared metadata-value
groups and whole-file graphs/native slices are still incomplete. Pack-v5 is a
development format and is not yet a frozen benchmark candidate.

Retained development checks (all macOS, same worktree/HEAD and dirty source):

- `namespace-reconciliation-tests-2.log`: both public workspace reconciliation
  cases pass after the original two failures in `-1.log`.
- `namespace-native-initial-oracle-tests-6.log`: four initialization shapes pass
  against legacy logical content/metadata/hardlink oracles, with exact indexed
  reachability and reopen. The thirteen existing exact legacy-identity tests
  explicitly use schema 9 and retain all original identity assertions.
- `namespace-frontier-budget-tests-8.log`: original tight path-budget check passes.
- `namespace-provenance-tests-10.log`: paired legacy/native provenance passes.
- `namespace-format-guards-tests-12.log`: compact lifecycle/reconciliation 2/2
  and namespace-model 19/19 pass.
- `metadata-chain-check-1.log`: `cargo check -p layerfs-layerstack-store --tests`.
- `metadata-chain-tests-2.log`: two new test fixtures rejected for missing logical
  dependencies; the product closure check was correct and was preserved.
- `metadata-chain-tests-3.log`: `cargo test -p layerfs-layerstack-store metadata_
  -- --nocapture` passes 8/8 selected tests. The two new cases exercise depth/
  canonical closure fallback, reopen, exact CAS, corrupt unused base bytes and
  missing dependencies. Fixtures now publish all their referenced objects.

Next gate: finish shared value groups and whole-file prefix/native slice support,
then focused public lifecycle/FUSE validation and a source-isolated stride3 arm.
Use the sealed fixture's direct transitions 1,4,...,157 and verify all 53 original
oracles through the frozen measured Store. Compare allocated storage with the
historical offline 54,382,592 B and Git53 49,332,224 B references. No full157 or
broad #102 benchmark campaign is scheduled under the revised scope.


`metadata-public-suites-4.log` completes `cargo test -p layerfs-layerstack-store
-p layerfs-workspace`: Store library **117 pass, 4 existing ignored**; public
Store **8 pass**; legacy staging **1 pass**; workspace library **62 pass**;
file-edit integration **12 pass**; reconciliation/recovery integration **2 pass**;
compile-fail documentation **1 pass**. These are development tests, not benchmark
or timing qualification.

The exact dirty sources, binary diff, SHA-256 manifest, compiler identity and
six executed development test binaries are archived at:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/metadata-chain-checkpoint-20260910T062039Z/`.
HEAD remains `c9d6be0df4cb2b2c9131381bf0d44ab6534e577f`; no benchmark-arm
binary was switched. The checkpoint includes documentation as it stood before
this final test-results paragraph was appended.


## Continuation 4 — authenticated shared metadata-value groups

Schema-10 ordinary initialization and Commit now write pooled compact inode
leaves in **pack version 6**. They retain their exact original canonical ObjectId
and indexed canonical length. The physical leaf stores its 44-byte canonical
header followed by `(serial:u64 BE, ordinal:u32 BE)` rows. The existing delta
matcher operates on those physical rows, and the reader expands and authenticates
**every** intermediate canonical leaf before replaying the next child. The
unpooled pack-v5 reader is retained; a pooled delta cannot use an unpooled base.

Product-owned ordinal policy: append contiguous groups from already published
Store state; assign new ordinals in admission order. Values are deduplicated
within a prepared publication (including splits into several packs), between
publications and after reopen. No Git objects, fixture identities, later snapshots
or complete-history oracle are used. Ordinals never renumber published leaves.
The schema's explicit u32 ordinal ceiling rejects exhaustion.

The persistent `metadata_value_groups` catalogue has one row per group, with
first ordinal, count, pack/group locator and a complete 32-byte digest. Each
group contains at most 165 canonical `LFSIVL1` values, only FULL records, in at
most 16 KiB decoded bytes. Pool groups share the metadata pack, retaining the
existing 256 KiB pack bound. The digest uses the product's existing domain-
separated BLAKE3 helper (the offline prototype used SHA-256); the full group,
including count, directory and all records, is authenticated before parsing.
Expanded canonical leaf authentication independently binds the selected values.
There are no per-value CAS rows or permanent per-value lookup indexes.

A disposable macOS SQLite lookup index reuses the existing scratch-file helper:
4 MiB page cache, explicit 4096-byte pages and a 32 MiB file ceiling. It rebuilds
from authenticated published groups, retains at most 131072 indexed value
occurrences, and clears its lookup window when full. Clearing may reduce sharing
on larger histories; it cannot invalidate published ordinals. This is a declared
resource ceiling, not a fixture-specific population assumption. Unpublished
values stay private to their PreparedAdmission; only selected group publication
makes them available to a later preparation. Rollback discards the derived index,
and pool rows follow their owned pack deletion via foreign keys. The index is
not a required Store sidecar; its creation/rebuild time and temporary allocation
must be included in the stride3 cost/resource evidence.

Read bounds are explicit and separate: metadata chains retain the 16-edge /
128 KiB canonical bound; logical pool work is at most 192 KiB, retained pool
values plus conservative map ownership at most 512 KiB, and cold group-work
reservation at most 32 MiB per required chain. Optional metadata pool discovery
has an 8 MiB target and 64 MiB batch reservation, separate from unchanged legacy
hint budgets, and falls back to FULL when exhausted. Actual decoded pool bytes,
group fetches, admitted pool groups/values and lookup-index synchronization time
now have physical receipt counters. These bounds are not 15-second read-performance
proof; historical_access still needs the measured integrated stride3 Store.

This completes the first product implementation of shared groups and pooled
bounded deltas. Whole-file prefix graphs and authenticated native slices are
still pending. Development schema 10 is still being constructed; its new exact
schema includes the pool catalogue. Earlier schema-10 development checkpoints
remain archived evidence and are not current-format benchmark qualification.
Supported schema 6–9 definitions and nonpromoting behavior remain intact.

Retained evidence in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/`:

- `metadata-pool-check-1.log`: compile errors from unsupported unsigned rusqlite
  bindings; corrected to checked/range-limited SQLite integer conversions.
- `metadata-pool-check-2.log`: Store test-target compilation passes.
- `metadata-pool-tests-3.log`: initial pooled metadata selection, chain and
  authentication checks pass 8/8 selected tests.
- `metadata-pool-public-tests-4.log`: compile-time module-path typo, retained.
- `metadata-pool-public-tests-5.log`: public lifecycle 7/8 pass; the exact schema
  test correctly reports the newly added catalogue. Its complete expected schema
  inventory was updated, keeping exact equality and legacy rejection checks.
- `metadata-pool-tests-6.log`: 11/11 selected tests pass. New checks prove one
  value pool across several prepared packs and reopen, catalogue digest/count/
  missing-group rejection, rollback of visible groups, and stale-index disposal.
  A negative case changes an unused base value **and its group digest** and still
  fails full intermediate canonical authentication.
- `metadata-pool-public-suites-7.log`: Store library 120 pass (4 existing ignored),
  public Store 8, legacy staging 1, workspace library 62, file-edit 12,
  reconciliation/recovery 2 and documentation 1 pass: 206 total.
- `metadata-pool-tests-8.log`: 12/12 selected checks pass after adding mixed
  unpooled/pooled history reads and optional pool-budget exhaustion. The public
  fault-injection list now also covers the additional catalogue insertion.

No repository_history run, full157 run, broad #102 campaign or storage claim was
made. Next implementation owner is whole-file content history/native slices;
stride3 remains gated on completing and checking that public path.


Final continuation-4 check: `cargo test -p layerfs-layerstack-store -p layerfs-workspace`
(`metadata-pool-public-suites-9.log`) passes **207 tests**, with four existing
ignored tests: Store library 121, public Store 8, legacy staging 1, workspace 62,
file-edit 12, reconciliation/recovery 2 and documentation 1. Exact dirty sources,
compiler identity, hashes and the six executed development test binaries were
archived before another build at:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/metadata-pool-checkpoint-20260910T064447Z`. The manifest predates only this documentation paragraph.


## Continuation 5 — product whole-file graphs and explicit compaction

The first implementation of the full selected product method is present. Its
remaining gates are runner integration, source-isolated qualified builds, live
Exec/FUSE checks, stride3 measurement/53-state oracle verification and scoped
historical_access. No storage benchmark has run yet.

`LayerStackStore::compact_into(destination, CompactionOptions)` is a supported
public operation, also exported through the SDK types and the product binary
`layerfs-store-compact SOURCE DESTINATION [TEMPORARY_BYTE_LIMIT]`. It creates a
self-contained replacement Store at a new destination while retaining the source
as the pre-compaction image. Continue ordinary writes against the destination.
The operation never overwrites an existing destination. Public initialization,
writes and Commit already use compact namespaces/shared metadata; explicit
compaction is required to select the complete whole-file content graph.

Selection is product-owned. It inventories and authenticates canonical objects
already in the Store, discovers regular-file roots from stored inode records,
and builds whole-file owners for 128 KiB through 2 MiB files. It uses the existing
eight-min-hash content signature, a bounded-cache disposable SQLite index,
largest-first ordering, and at most four already-selected same-role candidates
per object. Candidate ranking is shared-signature count then full ObjectId;
selection minimizes actual encoded record size and retains the first tie.
Every selected prefix must beat FULL including its 32-byte base ID and obey
50-edge / 64 MiB canonical / 64 MiB encoded closure bounds. Later *original*
checkpoints may supply bases only because they are already published at the
explicit compaction call. There is no runtime Git, fixture, path-name heuristic,
skipped-snapshot replay or future-snapshot oracle.

The content container uses the selected `LFCNT1`/107 flat framing: four-byte
record starts, at most 256 records and 4 MiB per pack. SmallContent retains its
original canonical identity; large physical owners use `LFSWFL1`; native slices
use the original native chunk ID plus an authenticated owner/offset/length.
Only complete original chunks with proved owner-byte equality become slices.
Uncovered chunks, including data from files above the owner cap, retain FULL
representations. Slice extent and owner-role checks precede materialization;
full owner/base and reconstructed chunk identities are authenticated.

The existing pinned Zstandard implementation now has an explicit whole-owner
profile, reusing its reset, frame validation, checksums and static contexts.
Small/native codec contracts are retained. Whole encoding has an 8 MiB static
workspace. Reads walk at most 51 identities/locations, then refetch and replay
one frame at a time rather than retaining a 64 MiB encoded closure. The new
format has a declared 16 MiB active-read scratch ceiling including a 4 MiB
per-wave owner cache, separate from caller-owned canonical result batches.
Encoded I/O counts include the second pass. These format/work bounds are not
an RSS cap or a 15-second historical-read qualification.

Compaction holds the source operation gate, works in private copies using the
existing disposable SQLite configuration, and leaves source bytes unchanged.
The default temporary budget is 4,294,967,296 bytes, divided into page-limited
work/output shares with a reserve for SQLite temporary work; scratch page caches
remain bounded. Each physical pack publication is at most 4 MiB. The operation
reclaims unreferenced old packs in the private copy and uses VACUUM INTO to
produce the selected 4096-byte layout, including from a supported 65536-byte
source. It verifies every original canonical object's exact bytes, every added
owner, complete logical-record digests, object cardinality and reachable closure
before publication. The final file is synced and published with an atomic
no-clobber hard link. Cleanup and parent-directory sync outcomes are explicit;
a post-publication sync failure reports `published=true` rather than pretending
nothing happened.

The public receipt includes source/final allocated bytes, final apparent bytes,
named temporary-file peak, verified counts, FULL/prefix/slice counts, graph
bounds, candidate trials and every compaction phase plus total time. The stride3
runner must additionally sample process resources and SQLite transient sort
storage; named-file observations alone are not a complete temporary-storage/RSS
measurement. Pre-compaction Source storage is separate retained evidence, not a
required base or sidecar of the compacted Store. The final measured Store will
be frozen before the independent 53-state oracle verification.

Retained checks in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/`:

- `whole-content-check-1.log`, `whole-content-check-2.log`: codec/read-path
  development compilation checks.
- `whole-content-tests-3.log`: four new format/read tests pass (five selected
  tests total), including 128 KiB/2 MiB boundaries, native slices, exact CAS,
  accepted/rejected depth and canonical closure, corrupt unused bases,
  cycles, frame checksums, missing/wrong-role owners and invalid directories.
- `compaction-check-4.log`: a Rust guard-dereference compile error, retained.
- `compaction-public-tests-5.log` and `compaction-vacuum-probe-6.log`: retained
  failure at VACUUM output creation. The linked Apple SQLite 3.51.0 rejects even
  an existing zero-byte output; the independent Python SQLite 3.51.2 probe accepts
  it. System SQLite also passes an absent destination. The product now atomically
  reserves a private directory and supplies an absent path within it. No SQLite
  library was switched to obtain a pass.
- `compaction-public-tests-7.log`: the first full public compaction lifecycle
  passes, verifying 81 original objects and six new owners, every retained test
  state, hardlinks and subsequent writes/fork/reopen. The synthetic allocation
  in that log is development evidence, not a repository_history result.
- `compaction-public-tests-8.log`: 4/4 public compaction cases pass, adding repeated
  compaction, writes that remain large, failure boundaries before/after publication
  and 64 KiB-source to 4 KiB-output layout.
- `whole-compaction-focused-suites-9.log`: `cargo test -p layerfs-content
  -p layerfs-layerstack-store -p layerfs-workspace` passes **342 tests**, four
  existing ignored. This includes six public compaction tests, with corrupt-input
  rejection and an actual SQLite quota failure during the private rewrite.
- `compaction-sdk-check-10.log`: `cargo check -p layerfs-sdk` passes.
- `compaction-cli-help-10.log`: the built product compactor's command entry point
  responds successfully to `--help`.

All these are development checks. The benchmark runner is unchanged at this
checkpoint. Its compaction phase needs a prospective committed contract, linked
schema checks and binary custody before collecting stride3. Full157 and the broad
#102 campaign remain outside the revised execution scope.


Continuation-5 custody: exact dirty sources, compiler, linked library paths, hashes,
17 executed development test binaries and the product compactor command are
archived at `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/whole-compaction-checkpoint-20260910T080549Z`.
The manifest predates only this final documentation paragraph. HEAD remains
`c9d6be0df4cb2b2c9131381bf0d44ab6534e577f`; source remains uncommitted and preserved.


## Continuation 6 — prospective runner integration

Product implementation and the prospective stride3/compaction contract were
committed as `948068a48`; frozen Store preimage retention was committed as
`4b6943623`, before harness edits. The original handoff and all intermediate
source/test archives remain preserved.

The harness now exposes a correctness-only live Exec/FUSE smoke, a linked
integrated-format probe, the public compaction phase, dedicated measured Store
freezing, and explicit verification Store selection. The performance Store's
closed image is retained before verification forks; historical access binds that
same byte image. New historical cases map original65 ->67 and original57 ->58
with unchanged paths/ranges and the original 15-second contracts. Original64's
pnpm-lock.yaml is too short for the retained 6421-byte range. Expected mapped
outputs are derived only from original sealed fixture bytes/oracles.

Compaction has operation-local process resource receipts, separate process wall
/time output, and a 100-ms open-file allocation trace that observes unlinked
SQLite files through stat on the current process's /dev/fd. It never duplicates
or closes SQLite descriptors. Named/FD sampling maxima remain labeled sampled;
trace artifacts live outside the measured Store directory. Builds archive host
and compactor binaries plus identity, probe the actual schema/namespace/pool/
content format, and retain Linux daemon/FUSE/workload binaries from the image.

The 51 Python runner tests and Rust1.85.1 harness checks pass in
`integrated-runner-tests-7.log` and `integrated-harness-check-7.log`. Initial
harness compile errors are retained in the preceding numbered logs. Docker was
stopped and was started for the required Linux roles. Qualified builds and the
live smoke are next; stride3 remains unmeasured.


### Qualified build and first live-smoke attempt

Commit `27677bbae7f3431a1ba1b8f8b163fb688982c83e` passed the qualified host
build and linked integrated-format probe: schema10, compact namespace, metadata
pool, content107 and system SQLite3.51.0. Source seal:
`1a638cd29b888a709176001feaa8a5517c485e193cb123e671fb494636ddc458`;
host SHA256: `0b4430c1079e9912c90b1c9f8bd078d61da8472baf7551c8cdfbf342f67ded55`.
Host/compactor copies and identities are retained in external evidence
`qualified-candidate-27677bbae/`; the runner also archives them and the matching
Linux image binaries. Image `layerfs-bench-infra:1a638cd29b888a70` built successfully.

`live-integration-1/` failed before filesystem reads/writes: the new harness
script builder emitted adjacent shell separators. The daemon execution receipt
was valid (zero Docker-engine calls), and cleanup passed. The shared builder was
fixed, retaining all checks; a shell-parse test covers both standalone checks and
checks followed by mutation. `integration-script-tests-8.log` passes. Product
code/seal is unchanged. Rebuild the qualified host/image pair for this harness
fix before retrying. No stride3 or other storage workload was run.


## Continuation 7 — qualified stride3 result

Measured candidate `80bc489281735c889a4d62ed576135586be3365b` passes the linked
schema10/integrated-format probe and `live-integration-2`. The first stride3
producer performs 53 direct public transitions at original indices1,4,…,157:
53 Created, zero UpToDate and zero presentation failures. Public compaction
reduces complete allocated Store storage from65,056,768 to **46,202,880 bytes**.
It takes339.132858 seconds, reaches135,069,696-byte peak RSS and observes
281,907,200-byte temporary allocation; source preservation, publication, sync
and cleanup all pass. The destination is self-contained.

After freezing its allocation and identity, **all53 states pass** original content
and metadata oracles through that measured product Store:306,861 path-states and
1,676,767,835 logical bytes. A retained byte-identical preimage makes verification
fork growth explicit and supplies historical-access copies. All11 performance
cases and their11 verification runs pass the original15-second contracts; there
are no read-limit failures. Cold range access still decodes17,029,550 bytes for
6,421 requested bytes, a material tradeoff carried into the final report.

The Store is8,179,712 bytes smaller than the historical offline53 reference and
3,129,344 bytes smaller than recorded Git53. Identical SmallContent/whole-owner
IDs establish that the dominant content reduction comes from physical encoding,
with5,582 more Small prefixes and6 more whole prefixes. SQL/index/pool allocation
is fully charged. These are storage comparisons, not fresh paired speed results.

The original verification launch failed during Docker tag lookup before Store
opening (`stride3-verification-1.log`); exact producer-image identity retry passed
without rerunning the storage workload. All failed attempts remain retained.
The final seal audit passes291 history artifacts,22 historical-access manifests/
completions,53 original oracle hashes and the frozen Store/build identities.
See [final report](stride3-integrated-results.md), [result JSON](stride3-integrated-results.json)
and [seal audit](stride3-seal-audit.json) for exact evidence and commands.

The user-revised integration/stride3 scope is complete. #103 stays open for its
original full157 final-storage and #102 handoff obligations. No full157 or broad
#102 campaign was run; the66MB/full157 result remains NOT QUALIFIED.

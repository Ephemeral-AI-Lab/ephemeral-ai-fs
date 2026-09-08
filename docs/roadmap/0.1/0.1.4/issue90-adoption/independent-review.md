# Issue 90 independent correctness review

Review date: 2026-09-09. This is source review, not a test receipt or phase-2 performance qualification. At review, the worktree was `layerfs-issue90-storage-adoption`, based on HEAD `73f619d42e7dc6955413eb5552e804ea344f755b`, with adoption edits and the issue-71 integration still uncommitted. The final candidate must record its own commit/tree and validation receipts. No build, benchmark, Store open, or measurement was performed by this reviewer.

The reviewer independently examined Store compatibility, initialization, admission ownership, rollback, bounded batching, relevant fixtures, and SDK live-gate activation. The reviewer authored the small FILE-context scopes and associated test in `layerfs-content/src/filesystem/apply.rs`; those functions require the root agent's separate review and are not represented as independently approved here.

## Disposition

No unresolved implementation correctness defect was identified in the reviewed final cleanup/session, format fence, or FILE-sink changes. Earlier concrete findings below were repaired in the inspected source. This disposition is conditional on actual focused and final correctness gates passing on the recorded candidate. It does not close issue 90, approve release, or qualify time performance.

Two validation/documentation obligations remain explicit: the integration receipt must prove which initialization route ran, and an old-binary execution must be distinguished from the unit test of its frozen version check. Root owns the final receipts and disposition.

## Ownership, publication, and cleanup

`AdmissionSession` acquires the existing Store operation gate before missing-object preparation and records the maximum existing pack ID. The guard travels through `CheckedOutputAdmission`, `MissingBatch`, and `PreparedAdmission`; intermediate commits do not release or retain the session. Existing selected rows are not relocated, and pack IDs are immutable. Consequently, packs above the captured boundary belong exclusively to this admission while the permit is held. Another writer cannot retain a dependency on those packs during a failing admission.

All production mutators examined use that operation gate: object publication, branch creation, layer addition, Workspace publication, and stage discard. Readers use the connection mutex but do not acquire the operation gate. Holding the operation permit through candidate construction therefore allows authenticated reads without reacquiring the same permit. Native and S1 preparation occur within the session, closing the race where a prepared candidate might otherwise select another failed owner's physical base.

Rollback selects at most 512 new object IDs with a keyset cursor, deletes them transactionally in bounded pages, then removes at most 512 owned packs per transaction. It preserves every preexisting pack and selected object. It does not rewrite or repack prior data, promise file truncation, or add durability guarantees. Returned pages can remain on SQLite's freelist, as the repaired D02 test explicitly permits.

Session states distinguish active, retained, and aborted. Aborted sessions cannot be revived by `retain`, and new preparation/admission/publication checks active state. This repairs the earlier risk of continuing a mutable admission whose prior batches had already been removed.

For Workspace publication, the session survives `admit_remaining` into `stage_workspace_root`. A successfully committed stage retains its objects before branch publication. A later head conflict or publication failure therefore preserves the intentionally retained stage. The former nested operation-gate acquisition was removed; the session remains in scope through the branch transaction. No nested-gate deadlock was found in the normal Workspace flow examined.

Ordinary preparation/publication/producer failures use explicit `resolve` wrappers so cleanup errors can be combined with the initiating failure. Abandoned tokens, unwinding, and some early Workspace validation exits use Drop cleanup. A cleanup failure quarantines subsequent writers and queued operation entrants while leaving preexisting authenticated reads available; the test injects an actual DELETE trigger failure and checks these consequences. Drop cannot return a combined error, so it logs that failure and quarantines writes. Quarantine is a property of the current Store owner, not an on-disk recovery marker or a crash-recovery promise.

The concurrency test verifies an actual waiter at the operation gate, aborts an owner with a committed private PREFIX, then lets the waiting owner admit the same canonical target. It verifies preexisting base/retained target reads and exact final pack count. The two-encoding CAS test was adapted to prepare FULL and PREFIX under one shared session, preserving both publication orders and canonical equality checks; it does not pretend two independent public owners prepare concurrently after the serialization change.

The admission token now holds mutation ownership until completed or dropped. Documentation must state this lifecycle: a caller must not keep a token while synchronously requesting another Store mutation on the same thread. This is the existing non-reentrant gate extended to the operation lifetime. Contention and foreground delay caused by that serialization are phase-2 questions.

## Compatibility fence

Fresh Stores use version 7. Version 6 and version 7 retain the same relational layout, validated against their registered schema definitions. Connect performs the supported-version check using a read-only connection before writable connection configuration. Legacy version 6 additionally rejects nonlegacy pack-version headers using a bounded-result SQL scan. Unsupported versions and research-native-under-6 Stores are not migrated, recompressed, or rewritten.

Admission consults the Store's recorded format capability. Supported version-6 Stores continue to read and write legacy encodings without version promotion; fresh version-7 Stores can hold native file payloads alongside legacy metadata/structure. Native readers retain legacy FULL support, required legacy/native dependency rules, and canonical authentication. S1/legacy DELTA code remains live.

The new compatibility fixtures check version-6 writes remain legacy, version-7 native admission and reopen/write/dedup, and unchanged bytes/absence of sidecars after unsupported-header rejection. `legacy_open_contract_rejects_native_store_before_writer_configuration` tests the old exact-version predicate through the current verifier; it is not execution of an old binary. Final reporting must preserve that distinction. The existing native reader fixture now reopens its disposable Store and verifies a native PREFIX over a legacy FULL base, as well as the deliberately invalid dependency cases.

## Initialization and ingestion

Issue 71's shared construction path changed `NativeImport` from separate sinks to one sink used by regular files and metadata. An unconditional FILE marker in the sink would therefore be wrong. The repaired `build_checked_file` scopes FILE context only around regular-file construction and restores the prior context on both success and Result failure. Both `FinalizedOutputWriter` and `InitializationDirectAdmissionWriter` transport that context and the actual span into authenticated hints. The latter was a concrete additional gap found by this review and is now repaired.

Generic metadata ropes remain unmarked even when their user bytes resemble an internal magic prefix. Symlinks use their existing canonical builder. Workspace capture/completion already enables regular-file context, preserves selected authenticated hints, and transports predecessor/span information through the shared publisher. The separate public `apply_changes` gap was repaired at content-layer regular-file build/replace boundaries, with independent review of that authored change delegated to root.

The FILE-scope fixture explicitly checks one marked regular payload, its span, an unmarked metadata payload, and restoration after an incomplete read. The Directory fixture now separates the fast nested-file case from the symlink/hard-link fallback case, authenticates the expected portable metadata root, checks file bytes and symlink/hard-link semantics after reopen, and requires native admissions. The earlier fixture accidentally placed a root symlink in both arms and therefore exercised fallback twice; that defect was corrected. Root should additionally record/assert the initialization receipt's source-pass count (fast: one; attempted frontier plus fallback: two), making route selection a checked outcome rather than source inference.

## Bounds and test repairs

- D03 retains both historical 8190/8191 input cases and adds 511/512/513. Actual batch formation uses the smaller 512-object cap; the public 8191 value remains a ceiling. The physical 2 MiB and canonical/data 6 MiB ownership guards remain enabled.
- D05 retains the original small-payload case and adds incompressible 1000-byte payloads to exercise byte pressure. No bound is raised.
- D04 counts shared and per-partition spill-ID reservations explicitly. It preserves exact membership, canonical equivalence, and failure privacy instead of treating spill-buffer bytes as the canonical memory limit.
- D06 retains the checked graph order and exact selected subset; consumption now expects that same child-first order.
- D01/D07 and stale integration fault tests target real transaction boundaries rather than estimating SQL cardinality from object count. The expected failure predicates remain errors.
- D08 uses SQLite's statement iterator, preserving manifest membership, headers, parameters, and schema preparation checks while ignoring comment semicolons correctly.
- D09 admits valid packed fixtures, then introduces unrelated selected-identity corruption. After its cache-retention deletion it restores the real dependency before admitting the large structural fixture, so that branch exercises cache behavior instead of failing dependency admission.
- Additional v4/v5 integration fixtures now distinguish unsupported old-format rejection from supported legacy-v6 staging, use the actual selected-pack layout, and retain publication/CAS/integrity checks.

## Validation boundary and source custody

All execution results remain pending root's receipts in this review. In particular, a normal successful workspace test command does not execute `live_fuse` or `live_docker`: those tests return early unless their explicit activation environment variables are present. The ignored large-spill lane also requires an explicit run or documented valid equivalent. None is counted passed here.

The source SHA-256 values below identify the reviewed files before final sealing; subsequent source changes require reconciliation rather than retroactive attribution:

| File under `crates/layerfs-layerstack-store/src/` | SHA-256 |
|---|---|
| `schema.rs` | `f28e60bbc76c311928375c8cd50cc79a680b6eb0574540f3e0690011a5c6f8b5` |
| `objects.rs` | `74ec88c83d8ddcfbe1eaac652a7709497f30149c2843f677010da2ddebfd0b89` |
| `objects/admission.rs` | `8c7a4b3b2405e452d547676a68bb73fb858478632fd434818e57fa84699b538e` |
| `workspace.rs` | `70d9ab0f2a9d1f9d3ae51ac8c25ff7181c97da958e50efef79b6fe96464c886d` |
| `layerstack.rs` | `e74be911af47cf0862ebe25606d545b88caf3dac34e470d8ab227dc14bb67ef1` |
| `objects/ingestion_tests.rs` | `ecd5c14aaf24250214ae2df8359a24c996ca89e3dfd3ecdcf200e20466b8f18e` |
| `schema/compatibility.rs` | `4fee691c0e1715c11480aa9c9a7f98f4a0e907e435558035da913160628f58c6` |
| `objects/admission/native_tests.rs` | `1625ecbfea1279b912dd29f7e6fa935bbf90f4e610c571a2458e19b31d00c7d1` |
| `objects/read/native_tests.rs` | `8c492371da934ee4246415529dc09b88c643874bc0292f4ad67b9236d0d2109a` |

Historical issue-88 allocated-byte, read-cost, CPU, write-observation, and depth-read findings remain evidence of their original producers. This review supplies no new measurement and does not transfer those numbers to the repaired candidate.

## Incremental review: candidate c095fa9b1

The reviewer inspected the changes after `c6e2d5901315a1a399a380b71d1d292f189d2200` through `c095fa9b19306f459a3546584c1658e34ebfb19f`, plus the route test and the older-code probe source/receipts. No new correctness finding was identified. Root owns execution; the reviewer read existing receipts and did not run any build or test.

**Correction to the earlier review recommendation:** asserting two source passes for the symlink fallback was incorrect. The frontier's early `None` does not propagate its metadata observations into that receipt; both fixtures correctly report one import source pass. The failed assertion in `focused-03/output.log` is preserved (one failed selected ingestion test). The replacement test in `layerstack/ingestion_tests.rs` invokes the actual direct initializer on the exact fixture, asserts `Some` for the fast arm and `None` for fallback, drops the unpublished result, checks zero object residue, and then performs public initialization/reopen. It takes the physical-counter baseline after that explicit routing check so prior test preparation cannot satisfy the public import's native-admission assertion. `focused-04/output.log` records the corrected test as one PASS, zero failures. This is a stronger direct routing proof than the withdrawn receipt inference.

Additional stale fixture A08, `durable_batches_hash_unique_rows_once_and_move_on_last_use`, now creates valid packed objects through shared authenticated admission and introduces unrelated corruption afterwards by altering its selected locator. Exact unique-hash and clone-byte assertions remain; singleton lookup still requires exactly one locator point query while allowing the separate packed-payload SQL. Missing singleton and mixed-batch reads assert their distinct existing error contracts. The intermediate `durable-fixture-01` failed because the singleton path was incorrectly expected to return the batch cardinality error; that failed receipt remains preserved. `prefinal-01/output.log` records the repaired selected test as one PASS, zero failures.

The SQL-shape fixture now recognizes actual selected-locator and pack INSERTs, requires real membership probes, and rejects point-read/RETURNING behavior. Its former host-parallelism early return was removed. Additional test-only lifetime fixes drop unfinished direct handoffs before filesystem cleanup. The remaining diagnostic late-CAS fixture now shares a session for two prepared cohorts, preserving its late canonical recheck without deadlocking on a second non-reentrant permit. The new `with_session` constructor is private; normal construction still obtains a new operation permit.

Clippy-related product changes do not alter their respective contracts: the final optional read-budget borrow is consumed after its last use, the spill visitor uses a concrete generic callback with the same arguments/results, and diagnostic receipt initialization is explicit. The benchmark changes examined are formatting and removal of two redundant borrows. No workload, timing boundary, public operation, or native encoding policy change was found there. `Dockerfile.layerfs` broadens the FUSE library correctness command from the `live_runtime` name filter to all library tests, bringing the issue-71 checkpoint tests into that lane; image execution remains pending its own receipt.

The old-code verification obligation now has an actual execution receipt. The reviewer read `old-binary-01/output.log`, `result.json`, `command.json`, and the probe's Cargo manifest and source. Exit status is zero. The probe links the candidate Store crate and the actual older Store crate from `layerfs-issue88-combined-control` into one executable. It creates a fresh version-7 Empty Store, verifies the older crate rejects connect with unchanged bytes, reopens with the candidate, and alternates old/new Empty initialization writes on a version-6 Store. It also checks no journal/WAL/SHM residue remains. This establishes the real old-code open/write boundary, supplementing the earlier frozen-predicate unit test. It is not a run of an archived released executable or a native-file-payload workload; native payload/authentication and mixed dependency behavior remain covered by the focused Store tests.

The old worktree's HEAD observed during review is `bfbc46c11dbb4d8dd949b54845e222cf42cd822f`. The probe uses its own Cargo lockfile, SHA-256 `5632033d2d88a9bb101b3ebcf26d3d7f6b04bc347092fda048e9f68e28c0c854`; it is not the candidate workspace lock. The old-code execution log SHA-256 is `3f3e8f2dd3d52e09d55d7468cb8cd8773e2d0af3242fb2dead4ae50773d26a6f`. Root must retain both crate source identities and the probe build identity in the final handoff.

Updated reviewed-source SHA-256 values:

| File | SHA-256 |
|---|---|
| `crates/layerfs-layerstack-store/src/objects.rs` | `99931eacf0e190b41deaabc96ec17356b83064ee74ac88cf2e511afae475d06e` |
| `crates/layerfs-layerstack-store/src/layerstack.rs` | `1b4a6f2cfb1755422798e4333b247463da7275636e890641c0bf6d6b25623f9c` |
| `crates/layerfs-layerstack-store/src/objects/read.rs` | `a52ec0ffe0d14991ca5c3d5e959b030810c49551f20484e1e4a0ebdec1c05724` |
| `crates/layerfs-layerstack-store/src/objects/spill.rs` | `fffbe32e6a83764a036b3e097856b1369755afe5caac56459a2f563334c3e7d8` |
| `crates/layerfs-layerstack-store/src/objects/diagnostic.rs` | `363fa4d6e5648890ef4ef8ccb62c25b0498dc82965eaa5c87d4a97408e9a3c58` |
| `crates/layerfs-layerstack-store/src/layerstack/ingestion_tests.rs` | `bbf476fbb2a77f2027aa56db98bfdc6be6deaa94a45a984c9f0dc602cc56ebc0` |
| `benchmark/fs-bench-pro/Dockerfile.layerfs` | `e05eba21100c134e62502ff81242e03d65214d4e2f16cee4433469fac201c31e` |

Full workspace, complete Store, explicitly ignored large-spill, live FUSE, and container-runtime results remain pending final root receipts at this addendum. Earlier focused passes are not silently relabeled as exact-final-candidate full-suite proof.

## Final receipt audit and closure disposition

The final audit supersedes the earlier pending-validation dispositions. The reviewer independently read the final raw logs, commands, result JSON, per-run source records, expanded compatibility-probe source, and `final-identity.json` under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue90-runs`. All listed final runs record the same clean candidate:

- Commit: `593f4ad018bf34b3f180baf66e1ae5cf40c36647`.
- Tree: `755ad78b2fa090d5afda6c27a1086dd628b3ec91`.
- Source seal: `55c0f273b8ffb5612a0c907f4c4692084ef006e740c917bf34d82b07a3c315d7`.
- Product seal: `e279fce6024490e04833038dd834b21e2f5f31077730c9e255477eb648d2804c`.
- Dirty patch: empty, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The only subsequent product-area diff examined after the prior review is A09's public Workspace **test** repair in `crates/layerfs-workspace/tests/file_edit.rs`. It replaces ineffective object-count-derived publication fault ordinals with the existing `u64::MAX - 1` / `u64::MAX - 2` sentinels. It strengthens residue assertions: construction/admission failure leaves exactly the baseline object count; failure after durable staging retains exactly the successful candidate's inserted objects. Branch and other metadata counts remain unchanged, and the existing retry checks remain. No production assertion was weakened or encoding policy altered by that change.

| Final receipt | Independently verified outcome |
|---|---|
| `workspace-build-final-01` | Workspace build exit 0. |
| `workspace-final-02` | Parsed 37 native test-executable summaries: 389 reported PASS, zero FAIL, one ignored. Warm execution reports 88 seconds and four bounded jobs. The Store executable contributes 88 PASS, zero FAIL, one ignored, with zero filtered tests. |
| `large-spill-final-01` | Exact named ignored test explicitly activated using `--exact --ignored`: one PASS, zero FAIL. Its source verifies four files totaling 100,004,100 bytes, identical canonical root/object count, and every retained object's bytes after fresh Store reopen. |
| `runtime-check-final-02` | Linux daemon five PASS; selected proxy completion test one PASS; all FUSE library tests 31 PASS, including checkpoint validation, retry, partial installation, and streaming beyond the former 16,384-node cap. Docker target build exits 0. |
| `live-final-02` | Logged `LAYERFS_LIVE_DOCKER=1` and the sealed image tag; all three managed SDK/container tests PASS with no filtering. Actual FUSE mount and lifecycle assertions execute. `containers-after.txt` is empty. |
| `old-binary-final-01` | Actual older-source/candidate probe exits 0. Expanded fixture imports an 8192-byte regular file, requires native admission in the version-7 arm, performs full readback, verifies old-code open rejection without mutation, and alternates old/new version-6 Directory writes with complete readback. No journal/WAL/SHM residue. |
| `quality-final-01` | Formatting check, workspace warning-denying Clippy (`-D warnings`), and `git diff --check` all exit 0. |
| `docs-final-01` | One real compile-fail doctest PASS, no failures. Other crates correctly report zero doctests; those are not represented as additional tested guarantees. |

The native workspace total is a harness total, not a claim that environment-gated live tests execute automatically. The four direct-host Linux-FUSE tests are unactivated on macOS and are not counted as live coverage. That topology is inapplicable on this host. Applicable managed Linux FUSE coverage ran explicitly in `live-final-02`, with host SDK/SQLite and Linux daemon/FUSE/workload; Linux runtime unit checks ran separately. This does not introduce or claim support for a Docker-owned SQLite Store. The separately activated managed lane resolves the previously pending supported FUSE/container integration obligation.

The final runtime image is Linux arm64 `sha256:bcdef310f25f5a430f4e6e8c8c312213d5561cd18d0d23955dcfbec5b8029299`, tagged `layerfs-issue90:55c0f273b8ffb561`; its labels match the candidate commit/tree/source/product seals. Native build/tests use Rust 1.85.1 and the locked workspace graph; quality checks use Rust 1.96.0. The actual older-source probe retains its separate lock (`abbb04982d514607adfb34ed6fa2114d1f9d2edb36ee8caeb0d15b5e7aacd66e`) and source hash (`6bf040c828daf3c113da949c119f9329d505feefdfd90fc939cd9ef904361b44`), with clean old source `bfbc46c11dbb4d8dd949b54845e222cf42cd822f`. It remains an actual-code compatibility verifier, not an archived shipping executable.

Earlier failing receipts remain preserved, including the incorrect route-count assertion, intermediate A08 mismatch, missed Workspace publication ordinal, unavailable Docker socket attempts, and the recorded Docker GUI quit event. They are not replaced by successful reruns or attributed to another producer. The recovered final Docker/live runs have their own exit results and source seals.

Raw final log SHA-256 values:

| Receipt `output.log` | SHA-256 |
|---|---|
| `workspace-final-02` | `3f4cb6e294b7e0e94c9d395f450bedac5b91f23d8a0d4d04c7ec933e0a2fa06f` |
| `large-spill-final-01` | `9d9eddd59fb043a100ef1144d77cff3d0a0db21536c6753c781f3fe4eb5a2ad4` |
| `runtime-check-final-02` | `40c1b04621d2bb648c121b3d1fb3c7bc757b9baa4c3d9e99e718e0e35aaf9135` |
| `live-final-02` | `4a0bc0a6abce510a409559f6b3ece3ade45a0930103925e5a2df05bf49178082` |
| `old-binary-final-01` | `c76a2a29b12946f28cd29c577af5061703aebf40d4dec4cf0382ffc964fae45c` |
| `quality-final-01` | `16f8d098f75fbb430b5d227b75730e5e6609120bcda25dcaf8bb512f0079b5ba` |
| `docs-final-01` | `b479d4283866cb993b570802ee5fbf44a651efe026d8ed1790ff6d6d58090ea8` |

**Final independent disposition:** no unresolved implementation or applicable correctness/quality-gate blocker was identified for phase-1 completion. The nine original failures and additional reviewed defects have concrete repairs with meaningful final passing coverage. Root may close issue 90 after publishing the resolved ledger, compatibility/ingestion contract, exact candidate handoff to issue 91, and reviewable source through the authorized workflow. This review does not itself publish or close an issue, merge a PR, start phase 2, or claim broad performance qualification. The reviewed adoption candidate remains the exact source above; subsequent reporting-only commits should reference it rather than silently reassigning its results.

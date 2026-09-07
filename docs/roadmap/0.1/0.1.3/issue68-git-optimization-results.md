# Issue #68: first-stage Git-workflow optimization results

This delivery implements the first-stage optimization accepted by the user. It does **not** claim the original aggressive performance objective is complete. The 500 ms / 1,000 ms complete-lifecycle targets and the 350 ms / 750 ms stretch targets remain unchanged; all six corrected final LayerFS observations miss them. Corrected-harness qualification is complete after the preparation and proof-routing repair described below. Further performance work belongs to [issue #70](https://github.com/Ephemeral-AI-Lab/layerfs/issues/70); first-stage acceptance does not redefine a target miss as a performance pass.

The machine-derived [results table](issue68-evidence/report.md) and [full report](issue68-evidence/report.json) use the corrected baseline and the predeclared final cohort. The [final declaration](issue68-evidence/raw/final-declaration.json) carries the repaired qualification source seal `7fc46647...` and was committed before its twelve observations were collected. A report or declaration still carrying the earlier source seal belongs to the archived pre-correction campaign, not to this qualification. Collection-mode PASS and the historical 15-second threshold are not acceptance criteria.

All durations below are milliseconds. Before is one fresh corrected-input observation of the old product; after is the predeclared three-run median and maximum. Native measures apply plus six Git commands, not a LayerFS lifecycle.

| Case | Before LayerFS (n=1) | Final LayerFS median | Final LayerFS maximum | Native before (n=1) | Final native median | Final native maximum | Main / stretch |
|---|---:|---:|---:|---:|---:|---:|---|
| Git-100 | 5827.275 | 1852.297 | 1861.588 | 273.585 | 249.184 | 267.418 | **TARGET_MISS / TARGET_MISS** |
| Git-500 | 14118.306 | 4635.514 | 4712.030 | 644.545 | 634.505 | 639.123 | **TARGET_MISS / TARGET_MISS** |

| Case and arm | r1 | r2 | r3 |
|---|---:|---:|---:|
| Git-100 LayerFS | 1852.297 | 1861.588 | 1847.577 |
| Git-100 native | 267.418 | 247.930 | 249.184 |
| Git-500 LayerFS | 4712.030 | 4601.212 | 4635.514 |
| Git-500 native | 639.123 | 634.505 | 624.705 |

Git-100: 68.21% lower lifecycle time (3.15× speedup); Git-500: 67.17% lower lifecycle time (3.05× speedup). These are descriptive comparisons from one baseline observation to the final median, not statistical confidence estimates. All six final LayerFS observations miss the original main targets. No slow run was replaced.


## Preparation and proof-routing correction

A wrap-up audit found two benchmark-harness defects. Copying `.git` back with `docker cp` changed the host fixture directory's root mtime before import. Separately, the mixed-v4 sampled-verification selection also selected Git cases, so the earlier Git-100/Git-500 receipts marked PASS had executed only sampled checks, not the existing full Git proof. The mismatched root metadata was a preparation defect; these findings are not evidence of product data corruption.

Qualification commit `6c076911617803f16cba44f29763c26317093c22` repairs both boundaries: restore the declared root metadata after the copy and before import; change Git preparation compatibility to `git-input-root-metadata-v2`; and exclude Git from sampled verification so the existing full canonical verification, expected Git head/tree/parent checks and precommit/reopened custody comparison actually execute. The preparation marker prevents old prepared inputs from being silently reused. Details belong in the [defect receipt](issue68-evidence/raw/preparation-defect.json), and the [audit script](issue68-evidence/audit.py) checks proof contents rather than accepting an outer PASS label.

The baseline received **the same harness repair**, without the product optimizations: origin/main product commit `2b12825d8334165bfdc64dd61a9b0b226f0c33c3`, with qualification commit `b345b3e4839b9949a073d3a1d8c35e92fd4e1976`. The [baseline harness patch](issue68-evidence/raw/baseline-harness.patch) records that separation. Corrected baseline lifecycle observations are `5,827,275,250 ns` for Git-100 and `14,118,306,251 ns` for Git-500. The corrected baseline full Git-100 and Git-500 proof audits passed; see their [Git-100 receipt](issue68-evidence/raw/baseline-proof-100/verification.json) and [Git-500 receipt](issue68-evidence/raw/baseline-proof-500/verification.json). Required evidence includes `canonical-verification`, `git-reopen-custody`, the expected Git identities/content checks and cleanup; `sampled-canonical-verification` alone is insufficient.

The earlier cohort is archived and excluded from corrected acceptance statistics. Its LayerFS medians, **2.094491 s for Git-100 and 5.065647 s for Git-500**, are historical pre-correction observations only. They are not the final numbers and must not be combined with the corrected baseline to claim a speedup. Rebuilding and collecting a newly predeclared cohort is required by the repaired preparation/proof contract, not a rerun to select favorable timing. The corrected final CI, build, cohort and identity-matched full proofs all completed; the original performance targets remain misses.

## Measured operation and comparison boundary

The LayerFS metric is the existing integer `pure_call_sum_ns`:

```text
Create + Exec[file changes + cold demand hydration + six Git commands]
       + LayerFS Commit + visibility + End
```

The six Git stages are first status, working-tree diff, add, cached diff/check, Git commit, and final status. Apply time and each stage are reported separately. A Git commit and a LayerFS Commit are different operations; neither is omitted from the LayerFS lifecycle.

The native control measures **apply plus the same six Git commands**. It has no invented equivalent of LayerFS Create, publication, visibility or End. Its independent fixture copy leaves ordinary filesystem pages warm and invalidates the copied index's inode/ctime relationship; there is no pre-timing status command that refreshes the index. Native workload time is therefore shown as its own scoped control, not relabeled as a LayerFS complete lifecycle.

The product topology stays host-store: macOS owns the SDK/coordinator, SQLite, canonical publication and physical spool; Docker runs Linux/FUSE and the workload with two CPUs, 2 GiB RAM, no swap, a 256-PID limit and no host data-sharing mounts. Each final observation has independent mutable state and a cold execution-owner/kernel cache. All demand acquisition, optional prefetch, byte copying and kernel-page prefill performed during execution remain inside Exec and the complete lifecycle. Fixture/genesis preparation and independent correctness proofs are separately timed.

Case IDs, seed 1, fixture sizes, the ignored 50 MiB blob, imported-index state, file-edit routes and Git settings are unchanged. The implementation contains no Git-path or benchmark-case special cases. Small-file eligibility follows a generic byte bound, so adjacency does not materialize the ignored large blob.

## Final implementation

The host snapshot reader reuses the authenticated direct lookup for singleton object reads and keeps structural objects in its bounded 8 MiB cache. Batch ordering, duplicate requests, missing objects, corrupt-object errors and authentication remain part of the existing read contract. Directory lookup information is reused rather than repeatedly reconstructing the same immutable search work.

The execution owner acquires useful bounded groups of immutable facts. An exact name acquisition can include its validated directory-leaf siblings and bounded small-file content. Directory pages retain an explicit continuation/completeness boundary. Cached partial pages establish absence only within their validated covered range; they are not treated as complete directories.

One daemon-shared 32 MiB charged cache holds immutable name facts, directory-page metadata and clean content ranges. Keys include a unique owner scope, namespace/directory identity where applicable, and immutable file-root identity for content. Owners do not share mutable namespace state or authorization. Eviction and rejected optional admission fall back to normal acquisition; cache fullness is not an ENOSPC result for an otherwise valid filesystem operation. Per-file content prefetch is limited to 8 KiB and each grouped response has a 512 KiB content budget. Metadata pages exclude duplicate inline content copies in the cache.

Authenticated backing connections retain their workspace capability boundary. Responses are decoded and checked before installation. `complete_name` revalidates namespace root and parent revision; directory installation checks the captured root/revision too. Delayed old-root replies cannot overwrite newer state. Existing live directory changes take precedence, and canonical inode reuse preserves already modified data rather than reinstalling prefetched base facts. Immutable old file roots remain usable by retained open/unlinked identities. No cache mutex is held over backing I/O.

```text
Before: dependent acquisition                After: bounded immutable reuse

Linux operation                             Linux operation
      |                                           |
Live owner                                  Live owner + live changes
      |                                           |
LOOKUP / READ_BASE                           immutable fact/range cached?
      |                                           | yes       | no
Host backing service                              v           v
      |                                      local result  grouped acquisition
SQLite / canonical decoding                                   |
      |                                                  host authenticated
reply, then next dependency                               immutable reader
                                                              |
                                                   validate root/revision
                                                              |
                                                  cache facts; install demand
```

Kernel inode lifetime is tracked with u64 lookup-reference counts and one live pin per referenced inode, with scheduler-charged storage. Positive replies retain their references atomically with namespace resolution. Reply guards release entries not emitted and roll back local pre-send failures; FORGET and drained detach release the remaining references. This supports inode-based file I/O while preserving unlinked-file identity. Internal SDK lookups do not manufacture kernel references.

The final file-open path is explicit OPEN, despite the earlier zero-message-open experiment. It observes writable flags, preserves `FOPEN_KEEP_CACHE` for normal cached opens and uses `FOPEN_NOFLUSH` only for read-only descriptors. Writable descriptors keep their flush path. Created write handles retain the existing direct-I/O behavior. FSYNC, write acknowledgements, Commit cuts, error handling and cleanup are not replaced by a timing shortcut.

An eligible immutable read-only open can optionally fill already-acquired complete small-file bytes through the public upstream `Notifier::store` API. A single dedicated worker per runtime has a 256 KiB stack and a zero-capacity, try-only handoff: busy or unavailable workers cause optional prefill to be skipped. Required SDK invalidation and physical backing workers are independent of this worker. There is no queue of optional jobs and no shared-cache lock held during STORE.

The accepted STORE job owns its ordinary-operation guard, bytes, owner and completion guard. Cancellation of the awaiting OPEN does not release those guards while kernel work continues. An owner-local active marker survives FORGET; writable OPEN, ordinary writes and truncation exclude an active prefill under their existing inode ordering. CREATE records writable access. Successful prefill state is installed only after file-root/data revalidation; panic and error completion release the marker and wake waiters.

```text
RO OPEN -> eligible cached bytes -> try-only dedicated STORE worker -> OPEN reply
                                      owns guards/completion

RW OPEN / WRITE / TRUNCATE -> inode order -> exclude active STORE -> mutation
SDK edit / Commit cut     -> ordinary-operation drain            -> publication
```

The final dependency remains unmodified registry `fuser` 0.18.0. There is no vendor fork, Cargo patch, custom dependency decoder or kernel patch. The implementation uses public supported APIs; dependency provenance alone is not a substitute for executed coherence tests.

## Source and evidence identity

| Identity | Recorded value |
|---|---|
| Product implementation commit | `dc1573023850ba6c175d3c8ed9549a1ea800ef5f` |
| Product implementation Git tree | `fb2b39b488b7c93ebe5ab81f5cdb9f6386b3047d` |
| Repaired qualification commit | `6c076911617803f16cba44f29763c26317093c22` |
| Repaired qualification Git tree | `ecdb6cb74c8e53c13d3f3f24bf6d0981cc6538ec` |
| Repaired qualification source seal | `7fc46647cf1ce52c10b036f33c16b1b7aa7aaa2a6db41aa3ecf8935fef00e074` |
| Unchanged product seal | `77a2704d2f3a3cb33a81d99b3459ac8df43cb278aae0c2bbf08435db6dfb8114` |
| Workload SHA-256 | `2ec809798656deef9ac05903c9e3bd77921f6ef9c6c87f24060812a81b93bf76` |
| Literal build dirty label | `LAYERFS_SOURCE_DIRTY=true` |

The dirty label is preserved exactly as recorded. The runner hard-codes it to `true`, even for committed product source; it is not a claim that the measured product contains uncommitted changes. Product implementation remains attributable to `dc157302...`; repaired qualification is attributable to `6c076911...`. The harness repair changes the source seal while leaving the product seal unchanged. The new image, prepared fixture, declaration, performance receipts and proof audits must all match the repaired qualification identities. The earlier source seal `8ba31f5e21a3614d1843e74094a2cd7656e073d60346566ef778b31a5c56ab56` identifies archived pre-correction evidence only.

The corrected final statistics must contain exactly three repetitions per case per arm, with alternating arm order, and report every result, median and maximum. They do not claim a p95 from three observations or replace a slow run with another attempt. Exploratory and profiling observations remain separate from final acceptance statistics. A speedup comparison identifies its actual baseline and native scope; SQL/cache counts are not converted arithmetically into elapsed-time claims.

Resource counters retain their measured scopes: host process CPU/I/O deltas, Linux-container CPU/memory, backing requests and waits, cache/SQL work, and transferred bytes. Nested waits and host-dispatch timers are not added together as independent elapsed components. Known unwired zero fields become annotated unavailable values in the derived report, while the raw receipts retain their literal contents.

## Verification and issue disposition

Corrected final correctness evidence is separate from timing observations. Each linked log/receipt must be audited against the repaired qualification source before it supports a final PASS:

- [Required CI log](issue68-evidence/raw/final-ci.log).
- [Focused live Docker coherence/concurrency log](issue68-evidence/raw/final-live-docker.log). A gated test returning early is not execution evidence; inspect the live flags and named test output.
- [Git-100 independent verification receipt](issue68-evidence/raw/final-proof-100/verification.json).
- [Git-500 independent verification receipt](issue68-evidence/raw/final-proof-500/verification.json).
- Compact controls, when published: [Git-1 receipt](issue68-evidence/raw/final-proof-1/verification.json) and [Git-10 receipt](issue68-evidence/raw/final-proof-10/verification.json). These do not replace either primary full proof.

The linked receipts, rather than this narrative, establish individual outcomes. Git proofs must match the final source/product/image/case/fixture identities and cover expected head/tree/parent, source contents, final Git status, LayerFS publication/reopened state and cleanup. The proof audit must confirm full canonical and Git semantic/custody checks actually executed; an outer PASS or a sampled-only receipt is insufficient. They use the Git-specific oracle and the bounded verification lane; a generic tree verifier that rejects `.git` is not substituted. No proof's elapsed time enters a performance median.

Focused implementation checks cover immutable-cache bounds/isolation, delayed acquisitions, malformed grouped replies, stable unlinked inode references, lookup-count overflow, partial-page reply rollback, drained detach, prefill exclusion, cancellation/panic cleanup and independent required-worker progress. Existing live SDK/FUSE, mapping, concurrent-Commit and lifecycle behavior must be supported by the executed log, not inferred from these unit seams. This delivery does not claim a new many-workspace scalability qualification.

| Issue | Delivery status and remaining condition |
|---|---|
| [#68](https://github.com/Ephemeral-AI-Lab/layerfs/issues/68) | First-stage implementation and evidence delivery accepted. The original 500 ms / 1,000 ms aggressive gates remain unchanged; earlier observations missed them, and corrected final qualification is pending. Further performance work is tracked in #70. Any first-stage issue disposition must state this distinction. |
| [#50](https://github.com/Ephemeral-AI-Lab/layerfs/issues/50) | Remains open. Git improvement does not supply the required same-source create/delete qualification, measured write/Exec disposition and applicable write/fence/resource controls. |
| [#51](https://github.com/Ephemeral-AI-Lab/layerfs/issues/51) | Remains open. The unmodified upstream dependency is retained. Corrected final CI/live evidence must be checked for the repaired qualification source, and its required same-source create/delete qualification and selected proofs remain absent. This Git-focused delivery does not close the issue. |

## Experiment dispositions

| Treatment | Disposition |
|---|---|
| Direct singleton reads, host structural caching, complete-leaf sibling acquisition and bounded execution cache | Retained in the implementation. Final cohort, not exploratory samples, determines reported performance. |
| Kernel lookup-reference lifetime, explicit inode-based OPEN and read-only NOFLUSH | Retained. Final OPEN remains observable so writable access and optional prefill can be handled. Writable FLUSH is preserved. |
| Public-API optional kernel STORE through a dedicated try-only worker | Retained in the declared implementation; its cost and mechanism counters belong in the measured lifecycle. |
| 60-second kernel attribute/entry TTL | Removed. It did not establish an elapsed-time benefit and raised completeness/coherence concerns. Final TTL remains one second. |
| Recursive/BFS acquisition groups | Rejected. Fewer requests did not establish useful lifecycle improvement over flat bounded acquisition. |
| Global suppression of writable FLUSH | Not adopted. Read-only NOFLUSH does not justify dropping writable error/fence behavior. |
| Passthrough or writeback-cache mode | Not implemented. Upstream capability inspection is not a correctness or performance qualification. |
| CPU profiling and diagnostic configurations | Diagnostic only; excluded from the fixed final cohort and target classification. |

## Reproducing the campaign

Run from an isolated checkout of the desired implementation with Docker available. Builds, LayerFS measurements and native controls remain serial under the existing measurement lock; the supplied runners perform their lock admission. Never run concurrent builds or timing arms. Do not reuse an output directory.

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
IMAGE=$(python3 benchmark/fs-bench-pro/shared/runner.py --build-image)
EVIDENCE=$(mktemp -d "${TMPDIR:-/tmp}/layerfs-git-cohort.XXXXXX")
```

Before collection, create and preserve a fresh declaration. The committed declaration is the operation-contract template, **not a source identity to copy onto a new version**. Replace every source-build identity with the current runner's output and record a new declaration time; retain the exact cases, schedule, boundaries and cache policy unless defining a separately labeled experiment.

```bash
python3 - "$EVIDENCE" <<'PY'
import datetime
import json
from pathlib import Path
import sys

repo = Path.cwd()
sys.path.insert(0, str(repo / 'benchmark/fs-bench-pro/shared'))
import runner

template = repo / 'docs/roadmap/0.1/0.1.3/issue68-evidence/raw/final-declaration.json'
declaration = json.loads(template.read_text())
declaration['source_build_args'] = runner.source_build_args()
declaration['declared_at_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
Path(sys.argv[1], 'final-declaration.json').write_text(json.dumps(declaration, indent=2) + '\n')
PY

LAYERFS_BACKING_PROFILE=1 python3 docs/roadmap/0.1/0.1.3/issue68-evidence/run_cohort.py \
  --repo "$PWD" --output "$EVIDENCE" \
  --declaration "$EVIDENCE/final-declaration.json" --image "$IMAGE"
```

For this repaired campaign, use preparation compatibility `git-input-root-metadata-v2` and audit the full Git proof route; do not import or reuse the pre-correction prepared fixture. The cohort script checks current source/product/workload seals and image labels, requires new mutable destinations, and follows the declared twelve-run schedule. It invokes the registered LayerFS family runner and the matched native control; native fixtures are taken from the case's first LayerFS preparation with matching identities. Preserve the declaration independently before starting collection; filesystem timestamps alone are not proof of predeclaration.

For one focused exploratory sample, use the registered family boundary and a new destination, keeping it out of final medians:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py \
  --topology host-store --family git_tool_workflow \
  --case git-tool-100-mixed-v4 --seed 1 --setup clone --image "$IMAGE" \
  --collection-mode --product-timeout 300 --timeout 310 --setup-timeout 600 \
  --output "$EVIDENCE/exploratory-100"
```

To regenerate the committed report from the portable raw bundle:

```bash
python3 docs/roadmap/0.1/0.1.3/issue68-evidence/derive.py \
  docs/roadmap/0.1/0.1.3/issue68-evidence/raw \
  --output docs/roadmap/0.1/0.1.3/issue68-evidence \
  --declaration final-declaration.json \
  --proof final-proof-100/verification.json \
  --proof final-proof-500/verification.json
```

For a newly collected campaign, use its raw directory and fresh declaration instead. Include the corrected baseline directories and matching native controls when deriving a before/after report, with their original qualification identities and baseline harness patch preserved; never substitute the archived pre-correction baseline or cohort. Keep the proof audit alongside each full verification receipt; missing baselines or proofs must remain explicit omissions. The derivation script never converts missing runs, identity mismatches or parsing errors into a passing gate.

## Executed final checks

- Required local CI passed: Rust 1.96 formatting, the full Rust 1.85.1 native suite (69 seconds), and Rust 1.96 Clippy with warnings denied. Remote PR checks must also pass on the published head before merge.
- All three Docker live tests actually executed on `layerfs-bench-infra:7fc46647cf1ce52c` with `LAYERFS_LIVE_DOCKER=1`: lifecycle/disconnect cleanup, ordinary writes during mapped Commit, and running commands/dirty mappings across Commit. The new metadata/alias/open-unlink and cold-RO/SDK-resize/RW-mmap/discard markers passed too.
- Corrected baseline and final Git-100/Git-500 full proof receipts passed. The audited records include canonical verification, expected head/tree/parent, precommit/reopened Git custody, and cleanup. Compact Git-1/Git-10 full proofs also passed. All remained within the existing 45-second work / 59-second hard lane.
- All corrected timing and native-control cleanup receipts passed, with unchanged protected masters, no reported OOM kill, and no swap use. The result tables preserve observed memory/CPU scopes.
- The [dependency audit](issue68-evidence/raw/final-dependency-audit.json) matched all 85 files of the local fuser 0.18.0 registry source against the release archive, whose SHA-256 equals Cargo.lock. Cargo dependency declarations and lockfile remain unchanged.

The [raw bundle index](issue68-evidence/raw/bundle-files.json) and [bundle notes](issue68-evidence/raw/README.md) describe archived native command output, omitted reproducible fixture tar payloads, and preserved pre-correction evidence. `read_ahead_unused_bytes` is not instrumented for this live cache and is reported unavailable; cache-served-byte counters cover FUSE READ delivery, while kernel-prefill bytes are separately recorded.

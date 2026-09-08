# Storage architecture v3 implementation progress

Status: **milestones 0–2 complete as a FULL/RAW development checkpoint; stop here**.
Updated 2026-09-08. This is **not final storage-v3 qualification**. Three integrated
smokes passed public-operation completion, historical mounted readback and cleanup.
Three comparisons exceed the original smoke elapsed allowances; the owner accepts
the checkpoint under the subsequently stated timing tolerance below. Compression,
delta emission and milestones 3–5 are deferred; their original targets are unchanged.

The latest owner scope explicitly supersedes the older full-completion instruction:
finish milestone 2, commit/push to existing PR #81, then stop. No merge, deployment,
conversion, automatic/in-place migration or user-data operation is authorized.
The [approved smoke contract](implementation-smoke-contract-v1.md),
[implementation plan](implementation-plan.md), [architecture](storage-architecture-spec.md),
[format](sqlite-storage-format.md) and [review disposition](review-disposition.md)
retain their original definitions. Their final-v3 compressed-storage and representation
requirements are deferred/not applicable as **milestone-2 exit gates**, not passed,
weakened or replaced. The three-pair final qualification has not run.

Subsequent owner timing disposition: the 1.58% replay increase is acceptable;
**up to 10 ms added elapsed time or less than 30% degradation** is acceptable for
this milestone-2 checkpoint. All reported elapsed comparisons meet that tolerance.
This records owner acceptance separately from the unchanged frozen smoke contract
and its original diagnostic misses; it is not final-v3 qualification.

## Recovery and source custody

- Existing worktree preserved: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3`,
  branch `codex/storage-v3-implementation`, draft [PR #81](https://github.com/Ephemeral-AI-Lab/layerfs/pull/81).
- Recovered HEAD was `ad80cf43157ded12c140caadae53f21980f74599`. The complete diff
  contained only the five reported Store files: layerstack.rs, objects.rs,
  objects/admission.rs, objects/read.rs and workspace.rs. All changes were retained.
- The old task `01a07de3-57bf-7b31-abc2-93e4358f98f6` was idle with its last turn
  interrupted. It was not resumed. No concurrent build/smoke writer was found.
  No second agent was used. The original `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`
  checkout and its unrelated work were untouched.
- The shared lock at
  `/var/folders/s4/xpkmz7wn6yq97w1ls_4f_dfc0000gn/T/layerfs-infra-measurement.lock`
  was free; the existing runner acquired it for every image build, performance
  and verification invocation. No jobs overlapped. The unrelated running
  `layerfs-phase21-binary-build` container contained only `sleep infinity` and
  was left alone, as were unrelated stopped containers.
- `builds/m2-host-11.log` was reconciled against the current source seal, host
  sidecar and actual binary hash. The binary matched; its matching image was
  absent. Only that image was built, using `--build-storage-smoke-image` with
  build self-check disabled. No extra test suite ran.
- The preserved correction is committed as
  **`7cad3f603abf08b5e5021c6545854286a473f1b5`**. The smoke binaries were built at
  `ad80cf4` plus this exact dirty patch; the committed product/harness bytes have
  the same seals. Documentation-only reporting does not change these seals.
- Exact recovery patch: `builds/m2-checkpoint-source.patch`, SHA-256
  `15b13f2379c48728545edbd61812ed8a6d0c41f681f5cdbaf2e931f0b06b1641`.
  All paths beginning with `builds/` or `m2-` below are under the external evidence
  root `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs`.

Applicable `benchmark/AGENTS.md`, the controlling documents, shared admission,
packed framing/readers, spill membership and selected-output delivery, Init sinks
and fallback handoff, direct/Workspace publishers, staging and schema gate were
reviewed against their actual callers. The prior milestone source-review history
remains available in the ledger at `ad80cf4`. Ponytail and systematic-debugging
skills were used; the owner's three-smoke-only scope overrides generic advice to
add unit/property/fault tests. No external library/dependency source was changed.

## Resolved authority and completed milestones

The owner approved the narrow v0.1.4 schema exception: explicit **new-Store-only
schema 6 / wire 1 creation**, exact legacy rejection in this implementation,
existing compatible tools for old Stores, and no converter. This decision and the
smoke fixtures/numerical contract are resolved and must not be requested again.

| Milestone | Actual disposition |
| --- | --- |
| 0 | Complete: committed approved contract, three smoke entrypoints, protected fixtures, unchanged-product baseline and all 33 historical mappings |
| 1 | Complete: paged seen/offset operations, buffered/fallible ID-order seals, selected output ownership transfer and retained spill authentication |
| 2 | Complete FULL/RAW vertical slice and integrated three-smoke checkpoint; elapsed deviations and evidence limits below remain explicit |
| 3 | Deferred: Zstandard encoder/decoder, codec allocation/framing qualification and compressed groups |
| 4 | Deferred: whole-file predecessor/first-span handoff, forward extent cursor, shallow delta selection and emission |
| 5 | Deferred: remaining checkpoint/reconciliation cleanup, final module deletion review and final three-pair qualification |

Milestone 2 includes:

- Schema-6 pack BLOBs and selected canonical-length/locator rows; canonical IDs,
  current CDC, CAS and localized COW semantics remain independent of placement.
- One shared prepared admission owner for Init, direct candidate and Workspace
  candidate publication. Initial exact reuse is filtered **before** physical
  accumulation. Only initially missing objects consume pack/admission capacity.
  The private `MissingBatch` handoff carries that partition into final preparation.
- Child-first output and carried available-dependency facts avoid repeated
  dependency probes. Unknown external dependencies use bounded presence SQL;
  demanded stored bytes still authenticate. Unique candidates receive one initial
  membership lookup and at most one final recheck. Supplied duplicate occurrences
  still pay actual byte equality work; this is not a zero-I/O duplicate claim.
- Bounded bulk pack/locator INSERTs under one admission permit, one forward winner
  selection, no UPSERT/retry/shrinking-set loop, and immutable selected locations.
  Zero-winner packs are skipped; mixed-pack unselected records are retained.
- Native serial/parallel/fallback and Empty Init deliver to the same physical
  sink. Production parent payload collection and all whole-Store Init cleanup
  calls are removed. Fallback retains earlier admissions and reports attempted
  source work; it does not delete another writer's objects.
- Targeted FIFO successor channels replace normal broadcast wakeups. Construction,
  source waits and new encoding stay outside the permit. Late equality validation
  retains the permit but releases SQLite before decode/hash/base access.
- Workspace admission precedes its separately committed stage and conditional
  publication. Existing head/base checks, no-change outcomes, required checkpoint
  installation, resume and cleanup remain in the public operation.
- RAW readers validate the selected directory entry once to choose the oversized
  route, including the 65,528–65,536-byte FULL/DELTA overlap. Canonical reads hash
  after releasing SQLite. Existing DELTA reader code is unexercised here; **no
  compression or delta emission is enabled**. Zstandard reads reject explicitly.

## Exact integrated checkpoint artifacts

[Machine-readable observations and comparisons](implementation-milestone-2.json)
contain integer timings/allocation, per-step admissions/routes, resource operands,
physical census, source identities and receipt-manifest hashes. The final
verification manifests were checked against all retained files. The earlier
performance manifests preserve pre-verifier Store custody; verification adds
Branches, so their Store hashes are not substituted for final Store hashes.

| Identity | Exact value |
| --- | --- |
| Implementation correction | `7cad3f603abf08b5e5021c6545854286a473f1b5` |
| Combined source seal | `7f63b4bd3f355c90c7bfaa266c9b956ab8ea8ec9ba83f54b0faa0c6ac1e80938` |
| Product seal | `16ecf5880990894eaf15599bedcb6eea6b59b59fbee3c036df1bff85e06a6d74` |
| Host binary SHA-256 | `342510f7ca63e609f11c71a7fd4fcce50b6b6b7f6b29a26e06900f027da290b2` |
| Runtime image | `sha256:c51d2431b2b29898fa431e1ab843362e482c2b4d96b85da7410df560032ce574` |
| Image tag | `layerfs-bench-infra:7f63b4bd3f355c90` |
| Schema-6 DDL SHA-256 | `53bda8792a601683038af508b183986e1f880a3c0f2158dc3cc9f6bbf1fced50` |
| Independent harness seal | `f51a4906e4176a51e74c5af6272d3cd0dc11aa681fda9527949373e60e84ca0a` (definition/files in JSON) |
| Retained host binary and original identity sidecar | `builds/m2-checkpoint-host`, `builds/m2-checkpoint-host.identity.json` |
| Matching preparation | `builds/m2-host-11.log`, `builds/m2-checkpoint-image-1.log` |

Executed serially on fresh mutable Stores with protected inputs, existing OS
caches, host SQLite/SDK/coordinator/spool, managed authenticated Linux daemon/live
core and real FUSE; no data-sharing mount or container Store. Performance and
historical verification used separate invocations and timers.

| Smoke / artifact directory | Performance | Historical mappings | Cleanup |
| --- | --- | ---: | --- |
| `m2-checkpoint-edits-1` | PASS: four distinct SDK/ordinary histories | 24/24 PASS | PASS, both phases/all four cases |
| `m2-checkpoint-small-1` | PASS: Init, two reads, three mutations/Commits | 4/4 PASS | PASS, both phases |
| `m2-checkpoint-deepseek-1` | PASS: exactly first FIVE frozen entries | 5/5 PASS | PASS, both phases |

PASS above means complete public operations/oracles/cleanup, **not numerical
qualification**. All created states and no-change aliases were independently
read after reopen through mounted FUSE. SDK edit member counts remain 1/1/1/3/0,
with public edit calls 1/1/1/1/0 and zero SDK-caused FUSE write counts. Ordinary
Exec keeps in-place range writes, whole replacements and tempfile/rename. DeepSeek
keeps the original ordinary importer. All four final unchanged edit Commits report
UpToDate. No selected DELTA or compressed group is claimed.

## Allocation, transactions and elapsed observations

Each arm below has **one observation**, using the initial diagnostic unchanged
baseline in [implementation-baseline.json](implementation-baseline.json). These
are descriptive ratios, not final paired estimates or statistical claims. The
current harness includes added untimed Init receipt rendering; final qualification
still requires its frozen identical-harness three-pair schedule.

| Case | Baseline final allocated bytes | FULL/RAW final allocated bytes | Allocation change | Baseline mutation+Commit sum ns | FULL/RAW sum ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| DeepSeek first five | 12,320,768 | 11,206,656 | −9.04% | 1,095,992,417 | 1,113,312,833 |
| SDK text-32k | 983,040 | 983,040 | 0% | 32,936,917 | 28,003,959 |
| SDK binary-8m | 11,141,120 | 10,354,688 | −7.06% | 28,219,670 | 35,819,625 |
| Ordinary text-32k | 1,179,648 | 1,179,648 | 0% | 40,490,999 | 45,286,042 |
| Ordinary binary-8m | 11,206,656 | 10,420,224 | −7.02% | 263,891,416 | 260,996,792 |
| Small files | 1,441,792 | 1,507,328 | +4.55% | 22,108,291 | 26,282,708 |

Initial small-file allocation is 1,441,792 bytes in both arms. Both binary candidate
histories start at 10,289,152 bytes versus 11,075,584 baseline; growth is 65,536 bytes
for SDK and 131,072 for ordinary in both arms. Small-file final growth is one
64-KiB SQLite page; it is retained, not reclaimed to improve the result.

All changed edit Commits have **one object-admission transaction**, plus existing
stage and publication transactions. Earlier `m2-edits-1` ordinary binary replacement
steps 2/3/4 had 29 admissions; `m2-edits-2` and this checkpoint have one. The unchanged
baseline had four for those replacements and two for the other changed edit steps.
DeepSeek candidate admission counts are **11/9/10/15/12**, versus baseline 2 each:
new-object and worst-selected-representation validation bounds still split larger
batches. There is no claim that every workload has fewer transactions. Stage and
publication are never counted as eliminated by the admission reduction.

Original-contract diagnostic elapsed misses (retained unchanged; accepted under
the owner’s subsequent milestone-2 tolerance):

| Quantity | Baseline ns | Candidate ns | Approved diagnostic allowance ns | Disposition |
| --- | ---: | ---: | ---: | --- |
| SDK binary step 2 | 5,971,876 | 8,699,042 | 7,971,876 | Above allowance |
| SDK binary return-to-A | 7,402,667 | 10,628,125 | 9,402,667 | Above allowance |
| Small-file shrink step 3 | 7,294,833 | 10,056,667 | 9,118,541.25 | Above allowance |

Small-file Init is 7,832,458 ns versus 7,028,667 baseline; first/repeated reads are
23,719,084 / 7,419,458 ns versus 19,658,084 / 7,439,041. These and the other
small-file steps are within their diagnostic allowances. DeepSeek replay is +1.58%
and each checkpoint is within its allowance. All ordinary and SDK-text steps and
all six historical-verification wall comparisons are within diagnostic allowances.
The JSON preserves every operand and individual step, including passing values.

The remaining SDK binary return-to-A latency was inspected without speculative
RAW tuning. Compared with `m2-edits-2`, object admission fell from 724,500 to
549,375 ns; content construction rose from 2,211,499 to 3,151,583 ns, while
snapshot calls/bytes remained 8 / 30,123 and CDC scanned 12,288 bytes. Runtime
wait/fence/checkpoint timings also vary. These receipts do not establish a new
membership/batching defect or justify a broader rewrite. The elapsed deviation
remains visible against the original contract and is accepted under the owner’s
subsequent milestone-2 tolerance. Neither noise nor future compression is claimed
to have removed the elapsed difference.

## Physical accounting, resources and coverage limits

Read-only physical census was performed after historical verification and outside
all operation timers. It does not replace pre-verifier allocated-byte receipts.
Canonical object counts/bytes equal the performance final counts; verifier-created
Branches do not add object packs. The six Stores contain only selected FULL/RAW
records: no unused records or retained workspace stages were observed. Unreachable
selected canonical objects, if any, remain counted; no reachability/GC claim is made.

| Case | Packs | RAW groups | FULL records | Pack BLOB bytes | Framing bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| DeepSeek first five | 58 | 424 | 5,557 | 9,017,260 | 37,193 |
| SDK text-32k | 5 | 11 | 46 | 62,863 | 530 |
| SDK binary-8m | 61 | 408 | 488 | 8,498,192 | 11,576 |
| Ordinary text-32k | 5 | 10 | 43 | 107,171 | 495 |
| Ordinary binary-8m | 61 | 408 | 481 | 8,521,222 | 11,541 |
| Small files | 7 | 23 | 461 | 399,152 | 2,877 |

Every observed pack is below the 256-KiB cap, including final partial packs. This
unused capacity is **not padding or allocated payload**. Full pack-size ranges and
zero observed unselected-record bytes are in the JSON. SQLite pages, indices and
other overhead remain in the actual allocated totals. Concurrent mixed-pack waste
and failed Init/fallback admissions are source-reviewed behaviors, not exercised
zero-waste guarantees. No DELETE, VACUUM or repack was run.

Performance host lifetime RSS peaks range from 10,158,080 to 32,817,152 bytes;
container lifetime peaks range from 4,919,296 to 14,909,440 bytes; sampled spool
peaks are 4,096–24,576 bytes. All are within diagnostic comparative allowances and
absolute safety budgets. No container OOM/swap or forced coordinator stop was
reported. Host phase CPU/I/O, container category snapshots, temporary/staging and
cleanup receipts are retained separately. These are lifetime/boundary/sampled
scopes, not exact operation-phase memory maxima or aggregate capacity evidence.

Candidate membership-query counts and actual BLOB/group-read call/byte counts are
**unavailable**, not zero. Snapshot database counts are logical reader API counts;
they cannot prove physical SQL/range-call counts. Source inspection establishes the
bounded page/recheck algorithm, and smoke receipts establish actual admission
transaction counts. Synchronous physical preparation and finalization remain inside
the measured public operations even where subphase instrumentation is incomplete.

Not exercised: arbitrary malformed/oversized records (including the RAW/DELTA
overlap), Zstandard or selected DELTA reads, late duplicate races, FIFO poison or
abandonment schedules, nonempty-Store/native topology fallback failures, direct
reconciliation conflicts, stage/head races, transaction/finalization failure
injection, disk-seen threshold populations and truncated spill recovery. Safeguards
were not weakened to avoid these cases. No unit/property/fuzz/race/crash suite,
fourth smoke, full-family campaign, 157-state replay or final-v3 qualification ran.

## Retained earlier evidence

- Unchanged product baseline: `28177560c8f049c02192e18c263cdc5543c1ab52`, approved
  contract commit `96e796431964f7a00af7fd1d7cc647029f2c5eaa`, harness/baseline commit
  `b4b967b91`. `implementation-baseline.json` retains exact source/binary/image and
  all six histories. Baseline host SHA-256 is
  `ce6a81a629d7d1adafaea74933acc502edb21e4ca650a396172f3574f26a0746`.
- M1 commit `86d04b95ed360835f1a861fd7104218794deb142`; `m1-small-1`,
  `m1-deepseek-1`, `m1-edits-1` retain their own source-specific performance,
  historical verification and cleanup results. They are not reused as M2 proof.
- `m2-small-1`: first-read 29,530,000 ns; `m2-small-2`: 23,176,834 ns. Both retain
  their valid observations and full historical receipts. The latter Init was
  9,228,167 ns, above its prospective allowance. Neither is final qualification.
- `m2-edits-1` source `654c583fd3c6129c6c8093851d56a0bf46f1dcf3d6c0d82e36139469d1ef1568`;
  `m2-edits-2` source `3601b48c2cf1f6d72a405ccf4055722c308971db1b103a16df8657866f7f530b`.
  Both passed 24 historical mappings and cleanup. Ordinary binary allocation
  changed **11,403,264 → 10,420,224 bytes between these two intermediate RAW
  iterations**, not between final candidate and unchanged baseline.
- Build failures remain under `builds/`: preparation-2/3 (captured Cargo diagnostics
  and ToSql correction), m2-host-1 (producer/sink edit correction), m2-host-6
  (infallible emitter correction), plus all subsequent build logs. No failed or
  slow observation was overwritten, discarded or reclassified as a passing gate.
- Frozen DeepSeek manifest SHA-256 remains
  `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`, tip
  `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Only its first five manifest entries
  were executed; no Git footprint arm or Git-proximity claim was added.

The checkpoint is ready for owner review in PR #81. Work stops at milestone 2.
Milestones 3–5, the full completion contract and final compressed-storage targets
remain deferred; this report grants no merge, deployment or migration authority.

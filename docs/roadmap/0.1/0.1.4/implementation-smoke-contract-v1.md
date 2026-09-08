# Storage v3 development smoke contract v1 — approved

Prepared 2026-09-08, before any baseline or candidate observation. This makes
PR #80's remaining choices concrete. **Approved by the owner in the implementation
task before observations.** Execution requires matching implemented entrypoints; no implied
0.1 exception, fourth smoke, full-family registration or release qualification.
Tracking: [implementation ledger](implementation-progress.md),
[PR #80](https://github.com/Ephemeral-AI-Lab/layerfs/pull/80), and
[issue #18](https://github.com/Ephemeral-AI-Lab/layerfs/issues/18).

Authority: [pinned smoke plan](storage-smoke-test-plan.md),
[implementation plan](implementation-plan.md),
[benchmark rules](../../../general/benchmark_rules.md), and
[hosting](../../../../benchmark/AGENTS.md).
The implementation request restricts executable verification to these three
smokes; historical self-check, parser regression, tiny-history and wider-family
commands are not additional authorized execution.

## Resolved owner decisions

1. **Approved by the owner in the implementation task:** a narrow **0.1.4 schema-only exception** for explicit new-Store creation
   with schema 6 / pack wire 1; preserve canonical and other compatibility
   requirements. New binaries reject legacy Stores without rewriting them.
   Existing compatible tools remain the legacy access route. No same-binary
   legacy reader/writer or converter is included. This release choice is resolved;
   do not request it again.
2. **Approved by the owner:** the smoke-specific comparison, resource and tradeoff criteria below.
   These are prospectively frozen requirements, not
   conclusions derived from historical measurements.

Both decisions are resolved. This contract is committed before smoke implementation
and observations. Later owner instruction grants autonomy within this scope and
requires no further routine approval requests.

## Identity, entrypoints and execution ownership

Unchanged product baseline: `28177560c8f049c02192e18c263cdc5543c1ab52`.
Apply the identical smoke harness/workload/oracle files to this product source
and the candidate; record product-only and harness-only seals independently.
Never execute the old experiment's product checkout. A dedicated unchanged
baseline build is current production source plus the approved harness only.

Proposed selectors in the existing shared runner (not currently implemented):

```text
--storage-smoke deepseek-five|frequent-edits|small-files
--source-arm baseline|candidate --repetition 1|2|3 --output UNIQUE_DIRECTORY
--storage-verify-run EXACT_PERFORMANCE_DIRECTORY
```

The existing host binary gets a thin `storage-smoke-session` dispatch. Reuse
`benchmark_container_binding`, `benchmark_client`, public SDK calls, the output
draining execution helper, resource snapshots and current managed runtime.
Adapt the pinned #72 preparation/import/oracle helpers into these owners.
No selector implies `--all`, old tiers, or existing family qualification.

Host: macOS SDK/coordinator, SQLite, canonical construction/publication and
physical spool. Container: managed authenticated daemon/live core, real FUSE
and workload helper. Reuse `runtime.start_sample` inspection and shared
`layerfs-infra-measurement.lock`; preserve one lock path across arms and builds.
Docker staging is outside FUSE; no data-sharing mounts or container Store.
Record actual mounted path, terminal execution receipts, Commit outcomes,
SDK/Exec counts and available FUSE counters. Missing counters are null with
source-traced route plus runtime witnesses; never fabricate zero.

## Smoke 1: exact frozen DeepSeek prefix

Manifest SHA-256:
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`.
Source tip: `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`.
Manifest and helpers come from documentation commit
`1c7c9235115d1b4f21bc2eae7af822552b7be3ed`.

| Entry | Ordinal | Source commit | Logical blob bytes | File entries |
| --- | --- | --- | ---: | ---: |
| 1 | 100 | ca4e1c3a1c61129bd95ee57dd744bc53c2e635bb | 1499109 | 276 |
| 2 | 200 | a009b1e995d38a2149d84067bbf461f8be0e3456 | 1888433 | 307 |
| 3 | 300 | bcf255fc57cb3b6cfca3cd4b1df79ee33815f1bd | 2331770 | 374 |
| 4 | 400 | 25eccdaedcfc918a32a30592219871a807f1e8a6 | 2755845 | 457 |
| 5 | 500 | 93a6dc6716aad759cdb036700d79451b058d2d70 | 3498777 | 530 |

One fresh empty Store, Empty Init, one Branch, one live replay Workspace,
five imports and five Commit attempts; no Add. Preserve pinned importer
`fs::copy` whole-file replacement, path removals/type changes, unchanged files,
0644/0755 modes and mtime normalization to 1000000000. It does not perform SDK
range edits or manufacture tempfile saves. The separate ordinary-edit schedule
below exercises tempfile/rename.

Read all five retained states after pre-verifier allocation capture, through
fresh mounted verification Workspaces on the reopened Store. Compare every
path/type/regular byte/executable bit/symlink target and directory against the
independent Git-derived oracle. Record actual Created/UpToDate mappings.
Never substitute the historical 157-state footprint as a denominator.

## Smoke 2: four short independent edit histories

Two fixed file cases, each with separate SDK and ordinary Exec/FUSE history:
`text-32k` (32768 bytes) and `binary-8m` (8388608 bytes). The larger file keeps
localized editing observable across many CDC chunks without importing the old
100-MiB matrix. Each history starts from fresh native Init, one Branch and one
FUSE Workspace, and retains its initial state plus four changed Commit attempts
and one no-change attempt. Do not pool file cases or operation surfaces.

Fixture byte recipe (ASCII, fixed independent of platform):

- Text A: repeat `export const storage_value = 123456789;\n`, truncate to 32768.
- Binary A: concatenate SHA-256 of `layerfs-storage-smoke-v1/binary/` followed
  by an unsigned 64-bit little-endian block index starting at zero; truncate
  to 8388608. This is deterministic pseudorandom input, not a compressibility
  guarantee. Generate input outside operation timing.
- All initial regular files mode 0644, parent directories 0755, mtime
  1000000000. Each history contains only `file` beneath its root.

For file length L, use 4096-byte spans starting at L/4, L/2 and 3L/4.
State B replaces span 1 with byte `B`; C additionally replaces span 2 with `C`;
D additionally replaces span 3 with `D`; then return to exact A content.

| Attempt | SDK history | Ordinary Exec/FUSE history | Expected content |
| --- | --- | --- | --- |
| 1 | One public range edit | Open existing file; seek/write span; close | B |
| 2 | One public range edit | Truncate/rewrite complete preprepared file; close | C |
| 3 | One public range edit | Write complete tempfile; close; rename over final path | D |
| 4 | One public same-file batch of three range restores | Truncate/rewrite complete A; close | A |
| 5 | Commit only, no mutation | Commit only, no mutation | A, UpToDate |

Complete replacement bodies are prepared outside operation timing and copied
from ordinary container staging through the workload's actual filesystem writes.
Do not infer a logical diff from those bodies. Normalize ordinary write metadata
as above as part of the measured helper. SDK edits keep public SDK metadata
semantics. Compare arms within the same surface only. Record Created versus
UpToDate honestly; identical bytes do not require identical metadata/root IDs.

Measure each edit/Exec and Commit independently and their sum. Attempt 4's SDK
restore uses `Client::edit_workspace_file_ranges`, not a loop of scalar calls.
Record member counts 1/1/1/3/0 and public edit-call counts 1/1/1/1/0.
Ordinary Exec counts are 1/1/1/1/0. Require all five attempts to acknowledge.
No FUSE writes may implement the SDK mutations.

After timing and retained allocation, verify all initial/changed/no-change
mappings through reopened FUSE, every byte and path, using independently prepared
expected states. Candidate ordinary replacement must demonstrate at least one
selected DELTA target and its authenticated selected FULL base across these
histories; missing coverage is not PASS. Baseline DELTA coverage is inapplicable.
Record recurrence reuse without assuming every newly constructed object is reused.

## Smoke 3: small-file Init, readback and three changes

One deterministic tree, 128 regular files, two symlinks and nine non-root
directories (`d0` through `d7`, plus empty `empty-dir`). Put file i under
`d{i mod 8}/f{i:03}`. No dependencies or installed packages.

| Inclusive file indices | Count | Exact content |
| --- | ---: | --- |
| 0–15 | 16 | Empty |
| 16–31 | 16 | ASCII `x` repeated i-15 times |
| 32–95 | 64 | Repeat ASCII `export const file_NNN = 123456789;\n` using three-digit i; truncate to 4096, except i=32 to 8191 |
| 96–111 | 16 | Exact content of file 33 (duplicate content, distinct files) |
| 112–127 | 16 | 4096 pseudorandom bytes: SHA-256 blocks of ASCII `layerfs-storage-smoke-v1/small/NNN/` plus u64 little-endian block index |

Mode 0755 for files 40 and 41; all other files 0644, directories 0755, regular
and directory mtime 1000000000. Root symlinks `source-link -> d1/f033` and
`dir-link -> d0`; symlink timestamps/uid/gid/ctime are outside oracle equality.

Measure public native-directory Init into a fresh Store. Mount FUSE, read all
regular files in bytewise lexicographic path order using a 64-KiB reusable buffer,
then repeat that same read order. Keep first and repeated read passes separate;
report them as first-pass and warm-eligible, never OS-cold. Record byte counts;
digesting and tree/oracle comparisons happen only in the verification phase.

Three separately reported public Exec + Commit steps: append exactly two ASCII
`g` bytes to f032 (8191 -> 8193); chmod f040 from 0755 to 0644 without changing
content; truncate f032 back to 8191. Require mounted current readback and full
initial plus all changed historical-state checks after primary allocation is
captured. Preserve the duplicate files and symlinks. Report initial and incremental
allocation, Init time, each mutation/Commit and both read-pass times separately.

Expected candidate coverage: FULL and Zstandard groups, short objects and final
partial packs, cross-8-KiB growth/shrink and metadata-only content preservation.
RAW/DELTA counts are observations; no direct object injection to force variants.
Oversized canonical records, every parser rejection and concurrency races are
source-review obligations, not exercised claims from this tree.

## Preparation, timing and safety budgets

Use immutable bytes/oracles with recipe, mode, path and helper hashes. Validate
cached manifest and input bytes before acquisition; publish new prepared entries
atomically under the existing lock. No completed Store or live Workspace reuse.
Create fresh Stores for these small histories, including each edit history;
do not call `_host_acquire`'s all-family SDK qualification loop. Canonical/format
changes cannot reuse an old-format prepared Store. Reuse Cargo/Docker caches and
matching sealed binaries; no OS cache purge. Preparation order and treatment match.

Container defaults remain 2 CPUs, 2 GiB memory, memory+swap equal to memory,
256 PIDs, /dev/fuse and required capability/security settings. Host CPU uncapped.
Require 50 GiB free host disk. Per history cap Store at 16 GiB, runtime/spool plus
staging at 16 GiB, combined owned run directory at 32 GiB; sampled coordinator RSS
must stay <=8 GiB. Retain actual available RSS/footprint/CPU/I/O and cgroup
anon/file/kernel/shmem/slab/dirty/writeback/swap/OOM separately. OOM, swap, resource
violation or incomplete cleanup fails. Boundary samples and lifetime peaks do not
claim exact phase peaks; sampled disk maxima do not claim instantaneous maxima.

| Scope | DeepSeek five | Frequent edits (all four histories) | Small files |
| --- | ---: | ---: | ---: |
| Immutable input preparation | 600 s | 120 s | 120 s |
| Runtime setup per instance | 120 s | 120 s | 120 s |
| Single operation acknowledgement | 120 s | 30 s | 30 s |
| Complete performance phase, excluding setup/cleanup | 600 s | 300 s | 120 s |
| Separate complete verification | 600 s | 300 s | 120 s |
| Cleanup per instance | 120 s | 120 s | 120 s |

Build preparation retains current runner's 900-second host/image deadlines and
two Cargo workers; no build during measurements. Timeouts are failure boundaries,
not latency targets. Retain timeout evidence without increasing these limits
after a valid observation to get a pass.

Use monotonic operation timers immediately around public calls. Exec includes
required output draining and terminal receipt, Commit includes required synchronous
finalization. Report presentation failure as failure with published identity;
any recovery is separately timed and does not erase the failure. Input generation,
transfer, setup, resource snapshots, receipt rendering, retained allocation,
verification, End and cleanup have separate scopes. Read timings exclude digest
comparison. Record enclosing wall and unattributed overhead rather than hiding it.

Capture Store `st_blocks * 512` plus every required sidecar (including journal)
after each acknowledged step, before verifier-created Branches. Preserve partial,
unpublished and redundant admitted records; no DELETE, VACUUM or repack to improve
the numerator. Canonical/encoded/framing/index accounting is outside operation
timers and cannot substitute for allocated bytes. Retain all owned evidence Stores;
cleanup removes only run-owned mounts, live sessions, processes and containers.

## Frozen comparison and acceptance

Run one unchanged baseline of all three smokes before product edits. Keep this
initial diagnostic baseline separate from final comparison. For each coherent
slice run one affected smoke sample, with failures/valid slow results retained.
Final qualification uses exactly **three independent pairs per smoke**, ordered
baseline/candidate, candidate/baseline, baseline/candidate. Fresh state every run;
identical final harness for both arms; same machine and declared cache treatment.
No seed search, extra warm-up runs or adaptive repetition count.

Report all three paired values, median and min–max absolute allocation/time,
ratio of medians and each paired ratio. Gate on the ratio of medians and require
each of the three pairs to satisfy the same gate, preventing one favorable sample
from hiding noise. An inconclusive set is not PASS; do not add repeats to rescue it.
If infrastructure invalidates a pair, retain it and rerun that complete pair once;
a second infrastructure failure blocks that qualification. Product failures are
retained failures requiring diagnosis and a newly identified candidate.

Owner-approved development-only gates:

| Separately gated quantity | Required storage outcome | Maximum elapsed increase |
| --- | --- | --- |
| DeepSeek final retained Store | At least 30% reduction | Replay sum of Exec+Commit <=25%; each checkpoint <=35% |
| Small-file initial and final retained Store | At least 20% reduction at both points | Init <=25%; each Exec+Commit <=25% |
| Each SDK file case, attempts 1–3 | Final Store no larger; post-Init growth <=baseline +65536 bytes | Each edit+Commit <=20% or +2 ms, whichever allowance is larger |
| SDK return-to-A and no-change, each file case | Included in same total/growth accounting | Each step <=20% or +2 ms, whichever allowance is larger |
| Each ordinary replacement step, each file case | Final Store no larger; growth <=baseline +65536 bytes | Each Exec+Commit <=25% or +5 ms, whichever allowance is larger |
| Small-file first and repeated read passes separately | Included above | Each pass <=20% or +5 ms, whichever allowance is larger |
| Full historical verification per smoke/file case | Not used to reduce primary Store allocation | <=25% or +100 ms, whichever allowance is larger |

Use no absolute noise allowance for primary Init/replay sums. Compare available
host lifetime RSS, container lifetime total peak, and sampled temporary peaks
per history: candidate <=1.25 * baseline +64 MiB, alongside the absolute safety
caps above. These broad resource scopes are not phase-memory claims. Report CPU
and I/O even where no additional numerical gate is proposed. Missing required
gate operands prevent qualification; source inspection cannot manufacture them.

These gates reject a 50% slowdown for 10% storage improvement and do not adopt
the owner's conditional 1.5x example as a universal allowance. They establish no
Git-closeness tolerance. Git proximity remains **unmeasured**; there is no Git arm.
Do not relax these criteria retrospectively to rescue candidate qualification.

## Result and coverage contract

Unique non-overwriting run directories contain exact sanitized commands, source/
product/harness/workload/input/oracle/binary/image identities, dirty diff seal,
performance JSONL, independent verification JSONL, failure/pending state, logs,
resource/cleanup receipts and a file hash manifest. Frozen raw receipts are never
rewritten; generated summaries reference their source files.

Report storage reduction `1 - C_alloc/B_alloc` and elapsed increase
`C_time/B_time - 1`, with absolute integer bytes and elapsed nanoseconds. Mark
unavailable fields null with reasons. Do not sum overlapping CAS/COW/delta/codec
savings. Every result remains `admission_eligible=false` for release purposes.
Development acceptance, correctness, route, resources, custody and cleanup each
have separate statuses. Source review covers safeguards not exercised here;
all-family, 157-state, parser/fuzz/race/crash and conflict matrices remain omitted.

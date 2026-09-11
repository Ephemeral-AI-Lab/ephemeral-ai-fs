# Stage 3: eliminate quadratic frontier spill merging

Work in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`. Execute end to end: inspect,
freeze a bounded protocol, reproduce the scaling defect, implement the smallest
shared-code fix, measure, verify, commit intended files and update issue #111.
This prompt authorizes execution. Do not stop at a plan or launch unrelated work.

## Objective and accepted starting point

Stage 2 is accepted with the owner's K10 absolute-target waiver. Its final clean
comparison confirmed K100 Commit330.23→186.05ms, namespace251.34→111.54ms and
CPU316.76→178.58ms inside a100000-file namespace. K10 measured54.90ms Commit and
33.75ms namespace; preserve those values and the waiver, not a fabricated PASS.
The earlier2.2x drift was misidentified executables caused by hard-linked builds;
it is resolved, not an ongoing unexplained cache effect. Do not restart Stage2.

Stage3 targets `FrontierInodes::merge_pending` in
`crates/layerfs-workspace/src/changes.rs`. It repeatedly rewrites the entire
accumulated sorted spill when the bounded pending map fills: O(K²/B) record traffic,
where K is frontier entries and B is pending-buffer capacity. K100 never triggered
this path. Eliminate that mechanism without unbounded RAM or unsafe fallback.

Success means O(K log K) or better spill work, demonstrated with deterministic
counts, bounded resources, exact results and independent proof. Do not invent a
large-change millisecond target before measuring its baseline. Keep ordinary
K100 Commit near its accepted performance; <=200ms remains the engineering goal.

## Read first and preserve evidence

Read applicable parent/root AGENTS.md if present, benchmark/AGENTS.md,
docs/general/benchmark_rules.md and benchmark/fs-bench-pro/QUICKSTART.md.
In docs/roadmap/0.1/0.1.5/issue111 read:

- commit-stage2-terminal-results.md, contract and evidence pointer;
- commit-stage2-hardening-results.md and commit-stage2-read-results.md;
- commit-optimization-direction.md and commit-baseline-contract.md.

Inspect actual callers, policy and existing spill/reference tests in changes.rs.
Read shared format/admission context in hybrid_mental_model.md, spec.md and
compaction-removal.md. Current preparation HEAD68f3c4823; re-read actual HEAD.
Current product includes Stage2 bd9dc1600 and hardening259a80a4b. Control is this
accepted optimized product plus the declared dirty treatment, not pre-Stage2.

Record status, tracked diff and preexisting untracked hashes. Preserve all
compaction-removal work, issue112/issue113, web/ and unrelated files. No stash,
reset, clean, revert or broad staging. Use independently owned worktrees/snapshots
including the exact dirty treatment. NO hard-linked writable copies or shared
target directories. Check inode ownership, binary hashes and immutable images.

All prior evidence roots remain read-only. Especially:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-commit-stage2-terminal-evidence/20260912T180000Z`.
Read its pointer for exact layout. Copy needed scripts/harness into a NEW external
timestamped evidence root; never execute archived collectors/analyzers in place.
Create outputs exclusively, retain every attempt, and never delete/overwrite a
run to rerun it. Preserve historical custody disclosures.

## Implementation: understand first, then change one mechanism

1. Determine actual B from the live memory policy and existing accounting; do not
   hardcode the earlier estimated threshold. Record record width, buffer sizes,
   lookup/update behavior, duplicate-key semantics, reference handling and finish.
2. Reproduce whole-prefix rewriting with small-budget deterministic tests. Count
   records/bytes read and written, flushes, merge levels, live runs, scratch and
   temporary disk. Show the control's growth before changing it.
3. Prefer bounded size-tiered sorted runs, reusing existing192-byte records,
   anonymous journals, pending BTreeMap and ordering helpers where appropriate.
   Merge similarly sized runs so a record is rewritten logarithmically many times.
   Preserve updates to already-spilled keys and last-write/checkpoint/reference
   semantics. Finalization must also avoid repeated growing-prefix rewrites.
4. Bound descriptors, open files and buffers; use checked counts/offsets. Charge
   actual capacities and size_of values, not assumed ObjectId/tuple widths. Retain
   old runs until new output is complete and successfully installed. Handle failed
   writes/flushes, disk exhaustion, retry and cleanup without data loss.
5. Do not add workers, raise memory limits, change storage encoding, or fall back
   to the old quadratic merge under pressure. Trace every fallback that can be
   reached by this change. No general sorting framework or unrelated refactor.

Use existing tests rather than a new test framework. Keep focused commits and
adapt to other owners' edits; never revert them. Diagnostic counters may remain
test-only or isolated if permanent telemetry is unnecessary.

## Scaling and correctness qualification

Freeze a contract and evidence pointer before implementation/performance sampling.
At a minimum test B−1,B,B+1,2B,4B,8B with a small test budget, then one authentic
production-budget sequence crossing multiple flushes. For the100000-file public
workspace choose K from the measured B and available files; state limits if8B
does not fit. Use a separate valid synthetic fixture only when needed, never
relabel it namespace-100000. Do not substitute reduced-budget timing for production.

Vary ascending/descending/interleaved keys, new versus repeatedly updated keys,
clustered/spread changes, deletion, hardlink reference updates and reversion.
Check exact final canonical roots/records against the control or reference model,
complete changed bytes, unchanged samples, reopened root/content, and failure/
retry behavior. Extend existing bounded spill tests and public integration proofs.

Derive a concrete bound on record traffic from the algorithm BEFORE inspecting
candidate results; verify it across sizes. Each record should visit only the
bounded logarithmic number of merge levels. Report reads AND writes and final
merge traffic. Timings or a fitted curve alone cannot establish complexity.
Measure peak live spill bytes, descriptors/files, allocated scratch and temporary
coexistence. Separate component bounds from whole-process peak observations.

## Benchmark environment and execution

macOS owns SQLite, SDK/coordinator, canonical construction/admission/publication
and spool. Docker owns daemon/live workspace/FUSE/workload only, with existing
2CPU/2GiB/no-swap/256PID settings. No Docker SQLite or changed topology.

Build each independently owned source using:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
python3 benchmark/fs-bench-pro/shared/runner.py --build-image
```

Preserve checked exit codes/logs and source/product/workload/compilation seals,
binary SHA256 and immutable image ID. Verify executed identities before/after
samples. Builds own their measurement lock. Collector holds that same runner
lock once in the parent; no child double-acquisition or sensitive overlap.
No crates/tools/benchmark edits after build through collection without resealing.

Reuse the retained commit-baseline harness and collector, adapted in the new root
for declared spill-scale K cases. It is NOT a registered `--family commit_baseline`.
The host invocation is:

```text
<qualified-binary> commit-baseline <fresh-store-dir> <fixture-payload> \
  <live-container-id> <scenario> <cell>
```

Inspect the copied collector's actual CLI/sequence format before running; freeze
the adapted harness identically for control/candidate. Run the real family
verification entrypoints with matching identities; proof-only families require
verification, not performance. Current CLI uses --verification where supported;
read help and registry rather than invent flags. Reuse unaffected #104 evidence.

For original namespace-100000 use100000 files/1000 data dirs/500000000 bytes,
pseudorandom content, digest
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.
Validate full inventory/content plus file0640/dir0750/mtime1700000000000000000ns,
including root. Fresh public Init+fork creates each independent Store outside
Commit timing; no output Store clone substitute. Time the full public Commit.
Report edit time separately and per-sample edit+matchingCommit sums correctly.
Cache remains declared uncontrolled for Commit; no cold inference or warmup Commit.

## Practical timing policy: minor misses do not derail this stage

Hard gates: correctness, authentication, no quadratic spill/fallback work,
declared resource bounds and evidence integrity. These are never waived by timing.

K10's old <=50ms/<=31ms absolute gates remain owner-waived. Do not resurrect them,
retry for a prettier median or reopen Stage2. Compare ordinary cases with current
accepted main. K100<=200ms remains the goal; a minor isolated exceedance is a
reported warning, not automatic failure of a successful scaling repair.

Recommended prospective regression policy, freeze before collection: n3 fresh
matched pairs per selected case, C1,T1;T2,C2;C3,T3. Call a timing regression material
only if the median paired wall slowdown exceeds max(15% of control median,3ms)
AND at least2/3 pairs slow down. For user+system CPU use max(15%,1ms) analogously.
Report every sample, breach, absolute miss and allocator/resource effect. This
explicit policy implements the owner's tolerance for minor misses; it changes no
historical verdict or unrelated benchmark rule. Investigate material regressions.

Small ordinary cells should not need spill: keep nochange/K1/K10/K100 as a bounded
regression screen, not an extra speed contest. For large spill cases, require
the proved work reduction and report paired time/CPU; set any absolute target
prospectively from the baseline, never after candidate results.

No timing-based replacement. At most one complete affected-pair replacement for
demonstrated infrastructure invalidity; retain original attempts. A material fix
requires a versioned treatment and new seal, not selective retries.

## Init overlap and completion

Trace whether Init uses the changed accumulator/helpers. Document shared versus
Commit-specific paths. Run affected Init correctness checks. If the change touches
Init's measured bottleneck, include a separately frozen cold Init comparison with
the existing verified-cold acquisition rules. Otherwise report no expected Init
speed effect. Shared code does not imply shared acceptance evidence; no requirement
to wait for every Commit stage before measuring a plausible Init benefit.

Deliver `commit-stage3-results.md` with baseline defect, exact fix, algorithmic
bound and counters, production-budget proof, memory/disk accounting, paired
timings/CPU, ordinary-case warnings/material-regression verdicts, correctness and
independent verification, exact seals and append-only evidence manifest.
Commit only intended product/tests/docs; update #111 with #115/#108 and provenance
links. Stage3 completion is elimination/proof of this quadratic mechanism, not
closure of every performance issue. Keep K10 waiver, unresolved Stage2 risks and
cold Init<=2.7s visible. Do not claim untested exhaustion paths are proved.

Finish this stage; do not bundle edit-barrier optimization, metadata history work,
new caches, release/tag/deployment or unrelated changes. Next queued task is fresh
edit-stage attribution, aiming to explain>=90% of wall before setting its latency
target. If a genuine blocker prevents Stage3 completion, state the exact failed
condition and evidence rather than manufacturing PASS.

# Issue49 restart handoff — integrate, measure, continue to terminal PASS

The user stopped task `01a0741d-2c37-7da3-b600-fa0f198662ae` after stalled progress and requested a new GPT-6 Astra task at medium effort. This is continuation of its implementation, not a fresh architecture project. The predecessor is interrupted/idle. No rewritten create-100 performance sample exists.

## Exact custody and validation boundary

- Predecessor branch: `codex/issue49-live-workspace`.
- Predecessor checkout: `/Users/yifanxu/.codex/worktrees/issue49-live/layerfs`.
- Last committed component-tested base before handoff capture: `162d7b8165e77c78b9600b5980685206c29bcb6e`.
- A WIP checkpoint accompanying this note preserves all 15 stopped-task modified/new files. It is NOT an accepted implementation or a tested full product. Resume from that checkpoint; do not reset to main and repeat extraction.
- Byte-for-byte backup and SHA256 manifest of the interrupted changes: `/Users/yifanxu/.codex/backups/issue49-stopped-20260906-125517`.
- Existing component evidence: `docs/roadmap/0.1/0.1.3/issue49-live-workspace-ledger.md` and the predecessor's `benchmark-results/issue49/` directories. Keep original artifacts/source identities. They are local evidence, not necessarily Git-tracked.
- The last host `cargo +1.85.1 check -p layerfs-workspace --no-default-features -j2` returned success before the final callback-guard edits, with unused BackingOwner warnings because it is not wired. The last edit added callback admission/guard retention to FUSE and was formatted but has no subsequent check recorded. Do not assume the WIP compiles on Linux.
- The report of hanging is the user's observation, not a diagnosed program deadlock. Do not spend a new research cycle diagnosing a hypothetical hang. Check for orphaned tools only if they actually block a lock/build; terminate only verified task-owned work, never unrelated processes.

## What already exists — use it

Committed portable core contains shared identity/metadata, PieceTree/backing ownership, exact prepared write/truncate with revision validation, namespace acquisition/create/mkdir/symlink, pins/reclamation, and frozen construction inputs. Existing HostSpool physical algorithms and CandidateInputs/StableFileInputs/frontier/journals are adapted. fuser has the narrow single-receiver/receive-capacity patch. Owned FUSE read/write replies and LiveRuntime/Scheduler/OperationGate components have native/Linux component evidence. Do not redo these extractions or the completed Commit producer refactor.

The latest WIP adds:

- `crates/layerfs-fuse/src/live_wire.rs`: bounded resolved node/range/fact framing; does not represent host POSIX mutation replay.
- `crates/layerfs-fuse/src/live_transport.rs`: BackingServer/BackingConnection, async frames/capability check, physical-job admission and disconnect behavior.
- `crates/layerfs-workspace/src/live_backing.rs`: BackingOwner immutable lookup, existing HostSpool reserve/append/read/check/release and fact groups.
- CallbackGuard/admit_callback additions in port.rs/filesystem.rs; guards retained with read/write replies; incomplete integration.
- LiveRuntime::shared, live-state reservation plumbing, module exports, crate feature/lock changes and small visibility adapters.

These are unfinished code, not approved protocol/resource guarantees. Review only concrete defects needed to wire the next path. In particular, confirm error/length/ownership handling, complete fact-group admission, lifetime/join behavior and callback guard placement while connecting real callers; do not suppress dead-code warnings instead of connecting callers.

## Immediate milestone — three blockers, then one sample

1. Wire real daemon mount startup to a shared live owner and route the unchanged create workload's callbacks to it.
2. Connect immutable acquisition and HostSpool reserve/append/read through the bounded backing connection. Resume exact prepared operations after I/O; do not block all filesystem workers or replay a mutation.
3. Connect Commit/observation/end to frozen inputs and exact checkpoint installation on the same live owner.

Then run one sealed `tiny-bulk-create-100-mixed-v3` sample with `--setup clone`, even if TARGET_MISS is expected. The previous task spent too long producing preparation milestones without product feedback. Your next substantive deliverable is that connected path and its measurement.

Defer general extraction, broad supervision consolidation, optional capture work, API cosmetics, scale tuning and redundant suites until after that first sample unless they are a concrete safety or integration dependency. A branch-local development sample is not terminal acceptance; later supported-surface, mapped-file and concurrent SDK/Commit obligations remain. Do not weaken required behavior of the measured path, add a benchmark-only engine, hide cleanup or present partial functionality as complete.

## Authoritative architecture and instructions

Read this note, the current implementation ledger, and:

- `docs/roadmap/0.1/0.1.3/issue49-fuse-exec-implementation-plan.md`
- `docs/roadmap/0.1/0.1.3/fuse-exec-rewrite-spec.md`
- `docs/roadmap/0.1/0.1.3/fuse-exec-redesign-requirements.md`
- `docs/roadmap/0.1/0.1.3/fuse-exec-rewrite-checklist-map.md`
- `docs/roadmap/0.1/0.1.3/fuse-exec-100-workspace-resource-review.md`
- `docs/roadmap/0.1/0.1.3/issue49-shared-producers-plan.md` (completed historical implementation)
- `benchmark/AGENTS.md`, `benchmark/fs-bench-pro/QUICKSTART.md`, `docs/general/benchmark_rules.md`.

Issue https://github.com/Ephemeral-AI-Lab/layerfs/issues/49, parent #47 under #46/#39. Preserve newer completed main work, but merge updates without overwriting this checkpoint. Do not migrate/cherry-pick the research Exec patch from task `01a072c8-2e80-7982-966b-df26da9fdeb5`; learn from its mechanisms and rejected experiments only.

Mental model:

```text
application / ordinary launcher           SDK live operation
             |                                  |
         Linux FUSE                             |
             +------> ONE shared live owner <---+
                            |
          owned input -> validate/apply/reply
          missing input -> park/acquire/revalidate/resume
                            |
                bounded bytes + resolved facts
                            |
                 existing HOST physical backing
                            |
          Commit operation cut / frozen changed inputs
                            |
             existing shared builders / SQLite publication
                            |
                 checkpoint SAME nodes and handles
```

CAS/CDC/COW/PieceTree/rope/extent algorithms, deduplication, sparse/zero behavior, aliases, old snapshots and exact publication recovery remain. No kernel rewrite, native-directory mirror, new generic effect/actor/lease/reconnect framework, or family/size/density engine route.

Core depends on portable content, not Store/SQLite/Unix files/FUSE replies. Host owns physical backing/canonical publication. One live authority means SDK and FUSE do not independently mutate separate namespaces. Local success needs validated/applied state and retained resources; reserve before mutation/allocation. ACK only complete groups with referenced ranges retained. Retain old-read/range ownership through partial failures and checkpoint.

Backing waits release state locks and filesystem workers. Apply revalidates exact inode revision; stale appended ranges stay charged until safely released. Do not repeat append against new state. Current session/diff metadata reads require no payload fence. No global registry/Store mutex across socket I/O, spawn, drain or join. Shared service buffers must not be permanently captured by stalled peers.

## Corrected Commit contract — supersedes original handoff Busy wording

Multiple commands/processes and ordinary open handles may remain alive while Commit succeeds. Shell/Exec is a launcher/benchmark surface, not filesystem quiescence. Remove has_executions-based Commit rejection and command-exit/ordinary-fd-close requirements from the final path; keep process accounting only for actual launcher/resource/teardown duties.

A minimal correct boundary may queue conflicting mutations, drain admitted operations, capture/acknowledge state, publish/install under a bounded mutation pause, then resume queued writes as subsequent dirty state. Do not kill commands, unmount, reconstruct Workspace, or require process exit. Preserve cwd, handles and open-unlinked backing. Commands can span snapshots; no whole-command transaction is promised. Correct supported kernel dirty-page/mapping behavior explicitly; do not remove guards without replacing consistency. Legitimate actual failures, head conflicts, resource/deadline errors and exact pending-publication recovery remain. End/discard is separate from Commit.

## Exact benchmark / targets

Primary: `tiny-bulk-create-100-mixed-v3`, seed1: 1,000 created files /100MiB including one50MiB file; separate unchanged200-file/1MiB witness. Do not change to2,000files or weaken names, metadata normalization, sync, bytes or generator. Workload uses new bulk directory, DIRECT_IO create handles, metadata normalization and root fsync. Use its actual path for the first measurement; broader final contracts remain open.

Working ambition: complete create approximately0.7000–0.8000s; indicative Exec0.3000–0.4000s; preserve finalized Commit efficiency. Formal parent result: both same-source tier100 create/delete full lifecycles strictly<1.0000s using unrounded receipts. About0.0500s from the preferred band should not cause prolonged marginal tuning. No new hard per-phase target.

Finalized historical reference: create Exec0.9172s /Commit0.3587s /total1.2914s. Research reference:0.4792s /0.4718s /0.9680s, different product. Never combine independently measured phases into claimed success. Display seconds with four decimals; preserve raw values and identities.

## Fast iteration and environment

Host macOS: SQLite, SDK/benchmark coordinator, physical spool, canonical construction/publication. Docker Linux: daemon/live owner/FUSE/workload. No container SQLite/coordinator or host-data mount. Container2CPUs/2GiB/no swap/256PIDs, host resource scope separate. Native component checks supplement real FUSE performance.

Use the existing shared measurement lock `$TMPDIR/layerfs-infra-measurement.lock`. Reuse compatible protected preparation and Cargo/Docker caches; preserve old artifacts. In a newly created worktree, the runner's HOST_ROOT is relative to that worktree: do not pretend old caches are automatically discovered. Reuse compatible owned fixture data only through valid runner custody, or let setup acquire its required master normally. Never modify identity sidecars or force mismatched cache reuse. Do not reset/prune Docker, delete masters or use routine fresh preparation.

From the assigned implementation worktree:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py --build-host
export LAYERFS_BENCH_IMAGE="$(python3 benchmark/fs-bench-pro/shared/runner.py --build-image)"

# Prepare once or ensure changed compatible input. Perf also acquires preparation.
bash benchmark/fs-bench-pro/families/tiny_file_churn/setup.sh \
  --topology host-store --case tiny-bulk-create-100-mixed-v3 \
  --seed 1 --setup clone --image "$LAYERFS_BENCH_IMAGE"

# One complete sample after a substantive change.
bash benchmark/fs-bench-pro/families/tiny_file_churn/perf.sh \
  --topology host-store --case tiny-bulk-create-100-mixed-v3 \
  --seed 1 --setup clone --perf-fast --image "$LAYERFS_BENCH_IMAGE"
```

Build only matching required artifacts, using caches. Setup is preparation-only; do not run it redundantly before every sample. Clone is checked closed-quiescent-byte-copy into a fresh disposable sample; runner owns container/FUSE startup/cleanup. Preserve master-unchanged checks. Performance watchdog120s product/130s outer is diagnostic allowance, not threshold. Record complete Exec/Commit/End and nested-timer limits, host/container CPU/memory, retained backing, setup and cleanup.

One hypothesis -> one focused change -> necessary changed-seam checks -> matching builds -> one serial create100 sample -> retain/replan. No favorable-repeat search, automatic seed multiplication, routine tier500/full-family runs, or repeated passing suites without relevant changes.

Independent proofs final-only after stable applicable performance: exact receipt source/input/image/case/seed through verify.sh/verify-selected.py;45s work/59s hard each, preparation separate, sampled coverage/omissions explicit. Use same-source delete100 control at stable candidate. Physical100-workspace exercise is NOT a current gate; use resource accounting/lower-volume inference and label capacity unverified.

## Persistence and terminal PASS

Keep going until terminal PASS. REPLAN/NO-GO rejects an approach, not the task. Preserve failed evidence, state the concrete finding, choose the next smallest structural correction and continue. If request counts drop without elapsed benefit, attribute real dispatch/wait/work/resumption rather than inventing protocol machinery. Do not stop at another generic extraction milestone.

Terminal PASS requires connected supported shared-owner behavior, corrected concurrent-command Commit, transferred real callers and removal of obsolete paths, same-source strict tier100 create/delete performance, required focused checks and final bounded sampled proofs, cleanup/ownership/provenance and honest remaining platform/scale limits. A developmental TARGET_MISS is useful input, not final success. No15secondfamilyPASS, mixed-source arithmetic, unit-only prototype or inferred100-workspace scale qualifies as terminal PASS.

Preserve correctness and actual resource bounds. If a real external blocker makes progress impossible, report its exact evidence; never invent success. Otherwise continue promptly and communicate concrete integration progress at least once per minute. Commit reviewable increments, publish/integrate authorized completed work through normal Git operations, and update issue49. No force push, unrelated edits or automatic parent closures. No new architecture research cycle or additional user approval request for already-authorized routine work.

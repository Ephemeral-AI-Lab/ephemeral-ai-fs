# Issue #100 full157 execution contract, revision 1

Frozen before measurement on 2026-09-09. Issue #100 and the current owner request
supersede the earlier smoke-only scope. This is exploratory product optimization,
`admission_eligible=false`; no release-admission PASS, release or tag is authorized.

## Sources and fixed treatment

Candidate starts at `6d87b6e72` (codex/v015-small-content), preserving the complete
v0.1.5 implementation and reviewed documents, in isolated `codex/issue100-full157`.
The initial candidate includes the shared oversized exact-CAS dispatch correction.
Control product is byte-identical to released
`101fa273d815f3aaedb0e06ba0de7b0777d83def`. Both arms carry the same current benchmark
harness. Source commit/tree/dirty patch, framed product and combined source seals,
harness file manifest, host binary hash and actual image ID are retained per arm.
Historical 184,582,144-byte allocation is context only, not the matched control.

Fixed candidate policy: nonempty regular content strictly below 131072 bytes uses
whole-file SmallContent; empties stay compact; larger content retains CDC 8/16/32
KiB and extent locality. Small FULL/one-level DELTA, one eligible known FULL base,
Zstandard level 3/windowLog 18/workers 0 and specified flags, pack grammar, limits
and authentication remain as in spec.md. New SQLite pages remain 4096 bytes;
supported older layouts/nonpromoting opens remain. Preserve shared Init/Commit,
#95 reuse, #98 SQL coalescing/staging handoff and ordered spill read-ahead <=64 KiB.
No codec, selection, page or threshold sweeps; no GC/repacker or dependency changes.

## Fixture, operation and ordering

Reuse deepseek-full in shared/runner.py and storage_smoke.py, existing compiled
importer and host/Linux build entrypoints. Fixture directory:
`/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data`.
Manifest SHA256 `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`;
source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`.
Use all 157 checkpoints in manifest order. Existing preparation validates Git tree
manifests, changed blob Git identities, exact input receipts and independent SHA256
oracles. Reuse those prepared inputs; never recreate or modify them for this task.

Run control performance then same-Store verification, initial candidate performance
then same-Store verification, then a final candidate only if substantive changes
invalidate the initial result. One history per arm; checkpoints are dependent
observations, not independent repetitions. Keep every attempt; no outlier removal
or unchanged reruns for a better number. No controlled cold cache claim.

Each arm creates a fresh host Store, native empty LayerStack/Branch/Workspace,
then 157 public Exec/FUSE full-file imports and public Commit attempts. Retain
all mappings and actual Created/UpToDate outcomes. Preserve deletion/type/mode/
symlink semantics and existing normalized regular/directory mtime 1000000000;
symlinks retain the existing timestamp rule. No SDK editing substitute.

## Prospective working criterion (not user-approved)

Propose keeping an optimization when it reduces final allocated Store storage
and retained-history growth with <=10% regression in median and nearest-rank p95
Exec/save, Commit and paired Exec+Commit latency, <=10% total performance and
historical-verification wall regression, and <=10% recorded host lifetime peak RSS
increase (allow 8 MiB absolute RSS noise). Compare final against matched released
control and each incremental optimization against its initial candidate. A >=1%
final-allocation reduction is the proposed worthwhile-benefit floor. These are
conservative working criteria, not release gates or a numerical PASS authority.
Report all actual tradeoffs and misses. Tiny timers may be noisy; do not hide them
with retrospective thresholds. Report min/median/p95/max and sums, n=157, and slow
checkpoints; no confidence or per-checkpoint p99 claims from this single history.

## Topology, budgets and measurement

Host macOS owns Store/SQLite, coordinator/SDK, canonical construction/publication
and physical spool. Linux Docker owns live core, daemon, FUSE and workload only.
Container: 2 CPUs, 2 GiB, no swap, 256 PIDs; existing device/security configuration,
no data-sharing mounts. Preserve original watchdogs: 300 s per operation/command,
14400 s each performance and verification, 120 s setup/cleanup, 14400 s immutable
input validation; builds 900 s and 2 Cargo workers. Store <=16 GiB, runtime/spool/
staging <=16 GiB, owned directory <=32 GiB, host free >=50 GiB and sampled RSS <=8
GiB. Integrity/oracle/route failures, OOM/swap and incomplete cleanup fail the run.
Runner owns existing measurement lock; do not double-acquire. Reuse shared Cargo
and BuildKit caches; no clean/prune/new target directories or stale seals.

Public operation-local monotonic timers include Exec/output drain and Commit
finalization; Init is separate. Preparation, transfers, allocation/census, resource
observations and verification remain outside those timers. Total case/work wall
includes its actual orchestration and is reported separately. Record allocated
Store plus applicable sidecars initially, at every checkpoint and finally; apparent
length, SQLite pages/freelist/table/index/overflow/slack are separate quantities.
Staging/spool, host CPU/current/lifetime RSS and container boundary/lifetime memory
categories retain their actual scopes; missing phase-local peaks are unavailable.
Build/setup/preparation/transfer/cleanup and verification costs are separate.

After performance allocation freezes and clean coordinator closure, preserve the
performance manifest including Store SHA256. Read-only census may run without
mutating Store. Before a new verifier coordinator opens the SAME path, validate
that digest. Retain identity/path/size evidence. Verify every mapped checkpoint
against the original full path/type/mode/symlink/complete-byte oracle via public
FUSE reads. Verification lifecycle writes occur only after the frozen window.
Require successful End/coordinator shutdown/container removal. A rebuilt history
or separate Store is never proof of the measured Store.

## Diagnosis and outcome

Attribute initial storage: small FULL/DELTA frames and physical FULL bases, exact
CAS reuse/fallback, large CDC/PREFIX, metadata/locators/indexes, pack headers and
slack, SQLite overflow/free/allocated pages; staging separately. Confirm actual
SmallContent and pack-v3 DELTA. Use existing receipts and post-window census before
adding targeted instrumentation. Choose the largest evidenced shared cause; no
metadata-anchor repetition without new evidence (prior attempt added 76 bytes).
Keep only justified changes, revert rejected experiments and retain their evidence.
Report sources, commands, artifact paths, all results and unrun qualification;
update issue #100. Broad Cargo/Clippy/doctest/family/release suites remain unrun.

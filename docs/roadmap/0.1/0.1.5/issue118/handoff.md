# Issue #118 — current progress, remaining work and execution contract

## Owner mandate and current status

**OPEN / INCOMPLETE. Finish the remaining implementation and independent qualification through terminal acceptance. Continue beyond attribution, a root-cause report, a proposed repair or one completed stage.**

This update incorporates the owner's progress review and subsequent instructions. It supersedes obsolete completion claims, earlier NOT_RUN lists and the original absolute cold-Init requirement. The owner explicitly requests subagents, aggressive infrastructure simplification, fast iteration, immediate diagnosis/repair of slow or unproductive tests, and no repeated tuning/reruns over minor performance misses.

Work on main in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs`. The review observed HEAD `9f8494b70`, plus an uncommitted diagnostic probe in `crates/layerfs-workspace/src/file_io.rs`. Inspect current status and preserve others' work. No external-library patches, release, tag, deployment, unrelated cloud work or new automation/task is requested.

## Completed work to reuse

Fresh evidence root: `benchmark-results/host-store/issue118/20260912/`.

| Work | Recorded result and scope |
|---|---|
| #117 infrastructure | Shared incremental Cargo targets, Docker dependency cache, immutable independent binary archives, corrected source-mtime freshness, fewer readiness/image operations and guarded retention; focused checks, smoke and small paired example PASS. Preserve these fixes. |
| Stage3 spill internals | Highest-tier growing-prefix repair, failure/retry, actual capacity/FD/disk accounting, production-budget components and reduced-budget Workspace proofs PASS. Default-budget public K32000 remains outstanding. |
| 100000-file physical reservation failure | `a36c60891` / `440584938`: prepared slot 184→112 B and corrected ownership of input-association charges. Original public case PASS 4.708 s plus independent proof; no 2 MiB quota increase. |
| SDK edit preparation / reopen | Attribution, removal of unnecessary SDK lookup/export and prior-fact work, n3 both-cache edit/Commit qualification, independent/mounted proofs, and real K1000×33 reopened sequence PASS. Existing linear catalogue-header/FK validation remains explicit. |
| Cold Init | Qualified with the owner's 2.7 s absolute WAIVER. Explicit paired wall WARN +1.039%, CPU improvement −2.855%; four namespace tiers/five proofs. No recovery of the historical 3.420 s absolute time is claimed. |
| Fsync optimization | `f8fa59fab`: 100-round n3 medians 4.492→2.677 s, 8007→1503 backing exchanges. Full unrelated500 screen PASS 13.791443406 s. Same 5000 writes/500 MiB/6000 fsyncs/500 Commits; 1 MiB spool peak. Fifteen focused checks, two selected proofs, four mounted reliability proofs and two actual Docker SDK tests PASS. |
| Remaining 18 shared selections | 18 complete performance samples +18 matching independent proofs PASS on `fsync-qualified`. The frozen plan uses n=1 each and no paired subset; do not call this n3 regression evidence. Tiny-create100 sample PASS 0.983329501 s. |
| Ordinary full157 | 157/157 construction steps PASS; complete performance command 766.577 s. Same measured Store verified against all157 original oracles, 712.570 s. Frozen measured Store, physical census and matching Git proof exist. |
| Historical access | 11 performance cases +11 separate proofs PASS on their recorded producer cohort. Complete performance commands 2.592–3.190 s; proofs 2.337–2.758 s; all below15 s with cleanup PASS. |
| #116 first repair | `ead812e78`: redundant 4096-edit rejection removed across adapters; counter widened with checked arithmetic; real budgets retained. Public Probe A accepts10000 calls. Separate approximately5000-edit tests cover readback/Commit/reopen; Probe A itself only checks acceptance. |

Existing evidence remains valid for its recorded producer and scope. Rerun it only when a relevant change, failure, missing required coverage or unresolved applicability warrants doing so.

## Required corrections to the acceptance record

Complete one short correction pass; do not turn it into another provenance campaign.

- Full157 and all22 access receipts name binary `408f3e2a762e059a5efc8f3634355ad259de16a95624427610e8f94fb7b7fc64`, source `0574853a…`, product `6e9e2840…`, image `08082f1a…`. `full157-candidate-1/identity.json` and the producer's `binary-archive/<sha256>/` receipt are authoritative. `terminal-outcome.md` incorrectly assigns them to later binary `6693224e…`. Correct the cohort, then make a bounded runtime-applicability assessment; a labeling mistake alone does not require rebuilding history.
- Different executable hashes are not byte-identical. A reused `final-treatment/` path does not prove which bytes ran earlier. Remove the false byte-identity statement.
- The default `FrontierInodes` pending capacity is **B=15873**, calculated at `changes.rs`'s frontier construction. The separate tree-work batch size128 is not this threshold. K5000/K5461 do **not** prove the default frontier spill crossing.
- The10000-edit Probe A asserts no rejection and then discards the workspace; it does not assert final bytes or Commit/reopen. Correct its scope and extend an existing focused proof to cover10000 edits with exact final contents and Commit/reopen.
- Keep the18 single-sample qualification rows distinct from matched comparative evidence. Freeze only the further comparisons actually required for changed paths.
- Preserve the Git allocation-layout WARN: measured Git allocated56197120 B versus older recorded56373248 B, with unchanged apparent/pack bytes. Preserve the earlier cold/Commit WARNs and all failed attempts.

## Remaining sequence and targets

### A. Finish #107 ordinary-storage improvement

Current attribution: LayerFS allocated83935232 B, apparent83525632 B. Matching live Git allocated56197120 B: LayerFS excess27738112 B (about49.36%). This is an allocated-storage comparison, not a pure compression ratio or a foreground-latency comparison with offline Git packing.

The current #107 objective still requires a prospective quantitative target and demonstrated reduction in ordinary allocated Store bytes. Attribution alone and an unspecified future experiment do not complete it; no owner waiver of this objective has been given.

1. Use the existing census to select the largest evidenced avoidable cost. Inspect the common caller/path, then choose the smallest justified change.
2. Declare the mechanism, numerical improvement target and latency/resource constraints **before** implementation/measurement. The earlier ≥1% proposal is not a pre-existing gate or achieved result; choose and freeze a meaningful target from the evidence.
3. Develop against one representative focused case. Reject ineffective changes promptly; retain failed experiments. Avoid speculative codec/format redesign and endless attribution.
4. When stable, build once, run the needed matched checks, and construct/verify the affected ordinary history. Reuse a proven-applicable archived baseline; no automatic n3 full157 replay. Qualify affected historical access and cold Init only when the changed path warrants it.

Savings must arise from ordinary Init/Commit. Keep4KiB pages, exact CAS, authentication, pack/DELTA compatibility and legacy reads. No compaction, VACUUM, post-workload repacking, fixture rewriting, quota increase or external-library patch.

### B. Complete #116 bounded capacity repair and default public spill

The first audit and counter repair exist. Reconcile the audit with final source and the corrections above; publish the scoped implementation decision before further restriction changes. Do not restart the entire audit.

The remaining problem is internal and actionable: a normal small splice costs three128-byte piece nodes; the2MiB piece budget permits about5461 files of that shape. K5000/K5461 pass, K6000/K32000 fail with the piece-allocation error. The proposed compact pending-splice representation is **not implemented**. More ceiling probes or a larger timeout cannot complete it.

- Implement the smallest bounded compact representation that demonstrably fits, reusing existing ownership and the existing piece tree for more complex edits. The suggested40–56 B/file figures are hypotheses, not measured allocations or promised capacity.
- Trace every affected path: SDK, ordinary FUSE writes, range reads, capture/wire, host backing, checkpoint/Commit, truncation, sparse ranges and aliases. Avoid new raw-pointer/manual-lifetime mechanisms.
- Keep actual retained and transient allocations accounted, including compact→tree conversion while old readers, captures or retries retain state. Preserve failure atomicity and checked arithmetic. Do not equate the logical2MiB piece charge with whole-process RSS.
- The separate96MiB facts/publication and128MiB shared live-state budgets can become binding next. Compact encoding or bounded streaming must preserve a coherent generation and acknowledgement semantics. RSS samples of a pending set do not justify lowering accounting multipliers/floors; any revised accounting needs a simultaneous-ownership/allocation proof.
- No whole growing-file reconstruction per edit, new global serialization, unbounded compact list, additional workers, quota increases or automatic intermediate Commits.
- First qualify focused mutation, conversion, retained-reader, pressure, cancellation and retry cases. Then execute the real `init_namespace / namespace-100000 --sequence 32000` route with the original100000-file/500000000-byte fixture and unchanged default budgets, one requested Commit and independent reopen/content verification. It must reach the required two B15873 capacity crossings plus a third partial run. K5000 cannot substitute.

**Owner concurrency requirement — hard acceptance:** the live daemon/FUSE workspace can serve multiple running commands. A successful Commit must preserve their processes, the workspace session/mount, open descriptors, mappings, cwd and continued writes. Keep other workspaces isolated. Preserve the existing filesystem-operation Commit barrier and resume ordering; do not replace it by terminating/restarting commands, waiting for all commands to exit, discarding the workspace or introducing hidden Commits. Measure same-workload throughput and barrier latency as well as edit/Commit time. Existing non-remote Busy behavior is a separate compatibility scope, not an excuse to regress the live path.

Reuse and extend `crates/layerfs-sdk/tests/live_docker.rs`'s existing two-command mapping/ordinary-write tests so the new compact form and promotion path are actually exercised. Keep failure/recovery behavior explicit. A Rust memory-safety argument alone does not prove concurrency or resource correctness.

### C. Complete the pinned package workflow

Prepare the necessary pinned package runtime/toolchain as part of the workload image/setup; reuse qualified daemon/FUSE artifacts when compatible. A missing package manager is an internal setup requirement, **not an unrelated-work exemption**. Do not substitute an execution-only smoke.

Select a deterministic project/lockfile and declare lifecycle-script treatment. Run clean install and a representative pinned update, each followed by the requested Commit/reopen and manifest/content/metadata plus meaningful application/module-resolution verification. Keep network acquisition pinned and separate from claimed product timing. Preserve the approved host Store/SQLite versus container workload topology. No prior npm failure is presumed.

### D. Focused final qualification and terminal reconciliation

Rerun only affected checks after final code changes. Resolve the source applicability of older cohorts explicitly. Keep real K32000, package workflow, required boundaries, storage saving and independent proofs mandatory. Update the local ledger and linked issue outcomes with current commits, exact artifacts, command costs, retained failures, permitted WARNs and owner-WAIVED items. Close this umbrella only when the checklist below is satisfied.

## Fast iteration: mandatory working method

- **Use subagents** with explicit file/responsibility ownership for independent development/review; preserve shared edits. Only independent read-only work or ordinary development may overlap. All builds, benchmarks and resource-sensitive tests are serialized under the runner-owned lock.
- Use `inspect shared path/callers → one meaningful reproducer → minimal fix → focused check → one selected public screen → stable qualification`. Prefer existing helpers, stdlib and cases. No new collector, worktree, diagnostic framework or giant test suite for each change.
- Prefer seconds for warm focused checks (roughly≤10 s is a development goal, not a product acceptance gate). Record compilation/setup separately. Reuse Cargo dependencies, fixtures and verified artifacts; use the existing `--build-host` / `--build-image` entrypoints after relevant changes. No rebuild for docs-only changes or one complete target directory per revision.
- **Immediately diagnose slow or unproductive tests.** Before each potentially long command declare purpose, expected scale, budget, progress signal and stop condition. If a focused check stalls, rebuilds unrelated dependencies, waits on an impossible condition or exhibits unnecessary growing-set work, inspect it immediately. Stop only your own unproductive attempt safely, preserve its log, and fix the test/harness/implementation cause before continuing. Do not blindly wait out a huge inherited timeout or simply enlarge it.
- Preserve the test's claim when fixing slowness. Small/reduced-budget cases are development proofs, not renamed full-scale acceptance. Never remove assertions, authentication, fsyncs, bytes/files or requested operations to get a faster PASS.
- **Recognize legitimate expensive qualification.** Full157 previously took about12.8 minutes for construction and11.9 minutes for verification, with roughly82 s input preparation. Run it once relevant code stabilizes and the proof is needed; use its existing per-step progress. Update progress at least every60 s and investigate stalls. Its legitimate duration is not permission to run it repeatedly during development or shrink its contract.
- Inspect compact aggregate receipts, not thousands of per-operation rows or fixture listings. Avoid repetitive seal reconstruction, report churn and counter-only probes after the causal mechanism is established. Repair the evidenced bottleneck and move to qualification.
- Build/freeze one treatment and one concise case/arm configuration when the change stabilizes. Use fresh output paths and actual source/product/input/image identities. Never overwrite an attempt or mutate a frozen baseline. No unchanged reruns for a nicer median and no slower-arm-only retries.

## Performance disposition: avoid minor-miss ping-pong

| Requirement | Disposition |
|---|---|
| Ordinary wall preservation | Prospectively use n3 alternating pairs when a comparative screen is required. Material regression only when median paired slowdown >max(15% of control median,3 ms) **and** at least2/3 pairs are slower. |
| Ordinary CPU preservation | Same rule with max(15%,1 ms). Preserve any stronger applicable existing requirement. |
| Minor ordinary misses/noise | Record **WARN and proceed**. Do not alternate tiny patches, reverts and retiming to chase small differences. No extra25% improvement requirement for already-optimized hardening. |
| Cold Init≤2.7 s | **Owner-WAIVED.** Preserve verified-cold acquisition and honest measured improvement/tradeoffs. Do not reopen this absolute target. |
| Stage2 K10 50/31 ms | **Owner-WAIVED.** |
| Unrelated-history500 | Unwaived **<15 s** product timer; latest full screen13.791443406 s passes on its recorded treatment. |
| Tiny-create100 | Unwaived **<1 s**; current sample0.983329501 s passes. |
| Historical access | All11 performance cases and11 separate proofs retain complete **15 s** envelopes, including required preparation, receipts and cleanup. |
| Correctness/authentication/resources/custody/concurrency | Hard gates. No noise allowance or timing waiver applies. |

`--collection-mode` completion is not a waiver of a stronger hard gate. If an unwaived hard target fails, record FAIL, diagnose its cause once and repair meaningfully before recollecting the affected comparison. Do not hide it as ordinary noise, but also do not rerun unchanged code until it happens to pass. Only an explicit owner decision changes such a target.

## Artifacts, topology and preservation

- Read applicable AGENTS.md, benchmark rules, QUICKSTART and current CLI help. Canonical command/proof recipes for full157/access are in `docs/roadmap/0.1/0.1.5/issue118/history-handoff.md`; do not automatically execute its old comparator choice.
- Current documents: `handoff.md` (this contract), `execution.md`, `terminal-outcome.md`, `issue116-audit.md`, `issue116-piece-ceiling-rca.md`, `fsync-results.md`, `cold-namespace-results.md`. The correction list above supersedes contradictory report statements. Historical evidence retired by the owner is not current raw proof.
- Artifact roles: `qualified-v2` is pre-fsync control; `fsync-qualified` is the qualified `f8fa59fab` treatment (binary440ae2c4…); full157/access ran archived408f3e2a…; the later `final-treatment` binary is6693224e…. Resolve complete identities from receipts. Never infer byte identity from a path or shared prefix/HEAD.
- macOS owns SQLite/Store, SDK/coordinator, canonical construction/publication and spool. Docker owns daemon/FUSE/workload, under the existing2CPU/2GiB/no-swap/256PID benchmark policy. No container-owned Store or fallback topology.
- The measurement lock is `Path(TMPDIR)/layerfs-infra-measurement.lock`. Runner/verifier/build wrappers acquire it themselves; **no nested locks**. Direct Cargo/SDK tests acquire the same lock externally. No builds alongside measurements.
- Preserve fixtures, frozen measured Stores, source recovery and referenced immutable controls/raw failures. Keep bounded owned scratch and clean it only after required proof. Do not repeat completed #117 retirement/pruning or run `cargo clean` between iterations.
- Preserve unrelated `web/`, issue112/issue113 docs, `docs/roadmap/0.2/cloud-sqlite-vfs.md` and current uncommitted work. No `git add .`, reset or rollback of others' changes.
- Preserve `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-transition-experiments`: it was separately owner-authorized and is an explicit exception to the original sole-worktree condition. Do not create additional persistent worktrees or touch this other task's source.

## Terminal checklist

- [x] Infrastructure simplification and original focused qualification recorded.
- [x] Stage3 component/reduced-budget proofs, physical reservation repair, edit/reopen optimization and fsync qualification recorded.
- [x] Cold-target waiver and permitted timing warnings recorded; existing selected cold/namespace proofs retained.
- [x] Eighteen shared selections and eighteen independent proofs recorded with their n=1 scope.
- [x] Full157, matching Git and11+11 access results recorded on their actual408f producer cohort.
- [x] #116 initial audit, counter repair and shape-dependent capacity root cause recorded.
- [ ] Correct provenance, spill-threshold and proof-scope errors; establish applicable final-source coverage.
- [ ] Complete #107 prospective ordinary-storage target, measured saving and affected correctness/latency/resource qualification.
- [ ] Complete bounded pending-representation/publication repair without changing required concurrency/lifecycle behavior or raising quotas.
- [ ] Pass exact10000-edit final-content/Commit/reopen proof and affected SDK/FUSE/mmap/failure/pressure checks.
- [ ] Pass default-budget public K32000 spill coverage and independent Commit/reopen proof.
- [ ] Pass the pinned package install/update workflow and its Commit/reopen verification; complete required large/wide/deep/history boundary dispositions.
- [ ] Complete affected final qualification, exact artifact/cost/cleanup reconciliation and linked-issue outcome table; no hidden required FAIL/NOT_RUN.

Start with the short record correction and remaining #107 work, then finish the #116 repair, package workflow and affected final gates. Continue through implementation and proof. An internal known limitation or missing workload toolchain is work to complete, not grounds for stopping at a report. If a genuinely external dependency prevents progress, state the exact missing prerequisite and leave the issue open with the unmet gate explicit.

# Issue 95: bounded reuse of authenticated Init comparisons

The retained repair cuts duplicate-heavy directory Init time by **80–82% in adjacent control/candidate observations**, preserves exact collision checks, and retains the full157 storage saving. The focused issue95 diagnosis, implementation, measurement and terminal-validation criteria are met. Unique/scattered publication costs and parts of mixed-file validation remain; restoring every case to v0.1.3 performance and global release qualification are **not** claimed.

This is a stacked follow-up to unmerged PR #94, starting at `cf3a058925c3012fda0fae922dc081766bb8fa99`. Product source is frozen at **`c48bb4903f456136ccbcdba78de38b9042d2755a`**; later commits package evidence only. The original issue91 acceptance and closure did not turn historical regressions into passes. Issue93 remains the broader tracker.

## Measured cause and retained behavior

These families time directory import through `initialize_layerstack`. The predecessor-bearing Workspace worker cap is outside this timer. The original [30-case diagnostic table](r26-diagnostics.csv) shows two different scaling patterns: identical/CDC-edit cases spend most time repeatedly reading and comparing known objects, while unique/scattered cases spend substantial time committing SQLite pages.

A short profile of the sealed R26 identical-500 control found a 529-sample comparison branch, including 361 samples in canonical authentication and 73 in native decompression/reconstruction's decompression branch. Locator lookup had 26 samples outside that comparison branch; an extraction branch had 48, and byte comparison within packed reads had 18. These are sampled, nested stacks from a short interval, not full-operation phase durations or physical disk-byte counts. [Profile and command custody](evidence/p03-identical-control/profile.txt.gz).

The tested hypothesis was that an initialization owner could retain already authenticated comparison operands across bounded incoming batches. The implementation moves successfully compared input bytes into owner-local state; it introduces no extra canonical payload clone. Every reuse hit still requires a fresh SQL locator lookup, an equal full physical `Location`, equal canonical length, and exact bytes. Misses use the existing authenticated packed reader. Session/publication epochs, dependency checks, physical encoding, SQL publication and final-root atomicity are unchanged. Generic, non-initialization admission follows its original comparison path.

The **2 MiB total reuse reservation** comes from the existing 16 MiB uniqueness/filter allowance. It includes `Vec` capacity and a conservative 2,048-byte per-entry B-tree charge, covering a whole node even with one live entry. Oversized operands bypass retention; saturation clears the retained set. This reduces the allowance available to the preexisting-object uniqueness index, which can spill earlier. It does not increase resource limits. Large or mixed working sets can miss repeatedly; mixed-500 still spent about 209 ms in storage comparison in the terminal observation. No more complex eviction policy or larger reservation is claimed necessary by the present evidence.

Identical-500's paired native decode count fell **24,883→56**, while its exact collision-check count stayed **29,999**. CDC overwrite's collision-check count stayed **26,544**. The latter is the safety invariant: an ID match or earlier occurrence never replaces byte validation. `collision_checks` includes reuse hits; `conflict_read_rows`, `conflict_read_bytes` and `conflict_read_ns` now cover only misses passed to `admission::compare`, excluding hit-comparison time. Canonical comparison bytes are not physical disk reads. Concurrent producer-blocked times must not be added to elapsed wall time.

## Adjacent measurements

The [prospective protocol](https://github.com/Ephemeral-AI-Lab/layerfs/blob/c48bb4903f456136ccbcdba78de38b9042d2755a/docs/roadmap/0.1/0.1.4/issue95/README.md) fixed one control-then-candidate pair per case, seed 1, fresh output, ordinary OS caches, unchanged fixtures/harness/timers, and standard Docker limits. Each arm has **n=1**; its median and min–max are the single elapsed value shown. These are selected development observations, not admission distributions. The large duplicate gains did not require a reversed-order pair; no reliable unique-file speedup is claimed from its smaller difference.

| Case (500 MiB input) | R26 control (ms) | Candidate (ms) | Elapsed change | Host CPU control→candidate (ms) | Lifetime peak RSS control→candidate (MiB) |
|---|---:|---:|---:|---:|---:|
| cross-file identical |704.759250|123.357000|−82.50%|1586.252→976.660|21.891→23.391|
| CDC overwrite |740.773625|146.270125|−80.25%|1589.060→942.634|39.016→38.844|
| cross-file unique |1448.267417|1374.039959|−5.13%, not a reliable gain claim|2304.927→2215.159|63.563→64.859|

[Exact paired operands, CPU, lifetime/sampled RSS, fixture compatibility, source identities and raw hashes](evidence/pair01/summary.json). CPU is after-minus-before host-process CPU, including worker threads; it is not wall time. Lifetime peak RSS and the existing broader sampled window are not incremental or exact Init-phase peaks. Matching fixture compatibility manifests and harness hashes establish pairing; the runner's outer `input_identity` also hashes source, so that source-bearing identity intentionally differs.

Unique-500's paired allocated Store observation increased **11,550,720 B**, while its live database grew **one 4 KiB page** (530,853,888→530,857,984 B). The allocation adjustment's filesystem cause was not measured; it must not be called proven preallocation, reclaimable waste, or extra bytes beyond EOF. The terminal full157 retained-state storage test below is separate.

## Unique-file diagnosis and no-change conclusion

The separate [unique-500 profile](evidence/p04-unique-control/profile.txt.gz) had 708 main-thread samples, 628 in Init. Commit had 293, including 267 in `guarded_pwrite_np`; insertion had 81, and native compression had 160. Parent/child counts overlap. Thus the observed commit cost is chiefly the page-write syscall path, not page-cache maintenance or B-tree work. This does not identify storage-device latency. Native compression is a substantial separate cost.

The path already has sorted locator insertion, bounded multi-row statements, bounded pack BLOB statements, and coalesced transactions below 8,192 objects/4 MiB. R24's larger locator statements were slower and consumed more RSS; that rejected experiment remains in the parent evidence. The profile provides no justified small unique-file repair while preserving 4 KiB pages, exact native encoding, imported libraries and resource/transaction limits. Custom VFS, incremental BLOB surgery, codec changes or larger limits are not retained.

## Full terminal campaign

| Validation | Result |
|---|---|
| Native workspace suite |410 passed in 37 binaries; warm suite 87 s, within its unchanged 120 s ceiling |
| Additional native checks |Ignored large-spill/fresh-reopen test and Store doctest passed separately |
| New regression coverage |5 checks: authenticated first read; exact reuse/collision/location checks; bounded eviction; exact-limit/spare-capacity node accounting; rollback of pending and previously committed private batches |
| Existing correctness coverage |Duplicate accounting, fresh-filter collisions, stale absence/publication epochs, physical ownership, dependency authentication, transaction limits, empty final publication, final-root/metadata/COMMIT-failure atomicity, native FULL/PREFIX read/authentication |
| Formatting / Clippy |Formatting and warning-denying workspace Clippy passed |
| Full benchmark |198/198 performance executions passed |
| Independent routine proofs |226 passed; one declared optional 600-second endurance proof not run |
| Focus families |All 30 performance cases and 31 proof members passed, including CDC boundaries |
| Supplemental cases |Small-files and all four SDK/FUSE text-32k/binary-8m performance and verification passed |
| Full157 |157 performance states,157 retained-history proofs,158 checkpoint validation/accounting records, resource/custody checks and cleanup passed |

[Check qualification](evidence/terminal-check-qualification.json), [coverage records](evidence/terminal-correctness-coverage.json), [full focused qualification](evidence/terminal-qualification.json), and [supplemental qualification](evidence/supplemental-qualification.json). Resource quantities that the existing observer cannot measure remain explicitly unavailable; a custody PASS is not a claim that unavailable quantities were measured.

The unchanged comparison reporter remains **INCOMPLETE with exactly the four historical Git image-bound fixture errors**. No reporter, fixture, threshold or assertion was changed. Among198 elapsed comparisons it records 56 SEVERE, 86 REVIEW, 35 OBSERVED_INCREASE, 17 NO_INCREASE and 4 INELIGIBLE. Original historical labels remain in the parent report; current execution success is not comparative speed success. The unrelated-history-500 absolute target still misses; the report also retains its Git target observations despite their ineligible historical fixture comparisons.

| Family | Published v0.1.3 sum (s) | Prior R26 sum (s) | Final sum (s) | Final vs R26 | Final vs published |
|---|---:|---:|---:|---:|---:|
| CDC locality, 20 cases |1.484589124|5.734003169|3.270421919|−42.96%|+120.29%|
| Cross-file, 10 cases |1.402381375|4.284768333|3.459667499|−19.26%|+146.70%|

These are descriptive sums of individual case timers, not campaign wall time, paired speedup estimates or statistical distributions. [All final diagnostics](evidence/terminal-diagnostics.csv) preserve the10/100/500 scaling and tier1 anchors. [Complete comparisons](evidence/terminal-benchmark-report/comparison.csv.gz), [performance table](evidence/terminal-benchmark-report/performance.csv.gz), [verification table](evidence/terminal-benchmark-report/verification.csv.gz), and [unchanged report JSON](evidence/terminal-benchmark-report/report.json.gz) preserve every remaining result.

## Storage and full157 timing

| Quantity | Supplemental control | Prior R26 candidate | Final candidate |
|---|---:|---:|---:|
| Original allocated Store bytes |218,116,096|184,586,240|**184,582,144**|
| Logical SQLite bytes |205,529,088|176,226,304|176,152,576|
| Performance case wall (s) |474.329086083|451.741927208|447.246954625|
| Historical verification case wall (s) |502.697860167|520.650875209|545.826543958|

The allocated reduction is **15.374%**, preserving the prior 15.37% gain and exceeding the established 10% storage-reduction gate. The candidate is 4,096 allocated bytes smaller than R26. Both control and candidate use 4 KiB pages, zero freelist pages, and zero unexplained SQLite accounting residual. Candidate native representation remains **58,306 PREFIX + 28,106 FULL**, with identical respective frame-byte totals to R26 (27,155,798 and 68,435,250 B).

Equal retained file states are established by both arms' complete historical proofs under the unchanged frozen workload, not equal object counts: candidate/control selected populations are 366,144/366,173. Neither has unselected records, base-only objects or selected objects outside required retention. [Authenticated inventory proof](evidence/terminal-full157/candidate-proof/inventory-proof.json), [accounting](evidence/terminal-full157/candidate-account/accounting-reconciliation.json), [validation](evidence/terminal-full157/candidate-validation/validation.json), and [resource/timing report](evidence/terminal-full157/resources/resource-timing-summary.json).

The unchanged supplemental control is reused under authenticated source/contract/custody applicability; it is **not** the published v0.1.3 performance baseline. Full157 timings are historical single observations, not adjacent pairs or latency distributions. The higher final historical-proof wall is retained. Setup, snapshot, census, case wall, nested public phases, enclosing invocation and verification have separate scopes and must not be added indiscriminately.

## Preserved unsuccessful attempts

- `p01-identical-control` sampled the waiting launcher; `p02-identical-control` failed to match the worker command. Both executed successfully but supply no useful mechanism profile. `p03` captured the authentic worker. Profiled elapsed values include observer overhead and are excluded from paired speed claims.
- First build `1d365f5a8` used an insufficient 512-byte B-tree entry charge. Review rejected that charge before performance collection; the final 2,048-byte charge and boundary test are in `c48bb4903`. The first binary remains in raw custody; it is not retained product qualification.
- The initial derived pair summary incorrectly required equality of source-bearing input identities. The corrected summary checks the actual fixture compatibility and unchanged harness. Raw measurements were never edited.
- Four terminal verifier launches encountered another task's measurement lock before any runtime, Store or assertion activity. Original receipts and the initial incomplete ledger were archived byte-for-byte; only those four refused proofs were retried. All passed; 222 passing proof receipts were reused. [Refusal/repair custody](evidence/prework-refusal-repair-01/custody.json). No product or verifier logic was changed for this infrastructure recovery.

## Identities and evidence custody

| Artifact | Exact identity |
|---|---|
| Product source |`c48bb4903f456136ccbcdba78de38b9042d2755a`|
| Source seal |`4bbc984b2875f824d75351bf433dc53567d67ba04008ed0ec783057df70375b8`|
| Product seal |`54e119cd73f8cba7548c7ee9e76f05beba7b3889f316ddff068314f608b41f52`|
| Host binary SHA-256 |`2e182ff922b997607a59a612b726392a88ae670020ca0f02c4c7826964fef5ae`|
| Docker image |`sha256:3816927080ea949d4b7a0489047669a3b6b014f67a1e3922d3e5aa351e63d28d`|
| Full157 frozen schedule SHA-256 |`97b02190616c1d99cb7d66d0f205d2b43568d590eddbaf6a573bf2f0c60f0dc4`|
| Final immutable snapshot SHA-256 |`808102f45e4bad58d9969b8d717338ba2a9603f4eb1f3ddb327d2d41970ca300`|

Control product is `861e388339ef5572659cb16ef0c8febbff0351df`, binary `c4be654eb87cf76a16536e832a0767e620f1e1647c547e17b9a3d3c57db3984f`, image `sha256:8e74a58284a6e8c22ac9cbf60231f69217c3cae31c231ef17b0afd81726b8249`. Product, harness and imported-dependency applicability from that source to the parent evidence-only head is unchanged.

The final host binary and sidecar are archived under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95-runs/final-host/`. Original raw observations remain under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95-runs`; no old issue91 evidence was overwritten. [Evidence manifest](evidence-manifest.json) binds packaged files to exact original paths/hashes; [remaining raw references](evidence/raw-references.json) bind larger observations. Gzip packages decompress to the exact original bytes. [Build qualification](evidence/terminal-build-qualification.json), [full157 source/control preflight](evidence/terminal-preparation/identity-preflight.json), [control reuse](evidence/terminal-preparation/control-reuse-applicability.json), and [decoder reuse](evidence/terminal-preparation/census-reuse-applicability.json) record applicability. The standalone decoder's executed read/codec/accounting source is unchanged across 249 authenticated files; Init admission is unreachable from that tool.

macOS owned SQLite, construction/publication, SDK coordination and spool. Docker owned only daemon/FUSE/workload execution, with the unchanged 2-CPU/2-GiB/no-swap profile. Resource-sensitive work used the existing measurement lock. No imported library, feature or page size was modified. Benchmark workloads, timers, comparison thresholds and verifier assertions are unchanged; unit budget assertions account for the new reservation within the same total allowance. No merge, tag, release or unrelated issue closure was performed.

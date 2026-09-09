# R26 terminal candidate and completed reruns

The user selected the best available optimization and requested the complete benchmark and DeepSeek 157 harness. Optional optimization has stopped. Product source is frozen at `861e388339ef5572659cb16ef0c8febbff0351df`; subsequent evidence-only commits do not change the measured product. Performance acceptance is separate from the unchanged historical gate classifications. This is not a release or a claim that all original v0.1.4 gates pass.

## Retained changes

New-file Commit streams finalized output, avoiding deferred spill replay. Fresh Init uses a bounded negative membership filter and coalesces existing physical publication batches into SQL transactions below 8192 objects and 4 MiB. Final-root publication and rollback remain atomic. The paired terminal Init diagnostic improved from 4.219862937 s (filter-only control mean) to 3.596874584 s, with approximately 8.85–8.93 MB additional peak RSS; earlier pairs cost 12–15 MB. These are two adjacent pairs, not a population estimate.

Terminal full157 uncovered an older scheduling incompatibility: splitting the 1 MiB output allowance among multiple Workspace producers denied every 608 KiB predecessor-correspondence grant. R26 caps predecessor-bearing plans at one already bounded worker, restoring PREFIX without increasing memory allowances. Mixed plans serialize all their file tasks; pure-new plans and native Init retain parallelism. Native encoding, dependency ordering, 4 KiB SQLite pages and imported libraries remain unchanged. The standalone census lockfile changes only two project-owned crate version identities.

## Completed validation

| Check | Result |
| --- | --- |
| Native suite | 405 passed, 1 ignored, 37 binaries |
| Focused correctness | Filter duplicates/collisions, SQL cohort limits, rollback, empty final publication and final-root atomicity covered; predecessor regression fails before and passes after repair |
| Additional Store checks | Large-spill/reopen and doctest passed; unchanged Store source applicability recorded |
| Formatting / warning-denying Clippy | Passed |
| Full benchmark | 198/198 performance executions and cleanup passed |
| Independent routine proofs | 226 passed; one optional 600-second endurance observation not run |
| Supplemental workloads | Small files plus SDK/FUSE text-32k and binary-8m performance and verification passed |
| DeepSeek full history | 157 performance states, 157 retained-history proofs, 158 receipt validation/accounting records, cleanup passed |

Execution PASS means the run and its assertions succeeded. It does not mean the candidate is faster than the published checkpoint. One historical absolute product target also misses (`dedup-history-unrelated-500-mixed-v2`).

## Measured performance and storage

| Observation | Comparison | Candidate | Change |
| --- | ---: | ---: | ---: |
| 100,000-file Init | published v0.1.3: 2.603162083 s | 3.803760250 s | 46.12% slower |
| 500 MiB creation | published v0.1.3: 3.068249542 s | 2.701167542 s | 11.96% faster |
| full157 allocated storage | supplemental C+S1 control: 218,116,096 B | 184,586,240 B | 15.37% lower |
| full157 logical database | supplemental control: 205,529,088 B | 176,226,304 B | 14.26% lower |
| full157 performance case wall | supplemental control: 474.329086083 s | 451.741927208 s | 4.76% lower |
| full157 historical verification wall | supplemental control: 502.697860167 s | 520.650875209 s | 3.57% higher |

The 500 MiB creation case retains 536,875,008 allocated Store bytes versus 637,599,744 at the published checkpoint (15.80% lower). The full157 storage comparison exceeds the required 10% reduction at equal retained state. Both snapshots use 4096-byte pages, zero freelist pages and zero unexplained accounting residual. The candidate contains 58,306 PREFIX and 28,106 FULL records. The rejected R25 all-FULL candidate occupied 318,803,968 allocated bytes; its failure remains in the evidence.

The full157 storage control is not the published v0.1.3 performance baseline. Its unchanged authenticated R25 run is explicitly reused under its original frozen schedule. All reported timings are single observations unless described as paired; nested phase, case and invocation scopes must not be summed. See the [storage and resource report](storage/report.md).

## Comparison report limitations

The initial report had 637 cascading binding errors because collection did not produce the generation-local declaration required by the existing reporter. A separately labelled post-collection view derives the missing binding from pre-existing source/image preflight, commands, frozen membership and observed identities. Six receipt paths are rebased in a derived ledger; raw measurements and original ledgers remain unchanged. The unchanged reporter now reports four Git prepared-fixture compatibility errors and retains status **INCOMPLETE**. No thresholds or comparison eligibility checks were weakened.

The [Git digest diagnosis](campaign/r26-report-repair/git-fixture-diagnosis.json) establishes that the only differing fixture field is `input_plan_sha256`. It hashes the Docker image ID along with the recipe and generator source. Four generator files are byte-identical to the published checkpoint. Read-only metadata evaluation with the recorded old and new image IDs reproduces both hashes for all four cases. This proves the digest difference comes from image provenance; it does not independently prove equality of every historical generated `.git` byte. Preserve INELIGIBLE and INCOMPLETE rather than claiming that unavailable proof. PR #94 remains draft because the original all-comparisons qualification gate is unresolved.

Among 198 elapsed-time comparisons, 194 are eligible: 64 SEVERE, 91 REVIEW, 23 OBSERVED_INCREASE and 16 NO_INCREASE. Four Git cases remain INELIGIBLE. Across all metrics there are 302 SEVERE rows; these are not 302 independent workloads. Accepted performance tradeoffs do not erase these labels or resolve fixture comparability.

## Evidence and provenance

- [Checks](checks/README.md): focused coverage, suite, failed attempts and source applicability.
- [Storage](storage/report.md): accounting, independent analyzer, immutable snapshot custody, resources and retained failed R25 result.
- [Campaign provenance](campaign/provenance.json): exact command/preflight/summary copies, paired Init measurements, comparison outputs and original failed report. Large report JSON/CSV files use deterministic gzip; decompress before inspecting.
- [Derived binding plan](campaign/r26-report-repair/REPAIR-PLAN.md) and [binding provenance](campaign/r26-report-repair/binding-provenance.json): explicitly post-collection, with original receipt hashes and exact path-only transformations.
- [User selection amendment](campaign/r25-terminal-preparation/user-selection-amendment.json): performance tradeoffs accepted; correctness, dependency, storage and page-size requirements retained.

Large raw runs, stores and inventories remain under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs`, with original paths and hashes in these receipts. They are not embedded in this compact review package. No merge, tag or release was performed.

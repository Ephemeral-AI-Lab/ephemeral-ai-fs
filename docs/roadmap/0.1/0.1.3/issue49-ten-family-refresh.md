# Ten-family current-product refresh after issue49

The user explicitly requests the full families listed in issue38 plus tiny-file churn. Use issue38's latest simplified fast-baseline scope: one complete current-product performance sample per registered case, seed1 (SDK repetition1), followed by separate selected bounded verification. No baseline rerun or automatic three-seed multiplication.

Current scope: init_namespace4, edit_length_preserving12, edit_length_changing32, edit_canonical_chunk_count12, store_footprint6, payload_create_read8, dedup_workspace_reuse14, dedup_cross_file10, dedup_cdc_locality20 and tiny_file_churn20:138cases total. Definitions are unchanged, including compact low tiers and mixed-v3 high-tier tiny bulk inputs. The CDC boundary case remains proof-only.

Source/product/binary/image are the delivered #49 build: source seal4ce40e2c3c357390..., product3fb2f18f..., image layerfs-bench-infra:4ce40e2c3c357390. Reuse the two matching tier100 mixed-v3 raw receipts from results attempts21–22; collect the other136cases once. Keep raw receipts immutable and source-bound. Every build/performance/proof runs serially through the global measurement lock. Use the existing host runner and protected compatible preparation; no new benchmark engine or product changes are planned.

macOS owns SDK/Workspace/spools/SQLite, Docker only daemon/FUSE/workloads;2CPUs/2GiB/no swap/256PIDs/no data mounts. Product targets stay the registered15second family target and separate strict tier100 #47 target. Defaults remain120second product/130second outer; retain the authorized600/630second allowance for tier500 selections. Allow600seconds for cold selected preparation, separately reported. No cache-state or warm/cold equivalence claim is introduced.

Collect all requested performance coverage before selected proofs. Retain valid TARGET_MISS and failures, and investigate concrete failures without discarding evidence. Reuse issue38's27 selected proof shapes on matching current identities with45second work/59second hard limits; selected low-tier tiny create/stat/unlink/bulk-create/bulk-delete proofs cover its operations. The existing strict #47 tier100 proof gate remains: its final mixed-v3 proof pair is deferred until both full lifecycles pass. Do not claim exhaustive namespace/full-file verification, full statistical/scaling qualification or parent completion.

Campaign root: `benchmark-results/host-store/campaigns/issue49-ten-family-4ce40e2c3c357390/`. `selections.json` freezes138unique rows and the prior selected proof shapes. The final assessment will enumerate every requested case, exact timer, identity, resource/cleanup result, proof coverage and omission. Existing issue38/issue46/issue47 evidence is historical and remains unchanged.

Selected tiny verification also includes the two tier500 mixed-v3 bulk cases if their family performance passes, to exercise all large-file range samples on the revised profile. This is additional bounded coverage within the full-family request; it does not bypass the separate strict tier100 #47 proof gate. The resulting planned proof cohort is27issue38 shapes plus7tiny shapes (34selected proofs), not all cases/seeds or exhaustive byte verification.

## Completed results

**PASS for the requested fast scope:**138/138registered performance cases and34/34selected independent proofs.136new performance samples were collected;2matching current-source tier100 mixed-v3 receipts were reused unchanged. No product or benchmark source was changed during this campaign. Each case has one seed1/repetition1 observation, not a statistical distribution.

| Family | Performance PASS | Declared timer | Across-case minimum ms | Across-case maximum ms |
|---|---:|---|---:|---:|
|init_namespace|4/4|`layerstack_init_ns`|10.689542|2995.610000|
|edit_length_preserving|12/12|`edit_commit_ns`|3.848333|6.230791|
|edit_length_changing|32/32|`edit_commit_ns`|3.992792|6.077667|
|edit_canonical_chunk_count|12/12|`edit_commit_ns`|4.634625|7.276542|
|store_footprint|6/6|`product_call_sum_ns`|28.199543|4925.692541|
|payload_create_read|8/8|`pure_call_sum_ns`|18.701709|2930.254168|
|dedup_workspace_reuse|14/14|`pure_call_sum_ns`|27.495541|4115.859625|
|dedup_cross_file|10/10|`pure_call_sum_ns`|4.131667|535.932208|
|dedup_cdc_locality|20/20|`pure_call_sum_ns`|4.070500|460.503459|
|tiny_file_churn|20/20|`pure_call_sum_ns`|17.793375|5927.606999|

Ranges span different registered cases and sizes, not confidence intervals. The existing15second family target passed everywhere. SDK rows measure edit+Commit; native import rows measure initialization; Workspace and storage rows retain their declared product-call sums. No timer substitution or cross-family throughput/scaling claim is made.

### Tiny bulk detail on the current product

| Tier | Operation | Exec ms | Commit ms | Complete lifecycle ms |
|---|---|---:|---:|---:|
|1|create|190.928750|10.164875|212.826875|
|1|delete|112.768792|7.191250|131.399375|
|10|create|518.582291|40.976958|571.908415|
|10|delete|185.394958|9.493167|205.758084|
|100|create|917.203209|358.675583|1291.390876|
|100|delete|268.385792|14.803083|298.008125|
|500|create|4128.404208|1779.048791|5927.606999|
|500|delete|910.342708|54.085792|977.119375|

Tier1/10 retain compact-v2; tier100/500 use mixed-v3. Different distributions are not a single unchanged scaling curve. The separate strict #47 threshold still misses for tier100 create at1,291.390876ms; tier100 delete is298.008125ms. Their final strict-gate proof pair remains deferred. Family PASS does not close #47/#46/#39 or claim unpublished #48 adoption.

### Independent sampled verification

The34proofs cover the27issue38 shapes plus7tiny shapes. SDK coverage is every1MiB edit/outcome shape plus capped500MiB insertion; namespace covers all4sizes; storage covers3main controls; selected500MiB payload/reuse/CAS/CDC and CDC-boundary proofs pass. Tiny covers individual create/stat/unlink and compact bulk create/delete, plus both500-tier mixed bulk operations. Every large file in the mixed500 create proof has64KiB beginning/midpoint/end samples, alongside declared medium/small/witness paths; delete checks target absence and witness retention.

Total measured proof wall is116.633502s; maximum13.993575s, within the45second work/59second hard limits. All34proof attempts passed;34separate preparation receipts are retained outside the proof-wall sum. Preparation is not hidden inside performance statistics. Coverage and omissions remain explicit; no exhaustive file-byte/namespace census, alias/failure-injection campaign or multiple-seed qualification is claimed.

### Source, resources and evidence

All performance/proof receipts share source seal `4ce40e2c3c3573902e3820d8e16c5f1be793f42dbeb14e4bb72ed4274557fc1f`, product `3fb2f18f1c636b8c3aa8b2a901bb3ca2a56cb075e674b6f520e8c685ca2b1053`, host build commit `6de381837d1c22e5eb21dfaf446f85c9c30ea6ec`, binary SHA-256 `4de1c0e44903299865434e94c1378f00d03545e930719e0a81e205f792ac9be7`, and image `layerfs-bench-infra:4ce40e2c3c357390` / `sha256:47d0d03f9a7079292c6ab5fa076a5392c7aa6e79a7f1e47a76467f89eed15e0e`. Later commits in this campaign are documentation only.

Observed container limits match2CPUs/2GiB/no swap/256PIDs, without data mounts; macOS owns SQLite/Workspace and host CPU/resources are separately scoped. All performance and proof cleanup passed; no campaign sample containers or host sample directories remain. Compatible protected preparation remains available. SQLite masters are checked unchanged; native imports use fresh output Stores and the existing owned-prepared-recipe source validation, so a SQLite-master flag is not applicable to those rows.

The first derived assessment draft incorrectly treated that native null master flag as failure. It is preserved as `performance-assessment-draft-native-na.json`; the final assessor applies the recorded setup policy correctly. No raw receipt was changed and no workload was rerun for this reporting correction.

Evidence under `benchmark-results/host-store/campaigns/issue49-ten-family-4ce40e2c3c357390/`:

- `selections.json`: exact138-row registry, source/build identity and2reused receipt references.
- `performance-assessment.json` and `performance-cases.csv`: every case/timer/result/input identity/raw hash and resource/cleanup validation.
- `performance/<family>/<case>/perf.jsonl`:136new immutable performance receipts; current-source tier100 reuse paths are recorded in the assessment.
- `proof-selections.json`, `verification/<family>/<case>/verification.json` and `proof-preparation/`:34proofs with exact coverage/omissions and preparation receipts.
- `terminal-assessment.json`: complete coverage, matching provenance, proof wall bounds and campaign-specific cleanup inventory.

This completes the requested ten-family run under the current fast benchmark contract. It does not alter the separate strict subsecond goal or close issues automatically.

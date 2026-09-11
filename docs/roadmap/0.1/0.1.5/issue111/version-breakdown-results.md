# Measured Init breakdown: historical v0.1.3 and current v0.1.5 (#111)

**4 KiB remains fixed for storage efficiency. No product optimization or format
change was made.** The fresh diagnostics locate the additional wall time in the
busy admission consumer and final root/inode construction. Increased producer
waiting does not explain the version difference in this cohort.

## 1. The two quoted plain numbers have no retrospective phase breakdown

| Retained plain observation | Init ns | Read B | Evidence |
|---|---:|---:|---|
| historical v0.1.3 |2,776,088,125|871,841,792|issue109/measurements.json, historical_product/plain_receipt|
| current v0.1.5 |3,435,659,584|865,738,752|cold-contract-evidence/20260911T040843Z/selected-terminal/perf.jsonl|

Neither plain receipt recorded internal clocks. Their literal 659,571,459 ns
difference cannot be decomposed after the fact. The v0.1.3 reference is distinct
from the similar-looking v0.1.5 warm 2.778672334 s observation. Historical v0.1.3
cache acquisition predates the new cold qualification guard.

Earlier companion diagnostics were 2.537925917 s (historical v0.1.3, 639,119,360
read B) and 3.760371083 s (v0.1.5, 873,615,360 read B); differing cache acquisition
prevents treating their phase difference as an exact explanation of the plain
numbers. The tables below are fresh companion diagnostics, not normalized or
fabricated splits of 2.776/3.436 s.

## 2. Fresh full-fixture cold diagnostics

The [prospective protocol](version-breakdown-contract.md) froze n=2/product and
order historical,current,current,historical. Same immutable 100,000 files /
500,000,000 logical B and fixture digest, fresh independent macOS Stores, same
public Client::initialize_layerstack entrypoint and direct init-only CLI shape.
Before each run the shared cold acquisition checked all100,000 files/125,169
pages, with a known-warm positive control and zero observed resident pages.
Each source has865,730,560 allocated B. The legacy cache-profile string is
reused-first-sample-uncontrolled; verified acquisition/read evidence is retained
separately. No untimed warm-up, resource-sensitive overlap or retries.

| Arm/sample in execution order | Init ns | initialization_disk_read_bytes | Resident pages | Classification |
|---|---:|---:|---:|---|
|v013/1|2,855,498,375|869,990,400|0|verified-cold-diagnostic|
|v015/1|3,920,191,250|875,466,752|0|verified-cold-diagnostic|
|v015/2|3,745,481,542|874,872,832|0|verified-cold-diagnostic|
|v013/2|2,488,704,250|865,730,560|0|verified-cold-diagnostic|

All four commands exit0 and match the complete scan counts/bytes. These are
nonce-enabled **diagnostics, never acceptance**. The historical binary has its
original harness and schema; this is not a paired speedup claim or feature
ablation. Different image/harness identities are retained, not rewritten.
The direct diagnostic needs no mounted runtime; all Store/publication work is
host-owned. No historical Docker-owned Store execution was used.

## 3. Exclusive wall breakdown (milliseconds)

Medians, n2/product; with n2 each median is the mean of its two observations, so
these phase medians sum exactly to the enclosing median before display rounding.

| Exclusive phase | v0.1.3 ms | v0.1.5 ms | New minus historical ms |
|---|---:|---:|---:|
|Input/admission pipeline|2,506.081|3,141.587|635.506|
|Final root/inode tree|141.947|613.791|471.844|
|Import setup/handoff remainder|12.736|37.243|24.507|
|Publication/outer remainder|11.337|40.215|28.878|
|**Public Init**|**2,672.101**|**3,832.836**|**1,160.735**|

Total ranges: v0.1.3 2,488,704,250–2,855,498,375 ns;
v0.1.5 3,745,481,542–3,920,191,250 ns. The 1,160.735 ms
historical diagnostic gap belongs to this cohort, not the prior659.571459ms
plain difference. Final-tree increase is stable across corresponding samples:
462.993ms and480.695ms. Import/outer remainders are derived from enclosing
clocks, not individually timed suboperations.

## 4. What happened inside the pipeline

| Nested/derived clock | v0.1.3 ms | v0.1.5 ms | Difference ms |
|---|---:|---:|---:|
|Consumer idle|1,135.570|1,104.558|-31.013|
|Pipeline minus consumer idle|1,370.510|2,037.029|666.519|
|Pipeline transaction COMMIT wall|370.952|977.424|606.472|

Consumer idle is slightly lower, while busy-side time increases666.519ms.
The pipeline COMMIT clock increases606.472ms. These are nested measurements;
**do not add them to the exclusive table or to one another**. Pipeline-minus-idle
also includes small receive/join/bookkeeping overhead, not exclusively SQL CPU.
Both products retain approximately the same total 500MB authoritative reads;
current source read calls205,102 versus historical210,687. No extra complete
input scan inside Init explains the regression.

Code confirms the COMMIT boundary: historical objects.rs times
transaction.commit(); current AdmissionCohort::commit times
connection.execute_batch("COMMIT"). This clock is not the entire admission
batch or all B-tree INSERT work. It includes the SQLite commit/page-write path;
it does not by itself distinguish CPU, kernel writes, or device waiting.
Do not describe the whole difference as fsync or as purely in-memory work.

Historical sql_bind_step_returning_ns is populated; current output is0 because
that instrumentation is not populated along the promoted path. It is not proof
that current SQL binding/execution is free.

## 5. Costs of the new small-content/pack/metadata path

Current operation-local physical counter medians:

| Counter | Observation | Interpretation |
|---|---:|---|
|encoding_ns|402.212ms|Small FULL/DELTA, native FULL/PREFIX and ordinary group encoding clocks combined|
|native_full_encode_ns|151.212ms|Subset of encoding_ns; large-file chunk FULL encoding|
|encoding remainder|251.000ms|Small FULL plus ordinary group encoding; not separately timed here|
|metadata_index_sync_ns|248.740ms|Pooled inode-value index sync, reading/decoding groups and inserting scratch index values|
|metadata_pool_admitted_values|100,002|Observed pooled metadata values|
|usable_bases / candidate_trials|0 /0|No candidate compression trial reached|
|delta_selected / native_prefix_encode_calls|0 /0|No selected DELTA or native PREFIX encoding|

These counters are nested within Init and can span pipeline/final work. They
are **costs present in v0.1.5**, not independently isolated wall-time regressions.
Metadata indexing supports the final-tree cost attribution, but its operation-wide
counter must not be relabelled as a separately clocked final-tree-only interval.

No stored DELTA does **not** mean zero DELTA-related work. Current producer
FinalizedOutputWriter::put_file_payload computes a rolling small-candidate
signature when there is no explicit predecessor; prepare_small consults the
candidate index, computes the FULL compressed alternative, and only then tries
DELTA if a usable base exists. The signature pass/lookup cost is not individually
timed by these counters. #109 previously measured substantial signature work
and moved it to producers; that historical calibration is not a new exact
per-feature clock for this cohort.

The bounded physical census of v015-1 confirms98998 small FULL records and0 small
DELTAs. Small raw bytes300,000,000 become301,310,582 Zstandard frame bytes:
**1,310,582 bytes larger**, before pack framing. All pack blobs507,088,415B.
Native FULL frames for the two large files total200,144,158B for200,000,000 raw B.
The pseudorandom, unique fixture pays compression/search costs with virtually no
content-compression or DELTA saving. The normal path still benefits compressible
and redundant corpora; see the separately verified real-source reports.
Do not use aggregate full_selected as a unique-small-object count; use the census.

## 6. Storage and resource tradeoffs; 4 KiB stays

| Recorded difference | v0.1.3 | v0.1.5 |
|---|---:|---:|
|Schema|5 raw object BLOBs|10 packs/scoped namespace/pooled metadata|
|SQLite page size|65,536B historical|**4,096B fixed**|
|Native import producers|8 historical|4 unchanged|
|Median apparent Store bytes|662,372,352|515,602,432|
|Median allocated Store bytes|670,662,656|529,362,944|
|Median SQLite page count|10,107|125,879.5|
|User CPU ns|2,577,938,979.5|3,492,481,374.5|
|System CPU ns|6,723,140,750.0|5,633,038,958.0|

Current storage is about22.2% smaller by apparent length despite incompressible
content, because representation/metadata overhead falls. The products also differ
in page granularity and producer caps; attributing their whole time difference
solely to small-file DELTA would be incorrect. This version comparison includes
the intervening v0.1.4 changes as well, not an isolated v0.1.5 feature ablation.
User CPU rises about0.915s, system CPU falls about1.090s, and combined CPU is
approximately9.30s versus9.13s despite the longer wall time. The placement and
serialization of work matter; the result is not simply "more total CPU work."
CPU seconds across threads are not additive wall phases. 64KiB is historical
context only: the owner explicitly keeps
4KiB for storage efficiency. No page-size experiment is proposed here.

## 7. Diagnostic conclusion and next priorities

1. Investigate the current4KiB transaction COMMIT path and reduce its measured
   overhead without changing page size, correctness or acknowledgement semantics.
2. Investigate pooled metadata index synchronization and the remaining serial
   final-tree work; the latter grows about472ms in this cohort.
3. Isolate small-candidate signature/search and FULL/group encoding costs before
   changing policies. No DELTA trials means actual delta compression is not the
   culprit for this random fixture; preparing/searching for opportunities still
   costs work. No speculative feature removal or bypass.

This evidence supersedes the earlier recommendation to start by adding producers:
consumer-idle growth does not explain the observed version gap. No worker change,
product optimization, speedup, release or cold-target PASS is claimed.

## Custody, code and validation

Evidence root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-init-breakdown-evidence/20260911T043613Z`. collect.py, raw stdout/stderr, cold-acquisition.json,
result.json, commit-clocks.json, phase summary and retained output Stores preserve
all four attempts. Phase equations, scan counts and physical output presence pass;
read-only pack census assertions pass; benchmark self-check passes. Existing
independent proofs are reused for their unchanged product scopes, not reissued as
new acceptance of these diagnostics.

Only8 harness lines were added: snapshot physical counters for nonce runs, then
emit their difference **after t1 and after resource snapshots**. Plain acceptance
and product code are unchanged by this task. Original compaction-removal edits
remain untouched. Separate metadata-cache changes appeared in objects.rs,
objects/metadata.rs, schema.rs and metadata tests after collection. The recorded
before/after measurement seals matched; those later external edits are not part
of these measurements or this task's commit and were not reverted.

- Historical binary: `6ae5b77f2c923c23b8cf534de0b75a59253b9e8b74003f3c1fcd6547957233eb`.
- Historical product: `3c797bc6dbfd9b03b919c270b609cad839000b68d67e34f3b00d24717e07f39a`.
- Current binary: `d6b5d6f7a3757f0bef7d98afaed0f0c90806dd284408cba7cb1c83eb27feea39`.
- Current product: `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`.
- Current source: `bec36ed775692a07e71fc03a731a1b6926be8a804e40cf3183969b16f5a6df2c`.
- Fixture: `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.

Related #109/#110/#108/#106/#102/#104/#100/#107; #114 supplies separate-workload
consumer investigation context, not interchangeable timings for this case.

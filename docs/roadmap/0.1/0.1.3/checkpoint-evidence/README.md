# v0.1.3 benchmark checkpoint — issues #74 and #75

The shared campaign has **198/198 performance PASS and 226/226 routine
verification PASS** across 17 admitted families (16 performance families and the
reliability family). One additional 600-second reliability definition is explicitly
excluded and was not executed. These are fixed-seed observations, not a latency
distribution or an exhaustive release qualification.

- [Full family and per-test tables](report.md)
- [Machine-readable report, identities, resources and coverage](report.json)
- [Every performance case in CSV](performance.csv)
- [Every verification case in CSV](verification.csv)
- [Frozen inventory](raw/registry.jsonl), [campaign declaration](raw/declaration.json)
- [Per-family simplification decisions and execution contract](../checkpoint-74-75.md)
- [PR #76](https://github.com/Ephemeral-AI-Lab/layerfs/pull/76),
  [issue #74](https://github.com/Ephemeral-AI-Lab/layerfs/issues/74),
  [issue #75](https://github.com/Ephemeral-AI-Lab/layerfs/issues/75)

## Performance and limits

Git-100 measured **1,879.182 ms** and Git-500 **4,689.306 ms** for the complete
LayerFS lifecycle. This is respectively +1.45% and +1.16% against the compatible
published #73 three-run medians; one new observation cannot establish a
statistically meaningful regression. Apply and all six Git commands are listed
separately from LayerFS Commit in the full table. Historical native controls are
reference-only: they omit the LayerFS lifecycle and were not rerun for this campaign.

The historical 500/1,000 ms Git targets remain missed. The unrelated-history-500
case also misses the historical 15-second target at 18,163.889 ms. All three
complete within the unchanged 300-second collection allowance. The separately
reported strict subsecond bulk targets are not silently replaced by 15 seconds.

Nineteen routine proofs exceed the aspirational 15-second wall time. The slowest
is `store-footprint-metadata-cardinality-100000` at 38.552 seconds. All routine
proofs pass the unchanged 45-second work / 59-second hard deadline. The five
original high-tier history timeout cases now pass; sampled indices and omitted
exhaustive history/object checks are recorded per proof. Git retains its full
head/tree/parent and reopened-custody checks.

## One campaign, preserved attempts, explicit requalification

The original pass collected 196 performance observations. Two attempts were
refused by the shared measurement lock before work began. The
[resume declaration](raw/resume-1.json) records filling only those two absent
performance cells and their dependent proofs. No successful performance sample
was rerun for a better number. Original ledgers and logs remain under
`raw/attempts/initial/`.

Ten large SDK length-changing proofs initially failed their resource gate. The
[diagnostic evidence](raw/development/sdk-resource-diagnostic-r2/verification/edit_length_changing/insert-middle-4k-on-100mib-ops-1/failure.log)
shows a pre-edit FUSE lookup caused shifted-suffix cache refresh: 64,868,352 bytes
incremental cgroup memory against the 32 MiB cold-operation limit, with no swap,
OOM, forbidden writes or content failure. The routine verifier now matches the
performance worker's cold projection: no pre-edit FUSE lookup, one post-commit
FUSE boundary read. Canonical inode preservation, roots, payload retention,
publication/reopen, cleanup and all resource limits remain checked. Pre-edit FUSE
inode-number stability is explicitly omitted; this is not a warm-cache/mmap
resource claim.

All 56 SDK proofs passed on the versioned verifier and were requalified again
on the final source after a Clippy-only receipt-formatting correction. The first
56 passing receipts and their manifest remain archived. Their final
[manifest](raw/sdk-requalification.json) binds old/new source seals, the unchanged
product/image/harness, identical registered recipes and prepared fixtures, and
both input hashes (the recipe hash includes source). The report checks those
bindings and rejects missing receipts, failed nested resource results, mismatched
fixtures or products. The original 56 receipts, including the ten failures, remain
under `raw/verification/`; final replacements are under `raw/requalification-sdk-v2-final/`; the first
passing replacements remain under `raw/requalification-sdk-v2/`.
All other proofs and every performance observation retain their original source
identity. The only changed execution source for this requalification is
`benchmark/fs-bench-pro/src/sdk_edit_verify.rs`; no product, performance, Linux
runtime, workload recipe or timer changed.

## Product repairs and validation

The checkpoint also repairs failed-owner Discard and presentation recovery,
reaches real streaming-admission/spool fault boundaries, corrects obsolete Busy
oracles, bounds history verification, and flattens the recursive expected-content
oracle responsible for high-tier history timeouts. A live test exposed stale
folio writeback overwriting SDK bytes; the owner now protects installed SDK ranges
until kernel reconciliation drains. These repairs are already in the product
seal shared by every final performance and proof result.

The three required live Docker tests executed and passed on that product:
[final live test log](raw/development/final-live-docker-tests.log). This includes
concurrent workspaces, live commands, dirty mappings, ordinary writes and SDK
resize/write coherence. The original live failure and three focused successful
repair repetitions are retained alongside that log. The later SDK verifier-only
change does not change the product or these tests.

Shared runner/collector tests: 40 passed. Report regression tests exercise missing,
duplicate, mismatched and failed resources, plus rejection of changed fixtures and
products during verifier-only requalification. Required Rust CI is attached to
PR #76 and must be green on its exact head before merge.

## Reproduce the tables

From the repository root:

```sh
python3 docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.py \
  docs/roadmap/0.1/0.1.3/checkpoint-evidence/raw \
  --registry docs/roadmap/0.1/0.1.3/checkpoint-evidence/raw/registry.jsonl \
  --output /tmp/layerfs-checkpoint-report
python3 -m unittest discover \
  -s docs/roadmap/0.1/0.1.3/checkpoint-evidence -p 'test_report.py'
```

Large raw files use deterministic gzip compression; small receipts remain plain
JSON/JSONL. [Encoding manifest](raw/encoding.json) records both original and
published checksums. The report reads either form and links actual published
files. Original absolute machine paths inside receipts are provenance, not
required inputs to the derivation. Full operation transcripts remain in raw
receipts; report metrics keep phase sums/counts and first/last observations per
group rather than incorrectly summing cumulative counters.

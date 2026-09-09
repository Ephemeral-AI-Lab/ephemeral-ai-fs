# Issue101 historical access qualification

The registered `historical_access` v2 family passed all **11 selected performance
cases and11 separate verification invocations** through the public SDK/FUSE
surface. All complete command walls, including preparation, cleanup, interpreter
startup and receipt publication, were below the hard15second test limit and the
prospectively selected15second verification watchdog.

This qualifies the benchmark on the supported schema9 product. The65.96MB
schema9304 offline experiment remains unintegrated under #100. These results
neither claim that format is publicly readable nor compare product speedups.
Each row is an unpaired selected diagnostic, `admission_eligible=false`; #102
owns the full regression campaign and statistical comparisons.

## Final v2 results

| Case | Complete performance (s) | Complete proof (s) | POSIX operation (ms) | Encoded read bytes | Decoded bytes | Native raw decoded bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| ha-stat-old-v2 | 2.603 | 2.524 | 2.539 | 195,114 | 334,159 | 0 |
| ha-stat-head-v2 | 2.481 | 2.534 | 10.775 | 1,409,503 | 2,604,473 | 0 |
| ha-directory-old-v2 | 2.570 | 2.495 | 2.533 | 185,087 | 317,875 | 0 |
| ha-directory-head-v2 | 2.617 | 2.491 | 12.806 | 1,409,503 | 2,604,473 | 0 |
| ha-small-old-v2 | 2.635 | 2.549 | 2.675 | 195,114 | 334,159 | 0 |
| ha-small-head-v2 | 3.484 | 2.437 | 12.830 | 1,409,503 | 2,604,473 | 0 |
| ha-range-history-cold-v2 | 2.572 | 2.649 | 8.484 | 864,525 | 1,450,969 | 25,061 |
| ha-range-history-warm-v2 | 2.549 | 2.567 | 0.046 | 0 | 0 | 0 |
| ha-full-head-cold-v2 | 2.550 | 2.519 | 29.022 | 1,875,951 | 2,619,073 | 3,100,135 |
| ha-full-head-warm-v2 | 2.547 | 2.598 | 0.532 | 0 | 0 | 0 |
| ha-metadata-worst-v2 | 2.668 | 2.558 | 17.724 | 2,058,651 | 3,436,774 | 0 |

Complete command wall includes interpreter startup and final receipt writes, measured by the external collector.
performance: n=11 different cases, range 2.481–3.484s; median across these cases 2.572s. Each case has n=1 (median=min=max for that case).
verification: n=11 different cases, range 2.437–2.649s; median across these cases 2.534s. Each case has n=1 (median=min=max for that case).

Custody: `{"binary_sha256": "490dad1106145db1944d72294eb0e01ae1ea8b95bcfe1b30a41db852a266b451", "fixture_sha256": "f8f90fb42403b5d2ff0f1201106e1eb1788ee089b15ba70db6c9579e1462bf3a", "image_id": "sha256:42774d05a63029a75278ae65ea895e2fec0c6646c2b40c4bb19e94bcf1daadc8", "source": {"LAYERFS_PRODUCT_SEAL": "24cde1dce88104daebf2d01e6b711665cfa52c52880b31157fbcfa5c27214673", "LAYERFS_SOURCE_COMMIT": "395a51ddd762e3dbacffb812046feb1dc1ba4de8", "LAYERFS_SOURCE_DIRTY": "false", "LAYERFS_SOURCE_SEAL": "08b32c4eae6ebde9036e5335ccbceb721e001bd77df470901fea031733a4acb5", "LAYERFS_SOURCE_TREE": "3220c90cb6a16843945d6ffd77443c54ff2082de", "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127"}}`


`decoded_read_bytes` and `native_raw_decoded_bytes` are separate recorded counters,
not interchangeable byte bases. The POSIX timer measures only the named access;
physical counters and host CPU/RSS cover the surrounding public execution window,
including proof metadata access. Complete command wall is the acceptance metric.

The worst-metadata source path produced3,436,774 decoded bytes,454 group fetches
and189 base fetches. The2,201-byte head README read produced2,604,473 decoded bytes.
The6,421-byte cold native-range read recorded1,450,969 general decoded bytes and
25,061 native raw decoded bytes. Both warm read cases recorded zero Store fetch
and decode work in the measured execution windows after one identical warm-up.
These are concrete follow-ups for #102; no universal complexity or speedup claim
is inferred from one sample per case.

Host RSS/current footprint/lifetime peak and CPU receipts, plus cgroup CPU,
current/lifetime memory, swap and OOM observations are retained per case. All
runtime resource/cleanup checks passed. Exact phase memory peaks and unique pack
counts are explicitly unavailable, rather than fabricated zero. Caches are fresh
application lifecycles or one-warm-up lifecycles; OS cache state is uncontrolled.

## Coverage and custody

- [Prospective v2 contract](historical-access-v2.md): checkpoints1,57,65,157.
- [Registered fixture](../../../../../benchmark/fs-bench-pro/families/historical_access/fixture.json): exact retained Commit IDs, original source commits/oracle SHA256s, ranges and expected metadata/content.
- [Read-only original fixture audit](verify-fixture.py): independently recomputes every expected row from preserved original oracles and source Git blobs; passed.
- [Report generator](report.py): validates all22 per-case evidence manifests, selected identities, paired proof custody, correctness, cleanup, resource status and external deadline checks before generating the table.
- Raw evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue101-evidence/v2`.
- Explicit prepared input: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/host-runtime/store.sqlite`.
- Store SHA256: `f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7`.
- Runtime image: `layerfs-bench-infra:08b32c4eae6ebde9`; immutable ID appears above.

The full157 metadata worst leaf maps to checkpoint57's architecture note;
the full157 native worst-amplification owner maps to checkpoint65's pnpm lockfile.
These precise paths are frozen in the v2 contract. v1 had covered ten cases on
stride3 and passed all ten pairs, but omitted checkpoint57. Its evidence and
[fixture](fixture-v1.json) remain retained; v2 was preregistered before extending
coverage and rerunning the changed input. No timeout or correctness gate was
relaxed, and no passing v1 row was relabeled as v2.

## Failure-path checks

All46 product-free shared harness tests passed (`python3 -m unittest discover -s benchmark/fs-bench-pro/shared -p "test_*.py"`); both standard host and Linux runtime image builds passed.

- Missing sealed input returns NOT_READY in0.175seconds without Docker creation.
- Measurement-lock contention returns failure in0.042seconds, without waiting or runtime creation.
- An injected60second Docker-CLI stall is stopped by the working watchdog at12.011seconds total. The expected result is FAIL; cleanup passes and the injected process is confirmed absent after return.
- A preliminary v1 missing-image diagnostic exposed unnecessary Docker cleanup before container creation. That failed attempt is retained. The runner now records creation attempts and skips nonexistent-container cleanup.
- Product-free receipt tests reject wrong bytes/metadata, missing/duplicate operation receipts and missing cleanup; runtime tests cover Store isolation/sidecar rejection and supervised process-group cancellation.

No historical-access containers remain after qualification. Successful host
copies are deleted; failed-run evidence is preserved. Missing history never
triggers an automatic build or history replay.

## Handoff to #102

Use the documented `--family historical_access --case ... --mode performance`
and separate `--mode verification --performance .../result.json` commands.
Explicit `--all` selects all11 performance cases, each with a separate15second
envelope. Default/ordinary runs never construct repository history. The
[quickstart](../../../../../benchmark/fs-bench-pro/QUICKSTART.md) lists invocation
examples. #102 owns the broader campaign, repeated statistical samples, optional
repository_history strides1/3/10, and optimization of the measured cold read work.

If product/harness/workload/fixture changes, rerun the affected exact-candidate
cases and proofs. #100 must first expose a supported compact reader before this
family can qualify that representation; existing offline storage numbers retain
their original scope.

# Issue88 version-aware encoding experiments

Dedicated issue: https://github.com/Ephemeral-AI-Lab/layerfs/issues/88
Parent evidence: #87/#18; exploration baseeb7050603.

The subsequent combined159,163,199-byte encoded milestone and134,221,004-byte
complete allocated-Store stretch objective are targets, not forecasts. The retained-content screens and isolated structural public
prototype are complete as reported in [findings](findings.md). No rollout or
existing-Store migration is included.

- [Prospective sequential contract](contract-v1.md) and [exact screen details](screen-details-v1.md).
- [Public structural continuation](public-s1-contract.md) and [full-history decision retaining regressions](full-history-continuation.md).
- [Independent review](review.md) and [final accounting review](final-review-addendum.md).
- [Offline paired results](published/offline-results.json).
- [Fresh public smoke comparisons](published/public-smokes.json).
- [Full157 public comparison](published/full157-results.json).
- [Pre-verification physical/role account](published/full157-census.json).
- [Matched offline read costs](published/read-costs.json).
- [Sealed artifact manifest](published/manifest.sha256.json).

The experiment lives in isolated branch `codex/issue88-encoding-experiments` and
checkout `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue88-experiments`. Original
Stores, inputs and prior sealed reports remain untouched. The small source changes
are an experimental structural-origin prototype, not an accepted production patch.

Complete run data remain under
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue88-*`.
`issue88-report-1` contains the final report/manifests; the manifest hashes all
large frame images, metadata indexes, snapshots, raw receipts, logs and producer
binaries without copying them into Git. Published compact files preserve sealed
bytes. A pre-verification logical snapshot is kept separately; its allocation is
not substituted for the original acknowledgement.

## Reproduction

All builds/encoders use the existing measurement lock and run sequentially.
`locked_run.py TIMEOUT_SECONDS COMMAND ...` supplies the lock for offline tools;
the public runner acquires the same lock itself and must not be wrapped again.
Use new output directories, never overwrite completed/failed outputs.

- Build `extract/Cargo.toml` and `s1/Cargo.toml` with their lockfiles, offline,
  two Cargo jobs and isolated target directories. S1's exact CLI is in
  [s1/protocol.md](s1/protocol.md).
- `extract_content.py RUN EXTRACT_BINARY NEW_OUTPUT` authenticates source blobs
  and emits S2/S3 input manifests using existing FastCDC/canonical code.
- `content_screen.py --manifest INPUT --output NEW_OUTPUT --arm S2|S3
  --library LIBZSTD --contract contract-v1.md --contract screen-details-v1.md`
  runs the fixed native prefix screen. The source CLI rejects setting changes.
- `test_content_screen.py` exercises the frozen synthetic boundaries and strict
  rejection paths. Supply the same library/contracts, extractor and new output.
- `read_costs.py RUNS_ROOT NEW_OUTPUT` performs the prospectively fixed matched
  read follow-up; `audit_results.py RUNS_ROOT NEW_REPORT_JSON` verifies results.
- Public builds use unchanged `benchmark/fs-bench-pro/shared/runner.py
  --build-host` and `--build-storage-smoke-image`. Samples use the original three
  selectors and full157 selector, exact source/binary/image identities and
  original input/oracle/import/timer definitions. See frozen public contracts.
- `summarize_public.py` and `summarize_full.py` consume receipts only.
  `census_report.py` aggregates the single reused exact-decoder inventory of the
  sealed pre-verification snapshot. No extra product replay is implicit.

Offline frame sizes are encoded potential, not complete product allocation.
S3's small incremental gain does not justify silently discarding chunk reuse or
filesystem semantics. Negative preparations, smoke regressions and all raw reads
are retained. #88 and #87 stay open for the next canonical-preserving direction;
no M5, cloud, merge or release qualification.

# Stride3 fast-iteration fixture and harness

2026-09-10. The explicit `deepseek-stride3` profile is implemented using the existing selected-snapshot generator and real host-owned save/Commit session.

- Exactly53 selected states: original indices1,4,7,…,157; campaign indices1–53 retain their corresponding `full157_index`.
- Direct transitions from the previous selected state; no hidden commits for skipped states.
- Original ten-state and full157 selections remain unchanged. The new profile requires Created for every selected step and uses the existing same-Store historical verification/custody/cleanup path.
- Fixture-only preparation completed in30.59seconds, authenticating all53 selected trees/blobs and original content/type/mode/symlink oracles.
- The focused selection check passed all53 input seals, original manifests and oracle hashes, direct transition inputs and **306,861 oracle path-states**. Python syntax, runner help and diff whitespace checks passed.
- No product/Docker53-state run or Git53 baseline has been performed. The reduction in commit count is66.24%; speedup is not measured.

Prepared immutable fixture: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-data/deepseek-stride3-218c1b81ed39c763ca22/fixture.json`.

SHA256: `3b2c12b682e4892837fc810969e4e90eaa8d756c0d6f738df50746ad478fb73e`.

## Commands

Use the existing qualified matching host/image build and a fresh output directory:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py \
  --storage-smoke deepseek-stride3 \
  --image "$LAYERFS_BENCH_IMAGE" \
  --source-arm candidate \
  --output /absolute/path/to/fresh-stride3-run
```

Verify that same Store through the existing separate verification phase:

```bash
python3 benchmark/fs-bench-pro/shared/runner.py \
  --storage-smoke deepseek-stride3 \
  --image "$LAYERFS_BENCH_IMAGE" \
  --storage-verify-run /absolute/path/to/fresh-stride3-run
```

Runnable fixture check, independent of any product campaign:

```bash
python3 benchmark/fs-bench-pro/shared/test_deepseek_selection.py \
  --fixture /Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-data/deepseek-stride3-218c1b81ed39c763ca22/fixture.json
```

The [fixed contract](stride3-snapshot-contract.md) requires a fresh matched LayerFS control and separate Git53 measurement before reporting a53-state ratio. The [structural full157 result](structural-investigations.md) remains79,790,080B with157original states verified; it is not relabeled or extrapolated from this fixture.

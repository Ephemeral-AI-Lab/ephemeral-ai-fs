# Empty-target host build: concurrency root cause

**Later implementation:** see [qualified host jobs](qualified-host-jobs.md) for the adopted eight-job default, reduced duplicate work, and full runner measurements. The results below retain their original measurement phase.

The dominant avoidable cost is the qualified runner's explicit **`-j2` host
build limit** at `benchmark/fs-bench-pro/shared/runner.py:795`. The machine has
14 logical CPUs. Raising only the scheduling limit to 8 reduced an empty-target
native build from **116.503663 s to 38.222729 s**, **78.280934 s / 67.2%**.
Both host executables are byte-identical; linked schema10/content107 probes pass.
This is a diagnosis and controlled experiment, not an adopted runner setting.

The earlier attribution to optimized LLVM work was incomplete: the expensive
work also had its concurrency restricted by the shared Cargo/rustc jobserver.
The earlier suggestion that compilation-unit/profile changes were the next
necessary step was premature. Test the scheduling limit first.

## Evidence and boundaries

Clean source: `ada559b9b04cd6a9916d04c071e5c07006581b34`, expected branch verified.
No active builds/benchmarks; 332 GiB free. New evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue105-evidence/cold-diagnosis-20260910T160424Z`. Earlier #104 and #105 evidence was not edited.

One run at j2, followed by one prospectively declared run at j8. Both targets
were absent before execution; neither target was seeded. Cargo reported the same
**103 compilation/build-script units**. Registry and OS caches were warm; this
is not a download or cold-machine experiment. The runner-owned lock covers each
build and its separate linked probes. No clean/prune, bootstrap, rustc wrapper,
product changes or product benchmarks.

```sh
/usr/bin/time -l cargo +1.85.1 build --locked --release -j2 \
  -p fs-benchmark-pro -p layerfs-layerstack-store --bins \
  --target-dir '<evidence>/target-j2' --timings
# Second run: only -j2 → -j8 and a new empty target-j8 directory.
```

This measures **native command wall**, including Cargo timing diagnostics and
`time`, not the full runner's source/hash/archive/publication command. Linked
probes are timed separately. No complete-runner j8 total is claimed.

| Metric | j2 | j8 |
| --- | ---: | ---: |
| Native command wall | 116.503663 s | 38.222729 s |
| User + system CPU work | 211.59 CPU s | 210.22 CPU s |
| Average CPU cores used | 1.82 | 5.50 |
| Wall with ready Cargo units waiting for capacity | 79.36 s | 11.95 s |
| Linked schema + format probes, separate | 1.662997 s | 0.748595 s |

The almost unchanged CPU work, substantially higher utilization, earlier starts
and shorter durations, identical unit count and byte-identical outputs support
**scheduling restriction** as the causal change. This is one observation per
setting, not a statistical estimate or a claim that eight is universally optimal.
Cargo unit concurrency excludes internal compiler worker threads; CPU utilization
and mean active Cargo units are distinct metrics. The `time` output reports no
process swaps. Its maximum RSS rises from 599,932,928 to 936,886,272 bytes; this
is not a measurement of aggregate simultaneous memory across the entire tree.

## Where the original 116 seconds go

| Unit | j2 start → end | j2 duration | j8 start → end | j8 duration |
| --- | --- | ---: | --- | ---: |
| Store library | 33.86 → 78.11 s | 44.25 s | 12.35 → 24.69 s | 12.34 s |
| FUSE library | 41.86 → 61.19 s | 19.33 s | 11.98 → 16.54 s | 4.56 s |
| Workspace library | 62.95 → 89.81 s | 26.86 s | 16.00 → 27.12 s | 11.12 s |
| Benchmark executable | 89.81 → 116.45 s | 26.64 s | 27.12 → 38.17 s | 11.05 s |
| zstd native build script | 14.19 → 28.17 s | 13.98 s | 3.86 → 12.28 s | 8.42 s |

These units overlap and must not be added as command wall. The two-slot cap
delays independent work and restricts backend workers even when only one large
crate is active. Store metadata becomes available in 2.66/2.57 seconds and
Workspace metadata in 1.75/1.78 seconds; most of their wall difference is after
metadata emission, not faster parsing/type checking. These are Cargo metadata
boundaries, not exact standalone LLVM timings for those libraries. The previous
separate benchmark compiler diagnostic also measured expensive LLVM passes and
a 0.124-second linker phase.

The required graph includes Store and FUSE dependencies, Workspace, Monitor/SDK,
and the benchmark. The benchmark cannot start before its required outputs are
ready; even with j8 it starts at 27.12 seconds. Remaining work includes zstd's
C compilation, large optimized libraries, and the benchmark's final 11.05-second
compilation. Those are further investigation targets, not proved irreducible
floors.

This is not a full-workspace/test/example build: only the declared two binaries
are selected. The Store library compiles once; the small compactor costs only
0.77/0.70 seconds. Duplicate syn/nix dependency versions exist, but their costs
are small compared with the dominant units; their presence does not explain
the 78-second improvement. No network download appears in either build log.

Cargo and rustc coordinate concurrency through the jobserver; `--jobs` controls
that concurrency and normally defaults to logical CPU count. See the official
[Cargo jobserver documentation](https://doc.rust-lang.org/cargo/reference/build-scripts.html#jobserver).
The current runner supplies `-j2` explicitly, so merely exporting
`CARGO_BUILD_JOBS=8` does not override it.

## What to improve next

The smallest justified next implementation is a bounded, recorded **host jobs
setting**, with 8 as the measured candidate on this machine. Preserve all
optimization/LTO/codegen-unit/debug/incremental/linker settings and keep Docker's
separate resource policy independent. Bind the actual selection to build custody
and test the complete qualified runner plus a real small edit before claiming
new complete-command times. Do not start by splitting crates, weakening release
optimization or adding another cache.

The benchmark's 11.05-second unit here is part of a first build, not a measured
small-edit loop; no sub-10-second small-edit claim follows from this run. The
first build remains **BUILD_SLOW**, even at j8.

## Compatibility

Benchmark SHA256, j2/j8/current existing qualified binary:
`60caa06fd67f0883148bc49c0f320a1f75cbec7fb5ba2770c9c82fc356203c67`.
Compactor SHA256, j2/j8:
`82d94c206243d45e2c64e1fe73cc18f5eca7f0705ced8553633d63bc206fb7d8`.
Actual linked schema10/content107 probes pass for both. Exact commands, inputs,
qualification records, CPU counters and interactive Cargo timing HTML remain
in the evidence directory. Production runner/source is unchanged in this pass.

# Qualified build loop — issue105

Status: implemented and measured; #105 remains open; optimized small Rust recompilation remains **BUILD_SLOW**.
No benchmark compiler/profile setting or LayerFS operation was changed.

The relevant improvement is automatic reuse of proven-compatible dependencies in
a fresh source-isolated target, using #104's existing independent-copy recipe.
It does not make LLVM optimize the benchmark crate faster. Docker additionally
keeps standalone workload compilation in an input-specific layer and excludes
host-only family Python/shell launchers from its compilation context.

## Scope and custody

Checkout: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb`, branch
`codex/issue100-40mb-experiments`, initially clean at
`be35274f14ae1bb9e796aabe48fc7f58dab3b603`.
The initial process inspection found no Cargo/benchmark work; 333 GiB was free.
The runner-owned measurement lock was available.

Evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue105-evidence/build-loop-20260910T154259Z`.
`declaration.md` predates source edits and timed builds; `amendments.md` retains
prospective clarifications. `attempts.jsonl` is append-only. Every timed child
has its exact argv, source status, launch/exit timestamps, monotonic wall,
exit code and a flushed stdout/stderr log. One attempt per declared cell is
retained; no best-run selection or statistical significance is claimed.

Both implementations ran in the same disposable detached source worktree at
`<evidence>/source`, beginning at the initial commit. `candidate.patch` plus
`test_build_reuse.py` records the candidate. Neither the user's Rust files nor
any sealed #104 campaign file was edited. Native targets and archived binaries
remain separate within the disposable source worktree.

## Exact boundaries and historical correction

From the disposable source repository root:

```sh
python3 -u benchmark/fs-bench-pro/shared/runner.py --build-host
python3 -u benchmark/fs-bench-pro/shared/runner.py --build-image
```

Host native command, unchanged by this implementation:

```sh
cargo +1.85.1 build --locked --release -j2 \
  -p fs-benchmark-pro -p layerfs-layerstack-store --bins \
  --target-dir "$SOURCE/benchmark-results/host-store/builds/native-$LAYERFS_COMPILATION_SEAL"
```

Complete command wall starts immediately before launching the Python runner and
ends after child exit/output drain. It includes source seals, native compilation,
dependency copying when applicable, linked schema/format probes, hashes,
publication and binary/image archiving. Snapshot creation and the separate
intrusive diagnostic copy are outside these measurements. Tee/progress overhead
is included. The first four baseline UTC start fields precede untimed git-status
collection; their monotonic `wall_ns` boundaries remain correct. Later UTC starts
are sampled directly beside the monotonic start.

The #104 command in `build-followup.py` for its **27.563837 s** Rust edit was
**direct Cargo against an already populated target**, not the complete qualified
runner. Its restore was also direct Cargo (**27.479643 s**), followed by a
separate **2.211249 s** unchanged qualification. Its probe added an unused
function, which can be removed by optimization. It proves Cargo recompilation,
but is weaker than evidence of a changed emitted executable. No speedup ratio
against that historical small-edit number is claimed here.

This experiment changes one existing emitted diagnostic in
`benchmark/fs-bench-pro/src/main.rs`:
`container cgroup snapshot failed` →
`container cgroup snapshot failed (issue105 edit)`.
The main executable's hash changes and matches the same edited baseline after
candidate compilation. Restoration triggers real recompilation and restores its
original hash. Operations, fixtures, timers and verifiers are unchanged.

## Measurement table

| Complete command boundary | Before (s) | After (s) | After status | Comparable reduction |
| --- | ---: | ---: | --- | --- |
| host-first | 117.596147 | 119.659660 | BUILD_SLOW | — |
| host-warm | 1.604768 | 2.072746 | PASS | — |
| host-edit | 115.411967 | 28.658360 | BUILD_SLOW | 86.754 s / 75.2% |
| host-restore | 27.966826 | 27.937846 | BUILD_SLOW | — |
| image-first | 98.685821 | 86.744003 | BUILD_SLOW | — |
| image-warm | 1.654027 | 3.208511 | PASS | — |
| image-python | 10.174413 | 4.459642 | PASS | 5.715 s / 56.2% |

Candidate exact native command walls: first **116.685321 s**, warm
**0.132262 s**, edit **26.337922 s**, restore **26.065424 s**. The edit's
independent dependency copy takes **0.346641 s**, inside its 28.658360-second
command. The remaining **1.973797 s** is source/probe/archive/Python work combined,
not a measured standalone qualification phase. Python/docs-only host qualification
is **2.765413 s**; final restored-source qualification is **1.996088 s**.

The old image's Python edit runs an **8.1 s** combined native step, with Cargo
reporting **0.04 / 0.03 s**. Candidate workload and Cargo RUNs are both **CACHED**;
image labels, source validation and archive hashes still execute as required.
The 56.2% observed command reduction includes ordinary BuildKit/context/archive
variation; it is not a 56.2% faster compiler. Warm command increases are retained
and remain below 10 seconds; they are not hidden or attributed precisely.

Every row has n=1 per arm, so its median and min–max both equal the printed
observation. First host targets have no native outputs; installed registry and
OS caches are warm. Candidate first use deliberately fails closed on the older
identity without a dependency seal. These are dependency-cold compilations,
not cold machines or fresh registry downloads. First image rows use empty
per-key native Cargo caches, cached registry/base-image layers and ordinary
BuildKit metadata resolution. First/warm differences are not treated as a
compiler speedup claim.

## Where the time goes

The baseline normal qualified edit selected a new empty native target and
rebuilt the dependency graph. The candidate copies only compatible dependencies
into its own target, excluding `fs-benchmark-pro*` and `fs_benchmark_pro*`
artifacts, including executable, dep-info and fingerprint paths. Cargo then
compiles only the changed benchmark crate. Source input seals are checked again
before output publication.

The intrusive diagnostic used its own copied target, `cargo rustc --timings -vv
-- -Ztime-passes`, and `RUSTC_BOOTSTRAP=1` only in that child. It reported
**26.417 s** for the benchmark compiler, **21.399 s LLVM passes**, and
**0.124 s run_linker** (**0.186 s complete link phase**). These are overlapping
compiler phases, not additive timings. The instrumentation changed
proc-macro2's build-script environment and caused some diagnostic-only dependency
recompilation; the **67.519 s** diagnostic command is excluded from clean results.
Its fingerprint log and Cargo timing HTML are retained. Linker tuning is not the
remaining opportunity indicated by this evidence.

The separate instrumented baseline warm qualification recorded Cargo command
**0.132 s**, linked schema probe **1.530 s**, and integrated format probe
**0.186 s**. Other source/hash/archive/Python work is part of the enclosing wall;
its individual durations are unavailable, not zero. Clean baseline Cargo
figures come from Cargo's rounded log; candidate identities now also record
exact `native_build_wall_ns`. Standalone linker metrics in clean runs are
unavailable. `results.json` preserves those distinctions and crate lists.

On Linux, the old combined RUN executes standalone `rustc` after a host-only
Python context change even when both Cargo commands are warm. The new workload
stage includes the workload tree and all included family Rust sources, before
product-crate COPYs or native-cache ARGs. `.dockerignore` excludes only host-only
family Python/shell files. Changes to real workload/family Rust still invalidate
compilation. No compiler flags or workload self-check requirements were removed.
Image archive/hash work now remains under the runner-owned lock as well.

## Compatibility and checks

All original/restored and edited host binaries are byte-identical between
baseline and candidate in the same source snapshot. All host qualifications pass
actual linked **schema10/content107** probes. The unedited benchmark SHA256 is
`9d7546aa522f856437aa6dc2350439808d1d8f662b53d23fa8fa4a0d7b54cf6f`; edited SHA256 is
`5aeb6c104cfe606916449d62b430598f0558982085a9203dcdc8995c8a43580a`.
The compactor stays `45079096cecf874d704b09e06af25b6cff915e47c7738ac541f24f81f4102e75`.

All four qualified Linux image archives have identical executable hashes:

- `fs-benchmark-workload`: `e2ae9a6b0859e3542f58dbf93e9088fe9f94e047cd205da8d927e3219d21aa9a`
- `layerfs-daemon`: `982b20a100535363884ca02060c1db3e952fe7614f2d558e977f7fc64f2625b2`
- `layerfs-fuse`: `f0b194ab3dbfab50005d0cc0cd818f8b733e896869281dad472d2116f69c3686`

The candidate native seal is `b68991c1a61f0e6e53f97cb8c3cdde6d2d6eb40cb918abbb46e3b00d5da4f32a`;
the edited native seal is
`bbbbd605af387758cfb8075592a6bfc4811f1ad729b3ccdf31d2e769fdf17518`.
Both share dependency seal `b8367039a6b78800aaa12a22c1efec85bba2a61d646147361ee963619ddc7fef`.
Product seal remains `b1a94e2223b1cd6c0eedffa3c6c60eca7134727c45b9018e1cea518cdf6d3dd5`.
Full source/build/image identities and exact target paths are in `results.json`;
the external evidence retains complete identities including all probe records.

The workload-source diagnostic really rebuilds standalone Rust (**8.2 s** build
step; **8.981745 s** command), emits a different hash
`cb8aa2794b96961992f2c1a05938bab5e6e0d5cae3772ba74190e46e8e569e8f`, and passes
self-check. Its diagnostic image is not a qualified benchmark image. Source bytes
were restored; the previously archived original candidate image remains intact.

No existing #104 measurement is invalidated by this build-only change: product,
operation, fixture, timer and verifier bytes are unchanged, and matched-snapshot
executables prove before/after build equivalence. #104's original source/build
matrix is not relabeled or merged with this experiment. Cross-directory host
hashes are not asserted equal: build/source paths are different. The primary
checkout's old qualified outputs retain their old identities; the measured
candidate outputs live in the disposable source worktree. Its next upgraded
primary-checkout build may take the measured first-use path.

The #104 manifest SHA256 is still
`c23e4c2f999e7fca874ea1797dbc2a0d5c5cb4b33458dfa32da42f300992468b`.
No sealed campaign file was written, and no product benchmark replay was needed.
Final checks find restored Rust sources, an available runner lock, and no running
containers. The final process inventory is retained.

Focused validation command from `benchmark/fs-bench-pro/shared`:

```sh
python3 -m unittest -v test_build_reuse test_runner test_runtime test_layout
```

**35 tests passed.** New checks cover benchmark-only compatibility, product Rust,
SQL, manifest/features, build scripts, Cargo config, `.dockerignore`, workload,
toolchain and build-environment invalidation; independent dependency copying;
benchmark-artifact exclusion; producer hash rejection; and archive lock scope.
Live Python/docs qualification additionally checks unchanged native/dependency
seals, changed full-source seal, no native recompilation and the same binary.
The workload-input diagnostic checks a real source edit reruns standalone Rust
compilation and passes self-check, then restores the exact source bytes.

## Remaining limits

The preferred 10-second edited qualified build target is still missed:
**BUILD_SLOW**. About 26 seconds of optimized benchmark compilation remains;
LLVM dominates and the linker is inexpensive. No lower optimization, changed
LTO/codegen/debug/incremental settings, faster substitute `check` executable,
crate split, compiler upgrade or alternate linker was adopted. A benchmark-crate
split would change compilation units and needs separate binary/performance
qualification; it is not a silent build-helper optimization.

Dependency reuse deliberately covers benchmark Rust edits only. Product or
dependency changes, configuration/toolchain changes, and older identities without
a dependency seal still take the conservative fresh-target path. Native keys
remain conservative across platforms and source paths. Broader dependency-DAG
reuse would require another demonstrated compatibility boundary; no generic
cache framework was introduced. Existing targets are not deleted.

The sealed #104 campaign remains attached to its original source/build matrix;
this report neither requalifies it as one final source nor restarts its completed
performance campaign. No #106/#107 work, release, tag or deployment was performed.

The concrete remaining blocker for #105 is the **28.658360-second** qualified
benchmark-source edit, dominated by optimized LLVM work. This bounded investigation
is complete; a sub-10-second result would need a separately justified compilation-unit
or toolchain/profile investigation with fresh compatibility evidence. No cold/full
release build within 10 seconds is promised.

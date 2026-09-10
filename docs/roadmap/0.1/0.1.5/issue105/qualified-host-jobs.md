# Qualified host jobs: implemented after root-cause diagnosis

The host build now defaults to `min(8, logical CPU count)` instead of the fixed
`-j2`. Explicit `CARGO_BUILD_JOBS=1..8` is supported; invalid/out-of-range settings
fail before building. The actual host job selection is recorded in identity and
bound into both native and dependency compatibility seals. Docker retains its
independent two-job policy. No optimization, features, LTO, codegen-unit count,
debug-info, incremental or linker setting changed.

[The controlled cold-build diagnosis](cold-root-cause.md) established causality
before implementation: changing only jobs from 2 to 8 in separate empty targets
reduced native command wall **116.503663 → 38.222729 s (67.2%)**, with almost
unchanged CPU work and byte-identical benchmark/compactor executables. The
previous LLVM-only explanation missed the jobserver restriction.

User then authorized aggressive removal of redundant build work. Besides the
job limit, native/dependency hashing now uses one input traversal and one set of
three toolchain subprocesses instead of two traversals and six subprocesses per
source check. Both seals still receive exactly the same ordered configuration
bytes. `seal-equivalence.json` proves equal hashes before/after this refactor.
Required before/after source checks, actual linked probes and original binary
archives remain. No dedicated timing claim is made for this smaller reduction.

## Complete qualified runner proof

| Complete runner command | Native command | Complete wall | Status |
| --- | ---: | ---: | --- |
| j8-host-first | 41.186993 s | 44.015306 s | BUILD_SLOW |
| j8-host-warm | 0.128635 s | 1.862476 s | PASS |
| j8-host-edit | 11.852020 s | 14.011688 s | BUILD_SLOW |
| j8-host-restore | 11.890797 s | 13.536329 s | BUILD_SLOW |
| candidate-host-python-docs | 0.133654 s | 1.815571 s | PASS |
| candidate-host-final-qualification | 0.134535 s | 1.820964 s | PASS |

Command: `python3 -u benchmark/fs-bench-pro/shared/runner.py --build-host`.
One prospectively declared observation per cell, same emitted-string Rust edit
as earlier #105, full runner child-launch through exit/output-drain boundary.
The first native target was absent, with no seeding and warm registry/OS caches.
Real Rust edit copies only compatible dependencies to an independent target and
recompiles `fs-benchmark-pro`. Its binary changes; restore recompile reproduces
the first hash. Python/docs-only source changes do not recompile native code.

This new proof uses a new detached disposable source worktree to preserve sealed
prior evidence. Accordingly it is not a strict same-path comparison with the
earlier 119.659660-second complete-runner observation; no percentage is claimed
for that comparison. The controlled j2/j8 diagnosis above is the matched-path
scheduling comparison. The 38.22-second diagnostic is native-only and must not
be substituted for the **44.02-second complete qualified command**.

Original/current restored snapshot binary:
`9437c5026710a83ed187800e63fd049f2ffbfb6023d2f370e06b6dd8c5dc8a4f`.
Edited snapshot binary:
`5a797853585b930376c4a2cdd61d4d69c3d648a4816cba0a6400cedbebe854a4`.
Actual linked schema10/content107 probes pass for every host qualification.
Cross-directory hash equality is not asserted. The controlled diagnosis used
one source directory and proved j2/j8 byte equality there. Product source/seal,
benchmark operation, fixture, timer and verifier bytes remain unchanged. No
existing #104 benchmark evidence is invalidated or replayed.

Evidence root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue105-evidence/qualified-j8-20260910T1611Z`.
Initial commit is `ada559b9b04cd6a9916d04c071e5c07006581b34`; exact candidate patch,
source identity, append-only attempts, logs, original edit bytes, archived
binaries and probe identities are retained. No sealed #104 or earlier #105
evidence directory was written. Primary-checkout Rust is untouched; it retains
its original qualified artifacts until the next normal build.

**36 focused tests pass** (`test_build_reuse`, `test_runner`, `test_runtime`,
`test_layout`). Coverage includes job bounds/default selection, job-count seal
invalidation, all prior native/configuration invalidation boundaries, dependency
copy isolation and exclusion, linked-schema rejection and archive lock scope.
No product benchmark was run.

## Remaining cost

First build and real Rust edit remain **BUILD_SLOW**. The edit is now 14.01 s
complete with about 11.8 s in native compilation. Further work should follow
measured remaining compiler/qualification phases. Do not remove required probes
or weaken optimization to manufacture a sub-10-second result. The cold trace
shows that removing the compactor would save under one second; it is not the
large opportunity. No crate split or cache framework was introduced.

The stronger job-aware key deliberately invalidates older qualified target
identities once. The shared host/Linux key remains conservative, so its migration
can also cause a Linux cache miss despite unchanged Linux executable inputs;
Docker compilation flags themselves remain unchanged. Subsequent compatible
benchmark-source edits continue to reuse dependencies.

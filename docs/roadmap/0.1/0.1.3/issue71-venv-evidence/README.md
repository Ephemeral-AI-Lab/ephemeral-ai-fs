# Issue 71 real-directory proof

This standalone probe exercises public SDK initialization and managed Linux/FUSE Workspace Exec + Commit. It keeps SQLite, canonical construction and publication on the macOS host. It is diagnostic evidence, not a replacement for the registered benchmark families.

Build with the same compiler and locked dependencies for both comparison arms:

```sh
cargo +1.85.1 build --release --manifest-path docs/roadmap/0.1/0.1.3/issue71-venv-evidence/probe/Cargo.toml
```

The native invocation takes a source directory and an **absent** output directory whose parent exists. Output cannot be inside the source. Setup and verification are outside the `Client::initialize_layerstack` timer:

```sh
layerfs-directory-proof /absolute/source /absolute/fresh-native-output
layerfs-directory-proof verify /absolute/source /absolute/fresh-native-output/store.sqlite
```

`verify` reads every regular-file byte, checks EOF/length, exact directory entries, symlink targets, normalized portable modes, and nanosecond mtimes. For an existing registered namespace Store, set `VERIFY_STACK_NAME` to its recorded LayerStack name. For Workspace output, set `VERIFY_BRANCH` to the branch ID printed by that invocation. Root IDs differ between native and Workspace allocation domains; compare observable semantics across them, exact canonical roots within one surface/seed.

For Workspace testing, run `python3 prepare.py /absolute/source /absolute/input.tar` from this evidence directory. This prepares a PAX tar archive outside measurement, preserving nanosecond timestamps in PAX `mtime` fields, all file bytes, modes, entries and symlink targets. Do not normalize the fixture to improve cache hits. Build the matching image using the existing `benchmark/fs-bench-pro/shared/runner.py --build-image` path. The probe creates one owned container with 2 CPUs / 2 GiB / 256 PIDs, copies the archive outside the projection, creates an empty LayerStack and Branch, then runs ordinary tar extraction through public Exec on real FUSE and ordinary Commit:

```sh
layerfs-directory-proof workspace /absolute/input.tar /absolute/fresh-workspace-output IMAGE
VERIFY_BRANCH=PRINTED_BRANCH layerfs-directory-proof verify /absolute/source /absolute/fresh-workspace-output/store.sqlite
```

Create, Exec, Commit, Exec+Commit and lifecycle are separate timings. Exec+Commit includes canonical construction performed during writes; lower Commit time cannot be claimed as a full-workflow improvement by moving work into Exec. Container preparation/archive delivery/build and full verification are excluded and must remain explicit. The probe requires daemon transport, no host binds, clean Workspace/Exec counts, no observed container swap/OOM, and removal of the owned container. An external supervisor should bound the whole process and clean only its printed, managed container on timeout.

Use the shared `layerfs-infra-measurement.lock` during builds, measurements and verification. Freeze paired sample order/count and record source/binary/image/lock identities before collection. Fresh output per sample; retain failures and every result. Default-option performance and diagnostic-counter runs stay separate. OS caches are uncontrolled; no cold-disk claim. The directory is trusted immutable test input during a run, not a coherent snapshot of concurrent native mutations.

Native regression inputs use the unchanged existing `namespace-fixture` generator and `namespace-init-diagnostic` entrypoint for all four profiles. Keep generated fixtures, Stores and archives outside the repository; only compact reports, receipts and this probe belong in source control.

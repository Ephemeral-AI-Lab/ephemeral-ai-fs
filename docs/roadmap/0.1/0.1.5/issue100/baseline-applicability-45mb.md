# Baseline applicability for the 45-million-byte investigation

Read-only audit on 2026-09-09 at primary source `2b950278b`, before the new
bounded-predecessor candidate. All three successful ten-snapshot baselines remain
applicable. No controls were rebuilt, histories replayed, or Stores reopened.
These observations support exploratory comparisons, never release admission.

Evidence root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence`.
The immutable `evidence-manifest.json` SHA256 is
`4121768f10b3924a4e24a537566ce9d6bfd99496f637b780286a725b9249e500`.
Every **196/196** listed file hashes correctly, including saved binaries,
identities, command receipts, phase evidence, image inspections, Git objects,
and LayerFS Stores after verification. The pre-verification Store hashes differ
from their final verification hashes because verification lifecycle writes are
outside the frozen measurement window; do not compare those two lifecycle
points as if they were identical bytes.

## Source and harness

All **110/110** recorded benchmark source hashes match across v014, existing
v015, and the primary worktree at the audit. The compiled workload and public
Init/Exec/FUSE/Commit/verifier remain unchanged. Git diff confirms saved v014
`crates`, Cargo manifests/lock and tools equal release
`101fa273d815f3aaedb0e06ba0de7b0777d83def`; saved v015 equals the tested
SmallContent plus exact-CAS fix `7ca59e244`.

| Seal | Released v014 baseline | Existing v015 baseline |
|---|---|---|
| Source commit | `26a80a8efae63b7606c845f20c69aa3a1ccef292` | `bb2474ae88fa60a0d62e689f419edfe1d568e255` |
| Product | `891067915b4152dd42e7510ef374a3d8a8827ead03a597051335c933d0485958` | `557a420d05d1b6cbd18e1b06c55a824c2fbe0ca2fdee596c3231398496a74f8a` |
| Combined source | `16159d88c1b8c0cf3b6887168690a8f3cf368a70893d9cedc2f9f3012f656987` | `aa8022757f47b42e69cf6d5315da30acce16c74bc5e40521e20d04231b44a4fc` |
| Host binary | `ecb4d5bec62c061b32a92fb1d2b2956390ac55042ac0e7611286ce24d6c3d915` | `b1f8fc5b9a033e447f50596fe03178e0c34dbb53ef56c7d91a8b10832c68f780` |
| Image ID | `sha256:a84448f29032a904171ed50edb445dbbefb10074c974cfe79ddeeff00e571773` | `sha256:4bf528c07e96846c5d7ed6c10ff3f31db4eb0e4b5ba23e898daeb6aa194a2675` |

Saved dirty patches are retained; the v015 patch is report-only. Store schema
hashes differ across products as expected; the benchmark harness is identical.
The ten contract hash remains
`9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50`.

## Fixtures, timing and environment

Both saved arm identities contain exactly `fixture.json`, SHA256
`6a28059656b671edc6f068d578a0f4670bfc7519aa0f54ee9dda10941cc7f64a`.
All **33,967** prepared-input file hashes and **10** original oracle hashes
were checked against the actual files, with zero mismatches. Original manifest
hash remains `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`;
source tip remains `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. The ten indices
are exactly **1, 18, 36, 53, 70, 88, 105, 122, 140, 157**.

Current platform equals recorded `macOS-26.4.1-arm64-arm-64bit-Mach-O`.
Recorded topology remains host Store/coordinator/publication/spool and Linux
live core/FUSE/workload; containers have 2 CPUs, 2 GiB RAM, no swap, 256 PIDs,
and no data-sharing binds/mounts. Public timers and the 600/120/600-second
phase/operation/verification limits are unchanged. Cache profile is
`fresh-store-existing-os-cache-uncontrolled`; no controlled cold-cache claim.
Lifetime RSS and boundary disk samples are not continuous phase-local peaks.

All four LayerFS performance/verification results have ten records, successful
coordinator closure and container removal. Actual outcomes are 10 Created /
0 UpToDate per arm. Historical verification covers 58,860 path states and
327,885,165 logical bytes from the same measured Store. Final allocated bytes
remain Git **38,223,872**, released v014 **67,145,728**, existing v015
**66,105,344**. Initial allocations are **8,192 / 69,632 / 69,632 bytes**.

Git 2.47.1 and its saved native binary hash
`5bebfad20c935dda15d9f54856422e4506bade025a575e8e0e5d416b9594349a` remain
recorded in `git-binary.json`. Its retained ten-tree repository has exact object
membership, strict fsck and full original-oracle proof. Compression 6/window
10/depth 50/threads 2 are unchanged. Construction and later packing timers have
different boundaries from LayerFS foreground calls, and Git retains narrower
filesystem metadata. No Git save-latency ratio is supported.

## Qualified full157 reuse

At `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-evidence`, both existing
full157 verification manifests hash-match **803/803** files and both saved
host binaries match their identities. The current full157 contract hash equals
the saved `e33e6ac09ed44cc5b9405800b55d5c16d09dd1ca3446a33298bb75124105200d`.
Saved full157 product seals equal the corresponding ten-state products above.
The released control source was `1a8e553d1772183e7504a9c52fa392ca20afb51f`;
its combined seal was
`e48580f3d9c54f19d268b2c9930b856b16c8982a26496678e094f16c8e55f251`.

The saved full157 harness is **not byte-identical** to today's harness.
Its only changed recorded file is `shared/storage_smoke.py`; review of the diff
from `7ca59e244` shows registration and conditional routing for `deepseek-ten`.
The full157 fixture path, operation route, timing boundaries, historical
verifier and 14400/300/14400-second limits are unchanged. The owner permits
reuse where no material measurement change occurs; this ten-only extension
supports qualified reuse, not a claim of byte-identical files. Final confirmation
must revalidate prepared full157 inputs and execution environment through the
existing runner. A material later change requires one affected matched control.
Historical release-closeout full157 walls are unpaired archival observations;
use issue100's matched control, not that older release timing headline.

## Next candidate custody

Use a new output and newly sealed binary/image. Exact command receipts and
`run-layerfs.py` in the evidence root show the existing serialized sequence:
build host, preserve binary/identity, build image, run performance, run
`python3 docs/roadmap/0.1/0.1.5/issue100/census.py NEW_RUN`, then invoke the same
runner arguments with `--storage-verify-run NEW_RUN`. Runner and census each own
the shared lock; do not nest another acquisition. Census is read-only on the
Store and creates `NEW_RUN/census.json` exclusively. Keep the old censuses and
raw measurements immutable; the schema-9 census update is an additive
post-measurement diagnostic and does not alter public timing or fixture behavior.

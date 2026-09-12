# v0.1.5 verification and source applicability

> **Status:** LayerFS 0.1.5 release record.

## Measured product and evidence base

| Field | Value |
|---|---|
| Measured product commit | `1ff1f2dddeb60493953311de316fa5bec4634a1a` (clean, == `origin/main` at freeze) |
| Host binary SHA256 | `c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295` |
| `LAYERFS_SOURCE_SEAL` | `a555211fdc6e96e025764eba2c2c36f709baa7e9446840a07a8669935252fec3` |
| `LAYERFS_PRODUCT_SEAL` | `276c5970aabf485594d90ae30920b3cdb310134a7572589c558063a9d52ce093` |
| Image | `sha256:c83085b897e272204e5477e66b33ec9b328af568299448872441e7a676ce1761` |
| Build identity probe | `schema_version: 10`, `storage_policy: ordinary`, SQLite 3.51.0, status PASS (recorded in every campaign receipt's `integrated_format_probe`) |
| Native gate on that tree | `tools/test-fast.sh` **PASS — 530 tests/benchmarks, 118 s, exit 0** (inherited from the #120 campaign; not re-run for this release) |
| Campaign | `docs/roadmap/0.1/0.1.5/issue120/final-report.md` (227 registered selections); receipts under `benchmark-results/host-store/issue120/` |

The build identity probe is the product's own format probe
(`benchmark/fs-bench-pro/src/storage_integrated.rs`), which reports the observed
schema version and storage policy for the exact measured binary. It reports
**ordinary schema 10**, which is what this release contract and manual claim; no
document in this release asserts schema 8 or schema 9 as the new-Store format.

## Exact-source applicability

`3e308a8f2` (rustfmt 1.96 reflow plus four named Clippy lint fixes) landed after
the final treatment `b5f089ebcd6fa2fa939feb9bccc5300ca8ede798820fe8fd9aace8799fe4ec0b`
that produced the ordinary full157, historical access 11 + 11, default-budget
K32000, K6000 boundary and default-budget frontier evidence. It was decided
**behavior-neutral before any campaign family was collected**, by line-by-line
review of every hunk: a transparent type alias, four identity reborrows
(`as_deref_mut`→`as_mut` on `Option<&mut T>`), an identical-control-flow
spill-merge re-raise, and a dropped `u64::from` on an already-`u64` operand; the
remainder is formatting. The full record is
`docs/roadmap/0.1/0.1.5/issue120/applicability-3e308a8f2.md`.

Therefore the final-treatment evidence is **reused with provenance** rather than
re-collected:

| Reused item | Producing treatment | Value |
|---|---|---|
| Ordinary full157 stride-1 construction | final treatment `b5f089eb…` | complete command 763.218 s, 157/157 steps PASS |
| full157 same-Store verification | final treatment | 735.749 s; 157 states / 904,143 entries / 4,936,693,030 B; all oracles PASS |
| Physical census | final treatment | allocated 83,951,616 B; apparent 82,583,552 B; pack rows 1,058; `store_sha256 88b4fe70…` |
| Git157 read-only control | final treatment | 157 checkpoints, mapping verified; allocation-layout WARN retained (56,197,120 vs recorded 56,373,248 B) |
| Historical access 11 performance + 11 proofs | final treatment | outer walls 2.525–3.232 s, verification 2.48–2.87 s, all 22 PASS inside the unwaived 15 s envelope |
| Default-budget K32000 route + independent verification | final treatment | 32,000 edits / 92,821 pieces / charge 2,048,000 under the unchanged 2 MiB budget; verification PASS in 68.576 s |
| K6000 boundary + default-budget frontier proof | final treatment | K6000 PASS (14.61 s); frontier PASS at exactly 2B, 4B and 8B |
| 16 registered campaign cells | `6693224e…` affected-rerun (3 cells) and `fsync-qualified` `440ae2c4…` (13 cells) | see the [closeout](benchmark-closeout.md); every reused row names its treatment |

Two registered `store_footprint` cells that appeared in the reuse list were
re-collected fresh by the campaign because #107 changes exactly their measured
quantity (pack layout); that recorded deviation is preserved, together with the
observation that the archived `binary-archive/6693224e…` identity seal differs
from the #118 record although the binary bytes are identical.

**Tag-tree delta.** The `v0.1.5` tag resolves to the final documentation commit
of this release. Relative to the measured product commit `1ff1f2ddd`, that tree
differs only by release documentation, the release-policy correction, the
versioned manual, the changelog, the `release-notes/0.1.5/` record and the
workspace version bump `0.1.4` → `0.1.5` (12 project-owned packages in
`Cargo.lock`; imported packages, versions, features and checksums unchanged). No
Rust implementation, schema, codec, public-operation, workload, timer or
verifier source differs, so no measured result is invalidated and no product
identity changes. A product change would have required affected
requalification — none was made.

## Release preparation checks run in this closure

Exactly four commands were run for this release (no benchmark, no proof, no test
suite):

| Gate | Result |
|---|---|
| `cargo build --workspace` | **PASS** — all 12 workspace crates compiled at `0.1.5` (dev profile, 16.86 s); `cargo build --locked --workspace` PASS |
| `cargo fmt --all -- --check` | **PASS** — no formatting change required |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **FAIL — pre-existing on the inherited tree; not fixed, and not introduced by this release.** All findings are inside `#[cfg(test)]` code (`crates/layerfs-layerstack-store/src/objects/admission.rs` `#[cfg(test)]` thread-local, `…/admission/{metadata_tests,small_candidate_tests}.rs`, `…/src/{layerstack,objects}.rs` test modules, `crates/layerfs-fuse/src/live_owner.rs` test helpers, `crates/layerfs-workspace-core/src/{lib,namespace}.rs` test modules). The same command fails identically with the version bump reverted to the inherited `0.1.4` tree (exit 101, same lint inventory) under both the default 1.96.0 toolchain and the release build toolchain 1.85.1. The repository's CI-enforced form `cargo +1.96.0 clippy --workspace --locked -- -D warnings` **PASSES** (8.93 s) on this tree. No source was changed to silence test-only lints: doing so would alter the measured product identity that this release's exact-source applicability depends on. |
| `git diff --check` | **PASS** — no whitespace error, conflict marker or stray CR |

`docs/general/release-policy.md` still lists warning-denying Clippy among the
release requirements. The requirement is satisfied in the form the repository
enforces in CI (library targets, `--locked`); the broader `--all-targets
--all-features` developer command surfaces pre-existing test-code lint noise
that this documentation-and-closure release deliberately did not touch. That
distinction is recorded here rather than presented as a clean pass.

The `0.1.4`-style preparation checks that require a test-suite execution
(native suite, doctests, dependency audit) were **not** run in this closure: the
issue's mandate forbids test-suite executions, and the native gate on the
measured source is inherited from the #120 campaign (`tools/test-fast.sh` PASS,
530 tests, 118 s).

## What is not verified by this release

- No new performance, storage, reliability or endurance measurement was made.
  Every number in this release record is inherited from the #120 campaign or
  from #118, with its producing treatment named.
- Endurance (600 s sustained proof) is not qualified.
- The reused final-treatment evidence was collected on the final treatment's
  binary; it is applied to the candidate under the recorded behavior-neutrality
  decision, not re-measured on `c55daf13…`.
- The two recorded custody/identity deviations (the re-collected store-footprint
  cells and the archived-binary seal mismatch) remain open observations, not
  resolved findings.

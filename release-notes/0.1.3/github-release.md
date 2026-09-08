# LayerFS 0.1.3 Developer Preview

LayerFS 0.1.3 is a benchmark-driven redesign of Commit processing, temporary backing, live Workspace execution, and authenticated reads.

## Highlights

- Shared live Workspace core for FUSE operations and SDK edits, with commands and handles continuing across live-FUSE Commit boundaries.
- In-place checkpoint installation, ordered namespace finalization, shared backing segments, and owned object admission remove repeated reconstruction, file cleanup, copying, and spill readback.
- Payload, CAS/CDC, and Workspace-reuse scaling improvements with separately recorded qualification campaigns.
- Approximately **3× faster measured high-tier Git workflows**: 5.83→1.85 seconds and 14.12→4.64 seconds in the corrected comparison (one baseline observation, three-run final medians); backing requests fell about 92%.
- Repairs for stale mapped-page writeback over SDK edits, failed-owner Discard, hard-link preservation, and post-publication presentation recovery. Release qualification additionally fixed premature SDK-protection retirement while admitted writebacks were still queued.
- **198/198 performance cases and 226/226 routine proofs pass across 17 families.** Final mixed-file lifecycle observations include create 1,000 files/100 MiB in **0.990 seconds** and delete in **0.259 seconds**. These are single observations, not latency guarantees.
- Simplified benchmark infrastructure and bounded history verification with explicit coverage.

Read the [numbered changelog](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.3/docs/releases/v0.1.3/CHANGELOG.md), [technical explanation](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.3/docs/releases/v0.1.3/engineering.md), and [every-family/every-case benchmark tables](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.3/docs/roadmap/0.1/0.1.3/checkpoint-evidence/report.md).

## Upgrade and limits

**Schema 4 is migrated to schema 5 on connect; old binaries cannot reopen the upgraded Store.** Keep a separate pre-upgrade backup and use matching 0.1.3 SDK/CLI, owner, daemon, and FUSE builds. Canonical object identities remain unchanged. Materialized Workspaces retain the active-execution Busy restriction.

This is a **source-only Developer Preview**, following the existing release model. There are no published crates.io packages, executables, or runtime images. Crash/power-loss durability remains unsupported. Git latency targets remain missed; history verification includes sampled snapshots, and the separate 600-second endurance test is excluded from the final checkpoint. Historical exploratory results and changed workload recipes remain explicitly identified.

## Verification and assets

See the [verification record](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.3/release-notes/0.1.3/verification.md), [manual](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.3/docs/versioned/0.1.3/README.md), and [release contract](https://github.com/Ephemeral-AI-Lab/layerfs/blob/v0.1.3/release-notes/0.1.3/release-contract.md).

Checksummed source archives, the published benchmark-evidence bundle, `Cargo.lock`, and `LICENSE` accompany this release. `SHA256SUMS` identifies each asset.

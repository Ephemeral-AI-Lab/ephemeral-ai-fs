# LayerFS 0.1.3 release contract

> **Status:** Release candidate for LayerFS 0.1.3 Developer Preview.

This is a source-only Developer Preview. The release tag identifies the complete reviewed source and manual; packages, executables, and container images are built from that source by the consumer.

## Upgrade and compatibility

The public SDK export surface, CLI parser, and daemon command protocol retain their source definitions from 0.1.2, but runtime behavior and the internal FUSE transport have evolved. Use matching 0.1.3 SDK/CLI, owner, daemon, and FUSE builds. Mixed-version live sessions are unsupported.

Store schema 4 is transactionally migrated to schema 5 on connect. Schema 5 adds `workspace_stages(workspace_id, branch_id, root_id)` for prepared-root publication. Canonical object encoding and identities remain unchanged. This is an explicit Developer Preview exception to the earlier blanket patch-level Store-format promise. There is no schema-5-to-4 downgrade path; old binaries reject the upgraded Store. Keep a separate pre-upgrade copy with all clients stopped. See the [storage manual](../../docs/versioned/0.1.3/storage-format.md).

Live-FUSE Workspaces can Commit while commands and open handles continue, subject to the operation cut and supported coherence rules. Materialized Workspaces retain their active-execution Busy guard. Acknowledged operations and SQLite transactions do not establish crash or power-loss durability.

## Evidence boundary

The accepted checkpoint has 198 passing performance cases and 226 passing routine proofs. One 600-second endurance definition is excluded. Sampled coverage, cold SDK verification limits, original failures, and unmet latency targets remain recorded. Final mixed-v3 bulk-create/delete-100 observations are below one second, but one sample does not establish a sustained subsecond guarantee. Git's aggressive latency targets remain missed.

The [engineering account](../../docs/releases/v0.1.3/engineering.md) includes historical exploratory results. They are not newly measured release-source speedups. The release is bound to the accepted checkpoint by unchanged Rust implementation sources, apart from package-version metadata, and by final native, static, live-Docker, and CI checks recorded in [verification](verification.md).

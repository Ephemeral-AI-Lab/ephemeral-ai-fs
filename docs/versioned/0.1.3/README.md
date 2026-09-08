# LayerFS 0.1.3 manual

> **Status:** Released for LayerFS 0.1.3.

LayerFS 0.1.3 is a Developer Preview for local versioned Workspaces. It shares
live filesystem state between FUSE operations and SDK edits, supports Commit
while commands remain active, and improves construction, backing-storage
lifetime, authenticated reads and recovery.

## Read this manual

1. [Quickstart](quickstart.md)
2. [Product specification](specification.md)
3. [CLI reference](cli.md)
4. [Rust SDK reference](sdk.md)
5. [Container runtime](container-runtime.md)
6. [Storage format](storage-format.md)
7. [Limitations](limitations.md)

## Upgrade boundary

Connecting a valid schema-v4 Store with 0.1.3 migrates it in place to schema v5.
This adds Workspace staging metadata while retaining canonical object encodings
and identities. Older binaries that require schema v4 cannot open the migrated
file. Keep an independent, closed-Store backup before upgrading if you need a
rollback path; there is no downgrade operation. See [storage
compatibility](storage-format.md#compatibility-and-migration).

Deploy the SDK, CLI owner, daemon and runtime image from the same release. This
manual does not promise mixed-version live protocol compatibility with 0.1.2.
The previous [0.1.2 manual](../0.1.2/README.md) remains its historical contract;
its unchanged-five-table and blanket protocol compatibility statements do not
describe 0.1.3.

The [changelog](../../releases/v0.1.3/CHANGELOG.md) summarizes the release, and
the [engineering account](../../releases/v0.1.3/engineering.md) explains the
benchmark-driven changes. The [development checkpoint
evidence](../../roadmap/0.1/0.1.3/checkpoint-evidence/README.md) retains exact
source identities, measurement boundaries and verification coverage. Historical
exploratory measurements are not universal release guarantees.

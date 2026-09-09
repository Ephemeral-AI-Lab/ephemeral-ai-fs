# LayerFS 0.1.4 limitations

> **Status:** LayerFS 0.1.4 Developer Preview manual.

LayerFS is for local evaluation and development. Keep independent copies of
important data; it is not a backup service or a hostile-code security boundary.

## Storage and compatibility

- One Client binds one local Store, with one live local Store authority and
  exclusive SQLite ownership. Cross-host synchronization and automatic repair
  are outside this release.
- Publication is committed and readable from the same live local Store process.
  Process-crash, OS-crash, power-loss and disaster-recovery guarantees are outside
  the MEMORY-journal/synchronous-OFF profile. Filesystem fsync does not upgrade
  database durability.
- New Stores use schema 7. Exact supported schema-6 Stores retain schema 6 and
  legacy writes. Published 0.1.3/schema-5 Stores are rejected. No migration or
  downgrade is supplied; see [compatibility](storage-format.md#compatibility-and-migration).
- Use matching-release SDK, CLI owner, daemon and runtime components. A protocol
  version field is not evidence that 0.1.3 and 0.1.4 components interoperate.
- Automatic object garbage collection and deletion are outside this release.
  LayerStack and Branch names remain immutable.

## Live filesystem boundaries

- Managed execution requires a compatible local container runtime. Real Linux
  FUSE requires `/dev/fuse` and the documented privileges; see [container
  runtime](container-runtime.md).
- Each execution starts its own process. A live Workspace and mount may survive
  multiple executions and Commit cuts, but there is no persistent shell-state
  guarantee across independent executions.
- Commit can capture a cut while a command continues. Applications requiring an
  atomic multi-syscall update must coordinate that update themselves.
- Ordinary writes are acknowledged into live buffered ownership; backing flush,
  filesystem fsync and canonical Commit are distinct operations. An acknowledged
  write is not a crash-durable database publication.
- A newly created FUSE file uses direct I/O for its create handle. mmap on that
  handle is unsupported (`ENODEV`); close/reopen restores the normal cached-open
  path. Tested dirty-mapping/SDK coherence does not imply unrestricted POSIX
  compatibility for every program or filesystem feature.
- SDK range-edit batches must be non-empty and target one regular file in one
  Workspace. Namespace and metadata changes remain filesystem operations.
- Clean End refuses dirty state; Discard must be explicit. End does not Commit.
  Applications remain responsible for releasing Workspaces, executions and
  containers they create.
- Retained stage and presentation recovery operate within the documented live
  ownership model; they are not restoration of arbitrary active commands after
  a host crash. OverlayFS projection is outside this release.

## Scale and evidence

Resource limits apply to their named buffers or policies, not automatically to
the whole host process. Native initialization fast paths require eligible input
shapes and retain canonical fallbacks. Ordinary Commit and every initialization
route do not share identical scheduling or admission behavior.

The [terminal qualification report](../../roadmap/0.1/0.1.4/issue95/README.md)
records 198 successful performance executions and 226 routine proofs. The
optional 600-second endurance observation was not run. Full157 completed 157
performance states, 157 retained-history proofs, 158 validation/accounting
records and cleanup. These are fixed-seed, source-bound observations with the
declared verification coverage, not latency distributions or crash qualification.

The qualified product source is `c48bb4903f456136ccbcdba78de38b9042d2755a`;
committed evidence head is `36a5d9da612211cf26f2a23370e9d59afdceb8e2`.
Full157 allocated storage was 184,582,144 B versus 218,116,096 B for the
supplemental equal-retained-state control (15.374% lower). That control is not
the published v0.1.3 performance baseline.

Duplicate-heavy Init improved materially against the adjacent prior candidate.
Unique/scattered imports and other measured cases still regress against v0.1.3.
The bounded comparison reuse can miss when the working set exceeds its allowance.
Three frozen latency targets remain missed. Four historical Git comparisons
remain INELIGIBLE, and the unchanged global comparison report remains INCOMPLETE.
Owner acceptance of this stopping point does not change those classifications.
Host CPU/storage scope differs from container limits; a successful execution or
proof does not establish a performance pass. Full measurement details and
immutable source/binary/image identities remain in the linked terminal report.

## Distribution

Build instructions are in the [quickstart](quickstart.md). Prebuilt binaries,
crates.io packages and runtime images are official release artifacts only when
the release record explicitly lists them with immutable identities. Do not infer
artifact publication from a repository tag or an example image name.

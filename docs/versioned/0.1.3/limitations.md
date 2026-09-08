# LayerFS 0.1.3 limitations

> **Status:** Release candidate for LayerFS 0.1.3 Developer Preview.

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
- Connect migrates an exact v4 Store to v5 in place. Older v4-only binaries cannot
  reopen that file, and no downgrade is provided. See [migration](storage-format.md#compatibility-and-migration).
- Use matching-release SDK, CLI owner, daemon and runtime components. A protocol
  version field is not evidence that 0.1.2 and 0.1.3 components interoperate.
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

The [published development checkpoint](../../roadmap/0.1/0.1.3/checkpoint-evidence/README.md)
records 198 performance observations and 226 routine proofs across 17 admitted
families. These are fixed-seed, source-bound results with explicitly sampled
coverage, not a latency distribution or exhaustive durability qualification.
One separate 600-second endurance definition was excluded and not run.
Collection PASS does not mean every historical latency objective passed.

Historical original and later mixed-file workloads have different recipes and
cannot supply an unqualified before/after speedup. Host CPU/storage scope also
differs from container limits. Benchmark preparation and verifier improvements
are not product throughput improvements. The [engineering account](../../releases/v0.1.3/engineering.md)
preserves those distinctions and the remaining performance targets.

## Distribution

Build instructions are in the [quickstart](quickstart.md). Prebuilt binaries,
crates.io packages and runtime images are official release artifacts only when
the release record explicitly lists them with immutable identities. Do not infer
artifact publication from a repository tag or an example image name.

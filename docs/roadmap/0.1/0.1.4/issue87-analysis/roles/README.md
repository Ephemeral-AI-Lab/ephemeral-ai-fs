# Issue 87 authenticated inventory

This standalone analysis executable opens only a quiescent retained database via
SQLite `mode=ro&immutable=1`. Check active handles/writers and original seals before
and after running it. It does not instantiate a product Store, replay commands,
encode data, migrate, VACUUM, or write the source database.

Run from the discovery repository after custody approval:

```sh
cargo test --manifest-path docs/roadmap/0.1/0.1.4/issue87-analysis/roles/Cargo.toml
cargo build --release --manifest-path docs/roadmap/0.1/0.1.4/issue87-analysis/roles/Cargo.toml
# Invoke the resulting issue87-roles with DATABASE and NEW_ANALYSIS_DIRECTORY.
# Then roles-report.py NEW_ANALYSIS_DIRECTORY ORIGINAL_RUN_DIRECTORY.
# Then roles-supplement.py NEW_ANALYSIS_DIRECTORY ORIGINAL_RUN_DIRECTORY.
```

The crate imports the exact existing pack wire parser/decompressor via `#[path]`
and depends on existing canonical, role, reference and ObjectId implementations.
No alternative storage implementation is maintained here. Do not run recursive
`cargo fmt`: formatter traversal can reach the included product source. The module
is marked `rustfmt::skip` to prevent that traversal. Product encoding
functions compile as part of the included module but are never called. The
analysis crate's lockfile identifies its reporter dependencies, not producer
identity. Seal and review the source decoder files as well as the reporter.

A single streaming pack pass validates header/version/reserved fields, contiguous
coverage, directory bounds, group codecs/checksums, complete decoded-record
coverage, identity and exact roles. It reconstructs DELTAs only against selected,
authenticated FULL bases and validates every COPY range. A 32 MiB FIFO group cache
bounds repeated base fetch/decode; the primary pass visits each pack once. Payloads
are discarded after use. The separate spillable SQLite analysis index contains
only object identities, sizes, locations, roles, graph edges and physical totals.
It is an analysis artifact, not a product index or Store replacement.

`roles-report.py` computes a chronological union of Init and all 157 receipt roots
using the disk-backed visited set. Previously visited subgraphs need not recur.
All durable layers, commits and workspace stages are included, as are Branch
head/base pointers through their retained commit/layer rows. Logical membership
and required physical FULL bases are separate. A base's own references are not
traversed merely because it is needed to reconstruct a DELTA. `uses` tracks
file-content and metadata-value paths separately from exclusive role counts.

Every large object inventory is emitted once. Supplemental physical/root tables
are small review additions. CSVs state snapshot, status and provenance; byte
columns are integer bytes, IDs/locators are identities, and named counts are
integer counts. Empty first-admission cells mean unknown with the adjacent
reason, never zero. Encoded group bytes have no exact per-record attribution.

The test checks outer chunk framing protects user content that resembles a
structural marker and checks malformed pack headers fail. The actual census adds
exhaustive conservation/authentication checks and reconciliation against all
selected index rows; reports assert the sealed run's canonical totals. Reusing
this reporter on another run requires changing those explicit expected totals.

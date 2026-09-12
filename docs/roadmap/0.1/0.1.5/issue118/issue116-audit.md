# #116 Phase 1: capability and resource-limit audit

Status: source inventory complete at `f8fa59fab`; bounded reproductions under
`benchmark-results/host-store/issue118/20260912/issue116-audit/`. This audit is
published **before** any restriction removal, as #116 requires. Nothing in this
document is a claim of unlimited capacity.

Scope of the audit: the restrictions listed in #116, re-read against the final
source after all optimization tracks. Values are source constants with their
enforcement sites; every number is a *charge* or a *count guard*, not an
additive whole-process memory claim. Host and daemon are separate resource
scopes.

## 1. Inventory with exact enforcement sites

### 1.1 Pending edits and workspace state

| Restriction | Value | Unit | Enforcement site | Effect at the boundary |
|---|---:|---|---|---|
| Pending edit count | 4,096 | counted edits per edited file | `crates/layerfs-workspace-core/src/file_edit.rs:5` `MAX_EDITS_PER_FILE`; `next_edit` (`:368`) and `extend_splices` (`:253-257`); wire re-validation `crates/layerfs-fuse/src/live_wire.rs:402`; owner re-validation `crates/layerfs-fuse/src/live_owner.rs:2919`; backing re-validation `crates/layerfs-workspace/src/live_backing.rs:1970` | Next counted edit is rejected with `Error::InvalidInput("workspace edit limit")`; live state is unchanged (the failure is before `apply_edit`) |
| Piece count | 8,193 | pieces per edited file | `file_edit.rs:6` `MAX_PIECES_PER_FILE`, enforced in `PieceTree::replace` (`:681`); wire `live_wire.rs:406` `count(5)` | `Error::InvalidInput("workspace piece limit")`; the edit is rejected, previous contents retained |
| Piece-allocation charge | 2 MiB | aggregate across the pending workspace | `file_edit.rs:9` `MAX_PIECE_ALLOCATION`; `LiveWorkspace::check_piece_resources` (`:343-365`); `check_logical_allocation_charge` (`:375`) | `Error::InvalidInput("workspace piece allocation limit")` |
| Inline replacement | 1 MiB | per edit | `file_edit.rs:7` `MAX_INLINE_PER_EDIT`, `extend_splices` (`:259-261`) | `Error::InvalidInput("workspace inline edit limit")` |
| Inline retained data | 8 MiB | per workspace | `file_edit.rs:8` `MAX_INLINE_PER_WORKSPACE`, `check_piece_resources` (`:348-353`) | `Error::InvalidInput("workspace inline limit")` |
| Spool quota | 1 GiB (default) | per workspace policy | `crates/layerfs-workspace-core/src/limits.rs:12`; `ResourcePolicy::check`; `write_resources` (`file_edit.rs:337-339`) | `Error::InvalidInput("workspace spool limit")` |
| Final-delta memory | 8 MiB (default) | per workspace policy | `limits.rs:13`; `check_final_delta` | `Error::InvalidInput("workspace final-delta limit")` |
| Resulting length | 1 TiB | edited file | `file_edit.rs:10`; `PieceTree::replace` (`:627`, `:683`) | `Error::InvalidInput("workspace piece limit")` |
| Logical zero content | 1 GiB | edited file | `file_edit.rs:11`; `PieceTree::replace` (`:684`) | `Error::InvalidInput("workspace piece limit")` |
| Predicted zero extents | 131,072 | edited file | `file_edit.rs:12`; `PieceTree::replace` (`:680`, `:685`) | `Error::InvalidInput("workspace piece limit")` |

The edit counter is **per edited file** and resets only when the pending
representation is retired: an explicit truncate to zero
(`file_edit.rs:172`), an empty whole-file replacement
(`:277-281`), node deletion (`crates/layerfs-workspace-core/src/namespace.rs:258`),
Commit checkpoint conversion (`crates/layerfs-workspace-core/src/checkpoint.rs:67-91`)
and `Workspace::discard` (`crates/layerfs-workspace/src/file_io.rs:598`).
Capture (`fsync`) alone does not reset it. This is therefore a *cumulative
pre-Commit* count, not a lifetime-history limit, and the SDK/FUSE wire, owner
and backing adapters re-validate the same value on their own boundaries so a
malformed peer cannot present a larger count.

### 1.2 Capture and live transport

| Restriction | Value | Unit | Enforcement site |
|---|---:|---|---|
| Changed-fact memory charge | 96 MiB | one published/received changed-state set | `crates/layerfs-fuse/src/live_wire.rs:14` `MAX_FACT_MEMORY`; owner `live_owner.rs:2714` and `:2794`; backing `crates/layerfs-workspace/src/live_backing.rs:112` |
| Encoded node | 16 MiB | one live-wire node | `live_wire.rs:13` `MAX_NODE_BYTES`; `:252`, `:348`, `:358`; `live_backing.rs:568` |
| Live-wire frame | 1 MiB + 64 KiB | one message | `live_wire.rs:16` `MAX_FRAME` |
| Fact page | 64 KiB / 128 nodes | transfer batching | `live_wire.rs:11-12` `FACT_PAGE_BYTES`, `FACT_PAGE_NODES` |

The changed-fact charge is **not** the encoded byte count. For a local
publication it is, per node, `(node_encoded_bound - inline_bytes) * 8 + 1024`
(`live_owner.rs:2705-2717`); for a received fact it is
`encoded.len() * 8 + 1024` (`live_backing.rs:102-111`). The bound itself is
`4 + path.len()` per path, `49 * piece_count + inline_len` for an edited file,
`12 + name.len()` per directory change and `target.len()` for a symlink
(`live_wire.rs:227-246`). An exhaustion here surfaces as `PortError::NoSpace`
(owner) or `StoreError::InvalidInput("backing fact limit")` (backing), and the
same charge is also reserved from the shared 128 MiB live-state semaphore
(`crates/layerfs-fuse/src/live_runtime.rs:89`), so a 96 MiB charge can fail on
either bound.

### 1.3 Shared runtime admission and cache

`crates/layerfs-fuse/src/live_runtime.rs`: ordinary request permits 256 (`:11`),
ordinary transfer permits 32 MiB (`:12`), shared live-state permits 128 MiB
(`:89`), control requests/transfer 32 / 4 MiB, lifecycle requests/transfer
2 / 8 MiB, backing-work permits 2, kernel-work permit 1, runtime workers 2,
blocking workers 2. `crates/layerfs-fuse/src/immutable_read_cache.rs:8`: 32 MiB
evictable read cache. None of these is a total request, file or workspace
count; they bound simultaneous owners. Exhaustion waits or returns a
nonblocking admission failure on the entrypoint that requires it.

### 1.4 Storage, trees, paths and history

`crates/layerfs-content/src/limits.rs`: path bytes 4,096; component bytes 255;
path components 256; canonical object 16 MiB; object field 8 MiB; child
references 100,000; encoded string 4,096; decode nesting depth 8.
`crates/layerfs-content/src/tree/batch.rs`: sorted-tree update scratch 4 MiB
(`SORTED_TREE_UPDATE_SCRATCH_BYTES`), page items 234, tree page 8 KiB,
sibling batch 32 children. The Store object/page limits and metadata budgets
(128 objects per request, 4 MiB object page, 512 KiB/128-group metadata pool,
32 MiB decoded work per chain, 16 DELTA edges/128 KiB closure, 131,072-value
fingerprint window, 32-bit metadata ordinal space) are unchanged by this audit
and retain their own documented scopes.

## 2. Capability matrix

Separate claims, as #116 requires: an existing stored size, a large pending
mutation, and a wide single directory are different capabilities.

| Capability | Limit reached first | Evidence class |
|---|---|---|
| Large single file, repeated same-range overwrite | see §3 probe A | bounded actual-code probe |
| Large single file, append growth | spool quota (1 GiB) or piece count | source proof |
| Many changed files in one pending set | see §3 probe B | bounded actual-code probe |
| Wide single directory | see §3 probe C | bounded actual-code probe |
| Deep path | 256 components / 4,096 path bytes | source proof |
| Long history | no small fixed ceiling; ancestry traversal 1,000,000 steps; reopened reconstruction cost grows with history | source proof + prior public sequence evidence |

## 3. Bounded reproductions

All probes drive the **actual** public SDK route
(`Workspaces::edit_workspace_file_range` over `LayerStackStore` and a real
Workspace session) and report the exact rejection point and public error. They
are diagnostics for the audit, not public performance cases; public cases and
their proofs remain the qualification surface. Fixture source files are created
before the session opens and are never part of a measured edit. Raw output is
recorded in `benchmark-results/host-store/issue118/20260912/issue116-audit/`.

```bash
cargo +1.85.1 test -p layerfs-workspace --release --test issue116_capacity \
  -- --ignored --nocapture --test-threads=1
```

## 4. Interactions

- Two independent 2 MiB-class ceilings exist for *different* quantities: the
  per-workspace piece-allocation charge (`MAX_PIECE_ALLOCATION`) and the
  per-file piece count (`MAX_PIECES_PER_FILE`). Which one binds first depends on
  the structural width of `PieceNode` and on whether a file's pending state is a
  single contiguous spool range (charged one word) or a fragmented tree (charged
  one node per piece).
- Sufficient disk does not bypass the piece, edit-count, fact-memory or
  protocol ceilings. The only disk-backed escape is the spool quota itself.
- The changed-fact charge (96 MiB) and the shared live-state permits (128 MiB)
  are separate; a pending set can fail on either, and the failure is reported at
  the adapter that noticed it.
- A rejection is transactional: `prepare_*` returns before mutating live state,
  so a failed edit leaves contents, charges and revisions unchanged. `edit_many`
  and `extend_splices` retain the same property per batch member.

## 5. Bounded reproductions: measured results

Probe source: `crates/layerfs-workspace/tests/issue116_capacity.rs` (explicit
selection only, `#[ignore]` by default). Raw output:
`benchmark-results/host-store/issue118/20260912/issue116-audit/probe.log` and
`probe-fixed-a.log`.

| Probe | Workload | Before the repair | After the repair |
|---|---|---|---|
| A | one file, repeated 4-byte overwrite at offset 0 through `Workspaces::edit_workspace_file_range` | accepted 4,096, then `Storage(InvalidInput("workspace edit limit"))` | accepted 10,000 of 10,000, no rejection, exact final contents |
| B | one empty file, repeated 4 KiB append-shaped inline edits | accepted 2,048 (8,388,608 bytes), then `Storage(InvalidInput("workspace inline limit"))` | unchanged: this is the real 8 MiB pending-workspace inline budget, not an edit counter |
| C | 2,000 distinct pre-existing files, one 1-byte inline edit each, same pending workspace | per-edit cost was flat (63 s per 100 edits) and the run was stopped at 400 edits as unproductive; no piece/fact rejection observed | not re-run after the repair; no rejection is attributable to the edit counter here |
| F | per-edit cost against pending-set size, public route | 1 file: 20 edits in 17 ms; 200 files: 20 edits in 1,114 ms (≈55 ms/edit) | unchanged |

Probe A is the #116 primary case. Before the repair the rejection happened while
the edited file still held **one** pending piece and the workspace spool held
only the few kilobytes of acknowledged replacement data: no piece-count,
piece-allocation, inline, spool or frame budget was near its limit. The count
itself was the only cause. After the repair the same public route accepts
10,000 counted edits, and the final file contents equal the last replacement
(asserted, not inferred).

Probe B is **not** an edit-counter rejection and is retained as a real budget:
8 MiB of pending inline data per workspace. Raising it is not part of this
repair and no quota was enlarged.

Probe F measures a separate, pre-existing cost question: the public
`Workspaces::edit_workspace_file_range` route costs roughly 55 ms per edit when
the pending workspace holds 200 changed files versus well under 1 ms with one
file. Probe C at 2,000 files was flat at ≈630 ms per edit, so the cost grows
with the pending-set size but was not observed to be super-linear across
100–400 edits. This is a **latency** observation, not a correctness or capacity
limit: it is reported here for a prospective follow-up and is not used to claim
a #116 restriction. The historical ≈5,461 changed-file boundary remains a
route-specific observation with no located counter; the located per-workspace
bounds in §1.1 remain the enforceable ones.

## 6. Dispositions and the bounded repair

| Restriction | Disposition | Rationale |
|---|---|---|
| `MAX_EDITS_PER_FILE` (4,096) | **Removed** as a rejection criterion (commit `ead812e78`) | Redundant proxy: probe A shows rejection with a single piece and a few kilobytes of spool. The counter is now `u64` with checked arithmetic so removal cannot move rejection to integer overflow. Wire, owner and backing re-validation of the same bound removed; the `count == 0` batch guard is retained. |
| `MAX_PIECES_PER_FILE` (8,193) | Retained | Real per-file representation bound; `PieceTree::replace` rejects before any state change. |
| `MAX_PIECE_ALLOCATION` (2 MiB) | Retained | Real aggregate pending-workspace bound; measured in `size_of::<PieceNode>()` units. |
| `MAX_INLINE_PER_EDIT` / `MAX_INLINE_PER_WORKSPACE` | Retained | Real request and workspace bounds; probe B reaches the workspace one at 8 MiB. |
| Spool quota, final-delta memory | Retained | Real policy budgets; `discard`/checkpoint release them. |
| Result length, logical zero, predicted zero extents | Retained | Representation bounds for sparse/growing results. |
| Fact memory 96 MiB, node 16 MiB, frame 1 MiB+64 KiB | Retained | Protocol/publication bounds; a wide changed set must stream through pages rather than raise them. |
| Path bytes/components, object sizes, tree scratch, runtime permits, read cache | Retained | Independent API/format/compatibility or liveness bounds. |
| Ancestry traversal 1,000,000 steps | Retained, documented | Incomplete traversal must not be returned as proven non-membership. |

The repair is deliberately narrow: it removes one count proxy and keeps every
resource bound. No quota, worker count, cache or protocol limit was increased,
and no external library was patched.


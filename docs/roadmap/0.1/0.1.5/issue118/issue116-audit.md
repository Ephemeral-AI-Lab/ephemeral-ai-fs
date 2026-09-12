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

All probes run the **actual** workspace code (`LayerStackStore` +
`Workspace`) and report the exact rejection point, the error value, and the
live charges at that point. They are diagnostics for the audit, not public
performance cases; public cases and their proofs remain the qualification
surface. Command and raw output are recorded in
`benchmark-results/host-store/issue118/20260912/issue116-audit/`.

```bash
cargo +1.85.1 test -p layerfs-workspace --release --lib issue116_phase1_probe -- --ignored --nocapture
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

## 5. Recommendations

See §6 for the verified observations, dispositions and the bounded repairs that
follow from them. No restriction is removed merely because a counter exists;
each removal below replaces the count proxy with the resource quantity that
actually bounds the representation.

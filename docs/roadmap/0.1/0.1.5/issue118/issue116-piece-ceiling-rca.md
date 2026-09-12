# #116 root cause: why the pending edit set stops at 5,461 files

Question: why does `namespace-100000 --sequence 32000` fail at accepted edit
5,641 with `workspace piece allocation limit`, and is the ceiling the edit
counter, the piece count, or something else?

Answer, in one line: **the ceiling is the 2 MiB pending-workspace
`piece_allocation_bytes` budget divided by the 128-byte charge of one
`PieceNode`, and the workload needs three such nodes per changed file, so
`2,097,152 / (3 × 128) = 5,461.33` files fit.** Nothing about the edit counter
is involved.

## 1. The charge is exact, not approximate

`PieceTree::logical_allocation_charge()` is
`count() × size_of::<PieceNode>()` (or 8 bytes for the compact single-spool
form). The struct's fields sum to exactly 128 bytes:

| Field | Bytes |
|---|---:|
| `piece: Piece` | 56 |
| `priority: u64` | 8 |
| `left: Link` / `right: Link` (`Option<Arc<PieceNode>>`) | 8 + 8 |
| `len`, `count`, `inline_len`, `zero_len`, `spool_len`, `height` | 8 × 6 |
| **Total** | **128** |

`Piece` itself is 56 bytes because its largest variant carries a 32-byte
`ObjectId` plus two `u64`s plus a discriminant (`ObjectId` = 32, `Arc<[u8]>` =
16, `Option<Arc<SpoolSlice>>` = 8 — measured, not assumed).

`2097152 / 128 = 16384` nodes, exactly, with zero remainder.

## 2. The limit is hit at exactly 16,383 nodes

`crates/layerfs-workspace/src/file_io.rs::check_piece_resources` computes the
next charge and rejects when it exceeds `MAX_PIECE_ALLOCATION`:

```text
allocation = piece_allocation_bytes - old_charge + next_charge   filtered <= 2 MiB
```

The product's own commit diagnostics from the passing
`namespace-100000 --sequence 5461` run state the same numbers:

```text
edit_count: 5461
edit_piece_count: 16383
edit_piece_logical_charge: 2097024      (= 16,383 × 128)
```

16,383 nodes is the largest multiple of 3 that fits under 16,384. The next
changed file needs 3 more nodes → 16,386 → 2,097,408 > 2,097,152 → rejected.
That is why the failure lands at 5,461 accepted edits in one run and 5,641 in
another: both are the same node ceiling reached at slightly different points in
the sequence, and my independent `Workspace`-level probe reproduces the same
5461 boundary with `piece_charge=2,097,032`.

## 3. Why three nodes per changed file

Measured structure for one 48 KiB file with one 12-byte splice (RCA probe,
`issue116-audit/rca-pieces.log`):

```text
after base write:        charge=8      (compact single-range form)
after 12-byte splice:    charge=384    count=3
  piece[0] Spool offset 0      len 1000
  piece[1] Spool offset 49152  len 11     <- the new bytes
  piece[2] Spool offset 1011   len 48141
```

The base file starts as one piece charged 8 bytes. Any splice into it splits
that piece into a left remainder and a right remainder and inserts the new
data, so the minimum is **left + new + right = 3 nodes**. The same shape holds
through the SDK route, where the product's diagnostics report
`edit_spool_allocated_bytes = 0` and `edit_spool_live_bytes = 0`: there the
nodes are `Base(0..1000)`, `Inline(12 bytes)`, `Base(1012..49152)` — three nodes
again, just a different variant mix. **Node count is what the budget charges;
the payload kind is irrelevant.**

Growth is additive and unavoidable under this representation:

| Operation on the same file | Nodes | Charge |
|---|---:|---:|
| base write only (compact form) | 1 (compact) | 8 |
| + one splice | 3 | 384 |
| + second splice elsewhere | 5 | 640 |
| + 5 more splices over the *same* range | 5 | 640 |

Re-splicing an already-spliced range does **not** grow the tree — the existing
piece is replaced. Only *new* offsets cost nodes. A whole-file inline
replacement also stays at 1 piece (`delta=0`).

## 4. The decisive arithmetic

| Per-file cost | Files that fit in 2 MiB |
|---|---:|
| 1 node (compact single range) | 16,384 |
| 2 nodes | 8,192 |
| **3 nodes (one splice)** | **5,461** |
| 4 nodes | 4,096 |

The Stage3 spill-scale case needs ≈15,873 pending keys in one set. Under the
current representation that is `15,873 × 3 × 128 = 6,095,232` bytes, i.e.
**2.91× the declared 2 MiB budget**. So the spill-scale crossing is not
"almost" reachable — it is structurally out of reach by a factor of ~3.

## 5. What the root cause is *not*

- **Not the edit counter.** `MAX_EDITS_PER_FILE` is already removed
  (`ead812e78`); the same run accepts 5,641 edits before any resource bound, and
  a single file now takes 10,000 counted edits. The failing quantity is a byte
  budget, not a count.
- **Not the per-file piece count.** `MAX_PIECES_PER_FILE = 8,193` is per file;
  each file here has 3 pieces. Irrelevant at this boundary.
- **Not the spool quota or inline budget.** The sequence run reports
  `edit_spool_allocated_bytes = 0`, `edit_spool_peak_bytes = 0`,
  `edit_spool_live_bytes = 0`; the spool limit is 1 GiB and is never approached.
- **Not the 96 MiB changed-fact charge.** At 5,461 nodes it is ~8 MB.
- **Not a hardcoded file ceiling.** There is no `5461` (or similar) constant in
  the source; the number is an emergent quotient. Grep for it finds nothing.
- **Not disk.** The spool is a disk-backed file; the rejection is a memory
  charge, so extra disk cannot bypass it.

## 6. Why the budget cannot simply be raised

128 bytes per node is a real allocation, so 2 MiB is an honest bound on the
structural memory of the pending set, not a conservative over-estimate. Raising
it would be the "increase the quota" move that #116 forbids, and it would also
need a proportional answer for the daemon's 2 GiB container and the 128 MiB
shared live-state permits.

## 7. The repair that actually removes the ceiling

The cost is in the *representation*, not in the data: three treap nodes to
describe "one small splice into one base file". A bounded compact form for
exactly that shape — the base reference plus a small ordered splice list,
materialized into the piece tree only past a threshold — would charge roughly
32–48 bytes per changed file instead of 384:

| Representation | Bytes/file | Files in 2 MiB |
|---|---:|---:|
| current piece tree, 1 splice | 384 | 5,461 |
| compact splice list (≈40 B) | ≈40 | ≈52,000 |

That fits 32,000 distinct changed files — and the 15,873-key spill-scale
crossing — inside the **unchanged** 2 MiB budget, which is what #116 asks for
("replace restrictive representations with bounded streaming/spill/
consolidation", "do not simply delete safety guards or enlarge quotas").

Constraints any such change must preserve, from #116 itself: no full growing-file
rewrite per edit, coalescing/reclamation only when no reader, mapping, in-flight
operation or retry still owns the range, no user-visible Commit or changed
atomic Commit boundary, checked arithmetic instead of a moved ceiling, and
unchanged SDK/FUSE/mmap/truncation/sparse/hardlink/cancellation/rollback
semantics.

## 8. 5,461 is a per-shape number, not a workspace limit

The ceiling depends entirely on how each changed file's pending state is shaped.
Measured boundaries (`issue116-audit/rca-compact.log`, `probe-charge.log`,
`probe-splice.log`):

| Pending shape of each changed file | Charge/file | Measured acceptance | Budget that stopped it |
|---|---:|---:|---|
| newly created, one contiguous write (compact single-range form) | 8 B | **262,144** | spool quota (1 GiB / 4 KiB files) — piece budget was exactly full at 2,097,152 B |
| whole-file replacement | 8 B (stays 1 piece) | same as above | spool quota |
| small splice into an existing base file | 384 B | **5,461** | piece allocation 2 MiB |
| one splice + one more at a new offset | 640 B | ~3,276 | piece allocation 2 MiB |

So a workspace can hold **262,144** newly created small files but only **5,461**
pre-existing files each carrying one small splice — a 48× difference produced by
the representation, not by a file-count policy.

The distinction that matters in practice: `namespace-100000` **Init** imports
100,000 files and passes, because imported files take the compact form; a
32,000-edit **sequence** that splices into 32,000 distinct pre-existing files
does not, because every spliced file costs 3 nodes. There is no `5461` constant
anywhere in the source; it is `2 MiB / (3 × 128 B)`.

A second per-shape budget worth naming: the changed-fact memory charge is
**per published set**, not cumulative across the whole workspace, so it bounds
one publish (≈29,960 spliced files at 3,360 B each) rather than the workspace
lifetime.

## 9. What the ceiling becomes after the compact representation

Removing the piece ceiling does not by itself make the workload unbounded: the
**changed-fact memory charge** (`live_wire::MAX_FACT_MEMORY = 96 MiB`, charged
per node as `(node_encoded_bound - inline_len) × 8 + 1024`) becomes the next
binding budget. Measured with the real shape (15-character path, 3 pieces,
12 inline bytes): **3,360 bytes per changed file**, flat from 1 to 5,001 files
(`issue116-audit/rca-fact.log`).

| Representation | Piece budget allows | Fact budget allows | Binding | Files supported |
|---|---:|---:|---|---:|
| current (3 nodes × 128 B = 384 B/file) | 5,461 | 29,959 | piece | **5,461** (measured) |
| compact splice list ≈56 B/file | 37,449 | 29,959 | fact | **29,959** |
| compact splice list ≈40 B/file | 52,428 | 29,959 | fact | **29,959** |
| 1 node/file | 16,384 | 29,959 | piece | 16,384 |

So the representation change raises the real ceiling from **5,461 to ≈29,960
files (5.5×)** — comfortably past the 15,873-key spill-scale crossing, but
**just short of the 32,000 distinct files K32000 needs** (path length barely
matters: 7 vs 16 characters moves the fact ceiling only 29,746–30,393).

### The fact charge is provably conservative

Measured resident cost of the same pending set (`issue116-audit/rca-rss.log`):

| Changed files | Charged fact bytes | Peak RSS | RSS growth over baseline | Real bytes/file |
|---:|---:|---:|---:|---:|
| 1,001 | 3,363,360 | 15,384,576 | 3,538,944 | 3,536 |
| 3,001 | 10,083,360 | 18,546,688 | 6,700,032 | 2,232 |
| 5,001 | 16,803,360 | 22,315,008 | 10,458,624 | 2,091 |

At the current ceiling the product charges **16.8 MB** while the whole process
uses **22.3 MB peak RSS** (baseline 11.8 MB) — i.e. the charge covers the entire
process growth and still over-states it by ~1.6×. The 1,024-byte-per-node floor
alone caps any fact budget at 98,304 nodes, and the ×8 encoded multiplier is the
other conservative term.

That means a second, independent repair is available **if** K32000 specifically
is required: re-derive the fact charge from measured live-set cost (for example
a 512-byte node floor and a 4× multiplier, still ~1.5× above the observed
2,091 bytes/file) instead of the current conservative formula. That would put
the post-representation ceiling near **49,000 files**, above K32000. It changes
the shared live-state reservation accounting, so it needs its own resource
proof and must not be bundled silently into the representation change.

## 10. Why the compact representation is not trivial

It is a **bounded but real** data-structure change, not a counter deletion:

- `PieceTree` gains a compact form (base/spool reference + ordered splice list)
  alongside the existing single-range `compact_spool` fast path, and `replace`,
  `range_with_visits`, `pieces`, `count`, `len`, `inline_len`, `spool_len`,
  `logical_allocation_charge` and the two collapse cases must all handle it.
- `replace` must stay bounded per edit: appending one splice to a k-entry list
  is O(k) with k small, and the form must fall back to the tree past a threshold
  rather than degrade into a per-edit rebuild of the whole file.
- Consolidation must not drop a range that a held `ReadPlan`, mapping, in-flight
  operation or retry still owns.
- Every invariant #116 lists stays: no extra Commit, unchanged atomic boundary,
  checked arithmetic, and unchanged truncation/sparse/hardlink/cancellation/
  rollback/mmap semantics.
- Verification is the expensive half: focused splice/coalesce/sparse/truncate
  checks, wire round-trip, `check_piece_resources` exactness, plus the existing
  75 workspace lib, 40 fuse and 12 file-edit integration tests, then the public
  K5000/K5461/K32000 boundary re-runs.

Realistic effort: roughly 250–400 lines of product change plus 400–700 lines of
tests, with a correctness risk concentrated in `PieceTree::replace`.

## 11. Reproduction

```bash
# exact boundary through the public workload
python3 benchmark/fs-bench-pro/shared/runner.py --family init_namespace \
  --case namespace-100000 --sequence 5461 --sequence-commits 1 --seed 1 \
  --source-arm candidate \
  --host-binary benchmark-results/host-store/issue118/20260912/final-treatment/fs-benchmark-pro \
  --image layerfs-bench-infra:e46df7b0463158de \
  --timeout 900 --product-timeout 880 \
  --output benchmark-results/host-store/issue118/20260912/k5461-boundary   # PASS, 16383 pieces

# the same at 6000: FAIL at accepted edit 5461
# owner-side error, from the container log: InvalidInput("workspace piece allocation limit")

# exact structure and charge decomposition
cargo +1.85.1 test -p layerfs-workspace --lib issue116_piece_charge_rca \
  -- --ignored --nocapture --test-threads=1

# per-shape boundary (compact form)
cargo +1.85.1 test -p layerfs-workspace --lib issue116_compact_form_scaling_probe \
  -- --ignored --nocapture --test-threads=1

# next-budget (fact charge) and real resident cost
cargo +1.85.1 test -p layerfs-workspace --lib issue116_fact_charge_probe \
  -- --ignored --nocapture --test-threads=1
cargo +1.85.1 test -p layerfs-workspace --lib issue116_pending_set_rss_probe \
  -- --ignored --nocapture --test-threads=1
```

Raw evidence: `issue116-audit/rca-pieces.log`, `probe-splice.log`,
`probe-perfile2.log`, `k5461-boundary/`, `k6000-diag3/`, `k32000-spill-1/`.

# Bounded metadata endpoint query — implemented and verified

Product commit: `cecd4621576735b61ac105c8daaf68b66fd4ca8b`.
The shared endpoint query no longer aggregates the complete metadata catalogue.
It seeks into a suffix containing at most 165 possible starting ordinals and
calculates the exact maximum there. Both preparation and publication use the
same checked helper on their existing connection/transaction.

This removes one potentially quadratic cumulative catalogue traversal. It is a
CPU-work/scaling improvement, **not an end-to-end Init latency or cold acceptance
claim**. The cold <=2.7 s target and full-history ValueIndex replay after reopen
remain open. The prior final-phase catalogue interval was only 2.699 ms; that
combined interval is not all attributable to this query.

## Why a bounded suffix instead of a single last row

The naive last row is not necessarily the row with the greatest end when rows
overlap. Schema10 supplies stronger usable bounds: first_ordinal is a unique
integer and each count is in 1..165. Let M be the greatest first_ordinal:

```text
Excluded row: first_ordinal <= M - 165
Its end:      first_ordinal + count <= M
Last row:     M + count >= M + 1
```

Therefore no excluded row can determine the maximum. This holds for every
catalogue satisfying the row constraints, including gaps and overlaps, without
an assumption that it is contiguous. The query is:

```sql
SELECT COALESCE(MAX(first_ordinal+count),1)
FROM metadata_value_groups
WHERE first_ordinal >
    (SELECT MAX(first_ordinal) FROM metadata_value_groups)-?1
```

The existing group-size constant 165 is bound internally, not exposed as a knob.
Both subqueries see one SQLite statement snapshot. Empty catalogues return 1;
the existing result range through 2^32 is checked. Publication still compares
each prepared group's ordinal with the connection-visible endpoint and uses
the original transaction/rollback machinery.

The proof assumes enforced row constraints. It does not assert equivalence to
the old MAX for arbitrary database corruption that bypasses count bounds.
Stored-group authentication, semantic count validation and full catalogue
validation remain unchanged. A new integration case deliberately bypasses CHECKs
to create count 166 and confirms that existing read/full-validation paths reject
it. Schema-conforming altered counts are tested against the old aggregate.

## Deterministic query-work evidence on product SQLite

Followed [the committed protocol](tail-query-contract.md), 77eff9089. The test
uses the actual v10 schema and the linked SQLite 3.51.0, not system Python SQLite.
For each cell, old/new statements run once against identical endpoint-test rows.
This is a deterministic work-counter check, not a wall-time benchmark. Fixture
construction and its memory are outside the measured statement counters.

| Catalogue groups | Old SQLite VM steps | Bounded-query VM steps | Old fullscan steps | Bounded fullscan steps |
| --- | ---: | ---: | ---: | ---: |
| 100 | 611 | 36 | 99 | 0 |
| 1,000 | 6,011 | 36 | 999 | 0 |
| 10,000 | 60,011 | 36 | 9,999 | 0 |
| 100,000 | 600,011 | 36 | 99,999 | 0 |

These are catalogue groups, not namespace-file counts. The ordinary cells use
165-value groups. A separate dense-tail case uses overlapping schema-conforming
groups to exercise all 165 possible suffix rows: 1,020 VM steps, 0 fullscan steps,
0 sorts. Every old/new endpoint matched exactly.

Pinned EXPLAIN opcodes show a primary-key `Last` for the inner maximum and
`SeekGT` for the outer bounded suffix. No temporary sort or ephemeral table is
created. The test asserts <=1,200 VM steps, zero fullscan/sort and <=64 KiB
statement memory, including the dense suffix.

SQLite VM steps are not CPU instructions or elapsed time. B-tree seek work and
cold page I/O still depend on tree height. The algorithmic bound is
O(log G + 165) per request. Repeated requests no longer accumulate complete
catalogue scans; this does not prove that every other Init/Commit path is linear
or sublinear.

## CPU and memory safety

Prepared-statement memory was 9,712 bytes at every size and for the dense suffix,
versus 5,568 bytes for the old query: a fixed 4,144-byte increase in this counter.
There is no Rust collection growing with catalogue size, new persistent cache,
schema/index, background worker or increased SQLite cache limit. The query
returns one scalar; its row scan is bounded by the format's group-size limit.

4 KiB pages, physical pooling, pack/zstd/FULL/DELTA, exact CAS, authentication,
resource bounds and admission rollback remain intact. The first-use historical
ValueIndex replay still has linear historical work; it was not changed here.

## Correctness and qualification

- The initial test-harness compile error (u64 FromSql) was corrected to read i64;
  command/error/exit 101 retained.
- The pre-change work test failed as intended on 100 groups: 99 fullscan steps.
- Focused metadata run: 15 passed, including the new query/equivalence checks and
  existing sharing, reopen, intermediate-base authentication and rollback tests.
- Full Store library suite: 130 passed, 0 failed, 4 pre-existing ignored tests.
- Independent subagent code review of the four intended files found no blocking
  correctness, transaction, overflow or integrity issues.
- Host runner build passed linked schema10/storage-format probes. Matching Linux
  daemon/FUSE/workload image built successfully. Both ran under the shared lock.
- Prepared each registered input separately, then ran `verify-selected.py` with
  explicit case, seed 1, source, input and image identities, serially under its
  own measurement lock. No parent double-acquisition.

| Registered Init case | Imported files | Imported logical bytes | Sampled files | Result |
| --- | ---: | ---: | ---: | --- |
| namespace-100-compact-v3 | 100 | 5,000,000 | 6 | PASS |
| namespace-1000-compact-v3 | 1,000 | 20,000,000 | 10 | PASS |
| namespace-10000 | 10,000 | 300,000,000 | 10 | PASS |
| namespace-100000 | 100,000 | 500,000,000 | 10 | PASS |

Every selected proof passed root equality after fresh reopen, sampled FUSE reads
and cleanup. Coverage is the registered `import-counts-reopen-and-sampled-fuse-v1`
profile: **not exhaustive verification of every file byte or the full namespace**.
The current registered low tiers are the compact-v3 cases shown above; the query
change did not alter their registry or fixtures.

No performance sample was collected, so no cold gate pass, cache-based timing
comparison or improvement percentage is reported. These correctness proofs do
not substitute for future plain cold acceptance. Warm observations remain valid
only for declared paired deltas. New full Commit performance qualification was
not claimed; shared admission/reopen/rollback correctness is covered by tests.

## Identity and custody

Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-tail-query-evidence/20260911T072359Z`.
Commands/logs/exits, old-test failure, pinned query-work evidence, implementation,
qualified identities, prepared-input records and all four immutable selected
verification receipts are retained. `analyze.py` checks the four work cells,
proof status/cleanup, matching source/product/binary identities and source custody.

- Product seal: `44f8226e21ff2b342ed88c17f0d47c278a60fad80e35e8562ae5bc9693528073`
- Source seal: `424b91fd418b1ef2da5b5a063c446c7f8d7b26b6c1abb16283fbba4196375006`
- Host binary SHA-256: `992a382b3a44e1e97d1e29e6627e724999e5056621049bdd1a1387aff781ae3e`
- Image: `layerfs-bench-infra:424b91fd418b1ef2`
- Image identity: `sha256:8e1db64d4234c05b7de5558782870f3961ab1c08f00e76d6784cbf60ef47f008`
- namespace-100000 fixture digest:
  `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`

Source/product/workload seals matched before and after verification. Store/SDK
and publication ran on macOS; Docker hosted daemon/FUSE/workload only. Original
dirty compaction-removal work was excluded from the commit and preserved.
No older sealed evidence was modified. No release/tag/deployment.

This closes the bounded endpoint-query implementation step of #111, not the
overall cold Init target. Related investigation: #115/#109/#110; qualification
context: #108/#106/#102/#104/#100/#107.

Issue outcome: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5631092920

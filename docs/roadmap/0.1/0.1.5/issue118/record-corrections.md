# Issue #118 acceptance-record corrections and bounded applicability assessment

Status: bounded correction pass over the acceptance record, executed before the
final qualification. It corrects the provenance, the pending-representation
threshold and the probe scope, and assesses whether older cohorts still apply to
the current ordinary path. It is not a provenance campaign: every statement below
is tied to a committed receipt or a reproducible command.

## 1. Producer cohort of the full157 and historical-access receipts

`full157-candidate-1/identity.json` records, for the production arm:

| Field | Value |
|---|---|
| `binary_sha256` | `408f3e2a762e059a5efc8f3634355ad259de16a95624427610e8f94fb7b7fc64` |
| `LAYERFS_SOURCE_COMMIT` | `226cfeea9617a6d7187600a98247887af4cd8c0e` (`LAYERFS_SOURCE_DIRTY=true`) |
| `LAYERFS_SOURCE_SEAL` | `0574853ae6cdb50c9be79316417146cb10e72d4fd08ddb9dab8038316a9badfb` |
| `LAYERFS_PRODUCT_SEAL` | `6e9e2840d7478c35e4370ab38c009914dd7ec43921aab4322624840c9e822768` |
| image | `sha256:08082f1af0927d64af9fa525e7dee041f02bdee0d8e84baf4cf6ecbfcd22660d` |

An automated read of every JSON receipt under the evidence root confirms the
cohort assignment:

| Evidence | Producer binary |
|---|---|
| `full157-candidate-1/**` (45 receipts) | `408f3e2a…` |
| `full157-candidate-1/access-performance/**` (22 receipts) | `408f3e2a…` |
| `full157-candidate-1/access-verification/**` (22 receipts) | `408f3e2a…` |
| `remaining-shared/**` (18 receipts) | `440ae2c4…` (fsync-qualified) |
| `affected-rerun/**` (6 receipts) | `6693224e…` (three SDK-edit selections) and `440ae2c4…` (control) |

**Correction.** `terminal-outcome.md` previously attributed rows 10, 11, 15 and 16
(the ordinary full157 performance, its same-Store verification and the 11+11
historical-access cases) to `final-treatment`, binary `6693224e…`. That attribution
is wrong: those receipts name `408f3e2a…`. The rows are relabelled to the archived
`408f3e2a…` producer with source seal `0574853a…`, product seal `6e9e2840…` and
image `08082f1a…`.

**Removed claim.** The same document asserted that the product executable is
"byte-identical across `fsync-qualified` and `final-treatment`". Different
executable hashes are not byte-identical, and a reused output path proves nothing
about which bytes ran earlier. The statement is deleted; every row names the exact
binary hash of the run that produced it.

## 2. Bounded applicability assessment

What changed between the `408f3e2a…` producer (`226cfeea9` + dirty tree) and the
current tree, by `git show --stat` over `226cfeea9..HEAD`:

| Commit | Product source? | Runtime effect on the ordinary path |
|---|---|---|
| `679212e45`, `58dbe4cde`, `9f8494b70`, `bde3d4e43`, `26c95c088` | no (docs, plus `#[cfg(test)]` probes in `file_io.rs` tests) | none |
| `aa96e7d53` | no (census script only) | none |
| `2dbc75ecb` | 3 lines in `live_owner.rs` | `eprintln!` gated on `LAYERFS_EDIT_FAILURE_DIAGNOSTIC`; no behaviour change when unset |
| `ead812e78` | yes, but **before** `226cfeea9` | already contained in the `408f` producer |
| current uncommitted work | `layerfs-layerstack-store` (pack coalescing), `layerfs-workspace-core` + `layerfs-fuse` (bounded pending form) | **changes the ordinary write path** |

So the recorded `408f`/`440ae2c4`/`6693224e` storage and latency results were produced
by product source that differs from the current tree only by an env-gated diagnostic
print and documentation. They therefore remain **applicable to the pre-change
ordinary path** and are reused as the recorded baseline; they are not applicable to
the current treatment, and the affected rows are re-run rather than inherited
(`issue107-coalesce-full157`, `issue116-compact-k32000`).

## 3. Build-cache integrity finding (recorded, not hidden)

While rebuilding, the shared incremental target
`benchmark-results/host-store/builds/incremental-1.85.1-release` was found to hold a
`layerfs-content` artifact compiled at 13:12 whose recorded source mtime predates
the fingerprint, so Cargo reused it even though the current sources call
`layerfs-content`'s bounded batch-read entry point:

```text
$ strings .../deps/liblayerfs_content-56e089ebed25bf10.rlib | grep -c get_authenticated_canonical_batch
0                      # before the repair (fingerprint 13:12-13:13)
$ touch $(find crates/layerfs-content -name '*.rs')
$ python3 benchmark/fs-bench-pro/shared/runner.py --build-host
1                      # after the repair (all nine workspace crates recompiled)
```

The observed failure was a hard compile error (`method
get_authenticated_canonical_batch is not a member of trait ObjectRead`) when the
dependent crate was next recompiled. The repair is a cache/mtime refresh only; no
source changed. Consequence for applicability: the recorded binary hashes remain
the identity of what ran, but their recorded `LAYERFS_SOURCE_SEAL` alone does not
prove which `layerfs-content` revision they linked. All final qualification in this
issue is therefore re-run on a freshly built, single-seal executable so that the
binary, source seal and measured behaviour agree. The finding is retained in
`execution.md`.

## 4. Default frontier capacity

The default pending-frontier capacity is **B = 15,873** keys, computed at the
`FrontierInodes` construction in `crates/layerfs-workspace/src/changes.rs` from the
default `ResourcePolicy::max_final_delta_memory_bytes` (8 MiB):

```text
io_bytes        = journal_io_bytes(8 MiB)                    = 65,536
frontier_budget = 8,388,608 - 4 * (65,536 - 256)             = 8,127,488
B               = 1 + (8,127,488 - 1,024) / 512              = 15,873
```

The tree-work batch size 128 is a *different* quantity (it bounds one sorted-tree
update batch) and is not this threshold. Corrections:

- `issue116-audit.md` §7 previously described `batch_size = (8 MiB / 4096).clamp(1, 128) = 128`
  pending frontier keys and concluded that K5000/K5461 "exercise the spill path".
  That is wrong and is corrected there: the frontier capacity is 15,873 keys, and a
  5,000- or 5,461-key pending set stays **below** it.
- The required spill-scale evidence is therefore: the registered default-policy
  proof at exactly 2B and beyond, plus the public 32,000-edit route whose pending
  set (32,000 keys) necessarily crosses B twice (2 × 15,873 = 31,746) and enters a
  third partial run.
- Measured default-policy proof (`production_budget_spill_boundaries_stay_bounded`,
  complete command 2.82 s, log `issue116-compact-k32000/default-budget-frontier-proof.log`):

  ```text
  batch=15873 count=31746 flushes=2 new_writes=63492 new_reads=31746 merges=1 deepest_level=1 peak_runs=3
  batch=15873 count=63492 flushes=4 new_writes=190476 new_reads=126984 merges=3 deepest_level=2 peak_runs=4
  batch=15873 count=126984 flushes=8 new_writes=507936 new_reads=380952 merges=7 deepest_level=3 peak_runs=5
  ```

## 5. 10,000-edit probe scope

`crates/layerfs-workspace/tests/issue116_capacity.rs` probe A asserts only that
10,000 counted edits are **accepted** through the public route, then drops the
workspace; it never asserted final bytes or Commit/reopen. That limitation is now
stated in the audit, and the claim is carried by proofs that do assert it:

- the extended focused proof in `crates/layerfs-workspace/tests/issue116_capacity.rs`
  (10,000 counted edits, exact final contents, Commit/checkpoint, reopen and byte
  comparison) — see §7 of the final outcome;
- the public 32,000-edit `namespace-100000 --sequence 32000` route with its
  independent verification (`changed_files_checked: 32000`, exact bytes/lengths
  before and after a fresh reopen).

## 6. Qualification rows kept distinct

The 18 remaining shared selections are single-sample complete public samples with
matching independent proofs on `fsync-qualified`; they are **not** matched n3
comparative evidence and are not used as regression controls. Comparative screens
are frozen only for the paths actually changed by this issue (ordinary storage
construction and the pending-edit representation).

## 7. Preserved warnings

- Git allocation layout WARN: measured Git157 allocation `56,197,120 B` versus the
  recorded `56,373,248 B`, with unchanged apparent/pack bytes; both are reported and
  neither is normalized.
- Cold Init 2.7 s absolute target: owner-WAIVED, wall WARN +1.039% preserved.
- Stage2 K10 50 ms/31 ms: owner-WAIVED.
- Active-K100 paired Commit WARN +0.311 ms, cold-openat paired WARN, the
  `image-freshness-failure.json` rejection, the `reopen-k1000-screen` harness-only
  observer failure and every failed spill-ceiling attempt remain recorded.

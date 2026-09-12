# #120 applicability decision for `3e308a8f2` (rustfmt 1.96 + Clippy lint fixes)

Status: decided **before any campaign family is collected**, as the first task of
the #120 finalization campaign. Bounded scope: determine whether the previously
qualified evidence (ordinary full157, historical access 11 + 11, the
default-budget `namespace-100000 --sequence 32000` route, the K6000 boundary and
the default-budget frontier proof — all produced by the **final treatment**,
binary `b5f089ebcd6fa2fa939feb9bccc5300ca8ede798820fe8fd9aace8799fe4ec0b`) is
still applicable to the #120 candidate, given that `3e308a8f2` touched product
source after that treatment.

## 1. Exact change inventory between the final treatment and the #120 candidate

The final treatment's source is `345f75ad9` plus the then-uncommitted #107/#116
work (committed later as `076a621c8`). `git log 345f75ad9..1ff1f2ddd --name-only`
shows every product-adjacent file that changed since; classifying each commit:

| Commit | Product source? | Runtime effect |
|---|---|---|
| `076a621c8` | yes — but this **is** the final treatment's own #107/#116 work | already inside the qualified binary |
| `0e583c7a3` | `crates/layerfs-workspace/src/file_io.rs` only, entirely inside `mod tests` (`#[test] #[ignore]` probes) | none (test-only, verified by diff: 77 insertions, all `#[test]` probe bodies) |
| `3e308a8f2` | yes — `live_owner.rs`, `live_wire.rs`, `write_metrics.rs`, `objects.rs`, `admission.rs`, `batch.rs`, `file_edit.rs`, `changes.rs`, `file_io.rs` | reviewed below, fix by fix |
| `f12d4ff26`, `1593bdb25`, `afab1c465`, `c06cb5a80`, `bb51d5b7c`, `1ff1f2ddd` | no (tools, docs, web demo, handoff text) | none |

`3e308a8f2` also reflowed `benchmark/fs-bench-pro/src/infra.rs` (harness) —
formatting only (one iterator chain re-indent), no behavior.

## 2. The four named lint fixes, reviewed line by line

1. **`layerfs-content/src/tree/batch.rs` — `BatchChildren` type alias.**
   A `type BatchChildren = (usize, Vec<(ObjectId, Vec<u8>)>, Option<Lease>);`
   alias replaces the inline nested tuple in one return type. Rust type aliases
   are transparent: no symbol, codegen, or semantic change. **Identity.**
2. **`layerfs-fuse/src/live_owner.rs` — `as_deref_mut()` → `as_mut()` (4 sites).**
   The operand is `Option<&mut EditDiagnostic>`; `&mut T: DerefMut<Target = T>`
   derefs to itself, so `as_deref_mut()` was the identity. `as_mut()` yields
   `Option<&mut &mut EditDiagnostic>`, and every use site only mutates through
   the reference (`diagnostic.values[..] += …`), which auto-derefs identically.
   No control-flow, aliasing, or ordering change. **Identity.**
3. **`layerfs-workspace/src/changes.rs` — spill-merge re-raise as `?`.**
   Before: `if let Err(error) = self.write_spill_row(..) { #[cfg(test)]
   self.note_spill_allocation(..)?; return Err(error); }`.
   After: `let written = self.write_spill_row(..); #[cfg(test)] if
   written.is_err() { self.note_spill_allocation(..)?; } written?;`.
   In the release build the `#[cfg(test)]` block is absent in both forms, so the
   only difference is `return Err(e)` versus `?` — identical propagation of the
   same error value. In the test build the note runs on the same condition, its
   own error propagates identically via `?`, and the original write error still
   propagates after it. **Identical control flow in both configurations.**
4. **`layerfs-workspace/src/file_io.rs` — dropped `u64::from` on a `u64` operand.**
   `u64::from(*file_edits)` where `file_edits: &u64` is the identity conversion.
   **Identity.**

Everything else in the product-source diff of `3e308a8f2` is rustfmt 1.96
reflow/brace normalisation (verified by inspecting every hunk; no token-level
change beyond the four fixes above and string-literal-preserving reflows).

## 3. Rebuild and validation of the frozen candidate

- `runner.py --build-host`: 12.68 s; recompiled `layerfs-monitor`,
  `layerfs-sdk`, `fs-benchmark-pro`; the remaining crates were already current in
  the shared incremental target (artifacts 01:32, all source mtimes ≤ 01:20:32,
  `BatchChildren` present in the `layerfs-content` rlib — no repeat of the #118
  stale-fingerprint anomaly, which is additionally harmless here because the
  only `layerfs-content` change is the transparent alias).
- Full native gate `tools/test-fast.sh` (every test/benchmark exactly once,
  disjoint batches): **PASS, 530 tests/benchmarks, 118 s, exit 0.**
- Candidate identity (frozen before any family):

| Field | Value |
|---|---|
| Commit | `1ff1f2dddeb60493953311de316fa5bec4634a1a` (clean, == `origin/main`) |
| `LAYERFS_SOURCE_SEAL` | `a555211fdc6e96e025764eba2c2c36f709baa7e9446840a07a8669935252fec3` |
| `LAYERFS_PRODUCT_SEAL` | `276c5970aabf485594d90ae30920b3cdb310134a7572589c558063a9d52ce093` |
| Host binary SHA256 | `c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295` |
| Image | `layerfs-bench-infra:a555211fdc6e96e0` = `sha256:c83085b897e272204e5477e66b33ec9b328af568299448872441e7a676ce1761` |

The superseded final-treatment binary `b5f089eb…` remains archived and untouched;
nothing is overwritten.

## 4. Decision

**`3e308a8f2` is behavior-neutral for the product.** The four lint fixes are
line-by-line identity transformations (transparent alias, identity reborrow,
identical control flow, identity conversion) and the remainder is rustfmt reflow.
The #120 candidate therefore behaves as the final treatment did, and the
final-treatment evidence remains applicable: it is **reused with provenance**
(ordinary full157 construction 763.218 s + same-Store verification + census +
Git157 control; historical access 11 + 11 at 2.48–2.88 s; the default-budget
K32000 route + its independent verification; K6000 boundary; the
default-budget frontier proof at 2B). The candidate is nevertheless a **new
identity**: every campaign cell is collected fresh on
`c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295`, and any fix
landed during the campaign produces a further new identity and voids only its
call-path impact set.

The 18 single-sample selections + 18 proofs on `fsync-qualified` and the three
affected SDK-edit selections on `6693224e…` are reused per the frozen contract
("for every mechanism this campaign does not change — cited, not re-run"). Their
rows in the final report name the producing run **and its treatment identity**,
including the note that those treatments predate the #107 pack-coalescing change
(and, for the fsync-qualified rows, the #116 bounded-pending change) that the
final treatment already carried; this is recorded so no reused row is mistaken
for a measurement of the #120 candidate's bytes.

If any cell collected fresh on this candidate shows a material deviation that
traces to one of the four fixes, this decision is voided for that call path, the
impact set is re-run, and the decision is re-recorded here.

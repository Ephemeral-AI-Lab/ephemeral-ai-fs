# #107 ordinary-storage result

Status: mechanism implemented, bounded and independently measured. The frozen
prospective **allocated-byte target is MISSED** and is reported as an explicit
FAIL; the component and apparent-byte reductions it was derived from are
measured and reported below. No waiver is claimed.

Declaration: [issue107-declaration.md](issue107-declaration.md) (mechanism,
target and constraints frozen before implementation and measurement).

## Mechanism

Pack coalescing: the ordinary admission path now fills the **existing** 256 KiB
`pack::PACK_LIMIT` within one `AdmissionSession` (one workspace Commit) instead of
opening a fresh pack row per bounded admission batch. A lane's final pack keeps its
group vector; the next admission of the same framing appends its groups to the
still-open row with one `UPDATE` inside the same transaction, offsetting the
locator group numbers. The pack version bytes, record framing, locator semantics,
exact CAS, authentication, DELTA selection, schema version 10, 4 KiB pages, every
quota and the failure/rollback path are unchanged.

## Matched histories

| Quantity | Recorded baseline (`408f3e2a…`, `full157-candidate-1`) | New treatment (`b5f089eb…`) | Δ |
|---|---:|---:|---:|
| **Allocated Store bytes** (`st_blocks × 512`) | 83,935,232 | **83,902,464** | **−32,768 (−0.039%)** |
| Apparent/logical Store bytes | 83,525,632 | 82,743,296 | −782,336 (−0.94%) |
| SQLite pack overflow-page unused | 2,142,701 | **1,214,634** | **−928,067 (−43.3%)** |
| Pack rows | 3,457 | **1,059** | −2,398 (−69.4%) |
| — `pack_v4` (small content) | 1,663 | 434 | −1,229 |
| — `pack_v1` (legacy metadata) | 702 | 159 | −543 |
| — `pack_v2` (native CDC) | 517 | 152 | −365 |
| — `pack_v6` (pooled metadata) | 575 | 314 | −261 |
| Encoded pack payload | 75,695,515 | 75,799,693 | +104,178 (+0.14%, framing) |
| `objects` locator table | 5,382,144 | 5,443,584 | +61,440 |
| Live-file allocation residue (`st_blocks × 512 − apparent`) | 409,600 | 1,159,168 | +749,568 |
| Frozen-copy allocation residue | 0 | 0 | — |

Provenance: the new treatment's Store is
`issue107-storage-full157/deepseek-full/host-runtime/store.sqlite`
(SHA256 and identity in that run's `identity.json`, `census.json`); 157/157
construction steps PASS.

## Why the allocated target was missed

1. The mechanism's real yield was smaller than the census simulation predicted.
   The simulation regrouped the frozen Store's own pack bytes into per-Commit
   256 KiB packs and predicted −1,933,312 B (−2.30%). The executed ordinary path
   produced 1,059 rows rather than the simulated 785 (one partial tail per lane per
   session), so the realised page-slack saving is 928,067 B — a 1.1% effect on the
   Store, not 2.3%.
2. The live Store file carries **1,159,168 B of allocated-but-unused blocks**
   (`st_blocks × 512` exceeds the logical file size), 749,568 B more than the
   recorded baseline's residue. An identical copy of the same file shows zero
   residue, so this is filesystem allocation residue of the live file, not Store
   content. It is reported, never normalized away, and it is not claimed as a
   saving.

Even with zero residue the same content would allocate 82,743,296 B, i.e. −1.42%
against the baseline: the frozen ≥1.5% target could not have been met by this
mechanism at its measured yield. The target was mis-declared from an optimistic
simulation; the *declaration* was prospective and is honoured as written.

## Latency and resources

- full157 complete performance command 775.002 s (preparation 83.449 s) versus the
  archived baseline 766.577 s (preparation ≈82 s): +1.1%, single sample, no n3
  alternating-pair screen ⇒ recorded as an explicit **WARN**, not a material
  regression under the frozen rule (which requires a median paired slowdown above
  max(15% of control median, 3 ms) *and* ≥2/3 pairs slower).
- No quota, page size, schema, format, worker count or cache change. The session
  holds at most one open pack per framing lane, each bounded by the unchanged
  256 KiB `PACK_LIMIT`.
- `crates/layerfs-layerstack-store` 152/159 tests pass with 7 pre-existing ignored
  (runtime instrumented build); all pack-framing, locator, rollback and
  legacy-format tests pass unchanged.

## Disposition

- PASS: bounded ordinary-Commit mechanism, page-slack reduction, pack-row
  reduction, exact bytes/oracles preserved (see the same-Store verification).
- **FAIL (explicit, unwaived): prospective allocated-byte target ≥1.5%.** Achieved
  0.039% allocated / 0.94% apparent.
- Retained: every attempt and the failed prediction (this document), the simulation
  in `issue107-declaration.md`, and the first full157 run
  (`issue107-coalesce-full157`, superseded for custody reasons: its image manifest
  was replaced by a later build, so its same-Store verification could not reuse the
  identical image).

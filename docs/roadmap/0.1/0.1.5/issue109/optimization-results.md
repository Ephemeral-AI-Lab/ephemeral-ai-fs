# Namespace Init shared-code optimization results (#109)

Execution completed 2026-09-10T19:00Z–22:30Z (Asia/Shanghai 2026-09-11
03:00–06:30). Evidence directory:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue109-evidence/optimization-20260910T190100Z`
(frozen protocol, all attempts, per-zone decisions, samples, profiles,
verification receipt).

Measured and verified here only for `init_namespace / namespace-100000`
(100,000 files / 1,000 data directories / 500,000,000 bytes; timer
`layerstack_init_ns`, public `Client::initialize_layerstack` through its
acknowledgement). Broader family performance and correctness qualification
remain pending. No other benchmark family, Init tier, deduplication case, or
full test/release suite was run.

## Retained shared-code changes (this commit)

All changes are in shared product code; no benchmark- or scenario-specific
branch exists. Zones were accepted or rejected individually against the
frozen `C,X,X,C` plain-timing protocol (n=2/arm, adjacent pairs, screening
margin max(5,000,000 ns, control range)).

| Zone | Shared change | Decision | Isolated plain effect |
| --- | --- | --- | --- |
| A | `PreparedAdmission` retains the SmallContent candidate-search signature computed during preparation and reuses it when a selected FULL winner enters the session candidate index; explicit-predecessor paths compute it lazily at publication. `prepare_small`'s physical-output preflight now explicitly charges the prepared-vector capacity of the enlarged slots. | ACCEPTED (17.1% median; both paired reductions 630–914 ms > 218 ms control range) | 4.508 s → 3.736 s median |
| B | Prepared-statement reuse for `metadata_group`/`next_metadata_ordinal` SELECTs, `ValueIndex::find` SELECT, one prepared INSERT per `ValueIndex::sync` transaction, publication MAX(pack_id)/MAX(ordinal) SELECTs and the metadata-catalogue INSERT, plus a conditional ordinal fallback in `prepare_values` (no eager query when the caller supplied `next`; `index.sync` chronology validation retained). Bundle attribution. | ACCEPTED (5.2% median; both paired reductions 174–297 ms > 30 ms control range) | 4.578 s → 4.342 s median |
| AB | A + B combined. | MEASURED (consistent; 25.6% median) | 4.670 s → 3.476 s median |
| P | Producer-side candidate signature: `PhysicalHints` carries an optional 64-byte signature (mirrored layout check); `FinalizedOutputWriter::put_file_payload` precomputes it on the parallel producer threads for anchor-less SmallContent payloads in small-chain Stores; `prepare_small` reuses it with the unchanged lazy fallback. | ACCEPTED (14.1% median vs AB; both paired reductions 477–499 ms > 51 ms range) | 3.465 s → 2.977 s median |
| S | Final-tree ValueIndex bundle: one scratch transaction per `sync` call (cursor/entries/eviction order preserved; failed commits invalidate through the existing rollback path) and batched absent-value lookups (one bounded IN-list statement per ≤512-value page; first-encounter ordinal assignment unchanged). | ACCEPTED (10.3% median vs P; both paired reductions 276–349 ms > 38 ms range) | 3.024 s → 2.712 s median |
| W | Small-content producer worker cap 4 → 6 (findings priority 6). | REJECTED (inconclusive: mixed signs −43 ms / +212 ms within the 183 ms S range; +2.1 s system CPU). No further count was tried. | — |

The final retained candidate is S = A + B + P + S (Zone W excluded).

## Final declared comparison (original current-source control vs final candidate)

Block `C,S,S,C`, plain receipts, fixture-cache profile
`reused-subsequent-sample-uncontrolled`, fresh independent Stores, no nonce:

| Sample | Arm | `layerstack_init_ns` |
| --- | --- | ---: |
| warm-up (discarded) | C | 4,600,282,583 |
| C1 | control | 4,581,821,917 |
| S1 | final candidate | 2,742,296,084 |
| S2 | final candidate | 2,778,672,334 |
| C2 | control | 4,569,062,916 |

- Median(C) 4,575,442,417 ns; median(S) 2,760,484,209 ns → reduction
  1,814,958,208 ns = **39.67%**.
- Paired reductions: C1−S1 = 1,839,525,833 ns; C2−S2 = 1,790,390,582 ns
  (both far exceed the 12.8 ms control range).
- Canonical objects (112,451), canonical bytes (513,026,835), admission
  transactions (127), Store database bytes ≈ 515.5 MB and swaps (0) are
  identical across arms; RSS growth 78.6–83.0 MB (candidate) vs
  82.5–83.0 MB (control).

**Target gate result: MISS.** The objective was every plain final-candidate
observation ≤ 2,700,000,000 ns (≈2.60 s preferred). The final candidate's two
declared observations are 2,742,296,084 ns and 2,778,672,334 ns — a miss by
42 ms and 79 ms respectively (1.6–2.9%). The 39.7% reduction is retained and
reported; #109 remains open.

## Mechanism evidence (diagnostic scope only)

Nonce-enabled internal clocks and intrusive `/usr/bin/sample` profiles were
used as diagnostics, never pooled with plain results:

- Control (nonce): pipeline 3.256 s, final-tree 1.171 s. Signature scans
  accounted for ~32% of pipeline main-thread samples (two scans before Zone A;
  the retained producer-side scan was measured standalone at ~3.3 ns per
  16-byte window ≈ 0.99 s over the fixture's ~298.5 M windows).
- Final candidate phases: signature work moved to the (blocked) producer
  threads; final-tree reduced to 0.591 s; the serial consumer then idled
  434 ms waiting for producer opens/hashes (evidence for the rejected Zone W).
- Remaining cost after all accepted zones (S nonce, init ≈ 2.76 s): pipeline
  ≈ 2.11 s (≈0.73 s authoritative Store commit page writes for ~515 MB of
  data, SQL inserts, zstd encoding; 0.43 s producer-bound consumer idle) plus
  final-tree ≈ 0.59 s (ValueIndex execution, value INSERTs, group encode).

## Independent verification (outside performance timing)

`verify-selected.py --family init_namespace --case namespace-100000 --seed 1`
against the exact final-candidate host binary (sha256
`dbbf1259186c60122286bb2a0503d6dcccfd0a41f49eb82799b5c3a4d6e6a36b`) and its
matching Linux image: **PASS** (`namespace-sampled-verification-v1`,
import-counts-reopen-and-sampled-fuse-v1; 100,000 files / 500,000,000 bytes
imported; persisted-root reopen equality; 10 sampled files / 132,793 bytes
verified through FUSE; fresh reopen and cleanup pass; wall 35.5 s within the
45 s work allowance).

## Build and qualification custody

Every arm was built with `shared/runner.py --build-host` under the runner
measurement lock, preserving the mandatory linked-schema (schema 10) and
integrated storage-format probes (all PASS). Build walls: 36.5–41.9 s per
candidate. The retained candidate's compilation seal is
`07c0f26fa1b9b50484924f94c3167687a403d9856f2930e207fd931dbfef9bd1`.
The control was the retained qualified current-source binary (sha256
`55acd9c061cafa404ade736b3ec46e222ebad73446ff4fee10af9c76cbea3532`); every
build-input file was verified byte-identical to its recorded snapshot.

Focused correctness checks (verified recompilation) pass for the retained
candidate: 24 tests covering SmallContent candidate selection/late-CAS/
rollback/eviction/retained handoff, the new lazy-signature fallback and
producer-signature paths, metadata pooling/share/reopen/rollback/corruption,
multi-page batched lookups (1,500 distinct values), and bounded
reservations/collision waves. Two earlier focused runs (B, AB) were executed
with stale test binaries due to an mtime-preservation bug in the arm-applier
and are retained as invalid evidence; the comprehensive re-run on the final
arm covers the same code paths.

Execution deviations recorded: one 60 s-watchdog kill of a deadlocked first
test attempt (test-harness session-ownership bug, fixed before timing); one
lock double-acquisition failure and one transient lock-contention failure
(all retained in `attempts.jsonl`).

## Remaining qualification work

- The 2.7 s objective is not met (2.74–2.78 s observed). Remaining dominant
  costs: authoritative Store commit page writes (~0.73 s, data-volume bound),
  producer-bound consumer idle (~0.43 s; the 4→6 worker treatment was
  inconclusive here and needs its own evidence on a quieter host), ValueIndex
  execution and value INSERTs in final-tree construction.
- No other family, tier, seed, Linux-image transfer proof, or release
  qualification was run. All four Init tiers, the three #108 history cases
  and every affected shared caller still require their own measurements and
  proofs before any broader claim.
- Shared-code findings cross-link to #108: Zone B's statement reuse and the
  conditional ordinal fallback live in the same shared metadata/admission
  functions that serve the dedup-history workloads. No deduplication
  benchmark was measured in this execution, so no deduplication improvement
  is claimed.

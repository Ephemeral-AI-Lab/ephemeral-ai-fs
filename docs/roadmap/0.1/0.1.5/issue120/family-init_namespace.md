# #120 family report: `init_namespace` (4 selections)

Candidate: commit `1ff1f2ddd` (`e785e21eb` docs tip), host binary
`c55daf13e372331a5ab6dbd465ece351a55923831c45864325ac46b1508fa295`, image
`sha256:c83085b897e272204e5477e66b33ec9b328af568299448872441e7a676ce1761`.
Protocol: one complete public sample per selection (`--perf-fast
--collection-mode`, 300/310/600 s allowances), seed 1, `--setup fresh`, plus one
independent identity-pinned proof per selection via `verify-selected.py`.

## Per-test rows

Reference values are the published v0.1.3 checkpoint single samples
(`docs/roadmap/0.1/0.1.3/checkpoint-evidence/performance.csv`); their
`cache_profile` is undeclared, so every ratio below is **historical context,
never a paired claim** — the candidate rows are single complete samples on the
frozen candidate, cold for `namespace-100000`, warm-prepared-reuse for the rest.

| Selection | Timer | Candidate | Reference (v0.1.3, profile undeclared) | Ratio | Δ abs | Δ per-op | Ops | Disposition | Proof | Evidence |
|---|---|---:|---:|---:|---:|---:|---:|---|---|---|
| `namespace-100-compact-v3` | `layerstack_init_ns` | 28,703,959 ns | 8,901,875 ns | 3.23× | +19.80 ms | +19.80 ms | 1 | **WARN** (Tier-3; per-op delta over the 10 ms/op hard cap but no n3 comparator exists, so not Tier-2) | PASS 2.18 s | `performance/init_namespace/namespace-100-compact-v3/perf.jsonl` `12ed12f4…` |
| `namespace-1000-compact-v3` | `layerstack_init_ns` | 112,732,750 ns | 38,077,083 ns | 2.96× | +74.66 ms | +74.66 ms | 1 | **WARN** (Tier-3, same rule) | PASS 2.26 s | `…/namespace-1000-compact-v3/perf.jsonl` `f815f797…` |
| `namespace-10000` | `layerstack_init_ns` | 1,020,422,292 ns | 403,467,916 ns | 2.53× | +616.95 ms | +616.95 ms | 1 | **WARN** (Tier-3; aggregate < 1 s) | PASS 2.89 s | `…/namespace-10000/perf.jsonl` `90232524…` |
| `namespace-100000` | `layerstack_init_ns` (cold) | 4,397,542,208 ns **VERIFIED_COLD** | 2,603,162,083 ns | 1.69× | +1.794 s | +1.794 s | 1 | **OWNER-WAIVED** for the registered 2.7 s cold absolute target (TARGET_MISS; `owner-cold-target-waiver.json`, waiver covers that target only); Tier-3 WARN vs the v0.1.3 reference | PASS 7.46 s | `…/namespace-100000/perf.jsonl` `1472cb10…` |

## Cold-contract evidence for `namespace-100000`

`cold_acquisition.status=VERIFIED_COLD` (contract `namespace-100000-cold-v2`,
method `darwin-shared-mmap-invalidate-mincore-v1`): 100,000 files checked,
expected 125,169 pages, **0 resident** after `MS_SYNC|MS_INVALIDATE`, metadata
validation VERIFIED (1,001 directories, mode/mtime contract), backend live
self-check detected 32 warm pages (the detector proves it can see warm data
before certifying cold), no errors, all descriptors/mappings closed before
product execution. Acquisition wall 25.942 s, reported separately from and
inside the complete envelope: complete command 31.615 s = acquisition 25.942 s
+ Init command window 4.919 s + cleanup 0.504 s (cleanup PASS). The complete
command exceeds the ordinary 20 s performance budget as declared before the
family ran; this cell executes under the owner-authorized issue111
cold-qualification allowances (n=1, 300 s product / 310 s command / 600 s
setup), which supersede the ordinary budget for this one cell.

## Diagnoses (mandatory for aggregate deltas > ~1 s)

- `namespace-100000` (+1.794 s vs v0.1.3): one Init of 100,000 files on the
  v0.1.5 ordinary path (authenticated CAS, pack assembly, delta encoding). The
  same cell measured 4,840,907,708 ns on the #104 schema10-uncompacted
  candidate and 2.60 s on pre-authentication v0.1.3; the frozen candidate is
  faster than #104 (−9.1 %) and slower than v0.1.3. The 2.7 s absolute target
  remains owner-waived; the waiver does not extend to any other tier or family.
- The three smaller tiers are 2.5–3.2× v0.1.3 with aggregates ≤ 0.62 s;
  context: #104 measured 21.74 ms / 94.13 ms / 1,252.65 ms on the same cells
  (candidate is faster than #104 on tiers 10000/100000, slower on 100/1000 —
  single-sample diagnostics, not paired claims).

## Cleanup, custody and gaps

4/4 performance samples COMPLETE, 4/4 cleanups PASS, 4/4 independent proofs
PASS (walls 2.18–7.46 s, inside the 60 s envelope). No failures, no S0/S1
findings, no omissions: all 4 registered `init_namespace` selections are
terminal (3 WARN + 1 OWNER-WAIVED with its Tier-1 miss recorded). The
default-budget sequence routes (`--sequence 32000` etc.) are reused from the
final treatment per the frozen contract, not re-collected here.

Receipt hashes (SHA256, pinning the immutable local evidence):
`12ed12f4328bc1c3cc978c0690bcdb277018173fa4f092aa74f7cbdbcd6348ee`,
`f815f797d296b603d5e7bf3e2b933245ad41890fd2e8d122f5d05e81dc7a8fc7`,
`902325241e7e49e668fba866796a3c2243eecf67337012b751a561f2a8ec226e`,
`1472cb10f32e32287e1ba37d2359ae4f7c844544351d102bf3467cad143f3619`;
verification receipts `2bf2166e…`, `a841884d…`, `816a5a04…`, `61ea19fd…`
under `benchmark-results/host-store/issue120/{performance,verification}/init_namespace/`.

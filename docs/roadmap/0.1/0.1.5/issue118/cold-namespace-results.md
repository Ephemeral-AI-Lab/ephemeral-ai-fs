# Issue #118: cold Init and namespace qualification

The current candidate is retained as a measured CPU tradeoff. Against this
session’s pre-edit control, all three paired CPU results improve, with a median
reduction of **235.273 ms (2.855%)**. Cold wall time has **NO_GAIN**: the median
paired slowdown is **39.320 ms (1.039%)**, an explicit permitted **WARN** below
the frozen material-regression bound of 15%. The owner waived the absolute
2.7-second target; all nine raw `TARGET_MISS` results remain unchanged. The
waiver does not apply to correctness, cold-input, resource or cleanup checks.

## Frozen comparison and identities

The prospective order was `H1,B1,T1; T2,B2,H2; B3,H3,T3`. H reconstructs the
historical product, B is the session’s pre-edit control, and T is the current
candidate. H/T and B/T are separate comparisons with three pairs each, sharing
the same three T observations. They are not six independent candidate samples.
The existing `cold.compare` validates both identity compatibility and the
relative pair order; its checks were unchanged. These small samples support
reporting observed differences, without a statistical-significance claim.

| Arm | Host executable SHA-256 | Product identity | Image |
| --- | --- | --- | --- |
| H | `a22785c49e3ace88a99b3f2b1d4e42d250b7921e512a5c8d973bdfcc34e16d05` | `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932` | `layerfs-bench-infra:caa7336aeb6db5f8` |
| B | `fc9192f13c125c0663cd9e8bb53ae2e7361c7af61fccb11a7ffece9afdd89764` | `f8db9e4ea64708ff211332096d4bb31b5d2a20b3582728ecd35fd39eecaabe25` | `layerfs-bench-infra:2f6dbfc942093034` |
| T | `dc07a23f98a77c4520c2f91464359c899167d39240ef50d669d18eecc7e3750d` | `a5f72e1b80516c913370adec514cb9ca373f50d4cdef53b67bb14d31e1d354e3` | `layerfs-bench-infra:e162aecd2b351811` |

All arms used Rust 1.85.1, the same schema hash `1efbb224…`, workload hash
`c6f1e4b1…`, and current common harness `15eccc8e…`. The registered
`namespace-100000` case retained 100,000 files and 500,000,000 logical bytes,
fixture digest `6fc793a9…`, 4 KiB SQLite, authentication, pack/FULL/DELTA
support and four import producers. Before every plain sample, the unchanged
cold-v2 acquisition verified the full input metadata/hash inventory and zero
resident pages. Product read volume independently corroborated that acquisition.
Effects reported here belong to the complete recorded source arms; the work
counts below establish that the selected openat treatment actually executed.

## Plain cold results

| Block | H Init, seconds | B Init, seconds | T Init, seconds |
| --- | ---: | ---: | ---: |
| 1 | 3.843165166 | 3.837565166 | 3.926466875 |
| 2 | 3.965555792 | 3.784265833 | 3.769590584 |
| 3 | 4.059055459 | 3.783918959 | 3.823239291 |
| Median | 3.965555792 | 3.784265833 | 3.823239291 |

B-minus-T paired wall differences are **−88.902, +14.675, −39.320 ms**.
B-minus-T paired CPU differences are **+243.693, +235.273, +155.102 ms**;
median aggregate CPU falls from 8.239352582 to 8.004079833 seconds. Neither
wall nor CPU triggers the frozen material-regression rule. The initial
exploratory screen is excluded from these distributions.

Against H, T’s median paired wall reduction is 195.965 ms (4.942%), and the
median paired CPU reduction is 362.113 ms (4.328%). The median difference of
the two wall distributions is 142.317 ms; it is distinct from the median of
paired differences above.

Every plain row has exactly **112,451 canonical objects / 513,026,835 bytes**,
one process thread before and after, zero host swaps, zero container OOM/swap,
and successful cleanup. T’s peak process RSS spans 80,216,064–81,707,008 bytes;
its quiescent Store spans 515,497,984–515,567,616 apparent bytes and
517,353,472–520,609,792 allocated bytes. Full Store, admission, CPU, I/O,
SQLite and process/container resource receipts are retained per row. The
explicit scratch accounting below is a separate scope from whole-process RSS.

## Historical gap

H is a **new executable** built from the exactly reconstructed historical
233-file product fingerprint. The historical measured executable
`9510f483539b759c9bc138ecbed3da1da5d05f7b694cc6680c09c9633fbeb1e8`
and its raw cells were owner-retired. H combines that product with the current
harness, build and preparation procedure; it is not the unavailable old binary.
One temporary shared local clone was used, with the existing Cargo and BuildKit
dependency caches, then removed after archiving source, manifests and executable.
No external library or dependency sources were patched.

The valid historical plain median was 3.419642375 seconds; the earlier invalid
copied-fixture cells remain excluded. That valid historical product already
included authentication, pack/FULL/DELTA, pooled metadata and fingerprint
indexing. Its reconstruction now measures 3.965555792 seconds, 15.964% above
the retired contextual median. Source additions alone do not explain this
cross-run gap. Its exact cause among build/harness/preparation/runtime conditions
remains unproven, and this candidate does not claim to recover the old level.
Current read-only `pmset` observation reported AC power and no recorded thermal
or performance warning; historical power/thermal state is unknown.

## Namespace correctness, diagnostic and retained failure

All five independent registered proofs passed: imported counts, fresh-reopen
root equality, sampled FUSE content/metadata and cleanup. Their actual coverage
is explicit:

| Proof | Imported files / bytes | Sampled files | Verified byte ranges, bytes |
| --- | ---: | ---: | ---: |
| T, compact 100 | 100 / 5,000,000 | 6 | 148,294 |
| T, compact 1,000 | 1,000 / 20,000,000 | 10 | 141,704 |
| T, 10,000 | 10,000 / 300,000,000 | 10 | 161,912 |
| T, 100,000 | 100,000 / 500,000,000 | 10 | 132,793 |
| H, 100,000 | 100,000 / 500,000,000 | 10 | 132,793 |

The verifier explicitly reports `full_namespace_verified=false` and
`full_file_bytes_verified=false`; sampled checks are not exhaustive content
verification. The three smaller T performance observations qualify those
registered inputs and proofs, with their existing uncontrolled cache profile;
they do not make a comparative cold-performance claim.

One failed attempt is retained. The cached 10,000-file input was missing
`payload/d0028/f002805`, the 100,000,000-byte anchor named in its preserved
manifest. Actual inventory was 9,999 files / 200,000,000 bytes. Init correctly
rejected its scan receipt, emitted no operation timing result, and cleaned up.
The exact owned cache was quarantined by rename, retaining its original data
and manifest. Normal deterministic preparation restored the registered input;
one corrected-input attempt and its independent proof passed, including the
formerly missing anchor among sampled paths. No cold timings were rerun.

The separate nonce diagnostic is intentionally `INELIGIBLE` for statistics,
with **only** the expected diagnostic-ineligibility reason. Its completed
operation and resource checks pass. It records 100,000 relative opens,
100,000 fstats, 1,000 parent-directory opens, 500,000,000 source bytes read,
four workers and zero active producers after completion. The parent-cache
peak is 188 bytes per importer, multiplied by four in explicit accounting;
each importer retains at most one parent descriptor. Total accounted explicit
scratch is **6,090,449 bytes**, below 6 MiB. Checked length/EOF reads, hardlink
identity, leaf symlink rejection and parent/root symlink compatibility remain
covered by the previously qualified focused tests.

## Evidence and execution cost

Raw evidence is under
`benchmark-results/host-store/issue118/20260912/namespace-qualification/`:
`frozen-plan.json`, `commands.json`, `cold-results.json`,
`qualification-results.json`, all individual performance/proof folders,
`fixture10000-quarantine.json`, and the intact `quarantined-fixture10000/`.
The frozen plan SHA-256 is
`238e526519f9d65b746d758e94b5c4f19a6c001cdecbb078a02757317a71942f`.
Historical build/source custody is in the adjacent `historical-cold-gap/`.

There were **19 retained attempts**: the planned 18 plus the justified
corrected-input attempt. Complete command wall sums to 256.511 seconds;
elapsed wall including diagnosis and repair was 410.758 seconds. Historical
host/image builds took a separate 32.341 / 55.652 seconds. All resource work
was serial under the existing runner/verifier lock. No additional cold
samples, statistical outlier removal, or unrelated suite reruns followed.

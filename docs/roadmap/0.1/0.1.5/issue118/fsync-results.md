# Fsync optimization: focused qualification

Product commit: `f8fa59fab`. The owner asked to finish this optimization promptly
and hand the remaining #118 work to the next agent because quota was running low.
This report closes the focused change, not the umbrella issue.

The existing unrelated-history workload is unchanged: each round writes ten
files totaling 1 MiB, calls twelve explicit fsyncs, and requests one Commit.
Development uses the already registered 100-round selection. The full 500-round
selection still has its unwaived 15-second product target.

## Change and retained guarantees

The host previously acknowledged APPEND, cancellation, CHECK and each facts
frame separately; every fsync discarded the remaining append reservation.
The new bounded `BATCH=17` envelope invokes the same handlers sequentially and
returns the exact completed prefix plus the original first error. It validates
all primitive framing before dispatch, and stops at the first handler error.
FACTS_END remains the coherent publication boundary; the batch is not atomic.

Fsync now reuses a bounded reservation tail. An idle tail retains metadata and
a backing reference, with no payload buffer or transfer permit. Full freeze,
turnover and final-reference retirement cancel it. Optional copied envelopes
charge their actual capacity; pressure or oversize falls back to the original
streaming path. Acknowledged APPENDs cannot replay. Unknown or canceled exchange
completion fails subsequent writes while retaining pending bytes. CHECK,
authentication, fsyncs, requested Commit boundaries and quotas remain intact.
Only LayerFS files changed; no external libraries, manifests or lockfile changed.

## Results

Prospective selection and raw evidence:
`benchmark-results/host-store/issue118/20260912/fsync-qualified/`:
`focused-plan.json`, `paired-identities.json`, `commands.json`, `results.json`,
all seven performance rows, two independent proofs and `mounted/`.
The three 100-round pairs used B/T, T/B, B/T order, seed 1 and fresh workspaces.
The control was immutable `qualified-v2`; treatment was `fsync-qualified`.

| Metric, 100 rounds | Control median | Candidate median | Reduction between medians |
|---|---:|---:|---:|
| Complete public product timer | 4.491570335 s | 2.676522959 s | 40.41% |
| Host process CPU | 2.164730124 s | 1.640706333 s | 24.21% |
| Exec calls | 2.945888044 s | 1.173411461 s | 60.17% |
| Matching Commits | 1.535510999 s | 1.469123497 s | 4.32% |
| Workload fsync | 1.753039382 s | 0.468636076 s | 73.27% |
| Backing exchanges | 8,007 | 1,503 | 81.23% |

Every pair improved in wall and host CPU. Median paired wall reduction was
1.905942425 s, or 42.43% of the control median; median paired CPU reduction was
0.555929125 s, or 25.68%. These paired statistics differ from subtracting arm
medians. All samples remain: B=[4.605139920,4.438078832,4.491570335] s;
T=[2.699197495,2.676522959,2.541566081] s. No material regression was observed.

The single full 500-round current-candidate screen **PASSed at 13.791443406 s**,
below 15 s. This is a full-workload observation, not a new n3 full500 comparison.
The old candidate's 23.848 s median remains historical comparison context.
The new screen retained exactly 5,000 writes, 524,288,000 bytes, 6,000 explicit
fsyncs and 500 Commits. It used 7,515 backing exchanges and 8.300964584 s host CPU.

Its disjoint public timer comprises 2.470338053 s workload fsync, 2.096587985 s
other inner workload, 1.645126423 s Exec outside the inner workload,
7.568846821 s Commit and 0.010544124 s other public operations. No work was moved
out of the public timer to obtain this result. Backing wait overlaps these
intervals and is not an additional disjoint network cost.

Every performance sample had 1 MiB physical spool peak, zero physical-observation
errors, zero host/container swap, zero OOM and PASS cleanup. Full500 host peak RSS
was 75,677,696 bytes; Linux container lifetime peak was 9,891,840 bytes. These
remain separate scopes, not a sum claiming an enforced whole-process budget.

## Correctness, cost and retained failures

- Six focused Fuse and nine host checks PASS. They cover malformed input,
  completed-prefix handling, lost replies, cancellation after actual dispatch,
  physical short-write/ENOSPC, chunked facts, tail reuse and retirement, pressure,
  aliases and peer isolation. Final independent review found no concrete blocker.
- Independent 100- and 500-round selected proofs PASS. Their declared oracle
  checks sampled metadata and seven ranges in five files (335,872 bytes), not
  exhaustive namespace, full-file bytes or all historical roots. Separate
  component and mounted checks provide the error/alias semantics coverage.
- Four registered reliability proofs PASS on the new image: invalid SDK edit,
  candidate failure/retry, hardlink alias, open/rename/unlink. Two freshly linked
  real Docker SDK tests PASS: exact lifecycle/disconnect cleanup and ordinary
  writes queued during mapped Commit. No compiler warnings in these final checks.
- Preserved before-fix failure: the optional envelope could fit yet starve local
  dispatch. `fsync-batch-pressure-before.*` records it; releasing the optional
  copy before safe scalar fallback repairs it, and final focused checks pass.
- Preserved selection failure: the first mounted proof reused an obsolete
  input identity from the earlier harness. The verifier rejected it before any
  product workload. Normal `--prepare-only` supplied the current recipe identity;
  four fresh current proofs then passed. See `mounted/proof1/verification.json`.

Complete build commands took 26.473 s host and 62.377 s image (88.849 s total).
Host Cargo compilation itself took 23.329 s, above the preferred ten-second
development goal; compatible dependencies were reused. Seven public measurements
plus two proofs took 69.480 s of complete commands. The full500 performance/proof
commands took 16.115/15.864 s, respectively; their inner product/proof timers are
not advertised as complete workflow cost. Mounted preparation, four proofs,
one 5.028 s incremental test build and two actual SDK tests took 23.356 s,
including the preserved 0.229 s selection failure.

## Exact identities

- Host SHA256: `440ae2c4ce03741676a2845becb9b1661dde341079cc62faad0998847d33a4e5`.
- Source seal: `8053ccba0350395727a01788c7b36de508d4202de27095028473c94be226cc74`.
- Product seal: `c2d5022b8be288304e3ca092eadc543eae8bdf4089198309813157a38e38951b`.
- Image: `sha256:a04cb1d9cd8c2696fdd3b8a4f7deffe2c55c2d38f0aced40be0c213b2227f0e7`.
- Image tag: `layerfs-bench-infra:8053ccba03503957`.
- Daemon SHA256: `be43df8989f05c4513e8ba23f654bca650d8c68bd718701a513f3ca60db0fffa`.
- Fuse SHA256: `a3f6df144b29e3480beb5701b156181a904122192a72bae689017148d83eca38`.

`image-freshness.json` verifies that actual daemon/Fuse binaries changed and the
workload executable remained byte-identical. Executables are independent readonly
copies with source identities; docs-only follow-through does not require a rebuild.
Remaining issue work is in [handoff.md](handoff.md).

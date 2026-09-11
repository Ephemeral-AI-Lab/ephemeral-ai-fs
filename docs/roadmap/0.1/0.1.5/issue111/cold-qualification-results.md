# Cold-only namespace qualification enforcement (#111)

Implemented the owner-authorized [contract](cold-qualification-contract.md),
committed as `96f0aa430` before implementation/timing. This is a harness safety
change, not a product optimization. The cold 2.7 s target remains open.

## Enforced behavior

- The registered namespace-100000 case has one cold policy and a fixed
  2,700,000,000 ns gate. Collection mode cannot waive it.
- Immutable fixture reuse remains internal. Every performance sample validates
  source bytes against the prepared manifest, fsyncs, shared-maps read-only,
  invalidates data pages and performs a whole-input non-faulting mincore sweep.
  A native positive-control self-check must first detect warm pages.
- Failed, unsupported, partial, stale, nonzero-residency or nonce acquisition
  remains diagnostic: raw operation records survive, qualification is INELIGIBLE,
  and qualified timing aggregates exclude the sample. Labels and process read
  volume alone cannot establish eligibility. No automatic warm fallback/retry.
- The campaign loader reclassifies raw receipts rather than saved PASS flags.
  Pair reporting requires eligible members and matching fixture, seed, timer,
  harness/workload, environment, acquisition and observed/declarative order.
  Incompatible pairs have null reduction/percentage and fail the paired campaign.
- The existing Linux resource snapshots bracket the product command after cache
  acquisition; there are no extra container calls. Independent verification and
  release admission remain separate.

## Validation on the terminal source

66 shared-harness tests passed. Regression coverage includes warm 2.6 s with
zero or large read counts, missing/partial/stale/foreign receipts, nonce runs,
wrong case/timer, collection-mode bypass, forged PASS summaries, mixed/incompatible
pairs, valid cold pairs and order checks. The native warm/eviction check passed.

One complete selected performance sample, seed 1, public Client::initialize_layerstack:

| Field | Observed |
| --- | ---: |
| File count / logical input bytes | 100,000 / 500,000,000 |
| Source allocated bytes | 865,730,560 |
| Files checked | 100,000 |
| Expected / observed pages | 125,169 / 125,169 |
| Resident pages after acquisition | 0 |
| Acquisition status | VERIFIED_COLD |
| Acquisition wall, outside Init | 14,361,472,625 ns |
| Legacy fixture_cache_profile field | reused-first-sample-uncontrolled |
| initialization_disk_read_bytes | 865,738,752 |
| layerstack_init_ns | 3,435,659,584 |
| Cold target / observed decision | 2,700,000,000 ns / TARGET_MISS |
| Process exit, including collection mode | 1 |
| Independent verifier | PASS |
| Cleanup | PASS |

n=1 on the terminal source: median=min=max=3,435,659,584 ns.
The report-loader recheck also returned TARGET_MISS. No paired product
control was run, and no speedup is claimed. The retained host binary and product
are unchanged. The standard independent namespace proof is sampled, not exhaustive.

## Custody and retained attempts

- Evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-cold-contract-evidence/20260911T040843Z`.
- Terminal source seal: `b744e31e485f272c266332c4d6030c655e5373f6bfc4f3668b2bd278694659e9`.
- Product seal: `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`.
- Host binary SHA256: `286860778261ea46149add333a46186b06b98f509b8de40622ddd4d1925ff27e`.
- Image: `sha256:22f2db6f37894530bd9fa2affcb024f37770d7710d12501c1e43b633d3b60c82`.
- Fixture digest: `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.
- Prepared manifest SHA256: `1786c23ad222cc74c3214e2707f9c4a140ca0ac197a366ade52fc8013ce5c124`.

`selected-live` and `selected-final` retain earlier integration samples on
intermediate harness seals. They are not pooled with the terminal sample. The
final run followed the resource-snapshot placement correction; no timing-based
retry or outlier deletion occurred. One verifier invocation omitted required
source/input arguments and failed before execution; its exit2 and corrected
identity-bound proof are both retained. Tests/builds/performance/proofs used the
runner measurement lock with no overlapping resource-sensitive work.

The acquisition certifies observed OS source-data page residency under the
controlled runner, corroborated by operation read volume. It does not certify
SSD/controller caches, cold metadata, or immunity to an unrelated process reading
the fixture. Cold acquisition adds explicit setup cost; it is not moved into or
subtracted from the public Init timer. Historical evidence is unchanged and lacks
this new qualification contract, so it cannot be promoted into a v1 gate pass.

The original compaction-removal edits remain uncommitted. No product code,
compression/deduplication policy, release, tag or deployment changed. Related:
#109/#110/#108/#106/#102/#104/#100/#107.

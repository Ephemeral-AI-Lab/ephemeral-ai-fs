# Identical-base Git matcher result

Five fixed original-fixture pairs, after the prospective Git-style amendment.
Every Git COPY/INSERT program replayed exactly to its original target. The exact
program survived pinned Zstandard roundtrip; current FULL/prefix frames also
reconstructed original bytes. No Store format change or allocation claim.

| Pair | Base FULL cost B | Target FULL cost B | Current prefix DELTA B | Git program + same Zstd B |
|---|---:|---:|---:|---:|
| translation-growth | 45,542 | 47,053 | 2,711 | 3,285 |
| catalog-growth | 31,703 | 34,827 | 8,057 | 8,423 |
| icons-growth | 38,207 | 39,177 | 1,220 | 1,171 |
| rename-ledger | 21,702 | 21,439 | 1,713 | 2,259 |
| tool-schemas | 6,575 | 6,402 | 85 | 88 |

Costs include the fixed record framing and16B group directory. Git program costs
include an additional4B prospective decoded-program length. Existing base cost is
identical on both sides and counted once; do not add another base representation.
The historical selected representation of some bases is DELTA, so the illustrative
base FULL column is not its actual incremental Store retention cost. The tool-schema
base is an actual already-selected FULL in the chain-1 Store. SQL, pack headers,
indexes and allocation remain outside this diagnostic table.

Current prefix encoding wins four of five pairs; Git COPY/INSERT wins49B on icons.
This does not establish a universal matcher ranking, but it does not justify a
new COPY/INSERT product format for this candidate. Existing prefix encoding can
already reduce the tool-schema target6402→85B when given the demonstrated base.
Candidate discovery is the selected next implementation owner.

The optimized standalone Git matcher took47,000–610,000ns and used115,249–564,274B
tracked peak live allocations across these pairs, including input buffers and
conservative reallocation overlap. The pinned Rust codec diagnostic used its debug
build; those timers cannot be compared as an optimized speed contest. Neither is
public save/Commit latency. Static tables/stack/allocator internals are outside the
C allocation tracker, and no3MiB product integration peak is claimed from it.

Upstream Git2.47.1 files and GPL headers remain external and verbatim, with URL/SHA
seals. No GPL implementation is copied into the MIT product. The first standalone
build failed because POSIX-only feature visibility hid macOS ru_maxrss; enabling
Darwin's API visibility fixed the diagnostic build. Its failed source/log remain.
The new candidate checks initially had an incorrect test-module path for MissingBatch;
the corrected three focused checks all executed and passed, including the original
available tool-schema pair, shifted fingerprints, negative/self candidates, actual
FULL-base selection, lateCAS target reuse and private rollback.

Evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/`:
`git-matcher-study/{pairs.json,upstream-source-seals.json,git-results.log,*-command.json}`,
`git-matcher-zstd-1.log`, `selected-candidate-check-{1,2}*`.

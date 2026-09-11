# Namespace-100000 cold qualification v1

Prospective harness correction authorized by the owner. Applies only to the
registered `init_namespace / namespace-100000`, seed as selected, full 100,000
files / 500,000,000 logical bytes and public `Client::initialize_layerstack`.
The absolute `layerstack_init_ns` limit is 2,700,000,000 ns. Collection mode
cannot waive this gate. Other corpora, smaller tiers and historical receipts
are not substituted for this case.

Prepared immutable fixture reuse remains internal. Before every performance
sample, under the existing runner measurement lock, acquire cold source data
on macOS: validate prepared file identity, fsync, read-only shared mmap,
MS_SYNC|MS_INVALIDATE, and check residency without faulting payload pages.
Complete a second whole-input residency check after eviction. Record expected
and observed file/byte/page counts, resident pages, errors, fixture identity,
sample binding, method, timestamps and acquisition wall separately from Init.
All descriptors/mappings close before product execution. No source content,
public operation, compiler setting or Store format changes.

This contract concerns OS source-data page residency, not SSD/controller
caches or a cold metadata cache. The residency backend must positively detect
warm data in a live self-check before it can certify anything. Unsupported,
failed, incomplete, stale or nonzero-residency acquisition is UNVERIFIED.
No warm fallback, user cache label or large process disk-read count can turn
UNVERIFIED into VERIFIED_COLD. Read counters are additional consistency
evidence, never the sole proof. Nonce/profile runs are diagnostic only.

Every attempted operation retains its raw timer and cache receipt. Ineligible
samples cannot enter qualification medians, pass counts or improvement claims;
failed acquisition is not silently retried. Registered repetitions still run
as declared. Performance qualification is distinct from independent correctness
verification and release admission.

The shared report classifier revalidates original receipts rather than trusting
saved status/eligibility booleans. Paired reductions require two eligible cold
members with identical fixture, seed, operation/timer, harness/workload,
acquisition procedure, environment and declared control/candidate ordering.
Mismatches yield no reduction or percentage. Historical/uncontrolled data stay
diagnostic and are never rewritten.

Validation: offline regression checks for warm 2.6 s, missing/forged/stale cache
evidence, nonce runs, collection-mode bypass, mixed pairs and compatible cold
pairs; native warm-detection/eviction check; one selected exact-case live run
with qualified host/image builds (n=1, 300 s product /310 s command/600 s setup).
No timing-based retry or speedup claim from that integration run. All attempts,
exit codes and logs retained under a fresh layerfs-cold-contract-evidence root.

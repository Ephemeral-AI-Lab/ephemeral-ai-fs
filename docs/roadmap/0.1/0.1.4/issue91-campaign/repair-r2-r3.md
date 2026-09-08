# R2/R3 prospective bounded repairs after G1 diagnostics

G1 (5eb1414f0) preserves R1's passing byte-equivalence/resource checks. Its four
selected observations are namespace10024883042ns, payload100m834306376ns,
namespace10000012155402250ns and payload500m3825552959ns; all four independent
proofs pass. Small cases did not improve. Large cases improved only partly;
severe published-baseline Init/Commit gaps remain. Do not call R1 complete
qualification. Retain G0/G1 values and allocation/CPU with every result.

## R2: separate transaction ownership from bounded collision-read waves

Current admission duplicates the reader's sum(validation_reserve)<=1MiB gate at
transaction formation, producing130→2519 transactions for namespace100000.
Actual diagnostic namespace100 shows19 transactions,6.853ms SQL Commit work,
23.261ms pipeline wall,47.243ms summed producer blocking,1.069ms consumer idle,
and seven producers. The consumer is occupied; configured parallelism was not
removed. No contention queue evidence justifies weakening admission ownership.

Retain the512-object/512KiB ordinary batch cap, maximal-object isolation, public
8191/4MiB-1 ceilings,2MiB physical and6MiB canonical/data limits. Replace the
transaction-wide validation sum with the existing reader's multiple bounded
waves. Publication must subtract its actual retained physical pack/descriptor
capacities and comparison associations from the existing1MiB pending reserve;
the other1MiB active-decode allowance stays fixed. Do not merely delete the gate.

Use a sorted unique vector of supplied canonical references with explicit capacity
accounting; retain exact ID/length/byte comparisons. Drop temporary IDs once the
lookup finishes. Keep read::visit_locations default behavior; its internal
reserve-limited form rejects limits above the current maximum and partitions by
both the available reserve and128-object cap. Each requested record must fit;
no skipped authentication or raised allowance. Oversized singleton comparison
remains streamed under its existing data-backing exception.

Regression: a valid bounded transaction larger than the old aggregate validation
sum must remain one batch, while a late collision compare spans bounded waves.
Check exact authentication, same-length corruption/error in a later wave,
resource rejection when retained ownership exhausts the reserve, and unchanged
failure cleanup/retained dependencies. Preserve original boundary tests.

## R3: amortize physical extraction within one demanded native group

Corrected worker profile2 retained in raw/diagnostics/g0-read-profile-2 identifies
330 samples in one read_object_rows subtree:188 authentication,94 extraction,
41 decompression,7 other. These sibling counts are not operation-time percentages.
Authentication remains mandatory. CoreReader's authenticated-batch route already
uses Store's authenticated output; no demonstrated duplicate hash may be removed.

Reuse header/directory/BLOB setup only within one existing bounded requested group.
Read only demanded record bodies, never neighboring bodies, and release SQLite
before decompression/authentication. A native group is at most65536 bytes;
requested-record count remains<=128. Account any prefetched records/associations
inside the unchanged active reconstruction scratch bound; no lifetime/global cache,
new service, dependency change or relaxed depth/closure/frame/point-read contract.
Single-record point behavior remains exact. Tests must exercise multiple selected
records, unrelated corrupt neighboring body, malformed directory/locator rejection,
legacy/native coexistence, canonical authentication and bounded ownership.

## Execution

Build/seal G2 only after focused regression/native/memory/corruption tests and
source review. One selected seed1 sample and independent proof for, in order:
namespace100, payload-create100m, namespace100000, store-footprint-metadata-
cardinality100000, payload-create500m, directory-content-scan500, tiny-bulk-create100.
Use original full workloads/300-310-600 and45-59 budgets; R4 preparation for its
three named proofs, separately recorded. No successful earlier observation is
replaced. Failed focused members stop dependent expansion for diagnosis.

Every affected ordinary family/verifier remains required on the final source,
plus additional smokes/full157 and affected phase1 checks. These are bounded
product/harness repairs, not permission to waive remaining severe regressions.

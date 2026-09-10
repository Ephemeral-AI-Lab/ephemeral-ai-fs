# Issue104 representation-aware verifier repair v1

Before implementation, payload_create_read attempt1 completed8/8 performance
cases with all product timing targets met, but all8 proofs failed `unsupported
canonical bytes`. The isolated existing workspace-qualify-fast reproduces this
at `fast qualification exhaustive positive` (exit1). Cleanup passes.

Shared cause: AuthenticatedNamespaceIndex::load still enumerates legacy separate
inode-record object IDs. Schema10 stores logical records inline. The independent
typed census also assumes legacy inode nodes and separate directory-state objects.
Use the existing authenticated visit_inode_records iterator for either format;
traverse inline inode metadata/content dependencies without inventing separate
object identities. Normalize compact directory roots as directory nodes. Preserve
full authentication, ordering, subtree bounds, all inode membership/hardlinks,
metadata/content oracles, corruption rejection and complete object accounting.
The existing CDC-boundary schema applicability must include supported schema10
with the existing SmallContent expectations, without changing source CDC oracles.

Changed owners: benchmark-only workspace_verify.rs index/census and
workspace_bench.rs verification schema selection. Callers include full/fast
Workspace proofs, dedup and canonical census; ordinary sampled SDK resolution
already uses the format-aware product API. Product, workload, fixture recipes,
operation timers, resources and deadlines remain unchanged. Eight completed
performance samples remain applicable under their original64c4e002d binaries;
only failed verification reruns. Verify on the new actual qualified harness and
record linked old-performance/new-verifier custody explicitly. No old seal edits.

Qualification: existing isolated workspace-qualify-fast positive/negative aggregate,
then the smallest failed payload proof, then the other seven failed proofs only.
No later family has started. This is verifier qualification, not restarted #103
product integration. If equivalence cannot be established, mark uncertain coverage
pending instead of silently reusing it.

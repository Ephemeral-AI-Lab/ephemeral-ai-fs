# v0.1.3 versus v0.1.5 Init breakdown

Owner-directed diagnosis before optimization. Start with the retained v0.1.5
plain 3,435,659,584 ns and historical v0.1.3 plain 2,776,088,125 ns receipts.
Neither contains internal phase clocks; never fabricate their exact splits
from companion runs. Explain the costs of small-content FULL/DELTA selection,
compression/packs, pooled metadata and publication, including storage tradeoffs.

Freeze four new nonce diagnostics in order historical,current,current,historical
(n=2/product), seed1, the complete namespace-100000 fixture/digest, independent
fresh host Stores, same direct namespace-init-diagnostic command/public Init.
Before each member use the shared fixed cold acquisition (manifest validation,
fsync/shared mmap/invalidate, positive-control mincore and whole-source residency
sweep). Record each acquisition and physical read count; an unverified member
remains diagnostic with its limitation, never renamed verified cold or replaced
for a nicer time. All commands/outputs/exit statuses and Stores retained.

Retain the historical binary SHA256
6ae5b77f2c923c23b8cf534de0b75a59253b9e8b74003f3c1fcd6547957233eb and its original
identity. Current product seal remains
95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4. Expose existing
physical counters in current nonce output only, outside Init/resource timing.
Requalify the current host/image before execution; edit no measurement inputs
until the four observations finish. Use the runner-owned lock without child
double-acquisition. SQLite/publication/spool remain on macOS.

This is a historical-product diagnostic comparison, not a paired product-speedup
or absolute acceptance claim: harnesses, schema, object layouts, page sizes and
producer policies differ. Historical runs cannot retroactively satisfy the new
cold qualification contract. No feature ablation or product setting change is
authorized by this protocol. No worker-count or prefetch experiment.

Owner clarification during collection: retain 4 KiB pages for storage efficiency.
Historical 64 KiB pages are comparison context only, not an optimization proposal.
Any follow-up must preserve the current 4 KiB storage format.

Report exclusive pipeline/final-tree/import-remainder/outer-remainder sums and
nested consumer idle, SQL commit, encoding and metadata-index clocks. Do not add
overlapping clocks or equate idle with pure I/O. CPU/profile history and code
traces can support hypotheses; they are not per-feature causal wall-time deltas.
Validate phase balances, fixture counts, physical receipt and read-only Store
format census. Reuse retained independent proofs for unchanged product semantics;
these new nonce samples are not release qualification. Preserve original dirty
compaction removal and every sealed evidence root.

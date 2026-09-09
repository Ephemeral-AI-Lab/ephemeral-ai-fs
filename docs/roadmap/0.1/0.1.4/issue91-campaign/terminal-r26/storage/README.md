# Storage evidence package

Start with [report.md](report.md). [summary.json](summary.json) contains exact machine-readable storage/resource values and original input hashes. [report.original.md](report.original.md) preserves the original report byte-for-byte; its earlier campaign-pending wording is historical.

Current terminal-coordinator status at packaging: **198 performance cases and226 routine proofs passed; one optional observation was not run. Comparison-reporter binding is being finalized.** This is an execution update, not a claim that all published-baseline severity/qualification gates passed. Storage remains a separate passed gate:4KiB pages and15.37% lower original acknowledged full157 allocation.

- `control/` retains the passed R25control account,158-receipt validation,157historical-proof summary, original snapshot/census custody and original schedule. It was reused, not measured again for R26.
- `candidate/` retains the corrected R26counterparts. Both official analyzer outputs and unchanged-input custody are included.
- `resources/` contains the exact paired resource summary and field contract; its reused-control schedule association is preserved.
- `failed-r25/` retains the failed all-FULL candidate accounting. It is not superseded into a passing observation.
- [provenance.json](provenance.json) maps every copy to its original absolute path and SHA256, records the human-report transformation, and hashes package artifacts.

No database, roles inventory, graph database, binary, large raw benchmark JSON or checkpoint-resource CSV is copied. JSON custody references retain their original absolute paths and original hashes; omitted upstream objects remain in the original artifact locations. This compact package preserves reviewable report inputs and proof metadata, not a self-contained replay dataset. The source artifacts were not modified or remeasured during packaging.

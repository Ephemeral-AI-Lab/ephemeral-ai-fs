# Cold qualification v2: fixture metadata evidence

Harness correctness follow-up only; no new Init optimization or performance
campaign. Preserve original dirty compaction-removal work and old evidence.

The fixed namespace-100000 fixture includes regular-file mode0640, directory
mode0750 and mtime1700000000000000000ns. Validate the payload root and complete
file/directory inventory, reject extra empty directories/symlinks/special entries,
and check metadata before acquisition and again in its final observation phase.
Do not repair or normalize the input. Metadata mismatch retains UNVERIFIED evidence
and existing raw timing retention; it cannot be overridden by a cache label or read
count. Keep the data-page invalidation/mincore procedure and2.7s target unchanged.

Advance qualification contract to namespace-100000-cold-v2 and require explicit
verified metadata receipt fields/counts in assess/report generation. Missing/old
metadata evidence must fail closed; historical v1 results remain historical under
their original protocol and must not be relabeled or overwritten.

Tests cover the actual copy error (unchanged bytes/file metadata, changed directory
mtime), root/nested directory and file mode/mtime drift, unexpected directories,
partial/missing/forged metadata receipts, raw diagnostic preservation and pair
rejection. Run shared harness tests and one untimed full-fixture acquisition on
original immutable input, plus rejection of the retained bad copy without edits.
All resource-sensitive work uses the runner-owned measurement lock once. Reseal
host/image builds after harness edits; product source/seal must remain unchanged.
Evidence root recorded in /tmp/layerfs-cold-metadata-root. Update #111 and commit
only intended harness/tests/docs. No new timing, release, tag or deployment claim.

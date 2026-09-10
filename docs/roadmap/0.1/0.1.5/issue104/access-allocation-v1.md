# Historical access allocation observations v1

Before first historical_access measurement: reuse the tested physical Store stat
helper on each independent writable copy before the host operation and after its
process has closed, outside the product/resource timer but inside the unchanged
15-second envelope. Record actual copied allocation, complete sidecars, apparent
length and physical identity; report access/fork growth separately, including
verification-only growth. No extra SQL, oracle/digest, compaction or changed input.
The master remains separately SHA-authenticated and unchanged. Preserve all eleven
original checkpoint/path/range oracles and independent verification semantics.
Use the fixture's explicit promoted-uncompacted treatment in its receipts.
This affects only the as-yet unrun historical family; no prior results invalidate.

# R14 prospective page-geometry causal control (never the product candidate)

The published v0.1.3 receipts use65536-byte SQLite pages; the accepted v0.1.4
candidate uses4096-byte pages. R10 identifies per-page SQLite writes and page-cache
pinning as remaining consumer costs after scratch and lookup repairs. Source
inspection alone cannot assign their elapsed contribution.

After G7's four R6 observations/proofs finish, build one separate diagnostic
control from its exact product source240609e6a. The sole product difference is
NEW_STORE_PAGE_SIZE_BYTES=65536 instead of4096. This is a supported existing-layout
granularity, not a changed memory, object, batch, codec, thread, queue, deadline,
or SQLite durability limit. Preserve native FULL/PREFIX bytes, schema7, SDK/FUSE,
all fixtures/timers and every correctness check. Do not merge this control or use
it as the final candidate: the accepted4KiB storage improvement remains binding.
This supplements causal diagnosis and never replaces the published v0.1.3 baseline.

Use a new isolated control worktree/output and newly prepared control-owned Stores
rather than any4KiB cached Store or original/customer Store. Keep the raw native
fixtures byte/metadata-identical to their frozen definitions. Verify the actual
receipt page size is65536 (candidate4096), and retain allocation/file bytes together
with elapsed/CPU/RSS/I/O/spool/cleanup. Source, binary and runtime image hashes must
be distinct and sealed. A wrong page size or fixture makes the control invalid.

One seed1 sample, in order namespace100000 then payload-create500m, each followed
by its independent proof. Standard300/310/600s and45/59s limits; R4 explicit proof
preparation for namespace100000 stays separately reported. One shared measurement
lock, no overlapping build/Store owner. These are unpaired diagnostics against G7's
already declared single observations, not a new statistical paired speedup or an
acceptance waiver. Preserve all failures/outliers. Even a faster control cannot
justify sacrificing v0.1.4 storage improvements.

Then continue the predeclared affected diagnostics/matrix on the actual4KiB
candidate. New-candidate full157 and independent review remain required.

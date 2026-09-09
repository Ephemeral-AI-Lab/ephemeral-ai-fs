# Read-only full157 content-to-Git attribution

Use immutable final D-B-CDC.sqlite SHA25620bc3581f29ecab3168712b8972f2eb24898f92d98e94d402657e953048efa6b, the existing full157 verified Git pack inventory, and only157 selected original manifests. No encoding, repacking, candidate experiment, product edits or source artifacts written.

Authenticate and decode all75,398SmallContent canonical objects using existing StoreReader, with4MiBcanonical and2MiBpack LRU byte caps. Join each exact raw content by Git blobSHA1; assert one-to-one identity coverage. Record frame, actual compact record, compact directory and pack costs separately from Git entry costs. Count BOTH opposingFULL/DELTA populations, so future-base mismatches cannot be mistaken for recoverable totals.

Derive first appearance and prior-path facts while streaming selected checkpoint manifests, retaining one preceding path map. Compute Git transitive base-closure availability from selected-first appearances; do not traverse unrelated upstream history. Current FULL prior-small depth/rawclosure checks are structural eligibility evidence only, not delivered-hint or fallback instrumentation. No causal byte recovery is inferred from a cap category without trials.

Reconcile all remaining Git blobs by original file size/mode with existing native graph accounting; do not unnecessarily redecode large files. Hash source Store and Git inventory before/after. Preserve raw attribution rows, class summaries, source facts and report. Propose the next evidence-led experiment without executing it.

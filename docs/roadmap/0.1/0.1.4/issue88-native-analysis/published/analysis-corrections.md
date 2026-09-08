# Analysis-only corrections and launch observations

The first candidate proof-helper launch was rejected at the exclusive measurement lock before executing analysis; retry succeeded without repeating census. Candidate verification launch1 likewise failed at the lock before creating verifier artifacts or opening a Store while another task built its image. Launch2 is the only candidate verification sample and passed. Logs/exit receipts are preserved.

The initial comparison tool incorrectly asserted per-checkpoint tree-manifest hashes equaled the entire checkpoint-manifest hash. This assertion failed before producing a report. Source inspection confirmed separate identities; the corrected tool authenticates the whole frozen manifest and each corresponding SHA/tree/manifest entry, then all output manifests,158-row validation inputs and ordered mappings. Its next execution passed. No producer, workload, timing or historical evidence was changed.

Independent source review strengthened pre-verification-start detection, fsync, identity/input invariance, per-page validity, performance-to-snapshot binding, cumulative resource scope and invalid-report handling before use. These are report/custody checks, not product treatment changes.

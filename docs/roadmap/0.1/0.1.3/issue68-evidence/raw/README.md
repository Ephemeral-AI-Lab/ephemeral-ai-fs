# Raw evidence bundle

Top-level baseline/final directories are the corrected-input qualification only.
`history/precorrection` retains earlier experiments, the initial cohort, setup failures and sampled-only proofs; they are excluded from the corrected statistics.

Native directories retain result, matching identities, container limits, command exit statuses and original SHA manifests as JSON. `command-traces.tar.gz` contains the original per-command stdout/stderr and verifier output bytes (including empty outputs). Extract beside the manifest to verify their original hashes. Large generated Git fixture tar outputs are omitted as listed in `bundle-omissions.json`; the original hashes and producing commands remain, and fixture content is reproducible from the registered source. No raw measurement or proof receipt was rewritten.

`bundle-files.json` inventories the published files with exact SHA-256 hashes. `final-declaration.json` was committed before the corrected cohort. `preparation-defect.json` and the failed compact proof in history explain why new corrected-input measurements were necessary.

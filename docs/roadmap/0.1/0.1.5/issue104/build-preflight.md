# Issue104 build preflight

Completed before the first family. Terminal evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue104-evidence/campaign-20260910T1115Z`.

| Command boundary | Wall seconds | Outcome |
| --- | ---: | --- |
| First host build and full linked qualification, fresh compiled target |119.80|PASS; not a10s claim|
| Warm unchanged host build and linked qualification |2.285691|PASS|
| Real small Rust edit, optimized benchmark crate recompilation/link |27.563837|BUILD_SLOW|
| Restore original Rust bytes and recompile |27.479643|original binary hash identical|
| Complete host build/qualification after restoration |2.211249|PASS|
| First qualified Linux image and archive |102.260263|PASS; fresh native cache|
| Warm unchanged Linux image/qualification/archive |1.629625|PASS|

Unchanged Cargo itself took0.06s. The real edit added a disposable Rust function
in the benchmark crate; actual Cargo output reports Compiling fs-benchmark-pro.
Dependencies were reused; the large optimized executable's compilation/linking
cost dominates. Linker-only time was not isolated. Release flags, features and
Rust1.85.1 remain unchanged. No cache deletion, clean, prune or stale fingerprints.
The probe function was removed and rebuilt; byte equality with the original
qualified binary was asserted before publication.

Native-input keys avoid fresh native compilation for Python-only collector edits
while full source/harness seals remain recorded. Actual native keys include
Rust/SQL/manifests/lockfile, existing Cargo config, fixed toolchain/profile/targets
and relevant build environment. Build-format probes alone use disposable isolated
Stores, including compaction; none of their outputs is a campaign input.

## Further improvement after the first verifier failure

The owner correctly flagged119.80s and27.56s as too slow. The verifier-only repair
would otherwise create another fully cold target. The retained one-off
`seed-compatible-dependencies.py` verifies the only native changes are two
benchmark verifier source files, unchanged product seal, and a clean tree.
`dependency-compatibility-audit.json` reconstructs the original native seal by
substituting only those two original committed files: it exactly matches the
original810cc87a... key, proving all remaining input/config/environment bytes
match. An independent copy into the new full-input target excludes every
benchmark output and fingerprint. No mutable control/candidate sharing or copied
stale benchmark fingerprint is involved. Copy cost0.591392s.

Qualified repair build then takes29.65s, with28.15s native compilation and no
dependency recompilation: about90.15s less than the measured cold path, with
copy cost separate. This is a different rebuild workload, not a statistical
speedup comparison. The approximately28s optimized benchmark compilation remains
BUILD_SLOW. No caching framework or altered release profile was introduced.

See `build-preflight.json`, `build-timings.jsonl`, `build-first.log`,
`verifier-qualified-build.log`, compatibility/copy receipts and `terminal.log`.

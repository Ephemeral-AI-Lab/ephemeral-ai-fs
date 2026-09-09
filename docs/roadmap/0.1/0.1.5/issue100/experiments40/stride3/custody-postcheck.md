# Supplemental custody postcheck

The first post-verification census assumed the database SHA256 would still equal the performance manifest. That assertion failed; no campaign or verification was rerun and no source database was edited.

Root cause: the existing public verifier at benchmark/fs-bench-pro/src/storage_smoke.rs:1016–1043 forks a named verify branch for each historical commit, mounts it, reads the original oracle and closes the workspace. The53-state Store therefore retains54branches after verification. Canonical count145769 and canonical bytes595887438 equal the frozen performance allocation receipt. The post-verification database SHA matches verification-manifest.json exactly. The original full157 campaign likewise has distinct performance and verification database hashes.

Correct boundary: retain the frozen after-end performance storage value100700160allocated /84844544logical, and the separate verification custody value100700160allocated /84852736logical. Both phases passed. Do not describe public verification as a byte-immutable reader pass; that property belonged to the separate offline diagnostic verifier.

# Verification receipts

Final correctness/quality receipts bind clean source593f4ad018bf34b3f180baf66e1ae5cf40c36647.
`identity.json` records exact source/tree, empty dirty patch, binaries, image,
compiler/dependency identities, host and topology. `manifest.json` hashes these
retained files. Gzip patches decompress to the SHA256 values in
`dirty-patch-sha256.json`; original local files remain unmodified.

Every failed/partial development attempt is retained. Early development
`python3 -` commands did not persist wrapper stdin, and early dirty patches
cover tracked files only; new test files were first fully sealed atc6e2d5901.
Their logs are diagnostic history, not exact-final-candidate qualification.
The final checks use clean committed source and directly named commands or the
frozen helper files here. No historical #88 Store/evidence was modified.

The initial old-code probe used Empty initialization. The final probe adds
8192-byte Directory files, full older/newer reader checks, and observed native
admission before old-open rejection. Its separate Cargo.lock is retained; it
is actual older source linked into a verifier, not an archived shipping binary.

Docker's missing-socket attempts remain failures. The backend quit log identifies
an explicit GUI /app/quit; runtime-check-final-02 and live-final-02 succeeded
after restoring Docker. No failed run was relabeled as passing.

# Ten selected DeepSeek snapshots: three baseline arms

Owner-approved diagnostic scope, 2026-09-09. Select full157 indices
**1, 18, 36, 53, 70, 88, 105, 122, 140, 157**, in order. These are ten complete
snapshot states, not ten full157 runs or 15,632 upstream commits. Skipped states
are not replayed. This supersedes the briefly discussed consecutive-source-commit
interpretation; no performance run used that interpretation.

## Fixed comparison

Case `deepseek-ten`, scenario `deepseek-ten-spread-v1`. One fresh history per arm:
Git, unchanged released v0.1.4 `101fa273d815f3aaedb0e06ba0de7b0777d83def`, and existing
v0.1.5 product from `7ca59e244` (SmallContent implementation plus tested exact-CAS
fix). No new storage optimization, anchor refresh, codec/threshold change or
release qualification in this baseline task. Same harness on both LayerFS arms.
No reruns of unchanged successful arms for better numbers.

Source `/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data`, manifest SHA256
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`, tip
`b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Authenticate selected original Git tree
manifests and original full-content SHA256 oracles. Derive importer previous/current
manifests for the selected transitions. Source blobs come from the pinned Git
repository and are checked against Git blob identities and original oracles.
Prepare once in an external immutable input cache; do not modify original inputs.
Reuse and revalidate that cache for both arms and verification. Fixture generation
and transfer stay outside public operation timers.

## Operations and verification

The Python deepseek-ten case schedules ten steps through the unchanged compiled
deepseek-full host session; its accepted step range already covers 1..10.
Reuse storage_smoke runner, compiled importer, host build and storage-smoke Linux
image entrypoints. Native empty Store/LayerStack Init, then ten ordinary public
Exec/FUSE full-file snapshot imports and public Commit attempts. Preserve changed
and unchanged paths, deletes/type transitions, modes, symlinks and the existing
regular/directory mtime 1000000000 rule. Record actual Created/UpToDate outcomes
and all ten mappings. No SDK editing substitution.

Freeze allocation before verification, close cleanly, record Store digest and
inode/path identity, census read-only, check digest before a new coordinator
reopens the SAME Store. Verify every file byte/path/type/mode/symlink in all ten
states against original oracles. Record successful shutdown/container cleanup.
Do not rebuild another history as the proof. Verification lifecycle mutations
remain outside measured storage/performance windows.

Git reuses the historical matched-control construction: import exactly the ten
selected tree/blob closures into a fresh bare repository without alternates;
create ten deterministic linear commits; disable automatic GC. Measure initial,
constructed and final allocation; final native packing uses compression 6,
window 10, depth 50, threads 2 with forced recomputation, as in the original Git
reference. Run only the final delta-pack lane; no separate no-delta campaign.
Validate ten commit/parent/tree mappings, exact object membership, strict fsck,
and SHA256 content/path/mode/symlink oracles from the SAME retained Git repository.
Git construction/packing are separate observations, not a matched Git-add versus
LayerFS Exec/Commit latency claim. Git does not retain all LayerFS filesystem
metadata or impose the same foreground-acknowledgement packing boundary.

## Budgets, timing and evidence

MacOS owns LayerFS Store/coordinator/canonical publication/spool; Docker only runs
Linux live core/daemon/FUSE/workload. Retain 2 CPUs, 2 GiB RAM, no swap, 256 PIDs.
Runner owns the existing shared measurement lock; serialize builds and arms,
never nest its lock. Reuse existing target/BuildKit caches; no manual cargo clean,
fresh target dirs, Docker pruning, dependency changes or forged seals.

600 s per LayerFS performance/verification phase and fixture preparation; 120 s
per operation; 120 s setup/cleanup. Git commands/each construction or packing phase
<=600 s. Builds retain 900 s/2 Cargo workers. Keep host RSS <=8 GiB, Store and
runtime/spool/staging <=16 GiB, owned run <=32 GiB, free host >=50 GiB, no OOM/swap.
Actual public operation timers remain unchanged. Compare integer-ns save/Commit
min/median/max and paired sums, n=10 dependent observations. No tail-confidence,
release-admission or numerical PASS claim. Report total case wall, verification,
resource scopes and setup/build/preparation/transfer/cleanup independently.

Storage includes allocated Store/repository files and sidecars. Report apparent
length, growth, SQLite pages/freelist, metadata/indexes, FULL/DELTA/PREFIX payload,
physical FULL bases, framing/slack and staging separately. Preserve exact source,
product/harness/fixture seals, dirty patches, binaries/images, commands and failures.
This establishes a small diagnostic baseline; it neither scales linearly to nor
qualifies full157. Full157 remains final confirmation after useful optimizations.

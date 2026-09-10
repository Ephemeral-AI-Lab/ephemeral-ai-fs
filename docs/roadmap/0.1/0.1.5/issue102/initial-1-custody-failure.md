# Initial1: invalid build custody, retained in full

The complete initial campaign produced198 nominal performance passes on each arm,
225 routine proofs on each arm,2 declared optional endurance rows,2 rejected CDC
proof setups, and11 failed candidate historical-access tests plus11 failed
linked proofs. It is **not a valid candidate/control comparison**.

Both labeled host binaries have SHA256
`f90af3adc4a64980ab3e3ab79e74ddae66ad7310de5b6e5bcf744036fc81b5cb`.
The declared control product is891067...5958/schema7; the declared candidate is
24cde1...4673/schema9. The actual candidate fails to open the sealed schema9 Store
with WrongStoreSchema. Its host executable is byte-identical to the released
control. The shared target symlink allowed Cargo's timestamp-based fingerprints
to reuse control artifacts when switching to older-mtime candidate files. The
build wrapper stamped source labels without probing the linked product.

Consequently earlier n=1 flags (payload-create1m and tiny-create500) cannot be
attributed to product changes. Preserve them as invalid-custody observations,
not regressions or improvement evidence. Historical-access failures occurred
before the measured access; original master SHA256 and cleanup checks passed.
#101's earlier successful schema9 opening remains evidence for its own build.

The Dockerfile also shared compiled target caches across source revisions;
its partial package-clean list omitted changed dependencies. Those images cannot
supply an independently justified workaround. Do not reuse initial1 as release
or candidate admission evidence.

## Required correction before further collection

- Partition host compiled targets and Docker compiled caches by complete source
  seal. Keep Cargo registry/download caches shared and preserve old compiled
  caches; do not clean/prune caches to hide the problem.
- Probe each newly linked host executable by creating a fresh host-owned Store
  and reading its actual user_version. Compare it to source schema before
  publishing binary identity. Keep7 and9 probes/binary hashes in custody.
- Replace the CDC proof's hardcoded clone setup with its registered fresh-output
  policy. The two prior ValueErrors remain retained, never relabeled PASS.
- Build byte-identical corrected harnesses for untouched released control and
  unchanged candidate product. Run a selected historical-access case/proof before
  launching the full corrected matrix. Do not optimize product code until a
  valid initial comparison exists.

Raw artifacts: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue102-evidence/initial-1`.
The corrected generation preserves the same case IDs, fixtures, operations,
timeouts and gates. Only build qualification/cache ownership and a pre-work
collector setup error change; no workload or correctness check is weakened.

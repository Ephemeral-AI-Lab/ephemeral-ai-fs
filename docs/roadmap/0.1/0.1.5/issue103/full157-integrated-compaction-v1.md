# Issue103 integrated full157 + compaction v1

Prospective contract, frozen before full157 runner implementation and collection.
Issue: https://github.com/Ephemeral-AI-Lab/layerfs/issues/103 . On 2026-09-10 the
owner authorized the 157-state case after integrated stride3 passed all53 states
at46,202,880 allocated bytes. This extends the previous stride3-only scope; the
broad #102 campaign and release remain outside scope.

## Claim, source and fixed workload

Scenario `deepseek-full157-integrated-compaction-v1`, explicitly selected with
`repository_history --profile stride-1 --storage-compact`. Start from clean
`efda330a72e51c7925b9e11a0ba960fb432733a9`; the product is unchanged from measured
stride3 source `80bc489281735c889a4d62ed576135586be3365b`, product seal
`b1a94e2223b1cd6c0eedffa3c6c60eca7134727c45b9018e1cea518cdf6d3dd5`.
Only runner registration, profile-specific custody and access-fixture binding
need extension. Preserve the prior product, binary archives, Stores and results.
No policy/codec/threshold/page/dependency sweep or unchanged storage rerun.

Use the existing original `deepseek-full` workload and all157 checkpoints at
indices1 through157, in original order, from
`/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data`.
Original manifest SHA256:
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`;
source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`.
Reuse the sealed inputs and original content/metadata oracles. Existing
`deepseek_inputs` independently validates original Git identities, transitions,
input receipts and oracle bytes before use. No omitted or additional snapshots,
fixture rewrites, hidden commits or producer access to verification oracles.

The storage criterion is **complete final allocated Store <=66,000,000 bytes AND
all157 original states verify through that measured product Store**. Decimal MB
is bytes/1,000,000. Report ACHIEVED/MISSED only with complete successful state
verification; otherwise NOT QUALIFIED regardless of observed size. Keep actual
Created/UpToDate/presentation outcomes for all157 attempts. A size miss is retained
and explained; it does not authorize rerunning an unchanged candidate.

Compare with recorded selected offline full15765,957,888B and matching historical
Git15756,373,248B. Report exact margin to66,000,000B, differences and causes. These
are storage references, not fresh paired speed controls. Keep the verified
stride3 result as the preceding observation, without rerunning it for this
harness-only extension.

## Operations, measurement and verification

Inherit the public operation, authenticated product compaction, publication,
resource observation, immutable Store freeze and verification-preimage contract
from [stride3 v1](stride3-integrated-compaction-v1.md), replacing53 with157 and
the profile-specific names. Inherit original full-history fixture, transition,
normalization and per-operation budget rules from
[full157 execution v1](../full157-execution-contract.md). This contract supersedes
that older contract's source/control policy, no-compaction restriction and old
shared-build advice: this candidate uses the already integrated supported
compactor and source-isolated qualified builds. No released-control arm is added.

Execution order:

1. Commit this specification before the minimal runner changes. Run focused
   product-free profile/custody tests. The product-focused and live lifecycle
   proofs already passed on the unchanged product. Build the final clean host
   and Linux image through existing qualified runner entrypoints; verify the
   actual linked schema10/namespace/pool/content107 probe, product seal, archive
   each host/compactor binary plus identity before switching, and archive Linux
   daemon/FUSE/workload bytes with the exact image identity.
2. Acquire the runner-owned measurement lock and create one fresh host Store.
   Execute157 original public Exec/FUSE imports and Commit attempts, preserving
   all receipts, identities, allocation and timing/resource observations.
3. After producer/coordinator/container cleanup, under the same measurement lock,
   call public `LayerStackStore::compact_into` once through the qualified host.
   Record API/process wall, phase CPU/RSS/footprint/I/O, `/usr/bin/time -l`, named
   temporary allocation and100-ms open-file stat observations, including unlinked
   files. Keep the precompaction source intact and charged separately. The new
   destination must be self-contained and publication/sync/cleanup must succeed.
4. Freeze all final Store directory files, allocated/apparent bytes, path,
   inode/device and SHA256 before verification. Retain the closed byte-identical
   preimage in `frozen-measured-store/`, outside the measured directory.
5. Verify all157 original content/metadata oracles by reopening the **same
   measured Store path** through the public product. Preserve every observation
   and original mapping. Record verification-only fork growth separately;
   never replace the frozen workload size with a post-verification size.
6. After all157 verify, run the eleven full157 historical-access cases below
   against copies of the frozen measured preimage, then their separate
   verification cases. Record failures independently from storage success.

The full Store measurement includes every index, pool, physical base, slice
adapter, logical table and required sidecar, not isolated payload or apparent
length. Freeze/seal the performance manifest before verification. Keep every
failed or incomplete attempt with unique non-overwriting evidence paths.

Budgets and ownership are unchanged: macOS owns SQLite/coordinator/canonical
construction/publication/spool/compaction; Linux Docker owns daemon/FUSE/workload.
Runner owns the existing lock, never a nested child lock. Linux2CPUs/2GiB/no swap;
300s per full-history operation,14400s performance and14400s verification,
120s setup/cleanup. Compaction14400s,4,294,967,296B temporary limit, host sampled
RSS<=8GiB, free disk>=50GiB; inherited Store/runtime/spool/staging disk limits.
No OOM/swap, authentication/route failure or incomplete cleanup may pass.
Build limits remain900s/two Cargo workers, with source-isolated targets.

Save/Exec and Commit operation timers remain separate from transfers/setup/
receipts and independent verification. Compaction's intrinsic object/logical
verification stays inside its API timing. Report save+Commit sums, medians,
nearest-rank p95, extrema, work/wrapper walls, compaction costs and resource scopes.
One dependent history is one run, not157 independent repetitions.

## Historical access on the complete history

Profile `historical-access-full157-integrated-v1`; template case suffix `-v2`
becomes `-f157-v1`. Preserve all11 template operations, paths, offsets, lengths,
cache policies, expected output counts, metadata checks and **15-second
end-to-end performance and separate verification limits**, including preparation,
receipts and cleanup. Original checkpoints1/157/65/57 map identically to retained
ordinals1/157/65/57. Do not reuse the stride3 forward mappings.

Fixture preparation runs outside access timers after successful157-state
verification. Bind actual product commit IDs and frozen measured Store SHA256;
retain original checkpoint/source/oracle hashes. Independently derive expected
bytes and metadata from original sealed inputs, never product output. Keep the
original historical-access-v2 and stride3 fixtures unchanged. Report cold encoded/
decoded bytes, pool/base/decompression work and warm-cache behavior explicitly.
Storage success alone does not establish acceptable read performance or complete
release qualification. Do not increase any limit after a miss.

The family remains optional with `admission_eligible=false`: the result can
qualify the stated157-state storage criterion, not the full mandatory #102
registry, a matched released-control speed comparison or a release. No release,
tag, deployment, routine cache pruning or unrelated cleanup is included.

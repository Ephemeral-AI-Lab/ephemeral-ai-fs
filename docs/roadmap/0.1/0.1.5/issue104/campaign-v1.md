# Issue104 prospective promoted-uncompacted campaign v1

Frozen before collector changes and measurement. The live schema10 binary's full
17-family registry passed issue54 validate_campaign, including complete row hash
9c47a0f7ee5911da8bf192d29ff6866d5ba97ef0efe1c5ff68a2f7ffb2cd2579.
The linked binary and archived format probe match the #103 handoff. Read-only
inventory is retained in /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue104-evidence/campaign-20260910T1115Z/live-inventory.json.

## Frozen matrix

Inherit every row, operation, fixture, seed, timer, resource, counter and proof
contract from ../issue102/mandatory-registry.jsonl and mandatory-campaign.json.
Execute host families in declaration insertion order, followed by historical_access.
Complete each family's performance and applicable proofs and print/retain its
terminal summary before the next. 198 host performance, 226 mandatory routine
host proofs, plus 11 access performance and 11 independent proofs: 18 families,
209 performance selections and 237 proofs. No cardinality difference observed.
One ordinary sample, seed/repetition 1; access seed 0/repetition 1. The unchanged
600-second sustained proof and all repository_history profiles are NOT_RUN_OPTIONAL.
Preparation-r4 from issue91 remains required for its three declared proofs.
Cases whose names contain compact refer to existing workload profiles, not permission
for explicit Store compaction.

## Product, control and uncompacted input

Candidate product seal b1a94e2223b1cd6c0eedffa3c6c60eca7134727c45b9018e1cea518cdf6d3dd5,
starting measured source 786d29575b1b7cf1123b5f9b8c97f1e4610c2bab.
Released control v0.1.4 / 101fa273d815f3aaedb0e06ba0de7b0777d83def stays unchanged.
Use identical current harness/workload only if APIs and topology support it;
otherwise INAPPLICABLE with compilation/format evidence. Historical numbers are
context only. Alternate control/candidate then candidate/control by global row
ordinal wherever comparison is valid. Never invent a comparator.

Treatment is promoted-uncompacted. No --storage-compact, compact_into,
layerfs-store-compact, post-workload VACUUM, repacking, GC or fixture rewriting
on campaign inputs/Stores. Normal promoted compact namespaces, metadata groups/
deltas, admission, publication and cleanup remain enabled. Audit call routes,
commands, prepared identities and receipts for zero explicit campaign compaction.
Build-format probes may compact only disposable isolated probe Stores; never
reuse their outputs. Record them separately.

Historical access uses original full157 source at
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue103-evidence/full157-integrated-1/deepseek-full/host-runtime/store.sqlite,
SHA256 b98795995a88a584d2793b5cdf4dc4e8867913fda3349f86dac4274e1f7dfe5a,
83935232 allocated bytes. Bind a new uncompacted fixture with eleven original v2
semantics and checkpoints 1/157/65/57; original path/range/content/metadata oracles
must match, not compacted results. Preserve master; independent writable copies.
Schema10 input is INAPPLICABLE to released schema7 control.

## Gates and reporting

Inherit 300s host product, 310s outer, 600s setup, 45s proof work/59s hard,
all stricter operation/resource gates, and independent proof preparation budgets.
Access performance and verification each retain 15s end-to-end including
preparation, receipts and cleanup. Never raise timeouts or shrink workload.
Report every existing target miss as failure/target miss even in collection mode.
Flag matched operation >20% AND >5ms slower; retain stricter registry review
thresholds as well. One sample is investigation only. Confirm suspicious affected
cases with three alternating matched pairs, retaining every attempt. Without a
valid comparator report absolute deadline/latency; relative regression unavailable.

Before first family print cold/first, warm unchanged, representative small Rust
edit rebuild and full qualification wall independently. Prefer <=10s warm/edit;
report BUILD_SLOW, dominant compile/link/probe costs and targeted improvement.
Keep release flags, toolchain and qualified source isolation. Reuse only proven
compatible compilation inputs; no stale seals/fingerprints, clean or prune.

SQLite/coordinator/publication/spool stay on macOS; Linux daemon/FUSE/workload
only, 2 CPUs/2 GiB/no swap/256 PIDs. Existing runner owns measurement lock; serial
builds/cases and no parent-held duplicate lock. Print commands, timestamps, progress,
exit codes, phase/resource breakdown, complete Store allocated bytes/decimal MB,
apparent/payload/temp/copy bytes and verification-only growth. Unavailable is null.

Append-only attempt ledger and unique case logs/receipts. Resume unfinished work;
never overwrite failures or rerun unchanged PASS for numbers. Every fix documents
files/functions/shared callers, affected families, fixture/timer/build/proof impact.
Only failed and actually invalidated/uncertain cases rerun; retained results retain
original source/build/fixture seals with applicability evidence. A source seal alone
is not global invalidation. Distinguish final-candidate qualification from mixed
identified checkpoints. Genuine global impact must be explained.

After each family print performance/proof completed/expected, failures, slow cases,
wall, outcome and evidence path. Final matrix and slowdown table go to terminal and
artifacts. Update #104 and v0.1.5/#102 docs; close only if obligations satisfied.
No #103 integration restart, release, deployment or post-compaction campaign.

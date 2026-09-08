# Full157 M4.5 storage diagnosis — issue87

**ADDITIONAL DIAGNOSTIC REQUIRED.** The final acknowledgement allocation is
335,552,512 bytes. Payload groups dominate the retained representation;
extra physical anchors and unselected residue are zero under all retained
roots. Missing hint counts do not establish useful missed-byte opportunity.
The one next step is a separately authorized unchanged-policy, byte-weighted
coverage diagnostic. No replay or optimization was executed. Keep #87 open.

- [Findings and exactly one next diagnostic](published/findings-and-next-experiment.md)
- [Independent custody/root-cause review](published/independent-review.md)
- [Sealed artifact manifest](published/manifest.sha256.json)
- [Source, producer and snapshot scope](published/identity-and-scope.json)
- [Four nested physical accounts](published/accounting-reconciliation.json)
- [Authenticated roles and retained closure](published/roles-summary.json)
- [Init plus157 checkpoint trajectory](published/checkpoint-trajectory.csv)
- [Checkpoint field units/populations/provenance](published/checkpoint-fields.json)
- [Counter and timing reconciliation](published/counter-reconciliation.json)
- [Read-only analysis contract](published/contract.md)

Complete immutable local evidence is retained at
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/storage-breakdown-014c0b9-full157-m45-1-issue87`.
The manifest names and hashes every large inventory in that directory,
including object-roles.csv, object-retention.csv, packs.csv, groups.csv,
delta-dependencies.csv and the spillable metadata analysis index. These
large inventories are not copied into Git. The published compact artifacts
are byte-identical to their sealed counterparts; local full-inventory paths
require access to that host. No new release or external storage is created.

Original evidence stays in sibling `full157-m45-1`; its verification manifest
is revalidated at final sealing. The post-verification census cannot replace
pre-verification allocation receipts or reconstruct a missing snapshot.

## Reproducibility without replay

Run only after checking quiescence and original seals, into a **new** output
directory. Do not rerun against the sealed completed directory. These are
analysis executables, not benchmark entrypoints.

1. `trajectory.py RUN OUTPUT FROZEN_MANIFEST` consumes existing receipts only.
2. `physical.py RUN OUTPUT` performs immutable SQLite page/SQL accounting.
3. The [standalone roles analyzer](roles/README.md) performs one bounded,
   authenticated group pass; `roles-report.py OUTPUT RUN` traverses its
   metadata index and `roles-supplement.py OUTPUT RUN` adds field/root details.
4. `physical.py --finish-shared OUTPUT` reconciles shared CSVs, with no Store
   access or second decoding pass.
5. Independently review findings, custody and field contracts; `seal.py RUN
   OUTPUT REPOSITORY` checks original manifests and emits identity/manifest.

`seal.py` intentionally fails on missing/empty required outputs, changed
original evidence or open Store handles. It preserves the single expected
performance-manifest Store mismatch caused by original historical verification;
all final verification hashes must match. It never bypasses product identity
checks or opens a writable product Store.

Small checks: `python3 test_trajectory.py`, `python3 physical.py --self-test`,
and `cargo test --manifest-path roles/Cargo.toml`. The exhaustive inventory
checks authenticate all selected ObjectIds, exact role/reference decoders,
physical FULL bases, page/pack/group conservation, roots and selected locations.
Do not run recursive rustfmt over the included product pack module: it can
format source outside this standalone analysis directory.

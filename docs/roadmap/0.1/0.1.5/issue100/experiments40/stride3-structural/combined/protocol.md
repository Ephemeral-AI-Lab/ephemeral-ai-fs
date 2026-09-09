# Improved offline structural format on stride3 history

Frozen2026-09-10 before component encoding/combination. User requested the same improved format behind79,790,080B full157, applied to the already captured53-state history. No public commit rerun and no new optimization sweep.

Source: /Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-comparison/layerfs/deepseek-stride3/host-runtime/store.sqlite, SHA2565c6ee04eee133539f043ee64d242c26769c77d30b434af2523666e326a8d999e. This is the quiescent verified53-state Store, including its54branch rows. Original selected indices1,4,...157. Matched recorded Git53allocated49,332,224B; Git53apparent48,951,284B. Public LayerFS53performance allocated100,700,160B /logical84,844,544B; post-verifier source logical84,852,736B reflects branch bookkeeping.

Apply the SAME components, with only fixture paths/counts/provenance adapted:
- D canonical metadata: inline inode values, direct directories and eight-byte scoped inode serials; preserve full-width content hashes, scope allocator and exact semantic state. Rederive all dependent typedSQL identities.
- Fixed chronological metadata16edge/128KiB canonical closure, previous-root greatest-shared-key candidate, exact existing matcher/codec/group threshold.
- Git53-selected SmallContent base DAG, pinned existing prefix codec, FULL-versus-delta+32byte-reference comparison, maximum50edges/64MiB closure. No use of unselected Git157blob dependencies; out-of-population origins fall back to FULL.
- Same bounded lockfile CDC similarity policy as prior full157, with actual53-state predecessor-file provenance and all native objects/bases/descendants included.
- Same kind103SmallContent compact packs in dependency order, full32byte object index, all allocator and required SQL rows. Native unchanged except fixed CDC treatment; preserve negative policy outcomes.

Construct fresh matched backup/VACUUM control and a complete fresh candidate; remove replaced metadata/SmallContent physical packs only inside the new copy. Measure actual final logical and allocated bytes. Check all physical records/locators/canonical IDs, dependencies, SQLite integrity/FKs, semantic metadata mapping and transformed SQL references. Verify all53 original oracles via the actual candidate reader, using same4MiBcanonical/8MiBpack caches and exact saved fixture seals. Hash source and candidate before/after.

This remains offline format evidence with extended read costs. Do not infer public Commit latency or product readiness from offline encoding/verification runtime. Keep every prior artifact, failed attempt and selection policy intact. Report matched controls and Git53only; do not compare its ratio to Git15756.37MB. Save a comparable10/53/157table only where policy and measurement scopes really match.

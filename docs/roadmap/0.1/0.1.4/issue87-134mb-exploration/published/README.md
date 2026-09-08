# Aggressive 134.2 MB architecture exploration

**Target:335,552,512→134,221,004 allocated bytes.** Research identifies bounded
whole-file actual-parent prefix encoding and structural-page deltas as the two
large opportunities. Neither has a measured LayerFS saving yet.

- [Reconciled findings and target budget](findings.md)
- [Whole-file/native-prefix and history representation](delta-history.md)
- [Structural lineage, canonical simplification and indexes](canonical-index.md)
- [Compression/pack costs and primary sources](compression-packs.md)
- [Independent review](independent-review.md)
- [Sealed measured accounts and hypothetical target budgets](published/target-budget.json)
- [Native read-only Git pack account](published/git-pack-account.json)
- [Whole-file size profile](published/wholefile-size-profile.json)
- [Complete new artifact manifest](published/manifest.sha256.json)

One target budget to test is70MB payload +45MB structure +19MB complete overhead.
This is an allocation of the goal, not an expected result. Historical Git stores
blob entries in46,982,533 bytes and trees in4,976,275, but final batch bases/depths
and metadata differ from LayerFS. No transfer of that ratio is assumed.

The user authorized aggressive research including canonical/architecture breaks
and speed tradeoffs. Product implementation, retained-data encoding and full157
replay were not run. Prior sealed artifacts remain unchanged. Synthetic codec
roundtrips prove capability only; Zstd resource API values are estimates only.

New full artifacts:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-runs/issue87-134mb-exploration-91a8307-1`.

Tools are self-contained analysis scripts. `target_model.py --self-test` checks
budget arithmetic. `delta-prefix-probe.py` exercises native Zstd on generated
bytes only. `compression_profile_costs.py` queries library estimates without
encoding. `canonical-metadata.py` uses the existing authenticated metadata index;
`git_pack_account.py` reads the existing Git pack with native verify-pack and
checks complete membership/hashes. Do not rerun any tool into sealed outputs.

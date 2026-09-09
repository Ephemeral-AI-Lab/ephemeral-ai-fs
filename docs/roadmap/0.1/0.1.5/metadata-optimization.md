# Metadata anchor optimization — rejected

Parent: `edd09a6d95e3af61bd807bf21f8208e814339e7b` (implementation `6f3fd25eac6336007e6d5936443f2e9b576b13e4`). The retained `candidate-2` ten-file smoke is the optimization control; the unchanged released-v0.1.4 control remains `control-2`.

## Observed cause and prospective change

A read-only census of the retained parent Store found 15 inode-table FULL updates and 15 DELTA updates over thirty commits. Structural packs alternate between approximately 369 and 986 bytes. `DeltaSearch::candidate` skips a DELTA inode origin even though its selected record names a usable FULL anchor. This is the released S1 policy, not a corruption or missing-provenance defect.

Permit schema-8 inode metadata to use the existing bounded anchor-following path. It inspects the known origin, follows its single named base, authenticates that base as FULL and validates its inode-leaf role. The existing matcher, complete group cost selection, limits, packing, admission and rollback paths remain unchanged. Schema 6/7 keep the released policy. SmallContent still has one base candidate and one FULL-base level; no new format, chain, history search, configurable threshold or codec setting is introduced.

This may trade fewer FULL metadata records for larger deltas and additional base reads. Measure the same `small_file_delta_smoke / small-file-delta-10x30-v1` once after matching host/Linux builds. Require 30 Created commits, all 310 file-states verified from the same frozen measured Store, cleanup and actual SmallContent DELTA execution. Compare complete initial/final allocation, growth, encoded metadata, save/Commit latency, verification time and recorded resources. Report regressions; do not infer benefit solely from the new selection count or invent a numerical admission gate.

Only the fixed smoke is run. The existing source-level inode-origin check is updated for the schema-8 policy but is not run as a separate suite. No settings sweep, broad benchmark, Cargo test/Clippy/doctest, release or tag is authorized by this optimization.

## Result and disposition

**Rejected and reverted.** The attempt passed the fixed smoke, but did not improve actual allocated storage. The existing verified implementation and its byte-identical host binary/identity have been restored; its matching image remains `layerfs-bench-infra:e0fc3d082c762f5a`. No unchanged smoke was rerun merely to restore the prior evidence.

| Metric | Verified parent | Anchor-reuse attempt |
|---|---:|---:|
| Initial allocated Store | 155,648 B | 155,648 B |
| Final allocated Store | 221,184 B | 221,184 B |
| Thirty-commit allocated growth | 65,536 B | 65,536 B |
| Final apparent Store | 208,896 B | 204,800 B |
| Metadata pack bytes | 22,451 B | 22,527 B |
| SmallContent pack bytes | 90,305 B | 90,305 B |
| Inode-table FULL objects, including genesis | 16 | 1 |
| Inode-table DELTA objects | 15 | 30 |
| Median Exec, ms | 6.776625 | 6.776208 |
| Median Commit, ms | 6.341583 | 6.395250 |
| Median paired Exec+Commit, ms | 13.138542 | 12.929125 |
| Verification case wall, s | 13.289990 | 14.050011 |

Despite removing fifteen FULL inode tables, cumulative changes against the original FULL anchor made later deltas larger. The new metadata pack total increased by 76 bytes (0.34%). A one-page reduction in apparent database length did not reduce allocated storage; do not equate SQLite logical page count with allocated bytes. The sampled paired median decreased 1.59%, while Commit median increased 0.85% and verification wall increased 5.72%. These single-history observations do not establish a reliable latency improvement and do not rescue the failed storage objective.

The group encoder already applies the existing Zstandard compression and complete FULL-versus-mixed group comparison. There is no omitted compression switch to turn on. The original alternating FULL/DELTA metadata policy avoided the cumulative-delta cost seen here. More DELTA records alone are not an optimization.

The retained implementation therefore remains at 64-KiB growth versus the released control's 72 KiB: **11.11% improvement**, not a newly improved figure. A substantially different result would require a new, concrete metadata/storage design or a separately specified workload, rather than another claim about this unchanged result. No format, page-size, codec or workload change was made to manufacture a gain.

## Verification and provenance

The attempt completed 30 public full-file Exec/FUSE saves and 30 Created commits. All ten files' exact paths, modes, lengths and bytes passed in all 31 retained states after checking the measured Store's frozen digest before reopen. Cleanup passed. SmallContent remained 10 FULL and 30 DELTA objects, each delta referencing a FULL base. A post-smoke metadata census also checked that all metadata DELTAs pointed directly to FULL records; there were no deeper structural chains.

Only the fixed smoke ran. The host build took 53.98 seconds and the matching Linux image build 72.42 seconds, using the established shared measurement lock and existing caches. The source-level unit check was updated for the attempt but was not run; it was reverted along with the policy. No Cargo test, Clippy, doctest, extra family, settings sweep, release or tag was run.

Attempt source seal: `bac503e7d468b8eece788d4a9b5848eb5acd3aaf7b203bf5bbf4d3d110a0770f`.
Attempt product seal: `d9f3894b7bcc6a9f314ba030e03d10f39cd56b83eaafbcb7afff882ab8e14c05`.
Attempt host binary SHA-256: `7329dbd7705e497e0d548ecb4a755d1000fc5553f8a98c53a8489af2538b0c66`.
Attempt image ID: `sha256:9612a352a43a51bea89b4eeb53753ff67685a3d1bce43168c87ffafb19602cd7`.

[Compact comparison](smoke/metadata-anchor-comparison.json) and [per-step raw timings](smoke/metadata-anchor-comparison.csv) accompany this report. Full receipts, the rejected patch, exact binary/image identities, analysis script and build logs remain in `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v015-smoke-evidence/metadata-anchor/`. The earlier successful reports are unchanged.

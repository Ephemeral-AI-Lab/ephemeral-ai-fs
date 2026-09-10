# Issue104 promoted-uncompacted mandatory campaign

**Execution complete; overall qualification FAIL.** 18 families, 209/209 performance selections and 237/237 routine proofs have outcomes. 208 performance targets pass; all 237 proofs pass. One timing target fails.

This is a matrix of individually identified family checkpoints on the unchanged promoted product, not one final host/harness build or a release qualification.

| Family | Performance completed/expected | Target pass | Verification pass/expected | Failures | Slow cases | All-attempt wall s | Outcome |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| payload_create_read | 8/8 | 8 | 8/8 | 0 | 0 | 103.135 | PASS |
| dedup_workspace_reuse | 14/14 | 14 | 14/14 | 0 | 0 | 134.621 | PASS |
| dedup_cross_file | 10/10 | 10 | 10/10 | 0 | 0 | 83.803 | PASS |
| dedup_cdc_locality | 20/20 | 20 | 21/21 | 0 | 0 | 190.095 | PASS |
| edit_length_preserving | 12/12 | 12 | 12/12 | 0 | 0 | 90.873 | PASS |
| edit_length_changing | 32/32 | 32 | 32/32 | 0 | 0 | 159.245 | PASS |
| edit_canonical_chunk_count | 12/12 | 12 | 12/12 | 0 | 0 | 55.810 | PASS |
| init_namespace | 4/4 | 4 | 4/4 | 0 | 1 | 62.529 | PASS |
| store_footprint | 6/6 | 6 | 6/6 | 0 | 2 | 114.992 | PASS |
| tiny_file_churn | 20/20 | 20 | 20/20 | 0 | 1 | 172.935 | PASS |
| namespace_mutation | 4/4 | 4 | 4/4 | 0 | 0 | 31.256 | PASS |
| directory_construction_traversal | 12/12 | 12 | 12/12 | 0 | 1 | 98.401 | PASS |
| workspace_change_locality | 16/16 | 16 | 16/16 | 0 | 1 | 114.940 | PASS |
| dedup_branch_history | 20/20 | 19 | 20/20 | 1 | 5 | 238.970 | FAIL |
| git_tool_workflow | 4/4 | 4 | 4/4 | 0 | 1 | 96.096 | PASS |
| mixed_load_bearing | 4/4 | 4 | 4/4 | 0 | 1 | 62.436 | PASS |
| workspace_reliability | 0/0 | 0 | 27/27 | 0 | 0 | 73.236 | PASS |
| historical_access | 11/11 | 11 | 11/11 | 0 | 0 | 51.961 | PASS |

All-attempt wall sums the executed family invocations, including affected reruns; it excludes intervening diagnosis/build/report time. All observed mandatory resource and cleanup gates passed. Exact scopes and omissions remain in case-results.json.

## Slowness

| Family/case | Classification | Timer | Candidate s | Threshold s | n |
| --- | --- | --- | ---: | ---: | ---: |
| init_namespace/namespace-100000 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 5.298821 | 5 | 1 |
| store_footprint/store-footprint-unique-100000 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 5.557701 | 5 | 1 |
| store_footprint/store-footprint-metadata-cardinality-100000 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 7.765183 | 5 | 1 |
| tiny_file_churn/tiny-bulk-create-500-mixed-v3 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 5.155232 | 5 | 1 |
| directory_construction_traversal/directory-content-scan-500-mixed-v4 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 5.436209 | 5 | 1 |
| workspace_change_locality/workspace-dense-rewrite-500-mixed-v4 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 9.880916 | 5 | 1 |
| dedup_branch_history/dedup-history-distributed-500 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 6.791952 | 5 | 1 |
| dedup_branch_history/dedup-history-hotset-500 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 7.036918 | 5 | 1 |
| dedup_branch_history/dedup-history-unrelated-10 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 10.648693 | 5 | 1 |
| dedup_branch_history/dedup-history-unrelated-100-mixed-v2 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 5.289992 | 5 | 1 |
| dedup_branch_history/dedup-history-unrelated-500-mixed-v2 | TARGET_MISS | pure_call_sum_ns | 23.781066 | 15 | 1 |
| git_tool_workflow/git-tool-500-mixed-v4 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 7.401498 | 5 | 1 |
| mixed_load_bearing/agent-episodes-500 | ABSOLUTE_COMMAND_SLOW | command_wall_ns | 9.181746 | 5 | 1 |

The 5 s command flag is an existing diagnostic, not a deadline/admission failure. Relative regression is unavailable because the released control is incompatible with the current complete harness. No zero-valued comparator or statistical-significance claim is used.

The 500-transition unrelated-history case takes 23.781066044s against 15 s: 500 Exec calls sum 15.601018213s and 500 Commits sum 8.168829081s. Preparation is 0.498389875s, command wall 25.056392791s, cleanup 0.410650709s. Host peak RSS 77,217,792B and Linux lifetime peak 10,739,712B; no OOM/swap. This is accumulated public-call/publication cost, not an isolated outlier. No workload/timeout relaxation or speculative global product change was made. See history-target-miss-diagnosis.json.

## Build and custody

Host first qualified build 119.80s; warm unchanged 2.285691s; actual small Rust edit 27.563837s BUILD_SLOW; complete warm qualification 2.211249s. Exact-input dependency reuse copies in 0.591392s and reduces the next isolated qualified repair build to 29.65s, with 28.15s optimized benchmark compilation/link remaining. No stale benchmark fingerprint/output was copied. First Linux image 102.260263s; warm 1.629625s. Final access host qualification 2.71s and compatible Linux image qualification 15.848061s, including about 10 s standalone workload recompilation. Release flags/toolchain are unchanged.

Promoted product seal: `b1a94e2223b1cd6c0eedffa3c6c60eca7134727c45b9018e1cea518cdf6d3dd5`. Payload performance uses 64c4e002d / host70edec49...; repaired host proofs and subsequent host performance use 84eaa5b61 / host60caa06f...; access uses cebb2f6be with the same 60caa06f... host bytes. Archived sidecars retain full exact source/tree/native/build identities; Python runner identities remain per receipt. Original source seals were never rewritten.

## Retention, treatment and limitations

All eight original payload performance samples were retained when fixing the schema10 inline-inode verifier; only its eight failed proofs reran. Exact prepared master/fixture/product/image/harness/seed equivalence is audited. Twelve SDK performance cases were recollected only because their original receipts lacked required physical allocation; their passing proofs were retained and explicitly linked. Original attempts remain in the append-only ledger.

No explicit compaction command or compaction operation receipt appears in campaign execution. Ordinary promoted compact namespaces/metadata groups/deltas/admission remain enabled. Disposable isolated build-format probes are separately recorded and never used as campaign inputs. No post-workload VACUUM/repacking/GC/fixture rewriting was used.

The authenticated full157 uncompacted master remains unchanged at 83,935,232 allocated B (83.935232 decimal MB), 83,697,664 apparent B, SHA256 `b98795995a88a584d2793b5cdf4dc4e8867913fda3349f86dac4274e1f7dfe5a`. All 157 original oracle hashes were checked; eleven original checkpoint/path/range cases use independent writable copies. Access before/after allocation and verification-only growth are retained separately.

Complete per-case allocated Store bytes, apparent length, source-copy/setup observations and raw resources are in case-results.json/CSV and raw receipts. Physical pack payload and ordinary proof-only growth are unavailable where the schema did not expose them; they are not fabricated or reported as zero. Registered sampled verification is not exhaustive. All three optional repository_history profiles and the 600 s sustained proof are NOT_RUN_OPTIONAL.

Evidence root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue104-evidence/campaign-20260910T1115Z`. Commands and stdout/stderr: `run/commands/`, family logs and `terminal.log`. Append-only ledger: `run/ledger.jsonl`. Raw receipt hashes: `receipt-audit.json`. Full artifact seal: `manifest.json` (created after reporting). #104 stays open for the timing-target failure; no release, tag or deployment.

## Historical access measured envelopes

| Case | Performance s | Verification s | Copy allocated B | Verification-only growth B |
| --- | ---: | ---: | ---: | ---: |
| ha-stat-old-uncompacted104-v1 | 2.390955 | 2.390682 | 83697664 | 0 |
| ha-stat-head-uncompacted104-v1 | 2.279345 | 2.230535 | 83697664 | 0 |
| ha-directory-old-uncompacted104-v1 | 2.230886 | 2.157898 | 83697664 | 0 |
| ha-directory-head-uncompacted104-v1 | 2.279189 | 2.278019 | 83697664 | 0 |
| ha-small-old-uncompacted104-v1 | 2.222776 | 2.161432 | 83697664 | 0 |
| ha-small-head-uncompacted104-v1 | 2.342240 | 2.281323 | 83697664 | 0 |
| ha-range-history-cold-uncompacted104-v1 | 2.223140 | 2.237247 | 83697664 | 0 |
| ha-range-history-warm-uncompacted104-v1 | 2.244799 | 2.224238 | 83697664 | 0 |
| ha-full-head-cold-uncompacted104-v1 | 2.215438 | 2.372591 | 83697664 | 0 |
| ha-full-head-warm-uncompacted104-v1 | 2.282290 | 2.449728 | 83697664 | 0 |
| ha-metadata-worst-uncompacted104-v1 | 2.332077 | 2.317190 | 83697664 | 0 |

All 22 access manifests/completions are authenticated. Independent byte copies preserve the master SHA but allocate 83,697,664B, equal to apparent length; the original master retains 83,935,232 allocated B. This is disclosed copy allocation, not a compacted fixture or storage-optimization claim. No copy grew during its measured performance or separate verifier run.

## Exact build and fixture custody

| Checkpoint | Source commit | Source seal | Host binary SHA256 | Linux image |
| --- | --- | --- | --- | --- |
| qualified-candidate | `64c4e002d3d9da953a426899a7c934dd20c213d3` | `cbc0531c03405d8e9903f313cd37131af27f0dccd0fc8e58e575274b773cb10b` | `70edec49418aec3f70f5ddbce5743838fbba3e80766cd0940939659f085db417` | `sha256:4d87cdc9429abb0ed09f3cb141e2673d0f6305c8bff4bdeafcad1ee56256446f` |
| qualified-verifier-84eaa5b61 | `84eaa5b619fb015dbc5b0175836a2e5451461328` | `53f90fe4f6693845ee9947f5b1db7af81094474e32d0fbcdd08130521326ac58` | `60caa06fd67f0883148bc49c0f320a1f75cbec7fb5ba2770c9c82fc356203c67` | `sha256:4d87cdc9429abb0ed09f3cb141e2673d0f6305c8bff4bdeafcad1ee56256446f` |
| qualified-access-cebb2f6be | `cebb2f6be23312443dfe0d8b7e43a9a8f215761d` | `2423307b59add1e699a23b0f3ea03b1d58e4802d5a9306949b85e47dd94097c9` | `60caa06fd67f0883148bc49c0f320a1f75cbec7fb5ba2770c9c82fc356203c67` | `sha256:36f647c8dd8f4e424c316fc77795969f793c8300ce0a939fee6212c862bbb6f3` |

Native Linux executable bytes are identical across the qualified images. The host verifier-only checkpoint intentionally retains the earlier qualified Linux image, with its real source labels; neither identity is rewritten. Host/compactor sidecars include the exact Git tree, native compilation key, platform, actual schema probe and linked SQLite version.
- fs-benchmark-workload: `e2ae9a6b0859e3542f58dbf93e9088fe9f94e047cd205da8d927e3219d21aa9a`
- layerfs-daemon: `982b20a100535363884ca02060c1db3e952fe7614f2d558e977f7fc64f2625b2`
- layerfs-fuse: `f0b194ab3dbfab50005d0cc0cd818f8b733e896869281dad472d2116f69c3686`

- Bound uncompacted access fixture SHA256: `81a32b5bd7cd311ea70caab465adbe30d513a31853e61a4131b466f96d82ba57`
- Original retained history producer result SHA256: `dae6a35f4dda579210bd5ea229fd84772782352cd2004d54ca6c7b0fbc5162f1`
- Full host registry SHA256: `9c47a0f7ee5911da8bf192d29ff6866d5ba97ef0efe1c5ff68a2f7ffb2cd2579`
- Released control remains `101fa273d815f3aaedb0e06ba0de7b0777d83def`. Inapplicability is supported by the retained API-symbol audit; a new control build or comparative measurement was not fabricated.

Evidence manifest SHA256: `c23e4c2f999e7fca874ea1797dbc2a0d5c5cb4b33458dfa32da42f300992468b`; 2786 retained entries, 274692036 regular-file bytes. The sealed raw evidence stays at the absolute root above. This report is a later documentation commit, not a replacement measured source identity.

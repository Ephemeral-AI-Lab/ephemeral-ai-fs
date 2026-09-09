# Stride3 comparison: public LayerFS and matched Git, 53 states

2026-09-10. **Both sides pass all53 original-state checks. Public LayerFS occupies100,700,160 allocated bytes versus49,332,224 for matched Git:2.0413×, a51,367,936-byte gap.** The reduced workload makes iteration substantially shorter but does not resolve the product's storage gap.

This is the retained public implementation with the new53-state harness. The79.79MB structural full157 artifact remains an offline format experiment with extended read bounds; it was not substituted into this public run. This53-state result establishes the runnable foundation for further experiments. It is exploratory, not release qualification.

## Identical selected histories

Original full157 indices **1,4,7,…,157**, exactly53 states. The selected fixture prepares direct transitions between those states; no skipped-state commits are performed. Source trees, original hashes, logical byte counts and oracles match between LayerFS and Git.

- 53 successful Created outcomes, each with one public Commit and one declared workload execution.
- All53 states verified through the same LayerFS Store: **306,861 path-states /1,676,767,835 logical bytes**.
- Git independently verifies the same53 trees, parent links, exact object membership and original content/type/mode/symlink oracles.
- LayerFS performance/verification teardown PASS; both containers removed. Linux swap and OOM-kill counters zero.

## Storage

| Measured boundary | LayerFS53 B | Git53 B | Difference B |
| --- | ---: | ---: | ---: |
| **Allocated retained storage** | **100,700,160** | **49,332,224** | **51,367,936** |
| Logical/apparent bytes | 84,844,544 | 48,951,284 | 35,893,260 |

LayerFS allocation is frozen at its `after-end` performance receipt, before historical verification. Sidecar allocation is zero; SQLite uses4KiB pages and reports zero freelist pages. Allocated bytes exceed logical file length by15,855,616B. That difference is filesystem allocation accounting, not15.86MB of extra database records or proof of removable SQLite free pages. No VACUUM/offline replacement was applied to improve this public measurement.

Git uses one delta pack of45,912,950B plus2,257,760B of pack index and other repository structures. There are80,596objects, including53new snapshot commits; no loose objects remain. The baseline excludes original unselected commit history and uses no object alternates. Compression6/window10/depth50/threads2 matches the established Git policy. The preliminary loose baseline was422,350,848B allocated and is retained only as lifecycle evidence.

Post-verification read-only census finds145,769canonical LayerFS objects /595,887,438canonical bytes, matching the performance receipt. Pack BLOBs comprise20,094,239B metadata,50,132,007B SmallContent and5,696,523B native content. This census has its own post-verification SQL scope; it is not substituted for the frozen allocated measurement.

## Does every-third-state iteration save time?

| LayerFS timing | This53-state run | Earlier157-state run | Earlier / current |
| --- | ---: | ---: | ---: |
| Total public Commit time | **21.430s** | 53.347s | **2.49×** |
| Performance-phase wall, including its setup/cleanup | **188.923s** | 520.957s | **2.76×** |
| Historical-verification phase wall | **141.100s** | 447.803s | **3.17×** |
| **Performance + verification wall** | **330.023s** | **968.760s** | **2.94×** |

Commit count falls66.24%. The measured development phases together fall from16.15minutes to5.50minutes, excluding first-use builds and outer fixture preparation. Median Commit is0.350s here versus0.273s in the earlier157-state run: fewer commits can contain larger changes, so per-commit latency is not expected to shrink.

This is a descriptive iteration-cost comparison across different workloads and execution times, using the same product seal. It is not a paired identical-workload product speedup or a guaranteed multiplier. Build/fixture preparation remains a separate scope. Git construction55.51s, repacking23.77s and whole control103.04s are different operations and are not compared to public Commit latency.

Peak observed host RSS115,130,368B; host process lifetime peak115,195,904B. Maximum recorded Linux cgroup memory peak138,784,768B. These are separate domains; none is presented as a combined process-memory bound.

## Verification custody

The runner checked performance Store custody before beginning same-Store verification. The public verifier then deliberately creates a `verify-*` fork branch for each historical state, mounts it, reads it and closes the workspace. Thus post-verification SQL contains54branches and the file hash changes. This is the same behavior as the earlier full157 campaign.

- Performance Store SHA256: `c58c4e1c8d7817864f7f7982a1e7986fc587fe32c31b68b0eab2299c85908c98`.
- Verification Store SHA256: `5c6ee04eee133539f043ee64d242c26769c77d30b434af2523666e326a8d999e`, matching the saved verification manifest.
- Post-verification logical bytes84,852,736; allocated bytes remain100,700,160. The extra8KiB of SQL bookkeeping is outside the reported performance storage boundary.

A supplemental postcheck initially assumed verification was byte-immutable and rejected the changed hash. Source inspection identified the fork-branch writes at `benchmark/fs-bench-pro/src/storage_smoke.rs:1016`. The check was corrected to use the verification manifest and preserve the two lifecycle identities. No benchmark or passing verifier was rerun, no database was edited, and no evidence was relabeled. See [custody postcheck](experiments40/stride3/custody-postcheck.md).

## Decisions

1. Keep53states as the fast development track: this measured workload reduced performance+verification wall by about2.94×.
2. Use **49,332,224B allocated Git53** and **100,700,160B allocated public LayerFS53** as this campaign's matched storage foundation, preserving the logical/apparent boundary too.
3. Measure future structural53-state candidates explicitly. Do not compare a reduced-state candidate with Git's56.37MB full157 baseline or claim the79.79MB offline157 format has been tested online.
4. Retain all157 states for final history/scaling qualification.

## Source and evidence

Execution checkpoint `72c2ed688` on `codex/issue100-40mb-experiments`. Product seal `24cde1dce88104daebf2d01e6b711665cfa52c52880b31157fbcfa5c27214673`; source/harness seal `f0088c10a8d47af4cd4fc934d724c01ed0ef6fc8d0590d323ac995b931cdfdd9`. The source snapshot records a local shared-Cargo-cache symlink as dirty; no product edits were present. Matching host and image identities passed the runner's checks. Existing build caches were reused; original saved host binary and new qualified host binary are retained.

Raw evidence: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-comparison`.

- [Comparison result](experiments40/stride3/result.json) and [reproducible summarizer](experiments40/stride3/summarize.py).
- [Campaign protocol](experiments40/stride3/protocol.md), [exact commands](experiments40/stride3/commands.json), [post-verification census](experiments40/stride3/census.json).
- [Stride3 fixture/commands](stride3-fixture-results.md), [frozen contract](stride3-snapshot-contract.md), [optimization ledger](optimization-checklist-and-experiment-ledger.md).

No product format changes, destructive source cleanup, issue publication or release took place in this campaign.

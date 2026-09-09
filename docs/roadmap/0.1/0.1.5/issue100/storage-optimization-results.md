# Issue #100: measured storage optimization outcome

**Kept: 49,319,936 final allocated bytes; 49,250,304 bytes growth from 69,632 bytes. The 45,000,000-byte objective is not achieved. This is 4,319,936 bytes above target and is not in the 45–46-MB near-target range.** No numerical/release-admission PASS is claimed. The issue remains open.

This is a material storage improvement with a disclosed Commit tradeoff: 25.39% below original v0.1.5 and 26.55% below released v0.1.4. Save/paired medians are slightly lower; Commit median/sum are 17.70%/23.64% higher, missing the prospective <=10% working criterion. That criterion was never an owner-approved release gate. Ten dependent history observations do not establish confidence intervals or a tail guarantee.

All ten public outcomes were **Created**, none UpToDate. After freezing allocation and checking the Store digest, a fresh coordinator reopened the **same measured Store** and verified **10 states, 58,860 path states and 327,885,165 logical bytes**, including paths/types/modes/symlinks. Performance and verification shutdown/container cleanup passed. The retained cache is empty after reopen; correctness does not depend on preserving that cache.

## Implementation kept

- Schema-9 bounded SmallContent predecessor chains: maximum 8 edges, 512-KiB summed canonical closure including target, 256-KiB retained encoded capacity. Iterative authenticated reader; unchanged canonical identity and old kind-1 FULL-only grammar.
- Frozen removed-name discovery, including bounded removed subtrees: unique basename/content-root only, genuine predecessors first, no fabricated before inode. Catalogue drops before producers; explicit work/memory caps and low-budget fallback.
- Compact selected-FULL fingerprint cache: each of 1024 candidate records stored once, 8192 u16 references, at most eight probes/one candidate, within the existing 128-KiB reservation.
- Transfer of that same cache only after a retained admission, under the writer permit. No array clone, raw cache, history scan or persisted index. Rollback discards private/inherited hints; Store close/reopen starts empty.

The existing 2-MiB reconstruction and 3-MiB encoding owners, pinned codec/frame settings, strict <131072-B cutoff, CDC8/16/32KiB, 4-KiB new pages, supported nonpromoting old layouts, #95/#98 and <=64-KiB spill read-ahead remain. No cloud/dependency/GC/repacker/reverse rewrite or broad metadata redesign was added. Specifications and prospective amendments are linked from [the roadmap](../README.md).

## Fixed scorecard and exact allocation

Case deepseek-ten, scenario deepseek-ten-spread-v1; full157 indices **1,18,36,53,70,88,105,122,140,157**, in order. Ten complete selected snapshots; skipped checkpoints are not replayed. Immutable controls are reused under [the applicability audit](baseline-applicability-45mb.md).

| Arm | Final allocated B | Growth B | Kept candidate difference B |
| --- | --- | --- | --- |
| Git | 38,223,872 | 38,215,680 | +11,096,064 |
| released v014 | 67,145,728 | 67,076,096 | -17,825,792 |
| original v015 | 66,105,344 | 66,035,712 | -16,785,408 |
| kept candidate | 49,319,936 | 49,250,304 | +0 |

Git2.47.1 retains exactly these ten trees, with compression6/window10/depth50/threads2. Its narrower metadata and separate construction/packing timers remain explicit; no Git-versus-public-save latency ratio is supported.

| Category | Kept candidate B | Git B | Difference B |
| --- | --- | --- | --- |
| content packs / blob entries | 37,879,487 | 34,306,253 | +3,573,234 |
| metadata packs / trees and commits | 6,550,582 | 1,566,186 | +4,984,396 |
| indexes, structures and allocation | 4,889,867 | 2,351,433 | +2,538,434 |
| total | 49,319,936 | 38,223,872 | +11,096,064 |

Candidate total reconciles as **37,879,487 content + 6,550,582 metadata + 4,783,371 SQLite nonpack + 106,496 filesystem allocation difference = 49,319,936 bytes**. Logical SQLite size is 49,213,440 B. Detailed FULL/DELTA records, actual physical FULL and DELTA base subsets, pack headers/directories, SQLite page types/payload/unused space and closure maxima are in the [complete census report](retained-candidate-1-results.md). Physical bases are counted once, not added again to pack totals.

As budget arithmetic only, matching Git content with this measured noncontent allocation would give **45,746,702 B**, near-target, and would still require another **746,702 B** for exact45,000,000. It is not a forecast or achieved allocation. The remaining content difference is **3,573,234 B**; current evidence does not show how to recover all of it under the designs measured. No universal impossibility claim is made.

## Public timings and resources

| Metric | Original v015 | Kept candidate | Change |
| --- | --- | --- | --- |
| exec_ns median s | 2.982280354 | 2.877358249 | -3.52% |
| commit_ns median s | 0.485983625 | 0.571987792 | +17.70% |
| paired_ns median s | 3.468263979 | 3.449346042 | -0.55% |
| exec_ns sum s | 36.068060415 | 35.702063250 | -1.01% |
| commit_ns sum s | 5.806243418 | 7.178636126 | +23.64% |
| paired_ns sum s | 41.874303833 | 42.880699376 | +2.40% |
| performance_wall_ns s | 56.145798708 | 58.616814833 | +4.40% |
| verification_wall_ns s | 26.492623625 | 27.655753750 | +4.39% |

| Step / full157 index | Save s | Commit s | Paired s | Historical verification step s | Allocated B |
| --- | --- | --- | --- | --- | --- |
| 1 / 1 | 0.155065000 | 0.038712167 | 0.193777167 | 0.178329667 | 692224 |
| 2 / 18 | 0.817925000 | 0.190154125 | 1.008079125 | 0.737189833 | 4231168 |
| 3 / 36 | 1.411108542 | 0.292553625 | 1.703662167 | 1.088895833 | 7376896 |
| 4 / 53 | 2.274160417 | 0.443838500 | 2.717998917 | 1.628581042 | 11571200 |
| 5 / 70 | 3.122992333 | 0.624653709 | 3.747646042 | 2.059005125 | 16814080 |
| 6 / 88 | 2.631724166 | 0.519321875 | 3.151046041 | 2.440613458 | 21008384 |
| 7 / 105 | 4.737789625 | 0.970014291 | 5.707803916 | 2.688672750 | 27299840 |
| 8 / 122 | 5.857543167 | 1.105095750 | 6.962638917 | 3.211256583 | 33591296 |
| 9 / 140 | 6.710663625 | 1.337270584 | 8.047934209 | 4.193977333 | 41979904 |
| 10 / 157 | 7.983091375 | 1.657021500 | 9.640112875 | 4.361261000 | 49319936 |

**performance:** host lifetime peak RSS 116,424,704 B; recorded host CPU 15.183215417 s; spool boundary maximum 45,056 B; staging boundary maximum 71,393,280 B. Cgroup CPU/memory categories, OOM/swap/throttling counters and scopes are retained in the complete report.

**verification:** host lifetime peak RSS 69,861,376 B; recorded host CPU 8.078623453 s; spool boundary maximum 40,960 B; staging boundary maximum 1,966,080 B. Cgroup CPU/memory categories, OOM/swap/throttling counters and scopes are retained in the complete report.

Host RSS remains below the original baseline, including the separately declared 8-MiB absolute allowance comparison. Cgroup limits remain 2CPUs/2GiB/no swap/256PIDs. No observed OOM/swap/cleanup failure occurred in these completed ten-state candidates. Boundary maxima are not simultaneous or continuous phase-local peaks.

| Separate scope | Seconds |
| --- | --- |
| performance_work_wall_ns | 54.569486208 |
| verification_work_wall_ns | 26.187865416 |
| setup_ns | 3.403733083 |
| verification_setup_ns | 1.089283917 |
| cleanup_ns | 0.597444875 |
| verification_cleanup_ns | 0.370338750 |
| preparation_ns | 2.980092916 |
| verification_preparation_ns | 3.268211917 |
| transfer_ns | 7.578186084 |

Preparation/setup/transfer/work/cleanup timers have nested existing boundaries and must not all be summed as disjoint costs. Build-host/build-image and complete command walls are separately recorded in the complete report and command receipts. Shared Cargo/BuildKit caches and prepared inputs were reused; no manual clean/prune/fresh-target or uncontrolled cold-cache claim.

## Attempts, rejection and gate decisions

| Candidate | Allocated B | Commit median s | Disposition |
| --- | --- | --- | --- |
| [chain-1](chain-1-results.md) | 56,668,160 | 0.501543562 | kept foundation |
| [selected-full-1](selected-full-1-results.md) | 54,562,816 | 0.544395209 | kept foundation; superseded |
| [removed-base-1](removed-base-1-results.md) | 50,372,608 | 0.580516875 | kept discovery; superseded |
| [selected-chain-1](selected-chain-1-results.md) | 50,364,416 | 0.621925063 | rejected/reverted |
| [compact-candidate-1](compact-candidate-1-results.md) | 50,368,512 | 0.581328645 | kept compact owner; standalone small gain |
| [retained-candidate-1](retained-candidate-1-results.md) | 49,319,936 | 0.571987792 | kept final measured product |
| [retained-chain-1](retained-chain-1-results.md) | 50,360,320 | 0.649245063 | rejected/reverted |

Both DELTA-cache expansions increased content-pack bytes and Commit cost; their complete verified Stores, code patches and identities remain retained. The compact cache and retained-FULL handoff were not described as multi-megabyte content savings: together their content effect is 400,284 B versus removed-base-1, with additional measured allocation differences.

The earlier recent128 long-line ring found no eligible base in its fixed diagnostic. The pinned Git COPY/INSERT matcher lost to the existing prefix frame on four of five identical-base original pairs; its GPL diagnostic code remains outside the MIT product. The nearest-size and parent-score metadata studies are not implemented selection rules or savings forecasts. The retained-DELTA five-family check found one >=2-fingerprint match; its subsequent whole-product measurement lost, and that version was reverted. Cross-CDC reuse remains unimplemented: the direct measured six-target FULL cohort is only112,093 frame bytes, not evidence for closing a4.32-MB gap (nor a bound on all descendants). No additional parameter/codec/cache sweep was run.

**Full157 was not run in this continuation.** The near-target gate was never reached. The earlier premature chain run remains incomplete: its historical verifier stopped at150/157 with forced coordinator stop and diagnostic cleanup, although the container was removed. It is not final confirmation. The original fully verified full157 regression (candidate201,371,648 B vs released184,582,144 B) remains unresolved by these ten-state results. See [the correction](followup-disposition.md).

## Exact source and evidence custody

Retained measured source: **`ee78028ba`** (full revision/seals below). Later reverts restore byte-identical product/harness source; `git diff ee78028ba -- crates Cargo.toml Cargo.lock tools benchmark` is empty. No restored product rerun was performed for a nicer number. The five requested roadmap docs and reports are later documentation edits, not substituted binary identities.

```json
{
  "source": {
    "LAYERFS_PRODUCT_SEAL": "24cde1dce88104daebf2d01e6b711665cfa52c52880b31157fbcfa5c27214673",
    "LAYERFS_SOURCE_COMMIT": "ee78028ba56741002a627b3a8875d3c888c86708",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "2941cd53eab45065b5b2461479805852d5779a4d0c3515a2f2c04baa64097196",
    "LAYERFS_SOURCE_TREE": "c5187d4eccc631895ac942cc0411c70364b12f69",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127"
  },
  "host": {
    "LAYERFS_PRODUCT_SEAL": "24cde1dce88104daebf2d01e6b711665cfa52c52880b31157fbcfa5c27214673",
    "LAYERFS_SOURCE_COMMIT": "ee78028ba56741002a627b3a8875d3c888c86708",
    "LAYERFS_SOURCE_DIRTY": "true",
    "LAYERFS_SOURCE_SEAL": "2941cd53eab45065b5b2461479805852d5779a4d0c3515a2f2c04baa64097196",
    "LAYERFS_SOURCE_TREE": "c5187d4eccc631895ac942cc0411c70364b12f69",
    "WORKLOAD_SOURCE_SHA256": "86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127",
    "binary_sha256": "98d7f8b99466a283b371cfb32b2113168ef44c939aa08cdc47d2b9e9e4d59086",
    "platform": "macOS-26.4.1-arm64-arm-64bit-Mach-O",
    "rust_toolchain": "1.85.1",
    "schema_sha256": "7ed3355be81cfdb82839d1651ee0919afc445ce6255c522f2b1bab54c9a92780"
  },
  "image": "sha256:158f34d7af6e21123e713a661a6e7644bdfcae15a00a76c663b7af04592c94bf",
  "fixture_digest": "eeb408e63b091aa7379aeefd1e3fbf819cdb4d882c67c03dd040c9fbe04feabb",
  "contract_digest": "9ed8ed27c07b16224d82ea751b21c6138c66685cd9c6e5ff9b10f670c56c6d50"
}
```

Original manifest SHA256 `03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`; source tip `b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. Prepared inputs `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-inputs`; immutable three-arm controls `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence`.

New raw evidence, Stores, saved host binaries/identities, image inspections, exact commands/build logs, source/dirty patches, frozen/post-verification manifests, JSON/CSV and diagnostics: **`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence`**. The [retained-candidate report](retained-candidate-1-results.md) links exact artifacts and commands. The execution driver uses the required existing runner entrypoints, fresh output per candidate, census before same-Store verification, and no nested runner lock.

Focused checks cover changed chain readers/upper-range CAS from the earlier chain work, removed-name/direct-subtree behavior, ambiguity and bounds, candidate fingerprints/collisions/eviction, selected bases, late CAS, retained handoff, rollback and cold reopen. Current retained product focused check is `retained-candidate-focused-1`, four passed/one fixture-dependent ignored. No broad Cargo/Clippy/doctest, unrelated qualification or exhaustive failure/POSIX/old-layout campaign is claimed. Final full157 and release qualification remain unrun for this product.

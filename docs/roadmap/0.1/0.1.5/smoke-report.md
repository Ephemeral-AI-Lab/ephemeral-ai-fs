# v0.1.5 fixed smoke result

**Implementation and fixed smoke complete; exploratory evidence, not release admission.** Code commit: `6f3fd25eac6336007e6d5936443f2e9b576b13e4`, branch `codex/v015-small-content`. The released product control is `101fa273d815f3aaedb0e06ba0de7b0777d83def`, with byte-identical released `crates/` source in the separate control worktree. No release was published or tagged.

The two measured histories use the byte-identical `small_file_delta_smoke / small-file-delta-10x30-v1` harness and fixture. There are ten flat source files, 240 KiB at genesis, thirty fixed overwrite/insert/delete operations, one full-file Exec/FUSE save and one public Commit per step. Native Init is measured separately. Each arm is one history, not thirty independent repetitions.

## Correctness, mechanism and custody

| Requirement | Released control (`control-2`) | Candidate (`candidate-2`) |
|---|---:|---:|
| Native Init | Completed | Completed |
| Public Exec saves / Created commits | 30 / 30 | 30 / 30 |
| Retained states verified | 31 | 31 |
| Exact path/mode/length/content file-states | 310 | 310 |
| Frozen Store digest checked before reopen | Matched | Matched |
| Performance / verification cleanup | PASS / PASS | PASS / PASS |
| Store schema | 7 | 8 |
| New small representations | Inapplicable | 40 |
| Small FULL / DELTA objects | Inapplicable | 10 / 30 |
| Selected FULL bases / maximum delta depth | Inapplicable | 10 / 1 |

The independent SHA-256 oracles cover all ten files in every retained state. The verifier reopens the SAME measured Store with a new coordinator after allocation observations are frozen; it does not regenerate history. Candidate record inspection runs after performance, before reopen: 31 v3 packs, 87,849 compressed frame bytes and 89,169 group bytes. The ten FULL anchors retain 245,760 raw bytes. These are payload/record quantities, not total allocated Store storage. DELTA reconstruction and canonical authentication execute through the integrated public FUSE history reads.

The fixture self-check passed exact sizes, line construction, changed-file cardinality, eight-byte hex rotations, inserted/deleted lines, unchanged siblings, deterministic preparation and all 31 oracles. Fixed watchdogs are 600 seconds per phase and 30 seconds per operation. Both final runs report clean coordinator shutdown and container removal, no observed OOM or swap violation. Spool/staging are reported separately from Store bytes.

## Matched storage and latency

Allocation is the sum of Store and applicable sidecars after acknowledgement. Final means after the thirtieth Commit, before verification mutates any lifecycle metadata. No VACUUM, pruning, GC or repacking is credited.

| Metric | Released control | Candidate | Candidate change |
|---|---:|---:|---:|
| Initial allocated bytes | 155,648 (152 KiB) | 155,648 (152 KiB) | 0 |
| Final allocated bytes | 229,376 (224 KiB) | 221,184 (216 KiB) | −8,192 B (−3.57%) |
| Thirty-commit allocated growth | 73,728 (72 KiB) | 65,536 (64 KiB) | −8,192 B (−11.11%) |
| Initial apparent bytes | 155,648 | 155,648 | 0 |
| Final apparent bytes | 212,992 | 208,896 | −4,096 B |
| Init, ms (one observation) | 4.614459 | 4.349083 | −5.75% |
| Exec save, ms; median (min–max), n=30 | 6.297896 (5.287334–13.762250) | 6.776625 (5.628875–10.152292) | Median +7.60% |
| Commit, ms; median (min–max), n=30 | 6.242500 (5.520042–8.319834) | 6.341583 (5.430000–10.381334) | Median +1.59% |
| Paired Exec+Commit, ms; median (min–max), n=30 | 12.452438 (10.983667–22.082084) | 13.138542 (11.209375–16.815708) | Median +5.51% |

Paired values are the sum of the two operation-local integer nanosecond timers for each step. They exclude preparation, transfers, receipts, storage census and verification. The raw values and each step's allocation, growth, save/Commit latency, transfer time and spool/staging observations are in [comparison.csv](smoke/comparison.csv); operands, exact medians/ranges and identities are in [comparison.json](smoke/comparison.json).

The candidate saved 8 KiB of final allocated storage and reduced growth by 11.11%, while its paired median increased by 0.686104 ms. Median host CPU was 1.965896→2.269751 ms for Exec and 2.945291→2.916647 ms for Commit; median recorded encoding work was 0.140980→0.159459 ms. These observations do not isolate a transport, scheduling or codec root cause for the elapsed regression. No unchanged candidate was repeated to obtain a better number, and no numerical PASS is claimed: this ten-file contract specifies no numerical threshold. The historical three-file figures and gates are not used.

## Build, setup, verification and resource observations

| Scope | Released control | Candidate |
|---|---:|---:|
| Matching host build command wall, s | 67.86 | 39.99 |
| Matching Linux image build command wall, s | 1.48 | 71.60 |
| Fixture/source/image acquisition/preparation, s | 0.294851 | 0.343788 |
| Performance container/coordinator setup including Init, s | 2.205483 | 1.708539 |
| Thirty untimed input transfers, s | 7.478107 | 7.794224 |
| Performance case wall including setup/cleanup, s | 19.488733 | 20.093899 |
| Verification setup, s | 0.862174 | 0.982301 |
| Verification work wall, s | 11.757017 | 11.908406 |
| Verification case wall including setup/cleanup, s | 13.048271 | 13.289990 |
| Maximum observed spool allocation | 126,976 B | 126,976 B |
| Maximum observed container staging allocation | 81,920 B | 81,920 B |
| Maximum recorded host lifetime peak RSS in operation receipts | 15,122,432 B | 16,793,600 B |

Build conditions differ: the final control Linux image reused its compiled layers after the Python custody/watchdog correction, whereas the candidate rebuilt changed shared consumers. These are build/setup costs, not product speedup comparisons. The host build uses the existing release target; Docker uses the existing storage-smoke entrypoint and BuildKit caches. Required targets were the host `fs-benchmark-pro`, Linux `layerfs-daemon`, Linux `layerfs-fuse` proxy, and compiled workload helper. No Cargo test, Clippy, doctest or other product/test campaign ran.

RSS is a recorded process-lifetime high-water observation, not a codec reservation or exact phase-local heap peak. Spool/staging are boundary observations, not continuous peaks. Container settings remain 2 CPUs, 2 GiB, no swap and 256 PIDs. Builds and sample/verification commands used the existing shared measurement lock, without nested runner acquisition.

## Exact identities

| Identity | Released control | Candidate |
|---|---|---|
| Product source seal | `891067915b4152dd42e7510ef374a3d8a8827ead03a597051335c933d0485958` | `85f14054f9679f5c7943afad9f5047b3c50e977e5422c120cf8df7678d5b2036` |
| Combined source seal | `2a32069c6b5b510a2cfd522bf85651b83e6ce2ba26e0f67a9ece6dc705201342` | `e0fc3d082c762f5a3c703d6838d5729c5aa8f1376d14f168ef293347db8081d1` |
| Host binary SHA-256 | `21a53dc8b7b58f5fbdf53767a8f0eed68d75d57ba0c242789e52d7c49e87aa5a` | `dcaefaa2d79f39a9fee9dd34b30f63d5f35dcaf2c5563be19260c91e02526178` |
| Linux image ID | `sha256:e169c088fad85c8909ebb775087a2dfc9f5da4d31699665a14e15c1ed0e2247d` | `sha256:65119034fb5b386d137280573d13055a98d4ce711aa0c0c29e6340af0831d5ae` |
| Fixture description/oracle digest | `b14b375db4d1bef82ae1b1779ce8cc8fe3f146f4d134975b42c0b5a9eec80c27` | `b14b375db4d1bef82ae1b1779ce8cc8fe3f146f4d134975b42c0b5a9eec80c27` |

Shared benchmark source digest: `f23f36e022d7e735b65b37061db96c65bce046f205b7200b44673cf062b35239`. Workload main-source digest: `86a12224417d3972c29c62c134e019a0ce8e80cf5360df5394f3537e15901127`. Digests include their recorded byte/path framing; the complete saved identities retain source commit/tree, dirty status, platform, toolchain and schema digest. The candidate host binary was built from the staged source before implementation commit 6f3fd25ea; its saved precommit identity is retained. The content seal matches the committed code and the matching image; no identity was relabeled. The untracked `target` symlink used to reuse the existing cache is also reflected honestly in dirty status.

Raw artifacts, retained Stores, binaries and build logs are at:

```text
/Users/yifanxu/Ephemeral-AI-Lab/layerfs-v015-smoke-evidence
```

`control-2/` and `candidate-2/` are the compared final runs. Their performance and verification manifests bind the receipts. `small-mechanism.py` and `summarize.py` retain the post-measurement census and report derivation. Raw Store/binary artifacts remain outside Git; the report and compact comparison are committed.

## Failures and later qualification

The original `control/` passed but was superseded by the verifier watchdog and same-Store digest check; it was not replaced to improve a number. Original `candidate/` completed performance and emitted 10 FULL/30 DELTA records, then failed genesis verification because batched inode acquisition retained an extent-only decoder. Its coordinator reported cleanup success and its container was removed; the wrapper classified verification cleanup as diagnostic because the operation failed. The common batch dispatch fixes this path. The final candidate also accounts for larger regular-root acquisition pages and bounds simultaneous producer buffers. All earlier logs/receipts remain retained.

Compile failures, source changes and the shared-target timestamp-cache repair are recorded in [implementation notes](implementation-notes.md). The bounded parser, size-transition, lifetime and upgrade code is implemented, but this smoke does not qualify malformed inputs, interrupted upgrade, corruption/collision/failure injection, old/new format combinations, threshold crossings, hard-link/open-unlinked/rename concurrency, worst-case ownership or large-file locality. Those remain later qualification. No broad matrix, old tiny case, admission issue, release publication or tag was added.

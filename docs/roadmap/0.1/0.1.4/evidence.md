# Supporting evidence for v0.1.4 storage efficiency

Status: planning evidence index, 2026-09-08. These observations motivate the
release priority; they do not qualify a v0.1.4 candidate or select an optimization.

## Baseline and experiment custody

The [v0.1.3 checkpoint report](../0.1.3/checkpoint-evidence/report.md) records
198 performance cases and 226 routine verification cases passing their recorded
completion gates, with latency-target misses retained and the optional 600-second
test excluded. PR #76 merged at `9f5a641d2`. This is a benchmark closeout, not a
claim here that a release tag was published.

[Issue #72](https://github.com/Ephemeral-AI-Lab/layerfs/issues/72) is the supporting
experiment record. Preserve its original description, versioned amendments,
failed runs, and results. Do not relabel exploratory results as release evidence.
The reports below are pinned to documentation commit
`1c7c9235115d1b4f21bc2eae7af822552b7be3ed`; each report separately records its
measured product, input, binary, and runtime identities. That documentation
commit is not a shared measured candidate for all experiments.

## Findings and their planning implications

| Evidence | Recorded observation | Implication |
| --- | --- | --- |
| [DeepSeek retained history](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/docs/roadmap/0.1/0.1.4/deepseek-history/results.md) | All 157 selected states verified; 4,936,693,030 logical bytes; 940,310,528 allocated Store bytes; 80.95% savings against calculated independent payload copies | Sharing and correctness are demonstrated; independent-copy savings do not establish competitive compactness |
| [Matched Git control](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/docs/roadmap/0.1/0.1.4/deepseek-history/git-control-results.md) | Same selected root trees: 56,373,248 allocated bytes in delta-packed Git versus 940,310,528 in LayerFS, a 16.68× allocation ratio | Retained storage is a material optimization opportunity |
| [Object accounting](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/docs/roadmap/0.1/0.1.4/deepseek-history/results.json) | 799,638,421 canonical object bytes; 140,672,107 bytes of Store allocation above canonical bytes | Distinguish content representation from physical and record overhead |
| [Small-edit control](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/docs/roadmap/0.1/0.1.4/deepseek-history/small-edit-results.md) | 18 cases, 360 edits, 378 states verified; host-native in-place edit plus LayerFS Commit medians around 1.5–1.7 ms for tested 2/32 KiB files versus roughly 12–13 ms for Git CLI | Preserve measured foreground behavior; the difference does not isolate CDC or engine cost |
| [Docker/FUSE control](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/docs/roadmap/0.1/0.1.4/deepseek-history/docker-edit-results.md) | Primary comparison: 12 histories, 240 edits, 252 states verified; 100 MiB binary, 4 KiB edits: SDK edit plus Commit median 4.893 ms versus Git CLI 2,859.517 ms | Localized checkpoint performance is a control to preserve, not a universal speed claim |

The matched Git control retains the selected file contents, names, executable
bits, and symlink targets; it does not retain LayerFS's full filesystem metadata.
Use this control for the selected states rather than equating the original
190.21 MB Git clone's 15,632 source commits with the selected LayerFS history.

Latency results use synthetic histories and different interfaces. The Docker
comparison places Git in a constrained container while LayerFS also uses the
macOS host SDK/Store. It is not an equal-compute, equal-durability, cold-cache,
or pure-engine comparison. Each case has one history, not independent repeated
histories. Ordinary Exec and SDK range edits remain separate operation surfaces.
The Docker campaign stopped during supplemental 100 MiB binary Git packing
(signal 9); the last ordinary-FUSE diagnostic was not run. The verified primary
comparison does not make the overall campaign complete or establish an OOM cause.

Raw result files and the [large host-native edit report](https://github.com/Ephemeral-AI-Lab/layerfs/blob/1c7c9235115d1b4f21bc2eae7af822552b7be3ed/docs/roadmap/0.1/0.1.4/deepseek-history/large-edit-results.md)
are linked by the individual reports. Machine-local Stores and logs remain
machine-local; their paths are not download links.

## Historical storage research

[Issue #18](https://github.com/Ephemeral-AI-Lab/layerfs/issues/18) preserves an older
physical-pack experiment: 542,909,962 canonical bytes and a 562,513,789-byte
conservative object-storage lower bound, compared with a historical primary
Store median of 662,831,104 bytes. All 422,085 indexed segments were reread and
matched. Production publication, recovery, migration, and cleanup were not
implemented and fully counted. This is prior research, not a selected v0.1.4
design, a transferred footprint prediction, or a current acceptance target.

## Follow-up source-size analysis

Read-only Git tree analysis reproduced the earlier small-file count using the
same 157 checkpoints and pinned tip
`b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`. It used `git ls-tree -r -l -z`
for each selected SHA, deduplicated regular-file content by Git blob ID, and
validated every checkpoint's file count and total blob bytes against the frozen
manifest. Symlinks are excluded from the regular-file distribution; installed
dependencies and `.git` are not part of this population.

- Unique regular contents: 75,922; 891,893,067 logical bytes.
- Below 8 KiB: 48,976 contents (64.51%); 141,595,086 bytes (15.88%).
- 8–64 KiB: 33.49% of contents and 56.99% of unique payload bytes.
- Final snapshot: 9,404 regular paths; 7,118 below 8 KiB (75.69%), accounting
  for 29.20% of regular-file bytes.
- Adjacent selected-checkpoint modifications include 2,479 crossings from below
  8 KiB to at least 8 KiB and 1,834 in the reverse direction. These are sampled
  checkpoint transitions, not every intervening Git edit or agent tool call.

Manifest SHA-256:
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`.
This analysis establishes source size/count distributions, not compressibility,
SQLite allocation, object access frequencies, or row-versus-pack performance.

Machine-local artifacts (not downloadable):
`/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data/size-analysis/`
contains `analyze.py`, `results.json`, and `report.md`. The independent earlier
“Measure deepseek-harness directory” task counted an installed working directory
with dependencies; its 76,011 files must not be substituted for this population.

## Planning consequence

[v0.1.4](README.md) prioritizes storage efficiency while preserving operation
behavior and retained-state correctness. The broader multi-history campaign
moves to [v0.1.5](../0.1.5/README.md). Acceptance of the proposed architecture and definition of new evaluation
contracts remain for a separate discussion.

## Architecture review disposition — no new measurements

The [architecture v3](storage-architecture-spec.md) and [format](sqlite-storage-format.md)
respond to the independent review of PR #77; [finding closure](review-disposition.md)
is a document/source assessment against product revision
`28177560c8f049c02192e18c263cdc5543c1ab52`, not a new measured candidate.
The historical table, report links, candidate identities, limitations and source
analysis above are unchanged. [PR #80's development-smoke plan](storage-efficiency-boundary.md#development-smokes-and-qualification)
subsequently records the first five frozen checkpoints, edit and small-file cases,
and host SQLite/SDK plus managed Docker daemon/real FUSE topology. Full-family
qualification remains open; no new runs or samples are produced by this revision.

An arithmetic implication of the existing numbers is that 799,638,421 canonical
object bytes alone are about 14.19 times matched delta-packed Git's 56,373,248
allocated bytes. Even eliminating every noncanonical byte would not close that
gap. This is an intentionally impossible placement-only lower bound, not an
estimate of the new design. Packing alone is insufficient for the stated Git
objective; compression and similarity must make an incremental contribution.

The matched Git control allowed deeper delta search, but supplies no resulting
chain-depth distribution proving depth-one anchors sufficient or insufficient.
Bounded online hints, partial synchronous groups, full anchors and the remaining
canonical graph/index cost can leave a material gap. No confidence label for the
revised prose changes those empirical uncertainties. A future compact footprint
must be achieved before the corresponding operation succeeds, with all required
work and allocation counted.

## Issue87 full157 retained-storage diagnosis (2026-09-08)

[Sealed report/index and analysis tools](issue87-analysis/README.md) reconcile
all157 acknowledgements plus Init, authenticate producer/verification custody,
and independently review physical allocation, exact roles and retention.
Final acknowledgement335,552,512 allocated bytes remains primary; the census is
explicitly post-verification. Payload representation dominates; all selected
objects are logically retained and physical-base-only/unselected residue is zero.
**ADDITIONAL DIAGNOSTIC REQUIRED:** one separately authorized byte-weighted
predecessor-coverage funnel diagnostic, with unchanged product policy. #87 stays
open. No replay, optimization, M5/S3 work, release claim or PR merge occurred.
Historical Git/LayerFS controls remain historical and retain metadata/timing
qualifications. Full inventories remain in the external runs-root directory
named in the sealed index; compact reports and hashes are committed.

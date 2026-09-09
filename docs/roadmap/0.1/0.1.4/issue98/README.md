# Issue #98: Workspace admission and spill readback

**Status: terminal qualification PASS; retained implementation ready for review.** Follow-up to the accepted v0.1.4 preparation (#97), requested by the owner after completing the full Torch `.venv` correctness proof. The [original plan](plan.md) and all failed attempts remain recorded.

## Measured problem and repair

The unchanged control previously spent about 89% of its 7.299-second diagnostic Commit in content construction/admission and subsequent object admission. Checkpoint installation was only 0.220 seconds. The run inserted 164,150 objects, reused three, and committed 1,324 SQL admission transactions.

1. Workspace admission now uses the existing bounded SQL coalescer, closes its cohort before staging, and preserves rollback of pending and earlier committed batches. Physical batches and strict <8,192-object/<4-MiB transaction bounds are unchanged. Init's comparison cache remains disabled on Workspace.
2. A short profile of the remaining late-Commit work located 1,431 of 1,816 sampled `admit_remaining` stacks in spill `read_exact`/`File::read`. Graph order can jump around a spool; the existing up-to-896-KiB read-ahead repeatedly copies large windows across those seeks. Ordered spill visitation now caps that buffer at the existing 64-KiB I/O bound. Sequential ID scanning retains its buffer. No frame order, hints, authentication, codec or file identity changes. Sampled stacks do not measure physical disk bytes or supply a complete elapsed-time decomposition.

## Adjacent observations

| Pair | Control Commit | Candidate Commit | Result |
|---|---:|---:|---|
| Coalescing, control first | 6.865396 s | 6.373749 s | 7.2% lower |
| Coalescing, reversed order | 6.711595 s | 6.263154 s | 6.7% lower |
| Smaller spill read-ahead, coalescing control | 6.265651 s | 4.194146 s | 33.1% lower |
| Both changes against original control | 6.671061 s | 4.152412 s | 37.8% lower |

The combined pair's Exec + Commit is 12.667237 → 10.178917 seconds (19.6% lower). Host CPU is 8.249865 → 5.714634 seconds. Peak RSS is 126,861,312 → 132,300,800 bytes (+5.1875 MiB). Both Stores allocate 218,107,904 bytes; logical sizes are 214,097,920 and 214,122,496 bytes. These focused samples use uncontrolled caches, fresh Stores and the same frozen input/image; no historical failed-workflow timing is a denominator.

All eight pair members passed exhaustive independent content/metadata readback and cleanup. The selected profile run also passed full readback. The fixture has 16,395 files, 581,658,413 regular-file bytes, 1,283 subdirectories and three symlinks. No optional optimization remains planned before qualification.

## Coverage and evidence

The new Workspace test covers coalesced bounds, exact collisions, rollback of both pending and committed objects while retaining baseline data, and a clean transaction handoff to staging. Existing selected-object spill-order coverage now crosses the read-ahead window; focused spill/authentication checks pass. Earlier compile and test-fixture failures are retained in [evidence](evidence/).

[Paired results](paired-results.json) and raw per-arm commands, hashes, stdout and oracle receipts are included. During these host-only Store experiments the prior sealed daemon/FUSE image is reused because runtime sources/protocols remain unchanged. Final source/host/image sealing, complete native checks, full benchmark and full157 storage/history qualification follow before delivery. Original raw Stores and binaries remain under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-workspace-admission-runs`.

## Independent review and strengthened boundaries

A read-only independent review found no actionable correctness defect. Its coverage suggestions were incorporated into the Workspace cohort test: a concurrent reader authenticates an object from the still-open cohort, a subsequent duplicate is validated without another insertion, a deliberate equal-length collision targets that pending object, and a one-shot COMMIT veto restores baseline objects and leaves no Workspace stage. The strengthened test and Store test-target warning-denying Clippy pass.

The full native suite (411 passes), explicit ignored large-spill run, doctests, formatting and workspace warning-denying Clippy passed at `211bc420f`. Only the strengthened test body and this evidence changed afterward; production source is identical. The changed test was rerun rather than repeating already-passing unrelated suites. Final source/binary/image and terminal campaign receipts follow separately.

## Final terminal qualification

- All 198 performance executions and 226 routine proofs passed. The optional 600-second endurance proof remains explicitly unrun.
- Native suite: 411 passes, plus the explicitly run ignored large-spill check; doctests, formatting and warning-denying Clippy passed. The final strengthened transaction test and test-target Clippy also passed.
- Matched-image small-files and all four SDK/FUSE frequent-edit variants passed execution and verification.
- Full157: all 157 performance states, 157 retained-history proofs, 158 checkpoint/accounting records, authentication/dependency validation and cleanup passed.
- Final allocated storage: **184,582,144 B**, identical to the prior qualified candidate and **15.374% below** the 218,116,096 B supplemental control. Logical database size is 176,152,576 B; no sidecar allocation or unexplained page residual. Native representations remain 58,306 PREFIX and 28,106 FULL.

### Final .venv confirmation

| Sample | Commit | Exec + Commit | Full byte/metadata proof |
|---|---:|---:|---|
| 1 | 4.643631 s | 10.891184 s | PASS |
| 2 | 4.578398 s | 10.777544 s | PASS |
| 3 | 4.620881 s | 10.774716 s | PASS |

These final observations are separate from the earlier adjacent pair; the 37.8% paired improvement is not a guaranteed latency. All final samples preserve the original limits, healthy presentation, clean End, zero active sessions/executions, no swap/OOM, and container removal. The first confirmation wrapper stopped before workload launch because Cargo built the standalone probe into its manifest target directory while the wrapper expected the repository target. That setup failure and input-custody receipt are preserved; a fresh retry used the archived byte-identical built binary. No product changes or reruns of passing campaigns were needed.

### Remaining performance limits

The unmodified comparison reporter remains **INCOMPLETE solely for the four historical Git fixture comparisons**. Current elapsed classifications are **49 SEVERE, 95 REVIEW, 31 OBSERVED_INCREASE, 19 NO_INCREASE and 4 INELIGIBLE**. The unrelated-history 500 target and Git 100/500 absolute targets remain missed. Historical reports keep their original classifications.

Full157 case wall times are **481.976 s performance** and **613.815 s verification**, versus previous historical observations of **447.247 s** and **545.827 s**. These are unpaired observations, not evidence of a whole-history speedup or a controlled estimate of regression caused by this patch. The retained benefit is established by the adjacent `.venv` comparisons; a universal speedup is not claimed.

## Final identities and review evidence

- Qualified source: `9cfb4be477116646258ea0621280ed13b1824c6d`. Subsequent edits are reporting/artifact preparation only.
- Host benchmark SHA-256: `47b4d44e3e961f3b6a57d2c18dc6d195973133dbdfb87681a55f1b8e07abe3a2`.
- Runtime image: `sha256:42eb806fcfe9db1dc28098eb423340de3e4eaf81fd487389128305e65606d347`.
- [Terminal qualification](evidence/terminal/terminal-qualification.json), [final observations](final-results.json), [complete terminal evidence manifest](terminal-evidence-manifest.json), and [raw evidence references](evidence/terminal/raw-references.json). All 198 raw performance receipts and routine verification receipts are packaged losslessly; large files use gzip.

The generation binding was materialized after performance collection and is explicitly labeled post-collection. It binds the already frozen source/build/campaign/registry/seed/limits from the pre-collection terminal declaration; neither observations nor benchmark rules were changed. The initial unused full157 schedule retained an old human-readable Issue95 label from reused tooling; the correctly labeled replacement schedule was frozen before full157 execution. Both are preserved.

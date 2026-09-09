# Issue #98: Workspace admission and spill readback

Status: retained candidate frozen; terminal qualification pending. Follow-up to the accepted v0.1.4 preparation (#97), requested by the owner after completing the full Torch `.venv` correctness proof. The [original plan](plan.md) and all failed attempts remain recorded.

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

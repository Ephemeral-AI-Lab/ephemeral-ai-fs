# v0.1.4 regression and repair ledger

This is an active ledger, not a qualification decision. G0 evidence is retained
under raw/generations/g0. No execution PASS waives the frozen severity gates.

| ID | Evidence / classification | Root cause or next causal step | Repair / verification status |
|---|---|---|---|
| I01 | G0 inventory initial lock refusal, no workload began | Coordinator mistakenly held outer measurement lock while runner --list acquired it; existing runner correctly refused | Removed redundant outer lock; unchanged command retried once, live inventory198/29 matched exactly; both logs retained |
| R01 | G0 payload-create500m total3068249542→4051680834ns; Commit1941935000→2930548000ns; hostCPU2361745792→3612498126ns; allocation637599744→536875008B | Same canonical workload; Exec flat, queue0, no dependency reconstruction. NativeFULL26995calls701507931ns, each allocated/cleared1MiB. Transactions127→1800, output admission357560177→1557849086ns | Bounded per-admission encoder reuse implemented, independent source review complete; execution pending. Residual transaction fragmentation remains separate R02 |
| R02 | Namespace100 Init8901875→24644250ns; namespace1000002603162083→13565385375ns. Same source fixture. Transactions2→18 and130→2519 | Eight producers/four-slab queue unchanged, admission single consumer. Validation reserve applied both to transaction and bounded reader waves; probable serial consumer bottleneck. Need measured SQL/codec/pipeline attribution. Large InitCPU broadly unchanged, CPU/wall6.95→1.33 | R1 first intervention; separately analyze bounded validation-wave budget after subtracting retained physical buffers. No memory limit raised or assertion waived |

All additional severe metric crossings from the full generator remain unresolved
until explicitly mapped to a causally tested repair and final affected receipts.
Canonical object-count differences alone do not establish changed workload:
ordinary Init derives internal inode seed from fresh LayerStackId; fixture seed1
pins input content, not allocation IDs. Compare semantic oracle/fixture separately.

The prospective repair schedule and unchanged limits are in
[repair-r1.md](../issue91-campaign/repair-r1.md). Final acceptance additionally
requires all ordinary independent verifiers, supplemental equal-retained-state
storage/additional cases, affected phase1 guarantees and independent review.

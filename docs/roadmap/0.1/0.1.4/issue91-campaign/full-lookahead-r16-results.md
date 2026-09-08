# R16 / G9 rejected lookahead treatment

Frozen experimental sourcec59d5c78d, clean isolated source-g9 and sealed
host/image. Four performance observations, all independent proofs and cleanup
PASS, but the R16 elapsed/benefit gate rejects the treatment. Revert its product
commit; retain its source, tests, counters and every raw receipt. No retry for
better numbers. Native packing and4KiB pages were preserved in the experiment.

| Case | G8 ns | G9 ns |
|---|---:|---:|
| namespace100 |27467834|22133000|
| payload100m |628719208|616496166|
| namespace100000 |5644902791|6976396208|
| payload500m |2727922667|2748062042|

Init worsened by1,331,493,417ns (~23.6%). Its528 worker runs computed9355frames,
consumed3191 and discarded6164. Worker/overlap time was62,657,042ns;
spawn14,308,357ns and join5,852,416ns. Conservative physical peak2,041,747bytes
stays below2MiB and actual frame-owned peak131,059bytes below128KiB.
Even total observed overlapping work is far too small to justify this larger
state/ownership mechanism. Discarded work further limits its useful fraction.
The observations do not prove all1.33s of regression is caused by worker overhead;
retain uncontrolled variation and differing producer schedules as limitations.
No causal attribution or statistical claim is needed to reject an unproven win.

Creation949runs computed4680frames, consumed4673, discarded7; observed overlap
82,908,967ns, spawn13,995,160ns, join4,351,847ns. Commit1,439,460,042ns versus
G8's1,450,348,292ns, while total worsened. Store allocation remains536875008bytes,
4096-byte pages, and admission spill readback remainszero. Keep streaming R15.

Three focused checks passed: exact serial/native pack equality and final-batch
consumption, resource/stale/hinted fallback, actual encoding inside publication
and joined rollback on failure. A separately retained serial-path counterfactual
(lookahead allowancezero) fails the actual-execution assertion; restored treatment
passes again. This counterfactual was run after initial prototype checks, not
misrepresented as an untouched historical pre-repair run.
Default full checks:95 Store unit passes/one ignored large-spill gate,8 Store
integration passes,60 Workspace unit passes,14 Workspace integration passes and
one Store compile-fail doctest. Product all-target/all-feature Clippy and benchmark
production-binary Clippy passed. A broader benchmark test-target Clippy invocation
found three preexisting test-only lints; failures retained without changing frozen
workload source just to silence them. Format check passed. Final R17 must still
run applicable resource/large-spill/phase1 gates; none is waived here.

Raw evidence: ../checkpoint-evidence/raw/diagnostics/g9-lookahead-rejected/.
Proceed to R17 direct native assembly with the G8 streaming path, without R16
sidecars, extra workers, metrics or state machine. Full157 and final affected
qualification remain necessary; no64KiB product treatment.

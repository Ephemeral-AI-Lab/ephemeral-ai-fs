# R15 / G8 streamed new-file results

Source548c3663c includes streaming93568b10c and delivery accounting548c3663c;
42beb5c0d only relocates existing tests to fix preexisting all-target Clippy.
R16's intervening commit is a prospective document, not product code.
The initially built93568b10c was superseded before samples after independent
review found missing streamed memory-delivery accounting; retain both builds.
Clean isolated source-g8a host/image seals and all receipts are retained under
../checkpoint-evidence/raw/diagnostics/g8-headline-01/. R15 check before failure,
passes, initial Clippy failure and correction are in r15-streaming-checks/.
The corrected direct check additionally proves memory-delivery bytes reconcile.

All four R6 seed1 performance observations, independent proofs and cleanups PASS;
eight receipt hashes match their ledgers. These remain diagnostic observations,
not population estimates or final qualification. Preserve unfavorable Init.

| Case | G7 ns | G8 ns |
|---|---:|---:|
| namespace100 |23720084|27467834|
| payload100m |748463708|628719208|
| namespace100000 |5312226875|5644902791|
| payload500m |3631767208|2727922667|

Published v0.1.3 remains2603162083ns for Init100000 and3068249542ns for creation500.
G8 creation is below that historical observation; Init remains severe/unqualified.
Streaming changes Workspace new-file delivery, not native Init's code path.
Do not attribute the unfavorable control movement to a proved cause.

| payload500 receipt scope | G7 | G8 |
|---|---:|---:|
| Commit ns |2437060292|1450348292|
| Exec ns |1176034250|1263373084|
| output admission ns |1206962111|1181630752|
| consumer idle ns |983754160|4203431|
| admission spill readback bytes |525954441|0|
| memory delivered bytes |not compared here|525955698|
| native FULL encode ns |387837750|389241633|
| admission transactions |1024|1024|
| after-product cumulative host write bytes |1057325056|526356480|
| after-product peak RSS bytes |63750144|59310080|
| acknowledged Store allocated bytes |536875008|536875008|

The large reduction in idle and readback with essentially unchanged native encode
and admission time supports the intended overlap/replay mechanism. Exec worsened
and remains in the total. Host CPU observations after-product sum to3.368603s G7
and2.540153s G8 before subtracting their separately retained before observations.
All physical page-size receipts remain4096. Init's database bytes remain550821888;
its G8 CPU/RSS observations worsened and are retained in raw receipts.

Correctness:60 Workspace unit tests;93 Store unit tests;8 Store integration tests;
one Store compile-fail doctest passed. The new regression proves delivery before
EOF, zero private spill, canonical identity equality, original slab bounds, and
late read error after committed admission batches followed by exact Store-count
rollback. Workspace check covers Preview privacy, old private-builder control,
abandonment/retry, empty input and historical content after later edits.
Formatting and all-target/all-feature warning-denying Clippy pass. No test or
four-cell proof substitutes for every affected final member and full157.

Storage caveat: streaming can change producer ordering and first duplicate hint
selection in mixed workloads. Same headline allocation and4KiB pages do not prove
full157 retained-state savings. Keep the original full157 allocation/census,
dependency authentication, complete historical proofs and independent-review gates.
Continue R16 only as a separately attributable treatment; never merge64KiB control.

# v0.1.5: ordinary storage without explicit compaction

On 2026-09-11 the owner removed explicit compaction from the product direction:
it is too slow and does not provide enough practical value for the intended
workloads. Existing ordinary deduplication and compression are already adequate
for the owner's use cases; an expensive separate rewrite is not worth requiring.
This is a product tradeoff decision, not a claim that compaction saves no bytes.
This decision supersedes the compaction requirement in the earlier issue103
integration plan.

The historical full157 run reduced allocation from **83,935,232 B to 55,476,224 B**,
but the explicit compaction API took **626.313062 seconds (about 10.4 minutes)**,
with **626.518567 seconds process wall**. That is a separate cost after ordinary
construction; it is not Commit latency. The [retained report](issue103/full157-integrated-results.md)
also records cold-read amplification. Preserve these results at their original
source/fixture identities; they are not a new measurement of the removal patch.

Storage and latency follow-ups must work through ordinary Init/Commit admission.
Do not reintroduce compaction as a background/async rewrite, mandatory maintenance,
or hidden benchmark preparation. The historical 66 MB compacted criterion is not
automatically an ordinary-write acceptance gate; any new numerical target must
be declared prospectively with its speed/resource tradeoffs.

Open coordination: [#107 storage](https://github.com/Ephemeral-AI-Lab/layerfs/issues/107),
[#100 storage tracking](https://github.com/Ephemeral-AI-Lab/layerfs/issues/100),
[#102 campaign](https://github.com/Ephemeral-AI-Lab/layerfs/issues/102), and
[#108 admission/publication cost](https://github.com/Ephemeral-AI-Lab/layerfs/issues/108).
Completed #103 and its evidence remain historical integration work.

The supported write architecture is Init or captured Workspace facts → shared
canonical construction → exact CAS → bounded physical admission → root publication.
Keep small whole-file content and FULL/DELTA encoding, large-file CDC/extents,
compact scoped namespaces, authenticated pooled metadata and metadata deltas.
Schema10 and normal durability/publication semantics do not change.

The local implementation removes `LayerStackStore::compact_into`, its SDK options/receipt exports, the
`layerfs-store-compact` binary and the benchmark compaction producer/flag.
Qualified host builds probe ordinary schema10 admission without running compaction.
The correctness-only live smoke reopens the ordinary Store; its new identity is
`ordinary-storage-integration-smoke-v1`, distinct from the historical smoke.

The local implementation retains authenticated LFCNT1/version107 decoding, whole-owner/native-slice
reconstruction and dependency traversal for already-compacted Stores. Encoding
helpers for that format are test-only. Do not reinterpret or delete old objects.
Historical reports, contracts, original results and evidence readers remain;
their compacted size claims do not describe current ordinary writes. No migration,
repacking, replacement optimizer or new benchmark qualification is introduced.

The retained correctness-only smoke has four Exec calls and two Created commits:
import small/large files and a hardlink; edit and Commit through the live owner;
reopen the same Store; read the old layer; fork the edited commit and write again;
reopen and read the new state. Require original SHA-256 bytes and hardlink counts,
zero compactions, successful session/container cleanup and unchanged 300-second
outer bound. It supplies no performance or release-admission claim. Historical
access fixture preparation keeps its original frozen contracts and custody checks.

Implementation consolidation under #118: removal is committed on main. Fresh
focused validation passed five old-format/authentication checks (7.73s complete
command) and two ordinary probe/script checks (14.03s including compilation).
The Store test build retained one pre-existing `unused_mut` warning in
`objects/admission/metadata_tests.rs:212`. Earlier validation recorded 140
passing tests and four pre-existing ignored tests. Live Docker/FUSE smoke is
tracked in the #118 execution ledger and is not claimed by these unit checks. Existing
latency misses remain open; no new speedup, storage result or release PASS is claimed.

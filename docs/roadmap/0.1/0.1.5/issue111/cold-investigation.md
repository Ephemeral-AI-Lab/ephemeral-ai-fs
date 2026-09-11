# Cold namespace-100000 investigation checkpoint (#111)

**Cold 2.7 s target remains OPEN; no additional product optimization retained.** The original optimized product reproduced the cold gap. One advisory-prefetch treatment was implemented, tested and rejected by its frozen paired gate. The owner then directed the investigation to [real LayerFS source data](real-source-results.md), which demonstrated 68.638% allocated-space reduction with the unchanged product. The proposed six-native-worker follow-up was not implemented or measured.

## Custody and protocol

Started at HEAD 6d42d0e3b (two documentation commits after the handoff's cc8025fcd), preserving all original compaction-removal dirty files. Qualified rebuild reproduced exactly the retained #109 host SHA256 `dbbf1259186c60122286bb2a0503d6dcccfd0a41f49eb82799b5c3a4d6e6a36b` (the handoff/issue body omitted one `c`; #109's result document contains the correct hash). Product seal `95e796f896c771b4386a509d9cc44fd3ee7e89972ade06d8d51fd3f86c35a3b4`, source seal `17334f5e6900bdaf73e1c49ccb9bc0ad7ec3b710eef3ef5efef9b9841c6975ba`. Full 100,000 files /1,000 data directories /500,000,000 bytes and fixture digest `6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e` unchanged.

Evidence root: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue111-evidence/cold-20260910T234639Z`. `protocol.md` froze samples, cache/read validation, alternating pairs, invalidity and retention before edits/timing. `protocol-cache-amendment.md` froze explicit invalidation before controlled trials; `treatment-prefetch.md` froze the sole implemented treatment. Runner-owned lock used; no resource-sensitive overlap. Original patch/file hashes and every build/sample/probe/exit retained. Docker's listed original image tag could not be inspected: first baseline attempt stopped before timing; the runner requalified the image and the one whole-cell replacement completed. One compile failed on the crate's unsafe lint; the corrected narrow FFI boundary compiled and passed 23 Init tests with 1 pre-existing ignored test.

Benchmark rules §§6/8 prohibit pooling cold/warm and require matching cache acquisition for pairs. The namespace spec explicitly calls fixture reuse warm/uncontrolled, not cold ingest. Cold is the absolute gate; warm is delta-only. No untimed warm-up was used for cold acceptance. The current runner hardcodes a `reused-first` label, so the actual process read counter is essential.

## Plain baseline and historical comparison

| Observation | Cache profile/acquisition | initialization_disk_read_bytes | layerstack_init_ns |
|---|---|---:|---:|
| #104 unoptimized (historical) | reused-first / historical first-use |676,016,128|4,840,907,708|
| #110 retained optimized (historical) | reused-first / historical first-use |676,442,112|3,759,134,000|
| #109 retained optimized S1 (historical) | reused-subsequent / warm block |32,768|2,742,296,084|
| #109 retained optimized S2 (historical) | reused-subsequent / warm block |4,096|2,778,672,334|
| New plain baseline | reused-first / fresh acquisition, no warm-up |763,719,680|3,656,208,250|

The #109 and #110 optimized host binaries are byte-identical and share fixture/canonical output:112,451 objects /513,026,835 canonical bytes /127 admission transactions. Their different cache profiles and read volumes support the OS-cache explanation, not a binary or fixture-size change. Historical warm median 2,760,484,209ns, range2,742,296,084–2,778,672,334ns (n2); new natural-cold n1. These are separate distributions, not a new paired speedup claim. New natural-cold allocation 520,372,224B, apparent 515,469,312B; page/layout allocation varies without canonical-content changes.

## Measured cold phase attribution

Existing `namespace-init-diagnostic` nonce clocks were sufficient for timing attribution; no new phase instrumentation was needed. A temporary nonce-only physical-counter print supported encoding inspection. Nonce results never enter plain acceptance.

| Phase | Fully invalidated cold nonce, ns | Scope |
|---|---:|---|
| Public Init |3,760,371,083|outer operation|
| Direct pipeline |3,038,730,708|producer/serial admission pipeline|
| Final root/inode tree |599,878,833|existing final-tree clock|
| Import remainder |73,309,001|prepare_import minus pipeline minus final-tree; discovery/setup/handoff/final-admission remainder|
| Outer remainder |48,452,541|public Init minus prepare_import|
| Consumer idle (nested) |1,045,406,548|inside pipeline; not additive|
| Pipeline SQL commit time (nested) |928,837,000|inside pipeline; not additive|

Cache:`reused-first-sample-uncontrolled`, explicit read-only shared mmap/touch/MS_SYNC|MS_INVALIDATE/unmap; actual reads873,615,360B. User CPU3,513,968,166ns; system CPU5,623,860,041ns. An earlier naturally colder nonce gave 4,257,528,208ns /917,831,680 read B, pipeline3,543,610,875ns and final tree648,837,375ns. It remains a separate acquisition profile.

Consumer idle contains both source-I/O waits and producer compute; it is not an exact I/O-only clock. This run did not obtain a validated current warm counterpart, so it does not claim an exact measured cold-minus-warm I/O split. Historical #109 warm phases remain differently acquired diagnostic context. Exact exposed-I/O attribution versus compute remains an open measurement limitation.

## Correction: the excess read bytes are not all Store reads

A separate locked read-only traversal of the input, with no LayerFS Store, measured **665,853,952 physical read bytes on each of two passes** while reading500,000,000 logical bytes. The input itself occupies **865,730,560 allocated bytes**, exactly its summed4KiB rounded file sizes. File/block granularity and source cache residency therefore explain much of the apparent read amplification. The two100MB anchor files can remain cached while many small-file pages do not. Do not assign676MB−500MB to SQLite by subtraction.

A fully invalidated operation's873,615,360 read B is close to the input's865,730,560 allocated B, with a7,884,800B residual. That residual is not an independently traced Store-only counter; OS metadata, Store and scratch activity can contribute. Intrinsic source reader counters prove500,000,000 requested content bytes,100,000 authoritative opens and205,102 read calls. No canonical object-segment spool rereads occurred; the nonce recorded8,181,000 inode-pair journal bytes written/read, and canonical-frame readback0. These logical counters and process physical reads use different byte bases.

## Rejected advisory-prefetch experiment

Native import advised the next16 regular files, up to128KiB each, on macOS, preserving four producers and ordinary authoritative reads. Both frontier blocks and recursive directory import used the shared product path. No benchmark-specific product branch. All format/authentication/publication rules preserved. Advisory opens/syscalls increased resource cost; their extra opens are not included in the old authoritative source-open counter.

Frozen plain order C,P,P,C, full invalidation before every member, n2/arm:

| Sample | Cache | Read B | Init ns | System CPU ns |
|---|---|---:|---:|---:|
| C1 |reused-first, invalidated|894,009,344|3,980,672,584|5,541,573,375|
| P1 |reused-first, invalidated|865,779,712|3,743,044,834|9,539,621,208|
| P2 |reused-first, invalidated|865,775,616|3,692,844,334|9,529,473,250|
| C2 |reused-first, invalidated|865,808,384|3,637,621,500|5,665,553,541|

C median 3,809,147,042ns, range 3,637,621,500–3,980,672,584; P median 3,717,944,584ns, range 3,692,844,334–3,743,044,834. Paired C−P differences+237,627,750ns and−55,222,834ns. Frozen margin max(5ms, C range)=343,051,084ns. **REJECT_INCONCLUSIVE**, with approximately +4 s system CPU. Every candidate cold observation misses2.7s. No sample rerun for a better number. The patch/binary/proofs are retained in evidence; the product patch was removed. Candidate product seal93b651dc5e9fecadefe45e35b7f35fd4284a6c1e59aad753f71ddfd233f991e0, source 0f5cc5e2efdab835b08a65b51ec853a72b566cef4b1736c28c039b57f8484d9d, binary 031cb21bc9c06451674f76a35989f8084e16d207336d2b3bb168ca9942c77a03, matching qualified image 0f5cc5e2efdab835.

Warm attempts were retained as invalid for warm comparison: simple subsequent nonce3,628,320,167ns with675,495,936 read B; descriptor-retaining preconditioning nonce3,654,413,500ns with608,669,696 read B. Both exceed the frozen1MB warm bound. No eligible new warm timing distribution (n0; median/range/paired differences unavailable), and no warm-based absolute claim. Observed Spotlight/interactive host activity reinforces the need for measured cache state rather than labels; no other user's process was stopped.

## Allocation and format

The unchanged retained product's #110 tier receipts provide the complete existing tier baseline; source content and the original Init harness remain unchanged by the later real-source entrypoint. These are historical n1 observations, not new cold qualifications of all tiers. The first three tiers actually reported 0 read bytes despite `reused-first` labels.

| Tier | Logical B | Canonical B | Apparent B | Allocated B |
|---|---:|---:|---:|---:|
|namespace-100-compact-v3|5,000,000|5,029,235|5,160,960|5,160,960|
|namespace-1000-compact-v3|20,000,000|20,188,375|20,434,944|20,434,944|
|namespace-10000|300,000,000|302,182,831|304,926,720|320,675,840|
|namespace-100000|500,000,000|513,026,835|515,371,008|516,861,952|

The inspected nonce Store has 98,998 unique small-content FULL records,0 small DELTAs,300,000,000 raw small-file B and301,310,582 Zstandard frame B. All pack blobs507,091,486B; Store apparent515,579,904B and allocated519,573,504B. Actual small pack version4 is schema10 compact framing of the earlier pack-v3 small-content design; native large content uses CDC/CAS and native packs. Shared prepare_small/prepare_native, zstd and schema10 feature flags remain active. Uncompacted does not mean uncompressed.

The per-file SHA256-seeded xoshiro fixture is pseudorandom and unique; Zstandard frame overhead exceeds its negligible compression savings. A much-smaller expectation applies to compressible/redundant corpora, confirmed by the separately verified real-source result linked above. `reused_objects=0` describes preexisting Store reuse, not every intra-input metadata/content duplicate; use the physical census for small-content representation claims.

## Remaining blocker and qualification

No evidenced additional speedup was retained. The fully cold pipeline and serial final-tree work still exceed2.7s; exact current warm/cold I/O attribution and a lower-overhead overlap strategy remain unresolved. Six-worker experiments were discussed but not executed after the owner's change to the real-source workload. Existing #110 all-four-tier proofs and unaffected shared-family evidence remain valid for their unchanged product/recorded scope; they are not a fresh admission of the rejected candidate. No release/tag/deployment, no unrelated optimization, and no change to the original compaction-removal work. Related #109/#110/#108/#106/#102/#104/#100/#107.

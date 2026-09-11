# Compact fingerprint metadata index: measured cold improvement, 2.7 s still open

Promoted the shared product change in **441be212e** after the diagnostic screen,
plain cold comparison, four registered Init tier proofs and shared Commit checks.
The final valid plain candidate median is **3.419642375 s**, range
**3.408577250–3.506457875 s**, versus contemporary control median **3.701592708 s**.
All three pairs improved:46.836333,293.015458,448.740041 ms. Median paired reduction
is 293.015458 ms; the difference of the two arm medians is 281.950333 ms. These are
different statistics and neither is a guarantee for every run.

The predeclared meaningful-gain condition passed: all pairs positive, median
paired reduction >=150 ms and at least two pairs >=150 ms. **Cold Init <=2.7 s did
not pass in any candidate run.** The residual gap from the candidate median is
719.642375 ms. #111 remains OPEN. No release, tag or deployment.

The earlier standalone 3.435659584 s observation is retained historical evidence,
not this experiment's paired control. Do not calculate progress by subtracting
measurements taken under different source/custody/host conditions. The first
current pair saved only 46.8 ms and the control range was 315.1 ms; retain that
variation rather than presenting a universal 293 ms saving.

## Change and measured cause

ValueIndex previously used the complete 73-byte inode value as a scratch SQLite
B-tree key. The new table uses a 64-bit fingerprint plus ordinal. Every candidate
match is resolved through the fully authenticated canonical value group and an
exact 73-byte comparison. Fingerprints never authorize equality. All ordinal rows
remain represented; sorting candidates and taking the earliest exact match
preserves the old retained-window first-winner rule, including collisions.

Globally deduplicating query fingerprints avoids rescanning a collision bucket
for each input/page. At most 131072 candidate ordinals are collected (512 KiB),
sorted and processed with one group buffer. Each selected group is authenticated
once per call. Query maps/vector/decoder reservation are bounded within the existing
index allowance. The existing 4 KiB pages,4 MiB scratch cache,32 MiB scratch file,
whole-group 131072-value eviction, transactions and rollback invalidation remain.
No new cache, schema change, benchmark-specific path, predecessor shortcut,
authentication removal, compression/DELTA relaxation or off-timer work.

This is a smaller index working set, not a universal sublinear algorithm. Sync
still replays newly visible groups, including O(history) on reopen. Extreme
collisions/duplicate physical history can scan/sort the bounded retained window;
there is no K-by-history repeated scan within a lookup. The extra candidate buffer
is bounded, not an increased SQLite cache allowance.

The nonce screen used n=2 per arm, C1,T1,T2,C2, with independent verified cold
acquisition before each run. All rows preserve 100000 files/500000000 logical bytes,
112451 canonical objects/513026835 canonical bytes and 100002 pooled values.
Nonce results cannot pass the absolute target.

|Diagnostic sample|Init s|Pipeline s|Final tree s|Import remainder s|Outer remainder s|Disk read B|Cache profile|
|---|---:|---:|---:|---:|---:|---:|---|
|control 1|3.684017334|2.951823667|0.643402417|0.055821249|0.032970001|875626496|reused-first-sample-uncontrolled / verified cold, nonce|
|control 2|3.570276875|2.892761708|0.584833125|0.057225876|0.035456166|874479616|reused-first-sample-uncontrolled / verified cold, nonce|
|candidate 1|3.311403333|2.809937667|0.473676209|0.008603124|0.019186333|865763328|reused-first-sample-uncontrolled / verified cold, nonce|
|candidate 2|3.329883750|2.824064167|0.472734250|0.008818875|0.024266458|871432192|reused-first-sample-uncontrolled / verified cold, nonce|

Screen whole-Init median:3.6271471045 s control versus 3.3206435415 s candidate;
paired reductions 372.614001/240.393125 ms. Metadata preparation median dropped
from 503.6151205 ms to 308.114663 ms. Final-tree median dropped 614.117771 ms to
473.2052295 ms. Pipeline median dropped 2922.2926875 ms to 2817.000917 ms.
Metadata preparation spans both phases: do not add it to their enclosing clocks.
The remainder changes are measured, but no finer causal split is claimed for them.

|Whole-Init diagnostic metric|Control runs|Candidate runs|
|---|---:|---:|
|Scratch maximum allocated page bytes|9371648 / 9240576|1798144 / 1753088|
|INSERT execution ms|217.765 / 184.343|76.191 / 73.156|
|Lookup execution ms|126.383 / 126.167|38.146 / 37.698|
|Sync cache misses|25629 / 24546|0 / 0|
|Sync cache spills|22279 / 21295|0 / 0|
|Lookup cache misses|18026 / 18373|0 / 0|
|Fingerprint candidates|0 / 0|73 / 74|
|Authenticated candidate groups|0 / 0|73 / 74|
|Fingerprint/query-set preparation ms|0.000 / 0.000|46.448 / 46.219|
|Candidate sort/authentication/matching ms|0.000 / 0.000|2.994 / 2.737|

SQLite cache misses/spills are cache events, not physical disk I/O. Their reduction
to zero with unchanged cache size supports the working-set explanation. Candidate
hashing and authenticated positive reads are included, not hidden. The point
measurements include instrumentation overhead; plain qualification below determines
the product result.

## Plain cold qualification

Predeclared n=3 per arm, order C1,T1,T2,C2,C3,T3. Every final row had a complete
fixture metadata audit,100000 content-checked files/125169 pages, zero resident
input pages, and a passing known-warm positive control. Acquisition was independent
before each run. The method is darwin-shared-mmap-invalidate-mincore-v 1: it certifies
observed source-data OS residency, not SSD/controller caches or cold filesystem
metadata. Read volume corroborates acquisition and never overrides it.

|Arm/run|Init s|fixture_cache_profile|initialization_disk_read_bytes|Acquisition|2.7 s gate|
|---|---:|---|---:|---|---|
|control 1|3.553294208|reused-first-sample-uncontrolled|873267200|VERIFIED_COLD,0 resident pages|TARGET_MISS|
|control 2|3.701592708|reused-first-sample-uncontrolled|876498944|VERIFIED_COLD,0 resident pages|TARGET_MISS|
|control 3|3.868382416|reused-first-sample-uncontrolled|875913216|VERIFIED_COLD,0 resident pages|TARGET_MISS|
|candidate 1|3.506457875|reused-first-sample-uncontrolled|875556864|VERIFIED_COLD,0 resident pages|TARGET_MISS|
|candidate 2|3.408577250|reused-first-sample-uncontrolled|874668032|VERIFIED_COLD,0 resident pages|TARGET_MISS|
|candidate 3|3.419642375|reused-first-sample-uncontrolled|874487808|VERIFIED_COLD,0 resident pages|TARGET_MISS|

|Pair|Control−candidate ms|Reduction %|Comparison eligibility|
|---|---:|---:|---|
|1|46.836333|1.318110|ELIGIBLE_COLD_PAIR|
|2|293.015458|7.915929|ELIGIBLE_COLD_PAIR|
|3|448.740041|11.600199|ELIGIBLE_COLD_PAIR|

The shared cold.compare independently rechecked fixture, operation, seed, harness,
environment, acquisition and observed arm order. The replacement analyzer also
required the expected canonical counts and fixture metadata audit. No warm row,
nonce row or incompatible pair contributes to these results.

SDK, SQLite, publication and spool ran on macOS. Linux containers supplied only
daemon/FUSE/workload with 2 CPUs/2 GiB. Host worker/compiler treatments remained.
All sensitive builds, tests and measurements were serialized under the runner's
measurement lock, with no child double-acquisition.

|Plain arm/run|User CPU s|System CPU s|Process lifetime peak RSS B|Post-Init apparent B|Post-Init allocated B|
|---|---:|---:|---:|---:|---:|
|control 1|3.425285|5.567191|83329024|515420160|520994816|
|control 2|3.444877|5.516768|83918848|515477504|524894208|
|control 3|3.441481|5.859140|83689472|515571712|517066752|
|candidate 1|3.346132|5.447033|80822272|515497984|522084352|
|candidate 2|3.316609|5.330442|82886656|515432448|525139968|
|candidate 3|3.351327|5.431184|80707584|515448832|529129472|

CPU sums can exceed wall time because Init has parallel work. Peak RSS is process
lifetime measurement, not a proof that every possible workload uses that amount.
No swap/OOM/resource gate failed. Source-storage allocation was 865730560 B in
every final cold acquisition. All final Stores retain the expected canonical
counts. Page allocation varies across fresh Stores; it is reported without
attributing every filesystem allocation difference to this change.

## Invalid first plain cell and the one replacement

The initial six plain attempts were not valid instances of the declared fixture.
I copied prepared fixtures with cp -cR, which preserved payload file metadata but
changed directory mtimes. The original directory timestamp was 1700000000000000000;
examples in the copies were 1789120080945726351 and 1789120104819303069. The initial
namespace 100 proof rejected mode/mtime. All six Stores had 116454 objects and
513393058 canonical bytes instead of 112451/513026835. Four also failed cold
residency/read corroboration. The remaining two passed the narrow cold guard but
still failed whole-fixture identity and are excluded from qualification.

|Initial invalid cell|Raw Init s|Resident input pages|Disk read B|Raw cold status|Overall treatment|
|---|---:|---:|---:|---|---|
|control 1|3.822227000|4921|816787456|INELIGIBLE|INVALID fixture metadata; excluded|
|control 2|3.764322166|5038|826998784|INELIGIBLE|INVALID fixture metadata; excluded|
|control 3|3.792674041|4696|860495872|INELIGIBLE|INVALID fixture metadata; excluded|
|candidate 1|3.731301042|0|873873408|TARGET_MISS|INVALID fixture metadata; excluded|
|candidate 2|3.710192459|0|872611840|TARGET_MISS|INVALID fixture metadata; excluded|
|candidate 3|3.680109000|1652|840331264|INELIGIBLE|INVALID fixture metadata; excluded|

All raw receipts, copied inputs and the failed verifier remain untouched. No
invalid row was relabeled. The initial pair-only contingency was not exercised.
Once whole-cell invalidity was established, the owner's original one affected-cell
replacement allowance was frozen in a separate append-only v 2 campaign at 65d297799
before any replacement timing. See
[fingerprint-invalid-cell-replacement.md](fingerprint-invalid-cell-replacement.md).
The entire six-run cell was replaced once in the same order, with the same exact
binaries/images and original immutable fixture. A full file/directory mode/mtime
preflight was added. No product source, target, cache-acquisition method or
improvement threshold changed. No further cell/pair/arm was retried.

This exposes a remaining harness-hardening item: content-only prepared manifests
and the current cold guard do not by themselves validate directory mtimes. The
replacement's explicit metadata preflight and canonical-count checks close that
gap for this evidence; the shared harness should acquire that protection separately.
No source-input mutation or speculative explanation of failed clone residency is
claimed. The directory metadata error is measured; its relation to the residency
failures is not established.

Other retained premeasurement errors: a diagnostic SQLite integer-type mismatch
was corrected before sampling; the first plain CLI rejected simultaneous seed and
inherited repetition before running; the 10000 verifier selector was corrected to
the actual registered namespace-10000 before that proof began. These produced no
valid timing rows and no selected faster-arm retry.

## Correctness and storage format

Both diagnostic and plain Store library suites passed 134 tests with 4 preexisting
ignored tests. Focused tests force fingerprint collisions, verify earliest duplicate
ordinal selection, low parameter limits/global query dedup, actual 131176-value
whole-group eviction, and corruption/missing-group rejection. Existing admission,
metadata, rollback, reopen, small-candidate and schema suites passed.

The exact measured candidate binary and matching image passed independent
verify-selected.py proofs for all four registered Init tiers. Coverage is import
counts, fresh reopen/root equality and sampled FUSE content/metadata:6/10/10/10
sampled files, respectively. These are not exhaustive 100000-file byte proofs.
The failed copied-fixture proof is retained separately from these corrected proofs.

Additional public SDK correctness used the same external harness source and
identical dependency locks for control/candidate. Host Materialize placement,
100 identical 4096-byte files with fixed modes/times; public Init/fork, retained
range edit/Commit, no-edit UpToDate, repeated edit, revert, Store reopen and a
second-file Commit. Each arm passed 8 complete 100-file/409600-byte canonical
checks. This is shared-caller correctness, not a timing or FUSE claim. The exact
fs-benchmark binary's FUSE proof is separate above.

For both arms, stage pooled-value counts were 2,1,0,1,0,1 and selected encoded
bytes 1930,530,0,530,0,518 for Init/first edit/no edit/second edit/revert/reopened
second-file edit. DELTA counts also matched. Revert and no-edit added no physical
values. The harness retains its raw logs/binaries/Stores; canonical-root Debug
strings in some log lines are plain text rather than valid JSON, so the summary
uses the valid metric records plus successful assertions and PASS terminal record.

The product change is only the disposable metadata lookup index. Init continues
through the same shared admission/encoding pipeline as Commit: small content
uses pack v 3 with zstd/FULL/DELTA policy; large content uses CDC/CAS; metadata
pool groups remain authenticated. Ordinary Store schema 10 format probes passed.
Uncompacted does not mean uncompressed. No VACUUM, repack, GC or explicit
compaction was performed.

|Registered Init tier|Logical B|Canonical objects|Canonical B|Apparent B after Init|Allocated B after Init|
|---|---:|---:|---:|---:|---:|
|namespace-100-compact-v 3|5000000|365|5029235|5152768|5152768|
|namespace-1000-compact-v 3|20000000|2023|20188375|20434944|20434944|
|namespace-10000|300000000|25158|302182831|304922624|312479744|
|namespace-100000 / run 1|500000000|112451|513026835|515497984|522084352|
|namespace-100000 / run 2|500000000|112451|513026835|515432448|525139968|
|namespace-100000 / run 3|500000000|112451|513026835|515448832|529129472|

The smaller tiers used one full registered sample for size/coverage; their
uncontrolled cache timings are not cold-gate evidence. The 100000 rows are the
three valid cold candidate samples. This fixture's per-file pseudorandom unique
payload is largely incompressible and non-deduplicable, so 500 MB of input does
not become a much smaller Store. Metadata/header compression still exists;
"compression approximately zero" is a payload observation, not removal of zstd.
The newly smaller scratch index is temporary and is not the 500 MB content Store.

## Seals, commits and remaining boundary

Original contract:347d4b297,
[fingerprint-index-contract.md](fingerprint-index-contract.md).
Whole-cell replacement contract:65d297799. Product commit:441be212e, only
objects/metadata.rs, admission/metadata_values.rs and metadata_fingerprint_tests.rs.
The original uncommitted compaction-removal patch remains byte-identical and was
not committed or attributed to this change.

Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-fingerprint-index-evidence/20260911T092737Z`.
Includes every attempt, frozen scripts/hashes, source patches/manifests, fixture
metadata audit, bad copies, metadata preflights, raw receipts, diagnostics,
independent proofs, public Commit smoke, build logs/exits and summaries.

Shared fixture digest:
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.
Shared workload digest:
`c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b`.
Rust 1.85.1, SQLite 3.51.0, schema 10. Diagnostic arms are kept separate from plain.

control:

- Source `97889730b41da5cf03e07ff1ae9738af133f31bc3b39264e0702b079ecd0914f`
- Product `a9e7c50262e7af5939a50fb81f6873109bfb5257885b062a2aee572f82a65ad3`
- Binary `a57123c0124ad31230f004efb30b3da0b9cbeef14e83a5c512db2f9a44ca94f1`

candidate:

- Source `e741f560331d7bcc69497bb24495997e5a16aeb52377c439590578b60940b922`
- Product `da497512823bd27ac605f8ae3b76d57d0a0a9f4ed6e95b30a6da9721e6219598`
- Binary `8952e59cedd9fa876421db84131807f354f72b98dec84268f4f924e3bd22dced`

plain-control:

- Source `424b91fd418b1ef2da5b5a063c446c7f8d7b26b6c1abb16283fbba4196375006`
- Product `44f8226e21ff2b342ed88c17f0d47c278a60fad80e35e8562ae5bc9693528073`
- Binary `4828ab76408c82e48bd39885421c3580a33e12f13c3777991e77b945ab01c2eb`

plain-candidate:

- Source `0c8f75eee5091c11b90fd9cc0bcad1e31d27b2c4ff8538cffea8c685024e7198`
- Product `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`
- Binary `9510f483539b759c9bc138ecbed3da1da5d05f7b694cc6680c09c9633fbeb1e8`

promoted:

- Source `0c8f75eee5091c11b90fd9cc0bcad1e31d27b2c4ff8538cffea8c685024e7198`
- Product `760eb0f2093488a6a00c47eaaed51ac40e459bf90f45e2f99514598b8e665932`
- Binary `0051058ac8e9ffca19fee65e595c19a43abc64ad315536aa14abc2f7e6983b63`

The main host binary was rebuilt through the runner at 441be212e. Its source and
product seals exactly equal the measured plain candidate. Its binary hash differs
from the measured artifact; both are retained and distinguished. Timing tables
refer to the measured `9510f483…` binary, not an invented new timing of the rebuilt
`0051058a…` binary. The promoted image is rebuilt with matching source/product seals;
terminal proof outcome is recorded below after that build completes.

Decision: retain this measured partial improvement; stop further optimization in
this experiment. The remaining measured diagnostic costs include about 2.817 s
pipeline and 0.473 s final-tree work. They leave insufficient headroom for 2.7 s;
closing that gap needs a separately evidenced change, not a warm-cache target pass
or extrapolation of scratch-index savings. Related: #111/#115/#109/#110/#108/#106/
#102/#104/#100/#107.

Terminal main-binary verification: PASS, namespace-100000, 10 sampled files,
exact binary `0051058ac8e9ffca19fee65e595c19a43abc64ad315536aa14abc2f7e6983b63` and image
`sha256:4d151019f50e74204a029bbe4b20a0dcca853fcb170b6b9762a7b72f3bc9aba9`. Source/product seals match the measured candidate.

The measured plain candidate used immutable image
`sha256:ebb3f414eaf3b24f285de5f85d5d20d07e6eb8bd242137b8a8e4314d95e3ec53`;
the rebuilt main image above is retained separately by its immutable identity.

Issue outcome: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5632991211

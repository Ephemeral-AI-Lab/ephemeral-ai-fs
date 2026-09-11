# Guarded metadata proof experiment: large reopened-Commit gain, screen not promoted

The shared-code candidate avoided global metadata index reconstruction on every
measured reopened-Store Commit. First Commit fell from 249.700–250.323 ms to
12.651–13.006 ms. Both candidate first calls resolved 49 values from an authenticated
predecessor plus 1 provably fresh value, created no index and replayed zero values.
The unchanged control replayed 100002 values per first Commit.

**Decision: retain the evidence and reject promotion from this screen.** The frozen
numerical encoded-byte comparator failed for one retained-mode second Commit:
3975 B versus 3973 B in both controls. Independent pack inspection localized the 2 B
difference to an ordinary version 1 compressed group, not metadata. Metadata pack
size, added values and DELTA records were unchanged. This is not evidence of a
material metadata storage regression, but the frozen checker is stricter than
that prose criterion; its failure remains visible and has not been waived.
Retained-mode candidate latencies were also slower in both pairs, although they
did not meet the frozen two-pair beyond-control-range regression condition.

This was a bounded nonce diagnostic experiment, not an Init acceptance campaign.
Reopened means dropping Client/Store and reconnecting the same SQLite file; it
says nothing about OS page-cache coldness. Every measured Commit read 0 process
disk bytes. Cold plain Init <= 2.7 s remains OPEN. No release, tag, deployment or
product promotion follows from these measurements.

## Protocol and custody

Contract committed before implementation at `f06866b6f`:
[metadata-proof-experiment-contract.md](metadata-proof-experiment-contract.md).
Full namespace-100000 fixture: 100,000 files, 1,000 data directories, 500,000,000 logical
bytes. Each cell created a fresh Store via public Init, forked a branch, created
one public FUSE workspace, performed a public SDK 10-byte edit/Commit, then a
second marker/edit/Commit. Init and workspace/edit timers are separate from
Commit. No work was moved out of the public Commit operation.

Two arms, two Store-lifetime modes, n=2/arm/mode. Exact order in each mode:
C1, T1, T2, C2; retained mode preceded reopened. All 8 cells/16 Created commits
completed once. No performance retries, replacements, exclusions or cache
acquisition attempts. Every row is qualification_eligible=false. OS cache is
uncontrolled; the immutable fixture recipe/digest is shared. Source seals were
checked before and after every cell. Host/image builds used shared runner; tests
and collection held its measurement lock once, without overlapping sensitive work.
SQLite/SDK/publication/spool stayed on macOS; Docker daemon/FUSE used 2 CPUs/2 GiB.
No page/cache-size/worker/compiler-policy change.

Evidence root:
`/Users/yifanxu/Ephemeral-AI-Lab/layerfs-metadata-proof-evidence/20260911T085037Z`.
Includes protocol pointer, original dirty snapshot, control/candidate worktrees,
source patches/manifests, qualified identities/build logs, frozen collector and
analyzer hashes, all commands/logs/exits, physical checks and summary.json.
Only admission.rs, admission/metadata_values.rs and admission/metadata_tests.rs
differ between arms. Telemetry and harness bytes are identical. Main product and
the original compaction-removal work were preserved; the candidate remains isolated.

## Timings

Milliseconds. Positive paired reduction means control minus candidate. Each row
has n=2 per arm; no cold/warm pooling or acceptance improvement headline.

|Store lifetime / Commit|Control samples|Candidate samples|Control median (range)|Candidate median (range)|Paired reductions|
|---|---:|---:|---:|---:|---:|
|retained / commit-1|23.770, 33.392|33.431, 34.662|28.581 (23.770, 33.392)|34.047 (33.431, 34.662)|-9.661, -1.270|
|retained / commit-2|6.232, 8.710|10.135, 8.754|7.471 (6.232, 8.710)|9.445 (8.754, 10.135)|-3.903, -0.044|
|reopened / commit-1|250.323, 249.700|12.650, 13.006|250.011 (249.700, 250.323)|12.828 (12.650, 13.006)|237.672, 236.694|
|reopened / commit-2|9.255, 7.411|8.320, 7.083|8.333 (7.411, 9.255)|7.702 (7.083, 8.320)|0.936, 0.328|

Both reopened first-call reductions (237.672 ms and 236.694 ms) exceeded the control
range of 0.623 ms. The retained first-call candidate increases were 9.661 ms and
1.270 ms against 9.623 ms control variation; second-call increases 3.903 ms and 0.044 ms
against 2.478 ms control variation. This small screen does not establish retained
noninferiority. Do not hide those slower rows behind the reopened gain.

## Measured mechanism and resources

First Commit values below. CPU is user+system; RSS is the post-call process sample,
not peak memory. All first and second Commit disk-read receipts are 0 B.

|Arm/mode/run|Index sync ms|Replayed values|Proof ms|Proof positive/new|Metadata prepare ms|CPU ms|RSS bytes|
|---|---:|---:|---:|---:|---:|---:|---:|
|control/retained/1|10.681|2090|0.000000|0/0|10.929|19.716|88375296|
|control/retained/2|18.773|4218|0.000000|0/0|18.992|27.247|84688896|
|candidate/retained/1|15.682|2685|0.000458|0/0|15.969|24.874|85721088|
|candidate/retained/2|19.507|4218|0.000459|0/0|19.752|28.486|89538560|
|control/reopened/1|233.159|100002|0.000000|0/0|234.172|241.848|72056832|
|control/reopened/2|234.494|100002|0.000000|0/0|235.113|236.462|72679424|
|candidate/reopened/1|0.000|0|0.126208|49/1|0.047|8.041|62816256|
|candidate/reopened/2|0.000|0|0.126375|49/1|0.050|8.019|64700416|

Control reopened synchronization alone cost 233.159/234.494 ms. Candidate proof
cost 0.126/0.126 ms, followed by 0.047/0.050 ms metadata assignment/encoding preparation.
These clocks are disjoint: proof_ns is outside metadata_prepare_ns. The control
remainder after sync was 17.163/15.206 ms; candidate full first Commit 12.651/13.006 ms.
Thus this is measured elimination of history replay, not a cache-label inference.
First-call CPU fell from 241.848/236.462 ms to 8.041/8.019 ms. First-call RSS fell from
72.1/72.7 MB to 62.8/64.7 MB; no peak or universal memory-safety claim follows from RSS.
Second reopened candidate calls also had zero sync and 49 positive/1 fresh proof.
Retained calls did not enter the proof because their Store already held the index;
the early guard cost roughly 0.46 us, while tail replay varied 2090–4218 values.

The successful path is bounded by one <=100-row changed leaf, <=200 exact
indexed dependency probes, an authenticated bounded predecessor chain and one
scan of bounded prepared objects. It avoids history-sized reconstruction. Unknown
values and unsupported shapes still use the original O(history) fallback; this is
not a universal sublinear restart guarantee. No quadratic cross-product scan was
introduced.

## Implementation and correctness

The proof requires an absent ValueIndex, a single leaf with an origin, no earlier
prepared/published metadata groups in the held admission session, and sufficient
existing physical/read budget. Complete 73-byte values match authenticated prior
ordinals; header, length and inode correspondence are checked. A fresh value must
reference either an object published after the session baseline or an absent
object already authenticated and prepared in this admission. Missing lookup alone
never proves absence. Any unknown retains exact index fallback.

The predecessor is passed to existing DELTA search, including unavailable outcomes,
without a second fetch or reset of its per-target allowance. Retained bytes and
vector/map associations are charged; fetching reserves the existing 1 MiB read
allowance plus 256 KiB conservative proof state inside the existing 2 MiB physical
budget. Cached state is released after ordinary preparation. Authentication,
4 KiB pages,4 MiB scratch cache,32 MiB scratch file,131072-entry retention, schema 10,
pack/compression and existing FULL/DELTA/CDC limits remain.

Full Store library suite:140 passed,0 failed,4 preexisting ignored. All 10 new focused
tests passed again on the final counter layout. Coverage: fresh pending/published
dependencies, old-value reversion, prior session metadata, rollback, unavailable/
corrupt/unpooled origins, duplicate ordinals, actual 131200-value catalogue/index
eviction, physical reservation fallback, and cached-read budget preservation.
The eviction test observed one real index eviction and 100 old-value misses before
successful authenticated predecessor reuse without new pool values.

All 16 public commits were Created. Each cell checked complete edited-file content
after each Commit, visible head, final reopened root and complete edited-file
content after reopen. These are full edited-file checks, not exhaustive verification
of all 100000 files. Broader shared-caller/plain qualification has not run for this
candidate. Existing #104 evidence is not relabeled as this candidate's proof.

One initial new-test compile failed because its call omitted the diagnostic
find_batch Store argument; corrected before sampling. An archival byte-identity
check found counter field ordering differed; candidate telemetry was made exactly
identical to control before final focused tests/builds. One exploratory physical
attribution output incorrectly interpreted native version 4 framing; it was retained
and replaced by a separately named corrected output. None affected timing runs.

## Storage and physical format

Each of 16 commits admitted exactly 1 metadata value in 1 pool group, selected 3 FULL and
3 DELTA objects overall, and had 0 memory-budget skips. Independent read-only pack
inspection found exactly 2 canonical metadata DELTA records per final Store, each
65 B; both per-Commit metadata packs were 224 B. The checker verifies physical
framing and backward base chronology; full edited-file/reopen proofs above provide
separate canonical checks.

|Arm/mode/run|Commit 1 selected encoded B|Commit 2 selected encoded B|After 2 commits apparent B|After close allocated B|
|---|---:|---:|---:|---:|
|control/retained/1|3975|3973|515506176|523628544|
|control/retained/2|3976|3973|515481600|530350080|
|candidate/retained/1|3976|3975|515522560|522735616|
|candidate/retained/2|3976|3973|515477504|527831040|
|control/reopened/1|3977|3974|515526656|515616768|
|control/reopened/2|3975|3973|515477504|529055744|
|candidate/reopened/1|3977|3974|515473408|528855040|
|candidate/reopened/2|3976|3974|515543040|518176768|

These file sizes are after two Commits, not Init-only allocations. Fresh Store
placement, scope-derived canonical bytes and concurrent Init admission ordering
vary across cells; apparent/allocation totals alone do not isolate this treatment.
The strict pre-sample analyzer required candidate encoded bytes <=control maximum
plus control range. Retained second Commit control 3973/3973 B versus candidate
3975/3973 B failed that comparator. Corrected pack attribution isolates the 2 B to an
ordinary version 1 group's compressed size 3740 B versus 3738 B, with the same 4036 B
decoded length. The metadata proof was not attempted in this retained cell.
The exact cause of those ordinary compressed-byte differences was not proven;
do not claim that the fast path caused a metadata storage regression or that
randomness conclusively explains it. Do not change the frozen gate after seeing it.

Authenticated prior ordinals may differ from the old index's retained first winner.
The duplicate/eviction tests exercise that explicit physical placement treatment;
canonical identity is exact, but universal byte-identical encoding is not claimed.
Uncompacted remains distinct from uncompressed: pack/zstd and bounded DELTA remain.

## Exact seals

Shared fixture digest:
`6fc793a9703bd0a21066f9fb12622c3451b16bd6ad7ef8b7382351351ac80a7e`.
Shared workload digest:
`c6f1e4b15fce502ee1c08bd875e758beb099d3398394831faeca507c4b4e579b`.
Schema 10 / linked SQLite 3.51.0 / Rust 1.85.1. Both source snapshots at `f06866b6f`
plus preserved dirty work and identical existing nonce diagnostics.

control:

- Product `863b9430f0e4d11064a2a0fdad8ef20067f13982ed0518d6a0f0a539e0ec3892`
- Source `5be3f78f516b180b662a841a9b9fccbb22a2859e7732be68ae0bd806aa02ddb9`
- Binary `4842e9297c32dcf4aecf3855eb11df1e257eb9a417b5b5b8f46aab7cf07e758d`
- Image `layerfs-bench-infra:5be3f78f516b180b`

candidate:

- Product `0ebdeb79cb2d214496cb10cb3a5b233e62fc8488e13cc1551ffffb4d2a7c3423`
- Source `58368fa02866adacdd3e60b67e9b1cc5bb7ad651b2d1b10bf08ce6940b6a31f3`
- Binary `5e64dcf6bbfb48f0364421ced1a137aa81db37c249a10879ac49040361646032`
- Image `layerfs-bench-infra:58368fa02866adac`

## Outcome and remaining work

Mechanism supported; clean screen promotion rejected. Keep this isolated candidate
and all attempts. No second optimization was stacked and no selected rerun was
performed. Any further qualification requires a new frozen protocol that resolves
ordinary-byte variability and retained-mode uncertainty before promotion; it must
preserve this screen's failure and slower rows.

This experiment addresses small Commit after Store reopen. Cold Init has no such
predecessor and retains its separate metadata preparation cost. Cold is the absolute
Init gate; uncontrolled/warm observations are diagnostic or matched deltas only,
never <= 2.7 s acceptance. #111 remains open.

Context: #111/#115/#109/#110/#108/#106/#102/#104/#100/#107.

Issue outcome: https://github.com/Ephemeral-AI-Lab/layerfs/issues/111#issuecomment-5632258478

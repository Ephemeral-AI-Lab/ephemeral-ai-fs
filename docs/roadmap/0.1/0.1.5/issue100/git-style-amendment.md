# Git-style optimization: prospective diagnostic and selection contract

2026-09-10. The owner said to proceed with Git-style candidate selection and an
identical-base matcher comparison. The ten-state target stays45,000,000 allocated
bytes;45–46million is near-target, not exact achievement. Full157 is prohibited
until a same-Store-verified ten-state candidate is stable near that target. The
premature full157 run is incomplete diagnostic evidence only.

## Fixed identical-base diagnostic

Use the five sealed pairs in external `git-matcher-study/pairs.json`: translation
step5→6, Chinese catalog7→8, icons8→9, proposed→implemented rename ledger8→9,
and both-mode-turn→skill-load tool schemas at step3. These are original fixture
bytes with earlier or same-snapshot raw bases, all strictly below131072B.
The133273→129991 CDC boundary remains separately recorded and ineligible here.
No pair/candidate/codec sweep. Compile Git2.47.1's original delta matcher and
replayer only in an external GPL-labelled diagnostic directory. No GPL source
is copied into LayerFS product code or relabelled MIT.

For each identical base/target, compare (1) current raw-prefix Zstandard frame,
(2) Git COPY/INSERT program compressed using the SAME pinned SmallContent
Zstandard3/window18/flags, and (3) FULL target. Retain the unchanged physical
base cost once on each side. Charge9B FULL or41B prefix-delta record framing,
16B group directory, and a conservative additional4B decoded-program length
for the hypothetical COPY/INSERT record. Pack headers/SQL/index/allocation are
excluded and labelled; these diagnostic costs are not achieved Store allocation.
Require exact prefix roundtrip and exact Git program replay against original
bytes, before and after program compression. Record actual index/program sizes,
tracked matcher allocation/CPU/wall and codec timers separately, with cold/cache
and debug/optimized compilation limitations. Do not infer public-call speed.

No new Store interpretation is admitted by this diagnostic. If COPY/INSERT
encoding materially wins, its integration requires an explicit additional
record kind/capability fence, program bounds, bounded reader, authentication,
retention and rollback amendment before changed-product measurement. Existing
canonical identity, static2MiB reconstruction/3MiB encoding allowances and
pinned Zstandard settings remain fixed. Matcher/index ownership must end before
the2MiB static encoder starts; bounded program/base/target/frame ownership must
fit the remaining1MiB, with no hidden allocation fallback.

## Candidate direction to evaluate after the diagnostic

Most remaining FULL frame bytes (29,601,395B) belong to later-snapshot new paths;
that does not mean no similar content exists. Investigate bounded available
candidates across paths, including the already-owned physical admission batch,
using Git's larger-base-first idea and a fixed shortlist. No global history
index, offline later-snapshot encoding or repacker. A candidate base is a real
retained file object, never an artificial duplicate FULL.

In-flight FULL selection must handle same-session lateCAS: exclusive session
ownership alone does not freeze the selected representation. If another batch
selects a proposed base first, a dependent must fall back successfully to its
retained prepared FULL; it must not name an unlocated private FULL or assume the
winning base has the same depth. Any chosen implementation must prospectively
specify shortlist selection, actual buffer/index charges, output grouping and
late-base fallback before encoding or measuring that changed candidate.

## Selected product candidate: bounded content-keyed FULL reuse

The identical-base diagnostic found current prefix frames smaller in four of five
pairs; Git COPY/INSERT won only49B on icons. For tool schemas the same available
base reduces the current6402B FULL record+directory to85B with our existing codec
(Git program88B). Implement candidate discovery, not a new program format.

Use a **session-local1024-slot direct-mapped fingerprint cache**, reserving
**128KiB from the existing16MiB admission index budget**. It contains only IDs
and eight64-bit fingerprints of actual selected SmallContent FULL winners from
this AdmissionSession. No raw cache, persisted/global index, past-session search,
namespace scan, artificial base or in-flight candidate. This differs from the
rejected recent128-entry long-line ring: content-derived lookup addresses at most
eight slots directly and does not require a candidate in the last128 publications.
The fixed1024-slot policy is not a size/window sweep.

Fingerprint the eight smallest distinct mixed polynomial hashes of all16-byte
windows, using a rolling257-base hash modulo2^64. This is bounded generic byte
matching, including shifted/binary content, not a fixture-path or line heuristic.
Use fixed arrays; no allocation proportional to file size for fingerprinting.
At most eight cache probes and at most eight candidate IDs; rank by shared
fingerprint count, require two matches, and use ObjectId order to break ties.
Pick one candidate; no trial of additional candidates after an uneconomical delta.

Preserve real predecessor priority. Only when no eligible immediate predecessor
exists, query this cache. Release its mutex before SQL/codec work, resolve the
candidate's selected location and authenticate its FULL bytes once. Require the
returned FULL ID to equal the selected ID; a stale/different-role entry is not
permission to silently choose a different base. Exclude the target's own ID.
Exact CAS remains first, and the usual late-CAS comparison still handles targets
which another same-session batch selected after the initial membership check.

Register cache entries only for actual FULL winners after publication, after
releasing the Store connection; compute fingerprints outside both Store and cache
locks. Losing prepared FULLs never enter it. Skip final-batch registration. Cache
lifetime is the session; failed sessions cannot prepare new objects, and their
cache is discarded with rollback/drop. Existing epoch/cohort/retention rules hold.

Emit existing kind1 for a cached selected-FULL base; no new persisted grammar or
schema is needed. Schema8 retains its old policy; this candidate runs only under
schema9. FULL-vs-DELTA cost still includes32B reference; FULL wins ties. Decoder
and encoder never overlap, and existing2MiB/3MiB per-active allowances remain.
The cache uses index ownership, not additional codec buffers. Missing/corrupt
persisted dependencies remain integrity errors. No per-object transport or SQL
transaction is added; optional selected-base acquisition uses the existing reader.

Focused checks must cover shifted fingerprints, an unrelated negative, bounds,
actual selected-FULL reuse under one session, late target reuse, and private
rollback. Then build affected artifacts and run one new ten-state performance,
frozen census, exact same-Store verification and cleanup with actual outcomes.
Compare the whole Store and speed/resources to all three controls and chain-1.
Do not run full157 unless a verified stable result is near45MB.

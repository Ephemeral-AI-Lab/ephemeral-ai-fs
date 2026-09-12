# Small/large representation experiments: findings

All 30 performance samples and 10 independent verification replays passed.
Performance exercised 81 public SDK edits and 72 Created commits. Verification
checked all 34 retained versions after reopening each proof Store, with complete
byte comparison and new FUSE workspaces; 82 FUSE SHA-256 checks covered live,
post-Commit and historical reads. Performance and verification have identical
source/product/host-image identities. No product code was changed.

Worktree: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-transition-experiments`.
Branch: `codex/small-large-transition-experiments`.
Product revision: `f8bb6dcb676f73a14746357d98489b0691398c26`.
Measured source seal: `0d0cec0b34cd648c0ed38e53bd7bf25bb1445a7c33b90a5612b5c772f08b27e6`.
Product seal: `f8db9e4ea64708ff211332096d4bb31b5d2a20b3582728ecd35fd39eecaabe25`.
Image tag: `layerfs-bench-infra:0d0cec0b34cd648c`; immutable identity is in each
run's identity.json. Source changes are confined to the experimental harness.
This PR publishes an archived copy of that harness plus evidence, without
registering it in the benchmark executable or importing the 98 development
commits ahead of the report branch's GitHub-main base. The measurements apply
to the product revision above; they do not qualify this report PR's base.

## What the observations establish

1. **Large-file range edits retain locality.** The 128 KiB, 129 KiB and 16 MiB
   one-byte replacements each passed exactly one byte through CDC, in every
   repetition. Median public edit+Commit was 7.223, 7.067 and 8.216 ms respectively.
   This verifies chunker input locality for these operations; it is not a count
   of all hashing, metadata processing or physical I/O.
2. **A size crossing performs conversion work.** Every 127->129 KiB grow passed
   all 132,096 final bytes through CDC. The first grow took 8.466 ms median
   edit+Commit. Later identical grows reused an earlier content root and admitted
   zero new FULL/DELTA objects, but still processed 132,096 CDC bytes. Exact-CAS
   storage reuse did not remove reconstruction work.
3. **Shrinking restores small representation and can reuse old content.** Every
   129->127 KiB truncate selected SmallContent and reused the original small
   content root. Every historical version remained readable after reconnect.
   Growing does not destroy the old small representation.
4. **One Commit resolves multiple live crossings once.** The sequence
   127->130->126->140->120 KiB used four public edits and one Created Commit,
   finished as SmallContent, and had zero CDC input at that Commit. Median
   edit+Commit was 12.639 ms. This is a different operation count from the
   six-Commit oscillation experiment; their sums are not a paired speedup.
5. **Large-to-small truncation stayed inexpensive at both tested source sizes.**
   Shrinking 1 MiB and 16 MiB to 80 KiB took median edit+Commit of 8.016 and
   8.300 ms, respectively, and both produced SmallContent with zero CDC input.
   The timed run did not inventory every native decode byte, so these timings
   alone do not prove an exact total read bound. The existing transition code
   reads retained ranges rather than reconstructing the discarded suffix.
6. **Bounded delta history incurs repeated decode work and resets.** In the
   ten-edit 64 KiB case, decoded-group bytes per Commit rose through 66,380,
   131,943, 197,502, 263,061, 328,620, 394,179 and 459,738. The seventh changed
   Commit selected four FULL records and zero DELTA records; the next Commit's
   decoded-group work dropped to 66,384 bytes. Counts include metadata, so they
   are not file-only record counts. The repeatable pattern is consistent with
   the bounded predecessor closure returning to FULL storage. It does not prove
   monotonically increasing latency; measured times do not show that.

## Assessment of optimality

The architecture behaves as described, including boundary-conversion costs.
This experiment does not establish an optimal cutoff. The 127 KiB small-file
one-byte edit had an 8.312 ms median edit+Commit, while 129 KiB chunked content
had 7.067 ms. This is an observation across different file sizes, not a
controlled treatment that changes only representation, and does not establish
a better cutoff. File sizes, affected offsets, and representations differ.
Small-file metadata/storage benefits,
compressible data, random reads, POSIX full saves, concurrent loads and alternate
cutoffs were not compared.

The next targeted optimization, if required, is avoiding repeated small/large
conversion when retained states already exist, or measuring separate grow/shrink
thresholds. Neither was implemented. Keep exact CAS, authentication, publication
and historical reads intact; no global optimizer or compactor was added.

## Evidence and timing scope

- [Complete timing table](results.md), [derived JSON](summary.json), and
  [reproducible analyzer](../analyze.py).
- [Prospective experiment scope](../README.md).
- [Complete performance receipts](../evidence/performance/summary.json) and
  [source/binary/image identity](../evidence/performance/identity.json).
- [Complete verification receipts](../evidence/verification/summary.json) and
  [source/binary/image identity](../evidence/verification/identity.json).
  These are byte-identical copies of the original run summaries and identities,
  preserving every sample's parsed raw records, timings, execution command,
  container resource observations and cleanup result. All sample stderr logs
  were empty. [Copy provenance and original SHA-256 values](../evidence/provenance.json)
  are included; runtime capability files, sample Stores and redundant log copies
  are excluded. Local originals remain under
  `benchmark-results/host-store/transition-experiments/{performance-1,verification-1}`
  in the measured worktree.
- [Focused self-check](../self-check-1.log): one pass.
- [First host build](../build-host-1.log): 47.69 s Cargo build; final host
  build 12.09 s Cargo / 14.949 s command, [log](../build-host-3.log).
- [Docker build](../build-image-1.log): 66.768 s command; matching Linux build
  cache reused. Workload self-check remains part of the image qualification.
- [Refused overlapping build](../build-host-2.log): another owner held the
  global measurement lock; no compilation or measurement occurred in that attempt.
- Performance collection: 40.239 s; independent verification: 15.362 s.
  Each includes fresh runtime setup and cleanup. These exclude build commands.

macOS owns SDK/SQLite/Commit/spool; Docker owns Linux daemon/FUSE and read-only
proof commands. Every container was validated at 2 CPUs, 2 GiB RAM, no swap,
256 PIDs, /dev/fuse and SYS_ADMIN, without shared data mounts. No host swap or
container OOM/swap occurred. Every experiment container and successful sample
Store/input was removed. Existing nonexperiment containers were untouched.

Fresh Store/process/container is not verified cold OS storage. Representation
inspection happens outside call timers and can warm later steps in a history.
The mutation surface is the public SDK, not ordinary POSIX writes. Proof FUSE
reads are separate from performance distributions. All results are exploratory,
`admission_eligible=false`, with only three performance repetitions per case.

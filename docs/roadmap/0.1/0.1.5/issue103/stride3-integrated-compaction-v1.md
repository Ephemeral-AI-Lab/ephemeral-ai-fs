# Issue103 integrated stride3 + compaction v1

Prospective contract, frozen before runner implementation and sample collection.
Issue: https://github.com/Ephemeral-AI-Lab/layerfs/issues/103 . User revised scope:
full promoted-method integration and **stride3 only**; no full157 or broad #102
campaign. Existing raw histories and historical-access-v2 fixtures remain intact.

## Claim and immutable workload

This optional scenario is `deepseek-stride3-integrated-compaction-v1`, selected
with repository_history profile `stride-3` and explicit `--storage-compact`.
It inherits `issue100/stride3-snapshot-contract.md` and all host/Linux ownership,
public Exec/save/Commit, oracle, resource and cleanup requirements. Retain exactly
53 original states at 1,4,7,...,157 from the original sealed fixture. Use its
existing direct transitions; no commits or state constructions for skipped
snapshots. Do not alter original fixture generation, manifests or oracles.

The claim is the complete final allocated storage of the integrated product,
with all 53 original content/metadata states verified through that same measured
Store. Compare against the recorded offline 54,382,592-byte and historical Git53
49,332,224-byte references. Report decimal MB and exact byte differences; these
are historical storage comparisons, not fresh paired speed measurements.
No full157/66-MB qualification or release claim is supported by this scenario.
A size miss is retained and explained; no rerun of an unchanged candidate.

## Operations, custody and order

1. Finish focused product tests. Build source-isolated release host and Linux
   daemon/FUSE/workload binaries through the existing runner measurement lock.
   Preserve every replaced binary and its identity. Probe actual linked schema
   and integrated namespace/pool/content-format behavior from the built host.
2. Run a correctness-only live Exec/FUSE integration smoke, using generated small
   and 256-KiB content plus a hardlink: initialize, Exec/write, Commit, end,
   compact via the public API, reopen, historical/fork reads and ordinary writes.
   It is `issue103-integration-smoke-v1`, not a repository-history storage sample.
3. Run the unchanged selected 53-state performance workload. Keep all actual
   Created/UpToDate/presentation outcomes, phase times, receipts and resources.
4. After coordinator/container cleanup, while still holding the runner-owned
   measurement lock, invoke exactly one public
   `LayerStackStore::compact_into(destination, CompactionOptions)` call from the
   qualified host binary. The source remains the pre-compaction evidence. Write
   the final Store into its own `compacted-store/` directory; never substitute a
   Python codec, converter, Git matcher, read-only adapter or wrapper mutation.
5. Freeze the final Store's path, SHA-256, inode/device, apparent length and full
   allocated directory bytes before independent verification. Record all files
   in that directory; fail on unexpected required sidecars/temporary files.
6. Reopen the exact measured Store for all 53 original oracles, passing its path
   explicitly to the existing public verification session. Compare before/after
   identity and allocation. Verification may create ordinary verification forks;
   retained canonical content is verified against the frozen pre-verification
   identity, with any verification-only SQL growth reported separately. Before
   opening it, retain a byte-identical closed image under `frozen-measured-store/`
   outside the measured directory. This preserves the measured preimage after
   verification forks; historical_access uses that identical frozen image as its
   master and binds the same pre-verification SHA-256.
7. Run the eleven mapped historical-access cases below, preserving their original
   15-second entry-through-receipts-and-teardown limits and verification rules.

The existing measurement lock is acquired by the runner, never by a child a
second time. macOS owns SQLite, coordinator, canonical construction/publication,
compaction, and physical spool. Docker owns only daemon/FUSE/workloads. No release,
tag, deployment, routine cache pruning or unrelated cleanup.

## Compaction timing and resources

Operation contract: `public-store-compact-into-v1`; surface is the public Store
API. The host process command is `storage-compact SOURCE DESTINATION LIMIT`.
Its operation timer brackets the one API call; process wall time separately
includes launch, parsing and receipt output. Intrinsic authentication and the
product's complete original-object/logical-record verification remain inside
compaction timing. Independent benchmark fixture/oracle verification stays out.

Retain the complete product receipt, phase CPU/RSS/footprint/I/O observations,
`/usr/bin/time -l` process resource output, periodic host RSS and owned temporary
file allocation samples, and process-open-file observations for unlinked SQLite
scratch. Label sampling maxima as sampled, not exact RSS peaks. Record command,
PID, timestamps, exit status, all publication/cleanup outcomes and known temporary
allocation, including the private source copy and lookup index. Use the product
4,294,967,296-byte default temporary budget, host RSS <=8 GiB, no swaps/OOM, host
free >=50 GiB and inherited run-owned disk limits. The complete phase limit is
14,400 seconds; keep save/Commit and compaction costs separate and also sum them.

A reported successful compaction requires published=true, cleanup_complete=true,
directory_synced=true, an empty publication-notes list and process exit zero.
The final allocation includes all indexes, metadata pools, physical bases,
native adapters and required sidecars. Source backups and transient work are
separate resource quantities; the compacted Store must be independently readable.
The original result/manifests remain preserved, and new compaction/freeze evidence
is bound into the performance manifest before verification starts.

## Historical-access profile and oracle binding

The new profile is `historical-access-stride3-integrated-v1`. Each case keeps the
operation, path, offset, length, cache policy, expected output count, full-file
metadata checks and 15-second end-to-end limit of its historical-access-v2 template.
All case IDs replace `-v2` with `-s3-v1`; the original template ID is recorded.
Old/head cases retain original indices 1 and 157. The two range-history cases map
65 -> 67 (retained ordinal 23), keeping pnpm-lock.yaml offset 537371/length 6421.
The historical metadata-path case maps 57 -> 58 (retained ordinal 20). These are
explicitly changed checkpoint samples, never reported as reads of omitted states.

The forward mappings are frozen from original fixture evidence: index64's
pnpm-lock.yaml is 541522 bytes, too short for the original range; index67 is
556697 bytes. The metadata path exists at index58 with size24108. Bind expected
content/range hashes and metadata to those original checkpoint oracles, not to
observed product output. Construct the fixture outside all access timers after
53-state verification, binding the frozen performance Store SHA and actual
retained commit IDs. Preserve source checkpoint IDs, source/oracle digests and
mapping fields. Do not add omitted snapshots to make an access case runnable.

Performance and verification each retain their normal separate receipts and
seals. Read-limit failures are reported independently of storage success. This
optional scope keeps admission_eligible=false; it does not complete #102 or a
release campaign.

# Accepted M4.5 — full frozen DeepSeek diagnostic

Status: owner-authorized and frozen before execution, 2026-09-08.
Parent evidence #72; accepted M4.5 #86 is completed. Continue draft #81.
No M5 or further product optimization is authorized.

## Candidate and scope

Start from accepted M4.5 at `1b45ce75deaef4c6775d5596d59f9d5b765d45d6`.
Product behavior is unchanged: Create4096, supported schema6 pages4096/65536,
wire1, level1 Zstandard, depth-one M4 DELTAs, canonical/CDC/COW/group/pack/batch/
memory limits and cache_size=-32768KiB. No new durability or acknowledgement policy.
The ordinary five-checkpoint smoke remains exactly five. A distinct `--deepseek-full`
selector reuses the current host smoke coordinator, importer, observers and verifier
with case identity `deepseek-full`, count157 and the original full-run bounds.
This is one diagnostic replay plus its independent historical verification, not a
new benchmark family or numerical acceptance campaign.

The original full infrastructure at documentation commit
`1c7c9235115d1b4f21bc2eae7af822552b7be3ed` established the input/operation contract.
Its historical binaries are not executed. Reuse already prepared immutable inputs
and authenticate their manifest, Git tree and changed blob/oracle seals using the
current shared reader. Manifest SHA256:
`03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271`;
source tip:`b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed`.
All157 entries in existing order, representing4,936,693,030 logical blob bytes.
No arbitrary commit selection, seed change, source program or Git hook execution.

## Topology, lifecycle and budgets

MacOS owns SQLite, SDK/coordinator, canonical construction/encoding/admission/
publication and physical spool. Managed Docker owns Linux daemon/live core/FUSE and
ordinary importer. Two CPUs,2GiB RAM,no swap,256PIDs,/dev/fuse and existing security
configuration; no data-sharing mounts or container Store. Record actual inspection.
One fresh empty Store, empty LayerStack, one Branch and live Workspace;157 ordinary
Exec imports and Commit attempts, no Add or SDK range-edit substitution. Preserve
whole-file replacement, deletion/type transitions, modes and normalized mtime.
Existing no-change handling records actual Created/UpToDate without inventing states.

Inherit original #72 safety bounds:300s per acknowledged step/command;4h performance
and4h independent verification;120s setup and120s cleanup. Immutable input validation
gets4h preparation bound, separate from product timing. Host/image builds retain
900s and2Cargo workers. Store<=16GiB;runtime/spool/staging<=16GiB;owned directory
<=32GiB;free host space>=50GiB before/between checkpoints;sampled host RSS<=8GiB.
OOM/swap, integrity/oracle/route failure or incomplete cleanup is a failure.
No timeout extension after observations, no parallel measurements/builds. Every
invocation holds the existing measurement lock. Existing uncontrolled OS caches,
no purge or cold-cache claim. Unrelated containers/worktrees are preserved.

## Observation and verification

Existing public timers enclose Exec with output drain, Commit with required
finalization, and separate Init/End where observed. Preparation/transfer/resource
sampling/allocation/census/verification stay outside operation timers. Existing
observer records complete allocated SQLite+sidecars at every acknowledgement,
logical size,page size/count/freelist,canonical counts/bytes,physical A/B/selected
encoding and FULL/DELTA/matching/base/group/read work plus CPU/RSS/I/O and spools.
Cumulative selected encoded bytes from receipts are not a complete durable pack
census; final read-only census reports all pack BLOBs, directories/records and
SQLite table/index/overflow/unused pages separately. No hidden/unselected bytes
are subtracted. Preserve full trajectory and partial failure receipts.

After freezing performance observations, reopen through the same host binary/image,
fork and mount every retained mapping, and compare independent original oracles for
all paths/types/content/modes/symlinks. Verification-created Branches change only
post-performance Store metadata. Preserve producer/verifier identities, all157
observed mappings, manifests and cleanup. No extra Git arm, malformed/race/crash
suite or M5 qualification. On failure preserve evidence and stop; a demonstrated
product correction requires new explicit candidate/build identity before rerun.

## Interpretation and reporting

Only after authenticating equal157-state historical scope compare complete
allocation with original LayerFS940,310,528 B and matched delta-packed Git56,373,248 B.
Controls are historical, not fresh timing pairs. Git retains selected content/paths/
executable bits/symlinks, not all LayerFS metadata; Git's later packing has a separate
cost. No Git parity, universal savings, aggregate load, durability or release claim.
A favorable trajectory does not prove allocation amplification's cause or bounds.

Retain new immutable artifacts under the existing external runs root, publish a
separate report/JSON and evidence-index entry, commit/push the existing branch/PR
without merging. Preserve all M2/M3/M4/M4.5 and #72 reports. Stop after this replay
and verification. No stronger codec, page sweep, migration, cloud or M5 work.

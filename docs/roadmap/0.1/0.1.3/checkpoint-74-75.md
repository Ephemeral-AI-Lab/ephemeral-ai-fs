# v0.1.3 checkpoint contract — issues #74 and #75

This contract implements the maintainer's September 7 checkpoint instruction.
It supersedes exhaustive routine-verification requirements for this campaign,
not the underlying filesystem semantics or historical result classifications.
Base: main 5c9cce92b446; host-store topology and public operation routes unchanged.

## Inventory and measurement

Use the normal host runner's admitted families, not every source folder. The
initial binary registry lists 232 definitions, including five capped-edit legacy
entries that the host runner does not admit. Those are duplicates of active
length-changing replacements, not five omitted performance obligations.
The admitted inventory is 198 performance cases plus 29 proof definitions:
28 reliability definitions and the CDC boundary proof. Of these, sustained-600s
is an optional long test, explicitly unexecuted in this checkpoint. A separately
named short sustained smoke may be added; it never replaces the long definition.
Freeze the final generated registry before the shared campaign.

Performance: one complete seed-1 sample (SDK repetition 1), clone preparation
where supported, fresh output for initialization. Product allowance 300 seconds,
outer 310 seconds, setup 600 seconds. Every case retains its original timer,
workload/fixture identity, operation counts and actual cold execution caches.
15-second and Git 500/1,000 ms goals remain reporting-only target classifications.
No before/after product speedup claim from a harness change. Old comparable rows
are descriptive references only. Keep all failed attempts without favorable reruns.

Verification: separate 45-second work/59-second hard invocation, preparation
reported separately. Aim below 15 seconds. Actual selected bytes, roots, paths,
snapshots and omissions are emitted. Schema/coverage marker: checkpoint-fast-v1.
A sampled PASS is not exhaustive historical/canonical-object qualification.

## Family review and intended routine coverage

| Family | Keep | Simplify / reuse / omit |
|---|---|---|
| init_namespace | sampled independent namespace/content, public initialization and reopen | reuse existing bounded verifier and protected native fixture |
| store_footprint | actual Store accounting and bounded edit-region proof | reuse bounded controls; no repeated exhaustive object census |
| edit_length_preserving | public SDK edit, changed region, witnesses, Commit/reopen, forbidden-write checks | existing bounded verifier; remove no useful assertions |
| edit_length_changing | size/offset/changed-region and SDK route | reuse bounded verifier, not duplicate capped legacy definitions |
| edit_canonical_chunk_count | exact expected chunk-count transition and edited bytes | existing bounded verifier, no whole-base scan |
| payload_create_read | actual create/read work, current content and reopened state | existing fast current-state proof; reusable source qualification |
| tiny_file_churn | changed entries/absence and witness, publication | existing mixed-range sampling; no full unchanged background walk |
| directory_construction_traversal | complete measured traversal/create counts and sampled resulting tree | existing mixed sampling; no repeat full payload scan in verifier |
| namespace_mutation | moved/deleted targets and unchanged witnesses | existing bounded sampled tree |
| workspace_change_locality | SDK/FS route, changed data, clean Commit, move/rewrite state | existing bounded sampling and no duplicate background validation |
| git_tool_workflow | full semantic head/tree/parent and precommit/reopen custody | preserve #73 repair; no generic sampled-only routing |
| mixed_load_bearing | episode outcomes, aliases and final state | reuse fast checker; no additional equivalent replay |
| dedup_cross_file | independent expected sharing transcripts/current bytes | reuse source qualification, no exhaustive Store census |
| dedup_cdc_locality | independent CDC transcript and boundary proof | existing bounded verification; no duplicate import qualification |
| dedup_workspace_reuse | additions, reuse/content identities, publication/reopen | existing fast references and selected changed bytes |
| dedup_branch_history | all Commit IDs/parent links, final head/reopen, selected exact content | 6–8 historical snapshots instead of all; no full-history object census |
| workspace_reliability | actual fault/contract exercised, published data, recovery and cleanup | smallest relevant fixtures/assertions; obsolete Busy oracles repaired |

## History selection and assurance

All workload Commits still execute. Validate all IDs and parent links separately
from content sampling. For depth 500 choose:
- distributed: 0,1,199,200,201,250,499,500;
- hotset: 0,1,7,8,9,250,499,500;
- metadata: 0,1,2,249,250,499,500;
- recurring: 0,1,2,3,499,500 (depth 100: 0,1,2,3,99,100).
Lower tiers may inspect every snapshot when there are at most eleven; larger
remaining histories use bounded early/middle/final selections. Mixed-v2 unrelated
retains its existing sampled-content and all-parent-link coverage.
Compare sampled recurring A/B file roots and metadata content preservation.
Changed content in an unselected intermediate snapshot can be missed; disclose it.
Do not rerun helper/container setup for every unselected historical state.

## Reliability repairs

Trace original errors and reproduce affected cases. Lease exclusion must still
exclude competing ownership; ordinary open writers/live commands use current
Commit-and-continue semantics, with explicit final data and cleanup assertions.
Fault proofs must actually reach the requested boundary. Prefer small direct
boundary fixtures over arbitrary large load; preserve existing product behavior.
Presentation failure checks one changed file plus witness, successful publication
exactly once, failure reporting, supported recovery, correct live reads, and an
UpToDate second Commit. Stage markers distinguish a blocked lifecycle operation
from expensive verification. Never hide a real recovery or cleanup failure.

## Shared infrastructure and final evidence

Reuse the current runner, fast verification and report helpers. One final frozen
campaign serves both issues. Selected development checks precede it. Build and
measure serially under the existing measurement lock; never overlap another task's
measurement. Keep host/container CPU/RSS scopes, no swap/OOM, master integrity,
independent samples and cleanup. No product cache prewarming in untimed setup.

Report Markdown/JSON/CSV with every admitted case and proof, sample count, timer,
phases (including six Git commands), setup/proof/cleanup wall, resources and
available backing/storage metrics; missing values carry a reason. Match identities
and reconcile inventory totals. Include target misses, historical comparisons only
when compatible, exact evidence links, optional long-test exclusion and omissions.
Required CI and relevant real Docker concurrency/coherence tests precede green-PR
merge and issue closure. This checkpoint is not a release or #70 optimization.

## Initial repair evidence

- The merged #73 image reproduced the presentation proof TIMEOUT. Its retained
  receipt shows Created + presentation_failed, reached PresentationResume fault,
  and successful canonical data verification before recovery failed. Recovery
  unconditionally paused an already-ended remote projection. It now pauses only
  when a projection handle remains; attach creates the fresh backing owner.
- The open-writer/live-command proofs expected Busy despite the live owner's
  supported Commit-and-continue path. They now require Created, committed data,
  continued execution/readability and an UpToDate subsequent Commit. The holder
  is released even if Commit fails, preventing assertion-induced cleanup leaks.
- An occupied daemon mount rejects a competing owner before a second Client's
  private manager can return the old WorkspaceBusy error. The proof requires
  rejection, checks that the original owner still works, and retains reacquisition.
- Current host backing APPEND bypassed file_io's test fault hooks. The hooks now
  exercise the actual host spool append and preserve native EIO/ENOSPC checks.
  Write versus fsync error timing follows the actual public path, not the retired
  deferred-proxy acknowledgement rule.
- Streaming canonical admission bypassed the old admission fault checkpoint and
  begins before final candidate publication. Activate the branch-scoped test
  fault at construction and checkpoint actual checked-page transactions. Require
  prior admission for the later-batch fault, but do not require an unrelated
  candidate spill for final-publication rollback.

Focused checks passed: history sample cycle/end selection, actual backing fault
routing/exact reservation consumption, and published-install recovery without a
second Commit. These local checks do not substitute for the pending Docker proofs.
The first attempted baseline proof and image build were refused because another
live history task held the shared measurement lock; neither is a product failure.

## Recovery smoke identity

The repaired original presentation proof passed in 2.128 seconds with its old
1 MiB fixture. To fulfill the explicitly requested minimal recovery contract,
replace only that active proof ID with
`workspace-published-presentation-failure-smoke-v3-proof`: genesis has one 4 KiB
unchanged witness under sentinels/ and empty work/a/; the operation creates one
4 KiB work/a/published.dat file. All canonical and reopened checks therefore cover
the two relevant files, without unrelated aliases, symlinks or filler bytes.
Family cardinality remains 28. Keep the original proof and timing as historical
repair evidence; never relabel it as the smaller smoke. Fixture/cache identity and
expected-state generation must use the new case and its exact 4/8 KiB states.

## First real Docker repair results

On image `layerfs-bench-infra:a847ad83ab8cea09`, separate development proofs passed:
presentation recovery 2.128 s, lease lifecycle 1.954 s, open-writer Commit 2.428 s,
live-command Commit 2.260 s, and later-admission retry 3.165 s; cleanup passed.
Final-publication injection reached its exact boundary (one fault, five earlier
transactions), but the verifier tried to Exec while a retained stage intentionally
excluded mutation. Move that read after the supported Commit retry; still check
unchanged prior publication before retry and exact current data afterward.
The corrected retry, two spool faults, and minimal smoke remain to be executed.

Collector fixes are independently checked: proof output must be absent at launch,
SDK row binding survives receipt reuse, and mismatched source/input/harness/image
receipts are rejected. The table generator rejects missing/duplicate evidence and
keeps target misses separate from execution PASS. Final runtime evidence is still
pending; these development results are not the complete checkpoint.

## Final repair findings before collection

All 27 routine reliability definitions passed in development (the 600-second
case remains excluded). Explicit Discard needed a product fix: a failed backing
owner rejects flush/FREEZE, so Discard now closes it without flushing or publishing
pending bytes. The spool proofs retain exact errno, unchanged published contents,
acknowledged backing-byte accounting, clean discard and fresh reopen.

The five history proofs now pass below 15 seconds: distributed500 9.137 s,
hotset500 9.121 s, metadata500 6.989 s, recurring100 3.664 s and recurring500
6.798 s. Sampling alone exposed another verifier defect: nested Slice/Concat
recipes recursively revalidated shared ancestry under repeated edits. Flat byte
oracles for the bounded small edited files preserve exact bytes and avoid that
exponential work. A shallow old/new parity check and deep-history regression pass.

The required live Docker suite exposed a genuine SDK/mmap race: a queued dirty
folio could replay old bytes over an SDK edit after the live view was installed.
During kernel-cache reconciliation, keep the installed SDK ranges in an owner-local
bounded view; merge those ranges into delayed folio writes, preserve unrelated
bytes, and prevent stale tails from regrowing a truncated file. Ordinary namespace
and size mutations stay excluded while folio callbacks drain. Release this view
before acknowledging the SDK operation. No cross-workspace lock or I/O-held mutex
is added. The unchanged failing Docker assertion passed three predeclared targeted
repetitions. Deterministic stale-folio, truncation-tail and gate-release tests pass.
The complete final live suite and final campaign remain required.

Collection freezes exact registry membership and source identity before executing.
The shared campaign cannot silently change inventory/source on resume. Report
comparisons retain historical scope and verification limits, including exclusion
of malformed old compact Git baselines. This is a checkpoint, not a product-speedup
claim from less verification work.

### SDK routine proof coverage v2: match the declared cold SDK setup

The first complete checkpoint campaign has 198 passing performance observations,
216 passing routine proofs, and ten failed length-changing SDK resource proofs.
The retained diagnostic `sdk-resource-diagnostic-r2` reproduces the 100 MiB
insertion failure: incremental cgroup memory 64,868,352 bytes exceeds 32 MiB;
container lifetime peak 100,147,200 bytes, zero swap/OOM, zero forbidden FUSE or
spool writes, and semantic/canonical/route checks pass. A pre-edit FUSE `stat`
causes LOOKUP to mark the inode potentially cached (required by stateless-open
semantics); a length-changing edit then refreshes the shifted suffix. The
performance worker never performs that pre-edit lookup. The proof was applying
cold SDK resource limits to an extra cache-coherence scenario.

Before requalification, declare coverage schema
`fs-bench-pro-sdk-edit-verification-v1-checkpoint-cold-v2` for the routine SDK
verifier. Remove its pre-edit FUSE `stat` and associated FUSE inode-number equality
assertion. Keep canonical inode identity/initial fixture size, exact qualified
roots/counts, independent changed-boundary bytes and unchanged adjacent witnesses,
post-commit FUSE size/content, payload retention, Store/Client reconnect,
publication/cleanup, one SDK edit and Commit, and ALL existing resource limits.
The one read-only FUSE execution occurs after resource sampling. Report that
pre-edit FUSE inode-number stability is omitted; this is not a warm-cache or mmap
resource qualification. Existing gated live SDK/mmap coherence tests retain their
separate coverage and must stay passing. No product or performance path changes.

Requalify all 56 cases routed through this changed SDK verifier once; preserve
all earlier proof receipts (including ten failures) and every original performance
sample. Bind replacements through an explicit manifest carrying old/new source
seals, the unchanged product/image/harness seals and identical registered recipe,
seed and prepared fixture. Input seals include source, so recompute the recipe
hash for the replacement source and record both instead of pretending equality.
Non-SDK proof and all performance execution paths are unchanged and retain their
source-bound results. This is a verifier-only repair of the shared campaign.

The first cold-v2 requalification passed all 56 SDK cases. Required CI then
reported a `format_in_format_args` lint in receipt serialization, after all
assertions and resource observations. Replace the nested `format!` with
`format_args!` (same receipt text), retain that first passing proof set and
manifest, and requalify all 56 proofs on the final source. Performance samples
and every product/resource/verification assertion remain unchanged.

# Issue #49 final performance pass: targeted NOFLUSH and backing rollover

Date: 2026-09-06. Three read-only reviews inspected the active restart implementation in `/Users/yifanxu/.codex/worktrees/1b4b/layerfs`, its retained receipts and Linux/fuser behavior. No new builds, tests, benchmarks or mounts were run by the reviewers. User direction: this is the **last optional performance-optimization scope**. Required correctness, supported-caller adoption, cleanup and final qualification remain obligations.

## Decision and ordering

Finish the active concurrent mapping/SDK-edit/Commit coherence correction first. Then consider A (targeted NOFLUSH) and, only if useful work remains, B (one combined backing-rollover exchange). Preserve the best correct implementation. Do not open another optimization campaign, worker/window-size sweep, cache/TTL change, storage-algorithm rewrite, additional connection, multi-window pipeline or scheduler framework merely to chase the preferred timing band.

Current measured create reference: Exec 0.5716 s, Commit 0.4046 s, complete 0.9946 s. Earlier daemon integration: 0.5718 s / 0.3638 s / 0.9533 s. Different development sources, one observation each; no statistical or isolated causal conclusion. Latest receipt records 1,000 FLUSH, 1,000 RELEASE, 1,099 WRITE callbacks, 273 backing calls and 0.1759 s cumulative exchange wait; host dispatch 0.0360 s, physical queue 0.0018 s, shared edit work 0.0016 s. Timings overlap. Exchange wait is not a removable budget or proof of exact savings.

Formal parent requirement remains same-source tier 100 create/delete complete lifecycles strictly <1.0000 s. Preferred create 0.7000–0.8000 s is an ambition, not another hard threshold. The current ~0.0054 s create margin is narrow and must be disclosed, but do not invent a mandatory headroom gate. Physical 100-workspace qualification remains deferred/inferred.

## A. Targeted NOFLUSH

**Feasibility:** high confidence in callback suppression; moderate confidence in useful net elapsed gain; conditional on error semantics.

Current `crates/layerfs-fuse/src/filesystem.rs::create` uses `FOPEN_DIRECT_IO` only. Its `flush` callback obtains admission, dispatches, obtains a gate, then replies OK. The existing created-handle class is the candidate for `FOPEN_DIRECT_IO | FOPEN_NOFLUSH`. Preserve ordinary-open KEEP_CACHE/mapping behavior, TTL, DefaultPermissions, capabilities, RELEASE/unpin, explicit FSYNC, Commit and End.

Linux 6.12.76 skips FLUSH for NOFLUSH when writeback cache is disabled. It does not skip RELEASE or explicit FSYNC. The retained #48 research used this same targeted class; no research patch migration is required.

**Do not call FLUSH completely behaviorless:** `LiveOwner::callback_gate` currently rejects a failed/closing owner, so this callback can expose an error at close. Before adoption, establish that close is not a required deferred-error boundary for this handle class and preserve required WRITE/FSYNC/Commit/End error reporting. RELEASE cannot replace close-error delivery. If retaining that FLUSH error boundary is required, skip this optional change; do not silently weaken it or add a new protocol solely to suppress callbacks.

Reuse focused created-handle contents/lifetime/error checks: close/reopen, explicit fsync, Commit, final RELEASE, retained backing on injected failure and error delivery at the declared boundary. Then one unchanged create-100 clone-backed sample, checking callback counts, complete lifecycle and CPU/resource/cleanup scope. Fewer callbacks without an elapsed benefit is a valid outcome, not a reason for favorable repeats.

## B. Combined backing-window rollover

**Feasibility:** implementable with existing HostSpool primitives and a single append window; performance magnitude unproven. Do not add a two-window payload pipeline.

```text
Current rollover when the next write does not fit:
  APPEND filled prefix -> wait
  CANCEL unused old tail, if any -> wait
  RESERVE next window -> wait
  copy next callback / apply existing prepared edit

Proposed rollover:
  reserve bounded allowance for the next window
  APPEND_CLOSE_RESERVE -> one response
      host appends old prefix
      host closes exact unused old tail
      host attempts next reservation
  decode old ACK + separately tagged next outcome
  copy next callback / apply existing prepared edit
```

Full rollovers can remove one serial request/reply turn; partial rollovers can remove two. This is removal of a named dependency, not merely releasing a mutex. Simply unlocking the window cannot speed a serial caller still awaiting the same exchanges.

### Exact changes and invariants

- `live_wire.rs`: proposed `APPEND_CLOSE_RESERVE` operation, reusing existing encoders. Validate all frame fields/ID/offset/append length/next desired and minimum sizes before physical mutation. Retain current frame limit.
- `live_backing.rs::BackingOwner::request`: reuse APPEND, CANCEL_RESERVATION and RESERVE through narrow shared helpers. At most one outstanding physical reservation remains. Consume exact old reservation, append its valid prefix, close exact unused tail, then attempt next reservation.
- `live_owner.rs::write_owned`: use the combined operation only at an actual rollover. Keep plain `flush_append` for fsync/freeze/teardown; these must not speculatively allocate another window.
- Keep `AppendWindow`, `BufferedFrame`, `PendingBytes` Filling/Sending/Backed ownership. Old data stays owned and charged until a valid ACK; old read references remain valid.
- `BackingConnection::call` remains serial with current cancellation/uncertain-stream behavior. No append replay or generic transport-error reinterpretation.
- `HostSpool::reserve_append`/`append` retain existing algorithms, physical high-water, failed-tail cleanup and segment lifetime.

**Critical response distinction:** old append success followed by next-reservation ENOSPC is not an overall failed append. Do not implement `append(...)?; reserve_next(...)?` as one undifferentiated Result. Return a successful old-prefix/tail-close acknowledgment plus a tagged next outcome: reserved exact range, or rejected with the existing error. A next-space failure must not undo the old write or make it replayable. Preserve existing exact-request fallback where appropriate; integrity and uncertain transport errors remain failures with retained ownership.

**Memory:** reserve next-window allowance while the old Sending frame is still charged. The combined frame needs up to 1 MiB + 37 bytes for the proposed extra lengths, within the existing 1 MiB + 64 KiB hard frame cap. Reserve header capacity before extending a vector; do not incur uncharged capacity doubling. Account old Sending bytes, next allowance, caller buffer and host/reply ownership simultaneously. This is one payload window plus future reserved capacity, not two independently filling/sending windows. If overlapping allowance is unavailable, use the existing sequential flush-then-reserve behavior; do not create a new NoSpace outcome merely because an optional optimization needs extra temporary allowance.

### Smallest validation

Extend the existing `append_consumes_only_its_exact_reservation` and owner continuation checks, rather than create another harness:

- Full and partial rollover retain exact bytes and create one valid next reservation.
- Wrong ID/offset, oversize append or malformed next size performs no append.
- Successful old append plus next ENOSPC acknowledges the old prefix without replay/owner corruption; quota fallback remains valid.
- Lost response/cancellation keeps exact old bytes/charges and never retries the append.
- Reads spanning old backed and new pending ranges work, including same-segment reuse.
- Fsync/freeze cancel unused tail without allocating a speculative next window.
- Direct and TCP backing semantics agree.

Then one source-identified unchanged create-100 clone sample. Retain a performance improvement only if complete lifecycle improves without equivalent cost shifted into Commit/End or changed resources. Otherwise reject/revert this performance hypothesis and finish required work.

## Final scope and stop rules

Use current host SQLite + Docker/FUSE topology, matching sealed cached builds, existing family setup/perf scripts, seed 1, `--setup clone`, shared measurement lock and unchanged 1,000-file / 100 MiB mixed-v3 recipe. Reuse compatible preparation. No tests or samples are launched by this review. During implementation, changed-seam checks plus one sample per substantive candidate; reuse a valid existing baseline where applicable. If correctness work changes the product, identify its new source and do not claim an isolated A/B against a different implementation.

- If A is semantically unsuitable, skip it rather than broaden the error contract.
- If A achieves the useful accepted outcome, B is optional; do not optimize merely because it is listed.
- If B removes exchanges but fails to improve elapsed, retire that hypothesis; no successor transport campaign.
- Once this pass has a measured keep/reject result, stop optional performance tuning. A preferred-band miss or roughly 0.0500 s difference does not reopen it when the formal result passes.
- Complete concurrent-command/handle Commit, native/direct and required caller adoption, exact recovery/resource/cleanup behavior, same-source delete control and final selected proofs. These are mandatory correctness/completion work, not new optional optimization targets.
- Final independent proofs remain final-only, exact identities, 45 s work / 59 s hard per proof with preparation separate.
- A formal lifecycle MISS or correctness failure is never terminal PASS. Replan within this final scope for a concrete demonstrated defect; if the formal result remains unmet without a justified bounded correction, report that condition rather than silently broaden scope or fabricate success.

Update the final implementation ledger with accepted/rejected changes, exact source and receipts, remaining margins and deferred scale evidence. No automatic parent issue closure.

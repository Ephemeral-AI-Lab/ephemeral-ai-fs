# Independent D implementation review

Reviewed the first implementation in the dedicated diagnostic checkout without building, opening a Store, scanning data or running a sample. D acceptance is pending the corrections and bounded tests below. The parent owns serialization and execution; this reviewer owns only this document.

## Blocking findings in initial source

1. **Generic chunk construction is not regular-file provenance.** Initial `ObjectBuffer::put_file_payload` unconditionally sets the diagnostic FILE bit. But `layerfs-content/src/filesystem/change.rs` constructs mode/mtime metadata values with `rope::build_bytes` (calls around206,248,380,381); that helper calls the same `put_file_payload`. Metadata-value chunks would enter the claimed file-only cohort and cease to appear in the non-file counter. Establish provenance at the actual regular-file construction owner, not by the generic chunk hook alone. Metadata construction must remain non-file unless an independently established file occurrence owns the selected admission. Shared-use objects and first-owner deduplication need explicit treatment. This is a diagnostic classification defect, not proof of product storage corruption.

2. **General budget events do not cover all matcher exits.** Initial terminal inference compares only `budget_skips` before/after `candidate()`. The existing low-level delta matcher can increment `match_budget_skips` or `instruction_budget_skips` without incrementing the general field. Such an incomplete target could be mislabeled no-useful-delta. Infer budget state from the relevant specific-counter deltas and optional-memory decision, preserving partial successful candidates as separate events.

3. **Uncharged per-group allocation changes memory ownership.** The new `diagnostic_terminals: Vec<u8>` allocates one byte per group member outside the existing physical scratch calculation. Layout equality assertions for PhysicalHints and PreparedObject are useful but do not cover this added heap. Reuse the existing diagnostic object's spare field before moving it into PreparedObject, or explicitly preserve the frozen simultaneous-memory contract without introducing a changed capacity gate. No invisible extra heap is justified by calling it diagnostic-only.

4. **Avoid needless observation work in the hot path.** Copying the whole expanded PhysicalStorageReceipt before each object and calling `note_physical` once per fresh object can copy/iterate hundreds of scalar fields repeatedly. Snapshot only the specific counters needed for outcome inference and accumulate a bounded batch receipt. This matters to the requirement for cheap fixed observations and honest observer overhead; it does not authorize changing candidate order or budget decisions.

All four findings were sent directly to the implementation owner and parent before any build approval.

## Initial mechanisms that are sound, subject to final tests

The first/inherited cursor-stop approach follows existing short-circuit ordering: memory availability, per-file reservation, then atomic per-operation reservation. A newly exhausted cursor without callback denial can be identified as descriptor exhaustion; inherited exhaustion preserves the first reason. Partial hints can proceed to candidate selection instead of being forced into an empty-limit terminal.

Two hint bytes use existing private-spill reserved space while keeping144-byte frames; remaining reserved byte and tag validity are checked. Compile-time size/alignment assertions compare old/new hints and prepared objects. Their effectiveness must be checked on the actual pinned build and augmented by unchanged queue/capacity/spill-boundary tests.

Successful admission records terminal/new-FULL/new-DELTA counters after transaction commit. Late recheck losers record race counts/bytes separately; a unique-cohort validator must reject races before interpreting winners' terminal totals as all attempts. No loser may overwrite a winner's unique outcome. A failed transaction may leave attempted observations without admission totals; preserve the failure and do not claim a complete diagnostic.

Pack provenance records successful pack count/bytes/records and last-ID high-water. The high-water counter correctly uses max and remains absolute in `since`; it is not additive across phase snapshots. Derive first/count intervals only at quiescent operation boundaries with no unrelated writer, and validate contiguous/nonoverlapping intervals against one final authenticated locator inventory. Do not infer exact per-transaction provenance from a sum of last IDs.

Current review status: **changes required before D is frozen for execution**. No experimental result is accepted by this source review. Further implementation revisions and tests must address these concrete findings without modifying accepted baseline choices or manufacturing unavailable causal categories.

## Revised implementation source review during first parent-owned compile

The revised source resolves the four initial findings at the code level. `DeferredObjectStore` now defaults to non-file context; actual `FrozenFile::build`, capture construction and complete-file builders opt into file provenance before constructing payloads. Generic mode/mtime `build_bytes` callers remain unmarked. The file builders finish their content root before general filesystem metadata construction, so the reviewed flag does not leak into those metadata operations.

The per-group diagnostic vector is removed. After initial CAS has consumed grant credit and the object has entered MissingBatch, its existing diagnostic grant byte temporarily carries the terminal until PreparedObject takes ownership; the reviewed prepare path performs no subsequent spill/rebuffer that would interpret that byte as grants. PhysicalHints/PreparedObject layout assertions remain. Specific fetch/match/instruction/memory counter deltas now detect incomplete matching correctly, and only the needed scalar counters are copied. Fresh-CAS observations aggregate before one telemetry publication instead of copying the expanded receipt per fresh object.

The actual metadata-versus-file test and forced-spill/predecessor/CAS/pack-gauge test exercise source provenance rather than simply setting test flags and calling a counter. Partial hints still proceed to search. `diag_invalid` is now an absolute sticky invalid-state observation in `since`, so a later phase cannot hide an earlier diagnostic failure merely by subtracting it away. Pack last-ID is likewise an absolute gauge and must not be summed across phase rows.

Execution acceptance remains pending the parent's compile/test result and two concrete test gaps already acknowledged by the implementation owner: an actual two-owner late-admission race and a partially winning pack with retained unlocated records. The existing duplicate-credit/gauge test alone does not prove those paths. The added DeferredObjectStore context boolean is a constant observation cost, but old/new hint/object layout assertions alone do not prove that larger owner struct remains identical; no size-based partition policy using that owner type was found in the reviewed source. Record or test its size rather than claim zero added memory globally.

Early product errors can currently return before final cursor diagnostic receipt emission. Such a sample must remain failed/partial and cannot support grant-conservation PASS. Preserve any available observation and sticky invalid state before propagating the original product error where practical, without changing its behavior or resetting a budget. This is failure-evidence completeness, not grounds to retry an invalid sample as if it passed.

No reviewer build, benchmark, Store access or source mutation occurred during the parent-owned compile. Source fixes are acknowledged; final D execution readiness is not claimed before the pending tests and frozen identity are available.

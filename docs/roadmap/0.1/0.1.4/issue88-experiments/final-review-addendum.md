# Independent final S1 accounting and prospective D review

This additive review owns only this document. The parent owns summary tools and publication; D's implementation owner owns its separate diagnostic checkout. No duplicate Store census, decoding, build, benchmark or encoding was performed by this reviewer.

## Pre-aggregation source review

Reviewed `census_report.py`, `summarize_full.py` and its `summarize_public.metrics` dependency. The census uses the already completed authenticated inventory and preserved pre-verification logical snapshot. Its core equations are correctly nested: SQLite page bytes plus freelist equal logical file size; actual BLOB lengths equal pack framing plus encoded group lengths; decoded groups equal count/directory plus FULL/DELTA record lengths. Overflow is included in the table-owned page partition. The copied file's allocation is not substituted for original acknowledgement allocation.

The following concrete checks/fields were requested from the parent before final publication; they do not require another decoder or original Store scan:

- Bind snapshot and inventory content hashes to upstream custody before and after aggregation. `custody.json` assertions alone are not independent authentication. Identify the executing census decoder binary/source qualification.
- Explicitly separate `object_packs` SQL row encoding from actual BLOB bytes: table SQL payload minus summed pack BLOB length. Preserve unused/page-overhead values without calling them recoverable.
- Validate each required physical base selects an authenticated FULL record under current S1's depth-one policy. Report B-minus-L count/canonical bytes and selected objects outside R=L∪B, rather than leaving `outside_logical_union` liable to be read as garbage. Base-only objects belong inside outside-L; do not count them twice. No recursive base traversal is needed for this particular proven depth-one encoding; future native chains will need it.
- Verify all157 indices/source states and performance→verifier mappings against original frozen scope. `metrics()` checks PASS/cleanup and counts verifier rows but does not itself prove identical all157 mappings or oracle contents. Preserve the independently executed all-state verifier and original manifest checks; final summary must reference them or assert the required equality directly.
- Counter totals in `summarize_full` are checkpoint-public phase sums excluding Init. Label that population explicitly. The underlying `performance_wall_ns` is case wall from the result, not the preparation-inclusive invocation wall.

The observed original335552512→candidate302006272 allocated-byte reduction remains33546240 B. Logical318832640→296845312 falls21987328 B; the remaining11558912 B change is the difference between signed allocation adjustments. It must not be attributed to additional encoded S1 savings. Initial/random canonical construction differs by152 objects/51168 B, so equality claims concern retained filesystem states rather than identical internal canonical populations.

## Finality boundary

Before the parent's finalization, a completed raw inventory is not a sealed report: output summary, manifests, source/binary custody and independent reconciliation must exist. This review does not treat an unexecuted source file or a `FULL_HISTORY_RESULT` placeholder as completed evidence. The parent can resolve the concrete aggregation checks above and publish without repeating successful product measurements.

## Selected integration control resolved

The selected next combined comparison is fresh S1 legacy physical representation versus S1 plus native payload wire2 representation. This measures the whole declared physical-replacement package, including framing, codec and selected-base reader policy. S1 policy remains fixed; its physical bytes and shared-budget behavior may still change and are measured. It is not a codec-only comparison. Existing same-unit offline FULL/PREFIX evidence supports mechanism interpretation; no mandatory four-arm full157 campaign is required. All required bases and new envelopes are counted once. The159163199 heterogeneous historical component sum remains a reference, not a new complete-Store gate.

## Prospective diagnostic review scope

D is a bounded prerequisite against explicitly frozen accepted M4.5, not an authorization to combine a new prefix codec or coverage policy into the observation run. Review will check exact file-content chunk/source qualification after authoritative CAS filtering, preserved first-owner hints and grant credits, first/inherited cursor-stop precedence, complete-empty versus exhausted-empty distinction, finite histograms, no timed per-target logs, unchanged capacity/spill/memory thresholds, and unique-cohort/race-invalid handling. Structural inode origins intentionally lack file spans; they must not become false missing-span defects in a role-generic counter. A reused occurrence can initialize correspondence for a later missing one, so grants triggered by reuse remain work attribution rather than measured waste.

D implementation acceptance is pending source delivery from its owner; this addendum does not certify unreviewed future code or claim any new diagnostic sample was executed.

## Completed aggregate review

The parent produced `issue88-report-1/full157-results.json` and `full157-census.json`; the earlier missing-summary limitation is resolved. Reviewed their complete aggregate fields without opening a Store or repeating decoding. JSON-only sums independently balance72472×4096=296845312 logical page B, every table payload+unused+overhead partition, all366293 selected objects, and273204901 encoded group B plus703520 outer framing B =273908421 pack BLOB B. SQL pack row encoding24347 B is explicitly separate.

The public candidate's payload lane is exactly216448341 encoded B, identical to the historical lane. Structural groups are56756560 encoded B versus78792537, a22035977 B reduction. Complete pack BLOB reduction is22036281 B, including304 B less outer framing. This independently localizes the public physical representation difference to structural data; shared-budget interaction was a prospective risk but the final measured payload lane did not change. There are4674 structural and8024 payload DELTAs. Candidate totals remain799576457 canonical B/366293 objects; internal-population differences versus historical control remain disclosed.

All366293 selected objects are within the logical root union. The summary validates selected FULL bases for all12698 DELTAs, with7736 distinct bases, zero physical-base-only objects/bytes, zero outside-required-union objects and zero unselected physical records. No garbage or reclaimable-base claim is needed.

The root enumeration is appropriate to this Store schema. The reused census explicitly includes all `layers`, `commits` and `workspace_stages` root IDs. Product `sql/query/branch_roots_page.sql` derives each branch's effective root as `COALESCE(commits.root_id,layers.root_id)`; branch head/base references target those tables through schema foreign keys. Branches therefore introduce no separate canonical root omitted by enumerating all commits/layers. Durable pending stage roots are included through `workspace_stages`; successful owner cleanup supplies the live-pending qualification. Future schema changes would require revisiting this argument.

Updated tools verify snapshot/index hashes before/after aggregation and expose them in the summary. The results now explicitly check all157 mapping/input scope and verified4936693030 bytes per arm, distinguish checkpoint counters from Init, and separate case wall from enclosing preparation-inclusive invocation summaries. These resolve the concrete source-review requests above. Final external manifest sealing and whole-run custody authentication remain the parent owner’s publication step; this reviewer does not represent an in-progress hash audit as completed.

The final candidate302006272 allocated B still does not meet134221004. It retains an observed33546240 B allocation reduction with21987328 B logical reduction and11558912 B signed-adjustment change. Historical timing is descriptive, not a fresh matched speedup, and earlier smoke regressions remain visible. No combined S1+S2 public size has been measured yet.

## Prospective native pack-v2 contract review

Read `issue88-native-contract.md` in full. The5-byte FULL/37-byte PREFIX headers, raw group directories and payload-only version2 packs are internally coherent. Prefix base IDs are canonical IDs, not the offline raw hashes; canonical length remains raw+21 and native output must regenerate/authenticate that exact canonical object before CAS comparison. Version dispatch prevents interpreting old COPY/INSERT records as native frames. Keeping structural groups on wire1 and native payload groups on wire2 avoids double compression and requires no separate database or canonical rewrite.

The declared chain bounds balance: five enclosing65536-byte groups plus five32-byte header/entry reads total327840 B, within393216 B; five group-work charges plus five32789-byte reconstructed canonical chunks total491625 B, within524288 B. Five raw chunks total at most163840 B, so the separate1MiB raw-closure limit is conservative. A maximum33024-byte frame plus37-byte native record header and one8-byte group directory fits65536 B. Sequential chain reconstruction, owned frame slices, one live prefix/output pair and one static decoder context make the1MiB scratch proposal plausible; exact vector/context capacities still require the contract's compiled worst-case tests, not arithmetic alone.

Requested one explicit budget clarification: retain existing per-target512KiB optional read limits as well as8MiB batch limits and8 lookup bound; the new chain ceilings do not reset or replace existing budget state. Initial optional locator lookup and any repeated extraction must remain charged consistently. Optional work exhaustion falls back FULL without a new allowance; mandatory static-context failure stops rather than silently changing codec settings.

Unsupported legacy custom-DELTA prior→native FULL fallback is coherent and intentionally narrower than reconstructing every prior representation. It avoids adding a second legacy-chain mechanism, but means the public prototype cannot inherit S2's102306097-byte expectation: S2 allowed actual selected native-prefix predecessors under an offline first-overlap policy. First-delivered-hint-only public choice, unavailable legacy-DELTA bases and shared budgets can materially alter coverage. This is an explicit physical-replacement experiment, not a claim that only the codec literal changed.

The contract correctly gates strict single ordinary Zstd frames, content size/checksum/no dictionary-ID field, bounded window, canonical authentication, selected-before-target provenance, no admitted missing/cyclic dependencies, CAS races and partial-pack residue. Old readers reject wire2; compatibility is new-reader support, not old-binary readability or in-place migration. No intrinsic feasibility contradiction was found. Implementation acceptance still requires compiled static-context/lifetime/capacity proofs and the D/C delivery decision; this review does not authorize skipping those gates or assert a new sample was run.

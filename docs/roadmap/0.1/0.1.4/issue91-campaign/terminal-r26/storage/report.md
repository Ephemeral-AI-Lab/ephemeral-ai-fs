# R26 full157 storage and resource report

**The corrected candidate passes the full157 storage gate:** original acknowledged allocation is **184,586,240 bytes**, versus **218,116,096 bytes** for the reused control—a saving of **33,529,856 bytes (15.37%)**. Both use 4,096-byte SQLite pages. Both completed their 157 historical verifications; storage accounting and the preceding receipt validations passed. This passes the storage comparison. The terminal coordinator reports 198 performance cases and 226 routine proofs passed, with one optional observation not run; comparison binding is resolved, with four historical Git comparisons still ineligible and the unchanged report INCOMPLETE. Final campaign qualification and release are not asserted. [Control accounting](control/accounting-reconciliation.json), [candidate accounting](candidate/accounting-reconciliation.json), [completed resource/proof summary](resources/resource-timing-summary.json).

The supplemental C+S1 control is the original passed R25 observation, retained under its original schedule and census custody. It is not the published v0.1.3 baseline. R26 is one fresh repaired-candidate observation; differences below are descriptive, not statistical paired-speedup claims.

| Storage dimension (bytes unless stated) | Control | Corrected R26 |
|---|---:|---:|
| Original acknowledged allocation | 218,116,096 | 184,586,240 |
| Logical database size | 205,529,088 | 176,226,304 |
| Signed allocation minus logical size | 12,587,008 | 8,359,936 |
| Sidecar allocation | 0 | 0 |
| SQLite pages (count) | 50,178 | 43,024 |
| Pack table pages, including overflow | 186,679,296 | 157,392,896 |
| Objects primary-key B-tree pages | 18,759,680 | 18,743,296 |
| Other metadata pages | 90,112 | 90,112 |
| Internal B-tree unused bytes | 5,510,090 | 4,328,866 |
| Freelist pages (count) | 0 | 0 |
| Unexplained page residual | 0 | 0 |

Logical size falls by 29,302,784 bytes (14.26%). Almost all of that reduction is in pack-table pages: 29,286,400 bytes; the locator B-tree drops by 16,384 bytes and other metadata is unchanged. Allocation savings also include a 4,227,072-byte change in the signed allocation adjustment; its filesystem cause is unproven. These are original acknowledgement values, not copied-snapshot allocation. [Reconciled accounts](candidate/accounting-reconciliation.json).

**PREFIX is restored.** Native header lengths imply **58,306 PREFIX records and 28,106 FULL records**, totaling 86,412. Their frames occupy 27,155,798 and 68,435,250 bytes respectively. The 705,162,813 native canonical bytes are reconstructed logical content, not another physical storage addend. Candidate pack count is 3,324 versus 4,917 control; encoded pack BLOBs are 154,891,515 versus 182,825,792 bytes. Both inventories have zero unselected records, zero base-only objects and zero selected objects outside required retention; selected object counts differ (366,247 versus 366,173), so equal retained file states must be established by historical proofs, not equal object counts. [Native and retention accounting](candidate/accounting-reconciliation.json).

Official SQLite analyzer reports independently agree with the logical page counts and zero freelist. Candidate object_packs uses 36,241 overflow pages versus 42,230 control; only 120,139 versus 154,750 overflow bytes are unused. Pack leaf slack is 2,145,482 versus 3,389,244 bytes; both pack B-trees have depth 3. Internal slack is not a freelist and is not automatically reclaimable. Analyzer custody records exit 0 and unchanged before/after snapshot hashes for both arms. [Control analyzer](control/analyzer.txt), [candidate analyzer](candidate/analyzer.txt), [control custody](control/analyzer-custody.json), [candidate custody](candidate/analyzer-custody.json).

| Timing/resource scope | Control | Corrected R26 |
|---|---:|---:|
| Performance enclosing invocation (s) | 552.430003334 | 527.151979209 |
| Performance fixture preparation (s) | 77.611382667 | 74.893891625 |
| Performance case wall (s) | 474.329086083 | 451.741927208 |
| Performance case work wall (s) | 472.506156625 | 448.302267709 |
| Performance maximum observed host lifetime RSS (bytes) | 114,278,400 | 115,326,976 |
| Verification enclosing invocation (s) | 576.399468583 | 594.101470000 |
| Verification fixture preparation (s) | 73.370156416 | 73.120071333 |
| Verification case wall (s) | 502.697860167 | 520.650875209 |
| Verification case work wall (s) | 499.836389583 | 518.113577042 |
| Verification maximum observed host lifetime RSS (bytes) | 75,071,488 | 74,268,672 |
| 157 public Commits elapsed (s) | 76.970696573 | 49.822261842 |
| 157 public Commits host CPU (s) | 77.665984538 | 50.422915089 |
| 157 verification executions elapsed (s) | 432.934444956 | 452.475388746 |
| 157 verification executions host CPU (s) | 183.297328664 | 189.836842171 |

Performance case wall is 4.76% lower, while verification case wall is **3.57% higher**. Commit elapsed falls 35.27%; verification execution elapsed rises 4.51% and its host CPU rises 3.57%. Do not hide that read/verification tradeoff. Named public phases are nested inside case/invocation times; do not add table rows. CPU values cover the named host phase only, and RSS maxima are cumulative lifetime high-water observations, not isolated phase allocations. [Exact resource definitions and values](resources/resource-timing-summary.json).

Snapshot/census observer times are separate: control **1.097683125 / 20.280793625 s**, candidate **1.029678459 / 23.363861125 s**. Analyzer elapsed is separately **0.442983250 / 0.368418750 s** for control/candidate. None is added to public Commit/Init timings. Host-runtime disk maxima already contain Store and spool; those scopes must not be summed. [Observer/resource custody](resources/resource-timing-summary.json).

**The failed R25 all-FULL attempt remains part of the evidence:** 318,803,968 allocated bytes and 317,616,128 logical bytes, with zero PREFIX—a 46.16% allocation regression against this same control. R26 restores PREFIX and passes the storage threshold; it does not erase that failure or automatically settle campaign severity gates. The 198 performance cases and 226 routine proofs passed, one optional observation was not run, and comparison binding is resolved, with four historical Git comparisons still ineligible and the unchanged report INCOMPLETE. [Preserved R25 failed accounting](failed-r25/accounting-reconciliation.json).

This report and its machine-readable [summary](summary.json) were derived only from existing JSON/CSV-compatible accounts and analyzer text/custody. The exact summary retains its original creation-time “full campaign pending” status; the execution update above is the terminal coordinator’s later status at packaging time. No database, snapshot or inventory was opened; no new measurement was taken.

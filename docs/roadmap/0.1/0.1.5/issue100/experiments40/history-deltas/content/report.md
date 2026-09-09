# Full157 content/Git attribution: the content gap grows materially with history

For the **same75,398SmallContent canonical objects**, current LayerFS frames occupy **56,463,014 B** and Git entries **44,975,837 B**: an **11,487,177-byte difference**. Including the current compact SmallContent records/directories/packs raises the same-object difference to **14,003,863 B**. The earlier ten-state433,886-byte frame difference does not describe this history.

This report reads the final unsupported D-B-CDC offline copy. All75,398objects were decoded and canonically authenticated through the existing StoreReader with4MiBcanonical and2MiBpack LRU payload caps; exact raw bytes were joined by Git blobSHA1 to the existing full157Git pack. Source Store and Git inventory hashes matched before/after. There was no encoding, repacking, policy trial or product change.

## Whole-population comparison, including opposite base choices

Git entry bytes include Git's own framing; LayerFS frame bytes exclude LayerFS framing. They are useful payload-level comparisons with that boundary distinction.

| Current LayerFS / Git representation | Objects | LayerFS frame B | Git entry B |
|---|---:|---:|---:|
| FULL / FULL |2,923 |6,638,329 |6,291,403 |
| FULL / DELTA needing future closure |6,037 |20,682,353 |1,088,574 |
| FULL / DELTA with same-checkpoint closure |82 |97,851 |32,440 |
| FULL / DELTA with earlier closure |322 |1,357,974 |125,449 |
| DELTA / DELTA needing future closure |49,272 |21,156,003 |7,897,514 |
| DELTA / DELTA with same-checkpoint closure |1,245 |308,573 |415,879 |
| DELTA / DELTA with earlier closure |6,314 |1,699,565 |1,794,553 |
| DELTA / FULL |9,203 |4,522,366 |27,330,025 |
| **Same canonical object population** |**75,398** |**56,463,014** |**44,975,837** |

The20.68-MB future-dependent FULL population is **not** an approximately19.59-MB saving forecast. In the opposite population, LayerFS DELTAs save **22,807,659 B** relative to Git FULL entries. An alternative graph must count newly FULL bases, all changed descendants and every retained representation once. After all opposing assignments cancel, the real frame-versus-entry difference remains11,487,177B.

The current SmallContent physical reconciliation is:

| Component | Bytes |
|---|---:|
| Compressed frames |56,463,014 |
| Compact kind/base-reference fields |2,188,486 |
| Compact group-offset directories |301,592 |
| Pack headers |26,608 |
| **SmallContent pack bytes** |**58,979,700** |

Thus **2,516,686 B** of the same-object physical difference lies outside compressed frames. This is the already compacted format, not the old9/41-byte record framing. Costs must not be subtracted twice from the final copy.

## Existing chain limits explain a concrete additional FULL population

The current9,364FULL objects consume28,776,507frame bytes. Their original first-appearance path facts are:

| Original prior-path situation | FULL objects | Current frame B |
|---|---:|---:|
| Initial snapshot, no prior path |220 |437,383 |
| Later first appearance at a new path |7,592 |18,820,719 |
| Previous SmallContent exceeds a structural chain limit when adding target |1,528 |**9,254,960** |
| Previous SmallContent remains within structural limits |16 |38,640 |
| Previous object uses a different representation |8 |224,805 |

For those1,528structurally capped cases:

| Limit exceeded by extending the prior object | Objects | Current FULL frame B |
|---|---:|---:|
| Eight-edge limit only |1,446 |**7,655,946** |
|512-KiB decoded canonical closure only |67 |**1,354,592** |
| Both limits |15 |**244,422** |

The capped prior graphs reach depth8 and523,264canonical closure bytes. Their maximum retained encoded closure is only56,629B; **encoded-byte capacity is not the binding cap in this population**. Raising only the encoded allowance would not make these predecessors eligible. There are4,039currently selected depth-eight SmallContent objects. Matching Git objects reach depth48; the fullGit blob population reaches50.

These are **structural eligibility facts, not runtime hint-cause telemetry**. The exact original previous-path content is known, and its selected graph cannot be extended within the current relevant limit. The diagnostic does not prove that this was the actual delivered predecessor hint, or that no cache candidate was tried. Current fallback can search a retained FULL candidate after predecessor rejection. The9,254,960B is a population to investigate, not an established recoverable amount.

Largest individual path families in this population:

| Original path | Capped FULL objects | Frame B |
|---|---:|---:|
| packages/client/ui-primitives/src/icons/index.tsx |3 |111,400 |
| scripts/gen-doc-graphs.ts |5 |93,498 |
| scripts/snapshots/translation-prompt-v4/request-response.expected.json |2 |92,848 |
| packages/client/connection/src/client/fixture.ts |3 |86,732 |
| scripts/gen-cordis-catalog.ts |6 |78,123 |
| docs/config-catalog.md |3 |75,316 |

This is distributed across real histories rather than one removable oversized file. The first table also shows a remaining13,258,489B frame-versus-entry difference among objects that both systems already deltify but Git represents with future-dependent closure. Chains exist; direction and selected base quality still matter. Of that group,42,846objects use current chain kind2 with15,758,542frame bytes versus6,824,541Git entry bytes. A claim that all excess is legacy FULL-anchor kind1 would therefore be wrong.

## Exact writer behavior and limits

Source references are under `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100`:

- `crates/layerfs-layerstack-store/src/objects.rs:2447` supplies the SmallContent predecessor through the shared prevalidated admission path. That logical identity is distinct from the physical base eventually selected.
- `objects/admission.rs:228` asks for the immediate predecessor's reconstructed bytes. `objects/read.rs:289` returns no eligible predecessor if adding the target exceeds8edges or512KiBcanonical closure. The fixed constants are `objects/delta.rs:7–9`.
- `objects/admission.rs:239` consults the retained candidate index **only when the predecessor is unavailable**, not after an otherwise valid predecessor yields a poor delta. It finds one FULL candidate and authenticates its stored bytes before encoding.
- `objects/admission.rs:272` selects a prefix only if its frame plus32-byte base reference beats FULL, while checking encoded closure. This is already a complete elementary FULL-versus-PREFIX comparison, not a missing reference-cost check.
- `objects/small_candidates.rs:4–9` bounds candidates to1,024FULL entries and8,192single-reference hash buckets; its sixteen-byte/eight-minhash signature selects one candidate with at least two matching fingerprints. `objects/admission.rs:982` admits only physically selected FULL winners to this index.
- `objects/read.rs:295–368` reconstructs and authenticates the dependency chain with independent scratch/closure guards. Increasing writer depth alone would violate reader expectations and existing format compatibility; limits cannot be removed as an unqualified tuning change.

Together these explain why long history can add FULL reset costs and why base availability/quality remains limited. They do not isolate codec quality: this attribution does not encode identical target/base alternatives. The earlier five-pair identical-base matcher study is too small to establish a full157codec ranking.

## Remaining large-file population is separate

Git has531blobs outside the75,398SmallContent identities: **523large regular-file versions,2,006,385entry bytes**, plus seven symlinks302B and one empty blob9B. Symlink/empty handling belongs to LayerFS metadata/empty representation, so these311B must not be silently charged as native payload.

The already optimized full157native packs occupy **7,211,036 B**. Compared with the523large Git versions, their difference is **5,204,651 B**. This is a complete large-population comparison, not per-file recoverability; chunk sharing, direction and roots/maps are different representations. Existing native graph authentication and large-file checks were reused; no unnecessary large content redecoding was performed here.

SmallContent packs plus native packs total66,190,736B. Git's all-blob entries total46,982,533B, a19,208,203B category difference including the311B placement difference above. This is content-only accounting; metadata/index differences remain separately owned.

## Best next content experiment, not executed

Freeze a small byte-ranked sample of real capped FULL targets across the named families. Compare actual FULL against **one newest already selected predecessor ancestor that satisfies the existing8edge/512KiB/256KiBlimits**, using the unchanged codec and complete record cost. Choosing an existing shallower ancestor can preserve the current format and bounds; it may avoid an expensive FULL reset, but can also accumulate larger deltas against an older base. No alternative is guaranteed better.

If that fixed sample is useful, extend only a fixed policy over complete selected original families in chronological admission order, updating descendant depths/closures after every choice, counting all new FULL resets and verifying all affected historical bytes. Do not sum isolated FULL wins or use Git future-base entry costs as replacement estimates. Record extra read/decode/authentication and Commit work. This tests a narrower evidence-backed intervention than raising all chain limits or starting a repacker.

Artifacts: `attribute.py` provides the complete authenticated identity join and executable invariants; `attribution.json` retains all75,398rows, original prior-path facts and net classes. `summarize.py`/`cap-summary.json` provide cap distributions, opposing kind1/kind2 populations and largest real families; `cap-maxima.json` gives explicit prior maxima. The first attempt stopped at a mistyped expected checksum literal before reading content; corrected source identity matched the supplied digest, with the initial script retained. No source Store or Git artifact changed.

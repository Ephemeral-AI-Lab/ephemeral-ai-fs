# Full157 offline CDC: fixed previous-file similarity policy

The chronological alternative retains all 1,347 lockfile chunk identities and changes complete record bytes **2,968,670→2,059,404**, a **909,266-byte net decrease**. This includes 133 worsened targets and 130 newly FULL records totaling 592,845B. Improvements: 433; unchanged: 781. Original FULL count 279 versus final 243. No isolated-pair saving extrapolation is used.

All 156 original large lockfile checkpoint states reconstruct exactly under the final graph. The first checkpoint's SmallContent file is outside this CDC policy. Actual frozen-product scanner spans, canonical IDs, first physical pack admissions and immediately preceding checkpoint first-overlap hints match without missing objects or mismatches. Unchanged prior versions remain the immediate predecessor. Prepared incremental input directories omit some unchanged blobs; exact originalGit object reads were checked against GitSHA1 and identity-sealed original oracleSHA256. No replacement history was built.

## Full native graph and physical layout

Every one of 3,221 native objects authenticates after 1,347 substitutions. Outside-family objects depending on substituted lock chunks: 0. Maximum depth 4; raw closure 163,840B; lookups 5; encoded-work 39,667B; decoded-work 173,899B. Canonical lengths, roles, checksums, strict original-pack dependency chronology and acyclic graphs pass.

Original native pack total **8,120,302→7,211,036B** when replacing record bodies and recomputing directories in their original groups/packs. Largest group 64,862B; largest native pack 250,489B; packs requiring split: 0. Original group and pack layout coordinates are exported with selected frames. This is offline physical accounting, not a product Store allocation measurement. The combined-layout agent owns actual isolated-copy construction and complete historical filesystem checks.

## Bounded policy and costs

The same16-byte/eight-minhash policy compares FULL, first-overlap, and at most one best previous-file signature winner (>=2shared, IDtie), with no second-ranked fallback or changed cutoff/codec. Previous-file index remains capped128entries/1MiBraw and is released when progressing to a new preceding checkpoint. Raw predecessor bytes are decoded sequentially without an unbounded content cache. Observed largest index: 42entries/830,939raw bytes.

Index construction across 142 preceding states costs 11,199 closure lookups, 29,484,509encoded-work B and 270,712,900decoded-work B. Index-building diagnostic time 60.029s; total simulation plus file verification 95.219s. These Python/ctypes scopes are not production CPU, peakRSS or publicCommit latency. Candidate dispositions: {'encoded': 1428, 'depth_or_raw_closure': 313, 'cumulative_target_read_budget': 76}. Changed predecessor graphs are used for all later candidates, so depth resets and losses remain in net totals.

Existing4edge/1MiBrawclosure and conservative cumulative optional8lookup/512KiBtarget envelopes are enforced. Original remaining512trial/8MiBbatch and operation quotas, actual product read-wave simultaneous ownership, rollback/durability and public performance remain unqualified. Native decoding uses authenticated diagnostic DCtx; no production decoder allocation equivalence is claimed. No48lookup batch limit exists in the current native-reader source; none is silently assumed or claimed tested.

All source Store reads remained immutable at SHA256 `f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7`. Protocol and executable assertions accompany each output. The shared tools use the exact product scanner, LayerFS-domain BLAKE3 and verified Zstd1.5.7 native parameter sequence. Each original selected target frame was reproduced before selecting its alternate, and every alternate target roundtripped.

## Export and audit

`family-result.json` contains every selected frame(hex), original pack/group/ordinal and updated baseID/kind, per-target costs/caps, and checkpoint reconstruction proofs. `all-native-verification.json` provides rawSHA256 for every native object plus closure and pack facts. `shared.py` exposes records/decode/object_id and the pinned codec; `verify_graph.py` applies the exported graph and checks all native objects. `provenance.json` records source seals and exact first-admission relationships.

The initial provenance attempt found an absent unchanged prepared blob and stopped; originalGit/oracle retrieval above resolved it without altering inputs. The first simulation attempt stopped because imported decoder globals did not follow the runner's original-versus-simulated graph switch; the corrected runner uses the ten-state validated shared-definition wiring. Failed scripts/log are retained. Neither attempt emitted a partial family result or changed the Store.

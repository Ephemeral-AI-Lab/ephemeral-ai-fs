# Stride3 offline CDC: replication of the fixed previous-file similarity policy

On the separately captured53-state stride3 history, the chronological alternative retains all 794 lockfile chunk identities and changes complete record bytes **1,970,026→1,269,941**, a **700,085-byte net decrease**. This includes 67 worsened targets and 66 newly FULL records totaling 339,827B. Improvements: 265; unchanged: 462. Original FULL count 192 versus final 144. No isolated-pair saving extrapolation is used.

All 52 original large lockfile checkpoint states reconstruct exactly under the final graph. The first checkpoint's SmallContent file is outside this CDC policy. Actual frozen-product scanner spans, canonical IDs, first physical pack admissions and immediately preceding checkpoint first-overlap hints match without missing objects or mismatches. Unchanged prior versions remain the immediate predecessor. Prepared incremental input directories omit some unchanged blobs; exact originalGit object reads were checked against GitSHA1 and identity-sealed original oracleSHA256. No replacement history was built.

## Full native graph and physical layout

Every one of 1,998 native objects was authenticated in the source graph first, and authenticates again after 794 substitutions. Outside-family objects depending on substituted lock chunks: 0. Maximum depth 4; raw closure 163,840B; lookups 5; encoded-work 34,936B; decoded-work 178,627B. Canonical lengths, roles, checksums, strict original-pack dependency chronology and acyclic graphs pass.

Original native pack total **5,696,523→4,996,438B** when replacing record bodies and recomputing directories in their original groups/packs. Largest group 65,528B; largest native pack 257,565B; packs requiring split: 0. Original group and pack layout coordinates are exported with selected frames. This is offline physical accounting, not a product Store allocation measurement. The combined-layout agent owns actual isolated-copy construction and complete historical filesystem checks.

## Bounded policy and costs

The same16-byte/eight-minhash policy compares FULL, first-overlap, and at most one best previous-file signature winner (>=2shared, IDtie), with no second-ranked fallback or changed cutoff/codec. Previous-file index remains capped128entries/1MiBraw and is released when progressing to a new preceding checkpoint. Raw predecessor bytes are decoded sequentially without an unbounded content cache. Observed largest index: 42entries/830,939raw bytes.

Index construction across 50 preceding states costs 3,897 closure lookups, 9,969,381encoded-work B and 93,381,700decoded-work B. Index-building diagnostic time 21.110s; total simulation plus file verification 39.555s. These Python/ctypes scopes are not production CPU, peakRSS or publicCommit latency. Candidate dispositions: {'encoded': 828, 'depth_or_raw_closure': 181, 'cumulative_target_read_budget': 52}. Changed predecessor graphs are used for all later candidates, so depth resets and losses remain in net totals.

Existing4edge/1MiBrawclosure and conservative cumulative optional8lookup/512KiBtarget envelopes are enforced. Original remaining512trial/8MiBbatch and operation quotas, actual product read-wave simultaneous ownership, rollback/durability and public performance remain unqualified. Native decoding uses authenticated diagnostic DCtx; no production decoder allocation equivalence is claimed. No48lookup batch limit exists in the current native-reader source; none is silently assumed or claimed tested.

All source Store reads remained immutable at SHA256 `5c6ee04eee133539f043ee64d242c26769c77d30b434af2523666e326a8d999e`. Protocol and executable assertions accompany each output. The shared tools use the exact product scanner, LayerFS-domain BLAKE3 and verified Zstd1.5.7 native parameter sequence. Each original selected target frame was reproduced before selecting its alternate, and every alternate target roundtripped.

## Export and audit

`family-result.json` contains every selected frame(hex), original pack/group/ordinal and updated baseID/kind, per-target costs/caps, and checkpoint reconstruction proofs. `all-native-verification.json` provides rawSHA256 for every native object plus closure and pack facts. `shared.py` exposes records/decode/object_id and the pinned codec; `verify_graph.py` applies the exported graph and checks all native objects. `provenance.json` records source seals and exact first-admission relationships.

This replication changed only source paths, selection/cardinality assertions and labels from the retained full157 scripts. The794target identities come from the53-state graph itself; no157-state selected frames or bases were reused. There were no failed provenance or encoding attempts in this replication. The frozen protocol is [protocol.md](protocol.md). All53selected source manifests/oracle identities belong to the sealed stride3 fixture; all52large lockfile states are explicitly reconstructed here. The parent complete-copy experiment owns all53complete filesystem/oracle checks.

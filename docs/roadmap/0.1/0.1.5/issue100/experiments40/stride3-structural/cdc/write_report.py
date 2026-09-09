import json,collections
from pathlib import Path
out=Path(__file__).parent
f=json.loads((out/'family-result.json').read_text());v=json.loads((out/'all-native-verification.json').read_text());p=json.loads((out/'provenance.json').read_text());s=f['summary'];cap=v['summary'];rows=f['results'];indexes=f['indexes'];reasons=collections.Counter(c.get('rejection','encoded') for r in rows for c in r['candidates'])
source=json.loads((out/'source-native-verification.json').read_text());assert source['status']=='PASS' and source['objects']==1998
text=f'''# Stride3 offline CDC: replication of the fixed previous-file similarity policy

On the separately captured53-state stride3 history, the chronological alternative retains all {s['objects']:,} lockfile chunk identities and changes complete record bytes **{s['original_record_bytes']:,}→{s['final_record_bytes']:,}**, a **{s['saved_record_bytes']:,}-byte net decrease**. This includes {s['worsened_targets']} worsened targets and {s['new_full_count']} newly FULL records totaling {s['new_full_record_bytes']:,}B. Improvements: {s['improved_targets']}; unchanged: {s['unchanged_targets']}. Original FULL count {s['original_full_count']} versus final {s['final_full_count']}. No isolated-pair saving extrapolation is used.

All {p['large_states']} original large lockfile checkpoint states reconstruct exactly under the final graph. The first checkpoint's SmallContent file is outside this CDC policy. Actual frozen-product scanner spans, canonical IDs, first physical pack admissions and immediately preceding checkpoint first-overlap hints match without missing objects or mismatches. Unchanged prior versions remain the immediate predecessor. Prepared incremental input directories omit some unchanged blobs; exact originalGit object reads were checked against GitSHA1 and identity-sealed original oracleSHA256. No replacement history was built.

## Full native graph and physical layout

Every one of {cap['native_objects']:,} native objects was authenticated in the source graph first, and authenticates again after {cap['substituted_objects']:,} substitutions. Outside-family objects depending on substituted lock chunks: {cap['outside_family_dependents']}. Maximum depth {cap['maximum_edges']}; raw closure {cap['maximum_raw_closure']:,}B; lookups {cap['maximum_lookups']}; encoded-work {cap['maximum_encoded_work']:,}B; decoded-work {cap['maximum_decoded_work']:,}B. Canonical lengths, roles, checksums, strict original-pack dependency chronology and acyclic graphs pass.

Original native pack total **{cap['original_v2_pack_bytes']:,}→{cap['simulated_v2_pack_bytes']:,}B** when replacing record bodies and recomputing directories in their original groups/packs. Largest group {cap['maximum_group_bytes']:,}B; largest native pack {cap['maximum_pack_bytes']:,}B; packs requiring split: {cap['packs_requiring_split']}. Original group and pack layout coordinates are exported with selected frames. This is offline physical accounting, not a product Store allocation measurement. The combined-layout agent owns actual isolated-copy construction and complete historical filesystem checks.

## Bounded policy and costs

The same16-byte/eight-minhash policy compares FULL, first-overlap, and at most one best previous-file signature winner (>=2shared, IDtie), with no second-ranked fallback or changed cutoff/codec. Previous-file index remains capped128entries/1MiBraw and is released when progressing to a new preceding checkpoint. Raw predecessor bytes are decoded sequentially without an unbounded content cache. Observed largest index: {max(i['entries'] for i in indexes)}entries/{max(i['indexed_raw_bytes'] for i in indexes):,}raw bytes.

Index construction across {len(indexes)} preceding states costs {s['index_lookups']:,} closure lookups, {s['index_encoded_work']:,}encoded-work B and {s['index_decoded_work']:,}decoded-work B. Index-building diagnostic time {s['index_build_ns']/1e9:.3f}s; total simulation plus file verification {s['total_elapsed_ns']/1e9:.3f}s. These Python/ctypes scopes are not production CPU, peakRSS or publicCommit latency. Candidate dispositions: {dict(reasons)}. Changed predecessor graphs are used for all later candidates, so depth resets and losses remain in net totals.

Existing4edge/1MiBrawclosure and conservative cumulative optional8lookup/512KiBtarget envelopes are enforced. Original remaining512trial/8MiBbatch and operation quotas, actual product read-wave simultaneous ownership, rollback/durability and public performance remain unqualified. Native decoding uses authenticated diagnostic DCtx; no production decoder allocation equivalence is claimed. No48lookup batch limit exists in the current native-reader source; none is silently assumed or claimed tested.

All source Store reads remained immutable at SHA256 `{p['store_sha256']}`. Protocol and executable assertions accompany each output. The shared tools use the exact product scanner, LayerFS-domain BLAKE3 and verified Zstd1.5.7 native parameter sequence. Each original selected target frame was reproduced before selecting its alternate, and every alternate target roundtripped.

## Export and audit

`family-result.json` contains every selected frame(hex), original pack/group/ordinal and updated baseID/kind, per-target costs/caps, and checkpoint reconstruction proofs. `all-native-verification.json` provides rawSHA256 for every native object plus closure and pack facts. `shared.py` exposes records/decode/object_id and the pinned codec; `verify_graph.py` applies the exported graph and checks all native objects. `provenance.json` records source seals and exact first-admission relationships.

This replication changed only source paths, selection/cardinality assertions and labels from the retained full157 scripts. The794target identities come from the53-state graph itself; no157-state selected frames or bases were reused. There were no failed provenance or encoding attempts in this replication. The frozen protocol is [protocol.md](protocol.md). All53selected source manifests/oracle identities belong to the sealed stride3 fixture; all52large lockfile states are explicitly reconstructed here. The parent complete-copy experiment owns all53complete filesystem/oracle checks.
'''
(out/'report.md').write_text(text)

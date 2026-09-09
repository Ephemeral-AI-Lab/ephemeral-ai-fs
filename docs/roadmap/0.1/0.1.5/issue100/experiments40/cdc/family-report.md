# Chronological CDC-family result:551,895-byte net physical-pack opportunity

The fixed policy reduces all **249 retained lockfile records from1,123,468 to571,573 bytes**, a **551,895-byte net decrease**. This is a chronological graph simulation, not a sum of independently optimal pairs: each selected representation updates the base graph used by later targets. Every canonical object remains retained exactly once.

The complete original v2group/pack membership remains valid after replacement: **3,259,492→2,707,597 native pack bytes**, also551,895B less. Maximum group65528B, maximum pack248118B; no pack split or new directory is required. This is verified physical-layout arithmetic plus alternate-graph reconstruction, not an executed product save or measured Store allocation.

| Result | Count / bytes |
|---|---:|
| Improved records |152 |
| Worsened records |12 |
| Unchanged records |85 |
| Gross improvements |577,621B |
| Worsening from changed graph/anchor resets |25,726B |
| **Net saving** |**551,895B** |
| Original FULL count |70 |
| Alternative FULL count |43 |
| Newly FULL records previously stored PREFIX |12 |
| Those twelve new FULL records, included in final total |79,971B |
| Final choices |152 similarity,54 first-overlap,43 FULL |

Selected step7 actually grows125,233→145,246record bytes. This illustrates why net chronological accounting matters: better earlier deltas can consume depth headroom and require later FULL resets. That regression is retained in the final551,895B result.

The unchanged policy indexes the preceding large file's chunks using the existing16-byte/eight-minhash signature, at most128entries and1MiBraw predecessor. It compares FULL, the original first overlap candidate, and one minhash winner. FULL wins equal costs; the first overlap wins equal prefix costs. It does not try a second-ranked similar candidate after depth/budget rejection. Targets without a preceding large file retain their original FULL representation. No cutoff, codec, delta depth, cache-size or ranking sweep was used.

## Validation and public-path limits

All nine original large lockfile versions reconstruct byte-for-byte under the final graph and match sealed SHA256. Every original selected frame was also reproduced with the exact pinned native encoder before substituting its representation. All735native objects—not only this family—were decoded and authenticated after249substitutions. No non-lockfile native object depends on a lockfile chunk. Complete-native maxima are4edges,163840raw closure bytes,5lookups,29239encoded-work bytes and171781decoded-work bytes. Strict physical-pack base chronology and canonical role/length are preserved. The retained Store stayed unchanged at SHA256713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4.

The simulator enforces cumulative optional per-target8lookups and512KiBwork while choosing first-overlap then similarity.33candidates hit depth/closure and22hit that cumulative read envelope;345candidate trials were encoded. It preserves observed group counts for record-directory accounting, and the final group/pack size check confirms those memberships fit. It does **not** run the product's actual read-wave retained-frame scratch accounting, original admission batch's remaining512trial/8MiBquotas, public save/Commit rollback/durability, or a new-format reader. Current source has an8lookup optional target counter; no48lookup batch counter exists in this reader. Thus no48lookup qualification is claimed. `OBJECT_PAGE_COUNT=128` is a separate association/read-wave bound.

## Discovery and timing costs

Eight preceding-file indexes require575simulated-graph dependency lookups,1,865,480encoded-work bytes and14,488,535decoded-work bytes. The largest index has37entries and761,954unique raw bytes; largest one-index decoded work3,409,526B. Index construction took3,427,901,376ns in Python/ctypes. Candidate reads consumed955lookups,3,240,549encoded-work bytes and24,577,314decoded-work bytes; these are total alternative-policy charges, not a measured increase over current product execution. Candidate compression took23,844,669ns, FULL comparison compression15,394,838ns, with baseline-frame reproduction measured separately at23,921,501ns. Full diagnostic wall time was8.862s. These timings include different Python/ctypes scopes and are not Rust performance or publicCommit figures.

A prospective implementation can preserve the existing native format and canonical identities. The new work is one bounded per-file candidate-signature index, built by authenticated sequential reads of predecessor chunks and consulted alongside the existing first overlap. Payload reconstruction/hashing must receive an explicit file/operation budget; spare current batch capacity cannot be assumed. Index entries need only objectID+eightu64fingerprints (at most128×96B before container overhead), but building them incurs the read work above. Only one base/target raw pair needs to be live at a time; current native encoder/decoder scratch separation remains mandatory. Public performance, combined read-wave memory, integration rollback and same-Store historical verification remain required before retaining product changes.

Artifacts: `family-result.json` stores every final frame(hex), baseID, record coordinate, per-target decisions/causes, index charges and nine original-file checks. `all-native-verification.json` stores all735canonical validations and80native pack size reconciliations. `family.py` and `verify_graph.py` are executable checks; `protocol3.md` froze the policy. `family-initial.py`/`family-initial-failure.txt` retain the first diagnostic's stopped attempt: it tried to decode an unprocessed target's old encoding after a predecessor encoding changed. The corrected script authenticates the original target under the original graph, then evaluates its bases and new representation under the current simulated graph. No Store was changed by either attempt.

# Conditional CDC similarity experiment: promising fixed sample

On the **same frozen twelve expensive lockfile records**, one content-similarity candidate from the previous file reduces potential complete record bytes **144,549→73,804**, saving **70,745 bytes**. Eight targets improve; four selected candidates hit the existing depth-four limit. All twelve selected similar candidates are outside the original same-offset overlap hints. This is materially stronger than the overlap-only experiment's602B saving, and isolates poor positional correspondence as a real owner in this fixed sample.

| Selected step / target ID prefix | Current record B | Eligible alternative B | Saving B | Result |
|---|---:|---:|---:|---|
| 4 / ec9541e9c8c9 | 14,445 | 1,985 | 12,460 | 7 shared fingerprints |
| 3 / e042070ea2d8 | 13,665 | 2,202 | 11,463 | 4 shared fingerprints |
| 8 / 8b9010976e40 | 12,957 | — | 0 | Winning candidate depth4 |
| 9 / 2c827bc46a03 | 12,832 | 5,925 | 6,907 | 4 shared fingerprints |
| 5 / 2ef8ceb56ec9 | 12,239 | 4,750 | 7,489 | 5 shared fingerprints |
| 8 / e1af325baf6f | 11,802 | — | 0 | Winning candidate depth4 |
| 8 / 11012a914fdb | 11,430 | — | 0 | Winning candidate depth4 |
| 7 / cee502cba1a2 | 11,336 | 1,167 | 10,169 | 5 shared fingerprints |
| 4 / 5a9a92bf262b | 11,175 | 1,002 | 10,173 | 7 shared fingerprints |
| 3 / 53a5c314565c | 10,958 | 4,942 | 6,016 | 5 shared fingerprints |
| 5 / 2056139fdbf4 | 10,855 | 4,787 | 6,068 | 5 shared fingerprints |
| 9 / 3b0682fe96af | 10,855 | — | 0 | Winning candidate depth4 |

The prospective policy was frozen in `protocol2.md`: exact existing16-byte window/eight-minhash signature, preceding-file chunks only, at most128entries/1MiB raw predecessor, greatest shared-fingerprint count >=2, full-ID ascending tie-break, exactly one candidate. Depth is checked after selecting that candidate; rejected candidates do not trigger a second trial. No parameter sweep, size-ranking heuristic or extra candidates were introduced. Authentication/chronology, existing physical dependency graph, raw/depth/read-work bounds and exact pinned encoder settings were preserved. Every alternative roundtripped to the authenticated target.

**Candidate discovery has a cost.** Six distinct preceding-file indexes contained at most34entries and680,660 unique raw bytes each. Index construction required368 dependency lookups,1,930,499 encoded-work bytes and9,869,698 decoded-work bytes across the six files; the largest one-file decoded-work charge was2,966,940B. Repeated group fetches across chunk closures are charged; raw JSON separately lists unique physical groups. Index construction plus hashing took2,480,913,459ns in Python/ctypes; the eight candidate compression calls totaled665,082ns. These are diagnostic microtimings, not Rust production speed or Commit latency. Signature index representation can be fixed and small, but previous chunk reconstruction and hashing are not free, especially for deep chains. The experiment does not prove that spare batch or operation quotas exist in the actual admission path.

All frozen fixtures were SHA-checked; original spans use the shared product scanner; canonical identities use BLAKE3 with LayerFS's object domain. The Store retained SHA256713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4. The same validated1.5.7 static encoder and native parameters as the overlap experiment were reused. Reconstruction used diagnostic dynamic DCtx with authenticated outputs; no product memory/RSS equivalence is claimed.

This result is **a fixed-sample opportunity**, not a70,745B measured Store reduction or evidence that249 records save the same proportion. Existing bases are charged once and remain in their old graph. Selecting alternate representations for predecessor chunks can change descendant depths and downstream choices; summing isolated target wins across the family would overstate certainty. A next family evaluation must propagate the selected graph chronologically and count every final representation once, including no-gain and depth-reset records. No family extension or integration has been run.

Artifacts: `similarity.py` reuses the verified parser/codec/authentication helpers without rerunning overlap trials or changing their results. `similarity-result.json` records complete IDs, per-index work, costs and microtimings. `protocol2.md` and unchanged `sample.json` establish the fixed policy and original sample.

# Experiments toward 40 MB

The owner requested execution of the metadata, CDC-base and SmallContent-framing experiments following the three-agent diagnosis. This directory holds new diagnostic artifacts; the retained measured Stores and product source are inputs, not destinations.

Baseline product: `ee78028ba56741002a627b3a8875d3c888c86708`, restored at source head `1955205db82f4bf84b73efaf60b97c1d508320cc`. Ten-state measured allocation: **49,319,936 bytes**. The target **40,000,000 bytes** requires **9,319,936 bytes** of net savings. Matching Git content alone leaves 45,746,702 bytes; a 40 MB budget then needs 5,746,702 bytes of non-content saving.

Input Store: `/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1/deepseek-ten/host-runtime/store.sqlite`. It is the post-verification Store matching verification-manifest SHA256 `713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4`; its frozen performance/census digest is a different lifecycle. Every experiment checks the input before and after and uses immutable read-only SQLite access.

## Fixed experiments

- **metadata:** actual replacement canonical leaves, namespaces and full-hash index inventory for original layout, inline inode values, and inline values plus direct directory mappings. Semantic verification covers all eleven roots. Matched offline packing is distinguished from the original online selected groups; raw-field and deleted-row projections are not reported as allocated savings.
- **cdc:** twelve expensive lockfile targets selected before encoding, using actual first-admission provenance. Compare the selected base with the existing at-most-three additional offset-overlap hints, with identical codec and chain limits. Extend only if the fixed sample warrants it. This is a target-record diagnostic, not a whole-Store result.
- **framing:** re-encode all 514 SmallContent packs under two explicitly diagnostic grammars, preserving frames and hashes. Exact expected pack saving is 398,604 bytes for compact directory entries and another 265,736 bytes for redundant record lengths. Reader checks cover reconstruction, authentication and malformed inputs.

Each lane writes its protocol before encoding, then scripts, raw results, checks and a report. No post-result parameter sweep. The shared helper tools use the cached BLAKE3 dependency and exact cached product CDC scanner. Their source, binary and input identities are recorded, with runnable known-vector/frozen-boundary checks. Codec alternatives must reproduce existing frames under the pinned 1.5.7 policy before results are interpreted.

## Interpretation

Experimental encoding bytes, compact standalone index layouts and microtimings are not public allocated-Store or Commit measurements. Do not add overlapping metadata/index savings, copy-VACUUM results, or unreclaimed replaced representations. A useful diagnostic informs a concrete versioned implementation; only an actual fresh-Store public ten-snapshot performance/census/same-Store verification/cleanup run can establish its storage and speed outcome.

Compact stable inode identities are a conditional fourth direction if the measured metadata economics remain insufficient. It requires explicit identity scope, allocation, branch/import behavior and index costs; it must not truncate authenticated content hashes or silently change semantics.

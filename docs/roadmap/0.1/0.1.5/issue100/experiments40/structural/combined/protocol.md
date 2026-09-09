# STRUCT-03 fixed combination protocol

2026-09-10, before component results. Source is the previous verified depth-one full157 copy, SHA256 `2c9e44a55042ee0f7989dbfa5e7e88b09beb284ce5fbcfa7682f72d940cfd080`, allocated 98,668,544 B.

1. Audit immutable source object/pack inventory, canonical ID/length membership, nonphysical SQL and page allocation. Copy source into a fresh control and VACUUM once to measure the same lifecycle used for the candidate.
2. Accept components only if their selected physical encoding authenticates every affected canonical object and declares complete graph bounds. Compare all candidate encodings including bases, not only favorable records. If a component loses, retain the control representation and report that decision.
3. Components preserving canonical metadata and content identities can replace their complete original pack/locator population. Preserve every other pack and SQL row. Components changing canonical identities or dependencies require a complete identity/reference migration; absent that, report separately rather than deducting their savings from the combined Store.
4. Build a new offline database with actual required pack rows and full 32-byte object index. Update locators transactionally, remove fully replaced packs, VACUUM once. Check complete physical locator coverage, SQL integrity/foreign keys and unchanged canonical ID/lengths plus nonphysical rows.
5. Verify all157 original snapshot oracles through the actual new reader. Use the previous verifier and original oracle seals, 4 MiB canonical / 8 MiB pack caches; no fixture generation. Report cumulative dependency costs separately from payload caches. Hash candidate before/after and preserve prior stores.
6. Report logical and allocated totals, actual packs by role, SQLite index/overhead, remaining gap to recorded Git56,373,248 B. Label any timing diagnostic, not product Save/Commit latency. Existing product guarantees are not relaxed by an offline result.

No component may claim savings by appending an alternative without reclaiming obsolete bytes. No raw-byte forecast is combined into an allocated result. Preserve every attempt and record a failure before retrying.

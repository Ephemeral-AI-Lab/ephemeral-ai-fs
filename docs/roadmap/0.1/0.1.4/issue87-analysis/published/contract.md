# Issue87 read-only analysis contract

Authority: owner task and https://github.com/Ephemeral-AI-Lab/layerfs/issues/87, 2026-09-08.
Analysis source: discovery worktree `layerfs-storage-v3`, branch
`codex/storage-v3-implementation`, initially clean HEAD
`014c0b9cb5d62bd50ded2052a2604d4aeab76dce`. This is a reporting/harness
checkpoint, not the revision embedded in the measured binary.
Evidence: sibling `layerfs-storage-v3-runs/full157-m45-1`.
Output: this new directory; finalized manifest seals files without rewriting
original evidence. Analysis-tool sources are committed under
`docs/roadmap/0.1/0.1.4/issue87-analysis`.

This work reads sealed receipts and the quiescent post-verification Store.
No replay, candidate sample, recompression, repacking, VACUUM, migration,
original Store write, kernel trace, benchmark/fixture/product change, S3,
M5 qualification or PR merge. No fresh timing pair is collected.
Original acknowledgement allocation is primary. The surviving Store has
verification-created metadata; no pre-verification logical snapshot exists
in the discovered artifacts. Current stat, logical census and copied-file
allocation cannot replace acknowledgement allocation.

Responsibilities: root owns checkpoint receipt tooling, scope/final report,
manifest/publication; physical agent owns filesystem/SQLite accounting;
roles agent owns the single shared streaming pack/group/canonical pass and
spillable graph metadata index; reviewer independently audits custody and
challenges conservation, root closure, role authentication and causal claims.
No duplicate full censuses. All Store connections must be read-only after
writer checks. No raw payloads are emitted. Bounded decoded groups and a
bounded base cache are allowed; analysis indexes contain metadata/edges.

The four accounts are nested: filesystem contains SQLite, SQLite contains
pack BLOBs, pack directories contain encoded groups; decoded records are a
separate representation. Encoded group bytes never receive fabricated
record-level attribution. Logical canonical target bytes are not additive
physical bytes. Signed allocation adjustment is unexplained until attributed.

158 checkpoint rows include Init. CSV null is literal `null`; field contracts
are in checkpoint-fields.json and the associated inventory schemas. Integer
bytes/ns/counts remain unrounded. Measured means direct receipt/stat/decoder
observation; derived means an explicit equation or graph closure; unknown
requires a reason. Estimated claims are not used for measured totals.
All aggregate fields inherit their enclosing unit, population, snapshot,
provenance and status; inventory columns inherit their table's field contract.
Event counters may overlap; candidate alternatives include race losers;
admitted representations count winners. Do not sum nested codec times into
public elapsed or sampled runtime disk into the Store it already contains.

The final conclusion must be TARGET IDENTIFIED, ADDITIONAL DIAGNOSTIC REQUIRED,
or NO MATERIAL OPPORTUNITY ESTABLISHED. Recommend exactly one separately
authorized next experiment/diagnostic with one variable, falsifiable
hypothesis, fixed workload/identity/oracle/environment, counters, correctness,
allocation/timing boundaries, stop rules and retention of negative results.
No optimization is implemented by this contract. Keep #87 open if necessary
diagnostic information remains missing; do not close parent/release issues.

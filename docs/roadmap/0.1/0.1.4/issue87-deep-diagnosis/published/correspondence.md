# Correspondence diagnosis: operation-wide reservation saturation

**A concrete coverage choke is established; its recoverable storage bytes are not.**
All 152 full157 Commit checkpoints reporting correspondence exhaustion finish at
exactly **16,654,272 reserved bytes = 127 × 131,136**. This is the last reservation
that fits under the **16,777,216-byte operation limit**. The remaining five
checkpoints do not report exhaustion. Exec has no correspondence activity.
The **125,254 exhaustion events count cursors, not initially missing targets**.

This is an independent read-only source/receipt diagnosis at product source
equivalent to `bd220ae4a`; it does not run the frozen workload, scan a Store,
recompress records, or change product policy. The source hashes and receipt hash
are in `correspondence-receipts.json`. The actual cursor is exercised with small,
synthetic authenticated in-memory metadata through its existing public Rust API.
Synthetic numbers below are algorithm checks, not measured workload samples.

## Mechanism and exact ownership

1. `crates/layerfs-workspace/src/changes.rs:586` creates one shared atomic
   correspondence reservation counter for the candidate construction. It is
   shared across file producers at `:1260`, not reset for each file.
2. `StableFileInputs::prepare_page` at `:1074` gets the predecessor from the
   base inode table, or from the same path in the base namespace for replacement
   inodes. It serializes that FileState root at `:1182`. A same-path replacement
   therefore does not automatically lose predecessor identity.
3. `FrozenFile::build` attaches the predecessor at `:1461`.
   `objects.rs:2037` invokes `PredecessorCursor::hints` during deferred candidate
   delivery, **before** handing the candidate page to the admission visitor.
4. `objects.rs:2042–2064` reserves **131,136 bytes per optional metadata read**,
   bounded by **1,048,576 bytes per file** and **16,777,216 bytes per operation**.
   These allow **7** and **127** metadata reads respectively, not 8 and 128.
   Reserved bytes are a conservative work allowance, not measured bytes read.
   Neither a cache hit nor a later CAS reuse refunds this allowance.
5. `file/rope/read.rs:439` first fetches the FileState, then the mapping node.
   Even a one-extent nonempty predecessor ordinarily uses two reservations.
   With sequential synthetic one-extent files, only **63** cursors get both
   nodes; cursor 64 consumes the final slot on its FileState and cannot read its
   mapping. Every later cursor fails its first reservation. Concurrent cursor
   scheduling can distribute incomplete starts differently; 63 is a maximum
   for fully initialized two-read cursors under this budget, not a claim that
   the measured run always selected exactly the first 63 files.
6. `read.rs:397` permanently returns no hints after exhaustion. The receipt
   uses `(descriptors, exhausted as u64)` at `:431`. `workspace.rs:668` adds that
   Boolean once per cursor to `correspondence_budget_skips`. One exhausted file
   can have many payload targets, and an exhausted file can contain only
   CAS-reused payload. Neither counts nor bytes of missed unique targets follow
   directly from this event count.

## Discriminating evidence

| Observation | Population, units and status | Consequence |
|---|---|---|
|152 of 157 Commits finish at 16,654,272 reserved bytes and have exhaustion|Original public Commit per-operation receipts; measured counters, derived equality|Operation-wide reservation saturation is present at every affected checkpoint|
|125,254 cursor exhaustion events; 2,575,248,768 total reserved bytes|Sum across 157 public Commits; derived, not persistent Store bytes|Events are not an exclusive eligible-target funnel and reservations are not actual read traffic|
|No Exec correspondence activity and no recorded memory-budget skips|Original public phase counters; measured|This choke is in Commit candidate construction; memory exclusion is not the recorded cause|
|Zero authenticated extent-branch objects in the retained all-history union|Existing final role inventory; measured post-verification logical roles|Retained predecessor maps are single leaves with at most 128 descriptors; their graphs need no more than two metadata reads|
|Actual API synthetic check: 63 hinted/65 exhausted cursors among128 leaf-file attempts, 16,654,272 reserved bytes|Synthetic deterministic algorithm result, not benchmark evidence|Two-read startup cost and shared-slot exhaustion are reproduced without a Store or matcher|

The absence of extent branches is especially useful. The source resolves
predecessors from base namespace/inode roots, rather than inventing a graph from
final pack order. For predecessor roots in the authenticated retained graph,
neither the **4096-descriptor limit** nor the **seven-read file limit** can bind:
a single leaf contains at most128 descriptors and uses only two metadata reads.
Transient predecessor roots outside that graph would require separate provenance;
the report does not infer their absence merely from an empty final staging table.
Even without that graph-scope inference, the receipt equality independently proves
global-budget saturation in every affected Commit.

The synthetic probe also isolates the other policies:

- A 4,224-extent, level-one synthetic tree gets only640 sequential hints with
  the actual per-file reservation policy (FileState + root branch + five leaves
  = seven reads,917,952 reserved bytes). With an unlimited reservation callback,
  the actual cursor instead stops at4096 descriptors. This distinguishes the
  two limits; it does not propose lifting either one.
- A span beyond predecessor EOF yields no hints with no exhaustion. Legitimate
  no-overlap must remain separate from budget exhaustion.
- A target covering eight distinct previous extents receives the first four
  unique overlapping IDs, not a content-similarity ranking. The cursor still
  advances across all eight. This is an intentional bounded correspondence
  policy, not evidence that any later base would produce a useful delta.
- Backward/overlapping requested spans are rejected by the actual API. No
  span-order defect was reproduced.

The leaf-file probe reads12,076 **canonical metadata bytes** from its in-memory
fixture while reserving16,654,272 bytes. Those units are deliberately different:
physical metadata fetching can decode whole groups and FULL anchors. This ratio
must not be reported as a real filesystem read amplification or reclaimable
reservation waste.

## Root-cause disposition

**Proven: global reservation coverage saturation.** Observed symptom: most
Commits exhaust correspondence. Affected bytes:2,575,248,768 reserved work bytes
over the run, not storage savings; exclusive affected-target canonical/encoded
bytes remain unavailable in these receipts. Source mechanism: all files share
127 metadata-read slots, charged before admission/CAS selection. Alternative
explanations for no hints remain genuinely new content, no overlap, unavailable
bases and per-target policy. Discriminator: distinguish each terminal cause with
unique initially missing payload **counts and bytes**, recording how much
correspondence was spent on later CAS-reused objects. Smallest possible future
implementation boundary is candidate delivery/admission ownership; increasing
the cap is not justified merely because it saturates.

**Source-supported inference: pre-CAS work can starve byte-bearing candidates.**
Candidate hints are produced before admission deduplication. The budget can be
spent on chunks that need no representation and unavailable slots cannot then
serve later initially missing chunks. The shared atomic makes distribution depend
on producer progress, not missed-byte opportunity. This is a concrete efficiency
risk, but its prevalence and useful storage opportunity are not measured here.
The smallest candidate change, after diagnosis, would be to defer optional
correspondence until initial CAS absence is known while retaining the same bounds
and normal ordering. That ownership change may require preserving span/cursor
state across filtering; it is not an authorized patch in this report.

**Contradicted as the dominant observed correspondence cause: descriptor-limit
exhaustion on retained single-leaf predecessors.** The graph shape cannot reach
4096 descriptors; the measured cap saturation has a direct competing mechanism.
No evidence here warrants stronger matching, more candidate trials, global
base search, deeper delta chains, or a canonical format change.

No product concepts/code are deleted by this diagnosis. Deferring unused work
could reduce reads/CPU, but additional useful coverage can increase read, decode,
match and encode work. Carrying a filtered cursor may add bounded state and
coordination. Writer-only hint scheduling need not change canonical identities
or reader compatibility; synchronous operation cost and integrity still need a
separately authorized experiment. No removable compressed-byte forecast is made.

## Reproduction

From repository root:

```sh
python3 docs/roadmap/0.1/0.1.4/issue87-deep-diagnosis/correspondence-receipts.py
CARGO_TARGET_DIR=/tmp/issue87-correspondence-target cargo run --offline --locked --quiet --manifest-path docs/roadmap/0.1/0.1.4/issue87-deep-diagnosis/correspondence-probe/Cargo.toml
```

The Python check pins the exact cursor and reservation-source hashes and validates
all157 rows. The Rust probe imports the real `layerfs-content` cursor/encoders,
copies only the small reservation predicate, and asserts startup exhaustion,
no-overlap/order handling, file/descriptor caps, and first-four overlap behavior.
All checks passed. Cargo output lives in a separate temporary target directory;
original evidence and product files are untouched.

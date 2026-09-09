# Mental model of the v0.1.5 hybrid

> **Status:** Agreed design, reconciled 2026-09-09. See [spec.md](spec.md) for the
> technical contract and [workflow.md](workflow.md) for detailed ASCII workflows.
> Only the ten-file/thirty-commit smoke runs during implementation verification.

## Identity and storage answer different questions

A canonical ObjectId identifies complete content with its canonical framing.
Physical encoding describes how those bytes are stored. File names and inode
metadata are outside the content payload.

```text
 Small file version                    Physical storage
 ------------------                    ----------------
 Object A: complete content ----------> FULL A
 Object B: complete content ----------> DELTA(B, FULL base A)
 Object C: complete content ----------> DELTA(C, FULL base A)
 Return to B ------------------------> reuse existing B
```

There is one whole-file CAS payload per distinct small content, not one CAS
identity per patch and not a small-file directory of CDC members. B and C each
reconstruct directly from a FULL base; C does not depend on delta B. A less similar
version can become a new FULL anchor. Exact duplicate content shares its already
selected representation; admission never rewrites a global winner merely to
make a convenient base.

## One size rule for newly prepared content

| Resulting content | Representation |
| --- | --- |
| Empty | Existing compact empty file representation. |
| 1..131071 bytes | Whole-file small CAS object; FULL or bounded DELTA. |
| 131072 bytes or more | Existing CDC chunks and extent tree. |
| Unchanged historical content | Reuse the recorded representation, regardless of today's size rule. |

Both Init and Commit apply that rule in the same construction/admission pipeline.
Init obtains files through discovery. Commit captures the authoritative live
workspace's FUSE/SDK changes. Those inputs differ; encoding, object ownership,
storage, and publication do not become separate engines.

For small explicit edits, Commit may assemble and hash the complete bounded
result. For large known edits, retain unchanged extents and process replacement
bytes. Do not advertise the large-file zero-old-payload-read property for the
small whole-content path. A compact delta can save disk while requiring more CPU
or base reads than an extent-only deletion or truncate.

## Insertions and deletions are normal operations

```text
 Base:    [ prefix ][ old ][ suffix ]
 Target:  [ prefix ][ longer new region ][ suffix ]
                                            |
                             Matching old bytes, shifted offset
```

The codec searches matching sequences within the bounded base. Equal byte counts,
line counts, or offsets are not required. Results depend on similarity and the
fixed codec's matching window; FULL fallback remains available. New small objects
have no canonical chunk-count constraint, while large-file CDC extent-count
contracts remain unchanged.

## Crossing the boundary changes one version

```text
 40 KiB              150 KiB               20 KiB              180 KiB
 Small A ----------> Chunked B ----------> Small C ----------> Chunked D
    |                    |                    |                    |
    +--------------------+--------------------+--------------------+
                      All retained versions remain readable
```

Small-to-large construction streams the resulting content into CDC. Large-to-small
construction reads only the retained result and necessary tree/decode dependencies,
not the discarded large tail. A chunked predecessor is not an eligible whole-file
FULL small base; create/reuse the new small object and start FULL if necessary.
No history scan is needed to find an older small episode.

Choose persistent representation for the final captured content, not after every
live write. Repeated committed crossings incur conversion work. Previously stored
small objects/chunks can deduplicate within their representation; overlapping
bytes across formats do not automatically share storage.

## Reads and retention

A small delta read obtains its FULL base and reconstructs the requested canonical
object, then authenticates it at the storage boundary. A short read may decode
the complete bounded object. A large range read locates the relevant extents and
payload dependencies. Neither operation replays intervening commits.

Saved roots retain canonical content. Delta records also introduce physical base
dependencies, which admission, integrity diagnostics and any future reclamation
must traverse. A base must remain available while dependents survive. This does
not introduce a production GC/repacker in v0.1.5.

## What to measure first

Run only the [ten-file/thirty-commit smoke](delta-encoding-benchmarks.md#first-round-execution-scope)
and its complete 31-state verifier in this implementation round. Record actual
Store growth, save/Commit time and existing resource receipts. Keep all settings
fixed and optimize demonstrated overhead: repeated authentication/readback,
unnecessary SQL transactions, tiny transport calls, broad scans or slow tooling.

The original three-file benchmark and the two later mixed-edit scenarios remain
separate evidence. A successful smoke means the initial implementation loop works;
it does not prove every format/corruption/POSIX corner case or qualify a release.

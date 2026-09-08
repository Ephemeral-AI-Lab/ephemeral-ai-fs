# P readiness: direct native-record reads with complete end-directory validation

Source-only review; no Store, inventory scan, build, encoder, test or benchmark was run. The current prospective whole-group contract is unchanged by this note. A pre-implementation contract amendment is justified if the parent selects the route below; it is not an observed performance result.

## Recommendation

**Use one version2-specific point-extraction helper rather than adding a cache.** Read the pack header and selected group entry, then the native group's count and complete end-offset directory, then only the selected native record. Validate the entire end directory before using the selected range. Version1 compressed/custom-DELTA extraction and its validation behavior stay unchanged.

This is a small extension of existing `objects/read.rs::extract_hint_group` at176–215. It already opens an incremental SQLite BLOB, reads the16-byte pack header and16-byte selected directory entry using `read_at_exact`, validates bounds, and keeps the connection/BLOB guard inside extraction. The extra ability needed for P is selecting a range inside a RAW native group. It does not require a new cache, database, persistent index, format field or canonical change.

Native frames are independently decodable given their authenticated base. Reading neighboring record bodies adds no information required to reconstruct the selected target. That is different from version1 Zstd-compressed groups, where extracting a record requires group decompression; do not apply this shortcut to version1.

## Exact extraction sequence

1. Resolve or reuse the selected locator through existing ObjectId membership lookup. Do not add a duplicate lookup just to enter this helper. Retain the canonical length/ObjectId demanded by the caller.
2. Under one reader/BLOB lifetime, read16-byte pack header, validate wire2/count/pack size, and read the selected16-byte group entry. Require RAW, equal group lengths, legal reserved bytes, selected entry in bounds and group≤65536. This retains current point-level outer validation; it does not claim full-pack directory contiguity.
3. Read the4-byte group record count. Require1..8191, selected record number<count, and checked `directory_length=4*count`; require `4+directory_length < group_length` before allocating/reading the directory.
4. Read **all** cumulative end offsets in one bounded range. Scan them exactly once without building a second decoded-offset vector. Require strictly increasing positive ends, each≤`group_length−4−directory_length`, and the final end equal that record-area length. Retain only the selected record's start/end. Optionally enforce the format's universal record-size envelope during this scan; selected kind-specific bounds are still checked after reading the selected header.
5. Only after the whole directory succeeds, release/reuse its temporary buffer and read the selected record range. Validate kind0/1, raw length0..32768, canonical-length agreement, optional32-byte canonical base ID and exact remaining-frame bounds. No codec runs while the reader/BLOB guard is held. Return the owned selected record plus parsed version facts to the already planned iterative native reader.
6. The native reader performs frame/checksum/window/size validation, strict base chronology/cycle/depth/role checks, bounded decode, exact canonical reconstruction and demanded ObjectId authentication. A corrupted selected directory/record/base is an integrity error; never return a provisional target or silently retry as FULL.

If the optional budget cannot pay the next read, stop before that read/allocation, preserve the work already consumed, and return the existing optional-exhaustion outcome. A mandatory admitted-object read fails on a bound violation. Do not reset/refund counters or retry with a full-group path to evade the limit.

## Integrity scope: what this validates, and what it does not

The complete group **end directory** is validated: selected-range extraction never relies on only two unchecked offsets. Nonmonotonic, overlapping, out-of-range or trailing record-area coverage fails before decoding any requested record. Current `pack.rs::visit_records` at209–248 combines directory and all-record parsing; P should factor/reuse its offset invariants rather than implementing a general second storage decoder.

Unselected native record bodies are **not** parsed or decompressed during one point read. Their invalid kind, frame or checksum need not fail an unrelated target read. Whole-group validation/final census still checks every body. This is a deliberate version2 point-read scope, not a claim that partial I/O authenticates a whole group. The current contract's sentence about parsing every fixed native header must be qualified accordingly if this amendment is adopted.

There is no loss of authentication for returned content: it is still checked against the caller's full canonical ObjectId, and every used base is authenticated. Redirecting the selected range to another valid record cannot silently substitute different canonical bytes. Identical duplicate physical content or other unselected physical residue remains a census/locator-accounting concern. Raw group directories are structural framing, not a cryptographic authentication envelope; do not advertise global group integrity from the selected record's checksum.

The outer pack directory scope already distinguishes point extraction from complete pack validation in `pack.rs:55–56`. Keep that scope: writer assembly and final census establish full-pack contiguity, while the point reader checks the selected entry. Reading all256 outer entries per request would be an unrelated extra requirement and must not be hidden in the32-byte extraction charge.

## Bytes, parser calls and bounded memory

Let a group contain N records and let R be the complete selected native record length. Whole-group extraction reads32+G bytes (outer header/entry plus group G). Point extraction reads:

```text
32 +4 +4*N +R
```

Because the directory and selected record are disjoint subranges of the validated RAW group, `4+4*N+R <= G`. Hence point extraction never requests more BLOB bytes than the whole-group route for the same demanded record, and saves the total lengths of all unselected record bodies. This is a mathematical byte-request bound, **not measured disk I/O or latency**.

A simple implementation uses five BLOB read calls: pack header, group entry, count, end directory, selected record. Existing extraction uses three. The two additional calls and an O(N) directory scan can offset byte savings for tiny/singleton groups. Each chain record rereads and rescans its directory without a cache; repeated directory work is charged, not called free. Do not add a special adaptive threshold before measuring the first fixed route. The existing frame/body is parsed once by the native-record parser; frame decoding happens once per required chain step.

Maximum end-directory allocation is32764 bytes. Maximum complete PREFIX record is33061 bytes under the frozen33024 frame cap. Dropping/reusing the directory buffer before returning the record avoids retaining a64KiB group. Any reuse/growth must charge actual Vec capacity, not just requested length. Metadata descriptors, copied selected frames for the bounded chain, static decoder/DDict workspace, raw prefix and next output remain under the existing1MiB chain/2MiB shared ownership. No memory budget increase is needed to make this route possible; compiled capacity tests remain mandatory.

At most five chain records are fetched. Worst-case byte bounds remain the already declared values because each point read is bounded by its enclosing group:

- `5*(65536+32)=327840` requested group/framing bytes, below the384KiB native encoded cap;
- parser bytes plus reconstructed canonical bytes are at most `5*(65536+32789)=491625`, below512KiB decoded-work cap.

The512KiB per-target and8MiB per-batch existing hint-work budgets remain unchanged. Charge **actual point-extracted directory/record bytes**, plus the outer framing charge and canonical reconstruction work according to the frozen P accounting convention; do not charge unrequested neighbor bytes as if read. Version1 continues charging actual whole encoded/decoded groups. Keep common telemetry's documented group-payload versus outer-header distinction intact; new P counters must identify parser/request bytes separately from native raw decoded bytes rather than silently changing old counter meaning.

This can materially reduce optional budget consumption when many small native records share groups, but cannot promise to prevent starvation. Actual group cardinalities, chain lengths, repeated directory work and shared structural activity determine the result. The8MiB cap can still exhaust; retain that negative result. It also does not repair the separate producer correspondence allowance that can prevent prior hints from being delivered at all.

## Why smaller than a cache

The helper has no retained cache lifetime, eviction policy, key invalidation, shared lock, duplicate cache ownership, warm/cold cache behavior or cache-hit accounting. It reuses immutable selected locators and BLOB range access. Its extra state lasts one extraction: bounded end-directory buffer, selected endpoints and selected record bytes.

A cache might reduce repeated directory/record work further, but it is not required for correctness or for avoiding unrelated RAW record bodies. Choose direct extraction first; do not add both strategies to the first treatment. Full-file/batch reads can still reveal that many point calls are slower than one group read; measure that cost before any later batching/cache policy change.

## Prospective acceptance checks and amendment

Before P code/data, amend only the version2 read-extraction paragraph and budget charging definitions; wire grammar, codec parameters, bases/depth, all ceilings and version1 behavior stay fixed. Direct extraction is part of the declared new physical reader, not a post-result parameter adjustment.

Required focused checks: first/middle/last native records; one-record and many-record groups; malicious count/offset arithmetic; truncated directories; invalid selected ranges; correct byte/body authentication; unselected-body corruption demonstrating the explicitly limited point scope versus full-census failure; exact five-range byte accounting; already-exhausted optional budgets; repeated-chain directory charges; and connection/BLOB release before codec work. Match direct extraction's authenticated target to the reference whole-group decoder on fixed synthetic groups. No retained dataset sweep is needed just to validate the parser.

After correctness, the normal matched P smoke/read evidence must report range-call count, requested bytes, parser work, native decompressed payload, chain depth, public small/full read latency, CPU and memory. A source proof of fewer requested bytes is not an empirical speedup. If added read calls dominate, preserve the result and justify one later change; do not hide it with an unrequested cache in the same candidate.

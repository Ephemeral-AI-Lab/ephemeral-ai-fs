# Experiment 3 result: compact SmallContent framing

**PASS: the two fixed diagnostic grammars save exactly 664,340 pack bytes while retaining every compressed frame, full object/base identity and canonical length.** Across all 514 SmallContent packs and 33,217 selected objects, both round trips reconstructed the original v3 pack bytes exactly. Every selected object's canonical bytes and each chain dependency were decoded and authenticated. No product code or measured Store was changed.

| Fixed grammar | SmallContent pack bytes | Pack-byte saving vs v3 |
| --- | ---: | ---: |
| Existing v3, 16-byte directory entries and 9/41-byte record headers | 34,619,995 | 0 |
| A: diagnostic version101, 4-byte group starts | 34,221,391 | 398,604 |
| B: diagnostic version102, A plus inferred raw/frame lengths | 33,955,655 | 664,340 |

Grammar B saves another 265,736 B over A. This matches the protocol's predictions exactly: 12 bytes per group, then 8 bytes per record. New magic `LFDIAG\0\0` and explicit versions101/102 prevent reinterpretation as existing LayerFS packs. These are intentionally unsupported diagnostic formats, not candidate product schema numbers.

## What was executed

`experiment.py` read the post-verification retained candidate via SQLite `mode=ro&immutable=1`, requiring SHA-256 `713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4` before and after. It retained full32-byte CAS IDs and all32-byte base references, pack/group associations, kind tags and raw lengths. It reencoded both fixed grammars, decoded them back to v3, compared every record and entire original pack byte-for-byte, and recorded original/new per-pack SHA-256 digests in `result.json`.

Canonical authentication used `layerfs/object/v2\0` with exact `LFSO` Bytes framing and `LFS5SML` payload, through the shared BLAKE3 helper linked to the cached dependency. Zstd1.5.7 matches the pinned product codec version. The decoder checks the complete single-frame size, declared output length, window<=256KiB, zero dictionary ID and mandatory checksum before allocating output. It uses explicit static decoder/dictionary buffers rather than unbounded implicit decoder allocation.

All chain walks enforce missing/corrupt-base rejection, chronology, kind1 requiring a FULL base, cycle checks, maximum8 edges,512KiB canonical closure and256KiB retained original encoded-record closure. Maximums observed: depth8, canonical closure521,863 B, encoded closure71,494 B. Maximum static decoder plus dictionary workspace was123,336 B. That workspace number excludes output buffers, retained frames, canonical construction and Python's offline corpus/index storage; it is not process RSS or proof of the Rust reader's complete2MiB budget. Using original record lengths for closure accounting makes the compact grammars conservative.

**41 runnable negative checks passed**, covering old/new/cross-version refusal; zero/oversized group counts; truncated directories/bodies; offsets inside the directory, nonmonotonic or outside the pack; oversized packs; invalid kind; invalid locator length and mismatched inferred raw length; missing/extra/oversized frames; wrong selected locator identity; missing and wrong existing bases; cycles, future bases, kind1->delta violations; depth and both closure limits. Source schema9 is explicitly required before processing kind2 records.

## SQLite geometry experiment

`layout.py` created three fresh disposable complete copies, preserving every one of80,240 object locator rows exactly. It updated only v3 pack BLOBs for A/B, set unsupported diagnostic user versions9101/9102, committed and VACUUMed every copy including the unchanged baseline. SQLite integrity and foreign-key checks passed. The measured source hash remained unchanged. These copies have no LayerFS product compatibility or history/rollback qualification.

| Offline copy, equally VACUUMed | Logical bytes | Observed copy allocation | Saving vs compact original |
| --- | ---: | ---: | ---: |
| Unchanged original | 48,783,360 | 48,783,360 | 0 |
| A | 48,390,144 | 48,390,144 | 393,216 |
| B | 48,066,560 | 48,066,560 | 716,800 |

B's SQLite saving exceeds the664,340-byte pack saving because pack-page slack falls from528,030 to476,592 B and page/framing geometry changes. A recovers393,216 B rather than398,604 B. Independent in-memory full-copy page counts agree with the file copies. These are observations about disposable-copy allocation, **not a claim that a public or live Store now occupies48,066,560 B**. The original49,319,936 B performance allocation includes a different growth/compaction lifecycle; subtracting from that number would conflate framing, VACUUM and filesystem allocation effects.

## Decision

The experiment confirms a modest, deterministic opportunity worth combining with metadata and CDC changes: ~0.664MB of intrinsic bytes, ~0.717MB in this equally compact SQLite comparison. It does not independently move LayerFS near40MB. The next integration can use the existing locator canonical length for preallocation and checksum/content-size comparison, retaining full hashes and all current authentication/resource checks. Actual Rust format fencing, migration/open refusal, range-reader behavior, transaction rollback, cold reopen, history verification, commit latency and un-compacted online allocation remain required product work.

Protocol: `protocol.md`, written before reencoding. Main diagnostic: `experiment.py` / `result.json` (includes514 per-pack digest records). Geometry: `layout.py` / `layout-result.json`. The `offline-layout-*.sqlite` files are disposable unsupported layout artifacts; never substitute them for measured Stores.

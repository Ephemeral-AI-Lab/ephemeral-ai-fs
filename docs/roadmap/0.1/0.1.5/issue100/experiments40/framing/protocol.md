# Experiment 3: fixed SmallContent framing grammars

Written before re-encoding. This is a standalone physical-format diagnostic, not a supported Store format, product compatibility proof, or benchmark.

Input: all 514 v3 packs and 33,217 selected SmallContent objects from retained-candidate-1 post-verification SQLite. Open `mode=ro&immutable=1`; require source SHA-256 `713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4` before and after. Preserve every compressed frame byte, kind tag, canonical object ID, full base hash, pack/group association and canonical length. No compression or candidate selection changes.

Grammar A: diagnostic magic `LFDIAG\0\0`, little-endian version 101 and group count in the existing 16-byte header. Replace each 16-byte `(start, encoded_len, decoded_len, codec)` entry with a 4-byte absolute start. End is next start or pack length. Keep original record bytes. Expected saving exactly `12*33217 = 398604` bytes.

Grammar B: magic as above, version 102, same 4-byte starts. Each record contains kind byte, optional full 32-byte base hash, and exact existing compressed frame. Raw length derives from validated selected locator canonical length minus 23; frame length derives from record range minus kind/base framing. Expected additional saving exactly `8*33217 = 265736` bytes.

Readers accept only their explicit diagnostic version and refuse old magic/version and each other's version. Old v3 parser refuses diagnostic packs. Counts remain1..256, packs <=256KiB, selected group valid and record number0. First offset must equal directory end; starts strictly increase and remain in pack; selected record <=192KiB. Raw length1..131071; frame1..135168. Validate complete Zstd single-frame grammar, content size equals inferred raw length, checksum on, dictID0 and window <=256KiB before output allocation. Reconstruct every original v3 pack byte-for-byte, including all original directories and record lengths.

Authenticate canonical reconstruction of all selected object IDs and every dependency with the product domain `layerfs/object/v2\0`, canonical Bytes outer framing and SmallContent payload. Use recorded BLAKE3 helper from cached pinned sources and pinned-version Zstd1.5.7, recording binary hashes. Retain kind1->FULL rule, kind2 schema9 permission, chronology, cycle detection, eight edges,512KiB canonical closure,256KiB encoded closure and bounded static decoder/dictionary scratch. Offline corpus/index storage is not a product RSS measurement. The unchanged original encoded-record lengths are used to impose conservative closure bounds on both compact formats.

Runnable negative checks: malformed/nonmonotonic/out-of-range starts; invalid/truncated count/directory; pack/record/frame/raw bounds; missing/extra frame bytes; invalid kind; wrong canonical length or requested locator identity; missing/corrupted base; kind1 with delta base; cycle/future base; excessive depth and closure; old/new grammar refusal. Mutation must fail structural validation or canonical authentication. No claim that Python diagnostics exercise actual Rust rollback, migration or cold-open code.

Output: script, result JSON, report, and optionally an explicitly offline SQLite layout copy. No measured source mutation, product build, Docker run or full smoke. Exact byte prediction is fixed in advance and a mismatch fails the script.

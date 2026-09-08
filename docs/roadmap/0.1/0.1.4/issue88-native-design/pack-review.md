# Independent P pack/codec review at5da4779ae

Reviewed committed additive `objects/pack.rs`, the native physical contract and pre-implementation direct-record extraction decision. No tests, builds, encoder, Store access or benchmark was run by this reviewer. Parent reports the focused native gate3PASS, including672 disjoint source/prefix cases; the initial overlapping-operand failure is preserved.

**No blocking defect was found in the reviewed pack/codec layer.** This does not certify the in-progress admission/reader integration or the combined physical scratch budget.

## Grammar and version scope

Legacy `header` remains version1-only; callers must deliberately select `versioned_header` before native record dispatch. Native headers enforce magic, version, pack cap and group count. `versioned_entry` reuses bounded checked offsets/reserved fields, then requires RAW equal group lengths and no oversized exception. Native record parsing bounds raw length0..32768 and nonempty frame≤33024, with exact5-byte FULL or37-byte PREFIX framing. Kind1's32 bytes are canonical base ObjectId, not an offline raw digest.

`native_record_range` validates the complete cumulative end directory: count/range bounds, exact directory length, strictly increasing positive ends and final coverage. It returns only the selected group-relative range. The implementation decision explicitly narrows point-read authentication to directory plus demanded header/frame/dependencies; unrelated bodies are not claimed authenticated. Whole assembly/census still establishes full coverage. This is consistent with the reviewed direct-extraction amendment.

`native_group` parses every fixed native header and constructs the group directory. `assemble_native` checks RAW group and pack/count limits through existing assembly but accepts generic EncodedGroup values; it does not independently parse every group's native records. Admission must construct groups through `native_group` or equivalent validated construction facts, and final census must validate all records. Treat that as an integration invariant, not a blanket claim that calling `assemble_native` authenticates arbitrary record bodies.

## Static codec ownership and validation

Native compression preserves the exact S2 requested setters: level3/window20, content size/checksum, no dictionary ID and zero workers. It allocates the fixed1MiB aligned static CCtx workspace and bounded output. It does not substitute lower CParams to fit. Prefix bytes remain borrowed and alive through the synchronous call; CCtx cannot dynamically grow its static region.

Native decompression first checks ordinary Zstd magic and the descriptor bits excluding dictionary-ID fields, reserved/unused bits. It then verifies complete parsed header, expected content size, checksum,≤1MiB window and exactly one full frame. Skippable, concatenated, trailing and malformed data fail before trusted output. A successful raw decode still needs exact canonical-frame/ObjectId authentication in the caller; checksum is not identity.

The implementation correctly avoids `DCtx_refPrefix`, which would internally create a heap dictionary. Instead it owns aligned static DCtx and by-reference raw-content static DDict regions, subtracting actual DCtx capacity before allocating the dictionary workspace. Combined capacity is bounded by262144 B. The DDict borrows the authenticated raw prefix; context, dictionary, prefix and output all remain alive through `decompress_usingDDict`; no C free is called on static regions. Empty prefix uses no dictionary and empty target uses zero-capacity output with exact expected length semantics.

The three native tests and672-case matrix are meaningful mandatory-fit/equivalence evidence: source-length regime boundaries, empty/tiny/max prefixes, random/repeated/dictionary-magic content, dynamic-vs-static exact frames and round trips. They are not exhaustive arbitrary-content proof. Runtime static bounds remain mandatory, and unexpected inability to encode a valid FULL target is a resource failure, not permission to silently switch codec or increase workspace.

## The preserved first failure matters to integration

The initial fixture passed overlapping raw/prefix storage; native Zstd can shorten an overlapping dictionary range, changing effective prefix behavior. The corrected matrix uses separate allocations matching the intended product flow. `native_compress` itself does not enforce disjoint input regions. Admission must supply the original target canonical operand and a separately owned authenticated reconstructed predecessor payload, both live and disjoint. Do not share slices of one backing object as an optimization without explicitly proving compatibility; the initial failed gate demonstrates why this premise is substantive.

## Remaining integration gates

The pack layer does not decide prior availability, strict earlier immutable pack order, first-delivered-hint fallback, max4 edges, whole-closure/per-target/per-batch budgets or selected FULL-base-only structural policy. The reader/admission owners must enforce them and retain canonical CAS comparison bytes. Native frame choice must compare37+PREFIX length against5+FULL length, choosing FULL on equality; group capacity must be based on the complete FULL bound so substitutions cannot overflow.

Actual capacities of prepared backing, FULL/prefix outputs, canonical comparison operands, per-chain owned frame descriptors, parser buffers and static contexts must fit their simultaneous owners. Successful standalone codec tests do not prove total2MiB admission ownership or absence of hidden allocations in the complete call graph. Decoder context/DDict must be released before the writer encoder phase except for the intended surviving raw prior.

Legacy version1 DELTA prior rejection is explicit and therefore coherent; it is not the same predecessor population as offline S2. Unsupported optional hints can fall back FULL; missing/cyclic/wrong admitted dependencies must fail integrity rather than silently retry. Old readers reject version2; no migration/backward-old-binary-readability claim is supported.

Recommendation: proceed with the existing bounded admission/reader integration and mandatory synthetic tests, without another codec sweep or repeating the successful codec gate absent changed code/inputs. No candidate sample or complete combined-format approval follows from this source-layer review alone.

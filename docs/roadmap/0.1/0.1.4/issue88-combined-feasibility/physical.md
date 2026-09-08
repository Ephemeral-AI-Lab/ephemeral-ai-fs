# Combined S2/S1 physical coexistence: minimal feasible route

Read-only review against the complete updated issue88, the prior next-path report, current pack/admission/read code and retained screen contracts/results. No Store, census, build, encoding, replay or product edit was performed. This report supersedes my earlier diagnostic-only endpoint: the selected direction is a combined representation, with the unchanged-policy payload diagnostic as a bounded prerequisite where delivery remains unresolved.

## Verdict and minimum route

**Coexistence is technically feasible without changing canonical objects or introducing a new database/backend.** Preserve S1's structural wire1 groups and add payload-only version2 packs containing independently framed native FULL/PREFIX chunk records. Store both in the existing `object_packs` table and retain the existing canonical ObjectId→pack/group/record locator table. This is the smallest initial route that preserves structural group encoding while avoiding recompression of native payload frames.

The existing writer already separates content and structural groups (`objects/admission.rs:125–143`): content targets32KiB and structure16KiB. Reuse that separation, but select the native path by exact chunk role plus reviewed file-content provenance, not by the broader current `is_content` heuristic. Other supported objects retain the legacy path. Do not add global similarity search, whole-file canonical conversion, a separate payload database, or a custom persistent index.

Physical separation does not require separate Store files. `object_packs(pack_id INTEGER PRIMARY KEY,data BLOB)` and the existing `objects(object_id,canonical_length,pack_id,group_number,record_number)` schema already describe both packet families (`sql/schema/v6.sql:4–16`). The publisher already inserts several packs and all selected locators in the same transaction (`objects/admission.rs:339–364`). Its CAS recheck, canonical comparison, atomic publication, sidecar accounting and branch publication remain reusable.

## What must change and what need not

| Area | Necessary change | Keep / do not add |
|---|---|---|
|Physical discriminator|Declare payload pack version2 and strict FULL/PREFIX record grammar; new reader accepts both versions and rejects unknown combinations|Canonical chunk framing, ObjectIds, CDC, inode/namespace/FileState/extent schemas stay unchanged|
|Payload representation|Independent native frames on exact existing chunks; no outer Zstd1 pass over those frames|S1 structural record bytes, grouping codec and origin/FULL-base rules retained|
|Locator dispatch|Carry parsed pack version into record reader; interpret payload records through native decoder|Existing full32-byte ObjectId selected index and BLOB range reads|
|Base identity|Use canonical ObjectId for the actual prior reference; reconstruct/authenticate its exact raw chunk|No separate raw-SHA256→ObjectId global index|
|Dependencies|Bounded native prefix closure, strict selected-before-target provenance, cycles/missing-base validation|S1 structural custom DELTA remains depth-one with a structural FULL origin|
|Compatibility|Explicit pack-version rejection and newly created experimental Stores; document old-reader behavior|A SQL schema bump is not intrinsically required just to store new BLOB bytes; no existing-Store conversion|
|Resource ownership|Charge native contexts, frames, prefix/target buffers, metadata associations and complete enclosing-group read work|No unlimited cache or multiplying the offline1MiB closure cap by every pending read|

Current `pack.rs:44–46` accepts only wire1; `record` at108–133 accepts FULL and COPY/INSERT DELTA only. Merely passing a Zstd prefix frame as the old DELTA instruction body is invalid. The version2 parser may reuse the same bounded outer pack directory and group offsets, but must specify its payload record framing and length semantics. RAW payload groups contain already encoded native records; their stored/group length must not be confused with reconstructed canonical target bytes.

A decoder that opens schema6 but later rejects a version2 pack is still rejecting unsupported data, not converting it. If early Store-wide capability rejection is desired, an additional feature marker is an explicit compatibility choice; it is not a reason to redesign all SQL tables. The first prototype should create a new isolated Store, retain the old reader route, and test unsupported-version failure deliberately.

## Why separate payload packs first

A mixed version2 pack could contain unchanged structural groups and native RAW groups, distinguished by record kinds or a group-type tag. This is possible but not necessary. It forces the first implementation to prove mixed-version/group combinations, codec/record coherence and several fallback dispatch paths together.

Payload-only version2 packs keep the legacy structural decoder and assembler path intact and make read/accounting populations unambiguous. Native records can still be batched into bounded groups and packs; they need not become one SQLite BLOB per chunk. Reuse count/offset bounds and pack insertion, with declared capacity calculations based on complete record+frame lengths. Header/slack cost from the additional pack separation is real and must be measured. It is a tradeoff, not a free abstraction.

For an offline coexistence proof, exact S1 group bodies can be repacked without recompression. Physical pack IDs/offsets may change; canonical IDs and S1 base ObjectIds do not. Actual public grouping and candidate decisions can differ because producer order, CAS winners, batch limits and shared work budgets differ. Keeping the S1 policy fixed does not justify importing56,857,102 bytes as the public structural result. Matched controls must disclose these interactions.

## The159,163,199-byte figure has asymmetric framing

The exact standalone S2 account is:

```text
102,306,097 =95,601,473 native frame bytes
            +4,839,072 record headers (86,412 ×56)
            +1,865,536 prefix base IDs (58,298 ×32)
            +16 standalone file header
```

The S1 component56,857,102 comprises compressed structural group bodies, including their decoded-record framing through compression. It does not include a complete outer pack container. Therefore adding the two gives a useful planning milestone with different included framing, not a ready-made common container. A combined format must publish a reconciliation from these two operands to its own actual bytes:

- Preserve native FULL/PREFIX frame bytes where parameters and bases are identical.
- Account for any changed per-record fields. Replacing32-byte base raw digests with32-byte canonical ObjectIds does not inherently change width or native frame contents, but must be authenticated and declared.
- Replace, rather than double-charge, S2's standalone16-byte file header when repacketizing; add actual version2 group/pack directories and version1 structural pack framing.
- Count residual supported objects once. S2 excludes five non-file payload objects present in the original payload census; those cannot disappear because the regular-file screen omitted them.
- Use one common selected locator/index population, not the sum of two diagnostic SQLite indexes. Both diagnostic indexes contain experiment bookkeeping and are not production-size forecasts.
- Add actual SQLite pages, sidecars and filesystem allocation separately at the corresponding snapshot. No allocated159MB Store follows from the component arithmetic.

Removing redundant target digests/depth fields from the experimental56-byte header could reduce size, but is optional later simplification. The first coexistence proof should freeze a conservative explicit grammar rather than claim such reductions without a decoder and conservation evidence.

## Canonical and physical dependency interaction

Structural references still identify payload objects by canonical ObjectId, irrespective of whether the selected bytes come from a legacy FULL group or a native prefix chain. Structural S1 bases must remain exact inode leaves in FULL representation under its own policy. Payload chains must remain exact chunk payloads; reject cross-role bases. There is no need for a cross-role dependency search or a unified delta algorithm.

Both original screens used the same original canonical structural/content domain except S2's deliberately excluded non-file payloads. That makes an offline joint canonical graph possible. It does not guarantee that the different public S1 run's generated structural IDs, extra canonical objects, candidate ordering and observed bases can be combined with the old S2 image by addition. Freeze one common root/identity population and account for any differences.

A base record already selected and logically retained is counted once. If the combined public policy introduces physical-base-only objects, include their records/groups and transitive closure. Do not add every referenced base as another full copy, and do not drop a base merely because it is absent from HEAD. CAS recurrence must select the existing representation rather than silently re-encode it against a more favorable later version.

Old payload custom DELTAs may exist in a mixed-version read scenario. The first new-Store prototype need not create them for native-eligible payloads, but its backward reader should still authenticate them through the existing depth-one route. Define whether they are permissible prefix bases; the simplest first native treatment can require its actual prior to be a supported native FULL/PREFIX chunk and otherwise fall back, reporting this loss instead of inventing compatibility.

## Read and memory feasibility

The existing reader is not already a native-chain reader. `read.rs:218–284` exposes old FULL anchors for predecessor hints; `read.rs:516–518` explicitly rejects non-FULL bases. Native payload reading needs a separate bounded dependency traversal using existing locator/range extraction and canonical authentication. For maximum32KiB chunks and four prefix edges, raw dependency bytes are at most163,840; the offline1MiB raw closure limit is inactive at that unit/depth bound. Enclosing RAW groups, record directories, lookup work and repeated groups add cost beyond raw chunk bytes.

Decode a single bounded chain iteratively, retaining only the necessary current prefix/output plus bounded chain descriptors; sequential chain processing is sufficient initially and avoids multiplying resident closure bytes across a128-object wave. An optional cache is not required. Existing1MiB validation and2MiB physical scratch ownership (`read.rs:382–405`, `admission.rs:160–184`) must be recalculated for this route, not bypassed. The observed S2 contexts541,720B encoder/123,336B decoder and163,840 raw closure are encouraging, but worst-case static context/association/prefix/frame coexistence still needs executable checks.

The measured S2 largest-closure small-read median was340,125ns versus68,000ns for its FULL control in fixed offline probes. The benefit is not free. Public small-range/full-file reads, codec/base fetches, bytes decoded, CPU, memory and foreground cost determine whether the combined representation is worthwhile. No dictionary cache, deeper chain or more aggressive coverage policy should be added to hide a poor first result.

## One minimal pursuit, with a bounded prerequisite

Pursue **existing canonical chunks in native payload-only version2 packs alongside unchanged S1 structural wire1 groups**, using the existing SQLite storage and locator/publisher infrastructure. This is one combined physical direction, not simultaneous canonical or backend redesign.

First complete the already specified unchanged-policy payload delivery diagnostic on its explicitly frozen accepted M4.5 baseline. S1 leaves intentionally have no file spans and must be excluded from missing-span judgments. This prerequisite resolves whether the actual public path can supply the needed priors; it is not the final objective. If a delivery-policy change is necessary, freeze it separately before attributing native encoding benefit.

Then freeze a sequential matched combined screen/prototype with S1 fixed and the exact native codec, grouping, selected-prior rule, depth/resource limits and FULL fallback declared. Reuse valid S1 controls; do not recompute known screens merely to produce more charts. Reject or revise on changed structural output, missing dependencies, excessive frame/pack/index overhead, unacceptable reads or public costs. A negative interaction is the answer, not permission to import the old component number. The first milestone is a measured authenticated combined representation near the stated component budget, followed by actual acknowledged complete Store size—not automatic qualification against134.22MB.

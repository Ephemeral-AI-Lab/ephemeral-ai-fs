# Issue102 verification repair v2

Prospective repair contract after the valid initial2 comparison and before
implementation/qualification. Keep the same original registry,60 CDC-boundary
inputs, seeds, operations, timing and resource limits. No dedup family expansion.
The verifier receipt/profile advances to v2 for representation-aware results;
initial failures retain their original receipts and meaning.

## Source-work receipt

LayerStackInitializationReceipt.scanned_bytes is total source input work,
including a stopped native attempt before fallback. Current direct and fallback
imports incorrectly obtain it from cdc_bytes_scanned. SmallContent reads bytes
without CDC. Count actual input-reader bytes on both paths. Do not inflate CDC
counters, substitute metadata file sizes, or relax exact input-byte assertions.
Existing mixed-root/native import tests and namespace/footprint cases qualify it.

## Canonical regular-file verification

A regular-file root may be a chunked FileState or SmallContent. Authenticate the
entire canonical object before interpreting either representation. Independently
validate SmallContent's Bytes envelope, magic LFS5SML-NUL, version1, and payload
length1..131071. Empty and >=131072-byte files use the existing chunked model.
Metadata value ropes must remain chunked; reject SmallContent in that role.

Validate all original file bytes, types, modes, times, inode/hardlink bindings,
lengths, ranges and canonical identities with unchanged source oracles. Retain
full extent-tree/payload validation for chunked roots. Include SmallContent in
typed object accounting as a distinct role, not a native chunk or fictional CDC
transcript. Flat payload evidence may describe a whole SmallContent object by
its real object ID and bounds; its role must remain identifiable.

Source-only CDC oracle checks remain unchanged. For admitted SmallContent,
compute expected canonical identity from the independently generated expected
file bytes and specified framing, and verify complete content/root identity and
reuse. For chunked files, retain independent actual CDC/extent comparisons and
bounds. Report input CDC properties separately from admitted representation
properties; do not claim that SmallContent was physically split into CDC chunks.
Fresh schema9 imports must use the specified SmallContent representation for
nonempty files below the limit. Released schema7 keeps its native expectations.
No expected byte content may be derived from the observed product result.

## Failure recovery

The published-presentation-failure proof currently returns on canonical verifier
error before presentation recovery, then stalls on cleanup. Attempt the existing
public recovery even if the canonical assertion fails, and propagate the failure
afterward. Never convert a failed assertion to PASS. No deadline increase.

## Qualification

First reproduce the existing native import regression; qualify direct and fallback
source-work accounting after repair. Add product-free SmallContent framing and
role rejection checks, then run selected namespace, footprint, CDC, tiny-stat,
Git and presentation-failure proofs on the exact builds. Preserve failed attempts.
All applicable broad cases/proofs need final exact-candidate qualification after
shared changes. Controls use byte-identical harness/workload, untouched released
product. Only after correctness repairs are stable may measured performance
changes be tested under campaign-v1's selected-pair and no-regression criteria.

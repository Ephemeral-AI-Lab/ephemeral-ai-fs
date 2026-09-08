# Minimal C+S1 integration audit — no implementation performed

Scope: read-only comparison of accepted`eb7050603` against the measured S1 product
source through`3afe76243` in the experiments checkout, and the current C/D source.
No product edit, build, Store read, census or encoding was performed. C's final
verification and authenticated unique-cohort gate remain prerequisites to any
integration execution. Current untracked parent census/report scripts are untouched.

## Smallest port: four production files, existing S1 semantics

The complete old diff also includes tests and cfg(test) spill accessors. C already
contains those accessors from D; **do not reapply them or replace whole files**.
Select only the following S1 production hunks:

1. `crates/layerfs-content/src/object/access.rs`: add the default
   `ObjectStore::put_tree_origin(canonical, origin)` method, delegating to
   `put_owned(canonical)`. Existing stores without an override retain their
   canonical/publication behavior. No new persistent reference is introduced.
2. `crates/layerfs-content/src/tree/batch.rs`: in `Engine::persist`, replace only
   the existing changed-node `put_owned(canonical)` call with
   `put_tree_origin(canonical,page.origin)`. The engine already tracks origin;
   unchanged-node reuse stays unchanged. Existing split/merge behavior clearing
   origin remains. No inode-specific second tree implementation is required.
3. `crates/layerfs-layerstack-store/src/objects.rs`: reuse S1's exact
   `is_inode_table_leaf` helper and ObjectBuffer's `put_tree_origin` override.
   Authenticate canonical input as before; only exact decoded inode-table leaves
   receive `prior_ids[0]=origin` and `has_predecessor=origin.is_some()`. All other
   objects follow ordinary ownership. Do not set file context or the FILE source
   bit on structural origins.
4. `crates/layerfs-layerstack-store/src/objects/admission.rs`: reproduce S1's exact
   eligibility/base-role branches inside the current `DeltaSearch::candidate`.
   Permit authenticated inode leaves in addition to the existing chunk path.
   For inode leaves, selected DELTA origins fall back by skipping the candidate;
   no anchor retargeting and no deeper structural chain. Authenticate selected
   FULL bases and require exact inode-leaf role. Preserve the existing chunk
   anchor path, matcher, grouping, codec, limits and all D observations around
   this function.

The source commits establishing the S1 mechanism are`48284e6d9` (generic origin
hook) and`f71dba9d9` (ObjectBuffer/admission structural policy). Use the reviewed
method-level diff, not an unrestricted cherry-pick of experiment tooling/docs or
replacement of C's newer `objects.rs`/`admission.rs` files.

## What must survive the port exactly

- `CORRESPONDENCE_OPERATION_RESERVATION_BYTES` stays1073741824. Do not restore
 16MiB from the older S1 file, and do not change per-file/grant/matcher bounds.
- D's actual file-owner context remains in complete-file builders, FrozenFile
  and capture. Generic metadata construction stays unmarked. New structural
  origin hints intentionally have no file span and are **not missing-span bugs**.
- The two D observer bytes, strict spill transport, consumed-grant lifecycle,
  file-only eligibility and terminal classification, and post-commit pack gauges
  remain intact. S1 creates no new field or physical format.
- Existing charged layout guards stay. The prior S1 hook adds no per-object
  storage. Its call default must preserve other ObjectStore implementations.
- S1's exact subtype decoder matters: a user payload whose bytes begin with
  `LFS4INT` is still a chunk, not an inode leaf. Never replace the decoder with
  a raw magic-prefix classification.
- Canonical byte buffers remain the same collision operands. S1 already uses the
  existing custom DELTA path; it needs no native prefix or new reader support.

The old `PhysicalStorageReceipt.eligible_targets`, trial and DELTA fields become
mixed chunk+inode-leaf populations under S1. **Do not compare them to C as if all
were payload.** The D `diag_*` file-source subset is the stable payload population
and must remain separate from total structural/custom-DELTA observations.

## Minimal test inventory

Port the already measured S1 test
`s1_inode_origin_survives_delivery_and_delta_origin_stays_full`, adapting only
context/import lines to C. It exercises both memory and forced spill:

- Rebuild an actual inode leaf with the tree engine; expected canonical root ID
  remains exact and origin is the real prior leaf.
- Verify original hint ID and `has_predecessor`, with no file span.
- First changed leaf may select a DELTA against the earlier selected FULL.
- Next generation whose selected origin is a DELTA stays FULL, without redirecting
  to its ancestor anchor.
- Canonical Store readback equals the constructed node, and wrong-role/raw-prefix
  and malformed-node controls reject classification.

Add assertions to that same test (no new redundant suite) that source FILE bit is
unset and D file eligible/missing-span/invalid/terminal counters do not increase
for the structural leaf. The total physical `delta_selected` may increase while
`diag_new_delta_count` remains unchanged; that is the intended role separation.

Run the existing C/D focused tests as regression checks: actual file versus
metadata source, resume/spill/first-owner semantics, first/inherited operation
exhaustion at the named C constant, failed lookup grant preservation, actual
late-race/partial-pack persistence, and charged layouts. Preserve the previously
reproduced broader baseline failures; do not loosen their assertions in this port.

A default-hook equivalence check can reuse existing content/tree canonical tests
rather than inventing a second fixture. Source review must verify that no default
ObjectStore implementation now changes logical writes or acquires extra reads.

## Controls and interaction evidence

The prospective next arm is **C+fixed S1**, against the frozen verified C source,
with identical diagnostic observation and workload scope. Original S1-only
measurements on16MiB coverage remain historical context, not the matched control
for this combined foundation. Record the exact combined source, binary/image and
contract before smoke-first execution; no build/measurement concurrency.

Additional S1 work can compete with the now much larger payload trial population
for existing admission-batch search/read/encoding resources. Group construction,
which origins are selected FULL, physical locality and canonical structural
population can differ. The original S1 21935435-byte group reduction is not a
constant to subtract from C's204745502 pack bytes. An approximately183MB sum is
only an arithmetic planning illustration, not a combined image or an allocated
Store forecast. P remains a separate physical treatment with explicit reader and
framing work; do not implement it as part of this S1 port.

Measure actual payload and structural group bytes, current shared budgets,
file-only candidate/cohort counters, admitted inode-leaf DELTAs/FULL fallbacks,
required bases, complete pack/SQL/allocated totals, and public/read costs. Any
changed file-payload outcome must be explained as an interaction, not attributed
silently to structure. Retain unfavorable foreground costs and the gap to both
159163199 encoded bytes and134221004 allocated bytes.

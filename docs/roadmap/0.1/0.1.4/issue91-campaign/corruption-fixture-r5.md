# R5 corruption-proof repair contract

G0 workspace-corrupt-descendant-compact-v2-proof fails before exercising the
product: legacy SELECT/UPDATE `objects.bytes` raises `no such column: bytes`
against current selected-locator/packed schema. Missing-descendant's locator
removal remains applicable and passed. Preserve the failed receipt; its cleanup
PASS does not qualify correctness or unavailable terminal resource fields.

Reuse the existing v4 integration-test fixture pattern in
visible_missing_and_same_length_corrupt_objects_are_integrity_errors. Read the
chosen canonical object through the public API; close the Store. Install an
exact valid supported RAW singleton and retarget only its selected locator in
one transaction; positively reopen and authenticate the same canonical bytes.
Close, corrupt one byte of that singleton's payload with identical length and
framing, then execute the existing reopened Store/Workspace/exact-EIO assertions.
No arbitrary shared-pack byte, ignored SQL failure, disabled oracle or public
mutation shortcut. This verifier-only fault fixture does not change performance
workloads, timers, native/legacy policy or normal publication.

After source sealing, run the corrected corrupt proof and its unchanged missing
sibling once as focused verification, plus affected existing packed integrity
checks. Both proofs remain members of the complete final226-verifier campaign.
The R5 source and fixture change must be explicit in source/contract applicability;
no old failure is relabeled passed and no native-checksum-specific claim is
inferred from a valid legacy RAW corruption inside a supported native Store.

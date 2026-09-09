# Selected SmallContent candidate chains

Prospective amendment before encoding/measurement, 2026-09-10. The verified
removed-name candidate allocates 50,372,608 B, not near 45,000,000 B. Its content
packs use 38,279,771 B; 8,847 SmallContent FULLs use 21,649,395 frame bytes and
24,370 DELTAs use 11,752,395 frame bytes. The cross-path fingerprint cache still
admits only FULL winners despite the implemented bounded chain reader. This
excludes selected DELTA representations from the same discovery path. Counts
motivate investigation; they are not forecast savings.

Keep the existing 1024 slots, eight 16-byte rolling fingerprints, 128-KiB charge
to the existing admission index, at least two fingerprint matches, deterministic
ranking and one chosen candidate. Register actual selected SmallContent FULL
and DELTA winners after publication and after releasing the Store connection.
Keep only IDs/fingerprints, never raw payloads, unselected private alternatives,
past-session candidates or a persistent/global index. The already-owned canonical
winner supplies its fingerprint without rereading/authenticating it again.

Genuine predecessor/removed-name hints retain priority. With no eligible hint,
authenticate the one cache-selected object using the shared small_predecessor
reader. A FULL emits existing kind 1; a DELTA base emits existing schema-9 kind 2.
An ineligible depth/closure candidate falls back to FULL, without a second cache
candidate or encoding trial. Do not interpret a DELTA as FULL or follow its FULL
anchor silently. No new canonical format, persisted interpretation or schema;
schema 6/7/8 opens remain nonpromoting with their existing writer policies.

Maximum 8 edges, 512 KiB summed decoded canonical closure including the target,
256 KiB retained encoded capacity, 2-MiB active reconstruction and 3-MiB encoding
allowances remain unchanged, including static codec storage and actual simultaneous
buffers. Drop encoder before shared chain acquisition; drop decoder before encoder.
The same reader authenticates each node and carries verified closure facts once.
Prepare FULL once and at most one DELTA; FULL wins equal complete record costs.
No added retained base representation or unreclaimed-byte subtraction.

Selected locations are stable under the existing admission session. Required
missing/corrupt dependencies remain errors. Transitive physical retention and
late exact-CAS handling are unchanged. Only actual winners register, final-batch
registration stays omitted, rollback invalidates the owning session, and cache
storage drops with it. No long Store/cache lock spans fingerprinting or codecs.

Extend the existing focused selected-candidate test with a second cross-path
target based on the first selected DELTA, assert kind 2/actual base/exact bytes,
and preserve late-CAS/rollback checks. Then build affected host/image artifacts
and run one fresh ten-state performance, frozen census, exact same-Store
verification and cleanup. Reuse unchanged controls. Keep prospective <=10%
speed/resource criteria and separate 8-MiB RSS allowance as working comparisons,
not release gates; report the preceding Commit regression and extra fingerprint/
chain-read cost. Full157 remains deferred until stable verified near-target.

The metadata-only nearest-size ranking study remains unimplemented: a unique
size winner covers 3,654,587 current FULL frame bytes but several largest witnesses
select unrelated or much smaller files. Uniqueness is not evidence of similarity;
do not turn that coverage into a savings forecast or add a path/size sweep.

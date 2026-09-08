# D preparation decision before public samples

Focused D test build at ef7d733a2:7 passed (six new D tests plus one existing
matching diagnostic test). The full Store-library suite is not green: D49 passed,
9 failed,1 ignored; accepted control53395642e (eb7050603 plus only cfg(test) spill
accessors)43 passed, the same9 failed,1 ignored. Seven substantive messages match;
late-direct output has different generated IDs. Multi-batch failure retains19,572
accepted versus19,573 D objects, both expected0. This one-object variation remains
unattributed; it is not an exact numerical-equivalence proof or a harmlessness claim.
The unchanged source retains earlier admitted batches on final publication error;
no failure-path cleanup optimization or durability qualification is attempted here.

Independent source review and required D tests support ONE normal-path deepseek-five
preflight, followed by full157 only if actual diagnostics/correctness/custody/cleanup
pass. Retain the baseline failures and stop on actual run failure. Do not weaken
assertions, raise limits or rerun tests to obtain matching values. No full-suite
PASS or release claim. This decision is prospective, before public D samples.

Actual owner-size probe (one matching test executed) records DeferredObjectStore480,
ObjectBuffer496, authenticated object216 and PhysicalHints160 bytes. Charged hint/
authenticated/prepared layouts have compile-time equality checks. The new constant
owner context and fixed140-u64 receipt/counter memory are observer costs, not zero
cost. No policy capacity expressions depend on the owner context field.

Preparation failures are retained: first compile encountered the known private
spill-index test fields; cfg(test)-only accessors repaired test access. An analysis
validator closing-brace typo was corrected before its self-check passed. A first
owner-size probe accidentally selected zero tests after the accepted-control build
had reused the same Cargo target directory; that probe is invalid, not a PASS.
The shared Store-package dev outputs were cleared and D rebuilt before the corrected
one-test size probe. Do not share Cargo target outputs across these checkouts again;
normal D release builds use this checkout's own previously unbuilt release target.
The original D7-test and full-suite results preceded the control overwrite and retain
their actual executed-test evidence. No benchmark sample was collected during this
preparation. Raw logs and source custody are retained underissue88-diagnostic-custody-1.

The140-field validator self-check and source/serializer field-set equality pass.
Its graph/locator validation still requires the actual D snapshot and upstream
proof after the future run; synthetic PASS is not claimed to validate that history.

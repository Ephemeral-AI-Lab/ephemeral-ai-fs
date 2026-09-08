# P preflight and counter populations

The combined source has passed13 native tests (including the672-case disjoint
codec matrix and static exhaustion classification),7 diagnostic tests and1 S1
origin/cohort test. Six reader and three actual admission tests also passed as
focused development checks; these overlap the13 and must not be added as unique
tests. The native-aware census fixture passed; no retained Store census executed.

The full Store suite remains FAILED:62PASS/9FAIL/1ignored. Nine failing names match
accepted/D; independent comparison and the additional targeted assertion detail
are retained. Seven printed panic payloads match; unexpected initialization Ok
variants recur; final cleanup residues P19571/D19573/accepted19572 are not exactly
equal and their difference is unattributed. A previously opaque failure was
instrumented only in the assertion message: P and accepted both return unexpected
Ok. D actual value remains unavailable. No assertion, failure or cleanup gate is
waived; this is a scoped research continuation, not release/fault qualification.

Preparation failures remain: codec test1 aliased prefix and target, corrected only
in fixture operands at5da4779ae after pinned-source overlap analysis; reader test1
had a concurrent timestamp-only temporary-directory collision, corrected to
exclusive directory creation plus atomic sequence. No public samples have run.

Independent source review found and reconciled actual simultaneous ownership:
ordinary/native prepared pack capacities remain charged in both lanes; all pack
pointer slots are reserved once from total input count; actual lane vector
capacities replace the dropped original Vec; group/pack copy peaks and a fresh
post-flush/post-prior encoder phase are reserved before allocations. Canonical
comparison capacities are tracked in the6MiB data owner. Only the existing
constructed v1 RAW singleton has its data-owner exemption, never native packs.
The2MiB physical and6MiB data ceilings remain fixed. Optional native workspace
exhaustion selects FULL; mandatory FULL exhaustion and integrity errors stop.

Native objects are partitioned first and retain canonical order within the lane;
legacy/S1 follows in its same canonical order. Both share existing read/trial
budgets. This is the concrete pack order for the declared new physical treatment,
not an offline source-history sort or parameter sweep. Final CAS always compares
retained canonical bytes, including native FULL; first selected representation wins.

##38 additive counters, identical projection in both arms

Every counter is an integer phase interval sum. Units follow the suffix:ns,
count/calls/edges, or bytes. Depth0..4 fields are completed-chain histogram counts,
not maxima. No per-file bytes/path/ObjectId logging inside timed operations.

- native_record_fetches/request_bytes/parser_bytes: actual native extraction work;
 requested bytes include32-byte outer header/entry, parser bytes count actual count/
 end-directory/record bodies. Repeated fetches count again; partial budget stops
 retain actual work. Unrequested neighboring bytes are not charged as reads.
- native_raw_decoded_bytes/decode_calls/decode_ns/dependency_edges and depth0..4:
 actual native reconstruction; calls/time include failures, raw bytes only completed
 outputs; histogram only completed required/optional chains. Legacy FULL bases use
 the existing legacy decode counters. These are nested costs, not extra public time.
- native_full/prefix_encode_calls/ns: codec attempts. native_full/prefix_frame_count
 and frame_bytes: successfully produced complete frames, including alternatives
 later rejected or losing admission races. Failed codec attempts have no frame.
 These fields cannot establish durable savings or infer winning headers.
- native_fallback_{no_hint,unavailable,legacy_delta,role,depth,budget,full_wins}_{count,bytes}:
 one terminal FULL-preparation reason per missing native target; bytes are canonical
 target bytes including21-byte framing. They precede final CAS and may race.
- native_admitted_{full,prefix}_{count,bytes}: successful transaction winners only;
 bytes are canonical, not compressed. Existing diag_new_delta includes PREFIX on
 P; diag_new_full includes native FULL. Six D delivery classes remain the same.
 P diag_terminal_no_delta additionally means native complete-frame FULL wins;
 native_fallback_full_wins distinguishes this from legacy raw matcher rejection.

Generic full_alternative_bytes/mixed_alternative_bytes remain legacy-only.
selected_encoded_bytes includes native RAW group bytes, so comparing those generic
A/B fields against total selected bytes is invalid. Native frame attempts,5/37-byte
record headers, group directories and actual pack framing must be reconciled from
native-aware inventory. Counterfactual FULL pack boundaries are not assumed equal.
Generic encoding_calls/ns covers both codecs; native fields are subsets. Existing
encoded_read_bytes counts actual group-body bytes (native point ranges included),
excluding outer framing. decoded_read_bytes retains legacy full-group meaning;
native raw outputs and parser bytes are separately named above.

Source interfaces/resource guards and allnegativepreparations freeze before normal
host/image builds and fresh matched smoke samples. No runtime policy was selected
from a retained native dataset.159163199 encoded bytes remains hypothetical until
one combined public result;134221004 allocated bytes remains separate.

Reader integration inequality: existing sum(validation_reserve)≤1MiB includes
perwave512B/target associations and legacy pending instruction bounds; the new
sequential native chain uses the other≤1MiB. Existing request/group maps remain
in the old association class, and the first native extraction moves into its
chain without duplicating payload. Both reservations together remain≤2MiB.

Build custody: hostd4f26f0d1 is clean; imaged4f26f0d1 has the broad checkout dirty
flag because two census analysis files changed before its build. Product/harness
source seal f647323ee0c412048b7f1f063a3b94250d2606d9d613105fab56dfd21d754577
and product seals match normally. The producer-relevant dirty patch is empty;
the complete analysis-only checkout patch e27d7e0399ac48e55ac13c8f6a1aabaf665c0950e753481f6b6ca8398470a43d
and timestamps are archived in build-custody-qualification.json before samples.
No clean-image claim, identity relaxation or unnecessary rebuild is made.

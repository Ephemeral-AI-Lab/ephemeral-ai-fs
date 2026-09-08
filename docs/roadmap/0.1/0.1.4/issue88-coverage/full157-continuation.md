# C full157 continuation decision — before full C samples

Three prospectively frozen fresh D/C smoke pairs completed in order, with33
historical verifier mappings per arm, normal cleanup and140-field counter checks
PASS. These are single-pair development observations, not release qualification.
No sample is discarded or repeated for a nicer result.

DeepSeek-five: limited-empty canonical bytes1,718,547→0; grants508→890;
file DELTAs266→478. Logical SQLite3,158,016→2,772,992 bytes saves385,024.
Allocated4,202,496→3,158,016 saves1,044,480, of which659,456 is the changed
signed allocation adjustment, not attributed encoded savings. Commit sum
283,939,250→351,705,291ns (+67,766,041ns,+23.87%). Worst relative step3 rises
51.75%/+30,396,416ns; largest absolute step5 rises31,409,499ns/+47.32%.
Exec+Commit sum1,333,454,791→1,270,494,582ns is retained but is not used to
hide Commit regressions or claim a repeatable causal speedup. Host phase CPU
418,144,458→474,234,621ns (+13.41%).

SDK text public sum rises36,755,999→40,396,627ns (+9.90%); its logical/allocated
sizes and grant counts are unchanged. Other non-DeepSeek public sums vary within
about7%, with unchanged grants. Small-files logical and allocated208,896→212,992
bytes is a4,096-byte negative observation; no new coverage work occurred (one
grant in each arm). It is not silently removed or attributed to C without evidence.

Decision: ONE unchanged C full157 performance+verification run is justified.
The smoke exhibits the predicted delivery mechanism and meaningful logical saving
with a modest absolute Commit cost, while the full-history3,763-cursor case is
not represented by five checkpoints. Retain every negative value and source
limitation. The roughly30% guidance is not waived as an individual-call gate or
converted into a new acceptance rule. This decision tests history applicability;
it does not accept the1GiB allowance for production or guarantee further savings.

Freeze the already built host SHA256
54b7378c7b641b2222d53c516c836c55a6339ccae4d7f1c56465f890eabf9e1a,
product seal3b960f78bd8804a6188294d60ef186ad637b959905aebf459650c03a3e013fac,
source sealb39ba00c2c07c89aeda58e0bfcb28e1f9b8d928bff5238d2977067717a70c04a,
producer97f1131c846384af0470cb178717e38c901bbe6e. Image ID is bound by the
pre-execution expected manifest. Current later report commit is not that producer.
Same1GiB operation allowance, other limits/codec/CAS/order unchanged; no S1/P.

Expected run directoryissue88-C-full157-1. Preserve300s steps,4h phase budgets,
original topology/memory/disk/free-space/cleanup limits, source/workload/oracle
checks, one final pre-verification snapshot and the same158-row graph/cohort gate.
No extra cap increase, deadline extension, tuning sweep or failed-arm erasure.
D full157 is a historical diagnostic comparison, not a fresh paired timing control.
If increased coverage moves losses into existing search/anchor/group limits,
report that bottleneck rather than changing it mid-run. Conclude retain/reject for
this research foundation from measured size and costs before the combined stage.

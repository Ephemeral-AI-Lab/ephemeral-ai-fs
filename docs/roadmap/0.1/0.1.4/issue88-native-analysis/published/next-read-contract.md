# Prospective depth-stratified public-read diagnostic — not executed

One next action, requiring separate prospective authorization. Retain the completed
C+S1/P implementation unchanged; no replay, re-encoding, anchor-policy change,
codec/budget/cache modification or optimization is part of this diagnostic.

Hypothesis: actual depth1–4 PREFIX dependencies retain acceptable bounded public
range/full-read cost against matched C+S1 ranges, beyond the existing depth0-only
probe. A material public latency or resource regression concentrated in deeper
strata, correctness failure or violated bound falsifies that hypothesis. This is
a cost diagnostic, with no expected storage gain and no invented storage gate.

The only treatment variable is the already frozen retained representation:
control C+S1 producer2753453933c55ed7f93eb21619c235668a01ef4c versus native P
d4f26f0d16f0f91c1f75767cf699012b8794ac01, with the exact host/product/image seals
in the completed experiment schedule. Use the same frozen157 states, metadata,
source trees/oracles and container2CPU/2GiB/no-swap/256PID environment; host sampled RSS remains bounded at8GiB. If these frozen
artifacts cannot be authenticated, stop; do not silently rebuild a different arm.

Before observations, use existing authenticated inventory/graph metadata to fix
one source checkpoint/file/range for each candidate selected depth0,1,2,3,4.
Use deterministic source-index then path-byte then offset ordering, subject to
regular-file,4096B available range and at-most8MiB containing-file limits. Selection
is based on depth/size/role, never latency. Bind exact source SHA/tree/oracle,
LayerFS mappings, candidate selected locator/dependency proof and matched control
bytes in a sealed cohort manifest. If any stratum has no eligible sample, stop
and record that cohort limitation before timing; do not substitute later.

For each of five strata perform4096B range and full-containing-file public reads,
three repetitions per arm:30 observations/arm. Order arms C/P, P/C, C/P within
stratum/operation. Freeze the complete schedule before the first observation;
no best-of or outlier replacements. Reuse the existing standalone public SDK
probe and normal container lifecycle, with only the necessary authenticated
cohort-input adaptation. No second benchmark family, private Store read fast path
or per-read decompression scanner. One logical disposable Store copy per arm,
not one copy per checkpoint/read. Close normally; preserve original allocation
and original seals; copies have no allocation-equivalence claim.

Time public Exec through terminal byte-count output drain. Run digest verification
outside that interval against the frozen oracle; never log raw file bytes. Record
setup/fork/mount, read Exec, digest, End, cleanup, case/invocation and observer time
separately. Freeze any required phase-counter exposure before measuring, with
strict source/product identities; whole-session receipts must remain labeled as
such. Existing direct native fetch/request/parser/decode/raw-byte/depth/edge
counters must establish the selected read population actually exercised; a
candidate read with no expected chain activity invalidates that stratum, not a
zero-cost observation. CPU/I/O phase deltas, process RSS/current/lifetime peak,
container sampled total/peak memory and limits retain their actual scopes.
Unavailable counters remain null. OS-cache state is uncontrolled and disclosed;
fresh host/container does not imply cold filesystem cache.

Correctness checks: exact returned count/digest, all referenced base/target
ObjectIds and depth/closure bounds, equal matched state/metadata, successful
CleanEnd with unchanged logical namespace, no unexpected publication, complete
owned-container cleanup and unchanged original seals. Keep depth≤4, rawclosure≤1MiB,
reader encoded/decoded/ownership limits and existing synchronous public path.
Do not loosen any bound or add a cache to pass the diagnostic.

Stop on identity/cohort/oracle/reconstruction/cleanup failure,30s read/digest operation,
4h campaign,120s lifecycle limits,8GiB sampled host RSS, frozen container limit,
16GiB scoped runtime,32GiB owned outputs or50GiB free reserve. Resource breaches
remain failed evidence. No perf rerun or replacement sample after failure.
Report every raw observation, medians and absolute worst-case deltas with the
small sample limitation. Owner's roughly30% foreground guidance is interpretive,
not an invented automatic pass threshold; short calls require absolute-time
judgment. Retain negative results and decide retain/revise/reject explicitly.

The diagnostic ends with a read-cost decision. It neither changes storage nor
authorizes the possible depth-cap FULL-root substitution, global predecessor
search, deeper chains, migration, release qualification, rollout or PR merge.

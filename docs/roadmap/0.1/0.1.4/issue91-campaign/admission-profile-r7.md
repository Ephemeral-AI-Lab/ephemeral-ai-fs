# R7 prospective residual admission profile

G2 headline diagnostics (clean c71eb2746, product5f30d03e8) retain all four
performance observations and passing independent proofs. Namespace100000 is
12,462,528,625ns, still far above published2,603,162,083ns; transactions fell to
1,165 but pipeline wall remains12,108,576,792ns. SQL commit sum1,887,157,420ns is
nested within pipeline/final work; producer blocked sum81,462,585,313ns overlaps
across eight producers. Metadata cache100,984 hits/16 misses and final root
325,613,083ns do not explain the residual. Payload500 is3,868,836,500ns, also
unresolved. Do not discard these valid observations or call the repair sufficient.

Next use the same sealed G2 binary/image and unchanged registered namespace10000,
seed1, fresh output, standard collector/public timer/proof and existing diagnostic
nonce91. Capture one macOS `sample <infra-run PID> 1 1` stack profile during the
performance process. This is an instrumented diagnostic with no timing speedup
claim. Preserve exact PID/process command, profiler output/errors, operation and
resource receipts, independent verifier, cleanup and source identity. Profile only
our child; one coordinator holds the usual runner measurement lock. No overlapping
build or independent workload.

If the one-second profile cannot capture the admission consumer sufficiently to
distinguish SQL insertion/index/scratch work from encoding/pack construction,
repeat the profile at the original namespace100000 scale once, five seconds at
one-millisecond sampling, also diagnostic-only. No directory-shape variants or
fixture changes. Then freeze a bounded source repair and meaningful failing check
based on actual stacks/counters. Requalify both headline operations on each new
candidate; retain G0/G1/G2 and every failure. Existing severe/strict gates remain.

User's storage constraint remains binding: do not trade away v0.1.4 native
FULL/PREFIX packed-storage savings for speed. Keep format/authentication/retained
dependencies and codec selection contracts. Report allocation alongside timing
for every new observation and full157 equal-state storage on the final candidate.

The namespace10000 profile captured724 main-thread samples,689 in Init. It shows
SQL publication/commit plus native encoding, but this fixture has too few objects
to trigger the16MiB admission seen/available indexes' spill threshold. The large
case's422k objects crosses that threshold. Therefore execute the predeclared
100000 profile once, delaying4s after its observed public-operation process launch
then sampling5s. This specifically examines late admission/spill behavior absent
from the small reproduction; it is not a replacement timing observation.

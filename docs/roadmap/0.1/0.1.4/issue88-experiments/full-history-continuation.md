# Public S1 full-history continuation decision

Recorded after the frozen single-pair smokes and before the full157 candidate.
All three approved smokes and33 historical mappings pass correctness andcleanup.
No initial smoke is rerun or discarded. The measured S1 offline21,935,435-byte
structural reduction concerns the full history's large inode pages, not the tiny
single-file leaves in frequent-edits; a full-history run is needed to determine
whether that opportunity survives public admission/budget/placement behavior.

Observed smoke limitations: DeepSeek-five final allocation3153920→3162112 B;
small-files208896→204800 B; SDKtext77824→135168 B with bothlogicaldatabases77824 B
and46objects/62333canonicalB. The57344 B difference is a signed filesystem
allocation residual, NOT57KB additional encodedmetadata. Return-to-A SDKtext
elapsed6.728→13.231ms; extra matcherwork~0.477ms and anadditionalrejectedstructural
alternative are observed, but SDKedititself also increases2.335→5.278ms before
Commit, so the whole difference is not causally attributed to structuralencoding.
Other cases show roughly14–22% operation-sum differences; none establishes final
three-pair qualification. Historical numericalgate misses remainexplicit and are
notreinterpreted asPASS. Broaderowner experimental speedtradeoff permits research
on this candidate; it does not make those regressions acceptable forrelease.

Decision: proceed with ONE unchangedcandidate full157 performance+verification,
normal existingcontract/identity/limits, usingexactcandidatebinary89fc4cb9c87b4d4eb774f1747d1a6c5f3ea1c03fb0571864c6f9bdb5477f4869,
productseal1649675c68a853f8a9da7debd36e30ba7cacd883c016e842dcfe99ba34f1e91f,
image sha256:d95da176d7deca0d7ab9f5d7cd05fe983baf922ed00c25dd033e6e5bbb44cdbd.
This run evaluates applicability of the history-wide structural hypothesis and
measures fullallocation/canonical/oracle totals; baselinefull157 is historical,
not fresh timing pair. No claim134MB, combinednativeprefix improvement, M5,
release,smokegateacceptance or rollout. Preserve all earlierfailures and samples.

Expecteddirectory runs/issue88-s1-full157-1. Original full157 immutable.
Stop on integrity/oracle/provenance/resources/cleanup failure. No policychange,
no timeout extension, no extra seed or adaptive repeats. Complete new performance
receiptmanifest before normal samebinary verifier; current postverifylayout is
not substituted forackallocation. Follow originalrunner verificationcustody strictly.

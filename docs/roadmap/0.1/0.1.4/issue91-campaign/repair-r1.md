# R1 prospective diagnostic and repair loop

G0 remains sealed at f484a79861a11c5100ad6c1ee41df59af7648601. Its complete
ordinary collection continues from its immutable source checkout. Root working
code changes do not alter that source or replace a recorded observation.

Confirmed ordinary payload-create-500m operands: published3068249542ns,
G04051680834ns (+32.05%); Commit1941935000→2930548000ns (+50.91%). HostCPU
2361745792→3612498126ns (+52.96%). Exec1112948709→1105983500ns is flat.
Allocation637599744→536875008B (-15.80%). These are simultaneous storage/time
outcomes, not an overall PASS. Public canonical population remains27223 objects/
525955698B. Transactions127→1800 and nativeFULL26995 calls/701507931ns identify
admission/codec costs; zero admission queue time and no dependency reconstruction
reject contention/dependency depth as causes for this creation case.

Namespace100 readiness8901875→24566167ns is corroborated by ordinary24644250ns.
Fixture/content/operation match. Transactions2→18. Namespace100000 ordinary
2603162083→13565385375ns needs separate scale analysis; total InitCPU is broadly
flat despite elapsed growth, so worker/admission overlap must also be considered.

R1 intervention: reuse only the existing bounded1MiB static encoder allocation
sequentially within native admission, keeping the identical setter/frame sequence.
Release before predecessor reconstruction; charge retained scratch during group
assembly or release it if the unchanged2MiB bound cannot fit. Final assembly
releases it. No global cache or codec/selection/grouping change. Independent
source review confirms encoder/prefix lifetimes and unchanged physical gates.

Before first repaired measurement: run new reused-context byte-equivalence/
prefix-reset/error recovery check, existing native codec/admission/read tests and
all original direct/shared memory-bound tests. Save compile/test failures. Build
and seal clean G1 once. No G0 passing sample is replaced.

Diagnostic schedule after G0 ordinary performance AND independent verification:
1. G0 namespace100 fresh diagnostic with existing
   LAYERFS_INITIALIZATION_DIAGNOSTIC_NONCE=91; one seed1 sample and independent
   proof; retain pipeline/SQL counters. No changed timer or substituted workload.
2. G1 namespace100, payload-create100m, namespace100000, payload-create500m,
   one seed1 selected sample each in that order, each followed by independent
   verification. Standard300/310/600s performance/setup and45/59s verifier
   budgets. All are diagnostics; they do not fill the terminal campaign matrix.
3. Compare codec CPU/time, SQL/transaction counts, admission blocking and operation
   totals to G0 and immutable published operands. If severe residual remains,
   isolate batching/validation-wave ownership as R2 before terminal collection.
   Do not raise the frozen reader reserve,2MiB physical/6MiB canonical bounds,
   codec settings, original public ceilings or deadlines to get a pass.

Final combined repair source reruns all affected ordinary families and verifiers,
additional smokes/full157, and affected phase1 quality/correctness/resource/live/
compatibility guarantees. No partial-source average or diagnosis closes a severe
ledger item. All raw attempts and new identities remain separately attributed.

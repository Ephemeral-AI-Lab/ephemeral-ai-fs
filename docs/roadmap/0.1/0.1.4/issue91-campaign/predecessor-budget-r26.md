# R26 preserve predecessor correspondence within producer budgets

R25 completed both157-state performance/historical-verification arms, both158-receipt
validators, and storage accounting. Content/history passed; candidate storage failed:
318803968 acknowledgedallocated bytes versus218116096 control (+46.16%). Both4KiB.
All86412 file-eligible objects/705162813 canonicalbytes selectednativeFULL; PREFIX
attempts/admissions werezero. All134997 attachedpredecessor cursors were memorydenied,
whilecontrol hadzero denials and269994 grants. Preserve this failedcandidate and all
original observations; do not accept the storage regression as a speedtradeoff.

Cause: parallel Workspace output partitions divide1MiB among producers; predecessor
correspondence requires576KiB plus32KiB outputheadroom. Anycount>=2 denies every
correspondence cursor. This predates R15 and does not originate in Init coalescing
or4KiB pages. Existingpredecessor roots survive planning but cannotreceive grants.

Minimal correction: record whether the existing taskplanning lookup found any
regular-file predecessor; cap that plan's alreadybounded worker count with min(1).
Keep everymemoryguard/reservation and zero-worker check. The one1MiB producer can
reserve576KiB and retain448KiB outputownership. Mixedplansserializeallfiletasks;
pure-new/no-predecessor Workspace plans and nativeInit keep existingparallelism.
No native wireformat, dependencyordering, SQLite pages, importedlibrary source,
versions or features change. No additionalcache, workerpool ormemoryallowance.

The focused regression requestsfourworkers for two rewritten predecessorfiles;
it failsbefore thefix (no correspondencegrants), thenpasses with grants, no memory
denials, nativePREFIX records and exact current/retainedfile bytes. Rawbefore/after
commands/logs/patches are in r26-before-01 and r26-after-01 underissue91-runs.
Run affectednativechecks, formatting andClippy before newcandidate collection.

The user explicitly selected the bestavailable candidate and requested the full
benchmark plusDeepSeek157 rerun. Preserve originalperformance severity labels and
report acceptedperformance tradeoffs separately. Correctness,4KiB and>=10% equal-state
acknowledgedstorage savings remain requirements. After thisstoragefix, freeze/build
and run a freshcandidate157, then198performance cases/226routineproofs and required
supplementals. No further optionalperformance optimization in thispass.

Reuse the unchanged passing R25control performance/snapshot/census/accounting and
all157historical proofs. A newfrozen R26schedule references the originalschedule/hash
and preserves the entirecontrolarm entry. The resource reporter validates thatold
observerchain against its originalschedule/decoder, rather thanrewritingcustody or
remeasuringcontrol. Candidateobservers remainbound to the newfrozen schedule.
The report adaptation changes evidencebinding only; accountingequations andstorage
thresholds remainunchanged. Finalsource/host/image/decoder andhelperseals mustprecede
candidatecollection. Keep allbuild failures, failedR25storage andoriginalreports.

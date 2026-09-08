# Issue88 prospective sequential offline screens v1

Frozen before any retained-content encoding. Owner empirical authority is recorded
in issue88 and the delegated instruction. Base sourceeb7050603; isolated checkout
layerfs-issue88-experiments, branchcodex/issue88-encoding-experiments. Originalrun
full157-m45-1 and all issue87 evidence stay immutable. Objective completeallocated
Store<=134221004B is not a forecast or offline acceptance claim.

## Common custody and execution

Authenticate all original finalmanifest entries and exactStorehash2036b98181dd3fa47788619896cb50daa27ac5c85ea8cc482c20d4b447573ed9.
Frozenmanifest03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271;
all157 order and inputmanifest/blob seals retained. Hash contract/tool/source and
actualexecutable/codec before execution. Isolatedcode may add an observationhook
to currenttreeengine; defaultcanonicalbehavior unchanged. No dependency sourceedit.

Serialize builds, extraction, encoding and benchmark under existing
`$TMPDIR/layerfs-infra-measurement.lock`, nonblocking exclusive. Parent grants one
execution slot; subagents may edit/review in parallel but never run builds or
encoders concurrently. No currentStore openwriters; immutable SQLite only.
Outputs exclusive-create under runs/issue88-{stage}-{identity}-1. Preserve failed
outputs and newidentity for any fix. No old artifact overwrites.

Offline stages each have4h wallbudget, hostRSS<=8GiB, output<=32GiB, input/temporary
spool<=16GiB, hostfree>=50GiB atstart/checkpoints, twoCargo jobs/build900s, no
dockerneeded for offlinework. Use boundedgroups andcache<=32MiB; spillable metadata
instead of retainingpayloads. Stop on resource/auth/reconstruction/origin mismatch.
Offlinecold means freshdecoder+emptyapplicationcache, OS cacheuncontrolled; do not
claimdiskcold. Record processCPU/peakRSS/I/O and wall/encode/decode timing separately.

## S1 structural origin, canonical-preserving

Rebuild eachcheckpoint's inode table by applying exactsorted previous/current
binding differences to the retainedpriorroot with the existing treeengine. A
minimalObjectStore observerhook records the actual origin at Engine::persist;
rebuiltroot MUSTequal originalcheckpointroot. A mismatch aborts; inferredsimilar
leaves cannot replace actualengineorigin. Origins are reconstructed algorithmic
provenance, not a claimthat originalproducerloggedthem.

One treatment: inode-table-leaf actualorigin physicalDELTA, otherwiseexistingFULL.
Use currentcustomdelta matcher andgroupcodecZstd1/currentbounds. Identicalrecord
order/groupmembership and canonicalbytes forFULLcontrol/candidate. Bases must be
fromearliercheckpoint and remain FULLinCANDIDATErepresentation; an originpreviously
selectedDELTA causesFULLfallback, notanchorretarget/shadowFULL/deeperchain. Splits,
merges,missingorigin,marginalmixedgroup allnormalFULLfallbacks. Existingpayload
representation staysfixed andnotrecompressed forS1. Countcompleteaffectedgroup
encodedbytes, framing, selectedFULL/DELTA/baseclosure and allcanonicalauthchecks.
Do notadd canonicalreferencebytes asphysical savings. Validatefullrecorddecode
and currentdelta apply backtoexactObjectId; sharedstructuralgroupA/Bpopulation same.

S1 is first; sourcehookandtool review then committedidentity beforebuild/run.
A promisingS1 can trigger smallestpublicprototype plusoriginalthree smokes; scope
and exactcandidate freeze beforeexecution. No numericgate invented torescuefailure.

## S2 existing chunks, native prior prefix

Use existingFastCDC/profile and canonicalchunkcodec toextract unitidentities and
spans from authenticatedsourceblobs; no alternativechunker. Traverse157inputs in
checkpointorder and bytewisehexpath order,firstglobalCASoccurrence wins. Priorunit
is firstoverlapping chunk ofsamepath priorcheckpointregularfile. Onecandidatebase,
no globalsearch,no future or samecheckpointbase. Outputcanonicalid andrawpayload
SHA256; decoderreconstructs rawpayload then verifiescanonicalchunkidentity.

NativeZstd1.5.7 level3,singlethread,windowLog20,checksum1,contentsize1,dictID0.
FULL andPREFIX samecodecparameters; PREFIX references actualselectedpriorunit,
not its oldFULLanchor. Maximum4deltaedges and1048576 cumulative decodedpayloadB
includingbase/intermediates/target. FULLfallback whenbaseabsent/inadmissible,
closurecap or prefixrecordincluding32-bytebaseID doesnotstrictlybeatFULLrecord.
No futurebases or unchargedextraFULL copies. Allselected unitsandframesretained.
Eachunitmaximum32768rawB; originalcanonicalframing remains separately accounted.
The encoder is changed alongside declaredbasepolicy; no claimingmatcher-onlyeffect.

## S3 bounded whole-file units, same prefix policy

Sameinputorder,codec3/window20/depth4/closure1MiB/selectionrule asS2. Regularfiles
<=262144payloadB become content-only wholefile units. Largerfiles retain exact
S2CDCunits; wholefile/chunk IDs aretyped distinct. Cross-threshold priorunit mismatch
fallsbackFULL ratherthan inventrecipe. Samepathpriorwholefileonly ifbothwithinbound;
largefiles usefirstoverlapchunk. CASfirstoccurrence samepolicy. Sourceidentity via
Gitblob plusrawSHA256; proposedcanonicalframe is experimentalversion notexistingID.
Countloss ofcrossfilereuse, recurringidentities, frames/fallback/baseclosure/index
andfixednewcontainer framing. Payload-only results explicitly exclude regenerated
inodecanonicalstructures and are NOT134MBallocatedStore evidence.

## Required local falsifiers before/alongside screens

Use deterministic syntheticcases (notnewbenchmarkfamilies): smalllocalizededit,
identicalcrossfilecontent, exactreturntoA, and growth/shrink262143→262145→262143.
Use samefrozen settings; noseedsearch. RecordFULL/candidatebytes,encodeCPU/wall,
freshdecoder smallrange/fullfileread andwarmrepeat, depth/readbytes/peakmemory.
Bytecap doesnotprove latencyacceptable. Roundtripalltargets includeswrongbase,
truncatedframe rejection via strictdecoder checks beforetrustedresults.

## Results and continuation

Eachstage includes contract/source/executable/codec/settings hashes, exactcommands,
inputprovenance, targetcount/bytes, FULLcontrol/candidate completeframebytes,
baseclosure, indexes/container allocation separately, reconstructioncoverage,
phasecosts/resources, failuresandlimitations. Missingmeasurements nullwithreason.
Syntheticand offlineprobes notpublicExec/Commit samples. Groupcompressedbytes not
proportionallyattributed. Coldappcache notOScold. Historicalcontrolallocation/timers
remainseparate. Retain/reject/revise eachtreatment,then chooseONEproductiondirection.
Promisingofflineevidence requires isolatedpublicprototype andthreeapprovedsmokes
beforefull157 productclaim. Noautomaticcombinedrewrite/M5/cloud/merge/rollout.

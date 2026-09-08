# Additional pre-execution settings — issue88 offline screens

S1 optional matcher budget resets for each ORIGINAL physicalpack:16MiB charged
work and512trials. It usesactualengineorigin/candidateFULLonly, not a recreation
of historical admissionbatch scheduling. Originalgroups andrecordmembership fixed.
The controlmust match every originalstructuralgroup byte encoding exactly.

S2/S3 physicalcontainer:16-byte littleendian header `<8sII>` magicLFS88PFX,
version1,recordheaderlength56. Eachrecord uses `<BBHIQQ32s>`: kindu8,depthu8,
reservedu16zero,framesizeu32,payloadlengthu64,closurepayloadbytesu64,rawSHA25632.
PREFIX adds32-bytebaseSHA256 beforeframe; FULLdoesnot. Includeallheaders in
representationcomparison. JointanalysisSQLite stores bothcontrolandcandidate
locators; itsallocatedbytes are reportedseparately, not assignedtwice orcalled
productindex. Containerallocatedbytes measuredstat, notLayerFSStoreallocation.

S2 canonical reconstruction is existing21-byte canonicalchunkframing+rawpayload;
framing is deterministically reconstructed, not duplicatedinsidecompressedframe.
ObjectId wascomputed by existingcanonicalcode at extraction andverifiedthere;
every screenoutput authenticatesrawSHA256 andexactlength. Finalcodecproof must
also regeneratecanonicalID for completeS2classification (extractor target ID seal
alone insufficient for untrusteddecoder, rawsha+exactbytes establishes sameinput).
S3 experimental FileContent canonical domain:8-byteASCII `LFS88FC\0` followedby
u64bigendian payloadlength thenexactpayload; experimental canonicalID SHA256 of
that16-byteheader+payload. Typed sourcekey `file:<rawsha256>` is bijective content
index key, not existingLayerFSObjectId. Experimentalcanonicalbytes measured
separately16+payload; canonicalframing deterministicallyreconstructed, no duplicate
bytes charged tophysicalcontainer. Retainedstructuralregeneration remainsunmeasured.

Synthetic preflight fixture generation:Pythonrandom.Random(88).randbytes(262145).
Createunits at32768 and262143payloadbytes; forlocalizededit replace64bytes at
len//2 withASCIIZ; duplicatefiles reuseexactsamecontent; returntoA usesoriginalID.
Thresholdsequence262143→262145→262143 usesprefix ofsamegeneratedarray. Wholefile
routefallsbacktoexistingCDC on262145; recordbothunitidentityandfallbackcoverage.
Do notselectnewseed ormodifyoffsets basedonresults. Observeallroundtrips, FULLand
PREFIXframes/headerbytes; freshdecoder emptyapplicationcache smallrange first
min(4096,length), fullread, warmedrepeat. Everytarget reconstructedandhashed;
wrongbase andtruncatedframe mustreject beforetrustingnativecodec. Settingssameas
S2/S3. These areofflineboundary probes, notpublicsmokebenchmarks orOScoldclaims.

Extractiontargetorder:checkpointorder, bytewisehexpathorder, sourcechunkorder;
firstglobalCASoccurrencewins. Unitmanifests referencesharedoriginalimmutableblob
path+offset+length, notrawpayloadcopies. S3 unitsabovefilecap arecurrentFastCDCchunks,
notoneoversizeduncompressedfile. Earliertarget unitmustalready existinthearm's
selectedindexatstrictlyearliercheckpoint; otherwisepriornull. Metadata/symlink
payloads areoutsidecontentscreens andmustremainseparatebaselineaccounting.

# Exact157-state verification of the metadata-delta copy

Reuse the preceding full157 verifier unchanged except selecting results[delta] and importing the new actual-copy reader. Same157originalfixtureoracles,4MiBcanonical/8MiBpackcache, authenticatedcontent-rootdigestreuse, completepath/type/mode/size/SHA256/symlinkcomparison and inode-reference checks. The addedreader supports onlyone-level metadataDELTA referencing an earlier authenticatedFULLinodeleaf; its malformed-base tests havealready passed.

The entirecanonicalmetadata inventory, contentpackBLOBs andlocators, SQLroots/history andtypedIDs were proved unchanged duringlayoutconstruction. Thisadditionalfull pass qualifies the new actualcopyreadpath, not a publicAPIbenchmark. No unchangedpriorcopy orpolicy rerun. Require157states/904143paths/4936693030logicalbytes, originaloraclecustodyhashmatches andcopyhashunchanged before/after. Preserveerrors;neverpatchcandidate data to matchoracles.

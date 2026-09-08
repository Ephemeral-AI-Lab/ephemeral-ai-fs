#!/usr/bin/env python3
"""Exact budget requirements and sealed-manifest wholefile size profile; no encoding."""
import hashlib,json,pathlib,sys

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def main(repo,out):
    runs=repo.parent/'layerfs-storage-v3-runs'
    control=json.loads((runs/'builds/full157-controls.json').read_text())
    run=runs/'full157-m45-1'
    performance=json.loads((run/'deepseek-full/performance-result.json').read_text())
    objects={};manifest_hashes={}
    for row in performance['records']:
        p=pathlib.Path(row['input'])/'manifest.tsv';digest=sha(p)
        assert digest==row['input_seal']['manifest.tsv']
        manifest_hashes[str(row['index'])]=digest
        for line in p.read_text().splitlines():
            mode,oid,size,_path=line.split('\t');size=int(size)
            assert mode in ('100644','100755','120000')
            if oid in objects:assert objects[oid][0]==size
            else:objects[oid]=[size,set()]
            objects[oid][1].add('symlink' if mode=='120000' else 'regular')
    assert len(objects)==75929 and sum(v[0] for v in objects.values())==891893320
    cuts=[4096,16384,32768,65536,131072,262144,1048576,2**63-1]
    bins=[];lower=0
    for upper in cuts:
        selected=[v[0] for v in objects.values() if lower<=v[0]<upper]
        bins.append({'min_bytes_inclusive':lower,'max_bytes_exclusive':upper if upper<2**63-1 else None,'unique_blob_count':len(selected),'payload_bytes':sum(selected)})
        lower=upper
    profile={'status':'derived exact from authenticated frozen input manifests','population':'uniqueGitblobIDs across157 selected trees; repeatedpaths/versions countonce; regular/symlink usage may overlap','units':'payload bytes and integer counts; thresholds as named','snapshot':'frozen157 source inputs, not alternative-encoding result','provenance':{'manifest_sha256_by_checkpoint':manifest_hashes},'unique_blob_count':len(objects),'unique_payload_bytes':sum(v[0] for v in objects.values()),'maximum_payload_bytes':max(v[0] for v in objects.values()),'bins':bins,'wholefile_256KiB_or_less':{'count':sum(v[0]<=262144 for v in objects.values()),'payload_bytes':sum(v[0] for v in objects.values() if v[0]<=262144)},'regular_only':{'count':sum(v[1]=={'regular'} for v in objects.values()),'payload_bytes':sum(v[0] for v in objects.values() if v[1]=={'regular'})},'symlink_only':{'count':sum(v[1]=={'symlink'} for v in objects.values()),'payload_bytes':sum(v[0] for v in objects.values() if v[1]=={'symlink'})},'both_use_labels_count':sum(len(v[1])==2 for v in objects.values())}
    A=335552512;T=A*2//5;P=216448341;S=78792537;F=703824
    scenarios=[]
    # Assumed future allocated non-group budget; never splice current different snapshots.
    for overhead in [0,10000000,20000000,30000000,40000000]:
        for structure in [10000000,20000000,40000000,S]:
            payload=T-overhead-structure
            scenarios.append({'assumed_total_nongroup_allocated_bytes':overhead,'assumed_structural_group_bytes':structure,'maximum_payload_group_bytes':payload,'required_payload_reduction_bytes':P-payload,'required_payload_reduction_numerator':P-payload,'required_payload_reduction_denominator':P,'status':'scenario arithmetic; assumptions not measured forecasts'})
    git=control['git_control'];full=git['phases']['pack_no_delta']['storage']['pack_bytes'];delta=git['phases']['pack_delta']['storage']['pack_bytes']
    model={'status':'derived exact baseline arithmetic plus explicitly assumed future budgets','units':'integer bytes, ratio numerators/denominators','population':'full157 final acknowledgement allocated target; separate postverification encodedgroup buckets; future overhead is a scenario variable','snapshot':'acknowledgement335552512 primary; existing groupcensus postverification canonical data; no alternative Store measured','target_allocated_bytes_floor':T,'required_allocated_reduction_bytes':A-T,'baseline':{'ack_allocated_bytes':A,'payload_group_bytes':P,'structural_group_bytes':S,'pack_framing_bytes':F,'postverify_sqlite_logical_bytes':318869504,'postverify_nongroup_sqlite_bytes':318869504-P-S,'ack_signed_allocation_adjustment_bytes':16719872},'impossible_free_metadata_lower_bound':{'payload_bytes_still_required_to_remove':P-T,'interpretation':'Even zero structure/index/SQLite/filesystem overhead leaves payload216448341 greater than target; metadata-only optimization cannot reach60percent'},'historical_git':{'full_only_pack_bytes':full,'delta_pack_bytes':delta,'difference_bytes':full-delta,'ratio_reduction_numerator':full-delta,'ratio_reduction_denominator':full,'same_object_count':110081,'full_only_allocated_bytes':289480704,'delta_allocated_bytes':56373248,'full_only_repack_ns':git['phases']['pack_no_delta']['repack_ns'],'delta_repack_ns':git['phases']['pack_delta']['repack_ns'],'scope':'historical phase sequence; same157 trees, differentmetadata and batchpacking; no freshtiming or causal perfeature attribution'},'scenarios':scenarios,'limitations':['No60percent forecast; scenarios specify the reductions a candidate must demonstrate','Metadata and payload changes cannot have savings summed independently if their population orencoding changes','Currentfilesystem adjustment not assumedfree/reclaimable','All retainedsemanticstates andmetadata remainrequired even if canonicalencodingchanges']}
    for name,data in [('wholefile-size-profile.json',profile),('target-budget.json',model)]:
        with (out/name).open('x') as f:json.dump(data,f,indent=2);f.write('\n')
    print(json.dumps({'target':T,'unique_blob_count':len(objects),'payload_bytes':profile['unique_payload_bytes'],'wholefile256k':profile['wholefile_256KiB_or_less'],'git_delta_pack_reduction_bytes':full-delta},indent=2))

if __name__=='__main__':
    if sys.argv[1:]==['--self-test']:
        assert 335552512*2//5==134221004
        assert 216448341-134221004==82227337
        assert 275324594-51989900==223334694
        print('target model self-check PASS')
    else:main(*map(pathlib.Path,sys.argv[1:]))

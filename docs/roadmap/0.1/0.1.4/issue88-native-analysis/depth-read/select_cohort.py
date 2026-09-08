#!/usr/bin/env python3
"""Bounded cohort preparation. Root alone runs this under the measurement lock.

Inputs schema issue88-depth-selection-inputs-v1: sealed refs contract,schedule,
extraction,extraction_provenance,candidate_inventory,candidate_inventory_proof,
proof_binary,candidate_trajectory; runs:{control,candidate}; copies:{arm:{path,sha256}}.
Uses existing source blob/chunk metadata and exact product selected-path proof;
never reconstructs a Store or replays construction. No automatic measurement.
"""
import argparse,csv,hashlib,json,pathlib,sqlite3,subprocess,time
BUDGET={'candidate_path_attempts':64,'structural_reads_per_proof':4096,'structural_canonical_bytes_per_proof':16777216,'proof_seconds':120,'extents_per_file':4096,'source_digest_bytes':67108864,'selection_seconds':1800,'output_bytes':134217728}

def require(c,m):
    if not c:raise ValueError(m)
def sha(p):
    with pathlib.Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def load(p):return json.loads(pathlib.Path(p).read_text())
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2,sort_keys=True);f.write('\n')
def ref(v):
    p=pathlib.Path(v['path']);require(p.is_absolute() and p.is_file() and not p.is_symlink(),'regular absolute sealed path');require(sha(p)==v['sha256'],'sealed hash '+str(p));return p.resolve()
def safe(pathhex):
    require(isinstance(pathhex,str) and len(pathhex)%2==0 and all(c in '0123456789abcdef' for c in pathhex),'path hex')
    b=bytes.fromhex(pathhex);require(b and not b.startswith(b'/') and b'\0' not in b and all(c not in (b'',b'.',b'..') for c in b.split(b'/')),'unsafe relative path');return b

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--inputs',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);args=p.parse_args()
    config=load(args.inputs);require(config['schema']=='issue88-depth-selection-inputs-v1','input schema')
    out=args.output.resolve();require(not out.exists(),'new output')
    for run in config['runs'].values():require(not out.is_relative_to(pathlib.Path(run).resolve()),'output inside original')
    out.mkdir(parents=True)
    start=time.monotonic();attempts=0;source_bytes=0;selected={};exclusions={};blob_digests={};copy_refs={};status='INCOMPLETE';inputs={}
    def bound():
        require(time.monotonic()-start<=BUDGET['selection_seconds'],'selection time budget')
        require(sum(p.stat().st_size for p in out.rglob('*') if p.is_file())<=BUDGET['output_bytes'],'output budget')
    try:
        for name in ('preparation_custody','contract','schedule','extraction','extraction_provenance','candidate_inventory','candidate_inventory_proof','proof_binary','candidate_trajectory'):
            inputs[name]={'path':str(ref(config[name])),'sha256':config[name]['sha256']}
        schedule=load(inputs['schedule']['path']);require(schedule['schema']=='issue88-SP-full157-frozen-v1','SPschedule')
        ip=load(inputs['candidate_inventory_proof']['path']);require(ip['status']=='PASS' and ip['all_selected_ids_authenticated'] is True and ip['all_references_valid'] is True and ip['inventory_sha256']==inputs['candidate_inventory']['sha256'],'actual inventory authentication proof')
        preparation=load(inputs['preparation_custody']['path']);require(preparation['schema']=='issue88-depth-preparation-custody-v1' and preparation['status']=='PASS','root preparation custody')
        for name in ('contract','schedule','extraction','extraction_provenance'):require(preparation[name]==config[name],'rootcustody input differs: '+name)
        require(preparation['arms']['candidate']['inventory']==config['candidate_inventory'] and preparation['arms']['candidate']['inventory_proof']==config['candidate_inventory_proof'],'root inventorybinding')
        # Older extraction provenance must explicitly contain the sealed cache digest.
        provenance=load(inputs['extraction_provenance']['path'])
        extraction_path=pathlib.Path(inputs['extraction']['path'])
        relative=extraction_path.relative_to(pathlib.Path(provenance['runs_root']))
        require(len(relative.parts)==2,'historical extraction registry path')
        cached_entry=provenance['artifacts'][relative.parts[0]][relative.parts[1]]
        require(cached_entry['sha256']==inputs['extraction']['sha256'] and cached_entry['bytes']==extraction_path.stat().st_size,'exact extraction registry entry')
        data={};identity={};runrefs={}
        for arm in ('control','candidate'):
            run=pathlib.Path(config['runs'][arm]).resolve();require(not out.is_relative_to(run),'output inside original')
            frozen=next(r for r in schedule['order'] if r['arm']==arm);require(pathlib.Path(frozen['output']).resolve()==run,'wrong frozenrun')
            manifest=load(run/'verification-manifest.json');perf=run/'deepseek-full/performance-result.json';ident=run/'identity.json'
            for q in (perf,ident):require(sha(q)==manifest[str(q.relative_to(run))],'original receipt changed')
            require(load(run/'verification-summary.json')['status']=='PASS','original historical verifier incomplete')
            data[arm]=load(perf);identity[arm]=load(ident)
            require(data[arm]['status']==data[arm]['cleanup_status']=='PASS' and data[arm]['container_removed'] is True,'producer notclosed')
            require(identity[arm]['host_identity']==frozen['host'] and identity[arm]['image_id']==frozen['image_id'],'frozenproducer/image')
            require([r['index'] for r in data[arm]['records']]==list(range(1,158)),'157sourceorder')
            require(preparation['arms'][arm]['run']==str(run) and preparation['arms'][arm]['copy']==config['copies'][arm],'root copy/run binding')
            require(config['copies'][arm]['sha256']==preparation['arms'][arm]['source_snapshot']['sha256'],'copy differs from preverification snapshot')
            copy=ref(config['copies'][arm]);require(not copy.is_relative_to(run),'copy aliases originaltree')
            require(copy.stat().st_ino!=(run/'deepseek-full/host-runtime/store.sqlite').stat().st_ino or copy.stat().st_dev!=(run/'deepseek-full/host-runtime/store.sqlite').stat().st_dev,'hardlink source')
            copy_refs[arm]={'path':str(copy),'sha256':config['copies'][arm]['sha256']}
            runrefs[arm]={'run':str(run),'performance':{'path':str(perf),'sha256':sha(perf)},'identity':{'path':str(ident),'sha256':sha(ident)},'branch_id':(run/'deepseek-full/host-runtime/branch-id').read_text().strip()}
        report_manifest_path=ref(preparation['completed_report_manifest']);report_manifest=load(report_manifest_path)
        require(report_manifest['files']['candidate/checkpoint-trajectory.csv']['sha256']==inputs['candidate_trajectory']['sha256'],'trajectory differs from completed sealed report')
        validation_path=report_manifest_path.parent/'candidate/validation.json'
        require(sha(validation_path)==report_manifest['files']['candidate/validation.json']['sha256'],'completed validator seal')
        validation=load(validation_path)
        require(validation['status']=='PASS' and validation['native_representations'] is True and validation['checkpoint_count']==158,'completed native validator status')
        require(validation['input_hashes']['inventory']==inputs['candidate_inventory']['sha256'] and validation['input_hashes']['snapshot']==copy_refs['candidate']['sha256'] and validation['input_hashes']['performance']==runrefs['candidate']['performance']['sha256'],'validator not bound to this inventory/snapshot/performance')
        inputs['completed_report_manifest']=preparation['completed_report_manifest'];inputs['candidate_validation']={'path':str(validation_path),'sha256':sha(validation_path)}
        trajectory=list(csv.DictReader(pathlib.Path(inputs['candidate_trajectory']['path']).open()))
        require([int(r['checkpoint']) for r in trajectory]==list(range(158)) and all(r['status']=='PASS' for r in trajectory),'158cohortgate order')
        inventory=sqlite3.connect(pathlib.Path(inputs['candidate_inventory']['path']).as_uri()+'?mode=ro&immutable=1',uri=True)
        extraction=sqlite3.connect(pathlib.Path(inputs['extraction']['path']).as_uri()+'?mode=ro&immutable=1',uri=True)
        inventory.execute('pragma cache_size=-8192');extraction.execute('pragma cache_size=-8192')
        for cp in range(1,158):
            bound();c=data['candidate']['records'][cp-1];control=data['control']['records'][cp-1]
            require(all(c[k]==control[k] for k in ('sha','tree','oracle_sha256','manifest_sha256')),'paired source identities')
            source=pathlib.Path(c['input']);manifest=source/'manifest.tsv';require(sha(manifest)==c['input_seal']['manifest.tsv'],'source manifest seal')
            entries=[]
            for line in manifest.read_text().splitlines():
                mode,oid,size,pathhex=line.split('\t');require(len(oid)==40 and all(ch in '0123456789abcdef' for ch in oid),'Git blob identifier');entries.append((safe(pathhex),mode,oid,int(size),pathhex))
            for _,mode,oid,size,pathhex in sorted(entries):
                require(time.monotonic()-start<=BUDGET['selection_seconds'],'selection time budget during path search')
                if mode not in ('100644','100755') or not 4096<=size<=8388608:
                    exclusions['nonregular_or_file_size']=exclusions.get('nonregular_or_file_size',0)+1;continue
                cached=extraction.execute('select sha,size,chunks from blobs where id=? and length(chunks)<=2097152',(oid,)).fetchone()
                require(cached is not None and cached[1]==size,'missing/incompatible sealed blob map')
                chunks=json.loads(cached[2]);position=0
                for x in chunks:
                    require(type(x['offset']) is int and x['offset']==position and type(x['length']) is int and 0<x['length']<=32768 and len(x['id'])==64 and all(ch in '0123456789abcdef' for ch in x['id']),'cache chunk geometry/identity')
                    position+=x['length']
                require(position==size,'cache chunkcoverage')
                for chunk in sorted(chunks,key=lambda x:x['offset']):
                    require(time.monotonic()-start<=BUDGET['selection_seconds'],'selection time budget during chunk search')
                    if chunk['length']<4096:
                        exclusions['span_too_small']=exclusions.get('span_too_small',0)+1;continue
                    ident=bytes.fromhex(chunk['id'])
                    r=inventory.execute("select n.prefix_edges,r.pack,r.grp,r.rec,n.raw_bytes,n.closure_raw_bytes,r.kind from records r join native_records n using(pack,grp,rec) join file_content_objects f on f.id=r.id where r.id=? and r.selected=1",(ident,)).fetchone()
                    if r is None or r[0] in selected:
                        key='not_native_selected_file' if r is None else 'stratum_already_selected';exclusions[key]=exclusions.get(key,0)+1;continue
                    depth,pack,group,record,raw,closure,kind=r
                    require(depth in range(5) and (kind=='NATIVE_FULL')==(depth==0),'depth-kind')
                    # Published interval endpoint is authoritative only with retained path proof below.
                    require(pack<=int(trajectory[cp]['pack_last_id'] or trajectory[cp]['diag_selected_pack_last_id']),'selected object appears before its publication')
                    require(attempts<BUDGET['candidate_path_attempts'],'candidate path budget');attempts+=1;bound()
                    proofs={};offset=chunk['offset']
                    for arm,step in (('candidate',c),('control',control)):
                        cmd=[inputs['proof_binary']['path'],copy_refs[arm]['path'],step['commit_id'],pathhex,str(size),chunk['id'],str(offset)]
                        log=out/f'attempt-{attempts:02d}-{arm}.stderr';t=time.monotonic_ns()
                        with log.open('xb') as stderr:
                            completed=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=stderr,timeout=min(120,BUDGET['selection_seconds']-(time.monotonic()-start)))
                        require(len(completed.stdout)<=2097152,'proof stdout bound')
                        (out/f'attempt-{attempts:02d}-{arm}.stdout').write_bytes(completed.stdout)
                        receipt={'command':cmd,'exit_code':completed.returncode,'elapsed_ns':time.monotonic_ns()-t}
                        save(out/f'attempt-{attempts:02d}-{arm}-invocation.json',receipt)
                        require(completed.returncode==0,'retained path proof failed; no alternate stratum substitution')
                        proof=json.loads(completed.stdout);require(proof['status']=='PASS' and proof['path_hex']==pathhex and proof['target_id']==chunk['id'] and proof['offset_bytes']==offset and proof['file_length_bytes']==size,'proofoutputidentity')
                        require(proof['structural_reads_count']<=4096 and proof['structural_canonical_bytes']<=16777216 and len(proof['descriptors'])<=4096,'proofbudget')
                        proofs[arm]=proof
                    require(proofs['candidate']['metadata_root']==proofs['control']['metadata_root'],'paired metadata canonical root mismatch')
                    if oid not in blob_digests:
                        require(source_bytes+size<=BUDGET['source_digest_bytes'],'source digest byte budget')
                        body=(source/'blobs'/oid).read_bytes();source_bytes+=len(body)
                        digest=hashlib.sha256(body).hexdigest()
                        require(len(body)==size and digest==cached[0]==c['input_seal']['blobs/'+oid],'source blobcontent/cacheidentity')
                        require(hashlib.sha1(b'blob '+str(size).encode()+b'\0'+body).hexdigest()==oid,'Git blobidentity')
                        blob_digests[oid]={'full':digest,'range':{}}
                    else:
                        # Reuse only digests; new selected offsets consume declared source-read budget.
                        body=None
                    if str(offset) not in blob_digests[oid]['range']:
                        if body is None:
                            require(source_bytes+4096<=BUDGET['source_digest_bytes'],'source range digest byte budget')
                            with (source/'blobs'/oid).open('rb') as f:f.seek(offset);part=f.read(4096)
                            source_bytes+=len(part)
                        else:part=body[offset:offset+4096]
                        require(len(part)==4096,'source rangeEOF');blob_digests[oid]['range'][str(offset)]=hashlib.sha256(part).hexdigest()
                    del body
                    oracle=pathlib.Path(c['oracle']);require(sha(oracle)==c['oracle_sha256'],'sourceoracle seal')
                    require(load(oracle)[pathhex]==[mode,size,blob_digests[oid]['full']],'sourceoracle mapping')
                    span=proofs['candidate']['selected_span'];require(span['source_offset']+span['logical_length']<=raw,'extent exceeds raw target')
                    # Existing selected base graph is metadata only; no new frame decode.
                    chain=[];baseid=ident;previous_pack=None;rawsum=0
                    for edge in range(depth+1):
                        br=inventory.execute('select r.id,r.kind,r.pack,r.grp,r.rec,r.base,n.raw_bytes,n.prefix_edges,n.closure_raw_bytes from records r join native_records n using(pack,grp,rec) where r.id=? and r.selected=1',(baseid,)).fetchone()
                        require(br is not None and br[7]==depth-edge and (previous_pack is None or br[2]<previous_pack),'dependency selected/depth/order')
                        rawsum+=br[6];chain.append({'id':bytes(br[0]).hex(),'kind':br[1],'pack':br[2],'group':br[3],'record':br[4],'raw_bytes':br[6]});previous_pack=br[2];baseid=br[5]
                    require(baseid is None and rawsum==closure and closure<=1048576,'dependency closure')
                    composition={}
                    for d in proofs['candidate']['descriptors']:
                        k=inventory.execute('select r.kind,r.bytes,n.prefix_edges from records r left join native_records n using(pack,grp,rec) where r.id=? and r.selected=1',(bytes.fromhex(d['id']),)).fetchone();require(k is not None and d['source_offset']+d['logical_length']<=k[1]-21,'filedescriptor selectedlocator/rawbound');label=k[0]+':'+(str(k[2]) if k[2] is not None else 'legacy');composition[label]=composition.get(label,0)+d['logical_length']
                    proofpath=out/f'depth-{depth}-proof.json';save(proofpath,{'schema':'issue88-depth-selected-proof-v1','status':'PASS','selection_identity':{'depth':depth,'checkpoint':cp,'source_sha':c['sha'],'source_tree':c['tree'],'path_hex':pathhex,'offset_bytes':offset,'range_length_bytes':4096,'file_length_bytes':size,'expected_range_sha256':blob_digests[oid]['range'][str(offset)],'expected_full_sha256':blob_digests[oid]['full'],'candidate_target_id':chunk['id'],'control_commit_id':control['commit_id'],'candidate_commit_id':c['commit_id']},'disposable_copies':copy_refs,'arms':proofs,'candidate_dependency_chain':chain,'candidate_file_representation_logical_bytes':composition,'provenance':inputs})
                    selected[depth]={'depth':depth,'checkpoint':cp,'source_sha':c['sha'],'source_tree':c['tree'],'source_oracle_sha256':c['oracle_sha256'],'source_manifest_sha256':c['manifest_sha256'],'path_hex':pathhex,'file_length_bytes':size,'offset_bytes':offset,'range_length_bytes':4096,'expected_range_sha256':blob_digests[oid]['range'][str(offset)],'expected_full_sha256':blob_digests[oid]['full'],'control':{'branch_id':runrefs['control']['branch_id'],'commit_id':control['commit_id']},'candidate':{'branch_id':runrefs['candidate']['branch_id'],'commit_id':c['commit_id'],'target_id':chunk['id'],'pack':pack,'group':group,'record':record,'raw_bytes':raw,'depth':depth,'closure_raw_bytes':closure,'target_span':{'logical_offset':span['file_offset'],'object_offset':span['source_offset'],'length':span['logical_length']}},'proof':{'path':str(proofpath),'sha256':sha(proofpath)},'selection_provenance':'first proven source-index/path-bytes/offset eligible occurrence; existing sealed source CDC map plus exact retained structural path','full_file_scope':'target-selected depth stratum, not homogeneous depth file'}
                    if len(selected)==5:break
                if len(selected)==5:break
            if len(selected)==5:break
        inventory.close();extraction.close();require(set(selected)==set(range(5)),'missing eligible depth stratum')
        for arm,reference in copy_refs.items():require(sha(reference['path'])==reference['sha256'],'proof mutated disposable logical copy')
        for name,reference in inputs.items():require(sha(reference['path'])==reference['sha256'],'selection input changed: '+name)
        bound();save(out/'cohort-manifest.json',{'schema':'issue88-depth-read-cohort-v1','status':'PASS','selections':[selected[d] for d in range(5)],'inputs':inputs,'runs':runrefs,'copies':copy_refs,'budgets':BUDGET,'candidate_path_attempts':attempts,'source_digest_bytes':source_bytes,'selection_observer_ns':int((time.monotonic()-start)*1e9),'units':'integer bytes/count/ns; IDs/pathhex text; fullfiledepth is target-selected only'});status='PASS'
    finally:
        save(out/'preparation-status.json',{'status':status,'candidate_path_attempts':attempts,'source_digest_bytes':source_bytes,'selected_depths':sorted(selected),'exclusions':exclusions,'elapsed_ns':int((time.monotonic()-start)*1e9),'budgets':BUDGET})
        save(out/'manifest.sha256.json',{str(p.relative_to(out)):sha(p) for p in sorted(out.rglob('*')) if p.is_file()})
if __name__=='__main__':main()

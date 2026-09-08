#!/usr/bin/env python3
"""Seal issue87 analysis artifacts after review; never opens SQLite or runs product."""
import argparse, hashlib, json, pathlib, subprocess

def sha(path):
    with path.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def read(path):return json.loads(path.read_text())

def write(path,data):
    with path.open('x') as f:json.dump(data,f,indent=2);f.write('\n')

def main(run,out,repo):
    required=['contract.md','accounting-reconciliation.json','checkpoint-trajectory.csv','sqlite-breakdown.csv','packs.csv','groups.csv','object-roles.csv','object-retention.csv','delta-dependencies.csv','delta-opportunity.csv','counter-reconciliation.json','independent-review.md','findings-and-next-experiment.md','roles-summary.json','custody-review.json']
    assert not (out/'manifest.sha256.json').exists(), 'analysis already sealed'
    assert all((out/n).stat().st_size for n in required), 'missing/empty deliverable'
    checks={}
    for phase in ['performance','verification']:
        matches=[];changed=[]
        for name,digest in read(run/(phase+'-manifest.json')).items():
            (matches if sha(run/name)==digest else changed).append(name)
        assert changed==(['deepseek-full/host-runtime/store.sqlite'] if phase=='performance' else []),changed
        checks[phase]={'matching_count':len(matches),'changed':changed,'manifest_sha256':sha(run/(phase+'-manifest.json'))}
    original=run/'deepseek-full/host-runtime/store.sqlite';account=read(out/'accounting-reconciliation.json')
    assert sha(original)==account['source_sha256'], 'original Store changed during analysis'
    handles=subprocess.run(['lsof',str(original)],capture_output=True,text=True)
    assert handles.returncode==1 and not handles.stdout, 'Store open handles or lsof error'
    tool_root=repo/'docs/roadmap/0.1/0.1.4/issue87-analysis'
    tools={str(p.relative_to(repo)):sha(p) for p in sorted(tool_root.rglob('*')) if p.is_file() and not any(x in p.parts for x in ['target','__pycache__','published'])}
    identity=read(run/'identity.json');stat=original.stat()
    roles=read(out/'roles-summary.json')
    counters=read(out/'counter-reconciliation.json')
    payload=[r for r in roles['roles'] if r['role']=='payload_chunk']
    counters['final_eligible_shape_payload']={'object_count':sum(r['objects'] for r in payload),'canonical_bytes':sum(r['canonical_bytes'] for r in payload),'unit':'count and bytes','population':'final unique selected authenticated payloads, all eligible-sized','snapshot':'post-verification canonical inventory','status':'derived exact final sum; initial-missing denominator equivalence is source-supported run-specific inference','provenance':'roles-summary.json plus objects/admission.rs prepare_missing/candidate; count equals eligible_targets, no recorded optional-memory exclusions','limitation':'No directly recorded initial target byte denominator; missing-hint and exclusive terminal byte populations remain unavailable.'}
    (out/'counter-reconciliation.json').write_text(json.dumps(counters,indent=2)+'\n')
    builds=run.parent/'builds'
    scope={'schema':'issue87-identity-scope-v1','status':'derived from authenticated original receipts and independent review','units':'bytes/ns/counts integers; hashes SHA256; identifiers text','population':'full ordered157 source states plus Init; all selected post-verification objects/physical records','snapshot':'acknowledgement physical receipts primary; current retained Store post-verification','provenance':str(run),'initial_discovery_head':'014c0b9cb5d62bd50ded2052a2604d4aeab76dce','reporter_base_head':subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip(),'reporter_source_hashes':tools,'analysis_decoder_binary':{'path':str(tool_root/'roles/target/release/issue87-roles'),'sha256':sha(tool_root/'roles/target/release/issue87-roles'),'source_qualification':'One authenticated decode pass used this binary; final source additionally adds records(base) analysis index for graph-report speed, equivalent index added to analysis inventory before resuming metadata exports, plus formatting. Final source is not falsely claimed byte-identical to executed reporter build. Product decoder source unchanged at seal.'},'reused_decoder_source_hashes':{str(p.relative_to(repo)):sha(p) for p in sorted((repo/'crates/layerfs-content/src').rglob('*.rs'))} | {'crates/layerfs-layerstack-store/src/objects/pack.rs':sha(repo/'crates/layerfs-layerstack-store/src/objects/pack.rs')},'producer':identity['host_identity'],'source':identity['source'],'image_id':identity['image_id'],'contract_hash':identity['full_run_contract_sha256'],'original_identity_sha256':sha(run/'identity.json'),'original_manifest_checks':checks,'store':{'path':str(original),'sha256':sha(original),'current_logical_bytes':stat.st_size,'current_allocated_bytes':stat.st_blocks*512,'inode':stat.st_ino,'mtime_ns':stat.st_mtime_ns,'current_lsof_open_handles_count':0,'snapshot':'post-verification original; no pre-verification snapshot survives','pre_verification_sha256':read(run/'performance-manifest.json')['deepseek-full/host-runtime/store.sqlite'],'pre_verification_snapshot':None,'null_reason':'Verifier mutated same original Store; retained performance digest is not a snapshot'},'supporting_source_hashes':{n:sha(builds/n) for n in ['full157-source.patch','full157-host','full157-host.identity.json','full157-report.py','full157-controls.json','full157-environment.json']},'environment':read(builds/'full157-environment.json'),'verification':'custody-review.json all157 mappings, observed dictionaries and all802 final manifest entries independently verified','historical_controls':'940310528 LayerFS and56373248 delta-packed Git allocated bytes; same157 trees, historical not fresh timing pairs; Git metadata/interface/packing and normalized mtime limitations retained','analysis_boundary':'No original modifications, Store copy, replay, sample collection, recompression, optimization, fixture/benchmark edit, kernel trace, M5, S3 or merge. Spillable analysis SQLite contains metadata only.'}
    write(out/'identity-and-scope.json',scope)
    files={str(p.relative_to(out)):{'sha256':sha(p),'length_bytes':p.stat().st_size} for p in sorted(out.rglob('*')) if p.is_file()}
    write(out/'manifest.sha256.json',{'schema':'issue87-analysis-manifest-v1','status':'measured hashes and lengths at final seal','units':'length_bytes bytes; sha256 hex digest','population':'all analysis artifacts except this manifest itself','snapshot':'final reviewed immutable analysis','provenance':str(out),'reporter_source_hashes':tools,'files':files})
    print(json.dumps({'sealed_files':len(files),'manifest_sha256':sha(out/'manifest.sha256.json'),'original_unchanged':True}))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('run',type=pathlib.Path);p.add_argument('out',type=pathlib.Path);p.add_argument('repo',type=pathlib.Path);a=p.parse_args();main(a.run,a.out,a.repo)

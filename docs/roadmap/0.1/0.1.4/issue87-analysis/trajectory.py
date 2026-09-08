#!/usr/bin/env python3
"""Receipt-only issue87 analysis. Never opens a Store or executes product code."""
import argparse, csv, hashlib, json, pathlib, re

def read(p):
    return json.loads(p.read_text())

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def one(rows, kind):
    found = [r for r in rows if r.get('kind') == kind]
    assert len(found) == 1, (kind, len(found))
    return found[0]

def flat(d, prefix=''):
    out = {}
    for k,v in d.items():
        name = prefix+k
        if isinstance(v,dict): out.update(flat(v,name+'_'))
        else: out[name] = v
    return out

def stats(receipts):
    return {r['phase']:r for r in receipts if r.get('kind') == 'storage-smoke-phase'}

def parsed_candidate(receipt):
    text = receipt.get('commit_receipt','')
    m = re.search(r'CandidateStats \{([^}]+)\}',text)
    return {k:int(v) for k,v in re.findall(r'(\w+): (\d+)',m[1])} if m else {}

def write_json(path, data):
    with path.open('x') as f: json.dump(data,f,indent=2); f.write('\n')

def main(run, output, frozen):
    d = read(run/'deepseek-full/performance-result.json')
    v = read(run/'deepseek-full/verification-result.json')
    manifest = read(frozen)
    assert len(d['records']) == len(v['records']) == len(manifest['checkpoints']) == 157
    assert d['status'] == v['status'] == d['cleanup_status'] == v['cleanup_status'] == 'PASS'
    init = one(d['ready'],'storage-smoke-allocation')
    previous = None; previous_mapping = 'initial'; rows=[]; sums={}; fields={}
    previous_cgroup=d['cgroup_before']
    checkpoints = [{'index':0,'receipts':d['ready']}] + d['records']
    for s in checkpoints:
        i=s['index']; a=one(s['receipts'],'storage-smoke-allocation'); phases=stats(s['receipts'])
        row={'index':i,'snapshot':'performance acknowledgement before historical verification','status':'measured','provenance':f'deepseek-full/performance-result.json#{"ready" if i==0 else "records/"+str(i-1)}'}
        row.update({k:s.get(k) for k in ['sha','tree','ordinal','oracle_sha256','manifest_sha256','logical_bytes','files','transfer_ns','step_wall_ns']})
        row.update({k:x for k,x in a.items() if k not in ['kind','label']})
        row.update(prior_mapping=previous_mapping,resulting_mapping=s.get('identity','initial'),created=s.get('created'),preparation_ns=None,observer_ns=None,free_disk_bytes=None,persisted_new_pack_ids=None,first_admission_provenance=None)
        row['allocation_adjustment_bytes']=a['database_allocated_bytes']-a['database_logical_bytes']
        assert a['store_allocated_bytes']==a['database_allocated_bytes']+a['sidecar_allocated_bytes']
        assert a['database_logical_bytes']==a['page_count']*a['page_size_bytes']
        for key in ['store_allocated_bytes','database_logical_bytes','canonical_bytes','canonical_objects']:
            row['change_'+key]=a[key]-previous[key] if previous else None
        for phase,p in phases.items():
            row.update(flat({k:x for k,x in p.items() if k not in ['kind','phase']},phase+'_'))
            bucket=sums.setdefault(phase,{})
            for key,x in p.get('physical_storage',{}).items(): bucket[key]=bucket.get(key,0)+x
        if i:
            f=manifest['checkpoints'][i-1]; vr=v['records'][i-1]
            assert all(s[k]==f[k] for k in ['index','sha','tree','ordinal','logical_bytes','manifest_sha256'])
            assert vr['index']==i and vr['identity']==s['identity'] and vr['status']=='PASS'
            assert sha(pathlib.Path(s['oracle']))==s['oracle_sha256']
            cr=one(s['receipts'],'storage-smoke-receipt'); candidate=parsed_candidate(cr)
            row.update(candidate)
            row.update({k:cr.get(k) for k in ['fuse_write_requests_cumulative','fuse_write_bytes_cumulative']})
            assert candidate['candidate_objects']==candidate['inserted_objects']+candidate['reused_objects']
            assert candidate['candidate_bytes']==candidate['inserted_bytes']+candidate['reused_bytes']
            assert row['change_canonical_objects']==candidate['inserted_objects']
            assert row['change_canonical_bytes']==candidate['inserted_bytes']
            row['step_unattributed_ns']=s['step_wall_ns']-s['transfer_ns']-sum(p['elapsed_ns'] for p in phases.values())
            assert row['step_unattributed_ns']>=0
            row.update(flat({k:s[k] for k in ['cgroup','host_runtime_disk','spool_disk','container_staging_allocated_bytes']}))
            for key,value in s['cgroup'].items():
                if key.endswith('_usec'):
                    row.pop('cgroup_'+key)
                    row['cgroup_'+key[:-5]+'_ns_cumulative']=value*1000
                    row['cgroup_'+key[:-5]+'_ns_boundary_delta']=(value-previous_cgroup[key])*1000
            previous_cgroup=s['cgroup']
            row['verified_entries']=vr['verified_entries']; row['verified_bytes']=vr['verified_bytes']
            calls=one(s['receipts'],'storage-smoke-calls');row.update({k:x for k,x in calls.items() if k not in ['kind','index']})
            assert calls['exec_process_count']==calls['commit_call_count']==1 and calls['sdk_edit_call_count']==0
        rows.append(row);previous=a;previous_mapping=row['resulting_mapping']
    keys=list(dict.fromkeys(k for r in rows for k in r))
    for key in keys:
        unit='ns' if key.endswith('_ns') or '_ns_' in key else 'bytes' if 'bytes' in key else 'count' if any(x in key for x in ['objects','count','selected','hints','trials','fetches','calls','comparisons','skips','descriptors','predecessors','targets','ranges','entries','files']) else 'source-defined scalar'
        if key.startswith('cgroup_') and any(x in key for x in ['memory_current','memory_peak','swap_current']): unit='bytes'
        elif key.startswith('cgroup_') and unit=='source-defined scalar': unit='count'
        elif any(x in key for x in ['rejected_mixed_groups','usable_bases','transactions','requests_cumulative']) or key in ['index','ordinal']: unit='count'
        elif key=='created' or key.endswith('_success'): unit='boolean'
        elif key=='persisted_new_pack_ids': unit='list of pack IDs'
        elif unit=='source-defined scalar': unit='text'
        status='derived' if key.startswith('change_') or (key.startswith('cgroup_') and '_ns_' in key) or key in ['allocation_adjustment_bytes','step_unattributed_ns','prior_mapping'] else 'measured'
        null_reason='Not recorded for this checkpoint/phase; Init has no source checkpoint, candidate receipt or prior growth.'
        if key in ['observer_ns','free_disk_bytes','persisted_new_pack_ids','first_admission_provenance','preparation_ns']:
            status='unknown';null_reason={'observer_ns':'Observer time not independently timed; unattributed wall residual is not pure observer time.','free_disk_bytes':'50GiB reserve checked in source, exact samples not persisted.','persisted_new_pack_ids':'Receipts contain no authoritative pack/location admission checkpoint.','first_admission_provenance':'Final pack ordering cannot establish admission chronology.','preparation_ns':'Only invocation-level preparation exists; not assignable per checkpoint.'}[key]
        population='checkpoint/source identity or acknowledgement state'
        if '_physical_storage_' in key: population='shared Store interval delta for named public phase; trial events overlap; FULL/DELTA selected are admitted winners; encoding alternatives include race losers'
        elif key.startswith('cgroup_'): population='container lifetime cumulative CPU/events/peak or checkpoint-boundary memory current; boundary_delta spans prior sample through transfer/public work and observers, not isolated operation; nested memory categories overlap'
        elif key.startswith(('host_runtime_disk','spool_disk','container_staging')): population='post-step boundary sample; host runtime includes Store and spool; do not sum overlapping scopes'
        elif 'host_lifetime' in key: population='host process lifetime peak, not phase-exclusive'
        elif key.endswith('_cumulative'): population='cumulative process counter; not operation delta'
        fields[key]={'unit':unit,'population':population,'snapshot':'row snapshot; phase prefix if present','provenance':'row provenance; physical counters from named phase receipt; candidate fields parsed exact CandidateStats; cgroup ns fields derived from source usec * 1000 and boundary_delta subtracts prior sample','status':status,'null_reason':null_reason}
    with (output/'checkpoint-trajectory.csv').open('x',newline='') as f:
        writer=csv.DictWriter(f,fieldnames=keys);writer.writeheader();writer.writerows({k:'null' if r.get(k) is None else r[k] for k in keys} for r in rows)
    write_json(output/'checkpoint-fields.json',fields)
    timing={}
    for mode,data in [('performance',d),('verification',v)]:
        summary=read(run/(mode+'-summary.json'))
        pp=[p for s in data['records'] for p in stats(s['receipts']).values()]
        timing[mode]={'units':'ns','status':'measured unless named residual','provenance':f'{mode}-summary.json and deepseek-full/{mode}-result.json','preparation_ns':summary['preparation_ns'],'enclosing_invocation_ns':summary['wall_ns'],'case_wall_ns':data['wall_ns'],'work_wall_ns':data['work_wall_ns'],'setup_ns':data['setup_ns'],'cleanup_ns':data['cleanup_ns'],'transfer_ns':sum(s['transfer_ns'] for s in data['records']) if mode=='performance' else None,'public_phases_population':'checkpoint records only; ready/closed phases separately recorded','ready_phases_ns':{p:r['elapsed_ns'] for p,r in stats(data['ready']).items()},'closed_phases_ns':{p:r['elapsed_ns'] for p,r in stats(data.get('closed',[])).items()},'public_phases_ns':{p:sum(x['elapsed_ns'] for x in pp if x['phase']==p) for p in {x['phase'] for x in pp}},'step_wall_sum_ns':sum(s['step_wall_ns'] for s in data['records']),'outside_step_work_residual_ns':data['work_wall_ns']-sum(s['step_wall_ns'] for s in data['records']),'invocation_after_preparation_outside_case_residual_ns':summary['wall_ns']-summary['preparation_ns']-data['wall_ns'],'observer_ns':None,'observer_null_reason':'Not separately timed; work residual includes boundary observers, pending/final receipt writes, cgroup reads and orchestration. Step residual includes allocation observation, IPC and formatting. Neither is pure observer.'}
    totals={k:sum(bucket.get(k,0) for bucket in sums.values()) for k in next(iter(sums.values()))}
    reconciliation={'schema':'issue87-receipt-reconciliation-v1','timing':timing,'physical_by_phase':sums,'physical_ready_and_checkpoint_phase_totals':totals,'counter_units':'suffix bytes/ns else count','counter_population':'shared Store phase interval deltas; no summation of duplicated Commit textual receipt','source_mechanism':'crates/layerfs-layerstack-store/src/telemetry.rs; objects/admission.rs','checks':{'checkpoint_count':157,'plus_init_rows':158,'candidate_equations':'PASS','allocation_equations':'PASS','growth_matches_inserted':'PASS','all157_mapping_source_oracle':'PASS','final_objects':rows[-1]['canonical_objects'],'final_canonical_bytes':rows[-1]['canonical_bytes'],'created_count':sum(bool(r.get('created')) for r in rows),'up_to_date_count':sum(not s['created'] for s in d['records'])},'durable_savings_bytes':None,'durable_savings_null_reason':'Attempted alternatives/selected encoding bytes may include race losers and omit outer framing; durable group census is separate.','eligible_target_bytes':None,'eligible_target_bytes_null_reason':'Counter lacks byte-weighted unique initially missing eligible target denominator and mutually exclusive terminal outcomes.'}
    write_json(output/'counter-reconciliation.json',reconciliation)
    outcomes=['no predecessor','missing required span (handoff defect)','no overlap','correspondence/descriptor limit','unavailable/inadmissible base','fetch/match/trial/instruction/memory budget','no useful raw delta','compressed mixed rejection','DELTA admitted','admission race selected existing representation']
    with (output/'delta-opportunity.csv').open('x',newline='') as f:
        w=csv.DictWriter(f,fieldnames=['outcome','target_count','canonical_bytes','unit','population','snapshot','status','provenance','null_reason','decision']);w.writeheader()
        for outcome in outcomes:w.writerow(dict(outcome=outcome,target_count='null',canonical_bytes='null',unit='count and bytes',population='unique initially missing eligible validated payload targets after CAS filtering; mutually exclusive final target outcome',snapshot='prospective diagnostic; unavailable retrospectively',status='unknown',provenance='existing physical receipts lack exhaustive target terminal outcome and byte weights',null_reason='Overlapping event counters cannot reconstruct this exact population',decision='Distinguish missing coverage from base/matcher budgets, raw candidate quality, mixed rejection and race loss before selecting optimization'))
    print(json.dumps({'checks':reconciliation['checks'],'timing':timing,'totals':totals},indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('run',type=pathlib.Path);p.add_argument('output',type=pathlib.Path);p.add_argument('frozen',type=pathlib.Path);a=p.parse_args();main(a.run,a.output,a.frozen)

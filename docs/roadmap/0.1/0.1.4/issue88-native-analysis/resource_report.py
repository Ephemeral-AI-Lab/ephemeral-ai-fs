#!/usr/bin/env python3
"""Receipt-only full157 timing/resource report; never opens a Store or inventory.

One CSV per arm contains performance Init+157 checkpoints and verification's157
historical checkpoints. Field contract supplies units/population/scope/provenance;
per-row status and null-reason maps distinguish unavailable from measured zero.
"""
import argparse
import csv
import hashlib
import json
import pathlib

PHASES = {'init':'public_init_ns','fork':'public_fork_ns','mount':'public_mount_ns','exec':'public_exec_ns','commit':'public_commit_ns','end':'public_end_ns','verify-mount':'verification_mount_ns','verify-end':'verification_end_ns'}
HOST = ('host_cpu_ns','host_disk_read_bytes','host_disk_write_bytes')
GAUGES = {'host_rss_bytes':'host_sampled_current_rss_max_bytes','host_lifetime_peak_rss_bytes':'host_lifetime_peak_rss_max_bytes','host_footprint_bytes':'host_sampled_footprint_max_bytes'}
CPU = ('usage','user','system','throttled')
MEMORY = ('anon','file','kernel','shmem','slab','file_dirty','file_writeback')


def sha(path):
    with pathlib.Path(path).open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest()


def load(path):
    return json.loads(pathlib.Path(path).read_text())


def require(value,message):
    if not value:
        raise ValueError(message)


def integer(value):
    return type(value) is int and value >= 0


def save(path,value):
    with path.open('x') as stream:
        json.dump(value,stream,indent=2,sort_keys=True)
        stream.write('\n')


def measured(value,reason):
    if integer(value):return value,'measured',None
    return (None,'unknown',reason) if value is None else (None,'invalid','recorded field is not a nonnegative integer')


def aggregate(values,how='sum'):
    if not values:
        return None,'not_applicable','no matching public phase in this receipt population'
    if any(v is not None and not integer(v) for v in values):
        return None,'invalid','recorded phase field is not a nonnegative integer; aggregation refused'
    if any(v is None for v in values):
        return None,'unknown','one or more phase fields missing; incomplete sum/max forbidden'
    return (sum(values) if how=='sum' else max(values)),'derived',None


def difference(after,before):
    if not integer(after) or not integer(before):
        return None,'unknown','missing/malformed boundary; delta unavailable'
    if after < before:
        return None,'invalid','counter regression; not silently clamped'
    return after-before,'derived',None


def phases(receipts):
    return [r for r in receipts if r.get('kind')=='storage-smoke-phase']


def checkpoint(arm,mode,step,previous_cgroup,provenance):
    row={'arm':arm,'mode':mode,'checkpoint':step['index'],'source_sha':step.get('sha'),'layerfs_mapping':step.get('commit_id',step.get('identity')),'provenance':provenance}
    statuses={'source_sha':'measured' if step.get('sha') else 'unknown'}; nulls={}
    if not step.get('sha'):nulls['source_sha']='Init has no source SHA' if step['index']==0 else 'verification receipt carries only LayerFS identity; source SHA not joined in this resource-only report'
    def put(name,result):
        value,status,reason=result;row[name]=value;statuses[name]=status
        if reason:nulls[name]=reason
    ps=phases(step.get('receipts',[]))
    for phase,name in PHASES.items():
        put(name,aggregate([p.get('elapsed_ns') for p in ps if p.get('phase')==phase]))
    put('public_phase_elapsed_sum_ns',aggregate([p.get('elapsed_ns') for p in ps]))
    for name in HOST:put(name,aggregate([p.get(name) for p in ps]))
    for source,name in GAUGES.items():put(name,aggregate([p.get(source) for p in ps],'max'))
    for name in ('step_wall_ns','transfer_ns'):
        put(name,measured(step.get(name),'not recorded for this checkpoint/mode'))
    components=[row.get('public_phase_elapsed_sum_ns'),row.get('transfer_ns')]
    if mode=='verification':components=[row.get('public_phase_elapsed_sum_ns')]
    if integer(row.get('step_wall_ns')) and all(integer(v) for v in components):
        residual=row['step_wall_ns']-sum(components)
        put('step_unattributed_ns',(residual,'derived','includes receipt/observer/control work; not pure observer timing') if residual>=0 else (None,'invalid','component sum exceeds enclosing step wall'))
    else:put('step_unattributed_ns',(None,'unknown','step/component duration missing'))
    cg=step.get('cgroup',{})
    for name in CPU:
        current=cg.get(name+'_usec'); prior=previous_cgroup.get(name+'_usec')
        value,status,reason=measured(current,'container cumulative CPU field absent')
        put('container_'+name+'_cumulative_ns',(value*1000 if value is not None else None,'derived' if value is not None else status,reason))
        value,status,reason=difference(current,prior)
        put('container_'+name+'_observation_delta_ns',(value*1000 if value is not None else None,status,reason))
    for source,name in {'memory_current':'container_memory_current_bytes','memory_peak':'container_lifetime_memory_peak_bytes','swap_current':'container_swap_current_bytes','oom':'container_oom_count_cumulative','oom_kill':'container_oom_kill_count_cumulative','nr_throttled':'container_throttled_periods_cumulative'}.items():
        put(name,measured(cg.get(source),'container boundary field absent'))
    for name in MEMORY:put('container_'+name+'_bytes',measured(cg.get('memory_stat_bytes',{}).get(name),'memory.stat category absent'))
    for name,reason in {'container_rss_bytes':'not sampled: cgroup memory.current includes file cache/kernel and is not process RSS','container_io_read_bytes':'io.stat/block I/O not recorded by normal sampler','container_io_write_bytes':'io.stat/block I/O not recorded by normal sampler','free_disk_bytes':'normal wrapper enforces a free-space threshold but does not persist per-checkpoint free bytes','observer_elapsed_ns':'cgroup/disk/receipt observation time not separately timed by normal wrapper'}.items():put(name,(None,'unknown',reason))
    for scope in ('host_runtime','spool'):
        for quantity in ('allocated','apparent'):put(scope+'_'+quantity+'_bytes',measured(step.get(scope+'_disk',{}).get(quantity+'_bytes'),'disk scope not sampled in this row'))
    put('container_staging_allocated_bytes',measured(step.get('container_staging_allocated_bytes'),'staging sample absent'))
    acks=[r for r in step.get('receipts',[]) if r.get('kind')=='storage-smoke-allocation']
    for field in ('store_allocated_bytes','store_apparent_bytes','database_logical_bytes','database_allocated_bytes','sidecar_allocated_bytes','sidecar_logical_bytes'):
        put('ack_'+field,measured(acks[0].get(field) if len(acks)==1 else None,'unique acknowledgement allocation unavailable; verification/current stat cannot replace it'))
    row['field_status_json']=json.dumps(statuses,sort_keys=True,separators=(',',':'))
    row['null_reason_json']=json.dumps(nulls,sort_keys=True,separators=(',',':'))
    return row


def phase_totals(receipts):
    ps=phases(receipts);result={}
    for phase in dict.fromkeys(p['phase'] for p in ps):
        selected=[p for p in ps if p['phase']==phase]
        result[phase]={'calls_count':len(selected),'elapsed_ns':aggregate([p.get('elapsed_ns') for p in selected])[0],**{name:aggregate([p.get(name) for p in selected])[0] for name in HOST},**{name:aggregate([p.get(source) for p in selected],'max')[0] for source,name in GAUGES.items()}}
    return result


def frozen_json(run,relative,manifest,inputs):
    path=run/relative
    require(path.is_file(),'missing normal receipt '+str(path))
    digest=sha(path)
    require(manifest.get(relative)==digest,'normal receipt manifest mismatch '+relative)
    inputs[str(path)]=digest
    return load(path)


def mode_report(arm,mode,run,inputs):
    manifest_path=run/(mode+'-manifest.json'); require(manifest_path.is_file(),'missing '+mode+' manifest')
    manifest=load(manifest_path);inputs[str(manifest_path)]=sha(manifest_path)
    relative='deepseek-full/'+mode+'-result.json'
    data=frozen_json(run,relative,manifest,inputs)
    summary=frozen_json(run,mode+'-summary.json',manifest,inputs)
    require(data['mode']==mode and data['case']=='deepseek-full','receipt mode/population')
    require([r['index'] for r in data['records']]==list(range(1,158)),'full157 receipt cardinality/order')
    ready=data.get('ready',[]);rows=[]
    if mode=='performance':
        ready_mapping=next((r.get('layer_id') for r in ready if r.get('kind')=='storage-smoke-ready'),None)
        rows.append(checkpoint(arm,mode,{'index':0,'identity':ready_mapping,'receipts':ready},{},str(run/relative)+'#ready'))
    previous=data.get('cgroup_before',{})
    for index,step in enumerate(data['records']):
        rows.append(checkpoint(arm,mode,step,previous,str(run/relative)+'#records/'+str(index)))
        previous=step.get('cgroup',{})
    all_receipts=ready+[p for r in data['records'] for p in r.get('receipts',[])]+data.get('closed',[])
    step_sum=aggregate([r.get('step_wall_ns') for r in data['records']])[0]
    work_difference=difference(data.get('work_wall_ns'),step_sum)
    phase=phase_totals(all_receipts)
    invalid_phase_fields=any(p.get(k) is not None and not integer(p[k]) for p in phases(all_receipts) for k in ('elapsed_ns',)+HOST+tuple(GAUGES))
    close_ns=aggregate([r.get('elapsed_ns') for r in phases(data.get('closed',[]))])[0]
    if not phases(data.get('closed',[])):close_ns=0  # No public close phase actually emitted in this mode.
    outer_parts=[data.get(k) for k in ('setup_ns','work_wall_ns','cleanup_ns')]+[close_ns]
    outer_residual=difference(data.get('wall_ns'),sum(outer_parts) if all(integer(v) for v in outer_parts) else None)
    return rows,{'invalid_public_phase_fields':invalid_phase_fields,'status':data.get('status'),'cleanup_status':data.get('cleanup_status'),'container_removed':data.get('container_removed'),'normal_summary_status':summary.get('status'),'checkpoint_rows_count':len(rows),'preparation_ns':measured(summary.get('preparation_ns'),'summary preparation missing')[0],'enclosing_invocation_ns':measured(summary.get('wall_ns'),'summary wall missing')[0],'case_wall_ns':measured(data.get('wall_ns'),'case wall missing')[0],'case_setup_ns':measured(data.get('setup_ns'),'case setup missing')[0],'case_work_wall_ns':measured(data.get('work_wall_ns'),'case work wall missing')[0],'case_cleanup_ns':measured(data.get('cleanup_ns'),'case cleanup missing')[0],'step_wall_sum_ns':step_sum,'public_close_phase_sum_ns':close_ns,'case_unattributed_ns':outer_residual[0],'case_unattributed_status':outer_residual[1],'case_unattributed_scope':'case wall minus setup/work/cleanup and named close public phases; retains coordinator close/output/observer residual','transfer_sum_ns':aggregate([r.get('transfer_ns') for r in data['records']])[0] if mode=='performance' else None,'work_outside_step_walls_ns':work_difference[0],'work_outside_step_walls_status':work_difference[1],'work_outside_step_walls_scope':'includes initial cgroup observation, per-step cgroup/disk/staging observations and receipt persistence; not pure observer time','public_phases':phase,'public_phase_location':'Init/fork/mount are nested in case_setup; per-step phases in step_wall; end in close. Do not add public phases to enclosing timers.','container_before_work_observed':data.get('cgroup_before'),'container_before_work_scope':'after host ready/setup, before checkpoint work begins','container_final_observed':previous,'container_final_observed_scope':'last checkpoint boundary before close/container cleanup; not final-lifetime endpoint','sampled_disk_maxima':{key:aggregate([r.get(key) for r in rows if r['checkpoint']>0],'max')[0] for key in ('host_runtime_allocated_bytes','spool_allocated_bytes','container_staging_allocated_bytes')},'environment':data.get('environment'),'provenance':{'result':str(run/relative),'summary':str(run/(mode+'-summary.json'))}}


def field_contract(columns):
    result={}
    for name in columns:
        unit='ns' if name.endswith('_ns') else 'bytes' if name.endswith('_bytes') else 'count' if 'count' in name or 'requests' in name or 'periods' in name or name=='checkpoint' else 'text/structured JSON'
        scope='checkpoint identified by arm/mode/index; status/null reason in row maps'
        if name.startswith('container_'):scope='sample after checkpoint; cumulative counters since container start; derived CPU delta since previous sample (includes control/observer work), not public-operation-local'
        if 'lifetime' in name:scope='cumulative process/container lifetime high-water; never sum or claim phase-local peak'
        if name.startswith('host_runtime_'):scope='host-runtime tree sample already includes Store and spool; do not add either scope to it'
        if name.startswith('spool_'):scope='host-runtime/tmp subtree sample, nested within host-runtime'
        if name.startswith('ack_'):scope='primary acknowledgement allocation, performance only; not current stat or copied Store allocation'
        if name in HOST:scope='sum of host per-public-phase process counter deltas; not full-case CPU/I/O; producer disk counters use saturating subtraction'
        result[name]={'unit':unit,'population':scope,'provenance':'row provenance plus normal storage-smoke source field names; direct or derived status is explicit in field_status_json','missing':'empty CSV cell corresponds to null with explicit reason; never zero-fill'}
    return result


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('control','candidate','schedule','output'):p.add_argument('--'+name,type=pathlib.Path,required=True)
    for arm in ('control','candidate'):
        for kind in ('snapshot','census'):p.add_argument('--'+arm+'-'+kind+'-custody',type=pathlib.Path)
    args=p.parse_args();out=args.output.resolve();require(not out.exists(),'new output required')
    schedule=load(args.schedule);require(schedule['schema']=='issue88-SP-full157-frozen-v1','schedule schema')
    require([r['arm'] for r in schedule['order']]==['control','candidate'],'schedule arm order')
    inputs={str(args.schedule.resolve()):sha(args.schedule)};summaries={};trajectories={}
    for arm in ('control','candidate'):
        run=getattr(args,arm).resolve();require(out!=run and not out.is_relative_to(run),'output must be outside original runs')
        frozen=next(r for r in schedule['order'] if r['arm']==arm)
        require(pathlib.Path(frozen['output']).resolve()==run,'run differs from frozen arm')
        identity_path=run/'identity.json';identity=load(identity_path)
        require(load(run/'performance-manifest.json').get('identity.json')==sha(identity_path),'identity manifest seal mismatch')
        inputs[str(identity_path)]=sha(identity_path)
        require(identity['host_identity']==frozen['host'] and identity['image_id']==frozen['image_id'],'frozen host/image mismatch')
        rows=[];summaries[arm]={'producer_identity':{'host_identity':identity['host_identity'],'image_id':identity['image_id'],'source':identity.get('source')}}
        for mode in ('performance','verification'):
            found,report=mode_report(arm,mode,run,inputs);rows.extend(found);summaries[arm][mode]=report
        trajectories[arm]=rows
        observers={}
        for kind,key in (('snapshot','observer_copy_hash_ns'),('census','elapsed_ns')):
            path=getattr(args,arm+'_'+kind+'_custody')
            if path is None:observers[kind]={'elapsed_ns':None,'status':'unknown','reason':'explicit observer custody not supplied'};continue
            data=load(path);inputs[str(path.resolve())]=sha(path)
            require(data.get('status')=='PASS','observer custody did not complete')
            if kind=='snapshot':
                require(pathlib.Path(data['source']).resolve()==run/'deepseek-full/host-runtime/store.sqlite','snapshot observer source arm mismatch')
                require(data['frozen_schedule_sha256']==sha(args.schedule),'snapshot observer schedule mismatch')
            else:
                require(data.get('arm')==arm and data.get('exit_code')==0,'census observer arm/exit mismatch')
                reference=data['snapshot_custody']; custody_path=pathlib.Path(reference['path']).resolve()
                require(sha(custody_path)==reference['sha256'],'census snapshot custody seal mismatch')
                inputs[str(custody_path)]=reference['sha256']; custody=load(custody_path)
                require(custody.get('status')=='PASS' and custody.get('snapshot_phase')=='final-pre-verification','census snapshot custody phase/status')
                require(pathlib.Path(custody['source']).resolve()==run/'deepseek-full/host-runtime/store.sqlite','census timing belongs to another run')
                require(custody['frozen_schedule_sha256']==sha(args.schedule),'census timing schedule mismatch')
                decoder=data['decoder_binary']
                require(decoder['sha256']==schedule['census']['sha256'] and pathlib.Path(decoder['path']).resolve()==pathlib.Path(schedule['census']['binary']).resolve(),'census timing decoder differs from frozen census')
                snapshot_path=custody_path.parent/'store.sqlite'
                require(pathlib.Path(data['snapshot']['path']).resolve()==snapshot_path and data['snapshot']['sha256']==custody['copy_sha256']==custody['source_sha256'],'census timing snapshot link mismatch')
                inventory_path=pathlib.Path(data['inventory']['path']).resolve()
                require(data['command']==[str(pathlib.Path(decoder['path']).resolve()),str(snapshot_path),str(inventory_path.parent)],'census timing command/input links mismatch')
            value,status,reason=measured(data.get(key),'observer duration absent or invalid')
            observers[kind]={'elapsed_ns':value,'status':status,'reason':reason,'provenance':str(path.resolve()),'scope':'outside performance public intervals; cannot add to case wall unless separately enclosed; large snapshot/inventory/binary authentication remains owned by upstream census proof, not repeated here'}
        summaries[arm]['observers']=observers
    for path,digest in inputs.items():
        require(sha(path)==digest,'small receipt/custody/schedule input changed during report: '+path)
    invalid_rows=any('invalid' in json.loads(row['field_status_json']).values() for rows in trajectories.values() for row in rows)
    invalid_summaries=any(summaries[a][m].get('invalid_public_phase_fields') for a in summaries for m in ('performance','verification')) or any(summaries[a][m].get(k)=='invalid' for a in summaries for m in ('performance','verification') for k in ('work_outside_step_walls_status','case_unattributed_status'))
    report_status='INVALID' if invalid_rows or invalid_summaries else ('PASS' if all(summaries[a][m]['status']==summaries[a][m]['cleanup_status']==summaries[a][m]['normal_summary_status']=='PASS' and summaries[a][m]['container_removed'] is True for a in summaries for m in ('performance','verification')) else 'INCOMPLETE')
    out.mkdir(parents=True)
    columns=list(trajectories['control'][0])
    for arm,rows in trajectories.items():
        with (out/(arm+'-checkpoint-resources.csv')).open('x',newline='') as stream:
            writer=csv.DictWriter(stream,fieldnames=columns);writer.writeheader();writer.writerows(rows)
    save(out/'field-contract.json',field_contract(columns))
    save(out/'resource-timing-summary.json',{'schema':'issue88-combined-resources-v1','status':report_status,'invalid_counter_or_residual':invalid_rows or invalid_summaries,'input_hashes':inputs,'status_scope':'normal run completion plus source/report custody; unavailable resource quantities remain explicit and are not implied PASS measurements','arms':summaries,'frozen_campaign_free_bytes_before':schedule.get('free_bytes_before'),'frozen_campaign_free_scope':'single pre-campaign schedule observation; not a per-checkpoint minimum','units':'integer bytes/ns/counts','comparison_scope':'one frozen sequential matched pair; not repeatability or historical timing pairs','allocation_warning':'host-runtime contains Store and spool; scopes are not additive','unavailable':{'container_process_rss':'not sampled; cgroup total is not RSS','container_block_io':'normal sampler did not collect io.stat','per_checkpoint_free_disk':'threshold checked but observed byte values not persisted','pure_observer_time':'normal sampler does not separately time observers; residual remains unattributed'},'source_sampling_provenance':'benchmark/fs-bench-pro/shared/storage_smoke.py run_case records step_wall before cgroup/disk/staging sampling; work_wall includes those observations. src/storage_smoke.rs timed snapshots host resources around named public phases.'})
    for path,digest in inputs.items():
        require(sha(path)==digest,'small input changed before output sealing: '+path)
    save(out/'manifest.sha256.json',{p.name:sha(p) for p in sorted(out.iterdir()) if p.is_file()})
    print(json.dumps({'output':str(out),'rows_per_arm':{a:len(r) for a,r in trajectories.items()}}))


if __name__=='__main__':main()

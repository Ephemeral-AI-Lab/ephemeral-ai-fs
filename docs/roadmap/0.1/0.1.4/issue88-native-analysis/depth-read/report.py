#!/usr/bin/env python3
"""Report sealed depth-read results only: report.py RUN NEW_REPORT_DIRECTORY.

No Store/inventory/codec access, source resealing or automatic rerun. A partial
campaign retains60 scheduled rows with explicit UNRUN/INCOMPLETE states; all
comparison statistics are withheld unless the complete campaign validates.
"""
import csv
import hashlib
import json
import pathlib
import sys

PHYSICAL = ('group_fetches','blob_ranges','encoded_read_bytes','decoded_read_bytes',
 'decompression_calls','base_fetches','native_record_fetches','native_request_bytes',
 'native_parser_bytes','native_raw_decoded_bytes','native_decode_calls','native_decode_ns',
 'native_dependency_edges',*(f'native_depth_{d}' for d in range(5)))
HOST = ('elapsed_ns','acknowledgement_window_ns','host_cpu_ns','host_rss_bytes',
 'host_lifetime_peak_rss_bytes','host_footprint_bytes','host_disk_read_bytes','host_disk_write_bytes',
 'public_exec_count','workspace_output_reader_count','output_read_count','shell_process_count',
 'dd_process_count','sink_process_count','free_disk_bytes','scoped_runtime_allocated_bytes','owned_output_allocated_bytes')
CELL_METRICS = ('read_elapsed_ns','read_host_cpu_ns','read_host_rss_bytes','read_host_lifetime_peak_rss_bytes',
 'read_host_disk_read_bytes','read_host_disk_write_bytes',*(f'read_{name}' for name in PHYSICAL),
 'read_cgroup_boundary_cpu_ns','read_container_current_bytes','read_container_lifetime_peak_bytes',
 'read_cgroup_observer_ns','read_scoped_runtime_allocated_bytes','read_free_disk_bytes',
 'digest_elapsed_ns','setup_wrapper_ns','setup_host_ns','end_host_ns','cleanup_wrapper_ns','invocation_wall_ns')


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream,'sha256').hexdigest()


def require(ok, text):
    if not ok:
        raise ValueError(text)


def stats(values):
    require(len(values)==3 and all(type(v) is int for v in values),'exact3 integer observations required')
    ordered=sorted(values)
    return dict(n=3,min=ordered[0],median=ordered[1],max=ordered[2])


def main(run,out):
    require(not out.exists() and not out.is_relative_to(run),'new report outside sealed campaign required')
    out.mkdir(parents=True)
    errors=[]; input_hashes={}
    def load(path):
        try:
            value=json.loads(path.read_text())
            input_hashes[str(path)]=sha(path)
            return value
        except (OSError,ValueError) as error:
            errors.append(str(path)+': '+str(error));return None
    manifest=load(run/'manifest.sha256.json')
    def sealed(name):
        path=run/name
        value=load(path)
        if value is not None and (not isinstance(manifest,dict) or manifest.get(name)!=input_hashes[str(path)]):
            errors.append('campaign manifest mismatch: '+name)
        return value
    command=sealed('command.json'); summary=sealed('summary.json'); identities=sealed('identity.json'); copies=sealed('copy-custody.json')
    cohort=None; cohort_details=[]
    if command is not None:
        try:
            reference=command['cohort']; cohort=load(pathlib.Path(reference['path']))
            require(cohort is not None and input_hashes[reference['path']]==reference['sha256'],'cohort binding')
            require(cohort['schema']=='issue88-depth-read-cohort-v1' and cohort['status']=='PASS','cohort incomplete')
            require([s['depth'] for s in cohort['selections']]==list(range(5)),'five depth strata')
            for name in ('contract','reference_contract'):
                ref=command[name]; path=pathlib.Path(ref['path']);input_hashes[str(path)]=sha(path)
                require(input_hashes[str(path)]==ref['sha256'],name+' hash mismatch')
            require(command['contract']==cohort['inputs']['contract'],'governing cohort contract binding')
            ref=command['prepared_copies']; prepared=load(pathlib.Path(ref['path']))
            require(prepared==copies and input_hashes[ref['path']]==ref['sha256'],'prepared-copy custody binding')
            for selected in cohort['selections']:
                ref=selected['proof']; proof=load(pathlib.Path(ref['path']))
                require(proof is not None and input_hashes[ref['path']]==ref['sha256'] and proof['status']=='PASS','selected path proof changed/incomplete')
                composition=proof['candidate_file_representation_logical_bytes']
                require(sum(composition.values())==selected['file_length_bytes'],'file composition conservation')
                cohort_details.append(dict(depth=selected['depth'],checkpoint=selected['checkpoint'],file_length_bytes=selected['file_length_bytes'],offset_bytes=selected['offset_bytes'],closure_raw_bytes=selected['candidate']['closure_raw_bytes'],candidate_extent_count=len(proof['arms']['candidate']['descriptors']),candidate_file_representation_logical_bytes=composition))
        except (KeyError,ValueError,TypeError,OSError) as error:
            errors.append('campaign inputs: '+str(error))
    # This fixed order is independently reconstructed, not trusted from the runner.
    planned=[]
    for depth in range(5):
        selected=cohort['selections'][depth] if cohort is not None and len(cohort.get('selections',[]))==5 else {}
        for operation in ('range','full'):
            for repetition in (1,2,3):
                for arm in (('control','candidate') if repetition!=2 else ('candidate','control')):
                    planned.append(dict(arm=arm,depth=depth,checkpoint=selected.get('checkpoint'),operation=operation,
                     repetition=repetition,path_hex=selected.get('path_hex'),file_length_bytes=selected.get('file_length_bytes'),
                     offset_bytes=selected.get('offset_bytes') if operation=='range' else 0,
                     requested_bytes=4096 if operation=='range' else selected.get('file_length_bytes'),
                     expected_sha256=selected.get('expected_range_sha256' if operation=='range' else 'expected_full_sha256')))
    if command is None or command.get('schedule')!=planned: errors.append('exact60-row frozen schedule/order mismatch')
    if identities is not None and summary is not None:
        try:
            require(set(identities)=={'control','candidate'},'two exact identity arms')
            for arm in identities:
                identity=identities[arm]; final=summary['sources'][arm]
                require(final['source_unchanged'] is True and final['source_seal_after']==identity['source_seal_before'],'reported original seals changed: '+arm)
                for key in ('LAYERFS_SOURCE_SEAL','LAYERFS_PRODUCT_SEAL'):
                    require(identity['probe_identity'][key]==identity['producer_identity']['source'][key],'probe product/source mismatch')
                require(identity['image']==identity['producer_identity']['image_id'],'same frozen image')
                require(identity['probe_identity']['tool_files']==command['tool_files'],'probe tool source identity')
                require(identity['binary']==command['arm_config'][arm]['probe_binary'],'probe executable identity')
                require(copies['copies'][arm]['post_proof_sha256']==cohort['copies'][arm]['sha256'] and copies['copies'][arm]['path']==cohort['copies'][arm]['path'],'proof/campaign copy identity')
        except (KeyError,ValueError,TypeError) as error: errors.append('identity/seal reconciliation: '+str(error))
    rows=[]; utility=None
    for index,wanted in enumerate(planned,1):
        row=dict(wanted,row_index=index,status='UNRUN',errors=[],null_reason='scheduled observation has no result')
        path=run/f'row-{index:02d}/result.json'
        if not path.exists():
            rows.append(row);continue
        data=sealed(f'row-{index:02d}/result.json')
        row['status']='INCOMPLETE'; row['null_reason']='failed or invalid observation; available raw counters preserved'
        if data is None:
            row['errors'].append('missing/malformed result');rows.append(row);continue
        local=row['errors']
        def number(value,name):
            if type(value) is not int or value<0:
                local.append('missing/invalid integer '+name);return None
            return value
        def check(ok,message):
            if not ok: local.append(message)
        def bounded(value,limit):
            return type(value) is int and 0<=value<=limit
        check(all(data.get(k)==v for k,v in wanted.items()),'scheduled identity/range/order mismatch')
        check(data.get('row_index')==index,'row index')
        check(data.get('status')=='PASS' and data.get('cleanup_status')=='PASS' and data.get('container_removed') is True,'row/cleanup/container incomplete')
        check(data.get('admission_eligible') is False and data.get('allocation_comparison_eligible') is False,'wrong measurement eligibility')
        check(data.get('cache_profile')=='fresh-application-fuse-context-existing-os-cache-uncontrolled','cache profile')
        if utility is None: utility=data.get('utility_identity')
        check(isinstance(utility,list) and len(utility)==4 and data.get('utility_identity')==utility,'utility identity mismatch')
        for source,dest in [('setup_ns','setup_wrapper_ns'),('cleanup_ns','cleanup_wrapper_ns'),('invocation_wall_ns','invocation_wall_ns')]: row[dest]=number(data.get(source),dest)
        row['container_staging_allocated_bytes']=number(data.get('container_staging_allocated_bytes'),'staging')
        row['final_scoped_runtime_allocated_bytes']=number(data.get('final_scoped_runtime_allocated_bytes'),'final runtime')
        try:
            ready=data['ready']; closed=data['closed']; host=data['environment']['host_config']
            check(host['Memory']==host['MemorySwap']==2*1024**3 and host['NanoCpus']==2_000_000_000 and host['PidsLimit']==256,'container resource profile')
            check(data['environment']['validated'] is True,'container inspection')
            check(ready['checkpoint_index']==wanted['checkpoint'] and ready['selected_depth']==wanted['depth'] and ready['path_hex']==wanted['path_hex'],'ready cohort identity')
            check(ready['offset_bytes']==wanted['offset_bytes'] and ready['requested_bytes']==wanted['requested_bytes'] and ready['file_length_bytes']==wanted['file_length_bytes'] and ready['mount_proof']=='PASS','ready range/mount')
            for key in ('native_record_fetches','native_decode_calls'):
                row['setup_'+key]=number(ready['setup_physical_storage'][key],'setup '+key)
                check(row['setup_'+key]==0,'setup already read native payload')
            row['setup_host_ns']=number(ready['setup_ns'],'host setup');row['end_host_ns']=number(closed['cleanup_ns'],'host End')
            check(all(closed[k] is True for k in ('success','cleanup_ok','unchanged_head')) and closed['commit_call_count']==0,'CleanEnd/head/public Commit')
            fuse=closed['session_fuse'];check(fuse['present'] is True and fuse['read_status']=='measured','session FUSE receipt absent')
            row['session_fuse_read_requests_count']=number(fuse['read_requests'],'session FUSE requests')
            row['session_fuse_read_bytes']=number(fuse['read_bytes'],'session FUSE bytes')
            row['session_fuse_write_requests']=fuse['write_requests'];row['session_fuse_write_bytes']=fuse['write_bytes']
            check(fuse['write_requests'] is None and fuse['write_bytes'] is None,'unavailable session write counters misreported')
            input_row=sealed(f'row-{index:02d}/input.json')
            selected=cohort['selections'][wanted['depth']][wanted['arm']]
            check(input_row is not None and input_row.get('schema')=='issue88-depth-read-row-v1' and input_row.get('row_index')==index and all(input_row.get(k)==v for k,v in wanted.items()) and input_row['branch_id']==selected['branch_id'] and input_row['commit_id']==selected['commit_id'],'probe input mapping mismatch')
            check(data['command'][0]==identities[wanted['arm']]['binary'] and pathlib.Path(data['command'][1])==pathlib.Path(copies['copies'][wanted['arm']]['path']).parent,'actual probe/copy command mismatch')
        except (KeyError,TypeError,IndexError) as error: local.append('setup/identity/End fields: '+str(error))
        for phase in ('read','digest'):
            observed=data.get(phase)
            if not isinstance(observed,dict): local.append('phase missing: '+phase);continue
            for key in HOST: row[phase+'_'+key]=number(observed.get(key),phase+' '+key)
            physical=observed.get('physical_storage',{})
            for key in PHYSICAL: row[phase+'_'+key]=number(physical.get(key),phase+' '+key)
            check(observed.get('correct') is True and observed.get('resource_status')=='PASS','phase correctness/resource: '+phase)
            check(observed.get('expected_bytes')==wanted['requested_bytes'],'phase requested count')
            expected=str(wanted['requested_bytes']) if phase=='read' else wanted['expected_sha256']
            actual=observed.get('output','').split()
            check(bool(actual) and actual[0]==expected,'phase output count/digest')
            for key in ('public_exec_count','workspace_output_reader_count','shell_process_count','dd_process_count','sink_process_count'): check(observed.get(key)==1,'public route count '+key)
            check(isinstance(observed.get('execution_receipt'),str) and bool(observed['execution_receipt']),'public receipt absent')
            for key in ('fuse_read_requests','fuse_read_bytes','fuse_write_requests','fuse_write_bytes'):
                row[phase+'_'+key]=observed.get(key)
                check(key in observed and observed[key] is None,'per-action FUSE field must remain explicitly unavailable')
            for key in ('host_runtime_disk','copy_disk'):
                value=observed.get(key,{})
                for field in ('allocated_bytes','apparent_bytes'): row[phase+'_'+key+'_'+field]=number(value.get(field),phase+' '+key+' '+field)
            try:
                before,after=observed['cgroup_before'],observed['cgroup_after']
                for field in ('usage_usec','user_usec','system_usec'):
                    first=number(before[field],phase+' before '+field);last=number(after[field],phase+' after '+field)
                    delta=None if first is None or last is None else (last-first)*1000
                    if delta is not None and delta<0: local.append('cumulative container CPU counter decreased');delta=None
                    label='cpu' if field=='usage_usec' else field.removesuffix('_usec')+'_cpu'
                    row[phase+'_cgroup_boundary_'+label+'_ns']=delta
                row[phase+'_cgroup_observer_ns']=number(before['observer_ns']+after['observer_ns'],phase+' observer duration')
                for key,label in [('memory_current','container_current_bytes'),('memory_peak','container_lifetime_peak_bytes'),('swap_current','container_swap_current_bytes')]: row[phase+'_'+label]=number(after[key],phase+' '+key)
                for key in ('oom','oom_kill'): check(after[key]==before[key],phase+' OOM event')
                check(after['swap_current']==0 and after['memory_current']<=2*1024**3 and after['memory_peak']<=2*1024**3,'container memory/swap bound')
                for boundary,values in [('before',before),('after',after)]:
                    for key in ('anon','file','kernel','shmem','slab','file_dirty','file_writeback'):
                        row[f'{phase}_container_{boundary}_{key}_bytes']=number(values['memory_stat_bytes'].get(key),phase+' memory.stat '+key)
            except (KeyError,TypeError) as error: local.append('container boundary fields: '+str(error))
            row[phase+'_container_io_bytes']=None
            check(bounded(observed.get('elapsed_ns'),30_000_000_000) and bounded(observed.get('host_rss_bytes'),8*1024**3) and bounded(observed.get('host_lifetime_peak_rss_bytes'),8*1024**3),'phase time/host RSS bounds')
            check(type(observed.get('free_disk_bytes')) is int and observed['free_disk_bytes']>=50*1024**3 and bounded(observed.get('scoped_runtime_allocated_bytes'),16*1024**3) and bounded(observed.get('owned_output_allocated_bytes'),32*1024**3),'phase disk bounds')
            if phase=='read':
                try:
                    depth=wanted['depth']
                    if wanted['arm']=='candidate': check(physical[f'native_depth_{depth}']>=1 and physical['native_record_fetches']>=depth+1 and physical['native_decode_calls']>=depth+1 and physical['native_dependency_edges']>=depth,'dynamic selected-depth corroboration missing')
                    else: check(all(physical[k]==0 for k in ('native_record_fetches','native_decode_calls','native_dependency_edges')),'native activity in legacy control')
                    check(observed['population_coverage']['status']=='PASS','runner coverage gate')
                except (KeyError,TypeError) as error: local.append('coverage fields: '+str(error))
        check(row.get('setup_wrapper_ns') is not None and row['setup_wrapper_ns']<=120_000_000_000,'setup lifecycle bound')
        check(row.get('cleanup_wrapper_ns') is not None and row['cleanup_wrapper_ns']<=120_000_000_000,'cleanup lifecycle bound')
        check(row.get('end_host_ns') is not None and row['end_host_ns']<=120_000_000_000,'End lifecycle bound')
        row['status']='PASS' if not local else 'INVALID';row['null_reason']=None if not local else 'row validation failed; raw observations preserved'
        rows.append(row)
    complete_rows=sum(r['status']=='PASS' for r in rows)
    summary_complete=summary is not None and summary.get('status')=='PASS' and summary.get('completed_rows')==summary.get('expected_rows')==60
    if not summary_complete: errors.append('campaign incomplete/failed; no comparison statistics')
    if summary is not None and (type(summary.get('invocation_wall_ns')) is not int or summary['invocation_wall_ns']>4*3600*1_000_000_000): errors.append('campaign deadline invalid/exceeded')
    if complete_rows!=60: errors.append(f'{complete_rows}/60 rows validate; all cell statistics withheld')
    for row in rows:
        errors.extend(f"row{row['row_index']}: {e}" for e in row['errors'])
    valid=not errors and complete_rows==60 and summary_complete
    status='PASS' if valid else 'INVALID' if summary_complete else 'INCOMPLETE'
    cells=[]
    for depth in range(5):
        for operation in ('range','full'):
            observed={arm:sorted((r for r in rows if r['depth']==depth and r['operation']==operation and r['arm']==arm and r['status']=='PASS'),key=lambda r:r['repetition']) for arm in ('control','candidate')}
            cell=dict(depth=depth,depth_scope='selected target depth; actual full-file composition is in cohort_details, not assumed mixed or homogeneous',operation=operation,status='PASS' if valid else 'UNAVAILABLE_INCOMPLETE_OR_INVALID_CAMPAIGN',observed_n={a:len(v) for a,v in observed.items()},metrics=None,paired_candidate_minus_control_ns=None)
            if valid:
                cell['metrics']={key:{arm:stats([r[key] for r in observed[arm]]) for arm in observed} for key in CELL_METRICS}
                differences=[p['read_elapsed_ns']-c['read_elapsed_ns'] for c,p in zip(observed['control'],observed['candidate'])]
                cell['paired_candidate_minus_control_ns']=dict(by_repetition=differences,**stats(differences))
                cell['difference_of_read_medians_ns']=cell['metrics']['read_elapsed_ns']['candidate']['median']-cell['metrics']['read_elapsed_ns']['control']['median']
            cells.append(cell)
    # Recheck only JSON/text report inputs. Never re-open original Stores here.
    for path,digest in list(input_hashes.items()):
        if sha(pathlib.Path(path))!=digest: raise ValueError('report input changed: '+path)
    columns=list(dict.fromkeys([k for row in rows for k in row]+list(CELL_METRICS)+[phase+'_'+key for phase in ('read','digest') for key in HOST+PHYSICAL]))
    with (out/'observations.csv').open('x',newline='') as stream:
        writer=csv.DictWriter(stream,fieldnames=columns);writer.writeheader()
        for row in rows: writer.writerow({k:json.dumps(row.get(k)) if isinstance(row.get(k),(dict,list)) else 'null' if row.get(k) is None else row[k] for k in columns})
    contract={'units':'integer bytes/ns/count; signed paired differences remain signed; IDs/path_hex/status text',
      'read_elapsed_ns':'public Exec through terminal byte-count output drain; includes shell/pipe/FUSE/Store work',
      'digest':'second public digest Exec, outside read timer; can hit warmed caches',
      'setup_wrapper_vs_host':'wrapper includes container/utilities; host setup includes fresh Store/Client/fork/mount proof',
      'end_vs_cleanup':'host End and wrapper process/container cleanup separate; invocation encloses both',
      'cgroup_boundary':'cumulative before/after CPU delta includes broad protocol and observer activity, not pure read CPU; observer_ns is wall duration, not subtractable CPU',
      'memory':'host current and process-lifetime peak separate; container current and lifetime peak separate; never sum peaks',
      'io':'host I/O phase deltas; container I/O unavailable/null',
      'fuse':'per-action read/write counts unavailable/null; End session totals encompass mount proof, read, digest and End; not phase attribution',
      'disk':'separate copy/temp/combined runtime and owned preparation+campaign scopes; no allocation-comparison eligibility',
      'native':'direct physical work counters, repeated dependencies count; static proof plus fresh setup and aggregate counters corroborate selected population without per-ID trace',
      'null':'UNRUN/invalid missing observations or explicitly unavailable counters; never zero substitution',
      'statistics':'exact n3 min/median/max and by-repetition signed differences only after all60 rows validate; no p95/p99, cold-cache, depth-causality or general-tail claim'}
    result=dict(schema='issue88-depth-read-report-v1',status=status,errors=errors,scheduled_rows=60,validated_rows=complete_rows,run=str(run),input_hashes=input_hashes,
      original_seal_scope='runner-recorded before/after original manifests reconcile; originals not rehashed by this report',
      field_contract=contract,cells=cells,cohort_details=cohort_details,cohort_limitation='Selected small files and their measured closures; not8MiB bulk or maximum163840-byte closure. Different paths/checkpoints across strata prevent causal depth-only comparison.',limits='No retain/revise/reject decision is inferred from a storage or percentage gate; owner judges absolute time and workload costs.')
    with (out/'report.json').open('x') as stream: json.dump(result,stream,indent=2);stream.write('\n')
    with (out/'read-cells.md').open('x') as stream:
        stream.write(f'# Depth-read observations: {status}\n\nSelected target-depth strata; actual full-file composition is recorded below. Three observations per arm/cell. No cold-cache, depth-causal or general-tail claim.\n\n')
        for selected in cohort_details:
            stream.write(f"Depth{selected['depth']}: checkpoint{selected['checkpoint']}, file{selected['file_length_bytes']}B, {selected['candidate_extent_count']} extent(s), closure{selected['closure_raw_bytes']}B; composition `{json.dumps(selected['candidate_file_representation_logical_bytes'],sort_keys=True)}`.\n\n")
        stream.write('|Depth|Read|C n|P n|C min/median/max ns|P min/median/max ns|Paired P−C by repetition ns|\n|---:|---|---:|---:|---|---|---|\n')
        for cell in cells:
            if valid:
                fmt=lambda arm:'/'.join(str(cell['metrics']['read_elapsed_ns'][arm][key]) for key in ('min','median','max'))
                control,candidate=fmt('control'),fmt('candidate');differences=', '.join(map(str,cell['paired_candidate_minus_control_ns']['by_repetition']))
            else: control=candidate=differences='unavailable: incomplete/invalid campaign'
            stream.write(f"|{cell['depth']}|{cell['operation']}|{cell['observed_n']['control']}|{cell['observed_n']['candidate']}|{control}|{candidate}|{differences}|\n")
    manifest_out={'schema':'issue88-depth-read-report-manifest-v1','status':'SEALED','report_status':status,'tool_sha256':sha(pathlib.Path(__file__)),'files':{p.name:sha(p) for p in sorted(out.iterdir()) if p.is_file()}}
    with (out/'manifest.sha256.json').open('x') as stream: json.dump(manifest_out,stream,indent=2);stream.write('\n')
    print(json.dumps(dict(status=status,validated_rows=complete_rows,errors=errors)))
    return 0 if valid else 1


if __name__=='__main__':
    require(len(sys.argv)==3,__doc__)
    sys.exit(main(*[pathlib.Path(p).resolve() for p in sys.argv[1:]]))

#!/usr/bin/env python3
"""Frozen depth-stratified public read diagnostic. No build, copy creation or encoding."""
import argparse, fcntl, hashlib, json, os, queue, shutil, signal, subprocess, sys, threading, time, uuid
from pathlib import Path
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
sys.path.insert(0, str(REPO / 'benchmark/fs-bench-pro/shared'))
import runtime, runner, storage_smoke
REFERENCE_CONTRACT = REPO / 'docs/roadmap/0.1/0.1.4/issue88-native-analysis/published/next-read-contract.md'
CONTRACT = REPO / 'docs/roadmap/0.1/0.1.4/issue88-native-design/depth-read-contract-v1.md'
PROBE_FILES = ('Cargo.toml','Cargo.lock','run.py','README.md','src/main.rs','src/resources.rs')

def save(path, value):
    with Path(path).open('x') as out: json.dump(value, out, indent=2, sort_keys=True); out.write('\n')
def sha(path): return runtime.file_sha256(Path(path))
def load(path): return json.loads(Path(path).read_text())
def tool_seal(root):
    root=Path(root)
    return {name:sha(root/name) for name in PROBE_FILES}

def integer(value, name):
    if type(value) is not int or value<0: raise ValueError(name+' must be nonnegative integer')
    return value

def hex_value(value, size=None):
    if not isinstance(value,str) or not value or len(value)%2 or any(c not in '0123456789abcdef' for c in value) or (size is not None and len(value)!=size):
        raise ValueError('invalid lowercase hexadecimal field')
    return bytes.fromhex(value)

def validate_cohort(path, digest):
    if sha(path)!=digest: raise ValueError('cohort hash mismatch')
    cohort=load(path)
    if cohort['schema']!='issue88-depth-read-cohort-v1' or cohort['status']!='PASS': raise ValueError('cohort schema/status')
    governing=cohort['inputs']['contract']
    if Path(governing['path']).resolve()!=CONTRACT.resolve() or sha(CONTRACT)!=governing['sha256']: raise ValueError('governing cohort contract differs/changed')
    selected=cohort['selections']
    if len(selected)!=5 or [r['depth'] for r in selected]!=list(range(5)): raise ValueError('exact five ordered depth strata required')
    for row in selected:
        depth=integer(row['depth'],'depth'); cp=integer(row['checkpoint'],'checkpoint')
        length=integer(row['file_length_bytes'],'file length'); offset=integer(row['offset_bytes'],'offset')
        if not 1<=cp<=157 or not 4096<=length<=8*1024**2 or row['range_length_bytes']!=4096 or offset+4096>length: raise ValueError('cohort range bounds')
        raw=hex_value(row['path_hex'])
        if b'\0' in raw or any(p in (b'',b'.',b'..') for p in raw.split(b'/')): raise ValueError('noncanonical relative byte path')
        for name in ('expected_range_sha256','expected_full_sha256','source_oracle_sha256','source_manifest_sha256'): hex_value(row[name],64)
        for name in ('source_sha','source_tree'): hex_value(row[name],40)
        native=row['candidate']; span=native['target_span']
        if integer(native['depth'],'target depth')!=depth: raise ValueError('target depth')
        size=integer(native['raw_bytes'],'raw chunk'); closure=integer(native['closure_raw_bytes'],'closure')
        logical=integer(span['logical_offset'],'span logical offset'); source=integer(span['object_offset'],'span object offset'); span_length=integer(span['length'],'span length')
        if size>32768 or closure>1048576 or source+span_length>size or offset<logical or offset+4096>logical+span_length: raise ValueError('range not inside authenticated target span')
        for name in ('pack','group','record'): integer(native[name],name)
        if native['pack']==0: raise ValueError('target pack must be positive')
        hex_value(native['target_id'],64)
        proof=row['proof']
        if sha(proof['path'])!=proof['sha256']: raise ValueError('retained path/dependency proof changed')
        proven=load(proof['path'])
        identity={k:row[k] for k in ('depth','checkpoint','source_sha','source_tree','path_hex','offset_bytes','range_length_bytes','file_length_bytes','expected_range_sha256','expected_full_sha256')}
        identity.update(candidate_target_id=native['target_id'],control_commit_id=row['control']['commit_id'],candidate_commit_id=row['candidate']['commit_id'])
        if proven['schema']!='issue88-depth-selected-proof-v1' or proven['status']!='PASS' or proven['selection_identity']!=identity: raise ValueError('retained proof does not bind exact selected row')
        if proven['disposable_copies']!=cohort['copies']: raise ValueError('proof/cohort copies differ')
        for arm in ('control','candidate'):
            path_proof=proven['arms'][arm]
            if path_proof['status']!='PASS' or any(path_proof[k]!=wanted for k,wanted in [('path_hex',row['path_hex']),('file_length_bytes',length),('offset_bytes',offset),('range_length_bytes',4096),('target_id',native['target_id'])]): raise ValueError('arm retained path proof mismatch')
        observed_span=proven['arms']['candidate']['selected_span']
        if span!={'logical_offset':observed_span['file_offset'],'object_offset':observed_span['source_offset'],'length':observed_span['logical_length']}: raise ValueError('cohort target span differs from product proof')
        chain=proven['candidate_dependency_chain']
        if len(chain)!=depth+1 or not chain or chain[-1]['kind']!='NATIVE_FULL' or any(n['kind']!='NATIVE_PREFIX' for n in chain[:-1]): raise ValueError('native chain shape/depth')
        if any(chain[0][k]!=native[j] for k,j in [('id','target_id'),('pack','pack'),('group','group'),('record','record'),('raw_bytes','raw_bytes')]): raise ValueError('cohort selected locator differs from proof')
        for n in chain:
            hex_value(n['id'],64)
            if integer(n['pack'],'chain pack')==0 or integer(n['group'],'chain group')>=256 or integer(n['record'],'chain record')>=8191 or integer(n['raw_bytes'],'chain raw')>32768: raise ValueError('chain locator/raw bound')
        if any(a['pack']<=b['pack'] for a,b in zip(chain,chain[1:])) or sum(n['raw_bytes'] for n in chain)!=closure: raise ValueError('chain order/closure differs from declared facts')
    return cohort

def schedule(cohort):
    rows=[]
    for selected in cohort['selections']:
        for operation in ('range','full'):
            for rep in (1,2,3):
                for arm in (('control','candidate') if rep!=2 else ('candidate','control')):
                    rows.append({'arm':arm,'depth':selected['depth'],'checkpoint':selected['checkpoint'],
                        'operation':operation,'repetition':rep,'path_hex':selected['path_hex'],
                        'file_length_bytes':selected['file_length_bytes'],
                        'offset_bytes':selected['offset_bytes'] if operation=='range' else 0,
                        'requested_bytes':4096 if operation=='range' else selected['file_length_bytes'],
                        'expected_sha256':selected['expected_range_sha256'] if operation=='range' else selected['expected_full_sha256']})
    if len(rows)!=60: raise ValueError('schedule cardinality')
    return rows

def validate_arm(name, config, cohort):
    source=Path(config['source_run']).resolve(); binary=Path(config['probe_binary']).resolve()
    producer=load(source/'identity.json'); manifest=load(source/'verification-manifest.json')
    if load(source/'verification-summary.json')['status']!='PASS': raise ValueError('source verification incomplete')
    before=storage_smoke.seal(source)
    if any(before.get(p)!=digest for p,digest in manifest.items()): raise ValueError('sealed original evidence changed')
    identity=load(str(binary)+'.identity.json')
    if identity['binary_sha256']!=sha(binary) or identity['tool_files']!=tool_seal(HERE): raise ValueError('new probe binary/tool source mismatch')
    for key in ('LAYERFS_SOURCE_SEAL','LAYERFS_PRODUCT_SEAL'):
        if identity[key]!=producer['source'][key]: raise ValueError('probe reader differs from frozen product/source')
    image=runner.image_info(config['image'],time.monotonic()+30)
    labels=image['Config']['Labels']
    if image['Id']!=producer['image_id'] or labels['dev.layerfs.source-seal']!=identity['LAYERFS_SOURCE_SEAL'] or labels['dev.layerfs.product-seal']!=identity['LAYERFS_PRODUCT_SEAL']: raise ValueError('probe image/source identity')
    perf=load(source/'deepseek-full/performance-result.json')
    if perf['status']!='PASS' or perf['cleanup_status']!='PASS' or not perf['container_removed'] or [r['index'] for r in perf['records']]!=list(range(1,158)): raise ValueError('source full157 performance/cleanup')
    original=source/'deepseek-full/host-runtime/store.sqlite'
    opened=runtime.run(['lsof','-t','--',str(original)],deadline=runtime.Deadline.after(10),check=False)
    if opened.returncode not in (0,1) or opened.stdout.strip() or opened.stderr.strip(): raise ValueError('original Store active/unchecked')
    branch=(original.parent/'branch-id').read_text().strip()
    selections={}
    for row in cohort['selections']:
        record=perf['records'][row['checkpoint']-1]; mapping=row[name]
        if any(row[field]!=record[key] for field,key in [('source_sha','sha'),('source_tree','tree'),('source_oracle_sha256','oracle_sha256'),('source_manifest_sha256','manifest_sha256')]): raise ValueError('cohort source/oracle mismatch')
        if mapping['commit_id']!=record['commit_id'] or mapping['branch_id']!=branch: raise ValueError('cohort retained mapping mismatch')
        selections[row['depth']]=mapping
    ack=[r for r in perf['records'][-1]['receipts'] if r.get('kind')=='storage-smoke-allocation']
    if len(ack)!=1: raise ValueError('source acknowledgement')
    return {'name':name,'source':str(source),'binary':str(binary),'image':image['Id'],
        'producer_identity':producer,'probe_identity':identity,'source_seal_before':before,
        'selections':selections,'primary_final_ack':ack[0]}

def validate_copies(path,digest,arms,cohort):
    if sha(path)!=digest: raise ValueError('prepared copies custody hash mismatch')
    config=load(path)
    if config['schema']!='issue88-depth-read-copies-v1' or set(config['copies'])!={'control','candidate'}: raise ValueError('prepared copies schema')
    protected=[Path(a['source'])/'deepseek-full/host-runtime/store.sqlite' for a in arms.values()]+[Path(e['source_path']).resolve() for e in config['copies'].values()]
    roots={}
    for name,entry in config['copies'].items():
        copy=Path(entry['path']).resolve(); snapshot=Path(entry['source_path']).resolve()
        if copy.name!='store.sqlite' or copy==snapshot or copy.is_relative_to(Path(arms[name]['source'])): raise ValueError('unsafe copy location')
        observed=copy.stat()
        if any((observed.st_dev,observed.st_ino)==(p.stat().st_dev,p.stat().st_ino) for p in protected): raise ValueError('copy hardlinks immutable snapshot/original')
        if Path(cohort['copies'][name]['path']).resolve()!=copy or cohort['copies'][name]['sha256']!=entry['post_proof_sha256']: raise ValueError('campaign copy differs from exact proof copy')
        if sha(copy)!=entry['post_proof_sha256'] or sha(snapshot)!=entry['source_sha256'] or entry['post_proof_sha256']!=entry['source_sha256']: raise ValueError('post-proof copy differs from frozen pre-verification snapshot')
        custody_ref=entry['snapshot_custody']
        if sha(custody_ref['path'])!=custody_ref['sha256']: raise ValueError('snapshot custody changed')
        custody=load(custody_ref['path'])
        source=Path(arms[name]['source']); perf_manifest=source/'performance-manifest.json'
        if custody['copy_sha256']!=entry['source_sha256'] or custody['primary_final_ack']!=arms[name]['primary_final_ack'] or custody['performance_manifest_sha256']!=sha(perf_manifest): raise ValueError('copy snapshot/performance custody')
        if load(perf_manifest)['deepseek-full/performance-result.json']!=sha(source/'deepseek-full/performance-result.json'): raise ValueError('original performance manifest receipt binding')
        if list(copy.parent.glob(copy.name+'-*')): raise ValueError('prepared copy has sidecars')
        opened=runtime.run(['lsof','-t','--',str(copy)],deadline=runtime.Deadline.after(10),check=False)
        if opened.returncode not in (0,1) or opened.stdout.strip() or opened.stderr.strip(): raise ValueError('prepared copy has active owner/unchecked')
        roots[name]=copy.parent
    if roots['control']==roots['candidate'] or roots['control'].parent!=roots['candidate'].parent: raise ValueError('copies must be distinct siblings')
    return roots,config

def deadline(seconds,campaign_end):
    end=min(time.monotonic()+seconds,campaign_end)
    if end<=time.monotonic(): raise TimeoutError('four-hour campaign limit')
    return end

def check_coverage(data,row):
    p=data['physical_storage']; depth=row['depth']
    if row['arm']=='candidate':
        if p[f'native_depth_{depth}']<1 or p['native_record_fetches']<depth+1 or p['native_decode_calls']<depth+1 or p['native_dependency_edges']<depth: raise ValueError('declared native stratum not dynamically exercised')
        return {'status':'PASS','selected_depth':depth,'scope':'aggregate timed-read counters corroborate static authenticated target-span proof; not per-ObjectId tracing'}
    if any(p[k]!=0 for k in ('native_record_fetches','native_decode_calls','native_dependency_edges')): raise ValueError('legacy control unexpectedly reads native records')
    return {'status':'PASS','selected_depth':depth,'scope':'paired control oracle/range; candidate depth is not control physical depth'}

def receive(proc, messages, log, kind, end):
    while True:
        try: line=messages.get(timeout=max(0,end-time.monotonic()))
        except queue.Empty: raise TimeoutError('host waiting for '+kind)
        if line is None: raise RuntimeError('host exited before '+kind)
        log.write(line); log.flush(); value=json.loads(line)
        if value['kind']=='error': raise RuntimeError(value['value']['message'])
        if value['kind']==kind: return value['value']

def observe(sample):
    start=time.monotonic_ns(); value=storage_smoke.cgroup(sample)
    value['observer_ns']=time.monotonic_ns()-start
    return value

def row_run(output, index, row, arm, copy_root, utility_identity, campaign_end):
    folder=output/f'row-{index:02d}'; folder.mkdir()
    pending={**row,'row_index':index,'checkpoint_index':row['checkpoint'],'static_coverage':'sealed cohort selected locator/path/extent proof; see campaign command cohort hash','cache_profile':'fresh-application-fuse-context-existing-os-cache-uncontrolled','admission_eligible':False,'allocation_comparison_eligible':False}
    save(folder/'pending.json',pending)
    result={**pending,'status':'INCOMPLETE','cleanup_status':'NOT_RUN'}
    sample=proc=None; wall=time.monotonic_ns()
    with (folder/'host.jsonl').open('x') as log, (folder/'host.stderr').open('x') as err:
        try:
            setup=time.monotonic_ns(); end=deadline(120,campaign_end)
            sample=runtime.start_sample(arm['image'],'layerfs-read-'+uuid.uuid4().hex[:12],{'family':'issue88-depth-read','row':str(index)},deadline=runtime.Deadline(end))
            result['environment']=sample.observation
            host=sample.observation['host_config']
            if host['Memory']!=2*1024**3 or host['MemorySwap']!=2*1024**3 or host['NanoCpus']!=2_000_000_000 or host['PidsLimit']!=256: raise RuntimeError('frozen container resource profile')
            # These reads only touch installed utilities, never the mounted file.
            utilities=sample.exec(['/bin/bash','-o','pipefail','-c','/usr/bin/sha256sum /bin/bash /usr/bin/dd /usr/bin/wc /usr/bin/sha256sum; /usr/bin/dd if=/dev/zero bs=65536 iflag=skip_bytes,count_bytes,fullblock skip=3 count=7 status=none | /usr/bin/wc -c'],deadline=runtime.Deadline(end))
            lines=utilities.stdout_text().splitlines()
            if len(lines)!=5 or lines[-1].strip()!='7': raise RuntimeError('utility preflight')
            result['utility_identity']=lines[:4]
            if utility_identity and lines[:4]!=utility_identity: raise RuntimeError('utility binary identities differ across rows/arms')
            if not utility_identity: utility_identity.extend(lines[:4])
            selected=arm['selections'][row['depth']]
            temp=folder/'host-tmp'; temp.mkdir()
            env={**os.environ,'TMPDIR':str(temp),'LAYERFS_EXEC_TRANSPORT':'daemon','LAYERFS_FUSE_TRANSPORT':'daemon'}
            row_input={**row,'schema':'issue88-depth-read-row-v1','row_index':index,'branch_id':selected['branch_id'],'commit_id':selected['commit_id']}
            save(folder/'input.json',row_input)
            cmd=[arm['binary'],str(copy_root),sample.id,str(folder/'input.json')]
            result['command']=cmd
            proc=subprocess.Popen(cmd,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1,env=env,start_new_session=True)
            messages=queue.Queue()
            def collect():
                for line in proc.stdout: messages.put(line)
                messages.put(None)
            threading.Thread(target=collect,daemon=True).start()
            result['ready']=receive(proc,messages,log,'ready',end)
            setup_physical=result['ready']['setup_physical_storage']
            if setup_physical['native_record_fetches']!=0 or setup_physical['native_decode_calls']!=0: raise RuntimeError('setup already read native payload; cold-application coverage invalid')
            result['setup_ns']=time.monotonic_ns()-setup
            for phase in ('read','digest'):
                before=observe(sample); begin=time.monotonic_ns()
                proc.stdin.write(phase+'\n'); proc.stdin.flush()
                data=receive(proc,messages,log,phase,deadline(30,campaign_end))
                after=observe(sample)
                data['acknowledgement_window_ns']=time.monotonic_ns()-begin
                data['cgroup_before']=before; data['cgroup_after']=after
                data['host_runtime_disk']=storage_smoke.disk(temp)
                data['copy_disk']=storage_smoke.disk(copy_root)
                data['free_disk_bytes']=shutil.disk_usage(output).free
                result[phase]=data
                if phase=='read': data['population_coverage']=check_coverage(data,row)
                data['scoped_runtime_allocated_bytes']=data['host_runtime_disk']['allocated_bytes']+data['copy_disk']['allocated_bytes']+storage_smoke.disk(copy_root.parent/'container-control')['allocated_bytes']
                if data['scoped_runtime_allocated_bytes']>16*1024**3: raise RuntimeError('scoped runtime bound')
                data['owned_output_allocated_bytes']=storage_smoke.disk(output)['allocated_bytes']+storage_smoke.disk(copy_root.parent.parent)['allocated_bytes']
                if data['owned_output_allocated_bytes']>32*1024**3: raise RuntimeError('owned output bound')
                if not data['correct'] or data['resource_status']!='PASS' or data['elapsed_ns']>30_000_000_000: raise RuntimeError('read correctness/resource/forbidden route')
                if after['swap_current']!=0 or after['oom']!=before['oom'] or after['oom_kill']!=before['oom_kill']: raise RuntimeError('container swap/OOM')
                if after['memory_current']>2*1024**3 or after['memory_peak']>2*1024**3 or data['free_disk_bytes']<50*1024**3: raise RuntimeError('memory/disk ceiling')
            proc.stdin.write('end\n'); proc.stdin.flush()
            result['closed']=receive(proc,messages,log,'closed',deadline(120,campaign_end))
            proc.wait(timeout=5)
            if proc.returncode or not all(result['closed'].get(k) for k in ('cleanup_ok','success','unchanged_head')): raise RuntimeError('host cleanup/head')
            if not result['closed'].get('session_fuse',{}).get('present'): raise RuntimeError('missing end-of-session FUSE read receipt')
            result['status']='PASS'
        except BaseException as error:
            result['error']=str(error)
        finally:
            cleanup=time.monotonic_ns(); cleanup_end=time.monotonic()+120
            try:
                if proc and proc.poll() is None:
                    proc.stdin.close()
                    try: proc.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(proc.pid,signal.SIGTERM)
                        try: proc.wait(timeout=5)
                        except subprocess.TimeoutExpired: os.killpg(proc.pid,signal.SIGKILL); proc.wait()
            except BaseException as error:
                result['host_cleanup_error']=str(error)
            if sample:
                observation_ok=True
                try:
                    staging=sample.exec(['/bin/sh','-c',"if test -d /input; then du -sk /input; else printf '0\\t/input\\n'; fi"],deadline=runtime.Deadline.after(5))
                    result['container_staging_allocated_bytes']=int(staging.stdout_text().split()[0])*1024
                    result['final_scoped_runtime_allocated_bytes']=result['container_staging_allocated_bytes']+storage_smoke.disk(folder/'host-tmp')['allocated_bytes']+storage_smoke.disk(copy_root)['allocated_bytes']+storage_smoke.disk(copy_root.parent/'container-control')['allocated_bytes']
                    if result['final_scoped_runtime_allocated_bytes']>16*1024**3: raise RuntimeError('scoped runtime/staging bound')
                    container_log=runtime.run(['docker','logs',sample.id],deadline=runtime.Deadline.after(5),check=False)
                    if container_log.timed_out or container_log.returncode: raise RuntimeError('container log collection failed')
                    (folder/'container.log').write_bytes(container_log.stdout+container_log.stderr)
                except BaseException as error:
                    result['cleanup_observation_error']=str(error); observation_ok=False
                finally:
                    try:
                        sample.remove(runtime.Deadline(min(time.monotonic()+30,cleanup_end)));  result['container_removed']=True
                        result['cleanup_status']='PASS' if observation_ok and 'host_cleanup_error' not in result and result.get('closed',{}).get('cleanup_ok') else 'FAIL'
                    except BaseException as error: result['cleanup_error']=str(error); result['cleanup_status']='FAIL'
            result['cleanup_ns']=time.monotonic_ns()-cleanup
            if result['cleanup_ns']>120_000_000_000: result['cleanup_status']='FAIL'; result['cleanup_error']='120s cleanup limit'
            result['invocation_wall_ns']=time.monotonic_ns()-wall
            save(folder/'result.json',result)
    if result['status']!='PASS' or result['cleanup_status']!='PASS': raise RuntimeError('row failed; retained '+str(folder))
    return result

def main():
    p=argparse.ArgumentParser()
    for name in ('arms','cohort','copies','output'): p.add_argument('--'+name,type=Path,required=True)
    for name in ('cohort-sha256','copies-sha256'): p.add_argument('--'+name,required=True)
    args=p.parse_args(); config=load(args.arms)
    if set(config)!= {'control','candidate'}: raise ValueError('exact two arms required')
    output=args.output.resolve()
    for entry in config.values():
        source=Path(entry['source_run']).resolve()
        if output==source or output.is_relative_to(source): raise ValueError('output must be outside originals')
    output.mkdir(parents=True,exist_ok=False)
    status='INCOMPLETE'; arms={}; rows=[]; copies={}; attempted=0; stage='preflight'; campaign_error=None
    started=time.monotonic_ns(); campaign_end=time.monotonic()+4*3600
    lock=(Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a')
    try:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        if shutil.disk_usage(output).free<50*1024**3: raise RuntimeError('free disk reserve')
        cohort=validate_cohort(args.cohort,args.cohort_sha256); planned=schedule(cohort)
        arms={name:validate_arm(name,cfg,cohort) for name,cfg in config.items()}
        copies,copy_custody=validate_copies(args.copies,args.copies_sha256,arms,cohort)
        preparation=copies['control'].parent.parent
        if output==preparation or output.is_relative_to(preparation) or preparation.is_relative_to(output): raise ValueError('campaign output and preparation must be disjoint')
        save(output/'command.json',{'argv':sys.argv,'contract':{'path':str(CONTRACT),'sha256':sha(CONTRACT)},'reference_contract':{'path':str(REFERENCE_CONTRACT),'sha256':sha(REFERENCE_CONTRACT)},'arm_config':config,
            'tool_files':tool_seal(HERE),'cohort':{'path':str(args.cohort),'sha256':args.cohort_sha256},
            'prepared_copies':{'path':str(args.copies),'sha256':args.copies_sha256},'schedule':planned})
        save(output/'identity.json',arms); save(output/'copy-custody.json',copy_custody)
        utility_identity=[]; stage='samples'
        for index,row in enumerate(planned,1):
            attempted=index; deadline(1,campaign_end)
            if sha(args.cohort)!=args.cohort_sha256 or sha(args.copies)!=args.copies_sha256: raise ValueError('cohort/copy custody changed during campaign')
            rows.append(row_run(output,index,row,arms[row['arm']],copies[row['arm']],utility_identity,campaign_end))
        if len(rows)!=60: raise RuntimeError('row cardinality')
        status='PASS'
    except BaseException as error:
        status='FAIL'; campaign_error=str(error); raise
    finally:
        final={}
        for name,arm in arms.items():
            after=storage_smoke.seal(Path(arm['source'])); okay=after==arm['source_seal_before']
            final[name]={'source_unchanged':okay,'source_seal_after':after}
            if name in copies: final[name]['disposable_copy_final_sha256']=sha(copies[name]/'store.sqlite')
            if not okay: status='FAIL'
        save(output/'summary.json',{'status':status,'completed_rows':len(rows),'expected_rows':60,
            'admission_eligible':False,'allocation_comparison_eligible':False,
            'invocation_wall_ns':time.monotonic_ns()-started,'sources':final,'attempted_rows':attempted,'failure_stage':stage if status!='PASS' else None,'error':campaign_error,
            'copy_scope':'one pre-verification logical copy per arm reused after proof; forks/session metadata may change copy hash; never allocation equivalent'})
        save(output/'manifest.sha256.json',storage_smoke.seal(output))
        lock.close()
    if status!='PASS': raise RuntimeError('campaign incomplete')
def self_test():
    cohort={'selections':[{'depth':d,'checkpoint':d+1,'path_hex':'66696c65',
        'file_length_bytes':8192,'offset_bytes':0,'expected_range_sha256':'0'*64,
        'expected_full_sha256':'1'*64} for d in range(5)]}
    planned=schedule(cohort)
    assert len(planned)==60 and sum(r['arm']=='candidate' for r in planned)==30
    for start in range(0,60,6):
        assert [r['arm'] for r in planned[start:start+6]]==['control','candidate','candidate','control','control','candidate']
        assert [r['repetition'] for r in planned[start:start+6]]==[1,1,2,2,3,3]
    physical={f'native_depth_{d}':0 for d in range(5)}
    physical.update(native_record_fetches=3,native_decode_calls=3,native_dependency_edges=2,native_depth_2=1)
    assert check_coverage({'physical_storage':physical},{'arm':'candidate','depth':2})['status']=='PASS'
    for bad in [dict(physical,native_depth_2=0),dict(physical,native_record_fetches=2),dict(physical,native_decode_calls=2),dict(physical,native_dependency_edges=1)]:
        try: check_coverage({'physical_storage':bad},{'arm':'candidate','depth':2})
        except ValueError: pass
        else: raise AssertionError('invalid depth evidence accepted')
    missing=dict(physical);del missing['native_depth_2']
    try: check_coverage({'physical_storage':missing},{'arm':'candidate','depth':2})
    except KeyError: pass
    else: raise AssertionError('missing counter accepted as zero')
    control={k:0 for k in physical}
    assert check_coverage({'physical_storage':control},{'arm':'control','depth':2})['status']=='PASS'
    print('PASS:60-row order/cardinality and truthful dynamic depth/missing-counter checks; no Store/container')

if __name__=='__main__':
    if sys.argv[1:]==['--self-test']: self_test()
    else: main()

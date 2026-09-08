#!/usr/bin/env python3
"""Frozen issue88 read-only public diagnostic. No build or smoke construction."""
import argparse, fcntl, hashlib, json, os, queue, shutil, signal, subprocess, sys, threading, time, uuid
from pathlib import Path
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
sys.path.insert(0, str(REPO / 'benchmark/fs-bench-pro/shared'))
import runtime, runner, storage_smoke
CONTRACT = REPO / 'docs/roadmap/0.1/0.1.4/issue88-native-design/public-read-contract-v1.md'
CASES = {'sdk-text-32k':32768, 'sdk-binary-8m':8388608}

def save(path, value):
    with Path(path).open('x') as out: json.dump(value, out, indent=2, sort_keys=True); out.write('\n')
def sha(path): return runtime.file_sha256(Path(path))
def load(path): return json.loads(Path(path).read_text())
def tool_seal(root):
    root=Path(root)
    return {str(p.relative_to(root)):sha(p) for p in sorted(root.rglob('*')) if p.is_file() and 'target' not in p.relative_to(root).parts and '__pycache__' not in p.parts}
def schedule():
    rows=[]
    for case,n in CASES.items():
        for operation in ('range','full'):
            for rep in (1,2,3):
                for arm in (('control','candidate') if rep!=2 else ('candidate','control')):
                    rows.append({'arm':arm,'case':case,'operation':operation,'repetition':rep,'offset_bytes':3*n//4-2048 if operation=='range' else 0,'requested_bytes':4096 if operation=='range' else n})
    assert len(rows)==24
    return rows

def validate_arm(name, config):
    source=Path(config['source_run']).resolve(); binary=Path(config['probe_binary']).resolve()
    producer=load(source/'identity.json'); manifest=load(source/'verification-manifest.json')
    if producer['smoke']!='frequent-edits' or load(source/'verification-summary.json')['status']!='PASS': raise ValueError('source verification population/status')
    before=storage_smoke.seal(source)
    if any(before.get(p)!=digest for p,digest in manifest.items()): raise ValueError('sealed original evidence changed')
    identity=load(str(binary)+'.identity.json')
    # Probe binary identity is an additive sidecar supplied by the serialized build owner.
    # It binds the exact source/product and same-source normal image, never a reporting HEAD.
    if identity['binary_sha256']!=sha(binary) or identity['tool_files']!=tool_seal(HERE): raise ValueError('probe binary/tool source mismatch')
    for key in ('LAYERFS_SOURCE_SEAL','LAYERFS_PRODUCT_SEAL'):
        if identity[key]!=producer['source'][key]: raise ValueError('probe reader differs from producer product/source')
    image=runner.image_info(config['image'],time.monotonic()+30)
    labels=image['Config']['Labels']
    if image['Id']!=producer['image_id'] or labels['dev.layerfs.source-seal']!=identity['LAYERFS_SOURCE_SEAL'] or labels['dev.layerfs.product-seal']!=identity['LAYERFS_PRODUCT_SEAL']: raise ValueError('probe image/source identity')
    selections={}
    for case,n in CASES.items():
        fixture=producer['fixtures'][case]; folder=Path(fixture['input'])
        if storage_smoke.seal(folder)!=fixture['input_seal']: raise ValueError('frozen fixture identity')
        oracle=load(folder/'oracle-3.json'); body=(folder/'state-3').read_bytes()
        if len(body)!=n or oracle[os.fsencode('file').hex()]!=['100644',n,hashlib.sha256(body).hexdigest()]: raise ValueError('checkpoint3 oracle')
        perf=load(source/case/'performance-result.json')
        if perf['status']!='PASS' or perf['cleanup_status']!='PASS': raise ValueError('source producer not quiescent/PASS')
        rows=[r for r in perf['records'] if r['index']==3]
        if len(rows)!=1 or not rows[0]['commit_id']: raise ValueError('checkpoint3 mapping')
        opened=runtime.run(['lsof','-t','--',str(source/case/'host-runtime/store.sqlite')],deadline=runtime.Deadline.after(10),check=False)
        if opened.returncode not in (0,1) or opened.stdout.strip() or opened.stderr.strip(): raise ValueError('original Store has an active owner or lsof check failed')
        branch=(source/case/'host-runtime/branch-id').read_text().strip()
        selections[case]={'commit_id':rows[0]['commit_id'],'branch_id':branch,'expected':{'range':hashlib.sha256(body[3*n//4-2048:3*n//4+2048]).hexdigest(),'full':hashlib.sha256(body).hexdigest()},'fixture_state_sha256':sha(folder/'state-3'),'oracle_sha256':sha(folder/'oracle-3.json')}
    return {'name':name,'source':str(source),'binary':str(binary),'image':image['Id'],'producer_identity':producer,'probe_identity':identity,'source_seal_before':before,'selections':selections}

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

def row_run(output, index, row, arm, copy_root, utility_identity):
    folder=output/f'row-{index:02d}'; folder.mkdir()
    pending={**row,'row_index':index,'checkpoint_index':3,'path':'file','cache_profile':'fresh-application-fuse-context-existing-os-cache-uncontrolled','admission_eligible':False,'allocation_comparison_eligible':False}
    save(folder/'pending.json',pending)
    result={**pending,'status':'INCOMPLETE','cleanup_status':'NOT_RUN'}
    sample=proc=None; wall=time.monotonic_ns()
    with (folder/'host.jsonl').open('x') as log, (folder/'host.stderr').open('x') as err:
        try:
            setup=time.monotonic_ns(); end=time.monotonic()+120
            sample=runtime.start_sample(arm['image'],'layerfs-read-'+uuid.uuid4().hex[:12],{'family':'issue88-public-read','row':str(index)},deadline=runtime.Deadline(end))
            result['environment']=sample.observation
            # These reads only touch installed utilities, never the mounted file.
            utilities=sample.exec(['/bin/bash','-o','pipefail','-c','/usr/bin/sha256sum /bin/bash /usr/bin/dd /usr/bin/wc /usr/bin/sha256sum; /usr/bin/dd if=/dev/zero bs=65536 iflag=skip_bytes,count_bytes,fullblock skip=3 count=7 status=none | /usr/bin/wc -c'],deadline=runtime.Deadline(end))
            lines=utilities.stdout_text().splitlines()
            if len(lines)!=5 or lines[-1].strip()!='7': raise RuntimeError('utility preflight')
            result['utility_identity']=lines[:4]
            if utility_identity and lines[:4]!=utility_identity: raise RuntimeError('utility binary identities differ across rows/arms')
            if not utility_identity: utility_identity.extend(lines[:4])
            selected=arm['selections'][row['case']]
            temp=folder/'host-tmp'; temp.mkdir()
            env={**os.environ,'TMPDIR':str(temp),'LAYERFS_EXEC_TRANSPORT':'daemon','LAYERFS_FUSE_TRANSPORT':'daemon'}
            cmd=[arm['binary'],str(copy_root),sample.id,selected['branch_id'],selected['commit_id'],str(index),row['case'],row['operation'],selected['expected'][row['operation']]]
            result['command']=cmd
            proc=subprocess.Popen(cmd,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1,env=env,start_new_session=True)
            messages=queue.Queue()
            def collect():
                for line in proc.stdout: messages.put(line)
                messages.put(None)
            threading.Thread(target=collect,daemon=True).start()
            result['ready']=receive(proc,messages,log,'ready',end)
            result['setup_ns']=time.monotonic_ns()-setup
            for phase in ('read','digest'):
                before=observe(sample); begin=time.monotonic_ns()
                proc.stdin.write(phase+'\n'); proc.stdin.flush()
                data=receive(proc,messages,log,phase,time.monotonic()+30)
                after=observe(sample)
                data['acknowledgement_window_ns']=time.monotonic_ns()-begin
                data['cgroup_before']=before; data['cgroup_after']=after
                data['host_runtime_disk']=storage_smoke.disk(temp)
                data['copy_disk']=storage_smoke.disk(copy_root)
                data['free_disk_bytes']=shutil.disk_usage(output).free
                result[phase]=data
                if not data['correct'] or data['resource_status']!='PASS': raise RuntimeError('read correctness/resource/forbidden route')
                if after.get('swap_current',1)!=0 or after.get('oom',0)!=before.get('oom',0) or after.get('oom_kill',0)!=before.get('oom_kill',0): raise RuntimeError('container swap/OOM')
                if after.get('memory_current',2**64)>2*1024**3 or data['free_disk_bytes']<50*1024**3: raise RuntimeError('memory/disk ceiling')
            proc.stdin.write('end\n'); proc.stdin.flush()
            result['closed']=receive(proc,messages,log,'closed',time.monotonic()+30)
            proc.wait(timeout=5)
            if proc.returncode or not all(result['closed'].get(k) for k in ('cleanup_ok','success','unchanged_head')): raise RuntimeError('host cleanup/head')
            if not result['closed'].get('session_fuse',{}).get('present'): raise RuntimeError('missing end-of-session FUSE read receipt')
            result['status']='PASS'
        except BaseException as error:
            result['error']=str(error)
        finally:
            cleanup=time.monotonic_ns()
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
                    if result['container_staging_allocated_bytes']+storage_smoke.disk(folder/'host-tmp')['allocated_bytes']>16*1024**3: raise RuntimeError('spool/staging bound')
                    container_log=runtime.run(['docker','logs',sample.id],deadline=runtime.Deadline.after(5),check=False)
                    if container_log.timed_out or container_log.returncode: raise RuntimeError('container log collection failed')
                    (folder/'container.log').write_bytes(container_log.stdout+container_log.stderr)
                except BaseException as error:
                    result['cleanup_observation_error']=str(error); observation_ok=False
                finally:
                    try:
                        sample.remove(runtime.Deadline.after(30)); result['container_removed']=True
                        result['cleanup_status']='PASS' if observation_ok and 'host_cleanup_error' not in result and result.get('closed',{}).get('cleanup_ok') else 'FAIL'
                    except BaseException as error: result['cleanup_error']=str(error); result['cleanup_status']='FAIL'
            result['cleanup_ns']=time.monotonic_ns()-cleanup
            result['invocation_wall_ns']=time.monotonic_ns()-wall
            save(folder/'result.json',result)
    if result['status']!='PASS' or result['cleanup_status']!='PASS': raise RuntimeError('row failed; retained '+str(folder))
    return result

def main():
    p=argparse.ArgumentParser(); p.add_argument('--arms',type=Path,required=True); p.add_argument('--output',type=Path,required=True); args=p.parse_args()
    config=load(args.arms)
    if set(config)!= {'control','candidate'}: raise ValueError('exact two arms required')
    output=args.output.resolve()
    for entry in config.values():
        source=Path(entry['source_run']).resolve()
        if output==source or output.is_relative_to(source): raise ValueError('output must be outside every original source')
    output.mkdir(parents=True,exist_ok=False)
    save(output/'command.json',{'argv':sys.argv,'contract_sha256':sha(CONTRACT),'arm_config':config,'tool_files':tool_seal(HERE),'schedule':schedule()})
    status='INCOMPLETE'; arms={}; rows=[]; started=time.monotonic_ns()
    try:
        with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            if shutil.disk_usage(output).free<50*1024**3: raise RuntimeError('free disk reserve')
            arms={name:validate_arm(name,cfg) for name,cfg in config.items()}
            for case in CASES:
                if arms['control']['selections'][case]['expected']!=arms['candidate']['selections'][case]['expected']: raise ValueError('paired oracle mismatch')
            save(output/'identity.json',arms)
            copies={}
            for name,arm in arms.items():
                for case in CASES:
                    root=output/'copies'/name/case/'host-runtime'; root.mkdir(parents=True)
                    start=time.monotonic_ns()
                    receipt=runtime.closed_store_copy(Path(arm['source'])/case/'host-runtime/store.sqlite',root/'store.sqlite',deadline=runtime.Deadline.after(120))
                    receipt['copy_ns']=time.monotonic_ns()-start
                    receipt['allocation_comparison_eligible']=False
                    save(root.parent/'copy.json',receipt); copies[name,case]=root
            utility_identity=[]
            for index,row in enumerate(schedule(),1): rows.append(row_run(output,index,row,arms[row['arm']],copies[row['arm'],row['case']],utility_identity))
            if len(rows)!=24: raise RuntimeError('row cardinality')
            status='PASS'
    finally:
        final={}
        for name,arm in arms.items():
            after=storage_smoke.seal(Path(arm['source'])); okay=after==arm['source_seal_before']
            final[name]={'source_unchanged':okay,'source_seal_after':after}
            if not okay: status='FAIL'
        save(output/'summary.json',{'status':status,'completed_rows':len(rows),'expected_rows':24,'admission_eligible':False,'allocation_comparison_eligible':False,'invocation_wall_ns':time.monotonic_ns()-started,'sources':final})
        save(output/'manifest.sha256.json',storage_smoke.seal(output))
    if status!='PASS': raise RuntimeError('campaign incomplete')
if __name__=='__main__': main()

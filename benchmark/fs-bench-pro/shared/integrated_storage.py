"""Issue103 adapters for the public compactor and correctness-only live smoke."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import subprocess
import shutil
import time
import uuid

import runner
import runtime

CONTRACT = 'docs/roadmap/0.1/0.1.5/issue103/stride3-integrated-compaction-v1.md'
SCENARIO = 'deepseek-stride3-integrated-compaction-v1'
LIMIT = 4 * 1024**3


def save(path, value):
    with Path(path).open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True); stream.write('\n')


def records(raw):
    return [json.loads(line) for line in raw.decode().splitlines() if line.strip()]


def freeze(path):
    path = Path(path).resolve()
    files = {}
    for file in sorted(path.parent.rglob('*')):
        if file.is_symlink():
            raise ValueError('measured Store contains a symlink')
        if file.is_file():
            m = file.stat()
            files[str(file.relative_to(path.parent))] = {'sha256': runtime.file_sha256(file),
                'allocated_bytes': m.st_blocks*512, 'apparent_bytes': m.st_size,
                'inode': m.st_ino, 'device': m.st_dev}
    if set(files) != {path.name}:
        raise ValueError('measured Store has unexpected sidecars or temporary files')
    return {'path': str(path), 'sha256': files[path.name]['sha256'], 'files': files,
            'allocated_bytes': sum(f['allocated_bytes'] for f in files.values()),
            'apparent_bytes': sum(f['apparent_bytes'] for f in files.values()),
            'frozen_at_ns': time.time_ns()}


def compact(args, case_folder):
    source = case_folder/'host-runtime/store.sqlite'
    destination = case_folder/'compacted-store/store.sqlite'
    destination.parent.mkdir()
    tmp = case_folder/'compaction-tmp'; tmp.mkdir()
    if shutil.disk_usage(case_folder).free < 50*1024**3: raise RuntimeError('compaction free disk reserve')
    before = {'sha256': runtime.file_sha256(source), 'allocated_bytes': source.stat().st_blocks*512,
              'apparent_bytes': source.stat().st_size}
    command = [args.host_binary, 'storage-compact', str(source.resolve()), str(destination.resolve()),
               str(LIMIT), str((case_folder/'compaction-open-files.jsonl').resolve())]
    start = time.monotonic_ns()
    result = {'schema': SCENARIO, 'status':'INCOMPLETE', 'command':command, 'source_before':before,
              'temporary_byte_limit':LIMIT, 'source_preserved':False}
    save(case_folder/'compaction-invocation.json', result)
    try:
        raw = runtime.run(['/usr/bin/time','-l',*command], deadline=runtime.Deadline.after(14400),
            env={**os.environ,'TMPDIR':str(tmp.resolve()),'SQLITE_TMPDIR':str(tmp.resolve())}, check=False, output_limit=8*1024**2)
        (case_folder/'compaction.stdout.jsonl').write_bytes(raw.stdout)
        (case_folder/'compaction.stderr.time').write_bytes(raw.stderr)
        result.update(process_wall_ns=time.monotonic_ns()-start, exit_code=raw.returncode,
                      timed_out=raw.timed_out, records=records(raw.stdout))
        receipts=[r['receipt'] for r in result['records'] if r.get('kind')=='storage-compaction']
        if len(receipts)!=1: raise ValueError('compaction receipt cardinality')
        receipt=receipts[0];result['receipt']=receipt
        if raw.returncode or raw.timed_out or not all(receipt[k] for k in ('published','cleanup_complete','directory_synced')) or receipt['publication_notes']:
            raise ValueError('compaction operation/publication failed')
        observed=[r for r in result['records'] if r.get('kind')=='storage-compaction-open-files']
        if len(observed)!=1 or observed[0]['samples']<1: raise ValueError('compaction resource observations missing')
        if receipt['named_temporary_peak_bytes']>LIMIT or observed[0]['sampled_peak_allocated_bytes']>LIMIT: raise ValueError('compaction temporary allocation budget')
        phases=[r for r in result['records'] if r.get('phase')=='compaction']
        if len(phases)!=1 or not phases[0]['success']: raise ValueError('compaction phase receipt')
        result['measured_store']=freeze(destination)
        if result['measured_store']['allocated_bytes']!=receipt['final_allocated_bytes']:
            raise ValueError('complete Store allocation differs from product receipt')
        result['source_preserved']=runtime.file_sha256(source)==before['sha256']
        if not result['source_preserved']: raise ValueError('compaction source changed')
        if any(tmp.iterdir()): raise ValueError('compaction scratch cleanup incomplete')
        result['status']='PASS'
    except Exception as error:
        result.update(error_type=type(error).__name__,error=str(error),process_wall_ns=time.monotonic_ns()-start)
    save(case_folder/'compaction-result.json',result)
    return result


def check_identity(binary,image):
    current=runner.source_build_args()
    identity=json.loads(Path(str(binary)+'.identity.json').read_text())
    info=runner.image_info(image,time.monotonic()+30);labels=info['Config']['Labels']
    if identity['binary_sha256']!=runtime.file_sha256(binary) or identity['LAYERFS_SOURCE_SEAL']!=current['LAYERFS_SOURCE_SEAL'] or labels['dev.layerfs.source-seal']!=current['LAYERFS_SOURCE_SEAL'] or labels['dev.layerfs.product-seal']!=current['LAYERFS_PRODUCT_SEAL']:
        raise ValueError('stale host/image/source identity')
    if identity.get('integrated_format_probe',{}).get('status')!='PASS': raise ValueError('integrated linked-format probe missing')
    return current,identity,info


def smoke(args):
    output=Path(args.integration_smoke).resolve();output.mkdir(parents=True)
    with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        current,identity,info=check_identity(args.host_binary,args.image)
        save(output/'identity.json',{'schema':'issue103-integration-smoke-v1','source':current,'host_identity':identity,'image_id':info['Id'],'contract_sha256':runtime.file_sha256(runner.REPO/CONTRACT)})
        sample=None;result={'status':'INCOMPLETE','admission_eligible':False}
        host=output/'host-runtime';host.mkdir();tmp=host/'tmp';tmp.mkdir()
        try:
            sample=runtime.start_sample(info['Id'],'layerfs-i103-'+uuid.uuid4().hex[:12],{'family':'issue103-integration-smoke-v1','run':output.name},deadline=runtime.Deadline.after(120))
            command=[args.host_binary,'storage-integration-smoke',str(host),sample.id]
            result['command']=command
            raw=runtime.run(command,deadline=runtime.Deadline.after(300),env={**os.environ,'TMPDIR':str(tmp),'SQLITE_TMPDIR':str(tmp),'LAYERFS_EXEC_TRANSPORT':'daemon','LAYERFS_FUSE_TRANSPORT':'daemon'},check=False,output_limit=8*1024**2)
            (output/'stdout.jsonl').write_bytes(raw.stdout);(output/'stderr.log').write_bytes(raw.stderr)
            rows=records(raw.stdout);result.update(records=rows,exit_code=raw.returncode,timed_out=raw.timed_out)
            if raw.returncode or raw.timed_out or not any(r.get('kind')=='storage-integration-smoke' and r.get('status')=='PASS' for r in rows): raise ValueError('live integration smoke failed')
            result['status']='PASS'
        except Exception as error: result.update(error_type=type(error).__name__,error=str(error))
        finally:
            if sample:
                try:
                    logs=runtime.run(['docker','logs',sample.id],deadline=runtime.Deadline.after(30),check=False)
                    (output/'container.log').write_bytes(logs.stdout+logs.stderr)
                    sample.remove(runtime.Deadline.after(120));result['cleanup_status']='PASS'
                except Exception as error:result.update(status='INCOMPLETE',cleanup_status='FAIL',cleanup_error=str(error))
            save(output/'result.json',result)
        print(json.dumps({'status':result['status'],'output':str(output)}),flush=True)
        return 0 if result['status']=='PASS' and result.get('cleanup_status')=='PASS' else 1


def prepare_access(run, destination, data):
    import hashlib
    run=Path(run).resolve(); destination=Path(destination).resolve(); data=Path(data)
    identity=json.loads((run/'identity.json').read_text())
    if not identity.get('storage_compact') or identity['smoke']!='deepseek-stride3': raise ValueError('integrated stride3 producer required')
    if json.loads((run/'verification-summary.json').read_text())['status']!='PASS': raise ValueError('53-state verification must finish first')
    folder=run/'deepseek-stride3'
    performance=json.loads((folder/'performance-result.json').read_text())
    verification=json.loads((folder/'verification-result.json').read_text())
    if len(performance['records'])!=53 or len(verification['records'])!=53 or verification['status']!='PASS': raise ValueError('exact state verification incomplete')
    for observed,produced in zip(verification['records'],performance['records']):
        if observed['status']!='PASS' or observed['index']!=produced['index'] or observed['identity']!=produced['identity']:
            raise ValueError('state verification identity mismatch')
    if runtime.file_sha256(runner.REPO/CONTRACT)!=identity['integrated_contract_sha256']:
        raise ValueError('prospective contract changed')
    compaction=json.loads((folder/'compaction-result.json').read_text())
    master=folder/'frozen-measured-store/store.sqlite'
    if compaction['status']!='PASS' or runtime.file_sha256(master)!=compaction['measured_store']['sha256']: raise ValueError('measured frozen image mismatch')
    indexed={r['full157_index']:r for r in performance['records']}
    if sorted(indexed)!=list(range(1,158,3)): raise ValueError('retained checkpoint mapping')
    template=json.loads((runner.BENCH/'families/historical_access/fixture.json').read_text())
    cases=[]
    for old in template['cases']:
        requested=old['full157_index']; index={65:67,57:58}.get(requested,requested)
        row=indexed[index]
        oracle_path=Path(row['oracle'])
        if runtime.file_sha256(oracle_path)!=row['oracle_sha256']: raise ValueError('original oracle changed')
        oracle=json.loads(oracle_path.read_text())
        case={**old,'id':old['id'].replace('-v2','-s3-v1'),'template_case':old['id'],
            'requested_original_index':requested,'full157_index':index,'retained_ordinal':row['index'],
            'commit_id':row['commit_id'],'source_commit':row['sha'],'original_oracle_sha256':row['oracle_sha256']}
        if old['operation']=='directory':
            names={bytes.fromhex(path).split(b'/')[0] for path in oracle}
            case['expected']={'names':','.join(name.hex() for name in sorted(names))}
        else:
            mode,size,digest=oracle[old['path'].encode().hex()]
            expected={'mode':mode,'size':size,'mtime':1000000000,'mtime_nsec':0}
            if old['operation']=='read':
                if old['offset']+old['length']>size: raise ValueError('mapped range outside original file')
                rows=[line.split('\t') for line in (Path(row['input'])/'manifest.tsv').read_text().splitlines()]
                entry=next(entry for entry in rows if entry[3]==old['path'].encode().hex())
                raw=runtime.run(['git','--git-dir='+str(data/'source.git'),'cat-file','blob',entry[1]],deadline=runtime.Deadline.after(30),output_limit=2*1024**2).stdout
                if len(raw)!=size or hashlib.sha256(raw).hexdigest()!=digest or hashlib.sha1(b'blob '+str(size).encode()+b'\0'+raw).hexdigest()!=entry[1]: raise ValueError('original access content identity')
                expected['sha256']=hashlib.sha256(raw[old['offset']:old['offset']+old['length']]).hexdigest()
            case['expected']=expected
        cases.append(case)
    fixture={'schema':'historical-access-v2','profile':'historical-access-stride3-integrated-v1',
        'contract_commit':identity['source']['LAYERFS_SOURCE_COMMIT'],'contract_sha256':runtime.file_sha256(runner.REPO/CONTRACT),
        'store_sha256':compaction['measured_store']['sha256'],'store':str(master),
        'branch_id':(folder/'host-runtime/branch-id').read_text().strip(),
        'history_result_sha256':runtime.file_sha256(folder/'performance-result.json'),
        'verification_result_sha256':runtime.file_sha256(folder/'verification-result.json'),
        'compaction_result_sha256':runtime.file_sha256(folder/'compaction-result.json'),'cases':cases}
    save(destination,fixture)
    print(json.dumps({'fixture':str(destination),'store':str(master),'cases':len(cases),'status':'PASS'}))
    return 0


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__)
    choice=parser.add_mutually_exclusive_group(required=True)
    choice.add_argument('--integration-smoke')
    choice.add_argument('--prepare-access')
    parser.add_argument('--output')
    parser.add_argument('--data',default='/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data')
    parser.add_argument('--image')
    parser.add_argument('--host-binary',default=str(runner.REPO/'target/release/fs-benchmark-pro'))
    args=parser.parse_args(argv)
    if args.prepare_access:
        if not args.output: parser.error('--prepare-access requires --output')
        with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            return prepare_access(args.prepare_access,args.output,args.data)
    if not args.image: parser.error('--integration-smoke requires --image')
    return smoke(args)

if __name__=='__main__':raise SystemExit(main())

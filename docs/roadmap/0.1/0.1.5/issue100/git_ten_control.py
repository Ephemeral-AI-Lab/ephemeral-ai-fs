#!/usr/bin/env python3
"""Native Git storage controls for fixed selected LayerFS snapshot profiles."""
import argparse
import collections
import fcntl
import hashlib
import json
import mmap
import os
from pathlib import Path
import shutil
import struct
import subprocess
import time

MANIFEST_SHA='03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271'

def save(path,value):path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
def git(repo,*args,input=None,env=None):
    return subprocess.run(['git','--git-dir='+str(repo),*args],input=input,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True,timeout=600,env=env).stdout

def storage(repo):
    files=[p for p in repo.rglob('*') if p.is_file()]
    loose=[p for p in (repo/'objects').glob('[0-9a-f][0-9a-f]/*') if p.is_file()]
    packs=list((repo/'objects/pack').glob('*.pack'))
    return dict(apparent_bytes=sum(p.stat().st_size for p in files),allocated_bytes=sum(p.stat().st_blocks*512 for p in files),file_count=len(files),loose_object_count=len(loose),loose_apparent_bytes=sum(p.stat().st_size for p in loose),loose_allocated_bytes=sum(p.stat().st_blocks*512 for p in loose),pack_bytes=sum(p.stat().st_size for p in packs),pack_count=len(packs),index_bytes=sum(p.stat().st_size for p in (repo/'objects/pack').glob('*.idx')))

def pack_types(repo):
    counts=collections.Counter()
    for idx in (repo/'objects/pack').glob('*.idx'):
        data=idx.read_bytes();assert data[:8]==b'\xfftOc\0\0\0\2'
        n=struct.unpack_from('>I',data,8+255*4)[0];start=8+256*4+n*24
        with idx.with_suffix('.pack').open('rb') as f,mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ) as pack:
            assert pack[:4]==b'PACK' and struct.unpack_from('>I',pack,8)[0]==n
            for i in range(n):
                off=struct.unpack_from('>I',data,start+4*i)[0]
                assert 12<=off<len(pack)-20 and off<0x80000000
                counts[(pack[off]>>4)&7]+=1
    assert set(counts)<={1,2,3,4,6,7}
    return dict(object_count=sum(counts.values()),delta_objects=counts[6]+counts[7],types=dict(counts))

def verify(repo,mapping,objects,out,phase):
    assert not (repo/'objects/info/alternates').exists()
    commits=git(repo,'rev-list','--reverse','refs/heads/main').decode().splitlines()
    assert commits==[r['git_commit'] for r in mapping] and len(commits)==len(mapping)
    log=git(repo,'log','--reverse','--format=%H %T %P','refs/heads/main').decode().splitlines()
    for i,line in enumerate(log):
        fields=line.split();assert fields[:2]==[mapping[i]['git_commit'],mapping[i]['tree']]
        assert fields[2:]==([] if i==0 else [mapping[i-1]['git_commit']])
    actual=set(git(repo,'cat-file','--batch-all-objects','--batch-check=%(objectname)').decode().splitlines())
    assert actual==objects|set(commits)
    fsck=git(repo,'fsck','--full','--strict','--no-reflogs')
    (out/(phase+'-fsck.txt')).write_bytes(fsck)
    content = verify_contents(repo,mapping) if phase=='pack_delta' else None
    return dict(status='PASS',commits=len(mapping),trees_matched=len(mapping),object_count=len(actual),no_alternates=True,exact_object_membership=True,content=content)

def verify_contents(repo,mapping):
    started=time.monotonic_ns();digests={};entries_checked=bytes_checked=0
    proc=subprocess.Popen(['git','--git-dir='+str(repo),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    try:
        for row in mapping:
            actual={}
            for line in git(repo,'ls-tree','-rlz','--full-tree',row['git_commit']).split(b'\0'):
                if not line:continue
                meta,path=line.split(b'\t',1);mode,kind,oid,length=meta.split();size=int(length)
                assert kind==b'blob' and mode in (b'100644',b'100755',b'120000')
                if oid not in digests:
                    proc.stdin.write(oid+b'\n');proc.stdin.flush()
                    assert proc.stdout.readline().split()==[oid,b'blob',length]
                    body=proc.stdout.read(size);assert len(body)==size and proc.stdout.read(1)==b'\n'
                    digests[oid]=hashlib.sha256(body).hexdigest()
                actual[path.hex()]=[mode.decode(),size,digests[oid]]
                parts=path.split(b'/')
                for n in range(1,len(parts)):actual[b'/'.join(parts[:n]).hex()]=['40755',0,'-']
            assert actual==json.loads(Path(row['oracle']).read_text())
            entries_checked+=len(actual);bytes_checked+=sum(v[1] for v in actual.values())
        proc.stdin.close();assert proc.wait(timeout=30)==0
    finally:
        if proc.poll() is None:proc.kill();proc.wait()
        if not proc.stdin.closed:proc.stdin.close()
        proc.stdout.close();proc.stderr.close()
    return dict(status='PASS',states=len(mapping),verified_entries=entries_checked,verified_bytes=bytes_checked,unique_blobs_hashed=len(digests),elapsed_ns=time.monotonic_ns()-started)

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--data',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);parser.add_argument('--fixture',type=Path,required=True);parser.add_argument('--profile',choices=['deepseek-ten','deepseek-stride3'],default='deepseek-ten');args=parser.parse_args()
    raw=(args.data/'checkpoint-manifest.json').read_bytes();assert hashlib.sha256(raw).hexdigest()==MANIFEST_SHA
    manifest=json.loads(raw);fixture=json.loads(args.fixture.read_text())[args.profile];rows=fixture['states'];indices=[1,18,36,53,70,88,105,122,140,157] if args.profile=='deepseek-ten' else list(range(1,158,3));assert len(rows)==len(indices)
    assert fixture['full157_indices']==indices and [r['index'] for r in rows]==list(range(1,len(rows)+1))
    assert [r['full157_index'] for r in rows]==indices
    assert all(r['tree']==manifest['checkpoints'][r['full157_index']-1]['tree'] and r['sha']==manifest['checkpoints'][r['full157_index']-1]['sha'] for r in rows)
    args.output.mkdir(parents=True,exist_ok=False)
    source=args.data/'source.git';repo=args.output/'snapshots.git'
    result={'schema':'matched-git-ten-snapshots-v1' if args.profile=='deepseek-ten' else 'matched-git-stride3-snapshots-v1','profile':args.profile,'status':'INCOMPLETE','manifest_sha256':MANIFEST_SHA,'source_tip':manifest['tip'],'checkpoints':len(rows),'logical_bytes':sum(r['logical_bytes'] for r in rows),'fixture_sha256':hashlib.sha256(args.fixture.read_bytes()).hexdigest(),'git_version':subprocess.check_output(['git','--version']).decode().strip(),'policies':{'compression':6,'delta_window':10,'delta_depth':50,'threads':2,'gc_auto':False,'bare':True,'working_tree':False},'phases':{}}
    with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        total_started=time.monotonic_ns()
        try:
            assert shutil.disk_usage(args.output).free>=50*1024**3
            template=args.output/'empty-template';template.mkdir()
            subprocess.run(['git','init','--bare','--initial-branch=main','--template='+str(template),str(repo)],check=True,stdout=subprocess.DEVNULL)
            for k,v in [('gc.auto','0'),('gc.autoPackLimit','0'),('maintenance.auto','false'),('core.compression','6'),('pack.compression','6'),('pack.threads','2'),('commit.gpgsign','false'),('core.logAllRefUpdates','false')]:git(repo,'config',k,v)
            result['initial_storage']=storage(repo)
            print('Enumerating only the selected snapshot object closure',flush=True)
            objects=set(git(source,'rev-list','--objects','--no-object-names',*[r['tree'] for r in rows]).decode().splitlines())
            payload=('\n'.join(sorted(objects))+'\n').encode();(args.output/'snapshot-object-ids.txt').write_bytes(payload)
            result['snapshot_object_count']=len(objects)
            # Native pack is transport only: unpack-objects writes actual loose objects.
            started=time.monotonic_ns()
            with (args.output/'snapshot-object-ids.txt').open('rb') as ids,(args.output/'transport.stderr').open('wb') as errors:
                pack=subprocess.Popen(['git','--git-dir='+str(source),'pack-objects','--stdout','--window=0','--depth=0','--no-reuse-delta','--no-reuse-object','--compression=6','--threads=2'],stdin=ids,stdout=subprocess.PIPE,stderr=errors)
                try:
                    unpack=subprocess.run(['git','--git-dir='+str(repo),'unpack-objects','-q'],stdin=pack.stdout,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=600)
                    pack.stdout.close();assert pack.wait(timeout=600)==0
                    if unpack.returncode:raise RuntimeError(unpack.stderr.decode())
                finally:
                    if pack.poll() is None:pack.kill();pack.wait()
            assert not list((repo/'objects/pack').glob('*.pack'))
            mapping=[];previous=None
            for row in rows:
                env={**os.environ,'GIT_AUTHOR_NAME':'LayerFS snapshot control','GIT_AUTHOR_EMAIL':'experiment@example.invalid','GIT_COMMITTER_NAME':'LayerFS snapshot control','GIT_COMMITTER_EMAIL':'experiment@example.invalid','GIT_AUTHOR_DATE':str(1_000_000_000+row['index'])+' +0000','GIT_COMMITTER_DATE':str(1_000_000_000+row['index'])+' +0000'}
                command=['commit-tree',row['tree']]+(['-p',previous] if previous else [])
                commit=git(repo,*command,input=f"Snapshot {row['index']} from {row['sha']}\n".encode(),env=env).decode().strip()
                git(repo,'update-ref','refs/heads/main',commit)
                mapping.append(dict(index=row['index'],source_sha=row['sha'],tree=row['tree'],git_commit=commit,full157_index=row['full157_index'],oracle=row['oracle']));previous=commit
            save(args.output/'mapping.json',mapping)
            result['construction_ns']=time.monotonic_ns()-started
            result['phases']['loose']={'storage':storage(repo),'verification':verify(repo,mapping,objects,args.output,'loose')}
            assert result['phases']['loose']['storage']['loose_object_count']==len(objects)+len(rows)
            save(args.output/'results.json',result);print(json.dumps({'phase':'loose',**result['phases']['loose']['storage']}),flush=True)
            for name,window,depth in [('pack_delta',10,50)]:
                command=['repack','-a','-d','-f','-F','--window='+str(window),'--depth='+str(depth),'--threads=2']
                print('Starting '+name,flush=True);started=time.monotonic_ns();output=git(repo,*command);elapsed=time.monotonic_ns()-started
                (args.output/(name+'.stdout')).write_bytes(output)
                measure=storage(repo);types=pack_types(repo)
                assert measure['loose_object_count']==0 and types['object_count']==len(objects)+len(rows)
                if name=='pack_no_delta':assert types['delta_objects']==0
                assert measure['allocated_bytes']<16*1024**3
                result['phases'][name]=dict(command=command,storage=measure,pack_types=types,repack_ns=elapsed,verification=verify(repo,mapping,objects,args.output,name))
                save(args.output/'results.json',result);print(json.dumps({'phase':name,**measure,**types}),flush=True)
            result['status']='PASS';result['cleanup_status']='PASS';result['total_wall_ns']=time.monotonic_ns()-total_started;save(args.output/'results.json',result)
            print(json.dumps({'status':'PASS','output':str(args.output)}),flush=True)
        except BaseException as error:
            result['error']=repr(error);save(args.output/'results.json',result);raise

if __name__=='__main__':main()

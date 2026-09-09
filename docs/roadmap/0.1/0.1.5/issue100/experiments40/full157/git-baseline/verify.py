import pathlib,json,hashlib,subprocess,collections,struct,time
OUT=pathlib.Path(__file__).resolve().parent
DATA=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data');BASE=DATA/'git-snapshots-157-v1';REPO=BASE/'snapshots.git'
def sha(p):
 with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def inventory():
 return {str(p.relative_to(REPO)):dict(bytes=p.stat().st_size,allocated=p.stat().st_blocks*512,sha256=sha(p)) for p in sorted(REPO.rglob('*')) if p.is_file()}
start=time.monotonic();before=inventory();manifest=DATA/'checkpoint-manifest.json'
assert sha(manifest)=='03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271'
checkpoints=json.loads(manifest.read_text());mapping=json.loads((BASE/'mapping.json').read_text());original=json.loads((BASE/'results.json').read_text())
fixture=json.loads(pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/identity.json').read_text())['fixtures']['deepseek-full']['states']
assert len(mapping)==len(checkpoints['checkpoints'])==len(fixture)==157
assert original['source_tip']==checkpoints['tip']=='b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed'
assert original['manifest_sha256']==sha(manifest)
for m,c,f in zip(mapping,checkpoints['checkpoints'],fixture):assert (m['index'],m['source_sha'],m['tree'])==(c['index'],c['sha'],c['tree'])==(f['index'],f['sha'],f['tree'])
cmd=['git','--git-dir='+str(REPO),'cat-file','--batch']
p=subprocess.run(cmd,input=('\n'.join(m['git_commit'] for m in mapping)+'\n').encode(),capture_output=True,check=True);data=p.stdout;pos=0;parent=None
for m in mapping:
 end=data.index(b'\n',pos);identity,kind,size=data[pos:end].split();size=int(size);pos=end+1;body=data[pos:pos+size];pos+=size;assert data[pos:pos+1]==b'\n';pos+=1
 assert identity.decode()==m['git_commit'] and kind==b'commit'
 head=body.split(b'\n\n')[0].splitlines();assert head[0]==b'tree '+m['tree'].encode()
 parents=[line[7:].decode() for line in head if line.startswith(b'parent ')];assert parents==([] if parent is None else [parent]);parent=m['git_commit']
assert pos==len(data)
pack=next((REPO/'objects/pack').glob('*.pack'));idx=pack.with_suffix('.idx')
with (OUT/'verify-pack.txt').open('wb') as f:subprocess.run(['git','verify-pack','-v',str(idx)],stdout=f,stderr=subprocess.PIPE,check=True)
classes=collections.defaultdict(collections.Counter);ids=set();maxdepth=collections.Counter()
for line in (OUT/'verify-pack.txt').read_text().splitlines():
 w=line.split()
 if len(w) not in (5,7) or len(w[0])!=40 or w[1] not in ('blob','tree','commit','tag'):continue
 ids.add(w[0]);category=w[1]+('_delta' if len(w)==7 else '_full');classes[category].update(objects=1,verify_pack_reported_size_bytes=int(w[2]),packed_bytes=int(w[3]))
 if len(w)==7:maxdepth[w[1]]=max(maxdepth[w[1]],int(w[5]))
expected={line.split()[0] for line in (BASE/'snapshot-object-ids.txt').read_text().splitlines()}|{m['git_commit'] for m in mapping}
assert ids==expected and len(ids)==110081
assert sum(c['packed_bytes'] for c in classes.values())+32==pack.stat().st_size==51989900
index=idx.read_bytes();assert index[:8]==b'\xfftOc\0\0\0\2';n=struct.unpack_from('>I',index,8+255*4)[0];assert n==110081 and len(index)==1072+28*n==3083340
assert inventory()==before
apparent=sum(v['bytes'] for v in before.values());allocated=sum(v['allocated'] for v in before.values());saved=original['phases']['pack_delta']['storage']
result=dict(status='PASS',scope='Existing baseline live read-only verification, no repack/rebuild. Trees/membership and pack integrity checked; no new timing comparison.',manifest_sha256=sha(manifest),mapping_sha256=sha(BASE/'mapping.json'),checkpoints=157,source_tip=original['source_tip'],mapping_trees_and_parent_chain_verified=True,exact_snapshot_membership=True,objects=len(ids),git_verify_pack='PASS',repository_files_before_after=before,live_storage=dict(apparent_bytes=apparent,allocated_bytes=allocated,file_count=len(before),pack_bytes=pack.stat().st_size,index_bytes=len(index)),recorded_storage=saved,layout_matches_recorded=all([apparent==saved['apparent_bytes'],allocated==saved['allocated_bytes'],len(before)==saved['file_count']]),classes=dict(classes),max_delta_depth=dict(maxdepth),attribution=dict(content_pack_bytes=sum(v['packed_bytes'] for k,v in classes.items() if k.startswith('blob')),tree_commit_pack_bytes=sum(v['packed_bytes'] for k,v in classes.items() if not k.startswith('blob')),pack_header_trailer_bytes=32,index_bytes=len(index),other_apparent_bytes=apparent-pack.stat().st_size-len(index),allocation_difference_bytes=allocated-apparent),elapsed_seconds=time.monotonic()-start,script_sha256=sha(pathlib.Path(__file__)))
(OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='repository_files_before_after'},indent=2))

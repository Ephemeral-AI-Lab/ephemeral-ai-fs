from shared import *
import subprocess
checkpoints=json.loads((BASE/'checkpoint-manifest.json').read_text());assert checkpoints['selected_count']==157 and checkpoints['tip']=='b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed'
identity=json.loads((RUN/'identity.json').read_text());states=identity['fixtures']['deepseek-stride3']['states'];assert len(states)==53
assert [s['full157_index'] for s in states]==list(range(1,158,3))
fixture=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-data/deepseek-stride3-218c1b81ed39c763ca22/fixture.json');assert sha(fixture)=='3b2c12b682e4892837fc810969e4e90eaa8d756c0d6f738df50746ad478fb73e'
assert states==json.loads(fixture.read_text())['deepseek-stride3']['states']
spans={};first={};seals={};ranges={};missing=[]
for s in states:
 step=s['index'];manifest=Path(s['input'])/'manifest.tsv';assert sha(manifest)==s['input_seal']['manifest.tsv']
 for line in manifest.read_text().splitlines():
  mode,oid,size,path=line.split('\t')
  if bytes.fromhex(path)!=b'pnpm-lock.yaml':continue
  file=Path(s['input'])/'blobs'/oid
  oracle=Path(s['oracle']);assert sha(oracle)==s['oracle_sha256'];expected=json.loads(oracle.read_text())['pnpm-lock.yaml'.encode().hex()][2]
  data=file.read_bytes() if file.exists() else subprocess.check_output(['git','--git-dir='+str(BASE/'source.git'),'cat-file','blob',oid])
  assert hashlib.sha256(data).hexdigest()==expected and hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()==oid
  seals[str(step)]=dict(path=str(file) if file.exists() else None,sha256=expected,length=len(data),git_oid=oid,oracle_sha256=s['oracle_sha256'])
  if len(data)<131072:continue
  cursor=0;spans[step]=[]
  for size in lengths(data):
   raw=data[cursor:cursor+size];id=object_id(raw)
   if id not in records:missing.append(dict(step=step,id=id,start=cursor,size=size))
   span=dict(id=id,start=cursor,end=cursor+size);spans[step].append(span);first.setdefault(id,(step,span));cursor+=size
  assert cursor==len(data)
 p=json.loads((RUN/f'deepseek-stride3/performance-step-{step}.json').read_text());ranges[step]=[r['physical_storage']['diag_selected_pack_last_id'] for r in p['receipts'] if 'physical_storage' in r];assert len(ranges[step])==2
mismatches=[]
for id,(step,span) in first.items():
 if id not in records:continue
 r=records[id];lo,hi=ranges[step]
 if not(lo<r['pack']<=hi):mismatches.append(dict(id=id,step=step,kind='first_admission',pack=r['pack'],range=[lo,hi]))
 hints=[]
 for p in spans.get(step-1,[]):
  if p['start']<span['end'] and p['end']>span['start'] and p['id'] not in hints:hints.append(p['id'])
  if len(hints)==4:break
 if r['kind'] and (not hints or r['base']!=hints[0]):mismatches.append(dict(id=id,step=step,kind='base_hint',actual_base=r['base'],hints=hints))
result=dict(store_sha256=BEFORE,protocol_sha256=sha(OUT/'protocol.md'),checkpoint_manifest_sha256=sha(BASE/'checkpoint-manifest.json'),native_records=len(records),large_states=len(spans),lockfile_objects=len(first),missing=missing,mismatches=mismatches,seals=seals,spans=spans,first=first,ranges=ranges)
(OUT/'provenance.json').write_text(json.dumps(result,indent=2));print(json.dumps({k:v for k,v in result.items() if k not in ['seals','spans','first','ranges']},indent=2))
assert not missing and not mismatches

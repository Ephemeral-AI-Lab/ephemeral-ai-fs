"""Cold actual-reader representatives and three original oracles, unchanged reader."""
import hashlib,importlib.util,json,pathlib,struct,sys,time
HERE=pathlib.Path(__file__).resolve().parent
READER=HERE.parent.parent/'content/full157/reader.py'
DRIVER=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/verification/verify.py')
BASE=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural')
def sha(p):return hashlib.file_digest(pathlib.Path(p).open('rb'),'sha256').hexdigest()
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
sys.path.insert(0,str(BASE/'combined'));__import__('store_api');v=load('index_original_oracle',DRIVER);m=load('index_actual_reader',READER);v.physical=m
result=json.loads((HERE/'result.json').read_text());path=HERE/'candidate.sqlite';before=sha(path);assert before==result['candidate']['sha256'];assert not(HERE/'reader-check.json').exists();readerhash=sha(READER)
r=m.StoreReader(path);loc={(p,g,k):(i,n) for i,(p,g,k,n) in r.loc.items()};representatives={}
for p,b in r.db.execute('select pack_id,data from object_packs'):
 if b[:8]!=m.MAGIC:continue
 count=int.from_bytes(b[12:16],'little')
 for g in range(count):
  start=int.from_bytes(b[16+4*g:20+4*g],'little');kind=b[start];i,n=loc[p,g,0]
  if kind not in representatives or n>representatives[kind][1]:representatives[kind]=(i,n)
root=r.db.execute('select root_id from commits limit 1').fetchone()[0];r.close();samples=[]
for kind,(i,n) in sorted(representatives.items()):
 probe=m.StoreReader(path);data=probe.read_canonical(i);assert len(data)==n;probe.close();samples.append(dict(kind=kind,id=i.hex(),canonical_bytes=n,sha256=hashlib.sha256(data).hexdigest(),status='PASS'))
probe=m.StoreReader(path);canonical=probe.read_canonical(root);assert canonical[13:21]==b'LFS6FSR\0';table=canonical[97:129];tb=probe.read_canonical(table);assert tb[13:21]==b'LFS6INT\0';pv=probe.pool_value(1);assert len(pv)==94;probe.close()
identities=json.loads((BASE/'combined/identity-map.json').read_text())
checkpoints=json.loads((v.FIXTURE/'checkpoint-manifest.json').read_text())['checkpoints'];receipts=json.loads((v.SOURCE/'verification-result.json').read_text())['records'];assert len(checkpoints)==len(receipts)==157
reader=m.StoreReader(path);files=v.Files(reader);states=[];started=time.monotonic()
try:
 for index in (1,79,157):
  checkpoint=checkpoints[index-1];receipt=receipts[index-1];assert checkpoint['index']==receipt['index']==index
  commit=bytes.fromhex(identities['commits'][receipt['identity']]);root=reader.db.execute('select root_id from commits where commit_id=?',(commit,)).fetchone()[0]
  oracle=v.FIXTURE/'oracles'/(checkpoint['sha']+'.json');expected=json.loads(oracle.read_text());observed=v.observe(reader,files,root);assert observed==expected
  logical=sum(x[1] for x in observed.values());assert logical==checkpoint['logical_bytes'];states.append(dict(index=index,paths=len(observed),logical_bytes=logical,oracle_sha256=sha(oracle),status='PASS'));print('PASS original state',index,flush=True)
finally:reader.close()
assert before==sha(path) and readerhash==sha(READER)
out=dict(status='PASS',fresh_original_states=states,representative_content_kinds=samples,namespace_inode_table_and_pool_group='PASS',candidate_sha256=before,reader_sha256=readerhash,reader_path=str(READER),driver_sha256=sha(DRIVER),checkpoint_manifest_sha256=sha(v.FIXTURE/'checkpoint-manifest.json'),identity_map_sha256=sha(BASE/'combined/identity-map.json'),script_sha256=sha(pathlib.Path(__file__)),elapsed_seconds=time.monotonic()-started,scope='Fresh selected1/79/157originalstates andcoldrepresentativecontent/meta reads; not a fresh all157read. ExactlogicalDBequality carries source proof when parentfull157sourceoraclepasses.')
(HERE/'reader-check.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k not in ('fresh_original_states','representative_content_kinds')},indent=2))

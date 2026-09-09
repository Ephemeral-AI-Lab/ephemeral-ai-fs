"""Fixed12-target pnpm CDC overlap experiment; immutable current Store, no rebuild."""
import collections,ctypes,hashlib,json,sqlite3,struct,time
from pathlib import Path
OUT=Path(__file__).resolve().parent
ROOT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence');BASE=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-ten-evidence');RUN=ROOT/'retained-candidate-1';STORE=RUN/'deepseek-ten/host-runtime/store.sqlite'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
BEFORE=sha(STORE);assert BEFORE=='713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4'
z=ctypes.CDLL('/opt/homebrew/lib/libzstd.dylib')
def bind(lib,name,args,res):
 f=getattr(lib,name);f.argtypes=args;f.restype=res;return f
P=ctypes.c_void_p;S=ctypes.c_size_t;I=ctypes.c_int
version=bind(z,'ZSTD_versionNumber',[],ctypes.c_uint)();assert version==10507
for name,args,res in [('ZSTD_isError',[S],ctypes.c_uint),('ZSTD_initStaticCCtx',[P,S],P),('ZSTD_CCtx_reset',[P,I],S),('ZSTD_CCtx_setParameter',[P,I,I],S),('ZSTD_CCtx_refPrefix',[P,P,S],S),('ZSTD_compressBound',[S],S),('ZSTD_compress2',[P,P,S,P,S],S),('ZSTD_createDCtx',[],P),('ZSTD_freeDCtx',[P],S),('ZSTD_DCtx_refPrefix',[P,P,S],S),('ZSTD_decompressDCtx',[P,P,S,P,S],S)]:bind(z,name,args,res)
def checked(n):assert not z.ZSTD_isError(n),n;return n
PARAMS=[(100,3),(101,20),(200,1),(201,1),(202,0),(400,0)]
def compress(raw,prefix=b''):
 workspace=(ctypes.c_uint64*(1048576//8))();ctx=z.ZSTD_initStaticCCtx(workspace,1048576);assert ctx
 assert len(raw)<=32768 and len(prefix)<=32768
 checked(z.ZSTD_CCtx_reset(ctx,3))
 for k,v in PARAMS:checked(z.ZSTD_CCtx_setParameter(ctx,k,v))
 checked(z.ZSTD_CCtx_refPrefix(ctx,prefix,len(prefix)))
 bound=checked(z.ZSTD_compressBound(len(raw)));assert bound<=33024;dest=ctypes.create_string_buffer(bound)
 t=time.perf_counter_ns();n=checked(z.ZSTD_compress2(ctx,dest,bound,raw,len(raw)));elapsed=time.perf_counter_ns()-t
 checked(z.ZSTD_CCtx_reset(ctx,3));return dest.raw[:n],elapsed

def decompress(frame,raw,prefix=b''):
 d=z.ZSTD_createDCtx();dest=ctypes.create_string_buffer(raw)
 try:checked(z.ZSTD_DCtx_refPrefix(d,prefix,len(prefix)));assert checked(z.ZSTD_decompressDCtx(d,dest,raw,frame,len(frame)))==raw
 finally:z.ZSTD_freeDCtx(d)
 return dest.raw

db=sqlite3.connect(STORE.as_uri()+'?mode=ro&immutable=1',uri=True)
loc={(p,g,r):(oid.hex(),n) for oid,n,p,g,r in db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')};records={}
for p,b in db.execute("select pack_id,data from object_packs where substr(data,9,4)=x'02000000'"):
 for g in range(struct.unpack_from('<I',b,12)[0]):
  pos,size,decoded,codec=struct.unpack_from('<IIII',b,16+16*g);assert codec==0;group=b[pos:pos+size];n=struct.unpack_from('<I',group)[0];start=4+4*n
  for ordinal,end in enumerate(struct.unpack_from('<'+'I'*n,group,4)):
   end+=4+4*n;r=group[start:end];start=end;k,raw=struct.unpack_from('<BI',r);oid,canonical=loc[p,g,ordinal];assert canonical==raw+21
   records[oid]=dict(kind=k,raw=raw,base=r[5:37].hex() if k else None,frame=r[37 if k else 5:],size=len(r),pack=p,group=g,ordinal=ordinal,group_count=n)
db.close()
import sys
sys.path.insert(0,str(OUT.parent/'tools'))
from hashing import blake3
from cdc import lengths

def canonical(raw):return b'LFSO\x01'+struct.pack('>II',len(raw)+12,len(raw)+8)+b'LFS4CHK\0'+raw
def object_id(raw):return blake3(b'layerfs/object/v2\0'+canonical(raw)).hex()
def decode(oid):
 chain=[];node=oid
 while True:
  assert node not in [i for i,_ in chain];r=records[node];chain.append((node,r))
  if not r['base']:break
  assert records[r['base']]['pack']<r['pack'];node=r['base']
 assert len(chain)<=5 and sum(r['raw'] for _,r in chain)<=1048576
 data=b'';started=time.perf_counter_ns()
 for id,r in reversed(chain):
  data=decompress(r['frame'],r['raw'],data);assert object_id(data)==id
 info=dict(depth=len(chain)-1,raw_closure=sum(r['raw'] for _,r in chain),lookups=len(chain),encoded_work=sum(32+4+4*r['group_count']+r['size'] for _,r in chain),decoded_work=sum(32+4+4*r['group_count']+r['size']+r['raw']+21 for _,r in chain),groups=[[r['pack'],r['group']] for _,r in chain],decode_auth_ns=time.perf_counter_ns()-started)
 assert info['encoded_work']<=393216 and info['decoded_work']<=524288
 return data,info

fixture=json.loads((BASE/'fixture.json').read_text())['deepseek-ten']['states'];spans={};first={};seals={};ranges={}
for s in fixture:
 step=s['index'];manifest=Path(s['input'])/'manifest.tsv';assert sha(manifest)==s['input_seal']['manifest.tsv']
 for line in manifest.read_text().splitlines():
  mode,oid,size,path=line.split('\t')
  if bytes.fromhex(path)!=b'pnpm-lock.yaml':continue
  rawpath=Path(s['input'])/'blobs'/oid;data=rawpath.read_bytes();assert sha(rawpath)==s['input_seal']['blobs/'+oid];seals[str(step)]=dict(path=str(rawpath),sha256=sha(rawpath),length=len(data),git_oid=oid)
  if len(data)<131072:continue
  cursor=0;spans[step]=[]
  for size in lengths(data):
   raw=data[cursor:cursor+size];id=object_id(raw);assert id in records and records[id]['raw']==size
   span=dict(id=id,start=cursor,end=cursor+size);spans[step].append(span);first.setdefault(id,(step,span));cursor+=size
  assert cursor==len(data)
 performance=json.loads((RUN/f'deepseek-ten/performance-step-{step}.json').read_text());ids=[r['physical_storage']['diag_selected_pack_last_id'] for r in performance['receipts'] if 'physical_storage' in r];assert len(ids)==2;ranges[step]=ids
assert len(first)==249
samples=[]
for id,(step,span) in first.items():
 physical_step=next(s for s,(lo,hi) in ranges.items() if lo<records[id]['pack']<=hi);assert physical_step==step
 if step not in range(3,11):continue
 hints=[]
 for prior in spans[step-1]:
  if prior['start']<span['end'] and prior['end']>span['start'] and prior['id'] not in hints:
   hints.append(prior['id'])
   if len(hints)==4:break
 r=records[id]
 if r['kind']:assert hints and r['base']==hints[0],(id,r['base'],hints)
 samples.append(dict(id=id,step=step,span=span,hints=hints,current_kind=r['kind'],current_record_bytes=r['size'],current_base=r['base']))
samples=sorted(samples,key=lambda r:(-r['current_record_bytes'],r['id']))[:12]
(OUT/'sample.json').write_text(json.dumps(dict(protocol_sha256=sha(OUT/'protocol.md'),fixture_seals=seals,samples=samples),indent=2))
# Baseline reproduction is a prerequisite for any alternative.
for s in samples:
 r=records[s['id']];raw,_=decode(s['id']);prefix=decode(r['base'])[0] if r['base'] else b'';encoded,_=compress(raw,prefix);assert encoded==r['frame'],('codec mismatch',s['id'])
results=[]
for s in samples:
 r=records[s['id']];raw,target_work=decode(s['id']);full,full_ns=compress(raw);assert decompress(full,len(raw))==raw
 alternatives=[];budget=dict(lookups=0,encoded_work=0,decoded_work=0);best=r['size'];winner=r['base']
 for index,id in enumerate(s['hints']):
  prefix,info=decode(id);assert records[id]['pack']<r['pack'];eligible=info['depth']<4 and info['raw_closure']+len(raw)<=1048576
  reason=None
  if budget['lookups']+info['lookups']>8 or budget['encoded_work']+info['encoded_work']>524288 or budget['decoded_work']+info['decoded_work']>524288:reason='cumulative_target_read_budget'
  else:
   for k in budget:budget[k]+=info[k]
   if not eligible:reason='depth_or_raw_closure'
  if reason:
   alternatives.append(dict(id=id,hint_index=index,eligible=False,reason=reason,work=info));continue
  encoded,ns=compress(raw,prefix);assert decompress(encoded,len(raw),prefix)==raw;cost=37+len(encoded)
  alternatives.append(dict(id=id,hint_index=index,eligible=True,record_bytes=cost,frame_bytes=len(encoded),encode_ns=ns,work=info))
  if cost<best:best=cost;winner=id
 results.append(dict(**s,full_record_bytes=5+len(full),full_encode_ns=full_ns,target_decode_work=target_work,candidates=alternatives,best_record_bytes=best,saved_record_bytes=r['size']-best,winner=winner,cumulative_work=budget))
summary=dict(targets=len(results),current_record_bytes=sum(r['current_record_bytes'] for r in results),best_record_bytes=sum(r['best_record_bytes'] for r in results),saved_record_bytes=sum(r['saved_record_bytes'] for r in results),improved_targets=sum(r['saved_record_bytes']>0 for r in results),alternative_candidates=sum(c['hint_index']>0 for r in results for c in r['candidates']),eligible_alternative_candidates=sum(c['hint_index']>0 and c['eligible'] for r in results for c in r['candidates']))
assert sha(STORE)==BEFORE
result=dict(scope='Fixed12-record original-byte diagnostic. Existing physical base graph counted once; no Store rewrite or family/wholeStore savings claim. Exact original admission batch remaining trial/8MiB quotas are unavailable: target-level lookups/bytes enforced cumulatively, batch deployment effect requires product check.',store_sha256=BEFORE,library_sha256=sha(Path('/opt/homebrew/lib/libzstd.dylib')),codec_version=version,params=PARAMS,static_context_bytes=1048576,script_sha256=sha(Path(__file__)),sample_sha256=sha(OUT/'sample.json'),summary=summary,results=results)
(OUT/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(summary,indent=2))

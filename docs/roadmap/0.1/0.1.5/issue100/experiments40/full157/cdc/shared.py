"""Fixed12-target pnpm CDC overlap experiment; immutable current Store, no rebuild."""
import collections,ctypes,hashlib,json,sqlite3,struct,time
from pathlib import Path
OUT=Path(__file__).resolve().parent
ROOT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence');BASE=Path('/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data');RUN=ROOT/'retained-full157-1';STORE=RUN/'deepseek-full/host-runtime/store.sqlite'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
BEFORE=sha(STORE);assert BEFORE=='f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7'
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
sys.path.insert(0,'/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/tools')
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


mask=(1<<64)-1
def signature(raw):
 result=[];rolling=0;high=pow(257,15,1<<64)
 for i,b in enumerate(raw):
  if i>=16:rolling=(rolling-raw[i-16]*high)&mask
  rolling=(rolling*257+b)&mask
  if i<15:continue
  h=rolling;h=((h^(h>>30))*0xbf58476d1ce4e5b9)&mask;h=((h^(h>>27))*0x94d049bb133111eb)&mask;h=h^(h>>31)
  if h!=mask and (len(result)<8 or h<result[-1]) and h not in result:result=sorted(result+[h])[:8]
 return result

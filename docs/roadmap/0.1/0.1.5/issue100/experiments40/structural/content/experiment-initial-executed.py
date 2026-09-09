"""Complete fixed graph trials; diagnostic record exports, never writes input stores."""
import collections,ctypes as C,hashlib,json,sqlite3,struct,subprocess,sys,time
from pathlib import Path
sys.dont_write_bytecode=True
OUT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural/content');HERE=Path(__file__).resolve().parent
ATTR=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/content/attribution.json')
REPO=Path('/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data/git-snapshots-157-v1/snapshots.git')
INV=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/git-baseline/verify-pack.txt')
sys.path.insert(0,'/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/tools')
from hashing import blake3

def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
z=C.CDLL('/opt/homebrew/lib/libzstd.dylib');P=C.c_void_p;S=C.c_size_t;I=C.c_int
for name,args,res in [('ZSTD_versionNumber',[],C.c_uint),('ZSTD_isError',[S],C.c_uint),('ZSTD_createCCtx',[],P),('ZSTD_CCtx_reset',[P,I],S),('ZSTD_CCtx_setParameter',[P,I,I],S),('ZSTD_CCtx_refPrefix',[P,P,S],S),('ZSTD_compressBound',[S],S),('ZSTD_compress2',[P,P,S,P,S],S),('ZSTD_createDCtx',[],P),('ZSTD_DCtx_reset',[P,I],S),('ZSTD_DCtx_refPrefix',[P,P,S],S),('ZSTD_decompressDCtx',[P,P,S,P,S],S)]:
 f=getattr(z,name);f.argtypes=args;f.restype=res
assert z.ZSTD_versionNumber()==10507
cc=z.ZSTD_createCCtx();dc=z.ZSTD_createDCtx()
def ck(n):assert not z.ZSTD_isError(n),n;return n
def encode(raw,base=b''):
 ck(z.ZSTD_CCtx_reset(cc,3))
 for k,v in [(100,3),(101,18 if len(raw)<131072 else 20),(200,1),(201,1),(202,0),(400,0)]:ck(z.ZSTD_CCtx_setParameter(cc,k,v))
 ck(z.ZSTD_CCtx_refPrefix(cc,base,len(base)));size=z.ZSTD_compressBound(len(raw));buf=C.create_string_buffer(size);n=ck(z.ZSTD_compress2(cc,buf,size,raw,len(raw)));return buf.raw[:n]
def decode(frame,size,base=b''):
 ck(z.ZSTD_DCtx_reset(dc,3));ck(z.ZSTD_DCtx_refPrefix(dc,base,len(base)));buf=C.create_string_buffer(size);assert ck(z.ZSTD_decompressDCtx(dc,buf,size,frame,len(frame)))==size;return buf.raw
assert decode(encode(b'hello world',b'hello'),11,b'hello')==b'hello world'
class Cache:
 def __init__(self):self.rows=collections.OrderedDict();self.bytes=0
 def get(self,k):
  if k in self.rows:self.rows.move_to_end(k);return self.rows[k]
 def put(self,k,v):
  if k in self.rows:self.bytes-=len(self.rows.pop(k))
  if len(v)>67108864:return
  self.rows[k]=v;self.bytes+=len(v)
  while self.bytes>67108864:self.bytes-=len(self.rows.popitem(last=False)[1])
proc=subprocess.Popen(['git','--git-dir='+str(REPO),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE);cache=Cache();reads=0

def raw(oid):
 global reads
 b=cache.get(oid)
 if b is not None:return b
 proc.stdin.write(oid.encode()+b'\n');proc.stdin.flush();h=proc.stdout.readline().split();assert h[:2]==[oid.encode(),b'blob'];size=int(h[2]);b=proc.stdout.read(size);assert len(b)==size and proc.stdout.read(1)==b'\n';assert hashlib.sha1(b'blob '+str(size).encode()+b'\0'+b).hexdigest()==oid;reads+=1;cache.put(oid,b);return b
started=time.time();at=json.loads(ATTR.read_text());small={r['git_oid']:r for r in at['rows']};id_oid={r['id']:r['git_oid'] for r in at['rows']};git={}
for line in INV.read_text().splitlines():
 f=line.split()
 if len(f) in (5,7) and len(f[0])==40 and f[1]=='blob':git[f[0]]={'base':f[6] if len(f)==7 else None,'entry':int(f[3])}
assert len(small)==75398 and len(git)==75929
states=json.loads(Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/identity.json').read_text())['fixtures']['deepseek-full']['states']
paths=collections.defaultdict(list);first={};last={};sizes={};seals={}
for s in states:
 p=Path(s['input'])/'manifest.tsv';assert sha(p)==s['input_seal']['manifest.tsv'];seals[str(s['index'])]=sha(p)
 for line in p.read_text().splitlines():
  mode,oid,size,path=line.split('\t');size=int(size);first.setdefault(oid,s['index']);last[oid]=s['index'];sizes[oid]=size
  if not paths[path] or paths[path][-1]!=oid:paths[path].append(oid)
assert set(first)==set(git)
# Fixed candidate lists from complete path histories; choose nearest selected eligible entry at admission.
prev=collections.defaultdict(list);nxt=collections.defaultdict(list)
for path,hist in sorted(paths.items()):
 for pos,oid in enumerate(hist):
  for dst,seq in [(prev,reversed(hist[:pos])),(nxt,iter(hist[pos+1:]))]:
   for other in seq:
    if other!=oid and other not in dst[oid]:dst[oid].append(other)
del paths,at,states
summaries={}
for mode in sys.argv[1:] or ['forward-small','reverse-small','git-small','git-all']:
 t=time.time();population=git if mode=='git-all' else small;selected={};stats=collections.Counter();targetdb=OUT/(mode+'.sqlite');assert not targetdb.exists(),targetdb
 db=sqlite3.connect(targetdb);db.execute('create table records(oid text primary key,id blob,base text,raw integer,frame blob,depth integer,canonical integer,encoded integer) without rowid')
 if mode.startswith('git-'):
  order=[];visited=set()
  def visit(oid):
   if oid in visited:return
   visited.add(oid);base=git[oid]['base']
   if base in population:visit(base)
   order.append(oid)
  for oid in sorted(population):visit(oid)
  maxdepth=50;maxcanonical=maxencoded=64*1024*1024
 else:
  order=sorted(population,key=lambda oid:((first[oid] if mode=='forward-small' else -last[oid]),oid));maxdepth=8;maxcanonical=524288;maxencoded=262144
 for ordinal,oid in enumerate(order):
  data=raw(oid);full=encode(data);candidates=[];canonical=len(data)+23
  if mode.startswith('git-'):
   b=git[oid]['base']
   if b in selected:candidates=[b]
   elif b:stats['outside_population_base']+=1
  else:
   for b in (prev if mode=='forward-small' else nxt)[oid]:
    if b in selected and selected[b][0]<maxdepth and selected[b][1]+canonical<=maxcanonical:
     candidates.append(b);break
   original=small[oid]['base'];b=id_oid.get(original)
   if b in selected and b not in candidates:candidates.append(b)
  choice=(None,full,0,canonical,9+len(full));stats['full_compressed_bytes']+=len(full)
  for b in candidates:
   depth,can,enc=selected[b]
   if depth+1>maxdepth or can+canonical>maxcanonical:stats['structural_reject']+=1;continue
   frame=encode(data,raw(b));stats['candidate_encodes']+=1
   if enc+41+len(frame)>maxencoded:stats['encoded_reject']+=1;continue
   if len(frame)+32<len(choice[1])+(32 if choice[0] else 0):choice=(b,frame,depth+1,can+canonical,enc+41+len(frame))
   else:stats['cost_reject']+=1
  b,frame,d,c,e=choice;selected[oid]=(d,c,e);stats['delta' if b else 'full']+=1;stats['frame_bytes']+=len(frame);stats['record_directory_bytes']+=5+32*bool(b)+len(frame)
  stats['small_frame_bytes' if oid in small else 'other_frame_bytes']+=len(frame);stats['small_record_directory_bytes' if oid in small else 'other_record_directory_bytes']+=5+32*bool(b)+len(frame)
  ident=bytes.fromhex(small[oid]['id']) if oid in small else None;db.execute('insert into records values(?,?,?,?,?,?,?,?)',(oid,ident,b,len(data),frame,d,c,e))
  if ordinal%10000==0:db.commit();print(mode,ordinal,dict(stats),flush=True)
 db.commit();db.execute('vacuum');db.close()
 # Reconstruct from saved artifact, independent of source raw cache, authenticate every selected identity.
 db=sqlite3.connect(targetdb.as_uri()+'?mode=ro&immutable=1',uri=True);decoded=Cache();verified=0;logical=0
 def restore(oid):
  b=decoded.get(oid)
  if b is not None:return b
  ident,base,n,frame=db.execute('select id,base,raw,frame from records where oid=?',(oid,)).fetchone();b=decode(frame,n,restore(base) if base else b'');assert hashlib.sha1(b'blob '+str(n).encode()+b'\0'+b).hexdigest()==oid
  if ident:
   canonical=b'LFSO\x01'+struct.pack('>II',n+14,n+10)+b'LFS5SML\0\0\x01'+b;assert blake3(b'layerfs/object/v2\0'+canonical)==ident
  decoded.put(oid,b);return b
 for oid in order:logical+=len(restore(oid));verified+=1
 db.close();result=dict(mode=mode,stats=dict(stats),objects=len(order),verified=verified,unique_logical_bytes=logical,max_depth=max(v[0] for v in selected.values()),max_canonical_closure=max(v[1] for v in selected.values()),max_encoded_closure=max(v[2] for v in selected.values()),elapsed_seconds=time.time()-t,artifact_sha256=sha(targetdb),artifact=str(targetdb),scope='Encoded record graph; diagnostic SQLite size is NOT replacement Store size')
 summaries[mode]=result;(OUT/(mode+'.json')).write_text(json.dumps(result,indent=2));print(json.dumps(result),flush=True)
proc.stdin.close();proc.wait();assert proc.returncode==0
(OUT/'summary.json').write_text(json.dumps(dict(results=summaries,git_raw_reads=reads,elapsed_seconds=time.time()-started,manifest_seals=seals,attribution_sha256=sha(ATTR),git_inventory_sha256=sha(INV),script_sha256=sha(__file__),protocol_sha256=sha(HERE/'protocol.md')),indent=2))

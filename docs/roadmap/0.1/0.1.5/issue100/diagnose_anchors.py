"""Post-proof diagnostic of the largest actual SmallContent anchor populations.
No alternate encoding/base selection or performance claim. Never rebuild history.
"""
import collections,ctypes,ctypes.util,fcntl,hashlib,json,os,sqlite3,struct,sys,time
from pathlib import Path
run=Path(sys.argv[1]).resolve();started=time.monotonic_ns()
with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 assert json.loads((run/'verification-summary.json').read_text())['status']=='PASS'
 store=run/'deepseek-full/host-runtime/store.sqlite'
 db=sqlite3.connect(store.as_uri()+'?mode=ro&immutable=1',uri=True)
 loc={ (p,g,r):i for i,p,g,r in db.execute('select object_id,pack_id,group_number,record_number from objects') }
 records={};totals=collections.Counter();groups=collections.defaultdict(list)
 for p,b in db.execute("select pack_id,data from object_packs where substr(data,9,4)=x'03000000'"):
  n=struct.unpack_from('<I',b,12)[0]
  for g in range(n):
   off,size=struct.unpack_from('<II',b,16+16*g);r=b[off:off+size]
   kind,raw,frame=struct.unpack_from('<BII',r);base=r[9:41] if kind else None
   i=loc[p,g,0];records[i]=(p,g,kind,raw,frame,base)
   totals['delta' if kind else 'full']+=1;totals['frame_bytes']+=frame
   if kind:groups[base].append(i)
 census=json.loads((run/'census.json').read_text())['counts']
 assert totals['delta']==census['small_DELTA']['objects'] and totals['full']==census['small_FULL']['objects']
 assert totals['frame_bytes']==census['small_DELTA']['frame_bytes']+census['small_FULL']['frame_bytes']
 bins=collections.defaultdict(collections.Counter)
 for i,(p,g,kind,raw,frame,base) in records.items():
  if not kind:continue
  b=records[base];assert b[2]==0
  ratio=raw/max(b[3],1)
  key='target<=base' if ratio<=1 else 'base<target<=2x' if ratio<=2 else 'target>2xbase'
  bins[key].update(objects=1,frame_bytes=frame,target_raw_bytes=raw,base_raw_bytes_per_target=b[3])
 ranked=sorted(groups,key=lambda b:sum(records[i][4] for i in groups[b]),reverse=True)[:8]
 libpath=ctypes.util.find_library('zstd');z=ctypes.CDLL(libpath)
 z.ZSTD_createDCtx.restype=ctypes.c_void_p;z.ZSTD_freeDCtx.argtypes=[ctypes.c_void_p]
 z.ZSTD_decompress_usingDict.argtypes=[ctypes.c_void_p,ctypes.c_void_p,ctypes.c_size_t,ctypes.c_void_p,ctypes.c_size_t,ctypes.c_void_p,ctypes.c_size_t];z.ZSTD_decompress_usingDict.restype=ctypes.c_size_t
 def decode(i,base=b''):
  p,g,kind,raw,frame,_=records[i]
  blob=db.execute('select data from object_packs where pack_id=?',(p,)).fetchone()[0]
  off,size=struct.unpack_from('<II',blob,16+16*g);frame=blob[off+9+32*kind:off+size]
  out=ctypes.create_string_buffer(raw);ctx=z.ZSTD_createDCtx();assert ctx
  try:assert z.ZSTD_decompress_usingDict(ctx,out,raw,frame,len(frame),base,len(base))==raw
  finally:z.ZSTD_freeDCtx(ctx)
  return out.raw
 examples=[];wanted=set()
 for base in ranked:
  rawbase=decode(base);base_sha=hashlib.sha256(rawbase).hexdigest();wanted.add(base_sha)
  targets=[]
  for i in sorted(groups[base],key=lambda i:records[i][4],reverse=True)[:3]:
   digest=hashlib.sha256(decode(i,rawbase)).hexdigest();wanted.add(digest)
   targets.append(dict(object_id=i.hex(),raw_bytes=records[i][3],frame_bytes=records[i][4],sha256=digest))
  examples.append(dict(base_id=base.hex(),base_sha256=base_sha,base_raw_bytes=len(rawbase),dependent_count=len(groups[base]),dependent_frame_bytes=sum(records[i][4] for i in groups[base]),targets=targets))
 db.close()
 identity=json.loads((run/'identity.json').read_text());occurrences=collections.defaultdict(list)
 for state in identity['fixtures']['deepseek-full']['states']:
  oracle=json.loads(Path(state['oracle']).read_text())
  for path,(mode,size,digest) in oracle.items():
   if digest in wanted and mode in ('100644','100755'):
    occurrences[digest].append(dict(checkpoint=state['index'],path=bytes.fromhex(path).decode('utf8','backslashreplace')))
 assert wanted<=occurrences.keys()
 for e in examples:
  e['base_first_oracle_occurrence']=occurrences[e['base_sha256']][0]
  for t in e['targets']:t['first_oracle_occurrence']=occurrences[t['sha256']][0]
 result=dict(scope='Post-verification read-only diagnosis of actual immutable payload records; populations reconcile to frozen census. No candidate-selection or codec sweep.',
  totals=totals,size_relation=dict(bins),largest_anchor_populations=examples,elapsed_ns=time.monotonic_ns()-started,decoder_library=libpath)
 with (run/'anchor-diagnosis.json').open('x') as f:json.dump(result,f,indent=2)
 print(json.dumps(result,indent=2))

"""Read-only attribution: our measured FULL small objects versus the measured Git pack."""
import collections,ctypes,ctypes.util,fcntl,hashlib,json,os,sqlite3,struct,sys,time
from pathlib import Path
root=Path(sys.argv[1]).resolve();started=time.monotonic_ns()
with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
 fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 git={}
 for line in (root/'git-pack-attribution-raw.txt').read_text().splitlines():
  f=line.split()
  if len(f) in (5,7) and len(f[0])==40 and f[1]=='blob':
   git[f[0]]=(int(f[3]),f[6] if len(f)==7 else None)
 first={};paths={}
 fixture=json.loads((root/'fixture.json').read_text())['deepseek-ten']
 for state in fixture['states']:
  for line in (Path(state['input'])/'manifest.tsv').read_text().splitlines():
   mode,oid,size,path=line.split('\t');first.setdefault(oid,state['index']);paths.setdefault(oid,bytes.fromhex(path).decode('utf8','backslashreplace'))
 closure={}
 def available(oid):
  if oid not in closure:closure[oid]=max(first[oid],available(git[oid][1]) if git[oid][1] else 0)
  return closure[oid]
 run=root/'v015';assert json.loads((run/'verification-summary.json').read_text())['status']=='PASS'
 store=run/'deepseek-ten/host-runtime/store.sqlite';db=sqlite3.connect(store.as_uri()+'?mode=ro&immutable=1',uri=True)
 library=ctypes.util.find_library('zstd');z=ctypes.CDLL(library)
 z.ZSTD_decompress.argtypes=[ctypes.c_void_p,ctypes.c_size_t,ctypes.c_void_p,ctypes.c_size_t];z.ZSTD_decompress.restype=ctypes.c_size_t
 groups=collections.defaultdict(collections.Counter);examples=[]
 for pack_id,blob in db.execute("select pack_id,data from object_packs where substr(data,9,4)=x'03000000'"):
  n=struct.unpack_from('<I',blob,12)[0]
  for group in range(n):
   offset,size=struct.unpack_from('<II',blob,16+16*group);record=blob[offset:offset+size]
   kind,raw,frame=struct.unpack_from('<BII',record)
   if kind:continue
   assert 0<raw<131072 and size==frame+9
   dest=ctypes.create_string_buffer(raw);encoded=record[9:]
   assert z.ZSTD_decompress(dest,raw,encoded,len(encoded))==raw
   oid=hashlib.sha1(b'blob '+str(raw).encode()+b'\0'+dest.raw).hexdigest()
   packed,base=git[oid]
   if base is None:category='Git_also_FULL'
   else:
    when=available(base)
    category='Git_DELTA_earlier_closure' if when<first[oid] else 'Git_DELTA_same_snapshot_closure' if when==first[oid] else 'Git_DELTA_requires_later_snapshot'
   groups[category].update(objects=1,layerfs_full_frame_bytes=frame,git_packed_target_bytes=packed,raw_bytes=raw)
   if base:
    examples.append(dict(path=paths[oid],git_oid=oid,first_smoke_step=first[oid],git_base_closure_available_step=available(base),layerfs_full_frame_bytes=frame,git_packed_target_bytes=packed,category=category))
 db.close()
 frozen=json.loads((run/'census.json').read_text())['counts']['small_FULL']
 assert sum(g['objects'] for g in groups.values())==frozen['objects']
 assert sum(g['layerfs_full_frame_bytes'] for g in groups.values())==frozen['frame_bytes']
 examples=sorted(examples,key=lambda x:x['layerfs_full_frame_bytes']-x['git_packed_target_bytes'],reverse=True)[:12]
 result=dict(scope='Post-verification read-only comparison of actual selected small FULL frames and existing Git pack entries. Group populations match frozen census. Git packed target bytes are not standalone online replacement costs; base closure costs are excluded here.',groups=dict(groups),examples=examples,elapsed_ns=time.monotonic_ns()-started,decoder_library=library,script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
 with (root/'small-full-git-attribution.json').open('x') as f:json.dump(result,f,indent=2)
 print(json.dumps(result,indent=2))

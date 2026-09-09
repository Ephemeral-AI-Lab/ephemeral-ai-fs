"""Actual53-state whole-file graph + native slice adapter Store, original IDs retained."""
import collections,hashlib,importlib.util,json,pathlib,sqlite3,struct,subprocess,sys,time
sys.dont_write_bytecode=True
HERE=pathlib.Path(__file__).resolve().parent;sys.path.insert(0,str(HERE));import content_codec as codec;import reader
from cdc import lengths
SRC=HERE.parent/'metadata/v2-53/candidate.sqlite';DST=HERE/'candidate.sqlite';EXPECTED='54f47590d3f1189395b109901ab6b403c45074a9634cf7be3e28384365dfdb59'
FIXTURE=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-storage-v3-data/deepseek-stride3-218c1b81ed39c763ca22/fixture.json');REPO=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-comparison/git/snapshots.git');INV=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/content/verify-pack.txt')
ATTR=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/content/attribution.json')
def sha(p):
 with pathlib.Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def save(name,x):(HERE/name).write_text(json.dumps(x,indent=2)+'\n')
def disk(p):s=p.stat();return dict(apparent_bytes=s.st_size,allocated_bytes=s.st_blocks*512)
started=time.time();assert sha(SRC)==EXPECTED and not DST.exists();assert sha(FIXTURE)=='3b2c12b682e4892837fc810969e4e90eaa8d756c0d6f738df50746ad478fb73e'
source=reader.meta.StoreReader(SRC);db=source.db
all_old=dict((i,(n,p,g,r)) for i,n,p,g,r in db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects'))
oldpacks={};metapacks={}
for p,b in db.execute('select pack_id,data from object_packs'):
 if b[:8]==b'LFSTRC\0\0' or (b[:8]==b'LFPACK\0\0' and int.from_bytes(b[8:12],'little')==2):oldpacks[p]=len(b)
 else:metapacks[p]=hashlib.sha256(b).hexdigest()
old_content={i for i,(n,p,g,r) in all_old.items() if p in oldpacks};native={i for i,(n,p,g,r) in all_old.items() if p in oldpacks and source._pack(p)[:8]==b'LFPACK\0\0'}
assert len(native)==1998
small={r['git_oid']:bytes.fromhex(r['id']) for r in json.loads(ATTR.read_text())['rows']};assert len(small)==59768 and set(small.values())==old_content-native
fixture=json.loads(FIXTURE.read_text())['deepseek-stride3'];assert [s['full157_index'] for s in fixture['states']]==list(range(1,158,3))
regular={};file_seals={};manifestseals={}
for s in fixture['states']:
 manifest=pathlib.Path(s['input'])/'manifest.tsv';assert sha(manifest)==s['input_seal']['manifest.tsv'];manifestseals[str(s['index'])]=sha(manifest);oracle=pathlib.Path(s['oracle']);assert sha(oracle)==s['oracle_sha256'];expected=json.loads(oracle.read_text())
 for line in manifest.read_text().splitlines():
  mode,oid,n,path=line.split('\t');n=int(n)
  if mode in ('100644','100755') and n:
   regular[oid]=n;digest=expected[path][2];assert file_seals.get(oid,digest)==digest;file_seals[oid]=digest
large={o:n for o,n in regular.items() if n>=131072};assert len(large)==224 and set(regular)==set(small)|set(large) and max(large.values())<=codec.MAX_RAW
git={}
for line in INV.read_text().splitlines():
 f=line.split()
 if len(f) in (5,7) and len(f[0])==40 and f[1]=='blob':git[f[0]]=f[6] if len(f)==7 else None
assert set(regular)<=set(git)
proc=subprocess.Popen(['git','--git-dir='+str(REPO),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE);cache=collections.OrderedDict();cachebytes=0

def raw(oid):
 global cachebytes
 if oid in cache:cache.move_to_end(oid);return cache[oid]
 proc.stdin.write(oid.encode()+b'\n');proc.stdin.flush();h=proc.stdout.readline().split();assert h[:2]==[oid.encode(),b'blob'];n=int(h[2]);b=proc.stdout.read(n);assert len(b)==n and proc.stdout.read(1)==b'\n';assert n==regular[oid] and hashlib.sha1(b'blob '+str(n).encode()+b'\0'+b).hexdigest()==oid and hashlib.sha256(b).hexdigest()==file_seals[oid]
 while cache and cachebytes+len(b)>67108864:_,old=cache.popitem(last=False);cachebytes-=len(old)
 cache[oid]=b;cachebytes+=len(b);return b
ids=dict(small);coverage={};nativechecks={};large_identities=[]
for oid in sorted(large):
 data=raw(oid);ident=codec.object_id('large',data);assert ident not in all_old and ident not in ids.values();ids[oid]=ident;cursor=0
 for size in lengths(data):
  part=data[cursor:cursor+size];chunk=codec.object_id('native',part)
  if chunk in native:
   if chunk not in nativechecks:
    actual=source.read_canonical(chunk);assert actual==codec.canonical('native',part);nativechecks[chunk]=hashlib.sha256(part).hexdigest()
   coverage.setdefault(chunk,(ident,cursor,size))
  cursor+=size
 assert cursor==len(data);large_identities.append(dict(oid=oid,id=ident.hex(),raw_bytes=len(data)))
print('COVERAGE',len(coverage),'native',len(native),'large',len(large),flush=True)
# Original source canonical values must equal independently authenticated selected Git bytes.
source_small_verified=0
for oid,ident in small.items():
 data=raw(oid);assert source.read_canonical(ident)==codec.canonical('small',data);source_small_verified+=1
for ident in native-coverage.keys():
 actual=source.read_canonical(ident);assert actual[13:21]==b'LFS4CHK\0';nativechecks[ident]=hashlib.sha256(actual[21:]).hexdigest()
assert len(nativechecks)==len(native)
print('SOURCE AUTH',source_small_verified,len(nativechecks),flush=True)
# The fixed actual Git DAG defines candidate availability. Every final graph choice is updated before descendants.
order=[];visited=set()
def visit(oid):
 if oid in visited:return
 visited.add(oid);base=git[oid]
 if base in regular:visit(base)
 order.append(oid)
for oid in sorted(regular):visit(oid)
chosen={};closures={};rows=[];stats=collections.Counter()
for ordinal,oid in enumerate(order):
 data=raw(oid);ident=ids[oid];role='small' if oid in small else 'large';full=codec.encode(data);n=len(data)+(23 if role=='small' else 21);base=git[oid];frame=full;selected=None;depth=0;can=n;enc=1+len(full)
 if base in chosen:
  bd,bc,be=closures[base]
  if bd+1<=50 and bc+n<=67108864:
   alternative=codec.encode(data,raw(base));stats['candidate_encodes']+=1
   if len(alternative)+32<len(full) and be+33+len(alternative)<=67108864:frame=alternative;selected=base;depth=bd+1;can=bc+n;enc=be+33+len(frame)
   else:stats['cost_or_encoded_reject']+=1
  else:stats['structural_reject']+=1
 elif base:stats['outside_population_full']+=1
 kind=(0 if role=='small' else 2)+bool(selected);record=bytes([kind])+(ids[selected] if selected else b'')+frame
 chosen[oid]=record;closures[oid]=(depth,can,enc);rows.append((ident,n,record));stats[role+'_frames']+=len(frame);stats[role+'_records']+=len(record);stats[role+'_delta' if selected else role+'_full']+=1
 if ordinal%10000==0:print('ENCODE',ordinal,dict(stats),flush=True)
# Count every original native identity: mapped slice or explicit FULL fallback.
for ident in sorted(native):
 n=all_old[ident][0]
 if ident in coverage:
  owner,offset,length=coverage[ident];record=b'\4'+owner+struct.pack('<II',offset,length);stats['native_slice_records']+=1
 else:
  data=source.read_canonical(ident)[21:];record=b'\5'+codec.encode(data,window=20);stats['native_fallback_full_records']+=1
 rows.append((ident,n,record));stats['native_record_bytes']+=len(record)
proc.stdin.close();proc.wait();assert proc.returncode==0
assert {i for i,n,r in rows}==old_content|{ids[o] for o in large}
# Back up the actual verified source; preserve the complete existing index schema and typed SQL.
typed_tables=[name for (name,) in db.execute("select name from sqlite_schema where type='table' and name not in ('objects','object_packs')")];typed={name:list(db.execute('select * from '+name)) for name in typed_tables};schemas=list(db.execute('select type,name,tbl_name,sql from sqlite_schema order by name'))
out=sqlite3.connect(DST);db.backup(out);out.execute('pragma foreign_keys=on');nextpack=out.execute('select max(pack_id)+1 from object_packs').fetchone()[0];out.execute('begin');packs=[];pending=[];payload=0

def flush():
 global nextpack,pending,payload
 if not pending:return
 count=len(pending);start=16+4*count;offsets=[];body=[]
 for ident,n,record in pending:offsets.append(start);body.append(record);start+=len(record)
 blob=reader.MAGIC+struct.pack('<II',107,count)+struct.pack('<'+'I'*count,*offsets)+b''.join(body);assert len(blob)<=reader.MAX_PACK
 out.execute('insert into object_packs values(?,?)',(nextpack,blob))
 for g,(ident,n,record) in enumerate(pending):
  if ident in all_old:out.execute('update objects set pack_id=?,group_number=?,record_number=0 where object_id=?',(nextpack,g,ident))
  else:out.execute('insert into objects values(?,?,?,?,?)',(ident,n,nextpack,g,0))
 packs.append(dict(pack_id=nextpack,bytes=len(blob),records=count));nextpack+=1;pending=[];payload=0
for row in rows:
 if pending and (len(pending)==256 or 16+4*(len(pending)+1)+payload+len(row[2])>reader.MAX_PACK):flush()
 pending.append(row);payload+=len(row[2])
flush()
for p in oldpacks:out.execute('delete from object_packs where pack_id=?',(p,))
assert list(out.execute('pragma foreign_key_check'))==[];out.commit();out.execute('vacuum');assert out.execute('pragma integrity_check').fetchone()[0]=='ok';assert list(out.execute('pragma foreign_key_check'))==[]
assert list(out.execute('select type,name,tbl_name,sql from sqlite_schema order by name'))==schemas
for table,values in typed.items():assert list(out.execute('select * from '+table))==values
for p,h in metapacks.items():assert hashlib.sha256(out.execute('select data from object_packs where pack_id=?',(p,)).fetchone()[0]).hexdigest()==h
newobjects=dict((i,n) for i,n in out.execute('select object_id,canonical_length from objects'));assert set(newobjects)==set(all_old)|{ids[o] for o in large};assert all(newobjects[i]==v[0] for i,v in all_old.items())
packbytes=out.execute('select sum(length(data)) from object_packs').fetchone()[0];indexpages=list(out.execute('select name,sum(pgsize) from dbstat group by name'));out.close();source.close();assert sha(SRC)==EXPECTED
identity_by_id={i:o for o,i in ids.items()};candidate=reader.StoreReader(DST);verify=collections.Counter()
for ordinal,(ident,n,record) in enumerate(rows):
 data=candidate.read_canonical(ident);assert len(data)==n
 if ident in native:assert hashlib.sha256(data[21:]).hexdigest()==nativechecks[ident];verify['native']+=1
 else:
  oid=identity_by_id[ident];header=23 if oid in small else 21;b=data[header:];assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==oid and hashlib.sha256(b).hexdigest()==file_seals[oid];verify['small' if oid in small else 'large']+=1
 assert candidate.cache_bytes<=candidate.cache_limit and candidate.pack_bytes<=candidate.pack_limit
 if ordinal%10000==0:print('VERIFY',ordinal,flush=True)
metrics=dict(candidate.metrics);candidate.close()
result=dict(status='PASS',scope='Actual content-only copy over verified metadata baseline; parent all53filesystem oracle verification still required',source=str(SRC),source_sha256=EXPECTED,candidate=str(DST),candidate_sha256=sha(DST),source_disk=disk(SRC),candidate_disk=disk(DST),allocated_saved=disk(SRC)['allocated_bytes']-disk(DST)['allocated_bytes'],original_content_objects=len(old_content),new_whole_file_objects=len(large),source_content_authenticated=dict(small=source_small_verified,native=len(nativechecks)),candidate_content_authenticated=dict(verify),native_coverage=dict(mapped=len(coverage),fallback=len(native)-len(coverage)),statistics=dict(stats),new_content_pack_bytes=sum(p['bytes'] for p in packs),new_content_pack_count=len(packs),old_content_pack_bytes=sum(oldpacks.values()),all_pack_bytes=packbytes,sqlite_nonpack_bytes=disk(DST)['apparent_bytes']-packbytes,dbstat=indexpages,reader_metrics=metrics,maximum_selected_depth=max(v[0] for v in closures.values()),maximum_selected_canonical_closure=max(v[1] for v in closures.values()),maximum_selected_encoded_closure=max(v[2] for v in closures.values()),maximum_frame_bytes=max(len(r)-1-32*(r[0] in (1,3)) for i,n,r in rows if r[0]!=4),maximum_pack_bytes=max(p['bytes'] for p in packs),schema_typed_sql_metadata_packs_unchanged=True,original_canonical_ids_lengths_retained=True,manifest_seals=manifestseals,protocol_sha256=sha(HERE/'protocol.md'),script_sha256=sha(__file__),reader_sha256=sha(HERE/'reader.py'),codec_sha256=sha(HERE/'content_codec.py'),fixture_sha256=sha(FIXTURE),git_inventory_sha256=sha(INV),source_attribution_sha256=sha(ATTR),elapsed_seconds=time.time()-started)
save('result.json',result);save('coverage.json',dict(large_objects=large_identities,native_adapters=[dict(id=i.hex(),owner=o.hex(),offset=p,length=n) for i,(o,p,n) in coverage.items()],native_fallback_ids=[i.hex() for i in native-coverage.keys()],packs=packs));print(json.dumps(result,indent=2),flush=True)

"""Two fixed whole-copy layouts, unchanged canonical inventory/content/history."""
import collections,hashlib,json,pathlib,shutil,sqlite3,struct,time
from store_api import StoreReader,delta_api,old
HERE=pathlib.Path(__file__).resolve().parent
SOURCE_ROOT=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157')
SRC=SOURCE_ROOT/'combined/D-B-CDC.sqlite';EXPECTED='20bc3581f29ecab3168712b8972f2eb24898f92d98e94d402657e953048efa6b'
sha=old.sha;check=old.check

def table_rows(db):
 return {name:sorted(db.execute('select * from '+name),key=repr) for name, in db.execute("select name from sqlite_schema where type='table' and name not in ('objects','object_packs')")}
def metadata_records(db):
 byloc={(p,g,r):(i,n) for i,n,p,g,r in db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')}
 records={};locations={};packs={};seen=set()
 for p,b in db.execute('select pack_id,data from object_packs order by pack_id'):
  if b[:8]!=b'LFPACK\0\0' or struct.unpack_from('<I',b,8)[0]!=1:continue
  version,groups=old.groups(b);packs[p]=hashlib.sha256(b).hexdigest()
  for g,rows in enumerate(groups):
   for r,record in enumerate(rows):
    check((p,g,r) in byloc,'unlocated metadata');identity,n=byloc[p,g,r]
    check(identity not in records,'duplicate metadata identity');records[identity]=record;locations[identity]=(p,g,r,n);seen.add((p,g,r))
 expected={key for key in byloc if key[0] in packs};check(expected==seen,'metadata locator coverage')
 return records,locations,packs

def reader_fixtures(inv,ledger):
 target=next(i for i,s,k,b,r in ledger if k==1)
 records={i:r for i,s,k,b,r in ledger};record=records[target];base=record[1:33];base_record=records[base]
 def fake(trec=record,brec=base_record,baseid=base,future=False):
  obj=StoreReader.__new__(StoreReader);obj.cache=collections.OrderedDict();obj.cache_bytes=0;obj.cache_limit=65536
  obj.loc={target:(2,0,0,len(inv[target]))}
  mapping={target:(1,trec,1)}
  if brec is not None:
   obj.loc[baseid]=(3 if future else 1,0,0,len(brec)-1);mapping[baseid]=(1,brec,1)
  obj._physical=lambda i:mapping[i]
  return obj
 check(fake().read_canonical(target)==inv[target],'valid reader delta')
 tests=[]
 wrongid=next(i for i,b in inv.items() if b[13:21]==b'LFS4MET\0')
 cases=[('missing_base',lambda:fake(brec=None)),('corrupt_base',lambda:fake(brec=base_record[:-1]+bytes([base_record[-1]^1]))),('non_FULL_base',lambda:fake(brec=record)),('future_base',lambda:fake(future=True)),('wrong_role_base',lambda:fake(trec=record[:1]+wrongid+record[33:],brec=b'\0'+inv[wrongid],baseid=wrongid))]
 for name,build in cases:
  try:build().read_canonical(target)
  except (AssertionError,ValueError,KeyError,struct.error):tests.append(name)
  else:raise AssertionError('reader accepted '+name)
 return dict(valid_delta='PASS',rejected=tests)

start=time.monotonic();check(sha(SRC)==EXPECTED,'source SHA')
inv,roots,unused_content,scope,high=delta_api.original_inventory();check(len(inv)==24748 and len(roots)==158 and high==52724,'inventory')
ledger=delta_api.record_ledger();reader_checks=reader_fixtures(inv,ledger)
source=sqlite3.connect(SRC.as_uri()+'?mode=ro&immutable=1',uri=True)
source_records,source_locs,source_pack_hashes=metadata_records(source)
check(set(source_records)==set(inv),'source metadata membership')
for i,r in source_records.items():check(r[:1]==b'\0' and r[1:]==inv[i] and old.blake3(b'layerfs/object/v2\0'+r[1:])==i,'source exact metadata')
source_rows=list(source.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects order by object_id'))
content_rows=[r for r in source_rows if r[0] not in inv];check(len(content_rows)==78619,'content locator count')
content_packs={p:b for p,b in source.execute('select pack_id,data from object_packs') if p not in source_pack_hashes}
check(len(content_packs)==2180,'content pack count');content_digest=hashlib.sha256(b''.join(p.to_bytes(8,'little')+hashlib.sha256(b).digest() for p,b in sorted(content_packs.items()))).hexdigest()
unchanged_sql=table_rows(source);maxpack=source.execute('select max(pack_id) from object_packs').fetchone()[0]
source_canonical_columns=[r[:2] for r in source_rows]
metadata_result=json.loads((HERE.parent/'metadata/result.json').read_text());results={};locator_control=None
old_proof=SOURCE_ROOT/'verification/result.json';proof=json.loads(old_proof.read_text());check(proof['status']=='PASS' and proof['states']==157 and proof['copy_sha256_after']==EXPECTED,'prior oracle proof')
for layout in ('full','delta'):
 packs,locators=delta_api.pack_blobs(inv,maxpack,layout=layout)
 expected=metadata_result['layouts'][layout]['pack_bytes'];check(len(packs)==388 and sum(len(b) for p,b in packs)==expected,'sealed layout')
 if locator_control is None:locator_control=locators
 else:check(locators==locator_control,'matched physical membership')
 path=HERE/(layout+'.sqlite')
 with path.open('xb'):pass
 db=sqlite3.connect(path);source.backup(db);db.execute('pragma foreign_keys=ON');db.execute('begin');db.execute('pragma defer_foreign_keys=ON')
 db.executemany('insert into object_packs values(?,?)',packs)
 db.executemany('update objects set pack_id=?,group_number=?,record_number=? where object_id=?',[(p,g,r,i) for i,n,p,g,r in locators])
 db.executemany('delete from object_packs where pack_id=?',[(p,) for p in source_pack_hashes]);db.commit()
 check(db.execute('pragma foreign_key_check').fetchone() is None,'foreign keys')
 check(table_rows(db)==unchanged_sql,'unchanged root/typed/allocator SQL')
 rows=list(db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects order by object_id'))
 check([r[:2] for r in rows]==source_canonical_columns,'all canonical IDs/lengths')
 check([r for r in rows if r[0] not in inv]==content_rows,'content locator identity')
 for p,b in content_packs.items():check(db.execute('select data from object_packs where pack_id=?',(p,)).fetchone()==(b,),'unchanged content BLOB')
 actual,loc,pack_hashes=metadata_records(db);check(set(actual)==set(inv),'metadata membership')
 maxclosure=0;deltas=0;bases=set()
 for i,r in actual.items():
  if r[:1]==b'\1':
   base=r[1:33];check(base in loc and loc[base][0]<loc[i][0],'earlier selected metadata base')
   check(actual[base][:1]==b'\0','depth1 selected FULL');bases.add(base);deltas+=1;maxclosure=max(maxclosure,len(inv[i])+len(inv[base]))
  canonical=delta_api.decode_record(i,actual);check(canonical==inv[i] and len(canonical)==loc[i][3],'exact canonical metadata')
 check(deltas==(0 if layout=='full' else 4242),'selected DELTA count');check(maxclosure<=16384,'closure cap')
 # Metadata coverage plus identical content packs/locators carries prior complete physical membership proof.
 check(len(rows)==103367 and len(pack_hashes)+len(content_packs)==2568,'whole-copy count')
 db.execute('vacuum');check(db.execute('pragma integrity_check').fetchone()==('ok',),'SQLite integrity');check(db.execute('pragma foreign_key_check').fetchone() is None,'final FKs')
 pages={n:dict(bytes=b,payload=p,unused=u) for n,b,p,u in db.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')}
 categories=dict(metadata_pack_bytes=expected,small_pack_bytes=58979700,native_pack_bytes=7211036);packbytes=sum(categories.values())
 check(db.execute('select sum(length(data)) from object_packs').fetchone()[0]==packbytes,'all pack bytes')
 db.close();st=path.stat();reader=StoreReader(path,cache_bytes=1024*1024,pack_cache_bytes=262144)
 for i in roots:check(reader.read_canonical(i)==inv[i],'actual root reader')
 target_samples=[i for i,r in actual.items() if r[:1]==b'\1'][:32]
 for i in target_samples:check(reader.read_canonical(i)==inv[i],'actual DELTA reader')
 reader.close()
 results[layout]=dict(path=str(path),sha256=sha(path),logical_bytes=st.st_size,allocated_bytes=st.st_blocks*512,objects=len(rows),packs=2568,metadata_objects=len(actual),metadata_packs=len(pack_hashes),metadata_deltas=deltas,physical_FULL_bases=len(bases),maximum_canonical_closure=maxclosure,pack_categories=categories,all_pack_bytes=packbytes,sqlite_nonpack_bytes=st.st_size-packbytes,allocation_difference=st.st_blocks*512-st.st_size,pages=pages,metadata_canonical_byte_equal=True,all_canonical_IDs_lengths_unchanged=True,content_pack_BLOBs_and_locators_unchanged=True,root_typed_history_allocator_SQL_unchanged=True,physical_membership='PASS',sql_integrity='PASS',sql_foreign_keys='PASS',actual_reader_root_and_delta_checks=158+len(target_samples))
 print(layout,st.st_size,st.st_blocks*512,flush=True)
source.close();check(sha(SRC)==EXPECTED,'source unchanged')
identity=SOURCE_ROOT/'combined/identity-map.json';target=HERE/'identity-map.json';check(not target.exists(),'new identity map');shutil.copyfile(identity,target);check(sha(identity)==sha(target),'identity map exact copy')
result=dict(scope='Two fixed offline physical metadata layouts; unchanged canonical IDs/content/history/allocator. No product format, online allocation, latency or rollback qualification.',source=str(SRC),source_sha256_before_and_after=EXPECTED,source_logical_bytes=SRC.stat().st_size,results=results,matched_saved_logical_bytes=results['full']['logical_bytes']-results['delta']['logical_bytes'],matched_saved_pack_bytes=results['full']['all_pack_bytes']-results['delta']['all_pack_bytes'],saved_vs_prior_hashsorted_copy=SRC.stat().st_size-results['delta']['logical_bytes'],identical_metadata_locator_membership_between_treatments=True,identity_map_sha256=sha(target),reader_fixtures=reader_checks,prior_full157_oracle_proof=dict(path=str(old_proof),sha256=sha(old_proof),states=157,path_states=proof['path_states'],logical_bytes=proof['logical_bytes'],reused_by='Exact unchanged canonical metadata, content BLOBs/locators and SQL roots/history; parent additionally verifies new reader separately.'),content_pack_digest=content_digest,protocol_sha256=sha(HERE/'protocol.md'),script_sha256=sha(__file__),reader_sha256=sha(HERE/'store_api.py'),metadata_cache_sha256=sha(HERE.parent/'metadata/cache.sqlite'),metadata_result_sha256=sha(HERE.parent/'metadata/result.json'),elapsed_seconds=time.monotonic()-start)
(HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='results'},indent=2))

"""Single fixed full157 diagnostic combination; see protocol, no source writes."""
from store_api import *
start=time.monotonic();check(sha(SRC)==EXPECTED,'source identity')
baseline=json.loads((HERE/'baseline.json').read_text());check(sha(baseline['path'])==baseline['sha256'],'baseline reuse hash')
source=sqlite3.connect(SRC.as_uri()+'?mode=ro&immutable=1',uri=True)
# Only expected SQL foreign-key references to object identities are allowed.
refs=set()
for name, in source.execute("select name from sqlite_schema where type='table'"):
 for r in source.execute('pragma foreign_key_list('+name+')'):
  if r[2]=='objects':refs.add((name,r[3],r[4]))
check(refs=={(t,'root_id','object_id') for t in ('commits','layers','workspace_stages')},'unexpected object references')
rootchecks=json.loads((EXP/'metadata'/'roots.json').read_text());rootmap={bytes.fromhex(r['original_root']):bytes.fromhex(r['new_root']) for r in rootchecks}
inventory,newroots,content,scope,highwater=d_api.original_inventory();check(set(newroots)==set(rootmap.values()),'D root mapping')
original,rewritten,lm,cm=derive_sql(source,rootmap)
maxpack=source.execute('select max(pack_id) from object_packs').fetchone()[0];packs,locators=d_api.pack_blobs(inventory,maxpack)
dresult=json.loads((EXP/'metadata'/'result.json').read_text());check(len(inventory)==24748 and len(packs)==314 and sum(len(b) for p,b in packs)==34917104,'D inventory totals')
check([hashlib.sha256(b).hexdigest() for p,b in packs]==[v for k,v in sorted(dresult['pack_sha256'].items(),key=lambda x:int(x[0]))],'D pack reproducibility')
source_rows=list(source.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects'));byloc={(p,g,r):(i,n) for i,n,p,g,r in source_rows}
oldmeta=[p for p,b in source.execute('select pack_id,data from object_packs') if fr.u32(b,8)==1]
selected=json.loads((EXP/'cdc'/'family-result.json').read_text())['selected_graph'];check(len(selected)==1347,'family graph')
all_native=json.loads((EXP/'cdc'/'all-native-verification.json').read_text())
seen_ids=set(inventory)|{r[0] for r in content};z=fr.setup_decoder();small_verified=None;results={}
for treatment,cdc,version in [('D-B-CDC',True,9301)]:
 path=HERE/(treatment+'.sqlite')
 with path.open('xb'):pass
 copy=sqlite3.connect(path);source.backup(copy);copy.execute('pragma foreign_keys=ON');copy.execute('begin');copy.execute('pragma defer_foreign_keys=ON')
 copy.executemany('delete from objects where pack_id=?',[(p,) for p in oldmeta]);copy.executemany('delete from object_packs where pack_id=?',[(p,) for p in oldmeta])
 copy.executemany('insert into object_packs(pack_id,data) values(?,?)',packs);copy.executemany('insert into objects values(?,?,?,?,?)',locators)
 # All updates occur within one deferred-FK transaction; each original row is replaced by its rederived equivalent.
 for table in ('layers','commits','branches','layer_stacks','workspace_stages'):
  pk={'layers':'layer_id','commits':'commit_id','branches':'branch_id','layer_stacks':'layer_stack_id','workspace_stages':'workspace_id'}[table]
  for old,new in zip(original[table],rewritten[table]):
   columns=list(new);copy.execute('update '+table+' set '+','.join(k+'=?' for k in columns)+' where '+pk+'=?',[new[k] for k in columns]+[old[pk]])
 copy.execute('CREATE TABLE scope_allocator(scope BLOB PRIMARY KEY CHECK(length(scope)=32),highwater INTEGER NOT NULL CHECK(highwater>=0)) STRICT, WITHOUT ROWID');copy.execute('insert into scope_allocator values(?,?)',(scope,highwater))
 for p,b in source.execute('select pack_id,data from object_packs'):
  v=fr.u32(b,8)
  if v==3:
   rs=fr.original(b);compact=fr.encode(rs,102);lengths=[byloc[p,g,0][1] for g in range(len(rs))]
   check(fr.restore(fr.unpack(compact,102,lengths))==b,'framing roundtrip');copy.execute('update object_packs set data=? where pack_id=?',(compact,p))
  elif v==2 and cdc:copy.execute('update object_packs set data=? where pack_id=?',(rebuild_native(b,p,byloc,selected),p))
 copy.execute('pragma user_version='+str(version));copy.commit()
 check(copy.execute('pragma foreign_key_check').fetchone() is None,'foreign keys')
 for table,expected in rewritten.items():
  cur=copy.execute('select * from '+table);columns=[d[0] for d in cur.description];got=[dict(zip(columns,r)) for r in cur]
  check(sorted(map(repr,got))==sorted(map(repr,expected)),'SQL rewrite '+table)
 gotcontent=[r for r in copy.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects') if r[0] not in inventory]
 check(sorted(gotcontent)==sorted(content),'unchanged full content locators')
 categories,smallcheck,nativecheck=verify_copy(copy,inventory,newroots,small_verified,z);small_verified=smallcheck
 check(categories['metadata_pack_bytes']==34917104 and categories['small_pack_bytes']==58979700 and categories['native_pack_bytes']==all_native['summary']['simulated_v2_pack_bytes'],'exact categories')
 copy.execute('vacuum');check(copy.execute('pragma integrity_check').fetchone()==('ok',),'SQLite integrity')
 pages={name:dict(bytes=size,payload=payload,unused=unused) for name,size,payload,unused in copy.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')}
 check(copy.execute('select scope,highwater from scope_allocator').fetchone()==(scope,52724),'allocator')
 count=copy.execute('select count(*) from objects').fetchone()[0];packcount=copy.execute('select count(*) from object_packs').fetchone()[0];copy.close();st=path.stat();logical=st.st_size;allocated=st.st_blocks*512
 results[treatment]=dict(path=str(path),sha256=sha(path),objects=count,packs=packcount,logical_bytes=logical,copy_allocated_bytes=allocated,pack_categories=categories,all_pack_bytes=sum(categories.values()),sqlite_nonpack_bytes=logical-sum(categories.values()),allocation_difference=allocated-logical,saving_vs_equal_vacuumed_baseline=baseline['logical_bytes']-logical,pages=pages,small_verification=smallcheck,native_verification=nativecheck,sql_foreign_keys='PASS',sql_integrity='PASS',all_physical_locator_membership='PASS',metadata_inventory_and_namespace_checks='PASS')
 reader=StoreReader(path,cache_bytes=1048576,pack_cache_bytes=262144)
 sample=list(newroots)+[r[0] for r in content[:24]]
 for i in sample:
  canonical=reader.read_canonical(i)
  check(blake3(b'layerfs/object/v2\0'+canonical)==i,'point reader selfcheck')
  if i in inventory:check(canonical==inventory[i],'point metadata equality')
 check(reader.cache_bytes<=1048576 and reader.pack_bytes<=262144,'point cache caps')
 reader.close();results[treatment]['point_reader_selfcheck_objects']=len(sample)
 print(treatment,logical,allocated,flush=True)
source.close();check(sha(SRC)==EXPECTED,'source unchanged')
mapping={'namespace_roots':{k.hex():v.hex() for k,v in rootmap.items()},'layers':{k.hex():v.hex() for k,v in lm.items()},'commits':{k.hex():v.hex() for k,v in cm.items()}}
(HERE/'identity-map.json').write_text(json.dumps(mapping,indent=2)+'\n')
result=dict(scope='Combined unsupported offline complete-copy layout. Canonical metadata and typed history IDs change; content IDs and snapshot semantics retained. No product open/rollback/history API or public performance qualification.',source_sha256_before_and_after=EXPECTED,baseline=baseline,protocol_sha256=sha(HERE/'protocol.md'),script_sha256=sha(__file__),results=results,semantic_namespace_states=158,original_typed_identites_verified=True,typed_identity_cascade=True,scope_hex=scope.hex(),allocator_highwater=highwater,elapsed_seconds=time.monotonic()-start,input_hashes={str(p.relative_to(EXP)):sha(p) for p in [EXP/'metadata'/'api.py',EXP/'metadata'/'scoped.py',EXP/'metadata'/'codec.py',EXP/'metadata'/'roots.json',EXP/'cdc'/'family-result.json',EXP/'cdc'/'all-native-verification.json',HERE/'store_api.py']})
(HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='results'},indent=2))

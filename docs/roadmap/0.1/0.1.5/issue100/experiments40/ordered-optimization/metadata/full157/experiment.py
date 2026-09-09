"""One fixed authenticated shared-value physical metadata experiment."""
import collections as C, hashlib, json, pathlib, sqlite3, struct, sys, time
HERE=pathlib.Path(__file__).resolve().parent
BASE=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural')
sys.path.insert(0,str(BASE/'metadata'));import experiment as e
sys.path.insert(0,str(BASE/'combined'));import store_api as old
SOURCE=BASE/'combined/candidate.sqlite'
EXPECTED='e5837e3484d561225fa464433ed94410705bdb6cde1f60b2c057faa834a73281'
MAGIC=b'LFSIVL1\0'
def sha(p):
 with pathlib.Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def pool_canonical(v):
 assert len(v)==73
 return e.codec.canonical(MAGIC+v)
def restored(body,values):
 assert e.leaf(body) and (len(body)-44)%12==0
 out=body[:44]+b''.join(body[p:p+8]+values[int.from_bytes(body[p+8:p+12],'big')] for p in range(44,len(body),12))
 assert len(out)<=8192 and len(out)==int.from_bytes(out[5:9],'big')+9
 return out

def main():
 start=time.monotonic();assert sha(SOURCE)==EXPECTED
 for n in ('candidate.sqlite','control.sqlite','result.json','cache.sqlite'):assert not(HERE/n).exists(),('appendonly',n)
 inv,roots,content,scope,high=e.D.original_inventory();assert len(inv)==24748 and len(roots)==158
 cohorts,rootleaves,allkeys,first,unrooted=e.chronology(inv,roots)
 raw_values={b[p+8:p+81] for b in inv.values() if e.leaf(b) for p in range(44,len(b),81)}
 pool={e.codec.oid(pool_canonical(v)):pool_canonical(v) for v in raw_values};assert len(pool)==len(raw_values)<2**32
 ids=sorted(pool);ordinal={pool[i][-73:]:n for n,i in enumerate(ids,1)};values={n:pool[i][-73:] for n,i in enumerate(ids,1)}
 shared={i:(b[:44]+b''.join(b[p:p+8]+ordinal[b[p+8:p+81]].to_bytes(4,'big') for p in range(44,len(b),81)) if e.leaf(b) else b) for i,b in inv.items()}
 assert all((restored(shared[i],values) if e.leaf(b) else shared[i])==b for i,b in inv.items())
 def closure(identity,records):
  seen=set();total=0;depth=0;valuework=0
  while True:
   assert identity not in seen;seen.add(identity);total+=len(inv[identity])
   if e.leaf(inv[identity]):valuework+=(len(inv[identity])-44)//81*94
   r=records[identity]
   if not r[0]:break
   assert r[0]==1;identity=r[1:33];depth+=1
  assert depth<=16 and total<=131072 and valuework<=196608
  return depth,total,valuework
 records={};packs=[];locators=[];counters=C.Counter();groups_ledger=[]
 for step,cohort in enumerate(cohorts):
  previous=rootleaves[step-1] if step else [];remaining=16*1024*1024;trials=0;pending=[];full_pack_size=16;recordcount=0
  def flush():
   nonlocal pending,full_pack_size,recordcount
   if not pending:return
   p=len(packs)+1;packs.append((p,e.assemble(pending,'delta')))
   for g,group in enumerate(pending):
    for k,i in enumerate(group['ids']):locators.append((i,len(inv[i]),p,g,k))
   pending=[];full_pack_size=16;recordcount=0
  for groupids in e.grouped(inv,cohort):
   candidates={}
   for i in groupids:
    if not e.leaf(inv[i]):continue
    targetkeys=allkeys[i];ts=set(targetkeys);rank=[]
    for origin in previous:
     oldkeys=allkeys[origin]
     if min(targetkeys[-1],oldkeys[-1])<max(targetkeys[0],oldkeys[0]):continue
     overlap=len(ts.intersection(oldkeys))
     if overlap:rank.append((-overlap,origin))
    if not rank:counters['no_previous_or_overlap']+=1;continue
    _,origin=min(rank);assert first[origin]<step and origin in records
    if closure(origin,records)[0]>=16 or closure(origin,records)[1]+len(inv[i])>131072:counters['closure_rejected']+=1;continue
    if trials>=512 or not remaining:counters['root_budget_rejected']+=1;continue
    trials+=1;candidate,remaining,counts=e.matcher.delta(origin,shared[origin],shared[i],remaining);counters.update(counts)
    if candidate is None:counters['matcher_no_candidate']+=1;continue
    assert e.matcher.replay(candidate,origin,shared[origin])==shared[i]
    candidates[i]=candidate
   full=e.frame([b'\0'+shared[i] for i in groupids]);chosen=full;use=False
   if candidates:
    mixed=e.frame([candidates.get(i,b'\0'+shared[i]) for i in groupids]);use=len(full['data'])-len(mixed['data'])>=max(64,(len(full['data'])+7)//8)
    if use:chosen=mixed
    counters['selected_groups' if use else 'rejected_groups']+=1
   for i in groupids:
    records[i]=candidates[i] if use and i in candidates else b'\0'+shared[i]
    closure(i,records)
   # Bound pack membership with original full canonical bytes, identical to retained policy.
   original_full=e.frame([b'\0'+inv[i] for i in groupids])
   if pending and (full_pack_size+16+len(original_full['body'])>262144 or len(pending)==256 or recordcount+len(groupids)>8191):flush()
   pending.append(dict(ids=groupids,full=original_full,delta=chosen));full_pack_size+=16+len(original_full['body']);recordcount+=len(groupids)
   groups_ledger.append(dict(step=step,records=len(groupids),full_bytes=len(full['data']),selected_bytes=len(chosen['data'])))
  flush()
  if step%20==0:print('encoded metadata root',step+1,'of',len(roots),flush=True)
 assert len(records)==len(inv)
 # FULL-only pools use the exact fixed metadata group and pack codec.
 import pack_api
 poolpacks,poolloc=pack_api.pack_blobs(pool,len(packs));assert len(poolloc)==len(pool)
 cache=sqlite3.connect(HERE/'cache.sqlite');cache.execute('create table records(id blob primary key,record blob) without rowid');cache.executemany('insert into records values(?,?)',sorted(records.items()));cache.commit();cache.close()
 source=sqlite3.connect(SOURCE.as_uri()+'?mode=ro&immutable=1',uri=True)
 control=sqlite3.connect(HERE/'control.sqlite');source.backup(control);control.execute('vacuum');control.close()
 db=sqlite3.connect(HERE/'candidate.sqlite');source.backup(db);db.execute('pragma foreign_keys=on');db.execute('begin');db.execute('pragma defer_foreign_keys=on')
 oldpacks={p for p,b in db.execute('select pack_id,data from object_packs') if int.from_bytes(b[8:12],'little')==1}
 original_objects={i for i,p in db.execute('select object_id,pack_id from objects') if p in oldpacks};assert original_objects==set(inv)
 content_rows=list(db.execute('select * from objects where pack_id not in ('+','.join('?'*len(oldpacks))+') order by object_id',list(oldpacks)))
 content_packs={p:hashlib.sha256(b).hexdigest() for p,b in db.execute('select pack_id,data from object_packs') if p not in oldpacks}
 sql={name:list(db.execute('select * from '+name)) for name in ('layers','commits','branches','layer_stacks','workspace_stages','scope_allocator')}
 db.executemany('delete from objects where pack_id=?',[(p,) for p in oldpacks]);db.executemany('delete from object_packs where pack_id=?',[(p,) for p in oldpacks])
 offset=db.execute('select max(pack_id) from object_packs').fetchone()[0]
 db.executemany('insert into object_packs values(?,?)',[(p+offset,b) for p,b in packs+poolpacks]);db.executemany('insert into objects values(?,?,?,?,?)',[(i,n,p+offset,g,k) for i,n,p,g,k in locators+poolloc])
 db.execute('create table shared_inode_values(ordinal INTEGER PRIMARY KEY CHECK(ordinal BETWEEN 1 AND 4294967295), object_id BLOB UNIQUE NOT NULL REFERENCES objects(object_id) CHECK(length(object_id)=32)) STRICT')
 db.executemany('insert into shared_inode_values values(?,?)',enumerate(ids,1));db.execute('pragma user_version=9303');db.commit();db.execute('vacuum')
 assert db.execute('pragma integrity_check').fetchone()==('ok',) and not list(db.execute('pragma foreign_key_check'))
 assert all(list(db.execute('select * from '+n))==rows for n,rows in sql.items())
 assert {p:hashlib.sha256(b).hexdigest() for p,b in db.execute('select pack_id,data from object_packs') if p in content_packs}==content_packs
 assert list(db.execute('select * from objects where pack_id in ('+','.join('?'*len(content_packs))+') order by object_id',list(content_packs)))==content_rows
 assert {i for i, in db.execute('select object_id from objects')}==set(inv)|set(pool)|{r[0] for r in content_rows}
 pages={n:dict(bytes=b,payload=p,unused=u) for n,b,p,u in db.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')}
 db.close();source.close()
 from reader import StoreReader
 reader=StoreReader(HERE/'candidate.sqlite');assert len(reader.ordinals)==len(pool)
 for i,b in pool.items():assert reader.read_canonical(i)==b
 for i,b in inv.items():assert reader.read_canonical(i)==b
 physical=set()
 for p,b in reader.db.execute('select pack_id,data from object_packs'):
  if int.from_bytes(b[8:12],'little')==1:
   _,gs=old.old.groups(b)
   physical.update((p,g,k) for g,rs in enumerate(gs) for k in range(len(rs)))
 assert physical=={(p,g,k) for i,n,p,g,k in locators+poolloc for p in [p+offset]}
 for root in roots:
  table=e.codec.value(inv[root])[84:116]
  got=e.D.decode_table(reader,table);expected=e.D.decode_table(inv,table);assert got==expected
  for _,v in got:
   if v[0]==2:assert e.D.decode_directory(reader,v[9:41])==e.D.decode_directory(inv,v[9:41])
 work=dict(reader.metrics);reader.close();assert sha(SOURCE)==EXPECTED
 ctl=(HERE/'control.sqlite').stat();st=(HERE/'candidate.sqlite').stat()
 result=dict(status='PASS',decision='PROMISING' if st.st_size<ctl.st_size else 'REJECT',source_sha256=EXPECTED,control_bytes=ctl.st_size,control_allocated_bytes=ctl.st_blocks*512,candidate_bytes=st.st_size,candidate_allocated_bytes=st.st_blocks*512,saving_bytes=ctl.st_size-st.st_size,metadata_packs_bytes=sum(len(b) for p,b in packs),pool_packs_bytes=sum(len(b) for p,b in poolpacks),pool_objects=len(pool),original_metadata_objects=len(inv),inline_value_occurrences=sum((len(b)-44)//81 for b in inv.values() if e.leaf(b)),original_metadata_pack_bytes=17459061,selected_delta=sum(bool(r[0]) for r in records.values()),maximum_closure=max(closure(i,records)[1] for i in inv),maximum_depth=max(closure(i,records)[0] for i in inv),maximum_pool_canonical_work=max(closure(i,records)[2] for i in inv),pages=pages,original_metadata_and_pool_canonical_equality='PASS',physical_metadata_and_pool_membership='PASS',namespace_semantics_verified=len(roots),unchanged_content_packs_and_locators='PASS',unchanged_typed_SQL_history='PASS',reader_metrics=work,counters=dict(counters),candidate_sha256=sha(HERE/'candidate.sqlite'),elapsed_seconds=time.monotonic()-start,protocol_sha256=sha(HERE/'protocol.md'))
 (HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2),flush=True)
if __name__=='__main__':main()

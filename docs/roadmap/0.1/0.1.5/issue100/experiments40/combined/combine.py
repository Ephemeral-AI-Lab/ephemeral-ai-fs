"""Two fixed unsupported complete-copy layouts; semantic/identity checks, no product writes."""
import collections, hashlib, importlib.util,json,pathlib,sqlite3,struct,sys,time
HERE=pathlib.Path(__file__).resolve().parent;EXP=HERE.parent
sys.path[:0]=[str(EXP/'metadata'),str(EXP/'tools')]
import d_api
from hashing import blake3
spec=importlib.util.spec_from_file_location('framing_diag',EXP/'framing'/'experiment.py');fr=importlib.util.module_from_spec(spec);spec.loader.exec_module(fr)
sha=fr.sha;check=fr.check;SRC=fr.SRC;EXPECTED=fr.EXPECTED

def optional(b):return b'\x00' if b is None else b'\x01'+b
def layer_id(stack,parent,root):return b'\x32'+blake3(b'layerfs/layer/v2\0'+stack+optional(parent)+root)
def commit_id(root,parent,layer):return b'\x12'+blake3(b'layerfs/commit/v2\0'+root+optional(parent)+layer)
def derive_sql(source,rootmap):
 tables={}
 for name in ('layers','commits','branches','layer_stacks','workspace_stages'):
  cur=source.execute('select * from '+name);columns=[x[0] for x in cur.description];tables[name]=[dict(zip(columns,row)) for row in cur]
 check(len(tables['layers'])==1 and len(tables['commits'])==10 and not tables['workspace_stages'],'fixture SQL shape')
 layer=tables['layers'][0]
 check(layer['parent_layer_id'] is None and layer['source_commit_id'] is None and layer['source_branch_id'] is None,'genesis')
 check(layer['layer_id']==layer_id(layer['layer_stack_id'],None,layer['root_id']),'original layer identity')
 lm={layer['layer_id']:layer_id(layer['layer_stack_id'],None,rootmap[layer['root_id']])};cm={};pending=tables['commits'][:]
 while pending:
  ready=[r for r in pending if r['parent_commit_id'] is None or r['parent_commit_id'] in cm]
  check(ready,'commit DAG')
  for r in ready:
   check(r['commit_id']==commit_id(r['root_id'],r['parent_commit_id'],r['base_layer_id']),'original commit identity')
   cm[r['commit_id']]=commit_id(rootmap[r['root_id']],cm.get(r['parent_commit_id']),lm[r['base_layer_id']]);pending.remove(r)
 check(len(set(cm.values()))==10,'distinct commits')
 mappings={'root_id':rootmap,'layer_id':lm,'parent_layer_id':lm,'head_layer_id':lm,'base_layer_id':lm,'commit_id':cm,'parent_commit_id':cm,'head_commit_id':cm,'source_commit_id':cm}
 rewritten={name:[{k:(mappings[k][v] if k in mappings and v is not None else v) for k,v in row.items()} for row in rows] for name,rows in tables.items()}
 return tables,rewritten,lm,cm

def groups(b):
 check(b[:8]==b'LFPACK\0\0' and len(b)<=262144,'pack framing');version,n=struct.unpack_from('<II',b,8);check(version in (1,2) and 1<=n<=256,'pack kind/count')
 p=16+16*n;out=[]
 for g in range(n):
  pos,encoded,decoded,codec=struct.unpack_from('<IIII',b,16+16*g);check(pos==p and p+encoded<=len(b),'group bounds');p+=encoded
  body=d_api.base.decompress(b[pos:p],decoded) if codec else b[pos:p]
  check(len(body)==decoded and len(body)<=65536,'group decoded bound');check(version!=2 or codec==0,'native codec')
  count=fr.u32(body,0);check(1<=count<=8191,'record count');start=4+4*count;last=start;records=[]
  for i in range(count):
   end=start+fr.u32(body,4+4*i);check(last<end<=len(body),'record ends');records.append(body[last:end]);last=end
  check(last==len(body),'group trailing');out.append(records)
 check(p==len(b),'pack trailing');return version,out

def native_record(b):
 check(len(b)>=6 and b[0] in (0,1),'native kind');kind=b[0];raw=fr.u32(b,1);check(0<=raw<=32768,'native raw')
 start=37 if kind else 5;frame=b[start:];check(1<=len(frame)<=33024,'native frame')
 return kind,raw,b[5:37] if kind else None,frame

def rebuild_native(b,p,byloc,selected):
 v,gs=groups(b);check(v==2,'native pack')
 bodies=[]
 for g,rs in enumerate(gs):
  new=[]
  for k,r in enumerate(rs):
   identity,n=byloc[p,g,k]
   if identity.hex() in selected:
    s=selected[identity.hex()];check((s['pack'],s['group'],s['ordinal'])==(p,g,k),'selected locator')
    check(s['raw']+21==n and len(rs)==s['group_count'],'selected canonical')
    frame=bytes.fromhex(s['frame']);r=bytes([s['kind']])+struct.pack('<I',s['raw'])+(bytes.fromhex(s['base']) if s['base'] else b'')+frame
    check(len(r)==s['size'],'selected size')
   native_record(r);new.append(r)
  ends=[];end=0
  for r in new:end+=len(r);ends.append(end)
  body=struct.pack('<I',len(new))+struct.pack('<'+'I'*len(ends),*ends)+b''.join(new);check(len(body)<=65536,'native group limit');bodies.append(body)
 pos=16+16*len(bodies);entries=[]
 for b in bodies:entries.append(struct.pack('<IIII',pos,len(b),len(b),0));pos+=len(b)
 result=b'LFPACK\0\0'+struct.pack('<II',2,len(bodies))+b''.join(entries)+b''.join(bodies)
 check(len(result)<=262144,'native pack limit');return result

def auth_native(records,loc,z):
 maxima=[0]*4
 for target in records:
  nodes=[];seen=set();node=target
  while True:
   check(node not in seen and node in records,'native cycle/missing');seen.add(node)
   kind,raw,base,frame=records[node];nodes.append(node)
   if base is None:break
   check(base in loc and loc[base][0]<loc[node][0],'native chronology');node=base
  check(len(nodes)<=5,'native depth');rawclosure=sum(records[n][1] for n in nodes)
  encoded=sum(32+4+4*loc[n][5]+loc[n][4] for n in nodes);decoded=encoded+sum(records[n][1]+21 for n in nodes)
  check(rawclosure<=1048576 and encoded<=393216 and decoded<=524288,'native closure budget')
  basebytes=None
  for n in reversed(nodes):
   kind,raw,base,frame=records[n];basebytes,workspace=fr.decode(z,frame,raw,basebytes)
   canonical=b'LFSO\x01'+struct.pack('>II',raw+12,raw+8)+b'LFS4CHK\0'+basebytes
   check(blake3(b'layerfs/object/v2\0'+canonical)==n and len(canonical)==loc[n][3],'native authentication')
  maxima=[max(a,b) for a,b in zip(maxima,(len(nodes)-1,rawclosure,encoded,decoded))]
 return dict(objects=len(records),max_depth=maxima[0],max_raw_closure=maxima[1],max_encoded_work=maxima[2],max_decoded_work=maxima[3])

def verify_copy(copy,inventory,newroots,small_verified,z):
 rows=list(copy.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects'))
 byloc={(p,g,r):(i,n) for i,n,p,g,r in rows};check(len(rows)==len(byloc),'unique locators')
 seen=set();small={};sloc={};native={};nloc={};meta={};small_hash=hashlib.sha256();categories=collections.Counter()
 for p,b in copy.execute('select pack_id,data from object_packs order by pack_id'):
  if b[:8]==fr.MAGIC:
   n=fr.u32(b,12);lengths=[byloc[p,g,0][1] for g in range(n)];rs=fr.unpack(b,102,lengths);categories['small_pack_bytes']+=len(b)
   small_hash.update(p.to_bytes(8,'little')+b)
   for g,r in enumerate(rs):
    i,n=byloc[p,g,0];seen.add((p,g,0));small[i]=r;sloc[i]=(p,g,0,n)
  else:
   v,gs=groups(b);categories['metadata_pack_bytes' if v==1 else 'native_pack_bytes']+=len(b)
   for g,rs in enumerate(gs):
    for k,r in enumerate(rs):
     i,n=byloc[p,g,k];seen.add((p,g,k))
     if v==1:
      check(r[0]==0,'D metadata FULL');canonical=r[1:];check(len(canonical)==n and d_api.base.oid(canonical)==i,'D physical authentication');meta[i]=canonical
     else:
      parsed=native_record(r);check(n==parsed[1]+21,'native length');native[i]=parsed;nloc[i]=(p,g,k,n,len(r),len(rs))
 check(seen==set(byloc),'physical locator membership');check(meta==inventory,'exact D inventory');check(len(small)==33217 and len(native)==735,'content counts')
 for root in newroots:
  ns=d_api.base.value(meta[root]);check(ns[:8]==b'LFS6FSR\0','D namespace');table=d_api.d.decode_table(meta,ns[84:116]);keys={k for k,v in table};check(ns[76:84] in keys,'root inode')
  for k,v in table:
   check(v[41:73] in meta,'inode metadata reference')
   if v[0]==2:
    for name,key in d_api.d.decode_directory(meta,v[9:41]):check(key in keys,'directory inode reference')
   else:check(v[9:41] in seen_ids,'inode content reference')
 digest=small_hash.hexdigest()
 if small_verified is None:
  maximum=[0]*4
  for i in small:
   facts=fr.authenticate(i,small,sloc,z,blake3);maximum=[max(a,b) for a,b in zip(maximum,facts)]
  smallcheck=dict(objects=len(small),pack_graph_sha256=digest,max_depth=maximum[0],max_canonical_closure=maximum[1],max_encoded_closure=maximum[2],max_decoder_dictionary_workspace=maximum[3],verification='all targets/dependencies authenticated')
 else:
  check(digest==small_verified['pack_graph_sha256'],'same authenticated small graph');smallcheck={**small_verified,'verification':'exact same pack bytes/locators as fully authenticated first treatment'}
 return dict(categories),smallcheck,auth_native(native,nloc,z)

start=time.monotonic();check(sha(SRC)==EXPECTED,'source identity')
baselines=json.loads((EXP/'framing'/'layout-result.json').read_text());baseline=baselines['copies']['3'];check(sha(baseline['path'])==baseline['sha256'],'baseline reuse hash')
source=sqlite3.connect(SRC.as_uri()+'?mode=ro&immutable=1',uri=True)
# Only expected SQL foreign-key references to object identities are allowed.
refs=set()
for name, in source.execute("select name from sqlite_schema where type='table'"):
 for r in source.execute('pragma foreign_key_list('+name+')'):
  if r[2]=='objects':refs.add((name,r[3],r[4]))
check(refs=={(t,'root_id','object_id') for t in ('commits','layers','workspace_stages')},'unexpected object references')
rootchecks=json.loads((EXP/'metadata'/'roots-D.json').read_text());rootmap={bytes.fromhex(r['original_root']):bytes.fromhex(r['new_root']) for r in rootchecks}
inventory,newroots,content,scope,highwater=d_api.original_inventory();check(set(newroots)==set(rootmap.values()),'D root mapping')
original,rewritten,lm,cm=derive_sql(source,rootmap)
maxpack=source.execute('select max(pack_id) from object_packs').fetchone()[0];packs,locators=d_api.pack_blobs(inventory,maxpack)
dresult=json.loads((EXP/'metadata'/'result-D.json').read_text());check(len(inventory)==4900 and len(packs)==25 and sum(len(b) for p,b in packs)==2639978,'D inventory totals')
check([hashlib.sha256(b).hexdigest() for p,b in packs]==dresult['pack_sha256'],'D pack reproducibility')
source_rows=list(source.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects'));byloc={(p,g,r):(i,n) for i,n,p,g,r in source_rows}
oldmeta=[p for p,b in source.execute('select pack_id,data from object_packs') if fr.u32(b,8)==1]
selected=json.loads((EXP/'cdc'/'family-result.json').read_text())['selected_graph'];check(len(selected)==249,'family graph')
all_native=json.loads((EXP/'cdc'/'all-native-verification.json').read_text())
seen_ids=set(inventory)|{r[0] for r in content};z=fr.setup_decoder();small_verified=None;results={}
for treatment,cdc,version in [('D-B',False,9201),('D-B-CDC',True,9202)]:
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
 check(categories['metadata_pack_bytes']==2639978 and categories['small_pack_bytes']==33955655 and categories['native_pack_bytes']==(2707597 if cdc else 3259492),'exact categories')
 copy.execute('vacuum');check(copy.execute('pragma integrity_check').fetchone()==('ok',),'SQLite integrity')
 pages={name:dict(bytes=size,payload=payload,unused=unused) for name,size,payload,unused in copy.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')}
 check(copy.execute('select scope,highwater from scope_allocator').fetchone()==(scope,17922),'allocator')
 count=copy.execute('select count(*) from objects').fetchone()[0];packcount=copy.execute('select count(*) from object_packs').fetchone()[0];copy.close();st=path.stat();logical=st.st_size;allocated=st.st_blocks*512
 results[treatment]=dict(path=str(path),sha256=sha(path),objects=count,packs=packcount,logical_bytes=logical,copy_allocated_bytes=allocated,pack_categories=categories,all_pack_bytes=sum(categories.values()),sqlite_nonpack_bytes=logical-sum(categories.values()),allocation_difference=allocated-logical,saving_vs_equal_vacuumed_baseline=baseline['logical_bytes']-logical,pages=pages,small_verification=smallcheck,native_verification=nativecheck,sql_foreign_keys='PASS',sql_integrity='PASS',all_physical_locator_membership='PASS',metadata_inventory_and_namespace_checks='PASS')
 print(treatment,logical,allocated,flush=True)
source.close();check(sha(SRC)==EXPECTED,'source unchanged')
mapping={'namespace_roots':{k.hex():v.hex() for k,v in rootmap.items()},'layers':{k.hex():v.hex() for k,v in lm.items()},'commits':{k.hex():v.hex() for k,v in cm.items()}}
(HERE/'identity-map.json').write_text(json.dumps(mapping,indent=2)+'\n')
result=dict(scope='Combined unsupported offline complete-copy layout. Canonical metadata and typed history IDs change; content IDs and snapshot semantics retained. No product open/rollback/history API or public performance qualification.',source_sha256_before_and_after=EXPECTED,baseline=baseline,protocol_sha256=sha(HERE/'protocol.md'),script_sha256=sha(__file__),results=results,semantic_namespace_states=11,original_typed_identites_verified=True,typed_identity_cascade=True,scope_hex=scope.hex(),allocator_highwater=highwater,elapsed_seconds=time.monotonic()-start,input_hashes={str(p.relative_to(EXP)):sha(p) for p in [EXP/'metadata'/'d_api.py',EXP/'metadata'/'compact_ids.py',EXP/'metadata'/'run.py',EXP/'metadata'/'roots-D.json',EXP/'cdc'/'family-result.json',EXP/'cdc'/'all-native-verification.json',EXP/'framing'/'experiment.py']})
(HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='results'},indent=2))

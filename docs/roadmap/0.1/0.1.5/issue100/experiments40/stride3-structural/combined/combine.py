"""Same structural policies assembled directly into a fresh53-state database."""
import hashlib,json,shutil,sqlite3,struct,time
from pathlib import Path
from store_api import StoreReader,chain_api,old,MAGIC
check=old.check;layer_id=old.layer_id;commit_id=old.commit_id
ROOT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural');HERE=ROOT/'combined'
SOURCE=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-comparison/layerfs/deepseek-stride3/host-runtime/store.sqlite')
EXPECTED='5c6ee04eee133539f043ee64d242c26769c77d30b434af2523666e326a8d999e'
def sha(path):
 with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def derive_sql(source,rootmap):
 tables={}
 for name in ('layers','commits','branches','layer_stacks','workspace_stages'):
  cur=source.execute('select * from '+name);columns=[x[0] for x in cur.description];tables[name]=[dict(zip(columns,row)) for row in cur]
 check(len(tables['layers'])==1 and len(tables['commits'])==53 and not tables['workspace_stages'],'fixture SQL shape')
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
 check(len(set(cm.values()))==53,'distinct commits')
 mappings={'root_id':rootmap,'layer_id':lm,'parent_layer_id':lm,'head_layer_id':lm,'base_layer_id':lm,'commit_id':cm,'parent_commit_id':cm,'head_commit_id':cm,'source_commit_id':cm}
 rewritten={name:[{k:(mappings[k][v] if k in mappings and v is not None else v) for k,v in row.items()} for row in rows] for name,rows in tables.items()}
 return tables,rewritten,lm,cm

def small_packs(path, start):
    db = sqlite3.connect(path.as_uri()+'?mode=ro&immutable=1',uri=True)
    rows = list(db.execute('select oid,id,base,raw,frame,depth,canonical,encoded from records order by depth,oid'))
    db.close()
    ids = {oid:identity for oid,identity,*_ in rows}
    assert len(rows) == len(ids) == 59768 and all(ids.values())
    packs=[]; locators=[]; pending=[]; size=16
    def flush():
        nonlocal pending,size
        if not pending: return
        pack = start+len(packs)+1
        offset=16+4*len(pending); starts=[]; records=[]
        for group,(identity,n,record) in enumerate(pending):
            starts.append(offset); offset+=len(record); records.append(record)
            locators.append((identity,n,pack,group,0))
        blob=MAGIC+struct.pack('<II',103,len(pending))+struct.pack('<'+'I'*len(starts),*starts)+b''.join(records)
        assert len(blob)==offset==size and len(blob)<=262144
        packs.append((pack,blob)); pending=[]; size=16
    for oid,identity,base,raw,frame,depth,canonical,encoded in rows:
        assert 0 < raw < 131072 and depth<=50 and canonical<=67108864 and encoded<=67108864
        record=bytes([bool(base)])+(ids[base] if base else b'')+frame
        if pending and (len(pending)==256 or size+4+len(record)>262144): flush()
        assert 16+4+len(record)<=262144
        pending.append((identity,raw+23,record)); size+=4+len(record)
    flush()
    return packs,locators

def negative_checks(reader, inv):
    target=next(i for i in inv if reader._physical(i)[1][0])
    _,record,_=reader._physical(target); base=record[1:33]
    fixtures=[]
    def reject(name, change):
        probe=StoreReader(reader.path,cache_bytes=0,pack_cache_bytes=262144)
        try:
            change(probe)
            try: probe.read_canonical(target)
            except (AssertionError,ValueError,KeyError,struct.error): fixtures.append(name)
            else: raise AssertionError('accepted '+name)
        finally: probe.close()
    reject('missing_metadata_base',lambda p:p.loc.pop(base))
    def future(p):
        oldloc=p.loc[base]; p.loc[base]=(p.loc[target][0]+100000,*oldloc[1:])
    reject('future_metadata_base',future)
    def corrupt(p):
        physical=p._physical
        def altered(i):
            v,r,n=physical(i)
            if i==target: r=r[:-1]+bytes([r[-1]^1])
            return v,r,n
        p._physical=altered
    reject('corrupt_metadata_delta',corrupt)
    return fixtures

def main():
    started=time.monotonic();assert sha(SOURCE)==EXPECTED
    source=sqlite3.connect(SOURCE.as_uri()+'?mode=ro&immutable=1',uri=True)
    source_rows=list(source.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects order by object_id'))
    source_packs=dict(source.execute('select pack_id,data from object_packs'))
    versions={p:struct.unpack_from('<I',b,8)[0] for p,b in source_packs.items()}
    byloc={(p,g,r):(i,n) for i,n,p,g,r in source_rows};assert len(byloc)==len(source_rows)==145769
    control=HERE/'control.sqlite'
    with control.open('xb'):pass
    db=sqlite3.connect(control);source.backup(db);db.execute('vacuum')
    assert db.execute('pragma integrity_check').fetchone()==('ok',);db.close()
    inv,roots,content,scope,highwater=chain_api.original_inventory();rootmap=chain_api.root_map()
    assert len(roots)==len(rootmap)==54 and len(inv)==10542 and set(roots)==set(rootmap.values())
    assert sorted(content)==sorted(r for r in source_rows if versions[r[2]]!=1)
    original,rewritten,lm,cm=derive_sql(source,rootmap)
    meta_packs,meta_loc=chain_api.pack_blobs(inv,max(source_packs))
    content_file=ROOT/'content/git-small.sqlite';proof=json.loads(content_file.with_suffix('.json').read_text())
    assert proof['verified']==59768 and sha(content_file)==proof['artifact_sha256']
    small_blobs,small_loc=small_packs(content_file,max(p for p,b in meta_packs))
    small_ids={r[0] for r in small_loc};assert small_ids=={i for i,n,p,g,r in source_rows if versions[p]==3}
    family=json.loads((ROOT/'cdc/family-result.json').read_text());selected=family['selected_graph']
    native_blobs=[(p,old.rebuild_native(b,p,byloc,selected)) for p,b in source_packs.items() if versions[p]==2]
    replaced={p for p,v in versions.items() if v in (1,3)}
    path=HERE/'candidate.sqlite'
    with path.open('xb'):pass
    db=sqlite3.connect(path);source.backup(db);db.execute('pragma foreign_keys=on');db.execute('begin');db.execute('pragma defer_foreign_keys=on')
    db.executemany('delete from objects where pack_id=?',[(p,) for p in replaced]);db.executemany('delete from object_packs where pack_id=?',[(p,) for p in replaced])
    db.executemany('insert into object_packs values(?,?)',meta_packs+small_blobs)
    db.executemany('insert into objects values(?,?,?,?,?)',meta_loc+small_loc)
    db.executemany('update object_packs set data=? where pack_id=?',[(b,p) for p,b in native_blobs])
    for table in ('layers','commits','branches','layer_stacks','workspace_stages'):
        pk={'layers':'layer_id','commits':'commit_id','branches':'branch_id','layer_stacks':'layer_stack_id','workspace_stages':'workspace_id'}[table]
        for previous,new in zip(original[table],rewritten[table]):
            columns=list(new);db.execute('update '+table+' set '+','.join(k+'=?' for k in columns)+' where '+pk+'=?',[new[k] for k in columns]+[previous[pk]])
    db.execute('CREATE TABLE scope_allocator(scope BLOB PRIMARY KEY CHECK(length(scope)=32),highwater INTEGER NOT NULL CHECK(highwater>=0)) STRICT, WITHOUT ROWID')
    db.execute('insert into scope_allocator values(?,?)',(scope,highwater));db.execute('pragma user_version=9302');db.commit()
    assert not list(db.execute('pragma foreign_key_check'))
    for table,expected in rewritten.items():
        cur=db.execute('select * from '+table);cols=[d[0] for d in cur.description]
        assert sorted(map(repr,[dict(zip(cols,row)) for row in cur]))==sorted(map(repr,expected))
    rows=list(db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects order by object_id'))
    assert {i for i,n,p,g,r in rows}==set(inv)|{r[0] for r in content}
    assert sorted((i,n) for i,n,p,g,r in rows if i not in inv)==sorted((r[0],r[1]) for r in content)
    assert [r for r in rows if versions.get(r[2])==2]==[r for r in source_rows if versions[r[2]]==2]
    actual_locs={(p,g,r):i for i,n,p,g,r in rows};assert len(actual_locs)==len(rows)
    seen=set();categories=dict(metadata=0,small=0,native=0)
    for p,b in db.execute('select pack_id,data from object_packs'):
        version=struct.unpack_from('<I',b,8)[0];categories[{1:'metadata',2:'native',103:'small'}[version]]+=len(b)
        if version==103:
            for g in range(struct.unpack_from('<I',b,12)[0]):seen.add((p,g,0))
        else:
            _,groups=old.groups(b)
            for g,records in enumerate(groups):
                for r in range(len(records)):seen.add((p,g,r))
    assert seen==set(actual_locs)
    db.execute('vacuum');assert db.execute('pragma integrity_check').fetchone()==('ok',) and not list(db.execute('pragma foreign_key_check'))
    pages={n:dict(bytes=b,payload=p,unused=u) for n,b,p,u in db.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')};db.close();source.close()
    reader=StoreReader(path)
    for identity,canonical in inv.items():assert reader.read_canonical(identity)==canonical
    for i,n,p,g,r in rows:
        if i not in inv:reader.read_canonical(i)
    tests=negative_checks(reader,inv);reader.close()
    assert sha(SOURCE)==EXPECTED
    mapping=dict(commits={k.hex():v.hex() for k,v in cm.items()},layers={k.hex():v.hex() for k,v in lm.items()},roots={k.hex():v.hex() for k,v in rootmap.items()})
    (HERE/'identity-map.json').write_text(json.dumps(mapping,indent=2)+'\n')
    st=path.stat();ctl=control.stat()
    candidate=dict(path=str(path),sha256=sha(path),logical_bytes=st.st_size,allocated_bytes=st.st_blocks*512,pack_bytes=categories,sqlite_nonpack=st.st_size-sum(categories.values()),pages=pages,objects=len(rows),packs=len(meta_packs)+len(small_blobs)+len(native_blobs),metadata_authenticated=len(inv),small_authenticated=len(small_ids),native_authenticated=1998,physical_membership='PASS',sql_integrity_and_foreign_keys='PASS',typed_SQL_identity_rewrite='PASS',content_IDs_lengths_unchanged=True,negative_checks=tests)
    result=dict(status='PASS',scope='Same full157 structural policies on actual53state source; offline format only',source=str(SOURCE),source_sha256_before_and_after=EXPECTED,source_logical_bytes=SOURCE.stat().st_size,source_allocated_bytes=SOURCE.stat().st_blocks*512,matched_control=dict(path=str(control),logical_bytes=ctl.st_size,allocated_bytes=ctl.st_blocks*512,sha256=sha(control)),results={'delta':candidate},saved_vs_matched_control=ctl.st_blocks*512-st.st_blocks*512,recorded_git_allocated_bytes=49332224,gap_to_recorded_git=st.st_blocks*512-49332224,metadata_cache_sha256=sha(ROOT/'metadata/cache.sqlite'),content_artifact_sha256=sha(content_file),cdc_result_sha256=sha(ROOT/'cdc/family-result.json'),reader_sha256=sha(HERE/'store_api.py'),script_sha256=sha(Path(__file__)),elapsed_seconds=time.monotonic()-started)
    (HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':main()

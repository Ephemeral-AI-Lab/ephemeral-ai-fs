"""Exact fixedD on all158 original namespace roots; append-only output artifacts."""
import collections, hashlib, json, pathlib, sqlite3, time
import codec, scoped, pack_api
HERE=pathlib.Path(__file__).resolve().parent
TEN=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/metadata')
def main():
    started=time.monotonic();sealed={}
    for filename in ('manifest.json','manifest-D.json'):
        manifest=json.loads((TEN/filename).read_text())
        for name,digest in manifest.items():assert codec.sha(TEN/name)==digest;sealed[name]=digest
    source_manifest=codec.STORE.parent.parent.parent/'verification-manifest.json'
    assert json.loads(source_manifest.read_text())['deepseek-full/host-runtime/store.sqlite']==codec.SHA
    for name in ('cache.sqlite','D-index.sqlite','result.json','roots.json','inode-map.json'):
        assert not (HERE/name).exists(),('refuse to overwrite previous attempt',name)
    fixture=scoped.fixture();raw,rows,roots,original=codec.load()
    print('PASS authenticated original metadata + exactcodec',len(raw),'objects',original,flush=True)
    inventory,serial,allocator,scope,checks,dir_cache=scoped.build_inventory(raw,roots)
    assert len(checks)==158
    print('PASS all158namespace semantics + alldirectories',len(dir_cache),'uniqueinodes',len(serial),flush=True)
    content=[row for row in rows if row[0] not in raw];assert len(content)==78619
    packs,locators=pack_api.pack_blobs(inventory,max(r[2] for r in rows))
    print('EncodedD',len(inventory),'metadataobjects',len(packs),'packs',sum(len(b) for p,b in packs),'bytes',flush=True)
    index=codec.project_index('D',content+locators);db=sqlite3.connect(HERE/'D-index.sqlite')
    db.execute('CREATE TABLE scope_allocator(scope BLOB PRIMARY KEY CHECK(length(scope)=32),highwater INTEGER NOT NULL CHECK(highwater>=0)) STRICT, WITHOUT ROWID')
    db.execute('insert into scope_allocator values(?,?)',(scope,allocator.highwater));db.commit();db.execute('vacuum')
    pages={name:dict(pages=n,bytes=size,payload=payload,unused=unused) for name,n,size,payload,unused in db.execute('select name,count(*),sum(pgsize),sum(payload),sum(unused) from dbstat group by name')}
    assert db.execute('pragma integrity_check').fetchone()[0]=='ok';assert db.execute('select scope,highwater from scope_allocator').fetchone()==(scope,allocator.highwater)
    db.close();index.update(index_bytes=pages['objects']['bytes'],allocator=pages['scope_allocator'],pages=pages,database_bytes=(HERE/'D-index.sqlite').stat().st_size,sha256=codec.sha(HERE/'D-index.sqlite'))
    cache=sqlite3.connect(HERE/'cache.sqlite')
    cache.execute('create table metadata(id blob primary key,canonical blob not null) without rowid')
    cache.execute('create table packs(pack_id integer primary key,data blob not null)')
    cache.execute('create table locators(id blob primary key,canonical_length integer,pack_id integer,group_number integer,record_number integer) without rowid')
    cache.execute('create table content_locators(id blob primary key,canonical_length integer,pack_id integer,group_number integer,record_number integer) without rowid')
    cache.executemany('insert into metadata values(?,?)',sorted(inventory.items()))
    cache.executemany('insert into packs values(?,?)',packs);cache.executemany('insert into locators values(?,?,?,?,?)',sorted(locators));cache.executemany('insert into content_locators values(?,?,?,?,?)',sorted(content))
    cache.commit();assert cache.execute('pragma integrity_check').fetchone()[0]=='ok';cache.close()
    (HERE/'roots.json').write_text(json.dumps(checks,indent=2)+'\n')
    (HERE/'inode-map.json').write_text(json.dumps({'scope':scope.hex(),'highwater':allocator.highwater,'scope_policy':'singleorigin freshformat; immutable serial by firstappearance','original_to_serial':{k.hex():v.hex() for k,v in sorted(serial.items())}},indent=2)+'\n')
    original_db=sqlite3.connect(codec.STORE.as_uri()+'?mode=ro&immutable=1',uri=True)
    original_index=original_db.execute("select sum(pgsize) from dbstat where name='objects'").fetchone()[0];original_db.close()
    summary=dict(metadata_objects=len(inventory),canonical_bytes=sum(map(len,inventory.values())),max_canonical_bytes=max(map(len,inventory.values())),role_counts=dict(collections.Counter(codec.role(b).rstrip(b'\0').decode() for b in inventory.values())),packs=len(packs),pack_bytes=sum(len(b) for p,b in packs),pack_sha256={str(p):hashlib.sha256(b).hexdigest() for p,b in packs},groups=sum(int.from_bytes(b[12:16],'little') for p,b in packs),new_canonical_objects=sum(i not in raw for i in inventory),removed_original_objects=sum(i not in inventory for i in raw))
    for name,digest in sealed.items():assert codec.sha(TEN/name)==digest
    assert codec.sha(codec.STORE)==codec.SHA
    result=dict(source_store=str(codec.STORE),source_sha256_before=codec.SHA,source_sha256_after=codec.sha(codec.STORE),source_verification_manifest_sha256=codec.sha(source_manifest),original=original,original_canonical_bytes=sum(map(len,raw.values())),original_objects_index_bytes=original_index,original_all_locator_rows=len(rows),index=index,**summary,scope=scope.hex(),allocated_serials=allocator.highwater,verified_states=len(checks),verified_directory_objects=len(dir_cache),content_locator_rows=len(content),fixture=fixture,zstd_library=codec.zpath,zstd_library_sha256=codec.sha(pathlib.Path(codec.zpath)),zstd_version=codec.z.ZSTD_versionNumber(),codec_calls=codec.codec_calls,max_codec_workspace_bytes=codec.max_workspace,elapsed_seconds=time.monotonic()-started,cache_sha256=codec.sha(HERE/'cache.sqlite'),metadata_plus_objectindex_plus_allocator_bytes=summary['pack_bytes']+index['index_bytes']+index['allocator']['bytes'],metadata_pack_difference_vs_original=original['pack_bytes']-summary['pack_bytes'],scope_note='FixedDnewfreshmetadataformat;originalusesselectedlegacydeltas/onlineorder whileDusesfixedFULL-onlyrole/IDsortedpacking. CompleteStoreallocation/latencyunmeasured here.')
    result['artifact_hashes']={name:codec.sha(HERE/name) for name in ('protocol.md','codec.py','scoped.py','pack_api.py','full.py','roots.json','inode-map.json')}
    (HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:result[k] for k in ('metadata_objects','canonical_bytes','pack_bytes','metadata_plus_objectindex_plus_allocator_bytes','metadata_pack_difference_vs_original','verified_states','allocated_serials','elapsed_seconds')},indent=2),flush=True)
if __name__=='__main__':main()

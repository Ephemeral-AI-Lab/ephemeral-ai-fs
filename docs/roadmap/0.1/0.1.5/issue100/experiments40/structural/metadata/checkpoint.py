"""Frozen checkpoint/change metadata reference; see protocol.md."""
import hashlib,json,pathlib,sqlite3,struct,time
import experiment as e
HERE=pathlib.Path(__file__).resolve().parent

def encode(old,new,full):
    removed=[] if full else sorted(old.keys()-new.keys())
    changed=sorted(new) if full else [k for k in sorted(new) if old.get(k)!=new[k]]
    return b'LFSCPT01'+struct.pack('<BII',full,len(removed),len(changed))+b''.join(removed)+b''.join(k+new[k] for k in changed)

def apply(raw,prior):
    assert raw[:8]==b'LFSCPT01' and len(raw)>=17
    full,deleted,changed=struct.unpack_from('<BII',raw,8);assert full in (0,1)
    assert len(raw)==17+8*deleted+81*changed
    out={} if full else dict(prior);p=17
    assert not full or deleted==0
    for _ in range(deleted):key=raw[p:p+8];p+=8;assert key in out;del out[key]
    previous=None
    for _ in range(changed):
        key=raw[p:p+8];value=raw[p+8:p+81];p+=81
        assert previous is None or key>previous;previous=key;out[key]=value
    return out

def digest(table):return hashlib.sha256(b''.join(k+v for k,v in sorted(table.items()))).digest()

def main():
    start=time.monotonic();inv,roots,content,scope,high=e.D.original_inventory()
    path=HERE/'checkpoint.sqlite';assert not path.exists()
    db=sqlite3.connect(path);db.executescript('''
    pragma page_size=4096;
    create table states(step integer primary key, original_root blob unique, original_table blob, checkpoint integer, table_hash blob, decoded_length integer, full integer);
    create table segments(step integer, ordinal integer, codec integer, decoded_length integer, data blob, primary key(step,ordinal)) without rowid;
    create table packs(pack_id integer primary key, data blob);
    create table objects(id blob primary key, canonical_length integer,pack_id integer,group_number integer,record_number integer) without rowid;
    create table allocator(scope blob primary key,highwater integer) without rowid;
    ''')
    retained={i:b for i,b in inv.items() if e.codec.role(b)!=b'LFS6INT\0'}
    groups=e.grouped(retained,sorted(retained,key=lambda i:(e.codec.role(retained[i]),i)))
    pending=[];size=16;pid=0;packbytes=0
    def flush():
        nonlocal pending,size,pid,packbytes
        if not pending:return
        pid+=1;blob=e.assemble(pending,'full');packbytes+=len(blob)
        db.execute('insert into packs values(?,?)',(pid,blob))
        for g,item in enumerate(pending):
            db.executemany('insert into objects values(?,?,?,?,?)',[(i,len(retained[i]),pid,g,k) for k,i in enumerate(item['ids'])])
        pending=[];size=16
    for ids in groups:
        framed=e.frame([b'\0'+retained[i] for i in ids])
        if pending and (size+16+len(framed['body'])>262144 or len(pending)==256):flush()
        pending.append(dict(ids=ids,full=framed));size+=16+len(framed['body'])
    flush();db.execute('insert into allocator values(?,?)',(scope,high))
    prior={};rows=[];checkpoint=0;maximum_work=0;work=0
    for step,root in enumerate(roots):
        tableid=e.codec.value(inv[root])[84:116];table=dict(e.D.decode_table(inv,tableid));full=step%16==0
        if full:checkpoint=step;work=0
        raw=encode(prior,table,full);assert apply(raw,prior)==table;work+=len(raw);maximum_work=max(work,maximum_work)
        db.execute('insert into states values(?,?,?,?,?,?,?)',(step,root,tableid,checkpoint,digest(table),len(raw),full))
        encoded=0
        for ordinal,pos in enumerate(range(0,len(raw),16384)):
            chunk=raw[pos:pos+16384];tag,data=e.codec.compress(chunk);encoded+=len(data)
            db.execute('insert into segments values(?,?,?,?,?)',(step,ordinal,tag,len(chunk),data))
        rows.append(dict(step=step,checkpoint=checkpoint,full=full,inodes=len(table),changed=sum(prior.get(k)!=v for k,v in table.items()),deleted=len(prior.keys()-table.keys()),decoded_bytes=len(raw),encoded_bytes=encoded,reconstruction_decoded_bytes=work))
        prior=table
    db.commit();db.execute('vacuum');assert db.execute('pragma integrity_check').fetchone()[0]=='ok'
    reconstructed={};locations={}
    for i,n,p,g,k in db.execute('select * from objects'):locations.setdefault(p,{})[g,k]=i
    for p,blob in db.execute('select * from packs'):
        for i,record in e.unpack(blob,locations[p]).items():assert record[0]==0;reconstructed[i]=record[1:];assert e.codec.oid(record[1:])==i
    assert reconstructed==retained
    checks=0
    for step,root in enumerate(roots):
        checkpoint=db.execute('select checkpoint from states where step=?',(step,)).fetchone()[0];table={}
        for state in range(checkpoint,step+1):
            raw=b''.join(e.codec.decompress(data,n) if tag else data for ordinal,tag,n,data in db.execute('select ordinal,codec,decoded_length,data from segments where step=? order by ordinal',(state,)))
            table=apply(raw,table)
            assert digest(table)==db.execute('select table_hash from states where step=?',(state,)).fetchone()[0]
        expected=dict(e.D.decode_table(inv,e.codec.value(inv[root])[84:116]));assert table==expected
        for value in table.values():
            if value[0]==2:
                directory=value[9:41];assert e.D.decode_directory(reconstructed,directory)==e.D.decode_directory(inv,directory);checks+=1
    categories={name:dict(bytes=size,payload=payload,unused=unused) for name,size,payload,unused in db.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')}
    total=db.execute('pragma page_count').fetchone()[0]*4096;assert total==path.stat().st_size
    result=dict(metadata_reference_database_bytes=total,allocated_bytes=path.stat().st_blocks*512,retained_non_table_pack_bytes=packbytes,checkpoint_segment_encoded_bytes=sum(x['encoded_bytes'] for x in rows),checkpoint_count=sum(x['full'] for x in rows),state_count=len(roots),retained_objects=len(retained),replaced_table_objects=len(inv)-len(retained),maximum_reconstruction_decoded_bytes=maximum_work,maximum_materialized_table_bytes=max(x['inodes']*81 for x in rows),metadata_pack_baseline_depth1=25624588,difference_vs_depth1_pack_bytes=25624588-total,dbstat=categories,verification=dict(roots=158,exact_inode_tables=True,exact_retained_canonical_objects=True,directory_checks=checks,sqlite_integrity='ok'),elapsed_seconds=time.monotonic()-start,source_manifest_sha256=e.sha(e.SOURCE/'manifest.json'),database_sha256=e.sha(path),scope_note='Reference metadata database includes retained canonical object packs, their full hash index, checkpoint/change segments and state indexes/allocator. Content packs and their object index are excluded. Semantic state binding replaces original table CAS references; no production runtime or random inode lookup. A read materializes a table and replays up to15 changes; source graph branch resolution is not implemented.')
    db.close();(HERE/'checkpoint-result.json').write_text(json.dumps(result,indent=2)+'\n');(HERE/'checkpoint-states.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()

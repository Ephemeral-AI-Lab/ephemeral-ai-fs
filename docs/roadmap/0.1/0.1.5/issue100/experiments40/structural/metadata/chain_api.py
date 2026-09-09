"""Cached matched FULL/depth16 pack export, unchanged Dcanonical inventory/roots."""
import hashlib,json,pathlib,sqlite3
import experiment as e
HERE=pathlib.Path(__file__).resolve().parent

def _cache():
    result=json.loads((HERE/'result.json').read_text());path=HERE/'cache.sqlite';assert e.sha(path)==result['cache_sha256']
    return sqlite3.connect(path.as_uri()+'?mode=ro&immutable=1',uri=True),result

def original_inventory():return e.D.original_inventory()
def root_map():return e.D.root_map()
def inode_oracle():return e.D.inode_oracle()
decode_table=e.D.decode_table
decode_directory=e.D.decode_directory
decode_record=e.decode_record

def pack_blobs(inventory,base_pack,layout='delta'):
    assert layout in ('full','delta')
    db,result=_cache();packs=list(db.execute('select pack_id,data from packs where layout=? order by pack_id',(layout,)))
    loc=list(db.execute('select * from locators order by pack_id,group_number,record_number'));db.close()
    assert len(inv:=inventory)==result['metadata_objects'] and len(loc)==len(inv)
    for i,n,p,g,k in loc:assert i in inv and len(inv[i])==n
    for p,b in packs:assert hashlib.sha256(b).hexdigest()==result['layouts'][layout]['pack_sha256'][str(p)]
    remap={old:base_pack+1+n for n,(old,b) in enumerate(packs)}
    return [(remap[p],b) for p,b in packs],[(i,n,remap[p],g,k) for i,n,p,g,k in loc]

def record_ledger():
    db,result=_cache();rows=list(db.execute('select id,first_step,kind,base_id,record from records order by id'));db.close();return rows

if __name__=='__main__':
    inv,roots,content,scope,high=original_inventory()
    for layout in ('full','delta'):
        packs,locators=pack_blobs(inv,17,layout)
        result=json.loads((HERE/'result.json').read_text())
        assert sum(len(b) for p,b in packs)==result['layouts'][layout]['pack_bytes'] and len(locators)==24748
    records={i:r for i,step,kind,base,r in record_ledger()}
    assert all(decode_record(i,records)==b for i,b in inv.items())
    assert len(root_map())==len(roots)==158
    print('PASS cached FULL/DELTAexports: all pack hashes/locators,all24748exactcanonicalreconstructions,158unchangedroots')

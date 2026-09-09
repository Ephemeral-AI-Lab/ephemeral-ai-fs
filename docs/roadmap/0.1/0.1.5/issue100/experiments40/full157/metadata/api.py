"""Cached full158-D metadata and actualpack API; never replays or re-encodes history."""
import hashlib,json,pathlib,sqlite3
import codec,scoped
HERE=pathlib.Path(__file__).resolve().parent

def _cache():
    result=json.loads((HERE/'result.json').read_text());path=HERE/'cache.sqlite'
    assert codec.sha(path)==result['cache_sha256']
    return sqlite3.connect(path.as_uri()+'?mode=ro&immutable=1',uri=True),result

def original_inventory():
    """Samefive-itemAPI as tenstate: inventory,newroots,contentrows,scope,highwater."""
    db,result=_cache();inventory=dict(db.execute('select id,canonical from metadata'))
    content=list(db.execute('select * from content_locators order by id'));db.close()
    assert len(inventory)==result['metadata_objects'] and all(codec.oid(b)==i for i,b in inventory.items())
    roots=[bytes.fromhex(x['new_root']) for x in json.loads((HERE/'roots.json').read_text())]
    return inventory,roots,content,bytes.fromhex(result['scope']),result['allocated_serials']

def pack_blobs(inventory,base_pack):
    """Return fixedactualpackbytes and locators, rebasingphysicalpacknumbers only."""
    db,result=_cache();packs=list(db.execute('select pack_id,data from packs order by pack_id'))
    loc=list(db.execute('select * from locators order by pack_id,group_number,record_number'));db.close()
    assert len(inventory)==result['metadata_objects'] and len(loc)==len(inventory)
    for i,n,p,g,k in loc:assert i in inventory and len(inventory[i])==n
    for p,b in packs:assert hashlib.sha256(b).hexdigest()==result['pack_sha256'][str(p)]
    remap={old:base_pack+1+n for n,(old,blob) in enumerate(packs)}
    return [(remap[p],b) for p,b in packs],[(i,n,remap[p],g,k) for i,n,p,g,k in loc]

def root_map():return {bytes.fromhex(x['original_root']):bytes.fromhex(x['new_root']) for x in json.loads((HERE/'roots.json').read_text())}
def inode_oracle():
    j=json.loads((HERE/'inode-map.json').read_text());return {bytes.fromhex(k):bytes.fromhex(v) for k,v in j['original_to_serial'].items()}
decode_table=scoped.decode_table
decode_directory=scoped.decode_directory

if __name__=='__main__':
    inv,roots,content,scope,high=original_inventory();packs,loc=pack_blobs(inv,0)
    result=json.loads((HERE/'result.json').read_text())
    assert len(roots)==158 and sum(len(b) for p,b in packs)==result['pack_bytes']
    assert len(inode_oracle())==high and len(root_map())==158
    print('PASS cachedfull158metadataAPI: exactinventory,allpackSHA256hashes,158roots,oracle/highwater')

"""Build a fresh complete database from authenticated compatible components."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sqlite3
import struct
import time
from store_api import StoreReader, chain_api, old, MAGIC
from audit import census, sha, SOURCE, EXPECTED

ROOT = Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural')
HERE = ROOT/'combined'

def small_packs(path, start):
    db = sqlite3.connect(path.as_uri()+'?mode=ro&immutable=1',uri=True)
    rows = list(db.execute('select oid,id,base,raw,frame,depth,canonical,encoded from records order by depth,oid'))
    db.close()
    ids = {oid:identity for oid,identity,*_ in rows}
    assert len(rows) == len(ids) == 75398 and all(ids.values())
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
    parser=argparse.ArgumentParser(); parser.add_argument('--small-mode',required=True,choices=['current','forward-small','reverse-small','git-small']); args=parser.parse_args()
    started=time.monotonic(); assert sha(SOURCE)==EXPECTED
    source=sqlite3.connect(SOURCE.as_uri()+'?mode=ro&immutable=1',uri=True)
    before=census(source)
    source_rows=list(source.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects order by object_id'))
    source_packs=dict(source.execute('select pack_id,data from object_packs'))
    metadata_packs={p for p,b in source_packs.items() if struct.unpack_from('<I',b,8)[0]==1}
    small_old={p for p,b in source_packs.items() if struct.unpack_from('<I',b,8)[0]==102}
    inv,roots,_,_,_=chain_api.original_inventory()
    selected,locators=chain_api.pack_blobs(inv,max(source_packs))
    assert len(inv)==24748 and len(roots)==158
    replace=metadata_packs.copy(); small_ids=set(); component_seals={}
    if args.small_mode!='current':
        path=ROOT/'content'/(args.small_mode+'.sqlite')
        proof=json.loads((path.with_suffix('.json')).read_text())
        assert proof['verified']==75398 and sha(path)==proof['artifact_sha256']
        packs,locations=small_packs(path,max(p for p,b in selected))
        assert sum(len(b) for p,b in packs)<58979700, 'small component must improve complete packs'
        small_ids={i for i,n,p,g,r in locations}
        assert small_ids=={i for i,n,p,g,r in source_rows if p in small_old}
        component_seals[args.small_mode]=dict(path=str(path),sha256=sha(path),proof_sha256=sha(path.with_suffix('.json')))
        selected+=packs; locators+=locations; replace|=small_old
    assert {i for i,n,p,g,r in locators}=={i for i,n,p,g,r in source_rows if p in replace}
    path=HERE/'candidate.sqlite'
    with path.open('xb'): pass
    db=sqlite3.connect(path); source.backup(db)
    db.execute('pragma foreign_keys=on'); db.execute('begin'); db.execute('pragma defer_foreign_keys=on')
    db.executemany('insert into object_packs values(?,?)',selected)
    db.executemany('update objects set pack_id=?,group_number=?,record_number=? where object_id=?',[(p,g,r,i) for i,n,p,g,r in locators])
    db.executemany('delete from object_packs where pack_id=?',[(p,) for p in replace])
    db.execute('pragma user_version=9302'); db.commit(); db.execute('vacuum')
    rows=list(db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects order by object_id'))
    assert [r[:2] for r in rows]==[r[:2] for r in source_rows]
    unchanged={p:b for p,b in source_packs.items() if p not in replace}
    for p,b in unchanged.items(): assert db.execute('select data from object_packs where pack_id=?',(p,)).fetchone()==(b,)
    assert db.execute('pragma integrity_check').fetchone()==('ok',) and not list(db.execute('pragma foreign_key_check'))
    categories=dict(metadata=0,small=0,native=0); membership=set()
    byloc={(p,g,r):i for i,n,p,g,r in rows}; assert len(byloc)==len(rows)
    for p,b in db.execute('select pack_id,data from object_packs'):
        version=struct.unpack_from('<I',b,8)[0]; role={1:'metadata',2:'native',102:'small',103:'small'}[version]
        categories[role]+=len(b)
        if version in (102,103):
            count=struct.unpack_from('<I',b,12)[0]
            for g in range(count): membership.add((p,g,0))
        else:
            _,groups=old.groups(b)
            for g,records in enumerate(groups):
                for r in range(len(records)): membership.add((p,g,r))
    assert membership==set(byloc)
    for name,proof in before['nonphysical_tables'].items():
        table_rows=sorted(db.execute('select * from "'+name.replace('"','""')+'"'),key=repr)
        assert hashlib.sha256(repr(table_rows).encode()).hexdigest()==proof['sha256']
    pages={n:dict(bytes=b,payload=p,unused=u) for n,b,p,u in db.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')}
    db.close(); source.close()
    reader=StoreReader(path)
    for identity,canonical in inv.items(): assert reader.read_canonical(identity)==canonical
    for identity in sorted(small_ids): reader.read_canonical(identity)
    fixtures=negative_checks(reader,inv); reader.close()
    assert sha(SOURCE)==EXPECTED
    identity=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/combined/identity-map.json')
    shutil.copyfile(identity,HERE/'identity-map.json')
    size=path.stat(); total=sum(categories.values())
    candidate=dict(path=str(path),sha256=sha(path),logical_bytes=size.st_size,allocated_bytes=size.st_blocks*512,pack_bytes=categories,sqlite_nonpack=size.st_size-total,pages=pages,objects=len(rows),packs=len(unchanged)+len(selected),metadata_authenticated=len(inv),small_authenticated=len(small_ids),unchanged_content_proof='Source byte-identical packs and locators outside selected replacements',physical_membership='PASS',sql_integrity='PASS',unchanged_canonical_inventory_and_history='PASS',negative_checks=fixtures)
    result=dict(status='PASS',scope='Actual offline rebuilt copy; extended metadata and optional repacked content graph; no public-path performance or operational qualification',source=str(SOURCE),source_sha256=EXPECTED,selected_small_mode=args.small_mode,component_seals=component_seals,metadata_cache_sha256=sha(ROOT/'metadata/cache.sqlite'),results={'delta':candidate},saved_vs_matched_control=98668544-size.st_blocks*512,remaining_gap_to_recorded_git=size.st_blocks*512-56373248,reader_sha256=sha(HERE/'store_api.py'),script_sha256=sha(Path(__file__)),identity_map_sha256=sha(HERE/'identity-map.json'),elapsed_seconds=time.monotonic()-started)
    (HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n'); print(json.dumps(result,indent=2))

if __name__=='__main__': main()

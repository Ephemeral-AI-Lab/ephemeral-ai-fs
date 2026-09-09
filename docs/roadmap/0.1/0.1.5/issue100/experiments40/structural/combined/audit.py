"""Audit a sealed baseline and one freshly compacted matched control."""
import hashlib
import json
from pathlib import Path
import sqlite3
import struct

ROOT = Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-structural/combined')
SOURCE = Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-history-deltas/combined/delta.sqlite')
EXPECTED = '2c9e44a55042ee0f7989dbfa5e7e88b09beb284ce5fbcfa7682f72d940cfd080'

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def census(db):
    categories = dict(metadata=0, small=0, native=0)
    counts = dict.fromkeys(categories, 0)
    pack_hashes = []
    for pack, data in db.execute('select pack_id,data from object_packs order by pack_id'):
        version = struct.unpack_from('<I', data, 8)[0]
        role = {1:'metadata', 2:'native', 102:'small'}[version]
        categories[role] += len(data)
        counts[role] += 1
        pack_hashes.append((pack, hashlib.sha256(data).hexdigest()))
    objects = list(db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects order by object_id'))
    assert len({row[2:] for row in objects}) == len(objects)
    assert db.execute('pragma integrity_check').fetchone() == ('ok',)
    assert not list(db.execute('pragma foreign_key_check'))
    tables = {}
    for name, in db.execute("select name from sqlite_schema where type='table' and name not in ('objects','object_packs') order by name"):
        rows = sorted(db.execute('select * from "'+name.replace('"','""')+'"'), key=repr)
        tables[name] = dict(rows=len(rows), sha256=hashlib.sha256(repr(rows).encode()).hexdigest())
    pages = {name: dict(bytes=size,payload=payload,unused=unused) for name,size,payload,unused in db.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')}
    return dict(pack_bytes=categories,pack_counts=counts,objects=len(objects),canonical_inventory_sha256=hashlib.sha256(b''.join(i+n.to_bytes(8,'little') for i,n,*_ in objects)).hexdigest(),physical_inventory_sha256=hashlib.sha256(repr(objects).encode()).hexdigest(),pack_inventory_sha256=hashlib.sha256(repr(pack_hashes).encode()).hexdigest(),nonphysical_tables=tables,pages=pages)

def main():
    assert sha(SOURCE) == EXPECTED
    source = sqlite3.connect(SOURCE.as_uri()+'?mode=ro&immutable=1',uri=True)
    before = census(source)
    path = ROOT/'control.sqlite'
    with path.open('xb'): pass
    db = sqlite3.connect(path)
    source.backup(db)
    db.execute('vacuum')
    after = census(db)
    for key in before:
        if key != 'pages': assert before[key] == after[key], key
    db.close()
    source.close()
    assert sha(SOURCE) == EXPECTED
    assert before['pack_bytes'] == dict(metadata=25624588,small=58979700,native=7211036)
    assert before['objects'] == 103367
    size = path.stat()
    result = dict(status='PASS',source=str(SOURCE),source_sha256=EXPECTED,source_logical=SOURCE.stat().st_size,source_allocated=SOURCE.stat().st_blocks*512,control=str(path),control_sha256=sha(path),control_logical=size.st_size,control_allocated=size.st_blocks*512,sqlite_nonpack=size.st_size-sum(after['pack_bytes'].values()),source_census=before,control_census=after,script_sha256=sha(Path(__file__)))
    (ROOT/'audit.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if not k.endswith('_census')},indent=2))

if __name__ == '__main__': main()

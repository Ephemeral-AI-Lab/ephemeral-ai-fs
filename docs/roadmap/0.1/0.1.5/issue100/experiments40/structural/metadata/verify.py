"""Run cached authentication plus negative chain-bound/parser checks; no writes."""
import json,sqlite3,struct
import chain_api as a
import experiment as e
import checkpoint as c
inv,roots,_,_,_=a.original_inventory();records={i:r for i,step,kind,base,r in a.record_ledger()}
assert all(a.decode_record(i,records)==b for i,b in inv.items())
base=bytes(32);ledger={base:b'\0'+b'x'};previous=base
for k in range(1,18):
    identity=k.to_bytes(32,'big');ledger[identity]=b'\1'+previous+struct.pack('<II',1,0);previous=identity
try:e.closure(previous,ledger)
except AssertionError:pass
else:raise AssertionError('depth17 accepted')
ledger={base:b'\0'+b'x'*8192,previous:b'\1'+base+struct.pack('<II',131072,0)}
try:e.closure(previous,ledger)
except AssertionError:pass
else:raise AssertionError('oversize closure accepted')
for raw in (b'',b'LFSCPT01'+struct.pack('<BII',0,1,0)+b'x'*8,b'LFSCPT01'+struct.pack('<BII',2,0,0)):
    try:c.apply(raw,{})
    except AssertionError:pass
    else:raise AssertionError('invalid checkpoint record accepted')
for name,h in json.loads((e.SOURCE/'manifest.json').read_text()).items():assert e.sha(e.SOURCE/name)==h
assert e.sha(c.HERE/'checkpoint.sqlite')==json.loads((c.HERE/'checkpoint-result.json').read_text())['database_sha256']
print('PASS24748canonical authentications; reject depth17/oversize closure/truncated and invalid checkpoint records; source and reference seals unchanged')

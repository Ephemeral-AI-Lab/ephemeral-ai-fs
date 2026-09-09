"""Negative dependency checks against the actual assembled reader, no database edits."""
import json
from pathlib import Path
from store_api import StoreReader

ROOT=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/combined')

def main():
    result=json.loads((ROOT/'result.json').read_text());path=Path(result['results']['delta']['path'])
    reader=StoreReader(path,cache_bytes=0)
    target=next(i for i in reader.loc if reader._physical(i)[0]==4 and reader._physical(i)[1][0]==1)
    _,record,_=reader._physical(target);base=record[1:33]
    assert reader.read_canonical(target)
    reader.close();checks=[]
    def reject(name,mutate,expected=None):
        probe=StoreReader(path,cache_bytes=0)
        try:
            mutate(probe)
            try:probe.read_canonical(target)
            except (AssertionError,ValueError,KeyError) as error:
                if expected:assert expected in str(error),(name,str(error))
                checks.append(name)
            else:raise AssertionError('accepted '+name)
        finally:probe.close()
    reject('missing_content_base',lambda p:p.loc.pop(base),'missing base')
    def alter(p,transform):
        original=p._physical
        p._physical=lambda i:transform(i,original(i))
    reject('content_cycle',lambda p:alter(p,lambda i,row:(row[0],row[1][:1]+target+row[1][33:],row[2]) if i==target else row),'dependency cycle')
    reject('wrong_content_base_role',lambda p:alter(p,lambda i,row:(1,row[1],row[2]) if i==base else row),'dependency role')
    reject('corrupt_content_frame',lambda p:alter(p,lambda i,row:(row[0],row[1][:-1]+bytes([row[1][-1]^1]),row[2]) if i==target else row))
    def deep(p):
        nodes=[target]+[(i+1000000).to_bytes(32,'big') for i in range(51)]
        for i in nodes:p.loc[i]=(1,0,0,24)
        positions={i:n for n,i in enumerate(nodes)}
        def physical(i):
            n=positions[i]
            return (4,b'\1'+nodes[n+1]+b'x',1) if n+1<len(nodes) else (4,b'\0x',1)
        p._physical=physical
    reject('51_edge_content_chain',deep,'edge limit')
    (ROOT/'reader-checks.json').write_text(json.dumps(dict(status='PASS',valid_actual_content_target=target.hex(),rejected=checks),indent=2)+'\n')
    print('PASS:',', '.join(checks))

if __name__=='__main__':main()

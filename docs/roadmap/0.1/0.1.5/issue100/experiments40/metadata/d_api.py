"""Callable exact-D original-data inventory and bounded metadata pack assembly."""
import json, pathlib, struct
import compact_ids as d
base=d.base

def original_inventory():
    """Read-only authenticated source; returns inventory, roots, content rows and highwater/scope."""
    raw,rows,roots,gate=base.load()
    inventory,serial,allocator,scope,checks,dirs=d.build_inventory(raw,roots)
    content=[row for row in rows if row[0] not in raw]
    return inventory,[bytes.fromhex(r['new_root']) for r in checks],content,scope,allocator.highwater

def pack_blobs(inventory,base_pack):
    """Return [(pack_id,actual bytes)], full locators; exact ABC/D fixed FULL-only policy."""
    groups=[];pending=[];length=4
    def group(rows):
        end=0;ends=[];area=[]
        for i,b in rows:end+=1+len(b);ends.append(end);area.append(b'\0'+b)
        body=struct.pack('<I',len(rows))+struct.pack('<'+'I'*len(rows),*ends)+b''.join(area)
        assert len(body)<=16384;codec,frame=base.compress(body)
        return [i for i,b in rows],body,codec,frame
    for i,b in sorted(inventory.items(),key=lambda kv:(base.role(kv[1]),kv[0])):
        assert len(b)<=8192
        if pending and length+5+len(b)>16384:groups.append(group(pending));pending=[];length=4
        pending.append((i,b));length+=5+len(b)
    if pending:groups.append(group(pending))
    packs=[];locators=[];pending=[];encoded=decoded=16;records=0
    def flush(gs):
        offset=16+16*len(gs);directory=[]
        for ids,body,codec,frame in gs:directory.append(struct.pack('<IIII',offset,len(frame),len(body),codec));offset+=len(frame)
        blob=b'LFPACK\0\0'+struct.pack('<II',1,len(gs))+b''.join(directory)+b''.join(g[3] for g in gs)
        assert len(blob)==offset and len(blob)<=262144
        return blob
    for ids,body,codec,frame in groups:
        if pending and (encoded+16+len(frame)>262144 or decoded+16+len(body)>262144 or len(pending)==256 or records+len(ids)>8191):
            packs.append((base_pack+len(packs)+1,flush(pending)));pending=[];encoded=decoded=16;records=0
        p=base_pack+len(packs)+1;g=len(pending)
        locators.extend((i,len(inventory[i]),p,g,k) for k,i in enumerate(ids))
        pending.append((ids,body,codec,frame));encoded+=16+len(frame);decoded+=16+len(body);records+=len(ids)
    if pending:packs.append((base_pack+len(packs)+1,flush(pending)))
    assert len(locators)==len(inventory)
    return packs,locators

if __name__=='__main__':
    inventory,roots,content,scope,highwater=original_inventory()
    packs,locators=pack_blobs(inventory,max(r[2] for r in content))
    result=json.loads((pathlib.Path(__file__).parent/'result-D.json').read_text())
    assert len(inventory)==result['metadata_objects'] and sum(len(b) for p,b in packs)==result['pack_bytes']
    assert [base.hashlib.sha256(b).hexdigest() for p,b in packs]==result['pack_sha256']
    assert len(roots)==11 and len(locators)==4900 and highwater==17922
    print('D callable API matches all25 recorded pack SHA256 hashes and complete inventory')

"""Conditional fixed D experiment, with sealed C control. See protocol-compact-ids.md."""
import collections as C, hashlib, json, pathlib, sqlite3, struct
import codec as base
HERE=pathlib.Path(__file__).resolve().parent
b3=base.blake3;oid=base.oid;canonical=base.canonical;value=base.value

def build_table(pairs,inv):
    def put(role,level,rows,total,width):
        v=b'LFS6INT\0'+struct.pack('>HBBB HQQ',1,role,level,0,len(rows),total,total*81)+b''.join(k+r for k,r in rows)
        b=canonical(v);assert len(b)<=8192;key=oid(b);assert key not in inv or inv[key]==b;inv[key]=b;return key
    level=[]
    for chunk in base.partition(pairs,100):
        node=put(7,0,chunk,len(chunk),81);level.append((chunk[-1][0],node,len(chunk)))
    depth=0
    while len(level)>1:
        depth+=1;next_level=[]
        for chunk in base.partition(level,127):
            n=sum(n for k,r,n in chunk);node=put(8,depth,[(k,r) for k,r,n in chunk],n,40);next_level.append((chunk[-1][0],node,n))
        level=next_level
    return level[0][1]
def decode_table(inv,root):
    b=inv[root];assert oid(b)==root and len(b)<=8192;v=value(b);assert v[:8]==b'LFS6INT\0'
    role,level,flags=v[10:13];n=int.from_bytes(v[13:15],'big');width=81 if role==7 else 40
    assert flags==0 and len(v)==31+n*width
    rows=[(v[p:p+8],v[p+8:p+width]) for p in range(31,len(v),width)]
    assert [k for k,r in rows]==sorted(set(k for k,r in rows))
    if role==7:assert level==0 and n<=100;result=rows
    else:
        assert role==8 and 0<level<=31 and n<=127;result=[]
        for maxkey,child in rows:
            assert value(inv[child])[11]+1==level
            child_rows=decode_table(inv,child);assert child_rows[-1][0]==maxkey;result+=child_rows
    assert len(result)==int.from_bytes(v[15:23],'big') and len(result)*81==int.from_bytes(v[23:31],'big')
    return result

def transform_directory(raw,root,serial,inv,cache):
    if root in cache:return cache[root]
    v=value(raw[root]);assert v[:8]==b'LFS4NSP\0';n=int.from_bytes(v[13:15],'big');p=31;rows=[]
    for _ in range(n):
        size=int.from_bytes(v[p:p+2],'big');p+=2;name=v[p:p+size];p+=size;ref=v[p:p+32];p+=32;rows.append((name,ref))
    assert p==len(v)
    if v[10]==1:
        encoded=[(name,serial[ref]) for name,ref in rows];total=sum(10+len(name) for name,r in rows)
    else:
        assert v[10]==2;encoded=[(name,transform_directory(raw,ref,serial,inv,cache)) for name,ref in rows]
        total=sum(int.from_bytes(value(inv[ref])[23:31],'big') for name,ref in encoded)
    header=b'LFS6NSP\0'+v[8:23]+total.to_bytes(8,'big')
    b=canonical(header+b''.join(len(name).to_bytes(2,'big')+name+ref for name,ref in encoded));assert len(b)<=8192
    key=oid(b);assert key not in inv or inv[key]==b;inv[key]=b;cache[root]=key;return key

def decode_directory(inv,root):
    b=inv[root];assert oid(b)==root and len(b)<=8192;v=value(b);assert v[:8]==b'LFS6NSP\0'
    width=8 if v[10]==1 else 32;n=int.from_bytes(v[13:15],'big');p=31;rows=[]
    for _ in range(n):
        size=int.from_bytes(v[p:p+2],'big');p+=2;name=v[p:p+size];p+=size;ref=v[p:p+width];p+=width;rows.append((name,ref))
    assert p==len(v) and rows==sorted(rows)
    if v[10]==1:assert v[11]==0;result=rows
    else:
        assert v[10]==2 and v[11]>0;result=[]
        for name,child in rows:
            assert value(inv[child])[11]+1==v[11]
            result+=decode_directory(inv,child)
    assert len(result)==int.from_bytes(v[15:23],'big') and sum(10+len(name) for name,ref in result)==int.from_bytes(v[23:31],'big')
    return result

class Allocator:
    def __init__(self):self.highwater=0
    def reserve(self):
        assert self.highwater<2**63-1
        self.highwater+=1;return self.highwater.to_bytes(8,'big')

def fixture():
    a=Allocator();ancestor=a.reserve();branchA_new=a.reserve();branchB_new=a.reserve()
    assert len({ancestor,branchA_new,branchB_new})==3
    value73=b'\1'+(2).to_bytes(8,'big')+b3(b'content')+b3(b'metadata')
    inv={};table=build_table([(ancestor,value73)],inv);assert decode_table(inv,table)==[(ancestor,value73)]
    def make(rows):return canonical(b'LFS6NSP\0'+struct.pack('>HBBB HQQ',1,1,0,0,len(rows),len(rows),sum(10+len(n) for n,s in rows))+b''.join(len(n).to_bytes(2,'big')+n+s for n,s in rows))
    before=make([(b'a',ancestor),(b'b',ancestor)]);after=make([(b'b',ancestor),(b'c',ancestor)])
    before_id=oid(before);after_id=oid(after);inv[before_id]=before;inv[after_id]=after
    assert before_id!=after_id and decode_directory(inv,before_id)==[(b'a',ancestor),(b'b',ancestor)]
    assert decode_directory(inv,after_id)==[(b'b',ancestor),(b'c',ancestor)]
    sharedA=dict(decode_table(inv,table));sharedB=dict(decode_table(inv,table));assert sharedA[ancestor]==sharedB[ancestor]
    return dict(status='PASS',highwater=a.highwater,checks=['stable serial across namespace rename','two hardlink names share serial/refcount2','distinct inode allocations differ','shared ancestor retains same inode in branches','subsequent global allocations for separate branches differ'],limit='global allocation primitive model only; no product concurrent allocator/import implementation')

def build_inventory(raw,roots):
    """Return the exact fixed D inventory and verification data; no writes/index creation."""
    allocator=Allocator();serial={}
    for root in roots:
        for inode,record in base.original_pairs(raw,value(raw[root])[76:108]):
            if inode not in serial:serial[inode]=allocator.reserve()
    reverse={v:k for k,v in serial.items()};assert len(serial)==len(reverse) and len(serial)>0
    scope=b3(b'layerfs/diagnostic/scoped-inode-origin/v1\0'+roots[0])
    profile=b3(b'layerfs/diagnostic/scoped-inline/v1\0/serial8/leaf100/branch127/page8192')
    removed={b'LFS4INO\0',b'LFS4INT\0',b'LFS4FSR\0',b'LFS4DIR\0',b'LFS4NSP\0'}
    inventory={i:b for i,b in raw.items() if base.role(b) not in removed};dir_cache={}
    for i,b in raw.items():
        if base.role(b)==b'LFS4NSP\0':transform_directory(raw,i,serial,inventory,dir_cache)
    state_checks=[]
    for root in roots:
        ns=value(raw[root]);original_pairs=base.original_pairs(raw,ns[76:108]);pairs=[]
        for inode,record in original_pairs:
            rec=value(raw[record])[12:85]
            if rec[0]==2:
                state=value(raw[rec[9:41]]);mapping=state[53:85]
                assert state[20]==value(raw[mapping])[11] and state[12:20]==value(raw[mapping])[15:23]
                rec=rec[:9]+dir_cache[mapping]+rec[41:]
            pairs.append((serial[inode],rec))
        pairs.sort();table=build_table(pairs,inventory)
        namespace=canonical(b'LFS6FSR\0'+ns[8:12]+profile+scope+serial[ns[44:76]]+table)
        new_root=oid(namespace);inventory[new_root]=namespace;assert len(namespace)==129 and value(namespace)[44:76]==scope
        assert reverse[value(namespace)[76:84]]==ns[44:76]
        decoded=decode_table(inventory,value(namespace)[84:116]);assert decoded==pairs
        old_values={inode:value(raw[record])[12:85] for inode,record in original_pairs};restored=[];directories=entries=0
        for key,v in decoded:
            inode=reverse[key];old=old_values[inode];assert v[:9]==old[:9] and v[41:]==old[41:]
            if v[0]==2:
                old_state=value(raw[old[9:41]]);old_mapping=old_state[53:85]
                got=[(name,reverse[ref]) for name,ref in decode_directory(inventory,v[9:41])]
                expected=base.dirs(raw,old_mapping);assert got==expected
                assert value(inventory[v[9:41]])[11]==old_state[20]
                restored.append((inode,v[:9]+old[9:41]+v[41:]));directories+=1;entries+=len(expected)
            else:assert v==old;restored.append((inode,v))
        restored.sort();assert restored==sorted(old_values.items())
        print("verified namespace",len(state_checks)+1,"/",len(roots),flush=True) if (len(state_checks)+1)%20==0 else None
        state_checks.append(dict(original_root=root.hex(),new_root=new_root.hex(),inodes=len(decoded),directories=directories,directory_entries=entries,semantic_sha256=hashlib.sha256(b''.join(i+v for i,v in restored)).hexdigest()))
    # Full original directory population, including nodes shared across states, also matched.
    for old,new in dir_cache.items():assert [(n,reverse[s]) for n,s in decode_directory(inventory,new)]==base.dirs(raw,old)
    for i,b in inventory.items():assert oid(b)==i
    return inventory,serial,allocator,scope,state_checks,dir_cache

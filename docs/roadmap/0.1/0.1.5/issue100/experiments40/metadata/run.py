"""Three fixed original-metadata alternatives; see protocol.md before running."""
import collections as C, ctypes as T, ctypes.util, hashlib, json, math, pathlib, sqlite3, struct, sys
HERE=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/'tools'))
from hashing import blake3
STORE=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-candidate-1/deepseek-ten/host-runtime/store.sqlite')
SHA='713e43e4f31a489c8eb347b953702ca97c33b17832fbdc0633018584308507f4'
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def oid(b):return blake3(b'layerfs/object/v2\0'+b)
def canonical(v):return b'LFSO\1'+struct.pack('>II',len(v)+4,len(v))+v
def value(b):
    assert b[:5]==b'LFSO\1' and struct.unpack_from('>II',b,5)==(len(b)-9,len(b)-13)
    return b[13:]
def role(b):return value(b)[:8]
class CP(T.Structure):_fields_=[(n,T.c_uint) for n in ('windowLog','chainLog','hashLog','searchLog','minMatch','targetLength','strategy')]
class FP(T.Structure):_fields_=[(n,T.c_uint) for n in ('contentSizeFlag','checksumFlag','noDictIDFlag')]
zpath=T.util.find_library('zstd');z=T.CDLL(zpath)
def bind(name,args,ret):
    f=getattr(z,name);f.argtypes=args;f.restype=ret;return f
u=T.c_size_t;p=T.c_void_p
bind('ZSTD_getCParams',[T.c_int,T.c_ulonglong,u],CP);bind('ZSTD_estimateCCtxSize_usingCParams',[CP],u)
bind('ZSTD_initStaticCCtx',[p,u],p);bind('ZSTD_CCtx_setCParams',[p,CP],u);bind('ZSTD_CCtx_setFParams',[p,FP],u)
bind('ZSTD_compressBound',[u],u);bind('ZSTD_compress2',[p,p,u,p,u],u);bind('ZSTD_isError',[u],T.c_uint)
bind('ZSTD_decompress',[p,u,p,u],u);bind('ZSTD_versionNumber',[],T.c_uint)
def check(v):
    assert not z.ZSTD_isError(v),('zstd error',v)
    return v
def decompress(frame,n):
    dest=T.create_string_buffer(n);assert check(z.ZSTD_decompress(dest,n,frame,len(frame)))==n;return dest.raw
codec_calls=0;max_workspace=0
def compress(raw):
    global codec_calls,max_workspace
    assert 0<len(raw)<=65536
    params=z.ZSTD_getCParams(1,len(raw),0);params.windowLog=min(params.windowLog,16)
    size=check(z.ZSTD_estimateCCtxSize_usingCParams(params));assert 0<size<=1048576
    mem=(T.c_uint64*((size+7)//8))();ctx=z.ZSTD_initStaticCCtx(mem,T.sizeof(mem));assert ctx
    check(z.ZSTD_CCtx_setCParams(ctx,params));check(z.ZSTD_CCtx_setFParams(ctx,FP(1,1,1)))
    bound=check(z.ZSTD_compressBound(len(raw)));assert bound<=66560
    buf=T.create_string_buffer(bound);n=check(z.ZSTD_compress2(ctx,buf,bound,raw,len(raw)))
    encoded=buf.raw[:n];assert decompress(encoded,len(raw))==raw
    codec_calls+=1;max_workspace=max(max_workspace,T.sizeof(mem))
    return (1,encoded) if n+16<=len(raw) else (0,raw)
def apply_delta(r,base):
    n,count=struct.unpack_from('<II',r,33);out=bytearray();pos=41
    for _ in range(count):
        op=r[pos];pos+=1
        if op==0:
            start,length=struct.unpack_from('<II',r,pos);pos+=8;assert start+length<=len(base);out+=base[start:start+length]
        else:
            assert op==1;length=struct.unpack_from('<I',r,pos)[0];pos+=4;out+=r[pos:pos+length];pos+=length
        assert length>0
    assert len(out)==n and pos==len(r);return bytes(out)
def load():
    assert sha(STORE)==SHA
    db=sqlite3.connect(STORE.as_uri()+'?mode=ro&immutable=1',uri=True)
    rows=list(db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects'))
    locations={(p,g,k):(i,n) for i,n,p,g,k in rows};records={};raw={};groups=0;encoded_bytes=0;packs=0
    for pack,blob in db.execute('select pack_id,data from object_packs order by pack_id'):
        if struct.unpack_from('<I',blob,8)[0]!=1:continue
        packs+=1;num=struct.unpack_from('<I',blob,12)[0]
        for group in range(num):
            off,enc,dec,codec=struct.unpack_from('<IIII',blob,16+16*group);frame=blob[off:off+enc]
            body=decompress(frame,dec) if codec else frame
            got_codec,got_frame=compress(body)
            assert (got_codec,got_frame)==(codec,frame),('original codec mismatch',pack,group)
            count=struct.unpack_from('<I',body)[0];start=4+4*count;last=start
            for k in range(count):
                end=start+struct.unpack_from('<I',body,4+4*k)[0];r=body[last:end];last=end
                i,n=locations[pack,group,k];records[i]=r
                if r[0]==0:raw[i]=r[1:]
            assert last==len(body);groups+=1;encoded_bytes+=len(frame)
    for i,r in records.items():
        if r[0]:assert r[0]==1 and records[r[1:33]][0]==0;raw[i]=apply_delta(r,raw[r[1:33]])
        assert oid(raw[i])==i
    for i,n,p,g,k in rows:
        if i in raw:assert len(raw[i])==n
    assert len(raw)==46288 and groups==782 and encoded_bytes==6535686
    roots=sorted([i for i,b in raw.items() if role(b)==b'LFS4FSR\0'],key=lambda i:next(p for j,n,p,g,k in rows if i==j))
    assert len(roots)==11;db.close()
    return raw,rows,roots,dict(groups=groups,packs=packs,encoded_groups_bytes=encoded_bytes,pack_bytes=encoded_bytes+16*groups+16*packs,authenticated_objects=len(raw))
def original_pairs(raw,root):
    v=value(raw[root]);assert v[:8]==b'LFS4INT\0';count=int.from_bytes(v[13:15],'big');assert len(v)==31+count*64
    rows=[(v[j:j+32],v[j+32:j+64]) for j in range(31,len(v),64)]
    assert rows==sorted(rows)
    if v[10]==7:return rows
    assert v[10]==8;return [x for k,r in rows for x in original_pairs(raw,r)]
def dirs(raw,root):
    v=value(raw[root]);assert v[:8]==b'LFS4NSP\0';pos=31;rows=[]
    for _ in range(int.from_bytes(v[13:15],'big')):
        n=int.from_bytes(v[pos:pos+2],'big');pos+=2;name=v[pos:pos+n];pos+=n;ref=v[pos:pos+32];pos+=32;rows.append((name,ref))
    assert pos==len(v) and rows==sorted(rows)
    if v[10]==1:result=rows
    else:assert v[10]==2;result=[x for name,r in rows for x in dirs(raw,r)]
    assert len(result)==int.from_bytes(v[15:23],'big')
    assert sum(34+len(n) for n,i in result)==int.from_bytes(v[23:31],'big')
    return result

def partition(rows,limit):
    num=math.ceil(len(rows)/limit);q,r=divmod(len(rows),num);pos=0;out=[]
    for j in range(num):
        size=q+(j<r);assert 1<=size<=limit
        if num>1:assert size>=limit//2
        out.append(rows[pos:pos+size]);pos+=size
    assert pos==len(rows);return out

def build_table(pairs,inventory):
    def put(v):
        b=canonical(v);assert len(b)<=8192;i=oid(b);assert i not in inventory or inventory[i]==b;inventory[i]=b;return i
    def header(kind,level,count,subtree):return b'LFS5INT\0'+struct.pack('>HBBB HQQ',1,kind,level,0,count,subtree,subtree*105)
    current=[]
    for rows in partition(pairs,77):
        i=put(header(7,0,len(rows),len(rows))+b''.join(k+v for k,v in rows));current.append((rows[-1][0],i,len(rows)))
    level=0
    while len(current)>1:
        level+=1;next_level=[]
        for rows in partition(current,127):
            count=sum(n for k,i,n in rows);i=put(header(8,level,len(rows),count)+b''.join(k+i for k,i,n in rows));next_level.append((rows[-1][0],i,count))
        current=next_level
    assert level<=31;return current[0][1]
def decode_table(inventory,root):
    b=inventory[root];assert oid(b)==root and len(b)<=8192;v=value(b);assert v[:8]==b'LFS5INT\0'
    kind,level,flags=v[10:13];n=int.from_bytes(v[13:15],'big');assert flags==0
    width=105 if kind==7 else 64;assert len(v)==31+n*width
    rows=[(v[j:j+32],v[j+32:j+width]) for j in range(31,len(v),width)]
    assert [k for k,r in rows]==sorted(set(k for k,r in rows))
    if kind==7:assert level==0 and n<=77;result=rows
    else:
        assert kind==8 and 1<=level<=31 and n<=127;result=[]
        for maximum,child in rows:
            child_value=value(inventory[child]);assert child_value[11]+1==level
            pairs=decode_table(inventory,child);assert pairs[-1][0]==maximum;result+=pairs
    assert len(result)==int.from_bytes(v[15:23],'big') and len(result)*105==int.from_bytes(v[23:31],'big')
    return result

def treatment(raw,roots,label):
    if label=='A':return dict(raw),roots,[]
    removed={b'LFS4INO\0',b'LFS4INT\0',b'LFS4FSR\0'}
    if label=='C':removed.add(b'LFS4DIR\0')
    inventory={i:b for i,b in raw.items() if role(b) not in removed};new_roots=[];checks=[]
    profile=blake3(b'layerfs/diagnostic/inline-inode/v1\0'+label.encode()+b'/stable32/leaf77/branch127/page8192')
    for original_root in roots:
        namespace=value(raw[original_root]);old=original_pairs(raw,namespace[76:108]);pairs=[];old_semantic=[]
        for inode,record_id in old:
            rec=value(raw[record_id]);assert rec[:8]==b'LFS4INO\0';inode_value=rec[12:85];assert len(inode_value)==73
            old_semantic.append((inode,inode_value))
            if label=='C' and inode_value[0]==2:
                state=value(raw[inode_value[9:41]]);assert state[:8]==b'LFS4DIR\0';mapping=state[53:85];m=value(raw[mapping])
                assert state[20]==m[11] and state[12:20]==m[15:23]
                assert len(dirs(raw,mapping))==int.from_bytes(state[12:20],'big')
                inode_value=inode_value[:9]+mapping+inode_value[41:]
            pairs.append((inode,inode_value))
        table=build_table(pairs,inventory);new_namespace=canonical(b'LFS5FSR\0'+namespace[8:12]+profile+namespace[44:76]+table)
        new_id=oid(new_namespace);inventory[new_id]=new_namespace;new_roots.append(new_id)
        decoded=decode_table(inventory,table);assert decoded==pairs
        restored=[];directory_count=0;mapping_entries=0
        for (inode,v),(old_inode,old_v) in zip(decoded,old_semantic):
            assert inode==old_inode and v[:9]==old_v[:9] and v[41:]==old_v[41:]
            if v[0]==2:
                old_state=value(raw[old_v[9:41]]);old_map=old_state[53:85]
                new_map=v[9:41] if label=='C' else value(inventory[v[9:41]])[53:85]
                assert new_map==old_map and dirs(inventory,new_map)==dirs(raw,old_map)
                restored.append((inode,v[:9]+old_v[9:41]+v[41:]));directory_count+=1;mapping_entries+=len(dirs(raw,old_map))
            else:assert v==old_v;restored.append((inode,v))
        assert restored==old_semantic
        assert value(new_namespace)[44:76]==namespace[44:76]
        checks.append(dict(original_root=original_root.hex(),new_root=new_id.hex(),inodes=len(decoded),directories=directory_count,directory_entries=mapping_entries,semantic_sha256=hashlib.sha256(b''.join(i+v for i,v in restored)).hexdigest()))
    return inventory,new_roots,checks

def hardlink_rename_check():
    inode=blake3(b'fixture/stable inode');v=b'\1'+(2).to_bytes(8,'big')+blake3(b'content')+blake3(b'metadata')
    inv={};root=build_table([(inode,v)],inv);assert decode_table(inv,root)==[(inode,v)]
    def node(rows):
        return canonical(b'LFS4NSP\0'+struct.pack('>HBBB HQQ',1,1,0,0,len(rows),len(rows),sum(34+len(n) for n,i in rows))+b''.join(len(n).to_bytes(2,'big')+n+i for n,i in rows))
    before=node([(b'a',inode),(b'b',inode)]);after=node([(b'b',inode),(b'c',inode)])
    before_id=oid(before);after_id=oid(after);inv.update({before_id:before,after_id:after})
    assert before_id!=after_id and dirs(inv,before_id)==[(b'a',inode),(b'b',inode)] and dirs(inv,after_id)==[(b'b',inode),(b'c',inode)]
    assert decode_table(inv,root)[0][1][1:9]==(2).to_bytes(8,'big')
    for n,i in dirs(inv,after_id):assert i==inode
    return dict(status='PASS',scope='representation roundtrip preserves stable inode and linkcount2 across namespace rename; no product mutation implementation exercised')

SCHEMA='''CREATE TABLE objects(object_id BLOB NOT NULL PRIMARY KEY CHECK(length(object_id)=32),canonical_length INTEGER NOT NULL CHECK(canonical_length>0 AND canonical_length<=16777216),pack_id INTEGER NOT NULL CHECK(pack_id>0),group_number INTEGER NOT NULL CHECK(group_number>=0 AND group_number<256),record_number INTEGER NOT NULL CHECK(record_number>=0 AND record_number<8191)) STRICT, WITHOUT ROWID'''
def pack_inventory(inventory,base_pack):
    groups=[];pending=[];size=4
    def finish(rows):
        area=b''.join(b'\0'+b for i,b in rows);ends=[];end=0
        for i,b in rows:end+=1+len(b);ends.append(end)
        body=struct.pack('<I',len(rows))+struct.pack('<'+'I'*len(ends),*ends)+area
        assert len(body)<=16384;codec,frame=compress(body)
        return dict(ids=[i for i,b in rows],body=body,codec=codec,frame=frame)
    for i,b in sorted(inventory.items(),key=lambda kv:(role(kv[1]),kv[0])):
        assert len(b)<=8192
        if pending and size+5+len(b)>16384:groups.append(finish(pending));pending=[];size=4
        pending.append((i,b));size+=5+len(b)
    if pending:groups.append(finish(pending))
    packs=[];pending=[];enc=dec=16;records=0
    def flush(gs):
        header=b'LFPACK\0\0'+struct.pack('<II',1,len(gs));offset=16+16*len(gs);directory=b''
        for g in gs:
            directory+=struct.pack('<IIII',offset,len(g['frame']),len(g['body']),g['codec']);offset+=len(g['frame'])
        blob=header+directory+b''.join(g['frame'] for g in gs);assert len(blob)==offset
        return blob
    locators=[]
    for group in groups:
        if pending and (enc+16+len(group['frame'])>262144 or dec+16+len(group['body'])>262144 or len(pending)==256 or records+len(group['ids'])>8191):
            packs.append(flush(pending));pending=[];enc=dec=16;records=0
        pnum=base_pack+len(packs)+1;gnum=len(pending)
        for k,i in enumerate(group['ids']):locators.append((i,len(inventory[i]),pnum,gnum,k))
        pending.append(group);enc+=16+len(group['frame']);dec+=16+len(group['body']);records+=len(group['ids'])
    if pending:packs.append(flush(pending))
    assert len(locators)==len(inventory)
    return locators,dict(metadata_objects=len(inventory),canonical_bytes=sum(map(len,inventory.values())),groups=len(groups),packs=len(packs),encoded_group_bytes=sum(len(g['frame']) for g in groups),decoded_group_bytes=sum(len(g['body']) for g in groups),pack_bytes=sum(map(len,packs)),pack_sha256=[hashlib.sha256(p).hexdigest() for p in packs],max_canonical_bytes=max(map(len,inventory.values())),role_counts=dict(C.Counter(role(b).rstrip(b'\0').decode() for b in inventory.values())))
def project_index(label,rows):
    path=HERE/(label+'-index.sqlite');assert not path.exists(),('do not overwrite earlier projection',path)
    db=sqlite3.connect(path);db.execute('pragma page_size=4096');db.execute(SCHEMA)
    db.executemany('insert into objects values(?,?,?,?,?)',sorted(rows));db.commit();db.execute('vacuum')
    counts=db.execute("select count(*),sum(pgsize),sum(payload),sum(unused) from dbstat where name='objects'").fetchone()
    assert db.execute('pragma integrity_check').fetchone()[0]=='ok'
    result=dict(rows=len(rows),index_pages=counts[0],index_bytes=counts[1],payload_bytes=counts[2],unused_bytes=counts[3],database_bytes=path.stat().st_size)
    db.close();result['sha256']=sha(path);return result

def main():
    assert blake3(b'').hex()=='af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262'
    assert canonical(b'abc')==b'LFSO\1\0\0\0\7\0\0\0\3abc'
    assert [len(x) for x in partition(list(range(78)),77)]==[39,39]
    fixture=hardlink_rename_check();raw,rows,roots,original=load();print('Authenticated original metadata and byte-matched all782 original groups',flush=True)
    content_rows=[r for r in rows if r[0] not in raw];assert len(content_rows)==33952
    results={};root_maps={}
    for label in ('A','B','C'):
        inventory,new_roots,checks=treatment(raw,roots,label)
        for i,b in inventory.items():assert oid(b)==i
        locators,pack=pack_inventory(inventory,max(r[2] for r in rows));index=project_index(label,content_rows+locators)
        results[label]={**pack,'index':index,'metadata_plus_all_object_index_bytes':pack['pack_bytes']+index['index_bytes'],'new_canonical_objects':sum(i not in raw for i in inventory),'removed_original_objects':sum(i not in inventory for i in raw),'verified_states':len(checks) if label!='A' else len(roots),'original_unchanged_objects':sum(i in raw for i in inventory)}
        root_maps[label]=checks or [{'original_root':i.hex(),'new_root':i.hex()} for i in roots]
        print(label,json.dumps({k:results[label][k] for k in ('metadata_objects','canonical_bytes','groups','packs','pack_bytes','metadata_plus_all_object_index_bytes')}),flush=True)
    assert sha(STORE)==SHA
    out=dict(protocol_sha256=sha(HERE/'protocol.md'),script_sha256=sha(pathlib.Path(__file__)),source_store=str(STORE),source_sha256_before=SHA,source_sha256_after=sha(STORE),zstd_library=zpath,zstd_version=z.ZSTD_versionNumber(),zstd_library_sha256=sha(pathlib.Path(zpath)),hash_helper_sha256=sha(HERE.parent/'tools/libdiagnostic_hash.dylib'),original_selected=original,codec_calls=codec_calls,max_codec_workspace_bytes=max_workspace,hardlink_rename=fixture,results=results,scope='Offline matched FULL-only sorted metadata packing and separate full-hash locator projection. No public/product allocation or Commit measurement.')
    (HERE/'result.json').write_text(json.dumps(out,indent=2)+'\n');(HERE/'roots.json').write_text(json.dumps(root_maps,indent=2)+'\n')
if __name__=='__main__':main()

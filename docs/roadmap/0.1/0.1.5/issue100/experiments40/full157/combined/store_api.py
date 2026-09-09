"""Two fixed unsupported complete-copy layouts; semantic/identity checks, no product writes."""
import collections, hashlib, importlib.util,json,pathlib,sqlite3,struct,sys,time
HERE=pathlib.Path(__file__).resolve().parent;EXP=HERE.parent
sys.path[:0]=[str(EXP/'metadata'),str(pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/tools'))]
import api as d_api
from hashing import blake3
spec=importlib.util.spec_from_file_location('framing_diag',pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/framing/experiment.py'));fr=importlib.util.module_from_spec(spec);spec.loader.exec_module(fr)
sha=fr.sha;check=fr.check;SRC=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full/host-runtime/store.sqlite');EXPECTED='f323de0e0f9ae1030efc142402bc033ad427134dd8c69f21eb5b5d6ef7426eb7'

def optional(b):return b'\x00' if b is None else b'\x01'+b
def layer_id(stack,parent,root):return b'\x32'+blake3(b'layerfs/layer/v2\0'+stack+optional(parent)+root)
def commit_id(root,parent,layer):return b'\x12'+blake3(b'layerfs/commit/v2\0'+root+optional(parent)+layer)
def derive_sql(source,rootmap):
 tables={}
 for name in ('layers','commits','branches','layer_stacks','workspace_stages'):
  cur=source.execute('select * from '+name);columns=[x[0] for x in cur.description];tables[name]=[dict(zip(columns,row)) for row in cur]
 check(len(tables['layers'])==1 and len(tables['commits'])==157 and not tables['workspace_stages'],'fixture SQL shape')
 layer=tables['layers'][0]
 check(layer['parent_layer_id'] is None and layer['source_commit_id'] is None and layer['source_branch_id'] is None,'genesis')
 check(layer['layer_id']==layer_id(layer['layer_stack_id'],None,layer['root_id']),'original layer identity')
 lm={layer['layer_id']:layer_id(layer['layer_stack_id'],None,rootmap[layer['root_id']])};cm={};pending=tables['commits'][:]
 while pending:
  ready=[r for r in pending if r['parent_commit_id'] is None or r['parent_commit_id'] in cm]
  check(ready,'commit DAG')
  for r in ready:
   check(r['commit_id']==commit_id(r['root_id'],r['parent_commit_id'],r['base_layer_id']),'original commit identity')
   cm[r['commit_id']]=commit_id(rootmap[r['root_id']],cm.get(r['parent_commit_id']),lm[r['base_layer_id']]);pending.remove(r)
 check(len(set(cm.values()))==157,'distinct commits')
 mappings={'root_id':rootmap,'layer_id':lm,'parent_layer_id':lm,'head_layer_id':lm,'base_layer_id':lm,'commit_id':cm,'parent_commit_id':cm,'head_commit_id':cm,'source_commit_id':cm}
 rewritten={name:[{k:(mappings[k][v] if k in mappings and v is not None else v) for k,v in row.items()} for row in rows] for name,rows in tables.items()}
 return tables,rewritten,lm,cm

def groups(b):
 check(b[:8]==b'LFPACK\0\0' and len(b)<=262144,'pack framing');version,n=struct.unpack_from('<II',b,8);check(version in (1,2) and 1<=n<=256,'pack kind/count')
 p=16+16*n;out=[]
 for g in range(n):
  pos,encoded,decoded,codec=struct.unpack_from('<IIII',b,16+16*g);check(pos==p and p+encoded<=len(b),'group bounds');p+=encoded
  body=d_api.codec.decompress(b[pos:p],decoded) if codec else b[pos:p]
  check(len(body)==decoded and len(body)<=65536,'group decoded bound');check(version!=2 or codec==0,'native codec')
  count=fr.u32(body,0);check(1<=count<=8191,'record count');start=4+4*count;last=start;records=[]
  for i in range(count):
   end=start+fr.u32(body,4+4*i);check(last<end<=len(body),'record ends');records.append(body[last:end]);last=end
  check(last==len(body),'group trailing');out.append(records)
 check(p==len(b),'pack trailing');return version,out

def native_record(b):
 check(len(b)>=6 and b[0] in (0,1),'native kind');kind=b[0];raw=fr.u32(b,1);check(0<=raw<=32768,'native raw')
 start=37 if kind else 5;frame=b[start:];check(1<=len(frame)<=33024,'native frame')
 return kind,raw,b[5:37] if kind else None,frame

def rebuild_native(b,p,byloc,selected):
 v,gs=groups(b);check(v==2,'native pack')
 bodies=[]
 for g,rs in enumerate(gs):
  new=[]
  for k,r in enumerate(rs):
   identity,n=byloc[p,g,k]
   if identity.hex() in selected:
    s=selected[identity.hex()];check((s['pack'],s['group'],s['ordinal'])==(p,g,k),'selected locator')
    check(s['raw']+21==n and len(rs)==s['group_count'],'selected canonical')
    frame=bytes.fromhex(s['frame']);r=bytes([s['kind']])+struct.pack('<I',s['raw'])+(bytes.fromhex(s['base']) if s['base'] else b'')+frame
    check(len(r)==s['size'],'selected size')
   native_record(r);new.append(r)
  ends=[];end=0
  for r in new:end+=len(r);ends.append(end)
  body=struct.pack('<I',len(new))+struct.pack('<'+'I'*len(ends),*ends)+b''.join(new);check(len(body)<=65536,'native group limit');bodies.append(body)
 pos=16+16*len(bodies);entries=[]
 for b in bodies:entries.append(struct.pack('<IIII',pos,len(b),len(b),0));pos+=len(b)
 result=b'LFPACK\0\0'+struct.pack('<II',2,len(bodies))+b''.join(entries)+b''.join(bodies)
 check(len(result)<=262144,'native pack limit');return result

def auth_native(records,loc,z):
 maxima=[0]*4
 for target in records:
  nodes=[];seen=set();node=target
  while True:
   check(node not in seen and node in records,'native cycle/missing');seen.add(node)
   kind,raw,base,frame=records[node];nodes.append(node)
   if base is None:break
   check(base in loc and loc[base][0]<loc[node][0],'native chronology');node=base
  check(len(nodes)<=5,'native depth');rawclosure=sum(records[n][1] for n in nodes)
  encoded=sum(32+4+4*loc[n][5]+loc[n][4] for n in nodes);decoded=encoded+sum(records[n][1]+21 for n in nodes)
  check(rawclosure<=1048576 and encoded<=393216 and decoded<=524288,'native closure budget')
  basebytes=None
  for n in reversed(nodes):
   kind,raw,base,frame=records[n];basebytes,workspace=fr.decode(z,frame,raw,basebytes)
   canonical=b'LFSO\x01'+struct.pack('>II',raw+12,raw+8)+b'LFS4CHK\0'+basebytes
   check(blake3(b'layerfs/object/v2\0'+canonical)==n and len(canonical)==loc[n][3],'native authentication')
  maxima=[max(a,b) for a,b in zip(maxima,(len(nodes)-1,rawclosure,encoded,decoded))]
 return dict(objects=len(records),max_depth=maxima[0],max_raw_closure=maxima[1],max_encoded_work=maxima[2],max_decoded_work=maxima[3])

def verify_copy(copy,inventory,newroots,small_verified,z):
 rows=list(copy.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects'))
 byloc={(p,g,r):(i,n) for i,n,p,g,r in rows};check(len(rows)==len(byloc),'unique locators')
 seen_ids={row[0] for row in rows};seen=set();small={};sloc={};native={};nloc={};meta={};small_hash=hashlib.sha256();categories=collections.Counter()
 for p,b in copy.execute('select pack_id,data from object_packs order by pack_id'):
  if b[:8]==fr.MAGIC:
   n=fr.u32(b,12);lengths=[byloc[p,g,0][1] for g in range(n)];rs=fr.unpack(b,102,lengths);categories['small_pack_bytes']+=len(b)
   small_hash.update(p.to_bytes(8,'little')+b)
   for g,r in enumerate(rs):
    i,n=byloc[p,g,0];seen.add((p,g,0));small[i]=r;sloc[i]=(p,g,0,n)
  else:
   v,gs=groups(b);categories['metadata_pack_bytes' if v==1 else 'native_pack_bytes']+=len(b)
   for g,rs in enumerate(gs):
    for k,r in enumerate(rs):
     i,n=byloc[p,g,k];seen.add((p,g,k))
     if v==1:
      check(r[0]==0,'D metadata FULL');canonical=r[1:];check(len(canonical)==n and d_api.codec.oid(canonical)==i,'D physical authentication');meta[i]=canonical
     else:
      parsed=native_record(r);check(n==parsed[1]+21,'native length');native[i]=parsed;nloc[i]=(p,g,k,n,len(r),len(rs))
 check(seen==set(byloc),'physical locator membership');check(meta==inventory,'exact D inventory');check(len(small)==75398 and len(native)==3221,'content counts')
 for root in newroots:
  ns=d_api.codec.value(meta[root]);check(ns[:8]==b'LFS6FSR\0','D namespace');table=d_api.decode_table(meta,ns[84:116]);keys={k for k,v in table};check(ns[76:84] in keys,'root inode')
  for k,v in table:
   check(v[41:73] in meta,'inode metadata reference')
   if v[0]==2:
    for name,key in d_api.decode_directory(meta,v[9:41]):check(key in keys,'directory inode reference')
   else:check(v[9:41] in seen_ids,'inode content reference')
 digest=small_hash.hexdigest()
 if small_verified is None:
  maximum=[0]*4
  for i in small:
   facts=fr.authenticate(i,small,sloc,z,blake3);maximum=[max(a,b) for a,b in zip(maximum,facts)]
  smallcheck=dict(objects=len(small),pack_graph_sha256=digest,max_depth=maximum[0],max_canonical_closure=maximum[1],max_encoded_closure=maximum[2],max_decoder_dictionary_workspace=maximum[3],verification='all targets/dependencies authenticated')
 else:
  check(digest==small_verified['pack_graph_sha256'],'same authenticated small graph');smallcheck={**small_verified,'verification':'exact same pack bytes/locators as fully authenticated first treatment'}
 return dict(categories),smallcheck,auth_native(native,nloc,z)


class StoreReader:
 """Read actual unsupported combined copy, authenticating bytes; bounded payload caches."""
 def __init__(self,path,cache_bytes=32*1024*1024,pack_cache_bytes=8*1024*1024):
  self.path=pathlib.Path(path).resolve();self.db=sqlite3.connect(self.path.as_uri()+'?mode=ro&immutable=1',uri=True)
  check(self.db.execute('pragma user_version').fetchone()[0]==9301,'diagnostic full157 version')
  self.loc={i:(p,g,r,n) for i,n,p,g,r in self.db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')}
  self.cache=collections.OrderedDict();self.cache_bytes=0;self.cache_limit=cache_bytes
  self.packs=collections.OrderedDict();self.pack_bytes=0;self.pack_limit=pack_cache_bytes;self.z=fr.setup_decoder()
 def close(self):self.db.close();self.cache.clear();self.packs.clear()
 def _pack(self,p):
  if p in self.packs:self.packs.move_to_end(p);return self.packs[p]
  row=self.db.execute('select data from object_packs where pack_id=?',(p,)).fetchone();check(row,'missing pack');b=row[0];check(len(b)<=262144,'pack cap')
  while self.packs and self.pack_bytes+len(b)>self.pack_limit:_,old=self.packs.popitem(last=False);self.pack_bytes-=len(old)
  self.packs[p]=b;self.pack_bytes+=len(b);return b
 def _physical(self,i):
  check(i in self.loc,'missing object');p,g,r,n=self.loc[i];b=self._pack(p);count=fr.u32(b,12);check(1<=count<=256 and 0<=g<count,'group locator')
  if b[:8]==fr.MAGIC:
   check(fr.u32(b,8)==102 and r==0 and 24<=n<131095,'small locator')
   starts=[fr.u32(b,16+4*k) for k in range(count)]+[len(b)]
   check(starts[0]==16+4*count and all(a<e<=len(b) for a,e in zip(starts,starts[1:])),'small offsets')
   record=b[starts[g]:starts[g+1]];check(record and record[0] in (0,1,2),'small kind')
   size=len(record)-1-32*(record[0]!=0);check(1<=size<=135168,'small frame size')
   old=record[:1]+struct.pack('<II',n-23,size)+record[1:];fr.record(old);return 3,old,count
  check(b[:8]==fr.OLD and fr.u32(b,8) in (1,2),'pack version');version=fr.u32(b,8)
  start,encoded,decoded,codec=struct.unpack_from('<IIII',b,16+16*g)
  check(start>=16+16*count and start+encoded<=len(b) and 1<=decoded<=65536,'group bounds')
  check(codec in (0,1) and (version==1 or codec==0),'group codec')
  body=d_api.codec.decompress(b[start:start+encoded],decoded) if codec else b[start:start+encoded]
  check(len(body)==decoded,'group length');count=fr.u32(body,0);check(1<=count<=8191 and 0<=r<count,'record locator')
  directory=4+4*count;ends=[0]+[fr.u32(body,4+4*k) for k in range(count)]
  check(ends[-1]+directory==len(body) and all(a<e for a,e in zip(ends,ends[1:])),'record directory')
  return version,body[directory+ends[r]:directory+ends[r+1]],count
 def _put(self,i,b):
  check(len(b)==self.loc[i][3] and blake3(b'layerfs/object/v2\0'+b)==i,'canonical identity')
  if len(b)>self.cache_limit:return b
  while self.cache and self.cache_bytes+len(b)>self.cache_limit:_,old=self.cache.popitem(last=False);self.cache_bytes-=len(old)
  if i not in self.cache:self.cache[i]=b;self.cache_bytes+=len(b)
  return b
 def read_canonical(self,i):
  if isinstance(i,str):i=bytes.fromhex(i)
  if i in self.cache:self.cache.move_to_end(i);return self.cache[i]
  version,record,count=self._physical(i)
  if version==1:check(record[0]==0,'D FULL metadata');return self._put(i,record[1:])
  nodes=[];seen=set();node=i;rawsum=encsum=decsum=0;requirefull=False
  while True:
   check(node not in seen,'cycle');seen.add(node);v,r,count=self._physical(node);check(v==version,'base role')
   kind,raw,base,frame=fr.record(r) if v==3 else native_record(r)
   check(self.loc[node][3]==raw+(23 if v==3 else 21),'canonical length')
   check(not requirefull or kind==0,'legacy FULL base');requirefull=v==3 and kind==1
   rawsum+=self.loc[node][3] if v==3 else raw
   encsum+=len(r) if v==3 else 32+4+4*count+len(r)
   decsum+=0 if v==3 else 32+4+4*count+len(r)+raw+21
   check(rawsum<=(524288 if v==3 else 1048576) and encsum<=(262144 if v==3 else 393216) and decsum<=524288,'closure')
   nodes.append((node,raw,frame))
   if base is None:break
   check(len(nodes)<=(8 if v==3 else 4),'depth');check(base in self.loc,'base missing')
   check(self.loc[base][0]<=self.loc[node][0] if v==3 else self.loc[base][0]<self.loc[node][0],'chronology');node=base
  prefix=None;canonical=None
  for node,raw,frame in reversed(nodes):
   prefix,workspace=fr.decode(self.z,frame,raw,prefix)
   canonical=(b'LFSO\x01'+struct.pack('>II',raw+14,raw+10)+b'LFS5SML\0\0\x01'+prefix) if version==3 else (b'LFSO\x01'+struct.pack('>II',raw+12,raw+8)+b'LFS4CHK\0'+prefix)
   self._put(node,canonical)
  return canonical

"""Bounded reader for shared-inode-value physical metadata, original canonical IDs."""
import collections, pathlib, sqlite3, sys
BASE=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural')
sys.path.insert(0,str(BASE/'combined'));import store_api as old
check=old.check;d_api=old.d_api
class StoreReader(old.StoreReader):
 def __init__(self,path,cache_bytes=4*1024*1024,pack_cache_bytes=8*1024*1024):
  self.path=pathlib.Path(path).resolve();self.db=sqlite3.connect(self.path.as_uri()+'?mode=ro&immutable=1',uri=True)
  check(self.db.execute('pragma user_version').fetchone()[0]==9303,'shared physical format')
  self.loc={i:(p,g,r,n) for i,n,p,g,r in self.db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')}
  self.ordinals=dict(self.db.execute('select ordinal,object_id from shared_inode_values'));check(len(self.ordinals)<2**32,'ordinal range')
  self.cache=collections.OrderedDict();self.cache_bytes=0;self.cache_limit=cache_bytes
  self.packs=collections.OrderedDict();self.pack_bytes=0;self.pack_limit=pack_cache_bytes
  self.z=old.old.fr.setup_decoder();self.metrics=collections.Counter()
 def __getitem__(self,i):return self.read_canonical(i)
 def _expand(self,b):
  if not old.chain_api.e.leaf(b):return b
  check((len(b)-44)%12==0 and 0<(len(b)-44)//12<=100,'shared leaf bounds')
  pieces=[b[:44]]
  for p in range(44,len(b),12):
   ordinal=int.from_bytes(b[p+8:p+12],'big');check(ordinal in self.ordinals,'missing value ordinal')
   i=self.ordinals[ordinal];check(self.loc[i][3]==94,'pool canonical bound')
   raw=self.read_canonical(i);check(len(raw)==94 and raw[13:21]==b'LFSIVL1\0','pool value role')
   pieces.append(b[p:p+8]+raw[-73:]);self.metrics['pool_canonical_logical_bytes']+=94;self.metrics['pool_lookups']+=1
  return b''.join(pieces)
 def read_canonical(self,identity):
  if isinstance(identity,str):identity=bytes.fromhex(identity)
  if identity in self.cache:self.cache.move_to_end(identity);return self.cache[identity]
  version,record,_=self._physical(identity)
  if version!=1:return super().read_canonical(identity)
  nodes=[];seen=set();node=identity;total=0;poolwork=0
  while True:
   check(node not in seen,'metadata dependency cycle');seen.add(node)
   v,r,_=self._physical(node);check(v==1 and r and r[0] in (0,1),'metadata role')
   n=self.loc[node][3];check(0<n<=8192,'canonical object bound');total+=n;check(total<=131072,'canonical closure bound')
   nodes.append((node,r,n))
   if not r[0]:break
   check(len(nodes)<=16 and len(r)>41,'metadata delta bound')
   base=r[1:33];check(base in self.loc and self.loc[base][0]<self.loc[node][0],'chronological base');node=base
  previous=previous_id=None;canonical=None
  for node,r,n in reversed(nodes):
   check(len(r)<=8193,'metadata physical bound')
   if r[0]:
    check(old.chain_api.e.leaf(previous),'shared base leaf');body=old.chain_api.e.matcher.replay(r,previous_id,previous);check(old.chain_api.e.leaf(body),'shared target leaf')
   else:body=r[1:]
   if old.chain_api.e.leaf(body):poolwork+=(len(body)-44)//12*94
   check(poolwork<=196608,'value canonical closure bound')
   canonical=self._expand(body);self._put(node,canonical);previous=body;previous_id=node
  self.metrics['maximum_pool_canonical_work']=max(self.metrics['maximum_pool_canonical_work'],poolwork)
  self.metrics['maximum_original_canonical_closure']=max(self.metrics['maximum_original_canonical_closure'],total)
  return canonical

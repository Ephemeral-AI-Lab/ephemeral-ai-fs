"""Physical value groups, fully hashed catalogue, original canonical leaf IDs."""
import bisect,collections,hashlib,importlib.util,pathlib,sqlite3,struct
HERE=pathlib.Path(__file__).resolve().parent
V1=HERE.parent/'full157'
spec=importlib.util.spec_from_file_location('shared_value_v1_reader',V1/'reader.py');v1=importlib.util.module_from_spec(spec);spec.loader.exec_module(v1)
old=v1.old;check=old.check;d_api=old.d_api
class StoreReader(v1.StoreReader):
 def __init__(self,path,cache_bytes=4*1024*1024,pack_cache_bytes=8*1024*1024):
  self.path=pathlib.Path(path).resolve();self.db=sqlite3.connect(self.path.as_uri()+'?mode=ro&immutable=1',uri=True)
  check(self.db.execute('pragma user_version').fetchone()[0]==9304,'group shared physical format')
  self.loc={i:(p,g,r,n) for i,n,p,g,r in self.db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects')}
  rows=list(self.db.execute('select first_ordinal,count,pack_id,group_number,group_sha256 from pool_groups order by first_ordinal'))
  self.catalogue={first:(count,p,g,h) for first,count,p,g,h in rows};self.starts=sorted(self.catalogue)
  end=1
  for first,count,p,g,h in rows:check(first==end and 1<=count<=166 and len(h)==32,'pool catalogue range');end+=count
  check(end<2**32,'pool ordinal bound');self.value_count=end-1
  self.cache=collections.OrderedDict();self.cache_bytes=0;self.cache_limit=cache_bytes
  self.packs=collections.OrderedDict();self.pack_bytes=0;self.pack_limit=pack_cache_bytes
  self.pool_groups_cache=collections.OrderedDict();self.pool_cache_bytes=0;self.pool_cache_limit=4*1024*1024
  self.z=old.old.fr.setup_decoder();self.metrics=collections.Counter()
 def pool_value(self,ordinal):
  idx=bisect.bisect_right(self.starts,ordinal)-1;check(idx>=0,'missing value ordinal')
  first=self.starts[idx];count,p,g,digest=self.catalogue[first];check(first<=ordinal<first+count,'missing value ordinal')
  if first in self.pool_groups_cache:
   self.pool_groups_cache.move_to_end(first);body,records=self.pool_groups_cache[first]
  else:
   blob=self._pack(p);check(blob[:8]==b'LFPACK\0\0' and int.from_bytes(blob[8:12],'little')==1,'pool pack role')
   groups=int.from_bytes(blob[12:16],'little');check(0<=g<groups<=256,'pool group locator')
   offset,enc,dec,codec=struct.unpack_from('<IIII',blob,16+16*g);check(0<dec<=16384 and codec in (0,1) and 16+16*groups<=offset and offset+enc<=len(blob),'pool group bound')
   frame=blob[offset:offset+enc];body=old.chain_api.e.codec.decompress(frame,dec) if codec else frame
   check(len(body)==dec and hashlib.sha256(body).digest()==digest,'pool group authentication')
   check(int.from_bytes(body[:4],'little')==count,'pool count')
   start=4+4*count;records=[];last=start
   for k in range(count):
    end=start+int.from_bytes(body[4+4*k:8+4*k],'little');check(end-last==95 and end<=len(body),'pool record shape')
    record=body[last:end];check(record[0]==0 and record[14:22]==b'LFSIVL1\0','pool value role');canonical=record[1:]
    check(canonical[:5]==b'LFSO\1' and struct.unpack_from('>II',canonical,5)==(85,81),'pool canonical envelope')
    records.append(canonical);last=end
   check(last==len(body),'pool group trailing')
   self.pool_groups_cache[first]=(body,records);self.pool_cache_bytes+=len(body)
   while self.pool_cache_bytes>self.pool_cache_limit:
    _,(discard,_)=self.pool_groups_cache.popitem(last=False);self.pool_cache_bytes-=len(discard)
   self.metrics['decoded_pool_group_bytes']+=len(body)
  self.metrics['pool_lookups']+=1;self.metrics['pool_canonical_logical_bytes']+=94
  return records[ordinal-first]
 def _expand(self,b):
  if not old.chain_api.e.leaf(b):return b
  check((len(b)-44)%12==0 and 0<(len(b)-44)//12<=100,'shared leaf bounds')
  return b[:44]+b''.join(b[p:p+8]+self.pool_value(int.from_bytes(b[p+8:p+12],'big'))[-73:] for p in range(44,len(b),12))

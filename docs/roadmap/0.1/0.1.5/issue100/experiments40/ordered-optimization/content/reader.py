"""Whole-file prefix graph and authenticated native slice adapters over metadata9304."""
import collections,importlib.util,pathlib,struct,sys
HERE=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(HERE));import content_codec as codec
spec=importlib.util.spec_from_file_location('content_metadata_reader',HERE.parent/'metadata/v2-53/reader.py');meta=importlib.util.module_from_spec(spec);spec.loader.exec_module(meta)
check=codec.check;d_api=meta.d_api;MAGIC=b'LFCNT1\0\0';MAX_PACK=4*1024*1024

class StoreReader(meta.StoreReader):
 def _pack(self,p):
  if p in self.packs:self.packs.move_to_end(p);return self.packs[p]
  row=self.db.execute('select data from object_packs where pack_id=?',(p,)).fetchone();check(row,'missing pack');data=row[0]
  check(16<=len(data)<=(MAX_PACK if data[:8]==MAGIC else 262144),'pack length')
  self.metrics['physical_pack_reads']+=1;self.metrics['physical_pack_read_bytes']+=len(data)
  if len(data)<=self.pack_limit:
   while self.packs and self.pack_bytes+len(data)>self.pack_limit:_,old=self.packs.popitem(last=False);self.pack_bytes-=len(old)
   self.packs[p]=data;self.pack_bytes+=len(data)
  return data
 def _physical(self,identity):
  check(identity in self.loc,'missing object');p,g,r,n=self.loc[identity];data=self._pack(p)
  if data[:8]!=MAGIC:return super()._physical(identity)
  version,count=struct.unpack_from('<II',data,8);check(version==107 and 1<=count<=256 and 0<=g<count and r==0 and len(data)>=16+4*count,'content locator')
  starts=list(struct.unpack_from('<'+'I'*count,data,16))+[len(data)];check(starts[0]==16+4*count and all(a<b<=len(data) for a,b in zip(starts,starts[1:])),'content offsets')
  record=data[starts[g]:starts[g+1]];check(record and record[0] in range(6),'content kind');kind=record[0]
  if kind in (0,1):check(24<=n<131095,'small canonical length')
  elif kind in (2,3):check(131072+21<=n<=codec.MAX_RAW+21,'large canonical length')
  else:check(22<=n<=32768+21,'native canonical length')
  if kind==4:check(len(record)==41,'slice record length')
  else:check(1<=len(record)-1-32*(kind in (1,3))<=codec.MAX_FRAME,'content frame length')
  return 107,record,count
 def _file(self,identity):
  nodes=[];seen=set();node=identity;decoded=encoded=0
  while True:
   check(node not in seen,'file dependency cycle');seen.add(node);v,r,_=self._physical(node);check(v==107 and r[0] in (0,1,2,3),'file base role');n=self.loc[node][3];nodes.append((node,r,n));decoded+=n;encoded+=len(r)
   check(decoded<=67108864 and encoded<=67108864,'file closure bound')
   if r[0] in (0,2):break
   check(len(nodes)<=50,'file edge bound');node=r[1:33]
  self.metrics['maximum_file_edges']=max(self.metrics['maximum_file_edges'],len(nodes)-1);self.metrics['maximum_file_canonical_closure']=max(self.metrics['maximum_file_canonical_closure'],decoded);self.metrics['maximum_file_encoded_closure']=max(self.metrics['maximum_file_encoded_closure'],encoded)
  previous=b'';out=None
  for node,r,n in reversed(nodes):
   role='small' if r[0] in (0,1) else 'large';delta=r[0] in (1,3);header=23 if role=='small' else 21
   previous=codec.decode(r[33:] if delta else r[1:],n-header,previous,18 if role=='small' else 20);out=codec.canonical(role,previous);self._put(node,out);self.metrics['content_canonical_decoded_bytes']+=len(out)
  return out
 def read_canonical(self,identity):
  if isinstance(identity,str):identity=bytes.fromhex(identity)
  if identity in self.cache:self.cache.move_to_end(identity);return self.cache[identity]
  v,r,_=self._physical(identity)
  if v!=107:return super().read_canonical(identity)
  if r[0] in (0,1,2,3):return self._file(identity)
  n=self.loc[identity][3]
  if r[0]==4:
   owner=r[1:33];offset,length=struct.unpack_from('<II',r,33);v,body,_=self._physical(owner);check(v==107 and body[0] in (2,3),'slice owner role');check(length==n-21 and 0<length<=32768 and offset+length<=self.loc[owner][3]-21,'slice extent')
   whole=self.read_canonical(owner);check(whole[13:21]==b'LFSWFL1\0','whole canonical role');raw=whole[21+offset:21+offset+length];self.metrics['slice_reads']+=1;self.metrics['slice_output_bytes']+=length
  else:raw=codec.decode(r[1:],n-21,window=20)
  canonical=codec.canonical('native',raw);self._put(identity,canonical);return canonical

"""Actual-copy reader extension: selected depth1 legacy metadata deltas only."""
import importlib.util,pathlib,sys
HERE=pathlib.Path(__file__).resolve().parent
prior=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/combined/store_api.py')
spec=importlib.util.spec_from_file_location('prior_full157_reader',prior);old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
sys.path.insert(0,str(HERE.parent/'metadata'))
import delta_api
check=old.check

class StoreReader(old.StoreReader):
 def read_canonical(self,identity):
  if isinstance(identity,str):identity=bytes.fromhex(identity)
  if identity in self.cache:self.cache.move_to_end(identity);return self.cache[identity]
  version,record,count=self._physical(identity)
  if version!=1:return super().read_canonical(identity)
  check(record and record[0] in (0,1),'metadata kind')
  if record[0]==0:
   check(0<len(record)-1<=8192,'metadata FULL bound');return self._put(identity,record[1:])
  check(41<len(record)<=8193,'metadata delta bound');base=record[1:33]
  check(base!=identity and base in self.loc,'metadata base identity')
  check(self.loc[base][0]<self.loc[identity][0],'metadata earlier-pack base')
  base_version,base_record,_=self._physical(base)
  check(base_version==1 and base_record[:1]==b'\0','metadata selected FULL base')
  canonical=delta_api.decode_record(identity,{identity:record,base:base_record})
  return self._put(identity,canonical)

d_api=delta_api

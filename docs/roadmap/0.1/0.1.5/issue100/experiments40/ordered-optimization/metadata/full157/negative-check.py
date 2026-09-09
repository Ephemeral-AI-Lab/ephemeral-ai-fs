"""Reject missing/remapped values and corrupt pool bytes using the actual copy."""
import hashlib,json,pathlib,sys
HERE=pathlib.Path(__file__).resolve().parent
from reader import StoreReader,old
inv,*_=old.chain_api.original_inventory();target=next(i for i,b in inv.items() if old.chain_api.e.leaf(b) and len(b)>44)
def sha(p):return hashlib.file_digest(p.open('rb'),'sha256').hexdigest()
source=HERE/'candidate.sqlite';before=sha(source);passed=[]
for name in ('missing_ordinal','remapped_ordinal','corrupt_pool'):
 r=StoreReader(source,cache_bytes=0);original=r._expand
 def intercept(b):
  if old.chain_api.e.leaf(b):
   ordinal=int.from_bytes(b[52:56],'big')
   if name=='missing_ordinal':r.ordinals.pop(ordinal,None)
   elif name=='remapped_ordinal':r.ordinals[ordinal]=next(i for n,i in r.ordinals.items() if i!=r.ordinals[ordinal])
  return original(b)
 if name!='corrupt_pool':r._expand=intercept
 else:
  physical=r._physical
  def changed(i):
   v,b,n=physical(i)
   if r.loc[i][3]==94:b=b[:-1]+bytes([b[-1]^1])
   return v,b,n
  r._physical=changed
 try:
  try:r.read_canonical(target)
  except (AssertionError,ValueError,KeyError):passed.append(name)
  else:raise AssertionError('accepted '+name)
 finally:r.close()
assert sha(source)==before
out=dict(status='PASS',rejected=passed,candidate_sha256=before,script_sha256=sha(pathlib.Path(__file__)))
assert not(HERE/'negative-check.json').exists();(HERE/'negative-check.json').write_text(json.dumps(out,indent=2)+'\n');print(out)

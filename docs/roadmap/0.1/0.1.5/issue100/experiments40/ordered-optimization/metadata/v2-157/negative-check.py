"""Actual reader rejects missing, misrouted, corrupt and wrong-role physical values."""
import bisect,hashlib,json,pathlib,struct
from reader import StoreReader,old
HERE=pathlib.Path(__file__).resolve().parent
inv,*_=old.chain_api.original_inventory();target=next(i for i,b in inv.items() if old.chain_api.e.leaf(b) and len(b)>44)
def sha(p):return hashlib.file_digest(p.open('rb'),'sha256').hexdigest()
path=HERE/'candidate.sqlite';before=sha(path);passed=[]
for name in ('missing_group','wrong_ordinal','bad_group_digest','corrupt_group','wrong_value_role','canonical_leaf_hash'):
 r=StoreReader(path,cache_bytes=0);original=r.pool_value;triggered=False
 def intercept(ordinal):
  global triggered
  if triggered:return original(ordinal)
  triggered=True
  first=r.starts[bisect.bisect_right(r.starts,ordinal)-1];count,p,g,h=r.catalogue[first]
  if name=='missing_group':r.catalogue.pop(first)
  elif name=='wrong_ordinal':return original(ordinal+1 if ordinal<r.value_count else 1)
  elif name=='bad_group_digest':r.catalogue[first]=(count,p,g,bytes([h[0]^1])+h[1:])
  else:
   blob=r._pack(p);groups=int.from_bytes(blob[12:16],'little');encoded=[]
   for group in range(groups):
    off,enc,dec,codec=struct.unpack_from('<IIII',blob,16+16*group);frame=blob[off:off+enc]
    if group==g:
     body=bytearray(old.chain_api.e.codec.decompress(frame,dec) if codec else frame);slot=ordinal-first;start=4+4*count
     if slot:start+=int.from_bytes(body[4+4*(slot-1):8+4*(slot-1)],'little')
     pos=start+14 if name=='wrong_value_role' else start+94
     body[pos]^=1
     if name!='corrupt_group':r.catalogue[first]=(count,p,g,hashlib.sha256(body).digest())
     codec,frame=old.chain_api.e.codec.compress(bytes(body))
    encoded.append((frame,dec,codec))
   offset=16+16*groups;directory=[]
   for frame,dec,codec in encoded:directory.append(struct.pack('<IIII',offset,len(frame),dec,codec));offset+=len(frame)
   changed=blob[:16]+b''.join(directory)+b''.join(frame for frame,_,_ in encoded)
   originalpack=r._pack
   r._pack=lambda identity:changed if identity==p else originalpack(identity)
  return original(ordinal)
 r.pool_value=intercept
 try:
  try:r.read_canonical(target)
  except (AssertionError,ValueError,KeyError):passed.append(name)
  else:raise AssertionError('accepted '+name)
 finally:r.close()
assert sha(path)==before
out=dict(status='PASS',rejected=passed,candidate_sha256=before,script_sha256=sha(pathlib.Path(__file__)));assert not(HERE/'negative-check.json').exists();(HERE/'negative-check.json').write_text(json.dumps(out,indent=2)+'\n');print(out)

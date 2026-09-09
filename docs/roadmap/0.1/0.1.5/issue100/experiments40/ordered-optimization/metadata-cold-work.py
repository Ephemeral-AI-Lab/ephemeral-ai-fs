"""Exact cold leaf dependency/group amplification for the fixed shared pool layout."""
import argparse,collections,hashlib,json,sqlite3,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent
sys.path.insert(0,'/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/metadata')
import codec

def main():
 p=argparse.ArgumentParser();p.add_argument('--history',type=int,choices=(53,157),required=True);args=p.parse_args()
 where=ROOT/'metadata' if args.history==53 else ROOT/'metadata/full157'
 data=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-stride3-structural/metadata/d/cache.sqlite') if args.history==53 else Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-full157/metadata/cache.sqlite')
 db=sqlite3.connect(data.as_uri()+'?mode=ro&immutable=1',uri=True);inv=dict(db.execute('select id,canonical from metadata'));db.close()
 db=sqlite3.connect((where/'cache.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);records=dict(db.execute('select id,record from records'));db.close()
 db=sqlite3.connect((where/'candidate.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);loc={i:(p,g,r) for i,p,g,r in db.execute('select object_id,pack_id,group_number,record_number from objects')};ordinalids={i for i, in db.execute('select object_id from shared_inode_values')}
 groups={};packlength={}
 for pid,b in db.execute('select pack_id,data from object_packs'):
  if b[:8]!=b'LFPACK\0\0' or struct.unpack_from('<I',b,8)[0]!=1:continue
  n=struct.unpack_from('<I',b,12)[0];packlength[pid]=len(b)
  for g in range(n):off,enc,dec,tag=struct.unpack_from('<IIII',b,16+16*g);groups[pid,g]=(enc,dec)
 db.close();valueids={}
 def valueid(v):
  if v not in valueids:valueids[v]=codec.oid(codec.canonical(b'LFSIVL1\0'+v))
  assert valueids[v] in ordinalids
  return valueids[v]
 values={i:{valueid(b[p+8:p+81]) for p in range(44,len(b),81)} for i,b in inv.items() if b[13:21]==b'LFS6INT\0' and b[23]==7}
 maxima=collections.Counter();worst=None
 for target in values:
  chain=[];node=target;seen=set()
  while True:
   assert node not in seen;seen.add(node);chain.append(node);r=records[node]
   if r[0]==0:break
   node=r[1:33]
  pooled=set().union(*(values[i] for i in chain));pg={loc[i][:2] for i in pooled};mg={loc[i][:2] for i in chain};allgroups=pg|mg
  item=dict(target=target.hex(),depth=len(chain)-1,canonical_closure=sum(len(inv[i]) for i in chain),unique_pool_values=len(pooled),pool_groups=len(pg),pool_packs=len({p for p,g in pg}),pool_group_decoded_bytes=sum(groups[g][1] for g in pg),all_group_decoded_bytes=sum(groups[g][1] for g in allgroups),all_group_encoded_bytes=sum(groups[g][0] for g in allgroups),whole_pack_fetch_bytes=sum(packlength[p] for p in {p for p,g in allgroups}))
  for k,v in item.items():
   if isinstance(v,int):maxima[k]=max(maxima[k],v)
  if worst is None or item['all_group_decoded_bytes']>worst['all_group_decoded_bytes']:worst=item
 result=dict(history=args.history,leaves=len(values),maxima=dict(maxima),worst_decoded_group_target=worst,scope='Cold single-leaf reconstruction, shared pool values/groups counted once per target chain; full-group decompression and whole-pack fetch costs. Payload caches may amortize repeated reads, not erase cold costs.')
 (ROOT/f'metadata-cold-{args.history}.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':main()

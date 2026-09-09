"""Replace only v1 per-value CAS/index rows with full-hashed physical group catalogue."""
import hashlib,json,pathlib,sqlite3,struct,time
from reader import StoreReader,V1,old
HERE=pathlib.Path(__file__).resolve().parent
def sha(p):return hashlib.file_digest(pathlib.Path(p).open('rb'),'sha256').hexdigest()
def main():
 started=time.monotonic();source=V1/'candidate.sqlite';v1=json.loads((V1/'result.json').read_text());assert v1['status']=='PASS' and v1['decision']=='REJECT' and sha(source)==v1['candidate_sha256']
 assert not(HERE/'candidate.sqlite').exists() and not(HERE/'result.json').exists()
 db=sqlite3.connect(source.as_uri()+'?mode=ro&immutable=1',uri=True)
 rows=list(db.execute('select v.ordinal,o.object_id,o.canonical_length,o.pack_id,o.group_number,o.record_number from shared_inode_values v join objects o on o.object_id=v.object_id order by v.ordinal'))
 descriptors=[];expected_pool={};members=set()
 for ordinal,i,n,p,g,k in rows:
  assert n==94 and ordinal==len(expected_pool)+1;expected_pool[ordinal]=i;members.add((p,g,k))
  if k==0:
   blob=db.execute('select data from object_packs where pack_id=?',(p,)).fetchone()[0];off,enc,dec,codec=struct.unpack_from('<IIII',blob,16+16*g)
   body=old.chain_api.e.codec.decompress(blob[off:off+enc],dec) if codec else blob[off:off+enc];count=int.from_bytes(body[:4],'little')
   descriptors.append((ordinal,count,p,g,hashlib.sha256(body).digest()))
  first,count,p0,g0,h=descriptors[-1];assert (p,g)==(p0,g0) and k==ordinal-first and k<count
 assert len(rows)==v1['pool_objects'] and descriptors[-1][0]+descriptors[-1][1]-1==len(rows)
 packhashes={p:hashlib.sha256(b).hexdigest() for p,b in db.execute('select pack_id,data from object_packs')}
 poolids=set(expected_pool.values());ordinary=[r for r in db.execute('select * from objects order by object_id') if r[0] not in poolids]
 sql={n:list(db.execute('select * from '+n)) for n in ('layers','commits','branches','layer_stacks','workspace_stages','scope_allocator')}
 out=sqlite3.connect(HERE/'candidate.sqlite');db.backup(out);db.close();out.execute('pragma foreign_keys=on');out.execute('begin')
 out.execute('drop table shared_inode_values');out.executemany('delete from objects where object_id=?',[(i,) for i in expected_pool.values()])
 out.execute('create table pool_groups(first_ordinal INTEGER PRIMARY KEY CHECK(first_ordinal BETWEEN 1 AND 4294967295), count INTEGER NOT NULL CHECK(count BETWEEN 1 AND 166), pack_id INTEGER NOT NULL REFERENCES object_packs(pack_id), group_number INTEGER NOT NULL CHECK(group_number BETWEEN 0 AND 255), group_sha256 BLOB NOT NULL CHECK(length(group_sha256)=32)) STRICT')
 out.executemany('insert into pool_groups values(?,?,?,?,?)',descriptors);out.execute('pragma user_version=9304');out.commit();out.execute('vacuum')
 assert out.execute('pragma integrity_check').fetchone()==('ok',) and not list(out.execute('pragma foreign_key_check'))
 assert list(out.execute('select * from objects order by object_id'))==ordinary
 assert all(list(out.execute('select * from '+n))==r for n,r in sql.items())
 assert {p:hashlib.sha256(b).hexdigest() for p,b in out.execute('select pack_id,data from object_packs')}==packhashes
 pages={n:dict(bytes=b,payload=p,unused=u) for n,b,p,u in out.execute('select name,sum(pgsize),sum(payload),sum(unused) from dbstat group by name')};out.close()
 reader=StoreReader(HERE/'candidate.sqlite');assert reader.value_count==len(rows)
 for ordinal,i in expected_pool.items():assert old.chain_api.e.codec.oid(reader.pool_value(ordinal))==i
 inv,roots,*_=old.chain_api.original_inventory()
 for i,b in inv.items():assert reader.read_canonical(i)==b
 for root in roots:
  table=old.chain_api.e.codec.value(inv[root])[84:116];pairs=old.chain_api.decode_table(reader,table);assert pairs==old.chain_api.decode_table(inv,table)
  for _,v in pairs:
   if v[0]==2:assert old.chain_api.decode_directory(reader,v[9:41])==old.chain_api.decode_directory(inv,v[9:41])
 seen=set()
 for p,b in reader.db.execute('select pack_id,data from object_packs'):
  if int.from_bytes(b[8:12],'little')==1:
   _,groups=old.old.groups(b);seen.update((p,g,k) for g,rs in enumerate(groups) for k in range(len(rs)))
 indexed={(p,g,k) for i,n,p,g,k in ordinary if (p,g,k) in seen};catalogued={(p,g,k) for first,count,p,g,h in descriptors for k in range(count)}
 assert catalogued==members and not indexed&catalogued and seen==indexed|catalogued
 metrics=dict(reader.metrics);reader.close();assert sha(source)==v1['candidate_sha256']
 st=(HERE/'candidate.sqlite').stat();result=dict(status='PASS',decision='PROMISING' if st.st_size<v1['control_bytes'] else 'REJECT',candidate_bytes=st.st_size,candidate_allocated_bytes=st.st_blocks*512,baseline_improved_bytes=v1['control_bytes'],saving_vs_improved=v1['control_bytes']-st.st_size,saving_vs_v1=v1['candidate_bytes']-st.st_size,pool_values=len(rows),pool_groups=len(descriptors),metadata_objects=len(inv),namespace_semantics_verified=len(roots),metadata_pack_bytes=v1['metadata_packs_bytes'],pool_pack_bytes=v1['pool_packs_bytes'],pages=pages,all_original_canonical_objects_and_values='PASS',physical_metadata_group_coverage='PASS',all_packs_byte_equal_v1='PASS',unchanged_content_locators_and_typed_SQL='PASS',reader_metrics=metrics,candidate_sha256=sha(HERE/'candidate.sqlite'),source_v1_sha256=v1['candidate_sha256'],elapsed_seconds=time.monotonic()-started,protocol_sha256=sha(HERE/'protocol.md'))
 (HERE/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2),flush=True)
if __name__=='__main__':main()

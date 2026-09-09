"""Offline unsupported Store copies for SQLite geometry, not public allocation proof."""
import sqlite3,json
from experiment import ROOT,SRC,EXPECTED,sha,check,u32,original,encode
check(sha(SRC)==EXPECTED,'source hash')
source=sqlite3.connect(SRC.as_uri()+'?mode=ro&immutable=1',uri=True)
results={'scope':'Offline complete-copy replacement layout only; diagnostic packs and user_version intentionally unsupported by LayerFS. Filesystem allocation depends on this copy lifecycle. Not benchmark/public Store results.','source_sha256_before_and_after':EXPECTED,'copies':{}}
for version in (3,101,102):
 path=ROOT/('offline-layout-'+str(version)+'.sqlite')
 with path.open('xb'):pass
 copy=sqlite3.connect(path);source.backup(copy)
 if version!=3:
  for p,b in source.execute('select pack_id,data from object_packs'):
   if u32(b,8)==3:copy.execute('update object_packs set data=? where pack_id=?',(encode(original(b),version),p))
  copy.execute('pragma user_version='+str(9000+version));copy.commit()
 copy.execute('vacuum')
 check(copy.execute('pragma integrity_check').fetchone()==('ok',),'SQLite integrity')
 check(copy.execute('pragma foreign_key_check').fetchone() is None,'foreign keys')
 check(copy.execute('select count(*) from objects').fetchone()[0]==80240,'locator count')
 # All object rows identical; streaming digest covers identity, length and every locator.
 import hashlib
 def rows_hash(db):
  h=hashlib.sha256()
  for row in db.execute('select object_id,canonical_length,pack_id,group_number,record_number from objects order by object_id'):h.update(repr(row).encode())
  return h.hexdigest()
 check(rows_hash(source)==rows_hash(copy),'unchanged locator rows')
 stats=dict(zip(('bytes','payload','unused'),copy.execute("select sum(pgsize),sum(payload),sum(unused) from dbstat where name='object_packs'").fetchone()))
 pack_bytes=copy.execute('select sum(length(data)) from object_packs').fetchone()[0]
 copy.close();st=path.stat()
 results['copies'][str(version)]={'path':str(path),'sha256':sha(path),'logical_bytes':st.st_size,'filesystem_allocated_bytes':st.st_blocks*512,'all_pack_bytes':pack_bytes,'pack_btree':stats,'object_rows_identical':True}
source.close();check(sha(SRC)==EXPECTED,'source unchanged')
(ROOT/'layout-result.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(results,indent=2))

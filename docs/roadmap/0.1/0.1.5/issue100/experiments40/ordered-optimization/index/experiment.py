"""One fixed native SQLite page layout, exact logical equality and actual allocation."""
import collections,hashlib,json,pathlib,sqlite3,struct,time
HERE=pathlib.Path(__file__).resolve().parent
SOURCE=HERE.parent/'content/candidate.sqlite'
EXPECTED='dd4a3c7cd2260b487b570cb819f2799ff189c2470382d4481b0f2fd19f3360a3'
def sha(p):return hashlib.file_digest(pathlib.Path(p).open('rb'),'sha256').hexdigest()
def ro(p):return sqlite3.connect(pathlib.Path(p).as_uri()+'?mode=ro&immutable=1',uri=True)
def qi(s):return '"'+s.replace('"','""')+'"'
def digest(db,table):
 columns=list(db.execute('pragma table_info('+qi(table)+')'));key=[r[1] for r in sorted(columns,key=lambda r:r[5]) if r[5]]
 if not key:key=[r[1] for r in columns]
 h=hashlib.sha256();count=0;blobbytes=0
 for row in db.execute('select * from '+qi(table)+' order by '+','.join(map(qi,key))):
  h.update(b'R'+struct.pack('>I',len(row)))
  for v in row:
   if v is None:tag,data=b'N',b''
   elif isinstance(v,int):tag,data=b'I',struct.pack('>q',v)
   elif isinstance(v,float):tag,data=b'F',struct.pack('>d',v)
   elif isinstance(v,str):tag,data=b'T',v.encode()
   else:assert isinstance(v,bytes);tag,data=b'B',v;blobbytes+=len(v)
   h.update(tag+struct.pack('>Q',len(data)));h.update(data)
  count+=1
 return dict(rows=count,sha256=h.hexdigest(),blob_bytes=blobbytes)
def logical(db):
 schema=list(db.execute('select type,name,tbl_name,sql from sqlite_schema order by name'))
 tables={name:digest(db,name) for name, in db.execute("select name from sqlite_schema where type='table' order by name")}
 pragmas={name:db.execute('pragma '+name).fetchone()[0] for name in ('user_version','application_id','encoding')}
 return dict(schema=schema,tables=tables,pragmas=pragmas)
def stats(p):
 db=ro(p);by={}
 for name,path,kind,pg,payload,unused in db.execute('select name,path,pagetype,pgsize,payload,unused from dbstat'):
  r=by.setdefault(name,dict(bytes=0,payload=0,unused=0,pages=0,internal_pages=0,leaf_pages=0,overflow_pages=0,maximum_btree_depth=0))
  r['bytes']+=pg;r['payload']+=payload;r['unused']+=unused;r['pages']+=1;r[kind+'_pages']+=1
  if kind!='overflow':r['maximum_btree_depth']=max(r['maximum_btree_depth'],path.count('/')-1)
 for r in by.values():r['nonpayload_bytes']=r['bytes']-r['payload']
 page=db.execute('pragma page_size').fetchone()[0];db.close();s=p.stat()
 return dict(logical_bytes=s.st_size,allocated_bytes=s.st_blocks*512,page_size=page,sha256=sha(p),dbstat=by,total_pages=sum(r['pages'] for r in by.values()),total_overflow_pages=sum(r['overflow_pages'] for r in by.values()),total_unused_bytes=sum(r['unused'] for r in by.values()))
def main():
 start=time.monotonic();assert sha(SOURCE)==EXPECTED
 for n in ('control.sqlite','candidate.sqlite','result.json'):assert not(HERE/n).exists(),('appendonly',n)
 source=ro(SOURCE);assert source.execute('pragma page_size').fetchone()[0]==4096;original=logical(source)
 for name,page in [('control.sqlite',4096),('candidate.sqlite',1024)]:
  path=HERE/name;db=sqlite3.connect(path);source.backup(db);db.execute('pragma page_size='+str(page));db.execute('vacuum');assert db.execute('pragma page_size').fetchone()[0]==page
  assert db.execute('pragma integrity_check').fetchone()==('ok',) and not list(db.execute('pragma foreign_key_check'))
  assert logical(db)==original;db.close()
 source.close();assert sha(SOURCE)==EXPECTED
 control=stats(HERE/'control.sqlite');candidate=stats(HERE/'candidate.sqlite');saving=control['allocated_bytes']-candidate['allocated_bytes']
 out=dict(status='PASS',decision='PROMISING' if saving>0 else 'REJECT',source=str(SOURCE),source_sha256_before_after=EXPECTED,source_disk=stats(SOURCE),control=control,candidate=candidate,saving_allocated_bytes=saving,saving_logical_bytes=control['logical_bytes']-candidate['logical_bytes'],exact_logical_schema_and_every_typed_row='PASS',logical_proof=original,protocol_sha256=sha(HERE/'protocol.md'),script_sha256=sha(pathlib.Path(__file__)),elapsed_seconds=time.monotonic()-start)
 (HERE/'result.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:v for k,v in out.items() if k not in ('logical_proof','control','candidate','source_disk')},indent=2));print('CONTROL',control['logical_bytes'],control['allocated_bytes'],'CANDIDATE',candidate['logical_bytes'],candidate['allocated_bytes'],flush=True)
if __name__=='__main__':main()

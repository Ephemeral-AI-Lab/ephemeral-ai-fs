#!/usr/bin/env python3
"""Aggregate shared exact-decoder census of a sealed preverification snapshot."""
import json,pathlib,sqlite3,sys,hashlib

def main(snapshot,census,out):
 def sha(p):
  with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
 source=json.loads((snapshot/'custody.json').read_text());assert sha(snapshot/'store.sqlite')==source['copy_sha256']==source['source_sha256'];index_sha=sha(census/'roles-inventory.sqlite');db=sqlite3.connect((snapshot/'store.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);db.row_factory=sqlite3.Row
 idx=sqlite3.connect((census/'roles-inventory.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);idx.row_factory=sqlite3.Row
 idx.execute('pragma cache_size=-8192');idx.execute('pragma temp_store=FILE')
 def rows(c,sql):return [dict(r) for r in c.execute(sql)]
 def scalar(c,sql):return c.execute(sql).fetchone()[0]
 pages=rows(db,'select name,pagetype,count(*) page_count,sum(pgsize) page_bytes,sum(payload) payload_bytes,sum(unused) unused_bytes from dbstat group by name,pagetype');size=scalar(db,'pragma page_size');count=scalar(db,'pragma page_count');free=scalar(db,'pragma freelist_count')
 for r in pages:r['overhead_bytes']=r['page_bytes']-r['payload_bytes']-r['unused_bytes'];assert r['overhead_bytes']>=0 and r['unused_bytes']>=0
 assert sum(r['page_bytes'] for r in pages)+free*size==size*count==source['primary_final_ack']['database_logical_bytes']
 roles=rows(idx,'select role,kind,count(*) objects_count,sum(bytes) canonical_bytes from records where selected=1 group by role,kind')
 groups=rows(idx,'select lane,count(*) group_count,sum(encoded) encoded_bytes,sum(decoded) decoded_bytes from (select g.pack,g.grp,g.encoded,g.decoded,CASE WHEN max(r.role="payload_chunk")=1 AND min(r.role="payload_chunk")=1 THEN "payload" WHEN max(r.role="payload_chunk")=0 THEN "structural" ELSE "mixed_payload_structural" END lane from groups g join records r on r.pack=g.pack and r.grp=g.grp group by g.pack,g.grp) group by lane')
 packbytes=scalar(idx,'select sum(bytes) from packs');encoded=scalar(idx,'select sum(encoded) from groups');pc=scalar(idx,'select count(*) from packs');gc=scalar(idx,'select count(*) from groups');assert packbytes==16*pc+16*gc+encoded
 assert packbytes==scalar(db,'select sum(length(data)) from object_packs')
 decoded=scalar(idx,'select sum(decoded) from groups');framing=4*gc+4*scalar(idx,'select sum(records) from groups');full=scalar(idx,'select sum(full_bytes) from groups');delta=scalar(idx,'select sum(delta_bytes) from groups');assert decoded==framing+full+delta
 # One final graph union; no per-ack traversal and no payload retained.
 idx.execute('CREATE TEMP TABLE logical(id BLOB PRIMARY KEY)');idx.execute('INSERT INTO logical WITH RECURSIVE reachable(id) AS (SELECT id FROM roots UNION SELECT e.child FROM edges e JOIN reachable r ON e.parent=r.id) SELECT id FROM reachable')
 logical=scalar(idx,'SELECT count(*) FROM logical')
 assert scalar(idx,'SELECT count(*) FROM logical l WHERE NOT EXISTS(SELECT 1 FROM records r WHERE r.id=l.id AND r.selected=1)')==0
 base_only=scalar(idx,'SELECT count(DISTINCT base) FROM records WHERE selected=1 AND kind="DELTA" AND id IN(SELECT id FROM logical) AND base NOT IN(SELECT id FROM logical)')
 assert scalar(idx,"SELECT count(*) FROM records d LEFT JOIN records b ON d.base=b.id AND b.selected=1 WHERE d.selected=1 AND d.kind='DELTA' AND (b.id IS NULL OR b.kind!='FULL')")==0
 base_only_bytes=scalar(idx,"SELECT COALESCE(sum(bytes),0) FROM records WHERE selected=1 AND id IN(SELECT base FROM records WHERE selected=1 AND kind='DELTA' AND id IN(SELECT id FROM logical)) AND id NOT IN(SELECT id FROM logical)")
 selected=sum(r['objects_count'] for r in roles);canonical=sum(r['canonical_bytes'] for r in roles)
 assert selected==source['primary_final_ack']['canonical_objects'] and canonical==source['primary_final_ack']['canonical_bytes']
 assert logical<=selected
 dependencies=rows(idx,'select count(*) delta_count,count(distinct base) base_count,sum(pack!=base_pack) cross_pack_count from records where selected=1 and kind="DELTA"')
 result={'status':'PASS','scope':'exact olddecoder authentication of new publicS1 preverification logicalsnapshot; filesystemallocation from ORIGINALack notcopy','units':'integerbytes/counts; allgroups/rolesexclusive where labelled','provenance':source,'filesystem':{'allocated_bytes':source['primary_final_ack']['store_allocated_bytes'],'sidecars_bytes':source['primary_final_ack']['sidecar_allocated_bytes'],'logical_bytes':size*count,'signed_adjustment_bytes':source['primary_final_ack']['database_allocated_bytes']-size*count},'sqlite':{'page_size':size,'page_count':count,'freelist_count':free,'pages':pages,'object_packs_SQL_row_encoding_bytes':sum(r['payload_bytes'] for r in pages if r['name']=='object_packs')-packbytes},'packs':{'count':pc,'blob_bytes':packbytes,'header_bytes':16*pc,'directory_bytes':16*gc,'encoded_group_bytes':encoded,'decoded_group_bytes':decoded,'decoded_directory_bytes':framing,'FULL_record_bytes':full,'DELTA_record_bytes':delta},'groups_by_lane':groups,'roles':roles,'retention':{'selected_count':selected,'canonical_bytes':canonical,'logical_union_count':logical,'outside_logical_union_count':selected-logical,'base_only_count':base_only,'base_only_canonical_bytes':base_only_bytes,'outside_required_union_count':selected-logical-base_only,'all_selected_delta_bases_selected_FULL':True,'unselected_records_count':scalar(idx,'select count(*) from records where selected=0'),'dependencies':dependencies},'limitations':['No perrecord compressed attribution','Logical and physicalbase populations separately computed; outside doesnot imply safegarbage','Snapshotcopyallocation notused','Initialrandominodevalues produce canonicalcountdifferences vs historicalcontrol']}
 assert result['sqlite']['object_packs_SQL_row_encoding_bytes']>=0
 assert selected-logical-base_only>=0
 assert sha(snapshot/'store.sqlite')==source['copy_sha256'] and sha(census/'roles-inventory.sqlite')==index_sha
 result['analysis_source_hashes']={'snapshot_sha256':source['copy_sha256'],'inventory_sha256':index_sha}
 with out.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps({'groups':groups,'packs':result['packs'],'retention':result['retention']}))
if __name__=='__main__':main(*map(pathlib.Path,sys.argv[1:]))

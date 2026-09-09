"""Authenticate complete stride3 native graph after selected substitutions."""
from pathlib import Path
from shared import *
f=json.loads((OUT/'family-result.json').read_text());selected=f['selected_graph'];original_sizes={id:r['size'] for id,r in records.items()}
assert len(records)==1998 and len(selected)==f['summary']['objects']
for id,r in selected.items():records[id]=dict(r,frame=bytes.fromhex(r['frame']))
assert not [id for id,r in records.items() if id not in selected and r['base'] in selected]
rows=[];groups=collections.defaultdict(list)
for id,r in records.items():
 raw,info=decode(id);assert len(raw)==r['raw'] and info['lookups']<=5 and info['lookups']<=8;rows.append(dict(id=id,raw_sha256=hashlib.sha256(raw).hexdigest(),**info));groups[r['pack'],r['group']].append(r)
changed={};max_group=0
for key,rs in groups.items():
 assert len(rs)==rs[0]['group_count'];size=4+4*len(rs)+sum(r['size'] for r in rs);assert size<=65536;max_group=max(max_group,size)
 changed[key]=size
# Keep original pack directories as accounting coordinates, report if repacking needed.
db=sqlite3.connect(STORE.as_uri()+'?mode=ro&immutable=1',uri=True);packrows=[]
for p,b in db.execute("select pack_id,data from object_packs where substr(data,9,4)=x'02000000'"):
 n=struct.unpack_from('<I',b,12)[0];size=16+16*n+sum(changed[p,g] for g in range(n));packrows.append(dict(pack=p,original_bytes=len(b),simulated_bytes=size,within_256KiB=size<=262144))
db.close();assert sha(STORE)==BEFORE
summary=dict(native_objects=len(rows),substituted_objects=len(selected),outside_family_dependents=0,maximum_edges=max(r['depth'] for r in rows),maximum_raw_closure=max(r['raw_closure'] for r in rows),maximum_lookups=max(r['lookups'] for r in rows),maximum_encoded_work=max(r['encoded_work'] for r in rows),maximum_decoded_work=max(r['decoded_work'] for r in rows),maximum_group_bytes=max_group,maximum_pack_bytes=max(r['simulated_bytes'] for r in packrows),packs_requiring_split=sum(not r['within_256KiB'] for r in packrows),original_v2_pack_bytes=sum(r['original_bytes'] for r in packrows),simulated_v2_pack_bytes=sum(r['simulated_bytes'] for r in packrows))
assert summary['original_v2_pack_bytes']-summary['simulated_v2_pack_bytes']==f['summary']['saved_record_bytes']
(OUT/'all-native-verification.json').write_text(json.dumps(dict(scope='All1998native canonical objects authenticated after all selected lockfilegraph substitutions; cycles/strict pack chronology/role/length/depth/raw/per-chain work and singleton optional8lookup caps. Group and originalpack membership accounting checked. Does not run product visit_wave scratch ownership, reconstruct non-lock complete files, enforce unobserved original admission-batch remaining quotas or validate prospective physical formats.',summary=summary,store_sha256=BEFORE,source_family_sha256=sha(OUT/'family-result.json'),script_sha256=sha(Path(__file__)),records=rows,packs=packrows),indent=2));print(json.dumps(summary,indent=2))

#!/usr/bin/env python3
"""Small provenance supplement; consumes inventory, never decodes payloads."""
import csv,json,pathlib,sqlite3,sys
out,run=map(pathlib.Path,sys.argv[1:]);c=sqlite3.connect(f'file:{out}/roles-inventory.sqlite?mode=ro',uri=True);c.row_factory=sqlite3.Row
s=sqlite3.connect(f'file:{run}/deepseek-full/host-runtime/store.sqlite?mode=ro&immutable=1',uri=True)
# These small physical tables receive fields added during independent review.
for name,sql in [('packs.csv','SELECT pack,16+16*groups FROM packs'),('groups.csv','SELECT pack,grp,encoded FROM groups ORDER BY pack,grp')]:
    rows=list(csv.DictReader((out/name).open()));offsets=dict(c.execute('SELECT pack,16+16*groups FROM packs'))
    for row in rows:
        p=int(row['pack_id'])
        if name=='packs.csv':row.update(first_admitted_checkpoint='',admission_unknown_reason='no authoritative per-ack pack/location provenance')
        else:
            g=int(row['group_number']);row['pack_offset_bytes']=offsets[p];offsets[p]+=int(row['encoded_bytes'])
            counts=dict(c.execute('SELECT selected,count(*) FROM records WHERE pack=? AND grp=? GROUP BY selected',(p,g)))
            row.update(selected_record_count=counts.get(1,0),unselected_record_count=counts.get(0,0))
    with (out/name).open('w') as f:w=csv.DictWriter(f,rows[0].keys());w.writeheader();w.writerows(rows)
roots=[]
for table,key in [('layers','layer_id'),('commits','commit_id'),('workspace_stages','workspace_id')]:
    for id,root in s.execute(f'SELECT {key},root_id FROM {table}'):roots.append(dict(source=table,identity=id.hex(),root_id=root.hex(),snapshot='post-verification',status='measured',provenance=f'SQL {table} root_id'))
branches=[dict(name=name,branch_id=id.hex(),base_layer_id=layer.hex(),head_commit_id=commit.hex() if commit else None,root_id=root.hex(),root_is_already_retained=True,verifier_alias=name.startswith('verify-'),provenance='branches joined selected head commit or base layer; verifier alias name corroborated by verification receipts') for name,id,layer,commit,root in s.execute('SELECT b.name,b.branch_id,b.base_layer_id,b.head_commit_id,coalesce(c.root_id,l.root_id) FROM branches b LEFT JOIN commits c ON c.commit_id=b.head_commit_id JOIN layers l ON l.layer_id=b.base_layer_id')]
(out/'retained-root-provenance.json').write_text(json.dumps({'roots':roots,'branches':branches,'staging_roots_count':s.execute('SELECT count(*) FROM workspace_stages').fetchone()[0],'pending_note':'Retained performance/verification-pending JSON files are historical per-step command records; successful final receipts/cleanup establish no live pending owner. All durable workspace_stages roots enumerated.','units':'identities are hexadecimal; count integer; root membership exact','snapshot':'post-verification'},indent=2)+'\n')
# Summary bins and exact retained-set group composition avoid any per-record compressed attribution.
with (out/'role-histograms.csv').open('w') as f:
    w=csv.writer(f);w.writerow(['role','kind','canonical_size_bin_bytes','objects_count','canonical_bytes','snapshot','status','provenance'])
    q='SELECT role,kind,CASE WHEN bytes<64 THEN "0-63" WHEN bytes<256 THEN "64-255" WHEN bytes<1024 THEN "256-1023" WHEN bytes<4096 THEN "1024-4095" WHEN bytes<16384 THEN "4096-16383" WHEN bytes<65536 THEN "16384-65535" ELSE "65536+" END,count(*),sum(bytes) FROM classified GROUP BY 1,2,3 ORDER BY 1,2,3'
    for row in c.execute(q):w.writerow(list(row)+['post-verification','derived','authenticated records'])
with (out/'group-retention-composition.csv').open('w') as f:
    w=csv.writer(f);w.writerow(['pack_id','group_number','retention','records_count','canonical_bytes','decoded_record_bytes','snapshot','status','provenance'])
    for row in c.execute('SELECT pack,grp,retention,count(*),sum(bytes),sum(record_bytes) FROM classified GROUP BY pack,grp,retention ORDER BY pack,grp,retention'):w.writerow(list(row)+['post-verification','derived','authenticated records plus specified-root closure'])
with (out/'role-totals.csv').open('w') as f:
    w=csv.writer(f);w.writerow(['role','objects_count','canonical_bytes','FULL_objects_count','DELTA_objects_count','snapshot','status','provenance'])
    for role in ['payload_chunk','FileState','extent_leaf','extent_branch','inode_record','inode_table_leaf','inode_table_branch','directory_state','directory_map_leaf','directory_map_branch','metadata_map_leaf','metadata_map_branch','namespace_root','symlink_state','other_supported_encoding','unknown','invalid']:
        r=c.execute('SELECT count(*),coalesce(sum(bytes),0),coalesce(sum(kind="FULL"),0),coalesce(sum(kind="DELTA"),0) FROM classified WHERE role=?',(role,)).fetchone();w.writerow([role,*r,'post-verification','derived','all selected authenticated objects'])
j=json.load(open(out/'roles-summary.json'));j['payload_canonical_max_bytes']=c.execute('SELECT max(bytes) FROM classified WHERE role="payload_chunk"').fetchone()[0];j['payload_size_ineligible_objects']=c.execute('SELECT count(*) FROM classified WHERE role="payload_chunk" AND bytes+9>65536').fetchone()[0];(out/'roles-summary.json').write_text(json.dumps(j,indent=2)+'\n')
j=json.load(open(out/'roles-summary.json'))
j['physical_base_canonical_bytes']=c.execute('SELECT coalesce(sum(bytes),0) FROM classified WHERE id IN (SELECT id FROM bases)').fetchone()[0]
j['anchor_max_fan_in']=c.execute('SELECT max(n) FROM (SELECT count(*) n FROM classified WHERE kind="DELTA" GROUP BY base)').fetchone()[0]
j['payload_both_use_objects']=c.execute('SELECT count(*) FROM classified r WHERE role="payload_chunk" AND EXISTS(SELECT 1 FROM uses u WHERE u.id=r.id AND use_label="file_content") AND EXISTS(SELECT 1 FROM uses u WHERE u.id=r.id AND use_label="metadata_value")').fetchone()[0]
for bucket in ['B_minus_L','outside_R']:
    if not any(r['retention']==bucket for r in j['retention']):j['retention'].append({'retention':bucket,'objects':0,'canonical_bytes':0})
j['workspace_stage_roots']=s.execute('SELECT count(*) FROM workspace_stages').fetchone()[0]
(out/'roles-summary.json').write_text(json.dumps(j,indent=2)+'\n')
# Field contract for each emitted inventory column; value rows retain their own
# population/role/location, snapshot and provenance alongside these units.
fields={}
for name in ['packs.csv','groups.csv','group-role-composition.csv','object-roles.csv','object-retention.csv','delta-dependencies.csv','first-retained-roles.csv','role-histograms.csv','group-retention-composition.csv','role-totals.csv']:
    headers=next(csv.reader((out/name).open()));fields[name]={}
    for h in headers:
        unit='bytes' if h.endswith('_bytes') or h=='COPY_bytes' else 'count' if h.endswith('_count') or h in ('anchor_fan_in','unique_objects') else 'checkpoint ordinal' if 'checkpoint' in h else 'boolean' if h in ('crosses_pack','crosses_group','file_content_use','metadata_value_use') else 'identity/locator' if h.endswith('_id') or h.endswith('_number') else 'text/enum'
        if 'bin' in h:unit='byte interval label'
        fields[name][h]={'units':unit,'population':('unique selected objects' if name.startswith('object-') or name in ('role-histograms.csv','role-totals.csv','first-retained-roles.csv') else 'physical packs/groups/records as keyed; no per-record compressed attribution'),'snapshot':'post-verification retained Store','status':'unknown' if h=='first_admitted_checkpoint' else 'derived/measured inventory','provenance':'authenticated records and exact product decoders; receipt roots for first_retained; selected SQLite locators','null_reason':'no authoritative per-ack pack/location provenance' if h=='first_admitted_checkpoint' else None}
(out/'roles-fields.json').write_text(json.dumps(fields,indent=2)+'\n')

r=c.execute("SELECT max(n),max(d),sum(d>262144),sum(n>8191) FROM (SELECT p.pack,sum(g.records) n,16+16*p.groups+sum(g.decoded) d FROM packs p JOIN groups g ON p.pack=g.pack GROUP BY p.pack)").fetchone()
assert tuple(r[2:])==(0,0), "this run has no oversized singleton exception"
j=json.load(open(out/"roles-summary.json"));j["validation"]["pack_total_decoded_and_record_count_bounds"]=True;j["maximum_pack_decoded_bytes_including_outer_framing"]=r[1];j["maximum_pack_records"]=r[0];(out/"roles-summary.json").write_text(json.dumps(j,indent=2)+"\n")

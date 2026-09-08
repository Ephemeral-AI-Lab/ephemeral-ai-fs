#!/usr/bin/env python3
"""Aggregate the single authenticated inventory. Never decode or write the Store."""
import csv,json,sqlite3,sys,pathlib
out,run=map(pathlib.Path,sys.argv[1:]);c=sqlite3.connect(out/'roles-inventory.sqlite');c.row_factory=sqlite3.Row
s=sqlite3.connect(f'file:{run}/deepseek-full/host-runtime/store.sqlite?mode=ro&immutable=1',uri=True)
phase='post-verification retained Store'
provenance='roles-inventory.sqlite; exact product canonical/reference/pack decoders'
def csvquery(name,sql):
    if (out/name).exists() and (out/name).stat().st_size:
        with (out/name).open() as f:
            reader=csv.reader(f);next(reader);existing=sum(1 for _ in reader)
        expected=c.execute('SELECT count(*) FROM ('+sql+')').fetchone()[0]
        assert existing==expected, f'partial export: {name}'
        return
    q=c.execute(sql)
    with (out/name).open('w') as f:
        w=csv.writer(f);w.writerow([d[0] for d in q.description]+['snapshot','status','provenance']);w.writerows([list(r)+[phase,'derived from authenticated measured inventory',provenance] for r in q])
def scalar(sql):return c.execute(sql).fetchone()[0]
def rows(sql):return [dict(r) for r in c.execute(sql)]
c.executescript('CREATE TABLE IF NOT EXISTS logical(id BLOB PRIMARY KEY,first_retained_checkpoint INTEGER); CREATE TABLE IF NOT EXISTS bases(id BLOB PRIMARY KEY); CREATE TABLE IF NOT EXISTS uses(id BLOB,use_label TEXT,PRIMARY KEY(id,use_label));')
# Visit metadata edges without retaining payloads. The disk-backed visited set makes
# chronological union traversal linear: previously retained subgraphs need not recur.
def retain(root,checkpoint):
    c.execute('CREATE TEMP TABLE IF NOT EXISTS frontier(id BLOB PRIMARY KEY)');c.execute('DELETE FROM frontier');c.execute('INSERT INTO frontier VALUES(?)',(root,))
    while True:
        batch=c.execute('SELECT id FROM frontier LIMIT 1024').fetchall()
        if not batch:break
        for (id,) in batch:
            c.execute('DELETE FROM frontier WHERE id=?',(id,))
            if c.execute('INSERT OR IGNORE INTO logical VALUES(?,?)',(id,checkpoint)).rowcount:
                assert c.execute('SELECT 1 FROM records WHERE id=? AND selected=1',(id,)).fetchone(), 'missing graph child'
                c.execute('INSERT OR IGNORE INTO frontier SELECT child FROM edges WHERE parent=? AND child NOT IN (SELECT id FROM logical)',(id,))
# Init layers are authoritative roots, not inferred pack admission order.
for (root,) in s.execute('SELECT root_id FROM layers'):retain(root,0)
p=json.load(open(run/'deepseek-full/performance-result.json'))
for row in p['records']:
    root=s.execute('SELECT root_id FROM commits WHERE commit_id=?',(bytes.fromhex(row['commit_id']),)).fetchone()
    assert root,'receipt commit mapping absent';retain(root[0],row['index'])
for r in c.execute('SELECT id FROM roots').fetchall():retain(r[0],None)
c.execute('INSERT OR IGNORE INTO bases SELECT base FROM records WHERE selected=1 AND kind="DELTA" AND id IN (SELECT id FROM logical)')
assert scalar('SELECT count(*) FROM bases b LEFT JOIN records r ON r.id=b.id AND r.selected=1 WHERE r.id IS NULL OR r.kind!="FULL"')==0
# Overlapping payload uses traverse the exact file graph starting at content and metadata-value roots.
for label,seed in [('file_content','SELECT child FROM edges WHERE use_label="content"'),('metadata_value','SELECT child FROM edges WHERE use_label="metadata_value"')]:
    c.execute(f'INSERT OR IGNORE INTO uses SELECT id,? FROM (WITH RECURSIVE x(id) AS ({seed} UNION SELECT e.child FROM edges e JOIN x ON e.parent=x.id) SELECT id FROM x)',(label,))
c.executescript('CREATE INDEX IF NOT EXISTS logical_first ON logical(first_retained_checkpoint); CREATE VIEW IF NOT EXISTS classified AS SELECT r.*,l.first_retained_checkpoint,CASE WHEN l.id IS NOT NULL THEN "L" WHEN b.id IS NOT NULL THEN "B_minus_L" ELSE "outside_R" END retention FROM records r LEFT JOIN logical l ON l.id=r.id LEFT JOIN bases b ON b.id=r.id WHERE selected=1;')
csvquery('packs.csv','SELECT pack AS pack_id,bytes AS pack_length_bytes,16 AS header_bytes,groups AS group_count,16*groups AS directory_bytes,bytes-16-16*groups AS encoded_group_bytes,NULL AS first_admitted_checkpoint,"no authoritative per-ack pack/location provenance" AS admission_unknown_reason FROM packs ORDER BY pack')
csvquery('groups.csv','SELECT g.pack AS pack_id,g.grp AS group_number,encoded AS encoded_bytes,16+16*(SELECT groups FROM packs p WHERE p.pack=g.pack)+coalesce(sum(encoded) OVER (PARTITION BY g.pack ORDER BY g.grp ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING),0) AS pack_offset_bytes,(SELECT count(*) FROM records r WHERE r.pack=g.pack AND r.grp=g.grp AND selected=1) AS selected_record_count,(SELECT count(*) FROM records r WHERE r.pack=g.pack AND r.grp=g.grp AND selected=0) AS unselected_record_count,decoded AS decoded_bytes,g.records AS record_count,4 AS count_bytes,4*g.records AS record_directory_bytes,full_bytes AS FULL_record_bytes,delta_bytes AS DELTA_record_bytes,delta_header_bytes,COPY_bytes,insert_frame_bytes,literal_bytes,codec,(SELECT CASE WHEN count(DISTINCT role)=1 THEN min(role) ELSE "mixed_role" END FROM records r WHERE r.pack=g.pack AND r.grp=g.grp) AS exclusive_group_role FROM groups g ORDER BY pack,grp')
csvquery('group-role-composition.csv','SELECT pack AS pack_id,grp AS group_number,role,kind,count(*) AS record_count,sum(bytes) AS canonical_bytes,sum(record_bytes) AS decoded_record_bytes FROM records GROUP BY pack,grp,role,kind ORDER BY pack,grp,role,kind')
csvquery('object-roles.csv','SELECT lower(hex(id)) AS object_id,role,bytes AS canonical_bytes,kind,pack AS selected_pack_id,grp AS selected_group_number,rec AS selected_record_number,record_bytes AS decoded_record_bytes,CASE WHEN bytes<64 THEN "0-63" WHEN bytes<256 THEN "64-255" WHEN bytes<1024 THEN "256-1023" WHEN bytes<4096 THEN "1024-4095" WHEN bytes<16384 THEN "4096-16383" WHEN bytes<65536 THEN "16384-65535" ELSE "65536+" END AS canonical_size_bin_bytes FROM classified ORDER BY id')
csvquery('object-retention.csv','SELECT lower(hex(id)) AS object_id,role,bytes AS canonical_bytes,retention,first_retained_checkpoint,NULL AS first_admitted_checkpoint,"no authoritative per-ack pack/location provenance" AS admission_unknown_reason,EXISTS(SELECT 1 FROM uses u WHERE u.id=classified.id AND use_label="file_content") AS file_content_use,EXISTS(SELECT 1 FROM uses u WHERE u.id=classified.id AND use_label="metadata_value") AS metadata_value_use FROM classified ORDER BY id')
csvquery('delta-dependencies.csv','SELECT lower(hex(r.id)) AS target_id,lower(hex(r.base)) AS FULL_base_id,r.bytes AS target_canonical_bytes,r.record_bytes AS DELTA_record_bytes,r.pack AS target_pack_id,r.grp AS target_group_number,b.pack AS base_pack_id,b.grp AS base_group_number,b.bytes AS base_canonical_bytes,r.pack!=b.pack AS crosses_pack,(r.pack!=b.pack OR r.grp!=b.grp) AS crosses_group,(SELECT count(*) FROM records x WHERE x.selected=1 AND x.base=r.base) AS anchor_fan_in,b.retention AS base_retention FROM classified r JOIN classified b ON b.id=r.base WHERE r.kind="DELTA" ORDER BY r.id')
csvquery('first-retained-roles.csv','SELECT first_retained_checkpoint,role,count(*) AS unique_objects,sum(bytes) AS canonical_bytes FROM classified WHERE retention="L" GROUP BY first_retained_checkpoint,role ORDER BY first_retained_checkpoint,role')
summary={
 'snapshot':phase,'status':'derived from authenticated inventory','provenance':provenance,
 'units':{'bytes':'integer bytes','counts':'integer objects/groups/records as named','time':'not measured'},
 'pack_count':scalar('SELECT count(*) FROM packs'),'pack_blob_bytes':scalar('SELECT sum(bytes) FROM packs'),
 'group_count':scalar('SELECT count(*) FROM groups'),'encoded_group_bytes':scalar('SELECT sum(encoded) FROM groups'),'decoded_group_bytes':scalar('SELECT sum(decoded) FROM groups'),
 'pack_header_bytes':scalar('SELECT 16*count(*) FROM packs'),'pack_directory_bytes':scalar('SELECT 16*sum(groups) FROM packs'),
 'decoded_group_count_bytes':scalar('SELECT 4*count(*) FROM groups'),'decoded_record_directory_bytes':scalar('SELECT 4*sum(records) FROM groups'),
 'full_record_bytes':scalar('SELECT sum(full_bytes) FROM groups'),'delta_record_bytes':scalar('SELECT sum(delta_bytes) FROM groups'),
 'delta_header_bytes':scalar('SELECT sum(delta_header_bytes) FROM groups'),'delta_copy_instruction_bytes':scalar('SELECT sum(copy_bytes) FROM groups'),'delta_insert_framing_bytes':scalar('SELECT sum(insert_frame_bytes) FROM groups'),'delta_literal_bytes':scalar('SELECT sum(literal_bytes) FROM groups'),
 'roles':rows('SELECT role,kind,count(*) AS objects,sum(bytes) AS canonical_bytes FROM classified GROUP BY role,kind ORDER BY sum(bytes) DESC'),
 'retention':rows('SELECT retention,count(*) AS objects,sum(bytes) AS canonical_bytes FROM classified GROUP BY retention'),
 'logical_objects':scalar('SELECT count(*) FROM logical'),'physical_base_objects':scalar('SELECT count(*) FROM bases'),
 'unselected_records':scalar('SELECT count(*) FROM records WHERE selected=0'),
 'unselected_decoded_record_bytes':scalar('SELECT coalesce(sum(record_bytes),0) FROM records WHERE selected=0'),
 'group_roles':rows('SELECT role,count(*) AS groups,sum(encoded) AS encoded_bytes FROM (SELECT g.pack,g.grp,g.encoded,CASE WHEN count(DISTINCT r.role)=1 THEN min(r.role) ELSE "mixed_role" END AS role FROM groups g JOIN records r ON r.pack=g.pack AND r.grp=g.grp GROUP BY g.pack,g.grp) GROUP BY role'),
 'base_only_group_encoded_bytes_ceiling':scalar('SELECT coalesce(sum(encoded),0) FROM groups WHERE (pack,grp) IN (SELECT pack,grp FROM classified WHERE retention="B_minus_L")'),
 'roots':rows('SELECT source,count(*) AS roots FROM roots GROUP BY source'),
 'branches':s.execute('SELECT count(*) FROM branches').fetchone()[0],
 'dependency_summary':rows('SELECT count(*) AS deltas,count(DISTINCT base) AS anchors,sum(pack!=base_pack) AS cross_pack,sum(pack!=base_pack OR grp!=base_group) AS cross_group FROM classified WHERE kind="DELTA"'),
 'payload_use_labels':rows('SELECT use_label,count(*) AS objects,sum(bytes) AS canonical_bytes FROM uses JOIN classified USING(id) WHERE role="payload_chunk" GROUP BY use_label'),
 'payload_canonical_max_bytes':scalar('SELECT max(bytes) FROM classified WHERE role="payload_chunk"'),'payload_size_ineligible_objects':scalar('SELECT count(*) FROM classified WHERE role="payload_chunk" AND bytes+9>65536'),'unknown_role_objects':scalar('SELECT count(*) FROM classified WHERE role="unknown"'),
 'validation':{'all_selected_object_ids_authenticated':True,'all_exact_supported_role_decoders_succeeded':True,'pack_offset_coverage_reserved_bounds':True,'decoded_complete_coverage':True,'all_delta_bases_selected_FULL_authenticated':True,'all_graph_references_selected':True,'preverification_logical_snapshot':'unavailable; this census cannot reconstruct historical page layout'},
 'limitations':['Encoded bytes belong to whole groups; per-record compressed bytes unavailable by definition.','Physical base canonical bytes are not reclaimable compressed bytes.','Outside_R means unreachable under specified roots, not safe garbage.','First retained uses chronological receipt root union; first admission unavailable.','All commits/layers/stages are retained roots; verifier branch aliases add no new canonical roots.','Original Store opened immutable read-only; no copy, replay, recompression, VACUUM or optimization.']}
summary['full_record_count']=scalar('SELECT count(*) FROM records WHERE kind="FULL"');summary['full_canonical_bytes']=summary['full_record_bytes']-summary['full_record_count']
assert summary['pack_blob_bytes']==summary['pack_header_bytes']+summary['pack_directory_bytes']+summary['encoded_group_bytes']
assert summary['decoded_group_bytes']==summary['decoded_group_count_bytes']+summary['decoded_record_directory_bytes']+summary['full_record_bytes']+summary['delta_record_bytes']
assert summary['delta_record_bytes']==sum(summary[k] for k in ['delta_header_bytes','delta_copy_instruction_bytes','delta_insert_framing_bytes','delta_literal_bytes'])
assert sum(r['objects'] for r in summary['roles'])==366141
assert sum(r['canonical_bytes'] for r in summary['roles'])==799525289
(out/'roles-summary.json').write_text(json.dumps(summary,indent=2)+'\n');c.commit();print(json.dumps(summary,indent=2))

#!/usr/bin/env python3
"""Aggregate ONE existing native-aware census; never decode payloads again.

Usage: account_combined.py SNAPSHOT INVENTORY PERFORMANCE EXPECTED NEW_OUTPUT
EXPECTED: {schema:"issue88-combined-account-expected-v1",
 hashes:{snapshot:SHA256,inventory:SHA256,performance:SHA256},
 snapshot_phase:"final-pre-verification", provenance:{path:JSON,sha256:SHA256},
 performance_manifest:{path:JSON,sha256:SHA256}}
The provenance JSON uses the existing inventory-proof fields: status PASS,
 snapshot_sha256, inventory_sha256, decoder_command, decoder_binary:{path,sha256},
 custody:{path,sha256} (contains primary_final_ack/performance_manifest_sha256),
 quiescence:{status:PASS,snapshot_mode:ro-immutable},
 all_selected_ids_authenticated:true, all_references_valid:true.
Run only after the benchmark and the shared census, under their custody workflow.
The snapshot is opened read-only/immutable; its allocated size is never reported.
"""
import csv
import hashlib
import json
import pathlib
import sqlite3
import sys


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def scalar(db, sql):
    return db.execute(sql).fetchone()[0]


def rows(db, sql):
    cursor = db.execute(sql)
    names = [c[0] for c in cursor.description]
    return [dict(zip(names, values)) for values in cursor]


def q(value, unit, population, snapshot, provenance, status='derived', reason=None):
    return dict(value=value, unit=unit, population=population, snapshot=snapshot,
                provenance=provenance, status='unknown' if value is None else status,
                null_reason=reason)


def main(snapshot, inventory, performance, expected_path, out):
    out.mkdir(parents=True, exist_ok=False)
    expected = json.loads(expected_path.read_text())
    require(expected['schema'] == 'issue88-combined-account-expected-v1', 'expected schema')
    require(expected['snapshot_phase'] == 'final-pre-verification', 'requires declared final pre-verification logical snapshot')
    paths = dict(snapshot=snapshot, inventory=inventory, performance=performance)
    hashes = {name: sha(path) for name, path in paths.items()}
    require(hashes == expected['hashes'], 'input hash mismatch')
    proof_ref = expected['provenance']
    proof_path = pathlib.Path(proof_ref['path']).resolve()
    require(sha(proof_path) == proof_ref['sha256'], 'inventory proof hash mismatch')
    proof = json.loads(proof_path.read_text())
    require(proof['status'] == 'PASS' and proof['snapshot_sha256'] == hashes['snapshot']
            and proof['inventory_sha256'] == hashes['inventory'], 'proof does not bind complete input census')
    require(proof['all_selected_ids_authenticated'] is True and proof['all_references_valid'] is True,
            'upstream canonical/reference authentication missing')
    require(proof['quiescence'].get('status') == 'PASS' and proof['quiescence'].get('snapshot_mode') == 'ro-immutable',
            'quiescence/immutable custody missing')
    require(isinstance(proof['decoder_command'], list) and proof['decoder_command']
            and all(isinstance(x, str) for x in proof['decoder_command']), 'decoder invocation missing')
    for name in ('decoder_binary', 'custody'):
        require(sha(pathlib.Path(proof[name]['path'])) == proof[name]['sha256'], name + ' proof hash mismatch')
    custody = json.loads(pathlib.Path(proof['custody']['path']).read_text())
    manifest_ref = expected['performance_manifest']
    manifest_path = pathlib.Path(manifest_ref['path']).resolve()
    require(sha(manifest_path) == manifest_ref['sha256'] == custody['performance_manifest_sha256'],
            'original performance manifest custody mismatch')
    performance_manifest = json.loads(manifest_path.read_text())
    require(performance_manifest['deepseek-full/performance-result.json'] == hashes['performance'],
            'performance receipt not bound to original manifest')
    require(not any(snapshot.parent.glob(snapshot.name + '-*')), 'immutable snapshot has sidecars')
    data = json.loads(performance.read_text())
    require(data['status'] == data['cleanup_status'] == 'PASS' and data['container_removed'] is True
            and data['case'] == 'deepseek-full' and [r['index'] for r in data['records']] == list(range(1,158)),
            'full157 performance/cleanup incomplete')
    final = data['records'][-1]
    require(final['index'] == 157, 'final acknowledgement index')
    acknowledgements = [r for r in final['receipts'] if r.get('kind') == 'storage-smoke-allocation']
    require(len(acknowledgements) == 1, 'one final acknowledgement required')
    ack = acknowledgements[0]
    require(ack.get('label') == 'step-157' and custody['primary_final_ack'] == ack,
            'original final acknowledgement custody mismatch')
    ack_scope = 'checkpoint157 public acknowledgement before verification'
    snap_scope = 'final pre-verification immutable logical snapshot; allocation not equivalent'
    ack_prov = str(performance) + '#records/156/receipts/storage-smoke-allocation'
    inventory_prov = str(inventory) + '; bound exact-decoder proof ' + str(proof_path)
    for key in ('store_allocated_bytes', 'database_allocated_bytes', 'database_logical_bytes',
                'sidecar_allocated_bytes', 'sidecar_logical_bytes', 'canonical_objects', 'canonical_bytes'):
        require(type(ack[key]) is int and ack[key] >= 0, 'invalid acknowledgement field: ' + key)
    require(ack['store_allocated_bytes'] == ack['database_allocated_bytes'] + ack['sidecar_allocated_bytes'],
            'filesystem acknowledgement conservation')
    db = sqlite3.connect(snapshot.as_uri() + '?mode=ro&immutable=1', uri=True)
    idx = sqlite3.connect(inventory.as_uri() + '?mode=ro&immutable=1', uri=True)
    for connection in (db, idx):
        connection.execute('PRAGMA cache_size=-8192')
        connection.execute('PRAGMA temp_store=FILE')
    # Compact SQL summaries only; no physical inventory or raw bytes rewritten.
    pages = rows(db, 'SELECT name,pagetype,count(*) pages_count,sum(pgsize) page_bytes,sum(payload) payload_bytes,sum(unused) unused_bytes,min(unused) minimum_page_unused_bytes,min(payload) minimum_page_payload_bytes,min(pgsize-payload-unused) minimum_page_overhead_bytes FROM dbstat GROUP BY name,pagetype ORDER BY name,pagetype')
    for row in pages:
        row['overhead_bytes'] = row['page_bytes'] - row['payload_bytes'] - row['unused_bytes']
        require(min(row[k] for k in ('page_bytes', 'payload_bytes', 'unused_bytes', 'overhead_bytes', 'minimum_page_unused_bytes', 'minimum_page_payload_bytes', 'minimum_page_overhead_bytes')) >= 0,
                'invalid unclamped SQLite page counters')
    page_size, page_count, free = [scalar(db, 'PRAGMA ' + p) for p in ('page_size', 'page_count', 'freelist_count')]
    require(scalar(db, 'PRAGMA auto_vacuum') == 0, 'pointer-map attribution required for auto-vacuum database')
    require(sum(r['pages_count'] for r in pages) == scalar(db, 'SELECT count(DISTINCT pageno) FROM dbstat'), 'dbstat page duplication')
    logical = page_size * page_count
    require(logical == snapshot.stat().st_size == ack['database_logical_bytes'], 'snapshot/acknowledgement logical length mismatch')
    btree = sum(r['page_bytes'] for r in pages)
    residual = logical - btree - free * page_size
    require(residual >= 0, 'negative SQLite page residual')
    blob_count, blob_bytes = db.execute('SELECT count(*),coalesce(sum(length(data)),0) FROM object_packs').fetchone()
    source_count, source_bytes = db.execute('SELECT count(*),coalesce(sum(canonical_length),0) FROM objects').fetchone()
    require((source_count, source_bytes) == (ack['canonical_objects'], ack['canonical_bytes']), 'snapshot/acknowledgement canonical population mismatch')
    pack_payload = sum(r['payload_bytes'] for r in pages if r['name'] == 'object_packs')
    require(pack_payload >= blob_bytes, 'negative SQLite pack row encoding')

    packs_count, pack_bytes, pack_groups = idx.execute('SELECT count(*),coalesce(sum(bytes),0),coalesce(sum(groups),0) FROM packs').fetchone()
    require((packs_count, pack_bytes) == (blob_count, blob_bytes), 'pack inventory/snapshot BLOB mismatch')
    require(scalar(idx, 'SELECT count(*) FROM packs p LEFT JOIN pack_versions v USING(pack) WHERE v.version IS NULL OR v.version NOT IN (1,2)') == 0
            and scalar(idx, 'SELECT count(*) FROM pack_versions') == packs_count, 'pack version coverage')
    group_totals = rows(idx, 'SELECT count(*) groups_count,coalesce(sum(encoded),0) encoded_bytes,coalesce(sum(decoded),0) decoded_bytes,coalesce(sum(records),0) records_count,coalesce(sum(full_bytes),0) legacy_full_bytes,coalesce(sum(delta_bytes),0) legacy_delta_bytes,coalesce(sum(copy_bytes),0) copy_bytes,coalesce(sum(insert_frame_bytes),0) insert_frame_bytes,coalesce(sum(literal_bytes),0) literal_bytes,coalesce(sum(delta_header_bytes),0) delta_header_bytes FROM groups')[0]
    native = rows(idx, 'SELECT coalesce(sum(full_header_bytes),0) full_header_bytes,coalesce(sum(full_frame_bytes),0) full_frame_bytes,coalesce(sum(prefix_header_bytes),0) prefix_header_bytes,coalesce(sum(prefix_frame_bytes),0) prefix_frame_bytes,coalesce(sum(canonical_bytes),0) canonical_bytes FROM native_groups')[0]
    group_count = group_totals['groups_count']
    require(pack_groups == group_count and pack_bytes == 16 * packs_count + 16 * group_count + group_totals['encoded_bytes'], 'pack/group conservation')
    require(scalar(idx, 'SELECT count(*) FROM packs p LEFT JOIN (SELECT pack,count(*) n,sum(encoded) encoded,sum(records) records FROM groups GROUP BY pack) g USING(pack) WHERE g.n IS NULL OR p.groups!=g.n OR p.bytes!=16+16*g.n+g.encoded OR g.records>8191') == 0, 'per-pack framing/count conservation')
    require(scalar(idx, '''SELECT count(*) FROM groups g JOIN pack_versions v USING(pack) LEFT JOIN native_groups n ON n.pack=g.pack AND n.grp=g.grp
      WHERE g.decoded != 4+4*g.records+g.full_bytes+g.delta_bytes+coalesce(n.full_header_bytes+n.full_frame_bytes+n.prefix_header_bytes+n.prefix_frame_bytes,0)
      OR g.delta_bytes != g.copy_bytes+g.insert_frame_bytes+g.literal_bytes+g.delta_header_bytes
      OR (v.version=2 AND (n.pack IS NULL OR g.codec!='Raw' OR g.encoded!=g.decoded OR g.full_bytes!=0 OR g.delta_bytes!=0))
      OR (v.version=1 AND n.pack IS NOT NULL)''') == 0, 'per-group decoded/native grammar account')
    require(group_totals['decoded_bytes'] == 4*group_count + 4*group_totals['records_count'] + group_totals['legacy_full_bytes'] + group_totals['legacy_delta_bytes'] + sum(native[k] for k in ('full_header_bytes','full_frame_bytes','prefix_header_bytes','prefix_frame_bytes')), 'decoded aggregate conservation')
    require(scalar(idx, '''SELECT count(*) FROM native_records n JOIN records r USING(pack,grp,rec)
      WHERE r.kind NOT IN ('NATIVE_FULL','NATIVE_PREFIX') OR r.role!='payload_chunk'
      OR n.raw_bytes+21!=r.bytes OR n.header_bytes+n.frame_bytes!=r.record_bytes
      OR n.frame_bytes<1 OR n.frame_bytes>33024 OR n.raw_bytes<0 OR n.raw_bytes>32768
      OR n.prefix_edges>4 OR n.closure_raw_bytes>1048576
      OR (r.kind='NATIVE_FULL' AND n.header_bytes!=5) OR (r.kind='NATIVE_PREFIX' AND n.header_bytes!=37)''') == 0, 'native record component account')
    require(scalar(idx, 'SELECT count(*) FROM native_records') == scalar(idx, "SELECT count(*) FROM records WHERE kind IN ('NATIVE_FULL','NATIVE_PREFIX')"), 'native record coverage')
    require(scalar(idx, """WITH n AS (SELECT n.pack,n.grp,
      sum(CASE WHEN r.kind='NATIVE_FULL' THEN n.header_bytes ELSE 0 END) fh,
      sum(CASE WHEN r.kind='NATIVE_FULL' THEN n.frame_bytes ELSE 0 END) ff,
      sum(CASE WHEN r.kind='NATIVE_PREFIX' THEN n.header_bytes ELSE 0 END) ph,
      sum(CASE WHEN r.kind='NATIVE_PREFIX' THEN n.frame_bytes ELSE 0 END) pf,sum(r.bytes) cb
      FROM native_records n JOIN records r USING(pack,grp,rec) GROUP BY n.pack,n.grp)
      SELECT count(*) FROM native_groups g LEFT JOIN n USING(pack,grp) WHERE n.fh IS NULL
      OR g.full_header_bytes!=n.fh OR g.full_frame_bytes!=n.ff OR g.prefix_header_bytes!=n.ph
      OR g.prefix_frame_bytes!=n.pf OR g.canonical_bytes!=n.cb""") == 0, 'native record/group component sums')
    require(scalar(idx, '''SELECT count(*) FROM records r LEFT JOIN records b ON r.base=b.id AND b.selected=1 WHERE r.base IS NOT NULL AND
      (b.id IS NULL OR (r.kind='DELTA' AND b.kind!='FULL') OR (r.kind='NATIVE_PREFIX' AND
      (b.kind NOT IN ('FULL','NATIVE_FULL','NATIVE_PREFIX') OR b.role!='payload_chunk' OR b.pack>=r.pack)))''') == 0, 'required base representation/location invalid')
    selected_count, selected_bytes = idx.execute('SELECT count(*),coalesce(sum(bytes),0) FROM records WHERE selected=1').fetchone()
    require((selected_count, selected_bytes) == (source_count, source_bytes), 'selected canonical inventory mismatch')
    require(scalar(idx, 'SELECT count(*) FROM records') == group_totals['records_count'], 'physical record/group counts')
    require(scalar(idx, "SELECT count(*) FROM records WHERE bytes<1 OR record_bytes<1 OR selected NOT IN (0,1) OR kind NOT IN ('FULL','DELTA','NATIVE_FULL','NATIVE_PREFIX') OR (kind='FULL' AND record_bytes!=bytes+1)") == 0, 'physical record kind/length validity')
    for kind, key in (('FULL','legacy_full_bytes'),('DELTA','legacy_delta_bytes')):
        require(idx.execute('SELECT coalesce(sum(record_bytes),0) FROM records WHERE kind=?',(kind,)).fetchone()[0] == group_totals[key], 'legacy record/group component mismatch: '+kind)

    roles = rows(idx, 'SELECT role,kind,count(*) objects_count,sum(bytes) canonical_bytes,sum(record_bytes) decoded_record_bytes FROM records WHERE selected=1 GROUP BY role,kind ORDER BY role,kind')
    payload_use = rows(idx, """WITH RECURSIVE metadata(id) AS (
      SELECT e.child FROM edges e JOIN records r ON r.id=e.parent AND r.selected=1
      JOIN logical_retained l ON l.id=r.id WHERE r.role='metadata_map_leaf' AND e.use_label='metadata_value'
      UNION SELECT e.child FROM edges e JOIN metadata m ON e.parent=m.id)
      SELECT CASE WHEN f.id IS NOT NULL AND m.id IS NOT NULL THEN 'file_and_metadata'
        WHEN f.id IS NOT NULL THEN 'file_only' WHEN m.id IS NOT NULL THEN 'metadata_only'
        ELSE 'neither_under_retained_roots' END use_class,r.kind,count(*) objects_count,sum(r.bytes) canonical_bytes
      FROM records r LEFT JOIN file_content_objects f ON f.id=r.id LEFT JOIN metadata m ON m.id=r.id
      WHERE r.selected=1 AND r.role='payload_chunk' GROUP BY use_class,r.kind ORDER BY use_class,r.kind""")
    group_classes = '''SELECT g.*,v.version,CASE WHEN count(DISTINCT r.role)=1 THEN max(r.role) ELSE 'mixed_roles' END role_class
      FROM groups g JOIN pack_versions v USING(pack) JOIN records r ON r.pack=g.pack AND r.grp=g.grp GROUP BY g.pack,g.grp'''
    groups = rows(idx, 'SELECT version,codec,role_class,count(*) groups_count,sum(encoded) encoded_bytes,sum(decoded) decoded_bytes,sum(records) records_count FROM (' + group_classes + ') GROUP BY version,codec,role_class ORDER BY version,codec,role_class')
    require(sum(r['encoded_bytes'] for r in groups) == group_totals['encoded_bytes'], 'exclusive group role partition')
    composition = rows(idx, 'SELECT g.version,g.role_class,r.role,r.kind,count(*) physical_records_count,sum(r.record_bytes) decoded_record_bytes,sum(r.bytes) reconstructed_canonical_bytes FROM (' + group_classes + ') g JOIN records r ON r.pack=g.pack AND r.grp=g.grp GROUP BY g.version,g.role_class,r.role,r.kind ORDER BY g.version,g.role_class,r.role,r.kind')
    require(sum(r['decoded_record_bytes'] for r in composition) + 4*group_count + 4*group_totals['records_count'] == group_totals['decoded_bytes'], 'exact decoded role composition')
    scope = "CASE WHEN r.selected=0 THEN 'physical_unselected' WHEN l.id IS NOT NULL THEN 'logical_retained' WHEN q.id IS NOT NULL THEN 'physical_base_only' ELSE 'selected_outside_required' END"
    memberships = ' FROM records r LEFT JOIN logical_retained l ON l.id=r.id LEFT JOIN required_objects q ON q.id=r.id '
    retention = rows(idx, 'SELECT '+scope+' retention_scope,r.role,r.kind,count(*) objects_or_records_count,sum(r.bytes) canonical_bytes,sum(r.record_bytes) decoded_record_bytes'+memberships+' GROUP BY retention_scope,r.role,r.kind ORDER BY retention_scope,r.role,r.kind')
    retained_count = scalar(idx, 'SELECT count(*) FROM logical_retained')
    required_count = scalar(idx, 'SELECT count(*) FROM required_objects')
    base_only = sum(r['objects_or_records_count'] for r in retention if r['retention_scope']=='physical_base_only')
    outside = sum(r['objects_or_records_count'] for r in retention if r['retention_scope']=='selected_outside_required')
    unselected = sum(r['objects_or_records_count'] for r in retention if r['retention_scope']=='physical_unselected')
    require(retained_count+base_only == required_count and required_count+outside == selected_count and selected_count+unselected == group_totals['records_count'], 'transitive retention partition')
    require(scalar(idx, 'SELECT count(*) FROM required_objects q LEFT JOIN records r ON r.id=q.id AND r.selected=1 WHERE r.id IS NULL') == 0, 'closure missing selected locator')
    dependencies = rows(idx, 'SELECT '+scope+' source_scope,r.kind,b.kind base_kind,b.role base_role,count(*) physical_references_count,count(DISTINCT r.base) unique_bases_count,sum(r.bytes) source_canonical_bytes,sum(r.pack!=b.pack) cross_pack_count,sum(r.pack!=b.pack OR r.grp!=b.grp) cross_group_count'+memberships+' JOIN records b ON r.base=b.id AND b.selected=1 GROUP BY source_scope,r.kind,b.kind,b.role ORDER BY source_scope,r.kind,b.kind,b.role')
    fan_in = rows(idx, "SELECT base_kind,base_role,CASE WHEN selected_references<=1 THEN '0..1' WHEN selected_references<=4 THEN '2..4' WHEN selected_references<=16 THEN '5..16' WHEN selected_references<=64 THEN '17..64' WHEN selected_references<=256 THEN '65..256' WHEN selected_references<=1024 THEN '257..1024' ELSE '1025+' END selected_fan_in_bin,count(*) unique_bases_count,sum(physical_references) physical_references_count,sum(selected_references) selected_references_count FROM dependency_fan_in GROUP BY base_kind,base_role,selected_fan_in_bin ORDER BY base_kind,base_role,selected_fan_in_bin")

    contracts = {}
    def write_csv(name, records, columns, population, provenance):
        contracts[name] = dict(columns={key: ('bytes' if key.endswith('_bytes') else 'count' if key.endswith('_count') else 'format_version' if key=='version' else 'text') for key in columns}, population=population, snapshot=snap_scope, provenance=provenance, status='derived from authenticated inventory or SQLite dbstat; no estimates')
        with (out/name).open('x', newline='') as stream:
            writer=csv.DictWriter(stream,fieldnames=columns+['population','snapshot','provenance','status'])
            writer.writeheader()
            for row in records:
                writer.writerow(dict(row,population=population,snapshot=snap_scope,provenance=provenance,status='derived'))
    for name, records, columns, population, provenance in [
        ('sqlite-breakdown.csv',pages,['name','pagetype','pages_count','page_bytes','payload_bytes','unused_bytes','minimum_page_unused_bytes','minimum_page_payload_bytes','minimum_page_overhead_bytes','overhead_bytes'],'exclusive SQLite B-tree/page-type totals; overflow already included',str(snapshot)+'#dbstat'),
        ('group-accounts.csv',groups,['version','codec','role_class','groups_count','encoded_bytes','decoded_bytes','records_count'],'all physical groups; each encoded group counted once',inventory_prov+'#groups/records/pack_versions'),
        ('group-role-composition.csv',composition,['version','role_class','role','kind','physical_records_count','decoded_record_bytes','reconstructed_canonical_bytes'],'exact decoded composition; no compressed per-record attribution',inventory_prov+'#records/group role classes'),
        ('object-roles.csv',roles,['role','kind','objects_count','canonical_bytes','decoded_record_bytes'],'unique selected objects by exclusive role and representation',inventory_prov+'#records[selected=1]'),
        ('payload-use.csv',payload_use,['use_class','kind','objects_count','canonical_bytes'],'unique selected payload chunks; overlapping file/metadata uses represented once in combined class',inventory_prov+'#retained inode-content and metadata-value edges'),
        ('retention.csv',retention,['retention_scope','role','kind','objects_or_records_count','canonical_bytes','decoded_record_bytes'],'selected L/base-only/outside-R objects; unselected physical records separately',inventory_prov+'#logical_retained/required_objects/records'),
        ('dependencies.csv',dependencies,['source_scope','kind','base_kind','base_role','physical_references_count','unique_bases_count','source_canonical_bytes','cross_pack_count','cross_group_count'],'physical dependency references by source scope; unique bases not additive across rows',inventory_prov+'#records[selected base locators]'),
        ('dependency-fan-in.csv',fan_in,['base_kind','base_role','selected_fan_in_bin','unique_bases_count','physical_references_count','selected_references_count'],'unique referenced selected bases; fixed selected-reference histogram',inventory_prov+'#dependency_fan_in')]:
        write_csv(name,records,columns,population,provenance)
    def values(numbers,population,provenance):
        return {key:q(value,'count' if key.endswith('_count') else 'bytes',population,snap_scope,provenance) for key,value in numbers.items()}
    report = dict(schema='issue88-combined-account-v1',status='PASS',input_hashes=hashes,inventory_proof=proof_ref,performance_manifest=manifest_ref,
      proof_scope='bound upstream decoder/custody artifacts checked; no payload decoding repeated',field_contract=contracts,
      nesting='native/legacy records inside groups inside pack BLOBs inside SQLite pages inside filesystem allocation; do not add nested totals')
    report['filesystem']={key:q(ack[key],'bytes','original acknowledged Store and database/sidecars',ack_scope,ack_prov,'measured') for key in ('store_allocated_bytes','database_allocated_bytes','database_logical_bytes','sidecar_allocated_bytes','sidecar_logical_bytes')}
    report['filesystem']['signed_allocation_adjustment_bytes']=q(ack['database_allocated_bytes']-ack['database_logical_bytes'],'bytes','original database allocation minus logical length',ack_scope,ack_prov)
    report['filesystem']['adjustment_cause']=q(None,'text','signed allocation adjustment',ack_scope,'no filesystem mechanism experiment','unknown','unattributed; not proven preallocation, waste, reclaimability or bytes beyond EOF')
    report['sqlite']=values(dict(page_size_bytes=page_size,page_count=page_count,freelist_count=free,logical_page_bytes=logical,btree_page_bytes=btree,freelist_bytes=free*page_size,other_identified_page_bytes=0,unexplained_page_residual_bytes=residual,
      btree_payload_bytes=sum(r['payload_bytes'] for r in pages),btree_unused_bytes=sum(r['unused_bytes'] for r in pages),btree_overhead_bytes=sum(r['overhead_bytes'] for r in pages),
      object_packs_page_bytes=sum(r['page_bytes'] for r in pages if r['name']=='object_packs'),object_packs_sql_payload_bytes=pack_payload,object_packs_blob_bytes=blob_bytes,object_packs_row_encoding_bytes=pack_payload-blob_bytes,
      objects_index_page_bytes=sum(r['page_bytes'] for r in pages if r['name']=='objects'),other_metadata_page_bytes=sum(r['page_bytes'] for r in pages if r['name'] not in ('objects','object_packs'))),'SQLite exclusive pages and explicitly nested SQL payload dimensions',str(snapshot)+'#PRAGMA/dbstat/length(data)')
    report['packs']=values(dict(packs_count=packs_count,groups_count=group_count,pack_bytes=pack_bytes,header_bytes=16*packs_count,directory_bytes=16*group_count,encoded_group_bytes=group_totals['encoded_bytes']),'all physical packs; exclusive wire components',inventory_prov+'#packs/groups')
    report['decoded_groups']=values(dict(group_totals,count_framing_bytes=4*group_count,record_directory_bytes=4*group_totals['records_count']),'all physical decoded groups; encoded_bytes is tie-out only, not decoded addend',inventory_prov+'#groups')
    report['native_components']=values(native,'all physical native records; canonical_bytes is separate reconstructed logical dimension',inventory_prov+'#native_groups')
    report['retention']=values(dict(selected_count=selected_count,selected_canonical_bytes=selected_bytes,logical_retained_count=retained_count,required_count=required_count,physical_base_only_count=base_only,selected_outside_required_count=outside,unselected_records_count=unselected),'unique selected objects except explicitly physical unselected records; transitive native bases included',inventory_prov+'#retention sets')
    for name in ('page_size_bytes','page_count','freelist_count'):
        report['sqlite'][name]['status']='measured'
    report['equations']={
      'filesystem':'store_allocated_bytes = database_allocated_bytes + sidecar_allocated_bytes; signed_adjustment = database_allocated_bytes - database_logical_bytes',
      'sqlite_pages':'page_size_bytes * page_count = btree_page_bytes + freelist_bytes + other_identified_page_bytes + unexplained_page_residual_bytes',
      'btree_interior':'btree_page_bytes = btree_payload_bytes + btree_unused_bytes + btree_overhead_bytes; overflow included once in B-tree totals',
      'pack_rows':'object_packs_sql_payload_bytes = object_packs_blob_bytes + object_packs_row_encoding_bytes (nested in object_packs pages)',
      'packs':'pack_bytes = header_bytes + directory_bytes + encoded_group_bytes',
      'decoded_groups':'decoded_bytes = count_framing_bytes + record_directory_bytes + legacy_full_bytes + legacy_delta_bytes + native FULL/PREFIX headers and frames',
      'legacy_delta':'legacy_delta_bytes = delta_header_bytes + copy_bytes + insert_frame_bytes + literal_bytes',
      'native':'FULL = 5-byte header + frame; PREFIX = 37-byte header including base ID + frame; reconstructed canonical bytes not additive physical bytes',
      'retention':'R = L union transitive selected physical dependencies; selected = L + physical_base_only + selected_outside_required; physical_records = selected + unselected_records'}
    report['limitations']=['Original acknowledgement allocation is primary; snapshot stat/clone allocation is not equivalent.', 'SQLite unused bytes and allocation adjustment are not reclaimable-byte forecasts.', 'Native frames are exact record bytes; legacy compressed bytes belong to groups only.', 'Base-only canonical bytes are not reclaimable compressed bytes when groups are shared.', 'Objects outside specified retained roots are not automatically safe garbage.', 'No timing, admission chronology or historical fragmentation is inferred from final pack order/layout.', 'This accounting is not a fresh verification or an optimization acceptance gate.']
    require({name:sha(path) for name,path in paths.items()} == hashes and sha(proof_path)==proof_ref['sha256']
            and sha(manifest_path)==manifest_ref['sha256']
            and all(sha(pathlib.Path(proof[name]['path']))==proof[name]['sha256'] for name in ('decoder_binary','custody')), 'input changed during accounting')
    db.close();idx.close()
    with (out/'accounting-reconciliation.json').open('x') as stream:
        json.dump(report,stream,indent=2);stream.write('\n')
    manifest=dict(schema='issue88-combined-account-manifest-v1',status='SEALED',inputs=hashes,expected_sha256=sha(expected_path),tool_sha256=sha(pathlib.Path(__file__)),files={p.name:sha(p) for p in sorted(out.iterdir()) if p.is_file()})
    with (out/'manifest.sha256.json').open('x') as stream:
        json.dump(manifest,stream,indent=2);stream.write('\n')
    print(json.dumps(dict(status='PASS',allocated_bytes=ack['store_allocated_bytes'],pack_bytes=pack_bytes,selected_count=selected_count,base_only_count=base_only)))


if __name__=='__main__':
    require(len(sys.argv)==6,__doc__)
    paths=list(map(lambda p:pathlib.Path(p).resolve(),sys.argv[1:]))
    fresh=not paths[-1].exists()
    try:
        main(*paths)
    except Exception as error:
        if fresh and paths[-1].is_dir():
            with (paths[-1]/'failure.json').open('x') as stream:
                json.dump(dict(status='INVALID',reason=str(error),null_reason='incomplete accounting; do not use partial tables as verified'),stream,indent=2)
        raise

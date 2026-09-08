#!/usr/bin/env python3
"""Read-only SQLite account; pack/group decoding belongs to the shared roles pass.

Usage: physical.py RUN OUTPUT; physical.py --self-test
After the shared roles pass: physical.py --finish-shared OUTPUT
Requires a quiescent original, no SQLite sidecars, and previously checked custody.
Outputs are exclusive-created. Never run against an active Store.
"""
import csv
import hashlib
import json
import pathlib
import sqlite3
import sys


def sha(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def quantity(value, unit, population, snapshot, provenance, status='measured', reason=None):
    return dict(value=value, unit=unit, population=population, snapshot=snapshot,
                provenance=provenance, status=status, null_reason=reason)


def partition(page_bytes, payload, unused):
    overhead = page_bytes - payload - unused
    return overhead, min(page_bytes, payload, unused, overhead) >= 0


def finish_shared(out):
    """Reconcile shared CSVs without reopening SQLite or decoding a group."""
    if (out/'manifest.sha256.json').exists():
        raise ValueError('Refuse to modify sealed analysis')
    report=json.loads((out/'accounting-reconciliation.json').read_text())
    summary=json.loads((out/'roles-summary.json').read_text())
    post='post-verification retained original; issue87 shared authenticated census'
    def aggregate(name, columns, equation):
        totals={}
        with (out/name).open() as f:
            for row in csv.DictReader(f):
                if 'COPY_bytes' in row:
                    row['copy_bytes']=row.pop('COPY_bytes')
                numbers={k:int(row[k]) for k in columns}
                assert equation(numbers), (name,row)
                for k,v in numbers.items(): totals[k]=totals.get(k,0)+v
        return totals
    packs=aggregate('packs.csv',['pack_length_bytes','header_bytes','directory_bytes','encoded_group_bytes','group_count'],lambda n:n['pack_length_bytes']==n['header_bytes']+n['directory_bytes']+n['encoded_group_bytes'])
    groups=aggregate('groups.csv',['encoded_bytes','decoded_bytes','count_bytes','record_count','record_directory_bytes','FULL_record_bytes','DELTA_record_bytes','delta_header_bytes','copy_bytes','insert_frame_bytes','literal_bytes'],lambda n:n['decoded_bytes']==n['count_bytes']+n['record_directory_bytes']+n['FULL_record_bytes']+n['DELTA_record_bytes'] and n['DELTA_record_bytes']==n['delta_header_bytes']+n['copy_bytes']+n['insert_frame_bytes']+n['literal_bytes'])
    assert packs['pack_length_bytes']==report['B_sqlite']['object_packs_blob_bytes']['value']
    assert packs['encoded_group_bytes']==groups['encoded_bytes']
    assert packs['group_count']*4==groups['count_bytes']
    assert packs['directory_bytes']==16*packs['group_count']
    assert groups['FULL_record_bytes']==summary['full_record_count']+summary['full_canonical_bytes']
    assert packs['pack_length_bytes']==summary['pack_blob_bytes']
    assert groups['decoded_bytes']==summary['decoded_group_bytes']
    for name,data,source,pop in [('C_packs',packs,'packs.csv','all physical pack BLOBs, exclusive wire components'),
                                 ('D_decoded_groups',groups,'groups.csv','all decoded physical groups, exclusive decoded components')]:
        report[name]={k:quantity(v,'count' if k.endswith('_count') else 'bytes',pop,post,source+' sum '+k,'derived') for k,v in data.items()}
        report[name]['equation_residual_bytes']=quantity(0,'bytes',pop,post,source+' per-row and aggregate conservation assertions','derived')
    report['D_decoded_groups']['interpretation']='Decoded bytes are a logical codec input dimension, not additional on-disk bytes. Reconstructed DELTA target canonical bytes are separately reported by roles.'
    report['D_decoded_groups']['encoded_bytes']['population']='all encoded group bodies; repeated C operand for tie-out, not an additive decoded component'
    report['C_packs']['equation']='pack_length_bytes = header_bytes + directory_bytes + encoded_group_bytes'
    report['D_decoded_groups']['equation']='decoded_bytes = count_bytes + record_directory_bytes + FULL_record_bytes + DELTA_record_bytes'
    report['D_decoded_groups']['DELTA_equation']='DELTA_record_bytes = delta_header_bytes + copy_bytes + insert_frame_bytes + literal_bytes'
    report['D_decoded_groups']['FULL_equation']='FULL_record_bytes = FULL_kind_bytes + FULL_canonical_bytes'
    for key,value in [('FULL_kind_bytes',summary['full_record_count']),('FULL_canonical_bytes',summary['full_canonical_bytes'])]:
        report['D_decoded_groups'][key]=quantity(value,'bytes','all physical FULL records; nested FULL record bytes',post,'roles-summary.json; FULL has one kind byte followed by canonical bytes','derived')
    report['shared_inventory_provenance']={name:sha(out/name) for name in ('packs.csv','groups.csv','roles-summary.json')}
    report['field_contract']['C_D']='Shared validated pack/group CSV sums; all physical records, including unselected records; no proportional compressed attribution; exact equations asserted per row.'
    with (out/'accounting-reconciliation.json').open('w') as f:
        json.dump(report,f,indent=2);f.write('\n')
    print('PASS: shared pack/group conservation, SQLite BLOB tie-out; no Store opened')


def main(run, out):
    db = run / 'deepseek-full/host-runtime/store.sqlite'
    sidecars = [p for p in db.parent.glob(db.name + '-*') if p.is_file()]
    if sidecars:
        raise ValueError('immutable read requires absent sidecars: ' + str(sidecars))
    before = db.stat()
    digest = sha(db)
    result = json.loads((run / 'deepseek-full/performance-result.json').read_text())
    ack = next(r for r in result['records'][-1]['receipts'] if r['kind'] == 'storage-smoke-allocation')
    post = 'post-verification retained original; issue87 read-only census'
    primary = 'checkpoint157 acknowledgement; pre-verification receipt'
    receipt = str(run / 'deepseek-full/performance-result.json') + '#records[-1].receipts/storage-smoke-allocation'
    report = dict(schema_version=1, source_store=str(db), source_sha256=digest,
                  sqlite_library_version=sqlite3.sqlite_version,
                  nesting='D decoded records -> C encoded packs -> B SQLite pages -> A filesystem; never add nested totals',
                  existing_census='No completed full157 census artifact existed; unexecuted builds/full157-report.py inspected and not run.',
                  limitations=['Current stat is not acknowledgement allocation.',
                               'No pre-verification logical snapshot survives; current pages cannot reconstruct historical placement.',
                               'Unused page bytes and signed allocation adjustment are not measured reclaimable bytes.'])
    def q(value, unit='bytes', population='whole Store', snapshot=post, provenance=str(db), status='measured', reason=None):
        return quantity(value, unit, population, snapshot, provenance, status, reason)
    report['A_filesystem'] = dict(
        acknowledgement={k:q(ack[k], snapshot=primary, provenance=receipt) for k in
                         ('store_allocated_bytes','database_allocated_bytes','database_logical_bytes','sidecar_allocated_bytes','sidecar_logical_bytes')},
        acknowledgement_allocation_adjustment=q(ack['database_allocated_bytes']-ack['database_logical_bytes'], snapshot=primary, provenance=receipt, status='derived'),
        acknowledgement_residual=q(ack['store_allocated_bytes']-ack['database_allocated_bytes']-ack['sidecar_allocated_bytes'], snapshot=primary, provenance=receipt, status='derived'),
        current_database_allocated=q(before.st_blocks * 512, provenance='stat.st_blocks * 512'),
        current_database_logical=q(before.st_size, provenance='stat.st_size'),
        current_sidecars=q(0, provenance='no store.sqlite-* files present'),
        current_allocation_adjustment=q(before.st_blocks*512-before.st_size, provenance='stat.st_blocks * 512 - stat.st_size', status='derived'),
        allocation_adjustment_explanation=q(None, unit='text', status='unknown', reason='Allocation mechanism was not traced; signed difference alone cannot identify preallocation, waste or bytes beyond EOF.'))
    with sqlite3.connect(db.as_uri() + '?mode=ro&immutable=1', uri=True) as c:
        page_size = c.execute('pragma page_size').fetchone()[0]
        page_count = c.execute('pragma page_count').fetchone()[0]
        free = c.execute('pragma freelist_count').fetchone()[0]
        if c.execute('pragma auto_vacuum').fetchone()[0] != 0:
            raise ValueError('Pointer-map account required for auto-vacuum Store')
        types = dict(c.execute('select name,type from sqlite_schema'))
        # dbstat streams once; aggregate rows only, never retain pages or payloads.
        rows = []
        for name, kind, count, size, payload, unused, low in c.execute(
                'select name,pagetype,count(*),sum(pgsize),sum(payload),sum(unused),min(unused) '
                'from dbstat group by name,pagetype order by name,pagetype'):
            overhead, valid = partition(size, payload, unused)
            rows.append(dict(snapshot=post, name=name, schema_type=types.get(name, 'schema'),
                             page_type=kind, pages_count=count, page_bytes=size, payload_bytes=payload,
                             unused_bytes=unused, overhead_bytes=overhead, minimum_page_unused_bytes=low,
                             counter_valid=valid and low >= 0, population='exclusive pages by B-tree and page type; overflow already included',
                             provenance='SQLite dbstat; overhead=page_bytes-payload_bytes-unused_bytes',
                             status='measured counters; derived overhead', null_reason=''))
        if sum(r['pages_count'] for r in rows) != c.execute('select count(distinct pageno) from dbstat').fetchone()[0]:
            raise ValueError('dbstat duplicated pages')
        blobs = c.execute('select count(*),sum(length(data)) from object_packs').fetchone()
    with (out/'sqlite-breakdown.csv').open('x', newline='') as f:
        w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
    btree = sum(r['page_bytes'] for r in rows)
    pack_rows = [r for r in rows if r['name']=='object_packs']
    pack_payload = sum(r['payload_bytes'] for r in pack_rows)
    report['B_sqlite'] = {k:q(v, unit=('count' if k.endswith('_count') else 'bytes'), population='exclusive SQLite pages', provenance='PRAGMA/dbstat/SQL length; corresponding sums and differences', status=('measured' if k in ('page_size_bytes','page_count','freelist_count','object_packs_blob_bytes') else 'derived')) for k,v in dict(
        page_size_bytes=page_size,page_count=page_count,logical_page_bytes=page_size*page_count,
        btree_page_bytes=btree,freelist_count=free,freelist_bytes=free*page_size,
        other_identified_page_bytes=0, unexplained_page_residual_bytes=page_size*page_count-btree-free*page_size,
        payload_bytes=sum(r['payload_bytes'] for r in rows),unused_bytes=sum(r['unused_bytes'] for r in rows),
        overhead_bytes=sum(r['overhead_bytes'] for r in rows),
        object_packs_page_bytes=sum(r['page_bytes'] for r in pack_rows),object_packs_sql_payload_bytes=pack_payload,
        object_packs_blob_bytes=blobs[1],object_packs_sql_row_encoding_bytes=pack_payload-blobs[1],
        objects_index_page_bytes=sum(r['page_bytes'] for r in rows if r['name']=='objects'),
        other_metadata_page_bytes=sum(r['page_bytes'] for r in rows if r['name'] not in ('objects','object_packs')),
        verifier_logical_growth_bytes=page_size*page_count-ack['database_logical_bytes']).items()}
    report['B_sqlite']['all_counter_partitions_valid']=q(all(r['counter_valid'] for r in rows),unit='boolean',provenance='unclamped payload/unused/overhead; per-page min unused')
    for k in ('object_packs_blob_bytes','object_packs_sql_payload_bytes','object_packs_sql_row_encoding_bytes'):
        report['B_sqlite'][k]['population']='all object_packs SQL rows; nested inside object_packs B-tree pages'
    report['B_sqlite']['object_packs_blob_bytes']['provenance']='SELECT sum(length(data)) FROM object_packs'
    report['B_sqlite']['object_packs_sql_row_encoding_bytes']['provenance']='dbstat object_packs payload minus sum(length(data)); INTEGER PRIMARY KEY payload is NULL'
    report['B_sqlite']['verifier_logical_growth_bytes']['population']='difference of post-verification logical bytes and final performance acknowledgement logical bytes'
    report['B_sqlite']['snapshot_delta_interpretation']='Net added pages are observable; final layout does not prove individual verifier page placement.'
    report['field_contract'] = {'csv_units':'*_bytes bytes; *_count counts; counter_valid boolean; all remaining fields text',
                              'csv_snapshot_population_provenance_status':'provided in every row; overhead is derived; no missing numerical observations in this table',
                              'other_identified_page_bytes':'auto_vacuum=0, zero freelist; sqlite_schema includes page1; all pages accounted by dbstat',
                              'C_D':'Shared validated pack/group pass in packs.csv and groups.csv; equations finalized separately without second decoding pass.'}
    after=db.stat()
    if (before.st_size,before.st_mtime_ns,before.st_ino)!=(after.st_size,after.st_mtime_ns,after.st_ino) or sha(db)!=digest:
        raise ValueError('Store changed during census')
    report['source_unchanged']=True
    with (out/'accounting-reconciliation.json').open('x') as f:
        json.dump(report,f,indent=2);f.write('\n')
    print(json.dumps({k:v['value'] for k,v in report['B_sqlite'].items() if isinstance(v,dict)},indent=2))


if __name__=='__main__':
    if sys.argv[1:]==['--self-test']:
        assert partition(4096,3000,1000)==(96,True)
        assert partition(4096,3000,1200)==(-104,False)
        assert partition(4096,4100,-4)==(0,False)
        assert quantity(None,'bytes','selected','post','missing','unknown','not sampled')['value'] is None
        print('PASS: conservation and invalid-counter preservation')
    elif len(sys.argv)==3 and sys.argv[1]=='--finish-shared':
        finish_shared(pathlib.Path(sys.argv[2]).resolve())
    else:
        main(pathlib.Path(sys.argv[1]).resolve(),pathlib.Path(sys.argv[2]).resolve())

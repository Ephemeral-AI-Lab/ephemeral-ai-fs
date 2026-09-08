#!/usr/bin/env python3
"""Existing-inventory metadata only: INVENTORY PROOF NEW_OUTPUT.

PROOF is the sealed upstream inventory-proof JSON (status PASS,
inventory_sha256, all_selected_ids_authenticated, all_references_valid).
No Store, codec, census or allocation query. Run only in the final analysis slot.
"""
import csv
import hashlib
import json
import pathlib
import sqlite3
import sys


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def require(ok, message):
    if not ok:
        raise ValueError(message)


SIZE_SQL = '''WITH bins(bin,lo,hi) AS (
 VALUES ('0..63',0,63),('64..255',64,255),('256..1023',256,1023),
 ('1024..4095',1024,4095),('4096..16383',4096,16383),
 ('16384..65535',16384,65535),('65536+',65536,NULL)
), combos AS (SELECT DISTINCT role,kind FROM records WHERE selected=1),
 observed AS (
 SELECT r.role,r.kind,b.bin,count(*) n,sum(r.bytes) canonical_bytes
 FROM records r JOIN bins b ON r.bytes>=b.lo AND (b.hi IS NULL OR r.bytes<=b.hi)
 WHERE r.selected=1 GROUP BY r.role,r.kind,b.bin
)
SELECT c.role,c.kind,b.bin,b.lo AS lower_bytes,b.hi AS upper_bytes,
 coalesce(o.n,0) AS objects_count,coalesce(o.canonical_bytes,0) AS canonical_bytes
FROM combos c CROSS JOIN bins b LEFT JOIN observed o
 ON o.role=c.role AND o.kind=c.kind AND o.bin=b.bin ORDER BY c.role,c.kind,b.lo'''

ROOTS_SQL = '''WITH types(source) AS (VALUES ('layers'),('commits'),('workspace_stages'))
SELECT t.source,count(r.id) AS root_rows_count,count(DISTINCT r.id) AS unique_roots_count
FROM types t LEFT JOIN roots r ON r.source=t.source GROUP BY t.source ORDER BY t.source'''

DEPTH_SQL = '''WITH depths(prefix_edges) AS (VALUES (0),(1),(2),(3),(4)),
 kinds(kind) AS (VALUES ('NATIVE_FULL'),('NATIVE_PREFIX')),
 selection(selected) AS (VALUES (0),(1)),
 observed AS (
 SELECT r.selected,r.kind,n.prefix_edges,count(*) AS physical_records_count,
 sum(r.bytes) AS canonical_bytes,sum(n.raw_bytes) AS target_raw_bytes,
 sum(n.header_bytes) AS header_bytes,sum(n.frame_bytes) AS frame_bytes,
 max(n.closure_raw_bytes) AS maximum_closure_raw_bytes
 FROM native_records n JOIN records r USING(pack,grp,rec)
 GROUP BY r.selected,r.kind,n.prefix_edges
)
SELECT s.selected,k.kind,d.prefix_edges,
 coalesce(o.physical_records_count,0) AS physical_records_count,
 coalesce(o.canonical_bytes,0) AS canonical_bytes,
 coalesce(o.target_raw_bytes,0) AS target_raw_bytes,
 coalesce(o.header_bytes,0) AS header_bytes,coalesce(o.frame_bytes,0) AS frame_bytes,
 o.maximum_closure_raw_bytes
FROM selection s CROSS JOIN kinds k CROSS JOIN depths d LEFT JOIN observed o
 ON o.selected=s.selected AND o.kind=k.kind AND o.prefix_edges=d.prefix_edges
ORDER BY s.selected,k.kind,d.prefix_edges'''


def main(inventory, proof_path, out):
    out.mkdir(parents=True, exist_ok=False)
    digests = {'inventory': sha(inventory), 'proof': sha(proof_path), 'tool': sha(pathlib.Path(__file__))}
    proof = json.loads(proof_path.read_text())
    require(proof['status'] == 'PASS' and proof['inventory_sha256'] == digests['inventory'],
            'proof does not bind this completed inventory')
    require(proof['all_selected_ids_authenticated'] is True and proof['all_references_valid'] is True,
            'upstream canonical/reference validation incomplete')
    require(proof['snapshot_phase'] == 'final-pre-verification'
            and proof['quiescence']['status'] == 'PASS'
            and proof['quiescence']['snapshot_mode'] == 'ro-immutable',
            'final pre-verification immutable custody required for supplement scope')
    require(not any(inventory.parent.glob(inventory.name + '-*')), 'immutable inventory has sidecars')
    db = sqlite3.connect(inventory.as_uri() + '?mode=ro&immutable=1', uri=True)
    db.execute('PRAGMA query_only=ON')
    db.execute('PRAGMA cache_size=-8192')
    db.execute('PRAGMA temp_store=FILE')
    def scalar(sql):
        return db.execute(sql).fetchone()[0]
    require(scalar("SELECT count(*) FROM roots WHERE source NOT IN ('layers','commits','workspace_stages') OR source IS NULL OR id IS NULL") == 0,
            'unrecognized/missing root source; fixed source table would omit roots')
    require(scalar('SELECT count(*) FROM records WHERE selected NOT IN (0,1) OR selected IS NULL OR bytes<0 OR bytes IS NULL') == 0,
            'invalid record selection/canonical size')
    require(scalar("SELECT count(*) FROM native_records n LEFT JOIN records r USING(pack,grp,rec) WHERE r.kind IS NULL OR r.kind NOT IN ('NATIVE_FULL','NATIVE_PREFIX') OR n.prefix_edges NOT BETWEEN 0 AND 4 OR n.prefix_edges IS NULL OR (r.kind='NATIVE_FULL' AND n.prefix_edges!=0) OR (r.kind='NATIVE_PREFIX' AND n.prefix_edges=0)") == 0,
            'invalid native depth/record population')
    native_count = scalar('SELECT count(*) FROM native_records')
    require(native_count == scalar("SELECT count(*) FROM records WHERE kind IN ('NATIVE_FULL','NATIVE_PREFIX')"), 'native record coverage')
    snapshot = 'final pre-verification logical snapshot represented by bound inventory; phase and immutable custody checked in upstream proof'
    specifications = [
        ('role-size-histogram.csv', SIZE_SQL, 'unique selected objects by exclusive role, representation and fixed canonical-byte bin'),
        ('roots.csv', ROOTS_SQL, 'collected retained-root rows by source; unique roots across sources are not additive'),
        ('native-depth.csv', DEPTH_SQL, 'physical native records by selected flag/representation/depth; not public read-call histogram'),
    ]
    contract = {}
    for name, sql, population in specifications:
        cursor = db.execute(sql)
        columns = [c[0] for c in cursor.description]
        values = cursor.fetchall()  # Compact fixed histograms, never physical records/payloads.
        if name == 'role-size-histogram.csv':
            require(sum(row[5] for row in values) == scalar('SELECT count(*) FROM records WHERE selected=1'), 'size histogram count conservation')
            require(sum(row[6] for row in values) == scalar('SELECT coalesce(sum(bytes),0) FROM records WHERE selected=1'), 'size histogram canonical-byte conservation')
        if name == 'native-depth.csv':
            require(sum(row[3] for row in values) == native_count, 'native depth histogram count conservation')
            require(sum(row[4] for row in values) == scalar('SELECT coalesce(sum(r.bytes),0) FROM native_records n JOIN records r USING(pack,grp,rec)'), 'native depth canonical-byte conservation')
        with (out/name).open('x', newline='') as stream:
            writer = csv.writer(stream)
            writer.writerow(columns + ['population','snapshot','provenance','status'])
            for row in values:
                writer.writerow(['null' if x is None else x for x in row] + [population,snapshot,str(inventory),'derived'])
        contract[name] = dict(population=population,snapshot=snapshot,provenance=str(inventory),status='derived from authenticated upstream inventory',
          columns={key: ('bytes' if key.endswith('_bytes') else 'count' if key.endswith('_count') or key=='prefix_edges' else 'boolean integer 0/1' if key=='selected' else 'text') for key in columns},
          null_reason={'upper_bytes':'unbounded upper endpoint for65536+ bin','maximum_closure_raw_bytes':'no native records in this exact bin'},
          empty_bins='zero counts and byte sums require present validated tables; an empty maximum remains null')
    shared_roots = dict(root_rows_count=scalar('SELECT count(*) FROM roots'),
                        unique_roots_count=scalar('SELECT count(DISTINCT id) FROM roots'),
                        units='integer counts',population='all collected root sources together; deduplicated root identity',snapshot=snapshot,status='derived',provenance=str(inventory))
    db.close()
    require(sha(inventory)==digests['inventory'] and sha(proof_path)==digests['proof'] and sha(pathlib.Path(__file__))==digests['tool'], 'input changed during supplements')
    with (out/'field-contract.json').open('x') as stream:
        json.dump(dict(schema='issue88-inventory-supplements-v1',status='PASS',hashes=digests,inventory_proof=str(proof_path),
          proof_scope='caller-supplied upstream proof hash preserved; inventory hash matched; no decoding or independent producer authentication repeated',
          tables=contract,all_root_sources=shared_roots),stream,indent=2);stream.write('\n')
    with (out/'manifest.sha256.json').open('x') as stream:
        json.dump(dict(schema='issue88-inventory-supplements-manifest-v1',status='SEALED',inputs=digests,
          files={p.name:sha(p) for p in sorted(out.iterdir()) if p.is_file() and p.name!='manifest.sha256.json'}),stream,indent=2);stream.write('\n')
    print(json.dumps(dict(status='PASS',native_records_count=native_count,unique_roots_count=shared_roots['unique_roots_count'])))


if __name__=='__main__':
    require(len(sys.argv)==4,__doc__)
    inventory,proof,out=map(lambda p:pathlib.Path(p).resolve(),sys.argv[1:])
    fresh=not out.exists()
    try:
        main(inventory,proof,out)
    except Exception as error:
        if fresh and out.is_dir():
            with (out/'failure.json').open('x') as stream:
                json.dump(dict(status='INVALID',reason=str(error),null_reason='incomplete supplement; no zero/empty interpretation'),stream,indent=2)
        raise

#!/usr/bin/env python3
"""Bind one completed frozen control/P census to the existing158-row validator.

No Store connection or canonical decoding. Only ordinary file hashes and read-only
analysis-inventory queries. Required census completion schema:
 issue88-combined-census-completion-v1; arm; status:PASS; exit_code:0;
 command:[absolute_decoder,absolute_snapshot_store,absolute_inventory_parent];
 decoder_binary/log/snapshot/inventory/snapshot_custody:{path,sha256};
 single_writer_append_only:true; single_writer_basis:nonempty actual-custody text.
 elapsed_ns is an optional nonnegative measured observer duration.
Outputs are exclusively created in a NEW directory; failed preparation writes no
PASS proof. Run only in the serialized census/analysis slot, after actual decoder
completion and before normal verification, per the frozen schedule.
"""
import argparse
import hashlib
import json
import pathlib
import re
import sqlite3
import sys


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    with pathlib.Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def load(path):
    return json.loads(pathlib.Path(path).read_text())


def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write('\n')


def sealed(reference, expected_path=None):
    require(isinstance(reference, dict) and set(('path', 'sha256')) <= reference.keys(), 'missing sealed file reference')
    path = pathlib.Path(reference['path'])
    require(path.is_absolute() and path.is_file() and not path.is_symlink(), 'sealed path must be an absolute regular file')
    if expected_path is not None:
        require(path.resolve() == pathlib.Path(expected_path).resolve(), 'sealed reference path mismatch')
    digest = sha(path)
    require(digest == reference['sha256'], 'sealed file hash mismatch: ' + str(path))
    return path.resolve(), digest


def flag(command, name):
    require(isinstance(command, list) and command.count(name) == 1, 'frozen command flag missing/ambiguous: ' + name)
    index = command.index(name)
    require(index + 1 < len(command), 'frozen command flag value missing: ' + name)
    return command[index + 1]


def check_completion(completion, arm, snapshot, inventory, custody_path, census):
    require(completion['schema'] == 'issue88-combined-census-completion-v1', 'completion schema')
    require(completion['arm'] == arm and completion['status'] == 'PASS', 'completion arm/status')
    require(type(completion['exit_code']) is int and completion['exit_code'] == 0, 'decoder did not exit successfully')
    require(completion['single_writer_append_only'] is True and isinstance(completion['single_writer_basis'], str) and completion['single_writer_basis'].strip(), 'actual single-writer execution custody missing')
    if 'elapsed_ns' in completion:
        require(type(completion['elapsed_ns']) is int and completion['elapsed_ns'] >= 0, 'invalid observer elapsed ns')
    refs = {}
    for name, target in (('snapshot', snapshot), ('inventory', inventory), ('snapshot_custody', custody_path), ('decoder_binary', census['binary']), ('log', None)):
        path, digest = sealed(completion[name], target)
        refs[name] = {'path': str(path), 'sha256': digest}
    require(refs['decoder_binary']['sha256'] == census['sha256'], 'decoder differs from prospectively frozen binary')
    expected_command = [refs['decoder_binary']['path'], str(snapshot), str(inventory.parent)]
    require(completion['command'] == expected_command, 'actual decoder invocation does not bind these inputs/output')
    return refs


def inventory_checks(path, native):
    db = sqlite3.connect(path.as_uri() + '?mode=ro&immutable=1', uri=True)
    try:
        db.execute('PRAGMA query_only=ON')
        db.execute('PRAGMA cache_size=-8192')
        queries = {
            'missing_reference_count': 'SELECT count(*) FROM edges e WHERE NOT EXISTS(SELECT 1 FROM records r WHERE r.id=e.child AND r.selected=1)',
            'missing_required_locator_count': 'SELECT count(*) FROM required_objects q WHERE NOT EXISTS(SELECT 1 FROM records r WHERE r.id=q.id AND r.selected=1)',
            'duplicate_selected_id_count': 'SELECT count(*) FROM (SELECT id FROM records WHERE selected=1 GROUP BY id HAVING count(*)!=1)',
            'duplicate_physical_locator_count': 'SELECT count(*) FROM (SELECT pack,grp,rec FROM records GROUP BY pack,grp,rec HAVING count(*)!=1)',
            'bad_record_fields_count': 'SELECT count(*) FROM records WHERE id IS NULL OR bytes IS NULL OR bytes<1 OR role IS NULL OR pack IS NULL OR grp IS NULL OR rec IS NULL OR kind IS NULL',
            'orphan_native_detail_count': 'SELECT count(*) FROM native_records n LEFT JOIN records r USING(pack,grp,rec) WHERE r.pack IS NULL',
            'bad_selected_flag_count': 'SELECT count(*) FROM records WHERE selected IS NULL OR selected NOT IN (0,1)',
            'bad_kind_count': "SELECT count(*) FROM records WHERE kind IS NULL OR kind NOT IN ('FULL','DELTA','NATIVE_FULL','NATIVE_PREFIX')",
            'bad_legacy_delta_base_count': "SELECT count(*) FROM records d LEFT JOIN records b ON d.base=b.id AND b.selected=1 WHERE d.kind='DELTA' AND (b.id IS NULL OR b.kind!='FULL' OR d.base_pack IS NULL OR d.base_group IS NULL OR d.base_pack!=b.pack OR d.base_group!=b.grp)",
            'bad_native_base_count': "SELECT count(*) FROM records d LEFT JOIN records b ON d.base=b.id AND b.selected=1 WHERE d.kind='NATIVE_PREFIX' AND (b.id IS NULL OR d.base_pack IS NULL OR d.base_group IS NULL OR b.kind NOT IN ('FULL','NATIVE_FULL','NATIVE_PREFIX') OR b.role!='payload_chunk' OR b.pack>=d.pack OR d.base_pack!=b.pack OR d.base_group!=b.grp)",
            'bad_native_record_count': "SELECT count(*) FROM native_records n JOIN records r USING(pack,grp,rec) WHERE n.prefix_edges IS NULL OR n.closure_raw_bytes IS NULL OR n.frame_bytes IS NULL OR n.header_bytes IS NULL OR r.role!='payload_chunk' OR r.kind NOT IN ('NATIVE_FULL','NATIVE_PREFIX') OR n.prefix_edges NOT BETWEEN 0 AND 4 OR n.closure_raw_bytes NOT BETWEEN 0 AND 1048576 OR n.raw_bytes IS NULL OR n.raw_bytes NOT BETWEEN 0 AND 32768 OR n.frame_bytes NOT BETWEEN 1 AND 33024 OR r.bytes!=n.raw_bytes+21 OR (r.kind='NATIVE_FULL' AND (n.header_bytes!=5 OR n.prefix_edges!=0)) OR (r.kind='NATIVE_PREFIX' AND (n.header_bytes!=37 OR n.prefix_edges<1))",
            'duplicate_native_detail_count': 'SELECT count(*) FROM (SELECT pack,grp,rec FROM native_records GROUP BY pack,grp,rec HAVING count(*)!=1)',
            'missing_native_detail_count': "SELECT count(*) FROM records r LEFT JOIN native_records n USING(pack,grp,rec) WHERE r.kind IN ('NATIVE_FULL','NATIVE_PREFIX') AND n.pack IS NULL",
            'bad_version_count': 'SELECT count(*) FROM pack_versions WHERE version IS NULL OR version NOT IN (1,2)',
            'version_record_mismatch_count': "SELECT count(*) FROM records r LEFT JOIN pack_versions v USING(pack) WHERE v.pack IS NULL OR (v.version=1 AND r.kind NOT IN ('FULL','DELTA')) OR (v.version=2 AND r.kind NOT IN ('NATIVE_FULL','NATIVE_PREFIX'))",
        }
        checks = {}
        for name, query in queries.items():
            checks[name] = db.execute(query).fetchone()[0]
            require(checks[name] == 0, name + ': ' + str(checks[name]))
        selected, canonical = db.execute('SELECT count(*),coalesce(sum(bytes),0) FROM records WHERE selected=1').fetchone()
        packs = db.execute('SELECT count(*) FROM packs').fetchone()[0]
        representations = dict(db.execute('SELECT kind,count(*) FROM records GROUP BY kind'))
        require(native or not any(representations.get(k, 0) for k in ('NATIVE_FULL', 'NATIVE_PREFIX')), 'control contains native representations')
        require(db.execute('SELECT count(*) FROM pack_versions').fetchone()[0] == packs, 'pack/version cardinality')
        return {'selected_objects_count': selected, 'selected_canonical_bytes': canonical, 'packs_count': packs, 'physical_representation_counts': representations, 'zero_violation_counts': checks}
    finally:
        db.close()


def prepare(args):
    run, snapshot_dir, inventory = args.run.resolve(), args.snapshot.resolve(), args.inventory.resolve()
    snapshot = snapshot_dir / 'store.sqlite'
    custody_path = snapshot_dir / 'custody.json'
    out = args.output.resolve()
    require(not out.exists(), 'output must be new')
    for source in (run, snapshot_dir, inventory.parent):
        require(out != source and not out.is_relative_to(source), 'output must be separate from sealed inputs')
    schedule = load(args.frozen_schedule)
    require(schedule['schema'] == 'issue88-SP-full157-frozen-v1', 'frozen schedule schema')
    require([row['arm'] for row in schedule['order']] == ['control', 'candidate'], 'frozen arm order/cardinality')
    frozen = next(row for row in schedule['order'] if row['arm'] == args.arm)
    require(pathlib.Path(frozen['output']).resolve() == run, 'run differs from frozen arm output')
    require(pathlib.Path(flag(frozen['performance'], '--output')).resolve() == run, 'frozen performance output')
    require(flag(frozen['performance'], '--image') == frozen['image_id'], 'frozen performance image')
    require(flag(frozen['performance'], '--source-arm') == ('baseline' if args.arm == 'control' else 'candidate'), 'frozen source-arm setting')
    host_binary = pathlib.Path(flag(frozen['performance'], '--host-binary'))
    require(sha(host_binary) == frozen['host']['binary_sha256'], 'producer host binary seal mismatch')
    contract, contract_sha = sealed(schedule['contract'])
    # Build qualification is additive custody, not a substitute for source seals.
    sealed(schedule['candidate_build_qualification'])
    custody = load(custody_path)
    require(custody['status'] == 'PASS' and custody['snapshot_phase'] == 'final-pre-verification' and custody['snapshot_mode'] == 'ro-immutable', 'snapshot phase/status/mode')
    require(custody['quiescence'] == {'status': 'PASS', 'lsof_no_open_handles': True, 'performance_cleanup': 'PASS'}, 'snapshot quiescence custody')
    require(custody['frozen_schedule_sha256'] == sha(args.frozen_schedule), 'snapshot differs from frozen schedule')
    expected_source = run / 'deepseek-full/host-runtime/store.sqlite'
    require(pathlib.Path(custody['source']).resolve() == expected_source, 'snapshot source path mismatch')
    manifest_path = run / 'performance-manifest.json'
    require(sha(manifest_path) == custody['performance_manifest_sha256'], 'performance manifest seal mismatch')
    manifest = load(manifest_path)
    require(manifest[str(expected_source.relative_to(run))] == custody['source_sha256'] == custody['copy_sha256'], 'original/snapshot content identity mismatch')
    paths = {'performance': run/'deepseek-full/performance-result.json', 'identity': run/'identity.json', 'snapshot': snapshot, 'inventory': inventory, 'contract': contract}
    hashes = {name: sha(path) for name, path in paths.items()}
    require(hashes['snapshot'] == custody['copy_sha256'], 'snapshot logical content changed')
    for name in ('performance', 'identity'):
        require(hashes[name] == manifest[str(paths[name].relative_to(run))], 'sealed run artifact changed: ' + name)
    data, identity = load(paths['performance']), load(paths['identity'])
    require(data['status'] == data['cleanup_status'] == 'PASS' and data['container_removed'] is True, 'performance cleanup/status')
    require([row['index'] for row in data['records']] == list(range(1, 158)), 'full157 checkpoint order/cardinality')
    require(identity['host_identity'] == frozen['host'] and identity['image_id'] == frozen['image_id'], 'measured host/image identity mismatch')
    require(identity['smoke'] == 'deepseek-full' and identity['schema'] == 'deepseek-full-m45-v1', 'run identity population')
    for key in ('LAYERFS_SOURCE_SEAL', 'LAYERFS_PRODUCT_SEAL', 'WORKLOAD_SOURCE_SHA256'):
        require(identity['source'][key] == frozen['host'][key], 'source/product/workload seal mismatch')
    ack = [row for row in data['records'][-1]['receipts'] if row.get('kind') == 'storage-smoke-allocation']
    require(len(ack) == 1 and ack[0]['label'] == 'step-157' and ack[0] == custody['primary_final_ack'], 'primary acknowledgement mismatch')
    completion = load(args.census_custody)
    refs = check_completion(completion, args.arm, snapshot, inventory, custody_path, schedule['census'])
    require(refs['snapshot']['sha256'] == hashes['snapshot'] and refs['inventory']['sha256'] == hashes['inventory'], 'completion input hashes')
    stats = inventory_checks(inventory, args.arm == 'candidate')
    require((stats['selected_objects_count'], stats['selected_canonical_bytes']) == (ack[0]['canonical_objects'], ack[0]['canonical_bytes']), 'acknowledgement/inventory selected population mismatch')
    footer = re.findall(r'^authenticated (\d+) selected objects across (\d+) packs$', pathlib.Path(refs['log']['path']).read_text(), re.MULTILINE)
    require(footer == [(str(stats['selected_objects_count']), str(stats['packs_count']))], 'actual decoder completion footer mismatch/missing')
    require(sha(inventory) == hashes['inventory'], 'analysis inventory changed during preparation')
    proof = {'schema':'issue88-D-inventory-proof-v1','status':'PASS','arm':args.arm,'native_representations':args.arm=='candidate','snapshot_sha256':hashes['snapshot'],'inventory_sha256':hashes['inventory'],'decoder_command':completion['command'],'decoder_binary':refs['decoder_binary'],'custody':refs['snapshot_custody'],'quiescence':{'status':'PASS','snapshot_mode':'ro-immutable'},'all_selected_ids_authenticated':True,'all_references_valid':True,'selected_delta_bases_selected_FULL':True,'selected_delta_bases_scope':'legacy DELTA only; native PREFIX supports authenticated FULL/NATIVE_FULL/NATIVE_PREFIX','native_dependencies_authenticated_by_completed_decoder':True,'decoder_exit_status':0,'decoder_observer_elapsed_ns':completion.get('elapsed_ns'),'decoder_observer_elapsed_status':'measured' if 'elapsed_ns' in completion else 'unknown: completion receipt omitted elapsed_ns','decoder_log_sha256':refs['log']['sha256'],'decoder_log':refs['log'],'census_completion':{'path':str(args.census_custody.resolve()),'sha256':sha(args.census_custody)},'frozen_schedule':{'path':str(args.frozen_schedule.resolve()),'sha256':sha(args.frozen_schedule)},'single_writer_append_only':True,'single_writer_basis':completion['single_writer_basis'],'inventory_checks':stats,'snapshot_phase':'final-pre-verification','scope':'Completed frozen exact decoder authenticates canonical/physical records; helper hashes custody and checks existing metadata only. No second decoding or Store connection.'}
    expected = {'schema':'issue88-D-expected-v1','hashes':hashes,'contract_path':str(contract),'identity':{'host_identity':frozen['host'],'image_id':frozen['image_id'],'smoke':'deepseek-full','schema':'deepseek-full-m45-v1'},'inventory_authenticated':True,'single_writer_append_only':True,'display_encoding':'tagged33','id_prefixes':{'commit':18,'layer':50},'native_representations':args.arm=='candidate','provenance':{'path':str(out/'inventory-proof.json')}}
    out.mkdir(parents=True)
    write(out/'inventory-proof.json', proof)
    expected['provenance']['sha256'] = sha(out/'inventory-proof.json')
    write(out/'validation-expected.json', expected)
    write(out/'manifest.sha256.json', {p.name:sha(p) for p in sorted(out.iterdir()) if p.is_file()})
    print(json.dumps({'status':'PASS','arm':args.arm,'native_representations':expected['native_representations'],'authenticated_selected_count':stats['selected_objects_count'],'output':str(out)}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--arm', choices=('control', 'candidate'), required=True)
    for name in ('run', 'snapshot', 'inventory', 'census-custody', 'frozen-schedule', 'output'):
        parser.add_argument('--'+name, type=pathlib.Path, required=True)
    prepare(parser.parse_args())


if __name__ == '__main__':
    main()

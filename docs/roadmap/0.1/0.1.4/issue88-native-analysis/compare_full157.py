#!/usr/bin/env python3
"""Reconcile the frozen pair's receipts and manifests; no Store decoding."""
import argparse, hashlib, json, pathlib, runpy

HERE = pathlib.Path(__file__).resolve().parent

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('schedule', type=pathlib.Path)
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args()
    frozen = json.loads(args.schedule.read_text())
    assert frozen['schema'] == 'issue88-SP-full157-frozen-v1'
    assert [a['arm'] for a in frozen['order']] == ['control','candidate']
    root = args.schedule.parent.parent
    workload_path=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data/checkpoint-manifest.json')
    assert sha(workload_path)==frozen['workload_manifest_sha256']
    workload=json.loads(workload_path.read_text())['checkpoints']
    assert len(workload)==157
    validator = runpy.run_path(str(HERE.parent / 'issue88-delivery/validate_run.py'))
    result = dict(schema='issue88-SP-comparison-v1', status='PASS', units='integer bytes/ns/counts',
                  scope='one frozen fresh full157 pair; not release statistics', arms={},
                  provenance={'schedule': {'path': str(args.schedule), 'sha256': sha(args.schedule)}},
                  unknowns={'allocation_adjustment_cause': None, 'repeatable_timing_effect': None,
                            'compressed_savings_from_legacy_AB_vs_native_selected': None})
    reference = None
    for arm in frozen['order']:
        name = arm['arm']; run = pathlib.Path(arm['output'])
        perf = json.loads((run / 'deepseek-full/performance-result.json').read_text())
        ver = json.loads((run / 'deepseek-full/verification-result.json').read_text())
        identity = json.loads((run / 'identity.json').read_text())
        assert identity['host_identity'] == arm['host'] and identity['image_id'] == arm['image_id']
        for phase in [perf, ver]:
            assert phase['status'] == phase['cleanup_status'] == 'PASS' and phase['container_removed']
        rows = perf['records']
        assert [r['index'] for r in rows] == [r['index'] for r in ver['records']] == list(range(1,158))
        verified = {row['index']: row for row in ver['records']}
        assert len(rows) == len(verified) == 157 and set(verified) == set(range(1, 158))
        mapping = []
        for row in rows:
            entry=workload[row['index']-1]
            for key in ['sha','tree','manifest_sha256']:
                assert row[key]==entry[key]
            assert verified[row['index']]['status'] == 'PASS'
            assert verified[row['index']]['identity'] == row['identity']
            mapping.append({key: row[key] for key in ['index', 'sha', 'tree', 'oracle_sha256', 'manifest_sha256', 'files', 'logical_bytes', 'created']})
        if reference is None:
            reference = mapping; fixtures = identity['fixtures']
        else:
            assert mapping == reference and identity['fixtures'] == fixtures
        snapshot = root / f'issue88-SP-{name}-snapshot-1/store.sqlite'
        manifests = []
        for phase in ['performance', 'verification']:
            manifest = run / f'{phase}-manifest.json'
            entries = json.loads(manifest.read_text())
            for relative, digest in entries.items():
                path = snapshot if phase == 'performance' and relative == 'deepseek-full/host-runtime/store.sqlite' else run / relative
                assert sha(path) == digest, (name, phase, relative)
            manifests.append(dict(phase=phase, entries=len(entries), sha256=sha(manifest),
                                  store_scope='explicit preverification logical snapshot' if phase == 'performance' else 'postverification original'))
        native = name == 'candidate'
        counters = [validator['aggregate'](receipts, native_representations=native) for receipts in [perf['ready'], *[row['receipts'] for row in rows]]]
        assert all(not validator['validate_counters'](row, native_representations=native) for row in counters)
        fields = validator['FIELDS'] + validator['NATIVE_FIELDS']
        totals = {key: (max(row[key] for row in counters) if key in validator['GAUGES'] else sum(row[key] for row in counters)) for key in fields}
        phases = {}
        for row in rows:
            for receipt in row['receipts']:
                if receipt['kind'] == 'storage-smoke-phase':
                    label = receipt['phase']; phases[label] = phases.get(label, 0) + receipt['elapsed_ns']
        validation_path = root / f'issue88-SP-{name}-validation-1/validation.json'
        validation = json.loads(validation_path.read_text()); assert validation['status'] == 'PASS'
        assert validation['checkpoint_count'] == 158 and not validation['errors'] and validation['unique_cohort']
        assert validation['native_representations'] == native
        for key,path in {'performance':run/'deepseek-full/performance-result.json','identity':run/'identity.json','snapshot':snapshot}.items():
            assert validation['input_hashes'][key] == sha(path)
        result['arms'][name] = dict(final_ack=next(v for v in rows[-1]['receipts'] if v['kind'] == 'storage-smoke-allocation'),
            public_checkpoint_phase_ns=phases, init_receipts=perf['ready'], end_receipts=perf['closed'],
            transfer_ns=sum(row['transfer_ns'] for row in rows), step_wall_ns=sum(row['step_wall_ns'] for row in rows),
            preparation_ns={phase:json.loads((run/f'{phase}-summary.json').read_text())['preparation_ns'] for phase in ['performance','verification']},
            resources_scope='Raw host/container CPU/RSS/I/O, spool/runtime/free-disk observations remain in sealed performance/verification receipts; detailed scoped resource report required before final disposition',
            performance_enclosing={k: perf[k] for k in ['setup_ns','work_wall_ns','wall_ns','cleanup_ns']},
            verification_enclosing={k: ver[k] for k in ['setup_ns','work_wall_ns','wall_ns','cleanup_ns']},
            verified_bytes=sum(row['verified_bytes'] for row in verified.values()),
            verified_entries=sum(row['verified_entries'] for row in verified.values()),
            counters=totals, counter_scope='Init plus157checkpoint public phases; gauges max, other fields summed; nested work is not elapsed',
            manifests=manifests, validation={'path':str(validation_path),'sha256':sha(validation_path)},
            performance_summary={'path':str(run/'performance-summary.json'),'sha256':sha(run/'performance-summary.json')},
            identity={'path':str(run/'identity.json'),'sha256':sha(run/'identity.json')})
    a,b = (result['arms'][name] for name in ['control','candidate'])
    result['candidate_minus_control'] = {key:b['final_ack'][key]-a['final_ack'][key] for key in ['store_allocated_bytes','database_logical_bytes','canonical_bytes','canonical_objects']}
    result['candidate_minus_control']['checkpoint_phase_ns'] = {key:b['public_checkpoint_phase_ns'][key]-a['public_checkpoint_phase_ns'][key] for key in a['public_checkpoint_phase_ns']}
    with args.output.open('x') as stream:
        json.dump(result, stream, indent=2); stream.write('\n')
    print(json.dumps(result['candidate_minus_control']))

if __name__ == '__main__':
    main()

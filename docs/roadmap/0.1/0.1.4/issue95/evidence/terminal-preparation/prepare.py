#!/usr/bin/env python3
"""Seal prospective issue95 full157 inputs; no workload, build, or Store access."""
import argparse
import copy
import datetime
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
OLD = Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue91-runs/r26-preparation')

def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def ref(path):
    path = Path(path).resolve()
    return {'path': str(path), 'sha256': sha(path)}

def save(name, value):
    path = HERE / name
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')
    return ref(path)

def check_ref(value):
    assert sha(value['path']) == value['sha256'], value['path']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue95'))
    parser.add_argument('--host-binary', type=Path, required=True)
    parser.add_argument('--image', required=True)
    parser.add_argument('--build-qualification', type=Path, required=True)
    parser.add_argument('--check-qualification', type=Path, required=True)
    parser.add_argument('--terminal-declaration', type=Path, required=True)
    parser.add_argument('--census-binary', type=Path, required=True)
    parser.add_argument('--census-applicability', type=Path, required=True,
                        help='Root-authored receipt binding decoder source/build or unchanged decoder applicability.')
    parser.add_argument('--output-root', type=Path, default=HERE.parent/'terminal-full157')
    args = parser.parse_args()
    source, binary, output = args.source.resolve(), args.host_binary.resolve(), args.output_root.resolve()
    assert not (HERE/'schedule.frozen.json').exists() and not output.exists(), 'fresh evidence required'
    lock_path = Path(os.environ.get('TMPDIR', '/tmp'))/'layerfs-infra-measurement.lock'
    with lock_path.open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        old = json.loads((OLD/'schedule.frozen.json').read_text())
        check_ref(old['contract'])
        check_ref(old['control_reuse_applicability'])
        reuse = json.loads(Path(old['control_reuse_applicability']['path']).read_text())
        assert reuse['status'] == 'PASS_REUSE_ELIGIBLE' and reuse['original_control_arm'] == old['order'][0]
        for item in reuse['references'].values():
            check_ref(item)
        sys.path.insert(0, str(source/'benchmark/fs-bench-pro/shared'))
        spec = importlib.util.spec_from_file_location('issue95_runner', source/'benchmark/fs-bench-pro/shared/runner.py')
        runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runner)
        observed = runner.source_build_args()
        assert observed['LAYERFS_SOURCE_DIRTY'] == 'false', 'commit final source first'
        host = json.loads(Path(str(binary)+'.identity.json').read_text())
        assert host['binary_sha256'] == sha(binary)
        assert all(host[key] == value for key, value in observed.items()), 'host/source identity mismatch'
        info = json.loads(subprocess.check_output(['docker', 'image', 'inspect', args.image], text=True))[0]
        labels = info['Config']['Labels']
        assert not info['Config'].get('Volumes')
        for key, label in [('LAYERFS_SOURCE_SEAL', 'dev.layerfs.source-seal'),
                           ('LAYERFS_PRODUCT_SEAL', 'dev.layerfs.product-seal'),
                           ('WORKLOAD_SOURCE_SHA256', 'dev.layerfs.workload-source-sha256')]:
            assert labels.get(label) == host[key], (key, labels)
        assert host['WORKLOAD_SOURCE_SHA256'] == old['order'][0]['host']['WORKLOAD_SOURCE_SHA256']
        helpers = source/'docs/roadmap/0.1/0.1.4/issue88-native-analysis'
        contract = source/'docs/roadmap/0.1/0.1.4/issue88-native-design/full157-contract-v1.md'
        assert sha(contract) == old['contract']['sha256'], 'full157 contract changed'
        for path in (args.build_qualification, args.check_qualification, args.census_applicability):
            receipt = json.loads(path.read_text())
            assert str(receipt.get('status', '')).startswith('PASS'), ('qualification not PASS', path)
        free = shutil.disk_usage(output.parent).free
        assert free >= old['limits']['minimum_free_host_bytes']
        s = copy.deepcopy(old)
        for key in ('reuse_guard_preflight', 'candidate_check_applicability'):
            s.pop(key, None)
        candidate = s['order'][1]
        candidate.update(cwd=str(source), host=host, image_id=info['Id'], reporter_head=observed['LAYERFS_SOURCE_COMMIT'], checkout_status='')
        for key, suffix in [('output','run'), ('snapshot','snapshot'), ('inventory_output','census'),
                            ('proof_output','proof'), ('validation_output','validation'), ('account_output','account')]:
            candidate[key] = str(output/('candidate-'+suffix))
        for key, flag in [('performance','--output'), ('verification','--storage-verify-run')]:
            candidate[key] = ['python3', 'benchmark/fs-bench-pro/shared/runner.py', '--deepseek-full', '--source-arm',
                              'candidate', '--repetition', '1', '--image', info['Id'], '--host-binary', str(binary), flag, candidate['output']]
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        candidate['source_revalidation'] = save('candidate-final-preflight.json', {
            'status':'PASS', 'arm':'candidate', 'observed_at_utc':now, 'source':observed,
            'host_binary':ref(binary), 'host_identity':host, 'image_id':info['Id'], 'image_labels':labels,
            'scope':'Read-only actual source/binary/image identity checks; no Store/workload access.'})
        reuse.update(schema='issue95-control-reuse-applicability-v1', scope='Unchanged R25 supplemental storage control reused with original source, contract, workload and completed custody. Not published v0.1.3 performance or a new paired observation.')
        reuse['references']['prior_reuse_applicability'] = old['control_reuse_applicability']
        s['control_reuse_applicability'] = save('control-reuse-applicability.json', reuse)
        s.update(frozen_at_utc=now, analysis_source=str(source), output_root=str(output), free_bytes_before=free,
                 prospective_terminal_commit=observed['LAYERFS_SOURCE_COMMIT'], ready_to_execute=True,
                 scope='Issue95 final candidate fresh full157; authenticated unchanged R25 control reuse; preserve all historical attempts.')
        s['contract'] = ref(contract)
        s['candidate_build_qualification'] = ref(args.build_qualification)
        s['candidate_check_qualification'] = ref(args.check_qualification)
        s['terminal_declaration'] = ref(args.terminal_declaration)
        s['census'] = {'binary':str(args.census_binary.resolve()), 'sha256':sha(args.census_binary),
                       'applicability':ref(args.census_applicability)}
        s['sealed_helpers'] = {str(source/Path(path).relative_to(Path(old['analysis_source']))):sha(source/Path(path).relative_to(Path(old['analysis_source'])))
                               for path in old['sealed_helpers'] if Path(path).is_relative_to(Path(old['analysis_source']))}
        s['sealed_helpers'].update({str(HERE/'run_frozen.py'):sha(HERE/'run_frozen.py'), str(HERE/'prepare.py'):sha(HERE/'prepare.py')})
        s['identity_preflight'] = save('identity-preflight.json', {'status':'PASS', 'arms':s['order'],
            'prospective_terminal_commit':s['prospective_terminal_commit'], 'candidate':candidate['source_revalidation'],
            'control_reuse_applicability':s['control_reuse_applicability']})
        seal = save('schedule.frozen.json', s)
        save('schedule-seal.json', seal)
        print(json.dumps({'schedule':seal, 'execute':['python3',str(HERE/'run_frozen.py'),seal['path'],'--execute']}))

if __name__ == '__main__':
    main()

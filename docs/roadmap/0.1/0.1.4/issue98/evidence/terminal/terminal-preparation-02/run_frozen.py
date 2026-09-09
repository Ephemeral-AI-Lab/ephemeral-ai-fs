#!/usr/bin/env python3
"""Issue98 full157 sequence with authenticated unchanged R25 control reuse."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, value):
    with Path(path).open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def reference(path):
    return {'path': str(path), 'sha256': sha(path)}


def account_expected(arm):
    run = Path(arm['output'])
    return {
        'schema': 'issue88-combined-account-expected-v1',
        'hashes': {
            'snapshot': sha(Path(arm['snapshot'])/'store.sqlite'),
            'inventory': sha(Path(arm['inventory_output'])/'roles-inventory.sqlite'),
            'performance': sha(run/'deepseek-full/performance-result.json'),
        },
        'snapshot_phase': 'final-pre-verification',
        'provenance': reference(Path(arm['proof_output'])/'inventory-proof.json'),
        'performance_manifest': reference(run/'performance-manifest.json'),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('schedule', type=Path)
    parser.add_argument('--execute', action='store_true', required=True)
    args = parser.parse_args()
    schedule_path = args.schedule.resolve()
    raw = schedule_path.read_text()
    schedule = json.loads(raw)
    assert schedule.get('ready_to_execute') is True and 'UNRESOLVED' not in raw, 'unsealed template forbidden'
    lock_path = Path(os.environ.get('TMPDIR', '/tmp'))/'layerfs-infra-measurement.lock'
    preflight_lock = lock_path.open('a')
    fcntl.flock(preflight_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    assert schedule['schema'] == 'issue88-SP-full157-frozen-v1'
    assert [arm['arm'] for arm in schedule['order']] == ['control', 'candidate']
    assert schedule['workload_manifest_sha256'] == '03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271'
    source = Path(schedule['analysis_source'])
    helpers = source/'docs/roadmap/0.1/0.1.4/issue88-native-analysis'
    validator = source/'docs/roadmap/0.1/0.1.4/issue88-delivery/validate_run.py'
    required = {str(helpers/name) for name in ('snapshot_combined.py', 'census_combined.py',
        'prepare_combined_validation.py', 'account_combined.py', 'resource_report.py')}
    required |= {str(validator), str(Path(__file__).resolve()),
                 str(source/'docs/roadmap/0.1/0.1.4/issue87-diagnostic-design/validate_diagnostic.py')}
    assert required <= schedule['sealed_helpers'].keys(), 'missing helper/source seals'
    for path, digest in schedule['sealed_helpers'].items():
        assert sha(path) == digest, ('helper changed', path)
    for key in ('contract', 'candidate_build_qualification', 'candidate_check_qualification', 'terminal_declaration'):
        item = schedule[key]
        assert sha(item['path']) == item['sha256'], key
    assert sha(schedule['census']['binary']) == schedule['census']['sha256'], 'census seal'
    item = schedule['census']['applicability']
    assert sha(item['path']) == item['sha256'], 'census applicability seal'
    # Both arms must have a coordinator-sealed identity preflight BEFORE arm 1.
    # Receipt contains actual source/host/image validation, not a user-filled label.
    preflight = schedule['identity_preflight']
    assert sha(preflight['path']) == preflight['sha256'], 'identity preflight seal'
    checked = json.loads(Path(preflight['path']).read_text())
    assert checked['status'] == 'PASS' and checked['arms'] == schedule['order'], 'identity preflight binding'
    assert checked['prospective_terminal_commit'] == schedule['prospective_terminal_commit'], 'terminal declaration binding'
    reused = schedule.get('reused_arm_schedules', {})
    assert set(reused) == {'control'}, 'Issue98 must reuse exactly the original control'
    original_ref = reused['control']
    assert Path(original_ref['path']).is_absolute() and sha(original_ref['path']) == original_ref['sha256']
    original = json.loads(Path(original_ref['path']).read_text())
    assert original['schema'] == schedule['schema']
    assert original['order'][0] == schedule['order'][0], 'control arm changed'
    assert original['workload_manifest_sha256'] == schedule['workload_manifest_sha256']
    assert original['contract']['sha256'] == schedule['contract']['sha256']
    applicability = schedule['control_reuse_applicability']
    assert sha(applicability['path']) == applicability['sha256'], 'control reuse applicability seal'
    reuse_proof = json.loads(Path(applicability['path']).read_text())
    assert reuse_proof['status'] == 'PASS_REUSE_ELIGIBLE' and reuse_proof['original_control_arm'] == schedule['order'][0]
    assert reuse_proof['references']['schedule'] == original_ref, 'control proof/schedule mismatch'
    for reference in reuse_proof['references'].values():
        assert sha(reference['path']) == reference['sha256'], ('reused metadata changed', reference['path'])
    for arm in schedule['order']:
        host = Path(arm['performance'][arm['performance'].index('--host-binary') + 1])
        assert sha(host) == arm['host']['binary_sha256'], 'producer binary changed'
        assert json.loads(Path(str(host)+'.identity.json').read_text()) == arm['host'], 'producer identity changed'
        if arm['arm'] in reused:
            continue  # Original control evidence was authenticated above; never recollect it.
        for key in ('output', 'snapshot', 'inventory_output', 'proof_output', 'validation_output', 'account_output'):
            assert not Path(arm[key]).exists(), ('fresh output required', arm[key])
    root = Path(schedule['output_root'])
    log_root = root/'execution'
    log_root.mkdir(parents=True, exist_ok=False)
    schedule_sha = sha(schedule_path)
    fcntl.flock(preflight_lock, fcntl.LOCK_UN)
    preflight_lock.close()
    sequence = 0

    def command(argv, cwd=source, own_lock=False):
        nonlocal sequence
        assert sha(schedule_path) == schedule_sha, 'schedule changed'
        sequence += 1
        save(log_root/f'{sequence:02}-command.json', {'argv': list(map(str, argv)), 'cwd': str(cwd)})
        with (log_root/f'{sequence:02}.log').open('xb') as log, lock_path.open('a') as lock:
            if not own_lock:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            result = subprocess.run(list(map(str, argv)), cwd=cwd, stdout=log, stderr=subprocess.STDOUT,
                                    timeout=None if own_lock else 14400)
        save(log_root/f'{sequence:02}-exit.json', {'exit_code': result.returncode})
        assert result.returncode == 0, ('step failed; preserve artifacts; do not rerun passing steps', sequence)

    for arm in schedule['order']:
        if arm['arm'] in reused:
            continue
        name = arm['arm']
        snapshot = Path(arm['snapshot'])/'store.sqlite'
        inventory = Path(arm['inventory_output'])/'roles-inventory.sqlite'
        performance = Path(arm['output'])/'deepseek-full/performance-result.json'
        proof = Path(arm['proof_output'])
        command(arm['performance'], Path(arm['cwd']), own_lock=True)
        command(['python3', helpers/'snapshot_combined.py', schedule_path, name, arm['snapshot']], own_lock=True)
        command(['python3', helpers/'census_combined.py', schedule_path, name, arm['snapshot'], arm['inventory_output']], own_lock=True)
        command(['python3', helpers/'prepare_combined_validation.py', '--arm', name, '--run', arm['output'],
            '--snapshot', arm['snapshot'], '--inventory', inventory, '--census-custody', Path(arm['inventory_output'])/'completion.json',
            '--frozen-schedule', schedule_path, '--output', proof])
        command(['python3', validator, arm['output'], snapshot, inventory, proof/'validation-expected.json', arm['validation_output']])
        expected = log_root/(name+'-account-expected.json')
        with lock_path.open('a') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            save(expected, account_expected(arm))
        command(['python3', helpers/'account_combined.py', snapshot, inventory, performance, expected, arm['account_output']])
        command(arm['verification'], Path(arm['cwd']), own_lock=True)
    command(['python3', helpers/'resource_report.py', '--control', schedule['order'][0]['output'],
        '--candidate', schedule['order'][1]['output'], '--schedule', schedule_path, '--output', root/'resources',
        '--control-snapshot-custody', Path(schedule['order'][0]['snapshot'])/'custody.json',
        '--control-census-custody', Path(schedule['order'][0]['inventory_output'])/'completion.json',
        '--candidate-snapshot-custody', Path(schedule['order'][1]['snapshot'])/'custody.json',
        '--candidate-census-custody', Path(schedule['order'][1]['inventory_output'])/'completion.json'])
    assert sha(schedule_path) == schedule_sha
    save(log_root/'completion.json', {'status': 'PASS', 'schedule_sha256': schedule_sha, 'steps': sequence, 'reused_arm_schedules': reused,
        'scope': 'Collected proof inputs only; qualification requires review of storage threshold, timing, resources and all campaign obligations. No merge/release.'})


if __name__ == '__main__':
    main()

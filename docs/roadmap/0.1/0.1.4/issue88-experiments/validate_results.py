#!/usr/bin/env python3
"""Validate a frozen screen envelope and its sealed artifacts, never run product."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import tempfile


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def integer(value, name):
    require(type(value) is int and value >= 0, name + ': integer >= 0 required')
    return value


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def validate(result, frozen, directory):
    require(result['schema'] == 'issue88-screen-envelope-v1', 'schema')
    require(result['status'] == 'PASS', 'screen not complete PASS; retain failure evidence')
    for key in ['stage', 'contract_sha256', 'identity', 'settings', 'population']:
        require(result[key] == frozen[key], key + ' differs from frozen expected value')
    require(result['claim_scope'] == 'offline_encoded_potential', 'offline screen cannot claim allocated product savings')
    require(result['allocated_store_bytes'] is None and bool(result['allocation_null_reason']),
            'offline envelope must not manufacture complete Store allocation')
    directory = directory.resolve()
    require(bool(result['artifacts']), 'no sealed artifacts')
    for name, expected in result['artifacts'].items():
        path = (directory / name).resolve()
        require(path.is_relative_to(directory) and path.is_file(), 'artifact path escapes output or missing')
        require(sha(path) == expected['sha256'], 'artifact digest mismatch: ' + name)
        require(path.stat().st_size == integer(expected['length_bytes'], name), 'artifact length mismatch')
    require(set(result['arms']) == {'control', 'candidate'}, 'paired control/candidate required')
    for name, arm in result['arms'].items():
        parts = arm['exclusive_physical_components_bytes']
        require(set(parts) == {'selected_record_bodies', 'physical_base_only_records',
                              'record_and_pack_framing', 'dictionaries', 'index_and_metadata'},
                'exclusive component partition incomplete')
        require(sum(integer(v, name + '.' + k) for k, v in parts.items()) ==
                integer(arm['total_representation_bytes'], name + '.total'), 'representation conservation')
        require(arm['retained_states_count'] == frozen['retained_states_count'], 'retained state scope')
        require(arm['authenticated_targets_count'] == arm['targets_count'], 'incomplete target authentication')
        require(arm['authenticated_canonical_bytes'] == arm['canonical_bytes'], 'incomplete byte authentication')
        for field in ['targets_count', 'canonical_bytes', 'authenticated_targets_count', 'authenticated_canonical_bytes']:
            integer(arm[field], name + '.' + field)
        require(arm['future_base_count'] == arm['missing_base_count'] == arm['cycle_count'] == 0,
                'invalid chronological dependency closure')
        for field in ['future_base_count', 'missing_base_count', 'cycle_count', 'max_delta_edges', 'max_closure_decoded_bytes']:
            integer(arm[field], name + '.' + field)
        require(arm['max_delta_edges'] <= frozen['max_delta_edges'] and
                arm['max_closure_decoded_bytes'] <= frozen['max_closure_decoded_bytes'], 'dependency limit')
        require(arm['scope'] == frozen['arm_scope'], 'partial representation scope mismatch')
    if frozen['same_canonical_population']:
        for key in ['targets_count', 'canonical_bytes', 'canonical_identity_set_sha256']:
            require(result['arms']['control'][key] == result['arms']['candidate'][key],
                    'unmatched canonical control: ' + key)
    require(result['lineage_provenance'] == frozen['lineage_provenance'], 'actual origin/inferred lineage conflated')
    require(result['cache_condition'] in ('fresh_application_decode_cache_os_cache_uncontrolled',
                                         'warm_application_decode_cache_os_cache_uncontrolled', None), 'unsupported cold-cache claim')
    if result['cache_condition'] is None:
        require(bool(result['cache_null_reason']), 'missing cache/read measurement reason')
    for key, limit in frozen['resource_limits'].items():
        require(integer(result['resources'][key], key) <= limit, 'resource limit: ' + key)
    require(result['cleanup_status'] == 'PASS' and result['original_evidence_unchanged'] is True,
            'cleanup or original custody failure')
    return {'status': 'PASS', 'stage': result['stage'], 'claim_scope': result['claim_scope'],
            'candidate_representation_bytes': result['arms']['candidate']['total_representation_bytes'],
            'limitation': 'Checks sealed evidence/envelope consistency; source review and real oracle/provenance execution remain mandatory.'}


def self_test():
    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        (d / 'raw.json').write_text('{}\n')
        frozen = dict(stage='S1', contract_sha256='c' * 64, identity={'binary_sha256': 'b' * 64},
                      settings={'codec': 'fixed'}, population='synthetic', retained_states_count=157,
                      max_delta_edges=1, max_closure_decoded_bytes=65536,
                      same_canonical_population=True, arm_scope='structural_subset',
                      lineage_provenance='actual_editor_origin', resource_limits={'wall_ns': 1000})
        arm = dict(exclusive_physical_components_bytes=dict(selected_record_bodies=70,
                   physical_base_only_records=5, record_and_pack_framing=10, dictionaries=0, index_and_metadata=15),
                   total_representation_bytes=100, retained_states_count=157, targets_count=3,
                   authenticated_targets_count=3, canonical_bytes=1000, authenticated_canonical_bytes=1000,
                   canonical_identity_set_sha256='a' * 64, future_base_count=0, missing_base_count=0,
                   cycle_count=0, max_delta_edges=1, max_closure_decoded_bytes=4096, scope='structural_subset')
        result = {k: copy.deepcopy(frozen[k]) for k in ['stage', 'contract_sha256', 'identity', 'settings', 'population']}
        result.update(schema='issue88-screen-envelope-v1', status='PASS', claim_scope='offline_encoded_potential',
                      allocated_store_bytes=None, allocation_null_reason='offline structural subset only',
                      artifacts={'raw.json': {'sha256': sha(d / 'raw.json'), 'length_bytes': 3}},
                      arms={'control': copy.deepcopy(arm), 'candidate': copy.deepcopy(arm)},
                      lineage_provenance='actual_editor_origin', cache_condition='fresh_application_decode_cache_os_cache_uncontrolled',
                      resources={'wall_ns': 500}, cleanup_status='PASS', original_evidence_unchanged=True)
        validate(result, frozen, d)
        changes = [lambda r: r.update(allocated_store_bytes=100),
                   lambda r: r['settings'].update(codec='changed'),
                   lambda r: r['arms']['candidate'].update(total_representation_bytes=99),
                   lambda r: r['arms']['candidate'].update(future_base_count=1),
                   lambda r: r['arms']['candidate'].update(authenticated_targets_count=2),
                   lambda r: r['arms']['candidate'].update(max_delta_edges=2),
                   lambda r: r.update(lineage_provenance='inferred_final_similarity'),
                   lambda r: r.update(cache_condition='cold_disk'),
                   lambda r: r['resources'].update(wall_ns=1001),
                   lambda r: r['artifacts']['raw.json'].update(sha256='0' * 64)]
        for change in changes:
            bad = copy.deepcopy(result)
            change(bad)
            try:
                validate(bad, frozen, d)
            except ValueError:
                pass
            else:
                raise AssertionError('invalid result accepted')
    print('PASS: one valid envelope; ten identity/accounting/provenance/resource failures rejected')


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--self-test', action='store_true')
    p.add_argument('result', nargs='?', type=Path)
    p.add_argument('expected', nargs='?', type=Path)
    args = p.parse_args()
    if args.self_test:
        self_test()
    else:
        require(args.result is not None and args.expected is not None, 'result and frozen expected JSON required')
        try:
            print(json.dumps(validate(json.loads(args.result.read_text()), json.loads(args.expected.read_text()),
                                      args.result.parent), indent=2))
        except (ValueError, KeyError, TypeError, OSError) as error:
            print(json.dumps({'status': 'FAIL', 'reason': str(error)}))
            raise SystemExit(1)

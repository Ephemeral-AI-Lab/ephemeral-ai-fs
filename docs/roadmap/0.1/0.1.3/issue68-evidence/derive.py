#!/usr/bin/env python3
"""Derive the fixed issue-68 cohort from raw receipts using only the standard library.

Usage: python3 derive.py EVIDENCE_ROOT --output OUTPUT_DIR --declaration cohort.json
Optional --proof paths are relative to EVIDENCE_ROOT and remain separate from timings.
The declaration's `runs` must enumerate the twelve final directory names exactly.
Missing repetitions, parsing errors and identity mismatches never become passing gates.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import statistics

ROOT = None
RAW = {}


def track(path):
    relative = str(path.relative_to(ROOT))
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    RAW[relative] = {'path': relative, 'bytes': path.stat().st_size, 'sha256': digest.hexdigest()}
    return RAW[relative]


GIT = ['git_first_status_ns', 'git_diff_ns', 'git_add_ns', 'git_cached_check_ns', 'git_commit_ns', 'git_final_status_ns']
TARGET = {'git-tool-100-mixed-v4': 500_000_000, 'git-tool-500-mixed-v4': 1_000_000_000}
STRETCH = {'git-tool-100-mixed-v4': 350_000_000, 'git-tool-500-mixed-v4': 750_000_000}

def numeric(value):
    return int(value) if isinstance(value, str) and re.fullmatch(r'-?\d+', value) else value

def kv(text, separator='='):
    return {k: numeric(v) for line in text.splitlines() if separator in line for k, v in [line.split(separator, 1)]}

def read(path):
    track(path)
    return json.loads(path.read_text())

def metric_groups(records, baseline=False):
    result = []
    for record in records:
        for name, contents in re.findall(r'(\w+(?:Receipt|Diagnostics)) \{ ([^{}]*) \}', record.get('details', '')):
            values = {key: int(value) for key, value in re.findall(r'(\w+): (?:Some\()?(-?\d+)(?:\))?(?=,|$)', contents)}
            if name == 'WorkspaceReadReceipt':
                unavailable = [k for k, v in values.items() if v == 0 and
                    (k.startswith(('read_ahead_', 'host_response_', 'client_response_', 'client_decode_', 'client_socket_', 'host_encode_', 'host_socket_')) or
                     k == 'host_dispatch_ns' or (baseline and k in ('read_plan_builds', 'rope_nodes_read', 'payload_ids', 'payload_batches', 'max_payload_batch', 'payload_bytes_read')))]
                for key in unavailable:
                    values[key] = None
            else:
                unavailable = []
            result.append({'type': name, 'operation': record.get('receipt', {}).get('family'),
                           'metrics': values, 'unavailable_or_unwired_zero_fields': unavailable})
    return result

def group(row, name):
    return next((x['metrics'] for x in row.get('metric_groups', []) if x['type'] == name), {})

def summarize_sample(path, sample, index):
    records = sample.get('records', [])
    identity = sample.get('identities', {})
    complete = next((r for r in records if r.get('kind') == 'sample-complete'), {})
    phases = [r for r in records if r.get('kind') == 'phase']
    workload = [kv(r['workload_receipt']) for r in phases if 'workload_receipt' in r]
    resources = {r['phase']: r for r in records if r.get('kind') == 'host-resources'}
    before, after = resources.get('before', {}), resources.get('after-product', {})
    deltas = {k: after[k] - before[k] for k in ('user_cpu_ns', 'system_cpu_ns', 'disk_read_bytes', 'disk_write_bytes') if k in after and k in before}
    elapsed = complete.get('pure_call_sum_ns')
    case = identity.get('case')
    if elapsed is not None:
        assert sum(p['elapsed_ns'] for p in phases) == elapsed, f'phase balance: {path}'
    return {'run': str(path.parent.relative_to(ROOT)), 'sample_index': index, 'arm': 'LayerFS',
            'case': case, 'identities': identity, 'raw': str(path.relative_to(ROOT)),
            'raw_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'collection_status': sample.get('status'), 'completion_status': sample.get('completion_status'),
            'complete_lifecycle_ns': elapsed, 'main_target_ns': TARGET.get(case),
            'main_target_status': ('PASS' if elapsed <= TARGET[case] else 'TARGET_MISS') if elapsed is not None and case in TARGET else 'INCOMPLETE',
            'stretch_status': ('PASS' if elapsed <= STRETCH[case] else 'TARGET_MISS') if elapsed is not None and case in STRETCH else 'INCOMPLETE',
            'phases': phases, 'workload_receipts': workload, 'metric_groups': metric_groups(records, path.parent.name.startswith('baseline-')),
            'host_resources': resources, 'host_resource_deltas': deltas, 'container_resources': sample.get('resources'),
            'preparation': sample.get('preparation'), 'setup': sample.get('setup'),
            'preparation_wall_ns': sample.get('preparation_wall_ns'), 'cleanup': sample.get('cleanup'),
            'prepared_master_unchanged': sample.get('prepared_master_unchanged'),
            'other_observations': [r for r in records if r.get('kind') in ('published-root', 'store-observation', 'workspace-physical-spool', 'workspace-spool-observation', 'host-rss-samples', 'runtime-observation-window')],
            'cache_state': 'Independent mutable workspace; runner prepared-fixture identity is recorded. Cold execution-cache policy must be corroborated by the cohort declaration and runner, not inferred from timing.',
            'proof_status': 'NOT_ESTABLISHED_BY_PERFORMANCE_COLLECTION'}



def summarize_native(path):
    raw = read(path)
    identities = read(path.with_name('matched-identities.json'))
    before = kv(raw.get('cpu_before_workload', ''), ' ')
    after = kv(raw.get('cpu_after_workload', ''), ' ')
    resources = raw.get('resources', '').splitlines()
    workload = {k: numeric(v) for k, v in raw['workload_receipt'].items()}
    container = read(path.with_name('container.json')) if path.with_name('container.json').exists() else {}
    return {
        'run': str(path.parent.relative_to(ROOT)), 'arm': 'native', 'case': raw['case'],
        'identities': identities, 'raw': str(path.relative_to(ROOT)),
        'raw_sha256': RAW[str(path.relative_to(ROOT))]['sha256'],
        'native_workload_ns': workload['workload_ns'], 'workload_receipts': [workload],
        'native_resources': {
            'workload_cpu_ns': (after['usage_usec'] - before['usage_usec']) * 1000
                if 'usage_usec' in before and 'usage_usec' in after else None,
            'container_lifetime_peak_bytes': int(resources[0])
                if resources and resources[0].isdigit() else None,
            'raw': raw.get('resources'), 'cpu_before': before, 'cpu_after': after,
        },
        'cache_state': raw.get('cache_policy'),
        'native_scope': 'Apply + six Git commands; no LayerFS Create, Commit, visibility or End.',
        'cleanup': read(path.with_name('cleanup.json')) if path.with_name('cleanup.json').exists() else None,
        'verification_status': raw.get('verification_status'), 'verification_ns': raw.get('verification_ns'),
        'setup_ns': raw.get('setup_ns'), 'input_manifest_sha256': raw.get('input_manifest_sha256'),
        'filesystem_environment': raw.get('filesystem_environment'),
        'container_observation': {'image': container.get('Image'), 'mounts': container.get('Mounts'),
            'host_config': {key: container.get('HostConfig', {}).get(key) for key in
                ('NanoCpus', 'Memory', 'MemorySwap', 'PidsLimit', 'Binds')}},
    }


IDENTITY_FIELDS = ('source_identity', 'product_identity', 'image', 'harness_identity',
                   'input_identity', 'fixture_profile', 'seed')
FINAL_RUNS = [f'final-{tier}-{arm}-r{rep}' for tier in (100, 500)
              for arm in ('layerfs', 'native') for rep in (1, 2, 3)]
BASELINE_RUNS = ['baseline-100', 'baseline-500', 'baseline-native-100-ownership-fixed',
                 'baseline-native-500']


def load_run(run):
    directory = ROOT / run
    native = '-native-' in run or run.startswith('baseline-native-')
    path = directory / ('native-result.json' if native else 'perf.jsonl')
    if not path.is_file():
        return None
    track(path)
    if native:
        row = summarize_native(path)
    else:
        lines = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
        samples = [(index, sample) for index, sample in enumerate(lines, 1) if sample.get('kind') == 'sample']
        if len(samples) != 1:
            raise ValueError(f'{run}: expected exactly one sample, found {len(samples)}')
        index, sample = samples[0]
        if sample.get('identities', {}).get('proof_only'):
            raise ValueError(f'{run}: proof-only sample is not a performance observation')
        row = summarize_sample(path, sample, index)
        row['environment_observation'] = sample.get('environment_observation')
        headers = [line for line in lines if line.get('kind') == 'header']
        if headers and headers[0].get('identities') != row['identities']:
            raise ValueError(f'{run}: header/sample identities differ')
    row['cohort'] = 'final-predeclared-slots' if run in FINAL_RUNS else 'fresh-before-single-observation'
    row['repetition'] = int(run[-1]) if run in FINAL_RUNS else None
    for relative in ('manifest.json', 'matched-identities.json', 'cleanup.json'):
        candidate = directory / relative
        if candidate.is_file():
            track(candidate)
    return row


def duration(row):
    return row.get('complete_lifecycle_ns', row.get('native_workload_ns'))


def cohort_statistics(rows, declaration_valid):
    result = []
    final = [row for row in rows if row['cohort'] == 'final-predeclared-slots']
    # Case-specific input identities may differ; final source/product/image may not.
    source_fields = IDENTITY_FIELDS[:4]
    global_identity_match = all(len({row['identities'].get(field) for row in final}) == 1
                                and all(row['identities'].get(field) for row in final)
                                for field in source_fields)
    for tier in (100, 500):
        case = f'git-tool-{tier}-mixed-v4'
        peers = [row for row in final if row['case'] == case]
        matching = len(peers) == 6 and global_identity_match and all(
            len({row['identities'].get(field) for row in peers}) == 1
            and all(row['identities'].get(field) not in (None, '') for row in peers)
            for field in IDENTITY_FIELDS)
        for arm in ('LayerFS', 'native'):
            selected = sorted([row for row in peers if row['arm'] == arm], key=lambda row: row['repetition'])
            values = [duration(row) for row in selected]
            complete = (len(selected) == 3 and [row['repetition'] for row in selected] == [1, 2, 3]
                        and all(isinstance(value, int) for value in values))
            collected = complete and all(
                (row.get('completion_status') == 'COMPLETE' if arm == 'LayerFS'
                 else row.get('verification_status') == 'PASS')
                and isinstance(row.get('cleanup'), dict) and row['cleanup'].get('status') == 'PASS'
                for row in selected)
            eligible = collected and matching and declaration_valid
            median = statistics.median(values) if complete else None
            maximum = max(values) if complete else None
            status = ('ELIGIBLE' if eligible else 'INCOMPLETE_OR_IDENTITY_OR_DECLARATION_FAILURE')
            result.append({
                'case': case, 'arm': arm, 'count': len(selected), 'runs': [row['run'] for row in selected],
                'observations_ns': values, 'median_ns': median, 'maximum_ns': maximum,
                'identity_match': matching, 'collection_and_cleanup_complete': collected,
                'cohort_status': status,
                'main_target_ns': TARGET[case] if arm == 'LayerFS' else None,
                'stretch_target_ns': STRETCH[case] if arm == 'LayerFS' else None,
                'main_target_status': ('PASS' if median <= TARGET[case] else 'TARGET_MISS')
                    if eligible and arm == 'LayerFS' else ('NOT_APPLICABLE' if arm == 'native' else 'NOT_ESTABLISHED'),
                'stretch_target_status': ('PASS' if median <= STRETCH[case] else 'TARGET_MISS')
                    if eligible and arm == 'LayerFS' else ('NOT_APPLICABLE' if arm == 'native' else 'NOT_ESTABLISHED'),
                'individual_main_misses': [row['run'] for row in selected
                    if arm == 'LayerFS' and isinstance(duration(row), int) and duration(row) > TARGET[case]],
            })
    return result


def table(headers, values):
    def cell(value):
        return str(value).replace('|', '\\|').replace('\n', ' ')
    return '\n'.join(['| ' + ' | '.join(headers) + ' |', '| ' + ' | '.join(['---'] * len(headers)) + ' |']
                     + ['| ' + ' | '.join(map(cell, row)) + ' |' for row in values])


def ms(value):
    return 'unavailable' if value is None else f'{value / 1_000_000:.3f}'


def render(report):
    rows = report['rows']
    lines = ['# Fixed Git-100 / Git-500 cohort', '',
        'Timing tables display milliseconds; all target decisions use unrounded integer nanoseconds. '
        'Before observations are n=1. Final statistics use only the twelve explicitly named slots, '
        'with three repetitions per case and arm. Missing or invalid observations are not replaced.', '',
        '**Scopes differ:** LayerFS is Create + required cold hydration + apply + six Git commands + '
        'LayerFS Commit + visibility + End. Native is apply + six Git commands only. '
        'Native is not a complete LayerFS lifecycle and receives no lifecycle target classification.', '',
        '## Observations', '', table(
            ['Run', 'Arm', 'Create', 'Exec', 'LayerFS Commit', 'Visibility', 'End', 'Total', 'Main gate'],
            [[row['run'], row['arm'], *[ms(next((phase['elapsed_ns'] for phase in row.get('phases', [])
                if phase['phase'] == name), None)) for name in ('create', 'exec', 'commit', 'visibility', 'end')],
              ms(duration(row)), row.get('main_target_status', 'N/A')] for row in rows]), '',
        '## Apply and all six Git commands', '', table(
            ['Run', 'Apply', 'First status', 'Diff', 'Add', 'Cached check', 'Git commit', 'Final status', 'Workload'],
            [[row['run'], *[ms(workload.get(key)) for key in ['apply_ns'] + GIT + ['workload_ns']]]
             for row in rows for workload in row.get('workload_receipts', [])]), '',
        '## Final cohort median and maximum', '', table(
            ['Case', 'Arm', 'n', 'Median ns', 'Maximum ns', 'Median ms', 'Maximum ms', 'Main', 'Stretch', 'Cohort'],
            [[item['case'], item['arm'], item['count'], item['median_ns'], item['maximum_ns'],
              ms(item['median_ns']), ms(item['maximum_ns']), item['main_target_status'],
              item['stretch_target_status'], item['cohort_status']] for item in report['statistics']]), '',
        'Individual main-target misses: ' + json.dumps({item['case']: item['individual_main_misses']
            for item in report['statistics'] if item['arm'] == 'LayerFS'}) + '.', '',
        '## Backing, canonical database and cache work', '',
        'Database counters describe SnapshotReader queries and authenticated returned objects; '
        'they are not a count of all SQLite work. Nested receipt durations overlap and must not be added to lifecycle totals.', '',
        table(['Run', 'Backing calls', 'Request bytes', 'Backing wait ns', 'Host dispatch ns',
               'DB calls', 'DB rows', 'DB bytes', 'Cache hits', 'Cache bytes', 'Local auth/read ns'],
              [[row['run'], *[group(row, 'FuseWriteReceipt').get(key) for key in
                ('live_backing_calls', 'live_backing_request_bytes', 'live_backing_wait_ns', 'host_dispatch_ns')],
                *[group(row, 'WorkspaceReadReceipt').get(key) for key in
                ('snapshot_database_calls', 'snapshot_database_rows', 'snapshot_database_bytes',
                 'snapshot_cache_hits', 'snapshot_cache_bytes', 'local_read_auth_ns')]]
               for row in rows if row['arm'] == 'LayerFS']), '',
        '## Resource scopes', '',
        'Host CPU deltas bracket before → after-product and include in-loop observations. '
        'LayerFS container CPU is the runner command window. Native CPU brackets its workload; '
        'container lifetime peak memory also includes preparation/verification. Host CPU is not container-capped.', '',
        table(['Run', 'Host user CPU ns', 'Host system CPU ns', 'Host after-product RSS bytes',
               'Container CPU ns', 'Container lifetime peak bytes', 'Cleanup'],
              [[row['run'], row.get('host_resource_deltas', {}).get('user_cpu_ns'),
                row.get('host_resource_deltas', {}).get('system_cpu_ns'),
                row.get('host_resources', {}).get('after-product', {}).get('resident_bytes'),
                (row.get('container_resources') or {}).get('command_window_cpu_ns',
                    row.get('native_resources', {}).get('workload_cpu_ns')),
                (row.get('container_resources') or {}).get('sample_container_lifetime_peak_bytes',
                    row.get('native_resources', {}).get('container_lifetime_peak_bytes')),
                row.get('cleanup')] for row in rows]), '',
        '## Source and fixture identities', '', table(
            ['Run', 'Source', 'Product', 'Image', 'Input', 'Raw SHA-256'],
            [[row['run'], *[row['identities'].get(key) for key in
                ('source_identity', 'product_identity', 'image', 'input_identity')], row['raw_sha256']] for row in rows]), '',
        '## Evidence limits and separate correctness receipts', '',
        'Performance collection PASS and the historical 15-second classifier are not acceptance against '
        '500,000,000 ns / 1,000,000,000 ns. Performance does not establish Git correctness, issue closure or publication. '
        'Separate proof receipts below are retained without treating their durations as performance observations.', '',
        '```json', json.dumps({key: report[key] for key in
            ('declaration', 'missing_runs', 'parse_errors', 'proofs')}, indent=2), '```', '',
        'Legacy zero counters known to be unwired are null/unavailable in derived metric groups; '
        'raw receipts retain their original zeros. All metric groups, lifecycle phases, workload counters, '
        'resource observations, cache policies and identities are retained in [report.json](report.json).', '',
        '## Raw evidence hashes', '', table(['Relative path', 'Bytes', 'SHA-256'],
            [[entry['path'], entry['bytes'], entry['sha256']] for entry in report['raw_files']]), '']
    return '\n'.join(lines)


def main():
    global ROOT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('evidence_root', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--declaration', type=Path)
    parser.add_argument('--proof', action='append', default=[], help='Relative JSON correctness receipt; never enters timing statistics')
    args = parser.parse_args()
    ROOT = args.evidence_root.resolve()
    rows, missing, errors = [], [], []
    for run in BASELINE_RUNS + FINAL_RUNS:
        try:
            row = load_run(run)
            if row is None:
                missing.append(run)
            else:
                tier = 100 if '100' in run.split('-') else 500
                expected_case = f'git-tool-{tier}-mixed-v4'
                if row['case'] != expected_case or row['identities'].get('case') != expected_case:
                    raise ValueError(f'{run}: case mismatch')
                if row['identities'].get('seed') != 1:
                    raise ValueError(f'{run}: expected seed 1')
                rows.append(row)
        except (ValueError, KeyError, TypeError, AssertionError, OSError) as error:
            errors.append({'run': run, 'error': str(error)})
    declaration = {'status': 'NOT_PROVIDED'}
    if args.declaration:
        path = args.declaration if args.declaration.is_absolute() else ROOT / args.declaration
        path = path.resolve()
        try:
            raw = read(path)
            declared = raw.get('runs', [])
            valid = isinstance(declared, list) and len(declared) == len(FINAL_RUNS) and set(declared) == set(FINAL_RUNS)
            declaration = {'status': 'EXACT_RUN_SET' if valid else 'RUN_SET_MISMATCH',
                           'raw': str(path.relative_to(ROOT)), 'receipt': raw,
                           'limit': 'The supplied declaration must be independently shown to precede collection; this generator does not infer that from filesystem timestamps.'}
        except (ValueError, TypeError, OSError) as error:
            declaration = {'status': 'INVALID', 'error': str(error)}
    proofs = []
    for relative in args.proof:
        try:
            path = (ROOT / relative).resolve()
            proofs.append({'raw': str(path.relative_to(ROOT)), 'receipt': read(path),
                           'status': 'SEPARATE_PROOF_NOT_A_TIMING_OBSERVATION'})
        except (ValueError, TypeError, OSError) as error:
            errors.append({'proof': relative, 'error': str(error)})
    report = {'schema': 'layerfs-issue68-fixed-cohort-v1', 'rows': rows,
              'selection': {'baseline': BASELINE_RUNS, 'final': FINAL_RUNS},
              'missing_runs': missing, 'parse_errors': errors, 'declaration': declaration,
              'statistics': cohort_statistics(rows, declaration['status'] == 'EXACT_RUN_SET'),
              'proofs': proofs, 'raw_files': sorted(RAW.values(), key=lambda value: value['path']),
              'issue_closure': 'NOT_DECIDED_BY_THIS_GENERATOR',
              'timing_units': 'integer nanoseconds; display milliseconds only',
              'zero_metrics': 'Known unwired zero fields are null with field-name annotations; raw files preserve exact evidence.'}
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    (args.output / 'report.md').write_text(render(report))
    print(json.dumps({'rows': len(rows), 'missing_runs': missing, 'parse_errors': errors,
                      'output': str(args.output)}))


if __name__ == '__main__':
    main()

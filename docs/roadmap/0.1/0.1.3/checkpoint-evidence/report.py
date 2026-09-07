#!/usr/bin/env python3
"""Derive checkpoint tables from the frozen registry and immutable runner receipts."""
import argparse
from collections import Counter, defaultdict
import csv
import gzip
import hashlib
import importlib.util
import json
import statistics
from pathlib import Path
import sys

REPO = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(REPO / 'benchmark/fs-bench-pro/shared'))
import runner
spec = importlib.util.spec_from_file_location('git_report', Path(__file__).parents[1] / 'issue68-evidence/derive.py')
git_report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(git_report)


def existing(path):
    return path if path.exists() else path.with_name(path.name + '.gz')


def read(path):
    text = gzip.decompress(path.read_bytes()).decode() if path.suffix == '.gz' else path.read_text()
    if path.name.removesuffix('.gz').endswith('.jsonl'):
        return [json.loads(line) for line in text.splitlines() if line.strip()]
    return json.loads(text)


def endpoints(rows, key):
    groups = defaultdict(list)
    for row in rows:
        groups[key(row)].append(row)
    return [{'group': name, 'record_count': len(values), 'first': values[0], 'last': values[-1]}
            for name, values in groups.items()]



def proof_coverage(receipt):
    checks = receipt.get('checks') or []
    history = next((r for r in checks if r.get('kind') == 'fast-history-complete'), None)
    sdk = next((r for r in checks if 'verified_start' in r and 'verified_end' in r), None)
    kinds = {r.get('kind') for r in checks}
    if history:
        return {'type': 'sampled-history', 'summary': 'all parent links; snapshots ' + str(history['verified_steps']),
                'verified_steps': history['verified_steps'], 'omissions': history.get('omissions')}
    if sdk:
        path = sdk.get('sampled_path', 'payload.bin')
        return {'type': 'bounded-content', 'summary': f"bounded bytes; {path} [{sdk['verified_start']}, {sdk['verified_end']})",
                'path': path, 'start': sdk['verified_start'], 'end': sdk['verified_end'],
                'schema': sdk.get('schema'), 'cache_scope': sdk.get('verification_cache_scope'),
                'profile': sdk.get('verification_profile'),
                'omissions': ['full-file byte replay'] + ([sdk['pre_edit_fuse_inode_stability']] if 'pre_edit_fuse_inode_stability' in sdk else [])}
    if 'git-semantic-verification' in kinds and 'git-reopen-custody' in kinds:
        return {'type': 'full-git-contract', 'summary': 'full head/tree/parent and reopened custody'}
    if 'fast-verification-complete' in kinds:
        return {'type': 'sampled-content', 'summary': 'changed data and selected unchanged witnesses; qualified roots',
                'details': [r for r in checks if r.get('kind') in ('fast-canonical-verification', 'fast-native-verification')]}
    if 'proof-complete' in kinds:
        return {'type': 'targeted-contract', 'summary': 'case-specific lifecycle/fault/recovery contract',
                'details': [r for r in checks if r.get('kind') in ('proof-complete', 'fault-reachability')]}
    return {'type': 'registered-proof', 'summary': 'registered verification; detailed checks in receipt'}


def previous_results():
    """Published host-store observations; comparisons remain descriptive."""
    base = Path(__file__).parents[1]
    result = {}
    for filename in ('issue38-main-refresh-results.json', 'issue54-remaining-family-stats.json'):
        data = read(base / filename)
        rows = data['performance']
        if isinstance(rows, dict):
            rows = rows['results']
        for row in rows:
            if row.get('status') != 'PASS' or row.get('family') == 'git_tool_workflow':
                continue
            result[row['case']] = {'elapsed_ns': row['elapsed_ns'], 'timer': row['timer'],
                                  'sample_count': 1, 'source_identity': row.get('source_identity') or row.get('identities', {}).get('source_identity'),
                                  'report': '../' + filename, 'scope': 'historical host-store observation; original verification limits retained'}
    for path in sorted(base.glob('issue62-*-results.json')) + [base / 'issue65-unrelated-mixed-v2-results.json']:
        data = read(path)
        for row in data['cases']:
            case = row['case_id']
            if case.startswith('git-tool-') or row.get('status') not in ('PASS', 'COMPLETE'):
                continue
            result[case] = {'elapsed_ns': row['pure_call_sum_ns'], 'timer': row['timer'],
                            'sample_count': 1, 'fixture_bytes': row['fixture_bytes'],
                            'fixture_files': row['fixture_files'], 'report': '../' + path.name,
                            'source_identity': data.get('source_identity') or data.get('identities', {}).get('source_identity'),
                            'scope': 'same versioned workload, host-store; sampled verification'}
    data = read(base / 'issue68-evidence/report.json')
    for case in ('git-tool-100-mixed-v4', 'git-tool-500-mixed-v4'):
        rows = [r for r in data['rows'] if r.get('case') == case and r.get('run', '').startswith('final-') and r.get('arm') == 'LayerFS']
        if len(rows) != 3:
            raise ValueError('published corrected Git cohort cardinality')
        result[case] = {'elapsed_ns': statistics.median(r['complete_lifecycle_ns'] for r in rows),
                        'timer': 'pure_call_sum_ns', 'sample_count': 3,
                        'source_identity': rows[0]['identities']['source_identity'],
                        'report': '../issue68-evidence/report.json',
                        'scope': 'corrected-input host-store final median, full Git proofs; compared to one checkpoint observation'}
    return result


def derive(campaign, registry):
    rows, proofs, errors, evidence = [], [], [], {}
    identities = set()
    previous = previous_results()
    admitted = [r for r in registry if r['family_id'] in runner.HOST_FAMILIES]
    keys = [(r['family_id'], r['scenario_id']) for r in admitted]
    if len(keys) != len(set(keys)):
        raise ValueError('duplicate registry case')

    def load(path):
        path = existing(path)
        if not path.exists():
            errors.append(f'missing receipt: {path.relative_to(campaign)}')
            return None
        evidence[str(path.relative_to(campaign))] = hashlib.sha256(path.read_bytes()).hexdigest()
        return read(path)

    declaration = load(campaign / 'declaration.json') or {}
    if [r['scenario_id'] for r in admitted if not r['proof_only']] != declaration.get('performance_order'):
        errors.append('performance registry does not match the frozen declaration')
    if [r['scenario_id'] for r in admitted] != declaration.get('verification_order'):
        errors.append('verification registry does not match the frozen declaration')
    requalification = load(campaign / 'sdk-requalification.json') if (campaign / 'sdk-requalification.json').exists() else None
    if requalification:
        expected_cases = {r['scenario_id'] for r in admitted if r.get('route') == 'sdk'}
        if (set(requalification.get('cases', {})) != expected_cases
                or requalification.get('old_host') != declaration.get('host')
                or requalification.get('new_host', {}).get('LAYERFS_PRODUCT_SEAL') != declaration.get('host', {}).get('LAYERFS_PRODUCT_SEAL')
                or requalification.get('harness_identity') != declaration.get('harness_identity')
                or requalification.get('changed_execution_files') != ['benchmark/fs-bench-pro/src/sdk_edit_verify.rs']):
            errors.append('invalid SDK verifier-only requalification scope')
    native_reference = {}
    old_git = read(Path(__file__).parents[1] / 'issue68-evidence/report.json')
    for case in git_report.TARGET:
        native = [r for r in old_git['rows'] if r.get('case') == case and r.get('arm') == 'native' and r.get('run', '').startswith('final-')]
        if len(native) != 3:
            raise ValueError('published native Git cohort cardinality')
        native_reference[case] = {'median_ns': statistics.median(r['native_workload_ns'] for r in native),
                                  'sample_count': 3, 'report': '../issue68-evidence/report.json',
                                  'source_identity': native[0]['identities']['source_identity'],
                                  'scope': 'historical native Apply + six Git commands; excludes LayerFS lifecycle; reference, not a matched total-latency comparison'}
    for definition in admitted:
        family, case = definition['family_id'], definition['scenario_id']
        proof_path = campaign / 'verification' / family / case / 'verification.json'
        replacement = (requalification or {}).get('cases', {}).get(case)
        original_receipt = None
        if replacement:
            original_receipt = load(proof_path)
            proof_path = campaign / replacement['receipt']
            if replacement.get('recipe') != definition or replacement.get('family') != family:
                errors.append(f'requalification recipe mismatch: {case}')
            for prefix, host in (('old', declaration.get('host', {})), ('new', requalification['new_host'])):
                recipe = {'family': family, 'case': case, 'seed': 1,
                          'source': host.get('LAYERFS_SOURCE_SEAL'), 'recipe': definition}
                if replacement.get(prefix + '_input_identity') != runner.digest(recipe):
                    errors.append(f'requalification input recipe hash mismatch: {case}')
        if not definition['verification_supported']:

            if case != declaration.get('long_test_exclusion'):
                errors.append(f'undeclared verification exclusion: {case}')
            proof = {'family': family, 'case': case, 'status': 'EXCLUDED_LONG',
                     'reason': '600-second endurance definition outside routine checkpoint; not executed'}
        else:
            receipt = load(proof_path)
            proof = {'family': family, 'case': case, 'status': receipt.get('status') if receipt else 'NOT_RUN'}
            if receipt:
                proof.update(wall_seconds=receipt.get('wall_seconds'), error=receipt.get('error'),
                             cleanup=receipt.get('cleanup'), omissions=receipt.get('omissions'),
                             sampled_paths_or_ranges=receipt.get('sampled_paths_or_ranges'),
                             evidence=str(existing(proof_path).relative_to(campaign)),
                             source_identity=receipt.get('source_identity'),
                             product_identity=receipt.get('product_identity'),
                             input_identity=receipt.get('input_identity'),
                             image_identity=receipt.get('image_identity'),
                             checks=[r for r in receipt.get('checks', []) if any(word in str(r.get('kind', '')) for word in ('verif', 'proof', 'fault', 'history-complete')) or r.get('receipt_kind') == 'source-arm-subproof'], coverage=proof_coverage(receipt),
                             preparation_wall_ns=receipt.get('preparation_wall_ns'), resources=receipt.get('resources'))
                if receipt.get('case') != case or receipt.get('family') != family:
                    errors.append(f'proof identity mismatch: {case}')
                expected = requalification['new_host'] if replacement else declaration.get('host', {})
                if (receipt.get('source_identity') != expected.get('LAYERFS_SOURCE_SEAL')
                        or receipt.get('product_identity') != expected.get('LAYERFS_PRODUCT_SEAL')
                        or receipt.get('image_identity') != declaration.get('image')
                        or receipt.get('harness_identity') != declaration.get('harness_identity')):
                    errors.append(f'proof differs from frozen source/harness/image: {case}')
                if replacement:
                    proof['requalification'] = replacement
                    proof['original_status'] = (original_receipt or {}).get('status')
                    if (receipt.get('input_identity') != replacement['new_input_identity']
                            or (original_receipt or {}).get('input_identity') != replacement['old_input_identity']
                            or (original_receipt or {}).get('source_identity') != declaration['host']['LAYERFS_SOURCE_SEAL']
                            or not receipt.get('preparation', {}).get('fixture')
                            or receipt.get('preparation', {}).get('fixture') != (original_receipt or {}).get('preparation', {}).get('fixture')):
                        errors.append(f'requalification proof/fixture mismatch: {case}')
                if receipt.get('status') != 'PASS' or receipt.get('cleanup', {}).get('status') != 'PASS':
                    errors.append(f'proof did not pass: {case}')
        proofs.append(proof)
        if definition['proof_only']:
            continue
        path = campaign / 'performance' / family / case / 'perf.jsonl'
        data = load(path)
        row = {'family': family, 'case': case, 'definition': definition,
               'status': 'NOT_RUN', 'verification': proof['status'], 'sample_count': 0,
               'timer': None, 'elapsed_ns': None,
               **{key: definition.get(key) for key in ('tier', 'fixture_bytes', 'fixture_files', 'fixture_profile')},
               'proof_evidence': proof.get('evidence'), 'seed': declaration.get('seed')}
        rows.append(row)
        if data is None:
            continue
        samples = [r for r in data if r.get('kind') == 'sample']
        if len(samples) != 1:
            errors.append(f'expected exactly one sample: {case}')
            continue
        if not data or data[-1].get('kind') != 'summary' or data[-1].get('status') != 'PASS':
            errors.append(f'missing or failed performance completion summary: {case}')
        sample = samples[0]
        identity = sample.get('identities', {})
        identities.add((identity.get('source_identity'), identity.get('product_identity'), identity.get('image')))
        if identity.get('case') != case or identity.get('family') != family:
            errors.append(f'performance identity mismatch: {case}')
        bound_source = declaration['host']['LAYERFS_SOURCE_SEAL'] if replacement else proof.get('source_identity')
        bound_input = replacement['old_input_identity'] if replacement else proof.get('input_identity')
        if (bound_source != identity.get('source_identity') or bound_input != identity.get('input_identity')
                or proof.get('product_identity') != identity.get('product_identity')
                or proof.get('image_identity') != identity.get('image')):
            errors.append(f'performance/proof source mismatch: {case}')
        expected = declaration.get('host', {})
        if (identity.get('source_identity') != expected.get('LAYERFS_SOURCE_SEAL')
                or identity.get('product_identity') != expected.get('LAYERFS_PRODUCT_SEAL')
                or identity.get('image') != declaration.get('image')
                or identity.get('harness_identity') != declaration.get('harness_identity')):
            errors.append(f'performance differs from frozen source/harness/image: {case}')
        timer, elapsed = runner._timer(sample)
        records = sample.get('records', [])
        target = git_report.TARGET.get(case, runner.PRODUCT_TARGET_NS)
        row.update(status=sample.get('status'), sample_count=1, timer=timer, elapsed_ns=elapsed,
                   target_ns=target, target_status=('PASS' if elapsed <= target else 'TARGET_MISS') if elapsed is not None else 'UNAVAILABLE',
                   identities=identity, phases=[r for r in records if r.get('kind') == 'phase'],
                   workload_receipts=[git_report.kv(r['workload_receipt']) for r in records if r.get('kind') == 'phase' and 'workload_receipt' in r],
                   metric_groups=git_report.metric_groups(records),
                   host_resources=[r for r in records if r.get('kind') == 'host-resources'],
                   route_metrics=[r for r in records if timer in r or r.get('receipt_kind') == 'performance' or any(key in r for key in ('process_peak_rss_bytes', 'process_t1_peak_rss_bytes', 'process_lifetime_peak_rss_bytes'))],
                   container_resources=sample.get('resources'), cleanup=sample.get('cleanup'),
                   preparation_wall_ns=sample.get('preparation_wall_ns'), command_wall_ns=sample.get('command_wall_ns'),
                   setup=sample.get('setup'), preparation=sample.get('preparation'),
                   prepared_master_unchanged=sample.get('prepared_master_unchanged'),
                   additional_target_assessments=sample.get('issue47_assessment'),
                   observations=[r for r in records if r.get('kind') in ('store-observation', 'workspace-spool-observation', 'workspace-physical-spool')],
                   evidence=str(existing(path).relative_to(campaign)), error=sample.get('error'),
                   unavailable_policy='Absent fields are unavailable or inapplicable; no zero is inferred.')
        phase_totals = defaultdict(int)
        for phase in row['phases']:
            phase_totals[phase['phase']] += phase['elapsed_ns']
        for phase in ('create', 'exec', 'sdk-edit', 'commit', 'visibility', 'end'):
            row[phase.replace('-', '_') + '_ns'] = phase_totals.get(phase)
        for record in row['route_metrics']:
            for phase, key in (('create', 'workspace_create_ns'), ('sdk_edit', 'edit_call_ns'),
                               ('commit', 'commit_call_ns'), ('visibility', 'visibility_validation_ns'), ('end', 'workspace_end_ns')):
                if isinstance(record.get(key), int): row[phase + '_ns'] = record[key]
        host_peaks = [r['peak_resident_bytes'] for r in row['host_resources'] if isinstance(r.get('peak_resident_bytes'), int)]
        host_peaks += [r[key] for r in row['route_metrics'] for key in ('process_lifetime_peak_rss_bytes', 'process_t1_peak_rss_bytes', 'process_peak_rss_bytes') if isinstance(r.get(key), int)]
        row['host_peak_rss_bytes'] = max(host_peaks) if host_peaks else None
        row['container_peak_bytes'] = (sample.get('resources') or {}).get('sample_container_lifetime_peak_bytes')
        row['container_cpu_ns'] = (sample.get('resources') or {}).get('command_window_cpu_ns')
        row['host_cpu_ns'] = None
        row['host_cpu_scope'] = 'unavailable: no host CPU counter in this route receipt'
        host = {r.get('phase'): r for r in row['host_resources']}
        before, after = host.get('before', {}), host.get('after-product', {})
        if all(k in before and k in after for k in ('user_cpu_ns', 'system_cpu_ns')):
            row['host_cpu_ns'] = sum(after[k] - before[k] for k in ('user_cpu_ns', 'system_cpu_ns'))
            row['host_cpu_scope'] = 'host process after-product minus before counters'
        for record in row['route_metrics']:
            for prefix, scope in (('initialization_', 'native initialization phase'), ('process_', 'host process cumulative at observation')):
                keys = [prefix + 'user_cpu_ns', prefix + 'system_cpu_ns']
                if all(isinstance(record.get(k), int) for k in keys):
                    row['host_cpu_ns'] = sum(record[k] for k in keys)
                    row['host_cpu_scope'] = scope
        row['verification_wall_seconds'] = proof.get('wall_seconds')
        row['cleanup_status'] = (sample.get('cleanup') or {}).get('status')
        row['cleanup_wall_ns'] = (sample.get('cleanup') or {}).get('wall_ns')
        row['coverage'] = proof.get('coverage')
        for key in ('source_identity', 'product_identity', 'harness_identity', 'input_identity', 'image'):
            row[key] = identity.get(key)
        for key in ['apply_ns'] + git_report.GIT:
            values = [r[key] for r in row['workload_receipts'] if isinstance(r.get(key), int)]
            row[key] = sum(values) if values else None
        row['resource_scope'] = 'Host process high-water and container lifetime peak, not incremental per-operation memory'
        old = previous.get(case)
        compatible = old and old['timer'] == timer and all(
            old.get(key) in (None, definition.get(key)) for key in ('fixture_bytes', 'fixture_files'))
        row['previous'] = old if compatible else None
        row['comparison_reason'] = ('Same registered case/version, host-store and timer; descriptive cross-source comparison, not statistical speedup evidence'
                                    if compatible else 'No compatible published observation; old Git compact fixtures are excluded')
        row['previous_elapsed_ns'] = old['elapsed_ns'] if compatible else None
        row['difference_ns'] = elapsed - old['elapsed_ns'] if compatible and elapsed is not None else None
        row['difference_percent'] = 100 * row['difference_ns'] / old['elapsed_ns'] if row['difference_ns'] is not None and old['elapsed_ns'] else None
        resources = sample.get('resources') or {}
        route_resource_ok = all(str(record[key]).lower() == 'pass' for record in records
                                for key in ('resource_status', 'row_resource_status', 'cleanup_status') if key in record)
        if definition.get('route') == 'sdk':
            route_resource_ok = route_resource_ok and any(record.get('row_resource_status') == 'pass' for record in records)
        resource_ok = (route_resource_ok and sample.get('environment_observation', {}).get('validated') is True
                       and resources.get('oom_kill_delta') == 0
                       and resources.get('swap_current_bytes') == 0)
        row['resource_status'] = 'PASS' if resource_ok else 'INCOMPLETE_OR_FAIL'
        if not resource_ok:
            errors.append(f'missing or failed environment/resource evidence: {case}')
        if definition.get('setup_policy') != 'fresh-output' and sample.get('prepared_master_unchanged') is not True:
            errors.append(f'missing or failed master isolation evidence: {case}')
        if sample.get('status') != 'PASS' or elapsed is None or sample.get('cleanup', {}).get('status') != 'PASS':
            errors.append(f'performance did not pass: {case}')
        # Keep the table compact; full operation transcripts remain in raw receipts.
        row['phases'] = [{'phase': name, 'elapsed_ns': total,
                          'call_count': sum(p['phase'] == name for p in row['phases'])}
                         for name, total in phase_totals.items()]
        row['metric_groups'] = endpoints(row['metric_groups'], lambda r: (r['type'], r.get('operation')))
        row['observations'] = endpoints(row['observations'], lambda r: r['kind'])
        if family != 'git_tool_workflow':
            row['workload_receipts'] = endpoints(row['workload_receipts'], lambda r: 'workload')
        row['transcript_scope'] = 'Phase sums and call counts; metric/observation first and last records per group, not summed counters. Full transcript in linked raw receipt.'
    if len(identities) != 1 or any(value is None for identity in identities for value in identity):
        errors.append('checkpoint does not have one complete source/product/image identity')
    families = {}
    for family in sorted({r['family'] for r in rows + proofs}):
        performance = [r for r in rows if r['family'] == family]
        verification = [r for r in proofs if r['family'] == family]
        times = [r['elapsed_ns'] for r in performance if r['elapsed_ns'] is not None]
        families[family] = {'performance_cases': len(performance), 'verification_rows': len(verification),
                            'proof_only_definitions': len(verification) - len(performance),
                            'coverage_types': dict(Counter(r.get('coverage', {}).get('type', 'excluded') for r in verification)),
                            'latency_target_statuses': dict(Counter(r.get('target_status', 'UNAVAILABLE') for r in performance)),
                            'performance_statuses': dict(Counter(r['status'] for r in performance)),
                            'verification_statuses': dict(Counter(r['status'] for r in verification)),
                            'across_case_min_ns': min(times) if times else None,
                            'across_case_max_ns': max(times) if times else None}
    return {'schema': 'layerfs-checkpoint-74-75-v1', 'status': 'PASS' if not errors else 'INCOMPLETE',
            'errors': errors, 'families': families, 'performance': rows, 'verification': proofs,
            'raw_sha256': evidence, 'identities': sorted(identities), 'declaration': declaration,
            'sdk_requalification': requalification, 'native_git_reference': native_reference,
            'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            'qualification_limit': 'Routine fixed-seed checkpoint; no three-seed scaling qualification, percentile distribution or exhaustive history claim',
            'excluded_registry_entries': [r['scenario_id'] for r in registry if r['family_id'] not in runner.HOST_FAMILIES]}


def milliseconds(value):
    return '—' if value is None else f'{value / 1_000_000:.3f}'


def write(report, output):
    output.mkdir(parents=True, exist_ok=True)
    (output / 'report.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    columns = ['family', 'case', 'tier', 'fixture_bytes', 'fixture_files', 'fixture_profile', 'seed', 'sample_count', 'timer', 'elapsed_ns', 'status', 'verification', 'target_ns', 'target_status', 'preparation_wall_ns', 'command_wall_ns', 'create_ns', 'exec_ns', 'sdk_edit_ns', 'commit_ns', 'visibility_ns', 'end_ns', 'host_peak_rss_bytes', 'container_peak_bytes', 'host_cpu_ns', 'host_cpu_scope', 'container_cpu_ns', 'verification_wall_seconds', 'apply_ns', *git_report.GIT, 'previous_elapsed_ns', 'difference_ns', 'difference_percent', 'resource_status', 'cleanup_status', 'cleanup_wall_ns', 'source_identity', 'product_identity', 'harness_identity', 'input_identity', 'image', 'proof_evidence', 'evidence']
    with (output / 'performance.csv').open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=columns, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(report['performance'])
    with (output / 'verification.csv').open('w', newline='') as stream:
        columns = ['family', 'case', 'status', 'wall_seconds', 'source_identity', 'product_identity', 'input_identity', 'image_identity', 'evidence', 'omissions', 'sampled_paths_or_ranges', 'coverage', 'error']
        writer = csv.DictWriter(stream, fieldnames=columns, extrasaction='ignore')
        writer.writeheader()
        writer.writerows({**r, **{k: json.dumps(r.get(k), separators=(',', ':')) for k in ('omissions', 'sampled_paths_or_ranges', 'coverage')}} for r in report['verification'])
    text = ['# v0.1.3 benchmark checkpoint', '', f"Status: **{report['status']}**.", '',
            'One fixed-seed observation per case. Timers retain their family-specific scopes. Setup and verification are separate. Across-case ranges are not latency distributions. [JSON report](report.json), [performance CSV](performance.csv), and [verification CSV](verification.csv) contain phases, identities, resources, coverage and evidence hashes.', '',
            '| Family | Performance cases | Proof-only definitions | Performance outcomes | Verification outcomes | Target misses | Across-case range (ms) |',
            '|---|---:|---:|---|---|---:|---|']
    for family, row in report['families'].items():
        text.append(f"| {family} | {row['performance_cases']} | {row['proof_only_definitions']} | {row['performance_statuses']} | {row['verification_statuses']} | {row['latency_target_statuses'].get('TARGET_MISS', 0)} | {milliseconds(row['across_case_min_ns'])}–{milliseconds(row['across_case_max_ns'])} |")
    for family in report['families']:
        text += ['', f'## {family}', '', '| Test | Tier / files / bytes | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |', '|---|---|---|---:|---|---|---:|---|---|']
        for row in report['performance']:
            if row['family'] == family:
                phases = ' / '.join(milliseconds(row.get(key)) for key in ('exec_ns', 'sdk_edit_ns', 'commit_ns'))
                memory = ' / '.join('—' if row.get(key) is None else f"{row[key] / 2**20:.2f}" for key in ('host_peak_rss_bytes', 'container_peak_bytes'))
                label = f"[{row['case']}](raw/{row['evidence']})" if row.get('evidence') else row['case']
                size = ' / '.join(str(row.get(key)) if row.get(key) is not None else '—' for key in ('tier', 'fixture_files', 'fixture_bytes'))
                text.append(f"| {label} | {size} | {row['timer']} | {milliseconds(row['elapsed_ns'])} | {phases} | {memory} | {milliseconds(row.get('previous_elapsed_ns'))} | {row['status']} / {row['verification']} | {row.get('target_status', '—')} |")
    if report.get('sdk_requalification'):
        text += ['', 'SDK verification uses the versioned cold-projection proof on an explicitly compatible verifier-only source. All 56 SDK proofs were requalified; their original receipts, including ten resource failures from the pre-edit-lookup proof, remain in raw evidence. Product code, performance paths, fixtures, image and harness are unchanged. See sdk_requalification in the JSON report for both source and recipe seals.']
    text += ['', 'Historical subsecond bulk targets are separate from the family threshold:']
    for row in report['performance']:
        if row.get('additional_target_assessments'):
            assessment = row['additional_target_assessments']
            text.append(f"- {row['case']}: {assessment['status']} against strict <1,000 ms.")
    text += ['', '## Git stages', '', '| Test | Apply | First status | Diff | Add | Cached check | Git commit | Final status | LayerFS Commit |', '|---|---:|---:|---:|---:|---:|---:|---:|---:|']
    for row in report['performance']:
        if row['family'] == 'git_tool_workflow':
            values = [milliseconds(row.get(key)) for key in ['apply_ns'] + git_report.GIT + ['commit_ns']]
            text.append('| ' + row['case'] + ' | ' + ' | '.join(values) + ' |')
    text += ['', 'Historical native Git reference (three-run median):']
    text += [f"- {case}: {milliseconds(value['median_ns'])} ms, Apply + six Git commands. [Published source]({value['report']})." for case, value in report['native_git_reference'].items()]
    text += ['', 'Native references exclude LayerFS Create/Commit/visibility/End and are not matched total-lifecycle comparisons.']
    text += ['', 'All Git stage values are milliseconds. Git commit and LayerFS Commit are separate operations.']
    text += ['', '## Verification', '', '| Test | Result | Wall (s) | Coverage |', '|---|---|---:|---|']
    text += [f"| " + (f"[{r['case']}](raw/{r['evidence']})" if r.get('evidence') else r['case']) + f" | {r['status']} | {r.get('wall_seconds', '—')} | {r.get('coverage', {}).get('summary', 'excluded; not executed')} |" for r in report['verification']]
    if report['errors']:
        text += ['', '## Incomplete requirements', ''] + ['- ' + error for error in report['errors']]
    (output / 'report.md').write_text('\n'.join(text) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('campaign', type=Path)
    parser.add_argument('--registry', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    report = derive(args.campaign.resolve(), read(args.registry))
    write(report, args.output)
    raise SystemExit(0 if report['status'] == 'PASS' else 1)

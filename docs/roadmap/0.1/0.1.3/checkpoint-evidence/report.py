#!/usr/bin/env python3
"""Derive checkpoint tables from the frozen registry and immutable runner receipts."""
import argparse
from collections import Counter, defaultdict
import csv
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


def read(path):
    if path.suffix == '.jsonl':
        return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    return json.loads(path.read_text())


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
        if not path.exists():
            errors.append(f'missing receipt: {path.relative_to(campaign)}')
            return None
        evidence[str(path.relative_to(campaign))] = hashlib.sha256(path.read_bytes()).hexdigest()
        return read(path)

    for definition in admitted:
        family, case = definition['family_id'], definition['scenario_id']
        proof_path = campaign / 'verification' / family / case / 'verification.json'
        if not definition['verification_supported']:
            proof = {'family': family, 'case': case, 'status': 'EXCLUDED_LONG',
                     'reason': '600-second endurance definition outside routine checkpoint; not executed'}
        else:
            receipt = load(proof_path)
            proof = {'family': family, 'case': case, 'status': receipt.get('status') if receipt else 'NOT_RUN'}
            if receipt:
                proof.update(wall_seconds=receipt.get('wall_seconds'), error=receipt.get('error'),
                             cleanup=receipt.get('cleanup'), omissions=receipt.get('omissions'),
                             sampled_paths_or_ranges=receipt.get('sampled_paths_or_ranges'),
                             evidence=str(proof_path.relative_to(campaign)),
                             source_identity=receipt.get('source_identity'),
                             product_identity=receipt.get('product_identity'),
                             input_identity=receipt.get('input_identity'),
                             image_identity=receipt.get('image_identity'),
                             checks=receipt.get('checks'))
                if receipt.get('case') != case or receipt.get('family') != family:
                    errors.append(f'proof identity mismatch: {case}')
                if receipt.get('status') != 'PASS' or receipt.get('cleanup', {}).get('status') != 'PASS':
                    errors.append(f'proof did not pass: {case}')
        proofs.append(proof)
        if definition['proof_only']:
            continue
        path = campaign / 'performance' / family / case / 'perf.jsonl'
        data = load(path)
        row = {'family': family, 'case': case, 'definition': definition,
               'status': 'NOT_RUN', 'verification': proof['status'], 'sample_count': 0,
               'timer': None, 'elapsed_ns': None}
        rows.append(row)
        if data is None:
            continue
        samples = [r for r in data if r.get('kind') == 'sample']
        if len(samples) != 1:
            errors.append(f'expected exactly one sample: {case}')
            continue
        sample = samples[0]
        identity = sample.get('identities', {})
        identities.add((identity.get('source_identity'), identity.get('product_identity'), identity.get('image')))
        if identity.get('case') != case or identity.get('family') != family:
            errors.append(f'performance identity mismatch: {case}')
        if any(proof.get(key) != identity.get(key) for key in ('source_identity', 'product_identity', 'input_identity')) or proof.get('image_identity') != identity.get('image'):
            errors.append(f'performance/proof source mismatch: {case}')
        timer, elapsed = runner._timer(sample)
        records = sample.get('records', [])
        target = git_report.TARGET.get(case, runner.PRODUCT_TARGET_NS)
        row.update(status=sample.get('status'), sample_count=1, timer=timer, elapsed_ns=elapsed,
                   target_ns=target, target_status=('PASS' if elapsed <= target else 'TARGET_MISS') if elapsed is not None else 'UNAVAILABLE',
                   identities=identity, phases=[r for r in records if r.get('kind') == 'phase'],
                   workload_receipts=[git_report.kv(r['workload_receipt']) for r in records if r.get('kind') == 'phase' and 'workload_receipt' in r],
                   metric_groups=git_report.metric_groups(records),
                   host_resources=[r for r in records if r.get('kind') == 'host-resources'],
                   route_metrics=[r for r in records if timer in r or r.get('receipt_kind') == 'performance'],
                   container_resources=sample.get('resources'), cleanup=sample.get('cleanup'),
                   preparation_wall_ns=sample.get('preparation_wall_ns'), command_wall_ns=sample.get('command_wall_ns'),
                   setup=sample.get('setup'), preparation=sample.get('preparation'),
                   prepared_master_unchanged=sample.get('prepared_master_unchanged'),
                   observations=[r for r in records if r.get('kind') in ('store-observation', 'workspace-spool-observation', 'workspace-physical-spool')],
                   evidence=str(path.relative_to(campaign)), error=sample.get('error'),
                   unavailable_policy='Absent fields are unavailable or inapplicable; no zero is inferred.')
        phase_totals = defaultdict(int)
        for phase in row['phases']:
            phase_totals[phase['phase']] += phase['elapsed_ns']
        for phase in ('create', 'exec', 'sdk-edit', 'commit', 'visibility', 'end'):
            row[phase.replace('-', '_') + '_ns'] = phase_totals.get(phase)
        for record in row['route_metrics']:
            for phase, key in (('create', 'workspace_create_ns'), ('sdk_edit', 'edit_call_ns'),
                               ('commit', 'commit_call_ns'), ('end', 'workspace_end_ns')):
                if isinstance(record.get(key), int): row[phase + '_ns'] = record[key]
        host_peaks = [r['peak_resident_bytes'] for r in row['host_resources'] if isinstance(r.get('peak_resident_bytes'), int)]
        host_peaks += [r['process_lifetime_peak_rss_bytes'] for r in row['route_metrics'] if isinstance(r.get('process_lifetime_peak_rss_bytes'), int)]
        row['host_peak_rss_bytes'] = max(host_peaks) if host_peaks else None
        row['container_peak_bytes'] = (sample.get('resources') or {}).get('sample_container_lifetime_peak_bytes')
        row['verification_wall_seconds'] = proof.get('wall_seconds')
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
        resource_ok = (sample.get('environment_observation', {}).get('validated') is True
                       and resources.get('oom_kill_delta') == 0
                       and resources.get('swap_current_bytes') == 0)
        row['resource_status'] = 'PASS' if resource_ok else 'INCOMPLETE_OR_FAIL'
        if not resource_ok:
            errors.append(f'missing or failed environment/resource evidence: {case}')
        if definition.get('setup_policy') != 'fresh-output' and sample.get('prepared_master_unchanged') is not True:
            errors.append(f'missing or failed master isolation evidence: {case}')
        if sample.get('status') != 'PASS' or elapsed is None or sample.get('cleanup', {}).get('status') != 'PASS':
            errors.append(f'performance did not pass: {case}')
    if len(identities) != 1 or any(value is None for identity in identities for value in identity):
        errors.append('checkpoint does not have one complete source/product/image identity')
    families = {}
    for family in sorted({r['family'] for r in rows + proofs}):
        performance = [r for r in rows if r['family'] == family]
        verification = [r for r in proofs if r['family'] == family]
        times = [r['elapsed_ns'] for r in performance if r['elapsed_ns'] is not None]
        families[family] = {'performance_cases': len(performance), 'verification_rows': len(verification),
                            'performance_statuses': dict(Counter(r['status'] for r in performance)),
                            'verification_statuses': dict(Counter(r['status'] for r in verification)),
                            'across_case_min_ns': min(times) if times else None,
                            'across_case_max_ns': max(times) if times else None}
    return {'schema': 'layerfs-checkpoint-74-75-v1', 'status': 'PASS' if not errors else 'INCOMPLETE',
            'errors': errors, 'families': families, 'performance': rows, 'verification': proofs,
            'raw_sha256': evidence, 'identities': sorted(identities),
            'excluded_registry_entries': [r['scenario_id'] for r in registry if r['family_id'] not in runner.HOST_FAMILIES]}


def milliseconds(value):
    return '—' if value is None else f'{value / 1_000_000:.3f}'


def write(report, output):
    output.mkdir(parents=True, exist_ok=True)
    (output / 'report.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    columns = ['family', 'case', 'sample_count', 'timer', 'elapsed_ns', 'status', 'verification', 'target_ns', 'target_status', 'preparation_wall_ns', 'command_wall_ns', 'create_ns', 'exec_ns', 'sdk_edit_ns', 'commit_ns', 'visibility_ns', 'end_ns', 'host_peak_rss_bytes', 'container_peak_bytes', 'verification_wall_seconds', 'apply_ns', *git_report.GIT, 'previous_elapsed_ns', 'difference_ns', 'difference_percent', 'evidence']
    with (output / 'performance.csv').open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=columns, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(report['performance'])
    text = ['# v0.1.3 benchmark checkpoint', '', f"Status: **{report['status']}**.", '',
            'One fixed-seed observation per case. Timers retain their family-specific scopes. Setup and verification are separate. Across-case ranges are not latency distributions. See report.json for phases, identities, resources, coverage and evidence hashes.', '',
            '| Family | Performance cases | Performance outcomes | Verification outcomes | Across-case range (ms) |',
            '|---|---:|---|---|---|']
    for family, row in report['families'].items():
        text.append(f"| {family} | {row['performance_cases']} | {row['performance_statuses']} | {row['verification_statuses']} | {milliseconds(row['across_case_min_ns'])}–{milliseconds(row['across_case_max_ns'])} |")
    for family in report['families']:
        text += ['', f'## {family}', '', '| Test | Timer | Time (ms) | Exec / SDK / Commit (ms) | Host / container peak (MiB) | Previous (ms) | Execution / verification | Target |', '|---|---|---:|---|---|---:|---|---|']
        for row in report['performance']:
            if row['family'] == family:
                phases = ' / '.join(milliseconds(row.get(key)) for key in ('exec_ns', 'sdk_edit_ns', 'commit_ns'))
                memory = ' / '.join('—' if row.get(key) is None else f"{row[key] / 2**20:.2f}" for key in ('host_peak_rss_bytes', 'container_peak_bytes'))
                text.append(f"| {row['case']} | {row['timer']} | {milliseconds(row['elapsed_ns'])} | {phases} | {memory} | {milliseconds(row.get('previous_elapsed_ns'))} | {row['status']} / {row['verification']} | {row.get('target_status', '—')} |")
    text += ['', '## Git stages', '', '| Test | Apply | First status | Diff | Add | Cached check | Git commit | Final status | LayerFS Commit |', '|---|---:|---:|---:|---:|---:|---:|---:|---:|']
    for row in report['performance']:
        if row['family'] == 'git_tool_workflow':
            values = [milliseconds(row.get(key)) for key in ['apply_ns'] + git_report.GIT + ['commit_ns']]
            text.append('| ' + row['case'] + ' | ' + ' | '.join(values) + ' |')
    text += ['', 'All Git stage values are milliseconds. Git commit and LayerFS Commit are separate operations.']
    text += ['', '## Verification', '', '| Test | Result | Wall (s) |', '|---|---|---:|']
    text += [f"| {r['case']} | {r['status']} | {r.get('wall_seconds', '—')} |" for r in report['verification']]
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

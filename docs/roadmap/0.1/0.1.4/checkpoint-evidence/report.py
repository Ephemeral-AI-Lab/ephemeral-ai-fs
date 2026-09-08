#!/usr/bin/env python3
"""v0.1.4 historical comparison; reuse the receipt collector, never its older baseline."""
import argparse
import csv
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
FROZEN = HERE.parent / 'issue91-campaign'
BASE = HERE.parents[1] / '0.1.3/checkpoint-evidence'
spec = importlib.util.spec_from_file_location('checkpoint_v013', BASE / 'report.py')
legacy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(legacy)
# The historical generator's previous_results points to pre-v0.1.3 observations.
legacy.previous_results = lambda: {}


def key(row):
    return row['family'], row['case']


def index(rows, errors, label):
    result = {}
    for row in rows:
        ident = key(row)
        if ident in result:
            errors.append(f'duplicate {label}: {ident}')
        result[ident] = row
    return result


def numeric(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def metrics(row):
    """Individual counters with scope names; never sum overlapping observations."""
    values = {k: v for k, v in row.items() if numeric(v) and
              (k in ('elapsed_ns', 'host_peak_rss_bytes', 'container_peak_bytes', 'host_cpu_ns', 'container_cpu_ns')
               or k in ('create_ns', 'exec_ns', 'sdk_edit_ns', 'commit_ns', 'visibility_ns', 'end_ns'))}
    host = {r.get('phase'): r for r in row.get('host_resources', [])}
    for field in ('disk_read_bytes', 'disk_write_bytes'):
        before, after = host.get('before', {}).get(field), host.get('after-product', {}).get(field)
        if numeric(before) and numeric(after):
            values['host.after-product-minus-before.' + field] = after - before
    for group in row.get('observations', []):
        for endpoint in ('first', 'last'):
            record = group.get(endpoint, {})
            for field, value in record.items():
                if field.endswith('_bytes') and numeric(value):
                    values[f"observation.{group['group']}.{endpoint}.{record.get('phase')}.{field}"] = value
    counts = {}
    for record in row.get('route_metrics', []):
        scope = record.get('schema', record.get('kind', 'route'))
        for field, value in record.items():
            if numeric(value) and (field.endswith('_bytes') or field.endswith('_ns')) and not any(
                    token in field for token in ('clock', 'sample_interval', 'sample_gap', 'first_sample', 'last_sample',
                                                'host_t0', 'host_t3', 'rss_t0', 'rss_t3', 'window_start', 'window_end', 'observation_ready', 'observation_finish')):
                name = f'route.{scope}.{field}'
                counts[name] = counts.get(name, 0) + 1
                values[name] = value
    # Repeated route counters have no unambiguous aggregation contract.
    return {k: v for k, v in values.items() if counts.get(k, 1) == 1}


def severity(metric, baseline, candidate, plan):
    if baseline is None or candidate is None:
        return 'UNAVAILABLE'
    delta = candidate - baseline
    if delta <= 0:
        return 'NO_INCREASE'
    ratio = delta / baseline if baseline > 0 else None
    if metric.endswith('_ns'):
        if 'cpu' in metric:
            relative, absolute = 0.5, 50_000_000
        else:
            relative = plan['severe_time']['relative_increase']
            absolute = next(value for limit, value in plan['severe_time']['absolute_ns_by_baseline'] if limit is None or baseline < limit)
        if delta >= absolute and (ratio is None or ratio >= relative):
            return 'SEVERE'
        if delta >= plan['review']['absolute_ns'] and (ratio is None or ratio >= plan['review']['relative_increase']):
            return 'REVIEW'
    elif metric.endswith('_bytes'):
        if 'write' in metric:
            relative, absolute = 0.5, 16 * 2**20
        elif any(s in metric for s in ('rss', 'peak', 'memory', 'cgroup', 'resident', 'footprint')) and 'store' not in metric:
            relative, absolute = 0.5, 64 * 2**20
        elif any(s in metric for s in ('allocated', 'durable_store', 'database', 'live_page', 'file_bytes', 'store_growth')):
            relative, absolute = 0.1, 2**20
        else:
            return 'OBSERVED_INCREASE'
        if delta >= absolute and (ratio is None or ratio >= relative):
            return 'SEVERE'
        return 'REVIEW'
    return 'OBSERVED_INCREASE'


def contract_errors(old, current, mapping):
    errors = []
    for field in ('timer', 'seed', 'fixture_profile', 'fixture_bytes', 'fixture_files'):
        if old.get(field) != current.get(field):
            errors.append('incompatible ' + field)
    if current.get('timer') != mapping.get('timer'):
        errors.append('timer differs from mapping')
    if old.get('definition') != current.get('definition'):
        errors.append('registered workload/public route differs')
    if (old.get('preparation') or {}).get('fixture') != (current.get('preparation') or {}).get('fixture'):
        errors.append('prepared fixture content differs')
    # Include exact operation-source manifests; a source change needs an explicit
    # reviewed contract mapping, never a silent waiver based on matching names.
    fields = ('fixture_digest', 'edited_fixture_digest', 'edit_plan_sha256', 'replacement_sha256',
              'operation_key', 'operation_surface', 'payload_seed', 'logical_operation_count',
              'timed_call_graph_manifest_sha256', 'operation_route_manifest_sha256')
    for field in fields:
        previous = [r[field] for r in old.get('route_metrics', []) if field in r]
        actual = [r[field] for r in current.get('route_metrics', []) if field in r]
        if previous != actual:
            errors.append('incompatible content/operation ' + field)
    return errors


def compare(report, baseline, mapping, plan):
    errors = report['errors']
    historical = index(baseline, errors, 'historical case')
    current = index(report['performance'], errors, 'candidate case')
    mapped = index(mapping, errors, 'mapping case')
    comparisons = []
    for ident in sorted(historical.keys() | current.keys()):
        old, row, match = historical.get(ident), current.get(ident), mapped.get(ident)
        if old is not None and row is None and (match or {}).get('classification') == 'directly-comparable':
            errors.append(f'missing candidate case: {ident}')
        if match is None:
            errors.append(f'missing mapping: {ident}')
        eligible = bool(old and row and match and match['classification'] == 'directly-comparable')
        reasons = contract_errors(old, row, match) if eligible and row.get('sample_count') else []
        if row and row.get('sample_count') != plan['sample_count']:
            errors.append(f'missing/wrong sample count: {ident}')
            eligible = False
        if reasons:
            errors.extend(f'{ident}: {reason}' for reason in reasons)
            eligible = False
        if row:
            row.update(previous=None, previous_elapsed_ns=old.get('elapsed_ns') if eligible else None,
                       difference_ns=None, difference_percent=None,
                       comparison_eligibility='directly-comparable' if eligible else 'incompatible-contract' if reasons else 'missing-observation' if row.get('sample_count') != plan['sample_count'] else (match or {}).get('classification', 'unmapped'),
                       comparison_reason='; '.join(reasons) or (match or {}).get('reason'),
                       historical_checkpoint='9f5a641d223606c45e5e6aa8a20094c12f9139a1')
        old_metrics, new_metrics = metrics(old or {}), metrics(row or {})
        for metric in sorted(old_metrics.keys() | new_metrics.keys()):
            b, c = old_metrics.get(metric), new_metrics.get(metric)
            compatible = eligible
            reason = '; '.join(reasons)
            if metric == 'host_cpu_ns' and (old or {}).get('host_cpu_scope') != (row or {}).get('host_cpu_scope'):
                compatible, reason = False, 'host CPU scope differs'
            delta = c - b if compatible and b is not None and c is not None else None
            percent = 100 * delta / b if delta is not None and b > 0 else None
            state = severity(metric, b, c, plan) if compatible else 'INELIGIBLE'
            comparisons.append(dict(family=ident[0], case=ident[1], metric=metric,
                unit='ns' if metric.endswith('_ns') else 'bytes', eligibility='directly-comparable' if compatible else 'unavailable-or-incompatible',
                baseline_value=b, candidate_samples=[] if c is None else [c], actual_n=0 if c is None else (row or {}).get('sample_count', 0),
                aggregate=c, difference=delta, difference_percent=percent, severity=state,
                reason=reason or ('historical metric unavailable' if b is None else 'candidate metric unavailable' if c is None else 'zero baseline: percentage unavailable' if b == 0 else 'published single observation; not paired'),
                baseline_source=(old or {}).get('source_identity'), candidate_source=(row or {}).get('source_identity'),
                baseline_evidence=(old or {}).get('evidence'), candidate_evidence=(row or {}).get('evidence')))
            if row and metric == 'elapsed_ns':
                row.update(difference_ns=delta, difference_percent=percent)
    report.update(comparisons=comparisons, baseline_mapping=mapping,
                  unmatched_baseline=[r for ident, r in historical.items() if ident not in current],
                  severe_regressions=[r for r in comparisons if r['severity'] == 'SEVERE'])


def normalize_historical_seed(row, baseline_dir, declaration, manifest, errors):
    """Recover the omitted report seed from authenticated original sample evidence."""
    path = legacy.existing(baseline_dir / 'raw' / row['evidence'])
    relative = 'docs/roadmap/0.1/0.1.3/checkpoint-evidence/' + str(path.relative_to(baseline_dir))
    if not path.exists() or hashlib.sha256(path.read_bytes()).hexdigest() != manifest['sha256'].get(relative):
        errors.append(f'historical raw seed evidence hash mismatch: {key(row)}')
        return
    samples = [r for r in legacy.read(path) if r.get('kind') == 'sample']
    if len(samples) != 1:
        errors.append(f'historical raw seed sample cardinality: {key(row)}')
        return
    identity = samples[0].get('identities', {})
    seed = identity.get('seed')
    expected = declaration.get('seed_or_sdk_repetition')
    if (identity.get('family') != row['family'] or identity.get('case') != row['case']
            or seed != expected or seed is None
            or (row.get('definition', {}).get('route') == 'sdk' and identity.get('repetition') != expected)
            or identity != row.get('identities')):
        errors.append(f'historical raw seed identity mismatch: {key(row)}')
        return
    row['seed_provenance'] = {'original_report_seed': row.get('seed'), 'raw_sample_seed': seed,
                              'raw_sample_repetition': identity.get('repetition'),
                              'declaration_seed_or_sdk_repetition': expected,
                              'evidence': str(path.relative_to(baseline_dir)),
                              'sha256': manifest['sha256'][relative]}
    row['seed'] = seed


def validate_outcomes(row, errors):
    if row.get('sample_count') != 1:
        return  # Legacy derivation records missing receipts separately.
    for field in ('status', 'verification', 'resource_status', 'cleanup_status'):
        if row.get(field) != 'PASS':
            errors.append(f"failed or missing {field}: {key(row)}")
    for field in ('host_peak_rss_bytes', 'container_peak_bytes', 'container_cpu_ns'):
        value = row.get(field)
        if not numeric(value) or not math.isfinite(value) or value < 0:
            errors.append(f"missing or invalid required resource {field}: {key(row)}")
    if row.get('definition', {}).get('setup_policy') != 'fresh-output' and row.get('prepared_master_unchanged') is not True:
        errors.append(f"failed or missing custody: {key(row)}")


def derive(campaign, registry, frozen=FROZEN, baseline_dir=BASE):
    plan = legacy.read(frozen / 'declaration.json')
    report = legacy.derive(campaign, registry)
    manifest = legacy.read(frozen / 'baseline-manifest.json')
    for name in ('performance.csv', 'report.json', 'raw/declaration.json'):
        path = baseline_dir / name
        expected = manifest['sha256'][f'docs/roadmap/0.1/0.1.3/checkpoint-evidence/{name}']
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            report['errors'].append('historical baseline hash mismatch: ' + name)
    baseline = legacy.read(baseline_dir / 'report.json')['performance']
    historical_declaration = legacy.read(baseline_dir / 'raw/declaration.json')
    for row in baseline:
        normalize_historical_seed(row, baseline_dir, historical_declaration, manifest, report['errors'])
    report['historical_seed_provenance'] = [{'family': row['family'], 'case': row['case'],
                                           **row.get('seed_provenance', {})} for row in baseline]
    with (baseline_dir / 'performance.csv').open(newline='') as stream:
        csv_rows = index(list(csv.DictReader(stream)), report['errors'], 'baseline CSV')
    for row in baseline:
        csv_row = csv_rows.get(key(row), {})
        if str(row['elapsed_ns']) != csv_row.get('elapsed_ns'):
            report['errors'].append(f'historical JSON/CSV elapsed mismatch: {key(row)}')
    with (frozen / 'baseline-mapping.csv').open(newline='') as stream:
        mapping = list(csv.DictReader(stream))
    expected = legacy.read(frozen / 'registry.jsonl')
    if registry != expected:
        report['errors'].append('registry differs from prospective frozen registry')
    for field in ('performance_order', 'verification_order', 'seed', 'long_test_exclusion'):
        if report['declaration'].get(field) != plan[field]:
            report['errors'].append('generation differs from prospective declaration: ' + field)
    compare(report, baseline, mapping, plan)
    for row in report['performance']:
        validate_outcomes(row, report['errors'])
    for proof in report['verification']:
        if proof['status'] == 'EXCLUDED_LONG':
            proof['status'] = 'NOT_RUN_OPTIONAL'
            proof['reason'] = plan['long_test_reason']
    for family, summary in report['families'].items():
        statuses = {}
        for proof in report['verification']:
            if proof['family'] == family:
                statuses[proof['status']] = statuses.get(proof['status'], 0) + 1
        summary['verification_statuses'] = statuses
    report.update(schema='layerfs-v014-historical-checkpoint-v1', release='0.1.4',
                  baseline={k: manifest[k] for k in ('tag', 'tag_commit', 'accepted_checkpoint')},
                  baseline_release_limit='v0.1.3 release includes later FUSE correctness repair; complete historical performance campaign not rerun',
                  generation=str(campaign), fresh_diagnostics=[],
                  additional_requirements=[{'case': case, 'status': 'REQUIRES_SEPARATE_EVIDENCE'} for case in plan['additional_cases']],
                  generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
    report['status'] = 'INCOMPLETE' if report['errors'] else 'REVISE' if report['severe_regressions'] else 'ORDINARY_COMPLETE'
    report['qualification'] = 'REVISE: supplemental storage/additional cases, repair ledger and independent review must also qualify'
    return report


def write(report, output):
    legacy.write(report, output)
    # Retain legacy CSV fields but expose eligibility and descriptive raw n explicitly.
    path = output / 'performance.csv'
    with path.open(newline='') as stream:
        columns = next(csv.reader(stream))
    columns += ['comparison_eligibility', 'comparison_reason', 'historical_checkpoint']
    with path.open('w', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=columns, extrasaction='ignore')
        writer.writeheader()
        writer.writerows(report['performance'])
    with (output / 'comparison.csv').open('w', newline='') as stream:
        if report['comparisons']:
            writer = csv.DictWriter(stream, fieldnames=list(report['comparisons'][0]))
            writer.writeheader()
            writer.writerows(report['comparisons'])
    text = (output / 'report.md').read_text().replace('# v0.1.3 benchmark checkpoint', '# v0.1.4 benchmark checkpoint against published v0.1.3')
    text = text.replace('(raw/', '(' + os.path.relpath(report['generation'], output) + '/')
    text += '\n## Historical comparison and qualification\n\n' + report['qualification'] + '\n\n'
    text += report['baseline_release_limit'] + '. Baseline operands are elapsed_ns, never older comparison fields. Resource scopes are individual; no overlapping sums. [All metric operands and raw n](comparison.csv).\n\n'
    text += '| Family / case | Metric | v0.1.3 | v0.1.4 (n=1) | Change | Severity |\n|---|---|---:|---:|---:|---|\n'
    for row in report['comparisons']:
        if row['metric'] == 'elapsed_ns' or row['severity'] == 'SEVERE':
            percent = 'unavailable' if row['difference_percent'] is None else f"{row['difference_percent']:+.2f}%"
            text += f"| {row['family']} / {row['case']} | {row['metric']} | {row['baseline_value']} | {row['aggregate']} | {percent} | {row['severity']} |\n"
    (output / 'report.md').write_text(text)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('campaign', type=Path)
    parser.add_argument('--registry', type=Path, required=True)
    parser.add_argument('--frozen', type=Path, default=FROZEN)
    parser.add_argument('--baseline', type=Path, default=BASE)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = derive(args.campaign.resolve(), legacy.read(args.registry), args.frozen, args.baseline)
    write(result, args.output.resolve())
    raise SystemExit(0 if result['status'] == 'ORDINARY_COMPLETE' else 1)

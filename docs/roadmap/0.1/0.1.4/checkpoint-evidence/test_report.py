#!/usr/bin/env python3
"""Product-free checks: python3 test_report.py. No benchmark or Store access."""
import copy
import importlib.util
from pathlib import Path

spec = importlib.util.spec_from_file_location('v014_report', Path(__file__).with_name('report.py'))
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)
plan = r.legacy.read(r.FROZEN / 'declaration.json')


def check():
    old = dict(family='f', case='c', timer='pure_call_sum_ns', seed=1,
               fixture_profile='fixed', fixture_bytes=42, fixture_files=1,
               definition={'route': 'workspace'}, elapsed_ns=10_000_000,
               previous_elapsed_ns=1, source_identity='old', sample_count=1)
    new = dict(old, elapsed_ns=20_000_000, source_identity='new')
    mapping = [dict(family='f', case='c', classification='directly-comparable',
                    timer='pure_call_sum_ns', reason='frozen source contract')]
    report = dict(errors=[], performance=[new])
    r.compare(report, [old], mapping, plan)
    assert new['previous_elapsed_ns'] == 10_000_000
    assert new['difference_ns'] == 10_000_000 and new['difference_percent'] == 100
    assert report['severe_regressions'][0]['baseline_value'] == 10_000_000
    assert report['comparisons'][0]['actual_n'] == 1
    for field, value in [('timer', 'other'), ('fixture_bytes', 43), ('seed', 2)]:
        bad = dict(new, **{field: value})
        result = dict(errors=[], performance=[bad])
        r.compare(result, [old], mapping, plan)
        assert result['errors'] and bad['difference_percent'] is None
    result = dict(errors=[], performance=[dict(new), dict(new)])
    r.compare(result, [old], mapping, plan)
    assert any('duplicate candidate' in e for e in result['errors'])
    result = dict(errors=[], performance=[])
    r.compare(result, [old], mapping, plan)
    assert result['unmatched_baseline'] == [old]
    result = dict(errors=[], performance=[dict(new, sample_count=0)])
    r.compare(result, [old], mapping, plan)
    assert result['errors'] and not result['severe_regressions']
    result = dict(errors=[], performance=[dict(new)])
    r.compare(result, [old], [dict(mapping[0], classification='new-current')], plan)
    assert all(c['difference'] is None for c in result['comparisons'])
    assert r.severity('elapsed_ns', 10_000_000, 13_000_000, plan) == 'REVIEW'
    assert r.severity('elapsed_ns', 10_000_000, 15_000_000, plan) == 'SEVERE'
    assert r.severity('host_cpu_ns', 100_000_000, 150_000_000, plan) == 'SEVERE'
    assert r.severity('store.allocated_bytes', 10*2**20, 11*2**20, plan) == 'SEVERE'
    assert r.severity('elapsed_ns', None, 1, plan) == 'UNAVAILABLE'
    zero = dict(old, elapsed_ns=0)
    result = dict(errors=[], performance=[dict(new)])
    r.compare(result, [zero], mapping, plan)
    assert result['comparisons'][0]['difference_percent'] is None
    good = dict(new, status='PASS', verification='PASS', resource_status='PASS',
                cleanup_status='PASS', host_peak_rss_bytes=1, container_peak_bytes=1,
                container_cpu_ns=0, prepared_master_unchanged=True)
    errors = []
    r.validate_outcomes(good, errors)
    assert not errors
    for field in ('status', 'verification', 'resource_status', 'cleanup_status',
                  'host_peak_rss_bytes', 'container_peak_bytes', 'container_cpu_ns', 'prepared_master_unchanged'):
        errors = []
        r.validate_outcomes(dict(good, **{field: None}), errors)
        assert errors, field
    for invalid in (-1, float('nan'), float('inf')):
        errors = []
        r.validate_outcomes(dict(good, host_peak_rss_bytes=invalid), errors)
        assert errors
    values = r.metrics(dict(host_resources=[dict(phase='before', disk_write_bytes=20),
        dict(phase='after-product', disk_write_bytes=50), dict(phase='final', disk_write_bytes=90)],
        route_metrics=[dict(schema='x', spool_write_bytes=10), dict(schema='x', spool_write_bytes=20)]))
    assert values == {'host.after-product-minus-before.disk_write_bytes': 30}
    bad = copy.deepcopy(new)
    bad['route_metrics'] = [dict(fixture_digest='changed')]
    assert r.contract_errors(old, bad, mapping[0])
    historical = r.legacy.read(r.BASE / 'report.json')['performance'][0]
    assert historical['seed'] is None  # Actual published generator omission.
    errors = []
    manifest = r.legacy.read(r.FROZEN / 'baseline-manifest.json')
    declaration = r.legacy.read(r.BASE / 'raw/declaration.json')
    r.normalize_historical_seed(historical, r.BASE, declaration, manifest, errors)
    assert not errors and historical['seed'] == 1
    assert historical['seed_provenance']['original_report_seed'] is None
    current = copy.deepcopy(historical)
    current.pop('seed_provenance')
    assert not r.contract_errors(historical, current, {'timer': current['timer']})
    errors = []
    r.normalize_historical_seed(copy.deepcopy(historical), r.BASE,
                                dict(declaration, seed_or_sdk_repetition=2), manifest, errors)
    assert errors  # Never infer seed from candidate when original receipts disagree.
    assert r.contract_errors(old, dict(new, preparation={'fixture': {'input_plan_sha256': 'changed'}}), mapping[0])
    print('v0.1.4 report focused checks: PASS')


if __name__ == '__main__':
    check()

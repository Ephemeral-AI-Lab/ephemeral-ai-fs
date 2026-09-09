#!/usr/bin/env python3
"""Product-free checks: python3 test_report.py. No benchmark or Store access."""
import copy
import importlib.util
import json
import hashlib
import tempfile
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
    baseline_namespace = next(row for row in r.legacy.read(r.BASE / 'report.json')['performance']
                              if row['case'] == 'namespace-100-compact-v3')
    g0_path = r.legacy.existing(r.HERE / 'raw/generations/g0' / baseline_namespace['evidence'])
    if g0_path.exists():
        sample = next(row for row in r.legacy.read(g0_path) if row.get('kind') == 'sample')
        candidate_fixture = {'preparation': sample['preparation']}
        assert baseline_namespace['preparation']['fixture'] != sample['preparation']['fixture']
        assert r.fixture_contract(baseline_namespace) == r.fixture_contract(candidate_fixture)
        candidate_fixture = copy.deepcopy(candidate_fixture)
        candidate_fixture['preparation']['fixture']['fixture_digest'] = 'changed'
        assert r.fixture_contract(baseline_namespace) != r.fixture_contract(candidate_fixture)
    cold = copy.deepcopy(baseline_namespace)
    cold['preparation']['fixture']['fixture_cache_profile'] = 'cold'
    assert r.fixture_contract(baseline_namespace) != r.fixture_contract(cold)
    check_proof_preparation()
    print('v0.1.4 report focused checks: PASS')


def check_proof_preparation():
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        frozen, campaign = root / 'frozen', root / 'generation'
        frozen.mkdir()
        campaign.mkdir()
        def save(path, value):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(value))
            return hashlib.sha256(path.read_bytes()).hexdigest()
        declaration_sha = save(frozen / 'verification-preparation-r4.json', {'cases': ['c']})
        save(campaign / 'selections.json', {'proof_preparation_declaration_sha256': declaration_sha})
        identity = dict(family='f', case='c', seed=1, setup_identity='fresh-output',
                        source_identity='s', product_identity='p', input_identity='i',
                        harness_identity='h', image='image')
        raw_proof = {k: identity[k] for k in ('source_identity', 'product_identity', 'input_identity', 'harness_identity')}
        raw_proof.update(family='f', case='c', image_identity='image', wall_seconds=2,
                         status='PASS', cleanup={'status': 'PASS'})
        proof_path = campaign / 'verification/f/c/verification.json'
        proof_sha = save(proof_path, raw_proof)
        runner_path = campaign / 'preparation/f/c/runner.json'
        runner = dict(identities=identity, status='PASS', cleanup={'status': 'PASS'})
        runner_sha = save(runner_path, runner)
        prep = dict(identities=identity, declaration_sha256=declaration_sha,
                    runner_sha256=runner_sha, status='PASS', returncode=0, wall_ns=500_000_000)
        prep_path = runner_path.with_name('preparation.json')
        prep_sha = save(prep_path, prep)
        entry = dict(family='f', case='c', status='PASS', wall_seconds=2,
                     receipt=str(proof_path), receipt_sha256=proof_sha,
                     independent_preparation=dict(prep, receipt=str(prep_path), receipt_sha256=prep_sha),
                     preparation_plus_verification_wall_seconds=2.5)
        def derive_entry(value):
            save(campaign / 'verification-ledger.json', [value])
            report = dict(errors=[], raw_sha256={}, declaration={'seed': 1}, verification=[copy.deepcopy(raw_proof)])
            r.proof_preparations(report, campaign, frozen)
            return report
        report = derive_entry(entry)
        assert not report['errors'], report['errors']
        assert report['verification'][0]['preparation_plus_verification_wall_seconds'] == 2.5
        assert report['verification'][0]['verification_timing_comparison'].startswith('INELIGIBLE')
        for field, value in [('preparation_plus_verification_wall_seconds', 2), ('receipt_sha256', 'wrong')]:
            assert derive_entry(dict(entry, **{field: value}))['errors']
        for field, value in [('receipt', str(root / 'outside.json')), ('receipt_sha256', 'wrong'), ('status', 'FAIL')]:
            bad = copy.deepcopy(entry)
            bad['independent_preparation'][field] = value
            assert derive_entry(bad)['errors']
        bad = copy.deepcopy(entry)
        bad.pop('independent_preparation')
        assert derive_entry(bad)['errors']
        runner['identities'] = dict(identity, source_identity='other')
        prep['runner_sha256'] = save(runner_path, runner)
        prep_sha = save(prep_path, prep)
        bad = dict(entry, independent_preparation=dict(prep, receipt=str(prep_path), receipt_sha256=prep_sha))
        assert derive_entry(bad)['errors']


if __name__ == '__main__':
    check()

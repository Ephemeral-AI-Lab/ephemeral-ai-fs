#!/usr/bin/env python3
"""Validate proposed aggregate receipts. No Store access or product execution."""
import copy
import json
import re
import sys

TERMINALS = (
    'no_predecessor', 'missing_required_span', 'no_overlap',
    'correspondence_limit', 'unavailable_base', 'search_budget',
    'no_useful_raw_delta', 'compressed_mixed_rejection', 'delta_admitted',
    'admission_race',
)
LIMITS = {'memory_limit', 'file_limit', 'operation_limit', 'descriptor_limit'}
ZERO = (0, 0)
EVENT_COUNTS = {'candidate_trials', 'predecessor_hints', 'usable_bases', 'base_fetches',
                'budget_skips', 'correspondence_stop_events', 'fetch_budget_events',
                'match_budget_events', 'trial_budget_events', 'instruction_budget_events',
                'memory_budget_events', 'rejected_mixed_groups', 'duplicate_observation_conflicts'}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def integer(value, name):
    require(type(value) is int and value >= 0, name + ': nonnegative integer required')
    return value


def pair(row, name):
    n = integer(row['count'], name + '.count')
    b = integer(row['canonical_bytes'], name + '.canonical_bytes')
    require((n == 0) == (b == 0), name + ': zero count/byte disagreement')
    return n, b


def add(a, b):
    return a[0] + b[0], a[1] + b[1]


def total(values):
    result = ZERO
    for value in values:
        result = add(result, value)
    return result


def flags(row, names):
    require(all(type(row[k]) is bool for k in names), 'histogram flags must be booleans')


def validate(receipt, expected_identity):
    require(set(receipt) == {'schema', 'identity', 'checkpoint', 'phase', 'complete',
            'eligible_attempts', 'terminals', 'event_counts', 'occurrence_routes',
            'eligible_shape_occurrences', 'cursor_histogram', 'search_histogram',
            'late_recheck_existing_count', 'new_location_provenance', 'previous_selected',
            'ack_selected', 'retained_prefix', 'correspondence_grant_totals'}, 'unexpected/missing diagnostic field')
    require(receipt['schema'] == 'issue87-coverage-diagnostic-v1', 'unsupported schema')
    require(receipt['identity'] == expected_identity, 'identity differs from frozen contract')
    for key in ('source_seal', 'product_seal', 'workload_manifest_sha256'):
        require(re.fullmatch('[0-9a-f]{64}', expected_identity[key]) is not None,
                key + ': SHA256 required')
    i = integer(receipt['checkpoint'], 'checkpoint')
    require(i <= 157, 'checkpoint outside frozen157 workload')
    require(receipt['phase'] == ('init' if i == 0 else 'commit'), 'wrong checkpoint phase')
    require(receipt['complete'] is True, 'incomplete operation; preserve partial evidence')
    eligible = pair(receipt['eligible_attempts'], 'eligible_attempts')
    terminal = {k: pair(receipt['terminals'][k], 'terminals.' + k) for k in TERMINALS}
    require(set(receipt['terminals']) == set(TERMINALS), 'unexpected terminal reason')
    require(total(terminal.values()) == eligible, 'terminal count/byte conservation failed')
    require(set(receipt['event_counts']) <= EVENT_COUNTS, 'unknown/dynamic event counter name')
    for name, count in receipt['event_counts'].items():
        integer(count, 'event_counts.' + name)  # Overlapping events are never target totals.

    # Occurrence work is a separate population, including repeated/reused payloads.
    routes = receipt['occurrence_routes']
    require(set(routes) == {'existing', 'duplicate', 'initially_missing'}, 'occurrence routes')
    require(total(pair(v, 'occurrence route') for v in routes.values()) ==
            pair(receipt['eligible_shape_occurrences'], 'occurrences'), 'occurrence conservation')
    require(pair(routes['initially_missing'], 'initially_missing') == eligible,
            'initial CAS route differs from eligible attempts')

    granted = reserved = 0
    for name, route in routes.items():
        grants = integer(route['grants_count'], name + '.grants_count')
        cost = integer(route['reserved_bytes'], name + '.reserved_bytes')
        require(cost == grants * 131136, 'reservation/grant size disagreement')
        require(route['count'] > 0 or grants == 0, 'grant credit without an occurrence')
        granted += grants
        reserved += cost
    observed = receipt['correspondence_grant_totals']
    require(granted == integer(observed['grants_count'], 'observed grants') and
            reserved == integer(observed['reserved_bytes'], 'observed reservations'),
            'occurrence grant credits do not conserve observed reservations')
    require(reserved <= 16 * 1024 * 1024, 'operation correspondence reservation limit exceeded')

    # Histogram cells are fixed aggregates, never per-path/ObjectId logs.
    derived = dict.fromkeys(TERMINALS, ZERO)
    cursor_total = hinted = ZERO
    seen = set()
    for row in receipt['cursor_histogram']:
        require(set(row) == {'has_predecessor', 'span_present', 'inherited_exhaustion',
                            'hint_count', 'stop', 'count', 'canonical_bytes'}, 'cursor cell fields')
        flags(row, ('has_predecessor', 'span_present', 'inherited_exhaustion'))
        hints = integer(row['hint_count'], 'hint_count')
        require(hints <= 4, 'hint count exceeds existing four-ID bound')
        stop = row['stop']
        require(stop in LIMITS | {'not_applicable', 'complete'}, 'unknown cursor stop')
        key = (row['has_predecessor'], row['span_present'], stop, hints, row['inherited_exhaustion'])
        require(key not in seen, 'duplicate cursor histogram cell')
        seen.add(key)
        weight = pair(row, 'cursor cell')
        cursor_total = add(cursor_total, weight)
        require(not row['inherited_exhaustion'] or stop in LIMITS, 'inherited stop without limit')
        if not row['has_predecessor'] or not row['span_present']:
            require(hints == 0 and stop == 'not_applicable', 'hint/cursor work without predecessor+span')
            reason = 'no_predecessor' if not row['has_predecessor'] else 'missing_required_span'
        else:
            require(stop != 'not_applicable', 'required correspondence outcome missing')
            if hints:
                hinted = add(hinted, weight)
                continue
            reason = 'no_overlap' if stop == 'complete' else 'correspondence_limit'
        derived[reason] = add(derived[reason], weight)
    require(cursor_total == eligible, 'cursor histogram does not partition eligible attempts')

    searched = ZERO
    seen = set()
    for row in receipt['search_histogram']:
        require(set(row) == {'usable_base', 'search_complete', 'raw_candidate_complete',
                            'mixed', 'count', 'canonical_bytes'}, 'search cell fields')
        flags(row, ('usable_base', 'search_complete', 'raw_candidate_complete'))
        key = (row['usable_base'], row['search_complete'], row['raw_candidate_complete'], row['mixed'])
        require(key not in seen, 'duplicate search histogram cell')
        seen.add(key)
        weight = pair(row, 'search cell')
        searched = add(searched, weight)
        if row['raw_candidate_complete']:
            require(row['usable_base'] and row['mixed'] in ('selected', 'rejected'),
                    'complete candidate requires usable base and completed group comparison')
            reason = 'delta_admitted' if row['mixed'] == 'selected' else 'compressed_mixed_rejection'
        else:
            require(row['mixed'] == 'not_run', 'mixed result without complete candidate')
            reason = ('search_budget' if not row['search_complete'] else
                      'no_useful_raw_delta' if row['usable_base'] else 'unavailable_base')
        derived[reason] = add(derived[reason], weight)
    require(searched == hinted, 'search histogram does not partition hinted attempts')

    # Fail closed on races before equating prepared reasons with unique winners.
    require(terminal['admission_race'] == ZERO and integer(receipt['late_recheck_existing_count'], 'late_recheck_existing_count') == 0,
            'admission race: attempt evidence retained; unique-cohort table unavailable')
    require(derived == terminal, 'terminal reason disagrees with complete/incomplete causal state')
    require(i != 0 or terminal['no_predecessor'] == eligible,
            'fresh Init cannot have predecessor/search outcomes')
    loc = receipt['new_location_provenance']
    require(integer(loc['checkpoint'], 'location checkpoint') == i and loc['status'] == 'authenticated', 'location custody unavailable')
    require(loc['source'] == 'successful_transaction_pack_ranges_plus_final_authenticated_locator_join', 'locations lack admission transaction provenance')
    require(re.fullmatch('[0-9a-f]{64}', loc['manifest_sha256']) is not None, 'location manifest hash')
    require(integer(loc['duplicate_object_count'], 'duplicate_object_count') == 0 and
            integer(loc['duplicate_locator_count'], 'duplicate_locator_count') == 0,
            'duplicate new selected identity/location')
    full = pair(loc['eligible_FULL'], 'eligible_FULL')
    delta = pair(loc['eligible_DELTA'], 'eligible_DELTA')
    require(add(full, delta) == eligible, 'new selected/attempt cohort bijection failed')
    require(delta == terminal['delta_admitted'], 'DELTA winner count/byte mismatch')
    before = pair(receipt['previous_selected'], 'previous_selected')
    after = pair(receipt['ack_selected'], 'ack_selected')
    require(add(before, pair(loc['all_objects'], 'all new objects')) == after,
            'new location/acknowledgement count or byte mismatch')
    require(pair(receipt['retained_prefix'], 'retained_prefix') == after,
            'retained-prefix/acknowledged set equality unavailable')
    require(all(after[j] >= eligible[j] and pair(loc['all_objects'], 'all new objects')[j] >= eligible[j]
                for j in (0, 1)), 'eligible cohort exceeds selected objects')
    findings = []
    if terminal['missing_required_span'][0]:
        findings.append({'kind': 'handoff_defect', 'count': terminal['missing_required_span'][0],
                         'canonical_bytes': terminal['missing_required_span'][1]})
    return {'status': 'PASS', 'unique_cohort_status': 'validated_under_reviewed_source_premises',
            'checkpoint': i, 'eligible_count': eligible[0], 'eligible_canonical_bytes': eligible[1],
            'findings': findings, 'scope': 'canonical target bytes; no compressed-savings claim',
            'provenance_authentication': 'required upstream; this validator checks metadata only'}


def validate_run(receipts, expected_identity):
    require(len(receipts) == 158, 'full run requires Init plus all157 acknowledgements')
    previous = ZERO
    results = []
    for i, receipt in enumerate(receipts):
        require(receipt['checkpoint'] == i, 'missing, duplicate or reordered checkpoint')
        require(pair(receipt['previous_selected'], 'previous_selected') == previous,
                'acknowledgement chain does not join')
        results.append(validate(receipt, expected_identity))
        previous = pair(receipt['ack_selected'], 'ack_selected')
    return {'status': 'PASS', 'validated_checkpoints': 158, 'receipts': results}


def self_test():
    def w(n, b):
        return {'count': n, 'canonical_bytes': b}
    identity = dict.fromkeys(('source_seal', 'product_seal', 'workload_manifest_sha256'), 'a' * 64)
    r = {'schema': 'issue87-coverage-diagnostic-v1', 'identity': identity, 'checkpoint': 1,
         'phase': 'commit', 'complete': True, 'eligible_attempts': w(3, 900),
         'eligible_shape_occurrences': w(5, 1500),
         'occurrence_routes': {'existing': w(1, 300), 'duplicate': w(1, 300), 'initially_missing': w(3, 900)},
         'terminals': {k: w(0, 0) for k in TERMINALS}, 'event_counts': {'candidate_trials': 11, 'budget_skips': 7},
         'late_recheck_existing_count': 0,
         'cursor_histogram': [dict(has_predecessor=True, span_present=False, stop='not_applicable',
                                   hint_count=0, inherited_exhaustion=False, **w(1, 100)),
                              dict(has_predecessor=True, span_present=True, stop='complete',
                                   hint_count=0, inherited_exhaustion=False, **w(1, 300)),
                              dict(has_predecessor=True, span_present=True, stop='file_limit',
                                   hint_count=1, inherited_exhaustion=False, **w(1, 500))],
         'search_histogram': [dict(usable_base=True, search_complete=False, raw_candidate_complete=True,
                                   mixed='selected', **w(1, 500))],
         'previous_selected': w(2, 80), 'ack_selected': w(6, 1000), 'retained_prefix': w(6, 1000),
         'new_location_provenance': {'checkpoint': 1, 'status': 'authenticated',
                                     'source': 'successful_transaction_pack_ranges_plus_final_authenticated_locator_join', 'manifest_sha256': 'b' * 64,
                                     'duplicate_object_count': 0, 'duplicate_locator_count': 0,
                                     'all_objects': w(4, 920), 'eligible_FULL': w(2, 400),
                                     'eligible_DELTA': w(1, 500)}}
    r['correspondence_grant_totals'] = {'grants_count': 6, 'reserved_bytes': 6 * 131136}
    for name, grants in [('existing', 2), ('duplicate', 1), ('initially_missing', 3)]:
        r['occurrence_routes'][name].update(grants_count=grants, reserved_bytes=grants * 131136)
    for k, value in [('missing_required_span', w(1, 100)), ('no_overlap', w(1, 300)), ('delta_admitted', w(1, 500))]:
        r['terminals'][k] = value
    assert validate(r, identity)['findings'] == [{'kind': 'handoff_defect', 'count': 1, 'canonical_bytes': 100}]
    # Partial correspondence/search can still win; overlapping event counts may exceed targets.
    failures = [
        lambda x: x['identity'].update(source_seal='c' * 64),
        lambda x: x['terminals']['no_overlap'].update(canonical_bytes=301),
        lambda x: x['cursor_histogram'][1].update(stop='operation_limit'),
        lambda x: x['cursor_histogram'][0].update(span_present=True),
        lambda x: x['search_histogram'][0].update(raw_candidate_complete=False),
        lambda x: x.update(late_recheck_existing_count=1),
        lambda x: x['new_location_provenance']['eligible_FULL'].update(canonical_bytes=401),
        lambda x: x['retained_prefix'].update(count=5),
        lambda x: x['new_location_provenance'].update(duplicate_object_count=1),
        lambda x: x['cursor_histogram'].append(copy.deepcopy(x['cursor_histogram'][0])),
        lambda x: x['eligible_attempts'].update(count=True),
        lambda x: x['cursor_histogram'][0].update(object_id='forbidden timed identifier'),
        lambda x: x['event_counts'].update(per_object_dynamic_counter=1),
        lambda x: x['occurrence_routes']['duplicate'].update(grants_count=2, reserved_bytes=2 * 131136),
        lambda x: (x['occurrence_routes']['initially_missing'].update(grants_count=126, reserved_bytes=126 * 131136),
                   x['correspondence_grant_totals'].update(grants_count=129, reserved_bytes=129 * 131136)),
    ]
    for mutate in failures:
        bad = copy.deepcopy(r)
        mutate(bad)
        try:
            validate(bad, identity)
        except ValueError:
            pass
        else:
            raise AssertionError('invalid diagnostic accepted')
    run = []
    previous = (0, 0)
    for i in range(158):
        row = copy.deepcopy(r)
        row.update(checkpoint=i, phase='init' if i == 0 else 'commit')
        row['new_location_provenance']['checkpoint'] = i
        if i == 0:
            row['cursor_histogram'] = [dict(has_predecessor=False, span_present=False,
                stop='not_applicable', hint_count=0, inherited_exhaustion=False, **w(3, 900))]
            row['search_histogram'] = []
            row['event_counts'] = {}
            row['correspondence_grant_totals'] = {'grants_count': 0, 'reserved_bytes': 0}
            for route in row['occurrence_routes'].values():
                route.update(grants_count=0, reserved_bytes=0)
            row['terminals'] = {k: w(0, 0) for k in TERMINALS}
            row['terminals']['no_predecessor'] = w(3, 900)
            row['new_location_provenance'].update(eligible_FULL=w(3, 900), eligible_DELTA=w(0, 0))
        row['previous_selected'] = w(*previous)
        previous = add(previous, (4, 920))
        row['ack_selected'] = row['retained_prefix'] = w(*previous)
        run.append(row)
    assert validate_run(run, identity)['validated_checkpoints'] == 158
    for broken in [run[:-1], run[:1] + run[:1] + run[2:]]:
        try:
            validate_run(broken, identity)
        except ValueError:
            pass
        else:
            raise AssertionError('invalid checkpoint collection accepted')
    print('PASS: valid partial-success/handoff and158-checkpoint chain; 17 malformed, race, custody and conservation cases rejected')


if __name__ == '__main__':
    if sys.argv[1:] == ['--self-test']:
        self_test()
    elif len(sys.argv) == 3:
        try:
            with open(sys.argv[1]) as f:
                receipt = json.load(f)
            with open(sys.argv[2]) as f:
                expected = json.load(f)
            print(json.dumps(validate_run(receipt, expected) if isinstance(receipt, list) else
                             validate(receipt, expected), indent=2))
        except (ValueError, KeyError, TypeError) as error:
            print(json.dumps({'status': 'FAIL', 'reason': str(error), 'unique_cohort_status': 'unavailable'}))
            sys.exit(1)
    else:
        sys.exit('usage: validate_diagnostic.py RECEIPT.json EXPECTED_IDENTITY.json | --self-test')

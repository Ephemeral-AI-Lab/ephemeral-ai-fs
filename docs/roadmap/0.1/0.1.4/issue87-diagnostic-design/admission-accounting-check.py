#!/usr/bin/env python3
"""Synthetic accounting invariants; no product execution, Store, or workload input.

Each dict models an already resolved diagnostic occurrence, not a new storage API.
"""
import json


def totals(rows):
    buckets = {}
    for row in rows:
        bucket = buckets.setdefault(row['route'], {'occurrences_count': 0,
                                                  'occurrence_canonical_bytes': 0,
                                                  'reservation_grants_count': 0})
        bucket['occurrences_count'] += 1
        bucket['occurrence_canonical_bytes'] += row['canonical_bytes']
        bucket['reservation_grants_count'] += row['grants']
    return buckets


def valid_unique(attempts, selected):
    return (len(attempts), sum(a['canonical_bytes'] for a in attempts)) == (
        len(selected), sum(a['canonical_bytes'] for a in selected))


def main():
    occurrences = [
        {'route': 'fresh_probe_preexisting', 'canonical_bytes': 100, 'grants': 2},
        {'route': 'duplicate_occurrence', 'canonical_bytes': 100, 'grants': 0},
        {'route': 'fresh_probe_missing', 'canonical_bytes': 200, 'grants': 0},
    ]
    work = totals(occurrences)
    assert sum(v['occurrences_count'] for v in work.values()) == 3
    assert sum(v['occurrence_canonical_bytes'] for v in work.values()) == 400
    assert sum(v['reservation_grants_count'] for v in work.values()) == 2
    assert work['fresh_probe_missing']['occurrence_canonical_bytes'] == 200
    # One cursor's metadata initialization is shared. Skipping its reused first
    # target only moves the two reads to its later missing target; none disappears.
    reads_original = 2
    reads_if_reused_first_target_skipped = 2
    triggered_by_reused = work['fresh_probe_preexisting']['reservation_grants_count']
    assert triggered_by_reused == 2
    assert reads_original - reads_if_reused_first_target_skipped == 0
    # Two owners can prepare the same absent object. Attempts conserve, but their
    # totals must not masquerade as a unique selected cohort.
    attempts = [
        {'terminal': 'delta_admitted', 'canonical_bytes': 200},
        {'terminal': 'admission_race', 'canonical_bytes': 200},
    ]
    selected = [attempts[0]]
    assert not valid_unique(attempts, selected)
    assert valid_unique(selected, selected)
    # Mixed group A and B have one membership. Partial admission still retains
    # the whole winning pack; winner-count proportional attribution is invalid.
    a_encoded_bytes, b_encoded_bytes = 1000, 800
    prepared_records, selected_records = 2, 1
    persisted_group_bytes = b_encoded_bytes
    assert persisted_group_bytes != b_encoded_bytes * selected_records // prepared_records
    print(json.dumps({
        'status': 'PASS',
        'scope': 'synthetic diagnostic algebra; no measured workload inference',
        'occurrence_work_buckets': work,
        'reservation_bytes_per_grant': 131136,
        'reused_triggered_grants_count': triggered_by_reused,
        'avoidable_grants_count_in_shared_cursor_witness': 0,
        'race_unique_cohort_gate': 'reject',
        'mixed_group_A_encoded_bytes': a_encoded_bytes,
        'mixed_group_B_encoded_bytes': b_encoded_bytes,
        'partial_admission_persisted_group_bytes': persisted_group_bytes,
        'field_contract': 'integer counts/bytes; exclusive occurrence routes and distinct attempt/selected populations; derived synthetic witnesses, not production measurements'
    }, indent=2))


if __name__ == '__main__':
    main()

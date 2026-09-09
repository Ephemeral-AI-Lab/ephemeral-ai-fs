"""Reproduce the v2 qualification table from sealed per-case receipts."""
import hashlib
import json
import statistics
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
fixture = json.loads((Path(__file__).resolve().parents[5] / 'benchmark/fs-bench-pro/families/historical_access/fixture.json').read_text())
rows = []
for case in fixture['cases']:
    pair = []
    for mode in ('performance', 'verification'):
        folder = root / (case['id'] + '-' + mode + '-1')
        manifest = json.loads((folder / 'manifest.json').read_text())
        for name, digest in manifest.items():
            assert Path(name).name == name
            assert hashlib.sha256((folder / name).read_bytes()).hexdigest() == digest
        result = json.loads((folder / 'result.json').read_text())
        external = json.loads((folder / 'external-command-wall.json').read_text())
        assert result['status'] == result['cleanup_status'] == result['correctness_status'] == result['resource_status'] == 'PASS'
        assert result['case'] == case and result['mode'] == mode and not result['admission_eligible']
        assert external['returncode'] == 0 and external['elapsed_ns'] < 15_000_000_000
        pair.append((result, external['elapsed_ns']))
    perf, proof = pair[0][0], pair[1][0]
    assert perf['custody'] == proof['custody']
    assert proof['performance_sha256'] == hashlib.sha256((root / (case['id'] + '-performance-1') / 'result.json').read_bytes()).hexdigest()
    phase = next(p for p in perf['records'] if p.get('phase') == 'access-measured')
    physical = phase['physical_storage']
    rows.append(dict(case=case['id'], performance_ns=pair[0][1], verification_ns=pair[1][1],
        operation_ns=int(perf['observed'][-1]['access_operation_ns']), returned_bytes=case['length'],
        encoded_read_bytes=physical['encoded_read_bytes'], decoded_read_bytes=physical['decoded_read_bytes'],
        native_raw_decoded_bytes=physical['native_raw_decoded_bytes'], group_fetches=physical['group_fetches'],
        base_fetches=physical['base_fetches']))
assert len(rows) == 11 and len(list(root.glob('*/result.json'))) == 22
print('| Case | Complete performance (s) | Complete proof (s) | POSIX operation (ms) | Encoded read bytes | Decoded bytes | Native raw decoded bytes |')
print('| --- | ---: | ---: | ---: | ---: | ---: | ---: |')
for r in rows:
    print(f"| {r['case']} | {r['performance_ns']/1e9:.3f} | {r['verification_ns']/1e9:.3f} | {r['operation_ns']/1e6:.3f} | {r['encoded_read_bytes']:,} | {r['decoded_read_bytes']:,} | {r['native_raw_decoded_bytes']:,} |")
print('\nComplete command wall includes interpreter startup and final receipt writes, measured by the external collector.')
for mode in ('performance','verification'):
    values = [r[mode + '_ns'] for r in rows]
    print(f"{mode}: n=11 different cases, range {min(values)/1e9:.3f}–{max(values)/1e9:.3f}s; median across these cases {statistics.median(values)/1e9:.3f}s. Each case has n=1 (median=min=max for that case).")
print('\nCustody: `' + json.dumps(perf['custody'],sort_keys=True) + '`')

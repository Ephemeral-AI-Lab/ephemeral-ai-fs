#!/usr/bin/env python3
"""Install an derived binding without modifying existing campaign receipts."""
import fcntl
import hashlib
import json
import os
from pathlib import Path

HERE = Path(__file__).resolve().parent

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def check_identity(row, binding):
    identity = row['identities']
    assert identity['source_identity'] == binding['host']['LAYERFS_SOURCE_SEAL'], row['case']
    assert identity['product_identity'] == binding['host']['LAYERFS_PRODUCT_SEAL'], row['case']
    assert identity['host_executor'] == binding['host'], row['case']
    assert identity['image'] == binding['image'], row['case']
    assert identity['harness_identity'] == binding['harness_identity'], row['case']
    assert row['seed'] == binding['seed'], row['case']


def main():
    binding_path = HERE/'declaration.json'
    binding = json.loads(binding_path.read_text())
    campaign = Path(binding['raw_generation'])
    with (Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        for reference in binding['references'].values():
            assert sha(Path(reference['path'])) == reference['sha256'], reference
        assert sha(campaign/'registry.jsonl') == binding['references']['registry']['sha256']
        performance = json.loads((campaign/'performance-ledger.json').read_text())
        verification = json.loads((campaign/'verification-ledger.json').read_text())
        assert [r['case'] for r in performance] == binding['performance_order']
        assert [r['case'] for r in verification] == binding['verification_order']
        for row in performance:
            check_identity(row, binding)
        # Absolute receipt paths already point inside the original generation.
        # R26 needed six rebases only because its report used a relocated projection.
        for row in verification:
            prep = row.get('independent_preparation')
            if prep:
                expected = campaign/'preparation'/row['family']/row['case']/'preparation.json'
                assert Path(prep['receipt']).resolve() == expected.resolve()
                assert sha(expected) == prep['receipt_sha256']
                expected = campaign/'verification'/row['family']/row['case']/'verification.json'
                assert Path(row['receipt']).resolve() == expected.resolve()
                assert sha(expected) == row['receipt_sha256']
        target = campaign/'declaration.json'
        with target.open('xb') as stream:
            stream.write(binding_path.read_bytes())
        receipt = {'schema':'issue95-prospective-binding-installation-v1',
                   'source':str(binding_path), 'destination':str(target), 'sha256':sha(target),
                   'performance_rows':len(performance), 'verification_rows':len(verification),
                   'scope':'Installed explicitly post-collection derived declaration byte-identically; all existing raw files and ledgers unchanged; no derived projection needed.'}
        with (HERE/'installation.json').open('x') as stream:
            json.dump(receipt, stream, indent=2)
            stream.write('\n')
        print(json.dumps(receipt))

if __name__ == '__main__':
    main()

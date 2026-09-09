"""Bind the executed157 oracle comparisons to the saved original fixture seals."""
import hashlib, json
from pathlib import Path

root = Path(__file__).resolve().parent
identity = Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/identity.json')
original = json.loads(identity.read_text())['fixtures']['deepseek-full']['states']
result = json.loads((root/'result.json').read_text())
assert result['status'] == 'PASS' and len(original) == len(result['records']) == 157
for expected, actual in zip(original, result['records']):
    assert expected['index'] == actual['index']
    assert expected['sha'] == actual['source_sha']
    assert expected['logical_bytes'] == actual['logical_bytes']
    assert expected['oracle_sha256'] == actual['oracle_sha256']
    with Path(expected['oracle']).open('rb') as f:
        assert hashlib.file_digest(f, 'sha256').hexdigest() == expected['oracle_sha256']
with identity.open('rb') as f:
    identity_hash = hashlib.file_digest(f,'sha256').hexdigest()
output = dict(status='PASS',states=157,original_identity_path=str(identity),original_identity_sha256=identity_hash,executed_verification_sha256=hashlib.sha256((root/'result.json').read_bytes()).hexdigest(),scope='Every oracle SHA used by the completed comparison equals its original saved fixture identity and current bytes; no candidate replay')
(root/'oracle-custody.json').write_text(json.dumps(output,indent=2)+'\n')
print('PASS: all157 executed oracle hashes match original saved fixture seals')

"""Read existing evidence only: audit.py root OLD_VERIFICATION_JSON | proof NEW_VERIFICATION_JSON."""
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import sys

mode, filename = sys.argv[1:]
proof_path = Path(filename).resolve()
proof = json.loads(proof_path.read_text())
prepared = Path(proof['setup_observation']['prepared_root'])

def sha(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

def receipt(record):
    return dict(line.split('=', 1) for line in record['receipt'].splitlines() if '=' in line)

if mode == 'proof':
    records = proof['checks']
    required = ['git-precommit-custody', 'canonical-verification', 'git-reopen-custody', 'git-semantic-verification']
    found = {kind: [r for r in records if r.get('kind') == kind] for kind in required}
    assert proof['status'] == 'PASS' and proof['cleanup']['status'] == 'PASS'
    assert all(len(items) == 1 for items in found.values()), {k: len(v) for k,v in found.items()}
    assert not any(r.get('kind') == 'sampled-native-verification' for r in records)
    before = receipt(found['git-precommit-custody'][0])
    after = receipt(found['git-reopen-custody'][0])
    assert before['repository_manifest_sha256'] == after['repository_manifest_sha256']
    semantics = receipt(found['git-semantic-verification'][0])
    assert semantics['git_semantic_verification_status'] == 'pass'
    expected = {name: (prepared/'reference/input'/('expected-'+name)).read_text().strip() for name in ('head','tree','parent')}
    assert all(semantics['git_'+name] == value for name,value in expected.items())
    print(json.dumps({'proof':str(proof_path),'sha256':sha(proof_path),'status':'PASS',
        'required_markers':required,'expected_git':expected,'observed_git':{k:semantics['git_'+k] for k in expected},
        'precommit_reopened_repository_manifest_sha256':before['repository_manifest_sha256'],
        'cleanup':proof['cleanup']},indent=2))
    raise SystemExit(0)

assert mode == 'root'
store = prepared/'store.sqlite'
connection = sqlite3.connect(store.as_uri()+'?mode=ro', uri=True)

def object_value(identity):
    row = connection.execute('SELECT bytes FROM objects WHERE object_id=?',(identity,)).fetchone()
    assert row is not None
    data = row[0]
    assert data[:5] == b'LFSO\x01'
    assert int.from_bytes(data[5:9],'big') == len(data)-9
    assert int.from_bytes(data[9:13],'big') == len(data)-13
    return data[13:]

root_id = connection.execute('SELECT root_id FROM layers LIMIT 1').fetchone()[0]
namespace = object_value(root_id)
assert namespace[:8] == b'LFS4FSR\0' and len(namespace) == 108
root_inode = namespace[44:76]

def lookup(table):
    data = object_value(table)
    assert data[:8] == b'LFS4INT\0' and (len(data)-31)%64 == 0
    for at in range(31,len(data),64):
        inode, value = data[at:at+32], data[at+32:at+64]
        if data[10] == 7:
            if inode == root_inode:
                return value
        else:
            assert data[10] == 8
            found = lookup(value)
            if found is not None:
                return found

record_id = lookup(namespace[76:108])
record = object_value(record_id)
assert record[:8] == b'LFS4INO\0' and record[12] == 2 and len(record) == 85
metadata_id = record[53:85]
metadata = object_value(metadata_id)
assert metadata[:8] == b'LFS4MET\0' and metadata[10:12] == bytes([9,0])
position, values = 31, {}
while position < len(metadata):
    keys = []
    for _ in range(2):
        size = int.from_bytes(metadata[position:position+2],'big'); position += 2
        keys.append(metadata[position:position+size]); position += size
    assert keys[0] == b'portable' and metadata[position] == 1
    position += 1
    file_state = object_value(metadata[position:position+32]); position += 32
    assert file_state[:8] == b'LFS4MAP\0' and file_state[10] == 10 and file_state[28] == 0
    mapping = object_value(file_state[61:93])
    assert mapping[:8] == b'LFS4MAP\0' and mapping[10:12] == bytes([8,0]) and len(mapping) == 71
    chunk = object_value(mapping[31:63]); assert chunk[:8] == b'LFS4CHK\0'
    offset, length = int.from_bytes(mapping[63:67],'big'), int.from_bytes(mapping[67:71],'big')
    value = chunk[8+offset:8+offset+length]
    assert len(value) == int.from_bytes(file_state[12:20],'big')
    values[keys[1].decode()] = value
assert set(values) == {'mode','mtime'}
actual = {'mode_octal':format(int.from_bytes(values['mode'],'big'),'o'),
          'mtime_seconds':int.from_bytes(values['mtime'][:8],'big',signed=True),
          'mtime_nanoseconds':int.from_bytes(values['mtime'][8:],'big')}
manifest = prepared/'input-manifest.tsv'
fields = next(line.split('\t') for line in manifest.read_text().splitlines() if line.startswith('.\t'))
expected = {'mode_octal':fields[3],'mtime_seconds':int(fields[4]),'mtime_nanoseconds':int(fields[5])}
repo = next(p for p in Path(__file__).resolve().parents if (p/'.git').exists())
source_path = 'benchmark/fs-bench-pro/src/workspace_bench.rs'
source = {}
for revision in ('2b12825d8','6c0769116'):
    text = subprocess.check_output(['git','show',revision+':'+source_path],cwd=repo).decode()
    function = text[text.index('fn native_preparation('):text.index('fn reference_info(')]
    source[revision] = hashlib.sha256(function.encode()).hexdigest()
assert len(set(source.values())) == 1
result = {'status':'CONFIRMED_PREPARATION_DEFECT' if actual != expected else 'MATCH',
    'original_failure_proof':str(proof_path),'failure_proof_sha256':sha(proof_path),
    'failure_error':proof.get('error'),'proof_source_identity':proof.get('source_identity'),
    'prepared_master':str(prepared),'store_sha256':sha(store),
    'recorded_store_sha256':proof['setup_observation']['master_store_sha256'],
    'manifest_sha256':sha(manifest),'expected_root_metadata':expected,'actual_root_metadata':actual,
    'canonical_ids':{'namespace_root':root_id.hex(),'root_inode':root_inode.hex(),'root_inode_record':record_id.hex(),'root_metadata':metadata_id.hex()},
    'unchanged_native_preparation_function_sha256':source,
    'cause':'Docker cp of container input contents back into existing host directory changes root mtime; declared root metadata was not restored before import.',
    'scope':'Read-only stored-object inspection, not a replacement for authenticated product verification.',
    'fix':'Restore declared root before import; Git-only preparation compatibility revision; route all Git proofs through full semantic/reopen custody verification.'}
assert result['store_sha256'] == result['recorded_store_sha256']
print(json.dumps(result,indent=2))

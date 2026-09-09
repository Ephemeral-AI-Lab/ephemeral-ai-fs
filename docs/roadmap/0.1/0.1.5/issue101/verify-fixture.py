"""Read-only fixture audit against the preserved original history and Git source.
Not part of selected performance; never constructs or changes history.
"""
import json,hashlib,subprocess
from pathlib import Path
root=Path('/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-45mb-evidence/retained-full157-1/deepseek-full')
fixture=Path(__file__).resolve().parents[5] / 'benchmark/fs-bench-pro/families/historical_access/fixture.json';d=json.loads((Path(__file__).resolve().parent / 'fixture-v1.json').read_text())
rows=json.loads((root/'performance-result.json').read_text())['records']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
d.update(schema='historical-access-v2',contract_commit='2192797ec',store_sha256=sha(root/'host-runtime/store.sqlite'),branch_id=(root/'host-runtime/branch-id').read_text().strip(),history_result_sha256=sha(root/'performance-result.json'))
d['cases'].append({'id':'ha-metadata-worst-v1','full157_index':57,'operation':'stat','path':'.agents/notes/implemented/architecture/2026-07-19-gui-layering-and-rpc-protocol.zh.md','offset':0,'length':0,'cache':'cold'})
for c in d['cases']:
 c['id']=c['id'].replace('-v1','-v2')
 if c['id'].startswith('ha-range'):c.update(full157_index=65,offset=537371)
 r=rows[c['full157_index']-1];assert r['index']==c['full157_index']
 c.update(commit_id=r['commit_id'],source_commit=r['sha'],original_oracle_sha256=r['oracle_sha256'])
 oracle=json.loads(Path(r['oracle']).read_text());assert sha(Path(r['oracle']))==r['oracle_sha256']
 if c['operation']=='directory':
  oracle=json.loads(Path(r['oracle']).read_text());assert sha(Path(r['oracle']))==r['oracle_sha256'];c['expected']={'names':','.join(sorted(k for k in oracle if '/' not in bytes.fromhex(k).decode()))};continue
 mode,size,digest=oracle[c['path'].encode().hex()]
 c['expected']={'mode':mode,'size':size,'mtime':1000000000,'mtime_nsec':0}
 if c['operation']=='read':
  b=subprocess.check_output(['git','--git-dir=/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data/source.git','cat-file','blob',r['sha']+':'+c['path']]);assert len(b)==size and hashlib.sha256(b).hexdigest()==digest
  span=b[c['offset']:c['offset']+c['length']];assert len(span)==c['length'];c['expected']['sha256']=hashlib.sha256(span).hexdigest()
assert d == json.loads(fixture.read_text()), 'fixture differs from original source oracles'
print('PASS: 11 case fixtures independently match original source history/oracles')

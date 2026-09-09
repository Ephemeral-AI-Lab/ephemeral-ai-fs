"""Fixed original snapshot selections; prepare only selected transitions."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import uuid

INDICES = (1, 18, 36, 53, 70, 88, 105, 122, 140, 157)
CONTRACT = 'docs/roadmap/0.1/0.1.5/issue100/ten-snapshot-contract.md'
PROFILES = {
    'deepseek-stride10': (tuple(sorted(set(range(1,158,10)) | {157})), 'deepseek-stride10-v1',
        'docs/roadmap/0.1/0.1.5/issue102/campaign-v1.md'),
    'deepseek-ten': (INDICES, 'deepseek-ten-spread-v1', CONTRACT),
    'deepseek-stride3': (tuple(range(1, 158, 3)), 'deepseek-stride3-v1',
        'docs/roadmap/0.1/0.1.5/issue100/stride3-snapshot-contract.md'),
}


def inputs(data, cache, deadline, profile="deepseek-ten"):
    indices, scenario, _ = PROFILES[profile]
    from storage_smoke import MANIFEST_SHA, SOURCE_TIP, entries, encode, git, save, seal
    import runtime
    raw = (data/'checkpoint-manifest.json').read_bytes()
    if hashlib.sha256(raw).hexdigest() != MANIFEST_SHA:
        raise ValueError('original manifest identity')
    manifest = json.loads(raw)
    if manifest['tip'] != SOURCE_TIP or len(manifest['checkpoints']) != 157:
        raise ValueError('original history identity')
    assert indices[0] == 1 and indices[-1] == 157 and tuple(sorted(set(indices))) == indices
    rows = [manifest['checkpoints'][i-1] for i in indices]
    trees = []
    for row in rows:
        raw_tree = git(data/'source.git', 'ls-tree', '-rlz', '--full-tree', row['sha'], deadline=deadline)
        if hashlib.sha256(raw_tree).hexdigest() != row['manifest_sha256']:
            raise ValueError('selected original tree identity')
        trees.append(entries(raw_tree))
    key = hashlib.sha256(raw + repr(indices).encode() + Path(__file__).read_bytes()).hexdigest()
    cache.mkdir(parents=True, exist_ok=True)
    final = cache/(profile+'-'+key[:20])
    if not final.exists():
        temp = cache/('partial-'+uuid.uuid4().hex); temp.mkdir()
        previous = {}; digests = {}; result = []
        proc = subprocess.Popen(['git','--git-dir='+str(data/'source.git'),'cat-file','--batch'],
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            for index,(row,tree) in enumerate(zip(rows,trees),1):
                folder = temp/str(index); (folder/'blobs').mkdir(parents=True)
                (folder/'manifest.tsv').write_text(encode(tree))
                (folder/'previous.tsv').write_text(encode(previous))
                for path,(mode,oid,size) in tree.items():
                    deadline.require(profile+' preparation')
                    if previous.get(path) == (mode,oid,size):
                        continue
                    dest = folder/'blobs'/oid
                    if not dest.exists():
                        proc.stdin.write((oid+'\n').encode()); proc.stdin.flush()
                        header = proc.stdout.readline().split()
                        if header != [oid.encode(),b'blob',str(size).encode()]:
                            raise ValueError('source blob header')
                        body = proc.stdout.read(size)
                        if proc.stdout.read(1) != b'\n' or len(body)!=size or hashlib.sha1(b'blob '+str(size).encode()+b'\0'+body).hexdigest()!=oid:
                            raise ValueError('source blob identity')
                        digests[oid] = hashlib.sha256(body).hexdigest()
                        dest.write_bytes(body)
                expected = {}
                for path,(mode,oid,size) in tree.items():
                    expected[path] = [mode,size,digests[oid]]
                    parts = bytes.fromhex(path).split(b'/')
                    for n in range(1,len(parts)):
                        expected[b'/'.join(parts[:n]).hex()] = ['40755',0,'-']
                oracle = data/'oracles'/(row['sha']+'.json')
                if json.loads(oracle.read_text()) != expected:
                    raise ValueError('selected original full-byte oracle mismatch')
                result.append({**row,'index':index,'full157_index':row['index'],
                    'input':str(final/str(index)),'oracle':str(oracle),
                    'input_seal':seal(folder),'oracle_sha256':runtime.file_sha256(oracle)})
                previous = tree
            proc.stdin.close()
            if proc.wait(timeout=30):
                raise RuntimeError(proc.stderr.read().decode())
            save(temp/'fixture.json',{profile:{'input':'-','states':result,
                 'scenario':scenario,'full157_indices':list(indices),
                 'manifest_sha256':MANIFEST_SHA,'generator_sha256':runtime.file_sha256(Path(__file__))}})
            for p in temp.rglob('*'):
                p.chmod(0o555 if p.is_dir() else 0o444)
            temp.chmod(0o555); temp.rename(final)
        finally:
            if proc.poll() is None: proc.kill(); proc.wait()
            proc.stdout.close(); proc.stderr.close()
            if not proc.stdin.closed: proc.stdin.close()
    fixture = json.loads((final/'fixture.json').read_text())
    states = fixture[profile]['states']
    if [r['full157_index'] for r in states] != list(indices):
        raise ValueError('cached selection changed')
    previous = {}
    for state,original,tree in zip(states,rows,trees):
        folder=Path(state['input'])
        if folder != final/str(state['index']) or folder.is_symlink() or seal(folder)!=state['input_seal']:
            raise ValueError('cached selected input changed')
        if state['sha']!=original['sha'] or state['tree']!=original['tree'] or (folder/'manifest.tsv').read_text()!=encode(tree) or (folder/'previous.tsv').read_text()!=encode(previous):
            raise ValueError('cached selected transition changed')
        if runtime.file_sha256(Path(state['oracle']))!=state['oracle_sha256']:
            raise ValueError('original oracle changed')
        previous=tree
    return fixture

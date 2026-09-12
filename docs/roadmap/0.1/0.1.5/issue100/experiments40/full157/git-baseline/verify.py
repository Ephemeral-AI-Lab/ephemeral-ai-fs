"""Read-only existing Git157 control verification against an explicit current history."""
import argparse, collections, fcntl, hashlib, importlib.util, json, os, pathlib, struct, subprocess, time

MANIFEST_SHA = '03f21acfb415907f521217e7a972ed512265c8d0c2da0f8034e2ff3014334271'
SOURCE_TIP = 'b0a7d2ce3b4c19d7452e364b2d7acbfa87e707ed'


def sha(path):
    with path.open('rb') as stream: return hashlib.file_digest(stream, 'sha256').hexdigest()


def inventory(repo):
    assert repo.is_dir() and not repo.is_symlink(), 'regular existing Git repository required'
    result = {}
    for path in sorted(repo.rglob('*')):
        assert not path.is_symlink(), 'Git control contains a symlink'
        if path.is_file():
            stat = path.stat()
            result[str(path.relative_to(repo))] = dict(bytes=stat.st_size, allocated=stat.st_blocks*512, sha256=sha(path))
    return result


def mappings(checkpoints, mapping, fixture):
    assert len(mapping) == len(checkpoints) == len(fixture) == 157, 'all157 mappings required'
    bound = []
    for index, (mapped, original, produced) in enumerate(zip(mapping, checkpoints, fixture), 1):
        assert (mapped['index'],mapped['source_sha'],mapped['tree']) == (index,original['sha'],original['tree']) == (produced['index'],produced['sha'],produced['tree']), 'checkpoint mapping mismatch'
        assert original['index'] == produced['full157_index'] == index, 'original checkpoint ordinal mismatch'
        oracle = pathlib.Path(produced['oracle'])
        assert sha(oracle) == produced['oracle_sha256'], 'original oracle changed'
        bound.append({**mapped, 'oracle':str(oracle)})
    return bound


def main(run, data, output):
    started = time.monotonic_ns()
    base = data/'git-snapshots-157-v1'; repo = base/'snapshots.git'
    before = inventory(repo)
    assert not (repo/'objects/info/alternates').exists(), 'Git alternates prohibited'
    assert not any((repo/'objects').glob('[0-9a-f][0-9a-f]/*')), 'existing packed control has loose objects'
    manifest = data/'checkpoint-manifest.json'
    assert sha(manifest) == MANIFEST_SHA, 'pinned manifest changed'
    checkpoints = json.loads(manifest.read_text())
    identity = json.loads((run/'identity.json').read_text())
    assert identity['smoke'] == 'deepseek-full' and identity.get('storage_compact') is False, 'ordinary full157 run required'
    performance = json.loads((run/'deepseek-full/performance-result.json').read_text())
    assert performance['status'] == performance['cleanup_status'] == 'PASS', 'current full157 performance incomplete'
    assert [r['index'] for r in performance['records']] == list(range(1,158)), 'current full157 states incomplete'
    seals = json.loads((run/'performance-manifest.json').read_text())
    assert seals['deepseek-full/performance-result.json'] == sha(run/'deepseek-full/performance-result.json'), 'current performance seal changed'
    original = json.loads((base/'results.json').read_text())
    assert original['status'] == 'PASS' and original['source_tip'] == checkpoints['tip'] == SOURCE_TIP
    assert original['manifest_sha256'] == MANIFEST_SHA
    mapping = mappings(checkpoints['checkpoints'], json.loads((base/'mapping.json').read_text()), identity['fixtures']['deepseek-full']['states'])
    for observed, expected in zip(performance['records'], mapping):
        assert (observed['index'],observed['full157_index'],observed['sha'],observed['tree']) == (expected['index'],expected['index'],expected['source_sha'],expected['tree']), 'measured history differs from Git selection'
    output.mkdir(parents=True, exist_ok=False)
    print('Git157: current original checkpoint mappings verified', flush=True)
    packs=list((repo/'objects/pack').glob('*.pack')); assert len(packs) == 1, 'one existing final pack required'
    pack=packs[0]; idx=pack.with_suffix('.idx')
    with (output/'verify-pack.txt').open('xb') as stream:
        subprocess.run(['git','verify-pack','-v',str(idx)],stdout=stream,stderr=subprocess.PIPE,check=True,timeout=120,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'})
    classes=collections.defaultdict(collections.Counter); ids=set(); maxdepth=collections.Counter()
    with (output/'verify-pack.txt').open() as stream:
        for line in stream:
            fields=line.split()
            if len(fields) not in (5,7) or len(fields[0]) != 40 or fields[1] not in ('blob','tree','commit','tag'): continue
            assert fields[0] not in ids, 'duplicate packed object'
            ids.add(fields[0]); category=fields[1]+('_delta' if len(fields)==7 else '_full')
            classes[category].update(objects=1,verify_pack_reported_size_bytes=int(fields[2]),packed_bytes=int(fields[3]))
            if len(fields)==7: maxdepth[fields[1]]=max(maxdepth[fields[1]],int(fields[5]))
    objects={line.split()[0] for line in (base/'snapshot-object-ids.txt').read_text().splitlines()}
    assert ids == objects|{m['git_commit'] for m in mapping} and len(ids) == 110081, 'exact selected object membership'
    assert sum(c['packed_bytes'] for c in classes.values())+32 == pack.stat().st_size, 'pack byte conservation'
    index=idx.read_bytes(); assert index[:8] == b'\xfftOc\0\0\0\2'
    count=struct.unpack_from('>I',index,8+255*4)[0]
    assert count == len(ids) and len(index) == 1072+28*count, 'fixed pack index grammar'
    print('Git157: checking original oracles, exact membership and synthetic ancestry', flush=True)
    helper=pathlib.Path(__file__).resolve().parents[3]/'git_ten_control.py'
    spec=importlib.util.spec_from_file_location('git_snapshot_control',helper)
    control=importlib.util.module_from_spec(spec); spec.loader.exec_module(control)
    # Existing generic verifier checks refs/parents, exact all-object membership,
    # fsck and every original path/mode/symlink/content oracle. It performs no pack.
    proof=control.verify(repo,mapping,objects,output,'pack_delta')
    assert inventory(repo) == before, 'Git repository changed during read-only verification'
    apparent=sum(v['bytes'] for v in before.values()); allocated=sum(v['allocated'] for v in before.values())
    saved=original['phases']['pack_delta']['storage']
    live=dict(apparent_bytes=apparent,allocated_bytes=allocated,file_count=len(before),pack_bytes=pack.stat().st_size,index_bytes=len(index))
    layout_matches=all(live[k] == saved[k] for k in live)
    result=dict(status='PASS',scope='Existing Git157 control, read-only; no repack, reconstruction, checkout or timing comparison with foreground LayerFS writes.',
        manifest_sha256=MANIFEST_SHA,mapping_sha256=sha(base/'mapping.json'),checkpoints=157,source_tip=SOURCE_TIP,
        current_run=str(run),current_identity_sha256=sha(run/'identity.json'),current_performance_manifest_sha256=sha(run/'performance-manifest.json'),
        current_performance_result_sha256=sha(run/'deepseek-full/performance-result.json'),
        verification=proof,objects=len(ids),git_verify_pack='PASS',repository_files_before_after=before,
        live_storage=live,recorded_storage=saved,recorded_packing_policies=original['policies'],
        layout_matches_recorded=layout_matches,warnings=[] if layout_matches else ['Current repository allocation/layout differs from the recorded observation; report both, never normalize.'],
        classes=dict(classes),max_delta_depth=dict(maxdepth),attribution=dict(
            content_pack_bytes=sum(v['packed_bytes'] for k,v in classes.items() if k.startswith('blob')),
            tree_commit_pack_bytes=sum(v['packed_bytes'] for k,v in classes.items() if not k.startswith('blob')),
            pack_header_trailer_bytes=32,index_bytes=len(index),other_apparent_bytes=apparent-pack.stat().st_size-len(index),
            allocation_difference_bytes=allocated-apparent),elapsed_ns=time.monotonic_ns()-started,
        script_sha256=sha(pathlib.Path(__file__)),oracle_helper_sha256=sha(helper))
    assert sum(result['attribution'].values()) == allocated, 'complete Git allocation conservation'
    with (output/'result.json').open('x') as stream: json.dump(result,stream,indent=2); stream.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k != 'repository_files_before_after'},sort_keys=True),flush=True)
    return result


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run',type=pathlib.Path,required=True,help='current complete ordinary full157 performance run')
    parser.add_argument('--data',type=pathlib.Path,default=pathlib.Path('/Users/yifanxu/Ephemeral-AI-Lab/deepseek-history-data'))
    parser.add_argument('--output',type=pathlib.Path,required=True,help='new output directory; existing Git control stays untouched')
    args=parser.parse_args()
    with (pathlib.Path(os.environ.get('TMPDIR','/tmp'))/'layerfs-infra-measurement.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        main(args.run.resolve(),args.data.resolve(),args.output.resolve())

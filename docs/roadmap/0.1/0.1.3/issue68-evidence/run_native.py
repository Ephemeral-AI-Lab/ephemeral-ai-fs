"""One matched native control. Usage: run_native.py --perf PERF_JSONL --output NEW_DIR."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import uuid

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--perf', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
rows = [json.loads(line) for line in args.perf.read_text().splitlines()]
samples = [row for row in rows if row.get('kind') == 'sample']
assert len(samples) == 1, 'select exactly one performance sample'
sample = samples[0]
identity = sample['identities']
case = identity['case']
assert case in ('git-tool-100-mixed-v4', 'git-tool-500-mixed-v4')
assert identity['seed'] == 1 and identity['topology'] == 'host-store'
image = identity['image']
assert image.startswith('sha256:')
prepared = Path(sample['preparation']['host_root'])
assert prepared.is_dir()
root = args.output.resolve()
root.mkdir(parents=True, exist_ok=False)
(root / 'layerfs-perf.jsonl').write_bytes(args.perf.read_bytes())
(root / 'matched-identities.json').write_text(json.dumps(identity, indent=2))
calls = []

def run(argv, *, timeout=60, check=True):
    started = time.monotonic_ns()
    result = subprocess.run(argv, capture_output=True, timeout=timeout)
    number = len(calls)
    (root / f'host-{number:02d}.stdout').write_bytes(result.stdout)
    (root / f'host-{number:02d}.stderr').write_bytes(result.stderr)
    calls.append({'args': argv, 'wall_ns': time.monotonic_ns() - started,
                  'returncode': result.returncode})
    (root / 'host-commands.json').write_text(json.dumps(calls, indent=2))
    if check and result.returncode:
        raise RuntimeError(f'{argv}: {result.stderr.decode(errors="replace")}')
    return result

lock_path = Path(os.environ.get('TMPDIR', '/tmp')) / 'layerfs-infra-measurement.lock'
with lock_path.open('a') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    info = json.loads(run(['docker', 'image', 'inspect', image]).stdout)[0]
    assert info['Id'] == image
    assert info['Config']['Labels']['dev.layerfs.product-seal'] == identity['product_identity']
    name = 'layerfs-git-native-' + uuid.uuid4().hex[:12]
    created = False
    try:
        run(['docker', 'create', '--name', name, '--cpus', '2', '--memory', '2g',
             '--memory-swap', '2g', '--pids-limit', '256', '--entrypoint', '/bin/sleep',
             image, 'infinity'])
        created = True
        run(['docker', 'start', name])
        run(['docker', 'exec', name, 'mkdir', '/control'])
        run(['docker', 'cp', str(prepared / 'reference/input'), name + ':/control/reference'])
        run(['docker', 'cp', str(prepared / 'input-manifest.tsv'), name + ':/control/input-manifest.tsv'])
        run(['docker', 'cp', str(Path(__file__).with_name('native_control.py')), name + ':/control/native_control.py'])
        container = json.loads(run(['docker', 'inspect', name]).stdout)[0]
        config = container['HostConfig']
        assert not container['Mounts'] and config['NanoCpus'] == 2_000_000_000
        assert config['Memory'] == config['MemorySwap'] == 2 * 1024**3
        assert config['PidsLimit'] == 256
        (root / 'container.json').write_text(json.dumps(container, indent=2))
        result = run(['docker', 'exec', name, 'python3', '/control/native_control.py',
                      str(identity['tier'])], timeout=600)
        (root / 'native-result.json').write_bytes(result.stdout)
    finally:
        if created:
            run(['docker', 'cp', name + ':/control/evidence', str(root)], check=False)
            run(['docker', 'rm', '-f', name])
            assert run(['docker', 'inspect', name], check=False).returncode != 0
            (root / 'cleanup.json').write_text('{"status":"PASS","container_removed":true}\n')
        hashes = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in root.rglob('*') if p.is_file() and p.name != 'manifest.json'}
        (root / 'manifest.json').write_text(json.dumps(hashes, indent=2))
print(root)

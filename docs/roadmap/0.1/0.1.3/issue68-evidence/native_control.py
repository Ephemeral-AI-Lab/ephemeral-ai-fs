"""Run inside the selected image; only ordinary filesystem/Git workload code runs here."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

root = Path('/control')
tier = int(sys.argv[1])
assert tier in (100, 500)
case = f'git-tool-{tier}-mixed-v4'
reference = root / 'reference'
prepared = root / 'prepared'
work = root / 'work'
helper = '/usr/local/bin/fs-benchmark-workload'
evidence = root / 'evidence'
evidence.mkdir()
calls = []

def run(args, *, cwd=None, env=None, timeout=300):
    started = time.monotonic_ns()
    result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, timeout=timeout)
    number = len(calls)
    calls.append({'args': args, 'cwd': str(cwd) if cwd else None,
                  'wall_ns': time.monotonic_ns() - started, 'returncode': result.returncode})
    (evidence / f'{number:02d}.stdout').write_bytes(result.stdout)
    (evidence / f'{number:02d}.stderr').write_bytes(result.stderr)
    (evidence / 'commands.json').write_text(json.dumps(calls, indent=2))
    if result.returncode:
        raise RuntimeError(f'{args}: {result.stderr.decode(errors="replace")}')
    return result.stdout

def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

setup_start = time.monotonic_ns()
# docker cp preserves host ownership; ordinary Git requires the execution uid.
run(['chown', '-R', f'{os.geteuid()}:{os.getegid()}', str(root)])
assert reference.stat().st_uid == os.geteuid()
assert (reference / 'repository/.git').stat().st_uid == os.geteuid()
parent = (reference / 'expected-parent').read_text().strip()
assert len(parent) == 40 and all(c in '0123456789abcdef' for c in parent)
prepared.mkdir()
archive = run(['git', 'archive', '--format=tar', parent], cwd=reference / 'repository')
(root / 'genesis.tar').write_bytes(archive)
run(['tar', '-xf', str(root / 'genesis.tar'), '-C', str(prepared)])
blob = Path('wide/s000-f000.dat')
shutil.copy2(reference / 'repository' / blob, prepared / blob)
manifest = []
for line in (root / 'input-manifest.tsv').read_text().splitlines()[1:]:
    fields = line.split('\t')
    assert len(fields) == 7
    path, kind, length, mode, sec, nano, sha = fields
    if path == '.git' or path.startswith('.git/'):
        continue
    assert not Path(path).is_absolute() and '..' not in Path(path).parts
    target = prepared / path
    if kind == 'directory':
        target.mkdir(parents=True, exist_ok=True)
    else:
        assert kind == 'file' and target.stat().st_size == int(length)
        assert digest(target) == sha, path
    manifest.append(fields)
for path, kind, length, mode, sec, nano, sha in reversed(manifest):
    target = prepared / path
    os.chmod(target, int(mode, 8))
    timestamp = int(sec) * 10**9 + int(nano)
    os.utime(target, ns=(timestamp, timestamp))
run([helper, 'workspace-git-prepare', str(prepared), '1'])
assert run(['git', 'rev-parse', 'HEAD'], cwd=prepared).decode().strip() == parent
shutil.copytree(prepared, work, copy_function=shutil.copy2)
assert (prepared / '.git/index').stat().st_ino != (work / '.git/index').stat().st_ino
assert digest(prepared / '.git/index') == digest(work / '.git/index')
for path, kind, length, mode, sec, nano, sha in manifest:
    attr = (work / path).stat()
    assert attr.st_mode & 0o7777 == int(mode, 8), path
    assert attr.st_mtime_ns == int(sec) * 10**9 + int(nano), path
environment = run(['sh', '-c', 'git --version; uname -a; stat -f -c %T /control/work; cat /sys/fs/cgroup/cpu.max /sys/fs/cgroup/memory.max /sys/fs/cgroup/memory.swap.max /sys/fs/cgroup/pids.max; sha256sum /usr/local/bin/fs-benchmark-workload /usr/bin/git']).decode()
setup_ns = time.monotonic_ns() - setup_start
env = dict(os.environ, LAYERFS_V013_GIT_REFERENCE=str(reference))
cpu_before = Path('/sys/fs/cgroup/cpu.stat').read_text()
output = run([helper, 'workspace-apply', case, '1', '0', 'performance'], cwd=work, env=env)
cpu_after = Path('/sys/fs/cgroup/cpu.stat').read_text()
receipt = dict(line.split('=', 1) for line in output.decode().splitlines())
assert receipt['workload_status'] == 'pass'
assert int(receipt['git_process_count']) == 6
assert int(receipt['completed_target_count']) == tier
assert int(receipt['benchmark_verifier_count']) == 0
verify_start = time.monotonic_ns()
for filename, revision in [('expected-head', 'HEAD'), ('expected-tree', 'HEAD^{tree}'), ('expected-parent', 'HEAD^')]:
    assert run(['git', 'rev-parse', revision], cwd=work).decode().strip() == (reference / filename).read_text().strip()
run(['git', 'fsck', '--strict'], cwd=work)
# Independent full source-tree expected bytes/modes/mtimes, after the timed work.
run([helper, 'workspace-git-verify', case, '1', str(reference)], cwd=work, env=env, timeout=45)
verification_ns = time.monotonic_ns() - verify_start
resources = run(['sh', '-c', 'cat /sys/fs/cgroup/memory.peak /sys/fs/cgroup/memory.swap.current /sys/fs/cgroup/memory.events /sys/fs/cgroup/cpu.stat']).decode()
result = {'case': case, 'seed': 1, 'samples': 1, 'admission_eligible': False,
          'arm': 'native-container-filesystem', 'filesystem_environment': environment,
          'cache_policy': 'prepared warm; copied repository invalidates index inode/ctime; no pre-timing git status',
          'cpu_before_workload': cpu_before, 'cpu_after_workload': cpu_after,
          'setup_ns': setup_ns, 'workload_receipt': receipt, 'verification_status': 'PASS',
          'verification_ns': verification_ns, 'resources': resources,
          'genesis_head': parent, 'input_manifest_sha256': digest(root / 'input-manifest.tsv')}
(evidence / 'result.json').write_text(json.dumps(result, indent=2))
print(json.dumps(result))

"""Tiny fixture check for the v0.1.5 artifact helper; writes only under a temp dir."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile

helper = Path(__file__).with_name('prepare_artifacts.py')
spec = importlib.util.spec_from_file_location('artifacts', helper)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
with tempfile.TemporaryDirectory(prefix='layerfs-015-artifact-check-') as directory:
    root = Path(directory)
    repo = root / 'repo'
    repo.mkdir()
    module.REPO = repo
    def git(*args):
        return subprocess.check_output(['git', '-C', str(repo), *args], stderr=subprocess.DEVNULL)
    git('init')
    for scope in module.EVIDENCE:
        path = repo / scope
        if not path.suffix:
            path = path / 'README.md'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('fixture\n')
    for name, content in [('Cargo.toml', '[workspace.package]\nversion="0.1.5"\n'), ('Cargo.lock', 'lock\n'), ('LICENSE', 'license\n')]:
        (repo / name).write_text(content)
    git('add', '.')
    git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-m', 'fixture')
    commit = module.resolve('HEAD', True)
    try:
        module.resolve('HEAD', False)
    except subprocess.CalledProcessError:
        pass
    else:
        raise AssertionError('missing release tag accepted')
    (repo / 'untracked-secret').write_text('must not archive\n')
    output = root / 'candidate'
    sys.argv = [str(helper), str(output), '--ref', commit, '--candidate']
    module.main()
    assert module.validate(output, commit, True)['validation'] == 'PASS'
    try:
        module.main()
    except FileExistsError:
        pass
    else:
        raise AssertionError('existing output overwritten')
    (output / 'LICENSE').write_text('tampered\n')
    try:
        module.validate(output, commit, True)
    except ValueError:
        pass
    else:
        raise AssertionError('checksum corruption accepted')
    git('tag', 'v0.1.5')
    assert module.resolve('refs/tags/v0.1.5', False) == commit
print('artifact helper fixture check: PASS')
